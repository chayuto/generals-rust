//! `WP-5.1-D`: `DamageModule` hierarchy (`DamageModule`, `BoneFXDamage`, `TransitionDamageFX`).

pub mod bone_fx_damage;
pub mod damage_module;
pub mod transition_damage_fx;

pub use bone_fx_damage::{parse_bone_fx_damage_data, BoneFXDamage};
pub use damage_module::{DamageModule, DamageModuleBase, DamageModuleData};
pub use transition_damage_fx::{
    parse_transition_damage_fx_data, FxDamageFxListInfo, FxDamageLocType, FxDamageOclInfo,
    FxDamageParticleSystemInfo, FxLocInfo, TransitionDamageFX, TransitionDamageFxModuleData,
    DAMAGE_MODULE_MAX_FX,
};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `DamageModule` subclasses into [`ModuleRegistry`] (`WP-5.1-D`).
pub fn register_damage_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "BoneFXDamage",
        ModuleType::Behavior,
        FacetMask::DAMAGE,
        SleepyUpdatePhase::Normal,
        false,
        parse_bone_fx_damage_data,
        BoneFXDamage::create,
    );
    reg.register_module(
        nk,
        "TransitionDamageFX",
        ModuleType::Behavior,
        FacetMask::DAMAGE,
        SleepyUpdatePhase::Normal,
        false,
        parse_transition_damage_fx_data,
        TransitionDamageFX::create,
    );
}
