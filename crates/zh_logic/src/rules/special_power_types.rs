//! `PKT-135` & `PKT-136`: Zero Hour `SpecialPowerType` & `SpecialPowerMaskType`.

#![allow(unused_variables, dead_code)]

use zh_common::types::Int;
use zh_common::xfer::bit_flags::BitFlags;

pub const SPECIALPOWER_COUNT: usize = 67;

pub type SpecialPowerMaskType = BitFlags<SPECIALPOWER_COUNT>;

/// Skeleton enum: populate all 67 Zero Hour `SpecialPowerType` variants (`PKT-136`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpecialPowerType {
    #[default]
    Invalid = 0,
    Count = SPECIALPOWER_COUNT as i32,
}

impl SpecialPowerType {
    pub fn from_i32(v: Int) -> Self {
        let _ = v;
        todo!("PKT-136: implement SpecialPowerType::from_i32")
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Skeleton table: populate exact 67 `SpecialPowerMaskType::s_bitNameList` entries from `SpecialPower.cpp`.
pub const SPECIAL_POWER_NAMES: [&str; SPECIALPOWER_COUNT] = [""; SPECIALPOWER_COUNT];
