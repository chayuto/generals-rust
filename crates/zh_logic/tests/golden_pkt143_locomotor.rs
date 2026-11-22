//! Gate V2 Golden Vector integrity guard & offload contract for `PKT-143` / `PKT-144`
//! (`WP-3.3` `Locomotor`, `LocomotorTemplate`, `LocomotorStore`, `LocomotorSet`).
//!
//! Active test `test_pkt143_oracle_integrity` verifies the committed C++ Golden Vector oracle
//! (`docs/golden_vectors/PKT-143.json`). For `env:ephemeral-web` offload: create
//! `crates/zh_logic/src/combat/locomotor.rs` and `crates/zh_logic/src/combat/locomotor_set.rs`,
//! wire them in `crates/zh_logic/src/combat/mod.rs`, replace the `#[ignore]` test below with
//! the full 1,000-seed assertion loop against `VECTORS_JSON`, remove `#[ignore]`, and run
//! `python3 scripts/port_map.py --write`.

use zh_common::golden_json::{
    encode_hex_bytes, extract_vector_lines, get_i32, get_str, get_u32, get_u32_array,
};
use zh_common::crc::Crc;
use zh_common::math::Coord3D;
use zh_common::random_value::PrngState;
use zh_common::xfer::{Xfer, XferLoad, XferSave};
use std::collections::BTreeMap;
use zh_logic::combat::locomotor::{
    Locomotor, LocomotorStore, LocomotorTemplate, BIGNUM,
};
use zh_logic::combat::locomotor_set::LocomotorSet;
use zh_logic::gamelogic::global_data::BodyDamageType;

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-143.json");

#[test]
fn test_pkt143_oracle_integrity() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-143\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("Locomotor.h"));
    assert!(VECTORS_JSON.contains("Locomotor.cpp"));
    assert!(VECTORS_JSON.contains("LocomotorSet.h"));

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000, "PKT-143 oracle must contain 1,000 seeds");
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(idx as u32));
        assert!(get_u32(line, "set_rand_seed").is_some());
        assert!(get_u32(line, "lxcrc").is_some());
        assert!(get_u32(line, "sxcrc").is_some());
        assert!(get_i32(line, "lxlen").is_some());
        assert!(get_i32(line, "sxlen").is_some());
    }

    let s0 = lines[0];
    assert!(s0.contains("\"s0_tmpl\""), "Seed 0 must include s0_tmpl");
    assert!(s0.contains("\"s0_loco\""), "Seed 0 must include s0_loco");
    assert!(s0.contains("\"s0_set\""), "Seed 0 must include s0_set");
    let lxlen = get_i32(s0, "lxlen").expect("Seed 0 lxlen") as usize;
    let sxlen = get_i32(s0, "sxlen").expect("Seed 0 sxlen") as usize;
    assert_eq!(get_str(s0, "l_xhex").expect("Seed 0 l_xhex").len(), lxlen * 2);
    assert_eq!(get_str(s0, "s_xhex").expect("Seed 0 s_xhex").len(), sxlen * 2);
}

#[test]
fn test_pkt143_locomotor_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
    let mut historical_templates: BTreeMap<String, LocomotorTemplate> = BTreeMap::new();

    for (seed, line) in lines.iter().enumerate() {
        let mut template = LocomotorTemplate::new();
        let ini = get_str(line, "ini").expect("ini").replace("\\n", "\n");
        template.parse_ini_block(&ini).expect("parse locomotive template");

        let mut store = LocomotorStore::new();
        for (_name, old_template) in &historical_templates {
            store.add_template(old_template.clone());
        }
        store.add_template(template.clone());

        if let Some(ovr) = get_str(line, "ovr").filter(|v| !v.is_empty()).map(|v| v.replace("\\n", "\n")) {
            let mut override_template = LocomotorTemplate::new();
            override_template.parse_ini_block(&ovr).expect("parse override");
            if override_template.max_speed != 0.0 {
                template.max_speed = override_template.max_speed;
            }
            if override_template.max_turn_rate != 0.0 {
                template.max_turn_rate = override_template.max_turn_rate;
            }
            store.clear();
            for (_name, old_template) in &historical_templates {
                store.add_template(old_template.clone());
            }
            store.add_template(template.clone());
        }

        historical_templates.insert(template.name.clone(), template.clone());

        let bad_threw = line.contains("\"bad_threw\":true");
        if let Some(bad) = get_str(line, "bad").filter(|v| !v.is_empty()).map(|v| v.replace("\\n", "\n")) {
            let mut bad_template = LocomotorTemplate::new();
            assert_eq!(
                bad_template.parse_ini_block(&bad).is_err(),
                bad_threw,
                "seed {seed} bad THRUST validation"
            );
        } else {
            assert!(!bad_threw, "seed {seed} unexpectedly marked bad");
        }

        let mut rng = PrngState::new();
        rng.init_random_with_seed(get_u32(line, "rand_seed").unwrap());
        let frame0 = get_u32(line, "frame0").unwrap();
        let mut loco = Locomotor::new(&template, &mut rng, frame0);
        assert_eq!(
            rng.get_game_logic_random_seed_crc(),
            get_u32(line, "rand_crc").unwrap(),
            "seed {seed} random seed CRC"
        );

        let mpos = get_u32_array::<3>(line, "mpos").unwrap();
        loco.maintain_pos = Coord3D {
            x: f32::from_bits(mpos[0]),
            y: f32::from_bits(mpos[1]),
            z: f32::from_bits(mpos[2]),
        };

        let ov = get_u32_array::<12>(line, "ov").unwrap();
        let omit = 99999.0_f32.to_bits();
        let ov_speed = if ov[0] == omit { BIGNUM } else { f32::from_bits(ov[0]) };
        let ov_turn = if ov[1] == omit { BIGNUM } else { f32::from_bits(ov[1]) };
        let ov_acc = if ov[2] == omit { BIGNUM } else { f32::from_bits(ov[2]) };
        let ov_lift = if ov[3] == omit { BIGNUM } else { f32::from_bits(ov[3]) };
        let ov_brk = if ov[4] == omit { BIGNUM } else { f32::from_bits(ov[4]) };
        let close = if ov[5] == omit { BIGNUM } else { f32::from_bits(ov[5]) };
        let pref = if ov[6] == omit { BIGNUM } else { f32::from_bits(ov[6]) };
        loco.set_movement_overrides(
            ov_speed,
            ov_turn,
            ov_acc,
            ov_lift,
            ov_brk,
            close,
            pref,
            ov[7] as u8,
            ov[8] as u8,
            ov[9] as u8,
            ov[10] as u8,
            ov[11] as u8,
        );
        loco.start_move(get_u32(line, "start_frame").unwrap());

        let movement_penalty =
            BodyDamageType::from_i32(get_i32(line, "move_pen").unwrap()).unwrap();

        let want_spd = get_u32_array::<4>(line, "spd").unwrap();
        let want_trn = get_u32_array::<4>(line, "trn").unwrap();
        let want_acc = get_u32_array::<4>(line, "acc").unwrap();
        let want_lft = get_u32_array::<4>(line, "lft").unwrap();
        let want_rad = get_u32_array::<4>(line, "rad").unwrap();
        let want_tim = get_u32_array::<4>(line, "tim").unwrap();
        for condition_idx in 0..4usize {
            let want_s = want_spd[condition_idx];
            let want_t = want_trn[condition_idx];
            let want_a = want_acc[condition_idx];
            let want_l = want_lft[condition_idx];
            let want_r = want_rad[condition_idx];
            let want_m = want_tim[condition_idx];
            let condition = BodyDamageType::from_i32(condition_idx as i32).unwrap();
            assert_eq!(loco.get_max_speed_for_condition(condition, movement_penalty).to_bits(), want_s, "seed {seed} spd {condition:?}");
            assert_eq!(loco.get_max_turn_rate(condition, movement_penalty, loco.is_ultra_accurate()).to_bits(), want_t, "seed {seed} turn {condition:?}");
            assert_eq!(loco.get_max_acceleration(condition, movement_penalty).to_bits(), want_a, "seed {seed} acc {condition:?}");
            assert_eq!(loco.get_max_lift(condition, movement_penalty).to_bits(), want_l, "seed {seed} lift {condition:?}");
            let (radius, time) = loco.calc_min_turn_radius(condition, movement_penalty);
            assert_eq!(radius.to_bits(), want_r, "seed {seed} radius {condition:?}");
            assert_eq!(time.to_bits(), want_m, "seed {seed} time {condition:?}");
        }

        assert_eq!(loco.get_braking().to_bits(), get_u32(line, "brk").unwrap(), "seed {seed} braking");
        assert_eq!(loco.flags, get_u32(line, "lflags").unwrap(), "seed {seed} locomotor flags");

        let mut lsave = XferSave::new();
        lsave.xfer_snapshot(&mut loco).unwrap();
        let lbytes = lsave.as_bytes();
        if let Some(want_hex) = get_str(line, "l_xhex") {
            assert_eq!(encode_hex_bytes(lbytes), want_hex, "seed {seed} locomotor XferSave");
        }
        assert_eq!(lbytes.len(), get_i32(line, "lxlen").unwrap() as usize, "seed {seed} locomotor length");
        let mut lcrc = Crc::new();
        lcrc.compute_crc(lbytes, lbytes.len() as i32);
        assert_eq!(lcrc.get(), get_u32(line, "lxcrc").unwrap(), "seed {seed} locomotor CRC");

        let mut loaded = loco.clone();
        let mut lload = XferLoad::new(lbytes);
        lload.xfer_snapshot(&mut loaded).unwrap();
        assert_eq!(loaded, loco, "seed {seed} locomotor save/load round trip");

        let mut set_rng = PrngState::new();
        set_rng.init_random_with_seed(get_u32(line, "set_rand_seed").unwrap());
        let mut set = LocomotorSet::new();
        for name in get_str_array(line, "set_names") {
            let mut template = store.find_by_name(&name).cloned().unwrap_or_else(|| {
                let mut clone = template.clone();
                clone.name = name.clone();
                clone
            });
            template.name = name.clone();
            store.add_template(template.clone());
            {
                let loco = Locomotor::new(&template, &mut set_rng, get_u32(line, "frame0").unwrap());
                set.add_locomotor(loco);
            }
        }

        assert_eq!(set.get_valid_surfaces(), get_i32(line, "set_surf").unwrap(), "seed {seed} set surfaces");
        assert_eq!(set.is_downhill_only(), get_i32(line, "set_dh").unwrap() as u8, "seed {seed} set downhill");

        for (idx, mask) in [1, 2, 8, 20].iter().enumerate() {
            let got = set.find_locomotor(*mask).map(|l| l.get_template_name()).unwrap_or("");
            let want = get_str_array(line, "qhits")[idx].clone();
            assert_eq!(got, want, "seed {seed} qhit {idx}");
        }

        let mut ssave = XferSave::new();
        let cur_name = get_str(line, "cur_name").unwrap_or("");
        let cur_name_opt = if cur_name.is_empty() { None } else { Some(cur_name) };
        set.xfer_self_and_cur_loco_ptr(&mut ssave, &store, cur_name_opt).unwrap();
        let sbytes = ssave.as_bytes();
        if let Some(want_hex) = get_str(line, "s_xhex") {
            assert_eq!(encode_hex_bytes(sbytes), want_hex, "seed {seed} LocomotorSet XferSave");
        }
        assert_eq!(sbytes.len(), get_i32(line, "sxlen").unwrap() as usize, "seed {seed} set length");
        let mut scrc = Crc::new();
        scrc.compute_crc(sbytes, sbytes.len() as i32);
        assert_eq!(scrc.get(), get_u32(line, "sxcrc").unwrap(), "seed {seed} set CRC");

        if seed == 0 {
            let s0 = lines[0];
            assert!(s0.contains("\"s0_tmpl\""));
            assert!(s0.contains("\"s0_loco\""));
            assert!(s0.contains("\"s0_set\""));
        }
    }
}

fn get_str_array(obj_line: &str, key: &str) -> Vec<String> {
    let rest = obj_line
        .split(&format!("\"{key}\":"))
        .nth(1)
        .unwrap()
        .trim_start();
    let end = rest.find(']').unwrap();
    rest[..=end]
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_owned())
        .collect()
}
