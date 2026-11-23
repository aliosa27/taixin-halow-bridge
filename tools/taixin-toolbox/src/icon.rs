// App icon, drawn in code so there is no image file to keep in sync: a blue rounded square with
// white signal arcs rising from an amber dot. Used for the window icon (main.rs) and, through
// build.rs (which includes this file), for the Windows .exe icon.

/// RGBA pixels of the icon at `size` x `size`, anti-aliased by 4x4 supersampling.
pub fn rgba(size: u32) -> Vec<u8> {
    const SS: u32 = 4;
    let mut out = Vec::with_capacity((size * size * 4) as usize);
    for py in 0..size {
        for px in 0..size {
            let mut acc = [0.0f32; 4]; // premultiplied
            for sy in 0..SS {
                for sx in 0..SS {
                    let x = (px as f32 + (sx as f32 + 0.5) / SS as f32) / size as f32;
                    let y = (py as f32 + (sy as f32 + 0.5) / SS as f32) / size as f32;
                    let [r, g, b, a] = sample(x, y);
                    acc[0] += r * a;
                    acc[1] += g * a;
                    acc[2] += b * a;
                    acc[3] += a;
                }
            }
            let n = (SS * SS) as f32;
            let a = acc[3] / n;
            let un = |c: f32| if a > 0.0 { (c / n / a * 255.0).round().clamp(0.0, 255.0) as u8 } else { 0 };
            out.extend_from_slice(&[un(acc[0]), un(acc[1]), un(acc[2]), (a * 255.0).round() as u8]);
        }
    }
    out
}

/// Colour at (x, y) in the unit square, as [r, g, b, a] in 0..1.
fn sample(x: f32, y: f32) -> [f32; 4] {
    // Rounded square, radius 0.22, with a small margin.
    let (m, r) = (0.03, 0.22);
    let qx = (x - 0.5).abs() - (0.5 - m - r);
    let qy = (y - 0.5).abs() - (0.5 - m - r);
    let outside = (qx.max(0.0).powi(2) + qy.max(0.0).powi(2)).sqrt() + qx.max(qy).min(0.0) - r;
    if outside > 0.0 {
        return [0.0; 4];
    }

    // Signal: arcs around (0.5, 0.70), within 42 degrees of straight up, and a dot at the centre.
    let (cx, cy) = (0.5, 0.70);
    let (dx, dy) = (x - cx, y - cy);
    let d = (dx * dx + dy * dy).sqrt();
    if d < 0.075 {
        return [1.0, 0.70, 0.0, 1.0]; // amber dot
    }
    let up = dy < 0.0 && dx.abs() < -dy * 0.9;
    if up {
        for ring in [0.20, 0.33, 0.46] {
            if (d - ring).abs() < 0.04 {
                return [1.0, 1.0, 1.0, 1.0];
            }
        }
    }

    // Background: blue, a little lighter at the top.
    let t = y.clamp(0.0, 1.0);
    let top = [0.12, 0.47, 0.85];
    let bottom = [0.05, 0.28, 0.63];
    [
        top[0] + (bottom[0] - top[0]) * t,
        top[1] + (bottom[1] - top[1]) * t,
        top[2] + (bottom[2] - top[2]) * t,
        1.0,
    ]
}

/// A Windows .ico with 32-bit BMP entries (which every Windows version reads), sizes 16, 32, 48
/// and 256.
#[allow(dead_code)] // used by build.rs only
pub fn ico() -> Vec<u8> {
    let sizes = [16u32, 32, 48, 256];
    let images: Vec<Vec<u8>> = sizes.iter().map(|&s| bmp_entry(s)).collect();
    let mut out = Vec::new();
    out.extend_from_slice(&[0, 0, 1, 0]);
    out.extend_from_slice(&(sizes.len() as u16).to_le_bytes());
    let mut offset = 6 + 16 * sizes.len() as u32;
    for (s, img) in sizes.iter().zip(&images) {
        let b = if *s >= 256 { 0 } else { *s as u8 };
        out.extend_from_slice(&[b, b, 0, 0]);
        out.extend_from_slice(&1u16.to_le_bytes()); // planes
        out.extend_from_slice(&32u16.to_le_bytes()); // bits per pixel
        out.extend_from_slice(&(img.len() as u32).to_le_bytes());
        out.extend_from_slice(&offset.to_le_bytes());
        offset += img.len() as u32;
    }
    for img in images {
        out.extend_from_slice(&img);
    }
    out
}

/// One .ico image: BITMAPINFOHEADER (double height for the mask), BGRA rows bottom-up, then an
/// all-zero AND mask (the alpha channel does the work).
fn bmp_entry(size: u32) -> Vec<u8> {
    let px = rgba(size);
    let mask_row = ((size + 31) / 32) * 4;
    let mut out = Vec::new();
    out.extend_from_slice(&40u32.to_le_bytes());
    out.extend_from_slice(&(size as i32).to_le_bytes());
    out.extend_from_slice(&((size * 2) as i32).to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&32u16.to_le_bytes());
    out.extend_from_slice(&[0; 24]); // compression, image size, resolution, colours
    for y in (0..size).rev() {
        for x in 0..size {
            let i = ((y * size + x) * 4) as usize;
            out.extend_from_slice(&[px[i + 2], px[i + 1], px[i], px[i + 3]]);
        }
    }
    out.extend(std::iter::repeat(0u8).take((mask_row * size) as usize));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ico_layout() {
        let ico = ico();
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 4, 0]);
        // The last entry ends at the end of the file.
        let e = 6 + 16 * 3;
        let len = u32::from_le_bytes(ico[e + 8..e + 12].try_into().unwrap()) as usize;
        let off = u32::from_le_bytes(ico[e + 12..e + 16].try_into().unwrap()) as usize;
        assert_eq!(off + len, ico.len());
    }

    /// `ICON_PNG=path cargo test write_png` writes a 256 px PNG preview (uncompressed deflate).
    #[test]
    fn write_png() {
        let Ok(path) = std::env::var("ICON_PNG") else { return };
        let s = 256u32;
        let px = rgba(s);
        let mut raw = Vec::new();
        for y in 0..s as usize {
            raw.push(0);
            raw.extend_from_slice(&px[y * s as usize * 4..(y + 1) * s as usize * 4]);
        }
        let mut z = vec![0x78, 0x01];
        for (i, block) in raw.chunks(65535).enumerate() {
            let last = (i + 1) * 65535 >= raw.len();
            z.push(last as u8);
            z.extend_from_slice(&(block.len() as u16).to_le_bytes());
            z.extend_from_slice(&(!(block.len() as u16)).to_le_bytes());
            z.extend_from_slice(block);
        }
        let (mut a, mut b) = (1u32, 0u32);
        for &c in &raw {
            a = (a + c as u32) % 65521;
            b = (b + a) % 65521;
        }
        z.extend_from_slice(&((b << 16) | a).to_be_bytes());
        fn crc(d: &[u8]) -> u32 {
            let mut c = !0u32;
            for &x in d {
                c ^= x as u32;
                for _ in 0..8 {
                    c = if c & 1 != 0 { 0xEDB8_8320 ^ (c >> 1) } else { c >> 1 };
                }
            }
            !c
        }
        let mut png = vec![0x89, b'P', b'N', b'G', 13, 10, 26, 10];
        let mut chunk = |t: &[u8], d: &[u8]| {
            png.extend_from_slice(&(d.len() as u32).to_be_bytes());
            let mut td = t.to_vec();
            td.extend_from_slice(d);
            png.extend_from_slice(&td);
            png.extend_from_slice(&crc(&td).to_be_bytes());
        };
        let mut ihdr = Vec::new();
        ihdr.extend_from_slice(&s.to_be_bytes());
        ihdr.extend_from_slice(&s.to_be_bytes());
        ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
        chunk(b"IHDR", &ihdr);
        chunk(b"IDAT", &z);
        chunk(b"IEND", &[]);
        std::fs::write(path, png).unwrap();
    }
}
