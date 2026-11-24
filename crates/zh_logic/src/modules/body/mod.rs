//! `WP-5.1-B`: `BodyModule` hierarchy (`ActiveBody`, `BodyModule`, `HighlanderBody`,
//! `HiveStructureBody`, `ImmortalBody`, `InactiveBody`, `StructureBody`, `UndeadBody`).

pub mod active_body;
pub mod body_module;
pub mod highlander_body;
pub mod hive_structure_body;
pub mod immortal_body;
pub mod inactive_body;
pub mod structure_body;
pub mod undead_body;

pub use active_body::{parse_active_body_data, ActiveBody, ActiveBodyModuleData};
pub use body_module::{BodyModule, BodyModuleBase, BodyModuleData};
pub use highlander_body::HighlanderBody;
pub use hive_structure_body::{
    parse_hive_structure_body_data, HiveStructureBody, HiveStructureBodyModuleData,
};
pub use immortal_body::ImmortalBody;
pub use inactive_body::{parse_inactive_body_data, InactiveBody};
pub use structure_body::StructureBody;
pub use undead_body::{parse_undead_body_data, UndeadBody, UndeadBodyModuleData};

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `BodyModule` subclasses into [`ModuleRegistry`] (`WP-5.1-B`).
pub fn register_body_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    let entries: [(
        &str,
        crate::modules::module_factory::ModuleDataParseFn,
        crate::modules::module_factory::ModuleCreateFn,
    ); 7] = [
        ("ActiveBody", parse_active_body_data, ActiveBody::create),
        ("HighlanderBody", parse_active_body_data, HighlanderBody::create),
        (
            "HiveStructureBody",
            parse_hive_structure_body_data,
            HiveStructureBody::create,
        ),
        ("ImmortalBody", parse_active_body_data, ImmortalBody::create),
        ("InactiveBody", parse_inactive_body_data, InactiveBody::create),
        ("StructureBody", parse_active_body_data, StructureBody::create),
        ("UndeadBody", parse_undead_body_data, UndeadBody::create),
    ];
    for (name, parse_fn, create_fn) in entries {
        reg.register_module(
            nk,
            name,
            ModuleType::Behavior,
            FacetMask::BODY,
            SleepyUpdatePhase::Normal,
            false,
            parse_fn,
            create_fn,
        );
    }
}
