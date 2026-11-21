//! `PKT-416`: Pathfinder tuning constants, layer enum, and retail-compatibility flags
//! (`Core/GameEngine/Include/GameLogic/Pathfinder/PathfindConstants.h` & `.cpp`).
//!
//! Reference C++ packet: `packets/phase6/PKT-413_PathfindCell.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt413_pathfind_cell.rs`

use zh_common::types::{Int, Real, UnsignedInt};

pub const MAX_WALL_PIECES: Int = 128;
pub const PATHFIND_QUEUE_LEN: Int = 512;
pub const MOVE_ALLIES_MAX_DEPTH: Int = 2;

/// How close a unit has to be in Z to interact with a bridge/tunnel layer.
pub const LAYER_Z_CLOSE_ENOUGH_F: Real = 10.0_f32;

pub const PATHFIND_CELL_SIZE: UnsignedInt = 10;
pub const PATHFIND_CELL_SIZE_F: Real = 10.0_f32;

pub const ZONE_UPDATE_FREQUENCY: UnsignedInt = 300;
pub const MAX_CELL_COUNT: UnsignedInt = 500;
pub const MAX_ADJUSTMENT_CELL_COUNT: UnsignedInt = 400;
pub const MAX_SAFE_PATH_CELL_COUNT: UnsignedInt = 2000;

/// Maximum cells expanded per pathfind frame in retail insertion sort (`PATHFIND_CELLS_PER_FRAME`).
pub const PATHFIND_CELLS_PER_FRAME: UnsignedInt = 5000;

pub const COST_ORTHOGONAL: Int = 10;
pub const COST_DIAGONAL: Int = 14;
pub const COST_TO_DISTANCE_FACTOR: Real = 1.0_f32 / 10.0_f32;
pub const COST_TO_DISTANCE_FACTOR_SQR: Real = COST_TO_DISTANCE_FACTOR * COST_TO_DISTANCE_FACTOR;

/// Cell layer enum (`GameType.h`: `enum PathfindLayerEnum : Int`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PathfindLayerEnum {
    Invalid = 0,
    #[default]
    Ground = 1,
    Wall = 15,
}

impl PathfindLayerEnum {
    pub const LAST: Self = Self::Wall;

    /// Converts a raw integer value into `PathfindLayerEnum`.
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::Ground),
            15 => Some(Self::Wall),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Runtime failover switches replacing globals `s_useFixedPathfinding` and `s_forceCleanCells`
/// (`RETAIL_COMPATIBLE_PATHFINDING = 1`, `RETAIL_COMPATIBLE_PATHFINDING_ALLOCATION = 1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PathfindMode {
    pub use_fixed_pathfinding: bool,
    pub force_clean_cells: bool,
}
