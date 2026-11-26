//! `PKT-271`: `GarrisonContain` (`GarrisonContain.h`, `GarrisonContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-271_GarrisonContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.
//!
//! Unported subsystems this module touches:
//! - Art bones: headless drawables report no `FIREPOINT` / `STATION` bones.
//! - Client effects: the `GarrisonGun` muzzle drawables and occlusion frames are not modelled,
//!   so effect drawable ids stay invalid and last-effect frames stay zero.
//! - Weapon range tests: a garrisoned unit that holds a weapon is treated as in range, and
//!   `attemptBestFirePointPosition` (weapon system entry point) is not provided.
//! - Academy statistics.

use crate::arena::{FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::{BodyDamageType, WeaponBonusConditionType};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    add_to_contain, impl_contain_module, order_all_passengers_to_exit, remove_all_contained,
    remove_from_contain, update_open_contain, ContainBehavior, OpenContainCore,
    OpenContainModuleData,
};
use crate::modules::contain::transport_contain::{infantry_kind_of_mask, InitialPayload};
use crate::modules::module::{
    BehaviorModule, CommandSourceType, EvacDisposition, ExitDoorType, ModuleData,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{FindPositionOptions, SimApi, SimView, FPF_IGNORE_ALLY_OR_NEUTRAL_UNITS};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort,
    UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const MAX_GARRISON_POINTS: usize = 40;
pub const MAX_GARRISON_POINT_CONDITIONS: usize = 3;

const GARRISON_POINT_PRISTINE: usize = 0;
const GARRISON_POINT_DAMAGED: usize = 1;
const GARRISON_POINT_REALLY_DAMAGED: usize = 2;

const TEAM_ID_INVALID: Int = 0;
const INVALID_DRAWABLE_ID: UnsignedInt = 0;

/// INI template data matching `GarrisonContainModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct GarrisonContainModuleData {
    pub open: OpenContainModuleData,
    pub do_i_heal_objects: Bool,
    pub frames_for_full_heal: Real,
    pub mobile_garrison: Bool,
    pub immune_to_clear_building_attacks: Bool,
    pub is_enclosing_container: Bool,
    pub initial_roster: InitialPayload,
}

impl GarrisonContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut open = OpenContainModuleData::new(tag_key, module_name_key);
        open.allow_inside_kind_of = infantry_kind_of_mask();
        Self {
            open,
            do_i_heal_objects: FALSE,
            frames_for_full_heal: 1.0,
            mobile_garrison: FALSE,
            immune_to_clear_building_attacks: FALSE,
            is_enclosing_container: TRUE,
            initial_roster: InitialPayload::default(),
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        let result = if is("MobileGarrison") {
            reader.parse_bool(&mut self.mobile_garrison)
        } else if is("HealObjects") {
            reader.parse_bool(&mut self.do_i_heal_objects)
        } else if is("TimeForFullHeal") {
            reader.parse_duration_real(&mut self.frames_for_full_heal)
        } else if is("InitialRoster") {
            self.initial_roster.parse(reader)
        } else if is("ImmuneToClearBuildingAttacks") {
            reader.parse_bool(&mut self.immune_to_clear_building_attacks)
        } else if is("IsEnclosingContainer") {
            reader.parse_bool(&mut self.is_enclosing_container)
        } else {
            return self.open.parse_field(field, reader, parse_ctx);
        };
        Some(result)
    }
}

impl ModuleData for GarrisonContainModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.open.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.open.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// `GarrisonContain::GarrisonPointData`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GarrisonPointData {
    /// Object at this garrison point ([`ObjectId::INVALID`] = free).
    pub object_id: ObjectId,
    pub target_id: ObjectId,
    pub place_frame: UnsignedInt,
    pub last_effect_frame: UnsignedInt,
    pub effect_drawable_id: UnsignedInt,
}

impl Default for GarrisonPointData {
    fn default() -> Self {
        Self {
            object_id: ObjectId::INVALID,
            target_id: ObjectId::INVALID,
            place_frame: 0,
            last_effect_frame: 0,
            effect_drawable_id: INVALID_DRAWABLE_ID,
        }
    }
}

/// `GarrisonContain::StationPointData`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StationPointData {
    pub occupant_id: ObjectId,
    pub position: Coord3D,
}

/// Runtime instance of `GarrisonContain` (`PKT-271`).
pub struct GarrisonContain {
    pub data: GarrisonContainModuleData,
    pub core: OpenContainCore,
    /// Our team before we were garrisoned.
    pub original_team: Option<TeamId>,
    pub point_data: [GarrisonPointData; MAX_GARRISON_POINTS],
    pub points_in_use: Int,
    pub points: [[Coord3D; MAX_GARRISON_POINTS]; MAX_GARRISON_POINT_CONDITIONS],
    pub exit_rally_point: Coord3D,
    pub station_points: Vec<StationPointData>,
    /// Not serialized.
    pub station_points_initialized: Bool,
    pub points_initialized: Bool,
    /// Do not appear garrisoned to non-allies (everyone inside is stealthy).
    pub hide_garrisoned_state_from_nonallies: Bool,
    pub rally_valid: Bool,
    pub evac_disposition: EvacDisposition,
}

fn dist_sqr(a: &Coord3D, b: &Coord3D) -> Real {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    let dz = a.z - b.z;
    dx * dx + dy * dy + dz * dz
}

impl GarrisonContain {
    pub fn new(data: GarrisonContainModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        Self {
            data,
            core,
            original_team: None,
            point_data: [GarrisonPointData::default(); MAX_GARRISON_POINTS],
            points_in_use: 0,
            points: [[Coord3D::default(); MAX_GARRISON_POINTS]; MAX_GARRISON_POINT_CONDITIONS],
            exit_rally_point: Coord3D::default(),
            station_points: Vec::new(),
            station_points_initialized: FALSE,
            points_initialized: FALSE,
            hide_garrisoned_state_from_nonallies: FALSE,
            rally_valid: FALSE,
            evac_disposition: EvacDisposition::BurstFromCenter,
        }
    }

    /// `GarrisonContain::getApparentControllingPlayer`, in player indices.
    pub fn apparent_controlling_player(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        observing_player: Int,
    ) -> Int {
        let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return -1;
        };
        let mine = sim.player_index(team);
        if self.hide_garrisoned_state_from_nonallies != FALSE {
            if let Some(original) = self.original_team {
                if sim.team_relationship_to_player(team, observing_player) != Relationship::Allies {
                    return sim.player_index(original);
                }
            }
        }
        mine
    }

    /// `Object::setTeam`: dead players hand things to the neutral team; the capture hooks run.
    fn set_team(sim: &mut dyn SimApi, owner: ObjectSlot, team: TeamId) {
        let team = if sim.player_is_active(team) == FALSE {
            sim.neutral_team_id()
        } else {
            team
        };
        sim.capture_object(owner, team);
    }

    /// `GarrisonContain::recalcApparentControllingPlayer`.
    pub fn recalc_apparent_controlling_player(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) {
        let owner = ctx.owner_slot;
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return;
        };
        // Record the original team the first time through.
        if self.original_team.is_none() {
            self.original_team = Some(team);
        }

        let first_rider = self
            .core
            .contain_list
            .first()
            .and_then(|&id| sim.find_object(id));
        let detected = |sim: &dyn SimApi, rider: ObjectSlot| {
            sim.object(rider).is_some_and(|o| {
                o.status.test(ObjectStatusType::Detected.as_i32()) != FALSE
            })
        };

        if self.core.contain_list_size > 0 {
            if let Some(rider) = first_rider {
                // Needed before the capture so radar colouring sees the right flag.
                self.hide_garrisoned_state_from_nonallies = Bool::from(
                    !detected(sim, rider)
                        && self.core.stealth_units_contained == self.core.contain_list_size,
                );
                if let Some(rider_team) = sim.object(rider).map(|o| o.team_id) {
                    let team = sim.player_default_team(rider_team);
                    Self::set_team(sim, owner, team);
                }
            }
        } else {
            if let Some(original) = self.original_team {
                Self::set_team(sim, owner, original);
            }
            self.hide_garrisoned_state_from_nonallies = FALSE;
        }

        if sim.object_has_drawable(owner) == FALSE {
            return;
        }
        let mut set_model_garrisoned = false;
        if self.core.contain_list_size > 0 {
            if let Some(occupant) = first_rider {
                let local = sim.local_player_index();
                let apparent = self.apparent_controlling_player(sim, ctx, local);
                let controlling = sim.object(owner).map_or(-1, |o| sim.player_index(o.team_id));
                set_model_garrisoned = detected(sim, occupant) || apparent == controlling;
            }
        }
        if set_model_garrisoned {
            sim.set_model_condition_state(owner, ModelConditionFlagType::Garrisoned);
        } else {
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Garrisoned);
        }

        // Now that something is inside, cache the garrison positions.
        if self.core.contain_list_size > 0 {
            if self.data.is_enclosing_container != FALSE {
                if self.points_initialized == FALSE {
                    self.load_garrison_points(sim, ctx);
                }
            } else if self.station_points_initialized == FALSE {
                self.load_station_garrison_points(sim, ctx);
            }
        }
    }

    /// `loadGarrisonPoints`: with no `FIREPOINT` bones every point is the structure centre.
    fn load_garrison_points(&mut self, sim: &dyn SimView, ctx: ModuleContext) {
        let Some(pos) = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position()) else {
            return;
        };
        self.points = [[pos; MAX_GARRISON_POINTS]; MAX_GARRISON_POINT_CONDITIONS];
        self.points_initialized = TRUE;
    }

    /// `loadStationGarrisonPoints`: no `STATION` bones, so the list ends up empty.
    fn load_station_garrison_points(&mut self, _sim: &dyn SimView, _ctx: ModuleContext) {
        self.station_points.clear();
        self.station_points_initialized = TRUE;
    }

    /// `findConditionIndex`.
    fn find_condition_index(sim: &dyn SimView, ctx: ModuleContext) -> usize {
        let state = sim
            .object(ctx.owner_slot)
            .and_then(|o| o.body_slot)
            .and_then(|body| match sim.module_sidecar(body) {
                ModuleSidecar::Body { damage_state, .. } => Some(Int::from(damage_state)),
                _ => None,
            })
            .unwrap_or(BodyDamageType::Pristine.as_i32());
        if state == BodyDamageType::Pristine.as_i32() {
            GARRISON_POINT_PRISTINE
        } else if state == BodyDamageType::Damaged.as_i32() {
            GARRISON_POINT_DAMAGED
        } else {
            GARRISON_POINT_REALLY_DAMAGED
        }
    }

    fn find_closest_free_point(&self, condition: usize, target: &Coord3D) -> Option<usize> {
        if self.points_in_use as usize == MAX_GARRISON_POINTS {
            return None;
        }
        let mut closest = None;
        let mut closest_dist_sq: Real = -1.0;
        for i in 0..MAX_GARRISON_POINTS {
            if self.point_data[i].object_id == ObjectId::INVALID {
                let d = dist_sqr(target, &self.points[condition][i]);
                if d < closest_dist_sq || closest_dist_sq == -1.0 {
                    closest_dist_sq = d;
                    closest = Some(i);
                }
            }
        }
        closest
    }

    fn point_index_of(&self, obj_id: ObjectId) -> Option<usize> {
        self.point_data.iter().position(|p| p.object_id == obj_id)
    }

    /// `calcBestGarrisonPosition`: where a range check should assume the unit will stand.
    pub fn calc_best_garrison_position(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        target_pos: &Coord3D,
    ) -> Option<Coord3D> {
        let condition = Self::find_condition_index(sim, ctx);
        self.find_closest_free_point(condition, target_pos)
            .map(|i| self.points[condition][i])
    }

    fn put_object_at_point(
        &mut self,
        sim: &mut dyn SimApi,
        obj: ObjectSlot,
        target_id: ObjectId,
        condition: usize,
        index: usize,
    ) {
        let Some(obj_id) = sim.object(obj).map(|o| o.id) else {
            return;
        };
        if self.point_data[index].object_id != ObjectId::INVALID {
            return;
        }
        let pos = self.points[condition][index];
        sim.set_object_position(obj, &pos);
        self.point_data[index].object_id = obj_id;
        self.point_data[index].target_id = target_id;
        self.point_data[index].place_frame = sim.frame();
        self.points_in_use += 1;
    }

    fn put_object_at_best_point(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        target: Option<ObjectSlot>,
        target_pos: Option<Coord3D>,
    ) {
        let target_record = target.and_then(|t| sim.object(t));
        let target_id = target_record.map_or(ObjectId::INVALID, |o| o.id);
        let Some(target_pos) = target_record
            .map(|o| *o.thing.get_position())
            .or(target_pos)
        else {
            return;
        };
        let Some(obj_id) = sim.object(obj).map(|o| o.id) else {
            return;
        };
        if self.point_index_of(obj_id).is_some() {
            return;
        }
        let condition = Self::find_condition_index(sim, ctx);
        if let Some(index) = self.find_closest_free_point(condition, &target_pos) {
            self.put_object_at_point(sim, obj, target_id, condition, index);
        }
    }

    /// `removeObjectFromGarrisonPoint` (`index = None` searches for the object).
    fn remove_object_from_point(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        index: Option<usize>,
    ) {
        if self.data.is_enclosing_container == FALSE {
            return; // Not enclosed, so never at a garrison point.
        }
        let Some(obj_id) = sim.object(obj).map(|o| o.id) else {
            return;
        };
        let Some(index) = index.or_else(|| self.point_index_of(obj_id)) else {
            return;
        };
        self.point_data[index] = GarrisonPointData::default();
        self.points_in_use -= 1;
        if let Some(pos) = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position()) {
            sim.set_object_position(obj, &pos);
        }
    }

    fn remove_invalid_objects_from_points(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.points_in_use == 0 {
            return;
        }
        for i in 0..MAX_GARRISON_POINTS {
            let Some(obj) = sim.find_object(self.point_data[i].object_id) else {
                continue;
            };
            let target_is_valid = sim.object_ai_goal_object(obj).is_none()
                || sim
                    .object(obj)
                    .is_some_and(|o| o.weapon_set.has_any_weapon() != FALSE);
            let attacking = sim.object(obj).is_some_and(|o| {
                o.status.test(ObjectStatusType::IsAttacking.as_i32()) != FALSE
            });
            if !attacking || !target_is_valid {
                self.remove_object_from_point(sim, ctx, obj, Some(i));
            }
        }
    }

    fn add_valid_objects_to_points(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for id in self.core.contain_list.clone() {
            let Some(obj) = sim.find_object(id) else {
                continue;
            };
            if sim.has_ai(obj) == FALSE {
                continue;
            }
            if let Some(victim) = sim.ai_current_victim(obj) {
                self.put_object_at_best_point(sim, ctx, obj, Some(victim), None);
            } else if let Some(pos) = sim.ai_current_victim_pos(obj) {
                self.put_object_at_best_point(sim, ctx, obj, None, Some(pos));
            }
        }
    }

    /// Keep attackers at the free point closest to their target.
    fn track_targets(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.data.is_enclosing_container == FALSE {
            return;
        }
        let condition = Self::find_condition_index(sim, ctx);
        for id in self.core.contain_list.clone() {
            let Some(obj) = sim.find_object(id) else {
                continue;
            };
            let Some(index) = self.point_index_of(id) else {
                continue;
            };
            if sim.has_ai(obj) == FALSE {
                continue;
            }
            let victim = sim.ai_current_victim(obj);
            let victim_pos = victim
                .and_then(|v| sim.object(v))
                .map(|o| *o.thing.get_position())
                .or_else(|| sim.ai_current_victim_pos(obj));
            let (Some(victim_pos), Some(our_pos)) = (
                victim_pos,
                sim.object(obj).map(|o| *o.thing.get_position()),
            ) else {
                continue;
            };
            if let Some(new_index) = self.find_closest_free_point(condition, &victim_pos) {
                let current = dist_sqr(&victim_pos, &our_pos);
                let new_dist = dist_sqr(&victim_pos, &self.points[condition][new_index]);
                if new_dist < current {
                    self.remove_object_from_point(sim, ctx, obj, Some(index));
                    let target_id = victim
                        .and_then(|v| sim.object(v))
                        .map_or(ObjectId::INVALID, |o| o.id);
                    self.put_object_at_point(sim, obj, target_id, condition, new_index);
                }
            }
        }
    }

    fn match_objects_to_points(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.data.is_enclosing_container == FALSE {
            self.position_objects_at_station_points(sim, ctx);
        } else {
            self.remove_invalid_objects_from_points(sim, ctx);
            self.add_valid_objects_to_points(sim, ctx);
            // Muzzle flashes (`updateEffects`) are client only.
            self.track_targets(sim, ctx);
        }
    }

    fn position_objects_at_station_points(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.station_points_initialized == FALSE {
            self.load_station_garrison_points(sim, ctx);
        }
        for id in self.core.contain_list.clone() {
            let Some(obj) = sim.find_object(id) else {
                continue;
            };
            if let Some(spot) = self.station_points.iter().find(|p| p.occupant_id == id) {
                let pos = spot.position;
                sim.set_object_position(obj, &pos);
            } else if let Some(vacancy) = self
                .station_points
                .iter_mut()
                .find(|p| p.occupant_id == ObjectId::INVALID)
            {
                vacancy.occupant_id = id;
            }
        }
    }

    fn remove_object_from_station_point(&mut self, obj_id: ObjectId) {
        if let Some(spot) = self
            .station_points
            .iter_mut()
            .find(|p| p.occupant_id == obj_id)
        {
            spot.occupant_id = ObjectId::INVALID;
        }
    }

    fn heal_objects(&mut self, sim: &mut dyn SimApi) {
        if self.data.do_i_heal_objects == FALSE {
            return;
        }
        let frames = self.data.frames_for_full_heal;
        for id in self.core.contain_list.clone() {
            let Some(obj) = sim.find_object(id) else {
                continue;
            };
            let Some(contained_by_frame) = sim.object(obj).map(|o| o.contained_by_frame) else {
                continue;
            };
            let max_health = sim.object_max_health(obj);
            let mut heal = DamageInfo::default();
            heal.input.damage_type = DamageType::Healing;
            heal.input.death_type = DeathType::NoneType;
            heal.input.amount =
                if sim.frame().wrapping_sub(contained_by_frame) as Real >= frames {
                    max_health
                } else {
                    max_health / frames
                };
            sim.attempt_healing(obj, &mut heal);
        }
    }

    /// `validateRallyPoint`: keep or pick an exit rally point.
    fn validate_rally_point(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        let Some((pos, radius)) = sim.object(owner).map(|o| {
            (
                *o.thing.get_position(),
                o.geometry.get_bounding_circle_radius(),
            )
        }) else {
            return;
        };
        let mut options = FindPositionOptions {
            flags: FPF_IGNORE_ALLY_OR_NEUTRAL_UNITS,
            ignore_object: Some(owner),
            relationship_object: Some(owner),
            ..FindPositionOptions::default()
        };
        if self.rally_valid != FALSE {
            options.min_radius = 0.0;
            options.max_radius = 0.0;
            let rally = self.exit_rally_point;
            if sim.find_position_around(&rally, &options).is_none() {
                self.rally_valid = FALSE;
            }
        }
        if self.rally_valid == FALSE {
            options.min_radius = radius;
            options.max_radius = radius * 1.8;
            if let Some(found) = sim.find_position_around(&pos, &options) {
                self.exit_rally_point = found;
                self.rally_valid = TRUE;
            }
        }
    }

    /// `GarrisonContain::exitObjectViaDoor`.
    fn exit_garrison(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, exit_obj: ObjectSlot) {
        let owner = ctx.owner_slot;
        remove_from_contain(self, sim, ctx, exit_obj, FALSE);

        let Some((owner_pos, exit_angle, half_length, half_width, mtx)) =
            sim.object(owner).map(|o| {
                (
                    *o.thing.get_position(),
                    o.thing.get_orientation(),
                    o.geometry.get_major_radius(),
                    o.geometry.get_minor_radius(),
                    *o.thing.get_transform_matrix(),
                )
            })
        else {
            return;
        };
        let mut start = owner_pos;
        let has_ai = sim.has_ai(exit_obj) != FALSE;
        // Cliff bunkers: units start in a cliff, so try the front and back instead.
        if has_ai
            && sim.ai_has_cur_locomotor(exit_obj) != FALSE
            && sim.valid_movement_terrain(PathfindLayerEnum::Ground, exit_obj, &start) == FALSE
        {
            start.x -= half_length * gm_cosf(exit_angle);
            start.y -= half_length * gm_sinf(exit_angle);
            if sim.valid_movement_terrain(PathfindLayerEnum::Ground, exit_obj, &start) == FALSE {
                start.x += 2.0 * half_length * gm_cosf(exit_angle);
                start.y += 2.0 * half_length * gm_sinf(exit_angle);
                if sim.valid_movement_terrain(PathfindLayerEnum::Ground, exit_obj, &start) == FALSE
                {
                    start = owner_pos;
                }
            }
        }

        let end = match self.evac_disposition {
            EvacDisposition::ToLeft | EvacDisposition::ToRight => {
                let scalar: Real = if self.evac_disposition == EvacDisposition::ToLeft {
                    1.0
                } else {
                    -1.0
                };
                // A rectangular pocket to act as the "doorway".
                let door_x = sim.rng_logic_real(-half_length / 4.0, half_length / 4.0);
                let door_y = sim.rng_logic_real(half_width / 2.0, half_width * 2.0) * scalar;
                let walk_x = sim.rng_logic_real(-half_length, half_length);
                let walk_y = half_width * 10.0 * scalar; // spread out
                start = Matrix3D::transform_vector(&mtx, &Coord3D::new(door_x, door_y, 0.0));
                let end = Matrix3D::transform_vector(&mtx, &Coord3D::new(walk_x, walk_y, 0.0));

                sim.set_object_position(exit_obj, &start);
                sim.set_object_orientation(exit_obj, exit_angle);
                end
            }
            _ => {
                // Non-enclosed units just walk away from where they already are.
                if self.data.is_enclosing_container != FALSE {
                    let mut pos = start;
                    pos.z = sim.terrain().get_ground_height(start.x, start.y, None);
                    sim.set_object_position(exit_obj, &pos);
                }
                sim.set_object_orientation(exit_obj, exit_angle);
                start
            }
        };

        sim.pathfinder_add_object(exit_obj);
        if has_ai {
            let mut end = end;
            let _ = sim.pathfinder_adjust_to_possible_destination(exit_obj, &mut end);
            sim.ai_follow_path(exit_obj, &[end], Some(owner), CommandSourceType::FromAi);
            let layer = sim.get_layer_for_destination(&end);
            sim.pathfinder_update_goal(exit_obj, &end, layer);
        }

        self.recalc_apparent_controlling_player(sim, ctx);
    }

    fn xfer_garrison(
        &mut self,
        xfer: &mut dyn Xfer,
        sim: &dyn SimView,
    ) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;

        let mut team_id = self.original_team.map_or(TEAM_ID_INVALID, |t| t.0);
        xfer.xfer_int(&mut team_id)?;
        if xfer.mode() == XferMode::Load {
            self.original_team = (team_id != TEAM_ID_INVALID).then_some(TeamId(team_id));
        }
        xfer.xfer_bool(&mut self.hide_garrisoned_state_from_nonallies)?;

        let mut count = MAX_GARRISON_POINTS as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        for i in 0..usize::from(count) {
            let mut point = self.point_data.get(i).copied().unwrap_or_default();
            xfer.xfer_object_id(&mut point.object_id)?;
            xfer.xfer_object_id(&mut point.target_id)?;
            xfer.xfer_unsigned_int(&mut point.place_frame)?;
            xfer.xfer_unsigned_int(&mut point.last_effect_frame)?;
            xfer.xfer_unsigned_int(&mut point.effect_drawable_id)?;
            if let Some(slot) = self.point_data.get_mut(i) {
                *slot = point;
            }
        }

        xfer.xfer_int(&mut self.points_in_use)?;
        for condition in &mut self.points {
            for point in condition {
                xfer.xfer_coord3d(point)?;
            }
        }
        xfer.xfer_bool(&mut self.points_initialized)?;
        xfer.xfer_bool(&mut self.rally_valid)?;
        xfer.xfer_coord3d(&mut self.exit_rally_point)
    }
}

impl ContainBehavior for GarrisonContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }
    fn is_enclosing(&self) -> Bool {
        self.data.is_enclosing_container
    }
    fn is_garrisonable_container(&self) -> Bool {
        TRUE
    }
    fn is_bustable_container(&self) -> Bool {
        TRUE
    }
    fn is_immune_to_clear(&self) -> Bool {
        self.data.immune_to_clear_building_attacks
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.xfer_garrison(xfer, sim)
    }

    /// `GarrisonContain::onObjectCreated`: spawn the initial roster.
    fn object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.schedule.bind(ctx);
        let sidecar = self
            .core
            .sidecar(self.data.open.contain_max, self.data.is_enclosing_container);
        sim.set_module_sidecar(ctx.mod_slot, sidecar);

        let roster = self.data.initial_roster.clone();
        let template = sim.rules().thing_factory.find_template_id(&roster.name);
        let owner_team = sim.object(ctx.owner_slot).map(|o| o.team_id);
        let (Some(template), Some(owner_team)) = (template, owner_team) else {
            return;
        };
        for _ in 0..roster.count {
            let team = sim.player_default_team(owner_team);
            let rider = sim.spawn_object(template, team, Coord3D::default(), 0.0);
            if self.is_valid_container(sim, ctx, rider, TRUE) != FALSE {
                add_to_contain(self, sim, ctx, rider);
            }
        }
    }

    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        obj: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        if self
            .core
            .is_valid_container_for(&self.data.open, sim, ctx, obj)
            == FALSE
        {
            return FALSE;
        }
        let Some((health, damage_state)) = sim
            .object(ctx.owner_slot)
            .and_then(|o| o.body_slot)
            .and_then(|body| match sim.module_sidecar(body) {
                ModuleSidecar::Body {
                    health,
                    damage_state,
                    ..
                } => Some((health, Int::from(damage_state))),
                _ => None,
            })
        else {
            return FALSE;
        };
        // Zero-health buildings are not garrisonable, nor are really damaged ones.
        if health <= 0.0 {
            return FALSE;
        }
        if damage_state == BodyDamageType::ReallyDamaged.as_i32()
            && sim.object_is_kind_of(ctx.owner_slot, KindOfType::GarrisonableUntilDestroyed)
                == FALSE
        {
            return FALSE;
        }
        if sim.object_is_kind_of(obj, KindOfType::NoGarrison) != FALSE {
            return FALSE;
        }
        if check_capacity != FALSE {
            Bool::from((self.core.contain_list_size as Int) < self.data.open.contain_max)
        } else {
            TRUE
        }
    }

    /// A garrison always lets people shoot out unless subdued.
    fn passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        _id: ObjectId,
    ) -> Bool {
        let subdued = sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.is_disabled_by_type(DisabledType::Subdued) != FALSE);
        Bool::from(!subdued)
    }

    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        let structure = ctx.owner_slot;
        sim.set_disabled(rider, DisabledType::Held);
        // The building can now attack, since it has soldiers inside.
        sim.set_status(structure, ObjectStatusType::CanAttack);
        sim.set_weapon_bonus_condition(rider, WeaponBonusConditionType::Garrisoned);
        if self.data.is_enclosing_container != FALSE {
            if let Some(pos) = sim.object(structure).map(|o| *o.thing.get_position()) {
                sim.set_object_position(rider, &pos);
            }
        }
        self.recalc_apparent_controlling_player(sim, ctx);
    }

    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        let owner = ctx.owner_slot;
        if self.data.is_enclosing_container != FALSE {
            self.remove_object_from_point(sim, ctx, rider, None);
        } else {
            if let Some(id) = sim.object(rider).map(|o| o.id) {
                self.remove_object_from_station_point(id);
            }
            if let Some(mut pos) = sim.object(rider).map(|o| *o.thing.get_position()) {
                pos.z = sim.terrain().get_ground_height(pos.x, pos.y, None);
                sim.set_object_position(rider, &pos);
            }
        }
        sim.clear_weapon_bonus_condition(rider, WeaponBonusConditionType::Garrisoned);
        sim.clear_disabled(rider, DisabledType::Held);

        // Nothing left inside: back to the original team.
        if self.core.contain_list_size == 0 {
            if let Some(original) = self.original_team.take() {
                Self::set_team(sim, owner, original);
            }
            sim.clear_status(owner, ObjectStatusType::CanAttack);
            self.hide_garrisoned_state_from_nonallies = FALSE;
            if sim.object_has_drawable(owner) != FALSE {
                sim.clear_model_condition_state(owner, ModelConditionFlagType::Garrisoned);
            }
        } else if self.core.stealth_units_contained != self.core.contain_list_size {
            self.hide_garrisoned_state_from_nonallies = FALSE;
        }

        self.recalc_apparent_controlling_player(sim, ctx);
    }

    /// `GarrisonContain::redeployOccupants` (does not run the `OpenContain` version).
    fn redeploy(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let copy = self.point_data;
        // Forces fresh station points based on the new transform.
        self.station_points_initialized = FALSE;
        self.match_objects_to_points(sim, ctx);
        // Preserve the frames at which things were originally placed.
        for old in copy {
            if old.object_id != ObjectId::INVALID {
                if let Some(index) = self.point_index_of(old.object_id) {
                    self.point_data[index].place_frame = old.place_frame;
                }
            }
        }
    }

    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let _ = update_open_contain(self, sim, ctx);

        // Remove effectively dead occupants.
        for id in self.core.contain_list.clone() {
            let Some(rider) = sim.find_object(id) else {
                continue;
            };
            if sim
                .object(rider)
                .is_some_and(|o| o.is_effectively_dead() != FALSE)
            {
                remove_from_contain(self, sim, ctx, rider, FALSE);
            }
        }

        self.match_objects_to_points(sim, ctx);
        self.heal_objects(sim);

        let mobile = sim.object(ctx.owner_slot).is_some_and(|o| o.is_disabled() == FALSE)
            && sim.object_is_kind_of(ctx.owner_slot, KindOfType::Immobile) == FALSE;
        if self.data.mobile_garrison != FALSE && mobile {
            if let Some(pos) = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position()) {
                for id in self.core.contain_list.clone() {
                    if let Some(rider) = sim.find_object(id) {
                        sim.set_object_position(rider, &pos);
                    }
                }
            }
        }
        UpdateSleepTime::NONE
    }

    fn remove_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, expose_stealthed: Bool) {
        if self.core.contain_list_size > 0 {
            self.validate_rally_point(sim, ctx);
        }
        remove_all_contained(self, sim, ctx, expose_stealthed);
        self.recalc_apparent_controlling_player(sim, ctx);
    }

    fn selling(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.remove_all(sim, ctx, FALSE);
        order_all_passengers_to_exit(self, sim, ctx, CommandSourceType::FromAi);
    }

    fn exit_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        _door: ExitDoorType,
    ) {
        self.exit_garrison(sim, ctx, obj);
    }

    /// Everyone is ejected on crossing the really-damaged threshold.
    fn body_damage_state_changed(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        if new_state == BodyDamageType::ReallyDamaged
            && sim.object_is_kind_of(ctx.owner_slot, KindOfType::GarrisonableUntilDestroyed)
                == FALSE
            && self.core.contain_list_size > 0
        {
            order_all_passengers_to_exit(self, sim, ctx, CommandSourceType::FromAi);
        }
    }

    fn set_evac(&mut self, disp: EvacDisposition) {
        self.evac_disposition = disp;
    }
}

impl_contain_module!(GarrisonContain);

pub fn parse_garrison_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = GarrisonContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_garrison_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<GarrisonContainModuleData>()
        .expect("GarrisonContain data type mismatch")
        .clone();
    Box::new(GarrisonContain::new(typed))
}
