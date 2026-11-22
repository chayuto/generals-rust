//! `PKT-143`: Locomotor templates, stores, runtime overrides, and snapshots.

use std::collections::BTreeMap;

use crate::gamelogic::global_data::BodyDamageType;
use crate::rules::combat_types::{
    LocomotorSurfaceTypeMask, LOCOMOTOR_SURFACE_TYPE_NAMES,
};
use zh_common::ini::{
    IniError, IniReader, LOGICFRAMES_PER_SECOND, RADS_PER_DEGREE, SECONDS_PER_LOGICFRAME_REAL,
};
use zh_common::math::{Coord3D, PI};
use zh_common::random_value::PrngState;
use zh_common::types::{Bool, Int, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const BIGNUM: Real = 99999.0_f32;

#[inline]
fn parse_velocity(value: &str) -> Result<Real, IniError> {
    let v = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?;
    Ok(v * SECONDS_PER_LOGICFRAME_REAL)
}

#[inline]
fn parse_acceleration(value: &str) -> Result<Real, IniError> {
    let v = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?;
    Ok(v * (SECONDS_PER_LOGICFRAME_REAL * SECONDS_PER_LOGICFRAME_REAL))
}

#[inline]
fn parse_angular_velocity(value: &str) -> Result<Real, IniError> {
    let v = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?;
    Ok(v * (SECONDS_PER_LOGICFRAME_REAL * RADS_PER_DEGREE))
}

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
            preferred_height_damping: 1.0, circling_radius: 0.0, speed_limit_z: BIGNUM,
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
        for line in text.lines().skip(1) {
            let line = line.trim();
            if line.eq_ignore_ascii_case("End") { break; }
            let Some((key, value)) = line.split_once('=') else { continue };
            let key = key.trim();
            let value = value.trim();
            match key {
                "Surfaces" => {
                    self.surfaces = 0;
                    for name in value.split_whitespace() {
                        let idx = IniReader::scan_index_list(name, &LOCOMOTOR_SURFACE_TYPE_NAMES)?;
                        self.surfaces |= 1 << idx;
                    }
                }
                "Speed" => self.max_speed = parse_velocity(value)?,
                "SpeedDamaged" => self.max_speed_damaged = parse_velocity(value)?,
                "TurnRate" => self.max_turn_rate = parse_angular_velocity(value)?,
                "TurnRateDamaged" => self.max_turn_rate_damaged = parse_angular_velocity(value)?,
                "Acceleration" => self.acceleration = parse_acceleration(value)?,
                "AccelerationDamaged" => self.acceleration_damaged = parse_acceleration(value)?,
                "Lift" => self.lift = parse_acceleration(value)?,
                "LiftDamaged" => self.lift_damaged = parse_acceleration(value)?,
                "Braking" => self.braking = parse_acceleration(value)?,
                "MinSpeed" => self.min_speed = parse_velocity(value)?,
                "MinTurnSpeed" => self.min_turn_speed = parse_velocity(value)?,
                "PreferredHeight" => self.preferred_height = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "PreferredHeightDamping" => self.preferred_height_damping = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "CirclingRadius" => self.circling_radius = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "Extra2DFriction" => self.extra_2d_friction = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * SECONDS_PER_LOGICFRAME_REAL,
                "SpeedLimitZ" => self.speed_limit_z = parse_velocity(value)?,
                "MaxThrustAngle" => self.max_thrust_angle = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * RADS_PER_DEGREE,
                "ZAxisBehavior" => self.behavior_z = LocomotorBehaviorZ::from_i32(
                    IniReader::scan_index_list(value, &LOCOMOTOR_BEHAVIOR_Z_NAMES)?
                ).unwrap_or(LocomotorBehaviorZ::NoZMotiveForce),
                "Appearance" => self.appearance = LocomotorAppearance::from_i32(
                    IniReader::scan_index_list(value, &LOCOMOTOR_APPEARANCE_NAMES)?
                ).unwrap_or(LocomotorAppearance::Other),
                "GroupMovementPriority" => {
                    let v = IniReader::scan_index_list(value, &LOCOMOTOR_PRIORITY_NAMES)?;
                    self.move_priority = match v { 0 => LocomotorPriority::MovesBack, 2 => LocomotorPriority::MovesFront, _ => LocomotorPriority::MovesMiddle };
                }
                "AccelerationPitchLimit" | "DecelerationPitchLimit" => {
                    let v = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * RADS_PER_DEGREE;
                    if key == "AccelerationPitchLimit" { self.accel_pitch_limit = v; } else { self.decel_pitch_limit = v; }
                }
                "BounceAmount" => self.bounce_kick = parse_angular_velocity(value)?,
                "PitchStiffness" => self.pitch_stiffness = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "RollStiffness" => self.roll_stiffness = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "PitchDamping" => self.pitch_damping = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "RollDamping" => self.roll_damping = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ThrustRoll" => self.thrust_roll = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ThrustWobbleRate" => self.wobble_rate = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ThrustMinWobble" => self.min_wobble = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ThrustMaxWobble" => self.max_wobble = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "PitchInDirectionOfZVelFactor" => self.pitch_by_z_vel_coef = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ForwardVelocityPitchFactor" => self.forward_vel_coef = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "LateralVelocityRollFactor" => self.lateral_vel_coef = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ForwardAccelerationPitchFactor" => self.forward_accel_coef = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "LateralAccelerationRollFactor" => self.lateral_accel_coef = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "UniformAxialDamping" => self.uniform_axial_damping = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "TurnPivotOffset" => self.turn_pivot_offset = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "Apply2DFrictionWhenAirborne" => self.apply_2d_friction_airborne = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "DownhillOnly" => self.downhill_only = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "AllowAirborneMotiveForce" => self.allow_motive_force_airborne = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "LocomotorWorksWhenDead" => self.locomotor_works_when_dead = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "AirborneTargetingHeight" => self.airborne_targeting_height = value.parse::<Int>().map_err(|_| IniError::InvalidData)?,
                "StickToGround" => self.stick_to_ground = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "CanMoveBackwards" => self.can_move_backward = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "HasSuspension" => self.has_suspension = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "FrontWheelTurnAngle" => self.wheel_turn_angle = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * RADS_PER_DEGREE,
                "MaximumWheelExtension" => self.maximum_wheel_extension = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "MaximumWheelCompression" => self.maximum_wheel_compression = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "CloseEnoughDist" => self.close_enough_dist = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "CloseEnoughDist3D" => self.is_close_enough_dist3d = IniReader::scan_bool(value).map_err(|_| IniError::InvalidData)?,
                "SlideIntoPlaceTime" => self.ultra_accurate_slide_factor = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)? * 30.0_f32 / 1000.0_f32,
                "WanderWidthFactor" => self.wander_width_factor = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "WanderLengthFactor" => self.wander_length_factor = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "WanderAboutPointRadius" => self.wander_about_point_radius = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "RudderCorrectionDegree" => self.rudder_correction_degree = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "RudderCorrectionRate" => self.rudder_correction_rate = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ElevatorCorrectionDegree" => self.elevator_correction_degree = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                "ElevatorCorrectionRate" => self.elevator_correction_rate = IniReader::scan_real(value).map_err(|_| IniError::InvalidData)?,
                _ => {}
            }
        }
        if let Some(first) = text.lines().next().and_then(|l| l.split_whitespace().nth(1)) {
            self.name = first.to_owned();
        }
        self.validate()
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
        if self.decel_pitch_limit == 0.0 { self.decel_pitch_limit = self.accel_pitch_limit; }
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
            donut_timer: frame.wrapping_add((2.5_f32 * LOGICFRAMES_PER_SECOND as f32) as u32),
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
        self.donut_timer = frame.wrapping_add((2.5_f32 * LOGICFRAMES_PER_SECOND as f32) as u32);
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
