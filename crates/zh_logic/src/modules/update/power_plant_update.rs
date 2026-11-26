//! `PKT-351`: `PowerPlantUpdateModuleData` and `PowerPlantUpdate`
//! (`PowerPlantUpdate.h`, `PowerPlantUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-351_PowerPlantUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt358_351_radar_power.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, PowerPlantFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `PowerPlantUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct PowerPlantUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub rods_extend_time: UnsignedInt,
}

impl PowerPlantUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            rods_extend_time: 0,
        }
    }
}

impl ModuleData for PowerPlantUpdateModuleData {
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
        FacetMask(0x001 | FacetMask::POWER_PLANT_UPDATE.0)
    }
}

/// Runtime instance of `PowerPlantUpdate`.
pub struct PowerPlantUpdate {
    pub data: PowerPlantUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub extended: Bool,
}

impl PowerPlantUpdate {
    pub fn new(data: PowerPlantUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            extended: FALSE,
        }
    }

    fn clear_flags(sim: &mut dyn SimApi, owner: ObjectSlot) {
        sim.clear_model_condition_state(owner, ModelConditionFlagType::PowerPlantUpgrading);
        sim.clear_model_condition_state(owner, ModelConditionFlagType::PowerPlantUpgraded);
    }
}

impl PowerPlantFacet for PowerPlantUpdate {
    /// `extendRods`.
    fn extend_rods(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, extend: Bool) {
        let owner = ctx.owner_slot;
        if extend != FALSE {
            if self.extended == FALSE {
                if sim.object_has_drawable(owner) != FALSE {
                    sim.set_model_condition_state(
                        owner,
                        ModelConditionFlagType::PowerPlantUpgrading,
                    );
                }
                self.extended = TRUE;
                sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(self.data.rods_extend_time));
            }
        } else {
            if sim.object_has_drawable(owner) != FALSE {
                Self::clear_flags(sim, owner);
            }
            self.extended = FALSE;
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
        }
    }
}

impl UpdateFacet for PowerPlantUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if sim.object_has_drawable(ctx.owner_slot) != FALSE {
            sim.clear_model_condition_state(
                ctx.owner_slot,
                ModelConditionFlagType::PowerPlantUpgrading,
            );
            sim.set_model_condition_state(
                ctx.owner_slot,
                ModelConditionFlagType::PowerPlantUpgraded,
            );
        }
        self.extended = TRUE;
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for PowerPlantUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.extended)
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

    fn as_power_plant_mut(&mut self) -> Option<&mut dyn PowerPlantFacet> {
        Some(self)
    }
}

pub fn parse_power_plant_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = PowerPlantUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("RodsExtendTime") {
            r.parse_duration_unsigned_int(&mut data.rods_extend_time)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_power_plant_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<PowerPlantUpdateModuleData>()
        .expect("PowerPlantUpdate data type mismatch")
        .clone();
    Box::new(PowerPlantUpdate::new(typed))
}
