//! `PKT-259`: Base `BehaviorModule` / `BehaviorModuleData` helpers and shared `UpdateScheduleTracker`
//! (`BehaviorModule.h`, `BehaviorModule.cpp`, `UpdateModule.h`, `UpdateModule.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::module::{BehaviorModule, ModuleData, ModuleSnapshotHeader};
use crate::scheduler::{SleepyUpdatePhase, UpdateModuleScheduleState};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use std::collections::BTreeSet;
use zh_common::types::{Int, NameKeyType, XferVersion};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// Shared base template data for pure `BehaviorModule` subclasses with no extra fields (`BehaviorModule.h`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl BehaviorModuleData {
    #[inline]
    pub const fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
        }
    }
}

impl ModuleData for BehaviorModuleData {
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
        FacetMask::NONE
    }
}

/// Tracks and serializes an `UpdateModule`'s bound [`ModuleSlot`] and sleepy schedule state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpdateScheduleTracker {
    pub mod_slot: Option<ModuleSlot>,
    pub schedule: UpdateModuleScheduleState,
}

impl UpdateScheduleTracker {
    #[inline]
    pub const fn new() -> Self {
        Self {
            mod_slot: None,
            schedule: UpdateModuleScheduleState::new(SleepyUpdatePhase::Normal),
        }
    }

    #[inline]
    pub fn bind(&mut self, ctx: ModuleContext) {
        self.mod_slot = Some(ctx.mod_slot);
    }

    /// Base `UpdateModule::crc` (`BehaviorModule::crc` -> `ObjectModule::crc` -> `Module::crc`,
    /// all of which are no-ops in C++).
    #[inline]
    pub fn crc(&self, _xfer: &mut dyn Xfer) {}

    /// Serializes `UpdateModule::xfer` while syncing live schedule fields during `XferMode::Save`.
    pub fn xfer_update_header(
        &mut self,
        xfer: &mut dyn Xfer,
        sim: &dyn SimView,
    ) -> Result<(), XferError> {
        if xfer.mode() == XferMode::Save {
            if let Some(slot) = self.mod_slot {
                if let Some(live) = sim.module_schedule(slot) {
                    self.schedule = live;
                }
            }
        }
        ModuleSnapshotHeader::xfer_update_module_header(xfer, &mut self.schedule)
    }

    /// Restores the bound slot and sleepy schedule during `loadPostProcess`.
    #[inline]
    pub fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.mod_slot = Some(ctx.mod_slot);
        sim.set_module_schedule(ctx.mod_slot, self.schedule);
    }
}

/// Runs `f` with `this` back in the arena, for a call a C++ module makes that can call it
/// back (damage to its own owner). `this` ends up with what the callbacks changed.
pub fn lend_self<T>(
    this: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    mut f: impl FnMut(&mut dyn SimApi),
) where
    T: BehaviorModule + Clone + 'static,
{
    let copy = Box::new(this.clone());
    if let Some(mut back) = sim.lend_module(ctx.mod_slot, copy, &mut f) {
        if let Some(back) = back.as_any_mut().and_then(|a| a.downcast_mut::<T>()) {
            std::mem::swap(this, back);
        }
    }
}

/// Serializes `BehaviorModule::xfer` (version `1` + `ObjectModule::xfer`).
#[inline]
pub fn xfer_behavior_module_header(xfer: &mut dyn Xfer) -> Result<(), XferError> {
    ModuleSnapshotHeader::xfer_behavior_module_header(xfer)
}

/// Resolves an optional `FXList` or `ObjectCreationList` name, mapping `"None"` to `None`.
pub fn resolve_optional_asset_name(tok: &str, known: &BTreeSet<String>) -> Option<String> {
    if tok.eq_ignore_ascii_case("None") {
        return None;
    }
    Some(
        known
            .iter()
            .find(|n| n.eq_ignore_ascii_case(tok))
            .cloned()
            .unwrap_or_else(|| tok.to_owned()),
    )
}

/// Serializes a `u8` module version checked against `current_version`.
#[inline]
pub fn xfer_module_version(
    xfer: &mut dyn Xfer,
    current_version: XferVersion,
) -> Result<XferVersion, XferError> {
    let mut version = current_version;
    xfer.xfer_version(&mut version, current_version)?;
    Ok(version)
}

/// Rebuilds the `DamageInfo` carried by a deferred `onDie` ([`DeferredSelfOp::Die`]).
pub fn damage_info_from_die_op(op: &crate::arena::DeferredSelfOp) -> Option<DamageInfo> {
    let crate::arena::DeferredSelfOp::Die {
        damage_type,
        death_type,
        source_id,
        amount,
        kill,
        actual_damage_dealt,
        actual_damage_clipped,
    } = *op
    else {
        return None;
    };
    let mut dmg = DamageInfo::default();
    dmg.input.damage_type = crate::rules::combat_types::DamageType::from_i32(damage_type as Int)?;
    dmg.input.death_type = crate::rules::combat_types::DeathType::from_i32(death_type as Int)?;
    dmg.input.source_id = source_id;
    dmg.input.amount = amount;
    dmg.input.kill = kill;
    dmg.output.actual_damage_dealt = actual_damage_dealt;
    dmg.output.actual_damage_clipped = actual_damage_clipped;
    Some(dmg)
}
