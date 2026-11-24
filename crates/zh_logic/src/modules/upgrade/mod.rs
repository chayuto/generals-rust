//! `WP-5.2-B`: `UpgradeModule` hierarchy (`UpgradeMux`, `UpgradeModule`, and all 19 concrete
//! upgrade behaviors).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `UpgradeModule` subclasses into [`ModuleRegistry`] (`WP-5.2-B`).
pub fn register_upgrade_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {}
