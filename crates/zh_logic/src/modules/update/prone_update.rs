//! `PKT-356`: `ProneUpdateModuleData` and `ProneUpdate`
//! (`ProneUpdate.h`, `ProneUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-356_ProneUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_a.rs`.
//!
//! `go_prone` is the target of `AIUpdateInterface::privateGoProne`, which is not ported.

use crate::arena::ObjectSlot;
use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, Real, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `ProneUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct ProneUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub damage_to_frames_ratio: Real,
}

impl ProneUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            damage_to_frames_ratio: 1.0,
        }
    }
}

impl ModuleData for ProneUpdateModuleData {
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

/// Runtime instance of `ProneUpdate`.
pub struct ProneUpdate {
    pub data: ProneUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub prone_frames: Int,
}

impl ProneUpdate {
    pub fn new(data: ProneUpdateModuleData) -> Self {
        Self {
            prone_frames: 0,
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `goProne`.
    pub fn go_prone(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let was_prone = self.prone_frames > 0;
        let damage_taken = dmg.output.actual_damage_dealt as Int;
        // `Int += Int * Real` adds in floating point and truncates the sum.
        let scaled = damage_taken as Real * self.data.damage_to_frames_ratio;
        self.prone_frames = (self.prone_frames as Real + scaled) as Int;
        if !was_prone && self.prone_frames > 0 {
            Self::start_prone_effects(sim, ctx.owner_slot);
        }
    }

    fn start_prone_effects(sim: &mut dyn SimApi, owner: ObjectSlot) {
        if sim.object_has_drawable(owner) != FALSE {
            sim.set_model_condition_state(owner, ModelConditionFlagType::Prone);
        }
        sim.set_status(owner, ObjectStatusType::NoAttack);
    }

    fn stop_prone_effects(sim: &mut dyn SimApi, owner: ObjectSlot) {
        if sim.object_has_drawable(owner) != FALSE {
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Prone);
        }
        sim.clear_status(owner, ObjectStatusType::NoAttack);
    }
}

impl UpdateFacet for ProneUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.prone_frames > 0 {
            self.prone_frames -= 1;
            if self.prone_frames == 0 {
                Self::stop_prone_effects(sim, ctx.owner_slot);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for ProneUpdate {
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
        xfer.xfer_int(&mut self.prone_frames)
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

pub fn parse_prone_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = ProneUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("DamageToFramesRatio") {
            r.parse_real(&mut data.damage_to_frames_ratio)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_prone_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<ProneUpdateModuleData>()
        .expect("ProneUpdate data type mismatch")
        .clone();
    Box::new(ProneUpdate::new(typed))
}
