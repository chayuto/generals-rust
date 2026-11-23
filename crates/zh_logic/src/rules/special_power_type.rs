//! `PKT-136`: Zero Hour `SpecialPowerType` (`Common/SpecialPowerType.h`).
//!
//! Manifest-aligned re-export of the canonical 67-variant Zero Hour enum in
//! [`crate::rules::special_power_types`], preserving all existing `zh_logic::rules::*` paths.

pub use crate::rules::special_power_types::{
    SpecialPowerType, SPECIALPOWER_COUNT, SPECIAL_POWER_NAMES,
};
