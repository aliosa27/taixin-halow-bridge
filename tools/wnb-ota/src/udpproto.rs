//! NETAT (UDP 56789) and NETLOG (UDP 64320), the vendor's broadcast AT console and log stream.
//! Layouts from github.com/aliosa27/taixin_tools (libnetat.c, netlog.py). Both are sent here as raw
//! Ethernet/IPv4/UDP broadcast frames, so the PC needs no address on the device's subnet.
//! Both services are off in this project's firmware (SYS_APP_NETAT / SYS_APP_NETLOG = 0); stock
//! firmware has them.

use crate::proto::{Mac, BROADCAST};

pub const NETAT_PORT: u16 = 56789;
pub const NETLOG_PORT: u16 = 64320;

// ---- Ethernet / IPv4 / UDP ----

fn ip_checksum(h: &[u8]) -> u16 {
    let mut sum: u32 = h.chunks(2).map(|w| u16::from_be_bytes([w[0], *w.get(1).unwrap_or(&0)]) as u32).sum();
    while sum >> 16 != 0 {
        sum = (sum & 0xffff) + (sum >> 16);
    }
    !(sum as u16)
}

/// Broadcast UDP frame (dst MAC ff.., dst IP 255.255.255.255), same port both sides.
pub fn udp_broadcast(src_mac: &Mac, src_ip: [u8; 4], port: u16, payload: &[u8]) -> Vec<u8> {
    let udp_len = 8 + payload.len();
    let ip_len = 20 + udp_len;
    let mut f = Vec::with_capacity(14 + ip_len);
    f.extend_from_slice(&BROADCAST);
    f.extend_from_slice(src_mac);
    f.extend_from_slice(&0x0800u16.to_be_bytes());
    let mut ip = [0u8; 20];
    ip[0] = 0x45;
    ip[2..4].copy_from_slice(&(ip_len as u16).to_be_bytes());
    ip[8] = 64; // TTL
    ip[9] = 17; // UDP
    ip[12..16].copy_from_slice(&src_ip);
    ip[16..20].copy_from_slice(&[255; 4]);
    let c = ip_checksum(&ip);
    ip[10..12].copy_from_slice(&c.to_be_bytes());
    f.extend_from_slice(&ip);
    f.extend_from_slice(&port.to_be_bytes());
    f.extend_from_slice(&port.to_be_bytes());
    f.extend_from_slice(&(udp_len as u16).to_be_bytes());
    f.extend_from_slice(&[0, 0]); // no UDP checksum (allowed for IPv4)
    f.extend_from_slice(payload);
    if f.len() < 60 {
        f.resize(60, 0);
    }
    f
}

/// UDP payload of an IPv4 frame to `port`, with the sender's IP.
pub fn udp_payload(f: &[u8], port: u16) -> Option<([u8; 4], &[u8])> {
    if f.len() < 14 + 20 + 8 || f[12..14] != [0x08, 0x00] {
        return None;
    }
    let ihl = ((f[14] & 0x0f) as usize) * 4;
    if f[14 + 9] != 17 || f.len() < 14 + ihl + 8 {
        return None;
    }
    let src_ip = [f[26], f[27], f[28], f[29]];
    let u = &f[14 + ihl..];
    if u16::from_be_bytes([u[2], u[3]]) != port {
        return None;
    }
    let len = (u16::from_be_bytes([u[4], u[5]]) as usize).clamp(8, u.len());
    Some((src_ip, &u[8..len]))
}

// ---- NETAT: cmd u8, len u16 BE, dest[6], src[6], data ----

pub const AT_SCAN_REQ: u8 = 1;
pub const AT_SCAN_RESP: u8 = 2;
pub const AT_REQ: u8 = 3;
pub const AT_RESP: u8 = 4;

pub fn netat(cmd: u8, dest: &Mac, cookie: &Mac, data: &[u8]) -> Vec<u8> {
    let mut p = vec![cmd];
    p.extend_from_slice(&(data.len() as u16).to_be_bytes());
    p.extend_from_slice(dest);
    p.extend_from_slice(cookie);
    p.extend_from_slice(data);
    p
}

pub struct NetAt<'a> {
    pub cmd: u8,
    pub dest: Mac,
    pub src: Mac,
    pub data: &'a [u8],
}

pub fn parse_netat(p: &[u8]) -> Option<NetAt<'_>> {
    if p.len() < 15 {
        return None;
    }
    let mut dest = [0; 6];
    let mut src = [0; 6];
    dest.copy_from_slice(&p[3..9]);
    src.copy_from_slice(&p[9..15]);
    Some(NetAt { cmd: p[0], dest, src, data: &p[15..] })
}

// ---- NETLOG: type u8, signature[6], data ----

pub const LOG_DISCOVERY: u8 = 1;
pub const LOG_DISCOVERY_RESP: u8 = 2;
pub const LOG_HEARTBEAT: u8 = 3;
pub const LOG_DATA: u8 = 4;

pub fn netlog(kind: u8, sig: &Mac, data: &[u8]) -> Vec<u8> {
    let mut p = vec![kind];
    p.extend_from_slice(sig);
    p.extend_from_slice(data);
    p
}

pub fn parse_netlog(p: &[u8]) -> Option<(u8, Mac, &[u8])> {
    if p.len() < 7 {
        return None;
    }
    let mut sig = [0; 6];
    sig.copy_from_slice(&p[1..7]);
    Some((p[0], sig, &p[7..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn udp_roundtrip() {
        let f = udp_broadcast(&[2, 0, 0, 0, 0, 1], [172, 16, 70, 6], NETAT_PORT, b"hello");
        assert_eq!(ip_checksum(&f[14..34]), 0); // header checksum verifies
        let (ip, p) = udp_payload(&f, NETAT_PORT).unwrap();
        assert_eq!(ip, [172, 16, 70, 6]);
        assert_eq!(p, b"hello");
        assert!(udp_payload(&f, NETLOG_PORT).is_none());
    }

    #[test]
    fn netat_layout() {
        let p = netat(AT_REQ, &[1, 2, 3, 4, 5, 6], &[9; 6], b"AT+VERSION");
        assert_eq!(&p[0..3], &[3, 0, 10]);
        let a = parse_netat(&p).unwrap();
        assert_eq!(a.dest, [1, 2, 3, 4, 5, 6]);
        assert_eq!(a.src, [9; 6]);
        assert_eq!(a.data, b"AT+VERSION");
    }
}
