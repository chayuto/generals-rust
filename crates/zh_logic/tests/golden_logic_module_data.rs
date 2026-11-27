//! `ModuleData` of the logic modules with no ported behaviour, against the reference
//! parsers (`tests/cpp_harness/gen_logic_module_data.cpp`): made-up blocks that set every
//! field of every reference table, and digests over the installed game's data.

use std::collections::{BTreeMap, BTreeSet};

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::IniLoadType;
use zh_common::io::file::OsDiskFileHost;
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::modules::module::ModuleType;
use zh_logic::modules::module_data::{DataOnlyModule, FieldList, DATA_ONLY_MODULES};
use zh_logic::object::thing_template::ThingTemplate;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/LogicModuleData.json");

fn rows(kind: &str) -> impl Iterator<Item = &'static str> + '_ {
    VECTORS
        .lines()
        .filter(move |l| l.starts_with('{') && get_str(l, "row") == Some(kind))
}

/// A string member with the generator's escapes undone.
fn text(line: &str, key: &str) -> String {
    let start = line.find(&format!("\"{key}\":\"")).unwrap() + key.len() + 4;
    let mut out = String::new();
    let mut chars = line[start..].chars();
    loop {
        match chars.next().unwrap() {
            '"' => return out,
            '\\' => match chars.next().unwrap() {
                'n' => out.push('\n'),
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
}

fn module(name: &str) -> &'static DataOnlyModule {
    DATA_ONLY_MODULES.iter().find(|m| m.name == name).unwrap()
}

/// The members of one module declaration of `template`.
fn members(
    loader: &GameDataLoader,
    template: &ThingTemplate,
    tag: &str,
    module: &DataOnlyModule,
) -> FieldList {
    let nugget = template
        .get_behavior_module_info()
        .get_nugget_with_tag(tag)
        .unwrap();
    let data = loader.rules.module_data_store.get(nugget.data_id).unwrap();
    (module.fields)(data, loader.rules.global_data.gravity)
        .unwrap_or_else(|| panic!("{}: another data type", module.name))
}

fn synthetic_loader() -> GameDataLoader {
    let mut loader = GameDataLoader::new();
    let preamble = rows("preamble").next().unwrap();
    loader.load_bytes(
        "preamble.ini",
        text(preamble, "text").as_bytes(),
        IniLoadType::Overwrite,
    );
    assert!(loader.report.failures.is_empty());
    loader
}

fn object_name(block: &str) -> &str {
    block.split_whitespace().nth(1).unwrap()
}

#[test]
fn every_field_matches_the_reference() {
    let mut loader = synthetic_loader();
    let mut seen = BTreeMap::new();
    for row in rows("syn") {
        let name = get_str(row, "module").unwrap();
        let block = text(row, "text");
        loader.load_bytes("syn.ini", block.as_bytes(), IniLoadType::Overwrite);
        assert!(
            loader.report.failures.is_empty(),
            "{name}: {:?}\n{block}",
            loader.report.failures
        );
        let template = loader
            .rules
            .thing_factory
            .find_template(object_name(&block))
            .unwrap();
        let fields = members(&loader, template, "ModuleTag_01", module(name));
        let mut sorted = BTreeMap::new();
        for (key, value) in fields {
            assert!(
                sorted.insert(key.clone(), value).is_none(),
                "{name}.{key} twice"
            );
        }
        let ours: Vec<String> = sorted.iter().map(|(k, v)| format!("{k}={v}")).collect();
        let theirs = text(row, "values");
        assert_eq!(ours, theirs.lines().collect::<Vec<_>>(), "{name}\n{block}");
        *seen.entry(name).or_insert(0) += 1;
    }
    assert_eq!(seen.len(), DATA_ONLY_MODULES.len());
    assert!(seen.values().all(|&n| n == 3));
}

/// The reference build only asserts on a field the table lacks (`DEBUG_CRASH`) and goes
/// on without moving a member. The port rejects the block, as its other module parsers
/// do; the installed data has no such key.
#[test]
fn unknown_fields_are_rejected() {
    let mut loader = synthetic_loader();
    let mut count = 0;
    for row in rows("unknown") {
        let name = get_str(row, "module").unwrap();
        assert_eq!(get_u32(row, "accepted"), Some(1), "{name}");
        assert_eq!(get_u32(row, "untouched"), Some(1), "{name}");
        let failed = loader.report.failures.len();
        loader.load_bytes(
            "unknown.ini",
            text(row, "text").as_bytes(),
            IniLoadType::Overwrite,
        );
        assert!(loader.report.failures.len() > failed, "{name}: accepted");
        count += 1;
    }
    assert_eq!(count, 2 * DATA_ONLY_MODULES.len());
}

#[test]
fn registered_as_data_only() {
    let loader = GameDataLoader::new();
    let mut names = BTreeSet::new();
    for module in DATA_ONLY_MODULES {
        assert!(names.insert(module.name), "{} twice", module.name);
        let desc = loader
            .rules
            .module_registry
            .find_descriptor(module.name, ModuleType::Behavior)
            .unwrap();
        assert!(desc.parse_data.is_some(), "{}", module.name);
        // These two also have construction (`golden_s16_dozer_worker`).
        let constructed = ["DozerAIUpdate", "WorkerAIUpdate"].contains(&module.name);
        assert_eq!(desc.create_module.is_some(), constructed, "{}", module.name);
    }
    // The declared data carries the factory entry's masks.
    let mut loader = synthetic_loader();
    for row in rows("syn") {
        let name = get_str(row, "module").unwrap();
        let block = text(row, "text");
        loader.load_bytes("syn.ini", block.as_bytes(), IniLoadType::Overwrite);
        let template = loader
            .rules
            .thing_factory
            .find_template(object_name(&block))
            .unwrap();
        let nugget = &template.get_behavior_module_info().nuggets()[0];
        let data = loader.rules.module_data_store.get(nugget.data_id).unwrap();
        let desc = loader
            .rules
            .module_registry
            .find_descriptor(name, ModuleType::Behavior)
            .unwrap();
        assert_eq!(data.facet_mask(), desc.facet_mask, "{name}");
        assert_eq!(data.update_phase(), desc.update_phase, "{name}");
        assert_eq!(data.is_ai_module(), desc.is_ai_module, "{name}");
        assert_eq!(data.module_name_key(), nugget_name_key(&mut loader, name));
    }
}

fn nugget_name_key(loader: &mut GameDataLoader, name: &str) -> zh_common::types::NameKeyType {
    loader.rules.name_keys.name_to_key(name)
}

fn game_dir() -> Option<String> {
    let dir = std::env::var("ZH_DIR").unwrap_or_else(|_| {
        format!(
            "{}/Games/generalszh",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    std::path::Path::new(&dir)
        .join("INIZH.big")
        .exists()
        .then_some(dir)
}

#[test]
fn retail_module_data_matches_the_reference() {
    let Some(dir) = game_dir() else {
        eprintln!("skipped: the game is not installed");
        return;
    };
    let fs = mount_game_archives(&OsDiskFileHost, &dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    assert!(loader.report.failures.is_empty());
    let unparsed: Vec<_> = loader
        .report
        .stub_modules
        .keys()
        .filter(|k| k.starts_with("logic"))
        .collect();
    assert!(unparsed.is_empty(), "{unparsed:?}");

    let mut lines = Vec::new();
    let mut instances: BTreeMap<&str, u32> = BTreeMap::new();
    for id in 1..=u16::MAX {
        let Some(template) = loader.rules.thing_factory.find_by_template_id(id) else {
            continue;
        };
        for nugget in template.get_behavior_module_info().nuggets() {
            let Some(module) = DATA_ONLY_MODULES.iter().find(|m| m.name == nugget.name) else {
                continue;
            };
            *instances.entry(module.name).or_insert(0) += 1;
            let entry = format!(
                "{}|{}/{}|",
                module.name, template.name_string, nugget.module_tag
            );
            for (key, value) in members(&loader, template, &nugget.module_tag, module) {
                lines.push(format!("{entry}{key}={value}"));
            }
        }
    }
    lines.sort();
    if let Ok(path) = std::env::var("MODULE_DATA_DUMP") {
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
    }

    // FNV-1a 64 over the sorted lines of each (module, field).
    let mut sums: BTreeMap<(&str, &str), (u32, u64)> = BTreeMap::new();
    for line in &lines {
        let (name, rest) = line.split_once('|').unwrap();
        let field = rest.split_once('|').unwrap().1.split_once('=').unwrap().0;
        let sum = sums
            .entry((name, field))
            .or_insert((0, 0xcbf2_9ce4_8422_2325));
        sum.0 += 1;
        for byte in line.bytes().chain(std::iter::once(b'\n')) {
            sum.1 = (sum.1 ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
        }
    }

    let theirs: BTreeMap<&str, u32> = rows("instances")
        .map(|r| (get_str(r, "module").unwrap(), get_u32(r, "count").unwrap()))
        .collect();
    assert_eq!(instances, theirs);
    let mut wrong = Vec::new();
    let mut checked = 0;
    for row in rows("retail") {
        let key = (
            get_str(row, "module").unwrap(),
            get_str(row, "field").unwrap(),
        );
        let want = (
            get_u32(row, "count").unwrap(),
            u64::from_str_radix(get_str(row, "fnv").unwrap(), 16).unwrap(),
        );
        if sums.get(&key) != Some(&want) {
            wrong.push(key);
        }
        checked += 1;
    }
    assert!(wrong.is_empty(), "{} differ: {wrong:?}", wrong.len());
    assert_eq!(checked, sums.len(), "module and field set");
}
