//! Deterministic `Xfer`, `XferCrc`, `XferDeepCrc`, `XferSave`, `XferLoad`, `Snapshot`,
//! and `BitFlags` (`WP-2.1`: `Core/GameEngine/Source/Common/System/Xfer*.cpp`).

pub mod bit_flags;

use crate::math::{
    Coord2D, Coord3D, ICoord2D, ICoord3D, IRegion2D, IRegion3D, Matrix3D, RGBAColorInt,
    RGBAColorReal, RGBColor, RealRange, Region2D, Region3D,
};
use crate::types::{
    Bool, DrawableId, Int, ObjectId, Real, UnsignedByte, UnsignedInt, UnsignedShort, XferVersion,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XferMode {
    Save,
    Load,
    Crc,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum XferError {
    UnexpectedEof,
    VersionTooNew { found: u8, max_supported: u8 },
    InvalidData,
    StringTooLong,
    BeginEndMismatch,
    ListNotEmpty,
    InvalidParameters,
    SkipOutOfBounds,
}

/// Core bidirectional snapshot/CRC stream abstraction matching C++ `Xfer` (`Xfer.h` / `Xfer.cpp`).
pub trait Xfer {
    fn mode(&self) -> XferMode;
    fn xfer_bytes(&mut self, buf: &mut [u8]) -> Result<(), XferError>;

    fn begin_block(&mut self) -> Result<Int, XferError> {
        Ok(0)
    }

    fn end_block(&mut self) -> Result<(), XferError> {
        Ok(())
    }

    fn skip(&mut self, _data_size: Int) -> Result<(), XferError> {
        Ok(())
    }

    fn xfer_snapshot(&mut self, snapshot: &mut dyn Snapshot) -> Result<(), XferError>;

    #[inline]
    fn xfer_marker_label(&mut self, _label: &str) {}

    #[inline]
    fn xfer_version(
        &mut self,
        version: &mut XferVersion,
        current_version: XferVersion,
    ) -> Result<(), XferError> {
        let mut b = [*version];
        self.xfer_bytes(&mut b)?;
        *version = b[0];
        if *version > current_version {
            return Err(XferError::VersionTooNew {
                found: *version,
                max_supported: current_version,
            });
        }
        Ok(())
    }

    #[inline]
    fn xfer_bool(&mut self, v: &mut Bool) -> Result<(), XferError> {
        let mut b = [*v];
        self.xfer_bytes(&mut b)?;
        *v = b[0];
        Ok(())
    }

    #[inline]
    fn xfer_byte(&mut self, v: &mut i8) -> Result<(), XferError> {
        let mut b = [*v as u8];
        self.xfer_bytes(&mut b)?;
        *v = b[0] as i8;
        Ok(())
    }

    #[inline]
    fn xfer_unsigned_byte(&mut self, v: &mut UnsignedByte) -> Result<(), XferError> {
        let mut b = [*v];
        self.xfer_bytes(&mut b)?;
        *v = b[0];
        Ok(())
    }

    #[inline]
    fn xfer_short(&mut self, v: &mut i16) -> Result<(), XferError> {
        let mut b = v.to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = i16::from_le_bytes(b);
        Ok(())
    }

    #[inline]
    fn xfer_unsigned_short(&mut self, v: &mut UnsignedShort) -> Result<(), XferError> {
        let mut b = v.to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = u16::from_le_bytes(b);
        Ok(())
    }

    #[inline]
    fn xfer_int(&mut self, v: &mut Int) -> Result<(), XferError> {
        let mut b = v.to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = i32::from_le_bytes(b);
        Ok(())
    }

    #[inline]
    fn xfer_int64(&mut self, v: &mut i64) -> Result<(), XferError> {
        let mut b = v.to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = i64::from_le_bytes(b);
        Ok(())
    }

    #[inline]
    fn xfer_unsigned_int(&mut self, v: &mut UnsignedInt) -> Result<(), XferError> {
        let mut b = v.to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = u32::from_le_bytes(b);
        Ok(())
    }

    #[inline]
    fn xfer_real(&mut self, v: &mut Real) -> Result<(), XferError> {
        let mut b = v.to_bits().to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = f32::from_bits(u32::from_le_bytes(b));
        Ok(())
    }

    #[inline]
    fn xfer_double(&mut self, v: &mut f64) -> Result<(), XferError> {
        let mut b = v.to_bits().to_le_bytes();
        self.xfer_bytes(&mut b)?;
        *v = f64::from_bits(u64::from_le_bytes(b));
        Ok(())
    }

    /// Base `Xfer::xferAsciiString`: feeds raw string bytes with no length prefix (used by `XferCrc`).
    fn xfer_ascii_string(&mut self, s: &mut String) -> Result<(), XferError> {
        if !s.is_empty() {
            let mut copy = s.as_bytes().to_vec();
            self.xfer_bytes(&mut copy)?;
        }
        Ok(())
    }

    /// Base `Xfer::xferUnicodeString`: feeds all `u16` LE bytes in a single contiguous call (no length prefix, used by `XferCrc`).
    fn xfer_unicode_string(&mut self, s: &mut Vec<u16>) -> Result<(), XferError> {
        if !s.is_empty() {
            let mut buf = Vec::with_capacity(s.len() * 2);
            for &unit in s.iter() {
                buf.extend_from_slice(&unit.to_le_bytes());
            }
            self.xfer_bytes(&mut buf)?;
        }
        Ok(())
    }

    #[inline]
    fn xfer_coord2d(&mut self, c: &mut Coord2D) -> Result<(), XferError> {
        self.xfer_real(&mut c.x)?;
        self.xfer_real(&mut c.y)
    }

    #[inline]
    fn xfer_icoord2d(&mut self, c: &mut ICoord2D) -> Result<(), XferError> {
        self.xfer_int(&mut c.x)?;
        self.xfer_int(&mut c.y)
    }

    #[inline]
    fn xfer_region2d(&mut self, r: &mut Region2D) -> Result<(), XferError> {
        self.xfer_coord2d(&mut r.lo)?;
        self.xfer_coord2d(&mut r.hi)
    }

    #[inline]
    fn xfer_iregion2d(&mut self, r: &mut IRegion2D) -> Result<(), XferError> {
        self.xfer_icoord2d(&mut r.lo)?;
        self.xfer_icoord2d(&mut r.hi)
    }

    #[inline]
    fn xfer_coord3d(&mut self, c: &mut Coord3D) -> Result<(), XferError> {
        self.xfer_real(&mut c.x)?;
        self.xfer_real(&mut c.y)?;
        self.xfer_real(&mut c.z)
    }

    #[inline]
    fn xfer_icoord3d(&mut self, c: &mut ICoord3D) -> Result<(), XferError> {
        self.xfer_int(&mut c.x)?;
        self.xfer_int(&mut c.y)?;
        self.xfer_int(&mut c.z)
    }

    #[inline]
    fn xfer_region3d(&mut self, r: &mut Region3D) -> Result<(), XferError> {
        self.xfer_coord3d(&mut r.lo)?;
        self.xfer_coord3d(&mut r.hi)
    }

    #[inline]
    fn xfer_iregion3d(&mut self, r: &mut IRegion3D) -> Result<(), XferError> {
        self.xfer_icoord3d(&mut r.lo)?;
        self.xfer_icoord3d(&mut r.hi)
    }

    #[inline]
    fn xfer_real_range(&mut self, r: &mut RealRange) -> Result<(), XferError> {
        self.xfer_real(&mut r.lo)?;
        self.xfer_real(&mut r.hi)
    }

    #[inline]
    fn xfer_color(&mut self, c: &mut Int) -> Result<(), XferError> {
        self.xfer_int(c)
    }

    #[inline]
    fn xfer_rgb_color(&mut self, c: &mut RGBColor) -> Result<(), XferError> {
        self.xfer_real(&mut c.red)?;
        self.xfer_real(&mut c.green)?;
        self.xfer_real(&mut c.blue)
    }

    #[inline]
    fn xfer_rgba_color_real(&mut self, c: &mut RGBAColorReal) -> Result<(), XferError> {
        self.xfer_real(&mut c.red)?;
        self.xfer_real(&mut c.green)?;
        self.xfer_real(&mut c.blue)?;
        self.xfer_real(&mut c.alpha)
    }

    #[inline]
    fn xfer_rgba_color_int(&mut self, c: &mut RGBAColorInt) -> Result<(), XferError> {
        self.xfer_unsigned_int(&mut c.red)?;
        self.xfer_unsigned_int(&mut c.green)?;
        self.xfer_unsigned_int(&mut c.blue)?;
        self.xfer_unsigned_int(&mut c.alpha)
    }

    #[inline]
    fn xfer_object_id(&mut self, v: &mut ObjectId) -> Result<(), XferError> {
        self.xfer_int(&mut v.0)
    }

    #[inline]
    fn xfer_drawable_id(&mut self, v: &mut DrawableId) -> Result<(), XferError> {
        self.xfer_int(&mut v.0)
    }

    fn xfer_stl_object_id_vec(&mut self, vec: &mut Vec<ObjectId>) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        self.xfer_version(&mut version, 1)?;
        let mut count: UnsignedShort = vec.len() as UnsignedShort;
        self.xfer_unsigned_short(&mut count)?;
        match self.mode() {
            XferMode::Save | XferMode::Crc => {
                for item in vec.iter_mut() {
                    self.xfer_object_id(item)?;
                }
            }
            XferMode::Load => {
                if !vec.is_empty() {
                    return Err(XferError::ListNotEmpty);
                }
                for _ in 0..count {
                    let mut id = ObjectId::INVALID;
                    self.xfer_object_id(&mut id)?;
                    vec.push(id);
                }
            }
        }
        Ok(())
    }

    fn xfer_stl_int_list(&mut self, list: &mut Vec<Int>) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        self.xfer_version(&mut version, 1)?;
        let mut count: UnsignedShort = list.len() as UnsignedShort;
        self.xfer_unsigned_short(&mut count)?;
        match self.mode() {
            XferMode::Save | XferMode::Crc => {
                for item in list.iter_mut() {
                    self.xfer_int(item)?;
                }
            }
            XferMode::Load => {
                if !list.is_empty() {
                    return Err(XferError::ListNotEmpty);
                }
                for _ in 0..count {
                    let mut v: Int = 0;
                    self.xfer_int(&mut v)?;
                    list.push(v);
                }
            }
        }
        Ok(())
    }

    fn xfer_matrix3d(&mut self, mtx: &mut Matrix3D) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        self.xfer_version(&mut version, 1)?;
        for row in &mut mtx.rows {
            for cell in row {
                self.xfer_real(cell)?;
            }
        }
        Ok(())
    }

    #[inline]
    fn xfer_user(&mut self, data: &mut [u8]) -> Result<(), XferError> {
        self.xfer_bytes(data)
    }

    #[inline]
    fn xfer_enum_i32(&mut self, v: &mut i32) -> Result<(), XferError> {
        self.xfer_int(v)
    }
}

pub trait Snapshot {
    fn crc(&self, xfer: &mut dyn Xfer);
    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError>;
    fn load_post_process(&mut self) {}
}

fn decode_sage_bytes(bytes: &[u8]) -> String {
    match core::str::from_utf8(bytes) {
        Ok(s) => s.to_owned(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    }
}

/// Exact 32-bit word-at-a-time rotate-left-add `XferCRC` implementation
/// matching `Core/GameEngine/Source/Common/System/XferCRC.cpp`.
#[derive(Debug, Clone, Default)]
pub struct XferCrc {
    crc: u32,
}

impl XferCrc {
    #[inline]
    pub const fn new() -> Self {
        Self { crc: 0 }
    }

    #[inline]
    pub const fn with_initial(crc: u32) -> Self {
        Self { crc }
    }

    #[inline]
    pub fn add_crc(&mut self, val: u32) {
        #[cfg(feature = "retail-crc")]
        let v = val.swap_bytes();
        #[cfg(not(feature = "retail-crc"))]
        let v = val;

        self.crc = self.crc.rotate_left(1).wrapping_add(v);
    }

    #[inline]
    pub fn xfer_implementation(&mut self, data: &[u8]) {
        let mut chunks = data.chunks_exact(4);
        for chunk in &mut chunks {
            let word = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            self.add_crc(word);
        }
        let rem = chunks.remainder();
        if !rem.is_empty() {
            let mut val: u32 = 0;
            if rem.len() == 3 {
                val = val.wrapping_add((rem[2] as u32) << 16);
            }
            if rem.len() >= 2 {
                val = val.wrapping_add((rem[1] as u32) << 8);
            }
            val = val.wrapping_add(rem[0] as u32);
            // Note: in C++ XferCRC::xferImplementation, trailing 1..3 bytes never call htobe(val)
            self.crc = self.crc.rotate_left(1).wrapping_add(val);
        }
    }

    #[inline]
    pub const fn get_crc(&self) -> u32 {
        #[cfg(feature = "retail-crc")]
        {
            self.crc.swap_bytes()
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            self.crc
        }
    }
}

impl Xfer for XferCrc {
    #[inline]
    fn mode(&self) -> XferMode {
        XferMode::Crc
    }

    #[inline]
    fn xfer_bytes(&mut self, buf: &mut [u8]) -> Result<(), XferError> {
        self.xfer_implementation(buf);
        Ok(())
    }

    #[inline]
    fn xfer_snapshot(&mut self, snapshot: &mut dyn Snapshot) -> Result<(), XferError> {
        snapshot.crc(self);
        Ok(())
    }
}

/// Deep CRC diagnostic logger matching `XferDeepCRC` (`XferCRC.cpp:189-432`).
/// Logs every byte fed to the CRC calculation (with explicit string length headers)
/// while computing the exact `XferCrc` over that logged stream.
#[derive(Debug, Clone, Default)]
pub struct XferDeepCrc {
    inner: XferCrc,
    log_bytes: Vec<u8>,
}

impl XferDeepCrc {
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: XferCrc::new(),
            log_bytes: Vec::new(),
        }
    }

    #[inline]
    pub fn get_crc(&self) -> u32 {
        self.inner.get_crc()
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.log_bytes
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.log_bytes
    }
}

impl Xfer for XferDeepCrc {
    #[inline]
    fn mode(&self) -> XferMode {
        XferMode::Crc
    }

    #[inline]
    fn xfer_bytes(&mut self, buf: &mut [u8]) -> Result<(), XferError> {
        if buf.is_empty() {
            return Ok(());
        }
        self.log_bytes.extend_from_slice(buf);
        self.inner.xfer_implementation(buf);
        Ok(())
    }

    #[inline]
    fn xfer_snapshot(&mut self, snapshot: &mut dyn Snapshot) -> Result<(), XferError> {
        snapshot.crc(self);
        Ok(())
    }

    fn xfer_ascii_string(&mut self, s: &mut String) -> Result<(), XferError> {
        if s.len() > 16385 {
            return Err(XferError::StringTooLong);
        }
        let mut len = s.len() as UnsignedShort;
        self.xfer_unsigned_short(&mut len)?;
        if len > 0 {
            let mut copy = s.as_bytes().to_vec();
            self.xfer_user(&mut copy)?;
        }
        Ok(())
    }

    fn xfer_unicode_string(&mut self, s: &mut Vec<u16>) -> Result<(), XferError> {
        if s.len() > 255 {
            return Err(XferError::StringTooLong);
        }
        let mut len = s.len() as i8;
        self.xfer_byte(&mut len)?;
        if len > 0 {
            let mut buf = Vec::with_capacity(s.len() * 2);
            for &unit in s.iter() {
                buf.extend_from_slice(&unit.to_le_bytes());
            }
            self.xfer_user(&mut buf)?;
        }
        Ok(())
    }
}

/// In-memory binary snapshot writer matching `XferSave` (`XferSave.cpp`).
#[derive(Debug, Clone, Default)]
pub struct XferSave {
    bytes: Vec<u8>,
    block_stack: Vec<usize>,
}

impl XferSave {
    #[inline]
    pub fn new() -> Self {
        Self {
            bytes: Vec::new(),
            block_stack: Vec::new(),
        }
    }

    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    #[inline]
    pub fn into_bytes(self) -> Vec<u8> {
        self.bytes
    }
}

impl Xfer for XferSave {
    #[inline]
    fn mode(&self) -> XferMode {
        XferMode::Save
    }

    #[inline]
    fn xfer_bytes(&mut self, buf: &mut [u8]) -> Result<(), XferError> {
        self.bytes.extend_from_slice(buf);
        Ok(())
    }

    fn begin_block(&mut self) -> Result<Int, XferError> {
        let pos = self.bytes.len();
        self.bytes.extend_from_slice(&0i32.to_le_bytes());
        self.block_stack.push(pos);
        Ok(1)
    }

    fn end_block(&mut self) -> Result<(), XferError> {
        let start = self.block_stack.pop().ok_or(XferError::BeginEndMismatch)?;
        let size = (self.bytes.len() - start - 4) as Int;
        self.bytes[start..start + 4].copy_from_slice(&size.to_le_bytes());
        Ok(())
    }

    #[inline]
    fn xfer_snapshot(&mut self, snapshot: &mut dyn Snapshot) -> Result<(), XferError> {
        snapshot.xfer(self)
    }

    fn xfer_ascii_string(&mut self, s: &mut String) -> Result<(), XferError> {
        if s.len() > 255 {
            return Err(XferError::StringTooLong);
        }
        let mut len = s.len() as UnsignedByte;
        self.xfer_unsigned_byte(&mut len)?;
        if len > 0 {
            let mut copy = s.as_bytes().to_vec();
            self.xfer_user(&mut copy)?;
        }
        Ok(())
    }

    fn xfer_unicode_string(&mut self, s: &mut Vec<u16>) -> Result<(), XferError> {
        if s.len() > 255 {
            return Err(XferError::StringTooLong);
        }
        let mut len = s.len() as UnsignedByte;
        self.xfer_unsigned_byte(&mut len)?;
        if len > 0 {
            let mut buf = Vec::with_capacity(s.len() * 2);
            for &unit in s.iter() {
                buf.extend_from_slice(&unit.to_le_bytes());
            }
            self.xfer_user(&mut buf)?;
        }
        Ok(())
    }
}

/// In-memory binary snapshot reader matching `XferLoad` (`XferLoad.cpp`).
#[derive(Debug, Clone)]
pub struct XferLoad<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> XferLoad<'a> {
    #[inline]
    pub const fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    #[inline]
    pub const fn position(&self) -> usize {
        self.pos
    }
}

impl<'a> Xfer for XferLoad<'a> {
    #[inline]
    fn mode(&self) -> XferMode {
        XferMode::Load
    }

    #[inline]
    fn xfer_bytes(&mut self, buf: &mut [u8]) -> Result<(), XferError> {
        let end = self.pos.saturating_add(buf.len());
        if end > self.data.len() {
            return Err(XferError::UnexpectedEof);
        }
        buf.copy_from_slice(&self.data[self.pos..end]);
        self.pos = end;
        Ok(())
    }

    fn begin_block(&mut self) -> Result<Int, XferError> {
        let mut block_size: Int = 0;
        if self.xfer_int(&mut block_size).is_err() {
            return Ok(0);
        }
        Ok(block_size)
    }

    fn skip(&mut self, data_size: Int) -> Result<(), XferError> {
        if data_size < 0 {
            return Err(XferError::SkipOutOfBounds);
        }
        let end = self.pos.saturating_add(data_size as usize);
        if end > self.data.len() {
            return Err(XferError::SkipOutOfBounds);
        }
        self.pos = end;
        Ok(())
    }

    #[inline]
    fn xfer_snapshot(&mut self, snapshot: &mut dyn Snapshot) -> Result<(), XferError> {
        snapshot.xfer(self)
    }

    fn xfer_ascii_string(&mut self, s: &mut String) -> Result<(), XferError> {
        let mut len: UnsignedByte = 0;
        self.xfer_unsigned_byte(&mut len)?;
        if len > 0 {
            let mut buf = vec![0u8; len as usize];
            self.xfer_user(&mut buf)?;
            *s = decode_sage_bytes(&buf);
        } else {
            s.clear();
        }
        Ok(())
    }

    fn xfer_unicode_string(&mut self, s: &mut Vec<u16>) -> Result<(), XferError> {
        let mut len: UnsignedByte = 0;
        self.xfer_unsigned_byte(&mut len)?;
        s.clear();
        if len > 0 {
            let mut buf = vec![0u8; (len as usize) * 2];
            self.xfer_user(&mut buf)?;
            s.reserve(len as usize);
            for pair in buf.chunks_exact(2) {
                s.push(u16::from_le_bytes([pair[0], pair[1]]));
            }
        }
        Ok(())
    }
}
