//! `TerrainLogic.cpp` without bridges, with the overrides of
//! `W3DTerrainLogic.cpp`: map loading, extents, heights, waypoints, water,
//! terrain flattening and craters, and the snapshot.
//!
//! Seams, each a callee that is not ported:
//! - [`ElevatedLayers`]: the pathfinder's wall and the bridges.
//! - [`WaterGrid`]: the client's deformable water mesh.
//! - [`TerrainEffects`]: pathfind recalculation and water damage.
//! - [`TerrainLogic::set_active_boundary`] changes the boundary only; the
//!   caller redoes the partition, radar, shroud and camera.
//!
//! Golden test: `crates/zh_logic/tests/golden_s1_terrain.rs`.

use crate::ai::seams::{MapTerrain, WaypointInfo};
use crate::object::thing::{make_align_to_normal_matrix, ThingTerrainAccess};
use crate::pathfinder::ai_pathfind::Pathfinder;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::terrain::bridge::{highest_layer, nearest_layer, Bridge, BridgeList, BridgeWorld};
use crate::spatial::polygon_trigger::{PolygonTrigger, PolygonTriggerList};
use crate::spatial::terrain::{find_closest_edge_point, find_farthest_edge_point};
use crate::terrain::height_map::{HeightMap, MAX_HEIGHT_VALUE};
use zh_common::core::dict::{Dict, DictDataType};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::io::data_chunk::{
    CachedFileInputStream, DataChunkError, DataChunkInfo, DataChunkInput,
};
use zh_common::io::map_object::{MapObject, MAP_HEIGHT_SCALE, MAP_XY_FACTOR};
use zh_common::io::map_reader_writer_info::K_OBJECTS_VERSION_2;
use zh_common::math::gamemath::{gm_cosf, gm_sinf, real_to_int_floor};
use zh_common::math::geometry::GeometryType;
use zh_common::math::{gm_sqrtf, Coord3D, ICoord2D, ICoord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, Real, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// `MAX_DYNAMIC_WATER`.
pub const MAX_DYNAMIC_WATER: usize = 64;
/// `Waypoint::MAX_LINKS`.
pub const MAX_WAYPOINT_LINKS: usize = 8;
/// The name `getWaterHandleByName` answers with the water grid.
pub const WATER_GRID_NAME: &str = "Water Grid";
const LOGIC_FRAMES_PER_SECOND: u32 = 30;

/// A waypoint (`Waypoint`). Links are indices into [`TerrainLogic`]'s storage.
#[derive(Debug, Clone)]
pub struct Waypoint {
    pub id: u32,
    pub name: String,
    pub location: Coord3D,
    pub path_labels: [String; 3],
    pub bi_directional: Bool,
    links: Vec<usize>,
}

/// A water area (`WaterHandle`): the grid, or a water polygon by its place in
/// the trigger list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WaterRef {
    Grid,
    Trigger(usize),
}

#[derive(Debug, Clone, Copy)]
struct DynamicWater {
    water: WaterRef,
    change_per_frame: Real,
    target_height: Real,
    damage_amount: Real,
    current_height: Real,
}

const IDLE_WATER: DynamicWater = DynamicWater {
    water: WaterRef::Grid,
    change_per_frame: 0.0,
    target_height: 0.0,
    damage_amount: 0.0,
    current_height: 0.0,
};

/// A bridge deck over a point.
#[derive(Debug, Clone, Copy)]
pub struct BridgeDeck {
    pub layer: PathfindLayerEnum,
    pub height: Real,
    pub rubble: Bool,
}

/// The surfaces above the ground. The defaults are a map with none.
pub trait ElevatedLayers {
    /// `Pathfinder::getWallHeight`.
    fn wall_height(&self) -> Real {
        0.0
    }
    /// `Pathfinder::isPointOnWall`.
    fn is_point_on_wall(&self, _pos: &Coord3D) -> Bool {
        FALSE
    }
    /// The bridges `pos` is on, in list order.
    fn bridges_at(&self, _pos: &Coord3D) -> Vec<BridgeDeck> {
        Vec::new()
    }
    /// `findBridgeLayerAt` then `Bridge::getBridgeHeight`.
    fn bridge_height(
        &self,
        _pos: &Coord3D,
        _layer: PathfindLayerEnum,
        _clip: Bool,
        _normal: Option<&mut Coord3D>,
    ) -> Option<Real> {
        None
    }
}

/// A map without walls or bridges.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoElevatedLayers;

impl ElevatedLayers for NoElevatedLayers {}

/// The deformable water mesh (`TheTerrainVisual`'s water grid).
pub trait WaterGrid {
    /// `getWaterGridHeight`: `None` off the grid or with the grid disabled.
    fn height_at(&self, x: Real, y: Real) -> Option<Real>;
    /// Moves the grid to `height` and returns where it was.
    fn set_height(&mut self, height: Real) -> Real;
    /// The corners of the grid in world space.
    fn corners(&self) -> [Coord3D; 4];
}

/// What a change of water height does elsewhere.
pub trait TerrainEffects {
    /// `Pathfinder::forceMapRecalculation` over the changed terrain.
    fn force_map_recalculation(&mut self, _terrain: &TerrainLogic) {}
    /// The objects within `radius` of `center` that are now underwater take
    /// `amount` water damage.
    fn damage_objects_in_water(
        &mut self,
        _terrain: &TerrainLogic,
        _center: &Coord3D,
        _radius: Real,
        _amount: Real,
    ) {
    }
}

/// No pathfinder and no objects.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoTerrainEffects;

impl TerrainEffects for NoTerrainEffects {}

/// The footprint `flattenTerrain` and `createCraterInTerrain` read from an object.
#[derive(Debug, Clone, Copy)]
pub struct TerrainShape {
    pub position: Coord3D,
    pub angle: Real,
    pub geom_type: GeometryType,
    pub is_small: Bool,
    pub major_radius: Real,
    pub minor_radius: Real,
}

/// Why a map did not load.
#[derive(Debug, Clone, PartialEq)]
pub enum MapLoadError {
    /// Not a chunk file.
    NotAMap,
    Chunk(DataChunkError),
}

impl From<DataChunkError> for MapLoadError {
    fn from(e: DataChunkError) -> Self {
        MapLoadError::Chunk(e)
    }
}

/// The map-wide terrain (`W3DTerrainLogic` and the height map of
/// `TheTerrainVisual`).
pub struct TerrainLogic {
    height_map: Option<HeightMap>,
    map_dx: Int,
    map_dy: Int,
    boundaries: Vec<ICoord2D>,
    active_boundary: Int,
    map_min_z: Real,
    map_max_z: Real,
    filename: String,
    /// In the order added; the list head is the last.
    waypoints: Vec<Waypoint>,
    triggers: PolygonTriggerList,
    water_updates: [DynamicWater; MAX_DYNAMIC_WATER],
    water_update_count: Int,
    water_grid_enabled: Bool,
    map_objects: Vec<MapObject>,
    world_dict: Dict,
    elevated: Box<dyn ElevatedLayers>,
    water_grid: Option<Box<dyn WaterGrid>>,
    bridges: BridgeList,
}

impl Default for TerrainLogic {
    fn default() -> Self {
        Self::new()
    }
}

/// What the measuring pass of a map file collects.
struct MeasurePass {
    map: HeightMap,
    objects: Vec<MapObject>,
    world_dict: Dict,
    /// The caller's generator, lent for the pass.
    name_keys: NameKeyGenerator,
}

fn parse_size_only(
    file: &mut DataChunkInput<'_, MeasurePass>,
    info: &DataChunkInfo,
    pass: &mut MeasurePass,
) -> Result<bool, DataChunkError> {
    pass.map.parse_size_only(file, info)?;
    Ok(true)
}

fn parse_world_dict(
    file: &mut DataChunkInput<'_, MeasurePass>,
    _info: &DataChunkInfo,
    pass: &mut MeasurePass,
) -> Result<bool, DataChunkError> {
    pass.world_dict = file.read_dict(&mut pass.name_keys)?;
    Ok(true)
}

fn parse_objects_list(
    file: &mut DataChunkInput<'_, MeasurePass>,
    info: &DataChunkInfo,
    pass: &mut MeasurePass,
) -> Result<bool, DataChunkError> {
    file.register_parser("Object", &info.label, parse_object);
    file.parse(pass)
}

/// `ParseObjectData`. Objects outside the height range are dropped.
fn parse_object(
    file: &mut DataChunkInput<'_, MeasurePass>,
    info: &DataChunkInfo,
    pass: &mut MeasurePass,
) -> Result<bool, DataChunkError> {
    let mut loc = Coord3D::new(file.read_real()?, file.read_real()?, file.read_real()?);
    let min_z = -100.0 * MAP_XY_FACTOR;
    let max_z = (255 * 10) as Real * MAP_HEIGHT_SCALE;
    if info.version <= K_OBJECTS_VERSION_2 {
        loc.z = 0.0;
    }
    let angle = file.read_real()?;
    let flags = file.read_int()?;
    let name = file.read_ascii_string()?;
    let dict = if info.version >= K_OBJECTS_VERSION_2 {
        file.read_dict(&mut pass.name_keys)?
    } else {
        Dict::new()
    };
    if loc.z < min_z || loc.z > max_z {
        return Ok(true);
    }
    let keys = &mut pass.name_keys;
    let mut object = MapObject::new(loc, &name, angle, flags, Some(dict), keys);
    if object.properties().get_type(keys.name_to_key("waypointID")) == DictDataType::Int {
        object.set_is_waypoint();
    }
    let light = keys.name_to_key("lightHeightAboveTerrain");
    if object.properties().get_type(light) == DictDataType::Real {
        object.set_is_light();
    }
    if object.properties().get_type(keys.name_to_key("scorchType")) == DictDataType::Int {
        object.set_is_scorch();
    }
    pass.objects.push(object);
    Ok(true)
}

fn parse_waypoint_links(
    file: &mut DataChunkInput<'_, TerrainLogic>,
    _info: &DataChunkInfo,
    terrain: &mut TerrainLogic,
) -> Result<bool, DataChunkError> {
    let count = file.read_int()?;
    for _ in 0..count {
        let first = file.read_int()?;
        let second = file.read_int()?;
        terrain.add_waypoint_link(first, second);
    }
    Ok(true)
}

fn parse_height_map(
    file: &mut DataChunkInput<'_, HeightMap>,
    info: &DataChunkInfo,
    map: &mut HeightMap,
) -> Result<bool, DataChunkError> {
    map.parse_height_map_data(file, info)?;
    Ok(true)
}

fn parse_blend_tiles(
    file: &mut DataChunkInput<'_, HeightMap>,
    info: &DataChunkInfo,
    map: &mut HeightMap,
) -> Result<bool, DataChunkError> {
    map.parse_blend_tile_data(file, info)?;
    Ok(true)
}

/// The full reading of a map that `TheTerrainVisual::load` keeps for queries.
pub fn load_logic_height_map(bytes: &[u8]) -> Result<HeightMap, MapLoadError> {
    let mut stream = CachedFileInputStream::from_bytes(bytes);
    let mut file: DataChunkInput<'_, HeightMap> = DataChunkInput::new(&mut stream);
    if !file.is_valid_file_type() {
        return Err(MapLoadError::NotAMap);
    }
    file.register_parser("HeightMapData", "", parse_height_map);
    file.register_parser("BlendTileData", "", parse_blend_tiles);
    let mut map = HeightMap::default();
    if !file.parse(&mut map)? {
        return Err(MapLoadError::NotAMap);
    }
    map.update_max_height();
    Ok(map)
}

impl TerrainLogic {
    pub fn new() -> Self {
        Self {
            height_map: None,
            map_dx: 0,
            map_dy: 0,
            boundaries: Vec::new(),
            active_boundary: 0,
            map_min_z: 0.0,
            map_max_z: 1.0,
            filename: String::new(),
            waypoints: Vec::new(),
            triggers: PolygonTriggerList::new(),
            water_updates: [IDLE_WATER; MAX_DYNAMIC_WATER],
            water_update_count: 0,
            water_grid_enabled: FALSE,
            map_objects: Vec::new(),
            world_dict: Dict::new(),
            elevated: Box::new(NoElevatedLayers),
            water_grid: None,
            bridges: BridgeList::default(),
        }
    }

    pub fn bridges(&self) -> &BridgeList {
        &self.bridges
    }

    pub fn bridges_mut(&mut self) -> &mut BridgeList {
        &mut self.bridges
    }

    /// `addBridgeToLogic` and `addLandmarkBridgeToLogic`: links the bridge and
    /// gives it a pathfind layer. Making the objects is the caller's.
    pub fn add_bridge(&mut self, mut bridge: Bridge, pathfinder: &mut Pathfinder) {
        bridge.layer = pathfinder.add_bridge(bridge.geometry, bridge.object_id);
        self.bridges.push(bridge);
    }

    /// `updateBridgeDamageStates`: closes and reopens bridge layers.
    pub fn update_bridge_damage_states(
        &mut self,
        world: &mut dyn BridgeWorld,
        pathfinder: &mut Pathfinder,
        frame: u32,
    ) {
        let mut bridges = std::mem::take(&mut self.bridges);
        let terrain: &Self = self;
        bridges.update_damage_states(world, &mut |layer, repaired| {
            pathfinder.change_bridge_state(layer, repaired, terrain, frame);
        });
        self.bridges = bridges;
    }

    /// The decks over `pos`, in list order.
    fn decks_at(&self, pos: &Coord3D) -> Vec<BridgeDeck> {
        let mut decks = self.bridges.decks_at(pos);
        decks.extend(self.elevated.bridges_at(pos));
        decks
    }

    /// `objectInteractsWithBridgeLayer`.
    pub fn object_interacts_with_bridge_layer(
        &self,
        pos: &Coord3D,
        minor_radius: Real,
        object_layer: PathfindLayerEnum,
        layer: PathfindLayerEnum,
        consider_health: bool,
    ) -> bool {
        if layer == PathfindLayerEnum::Wall {
            return object_layer == PathfindLayerEnum::Wall
                || self.elevated.is_point_on_wall(pos) != FALSE;
        }
        self.bridges
            .interacts_with_layer(pos, minor_radius, layer, consider_health)
    }

    pub fn set_elevated_layers(&mut self, elevated: Box<dyn ElevatedLayers>) {
        self.elevated = elevated;
    }

    pub fn set_water_grid(&mut self, grid: Option<Box<dyn WaterGrid>>) {
        self.water_grid = grid;
    }

    /// `reset`: waypoints, triggers, water changes and map objects.
    pub fn reset(&mut self) {
        self.waypoints.clear();
        self.triggers.delete_triggers();
        self.water_update_count = 0;
        self.map_dx = 0;
        self.map_dy = 0;
        self.map_min_z = 0.0;
        self.map_max_z = 1.0;
        self.map_objects.clear();
        self.world_dict = Dict::new();
        self.bridges.clear();
    }

    /// `loadMap` over the bytes of a map file. `query` reads the sizes,
    /// objects and waypoints without installing the height map.
    ///
    /// Waypoints take their height from the map installed before this call, as
    /// the original does; [`Self::new_map`] corrects it. `water_extent` is
    /// `GlobalData`'s, used by maps older than the water triggers.
    pub fn load_map(
        &mut self,
        bytes: &[u8],
        filename: &str,
        name_keys: &mut NameKeyGenerator,
        water_extent: ICoord2D,
        query: Bool,
    ) -> Result<(), MapLoadError> {
        let mut stream = CachedFileInputStream::from_bytes(bytes);
        let mut pass = MeasurePass {
            map: HeightMap::default(),
            objects: Vec::new(),
            world_dict: Dict::new(),
            name_keys: std::mem::take(name_keys),
        };
        let parsed = {
            let mut file: DataChunkInput<'_, MeasurePass> = DataChunkInput::new(&mut stream);
            if file.is_valid_file_type() {
                file.register_parser("HeightMapData", "", parse_size_only);
                file.register_parser("WorldInfo", "", parse_world_dict);
                file.register_parser("ObjectsList", "", parse_objects_list);
                file.parse(&mut pass)
            } else {
                Ok(false)
            }
        };
        let MeasurePass {
            map: measured,
            objects,
            world_dict,
            name_keys: keys,
        } = pass;
        *name_keys = keys;
        if !parsed? {
            return Err(MapLoadError::NotAMap);
        }
        self.map_objects = objects;
        self.world_dict = world_dict;

        self.triggers.delete_triggers();
        self.triggers
            .parse_polygon_triggers_chunk_stream(bytes, water_extent.x, water_extent.y)?;
        for trigger in self.triggers.triggers_mut() {
            trigger.update_bounds();
        }

        self.map_dx = measured.width();
        self.map_dy = measured.height();
        self.boundaries = measured.boundaries().to_vec();
        self.active_boundary = 0;
        let (_, high) = measured.height_range();
        let mut low = MAX_HEIGHT_VALUE;
        for j in 0..self.map_dy {
            for i in 0..self.map_dx {
                low = low.min(Int::from(measured.raw_height(i, j)));
            }
        }
        self.map_min_z = low as Real * MAP_HEIGHT_SCALE;
        self.map_max_z = high as Real * MAP_HEIGHT_SCALE;

        self.filename = filename.to_owned();
        let waypoint_objects: Vec<MapObject> = self
            .map_objects
            .iter()
            .filter(|o| o.is_waypoint() != FALSE)
            .cloned()
            .collect();
        for object in &waypoint_objects {
            self.add_waypoint(object, name_keys);
        }

        let mut stream = CachedFileInputStream::from_bytes(bytes);
        let mut file: DataChunkInput<'_, TerrainLogic> = DataChunkInput::new(&mut stream);
        if file.is_valid_file_type() {
            file.register_parser("WaypointsList", "", parse_waypoint_links);
            file.parse(self)?;
        }

        if query == FALSE {
            self.height_map = Some(load_logic_height_map(bytes)?);
        }
        Ok(())
    }

    /// Installs a height map and the sizes `loadMap` would take from it.
    pub fn set_height_map(&mut self, map: HeightMap) {
        self.map_dx = map.width();
        self.map_dy = map.height();
        self.boundaries = map.boundaries().to_vec();
        self.active_boundary = 0;
        let (low, high) = map.height_range();
        self.map_min_z = low as Real * MAP_HEIGHT_SCALE;
        self.map_max_z = high as Real * MAP_HEIGHT_SCALE;
        self.height_map = Some(map);
    }

    pub fn height_map(&self) -> Option<&HeightMap> {
        self.height_map.as_ref()
    }

    /// The objects of the loaded map, in file order.
    pub fn map_objects(&self) -> &[MapObject] {
        &self.map_objects
    }

    /// `MapObject::TheWorldDict`.
    pub fn world_dict(&self) -> &Dict {
        &self.world_dict
    }

    pub fn source_filename(&self) -> &str {
        &self.filename
    }

    pub fn triggers(&self) -> &PolygonTriggerList {
        &self.triggers
    }

    /// The trigger list, for the script actions that edit it. Call
    /// `update_bounds` on a polygon after moving its points.
    pub fn triggers_mut(&mut self) -> &mut PolygonTriggerList {
        &mut self.triggers
    }

    /// `newMap`: puts the waypoints on the ground and enables the water grid on
    /// the maps that carry its marker. Bridges are loaded by the caller first.
    pub fn new_map(&mut self) {
        for i in 0..self.waypoints.len() {
            let loc = self.waypoints[i].location;
            self.waypoints[i].location.z = self.ground_height(loc.x, loc.y, None);
        }
        let enable = Bool::from(self.waypoint_by_name("WaveGuide1").is_some());
        self.water_grid_enabled = enable;
    }

    pub fn is_water_grid_enabled(&self) -> Bool {
        self.water_grid_enabled
    }

    // ---- waypoints ----------------------------------------------------------

    /// `addWaypoint`.
    fn add_waypoint(&mut self, object: &MapObject, name_keys: &mut NameKeyGenerator) {
        let mut location = *object.location();
        location.z = self.ground_height(location.x, location.y, None);
        let mut label = |key: &str| {
            let key = name_keys.name_to_key(key);
            object.properties().get_ascii_string(key, None)
        };
        let path_labels = [
            label("waypointPathLabel1"),
            label("waypointPathLabel2"),
            label("waypointPathLabel3"),
        ];
        let bi_directional = object
            .properties()
            .get_bool(name_keys.name_to_key("waypointPathBiDirectional"), None);
        self.waypoints.push(Waypoint {
            id: object.waypoint_id(name_keys) as u32,
            name: object.waypoint_name(name_keys),
            location,
            path_labels,
            bi_directional,
            links: Vec::new(),
        });
    }

    /// `addWaypointLink`. With a repeated id, the last in list order is taken.
    fn add_waypoint_link(&mut self, first: Int, second: Int) {
        let mut one = None;
        let mut two = None;
        for i in (0..self.waypoints.len()).rev() {
            if self.waypoints[i].id == first as u32 {
                one = Some(i);
            }
            if self.waypoints[i].id == second as u32 {
                two = Some(i);
            }
        }
        let (Some(one), Some(two)) = (one, two) else {
            return;
        };
        if one == two || self.waypoints[one].links.contains(&two) {
            return;
        }
        self.add_link(one, two);
        if self.waypoints[one].bi_directional != FALSE && !self.waypoints[two].links.contains(&one)
        {
            self.add_link(two, one);
        }
    }

    fn add_link(&mut self, from: usize, to: usize) {
        let links = &mut self.waypoints[from].links;
        if links.len() < MAX_WAYPOINT_LINKS {
            links.push(to);
        }
    }

    /// The waypoints in list order (`getFirstWaypoint` and `getNext`).
    pub fn waypoints(&self) -> impl Iterator<Item = &Waypoint> {
        self.waypoints.iter().rev()
    }

    /// The waypoints `waypoint` links to (`getLink`).
    pub fn waypoint_links<'a>(
        &'a self,
        waypoint: &'a Waypoint,
    ) -> impl Iterator<Item = &'a Waypoint> {
        waypoint.links.iter().map(|&i| &self.waypoints[i])
    }

    /// `getWaypointByName`.
    pub fn waypoint_by_name(&self, name: &str) -> Option<&Waypoint> {
        self.waypoints().find(|w| w.name == name)
    }

    /// `getWaypointByID`.
    pub fn waypoint_by_id(&self, id: u32) -> Option<&Waypoint> {
        self.waypoints().find(|w| w.id == id)
    }

    /// `getClosestWaypointOnPath`: labels compare without case; ties keep the
    /// first in list order.
    pub fn closest_waypoint_on_path(&self, pos: &Coord3D, label: &str) -> Option<&Waypoint> {
        if label.is_empty() {
            return None;
        }
        let mut closest: Option<(&Waypoint, Real)> = None;
        for way in self.waypoints() {
            if !way
                .path_labels
                .iter()
                .any(|l| l.eq_ignore_ascii_case(label))
            {
                continue;
            }
            let dx = way.location.x - pos.x;
            let dy = way.location.y - pos.y;
            let distance = dx * dx + dy * dy;
            if closest.is_none_or(|(_, best)| distance < best) {
                closest = Some((way, distance));
            }
        }
        closest.map(|(w, _)| w)
    }

    /// `isPurposeOfPath`: labels compare with case.
    pub fn is_purpose_of_path(way: Option<&Waypoint>, label: &str) -> Bool {
        match way {
            Some(way) if !label.is_empty() => {
                Bool::from(way.path_labels.iter().any(|l| l == label))
            }
            _ => FALSE,
        }
    }

    /// `getTriggerAreaByName`.
    pub fn trigger_area_by_name(&self, name: &str) -> Option<&PolygonTrigger> {
        self.triggers
            .triggers()
            .iter()
            .find(|t| t.trigger_name == name)
    }

    // ---- extents -------------------------------------------------------------

    /// `getExtent`: the active boundary.
    pub fn extent(&self) -> Region3D {
        let mut extent = Region3D::default();
        if !self.boundaries.is_empty() {
            if let Some(b) = self.boundaries.get(self.active_boundary as usize) {
                extent.hi.x = b.x as Real * MAP_XY_FACTOR;
                extent.hi.y = b.y as Real * MAP_XY_FACTOR;
            }
        }
        extent.lo.z = self.map_min_z;
        extent.hi.z = self.map_max_z;
        extent
    }

    /// `getMaximumPathfindExtent`: the largest of the boundaries.
    pub fn maximum_pathfind_extent(&self) -> Region3D {
        let mut extent = Region3D::default();
        for b in &self.boundaries {
            if extent.hi.x < b.x as Real * MAP_XY_FACTOR {
                extent.hi.x = b.x as Real * MAP_XY_FACTOR;
            }
            if extent.hi.y < b.y as Real * MAP_XY_FACTOR {
                extent.hi.y = b.y as Real * MAP_XY_FACTOR;
            }
        }
        extent.lo.z = self.map_min_z;
        extent.hi.z = self.map_max_z;
        extent
    }

    /// `getExtentIncludingBorder`. The original leaves `z` untouched; it is
    /// zero here.
    pub fn extent_including_border(&self) -> Region3D {
        let border = self.height_map.as_ref().map_or(0, HeightMap::border_size);
        let border = border as Real * MAP_XY_FACTOR;
        let mut extent = Region3D::default();
        extent.lo.x -= border;
        extent.lo.y -= border;
        extent.hi.x = (self.map_dx as Real * MAP_XY_FACTOR) - border;
        extent.hi.y = (self.map_dy as Real * MAP_XY_FACTOR) - border;
        extent
    }

    pub fn active_boundary(&self) -> Int {
        self.active_boundary
    }

    /// The state change of `setActiveBoundary`; true when it changed.
    pub fn set_active_boundary(&mut self, boundary: Int) -> Bool {
        let Some(size) = usize::try_from(boundary)
            .ok()
            .and_then(|i| self.boundaries.get(i))
        else {
            return FALSE;
        };
        if boundary == self.active_boundary || size.x == 0 || size.y == 0 {
            return FALSE;
        }
        self.active_boundary = boundary;
        TRUE
    }

    /// `findClosestEdgePoint`.
    pub fn closest_edge_point(&self, closest_to: &Coord3D) -> Coord3D {
        find_closest_edge_point(self, &self.extent(), closest_to)
    }

    /// `findFarthestEdgePoint`.
    pub fn farthest_edge_point(&self, farthest_from: &Coord3D) -> Coord3D {
        find_farthest_edge_point(self, &self.extent(), farthest_from)
    }

    // ---- heights -------------------------------------------------------------

    fn ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        match &self.height_map {
            Some(map) => map.ground_height(x, y, normal),
            None => {
                if let Some(n) = normal {
                    *n = Coord3D::new(0.0, 0.0, 1.0);
                }
                0.0
            }
        }
    }

    // ---- water ---------------------------------------------------------------

    /// `getWaterHandle`: the highest water over the point.
    pub fn water_at(&self, x: Real, y: Real) -> Option<WaterRef> {
        let mut found = None;
        let mut water_z: Real = 0.0;
        let location = ICoord3D::new(real_to_int_floor(x + 0.5), real_to_int_floor(y + 0.5), 0);
        for (i, trigger) in self.triggers.triggers().iter().enumerate() {
            if trigger.is_water_area == FALSE || trigger.contains_point(location) == FALSE {
                continue;
            }
            let z = trigger.get_point(0).map_or(0, |p| p.z) as Real;
            if z >= water_z {
                water_z = z;
                found = Some(WaterRef::Trigger(i));
            }
        }
        if let Some(z) = self.water_grid.as_ref().and_then(|g| g.height_at(x, y)) {
            if z >= water_z {
                found = Some(WaterRef::Grid);
            }
        }
        found
    }

    /// `getWaterHandleByName`.
    pub fn water_by_name(&self, name: &str) -> Option<WaterRef> {
        if name == WATER_GRID_NAME {
            return Some(WaterRef::Grid);
        }
        self.triggers
            .triggers()
            .iter()
            .position(|t| t.trigger_name == name && t.is_water_area != FALSE)
            .map(WaterRef::Trigger)
    }

    /// `getWaterHeight`. Zero for the grid, which cannot be asked this way.
    pub fn water_height(&self, water: Option<WaterRef>) -> Real {
        match water {
            Some(WaterRef::Trigger(i)) => self
                .triggers
                .triggers()
                .get(i)
                .and_then(|t| t.get_point(0))
                .map_or(0.0, |p| p.z as Real),
            _ => 0.0,
        }
    }

    /// `findAxisAlignedBoundingRect`. `z` starts from zero.
    pub fn water_bounding_rect(&self, water: WaterRef) -> Region3D {
        const BUFFER: Real = 99999.9;
        let map = self.extent();
        let mut region = Region3D::default();
        region.lo.x = map.hi.x + BUFFER;
        region.lo.y = map.hi.y + BUFFER;
        region.hi.x = map.lo.x - BUFFER;
        region.hi.y = map.lo.y - BUFFER;
        let mut cover = |x: Real, y: Real| {
            if x < region.lo.x {
                region.lo.x = x;
            }
            if x > region.hi.x {
                region.hi.x = x;
            }
            if y < region.lo.y {
                region.lo.y = y;
            }
            if y > region.hi.y {
                region.hi.y = y;
            }
        };
        match water {
            WaterRef::Grid => {
                if let Some(grid) = &self.water_grid {
                    for corner in grid.corners() {
                        cover(corner.x, corner.y);
                    }
                }
            }
            WaterRef::Trigger(i) => {
                let points = self
                    .triggers
                    .triggers()
                    .get(i)
                    .map_or(&[][..], |t| &t.points[..]);
                for p in points {
                    cover(p.x as Real, p.y as Real);
                }
                for p in points {
                    if (p.z as Real) < region.lo.z {
                        region.lo.z = p.z as Real;
                    }
                    if p.z as Real > region.hi.z {
                        region.hi.z = p.z as Real;
                    }
                }
            }
        }
        region
    }

    /// `setWaterHeight`. A polygon holds whole heights, so the new one is
    /// truncated.
    pub fn set_water_height(
        &mut self,
        water: Option<WaterRef>,
        height: Real,
        damage_amount: Real,
        force_pathfind_update: Bool,
        effects: &mut dyn TerrainEffects,
    ) {
        let Some(water) = water else {
            return;
        };
        let mut height = height;
        let previous = match water {
            WaterRef::Grid => self
                .water_grid
                .as_mut()
                .map_or(0.0, |g| g.set_height(height)),
            WaterRef::Trigger(i) => {
                let previous = self.water_height(Some(water));
                if let Some(trigger) = self.triggers.triggers_mut().get_mut(i) {
                    for n in 0..trigger.num_points() {
                        if let Some(mut p) = trigger.get_point(n) {
                            p.z = height as Int;
                            trigger.set_point(p, n);
                        }
                    }
                    trigger.update_bounds();
                }
                height = self.water_height(Some(water));
                previous
            }
        };
        let region = self.water_bounding_rect(water);
        if force_pathfind_update != FALSE || previous != height {
            effects.force_map_recalculation(self);
        }
        if damage_amount > 0.0 && height > previous {
            let center = Coord3D::new(
                region.lo.x + region.width() / 2.0,
                region.lo.y + region.height() / 2.0,
                0.0,
            );
            let radius =
                gm_sqrtf(region.width() * region.width() + region.height() * region.height());
            effects.damage_objects_in_water(self, &center, radius, damage_amount);
        }
    }

    /// `changeWaterHeightOverTime`.
    pub fn change_water_height_over_time(
        &mut self,
        water: Option<WaterRef>,
        final_height: Real,
        seconds: Real,
        damage_amount: Real,
    ) {
        if self.water_update_count >= MAX_DYNAMIC_WATER as Int {
            return;
        }
        let Some(water) = water else {
            return;
        };
        let mut i: Int = 0;
        while i < self.water_update_count {
            if self.water_updates[i as usize].water == water {
                self.water_updates[i as usize] =
                    self.water_updates[(self.water_update_count - 1) as usize];
                self.water_update_count -= 1;
                i -= 1;
            }
            i += 1;
        }
        let current = self.water_height(Some(water));
        self.water_updates[self.water_update_count as usize] = DynamicWater {
            water,
            change_per_frame: (final_height - current)
                / (LOGIC_FRAMES_PER_SECOND as Real * seconds),
            target_height: final_height,
            damage_amount,
            current_height: current,
        };
        self.water_update_count += 1;
    }

    /// The number of water areas still moving.
    pub fn moving_water_count(&self) -> Int {
        self.water_update_count
    }

    /// `update`: moves the water. Damage is dealt once a second.
    pub fn update(&mut self, frame: u32, effects: &mut dyn TerrainEffects) {
        if self.water_update_count == 0 {
            return;
        }
        let damage_frame = frame % LOGIC_FRAMES_PER_SECOND == 0;
        let mut i = self.water_update_count - 1;
        while i >= 0 {
            let slot = i as usize;
            let entry = self.water_updates[slot];
            let next = entry.current_height + entry.change_per_frame;
            let done = if entry.change_per_frame > 0.0 {
                next >= entry.target_height
            } else {
                next <= entry.target_height
            };
            if done {
                self.set_water_height(
                    Some(entry.water),
                    entry.target_height,
                    entry.damage_amount,
                    TRUE,
                    effects,
                );
                // Copies every later entry into this slot, leaving the last.
                for j in i..self.water_update_count {
                    self.water_updates[slot] = self.water_updates[j as usize];
                }
                self.water_update_count -= 1;
            } else {
                let damage = if damage_frame {
                    entry.damage_amount
                } else {
                    0.0
                };
                self.water_updates[slot].current_height = next;
                self.set_water_height(Some(entry.water), next, damage, FALSE, effects);
            }
            i -= 1;
        }
    }

    // ---- changing the ground -------------------------------------------------

    /// Lowers the sample at `(i, j)` and its eight neighbours.
    fn lower_around(&mut self, i: Int, j: Int, height: Int) {
        let Some(map) = &mut self.height_map else {
            return;
        };
        for (dx, dy) in [
            (0, 0),
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
        ] {
            map.set_raw_map_height(
                ICoord2D {
                    x: i.wrapping_add(dx),
                    y: j.wrapping_add(dy),
                },
                height,
            );
        }
    }

    /// `flattenTerrain`: levels the ground under a structure to the average of
    /// the samples it covers, only ever lowering.
    pub fn flatten_terrain(&mut self, shape: &TerrainShape) {
        if shape.is_small != FALSE {
            return;
        }
        let pos = shape.position;
        let floor_cell = |v: Real| real_to_int_floor(v / MAP_XY_FACTOR);
        let is_box = shape.geom_type == GeometryType::Box;
        let (corners, mut min, max);
        if is_box {
            let hx = shape.major_radius;
            let hy = shape.minor_radius;
            let c = gm_cosf(shape.angle);
            let s = gm_sinf(shape.angle);
            let top_left = [pos.x - hx * c - hy * s, pos.y + hy * c - hx * s];
            let top_right = [pos.x + hx * c - hy * s, pos.y + hy * c + hx * s];
            let bottom_right = [pos.x + hx * c + hy * s, pos.y - hy * c + hx * s];
            let bottom_left = [pos.x - hx * c + hy * s, pos.y - hy * c - hx * s];
            corners = [top_left, top_right, bottom_right, bottom_left];
            let mut low = top_left;
            let mut high = top_left;
            for p in &corners[1..] {
                for axis in 0..2 {
                    if low[axis] > p[axis] {
                        low[axis] = p[axis];
                    }
                    if high[axis] < p[axis] {
                        high[axis] = p[axis];
                    }
                }
            }
            min = ICoord2D {
                x: floor_cell(low[0]),
                y: floor_cell(low[1]),
            };
            max = ICoord2D {
                x: floor_cell(high[0]),
                y: floor_cell(high[1]),
            };
        } else {
            corners = [[0.0; 2]; 4];
            let r = shape.major_radius;
            min = ICoord2D {
                x: floor_cell(pos.x - r),
                y: floor_cell(pos.y - r),
            };
            max = ICoord2D {
                x: floor_cell(pos.x + r),
                y: floor_cell(pos.y + r),
            };
        }
        if cfg!(feature = "retail-crc") {
            min.y = min.y.max(0);
        }

        let radius_sqr = shape.major_radius * shape.major_radius;
        let covers = |x: Real, y: Real| {
            if is_box {
                let [top_left, top_right, bottom_right, bottom_left] = corners;
                point_in_triangle(top_left, top_right, bottom_left, [x, y])
                    || point_in_triangle(top_right, bottom_right, bottom_left, [x, y])
            } else {
                let dx = x - pos.x;
                let dy = y - pos.y;
                dx * dx + dy * dy < radius_sqr
            }
        };

        let mut total: Real = 0.0;
        let mut samples: Int = 0;
        for i in min.x..=max.x {
            for j in min.y..=max.y {
                let (x, y) = (i as Real * MAP_XY_FACTOR, j as Real * MAP_XY_FACTOR);
                if covers(x, y) {
                    total += self.ground_height(x, y, None);
                    samples += 1;
                }
            }
        }
        if samples == 0 {
            return;
        }
        let average = total / samples as Real;
        let mut raw = real_to_int_floor(0.5 + average / MAP_HEIGHT_SCALE);
        if is_box {
            // A box also never ends above the ground at its centre.
            let center =
                real_to_int_floor(self.ground_height(pos.x, pos.y, None) / MAP_HEIGHT_SCALE);
            if raw > center {
                raw = center;
            }
        }
        for i in min.x..=max.x {
            for j in min.y..=max.y {
                if covers(i as Real * MAP_XY_FACTOR, j as Real * MAP_XY_FACTOR) {
                    self.lower_around(i, j, raw);
                }
            }
        }
    }

    /// `createCraterInTerrain`.
    pub fn create_crater_in_terrain(&mut self, shape: &TerrainShape) {
        if shape.is_small != FALSE || self.height_map.is_none() {
            return;
        }
        let pos = shape.position;
        let radius = shape.major_radius;
        if radius <= 0.0 || !radius.is_finite() || !pos.x.is_finite() || !pos.y.is_finite() {
            return;
        }
        let extent = self.extent();
        if extent.hi.x <= extent.lo.x || extent.hi.y <= extent.lo.y {
            return;
        }
        let larger = |a: Real, b: Real| if a > b { a } else { b };
        let smaller = |a: Real, b: Real| if a < b { a } else { b };
        let min_x = larger(extent.lo.x, pos.x - radius);
        let max_x = smaller(extent.hi.x, pos.x + radius);
        let min_y = larger(extent.lo.y, pos.y - radius);
        let max_y = smaller(extent.hi.y, pos.y + radius);
        if min_x > max_x || min_y > max_y {
            return;
        }
        let floor_cell = |v: Real| real_to_int_floor(v / MAP_XY_FACTOR);
        let min = ICoord2D {
            x: floor_cell(min_x),
            y: floor_cell(min_y),
        };
        let max = ICoord2D {
            x: floor_cell(max_x),
            y: floor_cell(max_y),
        };
        let min_row = if cfg!(feature = "retail-crc") {
            min.y.max(0)
        } else {
            min.y
        };
        let Some(map) = &mut self.height_map else {
            return;
        };
        for i in min.x..=max.x {
            for j in min_row..=max.y {
                let dx = (i as Real * MAP_XY_FACTOR) - pos.x;
                let dy = (j as Real * MAP_XY_FACTOR) - pos.y;
                let distance = gm_sqrtf(dx * dx + dy * dy);
                if distance < radius {
                    let cell = ICoord2D { x: i, y: j };
                    let displacement = radius * (1.0 - distance / radius);
                    let lowered = map.raw_map_height(cell) as Real - displacement;
                    let target = if 1.0 > lowered { 1.0 } else { lowered };
                    map.set_raw_map_height(cell, target as Int);
                }
            }
        }
    }
}

/// `Point_In_Triangle_2D` on the first two axes.
fn point_in_triangle(p0: [Real; 2], p1: [Real; 2], p2: [Real; 2], test: [Real; 2]) -> bool {
    let sub = |a: [Real; 2], b: [Real; 2]| [a[0] - b[0], a[1] - b[1]];
    let perp = |a: [Real; 2], b: [Real; 2]| a[0] * -b[1] + a[1] * b[0];
    let length2 = |a: [Real; 2]| a[0] * a[0] + a[1] * a[1];
    let p0p1 = sub(p1, p0);
    let p1p2 = sub(p2, p1);
    let p2p0 = sub(p0, p2);
    let area = perp(p0p1, sub(p2, p0));
    if area != 0.0 {
        let side: Real = if area > 0.0 { 1.0 } else { -1.0 };
        return !(perp(p0p1, sub(test, p0)) * side < 0.0
            || perp(p1p2, sub(test, p1)) * side < 0.0
            || perp(p2p0, sub(test, p2)) * side < 0.0);
    }
    // A degenerate triangle is its longest edge. The third length is read from
    // the second edge.
    let d01 = length2(p0p1);
    let d12 = length2(p1p2);
    let d20 = length2(p1p2);
    let (end, from_start, longest) = if d01 > d12 {
        if d01 > d20 {
            (p0p1, sub(test, p0), d01)
        } else {
            (p2p0, sub(test, p2), d20)
        }
    } else if d12 > d20 {
        (p1p2, sub(test, p1), d12)
    } else {
        (p2p0, sub(test, p2), d20)
    };
    if longest != 0.0 {
        if perp(end, from_start) != 0.0 {
            return false;
        }
        length2(from_start) <= longest && length2(sub(from_start, end)) <= longest
    } else {
        length2(from_start) == 0.0
    }
}

impl ThingTerrainAccess for TerrainLogic {
    fn playable_extent(&self) -> Option<Region3D> {
        Some(self.extent())
    }

    /// `getGroundHeight`.
    fn terrain_logic(&self) -> Option<&TerrainLogic> {
        Some(self)
    }

    fn terrain_logic_mut(&mut self) -> Option<&mut TerrainLogic> {
        Some(self)
    }

    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        self.ground_height(x, y, normal)
    }

    /// `alignOnTerrain`.
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        let layer = self.get_layer_for_destination(pos);
        let mut normal = Coord3D::default();
        let mut height = self.get_layer_height(pos.x, pos.y, layer, Some(&mut normal), TRUE);
        if layer != PathfindLayerEnum::Ground {
            height += 2.5;
        }
        make_align_to_normal_matrix(angle, pos, &normal, mtx);
        if stick_to_ground != FALSE {
            mtx.set_z_translation(height);
        }
        layer
    }

    /// `isCliffCell`.
    fn is_cliff_cell(&self, x: Real, y: Real) -> Bool {
        self.height_map
            .as_ref()
            .map_or(FALSE, |m| m.is_cliff_cell(x, y))
    }

    /// `isClearLineOfSight`.
    fn is_clear_line_of_sight(&self, pos: &Coord3D, other: &Coord3D) -> Bool {
        self.height_map
            .as_ref()
            .map_or(FALSE, |m| m.is_clear_line_of_sight(pos, other))
    }

    /// `isUnderwater`. `water_z` is only written where there is water.
    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        let Some(water) = self.water_at(x, y) else {
            if let Some(tz) = terrain_z {
                *tz = self.ground_height(x, y, None);
            }
            return FALSE;
        };
        let level = match water {
            WaterRef::Grid => self
                .water_grid
                .as_ref()
                .and_then(|g| g.height_at(x, y))
                .unwrap_or(0.0),
            WaterRef::Trigger(_) => self.water_height(Some(water)),
        };
        if let Some(wz) = water_z {
            *wz = level;
        }
        let ground = self.ground_height(x, y, None);
        if let Some(tz) = terrain_z {
            *tz = ground;
        }
        Bool::from(ground < level)
    }

    /// `getLayerForDestination`: the surface nearest in height.
    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum {
        nearest_layer(
            pos,
            self.ground_height(pos.x, pos.y, None),
            self.elevated.wall_height(),
            || self.elevated.is_point_on_wall(pos) != FALSE,
            &self.decks_at(pos),
        )
    }

    /// `getLayerHeight`.
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        mut normal: Option<&mut Coord3D>,
        clip: Bool,
    ) -> Real {
        let height = self.ground_height(x, y, normal.as_deref_mut());
        if self.height_map.is_none() || layer == PathfindLayerEnum::Ground {
            return height;
        }
        let loc = Coord3D::new(x, y, height);
        if layer == PathfindLayerEnum::Wall {
            if clip == FALSE || self.elevated.is_point_on_wall(&loc) != FALSE {
                return self.elevated.wall_height();
            }
            return height;
        }
        let found = match self.bridges.find_layer_at(&loc, layer, clip != FALSE) {
            Some(bridge) => Some(bridge.geometry.height(&loc, normal)),
            None => self.elevated.bridge_height(&loc, layer, clip, normal),
        };
        match found {
            Some(deck) if deck > height => deck,
            _ => height,
        }
    }

    /// `getHighestLayerForDestination`: the highest surface not above `pos`.
    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        only_healthy_bridges: Bool,
    ) -> PathfindLayerEnum {
        highest_layer(
            pos,
            self.ground_height(pos.x, pos.y, None),
            self.elevated.wall_height(),
            || self.elevated.is_point_on_wall(pos) != FALSE,
            &self.decks_at(pos),
            only_healthy_bridges != FALSE,
        )
    }
}

impl MapTerrain for TerrainLogic {
    fn get_extent(&self) -> Region3D {
        self.extent()
    }

    fn get_extent_including_border(&self) -> Region3D {
        self.extent_including_border()
    }

    fn get_maximum_pathfind_extent(&self) -> Region3D {
        self.maximum_pathfind_extent()
    }

    fn first_waypoint(&self) -> Option<u32> {
        self.waypoints().next().map(|w| w.id)
    }

    fn waypoint_by_id(&self, id: u32) -> Option<WaypointInfo> {
        TerrainLogic::waypoint_by_id(self, id).map(|w| WaypointInfo {
            id: w.id,
            name: w.name.clone(),
            location: w.location,
            links: self.waypoint_links(w).map(|l| l.id).collect(),
        })
    }

    fn waypoint_by_name(&self, name: &str) -> Option<u32> {
        TerrainLogic::waypoint_by_name(self, name).map(|w| w.id)
    }
}

/// `W3DTerrainLogic::xfer` around `TerrainLogic::xfer`. `crc` is empty.
impl Snapshot for TerrainLogic {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut version = 2;
        xfer.xfer_version(&mut version, 2)?;
        let mut boundary = self.active_boundary;
        xfer.xfer_int(&mut boundary)?;
        if xfer.mode() == XferMode::Load {
            self.set_active_boundary(boundary);
        }
        if version < 2 {
            return Ok(());
        }
        xfer.xfer_int(&mut self.water_update_count)?;
        let count = usize::try_from(self.water_update_count)
            .ok()
            .filter(|&n| n <= MAX_DYNAMIC_WATER)
            .ok_or(XferError::InvalidData)?;
        for i in 0..count {
            match xfer.mode() {
                XferMode::Save => {
                    let mut id = match self.water_updates[i].water {
                        WaterRef::Trigger(t) => self
                            .triggers
                            .triggers()
                            .get(t)
                            .map(PolygonTrigger::id)
                            .ok_or(XferError::InvalidData)?,
                        WaterRef::Grid => return Err(XferError::InvalidData),
                    };
                    xfer.xfer_int(&mut id)?;
                }
                XferMode::Load => {
                    let mut id = 0;
                    xfer.xfer_int(&mut id)?;
                    let found = self
                        .triggers
                        .triggers()
                        .iter()
                        .position(|t| t.id() == id)
                        .ok_or(XferError::InvalidData)?;
                    if self.triggers.triggers()[found].is_water_area == FALSE {
                        return Err(XferError::InvalidData);
                    }
                    self.water_updates[i].water = WaterRef::Trigger(found);
                }
                _ => {}
            }
            let entry = &mut self.water_updates[i];
            xfer.xfer_real(&mut entry.change_per_frame)?;
            xfer.xfer_real(&mut entry.target_height)?;
            xfer.xfer_real(&mut entry.damage_amount)?;
            xfer.xfer_real(&mut entry.current_height)?;
        }
        Ok(())
    }
}
