//! `GameLogic::startNewGame` compared with the running reference engine: for each of
//! the baseline replays, the players, teams and objects at two stages of the start.
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

use std::collections::BTreeSet;
use std::path::PathBuf;

use zh_common::io::file::OsDiskFileHost;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::gamelogic::game_logic::{start_new_game_observed, GameStart};

#[path = "game_start_support/mod.rs"]
mod support;

use support::{dump, replay_parms, retail_dirs};

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

const KNOWN_AT_END: [&str; 11] = [
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
    out
}

/// One golden line per dump: `replay stage=<stage> key=value ...`.
fn golden_cases() -> Vec<(String, String, Vec<(String, String)>)> {
    GOLDEN
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',').strip_prefix('"')?;
            let line = line.strip_suffix('"')?;
            let mut words = line.split(' ');
            let replay = words.next()?.to_owned();
            let stage = words.next()?.strip_prefix("stage=")?.to_owned();
            let values = words
                .filter_map(|w| w.split_once('='))
                .map(|(k, v)| (k.to_owned(), v.to_owned()))
                .collect();
            Some((replay, stage, values))
        })
        .collect()
}

#[test]
fn golden_lists_every_replay_at_both_stages() {
    let cases = golden_cases();
    assert_eq!(cases.len(), 12);
    for (replay, _, values) in cases {
        assert_eq!(values.len(), 26, "{replay}");
    }
}

/// The started game, with its rows before the player maps and at the end.
fn start(replay: &str) -> Option<(GameStart, Vec<String>, Vec<String>)> {
    let (game_dir, user_dir) = retail_dirs()?;
    let fs = mount_game_archives(&OsDiskFileHost, &game_dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let path = PathBuf::from(&user_dir).join("Replays").join(replay);
    let parms = replay_parms(&fs, &loader, &path, &game_dir, &user_dir)?;
    let mut before = Vec::new();
    let game = start_new_game_observed(loader, parms, |sim, players, teams| {
        before = dump(sim, players, teams);
    })
    .expect("start");
    let end = dump(&game.sim, &game.players, &game.teams);
    Some((game, before, end))
}

#[test]
fn retail_game_starts_match_the_reference_and_run() {
    let cases = golden_cases();
    let mut replays: Vec<&str> = cases.iter().map(|c| c.0.as_str()).collect();
    replays.dedup();
    let mut failures = Vec::new();
    // `(stage, key)` of every digest that differed on some replay.
    let mut differing: BTreeSet<(&str, &str)> = BTreeSet::new();
    let mut compared = 0;
    for replay in replays {
        let Some((mut game, before, end)) = start(replay) else {
            eprintln!("skipped: the game or {replay} is not installed");
            continue;
        };
        compared += 1;
        if let Ok(dir) = std::env::var("GAME_START_DUMP") {
            for (stage, rows) in [("before_ai", &before), ("end", &end)] {
                let name = format!("{}.{stage}.rs.txt", replay.trim_end_matches(".rep"));
                std::fs::write(PathBuf::from(&dir).join(name), rows.join("\n") + "\n")
                    .expect("write dump");
            }
            eprintln!("{replay}: {:#?}", game.report);
        }
        for (_, stage, values) in cases.iter().filter(|c| c.0 == replay) {
            let ours = summarize(if stage == "end" { &end } else { &before });
            for (key, value) in values {
                let got = ours.iter().find(|(k, _)| k == key).map(|(_, v)| v);
                if got == Some(value) {
                    continue;
                }
                differing.insert((stage.as_str(), key.as_str()));
                if known_divergence(stage, key).is_none() {
                    failures.push(format!(
                        "{replay} {stage} {key}: reference {value}, port {got:?}"
                    ));
                }
            }
        }
        for _ in 0..FRAMES {
            run_frame(&mut game.sim, &mut NoFrameSeams);
        }
        assert_eq!(game.sim.world.frame, FRAMES, "{replay}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if compared == 6 {
        let known = KNOWN_AT_BOTH
            .iter()
            .flat_map(|&(k, _)| [("before_ai", k), ("end", k)])
            .chain(KNOWN_AT_END.iter().map(|&k| ("end", k)));
        for entry in known {
            assert!(
                differing.contains(&entry),
                "{entry:?} matches now: take it out of the known divergences"
            );
        }
    }
    eprintln!("compared {compared} game starts");
}
