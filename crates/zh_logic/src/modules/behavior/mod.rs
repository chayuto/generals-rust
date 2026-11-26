//! `WP-5.3`: `BehaviorModule` implementations (`GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Behavior/`).

pub mod auto_heal_behavior;
pub mod battle_bus_slow_death_behavior;
pub mod behavior_module;
pub mod fire_weapon_when_damaged_behavior;
pub mod fire_weapon_when_dead_behavior;
pub mod grant_stealth_behavior;
pub mod instant_death_behavior;
pub mod jet_slow_death_behavior;
pub mod prison_behavior;
pub mod slow_death_behavior;

use crate::arena::FacetMask;
use crate::modules::module::ModuleType;
use crate::modules::module_factory::ModuleRegistry;
use crate::scheduler::SleepyUpdatePhase;
use zh_common::core::name_key_generator::NameKeyGenerator;

/// Registers `WP-5.3` concrete `Behavior` module factories into `reg`.
pub fn register_behavior_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "AutoHealBehavior",
        ModuleType::Behavior,
        FacetMask(0x085),
        SleepyUpdatePhase::Normal,
        false,
        auto_heal_behavior::parse_auto_heal_behavior_data,
        auto_heal_behavior::create_auto_heal_behavior,
    );
    reg.register_module(
        nk,
        "FireWeaponWhenDamagedBehavior",
        ModuleType::Behavior,
        FacetMask(0x085),
        SleepyUpdatePhase::Normal,
        false,
        fire_weapon_when_damaged_behavior::parse_fire_weapon_when_damaged_behavior_data,
        fire_weapon_when_damaged_behavior::create_fire_weapon_when_damaged_behavior,
    );
    reg.register_module(
        nk,
        "FireWeaponWhenDeadBehavior",
        ModuleType::Behavior,
        FacetMask(0x082),
        SleepyUpdatePhase::Normal,
        false,
        fire_weapon_when_dead_behavior::parse_fire_weapon_when_dead_behavior_data,
        fire_weapon_when_dead_behavior::create_fire_weapon_when_dead_behavior,
    );
    reg.register_module(
        nk,
        "GrantStealthBehavior",
        ModuleType::Behavior,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        false,
        grant_stealth_behavior::parse_grant_stealth_behavior_data,
        grant_stealth_behavior::create_grant_stealth_behavior,
    );
    reg.register_module(
        nk,
        "InstantDeathBehavior",
        ModuleType::Behavior,
        FacetMask(0x002),
        SleepyUpdatePhase::Normal,
        false,
        instant_death_behavior::parse_instant_death_behavior_data,
        instant_death_behavior::create_instant_death_behavior,
    );
    reg.register_module(
        nk,
        "SlowDeathBehavior",
        ModuleType::Behavior,
        FacetMask(0x003 | FacetMask::SLOW_DEATH.0),
        SleepyUpdatePhase::Normal,
        false,
        slow_death_behavior::parse_slow_death_behavior_data,
        slow_death_behavior::create_slow_death_behavior,
    );
    reg.register_module(
        nk,
        "BattleBusSlowDeathBehavior",
        ModuleType::Behavior,
        FacetMask(0x003 | FacetMask::SLOW_DEATH.0),
        SleepyUpdatePhase::Normal,
        false,
        battle_bus_slow_death_behavior::parse_battle_bus_slow_death_behavior_data,
        battle_bus_slow_death_behavior::create_battle_bus_slow_death_behavior,
    );
    reg.register_module(
        nk,
        "JetSlowDeathBehavior",
        ModuleType::Behavior,
        FacetMask(0x003 | FacetMask::SLOW_DEATH.0),
        SleepyUpdatePhase::Normal,
        false,
        jet_slow_death_behavior::parse_jet_slow_death_behavior_data,
        jet_slow_death_behavior::create_jet_slow_death_behavior,
    );
}

/// Registers the `ALLOW_SURRENDER` behaviors. They are compiled out of the reference game, so
/// this is never part of [`register_behavior_modules`].
pub fn register_surrender_behavior_modules(reg: &mut ModuleRegistry, nk: &mut NameKeyGenerator) {
    reg.register_module(
        nk,
        "PrisonBehavior",
        ModuleType::Behavior,
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0),
        SleepyUpdatePhase::Normal,
        false,
        prison_behavior::parse_prison_behavior_data,
        prison_behavior::create_prison_behavior,
    );
}
