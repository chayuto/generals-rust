//! `PKT-298`: `TechBuildingBehaviorModuleData` and `TechBuildingBehavior`
//! (`TechBuildingBehavior.h`, `TechBuildingBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-298_TechBuildingBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c1.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{NameKeyType, TeamId, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `TechBuildingBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TechBuildingBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub pulse_fx: Option<String>,
    pub pulse_fx_rate: UnsignedInt,
}

impl TechBuildingBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            pulse_fx: None,
            pulse_fx_rate: 0,
        }
    }
}

impl ModuleData for TechBuildingBehaviorModuleData {
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
        FacetMask(0x003)
    }
}

/// Runtime instance of `TechBuildingBehavior` (`PKT-298`).
pub struct TechBuildingBehavior {
    pub data: TechBuildingBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
}

impl TechBuildingBehavior {
    pub fn new(data: TechBuildingBehaviorModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }
}

impl UpdateFacet for TechBuildingBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return UpdateSleepTime::FOREVER;
        };
        let captured = sim.player_is_playable_side(team) != FALSE;
        if captured {
            sim.set_model_condition_state(owner, ModelConditionFlagType::Captured);
        } else {
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Captured);
        }

        match &self.data.pulse_fx {
            Some(fx) if self.data.pulse_fx_rate > 0 && captured => {
                if let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) {
                    sim.trigger_fx_list(fx, &pos);
                }
                UpdateSleepTime(self.data.pulse_fx_rate)
            }
            _ => UpdateSleepTime::FOREVER,
        }
    }
}

impl DieFacet for TechBuildingBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        let owner = ctx.owner_slot;
        sim.clear_model_condition_state(owner, ModelConditionFlagType::Captured);
        let Some(old_team) = sim.object(owner).map(|o| o.team_id) else {
            return;
        };
        let neutral = sim.player_default_team(sim.neutral_team_id());
        sim.set_object_team(owner, neutral);
        // `Object::setTeam` calls `onCapture` on every module when the team changes. This
        // module is executing, so its own wake-up is applied here.
        if old_team != neutral {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        }
    }
}

impl BehaviorModule for TechBuildingBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        // At least one evaluation once the module is in the world.
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _old_team: TeamId,
        _new_team: TeamId,
    ) {
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

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

pub fn parse_tech_building_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = TechBuildingBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("PulseFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.pulse_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("PulseFXRate") {
            r.parse_duration_unsigned_int(&mut data.pulse_fx_rate)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_tech_building_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<TechBuildingBehaviorModuleData>()
        .expect("TechBuildingBehavior data type mismatch")
        .clone();
    Box::new(TechBuildingBehavior::new(typed))
}
