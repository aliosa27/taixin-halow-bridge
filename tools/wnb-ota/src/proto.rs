//! WNB Ethernet OTA frames. See docs/wnb-ota-protocol.md.

pub const ETHERTYPE: u16 = 0x4847;
pub const HDR_LEN: usize = 16;

pub const STYPE_REBOOT: u8 = 1;
pub const STYPE_SCAN: u8 = 2;
pub const STYPE_SCAN_REPORT: u8 = 3;
pub const STYPE_FW_DATA: u8 = 4;
pub const STYPE_FW_DATA_RESP: u8 = 5;

pub type Mac = [u8; 6];
pub const BROADCAST: Mac = [0xff; 6];

pub fn mac_str(m: &Mac) -> String {
    m.iter().map(|b| format!("{b:02x}")).collect::<Vec<_>>().join(":")
}

fn header(dst: &Mac, src: &Mac, stype: u8) -> Vec<u8> {
    let mut f = Vec::with_capacity(64);
    f.extend_from_slice(dst);
    f.extend_from_slice(src);
    f.extend_from_slice(&ETHERTYPE.to_be_bytes());
    f.push(stype);
    f.push(0);
    f
}

/// Pad to the Ethernet minimum (60 bytes without FCS); some capture drivers do not.
fn pad(mut f: Vec<u8>) -> Vec<u8> {
    if f.len() < 60 {
        f.resize(60, 0);
    }
    f
}

pub fn scan(src: &Mac) -> Vec<u8> {
    pad(header(&BROADCAST, src, STYPE_SCAN))
}

/// REBOOT flag: the running firmware runs `syscfg_loaddef("syscfg")` before resetting.
pub const REBOOT_LOADDEF: u32 = 1;

/// flags bit 0 = load factory defaults first. The device reads this field little-endian.
pub fn reboot(dst: &Mac, src: &Mac, flags: u32) -> Vec<u8> {
    let mut f = header(dst, src, STYPE_REBOOT);
    f.extend_from_slice(&flags.to_le_bytes());
    pad(f)
}

/// libota_check_sum: RFC 1071 sum over little-endian 16-bit words, inverted.
pub fn checksum(data: &[u8]) -> u16 {
    let mut sum: u32 = 0;
    let mut chunks = data.chunks_exact(2);
    for w in &mut chunks {
        sum += u16::from_le_bytes([w[0], w[1]]) as u32;
    }
    if let [b] = chunks.remainder() {
        sum += *b as u32;
    }
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}


pub fn fw_data(dst: &Mac, src: &Mac, chipid: u16, off: u32, tot_len: u32, data: &[u8]) -> Vec<u8> {
    let mut f = header(dst, src, STYPE_FW_DATA);
    f.extend_from_slice(&3u32.to_be_bytes()); // device checks the low byte == 3
    f.extend_from_slice(&off.to_be_bytes());
    f.extend_from_slice(&tot_len.to_be_bytes());
    f.extend_from_slice(&(data.len() as u16).to_be_bytes());
    f.extend_from_slice(&checksum(data).to_be_bytes());
    f.extend_from_slice(&chipid.to_be_bytes());
    f.extend_from_slice(data);
    pad(f)
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScanReport {
    pub mac: Mac,
    pub version: u32,
    pub chipid: u16,
    pub mode: u8,
    pub svn: u32,
    pub app: u32,
}

impl ScanReport {
    pub fn version_str(&self) -> String {
        let v = self.version;
        let mut s = format!("v{}.{}.{}.{}-{}", v >> 24, (v >> 16) & 0xff, (v >> 8) & 0xff, v & 0xff, self.svn);
        if self.app != 0 {
            s += &format!(" app-{}", self.app);
        }
        s
    }

    /// The report carries the WiFi interface index, which in this firmware is the mode number.
    pub fn mode_str(&self) -> String {
        match self.mode {
            1 => "Station".into(),
            2 => "WNB Station".into(),
            3 => "Access Point".into(),
            4 => "WNB Access Point".into(),
            n => format!("mode {n}"),
        }
    }
}

fn be16(b: &[u8], o: usize) -> u16 {
    u16::from_be_bytes([b[o], b[o + 1]])
}
fn be32(b: &[u8], o: usize) -> u32 {
    u32::from_be_bytes([b[o], b[o + 1], b[o + 2], b[o + 3]])
}

/// Returns (stype, status) for any frame of our EtherType.
pub fn parse_hdr(f: &[u8]) -> Option<(u8, u8)> {
    if f.len() < HDR_LEN || be16(f, 12) != ETHERTYPE {
        return None;
    }
    Some((f[14], f[15]))
}

pub fn parse_scan_report(f: &[u8]) -> Option<ScanReport> {
    let (stype, _) = parse_hdr(f)?;
    if stype != STYPE_SCAN_REPORT || f.len() < 32 {
        return None;
    }
    let mut mac = [0u8; 6];
    mac.copy_from_slice(&f[6..12]);
    Some(ScanReport {
        mac,
        version: be32(f, 16),
        chipid: be16(f, 20),
        mode: f[22],
        svn: be32(f, 24),
        app: be32(f, 28),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_matches_rfc1071_le() {
        // words 0x0201 + 0x0403 = 0x0604 -> !0x0604
        assert_eq!(checksum(&[1, 2, 3, 4]), !0x0604u16);
        // odd byte is added as-is
        assert_eq!(checksum(&[1, 2, 5]), !(0x0201u16 + 5));
        // carry folding
        assert_eq!(checksum(&[0xff, 0xff, 0x02, 0x00]), !0x0002u16);
        assert_eq!(checksum(&[]), 0xffff);
    }

    #[test]
    fn scan_report_roundtrip() {
        let mut f = vec![0u8; 60];
        f[6..12].copy_from_slice(&[0x06, 0xbf, 0x00, 0x73, 0x58, 0xc8]);
        f[12..14].copy_from_slice(&ETHERTYPE.to_be_bytes());
        f[14] = STYPE_SCAN_REPORT;
        f[16..20].copy_from_slice(&0x0204_0103u32.to_be_bytes());
        f[20..22].copy_from_slice(&0x4002u16.to_be_bytes());
        f[22] = 4;
        f[24..28].copy_from_slice(&43933u32.to_be_bytes());
        f[28..32].copy_from_slice(&1u32.to_be_bytes());
        let r = parse_scan_report(&f).unwrap();
        assert_eq!(mac_str(&r.mac), "06:bf:00:73:58:c8");
        assert_eq!(r.version_str(), "v2.4.1.3-43933 app-1");
        assert_eq!(r.mode_str(), "WNB Access Point");
        assert_eq!(r.chipid, 0x4002);
    }

    #[test]
    fn fw_data_layout() {
        let f = fw_data(&BROADCAST, &[2, 0, 0, 0, 0, 1], 0x4002, 1024, 472592, &[1, 2, 3, 4]);
        assert_eq!(&f[12..16], &[0x48, 0x47, STYPE_FW_DATA, 0]);
        assert_eq!(be32(&f, 16), 3);
        assert_eq!(be32(&f, 20), 1024);
        assert_eq!(be32(&f, 24), 472592);
        assert_eq!(be16(&f, 28), 4);
        assert_eq!(be16(&f, 30), checksum(&[1, 2, 3, 4]));
        assert_eq!(be16(&f, 32), 0x4002);
        assert_eq!(&f[34..38], &[1, 2, 3, 4]);
    }
}
