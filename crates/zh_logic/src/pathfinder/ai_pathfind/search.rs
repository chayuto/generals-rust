//! The search half of `Pathfinder` (`PKT-411`): the open and closed lists, the
//! neighbour checks, the ground A* search, building a path from the cells it
//! leaves and the cost-only search.
//!
//! Golden test: `crates/zh_logic/tests/golden_s5_search.rs`.
//!
//! The reference build is `RETAIL_COMPATIBLE_PATHFINDING`: until a search would
//! crash it runs the old list code, then `PathfindMode::use_fixed_pathfinding`
//! stays set for the rest of the map.

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::{gm_sqrtf, Coord3D, ICoord2D, IRegion2D};
use zh_common::types::{Int, Real, UnsignedInt, FALSE};

use super::Pathfinder;
use super::{radius_and_center, AdjustUnit, CheckMovementInfo, PathAgent, PathCost, PathUnits};
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::{Path, PathQuery};
use crate::pathfinder::pathfind_cell::CellType;
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, COST_DIAGONAL, COST_ORTHOGONAL, MAX_CELL_COUNT, PATHFIND_CELL_SIZE_F,
};
use crate::rules::combat_types::LocomotorSurfaceTypeMask;

/// `checkPathCost`'s answer for "no path".
pub const MAX_PATH_COST: Int = 0x7fff_0000;

/// What a search reads from the unit it searches for.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchUnit {
    pub agent: PathAgent,
    /// `LocomotorSet::getValidSurfaces`.
    pub surfaces: LocomotorSurfaceTypeMask,
    /// `LocomotorSet::isDownhillOnly`.
    pub downhill_only: bool,
    /// False for the unit of a computer player.
    pub human: bool,
    /// `AIUpdateInterface::canPathThroughUnits`, false without an AI.
    pub can_path_through_units: bool,
    /// `KINDOF_DOZER`.
    pub dozer: bool,
    /// `KINDOF_VEHICLE`.
    pub vehicle: bool,
}

/// What stays the same while one cell's neighbours are examined.
#[derive(Debug, Clone, Copy)]
pub struct NeighborSearch<'a> {
    pub unit: &'a SearchUnit,
    pub goal: Option<CellIdx>,
    pub center_in_cell: bool,
    pub radius: Int,
    /// `startCellNdx`.
    pub start: ICoord2D,
    /// Zero (`NO_ATTACK`) except in the attack search.
    pub attack_distance: Int,
}

const NEIGHBORS: [(Int, Int); 8] = [
    (1, 0),
    (0, 1),
    (-1, 0),
    (0, -1),
    (1, 1),
    (-1, 1),
    (-1, -1),
    (1, -1),
];
const FIRST_DIAGONAL: usize = 4;
const ADJACENT: [usize; 5] = [0, 1, 2, 3, 0];

/// `ClipLine2D`: the ends of the line cut to the region, `None` when it misses.
fn clip_line(p1: ICoord2D, p2: ICoord2D, region: &IRegion2D) -> Option<(ICoord2D, ICoord2D)> {
    const LEFT: u8 = 0x01;
    const RIGHT: u8 = 0x02;
    const BOTTOM: u8 = 0x04;
    const TOP: u8 = 0x08;
    let (left, right, top, bottom) = (region.lo.x, region.hi.x, region.lo.y, region.hi.y);
    let code = |x: Int, y: Int| {
        let mut code = 0;
        if x < left {
            code = LEFT;
        } else if x > right {
            code = RIGHT;
        }
        if y < top {
            code |= TOP;
        } else if y > bottom {
            code |= BOTTOM;
        }
        code
    };
    let (mut x1, mut y1, mut x2, mut y2) = (p1.x, p1.y, p2.x, p2.y);
    let (code1, code2) = (code(x1, y1), code(x2, y2));
    if code1 | code2 == 0 {
        return Some((p1, p2));
    }
    if code1 & code2 != 0 {
        return None;
    }
    // `(span * part) / diff` in `Int`, `None` for a zero `diff`.
    let along = |span: Int, part: Int, diff: Int| {
        (diff != 0).then(|| span.wrapping_mul(part).wrapping_div(diff))
    };
    if code1 != 0 {
        if code1 & (TOP | BOTTOM) != 0 {
            let edge = if code1 & TOP != 0 { top } else { bottom };
            let diff = y2.wrapping_sub(y1);
            x1 = x1.wrapping_add(along(x2.wrapping_sub(x1), edge.wrapping_sub(y1), diff)?);
            y1 = edge;
        }
        if x1 > right || x1 < left {
            let edge = if x1 > right { right } else { left };
            let diff = x2.wrapping_sub(x1);
            y1 = y1.wrapping_add(along(y2.wrapping_sub(y1), edge.wrapping_sub(x1), diff)?);
            x1 = edge;
        }
    }
    if code2 != 0 {
        if code2 & (TOP | BOTTOM) != 0 {
            let edge = if code2 & TOP != 0 { top } else { bottom };
            let diff = y2.wrapping_sub(y1);
            x2 = x2.wrapping_add(along(x2.wrapping_sub(x1), edge.wrapping_sub(y2), diff)?);
            y2 = edge;
        }
        if x2 > right || x2 < left {
            let edge = if x2 > right { right } else { left };
            let diff = x2.wrapping_sub(x1);
            y2 = y2.wrapping_add(along(y2.wrapping_sub(y1), edge.wrapping_sub(x2), diff)?);
            x2 = edge;
        }
    }
    let inside = |x: Int, y: Int| x >= left && x <= right && y >= top && y <= bottom;
    (inside(x1, y1) && inside(x2, y2)).then(|| (ICoord2D::new(x1, y1), ICoord2D::new(x2, y2)))
}

impl Pathfinder {
    // ---- cell records -----------------------------------------------------

    pub(super) fn has_info(&self, idx: CellIdx) -> bool {
        self.cell_ref(idx).info_idx.is_some()
    }

    /// `getXIndex` and `getYIndex`, which read the record.
    pub(super) fn index_of(&self, idx: CellIdx) -> ICoord2D {
        let pos = self
            .cell_info(idx)
            .map_or_else(ICoord2D::default, |i| i.pos);
        ICoord2D::new(Int::from(pos.x as u16), Int::from(pos.y as u16))
    }

    /// `hasInfo() && (getOpen() || getClosed())`.
    pub(super) fn is_listed(&self, idx: CellIdx) -> bool {
        self.cell_info(idx).is_some_and(|i| i.open || i.closed)
    }

    fn is_open(&self, idx: CellIdx) -> bool {
        self.cell_info(idx).is_some_and(|i| i.open)
    }

    fn is_closed(&self, idx: CellIdx) -> bool {
        self.cell_info(idx).is_some_and(|i| i.closed)
    }

    pub(super) fn cost_so_far_of(&self, idx: CellIdx) -> UnsignedInt {
        self.cell_info(idx)
            .map_or(0, |i| UnsignedInt::from(i.cost_so_far))
    }

    /// `getParentCell`: the cell named by the parent record, which in the retail
    /// layout may since have lost it.
    pub fn parent_cell(&self, idx: CellIdx) -> Option<CellIdx> {
        let parent = self.cell_info(idx)?.path_parent?;
        self.map.pool().get(parent)?.cell
    }

    pub(super) fn allocate_cell_info(&mut self, idx: CellIdx, pos: ICoord2D) -> bool {
        self.map.grid_mut().allocate_info(idx, pos)
    }

    pub(super) fn release_cell_info(&mut self, idx: CellIdx) {
        self.map.grid_mut().release_info(idx);
    }

    pub(super) fn open_cell(&mut self, idx: CellIdx) {
        self.map
            .grid_mut()
            .put_on_sorted_open_list(idx, &mut self.open_list);
    }

    pub(super) fn close_cell(&mut self, idx: CellIdx) {
        self.map
            .grid_mut()
            .put_on_closed_list(idx, &mut self.closed_list);
    }

    /// Takes a cell off whichever list it is on.
    pub(super) fn unlist_cell(&mut self, idx: CellIdx) {
        if self.is_closed(idx) {
            self.map
                .grid_mut()
                .remove_from_closed_list(idx, &mut self.closed_list);
        }
        if self.is_open(idx) {
            self.map
                .grid_mut()
                .remove_from_open_list(idx, &mut self.open_list);
        }
    }

    /// The head of the open list, taken off it. A head without a record would
    /// fault the reference, so the search ends there.
    pub(super) fn pop_open(&mut self) -> Option<CellIdx> {
        let head = self.open_list.get_head().filter(|&h| self.has_info(h))?;
        self.map
            .grid_mut()
            .remove_from_open_list(head, &mut self.open_list);
        Some(head)
    }

    pub(super) fn fixed_pathfinding(&self) -> bool {
        self.map.mode().use_fixed_pathfinding
    }

    /// `startPathfind` and putting the first cell on the emptied lists.
    pub(super) fn start_search(&mut self, start: CellIdx, goal: Option<CellIdx>) {
        self.map.grid_mut().start_pathfind(start, goal);
        if self.fixed_pathfinding() {
            self.open_list.reset(None);
            self.open_cell(start);
        } else {
            self.open_list.reset(Some(start));
        }
        self.closed_list.reset(None);
    }

    /// How a successful search ends: the retail code releases the cell before
    /// it empties the lists, the fixed code after.
    pub(super) fn finish_search(&mut self, cell: CellIdx) {
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            self.release_cell_info(cell);
        } else {
            self.release_cell_info(cell);
            self.clean_open_and_closed_lists();
        }
    }

    // ---- lists ------------------------------------------------------------

    /// `forceCleanCells`: drops every list link and releases the records of the
    /// ground cells. Layer cells keep theirs.
    pub fn force_clean_cells(&mut self) {
        self.map.pool_mut().force_clean_pathfind_cell_infos();
        self.open_list.reset(None);
        self.closed_list.reset(None);
        for j in 0..=self.extent.hi.y {
            for i in 0..=self.extent.hi.x {
                let idx = self.map.index(i, j);
                let grid = self.map.grid_mut();
                if grid.is_obstruction_invalid(idx) {
                    grid.clear_obstruction(idx);
                }
                if grid.cell(idx).info_idx.is_some() {
                    grid.release_info(idx);
                }
            }
        }
    }

    fn clean_if_forced(&mut self) {
        if self.map.mode().force_clean_cells {
            self.force_clean_cells();
            self.map.mode_mut().force_clean_cells = false;
        }
    }

    /// `cleanOpenAndClosedLists`: counts the cells into
    /// `cumulative_cells_allocated`.
    pub fn clean_open_and_closed_lists(&mut self) {
        let mut count: Int = 0;
        if !self.open_list.empty() {
            count = count.wrapping_add(self.map.grid_mut().release_open_list(&mut self.open_list));
            self.open_list.reset(None);
        }
        self.clean_if_forced();
        if !self.closed_list.empty() {
            count = count.wrapping_add(
                self.map
                    .grid_mut()
                    .release_closed_list(&mut self.closed_list),
            );
            self.closed_list.reset(None);
        }
        self.clean_if_forced();
        self.cumulative_cells_allocated = self.cumulative_cells_allocated.wrapping_add(count);
    }

    // ---- neighbours -------------------------------------------------------

    /// `checkCellOutsideExtents`.
    pub fn check_cell_outside_extents(&self, cell: ICoord2D) -> bool {
        let e = &self.logical_extent;
        cell.x < e.lo.x || cell.x > e.hi.x || cell.y < e.lo.y || cell.y > e.hi.y
    }

    /// `checkChangeLayers`: opens the cell this one connects to.
    pub(super) fn check_change_layers(&mut self, parent: CellIdx) {
        let layer = self.cell_ref(parent).connects_to_layer;
        if layer == PathfindLayerEnum::Invalid {
            return;
        }
        let at = self.index_of(parent);
        let Some(cell) = self.get_cell(layer, at.x, at.y) else {
            return;
        };
        if self.is_listed(cell) || !self.allocate_cell_info(cell, at) {
            return;
        }
        let cost = self.cost_so_far_of(parent);
        let total = self
            .cell_info(parent)
            .map_or(0, |i| UnsignedInt::from(i.total_cost));
        let grid = self.map.grid_mut();
        grid.set_parent_cell(cell, parent);
        grid.set_cost_so_far(cell, cost);
        grid.set_total_cost(cell, total);
        self.open_cell(cell);
    }

    /// `examineCellsCallback`: one step of the straight line towards the goal.
    /// False ends the walk.
    fn examine_line_cell(
        &mut self,
        units: &dyn PathUnits,
        search: &NeighborSearch,
        goal: CellIdx,
        from: CellIdx,
        to: CellIdx,
        at: ICoord2D,
    ) -> bool {
        let unit = search.unit;
        let crusher = unit.agent.crusher_level > 0;
        if !self.valid_movement_cell(crusher, unit.surfaces, Some(to)) {
            return false;
        }
        let cell = self.cell_ref(to);
        if cell.layer == PathfindLayerEnum::Ground && self.zones.is_passable(at.x, at.y) == FALSE {
            return false;
        }
        if cell.pinched {
            return false;
        }
        if unit.human && self.check_cell_outside_extents(at) {
            return false;
        }
        let mut info = CheckMovementInfo {
            cell: at,
            layer: self.cell_ref(from).layer,
            radius: search.radius,
            center_in_cell: search.center_in_cell,
            consider_transient: false,
            ..CheckMovementInfo::default()
        };
        if !self.check_for_movement(units, Some(&unit.agent), &mut info)
            || info.enemy_fixed
            || info.ally_fixed_count != 0
        {
            return false;
        }
        let cost =
            (self.cost_so_far_of(from) as Real + 0.5 * COST_ORTHOGONAL as Real) as UnsignedInt;
        if self.cell_ref(to).cell_type == CellType::Cliff {
            return false;
        }
        if !self.allocate_cell_info(to, at) {
            return false;
        }
        self.map.grid_mut().set_blocked_by_ally(to, false);
        let remaining = self.map.grid().cost_to_goal(to, goal);
        if self.is_listed(to) && self.cost_so_far_of(to) <= cost {
            return true;
        }
        let grid = self.map.grid_mut();
        grid.set_cost_so_far(to, cost);
        grid.set_parent_cell(to, from);
        let total = self.cost_so_far_of(to).wrapping_add(remaining);
        self.map.grid_mut().set_total_cost(to, total);
        self.unlist_cell(to);
        self.open_cell(to);
        true
    }

    /// `examineNeighboringCells`: opens what can be entered from `parent`.
    /// Returns how many cells took a new record.
    pub fn examine_neighboring_cells(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        parent: CellIdx,
        search: &NeighborSearch,
    ) -> Int {
        let unit = search.unit;
        let crusher = unit.agent.crusher_level > 0;
        let parent_layer = self.cell_ref(parent).layer;
        let parent_at = self.index_of(parent);
        if let Some(goal) = search.goal {
            if search.attack_distance == 0 && !self.is_tunneling && !unit.downhill_only {
                // The walk only depends on the map's shape, so it is listed first.
                let mut steps = Vec::new();
                self.iterate_cells_along_line(
                    parent_at,
                    self.index_of(goal),
                    parent_layer,
                    &mut |from, to, x, y| {
                        steps.push((from, to, ICoord2D::new(x, y)));
                        0
                    },
                );
                for (from, to, at) in steps {
                    let Some(from) = from else { continue };
                    if !self.examine_line_cell(units, search, goal, from, to, at) {
                        break;
                    }
                }
            }
        }

        let mut count: Int = 0;
        let mut flags = [false; 8];
        let size = PATHFIND_CELL_SIZE_F;
        let from_z =
            terrain.get_ground_height(parent_at.x as Real * size, parent_at.y as Real * size, None);
        for (i, (step_x, step_y)) in NEIGHBORS.into_iter().enumerate() {
            let at = ICoord2D::new(
                parent_at.x.wrapping_add(step_x),
                parent_at.y.wrapping_add(step_y),
            );
            let Some(cell) = self.get_cell(parent_layer, at.x, at.y) else {
                continue;
            };
            let not_zone_passable = self.cell_ref(cell).layer == PathfindLayerEnum::Ground
                && self.zones.is_passable(at.x, at.y) == FALSE;
            if unit.human && self.check_cell_outside_extents(at) {
                continue;
            }
            if self.is_listed(cell) {
                continue;
            }
            if i >= FIRST_DIAGONAL && !flags[ADJACENT[i - 4]] && !flags[ADJACENT[i - 3]] {
                continue;
            }
            let ground = |at: ICoord2D| {
                terrain.get_ground_height(at.x as Real * size, at.y as Real * size, None)
            };
            if unit.downhill_only && from_z < ground(at) {
                continue;
            }
            let mut movement_valid = self.valid_movement_cell(crusher, unit.surfaces, Some(cell));
            let mut dozer_hack = false;
            if !movement_valid && unit.dozer && self.cell_ref(cell).cell_type == CellType::Obstacle
            {
                let obstacle = self
                    .cell_ref(cell)
                    .obstacle_id(self.map.pool(), self.map.mode());
                if units.is_non_enemy_object(unit.agent.id, obstacle) {
                    movement_valid = true;
                    dozer_hack = true;
                }
            }
            if !movement_valid && !self.is_tunneling {
                continue;
            }
            if !dozer_hack {
                flags[i] = true;
            }
            let mut info = CheckMovementInfo {
                cell: at,
                layer: parent_layer,
                radius: search.radius,
                center_in_cell: search.center_in_cell,
                consider_transient: false,
                ..CheckMovementInfo::default()
            };
            let mut dx = at.x.wrapping_sub(search.start.x).wrapping_abs();
            let mut dy = at.y.wrapping_sub(search.start.y).wrapping_abs();
            if !self.check_for_movement(units, Some(&unit.agent), &mut info) || info.enemy_fixed {
                if !self.is_tunneling {
                    continue;
                }
                movement_valid = false;
            }
            if movement_valid && !self.cell_ref(cell).pinched {
                self.is_tunneling = false;
            }
            if !self.has_info(cell) {
                if !self.allocate_cell_info(cell, at) {
                    return count;
                }
                count += 1;
            }
            let mut cost = self.map.grid().cost_so_far(cell, Some(parent));
            if info.ally_moving && dx < 10 && dy < 10 {
                cost = cost.wrapping_add(3 * COST_DIAGONAL as UnsignedInt);
            }
            let pinched = self.cell_ref(cell).pinched;
            if self.cell_ref(cell).cell_type == CellType::Cliff && !pinched {
                if (from_z - ground(at)).abs() < size {
                    cost = cost.wrapping_add(7 * COST_DIAGONAL as UnsignedInt);
                }
            } else if pinched {
                cost = cost.wrapping_add(COST_ORTHOGONAL as UnsignedInt);
            }
            self.map.grid_mut().set_blocked_by_ally(cell, false);
            if info.ally_fixed_count > 0 {
                cost = cost.wrapping_add(3 * COST_DIAGONAL as UnsignedInt);
                if !unit.can_path_through_units {
                    self.map.grid_mut().set_blocked_by_ally(cell, true);
                }
            }
            let mut remaining: Int = 0;
            if let Some(goal) = search.goal {
                if search.attack_distance == 0 {
                    remaining = self.map.grid().cost_to_goal(cell, goal) as Int;
                } else {
                    let goal_at = self.index_of(goal);
                    dx = at.x.wrapping_sub(goal_at.x);
                    dy = at.y.wrapping_sub(goal_at.y);
                    let square = dx.wrapping_mul(dx).wrapping_add(dy.wrapping_mul(dy));
                    remaining = (COST_ORTHOGONAL as Real * gm_sqrtf(square as Real)) as Int;
                    remaining = remaining.wrapping_sub(search.attack_distance / 2);
                    if remaining < 0 {
                        remaining = 0;
                    }
                    if info.ally_goal {
                        cost = cost.wrapping_add(if unit.vehicle {
                            3 * COST_ORTHOGONAL as UnsignedInt
                        } else {
                            COST_ORTHOGONAL as UnsignedInt
                        });
                    }
                }
            }
            if not_zone_passable {
                cost = cost.wrapping_add(100 * COST_ORTHOGONAL as UnsignedInt);
            }
            if self.cell_ref(cell).cell_type == CellType::Obstacle {
                cost = cost.wrapping_add(100 * COST_ORTHOGONAL as UnsignedInt);
            }
            if self.is_tunneling && !self.valid_movement_cell(crusher, unit.surfaces, Some(cell)) {
                cost = cost.wrapping_add(10 * COST_ORTHOGONAL as UnsignedInt);
            }
            let grid = self.map.grid_mut();
            grid.set_cost_so_far(cell, cost);
            grid.set_parent_cell(cell, parent);
            if self.is_tunneling {
                remaining = 0;
            }
            let total = self
                .cost_so_far_of(cell)
                .wrapping_add(remaining as UnsignedInt);
            self.map.grid_mut().set_total_cost(cell, total);
            self.unlist_cell(cell);
            self.open_cell(cell);
        }
        count
    }

    // ---- ground search ----------------------------------------------------

    /// `clip`: moves the ends of a line onto the map.
    pub fn clip(&self, from: &mut Coord3D, to: &mut Coord3D) {
        let cell = |pos: &Coord3D| {
            ICoord2D::new(
                real_to_int_floor(pos.x / PATHFIND_CELL_SIZE_F),
                real_to_int_floor(pos.y / PATHFIND_CELL_SIZE_F),
            )
        };
        let (from_cell, to_cell) = (cell(from), cell(to));
        let Some((clip_from, clip_to)) = clip_line(from_cell, to_cell, &self.extent) else {
            return;
        };
        let place = |pos: &mut Coord3D, old: ICoord2D, new: ICoord2D| {
            if old != new {
                pos.x = new.x as Real * PATHFIND_CELL_SIZE_F + 0.05;
                pos.y = new.y as Real * PATHFIND_CELL_SIZE_F + 0.05;
            }
        };
        place(from, from_cell, clip_from);
        place(to, to_cell, clip_to);
    }

    /// `internalFindPath`: A* over the cells. It reads the passable flags the
    /// hierarchical pass left, and expects empty lists.
    pub fn internal_find_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        from: &Coord3D,
        raw_to: &Coord3D,
    ) -> Option<Path> {
        let agent = &unit.agent;
        let (radius, center_in_cell) = radius_and_center(Some(agent));
        if raw_to.x == 0.0 && raw_to.y == 0.0 {
            return None;
        }
        if !self.is_map_ready {
            return None;
        }
        let mut to = *raw_to;
        let mut clip_from = *from;
        self.clip(&mut clip_from, &mut to);
        if !center_in_cell {
            to.x += PATHFIND_CELL_SIZE_F / 2.0;
            to.y += PATHFIND_CELL_SIZE_F / 2.0;
        }
        self.is_tunneling = false;

        let destination_layer = terrain.get_layer_for_destination(&to);
        let goal = self.get_cell_at(destination_layer, &to)?;
        let (cell, _) = self.world_to_cell(&to);
        if !self.check_destination(
            units,
            Some(agent),
            cell.x,
            cell.y,
            destination_layer,
            radius,
            center_in_cell,
        ) {
            return None;
        }
        let (start, _) = self.world_to_cell(&clip_from);
        let layer = agent.layer;
        let first = self.clipped_cell(layer, &clip_from)?;

        if !self.allocate_cell_info(goal, cell) {
            return None;
        }
        if first != goal && !self.allocate_cell_info(first, start) {
            self.release_cell_info(goal);
            return None;
        }
        self.is_tunneling = self.cell_ref(first).cell_type == CellType::Obstacle;

        let crusher = agent.crusher_level > 0;
        let zone_of = |pf: &Self, idx: CellIdx| {
            pf.zones
                .get_effective_zone(unit.surfaces, u8::from(crusher), pf.cell_ref(idx).zone)
        };
        let mut zone1 = zone_of(self, first);
        let mut zone2 = zone_of(self, goal);
        if (layer == PathfindLayerEnum::Wall && zone1 == 0)
            || (destination_layer == PathfindLayerEnum::Wall && zone2 == 0)
        {
            // The retail code leaves both records behind.
            if self.fixed_pathfinding() {
                self.release_cell_info(goal);
                self.release_cell_info(first);
            }
            return None;
        }
        if self.cell_ref(goal).is_obstacle_present(
            self.ignore_obstacle_id,
            self.map.pool(),
            self.map.mode(),
        ) || self.is_tunneling
        {
            let terrain_pair = |zone| {
                let zones = &self.zones;
                let zone = zones.get_effective_terrain_zone(zone);
                zones.get_effective_zone(unit.surfaces, u8::from(crusher), zone)
            };
            zone2 = terrain_pair(zone2);
            zone1 = terrain_pair(zone1);
        }
        if zone1 != zone2 {
            self.release_cell_info(goal);
            self.release_cell_info(first);
            return None;
        }
        if !self.valid_movement_position(crusher, destination_layer, unit.surfaces, &to) {
            self.is_tunneling = false;
            self.release_cell_info(goal);
            self.release_cell_info(first);
            return None;
        }
        if !self.valid_movement_position(crusher, layer, unit.surfaces, from) {
            self.is_tunneling = true;
        }

        self.start_search(first, Some(goal));
        let search = NeighborSearch {
            unit,
            goal: Some(goal),
            center_in_cell,
            radius,
            start,
            attack_distance: 0,
        };
        let mut parent = first;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            if parent == goal {
                self.is_tunneling = false;
                let path =
                    self.build_actual_path(terrain, units, unit, from, goal, center_in_cell, false);
                self.finish_search(parent);
                return Some(path);
            }
            self.close_cell(parent);
            self.check_change_layers(parent);
            self.examine_neighboring_cells(terrain, units, parent, &search);
        }
        self.is_tunneling = false;
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            self.release_cell_info(parent);
            self.release_cell_info(goal);
        } else {
            self.release_cell_info(goal);
            self.clean_open_and_closed_lists();
        }
        None
    }

    // ---- path building ----------------------------------------------------

    /// `buildActualPath`: the optimized path to `goal` along the parent links.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn build_actual_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        from: &Coord3D,
        goal: CellIdx,
        center_in_cell: bool,
        blocked: bool,
    ) -> Path {
        let mut goal = goal;
        if self.cell_ref(goal).pinched {
            if let Some(parent) = self.parent_cell(goal) {
                if !self.cell_ref(parent).pinched {
                    goal = parent;
                }
            }
        }
        let mut path = Path::new();
        self.prepend_cells(terrain, &mut path, from, goal, center_in_cell);
        let query = PathQuery {
            pathfinder: self,
            units,
            agent: Some(&unit.agent),
        };
        path.optimize(&query, unit.surfaces, blocked);
        path
    }

    /// `prependCells`. A parent chain that reaches a cell without a record
    /// switches to the fixed code.
    pub fn prepend_cells(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        path: &mut Path,
        from: &Coord3D,
        goal: CellIdx,
        center_in_cell: bool,
    ) {
        let mut previous: Option<CellIdx> = None;
        let goal_is_alone = self.parent_cell(goal).is_none();
        let mut cell = goal;
        while self.parent_cell(cell).is_some() {
            let at = self.index_of(cell);
            self.zones.set_passable(at.x, at.y, 1);
            let layer = self.cell_ref(cell).layer;
            let pos = self.adjust_coord_to_cell(terrain, at.x, at.y, center_in_cell, layer);
            if let Some(prev) = previous.filter(|&p| self.index_of(p) == at) {
                // Two layers of one cell make one node, off the ground.
                let layer = if layer == PathfindLayerEnum::Ground {
                    self.cell_ref(prev).layer
                } else {
                    layer
                };
                if let Some(first) = path.first_node() {
                    path.set_layer(first, layer);
                }
            } else {
                let cliff = |idx: CellIdx| self.cell_ref(idx).cell_type == CellType::Cliff;
                let mut can_optimize = true;
                if cliff(cell) {
                    if previous.is_some_and(|p| !cliff(p)) {
                        if let Some(first) = path.first_node() {
                            path.set_can_optimize(first, false);
                        }
                    }
                } else if previous.is_some_and(cliff) {
                    can_optimize = false;
                }
                path.prepend_node(&pos, layer);
                if let Some(first) = path.first_node() {
                    path.set_can_optimize(first, can_optimize);
                }
                if self.map.grid().is_blocked_by_ally(cell) == Some(true) {
                    path.set_blocked_by_ally(1);
                }
                if let Some(prev) = previous {
                    self.map.grid_mut().clear_parent_cell(prev);
                }
                previous = Some(cell);
            }
            let Some(parent) = self.parent_cell(cell) else {
                break;
            };
            cell = parent;
        }

        let mut crashed = false;
        if !self.fixed_pathfinding() && !self.has_info(cell) {
            let Some(prev) = previous else { return };
            cell = prev;
            crashed = true;
        }
        let at = self.index_of(cell);
        self.zones.set_passable(at.x, at.y, 1);
        let layer = self.cell_ref(cell).layer;
        if goal_is_alone {
            let pos = self.adjust_coord_to_cell(terrain, at.x, at.y, center_in_cell, layer);
            path.prepend_node(&pos, layer);
        }
        let differs = path.first_node().is_some_and(|first| {
            let pos = path.node(first).position();
            from.x != pos.x || from.y != pos.y
        });
        if differs {
            path.prepend_node(from, layer);
        }
        if crashed {
            self.map.mode_mut().use_fixed_pathfinding = true;
            self.force_clean_cells();
        }
    }

    // ---- cost only --------------------------------------------------------

    /// `checkPathCost`: the cost of the path between two points, ignoring units,
    /// or [`MAX_PATH_COST`].
    pub fn check_path_cost(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        crusher: bool,
        surfaces: LocomotorSurfaceTypeMask,
        from: &Coord3D,
        raw_to: &Coord3D,
    ) -> Int {
        if !self.is_map_ready {
            return 0;
        }
        let mut count: UnsignedInt = 0;
        let goal_layer = terrain.get_layer_for_destination(raw_to);
        let Some(goal) = self.clipped_cell(goal_layer, raw_to) else {
            return MAX_PATH_COST;
        };
        let (start, _) = self.world_to_cell(from);
        let from_layer = terrain.get_layer_for_destination(from);
        let Some(first) = self.get_cell_at(from_layer, from) else {
            return MAX_PATH_COST;
        };
        let (goal_at, _) = self.world_to_cell(raw_to);
        if !self.allocate_cell_info(goal, goal_at) {
            return MAX_PATH_COST;
        }
        if first != goal && !self.allocate_cell_info(first, start) {
            self.release_cell_info(goal);
            return MAX_PATH_COST;
        }
        if !self.valid_movement_cell(crusher, surfaces, Some(first)) {
            self.release_cell_info(first);
            self.release_cell_info(goal);
            return MAX_PATH_COST;
        }
        self.start_search(first, Some(goal));

        let mut parent = first;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            if !self.fixed_pathfinding() {
                self.close_cell(parent);
            }
            if parent == goal {
                let cost = self
                    .cell_info(parent)
                    .map_or(0, |i| Int::from(i.total_cost));
                self.is_tunneling = false;
                self.clean_open_and_closed_lists();
                if self.fixed_pathfinding() {
                    self.release_cell_info(parent);
                }
                return cost;
            }
            if self.fixed_pathfinding() {
                self.close_cell(parent);
            }
            if count > MAX_CELL_COUNT {
                continue;
            }
            self.check_change_layers(parent);

            let parent_layer = self.cell_ref(parent).layer;
            let parent_at = self.index_of(parent);
            let mut flags = [false; 8];
            for (i, (step_x, step_y)) in NEIGHBORS.into_iter().enumerate() {
                let at = ICoord2D::new(
                    parent_at.x.wrapping_add(step_x),
                    parent_at.y.wrapping_add(step_y),
                );
                let Some(cell) = self.get_cell(parent_layer, at.x, at.y) else {
                    continue;
                };
                let on_list = self.is_listed(cell);
                if i >= FIRST_DIAGONAL && !flags[ADJACENT[i - 4]] && !flags[ADJACENT[i - 3]] {
                    continue;
                }
                if !self.valid_movement_cell(crusher, surfaces, Some(cell)) {
                    continue;
                }
                flags[i] = true;
                if !self.allocate_cell_info(cell, at) {
                    if self.fixed_pathfinding() {
                        self.clean_open_and_closed_lists();
                        self.release_cell_info(parent);
                        self.release_cell_info(goal);
                    }
                    // The reference answers the cell count here.
                    return count as Int;
                }
                count += 1;
                let cost = self.map.grid().cost_so_far(cell, Some(parent));
                self.map.grid_mut().set_blocked_by_ally(cell, false);
                if on_list && self.cost_so_far_of(cell) <= cost {
                    continue;
                }
                let grid = self.map.grid_mut();
                grid.set_parent_cell(cell, parent);
                grid.set_cost_so_far(cell, cost);
                let remaining = self.map.grid().cost_to_goal(cell, goal);
                let total = self.cost_so_far_of(cell).wrapping_add(remaining);
                self.map.grid_mut().set_total_cost(cell, total);
                self.unlist_cell(cell);
                if !self.fixed_pathfinding()
                    && self.open_list.get_head().is_some_and(|h| !self.has_info(h))
                {
                    self.map.mode_mut().use_fixed_pathfinding = true;
                    self.force_clean_cells();
                    return MAX_PATH_COST;
                }
                self.open_cell(cell);
            }
        }
        self.is_tunneling = false;
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            self.release_cell_info(parent);
            self.release_cell_info(goal);
        } else {
            if self.has_info(goal) && !self.is_listed(goal) {
                self.release_cell_info(goal);
            }
            self.clean_open_and_closed_lists();
        }
        MAX_PATH_COST
    }
}

/// [`PathCost`] answered by `checkPathCost`, for the destination searches.
pub struct SearchPathCost<'a> {
    pub terrain: &'a dyn ThingTerrainAccess,
}

impl PathCost for SearchPathCost<'_> {
    fn check_path_cost(
        &mut self,
        pathfinder: &mut Pathfinder,
        unit: &AdjustUnit,
        from: &Coord3D,
        to: &Coord3D,
    ) -> Int {
        let crusher = unit.agent.crusher_level > 0;
        pathfinder.check_path_cost(self.terrain, crusher, unit.surfaces, from, to)
    }
}
