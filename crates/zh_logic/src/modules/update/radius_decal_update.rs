//! `PKT-359`: `RadiusDecalUpdate`
//! (`RadiusDecalUpdate.h`, `RadiusDecalUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-359_RadiusDecalUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_c.rs`.
//!
//! The decal itself is drawing only: `RadiusDecal::xferRadiusDecal` writes
//! nothing, so only its effect on the wake frame is kept. Decal templates are not
//! ported; `create_radius_decal` takes the template name and radius that decide
//! whether the reference decal is empty. Its callers, and the callers of
//! `set_kill_when_no_longer_attacking`, are not ported yet.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Real, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct RadiusDecalUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl RadiusDecalUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
        }
    }
}

impl ModuleData for RadiusDecalUpdateModuleData {
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
        FacetMask(0x001)
    }
}

/// Runtime instance of `RadiusDecalUpdate`.
pub struct RadiusDecalUpdate {
    pub data: RadiusDecalUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub kill_when_no_longer_attacking: Bool,
}

impl RadiusDecalUpdate {
    pub fn new(data: RadiusDecalUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            kill_when_no_longer_attacking: FALSE,
        }
    }

    /// `createRadiusDecal`.
    pub fn create_radius_decal(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        template_name: &str,
        radius: Real,
    ) {
        // Written as the reference's rejection test, so a NaN radius is accepted.
        let is_empty = template_name.is_empty() || radius <= 0.0;
        let sleep = if is_empty {
            UpdateSleepTime::FOREVER
        } else {
            UpdateSleepTime::NONE
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }

    /// `killRadiusDecal`.
    pub fn kill_radius_decal(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    /// `killWhenNoLongerAttacking`.
    pub fn set_kill_when_no_longer_attacking(&mut self, value: Bool) {
        self.kill_when_no_longer_attacking = value;
    }
}

impl UpdateFacet for RadiusDecalUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let attacking = sim
            .object(ctx.owner_slot)
            .is_some_and(|obj| obj.status.test(ObjectStatusType::IsAttacking.as_i32()) != FALSE);
        if self.kill_when_no_longer_attacking != FALSE && !attacking {
            return UpdateSleepTime::FOREVER;
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for RadiusDecalUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.kill_when_no_longer_attacking)
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
}

pub fn create_radius_decal_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<RadiusDecalUpdateModuleData>()
        .expect("RadiusDecalUpdate data type mismatch")
        .clone();
    Box::new(RadiusDecalUpdate::new(typed))
}

pub fn parse_radius_decal_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(RadiusDecalUpdateModuleData::new(
        tag_key,
        module_name_key,
    )))
}
