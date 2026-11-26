# Spatial grid (`PKT-160 PartitionManager`) scope

Stages 1 to 4b are done: cell filing (`spatial/partition_grid.rs`), searches (`spatial/partition_query.rs`), collisions (`spatial/partition_collide.rs`), cell shroud (`spatial/partition_shroud.rs`), and object-side vision, shroud clearing, active shroud jamming, `SightingInfo` persistence, and wiring (`sim/mod.rs`, `spatial/partition_manager.rs`, golden `golden_partition_wiring`, `golden_partition_shroud`, `golden_partition_object_shroud`). `PKT-160` is marked implemented; Stage 4c (`getShroudedStatus` lazy cache + ghost objects) remains deferred.

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
| 4a | Cell shroud counts, circle reveal and cover, delayed unlook queue, map reveals, `crc`/`xfer` | ~400 | Done, wired: `golden_partition_shroud`, `golden_partition_object_shroud` |
| 4b | `Object` side: ranges, last-look records, `look`/`shroud`, refresh sites | ~400 | Done: `golden_partition_object_shroud` |
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
- No Rust counterpart yet: the register and unregister calls in `TunnelContain`, `CaveContain`, `TunnelTracker::loadPostProcess`, `HijackerUpdate` and `DeliverPayloadAIUpdate`, `TunnelContain::onContaining` (`PKT-284` is unported; see `OPEN_DECISIONS.md` items 4–5), and the re-filing on script disabled or unpowered changes. Add them when porting those.
- `Object::setStatus` also detonates mines on an `UnderConstruction` change. Not ported.
- Threat and value maps (`onPartitionCellChange`) are not ported (the shroud half of `onPartitionCellChange` is wired in 4b).
- `iteratePotentialCollisions` and `PartitionFilterWouldCollide` are not ported; `geom_collides_with_geom` is.
- Ghost objects are not modelled.
- Terrain is not consulted for cell heights. The shroud code is two-dimensional and does not need them.

## Stage 4: shroud and vision

`GameLogic::getCRC` hashes the grid right after the objects, so replay gates need this. `PartitionManager::crc` is, for each cell in order, 16 players of `{Int16 currentShroud, Int16 activeShroudLevel}` and then `Int16 cellX, cellY`. Threat and value maps are not in it.

### 4a: cells (done, in `spatial/partition_shroud.rs`)

- A cell starts at `{1, 0}` for every player (passive shroud). `currentShroud` 1 is shrouded, 0 fogged, negative is the number of lookers. `addLooker`, `removeLooker`, `addShrouder` and `removeShrouder` are not symmetric. Removing a looker that was never added leaves a value above 1, which reads as clear.
- The area is a `DiscreteCircle`, cell radius `ceil(range / cellSize)`, at least 1.
- An unlook is queued, not run: it executes once `frame + UnlookPersistDuration` is less than the current frame. Undoing a permanent reveal and shrouding the map flush the queue first; the two reveals do not.
- A map boundary change stores the cells around the rebuild. On a map with fewer rows only player 0 is restored, because `restoreFoggedCells` returns early. Kept.
- The saved cell size and count must match the grid. A load appends to the queue.
- Not verified against the original: standalone `ShroudStatusStoreRestore` load edge cases (live `PartitionGrid::xfer` save/load round-trips and 8-step `init_partition` boundary store/restore are verified by `golden_partition_shroud` and `golden_partition_object_shroud`).
- Wired in 4b: `update_partition` refreshes changed object IDs before `process_contacts` and drains `process_pending_undo_shroud_reveals(frame, player_count)` after; `init_partition` runs the 8-step `TerrainLogic::setActiveBoundary` flush/store/restore sequence; `game_logic` CRC assembly outside individual objects/partition grid awaits the full frame driver.

### 4b: objects (done, in `sim/mod.rs` and `modules/`)

- Each object has `vision_range`, `shroud_clearing_range` (`-1.0` falls back to `vision_range`), `shroud_range` (starting at 0), `partition_last_look`, `partition_reveal_all_last_look`, `partition_last_shroud`, and `vision_spied_by` / `vision_spied_mask`. None of these is in `Object::crc`; all are serialized in `ObjectRecord::xfer_vision_and_shroud` (`Object.cpp:4228-4257`).
- `look` and `shroud` resolve each player slot's default team's controlling player (`Player::getRelationship` to `currentPlayer->getDefaultTeam()`, not `Object::getRelationship`, so defector rules do not apply). Allies and spies get the clear reveal; `REVEAL_TO_ALL` reveals to `PLAYERMASK_ALL`; `ShroudRevealToAllRange` adds a second reveal to enemies and neutrals unless `UnderConstruction` or un-detected stealthed.
- `look` skips `Destroyed`, `IsEffectivelyDead`, and riders of a non-garrisonable container (`ModuleSidecar::Contain.garrisonable`). `unlook` keeps the C++ early return when `partition_last_look.is_invalid()`. `shroud` skips `UnderConstruction`, `IsEffectivelyDead`, or `shroud_range <= 0`.
- A refresh (`handle_partition_cell_maintenance`) runs `object_unlook`, `object_unshroud`, `object_shroud`, `object_look` on: partition cell change in `update_partition`, contain add/remove, `destroy_object_now` (`Object::onDestroy`, after `on_delete`), `dispatch_on_die`, `capture_object`, `defect_object` (two refreshes via `set_object_team` + trailing refresh), `change_status` for `Stealthed`/`Detected`/`Disguised` when `shroud_reveal_to_all_range > 0` (`UnderConstruction` only calls `make_dirty` in `Object::setStatus`), `set_shroud_clearing_range` away from `(0, 0, 0)`, `set_vision_spied`, `set_team_controlling_player` (iterating `SimWorld::team_members` head-prepend order), `init_partition` (`TerrainLogic::setActiveBoundary`), and `ActiveShroudUpgrade`.
- `SimWorld` has no ordered player list; player slots and `PLAYERMASK_ALL` (`0xFFFF`) are derived from `player_default_teams`.
- The relationship overrides noted in `OPEN_DECISIONS.md` also affect `look`.
- Stage 4b stand-ins were replaced: `ActiveShroudUpgrade` sets `shroud_range` and refreshes; `SpecialPowerModule::create_view_object` spawns on `player_default_team` with `shroud_clearing_range` and lifetime; `SpyVisionSpecialPower` drops its stand-in reveal (awaits `PKT-372 SpyVisionUpdate`); car-bomb and hijack conversions copy both `vision_range` and `shroud_clearing_range`; `SimWorld::{shroud_reveals, shroud_removals, map_reveals}` and `SimApi::{reveal_shroud, remove_shroud}` were removed; `reveal_map_for_player` delegates directly to `PartitionGrid`.

### 4c: deferred

`getShroudedStatus` is a lazy per-player cache that is not hashed. Its callers in the original's logic are `StealthDetectorUpdate` and `LaserUpdate` (local player only) plus command and build checks (`ActionManager`, `BuildAssistant`), none of them ported. It also drives ghost objects.

Modules that become portable or fuller after 4b: `PKT-372 SpyVisionUpdate`, `PKT-323 DynamicShroudClearingRangeUpdate`, and the sight parts of `PKT-309 BattlePlanUpdate`.

## Values the original leaves uninitialised

When two non-sphere shapes are close enough to skip the XY test, the C++ never writes `loc.x`, `loc.y` or `normal`, so `onCollide` receives stack leftovers. The port passes zeros (negative zeros for the second object's normal) and the vectors ignore those values. A scan of the `onCollide` bodies in the original's game engine found none that reads the normal, and only `RailroadBehavior` reads `loc`, for an impact sound. Recheck when porting a collide module.
