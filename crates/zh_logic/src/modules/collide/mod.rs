//! `WP-5.2-D`: `CollideModule` hierarchy (`CollideModule`, `CrateCollide`, `FireWeaponCollide`,
//! `SquishCollide`, and all 16 `CrateCollide` subclasses).

pub mod collide_module;
pub mod crate_collide;
pub mod sabotage_command_center_crate_collide;
pub mod sabotage_fake_building;
pub mod sabotage_internet_center_crate_collide;
pub mod sabotage_military_factory_crate_collide;
pub mod sabotage_power_plant_crate_collide;
pub mod sabotage_superweapon_crate_collide;
pub mod sabotage_supply_center_crate_collide;
pub mod sabotage_supply_dropzone_crate_collide;

pub use collide_module::{
    parse_collide_data_with, parse_collide_module_data, CollideBehaviorImpl, CollideModule,
    CollideModuleData,
};
pub use crate_collide::{
    execute_crate_behavior, is_valid_to_execute, parse_crate_collide_data_with,
    sabotage_goal_matches, sabotage_is_valid_to_execute, sabotage_steal_cash, CrateCollide,
    CrateCollideData, CrateCollideImpl, SabotageVictimType, PRESERVE_MULTI_CRATE_PICKUP,
};
pub use sabotage_command_center_crate_collide::{
    parse_sabotage_command_center_crate_collide_data, SabotageCommandCenterCrateCollide,
    SabotageCommandCenterCrateCollideModuleData,
};
pub use sabotage_fake_building::{
    parse_sabotage_fake_building_crate_collide_data, SabotageFakeBuildingCrateCollide,
    SabotageFakeBuildingCrateCollideModuleData,
};
pub use sabotage_internet_center_crate_collide::{
    parse_sabotage_internet_center_crate_collide_data, SabotageInternetCenterCrateCollide,
    SabotageInternetCenterCrateCollideModuleData,
};
pub use sabotage_military_factory_crate_collide::{
    parse_sabotage_military_factory_crate_collide_data, SabotageMilitaryFactoryCrateCollide,
    SabotageMilitaryFactoryCrateCollideModuleData,
};
pub use sabotage_power_plant_crate_collide::{
    parse_sabotage_power_plant_crate_collide_data, SabotagePowerPlantCrateCollide,
    SabotagePowerPlantCrateCollideModuleData,
};
pub use sabotage_superweapon_crate_collide::{
    parse_sabotage_superweapon_crate_collide_data, SabotageSuperweaponCrateCollide,
    SabotageSuperweaponCrateCollideModuleData,
};
pub use sabotage_supply_center_crate_collide::{
    parse_sabotage_supply_center_crate_collide_data, SabotageSupplyCenterCrateCollide,
    SabotageSupplyCenterCrateCollideModuleData,
};
pub use sabotage_supply_dropzone_crate_collide::{
    parse_sabotage_supply_dropzone_crate_collide_data, SabotageSupplyDropzoneCrateCollide,
    SabotageSupplyDropzoneCrateCollideModuleData,
};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::{ModuleCreateFn, ModuleDataParseFn, ModuleRegistry};
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `CollideModule` subclasses into [`ModuleRegistry`] (`WP-5.2-D1`).
pub fn register_collide_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    let entries: &[(&str, FacetMask, ModuleDataParseFn, ModuleCreateFn)] = &[
        (
            "SabotageCommandCenterCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_command_center_crate_collide_data,
            SabotageCommandCenterCrateCollide::create,
        ),
        (
            "SabotageFakeBuildingCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_fake_building_crate_collide_data,
            SabotageFakeBuildingCrateCollide::create,
        ),
        (
            "SabotageInternetCenterCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_internet_center_crate_collide_data,
            SabotageInternetCenterCrateCollide::create,
        ),
        (
            "SabotageMilitaryFactoryCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_military_factory_crate_collide_data,
            SabotageMilitaryFactoryCrateCollide::create,
        ),
        (
            "SabotagePowerPlantCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_power_plant_crate_collide_data,
            SabotagePowerPlantCrateCollide::create,
        ),
        (
            "SabotageSuperweaponCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_superweapon_crate_collide_data,
            SabotageSuperweaponCrateCollide::create,
        ),
        (
            "SabotageSupplyCenterCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_supply_center_crate_collide_data,
            SabotageSupplyCenterCrateCollide::create,
        ),
        (
            "SabotageSupplyDropzoneCrateCollide",
            FacetMask::COLLIDE,
            parse_sabotage_supply_dropzone_crate_collide_data,
            SabotageSupplyDropzoneCrateCollide::create,
        ),
    ];

    for &(name, facet_mask, parse_fn, create_fn) in entries {
        reg.register_module(
            nk,
            name,
            ModuleType::Behavior,
            facet_mask,
            SleepyUpdatePhase::Normal,
            false,
            parse_fn,
            create_fn,
        );
    }
}
