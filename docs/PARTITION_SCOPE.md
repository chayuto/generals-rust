# Spatial grid (`PKT-160 PartitionManager`) scope

Stages 1 to 3 are done: cell filing (`spatial/partition_grid.rs`), searches (`spatial/partition_query.rs`), collisions (`spatial/partition_collide.rs`) and their wiring into the sim (`SimWorld::partition`, golden `golden_partition_wiring`). Stage 4, shroud and vision, is not started. `PKT-160` stays unported until it is.

## Ordering rules a port must keep

- `registerObject` only marks the object dirty. It is in no cell until `PartitionManager::update`.
- `GameLogic::update` runs sleepy modules, AI, build assistant, **then** the grid update, then the destroy list. Modules therefore see cells as filed at the end of the previous frame.
- The dirty list and each cell's list insert at the head.
- A query walks precomputed rings of cells outward from the centre, then each cell's list, skipping objects already seen.
- `update` also detects collisions and runs `onCollide` for what it finds.

## Stages

| Stage | Content | C++ lines | State |
|---|---|---:|---|
| 1 | Cells, geometry fill, dirty list, radius rings, range and closest queries, four distance rules | ~1,800 to 2,200 | Done: `golden_partition_grid`, `golden_partition_query` |
| 2 | Sim queries on the grid, registration, dirty marking, collisions in the frame | Rust only | Done: `golden_partition_wiring` |
| 3 | Collision tests and contact list | ~600 | Done: `golden_partition_collide` |
| 4 | Shroud and vision | ~700 | Not started. `GameLogic::getCRC` hashes it, so replay gates need it |

Not needed yet: threat and value maps (~340), position finding and line of sight (~380). Query filters (~500) are added per caller.

## Using the wired grid

- A frame is `step_sleepy_updates`, `update_partition`, `process_destroy_list`. There is no frame driver yet, so callers run the three.
- A new object is invisible to searches until the next `update_partition`. Destroyed objects stay visible until the destroy list is processed, so a caller must apply the C++ caller's own filters (`PartitionFilterAlive` is `!isEffectivelyDead` only).
- Every search comparison is strict. Match a new caller to its C++ distance rule, order type, filters and from-object or from-position form.
- `init_partition` is a map boundary change: it sets the extent and registers every object. A position on the map edge counts as off the map.
- The default is 40-unit cells (retail `GameData.ini`) over 0 to 1000. The shared C++ harness world uses 20-unit cells, so its suites call `init_partition(20.0, ..)`.

## Not mirrored or not verified

- Containment registration (`set_in_partition` from the contain modules and the hijacker crate) has no oracle of its own; the existing contain suites pass.
- No Rust counterpart yet: the register and unregister calls in `TunnelContain`, `CaveContain`, `TunnelTracker::loadPostProcess`, `HijackerUpdate` and `DeliverPayloadAIUpdate`, and the re-filing on script disabled or unpowered changes. Add them when porting those.
- `Object::setStatus` also detonates mines on an `UnderConstruction` change. Not ported.
- `onPartitionCellChange` (shroud, threat and value maps) is not ported.
- `iteratePotentialCollisions` and `PartitionFilterWouldCollide` are not ported; `geom_collides_with_geom` is.
- Ghost objects are not modelled.
- Terrain is not consulted for cell heights; stage 4 will need it.

## Values the original leaves uninitialised

When two non-sphere shapes are close enough to skip the XY test, the C++ never writes `loc.x`, `loc.y` or `normal`, so `onCollide` receives stack leftovers. The port passes zeros (negative zeros for the second object's normal) and the vectors ignore those values. A scan of the `onCollide` bodies in the original's game engine found none that reads the normal, and only `RailroadBehavior` reads `loc`, for an impact sound. Recheck when porting a collide module.
