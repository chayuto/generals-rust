//! `WP-3.2`: Phase 3 Zero Hour Core Enum, BitFlags & Tech Tree Registry.
//!
//! Includes `WP-3.2a` (`PKT-125`..`PKT-127`, `PKT-129`, `PKT-135`..`PKT-136`, `PKT-139`..`PKT-140`,
//! `PKT-144`, `PKT-146`..`PKT-149`) and `WP-3.2b` (`PKT-132` `RankInfo`, `PKT-133` `Science`,
//! `PKT-134` `SpecialPower`, `PKT-137` `Upgrade`).
//!
//! Reference C++ bundles: `packets/phase3/WP3_2a_RulesEnums_Bundle.md`,
//! `docs/golden_vectors/WP3_2b_TechTree.json`
//! Golden Vector tests: `crates/zh_logic/tests/golden_wp3_2a_rules.rs`,
//! `crates/zh_logic/tests/golden_wp3_2b_tech_tree.rs`

use zh_common::types::Int;

/// Classification tags used by `UpgradeTemplate` and `SpecialPowerTemplate` (`AcademyStats.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AcademyClassificationType {
    #[default]
    None = 0,
    UpgradeRadar = 1,
    Superpower = 2,
    Count = 3,
}

impl AcademyClassificationType {
    /// Converts a raw `Int` index into `AcademyClassificationType`, returning `None` if out of range.
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::None),
            1 => Some(Self::UpgradeRadar),
            2 => Some(Self::Superpower),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// INI string table matching `TheAcademyClassificationTypeNames[]` in `AcademyStats.cpp`.
pub const ACADEMY_CLASSIFICATION_TYPE_NAMES: [&str; 3] =
    ["ACT_NONE", "ACT_UPGRADE_RADAR", "ACT_SUPERPOWER"];

pub mod combat_types;
pub mod disabled_types;
pub mod kind_of;
pub mod model_state;
pub mod multiplayer_settings;
pub mod object_status_types;
pub mod player_template;
pub mod production_prerequisite;
pub mod rank_info;
pub mod science;
pub mod special_power;
pub mod special_power_types;
pub mod upgrade;

pub use combat_types::*;
pub use disabled_types::*;
pub use kind_of::*;
pub use model_state::*;
pub use multiplayer_settings::*;
pub use object_status_types::*;
pub use player_template::*;
pub use production_prerequisite::*;
pub use rank_info::*;
pub use science::*;
pub use special_power::*;
pub use special_power_types::*;
pub use upgrade::*;
