//! `PKT-148`: `WeaponSetType` (`GameLogic/WeaponSetType.h`).
//!
//! Manifest-aligned re-export of the 17-variant `WeaponSetType` enum and
//! `WEAPON_SET_NAMES` table from [`crate::rules::combat_types`].

pub use crate::rules::combat_types::{WeaponSetType, WEAPONSET_COUNT, WEAPON_SET_NAMES};
