//! `PKT-411`, map half: building and maintaining the pathfind map
//! (`Core/GameEngine/Source/GameLogic/AI/AIPathfind.cpp`): cell classification,
//! object footprints, fences, wall pieces, bridge layers, zones and
//! `Pathfinder::crc`. Searches, paths and unit reservations are not here yet.
//!
//! Golden test: `crates/zh_logic/tests/golden_s2_pathfind_map.rs`.
//!
//! The game is passed in: terrain per call, objects as [`Footprint`] values.
//! The frame number stands for `TheGameLogic->getFrame()`.

use zh_common::math::gamemath::{real_to_int_ceil, real_to_int_floor};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, ICoord2D, IRegion2D, Region3D};
use zh_common::types::{Int, ObjectId, Real, UnsignedInt, FALSE};
use zh_common::xfer::{Xfer, XferError};

use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::pathfind_cell::{CellType, ObstacleDescriptor};
use crate::pathfinder::pathfind_cell_info::CellIdx;
use crate::pathfinder::pathfind_constants::{
    PathfindLayerEnum, MAX_WALL_PIECES, PATHFIND_CELL_SIZE_F, PATHFIND_QUEUE_LEN,
};
use crate::pathfinder::pathfind_layer::{is_point_on_wall_pieces, LayerBridge, PathfindLayer};
use crate::pathfinder::pathfind_zone_manager::{
    PathfindZoneManager, PathfindZoneMap, ZoneLayerSlot, LAYER_COUNT, UNINITIALIZED_ZONE,
};
use crate::terrain::bridge::BridgeGeometry;
use crate::terrain::{ElevatedLayers, TerrainEffects, TerrainLogic};

/// What the pathfinder reads from an object.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Footprint {
    pub id: ObjectId,
    pub position: Coord3D,
    pub orientation: Real,
    pub geometry: GeometryInfo,
    /// `ThingTemplate::getFenceWidth` and `getFenceXOffset`.
    pub fence_width: Real,
    pub fence_x_offset: Real,
    pub mine: bool,
    pub projectile: bool,
    pub bridge_tower: bool,
    pub defensive_wall: bool,
    pub blast_crater: bool,
    pub walk_on_top_of_wall: bool,
    pub structure: bool,
    /// `KINDOF_CAN_SEE_THROUGH_STRUCTURE`.
    pub see_through: bool,
    /// `Object::isMobile`: neither `IMMOBILE` nor disabled.
    pub mobile: bool,
    pub height_above_terrain: Real,
    /// The body is in `BODY_RUBBLE`.
    pub rubble: bool,
}

/// The objects the wall code looks up.
pub trait PathWorld {
    /// `findObjectByID`.
    fn find_object(&self, id: ObjectId) -> Option<Footprint>;
    /// Objects whose layer is `LAYER_WALL`, in `getFirstObject` order.
    fn objects_on_wall_layer(&self) -> Vec<(ObjectId, Coord3D)> {
        Vec::new()
    }
}

/// A world without wall pieces.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoPathWorld;

impl PathWorld for NoPathWorld {
    fn find_object(&self, _id: ObjectId) -> Option<Footprint> {
        None
    }
}

/// The pathfinder as `TerrainLogic`'s water changes reach it.
pub struct PathfindEffects<'a> {
    pub pathfinder: &'a mut Pathfinder,
    pub world: &'a dyn PathWorld,
}

impl TerrainEffects for PathfindEffects<'_> {
    fn force_map_recalculation(&mut self, terrain: &TerrainLogic) {
        self.pathfinder.force_map_recalculation(terrain, self.world);
    }
}

/// The part of the wall layer that `getLayerForDestination` and friends ask the
/// pathfinder about, to hand to `TerrainLogic::set_elevated_layers`.
#[derive(Debug, Clone, Default)]
pub struct WallSurface {
    height: Real,
    extent: IRegion2D,
    clear: Vec<ICoord2D>,
}

impl ElevatedLayers for WallSurface {
    fn wall_height(&self) -> Real {
        self.height
    }

    fn is_point_on_wall(&self, pos: &Coord3D) -> u8 {
        let (cell, overflow) = clip_to_cell(&self.extent, pos);
        u8::from(!overflow && self.clear.contains(&cell))
    }
}

/// `worldToCell`: the clipped cell and whether it was clipped.
fn clip_to_cell(extent: &IRegion2D, pos: &Coord3D) -> (ICoord2D, bool) {
    let mut cell = ICoord2D::new(
        real_to_int_floor(pos.x / PATHFIND_CELL_SIZE_F),
        real_to_int_floor(pos.y / PATHFIND_CELL_SIZE_F),
    );
    let before = cell;
    cell.x = cell.x.max(extent.lo.x).min(extent.hi.x);
    cell.y = cell.y.max(extent.lo.y).min(extent.hi.y);
    (cell, cell != before)
}

fn cell_bounds_of(region: &Region3D) -> IRegion2D {
    let mut bounds = IRegion2D {
        lo: ICoord2D::new(
            real_to_int_floor(region.lo.x / PATHFIND_CELL_SIZE_F),
            real_to_int_floor(region.lo.y / PATHFIND_CELL_SIZE_F),
        ),
        hi: ICoord2D::new(
            real_to_int_floor(region.hi.x / PATHFIND_CELL_SIZE_F),
            real_to_int_floor(region.hi.y / PATHFIND_CELL_SIZE_F),
        ),
    };
    bounds.hi.x -= 1;
    bounds.hi.y -= 1;
    bounds
}

fn include(bounds: &mut IRegion2D, x: Int, y: Int) {
    bounds.lo.x = bounds.lo.x.min(x);
    bounds.lo.y = bounds.lo.y.min(y);
    bounds.hi.x = bounds.hi.x.max(x);
    bounds.hi.y = bounds.hi.y.max(y);
}

/// The map half of `Pathfinder`.
pub struct Pathfinder {
    map: PathfindZoneMap,
    /// `m_blockOfMapCells` is allocated.
    allocated: bool,
    extent: IRegion2D,
    logical_extent: IRegion2D,
    layers: Vec<PathfindLayer>,
    zones: PathfindZoneManager,
    wall_pieces: [ObjectId; MAX_WALL_PIECES as usize],
    num_wall_pieces: Int,
    wall_height: Real,
    is_map_ready: bool,
    pub is_tunneling: bool,
    pub ignore_obstacle_id: ObjectId,
    queued_requests: [ObjectId; PATHFIND_QUEUE_LEN as usize],
    queue_head: Int,
    queue_tail: Int,
    pub cumulative_cells_allocated: Int,
    /// Set by `GameLogic::processDestroyList`; read under `retail-crc` only.
    pub classify_fence_zero_init: bool,
}

impl Default for Pathfinder {
    fn default() -> Self {
        Self::new(0.0)
    }
}

impl Pathfinder {
    /// `wall_height` is `AIData::m_wallHeight`.
    pub fn new(wall_height: Real) -> Self {
        let mut out = Self {
            map: PathfindZoneMap::new(0, 0),
            allocated: false,
            extent: IRegion2D::default(),
            logical_extent: IRegion2D::default(),
            layers: vec![PathfindLayer::default(); LAYER_COUNT],
            zones: PathfindZoneManager::new(),
            wall_pieces: [ObjectId::INVALID; MAX_WALL_PIECES as usize],
            num_wall_pieces: 0,
            wall_height,
            is_map_ready: false,
            is_tunneling: false,
            ignore_obstacle_id: ObjectId::INVALID,
            queued_requests: [ObjectId::INVALID; PATHFIND_QUEUE_LEN as usize],
            queue_head: 0,
            queue_tail: 0,
            cumulative_cells_allocated: 0,
            classify_fence_zero_init: false,
        };
        out.reset(wall_height);
        out
    }

    /// `reset`.
    pub fn reset(&mut self, wall_height: Real) {
        for layer in &mut self.layers {
            layer.reset(self.map.grid_mut());
        }
        self.map = PathfindZoneMap::new(0, 0);
        self.allocated = false;
        self.extent = IRegion2D::default();
        self.logical_extent = IRegion2D::default();
        self.ignore_obstacle_id = ObjectId::INVALID;
        self.is_tunneling = false;
        self.is_map_ready = false;
        self.cumulative_cells_allocated = 0;
        self.queued_requests = [ObjectId::INVALID; PATHFIND_QUEUE_LEN as usize];
        self.queue_head = 0;
        self.queue_tail = 0;
        self.num_wall_pieces = 0;
        self.wall_pieces = [ObjectId::INVALID; MAX_WALL_PIECES as usize];
        self.wall_height = wall_height;
        self.zones.reset();
        self.classify_fence_zero_init = false;
    }

    pub const fn extent(&self) -> IRegion2D {
        self.extent
    }

    pub const fn logical_extent(&self) -> IRegion2D {
        self.logical_extent
    }

    pub const fn is_map_ready(&self) -> bool {
        self.is_map_ready
    }

    pub const fn wall_height(&self) -> Real {
        self.wall_height
    }

    pub fn map(&self) -> &PathfindZoneMap {
        &self.map
    }

    pub fn map_mut(&mut self) -> &mut PathfindZoneMap {
        &mut self.map
    }

    pub fn zones(&self) -> &PathfindZoneManager {
        &self.zones
    }

    pub fn layer(&self, layer: PathfindLayerEnum) -> &PathfindLayer {
        &self.layers[layer as usize]
    }

    pub fn wall_pieces(&self) -> &[ObjectId] {
        &self.wall_pieces[..self.num_wall_pieces as usize]
    }

    /// `getCell`: the layer's cell where it has one, else the ground cell.
    pub fn get_cell(&self, layer: PathfindLayerEnum, x: Int, y: Int) -> Option<CellIdx> {
        let e = &self.extent;
        if !self.allocated || x < e.lo.x || x > e.hi.x || y < e.lo.y || y > e.hi.y {
            return None;
        }
        if layer as Int > PathfindLayerEnum::Ground as Int {
            if let Some(idx) = self.layers[layer as usize].get_cell(&self.map, x, y) {
                return Some(idx);
            }
        }
        Some(self.map.index(x, y))
    }

    /// `getCell(layer, pos)`: `None` off the map.
    pub fn get_cell_at(&self, layer: PathfindLayerEnum, pos: &Coord3D) -> Option<CellIdx> {
        let (cell, overflow) = clip_to_cell(&self.extent, pos);
        if overflow {
            return None;
        }
        self.get_cell(layer, cell.x, cell.y)
    }

    pub fn cell_type(&self, layer: PathfindLayerEnum, x: Int, y: Int) -> Option<CellType> {
        self.get_cell(layer, x, y)
            .map(|i| self.map.grid().cell(i).cell_type())
    }

    // ---- wall pieces -----------------------------------------------------

    /// `addWallPiece`.
    pub fn add_wall_piece(&mut self, id: ObjectId) {
        if self.num_wall_pieces < MAX_WALL_PIECES - 1 {
            self.wall_pieces[self.num_wall_pieces as usize] = id;
            self.num_wall_pieces += 1;
        }
    }

    /// `removeWallPiece`: the last piece takes the removed slot.
    pub fn remove_wall_piece(&mut self, id: ObjectId) {
        for i in 0..self.num_wall_pieces as usize {
            if self.wall_pieces[i] == id {
                self.wall_pieces[i] = self.wall_pieces[self.num_wall_pieces as usize - 1];
                self.num_wall_pieces -= 1;
                return;
            }
        }
    }

    /// `isPointOnWall`.
    pub fn is_point_on_wall(&self, pos: &Coord3D) -> bool {
        if self.num_wall_pieces == 0 || self.layers[PathfindLayerEnum::Wall as usize].is_unused() {
            return false;
        }
        self.get_cell_at(PathfindLayerEnum::Wall, pos)
            .is_some_and(|i| {
                let cell = self.map.grid().cell(i);
                cell.layer() == PathfindLayerEnum::Wall && cell.cell_type() == CellType::Clear
            })
    }

    /// A copy of what `is_point_on_wall` answers now. Take it again after the
    /// map or the wall changes.
    pub fn wall_surface(&self) -> WallSurface {
        let mut clear = Vec::new();
        let (x0, y0, w, h) = self.layers[PathfindLayerEnum::Wall as usize].rect();
        for x in x0..x0 + w {
            for y in y0..y0 + h {
                let on = self
                    .get_cell(PathfindLayerEnum::Wall, x, y)
                    .is_some_and(|i| {
                        let cell = self.map.grid().cell(i);
                        cell.layer() == PathfindLayerEnum::Wall
                            && cell.cell_type() == CellType::Clear
                    });
                if on && self.num_wall_pieces != 0 {
                    clear.push(ICoord2D::new(x, y));
                }
            }
        }
        WallSurface {
            height: self.wall_height,
            extent: self.extent,
            clear,
        }
    }

    fn wall_footprints(&self, world: &dyn PathWorld) -> Vec<Footprint> {
        self.wall_pieces()
            .iter()
            .filter_map(|&id| world.find_object(id))
            .collect()
    }

    // ---- bridge layers ---------------------------------------------------

    /// `addBridge`: the layer given to the bridge, or ground if none.
    pub fn add_bridge(
        &mut self,
        geometry: BridgeGeometry,
        object_id: ObjectId,
    ) -> PathfindLayerEnum {
        for raw in PathfindLayerEnum::Ground as Int + 1..=PathfindLayerEnum::Wall as Int {
            if !self.layers[raw as usize].is_unused() {
                continue;
            }
            let Some(layer) = PathfindLayerEnum::from_i32(raw) else {
                break;
            };
            let bridge = LayerBridge {
                geometry,
                object_id,
            };
            if self.layers[raw as usize].init(Some(bridge), layer) {
                return layer;
            }
            return PathfindLayerEnum::Ground;
        }
        PathfindLayerEnum::Ground
    }

    /// `changeBridgeState`.
    pub fn change_bridge_state(
        &mut self,
        layer: PathfindLayerEnum,
        repaired: bool,
        terrain: &dyn ThingTerrainAccess,
        frame: UnsignedInt,
    ) {
        let slot = &mut self.layers[layer as usize];
        if slot.is_unused() {
            return;
        }
        if slot.set_destroyed(!repaired, &mut self.map, &self.extent, terrain) {
            self.zones.mark_zones_dirty(frame);
        }
    }

    // ---- classification --------------------------------------------------

    /// `classifyMapCell`.
    fn classify_map_cell(&mut self, terrain: &dyn ThingTerrainAccess, i: Int, j: Int) {
        let idx = self.map.index(i, j);
        let grid = self.map.grid_mut();
        let has_obstacle = grid.cell(idx).cell_type() == CellType::Obstacle;
        let top = j as Real * PATHFIND_CELL_SIZE_F;
        let bottom = top + PATHFIND_CELL_SIZE_F;
        let left = i as Real * PATHFIND_CELL_SIZE_F;
        let right = left + PATHFIND_CELL_SIZE_F;
        grid.set_pinched(idx, false);

        let mut cell_type = CellType::Clear;
        if terrain.is_cliff_cell(left, top) != FALSE {
            cell_type = CellType::Cliff;
        }
        for (x, y) in [(left, top), (left, bottom), (right, bottom), (right, top)] {
            if terrain.is_underwater(x, y, None, None) != FALSE {
                cell_type = CellType::Water;
            }
        }
        if has_obstacle {
            cell_type = CellType::Obstacle;
        }
        grid.set_type(idx, cell_type);
        grid.release_info(idx);
    }

    /// Sets `pinched` on the clear cells next to a cliff. The original tests
    /// the type with `&`, so rubble and impassable cells count as cliffs.
    fn pinch_around_cliffs(&mut self) {
        let e = self.extent;
        for j in e.lo.y..=e.hi.y {
            for i in e.lo.x..=e.hi.x {
                if self.map.cell(i, j).cell_type() as Int & CellType::Cliff as Int == 0 {
                    continue;
                }
                for k in i - 1..i + 2 {
                    if k < e.lo.x || k > e.hi.x {
                        continue;
                    }
                    for l in j - 1..j + 2 {
                        if l < e.lo.y || l > e.hi.y {
                            continue;
                        }
                        if self.map.cell(k, l).cell_type() == CellType::Clear {
                            self.map.cell_mut(k, l).set_pinched(true);
                        }
                    }
                }
            }
        }
    }

    /// `classifyMap`.
    fn classify_map(&mut self, terrain: &dyn ThingTerrainAccess, world: &dyn PathWorld) {
        let e = self.extent;
        for j in e.lo.y..=e.hi.y {
            for i in e.lo.x..=e.hi.x {
                self.classify_map_cell(terrain, i, j);
            }
        }
        self.pinch_around_cliffs();
        for j in e.lo.y..=e.hi.y {
            for i in e.lo.x..=e.hi.x {
                let cell = self.map.cell(i, j);
                if cell.pinched() && cell.cell_type() == CellType::Clear {
                    let idx = self.map.index(i, j);
                    self.map.grid_mut().set_type(idx, CellType::Cliff);
                }
            }
        }
        self.pinch_around_cliffs();

        let wall = PathfindLayerEnum::Wall as usize;
        for layer in &mut self.layers[..wall] {
            if !layer.is_unused() {
                layer.classify_cells(&mut self.map, &self.extent, terrain);
            }
        }
        if !self.layers[wall].is_unused() {
            let pieces = self.wall_footprints(world);
            self.layers[wall].classify_wall_cells(&mut self.map, &self.extent, &pieces);
        }
        self.calculate_zones();
    }

    /// `PathfindZoneManager::calculateZones` over the ground and the layers.
    pub fn calculate_zones(&mut self) {
        let mut slots: [ZoneLayerSlot; LAYER_COUNT] = std::array::from_fn(|i| {
            let layer = &self.layers[i];
            ZoneLayerSlot {
                zone: layer.zone(),
                unused: layer.is_unused(),
                destroyed: layer.is_destroyed(),
                start_cell: layer.start_cell_index(),
                end_cell: layer.end_cell_index(),
                applied_zone: 0,
            }
        });
        self.zones
            .calculate_zones(&mut self.map, &mut slots, &self.extent);
        for (layer, slot) in self.layers.iter_mut().zip(&slots) {
            layer.set_zone(slot.zone);
            layer.apply_zone(&mut self.map);
        }
    }

    /// `processPathfindQueue` before it takes requests: recalculates the zones
    /// when due, else sets the logical extent from `getExtent`. Whether the
    /// queue is processed this frame.
    pub fn begin_path_queue(&mut self, terrain_extent: &Region3D, frame: UnsignedInt) -> bool {
        if !self.is_map_ready {
            return false;
        }
        if self.zones.need_to_calculate_zones(frame) != FALSE {
            self.calculate_zones();
            return false;
        }
        self.logical_extent = cell_bounds_of(terrain_extent);
        self.cumulative_cells_allocated = 0;
        true
    }

    fn update_zones_for_modify(&mut self, bounds: &IRegion2D) {
        let mut slots: [ZoneLayerSlot; LAYER_COUNT] = Default::default();
        self.zones
            .update_zones_for_modify(&mut self.map, &mut slots, bounds, &self.extent);
    }

    /// `newMap`. `terrain_extent` is `getMaximumPathfindExtent`; `objects`
    /// are all objects in `getFirstObject` order.
    pub fn new_map(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        terrain_extent: &Region3D,
        wall_height: Real,
        objects: &[Footprint],
        world: &dyn PathWorld,
        frame: UnsignedInt,
    ) {
        self.wall_height = wall_height;
        let bounds = cell_bounds_of(terrain_extent);
        let data_allocated = self.extent.hi == bounds.hi && self.allocated;
        if !data_allocated {
            self.extent = bounds;
            self.zones.allocate_blocks(&self.extent);
            self.map = PathfindZoneMap::new(bounds.hi.x + 1, bounds.hi.y + 1);
            self.map.mode_mut().use_fixed_pathfinding = false;
            self.map.pool_mut().allocate_cell_infos();
            self.allocated = true;
            for layer in &mut self.layers {
                layer.drop_cells();
            }
            let wall = PathfindLayerEnum::Wall as usize;
            for layer in &mut self.layers[..wall] {
                if !layer.is_unused() {
                    layer.allocate_cells(&mut self.map, &self.extent);
                }
            }
            if self.num_wall_pieces > 0 {
                let pieces = self.wall_footprints(world);
                self.layers[wall].init(None, PathfindLayerEnum::Wall);
                self.layers[wall].allocate_cells_for_wall(&mut self.map, &self.extent, &pieces);
            }
        }
        self.classify_map(terrain, world);
        for object in objects {
            // Inserting a footprint never drops units off a wall.
            self.classify_object_footprint(object, true, world, frame);
        }
        self.is_map_ready = true;
    }

    /// `forceMapRecalculation`.
    pub fn force_map_recalculation(
        &mut self,
        terrain: &dyn ThingTerrainAccess,
        world: &dyn PathWorld,
    ) {
        if self.allocated {
            self.classify_map(terrain, world);
        }
    }

    // ---- object footprints -----------------------------------------------

    /// `classifyObjectFootprint` (`addObjectToPathfindMap` and
    /// `removeObjectFromPathfindMap`). On removal of a wall piece, returns the
    /// objects that were standing on it and take falling damage.
    ///
    /// `removeUnitFromPathfindMap`, which the original calls first on removal,
    /// clears reservations and is not ported.
    pub fn classify_object_footprint(
        &mut self,
        obj: &Footprint,
        insert: bool,
        world: &dyn PathWorld,
        frame: UnsignedInt,
    ) -> Vec<ObjectId> {
        let mut fallers = Vec::new();
        if !self.allocated || obj.mine || obj.projectile || obj.bridge_tower {
            return fallers;
        }
        if obj.fence_width > 0.0 && !obj.defensive_wall {
            self.classify_fence(obj, insert, frame);
            return fallers;
        }
        if !insert {
            if obj.blast_crater {
                return fallers;
            }
            let wall = PathfindLayerEnum::Wall as usize;
            if obj.walk_on_top_of_wall && !self.layers[wall].is_unused() {
                for i in 0..self.num_wall_pieces as usize {
                    if self.wall_pieces[i] == obj.id {
                        self.wall_pieces[i] = ObjectId::INVALID;
                    }
                }
                let removed: Vec<Footprint> = world.find_object(obj.id).into_iter().collect();
                for (id, position) in world.objects_on_wall_layer() {
                    if is_point_on_wall_pieces(&removed, &position) {
                        fallers.push(id);
                    }
                }
                let pieces = self.wall_footprints(world);
                self.layers[wall].classify_wall_cells(&mut self.map, &self.extent, &pieces);
            }
        }
        if !obj.structure || obj.mobile || obj.geometry.get_is_small() != FALSE {
            return fallers;
        }
        if obj.height_above_terrain > PATHFIND_CELL_SIZE_F && !obj.blast_crater {
            return fallers;
        }
        self.classify_footprint_cells(obj, insert, frame);
        fallers
    }

    /// Inserts or removes the obstacle at one cell, if it is below the extent's
    /// high edge, and grows `bounds`.
    fn stamp(
        &mut self,
        obstacle: &ObstacleDescriptor,
        insert: bool,
        fence: bool,
        x: Int,
        y: Int,
        bounds: &mut IRegion2D,
    ) -> bool {
        if x < 0 || y < 0 || x >= self.extent.hi.x || y >= self.extent.hi.y {
            return false;
        }
        let idx = self.map.index(x, y);
        let grid = self.map.grid_mut();
        let changed = if insert {
            let obstacle = ObstacleDescriptor {
                is_fence: fence,
                ..*obstacle
            };
            grid.set_type_as_obstacle(idx, &obstacle, ICoord2D::new(x, y))
        } else {
            grid.remove_obstacle(idx, obstacle.id)
        };
        if changed {
            grid.set_zone(idx, UNINITIALIZED_ZONE);
        }
        include(bounds, x, y);
        changed
    }

    fn descriptor(obj: &Footprint) -> ObstacleDescriptor {
        ObstacleDescriptor {
            id: obj.id,
            is_rubble: obj.rubble,
            is_fence: false,
            is_transparent: obj.see_through,
        }
    }

    /// `classifyFence`: a thin line of obstacle cells along the object.
    fn classify_fence(&mut self, obj: &Footprint, insert: bool, frame: UnsignedInt) {
        let pos = obj.position;
        let half_x = obj.fence_width / 2.0;
        let half_y = PATHFIND_CELL_SIZE_F / 10.0;
        let offset = obj.fence_x_offset;
        let c = gm_cosf(obj.orientation);
        let s = gm_sinf(obj.orientation);
        const STEP: Real = PATHFIND_CELL_SIZE_F * 0.5;
        let (ydx, ydy) = (s * STEP, -c * STEP);
        let (xdx, xdy) = (c * STEP, s * STEP);
        let steps_x = real_to_int_ceil(2.0 * half_x / STEP);
        let steps_y = real_to_int_ceil(2.0 * half_y / STEP);
        let mut tl_x = pos.x - offset * c - half_y * s;
        let mut tl_y = pos.y + half_y * c - offset * s;

        let lo = ICoord2D::new(
            real_to_int_floor((pos.x + 0.5) / PATHFIND_CELL_SIZE_F),
            real_to_int_floor((pos.y + 0.5) / PATHFIND_CELL_SIZE_F),
        );
        #[cfg(feature = "retail-crc")]
        let hi = if self.classify_fence_zero_init {
            ICoord2D::new(0, 0)
        } else {
            ICoord2D::new(1_000_000, 1_000_000)
        };
        #[cfg(not(feature = "retail-crc"))]
        let hi = ICoord2D::new(
            real_to_int_ceil((pos.x + 0.5) / PATHFIND_CELL_SIZE_F),
            real_to_int_ceil((pos.y + 0.5) / PATHFIND_CELL_SIZE_F),
        );
        let mut bounds = IRegion2D { lo, hi };
        let obstacle = Self::descriptor(obj);
        let mut changed = false;
        for _ in 0..steps_y {
            let (mut x, mut y) = (tl_x, tl_y);
            for _ in 0..steps_x {
                let cx = real_to_int_floor((x + 0.5) / PATHFIND_CELL_SIZE_F);
                let cy = real_to_int_floor((y + 0.5) / PATHFIND_CELL_SIZE_F);
                changed |= self.stamp(&obstacle, insert, true, cx, cy, &mut bounds);
                x += xdx;
                y += xdy;
            }
            tl_x += ydx;
            tl_y += ydy;
        }
        if changed {
            self.zones.mark_zones_dirty(frame);
            self.update_zones_for_modify(&bounds);
        }
    }

    /// `createAWallFromMyFootprint` / `removeWallFromMyFootprint`, and the
    /// body of a structure's classification
    /// (`internal_classifyObjectFootprint`).
    pub fn classify_footprint_cells(&mut self, obj: &Footprint, insert: bool, frame: UnsignedInt) {
        if !self.allocated {
            return;
        }
        let pos = obj.position;
        let lo = ICoord2D::new(
            real_to_int_floor((pos.x + 0.5) / PATHFIND_CELL_SIZE_F),
            real_to_int_floor((pos.y + 0.5) / PATHFIND_CELL_SIZE_F),
        );
        let mut bounds = IRegion2D { lo, hi: lo };
        let obstacle = Self::descriptor(obj);
        self.zones.mark_zones_dirty(frame);

        match obj.geometry.get_geom_type() {
            GeometryType::Box => {
                let c = gm_cosf(obj.orientation);
                let s = gm_sinf(obj.orientation);
                let half_x = obj.geometry.get_major_radius();
                let half_y = obj.geometry.get_minor_radius();
                const STEP: Real = PATHFIND_CELL_SIZE_F * 0.5;
                let (ydx, ydy) = (s * STEP, -c * STEP);
                let (xdx, xdy) = (c * STEP, s * STEP);
                let steps_x = real_to_int_ceil(2.0 * half_x / STEP);
                let steps_y = real_to_int_ceil(2.0 * half_y / STEP);
                let mut tl_x = pos.x - half_x * c - half_y * s;
                let mut tl_y = pos.y + half_y * c - half_x * s;
                for _ in 0..steps_y {
                    let (mut x, mut y) = (tl_x, tl_y);
                    for _ in 0..steps_x {
                        let cx = real_to_int_floor((x + 0.5) / PATHFIND_CELL_SIZE_F);
                        let cy = real_to_int_floor((y + 0.5) / PATHFIND_CELL_SIZE_F);
                        self.stamp(&obstacle, insert, false, cx, cy, &mut bounds);
                        x += xdx;
                        y += xdy;
                    }
                    tl_x += ydx;
                    tl_y += ydy;
                }
            }
            GeometryType::Sphere | GeometryType::Cylinder => {
                let radius = obj.geometry.get_major_radius();
                let left = real_to_int_floor(0.5 + (pos.x - radius) / PATHFIND_CELL_SIZE_F) - 1;
                let top = real_to_int_floor(0.5 + (pos.y - radius) / PATHFIND_CELL_SIZE_F) - 1;
                let mut size = radius / PATHFIND_CELL_SIZE_F;
                let center_x = pos.x / PATHFIND_CELL_SIZE_F;
                let center_y = pos.y / PATHFIND_CELL_SIZE_F;
                size += 0.4;
                let r2 = size * size;
                let right = (left as Real + 2.0 * size + 2.0) as Int;
                let bottom = (top as Real + 2.0 * size + 2.0) as Int;
                for j in top..bottom {
                    for i in left..right {
                        let dx = i as Real + 0.5 - center_x;
                        let dy = j as Real + 0.5 - center_y;
                        if dx * dx + dy * dy <= r2 {
                            self.stamp(&obstacle, insert, false, i, j, &mut bounds);
                        }
                    }
                }
            }
        }

        self.update_zones_for_modify(&bounds);
        bounds.lo.x -= 2;
        bounds.lo.y -= 2;
        bounds.hi.x += 2;
        bounds.hi.y += 2;
        bounds.intersect_with(&self.extent);
        self.pinch_around_footprint(&bounds, insert);
    }

    /// The pinching passes over `bounds` after a footprint change.
    fn pinch_around_footprint(&mut self, bounds: &IRegion2D, insert: bool) {
        let e = self.extent;
        let cells = || {
            (bounds.lo.y..=bounds.hi.y)
                .flat_map(move |j| (bounds.lo.x..=bounds.hi.x).map(move |i| (i, j)))
        };
        let near = |i: Int, j: Int| {
            (i - 1..i + 2)
                .filter(move |&k| k >= e.lo.x && k <= e.hi.x)
                .flat_map(move |k| {
                    (j - 1..j + 2)
                        .filter(move |&l| l >= e.lo.y && l <= e.hi.y)
                        .map(move |l| (k, l))
                })
                .filter(move |&(k, l)| (k, l) != (i, j))
        };
        if !insert {
            for (i, j) in cells() {
                if self.map.cell(i, j).cell_type() == CellType::Impassable {
                    let idx = self.map.index(i, j);
                    self.map.grid_mut().set_type(idx, CellType::Clear);
                }
            }
        }
        for (i, j) in cells() {
            self.map.cell_mut(i, j).set_pinched(false);
            if self.map.cell(i, j).cell_type() != CellType::Clear {
                continue;
            }
            let mut total = 0;
            let mut orthogonal = 0;
            for (k, l) in near(i, j) {
                if self.map.cell(k, l).cell_type() == CellType::Clear {
                    total += 1;
                    if k == i || l == j {
                        orthogonal += 1;
                    }
                }
            }
            if orthogonal < 2 || total < 4 {
                self.map.cell_mut(i, j).set_pinched(true);
            }
        }
        // `RETAIL_COMPATIBLE_PATHFINDING`: pinched cells become impassable.
        for (i, j) in cells() {
            let cell = self.map.cell(i, j);
            if cell.pinched() && cell.cell_type() == CellType::Clear {
                let idx = self.map.index(i, j);
                self.map.grid_mut().set_type(idx, CellType::Impassable);
                self.map.grid_mut().set_pinched(idx, false);
            }
        }
        for (i, j) in cells() {
            if self.map.cell(i, j).cell_type() != CellType::Clear {
                continue;
            }
            let adjacent = near(i, j)
                .filter(|&(k, l)| k == i || l == j)
                .any(|(k, l)| self.map.cell(k, l).cell_type() == CellType::Obstacle);
            if adjacent {
                self.map.cell_mut(i, j).set_pinched(true);
            }
        }
    }

    /// `crc`.
    pub fn crc(&self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut extent = [0u8; 16];
        let values = [
            self.extent.lo.x,
            self.extent.lo.y,
            self.extent.hi.x,
            self.extent.hi.y,
        ];
        for (chunk, v) in extent.chunks_exact_mut(4).zip(values) {
            chunk.copy_from_slice(&v.to_le_bytes());
        }
        xfer.xfer_user(&mut extent)?;
        let mut ready = u8::from(self.is_map_ready);
        xfer.xfer_bool(&mut ready)?;
        let mut tunneling = u8::from(self.is_tunneling);
        xfer.xfer_bool(&mut tunneling)?;
        let mut obsolete = 0;
        xfer.xfer_int(&mut obsolete)?;
        let mut ignore = self.ignore_obstacle_id.0.to_le_bytes();
        xfer.xfer_user(&mut ignore)?;
        let mut queue: Vec<u8> = self
            .queued_requests
            .iter()
            .flat_map(|id| id.0.to_le_bytes())
            .collect();
        xfer.xfer_user(&mut queue)?;
        let mut head = self.queue_head;
        xfer.xfer_int(&mut head)?;
        let mut tail = self.queue_tail;
        xfer.xfer_int(&mut tail)?;
        let mut count = self.num_wall_pieces;
        xfer.xfer_int(&mut count)?;
        #[cfg(feature = "retail-crc")]
        for _ in 0..MAX_WALL_PIECES {
            xfer.xfer_int(&mut count)?;
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            let mut pieces: Vec<u8> = self
                .wall_pieces
                .iter()
                .flat_map(|id| id.0.to_le_bytes())
                .collect();
            xfer.xfer_user(&mut pieces)?;
        }
        let mut wall_height = self.wall_height;
        xfer.xfer_real(&mut wall_height)?;
        let mut cumulative = self.cumulative_cells_allocated;
        xfer.xfer_int(&mut cumulative)?;
        Ok(())
    }
}
