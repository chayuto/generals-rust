//! `PKT-164`: Module class registry (`ModuleFactory.h`, `ModuleFactory.cpp`) and `ModuleDataStore`.
//!
//! Reference C++ packet: `packets/phase5/PKT-164_ModuleFactory.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt164_module_factory.rs`

use std::collections::{BTreeMap, BTreeSet};

use crate::arena::{FacetMask, ModuleContext, ModuleDataId, ThingTemplateId};
use crate::combat::armor::ArmorStore;
use crate::modules::module::{BehaviorModule, ModuleData, ModuleType, StubModuleData};
use crate::object::thing_template::ModuleMaskLookup;
use crate::object::weapon::WeaponStore;
use crate::rules::science::ScienceStore;
use crate::rules::special_power::SpecialPowerStore;
use crate::rules::upgrade::UpgradeCenter;
use crate::scheduler::SleepyUpdatePhase;
use crate::spatial::crate_system::CrateSystem;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, NameKeyType, XferVersion};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// Read/interning context passed into [`ModuleDataParseFn`] while parsing `Module ... End` blocks.
pub struct ModuleParseContext<'a> {
    pub name_keys: &'a mut NameKeyGenerator,
    pub science_store: &'a ScienceStore,
    pub upgrade_center: &'a UpgradeCenter,
    pub armor_store: &'a ArmorStore,
    pub weapon_store: &'a WeaponStore,
    pub fx_list_names: &'a BTreeSet<String>,
    pub damage_fx_names: &'a BTreeSet<String>,
    pub ocl_names: &'a BTreeSet<String>,
    pub particle_system_names: &'a BTreeSet<String>,
    pub special_power_store: &'a SpecialPowerStore,
    pub crate_system: &'a CrateSystem,
}

/// Parser callback converting an INI module block (`Module ... End`) into typed [`ModuleData`].
pub type ModuleDataParseFn = fn(
    &mut IniReader<'_>,
    NameKeyType,
    NameKeyType,
    &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError>;

/// Factory callback instantiating a runtime [`BehaviorModule`] from parsed [`ModuleData`].
pub type ModuleCreateFn = fn(&dyn ModuleData, ModuleContext) -> Box<dyn BehaviorModule>;

/// One static C++ `ModuleFactory::init()` registration entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinModuleEntry {
    pub name: &'static str,
    pub module_type: ModuleType,
    pub facet_mask: FacetMask,
    pub update_phase: SleepyUpdatePhase,
    pub is_ai_module: bool,
}

/// Exact 198 modules registered by `ModuleFactory::init()` in Zero Hour
/// (`GeneralsMD/Code/GameEngine/Source/Common/Thing/ModuleFactory.cpp`, keys `1..=198`).
pub const BUILTIN_ZH_MODULES: [BuiltinModuleEntry; 198] = [
    BuiltinModuleEntry { name: "AutoHealBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x085), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "GrantStealthBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "NeutronBlastBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BridgeBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x007 | FacetMask::BRIDGE_BEHAVIOR.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BridgeScaffoldBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::BRIDGE_SCAFFOLD.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BridgeTowerBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x006 | FacetMask::BRIDGE_TOWER.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CountermeasuresBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x081 | FacetMask::COUNTERMEASURES.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DumbProjectileBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::PROJECTILE_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PhysicsBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x011 | FacetMask::PHYSICS_UPDATE.0), update_phase: SleepyUpdatePhase::Physics, is_ai_module: false },
    BuiltinModuleEntry { name: "InstantDeathBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SlowDeathBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::SLOW_DEATH.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HelicopterSlowDeathBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::SLOW_DEATH.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "NeutronMissileSlowDeathBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::SLOW_DEATH.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CaveContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x05f | FacetMask::CAVE_INTERFACE.0 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "OpenContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "OverchargeBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x005 | FacetMask::OVERCHARGE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HealContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "GarrisonContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "InternetHackContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "TransportContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RiderChangeContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RailedTransportContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MobNexusContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0 | FacetMask::TRANSPORT_PASSENGER.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "TunnelContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x05f | FacetMask::TUNNEL_CONTAIN.0 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "OverlordContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HelixContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ParachuteContain", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PropagandaTowerBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::PROPAGANDA_TOWER.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BunkerBusterBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireWeaponWhenDamagedBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x085), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireWeaponWhenDeadBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x082), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "GenerateMinefieldBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x083), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ParkingPlaceBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::PARKING_PLACE.0 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FlightDeckBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::AI_UPDATE.0 | FacetMask::PARKING_PLACE.0 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "PoisonedBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x005), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RebuildHoleBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::REBUILD_HOLE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SupplyWarehouseCripplingBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x005), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "TechBuildingBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MinefieldBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x017 | FacetMask::LAND_MINE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BattleBusSlowDeathBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::SLOW_DEATH.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "JetSlowDeathBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::SLOW_DEATH.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RailroadBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x011 | FacetMask::PHYSICS_UPDATE.0), update_phase: SleepyUpdatePhase::Physics, is_ai_module: false },
    BuiltinModuleEntry { name: "SpawnBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x007 | FacetMask::SPAWN_BEHAVIOR.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DestroyDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FXListDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x082), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CrushDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DamDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CreateCrateDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CreateObjectDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "EjectPilotDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002 | FacetMask::EJECT_PILOT_DIE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpecialPowerCompletionDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RebuildHoleExposeDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "UpgradeDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "KeepObjectDie", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x002), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "AssistedTargetingUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "AutoFindHealingUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BaseRegenerateUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x005), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StealthDetectorUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StealthUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::STEALTH_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DeletionUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SmartBombTargetHomingUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DynamicShroudClearingRangeUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DeployStyleAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "AssaultTransportAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "HordeUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::HORDE_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ToppleUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x011), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "EnemyNearUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "LifetimeUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RadiusDecalUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "EMPUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "LeafletDropBehavior", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "AutoDepositUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "WeaponBonusUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MissileAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "NeutronMissileUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::PROJECTILE_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireSpreadUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireWeaponUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FlammableUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x005), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FloatUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "TensileFormationUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HeightDieUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ChinookAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "JetAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "AIUpdateInterface", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "SupplyTruckAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "DeliverPayloadAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "HackInternetAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "DynamicGeometryInfoUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FirestormDynamicGeometryInfoUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "LaserUpdate", module_type: ModuleType::ClientUpdate, facet_mask: FacetMask(0x800), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PointDefenseLaserUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CleanupHazardUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CommandButtonHuntUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PilotFindVehicleUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DemoTrapUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ParticleUplinkCannonUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpectreGunshipUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpectreGunshipDeploymentUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BaikonurLaunchPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BattlePlanUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ProjectileStreamUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "QueueProductionExitUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RepairDockUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::DOCK_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RailedTransportDockUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::DOCK_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DefaultProductionExitUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpawnPointProductionExitUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpyVisionUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x081), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SlavedUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::SLAVED_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MobMemberSlavedUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::SLAVED_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "OCLUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpecialAbilityUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::SPECIAL_POWER_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MissileLauncherBuildingUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SupplyCenterProductionExitUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SupplyCenterDockUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::DOCK_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SupplyWarehouseDockUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::DOCK_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DozerAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "RailedTransportAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "ProductionUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003 | FacetMask::PRODUCTION_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ProneUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StickyBombUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireOCLAfterWeaponCooldownUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HijackerUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StructureToppleUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StructureCollapseUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x003), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BoneFXUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RadarUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "AnimationSteeringUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "TransportAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "WanderAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "WaveGuideUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "WorkerAIUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::AI_UPDATE.0), update_phase: SleepyUpdatePhase::Initial, is_ai_module: true },
    BuiltinModuleEntry { name: "PowerPlantUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001 | FacetMask::POWER_PLANT_UPDATE.0), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CheckpointUpdate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x001), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CostModifierUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ActiveShroudUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ArmorUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CommandSetUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "GrantScienceUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PassengersFireUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StatusBitsUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SubObjectsUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StealthUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "RadarUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PowerPlantUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "LocomotorSetUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ObjectCreationUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ReplaceObjectUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ModelConditionUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "UnpauseSpecialPowerUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "WeaponBonusUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "WeaponSetUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ExperienceScalarUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MaxHealthUpgrade", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x080), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "LockWeaponCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "PreorderCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SupplyCenterCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SupplyWarehouseCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpecialPowerCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "GrantUpgradeCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "VeterancyGainCreate", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x008), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BoneFXDamage", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x004), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "TransitionDamageFX", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x004), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireWeaponCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SquishCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HealCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "MoneyCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ShroudCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "UnitCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "VeterancyCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ConvertToCarBombCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ConvertToHijackedVehicleCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageCommandCenterCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageFakeBuildingCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageInternetCenterCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageMilitaryFactoryCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotagePowerPlantCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageSuperweaponCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageSupplyCenterCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SabotageSupplyDropzoneCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SalvageCrateCollide", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x010), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "InactiveBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ActiveBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HighlanderBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "ImmortalBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "StructureBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "HiveStructureBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "UndeadBody", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x020), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CashHackSpecialPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "DefectorSpecialPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "OCLSpecialPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "FireWeaponPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpecialAbility", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SpyVisionSpecialPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CashBountyPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "CleanupAreaPower", module_type: ModuleType::Behavior, facet_mask: FacetMask(0x100), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "AnimatedParticleSysBoneClientUpdate", module_type: ModuleType::ClientUpdate, facet_mask: FacetMask(0x800), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "SwayClientUpdate", module_type: ModuleType::ClientUpdate, facet_mask: FacetMask(0x800), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
    BuiltinModuleEntry { name: "BeaconClientUpdate", module_type: ModuleType::ClientUpdate, facet_mask: FacetMask(0x800), update_phase: SleepyUpdatePhase::Normal, is_ai_module: false },
];

/// Draw modules registered by `W3DModuleFactory::init()` (`W3DModuleFactory.cpp:62-84`).
pub const W3D_DRAW_STUB_NAMES: [&str; 19] = [
    "W3DDefaultDraw",
    "W3DDebrisDraw",
    "W3DModelDraw",
    "W3DLaserDraw",
    "W3DOverlordTankDraw",
    "W3DOverlordTruckDraw",
    "W3DOverlordAircraftDraw",
    "W3DProjectileStreamDraw",
    "W3DPoliceCarDraw",
    "W3DRopeDraw",
    "W3DScienceModelDraw",
    "W3DSupplyDraw",
    "W3DDependencyModelDraw",
    "W3DTankDraw",
    "W3DTruckDraw",
    "W3DTracerDraw",
    "W3DTankTruckDraw",
    "W3DTreeDraw",
    "W3DPropDraw",
];

/// Immutable store owning all parsed template [`ModuleData`] entries (`ModuleFactory::m_moduleDataList`).
#[derive(Default)]
pub struct ModuleDataStore {
    entries: Vec<Box<dyn ModuleData>>,
    base_len: usize,
    by_template_and_tag: BTreeMap<(ThingTemplateId, NameKeyType), ModuleDataId>,
}

impl std::fmt::Debug for ModuleDataStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleDataStore")
            .field("len", &self.entries.len())
            .field("base_len", &self.base_len)
            .field("mapped_count", &self.by_template_and_tag.len())
            .finish()
    }
}

impl ModuleDataStore {
    pub fn new() -> Self {
        Self::default()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Appends `data` and returns its 1-based [`ModuleDataId`].
    pub fn insert(&mut self, data: Box<dyn ModuleData>) -> ModuleDataId {
        self.entries.push(data);
        ModuleDataId(self.entries.len() as u32)
    }

    /// Appends `data` and associates it with `(template_id, tag_key)` for Pattern-3 FFI lookup.
    pub fn insert_for_template(
        &mut self,
        template_id: ThingTemplateId,
        tag_key: NameKeyType,
        data: Box<dyn ModuleData>,
    ) -> ModuleDataId {
        let id = self.insert(data);
        if template_id != ThingTemplateId::INVALID && tag_key != NameKeyType::INVALID {
            self.by_template_and_tag.insert((template_id, tag_key), id);
        }
        id
    }

    #[inline]
    pub fn get(&self, id: ModuleDataId) -> Option<&dyn ModuleData> {
        if id.0 == 0 {
            return None;
        }
        self.entries.get((id.0 as usize) - 1).map(Box::as_ref)
    }

    #[inline]
    pub fn find_for_template(
        &self,
        template_id: ThingTemplateId,
        tag_key: NameKeyType,
    ) -> Option<ModuleDataId> {
        self.by_template_and_tag
            .get(&(template_id, tag_key))
            .copied()
    }

    /// Marks the end of base INI loading so map overrides can be truncated cleanly in [`Self::reset`].
    #[inline]
    pub fn mark_base_loaded(&mut self) {
        self.base_len = self.entries.len();
    }

    /// Discards map-local `ModuleData` entries added after [`Self::mark_base_loaded`].
    pub fn reset(&mut self) {
        self.entries.truncate(self.base_len);
        self.by_template_and_tag
            .retain(|_, id| (id.0 as usize) <= self.base_len);
    }
}

/// Registered module metadata and optional parser/constructor hooks (`ModuleTemplate`).
#[derive(Clone)]
pub struct ModuleDescriptor {
    pub name: String,
    pub module_type: ModuleType,
    pub decorated_key: NameKeyType,
    pub facet_mask: FacetMask,
    pub update_phase: SleepyUpdatePhase,
    pub is_ai_module: bool,
    pub parse_data: Option<ModuleDataParseFn>,
    pub create_module: Option<ModuleCreateFn>,
}

impl std::fmt::Debug for ModuleDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModuleDescriptor")
            .field("name", &self.name)
            .field("module_type", &self.module_type)
            .field("decorated_key", &self.decorated_key)
            .field("facet_mask", &self.facet_mask)
            .field("update_phase", &self.update_phase)
            .field("is_ai_module", &self.is_ai_module)
            .field("has_parser", &self.parse_data.is_some())
            .field("is_ported", &self.create_module.is_some())
            .finish()
    }
}

/// Sub-blocks of module data that close with their own `End`. `Turret` and `AltTurret`
/// open one only when nothing follows them: draw modules use `Turret = <bone>`.
const NESTED_MODULE_BLOCKS: [&str; 7] = [
    "ConditionState",
    "DefaultConditionState",
    "ModelConditionState",
    "TransitionState",
    "DeliveryDecal",
    "AttackAreaDecal",
    "TargetingReticleDecal",
];

/// Consumes a module block that has no parser, up to its own `End`.
fn skip_module_block(ini: &mut IniReader<'_>) -> Result<(), IniError> {
    ini.read_block_fields(|field, ini| {
        let nested = match field {
            "Turret" | "AltTurret" => ini.next_token_or_null(SEPS_DEFAULT).is_none(),
            _ => NESTED_MODULE_BLOCKS.contains(&field),
        };
        if nested {
            skip_module_block(ini)
        } else {
            Ok(())
        }
    })
}

/// Deterministic registry mirroring `ModuleFactory` (`PKT-164`).
#[derive(Debug, Clone, Default)]
pub struct ModuleRegistry {
    by_key: BTreeMap<NameKeyType, usize>,
    by_name_and_type: BTreeMap<(String, Int), usize>,
    descriptors: Vec<ModuleDescriptor>,
}

impl ModuleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Computes C++ `ModuleFactory::makeDecoratedNameKey` (`ModuleFactory.cpp:600-606`).
    pub fn make_decorated_name_key(
        name_keys: &mut NameKeyGenerator,
        name: &str,
        mod_type: ModuleType,
    ) -> NameKeyType {
        let prefix = (b'0' + (mod_type as u8)) as char;
        let mut buf = String::with_capacity(1 + name.len());
        buf.push(prefix);
        buf.push_str(name);
        name_keys.name_to_key(&buf)
    }

    /// Builds a registry pre-populated with all 198 Zero Hour `ModuleFactory::init()` entries
    /// in exact C++ registration order so decorated keys match `PKT-164` bit-for-bit.
    pub fn with_builtin_zh_modules(name_keys: &mut NameKeyGenerator) -> Self {
        let mut reg = Self::new();
        for entry in BUILTIN_ZH_MODULES {
            reg.register_descriptor(
                name_keys,
                entry.name,
                entry.module_type,
                entry.facet_mask,
                entry.update_phase,
                entry.is_ai_module,
                None,
                None,
            );
        }
        reg
    }

    /// Registers the 19 `W3D*Draw` stubs from `W3DModuleFactory::init()` for retail `Object` INI parsing.
    pub fn register_w3d_draw_stubs(&mut self, name_keys: &mut NameKeyGenerator) {
        for draw_name in W3D_DRAW_STUB_NAMES {
            self.register_descriptor(
                name_keys,
                draw_name,
                ModuleType::Draw,
                FacetMask::DRAW,
                SleepyUpdatePhase::Normal,
                false,
                None,
                None,
            );
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn register_descriptor(
        &mut self,
        name_keys: &mut NameKeyGenerator,
        name: &str,
        mod_type: ModuleType,
        facet_mask: FacetMask,
        update_phase: SleepyUpdatePhase,
        is_ai_module: bool,
        parse_data: Option<ModuleDataParseFn>,
        create_module: Option<ModuleCreateFn>,
    ) -> NameKeyType {
        let decorated_key = Self::make_decorated_name_key(name_keys, name, mod_type);
        let desc = ModuleDescriptor {
            name: name.to_owned(),
            module_type: mod_type,
            decorated_key,
            facet_mask,
            update_phase,
            is_ai_module,
            parse_data,
            create_module,
        };
        if let Some(&idx) = self.by_key.get(&decorated_key) {
            self.descriptors[idx] = desc;
        } else {
            let idx = self.descriptors.len();
            self.descriptors.push(desc);
            self.by_key.insert(decorated_key, idx);
            self.by_name_and_type
                .insert((name.to_owned(), mod_type as Int), idx);
        }
        decorated_key
    }

    /// Registers parser + constructor hooks for a ported `WP-5.x` module.
    #[allow(clippy::too_many_arguments)]
    pub fn register_module(
        &mut self,
        name_keys: &mut NameKeyGenerator,
        name: &str,
        mod_type: ModuleType,
        facet_mask: FacetMask,
        update_phase: SleepyUpdatePhase,
        is_ai_module: bool,
        parse_data: ModuleDataParseFn,
        create_module: ModuleCreateFn,
    ) -> NameKeyType {
        self.register_descriptor(
            name_keys,
            name,
            mod_type,
            facet_mask,
            update_phase,
            is_ai_module,
            Some(parse_data),
            Some(create_module),
        )
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.descriptors.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.descriptors.is_empty()
    }

    #[inline]
    pub fn find_descriptor(&self, name: &str, mod_type: ModuleType) -> Option<&ModuleDescriptor> {
        if name.is_empty() {
            return None;
        }
        self.by_name_and_type
            .get(&(name.to_owned(), mod_type as Int))
            .map(|&idx| &self.descriptors[idx])
    }

    #[inline]
    pub fn find_by_decorated_key(&self, key: NameKeyType) -> Option<&ModuleDescriptor> {
        self.by_key.get(&key).map(|&idx| &self.descriptors[idx])
    }

    #[inline]
    pub fn is_ported(&self, key: NameKeyType) -> bool {
        self.find_by_decorated_key(key)
            .is_some_and(|d| d.create_module.is_some())
    }

    /// Returns the exact C++ `MODULEINTERFACE_*` mask (`0..=0x0fff`), or `0` if empty/unknown.
    pub fn find_module_interface_mask(&self, name: &str, mod_type: ModuleType) -> Int {
        self.find_descriptor(name, mod_type)
            .map_or(0, |d| d.facet_mask.to_cpp_interface_mask())
    }

    /// Parses module INI data into `store` via its registered parser, or skips the block and
    /// stores [`StubModuleData`] when the module is not yet ported.
    #[allow(clippy::too_many_arguments)]
    pub fn parse_module_data(
        &self,
        ini: &mut IniReader<'_>,
        name: &str,
        mod_type: ModuleType,
        tag_key: NameKeyType,
        mod_name_key: NameKeyType,
        store: &mut ModuleDataStore,
        template_id: ThingTemplateId,
        parse_ctx: &mut ModuleParseContext<'_>,
    ) -> Result<ModuleDataId, IniError> {
        if let Some(desc) = self.find_descriptor(name, mod_type) {
            if let Some(parse_fn) = desc.parse_data {
                let data = parse_fn(ini, tag_key, mod_name_key, parse_ctx)?;
                return Ok(store.insert_for_template(template_id, tag_key, data));
            }
            skip_module_block(ini)?;
            let stub = Box::new(StubModuleData {
                module_name: desc.name.clone(),
                tag_key,
                module_name_key: mod_name_key,
                facet_mask: desc.facet_mask,
                update_phase: desc.update_phase,
                is_ai_module: desc.is_ai_module,
            });
            return Ok(store.insert_for_template(template_id, tag_key, stub));
        }

        skip_module_block(ini)?;
        let stub = Box::new(StubModuleData {
            module_name: name.to_owned(),
            tag_key,
            module_name_key: mod_name_key,
            facet_mask: FacetMask::NONE,
            update_phase: SleepyUpdatePhase::Normal,
            is_ai_module: false,
        });
        Ok(store.insert_for_template(template_id, tag_key, stub))
    }
}

impl ModuleMaskLookup for ModuleRegistry {
    fn find_module_interface_mask(&self, name: &str, mod_type: ModuleType) -> Int {
        ModuleRegistry::find_module_interface_mask(self, name, mod_type)
    }

    fn is_ai_module_name(&self, name: &str) -> bool {
        self.find_descriptor(name, ModuleType::Behavior)
            .map_or(false, |d| d.is_ai_module)
    }
}

impl Snapshot for ModuleRegistry {
    fn crc(&self, _xfer: &mut dyn Xfer) {
        // Matches `ModuleFactory::crc`: base `ModuleData::crc` is a no-op.
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let _ = xfer.mode() == XferMode::Load;
        Ok(())
    }
}
