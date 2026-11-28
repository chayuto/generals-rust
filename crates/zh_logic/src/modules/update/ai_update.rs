//! `PKT-301`: `AIUpdateInterface` (`AIUpdate.h`, `AIUpdate.cpp`): module data,
//! orders, the frame update, locomotor choice and drive, and `xfer`. Packet doc:
//! `packets/phase5/PKT-301_AIUpdate.md`. Golden test:
//! `tests/golden_s8_ai_orders.rs`.
//!
//! The module is resident (see [`crate::ai`]): everything that acts is a function
//! of `(sim, ctx)` that borrows the module through [`ai`] for one statement.
//! Orders arrive through [`execute`], the module's command entry.
//!
//! `AIUpdateInterface::crc` is its `xfer` in CRC mode, and that includes the
//! module's next call frame, which only the world has. A game CRC must call
//! `xfer` for this module; its `crc` writes nothing.
//!
//! Path requests, `computePath` and the drive along the path are in
//! `ai_update/path_follow.rs`, asking allies aside in `allies.rs`, blocking and
//! collisions in `collision.rs` (S9; golden test `tests/golden_s9_path_follow.rs`).
//!
//! Turrets: [`turrets`]. Not here: mood targets (S12); the derived AI classes (S15, S17,
//! S18; the dozer and worker are in `dozer_ai_update.rs` and
//! `worker_ai_update.rs`, see [`DerivedAi`]); groups and command buttons.

use crate::object::weapon_queries;
use crate::object::weapon_set::WeaponSlotType;
use crate::ai::ai_state_machine::{self as machine, state_id, AiStateMachine};
use crate::ai::ai_states;
use crate::ai::seams::{ai_seams, AiActionQuery};
use crate::ai::ResidentEntries;
use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::locomotor::{Locomotor, LocomotorWorld};
use crate::combat::locomotor_set::{LocomotorSet, LocomotorSetType};
use crate::ai::ai::{MM_CONTROLLER_AI, MM_MOOD_SLEEP};
use crate::ai::mood;
use crate::combat::turret_ai::{TurretAIData, WhichTurretType, TURRET_AI_DATA_FIELD_PARSE};
use crate::modules::update::dozer_ai_update::{self, DozerAi};
use crate::modules::update::worker_ai_update::{self, WorkerAi};
use crate::modules::behavior::behavior_module::UpdateScheduleTracker;
use crate::modules::module::{
    AiUpdateFacet, AttitudeType, BehaviorModule, CommandSourceType, ModuleData,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::physics_update;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::pathfinder::Path;
use crate::rules::disabled_types::{DisabledMaskType, DisabledType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{FindPositionOptions, SimApi, SimView};
use std::any::Any;
use std::sync::Arc;
use zh_common::core::state_machine::StateReturnType;
use zh_common::ini::{IniError, IniReader, LOGICFRAMES_PER_SECOND};
use zh_common::math::{gm_sqrtf, Coord3D, ICoord2D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

mod allies;
mod attack_path;
mod collision;
mod mood_target;
mod path_follow;
pub mod turrets;

use allies::{move_allies, move_allies_away_from_destination, private_move_away_from_unit};
pub use collision::process_collision;
pub(crate) use collision::relative_angle_2d;
pub use path_follow::{
    can_compute_quick_path, compute_path, compute_quick_path, is_valid_locomotor_position,
    locomotor_distance_to_goal, request_approach_path, request_attack_path, request_path,
    request_safe_path,
};
pub use attack_path::{
    is_attack_view_blocked_by_obstacle, is_view_blocked_by_obstacle, line_of_sight_filter,
};
pub use crate::ai::mood::MoodMatrixAction;
pub use mood_target::{drop_team_target, next_mood_target, offer_team_target};
pub(crate) use path_follow::{goal_position, maximum_pathfind_extent, update_goal};
use path_follow::{point_on_path, set_path, with_pathfinder};

/// `AI::findClosestRepulsor` for the unit of `ctx`.
pub fn closest_repulsor(sim: &mut dyn SimApi, ctx: ModuleContext, range: Real) -> Option<ObjectId> {
    if let Some(answer) = ai_seams(sim).closest_repulsor(ctx.owner_id, range) {
        return answer;
    }
    let found = crate::ai::ai::find_closest_repulsor(
        &*sim,
        &sim.rules().ai_data,
        ctx.owner_slot,
        range,
    )?;
    sim.object(found).map(|o| o.id)
}

/// `FAST_AS_POSSIBLE`.
pub const FAST_AS_POSSIBLE: Real = 999_999.0;
pub const MAX_WAYPOINTS: usize = 16;
pub const MAX_TURRETS: usize = 2;
const INVALID_WAYPOINT_ID: UnsignedInt = 0x7fff_ffff;
const UNSET_WAYPOINT_ID: UnsignedInt = 0x00fa_cade;
const LOCOMOTORSURFACE_GROUND: Int = 1 << 0;
const LOCOMOTORSURFACE_AIR: Int = 1 << 3;
const PATHFIND_CELL_SIZE_F: Real = 10.0;

/// `GuardTargetType`.
pub const GUARDTARGET_LOCATION: Int = 0;
pub const GUARDTARGET_OBJECT: Int = 1;
pub const GUARDTARGET_AREA: Int = 2;
pub const GUARDTARGET_NONE: Int = 3;

/// `LocoGoalType`; written by `xfer`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocoGoalType {
    None = 0,
    PositionOnPath = 1,
    PositionExplicit = 2,
    Angle = 3,
}

// -----------------------------------------------------------------------------
// Module data
// -----------------------------------------------------------------------------

/// `AIUpdateModuleData`. The locomotor sets live in the thing template.
#[derive(Debug, Clone)]
pub struct AiUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub auto_acquire_enemies_when_idle: UnsignedInt,
    pub mood_attack_check_rate: UnsignedInt,
    pub forbid_player_commands: Bool,
    pub turrets_linked: Bool,
    /// `Turret` and `AltTurret`.
    pub turrets: [Option<TurretAIData>; MAX_TURRETS],
}

impl AiUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            auto_acquire_enemies_when_idle: 0,
            mood_attack_check_rate: LOGICFRAMES_PER_SECOND as UnsignedInt * 2,
            forbid_player_commands: FALSE,
            turrets_linked: FALSE,
            turrets: [None, None],
        }
    }

    /// One field of `AIUpdateModuleData::buildFieldParse`; derived classes add theirs.
    pub fn parse_field(&mut self, field: &str, r: &mut IniReader<'_>) -> Result<(), IniError> {
        const AUTO_ACQUIRE_NAMES: [&str; 5] = [
            "YES",
            "STEALTHED",
            "NO",
            "NOTWHILEATTACKING",
            "ATTACK_BUILDINGS",
        ];
        let turret = if field.eq_ignore_ascii_case("Turret") {
            Some(0)
        } else if field.eq_ignore_ascii_case("AltTurret") {
            Some(1)
        } else {
            None
        };
        if let Some(index) = turret {
            if self.turrets[index].is_some() {
                return Err(IniError::InvalidData);
            }
            let mut data = TurretAIData::default();
            r.init_from_ini(&mut data, &TURRET_AI_DATA_FIELD_PARSE)?;
            self.turrets[index] = Some(data);
            Ok(())
        } else if field.eq_ignore_ascii_case("AutoAcquireEnemiesWhenIdle") {
            r.parse_bit_string32(
                &mut self.auto_acquire_enemies_when_idle,
                &AUTO_ACQUIRE_NAMES,
            )
        } else if field.eq_ignore_ascii_case("MoodAttackCheckRate") {
            r.parse_duration_unsigned_int(&mut self.mood_attack_check_rate)
        } else if field.eq_ignore_ascii_case("ForbidPlayerCommands") {
            r.parse_bool(&mut self.forbid_player_commands)
        } else if field.eq_ignore_ascii_case("TurretsLinked") {
            r.parse_bool(&mut self.turrets_linked)
        } else {
            Err(IniError::UnknownToken)
        }
    }
}

impl ModuleData for AiUpdateModuleData {
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
        FacetMask(0x001 | FacetMask::AI_UPDATE.0)
    }

    fn is_ai_module(&self) -> bool {
        true
    }
}

pub fn parse_ai_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AiUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| data.parse_field(field, r))?;
    Ok(Box::new(data))
}

// -----------------------------------------------------------------------------
// Orders
// -----------------------------------------------------------------------------

/// `AICommandType`, the commands the base class takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiCommandType {
    MoveToPosition,
    MoveToPositionEvenIfSleeping,
    MoveToObject,
    TightenToPosition,
    MoveToPositionAndEvacuate,
    MoveToPositionAndEvacuateAndExit,
    Idle,
    FollowWaypointPath,
    FollowWaypointPathAsTeam,
    FollowWaypointPathExact,
    FollowWaypointPathAsTeamExact,
    FollowPath,
    FollowPathAppend,
    FollowExitProductionPath,
    AttackObject,
    ForceAttackObject,
    GuardRetaliate,
    AttackTeam,
    AttackPosition,
    AttackMoveToPosition,
    AttackFollowWaypointPath,
    AttackFollowWaypointPathAsTeam,
    Hunt,
    AttackArea,
    Repair,
    ResumeConstruction,
    GetHealed,
    GetRepaired,
    Enter,
    Dock,
    Exit,
    ExitInstantly,
    Evacuate,
    EvacuateInstantly,
    ExecuteRailedTransport,
    GuardPosition,
    GuardObject,
    GuardTunnelNetwork,
    GuardArea,
    HackInternet,
    FaceObject,
    FacePosition,
    RappelInto,
    CombatDrop,
    Wander,
    WanderInPlace,
    Panic,
    Busy,
    MoveAwayFromUnit,
}

/// `AICommandParms`.
#[derive(Debug, Clone)]
pub struct AiCommandParms {
    pub cmd: AiCommandType,
    pub source: CommandSourceType,
    pub pos: Coord3D,
    pub obj: Option<ObjectId>,
    pub int_value: Int,
    pub coords: Vec<Coord3D>,
    /// A waypoint's id and name.
    pub waypoint: Option<(UnsignedInt, String)>,
    /// A polygon trigger's name and center point.
    pub area: Option<(String, Coord3D)>,
}

impl AiCommandParms {
    pub fn new(cmd: AiCommandType, source: CommandSourceType) -> Self {
        Self {
            cmd,
            source,
            pos: Coord3D::default(),
            obj: None,
            int_value: 0,
            coords: Vec::new(),
            waypoint: None,
            area: None,
        }
    }
}

/// Everything another part of the world makes an AI do.
#[derive(Debug, Clone)]
pub enum AiCall {
    /// `aiDoCommand`.
    Command(AiCommandParms),
    /// `setTurretEnabled`.
    SetTurretEnabled(WhichTurretType, Bool),
    /// `recenterTurret`.
    RecenterTurret(WhichTurretType),
    /// `onObjectCreated`.
    ObjectCreated,
    /// `loadPostProcess`.
    LoadPostProcess,
    /// `markAsDead`.
    MarkAsDead,
    ChooseLocomotorSet(LocomotorSetType),
    SetLocomotorUpgrade(Bool),
    SetLocomotorGoalNone,
    /// `friend_setGoalObject`.
    SetGoalObject(Option<ObjectId>),
    WakeUpAndAttemptToTarget,
    /// `joinTeam`.
    JoinTeam,
    /// `DozerAIInterface::cancelAllTasks`; nothing for a unit that is no dozer.
    CancelAllDozerTasks,
    /// `doQuickExit`.
    QuickExit(Vec<Coord3D>),
    /// `onDisabledEdge`, which only the dozer and the worker override.
    DisabledEdge(Bool),
    /// `onDelete`, which only the dozer and the worker override.
    Delete,
}

/// The command entry of a resident AI module.
pub type AiCallEntry = fn(&mut dyn SimApi, ModuleContext, &AiCall);

// -----------------------------------------------------------------------------
// The module
// -----------------------------------------------------------------------------

/// `processCollision` of a resident AI module, which runs with the module in the
/// arena.
pub type AiCollisionEntry = fn(&mut dyn SimApi, ModuleContext, ObjectSlot) -> Bool;

/// `AIUpdateInterface`. The fields are in `xfer` order.
pub struct AiUpdate {
    pub data: Arc<AiUpdateModuleData>,
    pub schedule: UpdateScheduleTracker,
    pub prior_waypoint_id: UnsignedInt,
    pub current_waypoint_id: UnsignedInt,
    /// `None` until `onObjectCreated`.
    pub machine_made: Bool,
    pub machine: AiStateMachine,
    pub is_ai_dead: Bool,
    pub is_recruitable: Bool,
    pub next_enemy_scan_time: UnsignedInt,
    pub current_victim_id: ObjectId,
    pub desired_speed: Real,
    pub last_command_source: CommandSourceType,
    pub guard_mode: Int,
    pub guard_target_type: [Int; 2],
    pub location_to_guard: Coord3D,
    pub object_to_guard: ObjectId,
    pub area_to_guard: String,
    pub attack_info: String,
    pub waypoint_count: Int,
    pub waypoint_queue: [Coord3D; MAX_WAYPOINTS],
    pub waypoint_index: Int,
    pub executing_waypoint_queue: Bool,
    pub completed_waypoint: Option<UnsignedInt>,
    pub waiting_for_path: Bool,
    /// `m_path != nullptr`.
    pub has_path: Bool,
    /// `m_path`. Without a pathfind map a path may exist only behind the seams,
    /// with `has_path` set and nothing here.
    pub path: Option<Path>,
    pub requested_victim_id: ObjectId,
    pub requested_destination: Coord3D,
    pub requested_destination2: Coord3D,
    pub ignore_obstacle_id: ObjectId,
    pub path_extra_distance: Real,
    pub pathfind_goal_cell: ICoord2D,
    pub pathfind_cur_cell: ICoord2D,
    pub ignore_collisions_until: UnsignedInt,
    pub queue_for_path_frame: UnsignedInt,
    pub final_position: Coord3D,
    pub do_final_position: Bool,
    pub is_attack_path: Bool,
    pub is_final_goal: Bool,
    pub is_approach_path: Bool,
    pub is_safe_path: Bool,
    pub movement_complete: Bool,
    pub upgraded_locomotors: Bool,
    pub can_path_through_units: Bool,
    pub randomly_offset_mood_check: Bool,
    pub repulsor1: ObjectId,
    pub repulsor2: ObjectId,
    pub move_out_of_way1: ObjectId,
    pub move_out_of_way2: ObjectId,
    pub locomotor_set: LocomotorSet,
    /// `m_curLocomotor`, as an index into the set.
    pub cur_locomotor: Option<usize>,
    pub cur_locomotor_set: LocomotorSetType,
    pub locomotor_goal_type: LocoGoalType,
    pub locomotor_goal_data: Coord3D,
    /// `m_turretAI`.
    pub turrets: turrets::Turrets,
    pub turret_sync_flag: Int,
    pub attitude: AttitudeType,
    pub next_mood_check_time: UnsignedInt,
    pub crate_created: ObjectId,

    // Not saved.
    pub blocked_frames: UnsignedInt,
    pub cur_max_blocked_speed: Real,
    pub bump_speed_limit: Real,
    pub path_timestamp: UnsignedInt,
    pub is_blocked: Bool,
    pub is_blocked_and_stuck: Bool,
    pub is_moving_flag: Bool,
    pub is_in_update: Bool,
    /// `m_retryPath`: the last path only came close.
    pub retry_path: Bool,
    /// The object's `m_iPos`, kept here for the one reader the port has: the
    /// pathfind map learns of a move only when it changes.
    pub whole_position: (Int, Int),

    /// What a class derived from `AIUpdateInterface` adds.
    pub derived: DerivedAi,
}

/// The derived class of an AI module, with its members.
#[derive(Debug, Clone)]
pub enum DerivedAi {
    /// `AIUpdateInterface` itself, which also stands in for the unported classes.
    Base,
    Dozer(Box<DozerAi>),
    Worker(Box<WorkerAi>),
}

impl AiUpdate {
    pub fn new(data: Arc<AiUpdateModuleData>) -> Self {
        let zero = Coord3D::default();
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            prior_waypoint_id: UNSET_WAYPOINT_ID,
            current_waypoint_id: UNSET_WAYPOINT_ID,
            machine_made: FALSE,
            machine: AiStateMachine::default(),
            is_ai_dead: FALSE,
            is_recruitable: TRUE,
            next_enemy_scan_time: 0,
            current_victim_id: ObjectId::INVALID,
            desired_speed: FAST_AS_POSSIBLE,
            last_command_source: CommandSourceType::FromAi,
            guard_mode: 0,
            guard_target_type: [GUARDTARGET_NONE; 2],
            location_to_guard: zero,
            object_to_guard: ObjectId::INVALID,
            area_to_guard: String::new(),
            attack_info: String::new(),
            waypoint_count: 0,
            waypoint_queue: [zero; MAX_WAYPOINTS],
            waypoint_index: 0,
            executing_waypoint_queue: FALSE,
            completed_waypoint: None,
            waiting_for_path: FALSE,
            has_path: FALSE,
            path: None,
            requested_victim_id: ObjectId::INVALID,
            requested_destination: zero,
            requested_destination2: zero,
            ignore_obstacle_id: ObjectId::INVALID,
            path_extra_distance: 0.0,
            pathfind_goal_cell: ICoord2D { x: -1, y: -1 },
            pathfind_cur_cell: ICoord2D { x: -1, y: -1 },
            ignore_collisions_until: 0,
            queue_for_path_frame: 0,
            final_position: zero,
            do_final_position: FALSE,
            is_attack_path: FALSE,
            is_final_goal: FALSE,
            is_approach_path: FALSE,
            is_safe_path: FALSE,
            movement_complete: FALSE,
            upgraded_locomotors: FALSE,
            can_path_through_units: FALSE,
            randomly_offset_mood_check: FALSE,
            repulsor1: ObjectId::INVALID,
            repulsor2: ObjectId::INVALID,
            move_out_of_way1: ObjectId::INVALID,
            move_out_of_way2: ObjectId::INVALID,
            locomotor_set: LocomotorSet::new(),
            cur_locomotor: None,
            cur_locomotor_set: LocomotorSetType::Invalid,
            locomotor_goal_type: LocoGoalType::None,
            locomotor_goal_data: zero,
            turrets: [None, None],
            turret_sync_flag: -1,
            attitude: AttitudeType::Normal,
            next_mood_check_time: 0,
            crate_created: ObjectId::INVALID,
            blocked_frames: 0,
            cur_max_blocked_speed: 0.0,
            bump_speed_limit: FAST_AS_POSSIBLE,
            path_timestamp: 0,
            is_blocked: FALSE,
            is_blocked_and_stuck: FALSE,
            is_moving_flag: FALSE,
            is_in_update: FALSE,
            whole_position: (0, 0),
            retry_path: FALSE,
            derived: DerivedAi::Base,
        }
    }

    /// The rest of a derived class's constructor: `createMachines`.
    fn construct_derived(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        match &mut self.derived {
            DerivedAi::Base => {}
            DerivedAi::Dozer(dozer) => dozer.brain.create_machine(sim, ctx),
            // The supply truck and worker machines enter states that do nothing
            // while the object has no AI yet.
            DerivedAi::Worker(worker) => worker.brain.create_machine(sim, ctx),
        }
    }

    /// `getCurLocomotor`.
    pub fn cur_locomotor(&self) -> Option<&Locomotor> {
        self.cur_locomotor
            .and_then(|i| self.locomotor_set.locomotors().get(i))
    }

    pub(crate) fn cur_locomotor_mut(&mut self) -> Option<&mut Locomotor> {
        let index = self.cur_locomotor?;
        self.locomotor_set.get_mut(index)
    }

    /// `isIdle`.
    pub fn is_idle(&self) -> Bool {
        Bool::from(
            self.machine.current_state_id() == state_id::IDLE
                || self.machine.is_in_idle_state() != FALSE,
        )
    }

    /// `isMoving`.
    pub fn is_moving(&self) -> Bool {
        if self.is_idle() != FALSE {
            return FALSE;
        }
        Bool::from(self.locomotor_goal_type != LocoGoalType::None || self.is_moving_flag != FALSE)
    }

    /// `queueWaypoint`.
    pub fn queue_waypoint(&mut self, pos: &Coord3D) -> Bool {
        let count = self.waypoint_count;
        if count >= MAX_WAYPOINTS as Int {
            return FALSE;
        }
        if let Some(point) = self.waypoint_queue.get_mut(count as u32 as usize) {
            *point = *pos;
        }
        self.waypoint_count = count.wrapping_add(1);
        TRUE
    }

    /// `executeWaypointQueue`.
    pub fn execute_waypoint_queue(&mut self) {
        if self.is_ai_dead != FALSE {
            return;
        }
        if self.waypoint_count > 0 {
            self.waypoint_index = 0;
            self.executing_waypoint_queue = TRUE;
        }
    }

    /// `clearWaypointQueue`.
    pub fn clear_waypoint_queue(&mut self) {
        self.waypoint_count = 0;
        self.executing_waypoint_queue = FALSE;
    }

    /// `destroyPath`.
    pub fn destroy_path(&mut self) {
        self.has_path = FALSE;
        self.path = None;
        self.waiting_for_path = FALSE;
        self.is_attack_path = FALSE;
        self.locomotor_goal_type = LocoGoalType::None;
    }

    /// `setLocomotorGoalPositionOnPath`.
    pub fn set_goal_position_on_path(&mut self) {
        self.locomotor_goal_type = LocoGoalType::PositionOnPath;
        self.locomotor_goal_data = Coord3D::default();
    }

    /// `setFinalPosition`.
    pub fn set_final_position(&mut self, pos: &Coord3D) {
        self.final_position = *pos;
        self.do_final_position = FALSE;
    }

    /// `setCanPathThroughUnits`.
    pub fn set_can_path_through_units(&mut self, can: Bool) {
        self.can_path_through_units = can;
        if can != FALSE {
            self.is_blocked_and_stuck = FALSE;
        }
    }

    /// `m_locomotorGoalType != NONE`.
    pub fn is_locomotor_goal_set(&self) -> bool {
        self.locomotor_goal_type != LocoGoalType::None
    }

    /// `isAttacking`.
    fn is_attacking_state(&self) -> bool {
        matches!(
            self.machine.current_state_id(),
            state_id::ATTACK_POSITION
                | state_id::ATTACK_OBJECT
                | state_id::FORCE_ATTACK_OBJECT
                | state_id::ATTACK_AND_FOLLOW_OBJECT
        ) || self.machine.current_state_id() == state_id::ATTACK_MOVE_TO
            && self.machine.moves.attack_move.sub.is_in_attack_state()
    }

    /// `setLocomotorGoalPositionExplicit`.
    pub fn set_goal_position_explicit(&mut self, pos: &Coord3D) {
        self.locomotor_goal_type = LocoGoalType::PositionExplicit;
        self.locomotor_goal_data = *pos;
    }

    /// `setLocomotorGoalOrientation`.
    pub fn set_goal_orientation(&mut self, angle: Real) {
        self.locomotor_goal_type = LocoGoalType::Angle;
        self.locomotor_goal_data.x = angle;
    }

    /// `friend_startingMove`.
    pub fn starting_move(&mut self) {
        self.movement_complete = FALSE;
        self.is_moving_flag = TRUE;
        self.blocked_frames = 0;
        self.is_blocked_and_stuck = FALSE;
    }

    /// `friend_endingMove`.
    pub fn ending_move(&mut self) {
        self.movement_complete = TRUE;
        self.is_moving_flag = FALSE;
    }

    fn clear_blocked(&mut self) {
        self.blocked_frames = 0;
        self.is_blocked = FALSE;
        self.is_blocked_and_stuck = FALSE;
    }

    /// `chooseLocomotorSetExplicit` once the new set's locomotors exist.
    fn install_set(&mut self, set: LocomotorSet, set_type: LocomotorSetType) {
        self.locomotor_set = set;
        self.cur_locomotor = None;
        self.cur_locomotor_set = set_type;
    }

    /// `chooseGoodLocomotorFromCurrentSet` with the pathfinder's answer.
    fn adopt_locomotor(&mut self, chosen: Option<usize>) {
        let previous = self.cur_locomotor;
        let next = chosen
            .filter(|&i| i < self.locomotor_set.locomotors().len())
            .or(previous)
            .or_else(|| {
                self.locomotor_set
                    .locomotors()
                    .iter()
                    .position(|l| l.get_legal_surfaces() & LOCOMOTORSURFACE_GROUND != 0)
            });
        self.cur_locomotor = next;
        if previous != next {
            if let Some(locomotor) = self.cur_locomotor_mut() {
                locomotor.set_use_precise_z_pos(FALSE);
                locomotor.set_no_slow_down(FALSE);
                locomotor.set_ultra_accurate(FALSE);
            }
        }
    }

    /// `AIUpdateInterface::xfer`, writing `version` 4 (retail layout) or 5.
    /// Loading is not supported. In CRC mode the state machine, the path, the
    /// locomotors and the turrets add nothing, as their `crc` functions are empty.
    pub fn xfer_as_version(
        &mut self,
        xfer: &mut dyn Xfer,
        sim: &dyn SimView,
        owner: ObjectId,
        version: XferVersion,
    ) -> Result<(), XferError> {
        if xfer.mode() == XferMode::Load {
            return Err(XferError::InvalidParameters);
        }
        if matches!(self.derived, DerivedAi::Base) {
            return self.xfer_base(xfer, sim, owner, version);
        }
        // The dozer and worker versions follow the same switch as the base's.
        let derived_version: XferVersion = if version < 5 { 1 } else { 2 };
        let mut written = derived_version;
        xfer.xfer_version(&mut written, derived_version)?;
        self.xfer_base(xfer, sim, owner, version)?;
        match &mut self.derived {
            DerivedAi::Base => Ok(()),
            DerivedAi::Dozer(dozer) => dozer.brain.xfer(xfer, derived_version),
            DerivedAi::Worker(worker) => worker.xfer(xfer, derived_version),
        }
    }

    /// `AIUpdateInterface::xfer`.
    fn xfer_base(
        &mut self,
        xfer: &mut dyn Xfer,
        sim: &dyn SimView,
        owner: ObjectId,
        version: XferVersion,
    ) -> Result<(), XferError> {
        let crc = xfer.mode() == XferMode::Crc;
        let mut seams = ai_seams(sim);
        let mut written = version;
        xfer.xfer_version(&mut written, version)?;
        if let Some(live) = self.schedule.mod_slot.and_then(|s| sim.module_schedule(s)) {
            self.schedule.schedule = live;
        }
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.prior_waypoint_id)?;
        xfer.xfer_unsigned_int(&mut self.current_waypoint_id)?;
        self.machine.xfer(xfer, owner, &mut seams)?;
        xfer.xfer_bool(&mut self.is_ai_dead)?;
        xfer.xfer_bool(&mut self.is_recruitable)?;
        xfer.xfer_unsigned_int(&mut self.next_enemy_scan_time)?;
        xfer.xfer_object_id(&mut self.current_victim_id)?;
        xfer.xfer_real(&mut self.desired_speed)?;
        let mut source = self.last_command_source as Int;
        xfer.xfer_int(&mut source)?;
        let [mut first, mut second] = self.guard_target_type;
        xfer.xfer_int(&mut first)?;
        xfer.xfer_int(&mut second)?;
        if version < 5 {
            // The retail layout writes the second entry again, then the first
            // four bytes of the location.
            xfer.xfer_int(&mut second)?;
            let mut x = self.location_to_guard.x;
            xfer.xfer_real(&mut x)?;
        }
        xfer.xfer_coord3d(&mut self.location_to_guard)?;
        xfer.xfer_object_id(&mut self.object_to_guard)?;
        xfer.xfer_ascii_string(&mut self.area_to_guard)?;
        xfer.xfer_ascii_string(&mut self.attack_info)?;
        xfer.xfer_int(&mut self.waypoint_count)?;
        if self.waypoint_count < 0 || self.waypoint_count > MAX_WAYPOINTS as Int {
            return Err(XferError::InvalidData);
        }
        for point in self
            .waypoint_queue
            .iter_mut()
            .take(self.waypoint_count as u32 as _)
        {
            xfer.xfer_coord3d(point)?;
        }
        xfer.xfer_int(&mut self.waypoint_index)?;
        xfer.xfer_bool(&mut self.executing_waypoint_queue)?;
        let mut completed = self.completed_waypoint.unwrap_or(INVALID_WAYPOINT_ID);
        xfer.xfer_unsigned_int(&mut completed)?;
        xfer.xfer_bool(&mut self.waiting_for_path)?;
        let mut got_path = self.has_path;
        xfer.xfer_bool(&mut got_path)?;
        if got_path != FALSE && !crc {
            match self.path.as_mut() {
                Some(path) => path.xfer(xfer)?,
                None => seams.path_xfer(owner, xfer)?,
            }
        }
        xfer.xfer_object_id(&mut self.requested_victim_id)?;
        xfer.xfer_coord3d(&mut self.requested_destination)?;
        xfer.xfer_coord3d(&mut self.requested_destination2)?;
        xfer.xfer_object_id(&mut self.ignore_obstacle_id)?;
        xfer.xfer_real(&mut self.path_extra_distance)?;
        xfer.xfer_icoord2d(&mut self.pathfind_goal_cell)?;
        xfer.xfer_icoord2d(&mut self.pathfind_cur_cell)?;
        xfer.xfer_unsigned_int(&mut self.ignore_collisions_until)?;
        xfer.xfer_unsigned_int(&mut self.queue_for_path_frame)?;
        xfer.xfer_coord3d(&mut self.final_position)?;
        xfer.xfer_bool(&mut self.do_final_position)?;
        xfer.xfer_bool(&mut self.is_attack_path)?;
        xfer.xfer_bool(&mut self.is_final_goal)?;
        xfer.xfer_bool(&mut self.is_approach_path)?;
        xfer.xfer_bool(&mut self.is_safe_path)?;
        xfer.xfer_bool(&mut self.movement_complete)?;
        // The reference writes `m_isSafePath` twice.
        xfer.xfer_bool(&mut self.is_safe_path)?;
        xfer.xfer_bool(&mut self.upgraded_locomotors)?;
        xfer.xfer_bool(&mut self.can_path_through_units)?;
        xfer.xfer_bool(&mut self.randomly_offset_mood_check)?;
        xfer.xfer_object_id(&mut self.repulsor1)?;
        xfer.xfer_object_id(&mut self.repulsor2)?;
        xfer.xfer_object_id(&mut self.move_out_of_way1)?;
        xfer.xfer_object_id(&mut self.move_out_of_way2)?;
        let current = self
            .cur_locomotor()
            .map(|l| l.get_template_name().to_owned());
        self.locomotor_set.xfer_self_and_cur_loco_ptr(
            xfer,
            &sim.rules().locomotor_store,
            current.as_deref(),
        )?;
        let mut set_type = self.cur_locomotor_set.as_i32();
        xfer.xfer_int(&mut set_type)?;
        let mut goal_type = self.locomotor_goal_type as Int;
        xfer.xfer_int(&mut goal_type)?;
        xfer.xfer_coord3d(&mut self.locomotor_goal_data)?;
        if !crc {
            // The turret machine's version follows the same switch as the base's.
            let machine: XferVersion = if version < 5 { 1 } else { 2 };
            for turret in self.turrets.iter_mut().flatten() {
                turret.xfer_with_machine_version(xfer, machine)?;
            }
        }
        xfer.xfer_int(&mut self.turret_sync_flag)?;
        let mut attitude = self.attitude as Int;
        xfer.xfer_int(&mut attitude)?;
        xfer.xfer_unsigned_int(&mut self.next_mood_check_time)?;
        xfer.xfer_object_id(&mut self.crate_created)
    }
}

/// The module of `slot`, borrowed for one statement.
pub fn ai(sim: &mut dyn SimApi, ctx: ModuleContext) -> &mut AiUpdate {
    sim.resident_state_mut(ctx.mod_slot)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("a resident module stays in the arena while it runs")
}

// -----------------------------------------------------------------------------
// Sleeping
// -----------------------------------------------------------------------------

/// `wakeUpNow`. During the update the returned sleep time says it instead.
pub fn wake_up_now(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if sim.get_wake_frame(ctx.mod_slot).0 > UpdateSleepTime::NONE.0
        && ai(sim, ctx).is_in_update == FALSE
    {
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }
}

/// `setQueueForPathTime`.
pub fn set_queue_for_path_time(sim: &mut dyn SimApi, ctx: ModuleContext, frames: UnsignedInt) {
    if frames >= UpdateSleepTime::NONE.0
        && sim.get_wake_frame(ctx.mod_slot).0 > frames
        && ai(sim, ctx).is_in_update == FALSE
    {
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(frames));
    }
    let when = if frames != 0 {
        sim.frame().wrapping_add(frames)
    } else {
        0
    };
    ai(sim, ctx).queue_for_path_frame = when;
}

// -----------------------------------------------------------------------------
// Questions about the unit
// -----------------------------------------------------------------------------

/// `Object::isMobile`.
fn is_mobile(sim: &dyn SimApi, owner: ObjectSlot) -> bool {
    sim.object_is_kind_of(owner, KindOfType::Immobile) == FALSE
        && sim
            .object(owner)
            .is_some_and(|o| o.disabled_mask.to_unsigned_int() == 0)
}

fn is_disabled(sim: &dyn SimApi, owner: ObjectSlot, by: DisabledType) -> bool {
    sim.object(owner)
        .is_some_and(|o| o.is_disabled_by_type(by) != FALSE)
}

fn is_projectile(sim: &dyn SimApi, owner: ObjectSlot) -> bool {
    sim.object_is_kind_of(owner, KindOfType::Projectile) != FALSE
}

fn position_of(sim: &dyn SimApi, owner: ObjectSlot) -> Coord3D {
    sim.object(owner)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn layer_of(sim: &dyn SimApi, owner: ObjectSlot) -> PathfindLayerEnum {
    sim.object_layer(owner).unwrap_or(PathfindLayerEnum::Ground)
}

/// `isDoingGroundMovement`.
pub fn is_doing_ground_movement(sim: &mut dyn SimApi, ctx: ModuleContext) -> Bool {
    let owner = ctx.owner_slot;
    if is_disabled(sim, owner, DisabledType::Unmanned)
        && sim.object_is_kind_of(owner, KindOfType::ProducedAtHelipad) != FALSE
    {
        return TRUE;
    }
    let state = ai(sim, ctx);
    if state.locomotor_set.get_valid_surfaces() == LOCOMOTORSURFACE_AIR {
        return FALSE;
    }
    let Some(locomotor) = state.cur_locomotor() else {
        return FALSE;
    };
    if locomotor.get_legal_surfaces() & LOCOMOTORSURFACE_AIR != 0 {
        return FALSE;
    }
    if is_disabled(sim, owner, DisabledType::Held) {
        return FALSE;
    }
    if sim.object_is_above_terrain(owner) != FALSE
        && physics_update::physics_mut(sim, owner)
            .is_some_and(|p| p.flag(physics_update::ALLOW_TO_FALL))
    {
        return FALSE;
    }
    TRUE
}

/// What the mood code of [`mood`] asks this module about itself. The attack
/// priority sets are the script engine's, which no frame fills yet.
fn mood_unit(state: &AiUpdate) -> mood::MoodUnit<'static> {
    mood::MoodUnit {
        has_state_machine: state.machine_made,
        attitude: state.attitude,
        valid_surfaces: state.locomotor_set.get_valid_surfaces(),
        has_primary_turret: Bool::from(state.data.turrets[0].is_some()),
        is_attacking: Bool::from(state.is_attacking_state()),
        auto_acquire_enemies_when_idle: state.data.auto_acquire_enemies_when_idle,
        mood_attack_check_rate: state.data.mood_attack_check_rate,
        attack_info: None,
    }
}

/// `getMoodMatrixValue`.
pub fn mood_matrix_value(sim: &mut dyn SimApi, ctx: ModuleContext) -> UnsignedInt {
    let unit = mood_unit(ai(sim, ctx));
    mood::mood_matrix_value(sim, ctx.owner_slot, &unit)
}

/// `getMoodMatrixActionAdjustment`.
pub fn mood_matrix_action_adjustment(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    action: MoodMatrixAction,
) -> UnsignedInt {
    let unit = mood_unit(ai(sim, ctx));
    mood::mood_matrix_action_adjustment(sim, ctx.owner_slot, &unit, action)
}

/// `resetNextMoodCheckTime`.
pub fn reset_next_mood_check_time(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let mut timer = mood::MoodState::default();
    timer.reset_next_mood_check_time(sim.frame(), &sim.rules().ai_data);
    let state = ai(sim, ctx);
    state.next_mood_check_time = timer.next_mood_check_time;
    state.randomly_offset_mood_check = timer.randomly_offset_mood_check;
}

/// `setCurrentVictim`. Telling the old victim it lost a targeter only matters to
/// jets, which are not ported.
pub fn set_current_victim(sim: &mut dyn SimApi, ctx: ModuleContext, victim: Option<ObjectId>) {
    ai(sim, ctx).current_victim_id = victim.unwrap_or(ObjectId::INVALID);
}

/// `checkForCrateToPickup`.
pub fn check_for_crate_to_pickup(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<ObjectId> {
    let id = ai(sim, ctx).crate_created;
    if id == ObjectId::INVALID {
        return None;
    }
    ai(sim, ctx).crate_created = ObjectId::INVALID;
    if cfg!(feature = "retail-crc") {
        return None;
    }
    let crate_slot = sim.find_object(id)?;
    sim.would_like_to_collide(crate_slot, ctx.owner_slot)
        .then_some(id)
}

// -----------------------------------------------------------------------------
// Locomotors
// -----------------------------------------------------------------------------

/// The locomotors of one of the template's sets, new (three draws each), or
/// `None` when the template does not have the set.
fn new_locomotor_set(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    set_type: LocomotorSetType,
) -> Option<LocomotorSet> {
    let names: Vec<String> = sim
        .object(owner)?
        .thing
        .template_override()
        .get_final(&sim.rules().thing_factory)?
        .get_locomotor_set(set_type)
        .to_vec();
    if names.is_empty() {
        return None;
    }
    let mut set = LocomotorSet::new();
    for name in &names {
        if let Some(locomotor) = sim.new_locomotor(name) {
            set.add_locomotor(locomotor);
        }
    }
    Some(set)
}

/// `chooseLocomotorSet` on a module at hand (the constructor has it, entry
/// points reach it through [`choose_locomotor_set`]).
fn choose_locomotor_set_on(
    state: &mut AiUpdate,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    set_type: LocomotorSetType,
) {
    let wanted = effective_set(state, set_type);
    if wanted == state.cur_locomotor_set {
        return;
    }
    let Some(set) = new_locomotor_set(sim, ctx.owner_slot, wanted) else {
        return;
    };
    state.install_set(set, wanted);
    let (layer, pos) = (
        layer_of(sim, ctx.owner_slot),
        position_of(sim, ctx.owner_slot),
    );
    let chosen = best_locomotor(sim, ctx, &state.locomotor_set, layer, &pos);
    state.adopt_locomotor(chosen);
    mirror_locomotor(sim, ctx.owner_slot, wanted, state.cur_locomotor.is_some());
}

fn effective_set(state: &AiUpdate, set_type: LocomotorSetType) -> LocomotorSetType {
    if set_type == LocomotorSetType::Normal && state.upgraded_locomotors != FALSE {
        LocomotorSetType::NormalUpgraded
    } else {
        set_type
    }
}

/// The object fields other code reads.
fn mirror_locomotor(sim: &mut dyn SimApi, owner: ObjectSlot, set: LocomotorSetType, has: bool) {
    if let Some(o) = sim.object_mut(owner) {
        o.ai_locomotor_set = Some(set);
        o.cur_locomotor_set = set;
        o.ai_has_locomotor = Bool::from(has);
    }
}

/// `chooseBestLocomotorForPosition`: the index in the set. Without a pathfind
/// map the seam answers.
fn best_locomotor(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    set: &LocomotorSet,
    layer: PathfindLayerEnum,
    pos: &Coord3D,
) -> Option<usize> {
    let surfaces = sim.pathfinder().map(|p| p.locomotor_surfaces_at(layer, pos));
    let locomotors = set.locomotors();
    match surfaces {
        Some(mask) => locomotors
            .iter()
            .position(|l| l.get_legal_surfaces() & mask != 0),
        None => {
            ai_seams(sim).choose_best_locomotor(ctx.owner_id, layer, locomotors.len(), pos)
        }
    }
}

/// `chooseLocomotorSet`.
pub fn choose_locomotor_set(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    set_type: LocomotorSetType,
) -> Bool {
    let wanted = effective_set(ai(sim, ctx), set_type);
    if wanted == ai(sim, ctx).cur_locomotor_set {
        return TRUE;
    }
    let Some(set) = new_locomotor_set(sim, ctx.owner_slot, wanted) else {
        return FALSE;
    };
    ai(sim, ctx).install_set(set, wanted);
    choose_good_locomotor(sim, ctx);
    let has = ai(sim, ctx).cur_locomotor.is_some();
    mirror_locomotor(sim, ctx.owner_slot, wanted, has);
    TRUE
}

/// `chooseGoodLocomotorFromCurrentSet`.
pub fn choose_good_locomotor(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let (layer, pos) = (
        layer_of(sim, ctx.owner_slot),
        position_of(sim, ctx.owner_slot),
    );
    let set = ai(sim, ctx).locomotor_set.clone();
    let chosen = best_locomotor(sim, ctx, &set, layer, &pos);
    ai(sim, ctx).adopt_locomotor(chosen);
}

/// `setLocomotorUpgrade`.
fn set_locomotor_upgrade(sim: &mut dyn SimApi, ctx: ModuleContext, upgraded: Bool) {
    ai(sim, ctx).upgraded_locomotors = upgraded;
    if matches!(
        ai(sim, ctx).cur_locomotor_set,
        LocomotorSetType::Normal | LocomotorSetType::NormalUpgraded
    ) {
        choose_locomotor_set(sim, ctx, LocomotorSetType::Normal);
    }
}

pub fn set_locomotor_goal_none(sim: &mut dyn SimApi, ctx: ModuleContext) {
    ai(sim, ctx).locomotor_goal_type = LocoGoalType::None;
}

/// Runs a movement function of the current locomotor on a copy, then stores it:
/// the locomotor lives in the module, which the movement code reads the world
/// around.
fn drive<R>(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    f: impl FnOnce(&mut Locomotor, &mut LocomotorWorld<'_>) -> R,
) -> Option<R> {
    let mut locomotor = ai(sim, ctx).cur_locomotor()?.clone();
    let mut seams = ai_seams(sim);
    let result = f(
        &mut locomotor,
        &mut LocomotorWorld {
            sim: &mut *sim,
            owner: ctx.owner_slot,
            seams: &mut seams,
        },
    );
    if let Some(current) = ai(sim, ctx).cur_locomotor_mut() {
        *current = locomotor;
    }
    Some(result)
}

/// The speed to ask of the locomotor.
fn goal_speed(sim: &mut dyn SimApi, ctx: ModuleContext) -> Real {
    let condition = sim.object_body_damage_state(ctx.owner_slot);
    let penalty = sim.rules().global_data.movement_penalty_damage_state;
    let state = ai(sim, ctx);
    let max = state
        .cur_locomotor()
        .map_or(0.0, |l| l.get_max_speed_for_condition(condition, penalty));
    let speed = state.desired_speed;
    if speed == FAST_AS_POSSIBLE || speed > max {
        max
    } else {
        speed
    }
}

/// `doLocomotor`.
pub fn do_locomotor(sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
    let owner = ctx.owner_slot;
    if sim.object_is_kind_of(owner, KindOfType::Immobile) != FALSE {
        return UpdateSleepTime::FOREVER;
    }
    choose_good_locomotor(sim, ctx);

    let state = ai(sim, ctx);
    if state.is_blocked != FALSE {
        state.blocked_frames = state.blocked_frames.wrapping_add(1);
    } else {
        state.blocked_frames = 0;
    }
    state.is_blocked = FALSE;
    let mut blocked = Bool::from(state.blocked_frames > 0);
    let mut requires_constant_calling = TRUE;

    if let Some(works_when_dead) = state
        .cur_locomotor()
        .map(|l| l.template.locomotor_works_when_dead)
    {
        if let Some(locomotor) = ai(sim, ctx).cur_locomotor().cloned() {
            locomotor.set_physics_options(sim, owner);
        }
        let state = ai(sim, ctx);
        if state.is_ai_dead == FALSE || works_when_dead != FALSE {
            let goal = state.locomotor_goal_data;
            match state.locomotor_goal_type {
                LocoGoalType::PositionExplicit => {
                    let speed = goal_speed(sim, ctx);
                    drive(sim, ctx, |l, w| {
                        l.move_towards_position(w, &goal, 0.0, speed, &mut blocked)
                    });
                    ai(sim, ctx).do_final_position = FALSE;
                }
                LocoGoalType::PositionOnPath => {
                    if path_follow::drive_on_path(sim, ctx, &mut blocked).is_err() {
                        return UpdateSleepTime::FOREVER;
                    }
                }
                LocoGoalType::Angle => {
                    drive(sim, ctx, |l, w| l.move_towards_angle(w, goal.x));
                    ai(sim, ctx).do_final_position = FALSE;
                }
                LocoGoalType::None => {
                    if ai(sim, ctx).do_final_position != FALSE {
                        move_to_final_position(sim, ctx);
                    }
                    requires_constant_calling =
                        drive(sim, ctx, |l, w| l.maintain_current_position(w)).unwrap_or(TRUE);
                }
            }
        }
        let state = ai(sim, ctx);
        if blocked == FALSE && state.blocked_frames > 1 {
            state.blocked_frames = 1;
        }
        let ceiling = state
            .cur_locomotor()
            .map_or(0, |l| l.template.airborne_targeting_height);
        if sim.object_height_above_terrain(owner) > ceiling as Real {
            sim.set_status(owner, ObjectStatusType::AirborneTarget);
        } else {
            sim.clear_status(owner, ObjectStatusType::AirborneTarget);
        }
        ai(sim, ctx).cur_max_blocked_speed = FAST_AS_POSSIBLE;
    }

    let state = ai(sim, ctx);
    if state.cur_locomotor.is_some()
        && state.locomotor_goal_type == LocoGoalType::None
        && state.do_final_position == FALSE
        && state.is_blocked == FALSE
        && requires_constant_calling == FALSE
    {
        UpdateSleepTime::FOREVER
    } else {
        UpdateSleepTime::NONE
    }
}

/// The drift to `m_finalPosition` of a unit without a goal.
fn move_to_final_position(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    let mut pos = position_of(sim, owner);
    let on_ground = sim.object_is_above_terrain(owner) == FALSE
        && layer_of(sim, owner) == PathfindLayerEnum::Ground;
    let goal = ai(sim, ctx).final_position;
    let dx = goal.x - pos.x;
    let dy = goal.y - pos.y;
    let distance_sqr = dx * dx + dy * dy;
    if distance_sqr < 0.25 {
        ai(sim, ctx).do_final_position = FALSE;
        let z = if on_ground {
            sim.terrain().get_ground_height(goal.x, goal.y, None)
        } else {
            pos.z
        };
        let state = ai(sim, ctx);
        state.final_position.z = z;
        let target = state.final_position;
        sim.set_object_position(owner, &target);
    } else {
        let mut distance = gm_sqrtf(distance_sqr);
        if distance < 1.0 {
            distance = 1.0;
        }
        let frames = LOGICFRAMES_PER_SECOND as Real;
        pos.x += 2.0 * PATHFIND_CELL_SIZE_F * dx / (distance * frames);
        pos.y += 2.0 * PATHFIND_CELL_SIZE_F * dy / (distance * frames);
        if on_ground {
            pos.z = sim.terrain().get_ground_height(pos.x, pos.y, None);
        }
        sim.set_object_position(owner, &pos);
    }
}

// -----------------------------------------------------------------------------
// The frame
// -----------------------------------------------------------------------------

/// `AIUpdateInterface::update`.
pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
    let owner = ctx.owner_slot;
    let state = ai(sim, ctx);
    state.is_in_update = TRUE;
    state.completed_waypoint = None;
    let mut sleep = UpdateSleepTime::FOREVER;

    match machine::update_state_machine(sim, ctx) {
        StateReturnType::Sleep(frames) => {
            if frames < sleep.0 {
                sleep = UpdateSleepTime(frames);
            }
        }
        _ => sleep = UpdateSleepTime::NONE,
    }

    if ai(sim, ctx).movement_complete != FALSE {
        finish_movement(sim, ctx);
    }

    let now = sim.frame();
    let queue_frame = ai(sim, ctx).queue_for_path_frame;
    if queue_frame != 0 {
        if now >= queue_frame {
            let _ = sim.queue_for_path(ctx.owner_id);
            set_queue_for_path_time(sim, ctx, 0);
        } else if queue_frame - now < sleep.0 {
            sleep = UpdateSleepTime(queue_frame - now);
        }
    }

    let turrets_run = sim.object(owner).is_some_and(|o| {
        o.is_effectively_dead() == FALSE
            && [
                DisabledType::Paralyzed,
                DisabledType::Unmanned,
                DisabledType::Emp,
                DisabledType::Subdued,
                DisabledType::Hacked,
            ]
            .iter()
            .all(|&d| o.is_disabled_by_type(d) == FALSE)
    });
    if turrets_run {
        for turret in 0..MAX_TURRETS {
            if let Some(wanted) = turrets::update(sim, ctx, turret) {
                if wanted < sleep.0 {
                    sleep = UpdateSleepTime(wanted);
                }
            }
        }
    }

    let state = ai(sim, ctx);
    if state.is_ai_dead != FALSE && state.machine.current_state_id() != state_id::DEAD {
        machine::clear(sim, ctx);
        machine::set_state(sim, ctx, state_id::DEAD);
        ai(sim, ctx).machine.lock();
        sleep = UpdateSleepTime::NONE;
    }

    let wanted = do_locomotor(sim, ctx);
    if wanted.0 < sleep.0 {
        sleep = wanted;
    }

    let state = ai(sim, ctx);
    state.is_in_update = FALSE;
    if state.completed_waypoint.is_some() {
        UpdateSleepTime::NONE
    } else {
        sleep
    }
}

/// The `m_movementComplete` part of `update`.
fn finish_movement(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    set_queue_for_path_time(sim, ctx, 0);
    ai(sim, ctx).destroy_path();
    set_locomotor_goal_none(sim, ctx);
    sim.clear_model_condition_state(owner, ModelConditionFlagType::Moving);
    if let Some(mut goal) = goal_position(sim, ctx) {
        let pos = position_of(sim, owner);
        let dx = goal.x - pos.x;
        let dy = goal.y - pos.y;
        if dx * dx + dy * dy >= PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F {
            goal = path_follow::snap_position(sim, ctx, pos);
        }
        let state = ai(sim, ctx);
        state.final_position = goal;
        state.do_final_position = FALSE;
        let layer = layer_of(sim, owner);
        update_goal(sim, ctx, &goal, layer);
    }
    let state = ai(sim, ctx);
    state.movement_complete = FALSE;
    state.ignore_obstacle_id = ObjectId::INVALID;
}

/// `doPathfind`: the path queue reached this unit. Without a pathfind map the
/// seam computes the path.
pub fn do_pathfind(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if ai(sim, ctx).waiting_for_path == FALSE {
        return;
    }
    ai(sim, ctx).waiting_for_path = FALSE;
    if sim.pathfinder().is_some() {
        path_follow::do_pathfind_on_map(sim, ctx);
        return;
    }
    let state = ai(sim, ctx);
    // The safe and approach searches replace the path and end there.
    let early = state.is_safe_path != FALSE
        || (state.is_approach_path != FALSE && is_doing_ground_movement(sim, ctx) != FALSE);
    if early {
        ai(sim, ctx).destroy_path();
    } else if ai(sim, ctx).is_approach_path != FALSE {
        ai(sim, ctx).is_approach_path = FALSE;
    }
    let has_path = ai_seams(sim).compute_path(ctx.owner_id);
    ai(sim, ctx).has_path = has_path;
    if early {
        return;
    }
    if ai(sim, ctx).queue_for_path_frame > sim.frame() {
        ai(sim, ctx).waiting_for_path = TRUE;
    }
    if ai(sim, ctx).waiting_for_path == FALSE {
        wake_up_now(sim, ctx);
    }
}

// -----------------------------------------------------------------------------
// Orders
// -----------------------------------------------------------------------------

fn set_command_source(sim: &mut dyn SimApi, ctx: ModuleContext, source: CommandSourceType) {
    ai(sim, ctx).last_command_source = source;
    if let Some(o) = sim.object_mut(ctx.owner_slot) {
        o.ai_last_command_source = source;
    }
}

/// `StateMachine::setGoalObject` with the object's id.
fn set_goal_object(sim: &mut dyn SimApi, ctx: ModuleContext, id: Option<ObjectId>) {
    let goal = id.and_then(|id| {
        let slot = sim.find_object(id)?;
        Some((id, position_of(sim, slot)))
    });
    ai(sim, ctx).machine.set_goal_object(goal);
}

/// `friend_setGoalObject`: past the lock.
pub(crate) fn friend_set_goal_object(sim: &mut dyn SimApi, ctx: ModuleContext, id: Option<ObjectId>) {
    let locked = ai(sim, ctx).machine.locked;
    ai(sim, ctx).machine.unlock();
    set_goal_object(sim, ctx, id);
    if locked != FALSE {
        ai(sim, ctx).machine.lock();
    }
}

/// `setGoalPositionClipped`.
fn set_goal_position_clipped(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
    source: CommandSourceType,
) {
    let mut clipped = *pos;
    if source == CommandSourceType::FromPlayer {
        let mut fudge = sim.rules().global_data.partition_cell_size * 0.5;
        if sim.object_is_kind_of(ctx.owner_slot, KindOfType::Aircraft) != FALSE
            && sim.object_is_significantly_above_terrain(ctx.owner_slot) != FALSE
        {
            if let Some(locomotor) = ai(sim, ctx).cur_locomotor() {
                let height = locomotor.preferred_height;
                // The reference's `max` macro.
                fudge = if fudge > height { fudge } else { height };
            }
        }
        let region = sim.terrain_extent();
        if clipped.x < region.lo.x + fudge {
            clipped.x = region.lo.x + fudge;
        }
        if clipped.x > region.hi.x - fudge {
            clipped.x = region.hi.x - fudge;
        }
        if clipped.y < region.lo.y + fudge {
            clipped.y = region.lo.y + fudge;
        }
        if clipped.y > region.hi.y - fudge {
            clipped.y = region.hi.y - fudge;
        }
    }
    ai(sim, ctx).machine.set_goal_position(Some(&clipped));
}

fn set_max_shot_count(sim: &mut dyn SimApi, owner: ObjectSlot, max_shots: Int) {
    if let Some(o) = sim.object_mut(owner) {
        let slot = o.weapon_set.get_cur_weapon_slot();
        if let Some(weapon) = o.weapon_set.get_weapon_in_weapon_slot_mut(slot) {
            weapon.max_shot_count = max_shots;
        }
    }
}

/// `isAllowedToRespondToAiCommands`.
pub(super) fn is_allowed_to_respond(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) -> bool {
    if sim
        .object(ctx.owner_slot)
        .map_or(true, |o| o.is_effectively_dead() != FALSE)
    {
        return false;
    }
    let mood = mood_matrix_value(sim, ctx);
    if mood & MM_CONTROLLER_AI != 0
        && mood & MM_MOOD_SLEEP != 0
        && parms.cmd != AiCommandType::MoveToPositionEvenIfSleeping
    {
        return false;
    }
    !(parms.source == CommandSourceType::FromPlayer
        && ai(sim, ctx).data.forbid_player_commands != FALSE)
}

/// `aiDoCommand`, which the dozer and the worker override.
pub fn execute_command(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    match ai(sim, ctx).derived {
        DerivedAi::Base => base_command(sim, ctx, parms),
        DerivedAi::Dozer(_) => dozer_ai_update::do_command(sim, ctx, parms),
        DerivedAi::Worker(_) => worker_ai_update::do_command(sim, ctx, parms),
    }
}

/// `AIUpdateInterface::aiDoCommand`.
pub(super) fn base_command(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    if !is_allowed_to_respond(sim, ctx, parms) {
        return;
    }
    use AiCommandType as C;
    let source = parms.source;
    let pos = parms.pos;
    let obj = parms.obj.filter(|&id| sim.find_object(id).is_some());
    let owner = ctx.owner_slot;
    match parms.cmd {
        C::MoveToPosition | C::MoveToPositionEvenIfSleeping => {
            private_move_to_position(sim, ctx, &pos, source)
        }
        C::MoveToObject => {
            if ai(sim, ctx).is_ai_dead != FALSE || !is_mobile(sim, owner) {
                return;
            }
            begin_goal_order(sim, ctx, obj, source, state_id::MOVE_TO, true);
        }
        C::TightenToPosition => {
            if !is_mobile(sim, owner) {
                return;
            }
            machine::clear(sim, ctx);
            set_goal_object(sim, ctx, None);
            set_goal_position_clipped(sim, ctx, &pos, source);
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state_id::MOVE_AND_TIGHTEN);
        }
        C::MoveToPositionAndEvacuate => {
            move_order(sim, ctx, &pos, source, state_id::MOVE_AND_EVACUATE)
        }
        // `DeliverPayloadAIUpdate` takes this one over (S17).
        C::MoveToPositionAndEvacuateAndExit => {
            move_order(sim, ctx, &pos, source, state_id::MOVE_AND_EVACUATE_AND_EXIT)
        }
        C::Idle => private_idle(sim, ctx, source),
        C::FollowWaypointPath => waypoint_order(
            sim,
            ctx,
            parms,
            state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS,
        ),
        C::FollowWaypointPathAsTeam => {
            waypoint_order(sim, ctx, parms, state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM)
        }
        // `privateFollowWaypointPathExact` is not told apart from the loose one
        // by anything but its state.
        C::FollowWaypointPathExact => waypoint_order(
            sim,
            ctx,
            parms,
            state_id::FOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS_EXACT,
        ),
        C::FollowWaypointPathAsTeamExact => waypoint_order(
            sim,
            ctx,
            parms,
            state_id::FOLLOW_WAYPOINT_PATH_AS_TEAM_EXACT,
        ),
        C::FollowPath => private_follow_path(sim, ctx, &parms.coords, obj, source, false),
        C::FollowExitProductionPath => {
            private_follow_path(sim, ctx, &parms.coords, obj, source, true)
        }
        C::FollowPathAppend => private_follow_path_append(sim, ctx, &pos, source),
        C::AttackObject | C::ForceAttackObject => {
            let Some(victim) = obj else {
                return;
            };
            let state = if parms.cmd == C::AttackObject {
                state_id::ATTACK_OBJECT
            } else {
                state_id::FORCE_ATTACK_OBJECT
            };
            machine::clear(sim, ctx);
            set_goal_object(sim, ctx, Some(victim));
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state);
            set_max_shot_count(sim, owner, parms.int_value);
        }
        C::GuardRetaliate => {
            let Some(victim) = obj else {
                return;
            };
            machine::clear(sim, ctx);
            set_goal_object(sim, ctx, Some(victim));
            set_goal_position_clipped(sim, ctx, &pos, source);
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state_id::GUARD_RETALIATE);
            set_max_shot_count(sim, owner, parms.int_value);
        }
        C::AttackTeam => {
            machine::clear(sim, ctx);
            // `Squad::squadFromTeam` is not ported.
            ai(sim, ctx).machine.has_goal_squad = TRUE;
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state_id::ATTACK_SQUAD);
            set_max_shot_count(sim, owner, parms.int_value);
        }
        C::AttackPosition => private_attack_position(sim, ctx, parms),
        C::AttackMoveToPosition => {
            if ai(sim, ctx).is_ai_dead != FALSE || !is_mobile(sim, owner) {
                return;
            }
            machine::clear(sim, ctx);
            set_goal_position_clipped(sim, ctx, &pos, source);
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state_id::ATTACK_MOVE_TO);
            set_max_shot_count(sim, owner, parms.int_value);
        }
        C::AttackFollowWaypointPath | C::AttackFollowWaypointPathAsTeam => {
            if ai(sim, ctx).is_ai_dead != FALSE || !is_mobile(sim, owner) {
                return;
            }
            let state = if parms.cmd == C::AttackFollowWaypointPathAsTeam {
                state_id::ATTACKFOLLOW_WAYPOINT_PATH_AS_TEAM
            } else {
                state_id::ATTACKFOLLOW_WAYPOINT_PATH_AS_INDIVIDUALS
            };
            machine::clear(sim, ctx);
            ai(sim, ctx).machine.goal_waypoint = parms.waypoint.clone();
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state);
            set_max_shot_count(sim, owner, parms.int_value);
        }
        C::Hunt => {
            if !is_mobile(sim, owner) || is_projectile(sim, owner) {
                return;
            }
            plain_order(sim, ctx, source, state_id::HUNT);
        }
        C::AttackArea => {
            if !is_mobile(sim, owner) || is_projectile(sim, owner) {
                return;
            }
            ai(sim, ctx).area_to_guard = parms
                .area
                .as_ref()
                .map_or_else(String::new, |a| a.0.clone());
            plain_order(sim, ctx, source, state_id::ATTACK_AREA);
        }
        // The base class does nothing; dozers and workers take these over (S16).
        C::Repair | C::ResumeConstruction | C::ExecuteRailedTransport => {}
        C::GetHealed | C::GetRepaired => {
            let Some(depot) = obj else {
                return;
            };
            let (query, next) = if parms.cmd == C::GetHealed {
                (AiActionQuery::GetHealedAt, C::Enter)
            } else {
                (AiActionQuery::GetRepairedAt, C::Dock)
            };
            if ai_seams(sim).can_act_on(query, ctx.owner_id, depot, source) == FALSE {
                return;
            }
            let mut follow = AiCommandParms::new(next, source);
            follow.obj = Some(depot);
            execute_command(sim, ctx, &follow);
        }
        C::Enter => {
            if !is_mobile(sim, owner) {
                return;
            }
            let Some(target) = obj else {
                return;
            };
            if ai_seams(sim).can_act_on(AiActionQuery::Enter, ctx.owner_id, target, source) != FALSE
            {
                begin_goal_order(sim, ctx, Some(target), source, state_id::ENTER, false);
            }
        }
        C::Dock => {
            if !is_mobile(sim, owner) {
                return;
            }
            begin_goal_order(sim, ctx, obj, source, state_id::DOCK, false);
        }
        C::Exit => private_exit(sim, ctx, obj, source, state_id::EXIT),
        C::ExitInstantly => private_exit(sim, ctx, obj, source, state_id::EXIT_INSTANTLY),
        C::Evacuate | C::EvacuateInstantly => {
            if is_disabled(sim, owner, DisabledType::Subdued) {
                return;
            }
            if parms.int_value != 0 {
                sim.contain_mark_all_detected(owner);
            }
            let instantly = Bool::from(parms.cmd == C::EvacuateInstantly);
            sim.contain_order_all_to_exit(owner, source, instantly);
        }
        C::GuardPosition => {
            clear_guard_retaliation(sim, ctx);
            if !is_mobile(sim, owner) || is_projectile(sim, owner) {
                return;
            }
            push_guard_target(sim, ctx, GUARDTARGET_LOCATION);
            let mut adjusted = pos;
            if source == CommandSourceType::FromPlayer {
                let region = sim.terrain_extent();
                // `Region3D::containsWithin` of the 2D point.
                if !(region.lo.x < adjusted.x
                    && adjusted.x < region.hi.x
                    && region.lo.y < adjusted.y
                    && adjusted.y < region.hi.y)
                {
                    adjusted = ai_seams(sim).closest_edge_point(&adjusted);
                }
            }
            let state = ai(sim, ctx);
            state.location_to_guard = adjusted;
            state.guard_mode = parms.int_value;
            plain_order(sim, ctx, source, state_id::GUARD);
        }
        C::GuardTunnelNetwork => {
            clear_guard_retaliation(sim, ctx);
            if !is_mobile(sim, owner) || is_projectile(sim, owner) {
                return;
            }
            ai(sim, ctx).guard_mode = parms.int_value;
            plain_order(sim, ctx, source, state_id::GUARD_TUNNEL_NETWORK);
        }
        C::GuardObject => {
            clear_guard_retaliation(sim, ctx);
            let Some(target) = obj else {
                return;
            };
            if !is_mobile(sim, owner) || is_projectile(sim, owner) {
                return;
            }
            push_guard_target(sim, ctx, GUARDTARGET_OBJECT);
            let state = ai(sim, ctx);
            state.guard_mode = parms.int_value;
            state.object_to_guard = target;
            plain_order(sim, ctx, source, state_id::GUARD);
        }
        C::GuardArea => {
            clear_guard_retaliation(sim, ctx);
            if !is_mobile(sim, owner) || is_projectile(sim, owner) {
                return;
            }
            let Some((name, center)) = parms.area.clone() else {
                return;
            };
            push_guard_target(sim, ctx, GUARDTARGET_AREA);
            let state = ai(sim, ctx);
            state.area_to_guard = name;
            state.guard_mode = parms.int_value;
            state.location_to_guard = center;
            state.object_to_guard = ObjectId::INVALID;
            plain_order(sim, ctx, source, state_id::GUARD);
        }
        // `HackInternetAIUpdate::hackInternet` follows (S15).
        C::HackInternet => {
            machine::clear(sim, ctx);
            set_command_source(sim, ctx, source);
        }
        C::FaceObject => {
            if !is_mobile(sim, owner) {
                return;
            }
            begin_goal_order(sim, ctx, obj, source, state_id::FACE_OBJECT, true);
        }
        C::FacePosition => {
            if !is_mobile(sim, owner) {
                return;
            }
            move_order_unchecked(sim, ctx, &pos, source, state_id::FACE_POSITION);
        }
        C::RappelInto => {
            machine::clear(sim, ctx);
            set_goal_object(sim, ctx, obj);
            set_goal_position_clipped(sim, ctx, &pos, source);
            ai(sim, ctx).clear_blocked();
            set_command_source(sim, ctx, source);
            machine::set_state(sim, ctx, state_id::RAPPEL_INTO);
        }
        C::CombatDrop => {
            if sim.object_has_contain(owner) != FALSE {
                sim.contain_remove_all(owner, FALSE);
            }
        }
        C::Wander | C::Panic => {
            if !is_mobile(sim, owner) {
                return;
            }
            let state = if parms.cmd == C::Wander {
                state_id::WANDER
            } else {
                state_id::PANIC
            };
            machine::clear(sim, ctx);
            set_command_source(sim, ctx, source);
            ai(sim, ctx).machine.goal_waypoint = parms.waypoint.clone();
            machine::set_state(sim, ctx, state);
        }
        C::WanderInPlace => {
            if !is_mobile(sim, owner) {
                return;
            }
            plain_order(sim, ctx, source, state_id::WANDER_IN_PLACE);
        }
        C::Busy => plain_order(sim, ctx, source, state_id::BUSY),
        C::MoveAwayFromUnit => private_move_away_from_unit(sim, ctx, obj),
    }
}

/// `clear`, the command source, the state.
fn plain_order(sim: &mut dyn SimApi, ctx: ModuleContext, source: CommandSourceType, state: u32) {
    machine::clear(sim, ctx);
    set_command_source(sim, ctx, source);
    machine::set_state(sim, ctx, state);
}

/// An order on a goal object.
fn begin_goal_order(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    obj: Option<ObjectId>,
    source: CommandSourceType,
    state: u32,
    unblock: bool,
) {
    machine::clear(sim, ctx);
    set_goal_object(sim, ctx, obj);
    if unblock {
        ai(sim, ctx).clear_blocked();
    }
    set_command_source(sim, ctx, source);
    machine::set_state(sim, ctx, state);
}

fn move_order(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
    source: CommandSourceType,
    state: u32,
) {
    if is_mobile(sim, ctx.owner_slot) {
        move_order_unchecked(sim, ctx, pos, source, state);
    }
}

fn move_order_unchecked(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
    source: CommandSourceType,
    state: u32,
) {
    machine::clear(sim, ctx);
    set_goal_position_clipped(sim, ctx, pos, source);
    ai(sim, ctx).clear_blocked();
    set_command_source(sim, ctx, source);
    machine::set_state(sim, ctx, state);
}

fn waypoint_order(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms, state: u32) {
    if !is_mobile(sim, ctx.owner_slot) {
        return;
    }
    machine::clear(sim, ctx);
    ai(sim, ctx).machine.goal_waypoint = parms.waypoint.clone();
    set_command_source(sim, ctx, parms.source);
    machine::set_state(sim, ctx, state);
}

/// `privateMoveToPosition`.
fn private_move_to_position(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
    source: CommandSourceType,
) {
    if !is_mobile(sim, ctx.owner_slot) {
        return;
    }
    if ai(sim, ctx).is_idle() == FALSE && source == CommandSourceType::FromAi {
        set_goal_position_clipped(sim, ctx, pos, source);
        ai(sim, ctx).clear_blocked();
        machine::set_temporary_state(
            sim,
            ctx,
            state_id::MOVE_TO,
            LOGICFRAMES_PER_SECOND as Int * 20,
        );
    } else {
        move_order_unchecked(sim, ctx, pos, source, state_id::MOVE_TO);
    }
}

/// `privateIdle`.
fn private_idle(sim: &mut dyn SimApi, ctx: ModuleContext, source: CommandSourceType) {
    if is_projectile(sim, ctx.owner_slot) {
        return;
    }
    machine::clear(sim, ctx);
    machine::set_state(sim, ctx, state_id::IDLE);
    set_command_source(sim, ctx, source);
    for id in sim.contain_items(ctx.owner_slot) {
        if let Some(rider) = sim.find_object(id) {
            sim.ai_idle(rider, source);
        }
    }
}

/// `privateFollowPath`.
fn private_follow_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    path: &[Coord3D],
    ignore: Option<ObjectId>,
    source: CommandSourceType,
    exit_production: bool,
) {
    if !is_mobile(sim, ctx.owner_slot) {
        return;
    }
    machine::clear(sim, ctx);
    if let Some(goal) = path.last() {
        ai(sim, ctx).machine.set_goal_position(Some(goal));
    }
    ai(sim, ctx).machine.goal_path = path.to_vec();
    set_command_source(sim, ctx, source);
    ai(sim, ctx).ignore_obstacle_id = ignore.unwrap_or(ObjectId::INVALID);
    let state = if exit_production {
        state_id::FOLLOW_EXITPRODUCTION_PATH
    } else {
        state_id::FOLLOW_PATH
    };
    machine::set_state(sim, ctx, state);
}

/// `privateFollowPathAppend`.
fn private_follow_path_append(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    pos: &Coord3D,
    source: CommandSourceType,
) {
    let state = ai(sim, ctx);
    let moving = state.is_moving() != FALSE || state.waiting_for_path != FALSE;
    if state.machine.current_state_id() == state_id::FOLLOW_PATH
        && !state.machine.goal_path.is_empty()
        && moving
    {
        state.machine.add_to_goal_path(pos);
        return;
    }
    let path = if moving {
        vec![state.machine.goal_position, *pos]
    } else {
        vec![*pos]
    };
    private_follow_path(sim, ctx, &path, None, source, false);
}

/// `isWithinAttackRange(pos)` of the current weapon.
fn current_weapon_reaches(sim: &dyn SimApi, owner: ObjectSlot, pos: &Coord3D) -> bool {
    sim.object(owner).is_some_and(|o| {
        let slot = o.weapon_set.get_cur_weapon_slot();
        o.weapon_set.get_weapon_in_weapon_slot(slot).is_some_and(|w| {
            weapon_queries::is_within_attack_range_of_position(w, sim, owner, pos) != FALSE
        })
    })
}

/// `privateAttackPosition`.
fn private_attack_position(sim: &mut dyn SimApi, ctx: ModuleContext, parms: &AiCommandParms) {
    let owner = ctx.owner_slot;
    let source = parms.source;
    let mut pos = parms.pos;
    let mut max_shots = parms.int_value;
    let weapon = |sim: &dyn SimApi| {
        let o = sim.object(owner)?;
        let slot = o.weapon_set.get_cur_weapon_slot();
        o.weapon_set.get_weapon_in_weapon_slot(slot).map(|w| {
            (
                w.template.continue_attack_range,
                w.template.is_contact_weapon(),
            )
        })
    };
    let continue_range = weapon(sim).map_or(0.0, |w| w.0);
    if continue_range > 0.0 {
        sim.set_status(owner, ObjectStatusType::IgnoringStealth);
        let victim = crate::ai::ai_states::attack::closest_possible_victim(
            sim,
            owner,
            &pos,
            continue_range,
            source,
            None,
        );
        sim.clear_status(owner, ObjectStatusType::IgnoringStealth);
        let victim = victim.and_then(|slot| sim.object(slot).map(|o| o.id));
        if let Some(victim) = victim {
            let mut attack = AiCommandParms::new(AiCommandType::AttackObject, source);
            attack.obj = Some(victim);
            attack.int_value = max_shots;
            execute_command(sim, ctx, &attack);
            return;
        }
        max_shots = 1;
    }
    // A contact weapon out of range of a spot it has no path to goes somewhere
    // nearby instead. `retail-crc` does not ask about the range.
    if weapon(sim).is_some_and(|w| w.1 != FALSE)
        && (cfg!(feature = "retail-crc") || !current_weapon_reaches(sim, owner, &pos))
        && !path_follow::quick_path_exists(sim, ctx, &pos)
    {
        let options = FindPositionOptions {
            max_radius: 100.0,
            source_to_path_to_dest: Some(owner),
            ..FindPositionOptions::default()
        };
        if let Some(found) = sim.find_position_around(&pos, &options) {
            pos = found;
        }
    }
    machine::clear(sim, ctx);
    ai(sim, ctx).destroy_path();
    set_goal_position_clipped(sim, ctx, &pos, source);
    set_command_source(sim, ctx, source);
    machine::set_state(sim, ctx, state_id::ATTACK_POSITION);
    set_goal_object(sim, ctx, None);
    set_max_shot_count(sim, owner, max_shots);
}

/// `privateExit` and `privateExitInstantly`.
fn private_exit(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    container: Option<ObjectId>,
    source: CommandSourceType,
    state: u32,
) {
    let own = sim.object(ctx.owner_slot).and_then(|o| o.contained_by);
    let target = container.or_else(|| sim.object(own?).map(|o| o.id));
    let Some(slot) = target.and_then(|id| sim.find_object(id)) else {
        return;
    };
    // A named container must be the one holding the unit.
    if cfg!(not(feature = "retail-crc")) && container.is_some() && own != Some(slot) {
        return;
    }
    if is_disabled(sim, slot, DisabledType::Subdued) {
        return;
    }
    machine::clear(sim, ctx);
    set_goal_object(sim, ctx, target);
    set_command_source(sim, ctx, source);
    machine::set_state(sim, ctx, state);
}

/// A guard order ends a retaliation first.
fn clear_guard_retaliation(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if ai(sim, ctx).machine.current_state_id() == state_id::GUARD_RETALIATE {
        machine::clear(sim, ctx);
    }
}

fn push_guard_target(sim: &mut dyn SimApi, ctx: ModuleContext, kind: Int) {
    let types = &mut ai(sim, ctx).guard_target_type;
    if types[1] == GUARDTARGET_NONE {
        types[1] = kind;
    } else {
        types[0] = kind;
    }
}

/// `joinTeam`.
fn join_team(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    if ai(sim, ctx).is_ai_dead != FALSE || !is_mobile(sim, owner) {
        return;
    }
    choose_locomotor_set(sim, ctx, LocomotorSetType::Normal);
    machine::clear(sim, ctx);
    ai(sim, ctx).machine.goal_waypoint = None;
    let Some(team) = sim.object(owner).map(|o| o.team_id) else {
        return;
    };
    let other = sim.team_members_of(team).into_iter().find(|&member| {
        member != owner
            && sim.object(member).is_some_and(|o| o.ai_slot.is_some())
            && !is_disabled(sim, member, DisabledType::Held)
    });
    let Some(other) = other else {
        return;
    };
    if sim.ai_is_idle(other) != FALSE {
        let mut parms =
            AiCommandParms::new(AiCommandType::MoveToPosition, CommandSourceType::FromAi);
        parms.pos = position_of(sim, other);
        execute_command(sim, ctx, &parms);
        return;
    }
    let (goal_object, goal_position) = sim.ai_goal_of(other);
    if goal_object.is_some() {
        set_goal_object(sim, ctx, goal_object);
    } else {
        ai(sim, ctx).machine.set_goal_position(Some(&goal_position));
    }
    // The state read here is this unit's own, just cleared.
    let state = ai(sim, ctx).machine.current_state_id();
    set_command_source(sim, ctx, CommandSourceType::FromAi);
    machine::set_state(sim, ctx, state);
}

/// `onObjectCreated`: the machine, whose three attack-move sub machines enter
/// their idle state as they are built, then its own default state.
fn on_object_created(sim: &mut dyn SimApi, ctx: ModuleContext) {
    if ai(sim, ctx).machine_made != FALSE {
        return;
    }
    ai(sim, ctx).machine_made = TRUE;
    for sub in 0..3 {
        reset_next_mood_check_time(sim, ctx);
        let offset = ai_states::idle_on_enter(sim);
        let idle = &mut ai(sim, ctx).machine.attack_move_idle[sub];
        idle.inited = TRUE;
        idle.initial_sleep_offset = offset;
    }
    ai(sim, ctx).machine.moves.init_attack_move_machine();
    machine::init_default_state(sim, ctx);
    if let Some(o) = sim.object_mut(ctx.owner_slot) {
        o.ai_state_machine_ready = TRUE;
    }
}

/// `loadPostProcess`, the part that does not need the loaded locomotors.
fn load_post_process(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    if ai(sim, ctx).is_moving() == FALSE {
        ai(sim, ctx).pathfind_goal_cell = ICoord2D { x: -1, y: -1 };
        let (pos, layer) = (position_of(sim, owner), layer_of(sim, owner));
        update_goal(sim, ctx, &pos, layer);
        ai(sim, ctx).pathfind_cur_cell = ICoord2D { x: -1, y: -1 };
        ai_seams(sim).update_pos(ctx.owner_id, &pos);
    } else {
        let cell = ai(sim, ctx).pathfind_goal_cell;
        if cell.x >= 0 && cell.y >= 0 {
            // The reference never sets `z` here (it passes stack contents); only
            // the cell of the point is used.
            let goal = Coord3D {
                x: cell.x as Real * PATHFIND_CELL_SIZE_F + PATHFIND_CELL_SIZE_F * 0.5,
                y: cell.y as Real * PATHFIND_CELL_SIZE_F + PATHFIND_CELL_SIZE_F * 0.5,
                z: 0.0,
            };
            ai(sim, ctx).pathfind_goal_cell = ICoord2D { x: -1, y: -1 };
            let layer = layer_of(sim, owner);
            update_goal(sim, ctx, &goal, layer);
        }
    }
    let state = ai(sim, ctx);
    if state.waiting_for_path != FALSE && state.queue_for_path_frame == 0 {
        let _ = sim.queue_for_path(ctx.owner_id);
    }
}

/// The command entry.
pub fn execute(sim: &mut dyn SimApi, ctx: ModuleContext, call: &AiCall) {
    match call {
        AiCall::Command(parms) => execute_command(sim, ctx, parms),
        AiCall::ObjectCreated => on_object_created(sim, ctx),
        AiCall::LoadPostProcess => load_post_process(sim, ctx),
        AiCall::MarkAsDead => {
            ai(sim, ctx).is_ai_dead = TRUE;
            if let Some(o) = sim.object_mut(ctx.owner_slot) {
                o.set_effectively_dead(TRUE);
            }
            wake_up_now(sim, ctx);
        }
        AiCall::ChooseLocomotorSet(set) => {
            choose_locomotor_set(sim, ctx, *set);
        }
        AiCall::SetLocomotorUpgrade(upgraded) => set_locomotor_upgrade(sim, ctx, *upgraded),
        AiCall::SetLocomotorGoalNone => set_locomotor_goal_none(sim, ctx),
        AiCall::SetGoalObject(id) => friend_set_goal_object(sim, ctx, *id),
        AiCall::SetTurretEnabled(turret, enabled) => {
            if let Ok(turret) = usize::try_from(turret.as_i32()) {
                turrets::set_enabled(sim, ctx, turret, *enabled);
            }
        }
        AiCall::RecenterTurret(turret) => {
            if let Ok(turret) = usize::try_from(turret.as_i32()) {
                turrets::recenter(sim, ctx, turret);
            }
        }
        AiCall::WakeUpAndAttemptToTarget => {
            if ai(sim, ctx).is_idle() != FALSE {
                let now = sim.frame();
                let state = ai(sim, ctx);
                state.next_mood_check_time = now;
                state.randomly_offset_mood_check = TRUE;
            }
        }
        AiCall::JoinTeam => join_team(sim, ctx),
        AiCall::CancelAllDozerTasks => {
            if !matches!(self::ai(sim, ctx).derived, DerivedAi::Base) {
                dozer_ai_update::cancel_all_tasks(sim, ctx);
            }
        }
        AiCall::DisabledEdge(now_disabled) => {
            if !matches!(ai(sim, ctx).derived, DerivedAi::Base) {
                dozer_ai_update::on_disabled_edge(sim, ctx, *now_disabled);
            }
        }
        AiCall::Delete => {
            if !matches!(ai(sim, ctx).derived, DerivedAi::Base) {
                dozer_ai_update::on_delete(sim, ctx);
            }
        }
        AiCall::QuickExit(path) => {
            let locked = ai(sim, ctx).machine.locked;
            ai(sim, ctx).machine.unlock();
            ai(sim, ctx).machine.goal_path = path.clone();
            machine::set_temporary_state(
                sim,
                ctx,
                state_id::FOLLOW_EXITPRODUCTION_PATH,
                LOGICFRAMES_PER_SECOND as Int * 10,
            );
            if locked != FALSE {
                ai(sim, ctx).machine.lock();
            }
        }
    }
}

// -----------------------------------------------------------------------------
// Module plumbing
// -----------------------------------------------------------------------------

impl BehaviorModule for AiUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn resident(&self) -> Option<ResidentEntries> {
        let mut held = DisabledMaskType::new();
        held.set(DisabledType::Held.as_i32(), 1);
        Some(ResidentEntries {
            disabled_types_to_process: held,
            update: match self.derived {
                DerivedAi::Base => update,
                DerivedAi::Dozer(_) => dozer_ai_update::update,
                DerivedAi::Worker(_) => worker_ai_update::update,
            },
            do_pathfind,
        })
    }

    fn ai_call_entry(&self) -> Option<AiCallEntry> {
        Some(execute)
    }

    fn ai_collision_entry(&self) -> Option<AiCollisionEntry> {
        Some(process_collision)
    }

    /// The constructor: the turrets, the normal locomotor set, then the next frame.
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        turrets::construct(self, sim, ctx);
        choose_locomotor_set_on(self, sim, ctx, LocomotorSetType::Normal);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        self.construct_derived(sim, ctx);
    }

    /// The destructor halts the machine: no state exits.
    fn on_destruct(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.machine.halt();
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let owner = self
            .schedule
            .mod_slot
            .and_then(|slot| sim.module_owner_id(slot))
            .unwrap_or(ObjectId::INVALID);
        let version = if cfg!(feature = "retail-xfer-save") {
            4
        } else {
            5
        };
        self.xfer_as_version(xfer, sim, owner, version)
    }

    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }

    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

/// The queries, and the setters that only touch the module. What acts goes
/// through [`execute`].
impl AiUpdateFacet for AiUpdate {
    fn which_turret_for_cur_weapon(&self, sim: &dyn SimView, ctx: ModuleContext) -> WhichTurretType {
        turrets::which_for_cur_weapon(sim, self, ctx.owner_slot)
            .map_or(WhichTurretType::Invalid, |t| WhichTurretType::from_i32(t as Int))
    }

    fn is_weapon_slot_enabled(&self, slot: WeaponSlotType) -> Bool {
        turrets::which_for_weapon_slot(self, slot)
            .and_then(|t| self.turrets[t].as_deref())
            .map_or(TRUE, |t| t.is_turret_enabled())
    }

    fn is_turret_in_natural_position(&self, turret: WhichTurretType) -> Bool {
        self.turret_natural_position(turret, false)
    }

    fn turret_natural_position(&self, turret: WhichTurretType, under_construction: bool) -> Bool {
        usize::try_from(turret.as_i32()).map_or(FALSE, |t| {
            turrets::is_in_natural_position(self, t, under_construction)
        })
    }

    fn is_moving(&self) -> Bool {
        AiUpdate::is_moving(self)
    }

    fn is_attacking(&self) -> Bool {
        Bool::from(self.is_attacking_state())
    }

    fn current_victim_id(&self) -> Option<ObjectId> {
        Some(self.current_victim_id)
    }

    fn is_idle(&self) -> Bool {
        AiUpdate::is_idle(self)
    }

    fn is_ai_in_dead_state(&self) -> Bool {
        self.is_ai_dead
    }

    fn get_ignored_obstacle_id(&self) -> ObjectId {
        self.ignore_obstacle_id
    }

    fn ignore_obstacle(&mut self, obstacle: ObjectId) {
        self.ignore_obstacle_id = obstacle;
    }

    fn destroy_path(&mut self) {
        AiUpdate::destroy_path(self);
    }

    fn set_attitude(&mut self, attitude: AttitudeType) {
        self.attitude = attitude;
    }

    fn has_cur_locomotor(&self) -> Bool {
        Bool::from(self.cur_locomotor.is_some())
    }

    fn cur_locomotor(&self) -> Option<&Locomotor> {
        AiUpdate::cur_locomotor(self)
    }

    fn goal_of(&self) -> (Option<ObjectId>, Coord3D) {
        let id = self.machine.goal_object_id;
        (
            (id != ObjectId::INVALID).then_some(id),
            self.machine.goal_position,
        )
    }

    fn set_locomotor_goal_none(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.locomotor_goal_type = LocoGoalType::None;
    }

    fn set_locomotor_max_lift(&mut self, max_lift: Real) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_max_lift(max_lift);
        }
    }

    fn set_locomotor_max_turn_rate(&mut self, max_turn_rate: Real) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_max_turn_rate(max_turn_rate);
        }
    }

    fn set_locomotor_max_braking(&mut self, max_braking: Real) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_max_braking(max_braking);
        }
    }

    fn set_locomotor_ultra_accurate(&mut self, ultra: Bool) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_ultra_accurate(ultra);
        }
    }

    fn set_locomotor_close_enough_dist(&mut self, dist: Real) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_close_enough_dist(dist);
        }
    }

    fn set_locomotor_close_enough_dist_3d(&mut self, is_3d: Bool) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_close_enough_dist3d(is_3d);
        }
    }

    fn set_locomotor_use_precise_z_pos(&mut self, precise: Bool) {
        if let Some(l) = self.cur_locomotor_mut() {
            l.set_use_precise_z_pos(precise);
        }
    }
}

pub fn create_ai_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<AiUpdateModuleData>()
        .expect("AIUpdateInterface data type mismatch")
        .clone();
    Box::new(AiUpdate::new(Arc::new(typed)))
}
