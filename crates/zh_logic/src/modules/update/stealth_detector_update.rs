//! `PKT-373`: `StealthDetectorUpdateModuleData` and `StealthDetectorUpdate`
//! (`StealthDetectorUpdate.h`, `StealthDetectorUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-373_StealthDetectorUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt374_373_stealth.rs`.
//!
//! Left out because they are client only and reach neither saved state nor the logic
//! PRNG: the radar events, messages, EVA and sounds on a discovery, the drawable hint,
//! and the whole infrared ping block (particle systems, bone lookup, ping sounds).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, StealthDetectorFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{DisabledMaskType, DisabledType};
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `StealthDetectorUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct StealthDetectorUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub update_rate: UnsignedInt,
    pub detection_range: Real,
    pub initially_disabled: Bool,
    pub ping_sound: String,
    pub loud_ping_sound: String,
    pub ir_beacon_particle_system: String,
    pub ir_particle_system: String,
    pub ir_bright_particle_system: String,
    pub ir_grid_particle_system: String,
    pub ir_particle_system_bone: String,
    pub extra_detect_kindof: KindOfMaskType,
    pub extra_detect_kindof_not: KindOfMaskType,
    pub can_detect_while_garrisoned: Bool,
    pub can_detect_while_transported: Bool,
}

impl StealthDetectorUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            update_rate: 1,
            detection_range: 0.0,
            initially_disabled: FALSE,
            ping_sound: String::new(),
            loud_ping_sound: String::new(),
            ir_beacon_particle_system: String::new(),
            ir_particle_system: String::new(),
            ir_bright_particle_system: String::new(),
            ir_grid_particle_system: String::new(),
            ir_particle_system_bone: String::new(),
            extra_detect_kindof: KindOfMaskType::new(),
            extra_detect_kindof_not: KindOfMaskType::new(),
            can_detect_while_garrisoned: FALSE,
            can_detect_while_transported: FALSE,
        }
    }
}

impl ModuleData for StealthDetectorUpdateModuleData {
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

/// Runtime instance of `StealthDetectorUpdate`.
pub struct StealthDetectorUpdate {
    pub data: StealthDetectorUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub enabled: Bool,
}

impl StealthDetectorUpdate {
    pub fn new(data: StealthDetectorUpdateModuleData) -> Self {
        let enabled = Bool::from(data.initially_disabled == FALSE);
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            enabled,
        }
    }

    fn rate(&self) -> UpdateSleepTime {
        UpdateSleepTime(self.data.update_rate)
    }

    /// The four partition filters of `update`.
    fn passes_filters(&self, sim: &dyn SimView, owner: ObjectSlot, other: ObjectSlot) -> bool {
        let (Some(me), Some(them)) = (sim.object(owner), sim.object(other)) else {
            return false;
        };
        let stealthed = them.status.test(ObjectStatusType::Stealthed.as_i32()) != FALSE;
        let stealth_garrison = sim.contain_is_garrisonable(other) != FALSE
            && sim.contain_stealth_units_contained(other) != 0;
        (stealthed || stealth_garrison)
            && sim.player_relationship(owner, other) != Relationship::Allies
            && sim.object_is_kind_of_multi(
                other,
                &self.data.extra_detect_kindof,
                &self.data.extra_detect_kindof_not,
            ) != FALSE
            && (them.private_status & PRIVATE_STATUS_OFF_MAP)
                == (me.private_status & PRIVATE_STATUS_OFF_MAP)
    }
}

impl StealthDetectorFacet for StealthDetectorUpdate {
    fn is_sd_enabled(&self) -> Bool {
        self.enabled
    }

    /// `setSDEnabled`.
    fn set_sd_enabled(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, enabled: Bool) {
        self.enabled = enabled;
        let sleep = if enabled != FALSE {
            UpdateSleepTime::NONE
        } else {
            UpdateSleepTime::FOREVER
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }
}

impl UpdateFacet for StealthDetectorUpdate {
    fn disabled_types_to_process(&self) -> DisabledMaskType {
        let mut mask = DisabledMaskType::new();
        mask.set(DisabledType::Held.as_i32(), 1);
        mask
    }

    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let Some(me) = sim.object(owner) else {
            return UpdateSleepTime::FOREVER;
        };
        if me.is_effectively_dead() != FALSE {
            return UpdateSleepTime::FOREVER;
        }
        if me.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE {
            return UpdateSleepTime::NONE;
        }
        if me.status.test(ObjectStatusType::Sold.as_i32()) != FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let team = me.team_id;
        if let Some(container) = me.contained_by {
            if sim.object_has_contain(container) != FALSE {
                let allowed = if sim.contain_is_garrisonable(container) != FALSE {
                    self.data.can_detect_while_garrisoned
                } else {
                    self.data.can_detect_while_transported
                };
                if allowed == FALSE {
                    return self.rate();
                }
            }
        }

        let mut range = sim.object_vision_range(owner);
        if self.data.detection_range > 0.0 {
            range = self.data.detection_range;
        }

        // The filters run while the list is gathered, before anything is marked.
        let found: Vec<ObjectSlot> = sim
            .find_objects_in_range_of(owner, range)
            .into_iter()
            .filter(|&other| self.passes_filters(sim, owner, other))
            .collect();

        let player = sim.player_index(team);
        let rate = self.data.update_rate;
        for them in found {
            if sim
                .object(them)
                .is_none_or(|o| o.is_effectively_dead() != FALSE)
            {
                continue;
            }
            if sim.stealth_params(them).is_some() {
                sim.mark_stealth_detected(them, rate.wrapping_add(1));
            } else if sim.contain_is_garrisonable(them) != FALSE
                && sim.contain_stealth_units_contained(them) != 0
            {
                for id in sim.contain_items(them) {
                    let Some(rider) = sim.find_object(id) else {
                        continue;
                    };
                    if sim.stealth_params(rider).is_none() {
                        continue;
                    }
                    let theirs = sim.object(rider).map(|o| sim.player_index(o.team_id));
                    if theirs != Some(player)
                        && sim.player_relationship(owner, rider) != Relationship::Allies
                    {
                        sim.mark_stealth_detected(rider, rate.wrapping_add(2));
                    }
                }
            }
        }

        self.rate()
    }
}

impl BehaviorModule for StealthDetectorUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        let sleep = if self.enabled == TRUE {
            UpdateSleepTime(sim.rng_logic_int(1, self.data.update_rate as Int) as UnsignedInt)
        } else {
            UpdateSleepTime::FOREVER
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.enabled)
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

    fn as_stealth_detector_mut(&mut self) -> Option<&mut dyn StealthDetectorFacet> {
        Some(self)
    }
}

fn parse_name(reader: &mut IniReader<'_>, store: &mut String) -> Result<(), IniError> {
    reader.next_token(SEPS_DEFAULT)?.clone_into(store);
    Ok(())
}

pub fn parse_stealth_detector_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = StealthDetectorUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("DetectionRate") {
            r.parse_duration_unsigned_int(&mut data.update_rate)
        } else if is("DetectionRange") {
            r.parse_real(&mut data.detection_range)
        } else if is("InitiallyDisabled") {
            r.parse_bool(&mut data.initially_disabled)
        } else if is("PingSound") {
            parse_name(r, &mut data.ping_sound)
        } else if is("LoudPingSound") {
            parse_name(r, &mut data.loud_ping_sound)
        } else if is("IRBeaconParticleSysName") {
            parse_name(r, &mut data.ir_beacon_particle_system)
        } else if is("IRParticleSysName") {
            parse_name(r, &mut data.ir_particle_system)
        } else if is("IRBrightParticleSysName") {
            parse_name(r, &mut data.ir_bright_particle_system)
        } else if is("IRGridParticleSysName") {
            parse_name(r, &mut data.ir_grid_particle_system)
        } else if is("IRParticleSysBone") {
            r.parse_ascii_string(&mut data.ir_particle_system_bone)
        } else if is("ExtraRequiredKindOf") {
            data.extra_detect_kindof.parse_from_ini(r, &KINDOF_NAMES)
        } else if is("ExtraForbiddenKindOf") {
            data.extra_detect_kindof_not
                .parse_from_ini(r, &KINDOF_NAMES)
        } else if is("CanDetectWhileGarrisoned") {
            r.parse_bool(&mut data.can_detect_while_garrisoned)
        } else if is("CanDetectWhileContained") {
            r.parse_bool(&mut data.can_detect_while_transported)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_stealth_detector_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<StealthDetectorUpdateModuleData>()
        .expect("StealthDetectorUpdate data type mismatch")
        .clone();
    Box::new(StealthDetectorUpdate::new(typed))
}
