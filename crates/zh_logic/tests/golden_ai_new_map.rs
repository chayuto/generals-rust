//! `AIPlayer::newMap` and `AISkirmishPlayer::newMap` compared with the running
//! reference engine (`docs/golden_vectors/AiNewMap.json`, recorded by
//! `tests/cpp_harness/gen_ai_new_map.py`).
//!
//! `synth` lines are calls of the reference's functions with chosen inputs. The other
//! lines are digests of what a game start leaves behind that
//! `golden_game_start.rs` does not compare, before the players' `newMap`, at the
//! end, and after `AIPlayer::newMap` is run again over a build list marked as built
//! (stage `built`): the logic seed, computer players, build lists, score and energy,
//! object names and script status, unit orders, the pathfind map and the shroud.
//! Only `built` reaches the plain `AIPlayer` building loop, supply centers, names and
//! unsellable entries.
//!
//! The retail test needs the game (`ZH_DIR`) and the replays with their user maps
//! (`ZH_USER_DIR`), as `golden_game_start.rs` does; without them it passes without
//! comparing. `AI_NEW_MAP_DUMP=<dir>` writes the port's rows.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zh_common::io::file::OsDiskFileHost;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Region3D};
use zh_common::random_value::PrngState;
use zh_common::types::{ObjectId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferSave};
use zh_logic::ai::ai_player::{base_center_and_radius, new_map};
use zh_logic::ai::ai_skirmish_player::move_build_list;
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::gamelogic::game_logic::{start_new_game_observed, GameStart, StartStage};
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::player::build_assistant::{move_away_box, move_away_destination, BuildContext, PI};
use zh_logic::player::player_list::PlayerList;
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::sim::{SimContext, SimView};
use zh_logic::spatial::sides_list::BuildListInfo;

#[path = "game_start_support/mod.rs"]
#[allow(dead_code)] // the row dump is `golden_game_start.rs`'s
mod support;

use support::{options_parms, percent_decode, replay_parms, retail_dirs};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/AiNewMap.json");
/// The cases of `golden_game_start.rs`, which name each game's options string.
const GAMES: &str = include_str!("../../../docs/golden_vectors/GameStart.json");

const KINDS: [&str; 11] = [
    "seed", "flags", "ai", "build", "score", "energy", "objx", "unit", "path", "cells", "shroud",
];
/// `(game, stage)` where a computer player tells a unit to get out of a new
/// building's way. The reference's move state then reserves the goal cell and asks
/// for a path; the move states are not ported (`AI_PORT_PLAN.md` S10), so the `path`
/// rows differ.
const UNITS_TOLD_TO_MOVE: [(&str, &str); 9] = [
    ("linux_6p_custom_map.rep", "end"),
    ("linux_6p_custom_map.rep", "built"),
    ("macos_6p_custom_map.rep", "end"),
    ("macos_6p_custom_map.rep", "built"),
    ("macos_2p_custom_map.rep", "built"),
    ("whiteout.rep", "built"),
    ("silent_river.rep", "built"),
    ("dark_night.rep", "end"),
    ("dark_night.rep", "built"),
];

fn lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter_map(|line| {
        line.trim()
            .trim_end_matches(',')
            .strip_prefix('"')?
            .strip_suffix('"')
    })
}

/// The `key=value` words of a line.
fn words(line: &str) -> BTreeMap<&str, &str> {
    line.split(' ').filter_map(|w| w.split_once('=')).collect()
}

fn real(hex: &str) -> f32 {
    f32::from_bits(u32::from_str_radix(hex, 16).expect("float bits"))
}

fn reals(list: &str) -> Vec<f32> {
    list.split(',').map(real).collect()
}

fn entries(list: &str) -> impl Iterator<Item = Vec<&str>> {
    list.split(',')
        .filter(|e| !e.is_empty())
        .map(|e| e.split(':').collect())
}

fn synth(kind: &str) -> Vec<BTreeMap<&'static str, &'static str>> {
    let prefix = format!("synth {kind} ");
    lines(GOLDEN)
        .filter(|l| l.starts_with(&prefix))
        .map(words)
        .collect()
}

#[test]
fn golden_adjust_build_list() {
    let cases = synth("adjust");
    assert_eq!(cases.len(), 53);
    for case in cases {
        // `c` entries are command centers.
        let mut list: Vec<BuildListInfo> = case["shape"]
            .chars()
            .zip(entries(case["in"]))
            .map(|(c, at)| {
                let mut info = BuildListInfo::new();
                info.template_name = c.to_string();
                info.location.x = real(at[0]);
                info.location.y = real(at[1]);
                info
            })
            .collect();
        let start = reals(case["start"]);
        let bounds = reals(case["bounds"]);
        let mut region = Region3D::default();
        region.lo.x = bounds[0];
        region.lo.y = bounds[1];
        region.hi.x = bounds[2];
        region.hi.y = bounds[3];
        move_build_list(
            &mut list,
            |name| name == "c",
            &Coord3D {
                x: start[0],
                y: start[1],
                z: 0.0,
            },
            &region,
            case["rotate"] == "1",
        );
        let got: Vec<(u32, u32, bool)> = list
            .iter()
            .map(|i| {
                (
                    i.location.x.to_bits(),
                    i.location.y.to_bits(),
                    i.is_initially_built != FALSE,
                )
            })
            .collect();
        let expected: Vec<(u32, u32, bool)> = entries(case["out"])
            .map(|e| (real(e[0]).to_bits(), real(e[1]).to_bits(), e[2] == "1"))
            .collect();
        assert_eq!(got, expected, "{case:?}");
    }
}

#[test]
fn golden_base_center_and_radius() {
    let cases = synth("center");
    assert_eq!(cases.len(), 36);
    for case in cases {
        let buildings: Vec<(Coord3D, f32)> = entries(case["in"])
            .map(|e| {
                let at = Coord3D {
                    x: real(e[0]),
                    y: real(e[1]),
                    z: 0.0,
                };
                (at, real(e[2]))
            })
            .collect();
        let (center, radius, set) = base_center_and_radius(&buildings);
        let out: Vec<&str> = case["out"].split(',').collect();
        assert_eq!(
            [
                center.x.to_bits(),
                center.y.to_bits(),
                radius.to_bits(),
                u32::from(set != FALSE)
            ],
            [
                real(out[0]).to_bits(),
                real(out[1]).to_bits(),
                real(out[2]).to_bits(),
                u32::from(out[3] == "1")
            ],
            "{case:?}"
        );
    }
}

/// `WWMath::Pow(x, 2)` over two million values in the reference never differed from
/// `x * x`, which `move_away_box` relies on.
#[test]
fn golden_square_is_a_product() {
    let cases = synth("square");
    assert_eq!(cases.len(), 16);
    for case in cases {
        let x = real(case["in"]);
        assert_eq!((x * x).to_bits(), real(case["out"]).to_bits(), "{case:?}");
    }
    assert!(lines(GOLDEN).any(|l| l == "synth squares differ=0"));
}

#[test]
fn golden_move_for_construction() {
    let cases = synth("move");
    assert_eq!(cases.len(), 48);
    for case in cases {
        let kind = if case["box"] == "1" {
            GeometryType::Box
        } else {
            GeometryType::Cylinder
        };
        let geometry =
            GeometryInfo::new(kind, FALSE, 20.0, real(case["major"]), real(case["minor"]));
        let at = reals(case["at"]);
        let mut rng = PrngState::default();
        rng.init_random_with_seed(case["seed"].parse().expect("seed"));
        let scale = rng.get_game_logic_random_value_real(0.5, 1.5);
        let dir = rng.get_game_logic_random_value_real(-PI, PI);
        let (_, radius) = move_away_box(&geometry);
        let to = move_away_destination(
            &Coord3D {
                x: at[0],
                y: at[1],
                z: at[2],
            },
            radius,
            scale,
            dir,
        );
        let expected = reals(case["out"]);
        // The order's goal height is the ground's, which these inputs do not give.
        assert_eq!(
            [to.x.to_bits(), to.y.to_bits()],
            [expected[0].to_bits(), expected[1].to_bits()],
            "{case:?}"
        );
    }
}

fn digest(rows: &[String]) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for row in rows {
        for &byte in row.as_bytes().iter().chain(b"\n") {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{}:{hash:016x}", rows.len())
}

fn saved(snapshot: impl FnOnce(&mut XferSave)) -> String {
    let mut xfer = XferSave::new();
    snapshot(&mut xfer);
    xfer.into_bytes()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The rows `gen_ai_new_map.cpp` writes, apart from object names.
fn rows(sim: &SimContext, players: &PlayerList) -> Vec<String> {
    let mut out = vec![format!(
        "seed\t{:08x}",
        sim.world.rng.get_game_logic_random_seed_crc()
    )];
    for i in 0..players.player_count() {
        let player = players.player(i).expect("player");
        out.push(format!("flags\t{i}\t{}", u8::from(player.can_build_units)));
        if let Some(ai) = sim.world.ai_players.get(&i) {
            let mut ai = ai.clone();
            let bytes = saved(|x| ai.xfer(x).expect("save"));
            out.push(format!("ai\t{i}\t{bytes}"));
        }
        for (at, info) in player.build_list.iter().enumerate() {
            let mut info = info.clone();
            let bytes = saved(|x| info.xfer(x).expect("save"));
            out.push(format!("build\t{i}\t{at}\t{bytes}"));
        }
        let score = sim.world.score_keepers.get(&i);
        let mut built: Vec<String> = score.map_or_else(Vec::new, |s| {
            s.objects_built
                .values()
                .map(|e| format!("{}:{}", e.name, e.count))
                .collect()
        });
        built.sort();
        let built: String = built.iter().map(|e| format!("{e},")).collect();
        out.push(format!(
            "score\t{i}\t{}\t{}\t{}\t{}\t{built}",
            score.map_or(0, |s| s.total_money_spent),
            score.map_or(0, |s| s.total_money_earned),
            score.map_or(0, |s| s.total_units_built),
            score.map_or(0, |s| s.total_buildings_built),
        ));
        let energy = sim.world.player_energy.get(&i);
        out.push(format!(
            "energy\t{i}\t{}\t{}",
            energy.map_or(0, |e| e.get_production(sim.world.frame)),
            energy.map_or(0, |e| e.get_consumption()),
        ));
    }
    let mut objects: Vec<_> = sim
        .world
        .objects
        .world_iter()
        .filter_map(|slot| sim.world.objects.get(slot))
        .collect();
    objects.sort_by_key(|o| o.id.0);
    for obj in objects {
        out.push(format!("objx\t{}\t\t{}", obj.id.0, obj.script_status));
        if obj.is_destroyed() {
            continue;
        }
        let Some(ai) = obj
            .ai_slot
            .and_then(|m| sim.world.modules.payload(m))
            .and_then(|m| m.as_any())
            .and_then(|a| a.downcast_ref::<AiUpdate>())
        else {
            continue;
        };
        let goal = ai.machine.goal_position;
        out.push(format!(
            "unit\t{}\t{}\t{:08x}\t{:08x}\t{:08x}\t{}\t{}",
            obj.id.0,
            ai.machine.current_state_id(),
            goal.x.to_bits(),
            goal.y.to_bits(),
            goal.z.to_bits(),
            ai.machine.goal_object_id.0,
            ai.last_command_source as i32,
        ));
        out.push(format!(
            "path\t{}\t{}\t{}\t{}",
            obj.id.0,
            ai.pathfind_goal_cell.x,
            ai.pathfind_goal_cell.y,
            u8::from(ai.waiting_for_path != FALSE),
        ));
    }
    if let Some(pathfinder) = sim.world.pathfinder.as_ref() {
        let map = pathfinder.map();
        let ground = PathfindLayerEnum::Ground;
        let mut y = 0;
        while pathfinder.get_cell(ground, 0, y).is_some() {
            let mut row = format!("cells\t{y}\t");
            let mut x = 0;
            while let Some(index) = pathfinder.get_cell(ground, x, y) {
                let cell = map.grid().cell(index);
                let obstacle = cell.obstacle_id(map.pool(), map.grid().mode());
                row += &format!("{}.{},", cell.cell_type() as i32, obstacle.0);
                x += 1;
            }
            out.push(row);
            y += 1;
        }
    }
    let partition = &sim.world.partition;
    for player in 0..players.player_count() {
        let mut row = format!("shroud\t{player}\t");
        for y in 0..partition.cell_count_y() {
            for x in 0..partition.cell_count_x() {
                row.push(char::from(
                    b'0' + partition.shroud_status(player, x, y) as u8,
                ));
            }
        }
        out.push(row);
    }
    out
}

/// Puts the names `Object::setName` gave into the `objx` rows.
fn name_rows(rows: &mut [String], named: &[(String, ObjectId)]) {
    for row in rows.iter_mut().filter(|r| r.starts_with("objx\t")) {
        let cells: Vec<&str> = row.split('\t').collect();
        let id: i32 = cells[1].parse().expect("object id");
        let name = named
            .iter()
            .rev()
            .find(|(_, object)| object.0 == id)
            .map_or("", |(name, _)| name);
        *row = format!("objx\t{id}\t{name}\t{}", cells[3]);
    }
}

/// A game's rows at the three stages.
struct Rows {
    before: Vec<String>,
    end: Vec<String>,
    built: Vec<String>,
}

/// The draws the port does not make yet: a new turret's idle state picks its first
/// scan time in the constructor (`TurretAIIdleState::resetIdleScan`, turrets are
/// `AI_PORT_PLAN.md` S14). Only `built` raises turrets; the real map start does not.
fn draw_for_new_turrets(game: &mut GameStart, first_new: i32) {
    let world = &mut game.sim.world;
    let intervals: Vec<(i32, i32)> = world
        .objects
        .world_iter()
        .filter_map(|slot| world.objects.get(slot))
        .filter(|o| o.id.0 >= first_new)
        .filter_map(|o| world.modules.payload(o.ai_slot?)?.as_any())
        .filter_map(|a| a.downcast_ref::<AiUpdate>())
        .flat_map(|ai| ai.data.turrets.iter().flatten())
        .map(|t| {
            (
                t.min_idle_scan_interval as i32,
                t.max_idle_scan_interval as i32,
            )
        })
        .collect();
    for (min, max) in intervals {
        world.rng.get_game_logic_random_value(min, max);
    }
}

/// The first structure by name of `side` which docks supply trucks.
fn supply_center_of(game: &GameStart, side: &str) -> Option<String> {
    let factory = &game.sim.rules.thing_factory;
    (1..factory.next_template_id())
        .filter_map(|id| factory.find_by_template_id(id))
        .filter(|t| {
            t.get_default_owning_side() == side
                && t.is_kind_of(KindOfType::Structure) != FALSE
                && t.get_behavior_module_info()
                    .nuggets()
                    .iter()
                    .any(|n| n.name == "SupplyCenterDockUpdate")
        })
        .map(|t| t.get_name().to_owned())
        .min()
}

/// What the generator's `build_whole_base` does: `AIPlayer::newMap` once more over
/// the first skirmish player's build list, most of it marked as built.
fn build_whole_base(game: &mut GameStart) {
    let sim = &game.sim;
    let player = (0..game.players.player_count()).find(|&p| {
        sim.world
            .ai_players
            .get(&p)
            .is_some_and(|ai| ai.skirmish.is_some())
            && sim.world.objects.world_iter().any(|slot| {
                sim.world.objects.get(slot).is_some_and(|o| {
                    game.players
                        .player(p)
                        .is_some_and(|player| player.default_team == Some(o.team_id))
                }) && sim.object_is_kind_of(slot, KindOfType::CommandCenter) != FALSE
            })
    });
    let Some(player) = player else {
        return;
    };
    let list = &mut game.players.player_mut(player).expect("player").build_list;
    for (i, info) in list.iter_mut().enumerate() {
        info.is_initially_built = i32::from(i % 5 != 4) as _;
        info.unsellable = i32::from(i % 3 == 0) as _;
        if i % 4 == 0 {
            info.building_name = format!("n{i}");
        }
    }
    let side = game.players.player(player).expect("player").side.clone();
    let center = game
        .sim
        .world
        .objects
        .world_iter()
        .filter(|&slot| game.sim.object_is_kind_of(slot, KindOfType::CommandCenter) != FALSE)
        .filter_map(|slot| game.sim.world.objects.get(slot))
        .find(|o| game.players.player(player).expect("player").default_team == Some(o.team_id))
        .map(|o| *o.thing.get_position());
    if let (Some(name), Some(mut at)) = (supply_center_of(game, &side), center) {
        at.x += 300.0;
        at.y += 260.0;
        let mut info = BuildListInfo::new();
        info.template_name = name;
        info.location = at;
        info.is_initially_built = TRUE;
        let player = game.players.player_mut(player).expect("player");
        player.build_list.push(info);
    }
    let first_new = game.sim.world.next_object_id;
    let mut unported = BTreeMap::new();
    new_map(
        &mut BuildContext {
            sim: &mut game.sim,
            players: &mut game.players,
            named: &mut game.named_objects,
            unported: &mut unported,
            // Only the skirmish setup reads the extent.
            pathfind_extent: Region3D::default(),
        },
        player,
    );
    assert!(unported.is_empty(), "{unported:?}");
    draw_for_new_turrets(game, first_new);
}

fn start(name: &str, options: Option<&str>) -> Option<Rows> {
    let (game_dir, user_dir) = retail_dirs()?;
    let fs = mount_game_archives(&OsDiskFileHost, &game_dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let parms = match options {
        Some(options) => options_parms(&fs, &loader, options, &game_dir, &user_dir),
        None => {
            let path = PathBuf::from(&user_dir).join("Replays").join(name);
            replay_parms(&fs, &loader, &path, &game_dir, &user_dir)
        }
    }?;
    let mut before = Vec::new();
    let mut game = start_new_game_observed(loader, parms, |stage, sim, players, _| {
        if let StartStage::BeforePlayerMaps = stage {
            before = rows(sim, players);
        }
    })
    .expect("start");
    let mut end = rows(&game.sim, &game.players);
    // Only the start's map objects carry names, so both stages take the final list.
    name_rows(&mut before, &game.named_objects);
    name_rows(&mut end, &game.named_objects);
    build_whole_base(&mut game);
    let mut built = rows(&game.sim, &game.players);
    name_rows(&mut built, &game.named_objects);
    Some(Rows { before, end, built })
}

#[test]
fn retail_map_setup_matches_the_reference() {
    let options: BTreeMap<&str, String> = lines(GAMES)
        .filter_map(|l| {
            let name = l.split(' ').next()?;
            Some((name, percent_decode(words(l).get("options")?)))
        })
        .collect();
    let cases: Vec<&str> = lines(GOLDEN).filter(|l| !l.starts_with("synth ")).collect();
    assert_eq!(cases.len(), 72, "24 games at three stages");
    let mut failures = Vec::new();
    let mut current: Option<(&str, Rows)> = None;
    let mut compared = 0;
    for line in cases {
        let name = line.split(' ').next().expect("game");
        let values = words(line);
        if current.as_ref().is_none_or(|c| c.0 != name) {
            let Some(rows) = start(name, options.get(name).map(String::as_str)) else {
                eprintln!("skipped: the game or {name} is not installed");
                continue;
            };
            current = Some((name, rows));
            compared += 1;
        }
        let Some((_, game)) = current.as_ref() else {
            continue;
        };
        let stage = values["stage"];
        let ours = match stage {
            "end" => &game.end,
            "built" => &game.built,
            _ => &game.before,
        };
        if let Ok(dir) = std::env::var("AI_NEW_MAP_DUMP") {
            let file = format!("{}.{stage}.rs.ai.txt", name.trim_end_matches(".rep"));
            std::fs::write(PathBuf::from(dir).join(file), ours.join("\n") + "\n")
                .expect("write dump");
        }
        for kind in KINDS {
            let of: Vec<String> = ours
                .iter()
                .filter(|r| r.split('\t').next() == Some(kind))
                .cloned()
                .collect();
            let got = digest(&of);
            // Known to differ: a fix has to take the game off the list.
            if kind == "path" && UNITS_TOLD_TO_MOVE.contains(&(name, stage)) {
                assert_ne!(got, values[kind], "{name}: the move states are ported now?");
                continue;
            }
            if got != values[kind] {
                failures.push(format!(
                    "{name} {stage} {kind}: reference {}, port {got}",
                    values[kind]
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    eprintln!("compared {compared} game starts");
}
