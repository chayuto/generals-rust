//! `PKT-418`: Hierarchical pathfind zone manager (`PathfindZoneManager.h` / `.cpp`).
//!
//! Seeded for `WP-6.2` (`PKT-418` + `PKT-420`).
//! Reference C++ packet: `packets/phase6/PKT-418_PathfindZoneManager.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt418_zone_manager.rs`
//! (`docs/golden_vectors/PKT-418.json`)

#![allow(unused_variables, dead_code)]

use crate::pathfinder::pathfind_cell::{PathfindCell, PathfindCellGrid};
use crate::pathfinder::pathfind_cell_info::{CellIdx, PathfindCellInfoPool};
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, PathfindMode, ZONE_UPDATE_FREQUENCY,
};
use crate::pathfinder::zone_block::{ZoneBlock, ZoneStorageType};
use crate::rules::combat_types::{LocomotorSurfaceType, LocomotorSurfaceTypeMask};
use zh_common::math::{ICoord2D, IRegion2D};
use zh_common::types::{Bool, Int, UnsignedInt, UnsignedShort, FALSE, TRUE};

pub const INITIAL_ZONES: UnsignedShort = 256;
pub const ZONE_BLOCK_SIZE: Int = 10;
pub const UNINITIALIZED_ZONE: ZoneStorageType = 0;
pub const MAX_ZONES: usize = 24_000;
pub const LAYER_COUNT: usize = PathfindLayerEnum::LAST as usize + 1;

/// Layer descriptor bridging `PathfindZoneManager` with `PathfindLayer` (`0..=15`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneLayerSlot {
    pub zone: Int,
    pub unused: bool,
    pub destroyed: bool,
    pub start_cell: ICoord2D,
    pub end_cell: ICoord2D,
    pub applied_zone: Int,
}

impl Default for ZoneLayerSlot {
    fn default() -> Self {
        Self {
            zone: 0,
            unused: true,
            destroyed: false,
            start_cell: ICoord2D::default(),
            end_cell: ICoord2D::default(),
            applied_zone: 0,
        }
    }
}

impl ZoneLayerSlot {
    #[inline]
    pub fn apply_zone(&mut self) {
        self.applied_zone = self.zone;
    }
}

/// Column-major 2D `(x, y)` `PathfindCell` grid passed into `PathfindZoneManager`.
#[derive(Debug, Clone)]
pub struct PathfindZoneMap {
    width: Int,
    height: Int,
    grid: PathfindCellGrid,
}

impl PathfindZoneMap {
    pub fn new(width: Int, height: Int) -> Self {
        let mut coords = Vec::new();
        for x in 0..width.max(0) {
            for y in 0..height.max(0) {
                coords.push(ICoord2D::new(x, y));
            }
        }
        let mode = PathfindMode {
            use_fixed_pathfinding: true,
            force_clean_cells: false,
        };
        Self {
            width,
            height,
            grid: PathfindCellGrid::from_parts(coords, PathfindCellInfoPool::new(), mode),
        }
    }

    #[inline]
    pub const fn width(&self) -> Int {
        self.width
    }

    #[inline]
    pub const fn height(&self) -> Int {
        self.height
    }

    /// The grid index of the ground cell at `(x, y)`.
    #[inline]
    pub fn index(&self, x: Int, y: Int) -> CellIdx {
        CellIdx((x as usize) * (self.height as usize) + (y as usize))
    }

    /// The cells and info pool: ground first, then any appended layer cells.
    #[inline]
    pub fn grid(&self) -> &PathfindCellGrid {
        &self.grid
    }

    #[inline]
    pub fn grid_mut(&mut self) -> &mut PathfindCellGrid {
        &mut self.grid
    }

    #[inline]
    pub fn cell(&self, x: Int, y: Int) -> &PathfindCell {
        self.grid.cell(self.index(x, y))
    }

    #[inline]
    pub fn cell_mut(&mut self, x: Int, y: Int) -> &mut PathfindCell {
        let idx = self.index(x, y);
        self.grid.cell_mut(idx)
    }

    #[inline]
    pub fn pool(&self) -> &PathfindCellInfoPool {
        self.grid.pool()
    }

    #[inline]
    pub fn pool_mut(&mut self) -> &mut PathfindCellInfoPool {
        self.grid.pool_mut()
    }

    #[inline]
    pub fn mode(&self) -> PathfindMode {
        self.grid.mode()
    }

    #[inline]
    pub fn mode_mut(&mut self) -> &mut PathfindMode {
        self.grid.mode_mut()
    }
}

/// Connectivity zone manager and hierarchical block registry (`PathfindZoneManager`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathfindZoneManager {
    pub zone_blocks: Vec<ZoneBlock>,
    pub zone_block_extent: ICoord2D,
    pub max_zone: UnsignedShort,
    pub next_frame_to_calculate_zones: UnsignedInt,
    pub zones_allocated: UnsignedShort,
    pub ground_cliff_zones: Vec<ZoneStorageType>,
    pub ground_water_zones: Vec<ZoneStorageType>,
    pub ground_rubble_zones: Vec<ZoneStorageType>,
    pub terrain_zones: Vec<ZoneStorageType>,
    pub crusher_zones: Vec<ZoneStorageType>,
    pub hierarchical_zones: Vec<ZoneStorageType>,
}

impl Default for PathfindZoneManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PathfindZoneManager {
    pub fn new() -> Self {
        Self {
            zone_blocks: Vec::new(),
            zone_block_extent: ICoord2D::default(),
            max_zone: 0,
            next_frame_to_calculate_zones: 0,
            zones_allocated: 0,
            ground_cliff_zones: Vec::new(),
            ground_water_zones: Vec::new(),
            ground_rubble_zones: Vec::new(),
            terrain_zones: Vec::new(),
            crusher_zones: Vec::new(),
            hierarchical_zones: Vec::new(),
        }
    }

    pub fn reset(&mut self) {
        self.zone_blocks.clear();
        self.zone_block_extent = ICoord2D::default();
        self.max_zone = 0;
        self.next_frame_to_calculate_zones = 0;
        self.zones_allocated = 0;
        self.ground_cliff_zones.clear();
        self.ground_water_zones.clear();
        self.ground_rubble_zones.clear();
        self.terrain_zones.clear();
        self.crusher_zones.clear();
        self.hierarchical_zones.clear();
    }

    pub fn need_to_calculate_zones(&self, current_frame: UnsignedInt) -> Bool {
        if self.next_frame_to_calculate_zones <= current_frame { TRUE } else { FALSE }
    }

    pub fn mark_zones_dirty(&mut self, current_frame: UnsignedInt) {
        if current_frame < 2 {
            self.next_frame_to_calculate_zones = 2;
        } else {
            let candidate = current_frame.wrapping_add(ZONE_UPDATE_FREQUENCY);
            self.next_frame_to_calculate_zones =
                self.next_frame_to_calculate_zones.min(candidate);
        }
    }

    pub fn allocate_blocks(&mut self, global_bounds: &IRegion2D) {
        self.zone_blocks.clear();
        self.zone_block_extent.x =
            (global_bounds.hi.x - global_bounds.lo.x + 1 + ZONE_BLOCK_SIZE - 1)
                / ZONE_BLOCK_SIZE;
        self.zone_block_extent.y =
            (global_bounds.hi.y - global_bounds.lo.y + 1 + ZONE_BLOCK_SIZE - 1)
                / ZONE_BLOCK_SIZE;
        let count = (self.zone_block_extent.x as usize)
            .saturating_mul(self.zone_block_extent.y as usize);
        self.zone_blocks = vec![ZoneBlock::new(); count];
        for block in &mut self.zone_blocks {
            block.marked_passable = true;
        }
    }

    pub fn get_extent(&self) -> ICoord2D {
        self.zone_block_extent
    }

    fn allocate_zones(&mut self) {
        if self.zones_allocated as Int > self.max_zone as Int
            && !self.ground_cliff_zones.is_empty()
        {
            return;
        }
        self.ground_cliff_zones.clear();
        self.ground_water_zones.clear();
        self.ground_rubble_zones.clear();
        self.terrain_zones.clear();
        self.crusher_zones.clear();
        self.hierarchical_zones.clear();
        if self.zones_allocated == 0 {
            self.zones_allocated = INITIAL_ZONES;
        }
        while self.zones_allocated as Int <= self.max_zone as Int {
            self.zones_allocated = self.zones_allocated.wrapping_mul(2);
        }
        let len = self.zones_allocated as usize;
        self.ground_cliff_zones = (0..len).map(|i| i as ZoneStorageType).collect();
        self.ground_water_zones = (0..len).map(|i| i as ZoneStorageType).collect();
        self.ground_rubble_zones = (0..len).map(|i| i as ZoneStorageType).collect();
        self.terrain_zones = (0..len).map(|i| i as ZoneStorageType).collect();
        self.crusher_zones = (0..len).map(|i| i as ZoneStorageType).collect();
        self.hierarchical_zones = (0..len).map(|i| i as ZoneStorageType).collect();
    }

    fn block_index(&self, bx: Int, by: Int) -> Option<usize> {
        if bx < 0
            || by < 0
            || bx >= self.zone_block_extent.x
            || by >= self.zone_block_extent.y
        {
            return None;
        }
        Some((bx as usize) * (self.zone_block_extent.y as usize) + by as usize)
    }

    pub fn calculate_zones(
        &mut self,
        map: &mut PathfindZoneMap,
        layers: &mut [ZoneLayerSlot; LAYER_COUNT],
        global_bounds: &IRegion2D,
    ) {
        self.max_zone = 1;
        let mut zone_equivalency: Vec<ZoneStorageType> = (0..MAX_ZONES)
            .map(|i| i as ZoneStorageType)
            .collect();
        for layer in layers.iter_mut() {
            layer.zone = 0;
        }

        let x_count =
            (global_bounds.hi.x - global_bounds.lo.x + 1 + ZONE_BLOCK_SIZE - 1) / ZONE_BLOCK_SIZE;
        let y_count =
            (global_bounds.hi.y - global_bounds.lo.y + 1 + ZONE_BLOCK_SIZE - 1) / ZONE_BLOCK_SIZE;

        for bx in 0..x_count {
            for by in 0..y_count {
                let mut bounds = IRegion2D {
                    lo: ICoord2D::new(
                        global_bounds.lo.x + bx * ZONE_BLOCK_SIZE,
                        global_bounds.lo.y + by * ZONE_BLOCK_SIZE,
                    ),
                    hi: ICoord2D::default(),
                };
                bounds.hi.x = (bounds.lo.x + ZONE_BLOCK_SIZE - 1).min(global_bounds.hi.x);
                bounds.hi.y = (bounds.lo.y + ZONE_BLOCK_SIZE - 1).min(global_bounds.hi.y);
                if let Some(index) = self.block_index(bx, by) {
                    self.zone_blocks[index].set_interacts_with_bridge(false);
                }
                for y in bounds.lo.y..=bounds.hi.y {
                    for x in bounds.lo.x..=bounds.hi.x {
                        map.cell_mut(x, y).set_zone(0);
                        let same_left = x > bounds.lo.x
                            && map.cell(x, y).cell_type == map.cell(x - 1, y).cell_type;
                        if same_left {
                            let mut target = map.cell(x, y).clone();
                            let source = map.cell(x - 1, y).clone();
                            ZoneBlock::apply_zone(
                                &mut target,
                                &source,
                                &mut zone_equivalency,
                            );
                            map.cell_mut(x, y).set_zone(target.zone());
                        }
                        let same_top = y > bounds.lo.y
                            && map.cell(x, y).cell_type == map.cell(x, y - 1).cell_type;
                        if same_top {
                            let mut target = map.cell(x, y).clone();
                            let source = map.cell(x, y - 1).clone();
                            ZoneBlock::apply_zone(
                                &mut target,
                                &source,
                                &mut zone_equivalency,
                            );
                            map.cell_mut(x, y).set_zone(target.zone());
                        }
                        if map.cell(x, y).zone() == 0 {
                            map.cell_mut(x, y).set_zone(self.max_zone);
                            self.max_zone = self.max_zone.wrapping_add(1);
                        }
                        if map.cell(x, y).connect_layer() as i32 > PathfindLayerEnum::Ground as i32
                        {
                            if let Some(index) = self.block_index(bx, by) {
                                self.zone_blocks[index].set_interacts_with_bridge(true);
                            }
                        }
                    }
                }
            }
        }

        let total_zones = self.max_zone as usize;
        self.max_zone = 1;
        let mut collapsed = vec![0_u16; MAX_ZONES];
        collapsed[0] = 0;
        for i in 1..total_zones {
            let zone = zone_equivalency[i];
            if zone as usize == i {
                collapsed[i] = self.max_zone;
                self.max_zone = self.max_zone.wrapping_add(1);
            } else {
                collapsed[i] = collapsed[zone as usize];
            }
        }
        for y in global_bounds.lo.y..=global_bounds.hi.y {
            for x in global_bounds.lo.x..=global_bounds.hi.x {
                let zone = map.cell(x, y).zone() as usize;
                map.cell_mut(x, y).set_zone(collapsed[zone]);
            }
        }

        for layer in layers.iter_mut() {
            let zone_index = layer.zone as usize;
            let zone = if zone_index < collapsed.len() {
                collapsed[zone_index]
            } else {
                0
            };
            if zone == 0 {
                let zone = self.max_zone;
                self.max_zone = self.max_zone.wrapping_add(1);
                layer.zone = zone as Int;
            } else {
                layer.zone = zone as Int;
            }
            layer.apply_zone();
            if !layer.unused && !layer.destroyed {
                self.set_bridge(layer.start_cell.x, layer.start_cell.y, TRUE);
                self.set_bridge(layer.end_cell.x, layer.end_cell.y, TRUE);
            }
        }

        self.allocate_zones();

        for bx in 0..x_count {
            for by in 0..y_count {
                let mut bounds = IRegion2D {
                    lo: ICoord2D::new(
                        global_bounds.lo.x + bx * ZONE_BLOCK_SIZE,
                        global_bounds.lo.y + by * ZONE_BLOCK_SIZE,
                    ),
                    hi: ICoord2D::default(),
                };
                bounds.hi.x = (bounds.lo.x + ZONE_BLOCK_SIZE - 1).min(global_bounds.hi.x);
                bounds.hi.y = (bounds.lo.y + ZONE_BLOCK_SIZE - 1).min(global_bounds.hi.y);
                if let Some(index) = self.block_index(bx, by) {
                    self.zone_blocks[index].block_calculate_zones(map, &bounds);
                }
            }
        }

        for i in 0..self.zones_allocated as usize {
            self.ground_cliff_zones[i] = i as ZoneStorageType;
            self.ground_water_zones[i] = i as ZoneStorageType;
            self.ground_rubble_zones[i] = i as ZoneStorageType;
            self.terrain_zones[i] = i as ZoneStorageType;
            self.crusher_zones[i] = i as ZoneStorageType;
            self.hierarchical_zones[i] = i as ZoneStorageType;
        }

        for y in global_bounds.lo.y..=global_bounds.hi.y {
            for x in global_bounds.lo.x..=global_bounds.hi.x {
                let cell = map.cell(x, y).clone();
                if cell.connect_layer() as i32 > PathfindLayerEnum::Ground as i32
                    && cell.cell_type == crate::pathfinder::pathfind_cell::CellType::Clear
                {
                    let li = cell.connect_layer() as usize;
                    if li < layers.len() {
                        ZoneBlock::resolve_zones(
                            cell.zone() as Int,
                            layers[li].zone,
                            &mut self.hierarchical_zones,
                        );
                    }
                }
                if x > global_bounds.lo.x {
                    let left = map.cell(x - 1, y).clone();
                    if cell.zone() != left.zone() {
                        if cell.cell_type == left.cell_type {
                            let mut target = cell.clone();
                            ZoneBlock::apply_zone(
                                &mut target,
                                &left,
                                &mut self.hierarchical_zones,
                            );
                            map.cell_mut(x, y).set_zone(target.zone());
                        } else {
                            let mut matched = false;
                            if PathfindCell::terrain(&cell, &left) {
                                let mut target = cell.clone();
                                ZoneBlock::apply_zone(
                                    &mut target,
                                    &left,
                                    &mut self.terrain_zones,
                                );
                                map.cell_mut(x, y).set_zone(target.zone());
                                matched = true;
                            }
                            if PathfindCell::crusher_ground(
                                &cell,
                                &left,
                                map.pool(),
                                map.mode(),
                            ) {
                                let mut target = cell.clone();
                                ZoneBlock::apply_zone(
                                    &mut target,
                                    &left,
                                    &mut self.crusher_zones,
                                );
                                map.cell_mut(x, y).set_zone(target.zone());
                                matched = true;
                            }
                            if !matched {
                                if PathfindCell::water_ground(&cell, &left) {
                                    let mut target = cell.clone();
                                    ZoneBlock::apply_zone(
                                        &mut target,
                                        &left,
                                        &mut self.ground_water_zones,
                                    );
                                    map.cell_mut(x, y).set_zone(target.zone());
                                } else if PathfindCell::ground_rubble(&cell, &left) {
                                    let mut target = cell.clone();
                                    ZoneBlock::apply_zone(
                                        &mut target,
                                        &left,
                                        &mut self.ground_rubble_zones,
                                    );
                                    map.cell_mut(x, y).set_zone(target.zone());
                                } else if PathfindCell::ground_cliff(&cell, &left) {
                                    let mut target = cell.clone();
                                    ZoneBlock::apply_zone(
                                        &mut target,
                                        &left,
                                        &mut self.ground_cliff_zones,
                                    );
                                    map.cell_mut(x, y).set_zone(target.zone());
                                }
                            }
                        }
                    }
                }
                if y > global_bounds.lo.y {
                    let top = map.cell(x, y - 1).clone();
                    if cell.zone() != top.zone() {
                        if cell.cell_type == top.cell_type {
                            let mut target = cell.clone();
                            ZoneBlock::apply_zone(
                                &mut target,
                                &top,
                                &mut self.hierarchical_zones,
                            );
                            map.cell_mut(x, y).set_zone(target.zone());
                        } else {
                            let mut matched = false;
                            if PathfindCell::terrain(&cell, &top) {
                                let mut target = cell.clone();
                                ZoneBlock::apply_zone(
                                    &mut target,
                                    &top,
                                    &mut self.terrain_zones,
                                );
                                map.cell_mut(x, y).set_zone(target.zone());
                                matched = true;
                            }
                            if PathfindCell::crusher_ground(
                                &cell,
                                &top,
                                map.pool(),
                                map.mode(),
                            ) {
                                let mut target = cell.clone();
                                ZoneBlock::apply_zone(
                                    &mut target,
                                    &top,
                                    &mut self.crusher_zones,
                                );
                                map.cell_mut(x, y).set_zone(target.zone());
                                matched = true;
                            }
                            if !matched {
                                if PathfindCell::water_ground(&cell, &top) {
                                    let mut target = cell.clone();
                                    ZoneBlock::apply_zone(
                                        &mut target,
                                        &top,
                                        &mut self.ground_water_zones,
                                    );
                                    map.cell_mut(x, y).set_zone(target.zone());
                                } else if PathfindCell::ground_rubble(&cell, &top) {
                                    let mut target = cell.clone();
                                    ZoneBlock::apply_zone(
                                        &mut target,
                                        &top,
                                        &mut self.ground_rubble_zones,
                                    );
                                    map.cell_mut(x, y).set_zone(target.zone());
                                } else if PathfindCell::ground_cliff(&cell, &top) {
                                    let mut target = cell.clone();
                                    ZoneBlock::apply_zone(
                                        &mut target,
                                        &top,
                                        &mut self.ground_cliff_zones,
                                    );
                                    map.cell_mut(x, y).set_zone(target.zone());
                                }
                            }
                        }
                    }
                }
            }
        }

        for i in 1..self.max_zone as usize {
            if i < self.hierarchical_zones.len() {
                let zone = self.hierarchical_zones[i] as usize;
                if zone < self.hierarchical_zones.len() {
                    self.hierarchical_zones[i] = self.hierarchical_zones[zone];
                }
            }
        }
        ZoneBlock::flatten_zones(&mut self.ground_cliff_zones, &self.hierarchical_zones);
        ZoneBlock::flatten_zones(&mut self.ground_water_zones, &self.hierarchical_zones);
        ZoneBlock::flatten_zones(&mut self.ground_rubble_zones, &self.hierarchical_zones);
        ZoneBlock::flatten_zones(&mut self.terrain_zones, &self.hierarchical_zones);
        ZoneBlock::flatten_zones(&mut self.crusher_zones, &self.hierarchical_zones);
        self.next_frame_to_calculate_zones = UnsignedInt::MAX;
    }

    pub fn update_zones_for_modify(
        &mut self,
        map: &mut PathfindZoneMap,
        layers: &mut [ZoneLayerSlot; LAYER_COUNT],
        structure_bounds: &IRegion2D,
        global_bounds: &IRegion2D,
    ) {
        let mut bounds = IRegion2D {
            lo: ICoord2D::new(structure_bounds.lo.x, structure_bounds.lo.y),
            hi: ICoord2D::new(structure_bounds.hi.x + 1, structure_bounds.hi.y + 1),
        };
        bounds.hi.x = bounds.hi.x.min(global_bounds.hi.x);
        bounds.hi.y = bounds.hi.y.min(global_bounds.hi.y);

        for bx in 0..self.zone_block_extent.x {
            for by in 0..self.zone_block_extent.y {
                let mut block_bounds = IRegion2D {
                    lo: ICoord2D::new(
                        global_bounds.lo.x + bx * ZONE_BLOCK_SIZE,
                        global_bounds.lo.y + by * ZONE_BLOCK_SIZE,
                    ),
                    hi: ICoord2D::default(),
                };
                block_bounds.hi.x =
                    (block_bounds.lo.x + ZONE_BLOCK_SIZE - 1).min(global_bounds.hi.x);
                block_bounds.hi.y =
                    (block_bounds.lo.y + ZONE_BLOCK_SIZE - 1).min(global_bounds.hi.y);
                let lo_x = block_bounds.lo.x.max(bounds.lo.x);
                let lo_y = block_bounds.lo.y.max(bounds.lo.y);
                let hi_x = block_bounds.hi.x.min(bounds.hi.x);
                let hi_y = block_bounds.hi.y.min(bounds.hi.y);
                if lo_x > hi_x || lo_y > hi_y {
                    continue;
                }
                block_bounds.lo = ICoord2D::new(lo_x, lo_y);
                block_bounds.hi = ICoord2D::new(hi_x, hi_y);
                if let Some(index) = self.block_index(bx, by) {
                    self.zone_blocks[index].set_interacts_with_bridge(false);
                }

                for y in block_bounds.lo.y..=block_bounds.hi.y {
                    for x in block_bounds.lo.x..=block_bounds.hi.x {
                        if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                            continue;
                        }
                        if x > block_bounds.lo.x
                            && map.cell(x, y).cell_type == map.cell(x - 1, y).cell_type
                        {
                            let zone = map.cell(x - 1, y).zone();
                            map.cell_mut(x, y).set_zone(zone);
                            if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                                continue;
                            }
                        }
                        if y > block_bounds.lo.y
                            && map.cell(x, y).cell_type == map.cell(x, y - 1).cell_type
                        {
                            let zone = map.cell(x, y - 1).zone();
                            map.cell_mut(x, y).set_zone(zone);
                            if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                                continue;
                            }
                        }
                        if y > block_bounds.lo.y && x < block_bounds.hi.x {
                            let target = map.cell(x, y).clone();
                            let right_top = map.cell(x + 1, y - 1).clone();
                            let right = map.cell(x + 1, y).clone();
                            if PathfindCell::types_match(&target, &right_top)
                                && PathfindCell::types_match(&target, &right)
                            {
                                let zone = right_top.zone();
                                map.cell_mut(x, y).set_zone(zone);
                                if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                                    continue;
                                }
                            }
                        }
                    }
                }

                for y in (block_bounds.lo.y..=block_bounds.hi.y).rev() {
                    for x in (block_bounds.lo.x..=block_bounds.hi.x).rev() {
                        if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                            continue;
                        }
                        if x < block_bounds.hi.x
                            && map.cell(x, y).cell_type == map.cell(x + 1, y).cell_type
                        {
                            let zone = map.cell(x + 1, y).zone();
                            map.cell_mut(x, y).set_zone(zone);
                            if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                                continue;
                            }
                        }
                        if y < block_bounds.hi.y
                            && map.cell(x, y).cell_type == map.cell(x, y + 1).cell_type
                        {
                            let zone = map.cell(x, y + 1).zone();
                            map.cell_mut(x, y).set_zone(zone);
                            if map.cell(x, y).zone() != UNINITIALIZED_ZONE {
                                continue;
                            }
                        }
                        if y < block_bounds.hi.y && x < block_bounds.hi.x {
                            let target = map.cell(x, y).clone();
                            let right_bottom = map.cell(x + 1, y + 1).clone();
                            let right = map.cell(x + 1, y).clone();
                            if PathfindCell::types_match(&target, &right_bottom)
                                && PathfindCell::types_match(&target, &right)
                            {
                                let zone = right_bottom.zone();
                                map.cell_mut(x, y).set_zone(zone);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_effective_zone(
        &self,
        acceptable_surfaces: LocomotorSurfaceTypeMask,
        crusher: Bool,
        zone: ZoneStorageType,
    ) -> ZoneStorageType {
        if zone as Int > self.max_zone as Int {
            return UNINITIALIZED_ZONE;
        }
        let ground = LocomotorSurfaceType::Ground.as_i32();
        let water = LocomotorSurfaceType::Water.as_i32();
        let cliff = LocomotorSurfaceType::Cliff.as_i32();
        let rubble = LocomotorSurfaceType::Rubble.as_i32();
        let air = LocomotorSurfaceType::Air.as_i32();
        if acceptable_surfaces & air != 0 {
            return 1;
        }
        if acceptable_surfaces & ground != 0
            && acceptable_surfaces & water != 0
            && acceptable_surfaces & cliff != 0
        {
            return 1;
        }
        let zone_index = zone as usize;
        if zone_index >= self.zones_allocated as usize {
            return UNINITIALIZED_ZONE;
        }
        let mut effective = zone;
        if crusher != FALSE {
            if zone_index < self.crusher_zones.len() {
                effective = self.crusher_zones[zone_index];
            }
        }
        if acceptable_surfaces & ground != 0 && acceptable_surfaces & cliff != 0 {
            let i = effective as usize;
            return if i < self.ground_cliff_zones.len() {
                self.ground_cliff_zones[i]
            } else {
                UNINITIALIZED_ZONE
            };
        }
        if acceptable_surfaces & ground != 0 && acceptable_surfaces & water != 0 {
            let i = effective as usize;
            return if i < self.ground_water_zones.len() {
                self.ground_water_zones[i]
            } else {
                UNINITIALIZED_ZONE
            };
        }
        if acceptable_surfaces & ground != 0 && acceptable_surfaces & rubble != 0 {
            let i = effective as usize;
            return if i < self.ground_rubble_zones.len() {
                self.ground_rubble_zones[i]
            } else {
                UNINITIALIZED_ZONE
            };
        }
        let i = effective as usize;
        if i < self.hierarchical_zones.len() {
            self.hierarchical_zones[i]
        } else {
            UNINITIALIZED_ZONE
        }
    }

    pub fn get_effective_terrain_zone(&self, zone: ZoneStorageType) -> ZoneStorageType {
        let zone_index = zone as usize;
        if zone_index >= self.terrain_zones.len()
            || zone_index >= self.hierarchical_zones.len()
        {
            return UNINITIALIZED_ZONE;
        }
        let terrain = self.terrain_zones[zone_index] as usize;
        if terrain < self.hierarchical_zones.len() {
            self.hierarchical_zones[terrain]
        } else {
            UNINITIALIZED_ZONE
        }
    }

    pub fn get_block_zone(
        &self,
        acceptable_surfaces: LocomotorSurfaceTypeMask,
        crusher: Bool,
        cell_x: Int,
        cell_y: Int,
        map: &PathfindZoneMap,
    ) -> ZoneStorageType {
        if cell_x < 0
            || cell_y < 0
            || cell_x >= map.width()
            || cell_y >= map.height()
        {
            return UNINITIALIZED_ZONE;
        }
        let block_x = cell_x / ZONE_BLOCK_SIZE;
        let block_y = cell_y / ZONE_BLOCK_SIZE;
        let Some(index) = self.block_index(block_x, block_y) else {
            return UNINITIALIZED_ZONE;
        };
        let zone = self.zone_blocks[index].get_effective_zone(
            acceptable_surfaces,
            crusher,
            map.cell(cell_x, cell_y).zone(),
        );
        if zone as Int >= self.max_zone as Int {
            UNINITIALIZED_ZONE
        } else {
            zone
        }
    }

    pub fn clear_passable_flags(&mut self) {
        for block in &mut self.zone_blocks {
            block.clear_marked_passable();
        }
    }

    pub fn set_all_passable(&mut self) {
        for block in &mut self.zone_blocks {
            block.set_passable(true);
        }
    }

    pub fn set_passable(&mut self, cell_x: Int, cell_y: Int, passable: Bool) {
        let block_x = cell_x / ZONE_BLOCK_SIZE;
        let block_y = cell_y / ZONE_BLOCK_SIZE;
        if let Some(index) = self.block_index(block_x, block_y) {
            self.zone_blocks[index].set_passable(passable != FALSE);
        }
    }

    pub fn is_passable(&self, cell_x: Int, cell_y: Int) -> Bool {
        let block_x = cell_x / ZONE_BLOCK_SIZE;
        let block_y = cell_y / ZONE_BLOCK_SIZE;
        match self.block_index(block_x, block_y) {
            Some(index) => if self.zone_blocks[index].is_passable() { TRUE } else { FALSE },
            None => FALSE,
        }
    }

    pub fn clip_is_passable(&self, cell_x: Int, cell_y: Int) -> Bool {
        self.is_passable(cell_x, cell_y)
    }

    pub fn set_bridge(&mut self, cell_x: Int, cell_y: Int, bridge: Bool) {
        let block_x = cell_x / ZONE_BLOCK_SIZE;
        let block_y = cell_y / ZONE_BLOCK_SIZE;
        if let Some(index) = self.block_index(block_x, block_y) {
            self.zone_blocks[index].set_interacts_with_bridge(bridge != FALSE);
        }
    }

    pub fn interacts_with_bridge(&self, cell_x: Int, cell_y: Int) -> Bool {
        let block_x = cell_x / ZONE_BLOCK_SIZE;
        let block_y = cell_y / ZONE_BLOCK_SIZE;
        match self.block_index(block_x, block_y) {
            Some(index) => if self.zone_blocks[index].interacts_with_bridge() { TRUE } else { FALSE },
            None => FALSE,
        }
    }
}
