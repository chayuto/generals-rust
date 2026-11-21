//! Gate V2-Golden test suite for `WP-3.2a` Phase 3 Core Enum & BitFlags Registry Bundle
//! (`12` packets, 1,000 C++ reference vectors).
//!
//! When implementing `WP-3.2a`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

#![allow(dead_code)]

use zh_common::golden_json::{extract_vector_lines, get_i32_vec, get_str, get_u32, get_u64};
use zh_common::types::{Int, TRUE};
use zh_common::xfer::bit_flags::BitFlags;
use zh_logic::rules::{
    clear_damage_type_flag, is_health_damaging_damage, is_subdual_damage, kindof_mask_fs,
    ArmorSetType, DamageType, DamageTypeFlags, DeathType, DisabledMaskType, DisabledType,
    KindOfMaskType, KindOfType, ModelConditionFlagType, ObjectStatusMaskType, ObjectStatusType,
    SpecialPowerMaskType, SpecialPowerType, WeaponSetFlags, WeaponSetType, WeaponStatus,
    ARMORSET_COUNT, ARMOR_SET_NAMES, DAMAGE_NUM_TYPES, DAMAGE_TYPE_NAMES, DEATH_NUM_TYPES,
    DEATH_TYPE_NAMES, DISABLED_COUNT, DISABLED_TYPE_NAMES, KINDOF_COUNT, KINDOF_NAMES,
    LOCOMOTOR_SURFACE_COUNT, LOCOMOTOR_SURFACE_TYPE_NAMES, MODELCONDITION_COUNT,
    MODEL_CONDITION_NAMES, OBJECT_STATUS_COUNT, OBJECT_STATUS_NAMES, SPECIALPOWER_COUNT,
    SPECIAL_POWER_NAMES, WEAPONSET_COUNT, WEAPON_SET_NAMES, WEAPON_STATUS_COUNT,
};

const WP3_2A_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_2a_RulesEnums.json");

fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

fn str_crc(mut crc: u32, s: &str) -> u32 {
    let bytes = s.as_bytes();
    crc = rot_crc(crc, bytes.len() as u32);
    for &b in bytes {
        crc = rot_crc(crc, b as u32);
    }
    crc
}

fn table_crc(mut crc: u32, list: &[&str]) -> u32 {
    crc = rot_crc(crc, list.len() as u32);
    for (i, &name) in list.iter().enumerate() {
        crc = rot_crc(crc, i as u32);
        crc = str_crc(crc, name);
    }
    crc
}

fn splitmix32(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x9e37_79b9);
    let mut z = *state;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

#[test]
fn test_wp3_2a_rules_enums_1000_golden_vectors() {
    let lines = extract_vector_lines(WP3_2A_JSON);
    assert_eq!(
        lines.len(),
        1000,
        "Expected 1,000 golden vectors in WP3_2a_RulesEnums.json"
    );

    let first = lines[0];
    let expected_counts = get_i32_vec(first, "counts").expect("counts");
    assert_eq!(
        expected_counts,
        vec![
            DISABLED_COUNT as i32,
            KINDOF_COUNT as i32,
            MODELCONDITION_COUNT as i32,
            OBJECT_STATUS_COUNT as i32,
            SPECIALPOWER_COUNT as i32,
            ARMORSET_COUNT as i32,
            DAMAGE_NUM_TYPES as i32,
            DEATH_NUM_TYPES as i32,
            WEAPONSET_COUNT as i32,
            WEAPON_STATUS_COUNT as i32,
        ]
    );
    assert_eq!(LOCOMOTOR_SURFACE_COUNT, 5);

    let mut names_crc = 0u32;
    names_crc = table_crc(names_crc, &DISABLED_TYPE_NAMES);
    names_crc = table_crc(names_crc, &KINDOF_NAMES);
    names_crc = table_crc(names_crc, &MODEL_CONDITION_NAMES);
    names_crc = table_crc(names_crc, &OBJECT_STATUS_NAMES);
    names_crc = table_crc(names_crc, &SPECIAL_POWER_NAMES);
    names_crc = table_crc(names_crc, &ARMOR_SET_NAMES);
    names_crc = table_crc(names_crc, &DAMAGE_TYPE_NAMES);
    names_crc = table_crc(names_crc, &DEATH_TYPE_NAMES);
    names_crc = table_crc(names_crc, &LOCOMOTOR_SURFACE_TYPE_NAMES);
    names_crc = table_crc(names_crc, &WEAPON_SET_NAMES);
    assert_eq!(
        names_crc,
        get_u32(first, "names_crc").unwrap(),
        "names_crc mismatch"
    );

    let fs_hex_crc = str_crc(0, &kindof_mask_fs().to_hex_string());
    assert_eq!(
        fs_hex_crc,
        get_u32(first, "fs_hex_crc").unwrap(),
        "fs_hex_crc mismatch"
    );

    let mut subdual_mask = 0u64;
    let mut health_dmg_mask = 0u64;
    for d in 0..(DAMAGE_NUM_TYPES as Int) {
        assert_eq!(DamageType::from_i32(d).unwrap().as_i32(), d);
        if is_subdual_damage(d) {
            subdual_mask |= 1u64 << d;
        }
        if is_health_damaging_damage(d) {
            health_dmg_mask |= 1u64 << d;
        }
    }
    assert_eq!(subdual_mask, get_u64(first, "subdual_mask").unwrap());
    assert_eq!(health_dmg_mask, get_u64(first, "health_dmg_mask").unwrap());

    for i in 0..(DISABLED_COUNT as Int) {
        assert_eq!(DisabledType::from_i32(i).unwrap().as_i32(), i);
    }
    for i in 0..(KINDOF_COUNT as Int) {
        assert_eq!(KindOfType::from_i32(i).as_i32(), i);
    }
    for i in 0..(MODELCONDITION_COUNT as Int) {
        assert_eq!(ModelConditionFlagType::from_i32(i).as_i32(), i);
    }
    for i in 0..(OBJECT_STATUS_COUNT as Int) {
        assert_eq!(ObjectStatusType::from_i32(i).unwrap().as_i32(), i);
    }
    for i in 0..(SPECIALPOWER_COUNT as Int) {
        assert_eq!(SpecialPowerType::from_i32(i).as_i32(), i);
    }
    for i in 0..(ARMORSET_COUNT as Int) {
        assert_eq!(ArmorSetType::from_i32(i).unwrap().as_i32(), i);
    }
    for i in 0..(DEATH_NUM_TYPES as Int) {
        assert_eq!(DeathType::from_i32(i).unwrap().as_i32(), i);
    }
    for i in 0..(WEAPONSET_COUNT as Int) {
        assert!(WeaponSetType::from_i32(i).is_some());
    }
    for i in 0..(WEAPON_STATUS_COUNT as Int) {
        assert!(WeaponStatus::from_i32(i).is_some());
    }

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let mut rng = seed.wrapping_mul(0x9e37_79b9).wrapping_add(0x517c_c1b7);

        let mut kf_a = KindOfMaskType::new();
        let mut kf_b = KindOfMaskType::new();
        for _ in 0..6 {
            let idx_a = (splitmix32(&mut rng) % KINDOF_COUNT as u32) as Int;
            let idx_b = (splitmix32(&mut rng) % KINDOF_COUNT as u32) as Int;
            kf_a.set(idx_a, TRUE as Int);
            kf_b.set(idx_b, TRUE as Int);
        }
        let mut kf_res = kf_a.clone();
        let struct_mask = KindOfMaskType::from_single_index(KindOfType::Structure.as_i32());
        kf_res.clear_and_set(&kf_b, &struct_mask);
        let kf_match = kf_res.test_set_and_clear(&struct_mask, &kf_b);

        let mut dis = DisabledMaskType::new();
        dis.set(
            (splitmix32(&mut rng) % DISABLED_COUNT as u32) as Int,
            TRUE as Int,
        );
        dis.set(
            (splitmix32(&mut rng) % DISABLED_COUNT as u32) as Int,
            TRUE as Int,
        );

        let mut st = ObjectStatusMaskType::new();
        st.set(
            (splitmix32(&mut rng) % OBJECT_STATUS_COUNT as u32) as Int,
            TRUE as Int,
        );
        st.set(
            (splitmix32(&mut rng) % OBJECT_STATUS_COUNT as u32) as Int,
            TRUE as Int,
        );

        let mut sp = SpecialPowerMaskType::new();
        sp.set(
            (splitmix32(&mut rng) % SPECIALPOWER_COUNT as u32) as Int,
            TRUE as Int,
        );
        sp.set(
            (splitmix32(&mut rng) % SPECIALPOWER_COUNT as u32) as Int,
            TRUE as Int,
        );

        let mut ws = WeaponSetFlags::new();
        ws.set(
            (splitmix32(&mut rng) % WEAPONSET_COUNT as u32) as Int,
            TRUE as Int,
        );
        ws.set(
            (splitmix32(&mut rng) % WEAPONSET_COUNT as u32) as Int,
            TRUE as Int,
        );

        let mut df = DamageTypeFlags::new_set_all();
        let clear_dt = (splitmix32(&mut rng) % DAMAGE_NUM_TYPES as u32) as Int;
        df = clear_damage_type_flag(df, clear_dt);

        let k_idx = (splitmix32(&mut rng) % KINDOF_COUNT as u32) as Int;
        let sp_idx = (splitmix32(&mut rng) % SPECIALPOWER_COUNT as u32) as Int;
        let st_idx = (splitmix32(&mut rng) % OBJECT_STATUS_COUNT as u32) as Int;

        let orig_k =
            BitFlags::<KINDOF_COUNT>::get_name_from_single_bit(&KINDOF_NAMES, k_idx).unwrap();
        let k_mix: String = orig_k
            .bytes()
            .enumerate()
            .map(|(i, b)| {
                if ((i as u32).wrapping_add(seed) & 1) != 0 {
                    (b as char).to_ascii_lowercase()
                } else {
                    b as char
                }
            })
            .collect();
        let k_looked = BitFlags::<KINDOF_COUNT>::get_single_bit_from_name(&KINDOF_NAMES, &k_mix);

        assert_eq!(
            kf_res.to_hex_string(),
            get_str(line, "kf_res_hex").unwrap(),
            "seed {} kf_res_hex",
            seed
        );

        let mut out_crc = 0u32;
        out_crc = rot_crc(out_crc, kf_res.to_unsigned_int());
        out_crc = rot_crc(out_crc, kf_match as u32);
        out_crc = rot_crc(out_crc, dis.to_unsigned_int());
        out_crc = rot_crc(out_crc, st.to_unsigned_int());
        out_crc = rot_crc(out_crc, sp.to_unsigned_int());
        out_crc = rot_crc(out_crc, ws.to_unsigned_int());
        out_crc = rot_crc(out_crc, df.to_unsigned_int());
        out_crc = rot_crc(out_crc, k_looked as u32);
        out_crc = str_crc(out_crc, &sp.to_hex_string());
        out_crc = str_crc(out_crc, &st.to_hex_string());
        out_crc = str_crc(out_crc, &ws.to_hex_string());
        out_crc = str_crc(out_crc, &dis.to_hex_string());
        out_crc = str_crc(
            out_crc,
            BitFlags::<SPECIALPOWER_COUNT>::get_name_from_single_bit(&SPECIAL_POWER_NAMES, sp_idx)
                .unwrap(),
        );
        out_crc = str_crc(
            out_crc,
            BitFlags::<OBJECT_STATUS_COUNT>::get_name_from_single_bit(&OBJECT_STATUS_NAMES, st_idx)
                .unwrap(),
        );
        assert_eq!(
            out_crc,
            get_u32(line, "out_crc").unwrap(),
            "seed {} out_crc",
            seed
        );
    }
}
