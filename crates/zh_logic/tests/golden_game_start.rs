//! `GameLogic::startNewGame` compared with the running reference engine: for each of
//! the baseline replays, and for games on other retail maps described by an options
//! string alone, the players, teams, objects and bridges at three stages of the start.
//! `tests/cpp_harness/gen_game_start.py` recorded `docs/golden_vectors/GameStart.json`
//! (counts and digests only: the rows name map content).
//!
//! Every digest is compared and has to match, apart from what a map with bridges
//! rules out.
//!
//! The retail tests need the game (`ZH_DIR`, default `$HOME/Games/generalszh`) and the
//! replays with their user maps (`ZH_USER_DIR`, default
//! `$HOME/Library/Application Support/GeneralsX/GeneralsZH`); without them they pass
//! without comparing. `GAME_START_DUMP=<dir>` writes the port's rows for a diff
//! against the reference's.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zh_common::io::file::OsDiskFileHost;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::gamelogic::game_logic::{start_new_game_observed, GameStart, StartStage};
use zh_logic::modules::update::ai_update::{AiUpdate, DerivedAi};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;

#[path = "game_start_support/mod.rs"]
mod support;

use support::{dump, options_parms, percent_decode, replay_parms, retail_dirs};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/GameStart.json");

/// Logic frames each started game runs.
const FRAMES: u32 = 300;

/// FNV-1a over the rows, each ended by a newline.
fn digest<'a>(rows: impl Iterator<Item = &'a str>) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for row in rows {
        for &byte in row.as_bytes().iter().chain(b"\n") {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{hash:016x}")
}

/// `(column, tab fields)` of an object row; the id is always included.
const COLUMNS: [(&str, &[usize]); 8] = [
    ("template", &[2]),
    ("team", &[3, 4, 5]),
    ("position", &[6, 7, 8]),
    ("angle", &[9]),
    ("layer", &[10]),
    ("health", &[11, 12]),
    ("status", &[13, 14]),
    ("vision", &[15, 16]),
];

/// The digests a golden line holds, computed from dump rows.
fn summarize(rows: &[String]) -> Vec<(String, String)> {
    let of = |kind: &str| -> Vec<&str> {
        rows.iter()
            .filter(|r| r.split('\t').next() == Some(kind))
            .map(String::as_str)
            .collect()
    };
    let mut out = Vec::new();
    // `deck` rows are the pathfind cells under a bridge.
    for kind in ["seed", "game", "player", "team", "bridge", "deck"] {
        let rows = of(kind);
        out.push((format!("{kind}_count"), rows.len().to_string()));
        out.push((format!("{kind}_digest"), digest(rows.into_iter())));
    }
    let objects: Vec<Vec<&str>> = of("obj")
        .into_iter()
        .map(|r| r.split('\t').collect())
        .collect();
    for (group, started) in [("map", false), ("start", true)] {
        let cells: Vec<&Vec<&str>> = objects
            .iter()
            .filter(|c| {
                let team = c.get(4).copied().unwrap_or("");
                (team.starts_with("teamplayer") || team.starts_with("teamSkirmish")) == started
            })
            .collect();
        out.push((format!("{group}_count"), cells.len().to_string()));
        for (name, fields) in COLUMNS {
            let column: Vec<String> = cells
                .iter()
                .map(|c| {
                    let mut line = c.get(1).copied().unwrap_or("").to_owned();
                    for &f in fields {
                        line.push('\t');
                        line.push_str(c.get(f).copied().unwrap_or(""));
                    }
                    line
                })
                .collect();
            out.push((
                format!("{group}_{name}_digest"),
                digest(column.iter().map(String::as_str)),
            ));
        }
    }
    // The map's objects other than bridges, by content alone.
    let mut loose: Vec<String> = objects
        .iter()
        .filter(|c| {
            let team = c.get(4).copied().unwrap_or("");
            !(team.starts_with("teamplayer") || team.starts_with("teamSkirmish"))
                && !c.get(2).is_some_and(|t| t.contains("Bridge"))
        })
        .map(|c| {
            let mut cells = vec![c[2], c[4], c[5]];
            cells.extend_from_slice(&c[6..14]);
            cells.extend_from_slice(&c[15..17]);
            cells.join("\t")
        })
        .collect();
    loose.sort();
    out.push(("unbridged_count".to_owned(), loose.len().to_string()));
    out.push((
        "unbridged_digest".to_owned(),
        digest(loose.iter().map(String::as_str)),
    ));
    out
}

/// One golden line per dump: `case stage=<stage> [options=<%-encoded>] key=value ...`.
struct Case {
    name: String,
    stage: String,
    /// The options string of a game no replay on disk plays.
    options: Option<String>,
    values: Vec<(String, String)>,
}

fn golden_cases() -> Vec<Case> {
    GOLDEN
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',').strip_prefix('"')?;
            let line = line.strip_suffix('"')?;
            let mut words = line.split(' ');
            let name = words.next()?.to_owned();
            let stage = words.next()?.strip_prefix("stage=")?.to_owned();
            let mut options = None;
            let mut values = Vec::new();
            for (k, v) in words.filter_map(|w| w.split_once('=')) {
                if k == "options" {
                    options = Some(percent_decode(v));
                } else {
                    values.push((k.to_owned(), v.to_owned()));
                }
            }
            Some(Case {
                name,
                stage,
                options,
                values,
            })
        })
        .collect()
}

/// Stages recorded per game.
const STAGES: usize = 3;
/// Starting units which `adjustDestination` moves in the reference (the ones a port
/// without it placed differently). Each reserves its cell.
const MOVED_UNITS: u32 = 42;
/// Replays of the baseline, and games made from options strings.
const REPLAY_GAMES: usize = 6;
const VARIANT_GAMES: usize = 18;
/// `(game, bridges drawn into the map, towers of landmark bridges)` of the games
/// whose map has bridges; every other game has none.
const BRIDGED_GAMES: [(&str, u32, u32); 4] = [
    ("alpine_assault.rep", 0, 4),
    ("dark_night.rep", 3, 0),
    ("silent_river.rep", 3, 0),
    ("winding_river.rep", 4, 0),
];

#[test]
fn golden_lists_every_game_at_every_stage() {
    let cases = golden_cases();
    assert_eq!(cases.len(), STAGES * (REPLAY_GAMES + VARIANT_GAMES));
    assert_eq!(
        cases.iter().filter(|c| c.options.is_some()).count(),
        STAGES * VARIANT_GAMES
    );
    for case in cases {
        assert_eq!(case.values.len(), 33, "{}", case.name);
    }
}

/// A started game with its rows at each stage.
struct Started {
    game: GameStart,
    before_units: Vec<String>,
    before_ai: Vec<String>,
    end: Vec<String>,
}

impl Started {
    fn rows(&self, stage: &str) -> &[String] {
        match stage {
            "before_units" => &self.before_units,
            "before_ai" => &self.before_ai,
            _ => &self.end,
        }
    }
}

fn start(case: &Case) -> Option<Started> {
    let (game_dir, user_dir) = retail_dirs()?;
    let fs = mount_game_archives(&OsDiskFileHost, &game_dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let parms = match &case.options {
        Some(options) => options_parms(&fs, &loader, options, &game_dir, &user_dir),
        None => {
            let path = PathBuf::from(&user_dir).join("Replays").join(&case.name);
            replay_parms(&fs, &loader, &path, &game_dir, &user_dir)
        }
    }?;
    let mut before_units = Vec::new();
    let mut before_ai = Vec::new();
    let game = start_new_game_observed(loader, parms, |stage, sim, players, teams| {
        let rows = dump(sim, players, teams);
        match stage {
            StartStage::BeforeStartingUnits => before_units = rows,
            StartStage::BeforePlayerMaps => before_ai = rows,
        }
    })
    .expect("start");
    let end = dump(&game.sim, &game.players, &game.teams);
    Some(Started {
        game,
        before_units,
        before_ai,
        end,
    })
}

/// The starting objects' `(id, position digest)`, in id order.
fn start_positions(rows: &[String]) -> Vec<(i32, String)> {
    rows.iter()
        .map(|r| r.split('\t').collect::<Vec<_>>())
        .filter(|c| {
            c.first() == Some(&"obj")
                && c.get(4)
                    .is_some_and(|t| t.starts_with("teamplayer") || t.starts_with("teamSkirmish"))
        })
        .map(|c| {
            let line = [c[1], c[6], c[7], c[8]].join("\t");
            (
                c[1].parse().unwrap_or(0),
                digest(std::iter::once(line.as_str())),
            )
        })
        .collect()
}

/// The starting dozers and workers which hold a goal cell (`updateGoal`), which
/// no digest shows. A reserved cell must name its unit.
fn count_reservations(game: &GameStart, failures: &mut Vec<String>, name: &str) -> u32 {
    let world = &game.sim.world;
    let Some(pathfinder) = world.pathfinder.as_ref() else {
        return 0;
    };
    let mut reserved = 0;
    for slot in world.objects.world_iter() {
        let Some(obj) = world.objects.get(slot) else {
            continue;
        };
        let Some(ai) = obj
            .ai_slot
            .and_then(|m| world.modules.payload(m))
            .and_then(|m| m.as_any())
            .and_then(|a| a.downcast_ref::<AiUpdate>())
        else {
            continue;
        };
        let cell = ai.pathfind_goal_cell;
        if matches!(ai.derived, DerivedAi::Base) || (cell.x, cell.y) == (-1, -1) {
            continue;
        }
        reserved += 1;
        let holder = pathfinder
            .get_cell(PathfindLayerEnum::Ground, cell.x, cell.y)
            .map(|idx| pathfinder.goal_unit(idx));
        if holder != Some(obj.id) {
            failures.push(format!("{name}: unit {:?} holds no goal cell", obj.id));
        }
    }
    reserved
}

/// Every map object is made or counted as skipped, the bridges and towers are the
/// ones expected, and the calls that were left out are counted once each.
fn check_report(game: &GameStart, before: &[String], failures: &mut Vec<String>, name: &str) {
    let report = &game.report;
    let skipped: u32 = report.skipped_objects.values().sum();
    let map_rows = before
        .iter()
        .filter(|r| {
            let c: Vec<&str> = r.split('\t').collect();
            c.first() == Some(&"obj")
                && !c
                    .get(4)
                    .is_some_and(|t| t.starts_with("teamplayer") || t.starts_with("teamSkirmish"))
        })
        .count();
    // A bridge drawn into the map comes from two bridge points, which are counted as
    // skipped, and a landmark bridge's towers from none.
    let made = report.created_from_map + report.map_bridges + report.bridge_towers;
    if report.created_from_map + skipped != report.map_objects || made as usize != map_rows {
        failures.push(format!(
            "{name}: {} map objects, {made} made ({map_rows} listed), {skipped} skipped",
            report.map_objects
        ));
    }
    let bridges = BRIDGED_GAMES
        .iter()
        .find(|(game, ..)| *game == name)
        .map_or((0, 0), |&(_, drawn, towers)| (drawn, towers));
    if (report.map_bridges, report.bridge_towers) != bridges
        || report.unported.contains_key("bridge model not supplied")
    {
        failures.push(format!(
            "{name}: {} bridges and {} towers, expected {bridges:?}",
            report.map_bridges, report.bridge_towers
        ));
    }
    let count = |call: &str| report.unported.get(call).copied().unwrap_or(0);
    // Every mobile starting unit has an AI to place it, and the computer players'
    // map setup leaves nothing out.
    let left_out = report.unported.keys().any(|call| {
        [
            "buildObjectNow",
            "onStructureProduced",
            "runObjectScript",
            "aiMoveTo",
        ]
        .iter()
        .any(|part| call.contains(part))
    });
    if left_out || count("mobile starting unit without AI") != 0 {
        failures.push(format!("{name}: unported calls {:?}", report.unported));
    }
}

#[test]
fn retail_game_starts_match_the_reference_and_run() {
    let cases = golden_cases();
    let mut failures = Vec::new();
    let mut compared = 0;
    let mut started: BTreeMap<&str, Started> = BTreeMap::new();
    let mut reserved = 0;
    for case in &cases {
        let name = case.name.as_str();
        if std::env::var("GAME_START_ONLY").is_ok_and(|only| only != name) {
            continue;
        }
        if !started.contains_key(name) {
            // The previous game is done with: run it and drop it.
            run_all(&mut started);
            let Some(game) = start(case) else {
                eprintln!("skipped: the game or {name} is not installed");
                continue;
            };
            compared += 1;
            check_report(&game.game, &game.before_ai, &mut failures, name);
            let count = count_reservations(&game.game, &mut failures, name);
            reserved += count;
            if let Ok(dir) = std::env::var("GAME_START_DUMP") {
                for stage in ["before_units", "before_ai", "end"] {
                    let rows = game.rows(stage);
                    let file = format!("{}.{stage}.rs.txt", name.trim_end_matches(".rep"));
                    std::fs::write(PathBuf::from(&dir).join(file), rows.join("\n") + "\n")
                        .expect("write dump");
                }
                eprintln!("{name}: {:#?}", game.game.report);
            }
            started.insert(name, game);
        }
        let Some(run) = started.get(name) else {
            continue;
        };
        let rows = run.rows(&case.stage);
        let ours = summarize(rows);
        for (key, value) in &case.values {
            if key == "start_positions" {
                let found: Vec<String> = start_positions(rows).into_iter().map(|p| p.1).collect();
                let expected: Vec<&str> = value.split(',').filter(|v| !v.is_empty()).collect();
                if expected != found {
                    failures.push(format!("{name} {}: starting positions", case.stage));
                }
                continue;
            }
            let got = ours.iter().find(|(k, _)| k == key).map(|(_, v)| v);
            if got != Some(value) {
                failures.push(format!(
                    "{name} {} {key}: reference {value}, port {got:?}",
                    case.stage
                ));
            }
        }
    }
    run_all(&mut started);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if compared == REPLAY_GAMES + VARIANT_GAMES {
        assert_eq!(reserved, MOVED_UNITS, "starting units with a goal cell");
    }
    eprintln!("compared {compared} game starts");
}

/// Runs logic frames on each started game, then drops it.
fn run_all(started: &mut BTreeMap<&str, Started>) {
    for (name, Started { mut game, .. }) in std::mem::take(started) {
        for _ in 0..FRAMES {
            run_frame(&mut game.sim, &mut NoFrameSeams);
        }
        assert_eq!(game.sim.world.frame, FRAMES, "{name}");
    }
}
