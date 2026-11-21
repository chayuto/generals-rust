//! `PKT-153`: Unit power flags and name table
//! (`GeneralsMD/Code/GameEngine/Include/GameLogic/Powers.h`).

use zh_common::types::Int;

pub const POWERS_NUM_POWERS: usize = 4;

/// Zero Hour `PowerNames[]` lookup table (`Powers.h`).
pub const POWER_NAMES: [&str; POWERS_NUM_POWERS] = [
    "NONE",
    "FASTER",
    "DOUBLE_SHOT",
    "SELF_HEALING",
];

/// Unit power flag indices matching `enum` in `Powers.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum UnitPower {
    #[default]
    None = 0,
    Faster = 1,
    DoubleShot = 2,
    SelfHealing = 3,
}

impl UnitPower {
    /// Converts a raw integer discriminant into `UnitPower`.
    #[inline]
    pub const fn from_i32(value: Int) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Faster),
            2 => Some(Self::DoubleShot),
            3 => Some(Self::SelfHealing),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }

    /// Returns the canonical INI token name for this unit power.
    #[inline]
    pub const fn name(self) -> &'static str {
        POWER_NAMES[self as usize]
    }
}
