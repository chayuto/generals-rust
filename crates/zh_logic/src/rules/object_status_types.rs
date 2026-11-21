//! `PKT-129`: Zero Hour `ObjectStatusType` & `ObjectStatusMaskType` (`ObjectStatusTypes.h/.cpp`).

#![allow(unused_variables, dead_code)]

use zh_common::types::{Bool, Int};
use zh_common::xfer::bit_flags::BitFlags;

pub const OBJECT_STATUS_COUNT: usize = 45;

pub type ObjectStatusMaskType = BitFlags<OBJECT_STATUS_COUNT>;

/// Skeleton enum: populate all 45 Zero Hour `ObjectStatusTypes` variants (`PKT-129`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectStatusType {
    #[default]
    None = 0,
    Count = OBJECT_STATUS_COUNT as i32,
}

impl ObjectStatusType {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        todo!("PKT-129: implement ObjectStatusType::from_i32")
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Skeleton table: populate exact 45 `ObjectStatusMaskType::s_bitNameList` entries.
pub const OBJECT_STATUS_NAMES: [&str; OBJECT_STATUS_COUNT] = [""; OBJECT_STATUS_COUNT];

pub fn make_object_status_mask(t: ObjectStatusType) -> ObjectStatusMaskType {
    ObjectStatusMaskType::from_single_index(t as Int)
}

pub fn test_object_status_mask(mask: &ObjectStatusMaskType, t: ObjectStatusType) -> Bool {
    mask.test(t as Int)
}

pub fn object_status_mask_none() -> ObjectStatusMaskType {
    ObjectStatusMaskType::new()
}
