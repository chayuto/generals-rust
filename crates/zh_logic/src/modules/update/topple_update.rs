//! `PKT-383`: `ToppleUpdateModuleData` and `ToppleUpdate`
//! (`ToppleUpdate.h`, `ToppleUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-383_ToppleUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt383_377_305_topple.rs`.
//!
//! Client-only calls of the reference are not modelled: `SwayClientUpdate::stopSway`
//! and `Drawable::setShadowsEnabled`. An unknown `StumpName` makes no stump, where
//! the reference throws from `ThingFactory::newObject`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::combat::turret_ai::normalize_angle_cpp;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{BehaviorModule, CollideFacet, ModuleData, ToppleFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{
    div_safe_f32, gm_atan2f, gm_cosf, gm_fabsf, gm_sinf, real_to_int_floor, Coord3D, Matrix3D, PI,
};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

pub const TOPPLE_OPTIONS_NONE: UnsignedInt = 0x0000_0000;
pub const TOPPLE_OPTIONS_NO_BOUNCE: UnsignedInt = 0x0000_0001;
pub const TOPPLE_OPTIONS_NO_FX: UnsignedInt = 0x0000_0002;

pub const TOPPLE_UPRIGHT: Int = 0;
pub const TOPPLE_FALLING: Int = 1;
pub const TOPPLE_DOWN: Int = 2;

const ANGULAR_LIMIT: Real = PI / 2.0 - PI / 64.0;
const VELOCITY_BOUNCE_LIMIT: Real = 0.01;
const VELOCITY_BOUNCE_SOUND_LIMIT: Real = 0.03;

/// INI template data matching `ToppleUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct ToppleUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub topple_fx: Option<String>,
    pub bounce_fx: Option<String>,
    pub stump_name: String,
    pub initial_velocity_percent: Real,
    pub initial_accel_percent: Real,
    pub bounce_velocity_percent: Real,
    pub kill_when_toppled: Bool,
    pub kill_when_start_toppled: Bool,
    pub kill_stump_when_toppled: Bool,
    pub topple_left_or_right_only: Bool,
    pub reorient_toppled_rubble: Bool,
}

impl ToppleUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            topple_fx: None,
            bounce_fx: None,
            stump_name: String::new(),
            initial_velocity_percent: 0.2,
            initial_accel_percent: 0.01,
            bounce_velocity_percent: 0.3,
            kill_when_toppled: TRUE,
            kill_when_start_toppled: FALSE,
            kill_stump_when_toppled: FALSE,
            topple_left_or_right_only: FALSE,
            reorient_toppled_rubble: FALSE,
        }
    }
}

impl ModuleData for ToppleUpdateModuleData {
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
        FacetMask(0x011)
    }
}

/// `angleClosestTo`.
fn angle_closest_to(a1: Real, a2: Real, desired: Real) -> Real {
    let a1 = normalize_angle_cpp(a1);
    let a2 = normalize_angle_cpp(a2);
    if gm_fabsf(normalize_angle_cpp(desired - a1)) < gm_fabsf(normalize_angle_cpp(desired - a2)) {
        a1
    } else {
        a2
    }
}

/// `deathByToppling`.
fn death_by_toppling(sim: &mut dyn SimApi, victim: ObjectSlot) {
    let mut dmg = DamageInfo::default();
    dmg.input.damage_type = DamageType::Unresistable;
    dmg.input.death_type = DeathType::Toppled;
    dmg.input.source_id = ObjectId::INVALID;
    dmg.input.amount = HUGE_DAMAGE_AMOUNT;
    sim.attempt_damage(victim, &mut dmg);
}

fn transform_of(sim: &dyn SimView, slot: ObjectSlot) -> Option<Matrix3D> {
    sim.object(slot).map(|o| *o.thing.get_transform_matrix())
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn orientation_of(sim: &dyn SimView, slot: ObjectSlot) -> Real {
    sim.object(slot).map_or(0.0, |o| o.thing.get_orientation())
}

/// Runtime instance of `ToppleUpdate`.
pub struct ToppleUpdate {
    pub data: ToppleUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub angular_velocity: Real,
    pub angular_acceleration: Real,
    pub topple_direction: Coord3D,
    pub topple_state: Int,
    pub angular_accumulation: Real,
    pub angle_delta_x: Real,
    pub num_angle_delta_x: Int,
    pub do_bounce_fx: Bool,
    pub options: UnsignedInt,
    pub stump_id: ObjectId,
}

impl ToppleUpdate {
    pub fn new(data: ToppleUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            angular_velocity: 0.0,
            angular_acceleration: 0.0,
            topple_direction: Coord3D::default(),
            topple_state: TOPPLE_UPRIGHT,
            angular_accumulation: 0.0,
            angle_delta_x: 0.0,
            num_angle_delta_x: 0,
            do_bounce_fx: FALSE,
            options: TOPPLE_OPTIONS_NONE,
            stump_id: ObjectId::INVALID,
        }
    }

    fn make_stump(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let Some(tmpl) = sim
            .rules()
            .thing_factory
            .find_template_id(&self.data.stump_name)
        else {
            return;
        };
        // `newObject(ttn, nullptr)`: the neutral player's default team, at the origin.
        let team = sim.player_default_team(sim.neutral_team_id());
        let stump = sim.spawn_object(tmpl, team, Coord3D::default(), 0.0);
        let pos = position_of(sim, owner);
        sim.set_object_position(stump, &pos);
        let angle = orientation_of(sim, owner);
        sim.set_object_orientation(stump, angle);
        self.stump_id = sim.object(stump).map_or(ObjectId::INVALID, |o| o.id);
        let burned = sim.object_has_drawable(owner) != FALSE
            && sim.object(owner).is_some_and(|o| {
                o.model_condition
                    .test(ModelConditionFlagType::Burned as Int)
                    != FALSE
            });
        if burned && sim.object_has_drawable(stump) != FALSE {
            sim.set_model_condition_state(stump, ModelConditionFlagType::Burned);
        }
    }
}

impl ToppleFacet for ToppleUpdate {
    fn is_able_to_be_toppled(&self) -> Bool {
        Bool::from(self.topple_state == TOPPLE_UPRIGHT)
    }

    /// `applyTopplingForce`.
    fn apply_toppling_force(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        direction: &Coord3D,
        speed: Real,
        options: UnsignedInt,
    ) {
        let owner = ctx.owner_slot;
        if sim
            .object(owner)
            .is_none_or(|o| o.is_effectively_dead() != FALSE)
        {
            return;
        }
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        if self.data.kill_when_start_toppled != FALSE {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
            kill_object(sim, owner, DamageType::Unresistable, DeathType::Normal);
            return;
        }
        self.topple_direction = *direction;
        self.topple_direction.normalize();
        sim.adjust_topple_direction(owner, &mut self.topple_direction);
        self.angular_velocity = speed * self.data.initial_velocity_percent;
        self.angular_acceleration = speed * self.data.initial_accel_percent;
        self.topple_state = TOPPLE_FALLING;
        self.options = options;
        let cur_angle_x = normalize_angle_cpp(orientation_of(sim, owner));
        let mut topple_angle =
            normalize_angle_cpp(gm_atan2f(self.topple_direction.y, self.topple_direction.x));
        if self.data.topple_left_or_right_only != FALSE {
            topple_angle =
                angle_closest_to(cur_angle_x + PI / 2.0, cur_angle_x - PI / 2.0, topple_angle);
            self.topple_direction.x = gm_cosf(topple_angle);
            self.topple_direction.y = gm_sinf(topple_angle);
            sim.pathfinder_remove_object(owner);
        }
        let desired_angle_x = angle_closest_to(
            topple_angle + PI / 2.0,
            topple_angle - PI / 2.0,
            cur_angle_x,
        );
        self.num_angle_delta_x =
            real_to_int_floor(div_safe_f32(ANGULAR_LIMIT, self.angular_velocity * 2.0));
        if self.num_angle_delta_x < 1 {
            self.num_angle_delta_x = 1;
        }
        self.angle_delta_x = (desired_angle_x - cur_angle_x) / self.num_angle_delta_x as Real;
        sim.set_model_condition_state(owner, ModelConditionFlagType::Toppled);
        if let Some(fx) = &self.data.topple_fx {
            let pos = position_of(sim, owner);
            sim.trigger_fx_list(fx, &pos);
        }
        if !self.data.stump_name.is_empty() {
            self.make_stump(sim, owner);
        }
    }
}

impl UpdateFacet for ToppleUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.topple_state == TOPPLE_UPRIGHT || self.topple_state == TOPPLE_DOWN {
            return UpdateSleepTime::FOREVER;
        }
        let owner = ctx.owner_slot;
        if self.num_angle_delta_x != 0 {
            if let Some(mut xfrm) = transform_of(sim, owner) {
                xfrm.in_place_pre_rotate_z(self.angle_delta_x);
                sim.set_object_transform_matrix(owner, &xfrm);
            }
            self.num_angle_delta_x = self.num_angle_delta_x.wrapping_sub(1);
        }
        let mut cur_vel_to_use = self.angular_velocity;
        if self.angular_accumulation + cur_vel_to_use > ANGULAR_LIMIT {
            cur_vel_to_use = ANGULAR_LIMIT - self.angular_accumulation;
        }
        if let Some(mut xfrm) = transform_of(sim, owner) {
            xfrm.in_place_pre_rotate_x(-cur_vel_to_use * self.topple_direction.y);
            xfrm.in_place_pre_rotate_y(cur_vel_to_use * self.topple_direction.x);
            sim.set_object_transform_matrix(owner, &xfrm);
        }
        self.angular_accumulation += cur_vel_to_use;
        if self.angular_accumulation >= ANGULAR_LIMIT && self.angular_velocity > 0.0 {
            self.angular_velocity *= -self.data.bounce_velocity_percent;
            if self.options & TOPPLE_OPTIONS_NO_BOUNCE != 0
                || gm_fabsf(self.angular_velocity) < VELOCITY_BOUNCE_LIMIT
            {
                self.angular_velocity = 0.0;
                self.topple_state = TOPPLE_DOWN;
                if self.data.kill_when_toppled != FALSE {
                    death_by_toppling(sim, owner);
                    if self.data.reorient_toppled_rubble != FALSE {
                        if let Some((mtx, height)) = sim.object(owner).map(|o| {
                            (
                                *o.thing.get_transform_matrix(),
                                o.geometry.get_max_height_above_position(),
                            )
                        }) {
                            let pos =
                                Matrix3D::transform_vector(&mtx, &Coord3D::new(0.0, 0.0, height));
                            sim.set_object_position(owner, &pos);
                            let angle = orientation_of(sim, owner);
                            sim.set_object_orientation(owner, angle);
                        }
                    }
                }
                if self.data.kill_stump_when_toppled != FALSE {
                    if let Some(stump) = sim.find_object(self.stump_id) {
                        death_by_toppling(sim, stump);
                    }
                }
            } else if gm_fabsf(self.angular_velocity) >= VELOCITY_BOUNCE_SOUND_LIMIT
                && self.options & TOPPLE_OPTIONS_NO_FX == 0
            {
                if let Some(fx) = &self.data.bounce_fx {
                    let pos = position_of(sim, owner);
                    sim.trigger_fx_list(fx, &pos);
                }
            }
        } else {
            self.angular_velocity += self.angular_acceleration;
        }
        UpdateSleepTime::NONE
    }
}

impl CollideFacet for ToppleUpdate {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        let Some(other) = other else {
            return;
        };
        if sim.object_crusher_level(other) <= 1 {
            return;
        }
        let mut topple_vector = position_of(sim, ctx.owner_slot);
        let other_pos = position_of(sim, other);
        topple_vector.x -= other_pos.x;
        topple_vector.y -= other_pos.y;
        topple_vector.z = 0.0;
        // Zero without a `PhysicsBehavior`, as in the reference.
        let speed = sim.physics_velocity(other).length();
        // `Object::topple`.
        if self.is_able_to_be_toppled() != FALSE {
            self.apply_toppling_force(sim, ctx, &topple_vector, speed, TOPPLE_OPTIONS_NONE);
        }
    }
}

impl BehaviorModule for ToppleUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_real(&mut self.angular_velocity)?;
        xfer.xfer_real(&mut self.angular_acceleration)?;
        xfer.xfer_coord3d(&mut self.topple_direction)?;
        let mut state = self.topple_state.to_le_bytes();
        xfer.xfer_user(&mut state)?;
        self.topple_state = Int::from_le_bytes(state);
        xfer.xfer_real(&mut self.angular_accumulation)?;
        xfer.xfer_real(&mut self.angle_delta_x)?;
        xfer.xfer_int(&mut self.num_angle_delta_x)?;
        xfer.xfer_bool(&mut self.do_bounce_fx)?;
        xfer.xfer_unsigned_int(&mut self.options)?;
        xfer.xfer_object_id(&mut self.stump_id)
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

    fn as_topple_mut(&mut self) -> Option<&mut dyn ToppleFacet> {
        Some(self)
    }
}

/// `INI::parseFXList` into an optional name (`None` is a null list).
pub(crate) fn parse_fx_name(
    r: &mut IniReader<'_>,
    parse_ctx: &ModuleParseContext<'_>,
) -> Result<Option<String>, IniError> {
    let token = r.next_token(zh_common::ini::SEPS_DEFAULT)?;
    Ok(
        crate::modules::behavior::behavior_module::resolve_optional_asset_name(
            token,
            parse_ctx.fx_list_names,
        ),
    )
}

pub fn parse_topple_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = ToppleUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("ToppleFX") {
            data.topple_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("BounceFX") {
            data.bounce_fx = parse_fx_name(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("StumpName") {
            r.parse_ascii_string(&mut data.stump_name)
        } else if field.eq_ignore_ascii_case("KillWhenStartToppling") {
            r.parse_bool(&mut data.kill_when_start_toppled)
        } else if field.eq_ignore_ascii_case("KillWhenFinishedToppling") {
            r.parse_bool(&mut data.kill_when_toppled)
        } else if field.eq_ignore_ascii_case("KillStumpWhenToppled") {
            r.parse_bool(&mut data.kill_stump_when_toppled)
        } else if field.eq_ignore_ascii_case("ToppleLeftOrRightOnly") {
            r.parse_bool(&mut data.topple_left_or_right_only)
        } else if field.eq_ignore_ascii_case("ReorientToppledRubble") {
            r.parse_bool(&mut data.reorient_toppled_rubble)
        } else if field.eq_ignore_ascii_case("InitialVelocityPercent") {
            r.parse_percent_to_real(&mut data.initial_velocity_percent)
        } else if field.eq_ignore_ascii_case("InitialAccelPercent") {
            r.parse_percent_to_real(&mut data.initial_accel_percent)
        } else if field.eq_ignore_ascii_case("BounceVelocityPercent") {
            r.parse_percent_to_real(&mut data.bounce_velocity_percent)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_topple_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<ToppleUpdateModuleData>()
        .expect("ToppleUpdate data type mismatch")
        .clone();
    Box::new(ToppleUpdate::new(typed))
}
