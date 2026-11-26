//! `WP-4`: Phase 4 spatial & world map subsystems (`CaveSystem`, `CrateSystem`,
//! the partition grid, `PolygonTrigger`, `Powers`, `SidesList`, `Terrain`, `TerrainTypes`).

pub mod cave_system;
pub mod crate_system;
pub mod partition_collide;
pub mod partition_grid;
pub mod partition_manager;
pub mod partition_query;
pub mod partition_shroud;
pub mod polygon_trigger;
pub mod powers;
pub mod sides_list;
pub mod terrain;
pub mod terrain_types;

pub use cave_system::*;
pub use crate_system::*;
pub use partition_collide::*;
pub use partition_grid::*;
pub use partition_query::*;
pub use partition_shroud::*;
pub use polygon_trigger::*;
pub use powers::*;
pub use sides_list::*;
pub use terrain::*;
pub use terrain_types::*;
