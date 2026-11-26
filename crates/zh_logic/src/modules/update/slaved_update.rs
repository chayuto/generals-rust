//! `PKT-365`: `SlavedUpdateModuleData` and `SlavedUpdate`
//! (`SlavedUpdate.h`, `SlavedUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-365_SlavedUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt296_365_spawn_slaved.rs`.
//!
//! The welding particle system and its sound are client work: they are reported
//! to `ClientSink::on_welding_sparks`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::combat::locomotor_set::LocomotorSetType;
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, CommandSourceType, ModuleData, SlavedUpdateFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::geometry::GeometryInfo;
use zh_common::math::{div_safe_f32, gm_cosf, gm_sinf, gm_sqrtf, Coord3D, PI};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `SLAVED_UPDATE_RATE`: a quarter of a second.
const SLAVED_UPDATE_RATE: Int = 7;
const STRAY_MULTIPLIER: Real = 2.0;
const CLOSE_ENOUGH_SQR: Real = 15.0 * 15.0;

pub const REPAIRSTATE_NONE: Int = 0;
pub const REPAIRSTATE_UNPACKING: Int = 1;
pub const REPAIRSTATE_PACKING: Int = 2;
pub const REPAIRSTATE_READY: Int = 3;
pub const REPAIRSTATE_EXTENDING: Int = 4;
pub const REPAIRSTATE_RETRACTING: Int = 5;
pub const REPAIRSTATE_WELDING: Int = 6;

/// INI template data matching `SlavedUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SlavedUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub guard_max_range: Int,
    pub guard_wander_range: Int,
    pub attack_range: Int,
    pub attack_wander_range: Int,
    pub scout_range: Int,
    pub scout_wander_range: Int,
    pub dist_to_target_to_grant_range_bonus: Int,
    pub repair_range: Int,
    pub repair_min_altitude: Real,
    pub repair_max_altitude: Real,
    pub repair_rate_per_second: Real,
    pub repair_when_health_below_percentage: Int,
    pub min_ready_frames: Int,
    pub max_ready_frames: Int,
    pub min_weld_frames: Int,
    pub max_weld_frames: Int,
    pub welding_sys_name: String,
    pub welding_fx_bone: String,
    pub stay_on_same_layer_as_master: Bool,
}

impl SlavedUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            guard_max_range: 0,
            guard_wander_range: 0,
            attack_range: 0,
            attack_wander_range: 0,
            scout_range: 0,
            scout_wander_range: 0,
            dist_to_target_to_grant_range_bonus: 0,
            // The reference leaves this one uninitialised.
            repair_range: 0,
            repair_min_altitude: 0.0,
            repair_max_altitude: 0.0,
            repair_rate_per_second: 0.0,
            repair_when_health_below_percentage: 0,
            min_ready_frames: 0,
            max_ready_frames: 0,
            min_weld_frames: 0,
            max_weld_frames: 0,
            welding_sys_name: String::new(),
            welding_fx_bone: String::new(),
            stay_on_same_layer_as_master: FALSE,
        }
    }
}

impl ModuleData for SlavedUpdateModuleData {
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
        FacetMask(0x001 | FacetMask::SLAVED_UPDATE.0)
    }
}

/// `Int * Int` squared, as `sqr(int)` compared against a `Real`.
fn sqr_int(v: Int) -> Real {
    v.wrapping_mul(v) as Real
}

/// `GameLogicRandomValue(0, 2*PI)`: the bounds are `Int`, so a whole number of radians.
fn random_direction(sim: &mut dyn SimApi) -> Real {
    sim.rng_logic_int(0, (2.0 * PI) as Int) as Real
}

fn position(sim: &dyn SimView, obj: ObjectSlot) -> Coord3D {
    sim.object(obj)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn geometry(sim: &dyn SimView, obj: ObjectSlot) -> GeometryInfo {
    sim.object(obj)
        .map_or_else(GeometryInfo::default, |o| o.geometry.clone())
}

/// `getDistanceSquared(obj, .., FROM_BOUNDINGSPHERE_2D)`: the bounding circle radii of
/// the objects involved come off the distance. A point has no radius.
fn boundary_distance_squared_2d(
    sim: &dyn SimView,
    obj: ObjectSlot,
    other_pos: &Coord3D,
    other: Option<ObjectSlot>,
) -> Real {
    let here = position(sim, obj);
    let dx = other_pos.x - here.x;
    let dy = other_pos.y - here.y;
    let distance_squared = dx * dx + dy * dy;
    let total = geometry(sim, obj).get_bounding_circle_radius()
        + other.map_or(0.0, |o| geometry(sim, o).get_bounding_circle_radius());
    if total > 0.0 {
        let shrunk = gm_sqrtf(distance_squared) - total;
        if shrunk <= 0.0 {
            0.0
        } else {
            shrunk * shrunk
        }
    } else {
        distance_squared
    }
}

fn distance_squared_3d(a: &Coord3D, b: &Coord3D) -> Real {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    dx * dx + dy * dy + dz * dz
}

/// Runtime instance of `SlavedUpdate`.
pub struct SlavedUpdate {
    pub data: SlavedUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub slaver: ObjectId,
    /// An offset from the master while guarding; an absolute position once a repair
    /// spot has been chosen.
    pub guard_point_offset: Coord3D,
    pub frames_to_wait: Int,
    pub repair_state: Int,
    pub repairing: Bool,
}

impl SlavedUpdate {
    pub fn new(data: SlavedUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            slaver: ObjectId::INVALID,
            guard_point_offset: Coord3D::default(),
            frames_to_wait: 0,
            repair_state: REPAIRSTATE_NONE,
            repairing: FALSE,
        }
    }

    fn has_body(sim: &dyn SimView, obj: ObjectSlot) -> bool {
        sim.object(obj).is_some_and(|o| o.body_slot.is_some())
    }

    fn master(&self, sim: &dyn SimView) -> Option<ObjectSlot> {
        sim.find_object(self.slaver)
    }

    /// Picks a point `range` away in a random direction and stores it as the offset.
    fn roll_offset(&mut self, sim: &mut dyn SimApi, range: Int) {
        let direction = random_direction(sim);
        self.guard_point_offset = Coord3D::default();
        self.guard_point_offset.x += range as Real * gm_cosf(direction);
        self.guard_point_offset.y += range as Real * gm_sinf(direction);
    }

    /// The wander step shared by the attack, scout and guard logic.
    fn wander(&mut self, sim: &mut dyn SimApi, range: Int, destination: &mut Coord3D) {
        self.roll_offset(sim, range);
        destination.x += self.guard_point_offset.x;
        destination.y += self.guard_point_offset.y;
        self.guard_point_offset.z =
            sim.terrain()
                .get_ground_height(destination.x, destination.y, None);
    }

    /// `destination`, pulled back to within `range` of the master if `owner` is farther.
    fn leashed(
        sim: &dyn SimView,
        owner: ObjectSlot,
        master: ObjectSlot,
        destination: &Coord3D,
        range: Int,
    ) -> (Coord3D, Real) {
        let distance = boundary_distance_squared_2d(sim, owner, destination, None);
        if distance > sqr_int(range) {
            let mut vector = *destination;
            let home = position(sim, master);
            vector.sub(&home);
            vector.normalize();
            vector.scale(range as Real);
            let mut point = home;
            point.add(&vector);
            (point, distance)
        } else {
            (*destination, distance)
        }
    }

    fn do_attack_logic(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, target: ObjectSlot) {
        let Some(master) = self.master(sim) else {
            return;
        };
        let target_pos = position(sim, target);
        let (mut attack_position, distance) =
            Self::leashed(sim, owner, master, &target_pos, self.data.attack_range);
        if self.data.attack_wander_range != 0 {
            self.wander(sim, self.data.attack_wander_range, &mut attack_position);
        }
        sim.ai_move_to_position(owner, &attack_position, CommandSourceType::FromAi);
        if distance < sqr_int(self.data.dist_to_target_to_grant_range_bonus) {
            sim.set_weapon_bonus_condition(master, WeaponBonusConditionType::DroneSpotting);
        }
    }

    fn do_scout_logic(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, destination: &Coord3D) {
        let Some(master) = self.master(sim) else {
            return;
        };
        let (mut scout_position, _) =
            Self::leashed(sim, owner, master, destination, self.data.scout_range);
        if self.data.scout_wander_range != 0 {
            self.wander(sim, self.data.scout_wander_range, &mut scout_position);
        }
        sim.ai_move_to_position(owner, &scout_position, CommandSourceType::FromAi);
    }

    fn do_guard_logic(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, pinned: &mut Coord3D) {
        // `GuardWanderRange` only switches the wandering on; its radius is `GuardMaxRange`.
        if self.data.guard_wander_range != 0 {
            self.wander(sim, self.data.guard_max_range, pinned);
        }
        sim.ai_move_to_position(owner, pinned, CommandSourceType::FromAi);
    }

    fn do_repair_logic(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let Some(master) = self.master(sim) else {
            return;
        };
        if sim.has_ai(owner) == FALSE {
            return;
        }
        let master_pos = position(sim, master);
        let distance = boundary_distance_squared_2d(sim, owner, &master_pos, Some(master));
        let close_enough = distance < 12.0 * 12.0;
        if close_enough {
            match self.repair_state {
                REPAIRSTATE_NONE => self.set_repair_state(sim, owner, REPAIRSTATE_READY),
                REPAIRSTATE_READY | REPAIRSTATE_EXTENDING => {
                    if self.frames_to_wait == 0 {
                        self.set_repair_state(sim, owner, REPAIRSTATE_WELDING);
                    }
                }
                REPAIRSTATE_UNPACKING | REPAIRSTATE_WELDING | REPAIRSTATE_RETRACTING => {
                    if self.frames_to_wait == 0 {
                        self.set_repair_state(sim, owner, REPAIRSTATE_READY);
                    }
                }
                _ => {}
            }
        } else {
            self.repairing = FALSE;
            let reach = geometry(sim, master).get_bounding_sphere_radius() * 2.0;
            sim.set_locomotor_use_precise_z_pos(owner, Bool::from(distance < reach * reach));
            let mut pos = position(sim, master);
            let altitude =
                sim.rng_logic_real(self.data.repair_min_altitude, self.data.repair_max_altitude);
            pos.z += altitude;
            sim.ai_move_to_position(owner, &pos, CommandSourceType::FromAi);
            if self.frames_to_wait == 0 {
                self.set_repair_state(sim, owner, REPAIRSTATE_READY);
            }
        }

        if close_enough && self.repairing != FALSE && Self::has_body(sim, master) {
            let mut healing = DamageInfo::default();
            healing.input.amount = self.data.repair_rate_per_second / 30.0;
            healing.input.damage_type = DamageType::Healing;
            healing.input.death_type = DeathType::NoneType;
            sim.attempt_healing(master, &mut healing);
        }
    }

    fn end_repair(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if self.repair_state != REPAIRSTATE_NONE {
            self.repair_state = REPAIRSTATE_NONE;
            self.frames_to_wait = SLAVED_UPDATE_RATE;
            self.repairing = FALSE;
            Self::set_repair_model_condition_states(sim, owner, ModelConditionFlagType::Packing);
        }
        if sim.has_ai(owner) != FALSE {
            sim.ai_choose_locomotor_set(owner, LocomotorSetType::Normal);
            if sim.ai_has_cur_locomotor(owner) != FALSE {
                sim.set_locomotor_ultra_accurate(owner, FALSE);
                sim.set_locomotor_use_precise_z_pos(owner, FALSE);
            }
        }
    }

    fn set_repair_model_condition_states(
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        flag: ModelConditionFlagType,
    ) {
        for clear in [
            ModelConditionFlagType::Packing,
            ModelConditionFlagType::Unpacking,
            ModelConditionFlagType::FiringB,
            ModelConditionFlagType::FiringC,
            ModelConditionFlagType::BetweenFiringShotsB,
            ModelConditionFlagType::BetweenFiringShotsC,
            ModelConditionFlagType::ReloadingB,
            ModelConditionFlagType::ReloadingC,
        ] {
            sim.clear_model_condition_state(owner, clear);
        }
        sim.set_model_condition_state(owner, flag);
    }

    /// `setRepairState`. The module only ever asks for `READY` and `WELDING`; both may
    /// pass through an intermediate state first.
    fn set_repair_state(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, state: Int) {
        if state == self.repair_state {
            return;
        }
        match state {
            REPAIRSTATE_READY => match self.repair_state {
                REPAIRSTATE_NONE => {
                    Self::set_repair_model_condition_states(
                        sim,
                        owner,
                        ModelConditionFlagType::Unpacking,
                    );
                    self.repair_state = REPAIRSTATE_UNPACKING;
                    self.frames_to_wait = 15;
                }
                REPAIRSTATE_WELDING => {
                    self.repair_state = REPAIRSTATE_RETRACTING;
                    self.frames_to_wait = 5;
                    Self::set_repair_model_condition_states(
                        sim,
                        owner,
                        ModelConditionFlagType::FiringC,
                    );
                    self.move_to_new_repair_spot(sim, owner);
                }
                _ => {
                    self.repair_state = REPAIRSTATE_READY;
                    self.frames_to_wait =
                        sim.rng_logic_int(self.data.min_ready_frames, self.data.max_ready_frames);
                }
            },
            REPAIRSTATE_WELDING => {
                if self.repair_state == REPAIRSTATE_READY {
                    self.repair_state = REPAIRSTATE_EXTENDING;
                    self.frames_to_wait = 5;
                    Self::set_repair_model_condition_states(
                        sim,
                        owner,
                        ModelConditionFlagType::FiringB,
                    );
                    return;
                }
                self.repair_state = REPAIRSTATE_WELDING;
                self.frames_to_wait =
                    sim.rng_logic_int(self.data.min_weld_frames, self.data.max_weld_frames);
                if !self.data.welding_sys_name.is_empty() {
                    if let Some(id) = sim.object(owner).map(|o| o.id) {
                        let frames = self.frames_to_wait;
                        sim.client().on_welding_sparks(
                            id,
                            &self.data.welding_sys_name,
                            &self.data.welding_fx_bone,
                            frames,
                        );
                    }
                }
                self.repairing = TRUE;
            }
            _ => {}
        }
    }

    fn move_to_new_repair_spot(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if self.data.repair_range == 0 {
            return;
        }
        let Some(master) = self.master(sim) else {
            return;
        };
        let direction = random_direction(sim);
        self.guard_point_offset = position(sim, master);
        self.guard_point_offset.x += self.data.repair_range as Real * gm_cosf(direction);
        self.guard_point_offset.y += self.data.repair_range as Real * gm_sinf(direction);
        self.guard_point_offset.z = sim.terrain().get_ground_height(
            self.guard_point_offset.x,
            self.guard_point_offset.y,
            None,
        );
        let altitude =
            sim.rng_logic_real(self.data.repair_min_altitude, self.data.repair_max_altitude);
        self.guard_point_offset.z += altitude;
        if sim.has_ai(owner) != FALSE {
            sim.ai_choose_locomotor_set(owner, LocomotorSetType::Panic);
            sim.set_locomotor_ultra_accurate(owner, TRUE);
            let spot = self.guard_point_offset;
            sim.ai_move_to_position(owner, &spot, CommandSourceType::FromAi);
            sim.set_locomotor_use_precise_z_pos(owner, TRUE);
        }
    }

    fn stop_slaved_effects(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        self.slaver = ObjectId::INVALID;
        self.guard_point_offset = Coord3D::default();
        sim.clear_status(owner, ObjectStatusType::Unselectable);
        sim.clear_disabled(owner, DisabledType::Held);
    }
}

impl SlavedUpdateFacet for SlavedUpdate {
    fn get_slaver_id(&self) -> ObjectId {
        self.slaver
    }

    fn on_enslave(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, slaver: Option<ObjectSlot>) {
        let owner = ctx.owner_slot;
        let Some((slaver, stealthed)) = slaver.and_then(|s| {
            sim.object(s).map(|o| {
                (
                    o.id,
                    o.status.test(ObjectStatusType::Stealthed.as_i32()) != FALSE,
                )
            })
        }) else {
            return;
        };
        self.slaver = slaver;
        self.roll_offset(sim, self.data.guard_max_range);
        sim.set_status(owner, ObjectStatusType::Unselectable);
        if stealthed {
            sim.receive_stealth_grant(owner, TRUE, 0);
        }
    }

    fn on_slaver_die(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _dmg: Option<&DamageInfo>,
    ) {
        self.stop_slaved_effects(sim, ctx.owner_slot);
    }

    fn on_slaver_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        sim.ai_go_prone(ctx.owner_slot, dmg, CommandSourceType::FromAi);
    }
}

impl UpdateFacet for SlavedUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let next = UpdateSleepTime::NONE;
        if self.frames_to_wait > 0 {
            self.frames_to_wait -= 1;
        }
        if self.repair_state == REPAIRSTATE_NONE {
            if self.frames_to_wait > 0 {
                return next;
            }
            self.frames_to_wait = SLAVED_UPDATE_RATE;
        }

        if self.slaver == ObjectId::INVALID
            || sim.has_ai(owner) == FALSE
            || sim.ai_has_cur_locomotor(owner) == FALSE
        {
            return next;
        }

        let master = self.master(sim).filter(|&m| {
            sim.object(m).is_some_and(|o| {
                o.is_effectively_dead() == FALSE
                    && o.is_disabled_by_type(DisabledType::Unmanned) == FALSE
            })
        });
        let Some(master) = master else {
            self.stop_slaved_effects(sim, owner);
            sim.set_disabled(owner, DisabledType::Unmanned);
            sim.ai_idle(owner, CommandSourceType::FromAi);
            return next;
        };
        let (Some(master_team), Some(my_team)) = (
            sim.object(master).map(|o| o.team_id),
            sim.object(owner).map(|o| o.team_id),
        ) else {
            return next;
        };
        // `Team::getRelationship`, which knows nothing of undetected defectors.
        if sim.team_relationship_to_player(master_team, sim.player_index(my_team))
            != Relationship::Allies
        {
            sim.defect_object(owner, master_team, 0, TRUE);
        }

        if self.data.stay_on_same_layer_as_master != FALSE {
            if let Some(layer) = sim.object(master).map(|o| o.layer) {
                sim.set_object_layer(owner, layer);
            }
        }

        sim.clear_weapon_bonus_condition(master, WeaponBonusConditionType::DroneSpotting);

        let has_master_ai = sim.has_ai(master) != FALSE;
        let target = if has_master_ai {
            sim.ai_current_victim(master)
        } else {
            None
        };

        let mut health_percentage: Int = 100;
        if self.data.repair_rate_per_second > 0.0 && Self::has_body(sim, master) {
            let health = sim.object_health(master);
            let max_health = sim.object_max_health(master);
            health_percentage = (div_safe_f32(health, max_health) * 100.0) as Int;
        }

        if health_percentage <= self.data.repair_when_health_below_percentage {
            self.do_repair_logic(sim, owner);
            return next;
        }

        if self.data.attack_range != 0 {
            if let Some(target) = target {
                self.end_repair(sim, owner);
                self.do_attack_logic(sim, owner, target);
                return next;
            }
        }

        if self.data.scout_range != 0 {
            // The reference dereferences a master without an AI here.
            if let Some(destination) = sim.ai_path_destination(master) {
                let distance = boundary_distance_squared_2d(sim, master, &destination, None);
                let half = self.data.guard_max_range as Real * 0.5;
                if distance > half * half {
                    self.end_repair(sim, owner);
                    self.do_scout_logic(sim, owner, &destination);
                    return next;
                }
            }
        }

        if health_percentage < 100 {
            self.do_repair_logic(sim, owner);
            return next;
        }

        let mut pinned = position(sim, master);
        pinned.x += self.guard_point_offset.x;
        pinned.y += self.guard_point_offset.y;
        self.guard_point_offset.z = sim.terrain().get_ground_height(pinned.x, pinned.y, None);

        if self.data.guard_max_range != 0 {
            let here = position(sim, owner);
            let stray = STRAY_MULTIPLIER * self.data.guard_max_range as Real;
            if (sim.ai_is_idle(owner) != FALSE
                && distance_squared_3d(&here, &pinned) > CLOSE_ENOUGH_SQR)
                || distance_squared_3d(&here, &position(sim, master)) > stray * stray
            {
                self.end_repair(sim, owner);
                self.do_guard_logic(sim, owner, &mut pinned);
            }
        }
        next
    }
}

impl BehaviorModule for SlavedUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        // The condition lives on the drawable, which `ThingFactory::newObject` has not
        // made yet: a new slave is never put into the packing pose.
        if self.data.repair_rate_per_second > 0.0
            && sim.object_has_drawable(ctx.owner_slot) != FALSE
        {
            sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Packing);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_object_id(&mut self.slaver)?;
        xfer.xfer_coord3d(&mut self.guard_point_offset)?;
        xfer.xfer_int(&mut self.frames_to_wait)?;
        let mut state = self.repair_state.to_le_bytes();
        xfer.xfer_user(&mut state)?;
        self.repair_state = Int::from_le_bytes(state);
        xfer.xfer_bool(&mut self.repairing)
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

    fn as_slaved_update(&self) -> Option<&dyn SlavedUpdateFacet> {
        Some(self)
    }

    fn as_slaved_update_mut(&mut self) -> Option<&mut dyn SlavedUpdateFacet> {
        Some(self)
    }
}

pub fn parse_slaved_update_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SlavedUpdateModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        let duration = |reader: &mut IniReader<'_>, store: &mut Int| {
            let mut frames: UnsignedInt = 0;
            reader.parse_duration_unsigned_int(&mut frames)?;
            *store = frames as Int;
            Ok(())
        };
        let d = &mut data;
        let int_field = [
            ("GuardMaxRange", &mut d.guard_max_range),
            ("GuardWanderRange", &mut d.guard_wander_range),
            ("AttackRange", &mut d.attack_range),
            ("AttackWanderRange", &mut d.attack_wander_range),
            ("ScoutRange", &mut d.scout_range),
            ("ScoutWanderRange", &mut d.scout_wander_range),
            ("RepairRange", &mut d.repair_range),
            (
                "DistToTargetToGrantRangeBonus",
                &mut d.dist_to_target_to_grant_range_bonus,
            ),
            (
                "RepairWhenBelowHealth%",
                &mut d.repair_when_health_below_percentage,
            ),
        ]
        .into_iter()
        .find(|(name, _)| field.eq_ignore_ascii_case(name));
        if let Some((_, store)) = int_field {
            return reader.parse_int(store);
        }
        if field.eq_ignore_ascii_case("RepairMinAltitude") {
            reader.parse_real(&mut data.repair_min_altitude)
        } else if field.eq_ignore_ascii_case("RepairMaxAltitude") {
            reader.parse_real(&mut data.repair_max_altitude)
        } else if field.eq_ignore_ascii_case("RepairRatePerSecond") {
            reader.parse_real(&mut data.repair_rate_per_second)
        } else if field.eq_ignore_ascii_case("RepairMinReadyTime") {
            duration(reader, &mut data.min_ready_frames)
        } else if field.eq_ignore_ascii_case("RepairMaxReadyTime") {
            duration(reader, &mut data.max_ready_frames)
        } else if field.eq_ignore_ascii_case("RepairMinWeldTime") {
            duration(reader, &mut data.min_weld_frames)
        } else if field.eq_ignore_ascii_case("RepairMaxWeldTime") {
            duration(reader, &mut data.max_weld_frames)
        } else if field.eq_ignore_ascii_case("RepairWeldingSys") {
            reader.parse_ascii_string(&mut data.welding_sys_name)
        } else if field.eq_ignore_ascii_case("RepairWeldingFXBone") {
            reader.parse_ascii_string(&mut data.welding_fx_bone)
        } else if field.eq_ignore_ascii_case("StayOnSameLayerAsMaster") {
            reader.parse_bool(&mut data.stay_on_same_layer_as_master)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_slaved_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let data = data
        .as_any()
        .downcast_ref::<SlavedUpdateModuleData>()
        .expect("SlavedUpdateModuleData")
        .clone();
    Box::new(SlavedUpdate::new(data))
}
