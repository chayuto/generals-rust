//! `PKT-174`: `ImmortalBody` (`ImmortalBody.h` / `ImmortalBody.cpp`).

use crate::arena::{DeferredSelfOp, ModuleContext, ModuleSidecar};
use crate::combat::damage::DamageInfo;
use crate::delegate_body_facet_to_inner;
use crate::modules::body::active_body::{ActiveBody, ActiveBodyModuleData};
use crate::modules::module::{BehaviorModule, BodyFacet, ModuleData};
use crate::sim::{SimApi, SimView};
use zh_common::types::{NameKeyType, XferVersion};
use zh_common::xfer::{Xfer, XferError};

/// `ActiveBody` subclass whose health can never drop below `1.0` (`ImmortalBody.h` / `ImmortalBody.cpp`).
#[derive(Debug, Clone)]
pub struct ImmortalBody {
    pub inner: ActiveBody,
}

impl ImmortalBody {
    pub fn from_active_data(module_name_key: NameKeyType, data: &ActiveBodyModuleData) -> Self {
        Self {
            inner: ActiveBody::from_active_data(module_name_key, data, true),
        }
    }

    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let abd = data
            .as_any()
            .downcast_ref::<ActiveBodyModuleData>()
            .expect("ActiveBodyModuleData");
        Box::new(Self::from_active_data(data.module_name_key(), abd))
    }
}

impl BehaviorModule for ImmortalBody {
    fn module_name_key(&self) -> NameKeyType {
        self.inner.module_name_key()
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.inner.on_object_created(sim, ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.inner.apply_deferred_op(op, sim, ctx);
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        self.inner.sync_sidecar()
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.inner.xfer(xfer, sim)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.inner.load_post_process(sim, ctx);
    }

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl BodyFacet for ImmortalBody {
    fn attempt_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        self.inner.attempt_damage(sim, ctx, dmg);
    }

    delegate_body_facet_to_inner!(inner);
}
