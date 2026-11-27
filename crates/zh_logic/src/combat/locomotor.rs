//! `PKT-143`: Locomotor templates, stores, runtime overrides, snapshots and movement
//! (`Locomotor.cpp`). Packet doc: `packets/phase3/PKT-143_Locomotor.md`. Golden suites:
//! `golden_pkt143_locomotor` and `golden_s7_locomotor_movement`.
//!
//! The movement functions drive the object's `PhysicsBehavior` and transform through
//! [`SimApi`]. What they ask of code that is not ported yet goes through
//! [`LocomotorSeams`].

use std::collections::BTreeMap;

use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::dumb_projectile_behavior::build_transform_matrix;
use crate::modules::update::physics_update::{
    self as physics, APPLY_FRICTION2D_WHEN_AIRBORNE, IS_STUNNED, STICK_TO_GROUND, TURN_NEGATIVE,
    TURN_NONE, TURN_POSITIVE,
};
use crate::rules::object_status_types::ObjectStatusType;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{LocomotorSurfaceTypeMask, LOCOMOTOR_SURFACE_TYPE_NAMES};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::arena::ObjectSlot;
use crate::sim::SimApi;
use crate::combat::turret_ai::normalize_angle_cpp;
use zh_common::ini::{
    IniError, IniLoadType, IniReader, LOGICFRAMES_PER_SECOND, SECONDS_PER_LOGICFRAME_REAL, SEPS_DEFAULT,
};
use zh_common::math::{
    gm_acosf, gm_atan2f, gm_cosf, gm_fabsf, gm_sinf, reference_div_f32, Coord3D, Matrix3D, PI,
};
use zh_common::random_value::PrngState;
use zh_common::types::{Bool, Int, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const BIGNUM: Real = 99999.0_f32;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocomotorAppearance {
    #[default]
    LegsTwo = 0,
    WheelsFour = 1,
    Treads = 2,
    Hover = 3,
    Thrust = 4,
    Wings = 5,
    Climber = 6,
    Other = 7,
    Motorcycle = 8,
    Count = 9,
}

impl LocomotorAppearance {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::LegsTwo),
            1 => Some(Self::WheelsFour),
            2 => Some(Self::Treads),
            3 => Some(Self::Hover),
            4 => Some(Self::Thrust),
            5 => Some(Self::Wings),
            6 => Some(Self::Climber),
            7 => Some(Self::Other),
            8 => Some(Self::Motorcycle),
            _ => None,
        }
    }
    pub const fn as_i32(self) -> Int { self as Int }
}

pub const LOCOMOTOR_APPEARANCE_NAMES: [&str; 9] = [
    "TWO_LEGS", "FOUR_WHEELS", "TREADS", "HOVER", "THRUST", "WINGS",
    "CLIMBER", "OTHER", "MOTORCYCLE",
];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocomotorPriority {
    #[default]
    MovesBack = 0,
    MovesMiddle = 1,
    MovesFront = 2,
    Count = 3,
}

pub const LOCOMOTOR_PRIORITY_NAMES: [&str; 3] = ["MOVES_BACK", "MOVES_MIDDLE", "MOVES_FRONT"];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocomotorBehaviorZ {
    #[default]
    NoZMotiveForce = 0,
    SeaLevel = 1,
    SurfaceRelativeHeight = 2,
    AbsoluteHeight = 3,
    FixedSurfaceRelativeHeight = 4,
    FixedAbsoluteHeight = 5,
    RelativeToGroundAndBuildings = 6,
    RelativeToHighestLayer = 7,
    Count = 8,
}

impl LocomotorBehaviorZ {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::NoZMotiveForce),
            1 => Some(Self::SeaLevel),
            2 => Some(Self::SurfaceRelativeHeight),
            3 => Some(Self::AbsoluteHeight),
            4 => Some(Self::FixedSurfaceRelativeHeight),
            5 => Some(Self::FixedAbsoluteHeight),
            6 => Some(Self::RelativeToGroundAndBuildings),
            7 => Some(Self::RelativeToHighestLayer),
            _ => None,
        }
    }
    pub const fn as_i32(self) -> Int { self as Int }
}

pub const LOCOMOTOR_BEHAVIOR_Z_NAMES: [&str; 8] = [
    "NO_Z_MOTIVE_FORCE", "SEA_LEVEL", "SURFACE_RELATIVE_HEIGHT", "ABSOLUTE_HEIGHT",
    "FIXED_SURFACE_RELATIVE_HEIGHT", "FIXED_ABSOLUTE_HEIGHT",
    "FIXED_RELATIVE_TO_GROUND_AND_BUILDINGS", "RELATIVE_TO_HIGHEST_LAYER",
];

#[derive(Debug, Clone, PartialEq)]
pub struct LocomotorTemplate {
    pub name: String,
    pub surfaces: LocomotorSurfaceTypeMask,
    pub max_speed: Real,
    pub max_speed_damaged: Real,
    pub min_speed: Real,
    pub max_turn_rate: Real,
    pub max_turn_rate_damaged: Real,
    pub acceleration: Real,
    pub acceleration_damaged: Real,
    pub lift: Real,
    pub lift_damaged: Real,
    pub braking: Real,
    pub min_turn_speed: Real,
    pub preferred_height: Real,
    pub preferred_height_damping: Real,
    pub circling_radius: Real,
    pub speed_limit_z: Real,
    pub extra_2d_friction: Real,
    pub max_thrust_angle: Real,
    pub behavior_z: LocomotorBehaviorZ,
    pub appearance: LocomotorAppearance,
    pub move_priority: LocomotorPriority,
    pub accel_pitch_limit: Real,
    pub decel_pitch_limit: Real,
    pub bounce_kick: Real,
    pub pitch_stiffness: Real,
    pub roll_stiffness: Real,
    pub pitch_damping: Real,
    pub roll_damping: Real,
    pub pitch_by_z_vel_coef: Real,
    pub thrust_roll: Real,
    pub wobble_rate: Real,
    pub min_wobble: Real,
    pub max_wobble: Real,
    pub forward_vel_coef: Real,
    pub lateral_vel_coef: Real,
    pub forward_accel_coef: Real,
    pub lateral_accel_coef: Real,
    pub uniform_axial_damping: Real,
    pub turn_pivot_offset: Real,
    pub airborne_targeting_height: Int,
    pub close_enough_dist: Real,
    pub is_close_enough_dist3d: Bool,
    pub ultra_accurate_slide_factor: Real,
    pub locomotor_works_when_dead: Bool,
    pub allow_motive_force_airborne: Bool,
    pub apply_2d_friction_airborne: Bool,
    pub downhill_only: Bool,
    pub stick_to_ground: Bool,
    pub can_move_backward: Bool,
    pub has_suspension: Bool,
    pub wheel_turn_angle: Real,
    pub maximum_wheel_extension: Real,
    pub maximum_wheel_compression: Real,
    pub wander_width_factor: Real,
    pub wander_length_factor: Real,
    pub wander_about_point_radius: Real,
    pub rudder_correction_degree: Real,
    pub rudder_correction_rate: Real,
    pub elevator_correction_degree: Real,
    pub elevator_correction_rate: Real,
}

impl Default for LocomotorTemplate {
    fn default() -> Self {
        Self {
            name: String::new(), surfaces: 0, max_speed: 0.0, max_speed_damaged: -1.0,
            min_speed: 0.0, max_turn_rate: 0.0, max_turn_rate_damaged: -1.0,
            acceleration: 0.0, acceleration_damaged: -1.0, lift: 0.0, lift_damaged: -1.0,
            braking: BIGNUM, min_turn_speed: BIGNUM, preferred_height: 0.0,
            preferred_height_damping: 1.0, circling_radius: 0.0, speed_limit_z: 999999.0,
            extra_2d_friction: 0.0, max_thrust_angle: 0.0,
            behavior_z: LocomotorBehaviorZ::NoZMotiveForce,
            appearance: LocomotorAppearance::Other,
            move_priority: LocomotorPriority::MovesMiddle,
            accel_pitch_limit: 0.0, decel_pitch_limit: 0.0, bounce_kick: 0.0,
            pitch_stiffness: 0.1, roll_stiffness: 0.1, pitch_damping: 0.9, roll_damping: 0.9,
            pitch_by_z_vel_coef: 0.0, thrust_roll: 0.0, wobble_rate: 0.0,
            min_wobble: 0.0, max_wobble: 0.0, forward_vel_coef: 0.0,
            lateral_vel_coef: 0.0, forward_accel_coef: 0.0, lateral_accel_coef: 0.0,
            uniform_axial_damping: 1.0, turn_pivot_offset: 0.0,
            airborne_targeting_height: Int::MAX, close_enough_dist: 1.0,
            is_close_enough_dist3d: FALSE, ultra_accurate_slide_factor: 0.0,
            locomotor_works_when_dead: FALSE, allow_motive_force_airborne: FALSE,
            apply_2d_friction_airborne: FALSE, downhill_only: FALSE, stick_to_ground: FALSE,
            can_move_backward: FALSE, has_suspension: FALSE, wheel_turn_angle: 0.0,
            maximum_wheel_extension: 0.0, maximum_wheel_compression: 0.0,
            wander_width_factor: 0.0, wander_length_factor: 1.0, wander_about_point_radius: 0.0,
            rudder_correction_degree: 0.0, rudder_correction_rate: 0.0,
            elevator_correction_degree: 0.0, elevator_correction_rate: 0.0,
        }
    }
}

impl LocomotorTemplate {
    pub fn new() -> Self { Self::default() }

    pub fn parse_ini_block(&mut self, text: &str) -> Result<(), IniError> {
        self.parse_ini_block_collect(text, &mut Vec::new())
    }

    /// As [`Self::parse_ini_block`], appending the keys the field table lacks.
    pub fn parse_ini_block_collect(
        &mut self,
        text: &str,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let mut ini = IniReader::new("", text.as_bytes(), IniLoadType::Overwrite);
        ini.read_line(None)?;
        let _keyword = ini.next_token(SEPS_DEFAULT)?;
        self.name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        self.parse_fields(&mut ini, ignored)
    }

    /// `INI::initFromINI` over the locomotor field table, from the line after the
    /// header through `End`, then `validate`. A key outside the table is skipped, as
    /// the reference build does (`DEBUG_CRASH` only), and appended to `ignored`.
    pub fn parse_fields(
        &mut self,
        ini: &mut IniReader<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        ini.read_block_fields(|key, ini| {
            if !self.parse_field(key, ini)? {
                ignored.push(key.to_owned());
            }
            Ok(())
        })?;
        self.validate()
    }

    /// One row of the field table; `false` when `key` is not in it (case sensitive).
    fn parse_field(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        let index = |ini: &mut IniReader<'_>, names: &[&str]| {
            IniReader::scan_index_list(ini.next_token(SEPS_DEFAULT)?, names)
        };
        match key {
            "Surfaces" => {
                let mut bits = self.surfaces as UnsignedInt;
                ini.parse_bit_string32(&mut bits, &LOCOMOTOR_SURFACE_TYPE_NAMES)?;
                self.surfaces = bits as Int;
            }
            "Speed" => ini.parse_velocity_real(&mut self.max_speed)?,
            "SpeedDamaged" => ini.parse_velocity_real(&mut self.max_speed_damaged)?,
            "TurnRate" => ini.parse_angular_velocity_real(&mut self.max_turn_rate)?,
            "TurnRateDamaged" => ini.parse_angular_velocity_real(&mut self.max_turn_rate_damaged)?,
            "Acceleration" => ini.parse_acceleration_real(&mut self.acceleration)?,
            "AccelerationDamaged" => ini.parse_acceleration_real(&mut self.acceleration_damaged)?,
            "Lift" => ini.parse_acceleration_real(&mut self.lift)?,
            "LiftDamaged" => ini.parse_acceleration_real(&mut self.lift_damaged)?,
            "Braking" => ini.parse_acceleration_real(&mut self.braking)?,
            "MinSpeed" => ini.parse_velocity_real(&mut self.min_speed)?,
            "MinTurnSpeed" => ini.parse_velocity_real(&mut self.min_turn_speed)?,
            "PreferredHeight" => ini.parse_real(&mut self.preferred_height)?,
            "PreferredHeightDamping" => ini.parse_real(&mut self.preferred_height_damping)?,
            "CirclingRadius" => ini.parse_real(&mut self.circling_radius)?,
            "Extra2DFriction" => {
                ini.parse_real(&mut self.extra_2d_friction)?;
                self.extra_2d_friction *= SECONDS_PER_LOGICFRAME_REAL;
            }
            "SpeedLimitZ" => ini.parse_velocity_real(&mut self.speed_limit_z)?,
            "MaxThrustAngle" => ini.parse_angle_real(&mut self.max_thrust_angle)?,
            "ZAxisBehavior" => {
                let i = index(ini, &LOCOMOTOR_BEHAVIOR_Z_NAMES)?;
                self.behavior_z = LocomotorBehaviorZ::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "Appearance" => {
                let i = index(ini, &LOCOMOTOR_APPEARANCE_NAMES)?;
                self.appearance = LocomotorAppearance::from_i32(i).ok_or(IniError::InvalidData)?;
            }
            "GroupMovementPriority" => {
                self.move_priority = match index(ini, &LOCOMOTOR_PRIORITY_NAMES)? {
                    0 => LocomotorPriority::MovesBack,
                    2 => LocomotorPriority::MovesFront,
                    _ => LocomotorPriority::MovesMiddle,
                };
            }
            "AccelerationPitchLimit" => ini.parse_angle_real(&mut self.accel_pitch_limit)?,
            "DecelerationPitchLimit" => ini.parse_angle_real(&mut self.decel_pitch_limit)?,
            "BounceAmount" => ini.parse_angular_velocity_real(&mut self.bounce_kick)?,
            "PitchStiffness" => ini.parse_real(&mut self.pitch_stiffness)?,
            "RollStiffness" => ini.parse_real(&mut self.roll_stiffness)?,
            "PitchDamping" => ini.parse_real(&mut self.pitch_damping)?,
            "RollDamping" => ini.parse_real(&mut self.roll_damping)?,
            "ThrustRoll" => ini.parse_real(&mut self.thrust_roll)?,
            "ThrustWobbleRate" => ini.parse_real(&mut self.wobble_rate)?,
            "ThrustMinWobble" => ini.parse_real(&mut self.min_wobble)?,
            "ThrustMaxWobble" => ini.parse_real(&mut self.max_wobble)?,
            "PitchInDirectionOfZVelFactor" => ini.parse_real(&mut self.pitch_by_z_vel_coef)?,
            "ForwardVelocityPitchFactor" => ini.parse_real(&mut self.forward_vel_coef)?,
            "LateralVelocityRollFactor" => ini.parse_real(&mut self.lateral_vel_coef)?,
            "ForwardAccelerationPitchFactor" => ini.parse_real(&mut self.forward_accel_coef)?,
            "LateralAccelerationRollFactor" => ini.parse_real(&mut self.lateral_accel_coef)?,
            "UniformAxialDamping" => ini.parse_real(&mut self.uniform_axial_damping)?,
            "TurnPivotOffset" => ini.parse_real(&mut self.turn_pivot_offset)?,
            "Apply2DFrictionWhenAirborne" => ini.parse_bool(&mut self.apply_2d_friction_airborne)?,
            "DownhillOnly" => ini.parse_bool(&mut self.downhill_only)?,
            "AllowAirborneMotiveForce" => ini.parse_bool(&mut self.allow_motive_force_airborne)?,
            "LocomotorWorksWhenDead" => ini.parse_bool(&mut self.locomotor_works_when_dead)?,
            "AirborneTargetingHeight" => ini.parse_int(&mut self.airborne_targeting_height)?,
            "StickToGround" => ini.parse_bool(&mut self.stick_to_ground)?,
            "CanMoveBackwards" => ini.parse_bool(&mut self.can_move_backward)?,
            "HasSuspension" => ini.parse_bool(&mut self.has_suspension)?,
            "FrontWheelTurnAngle" => ini.parse_angle_real(&mut self.wheel_turn_angle)?,
            "MaximumWheelExtension" => ini.parse_real(&mut self.maximum_wheel_extension)?,
            "MaximumWheelCompression" => ini.parse_real(&mut self.maximum_wheel_compression)?,
            "CloseEnoughDist" => ini.parse_real(&mut self.close_enough_dist)?,
            "CloseEnoughDist3D" => ini.parse_bool(&mut self.is_close_enough_dist3d)?,
            "SlideIntoPlaceTime" => ini.parse_duration_real(&mut self.ultra_accurate_slide_factor)?,
            "WanderWidthFactor" => ini.parse_real(&mut self.wander_width_factor)?,
            "WanderLengthFactor" => ini.parse_real(&mut self.wander_length_factor)?,
            "WanderAboutPointRadius" => ini.parse_real(&mut self.wander_about_point_radius)?,
            "RudderCorrectionDegree" => ini.parse_real(&mut self.rudder_correction_degree)?,
            "RudderCorrectionRate" => ini.parse_real(&mut self.rudder_correction_rate)?,
            "ElevatorCorrectionDegree" => ini.parse_real(&mut self.elevator_correction_degree)?,
            "ElevatorCorrectionRate" => ini.parse_real(&mut self.elevator_correction_rate)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    pub fn validate(&mut self) -> Result<(), IniError> {
        if self.max_speed_damaged < 0.0 { self.max_speed_damaged = self.max_speed; }
        if self.max_turn_rate_damaged < 0.0 { self.max_turn_rate_damaged = self.max_turn_rate; }
        if self.acceleration_damaged < 0.0 { self.acceleration_damaged = self.acceleration; }
        if self.lift_damaged < 0.0 { self.lift_damaged = self.lift; }
        if self.appearance == LocomotorAppearance::Wings {
            if self.min_speed <= 0.0 { self.min_speed = 0.01; }
            if self.min_turn_speed <= 0.0 { self.min_turn_speed = 0.01; }
        }
        if self.appearance == LocomotorAppearance::Thrust {
            if self.behavior_z != LocomotorBehaviorZ::NoZMotiveForce || self.lift != 0.0 || self.lift_damaged != 0.0 {
                return Err(IniError::InvalidData);
            }
            if self.max_speed <= 0.0 { self.max_speed = 0.01; }
            if self.max_speed_damaged <= 0.0 { self.max_speed_damaged = 0.01; }
            if self.min_speed <= 0.0 { self.min_speed = 0.01; }
        }
        Ok(())
    }

    pub fn get_actual_max_speed(&self) -> Real { self.max_speed }
    pub fn get_actual_max_speed_damaged(&self) -> Real { self.max_speed_damaged }
    pub fn get_actual_min_speed(&self) -> Real { self.min_speed }
    pub fn get_actual_min_turn_speed(&self) -> Real { self.min_turn_speed }
    pub fn scale_speed(&self, speed: Real) -> Real { speed }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocomotorStore {
    templates: BTreeMap<i32, LocomotorTemplate>,
}

impl Default for LocomotorStore {
    fn default() -> Self { Self::new() }
}

impl LocomotorStore {
    pub fn new() -> Self { Self { templates: BTreeMap::new() } }
    pub fn clear(&mut self) { self.templates.clear(); }
    pub fn add_template(&mut self, template: LocomotorTemplate) {
        let key = fnv_name_key(&template.name);
        self.templates.insert(key, template);
    }
    pub fn find_by_name(&self, name: &str) -> Option<&LocomotorTemplate> {
        self.templates.get(&fnv_name_key(name))
    }
    pub fn find_by_key(&self, key: i32) -> Option<&LocomotorTemplate> {
        self.templates.get(&key)
    }
}

pub fn fnv_name_key(name: &str) -> i32 {
    let mut h: u32 = 0;
    for b in name.bytes() {
        h = h.wrapping_mul(0x1003_f103).wrapping_add(b as u32);
    }
    h as i32
}

pub fn is_condition_better(condition: BodyDamageType, movement_penalty: BodyDamageType) -> bool {
    condition.as_i32() < movement_penalty.as_i32()
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocomotorFlag {
    IsBraking = 0,
    AllowInvalidPosition = 1,
    MaintainPosValid = 2,
    PreciseZPos = 3,
    NoSlowDown = 4,
    OverWater = 5,
    UltraAccurate = 6,
    MovingBackwards = 7,
    DoingThreePointTurn = 8,
    Climbing = 9,
    CloseEnough3D = 10,
    OffsetIncreasing = 11,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Locomotor {
    pub template: LocomotorTemplate,
    pub maintain_pos: Coord3D,
    pub braking_factor: Real,
    pub max_lift: Real,
    pub max_speed: Real,
    pub max_accel: Real,
    pub max_braking: Real,
    pub max_turn_rate: Real,
    pub close_enough_dist: Real,
    pub flags: UnsignedInt,
    pub preferred_height: Real,
    pub preferred_height_damping: Real,
    pub angle_offset: Real,
    pub offset_increment: Real,
    pub donut_timer: UnsignedInt,
}

impl Locomotor {
    pub fn new(template: &LocomotorTemplate, rng: &mut PrngState, frame: UnsignedInt) -> Self {
        let angle_offset = rng.get_game_logic_random_value_real(-PI / 6.0, PI / 6.0);
        let wander = rng.get_game_logic_random_value_real(0.8, 1.2) / template.wander_length_factor;
        let offset_increment = (PI / 40.0) * wander;
        let offset_increasing = rng.get_game_logic_random_value(0, 1);
        let mut flags = 0u32;
        if template.is_close_enough_dist3d != FALSE { flags |= 1u32 << LocomotorFlag::CloseEnough3D as u32; }
        if offset_increasing != 0 { flags |= 1u32 << LocomotorFlag::OffsetIncreasing as u32; }
        Self {
            template: template.clone(), maintain_pos: Coord3D { x: 0.0, y: 0.0, z: 0.0 },
            braking_factor: 1.0, max_lift: BIGNUM, max_speed: BIGNUM, max_accel: BIGNUM,
            max_braking: BIGNUM, max_turn_rate: BIGNUM,
            close_enough_dist: template.close_enough_dist, flags,
            preferred_height: template.preferred_height,
            preferred_height_damping: template.preferred_height_damping,
            angle_offset, offset_increment,
            donut_timer: donut_deadline(frame),
        }
    }

    pub fn get_template_name(&self) -> &str { &self.template.name }
    pub fn get_legal_surfaces(&self) -> LocomotorSurfaceTypeMask { self.template.surfaces }
    pub fn get_is_downhill_only(&self) -> Bool { self.template.downhill_only }
    pub fn get_max_speed_for_condition(&self, condition: BodyDamageType, movement_penalty: BodyDamageType) -> Real {
        let mut speed = if is_condition_better(condition, movement_penalty) {
            self.template.get_actual_max_speed()
        } else {
            self.template.get_actual_max_speed_damaged()
        };
        let override_speed = self.template.scale_speed(self.max_speed);
        if speed > override_speed { speed = override_speed; }
        speed
    }
    pub fn get_max_turn_rate(&self, condition: BodyDamageType, movement_penalty: BodyDamageType, ultra: Bool) -> Real {
        let mut turn = if is_condition_better(condition, movement_penalty) {
            self.template.max_turn_rate
        } else {
            self.template.max_turn_rate_damaged
        };
        if turn > self.max_turn_rate { turn = self.max_turn_rate; }
        if ultra != FALSE { turn *= 2.0_f32; }
        turn
    }
    pub fn get_max_acceleration(&self, condition: BodyDamageType, movement_penalty: BodyDamageType) -> Real {
        let mut accel = if is_condition_better(condition, movement_penalty) {
            self.template.acceleration
        } else {
            self.template.acceleration_damaged
        };
        if accel > self.max_accel { accel = self.max_accel; }
        accel
    }
    pub fn get_max_lift(&self, condition: BodyDamageType, movement_penalty: BodyDamageType) -> Real {
        let mut lift = if is_condition_better(condition, movement_penalty) {
            self.template.lift
        } else {
            self.template.lift_damaged
        };
        if lift > self.max_lift { lift = self.max_lift; }
        lift
    }
    pub fn get_braking(&self) -> Real {
        if self.template.braking > self.max_braking { self.max_braking } else { self.template.braking }
    }
    pub fn get_min_speed(&self) -> Real { self.template.get_actual_min_speed() }
    pub fn get_min_turn_speed(&self) -> Real { self.template.get_actual_min_turn_speed() }
    pub fn calc_min_turn_radius(&self, condition: BodyDamageType, movement_penalty: BodyDamageType) -> (Real, Real) {
        let min_speed = self.get_min_speed();
        let max_turn = self.get_max_turn_rate(condition, movement_penalty, self.is_ultra_accurate());
        let radius = if max_turn > 0.0 { min_speed / max_turn } else { BIGNUM };
        let time = if min_speed > 0.0 { radius / min_speed } else { 0.0 };
        (radius, time)
    }
    pub fn set_allow_invalid_position(&mut self, allow: Bool) { self.set_flag(LocomotorFlag::AllowInvalidPosition, allow); }
    pub fn set_close_enough_dist(&mut self, dist: Real) { self.close_enough_dist = dist; }
    pub fn set_close_enough_dist3d(&mut self, setting: Bool) { self.set_flag(LocomotorFlag::CloseEnough3D, setting); }
    pub fn set_preferred_height(&mut self, height: Real) { self.preferred_height = height; }
    pub fn set_use_precise_z_pos(&mut self, value: Bool) { self.set_flag(LocomotorFlag::PreciseZPos, value); }
    pub fn set_no_slow_down(&mut self, value: Bool) { self.set_flag(LocomotorFlag::NoSlowDown, value); }
    pub fn set_ultra_accurate(&mut self, value: Bool) { self.set_flag(LocomotorFlag::UltraAccurate, value); }
    pub fn start_move(&mut self, frame: UnsignedInt) {
        self.donut_timer = donut_deadline(frame);
    }
    pub fn is_ultra_accurate(&self) -> Bool { self.get_flag(LocomotorFlag::UltraAccurate) }
    pub fn set_max_lift(&mut self, lift: Real) { self.max_lift = lift; }
    pub fn set_max_speed(&mut self, speed: Real) { self.max_speed = speed; }
    pub fn set_max_acceleration(&mut self, accel: Real) { self.max_accel = accel; }
    pub fn set_max_braking(&mut self, braking: Real) { self.max_braking = braking; }
    pub fn set_max_turn_rate(&mut self, turn: Real) { self.max_turn_rate = turn; }
    pub fn set_movement_overrides(&mut self, max_speed: Real, max_turn: Real, max_accel: Real, max_lift: Real, braking: Real, close: Real, pref_height: Real,
        allow_invalid: Bool, close3d: Bool, precise_z: Bool, no_slow: Bool, ultra: Bool) {
        if max_speed != BIGNUM { self.max_speed = max_speed; }
        if max_turn != BIGNUM { self.max_turn_rate = max_turn; }
        if max_accel != BIGNUM { self.max_accel = max_accel; }
        if max_lift != BIGNUM { self.max_lift = max_lift; }
        if braking != BIGNUM { self.max_braking = braking; }
        if close != BIGNUM { self.close_enough_dist = close; }
        if pref_height != BIGNUM { self.preferred_height = pref_height; }
        self.set_allow_invalid_position(allow_invalid);
        self.set_close_enough_dist3d(close3d);
        self.set_use_precise_z_pos(precise_z);
        self.set_no_slow_down(no_slow);
        self.set_ultra_accurate(ultra);
    }
    fn get_flag(&self, flag: LocomotorFlag) -> Bool {
        if self.flags & (1u32 << flag as u32) != 0 { TRUE } else { FALSE }
    }
    fn set_flag(&mut self, flag: LocomotorFlag, value: Bool) {
        if value != FALSE { self.flags |= 1u32 << flag as u32; } else { self.flags &= !(1u32 << flag as u32); }
    }
}

impl Snapshot for Locomotor {
    fn crc(&self, _xfer: &mut dyn Xfer) {}
    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = if xfer.mode() == XferMode::Load { 2 } else { 2 };
        xfer.xfer_version(&mut version, 2)?;
        if version >= 2 { xfer.xfer_unsigned_int(&mut self.donut_timer)?; }
        xfer.xfer_coord3d(&mut self.maintain_pos)?;
        xfer.xfer_real(&mut self.braking_factor)?;
        xfer.xfer_real(&mut self.max_lift)?;
        xfer.xfer_real(&mut self.max_speed)?;
        xfer.xfer_real(&mut self.max_accel)?;
        xfer.xfer_real(&mut self.max_braking)?;
        xfer.xfer_real(&mut self.max_turn_rate)?;
        xfer.xfer_real(&mut self.close_enough_dist)?;
        xfer.xfer_unsigned_int(&mut self.flags)?;
        xfer.xfer_real(&mut self.preferred_height)?;
        xfer.xfer_real(&mut self.preferred_height_damping)?;
        xfer.xfer_real(&mut self.angle_offset)?;
        xfer.xfer_real(&mut self.offset_increment)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Movement
// ---------------------------------------------------------------------------

const PATHFIND_CELL_SIZE_F: Real = 10.0;
/// `DONUT_DISTANCE`: four path cells.
const DONUT_DISTANCE: Real = 4.0 * PATHFIND_CELL_SIZE_F;
const MAX_BRAKING_FACTOR: Real = 5.0;
const LOCOMOTORSURFACE_AIR: LocomotorSurfaceTypeMask = 1 << 3;

/// `frame + DONUT_TIME_DELAY_SECONDS * LOGICFRAMES_PER_SECOND`. The sum is made in
/// single precision, so it rounds above 2^24.
fn donut_deadline(frame: UnsignedInt) -> UnsignedInt {
    (frame as Real + 2.5_f32 * LOGICFRAMES_PER_SECOND as Real) as UnsignedInt
}

/// What movement asks of code that is not ported yet.
pub trait LocomotorSeams {
    /// `Pathfinder::validMovementTerrain` for a locomotor with these surfaces.
    fn valid_movement_terrain(
        &mut self,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> Bool;
    /// `PartitionManager::getGroundOrStructureHeight`.
    fn ground_or_structure_height(&mut self, x: Real, y: Real) -> Real;
}

/// The object being moved and everything it is moved in.
pub struct LocomotorWorld<'a> {
    pub sim: &'a mut dyn SimApi,
    pub owner: ObjectSlot,
    pub seams: &'a mut dyn LocomotorSeams,
}

impl LocomotorWorld<'_> {
    /// `Pathfinder::validMovementTerrain`: the sim's map, else the seam.
    pub fn valid_movement_terrain(
        &mut self,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> Bool {
        match self.sim.pathfinder() {
            Some(map) => Bool::from(map.valid_movement_terrain(layer, surfaces, pos)),
            None => self.seams.valid_movement_terrain(layer, surfaces, pos),
        }
    }
    fn has_physics(&self) -> bool {
        physics::physics(self.sim, self.owner).is_some()
    }
    fn physics_flag(&self, flag: Int) -> bool {
        physics::physics(self.sim, self.owner).is_some_and(|p| p.flag(flag))
    }
    fn set_turning(&mut self, turning: Int) {
        if let Some(p) = physics::physics_mut(self.sim, self.owner) {
            p.turning = turning;
        }
    }
    fn apply_motive_force(&mut self, x: Real, y: Real, z: Real) {
        physics::apply_motive_force(self.sim, self.owner, &Coord3D { x, y, z });
    }
    fn scrub_velocity_2d(&mut self, desired: Real) {
        if let Some(p) = physics::physics_mut(self.sim, self.owner) {
            p.scrub_velocity_2d(desired);
        }
    }
    fn velocity(&self) -> Coord3D {
        physics::physics(self.sim, self.owner).map_or_else(Coord3D::default, |p| p.vel)
    }
    fn velocity_magnitude(&self) -> Real {
        physics::velocity_magnitude(self.sim, self.owner)
    }
    fn mass(&self) -> Real {
        physics::get_mass(self.sim, self.owner)
    }
    fn forward_speed_2d(&self) -> Real {
        physics::forward_speed_2d(self.sim, self.owner)
    }
    fn position(&self) -> Coord3D {
        self.sim
            .object(self.owner)
            .map_or_else(Coord3D::default, |o| *o.thing.get_position())
    }
    fn orientation(&self) -> Real {
        self.sim.object_orientation(self.owner).unwrap_or(0.0)
    }
    fn transform(&self) -> Matrix3D {
        self.sim.object_transform(self.owner).unwrap_or_default()
    }
    fn direction_2d(&self) -> Coord3D {
        self.sim
            .object(self.owner)
            .map_or_else(Coord3D::default, |o| o.thing.get_unit_direction_vector_2d())
    }
    fn layer(&self) -> PathfindLayerEnum {
        self.sim
            .object_layer(self.owner)
            .unwrap_or(PathfindLayerEnum::Ground)
    }
    fn is_kind_of(&self, kind: KindOfType) -> bool {
        self.sim.object_is_kind_of(self.owner, kind) != FALSE
    }
    fn has_status(&self, status: ObjectStatusType) -> bool {
        self.sim
            .object(self.owner)
            .is_some_and(|o| o.status.test(status as Int) != FALSE)
    }
    fn set_braking_status(&mut self, braking: Bool) {
        if braking != FALSE {
            self.sim.set_status(self.owner, ObjectStatusType::Braking);
        } else {
            self.sim.clear_status(self.owner, ObjectStatusType::Braking);
        }
    }
    fn damage_state(&self) -> BodyDamageType {
        self.sim.object_body_damage_state(self.owner)
    }
    fn penalty_state(&self) -> BodyDamageType {
        self.sim.rules().global_data.movement_penalty_damage_state
    }
    fn gravity(&self) -> Real {
        self.sim.rules().global_data.gravity
    }
    /// Force along the heading, in the ground plane.
    fn drive(&mut self, accel_force: Real) {
        let dir = self.direction_2d();
        self.apply_motive_force(accel_force * dir.x, accel_force * dir.y, 0.0);
    }
}

/// `calcSlowDownDist`.
fn slow_down_dist(cur_speed: Real, desired_speed: Real, max_braking: Real) -> Real {
    let delta = cur_speed - desired_speed;
    if delta <= 0.0 {
        return 0.0;
    }
    let dist = ((delta * delta) / gm_fabsf(max_braking)) * 0.5;
    dist * 1.05
}

fn is_nearly_zero(a: Real) -> bool {
    gm_fabsf(a) < 0.001
}

/// `mass * acceleration`, no larger than what reaches the goal speed this frame.
fn limited_force(mass: Real, acceleration: Real, speed_delta: Real) -> Real {
    let force = mass * acceleration;
    let needed = mass * speed_delta;
    if gm_fabsf(force) > gm_fabsf(needed) {
        needed
    } else {
        force
    }
}

/// The engine's `clamp(lo, val, hi)`: a NaN value stays NaN.
fn clamp(lo: Real, val: Real, hi: Real) -> Real {
    if val < lo {
        lo
    } else if val > hi {
        hi
    } else {
        val
    }
}

/// `Vector3::Length2`.
fn length2(v: &Coord3D) -> Real {
    v.x * v.x + v.y * v.y + v.z * v.z
}

/// `tryToRotateVector3D`: turns `cur` towards `goal` by at most `max_angle` (a negative
/// value is a fraction of the angle between them). Returns the turned vector and the
/// angle turned.
fn rotate_vector_towards(max_angle: Real, cur: &Coord3D, goal: &Coord3D) -> (Coord3D, Real) {
    let mut max_angle = max_angle;
    if is_nearly_zero(max_angle) {
        return (*cur, 0.0);
    }
    let mut cur_dir = *cur;
    cur_dir.normalize_ww_vector3();
    let mut goal_dir = *goal;
    goal_dir.normalize_ww_vector3();
    let cosine = cur_dir.x * goal_dir.x + cur_dir.y * goal_dir.y + cur_dir.z * goal_dir.z;
    let mut angle_between = gm_acosf(clamp(-1.0, cosine, 1.0));
    if max_angle < 0.0 {
        max_angle = -max_angle * angle_between;
        if is_nearly_zero(max_angle) {
            return (*cur, 0.0);
        }
    }
    if gm_fabsf(angle_between) <= max_angle {
        return (goal_dir, angle_between);
    }
    let mut axis = Coord3D::default();
    Coord3D::cross_product(&cur_dir, &goal_dir, &mut axis);
    axis.normalize_ww_vector3();
    angle_between = max_angle;
    // `Matrix3D(axis, angle)` and `Rotate_Vector`.
    let (c, s) = (gm_cosf(angle_between), gm_sinf(angle_between));
    let (x, y, z) = (axis.x, axis.y, axis.z);
    let rows = [
        [
            x * x + c * (1.0 - x * x),
            x * y * (1.0 - c) - z * s,
            z * x * (1.0 - c) + y * s,
        ],
        [
            x * y * (1.0 - c) + z * s,
            y * y + c * (1.0 - y * y),
            y * z * (1.0 - c) - x * s,
        ],
        [
            z * x * (1.0 - c) - y * s,
            y * z * (1.0 - c) + x * s,
            z * z + c * (1.0 - z * z),
        ],
    ];
    let rotate =
        |r: &[Real; 3]| r[0] * cur_dir.x + r[1] * cur_dir.y + r[2] * cur_dir.z;
    let turned = Coord3D {
        x: rotate(&rows[0]),
        y: rotate(&rows[1]),
        z: rotate(&rows[2]),
    };
    (turned, angle_between)
}

/// `tryToOrientInThisDirection3D`.
fn orient_towards_3d(w: &mut LocomotorWorld<'_>, max_turn_rate: Real, desired: &Coord3D) {
    let forward = w.transform().get_x_vector();
    let (actual, angle) = rotate_vector_towards(max_turn_rate, &forward, desired);
    if angle != 0.0 {
        let pos = w.position();
        let mtx = build_transform_matrix(&pos, &actual);
        w.sim.set_object_transform_matrix(w.owner, &mtx);
    }
}

/// `calcDirectionToApplyThrust`: the direction whose acceleration, with the current
/// velocity and gravity, hits the goal.
fn thrust_direction(w: &LocomotorWorld<'_>, goal: &Coord3D, max_accel: Real) -> Coord3D {
    let pos = w.position();
    let to_goal = Coord3D {
        x: goal.x - pos.x,
        y: goal.y - pos.y,
        z: goal.z - pos.z,
    };
    if is_nearly_zero(length2(&to_goal)) {
        return w.transform().get_x_vector();
    }
    let mut vel = w.velocity();
    vel.z += w.gravity();
    let mut found = None;
    let dist = zh_common::math::gm_sqrtf(length2(&to_goal));
    let vel_sqr = length2(&vel);
    let vel_mag = zh_common::math::gm_sqrtf(vel_sqr);
    let denom = vel_sqr - max_accel * max_accel;
    if !is_nearly_zero(denom) {
        let mut t = (dist * (vel_mag + max_accel)) / denom;
        let t2 = (dist * (vel_mag - max_accel)) / denom;
        if t >= 0.0 || t2 >= 0.0 {
            if t < 0.0 || (t2 >= 0.0 && t2 < t) {
                t = t2;
            }
            if !is_nearly_zero(t) {
                let mut dir = Coord3D {
                    x: (to_goal.x / t) - vel.x,
                    y: (to_goal.y / t) - vel.y,
                    z: (to_goal.z / t) - vel.z,
                };
                dir.normalize_ww_vector3();
                found = Some(dir);
            }
        }
    }
    found.unwrap_or_else(|| {
        let mut dir = to_goal;
        dir.normalize_ww_vector3();
        dir
    })
}

impl Locomotor {
    /// `getMaxSpeedForCondition` for the object being moved.
    fn speed_limit(&self, w: &LocomotorWorld<'_>) -> Real {
        self.get_max_speed_for_condition(w.damage_state(), w.penalty_state())
    }
    fn turn_limit(&self, w: &LocomotorWorld<'_>) -> Real {
        self.get_max_turn_rate(w.damage_state(), w.penalty_state(), self.is_ultra_accurate())
    }
    fn accel_limit(&self, w: &LocomotorWorld<'_>) -> Real {
        self.get_max_acceleration(w.damage_state(), w.penalty_state())
    }
    fn no_slow_down(&self) -> bool {
        self.get_flag(LocomotorFlag::NoSlowDown) != FALSE
    }
    fn is_braking(&self) -> bool {
        self.get_flag(LocomotorFlag::IsBraking) != FALSE
    }

    /// `locoUpdate_moveTowardsAngle`.
    pub fn move_towards_angle(&mut self, w: &mut LocomotorWorld<'_>, goal_angle: Real) {
        self.set_flag(LocomotorFlag::MaintainPosValid, FALSE);
        if !w.has_physics() || w.physics_flag(IS_STUNNED) {
            return;
        }
        let min_speed = self.get_min_speed();
        let mut desired = w.position();
        if min_speed > 0.0 {
            desired.x += gm_cosf(goal_angle) * min_speed * 2.0;
            desired.y += gm_sinf(goal_angle) * min_speed * 2.0;
            let mut blocked = FALSE;
            self.move_towards_position(w, &desired, 99999.0, min_speed, &mut blocked);
        } else {
            desired.x += gm_cosf(goal_angle) * 1000.0;
            desired.y += gm_sinf(goal_angle) * 1000.0;
            let turning = self.rotate_towards_position(w, &desired, None);
            w.set_turning(turning);
            let here = w.position();
            self.handle_behavior_z(w, &here);
        }
    }

    /// `rotateTowardsPosition`; returns the `PhysicsTurningType`.
    pub fn rotate_towards_position(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        rel_angle: Option<&mut Real>,
    ) -> Int {
        let turn_rate = self.turn_limit(w);
        self.rotate_around_pivot(w, goal, turn_rate, rel_angle)
    }

    /// `setPhysicsOptions`.
    pub fn set_physics_options(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let Some(p) = physics::physics_mut(sim, owner) else {
            return;
        };
        let extra = if self.is_ultra_accurate() != FALSE {
            0.5
        } else {
            0.0
        };
        p.extra_friction = self.template.extra_2d_friction + extra;
        p.set_flag(
            APPLY_FRICTION2D_WHEN_AIRBORNE,
            self.template.apply_2d_friction_airborne,
        );
        p.set_flag(STICK_TO_GROUND, self.template.stick_to_ground);
    }

    /// `locoUpdate_moveTowardsPosition`.
    pub fn move_towards_position(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path_dist_to_goal: Real,
        desired_speed: Real,
        blocked: &mut Bool,
    ) {
        let mut on_path = on_path_dist_to_goal;
        let mut desired_speed = desired_speed;
        self.set_flag(LocomotorFlag::MaintainPosValid, FALSE);
        let max_speed = self.speed_limit(w);
        if desired_speed > max_speed {
            desired_speed = max_speed;
        }
        let stop_dist = (max_speed / self.get_braking()) * max_speed / 2.0;
        if on_path > PATHFIND_CELL_SIZE_F && on_path > stop_dist {
            self.set_flag(LocomotorFlag::IsBraking, FALSE);
            self.braking_factor = 1.0;
        }
        if !w.has_physics() || w.physics_flag(IS_STUNNED) {
            return;
        }
        if self.template.surfaces & LOCOMOTORSURFACE_AIR == 0 {
            let (layer, pos) = (w.layer(), w.position());
            if w
                .valid_movement_terrain(layer, self.template.surfaces, &pos)
                == FALSE
                && self.get_flag(LocomotorFlag::AllowInvalidPosition) == FALSE
                && self.fix_invalid_position(w)
            {
                return;
            }
        }

        let start = w.position();
        let mut dx = goal.x - start.x;
        let mut dy = goal.y - start.y;
        let mut dz = goal.z - start.z;
        let mut dist = zh_common::math::gm_sqrtf(dx * dx + dy * dy);
        if dist > on_path {
            if !w.is_kind_of(KindOfType::Projectile) && dist > 2.0 * on_path {
                self.set_flag(LocomotorFlag::IsBraking, TRUE);
            }
            on_path = dist;
        }

        let pos = w.position();
        let layer = w.layer();
        let mut height = pos.z - w.sim.get_layer_height(pos.x, pos.y, layer);
        if w.has_status(ObjectStatusType::DeckHeightOffset) {
            height -= w.sim.carrier_deck_height(w.owner);
        }
        let airborne = height > -9.0 * w.gravity();

        w.apply_motive_force(0.0, 0.0, 0.0);

        if *blocked != FALSE {
            if desired_speed > w.velocity_magnitude() {
                *blocked = FALSE;
            }
            if airborne && self.template.surfaces & LOCOMOTORSURFACE_AIR != 0 {
                *blocked = FALSE;
            }
        }
        if *blocked != FALSE {
            w.scrub_velocity_2d(desired_speed);
            let turn_rate = self.turn_limit(w);
            if self.template.wander_width_factor == 0.0 {
                let turned = self.rotate_around_pivot(w, goal, turn_rate, None);
                *blocked = if turned != TURN_NONE { TRUE } else { FALSE };
            }
            self.handle_behavior_z(w, goal);
            return;
        }

        if self.template.appearance == LocomotorAppearance::Wings {
            self.set_flag(LocomotorFlag::IsBraking, FALSE);
        }
        let was_braking = w.has_status(ObjectStatusType::Braking);
        w.set_turning(TURN_NONE);
        if self.template.allow_motive_force_airborne != FALSE || !airborne {
            use LocomotorAppearance as A;
            match self.template.appearance {
                A::LegsTwo => self.move_legs(w, goal, on_path, desired_speed),
                A::Climber => self.move_climb(w, goal, on_path, desired_speed),
                A::WheelsFour | A::Motorcycle => self.move_wheels(w, goal, on_path, desired_speed),
                A::Treads => self.move_treads(w, goal, on_path, desired_speed),
                A::Hover => self.move_hover(w, goal, on_path, desired_speed),
                A::Thrust => self.move_thrust(w, goal, on_path, desired_speed),
                // `moveTowardsPositionWings` only adds the circle-to-land code, which
                // this build leaves out.
                A::Wings | A::Other | A::Count => {
                    self.move_other(w, goal, on_path, desired_speed)
                }
            }
        }
        self.handle_behavior_z(w, goal);
        let braking = self.get_flag(LocomotorFlag::IsBraking);
        w.set_braking_status(braking);

        if was_braking {
            let min_vel = PATHFIND_CELL_SIZE_F / LOGICFRAMES_PER_SECOND as Real;
            let mut pos = w.position();
            if w.is_kind_of(KindOfType::Projectile) {
                w.set_braking_status(TRUE);
                dist = zh_common::math::gm_sqrtf(dx * dx + dy * dy + dz * dz);
                let mut vel = w.velocity_magnitude();
                if vel < min_vel {
                    vel = min_vel;
                }
                if vel > dist {
                    vel = dist;
                }
                if dist > 0.001 {
                    dist = 1.0 / dist;
                    dx *= dist;
                    dy *= dist;
                    dz *= dist;
                    pos.x += dx * vel;
                    pos.y += dy * vel;
                    pos.z += dz * vel;
                }
            } else if dist > 0.001 {
                let mut vel = gm_fabsf(w.forward_speed_2d());
                if vel < min_vel {
                    vel = min_vel;
                }
                if vel > dist {
                    vel = dist;
                }
                dist = 1.0 / dist;
                dx *= dist;
                dy *= dist;
                pos.x += dx * vel;
                pos.y += dy * vel;
            }
            w.sim.set_object_position(w.owner, &pos);
        }
    }

    /// The braking state of the tracked and wheeled movers: how hard to brake and the
    /// speed to aim for.
    fn braking_goal_speed(
        &mut self,
        actual_speed: Real,
        slow_down: Real,
        on_path: Real,
        factor_is_one: bool,
    ) -> Real {
        self.braking_factor = reference_div_f32(slow_down, on_path);
        self.braking_factor *= self.braking_factor;
        if self.braking_factor > MAX_BRAKING_FACTOR {
            self.braking_factor = MAX_BRAKING_FACTOR;
        }
        if factor_is_one {
            self.braking_factor = 1.0;
        }
        let mut goal_speed;
        if slow_down > on_path {
            goal_speed = actual_speed - self.get_braking();
            if goal_speed < 0.0 {
                goal_speed = 0.0;
            }
        } else if slow_down > on_path * 0.75 {
            goal_speed = actual_speed - self.get_braking() / 2.0;
            if goal_speed < 0.0 {
                goal_speed = 0.0;
            }
        } else {
            goal_speed = actual_speed;
        }
        goal_speed
    }

    fn move_treads(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        let max_speed = self.speed_limit(w);
        let desired_speed = if desired_speed > max_speed {
            max_speed
        } else {
            desired_speed
        };
        let max_accel = self.accel_limit(w);
        // Not initialized in the reference: a goal within 0.1 of an offset turn pivot
        // leaves it unwritten.
        let mut rel_angle = 0.0;
        let turning = self.rotate_towards_position(w, goal, Some(&mut rel_angle));
        w.set_turning(turning);

        let mut angle_coeff = gm_fabsf(rel_angle) / (PI / 4.0);
        if angle_coeff > 1.0 {
            angle_coeff = 1.0;
        }
        let pos = w.position();
        let dx = pos.x - goal.x;
        let dy = pos.y - goal.y;
        let mut goal_speed = (1.0 - angle_coeff) * desired_speed;
        let actual_speed = w.forward_speed_2d();
        let slow_down_time = actual_speed / self.get_braking();
        let slow_down = (actual_speed / 1.50) * slow_down_time;
        let near = 2.0 * PATHFIND_CELL_SIZE_F;
        if dx * dx + dy * dy < near * near && f64::from(angle_coeff) > 0.05 {
            goal_speed = actual_speed * 0.6;
        }
        if on_path < slow_down && !self.is_braking() && !self.no_slow_down() {
            self.set_flag(LocomotorFlag::IsBraking, TRUE);
            self.braking_factor = 1.1;
        }
        if on_path > PATHFIND_CELL_SIZE_F && f64::from(on_path) > 2.0 * f64::from(slow_down) {
            self.set_flag(LocomotorFlag::IsBraking, FALSE);
        }
        if self.is_braking() {
            goal_speed = self.braking_goal_speed(actual_speed, slow_down, on_path, false);
        }
        let speed_delta = goal_speed - actual_speed;
        if speed_delta != 0.0 {
            let mass = w.mass();
            let accel = if speed_delta > 0.0 {
                max_accel
            } else {
                -self.braking_factor * self.get_braking()
            };
            w.drive(limited_force(mass, accel, speed_delta));
        }
    }

    fn move_wheels(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        let max_speed = self.speed_limit(w);
        let max_turn_rate = self.turn_limit(w);
        let max_accel = self.accel_limit(w);
        let mut desired_speed = if desired_speed > max_speed {
            max_speed
        } else {
            desired_speed
        };
        let mut turn_speed = self.get_min_turn_speed();
        let angle = w.orientation();
        let pos = w.position();
        let mut desired_angle = gm_atan2f(goal.y - pos.y, goal.x - pos.x);
        let mut rel_angle = normalize_angle_cpp(desired_angle - angle);
        let mut move_backwards = false;
        if turn_speed < max_speed / 4.0 {
            turn_speed = max_speed / 4.0;
        }
        let mut actual_speed = w.forward_speed_2d();
        let mut three_point_turn = false;
        if actual_speed == 0.0 {
            self.set_flag(LocomotorFlag::MovingBackwards, FALSE);
            if self.template.can_move_backward != FALSE && gm_fabsf(rel_angle) > PI / 2.0 {
                self.set_flag(LocomotorFlag::MovingBackwards, TRUE);
                self.set_three_point_turn(w, on_path);
            }
        }
        if self.get_flag(LocomotorFlag::MovingBackwards) != FALSE {
            if gm_fabsf(rel_angle) < PI / 2.0 {
                move_backwards = false;
                self.set_flag(LocomotorFlag::MovingBackwards, FALSE);
            } else {
                move_backwards = true;
                self.set_three_point_turn(w, on_path);
                three_point_turn = self.get_flag(LocomotorFlag::DoingThreePointTurn) != FALSE;
                if !three_point_turn {
                    desired_angle = normalize_angle_cpp(desired_angle - PI);
                    rel_angle = normalize_angle_cpp(desired_angle - angle);
                }
            }
        }
        if gm_fabsf(rel_angle) > PI / 20.0 && desired_speed > turn_speed {
            desired_speed = turn_speed;
        }
        let mut goal_speed = desired_speed;
        if move_backwards {
            actual_speed = -actual_speed;
        }
        let slow_down_time = actual_speed / self.get_braking() + 1.0;
        let slow_down = (actual_speed / 1.5) * slow_down_time + actual_speed;
        let mut effective_slow_down = slow_down;
        if effective_slow_down < PATHFIND_CELL_SIZE_F {
            effective_slow_down = PATHFIND_CELL_SIZE_F;
        }

        let project_frames = (LOGICFRAMES_PER_SECOND / 2) as Real;
        if gm_fabsf(rel_angle) > PI / 12.0 {
            let distance = project_frames * (goal_speed + actual_speed) / 2.0;
            let mut target_angle = w.orientation();
            let mut turn_factor = ((goal_speed + actual_speed) / 2.0) / turn_speed;
            if turn_factor > 1.0 {
                turn_factor = 1.0;
            }
            let turn_amount = project_frames * turn_factor * max_turn_rate / 4.0;
            if rel_angle < 0.0 {
                target_angle -= turn_amount;
            } else {
                target_angle += turn_amount;
            }
            let off_x = gm_cosf(target_angle) * distance;
            let off_y = gm_sinf(target_angle) * distance;
            let pos = w.position();
            let next = Coord3D {
                x: pos.x + off_x,
                y: pos.y + off_y,
                z: pos.z,
            };
            let half = Coord3D {
                x: pos.x + off_x / 2.0,
                y: pos.y + off_y / 2.0,
                z: pos.z,
            };
            let layer = w.layer();
            let surfaces = self.template.surfaces;
            if w.valid_movement_terrain(layer, surfaces, &half) == FALSE
                || w.valid_movement_terrain(layer, surfaces, &next) == FALSE
            {
                let turning = self.rotate_towards_position(w, goal, None);
                w.set_turning(turning);
                w.apply_motive_force(0.0, 0.0, 0.0);
                return;
            }
        }

        if on_path < effective_slow_down && !self.is_braking() && !self.no_slow_down() {
            self.set_flag(LocomotorFlag::IsBraking, TRUE);
            self.braking_factor = 1.1;
        }
        if on_path > PATHFIND_CELL_SIZE_F && f64::from(on_path) > 2.0 * f64::from(slow_down) {
            self.set_flag(LocomotorFlag::IsBraking, FALSE);
        }
        if on_path > DONUT_DISTANCE {
            self.donut_timer = donut_deadline(w.sim.frame());
        } else if self.donut_timer < w.sim.frame() {
            self.set_flag(LocomotorFlag::IsBraking, TRUE);
        }
        if self.is_braking() {
            goal_speed = self.braking_goal_speed(actual_speed, slow_down, on_path, true);
        }

        let mut turn_factor = actual_speed / turn_speed;
        if turn_factor < 0.0 {
            turn_factor = -turn_factor;
        }
        if turn_factor > 1.0 {
            turn_factor = 1.0;
        }
        let turn_amount = turn_factor * max_turn_rate;
        let turning = if move_backwards && !three_point_turn {
            let pos = w.position();
            let mut behind = pos;
            behind.x += -(goal.x - pos.x);
            behind.y += -(goal.y - pos.y);
            self.rotate_around_pivot(w, &behind, turn_amount, None)
        } else {
            self.rotate_around_pivot(w, goal, turn_amount, None)
        };
        w.set_turning(turning);

        let mut speed_delta = goal_speed - actual_speed;
        if move_backwards {
            speed_delta = -goal_speed + actual_speed;
        }
        if speed_delta != 0.0 {
            let mass = w.mass();
            let accel = if move_backwards {
                if speed_delta < 0.0 {
                    -max_accel
                } else {
                    self.braking_factor * self.get_braking()
                }
            } else if speed_delta > 0.0 {
                max_accel
            } else {
                -self.braking_factor * self.get_braking()
            };
            w.drive(limited_force(mass, accel, speed_delta));
        }
    }

    /// A three point turn is only worth it far from the goal.
    fn set_three_point_turn(&mut self, w: &LocomotorWorld<'_>, on_path: Real) {
        let radius = w
            .sim
            .object(w.owner)
            .map_or(0.0, |o| o.geometry.get_major_radius());
        let far = if on_path > 5.0 * radius { TRUE } else { FALSE };
        self.set_flag(LocomotorFlag::DoingThreePointTurn, far);
    }

    /// `fixInvalidPosition`: pushes an object out of terrain it may not be on.
    fn fix_invalid_position(&mut self, w: &mut LocomotorWorld<'_>) -> bool {
        if w.is_kind_of(KindOfType::Dozer) {
            return false;
        }
        let mut dx: Int = 0;
        let mut dy: Int = 0;
        for j in -1..2 {
            for i in -1..2 {
                let mut pos = w.position();
                pos.x += i as Real * PATHFIND_CELL_SIZE_F;
                pos.y += j as Real * PATHFIND_CELL_SIZE_F;
                let layer = w.layer();
                if w
                    .valid_movement_terrain(layer, self.template.surfaces, &pos)
                    == FALSE
                {
                    if i < 0 {
                        dx += 1;
                    }
                    if i > 0 {
                        dx -= 1;
                    }
                    if j < 0 {
                        dy += 1;
                    }
                    if j > 0 {
                        dy -= 1;
                    }
                }
            }
        }
        if dx == 0 && dy == 0 {
            return false;
        }
        let correction = Coord3D {
            x: dx as Real * w.mass() / 5.0,
            y: dy as Real * w.mass() / 5.0,
            z: 0.0,
        };
        let mut normalized = correction;
        normalized.normalize();
        let velocity = w.velocity();
        let mut dot = (velocity.x * normalized.x) + (velocity.y * normalized.y);
        if dot > 0.25 {
            return false;
        }
        if dot < 0.0 {
            dot = zh_common::math::gm_sqrtf(-dot);
            normalized.x *= dot * w.mass();
            normalized.y *= dot * w.mass();
            physics::apply_motive_force(w.sim, w.owner, &normalized);
        }
        physics::apply_motive_force(w.sim, w.owner, &correction);
        true
    }

    fn move_legs(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        if self.template.downhill_only != FALSE && w.position().z < goal.z {
            return;
        }
        let max_accel = self.accel_limit(w);
        let max_speed = self.speed_limit(w);
        let desired_speed = if desired_speed > max_speed {
            max_speed
        } else {
            desired_speed
        };
        let actual_speed = w.forward_speed_2d();
        let angle = w.orientation();
        let pos = w.position();
        let mut desired_angle = gm_atan2f(goal.y - pos.y, goal.x - pos.x);
        if self.template.wander_width_factor != 0.0 {
            let angle_limit = PI / 8.0 * self.template.wander_width_factor;
            if self.get_flag(LocomotorFlag::OffsetIncreasing) != FALSE {
                self.angle_offset += self.offset_increment * actual_speed;
                if self.angle_offset > angle_limit {
                    self.set_flag(LocomotorFlag::OffsetIncreasing, FALSE);
                }
            } else {
                self.angle_offset -= self.offset_increment * actual_speed;
                if self.angle_offset < -angle_limit {
                    self.set_flag(LocomotorFlag::OffsetIncreasing, TRUE);
                }
            }
            desired_angle = normalize_angle_cpp(desired_angle + self.angle_offset);
        }
        let rel_angle = normalize_angle_cpp(desired_angle - angle);
        self.move_towards_angle(w, desired_angle);

        let mut angle_coeff = gm_fabsf(rel_angle) / (PI / 4.0);
        if angle_coeff > 1.0 {
            angle_coeff = 1.0;
        }
        let mut goal_speed = (1.0 - angle_coeff) * desired_speed;
        let slow_down = slow_down_dist(actual_speed, self.get_min_speed(), self.get_braking());
        if on_path < slow_down && !self.no_slow_down() {
            goal_speed = self.get_min_speed();
        }
        let speed_delta = goal_speed - actual_speed;
        if speed_delta != 0.0 {
            let mass = w.mass();
            let accel = if speed_delta > 0.0 {
                max_accel
            } else {
                -self.get_braking()
            };
            w.drive(limited_force(mass, accel, speed_delta));
        }
    }

    fn move_climb(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        let max_accel = self.accel_limit(w);
        let max_speed = self.speed_limit(w);
        let mut desired_speed = if desired_speed > max_speed {
            max_speed
        } else {
            desired_speed
        };
        let mut move_backwards = false;
        let pos = w.position();
        let dz = pos.z - goal.z;
        if dz * dz > PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F {
            self.set_flag(LocomotorFlag::Climbing, TRUE);
        }
        if gm_fabsf(dz) < 1.0 {
            self.set_flag(LocomotorFlag::Climbing, FALSE);
        }
        if self.get_flag(LocomotorFlag::Climbing) != FALSE {
            let mut delta = *goal;
            delta.x -= pos.x;
            delta.y -= pos.y;
            delta.z = 0.0;
            delta.normalize();
            delta.x += pos.x;
            delta.y += pos.y;
            delta.z = w.sim.terrain().get_ground_height(delta.x, delta.y, None);
            if f64::from(delta.z) < f64::from(pos.z) - 0.1 {
                move_backwards = true;
            }
            let mut slope = gm_fabsf(delta.z - pos.z);
            if slope < 1.0 {
                slope = 1.0;
            }
            if slope > 1.0 {
                desired_speed /= slope * 4.0;
            }
        }
        self.set_flag(
            LocomotorFlag::MovingBackwards,
            if move_backwards { TRUE } else { FALSE },
        );
        let angle = w.orientation();
        let here = w.position();
        let mut desired_angle = gm_atan2f(goal.y - here.y, goal.x - here.x);
        let mut rel_angle = normalize_angle_cpp(desired_angle - angle);
        if move_backwards {
            desired_angle = normalize_angle_cpp(desired_angle - PI);
            rel_angle = normalize_angle_cpp(desired_angle - angle);
        }
        self.move_towards_angle(w, desired_angle);

        let mut angle_coeff = gm_fabsf(rel_angle) / (PI / 4.0);
        if angle_coeff > 1.0 {
            angle_coeff = 1.0;
        }
        let mut goal_speed = (1.0 - angle_coeff) * desired_speed;
        let mut actual_speed = w.forward_speed_2d();
        if move_backwards {
            actual_speed = -actual_speed;
        }
        let slow_down = slow_down_dist(actual_speed, self.get_min_speed(), self.get_braking());
        if on_path < slow_down && !self.no_slow_down() {
            goal_speed = self.get_min_speed();
        }
        let mut speed_delta = goal_speed - actual_speed;
        if move_backwards {
            speed_delta = -goal_speed + actual_speed;
        }
        if speed_delta != 0.0 {
            let mass = w.mass();
            let accel = if move_backwards {
                if speed_delta < 0.0 {
                    -max_accel
                } else {
                    self.get_braking()
                }
            } else if speed_delta > 0.0 {
                max_accel
            } else {
                -self.get_braking()
            };
            w.drive(limited_force(mass, accel, speed_delta));
        }
    }

    fn move_hover(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        self.move_other(w, goal, on_path, desired_speed);
        let pos = w.position();
        let wet = w.sim.terrain().is_underwater(pos.x, pos.y, None, None) != FALSE;
        if wet != (self.get_flag(LocomotorFlag::OverWater) != FALSE) {
            if wet {
                self.set_flag(LocomotorFlag::OverWater, TRUE);
                w.sim
                    .set_model_condition_state(w.owner, ModelConditionFlagType::OverWater);
            } else {
                self.set_flag(LocomotorFlag::OverWater, FALSE);
                w.sim
                    .clear_model_condition_state(w.owner, ModelConditionFlagType::OverWater);
            }
        }
    }

    fn move_thrust(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        let mut max_forward = self.speed_limit(w);
        let mut desired_speed = clamp(self.get_min_speed(), desired_speed, max_forward);
        let actual_forward = physics::forward_speed_3d(w.sim, w.owner);
        if self.get_braking() > 0.0 {
            let slow_down =
                slow_down_dist(actual_forward, self.get_min_speed(), self.get_braking());
            if on_path < slow_down && !self.no_slow_down() {
                desired_speed = self.get_min_speed();
            }
        }
        let mut local_goal = *goal;
        let pos = w.position();
        if self.preferred_height != 0.0 && self.get_flag(LocomotorFlag::PreciseZPos) == FALSE {
            let surface = surface_height(w, pos.x, pos.y);
            local_goal.z = self.preferred_height + surface;
            let mut delta = local_goal.z - pos.z;
            delta *= self.preferred_height_damping;
            local_goal.z = pos.z + delta;
        }
        let forward = w.transform().get_x_vector();
        let speed_delta = desired_speed - actual_forward;
        let max_accel = if speed_delta > 0.0 || self.get_braking() == 0.0 {
            self.accel_limit(w)
        } else {
            -self.get_braking()
        };
        let mut max_turn_rate = self.turn_limit(w);
        let desired_dir = thrust_direction(w, &local_goal, max_accel);
        let max_thrust_angle = if max_turn_rate > 0.0 {
            self.template.max_thrust_angle
        } else {
            0.0
        };
        let (thrust_dir, thrust_angle) =
            rotate_vector_towards(max_thrust_angle, &forward, &desired_dir);

        if !is_nearly_zero(w.velocity_magnitude()) {
            let mut vel = w.velocity();
            let mut adjust = true;
            if w.has_status(ObjectStatusType::Braking) {
                vel = Coord3D {
                    x: goal.x - pos.x,
                    y: goal.y - pos.y,
                    z: goal.z - pos.z,
                };
                if is_nearly_zero(length2(&vel)) {
                    adjust = false;
                }
                max_turn_rate *= 3.0;
            }
            if adjust {
                orient_towards_3d(w, max_turn_rate, &vel);
            }
        }

        if speed_delta != 0.0 || thrust_angle != 0.0 {
            if max_forward <= 0.0 {
                max_forward = 0.01;
            }
            let damping = clamp(0.0, max_accel / max_forward, 1.0);
            let cur = w.velocity();
            let accel = Coord3D {
                x: thrust_dir.x * max_accel - cur.x * damping,
                y: thrust_dir.y * max_accel - cur.y * damping,
                z: thrust_dir.z * max_accel - cur.z * damping,
            };
            let mass = w.mass();
            w.apply_motive_force(mass * accel.x, mass * accel.y, mass * accel.z);
        }
    }

    fn move_other(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        on_path: Real,
        desired_speed: Real,
    ) {
        let max_accel = self.accel_limit(w);
        let max_speed = self.speed_limit(w);
        let mut goal_speed = if desired_speed > max_speed {
            max_speed
        } else {
            desired_speed
        };
        let actual_speed = w.forward_speed_2d();
        let pos = w.position();
        let mut dir = w.direction_2d();
        let slide = goal_speed * self.template.ultra_accurate_slide_factor;
        if self.is_ultra_accurate() != FALSE
            && gm_fabsf(goal.y - pos.y) <= slide
            && gm_fabsf(goal.x - pos.x) <= slide
        {
            w.set_turning(TURN_NONE);
            dir.x = goal.x - pos.x;
            dir.y = goal.y - pos.y;
            dir.z = 0.0;
            dir.normalize();
        } else {
            let turning = self.rotate_towards_position(w, goal, None);
            w.set_turning(turning);
        }
        if !self.no_slow_down() {
            let slow_down = slow_down_dist(actual_speed, self.get_min_speed(), self.get_braking());
            if on_path < slow_down {
                goal_speed = self.get_min_speed();
            }
        }
        let speed_delta = goal_speed - actual_speed;
        if speed_delta != 0.0 {
            let mass = w.mass();
            let accel = if speed_delta > 0.0 {
                max_accel
            } else {
                -self.get_braking()
            };
            let force = limited_force(mass, accel, speed_delta);
            w.apply_motive_force(force * dir.x, force * dir.y, 0.0);
        }
    }

    /// `calcLiftToUseAtPt`.
    fn lift_to_use(&self, w: &LocomotorWorld<'_>, cur_z: Real, preferred_height: Real) -> Real {
        let max_gross_lift = self.get_max_lift(w.damage_state(), w.penalty_state());
        let gravity = w.gravity();
        let mut max_net_lift = max_gross_lift + gravity;
        if max_net_lift < 0.0 {
            max_net_lift = 0.0;
        }
        let vel_z = w.velocity().z;
        let ultra = self.is_ultra_accurate() != FALSE;
        let max_accel = if ultra {
            if vel_z < 0.0 {
                2.0 * max_net_lift
            } else {
                -2.0 * max_net_lift
            }
        } else if vel_z < 0.0 {
            max_net_lift
        } else {
            gravity
        };
        let desired_accel = if gm_fabsf(max_accel) > 0.001 {
            let delta_z = preferred_height - cur_z;
            let brake_dist = (vel_z * vel_z) / gm_fabsf(max_accel);
            if gm_fabsf(brake_dist) > gm_fabsf(delta_z) {
                max_accel
            } else if gm_fabsf(vel_z) > self.template.speed_limit_z {
                self.template.speed_limit_z - vel_z
            } else {
                2.0 * (delta_z - vel_z)
            }
        } else {
            0.0
        };
        let mut lift = desired_accel - gravity;
        if ultra {
            if lift > 3.0 * max_gross_lift {
                lift = 3.0 * max_gross_lift;
            } else if lift < -max_gross_lift {
                lift = -max_gross_lift;
            }
        } else if lift > max_gross_lift {
            lift = max_gross_lift;
        } else if lift < 0.0 {
            lift = 0.0;
        }
        lift
    }

    /// `rotateObjAroundLocoPivot`.
    fn rotate_around_pivot(
        &mut self,
        w: &mut LocomotorWorld<'_>,
        goal: &Coord3D,
        max_turn_rate: Real,
        rel_angle: Option<&mut Real>,
    ) -> Int {
        let angle = w.orientation();
        let mut offset = self.template.turn_pivot_offset;
        if self.is_braking() {
            offset = 0.0;
        }
        let limit = |amount: Real| {
            if amount > max_turn_rate {
                (max_turn_rate, TURN_POSITIVE)
            } else if amount < -max_turn_rate {
                (-max_turn_rate, TURN_NEGATIVE)
            } else {
                (amount, TURN_NONE)
            }
        };
        if offset != 0.0 {
            let radius = w
                .sim
                .object(w.owner)
                .map_or(0.0, |o| o.geometry.get_bounding_circle_radius());
            let pivot_offset = offset * radius;
            let mut pivot = w.position();
            let dir = w.direction_2d();
            pivot.x += dir.x * pivot_offset;
            pivot.y += dir.y * pivot_offset;
            let dx = goal.x - pivot.x;
            let dy = goal.y - pivot.y;
            if gm_fabsf(dx) < 0.1 && gm_fabsf(dy) < 0.1 {
                return TURN_NONE;
            }
            let amount = normalize_angle_cpp(gm_atan2f(dy, dx) - angle);
            if let Some(out) = rel_angle {
                *out = amount;
            }
            let (amount, turn) = limit(amount);
            let mut tmp = Matrix3D::identity();
            tmp.translate(pivot.x, pivot.y, 0.0);
            tmp.in_place_pre_rotate_z(amount);
            tmp.translate(-pivot.x, -pivot.y, 0.0);
            let mtx = Matrix3D::mul(&tmp, &w.transform());
            w.sim.set_object_transform_matrix(w.owner, &mtx);
            turn
        } else {
            let pos = w.position();
            let amount = normalize_angle_cpp(gm_atan2f(goal.y - pos.y, goal.x - pos.x) - angle);
            if let Some(out) = rel_angle {
                *out = amount;
            }
            let (amount, turn) = limit(amount);
            w.sim
                .set_object_orientation(w.owner, normalize_angle_cpp(angle + amount));
            turn
        }
    }

    /// `handleBehaviorZ`: returns whether it has to be called every frame.
    pub fn handle_behavior_z(&mut self, w: &mut LocomotorWorld<'_>, goal: &Coord3D) -> Bool {
        use LocomotorBehaviorZ as Z;
        let behavior = self.template.behavior_z;
        match behavior {
            Z::NoZMotiveForce | Z::Count => return FALSE,
            Z::SeaLevel => {
                let held = w
                    .sim
                    .object(w.owner)
                    .is_some_and(|o| o.is_disabled_by_type(DisabledType::Held) != FALSE);
                if !held {
                    let mut pos = w.position();
                    let mut water_z = 0.0;
                    if w.sim
                        .terrain()
                        .is_underwater(pos.x, pos.y, Some(&mut water_z), None)
                        != FALSE
                    {
                        pos.z = water_z;
                    } else {
                        let layer = w.layer();
                        pos.z = w.sim.get_layer_height(pos.x, pos.y, layer);
                    }
                    w.sim.set_object_position(w.owner, &pos);
                }
            }
            Z::FixedSurfaceRelativeHeight | Z::FixedAbsoluteHeight => {
                let mut pos = w.position();
                let relative = behavior == Z::FixedSurfaceRelativeHeight;
                let surface = if relative {
                    surface_height(w, pos.x, pos.y)
                } else {
                    0.0
                };
                pos.z = self.preferred_height + surface;
                w.sim.set_object_position(w.owner, &pos);
            }
            Z::RelativeToGroundAndBuildings => {
                let mut pos = w.position();
                let surface = w.seams.ground_or_structure_height(pos.x, pos.y);
                pos.z = self.preferred_height + surface;
                w.sim.set_object_position(w.owner, &pos);
            }
            Z::RelativeToHighestLayer | Z::SurfaceRelativeHeight | Z::AbsoluteHeight => {
                let precise = self.get_flag(LocomotorFlag::PreciseZPos) != FALSE;
                if self.preferred_height != 0.0 || precise {
                    let pos = w.position();
                    let surface = match behavior {
                        Z::RelativeToHighestLayer => {
                            let mut layer = w.layer();
                            if layer == PathfindLayerEnum::Ground {
                                layer = w.sim.get_highest_layer_for_destination(&pos, FALSE);
                            }
                            let mut normal = Coord3D::default();
                            w.sim.terrain().get_layer_height(
                                pos.x,
                                pos.y,
                                layer,
                                Some(&mut normal),
                                FALSE,
                            )
                        }
                        Z::SurfaceRelativeHeight => surface_height(w, pos.x, pos.y),
                        _ => 0.0,
                    };
                    let mut preferred = self.preferred_height + surface;
                    if precise {
                        preferred = goal.z;
                    }
                    let mut delta = preferred - pos.z;
                    delta *= self.preferred_height_damping;
                    preferred = pos.z + delta;
                    let lift = self.lift_to_use(w, pos.z, preferred);
                    if lift != 0.0 {
                        let force_z = lift * w.mass();
                        w.apply_motive_force(0.0, 0.0, force_z);
                    }
                }
            }
        }
        TRUE
    }

    /// `locoUpdate_maintainCurrentPosition`: returns whether it has to be called every
    /// frame.
    pub fn maintain_current_position(&mut self, w: &mut LocomotorWorld<'_>) -> Bool {
        if self.get_flag(LocomotorFlag::MaintainPosValid) == FALSE {
            self.maintain_pos = w.position();
            self.set_flag(LocomotorFlag::MaintainPosValid, TRUE);
        }
        self.donut_timer = donut_deadline(w.sim.frame());
        self.set_flag(LocomotorFlag::IsBraking, FALSE);
        if !w.has_physics() {
            return TRUE;
        }
        use LocomotorAppearance as A;
        let mut constant = TRUE;
        match self.template.appearance {
            A::Thrust => {
                let (goal, speed) = (self.maintain_pos, self.get_min_speed());
                self.move_thrust(w, &goal, 0.0, speed);
            }
            A::LegsTwo | A::Climber | A::WheelsFour | A::Motorcycle | A::Treads => {
                self.maintain_other(w);
                constant = FALSE;
            }
            A::Hover => self.maintain_hover(w),
            A::Wings => self.maintain_wings(w),
            A::Other | A::Count => self.maintain_other(w),
        }
        let goal = self.maintain_pos;
        if self.handle_behavior_z(w, &goal) != FALSE {
            constant = TRUE;
        }
        constant
    }

    /// Circles the held point.
    fn maintain_wings(&mut self, w: &mut LocomotorWorld<'_>) {
        w.set_turning(TURN_NONE);
        // The height is only asked, and cached, for a motive object.
        if physics::is_motive(w.sim, w.owner)
            && w.sim.object(w.owner).is_some_and(|o| {
                o.thing.is_above_terrain(w.sim.terrain()) != FALSE
            })
        {
            let mut turn_radius = self.template.circling_radius;
            if turn_radius == 0.0 {
                turn_radius = self
                    .calc_min_turn_radius(w.damage_state(), w.penalty_state())
                    .0;
            }
            let pos = w.position();
            let dx = self.maintain_pos.x - pos.x;
            let dy = self.maintain_pos.y - pos.y;
            let mut toward = if is_nearly_zero(dx) && is_nearly_zero(dy) {
                w.orientation()
            } else {
                gm_atan2f(dy, dx)
            };
            let mut aim = PI - PI / 8.0;
            if turn_radius < 0.0 {
                turn_radius = -turn_radius;
                aim = -aim;
            }
            toward += aim;
            let mut desired = self.maintain_pos;
            desired.x += gm_cosf(toward) * turn_radius;
            desired.y += gm_sinf(toward) * turn_radius;
            let speed = self.get_min_speed();
            self.move_other(w, &desired, 0.0, speed);
        }
    }

    fn maintain_hover(&mut self, w: &mut LocomotorWorld<'_>) {
        w.set_turning(TURN_NONE);
        if physics::is_motive(w.sim, w.owner) {
            let max_accel = self.accel_limit(w);
            let actual_speed = w.forward_speed_2d();
            // The engine's `max(a, b)`: a NaN minimum speed stays NaN.
            let min = self.get_min_speed();
            let min_speed = if 1.0e-10 > min { 1.0e-10 } else { min };
            let speed_delta = min_speed - actual_speed;
            if gm_fabsf(speed_delta) > min_speed {
                let mass = w.mass();
                let accel = if speed_delta > 0.0 {
                    max_accel
                } else {
                    -self.get_braking()
                };
                w.drive(limited_force(mass, accel, speed_delta));
            }
        }
    }

    fn maintain_other(&mut self, w: &mut LocomotorWorld<'_>) {
        w.set_turning(TURN_NONE);
        if physics::is_motive(w.sim, w.owner) {
            w.scrub_velocity_2d(0.0);
        }
    }
}

/// `getSurfaceHtAtPt`: the water surface under water, else the ground.
fn surface_height(w: &LocomotorWorld<'_>, x: Real, y: Real) -> Real {
    let mut water_z = 0.0;
    let mut ground_z = 0.0;
    if w.sim
        .terrain()
        .is_underwater(x, y, Some(&mut water_z), Some(&mut ground_z))
        != FALSE
    {
        0.0 + water_z
    } else {
        0.0 + ground_z
    }
}
