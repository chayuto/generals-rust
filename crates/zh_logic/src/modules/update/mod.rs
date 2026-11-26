//! `WP-5.4`: `UpdateModule` leaf modules.

pub mod base_renerate_update;
pub mod default_production_exit_update;
pub mod deletion_update;
pub mod dock_update;
pub mod repair_dock_update;
pub mod supply_center_dock_update;
pub mod supply_warehouse_dock_update;
pub mod dynamic_geometry_info_update;
pub mod dynamic_shroud_clearing_range_update;
pub mod float_update;
pub mod height_die_update;
pub mod fire_spread_update;
pub mod fire_weapon_update;
pub mod flammable_update;
pub mod lifetime_update;
pub mod physics_update;
pub mod projectile_stream_update;
pub mod power_plant_update;
pub mod prone_update;
pub mod queue_production_exit_update;
pub mod radar_update;
pub mod radius_decal_update;
pub mod smart_bomb_target_homing_update;
pub mod spawn_point_production_exit_update;
pub mod special_power_update_module;
pub mod spy_vision_update;
pub mod stealth_detector_update;
pub mod stealth_update;
pub mod supply_center_production_exit_update;
pub mod weapon_bonus_update;

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Replaces the built-in stubs of the ported update modules.
pub fn register_update_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "SupplyWarehouseDockUpdate",
        ModuleType::Behavior,
        dock_update::DockUpdateModuleData::FACET_MASK,
        SleepyUpdatePhase::Normal,
        false,
        supply_warehouse_dock_update::parse_supply_warehouse_dock_update_data,
        supply_warehouse_dock_update::create_supply_warehouse_dock_update,
    );
    reg.register_module(
        nk,
        "SupplyCenterDockUpdate",
        ModuleType::Behavior,
        dock_update::DockUpdateModuleData::FACET_MASK,
        SleepyUpdatePhase::Normal,
        false,
        supply_center_dock_update::parse_supply_center_dock_update_data,
        supply_center_dock_update::create_supply_center_dock_update,
    );
    reg.register_module(
        nk,
        "RepairDockUpdate",
        ModuleType::Behavior,
        dock_update::DockUpdateModuleData::FACET_MASK,
        SleepyUpdatePhase::Normal,
        false,
        repair_dock_update::parse_repair_dock_update_data,
        repair_dock_update::create_repair_dock_update,
    );
    reg.register_module(
        nk,
        "DefaultProductionExitUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        default_production_exit_update::parse_default_production_exit_update_data,
        default_production_exit_update::create_default_production_exit_update,
    );
    reg.register_module(
        nk,
        "SupplyCenterProductionExitUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        supply_center_production_exit_update::parse_supply_center_production_exit_update_data,
        supply_center_production_exit_update::create_supply_center_production_exit_update,
    );
    reg.register_module(
        nk,
        "QueueProductionExitUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        queue_production_exit_update::parse_queue_production_exit_update_data,
        queue_production_exit_update::create_queue_production_exit_update,
    );
    reg.register_module(
        nk,
        "SpawnPointProductionExitUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        spawn_point_production_exit_update::parse_spawn_point_production_exit_update_data,
        spawn_point_production_exit_update::create_spawn_point_production_exit_update,
    );
    reg.register_module(
        nk,
        "PhysicsBehavior",
        ModuleType::Behavior,
        FacetMask(0x011 | FacetMask::PHYSICS_UPDATE.0),
        SleepyUpdatePhase::Physics,
        false,
        physics_update::parse_physics_behavior_data,
        physics_update::create_physics_behavior,
    );
    reg.register_module(
        nk,
        "StealthDetectorUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        stealth_detector_update::parse_stealth_detector_update_data,
        stealth_detector_update::create_stealth_detector_update,
    );
    reg.register_module(
        nk,
        "StealthUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::STEALTH_UPDATE.0),
        SleepyUpdatePhase::Normal,
        false,
        stealth_update::parse_stealth_update_data,
        stealth_update::create_stealth_update,
    );
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
        "FireSpreadUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        fire_spread_update::parse_fire_spread_update_data,
        fire_spread_update::create_fire_spread_update,
    );
    reg.register_module(
        nk,
        "FlammableUpdate",
        ModuleType::Behavior,
        FacetMask(0x005),
        SleepyUpdatePhase::Normal,
        false,
        flammable_update::parse_flammable_update_data,
        flammable_update::create_flammable_update,
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
    reg.register_module(
        nk,
        "RadiusDecalUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        radius_decal_update::parse_radius_decal_update_data,
        radius_decal_update::create_radius_decal_update,
    );
    reg.register_module(
        nk,
        "HeightDieUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        height_die_update::parse_height_die_update_data,
        height_die_update::create_height_die_update,
    );
    reg.register_module(
        nk,
        "WeaponBonusUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        weapon_bonus_update::parse_weapon_bonus_update_data,
        weapon_bonus_update::create_weapon_bonus_update,
    );
    reg.register_module(
        nk,
        "DynamicShroudClearingRangeUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        dynamic_shroud_clearing_range_update::parse_dynamic_shroud_clearing_range_update_data,
        dynamic_shroud_clearing_range_update::create_dynamic_shroud_clearing_range_update,
    );
    reg.register_module(
        nk,
        "RadarUpdate",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        radar_update::parse_radar_update_data,
        radar_update::create_radar_update,
    );
    reg.register_module(
        nk,
        "PowerPlantUpdate",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::POWER_PLANT_UPDATE.0),
        SleepyUpdatePhase::Normal,
        false,
        power_plant_update::parse_power_plant_update_data,
        power_plant_update::create_power_plant_update,
    );
    reg.register_module(
        nk,
        "SpyVisionUpdate",
        ModuleType::Behavior,
        FacetMask(0x081),
        SleepyUpdatePhase::Normal,
        false,
        spy_vision_update::parse_spy_vision_update_data,
        spy_vision_update::create_spy_vision_update,
    );
}
