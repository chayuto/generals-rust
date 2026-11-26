//! `WP-5.4`: `UpdateModule` leaf modules.

pub mod base_renerate_update;
pub mod deletion_update;
pub mod dynamic_geometry_info_update;
pub mod float_update;
pub mod fire_weapon_update;
pub mod lifetime_update;
pub mod projectile_stream_update;
pub mod prone_update;
pub mod smart_bomb_target_homing_update;

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Replaces the built-in stubs of the ported update modules.
pub fn register_update_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "BaseRegenerateUpdate",
        ModuleType::Behavior,
        FacetMask(0x005),
        SleepyUpdatePhase::Normal,
        false,
        base_renerate_update::parse_base_regenerate_update_data,
        base_renerate_update::create_base_regenerate_update,
    );
    reg.register_module(
        nk,
        "DeletionUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        deletion_update::parse_deletion_update_data,
        deletion_update::create_deletion_update,
    );
    reg.register_module(
        nk,
        "FireWeaponUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        fire_weapon_update::parse_fire_weapon_update_data,
        fire_weapon_update::create_fire_weapon_update,
    );
    reg.register_module(
        nk,
        "FloatUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        float_update::parse_float_update_data,
        float_update::create_float_update,
    );
    reg.register_module(
        nk,
        "LifetimeUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        lifetime_update::parse_lifetime_update_data,
        lifetime_update::create_lifetime_update,
    );
    reg.register_module(
        nk,
        "ProneUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        prone_update::parse_prone_update_data,
        prone_update::create_prone_update,
    );
    reg.register_module(
        nk,
        "SmartBombTargetHomingUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        smart_bomb_target_homing_update::parse_smart_bomb_target_homing_update_data,
        smart_bomb_target_homing_update::create_smart_bomb_target_homing_update,
    );
    reg.register_module(
        nk,
        "DynamicGeometryInfoUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        dynamic_geometry_info_update::parse_dynamic_geometry_info_update_data,
        dynamic_geometry_info_update::create_dynamic_geometry_info_update,
    );
    reg.register_module(
        nk,
        "ProjectileStreamUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        projectile_stream_update::parse_projectile_stream_update_data,
        projectile_stream_update::create_projectile_stream_update,
    );
}
