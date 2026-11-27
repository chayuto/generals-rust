//! DDS textures: the header and the stored mip levels, still compressed.
//! [`Dds::engine_mip_levels`] gives what `DDSFileClass` would use.

use crate::chunk::{Cursor, Read, Truncated};

const CAPS2_CUBEMAP: u32 = 0x200;
const CAPS2_VOLUME: u32 = 0x0020_0000;

#[derive(Clone, Debug)]
pub struct Dds<'a> {
    pub flags: u32,
    pub height: u32,
    pub width: u32,
    pub pitch_or_linear_size: u32,
    pub depth: u32,
    pub mip_map_count: u32,
    pub pixel_flags: u32,
    pub four_cc: [u8; 4],
    pub rgb_bit_count: u32,
    /// Red, green, blue and alpha masks.
    pub masks: [u32; 4],
    pub caps: [u32; 2],
    /// Stored surfaces: for each face, every mip level, largest first.
    pub levels: Vec<&'a [u8]>,
    pub trailing: usize,
}

impl Dds<'_> {
    pub fn is_cube_map(&self) -> bool {
        self.caps[1] & CAPS2_CUBEMAP != 0
    }

    /// Bytes per 4x4 block, 0 when the file is not block compressed.
    pub fn block_bytes(&self) -> u32 {
        match &self.four_cc {
            b"DXT1" => 8,
            b"DXT2" | b"DXT3" | b"DXT4" | b"DXT5" => 16,
            _ => 0,
        }
    }

    /// `Calculate_DXTC_Surface_Size` for compressed formats, else packed rows.
    pub fn surface_size(&self, width: u32, height: u32) -> u64 {
        match self.block_bytes() {
            0 => u64::from(width) * u64::from(height) * u64::from(self.rgb_bit_count / 8),
            b => {
                u64::from(width.div_ceil(4).max(1))
                    * u64::from(height.div_ceil(4).max(1))
                    * u64::from(b)
            }
        }
    }

    /// Width and height as `DDSFileClass::Get_Width` reports level 0.
    pub fn engine_size(&self) -> (u32, u32) {
        (self.width.max(4), self.height.max(4))
    }

    /// The engine drops the two smallest levels of a chain longer than two.
    pub fn engine_mip_levels(&self) -> u32 {
        let n = self.mip_map_count.max(1);
        if n > 2 {
            n - 2
        } else {
            1
        }
    }
}

pub fn parse(data: &[u8]) -> Read<Dds<'_>> {
    let mut c = Cursor::new(data);
    if c.bytes(4)? != b"DDS " || c.u32()? != 124 {
        return Err(Truncated);
    }
    let flags = c.u32()?;
    let height = c.u32()?;
    let width = c.u32()?;
    let pitch_or_linear_size = c.u32()?;
    let depth = c.u32()?;
    let mip_map_count = c.u32()?;
    c.bytes(44)?;
    c.u32()?; // pixel format size
    let pixel_flags = c.u32()?;
    let four_cc = c.array()?;
    let rgb_bit_count = c.u32()?;
    let masks = c.uints()?;
    let caps = c.uints()?;
    c.bytes(12)?;
    let mut d = Dds {
        flags,
        height,
        width,
        pitch_or_linear_size,
        depth,
        mip_map_count,
        pixel_flags,
        four_cc,
        rgb_bit_count,
        masks,
        caps,
        levels: Vec::new(),
        trailing: 0,
    };
    let volume = d.caps[1] & CAPS2_VOLUME != 0;
    let faces = if d.is_cube_map() {
        (d.caps[1] & 0xfc00).count_ones()
    } else {
        1
    };
    if (d.block_bytes() == 0 && d.rgb_bit_count < 8) || mip_map_count > 32 {
        return Err(Truncated);
    }
    for _ in 0..faces {
        let (mut w, mut h, mut z) = (width, height, depth.max(1));
        for _ in 0..mip_map_count.max(1) {
            let size = d.surface_size(w, h) * u64::from(if volume { z } else { 1 });
            let size = usize::try_from(size).map_err(|_| Truncated)?;
            d.levels.push(c.bytes(size)?);
            w = (w / 2).max(1);
            h = (h / 2).max(1);
            z = (z / 2).max(1);
        }
    }
    d.trailing = c.remaining();
    Ok(d)
}
