//! `WP-3.2a`: Phase 3 Zero Hour Core Enum & BitFlags Registry Bundle (`12` packets).
//!
//! Includes `PKT-125` (`DisabledTypes`), `PKT-126` (`KindOf`), `PKT-127` (`ModelState`),
//! `PKT-129` (`ObjectStatusTypes`), `PKT-135` (`SpecialPowerMaskType`),
//! `PKT-136` (`SpecialPowerType`), `PKT-139` (`ArmorSet`), `PKT-140` (`Damage` enums/flags),
//! `PKT-144` (`LocomotorSet` surface types), and `PKT-146`–`149` (`Weapon*` types/flags).
//!
//! Reference C++ bundle: `packets/phase3/WP3_2a_RulesEnums_Bundle.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_wp3_2a_rules.rs`

pub mod combat_types;
pub mod disabled_types;
pub mod kind_of;
pub mod model_state;
pub mod object_status_types;
pub mod special_power_types;

pub use combat_types::*;
pub use disabled_types::*;
pub use kind_of::*;
pub use model_state::*;
pub use object_status_types::*;
pub use special_power_types::*;
