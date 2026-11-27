# Unit AI port plan

A plan only; nothing here is ported yet. Reference: `vendor/GeneralsX` at the pinned commit. "Lines" means C++ lines that are not blank or comment, counted by script, rounded. "One worker" means about 500k tokens. Past ports cost about 305k tokens for 900 lines and 270k for 450 (`HANDOFF.md`), so one worker is capped near 1,400 lines.

## What the plan rests on

- No Rust module implements `AiUpdateFacet` or `SupplyTruckAiFacet`. The AI is imitated in `sim/mod.rs` (`ai_construct`, `ai_init_state_machine`, `ai_idle`), and orders are only recorded (`ai_commands`, `exit_path_orders`).
- Some "implemented" packets are only partly there. `PKT-143 Locomotor` has templates and setters, no movement. `PKT-399 Weapon` had no `fireWeaponTemplate` when this was written; it has since been ported, so the weapons slice below is largely done. `PKT-145 TurretAI` is not connected to the sim.
- Already usable: pathfind cell, cell info, cell list and zone manager (`PKT-413..418`), partition grid, physics, the sleepy-update heap, `zh_common::core::state_machine`.
- Ground height is computed in device code that is **not in the manifest**: `W3DTerrainLogic.cpp`, `BaseHeightMap.cpp`, `WorldHeightMap.cpp`. Rust has only `FlatTerrain`, and no map loader.
- The Rust sim has no frame driver and no game-wide CRC.
- `AIUpdateInterface::crc` calls its `xfer`, so the module's own members are in the game CRC. The state machine, path, locomotor set and turrets are **not**: `xfer` reaches them through `xferSnapshot`, which in CRC mode calls their empty `crc` (found in S8). `AI::crc` adds the pathfinder, AI data and every group.
- The reference build has `RETAIL_COMPATIBLE_PATHFINDING=1`, `..._ALLOCATION=1`, `..._AIGROUP=1` and `..._XFER_SAVE=1`. `AIUpdateInterface` therefore saves version 4, while a default Rust build would write 5.
- The local replays create `AIUpdateInterface` 178 times, Dozer 45, Worker 30, SupplyTruck 26, Chinook 18, Transport 11, HackInternet 8 (`MODULE_USAGE.md`).

## How every slice is checked

The C++ is never edited. The harness links the stock `libz_gameengine.a` through `scripts/build_oracle.py` and `oracle_wp51_world.h`, which already creates `TheAI` and calls `pathfinder()->newMap()`. Because the library is linked whole, the C++ side always runs the real code around the slice. "Stubbed" therefore means two things:

- **C++ side:** only virtual overrides in the harness (synthetic `TerrainLogic`, null client objects, a recording AI subclass, as `gen_pkt296_365` does).
- **Rust side:** a callee that is not ported yet sits behind a trait. The test answers it from a **tape**: the calls and results the harness recorded from C++. The Rust test also fails if the calls differ in order or arguments. A later slice replaces the tape with real code and reruns the same vectors.

Three kinds of oracle: **Direct** (call a function; compare returns, save bytes, CRC, logic RNG seed), **Dump** (compare every cell, node or list after a scripted setup), **Trace** (give orders, step frames, compare per frame position and angle bits, AI save bytes, CRC and RNG seed).

All slices also follow `tests/cpp_harness/README.md`: identical output at `-O0`, `-O2` and sanitizers, source hashes, per-field first-seed diagnostics, and planted-bug checks on the Rust suite.

## Slices

### S0. Scenario oracle, frame driver, AI seams
- C++: `AI.cpp` `update`, the order inside `GameLogic::update`. Small. One worker (top tier).
- Oracle: Trace over a world with no AI behaviour. Must first prove two unknowns: that a real AI module runs headless in the oracle world, and whether real `GameLogic::update` can be called (otherwise the harness reproduces its order).
- Rust: one frame step (sleepy modules, AI and path queue, players, grid, destroy list); terrain, path and unit-query traits; AI code in new files, not in `sim/mod.rs`. Decide how a module calls back into an executing AI: today such calls are skipped or answer FALSE, which AI states would hit all the time.
- Risks: update order. Seen: scripted frames run in Rust and match C++ object by object.
- **Done.** Both unknowns hold: a real `AIUpdateInterface` paths and drives headless, and the real `GameLogic::update` can be called (it also needs `TheFramePacer`, `TheRecorder`, `TheCommandList`). The S1 question too: the device library links; cliff queries need a `TheTerrainRenderObject`.
  - Frame step: `gamelogic::frame::run_frame`; unported subsystems are `FrameSeams` methods. Trace: `golden_s0_frame_driver`.
  - Call-back decision: the AI is a **resident** module, never moved out of the arena (`crate::ai`). Its orders still arrive through `&mut self` facet methods; S8 must turn the orders that act into entry points.
  - The path queue (ring and cell budget) stays behind `ai::seams::PathServices` for S5. `PathUnitQueries` and `MapTerrain` are signatures only, unchecked.
  - For S8: the AI constructor asks for the next frame (the Rust `ai_construct` stand-in does not), and `doPathfind` ends in `wakeUpNow`.

### S1. Terrain queries and height sampling
- C++: `TerrainLogic.cpp` without bridges (about 1,400); height, cliff and map-data reading from the three device files (about 700). Two workers; the second, for the device code, is a small one.
- Oracle: Direct on synthetic height grids. Linking `libz_gameenginedevice.a` headless is untried; check it first. Map files: Dump of heights and cliff flags (local only, retail data).
- Seams: `ThingTerrainAccess` (ground and layer height, underwater, cliff), `CliffMap`, waypoints. Risks: float interpolation, waypoint list order.
- Seen: sloped and wet ground; waypoints can be looked up.
- **Done.** `zh_logic::terrain` (`TerrainLogic`, `HeightMap`) and `zh_common::io::map_object` load a map file and answer ground and layer height, cliff, line of sight, water and waypoint questions, with water changes, flattening, craters and the snapshot. Dump and Direct: `golden_s1_terrain` (synthetic maps of every height and blend chunk version, written by the reference; digests of the 116 maps in `MapsZH.big`, local only).
  - Seams: `ElevatedLayers` (bridges and the wall, S2), `WaterGrid` (the deformable water mesh), `TerrainEffects` (pathfind recalculation, water damage). `set_active_boundary` changes the boundary only.
  - Not done: `Bridge` and the bridge functions (S2), loading bridges and roads, the water grid settings, `loadPostProcess`, the `SidesList` chunk (parsed elsewhere). `GameLogic` does not own a `TerrainLogic` yet.

### S2. Pathfind map: classification, footprints, layers, bridges
- C++: `AIPathfind.cpp` map half (850), `PathfindLayer.cpp` (550), bridge code in `TerrainLogic.cpp` (about 600). Two workers: ground cells, then bridges and walls.
- Oracle: Dump of every cell type, layer and zone after placing structures, fences and bridges; `Pathfinder::crc`. Stubbed: terrain only.
- Seams: footprint add and remove already used by minefields and toppling; `link_bridge_tower` (open item 8). Risks: float-to-cell rounding, object order, zone numbering.
- Seen: buildings block cells; bridges open and close them.

### S3. Path objects and movement-surface queries
- C++: `Path.cpp`, `PathNode.cpp` (730), validity, line-walking and passability parts of `AIPathfind.cpp` (about 700). One worker.
- Oracle: Direct on hand-built paths and S2 maps; path save bytes. Stubbed: none beyond terrain.
- Seams: `valid_movement_terrain`. Risks: segment math (`sqrt`), cell visit order along a line.
- Seen: "can this unit stand or pass here" answers match.

### S4. Goal reservations and destination adjustment
- C++: the rest of the query half of `AIPathfind.cpp` (about 1,200): goal and position updates, `adjust*`, spiral search, `moveAllies`. One worker.
- Oracle: Dump of reservations after scripted places and moves. Stubbed: unit queries (moving, idle, radius) from tape.
- Seams: `pathfinder_adjust_destination`, `pathfinder_adjust_to_possible_destination`, the cliff and path checks missing from `find_position_around`. Risks: spiral order, ownership of shared cells.
- Seen: produced units and exits get the right landing cells.

### S5. Core search: queue, hierarchical pass, ground A*
- C++: `AIPathfind.cpp` `findPath`, hierarchical search, `internalFindPath`, neighbour and movement checks, path building, `processPathfindQueue` (about 1,500). Two workers (a ground move needs both parts).
- Oracle: Direct, comparing path nodes, cells examined and cell state afterwards. Stubbed: `doPathfind` and unit queries from tape.
- Risks: open-list tie order; 5,000 cells per frame budget and a 512-entry ring queue decide **which frame** a unit gets its path; the retail path switches to a fixed path at runtime when the old code would crash. The cell code must match that too.
- Seen: a requested path matches node for node.

### S6. Other searches
- C++: closest, attack, safe, ground-group, patch and move-away searches (about 1,500). One worker, split by search if the bundle is too large.
- Oracle and stubs as S5; weapon range and view-blocked questions from tape.
- Seen: blocked, attacking and fleeing units get matching paths.

### S7. Locomotor movement
- C++: `Locomotor.cpp` movement (1,240, of which air and hover 450). One worker; split ground and air if needed.
- Oracle: Direct per appearance with the real physics module; compare transform, forces, locomotor save bytes. Stubbed: `validMovementTerrain` from tape.
- Seams: `set_locomotor_*`, the unused `LocomotorSet`. Closes `PKT-143` properly.
- Risks: the most trigonometry in the plan (use `gm_*` only), `min`/`max` macro NaN order, three constructor draws already ported.
- Seen: a unit given one goal point drives, turns and brakes.
- **Done.** The movement functions are in `combat/locomotor.rs` and drive the real `PhysicsBehavior` (`golden_s7_locomotor_movement`: 1,200 vectors, every appearance, drives to one goal included). `validMovementTerrain` and `getGroundOrStructureHeight` sit behind `LocomotorSeams`, answered from a tape; `scripts/build_oracle.py` now records such calls (`// oracle-wrap:`).
  - Left: no caller. S8 owns the `LocomotorSet` and wires `set_locomotor_*`; S3 or S9 implements `LocomotorSeams` on the real pathfinder and partition grid. Gaps: `OPEN_DECISIONS.md`.

### S8. AI module shell, orders, state machine, idle
- C++: `AIUpdate.cpp` data, orders and `xfer` (1,540); `AIStateMachine.h`, idle, dead and temporary states (about 700). Two workers.
- Oracle: Direct on the real module: XferSave and CRC after each order, refusals included. Stubbed: path, turret and weapon calls from tape.
- Seams: `is_idle`, `is_ai_in_dead_state`, `mark_as_dead`, `ai_busy`, `ai_idle`, `set_attitude`, `has_cur_locomotor`, `ignore_obstacle`, `get_ignored_obstacle_id`, `ai_construct`, `ai_init_state_machine`, `ai_last_command_source`.
- Risks: draws move from the stand-ins into here (four at start, one per idle entry). Their order must not change, so rerun every older suite. `ai_goal_object` must hold an ID, not an arena slot.
- Seen: every unit has a real AI; its CRC contribution matches while idle.
- **Done for the base class.** `AIUpdateInterface` is a registered resident module (`modules/update/ai_update.rs`), the state machine is `ai/ai_state_machine.rs`, and idle, dead and busy are in `ai/ai_states.rs`. `golden_s8_ai_orders` compares save bytes (version 4), CRC and seed after every order and update. All other states, the pathfinder, turrets, mood targets and the action manager are `ai::seams::AiSeams`, answered from a tape.
  - Orders reach the module through `AiCall` (`BehaviorModule::ai_call_entry`). `ai_construct` and `ai_init_state_machine` remain for the module stand-ins of older suites.
  - A game CRC has to call `xfer` in CRC mode for this module: it needs the next call frame, which `BehaviorModule::crc` cannot reach.
  - Derived classes: none yet. They need hooks on `update`, `execute_command` and `xfer`, and their own states.
  - Left: request paths, `computeQuickPath`, `computePath`, the on-path drive and collisions (S9); `Object::reactToTransformChange`'s `updatePos` for AI objects (S4); squads, polygon areas, waypoint objects, `privateMoveAwayFromUnit`, command buttons; loading a save.

### S9. Path following inside the AI update
- C++: `AIUpdate.cpp` path request, `doPathfind`, `computePath`, `doLocomotor`, blocked and collision handling (1,630). One worker, tight; split collisions off if needed.
- Oracle: Trace with units placed in each other's way.
- Seams: `is_moving`, `destroy_path`, `get_path_destination`, `process_collision` (physics answers FALSE today). Risks: blocked counters, collision order, distance math.
- Seen: units follow a path and push or wait for each other.

### S10. Movement states
- C++: `AIStates.cpp` internal move, move to, follow path, tighten, move out of the way, attack-move shell (about 1,200). One worker.
- Oracle: Trace per order.
- Seams: `ai_move_to_position`, `ai_move_to_object`, `ai_follow_path`, `ai_follow_exit_production_path` (dispatch `exit_path_orders`).
- Seen: **first real play test**: order a move, the unit gets there; produced units leave the factory.

### S11. Other non-combat states
- C++: waypoint follow, wander, panic, repulsors, dock, enter, exit, evacuate, face, wait, rappel, crate pickup (about 1,600). Two workers.
- Oracle: Trace. Stubbed: action permissions from tape.
- Seams: `ai_enter`, `ai_exit` (including `instantly`), `ai_face_object`, `ai_face_position`, `is_free_to_exit`, `get_enter_target`, `ai_get_healed`, passenger exit and idle orders.
- Risks: most state-machine PRNG draws are here (waypoint, wander, panic).
- Seen: garrison, board, leave, patrol.

### S12. Targeting and mood
- C++: `AI.cpp` closest enemy, ally, repulsor and vision range (about 250), mood and target choice in `AIUpdate.cpp` (560). One worker.
- Oracle: Direct over random crowds. Stubbed: `getAbleToAttackSpecificObject` from `attack_abilities`.
- Seams: `wake_up_and_attempt_to_target`, `attack_priority`, `evaluate_morale_bonus`. Unblocks `PKT-325` and `PKT-311`.
- Risks: partition iteration order breaks ties; one draw offsets the mood check time.
- Seen: idle units notice enemies.
- **Done, without a caller.** `zh_logic::ai::ai` (searches, vision range) and `zh_logic::ai::mood` (mood matrix, mood timer, `next_mood_target`). Direct over random crowds: `golden_s12_targeting`.
  - S8 keeps a `MoodState` (it saves both members), fills a `MoodUnit` and calls in; `attack_priority` is `AttackPriorityInfo::priority_of`; `evaluate_morale_bonus` already sits in `horde_update`.
  - `PartitionManager` searches with filters are `SimView::closest_object_from` and `objects_from_near_to_far`. Gaps: `OPEN_DECISIONS.md`.

### S13. Weapon firing
- C++: `Weapon.cpp` `fireWeaponTemplate`, ranges, approach target, line of fire, damage estimate (about 800). One worker. Not AI code, but nothing fights without it.
- Oracle: Direct. Stubbed: line of sight from tape until S3 is wired.
- Seams: `fire_weapon_template`, `projectile_detonations`, the weapon-set write-back gap, `onWeaponBonusChange`, barrel counts (open item 9). Fix `PKT-216` here.
- Risks: scatter and infantry inaccuracy draws, range float compares.
- Seen: scripted shots, projectiles and mines deal damage.

### S14. Attack states and turrets
- C++: `AIStates.cpp` attack, approach, pursue, aim, fire, squad, hunt, attack area (1,670); wiring of the ported `TurretAI` (about 1,000 C++ lines, already written). Two workers: states, then turrets and the weapon half of the AI update.
- Oracle: Trace of duels. Stubbed: nothing new.
- Seams: `ai_attack_object`, `ai_force_attack_object`, `ai_attack_position`, `ai_hunt`, `is_attacking`, `get_current_victim`, `get_current_victim_pos`, `is_weapon_slot_enabled`, `ai_go_prone`, `order_slaves_*`, `FiringTracker`.
- Risks: aim angle math, hunt, squad, area and turret draws.
- Seen: **units fight and die**.

### S15. Supply, dock, transport and hacker AI
- C++: SupplyTruck (580), `AIDock.cpp` (460), Transport (130), HackInternet (370). About 1,500: one worker, or move HackInternet and Transport out.
- Oracle: Trace with the ported dock modules.
- Seams: all of `SupplyTruckAiFacet` and `supply_truck()`, `ai_hack_internet`. Needs `setCashValue` routes. Risks: dock queue order, boneless dock positions (known gap).
- Seen: supplies are gathered and money rises.

### S16. Dozer and worker AI
- C++: Dozer (1,390), Worker (860). Two workers, Dozer first.
- Seams: `is_clearing_mines`, construction percent, build and repair orders. Stubbed: build legality from tape until S19.
- Risks: task order; the worker switches between two state machines.
- Seen: **a base can be built**.

### S17. Aircraft and missiles
- C++: Chinook (1,250), Jet (2,170), `ParkingPlaceBehavior` (840), Missile AI (670). Four workers: Chinook, Jet (two with parking), missile.
- Oracle: Trace. Seams: the parking facet. Risks: 3D float math, `std::vector` and `std::list` order, Chinook and missile draws, the preserved Chinook passenger-dumping behaviour.
- Seen: airfields, drops, guided missiles.

### S18. Guard states and remaining AI classes
- C++: the three guard files (about 1,800); DeployStyle, AssaultTransport, DeliverPayload, Wander, POW truck, RailedTransport, Railroad (about 3,200). Three to five workers; one class each is independent.
- Oracle: Trace. Risks: guard, delivery, wander and rail draws; scan timing.
- Seen: guard orders, deploy, supply drops, trains.

### S19. Action manager and groups
- C++: `ActionManager.cpp` (1,390), `AIGroup.cpp` (2,510), `Squad.cpp` (140). Three workers.
- Oracle: Direct for permissions; Trace for group orders. Replaces the `action_permissions` stand-in.
- Risks: `std::list` membership order is hashed by `AI::crc`; group centre math, two draws, retail AIGroup quirks.
- Seen: one order to many units, with formations.

### S20. Player command dispatch
- C++: `GameLogicDispatch.cpp` (1,855, partly client code). One or two workers. Also needs `CommandButton` and `CommandSet` INI (`PKT-091`, `PKT-092`).
- Oracle: feed message lists, Trace the result.
- Seen: a recorded command stream can drive the Rust sim.

### S21. Computer opponent
- C++: `AIPlayer.cpp` (2,950), `AISkirmishPlayer.cpp` (890). Three workers. Depends on teams and the script engine (`PKT-410`, `PKT-433`), which are not in this plan.
- Seen: skirmish against the computer.

## Order and parallel work

```
S0 ─┬─ S1 ── S2 ── S3 ── S4 ── S5 ── S6 ──┐
    ├─ S7 ─────────────────────────────────┤
    ├─ S8 ─────────────────────────────────┴─ S9 ── S10 ── S11
    ├─ S12 ─┐
    └─ S13 ─┴─ S14 (also needs S10) ── S18 ── S19 ── S20
S11 ── S15, S16      S14 ── S17      S19 + scripts ── S21
```

- Safe in parallel after S0: **S1, S7, S8, S12, S13**. S2 may start with S1 if S0 fixes the terrain trait.
- Later: S15 with S16; the classes inside S17 and S18.
- Not parallel: S2 to S6 share the pathfinder; S9 to S11 and S14 share the AI module.
- All slices tend to touch `sim/mod.rs` and `module.rs`. Parallel workers need separate worktrees and new files, with one coordinator doing the wiring.
- This list shows what dependencies allow. The current project habit is one worker at a time unless quota headroom is good.

## Cost and coverage

- First moving unit (S0 to S10): about 15 workers. Whole list: about 40 workers, roughly 12 to 20 million tokens, before rework.
- None of this proves replay parity. That also needs a map loader, `Object.cpp` and `Team.cpp` gaps, scripts, game start-up and the game-wide CRC, and Gate 0 is still open (`BASELINE.md`).
- The engine-swap check is not planned for these slices: the adapter cannot carry state this tightly shared (open item 10).
- Rust play-like tests are scripted scenarios compared with C++ traces. They are not games a person can play.

## Decisions for the owner

1. **Is this worth doing?** Open item 10 is unanswered: the AI port serves a headless simulator, not the playable game. S0 to S10 is a reasonable first stopping point.
2. **XferSave version** (open item 2). The AI makes it urgent: default Rust writes version 5, the reference writes and hashes 4. Recommended: match the reference build, or compare traces with `retail-xfer-save` semantics.
3. **Retail pathfinding and group quirks.** Recommended: port what the reference build runs (retail path with its fallback, retail groups) as the default. A fixed-only port would be simpler but could not match the reference.
4. **Computer opponents.** Keep S21 (and the script engine it needs) or drop it.
5. **Map files.** Should S1 include loading real maps, or stay on synthetic terrain until replay work is planned?
