//! Checks a makecode .bin before it is sent (sdk/include/lib/ota/fwinfo.h).

/// OTA slot size of this project's firmware (two slots, config in the last 16 KB).
pub const FW_MAX: usize = 0x7C000;

#[derive(Clone, Debug)]
pub struct FwInfo {
    pub size: usize,
    pub sdk_version: u32,
    pub svn: u32,
    pub date: String,
    pub chipid: u16,
}

impl FwInfo {
    pub fn version_str(&self) -> String {
        let v = self.sdk_version;
        format!("v{}.{}.{}.{}-{}", v >> 24, (v >> 16) & 0xff, (v >> 8) & 0xff, v & 0xff, self.svn)
    }
}

/// Header chain: boot header (`69 5A`, byte 3 = bytes after the first 4), then `func, size, ...`
/// records (`2 + size` bytes each); func 2 is the firmware info. Fields are little-endian.
pub fn parse(b: &[u8]) -> Result<FwInfo, String> {
    if b.len() < 1024 || b[0..2] != [0x69, 0x5a] {
        return Err("not a TXW8301 firmware image (no 69 5A boot header). Use the makecode \
                    txw8301_v..._.bin, not txw8301.bin"
            .into());
    }
    if b.len() > FW_MAX {
        return Err(format!("image is {} bytes; the OTA slot holds {} bytes", b.len(), FW_MAX));
    }
    let mut off = 4 + b[3] as usize;
    for _ in 0..8 {
        if off + 2 > b.len() || off > 4096 {
            break;
        }
        let (func, size) = (b[off], b[off + 1] as usize);
        if func == 2 && off + 2 + 14 <= b.len() {
            let h = &b[off + 2..];
            let u32le = |o: usize| u32::from_le_bytes([h[o], h[o + 1], h[o + 2], h[o + 3]]);
            let date = u32le(8);
            return Ok(FwInfo {
                size: b.len(),
                sdk_version: u32le(0),
                svn: u32le(4),
                // day, month, year (u16)
                date: format!("{:04}-{:02}-{:02}", date >> 16, (date >> 8) & 0xff, date & 0xff),
                chipid: u16::from_le_bytes([h[12], h[13]]),
            });
        }
        off += 2 + size;
    }
    Err("firmware info header not found".into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_header_chain() {
        // Layout of a real build: boot header (32), SPI info (func 1, 72), firmware info (func 2).
        let mut b = vec![0u8; 2048];
        b[0..4].copy_from_slice(&[0x69, 0x5a, 0x00, 0x1c]);
        b[32..34].copy_from_slice(&[0x01, 0x48]);
        b[106..108].copy_from_slice(&[0x02, 0x17]);
        b[108..124].copy_from_slice(&[
            0x03, 0x01, 0x04, 0x02, 0x9d, 0xab, 0x00, 0x00, 0x14, 0x0b, 0xea, 0x07, 0x02, 0x40, 0x01, 0x00,
        ]);
        let f = parse(&b).unwrap();
        assert_eq!(f.version_str(), "v2.4.1.3-43933");
        assert_eq!(f.chipid, 0x4002);
        assert_eq!(f.date, "2026-11-20");
    }

    #[test]
    fn rejects_plain_bin() {
        assert!(parse(&[0u8; 4096]).is_err());
    }
}
