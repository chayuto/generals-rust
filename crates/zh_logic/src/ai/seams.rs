//! What the AI asks of code that is not ported yet. A test answers these from a
//! tape of what the reference did; later slices implement them for real.
//!
//! Only [`PathServices`] has a caller today. [`PathUnitQueries`] and
//! [`MapTerrain`] fix the questions the pathfinder and the AI ask, taken from the
//! calls in the reference; nothing implements or checks them yet.

use crate::object::thing::ThingTerrainAccess;
use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{Bool, ObjectId, FALSE};

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
