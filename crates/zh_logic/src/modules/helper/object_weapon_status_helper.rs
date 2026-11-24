//! `PKT-181`: `ObjectWeaponStatusHelper` (`ObjectWeaponStatusHelper.h` / `ObjectWeaponStatusHelper.cpp`).

use crate::arena::ModuleContext;
use crate::modules::helper::object_helper::ObjectHelperBase;
use crate::modules::module::{BehaviorModule, HelperFacet, UpdateFacet};
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimView};
use zh_common::types::{NameKeyType, UpdateSleepTime, XferVersion};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone)]
pub struct ObjectWeaponStatusHelper {
    pub base: ObjectHelperBase,
}

impl ObjectWeaponStatusHelper {
    #[inline]
    pub fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: ObjectHelperBase::new(module_name_key, SleepyUpdatePhase::Final),
        }
    }
}

impl BehaviorModule for ObjectWeaponStatusHelper {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.bind_slot(ctx);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer_base(xfer, sim)
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

impl UpdateFacet for ObjectWeaponStatusHelper {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.base.bind_slot(ctx);
        sim.trigger_weapon_status(ctx.owner_slot);
        UpdateSleepTime::NONE
    }

    fn get_update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Final
    }
}

impl HelperFacet for ObjectWeaponStatusHelper {}
