//! `findAttackPath` (`PKT-411`): a path to a cell a unit can fire from.
//!
//! Golden test: `crates/zh_logic/tests/golden_s6_other_searches.rs`.

use zh_common::math::{gm_sqrtf, Coord3D};
use zh_common::types::{Int, ObjectId, Real};

use super::{
    radius_and_center, CheckMovementInfo, HierarchicalSearch, NeighborSearch, PathUnits,
    Pathfinder, SearchUnit,
};
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::Path;
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, PATHFIND_CELL_SIZE, PATHFIND_CELL_SIZE_F,
};

/// `ATTACK_CELL_LIMIT`.
const ATTACK_CELL_LIMIT: Int = 2500;
/// How many cells from the end of a vehicle's path are looked at for others.
const STRIP_CELL_LIMIT: Int = 12;

/// What `findAttackPath` asks of the attacker's weapon. The weapon, the
/// attacker and the victim are the implementor's.
pub trait AttackProbe {
    /// `Weapon::getAttackDistance`.
    fn attack_distance(&mut self) -> Real;
    /// `Weapon::isGoalPosWithinAttackRange` with the attacker at `position`.
    fn in_range(&mut self, position: &Coord3D) -> bool;
    /// `isAttackViewBlockedByObstacle` with the attacker at `position`; see
    /// [`Pathfinder::is_attack_view_blocked`].
    fn view_blocked(&mut self, pathfinder: &Pathfinder, position: &Coord3D) -> bool;
}

/// The object attacked, when the attack is not on a position.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AttackVictim {
    pub position: Coord3D,
    pub significantly_above_terrain: bool,
}

/// The arguments of `findAttackPath`.
#[derive(Debug, Clone, Copy)]
pub struct AttackRequest<'a> {
    pub unit: &'a SearchUnit,
    /// The attacker's position.
    pub position: Coord3D,
    pub from: Coord3D,
    pub victim: Option<AttackVictim>,
    pub victim_position: Coord3D,
}

impl Pathfinder {
    /// `findAttackPath`: a straight step towards the victim if one of the next
    /// few will do, else a search for a cell in range with a clear view.
    pub fn find_attack_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        probe: &mut dyn AttackProbe,
        request: &AttackRequest,
    ) -> Option<Path> {
        if !self.is_map_ready {
            return None;
        }
        let unit = request.unit;
        let agent = &unit.agent;
        let crusher = agent.crusher_level > 0;
        let (radius, center_in_cell) = radius_and_center(Some(agent));
        let aim = request
            .victim
            .map_or(request.victim_position, |v| v.position);

        if let Some(path) = self.step_into_range(units, probe, request, &aim) {
            return Some(path);
        }

        self.zones.clear_passable_flags();
        let search = HierarchicalSearch {
            human: unit.human,
            surfaces: unit.surfaces,
            crusher,
            closest_ok: true,
        };
        if self
            .find_hierarchical_path(terrain, &search, &request.from, &request.victim_position)
            .is_none()
        {
            self.zones.set_all_passable();
        }

        let attack_distance =
            (probe.attack_distance() as Int).wrapping_add(3 * PATHFIND_CELL_SIZE as Int);

        let mut position = request.position;
        if center_in_cell {
            position.x += PATHFIND_CELL_SIZE_F / 2.0;
            position.y += PATHFIND_CELL_SIZE_F / 2.0;
        }
        if !agent.has_ai {
            return None;
        }
        let (start, _) = self.world_to_cell(&position);
        let mut parent = self.clipped_cell(agent.layer, &position)?;
        if !self.allocate_cell_info(parent, start) {
            return None;
        }
        let start_cell = parent;
        // A victim off the map leaves the start's record behind.
        self.map.grid_mut().start_pathfind(parent, None);
        let (victim_cell, _) = self.world_to_cell(&aim);
        let goal = self.get_cell(PathfindLayerEnum::Ground, victim_cell.x, victim_cell.y)?;
        if !self.allocate_cell_info(goal, victim_cell) {
            return None;
        }
        self.start_search(parent, None);

        let check_view = request
            .victim
            .is_none_or(|v| !v.significantly_above_terrain);
        let neighbors = NeighborSearch {
            unit,
            goal: Some(goal),
            center_in_cell,
            radius,
            start,
            attack_distance,
        };
        let mut count: Int = 0;
        while let Some(cell) = self.pop_open() {
            parent = cell;
            let at = self.index_of(parent);
            let layer = self.cell_ref(parent).layer;
            let center = self.adjust_coord_to_cell(terrain, at.x, at.y, center_in_cell, layer);
            if probe.in_range(&center)
                && self.check_destination(
                    units,
                    Some(agent),
                    at.x,
                    at.y,
                    layer,
                    radius,
                    center_in_cell,
                )
            {
                let mut view_blocked = check_view && probe.view_blocked(self, &center);
                if start_cell == parent {
                    // Were it good enough the unit would not be looking.
                    view_blocked = true;
                } else {
                    let dx = center.x - position.x;
                    let dy = center.y - position.y;
                    let near = PATHFIND_CELL_SIZE_F * 0.5;
                    if dx * dx + dy * dy < near * near {
                        view_blocked = true;
                    }
                }
                if !view_blocked {
                    self.close_cell(parent);
                    if unit.vehicle {
                        parent =
                            self.stop_short_of_others(units, unit, parent, radius, center_in_cell);
                    }
                    let path = self.build_actual_path(
                        terrain,
                        units,
                        unit,
                        &request.position,
                        parent,
                        center_in_cell,
                        false,
                    );
                    if self.fixed_pathfinding() {
                        self.clean_open_and_closed_lists();
                        self.release_cell_info(parent);
                        self.release_cell_info(goal);
                    } else {
                        if !self.is_listed(goal) {
                            self.release_cell_info(goal);
                        }
                        self.clean_open_and_closed_lists();
                    }
                    return Some(path);
                }
            }
            // The reference also tracks the nearest usable cell here, which it
            // no longer reads.
            self.close_cell(parent);
            if count < ATTACK_CELL_LIMIT {
                self.check_change_layers(parent);
                count = count.wrapping_add(
                    self.examine_neighboring_cells(terrain, units, parent, &neighbors),
                );
            }
        }
        self.is_tunneling = false;
        if self.fixed_pathfinding() {
            self.clean_open_and_closed_lists();
            self.release_cell_info(parent);
            self.release_cell_info(goal);
        } else {
            if !self.is_listed(goal) {
                self.release_cell_info(goal);
            }
            self.clean_open_and_closed_lists();
        }
        None
    }

    /// The quick check: up to nine half cells straight towards `aim`.
    fn step_into_range(
        &self,
        units: &dyn PathUnits,
        probe: &mut dyn AttackProbe,
        request: &AttackRequest,
        aim: &Coord3D,
    ) -> Option<Path> {
        let unit = request.unit;
        let agent = &unit.agent;
        let crusher = agent.crusher_level > 0;
        let (radius, center_in_cell) = radius_and_center(Some(agent));
        let here = request.position;
        let (dx, dy) = (aim.x - here.x, aim.y - here.y);
        let length = gm_sqrtf(dx * dx + dy * dy);
        let (mut dx, mut dy) = if length != 0.0 {
            (dx / length, dy / length)
        } else {
            (dx, dy)
        };
        dx *= PATHFIND_CELL_SIZE_F;
        dy *= PATHFIND_CELL_SIZE_F;
        for i in 1..10 {
            let mut test = here;
            test.x += dx * i as Real * 0.5;
            test.y += dy * i as Real * 0.5;
            let (at, _) = self.world_to_cell(&test);
            let cell = self.get_cell(agent.layer, at.x, at.y)?;
            if !self.valid_movement_cell(crusher, unit.surfaces, Some(cell))
                || !self.check_destination(
                    units,
                    Some(agent),
                    at.x,
                    at.y,
                    agent.layer,
                    radius,
                    center_in_cell,
                )
            {
                return None;
            }
            if !probe.in_range(&test) || probe.view_blocked(self, &test) {
                continue;
            }
            let mut path = Path::new();
            path.prepend_node(&test, agent.layer);
            path.prepend_node(&here, agent.layer);
            if let Some(first) = path.first_node() {
                let next = path.node(first).next();
                path.set_next_optimized(first, next);
            }
            return Some(path);
        }
        None
    }

    /// A vehicle stops before the cells at the end of its path that others
    /// hold or are headed for, unless the one standing there is idle.
    fn stop_short_of_others(
        &self,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        end: CellIdx,
        radius: Int,
        center_in_cell: bool,
    ) -> CellIdx {
        let mut limit = STRIP_CELL_LIMIT;
        let mut last_blocked = None;
        let mut large = false;
        let mut cursor = Some(end);
        while let Some(cur) = cursor {
            limit -= 1;
            if limit < 0 {
                break;
            }
            let mut info = CheckMovementInfo {
                cell: self.index_of(cur),
                layer: self.cell_ref(cur).layer,
                // A single cell until something is in the way.
                center_in_cell: !large || center_in_cell,
                radius: if large { radius } else { 0 },
                consider_transient: false,
                ..CheckMovementInfo::default()
            };
            let standing = self
                .get_cell(info.layer, info.cell.x, info.cell.y)
                .map_or(ObjectId::INVALID, |c| self.pos_unit(c));
            let idle = standing != ObjectId::INVALID && units.is_idle(standing);
            let can_move = self.check_for_movement(units, Some(&unit.agent), &mut info);
            let allies = !idle && (info.ally_fixed_count != 0 || info.ally_goal);
            if !can_move || info.enemy_fixed || allies {
                last_blocked = Some(cur);
                large = true;
            } else {
                large = false;
            }
            cursor = self.parent_cell(cur);
        }
        last_blocked.map_or(end, |blocked| self.parent_cell(blocked).unwrap_or(blocked))
    }
}
