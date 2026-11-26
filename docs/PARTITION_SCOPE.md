# Spatial grid (`PKT-160 PartitionManager`) scope

Stages 1 to 3 are done: cell filing (`spatial/partition_grid.rs`), searches (`spatial/partition_query.rs`), collisions (`spatial/partition_collide.rs`) and their wiring into the sim (`SimWorld::partition`, golden `golden_partition_wiring`). Stage 4, shroud and vision, is scoped below and not started. `PKT-160` stays unported until it is.

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
| 4a | Cell shroud counts, circle reveal and cover, delayed unlook queue, map reveals, `crc`/`xfer` | ~400 | Not started |
| 4b | `Object` side: ranges, last-look records, `look`/`shroud`, refresh sites | ~400 | Not started, needs 4a |
| 4c | Per-object shroud status (`PartitionData::getShroudedStatus`) | ~150 | Deferred |

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
- Terrain is not consulted for cell heights. The shroud code is two-dimensional and does not need them.

## Stage 4: shroud and vision

`GameLogic::getCRC` hashes the grid right after the objects, so replay gates need this. `PartitionManager::crc` is, for each cell in order, 16 players of `{Int16 currentShroud, Int16 activeShroudLevel}` and then `Int16 cellX, cellY`. Threat and value maps are not in it.

### 4a: cells (self-contained in `spatial/`)

- A cell starts at `{1, 0}` for every player (passive shroud). `currentShroud` 1 is shrouded, 0 fogged, negative is the number of lookers. `addLooker`, `removeLooker`, `addShrouder` and `removeShrouder` are not symmetric; port each as written.
- The area is `DiscreteCircle` (`Core/.../Common/DiscreteCircle.cpp`, a Bresenham circle with its own duplicate-row rule), cell radius `ceil(range / cellSize)`, at least 1. Players are applied from the highest index down.
- An unlook is queued, not run: it executes in `update`, after the contacts, once `frame + UnlookPersistDuration` is less than the current frame. Map reveal and shroud calls first flush or bypass the queue differently from one another.
- `xfer` (version 2) holds the cell size, cell count, each cell's shroud levels and the pending queue.
- Oracle: build against the real `PartitionManager`, as `gen_partition_grid.cpp` does; compare the cell checksum after every operation.

### 4b: objects

- Each object has a vision range, a shroud-clearing range (the template's, falling back to vision), a shroud (jamming) range starting at 0, two last-look records, a last-shroud record and per-player vision-spied counts. None of these is in `Object::crc`; they are in `Object::xfer`.
- `look` uses `Player::getRelationship` to each player's default team, not `Object::getRelationship`, so the defector rule does not apply. Allies and spies get the reveal; `REVEAL_TO_ALL` gives everyone; the template's reveal-to-all range is a second reveal to enemies and neutrals.
- `look` skips destroyed and effectively dead objects and riders of a non-garrisonable container. An object under construction clears only its bounding circle.
- A refresh (`handlePartitionCellMaintenance`) is `unlook`, `unshroud`, `shroud`, `look`. It runs on: a change of the cell under the object's centre (from the grid update, including the first filing and leaving the map), containment and release, destroy, die, capture, defect, an `UnderConstruction` change, a clearing-range change away from (0,0,0), a vision-spied change, a map boundary change, a team changing controller, `TunnelContain` and `ActiveShroudUpgrade`.
- The port has no ordered player list with masks. It must be derived from `player_default_teams`.
- The relationship overrides noted in `OPEN_DECISIONS.md` also affect `look`.

### 4c: deferred

`getShroudedStatus` is a lazy per-player cache that is not hashed. Its callers in the original's logic are `StealthDetectorUpdate` and `LaserUpdate` (local player only) plus command and build checks (`ActionManager`, `BuildAssistant`), none of them ported. It also drives ghost objects.

### Existing stand-ins to replace in 4b

These are wrong today, not only missing:

- `ActiveShroudUpgrade` sets the shroud-clearing range. The original sets the shroud (jamming) range and refreshes.
- `SpecialPowerModule` records a reveal as well as creating the view object. The original only creates the object (with a range and a lifetime).
- `SpyVisionSpecialPower` records a reveal. The original activates `SpyVisionUpdate`, which spies on enemy objects.
- The car bomb and hijack conversions copy only the clearing range. The original also copies the vision range, which objects here do not have.
- `reveal_shroud`, `remove_shroud` and `reveal_map_for_player` only append to lists.

Modules that become portable or fuller after 4b: `PKT-372 SpyVisionUpdate`, `PKT-323 DynamicShroudClearingRangeUpdate`, and the sight parts of `PKT-309 BattlePlanUpdate`.

## Values the original leaves uninitialised

When two non-sphere shapes are close enough to skip the XY test, the C++ never writes `loc.x`, `loc.y` or `normal`, so `onCollide` receives stack leftovers. The port passes zeros (negative zeros for the second object's normal) and the vectors ignore those values. A scan of the `onCollide` bodies in the original's game engine found none that reads the normal, and only `RailroadBehavior` reads `loc`, for an impact sound. Recheck when porting a collide module.
