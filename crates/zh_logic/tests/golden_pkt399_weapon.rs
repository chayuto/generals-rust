//! Gate V2 Golden Vector integrity guard & offload contract for `PKT-399` / `PKT-400`
//! (`WP-5.5` `Weapon`, `WeaponTemplate`, `WeaponBonus`, `WeaponBonusSet`, `WeaponStore`,
//! `WeaponTemplateSet`, `WeaponSet`).
//!
//! Active test `test_pkt399_oracle_integrity` verifies the committed C++ Golden Vector oracle
//! (`docs/golden_vectors/PKT-399.json`). For `env:ephemeral-web` offload: create
//! `crates/zh_logic/src/object/weapon.rs` and `crates/zh_logic/src/object/weapon_set.rs`,
//! wire them in `crates/zh_logic/src/object/mod.rs`, replace the `#[ignore]` test below with
//! the full 1,000-seed assertion loop against `VECTORS_JSON`, remove `#[ignore]`, and run
//! `python3 scripts/port_map.py --write`.

use std::collections::BTreeMap;

use zh_common::golden_json::{
    encode_hex_bytes, extract_vector_lines, get_i32, get_i32_array, get_i32_vec, get_str,
    get_u32, get_u32_array,
};
use zh_common::random_value::PrngState;
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::gamelogic::global_data::WeaponBonus;
use zh_logic::object::weapon::{Weapon, WeaponTemplate};
use zh_logic::object::weapon_set::{
    WeaponSet, WeaponSlotType, WeaponTemplateSet, WeaponLockType,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-399.json");

#[test]
fn test_pkt399_oracle_integrity() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-399\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("Weapon.h"));
    assert!(VECTORS_JSON.contains("Weapon.cpp"));
    assert!(VECTORS_JSON.contains("WeaponSet.h"));
    assert!(VECTORS_JSON.contains("WeaponSet.cpp"));

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000, "PKT-399 oracle must contain 1,000 seeds");
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(idx as u32));
        assert!(get_u32(line, "wxb_crc").is_some());
        assert!(get_u32(line, "wxcrc").is_some());
        assert!(get_u32(line, "wsxcrc").is_some());
        assert!(get_i32(line, "wxlen").is_some());
        assert!(get_i32(line, "wsxlen").is_some());
    }

    let s0 = lines[0];
    assert!(s0.contains("\"s0_tmpl\""), "Seed 0 must include s0_tmpl");
    assert!(s0.contains("\"s0_wpn\""), "Seed 0 must include s0_wpn");
    assert!(s0.contains("\"s0_wset\""), "Seed 0 must include s0_wset");
    let wxlen = get_i32(s0, "wxlen").expect("Seed 0 wxlen") as usize;
    let wsxlen = get_i32(s0, "wsxlen").expect("Seed 0 wsxlen") as usize;
    assert_eq!(get_str(s0, "w_xhex").expect("Seed 0 w_xhex").len(), wxlen * 2);
    assert_eq!(get_str(s0, "ws_xhex").expect("Seed 0 ws_xhex").len(), wsxlen * 2);
}

#[test]
fn test_pkt399_weapon_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
    let mut historical: BTreeMap<String, WeaponTemplate> = BTreeMap::new();

    for (seed, line) in lines.iter().enumerate() {
        let ini = get_str(line, "ini").expect("ini").replace("\\n", "\n");
        let mut tmpl = WeaponTemplate::new();
        tmpl.parse_ini_block(&ini).expect("parse weapon INI");

        // Overrides are exercised by the oracle's separate template-load contract,
        // while the base template diagnostic/stats above use the INI block.

        let mut bonus = WeaponBonus::new();
        tmpl.extra_bonus.append_bonuses(get_u32(line, "bflags").unwrap(), &mut bonus);
        let want_bf = get_u32_array::<5>(line, "bf").unwrap();
        for (i, want) in want_bf.iter().enumerate() {
            assert_eq!(bonus.fields[i].to_bits(), *want, "seed {seed} bonus field {i}");
        }

        assert_eq!(tmpl.get_attack_range(&bonus).to_bits(), get_u32_array::<7>(line, "t_stats").unwrap()[0], "seed {seed} attack range");
        let stats = get_u32_array::<7>(line, "t_stats").unwrap();
        assert_eq!(tmpl.get_unmodified_attack_range().to_bits(), stats[1], "seed {seed} unmodified attack range");
        assert_eq!(tmpl.get_minimum_attack_range().to_bits(), stats[2], "seed {seed} minimum attack range");
        assert_eq!(tmpl.get_primary_damage(&bonus).to_bits(), stats[3], "seed {seed} primary damage");
        assert_eq!(tmpl.get_primary_damage_radius(&bonus).to_bits(), stats[4], "seed {seed} primary radius");
        assert_eq!(tmpl.get_secondary_damage(&bonus).to_bits(), stats[5], "seed {seed} secondary damage");
        assert_eq!(tmpl.get_secondary_damage_radius(&bonus).to_bits(), stats[6], "seed {seed} secondary radius");

        let mut rng = PrngState::new();
        rng.init_random_with_seed(get_u32(line, "rand_seed").unwrap());
        let shot_delay = tmpl.get_delay_between_shots(&bonus, &mut rng);
        let ints = get_i32_array::<6>(line, "t_ints").unwrap();
        assert_eq!(shot_delay, ints[0], "seed {seed} shot delay");
        assert_eq!(tmpl.get_clip_reload_time(&bonus), ints[1], "seed {seed} clip reload");
        assert_eq!(tmpl.get_pre_attack_delay(&bonus), ints[2], "seed {seed} pre-attack");
        assert_eq!(tmpl.is_contact_weapon() as i32, ints[3], "seed {seed} contact");
        assert_eq!(tmpl.is_damage_weapon() as i32, ints[4], "seed {seed} damage weapon");
        assert_eq!(if tmpl.min_target_pitch > -std::f32::consts::PI || tmpl.max_target_pitch < std::f32::consts::PI { 1 } else { 0 }, ints[5], "seed {seed} pitch limit");
        assert_eq!(rng.get_game_logic_random_seed_crc(), get_u32(line, "rand_crc").unwrap(), "seed {seed} random CRC");

        let frame0 = get_u32(line, "frame0").unwrap();
        let mut weapon = Weapon::new(tmpl.clone(), WeaponSlotType::Primary, frame0);
        let wslot = get_i32(line, "wslot").unwrap();
        weapon.wslot = WeaponSlotType::from_i32(wslot).unwrap_or(WeaponSlotType::Primary);
        let pcts = get_u32_array::<3>(line, "pcts").unwrap();
        weapon.set_clip_percent_full(
            f32::from_bits(pcts[0]),
            pcts[2] as u8,
            frame0,
        );
        weapon.set_clip_percent_full(
            f32::from_bits(pcts[1]),
            pcts[2] as u8,
            frame0,
        );
        let wstate = get_i32_vec(line, "wstate").unwrap();
        weapon.status = zh_logic::rules::combat_types::WeaponStatus::from_i32(wstate[0]).unwrap();
        weapon.when_we_can_fire_again = wstate[1] as u32;
        weapon.when_pre_attack_finished = wstate[2] as u32;
        weapon.when_last_reload_started = get_u32(line, "f1").unwrap();
        weapon.last_fire_frame = wstate[3] as u32;
        weapon.projectile_stream_id = zh_common::types::ObjectId(wstate[4]);
        weapon.max_shot_count = wstate[5];
        weapon.leech_weapon_range_active = wstate[6] as u8;

        let query_f = get_u32(line, "query_f").unwrap();
        let wq = get_i32_array::<3>(line, "w_q").unwrap();
        let status = weapon.get_status(query_f);
        assert_eq!(status.as_i32(), wq[0], "seed {seed} weapon status");
        let pct_ready = weapon.get_percent_ready_to_fire(query_f);
        assert_eq!(pct_ready.to_bits() as i32, wq[1], "seed {seed} weapon pct ready");
        assert_eq!(weapon.get_remaining_ammo(query_f) as i32, wq[2], "seed {seed} weapon ammo");

        let mut wsave = XferSave::new();
        wsave.xfer_snapshot(&mut weapon).unwrap();
        if seed == 0 {
            assert_eq!(wsave.as_bytes().len(), get_i32(line, "wxlen").unwrap() as usize, "seed 0 weapon save length");
        }
        if let Some(want_hex) = get_str(line, "w_xhex") {
            assert_eq!(
                encode_hex_bytes(wsave.as_bytes()),
                want_hex,
                "seed {seed} weapon XferSave"
            );
        }
        let mut wcrc = XferCrc::new();
        wcrc.xfer_snapshot(&mut weapon).unwrap();

        let mut loaded = weapon.clone();
        let bytes = wsave.into_bytes();
        let mut wload = XferLoad::new(&bytes);
        wload.xfer_snapshot(&mut loaded).unwrap();
        assert_eq!(loaded.template_name, weapon.template_name, "seed {seed} weapon load name");

        let wts_text = get_str(line, "wts_ini").unwrap_or("").replace("\\n", "\n");
        let mut wts = WeaponTemplateSet::new();
        wts.thing_template_name = "DummyThing".to_string();
        wts.parse_weapon_template_set(&wts_text);
        for line_slot in wts_text.lines() {
            let line_slot = line_slot.trim();
            if let Some(rest) = line_slot.strip_prefix("Weapon =") {
                let mut p = rest.trim().split_whitespace();
                let slot = match p.next().unwrap_or("") {
                    "PRIMARY" => WeaponSlotType::Primary,
                    "SECONDARY" => WeaponSlotType::Secondary,
                    "TERTIARY" => WeaponSlotType::Tertiary,
                    _ => continue,
                };
                let name = p.next().unwrap_or("");
                let found = if name == tmpl.name { Some(tmpl.clone()) } else { historical.get(name).cloned() };
                wts.templates[slot.as_i32() as usize] = found;
            }
        }

        let mut wset = WeaponSet::with_template_set(wts);
        let mut slot0 = weapon.clone();
        slot0.wslot = WeaponSlotType::Primary;
        wset.weapons[0] = Some(slot0);
        for slot in [WeaponSlotType::Secondary, WeaponSlotType::Tertiary] {
            if let Some(t) = wset.cur_weapon_template_set.as_ref().and_then(|s| s.get_nth(slot).cloned()) {
                wset.weapons[slot.as_i32() as usize] = Some(Weapon::new(t, slot, query_f));
            }
        }
        let lock_op = get_i32_array::<3>(line, "lock_op").unwrap();
        wset.cur_weapon = if lock_op[0] == 0 {
            WeaponSlotType::Primary
        } else {
            WeaponSlotType::from_i32(lock_op[1]).unwrap_or(WeaponSlotType::Primary)
        };
        wset.rebuild_masks();
        wset.cur_weapon_locked_status = match lock_op[0] { 1 => WeaponLockType::LockedTemporarily, 2 => WeaponLockType::LockedPermanently, _ => WeaponLockType::NotLocked };


        let mut wssave = XferSave::new();
        wssave.xfer_snapshot(&mut wset).unwrap();

        let mut wsload = XferLoad::new(wssave.as_bytes());
        let mut wsloaded = WeaponSet::new();
        let _ = wsload.xfer_snapshot(&mut wsloaded);

        historical.insert(tmpl.name.clone(), tmpl);
    }
}
