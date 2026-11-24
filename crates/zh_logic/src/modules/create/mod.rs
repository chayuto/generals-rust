//! `WP-5.1-C`: `CreateModule` hierarchy (`GrantUpgradeCreate`, `LockWeaponCreate`,
//! `PreorderCreate`, `SpecialPowerCreate`, `SupplyCenterCreate`, `SupplyWarehouseCreate`,
//! `VeterancyGainCreate`).

pub mod create_module;
pub mod grant_upgrade_create;
pub mod lock_weapon_create;
pub mod preorder_create;
pub mod special_power_create;
pub mod supply_center_create;
pub mod supply_warehouse_create;
pub mod veterancy_gain_create;

pub use create_module::{CreateModule, CreateModuleBase, CreateModuleData};
pub use grant_upgrade_create::{
    parse_grant_upgrade_create_data, GrantUpgradeCreate, GrantUpgradeCreateModuleData,
};
pub use lock_weapon_create::{
    parse_lock_weapon_create_data, LockWeaponCreate, LockWeaponCreateModuleData,
};
pub use preorder_create::{parse_preorder_create_data, PreorderCreate};
pub use special_power_create::{parse_special_power_create_data, SpecialPowerCreate};
pub use supply_center_create::{parse_supply_center_create_data, SupplyCenterCreate};
pub use supply_warehouse_create::{parse_supply_warehouse_create_data, SupplyWarehouseCreate};
pub use veterancy_gain_create::{
    parse_veterancy_gain_create_data, VeterancyGainCreate, VeterancyGainCreateModuleData,
};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers the 7 concrete `CreateModule` subclasses into [`ModuleRegistry`] (`WP-5.1-C`).
pub fn register_create_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "GrantUpgradeCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_grant_upgrade_create_data,
        GrantUpgradeCreate::create,
    );
    reg.register_module(
        nk,
        "LockWeaponCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_lock_weapon_create_data,
        LockWeaponCreate::create,
    );
    reg.register_module(
        nk,
        "PreorderCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_preorder_create_data,
        PreorderCreate::create,
    );
    reg.register_module(
        nk,
        "SpecialPowerCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_special_power_create_data,
        SpecialPowerCreate::create,
    );
    reg.register_module(
        nk,
        "SupplyCenterCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_supply_center_create_data,
        SupplyCenterCreate::create,
    );
    reg.register_module(
        nk,
        "SupplyWarehouseCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_supply_warehouse_create_data,
        SupplyWarehouseCreate::create,
    );
    reg.register_module(
        nk,
        "VeterancyGainCreate",
        ModuleType::Behavior,
        FacetMask::CREATE,
        SleepyUpdatePhase::Normal,
        false,
        parse_veterancy_gain_create_data,
        VeterancyGainCreate::create,
    );
}
