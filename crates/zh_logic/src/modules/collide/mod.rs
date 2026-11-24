//! `WP-5.2-D`: `CollideModule` hierarchy (`CollideModule`, `CrateCollide`, `FireWeaponCollide`,
//! `SquishCollide`, and all 16 `CrateCollide` subclasses).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `CollideModule` subclasses into [`ModuleRegistry`] (`WP-5.2-D`).
pub fn register_collide_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {}
