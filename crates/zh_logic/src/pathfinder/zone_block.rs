//! `PKT-420`: Hierarchical 10x10 zone equivalency block (`ZoneBlock.h` / `ZoneBlock.cpp`).
//!
//! Seeded for `WP-6.2` (`PKT-418` + `PKT-420`).
//! Reference C++ packet: `packets/phase6/PKT-418_PathfindZoneManager.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt418_zone_manager.rs`
//! (`docs/golden_vectors/PKT-418.json`)

#![allow(unused_variables, dead_code)]

use crate::pathfinder::pathfind_cell::PathfindCell;
use crate::pathfinder::pathfind_zone_manager::PathfindZoneMap;
use crate::rules::combat_types::LocomotorSurfaceTypeMask;
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
        todo!("PKT-420")
    }

    pub fn get_effective_zone(
        &self,
        acceptable_surfaces: LocomotorSurfaceTypeMask,
        crusher: Bool,
        zone: ZoneStorageType,
    ) -> ZoneStorageType {
        todo!("PKT-420")
    }

    pub fn resolve_block_zones(
        src_zone: Int,
        target_zone: Int,
        zone_equivalency: &mut [ZoneStorageType],
    ) {
        todo!("PKT-420")
    }

    pub fn resolve_zones(
        src_zone: Int,
        target_zone: Int,
        zone_equivalency: &mut [ZoneStorageType],
    ) {
        todo!("PKT-420")
    }

    pub fn flatten_zones(
        zone_array: &mut [ZoneStorageType],
        zone_hierarchical: &[ZoneStorageType],
    ) {
        todo!("PKT-420")
    }

    pub fn apply_zone(
        target_cell: &mut PathfindCell,
        source_cell: &PathfindCell,
        zone_equivalency: &mut [ZoneStorageType],
    ) {
        todo!("PKT-420")
    }

    pub fn apply_block_zone(
        target_cell: &PathfindCell,
        source_cell: &PathfindCell,
        zone_equivalency: &mut [ZoneStorageType],
        first_zone: Int,
    ) {
        todo!("PKT-420")
    }
}
