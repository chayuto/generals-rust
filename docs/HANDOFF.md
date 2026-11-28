# Handover (2026-11-28)

For the agent continuing this work. Read `AGENTS.md` first: its rules are binding. This
note records where work stopped and how to carry on. Update or delete it once stale.
History is in `git log`; gaps module by module are in `OPEN_DECISIONS.md`.

## Goal and owner's direction

- **A full Rust port of the whole game, client included. No C++** in the result
  (`OPEN_DECISIONS.md` item 10, [FULL_PORT_PLAN.md](FULL_PORT_PLAN.md), six stages).
- **Completion blueprints:** See [FULL_PORT_PLAN.md](FULL_PORT_PLAN.md) (all 6 stages),
  [STAGE1_COMPLETION_PLAN.md](STAGE1_COMPLETION_PLAN.md) (`L1`–`L12` to headless replay CRC parity),
  and [CLIENT_ARCHITECTURE_PLAN.md](CLIENT_ARCHITECTURE_PLAN.md) (Stages 2–6: assets, `wgpu` renderer, UI/input, audio/video, lockstep network).
- Faithful: the exact behaviour of the original. Decide faithful-port design questions
  yourself. The aim is a playable game.
- **Client crates: ask the owner before adding the first one**, naming them and their use
  (item 12; candidate table prepared in `CLIENT_ARCHITECTURE_PLAN.md`). The simulation
  stays bit-exact and free of external crates.
- Keep building while it makes token sense: compute is cheap, tokens are not. The owner
  allowed up to 5 parallel workers when their work is independent.
- Batch pushes (Actions quota); `env:ephemeral-web` assignments are paused.

## Where things stand

Stage 1 (simulation) is in progress. **Nothing is playable: there is no graphics, input,
sound or game loop.**

- A Rust game starts from a retail map. On 24 compared games its digests match the stock
  engine at the three stages of game start, bridges and computer map setup included.
- Frames run with scripts, Dozer and Worker updates, turrets, idle targeting, recorded
  human orders (unit create, upgrade, dozer construct, science) and computer base
  building.
- A move order alone takes a unit to its goal, and an attack order alone destroys a
  target (`path_follow_sim`, `move_order_retail`, `attack_sim`). The duels use
  hand-built units; only moving was tried on retail maps.
- Name keys handed out at engine start match the reference.
- `PORT_MAP.md`: 290 of 463 packets marked implemented, 37 data only, 136 unported.
  **"Implemented" includes many partial packets**, so real progress is lower. By line
  count the whole game is very roughly a fifth to a quarter done.
- Seven small modules and the core services also run inside the C++ engine as a check
  (`BASELINE.md`).

## First 300 frames against the stock engine

`golden_game_frames` runs the 24 games with their recorded commands. It is a ratchet over
`game_frames_agreement.txt` (first differing frame per field and owner group), not a
parity test. It needs the retail game, so CI skips it. How many games agree for all 300:

| Field | Games | Others first differ at |
|---|---|---|
| seed | 9 / 24 | 2 to 241 |
| game, player | 19 / 24 | player: frame 1 |
| map object count | 20 / 24 | 2 to 181 |
| computer object count | 20 / 24 | 2 to 241 |
| computer positions | 5 / 24 | 2 to 181 |
| human object count | 24 / 24 | |
| human positions | 21 / 24 | 172 to 270 |
| human AI state | 22 / 24 | 185 to 223 |

## Slice L3 (2026-11-28, after the rest of this note)

Done: the `ObjectCreationList` runtime (`PKT-391`), the model bone catalog (barrels,
launch and turret bones, dock and spawn bones) and obstacle line of sight for special
ability range. `golden_game_frames` did not move. L4 comes next. The sections
below predate L1 to L3; check `git log` before taking an item from them.

Open from L3:

- Model bones, not ported: transition states, map time of day and weather, disguises,
  bones from `map.ini`, `MinLODRequired`, runtime name-key interning, INI validation
  errors, and other users of bones (garrison, contain exits, ropes).
- Model bones mutation: 60 of 102 mutants tried. Unguarded survivors: the spawn point
  start index, aggregate sub-model loading, and the filter for non-model draw modules.
- `ObjectCreationList`: payload delivery is only recorded (`DeliverPayloadAIUpdate`, L8).
  No mutation result was recorded for the slice.
- `golden_game_frames` and the other retail-data suites now run on `arm64` only: CI has
  no game data.

## Slice L4 (done, 2026-11-28; L5 comes next)

Part 1 done: the player list is `SimWorld::players`; sciences, radar, handicap, bans, cost
and time changes, bounty, attacked-by, preorder and shared power timers live in `Player`;
the power brown-out change is wired (`golden_player_holdings`).

Part 2 done: money, upgrade masks, energy, score keeper, academy statistics, battle plans,
side, colour and playability are `Player` members; the maps are gone, as is the captured
count the reference does not have. Income buckets now rotate in the frame, and `Money`
names player 0 until a template sets it up (harness worlds without templates set it).

Part 3 done: relations, human flag and skirmish AI are read from `Player`; the upgrade
list and the resource gathering lists are `Player` members; `Player::xfer` writes and
loads them, and `PlayerParts` writes the AI player and tunnel tracker
(`golden_player_xfer`, 120 states).

Open:

- No port host passes `xfer_player_part` on to `PlayerParts`, and none answers
  `score_template`: both fail loudly. Loading a tunnel with contained objects is refused.
- `Player::onUpgradeRemoved` and `findBestSupplyWarehouse` (which prunes the warehouse
  list) are not ported. Sim relationship queries ignore team override relations.
- The reference build writes the general name with four bytes a character; the port
  writes two (the Windows game's width; `golden_player_xfer` converts that one field).
  Before L12, check whether any game checksum or compared save reaches a wide string, and
  settle the width if so. The score keeper maps are ordered by template pointer in the
  reference and by template id here: they agreed in the vectors.
- `golden_player_xfer` has had no mutation run.
- Unverified: academy init values and playability of a non-default team have no vectors;
  `AcademyStats::update` is not called.
- Module science grants do not notify scripts, and the `PlayerWorld` brown-out hook is a
  stub.
- Unverified: brown-out during an executing module, team handover, construction.
- Mutation (40 of 81): four survivors not rechecked (`player.rs` `add_science` and the cost
  change match, the radar default in `sim/mod.rs`).
- Object lifecycle gaps closed against `PKT-390_lifecycle.txt` (`golden_object_lifecycle`,
  every row): weapon set changes on flag changes (and the `WeaponSet` save bytes),
  contained units leaving their group and the pathfind map, mines under an object whose
  `UNDER_CONSTRUCTION` changes, produced mines (and parked units) following a defection,
  `onCapture` between teams of one player (`set_object_team_same_player` now runs the
  whole capture), a captured dozer or worker dropping its tasks (non-retail), and battle
  plan bonuses on a new object.
- Not covered by those vectors: parked units defecting through `defect_object`, an allied
  capture keeping a dozer's tasks, a captured worker, and module `on_capture` hooks on a
  same-player transfer. Extend `gen_object_lifecycle` for these.
  Mutation (14 candidates: 11 caught, 1 did not build) left two survivors for the same
  reason: no vector cancels a dozer's build task, or tells the mine scan's 3D collision
  test from a 2D one (the port follows the reference default, 3D).
- Still open in `setTeam` and `onCapture`: a skirmish AI owner sells a captured faction
  structure (`sellObject` is not ported); the new team's initial attitude and attack
  priority set are not applied; `set_object_team_same_player` does not redirect a dead
  player's team to neutral.

## What to do next

In order of what the comparison shows. Rerun it after each merge; the list moves.

1. **Computer player, rest of S21** (about two more workers): team building and unit
   training (queues and their `xfer`, `queueUnits`, `queueDozer`, supply trucks, team
   selection, `BUILD_TEAM`, `startTraining`, `findFactory`, recruiting,
   `onUnitProduced`), then non-skirmish building, rebuild holes, bridge repair,
   defences, upgrades, superweapons. Check that dozer completion calls
   `onStructureProduced`.
2. **Why computer positions differ in 19 games**, many from early frames. Not
   investigated.
3. **A human dozer moves differently after a construct order** (`linux_2p_vanilla_map`,
   `human.ai`, frame 185). The port's dock point leads to a move the reference does not
   make. Cause not found; suspect dozer or position-search code.
4. Small causes: `StructureCollapseUpdate`; `RailroadBehavior` (extends
   `PhysicsBehavior`, replacing its update, collision and save; `createCarriages` and
   train movement are behind a locomotive flag); the random draws of `BoneFXUpdate`
   creation lists (not rechecked since `spawn_ocl` began creating them); one car in `tournament_city` with
   angle -pi against +pi at frame 2.
5. Script actions and conditions still counted as unported (`GameScripts::unported`,
   printed with `GAME_FRAMES_DUMP`), among them `BUILD_TEAM`,
   `TEAM_USE_COMMANDBUTTON_ABILITY`, special power readiness, trigger areas, and group,
   team-script and kill orders.
6. `AI_PORT_PLAN.md`: S11 other non-combat states; S15 supply, dock and transport (the
   Worker's supply half is missing); S17 aircraft and missiles (**missiles never
   detonate**); S18 guard states; S19; S20. Also squad, hunt and attack-area states and
   turret `transferAttack`. Check `AI_PORT_PLAN.md` "Done" notes before starting.
7. **A game loop.** The frame hookup (`FrameSeams`) lives in the test; nothing owns it.
8. **A game-wide CRC.** Stage 1 is done only when a Rust headless replay matches the C++
   CRC at every checkpoint.

Asset readers (stage 2) touch separate code and can run beside the simulation.

## Verification gaps from the last round

- Computer base building: no focused oracle vectors, no mutation run, placement
  rejection paths unverified. `isLocationLegalToBuild` is not applied to human dozer
  orders; the terrain construction restriction is missing.
- Turrets: no mutation run; no vectors for turreted containers, airborne pitch, bridges,
  computer crushing, version 2 saves or loading.
- Science purchase: two survivors have a new test, not rechecked.
- `BoneFXUpdate`: 9 survivors not rechecked (load path, bone positions, rubble state, FX
  calls).
- Many earlier slices have mutation survivors not rechecked; see `OPEN_DECISIONS.md`.
- An unverified `StructureCollapseUpdate` draft was left in a temporary session folder;
  assume it is gone.
- Gate 0 is still open (the public replays mismatch the local data). `check_replays.py`
  covers only the swapped parts.
- `PORT_MAP.md` shows as implemented, but only in part: `PKT-216`, `PKT-401`, `PKT-403`,
  `PKT-411`, `PKT-421`, `PKT-426`, `PKT-427`, `PKT-431` to `PKT-433`, `PKT-440`.
- No status was posted to GitHub Issues or Discussions.

## Running the work

One coordinator, workers in their own git worktrees. Give each worker
`docs/WORKER_BRIEF.md` plus a short task naming the files it owns.

Merging a worker's commits:

1. Check author and message (no co-author, session link or assistant name), and that no
   game-data file type is in the diff. Vectors hold digests or synthetic data only.
2. `git cherry-pick`. On a `docs/PORT_MAP.md` conflict take theirs and run
   `python3 scripts/port_map.py --write`. On a `game_frames_agreement.txt` conflict take
   theirs, rewrite with `GAME_FRAMES_RATCHET=write`, and compare field by field with the
   version from before the merge. Explain every loss.
3. `cargo test --workspace`, native only. CI covers x86 (`AGENTS.md` §4.4): do not run it
   locally unless CI failed or you are fixing an x86 bug.
   Set `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0`.
4. `lint_determinism.py`, `port_map.py --check`, `gen_engine_sim.py --check`,
   `cargo check -p zh_logic --features retail-crc` and `retail-xfer-save`, every
   `scripts/test_*.py`, `validate_manifest.py`, `check_vendor.py`.
5. Push once per batch. Do not wait for CI: it is advisory (`AGENTS.md` §4.4). Read the
   previous run before the next push and fix real failures.
6. Remove the worktree: delete its `vendor` symlink first, then `git worktree remove`.

## Pitfalls

- **Parallel slices break each other's traces.** A trace recorded while a state or search
  was unported replaced it; once ported, its test must say so
  (`AiSeams::replaces_state`, `replaces_searches`, `replaces_attack_path`). This failed
  several merges.
- **A timed-out `cargo test` leaves its test binary running**, which can spin for an
  hour. Kill the process group (`mutate_check.run`). If a worker looks stuck, check `ps`
  for test binaries with parent 1.
- Workers' isolation check misreads the reference's `Source/` folder as a shell builtin.
  Workers have repeatedly got round it with globs or shell tools despite instructions;
  `vendor/` stayed unchanged. Run `check_vendor.py` after every batch.
- `rustfmt` rewrites a file's out-of-line `mod` files too.
- A frame field can agree by coincidence while two parts are missing
  (`bitter_winter` money did). A later "loss" may be that coincidence ending: show it.
- Name keys are numbered in request order: a new startup request must go where the
  reference makes it. Keys first requested during a game do not match.
- A script chunk reader given an empty `ScriptTemplateRegistry` makes every condition
  false without an error.
- `BuildAssistant.cpp` uses the double `WWMath::Sqrt` of a sum of `Pow` results, which
  differs from `gm_sqrtf` by an ulp for some inputs.
- x86 gives a 0/0 NaN a sign bit: use `zh_common::math::reference_div_f32` where the
  value can reach saved state.
- The reference writes retail save versions (`RETAIL_COMPATIBLE_XFER_SAVE=1`); a
  whole-run save comparison needs the `retail-xfer-save` feature.
- `TeamFactory` member lists are filled at game start only; frames keep
  `world.team_members`.
- A harness that touches shroud cells must replace the local player
  (`gen_partition_shroud.cpp`).
- The repo is not `cargo fmt` clean. Do not reformat files you are not changing.

## Cost seen

Slice workers used 260k to 640k tokens each, most of them 300k to 500k, and about
9 million in total from game start to here. Long contexts are the main cost: split large
slices and hold workers to the limit in `WORKER_BRIEF.md`. Estimates made in advance
have run low.

## Waiting on the owner

- Item 12, client crates: ask when first needed (`CLIENT_ARCHITECTURE_PLAN.md` §2 documents the proposed crate table and the pure-Rust vs FFmpeg `.bik` video option; Stage 2 `zh_assets`, pure-Rust `zh_client_core`, and `zh_net` require zero external crates).
- Whether a searchable copy of the reference source under another folder name may be made
  for workers. It would sidestep the isolation check, so it needs a yes.
- The other unanswered items in `OPEN_DECISIONS.md`.
