//! The data loader hands out the reference's `NameKeyType` numbers.
//! `tests/cpp_harness/gen_name_keys.py` read the running reference engine's table
//! after its start and recorded `docs/golden_vectors/NameKeys.json`: the number
//! of keys and a digest of the names of every run of keys, and the same for the
//! thing templates by id (counts and digests only). A replay names upgrades and
//! sciences by key and templates by id.
//!
//! The retail test passes without comparing when the game data is missing; the
//! window and module names need none.

#[allow(dead_code)]
#[path = "game_start_support/mod.rs"]
mod support;

use zh_common::ini::IniLoadType;
use zh_common::io::big_file_system::BigFileSystem;
use zh_common::io::file::OsDiskFileHost;
use zh_common::types::NameKeyType;
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/NameKeys.json");

/// The strings of the `cases` array, one per line.
fn cases() -> Vec<String> {
    GOLDEN
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            Some(line.strip_prefix('"')?.strip_suffix('"')?.to_owned())
        })
        .collect()
}

const FNV_SEED: u64 = 0xCBF2_9CE4_8422_2325;

fn fnv(value: u64, bytes: impl Iterator<Item = u8>) -> u64 {
    bytes.fold(value, |v, b| {
        (v ^ u64::from(b)).wrapping_mul(0x100_0000_01B3)
    })
}

/// FNV-1a over the names of keys `first..=last`, each ended by a newline.
fn digest(loader: &GameDataLoader, first: i32, last: i32) -> String {
    let mut value = FNV_SEED;
    for key in first..=last {
        let name = loader.rules.name_keys.key_to_name(NameKeyType(key));
        value = fnv(value, name.bytes().chain([b'\n']));
    }
    format!("{value:016x}")
}

fn range(text: &str) -> (i32, i32) {
    let (first, last) = text.split_once('-').expect("range");
    (first.parse().unwrap(), last.parse().unwrap())
}

#[test]
fn window_and_module_names_without_game_data() {
    let mut loader = GameDataLoader::new();
    loader.load_startup(&BigFileSystem::new());
    let row = cases()
        .into_iter()
        .find(|c| c.starts_with("fixed "))
        .expect("fixed row");
    let words: Vec<&str> = row.split(' ').collect();
    let (first, last) = range(words[1]);
    // The science names come first in the reference and need the game data.
    let sciences = first - 1;
    assert_eq!(digest(&loader, first - sciences, last - sciences), words[2]);
}

#[test]
fn every_key_of_the_engine_start() {
    let Some((game_dir, _)) = support::retail_dirs() else {
        eprintln!("skipped: no retail data");
        return;
    };
    let host = OsDiskFileHost;
    let mut loader = GameDataLoader::new();
    loader.load_startup(&mount_game_archives(&host, &game_dir));
    let mut count = 0;
    let mut checked = 0;
    for row in cases() {
        let words: Vec<&str> = row.split(' ').collect();
        match words[0] {
            "count" => count = words[1].parse().unwrap(),
            "chunk" => {
                let (first, last) = range(words[1]);
                assert_eq!(digest(&loader, first, last), words[2], "keys {}", words[1]);
                checked = last;
            }
            _ => {}
        }
    }
    assert_eq!((checked, count), (4234, 4234));
    // A replay's build orders name templates by id.
    let templates = &loader.rules.thing_factory;
    let (mut count_of_templates, mut checked_ids) = (0, 0);
    for row in cases() {
        let words: Vec<&str> = row.split(' ').collect();
        match words[0] {
            "templates" => count_of_templates = words[1].parse().unwrap(),
            "ids" => {
                let (first, last) = range(words[1]);
                let mut value = FNV_SEED;
                for id in first..=last {
                    let template = templates.find_by_template_id(id as u16);
                    let name = template.map_or("", |t| t.name_string.as_str());
                    value = fnv(value, format!("{id}\t{name}").bytes().chain([b'\n']));
                }
                assert_eq!(
                    format!("{value:016x}"),
                    words[2],
                    "template ids {}",
                    words[1]
                );
                checked_ids = last;
            }
            _ => {}
        }
    }
    assert!(count_of_templates > 0);
    assert_eq!(checked_ids, count_of_templates);
    assert!(templates
        .find_by_template_id(count_of_templates as u16 + 1)
        .is_none());
    // The next name takes the next number: nothing was interned beyond these.
    assert_eq!(
        loader.rules.name_keys.name_to_key("no such name").0,
        count + 1
    );
}

/// A module tag takes its number after the names its draw data takes
/// (`parseBoneNameKey` takes none for `none`, a transition state takes both), and
/// a reference to a missing list takes one ahead of the tag.
#[test]
fn module_data_takes_numbers_before_its_tag() {
    let ini = [
        "Object Thing",
        "  Draw = W3DModelDraw DrawTag",
        "    DefaultConditionState",
        "      Turret = NONE",
        "      TurretPitch = BoneA",
        "      TransitionKey = None",
        "    End",
        "    TransitionState = One Two",
        "    End",
        "  End",
        "  Behavior = SlowDeathBehavior DieTag",
        "    FX = INITIAL MissingList None",
        "  End",
        "End",
        "",
    ]
    .join("\n");
    let mut loader = GameDataLoader::new();
    loader.load_bytes("keys.ini", ini.as_bytes(), IniLoadType::Overwrite);
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    let keys = &loader.rules.name_keys;
    let first = keys.lookup_key("bonea").expect("bone").0;
    let order = [
        "bonea",
        "none",
        "one",
        "two",
        "DrawTag",
        "MissingList",
        "DieTag",
    ];
    for (offset, name) in (0..).zip(order) {
        let key = keys.lookup_key(name).map(|k| k.0);
        assert_eq!(key, Some(first + offset), "{name}");
    }
}
