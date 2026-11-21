//! `PKT-415`: Open/closed intrusive cell list header (`PathfindCellList.h` / `.cpp`).
//!
//! Reference C++ packet: `packets/phase6/PKT-413_PathfindCell.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt413_pathfind_cell.rs`

#![allow(unused_variables, dead_code)]

use crate::pathfinder::pathfind_cell::PathfindCell;
use crate::pathfinder::pathfind_cell_info::{CellIdx, PathfindCellInfoPool};

/// Head/tail container for A* open and closed cell lists (`PathfindCellList`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PathfindCellList {
    pub head: Option<CellIdx>,
    pub tail: Option<CellIdx>,
}

impl PathfindCellList {
    /// Creates an empty `PathfindCellList`.
    pub const fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    /// Resets `head` and `tail` (`tail` is always cleared).
    #[inline]
    pub fn reset(&mut self, new_head: Option<CellIdx>) {
        self.head = new_head;
        self.tail = None;
    }

    #[inline]
    pub const fn get_head(&self) -> Option<CellIdx> {
        self.head
    }

    #[inline]
    pub const fn get_tail(&self) -> Option<CellIdx> {
        self.tail
    }

    #[inline]
    pub const fn empty(&self) -> bool {
        self.head.is_none()
    }

    /// Returns `true` when `current_cell`'s `total_cost` is closer to `tail` than `head` (`>`).
    pub fn can_reverse_sort(
        &self,
        cells: &[PathfindCell],
        pool: &PathfindCellInfoPool,
        current_cell: CellIdx,
    ) -> bool {
        let Some(head) = self.head else { return false };
        let Some(tail) = self.tail else { return false };
        let Some(head_info) = cells[head.0].info_idx.and_then(|idx| pool.get(idx)) else {
            return false;
        };
        let Some(tail_info) = cells[tail.0].info_idx.and_then(|idx| pool.get(idx)) else {
            return false;
        };
        let Some(cur_info) = cells[current_cell.0].info_idx.and_then(|idx| pool.get(idx)) else {
            return false;
        };
        let head_diff = (head_info.total_cost as i32 - cur_info.total_cost as i32).unsigned_abs();
        let tail_diff = (tail_info.total_cost as i32 - cur_info.total_cost as i32).unsigned_abs();
        head_diff > tail_diff
    }
}
