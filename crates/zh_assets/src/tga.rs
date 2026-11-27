//! TGA images as `Targa::Load` leaves them: run-length decoded and flipped by
//! the origin bits, channel order as stored (BGR/BGRA).

use crate::chunk::{Cursor, Read, Truncated};

const ORIGIN_X: u8 = 0x10;
const ORIGIN_Y: u8 = 0x20;
const FOOTER_SIGNATURE: &[u8] = b"TRUEVISION-XFILE.\0";

#[derive(Clone, Debug)]
pub struct Tga {
    pub id_length: u8,
    pub color_map_type: u8,
    /// 1 mapped, 2 true colour, 3 grey; plus 8 when run-length encoded.
    pub image_type: u8,
    pub color_map_start: u16,
    pub color_map_length: u16,
    pub color_map_depth: u8,
    pub width: u16,
    pub height: u16,
    pub pixel_depth: u8,
    pub descriptor: u8,
    pub palette: Vec<u8>,
    pub pixels: Vec<u8>,
    /// Bytes after the image: the TGA 2.0 areas and footer, if any.
    pub trailing: usize,
    pub has_footer: bool,
}

impl Tga {
    pub fn bytes_per_pixel(&self) -> usize {
        (usize::from(self.pixel_depth) + 7) / 8
    }
}

pub fn parse(data: &[u8]) -> Read<Tga> {
    let mut c = Cursor::new(data);
    let id_length = c.u8()?;
    let color_map_type = c.u8()?;
    let image_type = c.u8()?;
    let color_map_start = c.u16()?;
    let color_map_length = c.u16()?;
    let color_map_depth = c.u8()?;
    c.u32()?; // screen origin
    let width = c.u16()?;
    let height = c.u16()?;
    let pixel_depth = c.u8()?;
    let descriptor = c.u8()?;
    c.bytes(usize::from(id_length))?;
    let palette = if color_map_type == 1 {
        let size = usize::from(color_map_length) * usize::from(color_map_depth >> 3);
        c.bytes(size)?.to_vec()
    } else {
        Vec::new()
    };
    let mut t = Tga {
        id_length,
        color_map_type,
        image_type,
        color_map_start,
        color_map_length,
        color_map_depth,
        width,
        height,
        pixel_depth,
        descriptor,
        palette,
        pixels: Vec::new(),
        trailing: 0,
        has_footer: data.ends_with(FOOTER_SIGNATURE),
    };
    let bpp = t.bytes_per_pixel();
    let count = usize::from(width) * usize::from(height);
    if bpp == 0 {
        return Err(Truncated);
    }
    t.pixels = match image_type {
        1..=3 => c.bytes(count * bpp)?.to_vec(),
        9 | 10 => decode_runs(&mut c, count, bpp)?,
        _ => return Err(Truncated),
    };
    t.trailing = c.remaining();
    if descriptor & ORIGIN_X != 0 {
        flip_x(&mut t.pixels, usize::from(width), bpp);
        t.descriptor &= !ORIGIN_X;
    }
    if descriptor & ORIGIN_Y != 0 {
        flip_y(&mut t.pixels, usize::from(width) * bpp);
        t.descriptor &= !ORIGIN_Y;
    }
    Ok(t)
}

/// `DecodeImage`. A packet running past the image is an error here; the engine
/// would write past its buffer.
fn decode_runs(c: &mut Cursor, mut left: usize, bpp: usize) -> Read<Vec<u8>> {
    let mut out = Vec::new();
    while left > 0 {
        let head = c.u8()?;
        let n = usize::from(head & 0x7f) + 1;
        if n > left {
            return Err(Truncated);
        }
        if head & 0x80 != 0 {
            let pixel = c.bytes(bpp)?;
            for _ in 0..n {
                out.extend_from_slice(pixel);
            }
        } else {
            out.extend_from_slice(c.bytes(n * bpp)?);
        }
        left -= n;
    }
    Ok(out)
}

fn flip_x(pixels: &mut [u8], width: usize, bpp: usize) {
    if width == 0 {
        return;
    }
    for row in pixels.chunks_mut(width * bpp) {
        for x in 0..width / 2 {
            for i in 0..bpp {
                row.swap(x * bpp + i, (width - 1 - x) * bpp + i);
            }
        }
    }
}

fn flip_y(pixels: &mut [u8], stride: usize) {
    if stride == 0 {
        return;
    }
    let rows = pixels.len() / stride;
    for y in 0..rows / 2 {
        for i in 0..stride {
            pixels.swap(y * stride + i, (rows - 1 - y) * stride + i);
        }
    }
}
