//! What the AI asks of code that is not ported yet. A test answers these from a
//! tape of what the reference did; later slices implement them for real.
//!
//! [`PathServices`] and [`AiSeams`] have callers. [`PathUnitQueries`] and
//! [`MapTerrain`] fix the questions the pathfinder and the AI ask, taken from the
//! calls in the reference; nothing implements or checks them yet.

use crate::combat::locomotor::LocomotorSeams;
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::AiCommandParms;
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::Path;
use crate::rules::combat_types::LocomotorSurfaceTypeMask;
use crate::sim::SimView;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use zh_common::core::state_machine::{StateExitType, StateId, StateReturnType};
use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, UnsignedInt, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// `TheAI->pathfinder()` as far as the frame step and the AI orders use it. The
/// searches join this trait with the slices that port them.
pub trait PathServices: Send {
    /// `Pathfinder::queueForPath`. `FALSE` when the request was not taken.
    fn queue_for_path(&mut self, id: ObjectId) -> Bool;
    /// The start of `processPathfindQueue`, once per logic frame.
    fn begin_queue(&mut self);
    /// The next request to serve this frame. `None` once the queue is empty or
    /// the frame's search budget is used up. A search may queue again, so the
    /// frame step asks anew after every request.
    fn next_request(&mut self) -> Option<ObjectId>;
}

/// No pathfinder: every request is refused.
pub struct NoPathServices;

impl PathServices for NoPathServices {
    fn queue_for_path(&mut self, _id: ObjectId) -> Bool {
        FALSE
    }

    fn begin_queue(&mut self) {}

    fn next_request(&mut self) -> Option<ObjectId> {
        None
    }
}

/// The questions only a unit's AI can answer that `AIPathfind.cpp` asks. Position,
/// geometry, kind, relationship and crusher level come from `SimView`.
pub trait PathUnitQueries {
    fn is_moving(&self, id: ObjectId) -> Bool;
    fn is_idle(&self, id: ObjectId) -> Bool;
    fn is_busy(&self, id: ObjectId) -> Bool;
    fn is_attacking(&self, id: ObjectId) -> Bool;
    fn is_doing_ground_movement(&self, id: ObjectId) -> Bool;
    fn is_aircraft_that_adjusts_destination(&self, id: ObjectId) -> Bool;
    fn can_path_through_units(&self, id: ObjectId) -> Bool;
    fn ignored_obstacle_id(&self, id: ObjectId) -> ObjectId;
    fn building_to_not_path_around(&self, id: ObjectId) -> ObjectId;
    /// `getPathfindGoalCell`.
    fn pathfind_goal_cell(&self, id: ObjectId) -> ICoord2D;
    /// `getCurPathfindCell`.
    fn cur_pathfind_cell(&self, id: ObjectId) -> ICoord2D;
}

/// A waypoint of the map (`Waypoint`).
#[derive(Debug, Clone, PartialEq)]
pub struct WaypointInfo {
    pub id: u32,
    pub name: String,
    pub location: Coord3D,
    pub links: Vec<u32>,
}

/// The map-wide terrain questions the AI asks, on top of the height and water
/// questions of [`ThingTerrainAccess`].
pub trait MapTerrain: ThingTerrainAccess {
    /// `TerrainLogic::getExtent`.
    fn get_extent(&self) -> Region3D;
    /// `getExtentIncludingBorder`.
    fn get_extent_including_border(&self) -> Region3D;
    /// `getMaximumPathfindExtent`.
    fn get_maximum_pathfind_extent(&self) -> Region3D;
    /// `getFirstWaypoint`, as an id.
    fn first_waypoint(&self) -> Option<u32>;
    /// `getWaypointByID`.
    fn waypoint_by_id(&self, id: u32) -> Option<WaypointInfo>;
    /// `getWaypointByName`, as an id.
    fn waypoint_by_name(&self, name: &str) -> Option<u32>;
}

/// `ActionManager` questions the AI asks before taking an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiActionQuery {
    /// `canGetHealedAt`.
    GetHealedAt,
    /// `canGetRepairedAt`.
    GetRepairedAt,
    /// `canEnterObject` without a capacity check.
    Enter,
}

/// What `AIUpdateInterface` and its states ask of code that is not ported yet.
///
/// Every method has the answer of a world without that code, so a plain
/// [`NoAiSeams`] gives units that take orders and idle. A test answers from a
/// tape of the reference's calls.
pub trait AiSeams: Send {
    /// `Pathfinder::validMovementTerrain` (S3).
    fn valid_movement_terrain(
        &mut self,
        _layer: PathfindLayerEnum,
        _surfaces: LocomotorSurfaceTypeMask,
        _pos: &Coord3D,
    ) -> Bool {
        1
    }
    /// `PartitionManager::getGroundOrStructureHeight` (S3 or S9).
    fn ground_or_structure_height(&mut self, _x: Real, _y: Real) -> Real {
        0.0
    }
    /// `Pathfinder::chooseBestLocomotorForPosition` (S3): the index in the set.
    /// Without a pathfinder the first entry is taken.
    fn choose_best_locomotor(
        &mut self,
        _owner: ObjectId,
        _layer: PathfindLayerEnum,
        _count: usize,
        _pos: &Coord3D,
    ) -> Option<usize> {
        Some(0)
    }
    /// `Pathfinder::updateGoal` (S4).
    fn update_goal(&mut self, _owner: ObjectId, _pos: &Coord3D, _layer: PathfindLayerEnum) {}
    /// `Pathfinder::updatePos` (S4).
    fn update_pos(&mut self, _owner: ObjectId, _pos: &Coord3D) {}
    /// `Pathfinder::goalPosition` (S4).
    fn goal_position(&mut self, _owner: ObjectId) -> Option<Coord3D> {
        None
    }
    /// `Pathfinder::snapPosition` (S4).
    fn snap_position(&mut self, _owner: ObjectId, pos: Coord3D) -> Coord3D {
        pos
    }
    /// `Pathfinder::clientSafeQuickDoesPathExist` for the unit's locomotors (S5).
    fn quick_path_exists(&mut self, _owner: ObjectId, _to: &Coord3D) -> Bool {
        1
    }
    /// The searches of `AIUpdateInterface::doPathfind` and `computeQuickPath`
    /// (S5, S6, S9). Answers whether the unit has a path afterwards.
    fn compute_path(&mut self, _owner: ObjectId) -> Bool {
        FALSE
    }
    /// `Path::computePointOnPath` or `computeFlightDistToGoal` (S3, S9): the
    /// point to steer for and the distance along the path.
    /// Whether the seams answer the four searches below instead of the map.
    /// For harnesses whose reference replaced them too.
    fn replaces_searches(&mut self) -> Bool {
        FALSE
    }
    /// Whether [`AiSeams::attack_path`] answers `computeAttackPath` instead of the
    /// map. For harnesses whose reference replaced it.
    fn replaces_attack_path(&mut self) -> Bool {
        FALSE
    }
    /// `computeAttackPath`: `None` for FALSE, else the path it left.
    fn attack_path(
        &mut self,
        _owner: ObjectId,
        _victim: Option<ObjectId>,
        _victim_pos: &Coord3D,
    ) -> Option<Option<Path>> {
        None
    }
    /// `Pathfinder::findClosestPath`, which may move `to` (S6).
    fn closest_path(
        &mut self,
        _owner: ObjectId,
        _from: &Coord3D,
        _to: &mut Coord3D,
        _blocked: Bool,
        _path_cost_factor: Real,
    ) -> Option<Path> {
        None
    }
    /// `Pathfinder::patchPath` (S6).
    fn patch_path(
        &mut self,
        _owner: ObjectId,
        _current: Option<&Path>,
        _blocked: Bool,
    ) -> Option<Path> {
        None
    }
    /// `Pathfinder::findSafePath` (S6).
    fn safe_path(
        &mut self,
        _owner: ObjectId,
        _from: &Coord3D,
        _repulsor1: &Coord3D,
        _repulsor2: &Coord3D,
        _radius: Real,
    ) -> Option<Path> {
        None
    }
    /// `Pathfinder::getMoveAwayFromPath` (S6): off `unit_path`, and off the path
    /// of the unit moved away from before.
    fn move_away_path(
        &mut self,
        _owner: ObjectId,
        _unit: ObjectId,
        _unit_path: &Path,
        _second: Option<ObjectId>,
        _second_path: Option<&Path>,
    ) -> Option<Path> {
        None
    }
    fn point_on_path(&mut self, _owner: ObjectId, pos: &Coord3D) -> (Coord3D, Real) {
        (*pos, 0.0)
    }
    /// `Path::xfer` (S3).
    fn path_xfer(&mut self, _owner: ObjectId, _xfer: &mut dyn Xfer) -> Result<(), XferError> {
        Err(XferError::InvalidData)
    }

    /// `onEnter` of a state that is not ported. Without it the unit stays in the
    /// state and does nothing.
    /// Whether the seams answer a state that is ported (`ai_states::movement`).
    /// For harnesses whose reference replaced the state too.
    fn replaces_state(&mut self, _owner: ObjectId, _state: StateId) -> Bool {
        FALSE
    }
    fn state_enter(&mut self, _owner: ObjectId, _state: StateId) -> StateReturnType {
        StateReturnType::Continue
    }
    /// `update` of a state that is not ported.
    fn state_update(&mut self, _owner: ObjectId, _state: StateId) -> StateReturnType {
        StateReturnType::Continue
    }
    /// `onExit` of a state that is not ported.
    fn state_exit(&mut self, _owner: ObjectId, _state: StateId, _status: StateExitType) {}
    /// `xfer` of a state that is not ported. Saving such a unit fails.
    fn state_xfer(
        &mut self,
        _owner: ObjectId,
        _state: StateId,
        _xfer: &mut dyn Xfer,
    ) -> Result<(), XferError> {
        Err(XferError::InvalidData)
    }


    /// Replaces `AIUpdateInterface::getNextMoodTarget`
    /// (`ai_update::next_mood_target`) with a recorded answer; `None` leaves the
    /// question to the sim.
    fn next_mood_target(
        &mut self,
        _owner: ObjectId,
        _called_by_ai: Bool,
        _during_idle: Bool,
    ) -> Option<Option<ObjectId>> {
        None
    }
    /// `AI::findClosestRepulsor` (S12).
    fn closest_repulsor(&mut self, _owner: ObjectId, _range: Real) -> Option<ObjectId> {
        None
    }
    /// `ActionManager` (S19).
    fn can_act_on(
        &mut self,
        _query: AiActionQuery,
        _owner: ObjectId,
        _target: ObjectId,
        _source: CommandSourceType,
    ) -> Bool {
        FALSE
    }
    /// `TerrainLogic::findClosestEdgePoint` (S1).
    fn closest_edge_point(&mut self, pos: &Coord3D) -> Coord3D {
        *pos
    }

    // What `AIGroup` asks (S19).
    /// A recorder of group orders: `TRUE` takes the order, so the member's AI
    /// does not get it.
    fn group_order(&mut self, _owner: ObjectId, _parms: &AiCommandParms) -> Bool {
        FALSE
    }
    /// `Pathfinder::removeGoal` (S4).
    fn remove_goal(&mut self, _owner: ObjectId) {}
    /// `Pathfinder::adjustDestination` (S4).
    fn adjust_destination(
        &mut self,
        _owner: ObjectId,
        _dest: &mut Coord3D,
        _group_dest: Option<&Coord3D>,
    ) {
    }
    /// `Pathfinder::isLinePassable` (S3).
    fn is_line_passable(
        &mut self,
        _owner: ObjectId,
        _surfaces: LocomotorSurfaceTypeMask,
        _layer: PathfindLayerEnum,
        _from: &Coord3D,
        _to: &Coord3D,
    ) -> Bool {
        1
    }
    /// `Pathfinder::findGroundPath` (S5): the optimized nodes of the path.
    fn find_ground_path(
        &mut self,
        _from: &Coord3D,
        _to: &Coord3D,
        _diameter: Int,
    ) -> Option<Vec<Coord3D>> {
        None
    }
    /// `TerrainLogic::getExtent`, or `getMaximumPathfindExtent`.
    fn map_extent(&mut self, extent: Region3D, _maximum_pathfind: bool) -> Region3D {
        extent
    }
    /// `ActionManager::canDoSpecialPower` and its location and object forms.
    fn can_do_special_power(
        &mut self,
        _owner: ObjectId,
        _power: UnsignedInt,
        _target: (Option<ObjectId>, Option<Coord3D>),
        _options: UnsignedInt,
    ) -> Bool {
        FALSE
    }
    /// `BuildAssistant::sellObject`.
    fn sell_object(&mut self, _owner: ObjectId) {}
    /// `findSpecialPowerWithOverridableDestinationActive` and the call on it.
    fn override_special_power_destination(
        &mut self,
        _owner: ObjectId,
        _power_type: Int,
        _location: &Coord3D,
    ) {
    }
    /// `AIUpdateInterface::getSneakyTargetingOffset` (`JetAIUpdate`, S17): where
    /// shots at `victim` go while it is dodging.
    fn sneaky_targeting_offset(&mut self, _victim: ObjectId) -> Option<Coord3D> {
        None
    }
    /// `TerrainLogic::getBridgeAttackPoints`. Without a record of the bridge the
    /// reference gives the bridge's own position twice.
    fn bridge_attack_points(&mut self, _bridge: ObjectId, pos: &Coord3D) -> [Coord3D; 2] {
        [*pos, *pos]
    }
    /// `AIUpdateInterface::isAircraftThatAdjustsDestination` (S17).
    fn is_aircraft_that_adjusts_destination(&mut self, _owner: ObjectId) -> Bool {
        0
    }
    /// `Pathfinder::adjustTargetDestination` (S4): the approach spot to use.
    fn adjust_target_destination(
        &mut self,
        _owner: ObjectId,
        _target: Option<ObjectId>,
        _target_pos: &Coord3D,
        _weapon: &str,
        approach: Coord3D,
    ) -> Coord3D {
        approach
    }
    /// `AssaultTransportAIInterface::beginAssault` (S15), asked of every owner
    /// with an AI; only an assault transport acts on it.
    fn begin_assault(&mut self, _owner: ObjectId, _victim: Option<ObjectId>) {}


    // Garrisons (S11) and teams.
    /// `ContainModuleInterface::calcBestGarrisonPosition`.
    fn garrison_fire_position(
        &mut self,
        _container: ObjectId,
        _target_pos: &Coord3D,
    ) -> Option<Coord3D> {
        None
    }
    /// `ContainModuleInterface::attemptBestFirePointPosition`: moves the rider to
    /// the fire point nearest the target and says whether it is in range.
    fn attempt_best_fire_point(
        &mut self,
        _owner: ObjectId,
        _victim: Option<ObjectId>,
        _target_pos: &Coord3D,
    ) -> Bool {
        FALSE
    }
    /// A team that attacks a common target takes `victim` if it has none.
    fn offer_team_target(&mut self, _owner: ObjectId, _victim: ObjectId) {}
    /// The team forgets `victim` if it is its common target.
    fn drop_team_target(&mut self, _owner: ObjectId, _victim: ObjectId) {}
}

/// A world without any of the code behind [`AiSeams`].
pub struct NoAiSeams;

impl AiSeams for NoAiSeams {}

/// The seams as [`crate::sim::SimContext`] holds them.
pub type SharedAiSeams = Arc<Mutex<dyn AiSeams>>; // determinism-ok: single-threaded handle to unported code, not simulation state

pub fn shared<T: AiSeams + 'static>(seams: T) -> SharedAiSeams {
    Arc::new(Mutex::new(seams))
}

fn lock(seams: &SharedAiSeams) -> MutexGuard<'_, dyn AiSeams + 'static> {
    seams.lock().unwrap_or_else(PoisonError::into_inner)
}

/// The seams of the world, locked for one question. Do not keep the result across
/// a call into the world.
pub struct SeamsGuard(SharedAiSeams);

impl SeamsGuard {
    pub fn new(seams: SharedAiSeams) -> Self {
        Self(seams)
    }
}

macro_rules! forward {
    ($($name:ident($($arg:ident: $ty:ty),*) $(-> $ret:ty)?;)*) => {
        impl SeamsGuard {
            $(pub fn $name(&mut self, $($arg: $ty),*) $(-> $ret)? {
                lock(&self.0).$name($($arg),*)
            })*
        }
    };
}

forward! {
    valid_movement_terrain(layer: PathfindLayerEnum, surfaces: LocomotorSurfaceTypeMask, pos: &Coord3D) -> Bool;
    ground_or_structure_height(x: Real, y: Real) -> Real;
    choose_best_locomotor(owner: ObjectId, layer: PathfindLayerEnum, count: usize, pos: &Coord3D) -> Option<usize>;
    update_goal(owner: ObjectId, pos: &Coord3D, layer: PathfindLayerEnum);
    update_pos(owner: ObjectId, pos: &Coord3D);
    goal_position(owner: ObjectId) -> Option<Coord3D>;
    snap_position(owner: ObjectId, pos: Coord3D) -> Coord3D;
    quick_path_exists(owner: ObjectId, to: &Coord3D) -> Bool;
    compute_path(owner: ObjectId) -> Bool;
    point_on_path(owner: ObjectId, pos: &Coord3D) -> (Coord3D, Real);
    path_xfer(owner: ObjectId, xfer: &mut dyn Xfer) -> Result<(), XferError>;
    replaces_searches() -> Bool;
    replaces_attack_path() -> Bool;
    attack_path(owner: ObjectId, victim: Option<ObjectId>, victim_pos: &Coord3D) -> Option<Option<Path>>;
    closest_path(owner: ObjectId, from: &Coord3D, to: &mut Coord3D, blocked: Bool, path_cost_factor: Real) -> Option<Path>;
    patch_path(owner: ObjectId, current: Option<&Path>, blocked: Bool) -> Option<Path>;
    safe_path(owner: ObjectId, from: &Coord3D, repulsor1: &Coord3D, repulsor2: &Coord3D, radius: Real) -> Option<Path>;
    move_away_path(owner: ObjectId, unit: ObjectId, unit_path: &Path, second: Option<ObjectId>, second_path: Option<&Path>) -> Option<Path>;
    replaces_state(owner: ObjectId, state: StateId) -> Bool;
    state_enter(owner: ObjectId, state: StateId) -> StateReturnType;
    state_update(owner: ObjectId, state: StateId) -> StateReturnType;
    state_exit(owner: ObjectId, state: StateId, status: StateExitType);
    state_xfer(owner: ObjectId, state: StateId, xfer: &mut dyn Xfer) -> Result<(), XferError>;
    next_mood_target(owner: ObjectId, called_by_ai: Bool, during_idle: Bool) -> Option<Option<ObjectId>>;
    closest_repulsor(owner: ObjectId, range: Real) -> Option<ObjectId>;
    can_act_on(query: AiActionQuery, owner: ObjectId, target: ObjectId, source: CommandSourceType) -> Bool;
    closest_edge_point(pos: &Coord3D) -> Coord3D;
    group_order(owner: ObjectId, parms: &AiCommandParms) -> Bool;
    remove_goal(owner: ObjectId);
    adjust_destination(owner: ObjectId, dest: &mut Coord3D, group_dest: Option<&Coord3D>);
    is_line_passable(owner: ObjectId, surfaces: LocomotorSurfaceTypeMask, layer: PathfindLayerEnum, from: &Coord3D, to: &Coord3D) -> Bool;
    find_ground_path(from: &Coord3D, to: &Coord3D, diameter: Int) -> Option<Vec<Coord3D>>;
    map_extent(extent: Region3D, maximum_pathfind: bool) -> Region3D;
    sneaky_targeting_offset(victim: ObjectId) -> Option<Coord3D>;
    bridge_attack_points(bridge: ObjectId, pos: &Coord3D) -> [Coord3D; 2];
    is_aircraft_that_adjusts_destination(owner: ObjectId) -> Bool;
    adjust_target_destination(owner: ObjectId, target: Option<ObjectId>, target_pos: &Coord3D, weapon: &str, approach: Coord3D) -> Coord3D;
    begin_assault(owner: ObjectId, victim: Option<ObjectId>);
    garrison_fire_position(container: ObjectId, target_pos: &Coord3D) -> Option<Coord3D>;
    attempt_best_fire_point(owner: ObjectId, victim: Option<ObjectId>, target_pos: &Coord3D) -> Bool;
    offer_team_target(owner: ObjectId, victim: ObjectId);
    drop_team_target(owner: ObjectId, victim: ObjectId);
    can_do_special_power(owner: ObjectId, power: UnsignedInt, target: (Option<ObjectId>, Option<Coord3D>), options: UnsignedInt) -> Bool;
    sell_object(owner: ObjectId);
    override_special_power_destination(owner: ObjectId, power_type: Int, location: &Coord3D);
}

impl LocomotorSeams for SeamsGuard {
    fn valid_movement_terrain(
        &mut self,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> Bool {
        lock(&self.0).valid_movement_terrain(layer, surfaces, pos)
    }

    fn ground_or_structure_height(&mut self, x: Real, y: Real) -> Real {
        lock(&self.0).ground_or_structure_height(x, y)
    }
}

pub fn ai_seams(sim: &(impl SimView + ?Sized)) -> SeamsGuard {
    SeamsGuard::new(sim.shared_ai_seams())
}
