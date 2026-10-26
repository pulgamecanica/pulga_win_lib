//! Capturing what's on screen as a PNG: for documentation screenshots and
//! visual checks.
//!
//! [`capture`] reads the frame just drawn back from GL; [`encode_png`]
//! turns RGBA pixels into a PNG file's bytes. The encoder is our own and
//! deliberately minimal: it stores the image uncompressed inside the PNG
//! container (deflate "stored" blocks), so it needs no compression library.
//! Files are bigger than a compressing encoder's, which is fine for
//! screenshots.
//!
//! [`App`](crate::App) can do this for you: set the `PULGA_SCREENSHOT`
//! environment variable to a file path and the app saves its frame there
//! after a moment, then quits (see [`crate::run`]).

/// Reads the `width`×`height` pixels of the frame being drawn, as RGBA rows
/// from the top (GL stores them bottom-up; this flips them).
///
/// # Safety
/// Requires a current GL context. Call after drawing, before swapping
/// buffers.
pub unsafe fn capture(width: u32, height: u32) -> Vec<u8> {
    let (w, h) = (width as usize, height as usize);
    let mut pixels = vec![0u8; w * h * 4];
    unsafe {
        gl::PixelStorei(gl::PACK_ALIGNMENT, 1);
        gl::ReadPixels(
            0,
            0,
            width as i32,
            height as i32,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            pixels.as_mut_ptr().cast(),
        );
    }
    let row = w * 4;
    let mut flipped = Vec::with_capacity(pixels.len());
    for y in (0..h).rev() {
        flipped.extend_from_slice(&pixels[y * row..(y + 1) * row]);
    }
    flipped
}

fn crc32(chunks: &[&[u8]]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for byte in chunks.iter().flat_map(|c| c.iter()) {
        crc ^= *byte as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    out.extend_from_slice(&crc32(&[kind, data]).to_be_bytes());
}

/// A PNG file holding `rgba` (`width * height * 4` bytes, rows from the
/// top).
///
/// # Panics
/// If `rgba` has the wrong length.
pub fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let row = width as usize * 4;
    assert_eq!(
        rgba.len(),
        row * height as usize,
        "RGBA image needs width * height * 4 bytes"
    );

    // Raw scanlines, each prefixed with filter type 0 (none).
    let mut raw = Vec::with_capacity((row + 1) * height as usize);
    for line in rgba.chunks(row) {
        raw.push(0);
        raw.extend_from_slice(line);
    }

    // zlib stream of stored (uncompressed) deflate blocks, 65535 bytes max each.
    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = if raw.is_empty() {
        vec![&[]]
    } else {
        raw.chunks(65535).collect()
    };
    for (i, block) in blocks.iter().enumerate() {
        zlib.push(u8::from(i == blocks.len() - 1)); // BFINAL, BTYPE = stored
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut header = Vec::with_capacity(13);
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, 6, 0, 0, 0]); // 8-bit RGBA, no interlace
    chunk(&mut png, b"IHDR", &header);
    chunk(&mut png, b"IDAT", &zlib);
    chunk(&mut png, b"IEND", &[]);
    png
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums_match_known_values() {
        assert_eq!(crc32(&[b"IEND"]), 0xae42_6082); // every PNG's IEND CRC
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }

    #[test]
    fn png_has_signature_header_and_all_pixel_bytes() {
        let rgba: Vec<u8> = (0..2 * 3 * 4).map(|i| i as u8).collect();
        let png = encode_png(2, 3, &rgba);
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&png[12..16], b"IHDR");
        assert_eq!(u32::from_be_bytes(png[16..20].try_into().unwrap()), 2);
        assert_eq!(u32::from_be_bytes(png[20..24].try_into().unwrap()), 3);
        assert!(png.windows(rgba.len() / 3).any(|w| w == &rgba[..8]));
        assert_eq!(&png[png.len() - 8..png.len() - 4], b"IEND");
    }

    #[test]
    fn big_images_split_into_several_stored_blocks() {
        let rgba = vec![7u8; 200 * 200 * 4]; // > 65535 raw bytes
        let png = encode_png(200, 200, &rgba);
        let raw_len: usize = (200 * 4 + 1) * 200;
        // signature + IHDR + zlib header/adler + one 5-byte header per block + IEND
        let blocks = raw_len.div_ceil(65535);
        assert_eq!(png.len(), 8 + 25 + 12 + 2 + 4 + raw_len + 5 * blocks + 12);
    }
}
