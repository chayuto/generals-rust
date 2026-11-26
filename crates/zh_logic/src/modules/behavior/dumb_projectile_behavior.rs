//! `PKT-267`: `DumbProjectileBehavior` (`DumbProjectileBehavior.h`, `.cpp`): an unguided
//! projectile flying a Bezier arc.
//!
//! Packet: `packets/phase5/PKT-267_DumbProjectileBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt334_267_382_346_misc.rs`.
//!
//! The weapon side of it (`positionProjectileForLaunch`, `shouldProjectileCollideWith`,
//! `handleProjectileDetonation`) lives in [`crate::object::weapon_fire`]. Hiding the
//! drawable after detonation is client only and not modelled.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{BehaviorModule, ModuleData, ProjectileFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon_fire;
use crate::object::weapon_set::WeaponSlotType;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, LOGICFRAMES_PER_SECOND, SEPS_DEFAULT};
use zh_common::math::bezier_segment::BezierSegment;
use zh_common::math::{gm_ceilf, gm_sqrtf, Coord3D, Matrix3D, PI};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// INI template data matching `DumbProjectileBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct DumbProjectileBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub first_height: Real,
    pub second_height: Real,
    pub first_percent_indent: Real,
    pub second_percent_indent: Real,
    pub max_lifespan: UnsignedInt,
    pub tumble_randomly: Bool,
    pub orient_to_flight_path: Bool,
    pub detonate_calls_kill: Bool,
    pub garrison_hit_kill_count: Int,
    pub garrison_hit_kill_kindof: KindOfMaskType,
    pub garrison_hit_kill_kindof_not: KindOfMaskType,
    pub garrison_hit_kill_fx: Option<String>,
    pub flight_path_adjust_dist_per_frame: Real,
}

impl DumbProjectileBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            first_height: 0.0,
            second_height: 0.0,
            first_percent_indent: 0.0,
            second_percent_indent: 0.0,
            max_lifespan: 10 * LOGICFRAMES_PER_SECOND as UnsignedInt,
            tumble_randomly: FALSE,
            orient_to_flight_path: TRUE,
            detonate_calls_kill: FALSE,
            garrison_hit_kill_count: 0,
            garrison_hit_kill_kindof: KindOfMaskType::new(),
            garrison_hit_kill_kindof_not: KindOfMaskType::new(),
            garrison_hit_kill_fx: None,
            flight_path_adjust_dist_per_frame: 0.0,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Result<(), IniError> {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("MaxLifespan") {
            reader.parse_duration_unsigned_int(&mut self.max_lifespan)
        } else if is("TumbleRandomly") {
            reader.parse_bool(&mut self.tumble_randomly)
        } else if is("DetonateCallsKill") {
            reader.parse_bool(&mut self.detonate_calls_kill)
        } else if is("OrientToFlightPath") {
            reader.parse_bool(&mut self.orient_to_flight_path)
        } else if is("FirstHeight") {
            reader.parse_real(&mut self.first_height)
        } else if is("SecondHeight") {
            reader.parse_real(&mut self.second_height)
        } else if is("FirstPercentIndent") {
            reader.parse_percent_to_real(&mut self.first_percent_indent)
        } else if is("SecondPercentIndent") {
            reader.parse_percent_to_real(&mut self.second_percent_indent)
        } else if is("GarrisonHitKillRequiredKindOf") {
            self.garrison_hit_kill_kindof
                .parse_from_ini(reader, &KINDOF_NAMES)
        } else if is("GarrisonHitKillForbiddenKindOf") {
            self.garrison_hit_kill_kindof_not
                .parse_from_ini(reader, &KINDOF_NAMES)
        } else if is("GarrisonHitKillCount") {
            let mut count: UnsignedInt = 0;
            reader.parse_unsigned_int(&mut count)?;
            self.garrison_hit_kill_count = count as Int;
            Ok(())
        } else if is("GarrisonHitKillFX") {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            self.garrison_hit_kill_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if is("FlightPathAdjustDistPerSecond") {
            reader.parse_velocity_real(&mut self.flight_path_adjust_dist_per_frame)
        } else {
            Err(IniError::UnknownToken)
        }
    }
}

impl ModuleData for DumbProjectileBehaviorModuleData {
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
        FacetMask(0x001 | FacetMask::PROJECTILE_UPDATE.0)
    }
}

/// `Matrix3D::buildTransformMatrix(pos, dir)`: yaw then pitch from the direction's own
/// components, without trigonometry.
fn build_transform_matrix(pos: &Coord3D, dir: &Coord3D) -> Matrix3D {
    let len2 = gm_sqrtf((dir.x * dir.x) + (dir.y * dir.y));
    let sinp = dir.z;
    let cosp = len2;
    let (siny, cosy) = if len2 != 0.0 {
        (dir.y / len2, dir.x / len2)
    } else {
        (0.0, 1.0)
    };

    let mut m = Matrix3D::identity();
    m.translate(pos.x, pos.y, pos.z);
    // `Rotate_Z(siny, cosy)`.
    for row in &mut m.rows {
        let t1 = row[0];
        let t2 = row[1];
        row[0] = cosy * t1 + siny * t2;
        row[1] = -siny * t1 + cosy * t2;
    }
    // `Rotate_Y(-sinp, cosp)`.
    let (s, c) = (-sinp, cosp);
    for row in &mut m.rows {
        let t1 = row[0];
        let t2 = row[2];
        row[0] = c * t1 - s * t2;
        row[2] = s * t1 + c * t2;
    }
    m
}

/// Runtime instance of `DumbProjectileBehavior` (`PKT-267`).
pub struct DumbProjectileBehavior {
    pub data: DumbProjectileBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
    pub launcher_id: ObjectId,
    pub victim_id: ObjectId,
    /// Name of `m_detonationWeaponTmpl`.
    pub detonation_weapon: Option<String>,
    pub lifespan_frame: UnsignedInt,
    pub flight_path: Vec<Coord3D>,
    pub flight_path_start: Coord3D,
    pub flight_path_end: Coord3D,
    pub flight_path_speed: Real,
    pub flight_path_segments: Int,
    pub current_flight_path_step: Int,
    /// Not saved.
    pub extra_bonus_flags: UnsignedInt,
    /// Not saved.
    pub has_detonated: Bool,
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

fn center_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    let mut center = Coord3D::default();
    if let Some(o) = sim.object(slot) {
        o.geometry
            .get_center_position(o.thing.get_position(), &mut center);
    }
    center
}

fn unresistable_detonation_damage(sim: &mut dyn SimApi, owner: ObjectSlot) {
    let mut damage = DamageInfo::default();
    damage.input.damage_type = DamageType::Unresistable;
    damage.input.death_type = DeathType::Detonated;
    damage.input.source_id = ObjectId::INVALID;
    damage.input.amount = sim.object_max_health(owner);
    sim.attempt_damage(owner, &mut damage);
}

impl DumbProjectileBehavior {
    pub fn new(data: DumbProjectileBehaviorModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            launcher_id: ObjectId::INVALID,
            victim_id: ObjectId::INVALID,
            detonation_weapon: None,
            lifespan_frame: 0,
            flight_path: Vec::new(),
            flight_path_start: Coord3D::default(),
            flight_path_end: Coord3D::default(),
            flight_path_speed: 0.0,
            flight_path_segments: 0,
            current_flight_path_step: 0,
            extra_bonus_flags: 0,
            has_detonated: FALSE,
        }
    }

    /// `calcFlightPath`. `false` when the line leaves the map.
    fn calc_flight_path(&mut self, sim: &dyn SimView, recalc_num_segments: bool) -> bool {
        let start = self.flight_path_start;
        let end = self.flight_path_end;
        let Some(mut highest) = sim.estimate_highest_terrain_along_line(&start, &end) else {
            return false;
        };

        let mut dir = Coord3D::new(end.x - start.x, end.y - start.y, end.z - start.z);
        let distance = dir.length();
        dir.normalize_ww_vector3();

        let first = distance * self.data.first_percent_indent;
        let second = distance * self.data.second_percent_indent;

        if highest < start.z {
            highest = start.z;
        }
        if highest < end.z {
            highest = end.z;
        }

        let p1 = Coord3D::new(
            dir.x * first + start.x,
            dir.y * first + start.y,
            highest + self.data.first_height,
        );
        let p2 = Coord3D::new(
            dir.x * second + start.x,
            dir.y * second + start.y,
            highest + self.data.second_height,
        );
        let curve = BezierSegment::from_points(start, p1, p2, end);

        if recalc_num_segments {
            let length = curve.get_approximate_length(1.0);
            let divisor = if self.flight_path_speed < 1.0 {
                1.0
            } else {
                self.flight_path_speed
            };
            self.flight_path_segments = gm_ceilf(length / divisor) as Int;
        }
        curve.get_segment_points(self.flight_path_segments, &mut self.flight_path);
        true
    }

    /// `projectileFireAtObjectOrPosition`.
    fn fire_at(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        victim: Option<ObjectSlot>,
        victim_pos: &Coord3D,
        detonate_weapon_name: Option<&str>,
    ) {
        let projectile = ctx.owner_slot;
        let mut weapon_speed = 0.0;
        let mut scaled = None;
        if let Some(wt) =
            detonate_weapon_name.and_then(|n| sim.rules().weapon_store.find_weapon_template(n))
        {
            weapon_speed = wt.weapon_speed;
            if wt.scale_weapon_speed != FALSE {
                scaled = Some((
                    wt.get_minimum_attack_range(),
                    wt.get_unmodified_attack_range(),
                    wt.min_weapon_speed,
                ));
            }
        }

        let target = match victim {
            Some(v) => center_of(sim, v),
            None => *victim_pos,
        };

        if let Some((min_range, max_range, min_speed)) = scaled {
            let here = position_of(sim, projectile);
            let dx = here.x - target.x;
            let dy = here.y - target.y;
            let range = gm_sqrtf(dx * dx + dy * dy);
            let ratio = (range - min_range) / (max_range - min_range);
            self.flight_path_speed = (ratio * (weapon_speed - min_speed)) + min_speed;
        } else {
            self.flight_path_speed = weapon_speed;
        }

        if self.data.tumble_randomly != FALSE && sim.has_physics(projectile) != FALSE {
            let pitch = sim.rng_logic_real(-1.0 / PI, 1.0 / PI);
            sim.physics_set_pitch_rate(projectile, pitch);
            let yaw = sim.rng_logic_real(-1.0 / PI, 1.0 / PI);
            sim.physics_set_yaw_rate(projectile, yaw);
            let roll = sim.rng_logic_real(-1.0 / PI, 1.0 / PI);
            sim.physics_set_roll_rate(projectile, roll);
        }

        self.flight_path_start = position_of(sim, projectile);
        self.flight_path_end = target;
        if !self.calc_flight_path(sim, true) {
            sim.destroy_object(projectile);
            return;
        }
        self.current_flight_path_step = 0;
    }

    /// `detonate`.
    fn detonate(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.has_detonated != FALSE {
            return;
        }
        let us = ctx.owner_slot;
        if let Some(weapon) = self.detonation_weapon.clone() {
            let pos = position_of(sim, us);
            weapon_fire::handle_projectile_detonation(
                sim,
                us,
                &weapon,
                &pos,
                self.extra_bonus_flags,
            );
            if self.data.detonate_calls_kill != FALSE {
                unresistable_detonation_damage(sim, us);
            } else {
                sim.destroy_object(us);
            }
        } else {
            unresistable_detonation_damage(sim, us);
        }
        self.has_detonated = TRUE;
    }

    /// The garrison-clearing hit. `true` when somebody inside was killed.
    fn clear_garrison(
        &self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        launcher: Option<ObjectSlot>,
        other: ObjectSlot,
    ) -> bool {
        if sim.contain_count(other) == 0
            || sim.contain_is_garrisonable(other) == FALSE
            || sim.contain_is_immune_to_clear(other) != FALSE
        {
            return false;
        }
        let mut killed: Int = 0;
        for id in sim.contain_items(other) {
            if killed >= self.data.garrison_hit_kill_count {
                break;
            }
            let Some(victim) = sim.find_object(id) else {
                continue;
            };
            if sim
                .object(victim)
                .is_none_or(|o| o.is_effectively_dead() != FALSE)
                || sim.object_is_kind_of_multi(
                    victim,
                    &self.data.garrison_hit_kill_kindof,
                    &self.data.garrison_hit_kill_kindof_not,
                ) == FALSE
            {
                continue;
            }
            if launcher.is_some() {
                sim.notify_object_killed(victim, self.launcher_id, &DamageInfo::default());
            }
            kill_object(sim, victim, DamageType::Unresistable, DeathType::Normal);
            killed += 1;
        }
        if killed == 0 {
            return false;
        }
        if let Some(fx) = &self.data.garrison_hit_kill_fx {
            let pos = position_of(sim, other);
            sim.trigger_fx_list(fx, &pos);
        }
        sim.record_cleared_garrisoned_building(ctx.owner_slot);
        sim.destroy_object(ctx.owner_slot);
        true
    }

    fn collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
    ) -> Bool {
        let us = ctx.owner_slot;
        if let Some(other) = other {
            let launcher = sim.find_object(self.launcher_id);
            if !weapon_fire::should_projectile_collide_with(
                sim,
                self.detonation_weapon.as_deref(),
                launcher,
                us,
                other,
                self.victim_id,
            ) {
                // Reports the collision as handled, so the projectile just flies on.
                return TRUE;
            }
            if self.data.garrison_hit_kill_count > 0
                && self.clear_garrison(sim, ctx, launcher, other)
            {
                return TRUE;
            }
        }
        self.detonate(sim, ctx);
        sim.set_status(us, ObjectStatusType::NoCollisions);
        TRUE
    }

    /// Moves the end of the path towards a victim that has moved. `false` when the new
    /// path leaves the map.
    fn track_victim(&mut self, sim: &mut dyn SimApi) -> bool {
        let Some(victim) = sim.find_object(self.victim_id) else {
            return true;
        };
        let new_pos = center_of(sim, victim);
        let mut delta = Coord3D::new(
            new_pos.x - self.flight_path_end.x,
            new_pos.y - self.flight_path_end.y,
            new_pos.z - self.flight_path_end.z,
        );
        let moved_sqr = delta.length_sqr();
        if moved_sqr <= 0.1 {
            return true;
        }
        let mut moved = gm_sqrtf(moved_sqr);
        if moved > self.data.flight_path_adjust_dist_per_frame {
            moved = self.data.flight_path_adjust_dist_per_frame;
        }
        delta.normalize();
        self.flight_path_end.x += moved * delta.x;
        self.flight_path_end.y += moved * delta.y;
        self.flight_path_end.z += moved * delta.z;
        self.calc_flight_path(sim, false)
    }

    fn point(&self, index: i64) -> Coord3D {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.flight_path.get(i))
            .copied()
            .unwrap_or_default()
    }

    /// `DumbProjectileBehavior::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let us = ctx.owner_slot;
        if self.lifespan_frame != 0 && sim.frame() >= self.lifespan_frame {
            self.detonate(sim, ctx);
            return UpdateSleepTime::NONE;
        }
        // A negative step converts to a huge unsigned value and detonates too.
        if self.current_flight_path_step as i64 as u64 >= self.flight_path.len() as u64 {
            self.detonate(sim, ctx);
            return UpdateSleepTime::NONE;
        }

        if self.victim_id != ObjectId::INVALID
            && self.data.flight_path_adjust_dist_per_frame > 0.0
            && !self.track_victim(sim)
        {
            self.detonate(sim, ctx);
            return UpdateSleepTime::NONE;
        }

        let step = self.current_flight_path_step as i64;
        let mut flight_step = self.point(step);
        if self.data.orient_to_flight_path != FALSE && self.data.tumble_randomly == FALSE {
            let (prev, cur) = if step > 0 {
                (self.point(step - 1), flight_step)
            } else if self.flight_path.len() >= 2 {
                (self.point(0), self.point(1))
            } else {
                flight_step = self.flight_path_end;
                (self.flight_path_start, self.flight_path_end)
            };
            let mut dir = Coord3D::new(cur.x - prev.x, cur.y - prev.y, cur.z - prev.z);
            dir.normalize_ww_vector3();
            let mtx = build_transform_matrix(&flight_step, &dir);
            sim.set_object_transform_matrix(us, &mtx);
        } else {
            sim.set_object_position(us, &flight_step);
        }

        let Some(old_layer) = sim.object(us).map(|o| o.layer) else {
            return UpdateSleepTime::NONE;
        };
        let here = position_of(sim, us);
        let new_layer = sim.get_highest_layer_for_destination(&here, FALSE);
        sim.set_object_layer(us, new_layer);

        if old_layer != PathfindLayerEnum::Ground && new_layer == PathfindLayerEnum::Ground {
            let mut tmp = position_of(sim, us);
            tmp.z = 9999.0;
            let test_layer = sim.get_highest_layer_for_destination(&tmp, FALSE);
            if test_layer == old_layer {
                const FUDGE: Real = 2.0;
                tmp.z = sim.get_layer_height(tmp.x, tmp.y, test_layer) + FUDGE;
                sim.set_object_position(us, &tmp);
                self.detonate(sim, ctx);
                return UpdateSleepTime::NONE;
            }
        }

        self.current_flight_path_step = self.current_flight_path_step.wrapping_add(1);
        UpdateSleepTime::NONE
    }
}

impl ProjectileFacet for DumbProjectileBehavior {
    fn projectile_launch_at_object_or_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        victim: Option<ObjectSlot>,
        victim_pos: &Coord3D,
        launcher: Option<ObjectSlot>,
        weapon_slot: WeaponSlotType,
        specific_barrel_to_use: Int,
        detonate_weapon_name: Option<&str>,
        exhaust_sys_name: Option<&str>,
    ) {
        let id_of = |sim: &dyn SimApi, slot: Option<ObjectSlot>| {
            slot.and_then(|s| sim.object(s))
                .map_or(ObjectId::INVALID, |o| o.id)
        };
        self.launcher_id = id_of(sim, launcher);
        self.extra_bonus_flags = launcher
            .and_then(|l| sim.object(l))
            .map_or(0, |o| o.weapon_bonus_condition);
        self.victim_id = id_of(sim, victim);
        self.detonation_weapon = detonate_weapon_name
            .filter(|n| sim.rules().weapon_store.find_weapon_template(n).is_some())
            .map(str::to_owned);
        self.lifespan_frame = sim.frame().wrapping_add(self.data.max_lifespan);

        weapon_fire::position_projectile_for_launch(
            sim,
            ctx.owner_slot,
            launcher,
            weapon_slot,
            specific_barrel_to_use,
        );

        self.projectile_fire_at_object_or_position(
            sim,
            ctx,
            victim,
            victim_pos,
            detonate_weapon_name,
            exhaust_sys_name,
        );
    }

    fn projectile_fire_at_object_or_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        victim: Option<ObjectSlot>,
        victim_pos: &Coord3D,
        detonate_weapon_name: Option<&str>,
        _exhaust_sys_name: Option<&str>,
    ) {
        self.fire_at(sim, ctx, victim, victim_pos, detonate_weapon_name);
    }

    fn projectile_get_launcher_id(&self) -> ObjectId {
        self.launcher_id
    }

    fn projectile_handle_collision(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
    ) -> Bool {
        self.collide(sim, ctx, other)
    }
}

impl UpdateFacet for DumbProjectileBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }
}

impl BehaviorModule for DumbProjectileBehavior {
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
        let current: u8 = if cfg!(feature = "retail-xfer-save") {
            1
        } else {
            2
        };
        let version = xfer_module_version(xfer, current)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_object_id(&mut self.launcher_id)?;
        xfer.xfer_object_id(&mut self.victim_id)?;
        xfer.xfer_int(&mut self.flight_path_segments)?;
        xfer.xfer_real(&mut self.flight_path_speed)?;
        xfer.xfer_coord3d(&mut self.flight_path_start)?;
        xfer.xfer_coord3d(&mut self.flight_path_end)?;

        let mut name = self.detonation_weapon.clone().unwrap_or_default();
        xfer.xfer_ascii_string(&mut name)?;
        if xfer.mode() == XferMode::Load {
            self.detonation_weapon = if name.is_empty() {
                None
            } else if sim
                .rules()
                .weapon_store
                .find_weapon_template(&name)
                .is_some()
            {
                Some(name)
            } else {
                return Err(XferError::InvalidData);
            };
        }

        xfer.xfer_unsigned_int(&mut self.lifespan_frame)?;
        if version >= 2 {
            xfer.xfer_int(&mut self.current_flight_path_step)?;
        }
        Ok(())
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
        if self.flight_path_segments > 0 {
            // The path is not saved.
            self.calc_flight_path(sim, false);
        }
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

    fn as_projectile(&self) -> Option<&dyn ProjectileFacet> {
        Some(self)
    }

    fn as_projectile_mut(&mut self) -> Option<&mut dyn ProjectileFacet> {
        Some(self)
    }
}

pub fn parse_dumb_projectile_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = DumbProjectileBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| data.parse_field(field, r, parse_ctx))?;
    Ok(Box::new(data))
}

pub fn create_dumb_projectile_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<DumbProjectileBehaviorModuleData>()
        .expect("DumbProjectileBehavior data type mismatch")
        .clone();
    Box::new(DumbProjectileBehavior::new(typed))
}
