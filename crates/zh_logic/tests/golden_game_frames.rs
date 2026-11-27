//! The first logic frames after game start compared with the running reference
//! engine, for the games of `golden_game_start.rs`, playing the same recorded player
//! commands. `tests/cpp_harness/gen_game_frames.py` recorded
//! `docs/golden_vectors/GameFrames.json`: for each game and field, the frames at
//! which the field's digest changes (counts and digests only).
//!
//! The port does not agree yet, so the test is a ratchet.
//! `game_frames_agreement.txt` holds, per game and field, the first frame at which
//! the port differs. The test fails when a field differs earlier than that; when a
//! field agrees longer it says so, and `GAME_FRAMES_RATCHET=write` rewrites the
//! file from a run, which is how a gain is held.
//!
//! Frame N is the world after N logic frames. The retail test needs what
//! `golden_game_start.rs` needs and passes without comparing when it is missing.
//! `GAME_FRAMES_ONLY=<game.rep>` runs one game; `GAME_FRAMES_DUMP=<dir>` writes the
//! port's rows of every frame, laid out as the harness lays out the reference's.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zh_common::io::file::OsDiskFileHost;
use zh_common::io::recorder::ReplayReader;
use zh_common::math::Coord3D;
use zh_common::types::{Bool, ObjectId, TeamId};
use zh_logic::arena::FacetMask;
use zh_logic::gamelogic::command_target::{dispatch_command, CommandState};
use zh_logic::gamelogic::frame::{run_frame, FrameSeams};
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader, StartupStores};
use zh_logic::gamelogic::game_logic::{start_new_game_observed, GameStart, ReplayGameRules};
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::team::TeamFactory;
use zh_logic::scripts::{GameScripts, ScriptGame};
use zh_logic::sim::SimContext;

// The start comparison uses all of it; this one only part.
#[allow(dead_code)]
#[path = "game_start_support/mod.rs"]
mod support;

use support::{dump, options_parms, percent_decode, replay_parms, retail_dirs};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/GameFrames.json");
/// The games and their options strings.
const STARTS: &str = include_str!("../../../docs/golden_vectors/GameStart.json");
const AGREEMENT: &str = include_str!("game_frames_agreement.txt");

/// Frames recorded.
const FRAMES: u32 = 300;
/// Games: the baseline replays and the option variants.
const GAMES: usize = 24;
/// Variants play the commands of this replay.
const VARIANT_COMMANDS: &str = "macos_2p_vanilla_map.rep";

/// `(field, row kind, tab fields)`, as in the harness. No tab fields takes the row.
type Field = (&'static str, &'static str, &'static [usize]);

const WORLD_FIELDS: [Field; 7] = [
    ("seed", "seed", &[]),
    ("game", "game", &[]),
    ("player", "player", &[]),
    ("energy", "energy", &[]),
    ("team", "team", &[]),
    ("bridge", "bridge", &[]),
    ("deck", "deck", &[]),
];

/// Taken apart by who owns the object ([`GROUPS`]).
const OBJECT_FIELDS: [Field; 10] = [
    ("template", "obj", &[1, 2]),
    ("owner", "obj", &[1, 3, 4, 5]),
    ("position", "obj", &[1, 6, 7, 8]),
    ("angle", "obj", &[1, 9]),
    ("layer", "obj", &[1, 10]),
    ("health", "obj", &[1, 11, 12]),
    ("status", "obj", &[1, 13, 14]),
    ("vision", "obj", &[1, 15, 16]),
    ("wake", "wake", &[]),
    ("ai", "ai", &[]),
];

/// `(group, team name prefix)`: the human and the computer players' starting teams,
/// and the map's objects, which are all the others.
const GROUPS: [(&str, &str); 3] = [
    ("map", ""),
    ("computer", "teamSkirmish"),
    ("human", "teamplayer"),
];

/// The low half of FNV-1a over the rows, each ended by a newline.
fn digest32<'a>(rows: impl Iterator<Item = &'a str>) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for row in rows {
        for &byte in row.as_bytes().iter().chain(b"\n") {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("{:08x}", hash as u32)
}

fn group_of(team: &str) -> &'static str {
    GROUPS
        .iter()
        .find(|(_, prefix)| !prefix.is_empty() && team.starts_with(prefix))
        .map_or("map", |g| g.0)
}

fn pick_digest<'a, 'b: 'a>(
    rows: impl Iterator<Item = &'a Vec<&'b str>>,
    columns: &[usize],
) -> String {
    let picked: Vec<String> = rows
        .map(|cells| {
            if columns.is_empty() {
                cells.join("\t")
            } else {
                let picked: Vec<&str> = columns.iter().map(|&i| cells[i]).collect();
                picked.join("\t")
            }
        })
        .collect();
    digest32(picked.iter().map(String::as_str))
}

/// A frame's value of every field.
fn frame_fields(rows: &[String]) -> BTreeMap<String, String> {
    let mut kinds: BTreeMap<&str, Vec<Vec<&str>>> = BTreeMap::new();
    for row in rows {
        let cells: Vec<&str> = row.split('\t').collect();
        kinds.entry(cells[0]).or_default().push(cells);
    }
    let of = |kind: &str| kinds.get(kind).map_or(&[][..], Vec::as_slice);
    let owners: BTreeMap<&str, &str> = of("obj").iter().map(|c| (c[1], group_of(c[4]))).collect();
    let mut out = BTreeMap::new();
    for (field, kind, columns) in WORLD_FIELDS {
        out.insert(field.to_owned(), pick_digest(of(kind).iter(), columns));
    }
    for (group, _) in GROUPS {
        let count = owners.values().filter(|&&g| g == group).count();
        out.insert(format!("{group}.count"), count.to_string());
        for (field, kind, columns) in OBJECT_FIELDS {
            let rows = of(kind).iter().filter(|c| owners.get(c[1]) == Some(&group));
            out.insert(format!("{group}.{field}"), pick_digest(rows, columns));
        }
    }
    out
}

/// The rows `gen_game_frames.cpp` adds to those of the start comparison.
///
/// `tags` keeps the module tag names: looking a name key up searches every name.
fn frame_rows(game: &GameStart, tags: &mut BTreeMap<i32, String>) -> Vec<String> {
    let sim = &game.sim;
    let mut out = dump(sim, &game.players, &game.teams);
    // The frames keep a team's members in the world; the team factory's lists are
    // filled at game start only.
    for row in out.iter_mut().filter(|r| r.starts_with("team\t")) {
        let Some((head, _)) = row.rsplit_once('\t') else {
            continue;
        };
        let id = head.split('\t').nth(1).and_then(|c| c.parse().ok());
        let members = id.map_or(0, |id| {
            sim.world.team_members.get(&TeamId(id)).map_or(0, Vec::len)
        });
        *row = format!("{head}\t{members}");
    }
    for i in 0..game.players.player_count() {
        let (production, consumption) = sim.world.player_energy.get(&i).map_or((0, 0), |energy| {
            (
                energy.get_production(sim.world.frame),
                energy.get_consumption(),
            )
        });
        out.push(format!("energy\t{i}\t{production}\t{consumption}"));
    }
    let mut objects: Vec<_> = sim
        .world
        .objects
        .world_iter()
        .filter_map(|slot| Some((sim.world.objects.id_of(slot)?, slot)))
        .collect();
    objects.sort_by_key(|&(id, _)| id.0);
    for (id, slot) in objects {
        let Some(object) = sim.world.objects.get(slot) else {
            continue;
        };
        let mut wake = String::new();
        for &(module, facets) in &object.behaviors {
            if !facets.contains(FacetMask::UPDATE) {
                continue;
            }
            let meta = sim.world.modules.meta(module);
            let tag = tags
                .entry(meta.tag_key.0)
                .or_insert_with(|| sim.rules.name_keys.key_to_name(meta.tag_key).to_owned());
            wake.push_str(&format!("{tag}:{},", meta.schedule.next_call_frame()));
        }
        out.push(format!("wake\t{}\t{wake}", id.0));
        if let Some(ai) = object.ai_slot {
            // A kind of AI that is not the ported `AiUpdate` shows no state.
            let state = sim
                .world
                .modules
                .payload(ai)
                .and_then(|m| m.as_any())
                .and_then(|a| a.downcast_ref::<AiUpdate>())
                .map_or_else(
                    || "-".to_owned(),
                    |ai| ai.machine.current_state_id().to_string(),
                );
            out.push(format!("ai\t{}\t{state}", id.0));
        }
    }
    out
}

/// A replay's commands, given to the frames they were recorded on.
struct Playback<'a> {
    commands: ReplayReader<'a>,
    state: CommandState,
}

/// The unported parts of a frame this comparison runs: the commands and the scripts.
struct Frame<'p, 'a, 'g> {
    playback: &'p mut Playback<'a>,
    scripts: &'g mut GameScripts,
    players: &'g mut PlayerList,
    teams: &'g mut TeamFactory,
    stores: &'g StartupStores,
    waypoints: &'g [(String, Coord3D)],
    named_objects: &'g mut Vec<(String, ObjectId)>,
}

impl FrameSeams for Frame<'_, '_, '_> {
    fn script_engine(&mut self, sim: &mut SimContext) {
        let game = ScriptGame {
            players: self.players,
            teams: self.teams,
            stores: self.stores,
            waypoints: self.waypoints,
            named_objects: self.named_objects,
        };
        self.scripts.update(sim, game);
    }

    fn is_time_frozen(&mut self, _sim: &SimContext) -> Bool {
        Bool::from(self.scripts.is_time_frozen())
    }

    fn command_list(&mut self, sim: &mut SimContext) {
        let playback = &mut *self.playback;
        for message in playback.commands.commands_for_frame(sim.world.frame) {
            if message.is_delivered() {
                dispatch_command(sim, &mut playback.state, &message);
            }
        }
    }
}

/// `(game, options string)` of every game, in the order of the start comparison.
fn games() -> Vec<(String, Option<String>)> {
    STARTS
        .lines()
        .filter_map(|line| {
            let line = line.trim().strip_prefix('"')?;
            let mut words = line.split(' ');
            let name = words.next()?.to_owned();
            if words.next()? != "stage=end" {
                return None;
            }
            let options = words
                .find_map(|w| w.strip_prefix("options="))
                .map(percent_decode);
            Some((name, options))
        })
        .collect()
}

/// The reference's value of each `(game, field)` at every frame from 1.
fn reference() -> BTreeMap<(String, String), Vec<String>> {
    let mut out = BTreeMap::new();
    for line in GOLDEN.lines() {
        let Some(line) = line.trim().trim_end_matches(',').strip_prefix('"') else {
            continue;
        };
        let Some(line) = line.strip_suffix('"') else {
            continue;
        };
        let mut words = line.split(' ');
        let (Some(game), Some(field)) = (words.next(), words.next()) else {
            continue;
        };
        let changes: Vec<(u32, &str)> = words
            .map(|w| {
                let (frame, value) = w.split_once(':').expect("frame:value");
                (frame.parse().expect("frame"), value)
            })
            .collect();
        assert_eq!(changes.first().map(|c| c.0), Some(1), "{game} {field}");
        let mut values = Vec::new();
        let mut next = 0;
        let mut value = "";
        for frame in 1..=FRAMES {
            if changes.get(next).is_some_and(|c| c.0 == frame) {
                value = changes[next].1;
                next += 1;
            }
            values.push(value.to_owned());
        }
        assert_eq!(next, changes.len(), "{game} {field}: frames out of order");
        out.insert((game.to_owned(), field.to_owned()), values);
    }
    out
}

/// Every field's name: the world's, then each group's.
fn field_names() -> Vec<String> {
    let mut names: Vec<String> = WORLD_FIELDS.iter().map(|f| f.0.to_owned()).collect();
    for (group, _) in GROUPS {
        names.push(format!("{group}.count"));
        names.extend(OBJECT_FIELDS.iter().map(|f| format!("{group}.{}", f.0)));
    }
    names
}

/// `game field=frame ...`, with `-` for a field that agrees at every frame. Frames
/// past the recording count as `FRAMES + 1`.
fn agreement() -> BTreeMap<(String, String), u32> {
    let mut out = BTreeMap::new();
    for line in AGREEMENT.lines().filter(|l| !l.starts_with('#')) {
        let mut words = line.split_whitespace();
        let Some(game) = words.next() else {
            continue;
        };
        for word in words {
            let (field, frame) = word.split_once('=').expect("field=frame");
            let frame = if frame == "-" {
                FRAMES + 1
            } else {
                frame.parse().expect("frame")
            };
            out.insert((game.to_owned(), field.to_owned()), frame);
        }
    }
    out
}

fn agreement_line(game: &str, first: &BTreeMap<String, u32>) -> String {
    let mut line = game.to_owned();
    for field in field_names() {
        match first[&field] {
            frame if frame > FRAMES => line.push_str(&format!(" {field}=-")),
            frame => line.push_str(&format!(" {field}={frame}")),
        }
    }
    line
}

#[test]
fn golden_and_ratchet_list_every_game_and_field() {
    let games = games();
    assert_eq!(games.len(), GAMES);
    let reference = reference();
    let agreement = agreement();
    let fields = field_names().len();
    assert_eq!(reference.len(), GAMES * fields);
    assert_eq!(agreement.len(), GAMES * fields);
    for (game, _) in &games {
        for field in field_names() {
            let key = (game.clone(), field.clone());
            assert!(reference.contains_key(&key), "{game} {field}");
            assert!(agreement.contains_key(&key), "{game} {field}");
        }
    }
    // The recording is of games that run: every seed moves on.
    for (game, _) in &games {
        let seeds = &reference[&(game.clone(), "seed".to_owned())];
        assert!(seeds.iter().any(|s| s != &seeds[0]), "{game}: seed stands");
    }
}

/// The first frame at which each field of one game differs.
fn compare(
    game: &str,
    options: Option<&str>,
    (game_dir, user_dir): (&str, &str),
    reference: &BTreeMap<(String, String), Vec<String>>,
) -> Option<BTreeMap<String, u32>> {
    let fs = mount_game_archives(&OsDiskFileHost, game_dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let replays = PathBuf::from(user_dir).join("Replays");
    let parms = match options {
        Some(options) => options_parms(&fs, &loader, options, game_dir, user_dir),
        None => replay_parms(&fs, &loader, &replays.join(game), game_dir, user_dir),
    }?;
    let commands = options.map_or(game, |_| VARIANT_COMMANDS);
    let bytes = std::fs::read(replays.join(commands)).ok()?;
    let rules = ReplayGameRules::new(&loader);
    let mut playback = Playback {
        commands: ReplayReader::open(&bytes, &rules).expect("replay"),
        state: CommandState::default(),
    };
    let mut started = start_new_game_observed(loader, parms, |_, _, _, _| {}).expect("start");

    let dumps = std::env::var("GAME_FRAMES_DUMP")
        .ok()
        .map(|dir| PathBuf::from(dir).join(game.trim_end_matches(".rep")));
    if let Some(dir) = &dumps {
        std::fs::create_dir_all(dir).expect("dump directory");
    }
    let mut first: BTreeMap<String, u32> =
        field_names().into_iter().map(|f| (f, FRAMES + 1)).collect();
    let mut tags = BTreeMap::new();
    for frame in 1..=FRAMES {
        let mut seams = Frame {
            playback: &mut playback,
            scripts: &mut started.scripts,
            players: &mut started.players,
            teams: &mut started.teams,
            stores: &started.stores,
            waypoints: &started.waypoints,
            named_objects: &mut started.named_objects,
        };
        run_frame(&mut started.sim, &mut seams);
        assert_eq!(started.sim.world.frame, frame, "{game}");
        let rows = frame_rows(&started, &mut tags);
        if let Some(dir) = &dumps {
            std::fs::write(dir.join(format!("f{frame:04}.txt")), rows.join("\n") + "\n")
                .expect("write dump");
        }
        for (field, value) in frame_fields(&rows) {
            let expected = &reference[&(game.to_owned(), field.clone())][frame as usize - 1];
            if &value != expected {
                let at = first.get_mut(&field).expect("field");
                *at = (*at).min(frame);
            }
        }
    }
    // No order of these frames changes a compared field yet, so say directly that
    // the frames took their commands.
    assert!(
        playback
            .commands
            .next_frame()
            .is_none_or(|next| next >= FRAMES),
        "{game}: commands left untaken"
    );
    // An order naming a key or template id the rules lack would be dropped.
    assert_eq!(
        playback.state.unknown_names,
        [],
        "{game}: orders naming unknown keys or templates"
    );
    if std::env::var("GAME_FRAMES_DUMP").is_ok() {
        eprintln!(
            "{game}: commands not carried out {:?}",
            playback.state.not_carried_out
        );
        eprintln!(
            "{game}: unported conditions {:?} actions {:?} calls {:?}",
            started.scripts.engine.unported_conditions,
            started.scripts.engine.unported_actions,
            started.scripts.unported
        );
    }
    Some(first)
}

#[test]
fn retail_game_frames_match_the_ratchet() {
    let Some((game_dir, user_dir)) = retail_dirs() else {
        eprintln!("skipped: the game is not installed");
        return;
    };
    let reference = reference();
    let agreement = agreement();
    let only = std::env::var("GAME_FRAMES_ONLY").ok();
    let mut worse = Vec::new();
    let mut better = Vec::new();
    let mut lines = Vec::new();
    for (game, options) in games() {
        if only.as_ref().is_some_and(|o| o != &game) {
            continue;
        }
        let dirs = (game_dir.as_str(), user_dir.as_str());
        let Some(first) = compare(&game, options.as_deref(), dirs, &reference) else {
            eprintln!("skipped: {game} is not installed");
            continue;
        };
        for field in field_names() {
            let Some(&expected) = agreement.get(&(game.clone(), field.clone())) else {
                worse.push(format!("{game} {field}: not in the ratchet"));
                continue;
            };
            let found = first[&field];
            let report = format!("{game} {field}: first differs at {found}, ratchet {expected}");
            if found < expected {
                worse.push(report);
            } else if found > expected {
                better.push(report);
            }
        }
        lines.push(agreement_line(&game, &first));
    }
    eprintln!("compared {} games", lines.len());
    if std::env::var("GAME_FRAMES_RATCHET").is_ok_and(|v| v == "write") {
        assert!(
            lines.len() == GAMES,
            "a partial run cannot rewrite the file"
        );
        let head = "# Per game and field, the first frame at which the port differs from\n\
                    # GameFrames.json (`-`: none). See golden_game_frames.rs.\n";
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/game_frames_agreement.txt"
        );
        std::fs::write(path, format!("{head}{}\n", lines.join("\n"))).expect("write ratchet");
        return;
    }
    if !better.is_empty() {
        eprintln!(
            "agreement improved; record it with GAME_FRAMES_RATCHET=write:\n{}",
            better.join("\n")
        );
    }
    assert!(
        worse.is_empty(),
        "agreement got worse:\n{}",
        worse.join("\n")
    );
}
