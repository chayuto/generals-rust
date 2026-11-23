//! `WP-3.3`: Combat primitives (`PKT-138` `Armor`, `PKT-139` `ArmorSet`, `PKT-140` `DamageInfo`,
//! `PKT-141` `DamageFX`, `PKT-142` `FiringTracker`, `PKT-143`/`PKT-144` `Locomotor`/`LocomotorSet`,
//! `PKT-145` `TurretAI`, and `PKT-146`–`PKT-149`).

pub mod armor;
pub mod armor_set;
pub mod damage;
pub mod damage_fx;
pub mod firing_tracker;
pub mod locomotor;
pub mod locomotor_set;
pub mod turret_ai;
pub mod weapon_bonus_condition_flags;
pub mod weapon_set_flags;
pub mod weapon_set_type;
pub mod weapon_status;

pub use armor_set::ArmorTemplateSet;
pub use locomotor::{
    Locomotor, LocomotorAppearance, LocomotorBehaviorZ, LocomotorPriority, LocomotorStore,
    LocomotorTemplate,
};
pub use locomotor_set::{
    LocomotorSet, LocomotorSetType, LOCOMOTORSET_COUNT, LOCOMOTOR_SET_NAMES,
};
pub use turret_ai::{
    FireStateStep, TurretAI, TurretAIData, TurretAIError, TurretStateMachine, TurretStateType,
    TurretTargetType, TurretWorldAccess, WhichTurretType, DEFAULT_PITCH_RATE, DEFAULT_TURN_RATE,
    TURRET_AI_DATA_FIELD_PARSE,
};
