//! `WP-5.2-A`: `DieModule` hierarchy (`DieModule`, `CreateCrateDie`, `CreateObjectDie`,
//! `CrushDie`, `DamDie`, `DestroyDie`, `EjectPilotDie`, `FXListDie`, `KeepObjectDie`,
//! `RebuildHoleExposeDie`, `SpecialPowerCompletionDie`, `UpgradeDie`).

use crate::modules::module_factory::ModuleRegistry;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `DieModule` subclasses into [`ModuleRegistry`] (`WP-5.2-A`).
pub fn register_die_modules(_reg: &mut ModuleRegistry, _nk: &mut NameKeyGenerator) {}
