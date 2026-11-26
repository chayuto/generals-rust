//! `PKT-163`: Base `Module`, `ModuleData`, `ObjectModule`, `BehaviorModule`, and facet sub-traits
//! (`GeneralsMD/Code/GameEngine/Include/Common/Module.h`,
//! `GeneralsMD/Code/GameEngine/Source/Common/Thing/Module.cpp`, and
//! `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`).
//!
//! Reference C++ packets: `packets/phase5/PKT-163_Module.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt164_module_factory.rs`

use crate::rules::command_button::CommandButton;
use std::any::Any;

use crate::arena::{
    DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ModuleSlot, ObjectSlot, ThingTemplateId,
};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::{BodyDamageType, WeaponBonusConditionType};
use crate::modules::update::dock_update::DockUpdateState;
use crate::modules::update::horde_update::HordeActionType;
use crate::object::weapon_set::WeaponSlotType;
use crate::object::weapon::WEAPON_ANTI_MINE;
use crate::object::weapon_set::CanAttackResult;
pub use crate::object::thing_template::ModuleType;
use crate::rules::combat_types::ArmorSetType;
use crate::rules::disabled_types::{DisabledMaskType, DisabledType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::science::ScienceType;
use crate::rules::special_power_types::SpecialPowerType;
use crate::rules::upgrade::UpgradeMaskType;
use crate::scheduler::{SleepyUpdatePhase, UpdateModuleScheduleState};
use crate::sim::{SimApi, SimView};
use zh_common::ini::SEPS_DEFAULT;
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedInt, UnsignedShort, UpdateSleepTime,
    VeterancyLevel, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// Sentinel timestamp matching `InvalidBodyTimestamp = ~0u` in `ActiveBody.cpp`.
pub const INVALID_BODY_TIMESTAMP: UnsignedInt = u32::MAX;

/// Strategy applied to current health when `BodyModule::set_max_health` changes `max_health`
/// (`BodyModule.h`: `MaxHealthChangeType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaxHealthChangeType {
    #[default]
    SameCurrentHealth = 0,
    PreserveRatio = 1,
    AddCurrentHealthToo = 2,
    FullyHeal = 3,
}

impl MaxHealthChangeType {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::SameCurrentHealth),
            1 => Some(Self::PreserveRatio),
            2 => Some(Self::AddCurrentHealthToo),
            3 => Some(Self::FullyHeal),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const MAX_HEALTH_CHANGE_NAMES: [&str; 4] = [
    "SAME_CURRENTHEALTH",
    "PRESERVE_RATIO",
    "ADD_CURRENT_HEALTH_TOO",
    "FULLY_HEAL",
];

/// Exit door slot identifier (`UpdateModule.h`: `ExitDoorType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExitDoorType {
    #[default]
    Door1 = 0,
    Door2 = 1,
    Door3 = 2,
    Door4 = 3,
    DoorNoneAvailable = -1,
    DoorNoneNeeded = -2,
}

impl ExitDoorType {
    pub const DOOR_COUNT_MAX: usize = 4;

    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Door1),
            1 => Some(Self::Door2),
            2 => Some(Self::Door3),
            3 => Some(Self::Door4),
            -1 => Some(Self::DoorNoneAvailable),
            -2 => Some(Self::DoorNoneNeeded),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Container enter/exit queue intent (`ContainModule.h`: `ObjectEnterExitType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectEnterExitType {
    WantsToEnter = 0,
    WantsToExit = 1,
    #[default]
    WantsNeither = 2,
}

/// Evacuation disposition direction (`ContainModule.h`: `EvacDisposition`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EvacDisposition {
    #[default]
    Invalid = 0,
    ToLeft = 1,
    ToRight = 2,
    BurstFromCenter = 3,
}

/// AI engagement attitude (`AI.h`: `AttitudeType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AttitudeType {
    Sleep = -2,
    Passive = -1,
    #[default]
    Normal = 0,
    Alert = 1,
    Aggressive = 2,
    Invalid = 3,
}

/// Origin of a unit/container command (`GameCommon.h`: `CommandSourceType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CommandSourceType {
    #[default]
    FromPlayer = 0,
    FromScript = 1,
    FromAi = 2,
    FromDozer = 3,
    DefaultSwitchWeapon = 4,
}

/// Runway reservation kind (`BehaviorModule.h`: `RunwayReservationType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RunwayReservationType {
    #[default]
    Takeoff = 0,
    Landing = 1,
}

/// Bridge tower slot index (`TerrainRoads.h`: `BridgeTowerType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BridgeTowerType {
    #[default]
    FromLeft = 0,
    FromRight = 1,
    ToLeft = 2,
    ToRight = 3,
}

impl BridgeTowerType {
    pub const MAX_TOWERS: usize = 4;

    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::FromLeft),
            1 => Some(Self::FromRight),
            2 => Some(Self::ToLeft),
            3 => Some(Self::ToRight),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Bridge scaffold animation phase (`BridgeScaffoldBehavior.h`: `ScaffoldTargetMotion`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ScaffoldTargetMotion {
    #[default]
    Still = 0,
    Rise = 1,
    BuildAcross = 2,
    TearDownAcross = 3,
    Sink = 4,
}

impl ScaffoldTargetMotion {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Still),
            1 => Some(Self::Rise),
            2 => Some(Self::BuildAcross),
            3 => Some(Self::TearDownAcross),
            4 => Some(Self::Sink),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Slow-death execution milestone (`SlowDeathBehavior.h`: `SlowDeathPhaseType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SlowDeathPhaseType {
    #[default]
    Initial = 0,
    Midpoint = 1,
    Final = 2,
}

impl SlowDeathPhaseType {
    pub const COUNT: usize = 3;

    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Initial),
            1 => Some(Self::Midpoint),
            2 => Some(Self::Final),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Aircraft parking & runway waypoint geometry (`ParkingPlaceBehaviorInterface::PPInfo`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ParkingPlaceInfo {
    pub parking_space: Coord3D,
    pub parking_orientation: Real,
    pub runway_prep: Coord3D,
    pub runway_start: Coord3D,
    pub runway_end: Coord3D,
    pub runway_exit: Coord3D,
    pub runway_landing_start: Coord3D,
    pub runway_landing_end: Coord3D,
    pub runway_approach: Coord3D,
    pub hangar_internal: Coord3D,
    pub runway_takeoff_dist: Real,
    pub hangar_internal_orient: Real,
}

// Keep unused-import-free access to INI separator constant for facet helpers when needed.
const _: &str = SEPS_DEFAULT;

/// Immutable per-template module configuration loaded from INI (`ModuleData`).
pub trait ModuleData: Any + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn tag_key(&self) -> NameKeyType;
    fn module_name_key(&self) -> NameKeyType;
    fn facet_mask(&self) -> FacetMask;

    fn update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Normal
    }

    fn is_ai_module(&self) -> bool {
        false
    }

    fn subdual_damage_heal_rate(&self) -> UnsignedInt {
        0
    }

    fn subdual_damage_heal_amount(&self) -> Real {
        0.0
    }
}

/// Placeholder `ModuleData` recorded for unported modules during phased `WP-5.x` bring-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StubModuleData {
    pub module_name: String,
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub facet_mask: FacetMask,
    pub update_phase: SleepyUpdatePhase,
    pub is_ai_module: bool,
}

impl ModuleData for StubModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        self.facet_mask
    }

    fn update_phase(&self) -> SleepyUpdatePhase {
        self.update_phase
    }

    fn is_ai_module(&self) -> bool {
        self.is_ai_module
    }
}

/// Exact C++ snapshot header chain helpers for `Module` -> `ObjectModule` -> `BehaviorModule`
/// (`Module.cpp`, `BehaviorModule.cpp`, `UpdateModule.cpp`, `BodyModule.cpp`).
pub struct ModuleSnapshotHeader;

impl ModuleSnapshotHeader {
    /// Serializes `Module::xfer` (version `1`).
    pub fn xfer_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)
    }

    /// Serializes `ObjectModule::xfer` (version `1` + `Module::xfer`).
    pub fn xfer_object_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_module_header(xfer)
    }

    /// Serializes `BehaviorModule::xfer` (version `1` + `ObjectModule::xfer`).
    pub fn xfer_behavior_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_object_module_header(xfer)
    }

    /// Serializes `UpdateModule::xfer` (version `1` + `BehaviorModule::xfer` + `m_nextCallFrameAndPhase`).
    pub fn xfer_update_module_header(
        xfer: &mut dyn Xfer,
        schedule: &mut UpdateModuleScheduleState,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)?;
        schedule.xfer_schedule(xfer)
    }

    /// Serializes `BodyModule::xfer` (version `1` + `BehaviorModule::xfer` + `m_damageScalar`).
    pub fn xfer_body_module_header(
        xfer: &mut dyn Xfer,
        damage_scalar: &mut Real,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)?;
        xfer.xfer_real(damage_scalar)
    }

    /// Serializes `DieModule::xfer` (version `1` + `BehaviorModule::xfer`).
    pub fn xfer_die_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)
    }

    /// Serializes `DamageModule::xfer` (version `1` + `BehaviorModule::xfer`).
    pub fn xfer_damage_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)
    }

    /// Serializes `CollideModule::xfer` (version `1` + `BehaviorModule::xfer`).
    pub fn xfer_collide_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)
    }

    /// Serializes `CrateCollide::xfer` (version `1` + `CollideModule::xfer`).
    pub fn xfer_crate_collide_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_collide_module_header(xfer)
    }

    /// Serializes `UpgradeMux::upgradeMuxXfer` (version `1` + `m_upgradeExecuted`).
    pub fn xfer_upgrade_mux(
        xfer: &mut dyn Xfer,
        upgrade_executed: &mut Bool,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_bool(upgrade_executed)
    }

    /// Serializes `UpgradeModule::xfer` (version `1` + `BehaviorModule::xfer` + `UpgradeMux::upgradeMuxXfer`).
    pub fn xfer_upgrade_module_header(
        xfer: &mut dyn Xfer,
        upgrade_executed: &mut Bool,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)?;
        Self::xfer_upgrade_mux(xfer, upgrade_executed)
    }

    /// Serializes `SpecialPowerModule::xfer` (version `1` + `BehaviorModule::xfer` + timer state).
    pub fn xfer_special_power_module_header(
        xfer: &mut dyn Xfer,
        available_on_frame: &mut UnsignedInt,
        paused_count: &mut Int,
        paused_on_frame: &mut UnsignedInt,
        paused_percent: &mut Real,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        Self::xfer_behavior_module_header(xfer)?;
        xfer.xfer_unsigned_int(available_on_frame)?;
        xfer.xfer_int(paused_count)?;
        xfer.xfer_unsigned_int(paused_on_frame)?;
        xfer.xfer_real(paused_percent)
    }
}

/// Primary simulation module trait (`BehaviorModule`) stored in `ModuleArena` and dispatched via
/// Take-and-Restore (`with_module_dispatch`).
pub trait BehaviorModule: Send {
    fn module_name_key(&self) -> NameKeyType;

    /// Work the C++ module constructor does against the live world (PRNG draws, status bits).
    /// Runs once, in module order, while the owner is still being constructed.
    fn on_construct(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn on_object_created(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn on_delete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    /// The C++ destructor's work on the world, run by `process_destroy_list`
    /// (`Object::~Object`) after `on_delete`.
    fn on_destruct(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    /// Child objects destroyed by `on_delete` (`OpenContain::onDelete`, `HelixContain::onDelete`).
    /// Snapshotted by `SimContext::destroy_object_now` before taking the module payload so each
    /// rider's `Object::onDestroy` can synchronously re-enter `contain_remove` in C++ order.
    fn delete_child_objects(&self) -> Vec<ObjectId> {
        Vec::new()
    }
    fn on_capture(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_team: TeamId,
        _new_team: TeamId,
    ) {
    }
    fn on_disabled_edge(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _disabled: Bool) {}

    /// Applies a re-entrantly queued [`DeferredSelfOp`] immediately before arena restoration.
    fn apply_deferred_op(
        &mut self,
        _op: DeferredSelfOp,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
    ) {
    }

    /// Snapshots hot fields into [`ModuleSidecar`] before outward re-entrant calls.
    fn sync_sidecar(&self) -> ModuleSidecar {
        ModuleSidecar::None
    }

    fn crc(&self, _xfer: &mut dyn Xfer) {}
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError>;
    fn load_post_process(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        None
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        None
    }
    fn as_body(&self) -> Option<&dyn BodyFacet> {
        None
    }
    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        None
    }
    fn as_die(&self) -> Option<&dyn DieFacet> {
        None
    }
    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        None
    }
    fn as_flammable(&self) -> Option<&dyn FlammableFacet> {
        None
    }

    fn as_flammable_mut(&mut self) -> Option<&mut dyn FlammableFacet> {
        None
    }

    fn as_fire_spread_mut(&mut self) -> Option<&mut dyn FireSpreadFacet> {
        None
    }

    fn as_radar_mut(&mut self) -> Option<&mut dyn RadarFacet> {
        None
    }

    fn as_dock(&self) -> Option<&dyn DockFacet> {
        None
    }
    fn as_dock_mut(&mut self) -> Option<&mut dyn DockFacet> {
        None
    }
    fn as_topple_mut(&mut self) -> Option<&mut dyn ToppleFacet> {
        None
    }
    fn as_auto_deposit_mut(&mut self) -> Option<&mut dyn AutoDepositFacet> {
        None
    }
    fn as_power_plant_mut(&mut self) -> Option<&mut dyn PowerPlantFacet> {
        None
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        None
    }
    fn as_create_mut(&mut self) -> Option<&mut dyn CreateFacet> {
        None
    }
    fn as_collide(&self) -> Option<&dyn CollideFacet> {
        None
    }
    fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
        None
    }
    fn as_contain(&self) -> Option<&dyn ContainFacet> {
        None
    }
    fn as_contain_mut(&mut self) -> Option<&mut dyn ContainFacet> {
        None
    }
    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        None
    }
    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        None
    }
    fn as_special_power(&self) -> Option<&dyn SpecialPowerFacet> {
        None
    }
    fn as_special_power_mut(&mut self) -> Option<&mut dyn SpecialPowerFacet> {
        None
    }
    fn as_destroy_mut(&mut self) -> Option<&mut dyn DestroyFacet> {
        None
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        None
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        None
    }
    fn as_physics(&self) -> Option<&dyn PhysicsFacet> {
        None
    }
    fn as_physics_mut(&mut self) -> Option<&mut dyn PhysicsFacet> {
        None
    }
    fn as_stealth(&self) -> Option<&dyn StealthFacet> {
        None
    }
    fn as_stealth_mut(&mut self) -> Option<&mut dyn StealthFacet> {
        None
    }
    fn as_stealth_detector_mut(&mut self) -> Option<&mut dyn StealthDetectorFacet> {
        None
    }
    fn as_helper(&self) -> Option<&dyn HelperFacet> {
        None
    }
    fn as_helper_mut(&mut self) -> Option<&mut dyn HelperFacet> {
        None
    }
    fn as_slow_death(&self) -> Option<&dyn SlowDeathFacet> {
        None
    }
    fn as_slow_death_mut(&mut self) -> Option<&mut dyn SlowDeathFacet> {
        None
    }
    fn as_exit(&self) -> Option<&dyn ExitFacet> {
        None
    }
    fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
        None
    }
    fn as_projectile(&self) -> Option<&dyn ProjectileFacet> {
        None
    }
    fn as_projectile_mut(&mut self) -> Option<&mut dyn ProjectileFacet> {
        None
    }
    fn as_countermeasures(&self) -> Option<&dyn CountermeasuresFacet> {
        None
    }
    fn as_countermeasures_mut(&mut self) -> Option<&mut dyn CountermeasuresFacet> {
        None
    }
    fn as_production(&self) -> Option<&dyn ProductionFacet> {
        None
    }
    fn as_production_mut(&mut self) -> Option<&mut dyn ProductionFacet> {
        None
    }
    fn as_parking_place(&self) -> Option<&dyn ParkingPlaceFacet> {
        None
    }
    fn as_parking_place_mut(&mut self) -> Option<&mut dyn ParkingPlaceFacet> {
        None
    }
    fn as_horde(&self) -> Option<&dyn HordeFacet> {
        None
    }
    fn as_assisted_targeting_mut(&mut self) -> Option<&mut dyn AssistedTargetingFacet> {
        None
    }
    fn as_spawn_behavior(&self) -> Option<&dyn SpawnBehaviorFacet> {
        None
    }
    fn as_spawn_behavior_mut(&mut self) -> Option<&mut dyn SpawnBehaviorFacet> {
        None
    }
    fn as_bridge(&self) -> Option<&dyn BridgeFacet> {
        None
    }
    fn as_bridge_mut(&mut self) -> Option<&mut dyn BridgeFacet> {
        None
    }
    fn as_bridge_tower(&self) -> Option<&dyn BridgeTowerFacet> {
        None
    }
    fn as_bridge_tower_mut(&mut self) -> Option<&mut dyn BridgeTowerFacet> {
        None
    }
    fn as_bridge_scaffold(&self) -> Option<&dyn BridgeScaffoldFacet> {
        None
    }
    fn as_bridge_scaffold_mut(&mut self) -> Option<&mut dyn BridgeScaffoldFacet> {
        None
    }
    fn as_cave(&self) -> Option<&dyn CaveFacet> {
        None
    }
    fn as_cave_mut(&mut self) -> Option<&mut dyn CaveFacet> {
        None
    }
    fn as_land_mine(&self) -> Option<&dyn LandMineFacet> {
        None
    }
    fn as_land_mine_mut(&mut self) -> Option<&mut dyn LandMineFacet> {
        None
    }
    fn as_generate_minefield_mut(&mut self) -> Option<&mut dyn GenerateMinefieldFacet> {
        None
    }
    fn as_auto_heal_mut(&mut self) -> Option<&mut dyn AutoHealFacet> {
        None
    }
    fn as_overcharge(&self) -> Option<&dyn OverchargeFacet> {
        None
    }
    fn as_overcharge_mut(&mut self) -> Option<&mut dyn OverchargeFacet> {
        None
    }
    fn as_rebuild_hole(&self) -> Option<&dyn RebuildHoleFacet> {
        None
    }
    fn as_rebuild_hole_mut(&mut self) -> Option<&mut dyn RebuildHoleFacet> {
        None
    }
    fn as_transport_passenger(&self) -> Option<&dyn TransportPassengerFacet> {
        None
    }
    fn as_transport_passenger_mut(&mut self) -> Option<&mut dyn TransportPassengerFacet> {
        None
    }
    fn as_slaved_update(&self) -> Option<&dyn SlavedUpdateFacet> {
        None
    }
    fn as_slaved_update_mut(&mut self) -> Option<&mut dyn SlavedUpdateFacet> {
        None
    }
    fn as_any(&self) -> Option<&dyn Any> {
        None
    }
    fn as_special_power_update(&self) -> Option<&dyn SpecialPowerUpdateFacet> {
        None
    }
    fn as_special_power_update_mut(&mut self) -> Option<&mut dyn SpecialPowerUpdateFacet> {
        None
    }
    fn as_command_button_hunt_mut(&mut self) -> Option<&mut dyn CommandButtonHuntFacet> {
        None
    }
    fn as_sticky_bomb_mut(&mut self) -> Option<&mut dyn StickyBombFacet> {
        None
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        None
    }
}

/// Sleepy-update facet (`UpdateModuleInterface`).
pub trait UpdateFacet {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime;

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        DisabledMaskType::new()
    }

    fn get_update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Normal
    }

    /// Notifies `BoneFXUpdate` (via `BoneFXDamage`) when body damage state transitions.
    fn change_body_damage_state(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_state: BodyDamageType,
        _new_state: BodyDamageType,
    ) {
    }
}

/// Object health/armor body facet (`BodyModuleInterface`).
pub trait BodyFacet {
    fn attempt_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo);
    fn attempt_healing(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _healing: &mut DamageInfo,
    ) {
    }
    fn estimate_damage(&self, _sim: &dyn SimView, _ctx: ModuleContext, _dmg: &DamageInfo) -> Real {
        0.0
    }
    fn get_health(&self) -> Real;
    fn get_max_health(&self) -> Real {
        0.0
    }
    fn get_previous_health(&self) -> Real {
        0.0
    }
    fn get_initial_health(&self) -> Real {
        0.0
    }
    fn set_initial_health(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _initial_percent: Int,
    ) {
    }
    fn set_max_health(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_max: Real,
        _change_type: MaxHealthChangeType,
    ) {
    }
    fn get_damage_state(&self) -> BodyDamageType {
        BodyDamageType::Pristine
    }
    fn set_damage_state(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_state: BodyDamageType,
    ) {
    }
    fn set_aflame(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _setting: Bool) {}
    fn get_damage_scalar(&self) -> Real {
        1.0
    }
    fn apply_damage_scalar(&mut self, _scalar: Real) {}
    fn is_indestructible(&self) -> Bool {
        TRUE
    }
    fn set_indestructible(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _indestructible: Bool,
    ) {
    }
    fn internal_change_health(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _delta: Real) {}
    fn get_front_crushed(&self) -> Bool {
        FALSE
    }
    fn get_back_crushed(&self) -> Bool {
        FALSE
    }
    fn set_front_crushed(&mut self, _crushed: Bool) {}
    fn set_back_crushed(&mut self, _crushed: Bool) {}
    fn set_armor_set_flag(&mut self, _ast: ArmorSetType) {}
    fn clear_armor_set_flag(&mut self, _ast: ArmorSetType) {}
    fn test_armor_set_flag(&self, _ast: ArmorSetType) -> Bool {
        FALSE
    }
    fn get_last_damage_info(&self) -> Option<&DamageInfo> {
        None
    }
    fn get_last_damage_timestamp(&self) -> Option<UnsignedInt> {
        #[cfg(feature = "retail-crc")]
        {
            Some(0)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            None
        }
    }
    fn get_last_healing_timestamp(&self) -> Option<UnsignedInt> {
        #[cfg(feature = "retail-crc")]
        {
            Some(0)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            None
        }
    }
    fn get_clearable_last_attacker(&self) -> ObjectId {
        ObjectId::INVALID
    }
    fn clear_last_attacker(&mut self) {}
    fn clear_last_damage_info(&mut self) {
        self.clear_last_attacker();
    }
    fn get_current_subdual_damage(&self) -> Real {
        0.0
    }
    fn get_subdual_damage_cap(&self) -> Real {
        0.0
    }
    fn get_subdual_damage_heal_rate(&self) -> UnsignedInt {
        0
    }
    fn get_subdual_damage_heal_amount(&self) -> Real {
        0.0
    }
    fn has_any_subdual_damage(&self) -> Bool {
        FALSE
    }
    fn is_subdued(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn can_be_subdued(&self) -> Bool {
        FALSE
    }
    fn internal_add_subdual_damage(&mut self, _delta: Real) {}
    fn on_veterancy_level_changed(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_level: VeterancyLevel,
        _new_level: VeterancyLevel,
        _provide_feedback: Bool,
    ) {
    }
    fn evaluate_visual_condition(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn update_body_particle_systems(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn is_inactive_body(&self) -> Bool {
        FALSE
    }
    fn set_constructor_object(&mut self, _constructor_id: ObjectId) {}
}

/// Death reaction facet (`DieModuleInterface` + `EjectPilotDie` extension).
pub trait DieFacet {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo);
    fn is_die_applicable(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _dmg: &DamageInfo,
    ) -> Bool {
        TRUE
    }
    fn eject_pilot_dying(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn set_creator(&mut self, _creator_id: ObjectId) {}
    fn notify_script_engine(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
}

/// The calls `FireSpreadUpdate` makes on another object's `FlammableUpdate`.
pub trait FlammableFacet {
    /// `FlammableUpdate::wouldIgnite`.
    fn would_ignite(&self) -> Bool;
    /// `FlammableUpdate::tryToIgnite`.
    fn try_to_ignite(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// The call `FlammableUpdate` makes on its owner's `FireSpreadUpdate`.
pub trait FireSpreadFacet {
    /// `FireSpreadUpdate::startFireSpreading`.
    fn start_fire_spreading(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// The call `RadarUpgrade` makes on its owner's `RadarUpdate`.
pub trait RadarFacet {
    /// `RadarUpdate::extendRadar`.
    fn extend_radar(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// `PowerPlantUpdateInterface`.
pub trait PowerPlantFacet {
    /// `extendRods`.
    fn extend_rods(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, extend: Bool);
}

/// Damage reaction facet (`DamageModuleInterface`).
pub trait DamageFacet {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo);
    fn on_healing(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}
    fn on_body_damage_state_change(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dmg: &DamageInfo,
        _old_state: BodyDamageType,
        _new_state: BodyDamageType,
    ) {
    }
}

/// Object creation & build-completion facet (`CreateModuleInterface`).
pub trait CreateFacet {
    fn on_create(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
    fn on_build_complete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn should_do_on_build_complete(&self) -> Bool {
        FALSE
    }
}

/// The calls other code makes on a `ToppleUpdate` (`Object::topple`).
pub trait ToppleFacet {
    /// `isAbleToBeToppled`.
    fn is_able_to_be_toppled(&self) -> Bool;
    /// `applyTopplingForce`.
    fn apply_toppling_force(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        direction: &Coord3D,
        speed: Real,
        options: UnsignedInt,
    );
}

/// The call `Player::becomingTeamMember` makes on an `AutoDepositUpdate`.
pub trait AutoDepositFacet {
    /// `awardInitialCaptureBonus`, for the player now controlling the owner.
    fn award_initial_capture_bonus(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// Collision reaction facet (`CollideModuleInterface`).
pub trait CollideFacet {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    );
    fn would_like_to_collide_with(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _other: ObjectSlot,
    ) -> Bool {
        FALSE
    }
    fn is_hijacked_vehicle_crate_collide(&self) -> Bool {
        FALSE
    }
    fn is_sabotage_building_crate_collide(&self) -> Bool {
        FALSE
    }
    fn is_car_bomb_crate_collide(&self) -> Bool {
        FALSE
    }
    fn is_railroad(&self) -> Bool {
        FALSE
    }
    fn is_salvage_crate_collide(&self) -> Bool {
        FALSE
    }
}

/// Container facet (`ContainModuleInterface`).
pub trait ContainFacet {
    /// `OpenContain::m_containList`, for `getContainedItemsMass`.
    fn contain_list_for_mass(&self) -> &[ObjectId] {
        &[]
    }
    /// `isWeaponBonusPassedToPassengers`.
    fn is_weapon_bonus_passed_to_passengers(&self) -> Bool {
        FALSE
    }
    /// The count, maximum, item list, membership, garrison and capture queries take the
    /// game view because `TunnelContain` and `CaveContain` answer them from a tracker
    /// outside the module and `OverlordContain` forwards them to its rider.
    fn get_contain_count(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt;
    fn get_contain_max(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Int {
        -1
    }
    fn is_enclosing_container_for(&self, _other: ObjectSlot) -> Bool {
        FALSE
    }
    fn is_garrisonable(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn is_valid_container_for(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _obj: ObjectSlot,
        _check_capacity: Bool,
    ) -> Bool {
        FALSE
    }
    fn get_contained_items<'a>(
        &'a self,
        _sim: &'a dyn SimView,
        _ctx: ModuleContext,
    ) -> &'a [ObjectId] {
        &[]
    }
    fn add_to_contain(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot);
    fn add_to_contain_list(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _obj: ObjectSlot) {}
    fn remove_from_contain(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        expose_stealthed: Bool,
    );
    fn remove_all_contained(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _expose_stealthed: Bool,
    ) {
    }
    fn kill_all_contained(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn harm_and_force_exit_all_contained(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dmg: &DamageInfo,
    ) {
    }
    fn is_immune_to_clear_building_attacks(&self) -> Bool {
        FALSE
    }
    fn is_rider_change_contain(&self) -> Bool {
        FALSE
    }
    /// `recalcApparentControllingPlayer`; only `GarrisonContain` does anything here.
    fn recalc_apparent_controlling_player(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    /// `getApparentControllingPlayer(observer)` as a player index; `None` for no player.
    fn apparent_controlling_player_index(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _observer: Int,
    ) -> Option<Int> {
        None
    }
    fn is_bustable(&self) -> Bool {
        FALSE
    }
    fn is_heal_contain(&self) -> Bool {
        FALSE
    }
    fn is_tunnel_contain(&self) -> Bool {
        FALSE
    }
    fn is_special_overlord_style_container(&self) -> Bool {
        FALSE
    }
    fn is_special_zero_slot_container(&self) -> Bool {
        FALSE
    }
    fn is_kick_out_on_capture(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        TRUE
    }
    fn is_passenger_allowed_to_fire(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _id: ObjectId,
    ) -> Bool {
        FALSE
    }
    fn set_passenger_allowed_to_fire(&mut self, _permission: Bool) {}
    fn get_stealth_units_contained(&self) -> UnsignedInt {
        0
    }
    fn get_hero_units_contained(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt {
        0
    }
    fn get_rider_slot(&self, _sim: &dyn SimView) -> Option<ObjectSlot> {
        None
    }
    fn is_contained(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _obj_id: ObjectId,
    ) -> Bool {
        FALSE
    }
    fn get_player_entered_mask(&self) -> UnsignedShort {
        0
    }
    fn set_player_entered_mask(&mut self, _mask: UnsignedShort) {}
    fn on_object_wants_to_enter_or_exit(
        &mut self,
        _obj_id: ObjectId,
        _wants: ObjectEnterExitType,
    ) {
    }
    fn has_objects_wanting_to_enter_or_exit(&self) -> Bool {
        FALSE
    }
    fn on_containing(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _obj: ObjectSlot) {}
    fn on_removing(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _obj: ObjectSlot) {}
    fn on_selling(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn process_damage_to_contained(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _percent_damage: Real,
    ) {
    }
    fn get_closest_rider(&self, _sim: &dyn SimView, _pos: &Coord3D) -> Option<ObjectSlot> {
        None
    }
    fn order_all_passengers_to_exit(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _cmd_source: CommandSourceType,
        _instantly: Bool,
    ) {
    }
    fn order_all_passengers_to_idle(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn order_all_passengers_to_hack_internet(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn mark_all_passengers_detected(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn set_evac_disposition(&mut self, _disp: EvacDisposition) {}
    fn set_override_destination(&mut self, _pos: &Coord3D) {}
    /// `ContainModuleInterface::containReactToTransformChange`.
    fn contain_react_to_transform_change(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
}

/// Upgrade trigger facet (`UpgradeModuleInterface`).
pub trait UpgradeFacet {
    fn is_already_upgraded(&self) -> Bool;
    fn attempt_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool;
    fn would_upgrade(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool;
    fn reset_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool;
    fn test_upgrade_conditions(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _key_mask: &UpgradeMaskType,
    ) -> Bool {
        FALSE
    }
    fn get_upgrade_activation_masks(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _activation: &mut UpgradeMaskType,
        _conflicting: &mut UpgradeMaskType,
    ) {
    }
    fn perform_upgrade_fx(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn is_sub_objects_upgrade(&self) -> Bool {
        FALSE
    }
    fn give_self_upgrade(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn force_refresh_upgrade(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
}

/// Special-power activation facet (`SpecialPowerModuleInterface`).
pub trait SpecialPowerFacet {
    fn is_module_for_power(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _template_id: UnsignedInt,
    ) -> Bool {
        FALSE
    }
    fn is_ready(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool;
    fn get_percent_ready(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Real {
        0.0
    }
    fn get_ready_frame(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt;
    fn get_raw_ready_frame(&self) -> UnsignedInt;
    fn get_power_name(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> String {
        String::new()
    }
    fn get_special_power_id(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt {
        0
    }
    fn has_special_power(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _sp_type: SpecialPowerType,
    ) -> Bool {
        FALSE
    }
    fn get_required_science(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> ScienceType {
        crate::rules::science::SCIENCE_INVALID
    }
    fn set_ready_frame(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        frame: UnsignedInt,
    );
    fn is_paused(&self) -> Bool {
        FALSE
    }
    fn pause_countdown(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _pause: Bool) {}
    fn starts_ready(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn is_script_only(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn get_reference_thing_template_name(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
    ) -> Option<String> {
        None
    }
    fn on_special_power_creation(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn start_power_recharge(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn mark_special_power_triggered(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _loc: Option<&Coord3D>,
    ) {
    }
    fn do_special_power(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _command_options: UnsignedInt,
    ) {
    }
    fn do_special_power_at_object(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: ObjectSlot,
        _command_options: UnsignedInt,
    ) {
    }
    fn do_special_power_at_location(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _loc: &Coord3D,
        _angle: Real,
        _command_options: UnsignedInt,
    ) {
    }
    fn do_special_power_using_waypoints(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _pts: &[Coord3D],
        _command_options: UnsignedInt,
    ) {
    }
}

/// Polymorphic trigger facet implemented by internal `ObjectHelper` subclasses (`WP-5.1-A`).
pub trait HelperFacet {
    fn start_defection_timer(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _num_frames: UnsignedInt,
        _with_fx: Bool,
    ) {
    }
    fn do_status_damage(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _status: ObjectStatusType,
        _duration: Real,
    ) {
    }
    fn notify_subdual_damage(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _amount: Real,
    ) {
    }
    fn do_temp_weapon_bonus(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _bonus_type: WeaponBonusConditionType,
        _duration: UnsignedInt,
    ) {
    }
    fn sleep_until(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _when: UnsignedInt) {}
}

/// Slow-death facet (`SlowDeathBehaviorInterface`) used by `UndeadBody::start_second_life`.
pub trait SlowDeathFacet {
    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool;
    fn get_probability_modifier(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Int;
    fn begin_slow_death(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo);
    fn is_slow_death_in_progress(&self) -> Bool {
        FALSE
    }
}

/// `CanMakeType` (`BuildAssistant.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CanMakeType {
    Ok = 0,
    NoPrereq = 1,
    NoMoney = 2,
    FactoryIsDisabled = 3,
    QueueFull = 4,
    ParkingPlacesFull = 5,
    MaxedOutForPlayer = 6,
}

/// Build queue facet (`ProductionUpdateInterface`). Units are identified by template and
/// upgrades by name key.
pub trait ProductionFacet {
    fn production_count(&self) -> UnsignedInt;
    fn count_unit_type_in_queue(&self, tmpl: ThingTemplateId) -> UnsignedInt;
    fn is_upgrade_in_queue(&self, upgrade: NameKeyType) -> Bool;
    /// The thing template of `getSpecialPowerConstructionCommandButton`.
    fn special_power_construction_template(&self) -> Option<ThingTemplateId>;
    fn set_special_power_construction_template(&mut self, tmpl: Option<ThingTemplateId>);
    fn can_queue_create_unit(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
    ) -> CanMakeType;
    fn can_queue_upgrade(&self) -> CanMakeType;
    fn request_unique_unit_id(&mut self) -> Int;
    fn queue_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        upgrade: NameKeyType,
    ) -> Bool;
    fn cancel_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        upgrade: NameKeyType,
    ) -> Bool;
    fn queue_create_unit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
        production_id: Int,
    ) -> Bool;
    fn cancel_unit_create(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        production_id: Int,
    ) -> Bool;
    fn cancel_all_units_of_type(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        tmpl: ThingTemplateId,
    );
    fn cancel_and_refund_all_production(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
    fn set_hold_door_open(&mut self, sim: &dyn SimView, door: ExitDoorType, hold: Bool);
}

/// Exit/door dispatch facet (`ExitInterface`).
pub trait ExitFacet {
    fn is_exit_busy(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn reserve_door_for_exit(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _tmpl_id: ThingTemplateId,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        ExitDoorType::Door1
    }
    fn exit_object_via_door(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_obj: ObjectSlot,
        _exit_door: ExitDoorType,
    ) {
    }
    fn exit_object_by_budding(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_obj: ObjectSlot,
        _bud_host: Option<ObjectSlot>,
    ) {
    }
    fn unreserve_door_for_exit(&mut self, _exit_door: ExitDoorType) {}
    fn exit_object_in_a_hurry(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_obj: ObjectSlot,
    ) {
    }
    fn set_rally_point(&mut self, _pos: &Coord3D) {}
    fn get_rally_point(&self) -> Option<&Coord3D> {
        None
    }
    fn use_spawn_rally_point(&self) -> Bool {
        FALSE
    }
    /// The `ExitInterface` default clears the point and reports none.
    fn get_natural_rally_point(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        rally_point: &mut Coord3D,
        _offset: Bool,
    ) -> Bool {
        *rally_point = Coord3D::default();
        FALSE
    }
    /// The `ExitInterface` default clears the point and reports none.
    fn get_exit_position(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        exit_pos: &mut Coord3D,
    ) -> Bool {
        *exit_pos = Coord3D::default();
        FALSE
    }
}

/// Ballistic/unguided projectile facet (`ProjectileUpdateInterface`).
pub trait ProjectileFacet {
    #[allow(clippy::too_many_arguments)]
    fn projectile_launch_at_object_or_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _victim: Option<ObjectSlot>,
        _victim_pos: &Coord3D,
        _launcher: Option<ObjectSlot>,
        _weapon_slot: crate::object::weapon_set::WeaponSlotType,
        _specific_barrel_to_use: Int,
        _detonate_weapon_name: Option<&str>,
        _exhaust_sys_name: Option<&str>,
    ) {
    }
    fn projectile_fire_at_object_or_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _victim: Option<ObjectSlot>,
        _victim_pos: &Coord3D,
        _detonate_weapon_name: Option<&str>,
        _exhaust_sys_name: Option<&str>,
    ) {
    }
    fn projectile_is_armed(&self) -> Bool {
        TRUE
    }
    fn projectile_get_launcher_id(&self) -> ObjectId {
        ObjectId::INVALID
    }
    fn projectile_handle_collision(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _other: Option<ObjectSlot>,
    ) -> Bool {
        FALSE
    }
    fn set_frames_till_countermeasure_diversion_occurs(&mut self, _frames: UnsignedInt) {}
    fn projectile_now_jammed(&mut self) {}
}

/// Aircraft countermeasure flare diversion facet (`CountermeasuresBehaviorInterface`).
pub trait CountermeasuresFacet {
    fn report_missile_for_countermeasures(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _missile: ObjectSlot,
    ) {
    }
    fn calculate_countermeasure_to_divert_to(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _victim: ObjectSlot,
    ) -> ObjectId {
        ObjectId::INVALID
    }
    fn reload_countermeasures(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn is_active(&self) -> Bool {
        FALSE
    }
}

/// Airfield/carrier parking and runway reservation facet (`ParkingPlaceBehaviorInterface`).
pub trait ParkingPlaceFacet {
    fn should_reserve_door_when_queued(&self, _tmpl_id: ThingTemplateId) -> Bool {
        FALSE
    }
    fn has_available_space_for(&self, _sim: &dyn SimView, _tmpl_id: ThingTemplateId) -> Bool {
        FALSE
    }
    fn has_reserved_space(&self, _id: ObjectId) -> Bool {
        FALSE
    }
    fn get_space_index(&self, _id: ObjectId) -> Int {
        -1
    }
    fn reserve_space(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _id: ObjectId,
        _parking_offset: Real,
        _info: Option<&mut ParkingPlaceInfo>,
    ) -> Bool {
        FALSE
    }
    fn release_space(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _id: ObjectId) {}
    fn reserve_runway(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _id: ObjectId,
        _for_landing: Bool,
    ) -> Bool {
        FALSE
    }
    fn release_runway(&mut self, _id: ObjectId) {}
    fn calc_pp_info(&self, _sim: &dyn SimView, _ctx: ModuleContext, _id: ObjectId, _info: &mut ParkingPlaceInfo) {}
    fn get_runway_index(&self, _id: ObjectId) -> Int {
        -1
    }
    fn get_runway_count(&self) -> Int {
        0
    }
    fn get_runway_reservation(&self, _runway_idx: Int, _rtype: RunwayReservationType) -> ObjectId {
        ObjectId::INVALID
    }
    fn transfer_runway_reservation_to_next_in_line_for_takeoff(&mut self, _id: ObjectId) {}
    fn get_approach_height(&self) -> Real {
        0.0
    }
    fn get_landing_deck_height_offset(&self) -> Real {
        0.0
    }
    fn set_healee(&mut self, _healee: ObjectSlot, _add: Bool) {}
    fn kill_all_parked_units(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn defect_all_parked_units(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_team: TeamId,
        _detection_time: UnsignedInt,
    ) {
    }
}

/// Master/slave spawn manager facet (`SpawnBehaviorInterface`).
pub trait SpawnBehaviorFacet {
    fn may_spawn_self_task_ai(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
        _max_self_taskers_ratio: Real,
    ) -> Bool {
        FALSE
    }
    fn on_spawn_death(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dead_spawn: ObjectId,
        _dmg: Option<&DamageInfo>,
    ) {
    }
    fn get_closest_slave(&self, _sim: &dyn SimView, _pos: &Coord3D) -> Option<ObjectSlot> {
        None
    }
    fn order_slaves_to_attack_target(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: ObjectSlot,
        _max_shots_to_fire: Int,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn order_slaves_to_attack_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _pos: &Coord3D,
        _max_shots_to_fire: Int,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn get_can_any_slaves_attack_specific_target(
        &self,
        _sim: &dyn SimView,
        _attack_type: UnsignedInt,
        _target: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) -> CanAttackResult {
        CanAttackResult::NotPossible
    }
    fn get_can_any_slaves_use_weapon_against_target(
        &self,
        _sim: &dyn SimView,
        _attack_type: UnsignedInt,
        _victim: Option<ObjectSlot>,
        _pos: &Coord3D,
        _cmd_source: CommandSourceType,
    ) -> CanAttackResult {
        CanAttackResult::NotPossible
    }
    fn can_any_slaves_attack(&self, _sim: &dyn SimView) -> Bool {
        FALSE
    }
    fn order_slaves_to_go_idle(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn order_slaves_disabled_until(
        &mut self,
        _sim: &mut dyn SimApi,
        _disabled_type: DisabledType,
        _frame: UnsignedInt,
    ) {
    }
    fn order_slaves_to_clear_disabled(
        &mut self,
        _sim: &mut dyn SimApi,
        _disabled_type: DisabledType,
    ) {
    }
    fn give_slaves_stealth_upgrade(&mut self, _sim: &mut dyn SimApi, _grant_stealth: Bool) {}
    /// Asking a slave's `StealthUpdate` may reveal that slave's own slaves.
    fn are_all_slaves_stealthed(&self, _sim: &mut dyn SimApi) -> Bool {
        FALSE
    }
    fn reveal_slaves(&mut self, _sim: &mut dyn SimApi) {}
    fn do_slaves_have_freedom(&self) -> Bool {
        FALSE
    }
    fn get_slave_count(&self) -> Int {
        0
    }
    fn stop_spawning(&mut self) {}
    fn start_spawning(&mut self) {}
}

/// Bridge span coordination facet (`BridgeBehaviorInterface`).
pub trait BridgeFacet {
    fn set_tower(&mut self, tower_type: BridgeTowerType, tower_id: ObjectId);
    fn get_tower_id(&self, tower_type: BridgeTowerType) -> ObjectId;
    fn create_scaffolding(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
    fn remove_scaffolding(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
    fn is_scaffold_in_motion(&self, sim: &dyn SimView) -> Bool;
    fn is_scaffold_present(&self) -> Bool;
}

/// Bridge repair tower facet (`BridgeTowerBehaviorInterface`).
pub trait BridgeTowerFacet {
    fn set_bridge(&mut self, bridge_id: ObjectId);
    fn get_bridge_id(&self) -> ObjectId;
    fn set_tower_type(&mut self, tower_type: BridgeTowerType);
}

/// Animated bridge repair scaffold facet (`BridgeScaffoldBehaviorInterface`).
pub trait BridgeScaffoldFacet {
    fn set_positions(
        &mut self,
        create_pos: &Coord3D,
        rise_pos: &Coord3D,
        build_pos: &Coord3D,
    );
    fn set_motion(&mut self, target_motion: ScaffoldTargetMotion);
    fn get_current_motion(&self) -> ScaffoldTargetMotion;
    fn reverse_motion(&mut self);
    fn set_lateral_speed(&mut self, lateral_speed: Real);
    fn set_vertical_speed(&mut self, vertical_speed: Real);
}

/// Cave tunnel index facet (`CaveInterface`).
pub trait CaveFacet {
    fn try_to_set_cave_index(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_index: Int,
    );
    /// `TeamId::INVALID` is the null team.
    fn set_original_team(&mut self, old_team: TeamId);
    /// `CaveContain::recalcApparentControllingPlayer`, which `Player` and `StealthUpdate`
    /// call through the contain interface.
    fn recalc_apparent_controlling_player(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// Land mine arm/disarm facet (`LandMineInterface`).
pub trait LandMineFacet {
    /// `setScootParms`: send the mine from `start` to the ground at `end`.
    fn set_scoot_parms(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        start: &Coord3D,
        end: &Coord3D,
    );
    fn disarm(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// `GenerateMinefieldBehavior::setMinefieldTarget`, which `DeliverPayloadAIUpdate` calls.
pub trait GenerateMinefieldFacet {
    fn set_minefield_target(&mut self, pos: Option<&Coord3D>);
}

/// `AutoHealBehavior::stopHealing`, which `MinefieldBehavior` calls.
pub trait AutoHealFacet {
    fn stop_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// Reactor overcharge facet (`OverchargeBehaviorInterface`).
pub trait OverchargeFacet {
    fn toggle(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
    fn enable(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, enable: Bool);
    fn is_overcharge_active(&self) -> Bool;
}

/// GLA rebuild-hole reconstruction facet (`RebuildHoleBehaviorInterface`).
pub trait RebuildHoleFacet {
    fn start_rebuild_process(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        rebuild_tmpl: ThingTemplateId,
        spawner_id: ObjectId,
    );
    fn get_spawner_id(&self) -> ObjectId;
    fn get_reconstructed_building_id(&self) -> ObjectId;
    fn get_rebuild_template_id(&self) -> ThingTemplateId;
}

/// Angry-mob nexus evacuation facet (`TransportPassengerInterface`).
pub trait TransportPassengerFacet {
    fn try_to_evacuate(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        expose_stealthed_units: Bool,
    ) -> Bool;
}

/// Destruction cleanup facet (`DestroyModuleInterface`).
pub trait DestroyFacet {
    fn on_destroy(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// AI state-machine / locomotor facet (`AIUpdateInterface`).
pub trait AiUpdateFacet {
    /// `AIUpdateInterface::isClearingMines`: attacking with an anti-mine weapon.
    fn is_clearing_mines(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        let Some(obj) = sim.object(ctx.owner_slot) else {
            return FALSE;
        };
        let weapons = &obj.weapon_set;
        if obj.status.test(ObjectStatusType::IsAttacking.as_i32()) == FALSE
            || weapons.has_any_weapon() == FALSE
        {
            return FALSE;
        }
        let anti_mine = weapons
            .get_weapon_in_weapon_slot(weapons.get_cur_weapon_slot())
            .is_some_and(|w| w.template.anti_mask & WEAPON_ANTI_MINE != 0);
        if anti_mine {
            TRUE
        } else {
            FALSE
        }
    }
    /// `getSupplyTruckAIInterface`.
    fn supply_truck(&mut self) -> Option<&mut dyn SupplyTruckAiFacet> {
        None
    }
    fn is_moving(&self) -> Bool {
        FALSE
    }
    fn is_attacking(&self) -> Bool {
        FALSE
    }
    fn is_idle(&self) -> Bool {
        TRUE
    }
    fn is_ai_in_dead_state(&self) -> Bool {
        FALSE
    }
    fn get_ignored_obstacle_id(&self) -> ObjectId {
        ObjectId::INVALID
    }
    /// `AIUpdateInterface::processCollision` (unported). The reference answers `FALSE`
    /// except for blocked panicking infantry and squishable dead infantry.
    fn process_collision(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _other: ObjectSlot,
    ) -> Bool {
        FALSE
    }
    fn mark_as_dead(&mut self) {}
    fn get_enter_target(&self) -> Option<ObjectSlot> {
        None
    }
    fn ai_exit(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _container: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn set_locomotor_goal_none(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn destroy_path(&mut self) {}
    fn ai_idle(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _cmd_source: CommandSourceType) {
    }
    fn ai_hunt(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _cmd_source: CommandSourceType) {
    }
    fn ai_move_to_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _pos: &Coord3D,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_follow_path(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _path: &[Coord3D],
        _ignore_obj: Option<ObjectSlot>,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_hack_internet(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn set_attitude(&mut self, _attitude: AttitudeType) {}
    fn wake_up_and_attempt_to_target(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn is_free_to_exit(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        TRUE
    }
    fn get_current_victim(&self) -> Option<ObjectSlot> {
        None
    }
    fn get_current_victim_pos(&self) -> Option<Coord3D> {
        None
    }
    fn has_cur_locomotor(&self) -> Bool {
        FALSE
    }
    /// Orders of `AICommandInterface` only a port of the AI can carry out. Like
    /// every command, an implementation records `cmd_source` as the owner's
    /// `m_lastCommandSource`.
    fn ai_busy(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _cmd_source: CommandSourceType) {
    }
    fn ai_move_to_object(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_attack_object(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: ObjectSlot,
        _max_shots: Int,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_face_object(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_face_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _pos: &Coord3D,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_enter(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) {
    }
    /// `ignoreObstacle`; not a command.
    fn ignore_obstacle(&mut self, _obstacle: ObjectId) {}
    /// `getAttackInfo()->getPriority(victim template)`; `None` without attack info.
    fn attack_priority(&self, _sim: &dyn SimView, _victim: ObjectSlot) -> Option<Int> {
        None
    }
    fn set_locomotor_max_lift(&mut self, _max_lift: Real) {}
    fn set_locomotor_max_turn_rate(&mut self, _max_turn_rate: Real) {}
    fn set_locomotor_max_braking(&mut self, _max_braking: Real) {}
    fn set_locomotor_ultra_accurate(&mut self, _ultra: Bool) {}
    fn set_locomotor_close_enough_dist(&mut self, _dist: Real) {}
    fn set_locomotor_close_enough_dist_3d(&mut self, _is_3d: Bool) {}
    /// `AICommandInterface::aiGetHealed`.
    fn ai_get_healed(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _heal_unit: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) {
    }
    /// `FALSE` when the slot is on a turret that is disabled
    /// (`getWhichTurretForWeaponSlot`, `isTurretEnabled`).
    fn is_weapon_slot_enabled(&self, _slot: WeaponSlotType) -> Bool {
        TRUE
    }

    fn set_locomotor_use_precise_z_pos(&mut self, _precise: Bool) {}
    /// The position of the last node of `AIUpdateInterface::getPath`; `None` without a path.
    fn get_path_destination(&self) -> Option<Coord3D> {
        None
    }
    fn ai_force_attack_object(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _victim: ObjectSlot,
        _max_shots_to_fire: Int,
        _cmd_source: CommandSourceType,
    ) {
    }
    fn ai_attack_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _pos: &Coord3D,
        _max_shots_to_fire: Int,
        _cmd_source: CommandSourceType,
    ) {
    }
}

/// Marks a physics module (`PhysicsBehavior`). Its members are on the object
/// (`ObjectRecord::physics`); call the functions in `modules::update::physics_update`.
pub trait PhysicsFacet {}

/// `DockUpdateInterface`. A dock keeps its state in the module; the provided methods
/// are `DockUpdate`'s own, which a derived dock may override. A docker is named by
/// its slot, or by its id where only that is read.
pub trait DockFacet {
    fn dock_state(&self) -> &DockUpdateState;
    fn dock_state_mut(&mut self) -> &mut DockUpdateState;

    /// `action`: what the dock does for the docker this frame.
    fn action(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
        drone: Option<ObjectSlot>,
    ) -> Bool;

    fn is_clear_to_approach(&self, docker: ObjectId) -> Bool {
        self.dock_state().is_clear_to_approach(docker)
    }
    fn reserve_approach_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Option<(Coord3D, Int)> {
        self.dock_state_mut()
            .reserve_approach_position(sim, ctx, docker)
    }
    fn advance_approach_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
        index: Int,
    ) -> Option<(Coord3D, Int)> {
        self.dock_state_mut()
            .advance_approach_position(sim, ctx, docker, index)
    }
    fn is_clear_to_enter(&self, docker: ObjectId) -> Bool {
        self.dock_state().is_clear_to_enter(docker)
    }
    fn is_clear_to_advance(&self, docker: ObjectId, index: Int) -> Bool {
        self.dock_state().is_clear_to_advance(docker, index)
    }
    fn get_enter_position(
        &mut self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Coord3D {
        self.dock_state_mut().get_enter_position(sim, ctx, docker)
    }
    fn get_dock_position(
        &mut self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Coord3D {
        self.dock_state_mut().get_dock_position(sim, ctx, docker)
    }
    fn get_exit_position(
        &mut self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Coord3D {
        self.dock_state_mut().get_exit_position(sim, ctx, docker)
    }
    fn on_approach_reached(&mut self, docker: ObjectId) {
        self.dock_state_mut().on_approach_reached(docker);
    }
    fn on_enter_reached(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, docker: ObjectSlot) {
        self.dock_state_mut().on_enter_reached(sim, ctx, docker);
    }
    fn on_dock_reached(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, docker: ObjectSlot) {
        self.dock_state_mut().on_dock_reached(sim, ctx, docker);
    }
    fn on_exit_reached(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, docker: ObjectSlot) {
        self.dock_state_mut().on_exit_reached(sim, ctx, docker);
    }
    fn cancel_dock(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, docker: ObjectSlot) {
        self.dock_state_mut().cancel_dock(sim, ctx, docker);
    }
    fn is_dock_open(&self) -> Bool {
        self.dock_state().dock_open
    }
    fn set_dock_open(&mut self, open: Bool) {
        self.dock_state_mut().dock_open = open;
    }
    fn is_allow_passthrough_type(&self) -> Bool {
        self.dock_state().data.allows_passthrough
    }
    fn is_rally_point_after_dock_type(&self) -> Bool {
        FALSE
    }
    fn set_dock_crippled(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, crippled: Bool) {
        self.dock_state_mut().dock_crippled = crippled;
    }
}

/// `SupplyTruckAIInterface`, as far as the docks use it. The supply truck AIs are
/// unported, so only test stand-ins implement it so far.
pub trait SupplyTruckAiFacet {
    fn number_boxes(&self) -> Int;
    /// `getMaxBoxes`.
    fn max_boxes(&self) -> Int;
    fn upgraded_supply_boost(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int;
    fn lose_one_box(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> Bool;
    fn gain_one_box(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, remaining: Int) -> Bool;
    fn set_force_wanting_state(&mut self, wanting: Bool);
}

/// What other objects read from a `StealthUpdate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StealthParams {
    pub stealth_delay: UnsignedInt,
    pub stealth_level: UnsignedInt,
    pub order_idle_enemies_on_reveal: Bool,
    /// `Some` is `isDisguised`.
    pub disguised_template: Option<ThingTemplateId>,
    pub disguised_player_index: Int,
    pub granted_by_special_power: Bool,
    /// `isTemporaryGrant`.
    pub temporary_grant: Bool,
}

/// Stealth & disguise facet (`StealthUpdate`).
pub trait StealthFacet {
    fn is_disguised(&self) -> Bool {
        FALSE
    }
    fn stealth_params(&self) -> Option<StealthParams> {
        None
    }
    /// `markAsDetected`; zero frames means the module's own stealth delay.
    fn mark_as_detected(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _frames: UnsignedInt) {
    }
    /// `receiveGrant`.
    fn receive_grant(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _active: Bool,
        _frames: UnsignedInt,
    ) {
    }
    /// `disguiseAsObject`; no target reveals.
    fn disguise_as_object(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _target: Option<ObjectSlot>,
    ) {
    }
    /// `allowedToStealth`.
    fn allowed_to_stealth(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _stealth_owner: ObjectSlot,
    ) -> Bool {
        FALSE
    }
}


/// `CommandButtonHuntUpdate`, which scripts hand a command button.
pub trait CommandButtonHuntFacet {
    /// `setCommandButton`.
    fn set_command_button(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, button_name: &str);
}

/// `StickyBombUpdate` (unported): the two calls its owners make.
pub trait StickyBombFacet {
    /// `initStickyBomb(target, bomber)`.
    fn init_sticky_bomb(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        bomber: ObjectSlot,
    );
    /// `detonate`.
    fn detonate(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// `StealthDetectorUpdate`, which `BattlePlanUpdate` switches on and off.
pub trait StealthDetectorFacet {
    fn is_sd_enabled(&self) -> Bool;
    fn set_sd_enabled(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, enabled: Bool);
}

/// Slaved update facet (`SlavedUpdateInterface`).
pub trait SlavedUpdateFacet {
    /// `getSlaverID`.
    fn get_slaver_id(&self) -> ObjectId {
        ObjectId::INVALID
    }
    fn on_enslave(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _slaver: Option<ObjectSlot>,
    ) {
    }
    fn on_slaver_die(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dmg: Option<&DamageInfo>,
    ) {
    }
    fn on_slaver_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &DamageInfo) {}
    fn is_self_tasking(&self) -> Bool {
        FALSE
    }
}

/// `HordeUpdateInterface`.
pub trait HordeFacet {
    fn is_in_horde(&self) -> Bool;
    fn has_flag(&self) -> Bool;
    fn is_true_horde_member(&self) -> Bool;
    fn is_allowed_nationalism(&self) -> Bool;
    fn horde_action_type(&self) -> HordeActionType;
}

/// The special power module that called `initiateIntentToDoSpecialPower`. It is
/// dispatched at that moment, so calls back to it go through this reference.
pub struct SpecialPowerCaller<'a> {
    pub ctx: ModuleContext,
    pub module: &'a mut dyn SpecialPowerFacet,
}

impl SpecialPowerCaller<'_> {
    /// Runs `f` on the special power module `target`, which may be the caller.
    pub fn with(
        &mut self,
        sim: &mut dyn SimApi,
        target: ModuleSlot,
        f: &mut dyn FnMut(&mut dyn SpecialPowerFacet, &mut dyn SimApi, ModuleContext),
    ) {
        if target == self.ctx.mod_slot {
            f(&mut *self.module, sim, self.ctx);
        } else {
            sim.with_special_power_module(target, f);
        }
    }
}

/// `SpecialPowerUpdateInterface`.
pub trait SpecialPowerUpdateFacet {
    /// `initiateIntentToDoSpecialPower`: `TRUE` when this module takes the power.
    #[allow(clippy::too_many_arguments)]
    fn initiate_intent_to_do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        target_obj: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> Bool;
    /// `doesSpecialPowerUpdatePassScienceTest`.
    fn does_pass_science_test(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        TRUE
    }
    /// `isActive`.
    fn is_active(&self) -> Bool {
        FALSE
    }
    /// `isPowerCurrentlyInUse`.
    fn is_power_currently_in_use(
        &self,
        _sim: &dyn SimView,
        _command: Option<&CommandButton>,
    ) -> Bool {
        FALSE
    }
    /// `onExit`.
    fn on_exit(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _cleanup: Bool) {}
    /// `getSpecialObjectCount`.
    fn special_object_count(&self) -> UnsignedInt {
        0
    }
    /// `findSpecialObjectWithProducerID`.
    fn find_special_object_with_producer_id(
        &self,
        _sim: &dyn SimView,
        _producer: ObjectId,
    ) -> Option<ObjectSlot> {
        None
    }
}

/// `AssistedTargetingUpdate`'s two entry points.
pub trait AssistedTargetingFacet {
    fn is_free_to_assist(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> Bool;
    fn assist_attack(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        requesting: ObjectSlot,
        victim: ObjectSlot,
    );
}
