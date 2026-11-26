# Handoff note (2026-11-27)

For the next session. Rules are in `AGENTS.md`; this note only records where work stopped. Update or delete it once it is stale.

## State

- `PKT-372 SpyVisionUpdate` and `PKT-323 DynamicShroudClearingRangeUpdate` complete and verified (`golden_shroud_updates`, 300 vectors, 3 C++ build modes byte-identical, 6 single-line mutation checks verified), with wiring from `SpyVisionSpecialPower` (`PKT-245`) and `SabotageInternetCenterCrateCollide` (`PKT-234`).
- Owner's direction: a **faithful** port, the exact behaviour of the original, and a **playable** game. Keep building while it makes token sense.
- The Rust random generator, experience tracker, money, energy, one behavior module (`LifetimeUpdate`) and the sleepy-update heap now run inside the C++ engine (`GeneralsXZH-rust`) and match a local replay baseline; see `BASELINE.md`. Nothing else ported runs in the game. Windowed play is untested.
- Packets: 239 of 463 implemented, 224 unported (Phase 2: 102, Phase 4: 4, Phase 5: 72, Phase 6: 26, Phase 7: 20). See `PORT_MAP.md`.

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
| `047c3d6`, `86b3415` | Sleepy-update heap swapped into the engine; fixed the Rust finish step, which repaired the heap where C++ does not |
| `d3aa737` and next | Engine swap build, replay check, CRC baseline; fixed the tracker FFI calling back before storing the new level |

## Not verified

- "Verified" for the module ports means golden vectors, Gate V1 and local tests. Gate 0 is still open (the public replays mismatch the local data). `check_replays.py` covers only the swapped parts (`BASELINE.md`); the Rust sim has no frame driver or game-wide CRC.
- `PORT_MAP.md` counts `PKT-216 FireWeaponPower` as implemented, but it is not bit-exact (`OPEN_DECISIONS.md`).
- No status comments were posted on GitHub Issues or Discussions. The `WP-5.3` issue (#24) is still open with its old labels.

## Next

- `MODULE_USAGE.md` (from `python3 scripts/module_census.py`) ranks the unported behavior modules by how much the replays create them. Pick from the top.
- **Owner first:** `OPEN_DECISIONS.md` item 10 (how ported Rust reaches the playable game). The one-module trial is done and its measured cost is recorded there. It decides whether more module ports are the best use of effort.
- Cheap further swaps: the byte CRC (inline in a header, so it forces a large rebuild) and other self-contained translation units. Add each to `SWAPS` in `scripts/build_rust_engine.py` with its own fault variant.
- If module porting continues: `PKT-348 PhysicsUpdate` (read, not started; it needs a `set_object_transform` on `SimApi`, Matrix3D X/Y rotations, a deferred op for re-entrant forces, and `get_mass` to include contained items), or `PKT-284 OverlordContain` (needs the remaining containment hooks, `OPEN_DECISIONS.md` item 5).
- Spatial grid Stage 4c remains deferred per `PARTITION_SCOPE.md`.

## Waiting on the owner

- `OPEN_DECISIONS.md` item 10, and the other unanswered items there.

## Cost of module ports

One worker per packet, with a script-built reading bundle: about 305k tokens for `PKT-300` and 270k for `PKT-264` (roughly 900 and 450 lines of C++ without comments), plus the coordinator's checks. The coordinator rebuilds the oracle three ways, reruns the gates and runs random mutations on the new module (read each survivor against the C++: several were harmless). Start the coordinator from a fresh session.

## How the recent work was checked

1. C++ oracle in `tests/cpp_harness/`, built three ways with `scripts/build_oracle.py` (`--opt O0`, `--opt O2`, `--opt O2 --sanitize`), run with `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`. The three outputs must be byte-identical.
2. Rust suite in `crates/zh_logic/tests/` against the JSON in `docs/golden_vectors/`.
3. Mutation check: plant bugs in a copy of the tree and confirm the suite fails each one. A suite that passes first time is not proof until this is done.
4. For engine swaps: `python3 scripts/build_rust_engine.py`, then `check_replays.py --check --expect-rust --binary ...` and the fault variants (`BASELINE.md`). State the stock CRC leaves out needs the witness check there.
5. `python3 scripts/lint_determinism.py crates/`; `cargo check --workspace --tests` with `--features zh_logic/retail-crc` and again with `zh_logic/retail-xfer-save`; `cargo test --workspace`; `python3 scripts/port_map.py --check`.

## Practical notes

- The repo is not `cargo fmt` clean. Do not reformat files you are not changing.
- `env:ephemeral-web` assignments are paused and the Actions quota is nearly used up (`AGENTS.md` §4.4). Batch pushes.
- A harness that touches shroud cells must replace the local player, or the cells call the absent display. See `gen_partition_shroud.cpp`.
