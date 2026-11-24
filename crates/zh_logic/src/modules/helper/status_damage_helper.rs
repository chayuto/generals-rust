//! `PKT-184`: `StatusDamageHelper` (`StatusDamageHelper.h` / `StatusDamageHelper.cpp`).

use crate::arena::ModuleContext;
use crate::modules::helper::object_helper::ObjectHelperBase;
use crate::modules::module::{BehaviorModule, HelperFacet, UpdateFacet};
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimView};
use zh_common::math::real_to_int_floor;
use zh_common::types::{NameKeyType, Real, UnsignedInt, UpdateSleepTime, XferVersion};
use zh_common::xfer::{Xfer, XferError, XferMode};

#[derive(Debug, Clone)]
pub struct StatusDamageHelper {
    pub base: ObjectHelperBase,
    pub status_to_heal: ObjectStatusType,
    pub frame_to_heal: UnsignedInt,
}

impl StatusDamageHelper {
    #[inline]
    pub fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: ObjectHelperBase::new(module_name_key, SleepyUpdatePhase::Normal),
            status_to_heal: ObjectStatusType::None,
            frame_to_heal: 0,
        }
    }

    fn clear_status_condition(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.status_to_heal != ObjectStatusType::None {
            sim.clear_status(ctx.owner_slot, self.status_to_heal);
            self.status_to_heal = ObjectStatusType::None;
            self.frame_to_heal = 0;
        }
    }
}

impl BehaviorModule for StatusDamageHelper {
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
        let mut status_raw = self.status_to_heal.as_i32();
        xfer.xfer_int(&mut status_raw)?;
        if xfer.mode() == XferMode::Load {
            self.status_to_heal =
                ObjectStatusType::from_i32(status_raw).unwrap_or(ObjectStatusType::None);
        }
        xfer.xfer_unsigned_int(&mut self.frame_to_heal)
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

impl UpdateFacet for StatusDamageHelper {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.base.bind_slot(ctx);
        self.clear_status_condition(sim, ctx);
        UpdateSleepTime::FOREVER
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

impl HelperFacet for StatusDamageHelper {
    fn do_status_damage(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        status: ObjectStatusType,
        duration: Real,
    ) {
        self.base.bind_slot(ctx);
        let duration_as_int = real_to_int_floor(duration);
        if self.status_to_heal != status {
            self.clear_status_condition(sim, ctx);
        }
        sim.set_status(ctx.owner_slot, status);
        self.status_to_heal = status;
        self.frame_to_heal = (sim.frame() as i32).wrapping_add(duration_as_int) as UnsignedInt;
        sim.set_wake_frame(
            ctx.mod_slot,
            UpdateSleepTime(duration_as_int.max(1) as UnsignedInt),
        );
    }
}
