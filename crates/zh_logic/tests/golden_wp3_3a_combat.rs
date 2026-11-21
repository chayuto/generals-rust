//! Gate V2 Golden Vector parity tests for `WP-3.3a` (`PKT-138` `Armor` + `PKT-140` `DamageInfo`).

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_hex_u32, get_i32, get_str, get_u32, get_u32_array,
};
use zh_common::ini::{IniError, IniLoadType, IniReader, DAMAGE_TYPE_NAMES};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, ObjectId, Real};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::combat::armor::{Armor, ArmorStore, ArmorTemplate};
use zh_logic::combat::damage::{DamageInfo, DamageInfoInput, HUGE_DAMAGE_AMOUNT};
use zh_logic::rules::combat_types::{DamageType, DeathType, DAMAGE_NUM_TYPES};
use zh_logic::rules::object_status_types::ObjectStatusType;

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_3a_Combat.json");

const KNOWN_TEMPLATES: &[&str] = &[
    "AmericaTankCrusader",
    "GLAWorker",
    "ChinaTankOverlord",
    "AmericaInfantryRanger",
];

#[test]
fn test_wp3_3a_oracle_structure_and_metadata() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_wp3_3a_armor_1000_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    // Negative / error-path check matching C++ startup self-check.
    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let mut bad_store = ArmorStore::new();
    let mut bad_ini = IniReader::new(
        "bad_armor.ini",
        b"Armor Bad\n  Armor = NONEXISTENT_DAMAGE_XYZ 50%\nEnd\n",
        IniLoadType::Overwrite,
    );
    assert_eq!(
        bad_store.load_from_ini(&mut bad_ini, &mut name_keys),
        Err(IniError::InvalidData)
    );
    assert_eq!(
        IniReader::scan_index_list("NONEXISTENT_DAMAGE_XYZ", &DAMAGE_TYPE_NAMES),
        Err(IniError::InvalidData)
    );

    // Default ArmorTemplate is 100% across all 38 types.
    let fresh = ArmorTemplate::new();
    for i in 0..DAMAGE_NUM_TYPES {
        let dt = DamageType::from_i32(i as i32).unwrap();
        assert_eq!(fresh.adjust_damage(dt, 10.0_f32).to_bits(), 10.0_f32.to_bits());
    }

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let armor_name = get_str(line, "armor_name").unwrap();
        let ini_hex = get_str(line, "armor_ini_hex").unwrap();
        let ini_bytes = decode_hex_bytes(ini_hex).unwrap();

        let query_dt = DamageType::from_i32(get_i32(line, "query_dt").unwrap()).unwrap();
        let query_dmg = Real::from_bits(get_hex_u32(line, "query_dmg_bits").unwrap());
        let exp_adj = get_hex_u32(line, "adj_bits").unwrap();
        let exp_null = get_hex_u32(line, "null_adj_bits").unwrap();
        let exp_unres = get_hex_u32(line, "unres_bits").unwrap();
        let exp_subdual_unres = get_hex_u32(line, "subdual_unres_bits").unwrap();
        let exp_probe1: [u32; DAMAGE_NUM_TYPES] = get_u32_array(line, "probe1_bits").unwrap();

        name_keys.reset();
        let mut store = ArmorStore::new();
        let mut ini = IniReader::new("ArmorTest.ini", &ini_bytes, IniLoadType::Overwrite);
        store
            .load_from_ini(&mut ini, &mut name_keys)
            .unwrap_or_else(|e| panic!("seed {seed}: load_from_ini failed: {e:?}"));

        let tmpl = store
            .find_armor_template(&mut name_keys, armor_name)
            .unwrap_or_else(|| panic!("seed {seed}: missing armor {armor_name}"));
        let armor = store.make_armor(Some(tmpl));
        let empty = Armor::none();

        assert_eq!(
            armor.adjust_damage(query_dt, query_dmg).to_bits(),
            exp_adj,
            "seed {seed}: adjust_damage mismatch"
        );
        assert_eq!(
            empty.adjust_damage(query_dt, query_dmg).to_bits(),
            exp_null,
            "seed {seed}: null armor mismatch"
        );
        assert_eq!(
            armor.adjust_damage(DamageType::Unresistable, query_dmg).to_bits(),
            exp_unres,
            "seed {seed}: unresistable mismatch"
        );
        assert_eq!(
            armor
                .adjust_damage(DamageType::SubdualUnresistable, query_dmg)
                .to_bits(),
            exp_subdual_unres,
            "seed {seed}: subdual_unresistable mismatch"
        );

        for (i, &exp_bits) in exp_probe1.iter().enumerate() {
            let dt = DamageType::from_i32(i as i32).unwrap();
            assert_eq!(
                armor.adjust_damage(dt, 1.0_f32).to_bits(),
                exp_bits,
                "seed {seed} dt {i}: probe1 mismatch"
            );
        }
    }
}

#[test]
fn test_wp3_3a_damage_info_1000_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
    assert_eq!(HUGE_DAMAGE_AMOUNT, 999_999.0_f32);

    let default_info = DamageInfo::default();
    assert_eq!(default_info.input.source_id, ObjectId::INVALID);
    assert_eq!(default_info.input.damage_type, DamageType::Explosion);
    assert_eq!(default_info.input.damage_fx_override, DamageType::Unresistable);
    assert_eq!(default_info.input.death_type, DeathType::Normal);
    assert_eq!(default_info.input.damage_status_type, ObjectStatusType::None);
    assert_eq!(default_info.input.amount.to_bits(), 0);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let tpl = get_str(line, "source_template").unwrap();

        let mut info = DamageInfo {
            input: DamageInfoInput {
                source_id: ObjectId(get_u32(line, "source_id").unwrap() as i32),
                source_template: tpl.to_owned(),
                source_player_mask: get_u32(line, "source_player_mask").unwrap() as u16,
                damage_type: DamageType::from_i32(get_i32(line, "damage_type").unwrap()).unwrap(),
                damage_status_type: ObjectStatusType::from_i32(
                    get_i32(line, "damage_status_type").unwrap(),
                )
                .unwrap(),
                damage_fx_override: DamageType::from_i32(
                    get_i32(line, "damage_fx_override").unwrap(),
                )
                .unwrap(),
                death_type: DeathType::from_i32(get_i32(line, "death_type").unwrap()).unwrap(),
                amount: Real::from_bits(get_hex_u32(line, "amount_bits").unwrap()),
                kill: get_u32(line, "kill").unwrap() as Bool,
                shock_wave_vector: Coord3D {
                    x: Real::from_bits(get_hex_u32(line, "shock_x_bits").unwrap()),
                    y: Real::from_bits(get_hex_u32(line, "shock_y_bits").unwrap()),
                    z: Real::from_bits(get_hex_u32(line, "shock_z_bits").unwrap()),
                },
                shock_wave_amount: Real::from_bits(get_hex_u32(line, "shock_amt_bits").unwrap()),
                shock_wave_radius: Real::from_bits(get_hex_u32(line, "shock_rad_bits").unwrap()),
                shock_wave_taper_off: Real::from_bits(
                    get_hex_u32(line, "shock_taper_bits").unwrap(),
                ),
            },
            output: Default::default(),
        };
        info.output.actual_damage_dealt = Real::from_bits(get_hex_u32(line, "dealt_bits").unwrap());
        info.output.actual_damage_clipped =
            Real::from_bits(get_hex_u32(line, "clipped_bits").unwrap());
        info.output.no_effect = get_u32(line, "no_effect").unwrap() as Bool;

        let expected_save = decode_hex_bytes(get_str(line, "xfer_save_hex").unwrap()).unwrap();
        let expected_crc = get_hex_u32(line, "xfer_crc_hex").unwrap();

        // 1. XferSave parity
        let mut saver = XferSave::new();
        info.xfer(&mut saver).unwrap();
        assert_eq!(saver.as_bytes(), expected_save.as_slice(), "seed {seed}: XferSave mismatch");

        // 2. XferLoad bit-exact round-trip parity
        let mut loaded = DamageInfo::default();
        let mut loader = XferLoad::new(&expected_save);
        loaded
            .xfer_with_lookup(&mut loader, |name| KNOWN_TEMPLATES.contains(&name))
            .unwrap();
        assert_eq!(loader.position(), expected_save.len());

        assert_eq!(loaded.input.source_id, info.input.source_id);
        assert_eq!(loaded.input.source_template, info.input.source_template);
        assert_eq!(loaded.input.source_player_mask, info.input.source_player_mask);
        assert_eq!(loaded.input.damage_type, info.input.damage_type);
        assert_eq!(loaded.input.damage_status_type, info.input.damage_status_type);
        assert_eq!(loaded.input.damage_fx_override, info.input.damage_fx_override);
        assert_eq!(loaded.input.death_type, info.input.death_type);
        assert_eq!(loaded.input.amount.to_bits(), info.input.amount.to_bits());
        assert_eq!(loaded.input.kill, info.input.kill);
        assert_eq!(
            loaded.input.shock_wave_vector.x.to_bits(),
            info.input.shock_wave_vector.x.to_bits()
        );
        assert_eq!(
            loaded.input.shock_wave_vector.y.to_bits(),
            info.input.shock_wave_vector.y.to_bits()
        );
        assert_eq!(
            loaded.input.shock_wave_vector.z.to_bits(),
            info.input.shock_wave_vector.z.to_bits()
        );
        assert_eq!(
            loaded.input.shock_wave_amount.to_bits(),
            info.input.shock_wave_amount.to_bits()
        );
        assert_eq!(
            loaded.input.shock_wave_radius.to_bits(),
            info.input.shock_wave_radius.to_bits()
        );
        assert_eq!(
            loaded.input.shock_wave_taper_off.to_bits(),
            info.input.shock_wave_taper_off.to_bits()
        );
        assert_eq!(
            loaded.output.actual_damage_dealt.to_bits(),
            info.output.actual_damage_dealt.to_bits()
        );
        assert_eq!(
            loaded.output.actual_damage_clipped.to_bits(),
            info.output.actual_damage_clipped.to_bits()
        );
        assert_eq!(loaded.output.no_effect, info.output.no_effect);

        // 3. Unknown template resolves to empty string on load (matching findTemplate == nullptr).
        if !tpl.is_empty() {
            let mut unresolved = DamageInfo::default();
            let mut l2 = XferLoad::new(&expected_save);
            unresolved.xfer_with_lookup(&mut l2, |_| false).unwrap();
            assert!(unresolved.input.source_template.is_empty());
        }

        // 4. XferCrc parity
        let mut xcrc = XferCrc::new();
        xcrc.xfer_snapshot(&mut info).unwrap();
        assert_eq!(xcrc.get_crc(), expected_crc, "seed {seed}: XferCrc mismatch");
    }
}
