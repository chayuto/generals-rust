//! `WP-5.3`: `ContainModule` implementations (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Contain/`).

pub mod contain_module;
pub mod garrison_contain;
pub mod heal_contain;
pub mod helix_contain;
pub mod internet_hack_contain;
pub mod mob_nexus_contain;
pub mod open_contain;
pub mod parachute_contain;
pub mod railed_transport_contain;
pub mod transport_contain;

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `WP-5.3` concrete `Contain` module factories into `reg`.
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
    reg.register_module(
        nk,
        "TransportContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        transport_contain::parse_transport_contain_data,
        transport_contain::create_transport_contain,
    );
    reg.register_module(
        nk,
        "RailedTransportContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        transport_contain::parse_transport_contain_data,
        railed_transport_contain::create_railed_transport_contain,
    );
    reg.register_module(
        nk,
        "GarrisonContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        garrison_contain::parse_garrison_contain_data,
        garrison_contain::create_garrison_contain,
    );
    reg.register_module(
        nk,
        "HealContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        heal_contain::parse_heal_contain_data,
        heal_contain::create_heal_contain,
    );
    reg.register_module(
        nk,
        "InternetHackContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        internet_hack_contain::parse_internet_hack_contain_data,
        internet_hack_contain::create_internet_hack_contain,
    );
    reg.register_module(
        nk,
        "MobNexusContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0 | FacetMask::TRANSPORT_PASSENGER.0),
        SleepyUpdatePhase::Normal,
        false,
        mob_nexus_contain::parse_mob_nexus_contain_data,
        mob_nexus_contain::create_mob_nexus_contain,
    );
    reg.register_module(
        nk,
        "ParachuteContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        parachute_contain::parse_parachute_contain_data,
        parachute_contain::create_parachute_contain,
    );
    reg.register_module(
        nk,
        "HelixContain",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        helix_contain::parse_helix_contain_data,
        helix_contain::create_helix_contain,
    );
}
