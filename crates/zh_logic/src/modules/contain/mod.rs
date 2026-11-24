//! `WP-5.3`: `ContainModule` implementations (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/`).

pub mod contain_module;
pub mod open_contain;

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `WP-5.3-A` concrete `Contain` module factories into `reg`.
pub fn register_contain_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "OpenContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        open_contain::parse_open_contain_data,
        open_contain::create_open_contain,
    );
}
