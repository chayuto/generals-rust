# Handoff note (2026-11-26)

For the next session. Rules are in `AGENTS.md`; this note only records where work stopped. Update or delete it once it is stale.

## State

- `main` at `a31f330` plus this note. CI passed on `a31f330` (run `45781221090`).
- No work in progress, no uncommitted files, nothing running.
- Owner's direction: a **faithful** port, the exact behaviour of the original.
- Packets: 216 of 463 implemented, 247 unported (Phase 2: 102, Phase 4: 5, Phase 5: 94, Phase 6: 26, Phase 7: 20). See `PORT_MAP.md`.

## Recently done

| Commits | Work |
|---|---|
| `e898491` to `6f759e0` | `WP-5.4` batches A to C, the shared weapon firing state machine, `FireWeaponUpdate` |
| `c64ccb4` to `920ee37` | Spatial grid stages 1 to 3: filing, searches, collisions, wired into the sim |
| `213bdc7`, `fb975a9` | `PKT-388 WeaponBonusUpdate`; `FiringTracker` attached; undetected defectors honoured |
| `a31f330` | Spatial grid stage 4a: shroud cells |

## Not verified

- "Verified" for the work above means golden vectors, Gate V1 and local tests. **Replay gates V3 and V4 were not run.** There is no frame driver and no game-wide CRC assembly yet.
- `PORT_MAP.md` counts `PKT-216 FireWeaponPower` as implemented, but it is not bit-exact (`OPEN_DECISIONS.md`).
- No status comments were posted on GitHub Issues or Discussions. The `WP-5.3` issue (#24) is still open with its old labels.

## Next: spatial grid stage 4b (objects looking and jamming)

Design is in `PARTITION_SCOPE.md`, "Stage 4". `PKT-160` stays unported until 4b is done. 4c is deferred.

**Original, in `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Object.cpp`:**

- `onPartitionCellChange` 4975, `handlePartitionCellMaintenance` 4981, `handleShroud` 4989.
- `look` 5119, `unlook` 5217, `shroud` 5255, `unshroud` 5293.
- `setVisionRange` 5332, `setShroudClearingRange` 5369, `setShroudRange` 5426, `setVisionSpied` 5432.
- Other refresh calls: lines 702, 715, 827, 1064, 2916, 4703, 4768, 6397; `Team.cpp` 1423; `TunnelContain.cpp` 229.

**Wiring the cells (`spatial/partition_shroud.rs`):**

- `SimContext::update_partition` drops the changed ids from `partition.update`. Refresh those objects in the returned order, before `process_contacts`.
- After `process_contacts`, call `process_pending_undo_shroud_reveals(frame, player_count)`.
- `init_partition` must follow `TerrainLogic::setActiveBoundary`: flush the queue, store fogged cells, objects leave, store revealed cells, rebuild, restore revealed, objects return, restore fogged.
- Reveal calls take a mask with bit `i` for player `i`. The port has no ordered player list; derive one from `player_default_teams`.

**Stand-ins to replace** (what is wrong with each is in `PARTITION_SCOPE.md`):

- `sim/mod.rs`: `set_shroud_clearing_range`, `reveal_shroud`, `remove_shroud`, `reveal_map_for_player`, `shroud_clearing_range_override`, `SimWorld::{shroud_reveals, shroud_removals, map_reveals}`.
- Users: `active_shroud_upgrade.rs`, `shroud_crate_collide.rs`, `crate_collide.rs`, both car-bomb and hijack conversion collides, `special_power_module.rs`, `spy_vision_special_power.rs`.
- Tests that assert the stand-ins: `golden_wp52_special_power`, `golden_wp52_collide_crates`, `golden_wp52_upgrade`.

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
