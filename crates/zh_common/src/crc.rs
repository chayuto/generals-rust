//! `PKT-001`: Byte-wise rotate-left-add `CRC` (`Core/GameEngine/Include/Common/crc.h`).
//!
//! Target module for `WP-1.2a` (`PKT-001`).

use crate::types::{Int, UnsignedInt};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Crc {
    crc: UnsignedInt,
}

impl Crc {
    #[inline]
    pub const fn new() -> Self {
        Self { crc: 0 }
    }

    #[inline]
    pub const fn with_initial(crc: UnsignedInt) -> Self {
        Self { crc }
    }

    /// Computes the byte-wise rotate-left-add CRC for `buf[..len]`, folded into `self.crc`.
    /// If `len < 1` or `buf.is_empty()`, leaves `self.crc` unchanged.
    #[inline]
    pub fn compute_crc(&mut self, buf: &[u8], len: Int) {
        if buf.is_empty() || len < 1 {
            return;
        }
        let count = (len as usize).min(buf.len());
        for i in 0..count {
            let val = buf[i];
            let hibit = if (self.crc & 0x8000_0000) != 0 { 1u32 } else { 0u32 };
            self.crc = (self.crc << 1).wrapping_add(val as u32).wrapping_add(hibit);
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.crc = 0;
    }

    #[inline]
    pub const fn get(&self) -> UnsignedInt {
        self.crc
    }

    #[cfg(feature = "retail-crc")]
    #[inline]
    pub fn set(&mut self, v: UnsignedInt) {
        self.crc = v;
    }
}
