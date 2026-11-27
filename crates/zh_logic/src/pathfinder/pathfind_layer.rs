//! `PKT-417`: `PathfindLayer`, a bridge or the wall as a rectangle of cells over
//! the ground grid (`PathfindLayer.cpp`).
//!
//! Golden test: `crates/zh_logic/tests/golden_s2_pathfind_map.rs`.
//!
//! The layer's cells live in the pathfinder's grid after the ground cells, so
//! they share its info pool. `Bridge*` is the deck shape copied at `init`.

use zh_common::math::gamemath::{real_to_int_ceil, real_to_int_floor};
use zh_common::math::geometry::GeometryType;
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, ICoord2D, IRegion2D, Region2D};
use zh_common::types::{Int, ObjectId, Real};

use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::ai_pathfind::Footprint;
use crate::pathfinder::pathfind_cell::{CellType, PathfindCellGrid};
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::pathfinder::pathfind_zone_manager::{PathfindZoneManager, PathfindZoneMap};
use crate::rules::combat_types::LocomotorSurfaceTypeMask;
use crate::terrain::bridge::{BridgeGeometry, LAYER_Z_CLOSE_ENOUGH_F};

/// `m_bridge` of a layer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayerBridge {
    pub geometry: BridgeGeometry,
    pub object_id: ObjectId,
}

#[derive(Debug, Clone)]
pub struct PathfindLayer {
    /// First grid index of the `width * height` cells, column-major.
    base: Option<usize>,
    width: Int,
    height: Int,
    x_origin: Int,
    y_origin: Int,
    start_cell: ICoord2D,
    end_cell: ICoord2D,
    layer: PathfindLayerEnum,
    zone: Int,
    bridge: Option<LayerBridge>,
    destroyed: bool,
}

impl Default for PathfindLayer {
    fn default() -> Self {
        Self {
            base: None,
            width: 0,
            height: 0,
            x_origin: 0,
            y_origin: 0,
            start_cell: ICoord2D::new(-1, -1),
            end_cell: ICoord2D::new(-1, -1),
            layer: PathfindLayerEnum::Ground,
            zone: 0,
            bridge: None,
            destroyed: false,
        }
    }
}

/// `Pathfinder::getCell(LAYER_GROUND, x, y)`.
fn ground_cell(map: &PathfindZoneMap, extent: &IRegion2D, x: Int, y: Int) -> Option<CellIdx> {
    (x >= extent.lo.x && x <= extent.hi.x && y >= extent.lo.y && y <= extent.hi.y)
        .then(|| map.index(x, y))
}

/// The corners of a cell, in the order the layer classifiers test them.
fn corners(i: Int, j: Int) -> [Coord3D; 4] {
    let top = j as Real * PATHFIND_CELL_SIZE_F;
    let bottom = top + PATHFIND_CELL_SIZE_F;
    let left = i as Real * PATHFIND_CELL_SIZE_F;
    let right = left + PATHFIND_CELL_SIZE_F;
    // The original leaves z unset here; nothing downstream reads it.
    [
        Coord3D::new(left, top, 0.0),
        Coord3D::new(left, bottom, 0.0),
        Coord3D::new(right, bottom, 0.0),
        Coord3D::new(right, top, 0.0),
    ]
}

/// `PathfindLayer::isPointOnWall`.
pub fn is_point_on_wall_pieces(pieces: &[Footprint], pt: &Coord3D) -> bool {
    pieces.iter().any(|obj| {
        let major = obj.geometry.get_major_radius();
        let minor = if obj.geometry.get_geom_type() == GeometryType::Sphere {
            major
        } else {
            obj.geometry.get_minor_radius()
        };
        let c = gm_cosf(-obj.orientation);
        let s = gm_sinf(-obj.orientation);
        let x = pt.x - obj.position.x;
        let y = pt.y - obj.position.y;
        let new_x = (x * c - y * s).abs();
        let new_y = (x * s + y * c).abs();
        new_x <= major && new_y <= minor
    })
}

impl PathfindLayer {
    /// `reset`. The cells are returned to the reset state, freeing their infos.
    pub fn reset(&mut self, grid: &mut PathfindCellGrid) {
        if let Some(base) = self.base {
            for i in 0..(self.width * self.height) as usize {
                grid.reset_cell(CellIdx(base + i));
            }
        }
        let zone = self.zone;
        let destroyed = self.destroyed;
        *self = Self {
            zone,
            destroyed,
            ..Self::default()
        };
    }

    /// Forgets the cells of a grid that is being replaced.
    pub(crate) fn drop_cells(&mut self) {
        self.base = None;
        self.width = 0;
        self.height = 0;
    }

    pub fn init(&mut self, bridge: Option<LayerBridge>, layer: PathfindLayerEnum) -> bool {
        if self.bridge.is_some() {
            return false;
        }
        self.bridge = bridge;
        self.layer = layer;
        self.destroyed = false;
        true
    }

    pub fn is_unused(&self) -> bool {
        if self.layer == PathfindLayerEnum::Wall && self.width > 0 {
            return false;
        }
        self.bridge.is_none()
    }

    pub const fn is_destroyed(&self) -> bool {
        self.destroyed
    }

    pub const fn zone(&self) -> Int {
        self.zone
    }

    pub fn set_zone(&mut self, zone: Int) {
        self.zone = zone;
    }

    pub const fn start_cell_index(&self) -> ICoord2D {
        self.start_cell
    }

    pub const fn end_cell_index(&self) -> ICoord2D {
        self.end_cell
    }

    /// `(x origin, y origin, width, height)`.
    pub const fn rect(&self) -> (Int, Int, Int, Int) {
        (self.x_origin, self.y_origin, self.width, self.height)
    }

    /// `getBridgeID`.
    pub fn bridge_id(&self) -> ObjectId {
        self.bridge.map_or(ObjectId::INVALID, |b| b.object_id)
    }

    /// The grid index of local cell `(i, j)`, within [`Self::rect`].
    pub fn local(&self, i: Int, j: Int) -> CellIdx {
        CellIdx(self.base.unwrap_or(0) + (i * self.height + j) as usize)
    }

    fn allocate_over(&mut self, map: &mut PathfindZoneMap, extent: &IRegion2D, bounds: &Region2D) {
        // The original's margin is `PATHFIND_CELL_SIZE/100` in unsigned
        // integers, which is zero.
        self.x_origin = real_to_int_floor(bounds.lo.x / PATHFIND_CELL_SIZE_F);
        self.y_origin = real_to_int_floor(bounds.lo.y / PATHFIND_CELL_SIZE_F);
        self.width = 0;
        self.height = 0;
        let mut max_x = real_to_int_ceil(bounds.hi.x / PATHFIND_CELL_SIZE_F);
        let mut max_y = real_to_int_ceil(bounds.hi.y / PATHFIND_CELL_SIZE_F);
        self.x_origin -= 1;
        self.y_origin -= 1;
        max_x += 1;
        max_y += 1;
        self.x_origin = self.x_origin.max(extent.lo.x);
        self.y_origin = self.y_origin.max(extent.lo.y);
        max_x = max_x.min(extent.hi.x);
        max_y = max_y.min(extent.hi.y);
        if max_x <= self.x_origin || max_y <= self.y_origin {
            return;
        }
        self.width = max_x - self.x_origin;
        self.height = max_y - self.y_origin;
        let (x0, y0, h) = (self.x_origin, self.y_origin, self.height);
        let base = map.grid_mut().push_cells(
            (0..self.width).flat_map(|i| (0..h).map(move |j| ICoord2D::new(x0 + i, y0 + j))),
        );
        self.base = Some(base);
    }

    /// `allocateCells` for a bridge layer.
    pub fn allocate_cells(&mut self, map: &mut PathfindZoneMap, extent: &IRegion2D) {
        let Some(bridge) = self.bridge else { return };
        self.allocate_over(map, extent, &bridge.geometry.bounds);
    }

    /// `allocateCellsForWallLayer`. With no live piece the original reads unset
    /// bounds; no cells are made here.
    pub fn allocate_cells_for_wall(
        &mut self,
        map: &mut PathfindZoneMap,
        extent: &IRegion2D,
        pieces: &[Footprint],
    ) {
        if self.layer != PathfindLayerEnum::Wall {
            return;
        }
        let mut bounds: Option<Region2D> = None;
        for piece in pieces {
            let mut piece_bounds = Region2D::default();
            piece
                .geometry
                .get_2d_bounds(&piece.position, piece.orientation, &mut piece_bounds);
            match bounds.as_mut() {
                Some(all) => all.unite_with(&piece_bounds),
                None => bounds = Some(piece_bounds),
            }
        }
        if let Some(bounds) = bounds {
            self.allocate_over(map, extent, &bounds);
        }
    }

    /// `connectsZones`: whether this destroyed bridge would join two zones.
    #[allow(clippy::too_many_arguments)]
    pub fn connects_zones(
        &self,
        zones: &PathfindZoneManager,
        map: &PathfindZoneMap,
        extent: &IRegion2D,
        surfaces: LocomotorSurfaceTypeMask,
        zone1: Int,
        zone2: Int,
    ) -> bool {
        if !self.destroyed {
            return false;
        }
        let mut found1 = false;
        let mut found2 = false;
        for i in 0..self.width {
            for j in 0..self.height {
                if map.grid().cell(self.local(i, j)).connect_layer() != PathfindLayerEnum::Ground {
                    continue;
                }
                if let Some(ground) = ground_cell(map, extent, i + self.x_origin, j + self.y_origin)
                {
                    let zone =
                        zones.get_effective_zone(surfaces, 1, map.grid().cell(ground).zone());
                    let zone = Int::from(zones.get_effective_terrain_zone(zone));
                    found1 |= zone == zone1;
                    found2 |= zone == zone2;
                }
            }
        }
        found1 && found2
    }

    /// `classifyCells` of a bridge layer.
    pub fn classify_cells(
        &mut self,
        map: &mut PathfindZoneMap,
        extent: &IRegion2D,
        terrain: &dyn ThingTerrainAccess,
    ) {
        self.start_cell = ICoord2D::new(-1, -1);
        self.end_cell = ICoord2D::new(-1, -1);
        let Some(bridge) = self.bridge else { return };
        let g = bridge.geometry;
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = self.local(i, j);
                let cell = map.grid_mut().cell_mut(idx);
                cell.set_connect_layer(PathfindLayerEnum::Invalid);
                cell.set_layer(self.layer);
                self.classify_layer_cell(map, extent, terrain, &g, i, j);
            }
            let mut dir = Coord3D::new(g.to.x - g.from.x, g.to.y - g.from.y, g.to.z - g.from.z);
            dir.normalize();
            dir.x *= PATHFIND_CELL_SIZE_F * 0.7;
            dir.y *= PATHFIND_CELL_SIZE_F * 0.7;
            self.start_cell = ICoord2D::new(
                real_to_int_floor((g.from.x - dir.x) / PATHFIND_CELL_SIZE_F),
                real_to_int_floor((g.from.y - dir.y) / PATHFIND_CELL_SIZE_F),
            );
            self.end_cell = ICoord2D::new(
                real_to_int_floor((g.to.x + dir.x) / PATHFIND_CELL_SIZE_F),
                real_to_int_floor((g.to.y + dir.y) / PATHFIND_CELL_SIZE_F),
            );
        }
        if self.destroyed {
            self.cut_off(map, extent, CellType::BridgeImpassable);
        }
    }

    /// The tail of both classifiers for a destroyed layer.
    fn cut_off(&self, map: &mut PathfindZoneMap, extent: &IRegion2D, cell_type: CellType) {
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = self.local(i, j);
                if map.grid().cell(idx).connect_layer() == PathfindLayerEnum::Ground {
                    if let Some(ground) =
                        ground_cell(map, extent, i + self.x_origin, j + self.y_origin)
                    {
                        map.grid_mut()
                            .set_connect_layer(ground, PathfindLayerEnum::Invalid);
                    }
                }
                map.grid_mut().set_type(idx, cell_type);
            }
        }
    }

    /// `classifyLayerMapCell` for local cell `(i, j)`.
    fn classify_layer_cell(
        &self,
        map: &mut PathfindZoneMap,
        extent: &IRegion2D,
        terrain: &dyn ThingTerrainAccess,
        g: &BridgeGeometry,
        i: Int,
        j: Int,
    ) {
        let idx = self.local(i, j);
        let (x, y) = (i + self.x_origin, j + self.y_origin);
        let points = corners(x, y);
        let count = points.iter().filter(|p| g.is_point_on(p)).count();
        let grid = map.grid_mut();
        grid.reset_cell(idx);
        grid.set_layer(idx, self.layer);
        grid.set_type(idx, CellType::Impassable);
        if count == 4 {
            grid.set_type(idx, CellType::Clear);
        } else if count != 0 {
            grid.set_type(idx, CellType::BridgeImpassable);
        }

        let bounds = Region2D {
            lo: zh_common::math::Coord2D {
                x: points[0].x,
                y: points[0].y,
            },
            hi: zh_common::math::Coord2D {
                x: points[2].x,
                y: points[2].y,
            },
        };
        if g.is_cell_on_side(&bounds) {
            grid.set_type(idx, CellType::BridgeImpassable);
        } else {
            if g.is_cell_on_end(&bounds) {
                grid.set_type(idx, CellType::Clear);
            }
            if g.is_cell_entry_point(&bounds) {
                grid.set_type(idx, CellType::Clear);
                grid.set_connect_layer(idx, PathfindLayerEnum::Ground);
                if let Some(ground) = ground_cell(map, extent, x, y) {
                    map.grid_mut().set_connect_layer(ground, self.layer);
                }
            }
        }

        let mut center = points[0];
        center.x += 5.0;
        center.y += 5.0;
        let cell = map.grid().cell(idx);
        if cell.cell_type() != CellType::Impassable
            && cell.connect_layer() != PathfindLayerEnum::Ground
        {
            let ground_height = terrain.get_ground_height(center.x, center.y, None);
            let deck_height = g.height(&center, None);
            if ground_height + LAYER_Z_CLOSE_ENOUGH_F > deck_height {
                if let Some(ground) = ground_cell(map, extent, x, y) {
                    if map.grid().cell(ground).cell_type() != CellType::Obstacle {
                        map.grid_mut().set_type(ground, CellType::BridgeImpassable);
                    }
                }
            }
        }
    }

    /// `classifyWallCells`.
    pub fn classify_wall_cells(
        &mut self,
        map: &mut PathfindZoneMap,
        extent: &IRegion2D,
        pieces: &[Footprint],
    ) {
        if self.layer != PathfindLayerEnum::Wall || self.base.is_none() {
            return;
        }
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = self.local(i, j);
                let grid = map.grid_mut();
                grid.set_connect_layer(idx, PathfindLayerEnum::Invalid);
                grid.set_layer(idx, self.layer);
                let points = corners(i + self.x_origin, j + self.y_origin);
                let count = points
                    .iter()
                    .filter(|p| is_point_on_wall_pieces(pieces, p))
                    .count();
                grid.reset_cell(idx);
                grid.set_layer(idx, self.layer);
                grid.set_type(idx, CellType::Impassable);
                if count == 4 {
                    grid.set_type(idx, CellType::Clear);
                } else if count != 0 {
                    grid.set_type(idx, CellType::BridgeImpassable);
                }
                grid.set_pinched(idx, false);
            }
        }
        if self.destroyed {
            self.cut_off(map, extent, CellType::Impassable);
        }

        for i in 1..self.width - 1 {
            for j in 1..self.height - 1 {
                for k in i - 1..i + 2 {
                    for l in j - 1..j + 2 {
                        if map.grid().cell(self.local(k, l)).cell_type() != CellType::Clear {
                            let idx = self.local(i, j);
                            map.grid_mut().set_pinched(idx, true);
                        }
                    }
                }
            }
        }
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = self.local(i, j);
                let cell = map.grid().cell(idx);
                if cell.pinched() && cell.cell_type() == CellType::Clear {
                    map.grid_mut().set_type(idx, CellType::Cliff);
                }
                map.grid_mut().set_pinched(idx, false);
            }
        }
    }

    /// `setDestroyed`: whether the state changed.
    pub fn set_destroyed(
        &mut self,
        destroyed: bool,
        map: &mut PathfindZoneMap,
        extent: &IRegion2D,
        terrain: &dyn ThingTerrainAccess,
    ) -> bool {
        if destroyed == self.destroyed {
            return false;
        }
        self.destroyed = destroyed;
        self.classify_cells(map, extent, terrain);
        true
    }

    /// `applyZone`.
    pub fn apply_zone(&self, map: &mut PathfindZoneMap) {
        for i in 0..self.width {
            for j in 0..self.height {
                map.grid_mut().set_zone(self.local(i, j), self.zone as u16);
            }
        }
    }

    /// `getCell`: `None` off the layer and on impassable cells.
    pub fn get_cell(&self, map: &PathfindZoneMap, x: Int, y: Int) -> Option<CellIdx> {
        self.base?;
        let (i, j) = (x - self.x_origin, y - self.y_origin);
        if i < 0 || i >= self.width || j < 0 || j >= self.height {
            return None;
        }
        let idx = self.local(i, j);
        (map.grid().cell(idx).cell_type() != CellType::Impassable).then_some(idx)
    }
}
