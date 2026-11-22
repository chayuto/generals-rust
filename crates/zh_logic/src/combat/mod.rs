//! `WP-3.3a`: Combat primitives (`PKT-138` `Armor` and `PKT-140` `DamageInfo`).

pub mod armor;
pub mod armor_set;
pub mod damage;
pub mod locomotor;
pub mod locomotor_set;

pub use armor_set::ArmorTemplateSet;
pub use locomotor::{
    Locomotor, LocomotorAppearance, LocomotorBehaviorZ, LocomotorPriority, LocomotorStore,
    LocomotorTemplate,
};
pub use locomotor_set::LocomotorSet;
