//! `PathNode` (`PKT-419`): one node of a [`Path`](super::path::Path).
//!
//! The reference links nodes by pointer. Here they live in the path's list and
//! link by index; a path never removes a node.
//!
//! Golden test: `crates/zh_logic/tests/golden_s3_path.rs`.

use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{Int, Real};

use crate::pathfinder::pathfind_constants::PathfindLayerEnum;

/// Index of a node in its path.
pub type PathNodeIdx = usize;

#[derive(Debug, Clone)]
pub struct PathNode {
    pub(super) next_opti: Option<PathNodeIdx>,
    pub(super) next: Option<PathNodeIdx>,
    pub(super) prev: Option<PathNodeIdx>,
    pub(super) pos: Coord3D,
    pub(super) layer: PathfindLayerEnum,
    pub(super) can_optimize: bool,
    pub(super) next_opti_dist: Real,
    pub(super) next_opti_dir: Coord2D,
    /// `m_id`, written by `Path::xfer` and kept between saves.
    pub(super) id: Int,
}

impl Default for PathNode {
    fn default() -> Self {
        Self {
            next_opti: None,
            next: None,
            prev: None,
            pos: Coord3D::default(),
            layer: PathfindLayerEnum::Invalid,
            can_optimize: false,
            next_opti_dist: 0.0,
            next_opti_dir: Coord2D::default(),
            id: -1,
        }
    }
}

impl PathNode {
    pub fn position(&self) -> &Coord3D {
        &self.pos
    }

    pub fn layer(&self) -> PathfindLayerEnum {
        self.layer
    }

    pub fn can_optimize(&self) -> bool {
        self.can_optimize
    }

    pub fn next(&self) -> Option<PathNodeIdx> {
        self.next
    }

    pub fn previous(&self) -> Option<PathNodeIdx> {
        self.prev
    }

    pub fn next_optimized(&self) -> Option<PathNodeIdx> {
        self.next_opti
    }

    /// The unit direction and 2D length of the optimized segment, as last set.
    pub fn optimized_segment(&self) -> (Coord2D, Real) {
        (self.next_opti_dir, self.next_opti_dist)
    }
}
