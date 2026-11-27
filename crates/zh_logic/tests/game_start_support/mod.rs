//! Shared by `golden_game_start.rs`: finding the installed game and replays, reading
//! a replay's map, and writing a started game as `gen_game_start.cpp` writes the
//! reference's.

use std::path::{Path, PathBuf};

use zh_common::io::archive_file_system::ArchiveFilter;
use zh_common::io::big_file_system::BigFileSystem;
use zh_common::io::recorder::{read_replay_header, GameOptions};
use zh_common::types::Relationship;
use zh_logic::gamelogic::game_engine::GameDataLoader;
use zh_logic::gamelogic::game_logic::{MapSource, ReplayGameRules, StartParms};
use zh_logic::player::player_list::PlayerList;
use zh_logic::player::team::TeamFactory;
use zh_logic::sim::{SimContext, SimView};

/// `(game directory, user data directory)`, when both exist.
pub fn retail_dirs() -> Option<(String, String)> {
    let home = std::env::var("HOME").ok()?;
    let game = std::env::var("ZH_DIR").unwrap_or_else(|_| format!("{home}/Games/generalszh"));
    let user = std::env::var("ZH_USER_DIR")
        .unwrap_or_else(|_| format!("{home}/Library/Application Support/GeneralsX/GeneralsZH"));
    (Path::new(&game).join("INIZH.big").is_file() && Path::new(&user).is_dir())
        .then_some((game, user))
}

pub fn percent_decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Some(value) = std::str::from_utf8(&bytes[i + 1..i + 3])
                .ok()
                .and_then(|h| u8::from_str_radix(h, 16).ok())
            {
                out.push(value);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// A file of `dir` by name, ignoring case.
fn find_ignoring_case(dir: &Path, name: &str) -> Option<PathBuf> {
    std::fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .find(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.eq_ignore_ascii_case(name))
        })
}

/// The map of the options string's portable path (`maps/<name>` or
/// `userdata/maps/<name>`), with the INI files beside it.
fn map_source(fs: &BigFileSystem, portable: &[u8], user_dir: &str) -> Option<MapSource> {
    let portable = percent_decode(&String::from_utf8_lossy(portable)).replace('\\', "/");
    let mut parts: Vec<&str> = portable.split('/').filter(|p| !p.is_empty()).collect();
    if parts
        .last()
        .is_some_and(|p| p.to_ascii_lowercase().ends_with(".map"))
    {
        parts.pop();
    }
    let name = *parts.last()?;
    let user = parts.first()?.eq_ignore_ascii_case("userdata");
    let mut source = MapSource::default();
    if user {
        let maps = find_ignoring_case(Path::new(user_dir), "Maps")?;
        let dir = find_ignoring_case(&maps, name)?;
        let file = find_ignoring_case(&dir, &format!("{name}.map"))?;
        source.path = file.to_string_lossy().into_owned();
        source.bytes = std::fs::read(&file).ok()?;
        for leaf in ["map.ini", "solo.ini"] {
            if let Some(path) = find_ignoring_case(&dir, leaf) {
                source
                    .inis
                    .push((leaf.to_owned(), std::fs::read(path).ok()?));
            }
        }
    } else {
        let read = |path: &str| {
            fs.open_file(path, 0, ArchiveFilter::All)
                .map(|f| f.as_bytes().to_vec())
        };
        source.path = format!("Maps\\{name}\\{name}.map");
        source.bytes = read(&source.path)?;
        for leaf in ["map.ini", "solo.ini"] {
            let path = format!("Maps\\{name}\\{leaf}");
            if let Some(bytes) = read(&path) {
                source.inis.push((path, bytes));
            }
        }
    }
    Some(source)
}

fn parms_for(
    fs: &BigFileSystem,
    options: GameOptions,
    game_dir: &str,
    user_dir: &str,
) -> Option<StartParms> {
    // A loose file in the game directory is read before the archives.
    let read = |path: &str| {
        std::fs::read(Path::new(game_dir).join(path.replace('\\', "/")))
            .ok()
            .or_else(|| {
                fs.open_file(path, 0, ArchiveFilter::All)
                    .map(|f| f.as_bytes().to_vec())
            })
    };
    let map = map_source(fs, &options.map, user_dir)?;
    Some(StartParms {
        map,
        seed: options.seed as u32,
        game: Some(options),
        skirmish_scripts: read("Data\\Scripts\\SkirmishScripts.scb"),
        challenge_mode_ini: read("Data\\INI\\ChallengeMode.ini").unwrap_or_default(),
    })
}

/// What a playback of `replay` starts.
pub fn replay_parms(
    fs: &BigFileSystem,
    loader: &GameDataLoader,
    replay: &Path,
    game_dir: &str,
    user_dir: &str,
) -> Option<StartParms> {
    let bytes = std::fs::read(replay).ok()?;
    let rules = ReplayGameRules::new(loader);
    let (_, options) = read_replay_header(&bytes, &rules).expect("replay header");
    parms_for(fs, options, game_dir, user_dir)
}

/// The game of an options string, with the first slot local.
pub fn options_parms(
    fs: &BigFileSystem,
    loader: &GameDataLoader,
    options: &str,
    game_dir: &str,
    user_dir: &str,
) -> Option<StartParms> {
    let rules = ReplayGameRules::new(loader);
    let mut options = GameOptions::parse(options.as_bytes(), &rules).expect("options string");
    options.local_slot = 0;
    parms_for(fs, options, game_dir, user_dir)
}

fn bits(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

fn flags(count: i32, set: impl Fn(i32) -> bool) -> String {
    (0..count)
        .filter(|&i| set(i))
        .map(|i| format!("{i},"))
        .collect()
}

/// The rows `gen_game_start.cpp` writes, objects in id order.
pub fn dump(sim: &SimContext, players: &PlayerList, teams: &TeamFactory) -> Vec<String> {
    let mut out = vec![
        format!(
            "seed\t{}\t{:08x}",
            sim.world.rng.get_game_logic_random_seed(),
            sim.world.rng.get_game_logic_random_seed_crc()
        ),
        format!("game\t{}\t{}", sim.world.frame, sim.world.next_object_id),
    ];
    for i in 0..players.player_count() {
        let Some(p) = players.player(i) else {
            continue;
        };
        let (mut enemies, mut allies) = (0u32, 0u32);
        for j in 0..players.player_count() {
            let Some(team) = players.player(j).and_then(|o| o.default_team) else {
                continue;
            };
            match p.relationship(Some(team), Some(j)) {
                Relationship::Enemies => enemies |= 1 << j,
                Relationship::Allies => allies |= 1 << j,
                Relationship::Neutral => {}
            }
        }
        out.push(format!(
            "player\t{i}\t{}\t{}\t{}\t{}\t{enemies:04x}\t{allies:04x}",
            sim.rules.name_keys.key_to_name(p.name_key),
            p.side,
            sim.world.player_money.get(&i).copied().unwrap_or(0),
            p.default_team.map_or(0, |t| t.0),
        ));
        for &prototype in &p.team_prototypes {
            let Some(proto) = teams.prototype(prototype) else {
                continue;
            };
            for team in &proto.instances {
                out.push(format!(
                    "team\t{}\t{}\t{}\t{i}\t{}\t{}\t{}",
                    team.id.0,
                    prototype,
                    proto.name,
                    u8::from(proto.singleton),
                    u8::from(team.active),
                    team.members.len(),
                ));
            }
        }
    }
    let mut objects: Vec<_> = sim
        .world
        .objects
        .world_iter()
        .filter_map(|slot| Some((sim.world.objects.id_of(slot)?, slot)))
        .collect();
    objects.sort_by_key(|&(id, _)| id.0);
    for (id, slot) in objects {
        let Some(o) = sim.world.objects.get(slot) else {
            continue;
        };
        let template = o
            .thing
            .template_override()
            .get_final(&sim.rules.thing_factory)
            .map_or("", |t| t.get_name());
        let team = o.team_id;
        let team_name = teams.prototype_of(team).map_or("", |p| p.name.as_str());
        let player = teams.controlling_player(team).unwrap_or(-1);
        let pos = o.thing.get_position();
        let (health, max_health) = if o.body_slot.is_some() {
            (sim.object_health(slot), sim.object_max_health(slot))
        } else {
            (0.0, 0.0)
        };
        let status = flags(
            zh_logic::rules::object_status_types::OBJECT_STATUS_COUNT as i32,
            |i| o.status.test(i) != 0,
        );
        let disabled = flags(
            zh_logic::rules::disabled_types::DISABLED_COUNT as i32,
            |i| o.disabled_mask.test(i) != 0,
        );
        out.push(format!(
            "obj\t{}\t{template}\t{}\t{team_name}\t{player}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{status}\t{disabled}\t{}\t{}",
            id.0,
            team.0,
            bits(pos.x),
            bits(pos.y),
            bits(pos.z),
            bits(o.thing.get_orientation()),
            o.layer as i32,
            bits(health),
            bits(max_health),
            bits(o.vision_range),
            bits(o.shroud_clearing_range),
        ));
    }
    out
}
