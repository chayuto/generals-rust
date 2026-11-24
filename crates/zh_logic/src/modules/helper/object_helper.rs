//! `PKT-178`: Base `ObjectHelper` runtime support & `UpdateModule` snapshot chaining
//! (`ObjectHelper.h` / `ObjectHelper.cpp`).

use crate::arena::{ModuleContext, ModuleSlot};
use crate::modules::module::ModuleSnapshotHeader;
use crate::scheduler::{SleepyUpdatePhase, UpdateModuleScheduleState, UPDATE_SLEEP_FOREVER};
use crate::sim::{SimApi, SimView};
use zh_common::types::{NameKeyType, UnsignedInt, UpdateSleepTime, XferVersion};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// Shared base state embedded by all internal `ObjectHelper` subclasses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjectHelperBase {
    pub module_name_key: NameKeyType,
    pub mod_slot: ModuleSlot,
    pub schedule: UpdateModuleScheduleState,
}

impl ObjectHelperBase {
    /// Initializes a helper base whose `ObjectHelper` constructor schedules `UPDATE_SLEEP_FOREVER`.
    #[inline]
    pub fn new(module_name_key: NameKeyType, phase: SleepyUpdatePhase) -> Self {
        let mut schedule = UpdateModuleScheduleState::new(phase);
        schedule.set_raw_priority((UPDATE_SLEEP_FOREVER << 2) | (phase as u32));
        Self {
            module_name_key,
            mod_slot: ModuleSlot::INVALID,
            schedule,
        }
    }

    /// Binds `ctx.mod_slot` when attached inside `ModuleArena`.
    #[inline]
    pub fn bind_slot(&mut self, ctx: ModuleContext) {
        if ctx.mod_slot != ModuleSlot::INVALID {
            self.mod_slot = ctx.mod_slot;
        }
    }

    /// Wakes the owning helper on absolute frame `when` (`ObjectHelper::sleepUntil`).
    pub fn sleep_until(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, when: UnsignedInt) {
        self.bind_slot(ctx);
        if sim.object(ctx.owner_slot).is_some_and(|o| o.is_destroyed()) {
            return;
        }
        let wake_delay = if when == 0 || when == u32::MAX || when >= UpdateSleepTime::FOREVER.0 {
            UpdateSleepTime::FOREVER
        } else {
            let delta = when.saturating_sub(sim.frame());
            UpdateSleepTime(delta.max(UpdateSleepTime::NONE.0))
        };
        sim.set_wake_frame(ctx.mod_slot, wake_delay);
    }

    /// Serializes `ObjectHelper::xfer` (`version = 1` + `UpdateModule::xfer`) using live
    /// scheduler state when attached to `SimView`.
    pub fn xfer_base(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        if xfer.mode() == XferMode::Save && self.mod_slot != ModuleSlot::INVALID {
            if let Some(live) = sim.module_schedule(self.mod_slot) {
                self.schedule = live;
            }
        }
        ModuleSnapshotHeader::xfer_update_module_header(xfer, &mut self.schedule)
    }

    /// Restores schedule state into live `ModuleMeta` during `loadPostProcess`.
    #[inline]
    pub fn load_post_process_base(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.bind_slot(ctx);
        if ctx.mod_slot != ModuleSlot::INVALID {
            sim.set_module_schedule(ctx.mod_slot, self.schedule);
        }
    }
}

/// Concrete standalone `ObjectHelper` wrapper for `PKT-178` unit/golden snapshot checks.
#[derive(Debug, Clone)]
pub struct ObjectHelper {
    pub base: ObjectHelperBase,
}

impl ObjectHelper {
    #[inline]
    pub fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: ObjectHelperBase::new(module_name_key, SleepyUpdatePhase::Normal),
        }
    }

    pub fn sleep_until(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, when: UnsignedInt) {
        self.base.sleep_until(sim, ctx, when);
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.base.xfer_base(xfer, sim)
    }
}
