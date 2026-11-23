//! `PKT-163`: Base `Module`, `ModuleData`, `ObjectModule`, `BehaviorModule`, and facet sub-traits
//! (`GeneralsMD/Code/GameEngine/Include/Common/Module.h`,
//! `GeneralsMD/Code/GameEngine/Source/Common/Thing/Module.cpp`, and
//! `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BehaviorModule.h`).
//!
//! Reference C++ packets: `packets/phase5/PKT-163_Module.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt164_module_factory.rs`

use std::any::Any;

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
pub use crate::object::thing_template::ModuleType;
use crate::rules::combat_types::ArmorSetType;
use crate::rules::disabled_types::DisabledMaskType;
use crate::rules::upgrade::UpgradeMaskType;
use crate::scheduler::{SleepyUpdatePhase, UpdateModuleScheduleState};
use crate::sim::{SimApi, SimView};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

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
}

/// Primary simulation module trait (`BehaviorModule`) stored in `ModuleArena` and dispatched via
/// Take-and-Restore (`with_module_dispatch`).
pub trait BehaviorModule: Send {
    fn module_name_key(&self) -> NameKeyType;

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn on_delete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn on_capture(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _old: Int, _new: Int) {}
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
    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        None
    }
    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        None
    }
    fn as_create_mut(&mut self) -> Option<&mut dyn CreateFacet> {
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
    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
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
    fn as_physics_mut(&mut self) -> Option<&mut dyn PhysicsFacet> {
        None
    }
    fn as_stealth_mut(&mut self) -> Option<&mut dyn StealthFacet> {
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
    fn get_damage_scalar(&self) -> Real {
        1.0
    }
    fn apply_damage_scalar(&mut self, _scalar: Real) {}
    fn is_indestructible(&self) -> Bool {
        TRUE
    }
    fn set_indestructible(&mut self, _indestructible: Bool) {}
    fn internal_change_health(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _delta: Real) {}
    fn get_front_crushed(&self) -> Bool {
        FALSE
    }
    fn get_back_crushed(&self) -> Bool {
        FALSE
    }
    fn set_armor_set_flag(&mut self, _ast: ArmorSetType) {}
    fn clear_armor_set_flag(&mut self, _ast: ArmorSetType) {}
    fn test_armor_set_flag(&self, _ast: ArmorSetType) -> Bool {
        FALSE
    }
}

/// Death reaction facet (`DieModuleInterface`).
pub trait DieFacet {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo);
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
}

/// Container facet (`ContainModuleInterface`).
pub trait ContainFacet {
    fn get_contain_count(&self) -> UnsignedInt;
    fn get_contain_max(&self) -> Int {
        -1
    }
    fn is_enclosing_container_for(&self, _other: ObjectSlot) -> Bool {
        FALSE
    }
    fn is_garrisonable(&self) -> Bool {
        FALSE
    }
    fn add_to_contain(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot);
    fn remove_from_contain(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        expose_stealthed: Bool,
    );
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
    fn would_upgrade(&self, key_mask: &UpgradeMaskType) -> Bool;
    fn reset_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool;
}

/// Special-power activation facet (`SpecialPowerModuleInterface`).
pub trait SpecialPowerFacet {
    fn is_ready(&self, sim: &dyn SimView) -> Bool;
    fn get_ready_frame(&self) -> UnsignedInt;
    fn set_ready_frame(&mut self, frame: UnsignedInt);
    fn do_special_power(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _command_options: UnsignedInt,
    ) {
    }
}

/// Destruction cleanup facet (`DestroyModuleInterface`).
pub trait DestroyFacet {
    fn on_destroy(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);
}

/// AI state-machine / locomotor facet (`AIUpdateInterface`).
pub trait AiUpdateFacet {
    fn is_moving(&self) -> Bool {
        FALSE
    }
    fn is_attacking(&self) -> Bool {
        FALSE
    }
    fn is_idle(&self) -> Bool {
        TRUE
    }
    fn set_locomotor_goal_none(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn destroy_path(&mut self) {}
}

/// Rigid-body physics facet (`PhysicsBehavior`).
pub trait PhysicsFacet {
    fn apply_force(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, force: &Coord3D);
    fn velocity(&self) -> Coord3D {
        Coord3D::default()
    }
    fn is_airborne(&self) -> Bool {
        FALSE
    }
}

/// Stealth & disguise facet (`StealthUpdate`).
pub trait StealthFacet {
    fn is_disguised(&self) -> Bool {
        FALSE
    }
    fn mark_as_detected(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _frames: UnsignedInt) {
    }
}
