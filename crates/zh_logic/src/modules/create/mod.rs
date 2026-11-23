//! `WP-5.1-C`: `CreateModule` hierarchy (`GrantUpgradeCreate`, `LockWeaponCreate`,
//! `PreorderCreate`, `SpecialPowerCreate`, `SupplyCenterCreate`, `SupplyWarehouseCreate`,
//! `VeterancyGainCreate`).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `CreateModule` subclasses into [`ModuleRegistry`] (`WP-5.1-C`).
pub fn register_create_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {
    // Registered in `WP-5.1-C`.
}
