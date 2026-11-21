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
    Count = DISABLED_COUNT as i32,
}

impl DisabledType {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        todo!("PKT-125: implement DisabledType::from_i32")
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Skeleton table: populate exact `DisabledMaskType::s_bitNameList` entries from `DisabledTypes.cpp`.
pub const DISABLED_TYPE_NAMES: [&str; DISABLED_COUNT] = [""; DISABLED_COUNT];

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
