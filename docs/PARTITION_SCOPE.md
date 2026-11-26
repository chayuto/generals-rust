# Spatial grid (`PKT-160 PartitionManager`) scope

Stages 1 and 3 are ported and not wired into the sim: cell filing (`spatial/partition_grid.rs`, golden `golden_partition_grid`), searches (`spatial/partition_query.rs`, golden `golden_partition_query`) and collisions (`spatial/partition_collide.rs`, golden `golden_partition_collide`). Stage 2, the wiring, is next. Sizes are approximate line counts of `PartitionManager.cpp` (5,921 lines; the header adds 1,601), grouped by function name.

## Why it matters

- `find_objects_in_range` stands in for the grid with a walk of the world list. It returns a different order, so `PKT-291 PropagandaTowerBehavior` is not bit-exact and `PKT-388 WeaponBonusUpdate` was dropped.
- It has no cells, so `PKT-333 HeightDieUpdate` can find an object that C++ never visits.
- In the original, 43 game-logic files run a range query. `iterateObjectsInRange` has 40 call sites, `getClosestObject` 35.
- `GameLogic::getCRC` hashes the grid (each cell's shroud levels and coordinates), so replay gates need it.

## Ordering rules a port must keep

- `registerObject` only marks the object dirty. It is in no cell until `PartitionManager::update`.
- `GameLogic::update` runs sleepy modules, AI, build assistant, **then** the grid update, then the destroy list. Modules therefore see cells as filed at the end of the previous frame.
- The dirty list and each cell's list insert at the head.
- A query walks precomputed rings of cells outward from the centre, then each cell's list, skipping objects already seen.
- `update` also detects collisions and runs `onCollide` for what it finds.

## Stages

| Stage | Content | C++ lines | Verifiable by |
|---|---|---:|---|
| 1 | Cells, geometry fill (small, circle, rectangle), dirty list, radius rings, range and closest queries, four distance rules | ~1,800 to 2,200 | Oracle on the real `PartitionManager`: cell lists and result order |
| 2 | Move the port's queries onto the grid and run its update at the right point in the frame | Rust only | Existing golden suites, without `golden_wp53_c3`'s id sorting |
| 3 | Collision tests and contact list | ~600 | Oracle on contacts and `onCollide` order |
| 4 | Shroud and vision | ~700 | Oracle and grid CRC |

Not needed yet: threat and value maps (~340), position finding and line of sight (~380). Query filters (~500) can be added per caller.

Stage 3 was ported before stage 2 because resolving a collision re-queues both mobile objects (`processContactList` calls `makeDirty(false)`), which changes the order of later filing and so of search results. Wire both together.

## For wiring into the sim (stage 2)

- The caller supplies per object: id, position, angle, geometry, immobile flag and the raw template build cost.
- The sim's `find_objects_in_range` is inclusive (`<=`), measures 2D centre distance, skips destroyed objects and walks the world list. It is not a drop-in swap: match each caller to its C++ distance rule, order type, filters and from-object or from-position form.
- All grid distance comparisons are strict. Objects awaiting the destroy list stay registered.
- A closest search finishes the ring where it first accepts an object, so it is not always the nearest.
- Mark an object dirty only where C++ does: `reactToTransformChange` (past its thresholds) and the other `makeDirty(true)` sites in `Object.cpp`, including script disabled and unpowered changes.
- `setGeometryInfo` is unregister then register. `setGeometryInfoZ` does neither.
- A map boundary change rebuilds the grid and registers everything again.
- C++ calls `onPartitionCellChange` inside the update loop; `PartitionGrid::update` returns the ids afterwards.
- Each frame: `PartitionGrid::update`, then the returned cell changes, then `partition_collide::process_contacts`.
- The sim implements `CollisionHost`. Its `on_collide` is `Object::onCollide`: test `OBJECT_STATUS_NO_COLLISIONS` before **each** collide module, because the first object's reaction can set it on the second.
- `is_no_collide` is `KINDOF_NO_COLLIDE`. Positions and geometry are read when a contact is processed, not when it was found.
- `iteratePotentialCollisions` and `PartitionFilterWouldCollide` are not ported; `geom_collides_with_geom` is.
- Ghost objects are not modelled.

## Values the original leaves uninitialised

When two non-sphere shapes are close enough to skip the XY test, the C++ never writes `loc.x`, `loc.y` or `normal`, so `onCollide` receives stack leftovers. The port passes zeros (negative zeros for the second object's normal) and the vectors ignore those values. A scan of the `onCollide` bodies in the original's game engine found none that reads the normal, and only `RailroadBehavior` reads `loc`, for an impact sound. Recheck when porting a collide module.

## Decisions taken

The owner accepted the behaviour change and asked for a faithful port: new objects become invisible to queries until the next grid update, and some existing harnesses and vectors may have to be regenerated.

## Open questions

1. **Terrain interface.** The grid is sized from the map extent and reads cell heights. `ThingTerrainAccess` needs those.
2. **Where it lives.** Proposed: a field of `SimWorld`, because it is hashed state.

## Risks

- Geometry cell fill uses float arithmetic whose rounding decides cell membership.
- Every write to an object's position, angle or geometry must mark it dirty. The port's writes look funnelled through few places; this needs an audit.
- Shroud (stage 4) is in the CRC, so stages 1 and 2 alone do not make replays pass.
