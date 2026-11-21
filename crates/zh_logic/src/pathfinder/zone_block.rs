//! `PKT-420`: Hierarchical 10x10 zone equivalency block (`ZoneBlock.h` / `ZoneBlock.cpp`).
//!
//! Seeded for `WP-6.2` (`PKT-418` + `PKT-420`).
//! Reference C++ packet: `packets/phase6/PKT-418_PathfindZoneManager.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt418_zone_manager.rs`
//! (`docs/golden_vectors/PKT-418.json`)

#![allow(unused_variables, dead_code)]

use crate::pathfinder::pathfind_cell::PathfindCell;
use crate::pathfinder::pathfind_zone_manager::{
    PathfindZoneMap, UNINITIALIZED_ZONE,
};
use crate::rules::combat_types::{LocomotorSurfaceType, LocomotorSurfaceTypeMask};
use zh_common::math::{ICoord2D, IRegion2D};
use zh_common::types::{Bool, Int, UnsignedShort};

pub type ZoneStorageType = UnsignedShort;

/// Hierarchical 10x10 pathfinding zone equivalency block matching `ZoneBlock`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ZoneBlock {
    pub cell_origin: ICoord2D,
    pub first_zone: ZoneStorageType,
    pub num_zones: UnsignedShort,
    pub zones_allocated: UnsignedShort,
    pub ground_cliff_zones: Vec<ZoneStorageType>,
    pub ground_water_zones: Vec<ZoneStorageType>,
    pub ground_rubble_zones: Vec<ZoneStorageType>,
    pub crusher_zones: Vec<ZoneStorageType>,
    pub interacts_with_bridge: bool,
    pub marked_passable: bool,
}

impl Default for ZoneBlock {
    fn default() -> Self {
        Self::new()
    }
}

impl ZoneBlock {
    pub fn new() -> Self {
        Self {
            cell_origin: ICoord2D::default(),
            first_zone: 0,
            num_zones: 0,
            zones_allocated: 0,
            ground_cliff_zones: Vec::new(),
            ground_water_zones: Vec::new(),
            ground_rubble_zones: Vec::new(),
            crusher_zones: Vec::new(),
            interacts_with_bridge: false,
            marked_passable: true,
        }
    }

    #[inline]
    pub fn clear_marked_passable(&mut self) {
        self.marked_passable = false;
    }

    #[inline]
    pub const fn is_passable(&self) -> bool {
        self.marked_passable
    }

    #[inline]
    pub fn set_passable(&mut self, pass: bool) {
        self.marked_passable = pass;
    }

    #[inline]
    pub const fn interacts_with_bridge(&self) -> bool {
        self.interacts_with_bridge
    }

    #[inline]
    pub fn set_interacts_with_bridge(&mut self, interacts: bool) {
        self.interacts_with_bridge = interacts;
    }

    pub fn block_calculate_zones(&mut self, map: &PathfindZoneMap, bounds: &IRegion2D) {
        self.cell_origin = ICoord2D::new(bounds.lo.x, bounds.lo.y);

        let first = map.cell(bounds.lo.x, bounds.lo.y).zone();
        let mut min_zone = first;
        let mut max_zone = first;
        for y in bounds.lo.y..=bounds.hi.y {
            for x in bounds.lo.x..=bounds.hi.x {
                let zone = map.cell(x, y).zone();
                min_zone = min_zone.min(zone);
                max_zone = max_zone.max(zone);
            }
        }

        self.first_zone = min_zone;
        self.num_zones = max_zone - min_zone + 1;

        if self.zones_allocated == 0 {
            self.zones_allocated = 4;
        }
        if self.num_zones == 1 {
            self.zones_allocated = 0;
            self.ground_cliff_zones.clear();
            self.ground_water_zones.clear();
            self.ground_rubble_zones.clear();
            self.crusher_zones.clear();
            return;
        }
        while self.zones_allocated <= self.num_zones {
            self.zones_allocated = self.zones_allocated.wrapping_mul(2);
        }
        let len = self.zones_allocated as usize;
        self.ground_cliff_zones.resize(len, 0);
        self.ground_water_zones.resize(len, 0);
        self.ground_rubble_zones.resize(len, 0);
        self.crusher_zones.resize(len, 0);

        for i in 0..len {
            let zone = (i as Int).wrapping_add(self.first_zone as Int) as ZoneStorageType;
            self.ground_cliff_zones[i] = zone;
            self.ground_water_zones[i] = zone;
            self.ground_rubble_zones[i] = zone;
            self.crusher_zones[i] = zone;
        }

        for y in bounds.lo.y..=bounds.hi.y {
            for x in bounds.lo.x..=bounds.hi.x {
                if x > bounds.lo.x {
                    let left = map.cell(x - 1, y);
                    let cell = map.cell(x, y);
                    if cell.zone() != left.zone() {
                        if PathfindCell::water_ground(cell, left) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.ground_water_zones;
                            Self::apply_block_zone(cell, left, eq, first);
                        }
                        if PathfindCell::ground_rubble(cell, left) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.ground_rubble_zones;
                            Self::apply_block_zone(cell, left, eq, first);
                        }
                        if PathfindCell::ground_cliff(cell, left) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.ground_cliff_zones;
                            Self::apply_block_zone(cell, left, eq, first);
                        }
                        if PathfindCell::crusher_ground(cell, left, map.pool(), map.mode()) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.crusher_zones;
                            Self::apply_block_zone(cell, left, eq, first);
                        }
                    }
                }
                if y > bounds.lo.y {
                    let top = map.cell(x, y - 1);
                    let cell = map.cell(x, y);
                    if cell.zone() != top.zone() {
                        if PathfindCell::water_ground(cell, top) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.ground_water_zones;
                            Self::apply_block_zone(cell, top, eq, first);
                        }
                        if PathfindCell::ground_rubble(cell, top) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.ground_rubble_zones;
                            Self::apply_block_zone(cell, top, eq, first);
                        }
                        if PathfindCell::ground_cliff(cell, top) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.ground_cliff_zones;
                            Self::apply_block_zone(cell, top, eq, first);
                        }
                        if PathfindCell::crusher_ground(cell, top, map.pool(), map.mode()) {
                            let first = self.first_zone as Int;
                            let eq = &mut self.crusher_zones;
                            Self::apply_block_zone(cell, top, eq, first);
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
        if zone == UNINITIALIZED_ZONE {
            return zone;
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
        if self.num_zones < 2 {
            return self.first_zone;
        }
        if zone < self.first_zone || zone >= self.first_zone.wrapping_add(self.num_zones) {
            return self.first_zone;
        }

        let mut local = (zone - self.first_zone) as usize;
        if crusher != 0 {
            if local < self.crusher_zones.len() {
                local = (self.crusher_zones[local] - self.first_zone) as usize;
            }
        }
        if acceptable_surfaces & ground != 0 && acceptable_surfaces & cliff != 0 {
            return if local < self.ground_cliff_zones.len() {
                self.ground_cliff_zones[local]
            } else {
                self.first_zone
            };
        }
        if acceptable_surfaces & ground != 0 && acceptable_surfaces & water != 0 {
            return if local < self.ground_water_zones.len() {
                self.ground_water_zones[local]
            } else {
                self.first_zone
            };
        }
        if acceptable_surfaces & ground != 0 && acceptable_surfaces & rubble != 0 {
            return if local < self.ground_rubble_zones.len() {
                self.ground_rubble_zones[local]
            } else {
                self.first_zone
            };
        }
        self.first_zone.wrapping_add(local as ZoneStorageType)
    }

    pub fn resolve_block_zones(
        src_zone: Int,
        target_zone: Int,
        zone_equivalency: &mut [ZoneStorageType],
    ) {
        if target_zone < src_zone {
            for zone in zone_equivalency.iter_mut() {
                if *zone == src_zone as ZoneStorageType {
                    *zone = target_zone as ZoneStorageType;
                }
            }
        } else {
            for zone in zone_equivalency.iter_mut() {
                if *zone == target_zone as ZoneStorageType {
                    *zone = src_zone as ZoneStorageType;
                }
            }
        }
    }

    pub fn resolve_zones(
        src_zone: Int,
        target_zone: Int,
        zone_equivalency: &mut [ZoneStorageType],
    ) {
        if src_zone as usize >= zone_equivalency.len()
            || target_zone as usize >= zone_equivalency.len()
        {
            return;
        }
        let src = zone_equivalency[src_zone as usize];
        let target = zone_equivalency[target_zone as usize];
        let final_zone = if target < src { target } else { src };
        for zone in zone_equivalency.iter_mut() {
            if *zone == src as ZoneStorageType || *zone == target as ZoneStorageType {
                *zone = final_zone;
            }
        }
    }

    pub fn flatten_zones(
        zone_array: &mut [ZoneStorageType],
        zone_hierarchical: &[ZoneStorageType],
    ) {
        for i in 0..zone_array.len() {
            let zone1 = zone_array[i] as usize;
            if zone1 >= zone_hierarchical.len() {
                continue;
            }
            let zone2 = zone_hierarchical[zone1] as usize;
            if zone2 >= zone_array.len() {
                continue;
            }
            let zone1 = zone_array[zone2] as usize;
            if zone1 >= zone_hierarchical.len() {
                continue;
            }
            let zone2 = zone_hierarchical[zone1];
            zone_array[i] = zone2;
        }
        for i in 0..zone_array.len() {
            let zone1 = zone_array[i];
            let zone2 = if i < zone_hierarchical.len() {
                zone_hierarchical[i]
            } else {
                i as ZoneStorageType
            };
            if zone1 != zone2 {
                Self::resolve_zones(zone1 as Int, zone2 as Int, zone_array);
            }
        }
    }

    pub fn apply_zone(
        target_cell: &mut PathfindCell,
        source_cell: &PathfindCell,
        zone_equivalency: &mut [ZoneStorageType],
    ) {
        let source_zone = source_cell.zone() as usize;
        if source_zone >= zone_equivalency.len() {
            return;
        }
        let src = zone_equivalency[source_zone] as Int;
        let target_zone = target_cell.zone() as usize;
        let target = if target_zone < zone_equivalency.len() {
            zone_equivalency[target_zone] as Int
        } else {
            0
        };
        if target == 0 {
            target_cell.set_zone(src as ZoneStorageType);
        } else if target != src {
            Self::resolve_zones(src, target, zone_equivalency);
        }
    }

    pub fn apply_block_zone(
        target_cell: &PathfindCell,
        source_cell: &PathfindCell,
        zone_equivalency: &mut [ZoneStorageType],
        first_zone: Int,
    ) {
        let src_index = (source_cell.zone() as Int - first_zone) as usize;
        let target_index = (target_cell.zone() as Int - first_zone) as usize;
        if src_index >= zone_equivalency.len() || target_index >= zone_equivalency.len() {
            return;
        }
        let src = zone_equivalency[src_index] as Int;
        let target = zone_equivalency[target_index] as Int;
        if target != src {
            Self::resolve_block_zones(src, target, zone_equivalency);
        }
    }
}
