//! The draw module bone data and queries against the reference
//! (`tests/cpp_harness/gen_model_bones.cpp`). Rows carry their own INI.

use zh_common::golden_json::{decode_hex_bytes, get_str};
use zh_common::ini::{IniLoadType, IniReader};
use zh_logic::object::model_draw::{ModelDrawModuleData, ModelDrawSpec};
use zh_logic::rules::model_state::ModelConditionFlags;
use zh_logic::sim::RulesRegistry;

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
            let mut flags = ModelConditionFlags::default();
            for bit in bits.split_terminator('.') {
                flags.set(bit.parse().unwrap(), 1); // determinism-ok: oracle row
            }
            let found = spec.find_best_info(&flags).map_or(-1, |i| i as i32);
            let index: i32 = index.parse().unwrap(); // determinism-ok: oracle row
            assert_eq!(found, index, "{name} {bits}");
            count += 1;
        }
        assert_eq!(count, 48);
    }
}
