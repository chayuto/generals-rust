//! `WP-5.1-B`: `BodyModule` hierarchy (`ActiveBody`, `HighlanderBody`, `HiveStructureBody`,
//! `ImmortalBody`, `InactiveBody`, `StructureBody`, `UndeadBody`).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `BodyModule` subclasses into [`ModuleRegistry`] (`WP-5.1-B`).
pub fn register_body_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {
    // Registered in `WP-5.1-B`.
}
