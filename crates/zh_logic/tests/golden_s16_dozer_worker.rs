//! Construction of `DozerAIUpdate` and `WorkerAIUpdate` against the reference
//! (`tests/cpp_harness/gen_s16_dozer_worker.cpp`): the logic random seed after each
//! object is made, the AI module's save bytes and CRC, the mine clearing weapon set
//! flag and the weapon chosen by it.

use std::fmt::Write;

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{ObjectId, TeamId, TRUE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::update::ai_update::{AiUpdate, DerivedAi};
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::WeaponSetType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S16_DozerWorker.json");

const TEMPLATES: [&str; 6] = [
    "S16_Dozer",
    "S16_DozerBare",
    "S16_Worker",
    "S16_WorkerPlain",
    "S16_Unit",
    "S16_DozerStuck",
];
/// Neutral, then the two playing teams.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];

fn text(key: &str) -> String {
    get_str(VECTORS, key).expect("text").replace("\\n", "\n")
}

fn world() -> SimContext {
    let mut sim = SimContext::new();
    for block in text("locomotors").split_inclusive("End\n") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        sim.rules.locomotor_store.add_template(t);
    }
    for block in text("weapons").split_inclusive("End\n") {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(block).expect("weapon");
        sim.rules.weapon_store.add_template(weapon);
    }
    let ini = text("objects");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules.global_data.gravity = -1.0;
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as i32);
        w.player_default_teams.insert(player as i32, team);
    }
    w.team_is_neutral.insert(TEAMS[0], TRUE);
    w.neutral_team_id = TEAMS[0];
    w.local_player_index = 5;
    sim
}

/// `seed,crc,mine flag,weapon,save` of the object just made.
fn observe(sim: &mut SimContext, id: ObjectId, kind: usize) -> String {
    let seed = sim.world.rng.get_game_logic_random_seed_crc();
    let slot = sim.find_object(id).expect("object");
    let record = sim.world.objects.get(slot).expect("record");
    let mine = record
        .weapon_set_flags
        .test(WeaponSetType::MineClearingDetail.as_i32())
        != 0;
    let weapon = record.weapon_set.weapons[0]
        .as_ref()
        .map_or_else(|| "-".to_owned(), |w| w.template_name.clone());
    let module = record.ai_slot.expect("AI module");
    let (save, crc) = sim
        .dispatch_module(module, |m, sim, ctx| {
            let module = m
                .as_any_mut()
                .and_then(|a| a.downcast_mut::<AiUpdate>())
                .expect("AI module");
            let expected = match TEMPLATES[kind] {
                "S16_Unit" => matches!(module.derived, DerivedAi::Base),
                name if name.contains("Dozer") => matches!(module.derived, DerivedAi::Dozer(_)),
                _ => matches!(module.derived, DerivedAi::Worker(_)),
            };
            assert!(expected, "class of {}", TEMPLATES[kind]);
            // The reference library writes the retail save layout.
            let mut save = XferSave::new();
            module
                .xfer_as_version(&mut save, sim, ctx.owner_id, 4)
                .expect("save");
            let mut crc = XferCrc::new();
            module
                .xfer_as_version(&mut crc, sim, ctx.owner_id, 4)
                .expect("crc");
            let mut hex = String::new();
            for byte in save.as_bytes() {
                let _ = write!(hex, "{byte:02x}");
            }
            (hex, crc.get_crc())
        })
        .expect("dispatch");
    format!("{seed:08x},{crc:08x},{},{weapon},{save}", u8::from(mine))
}

/// The layout without `retail-xfer-save`, which the reference library does not
/// write: from the source, version 2 and the previous task before the dock points.
#[test]
fn the_newer_dozer_layout_adds_the_previous_task() {
    let mut sim = world();
    let template = sim
        .rules
        .thing_factory
        .find_template_id(TEMPLATES[0])
        .expect("template");
    let slot = sim.spawn_object(template, TEAMS[1], Coord3D::default(), 0.0);
    let module = sim.world.objects.get(slot).and_then(|o| o.ai_slot);
    let save = sim
        .dispatch_module(module.expect("AI module"), |m, sim, ctx| {
            let module = m
                .as_any_mut()
                .and_then(|a| a.downcast_mut::<AiUpdate>())
                .expect("AI module");
            let mut save = XferSave::new();
            module
                .xfer_as_version(&mut save, sim, ctx.owner_id, 5)
                .expect("save");
            save.as_bytes().to_vec()
        })
        .expect("dispatch");
    assert_eq!(save[0], 2);
    // From the end: the build sub task, nine dock points and their count; before
    // them the current task, the previous task and its info.
    let end = save.len() - 4 - 9 * 13 - 4;
    let mut tasks = vec![0xff; 8];
    tasks.extend([0; 8]);
    assert_eq!(&save[end - 16..end], tasks.as_slice());
}

#[test]
fn dozer_and_worker_construction_matches_the_reference() {
    let lines: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"i\":"))
        .collect();
    assert_eq!(
        lines.len() as u32,
        get_u32(VECTORS, "vector_count").expect("count")
    );
    assert!(lines.len() >= 150);
    let mut sim = world();
    let mut compared = [0u32; TEMPLATES.len()];
    for line in lines {
        let index = get_u32(line, "i").expect("i");
        sim.world.frame = get_u32(line, "frame").expect("frame");
        sim.world
            .rng
            .init_random_with_seed(get_u32(line, "seed").expect("seed"));
        sim.world.next_object_id = 1000;
        let mut made = Vec::new();
        for unit in get_str(line, "units").expect("units").split('#') {
            let mut fields = unit.splitn(3, ',');
            let kind: usize = fields.next().and_then(|f| f.parse().ok()).expect("kind");
            let team: usize = fields.next().and_then(|f| f.parse().ok()).expect("team");
            let want = fields.next().expect("observation");
            let template = sim
                .rules
                .thing_factory
                .find_template_id(TEMPLATES[kind])
                .expect("template");
            let slot = sim.spawn_object(template, TEAMS[1 + team], Coord3D::default(), 0.0);
            let id = sim.world.objects.id_of(slot).expect("id");
            let got = observe(&mut sim, id, kind);
            assert_eq!(got, want, "vector {index}: {}", TEMPLATES[kind]);
            compared[kind] += 1;
            made.push(id);
        }
        for id in made {
            sim.destroy_object(sim.find_object(id).expect("object"));
        }
        sim.process_destroy_list();
    }
    assert!(compared.iter().all(|&n| n > 20), "{compared:?}");
}
