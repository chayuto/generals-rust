//! The reservation half of `Pathfinder` (`PKT-411`): which cells a unit holds
//! and heads for, the searches that move a destination to a free cell, and
//! telling idle allies to get out of the way.
//!
//! Golden test: `crates/zh_logic/tests/golden_s4_reservations.rs`.

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, UnsignedShort};

use super::queries::{radius_and_center, PathAgent, PathUnits};
use super::Pathfinder;
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::Path;
use crate::pathfinder::pathfind_cell::{CellFlags, CellType};
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, MAX_ADJUSTMENT_CELL_COUNT, MOVE_ALLIES_MAX_DEPTH, PATHFIND_CELL_SIZE_F,
};
use crate::rules::combat_types::LocomotorSurfaceTypeMask;

/// What the reservation updates read from a unit, and the three values they
/// write back: the caller copies those to the object and its AI.
#[derive(Debug, Clone, PartialEq)]
pub struct GoalMover {
    /// `has_ai` false makes every update a no-op.
    pub agent: PathAgent,
    /// `KINDOF_IMMOBILE`.
    pub immobile: bool,
    /// `AIUpdateInterface::isDoingGroundMovement`.
    pub ground_movement: bool,
    /// `KINDOF_PRODUCED_AT_HELIPAD` and `DISABLED_UNMANNED`.
    pub unmanned_helicopter: bool,
    /// `Object::m_destinationLayer`. Written.
    pub destination_layer: PathfindLayerEnum,
    /// `AIUpdateInterface::m_pathfindGoalCell`. Written.
    pub goal_cell: ICoord2D,
    /// `AIUpdateInterface::m_pathfindCurCell`. Written.
    pub cur_cell: ICoord2D,
}

/// What the destination searches read from a unit.
#[derive(Debug, Clone, PartialEq)]
pub struct AdjustUnit {
    pub agent: PathAgent,
    pub position: Coord3D,
    /// `LocomotorSet::getValidSurfaces`.
    pub surfaces: LocomotorSurfaceTypeMask,
    /// `KINDOF_AIRCRAFT`.
    pub aircraft: bool,
    /// `KINDOF_PROJECTILE`.
    pub projectile: bool,
    /// False for the unit of a computer player.
    pub human: bool,
}

/// `Pathfinder::checkPathCost`, a search of its own that is not ported (S5):
/// the cost of a path from a group's destination to a candidate cell.
pub trait PathCost {
    fn check_path_cost(
        &mut self,
        pathfinder: &mut Pathfinder,
        unit: &AdjustUnit,
        from: &Coord3D,
        to: &Coord3D,
    ) -> Int;
}

/// For callers without a group destination, which never ask for a cost.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoPathCost;

impl PathCost for NoPathCost {
    fn check_path_cost(
        &mut self,
        _pathfinder: &mut Pathfinder,
        _unit: &AdjustUnit,
        _from: &Coord3D,
        _to: &Coord3D,
    ) -> Int {
        // `MAX_COST`, which rejects every candidate.
        0x7fff_0000
    }
}

/// What `moveAllies` reads from the unit that wants through.
#[derive(Debug, Clone, PartialEq)]
pub struct AllyMover {
    pub agent: PathAgent,
    /// `KINDOF_DOZER` or `KINDOF_HARVESTER`.
    pub dozer_or_harvester: bool,
}

/// What `moveAllies` reads from a unit standing on the path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AllyUnit {
    /// `mover->getRelationship(other) == ALLIES`.
    pub ally: bool,
    /// `KINDOF_INFANTRY`.
    pub infantry: bool,
    pub has_ai: bool,
    pub moving: bool,
    pub attacking: bool,
    /// `OBJECT_STATUS_IS_USING_ABILITY`.
    pub using_ability: bool,
    pub busy: bool,
}

/// The units `moveAllies` looks up and orders about.
pub trait MoveAlliesWorld {
    /// `findObjectByID`, as seen from `mover`.
    fn ally(&mut self, mover: ObjectId, other: ObjectId) -> Option<AllyUnit>;
    /// `other->getAI()->aiMoveAwayFromUnit(mover, CMD_FROM_AI)`. The order may
    /// come back into [`Pathfinder::move_allies`].
    fn move_away_from_unit(
        &mut self,
        pathfinder: &mut Pathfinder,
        other: ObjectId,
        mover: ObjectId,
    );
}

/// `IS_IMPASSABLE`.
fn is_impassable(cell_type: CellType) -> bool {
    matches!(
        cell_type,
        CellType::Impassable | CellType::Obstacle | CellType::BridgeImpassable
    )
}

/// The cell a unit with this footprint stands in at `pos`.
fn footprint_cell(pos: &Coord3D, center: bool) -> ICoord2D {
    let cell = |v: Real| {
        if center {
            real_to_int_floor(v / PATHFIND_CELL_SIZE_F)
        } else {
            real_to_int_floor(0.5 + v / PATHFIND_CELL_SIZE_F)
        }
    };
    ICoord2D::new(cell(pos.x), cell(pos.y))
}

/// The cells of a footprint around `at`, `x` outermost.
fn footprint(at: ICoord2D, radius: Int, above: Int) -> impl Iterator<Item = ICoord2D> {
    let (lo_x, hi_x) = (at.x.wrapping_sub(radius), at.x.wrapping_add(above));
    let (lo_y, hi_y) = (at.y.wrapping_sub(radius), at.y.wrapping_add(above));
    (lo_x..hi_x).flat_map(move |x| (lo_y..hi_y).map(move |y| ICoord2D::new(x, y)))
}

/// Moves a point by half a cell for a footprint that stands on cell corners.
fn bias(pos: &Coord3D, center: bool) -> Coord3D {
    let mut biased = *pos;
    if !center {
        biased.x += PATHFIND_CELL_SIZE_F / 2.0;
        biased.y += PATHFIND_CELL_SIZE_F / 2.0;
    }
    biased
}

/// The square spiral of the `adjust*` searches: right, up, left, down, with the
/// side growing every second turn. The budget is checked once per ring, so the
/// last ring is walked whole.
fn spiral<T>(start: ICoord2D, mut visit: impl FnMut(Int, Int) -> Option<T>) -> Option<T> {
    const DIRECTIONS: [(Int, Int); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];
    let mut limit = MAX_ADJUSTMENT_CELL_COUNT as Int;
    let (mut i, mut j) = (start.x, start.y);
    let mut length = 1;
    while limit > 0 {
        for (turn, (dx, dy)) in DIRECTIONS.into_iter().enumerate() {
            for _ in 0..length {
                i = i.wrapping_add(dx);
                j = j.wrapping_add(dy);
                limit -= 1;
                if let Some(found) = visit(i, j) {
                    return Some(found);
                }
            }
            if turn & 1 != 0 {
                length += 1;
            }
        }
    }
    None
}

impl Pathfinder {
    /// `PathfindCell::getGoalUnit`.
    pub fn goal_unit(&self, idx: CellIdx) -> ObjectId {
        self.cell_info(idx)
            .map_or(ObjectId::INVALID, |i| i.goal_unit_id)
    }

    /// `PathfindCell::getGoalAircraft`.
    pub fn goal_aircraft(&self, idx: CellIdx) -> ObjectId {
        self.cell_info(idx)
            .map_or(ObjectId::INVALID, |i| i.goal_aircraft_id)
    }

    /// `PathfindCell::getPosUnit`.
    pub fn unit_present(&self, idx: CellIdx) -> ObjectId {
        self.pos_unit(idx)
    }

    /// `adjustCoordToCell`: the point a footprint centred or cornered on the
    /// cell stands on.
    pub fn adjust_coord_to_cell(
        &self,
        terrain: &dyn ThingTerrainAccess,
        cell_x: Int,
        cell_y: Int,
        center_in_cell: bool,
        layer: PathfindLayerEnum,
    ) -> Coord3D {
        let coord = |c: Int| {
            if center_in_cell {
                (c as Real + 0.5) * PATHFIND_CELL_SIZE_F
            } else {
                // The 0.05 is a double in the reference.
                ((f64::from(c as Real) + 0.05) * f64::from(PATHFIND_CELL_SIZE_F)) as Real
            }
        };
        let (x, y) = (coord(cell_x), coord(cell_y));
        let z = terrain.get_layer_height(x, y, layer, None, 1);
        Coord3D::new(x, y, z)
    }

    // ---- reservations -------------------------------------------------------

    fn set_goal(&mut self, layer: PathfindLayerEnum, at: ICoord2D, id: ObjectId) {
        if let Some(idx) = self.get_cell(layer, at.x, at.y) {
            self.map.grid_mut().set_goal_unit(idx, id, at);
        }
    }

    /// `updateGoal`. `bridge_end` is `objectInteractsWithBridgeEnd` of the unit
    /// for a layer above the ground.
    pub fn update_goal(
        &mut self,
        mover: &mut GoalMover,
        new_goal: &Coord3D,
        layer: PathfindLayerEnum,
        bridge_end: &mut dyn FnMut(PathfindLayerEnum) -> bool,
    ) {
        if mover.immobile || !mover.agent.has_ai {
            return;
        }
        if !mover.ground_movement && !mover.unmanned_helicopter {
            self.update_aircraft_goal(mover, new_goal);
            return;
        }
        let layer_changed = mover.destination_layer != layer;
        let (do_ground, do_layer) = if layer == PathfindLayerEnum::Ground {
            (true, false)
        } else {
            (bridge_end(layer), true)
        };
        let goal = mover.goal_cell;
        let (radius, center) = radius_and_center(Some(&mover.agent));
        let above = radius + Int::from(center);
        let new_cell = footprint_cell(new_goal, center);
        if !layer_changed && new_cell == goal {
            return;
        }
        self.remove_goal(mover);
        mover.destination_layer = layer;
        mover.goal_cell = new_cell;
        let id = mover.agent.id;
        for at in footprint(new_cell, radius, above) {
            if do_layer {
                self.set_goal(layer, at, id);
            }
            if do_ground {
                self.set_goal(PathfindLayerEnum::Ground, at, id);
            }
        }
    }

    /// `updateAircraftGoal`.
    pub fn update_aircraft_goal(&mut self, mover: &mut GoalMover, new_goal: &Coord3D) {
        if mover.immobile {
            return;
        }
        self.remove_goal(mover);
        if !mover.agent.has_ai || mover.ground_movement || !mover.agent.aircraft_adjusts {
            return;
        }
        // `removeGoal` has just cleared the goal cell this compares with.
        let goal = mover.goal_cell;
        let (radius, center) = radius_and_center(Some(&mover.agent));
        let above = radius + Int::from(center);
        let new_cell = footprint_cell(new_goal, center);
        if new_cell == goal {
            return;
        }
        mover.goal_cell = new_cell;
        let id = mover.agent.id;
        for at in footprint(new_cell, radius, above) {
            if let Some(idx) = self.get_cell(PathfindLayerEnum::Ground, at.x, at.y) {
                self.map.grid_mut().set_goal_aircraft(idx, id, at);
            }
        }
    }

    /// `removeGoal`.
    pub fn remove_goal(&mut self, mover: &mut GoalMover) {
        if mover.immobile || !mover.agent.has_ai {
            return;
        }
        let goal = mover.goal_cell;
        let (mut radius, center) = radius_and_center(Some(&mover.agent));
        // One cell more than was reserved for a footprint of radius 0.
        if radius == 0 {
            radius = 1;
        }
        let above = radius + Int::from(center);
        let none = ICoord2D::new(-1, -1);
        if goal == none {
            return;
        }
        mover.goal_cell = none;
        if goal.x < 0 || goal.y < 0 {
            return;
        }
        let id = mover.agent.id;
        for at in footprint(goal, radius, above) {
            if let Some(idx) = self.get_cell(PathfindLayerEnum::Ground, at.x, at.y) {
                if self.goal_unit(idx) == id {
                    self.map
                        .grid_mut()
                        .set_goal_unit(idx, ObjectId::INVALID, at);
                }
                if self.goal_aircraft(idx) == id {
                    self.map
                        .grid_mut()
                        .set_goal_aircraft(idx, ObjectId::INVALID, at);
                }
            }
            if mover.destination_layer != PathfindLayerEnum::Ground {
                if let Some(idx) = self.get_cell(mover.destination_layer, at.x, at.y) {
                    if self.goal_unit(idx) == id {
                        self.map
                            .grid_mut()
                            .set_goal_unit(idx, ObjectId::INVALID, at);
                    }
                }
            }
        }
    }

    /// Takes the unit's presence off the cells around `at`.
    fn clear_presence(
        &mut self,
        id: ObjectId,
        layer: PathfindLayerEnum,
        at: ICoord2D,
        radius: Int,
        above: Int,
    ) {
        if at.x < 0 || at.y < 0 {
            return;
        }
        for at in footprint(at, radius, above) {
            let ground = (layer != PathfindLayerEnum::Ground).then_some(PathfindLayerEnum::Ground);
            for layer in std::iter::once(layer).chain(ground) {
                if let Some(idx) = self.get_cell(layer, at.x, at.y) {
                    if self.pos_unit(idx) == id {
                        self.map.grid_mut().set_pos_unit(idx, ObjectId::INVALID, at);
                    }
                }
            }
        }
    }

    /// `updatePos`.
    pub fn update_pos(
        &mut self,
        mover: &mut GoalMover,
        new_pos: &Coord3D,
        bridge_end: &mut dyn FnMut(PathfindLayerEnum) -> bool,
    ) {
        if mover.immobile || !self.is_map_ready() || !mover.agent.has_ai {
            return;
        }
        let current = mover.cur_cell;
        if !mover.ground_movement {
            if current.x >= 0 && current.y >= 0 {
                self.remove_pos(mover);
            }
            return;
        }
        let (radius, center) = radius_and_center(Some(&mover.agent));
        let above = radius + Int::from(center);
        let new_cell = footprint_cell(new_pos, center);
        if new_cell == current {
            return;
        }
        let layer = mover.agent.layer;
        let (do_ground, do_layer) = if layer == PathfindLayerEnum::Ground {
            (true, false)
        } else {
            (bridge_end(layer), true)
        };
        mover.cur_cell = new_cell;
        let id = mover.agent.id;
        self.clear_presence(id, layer, current, radius, above);
        for at in footprint(new_cell, radius, above) {
            let layers = [
                do_layer.then_some(layer),
                do_ground.then_some(PathfindLayerEnum::Ground),
            ];
            for layer in layers.into_iter().flatten() {
                if let Some(idx) = self.get_cell(layer, at.x, at.y) {
                    self.map.grid_mut().set_pos_unit(idx, id, at);
                }
            }
        }
    }

    /// `removePos`.
    pub fn remove_pos(&mut self, mover: &mut GoalMover) {
        if mover.immobile || !self.is_map_ready() || !mover.agent.has_ai {
            return;
        }
        let current = mover.cur_cell;
        let (radius, center) = radius_and_center(Some(&mover.agent));
        let above = radius + Int::from(center);
        mover.cur_cell = ICoord2D::new(-1, -1);
        self.clear_presence(mover.agent.id, mover.agent.layer, current, radius, above);
    }

    /// `removeUnitFromPathfindMap`.
    pub fn remove_unit_from_map(&mut self, mover: &mut GoalMover) {
        self.remove_pos(mover);
        self.remove_goal(mover);
    }

    /// `updateLayer`: the layer the object is set to when it is asked to move
    /// to `layer`. `interacts` is `objectInteractsWithBridgeLayer`.
    pub fn layer_for_update(
        layer: PathfindLayerEnum,
        interacts: &mut dyn FnMut(PathfindLayerEnum) -> bool,
    ) -> PathfindLayerEnum {
        if layer != PathfindLayerEnum::Ground && !interacts(layer) {
            return PathfindLayerEnum::Ground;
        }
        layer
    }

    // ---- snapping -----------------------------------------------------------

    /// `snapPosition`.
    pub fn snap_position(
        &self,
        terrain: &dyn ThingTerrainAccess,
        agent: Option<&PathAgent>,
        pos: &mut Coord3D,
    ) {
        let (_, center) = radius_and_center(agent);
        let (cell, _) = self.world_to_cell(&bias(pos, center));
        *pos =
            self.adjust_coord_to_cell(terrain, cell.x, cell.y, center, PathfindLayerEnum::Ground);
    }

    /// `snapClosestGoalPosition`: the nearest of the nine cells around `pos`
    /// that the unit may head for. Without one, `pos` is still snapped.
    pub fn snap_closest_goal_position(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        agent: &PathAgent,
        pos: &mut Coord3D,
    ) {
        let (radius, center) = radius_and_center(Some(agent));
        let biased = bias(pos, center);
        let layer = terrain.get_layer_for_destination(pos);
        let (cell, _) = self.world_to_cell(&biased);
        *pos =
            self.adjust_coord_to_cell(terrain, cell.x, cell.y, center, PathfindLayerEnum::Ground);
        if self.check_destination(units, Some(agent), cell.x, cell.y, layer, radius, center) {
            return;
        }
        let around = || {
            (cell.x - 1..cell.x + 2)
                .flat_map(move |i| (cell.y - 1..cell.y + 2).map(move |j| (i, j)))
        };
        for (i, j) in around() {
            if self.check_destination(units, Some(agent), i, j, layer, radius, center) {
                *pos = self.adjust_coord_to_cell(terrain, i, j, center, layer);
                return;
            }
        }
        if radius > 0 {
            return;
        }
        // A cell nobody else heads for, then one nobody stands fast in.
        let unclaimed = around().find(|&(i, j)| {
            self.get_cell(layer, i, j).is_some_and(|idx| {
                let goal = self.goal_unit(idx);
                goal == ObjectId::INVALID || goal == agent.id
            })
        });
        let found = unclaimed.or_else(|| {
            around().find(|&(i, j)| {
                self.get_cell(layer, i, j)
                    .is_some_and(|idx| self.cell_ref(idx).flags != CellFlags::UnitPresentFixed)
            })
        });
        if let Some((i, j)) = found {
            *pos = self.adjust_coord_to_cell(terrain, i, j, center, layer);
        }
    }

    /// `goalPosition`: where the reserved goal cell puts the unit. `pos` is
    /// zeroed even when there is no goal.
    pub fn goal_position(
        &self,
        terrain: &dyn ThingTerrainAccess,
        agent: &PathAgent,
        goal_cell: ICoord2D,
        pos: &mut Coord3D,
    ) -> bool {
        if !agent.has_ai {
            return false;
        }
        let (_, center) = radius_and_center(Some(agent));
        *pos = Coord3D::default();
        if goal_cell.x < 0 || goal_cell.y < 0 {
            return false;
        }
        *pos = self.adjust_coord_to_cell(
            terrain,
            goal_cell.x,
            goal_cell.y,
            center,
            PathfindLayerEnum::Ground,
        );
        true
    }

    // ---- destination searches -----------------------------------------------

    /// `checkForAdjust` without a group destination: the point for this cell,
    /// if the unit may head there and a path to it could exist.
    #[allow(clippy::too_many_arguments)]
    fn adjust_candidate(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &AdjustUnit,
        human: bool,
        cell_x: Int,
        cell_y: Int,
        layer: PathfindLayerEnum,
        dest: &Coord3D,
    ) -> Option<Coord3D> {
        let idx = self.get_cell(layer, cell_x, cell_y)?;
        let cell = self.cell_ref(idx);
        if cell.cell_type == CellType::Cliff {
            return None;
        }
        let logical = self.logical_extent();
        if human
            && (cell_x < logical.lo.x
                || cell_y < logical.lo.y
                || cell_x > logical.hi.x
                || cell_y > logical.hi.y)
        {
            return None;
        }
        let (radius, center) = radius_and_center(Some(&unit.agent));
        if !self.check_destination(
            units,
            Some(&unit.agent),
            cell_x,
            cell_y,
            layer,
            radius,
            center,
        ) {
            return None;
        }
        let adjusted = self.adjust_coord_to_cell(terrain, cell_x, cell_y, center, cell.layer);
        let reachable =
            |from: &Coord3D, to: &Coord3D| self.quick_path_exists(terrain, unit.surfaces, from, to);
        let exists = unit.aircraft
            || (!reachable(&unit.position, dest) && reachable(dest, &adjusted))
            || reachable(&unit.position, &adjusted);
        exists.then_some(adjusted)
    }

    /// `tightenPath`: moves `from` along the line towards `to`, to the last cell
    /// the unit may head for.
    pub fn tighten_path(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &AdjustUnit,
        from: &mut Coord3D,
        to: &Coord3D,
    ) {
        let layer = terrain.get_layer_for_destination(from);
        let mut found = false;
        let mut new_dest = Coord3D::default();
        self.walk(from, to, layer, &mut |previous, idx, x, y| {
            if previous.is_none() {
                return 0;
            }
            let cell_layer = self.cell_ref(idx).layer;
            if layer != cell_layer {
                return 0;
            }
            // The retail layout asks about the last cell found, not the target.
            #[cfg(feature = "retail-crc")]
            let dest = new_dest;
            #[cfg(not(feature = "retail-crc"))]
            let dest = *to;
            if let Some(pos) =
                self.adjust_candidate(terrain, units, unit, true, x, y, cell_layer, &dest)
            {
                found = true;
                new_dest = pos;
            }
            0
        });
        if found {
            *from = new_dest;
        }
    }

    /// `checkForAdjust`.
    #[allow(clippy::too_many_arguments)]
    fn check_for_adjust(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        cost: &mut dyn PathCost,
        unit: &AdjustUnit,
        cell_x: Int,
        cell_y: Int,
        layer: PathfindLayerEnum,
        dest: &mut Coord3D,
        group_dest: Option<&Coord3D>,
    ) -> bool {
        let Some(mut adjusted) = self.adjust_candidate(
            terrain, units, unit, unit.human, cell_x, cell_y, layer, dest,
        ) else {
            return false;
        };
        if let Some(group) = group_dest {
            self.tighten_path(terrain, units, unit, &mut adjusted, group);
            let price = cost.check_path_cost(self, unit, group, &adjusted);
            // `IABS` takes an `Int`: each difference is truncated first.
            let dx = ((group.x - adjusted.x) as Int).wrapping_abs();
            let dy = ((group.y - adjusted.y) as Int).wrapping_abs();
            if 1.4 * (dx.wrapping_add(dy) as Real) < price as Real {
                return false;
            }
        }
        *dest = adjusted;
        true
    }

    /// `adjustDestination`: moves `dest` to the nearest cell the unit may head
    /// for. With a group destination, cells far from it by path are passed over,
    /// and failing that it is ignored.
    pub fn adjust_destination(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        cost: &mut dyn PathCost,
        unit: &AdjustUnit,
        dest: &mut Coord3D,
        group_dest: Option<&Coord3D>,
    ) -> bool {
        if unit.projectile {
            return true;
        }
        let (_, center) = radius_and_center(Some(&unit.agent));
        let (cell, _) = self.world_to_cell(&bias(dest, center));
        let layer = terrain.get_layer_for_destination(group_dest.unwrap_or(dest));
        if self.check_for_adjust(
            terrain, units, cost, unit, cell.x, cell.y, layer, dest, group_dest,
        ) {
            return true;
        }
        let found = spiral(cell, |i, j| {
            self.check_for_adjust(terrain, units, cost, unit, i, j, layer, dest, group_dest)
                .then_some(())
        });
        if found.is_some() {
            return true;
        }
        group_dest.is_some() && self.adjust_destination(terrain, units, cost, unit, dest, None)
    }

    /// `checkForLanding`.
    #[allow(clippy::too_many_arguments)]
    fn landing_candidate(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        cell_x: Int,
        cell_y: Int,
        layer: PathfindLayerEnum,
        radius: Int,
        center: bool,
    ) -> Option<Coord3D> {
        let idx = self.get_cell(layer, cell_x, cell_y)?;
        let cell = self.cell_ref(idx);
        if matches!(
            cell.cell_type,
            CellType::Cliff | CellType::Water | CellType::Impassable
        ) {
            return None;
        }
        self.check_destination(units, None, cell_x, cell_y, layer, radius, center)
            .then(|| self.adjust_coord_to_cell(terrain, cell_x, cell_y, center, cell.layer))
    }

    /// `adjustToLandingDestination`. `extent` is `getMaximumPathfindExtent`.
    pub fn adjust_to_landing_destination(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        extent: &Region3D,
        agent: &PathAgent,
        position: &Coord3D,
        dest: &mut Coord3D,
    ) -> bool {
        let (radius, center) = radius_and_center(Some(agent));
        let within = |p: &Coord3D| {
            extent.lo.x < p.x && p.x < extent.hi.x && extent.lo.y < p.y && p.y < extent.hi.y
        };
        // A unit off the map flying further off is let be.
        if !within(dest) && !within(position) {
            return true;
        }
        let (cell, _) = self.world_to_cell(&bias(dest, center));
        let layer = terrain.get_layer_for_destination(dest);
        let at = |i, j| self.landing_candidate(terrain, units, i, j, layer, radius, center);
        match at(cell.x, cell.y).or_else(|| spiral(cell, at)) {
            Some(found) => {
                *dest = found;
                true
            }
            None => false,
        }
    }

    /// `adjustTargetDestination`: a free cell from which the weapon reaches.
    /// `in_range` is `Weapon::isGoalPosWithinAttackRange` for a point.
    pub fn adjust_target_destination(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        agent: &PathAgent,
        in_range: &mut dyn FnMut(&Coord3D) -> Bool,
        dest: &mut Coord3D,
    ) -> bool {
        let (radius, center) = radius_and_center(Some(agent));
        let (cell, off_map) = self.world_to_cell(&bias(dest, center));
        if off_map {
            return false;
        }
        let mut at = |i, j| {
            let layer = PathfindLayerEnum::Ground;
            if !self.check_destination(units, Some(agent), i, j, layer, radius, center) {
                return None;
            }
            let adjusted = self.adjust_coord_to_cell(terrain, i, j, center, layer);
            (in_range(&adjusted) != 0).then_some(adjusted)
        };
        match at(cell.x, cell.y).or_else(|| spiral(cell, &mut at)) {
            Some(found) => {
                *dest = found;
                true
            }
            None => false,
        }
    }

    /// The effective zone of a cell for this unit's locomotors.
    fn zone_for(&self, unit: &AdjustUnit, zone: UnsignedShort) -> UnsignedShort {
        let crusher = Bool::from(unit.agent.crusher_level > 0);
        self.zones.get_effective_zone(unit.surfaces, crusher, zone)
    }

    /// `checkForPossible`: true, with `dest` moved to the cell, when the cell is
    /// in the unit's zone. Whether it is free is asked afterwards.
    #[allow(clippy::too_many_arguments)]
    fn check_for_possible(
        &self,
        terrain: &dyn ThingTerrainAccess,
        unit: &AdjustUnit,
        from_zone: UnsignedShort,
        center: bool,
        cell_x: Int,
        cell_y: Int,
        layer: PathfindLayerEnum,
        dest: &mut Coord3D,
        starting_in_obstacle: bool,
    ) -> bool {
        let Some(idx) = self.get_cell(layer, cell_x, cell_y) else {
            return false;
        };
        let cell = self.cell_ref(idx);
        if is_impassable(cell.cell_type) {
            return false;
        }
        let mut zone = self.zone_for(unit, cell.zone);
        if starting_in_obstacle {
            zone = self.zones.get_effective_terrain_zone(zone);
        }
        if from_zone != zone {
            return false;
        }
        *dest = self.adjust_coord_to_cell(terrain, cell_x, cell_y, center, layer);
        true
    }

    /// `adjustToPossibleDestination`: moves `dest` to a free cell in the zone
    /// the unit stands in. A failed search can leave `dest` moved to an
    /// occupied cell of that zone.
    pub fn adjust_to_possible_destination(
        &self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &AdjustUnit,
        dest: &mut Coord3D,
    ) -> bool {
        let (radius, center) = radius_and_center(Some(&unit.agent));
        let (goal, off_map) = self.world_to_cell(&bias(dest, center));
        if off_map {
            return false;
        }
        let layer = terrain.get_layer_for_destination(dest);
        let (from, _) = self.world_to_cell(&unit.position);
        let Some(parent) = self.get_cell(unit.agent.layer, from.x, from.y) else {
            return false;
        };
        let parent = self.cell_ref(parent);
        let mut zone = self.zone_for(unit, parent.zone);
        let in_obstacle = parent.cell_type == CellType::Obstacle;
        if in_obstacle {
            zone = self.zones.get_effective_terrain_zone(zone);
            zone = self.zone_for(unit, zone);
        }
        let free =
            |i, j| self.check_destination(units, Some(&unit.agent), i, j, layer, radius, center);
        // The reference reads the goal cell without a check; it exists for a
        // point that was not clipped.
        let Some(goal_cell) = self.get_cell(layer, goal.x, goal.y) else {
            return false;
        };
        if zone == self.zone_for(unit, self.cell_ref(goal_cell).zone) && free(goal.x, goal.y) {
            return true;
        }
        spiral(goal, |i, j| {
            (self.check_for_possible(terrain, unit, zone, center, i, j, layer, dest, in_obstacle)
                && free(i, j))
            .then_some(())
        })
        .is_some()
    }

    // ---- allies -------------------------------------------------------------

    /// `moveAllies`: tells the idle allies standing on the path to move away.
    /// False when nothing was looked at.
    pub fn move_allies(
        &mut self,
        world: &mut dyn MoveAlliesWorld,
        mover: &AllyMover,
        path: &Path,
    ) -> bool {
        if !mover.dozer_or_harvester && path.blocked_by_ally() == 0 {
            return false;
        }
        self.move_allies_depth += 1;
        let looked = self.move_allies_depth <= MOVE_ALLIES_MAX_DEPTH;
        if looked {
            self.move_allies_along(world, mover, path);
        }
        self.move_allies_depth -= 1;
        looked
    }

    /// `moveAlliesAwayFromDestination`: orders every idle ally standing on the
    /// straight line from `position` to `destination` out of the way. `mover`
    /// has an AI. Unlike [`Pathfinder::move_allies`] it has no depth limit and
    /// no regard for infantry or attacks.
    pub fn move_allies_away_from_destination(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        world: &mut dyn MoveAlliesWorld,
        mover: &PathAgent,
        position: &Coord3D,
        destination: &Coord3D,
    ) {
        let mut layer = mover.layer;
        if layer == PathfindLayerEnum::Ground {
            layer = terrain.get_layer_for_destination(destination);
        }
        // The walk never stops early, so the cells can be listed first; who
        // stands in each is read when its turn comes.
        let mut cells = Vec::new();
        self.walk(position, destination, layer, &mut |_, idx, _, _| {
            cells.push(idx);
            0
        });
        for idx in cells {
            let there = self.pos_unit(idx);
            if there == ObjectId::INVALID || there == mover.id || there == mover.ignored_obstacle {
                continue;
            }
            let Some(other) = world.ally(mover.id, there) else {
                continue;
            };
            if other.ally && other.has_ai && !other.moving && !other.using_ability && !other.busy {
                world.move_away_from_unit(self, there, mover.id);
            }
        }
    }

    fn move_allies_along(
        &mut self,
        world: &mut dyn MoveAlliesWorld,
        mover: &AllyMover,
        path: &Path,
    ) {
        let agent = &mover.agent;
        let (radius, center) = radius_and_center(Some(agent));
        let above = radius + Int::from(center);
        let ignored = if agent.has_ai {
            agent.ignored_obstacle
        } else {
            ObjectId::INVALID
        };
        let mut node = path.last_node();
        while let Some(at) = node {
            if node == path.first_node() {
                break;
            }
            let here = path.node(at);
            let (cell, _) = self.world_to_cell(&here.pos);
            for around in footprint(cell, radius, above) {
                let Some(idx) = self.get_cell(here.layer, around.x, around.y) else {
                    continue;
                };
                let there = self.pos_unit(idx);
                if there == ObjectId::INVALID || there == agent.id || there == ignored {
                    continue;
                }
                let Some(other) = world.ally(agent.id, there) else {
                    continue;
                };
                if !other.ally {
                    continue;
                }
                if agent.infantry && other.infantry {
                    continue;
                }
                if agent.infantry && !other.infantry && path.blocked_by_ally() == 0 {
                    continue;
                }
                if !other.has_ai || other.moving {
                    continue;
                }
                if other.attacking || other.using_ability || other.busy {
                    continue;
                }
                world.move_away_from_unit(self, there, agent.id);
            }
            node = here.prev;
        }
    }
}
