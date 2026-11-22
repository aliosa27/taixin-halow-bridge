//! Background jobs. Each opens its own capture on the adapter, reports through `Out`, and ends.

use crate::pcap::Capture;
use crate::proto::{self, Mac, ScanReport};
use crate::udpproto as up;
use eframe::egui;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub enum Msg {
    Log(String),
    OtaFound(ScanReport),
    OtaScanDone,
    Progress(f32),
    FlashDone(Result<(), String>),
    AtFound(Mac, [u8; 4]),
    AtScanDone,
    AtText(String),
    AtDone,
    NlFound(Mac, [u8; 4]),
    NlScanDone,
    NlText(String),
    NlStopped,
}

#[derive(Clone)]
pub struct Out {
    pub tx: Sender<Msg>,
    pub ui: egui::Context,
}

impl Out {
    pub fn send(&self, m: Msg) {
        let _ = self.tx.send(m);
        self.ui.request_repaint();
    }
    pub fn log(&self, s: impl Into<String>) {
        self.send(Msg::Log(s.into()));
    }
}

/// Random locally administered MAC / 6-byte cookie.
pub fn random6() -> Mac {
    let t = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0) as u64;
    let mut x = t ^ (std::process::id() as u64).rotate_left(32) ^ 0x9e37_79b9_7f4a_7c15;
    x ^= x >> 33;
    x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
    x ^= x >> 33;
    let b = x.to_le_bytes();
    [0x02, 0x48, b[0], b[1], b[2], b[3]]
}

const READ_TIMEOUT: Duration = Duration::from_millis(30);

fn open(out: &Out, dev: &str, filter: &str) -> Option<Capture> {
    match Capture::open(dev, filter, READ_TIMEOUT) {
        Ok(c) => Some(c),
        Err(e) => {
            out.log(format!("Cannot open adapter: {e}"));
            None
        }
    }
}

fn ota_filter() -> String {
    format!("ether proto 0x{:04x}", proto::ETHERTYPE)
}

/// Broadcast SCAN a few times over `time` and call `on` for every frame captured.
fn listen(cap: &Capture, out: &Out, time: Duration, repeat: Duration, frame: &[u8], mut on: impl FnMut(&[u8])) {
    let start = Instant::now();
    let mut last: Option<Instant> = None;
    while start.elapsed() < time {
        if last.map_or(true, |t| t.elapsed() >= repeat) {
            if let Err(e) = cap.send(frame) {
                out.log(format!("Send failed: {e}"));
                return;
            }
            last = Some(Instant::now());
        }
        match cap.next() {
            Ok(Some(f)) => on(&f),
            Ok(None) => {}
            Err(e) => {
                out.log(format!("Capture failed: {e}"));
                return;
            }
        }
    }
}

// ---------------- OTA ----------------

pub fn ota_scan(dev: String, src: Mac, out: Out) {
    if let Some(cap) = open(&out, &dev, &ota_filter()) {
        let mut seen: Vec<Mac> = Vec::new();
        listen(&cap, &out, Duration::from_millis(1500), Duration::from_millis(400), &proto::scan(&src), |f| {
            if let Some(r) = proto::parse_scan_report(f) {
                if !seen.contains(&r.mac) {
                    seen.push(r.mac);
                    out.send(Msg::OtaFound(r));
                }
            }
        });
        out.log(format!("OTA scan: {} device(s).", seen.len()));
    }
    out.send(Msg::OtaScanDone);
}

// Timing follows github.com/I-AM-ENGINEER/RNode_Halow_OTA_Flasher (hgic_flash.py), which is
// patient enough for far-end devices over the HaLow link.
/// Data per FW_DATA frame: 34 header bytes + 1400 fits a 1514-byte Ethernet frame.
const CHUNK: usize = 1400;
/// Sends per chunk before the transfer is given up.
const TRIES: u32 = 10;
/// Wait for a chunk's reply. A reply can be slow while the device erases a 4 KB sector,
/// and longer when it comes back over HaLow.
const RESP_TIMEOUT: Duration = Duration::from_millis(2500);
/// Pause after a timeout before re-sending (the device may still be erasing).
const RETRY_PAUSE: Duration = Duration::from_millis(250);
/// A reply slower than this means the device was busy; pause before the next chunk.
const SLOW_REPLY: Duration = Duration::from_millis(500);
const SLOW_PAUSE: Duration = Duration::from_millis(100);
/// Whole-transfer attempts. A failed session is dropped by the device, and off 0 starts
/// a new one, so a transfer that dies part way is restarted from the beginning.
const ATTEMPTS: u32 = 3;
const ATTEMPT_PAUSE: Duration = Duration::from_secs(3);

enum FlashErr {
    /// Worth restarting the transfer from 0.
    Retry(String),
    /// Cancelled, or the device rejected the image itself.
    Fatal(String),
}

impl From<String> for FlashErr {
    fn from(e: String) -> Self {
        FlashErr::Fatal(e)
    }
}

/// Stop-and-wait upload (docs/wnb-ota-protocol.md). The last chunk commits the image.
/// `reboot`: REBOOT flags to send afterwards, or None to leave the device running.
pub fn flash(dev: String, src: Mac, target: ScanReport, image: Vec<u8>, reboot: Option<u32>, cancel: Arc<AtomicBool>, out: Out) {
    let r = flash_inner(&dev, src, &target, &image, reboot, &cancel, &out);
    out.send(Msg::FlashDone(r));
}

fn flash_inner(dev: &str, src: Mac, t: &ScanReport, image: &[u8], reboot: Option<u32>, cancel: &AtomicBool, out: &Out) -> Result<(), String> {
    let cap = Capture::open(dev, &ota_filter(), READ_TIMEOUT)?;
    let started = Instant::now();
    let mut attempt = 1;
    let retries = loop {
        out.log(format!(
            "Flashing {} bytes to {} (attempt {attempt} of {ATTEMPTS}) ...",
            image.len(),
            proto::mac_str(&t.mac)
        ));
        match send_image(&cap, src, t, image, cancel, out) {
            Ok(r) => break r,
            Err(FlashErr::Fatal(e)) => return Err(e),
            Err(FlashErr::Retry(e)) if attempt < ATTEMPTS => {
                out.log(format!("{e}; restarting from the beginning in {} s.", ATTEMPT_PAUSE.as_secs()));
                out.send(Msg::Progress(0.0));
                let until = Instant::now() + ATTEMPT_PAUSE;
                while Instant::now() < until {
                    if cancel.load(Ordering::Relaxed) {
                        return Err(cancelled(0));
                    }
                    std::thread::sleep(Duration::from_millis(100));
                }
                attempt += 1;
            }
            Err(FlashErr::Retry(e)) => return Err(format!("{e} (gave up after {ATTEMPTS} attempts)")),
        }
    };

    out.log(format!(
        "Image written and verified in {:.1} s ({} chunk retries).",
        started.elapsed().as_secs_f32(),
        retries
    ));
    if let Some(flags) = reboot {
        // The old firmware is still running and handles this, so with LOADDEF it is the
        // old firmware that clears the settings; the new one then boots on defaults.
        for _ in 0..3 {
            cap.send(&proto::reboot(&t.mac, &src, flags))?;
            std::thread::sleep(Duration::from_millis(100));
        }
        out.log(reboot_msg(flags, "Scan again in about 10 s to see the new version."));
    }
    Ok(())
}

fn cancelled(off: u32) -> String {
    format!("cancelled at {off} bytes; the device keeps running its current firmware")
}

/// One pass over the image. Returns the number of chunk re-sends.
fn send_image(cap: &Capture, src: Mac, t: &ScanReport, image: &[u8], cancel: &AtomicBool, out: &Out) -> Result<u32, FlashErr> {
    let total = image.len() as u32;
    let mut retries = 0u32;

    for (i, chunk) in image.chunks(CHUNK).enumerate() {
        let off = (i * CHUNK) as u32;
        let frame = proto::fw_data(&t.mac, &src, t.chipid, off, total, chunk);
        let mut acked = None;
        for attempt in 0..TRIES {
            if cancel.load(Ordering::Relaxed) {
                return Err(FlashErr::Fatal(cancelled(off)));
            }
            if attempt > 0 {
                retries += 1;
                std::thread::sleep(RETRY_PAUSE);
            }
            let sent = Instant::now();
            cap.send(&frame)?;
            let deadline = sent + RESP_TIMEOUT;
            while Instant::now() < deadline {
                let Some(f) = cap.next()? else { continue };
                let Some((stype, status)) = proto::parse_hdr(&f) else { continue };
                if stype != proto::STYPE_FW_DATA_RESP || f.len() < 24 || f[6..12] != t.mac {
                    continue;
                }
                if u32::from_be_bytes([f[20], f[21], f[22], f[23]]) != off {
                    continue; // late reply to an earlier retry
                }
                if status != 0 {
                    let e = format!(
                        "device rejected the chunk at {off} (status 0x{status:02x}): bad checksum, chip ID, or flash write"
                    );
                    // A rejected first chunk is the image itself (marker, customer ID);
                    // later the device has dropped the session and a restart may work.
                    return Err(if off == 0 { FlashErr::Fatal(e) } else { FlashErr::Retry(e) });
                }
                acked = Some(sent.elapsed());
                break;
            }
            if acked.is_some() {
                break;
            }
        }
        match acked {
            None => return Err(FlashErr::Retry(format!("no reply at {off} bytes after {TRIES} tries"))),
            Some(d) if d > SLOW_REPLY => std::thread::sleep(SLOW_PAUSE),
            Some(_) => {}
        }
        out.send(Msg::Progress((off as usize + chunk.len()) as f32 / total as f32));
    }
    Ok(retries)
}

fn reboot_msg(flags: u32, then: &str) -> String {
    if flags & proto::REBOOT_LOADDEF != 0 {
        format!("Reboot with settings reset sent. {then} The device comes back on its defaults (AP mode, 192.168.123.1 or DHCP); boards without a factory MAC get a new one.")
    } else {
        format!("Reboot sent. {then}")
    }
}

pub fn reboot(dev: String, src: Mac, mac: Mac, flags: u32, out: Out) {
    if let Some(cap) = open(&out, &dev, &ota_filter()) {
        let r = (0..3).try_for_each(|_| {
            std::thread::sleep(Duration::from_millis(50));
            cap.send(&proto::reboot(&mac, &src, flags))
        });
        match r {
            Ok(()) => out.log(reboot_msg(flags, &format!("({})", proto::mac_str(&mac)))),
            Err(e) => out.log(format!("Reboot failed: {e}")),
        }
    }
}

// ---------------- NETAT ----------------

fn netat_filter() -> String {
    format!("udp port {}", up::NETAT_PORT)
}

pub fn netat_scan(dev: String, ip: [u8; 4], src: Mac, out: Out) {
    if let Some(cap) = open(&out, &dev, &netat_filter()) {
        let cookie = random6();
        let req = up::udp_broadcast(&src, ip, up::NETAT_PORT, &up::netat(up::AT_SCAN_REQ, &proto::BROADCAST, &cookie, &[]));
        let mut seen: Vec<Mac> = Vec::new();
        listen(&cap, &out, Duration::from_millis(1500), Duration::from_millis(500), &req, |f| {
            let Some((from, p)) = up::udp_payload(f, up::NETAT_PORT) else { return };
            let Some(a) = up::parse_netat(p) else { return };
            if a.cmd == up::AT_SCAN_RESP && a.dest == cookie && !seen.contains(&a.src) {
                seen.push(a.src);
                out.send(Msg::AtFound(a.src, from));
            }
        });
        out.log(format!("NETAT scan: {} device(s).", seen.len()));
    }
    out.send(Msg::AtScanDone);
}

pub fn netat_cmd(dev: String, ip: [u8; 4], src: Mac, dest: Mac, cmd: String, out: Out) {
    if let Some(cap) = open(&out, &dev, &netat_filter()) {
        let cookie = random6();
        let req = up::udp_broadcast(&src, ip, up::NETAT_PORT, &up::netat(up::AT_REQ, &dest, &cookie, cmd.as_bytes()));
        let start = Instant::now();
        let mut done_at: Option<Instant> = None;
        let mut got = false;
        if let Err(e) = cap.send(&req) {
            out.log(format!("Send failed: {e}"));
        } else {
            // Replies may come in several packets; stop shortly after OK/ERROR, or after 3 s.
            while start.elapsed() < Duration::from_secs(3) && done_at.map_or(true, |t| t.elapsed() < Duration::from_millis(300)) {
                match cap.next() {
                    Ok(Some(f)) => {
                        let Some((_, p)) = up::udp_payload(&f, up::NETAT_PORT) else { continue };
                        let Some(a) = up::parse_netat(p) else { continue };
                        if a.cmd == up::AT_RESP && a.dest == cookie {
                            let s = String::from_utf8_lossy(a.data).trim_end_matches('\0').to_string();
                            if s.contains("OK") || s.contains("ERROR") {
                                done_at.get_or_insert(Instant::now());
                            }
                            got = true;
                            out.send(Msg::AtText(s));
                        }
                    }
                    Ok(None) => {}
                    Err(e) => {
                        out.log(format!("Capture failed: {e}"));
                        break;
                    }
                }
            }
            if !got {
                out.send(Msg::AtText("(no reply)".into()));
            }
        }
    }
    out.send(Msg::AtDone);
}

// ---------------- NETLOG ----------------

fn netlog_filter() -> String {
    format!("udp port {}", up::NETLOG_PORT)
}

pub fn netlog_scan(dev: String, ip: [u8; 4], src: Mac, out: Out) {
    if let Some(cap) = open(&out, &dev, &netlog_filter()) {
        let req = up::udp_broadcast(&src, ip, up::NETLOG_PORT, &up::netlog(up::LOG_DISCOVERY, &proto::BROADCAST, &src));
        let mut seen: Vec<Mac> = Vec::new();
        listen(&cap, &out, Duration::from_millis(1500), Duration::from_millis(500), &req, |f| {
            let Some((from, p)) = up::udp_payload(f, up::NETLOG_PORT) else { return };
            let Some((kind, sig, data)) = up::parse_netlog(p) else { return };
            if kind != up::LOG_DISCOVERY_RESP {
                return;
            }
            // The device's signature is in the data when present (netlog.py), else the header.
            let mut dev_sig = sig;
            if data.len() >= 6 {
                dev_sig.copy_from_slice(&data[..6]);
            }
            if !seen.contains(&dev_sig) {
                seen.push(dev_sig);
                out.send(Msg::NlFound(dev_sig, from));
            }
        });
        out.log(format!("NETLOG scan: {} device(s).", seen.len()));
    }
    out.send(Msg::NlScanDone);
}

/// Streams the device's log while heartbeats keep the session open; stops when `stop` is set.
pub fn netlog_stream(dev: String, ip: [u8; 4], src: Mac, device: Mac, stop: Arc<AtomicBool>, out: Out) {
    if let Some(cap) = open(&out, &dev, &netlog_filter()) {
        let hb = up::udp_broadcast(&src, ip, up::NETLOG_PORT, &up::netlog(up::LOG_HEARTBEAT, &device, &src));
        let mut last: Option<Instant> = None;
        out.log(format!("NETLOG: following {}.", proto::mac_str(&device)));
        while !stop.load(Ordering::Relaxed) {
            if last.map_or(true, |t| t.elapsed() >= Duration::from_millis(500)) {
                if let Err(e) = cap.send(&hb) {
                    out.log(format!("Send failed: {e}"));
                    break;
                }
                last = Some(Instant::now());
            }
            match cap.next() {
                Ok(Some(f)) => {
                    let Some((_, p)) = up::udp_payload(&f, up::NETLOG_PORT) else { continue };
                    let Some((kind, sig, data)) = up::parse_netlog(p) else { continue };
                    if kind == up::LOG_DATA && sig == src && data.len() > 6 {
                        out.send(Msg::NlText(String::from_utf8_lossy(&data[6..]).into_owned()));
                    }
                }
                Ok(None) => {}
                Err(e) => {
                    out.log(format!("Capture failed: {e}"));
                    break;
                }
            }
        }
    }
    out.send(Msg::NlStopped);
}
