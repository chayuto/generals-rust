//! `PKT-145`: Deterministic turret AI state machine, INI configuration, and snapshot serialization
//! (`GeneralsMD/Code/GameEngine/Include/GameLogic/TurretAI.h` and
//! `GeneralsMD/Code/GameEngine/Source/GameLogic/AI/TurretAI.cpp`).

#![allow(unused_variables, dead_code)]

use crate::object::weapon_set::{CanAttackResult, WeaponSlotType, WEAPONSLOT_COUNT};
use crate::rules::kind_of::KindOfType;
use zh_common::core::state_machine::{
    StateExitType, StateId, StateReturnType, EXIT_MACHINE_WITH_FAILURE, EXIT_MACHINE_WITH_SUCCESS,
    INVALID_STATE_ID, MACHINE_DONE_STATE_ID, MAX_TRANSITION_DEPTH, STATE_SLEEP_FOREVER_FRAMES,
};
use zh_common::ini::{
    FieldParseEntry, FieldUserData, IniError, IniLoadType, IniReader, LOGICFRAMES_PER_SECOND,
};
use zh_common::math::{gm_asinf, gm_fabsf, Coord3D, PI};
use zh_common::types::{
    Bool, Int, ObjectId, Real, TeamId, UnsignedInt, UpdateSleepTime, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const DEFAULT_TURN_RATE: Real = 0.01_f32;
pub const DEFAULT_PITCH_RATE: Real = 0.01_f32;
pub const WAIT_INDEFINITELY: UnsignedInt = 0xffff_ffff;
const ENABLE_SWEEP_FRAME_COUNT: UnsignedInt = 3;
const AIM_REL_THRESH: Real = 0.035_f32;
const WEAPON_SLOT_NAMES: [&str; WEAPONSLOT_COUNT] = ["PRIMARY", "SECONDARY", "TERTIARY"];

/// Normalizes `angle` into `(-PI, PI]` matching `GameCommon.cpp::normalizeAngle` bit-for-bit.
#[inline]
pub fn normalize_angle_cpp(mut angle: Real) -> Real {
    if angle.is_nan() {
        return 0.0_f32;
    }
    let two_pi = 2.0_f32 * PI;
    while angle > PI {
        angle -= two_pi;
    }
    while angle <= -PI {
        angle += two_pi;
    }
    angle
}

/// Identifies a turret mount slot on an object (`WhichTurretType` in `GameCommon.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum WhichTurretType {
    #[default]
    Invalid = -1,
    Main = 0,
    Alt = 1,
}

impl WhichTurretType {
    pub const MAX_TURRETS: usize = 2;

    #[inline]
    pub const fn from_i32(v: Int) -> Self {
        match v {
            0 => Self::Main,
            1 => Self::Alt,
            _ => Self::Invalid,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// States of [`TurretStateMachine`] matching `enum TurretStateType`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TurretStateType {
    #[default]
    Idle = 0,
    IdleScan = 1,
    Aim = 2,
    Fire = 3,
    Recenter = 4,
    Hold = 5,
}

impl TurretStateType {
    pub const NUM_STATES: usize = 6;

    #[inline]
    pub const fn from_state_id(id: StateId) -> Option<Self> {
        match id {
            0 => Some(Self::Idle),
            1 => Some(Self::IdleScan),
            2 => Some(Self::Aim),
            3 => Some(Self::Fire),
            4 => Some(Self::Recenter),
            5 => Some(Self::Hold),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_state_id(self) -> StateId {
        self as StateId
    }
}

/// Target mode for a turret matching `enum TurretTargetType`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TurretTargetType {
    #[default]
    None = 0,
    Object = 1,
    Position = 2,
}

impl TurretTargetType {
    #[inline]
    pub const fn from_i32(v: Int) -> Self {
        match v {
            1 => Self::Object,
            2 => Self::Position,
            _ => Self::None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Construction/validation errors for [`TurretAI`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TurretAIError {
    NoControlledWeaponSlots,
}

/// Outcome of stepping `AIAttackFireWeaponState` inside [`TurretStateType::Fire`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FireStateStep {
    pub status: StateReturnType,
    pub did_fire: Bool,
    pub transfer_victim: Option<ObjectId>,
}

impl Default for FireStateStep {
    fn default() -> Self {
        Self {
            status: StateReturnType::Failure,
            did_fire: FALSE,
            transfer_victim: None,
        }
    }
}

/// World and owner object interface consumed by [`TurretAI`] and [`TurretStateMachine`].
///
/// Decouples turret simulation from global singletons so `TurretAI` runs identically in
/// Phase 3 Gate V2 differential harnesses and Phase 5–6 `SimContext`/`AIUpdateInterface`.
pub trait TurretWorldAccess {
    fn current_frame(&self) -> UnsignedInt;
    /// `GameLogicRandomValue`.
    fn random_int(&mut self, lo: Int, hi: Int) -> Int;
    /// `GameLogicRandomValueReal`.
    fn random_real(&mut self, lo: Real, hi: Real) -> Real;
    fn object_exists(&self, id: ObjectId) -> bool;
    fn is_object_effectively_dead(&self, id: ObjectId) -> bool;
    fn get_object_team(&self, id: ObjectId) -> Option<TeamId>;
    fn get_object_position(&self, id: ObjectId) -> Option<Coord3D>;
    fn is_object_kind_of(&self, id: ObjectId, kind: KindOfType) -> Bool;
    fn is_owner_under_construction(&self, owner: ObjectId) -> Bool;
    fn set_model_condition_turret_rotate(&mut self, owner: ObjectId, set: Bool);
    fn react_to_turret_change(
        &mut self,
        owner: ObjectId,
        turret: WhichTurretType,
        old_angle: Real,
        new_pitch: Real,
    );
    fn get_owner_current_weapon_slot(&self, owner: ObjectId) -> Option<WeaponSlotType>;
    fn get_weapon_attack_range(&self, owner: ObjectId, slot: WeaponSlotType) -> Real;
    fn is_weapon_in_range_of_object(
        &self,
        owner: ObjectId,
        slot: WeaponSlotType,
        target: ObjectId,
    ) -> Bool;
    fn is_weapon_in_range_of_pos(
        &self,
        owner: ObjectId,
        slot: WeaponSlotType,
        pos: &Coord3D,
    ) -> Bool;
    fn are_turrets_linked(&self, owner: ObjectId) -> Bool;
    fn get_owner_ai_goal_object(&mut self, owner: ObjectId) -> ObjectId;
    fn is_owner_able_to_attack(&mut self, owner: ObjectId) -> Bool;
    fn can_owner_attack_specific_object(
        &mut self,
        owner: ObjectId,
        target: ObjectId,
        force_attacking: Bool,
    ) -> CanAttackResult;
    fn add_targeter(&mut self, target: ObjectId, targeter: ObjectId, add: Bool);
    fn is_temporarily_preventing_aim_success(&self, target: ObjectId) -> Bool;
    fn get_bridge_closest_attack_point(&self, owner: ObjectId, bridge: ObjectId) -> Coord3D;
    fn get_relative_angle_2d(&self, owner: ObjectId, pos: &Coord3D) -> Real;
    fn get_vector_to_center_3d(&self, owner: ObjectId, pos: &Coord3D) -> Coord3D;
    fn get_owner_max_height_above_position(&self, owner: ObjectId) -> Real;
    fn get_next_idle_mood_target_frame(&mut self, owner: ObjectId) -> UnsignedInt;
    fn reset_next_mood_check_time(&mut self, owner: ObjectId);
    fn get_turret_sync(&mut self, owner: ObjectId) -> WhichTurretType;
    fn set_turret_sync(&mut self, owner: ObjectId, tur: WhichTurretType);
    fn is_idle_mood_ignored(&mut self, owner: ObjectId) -> Bool;
    fn acquire_idle_mood_target(&mut self, owner: ObjectId) -> ObjectId;
    fn choose_best_weapon_for_target(&mut self, owner: ObjectId, target: ObjectId);
    fn out_of_weapon_range_object(&mut self, owner: ObjectId, victim: ObjectId) -> Bool;
    fn on_enter_fire_state(&mut self, owner: ObjectId, victim: ObjectId) -> StateReturnType;
    fn on_update_fire_state(
        &mut self,
        owner: ObjectId,
        turret: WhichTurretType,
        is_attacking_object: Bool,
        victim: ObjectId,
        goal_pos: &Coord3D,
        is_slot_ok_to_fire: Bool,
    ) -> FireStateStep;
    fn on_exit_fire_state(&mut self, owner: ObjectId);
    /// Called before a question or action that can come back to this turret,
    /// and [`Self::reclaim_turret`] after it: a world whose owner holds the
    /// turret swaps the running one in and out.
    fn lend_turret(&mut self, _turret: &mut TurretAI) {}
    fn reclaim_turret(&mut self, _turret: &mut TurretAI) {}

    fn on_start_rot_or_pitch_sound(&mut self, _owner: ObjectId, _turret: WhichTurretType) {}
    fn on_stop_rot_or_pitch_sound(&mut self, _owner: ObjectId, _turret: WhichTurretType) {}

    fn get_weapon_slot_status_for_delay(
        &self,
        _owner: ObjectId,
        _slot: WeaponSlotType,
    ) -> Option<crate::rules::combat_types::WeaponStatus> {
        None
    }
}

const INTER_TURRET_DELAY_ENABLED: bool = false;

/// INI configuration block for a turret (`TurretAIData`).
#[derive(Debug, Clone, PartialEq)]
pub struct TurretAIData {
    pub turn_rate: Real,
    pub pitch_rate: Real,
    pub natural_turret_angle: Real,
    pub natural_turret_pitch: Real,
    pub turret_fire_angle_sweep: [Real; WEAPONSLOT_COUNT],
    pub turret_sweep_speed_modifier: [Real; WEAPONSLOT_COUNT],
    pub fire_pitch: Real,
    pub min_pitch: Real,
    pub ground_unit_pitch: Real,
    pub turret_weapon_slots: UnsignedInt,
    pub inter_turret_delay: UnsignedInt,
    pub min_idle_scan_angle: Real,
    pub max_idle_scan_angle: Real,
    pub min_idle_scan_interval: UnsignedInt,
    pub max_idle_scan_interval: UnsignedInt,
    pub recenter_time: UnsignedInt,
    pub initially_disabled: Bool,
    pub fires_while_turning: Bool,
    pub is_allows_pitch: Bool,
}

impl Default for TurretAIData {
    fn default() -> Self {
        Self {
            turn_rate: DEFAULT_TURN_RATE,
            pitch_rate: DEFAULT_PITCH_RATE,
            natural_turret_angle: 0.0_f32,
            natural_turret_pitch: 0.0_f32,
            turret_fire_angle_sweep: [0.0_f32; WEAPONSLOT_COUNT],
            turret_sweep_speed_modifier: [1.0_f32; WEAPONSLOT_COUNT],
            fire_pitch: 0.0_f32,
            min_pitch: 0.0_f32,
            ground_unit_pitch: 0.0_f32,
            turret_weapon_slots: 0,
            inter_turret_delay: 0,
            min_idle_scan_angle: 0.0_f32,
            max_idle_scan_angle: 0.0_f32,
            min_idle_scan_interval: 9_999_999,
            max_idle_scan_interval: 9_999_999,
            recenter_time: 2 * (LOGICFRAMES_PER_SECOND as UnsignedInt),
            initially_disabled: FALSE,
            fires_while_turning: FALSE,
            is_allows_pitch: FALSE,
        }
    }
}

fn parse_turn_rate(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angular_velocity_real(&mut td.turn_rate)
}

fn parse_pitch_rate(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angular_velocity_real(&mut td.pitch_rate)
}

fn parse_natural_turret_angle(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.natural_turret_angle)
}

fn parse_natural_turret_pitch(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.natural_turret_pitch)
}

fn parse_fire_pitch(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.fire_pitch)
}

fn parse_min_pitch(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.min_pitch)
}

fn parse_ground_unit_pitch(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.ground_unit_pitch)
}

fn parse_turret_sweep(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(zh_common::ini::SEPS_DEFAULT)?;
    let wslot = IniReader::scan_index_list(tok, &WEAPON_SLOT_NAMES)?;
    ini.parse_angle_real(&mut td.turret_fire_angle_sweep[wslot as usize])
}

fn parse_turret_sweep_speed(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(zh_common::ini::SEPS_DEFAULT)?;
    let wslot = IniReader::scan_index_list(tok, &WEAPON_SLOT_NAMES)?;
    ini.parse_real(&mut td.turret_sweep_speed_modifier[wslot as usize])
}

fn parse_tws(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    let first = ini.next_token(zh_common::ini::SEPS_DEFAULT)?;
    let wslot = IniReader::scan_index_list(first, &WEAPON_SLOT_NAMES)?;
    td.turret_weapon_slots |= 1u32 << (wslot as u32);
    while let Some(tok) = ini.next_token_or_null(zh_common::ini::SEPS_DEFAULT) {
        let slot = IniReader::scan_index_list(tok, &WEAPON_SLOT_NAMES)?;
        td.turret_weapon_slots |= 1u32 << (slot as u32);
    }
    Ok(())
}

fn parse_allows_pitch(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut td.is_allows_pitch)
}

fn parse_inter_turret_delay(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_duration_unsigned_int(&mut td.inter_turret_delay)
}

fn parse_min_idle_scan_angle(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.min_idle_scan_angle)
}

fn parse_max_idle_scan_angle(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut td.max_idle_scan_angle)
}

fn parse_min_idle_scan_interval(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_duration_unsigned_int(&mut td.min_idle_scan_interval)
}

fn parse_max_idle_scan_interval(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_duration_unsigned_int(&mut td.max_idle_scan_interval)
}

fn parse_recenter_time(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_duration_unsigned_int(&mut td.recenter_time)
}

fn parse_initially_disabled(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut td.initially_disabled)
}

fn parse_fires_while_turning(
    ini: &mut IniReader<'_>,
    td: &mut TurretAIData,
    _: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut td.fires_while_turning)
}

/// Exact 19-entry `FieldParse` dispatch table matching `TurretAIData::buildFieldParse`.
pub static TURRET_AI_DATA_FIELD_PARSE: [FieldParseEntry<TurretAIData>; 19] = [
    FieldParseEntry { token: Some("TurretTurnRate"), parse: parse_turn_rate, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("TurretPitchRate"), parse: parse_pitch_rate, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("NaturalTurretAngle"), parse: parse_natural_turret_angle, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("NaturalTurretPitch"), parse: parse_natural_turret_pitch, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("FirePitch"), parse: parse_fire_pitch, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MinPhysicalPitch"), parse: parse_min_pitch, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("GroundUnitPitch"), parse: parse_ground_unit_pitch, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("TurretFireAngleSweep"), parse: parse_turret_sweep, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("TurretSweepSpeedModifier"), parse: parse_turret_sweep_speed, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("ControlledWeaponSlots"), parse: parse_tws, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("AllowsPitch"), parse: parse_allows_pitch, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("InterTurretDelay"), parse: parse_inter_turret_delay, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MinIdleScanAngle"), parse: parse_min_idle_scan_angle, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MaxIdleScanAngle"), parse: parse_max_idle_scan_angle, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MinIdleScanInterval"), parse: parse_min_idle_scan_interval, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("MaxIdleScanInterval"), parse: parse_max_idle_scan_interval, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("RecenterTime"), parse: parse_recenter_time, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("InitiallyDisabled"), parse: parse_initially_disabled, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("FiresWhileTurning"), parse: parse_fires_while_turning, user_data: FieldUserData::None },
];

impl TurretAIData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn parse_ini_reader(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        ini.init_from_ini(self, &TURRET_AI_DATA_FIELD_PARSE)
    }

    /// Parses fields inside a `Turret ... End` INI block (if `End` is missing, appends one).
    pub fn parse_ini_block(&mut self, text: &str) -> Result<(), IniError> {
        let mut owned = text.to_string();
        if !owned
            .lines()
            .rev()
            .find(|l| !l.trim().is_empty())
            .is_some_and(|l| l.trim().eq_ignore_ascii_case("End"))
        {
            if !owned.ends_with('\n') {
                owned.push('\n');
            }
            owned.push_str("End\n");
        }
        let mut reader = IniReader::new("turret_ai.ini", owned.as_bytes(), IniLoadType::Overwrite);
        self.parse_ini_reader(&mut reader)
    }
}

#[inline]
fn frame_to_sleep_time(
    now: UnsignedInt,
    mut frame1: UnsignedInt,
    frame2: UnsignedInt,
) -> StateReturnType {
    if frame1 > frame2 {
        frame1 = frame2;
    }
    if frame1 > STATE_SLEEP_FOREVER_FRAMES {
        frame1 = STATE_SLEEP_FOREVER_FRAMES;
    }
    if frame1 > now {
        StateReturnType::Sleep(frame1 - now)
    } else {
        StateReturnType::Continue
    }
}

/// Internal per-state data for [`TurretStateMachine`].
#[derive(Debug, Clone, PartialEq)]
struct TurretStatesData {
    next_idle_scan: UnsignedInt,
    desired_angle: Real,
    aim_extra_delay: UnsignedInt,
    hold_timestamp: UnsignedInt,
}

impl Default for TurretStatesData {
    fn default() -> Self {
        Self {
            next_idle_scan: 0,
            desired_angle: 0.0_f32,
            aim_extra_delay: 0,
            hold_timestamp: 0,
        }
    }
}

/// Embedded state machine state matching `TurretStateMachine` + `StateMachine`.
#[derive(Debug, Clone, PartialEq)]
pub struct TurretStateMachine {
    owner_id: ObjectId,
    sleep_till: UnsignedInt,
    default_state_id: StateId,
    current_state_id: Option<StateId>,
    goal_object_id: ObjectId,
    goal_position: Coord3D,
    locked: Bool,
    default_state_inited: Bool,
    transition_depth: u32,
    sleep_transition_depth: u32,
    states: TurretStatesData,
}

impl TurretStateMachine {
    fn new(owner_id: ObjectId) -> Self {
        Self {
            owner_id,
            sleep_till: 0,
            default_state_id: TurretStateType::Idle.as_state_id(),
            current_state_id: None,
            goal_object_id: ObjectId::INVALID,
            goal_position: Coord3D::default(),
            locked: FALSE,
            default_state_inited: FALSE,
            transition_depth: 0,
            sleep_transition_depth: 0,
            states: TurretStatesData::default(),
        }
    }

    #[inline]
    pub const fn get_current_state_id(&self) -> StateId {
        match self.current_state_id {
            Some(id) => id,
            None => INVALID_STATE_ID,
        }
    }

    #[inline]
    pub const fn get_current_state(&self) -> Option<TurretStateType> {
        match self.current_state_id {
            Some(id) => TurretStateType::from_state_id(id),
            None => None,
        }
    }

    #[inline]
    pub const fn get_goal_object_id(&self) -> ObjectId {
        self.goal_object_id
    }

    #[inline]
    pub const fn get_goal_position(&self) -> &Coord3D {
        &self.goal_position
    }

    #[inline]
    pub const fn peek_sleep_till(&self) -> UnsignedInt {
        self.sleep_till
    }

    #[inline]
    pub fn set_goal_object(&mut self, obj_id: ObjectId, world: &dyn TurretWorldAccess) {
        if self.locked == TRUE {
            return;
        }
        if obj_id != ObjectId::INVALID {
            self.goal_object_id = obj_id;
            if let Some(pos) = world.get_object_position(obj_id) {
                self.goal_position = pos;
            }
        } else {
            self.goal_object_id = ObjectId::INVALID;
        }
    }

    #[inline]
    pub fn set_goal_position(&mut self, pos: &Coord3D) {
        if self.locked == TRUE {
            return;
        }
        self.goal_position = *pos;
    }

    #[inline]
    fn internal_clear(&mut self) {
        self.goal_object_id = ObjectId::INVALID;
        self.goal_position = Coord3D::default();
    }

    #[inline]
    fn state_transitions(state: TurretStateType) -> (StateId, StateId) {
        match state {
            TurretStateType::Idle => (
                TurretStateType::Idle.as_state_id(),
                TurretStateType::IdleScan.as_state_id(),
            ),
            TurretStateType::IdleScan => (
                TurretStateType::Hold.as_state_id(),
                TurretStateType::Hold.as_state_id(),
            ),
            TurretStateType::Aim => (
                TurretStateType::Fire.as_state_id(),
                TurretStateType::Hold.as_state_id(),
            ),
            TurretStateType::Fire => (
                TurretStateType::Aim.as_state_id(),
                TurretStateType::Aim.as_state_id(),
            ),
            TurretStateType::Recenter => (
                TurretStateType::Idle.as_state_id(),
                TurretStateType::Idle.as_state_id(),
            ),
            TurretStateType::Hold => (
                TurretStateType::Recenter.as_state_id(),
                TurretStateType::Recenter.as_state_id(),
            ),
        }
    }

    fn xfer_state_by_id(
        &mut self,
        id: StateId,
        xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        let Some(st) = TurretStateType::from_state_id(id) else {
            return Ok(());
        };
        match st {
            TurretStateType::Idle => {
                let mut v: XferVersion = 1;
                xfer.xfer_version(&mut v, 1)?;
                xfer.xfer_unsigned_int(&mut self.states.next_idle_scan)?;
            }
            TurretStateType::IdleScan => {
                let mut v: XferVersion = 1;
                xfer.xfer_version(&mut v, 1)?;
                xfer.xfer_real(&mut self.states.desired_angle)?;
            }
            TurretStateType::Aim | TurretStateType::Fire | TurretStateType::Recenter => {
                let mut v: XferVersion = 1;
                xfer.xfer_version(&mut v, 1)?;
            }
            TurretStateType::Hold => {
                let mut v: XferVersion = 1;
                xfer.xfer_version(&mut v, 1)?;
                xfer.xfer_unsigned_int(&mut self.states.hold_timestamp)?;
            }
        }
        Ok(())
    }

    fn xfer_base_state_machine(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;

        xfer.xfer_unsigned_int(&mut self.sleep_till)?;
        xfer.xfer_unsigned_int(&mut self.default_state_id)?;

        let mut cur_state_id = self.get_current_state_id();
        xfer.xfer_unsigned_int(&mut cur_state_id)?;
        if xfer.mode() == XferMode::Load {
            if cur_state_id == INVALID_STATE_ID
                || cur_state_id == MACHINE_DONE_STATE_ID
                || TurretStateType::from_state_id(cur_state_id).is_none()
            {
                cur_state_id = self.default_state_id;
            }
            self.current_state_id = TurretStateType::from_state_id(cur_state_id)
                .map(|s| s.as_state_id());
        }

        let mut snapshot_all_states = FALSE;
        xfer.xfer_bool(&mut snapshot_all_states)?;

        if snapshot_all_states == TRUE {
            let mut count = TurretStateType::NUM_STATES as Int;
            xfer.xfer_int(&mut count)?;
            if xfer.mode() == XferMode::Load && count as usize != TurretStateType::NUM_STATES {
                return Err(XferError::InvalidData);
            }
            for idx in 0..TurretStateType::NUM_STATES {
                let mut saved_id = idx as StateId;
                xfer.xfer_unsigned_int(&mut saved_id)?;
                if saved_id != idx as StateId {
                    return Err(XferError::InvalidData);
                }
                self.xfer_state_by_id(saved_id, xfer)?;
            }
        } else {
            if xfer.mode() == XferMode::Load
                && (cur_state_id == INVALID_STATE_ID
                    || cur_state_id == MACHINE_DONE_STATE_ID
                    || TurretStateType::from_state_id(cur_state_id).is_none())
            {
                cur_state_id = self.default_state_id;
                self.current_state_id = TurretStateType::from_state_id(cur_state_id)
                    .map(|s| s.as_state_id());
            }
            if self.current_state_id.is_none() {
                self.current_state_id = Some(self.default_state_id);
            }
            if let Some(id) = self.current_state_id {
                self.xfer_state_by_id(id, xfer)?;
            }
        }

        xfer.xfer_object_id(&mut self.goal_object_id)?;
        xfer.xfer_coord3d(&mut self.goal_position)?;
        xfer.xfer_bool(&mut self.locked)?;
        xfer.xfer_bool(&mut self.default_state_inited)?;
        Ok(())
    }
}

/// The machine version this build saves.
const MACHINE_VERSION: XferVersion = if cfg!(feature = "retail-xfer-save") {
    1
} else {
    2
};

impl TurretStateMachine {
    fn xfer_as_version(
        &mut self,
        xfer: &mut dyn Xfer,
        current_version: XferVersion,
    ) -> Result<(), XferError> {
        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;

        if version >= 2 {
            self.xfer_base_state_machine(xfer)?;
        }
        Ok(())
    }
}

impl Snapshot for TurretStateMachine {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.xfer_as_version(xfer, MACHINE_VERSION)
    }

    fn load_post_process(&mut self) {}
}

/// Complete turret behavior controller (`TurretAI` in `TurretAI.h/.cpp`).
#[derive(Debug, Clone, PartialEq)]
pub struct TurretAI {
    pub data: TurretAIData,
    pub which_turret: WhichTurretType,
    pub owner_id: ObjectId,
    pub sm: TurretStateMachine,
    pub angle: Real,
    pub pitch: Real,
    pub enable_sweep_until: UnsignedInt,
    pub victim_initial_team: TeamId,
    pub target: TurretTargetType,
    pub continuous_fire_expiration_frame: UnsignedInt,
    pub sleep_until: UnsignedInt,

    pub play_rot_sound: Bool,
    pub play_pitch_sound: Bool,
    pub positive_sweep: Bool,
    pub did_fire: Bool,
    pub enabled: Bool,
    pub fires_while_turning: Bool,
    pub is_force_attacking: Bool,
    pub target_was_set_by_idle_mood: Bool,
}

impl TurretAI {
    /// Creates a new `TurretAI` and initializes its default state (`TURRETAI_IDLE`).
    pub fn new(
        owner_id: ObjectId,
        data: TurretAIData,
        which_turret: WhichTurretType,
        world: &mut dyn TurretWorldAccess,
    ) -> Result<Self, TurretAIError> {
        if data.turret_weapon_slots == 0 {
            return Err(TurretAIError::NoControlledWeaponSlots);
        }
        let angle = data.natural_turret_angle;
        let pitch = data.natural_turret_pitch;
        let enabled = if data.initially_disabled == TRUE { FALSE } else { TRUE };
        let fires_while_turning = data.fires_while_turning;

        let mut turret = Self {
            data,
            which_turret,
            owner_id,
            sm: TurretStateMachine::new(owner_id),
            angle,
            pitch,
            enable_sweep_until: 0,
            victim_initial_team: TeamId::INVALID,
            target: TurretTargetType::None,
            continuous_fire_expiration_frame: u32::MAX,
            sleep_until: 0,
            play_rot_sound: FALSE,
            play_pitch_sound: FALSE,
            positive_sweep: TRUE,
            did_fire: FALSE,
            enabled,
            fires_while_turning,
            is_force_attacking: FALSE,
            target_was_set_by_idle_mood: FALSE,
        };
        turret.init_default_state(world);
        Ok(turret)
    }

    #[inline]
    pub const fn get_turret_angle(&self) -> Real {
        self.angle
    }

    #[inline]
    pub const fn get_turret_pitch(&self) -> Real {
        self.pitch
    }

    #[inline]
    pub const fn get_min_pitch(&self) -> Real {
        self.data.min_pitch
    }

    #[inline]
    pub const fn is_allows_pitch(&self) -> Bool {
        self.data.is_allows_pitch
    }

    #[inline]
    pub const fn get_turn_rate(&self) -> Real {
        self.data.turn_rate
    }

    #[inline]
    pub const fn get_natural_turret_angle(&self) -> Real {
        self.data.natural_turret_angle
    }

    #[inline]
    pub const fn get_pitch_rate(&self) -> Real {
        self.data.pitch_rate
    }

    #[inline]
    pub const fn get_fire_pitch(&self) -> Real {
        self.data.fire_pitch
    }

    #[inline]
    pub const fn get_ground_unit_pitch(&self) -> Real {
        self.data.ground_unit_pitch
    }

    #[inline]
    pub const fn get_natural_turret_pitch(&self) -> Real {
        self.data.natural_turret_pitch
    }

    #[inline]
    pub const fn get_turret_fire_angle_sweep_for_weapon_slot(&self, slot: WeaponSlotType) -> Real {
        self.data.turret_fire_angle_sweep[slot.as_i32() as usize]
    }

    #[inline]
    pub const fn get_turret_sweep_speed_modifier_for_weapon_slot(
        &self,
        slot: WeaponSlotType,
    ) -> Real {
        self.data.turret_sweep_speed_modifier[slot.as_i32() as usize]
    }

    #[inline]
    pub const fn get_min_idle_scan_angle(&self) -> Real {
        self.data.min_idle_scan_angle
    }

    #[inline]
    pub const fn get_max_idle_scan_angle(&self) -> Real {
        self.data.max_idle_scan_angle
    }

    #[inline]
    pub const fn get_min_idle_scan_interval(&self) -> UnsignedInt {
        self.data.min_idle_scan_interval
    }

    #[inline]
    pub const fn get_max_idle_scan_interval(&self) -> UnsignedInt {
        self.data.max_idle_scan_interval
    }

    #[inline]
    pub const fn get_recenter_time(&self) -> UnsignedInt {
        self.data.recenter_time
    }

    #[inline]
    pub const fn get_owner_id(&self) -> ObjectId {
        self.owner_id
    }

    #[inline]
    pub const fn is_attacking_object(&self) -> Bool {
        if matches!(self.target, TurretTargetType::Object) {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub const fn is_force_attacking(&self) -> Bool {
        self.is_force_attacking
    }

    #[inline]
    pub const fn is_turret_enabled(&self) -> Bool {
        self.enabled
    }

    #[inline]
    pub const fn friend_get_positive_sweep(&self) -> Bool {
        self.positive_sweep
    }

    #[inline]
    pub fn friend_set_positive_sweep(&mut self, b: Bool) {
        self.positive_sweep = b;
    }

    #[inline]
    pub const fn friend_get_which_turret(&self) -> WhichTurretType {
        self.which_turret
    }

    #[inline]
    pub const fn friend_get_target_was_set_by_idle_mood(&self) -> Bool {
        self.target_was_set_by_idle_mood
    }

    #[inline]
    pub const fn friend_get_victim_initial_team(&self) -> TeamId {
        self.victim_initial_team
    }

    #[inline]
    pub fn is_weapon_slot_on_turret(&self, wslot: WeaponSlotType) -> Bool {
        if (self.data.turret_weapon_slots & (1u32 << (wslot.as_i32() as u32))) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_owners_cur_weapon_on_turret(&self, world: &dyn TurretWorldAccess) -> Bool {
        match world.get_owner_current_weapon_slot(self.owner_id) {
            Some(wslot) => self.is_weapon_slot_on_turret(wslot),
            None => FALSE,
        }
    }

    pub fn is_weapon_slot_ok_to_fire(
        &self,
        wslot: WeaponSlotType,
        world: &dyn TurretWorldAccess,
    ) -> Bool {
        if world.are_turrets_linked(self.owner_id) == TRUE {
            return TRUE;
        }
        self.is_weapon_slot_on_turret(wslot)
    }

    pub fn is_turret_in_natural_position(&self, world: &dyn TurretWorldAccess) -> Bool {
        if world.is_owner_under_construction(self.owner_id) == TRUE {
            return TRUE;
        }
        if self.data.natural_turret_angle == self.angle
            && self.data.natural_turret_pitch == self.pitch
        {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_trying_to_aim_at_target(
        &mut self,
        victim: ObjectId,
        world: &dyn TurretWorldAccess,
    ) -> Bool {
        if self.sm.get_current_state_id() != TurretStateType::Aim.as_state_id() {
            return FALSE;
        }
        let (ttype, obj_id, _pos) = self.friend_get_turret_target(true, world);
        if ttype == TurretTargetType::Object && obj_id == victim {
            TRUE
        } else {
            FALSE
        }
    }

    /// What `friend_getTurretTarget` does on finding its object target dead.
    pub fn drop_dead_target(&mut self) {
        if self.sm.locked == FALSE {
            self.sm.goal_object_id = ObjectId::INVALID;
        }
        self.target = TurretTargetType::None;
        self.target_was_set_by_idle_mood = FALSE;
    }

    pub fn friend_get_turret_target(
        &mut self,
        clear_dead_targets: bool,
        world: &dyn TurretWorldAccess,
    ) -> (TurretTargetType, ObjectId, Coord3D) {
        let mut obj = ObjectId::INVALID;
        let mut pos = Coord3D::default();

        match self.target {
            TurretTargetType::Object => {
                let goal_id = self.sm.get_goal_object_id();
                if goal_id != ObjectId::INVALID && world.object_exists(goal_id) {
                    obj = goal_id;
                }
                if clear_dead_targets
                    && (obj == ObjectId::INVALID || world.is_object_effectively_dead(obj))
                {
                    self.sm.set_goal_object(ObjectId::INVALID, world);
                    self.target = TurretTargetType::None;
                    self.target_was_set_by_idle_mood = FALSE;
                }
            }
            TurretTargetType::Position => {
                obj = ObjectId::INVALID;
                pos = *self.sm.get_goal_position();
            }
            TurretTargetType::None => {}
        }

        (self.target, obj, pos)
    }

    fn remove_self_as_targeter(&self, world: &mut dyn TurretWorldAccess) {
        if self.target == TurretTargetType::Object {
            let target_id = self.sm.get_goal_object_id();
            if self.owner_id != ObjectId::INVALID
                && target_id != ObjectId::INVALID
                && world.object_exists(target_id)
            {
                world.add_targeter(target_id, self.owner_id, FALSE);
            }
        }
    }

    pub fn set_turret_target_object(
        &mut self,
        mut victim: ObjectId,
        force_attacking: Bool,
        world: &mut dyn TurretWorldAccess,
    ) {
        let invalid_victim = victim == ObjectId::INVALID
            || !world.object_exists(victim)
            || world.is_object_effectively_dead(victim)
            || self.is_owners_cur_weapon_on_turret(world) == FALSE;

        if invalid_victim && world.are_turrets_linked(self.owner_id) == FALSE {
            victim = ObjectId::INVALID;
        }

        if victim == ObjectId::INVALID {
            self.remove_self_as_targeter(world);
        }

        self.sm.set_goal_object(victim, world);
        self.target = if victim != ObjectId::INVALID {
            TurretTargetType::Object
        } else {
            TurretTargetType::None
        };
        self.target_was_set_by_idle_mood = FALSE;
        self.is_force_attacking = force_attacking;

        let sid = self.sm.get_current_state_id();
        if victim != ObjectId::INVALID {
            if sid != TurretStateType::Aim.as_state_id()
                && sid != TurretStateType::Fire.as_state_id()
            {
                self.sm_set_state(TurretStateType::Aim.as_state_id(), world);
            }
            self.victim_initial_team = world.get_object_team(victim).unwrap_or(TeamId::INVALID);
        } else {
            if sid == TurretStateType::Aim.as_state_id()
                || sid == TurretStateType::Fire.as_state_id()
            {
                self.sm_set_state(TurretStateType::Hold.as_state_id(), world);
            }
            self.victim_initial_team = TeamId::INVALID;
        }
    }

    pub fn set_turret_target_position(
        &mut self,
        mut pos: Option<&Coord3D>,
        world: &mut dyn TurretWorldAccess,
    ) {
        if (pos.is_none() || self.is_owners_cur_weapon_on_turret(world) == FALSE)
            && world.are_turrets_linked(self.owner_id) == FALSE
        {
            pos = None;
        }

        self.remove_self_as_targeter(world);

        self.sm.set_goal_object(ObjectId::INVALID, world);
        if let Some(p) = pos {
            self.sm.set_goal_position(p);
        }
        self.target = if pos.is_some() {
            TurretTargetType::Position
        } else {
            TurretTargetType::None
        };
        self.target_was_set_by_idle_mood = FALSE;

        let sid = self.sm.get_current_state_id();
        if pos.is_some() {
            if sid != TurretStateType::Aim.as_state_id()
                && sid != TurretStateType::Fire.as_state_id()
            {
                self.sm_set_state(TurretStateType::Aim.as_state_id(), world);
            }
            self.victim_initial_team = TeamId::INVALID;
        } else {
            if sid == TurretStateType::Aim.as_state_id()
                || sid == TurretStateType::Fire.as_state_id()
            {
                self.sm_set_state(TurretStateType::Hold.as_state_id(), world);
            }
            self.victim_initial_team = TeamId::INVALID;
        }
    }

    pub fn recenter_turret(&mut self, world: &mut dyn TurretWorldAccess) {
        self.sm_set_state(TurretStateType::Recenter.as_state_id(), world);
    }

    pub fn set_turret_enabled(&mut self, enabled: Bool, world: &dyn TurretWorldAccess) {
        if enabled == TRUE && self.enabled == FALSE {
            self.sleep_until = world.current_frame();
        }
        self.enabled = enabled;
    }

    #[inline]
    pub fn notify_fired(&mut self) {
        self.did_fire = TRUE;
    }

    pub fn notify_new_victim_chosen(&mut self, victim: ObjectId, world: &mut dyn TurretWorldAccess) {
        self.set_turret_target_object(victim, FALSE, world);
    }

    #[inline]
    pub fn friend_notify_state_machine_changed(&mut self, world: &dyn TurretWorldAccess) {
        self.sleep_until = world.current_frame();
    }

    pub fn friend_turn_towards_angle(
        &mut self,
        mut desired_angle: Real,
        rate_modifier: Real,
        rel_thresh: Real,
        world: &mut dyn TurretWorldAccess,
    ) -> Bool {
        desired_angle = normalize_angle_cpp(desired_angle);

        let orig_angle = self.angle;
        let mut actual_angle = orig_angle;
        let turn_rate = self.data.turn_rate * rate_modifier;
        let angle_diff = normalize_angle_cpp(desired_angle - actual_angle);

        if gm_fabsf(angle_diff) < turn_rate {
            actual_angle = desired_angle;
            world.set_model_condition_turret_rotate(self.owner_id, FALSE);
        } else {
            if angle_diff > 0.0_f32 {
                actual_angle += turn_rate;
            } else {
                actual_angle -= turn_rate;
            }
            world.set_model_condition_turret_rotate(self.owner_id, TRUE);
            self.play_rot_sound = TRUE;
        }

        self.angle = normalize_angle_cpp(actual_angle);

        if self.angle != orig_angle {
            world.react_to_turret_change(self.owner_id, self.which_turret, orig_angle, self.pitch);
        }

        if gm_fabsf(self.angle - desired_angle) <= rel_thresh {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn friend_turn_towards_pitch(
        &mut self,
        mut desired_pitch: Real,
        rate_modifier: Real,
    ) -> Bool {
        if self.data.is_allows_pitch == FALSE {
            return TRUE;
        }

        desired_pitch = normalize_angle_cpp(desired_pitch);

        let mut actual_pitch = self.pitch;
        let pitch_rate = self.data.pitch_rate * rate_modifier;
        let pitch_diff = normalize_angle_cpp(desired_pitch - actual_pitch);

        if gm_fabsf(pitch_diff) < pitch_rate {
            actual_pitch = desired_pitch;
        } else {
            if pitch_diff > 0.0_f32 {
                actual_pitch += pitch_rate;
            } else {
                actual_pitch -= pitch_rate;
            }
            self.play_pitch_sound = TRUE;
        }

        self.pitch = normalize_angle_cpp(actual_pitch);

        if self.pitch == desired_pitch {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn friend_is_any_weapon_in_range_of(
        &self,
        target: ObjectId,
        world: &dyn TurretWorldAccess,
    ) -> Bool {
        for i in 0..WEAPONSLOT_COUNT {
            let slot = match WeaponSlotType::from_i32(i as Int) {
                Some(s) => s,
                None => continue,
            };
            if self.is_weapon_slot_on_turret(slot) == FALSE {
                continue;
            }
            if world.is_weapon_in_range_of_object(self.owner_id, slot, target) == TRUE {
                return TRUE;
            }
        }
        FALSE
    }

    #[inline]
    pub fn friend_is_sweep_enabled(&self, world: &dyn TurretWorldAccess) -> Bool {
        if self.enable_sweep_until != 0 && self.enable_sweep_until > world.current_frame() {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn friend_get_next_idle_mood_target_frame(
        &self,
        world: &mut dyn TurretWorldAccess,
    ) -> UnsignedInt {
        world.get_next_idle_mood_target_frame(self.owner_id)
    }

    pub fn friend_check_for_idle_mood_target(&mut self, world: &mut dyn TurretWorldAccess) {
        if world.is_idle_mood_ignored(self.owner_id) == TRUE {
            return;
        }
        world.lend_turret(self);
        let enemy = world.acquire_idle_mood_target(self.owner_id);
        world.reclaim_turret(self);
        if enemy != ObjectId::INVALID {
            self.set_turret_target_object(enemy, FALSE, world);
            world.lend_turret(self);
            world.choose_best_weapon_for_target(self.owner_id, enemy);
            world.reclaim_turret(self);
            self.target_was_set_by_idle_mood = TRUE;
        }
    }

    pub fn friend_get_inter_turret_delay(&self, world: &mut dyn TurretWorldAccess) -> UnsignedInt {
        use crate::rules::combat_types::WeaponStatus;
        if self.data.inter_turret_delay == 0 {
            return 0;
        }

        let mut num_self = 0i32;
        let mut num_self_reloading = 0i32;
        let mut num_self_ready = 0i32;
        for i in 0..WEAPONSLOT_COUNT {
            let Some(slot) = WeaponSlotType::from_i32(i as Int) else { continue };
            let Some(status) = world.get_weapon_slot_status_for_delay(self.owner_id, slot) else {
                continue;
            };
            if self.is_weapon_slot_on_turret(slot) == TRUE {
                num_self += 1;
                if status == WeaponStatus::ReloadingClip {
                    num_self_reloading += 1;
                } else if status == WeaponStatus::ReadyToFire {
                    num_self_ready += 1;
                }
            }
        }

        let t = world.get_turret_sync(self.owner_id);
        if t == WhichTurretType::Invalid || t == self.which_turret {
            if num_self == num_self_ready {
                world.set_turret_sync(self.owner_id, self.which_turret);
                if t == WhichTurretType::Invalid {
                    self.data.inter_turret_delay
                } else {
                    0
                }
            } else if num_self == num_self_reloading {
                world.set_turret_sync(self.owner_id, WhichTurretType::Invalid);
                WAIT_INDEFINITELY
            } else {
                WAIT_INDEFINITELY
            }
        } else {
            WAIT_INDEFINITELY
        }
    }

    /// Steps the turret behavior for one logic frame and returns the recommended sleep time.
    pub fn update_turret_ai(&mut self, world: &mut dyn TurretWorldAccess) -> UpdateSleepTime {
        let now = world.current_frame();
        if self.sleep_until != 0 && now < self.sleep_until {
            return UpdateSleepTime(self.sleep_until.wrapping_sub(now));
        }

        let mut sub_machine_sleep = UpdateSleepTime::FOREVER;

        if self.fires_while_turning == FALSE || self.continuous_fire_expiration_frame <= now {
            self.play_rot_sound = FALSE;
            self.play_pitch_sound = FALSE;
        }

        if self.enabled == TRUE
            || self.sm.get_current_state_id() == TurretStateType::Recenter.as_state_id()
        {
            self.did_fire = FALSE;

            let st_ret = self.sm_update_state_machine(world);

            if self.did_fire == TRUE {
                self.enable_sweep_until = now.wrapping_add(ENABLE_SWEEP_FRAME_COUNT);
                self.continuous_fire_expiration_frame =
                    now.wrapping_add(ENABLE_SWEEP_FRAME_COUNT);
            }

            if self.play_rot_sound == TRUE || self.play_pitch_sound == TRUE {
                world.on_start_rot_or_pitch_sound(self.owner_id, self.which_turret);
            } else {
                world.on_stop_rot_or_pitch_sound(self.owner_id, self.which_turret);
            }

            if let StateReturnType::Sleep(frames) = st_ret {
                if frames < sub_machine_sleep.0 {
                    sub_machine_sleep = UpdateSleepTime(frames);
                }
            } else {
                sub_machine_sleep = UpdateSleepTime::NONE;
            }
        }

        self.sleep_until = now.wrapping_add(sub_machine_sleep.0);
        sub_machine_sleep
    }

    pub fn load_post_process(&mut self, world: &dyn TurretWorldAccess) {
        let victim = self.sm.get_goal_object_id();
        if victim != ObjectId::INVALID && world.object_exists(victim) {
            self.victim_initial_team = world.get_object_team(victim).unwrap_or(TeamId::INVALID);
        }
    }

    // ---------------------------------------------------------------------------------------------
    // State machine lifecycle & per-state behavior
    // ---------------------------------------------------------------------------------------------

    fn init_default_state(&mut self, world: &mut dyn TurretWorldAccess) -> StateReturnType {
        if self.sm.default_state_inited == TRUE {
            return StateReturnType::Failure;
        }
        if self.sm.locked == TRUE {
            return StateReturnType::Continue;
        }
        self.sm.default_state_inited = TRUE;
        self.sm_internal_set_state(self.sm.default_state_id, world)
    }

    pub fn sm_clear(&mut self, world: &mut dyn TurretWorldAccess) {
        if self.sm.locked == TRUE {
            return;
        }
        if let Some(st) = self.sm.get_current_state() {
            self.on_exit_state(st, StateExitType::Reset, world);
        }
        self.sm.current_state_id = None;
        self.sm.internal_clear();
        self.friend_notify_state_machine_changed(world);
    }

    pub fn sm_reset_to_default_state(
        &mut self,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        if self.sm.locked == TRUE || self.sm.default_state_inited != TRUE {
            return StateReturnType::Failure;
        }
        if let Some(st) = self.sm.get_current_state() {
            self.on_exit_state(st, StateExitType::Reset, world);
        }
        self.sm.current_state_id = None;
        self.sm.internal_clear();
        let tmp = self.sm_internal_set_state(self.sm.default_state_id, world);
        self.friend_notify_state_machine_changed(world);
        tmp
    }

    pub fn sm_set_state(
        &mut self,
        new_state_id: StateId,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        if self.sm.locked == TRUE {
            return StateReturnType::Continue;
        }
        let old_id = self.sm.get_current_state_id();
        let tmp = self.sm_internal_set_state(new_state_id, world);
        if old_id != new_state_id {
            self.friend_notify_state_machine_changed(world);
        }
        tmp
    }

    fn sm_update_state_machine(&mut self, world: &mut dyn TurretWorldAccess) -> StateReturnType {
        let now = world.current_frame();
        if self.sm.sleep_till != 0 && now < self.sm.sleep_till {
            let Some(st) = self.sm.get_current_state() else {
                return StateReturnType::Failure;
            };
            let remaining = self.sm.sleep_till.wrapping_sub(now);
            return self.sm_check_sleep_transitions(StateReturnType::Sleep(remaining), st, world);
        }

        self.sm.sleep_till = 0;
        let Some(st) = self.sm.get_current_state() else {
            return StateReturnType::Failure;
        };
        let id = st.as_state_id();
        let mut status = self.on_update_state(st, world);

        if self.sm.current_state_id.is_none() {
            return StateReturnType::Failure;
        }
        if self.sm.current_state_id != Some(id) {
            status = StateReturnType::Continue;
        }

        if let StateReturnType::Sleep(frames) = status {
            self.sm.sleep_till = world.current_frame().wrapping_add(frames);
            let remaining = self.sm.sleep_till.wrapping_sub(world.current_frame());
            if let Some(cur_st) = self.sm.get_current_state() {
                return self.sm_check_sleep_transitions(
                    StateReturnType::Sleep(remaining),
                    cur_st,
                    world,
                );
            }
            return StateReturnType::Failure;
        }

        let result_st = self.sm.get_current_state().unwrap_or(st);
        self.sm_check_normal_transitions(status, result_st, world)
    }

    fn sm_internal_set_state(
        &mut self,
        new_state_id: StateId,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        self.sm.sleep_till = 0;
        if new_state_id == MACHINE_DONE_STATE_ID {
            if let Some(st) = self.sm.get_current_state() {
                self.on_exit_state(st, StateExitType::Normal, world);
            }
            self.sm.current_state_id = None;
            return StateReturnType::Continue;
        }

        let mut resolved_id = new_state_id;
        if resolved_id == INVALID_STATE_ID {
            resolved_id = self.sm.default_state_id;
            if resolved_id == INVALID_STATE_ID {
                return StateReturnType::Failure;
            }
        }
        let Some(new_state) = TurretStateType::from_state_id(resolved_id) else {
            return StateReturnType::Failure;
        };

        if let Some(old_st) = self.sm.get_current_state() {
            self.on_exit_state(old_st, StateExitType::Normal, world);
        }

        self.sm.current_state_id = Some(resolved_id);
        let mut status = self.on_enter_state(new_state, world);
        if self.sm.current_state_id.is_none() {
            return StateReturnType::Failure;
        }
        if self.sm.current_state_id != Some(resolved_id) {
            status = StateReturnType::Continue;
        }

        if let StateReturnType::Sleep(frames) = status {
            self.sm.sleep_till = world.current_frame().wrapping_add(frames);
            let remaining = self.sm.sleep_till.wrapping_sub(world.current_frame());
            if let Some(cur_st) = self.sm.get_current_state() {
                return self.sm_check_sleep_transitions(
                    StateReturnType::Sleep(remaining),
                    cur_st,
                    world,
                );
            }
            return StateReturnType::Failure;
        }

        let result_st = self.sm.get_current_state().unwrap_or(new_state);
        self.sm_check_normal_transitions(status, result_st, world)
    }

    fn sm_check_normal_transitions(
        &mut self,
        status: StateReturnType,
        state: TurretStateType,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        self.sm.transition_depth = self.sm.transition_depth.saturating_add(1);
        if self.sm.transition_depth >= MAX_TRANSITION_DEPTH {
            self.sm.transition_depth = self.sm.transition_depth.saturating_sub(1);
            return StateReturnType::Failure;
        }

        let (success_id, failure_id) = TurretStateMachine::state_transitions(state);
        let result = match status {
            StateReturnType::Success => match success_id {
                EXIT_MACHINE_WITH_SUCCESS => {
                    let _ = self.sm_internal_set_state(MACHINE_DONE_STATE_ID, world);
                    StateReturnType::Success
                }
                EXIT_MACHINE_WITH_FAILURE => {
                    let _ = self.sm_internal_set_state(MACHINE_DONE_STATE_ID, world);
                    StateReturnType::Failure
                }
                next_id => self.sm_internal_set_state(next_id, world),
            },
            StateReturnType::Failure => match failure_id {
                EXIT_MACHINE_WITH_SUCCESS => {
                    let _ = self.sm_internal_set_state(MACHINE_DONE_STATE_ID, world);
                    StateReturnType::Success
                }
                EXIT_MACHINE_WITH_FAILURE => {
                    let _ = self.sm_internal_set_state(MACHINE_DONE_STATE_ID, world);
                    StateReturnType::Failure
                }
                next_id => self.sm_internal_set_state(next_id, world),
            },
            StateReturnType::Continue => {
                if state == TurretStateType::Fire
                    && world.out_of_weapon_range_object(self.owner_id, self.sm.goal_object_id)
                        == TRUE
                {
                    self.sm_internal_set_state(TurretStateType::Aim.as_state_id(), world)
                } else {
                    StateReturnType::Continue
                }
            }
            sleep @ StateReturnType::Sleep(_) => sleep,
        };

        self.sm.transition_depth = self.sm.transition_depth.saturating_sub(1);
        result
    }

    fn sm_check_sleep_transitions(
        &mut self,
        status: StateReturnType,
        state: TurretStateType,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        self.sm.sleep_transition_depth = self.sm.sleep_transition_depth.saturating_add(1);
        if self.sm.sleep_transition_depth >= MAX_TRANSITION_DEPTH {
            self.sm.sleep_transition_depth = self.sm.sleep_transition_depth.saturating_sub(1);
            return StateReturnType::Failure;
        }

        let result = if state == TurretStateType::Fire
            && world.out_of_weapon_range_object(self.owner_id, self.sm.goal_object_id) == TRUE
        {
            self.sm_internal_set_state(TurretStateType::Aim.as_state_id(), world)
        } else {
            status
        };

        self.sm.sleep_transition_depth = self.sm.sleep_transition_depth.saturating_sub(1);
        result
    }

    fn on_enter_state(
        &mut self,
        state: TurretStateType,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        match state {
            TurretStateType::Idle => {
                world.reset_next_mood_check_time(self.owner_id);
                if world.get_turret_sync(self.owner_id) == self.which_turret {
                    world.set_turret_sync(self.owner_id, WhichTurretType::Invalid);
                }
                let now = world.current_frame();
                let delay = world.random_int(
                    self.data.min_idle_scan_interval as Int,
                    self.data.max_idle_scan_interval as Int,
                ) as UnsignedInt;
                self.sm.states.next_idle_scan = now.wrapping_add(delay);
                frame_to_sleep_time(
                    now,
                    self.friend_get_next_idle_mood_target_frame(world),
                    self.sm.states.next_idle_scan,
                )
            }
            TurretStateType::IdleScan => {
                let min_a = self.data.min_idle_scan_angle;
                let max_a = self.data.max_idle_scan_angle;
                if min_a == 0.0_f32 && max_a == 0.0_f32 {
                    return StateReturnType::Success;
                }
                self.sm.states.desired_angle = min_a
                    + world.random_real(0.0_f32, max_a - min_a);
                if world.random_int(0, 1) == 0 {
                    self.sm.states.desired_angle = -self.sm.states.desired_angle;
                }
                StateReturnType::Continue
            }
            TurretStateType::Aim => {
                if INTER_TURRET_DELAY_ENABLED {
                    self.sm.states.aim_extra_delay = 0;
                }
                StateReturnType::Continue
            }
            TurretStateType::Fire => {
                let victim = self.sm.goal_object_id;
                world.lend_turret(self);
                let status = world.on_enter_fire_state(self.owner_id, victim);
                world.reclaim_turret(self);
                status
            }
            TurretStateType::Recenter => StateReturnType::Continue,
            TurretStateType::Hold => {
                let now = world.current_frame();
                self.sm.states.hold_timestamp = now.wrapping_add(self.data.recenter_time);
                frame_to_sleep_time(
                    now,
                    self.friend_get_next_idle_mood_target_frame(world),
                    self.sm.states.hold_timestamp,
                )
            }
        }
    }

    fn on_update_state(
        &mut self,
        state: TurretStateType,
        world: &mut dyn TurretWorldAccess,
    ) -> StateReturnType {
        match state {
            TurretStateType::Idle => {
                let now = world.current_frame();
                if now >= self.sm.states.next_idle_scan {
                    return StateReturnType::Failure;
                }
                self.friend_check_for_idle_mood_target(world);
                frame_to_sleep_time(
                    world.current_frame(),
                    self.friend_get_next_idle_mood_target_frame(world),
                    self.sm.states.next_idle_scan,
                )
            }
            TurretStateType::IdleScan => {
                if world.is_owner_under_construction(self.owner_id) == TRUE {
                    return StateReturnType::Continue;
                }
                let angle_aligned = self.friend_turn_towards_angle(
                    self.data.natural_turret_angle + self.sm.states.desired_angle,
                    0.5_f32,
                    0.0_f32,
                    world,
                );
                let pitch_aligned =
                    self.friend_turn_towards_pitch(self.data.natural_turret_pitch, 0.5_f32);
                if angle_aligned == TRUE && pitch_aligned == TRUE {
                    StateReturnType::Success
                } else {
                    StateReturnType::Continue
                }
            }
            TurretStateType::Aim => self.update_aim_state(world),
            TurretStateType::Fire => {
                let is_ok = match world.get_owner_current_weapon_slot(self.owner_id) {
                    Some(slot) => self.is_weapon_slot_ok_to_fire(slot, world),
                    None => FALSE,
                };
                let (object, victim, position) = (
                    self.is_attacking_object(),
                    self.sm.goal_object_id,
                    self.sm.goal_position,
                );
                world.lend_turret(self);
                let step = world.on_update_fire_state(
                    self.owner_id,
                    self.which_turret,
                    object,
                    victim,
                    &position,
                    is_ok,
                );
                world.reclaim_turret(self);
                if let Some(new_victim) = step.transfer_victim {
                    self.sm.set_goal_object(new_victim, world);
                    if step.did_fire == FALSE && new_victim != ObjectId::INVALID {
                        self.notify_new_victim_chosen(new_victim, world);
                    }
                }
                if step.did_fire == TRUE {
                    self.notify_fired();
                }
                step.status
            }
            TurretStateType::Recenter => {
                if world.is_owner_under_construction(self.owner_id) == TRUE {
                    return StateReturnType::Continue;
                }
                let angle_aligned = self.friend_turn_towards_angle(
                    self.data.natural_turret_angle,
                    0.5_f32,
                    0.0_f32,
                    world,
                );
                let pitch_aligned =
                    self.friend_turn_towards_pitch(self.data.natural_turret_pitch, 0.5_f32);
                if angle_aligned == TRUE && pitch_aligned == TRUE {
                    StateReturnType::Success
                } else {
                    StateReturnType::Continue
                }
            }
            TurretStateType::Hold => {
                let now = world.current_frame();
                if now >= self.sm.states.hold_timestamp {
                    return StateReturnType::Success;
                }
                self.friend_check_for_idle_mood_target(world);
                frame_to_sleep_time(
                    world.current_frame(),
                    self.friend_get_next_idle_mood_target_frame(world),
                    self.sm.states.hold_timestamp,
                )
            }
        }
    }

    fn on_exit_state(
        &mut self,
        state: TurretStateType,
        _exit_type: StateExitType,
        world: &mut dyn TurretWorldAccess,
    ) {
        if state == TurretStateType::Fire {
            world.lend_turret(self);
            world.on_exit_fire_state(self.owner_id);
            world.reclaim_turret(self);
        }
    }

    fn update_aim_state(&mut self, world: &mut dyn TurretWorldAccess) -> StateReturnType {
        let (target_type, enemy_obj, mut enemy_position) =
            self.friend_get_turret_target(true, world);
        let enemy_for_distance_check_only = enemy_obj;
        let mut preventing = false;
        let mut nothing_in_range = false;

        match target_type {
            TurretTargetType::None => return StateReturnType::Failure,
            TurretTargetType::Object => {
                let is_primary_enemy = enemy_obj != ObjectId::INVALID
                    && enemy_obj == world.get_owner_ai_goal_object(self.owner_id);
                world.lend_turret(self);
                let mut able_to_attack_target = world.is_owner_able_to_attack(self.owner_id) == TRUE;
                if able_to_attack_target {
                    let result = world.can_owner_attack_specific_object(
                        self.owner_id,
                        enemy_obj,
                        self.is_force_attacking,
                    );
                    able_to_attack_target = matches!(
                        result,
                        CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving
                    );
                }
                world.reclaim_turret(self);

                nothing_in_range =
                    self.friend_is_any_weapon_in_range_of(enemy_obj, world) == FALSE;
                let enemy_team = if enemy_obj != ObjectId::INVALID {
                    world.get_object_team(enemy_obj).unwrap_or(TeamId::INVALID)
                } else {
                    TeamId::INVALID
                };

                if enemy_obj == ObjectId::INVALID
                    || !able_to_attack_target
                    || (!is_primary_enemy && nothing_in_range)
                    || enemy_team != self.victim_initial_team
                {
                    if self.target_was_set_by_idle_mood == TRUE {
                        self.set_turret_target_object(ObjectId::INVALID, FALSE, world);
                    }
                    return StateReturnType::Failure;
                }

                if world.is_object_kind_of(enemy_obj, KindOfType::Bridge) == TRUE {
                    enemy_position =
                        world.get_bridge_closest_attack_point(self.owner_id, enemy_obj);
                } else {
                    enemy_position = world.get_object_position(enemy_obj).unwrap_or_default();
                }

                world.add_targeter(enemy_obj, self.owner_id, TRUE);
                preventing = world.is_temporarily_preventing_aim_success(enemy_obj) == TRUE;
            }
            TurretTargetType::Position => {}
        }

        let Some(slot) = world.get_owner_current_weapon_slot(self.owner_id) else {
            return StateReturnType::Failure;
        };

        let mut turn_speed_modifier = 1.0_f32;
        let rel_angle = world.get_relative_angle_2d(self.owner_id, &enemy_position);

        let mut aim_angle = rel_angle;
        let sweep = self.get_turret_fire_angle_sweep_for_weapon_slot(slot);
        if sweep > 0.0_f32 && self.friend_is_sweep_enabled(world) == TRUE {
            if self.positive_sweep == TRUE {
                aim_angle += sweep;
            } else {
                aim_angle -= sweep;
            }
            turn_speed_modifier = self.get_turret_sweep_speed_modifier_for_weapon_slot(slot);
        }

        let mut turn_aligned_to_nemesis =
            self.friend_turn_towards_angle(aim_angle, turn_speed_modifier, AIM_REL_THRESH, world);

        if sweep > 0.0_f32 {
            if turn_aligned_to_nemesis == TRUE {
                self.positive_sweep = if self.positive_sweep == TRUE { FALSE } else { TRUE };
            }
            let angle_diff = normalize_angle_cpp(rel_angle - self.angle);
            turn_aligned_to_nemesis = if gm_fabsf(angle_diff) < sweep { TRUE } else { FALSE };
        }

        let mut pitch_aligned_to_nemesis = TRUE;
        if self.data.is_allows_pitch == TRUE {
            let mut desired_pitch;
            if self.data.fire_pitch > 0.0_f32 {
                desired_pitch = self.data.fire_pitch;
            } else {
                let mut v = world.get_vector_to_center_3d(self.owner_id, &enemy_position);
                v.z -= world.get_owner_max_height_above_position(self.owner_id) / 2.0_f32;

                let v_len = v.length();
                let actual_pitch = if v_len > 0.0_f32 {
                    gm_asinf(v.z / v_len)
                } else {
                    0.0_f32
                };

                desired_pitch = actual_pitch;
                if desired_pitch < self.data.min_pitch {
                    desired_pitch = self.data.min_pitch;
                }
                if self.data.ground_unit_pitch > 0.0_f32 {
                    // C++ sets `enemy = nullptr;` prior to pitch calculation, making `!enemy` true.
                    let mut range = world.get_weapon_attack_range(self.owner_id, slot);
                    let dist = v.length();
                    if range < 1.0_f32 {
                        range = 1.0_f32;
                    }
                    let ground_pitch = self.data.ground_unit_pitch * (dist / range);
                    desired_pitch = actual_pitch + ground_pitch;
                    if desired_pitch < self.data.min_pitch {
                        desired_pitch = self.data.min_pitch;
                    }
                }
            }
            pitch_aligned_to_nemesis = self.friend_turn_towards_pitch(desired_pitch, 1.0_f32);
        }

        if turn_aligned_to_nemesis != TRUE || pitch_aligned_to_nemesis != TRUE {
            return StateReturnType::Continue;
        }
        let in_range = if enemy_for_distance_check_only != ObjectId::INVALID {
            world.is_weapon_in_range_of_object(self.owner_id, slot, enemy_for_distance_check_only)
                == TRUE
        } else {
            world.is_weapon_in_range_of_pos(self.owner_id, slot, &enemy_position) == TRUE
        };

        if in_range {
            if INTER_TURRET_DELAY_ENABLED {
                if self.sm.states.aim_extra_delay > 0 {
                    self.sm.states.aim_extra_delay -= 1;
                } else {
                    self.sm.states.aim_extra_delay = self.friend_get_inter_turret_delay(world);
                    if self.sm.states.aim_extra_delay == WAIT_INDEFINITELY {
                        self.sm.states.aim_extra_delay = 0;
                        return StateReturnType::Continue;
                    }
                }
                if self.sm.states.aim_extra_delay != 0 {
                    return StateReturnType::Continue;
                }
            }
            if preventing || nothing_in_range {
                return StateReturnType::Continue;
            }
            return StateReturnType::Success;
        }

        StateReturnType::Continue
    }
}

impl TurretAI {
    /// `xfer` with the state machine saved as version 1 (the reference build's,
    /// which keeps nothing of it) or 2.
    pub fn xfer_with_machine_version(
        &mut self,
        xfer: &mut dyn Xfer,
        machine_version: XferVersion,
    ) -> Result<(), XferError> {
        let current_version: XferVersion = 2;
        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;

        self.sm.xfer_as_version(xfer, machine_version)?;

        xfer.xfer_real(&mut self.angle)?;
        xfer.xfer_real(&mut self.pitch)?;
        xfer.xfer_unsigned_int(&mut self.enable_sweep_until)?;

        let mut target_i32 = self.target.as_i32();
        xfer.xfer_enum_i32(&mut target_i32)?;
        if xfer.mode() == XferMode::Load {
            self.target = TurretTargetType::from_i32(target_i32);
        }

        xfer.xfer_unsigned_int(&mut self.continuous_fire_expiration_frame)?;

        xfer.xfer_bool(&mut self.play_rot_sound)?;
        xfer.xfer_bool(&mut self.play_pitch_sound)?;
        xfer.xfer_bool(&mut self.positive_sweep)?;
        xfer.xfer_bool(&mut self.did_fire)?;
        xfer.xfer_bool(&mut self.enabled)?;
        xfer.xfer_bool(&mut self.fires_while_turning)?;
        xfer.xfer_bool(&mut self.target_was_set_by_idle_mood)?;

        if version >= 2 {
            xfer.xfer_unsigned_int(&mut self.sleep_until)?;
        }

        Ok(())
    }
}

impl Snapshot for TurretAI {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.xfer_with_machine_version(xfer, MACHINE_VERSION)
    }

    fn load_post_process(&mut self) {}
}
