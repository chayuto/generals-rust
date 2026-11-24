//! `WP-5.2-B`: `UpgradeModule` hierarchy (`UpgradeMux`, `UpgradeModule`, and all 20 concrete
//! upgrade behaviors).

pub mod active_shroud_upgrade;
pub mod armor_upgrade;
pub mod command_set_upgrade;
pub mod cost_modifier_upgrade;
pub mod experience_scalar_upgrade;
pub mod grant_science_upgrade;
pub mod locomotor_set_upgrade;
pub mod max_health_upgrade;
pub mod model_condition_upgrade;
pub mod object_creation_upgrade;
pub mod passengers_fire_upgrade;
pub mod power_plant_upgrade;
pub mod radar_upgrade;
pub mod replace_object_upgrade;
pub mod status_bits_upgrade;
pub mod stealth_upgrade;
pub mod sub_objects_upgrade;
pub mod unpause_special_power_upgrade;
pub mod upgrade_module;
pub mod weapon_bonus_upgrade;
pub mod weapon_set_upgrade;

pub use active_shroud_upgrade::{
    parse_active_shroud_upgrade_data, ActiveShroudUpgrade, ActiveShroudUpgradeModuleData,
};
pub use armor_upgrade::{parse_armor_upgrade_data, ArmorUpgrade};
pub use command_set_upgrade::{
    parse_command_set_upgrade_data, CommandSetUpgrade, CommandSetUpgradeModuleData,
};
pub use cost_modifier_upgrade::{
    parse_cost_modifier_upgrade_data, CostModifierUpgrade, CostModifierUpgradeModuleData,
};
pub use experience_scalar_upgrade::{
    parse_experience_scalar_upgrade_data, ExperienceScalarUpgrade,
    ExperienceScalarUpgradeModuleData,
};
pub use grant_science_upgrade::{
    parse_grant_science_upgrade_data, GrantScienceUpgrade, GrantScienceUpgradeModuleData,
};
pub use locomotor_set_upgrade::{parse_locomotor_set_upgrade_data, LocomotorSetUpgrade};
pub use max_health_upgrade::{
    parse_max_health_upgrade_data, MaxHealthUpgrade, MaxHealthUpgradeModuleData,
};
pub use model_condition_upgrade::{
    parse_model_condition_upgrade_data, ModelConditionUpgrade, ModelConditionUpgradeModuleData,
};
pub use object_creation_upgrade::{
    parse_object_creation_upgrade_data, ObjectCreationUpgrade, ObjectCreationUpgradeModuleData,
};
pub use passengers_fire_upgrade::{parse_passengers_fire_upgrade_data, PassengersFireUpgrade};
pub use power_plant_upgrade::{parse_power_plant_upgrade_data, PowerPlantUpgrade};
pub use radar_upgrade::{parse_radar_upgrade_data, RadarUpgrade, RadarUpgradeModuleData};
pub use replace_object_upgrade::{
    parse_replace_object_upgrade_data, ReplaceObjectUpgrade, ReplaceObjectUpgradeModuleData,
};
pub use status_bits_upgrade::{
    parse_status_bits_upgrade_data, StatusBitsUpgrade, StatusBitsUpgradeModuleData,
};
pub use stealth_upgrade::{parse_stealth_upgrade_data, StealthUpgrade};
pub use sub_objects_upgrade::{
    parse_sub_objects_upgrade_data, SubObjectsUpgrade, SubObjectsUpgradeModuleData,
};
pub use unpause_special_power_upgrade::{
    parse_unpause_special_power_upgrade_data, UnpauseSpecialPowerUpgrade,
    UnpauseSpecialPowerUpgradeModuleData,
};
pub use upgrade_module::{
    parse_upgrade_data_with, parse_upgrade_module_data, UpgradeModule, UpgradeModuleData,
    UpgradeMux, UpgradeMuxData,
};
pub use weapon_bonus_upgrade::{parse_weapon_bonus_upgrade_data, WeaponBonusUpgrade};
pub use weapon_set_upgrade::{parse_weapon_set_upgrade_data, WeaponSetUpgrade};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::{ModuleCreateFn, ModuleDataParseFn, ModuleRegistry};
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers the 20 concrete `UpgradeModule` subclasses into [`ModuleRegistry`] (`WP-5.2-B`).
pub fn register_upgrade_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    let entries: &[(&str, ModuleDataParseFn, ModuleCreateFn)] = &[
        (
            "ActiveShroudUpgrade",
            parse_active_shroud_upgrade_data,
            ActiveShroudUpgrade::create,
        ),
        ("ArmorUpgrade", parse_armor_upgrade_data, ArmorUpgrade::create),
        (
            "CommandSetUpgrade",
            parse_command_set_upgrade_data,
            CommandSetUpgrade::create,
        ),
        (
            "CostModifierUpgrade",
            parse_cost_modifier_upgrade_data,
            CostModifierUpgrade::create,
        ),
        (
            "ExperienceScalarUpgrade",
            parse_experience_scalar_upgrade_data,
            ExperienceScalarUpgrade::create,
        ),
        (
            "GrantScienceUpgrade",
            parse_grant_science_upgrade_data,
            GrantScienceUpgrade::create,
        ),
        (
            "LocomotorSetUpgrade",
            parse_locomotor_set_upgrade_data,
            LocomotorSetUpgrade::create,
        ),
        (
            "MaxHealthUpgrade",
            parse_max_health_upgrade_data,
            MaxHealthUpgrade::create,
        ),
        (
            "ModelConditionUpgrade",
            parse_model_condition_upgrade_data,
            ModelConditionUpgrade::create,
        ),
        (
            "ObjectCreationUpgrade",
            parse_object_creation_upgrade_data,
            ObjectCreationUpgrade::create,
        ),
        (
            "PassengersFireUpgrade",
            parse_passengers_fire_upgrade_data,
            PassengersFireUpgrade::create,
        ),
        (
            "PowerPlantUpgrade",
            parse_power_plant_upgrade_data,
            PowerPlantUpgrade::create,
        ),
        ("RadarUpgrade", parse_radar_upgrade_data, RadarUpgrade::create),
        (
            "ReplaceObjectUpgrade",
            parse_replace_object_upgrade_data,
            ReplaceObjectUpgrade::create,
        ),
        (
            "StatusBitsUpgrade",
            parse_status_bits_upgrade_data,
            StatusBitsUpgrade::create,
        ),
        (
            "StealthUpgrade",
            parse_stealth_upgrade_data,
            StealthUpgrade::create,
        ),
        (
            "SubObjectsUpgrade",
            parse_sub_objects_upgrade_data,
            SubObjectsUpgrade::create,
        ),
        (
            "UnpauseSpecialPowerUpgrade",
            parse_unpause_special_power_upgrade_data,
            UnpauseSpecialPowerUpgrade::create,
        ),
        (
            "WeaponBonusUpgrade",
            parse_weapon_bonus_upgrade_data,
            WeaponBonusUpgrade::create,
        ),
        (
            "WeaponSetUpgrade",
            parse_weapon_set_upgrade_data,
            WeaponSetUpgrade::create,
        ),
    ];

    for &(name, parse_fn, create_fn) in entries {
        reg.register_module(
            nk,
            name,
            ModuleType::Behavior,
            FacetMask::UPGRADE,
            SleepyUpdatePhase::Normal,
            false,
            parse_fn,
            create_fn,
        );
    }
}
