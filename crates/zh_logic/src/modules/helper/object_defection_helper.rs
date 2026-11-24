//! `PKT-177`: `ObjectDefectionHelper` (`ObjectDefectionHelper.h` / `ObjectDefectionHelper.cpp`).

use crate::arena::ModuleContext;
use crate::modules::helper::object_helper::ObjectHelperBase;
use crate::modules::module::{BehaviorModule, HelperFacet, UpdateFacet};
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimView};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, XferVersion, FALSE,
};
use zh_common::xfer::{Xfer, XferError};

/// `DEFECTION_DETECTION_TIME_MAX = LOGICFRAMES_PER_SECOND * 10` (`ObjectDefectionHelper.h`).
pub const DEFECTION_DETECTION_TIME_MAX: UnsignedInt = 30 * 10;

#[derive(Debug, Clone)]
pub struct ObjectDefectionHelper {
    pub base: ObjectHelperBase,
    pub defection_detection_start: UnsignedInt,
    pub defection_detection_end: UnsignedInt,
    pub defection_detection_flash_phase: Real,
    pub do_defector_fx: Bool,
}

impl ObjectDefectionHelper {
    #[inline]
    pub fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: ObjectHelperBase::new(module_name_key, SleepyUpdatePhase::Normal),
            defection_detection_start: 0,
            defection_detection_end: 0,
            defection_detection_flash_phase: 0.0,
            do_defector_fx: FALSE,
        }
    }
}

impl BehaviorModule for ObjectDefectionHelper {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.bind_slot(ctx);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer_base(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.defection_detection_start)?;
        xfer.xfer_unsigned_int(&mut self.defection_detection_end)?;
        xfer.xfer_real(&mut self.defection_detection_flash_phase)?;
        xfer.xfer_bool(&mut self.do_defector_fx)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.load_post_process_base(sim, ctx);
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_helper(&self) -> Option<&dyn HelperFacet> {
        Some(self)
    }

    fn as_helper_mut(&mut self) -> Option<&mut dyn HelperFacet> {
        Some(self)
    }
}

impl UpdateFacet for ObjectDefectionHelper {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.base.bind_slot(ctx);
        let Some(obj) = sim.object(ctx.owner_slot) else {
            return UpdateSleepTime::FOREVER;
        };
        if obj.is_undetected_defector() == FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let now = sim.frame();
        if now >= self.defection_detection_end {
            if let Some(obj_mut) = sim.object_mut(ctx.owner_slot) {
                obj_mut.set_undetected_defector(FALSE);
            }
            return UpdateSleepTime::FOREVER;
        }
        let dead_or_firing = obj.is_effectively_dead() != FALSE
            || obj
                .status
                .test(ObjectStatusType::IsFiringWeapon.as_i32())
                != FALSE;
        if dead_or_firing {
            if let Some(obj_mut) = sim.object_mut(ctx.owner_slot) {
                obj_mut.set_undetected_defector(FALSE);
            }
            return UpdateSleepTime::FOREVER;
        }
        if self.do_defector_fx != FALSE {
            let last_phase = (self.defection_detection_flash_phase as Int) & 1;
            let time_left = self.defection_detection_end.wrapping_sub(now);
            self.defection_detection_flash_phase += 0.5_f32
                * (1.0_f32 - ((time_left as Real) / (DEFECTION_DETECTION_TIME_MAX as Real)));
            let this_phase = (self.defection_detection_flash_phase as Int) & 1;
            let _ = (last_phase, this_phase);
        }
        UpdateSleepTime::NONE
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

impl HelperFacet for ObjectDefectionHelper {
    fn start_defection_timer(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        num_frames: UnsignedInt,
        with_fx: Bool,
    ) {
        self.base.bind_slot(ctx);
        let is_undetected = sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.is_undetected_defector() != FALSE);
        if !is_undetected {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
            return;
        }
        let now = sim.frame();
        self.defection_detection_start = now;
        self.defection_detection_end = now.saturating_add(num_frames);
        self.defection_detection_flash_phase = 0.0;
        self.do_defector_fx = with_fx;
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }
}
