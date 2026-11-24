//! `WP-5.1-D`: `DestroyModule` base interface and cleanup module registry hook.

pub mod destroy_module;

pub use destroy_module::{DestroyModule, DestroyModuleBase, DestroyModuleData};

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `DestroyModule` subclasses into [`ModuleRegistry`] (`WP-5.1-D`).
pub fn register_destroy_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {
    // DestroyModule itself is abstract; concrete subclasses arrive in WP-5.2+.
}
