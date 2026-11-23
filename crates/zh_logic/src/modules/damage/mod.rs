//! `WP-5.1-D`: `DamageModule` hierarchy (`BoneFXDamage`, `TransitionDamageFX`).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `DamageModule` subclasses into [`ModuleRegistry`] (`WP-5.1-D`).
pub fn register_damage_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {
    // Registered in `WP-5.1-D`.
}
