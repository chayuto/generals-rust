//! `PKT-279`: `MinefieldBehaviorModuleData` and `MinefieldBehavior`
//! (`MinefieldBehavior.h`, `MinefieldBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-279_MinefieldBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt272_279_minefield.rs`.
//!
//! The module damages its own owner, and its `onDamage` and `onDie` run from inside
//! that damage. It lends itself to the arena around every such call (`lend_self`).
//! Not ported: firing the detonation weapon and creating the object creation list
//! only record the request (`fire_weapon_template`, `spawn_ocl`).

use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_sqrtf, real_to_int_ceil, real_to_int_floor, Coord3D, WWMATH_EPSILON};
use zh_common::types::{
    Bool, NameKeyType, ObjectId, Real, Relationship, UnsignedByte, UnsignedInt, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::behavior_module::{
    lend_self, resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{
    BehaviorModule, CollideFacet, DamageFacet, DieFacet, LandMineFacet, ModuleData, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon_fire::create_and_fire_temp_weapon;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};

const MIN_HEALTH: Real = 0.1;
const MAX_IMMUNITY: usize = 3;
const LOGICFRAMES_PER_SECOND: Real = 30.0;
/// `FOREVER` (`GameCommon.h`).
const FOREVER: UnsignedInt = 0x3fff_ffff;
/// `TheRelationshipNames`; the bit of a name is its `Relationship` value.
const RELATIONSHIP_NAMES: [&str; 3] = ["ENEMIES", "NEUTRAL", "ALLIES"];

/// INI template data matching `MinefieldBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct MinefieldBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub detonation_weapon: Option<String>,
    pub detonated_by: UnsignedInt,
    pub stops_regen_after_creator_dies: Bool,
    pub regenerates: Bool,
    pub workers_detonate: Bool,
    pub creator_death_check_rate: UnsignedInt,
    pub scoot_from_starting_point_time: UnsignedInt,
    pub num_virtual_mines: UnsignedInt,
    pub repeat_detonate_move_thresh: Real,
    pub health_percent_to_drain_per_second: Real,
    pub ocl: Option<String>,
}

impl MinefieldBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            detonation_weapon: None,
            // ENEMIES | NEUTRAL.
            detonated_by: 0b011,
            stops_regen_after_creator_dies: TRUE,
            regenerates: FALSE,
            workers_detonate: FALSE,
            creator_death_check_rate: LOGICFRAMES_PER_SECOND as UnsignedInt,
            scoot_from_starting_point_time: 0,
            num_virtual_mines: 1,
            repeat_detonate_move_thresh: 1.0,
            health_percent_to_drain_per_second: 0.0,
            ocl: None,
        }
    }
}

impl ModuleData for MinefieldBehaviorModuleData {
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
        FacetMask(0x017 | FacetMask::LAND_MINE.0)
    }
}

/// `ImmuneInfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImmuneInfo {
    pub id: ObjectId,
    pub collide_time: UnsignedInt,
}

const NOT_IMMUNE: ImmuneInfo = ImmuneInfo {
    id: ObjectId::INVALID,
    collide_time: 0,
};

/// Runtime instance of `MinefieldBehavior`.
#[derive(Debug, Clone)]
pub struct MinefieldBehavior {
    pub data: MinefieldBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
    pub next_death_check_frame: UnsignedInt,
    pub scoot_frames_left: UnsignedInt,
    pub scoot_vel: Coord3D,
    pub scoot_accel: Coord3D,
    pub virtual_mines_remaining: UnsignedInt,
    pub immunes: [ImmuneInfo; MAX_IMMUNITY],
    /// Who set the mine off and from where. Not saved.
    pub detonators: Vec<(ObjectId, Coord3D)>,
    pub ignore_damage: Bool,
    pub regenerates: Bool,
    pub draining: Bool,
}

fn sqr(x: Real) -> Real {
    x * x
}

fn dist_squared(a: &Coord3D, b: &Coord3D) -> Real {
    sqr(a.x - b.x) + sqr(a.y - b.y) + sqr(a.z - b.z)
}

fn set_rubble(sim: &mut dyn SimApi, ctx: ModuleContext, rubble: bool) {
    let owner = ctx.owner_slot;
    if rubble {
        sim.set_model_condition_state(owner, ModelConditionFlagType::Rubble);
        sim.set_status(owner, ObjectStatusType::Masked);
    } else {
        sim.clear_model_condition_state(owner, ModelConditionFlagType::Rubble);
        sim.clear_status(owner, ObjectStatusType::Masked);
    }
}

impl MinefieldBehavior {
    pub fn new(data: MinefieldBehaviorModuleData) -> Self {
        Self {
            schedule: UpdateScheduleTracker::new(),
            next_death_check_frame: 0,
            scoot_frames_left: 0,
            scoot_vel: Coord3D::default(),
            scoot_accel: Coord3D::default(),
            virtual_mines_remaining: data.num_virtual_mines,
            immunes: [NOT_IMMUNE; MAX_IMMUNITY],
            detonators: Vec::new(),
            ignore_damage: FALSE,
            regenerates: data.regenerates,
            draining: FALSE,
            data,
        }
    }

    /// `calcSleepTime`.
    fn calc_sleep_time(&self, now: UnsignedInt) -> UpdateSleepTime {
        if self.draining != FALSE
            || self.scoot_frames_left > 0
            || self.immunes.iter().any(|i| i.id != ObjectId::INVALID)
        {
            return UpdateSleepTime::NONE;
        }
        let mut sleep = FOREVER;
        if self.regenerates != FALSE && self.data.stops_regen_after_creator_dies != FALSE {
            sleep = sleep.min(self.next_death_check_frame.wrapping_sub(now));
        }
        UpdateSleepTime(sleep.max(1))
    }

    /// `Object::attemptDamage` on the owner, by the owner.
    fn damage_self(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, mut info: DamageInfo) {
        info.input.source_id = sim
            .object(ctx.owner_slot)
            .map_or(ObjectId::INVALID, |o| o.id);
        lend_self(self, sim, ctx, |s| {
            s.attempt_damage(ctx.owner_slot, &mut info)
        });
    }

    /// The damage that takes health away without waking this module's `onDamage`.
    fn remove_health(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, amount: Real) {
        self.ignore_damage = TRUE;
        let mut info = DamageInfo::default();
        info.input.damage_type = DamageType::Unresistable;
        info.input.death_type = DeathType::NoneType;
        info.input.amount = amount;
        self.damage_self(sim, ctx, info);
        self.ignore_damage = FALSE;
    }

    /// `detonateOnce`.
    fn detonate_once(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, position: Coord3D) {
        let owner = ctx.owner_slot;
        if let Some(weapon) = self.data.detonation_weapon.clone() {
            lend_self(self, sim, ctx, |s| {
                create_and_fire_temp_weapon(s, owner, &weapon, &position);
            });
        }
        if self.virtual_mines_remaining > 0 {
            self.virtual_mines_remaining -= 1;
        }
        if self.regenerates == FALSE && self.virtual_mines_remaining == 0 {
            sim.destroy_object(owner);
        } else {
            let percent =
                self.virtual_mines_remaining as Real / self.data.num_virtual_mines as Real;
            let health = sim.object_health(owner);
            let mut desired = percent * sim.object_max_health(owner);
            if desired < MIN_HEALTH {
                desired = MIN_HEALTH;
            }
            let amount = health - desired;
            if amount > 0.0 {
                self.remove_health(sim, ctx, amount);
            }
        }
        set_rubble(sim, ctx, self.virtual_mines_remaining == 0);
        if let Some(ocl) = &self.data.ocl {
            let pos = sim
                .object(owner)
                .map_or_else(Coord3D::default, |o| *o.thing.get_position());
            sim.spawn_ocl(ocl, owner, &pos, &pos);
        }
    }

    /// `onDamage`, which `onHealing` also runs.
    fn damaged(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.ignore_damage != FALSE {
            return;
        }
        let owner = ctx.owner_slot;
        let num = self.data.num_virtual_mines;
        loop {
            let max_health = sim.object_max_health(owner);
            let divisor = if max_health > WWMATH_EPSILON {
                max_health
            } else {
                WWMATH_EPSILON
            };
            let expected_f = (num as Real * sim.object_health(owner)) / divisor;
            let expected = if dmg.input.damage_type == DamageType::Healing {
                real_to_int_floor(expected_f)
            } else {
                real_to_int_ceil(expected_f)
            };
            // C++ compares the `Int` as unsigned.
            let expected = (expected as UnsignedInt).min(num);
            if self.virtual_mines_remaining < expected {
                self.virtual_mines_remaining = expected;
            } else if self.virtual_mines_remaining > expected {
                let own_id = sim.object(owner).map_or(ObjectId::INVALID, |o| o.id);
                if self.draining != FALSE
                    && dmg.input.source_id == own_id
                    && dmg.input.damage_type == DamageType::Unresistable
                {
                    self.virtual_mines_remaining -= 1;
                } else {
                    let pos = sim
                        .object(owner)
                        .map_or_else(Coord3D::default, |o| *o.thing.get_position());
                    self.detonate_once(sim, ctx, pos);
                }
            } else {
                break;
            }
        }
        if self.virtual_mines_remaining == 0 {
            let health = sim.object_health(owner);
            if self.regenerates != FALSE && health < MIN_HEALTH {
                sim.body_internal_change_health(owner, MIN_HEALTH - health);
            }
            set_rubble(sim, ctx, true);
        } else {
            set_rubble(sim, ctx, false);
        }
    }
}

impl LandMineFacet for MinefieldBehavior {
    /// `setScootParms`.
    fn set_scoot_parms(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        start: &Coord3D,
        end: &Coord3D,
    ) {
        let owner = ctx.owner_slot;
        let gravity = sim.rules().global_data.gravity;
        let mut time = self.data.scoot_from_starting_point_time;
        let mut end_on_ground = *end;
        end_on_ground.z = sim
            .terrain()
            .get_ground_height(end_on_ground.x, end_on_ground.y, None);
        if start.z > end_on_ground.z {
            let falling =
                real_to_int_ceil(gm_sqrtf(2.0 * (start.z - end_on_ground.z) / gravity.abs()))
                    as UnsignedInt;
            time = time.max(falling);
        }
        if time == 0 {
            sim.set_object_position(owner, &end_on_ground);
            self.scoot_frames_left = 0;
            return;
        }
        let dx = end_on_ground.x - start.x;
        let dy = end_on_ground.y - start.y;
        let dz = end_on_ground.z - start.z;
        let dist = gm_sqrtf(sqr(dx) + sqr(dy));
        if dist <= 0.1 && dz.abs() <= 0.1 {
            sim.set_object_position(owner, &end_on_ground);
            self.scoot_frames_left = 0;
            return;
        }
        let t = time as Real;
        let speed = dist / t;
        let accel = (2.0 * (dist - speed * t) / sqr(t)).abs();
        let dx_norm = if dist <= 0.1 { 0.0 } else { dx / dist };
        let dy_norm = if dist <= 0.1 { 0.0 } else { dy / dist };
        // C++ leaves `m_scootVel.z` as it was.
        self.scoot_vel.x = dx_norm * speed;
        self.scoot_vel.y = dy_norm * speed;
        self.scoot_accel.x = -dx_norm * accel;
        self.scoot_accel.y = -dy_norm * accel;
        self.scoot_accel.z = gravity;
        sim.set_object_position(owner, start);
        self.scoot_frames_left = time;
        let sleep = self.calc_sleep_time(sim.frame());
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }

    /// `disarm`.
    fn disarm(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.regenerates == FALSE {
            sim.destroy_object(ctx.owner_slot);
            return;
        }
        let amount = sim.object_health(ctx.owner_slot) - MIN_HEALTH;
        self.remove_health(sim, ctx, amount);
        self.virtual_mines_remaining = 0;
        set_rubble(sim, ctx, true);
    }
}

impl UpdateFacet for MinefieldBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let now = sim.frame();

        if self.scoot_frames_left > 0 {
            let mut pt = sim
                .object(owner)
                .map_or_else(Coord3D::default, |o| *o.thing.get_position());
            self.scoot_vel.x += self.scoot_accel.x;
            self.scoot_vel.y += self.scoot_accel.y;
            self.scoot_vel.z += self.scoot_accel.z;
            pt.x += self.scoot_vel.x;
            pt.y += self.scoot_vel.y;
            pt.z += self.scoot_vel.z;
            let mut tmp = pt;
            tmp.z = 99999.0;
            let layer = sim.get_highest_layer_for_destination(&tmp, FALSE);
            sim.set_object_layer(owner, layer);
            let mut ground = sim.get_layer_height(pt.x, pt.y, layer);
            if layer != PathfindLayerEnum::Ground {
                ground += 1.0;
            }
            if pt.z < ground || self.scoot_frames_left <= 1 {
                pt.z = ground;
            }
            sim.set_object_position(owner, &pt);
            self.scoot_frames_left -= 1;
        }

        for i in 0..MAX_IMMUNITY {
            let immune = self.immunes[i];
            if immune.id == ObjectId::INVALID {
                continue;
            }
            if sim.find_object(immune.id).is_none() || now > immune.collide_time.wrapping_add(2) {
                self.immunes[i] = NOT_IMMUNE;
            }
        }

        if now >= self.next_death_check_frame
            && self.regenerates != FALSE
            && self.data.stops_regen_after_creator_dies != FALSE
        {
            self.next_death_check_frame = now.wrapping_add(self.data.creator_death_check_rate);
            let producer_id = sim
                .object(owner)
                .map_or(ObjectId::INVALID, |o| o.producer_id);
            if producer_id != ObjectId::INVALID {
                let creator_gone = sim
                    .find_object(producer_id)
                    .and_then(|p| sim.object(p))
                    .is_none_or(|p| p.is_effectively_dead() != FALSE);
                if creator_gone {
                    self.regenerates = FALSE;
                    self.draining = TRUE;
                    sim.stop_auto_healing(owner);
                }
            }
        }

        if self.draining != FALSE {
            let mut info = DamageInfo::default();
            info.input.amount = (sim.object_max_health(owner)
                * self.data.health_percent_to_drain_per_second)
                / LOGICFRAMES_PER_SECOND;
            info.input.damage_type = DamageType::Unresistable;
            info.input.death_type = DeathType::Normal;
            self.damage_self(sim, ctx, info);
        }
        self.calc_sleep_time(now)
    }
}

impl CollideFacet for MinefieldBehavior {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<crate::arena::ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        let owner = ctx.owner_slot;
        let Some(other) = other else {
            return;
        };
        let Some(them) = sim.object(other) else {
            return;
        };
        if them.is_effectively_dead() != FALSE || self.virtual_mines_remaining == 0 {
            return;
        }
        let other_id = them.id;
        let other_pos = *them.thing.get_position();
        let tf = &sim.rules().thing_factory;
        let is_worker = them.thing.is_kind_of(tf, KindOfType::Infantry) != FALSE
            && them.thing.is_kind_of(tf, KindOfType::Dozer) != FALSE;
        let now = sim.frame();

        if let Some(immune) = self.immunes.iter_mut().find(|i| i.id == other_id) {
            immune.collide_time = now;
            return;
        }
        if self.data.workers_detonate == FALSE && is_worker {
            return;
        }
        let required = match sim.player_relationship(owner, other) {
            Relationship::Enemies => 0b001,
            Relationship::Neutral => 0b010,
            Relationship::Allies => 0b100,
        };
        if self.data.detonated_by & required == 0 || self.scoot_frames_left > 0 {
            return;
        }

        // Mine clearers are spared while they clear.
        if sim.ai_is_clearing_mines(other) != FALSE
            && sim
                .object_ai_goal_object(other)
                .is_some_and(|goal| sim.object(goal).is_some())
        {
            if let Some(immune) = self
                .immunes
                .iter_mut()
                .find(|i| i.id == ObjectId::INVALID || i.id == other_id)
            {
                immune.id = other_id;
                immune.collide_time = now;
                let sleep = self.calc_sleep_time(now);
                sim.set_wake_frame(ctx.mod_slot, sleep);
            }
            return;
        }

        let thresh = sqr(self.data.repeat_detonate_move_thresh);
        if let Some((_, at)) = self.detonators.iter_mut().find(|(id, _)| *id == other_id) {
            if dist_squared(&other_pos, at) <= thresh {
                return;
            }
            *at = other_pos;
        } else {
            self.detonators.push((other_id, other_pos));
        }

        let mut det_pt = other_pos;
        if let Some(me) = sim.object(owner) {
            me.geometry
                .clip_point_to_footprint(me.thing.get_position(), &mut det_pt);
        }
        self.detonate_once(sim, ctx, det_pt);
    }
}

impl DamageFacet for MinefieldBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        self.damaged(sim, ctx, dmg);
    }

    fn on_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        self.damaged(sim, ctx, dmg);
    }

    fn on_body_damage_state_change(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dmg: &DamageInfo,
        _old_state: BodyDamageType,
        _new_state: BodyDamageType,
    ) {
    }
}

impl DieFacet for MinefieldBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        sim.destroy_object(ctx.owner_slot);
    }
}

impl BehaviorModule for MinefieldBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        sim.set_status(ctx.owner_slot, ObjectStatusType::NoAttackFromAi);
    }

    /// `UpdateModule::crc` adds nothing.
    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.virtual_mines_remaining)?;
        xfer.xfer_unsigned_int(&mut self.next_death_check_frame)?;
        xfer.xfer_unsigned_int(&mut self.scoot_frames_left)?;
        xfer.xfer_coord3d(&mut self.scoot_vel)?;
        xfer.xfer_coord3d(&mut self.scoot_accel)?;
        xfer.xfer_bool(&mut self.ignore_damage)?;
        xfer.xfer_bool(&mut self.regenerates)?;
        xfer.xfer_bool(&mut self.draining)?;
        let mut count = MAX_IMMUNITY as UnsignedByte;
        xfer.xfer_unsigned_byte(&mut count)?;
        if usize::from(count) != MAX_IMMUNITY {
            // `SC_INVALID_DATA`.
            return Err(XferError::InvalidData);
        }
        for immune in &mut self.immunes {
            xfer.xfer_object_id(&mut immune.id)?;
            xfer.xfer_unsigned_int(&mut immune.collide_time)?;
        }
        if xfer.mode() == XferMode::Load {
            self.detonators.clear();
        }
        Ok(())
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

    fn as_collide(&self) -> Option<&dyn CollideFacet> {
        Some(self)
    }

    fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
        Some(self)
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_land_mine(&self) -> Option<&dyn LandMineFacet> {
        Some(self)
    }

    fn as_land_mine_mut(&mut self) -> Option<&mut dyn LandMineFacet> {
        Some(self)
    }
}

pub fn parse_minefield_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = MinefieldBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("DetonationWeapon") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.detonation_weapon = (!tok.eq_ignore_ascii_case("None")).then(|| tok.to_owned());
            Ok(())
        } else if field.eq_ignore_ascii_case("DetonatedBy") {
            r.parse_bit_string32(&mut data.detonated_by, &RELATIONSHIP_NAMES)
        } else if field.eq_ignore_ascii_case("StopsRegenAfterCreatorDies") {
            r.parse_bool(&mut data.stops_regen_after_creator_dies)
        } else if field.eq_ignore_ascii_case("Regenerates") {
            r.parse_bool(&mut data.regenerates)
        } else if field.eq_ignore_ascii_case("WorkersDetonate") {
            r.parse_bool(&mut data.workers_detonate)
        } else if field.eq_ignore_ascii_case("CreatorDeathCheckRate") {
            r.parse_duration_unsigned_int(&mut data.creator_death_check_rate)
        } else if field.eq_ignore_ascii_case("ScootFromStartingPointTime") {
            r.parse_duration_unsigned_int(&mut data.scoot_from_starting_point_time)
        } else if field.eq_ignore_ascii_case("NumVirtualMines") {
            r.parse_unsigned_int(&mut data.num_virtual_mines)
        } else if field.eq_ignore_ascii_case("RepeatDetonateMoveThresh") {
            r.parse_real(&mut data.repeat_detonate_move_thresh)
        } else if field.eq_ignore_ascii_case("DegenPercentPerSecondAfterCreatorDies") {
            r.parse_percent_to_real(&mut data.health_percent_to_drain_per_second)
        } else if field.eq_ignore_ascii_case("CreationList") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.ocl = resolve_optional_asset_name(tok, parse_ctx.ocl_names);
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_minefield_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<MinefieldBehaviorModuleData>()
        .expect("MinefieldBehavior data type mismatch")
        .clone();
    Box::new(MinefieldBehavior::new(typed))
}
