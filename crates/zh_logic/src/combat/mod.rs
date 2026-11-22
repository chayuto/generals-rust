//! `WP-3.3a`: Combat primitives (`PKT-138` `Armor` and `PKT-140` `DamageInfo`).

pub mod armor;
pub mod damage;
pub mod locomotor;
pub mod locomotor_set;

pub use locomotor::{
    Locomotor, LocomotorAppearance, LocomotorBehaviorZ, LocomotorPriority, LocomotorStore,
    LocomotorTemplate,
};
pub use locomotor_set::LocomotorSet;
