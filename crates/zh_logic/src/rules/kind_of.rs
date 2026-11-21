//! `PKT-126`: Zero Hour `KindOfType` & `KindOfMaskType` (`KindOf.h/.cpp`).
//! Built for `-DRTS_ZEROHOUR=1 -DRTS_GENERALS=0` (with `ALLOW_SURRENDER` undefined).

#![allow(unused_variables, dead_code)]

use zh_common::types::{Bool, Int};
use zh_common::xfer::bit_flags::BitFlags;

pub const KINDOF_COUNT: usize = 117;

pub type KindOfMaskType = BitFlags<KINDOF_COUNT>;

/// Skeleton enum: populate all 117 Zero Hour `KindOfType` variants from `KindOf.h` (`PKT-126`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KindOfType {
    #[default]
    Invalid = -1,
    Obstacle = 0,
    Structure = 7,
    Count = KINDOF_COUNT as i32,
}

impl KindOfType {
    pub fn from_i32(v: Int) -> Self {
        let _ = v;
        todo!("PKT-126: implement KindOfType::from_i32")
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Skeleton table: populate exact 117 `KindOfMaskType::s_bitNameList` entries from `KindOf.cpp`.
pub const KINDOF_NAMES: [&str; KINDOF_COUNT] = [""; KINDOF_COUNT];

pub fn make_kindof_mask(k: KindOfType) -> KindOfMaskType {
    KindOfMaskType::from_single_index(k as Int)
}

pub fn test_kindof_mask(mask: &KindOfMaskType, k: KindOfType) -> Bool {
    mask.test(k as Int)
}

pub fn kindof_mask_none() -> KindOfMaskType {
    KindOfMaskType::new()
}

pub fn kindof_mask_fs() -> KindOfMaskType {
    todo!("PKT-126: implement KINDOFMASK_FS from KindOf.cpp:170-188")
}
