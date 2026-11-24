//! `WP-5.2-C`: `SpecialPowerModule` hierarchy (`SpecialPowerModule`, `SpecialAbility`, and all 9
//! concrete special powers).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `SpecialPowerModule` subclasses into [`ModuleRegistry`] (`WP-5.2-C`).
pub fn register_special_power_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {}
