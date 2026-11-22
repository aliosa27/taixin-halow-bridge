//! Minimal libpcap binding, loaded at run time: Npcap's wpcap.dll on Windows, the system libpcap on
//! macOS/Linux. Loading at run time means no pcap SDK at build time and a clear message when Npcap
//! is missing.

use libloading::Library;
use std::ffi::{c_char, c_int, c_long, c_uint, c_void, CStr, CString};
use std::sync::OnceLock;
use std::time::Duration;

#[repr(C)]
struct PcapIf {
    next: *mut PcapIf,
    name: *mut c_char,
    description: *mut c_char,
    addresses: *mut PcapAddr,
    flags: c_uint,
}

#[repr(C)]
struct PcapAddr {
    next: *mut PcapAddr,
    addr: *mut u8, // struct sockaddr
    netmask: *mut u8,
    broadaddr: *mut u8,
    dstaddr: *mut u8,
}

/// IPv4 address from a struct sockaddr, if it is AF_INET (2). BSD sockaddrs start with a length
/// byte, Windows ones with a 16-bit family.
unsafe fn sockaddr_ipv4(sa: *const u8) -> Option<[u8; 4]> {
    if sa.is_null() {
        return None;
    }
    #[cfg(any(target_os = "macos", target_os = "freebsd"))]
    let family = *sa.add(1) as u16;
    #[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
    let family = u16::from_ne_bytes([*sa, *sa.add(1)]);
    if family != 2 {
        return None;
    }
    Some([*sa.add(4), *sa.add(5), *sa.add(6), *sa.add(7)])
}

#[repr(C)]
struct BpfProgram {
    bf_len: c_uint,
    bf_insns: *mut c_void,
}

#[repr(C)]
struct Timeval {
    tv_sec: c_long,
    tv_usec: c_long,
}

#[repr(C)]
struct PktHdr {
    ts: Timeval,
    caplen: u32,
    len: u32,
}

type FindAllDevs = unsafe extern "C" fn(*mut *mut PcapIf, *mut c_char) -> c_int;
type FreeAllDevs = unsafe extern "C" fn(*mut PcapIf);
type OpenLive = unsafe extern "C" fn(*const c_char, c_int, c_int, c_int, *mut c_char) -> *mut c_void;
type Compile = unsafe extern "C" fn(*mut c_void, *mut BpfProgram, *const c_char, c_int, u32) -> c_int;
type SetFilter = unsafe extern "C" fn(*mut c_void, *mut BpfProgram) -> c_int;
type FreeCode = unsafe extern "C" fn(*mut BpfProgram);
type SendPacket = unsafe extern "C" fn(*mut c_void, *const u8, c_int) -> c_int;
type NextEx = unsafe extern "C" fn(*mut c_void, *mut *mut PktHdr, *mut *const u8) -> c_int;
type GetErr = unsafe extern "C" fn(*mut c_void) -> *mut c_char;
type Close = unsafe extern "C" fn(*mut c_void);

struct Api {
    _lib: Library,
    findalldevs: FindAllDevs,
    freealldevs: FreeAllDevs,
    open_live: OpenLive,
    compile: Compile,
    setfilter: SetFilter,
    freecode: FreeCode,
    sendpacket: SendPacket,
    next_ex: NextEx,
    geterr: GetErr,
    close: Close,
}

const ERRBUF: usize = 256;

#[cfg(windows)]
fn load_lib() -> Result<Library, String> {
    use libloading::os::windows::{Library as WinLib, LOAD_WITH_ALTERED_SEARCH_PATH};
    // Npcap installs into System32\Npcap, which is not on the DLL search path. Loading it by full
    // path with altered search path also finds its Packet.dll.
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    let npcap = format!(r"{sysroot}\System32\Npcap\wpcap.dll");
    unsafe {
        WinLib::load_with_flags(&npcap, LOAD_WITH_ALTERED_SEARCH_PATH)
            .or_else(|_| WinLib::new("wpcap.dll"))
            .map(Library::from)
            .map_err(|e| format!("Npcap not found ({e}). Install it from https://npcap.com"))
    }
}

#[cfg(not(windows))]
fn load_lib() -> Result<Library, String> {
    let names = ["libpcap.dylib", "libpcap.A.dylib", "libpcap.so.1", "libpcap.so.0.8", "libpcap.so"];
    for n in names {
        if let Ok(l) = unsafe { Library::new(n) } {
            return Ok(l);
        }
    }
    Err("libpcap not found".into())
}

fn api() -> Result<&'static Api, String> {
    static API: OnceLock<Result<Api, String>> = OnceLock::new();
    API.get_or_init(|| {
        let lib = load_lib()?;
        unsafe {
            macro_rules! sym {
                ($n:literal) => {
                    *lib.get($n).map_err(|e| format!("pcap symbol missing: {e}"))?
                };
            }
            Ok(Api {
                findalldevs: sym!(b"pcap_findalldevs\0"),
                freealldevs: sym!(b"pcap_freealldevs\0"),
                open_live: sym!(b"pcap_open_live\0"),
                compile: sym!(b"pcap_compile\0"),
                setfilter: sym!(b"pcap_setfilter\0"),
                freecode: sym!(b"pcap_freecode\0"),
                sendpacket: sym!(b"pcap_sendpacket\0"),
                next_ex: sym!(b"pcap_next_ex\0"),
                geterr: sym!(b"pcap_geterr\0"),
                close: sym!(b"pcap_close\0"),
                _lib: lib,
            })
        }
    })
    .as_ref()
    .map_err(|e| e.clone())
}

#[derive(Clone, Debug)]
pub struct Device {
    pub name: String,
    pub description: String,
    pub loopback: bool,
    /// First IPv4 address of the adapter, used as the source of NETAT/NETLOG broadcasts.
    pub ipv4: Option<[u8; 4]>,
}

impl Device {
    pub fn label(&self) -> String {
        let ip = self.ipv4.map(|a| format!(" {}.{}.{}.{}", a[0], a[1], a[2], a[3])).unwrap_or_default();
        if self.description.is_empty() {
            format!("{}{ip}", self.name)
        } else {
            format!("{}{ip} ({})", self.description, self.name)
        }
    }
}

unsafe fn cstr(p: *const c_char) -> String {
    if p.is_null() {
        String::new()
    } else {
        CStr::from_ptr(p).to_string_lossy().into_owned()
    }
}

pub fn devices() -> Result<Vec<Device>, String> {
    let a = api()?;
    let mut err = [0 as c_char; ERRBUF];
    let mut list: *mut PcapIf = std::ptr::null_mut();
    unsafe {
        if (a.findalldevs)(&mut list, err.as_mut_ptr()) != 0 {
            return Err(cstr(err.as_ptr()));
        }
        let mut out = Vec::new();
        let mut p = list;
        while !p.is_null() {
            let d = &*p;
            let mut ipv4 = None;
            let mut a = d.addresses;
            while !a.is_null() && ipv4.is_none() {
                ipv4 = sockaddr_ipv4((*a).addr);
                a = (*a).next;
            }
            out.push(Device {
                name: cstr(d.name),
                description: cstr(d.description),
                loopback: d.flags & 1 != 0, // PCAP_IF_LOOPBACK
                ipv4,
            });
            p = d.next;
        }
        (a.freealldevs)(list);
        Ok(out)
    }
}

pub struct Capture {
    api: &'static Api,
    h: *mut c_void,
}

// The handle is used from one thread at a time (moved into the worker thread).
unsafe impl Send for Capture {}

impl Capture {
    /// Opens `dev` in promiscuous mode with a short read timeout and a BPF filter.
    pub fn open(dev: &str, filter: &str, timeout: Duration) -> Result<Capture, String> {
        let a = api()?;
        let mut err = [0 as c_char; ERRBUF];
        let name = CString::new(dev).map_err(|e| e.to_string())?;
        let h = unsafe { (a.open_live)(name.as_ptr(), 2048, 1, timeout.as_millis() as c_int, err.as_mut_ptr()) };
        if h.is_null() {
            return Err(unsafe { cstr(err.as_ptr()) });
        }
        let cap = Capture { api: a, h };
        let f = CString::new(filter).map_err(|e| e.to_string())?;
        let mut prog = BpfProgram { bf_len: 0, bf_insns: std::ptr::null_mut() };
        unsafe {
            if (a.compile)(h, &mut prog, f.as_ptr(), 1, 0xffff_ffff) != 0 {
                return Err(format!("filter: {}", cap.err()));
            }
            let r = (a.setfilter)(h, &mut prog);
            (a.freecode)(&mut prog);
            if r != 0 {
                return Err(format!("filter: {}", cap.err()));
            }
        }
        Ok(cap)
    }

    fn err(&self) -> String {
        unsafe { cstr((self.api.geterr)(self.h)) }
    }

    pub fn send(&self, frame: &[u8]) -> Result<(), String> {
        let r = unsafe { (self.api.sendpacket)(self.h, frame.as_ptr(), frame.len() as c_int) };
        if r != 0 {
            Err(self.err())
        } else {
            Ok(())
        }
    }

    /// Next frame, or None when the read timeout expires with nothing captured.
    pub fn next(&self) -> Result<Option<Vec<u8>>, String> {
        let mut hdr: *mut PktHdr = std::ptr::null_mut();
        let mut data: *const u8 = std::ptr::null();
        let r = unsafe { (self.api.next_ex)(self.h, &mut hdr, &mut data) };
        match r {
            1 => unsafe {
                let len = (*hdr).caplen as usize;
                Ok(Some(std::slice::from_raw_parts(data, len).to_vec()))
            },
            0 => Ok(None),
            _ => Err(self.err()),
        }
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        unsafe { (self.api.close)(self.h) }
    }
}

