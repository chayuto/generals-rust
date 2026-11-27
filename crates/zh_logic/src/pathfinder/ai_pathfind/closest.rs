//! `findClosestPath` and `patchPath` (`PKT-411`): the searches ground movement
//! falls back to, towards the nearest cell a unit can stop on and from a
//! blocked unit back onto its old path.
//!
//! Golden test: `crates/zh_logic/tests/golden_s6_other_searches.rs`.

use zh_common::math::{Coord3D, ICoord2D};
use zh_common::types::{Int, ObjectId, Real};

use super::{
    radius_and_center, CheckMovementInfo, HierarchicalSearch, NeighborSearch, PathUnits,
    Pathfinder, SearchUnit,
};
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::{Path, PathQuery};
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, COST_TO_DISTANCE_FACTOR_SQR, PATHFIND_CELL_SIZE_F,
};

/// `CELL_LIMIT` of `patchPath`.
const PATCH_CELL_LIMIT: Int = 2000;

impl Pathfinder {
    /// The records and lists a search leaves are dropped: the retail code
    /// releases `cells` before it empties the lists, the fixed code releases
    /// `fixed` after.
    pub(super) fn end_search(&mut self, retail: &[CellIdx], fixed: &[CellIdx]) {
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            for &cell in fixed {
                self.release_cell_info(cell);
            }
        } else {
            for &cell in retail {
                self.release_cell_info(cell);
            }
            self.clean_open_and_closed_lists();
        }
    }

    /// `findClosestPath`: the path to the goal, or to the cell nearest to it
    /// where the unit can stop. Then `raw_to` takes that cell's centre. Cost
    /// counts against nearness by `path_cost_multiplier`.
    ///
    /// The ignored obstacle may change to the airfield under the goal, and
    /// stays so.
    #[allow(clippy::too_many_arguments)]
    pub fn find_closest_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        from: &Coord3D,
        raw_to: &mut Coord3D,
        blocked: bool,
        path_cost_multiplier: Real,
    ) -> Option<Path> {
        let agent = &unit.agent;
        if unit.surfaces == 0 || !self.is_map_ready {
            return None;
        }
        self.is_tunneling = false;
        let (radius, center_in_cell) = radius_and_center(Some(agent));

        let mut to = *raw_to;
        if !center_in_cell {
            to.x += PATHFIND_CELL_SIZE_F / 2.0;
            to.y += PATHFIND_CELL_SIZE_F / 2.0;
        }
        let crusher = agent.crusher_level > 0;
        let mut clip_from = *from;
        self.clip(&mut clip_from, &mut to);

        let destination_layer = terrain.get_layer_for_destination(&to);
        let goal = self.clipped_cell(destination_layer, &to)?;
        if self.cell_ref(goal).zone == 0 && destination_layer == PathfindLayerEnum::Wall {
            return None;
        }
        let goal_on_obstacle = self.goal_on_ignored_obstacle(units, goal);

        let (start, _) = self.world_to_cell(from);
        let mut parent = self.clipped_cell(agent.layer, &clip_from)?;
        if !self.valid_movement_cell(crusher, unit.surfaces, Some(parent)) {
            self.is_tunneling = true;
        }
        let mut info = CheckMovementInfo {
            cell: start,
            layer: agent.layer,
            center_in_cell,
            radius,
            consider_transient: blocked,
            ..CheckMovementInfo::default()
        };
        if !self.check_for_movement(units, Some(agent), &mut info) || info.enemy_fixed {
            self.is_tunneling = true;
        }

        let mut got_hierarchical_path = false;
        if self.is_tunneling {
            self.zones.set_all_passable();
        } else {
            self.zones.clear_passable_flags();
            let search = HierarchicalSearch {
                human: unit.human,
                surfaces: unit.surfaces,
                crusher: false,
                closest_ok: true,
            };
            if self
                .find_hierarchical_path(terrain, &search, from, raw_to)
                .is_some()
            {
                got_hierarchical_path = true;
            } else {
                self.zones.set_all_passable();
            }
        }
        let started_stuck = self.is_tunneling;

        let (goal_at, _) = self.world_to_cell(&to);
        if !self.allocate_cell_info(goal, goal_at) {
            return None;
        }
        if parent != goal {
            let (at, _) = self.world_to_cell(&clip_from);
            if !self.allocate_cell_info(parent, at) {
                if self.fixed_pathfinding() {
                    self.release_cell_info(goal);
                }
                return None;
            }
        }
        self.start_search(parent, Some(goal));
        let search = NeighborSearch {
            unit,
            goal: Some(goal),
            center_in_cell,
            radius,
            start,
            attack_distance: 0,
        };

        let mut closest: Option<CellIdx> = None;
        let mut closest_square = Real::MAX;
        let mut closest_screen_square = Real::MAX;
        let mut count: Int = 0;
        let mut found_goal = false;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            if parent == goal {
                if !goal_on_obstacle && closest.is_some() && !unit.can_path_through_units {
                    let at = self.index_of(parent);
                    let layer = self.cell_ref(parent).layer;
                    if !self.check_destination(
                        units,
                        Some(agent),
                        at.x,
                        at.y,
                        layer,
                        radius,
                        center_in_cell,
                    ) {
                        // The goal is on neither list now; look for a closer
                        // cell among those already open.
                        found_goal = true;
                        continue;
                    }
                }
                self.is_tunneling = false;
                let path = self.build_actual_path(
                    terrain,
                    units,
                    unit,
                    from,
                    goal,
                    center_in_cell,
                    blocked,
                );
                self.end_search(&[parent, goal], &[parent, goal]);
                return Some(path);
            }
            self.close_cell(parent);
            let at = self.index_of(parent);
            let layer = self.cell_ref(parent).layer;
            if !self.is_tunneling
                && self.check_destination(
                    units,
                    Some(agent),
                    at.x,
                    at.y,
                    layer,
                    radius,
                    center_in_cell,
                )
                && (!started_stuck
                    || self.valid_movement_cell(crusher, unit.surfaces, Some(parent)))
            {
                let mut square = self.cell_square_to(goal, parent);
                if square < closest_screen_square {
                    closest_screen_square = square;
                }
                let cost = self.cost_so_far_of(parent) as Real;
                square += (cost * (cost * COST_TO_DISTANCE_FACTOR_SQR)) * path_cost_multiplier;
                if square < closest_square {
                    closest = Some(parent);
                    closest_square = square;
                }
            }

            let square = self.cell_square_to(goal, parent);
            // Twice as far as the closest cell found: give up on this branch.
            if square > closest_screen_square * 4.0
                && (!got_hierarchical_path
                    || count > 2000
                    || closest_screen_square < 10.0 * 10.0 * PATHFIND_CELL_SIZE_F)
            {
                continue;
            }
            if !found_goal {
                self.check_change_layers(parent);
                count = count
                    .wrapping_add(self.examine_neighboring_cells(terrain, units, parent, &search));
            }
        }

        if let Some(closest) = closest {
            self.is_tunneling = false;
            let at = self.index_of(closest);
            raw_to.x = at.x as Real * PATHFIND_CELL_SIZE_F + PATHFIND_CELL_SIZE_F / 2.0;
            raw_to.y = at.y as Real * PATHFIND_CELL_SIZE_F + PATHFIND_CELL_SIZE_F / 2.0;
            let path = self.build_actual_path(
                terrain,
                units,
                unit,
                from,
                closest,
                center_in_cell,
                blocked,
            );
            self.end_search(&[goal], &[parent, goal]);
            return Some(path);
        }
        self.is_tunneling = false;
        self.end_search(&[goal], &[parent, goal]);
        None
    }

    /// The squared distance in cells between two cells with records.
    fn cell_square_to(&self, goal: CellIdx, cell: CellIdx) -> Real {
        let (goal, cell) = (self.index_of(goal), self.index_of(cell));
        let dx = goal.x.wrapping_sub(cell.x).wrapping_abs() as Real;
        let dy = goal.y.wrapping_sub(cell.y).wrapping_abs() as Real;
        dx * dx + dy * dy
    }

    /// Whether the goal is on the structure the unit ignores. An airfield under
    /// the goal that shares its cells becomes the ignored obstacle.
    fn goal_on_ignored_obstacle(&mut self, units: &dyn PathUnits, goal: CellIdx) -> bool {
        if self.ignore_obstacle_id == ObjectId::INVALID {
            return false;
        }
        let Some(ignored) = units.goal_object(self.ignore_obstacle_id) else {
            return false;
        };
        let Some(ignore_cell) = self.clipped_cell(ignored.layer, &ignored.position) else {
            return false;
        };
        let obstacle_of = |idx: CellIdx| {
            self.cell_ref(idx)
                .obstacle_id(self.map.pool(), self.map.mode())
        };
        let under_goal = obstacle_of(goal);
        if under_goal != obstacle_of(ignore_cell) || under_goal == ObjectId::INVALID {
            return false;
        }
        if units.goal_object(under_goal).is_some_and(|o| o.airfield) {
            self.ignore_obstacle_id = under_goal;
            return true;
        }
        self.ignore_obstacle_id == under_goal
    }

    /// `patchPath`: a path from the unit at `position` back onto `original`,
    /// keeping its tail. Cells of the tail that are occupied are not rejoined
    /// across; `blocked` counts moving units next to the start.
    #[allow(clippy::too_many_arguments)]
    pub fn patch_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        position: &Coord3D,
        original: &Path,
        blocked: bool,
    ) -> Option<Path> {
        let agent = &unit.agent;
        let (radius, center_in_cell) = radius_and_center(Some(agent));
        self.zones.set_all_passable();

        let start = self.offset_start_cell(position, center_in_cell);
        let mut parent = self.clipped_cell(agent.layer, position)?;
        if !agent.has_ai {
            return None;
        }
        self.is_tunneling = false;
        if !self.allocate_cell_info(parent, start) {
            return None;
        }
        self.start_search(parent, None);

        // The stretch at the end of the old path that is free to rejoin.
        let last = original.last_node()?;
        let first = original.first_node()?;
        let square_to = |pos: &Coord3D| {
            (pos.x - position.x) * (pos.x - position.x)
                + (pos.y - position.y) * (pos.y - position.y)
        };
        let mut goal_pos = *original.node(last).position();
        let mut goal_square = square_to(&goal_pos);
        let mut stop = last;
        while stop != first {
            let node = original.node(stop);
            let (cell, _) = self.world_to_cell(node.position());
            let mut info = CheckMovementInfo {
                cell,
                layer: node.layer(),
                center_in_cell,
                radius,
                consider_transient: blocked,
                ..CheckMovementInfo::default()
            };
            let dx = cell.x.wrapping_sub(start.x);
            let dy = cell.y.wrapping_sub(start.y);
            if !(-2..=2).contains(&dx) || !(-2..=2).contains(&dy) {
                info.consider_transient = false;
            }
            if !self.check_for_movement(units, Some(agent), &mut info)
                || info.ally_fixed_count != 0
                || info.enemy_fixed
            {
                break;
            }
            let square = square_to(node.position());
            if square < goal_square {
                goal_pos = *node.position();
                goal_square = square;
            }
            let Some(previous) = node.previous() else {
                break;
            };
            stop = previous;
        }
        if stop == last {
            // The fixed code leaves the cell on the open list.
            if self.fixed_pathfinding() {
                self.release_cell_info(parent);
            } else {
                self.clean_open_and_closed_lists();
            }
            return None;
        }
        // Only for cost estimates; a goal off the map would fault the reference.
        let candidate = self.get_cell_at(PathfindLayerEnum::Ground, &goal_pos)?;
        let (candidate_at, _) = self.world_to_cell(&goal_pos);
        if !self.allocate_cell_info(candidate, candidate_at) {
            if self.fixed_pathfinding() {
                self.release_cell_info(parent);
            }
            return None;
        }

        let search = NeighborSearch {
            unit,
            goal: None,
            center_in_cell,
            radius,
            start,
            attack_distance: 0,
        };
        let mut count: Int = 0;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            let at = self.index_of(parent);
            let layer = self.cell_ref(parent).layer;
            let center = self.adjust_coord_to_cell(terrain, at.x, at.y, center_in_cell, layer);
            let mut node = last;
            let mut matched = None;
            while node != stop {
                let pos = original.node(node).position();
                if center.x == pos.x && center.y == pos.y {
                    matched = Some(node);
                    break;
                }
                let Some(previous) = original.node(node).previous() else {
                    break;
                };
                node = previous;
            }
            if let Some(matched) = matched {
                self.is_tunneling = false;
                let mut path = Path::new();
                let mut node = Some(last);
                while let Some(idx) = node.filter(|&idx| idx != matched) {
                    let old = original.node(idx);
                    path.prepend_node(old.position(), old.layer());
                    node = old.previous();
                }
                self.prepend_cells(terrain, &mut path, position, parent, center_in_cell);
                let query = PathQuery {
                    pathfinder: self,
                    units,
                    agent: Some(agent),
                };
                path.optimize(&query, unit.surfaces, blocked);
                if self.fixed_pathfinding() {
                    self.clean_open_and_closed_lists();
                    self.release_cell_info(parent);
                    self.release_cell_info(candidate);
                } else {
                    self.release_cell_info(parent);
                    self.clean_open_and_closed_lists();
                    self.release_cell_info(candidate);
                }
                return Some(path);
            }
            self.close_cell(parent);
            if count < PATCH_CELL_LIMIT {
                self.check_change_layers(parent);
                count = count
                    .wrapping_add(self.examine_neighboring_cells(terrain, units, parent, &search));
            }
        }
        self.is_tunneling = false;
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            self.release_cell_info(parent);
            self.release_cell_info(candidate);
        } else {
            if !self.is_listed(candidate) {
                self.release_cell_info(candidate);
            }
            self.clean_open_and_closed_lists();
        }
        None
    }

    /// The start cell of `patchPath` and `getMoveAwayFromPath`. For a unit not
    /// centred in a cell the reference adds half a cell to `x` twice and
    /// nothing to `y`.
    pub(super) fn offset_start_cell(&self, position: &Coord3D, center_in_cell: bool) -> ICoord2D {
        let mut pos = *position;
        if !center_in_cell {
            pos.x += PATHFIND_CELL_SIZE_F * 0.5;
            pos.x += PATHFIND_CELL_SIZE_F * 0.5;
        }
        self.world_to_cell(&pos).0
    }
}
