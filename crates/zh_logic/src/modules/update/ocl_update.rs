//! `PKT-346`: `OCLUpdate` (`OCLUpdate.h`, `OCLUpdate.cpp`): runs an object creation list
//! on a random timer (supply drop zones, civilian spawners).
//!
//! Packet: `packets/phase5/PKT-346_OCLUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt334_267_382_346_misc.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use crate::spatial::terrain::find_closest_edge_point;
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_COLON, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct FactionOclInfo {
    pub faction_name: String,
    pub ocl: Option<String>,
}

/// INI template data matching `OCLUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct OclUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub ocl: Option<String>,
    pub faction_ocl: Vec<FactionOclInfo>,
    pub min_delay: UnsignedInt,
    pub max_delay: UnsignedInt,
    pub is_create_at_edge: Bool,
    pub is_faction_triggered: Bool,
}

impl OclUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            ocl: None,
            faction_ocl: Vec::new(),
            min_delay: 0,
            max_delay: 0,
            is_create_at_edge: FALSE,
            is_faction_triggered: FALSE,
        }
    }

    /// `FactionOCL = Faction:<side> OCL:<list>`.
    fn parse_faction_ocl(
        &mut self,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Result<(), IniError> {
        if !reader
            .next_token(SEPS_COLON)?
            .eq_ignore_ascii_case("Faction")
        {
            return Err(IniError::InvalidData);
        }
        let faction_name = reader.next_token(SEPS_COLON)?.to_owned();
        if !reader.next_token(SEPS_COLON)?.eq_ignore_ascii_case("OCL") {
            return Err(IniError::InvalidData);
        }
        // C++ has already consumed the colon after `OCL`.
        let tok = reader.next_token(SEPS_COLON)?;
        let ocl = resolve_optional_asset_name(tok, parse_ctx.ocl_names);
        self.faction_ocl.push(FactionOclInfo { faction_name, ocl });
        Ok(())
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Result<(), IniError> {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("OCL") {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            self.ocl = resolve_optional_asset_name(tok, parse_ctx.ocl_names);
            Ok(())
        } else if is("FactionOCL") {
            self.parse_faction_ocl(reader, parse_ctx)
        } else if is("MinDelay") {
            reader.parse_duration_unsigned_int(&mut self.min_delay)
        } else if is("MaxDelay") {
            reader.parse_duration_unsigned_int(&mut self.max_delay)
        } else if is("CreateAtEdge") {
            reader.parse_bool(&mut self.is_create_at_edge)
        } else if is("FactionTriggered") {
            reader.parse_bool(&mut self.is_faction_triggered)
        } else {
            Err(IniError::UnknownToken)
        }
    }
}

impl ModuleData for OclUpdateModuleData {
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

/// Runtime instance of `OCLUpdate` (`PKT-346`).
pub struct OclUpdate {
    pub data: OclUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub next_creation_frame: UnsignedInt,
    pub timer_started_frame: UnsignedInt,
    pub is_faction_neutral: Bool,
    pub current_player_color: Int,
}

impl OclUpdate {
    pub fn new(data: OclUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            next_creation_frame: 0,
            timer_started_frame: 0,
            is_faction_neutral: TRUE,
            current_player_color: 0,
        }
    }

    fn set_next_creation_frame(&mut self, sim: &mut dyn SimApi) {
        // `GameLogicRandomValue` takes the unsigned delays as signed bounds.
        let delay = sim.rng_logic_int(self.data.min_delay as Int, self.data.max_delay as Int)
            as UnsignedInt;
        self.timer_started_frame = sim.frame();
        self.next_creation_frame = self.timer_started_frame.wrapping_add(delay);
    }

    /// `resetTimer`.
    pub fn reset_timer(&mut self, sim: &mut dyn SimApi) {
        self.set_next_creation_frame(sim);
    }

    /// `getCountdownPercent`.
    pub fn countdown_percent(&self, sim: &dyn SimView) -> Real {
        let now = sim.frame();
        let total = self
            .next_creation_frame
            .wrapping_sub(self.timer_started_frame);
        if total > 0 {
            return 1.0 - ((self.next_creation_frame.wrapping_sub(now) as Real) / (total as Real));
        }
        1.0
    }

    /// `getRemainingFrames`.
    pub fn remaining_frames(&self, sim: &dyn SimView) -> UnsignedInt {
        self.next_creation_frame.wrapping_sub(sim.frame())
    }

    fn should_create(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        sim.frame() >= self.next_creation_frame
            && sim.object(owner).is_some_and(|o| {
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()) == FALSE
            })
    }

    /// Follows who owns a faction-triggered object. `false` while it is neutral.
    fn track_faction(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) -> bool {
        let team = sim.object(owner).map(|o| o.team_id);
        let playable = team.is_some_and(|t| sim.player_is_playable_side(t) != FALSE);
        let color = team.map_or(0, |t| sim.player_color(t));
        if self.is_faction_neutral != FALSE {
            if playable {
                self.current_player_color = color;
                self.is_faction_neutral = FALSE;
                self.set_next_creation_frame(sim);
            }
        } else if !playable {
            self.is_faction_neutral = TRUE;
        } else if color != self.current_player_color {
            self.current_player_color = color;
            self.set_next_creation_frame(sim);
        }
        self.is_faction_neutral == FALSE
    }

    /// `OCLUpdate::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let us = ctx.owner_slot;
        let disabled = sim.object(us).is_some_and(|o| o.is_disabled() != FALSE);
        #[cfg(feature = "retail-crc")]
        if disabled {
            self.next_creation_frame = self.next_creation_frame.wrapping_add(1);
            return UpdateSleepTime::NONE;
        }
        #[cfg(not(feature = "retail-crc"))]
        if self.timer_started_frame > 0 && disabled {
            self.next_creation_frame = self.next_creation_frame.wrapping_add(1);
            self.timer_started_frame = self.timer_started_frame.wrapping_add(1);
            return UpdateSleepTime::NONE;
        }

        if self.data.is_faction_triggered != FALSE && !self.track_faction(sim, us) {
            return UpdateSleepTime::NONE;
        }

        if !self.should_create(sim, us) {
            return UpdateSleepTime::NONE;
        }
        if self.next_creation_frame == 0 {
            self.set_next_creation_frame(sim);
            return UpdateSleepTime::NONE;
        }
        self.set_next_creation_frame(sim);

        let Some((pos, team)) = sim.object(us).map(|o| (*o.thing.get_position(), o.team_id)) else {
            return UpdateSleepTime::NONE;
        };
        let creation: Coord3D = if self.data.is_create_at_edge != FALSE {
            let extent = sim.terrain_extent();
            find_closest_edge_point(sim.terrain(), &extent, &pos)
        } else {
            pos
        };

        let ocl = if self.data.is_faction_triggered != FALSE {
            let side = sim.player_side(team);
            self.data
                .faction_ocl
                .iter()
                .find(|info| info.faction_name == side)
                .and_then(|info| info.ocl.clone())
        } else {
            self.data.ocl.clone()
        };
        if let Some(ocl) = ocl {
            sim.spawn_ocl(&ocl, us, &creation, &pos);
        }
        UpdateSleepTime::NONE
    }
}

impl UpdateFacet for OclUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

impl BehaviorModule for OclUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.next_creation_frame)?;
        xfer.xfer_unsigned_int(&mut self.timer_started_frame)?;
        xfer.xfer_bool(&mut self.is_faction_neutral)?;
        xfer.xfer_int(&mut self.current_player_color)
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

pub fn parse_ocl_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = OclUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| data.parse_field(field, r, parse_ctx))?;
    Ok(Box::new(data))
}

pub fn create_ocl_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<OclUpdateModuleData>()
        .expect("OCLUpdate data type mismatch")
        .clone();
    Box::new(OclUpdate::new(typed))
}
