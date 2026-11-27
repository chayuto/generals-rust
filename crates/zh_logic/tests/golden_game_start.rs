//! `GameLogic::startNewGame` compared with the running reference engine: for each of
//! the baseline replays, and for games on other retail maps described by an options
//! string alone, the players, teams and objects at two stages of the start.
//! `tests/cpp_harness/gen_game_start.py` recorded `docs/golden_vectors/GameStart.json`
//! (counts and digests only: the rows name map content).
//!
//! Every digest is compared. The ones the port is known to miss are listed in
//! [`known_divergence`] with their cause, and must still differ: a fix has to take
//! its entry out.
//!
//! The retail tests need the game (`ZH_DIR`, default `$HOME/Games/generalszh`) and the
//! replays with their user maps (`ZH_USER_DIR`, default
//! `$HOME/Library/Application Support/GeneralsX/GeneralsZH`); without them they pass
//! without comparing. `GAME_START_DUMP=<dir>` writes the port's rows for a diff
//! against the reference's.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use zh_common::io::file::OsDiskFileHost;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::gamelogic::game_logic::{start_new_game_observed, GameStart};
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::sim::SimView;

#[path = "game_start_support/mod.rs"]
mod support;

use support::{dump, options_parms, percent_decode, replay_parms, retail_dirs};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/GameStart.json");

/// Logic frames each started game runs.
const FRAMES: u32 = 300;

/// Why a digest differs from the reference, if it is known to. `before_ai`:
///
/// - the seed: `DozerAIUpdate` and `WorkerAIUpdate` are not ported, so a starting
///   dozer or worker has no AI module and draws no random numbers;
/// - the starting objects' positions: `Pathfinder::adjustDestination` is not
///   ported, so a starting dozer stays where `findPositionAround` put it.
///
/// `end` adds `AIPlayer::newMap`, not ported: in the reference it destroys the
/// computer player's command center and what stands under the new one (which sets
/// their destroyed bit) and builds it again, which changes the next object id and
/// that team's member count.
fn known_divergence(stage: &str, key: &str) -> Option<&'static str> {
    KNOWN_AT_BOTH
        .iter()
        .find(|(k, _)| *k == key)
        .map(|&(_, cause)| cause)
        .or_else(|| {
            (stage == "end" && KNOWN_AT_END.contains(&key)).then_some("unported AIPlayer::newMap")
        })
}

const KNOWN_AT_BOTH: [(&str, &str); 2] = [
    ("seed_digest", "unported dozer and worker AI"),
    ("start_position_digest", "unported adjustDestination"),
];

const KNOWN_AT_END: [&str; 12] = [
    // `AIPlayer::newMap` clears map objects from under the new buildings.
    "unbridged_digest",
    "game_digest",
    "team_digest",
    "map_status_digest",
    "start_count",
    "start_template_digest",
    "start_team_digest",
    "start_angle_digest",
    "start_layer_digest",
    "start_health_digest",
    "start_status_digest",
    "start_vision_digest",
];

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
    for kind in ["seed", "game", "player", "team"] {
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
    // What can still be compared on a map with bridges, whose objects shift every id:
    // the map's other objects, by content.
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

/// Replays of the baseline, and games made from options strings.
const REPLAY_GAMES: usize = 6;
const VARIANT_GAMES: usize = 16;
const BRIDGED_GAMES: u32 = 4;

#[test]
fn golden_lists_every_game_at_both_stages() {
    let cases = golden_cases();
    assert_eq!(cases.len(), 2 * (REPLAY_GAMES + VARIANT_GAMES));
    assert_eq!(
        cases.iter().filter(|c| c.options.is_some()).count(),
        2 * VARIANT_GAMES
    );
    for case in cases {
        assert_eq!(case.values.len(), 29, "{}", case.name);
    }
}

/// The started game, with its rows before the player maps and at the end.
fn start(case: &Case) -> Option<(GameStart, Vec<String>, Vec<String>)> {
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
    let mut before = Vec::new();
    let game = start_new_game_observed(loader, parms, |sim, players, teams| {
        before = dump(sim, players, teams);
    })
    .expect("start");
    let end = dump(&game.sim, &game.players, &game.teams);
    Some((game, before, end))
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

/// Whether the object with `id` cannot move, so `adjustDestination` leaves it alone.
fn is_immobile(game: &GameStart, id: i32) -> bool {
    game.sim
        .world
        .objects
        .slot_by_id(zh_common::types::ObjectId(id))
        .is_some_and(|slot| game.sim.object_is_kind_of(slot, KindOfType::Immobile) != 0)
}

/// Every map object is made or counted as skipped, and the calls that were left
/// out are counted once each.
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
    if report.created_from_map + skipped != report.map_objects
        || report.created_from_map as usize != map_rows
    {
        failures.push(format!(
            "{name}: {} map objects, {} made ({map_rows} listed), {skipped} skipped",
            report.map_objects, report.created_from_map
        ));
    }
    let count = |call: &str| report.unported.get(call).copied().unwrap_or(0);
    let mobile = start_positions(before)
        .iter()
        .filter(|(id, _)| !is_immobile(game, *id))
        .count() as u32;
    let computers = count("AIPlayer");
    if count("Pathfinder::adjustDestination") != mobile
        || count("AIPlayer::newMap") != computers
        || count("ScriptEngine") != 1
    {
        failures.push(format!(
            "{name}: unported calls {:?}, {mobile} mobile starting units",
            report.unported
        ));
    }
}

#[test]
fn retail_game_starts_match_the_reference_and_run() {
    let cases = golden_cases();
    let mut failures = Vec::new();
    // `(stage, key)` of every digest that differed in some game.
    let mut differing: BTreeSet<(String, String)> = BTreeSet::new();
    let mut compared = 0;
    let mut bridged_games = 0;
    let mut started: BTreeMap<&str, (GameStart, Vec<String>, Vec<String>)> = BTreeMap::new();
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
            check_report(&game.0, &game.1, &mut failures, name);
            if let Ok(dir) = std::env::var("GAME_START_DUMP") {
                for (stage, rows) in [("before_ai", &game.1), ("end", &game.2)] {
                    let file = format!("{}.{stage}.rs.txt", name.trim_end_matches(".rep"));
                    std::fs::write(PathBuf::from(&dir).join(file), rows.join("\n") + "\n")
                        .expect("write dump");
                }
                eprintln!("{name}: {:#?}", game.0.report);
            }
            started.insert(name, game);
        }
        let Some((game, before, end)) = started.get(name) else {
            continue;
        };
        let rows = if case.stage == "end" { end } else { before };
        let ours = summarize(rows);
        // The port makes no bridge objects, so on such a map only the players and the
        // other map objects, by content, can be compared.
        let bridged = ["map bridge", "TerrainLogic::addLandmarkBridgeToLogic"]
            .iter()
            .any(|call| game.report.unported.contains_key(call));
        bridged_games += u32::from(bridged && case.stage == "end");
        for (key, value) in &case.values {
            if bridged && !(key.starts_with("player_") || key.starts_with("unbridged_")) {
                continue;
            }
            if key == "start_positions" {
                let found = start_positions(rows);
                let expected: Vec<&str> = value.split(',').filter(|v| !v.is_empty()).collect();
                // A mobile unit may have been moved; everything else is exact.
                for (i, (id, got)) in found.iter().enumerate() {
                    if expected.get(i) != Some(&got.as_str()) && is_immobile(game, *id) {
                        failures.push(format!("{name} {}: position of object {id}", case.stage));
                    }
                }
                continue;
            }
            let got = ours.iter().find(|(k, _)| k == key).map(|(_, v)| v);
            if got == Some(value) {
                continue;
            }
            differing.insert((case.stage.clone(), key.clone()));
            if known_divergence(&case.stage, key).is_none() {
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
        assert_eq!(bridged_games, BRIDGED_GAMES, "maps with bridges");
        let known = KNOWN_AT_BOTH
            .iter()
            .flat_map(|&(k, _)| [("before_ai", k), ("end", k)])
            .chain(KNOWN_AT_END.iter().map(|&k| ("end", k)));
        for (stage, key) in known {
            assert!(
                differing.contains(&(stage.to_owned(), key.to_owned())),
                "{stage} {key} matches now: take it out of the known divergences"
            );
        }
    }
    eprintln!("compared {compared} game starts, {bridged_games} on maps with bridges");
}

/// Runs logic frames on each started game, then drops it.
fn run_all(started: &mut BTreeMap<&str, (GameStart, Vec<String>, Vec<String>)>) {
    for (name, (mut game, _, _)) in std::mem::take(started) {
        for _ in 0..FRAMES {
            run_frame(&mut game.sim, &mut NoFrameSeams);
        }
        assert_eq!(game.sim.world.frame, FRAMES, "{name}");
    }
}
