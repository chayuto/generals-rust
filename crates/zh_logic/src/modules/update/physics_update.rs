//! `PKT-348`: `PhysicsBehaviorModuleData` and `PhysicsBehavior`
//! (`PhysicsUpdate.h`, `PhysicsUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-348_PhysicsUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt348_physics.rs`.
//!
//! The members of the C++ module live in [`PhysicsState`] on the owner's
//! `ObjectRecord`, as `Object::m_physics` reaches them. Other code calls into the
//! physics while the module is executing (a body or die module reacting to falling
//! damage, for one), which a module payload that has been taken out of the arena
//! could not answer. The functions here are the C++ member functions; each does
//! nothing for an object without physics.
//!
//! Not ported: the bounce sound (`setBounceSound`, `doBounceSound`; client audio only),
//! `TheScriptEngine->transferObjectName` and the AI's side of `processCollision`.

use crate::object::weapon_fire;
use crate::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::combat::locomotor_set::LocomotorSetType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{BehaviorModule, CollideFacet, ModuleData, PhysicsFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType, DisabledType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimView, PRIVATE_STATUS_CAPTURED, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use std::cell::Cell;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_atan2f, gm_fabsf, gm_sinf, gm_sqrtf, Coord3D, Matrix3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const STICK_TO_GROUND: Int = 0x0001;
pub const ALLOW_BOUNCE: Int = 0x0002;
pub const APPLY_FRICTION2D_WHEN_AIRBORNE: Int = 0x0004;
pub const UPDATE_EVER_RUN: Int = 0x0008;
pub const WAS_AIRBORNE_LAST_FRAME: Int = 0x0010;
pub const ALLOW_COLLIDE_FORCE: Int = 0x0020;
pub const ALLOW_TO_FALL: Int = 0x0040;
pub const HAS_PITCHROLLYAW: Int = 0x0080;
pub const IMMUNE_TO_FALLING_DAMAGE: Int = 0x0100;
pub const IS_IN_FREEFALL: Int = 0x0200;
pub const IS_IN_UPDATE: Int = 0x0400;
pub const IS_STUNNED: Int = 0x0800;

pub const TURN_NEGATIVE: Int = -1;
pub const TURN_NONE: Int = 0;
pub const TURN_POSITIVE: Int = 1;

const DEFAULT_LATERAL_FRICTION: Real = 0.15;
const MIN_AERO_FRICTION: Real = 0.00;
const MIN_NON_AERO_FRICTION: Real = 0.01;
const MAX_FRICTION: Real = 0.99;
const STUN_RELIEF_EPSILON: Real = 0.5;
const LOGICFRAMES_PER_SECOND: Int = 30;
/// `static_cast<float>(LOGICFRAMES_PER_SECOND) / 3`.
const MOTIVE_FRAMES: UnsignedInt = 10;
const INVALID_VEL_MAG: Real = -1.0;
/// `PI` of `BaseMacros.h`.
#[allow(clippy::approx_constant, clippy::excessive_precision)]
const PI: Real = 3.14159265359;
const DEFAULT_CRASH_BUILDING_WEAPON: &str = "VehicleCrashesIntoBuildingWeapon";
const DEFAULT_CRASH_NON_BUILDING_WEAPON: &str = "VehicleCrashesIntoNonBuildingWeapon";
/// `LOCOMOTORSURFACE_WATER` / `LOCOMOTORSURFACE_CLIFF`.
const SURFACE_WATER: Int = 1 << 1;
const SURFACE_CLIFF: Int = 1 << 2;

/// `heightToSpeed`.
pub fn height_to_speed(gravity: Real, height: Real) -> Real {
    gm_sqrtf(gm_fabsf(2.0 * gravity * height))
}

/// INI template data matching `PhysicsBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct PhysicsBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mass: Real,
    pub shock_resistance: Real,
    pub shock_max_yaw: Real,
    pub shock_max_pitch: Real,
    pub shock_max_roll: Real,
    pub forward_friction: Real,
    pub lateral_friction: Real,
    pub z_friction: Real,
    pub aerodynamic_friction: Real,
    pub center_of_mass_offset: Real,
    pub kill_when_resting_on_ground: Bool,
    pub allow_bouncing: Bool,
    pub allow_collide_force: Bool,
    /// `MinFallHeightForDamage`. The reference stores `heightToSpeed` of it while
    /// parsing; the port takes the speed from the gravity in force at construction.
    pub min_fall_height_for_damage: Real,
    pub fall_height_damage_factor: Real,
    pub pitch_roll_yaw_factor: Real,
    /// Weapon names; `None` for `None`. A name the weapon store lacks fires nothing.
    pub crash_into_building_weapon: Option<String>,
    pub crash_into_non_building_weapon: Option<String>,
}

impl PhysicsBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            mass: 1.0,
            shock_resistance: 0.0,
            shock_max_yaw: 0.05,
            shock_max_pitch: 0.025,
            shock_max_roll: 0.025,
            forward_friction: 0.15,
            lateral_friction: DEFAULT_LATERAL_FRICTION,
            z_friction: 0.8,
            aerodynamic_friction: 0.0,
            center_of_mass_offset: 0.0,
            kill_when_resting_on_ground: FALSE,
            allow_bouncing: FALSE,
            allow_collide_force: TRUE,
            min_fall_height_for_damage: 40.0,
            fall_height_damage_factor: 1.0,
            pitch_roll_yaw_factor: 2.0,
            crash_into_building_weapon: Some(DEFAULT_CRASH_BUILDING_WEAPON.to_owned()),
            crash_into_non_building_weapon: Some(DEFAULT_CRASH_NON_BUILDING_WEAPON.to_owned()),
        }
    }
}

impl ModuleData for PhysicsBehaviorModuleData {
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
        FacetMask(0x011 | FacetMask::PHYSICS_UPDATE.0)
    }

    fn update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Physics
    }
}

/// The members of a `PhysicsBehavior`.
#[derive(Debug, Clone)]
pub struct PhysicsState {
    pub data: Box<PhysicsBehaviorModuleData>,
    pub mod_slot: ModuleSlot,
    pub yaw_rate: Real,
    pub roll_rate: Real,
    pub pitch_rate: Real,
    pub accel: Coord3D,
    pub prev_accel: Coord3D,
    pub vel: Coord3D,
    pub turning: Int,
    pub ignore_collisions_with: ObjectId,
    pub flags: Int,
    pub mass: Real,
    pub current_overlap: ObjectId,
    pub previous_overlap: ObjectId,
    /// Not saved.
    pub last_collidee: ObjectId,
    pub motive_force_expires: UnsignedInt,
    pub extra_bounciness: Real,
    pub extra_friction: Real,
    /// `m_velMag` is `mutable`: reading the magnitude caches it, and the cache is saved.
    pub vel_mag: Cell<Real>,
    /// `m_pui != nullptr`.
    pub has_projectile_update: Bool,
    /// The reference never assigns `m_originalAllowBounce`; its memory pool zeroes it.
    pub original_allow_bounce: Bool,
    /// The module data's `m_minFallSpeedForDamage`.
    pub min_fall_speed_for_damage: Real,
}

impl PhysicsState {
    pub fn new(data: PhysicsBehaviorModuleData, mod_slot: ModuleSlot, gravity: Real) -> Self {
        let mut flags = 0;
        if data.allow_bouncing != FALSE {
            flags |= ALLOW_BOUNCE;
        }
        if data.allow_collide_force != FALSE {
            flags |= ALLOW_COLLIDE_FORCE;
        }
        Self {
            mod_slot,
            yaw_rate: 0.0,
            roll_rate: 0.0,
            pitch_rate: 0.0,
            accel: Coord3D::default(),
            prev_accel: Coord3D::default(),
            vel: Coord3D::default(),
            turning: TURN_NONE,
            ignore_collisions_with: ObjectId::INVALID,
            flags,
            mass: data.mass,
            current_overlap: ObjectId::INVALID,
            previous_overlap: ObjectId::INVALID,
            last_collidee: ObjectId::INVALID,
            motive_force_expires: 0,
            extra_bounciness: 0.0,
            extra_friction: 0.0,
            vel_mag: Cell::new(0.0),
            has_projectile_update: FALSE,
            original_allow_bounce: FALSE,
            min_fall_speed_for_damage: height_to_speed(gravity, data.min_fall_height_for_damage),
            data: Box::new(data),
        }
    }

    #[inline]
    pub fn flag(&self, f: Int) -> bool {
        (self.flags & f) != 0
    }

    #[inline]
    pub fn set_flag(&mut self, f: Int, set: Bool) {
        if set != FALSE {
            self.flags |= f;
        } else {
            self.flags &= !f;
        }
    }

    fn update_ypr_flag(&mut self) {
        let any = self.pitch_rate != 0.0 || self.roll_rate != 0.0 || self.yaw_rate != 0.0;
        self.set_flag(HAS_PITCHROLLYAW, Bool::from(any));
    }

    pub fn set_pitch_rate(&mut self, pitch: Real) {
        self.pitch_rate = pitch;
        self.update_ypr_flag();
    }

    pub fn set_roll_rate(&mut self, roll: Real) {
        self.roll_rate = roll;
        self.update_ypr_flag();
    }

    pub fn set_yaw_rate(&mut self, yaw: Real) {
        self.yaw_rate = yaw;
        self.update_ypr_flag();
    }

    pub fn apply_ypr_damping(&mut self, factor: Real) {
        self.pitch_rate *= factor;
        self.roll_rate *= factor;
        self.yaw_rate *= factor;
        self.update_ypr_flag();
    }

    fn clamped_friction(&self, base: Real, min: Real) -> Real {
        let mut f = base + self.extra_friction;
        if f < min {
            f = min;
        }
        if f > MAX_FRICTION {
            f = MAX_FRICTION;
        }
        f
    }

    pub fn aerodynamic_friction(&self) -> Real {
        self.clamped_friction(self.data.aerodynamic_friction, MIN_AERO_FRICTION)
    }

    pub fn forward_friction(&self) -> Real {
        self.clamped_friction(self.data.forward_friction, MIN_NON_AERO_FRICTION)
    }

    pub fn lateral_friction(&self) -> Real {
        self.clamped_friction(self.data.lateral_friction, MIN_NON_AERO_FRICTION)
    }

    pub fn z_friction(&self) -> Real {
        self.clamped_friction(self.data.z_friction, MIN_NON_AERO_FRICTION)
    }

    /// `getVelocityMagnitude`.
    pub fn velocity_magnitude(&self) -> Real {
        if self.vel_mag.get() == INVALID_VEL_MAG {
            let v = self.vel;
            self.vel_mag
                .set(gm_sqrtf(v.x * v.x + v.y * v.y + v.z * v.z));
        }
        self.vel_mag.get()
    }

    fn invalidate_velocity_magnitude(&mut self) {
        self.vel_mag.set(INVALID_VEL_MAG);
    }

    /// `addVelocityTo`. The cached magnitude is left as it is.
    pub fn add_velocity(&mut self, vel: &Coord3D) {
        self.vel.add(vel);
    }

    pub fn scrub_velocity_z(&mut self, desired: Real) {
        if gm_fabsf(desired) < 0.001 {
            self.vel.z = 0.0;
        } else if (desired < 0.0 && self.vel.z < desired) || (desired > 0.0 && self.vel.z > desired)
        {
            self.vel.z = desired;
        }
        self.invalidate_velocity_magnitude();
    }

    pub fn scrub_velocity_2d(&mut self, mut desired: Real) {
        if desired < 0.001 {
            self.vel.x = 0.0;
            self.vel.y = 0.0;
        } else {
            let cur = gm_sqrtf(self.vel.x * self.vel.x + self.vel.y * self.vel.y);
            if desired > cur {
                return;
            }
            desired /= cur;
            self.vel.x *= desired;
            self.vel.y *= desired;
        }
        self.invalidate_velocity_magnitude();
    }

    /// `PhysicsBehavior::xfer` from the first member on.
    fn xfer_members(&mut self, xfer: &mut dyn Xfer, version: u8) -> Result<(), XferError> {
        xfer.xfer_real(&mut self.yaw_rate)?;
        xfer.xfer_real(&mut self.roll_rate)?;
        xfer.xfer_real(&mut self.pitch_rate)?;
        xfer.xfer_coord3d(&mut self.accel)?;
        xfer.xfer_coord3d(&mut self.prev_accel)?;
        xfer.xfer_coord3d(&mut self.vel)?;
        if version < 2 {
            let mut tmp = Coord3D::default();
            xfer.xfer_coord3d(&mut tmp)?;
        }
        let mut turning = self.turning.to_le_bytes();
        xfer.xfer_user(&mut turning)?;
        self.turning = Int::from_le_bytes(turning);
        xfer.xfer_object_id(&mut self.ignore_collisions_with)?;
        xfer.xfer_int(&mut self.flags)?;
        xfer.xfer_real(&mut self.mass)?;
        xfer.xfer_object_id(&mut self.current_overlap)?;
        xfer.xfer_object_id(&mut self.previous_overlap)?;
        xfer.xfer_unsigned_int(&mut self.motive_force_expires)?;
        xfer.xfer_real(&mut self.extra_bounciness)?;
        xfer.xfer_real(&mut self.extra_friction)?;
        let mut vel_mag = self.vel_mag.get();
        xfer.xfer_real(&mut vel_mag)?;
        self.vel_mag.set(vel_mag);
        Ok(())
    }

    /// Takes over the members `xfer` loads; the rest keep their constructed values.
    fn take_saved_members(&mut self, from: &Self) {
        self.yaw_rate = from.yaw_rate;
        self.roll_rate = from.roll_rate;
        self.pitch_rate = from.pitch_rate;
        self.accel = from.accel;
        self.prev_accel = from.prev_accel;
        self.vel = from.vel;
        self.turning = from.turning;
        self.ignore_collisions_with = from.ignore_collisions_with;
        self.flags = from.flags;
        self.mass = from.mass;
        self.current_overlap = from.current_overlap;
        self.previous_overlap = from.previous_overlap;
        self.motive_force_expires = from.motive_force_expires;
        self.extra_bounciness = from.extra_bounciness;
        self.extra_friction = from.extra_friction;
        self.vel_mag.set(from.vel_mag.get());
    }
}

/// `Object::getPhysics`.
#[inline]
pub fn physics<'a>(sim: &'a dyn SimView, owner: ObjectSlot) -> Option<&'a PhysicsState> {
    sim.object(owner)?.physics.as_deref()
}

#[inline]
pub fn physics_mut<'a>(sim: &'a mut dyn SimApi, owner: ObjectSlot) -> Option<&'a mut PhysicsState> {
    sim.object_mut(owner)?.physics.as_deref_mut()
}

/// `getMass`: the own mass plus that of everything in the contain list.
pub fn get_mass(sim: &dyn SimView, owner: ObjectSlot) -> Real {
    let Some(state) = physics(sim, owner) else {
        return 0.0;
    };
    let mut mass = state.mass;
    // `OpenContain::getContainedItemsMass`.
    let mut contained = 0.0;
    for id in sim.contain_items_for_mass(owner) {
        if let Some(rider) = sim.find_object(id) {
            if physics(sim, rider).is_some() {
                contained += get_mass(sim, rider);
            }
        }
    }
    mass += contained;
    mass
}

/// `isMotive`.
pub fn is_motive(sim: &dyn SimView, owner: ObjectSlot) -> bool {
    physics(sim, owner).is_some_and(|p| p.motive_force_expires > sim.frame())
}

/// `setWakeFrame(UPDATE_SLEEP_NONE)` unless the module is inside its own update.
fn wake_unless_updating(sim: &mut dyn SimApi, owner: ObjectSlot) {
    if let Some(p) = physics(sim, owner) {
        if !p.flag(IS_IN_UPDATE) {
            let ms = p.mod_slot;
            sim.set_wake_frame(ms, UpdateSleepTime::NONE);
        }
    }
}

/// `applyForce`.
pub fn apply_force(sim: &mut dyn SimApi, owner: ObjectSlot, force: &Coord3D) {
    if physics(sim, owner).is_none() || force.x.is_nan() || force.y.is_nan() || force.z.is_nan() {
        return;
    }
    let mass = get_mass(sim, owner);
    let mut mod_force = *force;
    if is_motive(sim, owner) {
        let Some(dir) = sim
            .object(owner)
            .map(|o| o.thing.get_unit_direction_vector_2d())
        else {
            return;
        };
        let lateral_dot = force.x * (-dir.y) + force.y * dir.x;
        mod_force.x = lateral_dot * -dir.y;
        mod_force.y = lateral_dot * dir.x;
    }
    let mass_inv = 1.0 / mass;
    if let Some(p) = physics_mut(sim, owner) {
        p.accel.x += mod_force.x * mass_inv;
        p.accel.y += mod_force.y * mass_inv;
        p.accel.z += mod_force.z * mass_inv;
    }
    wake_unless_updating(sim, owner);
}

/// `applyShock`: the force less the shock resistance.
pub fn apply_shock(sim: &mut dyn SimApi, owner: ObjectSlot, force: &Coord3D) {
    let Some(resistance) = physics(sim, owner).map(|p| p.data.shock_resistance) else {
        return;
    };
    // `min(1.0f, max(0.0f, resistance))`.
    let low = if 0.0 < resistance { resistance } else { 0.0 };
    let clamped = if low < 1.0 { low } else { 1.0 };
    let mut resisted = *force;
    resisted.scale(1.0 - clamped);
    apply_force(sim, owner, &resisted);
}

/// `setStunned(true)`.
pub fn set_stunned(sim: &mut dyn SimApi, owner: ObjectSlot) {
    if let Some(p) = physics_mut(sim, owner) {
        p.set_flag(IS_STUNNED, TRUE);
    }
}

/// `applyRandomRotation`: three integer logic draws from `-1..=1`. It does not set
/// `HAS_PITCHROLLYAW`.
pub fn apply_random_rotation(sim: &mut dyn SimApi, owner: ObjectSlot) {
    match physics_mut(sim, owner) {
        Some(p) if !p.flag(STICK_TO_GROUND) => p.set_flag(ALLOW_BOUNCE, TRUE),
        _ => return,
    }
    let yaw = sim.rng_logic_int(-1, 1) as Real;
    if let Some(p) = physics_mut(sim, owner) {
        p.yaw_rate += p.data.shock_max_yaw * yaw;
    }
    let pitch = sim.rng_logic_int(-1, 1) as Real;
    if let Some(p) = physics_mut(sim, owner) {
        p.pitch_rate += p.data.shock_max_pitch * pitch;
    }
    let roll = sim.rng_logic_int(-1, 1) as Real;
    if let Some(p) = physics_mut(sim, owner) {
        p.roll_rate += p.data.shock_max_roll * roll;
    }
    wake_unless_updating(sim, owner);
}

/// `applyMotiveForce`: the whole force, then ten frames in which further forces keep
/// only their lateral part.
pub fn apply_motive_force(sim: &mut dyn SimApi, owner: ObjectSlot, force: &Coord3D) {
    match physics_mut(sim, owner) {
        Some(p) => p.motive_force_expires = 0,
        None => return,
    }
    apply_force(sim, owner, force);
    let expires = sim.frame().wrapping_add(MOTIVE_FRAMES);
    if let Some(p) = physics_mut(sim, owner) {
        p.motive_force_expires = expires;
    }
}

/// `resetDynamicPhysics`.
pub fn reset_dynamic_physics(sim: &mut dyn SimApi, owner: ObjectSlot) {
    let Some(p) = physics_mut(sim, owner) else {
        return;
    };
    p.accel = Coord3D::default();
    p.prev_accel = Coord3D::default();
    p.vel = Coord3D::default();
    p.vel_mag.set(0.0);
    p.turning = TURN_NONE;
    p.yaw_rate = 0.0;
    p.roll_rate = 0.0;
    p.pitch_rate = 0.0;
    p.set_flag(HAS_PITCHROLLYAW, FALSE);
    let ms = p.mod_slot;
    let sleep = calc_sleep_time(sim, owner);
    sim.set_wake_frame(ms, sleep);
}

/// `transferVelocityTo`.
pub fn transfer_velocity_to(sim: &mut dyn SimApi, from: ObjectSlot, to: ObjectSlot) {
    let Some(vel) = physics(sim, from).map(|p| p.vel) else {
        return;
    };
    if let Some(that) = physics_mut(sim, to) {
        that.vel.add(&vel);
        that.invalidate_velocity_magnitude();
    }
}

/// `getVelocityMagnitude`.
pub fn velocity_magnitude(sim: &dyn SimView, owner: ObjectSlot) -> Real {
    physics(sim, owner).map_or(0.0, PhysicsState::velocity_magnitude)
}

/// The retail forward speed: the length of the component products, signed by their sum.
fn legacy_forward_speed(sum_of_squares: Real, dot: Real) -> Real {
    let speed = gm_sqrtf(sum_of_squares);
    if dot >= 0.0 {
        speed
    } else {
        -speed
    }
}

/// `getForwardSpeed2D` (`useLegacyForwardSpeed()` is true in this build and in retail).
pub fn forward_speed_2d(sim: &dyn SimView, owner: ObjectSlot) -> Real {
    let (Some(p), Some(obj)) = (physics(sim, owner), sim.object(owner)) else {
        return 0.0;
    };
    let dir = obj.thing.get_unit_direction_vector_2d();
    let vx = p.vel.x * dir.x;
    let vy = p.vel.y * dir.y;
    legacy_forward_speed(vx * vx + vy * vy, vx + vy)
}

/// `getForwardSpeed3D`.
pub fn forward_speed_3d(sim: &dyn SimView, owner: ObjectSlot) -> Real {
    let (Some(p), Some(obj)) = (physics(sim, owner), sim.object(owner)) else {
        return 0.0;
    };
    let dir = obj.thing.get_transform_matrix().get_x_vector();
    let vx = p.vel.x * dir.x;
    let vy = p.vel.y * dir.y;
    let vz = p.vel.z * dir.z;
    legacy_forward_speed(vx * vx + vy * vy + vz * vz, vx + vy + vz)
}

/// `addOverlap`.
pub fn add_overlap(sim: &mut dyn SimApi, owner: ObjectSlot, other: ObjectSlot) {
    let Some(id) = sim.object(other).map(|o| o.id) else {
        return;
    };
    if let Some(p) = physics_mut(sim, owner) {
        p.current_overlap = id;
    }
}

/// `setAngles`: a transform at the current position with the given Euler angles.
pub fn set_angles(sim: &mut dyn SimApi, owner: ObjectSlot, yaw: Real, pitch: Real, roll: Real) {
    let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) else {
        return;
    };
    let mut xfrm = Matrix3D::identity();
    xfrm.translate(pos.x, pos.y, pos.z);
    xfrm.in_place_pre_rotate_x(-roll);
    xfrm.in_place_pre_rotate_y(pitch);
    xfrm.in_place_pre_rotate_z(yaw);
    sim.set_object_transform_matrix(owner, &xfrm);
}

fn is_very_small_3d(v: &Coord3D) -> bool {
    const THRESH: Real = 0.01;
    gm_fabsf(v.x) < THRESH && gm_fabsf(v.y) < THRESH && gm_fabsf(v.z) < THRESH
}

fn is_zero_3d(v: &Coord3D) -> bool {
    v.x == 0.0 && v.y == 0.0 && v.z == 0.0
}

/// `calcSleepTime`.
pub fn calc_sleep_time(sim: &dyn SimView, owner: ObjectSlot) -> UpdateSleepTime {
    let (Some(p), Some(obj)) = (physics(sim, owner), sim.object(owner)) else {
        return UpdateSleepTime::NONE;
    };
    if is_zero_3d(&p.vel)
        && is_zero_3d(&p.accel)
        && !p.flag(HAS_PITCHROLLYAW)
        && !is_motive(sim, owner)
        && obj.layer == crate::pathfinder::pathfind_constants::PathfindLayerEnum::Ground
        && sim.object_is_above_terrain(owner) == FALSE
        && p.current_overlap == ObjectId::INVALID
        && p.previous_overlap == ObjectId::INVALID
        && p.flag(UPDATE_EVER_RUN)
    {
        UpdateSleepTime::FOREVER
    } else {
        UpdateSleepTime::NONE
    }
}

fn has_status(sim: &dyn SimView, owner: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(owner)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn is_disabled(sim: &dyn SimView, owner: ObjectSlot, dt: DisabledType) -> bool {
    sim.object(owner)
        .is_some_and(|o| o.is_disabled_by_type(dt) != FALSE)
}

fn is_effectively_dead(sim: &dyn SimView, owner: ObjectSlot) -> bool {
    sim.object(owner)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn is_kind_of(sim: &dyn SimView, owner: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(owner, kind) != FALSE
}

/// `Object::setModelConditionState`, which only reaches an object with a drawable.
fn set_model(sim: &mut dyn SimApi, owner: ObjectSlot, flag: ModelConditionFlagType, on: bool) {
    if sim.object_has_drawable(owner) == FALSE {
        return;
    }
    if on {
        sim.set_model_condition_state(owner, flag);
    } else {
        sim.clear_model_condition_state(owner, flag);
    }
}

fn apply_gravitational_forces(sim: &mut dyn SimApi, owner: ObjectSlot) {
    let gravity = sim.rules().global_data.gravity;
    if let Some(p) = physics_mut(sim, owner) {
        p.accel.z += gravity;
    }
}

fn apply_frictional_forces(sim: &mut dyn SimApi, owner: ObjectSlot) {
    let deck_taxiing = has_status(sim, owner, ObjectStatusType::DeckHeightOffset)
        && sim.object(owner).is_some_and(|o| {
            o.ai_slot.is_some() && o.ai_locomotor_set == Some(LocomotorSetType::Taxiing)
        });
    let Some(airborne_friction) =
        physics(sim, owner).map(|p| p.flag(APPLY_FRICTION2D_WHEN_AIRBORNE))
    else {
        return;
    };
    if airborne_friction
        || sim.object_is_significantly_above_terrain(owner) == FALSE
        || deck_taxiing
    {
        if let Some(p) = physics_mut(sim, owner) {
            p.apply_ypr_damping(1.0 - DEFAULT_LATERAL_FRICTION);
        }
        let Some(vel) = physics(sim, owner).map(|p| p.vel) else {
            return;
        };
        if vel.x != 0.0 || vel.y != 0.0 {
            let Some(dir) = sim
                .object(owner)
                .map(|o| o.thing.get_unit_direction_vector_2d())
            else {
                return;
            };
            let mass = get_mass(sim, owner);
            let lateral_dot = vel.x * (-dir.y) + vel.y * dir.x;
            let lateral_x = lateral_dot * -dir.y;
            let lateral_y = lateral_dot * dir.x;
            let Some((lateral, forward)) =
                physics(sim, owner).map(|p| (p.lateral_friction(), p.forward_friction()))
            else {
                return;
            };
            let lf = mass * lateral;
            let mut accel = Coord3D::new(-(lf * lateral_x), -(lf * lateral_y), 0.0);
            if !is_motive(sim, owner) {
                let forward_dot = vel.x * dir.x + vel.y * dir.y;
                let forward_x = forward_dot * dir.x;
                let forward_y = forward_dot * dir.y;
                let ff = mass * forward;
                accel.x += -(ff * forward_x);
                accel.y += -(ff * forward_y);
            }
            apply_force(sim, owner, &accel);
        }
    } else if let Some(p) = physics_mut(sim, owner) {
        let aero = -p.aerodynamic_friction();
        p.accel.x += p.vel.x * aero;
        p.accel.y += p.vel.y * aero;
        p.accel.z += p.vel.z * aero;
        p.apply_ypr_damping(1.0 + aero);
    }
}

/// `handleBounce`: `Some(force)` when the ground throws the object back.
fn handle_bounce(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    old_z: Real,
    new_z: Real,
    ground_z: Real,
) -> Option<Coord3D> {
    if !(physics(sim, owner).is_some_and(|p| p.flag(ALLOW_BOUNCE)) && new_z <= ground_z) {
        return None;
    }
    const MIN_STIFF: Real = 0.01;
    const MAX_STIFF: Real = 0.99;
    let mut stiffness = sim.rules().global_data.ground_stiffness;
    if stiffness < MIN_STIFF {
        stiffness = MIN_STIFF;
    }
    if stiffness > MAX_STIFF {
        stiffness = MAX_STIFF;
    }
    let mut desired_accel_z = 0.0;
    let vz = physics(sim, owner)?.vel.z;
    if old_z > ground_z && vz < 0.0 {
        desired_accel_z = gm_fabsf(vz) * stiffness;
    }
    let bounce = Coord3D::new(0.0, 0.0, get_mass(sim, owner) * desired_accel_z);
    const DAMPING: Real = 0.7;
    physics_mut(sim, owner)?.apply_ypr_damping(DAMPING);
    if vz < 0.0 {
        let mtx = *sim.object(owner)?.thing.get_transform_matrix();
        let roll = if mtx.get_z_vector().z > 0.0 { 0.0 } else { PI };
        set_angles(sim, owner, mtx.get_z_rotation(), 0.0, roll);
    }
    if bounce.z > 0.0 {
        test_stunned_unit_for_destruction(sim, owner);
        Some(bounce)
    } else {
        let p = physics_mut(sim, owner)?;
        let original = p.original_allow_bounce;
        p.set_flag(ALLOW_BOUNCE, original);
        None
    }
}

/// `AIUpdateInterface::hasLocomotorForSurface` over the current locomotor set.
fn has_locomotor_for_surface(sim: &dyn SimView, owner: ObjectSlot, surface: Int) -> bool {
    let Some(obj) = sim.object(owner) else {
        return false;
    };
    let Some(set) = obj.ai_locomotor_set else {
        return false;
    };
    let rules = sim.rules();
    let Some(template) = obj
        .thing
        .template_override()
        .get_final(&rules.thing_factory)
    else {
        return false;
    };
    template.get_locomotor_set(set).iter().any(|name| {
        rules
            .locomotor_store
            .find_by_name(name)
            .is_some_and(|l| (l.surfaces & surface) != 0)
    })
}

fn test_stunned_unit_for_destruction(sim: &mut dyn SimApi, owner: ObjectSlot) {
    if !physics(sim, owner).is_some_and(|p| p.flag(IS_STUNNED)) {
        return;
    }
    let Some(obj) = sim.object(owner) else {
        return;
    };
    let pos = *obj.thing.get_position();
    let upside_down = obj.thing.get_transform_matrix().get_z_vector().z < 0.0;
    let off_map = (obj.private_status & PRIVATE_STATUS_OFF_MAP) != 0;
    let has_ai = obj.ai_slot.is_some();
    if upside_down || off_map {
        kill(sim, owner);
        return;
    }
    if !has_ai {
        return;
    }
    if sim.terrain().is_cliff_cell(pos.x, pos.y) != FALSE
        && !has_locomotor_for_surface(sim, owner, SURFACE_CLIFF)
    {
        kill(sim, owner);
        return;
    }
    if sim.terrain().is_underwater(pos.x, pos.y, None, None) != FALSE
        && !has_locomotor_for_surface(sim, owner, SURFACE_WATER)
    {
        kill(sim, owner);
    }
}

/// `Object::kill()` with its default damage and death types.
fn kill(sim: &mut dyn SimApi, victim: ObjectSlot) {
    kill_object(sim, victim, DamageType::Unresistable, DeathType::Normal);
}

/// `PhysicsBehavior::update`.
pub fn update(sim: &mut dyn SimApi, owner: ObjectSlot) -> UpdateSleepTime {
    if physics(sim, owner).is_none() {
        return UpdateSleepTime::NONE;
    }
    let airborne_at_start = sim.object_is_above_terrain(owner);
    let mut active_vel_z = 0.0;
    let mut bounce_force = None;
    if let Some(p) = physics_mut(sim, owner) {
        p.set_flag(IS_IN_UPDATE, TRUE);
        if !p.flag(UPDATE_EVER_RUN) {
            p.set_flag(WAS_AIRBORNE_LAST_FRAME, airborne_at_start);
        }
        p.prev_accel = p.accel;
    }
    if !is_disabled(sim, owner, DisabledType::Held) {
        move_one_frame(sim, owner, &mut active_vel_z, &mut bounce_force);
    }

    let mut allow_bounce = false;
    if let Some(p) = physics_mut(sim, owner) {
        p.accel = Coord3D::default();
        p.previous_overlap = p.current_overlap;
        p.current_overlap = ObjectId::INVALID;
        allow_bounce = p.flag(ALLOW_BOUNCE);
    }
    if let Some(force) = bounce_force {
        if allow_bounce {
            apply_force(sim, owner, &force);
        }
    }

    let airborne_at_end = sim.object_is_above_terrain(owner) != FALSE;
    if physics(sim, owner)
        .is_some_and(|p| p.flag(WAS_AIRBORNE_LAST_FRAME) && !p.flag(IMMUNE_TO_FALLING_DAMAGE))
        && !airborne_at_end
    {
        hit_ground(sim, owner, active_vel_z);
    }

    if !airborne_at_end {
        if let Some(p) = physics_mut(sim, owner) {
            p.set_flag(IS_IN_FREEFALL, FALSE);
        }
        if is_disabled(sim, owner, DisabledType::Freefall) {
            sim.clear_disabled(owner, DisabledType::Freefall);
        }
        set_model(sim, owner, ModelConditionFlagType::Freefall, false);
    }

    let rests = physics(sim, owner).is_some_and(|p| {
        p.data.kill_when_resting_on_ground != FALSE && !airborne_at_end && is_very_small_3d(&p.vel)
    });
    if rests
        && (!is_kind_of(sim, owner, KindOfType::Drone)
            || is_effectively_dead(sim, owner)
            || is_disabled(sim, owner, DisabledType::Unmanned))
    {
        kill(sim, owner);
    }

    if let Some(p) = physics_mut(sim, owner) {
        p.set_flag(UPDATE_EVER_RUN, TRUE);
        p.set_flag(WAS_AIRBORNE_LAST_FRAME, Bool::from(airborne_at_end));
        p.set_flag(IS_IN_UPDATE, FALSE);
    }
    calc_sleep_time(sim, owner)
}

/// The part of `update` an object that is not `DISABLED_HELD` runs.
fn move_one_frame(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    active_vel_z: &mut Real,
    bounce_force: &mut Option<Coord3D>,
) {
    let Some(mut mtx) = sim.object(owner).map(|o| *o.thing.get_transform_matrix()) else {
        return;
    };
    apply_gravitational_forces(sim, owner);
    apply_frictional_forces(sim, owner);

    let vel = {
        let Some(p) = physics_mut(sim, owner) else {
            return;
        };
        p.vel.x += p.accel.x;
        p.vel.y += p.accel.y;
        p.vel.z += p.accel.z;
        const THRESH: Real = 0.001;
        if gm_fabsf(p.vel.x) < THRESH {
            p.vel.x = 0.0;
        }
        if gm_fabsf(p.vel.y) < THRESH {
            p.vel.y = 0.0;
        }
        if gm_fabsf(p.vel.z) < THRESH {
            p.vel.z = 0.0;
        }
        p.invalidate_velocity_magnitude();
        p.vel
    };

    let old_pos_z = mtx.get_z_translation();
    if has_status(sim, owner, ObjectStatusType::Braking) {
        // The AI moves a braking object; only the height is integrated here.
        if !is_kind_of(sim, owner, KindOfType::Projectile) {
            mtx.adjust_z_translation(vel.z);
        }
    } else {
        mtx.adjust_x_translation(vel.x);
        mtx.adjust_y_translation(vel.y);
        mtx.adjust_z_translation(vel.z);
    }
    if mtx.get_x_translation().is_nan()
        || mtx.get_y_translation().is_nan()
        || mtx.get_z_translation().is_nan()
    {
        sim.destroy_object(owner);
    }

    if physics(sim, owner).is_some_and(|p| p.flag(IS_STUNNED))
        && ((gm_fabsf(vel.x) < STUN_RELIEF_EPSILON
            && gm_fabsf(vel.y) < STUN_RELIEF_EPSILON
            && gm_fabsf(vel.z) < STUN_RELIEF_EPSILON)
            || sim.object_is_significantly_above_terrain(owner) == FALSE)
    {
        if let Some(p) = physics_mut(sim, owner) {
            p.set_flag(IS_STUNNED, FALSE);
        }
        set_model(sim, owner, ModelConditionFlagType::Stunned, false);
    }

    if let Some(p) = physics(sim, owner).filter(|p| p.flag(HAS_PITCHROLLYAW)) {
        let factor = p.data.pitch_roll_yaw_factor;
        let yaw_rate = p.yaw_rate * factor;
        let mut pitch_rate = p.pitch_rate * factor;
        let roll_rate = p.roll_rate * factor;
        let offset = p.data.center_of_mass_offset;
        if offset != 0.0 {
            let x = mtx.get_x_vector();
            let xy = gm_sqrtf(x.x * x.x + x.y * x.y);
            let pitch_angle = gm_atan2f(x.z, xy);
            let remaining = if offset > 0.0 {
                (PI / 2.0) - pitch_angle
            } else {
                -(PI / 2.0) + pitch_angle
            };
            pitch_rate *= gm_sinf(remaining);
        }
        mtx.rotate_x(roll_rate);
        mtx.rotate_y(pitch_rate);
        mtx.rotate_z(yaw_rate);
    }

    let layer = sim.object(owner).map_or(
        crate::pathfinder::pathfind_constants::PathfindLayerEnum::Ground,
        |o| o.layer,
    );
    let mut ground_z =
        sim.get_layer_height(mtx.get_x_translation(), mtx.get_y_translation(), layer);
    if has_status(sim, owner, ObjectStatusType::DeckHeightOffset) {
        ground_z += sim.carrier_deck_height(owner);
    }

    *bounce_force = handle_bounce(sim, owner, old_pos_z, mtx.get_z_translation(), ground_z);

    *active_vel_z = physics(sim, owner).map_or(0.0, |p| p.vel.z);
    if mtx.get_z_translation() <= ground_z {
        let mut stunned = false;
        let dz = ground_z - mtx.get_z_translation();
        if let Some(p) = physics_mut(sim, owner) {
            p.vel.z += dz;
            if p.vel.z > 0.0 {
                p.vel.z = 0.0;
            }
            p.invalidate_velocity_magnitude();
            p.set_flag(ALLOW_TO_FALL, FALSE);
            stunned = p.flag(IS_STUNNED);
        }
        mtx.set_z_translation(ground_z);
        if stunned {
            set_model(sim, owner, ModelConditionFlagType::StunnedFlailing, false);
            set_model(sim, owner, ModelConditionFlagType::Stunned, true);
        }
    } else if mtx.get_z_translation() > ground_z {
        let Some((free_fall, stick)) = physics(sim, owner).map(|p| {
            (
                p.flag(IS_IN_FREEFALL),
                p.flag(STICK_TO_GROUND) && !p.flag(ALLOW_TO_FALL),
            )
        }) else {
            return;
        };
        if free_fall {
            sim.set_disabled(owner, DisabledType::Freefall);
            set_model(sim, owner, ModelConditionFlagType::Freefall, true);
        } else if stick {
            mtx.set_z_translation(ground_z);
        }
    }

    if bounce_force.is_some() {
        let Some(yaw) = sim
            .object(owner)
            .map(|o| o.thing.get_transform_matrix().get_z_rotation())
        else {
            return;
        };
        set_angles(sim, owner, yaw, 0.0, 0.0);
        let Some(mut after) = sim.object(owner).map(|o| *o.thing.get_transform_matrix()) else {
            return;
        };
        after.set_translation(&mtx.get_translation());
        sim.set_object_transform_matrix(owner, &after);
    } else {
        sim.set_object_transform_matrix(owner, &mtx);
    }
}

/// The landing frame: the null collision, then falling damage.
fn hit_ground(sim: &mut dyn SimApi, owner: ObjectSlot, active_vel_z: Real) {
    let normal = Coord3D::new(0.0, 0.0, -1.0);
    let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) else {
        return;
    };
    sim.dispatch_on_collide(owner, None, Some(&pos), Some(&normal));

    let Some(p) = physics(sim, owner) else {
        return;
    };
    let net_speed = -active_vel_z - p.min_fall_speed_for_damage;
    if !(net_speed > 0.0 && p.has_projectile_update == FALSE) {
        return;
    }
    const MIN_ANGLE_TAN: Real = 3.0;
    const TINY_DELTA: Real = 0.01;
    let vel = p.vel;
    let factor = p.data.fall_height_damage_factor;
    if (gm_fabsf(vel.x) <= TINY_DELTA || gm_fabsf(active_vel_z / vel.x) >= MIN_ANGLE_TAN)
        && (gm_fabsf(vel.y) <= TINY_DELTA || gm_fabsf(active_vel_z / vel.y) >= MIN_ANGLE_TAN)
    {
        let amount = net_speed * get_mass(sim, owner) * factor;
        let mut damage = DamageInfo::default();
        damage.input.damage_type = DamageType::Falling;
        damage.input.death_type = DeathType::Splatted;
        damage.input.source_id = sim.object(owner).map_or(ObjectId::INVALID, |o| o.id);
        damage.input.amount = amount;
        damage.input.shock_wave_amount = 0.0;
        sim.attempt_damage(owner, &mut damage);
        if is_effectively_dead(sim, owner) {
            set_model(sim, owner, ModelConditionFlagType::Splatted, true);
        }
    }
}

/// `PhysicsBehavior::onCollide`.
pub fn on_collide(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    other: Option<ObjectSlot>,
    loc: Option<&Coord3D>,
    normal: Option<&Coord3D>,
) {
    let Some(has_pui) = physics(sim, owner).map(|p| p.has_projectile_update) else {
        return;
    };
    if has_pui != FALSE && sim.projectile_handle_collision(owner, other) != FALSE {
        return;
    }
    let Some(obj) = sim.object(owner) else {
        return;
    };
    let obj_id = obj.id;
    let contained_by = obj.contained_by;
    let has_ai = obj.ai_slot.is_some();
    let team = obj.team_id;
    let Some(other) = other else {
        if let Some(container) = contained_by {
            sim.dispatch_on_collide(container, None, loc, normal);
        }
        return;
    };
    let Some(them) = sim.object(other) else {
        return;
    };
    let other_id = them.id;
    if them.contained_by == Some(owner) || contained_by == Some(other) {
        return;
    }
    if has_status(sim, owner, ObjectStatusType::Parachuting)
        && has_status(sim, other, ObjectStatusType::Parachuting)
    {
        return;
    }

    if has_ai && sim.ai_ignored_obstacle_id(owner) == other_id {
        // Infantry walking up to an unmanned vehicle take it over.
        if is_kind_of(sim, owner, KindOfType::Infantry)
            && is_disabled(sim, other, DisabledType::Unmanned)
        {
            sim.clear_disabled(other, DisabledType::Unmanned);
            if let Some(o) = sim.object_mut(other) {
                o.private_status |= PRIVATE_STATUS_CAPTURED;
            }
            sim.defect_object(other, team, 0, TRUE);
            sim.destroy_object(owner);
        }
        return;
    }
    if sim.object_has_ai(other) != FALSE && sim.ai_ignored_obstacle_id(other) == obj_id {
        return;
    }
    if physics(sim, owner)
        .is_some_and(|p| other_id != ObjectId::INVALID && other_id == p.ignore_collisions_with)
    {
        return;
    }
    let other_immobile = is_kind_of(sim, other, KindOfType::Immobile);
    match physics(sim, other) {
        Some(p) => {
            if obj_id != ObjectId::INVALID && obj_id == p.ignore_collisions_with {
                return;
            }
        }
        None => {
            if !other_immobile {
                return;
            }
        }
    }
    if check_for_overlap_collision(sim, owner, other) {
        return;
    }
    // The reference computes a mass here (999999 when immobile) and never reads it.

    if has_ai
        && !((is_effectively_dead(sim, owner)
            || has_status(sim, owner, ObjectStatusType::Parachuting))
            && other_immobile)
        && sim.ai_process_collision(owner, other) == FALSE
    {
        return;
    }

    let above = sim.object_is_above_terrain(owner) != FALSE;
    let (Some(us), Some(them)) = (sim.object(owner), sim.object(other)) else {
        return;
    };
    let mut us_center = Coord3D::default();
    let mut them_center = Coord3D::default();
    us.geometry
        .get_center_position(us.thing.get_position(), &mut us_center);
    them.geometry
        .get_center_position(them.thing.get_position(), &mut them_center);
    let mut delta = Coord3D::new(
        them_center.x - us_center.x,
        them_center.y - us_center.y,
        them_center.z - us_center.z,
    );
    let (us_radius, them_radius, dist_sqr) = if above {
        (
            us.geometry.get_bounding_sphere_radius(),
            them.geometry.get_bounding_sphere_radius(),
            delta.x * delta.x + delta.y * delta.y + delta.z * delta.z,
        )
    } else {
        let d = delta.x * delta.x + delta.y * delta.y;
        delta.z = 0.0;
        (
            us.geometry.get_bounding_circle_radius(),
            them.geometry.get_bounding_circle_radius(),
            d,
        )
    };
    if dist_sqr > (us_radius + them_radius) * (us_radius + them_radius) {
        return;
    }

    if let Some(p) = physics_mut(sim, owner) {
        p.last_collidee = other_id;
    }

    let mut dist = gm_sqrtf(dist_sqr);
    let mut overlap = us_radius + them_radius - dist;
    if dist < 1.0 {
        dist = 1.0;
    }

    if !physics(sim, owner).is_some_and(|p| p.flag(ALLOW_COLLIDE_FORCE)) {
        return;
    }
    let factor;
    let destroyed = sim.object(owner).is_none_or(|o| o.is_destroyed());
    if other_immobile && !destroyed {
        if has_status(sim, owner, ObjectStatusType::Parachuting) {
            bounce_parachute_out(sim, owner, us_radius, &delta, dist);
            return;
        }
        const MIN_STIFF: Real = 0.01;
        const MAX_STIFF: Real = 0.99;
        let mut stiffness = sim.rules().global_data.structure_stiffness;
        if stiffness < MIN_STIFF {
            stiffness = MIN_STIFF;
        }
        if stiffness > MAX_STIFF {
            stiffness = MAX_STIFF;
        }
        // The head-on speed, whatever the direction of travel.
        let mut mag = velocity_magnitude(sim, owner);
        let min_bounce_speed: Real = 1.0 / (LOGICFRAMES_PER_SECOND as Real * 5.0);
        if mag < min_bounce_speed {
            mag = min_bounce_speed;
        }
        factor = -mag * get_mass(sim, owner) * stiffness;

        let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) else {
            return;
        };
        // Falling onto it from above.
        if delta.z < 0.0 && pos.z >= sim.rules().global_data.default_structure_rubble_height {
            let vehicle = is_kind_of(sim, owner, KindOfType::Vehicle);
            if is_kind_of(sim, other, KindOfType::Structure) {
                if vehicle {
                    fire_crash_weapon(sim, owner, true, &pos);
                }
                sim.destroy_object(owner);
                return;
            } else if vehicle {
                fire_crash_weapon(sim, owner, false, &pos);
            }
        }

        if let Some(p) = physics_mut(sim, owner) {
            p.vel = Coord3D::default();
            p.invalidate_velocity_magnitude();
        }
    } else {
        if overlap > 5.0 {
            overlap = 5.0;
        }
        factor = -overlap;
    }

    let force = Coord3D::new(
        factor * delta.x / dist,
        factor * delta.y / dist,
        factor * delta.z / dist,
    );
    apply_force(sim, owner, &force);
}

/// A parachutist is nudged out of the building and stops drifting.
fn bounce_parachute_out(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    us_radius: Real,
    delta: &Coord3D,
    dist: Real,
) {
    let mut to_bounce = owner;
    while let Some(container) = sim.object(to_bounce).and_then(|o| o.contained_by) {
        to_bounce = container;
    }
    let bounce_out = us_radius * 0.1;
    let Some(mut pos) = sim.object(to_bounce).map(|o| *o.thing.get_position()) else {
        return;
    };
    pos.x -= bounce_out * delta.x / dist;
    pos.y -= bounce_out * delta.y / dist;
    sim.set_object_position(to_bounce, &pos);
    if let Some(p) = physics_mut(sim, to_bounce) {
        p.scrub_velocity_2d(0.0);
    }
}

/// `TheWeaponStore->createAndFireTempWeapon` for a vehicle that came down on something.
fn fire_crash_weapon(sim: &mut dyn SimApi, owner: ObjectSlot, building: bool, pos: &Coord3D) {
    let Some(name) = physics(sim, owner).and_then(|p| {
        if building {
            p.data.crash_into_building_weapon.clone()
        } else {
            p.data.crash_into_non_building_weapon.clone()
        }
    }) else {
        return;
    };
    weapon_fire::create_and_fire_temp_weapon(sim, owner, &name, pos);
}

fn perps_logically_equal(a: Real, b: Real) -> bool {
    const PERP_RANGE: Real = 0.15;
    gm_fabsf(a - b) <= PERP_RANGE
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CrushTarget {
    Total,
    FrontEnd,
    BackEnd,
}

/// The vector from the crusher to `target` and the distance of `target` off the
/// crusher's line of travel.
fn crush_vector(target: &Coord3D, crusher: &Coord3D, dir: &Coord3D) -> (Coord3D, Real) {
    let mut v = *target;
    v.x -= crusher.x;
    v.y -= crusher.y;
    v.z = 0.0;
    let ray = v.x * dir.x + v.y * dir.y;
    let perp = Coord3D::new(ray * dir.x - v.x, ray * dir.y - v.y, 0.0);
    (v, perp.length())
}

/// Which of the three crush points the crusher is lined up with.
fn pick_crush_target(
    front: (Coord3D, Real),
    back: (Coord3D, Real),
    center: (Coord3D, Real),
) -> CrushTarget {
    use CrushTarget::{BackEnd, FrontEnd, Total};
    let (front_v, front_p) = front;
    let (back_v, back_p) = back;
    let (center_v, center_p) = center;
    if front_p <= center_p && front_p <= back_p {
        if perps_logically_equal(front_p, center_p) || perps_logically_equal(front_p, back_p) {
            let front_len = front_v.length();
            if perps_logically_equal(front_p, center_p) {
                if front_len < center_v.length() {
                    FrontEnd
                } else {
                    Total
                }
            } else if front_len < back_v.length() {
                FrontEnd
            } else {
                BackEnd
            }
        } else {
            FrontEnd
        }
    } else if back_p <= center_p && back_p <= front_p {
        if perps_logically_equal(back_p, center_p) || perps_logically_equal(back_p, front_p) {
            let back_len = back_v.length();
            if perps_logically_equal(back_p, center_p) {
                if back_len < center_v.length() {
                    BackEnd
                } else {
                    Total
                }
            } else if back_len < front_v.length() {
                BackEnd
            } else {
                FrontEnd
            }
        } else {
            BackEnd
        }
    } else if perps_logically_equal(center_p, back_p) || perps_logically_equal(center_p, front_p) {
        let center_len = center_v.length();
        if perps_logically_equal(center_p, front_p) {
            if center_len < front_v.length() {
                Total
            } else {
                FrontEnd
            }
        } else if center_len < back_v.length() {
            Total
        } else {
            BackEnd
        }
    } else {
        Total
    }
}

fn crush_damage(sim: &mut dyn SimApi, victim: ObjectSlot, source: ObjectId, amount: Real) {
    let mut damage = DamageInfo::default();
    damage.input.damage_type = DamageType::Crush;
    damage.input.death_type = DeathType::Crushed;
    damage.input.source_id = source;
    damage.input.amount = amount;
    sim.attempt_damage(victim, &mut damage);
}

/// `checkForOverlapCollision`: `true` when crushing settles the collision.
fn check_for_overlap_collision(sim: &mut dyn SimApi, owner: ObjectSlot, other: ObjectSlot) -> bool {
    if physics(sim, owner).is_none_or(|p| is_very_small_3d(&p.vel)) {
        return false;
    }
    // `canCrushOrSquish(.., TEST_CRUSH_ONLY)`.
    let crushing = sim.object_can_crush(owner, other) != FALSE;
    let crushed = sim.object_can_crush(other, owner) != FALSE;
    if crushing && crushed {
        return false;
    }
    if crushed {
        return true;
    }
    if !crushing {
        return false;
    }

    add_overlap(sim, owner, other);
    let Some(source) = sim.object(owner).map(|o| o.id) else {
        return true;
    };
    let other_id = sim.object(other).map_or(ObjectId::INVALID, |o| o.id);
    if !physics(sim, owner).is_some_and(|p| p.previous_overlap == other_id) {
        crush_damage(sim, other, source, 0.0);
    }

    let (front_crushed, back_crushed) = sim.object_crushed_ends(other);
    if front_crushed != FALSE && back_crushed != FALSE {
        return true;
    }
    let (Some(me), Some(them)) = (sim.object(owner), sim.object(other)) else {
        return true;
    };
    let crushee_pos = *them.thing.get_position();
    let crusher_pos = *me.thing.get_position();
    let dir = me.thing.get_unit_direction_vector_2d();
    let crushee_dir = them.thing.get_unit_direction_vector_2d();
    let offset_distance = them.geometry.get_major_radius() / 2.0;
    let offset = Coord3D::new(
        crushee_dir.x * offset_distance,
        crushee_dir.y * offset_distance,
        0.0,
    );
    let mut front_point = crushee_pos;
    front_point.x += offset.x;
    front_point.y += offset.y;
    let mut back_point = crushee_pos;
    back_point.x -= offset.x;
    back_point.y -= offset.y;

    let target = if front_crushed != FALSE {
        CrushTarget::BackEnd
    } else if back_crushed != FALSE {
        CrushTarget::FrontEnd
    } else {
        pick_crush_target(
            crush_vector(&front_point, &crusher_pos, &dir),
            crush_vector(&back_point, &crusher_pos, &dir),
            crush_vector(&crushee_pos, &crusher_pos, &dir),
        )
    };

    // `2.25` is a double, so the product is rounded once.
    let too_far_squared =
        (2.25_f64 * f64::from(offset_distance) * f64::from(offset_distance)) as Real;
    let point = match target {
        CrushTarget::Total => crushee_pos,
        CrushTarget::FrontEnd => front_point,
        CrushTarget::BackEnd => back_point,
    };
    // Crush when the point has just been passed and is close enough.
    let dx = point.x - crusher_pos.x;
    let dy = point.y - crusher_pos.y;
    let dot = dir.x * dx + dir.y * dy;
    let distance_squared = (dx * dx) + (dy * dy);
    if dot < 0.0 && distance_squared < too_far_squared {
        crush_damage(sim, other, source, HUGE_DAMAGE_AMOUNT);
    }
    true
}

/// Runtime instance of `PhysicsBehavior`.
pub struct PhysicsBehavior {
    pub data: PhysicsBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
    owner: ObjectSlot,
    /// Members read by `xfer` in load mode, handed to the object in `load_post_process`.
    loaded: Option<Box<PhysicsState>>,
}

impl PhysicsBehavior {
    pub fn new(data: PhysicsBehaviorModuleData, owner: ObjectSlot) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            owner,
            loaded: None,
        }
    }
}

impl UpdateFacet for PhysicsBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        update(sim, ctx.owner_slot)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }

    fn get_update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Physics
    }
}

impl CollideFacet for PhysicsBehavior {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    ) {
        on_collide(sim, ctx.owner_slot, other, loc, normal);
    }
}

impl PhysicsFacet for PhysicsBehavior {}

impl BehaviorModule for PhysicsBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.owner = ctx.owner_slot;
        let gravity = sim.rules().global_data.gravity;
        let state = PhysicsState::new(self.data.clone(), ctx.mod_slot, gravity);
        if let Some(obj) = sim.object_mut(ctx.owner_slot) {
            obj.physics = Some(Box::new(state));
        }
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let pui = is_kind_of(sim, ctx.owner_slot, KindOfType::Projectile)
            && sim
                .object(ctx.owner_slot)
                .is_some_and(|o| o.projectile_slot.is_some());
        if let Some(p) = physics_mut(sim, ctx.owner_slot) {
            p.has_projectile_update = Bool::from(pui);
        }
    }

    /// `PhysicsBehavior::crc` is `UpdateModule::crc`, which adds nothing.
    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let mut state = match (&self.loaded, physics(sim, self.owner)) {
            (Some(loaded), _) => (**loaded).clone(),
            (None, Some(live)) => live.clone(),
            (None, None) => PhysicsState::new(
                self.data.clone(),
                self.schedule.mod_slot.unwrap_or_default(),
                sim.rules().global_data.gravity,
            ),
        };
        state.xfer_members(xfer, version)?;
        if xfer.mode() == XferMode::Load {
            self.loaded = Some(Box::new(state));
        }
        Ok(())
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
        if let Some(loaded) = self.loaded.take() {
            if let Some(p) = physics_mut(sim, ctx.owner_slot) {
                p.take_saved_members(&loaded);
            }
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

    fn as_collide(&self) -> Option<&dyn CollideFacet> {
        Some(self)
    }

    fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
        Some(self)
    }

    fn as_physics(&self) -> Option<&dyn PhysicsFacet> {
        Some(self)
    }

    fn as_physics_mut(&mut self) -> Option<&mut dyn PhysicsFacet> {
        Some(self)
    }
}

fn parse_friction_per_sec(r: &mut IniReader<'_>, store: &mut Real) -> Result<(), IniError> {
    let mut per_sec = 0.0;
    r.parse_real(&mut per_sec)?;
    // `SECONDS_PER_LOGICFRAME_REAL`.
    *store = per_sec * (1.0 / LOGICFRAMES_PER_SECOND as Real);
    Ok(())
}

fn parse_weapon_name(
    r: &mut IniReader<'_>,
    ctx: &ModuleParseContext<'_>,
    store: &mut Option<String>,
) -> Result<(), IniError> {
    let mut name = String::new();
    r.parse_ascii_string(&mut name)?;
    *store = if name.eq_ignore_ascii_case("None")
        || ctx.weapon_store.find_weapon_template(&name).is_none()
    {
        None
    } else {
        Some(name)
    };
    Ok(())
}

pub fn parse_physics_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut d = PhysicsBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("Mass") {
            r.parse_positive_non_zero_real(&mut d.mass)
        } else if is("ShockResistance") {
            r.parse_positive_non_zero_real(&mut d.shock_resistance)
        } else if is("ShockMaxYaw") {
            r.parse_positive_non_zero_real(&mut d.shock_max_yaw)
        } else if is("ShockMaxPitch") {
            r.parse_positive_non_zero_real(&mut d.shock_max_pitch)
        } else if is("ShockMaxRoll") {
            r.parse_positive_non_zero_real(&mut d.shock_max_roll)
        } else if is("ForwardFriction") {
            parse_friction_per_sec(r, &mut d.forward_friction)
        } else if is("LateralFriction") {
            parse_friction_per_sec(r, &mut d.lateral_friction)
        } else if is("ZFriction") {
            parse_friction_per_sec(r, &mut d.z_friction)
        } else if is("AerodynamicFriction") {
            parse_friction_per_sec(r, &mut d.aerodynamic_friction)
        } else if is("CenterOfMassOffset") {
            r.parse_real(&mut d.center_of_mass_offset)
        } else if is("AllowBouncing") {
            r.parse_bool(&mut d.allow_bouncing)
        } else if is("AllowCollideForce") {
            r.parse_bool(&mut d.allow_collide_force)
        } else if is("KillWhenRestingOnGround") {
            r.parse_bool(&mut d.kill_when_resting_on_ground)
        } else if is("MinFallHeightForDamage") {
            r.parse_real(&mut d.min_fall_height_for_damage)
        } else if is("FallHeightDamageFactor") {
            r.parse_real(&mut d.fall_height_damage_factor)
        } else if is("PitchRollYawFactor") {
            r.parse_real(&mut d.pitch_roll_yaw_factor)
        } else if is("VehicleCrashesIntoBuildingWeaponTemplate") {
            parse_weapon_name(r, parse_ctx, &mut d.crash_into_building_weapon)
        } else if is("VehicleCrashesIntoNonBuildingWeaponTemplate") {
            parse_weapon_name(r, parse_ctx, &mut d.crash_into_non_building_weapon)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(d))
}

pub fn create_physics_behavior(
    data: &dyn ModuleData,
    ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<PhysicsBehaviorModuleData>()
        .expect("PhysicsBehavior data type mismatch")
        .clone();
    Box::new(PhysicsBehavior::new(typed, ctx.owner_slot))
}
