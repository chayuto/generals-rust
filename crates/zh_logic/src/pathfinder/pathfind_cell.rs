//! `PKT-413`: Pathfinder grid cell state, A* list insertion, cost heuristics,
//! and transition predicates
//! (`Core/GameEngine/Include/GameLogic/Pathfinder/PathfindCell.h` &
//! `Core/GameEngine/Source/GameLogic/AI/Pathfinder/PathfindCell.cpp`).
//!
//! Reference C++ packet: `packets/phase6/PKT-413_PathfindCell.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt413_pathfind_cell.rs`
//! (`docs/golden_vectors/PKT-413.json`)

#![allow(unused_variables, dead_code)]

use crate::pathfinder::pathfind_cell_info::{CellIdx, InfoIdx, PathfindCellInfoPool};
use crate::pathfinder::pathfind_cell_list::PathfindCellList;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PathfindMode};
use zh_common::math::ICoord2D;
use zh_common::types::{Int, ObjectId, UnsignedInt, UnsignedShort};

/// Pathfinder terrain classification (`PathfindCell::CellType`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellType {
    #[default]
    Clear = 0,
    Water = 1,
    Cliff = 2,
    Rubble = 3,
    Obstacle = 4,
    BridgeImpassable = 5,
    Impassable = 6,
}

impl CellType {
    /// Converts a raw 4-bit integer into `CellType`.
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Clear),
            1 => Some(Self::Water),
            2 => Some(Self::Cliff),
            3 => Some(Self::Rubble),
            4 => Some(Self::Obstacle),
            5 => Some(Self::BridgeImpassable),
            6 => Some(Self::Impassable),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Unit occupancy state (`PathfindCell::CellFlags`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CellFlags {
    #[default]
    NoUnits = 0,
    UnitGoal = 1,
    UnitPresentMoving = 2,
    UnitPresentFixed = 3,
    UnitGoalOtherMoving = 5,
}

impl CellFlags {
    /// Converts a raw 4-bit integer into `CellFlags`.
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::NoUnits),
            1 => Some(Self::UnitGoal),
            2 => Some(Self::UnitPresentMoving),
            3 => Some(Self::UnitPresentFixed),
            5 => Some(Self::UnitGoalOtherMoving),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Lightweight obstacle metadata replacing `Object* obstacle` parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObstacleDescriptor {
    pub id: ObjectId,
    pub is_rubble: bool,
    pub is_fence: bool,
    pub is_transparent: bool,
}

/// Single pathfinding grid cell (`PathfindCell`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathfindCell {
    pub info_idx: Option<InfoIdx>,
    pub obstacle_id: ObjectId,
    pub blocked_by_ally: bool,
    pub obstacle_is_fence: bool,
    pub obstacle_is_transparent: bool,
    /// 14-bit zone ID (`m_zone : 14`, masked with `0x3FFF`).
    pub zone: UnsignedShort,
    pub aircraft_goal: bool,
    pub pinched: bool,
    pub cell_type: CellType,
    pub flags: CellFlags,
    pub connects_to_layer: PathfindLayerEnum,
    pub layer: PathfindLayerEnum,
}

impl Default for PathfindCell {
    fn default() -> Self {
        Self::new()
    }
}

impl PathfindCell {
    /// Creates a cell initialized to `PathfindCell::reset()` defaults.
    pub fn new() -> Self {
        Self {
            info_idx: None,
            obstacle_id: ObjectId::INVALID,
            blocked_by_ally: false,
            obstacle_is_fence: false,
            obstacle_is_transparent: false,
            zone: 0,
            aircraft_goal: false,
            pinched: false,
            cell_type: CellType::Clear,
            flags: CellFlags::NoUnits,
            connects_to_layer: PathfindLayerEnum::Invalid,
            layer: PathfindLayerEnum::Ground,
        }
    }

    #[inline]
    pub const fn cell_type(&self) -> CellType {
        self.cell_type
    }

    #[inline]
    pub const fn flags(&self) -> CellFlags {
        self.flags
    }

    #[inline]
    pub const fn is_aircraft_goal(&self) -> bool {
        self.aircraft_goal
    }

    #[inline]
    pub const fn has_info(&self) -> bool {
        self.info_idx.is_some()
    }

    #[inline]
    pub const fn zone(&self) -> UnsignedShort {
        self.zone
    }

    /// Sets the 14-bit zone ID (`m_zone : 14`, masked to `0x3FFF`).
    #[inline]
    pub fn set_zone(&mut self, zone: UnsignedShort) {
        self.zone = zone & 0x3FFF;
    }

    #[inline]
    pub const fn pinched(&self) -> bool {
        self.pinched
    }

    #[inline]
    pub fn set_pinched(&mut self, pinch: bool) {
        self.pinched = pinch;
    }

    #[inline]
    pub const fn layer(&self) -> PathfindLayerEnum {
        self.layer
    }

    #[inline]
    pub fn set_layer(&mut self, layer: PathfindLayerEnum) {
        self.layer = layer;
    }

    #[inline]
    pub const fn connect_layer(&self) -> PathfindLayerEnum {
        self.connects_to_layer
    }

    #[inline]
    pub fn set_connect_layer(&mut self, layer: PathfindLayerEnum) {
        self.connects_to_layer = layer;
    }

    pub fn is_obstacle_fence(&self, pool: &PathfindCellInfoPool, mode: PathfindMode) -> bool {
        if mode.use_fixed_pathfinding {
            self.obstacle_is_fence
        } else {
            self.info_idx
                .and_then(|idx| pool.get(idx))
                .is_some_and(|info| info.obstacle_is_fence)
        }
    }

    pub fn is_obstacle_transparent(
        &self,
        pool: &PathfindCellInfoPool,
        mode: PathfindMode,
    ) -> bool {
        if mode.use_fixed_pathfinding {
            self.obstacle_is_transparent
        } else {
            self.info_idx
                .and_then(|idx| pool.get(idx))
                .is_some_and(|info| info.obstacle_is_transparent)
        }
    }

    pub fn obstacle_id(&self, pool: &PathfindCellInfoPool, mode: PathfindMode) -> ObjectId {
        if mode.use_fixed_pathfinding {
            self.obstacle_id
        } else {
            self.info_idx
                .and_then(|idx| pool.get(idx))
                .map_or(ObjectId::INVALID, |info| info.obstacle_id)
        }
    }

    pub fn is_obstacle_present(
        &self,
        obj_id: ObjectId,
        pool: &PathfindCellInfoPool,
        mode: PathfindMode,
    ) -> bool {
        if obj_id != ObjectId::INVALID && self.cell_type == CellType::Obstacle {
            return self.obstacle_id(pool, mode) == obj_id;
        }
        false
    }

    pub fn total_cost_difference(&self, other: &PathfindCell, pool: &PathfindCellInfoPool) -> UnsignedInt {
        todo!("PKT-413: abs(totalCost_self - totalCost_other) or u32::MAX if info missing")
    }

    pub fn types_match(target: &PathfindCell, source: &PathfindCell) -> bool {
        todo!("PKT-413: target.cell_type == source.cell_type")
    }

    /// Note C++ bitwise quirk: checks `(type & CELL_WATER)` against `CELL_CLEAR`,
    /// which also matches `CELL_RUBBLE` (`3`) and `CELL_BRIDGE_IMPASSABLE` (`5`).
    pub fn water_ground(target: &PathfindCell, source: &PathfindCell) -> bool {
        todo!("PKT-413: bitwise (type & CELL_WATER) check matching PathfindCell::waterGround")
    }

    pub fn ground_rubble(target: &PathfindCell, source: &PathfindCell) -> bool {
        todo!("PKT-413: Clear <-> Rubble check")
    }

    pub fn terrain(target: &PathfindCell, source: &PathfindCell) -> bool {
        todo!("PKT-413: treats Obstacle as Clear before comparing types")
    }

    pub fn crusher_ground(
        target: &PathfindCell,
        source: &PathfindCell,
        pool: &PathfindCellInfoPool,
        mode: PathfindMode,
    ) -> bool {
        todo!("PKT-413: fence obstacle <-> Clear check using is_obstacle_fence")
    }

    pub fn ground_cliff(target: &PathfindCell, source: &PathfindCell) -> bool {
        todo!("PKT-413: Clear <-> Cliff check")
    }
}

/// Safe index-based container owning a pathfinding cell slice, `PathfindCellInfoPool`,
/// and `PathfindMode` (`use_fixed_pathfinding`, `force_clean_cells`).
#[derive(Debug, Clone)]
pub struct PathfindCellGrid {
    cells: Vec<PathfindCell>,
    coords: Vec<ICoord2D>,
    pool: PathfindCellInfoPool,
    mode: PathfindMode,
}

impl PathfindCellGrid {
    /// Creates a `width x height` pathfind grid with an allocated 30,000-slot info pool.
    pub fn new(width: Int, height: Int) -> Self {
        let w = width.max(0) as usize;
        let h = height.max(0) as usize;
        let count = w * h;
        let mut coords = Vec::with_capacity(count);
        for y in 0..h {
            for x in 0..w {
                coords.push(ICoord2D {
                    x: x as Int,
                    y: y as Int,
                });
            }
        }
        let mut pool = PathfindCellInfoPool::new();
        pool.allocate_cell_infos();
        Self {
            cells: vec![PathfindCell::new(); count],
            coords,
            pool,
            mode: PathfindMode::default(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.cells.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    #[inline]
    pub fn cell(&self, idx: CellIdx) -> &PathfindCell {
        &self.cells[idx.0]
    }

    #[inline]
    pub fn coord(&self, idx: CellIdx) -> ICoord2D {
        self.coords[idx.0]
    }

    #[inline]
    pub fn pool(&self) -> &PathfindCellInfoPool {
        &self.pool
    }

    #[inline]
    pub fn pool_mut(&mut self) -> &mut PathfindCellInfoPool {
        &mut self.pool
    }

    #[inline]
    pub const fn mode(&self) -> PathfindMode {
        self.mode
    }

    #[inline]
    pub fn mode_mut(&mut self) -> &mut PathfindMode {
        &mut self.mode
    }

    pub fn reset_cell(&mut self, idx: CellIdx) {
        todo!("PKT-413: PathfindCell::reset")
    }

    pub fn set_type(&mut self, idx: CellIdx, cell_type: CellType) {
        todo!("PKT-413: PathfindCell::setType")
    }

    pub fn set_type_as_obstacle(
        &mut self,
        idx: CellIdx,
        obstacle: &ObstacleDescriptor,
        pos: ICoord2D,
    ) -> bool {
        todo!("PKT-413: PathfindCell::setTypeAsObstacle")
    }

    pub fn remove_obstacle(&mut self, idx: CellIdx, obstacle_id: ObjectId) -> bool {
        todo!("PKT-413: PathfindCell::removeObstacle")
    }

    pub fn allocate_info(&mut self, idx: CellIdx, pos: ICoord2D) -> bool {
        todo!("PKT-413: PathfindCell::allocateInfo")
    }

    pub fn release_info(&mut self, idx: CellIdx) {
        todo!("PKT-413: PathfindCell::releaseInfo")
    }

    pub fn start_pathfind(&mut self, idx: CellIdx, goal: Option<CellIdx>) -> bool {
        todo!("PKT-413: PathfindCell::startPathfind")
    }

    pub fn set_goal_unit(&mut self, idx: CellIdx, unit_id: ObjectId, pos: ICoord2D) {
        todo!("PKT-413: PathfindCell::setGoalUnit")
    }

    pub fn set_pos_unit(&mut self, idx: CellIdx, unit_id: ObjectId, pos: ICoord2D) {
        todo!("PKT-413: PathfindCell::setPosUnit")
    }

    pub fn set_goal_aircraft(&mut self, idx: CellIdx, unit_id: ObjectId, pos: ICoord2D) {
        todo!("PKT-413: PathfindCell::setGoalAircraft")
    }

    pub fn set_blocked_by_ally(&mut self, idx: CellIdx, blocked: bool) {
        todo!("PKT-413: PathfindCell::setBlockedByAlly")
    }

    pub fn is_blocked_by_ally(&self, idx: CellIdx) -> Option<bool> {
        todo!("PKT-413: Some(bool) when fixed or has_info, else None")
    }

    pub fn set_pinched(&mut self, idx: CellIdx, pinched: bool) {
        self.cells[idx.0].set_pinched(pinched);
    }

    pub fn set_zone(&mut self, idx: CellIdx, zone: UnsignedShort) {
        self.cells[idx.0].set_zone(zone);
    }

    pub fn set_layer(&mut self, idx: CellIdx, layer: PathfindLayerEnum) {
        self.cells[idx.0].set_layer(layer);
    }

    pub fn set_connect_layer(&mut self, idx: CellIdx, layer: PathfindLayerEnum) {
        self.cells[idx.0].set_connect_layer(layer);
    }

    pub fn set_cost_so_far(&mut self, idx: CellIdx, cost: UnsignedInt) {
        todo!("PKT-413: PathfindCell::setCostSoFar (truncates to u16)")
    }

    pub fn set_total_cost(&mut self, idx: CellIdx, cost: UnsignedInt) {
        todo!("PKT-413: PathfindCell::setTotalCost (truncates to u16)")
    }

    pub fn set_parent_cell(&mut self, idx: CellIdx, parent: CellIdx) {
        todo!("PKT-413: PathfindCell::setParentCell")
    }

    pub fn set_parent_cell_hierarchical(&mut self, idx: CellIdx, parent: CellIdx) {
        todo!("PKT-413: PathfindCell::setParentCellHierarchical")
    }

    pub fn clear_parent_cell(&mut self, idx: CellIdx) {
        todo!("PKT-413: PathfindCell::clearParentCell")
    }

    pub fn is_obstruction_invalid(&self, idx: CellIdx) -> bool {
        todo!("PKT-413: PathfindCell::isObstructionInvalid")
    }

    pub fn clear_obstruction(&mut self, idx: CellIdx) {
        todo!("PKT-413: PathfindCell::clearObstruction")
    }

    pub fn put_on_sorted_open_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        todo!("PKT-413: PathfindCell::putOnSortedOpenList")
    }

    pub fn remove_from_open_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        todo!("PKT-413: PathfindCell::removeFromOpenList")
    }

    pub fn put_on_closed_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        todo!("PKT-413: PathfindCell::putOnClosedList")
    }

    pub fn remove_from_closed_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        todo!("PKT-413: PathfindCell::removeFromClosedList")
    }

    pub fn release_open_list(&mut self, list: &mut PathfindCellList) -> Int {
        todo!("PKT-413: PathfindCell::releaseOpenList")
    }

    pub fn release_closed_list(&mut self, list: &mut PathfindCellList) -> Int {
        todo!("PKT-413: PathfindCell::releaseClosedList")
    }

    pub fn cost_to_goal(&self, idx: CellIdx, goal: CellIdx) -> UnsignedInt {
        todo!("PKT-413: PathfindCell::costToGoal")
    }

    pub fn cost_to_hier_goal(&self, idx: CellIdx, goal: CellIdx) -> UnsignedInt {
        todo!("PKT-413: PathfindCell::costToHierGoal")
    }

    pub fn cost_so_far(&self, idx: CellIdx, parent: Option<CellIdx>) -> UnsignedInt {
        todo!("PKT-413: PathfindCell::costSoFar")
    }

    pub fn collect_list_ids(&self, list: &PathfindCellList) -> Vec<Int> {
        let mut out = Vec::new();
        let mut cur = list.get_head();
        let mut guard = 0usize;
        let limit = self.cells.len() * 2 + 1;
        while let Some(c) = cur {
            if guard >= limit {
                break;
            }
            out.push(c.0 as Int);
            cur = self.cells[c.0]
                .info_idx
                .and_then(|ii| self.pool.get(ii))
                .and_then(|info| info.next_open)
                .and_then(|ni| self.pool.get(ni))
                .and_then(|ninfo| ninfo.cell);
            guard += 1;
        }
        out
    }
}
