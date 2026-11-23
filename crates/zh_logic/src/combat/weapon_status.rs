//! `PKT-149`: `WeaponStatus` (`GameLogic/WeaponStatus.h`).
//!
//! Manifest-aligned re-export of the 5-variant `WeaponStatus` enum and
//! `WEAPON_STATUS_COUNT` constant from [`crate::rules::combat_types`].

pub use crate::rules::combat_types::{WeaponStatus, WEAPON_STATUS_COUNT};
