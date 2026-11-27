//! `PKT-155`: Common map/terrain filename length limit
//! (`GeneralsMD/Code/GameEngine/Include/Common/Terrain.h`).

use zh_common::types::{Bool, Int, Real, FALSE, TRUE};

/// Maximum map filename length including extension (`MAX_TERRAIN_NAME_LEN`).
pub const MAX_TERRAIN_NAME_LEN: usize = 64;

/// `TerrainLogic::findClosestEdgePoint`.
pub fn find_closest_edge_point(
    terrain: &dyn crate::object::thing::ThingTerrainAccess,
    extent: &zh_common::math::Region3D,
    closest_to: &zh_common::math::Coord3D,
) -> zh_common::math::Coord3D {
    // Top, right, bottom, left; the first of equals wins.
    let distances = [
        (closest_to.y - extent.lo.y).abs(),
        (closest_to.x - extent.hi.x).abs(),
        (closest_to.y - extent.hi.y).abs(),
        (closest_to.x - extent.lo.x).abs(),
    ];
    let mut best = 0;
    for i in 1..4 {
        if distances[i] < distances[best] {
            best = i;
        }
    }
    let mut point = *closest_to;
    match best {
        0 => point.y = extent.lo.y,
        1 => point.x = extent.hi.x,
        2 => point.y = extent.hi.y,
        _ => point.x = extent.lo.x,
    }
    point.z = terrain.get_ground_height(point.x, point.y, None);
    point
}

/// `TerrainLogic::findFarthestEdgePoint`: a corner of the map. The halves are of
/// the width and height, not offset by the low corner.
pub fn find_farthest_edge_point(
    terrain: &dyn crate::object::thing::ThingTerrainAccess,
    extent: &zh_common::math::Region3D,
    farthest_from: &zh_common::math::Coord3D,
) -> zh_common::math::Coord3D {
    let mut point = *farthest_from;
    point.x = if farthest_from.x < (extent.hi.x - extent.lo.x) / 2.0 {
        extent.hi.x
    } else {
        extent.lo.x
    };
    point.y = if farthest_from.y < (extent.hi.y - extent.lo.y) / 2.0 {
        extent.hi.y
    } else {
        extent.lo.y
    };
    point.z = terrain.get_ground_height(point.x, point.y, None);
    point
}

/// `MAP_XY_FACTOR`: the side of a height map cell in world units.
const CELL_SIZE: Real = 10.0;
/// `MAP_HEIGHT_SCALE`.
const HEIGHT_SCALE: Real = CELL_SIZE / 16.0;
/// `PATHFIND_CLIFF_SLOPE_LIMIT_F`.
const CLIFF_SLOPE_LIMIT: Real = 9.8;

/// The cliff flags of a height map (`WorldHeightMap`), which answer
/// `TerrainLogic::isCliffCell`. A terrain implements
/// [`crate::object::thing::ThingTerrainAccess::is_cliff_cell`] with one.
#[derive(Debug, Clone, Default)]
pub struct CliffMap {
    width: Int,
    height: Int,
    border: Int,
    /// Bytes per row of `bits`.
    stride: Int,
    bits: Vec<u8>,
}

impl CliffMap {
    /// A map of `width` by `height` height samples with no cliffs. `border` samples
    /// on each side lie outside the playable area.
    pub fn new(width: Int, height: Int, border: Int) -> Self {
        let width = width.max(0);
        let height = height.max(0);
        let stride = (width + 7) / 8;
        Self {
            width,
            height,
            border,
            stride,
            bits: vec![0; (stride * height) as usize],
        }
    }

    /// `initCliffFlagsFromHeights`, for maps that store no flags. `heights` is row by row.
    pub fn from_heights(width: Int, height: Int, border: Int, heights: &[u8]) -> Self {
        let mut map = Self::new(width, height, border);
        let sample = |x: Int, y: Int| -> Real {
            let ndx = y * map.width + x;
            Real::from(heights.get(ndx as usize).copied().unwrap_or(0)) * HEIGHT_SCALE
        };
        let mut cliffs = Vec::new();
        for x in 0..map.width - 1 {
            for y in 0..map.height - 1 {
                let h = [
                    sample(x, y),
                    sample(x + 1, y),
                    sample(x, y + 1),
                    sample(x + 1, y + 1),
                ];
                let low = h.iter().fold(h[0], |a, &b| if a > b { b } else { a });
                let high = h.iter().fold(h[0], |a, &b| if a < b { b } else { a });
                if high - low > CLIFF_SLOPE_LIMIT {
                    cliffs.push((x, y));
                }
            }
        }
        for (x, y) in cliffs {
            map.set_cliff_state(x, y, TRUE);
        }
        map
    }

    fn bit(&self, x: Int, y: Int) -> Option<(usize, u8)> {
        if x < 0 || y < 0 || x >= self.width || y >= self.height {
            return None;
        }
        Some(((y * self.stride + (x >> 3)) as usize, 1 << (x & 7)))
    }

    /// `setCliffState`.
    pub fn set_cliff_state(&mut self, x: Int, y: Int, cliff: Bool) {
        if let Some((byte, mask)) = self.bit(x, y) {
            if cliff != FALSE {
                self.bits[byte] |= mask;
            } else {
                self.bits[byte] &= !mask;
            }
        }
    }

    /// The stored flag rows and their length in bytes, as a map file carries them.
    pub(crate) fn rows_mut(&mut self) -> (usize, &mut [u8]) {
        (self.stride as usize, &mut self.bits)
    }

    /// Shrinks the sampled area without moving the stored rows, as a version 1
    /// blend chunk does after the flags exist.
    pub(crate) fn set_sample_size(&mut self, width: Int, height: Int) {
        self.width = width;
        self.height = height;
    }

    /// `getCliffState`.
    pub fn get_cliff_state(&self, x: Int, y: Int) -> Bool {
        match self.bit(x, y) {
            Some((byte, mask)) if self.bits[byte] & mask != 0 => TRUE,
            _ => FALSE,
        }
    }

    /// `BaseHeightMapRenderObjClass::isCliffCell`: the flag of the cell holding the
    /// point, clamped to the map.
    pub fn is_cliff_cell(&self, x: Real, y: Real) -> Bool {
        let cell = |world: Real, extent: Int| {
            let i = ((world / CELL_SIZE) as Int)
                .saturating_add(self.border)
                .max(0);
            if i >= extent - 1 {
                extent - 2
            } else {
                i
            }
        };
        self.get_cliff_state(cell(x, self.width), cell(y, self.height))
    }
}

/// `PartitionManager::estimateTerrainExtremesAlongLine`, the high extreme only: the cells
/// of the Bresenham line between the two points, each sampled as `calcHeights` caches it.
/// `None` when none of them is on the map. The cache is not kept, so terrain changed after
/// the map loaded is seen here but not by the original.
pub fn estimate_highest_terrain_along_line(
    grid: &crate::spatial::partition_grid::PartitionGrid,
    terrain: &dyn crate::object::thing::ThingTerrainAccess,
    pos: &zh_common::math::Coord3D,
    pos_other: &zh_common::math::Coord3D,
) -> Option<zh_common::types::Real> {
    use zh_common::types::{Int, Real};
    const HUGE_DIST: Real = 1_000_000.0;
    const MAP_XY_FACTOR: Real = 10.0;

    let (mut x, mut y) = grid.world_to_cell(pos.x, pos.y);
    let (end_x, end_y) = grid.world_to_cell(pos_other.x, pos_other.y);
    let delta_x = end_x.wrapping_sub(x).wrapping_abs();
    let delta_y = end_y.wrapping_sub(y).wrapping_abs();
    let step_x: Int = if end_x >= x { 1 } else { -1 };
    let step_y: Int = if end_y >= y { 1 } else { -1 };
    let (mut xinc1, mut xinc2, mut yinc1, mut yinc2) = (step_x, step_x, step_y, step_y);
    let (den, mut num, numadd, numpixels) = if delta_x >= delta_y {
        xinc1 = 0;
        yinc2 = 0;
        (delta_x, delta_x / 2, delta_y, delta_x)
    } else {
        xinc2 = 0;
        yinc1 = 0;
        (delta_y, delta_y / 2, delta_x, delta_y)
    };

    let cell_size = grid.cell_size();
    let lo = grid.world_extents().lo;
    let step = cell_size / zh_common::math::gm_ceilf(cell_size / MAP_XY_FACTOR);
    let mut highest = -HUGE_DIST;
    let mut valid = false;
    for _ in 0..=numpixels {
        if grid.cell_index(x, y).is_some() {
            valid = true;
            let x_base = lo.x + (x as Real) * cell_size;
            let y_base = lo.y + (y as Real) * cell_size;
            let mut cell_high = -HUGE_DIST;
            let mut yy: Real = 0.0;
            while yy <= cell_size {
                let mut xx: Real = 0.0;
                while xx <= cell_size {
                    let h = terrain.get_ground_height(x_base + xx, y_base + yy, None);
                    if h > cell_high {
                        cell_high = h;
                    }
                    xx += step;
                }
                yy += step;
            }
            if cell_high > highest {
                highest = cell_high;
            }
        }
        num += numadd;
        if num >= den {
            num -= den;
            x += xinc1;
            y += yinc1;
        }
        x += xinc2;
        y += yinc2;
    }
    valid.then_some(highest)
}
