//! `PKT-135`: Zero Hour `SpecialPowerMaskType` (`Common/SpecialPowerType.h` & `SpecialPower.cpp`).
//!
//! Manifest-aligned re-export of the canonical bitflag mask definition in
//! [`crate::rules::special_power_types`], preserving all existing `zh_logic::rules::*` paths.

pub use crate::rules::special_power_types::{
    SpecialPowerMaskType, SPECIALPOWER_COUNT, SPECIAL_POWER_NAMES,
};
