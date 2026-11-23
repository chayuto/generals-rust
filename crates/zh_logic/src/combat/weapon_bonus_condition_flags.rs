//! `PKT-146`: `WeaponBonusConditionFlags` (`GameLogic/Weapon.h` & `Common/GlobalData.h`).
//!
//! Manifest-aligned re-export of `WeaponBonusConditionFlags` (bitmask of
//! `WeaponBonusConditionType` flags on `Object` / `WeaponBonusSet`), preserving all
//! existing `zh_logic::rules::*` and `zh_logic::gamelogic::global_data::*` paths.

pub use crate::gamelogic::global_data::WeaponBonusConditionType;
pub use crate::rules::combat_types::WeaponBonusConditionFlags;
