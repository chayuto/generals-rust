//! `WP-5.2-A`: `DieModule` hierarchy (`DieModule`, `CreateCrateDie`, `CreateObjectDie`,
//! `CrushDie`, `DamDie`, `DestroyDie`, `EjectPilotDie`, `FXListDie`, `KeepObjectDie`,
//! `RebuildHoleExposeDie`, `SpecialPowerCompletionDie`, `UpgradeDie`).

pub mod create_crate_die;
pub mod create_object_die;
pub mod crush_die;
pub mod dam_die;
pub mod destroy_die;
pub mod die_module;
pub mod eject_pilot_die;
pub mod fx_list_die;
pub mod keep_object_die;
pub mod rebuild_hole_expose_die;
pub mod special_power_completion_die;
pub mod upgrade_die;

pub use create_crate_die::{
    parse_create_crate_die_data, CreateCrateDie, CreateCrateDieModuleData,
};
pub use create_object_die::{
    parse_create_object_die_data, CreateObjectDie, CreateObjectDieModuleData,
};
pub use crush_die::{parse_crush_die_data, CrushDie, CrushDieModuleData, CrushEnum};
pub use dam_die::{parse_dam_die_data, DamDie, DamDieModuleData};
pub use destroy_die::{parse_destroy_die_data, DestroyDie, DestroyDieModuleData};
pub use die_module::{
    get_veterancy_level_flag, parse_die_data_with, parse_die_module_data, DieModule,
    DieModuleData, DieMuxData,
};
pub use eject_pilot_die::{
    parse_eject_pilot_die_data, EjectPilotDie, EjectPilotDieModuleData,
};
pub use fx_list_die::{parse_fx_list_die_data, FXListDie, FXListDieModuleData};
pub use keep_object_die::{parse_keep_object_die_data, KeepObjectDie, KeepObjectDieModuleData};
pub use rebuild_hole_expose_die::{
    parse_rebuild_hole_expose_die_data, RebuildHoleExposeDie, RebuildHoleExposeDieModuleData,
};
pub use special_power_completion_die::{
    parse_special_power_completion_die_data, SpecialPowerCompletionDie,
    SpecialPowerCompletionDieModuleData,
};
pub use upgrade_die::{parse_upgrade_die_data, UpgradeDie, UpgradeDieModuleData};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::{ModuleCreateFn, ModuleDataParseFn, ModuleRegistry};
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers the 11 concrete `DieModule` subclasses into [`ModuleRegistry`] (`WP-5.2-A`).
pub fn register_die_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    let entries: &[(&str, FacetMask, ModuleDataParseFn, ModuleCreateFn)] = &[
        (
            "CreateCrateDie",
            FacetMask::DIE,
            parse_create_crate_die_data,
            CreateCrateDie::create,
        ),
        (
            "CreateObjectDie",
            FacetMask::DIE,
            parse_create_object_die_data,
            CreateObjectDie::create,
        ),
        (
            "CrushDie",
            FacetMask::DIE,
            parse_crush_die_data,
            CrushDie::create,
        ),
        ("DamDie", FacetMask::DIE, parse_dam_die_data, DamDie::create),
        (
            "DestroyDie",
            FacetMask::DIE,
            parse_destroy_die_data,
            DestroyDie::create,
        ),
        (
            "EjectPilotDie",
            FacetMask(FacetMask::DIE.0 | FacetMask::EJECT_PILOT_DIE.0),
            parse_eject_pilot_die_data,
            EjectPilotDie::create,
        ),
        (
            "FXListDie",
            FacetMask(FacetMask::DIE.0 | FacetMask::UPGRADE.0),
            parse_fx_list_die_data,
            FXListDie::create,
        ),
        (
            "KeepObjectDie",
            FacetMask::DIE,
            parse_keep_object_die_data,
            KeepObjectDie::create,
        ),
        (
            "RebuildHoleExposeDie",
            FacetMask::DIE,
            parse_rebuild_hole_expose_die_data,
            RebuildHoleExposeDie::create,
        ),
        (
            "SpecialPowerCompletionDie",
            FacetMask::DIE,
            parse_special_power_completion_die_data,
            SpecialPowerCompletionDie::create,
        ),
        (
            "UpgradeDie",
            FacetMask::DIE,
            parse_upgrade_die_data,
            UpgradeDie::create,
        ),
    ];

    for &(name, facet_mask, parse_fn, create_fn) in entries {
        reg.register_module(
            nk,
            name,
            ModuleType::Behavior,
            facet_mask,
            SleepyUpdatePhase::Normal,
            false,
            parse_fn,
            create_fn,
        );
    }
}
