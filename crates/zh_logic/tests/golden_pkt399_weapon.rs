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

use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32};

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
#[ignore = "Seeded for env:ephemeral-web offload (Issue #79): implement crates/zh_logic/src/object/weapon.rs & weapon_set.rs and remove #[ignore]"]
fn test_pkt399_weapon_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}
