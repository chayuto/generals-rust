//! `PKT-160`: `PartitionManager` (`PartitionManager.h` / `PartitionManager.cpp`).
//!
//! The partition grid implementation is split across the [`crate::spatial`]
//! modules and wired into [`crate::sim::SimContext`]:
//!
//! - Cell filing, dirty list, radius rings, and map boundary rebuild:
//!   [`crate::spatial::partition_grid`] (`PartitionGrid`, `PartitionObject`).
//! - Range, closest, and iterating queries: [`crate::spatial::partition_query`]
//!   (`DistanceCalculationType`, `IterOrderType`, `QueryCenter`, `QueryMatch`).
//! - Collision detection and contact processing:
//!   [`crate::spatial::partition_collide`] (`CollisionHost`, `geom_collides_with_geom`,
//!   `process_contacts`).
//! - Cell shroud levels, circle reveal/cover, delayed unlooks, map-wide reveals,
//!   store/restore, `crc`, and `xfer`: [`crate::spatial::partition_shroud`]
//!   (`ShroudCell`, `ShroudLevel`, `CellShroudStatus`, `SightingInfo`,
//!   `ShroudStatusStoreRestore`).
//! - Object-side looking, jamming (`shroud`), vision/shroud/spied ranges, and
//!   refresh sites: [`crate::sim::SimContext`] and [`crate::sim::ObjectRecord`].
//!
//! Deferred (see `docs/PARTITION_SCOPE.md`): per-object shroud status cache
//! (`PartitionData::getShroudedStatus`), ghost objects, threat and value maps,
//! `iteratePotentialCollisions`, position finding, and line-of-sight queries.

pub use crate::spatial::partition_collide::*;
pub use crate::spatial::partition_grid::*;
pub use crate::spatial::partition_query::*;
pub use crate::spatial::partition_shroud::*;
