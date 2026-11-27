//! The terrain of a map: the logic height map, `TerrainLogic` without bridges,
//! and the loader for the map file chunks they read.
//!
//! Golden test: `crates/zh_logic/tests/golden_s1_terrain.rs`.

pub mod bridge;
pub mod height_map;
pub mod terrain_logic;

pub use bridge::{
    highest_layer, line_in_region, nearest_layer, Bridge, BridgeGeometry, BridgeList, BridgeWorld,
    DeckObject,
};
pub use height_map::HeightMap;
pub use terrain_logic::{
    BridgeDeck, ElevatedLayers, MapLoadError, NoElevatedLayers, NoTerrainEffects, TerrainEffects,
    TerrainLogic, TerrainShape, WaterGrid, WaterRef, Waypoint,
};
