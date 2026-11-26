# Handoff note (2026-11-26)

For the next session. Rules are in `AGENTS.md`; this note only records where work stopped. Update or delete it once it is stale.

## State

- `PKT-372 SpyVisionUpdate` and `PKT-323 DynamicShroudClearingRangeUpdate` complete and verified (`golden_shroud_updates`, 300 vectors, 3 C++ build modes byte-identical, 6 single-line mutation checks verified), with wiring from `SpyVisionSpecialPower` (`PKT-245`) and `SabotageInternetCenterCrateCollide` (`PKT-234`).
- Owner's direction: a **faithful** port, the exact behaviour of the original.
- Packets: 219 of 463 implemented, 244 unported (Phase 2: 102, Phase 4: 4, Phase 5: 92, Phase 6: 26, Phase 7: 20). See `PORT_MAP.md`.

## Recently done

| Commits | Work |
|---|---|
| `e898491` to `6f759e0` | `WP-5.4` batches A to C, the shared weapon firing state machine, `FireWeaponUpdate` |
| `c64ccb4` to `920ee37` | Spatial grid stages 1 to 3: filing, searches, collisions, wired into the sim |
| `213bdc7`, `fb975a9` | `PKT-388 WeaponBonusUpdate`; `FiringTracker` attached; undetected defectors honoured |
| `a31f330`, `2700f1d` | Spatial grid stage 4a–4b (`PKT-160 PartitionManager`): shroud cells, object-side vision/shroud/jamming, `SightingInfo`, refresh sites, `TerrainLogic::setActiveBoundary`, faithful `Object::onDestroy` + `TeamMemberList` ordering |
| latest | `PKT-372 SpyVisionUpdate` (plus `SpyVisionSpecialPower` / `SabotageInternetCenterCrateCollide` wiring and `SimApi` vision-spied helpers) & `PKT-323 DynamicShroudClearingRangeUpdate` (`golden_shroud_updates`) |

## Not verified

- "Verified" for the work above means golden vectors, Gate V1 and local tests. **Replay gates V3 and V4 were not run.** There is no frame driver and no game-wide CRC assembly yet.
- `PORT_MAP.md` counts `PKT-216 FireWeaponPower` as implemented, but it is not bit-exact (`OPEN_DECISIONS.md`).
- No status comments were posted on GitHub Issues or Discussions. The `WP-5.3` issue (#24) is still open with its old labels.

## Next

- Next unblocked module with Stage 4b complete: sight parts of `PKT-309 BattlePlanUpdate` (alongside `Player::changeBattlePlan`, `StealthDetectorUpdate`, and turret control), or `PKT-348 PhysicsUpdate` / `PKT-156 TerrainLogic` spines.
- Spatial grid Stage 4c (`PartitionData::getShroudedStatus` lazy cache + ghost objects) remains deferred per `PARTITION_SCOPE.md`.

## Waiting on the owner

- `OPEN_DECISIONS.md` items 4 and 5: should the contain facet be able to read game state? The recommendation given was yes. Not answered. It blocks `PKT-264`, `PKT-284` and `PKT-300`.
- The other open decisions in that file are also unanswered.

## How the recent work was checked

1. C++ oracle in `tests/cpp_harness/`, built three ways with `scripts/build_oracle.py` (`--opt O0`, `--opt O2`, `--opt O2 --sanitize`), run with `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy`. The three outputs must be byte-identical.
2. Rust suite in `crates/zh_logic/tests/` against the JSON in `docs/golden_vectors/`.
3. Mutation check: plant bugs in a copy of the tree and confirm the suite fails each one. A suite that passes first time is not proof until this is done.
4. `python3 scripts/lint_determinism.py crates/`; `cargo check --workspace --tests` with `--features zh_logic/retail-crc` and again with `zh_logic/retail-xfer-save`; `cargo test --workspace`; `python3 scripts/port_map.py --check`.

## Practical notes

- The repo is not `cargo fmt` clean. Do not reformat files you are not changing.
- `env:ephemeral-web` assignments are paused and the Actions quota is nearly used up (`AGENTS.md` §4.4). Batch pushes.
- A harness that touches shroud cells must replace the local player, or the cells call the absent display. See `gen_partition_shroud.cpp`.
