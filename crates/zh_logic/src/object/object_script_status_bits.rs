//! `PKT-392`: Script-controlled object status bitflags (`ObjectScriptStatusBits.h`).

use zh_common::types::UnsignedByte;

/// Bit positions for script status overrides stored in `Object::m_scriptStatus` (`Byte` / `u8`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ObjectScriptStatusBit {
    /// Object is disabled via script.
    Disabled = 0x01,
    /// Object is unpowered via script.
    Unpowered = 0x02,
    /// Object cannot be sold.
    Unsellable = 0x04,
    /// Object cannot stealth.
    Unstealthed = 0x08,
    /// Unit can be targeted by the player, but not autoacquired.
    Targetable = 0x10,
}

impl ObjectScriptStatusBit {
    #[inline]
    pub const fn as_u8(self) -> UnsignedByte {
        self as UnsignedByte
    }
}

pub const OBJECT_STATUS_SCRIPT_DISABLED: UnsignedByte = 0x01;
pub const OBJECT_STATUS_SCRIPT_UNPOWERED: UnsignedByte = 0x02;
pub const OBJECT_STATUS_SCRIPT_UNSELLABLE: UnsignedByte = 0x04;
pub const OBJECT_STATUS_SCRIPT_UNSTEALTHED: UnsignedByte = 0x08;
pub const OBJECT_STATUS_SCRIPT_TARGETABLE: UnsignedByte = 0x10;
