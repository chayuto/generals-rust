//! Deterministic `Xfer`, `XferCrc`, `XferSave`, `XferLoad`, and `Snapshot` serialization contracts
//! matching `Core/GameEngine/Source/Common/System/XferCRC.cpp`.

use crate::types::{Bool, Int, ObjectId, Real, UnsignedByte, UnsignedInt, UnsignedShort, XferVersion};

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
}

pub trait Xfer {
    fn mode(&self) -> XferMode;
    fn xfer_bytes(&mut self, buf: &mut [u8]) -> Result<(), XferError>;

    #[inline]
    fn xfer_version(&mut self, version: &mut XferVersion, current_version: XferVersion) -> Result<(), XferError> {
        let mut b = [*version];
        self.xfer_bytes(&mut b)?;
        *version = b[0];
        if self.mode() == XferMode::Load && *version > current_version {
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
    fn xfer_object_id(&mut self, v: &mut ObjectId) -> Result<(), XferError> {
        self.xfer_int(&mut v.0)
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
}

/// In-memory binary snapshot writer matching `XferSave`.
#[derive(Debug, Clone, Default)]
pub struct XferSave {
    bytes: Vec<u8>,
}

impl XferSave {
    #[inline]
    pub fn new() -> Self {
        Self { bytes: Vec::new() }
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
}

/// In-memory binary snapshot reader matching `XferLoad`.
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
}
