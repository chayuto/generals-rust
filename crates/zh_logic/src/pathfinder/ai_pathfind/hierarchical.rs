//! The rest of the core search (`PKT-411`): the hierarchical pass over zone
//! blocks and bridges, `findPath`, the ground path for a wide group, and the
//! path request queue.
//!
//! Golden test: `crates/zh_logic/tests/golden_s5_search.rs`.

use zh_common::math::{Coord3D, ICoord2D};
use zh_common::types::{Int, ObjectId, Real, UnsignedInt, UnsignedShort, FALSE};

use super::{PathUnits, Pathfinder, SearchUnit};
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::Path;
use crate::pathfinder::pathfind_cell::CellType;
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, COST_ORTHOGONAL, PATHFIND_CELLS_PER_FRAME, PATHFIND_CELL_SIZE_F,
    PATHFIND_QUEUE_LEN,
};
use crate::pathfinder::pathfind_zone_manager::{UNINITIALIZED_ZONE, ZONE_BLOCK_SIZE};
use crate::rules::combat_types::{LocomotorSurfaceType, LocomotorSurfaceTypeMask};

/// What one hierarchical search asks for.
#[derive(Debug, Clone, Copy)]
pub struct HierarchicalSearch {
    /// Keeps the search inside the playable area.
    pub human: bool,
    pub surfaces: LocomotorSurfaceTypeMask,
    pub crusher: bool,
    /// Answer the path to the block nearest the goal when it is not reached.
    pub closest_ok: bool,
}

/// What the scan of one block edge carries from cell to cell.
struct EdgeScan {
    parent: CellIdx,
    goal: CellIdx,
    parent_zone: UnsignedShort,
    /// The neighbouring zones this edge has already opened.
    examined: Vec<UnsignedShort>,
}

fn ground() -> LocomotorSurfaceTypeMask {
    LocomotorSurfaceType::Ground.as_i32()
}

impl Pathfinder {
    fn block_zone(&self, surfaces: LocomotorSurfaceTypeMask, crusher: bool, at: ICoord2D) -> u16 {
        self.zones
            .get_block_zone(surfaces, u8::from(crusher), at.x, at.y, &self.map)
    }

    fn inside_extent(&self, at: ICoord2D) -> bool {
        let e = &self.extent;
        at.x >= e.lo.x && at.x <= e.hi.x && at.y >= e.lo.y && at.y <= e.hi.y
    }

    /// `costToHierGoal` from `cell` to `via` and to `goal`.
    fn hierarchical_costs(&self, cell: CellIdx, via: CellIdx, goal: CellIdx) -> (u32, u32) {
        let grid = self.map.grid();
        (
            grid.cost_to_hier_goal(cell, via),
            grid.cost_to_hier_goal(cell, goal),
        )
    }

    /// Sets the costs and the parent of a cell and opens it.
    fn open_block_cell(&mut self, cell: CellIdx, parent: CellIdx, so_far: u32, total: u32) {
        let grid = self.map.grid_mut();
        grid.set_cost_so_far(cell, so_far);
        grid.set_total_cost(cell, total);
        grid.set_parent_cell_hierarchical(cell, parent);
        self.open_cell(cell);
    }

    /// [`Self::open_block_cell`] with a total of the stored cost and what
    /// remains to the goal.
    fn reach_block_cell(&mut self, cell: CellIdx, parent: CellIdx, so_far: u32, remaining: u32) {
        let total = u32::from(so_far as UnsignedShort).wrapping_add(remaining);
        self.open_block_cell(cell, parent, so_far, total);
    }

    /// A cell record ran out half way over a bridge. The retail code would
    /// crash later, so it switches to the fixed code.
    fn abandon_bridge_search(&mut self, goal: CellIdx) {
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            self.release_cell_info(goal);
        } else {
            self.map.mode_mut().use_fixed_pathfinding = true;
            self.force_clean_cells();
        }
    }

    /// `processHierarchicalCell`: steps from one cell of a block edge to the
    /// block beyond it.
    fn process_hierarchical_cell(
        &mut self,
        at: ICoord2D,
        step: ICoord2D,
        crusher: bool,
        edge: &mut EdgeScan,
    ) {
        if !self.inside_extent(at) || edge.parent_zone == UNINITIALIZED_ZONE {
            return;
        }
        if edge.parent_zone != self.block_zone(ground(), crusher, at) {
            return;
        }
        let cell = self.map.index(at.x, at.y);
        // Only a cell that already has a record can be stepped from.
        if !self.has_info(cell) || self.is_listed(cell) {
            return;
        }
        if edge.parent_zone != self.cell_ref(cell).zone {
            return;
        }
        let beyond = ICoord2D::new(at.x.wrapping_add(step.x), at.y.wrapping_add(step.y));
        if !self.inside_extent(beyond) {
            return;
        }
        let next = self.map.index(beyond.x, beyond.y);
        if self.is_listed(next) {
            return;
        }
        let effective = |zone| {
            self.zones
                .get_effective_zone(ground(), u8::from(crusher), zone)
        };
        let new_zone = self.block_zone(ground(), crusher, beyond);
        if effective(new_zone) != effective(edge.parent_zone) {
            return;
        }
        if edge.examined.contains(&new_zone) {
            return;
        }
        self.close_cell(cell);
        if !self.allocate_cell_info(next, beyond) {
            return;
        }
        let (mut step_cost, remaining) = self.hierarchical_costs(next, edge.parent, edge.goal);
        if self.cell_ref(next).pinched || self.cell_ref(cell).pinched {
            step_cost = step_cost.wrapping_add(2 * COST_ORTHOGONAL as u32);
        } else {
            edge.examined.push(new_zone);
        }
        let so_far = self.cost_so_far_of(edge.parent).wrapping_add(step_cost);
        self.reach_block_cell(next, edge.parent, so_far, remaining);
    }

    /// `findHierarchicalPath`: a coarse path over zone blocks, which marks the
    /// blocks the cell search may enter.
    pub fn find_hierarchical_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        search: &HierarchicalSearch,
        from: &Coord3D,
        raw_to: &Coord3D,
    ) -> Option<Path> {
        if raw_to.x == 0.0 && raw_to.y == 0.0 {
            return None;
        }
        if !self.is_map_ready {
            return None;
        }
        let mut to = *raw_to;
        let mut clip_from = *from;
        self.clip(&mut clip_from, &mut to);
        self.is_tunneling = false;

        let destination_layer = terrain.get_layer_for_destination(&to);
        let (goal_at, _) = self.world_to_cell(&to);
        let goal = self.get_cell(destination_layer, goal_at.x, goal_at.y)?;
        if !self.allocate_cell_info(goal, goal_at) {
            return None;
        }
        let layer = terrain.get_layer_for_destination(from);
        // A start off the map leaves the goal's record behind.
        let first = self.clipped_cell(layer, &clip_from)?;
        if first != goal {
            let (start, _) = self.world_to_cell(&clip_from);
            if !self.allocate_cell_info(first, start) {
                self.release_cell_info(goal);
                return None;
            }
        }
        let surfaces = search.surfaces;
        let zone_of = |pf: &Self, idx: CellIdx| {
            pf.zones
                .get_effective_zone(surfaces, FALSE, pf.cell_ref(idx).zone)
        };
        if zone_of(self, first) != zone_of(self, goal) {
            self.release_cell_info(goal);
            self.release_cell_info(first);
            return None;
        }
        self.map.grid_mut().start_pathfind(first, Some(goal));
        self.closed_list.reset(None);

        let block_of = |at: ICoord2D| ICoord2D::new(at.x / ZONE_BLOCK_SIZE, at.y / ZONE_BLOCK_SIZE);
        let (goal_zone, goal_block) = if self.cell_ref(goal).layer == PathfindLayerEnum::Ground {
            let at = self.index_of(goal);
            (
                self.block_zone(surfaces, search.crusher, at),
                Some(block_of(at)),
            )
        } else {
            (self.cell_ref(goal).zone, None)
        };

        if self.fixed_pathfinding() {
            self.open_list.reset(None);
            self.open_cell(first);
        } else {
            self.open_list.reset(Some(first));
        }
        // A start on a bridge opens both of its ends.
        let first_layer = self.cell_ref(first).layer;
        if first_layer != PathfindLayerEnum::Ground {
            let layer = &self.layers[first_layer as usize];
            let (near, far) = (layer.start_cell_index(), layer.end_cell_index());
            let ground_layer = PathfindLayerEnum::Ground;
            let far_cell = self.get_cell(ground_layer, far.x, far.y);
            let near_cell = self.get_cell(ground_layer, near.x, near.y);
            if let (Some(far_cell), Some(near_cell)) = (far_cell, near_cell) {
                self.map
                    .grid_mut()
                    .remove_from_open_list(first, &mut self.open_list);
                self.close_cell(first);
                if !self.allocate_cell_info(near_cell, near) {
                    self.abandon_bridge_search(goal);
                    return None;
                }
                self.map
                    .grid_mut()
                    .set_parent_cell_hierarchical(near_cell, first);
                let (cost, remaining) = self.hierarchical_costs(near_cell, first, goal);
                self.open_block_cell(near_cell, first, cost, remaining);
                if !self.allocate_cell_info(far_cell, far) {
                    self.abandon_bridge_search(goal);
                    return None;
                }
                let (cost, remaining) = self.hierarchical_costs(far_cell, first, goal);
                self.open_block_cell(far_cell, first, cost, remaining);
            }
        }

        let mut closest: Option<CellIdx> = None;
        let mut closest_square: Real = 1_000_000.0 * 1_000_000.0;
        let mut parent = first;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            let at = self.index_of(parent);
            let parent_zone = if self.cell_ref(parent).layer == PathfindLayerEnum::Ground {
                self.block_zone(surfaces, search.crusher, at)
            } else {
                self.cell_ref(parent).zone
            };
            let block = block_of(at);
            let mut reached =
                parent_zone == goal_zone && goal_block.is_none_or(|goal| goal == block);
            if !reached && self.zones.interacts_with_bridge(at.x, at.y) != FALSE {
                match self.cross_bridges(search, parent, goal, parent_zone, goal_zone, block) {
                    Some(found) => reached = found,
                    None => return None,
                }
            }
            if reached {
                if parent != goal {
                    self.map
                        .grid_mut()
                        .set_parent_cell_hierarchical(goal, parent);
                }
                self.is_tunneling = false;
                let path = self.build_hierarchical_path(terrain, from, goal);
                if self.fixed_pathfinding() {
                    self.clean_open_and_closed_lists();
                    self.release_cell_info(parent);
                    self.release_cell_info(goal);
                } else {
                    if self.has_info(goal) && !self.is_listed(goal) {
                        self.release_cell_info(goal);
                    }
                    self.release_cell_info(parent);
                    self.clean_open_and_closed_lists();
                }
                return Some(path);
            }
            let goal_at = self.index_of(goal);
            let at = self.index_of(parent);
            let dx = goal_at.x.wrapping_sub(at.x).wrapping_abs() as Real;
            let dy = goal_at.y.wrapping_sub(at.y).wrapping_abs() as Real;
            let square = dx * dx + dy * dy;
            if square < closest_square {
                closest = Some(parent);
                closest_square = square;
            }
            self.close_cell(parent);
            self.scan_block_edges(search, parent, goal, parent_zone, block);
        }

        if let Some(closest) = closest.filter(|_| search.closest_ok) {
            self.is_tunneling = false;
            let path = self.build_hierarchical_path(terrain, from, closest);
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
            return Some(path);
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

    /// The bridges with an end in `parent`'s block. Answers whether one of them
    /// is in the goal's zone, or `None` when the search had to be abandoned.
    fn cross_bridges(
        &mut self,
        search: &HierarchicalSearch,
        parent: CellIdx,
        goal: CellIdx,
        parent_zone: u16,
        goal_zone: u16,
        block: ICoord2D,
    ) -> Option<bool> {
        let in_block =
            |at: ICoord2D| at.x / ZONE_BLOCK_SIZE == block.x && at.y / ZONE_BLOCK_SIZE == block.y;
        for i in 0..self.layers.len() {
            let layer = &self.layers[i];
            if layer.is_unused() || layer.is_destroyed() {
                continue;
            }
            let layer_zone = layer.zone();
            let (mut near, mut far) = (layer.start_cell_index(), layer.end_cell_index());
            if !in_block(near) {
                std::mem::swap(&mut near, &mut far);
            }
            if near.x < 0 || near.y < 0 || far.x < 0 || far.y < 0 || !in_block(near) {
                continue;
            }
            if self.block_zone(search.surfaces, search.crusher, near) != parent_zone {
                continue;
            }
            if layer_zone == Int::from(goal_zone) {
                return Some(true);
            }
            let Some(far_cell) = self.get_cell(PathfindLayerEnum::Ground, far.x, far.y) else {
                continue;
            };
            if self.is_listed(far_cell) {
                continue;
            }
            let Some(near_cell) = self.get_cell(PathfindLayerEnum::Ground, near.x, near.y) else {
                continue;
            };
            if near_cell != parent {
                if !self.allocate_cell_info(near_cell, near) {
                    self.abandon_bridge_search(goal);
                    return None;
                }
                self.map
                    .grid_mut()
                    .set_parent_cell_hierarchical(near_cell, parent);
                if !self.is_listed(near_cell) {
                    self.close_cell(near_cell);
                }
            }
            if !self.allocate_cell_info(far_cell, far) {
                self.abandon_bridge_search(goal);
                return None;
            }
            self.map
                .grid_mut()
                .set_parent_cell_hierarchical(far_cell, near_cell);
            let (cost, remaining) = self.hierarchical_costs(far_cell, near_cell, goal);
            let so_far = self.cost_so_far_of(near_cell).wrapping_add(cost);
            self.reach_block_cell(far_cell, near_cell, so_far, remaining);
        }
        Some(false)
    }

    /// The four edges of `parent`'s block, each walked outwards from its middle.
    fn scan_block_edges(
        &mut self,
        search: &HierarchicalSearch,
        parent: CellIdx,
        goal: CellIdx,
        parent_zone: u16,
        block: ICoord2D,
    ) {
        let blocks = self.zones.get_extent();
        let origin = ICoord2D::new(block.x * ZONE_BLOCK_SIZE, block.y * ZONE_BLOCK_SIZE);
        let middle = ZONE_BLOCK_SIZE / 2;
        let last = ZONE_BLOCK_SIZE - 1;
        // Whether there is a block that way, the edge's coordinate and the step.
        let edges = [
            (block.x > 0, 0, ICoord2D::new(-1, 0)),
            (block.x < blocks.x - 1, last, ICoord2D::new(1, 0)),
            (block.y > 0, 0, ICoord2D::new(0, -1)),
            (block.y < blocks.y - 1, last, ICoord2D::new(0, 1)),
        ];
        for (present, side, step) in edges {
            if !present {
                continue;
            }
            let mut edge = EdgeScan {
                parent,
                goal,
                parent_zone,
                examined: Vec::new(),
            };
            for i in 1..=ZONE_BLOCK_SIZE {
                let mut offset = i >> 1;
                if i & 1 != 0 {
                    offset = -offset;
                }
                let at = if step.x != 0 {
                    ICoord2D::new(origin.x + side, origin.y + middle + offset)
                } else {
                    ICoord2D::new(origin.x + middle + offset, origin.y + side)
                };
                let Some(cell) = self.get_cell(PathfindLayerEnum::Ground, at.x, at.y) else {
                    continue;
                };
                if self.is_listed(cell)
                    && parent_zone == self.block_zone(search.surfaces, search.crusher, at)
                {
                    break;
                }
                if search.human && self.check_cell_outside_extents(at) {
                    continue;
                }
                self.process_hierarchical_cell(at, step, search.crusher, &mut edge);
            }
        }
    }

    /// `buildHierarchicalPath`: the path over the parent links, with the blocks
    /// around its start made passable.
    fn build_hierarchical_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        from: &Coord3D,
        goal: CellIdx,
    ) -> Path {
        let mut path = Path::new();
        self.prepend_cells(terrain, &mut path, from, goal, true);
        if let Some(first) = path.first_node() {
            let pos = *path.node(first).position();
            let reach = ZONE_BLOCK_SIZE as Real * PATHFIND_CELL_SIZE_F;
            let corner = |sign: Real| {
                let mut corner = pos;
                corner.x += sign * reach;
                corner.y += sign * reach;
                corner
            };
            let (lo, _) = self.world_to_cell(&corner(-1.0));
            let (hi, _) = self.world_to_cell(&corner(1.0));
            for i in lo.x..=hi.x {
                for j in lo.y..=hi.y {
                    self.zones.set_passable(i, j, 1);
                }
            }
        }
        path
    }

    // ---- findPath ---------------------------------------------------------

    /// The hierarchical pass of `findPath` and `findGroundPath`: afterwards the
    /// passable flags are those along its path, or all set if it found none.
    fn mark_passable_blocks(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        search: &HierarchicalSearch,
        from: &Coord3D,
        to: &Coord3D,
    ) {
        self.zones.clear_passable_flags();
        if self
            .find_hierarchical_path(terrain, search, from, to)
            .is_none()
        {
            self.zones.set_all_passable();
        }
    }

    /// `findPath`.
    pub fn find_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        from: &Coord3D,
        raw_to: &Coord3D,
    ) -> Option<Path> {
        if !self.quick_path_exists(terrain, unit.surfaces, from, raw_to) {
            return None;
        }
        let search = HierarchicalSearch {
            human: unit.human,
            surfaces: unit.surfaces,
            crusher: false,
            closest_ok: false,
        };
        self.mark_passable_blocks(terrain, &search, from, raw_to);
        self.internal_find_path(terrain, units, unit, from, raw_to)
    }

    /// `slowDoesPathExist`: runs the whole search, ignoring `ignore`.
    pub fn slow_does_path_exist(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        from: &Coord3D,
        to: &Coord3D,
        ignore: ObjectId,
    ) -> bool {
        if !unit.agent.has_ai {
            return false;
        }
        self.ignore_obstacle_id = ignore;
        let found = self.find_path(terrain, units, unit, from, to).is_some();
        self.ignore_obstacle_id = ObjectId::INVALID;
        found
    }

    // ---- ground path for a group ------------------------------------------

    /// `groundCellsCallback`: one step of the straight line towards the goal.
    /// False ends the walk.
    #[allow(clippy::too_many_arguments)]
    fn ground_line_cell(
        &mut self,
        units: &dyn PathUnits,
        crusher: bool,
        diameter: Int,
        goal: CellIdx,
        from: CellIdx,
        to: CellIdx,
        at: ICoord2D,
    ) -> bool {
        if self.is_listed(to) {
            return false;
        }
        let layer = self.cell_ref(to).layer;
        if self.clear_cell_for_diameter(units, crusher, at.x, at.y, layer, diameter) != diameter {
            return false;
        }
        if !self.allocate_cell_info(to, at) {
            return false;
        }
        let cost =
            (self.cost_so_far_of(from) as Real + 0.5 * COST_ORTHOGONAL as Real) as UnsignedInt;
        self.map.grid_mut().set_blocked_by_ally(to, false);
        let remaining = self.map.grid().cost_to_goal(to, goal);
        let grid = self.map.grid_mut();
        grid.set_cost_so_far(to, cost);
        grid.set_parent_cell(to, from);
        let total = self.cost_so_far_of(to).wrapping_add(remaining);
        self.map.grid_mut().set_total_cost(to, total);
        self.open_cell(to);
        true
    }

    /// The cost of stepping into a cell, with a penalty for each cell it is
    /// narrower than the group.
    fn ground_step_cost(
        &mut self,
        cell: CellIdx,
        parent: CellIdx,
        clear: Int,
        diameter: Int,
    ) -> UnsignedInt {
        let mut cost = self.map.grid().cost_so_far(cell, Some(parent));
        if clear < diameter {
            let short = diameter.wrapping_sub(clear).wrapping_mul(COST_ORTHOGONAL);
            cost = (cost as Real + 0.6 * short as Real) as UnsignedInt;
        }
        self.map.grid_mut().set_blocked_by_ally(cell, false);
        cost
    }

    /// The cell a ground path of `diameter` cells ends in: `cell` itself, or the
    /// first near it that is wide enough.
    fn wide_goal_cell(
        &self,
        units: &dyn PathUnits,
        crusher: bool,
        layer: PathfindLayerEnum,
        mut cell: ICoord2D,
        diameter: Int,
    ) -> Option<ICoord2D> {
        const MAX_OFFSET: Int = 8;
        let wide = |at: ICoord2D| {
            self.clear_cell_for_diameter(units, crusher, at.x, at.y, layer, diameter) == diameter
        };
        if wide(cell) {
            return Some(cell);
        }
        // Eight moves that walk the corners of a square around the cell.
        const MOVES: [(Int, Int); 8] = [
            (1, 0),
            (0, 1),
            (-1, 0),
            (-1, 0),
            (0, -1),
            (0, -1),
            (1, 0),
            (1, 0),
        ];
        for offset in 1..MAX_OFFSET {
            let centre = cell;
            for (x, y) in MOVES {
                cell.x += x * offset;
                cell.y += y * offset;
                if wide(cell) {
                    return Some(cell);
                }
            }
            cell = centre;
        }
        None
    }

    /// `findGroundPath`: a path `path_diameter` cells wide, for a group.
    pub fn find_ground_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        from: &Coord3D,
        raw_to: &Coord3D,
        path_diameter: Int,
        crusher: bool,
    ) -> Option<Path> {
        let search = HierarchicalSearch {
            human: true,
            surfaces: ground(),
            crusher: false,
            closest_ok: false,
        };
        self.mark_passable_blocks(terrain, &search, from, raw_to);
        if raw_to.x == 0.0 && raw_to.y == 0.0 {
            return None;
        }
        if !self.is_map_ready {
            return None;
        }
        let mut to = *raw_to;
        let mut clip_from = *from;
        self.clip(&mut clip_from, &mut to);
        self.is_tunneling = false;

        let destination_layer = terrain.get_layer_for_destination(&to);
        let (cell, _) = self.world_to_cell(&to);
        let goal_at =
            self.wide_goal_cell(units, crusher, destination_layer, cell, path_diameter)?;
        let goal = self.get_cell(destination_layer, goal_at.x, goal_at.y)?;
        if !self.allocate_cell_info(goal, goal_at) {
            return None;
        }
        let layer = terrain.get_layer_for_destination(from);
        let Some(first) = self.clipped_cell(layer, &clip_from) else {
            if self.fixed_pathfinding() {
                self.release_cell_info(goal);
            }
            return None;
        };
        if first != goal {
            let (start, _) = self.world_to_cell(&clip_from);
            if !self.allocate_cell_info(first, start) {
                self.release_cell_info(goal);
                return None;
            }
        }
        let zone_of = |pf: &Self, idx: CellIdx| {
            pf.zones
                .get_effective_zone(ground(), FALSE, pf.cell_ref(idx).zone)
        };
        if zone_of(self, first) != zone_of(self, goal) {
            self.release_cell_info(goal);
            self.release_cell_info(first);
            return None;
        }
        self.start_search(first, Some(goal));

        // Retail kept the cost of the last neighbour from step to step.
        #[cfg(feature = "retail-crc")]
        let mut cost: UnsignedInt = 0;
        let mut parent = first;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            if parent == goal {
                self.is_tunneling = false;
                let mut path = Path::new();
                self.prepend_cells(terrain, &mut path, from, goal, false);
                path.optimize_ground_path(self, units, crusher, path_diameter);
                self.finish_search(parent);
                return Some(path);
            }
            self.close_cell(parent);
            self.check_change_layers(parent);

            let parent_layer = self.cell_ref(parent).layer;
            let parent_at = self.index_of(parent);
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
                if !self.ground_line_cell(units, crusher, path_diameter, goal, from, to, at) {
                    break;
                }
            }

            let mut flags = [false; 8];
            for (i, (step_x, step_y)) in super::NEIGHBORS.into_iter().enumerate() {
                let at = ICoord2D::new(
                    parent_at.x.wrapping_add(step_x),
                    parent_at.y.wrapping_add(step_y),
                );
                let Some(cell) = self.get_cell(parent_layer, at.x, at.y) else {
                    continue;
                };
                if self.cell_ref(cell).layer == PathfindLayerEnum::Ground
                    && self.zones.is_passable(at.x, at.y) == FALSE
                {
                    let near = [(3, 3), (-3, 3), (3, -3), (-3, -3)]
                        .iter()
                        .any(|&(x, y)| self.zones.clip_is_passable(at.x + x, at.y + y) != FALSE);
                    if !near {
                        continue;
                    }
                }
                let on_list = self.is_listed(cell);
                let mut clear = 0;
                if cell != goal {
                    if i >= super::FIRST_DIAGONAL
                        && !flags[super::ADJACENT[i - 4]]
                        && !flags[super::ADJACENT[i - 3]]
                    {
                        continue;
                    }
                    let layer = self.cell_ref(cell).layer;
                    clear = self.clear_cell_for_diameter(
                        units,
                        crusher,
                        at.x,
                        at.y,
                        layer,
                        path_diameter,
                    );
                    if self.cell_ref(cell).cell_type != CellType::Clear {
                        continue;
                    }
                    if self.cell_ref(cell).pinched {
                        continue;
                    }
                    flags[i] = true;
                    if !self.allocate_cell_info(cell, at) {
                        continue;
                    }
                }
                #[cfg(not(feature = "retail-crc"))]
                let cost = self.ground_step_cost(cell, parent, clear, path_diameter);
                // The retail layout leaves the goal the previous neighbour's cost.
                #[cfg(feature = "retail-crc")]
                if cell != goal {
                    cost = self.ground_step_cost(cell, parent, clear, path_diameter);
                }
                let remaining = self.map.grid().cost_to_goal(cell, goal);
                if on_list && self.cost_so_far_of(cell) <= cost {
                    continue;
                }
                let grid = self.map.grid_mut();
                grid.set_cost_so_far(cell, cost);
                grid.set_parent_cell(cell, parent);
                let total = self.cost_so_far_of(cell).wrapping_add(remaining);
                self.map.grid_mut().set_total_cost(cell, total);
                self.unlist_cell(cell);
                self.open_cell(cell);
            }
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

    // ---- the request queue ------------------------------------------------

    /// `queueForPath`: false when the queue is full. A unit already waiting is
    /// not queued twice.
    pub fn queue_for_path(&mut self, id: ObjectId) -> bool {
        let next = |slot: Int| {
            if slot + 1 >= PATHFIND_QUEUE_LEN {
                0
            } else {
                slot + 1
            }
        };
        let mut slot = self.queue_head;
        while slot != self.queue_tail {
            if self.queued_requests[slot as usize] == id {
                return true;
            }
            slot = next(slot);
        }
        let after = next(self.queue_tail);
        if after == self.queue_head {
            return false;
        }
        self.queued_requests[self.queue_tail as usize] = id;
        self.queue_tail = after;
        true
    }

    /// The loop of `processPathfindQueue`, after [`Pathfinder::begin_path_queue`]:
    /// the unit to run `doPathfind` for, if the queue has one and the frame's
    /// cell budget is not spent. Call [`Pathfinder::finish_path_request`] once
    /// it has run.
    pub fn next_path_request(&mut self) -> Option<ObjectId> {
        if self.cumulative_cells_allocated >= PATHFIND_CELLS_PER_FRAME as Int
            || self.queue_head == self.queue_tail
        {
            return None;
        }
        let slot = &mut self.queued_requests[self.queue_head as usize];
        let id = *slot;
        *slot = ObjectId::INVALID;
        Some(id)
    }

    /// Takes the served request off the queue.
    pub fn finish_path_request(&mut self) {
        self.queue_head += 1;
        if self.queue_head >= PATHFIND_QUEUE_LEN {
            self.queue_head = 0;
        }
    }
}
