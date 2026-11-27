# Handoff note (2026-11-27)

For the next session. Rules are in `AGENTS.md`; this note only records where work stopped. Update or delete it once it is stale.

## State

- `PKT-372 SpyVisionUpdate` and `PKT-323 DynamicShroudClearingRangeUpdate` complete and verified (`golden_shroud_updates`, 300 vectors, 3 C++ build modes byte-identical, 6 single-line mutation checks verified), with wiring from `SpyVisionSpecialPower` (`PKT-245`) and `SabotageInternetCenterCrateCollide` (`PKT-234`).
- Owner's direction: a **faithful** port, the exact behaviour of the original, and a **playable** game. Keep building while it makes token sense.
- The Rust random generator, experience tracker, money, energy, the sleepy-update heap and seven behavior modules (`LifetimeUpdate`, `SupplyWarehouseCreate`, `ArmorUpgrade`, `ModelConditionUpgrade`, `RadarUpdate`, `PowerPlantUpdate`, `DefaultProductionExitUpdate`) run inside the C++ engine (`GeneralsXZH-rust`) and match a local replay baseline. The replays verify only part of that: see `BASELINE.md` before treating a swapped module as checked. Windowed play is untested.
- Packets: 283 of 463 marked implemented, 40 data only, 140 unported (`PORT_MAP.md`). "Implemented" includes partial ones (`PKT-431` to `PKT-433`, `PKT-440`).
- A Rust game starts from a retail map and runs 300 frames (`gamelogic/game_logic.rs`, `start_new_game`). It is not playable: no unit moves and there is no client.

## Recently done

| Commits | Work |
|---|---|
| `e898491` to `6f759e0` | `WP-5.4` batches A to C, the shared weapon firing state machine, `FireWeaponUpdate` |
| `c64ccb4` to `920ee37` | Spatial grid stages 1 to 3: filing, searches, collisions, wired into the sim |
| `213bdc7`, `fb975a9` | `PKT-388 WeaponBonusUpdate`; `FiringTracker` attached; undetected defectors honoured |
| `a31f330`, `2700f1d` | Spatial grid stage 4a–4b (`PKT-160 PartitionManager`): shroud cells, object-side vision/shroud/jamming, `SightingInfo`, refresh sites, `TerrainLogic::setActiveBoundary`, faithful `Object::onDestroy` + `TeamMemberList` ordering |
| `de99fdb` | `PKT-372 SpyVisionUpdate` (plus `SpyVisionSpecialPower` / `SabotageInternetCenterCrateCollide` wiring and `SimApi` vision-spied helpers) & `PKT-323 DynamicShroudClearingRangeUpdate` (`golden_shroud_updates`) |
| `b428ad1` | Contain getters take the game view (owner decision on `OPEN_DECISIONS.md` items 4 and 5) |
| `fafa769` and next | `PKT-300 TunnelContain`, `PKT-264 CaveContain` |
| after `2b88a3b` | Module census (`MODULE_USAGE.md`); `PKT-330 FlammableUpdate` and `PKT-327 FireSpreadUpdate`; fixed `HighlanderBody` on NaN damage |
| after `2daf832` | `PKT-358 RadarUpdate` and `PKT-351 PowerPlantUpdate`; `RadarUpgrade` now extends the radar; CI time limit raised to 6 minutes |
| `a8a882a`, `5fdee17` | `PKT-320/381/378/363` dock updates (dock state in the module); `PKT-348 PhysicsBehavior`, replacing the physics stand-ins; `find_position_around` uses the reference collision shapes |
| after `c5f37a3` | `PKT-374 StealthUpdate` and `PKT-373 StealthDetectorUpdate`; dock stealth grants reach the real module |
| after `198371f` | `PKT-315/379/357/367` production exit updates; a production exit now wins over a container listed before it |
| after `83d6689` | `PKT-383 ToppleUpdate`, `PKT-377 StructureToppleUpdate`, `PKT-305 AutoDepositUpdate`; per-player score keepers |
| after `9d795fc` | `PKT-336 HordeUpdate`, `PKT-370 SpectreGunshipDeploymentUpdate`, `PKT-306 AutoFindHealingUpdate`, `PKT-304 AssistedTargetingUpdate` |
| after `9d53470` | `PKT-272 GenerateMinefieldBehavior`, `PKT-279 MinefieldBehavior`; cliff cells, overlap query, module self-lending |
| after `d7fc62e` | `PKT-368 SpecialAbilityUpdate`, `PKT-314 CommandButtonHuntUpdate`; one `SpecialPowerUpdateFacet` for both porting branches |
| after `52478cd` | `PKT-296 SpawnBehavior`, `PKT-365 SlavedUpdate` |
| after `c0b420d` | `PKT-334 HelicopterSlowDeathBehavior`, `PKT-267 DumbProjectileBehavior`, `PKT-382 TensileFormationUpdate`, `PKT-346 OCLUpdate`; weapon speeds now converted to per frame |
| after `5da9d25` | `PKT-354 ProductionUpdate` |
| after `4fc8921` | AI plan slice S0: logic frame step, resident AI seams, frame oracle; two disabled-state differences fixed |
| after `8eabb2f` | `PKT-286 ParkingPlaceBehavior`, `PKT-341`, `PKT-350`, `PKT-309 BattlePlanUpdate`; `Anti* = No` now clears the weapon bit |
| after `889e46a` | Joined oracle over ported modules; fixed energy counting, split score and academy tables, money deposits, capture records and the stealth velocity cache |
| after `f1501fc` | Faithfulness fixes: crushing rules, special power pause counts, weapon template parsing, reload retiming on bonus changes, academy power use |
| after `fc9d180` | Second joined oracle with save loading (34,259 rows); fixed the weapon set model condition lost at drawable binding |
| `047c3d6`, `86b3415` | Sleepy-update heap swapped into the engine; fixed the Rust finish step, which repaired the heap where C++ does not |
| `d3aa737` and next | Engine swap build, replay check, CRC baseline; fixed the tracker FFI calling back before storing the new level |
| after `575ade7` | `SWAP_CANDIDATES.md` count; six more modules swapped into the engine; value accessors for `object()` reads; money, upgrades and energy keyed by player (`golden_player_state`) |
| up to `2e28a9b` | Script engine, unit AI shell (S8), targeting (S12), pathfind map (S2), path queries (S3), players and teams, AI groups and squads, retail rule data loading |
| `1801249`, `f2c51a8` | Weapon, locomotor, AI, creation list and road data in full; data parsers for the unported logic modules in the retail rules |
| `4fe15f5` to `10fc0fc` | Map script lists, game start from a map, comparison with the reference (`golden_game_start`) |

## Not verified

- Game start against the reference (24 games: 6 baseline replays and 18 option variants; `GameStart.json`):
  - before the starting units, seed, players, teams and object fields match on 20 games; the 4 with bridges are compared only on players and non-bridge objects (bridges are skipped);
  - before the AI player's map setup, the seed also differs (no Dozer or Worker AI) and 38 mobile starting units are placed differently (no `adjustDestination`);
  - at the end, `AIPlayer::newMap` is unported too;
  - scripts affect only delay draws (no `ScriptWorld` adapter), wall height is 0, frames are not CRC-checked;
  - its mutation check is weak (17 of 60 caught before targeted rechecks): script status, shroud and observer paths are not exercised;
  - the retail tests skip without game data, so CI does not run them.
- "Verified" for the module ports means golden vectors, Gate V1 and local tests. Gate 0 is still open (the public replays mismatch the local data). `check_replays.py` covers only the swapped parts (`BASELINE.md`); the Rust sim has no game-wide CRC.
- `PORT_MAP.md` counts `PKT-216 FireWeaponPower` as implemented, but it is not bit-exact (`OPEN_DECISIONS.md`).
- No status comments were posted on GitHub Issues or Discussions. The `WP-5.3` issue (#24) is still open with its old labels.

## Next

- **Goal (owner, 2026-11-27): the whole game in Rust** (`OPEN_DECISIONS.md` item 10, `FULL_PORT_PLAN.md`). Simulation first.
- Toward a moving unit (`AI_PORT_PLAN.md`): S4 goal reservations, then S5 and S6 searches, S9 path following, S10 movement states. These build on each other. S13 weapon firing is separate.
- Wiring still missing: targeting has no caller; the script engine has no `ScriptWorld` adapter or `FrameSeams` hookup; players, teams and AI groups are only partly connected.
- From the game start comparison: bridge objects, Dozer and Worker AI, `adjustDestination`, `AIPlayer::newMap`.
- Client crates: ask the owner before adding the first one (item 12).
- Smaller work that needs no decision: the entries under "Known gaps" in `OPEN_DECISIONS.md`; vectors for the paths the two joined oracles leave uncovered; the behavior modules the replays never create (`PORT_MAP.md`, phase 5).
- Packet counts overstate completeness (see the known gaps). Weapons now do damage (`golden_pkt399_weapon_effects`); gaps are in `OPEN_DECISIONS.md`.
- Next faithfulness gap of the same kind as the per-player fix: sciences, radar, cost modifiers and handicaps are still keyed by team.
- Cheap further engine swaps: the byte CRC and other self-contained translation units. Add each to `SWAPS` in `scripts/build_rust_engine.py` with its own fault variant.
- Spatial grid Stage 4c remains deferred per `PARTITION_SCOPE.md`.

## Waiting on the owner

- The unanswered items in `OPEN_DECISIONS.md`. Item 12 (client crates) is to be asked when it is needed.

## Cost of module ports

One worker per packet, with a script-built reading bundle: about 305k tokens for `PKT-300` and 270k for `PKT-264` (roughly 900 and 450 lines of C++ without comments), plus the coordinator's checks. The coordinator rebuilds the oracle three ways, reruns the gates and runs random mutations on the new module (read each survivor against the C++: several were harmless). Start the coordinator from a fresh session.

## How the recent work was checked

1. C++ oracle in `tests/cpp_harness/`, built three ways with `scripts/build_oracle.py` (`--opt O0`, `--opt O2`, `--opt O2 --sanitize`), run with `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`. The three outputs must be byte-identical.
2. Rust suite in `crates/zh_logic/tests/` against the JSON in `docs/golden_vectors/`.
3. Modules that pass alone can disagree together: `golden_joins_modules` and `golden_joins2_modules` run ported modules in one world with no Rust stand-ins. Add new modules there.
4. CI runs on x86: run `cargo test --workspace --target x86_64-apple-darwin` before pushing and check the CI result after.
5. Mutation check: plant bugs in a copy of the tree and confirm the suite fails each one. A suite that passes first time is not proof until this is done.
6. For engine swaps: `python3 scripts/build_rust_engine.py`, then `check_replays.py --check --expect-rust --binary ...` and the fault variants (`BASELINE.md`). State the stock CRC leaves out needs the witness check there.
7. `python3 scripts/lint_determinism.py crates/`; `cargo check --workspace --tests` with `--features zh_logic/retail-crc` and again with `zh_logic/retail-xfer-save`; `cargo test --workspace`; `python3 scripts/port_map.py --check`.

## Practical notes

- The repo is not `cargo fmt` clean. Do not reformat files you are not changing.
- Concurrent porting branches added the same methods independently; after merging them, search for duplicated methods and tables before testing.
- `env:ephemeral-web` assignments are paused and the Actions quota is nearly used up (`AGENTS.md` §4.4). Batch pushes.
- A harness that touches shroud cells must replace the local player, or the cells call the absent display. See `gen_partition_shroud.cpp`.
