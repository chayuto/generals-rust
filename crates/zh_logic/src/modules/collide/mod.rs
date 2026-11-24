//! `WP-5.2-D`: `CollideModule` hierarchy (`CollideModule`, `CrateCollide`, `FireWeaponCollide`,
//! `SquishCollide`, and all 16 `CrateCollide` subclasses).

pub mod collide_module;
pub mod convert_to_car_bomb_crate_collide;
pub mod convert_to_hijacked_vehicle_crate_collide;
pub mod crate_collide;
pub mod fire_weapon_collide;
pub mod heal_crate_collide;
pub mod money_crate_collide;
pub mod sabotage_command_center_crate_collide;
pub mod sabotage_fake_building;
pub mod sabotage_internet_center_crate_collide;
pub mod sabotage_military_factory_crate_collide;
pub mod sabotage_power_plant_crate_collide;
pub mod sabotage_superweapon_crate_collide;
pub mod sabotage_supply_center_crate_collide;
pub mod sabotage_supply_dropzone_crate_collide;
pub mod salvage_crate_collide;
pub mod shroud_crate_collide;
pub mod squish_collide;
pub mod unit_crate_collide;
pub mod veterancy_crate_collide;

pub use collide_module::{
    parse_collide_data_with, parse_collide_module_data, CollideBehaviorImpl, CollideModule,
    CollideModuleData,
};
pub use convert_to_car_bomb_crate_collide::{
    parse_convert_to_car_bomb_crate_collide_data, ConvertToCarBombCrateCollide,
    ConvertToCarBombCrateCollideModuleData,
};
pub use convert_to_hijacked_vehicle_crate_collide::{
    parse_convert_to_hijacked_vehicle_crate_collide_data, ConvertToHijackedVehicleCrateCollide,
    ConvertToHijackedVehicleCrateCollideModuleData,
};
pub use crate_collide::{
    execute_crate_behavior, is_valid_to_execute, parse_crate_collide_data_with,
    sabotage_goal_matches, sabotage_is_valid_to_execute, sabotage_steal_cash, CrateCollide,
    CrateCollideData, CrateCollideImpl, SabotageVictimType, PRESERVE_MULTI_CRATE_PICKUP,
};
pub use fire_weapon_collide::{
    parse_fire_weapon_collide_data, FireWeaponCollide, FireWeaponCollideModuleData,
};
pub use heal_crate_collide::{
    parse_heal_crate_collide_data, HealCrateCollide, HealCrateCollideModuleData,
};
pub use money_crate_collide::{
    parse_money_crate_collide_data, MoneyCrateCollide, MoneyCrateCollideModuleData, UpgradePair,
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
pub use salvage_crate_collide::{
    parse_salvage_crate_collide_data, SalvageCrateCollide, SalvageCrateCollideModuleData,
};
pub use shroud_crate_collide::{
    parse_shroud_crate_collide_data, ShroudCrateCollide, ShroudCrateCollideModuleData,
};
pub use squish_collide::{parse_squish_collide_data, SquishCollide, SquishCollideModuleData};
pub use unit_crate_collide::{
    parse_unit_crate_collide_data, UnitCrateCollide, UnitCrateCollideModuleData,
};
pub use veterancy_crate_collide::{
    parse_veterancy_crate_collide_data, VeterancyCrateCollide, VeterancyCrateCollideModuleData,
};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::{ModuleCreateFn, ModuleDataParseFn, ModuleRegistry};
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers all 18 `CollideModule` subclasses into [`ModuleRegistry`] (`WP-5.2-D1` + `WP-5.2-D2`).
pub fn register_collide_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    let entries: &[(&str, FacetMask, ModuleDataParseFn, ModuleCreateFn)] = &[
        (
            "FireWeaponCollide",
            FacetMask::COLLIDE,
            parse_fire_weapon_collide_data,
            FireWeaponCollide::create,
        ),
        (
            "SquishCollide",
            FacetMask::COLLIDE,
            parse_squish_collide_data,
            SquishCollide::create,
        ),
        (
            "HealCrateCollide",
            FacetMask::COLLIDE,
            parse_heal_crate_collide_data,
            HealCrateCollide::create,
        ),
        (
            "MoneyCrateCollide",
            FacetMask::COLLIDE,
            parse_money_crate_collide_data,
            MoneyCrateCollide::create,
        ),
        (
            "ShroudCrateCollide",
            FacetMask::COLLIDE,
            parse_shroud_crate_collide_data,
            ShroudCrateCollide::create,
        ),
        (
            "UnitCrateCollide",
            FacetMask::COLLIDE,
            parse_unit_crate_collide_data,
            UnitCrateCollide::create,
        ),
        (
            "VeterancyCrateCollide",
            FacetMask::COLLIDE,
            parse_veterancy_crate_collide_data,
            VeterancyCrateCollide::create,
        ),
        (
            "ConvertToCarBombCrateCollide",
            FacetMask::COLLIDE,
            parse_convert_to_car_bomb_crate_collide_data,
            ConvertToCarBombCrateCollide::create,
        ),
        (
            "ConvertToHijackedVehicleCrateCollide",
            FacetMask::COLLIDE,
            parse_convert_to_hijacked_vehicle_crate_collide_data,
            ConvertToHijackedVehicleCrateCollide::create,
        ),
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
        (
            "SalvageCrateCollide",
            FacetMask::COLLIDE,
            parse_salvage_crate_collide_data,
            SalvageCrateCollide::create,
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
