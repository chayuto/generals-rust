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
