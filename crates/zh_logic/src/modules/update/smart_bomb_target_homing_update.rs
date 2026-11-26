//! `PKT-366`: `SmartBombTargetHomingUpdateModuleData` and `SmartBombTargetHomingUpdate`
//! (`SmartBombTargetHomingUpdate.h`, `SmartBombTargetHomingUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-366_SmartBombTargetHomingUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_a.rs`.
//!
//! `set_target_position` is called by `DeliverPayloadAIUpdate`, which is not ported.
//! Neither the target nor the received flag is part of the C++ snapshot.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, Real, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `SmartBombTargetHomingUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SmartBombTargetHomingUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub course_correction_scalar: Real,
}

impl SmartBombTargetHomingUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            course_correction_scalar: 0.99,
        }
    }
}

impl ModuleData for SmartBombTargetHomingUpdateModuleData {
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

/// Runtime instance of `SmartBombTargetHomingUpdate`.
pub struct SmartBombTargetHomingUpdate {
    pub data: SmartBombTargetHomingUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub target_received: Bool,
    pub target: Coord3D,
}

impl SmartBombTargetHomingUpdate {
    pub fn new(data: SmartBombTargetHomingUpdateModuleData) -> Self {
        Self {
            target_received: FALSE,
            target: Coord3D::default(),
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `SetTargetPosition`: a coordinate of zero length is refused.
    pub fn set_target_position(&mut self, target: &Coord3D) {
        // `length() > 0` holds exactly when the squared length is positive.
        let length_sqr = target.x * target.x + target.y * target.y + target.z * target.z;
        if length_sqr > 0.0 {
            self.target = *target;
            self.target_received = TRUE;
        }
    }
}

impl UpdateFacet for SmartBombTargetHomingUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        if self.target_received == FALSE
            || sim.object_is_significantly_above_terrain(owner) == FALSE
        {
            return UpdateSleepTime::NONE;
        }
        let Some(current) = sim.object(owner).map(|o| *o.thing.get_position()) else {
            return UpdateSleepTime::NONE;
        };

        // The `MIN` / `MAX` macros, spelled out.
        let scalar = self.data.course_correction_scalar;
        let capped = if 1.0 < scalar { 1.0 } else { scalar };
        let status_coeff: Real = if 0.0 > capped { 0.0 } else { capped };
        let target_coeff = 1.0 - status_coeff;

        let pos = Coord3D::new(
            self.target.x * target_coeff + current.x * status_coeff,
            self.target.y * target_coeff + current.y * status_coeff,
            current.z,
        );
        sim.set_object_position(owner, &pos);
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for SmartBombTargetHomingUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)
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

pub fn parse_smart_bomb_target_homing_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SmartBombTargetHomingUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("CourseCorrectionScalar") {
            r.parse_real(&mut data.course_correction_scalar)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_smart_bomb_target_homing_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SmartBombTargetHomingUpdateModuleData>()
        .expect("SmartBombTargetHomingUpdate data type mismatch")
        .clone();
    Box::new(SmartBombTargetHomingUpdate::new(typed))
}
