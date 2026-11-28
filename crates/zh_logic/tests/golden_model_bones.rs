//! The draw module bone data and queries against the reference
//! (`tests/cpp_harness/gen_model_bones.cpp`). Rows carry their own INI.

use std::collections::BTreeMap;
use std::sync::Arc;

use zh_common::golden_json::{decode_hex_bytes, get_str};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::NameKeyType;
use zh_logic::combat::turret_ai::WhichTurretType;
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::modules::update::dock_update::{DockUpdateModuleData, DockUpdateState};
use zh_logic::object::model_bones::{AttachBoneOffsets, DrawableBones, ModelBoneCatalog};
use zh_logic::object::model_draw::{ModelDrawModuleData, ModelDrawSpec};
use zh_logic::rules::model_state::ModelConditionFlags;
use zh_logic::object::weapon_fire::launch_transform;
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_common::types::TeamId;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimWorld};

const JSON: &str = include_str!("../../../docs/golden_vectors/ModelBones.json");

fn rows(kind: &str) -> Vec<&'static str> {
    JSON.lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| get_str(l, "case") == Some(kind))
        .collect()
}

fn new_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    let RulesRegistry {
        module_registry,
        name_keys,
        ..
    } = &mut rules;
    module_registry.register_w3d_draw_stubs(name_keys);
    rules
}

fn load_objects(rules: &mut RulesRegistry, ini: &[u8]) {
    let mut reader = IniReader::new("bones.ini", ini, IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("object ini");
}

/// The `n`th draw module of a template.
fn draw_spec<'a>(rules: &'a RulesRegistry, name: &str, n: usize) -> &'a ModelDrawSpec {
    let template = rules.thing_factory.find_template(name).expect(name);
    let nugget = &template.get_draw_module_info().nuggets()[n];
    let data = rules
        .module_data_store
        .get(nugget.data_id)
        .expect("module data");
    &data
        .as_any()
        .downcast_ref::<ModelDrawModuleData>()
        .expect("model draw data")
        .spec
}

fn bits_text(flags: &ModelConditionFlags) -> String {
    (0..flags.size())
        .filter(|&i| flags.test(i) != 0)
        .map(|i| i.to_string())
        .collect::<Vec<_>>()
        .join(".")
}

fn flags_of(bits: &str) -> ModelConditionFlags {
    let mut flags = ModelConditionFlags::default();
    for bit in bits.split_terminator('.') {
        flags.set(bit.parse().unwrap(), 1); // determinism-ok: oracle row
    }
    flags
}

fn number(text: &str) -> i32 {
    text.parse().unwrap() // determinism-ok: oracle row
}

fn words(text: &str) -> Vec<f32> {
    (0..text.len() / 8)
        .map(|i| f32::from_bits(u32::from_str_radix(&text[i * 8..i * 8 + 8], 16).unwrap()))
        .collect()
}

fn matrix_text(m: &Matrix3D) -> String {
    m.rows
        .iter()
        .flatten()
        .map(|v| format!("{:08x}", v.to_bits()))
        .collect()
}

fn pos_text(p: &Coord3D) -> String {
    [p.x, p.y, p.z]
        .iter()
        .map(|v| format!("{:08x}", v.to_bits()))
        .collect()
}

fn list_text(list: &[String]) -> String {
    list.iter().map(|s| format!("{s},")).collect()
}

fn dump_spec(d: &ModelDrawSpec) -> String {
    let mut out = format!("extra={}", list_text(&d.extra_public_bones));
    out += &format!(";ignore={}", bits_text(&d.ignore_condition_states));
    out += &format!(";default={}", d.default_state.map_or(-1, |i| i as i32));
    out += &format!(";attach={}", d.attach_to_drawable_bone);
    for s in &d.condition_states {
        out += "#c=";
        for f in &s.conditions_yes {
            out += &format!("{}|", bits_text(f));
        }
        out += &format!(";m={}", s.model_name);
        out += &format!(";a={}", s.first_animation.as_deref().unwrap_or(""));
        out += &format!(";f={}", u32::from(s.pristine_bones_in_final_frame()));
        out += &format!(";p={}", list_text(&s.public_bones));
        out += ";w=";
        for w in &s.weapons {
            out += &format!(
                "{}:{}:{}:{}/",
                w.fire_fx, w.recoil, w.muzzle_flash, w.launch
            );
        }
        out += ";t=";
        for t in &s.turrets {
            out += &format!(
                "{}:{}:{:08x}:{:08x}/",
                t.angle_bone,
                t.pitch_bone,
                t.art_angle.to_bits(),
                t.art_pitch.to_bits()
            );
        }
    }
    out
}

#[test]
fn draw_module_data_and_best_states_match_the_reference() {
    let specs = rows("spec");
    let best = rows("best");
    assert_eq!(specs.len(), 80);
    assert_eq!(best.len(), 80);
    let mut rules = new_rules();
    for (line, queries) in specs.iter().zip(best) {
        let name = get_str(line, "name").unwrap();
        let ini = decode_hex_bytes(get_str(line, "ini").unwrap()).unwrap();
        load_objects(&mut rules, &ini);
        let spec = draw_spec(&rules, name, 0);
        assert_eq!(dump_spec(spec), get_str(line, "dump").unwrap(), "{name}");

        assert_eq!(get_str(queries, "name"), Some(name));
        let mut count = 0;
        for query in get_str(queries, "queries").unwrap().split_terminator(';') {
            let (bits, index) = query.split_once('=').unwrap();
            let flags = flags_of(bits);
            let found = spec.find_best_info(&flags).map_or(-1, |i| i as i32);
            let index: i32 = index.parse().unwrap(); // determinism-ok: oracle row
            assert_eq!(found, index, "{name} {bits}");
            count += 1;
        }
        assert_eq!(count, 48);
    }
}

/// The generated objects and files, with the catalog the port builds from them.
struct Objects {
    sim: SimContext,
    /// `(name, approach positions, position and angle)`.
    objects: Vec<(&'static str, i32, Vec<f32>)>,
}

fn objects() -> Objects {
    let files: BTreeMap<&str, Vec<u8>> = rows("file")
        .into_iter()
        .map(|l| {
            let name = get_str(l, "name").unwrap();
            (name, decode_hex_bytes(get_str(l, "hex").unwrap()).unwrap())
        })
        .collect();
    let mut sim = SimContext::new();
    let mut rules = new_rules();
    let standard = rows("standard");
    rules.global_data.standard_public_bones = get_str(standard[0], "names")
        .unwrap()
        .split(',')
        .map(str::to_owned)
        .collect();
    let mut objects = Vec::new();
    for line in rows("object") {
        let ini = decode_hex_bytes(get_str(line, "ini").unwrap()).unwrap();
        load_objects(&mut rules, &ini);
        objects.push((
            get_str(line, "name").unwrap(),
            number(get_str(line, "approach").unwrap()),
            words(get_str(line, "at").unwrap()),
        ));
    }
    assert_eq!(objects.len(), 60);
    rules.thing_factory.post_process_load();
    let catalog = ModelBoneCatalog::load(&rules, |path| {
        let name = path.strip_prefix("Art\\W3D\\")?.to_ascii_lowercase();
        files.get(name.as_str()).cloned()
    });
    rules.model_bones = Some(Arc::new(catalog));
    sim.rules = rules;
    sim.world = SimWorld::new();
    let extent = Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(1000.0, 1000.0, 100.0),
    };
    sim.init_partition(20.0, extent);
    sim.world.next_object_id = 1;
    sim.world.player_default_teams.insert(1, TeamId(1));
    sim.add_player_team(TeamId(1), 1);
    Objects { sim, objects }
}

#[test]
fn drawable_bone_queries_match_the_reference() {
    let o = objects();
    let queries = rows("query");
    assert_eq!(queries.len(), 360);
    let mut checked = 0;
    let mut attach_offsets = AttachBoneOffsets::default();
    for (n, line) in queries.iter().enumerate() {
        let name = get_str(line, "name").unwrap();
        let (object, _, _) = &o.objects[n / 6];
        assert_eq!(*object, name);
        let rules = &o.sim.rules;
        let template = rules.thing_factory.find_template(name).unwrap();
        let conditions = flags_of(get_str(line, "conditions").unwrap());
        let bones = DrawableBones::new(rules, template, conditions).unwrap();
        for query in get_str(line, "results").unwrap().split_terminator(';') {
            let (head, expected) = query.split_once('=').unwrap();
            let head: Vec<&str> = head.split(':').collect();
            let actual = match head[0] {
                "P" => {
                    let found = bones.pristine_bone_transforms(
                        head[1],
                        number(head[2]),
                        number(head[3]) as usize,
                    );
                    let matrices: String = found.iter().map(matrix_text).collect();
                    format!("{}:{matrices}", found.len())
                }
                "B" => bones.barrel_count(number(head[1]) as usize).to_string(),
                "L" => {
                    let turret = match number(head[3]) {
                        0 => WhichTurretType::Main,
                        1 => WhichTurretType::Alt,
                        _ => WhichTurretType::Invalid,
                    };
                    let out = bones.projectile_launch_offset(
                        number(head[1]) as usize,
                        number(head[2]),
                        turret,
                        &mut attach_offsets,
                    );
                    format!(
                        "{}:{}:{}{}",
                        u32::from(out.launch.is_some()),
                        out.launch.as_ref().map_or_else(String::new, matrix_text),
                        pos_text(&out.turret_rot_pos),
                        pos_text(&out.turret_pitch_pos)
                    )
                }
                _ => continue,
            };
            assert_eq!(actual, expected, "{name} row {n} {head:?}");
            checked += 1;
        }
    }
    assert_eq!(checked, 360 * 35);
}

#[test]
fn launch_and_dock_positions_match_the_reference() {
    let mut o = objects();
    let queries = rows("query");
    let sim = &mut o.sim;
    let mut object = None;
    let mut checked = 0;
    let mut turreted = 0;
    for (n, line) in queries.iter().enumerate() {
        let name = get_str(line, "name").unwrap();
        let (_, approach, at) = &o.objects[n / 6];
        if n % 6 == 0 {
            if let Some(slot) = object {
                sim.destroy_object(slot);
            }
            let template = sim.rules.thing_factory.find_template_id(name).unwrap();
            let slot = sim.factory_new_object(template, TeamId(1));
            sim.set_object_position(slot, &Coord3D::new(at[0], at[1], at[2]));
            sim.set_object_orientation(slot, at[3]);
            object = Some(slot);
        }
        let slot = object.unwrap();
        let object = sim.world.objects.get_mut(slot).unwrap();
        object.model_condition = flags_of(get_str(line, "conditions").unwrap());
        let ai = object.ai_slot;
        let turned = words(get_str(line, "turrets").unwrap());
        assert_eq!(ai.is_some(), !turned.is_empty(), "{name}");
        if let Some(ai) = ai {
            let state = sim.resident_state_mut(ai).unwrap();
            let state = state.downcast_mut::<AiUpdate>().unwrap();
            for (turret, angles) in state.turrets.iter_mut().zip(turned.chunks(2)) {
                let turret = turret.as_mut().unwrap();
                (turret.angle, turret.pitch) = (angles[0], angles[1]);
            }
            turreted += 1;
        }
        for query in get_str(line, "results").unwrap().split_terminator(';') {
            let (head, expected) = query.split_once('=').unwrap();
            let head: Vec<&str> = head.split(':').collect();
            let actual = match head[0] {
                "W" => {
                    let wslot = WeaponSlotType::from_i32(number(head[1])).unwrap();
                    matrix_text(&launch_transform(sim, slot, wslot, number(head[2])))
                }
                "D" => {
                    let mut data =
                        DockUpdateModuleData::new(NameKeyType::INVALID, NameKeyType::INVALID);
                    data.number_approach_positions = *approach;
                    let mut dock = DockUpdateState::new(data);
                    dock.load_dock_positions(sim, slot);
                    let positions: String = dock.approach_positions.iter().map(pos_text).collect();
                    format!(
                        "{}{}{}:{}:{positions}",
                        pos_text(&dock.enter_position),
                        pos_text(&dock.dock_position),
                        pos_text(&dock.exit_position),
                        dock.number_approach_position_bones
                    )
                }
                _ => continue,
            };
            assert_eq!(actual, expected, "{name} row {n} {head:?}");
            checked += 1;
        }
    }
    assert_eq!(checked, 360 * 5);
    assert!(turreted > 100, "{turreted}");
}
