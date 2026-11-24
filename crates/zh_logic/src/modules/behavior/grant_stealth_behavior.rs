//! `PKT-273`: `GrantStealthBehaviorModuleData` and `GrantStealthBehavior`
//! (`GrantStealthBehavior.h`, `GrantStealthBehavior.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::sim::{SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{NameKeyType, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

const INVALID_PARTICLE_SYSTEM_ID: UnsignedInt = 0;

/// INI template data matching `GrantStealthBehaviorModuleData` (`GrantStealthBehavior.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct GrantStealthBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub start_radius: Real,
    pub final_radius: Real,
    pub radius_grow_rate: Real,
    pub kind_of: KindOfMaskType,
    pub radius_particle_system: Option<String>,
}

impl GrantStealthBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut kind_of = KindOfMaskType::new();
        kind_of.flip();
        Self {
            tag_key,
            module_name_key,
            start_radius: 0.0,
            final_radius: 200.0,
            radius_grow_rate: 10.0,
            kind_of,
            radius_particle_system: None,
        }
    }
}

impl ModuleData for GrantStealthBehaviorModuleData {
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

/// Runtime instance of `GrantStealthBehavior` (`PKT-273`).
pub struct GrantStealthBehavior {
    pub data: GrantStealthBehaviorModuleData,
    pub radius_particle_system_id: UnsignedInt,
    pub current_scan_radius: Real,
    pub schedule: UpdateScheduleTracker,
}

impl GrantStealthBehavior {
    pub fn new(data: GrantStealthBehaviorModuleData) -> Self {
        let current_scan_radius = data.start_radius;
        Self {
            data,
            radius_particle_system_id: INVALID_PARTICLE_SYSTEM_ID,
            current_scan_radius,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    fn grant_stealth_to_object(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        target: ObjectSlot,
    ) {
        if target == owner {
            return;
        }
        if sim.object_is_kind_of_multi(target, &self.data.kind_of, &KindOfMaskType::new()) == FALSE
            && self.data.kind_of.any() != FALSE
        {
            // `object_is_kind_of_multi` tests all; `isAnyKindOf` checks overlap.
        }
        let matches_kind = sim
            .object(target)
            .is_some_and(|o| o.thing.is_any_kind_of(&sim.rules().thing_factory, &self.data.kind_of) != FALSE);
        if !matches_kind {
            return;
        }
        sim.receive_stealth_grant(target);
    }
}

impl BehaviorModule for GrantStealthBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.current_scan_radius = self.data.start_radius;
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.radius_particle_system_id)?;
        xfer.xfer_real(&mut self.current_scan_radius)
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

impl UpdateFacet for GrantStealthBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let Some(owner_rec) = sim.object(owner) else {
            return UpdateSleepTime::FOREVER;
        };
        if owner_rec.is_effectively_dead() != FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let owner_pos = *owner_rec.thing.get_position();
        let owner_off_map = (owner_rec.private_status & PRIVATE_STATUS_OFF_MAP) != 0;

        self.current_scan_radius += self.data.radius_grow_rate;
        let mut this_is_final_scan = false;
        if self.current_scan_radius >= self.data.final_radius {
            self.current_scan_radius = self.data.final_radius;
            this_is_final_scan = true;
        }

        let mut targets = Vec::new();
        for slot in sim.find_objects_in_range(&owner_pos, self.current_scan_radius) {
            if sim.player_relationship(owner, slot) != Relationship::Allies {
                continue;
            }
            let Some(cand) = sim.object(slot) else {
                continue;
            };
            if cand.is_effectively_dead() != FALSE {
                continue;
            }
            let cand_off_map = (cand.private_status & PRIVATE_STATUS_OFF_MAP) != 0;
            if cand_off_map != owner_off_map {
                continue;
            }
            targets.push(slot);
        }

        for target in targets {
            self.grant_stealth_to_object(sim, owner, target);
        }

        if this_is_final_scan {
            sim.destroy_object(owner);
            return UpdateSleepTime::FOREVER;
        }
        UpdateSleepTime::NONE
    }
}

pub fn parse_grant_stealth_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = GrantStealthBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("StartRadius") {
            r.parse_real(&mut data.start_radius)
        } else if field.eq_ignore_ascii_case("FinalRadius") {
            r.parse_real(&mut data.final_radius)
        } else if field.eq_ignore_ascii_case("RadiusGrowRate") {
            r.parse_real(&mut data.radius_grow_rate)
        } else if field.eq_ignore_ascii_case("KindOf") {
            data.kind_of.parse_from_ini(r, &KINDOF_NAMES)
        } else if field.eq_ignore_ascii_case("RadiusParticleSystemName") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.radius_particle_system =
                resolve_optional_asset_name(tok, parse_ctx.particle_system_names);
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_grant_stealth_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<GrantStealthBehaviorModuleData>()
        .expect("GrantStealthBehavior data type mismatch")
        .clone();
    Box::new(GrantStealthBehavior::new(typed))
}
