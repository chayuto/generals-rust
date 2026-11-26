//! `PKT-377`: `StructureToppleUpdateModuleData` and `StructureToppleUpdate`
//! (`StructureToppleUpdate.h`, `StructureToppleUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-377_StructureToppleUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt383_377_305_topple.rs`.
//!
//! The burst delay comes from the *client* PRNG stream, as in the reference, so it
//! decides when logic-PRNG draws and OCLs happen. The reference never fills
//! `m_fxbones` (no INI field), so its particle-system loop is dead and not ported.
//! `BodyModuleInterface::updateBodyParticleSystems` is client-only and not called.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::topple_update::parse_fx_name;
use crate::object::weapon_fire::create_and_fire_temp_weapon;
use crate::rules::combat_types::{parse_damage_type_flags, DamageTypeFlags};
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_cosf, gm_sinf, Coord2D, Coord3D, PI};
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

pub const STPHASE_INITIAL: usize = 0;
pub const STPHASE_DELAY: usize = 1;
pub const STPHASE_FINAL: usize = 2;
pub const ST_PHASE_COUNT: usize = 3;

const PHASE_NAMES: [&str; ST_PHASE_COUNT] = ["INITIAL", "DELAY", "FINAL"];

pub const TOPPLESTATE_STANDING: Int = 0;
pub const TOPPLESTATE_WAITINGFORTOPPLESTART: Int = 1;
pub const TOPPLESTATE_TOPPLING: Int = 2;
pub const TOPPLESTATE_WAITINGFORDONE: Int = 3;
pub const TOPPLESTATE_DONE: Int = 4;

const TOPPLE_ACCELERATION_FACTOR: Real = 0.02;
const THETA_CEILING: Real = PI / 6.0;
const WEAPON_SPACING_PERPENDICULAR: Real = 25.0;
const WEAPON_SPACING_PARALLEL: Real = 25.0;
/// `PLAYERMASK_ALL`.
const PLAYERMASK_ALL: u16 = 0xffff;

/// INI template data matching `StructureToppleUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct StructureToppleUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub min_topple_delay: Int,
    pub max_topple_delay: Int,
    pub min_topple_burst_delay: Int,
    pub max_topple_burst_delay: Int,
    pub structural_integrity: Real,
    pub structural_decay: Real,
    pub damage_fx_types: DamageTypeFlags,
    pub topple_start_fx: Option<String>,
    pub topple_delay_fx: Option<String>,
    pub topple_fx: Option<String>,
    pub topple_done_fx: Option<String>,
    pub crushing_fx: Option<String>,
    pub crushing_weapon_name: String,
    /// Per phase. A `None` entry is a null list: it is drawn but creates nothing.
    pub ocls: [Vec<Option<String>>; ST_PHASE_COUNT],
    /// Angle in radians and the list played when the building passes it.
    pub angle_fx: Vec<(Real, Option<String>)>,
}

impl StructureToppleUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            die_mux: DieMuxData::new(),
            min_topple_delay: 0,
            max_topple_delay: 0,
            min_topple_burst_delay: 0,
            max_topple_burst_delay: 0,
            structural_integrity: 0.1,
            structural_decay: 0.0,
            damage_fx_types: DamageTypeFlags::new_set_all(),
            topple_start_fx: None,
            topple_delay_fx: None,
            topple_fx: None,
            topple_done_fx: None,
            crushing_fx: None,
            crushing_weapon_name: String::new(),
            ocls: [Vec::new(), Vec::new(), Vec::new()],
            angle_fx: Vec::new(),
        }
    }
}

impl ModuleData for StructureToppleUpdateModuleData {
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

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn orientation_of(sim: &dyn SimView, slot: ObjectSlot) -> Real {
    sim.object(slot).map_or(0.0, |o| o.thing.get_orientation())
}

/// Runtime instance of `StructureToppleUpdate`.
pub struct StructureToppleUpdate {
    pub data: StructureToppleUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub topple_frame: UnsignedInt,
    pub topple_direction: Coord2D,
    pub topple_state: Int,
    pub topple_velocity: Real,
    pub accumulated_angle: Real,
    pub structural_integrity: Real,
    pub last_crushed_location: Real,
    pub next_burst_frame: Int,
    pub delay_burst_location: Coord3D,
    /// Taken from the geometry at construction. Not saved.
    pub building_height: Real,
}

impl StructureToppleUpdate {
    pub fn new(data: StructureToppleUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            topple_frame: 0,
            topple_direction: Coord2D { x: 0.0, y: 0.0 },
            topple_state: TOPPLESTATE_STANDING,
            topple_velocity: 0.0,
            accumulated_angle: 0.001,
            structural_integrity: 0.0,
            last_crushed_location: 0.0,
            next_burst_frame: -1,
            delay_burst_location: Coord3D::default(),
            building_height: 0.0,
        }
    }

    /// The `DamageFXTypes` test every effect of this module is behind.
    fn fx_allowed(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        sim.object_last_damage_type(owner)
            .is_none_or(|t| self.data.damage_fx_types.test(t.as_i32()) != FALSE)
    }

    fn burst_delay(&self, sim: &mut dyn SimApi, now: UnsignedInt) -> Int {
        let delay = sim.rng_client_int(
            self.data.min_topple_burst_delay,
            self.data.max_topple_burst_delay,
        );
        now.wrapping_add(delay as UnsignedInt) as Int
    }

    fn begin_structure_topple(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        let delay = sim.rng_logic_int(self.data.min_topple_delay, self.data.max_topple_delay);
        self.topple_frame = now.wrapping_add(delay as UnsignedInt);
        let attacker = sim.find_object(dmg.input.source_id);
        let building_pos = position_of(sim, owner);
        let topple_angle = match attacker {
            None => sim.rng_logic_real(0.0, 2.0 * PI),
            Some(attacker) => {
                let attacker_pos = position_of(sim, attacker);
                self.topple_direction.x = building_pos.x - attacker_pos.x;
                self.topple_direction.y = building_pos.y - attacker_pos.y;
                let angle = self.topple_direction.to_angle();
                angle + sim.rng_logic_real(-PI / 8.0, PI / 8.0)
            }
        };
        self.topple_direction.x = gm_cosf(topple_angle);
        self.topple_direction.y = gm_sinf(topple_angle);
        // `ScriptEngine::adjustToppleDirection(Object*, Coord2D*)` goes through a
        // `Coord3D`.
        let mut dir = Coord3D::new(self.topple_direction.x, self.topple_direction.y, 0.0);
        sim.adjust_topple_direction(owner, &mut dir);
        self.topple_direction.x = dir.x;
        self.topple_direction.y = dir.y;
        let average_radius = sim.object(owner).map_or(0.0, |o| {
            (o.geometry.get_major_radius() + o.geometry.get_minor_radius()) / 2.0
        });
        let explosion_radius = (f64::from(average_radius) * 0.90) as Real;
        self.delay_burst_location.x = building_pos.x + explosion_radius * gm_cosf(topple_angle);
        self.delay_burst_location.y = building_pos.y + explosion_radius * gm_sinf(topple_angle);
        self.delay_burst_location.z = sim.terrain().get_ground_height(
            self.delay_burst_location.x,
            self.delay_burst_location.y,
            None,
        );
        self.do_topple_start_fx(sim, owner);
        self.next_burst_frame = self.burst_delay(sim, now);
        self.topple_state = TOPPLESTATE_WAITINGFORTOPPLESTART;
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn do_topple_start_fx(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let pos = position_of(sim, owner);
        if self.fx_allowed(sim, owner) {
            if let Some(fx) = &self.data.topple_start_fx {
                sim.trigger_fx_list(fx, &pos);
            }
        }
        self.do_phase_stuff(sim, owner, STPHASE_INITIAL, &pos);
    }

    fn do_topple_delay_burst_fx(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if self.fx_allowed(sim, owner) {
            if let Some(fx) = &self.data.topple_delay_fx {
                sim.trigger_fx_list(fx, &self.delay_burst_location);
            }
        }
        self.do_phase_stuff(sim, owner, STPHASE_DELAY, &self.delay_burst_location);
    }

    fn do_phase_stuff(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        phase: usize,
        target: &Coord3D,
    ) {
        let list = &self.data.ocls[phase];
        if list.is_empty() {
            return;
        }
        // `m_oclCount` is always 1, so `buildNonDupRandomIndexList` is one draw.
        let index = sim.rng_logic_int(0, list.len() as Int - 1);
        if let Some(Some(ocl)) = list.get(index as usize) {
            sim.spawn_ocl(ocl, owner, target, target);
        }
    }

    fn do_angle_fx(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        cur_angle: Real,
        new_angle: Real,
    ) {
        for (angle, fx) in &self.data.angle_fx {
            if *angle > cur_angle && *angle <= new_angle && self.fx_allowed(sim, owner) {
                if let Some(fx) = fx {
                    let pos = position_of(sim, owner);
                    sim.trigger_fx_list(fx, &pos);
                }
            }
        }
    }

    fn apply_crushing_damage(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, theta: Real) {
        if theta > THETA_CEILING {
            return;
        }
        let orientation = orientation_of(sim, owner);
        let topple_angle = self.topple_direction.to_angle();
        let angle = orientation - topple_angle;
        let (major, minor) = sim.object(owner).map_or((0.0, 0.0), |o| {
            (o.geometry.get_major_radius(), o.geometry.get_minor_radius())
        });
        let minor_component = minor * gm_cosf(angle);
        let major_component = major * gm_sinf(angle);
        let facing_width = Coord3D::new(major_component, minor_component, 0.0).length() / 2.0;
        if sim
            .rules()
            .weapon_store
            .find_weapon_template(&self.data.crushing_weapon_name)
            .is_none()
        {
            return;
        }
        let max_distance =
            (f64::from(self.building_height) * (1.0 - f64::from(gm_sinf(theta)))) as Real;
        let mut j = self.last_crushed_location;
        while j < max_distance {
            let jcos = j * gm_cosf(topple_angle);
            let jsin = j * gm_sinf(topple_angle);
            self.do_damage_line(sim, owner, jcos, jsin, facing_width, topple_angle);
            j += WEAPON_SPACING_PERPENDICULAR;
        }
        let jcos = max_distance * gm_cosf(topple_angle);
        let jsin = max_distance * gm_sinf(topple_angle);
        self.do_damage_line(sim, owner, jcos, jsin, facing_width, topple_angle);
        self.last_crushed_location = j;
    }

    fn do_damage_line(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        jcos: Real,
        jsin: Real,
        facing_width: Real,
        topple_angle: Real,
    ) {
        let mut i = -facing_width;
        while i < facing_width {
            self.crush_at(sim, owner, jcos, jsin, i, topple_angle);
            i += WEAPON_SPACING_PARALLEL;
        }
        self.crush_at(sim, owner, jcos, jsin, facing_width, topple_angle);
        let pos = position_of(sim, owner);
        let x = pos.x + jcos;
        let y = pos.y + jsin;
        let target = Coord3D::new(x, y, sim.terrain().get_ground_height(x, y, None));
        self.do_phase_stuff(sim, owner, STPHASE_FINAL, &target);
    }

    /// One crushing weapon shot, `offset` along the damage line.
    fn crush_at(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        jcos: Real,
        jsin: Real,
        offset: Real,
        topple_angle: Real,
    ) {
        let pos = position_of(sim, owner);
        // The reference swaps sin and cos here: the line is not perpendicular.
        let x = pos.x + jcos + offset * gm_sinf(topple_angle);
        let y = pos.y + jsin + offset * gm_cosf(topple_angle);
        let target = Coord3D::new(x, y, sim.terrain().get_ground_height(x, y, None));
        create_and_fire_temp_weapon(sim, owner, &self.data.crushing_weapon_name, &target);
        if self.fx_allowed(sim, owner) {
            if let Some(fx) = &self.data.crushing_fx {
                sim.trigger_fx_list(fx, &target);
            }
        }
    }

    fn do_topple_done_stuff(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        sim.stop_all_bone_fx(owner);
        let orig_angle = orientation_of(sim, owner);
        sim.set_object_orientation(owner, orig_angle);
        let topple_angle = self.topple_direction.to_angle();
        let Some(mut xfrm) = sim.object(owner).map(|o| *o.thing.get_transform_matrix()) else {
            return;
        };
        xfrm.in_place_pre_rotate_z(topple_angle - orig_angle);
        sim.set_object_transform_matrix(owner, &xfrm);
    }
}

impl DieFacet for StructureToppleUpdate {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let owner = ctx.owner_slot;
        if self.data.die_mux.is_die_applicable(sim, owner, dmg) == FALSE {
            return;
        }
        sim.ai_mark_as_dead(owner);
        sim.deselect_object(owner, PLAYERMASK_ALL, TRUE);
        self.begin_structure_topple(sim, ctx, dmg);
    }
}

impl UpdateFacet for StructureToppleUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.topple_state == TOPPLESTATE_STANDING {
            return UpdateSleepTime::FOREVER;
        }
        let owner = ctx.owner_slot;
        if self.topple_state == TOPPLESTATE_WAITINGFORTOPPLESTART {
            let now = sim.frame();
            if now >= self.next_burst_frame as UnsignedInt {
                self.do_topple_delay_burst_fx(sim, owner);
                self.next_burst_frame = self.burst_delay(sim, now);
            }
            if now >= self.topple_frame {
                self.topple_state = TOPPLESTATE_TOPPLING;
                self.structural_integrity = self.data.structural_integrity;
            }
        }
        if self.topple_state == TOPPLESTATE_TOPPLING {
            let now = sim.frame();
            // The reference mixes in double precision here.
            let acceleration = (f64::from(TOPPLE_ACCELERATION_FACTOR)
                * (f64::from(gm_sinf(self.accumulated_angle))
                    * (1.0 - f64::from(self.structural_integrity))))
                as Real;
            self.topple_velocity += acceleration;
            if self.structural_integrity > 0.0 {
                self.structural_integrity *= self.data.structural_decay;
                if self.structural_integrity < 0.0 {
                    self.structural_integrity = 0.0;
                }
            }
            self.do_angle_fx(
                sim,
                owner,
                self.accumulated_angle,
                self.accumulated_angle + self.topple_velocity,
            );
            self.accumulated_angle += self.topple_velocity;
            self.apply_crushing_damage(sim, owner, PI / 2.0 - self.accumulated_angle);
            if self.accumulated_angle >= PI / 2.0 {
                self.topple_velocity -= self.accumulated_angle - PI / 2.0;
                self.accumulated_angle = PI / 2.0;
                self.topple_state = TOPPLESTATE_WAITINGFORDONE;
                self.apply_crushing_damage(sim, owner, 0.0);
                let pos = position_of(sim, owner);
                self.do_phase_stuff(sim, owner, STPHASE_FINAL, &pos);
                if self.fx_allowed(sim, owner) {
                    if let Some(fx) = &self.data.topple_done_fx {
                        let pos = position_of(sim, owner);
                        sim.trigger_fx_list(fx, &pos);
                    }
                }
                self.topple_frame = sim.frame();
            }
            if now >= self.next_burst_frame as UnsignedInt {
                self.do_topple_delay_burst_fx(sim, owner);
                self.next_burst_frame = self.burst_delay(sim, now);
            }
            if let Some(mut xfrm) = sim.object(owner).map(|o| *o.thing.get_transform_matrix()) {
                xfrm.in_place_pre_rotate_x(-self.topple_velocity * self.topple_direction.y);
                xfrm.in_place_pre_rotate_y(self.topple_velocity * self.topple_direction.x);
                sim.set_object_transform_matrix(owner, &xfrm);
            }
        }
        if self.topple_state == TOPPLESTATE_WAITINGFORDONE && self.topple_frame <= sim.frame() {
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Rubble);
            sim.set_model_condition_state(owner, ModelConditionFlagType::PostCollapse);
            self.do_topple_done_stuff(sim, owner);
            self.topple_state = TOPPLESTATE_DONE;
            return UpdateSleepTime::FOREVER;
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for StructureToppleUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
        self.building_height = sim
            .object(ctx.owner_slot)
            .map_or(0.0, |o| o.geometry.get_max_height_above_position());
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.topple_frame)?;
        xfer.xfer_coord2d(&mut self.topple_direction)?;
        let mut state = self.topple_state.to_le_bytes();
        xfer.xfer_user(&mut state)?;
        self.topple_state = Int::from_le_bytes(state);
        xfer.xfer_real(&mut self.topple_velocity)?;
        xfer.xfer_real(&mut self.accumulated_angle)?;
        xfer.xfer_real(&mut self.structural_integrity)?;
        xfer.xfer_real(&mut self.last_crushed_location)?;
        xfer.xfer_int(&mut self.next_burst_frame)?;
        xfer.xfer_coord3d(&mut self.delay_burst_location)
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

pub fn parse_structure_topple_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = StructureToppleUpdateModuleData::new(tag_key, module_name_key);
    let duration = |r: &mut IniReader<'_>, store: &mut Int| -> Result<(), IniError> {
        let mut frames: UnsignedInt = 0;
        r.parse_duration_unsigned_int(&mut frames)?;
        *store = frames as Int;
        Ok(())
    };
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("MinToppleDelay") {
            duration(r, &mut data.min_topple_delay)
        } else if is("MaxToppleDelay") {
            duration(r, &mut data.max_topple_delay)
        } else if is("MinToppleBurstDelay") {
            duration(r, &mut data.min_topple_burst_delay)
        } else if is("MaxToppleBurstDelay") {
            duration(r, &mut data.max_topple_burst_delay)
        } else if is("StructuralIntegrity") {
            r.parse_real(&mut data.structural_integrity)
        } else if is("StructuralDecay") {
            r.parse_real(&mut data.structural_decay)
        } else if is("DamageFXTypes") {
            data.damage_fx_types = parse_damage_type_flags(r)?;
            Ok(())
        } else if is("TopplingFX") {
            data.topple_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if is("ToppleDelayFX") {
            data.topple_delay_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if is("ToppleStartFX") {
            data.topple_start_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if is("ToppleDoneFX") {
            data.topple_done_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if is("CrushingFX") {
            data.crushing_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if is("CrushingWeaponName") {
            r.parse_ascii_string(&mut data.crushing_weapon_name)
        } else if is("OCL") {
            let phase = IniReader::scan_index_list(r.next_token(SEPS_DEFAULT)?, &PHASE_NAMES)?;
            let mut token = Some(r.next_token(SEPS_DEFAULT)?);
            while let Some(name) = token {
                data.ocls[phase as usize]
                    .push(resolve_optional_asset_name(name, parse_ctx.ocl_names));
                token = r.next_token_or_null(SEPS_DEFAULT);
            }
            Ok(())
        } else if is("AngleFX") {
            let mut angle: Real = 0.0;
            r.parse_real(&mut angle)?;
            let angle = angle * PI / 180.0;
            let fx = parse_fx_name(r, parse_ctx)?;
            data.angle_fx.push((angle, fx));
            Ok(())
        } else if let Some(res) = data.die_mux.parse_field(field, r) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_structure_topple_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<StructureToppleUpdateModuleData>()
        .expect("StructureToppleUpdate data type mismatch")
        .clone();
    Box::new(StructureToppleUpdate::new(typed))
}

/// `Bool` is part of the module's public surface through `DieMuxData`.
const _: Bool = FALSE;
