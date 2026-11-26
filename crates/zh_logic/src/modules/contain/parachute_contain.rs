//! `PKT-285`: `ParachuteContain` (`ParachuteContain.h`, `ParachuteContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-285_ParachuteContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.
//!
//! Headless art has no `PARA_*` bones. Drawable-only work (hiding the closed chute, the sway
//! instance matrices and the opening sound) is not modelled.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::combat::locomotor_set::LocomotorSetType;
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    impl_contain_module, kill_object, on_die, update_open_contain, ContainBehavior,
    OpenContainCore, OpenContainModuleData,
};
use crate::modules::contain::transport_contain::transport_slot_count;
use crate::modules::module::{BehaviorModule, CommandSourceType, ExitDoorType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::pathfinder::pathfind_cell::CellType;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{FindPositionOptions, SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_fabsf, Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, NameKeyType, Real, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

const NO_START_Z: Real = 1e10;
const ALTITUDE_DAMP_START: Real = 20.0;
const PATHFIND_CELL_SIZE: Real = 10.0;

/// INI template data matching `ParachuteContainModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct ParachuteContainModuleData {
    pub open: OpenContainModuleData,
    pub pitch_rate_max: Real,
    pub roll_rate_max: Real,
    pub low_altitude_damping: Real,
    /// Deploy the parachute once we have fallen this far.
    pub para_open_dist: Real,
    pub free_fall_damage_percent: Real,
    pub kill_when_landing_in_water_slop: Real,
    pub parachute_open_sound: String,
}

impl ParachuteContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            open: OpenContainModuleData::new(tag_key, module_name_key),
            pitch_rate_max: 0.0,
            roll_rate_max: 0.0,
            low_altitude_damping: 0.2,
            para_open_dist: 0.0,
            free_fall_damage_percent: 0.5,
            kill_when_landing_in_water_slop: 10.0,
            parachute_open_sound: String::new(),
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        let result = if is("PitchRateMax") {
            reader.parse_angular_velocity_real(&mut self.pitch_rate_max)
        } else if is("RollRateMax") {
            reader.parse_angular_velocity_real(&mut self.roll_rate_max)
        } else if is("LowAltitudeDamping") {
            reader.parse_real(&mut self.low_altitude_damping)
        } else if is("ParachuteOpenDist") {
            reader.parse_real(&mut self.para_open_dist)
        } else if is("KillWhenLandingInWaterSlop") {
            reader.parse_real(&mut self.kill_when_landing_in_water_slop)
        } else if is("FreeFallDamagePercent") {
            reader.parse_percent_to_real(&mut self.free_fall_damage_percent)
        } else if is("ParachuteOpenSound") {
            reader.parse_ascii_string(&mut self.parachute_open_sound)
        } else {
            return self.open.parse_field(field, reader, parse_ctx);
        };
        Some(result)
    }
}

impl ModuleData for ParachuteContainModuleData {
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

/// Runtime instance of `ParachuteContain` (`PKT-285`).
pub struct ParachuteContain {
    pub data: ParachuteContainModuleData,
    pub core: OpenContainCore,
    pub pitch: Real,
    pub roll: Real,
    pub pitch_rate: Real,
    pub roll_rate: Real,
    pub start_z: Real,
    pub is_landing_override_set: Bool,
    pub landing_override: Coord3D,
    pub rider_attach_bone: Coord3D,
    pub rider_sway_bone: Coord3D,
    pub para_attach_bone: Coord3D,
    pub para_sway_bone: Coord3D,
    pub rider_attach_offset: Coord3D,
    pub rider_sway_offset: Coord3D,
    pub para_attach_offset: Coord3D,
    pub para_sway_offset: Coord3D,
    pub need_to_update_rider_bones: Bool,
    pub need_to_update_para_bones: Bool,
    pub opened: Bool,
}

fn sub(a: &Coord3D, b: &Coord3D) -> Coord3D {
    Coord3D::new(a.x - b.x, a.y - b.y, a.z - b.z)
}

fn clear_and_set_condition(
    sim: &mut dyn SimApi,
    obj: ObjectSlot,
    clear_flag: ModelConditionFlagType,
    set_flag: ModelConditionFlagType,
) {
    if sim.object_has_drawable(obj) == FALSE {
        return;
    }
    let mut clear = ModelConditionFlags::new();
    clear.set(clear_flag as Int, TRUE as Int);
    let mut set = ModelConditionFlags::new();
    set.set(set_flag as Int, TRUE as Int);
    sim.clear_and_set_model_condition_flags(obj, &clear, &set);
}

fn position_and_matrix(sim: &dyn SimView, obj: ObjectSlot) -> Option<(Coord3D, Matrix3D)> {
    sim.object(obj)
        .map(|o| (*o.thing.get_position(), *o.thing.get_transform_matrix()))
}

impl ParachuteContain {
    pub fn new(data: ParachuteContainModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        let zero = Coord3D::default();
        Self {
            data,
            core,
            pitch: 0.0,
            roll: 0.0,
            pitch_rate: 0.0,
            roll_rate: 0.0,
            start_z: NO_START_Z,
            is_landing_override_set: FALSE,
            landing_override: zero,
            rider_attach_bone: zero,
            rider_sway_bone: zero,
            para_attach_bone: zero,
            para_sway_bone: zero,
            rider_attach_offset: zero,
            rider_sway_offset: zero,
            para_attach_offset: zero,
            para_sway_offset: zero,
            need_to_update_rider_bones: TRUE,
            need_to_update_para_bones: TRUE,
            opened: FALSE,
        }
    }

    fn front_rider(&self, sim: &dyn SimView) -> Option<ObjectSlot> {
        self.core
            .contain_list
            .first()
            .and_then(|&id| sim.find_object(id))
    }

    fn update_bone_positions(&mut self, sim: &dyn SimView, ctx: ModuleContext) {
        if self.need_to_update_para_bones != FALSE {
            self.need_to_update_para_bones = FALSE; // yes, even if not found
            if sim.object_has_drawable(ctx.owner_slot) != FALSE {
                self.para_sway_bone = Coord3D::default();
                self.para_attach_bone = Coord3D::default();
            }
        }
        if self.need_to_update_rider_bones != FALSE {
            self.need_to_update_rider_bones = FALSE; // yes, even if not found
            if let Some(rider) = self.front_rider(sim) {
                if sim.object_has_drawable(rider) != FALSE {
                    // No `PARA_MAN` bone: use the rider's height.
                    self.rider_attach_bone = Coord3D::default();
                    if let Some(o) = sim.object(rider) {
                        self.rider_attach_bone.z += o.geometry.get_max_height_above_position();
                    }
                }
            }
        }
    }

    fn update_offsets_from_bones(&mut self, sim: &dyn SimView, ctx: ModuleContext) {
        let Some((pos, mtx)) = position_and_matrix(sim, ctx.owner_slot) else {
            return;
        };
        self.para_sway_offset = sub(&Matrix3D::transform_vector(&mtx, &self.para_sway_bone), &pos);
        self.para_attach_offset =
            sub(&Matrix3D::transform_vector(&mtx, &self.para_attach_bone), &pos);

        let Some((rider_pos, rider_mtx)) = self
            .front_rider(sim)
            .and_then(|r| position_and_matrix(sim, r))
        else {
            return;
        };
        let attach = sub(
            &Matrix3D::transform_vector(&rider_mtx, &self.rider_attach_bone),
            &rider_pos,
        );
        self.rider_attach_offset = sub(&self.para_attach_offset, &attach);
        self.rider_sway_offset = sub(&self.para_sway_offset, &self.rider_attach_offset);
    }

    fn position_rider(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        self.update_bone_positions(sim, ctx);
        self.update_offsets_from_bones(sim, ctx);

        let Some((mut pos, angle)) = sim
            .object(ctx.owner_slot)
            .map(|o| (*o.thing.get_position(), o.thing.get_orientation()))
        else {
            return;
        };
        pos.x += self.rider_attach_offset.x;
        pos.y += self.rider_attach_offset.y;
        pos.z += self.rider_attach_offset.z;
        sim.set_object_position(rider, &pos);

        let altitude = sim.object_height_above_terrain(rider);
        if altitude < 0.0 {
            // Don't let him go below ground.
            pos.z -= altitude;
            sim.set_object_position(rider, &pos);
        }
        sim.set_object_orientation(rider, angle);
    }

    /// `positionContainedObjectsRelativeToContainer`.
    fn position_contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for id in self.core.contain_list.clone() {
            if let Some(rider) = sim.find_object(id) {
                self.position_rider(sim, ctx, rider);
            }
        }
    }

    /// Wake the rider's physics and let it fall (`DISABLED_HELD` alone does not).
    fn wake_falling_rider(sim: &mut dyn SimApi, rider: ObjectSlot, free_fall: bool) {
        if sim.has_physics(rider) == FALSE {
            return;
        }
        sim.set_physics_allow_to_fall(rider, TRUE);
        if free_fall {
            sim.physics_set_is_in_free_fall(rider, TRUE);
        }
        sim.physics_apply_force(rider, &Coord3D::default());
    }

    fn kill_self(sim: &mut dyn SimApi, ctx: ModuleContext) {
        kill_object(
            sim,
            ctx.owner_slot,
            DamageType::Unresistable,
            DeathType::Normal,
        );
    }

    /// `ParachuteContain::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let _ = update_open_contain(self, sim, ctx);

        let me = ctx.owner_slot;
        if sim
            .object(me)
            .is_none_or(|o| o.is_disabled_by_type(DisabledType::Held) != FALSE)
        {
            return UpdateSleepTime::NONE;
        }
        let Some(pos) = sim.object(me).map(|o| *o.thing.get_position()) else {
            return UpdateSleepTime::NONE;
        };
        let rider = self.front_rider(sim);

        if self.start_z == NO_START_Z {
            self.start_z = pos.z;
            let ground = sim.terrain().get_ground_height(pos.x, pos.y, None);
            if self.start_z - ground < 2.0 * self.data.para_open_dist {
                // Ejected too close to the ground: fudge so the chute has room to open.
                self.start_z = ground + 2.0 * self.data.para_open_dist;
            }
        }

        if self.opened == FALSE {
            if gm_fabsf(self.start_z - pos.z) >= self.data.para_open_dist {
                self.opened = TRUE;
                clear_and_set_condition(
                    sim,
                    me,
                    ModelConditionFlagType::Freefall,
                    ModelConditionFlagType::Parachuting,
                );
                self.need_to_update_para_bones = TRUE;
                if let Some(rider) = rider {
                    clear_and_set_condition(
                        sim,
                        rider,
                        ModelConditionFlagType::Freefall,
                        ModelConditionFlagType::Parachuting,
                    );
                    self.need_to_update_rider_bones = TRUE;
                }

                // Look for a good place to land.
                if sim.has_ai(me) != FALSE {
                    let mut target = pos;
                    if self.is_landing_override_set != FALSE {
                        target = self.landing_override;
                        if sim.ai_has_cur_locomotor(me) != FALSE {
                            sim.set_locomotor_ultra_accurate(me, TRUE);
                        }
                    } else {
                        let options = FindPositionOptions {
                            min_radius: 0.0,
                            max_radius: 100.0,
                            ..FindPositionOptions::default()
                        };
                        if let Some(found) = sim.find_position_around(&target, &options) {
                            target = found;
                        }
                    }
                    sim.ai_move_to_position(me, &target, CommandSourceType::FromAi);
                }
            } else if let Some(rider) = rider {
                clear_and_set_condition(
                    sim,
                    rider,
                    ModelConditionFlagType::Parachuting,
                    ModelConditionFlagType::Freefall,
                );
            }
        }

        // Unopened or empty chutes don't collide with anything.
        let no_collisions = self.opened == FALSE || self.core.contain_list_size == 0;
        for obj in [Some(me), rider].into_iter().flatten() {
            if no_collisions {
                sim.set_status(obj, ObjectStatusType::NoCollisions);
            } else {
                sim.clear_status(obj, ObjectStatusType::NoCollisions);
            }
        }

        let alive = sim
            .object(me)
            .is_some_and(|o| o.is_effectively_dead() == FALSE);
        if sim.has_ai(me) != FALSE && alive {
            sim.ai_choose_locomotor_set(
                me,
                if self.opened != FALSE {
                    LocomotorSetType::Normal
                } else {
                    LocomotorSetType::Freefall
                },
            );
            let locomotor = sim.object(me).and_then(|o| o.ai_locomotor.as_ref()).map(|l| {
                (
                    l.template.pitch_stiffness,
                    l.template.roll_stiffness,
                    l.template.pitch_damping,
                    l.template.roll_damping,
                )
            });
            if let Some((pitch_stiffness, roll_stiffness, pitch_damping, roll_damping)) = locomotor {
                // Damp the swaying near the ground so landings are (nearly) vertical.
                let mut altitude_damping: Real = 0.0;
                if let Some(rider) = self.front_rider(sim) {
                    if sim.object_height_above_terrain(rider) <= ALTITUDE_DAMP_START {
                        altitude_damping = self.data.low_altitude_damping;
                    }
                }
                if self.opened != FALSE {
                    let pitch_damping = pitch_damping + altitude_damping;
                    let roll_damping = roll_damping + altitude_damping;
                    // Spring / damper.
                    self.pitch_rate +=
                        (-pitch_stiffness * self.pitch) + (-pitch_damping * self.pitch_rate);
                    self.roll_rate +=
                        (-roll_stiffness * self.roll) + (-roll_damping * self.roll_rate);

                    self.pitch += self.pitch_rate;
                    self.roll += self.roll_rate;

                    if self.is_landing_override_set != FALSE {
                        // Only after opening, otherwise the freefall locomotor gets it.
                        sim.set_locomotor_close_enough_dist(me, 10.0);
                        sim.set_locomotor_close_enough_dist_3d(me, FALSE);
                    }
                }
                if sim.object_has_drawable(me) != FALSE {
                    self.update_bone_positions(sim, ctx);
                    self.update_offsets_from_bones(sim, ctx);
                }
                self.position_contained(sim, ctx);
            }
        }

        // Allow landing on bridges.
        let layer = sim.get_highest_layer_for_destination(&pos, FALSE);
        sim.set_object_layer(me, layer);
        if let Some(rider) = rider {
            sim.set_object_layer(rider, layer);
        }

        // Lost our passenger for whatever reason: die early.
        if self.core.contain_list_size == 0 {
            Self::kill_self(sim, ctx);
        }

        // The collide system doesn't always report falling into water.
        let mut water_z: Real = 0.0;
        if sim
            .object(me)
            .is_some_and(|o| o.is_effectively_dead() == FALSE && o.layer == PathfindLayerEnum::Ground)
            && sim.terrain().is_underwater(pos.x, pos.y, Some(&mut water_z), None) != FALSE
            && (pos.z - water_z) < self.data.kill_when_landing_in_water_slop
        {
            Self::kill_self(sim, ctx);
        }

        UpdateSleepTime::NONE
    }
}

impl ContainBehavior for ParachuteContain {
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
        FALSE
    }
    fn is_special_zero_slot(&self) -> Bool {
        TRUE
    }

    /// The constructor draws the initial sway rates and marks the owner as parachuting.
    fn construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let max = self.data.pitch_rate_max;
        self.pitch_rate = sim.rng_logic_real(-max, max);
        let max = self.data.roll_rate_max;
        self.roll_rate = sim.rng_logic_real(-max, max);
        sim.set_status(ctx.owner_slot, ObjectStatusType::Parachuting);
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;
        xfer.xfer_real(&mut self.pitch)?;
        xfer.xfer_real(&mut self.roll)?;
        xfer.xfer_real(&mut self.pitch_rate)?;
        xfer.xfer_real(&mut self.roll_rate)?;
        xfer.xfer_real(&mut self.start_z)?;
        xfer.xfer_bool(&mut self.is_landing_override_set)?;
        xfer.xfer_coord3d(&mut self.landing_override)?;
        xfer.xfer_coord3d(&mut self.rider_attach_bone)?;
        xfer.xfer_coord3d(&mut self.rider_sway_bone)?;
        xfer.xfer_coord3d(&mut self.para_attach_bone)?;
        xfer.xfer_coord3d(&mut self.para_sway_bone)?;
        xfer.xfer_coord3d(&mut self.rider_attach_offset)?;
        xfer.xfer_coord3d(&mut self.rider_sway_offset)?;
        xfer.xfer_coord3d(&mut self.para_attach_offset)?;
        xfer.xfer_coord3d(&mut self.para_sway_offset)?;
        xfer.xfer_bool(&mut self.need_to_update_rider_bones)?;
        xfer.xfer_bool(&mut self.need_to_update_para_bones)?;
        xfer.xfer_bool(&mut self.opened)
    }

    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        _check_capacity: Bool,
    ) -> Bool {
        if self
            .core
            .is_valid_container_for(&self.data.open, sim, ctx, rider)
            == FALSE
        {
            return FALSE;
        }
        // Infantry may always be paradropped, even if otherwise not transportable.
        if transport_slot_count(sim, rider) == 0
            && sim.object_is_kind_of(rider, KindOfType::Infantry) == FALSE
            && sim.object_is_kind_of(rider, KindOfType::Parachutable) == FALSE
        {
            return FALSE;
        }
        // Only one item at a time.
        Bool::from(self.core.contain_list_size == 0)
    }

    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        sim.set_disabled(rider, DisabledType::Held);
        sim.set_status(rider, ObjectStatusType::Parachuting);
        clear_and_set_condition(
            sim,
            rider,
            ModelConditionFlagType::Parachuting,
            ModelConditionFlagType::Freefall,
        );
        self.need_to_update_rider_bones = TRUE;
        self.position_rider(sim, ctx, rider);
    }

    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        sim.clear_disabled(rider, DisabledType::Held);
        sim.clear_status(rider, ObjectStatusType::Parachuting);
        // The chute is ephemeral now; don't let it bounce off the soldier.
        sim.set_status(ctx.owner_slot, ObjectStatusType::NoCollisions);

        self.position_rider(sim, ctx, rider);

        if sim.object_has_drawable(rider) != FALSE {
            sim.clear_model_condition_state(rider, ModelConditionFlagType::Freefall);
            sim.clear_model_condition_state(rider, ModelConditionFlagType::Parachuting);
        }
        self.need_to_update_rider_bones = TRUE;

        Self::wake_falling_rider(sim, rider, false);

        if sim.has_ai(rider) != FALSE {
            let team = sim.object(rider).map(|o| o.team_id);
            if team.is_some_and(|t| sim.player_is_skirmish_ai(t) != FALSE) {
                sim.ai_hunt(rider, CommandSourceType::FromAi);
            } else {
                let mut has_rally_point = false;
                // The building that produced the transport that dropped the rider.
                let producer = sim
                    .object(rider)
                    .and_then(|o| sim.find_object(o.producer_id))
                    .and_then(|transport| sim.object(transport))
                    .and_then(|o| sim.find_object(o.producer_id));
                if let Some(building) = producer {
                    if sim.exit_use_spawn_rally_point(building) != FALSE {
                        sim.exit_object_via_door(building, rider, ExitDoorType::Door1);
                        has_rally_point = true;
                    }
                }
                if !has_rally_point {
                    sim.ai_idle(rider, CommandSourceType::FromAi);
                }
            }
        }

        // Landing in water kills.
        let Some((pos, layer)) = sim
            .object(rider)
            .map(|o| (*o.thing.get_position(), o.layer))
        else {
            return;
        };
        let mut water_z: Real = 0.0;
        let mut terrain_z: Real = 0.0;
        if sim
            .terrain()
            .is_underwater(pos.x, pos.y, Some(&mut water_z), Some(&mut terrain_z))
            != FALSE
            && pos.z <= water_z + self.data.kill_when_landing_in_water_slop
            && layer == PathfindLayerEnum::Ground
        {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Water;
            dmg.input.death_type = DeathType::Flooded;
            dmg.input.amount = HUGE_DAMAGE_AMOUNT;
            sim.attempt_damage(rider, &mut dmg);
        }

        // So does impassable ground, or ending up off the map.
        let Some((pos, layer, off_map)) = sim.object(rider).map(|o| {
            (
                *o.thing.get_position(),
                o.layer,
                (o.private_status & PRIVATE_STATUS_OFF_MAP) != 0,
            )
        }) else {
            return;
        };
        let cell_x = (pos.x / PATHFIND_CELL_SIZE) as Int;
        let cell_y = (pos.y / PATHFIND_CELL_SIZE) as Int;
        let cell = sim.pathfind_cell_type(layer, cell_x, cell_y);
        if off_map
            || matches!(cell, CellType::Cliff | CellType::Water | CellType::Impassable)
        {
            kill_object(sim, rider, DamageType::Unresistable, DeathType::Normal);
        }
    }

    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }

    /// Only toggles drawable visibility; riders are not redeployed.
    fn react_to_transform(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    /// `other == None` means we hit the ground.
    fn collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        if other.is_some() {
            return;
        }
        // Inside a container (eg a transport plane): ignore.
        if sim
            .object(ctx.owner_slot)
            .is_none_or(|o| o.contained_by.is_some())
        {
            return;
        }
        self.remove_all(sim, ctx, FALSE);
        // Kill rather than destroy, so the chute's slow death triggers.
        Self::kill_self(sim, ctx);
    }

    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        // Killed while airborne: the passenger falls.
        if sim.object_is_significantly_above_terrain(ctx.owner_slot) != FALSE {
            if let Some(rider) = self.front_rider(sim) {
                self.remove_all(sim, ctx, FALSE);
                let percent = self.data.free_fall_damage_percent;
                if percent > 0.0 {
                    let mut extra = DamageInfo::default();
                    extra.input.damage_type = DamageType::Falling;
                    extra.input.death_type = DeathType::Splatted;
                    extra.input.source_id = dmg.input.source_id;
                    extra.input.amount = sim.object_max_health(rider) * percent;
                    sim.attempt_damage(rider, &mut extra);
                }
                Self::wake_falling_rider(sim, rider, true);
            }
        }
        on_die(self, sim, ctx, dmg);
    }

    fn set_override_dest(&mut self, pos: &Coord3D) {
        self.landing_override = *pos;
        self.is_landing_override_set = TRUE;
    }
}

impl_contain_module!(ParachuteContain);

pub fn parse_parachute_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = ParachuteContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_parachute_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<ParachuteContainModuleData>()
        .expect("ParachuteContain data type mismatch")
        .clone();
    Box::new(ParachuteContain::new(typed))
}
