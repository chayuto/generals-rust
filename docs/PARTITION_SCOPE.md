# Spatial grid (`PKT-160 PartitionManager`) scope

Stage 1a (cell filing, `spatial/partition_grid.rs`, golden `golden_partition_grid`) is ported and not wired into the sim. Sizes are approximate line counts of `PartitionManager.cpp` (5,921 lines; the header adds 1,601), grouped by function name.

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

**Stage 2 cannot be exact before stage 3.** Resolving a collision re-queues both mobile objects (`processContactList` calls `makeDirty(false)`), which changes the order of later filing and so of search results. Until collisions are ported, a wired grid diverges wherever two mobile objects touch.

## For wiring into the sim (stage 2)

- Mark an object dirty only where C++ does: `reactToTransformChange` (past its thresholds) and the other `makeDirty(true)` sites in `Object.cpp`, including script disabled and unpowered changes.
- `setGeometryInfo` is unregister then register. `setGeometryInfoZ` does neither.
- A map boundary change rebuilds the grid and registers everything again.
- C++ calls `onPartitionCellChange` inside the update loop; `PartitionGrid::update` returns the ids afterwards.
- Collision checks belong between re-filings, at the marked point in `update`.
- Ghost objects are not modelled.

## Decisions taken

The owner accepted the behaviour change and asked for a faithful port.

## Open questions

1. **Stage 2 changes existing behaviour.** New objects become invisible to queries until the next grid update. Some existing harnesses and vectors may have to be regenerated.
2. **Terrain interface.** The grid is sized from the map extent and reads cell heights. `ThingTerrainAccess` needs those.
3. **Where it lives.** Proposed: a field of `SimWorld`, because it is hashed state.

## Risks

- Geometry cell fill uses float arithmetic whose rounding decides cell membership.
- Every write to an object's position, angle or geometry must mark it dirty. The port's writes look funnelled through few places; this needs an audit.
- Shroud (stage 4) is in the CRC, so stages 1 and 2 alone do not make replays pass.
