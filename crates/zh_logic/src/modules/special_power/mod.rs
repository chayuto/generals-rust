//! `WP-5.2-C`: `SpecialPowerModule` hierarchy (`SpecialPowerModule`, `SpecialAbility`, and all 9
//! concrete special powers plus `DemoralizeSpecialPower`).

pub mod baikonur_launch_power;
pub mod cash_bounty_power;
pub mod cash_hack_special_power;
pub mod cleanup_area_power;
pub mod defector_special_power;
pub mod demoralize_special_power;
pub mod fire_weapon_power;
pub mod ocl_special_power;
pub mod special_ability;
pub mod special_power_module;
pub mod spy_vision_special_power;

pub use baikonur_launch_power::{
    parse_baikonur_launch_power_data, BaikonurLaunchPower, BaikonurLaunchPowerModuleData,
};
pub use cash_bounty_power::{
    parse_cash_bounty_power_data, CashBountyPower, CashBountyPowerModuleData,
};
pub use cash_hack_special_power::{
    parse_cash_hack_special_power_data, CashHackSpecialPower, CashHackSpecialPowerModuleData,
    CashHackUpgrade,
};
pub use cleanup_area_power::{
    parse_cleanup_area_power_data, CleanupAreaPower, CleanupAreaPowerModuleData,
};
pub use defector_special_power::{
    parse_defector_special_power_data, DefectorSpecialPower, DefectorSpecialPowerModuleData,
};
pub use demoralize_special_power::{
    parse_demoralize_special_power_data, DemoralizeSpecialPower, DemoralizeSpecialPowerModuleData,
};
pub use fire_weapon_power::{
    parse_fire_weapon_power_data, FireWeaponPower, FireWeaponPowerModuleData,
};
pub use ocl_special_power::{
    parse_ocl_special_power_data, OCLSpecialPower, OCLSpecialPowerModuleData, OclCreateLocType,
    OclUpgrade, OCL_CREATE_LOC_TYPE_NAMES,
};
pub use special_ability::{parse_special_ability_data, SpecialAbility};
pub use special_power_module::{
    parse_special_power_data_with, parse_special_power_module_data, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
pub use spy_vision_special_power::{
    parse_spy_vision_special_power_data, SpyVisionSpecialPower, SpyVisionSpecialPowerModuleData,
};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::{ModuleCreateFn, ModuleDataParseFn, ModuleRegistry};
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers the 9 compiled `SpecialPowerModule` subclasses into [`ModuleRegistry`] (`WP-5.2-C`).
///
/// `DemoralizeSpecialPower` (`#ifdef ALLOW_DEMORALIZE`) is intentionally excluded to preserve
/// C++ `NameKeyGenerator` 1..=198 decorated key order; use [`register_demoralize_special_power`]
/// when testing it explicitly.
pub fn register_special_power_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    let entries: &[(&str, ModuleDataParseFn, ModuleCreateFn)] = &[
        (
            "BaikonurLaunchPower",
            parse_baikonur_launch_power_data,
            BaikonurLaunchPower::create,
        ),
        (
            "CashBountyPower",
            parse_cash_bounty_power_data,
            CashBountyPower::create,
        ),
        (
            "CashHackSpecialPower",
            parse_cash_hack_special_power_data,
            CashHackSpecialPower::create,
        ),
        (
            "CleanupAreaPower",
            parse_cleanup_area_power_data,
            CleanupAreaPower::create,
        ),
        (
            "DefectorSpecialPower",
            parse_defector_special_power_data,
            DefectorSpecialPower::create,
        ),
        (
            "FireWeaponPower",
            parse_fire_weapon_power_data,
            FireWeaponPower::create,
        ),
        (
            "OCLSpecialPower",
            parse_ocl_special_power_data,
            OCLSpecialPower::create,
        ),
        (
            "SpecialAbility",
            parse_special_ability_data,
            SpecialAbility::create,
        ),
        (
            "SpyVisionSpecialPower",
            parse_spy_vision_special_power_data,
            SpyVisionSpecialPower::create,
        ),
    ];

    for &(name, parse_fn, create_fn) in entries {
        reg.register_module(
            nk,
            name,
            ModuleType::Behavior,
            FacetMask::SPECIAL_POWER,
            SleepyUpdatePhase::Normal,
            false,
            parse_fn,
            create_fn,
        );
    }
}

/// Registers `#ifdef ALLOW_DEMORALIZE` module `DemoralizeSpecialPower` separately for tests/oracles.
pub fn register_demoralize_special_power(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "DemoralizeSpecialPower",
        ModuleType::Behavior,
        FacetMask::SPECIAL_POWER,
        SleepyUpdatePhase::Normal,
        false,
        parse_demoralize_special_power_data,
        DemoralizeSpecialPower::create,
    );
}
