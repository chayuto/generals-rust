//! `PKT-127`: Zero Hour `ModelConditionFlagType` & `ModelConditionFlags` (`ModelState.h` & `BitFlags.cpp`).

#![allow(unused_variables, dead_code)]

use zh_common::types::Int;
use zh_common::xfer::bit_flags::{BitFlags, MODEL_CONDITION_BIT_NAMES};

pub const MODELCONDITION_COUNT: usize = 117;

pub type ModelConditionFlags = BitFlags<MODELCONDITION_COUNT>;
pub const MODEL_CONDITION_NAMES: [&str; MODELCONDITION_COUNT] = MODEL_CONDITION_BIT_NAMES;

/// Skeleton enum: populate all 117 Zero Hour `ModelConditionFlagType` variants from `ModelState.h` (`PKT-127`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModelConditionFlagType {
    #[default]
    Invalid = -1,
    Toppled = 0,
    Count = MODELCONDITION_COUNT as i32,
}

impl ModelConditionFlagType {
    pub fn from_i32(v: Int) -> Self {
        let _ = v;
        todo!("PKT-127: implement ModelConditionFlagType::from_i32")
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}
