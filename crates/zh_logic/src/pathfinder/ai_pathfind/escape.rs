//! The searches that get a unit out of the way (`PKT-411`): `findSafePath`,
//! away from what repels it, and `getMoveAwayFromPath`, off another unit's
//! path.
//!
//! Golden test: `crates/zh_logic/tests/golden_s6_other_searches.rs`.

use zh_common::math::{Coord2D, Coord3D, Region2D};
use zh_common::types::{Int, Real};

use super::{
    radius_and_center, CheckMovementInfo, NeighborSearch, PathAgent, PathUnits, Pathfinder,
    SearchUnit,
};
use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::path::Path;
use crate::pathfinder::pathfind_constants::{MAX_SAFE_PATH_CELL_COUNT, PATHFIND_CELL_SIZE_F};
use crate::terrain::bridge::line_in_region;

/// Whether a segment of the optimised path crosses the box.
fn crosses(path: &Path, bounds: &Region2D) -> bool {
    let mut cursor = path.first_node();
    while let Some(node) = cursor {
        let Some(next) = path.node(node).next_optimized() else {
            return false;
        };
        let end = |idx| {
            let pos = path.node(idx).position();
            Coord2D { x: pos.x, y: pos.y }
        };
        if line_in_region(&end(node), &end(next), bounds) {
            return true;
        }
        cursor = Some(next);
    }
    false
}

impl Pathfinder {
    /// `findSafePath`: the path to the first cell farther than `radius` from
    /// both repulsors, or a good enough one once the search is long or done.
    #[allow(clippy::too_many_arguments)]
    pub fn find_safe_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        position: &Coord3D,
        repulsor1: &Coord3D,
        repulsor2: &Coord3D,
        radius: Real,
    ) -> Option<Path> {
        if !self.is_map_ready {
            return None;
        }
        let agent = &unit.agent;
        let (unit_radius, center_in_cell) = radius_and_center(Some(agent));
        let safe_square = radius * radius;
        self.zones.set_all_passable();

        let (start, _) = self.world_to_cell(position);
        let mut parent = self.clipped_cell(agent.layer, position)?;
        if !agent.has_ai || !self.allocate_cell_info(parent, start) {
            return None;
        }
        self.start_search(parent, None);

        let search = NeighborSearch {
            unit,
            goal: None,
            center_in_cell,
            radius: unit_radius,
            start,
            attack_distance: 0,
        };
        let mut farthest: Real = 0.0;
        let mut count: Int = 0;
        // `is_tunneling` is whatever the search before this one left.
        while let Some(cell) = self.pop_open() {
            parent = cell;
            let at = self.index_of(parent);
            let layer = self.cell_ref(parent).layer;
            let center = self.adjust_coord_to_cell(terrain, at.x, at.y, center_in_cell, layer);
            let square_to = |pos: &Coord3D| {
                let (dx, dy) = (center.x - pos.x, center.y - pos.y);
                dx * dx + dy * dy
            };
            let mut square = square_to(repulsor1);
            let other = square_to(repulsor2);
            if other < square {
                square = other;
            }
            let mut good = square > safe_square;
            if self.open_list.empty() && count > 0 {
                good = true;
            }
            if square > farthest {
                farthest = square;
                if count > MAX_SAFE_PATH_CELL_COUNT as Int {
                    good = true;
                }
            }
            if good
                && self.check_destination(
                    units,
                    Some(agent),
                    at.x,
                    at.y,
                    layer,
                    unit_radius,
                    center_in_cell,
                )
            {
                let path = self.build_actual_path(
                    terrain,
                    units,
                    unit,
                    position,
                    parent,
                    center_in_cell,
                    false,
                );
                self.end_search(&[parent], &[parent]);
                return Some(path);
            }
            self.close_cell(parent);
            self.check_change_layers(parent);
            count =
                count.wrapping_add(self.examine_neighboring_cells(terrain, units, parent, &search));
        }
        self.is_tunneling = false;
        self.end_search(&[], &[parent]);
        None
    }

    /// `getMoveAwayFromPath`: the path to the nearest cell where the unit
    /// clears `avoid` (and `avoid2`) by the room `other` needs.
    #[allow(clippy::too_many_arguments)]
    pub fn get_move_away_from_path(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        units: &dyn PathUnits,
        unit: &SearchUnit,
        position: &Coord3D,
        other: Option<&PathAgent>,
        avoid: &Path,
        avoid2: Option<&Path>,
    ) -> Option<Path> {
        if !self.is_map_ready {
            return None;
        }
        let agent = &unit.agent;
        let (other_radius, other_center) = radius_and_center(other);
        let crusher = agent.crusher_level > 0;
        self.zones.set_all_passable();
        let (radius, center_in_cell) = radius_and_center(Some(agent));

        let start = self.offset_start_cell(position, center_in_cell);
        let mut parent = self.clipped_cell(agent.layer, position)?;
        if !agent.has_ai {
            return None;
        }
        self.is_tunneling = !self.valid_movement_cell(crusher, unit.surfaces, Some(parent));
        let mut info = CheckMovementInfo {
            cell: start,
            layer: agent.layer,
            center_in_cell,
            radius,
            consider_transient: false,
            ..CheckMovementInfo::default()
        };
        if !self.check_for_movement(units, Some(agent), &mut info) || info.enemy_fixed {
            self.is_tunneling = true;
        }
        if !self.allocate_cell_info(parent, start) {
            return None;
        }
        self.start_search(parent, None);

        let mut half_width = radius as Real * PATHFIND_CELL_SIZE_F - (PATHFIND_CELL_SIZE_F / 4.0);
        if center_in_cell {
            half_width += PATHFIND_CELL_SIZE_F / 2.0;
        }
        half_width += other_radius as Real * PATHFIND_CELL_SIZE_F;
        if other_center {
            half_width += PATHFIND_CELL_SIZE_F / 2.0;
        }

        let search = NeighborSearch {
            unit,
            goal: None,
            center_in_cell,
            radius,
            start,
            attack_distance: 0,
        };
        while let Some(cell) = self.pop_open() {
            parent = cell;
            let at = self.index_of(parent);
            let layer = self.cell_ref(parent).layer;
            let center = self.adjust_coord_to_cell(terrain, at.x, at.y, center_in_cell, layer);
            let bounds = Region2D {
                lo: Coord2D {
                    x: center.x - half_width,
                    y: center.y - half_width,
                },
                hi: Coord2D {
                    x: center.x + half_width,
                    y: center.y + half_width,
                },
            };
            let mut overlap =
                crosses(avoid, &bounds) || avoid2.is_some_and(|path| crosses(path, &bounds));
            // Always move at least one cell.
            if start == at {
                overlap = true;
            }
            if !overlap
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
                self.is_tunneling = false;
                let path = self.build_actual_path(
                    terrain,
                    units,
                    unit,
                    position,
                    parent,
                    center_in_cell,
                    false,
                );
                self.end_search(&[parent], &[parent]);
                return Some(path);
            }
            self.close_cell(parent);
            self.check_change_layers(parent);
            self.examine_neighboring_cells(terrain, units, parent, &search);
        }
        self.is_tunneling = false;
        self.end_search(&[], &[parent]);
        None
    }
}
