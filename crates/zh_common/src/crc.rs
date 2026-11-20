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
    pub fn compute_crc(&mut self, _buf: &[u8], _len: Int) {
        // Pending PKT-001 translation in WP-1.2a
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
