//! `PKT-350`: `PointDefenseLaserUpdateModuleData` and `PointDefenseLaserUpdate`
//! (`PointDefenseLaserUpdate.h`, `PointDefenseLaserUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-350_PointDefenseLaserUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt341_350_309_support.rs`.
//!
//! Without a weapon template the C++ dereferences null; this port finds no target.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::gamelogic::global_data::WeaponBonus;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon::{WeaponTemplate, WEAPON_ANTI_GROUND};
use crate::object::weapon_fire;
use crate::object::weapon_set::WeaponSlotType;
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_sqrtf, Coord3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `PointDefenseLaserUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct PointDefenseLaserUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub weapon_template_name: String,
    pub primary_target_kind_of: KindOfMaskType,
    pub secondary_target_kind_of: KindOfMaskType,
    pub scan_frames: UnsignedInt,
    pub scan_range: Real,
    pub velocity_factor: Real,
}

impl PointDefenseLaserUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            weapon_template_name: String::new(),
            primary_target_kind_of: KindOfMaskType::new(),
            secondary_target_kind_of: KindOfMaskType::new(),
            scan_frames: 0,
            scan_range: 0.0,
            velocity_factor: 0.0,
        }
    }
}

impl ModuleData for PointDefenseLaserUpdateModuleData {
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

/// Runtime instance of `PointDefenseLaserUpdate`.
pub struct PointDefenseLaserUpdate {
    pub data: PointDefenseLaserUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub best_target_id: ObjectId,
    pub in_range: Bool,
    pub next_scan_frames: Int,
    pub next_shot_available_in_frames: Int,
}

fn position(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

/// `sqrt(getDistanceSquared(a, b, FROM_CENTER_2D))`.
fn distance_2d(sim: &dyn SimView, a: ObjectSlot, b: ObjectSlot) -> Real {
    let (a, b) = (position(sim, a), position(sim, b));
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    gm_sqrtf(dx * dx + dy * dy)
}

impl PointDefenseLaserUpdate {
    pub fn new(data: PointDefenseLaserUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            best_target_id: ObjectId::INVALID,
            in_range: FALSE,
            next_scan_frames: 0,
            next_shot_available_in_frames: 0,
        }
    }

    fn weapon_template(&self, sim: &dyn SimView) -> Option<WeaponTemplate> {
        sim.rules()
            .weapon_store
            .find_weapon_template(&self.data.weapon_template_name)
            .cloned()
    }

    /// `scanClosestTarget`.
    fn scan_closest_target(&mut self, sim: &dyn SimView, owner: ObjectSlot) -> Option<ObjectSlot> {
        let weapon = self.weapon_template(sim)?;
        let fire_range = weapon.get_attack_range(&WeaponBonus::default());
        // [primary, secondary]; each holds (object, distance).
        let mut in_range: [Option<(ObjectSlot, Real)>; 2] = [None; 2];
        let mut out_of_range: [Option<(ObjectSlot, Real)>; 2] = [None; 2];
        let centre = position(sim, owner);
        for other in sim.find_objects_in_range(&centre, self.data.scan_range) {
            let Some(obj) = sim.object(other) else {
                continue;
            };
            let factory = &sim.rules().thing_factory;
            let index = if obj
                .thing
                .is_any_kind_of(factory, &self.data.primary_target_kind_of)
                != FALSE
            {
                0
            } else if obj
                .thing
                .is_any_kind_of(factory, &self.data.secondary_target_kind_of)
                != FALSE
            {
                1
            } else {
                continue;
            };
            let status = |s: ObjectStatusType| obj.status.test(s.as_i32()) != FALSE;
            if !status(ObjectStatusType::AirborneTarget)
                && weapon.anti_mask & WEAPON_ANTI_GROUND == 0
            {
                continue;
            }
            if sim.player_relationship(owner, other) != Relationship::Enemies {
                continue;
            }
            if status(ObjectStatusType::Stealthed)
                && !status(ObjectStatusType::Detected)
                && !status(ObjectStatusType::Disguised)
            {
                continue;
            }
            // The velocity prediction computes a position and then measures the target
            // itself again, so `PredictTargetVelocityFactor` has no effect.
            let dist = distance_2d(sim, owner, other);
            if dist <= fire_range {
                if in_range[index].is_none_or(|(_, best)| dist < best) {
                    in_range[index] = Some((other, dist));
                }
            } else if in_range[index].is_none()
                && out_of_range[index].is_none_or(|(_, best)| dist < best)
            {
                out_of_range[index] = Some((other, dist));
            }
        }
        let (found, in_fire_range) = match (in_range, out_of_range) {
            ([Some(a), _], _) | ([None, Some(a)], _) => (Some(a.0), TRUE),
            (_, [Some(a), _]) | (_, [None, Some(a)]) => (Some(a.0), FALSE),
            _ => (None, FALSE),
        };
        self.best_target_id = found
            .and_then(|slot| sim.object(slot))
            .map_or(ObjectId::INVALID, |o| o.id);
        self.in_range = in_fire_range;
        found
    }

    /// Picks a new frame to scan after the target was lost: `TRUE` when it scanned at once.
    fn rescan(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) -> bool {
        self.next_scan_frames = sim.rng_logic_int(0, 3);
        self.best_target_id = ObjectId::INVALID;
        if self.next_scan_frames != 0 {
            return false;
        }
        self.scan_closest_target(&*sim, owner);
        self.next_scan_frames = self.data.scan_frames as Int;
        true
    }

    /// `fireWhenReady`.
    fn fire_when_ready(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let mut target = sim.find_object(self.best_target_id);
        let template = self.weapon_template(&*sim);
        if let (Some(victim), Some(weapon)) = (target, template.as_ref()) {
            let fire_range = weapon.get_attack_range(&WeaponBonus::default());
            if distance_2d(&*sim, owner, victim) < fire_range {
                self.in_range = TRUE;
            } else if self.in_range != FALSE {
                // A delayed rescan leaves `target` and `m_inRange`, so it can still fire.
                if self.rescan(sim, owner) {
                    target = None;
                }
            }
        }
        if self.next_shot_available_in_frames > 0 {
            self.next_shot_available_in_frames -= 1;
            return;
        }
        let (Some(template), Some(victim)) = (template, target) else {
            return;
        };
        if self.in_range == FALSE {
            return;
        }
        let dead = |sim: &dyn SimApi| {
            sim.object(victim)
                .is_none_or(|o| o.is_effectively_dead() != FALSE)
        };
        if !dead(sim) {
            let mut weapon = sim.rules().weapon_store.allocate_new_weapon(
                &template,
                WeaponSlotType::Tertiary,
                sim.frame(),
            );
            weapon_fire::load_ammo_now(&mut weapon, sim, owner);
            weapon_fire::fire_weapon_at_object(&mut weapon, sim, owner, victim);
            self.next_shot_available_in_frames =
                weapon_fire::delay_between_shots(&weapon, sim, &WeaponBonus::default());
        }
        if dead(sim) {
            self.rescan(sim, owner);
        }
    }
}

impl UpdateFacet for PointDefenseLaserUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        if sim
            .object(owner)
            .is_none_or(|o| o.is_effectively_dead() != FALSE)
        {
            return UpdateSleepTime::FOREVER;
        }
        if self.next_scan_frames > 0 {
            self.next_scan_frames -= 1;
            self.fire_when_ready(sim, owner);
            return UpdateSleepTime::NONE;
        }
        self.next_scan_frames = self.data.scan_frames as Int;
        if self.scan_closest_target(&*sim, owner).is_some() {
            self.fire_when_ready(sim, owner);
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for PointDefenseLaserUpdate {
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
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_object_id(&mut self.best_target_id)?;
        xfer.xfer_bool(&mut self.in_range)?;
        xfer.xfer_int(&mut self.next_scan_frames)?;
        xfer.xfer_int(&mut self.next_shot_available_in_frames)
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

pub fn parse_point_defense_laser_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = PointDefenseLaserUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("WeaponTemplate") {
            r.parse_ascii_string(&mut data.weapon_template_name)
        } else if is("PrimaryTargetTypes") {
            data.primary_target_kind_of.parse_from_ini(r, &KINDOF_NAMES)
        } else if is("SecondaryTargetTypes") {
            data.secondary_target_kind_of
                .parse_from_ini(r, &KINDOF_NAMES)
        } else if is("ScanRate") {
            r.parse_duration_unsigned_int(&mut data.scan_frames)
        } else if is("ScanRange") {
            r.parse_real(&mut data.scan_range)
        } else if is("PredictTargetVelocityFactor") {
            r.parse_real(&mut data.velocity_factor)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_point_defense_laser_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<PointDefenseLaserUpdateModuleData>()
        .expect("PointDefenseLaserUpdate data type mismatch")
        .clone();
    Box::new(PointDefenseLaserUpdate::new(typed))
}
