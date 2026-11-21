//! `PKT-125`: Zero Hour `DisabledType` & `DisabledMaskType` (`DisabledTypes.h/.cpp`).

#![allow(unused_variables, dead_code)]

use zh_common::types::{Bool, Int};
use zh_common::xfer::bit_flags::BitFlags;

pub const DISABLED_COUNT: usize = 13;
pub const DISABLED_ANY_SENTINEL: Int = 65535;

pub type DisabledMaskType = BitFlags<DISABLED_COUNT>;

/// Skeleton enum: populate all Zero Hour `DisabledType` variants from `PKT-125`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisabledType {
    #[default]
    Default = 0,
    Hacked = 1,
    Emp = 2,
    Held = 3,
    Paralyzed = 4,
    Unmanned = 5,
    Underpowered = 6,
    Freefall = 7,
    Awestruck = 8,
    Brainwashed = 9,
    Subdued = 10,
    ScriptDisabled = 11,
    ScriptUnderpowered = 12,
    Count = DISABLED_COUNT as i32,
}

impl DisabledType {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Default),
            1 => Some(Self::Hacked),
            2 => Some(Self::Emp),
            3 => Some(Self::Held),
            4 => Some(Self::Paralyzed),
            5 => Some(Self::Unmanned),
            6 => Some(Self::Underpowered),
            7 => Some(Self::Freefall),
            8 => Some(Self::Awestruck),
            9 => Some(Self::Brainwashed),
            10 => Some(Self::Subdued),
            11 => Some(Self::ScriptDisabled),
            12 => Some(Self::ScriptUnderpowered),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Skeleton table: populate exact `DisabledMaskType::s_bitNameList` entries from `DisabledTypes.cpp`.
pub const DISABLED_TYPE_NAMES: [&str; DISABLED_COUNT] = [
    "DEFAULT",
    "DISABLED_HACKED",
    "DISABLED_EMP",
    "DISABLED_HELD",
    "DISABLED_PARALYZED",
    "DISABLED_UNMANNED",
    "DISABLED_UNDERPOWERED",
    "DISABLED_FREEFALL",
    "DISABLED_AWESTRUCK",
    "DISABLED_BRAINWASHED",
    "DISABLED_SUBDUED",
    "DISABLED_SCRIPT_DISABLED",
    "DISABLED_SCRIPT_UNDERPOWERED",
];

pub fn make_disabled_mask(t: DisabledType) -> DisabledMaskType {
    DisabledMaskType::from_single_index(t as Int)
}

pub fn test_disabled_mask(mask: &DisabledMaskType, t: DisabledType) -> Bool {
    mask.test(t as Int)
}

pub fn disabled_mask_none() -> DisabledMaskType {
    DisabledMaskType::new()
}

pub fn disabled_mask_all() -> DisabledMaskType {
    DisabledMaskType::new_set_all()
}
