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
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, PathfindMode, COST_DIAGONAL, COST_ORTHOGONAL,
    PATHFIND_CELLS_PER_FRAME,
};
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
        match (self.info_idx, other.info_idx) {
            (Some(a), Some(b)) => {
                let a_cost = pool.get(a).map_or(0, |info| info.total_cost) as i32;
                let b_cost = pool.get(b).map_or(0, |info| info.total_cost) as i32;
                (a_cost - b_cost).unsigned_abs()
            }
            _ => UnsignedInt::MAX,
        }
    }

    pub fn types_match(target: &PathfindCell, source: &PathfindCell) -> bool {
        target.cell_type == source.cell_type
    }

    /// Note C++ bitwise quirk: checks `(type & CELL_WATER)` against `CELL_CLEAR`,
    /// which also matches `CELL_RUBBLE` (`3`) and `CELL_BRIDGE_IMPASSABLE` (`5`).
    pub fn water_ground(target: &PathfindCell, source: &PathfindCell) -> bool {
        ((target.cell_type == CellType::Clear)
            && ((source.cell_type as i32 & CellType::Water as i32) != 0))
            || ((source.cell_type == CellType::Clear)
                && ((target.cell_type as i32 & CellType::Water as i32) != 0))
    }

    pub fn ground_rubble(target: &PathfindCell, source: &PathfindCell) -> bool {
        (target.cell_type == CellType::Clear && source.cell_type == CellType::Rubble)
            || (source.cell_type == CellType::Clear && target.cell_type == CellType::Rubble)
    }

    pub fn terrain(target: &PathfindCell, source: &PathfindCell) -> bool {
        let target_type = if target.cell_type == CellType::Obstacle {
            CellType::Clear
        } else {
            target.cell_type
        };
        let source_type = if source.cell_type == CellType::Obstacle {
            CellType::Clear
        } else {
            source.cell_type
        };
        target_type == source_type
    }

    pub fn crusher_ground(
        target: &PathfindCell,
        source: &PathfindCell,
        pool: &PathfindCellInfoPool,
        mode: PathfindMode,
    ) -> bool {
        let target_is_fence = target.cell_type == CellType::Obstacle
            && target.is_obstacle_fence(pool, mode);
        let source_is_fence = source.cell_type == CellType::Obstacle
            && source.is_obstacle_fence(pool, mode);
        (target.cell_type == CellType::Clear && source_is_fence)
            || (source.cell_type == CellType::Clear && target_is_fence)
    }

    pub fn ground_cliff(target: &PathfindCell, source: &PathfindCell) -> bool {
        (target.cell_type == CellType::Clear && source.cell_type == CellType::Cliff)
            || (source.cell_type == CellType::Clear && target.cell_type == CellType::Cliff)
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
        let old_info = self.cells[idx.0].info_idx.take();
        if let Some(info) = old_info {
            self.pool.release_a_cell_info(info);
        }
        self.cells[idx.0] = PathfindCell::new();
    }

    pub fn set_type(&mut self, idx: CellIdx, cell_type: CellType) {
        let blocked = if self.mode.use_fixed_pathfinding {
            self.cells[idx.0].obstacle_id != ObjectId::INVALID
        } else {
            self.cells[idx.0].info_idx
                .and_then(|ii| self.pool.get(ii))
                .is_some_and(|info| info.obstacle_id != ObjectId::INVALID)
        };
        if !blocked {
            self.cells[idx.0].cell_type = cell_type;
        } else {
            self.cells[idx.0].cell_type = CellType::Obstacle;
        }
    }

    pub fn set_type_as_obstacle(
        &mut self,
        idx: CellIdx,
        obstacle: &ObstacleDescriptor,
        pos: ICoord2D,
    ) -> bool {
        if self.cells[idx.0].cell_type != CellType::Clear
            && self.cells[idx.0].cell_type != CellType::Impassable
        {
            return false;
        }
        if obstacle.is_rubble {
            self.cells[idx.0].cell_type = CellType::Rubble;
            self.cells[idx.0].obstacle_id = ObjectId::INVALID;
            self.cells[idx.0].obstacle_is_fence = false;
            self.cells[idx.0].obstacle_is_transparent = false;
            if !self.mode.use_fixed_pathfinding {
                if let Some(info) = self.cells[idx.0].info_idx {
                    if let Some(slot) = self.pool.get_mut(info) {
                        slot.obstacle_id = ObjectId::INVALID;
                    }
                    self.release_info(idx);
                }
            }
            return true;
        }

        self.cells[idx.0].cell_type = CellType::Obstacle;
        self.cells[idx.0].obstacle_id = obstacle.id;
        self.cells[idx.0].obstacle_is_fence = obstacle.is_fence;
        self.cells[idx.0].obstacle_is_transparent = obstacle.is_transparent;
        if !self.mode.use_fixed_pathfinding {
            if self.cells[idx.0].info_idx.is_none() {
                if !self.allocate_info(idx, pos) {
                    return false;
                }
            }
            if let Some(info) = self.cells[idx.0].info_idx {
                if let Some(slot) = self.pool.get_mut(info) {
                    slot.obstacle_id = obstacle.id;
                    slot.obstacle_is_fence = obstacle.is_fence;
                    slot.obstacle_is_transparent = obstacle.is_transparent;
                }
            }
        }
        true
    }

    pub fn remove_obstacle(&mut self, idx: CellIdx, obstacle_id: ObjectId) -> bool {
        if self.cells[idx.0].cell_type == CellType::Rubble {
            self.cells[idx.0].cell_type = CellType::Clear;
        }
        let matches = if self.mode.use_fixed_pathfinding {
            self.cells[idx.0].obstacle_id == obstacle_id
        } else {
            self.cells[idx.0].info_idx
                .and_then(|ii| self.pool.get(ii))
                .is_some_and(|info| info.obstacle_id == obstacle_id)
        };
        if !matches {
            return false;
        }
        self.cells[idx.0].cell_type = CellType::Clear;
        if !self.mode.use_fixed_pathfinding {
            if let Some(info) = self.cells[idx.0].info_idx {
                if let Some(slot) = self.pool.get_mut(info) {
                    slot.obstacle_id = ObjectId::INVALID;
                }
            }
            self.release_info(idx);
        }
        self.cells[idx.0].cell_type = CellType::Clear;
        self.cells[idx.0].obstacle_id = ObjectId::INVALID;
        self.cells[idx.0].obstacle_is_fence = false;
        self.cells[idx.0].obstacle_is_transparent = false;
        true
    }

    pub fn allocate_info(&mut self, idx: CellIdx, pos: ICoord2D) -> bool {
        if self.cells[idx.0].info_idx.is_some() {
            return true;
        }
        match self.pool.get_a_cell_info(idx, pos) {
            Some(info) => {
                self.cells[idx.0].info_idx = Some(info);
                true
            }
            None => false,
        }
    }

    pub fn release_info(&mut self, idx: CellIdx) {
        if let Some(info_idx) = self.cells[idx.0].info_idx {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.path_parent = None;
            }
        }
        let cell = &self.cells[idx.0];
        if cell.cell_type == CellType::Obstacle
            || cell.flags != CellFlags::NoUnits
            || cell.aircraft_goal
        {
            return;
        }
        let Some(info_idx) = cell.info_idx else { return };
        let should_release = self.pool.get(info_idx).is_some_and(|info| {
            info.prev_open.is_none()
                && info.next_open.is_none()
                && !info.open
                && !info.closed
                && info.goal_unit_id == ObjectId::INVALID
                && info.pos_unit_id == ObjectId::INVALID
                && info.goal_aircraft_id == ObjectId::INVALID
        });
        if should_release {
            self.pool.release_a_cell_info(info_idx);
            self.cells[idx.0].info_idx = None;
        }
    }

    pub fn start_pathfind(&mut self, idx: CellIdx, goal: Option<CellIdx>) -> bool {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return false };
        let total_cost = goal.map_or(0, |goal_idx| self.cost_to_goal(idx, goal_idx) as UnsignedShort);
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.next_open = None;
            info.prev_open = None;
            info.path_parent = None;
            info.cost_so_far = 0;
            info.total_cost = total_cost;
            info.closed = false;
            info.open = !self.mode.use_fixed_pathfinding;
        }
        true
    }

    pub fn set_goal_unit(&mut self, idx: CellIdx, unit_id: ObjectId, pos: ICoord2D) {
        if unit_id == ObjectId::INVALID {
            if let Some(info_idx) = self.cells[idx.0].info_idx {
                if let Some(info) = self.pool.get_mut(info_idx) {
                    info.goal_unit_id = ObjectId::INVALID;
                    if info.pos_unit_id == ObjectId::INVALID {
                        self.cells[idx.0].flags = CellFlags::NoUnits;
                        self.release_info(idx);
                    } else {
                        self.cells[idx.0].flags = CellFlags::UnitPresentMoving;
                    }
                }
            } else if self.cells[idx.0].flags != CellFlags::NoUnits {
                self.cells[idx.0].flags = CellFlags::NoUnits;
            }
            return;
        }
        if self.cells[idx.0].info_idx.is_none() {
            self.allocate_info(idx, pos);
        }
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        let pos_unit_id = self.pool.get(info_idx).map_or(ObjectId::INVALID, |i| i.pos_unit_id);
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.goal_unit_id = unit_id;
        }
        self.cells[idx.0].flags = if unit_id == pos_unit_id {
            CellFlags::UnitPresentFixed
        } else if pos_unit_id == ObjectId::INVALID {
            CellFlags::UnitGoal
        } else {
            CellFlags::UnitGoalOtherMoving
        };
    }

    pub fn set_pos_unit(&mut self, idx: CellIdx, unit_id: ObjectId, pos: ICoord2D) {
        if unit_id == ObjectId::INVALID {
            if let Some(info_idx) = self.cells[idx.0].info_idx {
                if let Some(info) = self.pool.get_mut(info_idx) {
                    info.pos_unit_id = ObjectId::INVALID;
                    if info.goal_unit_id == ObjectId::INVALID {
                        self.cells[idx.0].flags = CellFlags::NoUnits;
                        self.release_info(idx);
                    } else {
                        self.cells[idx.0].flags = CellFlags::UnitGoal;
                    }
                }
            }
            return;
        }
        if self.cells[idx.0].info_idx.is_none() {
            self.allocate_info(idx, pos);
        }
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        let (goal_unit_id, pos_unit_id) = self.pool.get(info_idx)
            .map_or((ObjectId::INVALID, ObjectId::INVALID), |i| (i.goal_unit_id, i.pos_unit_id));
        if goal_unit_id != ObjectId::INVALID && goal_unit_id == pos_unit_id {
            return;
        }
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.pos_unit_id = unit_id;
        }
        self.cells[idx.0].flags = if unit_id == goal_unit_id {
            CellFlags::UnitPresentFixed
        } else if goal_unit_id == ObjectId::INVALID {
            CellFlags::UnitPresentMoving
        } else {
            CellFlags::UnitGoalOtherMoving
        };
    }

    pub fn set_goal_aircraft(&mut self, idx: CellIdx, unit_id: ObjectId, pos: ICoord2D) {
        if unit_id == ObjectId::INVALID {
            if let Some(info_idx) = self.cells[idx.0].info_idx {
                if let Some(info) = self.pool.get_mut(info_idx) {
                    info.goal_aircraft_id = ObjectId::INVALID;
                }
                self.cells[idx.0].aircraft_goal = false;
                self.release_info(idx);
            }
            return;
        }
        if self.cells[idx.0].info_idx.is_none() {
            self.allocate_info(idx, pos);
        }
        if let Some(info_idx) = self.cells[idx.0].info_idx {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.goal_aircraft_id = unit_id;
            }
            self.cells[idx.0].aircraft_goal = true;
        }
    }

    pub fn set_blocked_by_ally(&mut self, idx: CellIdx, blocked: bool) {
        if self.mode.use_fixed_pathfinding {
            self.cells[idx.0].blocked_by_ally = blocked;
        } else if let Some(info_idx) = self.cells[idx.0].info_idx {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.blocked_by_ally = blocked;
            }
        }
    }

    pub fn is_blocked_by_ally(&self, idx: CellIdx) -> Option<bool> {
        if self.mode.use_fixed_pathfinding {
            Some(self.cells[idx.0].blocked_by_ally)
        } else {
            self.cells[idx.0].info_idx
                .and_then(|ii| self.pool.get(ii))
                .map(|info| info.blocked_by_ally)
        }
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
        if let Some(info_idx) = self.cells[idx.0].info_idx {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.cost_so_far = cost as UnsignedShort;
            }
        }
    }

    pub fn set_total_cost(&mut self, idx: CellIdx, cost: UnsignedInt) {
        if let Some(info_idx) = self.cells[idx.0].info_idx {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.total_cost = cost as UnsignedShort;
            }
        }
    }

    pub fn set_parent_cell(&mut self, idx: CellIdx, parent: CellIdx) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        let Some(parent_info) = self.cells[parent.0].info_idx else { return };
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.path_parent = Some(parent_info);
        }
    }

    pub fn set_parent_cell_hierarchical(&mut self, idx: CellIdx, parent: CellIdx) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        let Some(parent_info) = self.cells[parent.0].info_idx else { return };
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.path_parent = Some(parent_info);
        }
    }

    pub fn clear_parent_cell(&mut self, idx: CellIdx) {
        if let Some(info_idx) = self.cells[idx.0].info_idx {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.path_parent = None;
            }
        }
    }

    pub fn is_obstruction_invalid(&self, idx: CellIdx) -> bool {
        let cell = &self.cells[idx.0];
        cell.obstacle_id != ObjectId::INVALID
            && cell.info_idx.is_none()
            && (cell.cell_type == CellType::Obstacle || cell.cell_type == CellType::Impassable)
    }

    pub fn clear_obstruction(&mut self, idx: CellIdx) {
        self.cells[idx.0].cell_type = CellType::Clear;
        self.cells[idx.0].obstacle_id = ObjectId::INVALID;
        self.cells[idx.0].obstacle_is_fence = false;
        self.cells[idx.0].obstacle_is_transparent = false;
    }

    pub fn put_on_sorted_open_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.open = true;
            info.closed = false;
        }
        if !self.mode.use_fixed_pathfinding {
            list.tail = None;
            let Some(info_idx) = self.cells[idx.0].info_idx else { return };
            if list.head.is_none() {
                if let Some(info) = self.pool.get_mut(info_idx) {
                    info.prev_open = None;
                    info.next_open = None;
                }
                list.head = Some(idx);
                return;
            }
            let mut current = list.head;
            let mut previous = None;
            let mut cell_count = 0usize;
            let current_cost = self.pool.get(info_idx).map_or(0, |i| i.total_cost);
            while let Some(current_idx) = current {
                if cell_count >= PATHFIND_CELLS_PER_FRAME as usize {
                    break;
                }
                let current_info_idx = self.cells[current_idx.0].info_idx;
                let cost = current_info_idx.and_then(|ii| self.pool.get(ii)).map_or(0, |i| i.total_cost);
                if cost > current_cost {
                    break;
                }
                cell_count += 1;
                previous = Some(current_idx);
                current = current_info_idx
                    .and_then(|ii| self.pool.get(ii))
                    .and_then(|i| i.next_open)
                    .and_then(|ni| self.pool.get(ni))
                    .and_then(|n| n.cell);
            }
            if let Some(current_idx) = current {
                let current_prev = self.cells[current_idx.0].info_idx
                    .and_then(|ii| self.pool.get(ii))
                    .and_then(|i| i.prev_open);
                if let Some(prev_info) = current_prev {
                    if let Some(slot) = self.pool.get_mut(prev_info) {
                        slot.next_open = Some(info_idx);
                    }
                } else {
                    list.head = Some(idx);
                }
                if let Some(info) = self.pool.get_mut(info_idx) {
                    info.prev_open = current_prev;
                    info.next_open = self.cells[current_idx.0].info_idx;
                }
                if let Some(slot) = self.pool.get_mut(self.cells[current_idx.0].info_idx.unwrap()) {
                    slot.prev_open = Some(info_idx);
                }
            } else if let Some(previous_idx) = previous {
                if let Some(previous_info) = self.cells[previous_idx.0].info_idx {
                    if let Some(slot) = self.pool.get_mut(previous_info) {
                        slot.next_open = Some(info_idx);
                    }
                }
                if let Some(info) = self.pool.get_mut(info_idx) {
                    info.prev_open = self.cells[previous_idx.0].info_idx;
                    info.next_open = None;
                }
            }
            return;
        }

        if list.can_reverse_sort(&self.cells, &self.pool, idx) {
            self.reverse_sort_open(list, idx);
        } else {
            self.forward_sort_open(list, idx);
        }
    }

    pub fn remove_from_open_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        let (next, prev) = self.pool.get(info_idx).map_or((None, None), |i| (i.next_open, i.prev_open));
        if let Some(next_info) = next {
            if let Some(info) = self.pool.get_mut(next_info) {
                info.prev_open = prev;
            }
        } else {
            list.tail = prev.and_then(|pi| self.pool.get(pi)).and_then(|i| i.cell);
        }
        if let Some(prev_info) = prev {
            if let Some(info) = self.pool.get_mut(prev_info) {
                info.next_open = next;
            }
        } else {
            list.head = next.and_then(|ni| self.pool.get(ni)).and_then(|i| i.cell);
        }
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.open = false;
            info.next_open = None;
            info.prev_open = None;
        }
    }

    fn forward_sort_open(&mut self, list: &mut PathfindCellList, idx: CellIdx) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        if list.head.is_none() {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.prev_open = None;
                info.next_open = None;
            }
            list.head = Some(idx);
            list.tail = Some(idx);
            return;
        }
        let total = self.pool.get(info_idx).map_or(0, |i| i.total_cost);
        let head = list.head.unwrap();
        let head_total = self.cells[head.0].info_idx
            .and_then(|ii| self.pool.get(ii))
            .map_or(0, |i| i.total_cost);
        if total < head_total {
            if let Some(head_info) = self.cells[head.0].info_idx {
                if let Some(slot) = self.pool.get_mut(head_info) {
                    slot.prev_open = Some(info_idx);
                }
            }
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.prev_open = None;
                info.next_open = self.cells[head.0].info_idx;
            }
            list.head = Some(idx);
            return;
        }
        let mut current = head;
        loop {
            let current_info = match self.cells[current.0].info_idx {
                Some(ii) => ii,
                None => break,
            };
            let next = self.pool.get(current_info).and_then(|i| i.next_open);
            let next_total = next
                .and_then(|ni| self.pool.get(ni))
                .map_or(0xffff, |i| i.total_cost);
            if next.is_none() || next_total > total {
                break;
            }
            current = next.and_then(|ni| self.pool.get(ni)).and_then(|i| i.cell).unwrap_or(current);
        }
        let current_info = self.cells[current.0].info_idx.unwrap();
        let next = self.pool.get(current_info).and_then(|i| i.next_open);
        if let Some(next_info) = next {
            if let Some(slot) = self.pool.get_mut(next_info) {
                slot.prev_open = Some(info_idx);
            }
        } else {
            list.tail = Some(idx);
        }
        if let Some(slot) = self.pool.get_mut(current_info) {
            slot.next_open = Some(info_idx);
        }
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.prev_open = Some(current_info);
            info.next_open = next;
        }
    }

    fn reverse_sort_open(&mut self, list: &mut PathfindCellList, idx: CellIdx) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        if list.tail.is_none() {
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.prev_open = None;
                info.next_open = None;
            }
            list.tail = Some(idx);
            list.head = Some(idx);
            return;
        }
        let tail = list.tail.unwrap();
        let total = self.pool.get(info_idx).map_or(0, |i| i.total_cost);
        let tail_total = self.cells[tail.0].info_idx
            .and_then(|ii| self.pool.get(ii))
            .map_or(0, |i| i.total_cost);
        if total >= tail_total {
            let tail_info = self.cells[tail.0].info_idx.unwrap();
            if let Some(slot) = self.pool.get_mut(tail_info) {
                slot.next_open = Some(info_idx);
            }
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.prev_open = Some(tail_info);
                info.next_open = None;
            }
            list.tail = Some(idx);
            return;
        }
        let mut current = tail;
        loop {
            let current_info = match self.cells[current.0].info_idx {
                Some(ii) => ii,
                None => break,
            };
            let prev = self.pool.get(current_info).and_then(|i| i.prev_open);
            let prev_total = prev
                .and_then(|pi| self.pool.get(pi))
                .map_or(0, |i| i.total_cost);
            if prev.is_none() || prev_total <= total {
                break;
            }
            current = prev.and_then(|pi| self.pool.get(pi)).and_then(|i| i.cell).unwrap_or(current);
        }
        let current_info = self.cells[current.0].info_idx.unwrap();
        let prev = self.pool.get(current_info).and_then(|i| i.prev_open);
        if let Some(prev_info) = prev {
            if let Some(slot) = self.pool.get_mut(prev_info) {
                slot.next_open = Some(info_idx);
            }
        } else {
            list.head = Some(idx);
        }
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.prev_open = prev;
            info.next_open = Some(current_info);
        }
        if let Some(slot) = self.pool.get_mut(current_info) {
            slot.prev_open = Some(info_idx);
        }
    }

    pub fn put_on_closed_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        if self.pool.get(info_idx).is_some_and(|info| info.closed) {
            return;
        }
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.closed = true;
            info.open = false;
            info.prev_open = None;
            info.next_open = list.head
                .and_then(|head| self.cells[head.0].info_idx);
        }
        if let Some(head) = list.head {
            if let Some(head_info) = self.cells[head.0].info_idx {
                if let Some(slot) = self.pool.get_mut(head_info) {
                    slot.prev_open = Some(info_idx);
                }
            }
        }
        list.head = Some(idx);
    }

    pub fn remove_from_closed_list(&mut self, idx: CellIdx, list: &mut PathfindCellList) {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return };
        let (next, prev) = self.pool.get(info_idx).map_or((None, None), |i| (i.next_open, i.prev_open));
        if let Some(next_info) = next {
            if let Some(info) = self.pool.get_mut(next_info) {
                info.prev_open = prev;
            }
        }
        if let Some(prev_info) = prev {
            if let Some(info) = self.pool.get_mut(prev_info) {
                info.next_open = next;
            }
        } else {
            list.head = next.and_then(|ni| self.pool.get(ni)).and_then(|i| i.cell);
        }
        if let Some(info) = self.pool.get_mut(info_idx) {
            info.closed = false;
            info.next_open = None;
            info.prev_open = None;
        }
    }

    pub fn release_open_list(&mut self, list: &mut PathfindCellList) -> Int {
        let mut count = 0i32;
        while let Some(cur) = list.head {
            count = count.wrapping_add(1);
            let Some(info_idx) = self.cells[cur.0].info_idx else {
                list.reset(None);
                break;
            };
            let next = self.pool.get(info_idx).and_then(|info| info.next_open)
                .and_then(|ni| self.pool.get(ni))
                .and_then(|info| info.cell);
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.next_open = None;
                info.prev_open = None;
                info.open = false;
            }
            self.release_info(cur);
            list.head = next;
        }
        list.tail = None;
        count
    }

    pub fn release_closed_list(&mut self, list: &mut PathfindCellList) -> Int {
        let mut count = 0i32;
        while let Some(cur) = list.head {
            count = count.wrapping_add(1);
            let Some(info_idx) = self.cells[cur.0].info_idx else {
                list.reset(None);
                break;
            };
            let next = self.pool.get(info_idx).and_then(|info| info.next_open)
                .and_then(|ni| self.pool.get(ni))
                .and_then(|info| info.cell);
            if let Some(info) = self.pool.get_mut(info_idx) {
                info.next_open = None;
                info.prev_open = None;
                info.closed = false;
            }
            self.release_info(cur);
            list.head = next;
        }
        list.reset(None);
        count
    }

    pub fn cost_to_goal(&self, idx: CellIdx, goal: CellIdx) -> UnsignedInt {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return 100_000 };
        let Some(goal_info) = self.cells[goal.0].info_idx else { return 100_000 };
        let (self_pos, goal_pos) = match (self.pool.get(info_idx), self.pool.get(goal_info)) {
            (Some(a), Some(b)) => (a.pos, b.pos),
            _ => return 100_000,
        };
        let mut dx = (self_pos.x - goal_pos.x).abs();
        let mut dy = (self_pos.y - goal_pos.y).abs();
        if dx > dy {
            std::mem::swap(&mut dx, &mut dy);
        }
        let cost = (COST_ORTHOGONAL as i32).wrapping_mul(dy)
            .wrapping_add((COST_ORTHOGONAL as i32).wrapping_mul(dx) / 2);
        cost as UnsignedInt
    }

    pub fn cost_to_hier_goal(&self, idx: CellIdx, goal: CellIdx) -> UnsignedInt {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return 100_000 };
        let Some(goal_info) = self.cells[goal.0].info_idx else { return 100_000 };
        let (self_pos, goal_pos) = match (self.pool.get(info_idx), self.pool.get(goal_info)) {
            (Some(a), Some(b)) => (a.pos, b.pos),
            _ => return 100_000,
        };
        let dx = self_pos.x.wrapping_sub(goal_pos.x);
        let dy = self_pos.y.wrapping_sub(goal_pos.y);
        let dx2 = (dx as i64).wrapping_mul(dx as i64);
        let dy2 = (dy as i64).wrapping_mul(dy as i64);
        let dist_sq = (dx2.wrapping_add(dy2)) as f32;
        let value = 10.0_f32 * zh_common::math::gm_sqrtf(dist_sq) + 0.5_f32;
        zh_common::math::real_to_int_floor(value) as UnsignedInt
    }

    pub fn cost_so_far(&self, idx: CellIdx, parent: Option<CellIdx>) -> UnsignedInt {
        let Some(info_idx) = self.cells[idx.0].info_idx else { return 0 };
        let Some(parent_idx) = parent else { return 0 };
        let self_pos = self.pool.get(info_idx).map_or(ICoord2D::default(), |i| i.pos);
        let parent_info = match self.cells[parent_idx.0].info_idx {
            Some(ii) => ii,
            None => return 0,
        };
        let parent_pos = self.pool.get(parent_info).map_or(ICoord2D::default(), |i| i.pos);
        let parent_cost = self.pool.get(parent_info).map_or(0, |i| i.cost_so_far);
        let prev_dir = ICoord2D {
            x: parent_pos.x.wrapping_sub(self_pos.x),
            y: parent_pos.y.wrapping_sub(self_pos.y),
        };
        let mut cost: UnsignedInt = if prev_dir.x == 0 || prev_dir.y == 0 {
            (parent_cost as UnsignedInt).wrapping_add(COST_ORTHOGONAL as u32)
        } else {
            (parent_cost as UnsignedInt).wrapping_add(COST_DIAGONAL as u32)
        };
        if self.cells[idx.0].pinched {
            cost = cost.wrapping_add(COST_DIAGONAL as u32);
        }

        let grand_info = self.pool.get(parent_info)
            .and_then(|i| i.path_parent);
        if let Some(grand_info) = grand_info {
            let grand_pos = self.pool.get(grand_info).map_or(ICoord2D::default(), |i| i.pos);
            let dir = ICoord2D {
                x: grand_pos.x.wrapping_sub(parent_pos.x),
                y: grand_pos.y.wrapping_sub(parent_pos.y),
            };
            if dir.x != prev_dir.x || dir.y != prev_dir.y {
                let dot = (dir.x as i64).wrapping_mul(prev_dir.x as i64)
                    .wrapping_add((dir.y as i64).wrapping_mul(prev_dir.y as i64));
                let turns = if dot > 0 { 4 } else if dot == 0 { 8 } else { 16 };
                cost = cost.wrapping_add(turns);
            }
        }
        cost
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
