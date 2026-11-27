//! `PKT-414`: Slab pool of per-active-cell A* search records (`PathfindCellInfo.h` / `.cpp`).
//!
//! Reference C++ packet: `packets/phase6/PKT-413_PathfindCell.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt413_pathfind_cell.rs`

#![allow(unused_variables, dead_code)]

use zh_common::math::ICoord2D;
use zh_common::types::{ObjectId, UnsignedInt, UnsignedShort};

/// Fixed pool capacity matching `CELL_INFOS_TO_ALLOCATE` (`PathfindCellInfo.cpp`).
pub const CELL_INFOS_TO_ALLOCATE: UnsignedInt = 30_000;

/// Grid-local cell index replacing `PathfindCell*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellIdx(pub usize);

/// Pool-local index replacing `PathfindCellInfo*`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InfoIdx(pub usize);

/// A* transient search and occupancy payload (`PathfindCellInfo`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathfindCellInfo {
    pub next_open: Option<InfoIdx>,
    pub prev_open: Option<InfoIdx>,
    pub path_parent: Option<InfoIdx>,
    pub next_free: Option<InfoIdx>,
    pub cell: Option<CellIdx>,
    pub total_cost: UnsignedShort,
    pub cost_so_far: UnsignedShort,
    pub pos: ICoord2D,
    pub goal_unit_id: ObjectId,
    pub pos_unit_id: ObjectId,
    pub goal_aircraft_id: ObjectId,
    pub obstacle_id: ObjectId,
    pub is_free: bool,
    pub blocked_by_ally: bool,
    pub obstacle_is_fence: bool,
    pub obstacle_is_transparent: bool,
    pub open: bool,
    pub closed: bool,
}

impl Default for PathfindCellInfo {
    fn default() -> Self {
        Self {
            next_open: None,
            prev_open: None,
            path_parent: None,
            next_free: None,
            cell: None,
            total_cost: 0,
            cost_so_far: 0,
            pos: ICoord2D { x: 0, y: 0 },
            goal_unit_id: ObjectId::INVALID,
            pos_unit_id: ObjectId::INVALID,
            goal_aircraft_id: ObjectId::INVALID,
            obstacle_id: ObjectId::INVALID,
            is_free: true,
            blocked_by_ally: false,
            obstacle_is_fence: false,
            obstacle_is_transparent: false,
            open: false,
            closed: false,
        }
    }
}

/// Deterministic slab pool replacing static `s_infoArray` / `s_firstFree`.
#[derive(Debug, Clone)]
pub struct PathfindCellInfoPool {
    infos: Vec<PathfindCellInfo>,
    first_free: Option<InfoIdx>,
    free_count: UnsignedInt,
}

impl Default for PathfindCellInfoPool {
    fn default() -> Self {
        Self::new()
    }
}

impl PathfindCellInfoPool {
    /// Creates an unallocated pool; call `allocate_cell_infos()` before acquiring entries.
    pub fn new() -> Self {
        Self {
            infos: Vec::new(),
            first_free: None,
            free_count: 0,
        }
    }

    /// Allocates the `CELL_INFOS_TO_ALLOCATE` (`30,000`) free-list slab.
    pub fn allocate_cell_infos(&mut self) {
        if !self.infos.is_empty() {
            return;
        }
        let capacity = CELL_INFOS_TO_ALLOCATE as usize;
        self.infos = vec![PathfindCellInfo::default(); capacity];
        for i in 0..capacity {
            self.infos[i].is_free = true;
            self.infos[i].next_free = if i + 1 < capacity {
                Some(InfoIdx(i + 1))
            } else {
                None
            };
        }
        self.first_free = Some(InfoIdx(0));
        self.free_count = CELL_INFOS_TO_ALLOCATE;
    }

    /// Releases the slab pool (`releaseCellInfos`).
    pub fn release_cell_infos(&mut self) {
        self.infos.clear();
        self.first_free = None;
        self.free_count = 0;
    }

    /// Clears open/closed list links across all allocated infos (`forceCleanPathFindCellInfos`).
    pub fn force_clean_pathfind_cell_infos(&mut self) {
        let limit = (CELL_INFOS_TO_ALLOCATE - 1) as usize;
        for info in self.infos.iter_mut().take(limit) {
            info.next_open = None;
            info.prev_open = None;
            info.open = false;
            info.closed = false;
        }
    }

    /// Acquires and zero-initializes one `PathfindCellInfo` (`getACellInfo`).
    pub fn get_a_cell_info(&mut self, cell: CellIdx, pos: ICoord2D) -> Option<InfoIdx> {
        let idx = self.first_free?;
        let info = self.infos.get_mut(idx.0)?;
        if !info.is_free {
            return None;
        }
        self.first_free = info.next_free;
        self.free_count = self.free_count.saturating_sub(1);
        *info = PathfindCellInfo {
            next_free: None,
            is_free: false,
            cell: Some(cell),
            pos,
            ..PathfindCellInfo::default()
        };
        Some(idx)
    }

    /// Returns an entry to the free list head (`releaseACellInfo`).
    /// `releaseACellInfo` only pushes the record on the free list. Everything
    /// else stays: retail list links that dangle read it.
    pub fn release_a_cell_info(&mut self, info: InfoIdx) {
        if let Some(slot) = self.infos.get_mut(info.0) {
            if slot.is_free {
                return;
            }
            slot.is_free = true;
            slot.path_parent = None;
            slot.next_free = self.first_free;
            self.first_free = Some(info);
            self.free_count = self.free_count.wrapping_add(1);
        }
    }

    /// The head of the free list, `s_firstFree`.
    #[inline]
    pub const fn first_free(&self) -> Option<InfoIdx> {
        self.first_free
    }

    #[inline]
    pub const fn free_count(&self) -> UnsignedInt {
        self.free_count
    }

    #[inline]
    pub fn get(&self, idx: InfoIdx) -> Option<&PathfindCellInfo> {
        self.infos.get(idx.0)
    }

    #[inline]
    pub fn get_mut(&mut self, idx: InfoIdx) -> Option<&mut PathfindCellInfo> {
        self.infos.get_mut(idx.0)
    }
}
