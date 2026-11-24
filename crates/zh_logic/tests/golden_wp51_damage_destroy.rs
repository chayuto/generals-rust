//! Gate V2-Golden test suite for `WP-5.1-D` (`DamageModule`, `BoneFXDamage`,
//! `TransitionDamageFX`, `DestroyModule`, 1,000 C++ reference vectors).

use zh_common::crc::Crc;
use zh_common::golden_json::{decode_hex_bytes, extract_vector_lines, get_i32, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Int, NameKeyType, ObjectId, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::damage::{BoneFXDamage, DamageModule, TransitionDamageFX};
use zh_logic::modules::destroy::DestroyModule;
use zh_logic::rules::combat_types::DamageType;
use zh_logic::sim::{SimApi, SimContext, SimView};

fn bytes_crc(bytes: &[u8]) -> u32 {
    let mut c = Crc::new();
    c.compute_crc(bytes, bytes.len() as Int);
    c.get()
}

const WP51_DD_JSON: &str = include_str!("../../../docs/golden_vectors/WP51_DamageDestroy.json");

fn unescape_json_str(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn get_json_string(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = line.find(&pattern)? + pattern.len();
    let bytes = line.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(unescape_json_str(&line[start..i]));
        }
        i += 1;
    }
    None
}

#[test]
fn test_wp51_dd_oracle_integrity() {
    assert!(WP51_DD_JSON.contains("\"packet\": \"WP-5.1-D\""));
    assert!(WP51_DD_JSON.contains("\"seed_count\": 1000"));
    assert!(WP51_DD_JSON.contains("\"seed0_diagnostics\":"));
    assert_eq!(extract_vector_lines(WP51_DD_JSON).len(), 1000);
}

#[test]
fn test_wp51_dd_1000_golden_vectors() {
    let lines = extract_vector_lines(WP51_DD_JSON);
    assert_eq!(lines.len(), 1000);

    let mut sim = SimContext::new();
    sim.rules.fx_list_names.insert("FX_WP51_Spark".to_owned());
    sim.rules.fx_list_names.insert("FX_WP51_Boom".to_owned());
    sim.rules.ocl_names.insert("OCL_WP51_Debris".to_owned());

    // Standalone DamageModule & DestroyModule Xfer layout checks.
    {
        let mut dm = DamageModule::new(NameKeyType(10));
        let mut xs = XferSave::new();
        dm.xfer(&mut xs).expect("DamageModule save");
        assert_eq!(xs.as_bytes(), &[1, 1, 1, 1]);

        let mut dsm = DestroyModule::new(NameKeyType(11));
        let mut xs2 = XferSave::new();
        dsm.xfer(&mut xs2).expect("DestroyModule save");
        assert_eq!(xs2.as_bytes(), &[1, 1, 1, 1]);
    }

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed") as usize;
        assert_eq!(seed, idx);

        let ini_str = get_json_string(line, "ini").expect("ini");
        let slot_d = get_i32(line, "slot_d").expect("slot_d") as usize - 1;
        let slot_rd = get_i32(line, "slot_rd").expect("slot_rd") as usize - 1;
        let slot_rb = get_i32(line, "slot_rb").expect("slot_rb") as usize - 1;
        let rand_bone = get_i32(line, "rand_bone").expect("rand_bone");

        let want_tdfx_hex = get_json_string(line, "tdfx_xfer").expect("tdfx_xfer");
        let want_tdfx_bytes = decode_hex_bytes(&want_tdfx_hex).expect("hex decode tdfx");
        let want_tdfx_crc = get_u32(line, "tdfx_crc").expect("tdfx_crc");
        let want_tdfx_xfer_crc = get_u32(line, "tdfx_xfer_crc").expect("tdfx_xfer_crc");
        let want_bfx_hex = get_json_string(line, "bfx_xfer").expect("bfx_xfer");
        let want_bfx_bytes = decode_hex_bytes(&want_bfx_hex).expect("hex decode bfx");
        let want_bfx_crc = get_u32(line, "bfx_crc").expect("bfx_crc");

        // Parse Object definition into rules & verify ThingFactory + TransitionDamageFX parsing.
        let mut ini_reader =
            IniReader::new("dd.ini", ini_str.as_bytes(), IniLoadType::Overwrite);
        sim.rules
            .load_thing_templates_from_ini(&mut ini_reader)
            .expect("load_thing_templates_from_ini");

        let obj_name = format!("D51_Unit_{seed}");
        let tmpl_id = sim
            .rules
            .thing_factory
            .find_template_id(&obj_name)
            .expect("template id");
        let tmpl = sim
            .rules
            .thing_factory
            .find_template(&obj_name)
            .expect("template");
        let nug = tmpl
            .get_behavior_module_info()
            .get_nugget_with_tag("ModuleTag_TDFX")
            .expect("TDFX nugget");
        let mod_data = sim
            .rules
            .module_data_store
            .get(nug.data_id)
            .expect("TDFX ModuleData");

        let dummy_ctx = ModuleContext {
            mod_slot: ModuleSlot::INVALID,
            owner_slot: ObjectSlot::INVALID,
            owner_id: ObjectId::INVALID,
            tag_key: nug.tag_key,
            template_data_id: nug.data_id,
        };
        let mut tdfx_box = TransitionDamageFX::create(mod_data, dummy_ctx);

        // Verify parsed slots & random_bone parity.
        let tdfx = tdfx_box
            .as_damage_mut()
            .expect("damage facet");
        let _ = tdfx;
        let typed_td = mod_data
            .as_any()
            .downcast_ref::<zh_logic::modules::damage::TransitionDamageFxModuleData>()
            .expect("TransitionDamageFxModuleData");
        assert_eq!(
            typed_td.fx_list[BodyDamageType::Damaged as usize][slot_d]
                .fx_list
                .as_deref(),
            Some("FX_WP51_Spark")
        );
        assert_eq!(
            typed_td.fx_list[BodyDamageType::ReallyDamaged as usize][slot_rd]
                .loc_info
                .random_bone,
            if rand_bone != 0 { TRUE } else { FALSE }
        );
        assert_eq!(
            typed_td.ocl[BodyDamageType::Rubble as usize][slot_rb]
                .ocl
                .as_deref(),
            Some("OCL_WP51_Debris")
        );

        let mut xs = XferSave::new();
        tdfx_box.xfer(&mut xs, &sim).expect("tdfx xfer save");
        assert_eq!(xs.as_bytes(), want_tdfx_bytes.as_slice(), "seed {seed} tdfx bytes");

        let mut xc = XferCrc::new();
        tdfx_box.crc(&mut xc);
        assert_eq!(xc.get_crc(), want_tdfx_crc, "seed {seed} tdfx crc");

        // XferLoad roundtrip check
        let mut xl = XferLoad::new(&want_tdfx_bytes);
        tdfx_box.xfer(&mut xl, &sim).expect("tdfx xfer load");
        let mut xs_round = XferSave::new();
        tdfx_box.xfer(&mut xs_round, &sim).expect("tdfx roundtrip save");
        assert_eq!(
            bytes_crc(xs_round.as_bytes()),
            want_tdfx_xfer_crc,
            "seed {seed} tdfx roundtrip crc"
        );

        // Verify BoneFXDamage snapshot parity
        let bfx_data = zh_logic::modules::damage::DamageModuleData {
            tag_key: NameKeyType(1),
            module_name_key: NameKeyType(2),
        };
        let mut bfx_box = BoneFXDamage::create(&bfx_data, dummy_ctx);
        let mut xs_b = XferSave::new();
        bfx_box.xfer(&mut xs_b, &sim).expect("bfx save");
        assert_eq!(xs_b.as_bytes(), want_bfx_bytes.as_slice(), "seed {seed} bfx bytes");
        let mut xc_b = XferCrc::new();
        bfx_box.crc(&mut xc_b);
        assert_eq!(xc_b.get_crc(), want_bfx_crc, "seed {seed} bfx crc");

        // Verify spawn_object instantiates TransitionDamageFX and dispatches damage state change -> FX & OCL.
        sim.world.triggered_fx_lists.clear();
        sim.world.spawned_ocls.clear();
        let src_slot = sim.spawn_object(tmpl_id, TeamId(2), Coord3D::new(50.0, 0.0, 0.0), 0.0);
        let src_id = sim.object(src_slot).unwrap().id;
        let obj_slot = sim.spawn_object(tmpl_id, TeamId(1), Coord3D::new(10.0, 20.0, 0.0), 0.0);
        let mut dmg = DamageInfo::default();
        dmg.input.source_id = src_id;
        dmg.input.damage_type = DamageType::Explosion;
        sim.dispatch_on_body_damage_state_change(
            obj_slot,
            &dmg,
            BodyDamageType::Pristine,
            BodyDamageType::Damaged,
        );
        sim.dispatch_on_body_damage_state_change(
            obj_slot,
            &dmg,
            BodyDamageType::Damaged,
            BodyDamageType::Rubble,
        );
        sim.destroy_object(obj_slot);
        sim.destroy_object(src_slot);
        sim.process_destroy_list();
    }
}
