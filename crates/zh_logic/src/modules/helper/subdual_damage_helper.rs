//! `PKT-186`: `SubdualDamageHelper` (`SubdualDamageHelper.h` / `SubdualDamageHelper.cpp`).

use crate::arena::ModuleContext;
use crate::combat::damage::DamageInfo;
use crate::modules::helper::object_helper::ObjectHelperBase;
use crate::modules::module::{BehaviorModule, HelperFacet, UpdateFacet};
use crate::rules::combat_types::DamageType;
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimView};
use zh_common::types::{NameKeyType, Real, UnsignedInt, UpdateSleepTime, XferVersion, FALSE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone)]
pub struct SubdualDamageHelper {
    pub base: ObjectHelperBase,
    pub healing_step_countdown: UnsignedInt,
}

impl SubdualDamageHelper {
    #[inline]
    pub fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: ObjectHelperBase::new(module_name_key, SleepyUpdatePhase::Normal),
            healing_step_countdown: 0,
        }
    }
}

impl BehaviorModule for SubdualDamageHelper {
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
        xfer.xfer_unsigned_int(&mut self.healing_step_countdown)
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

impl UpdateFacet for SubdualDamageHelper {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.base.bind_slot(ctx);
        self.healing_step_countdown = self.healing_step_countdown.wrapping_sub(1);
        if self.healing_step_countdown > 0 {
            return UpdateSleepTime::NONE;
        }
        self.healing_step_countdown = sim.get_subdual_damage_heal_rate(ctx.owner_slot);
        let heal_amount = sim.get_subdual_damage_heal_amount(ctx.owner_slot);
        let mut remove_subdue = DamageInfo::default();
        remove_subdue.input.damage_type = DamageType::SubdualUnresistable;
        remove_subdue.input.amount = -heal_amount;
        sim.attempt_damage(ctx.owner_slot, &mut remove_subdue);
        if sim.has_any_subdual_damage(ctx.owner_slot) != FALSE {
            UpdateSleepTime::NONE
        } else {
            UpdateSleepTime::FOREVER
        }
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

impl HelperFacet for SubdualDamageHelper {
    fn notify_subdual_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, amount: Real) {
        self.base.bind_slot(ctx);
        if amount > 0.0 {
            self.healing_step_countdown = sim.get_subdual_damage_heal_rate(ctx.owner_slot);
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        }
    }
}
