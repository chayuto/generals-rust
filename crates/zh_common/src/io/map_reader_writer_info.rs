//! PKT-459 `MapReaderWriterInfo`: map chunk versions and size limits
//! (`Core/GameEngine/Include/Common/MapReaderWriterInfo.h`). The stream
//! interfaces of that header are `ChunkInputStream` in [`super::data_chunk`].
//!
//! Golden test: `crates/zh_logic/tests/golden_s1_terrain.rs`.

use crate::types::Int;

pub const K_HEIGHT_MAP_VERSION_1: u16 = 1;
pub const K_HEIGHT_MAP_VERSION_2: u16 = 2;
pub const K_HEIGHT_MAP_VERSION_3: u16 = 3;
pub const K_HEIGHT_MAP_VERSION_4: u16 = 4;

pub const K_BLEND_TILE_VERSION_1: u16 = 1;
pub const K_BLEND_TILE_VERSION_2: u16 = 2;
pub const K_BLEND_TILE_VERSION_3: u16 = 3;
pub const K_BLEND_TILE_VERSION_4: u16 = 4;
pub const K_BLEND_TILE_VERSION_5: u16 = 5;
pub const K_BLEND_TILE_VERSION_6: u16 = 6;
pub const K_BLEND_TILE_VERSION_7: u16 = 7;
pub const K_BLEND_TILE_VERSION_8: u16 = 8;

pub const K_OBJECTS_VERSION_1: u16 = 1;
pub const K_OBJECTS_VERSION_2: u16 = 2;
pub const K_OBJECTS_VERSION_3: u16 = 3;

pub const K_MAP_OBJECT_VERSION_1: u16 = 1;
pub const K_WAYPOINTS_VERSION_1: u16 = 1;
pub const K_PLAYERLIST_VERSION_1: u16 = 1;

pub const K_TRIGGERS_VERSION_1: u16 = 1;
pub const K_TRIGGERS_VERSION_2: u16 = 2;
pub const K_TRIGGERS_VERSION_3: u16 = 3;
pub const K_TRIGGERS_VERSION_4: u16 = 4;

pub const K_LIGHTING_VERSION_1: u16 = 1;
pub const K_LIGHTING_VERSION_2: u16 = 2;
pub const K_LIGHTING_VERSION_3: u16 = 3;

pub const K_WORLDDICT_VERSION_1: u16 = 1;
pub const K_MAPPREVIEW_VERSION_1: u16 = 1;

/// The shroud texture spans the map and shoreline tiles pack each coordinate
/// into 16 bits.
pub const MAX_MAP_DIMENSION: Int = 8192;
/// Terrain, pathfinding and partition arrays allocate per cell.
pub const MAX_MAP_AREA: Int = 4096 * 4096;
