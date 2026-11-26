//! `PKT-358`: `RadarUpdateModuleData` and `RadarUpdate`
//! (`RadarUpdate.h`, `RadarUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-358_RadarUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt358_351_radar_power.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, RadarFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `RadarUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct RadarUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub radar_extend_time: Real,
}

impl RadarUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            radar_extend_time: 0.0,
        }
    }
}

impl ModuleData for RadarUpdateModuleData {
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

/// Runtime instance of `RadarUpdate`.
pub struct RadarUpdate {
    pub data: RadarUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub extend_done_frame: UnsignedInt,
    pub extend_complete: Bool,
    pub radar_active: Bool,
}

impl RadarUpdate {
    pub fn new(data: RadarUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            extend_done_frame: 0,
            extend_complete: FALSE,
            radar_active: FALSE,
        }
    }
}

impl RadarFacet for RadarUpdate {
    /// `extendRadar`. `extend_complete` is not reset, so a second extension
    /// never finishes.
    fn extend_radar(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if sim.object_has_drawable(ctx.owner_slot) != FALSE {
            sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::RadarExtending);
        }
        // `UnsignedInt + Real` adds in floating point and truncates the sum.
        self.extend_done_frame = (sim.frame() as Real + self.data.radar_extend_time) as UnsignedInt;
        self.radar_active = TRUE;
    }
}

impl UpdateFacet for RadarUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.extend_done_frame == 0 || self.extend_complete == TRUE {
            return UpdateSleepTime::NONE;
        }
        if sim.frame() > self.extend_done_frame {
            self.extend_complete = TRUE;
            self.extend_done_frame = 0;
            if sim.object_has_drawable(ctx.owner_slot) != FALSE {
                sim.clear_model_condition_state(
                    ctx.owner_slot,
                    ModelConditionFlagType::RadarExtending,
                );
                sim.set_model_condition_state(
                    ctx.owner_slot,
                    ModelConditionFlagType::RadarUpgraded,
                );
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for RadarUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.extend_done_frame)?;
        xfer.xfer_bool(&mut self.extend_complete)?;
        xfer.xfer_bool(&mut self.radar_active)
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

    fn as_radar_mut(&mut self) -> Option<&mut dyn RadarFacet> {
        Some(self)
    }
}

pub fn parse_radar_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = RadarUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("RadarExtendTime") {
            r.parse_duration_real(&mut data.radar_extend_time)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_radar_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<RadarUpdateModuleData>()
        .expect("RadarUpdate data type mismatch")
        .clone();
    Box::new(RadarUpdate::new(typed))
}
