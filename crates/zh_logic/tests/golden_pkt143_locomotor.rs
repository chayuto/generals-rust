//! Gate V2 Golden Vector integrity guard & offload contract for `PKT-143` / `PKT-144`
//! (`WP-3.3` `Locomotor`, `LocomotorTemplate`, `LocomotorStore`, `LocomotorSet`).
//!
//! Active test `test_pkt143_oracle_integrity` verifies the committed C++ Golden Vector oracle
//! (`docs/golden_vectors/PKT-143.json`). For `env:ephemeral-web` offload: create
//! `crates/zh_logic/src/combat/locomotor.rs` and `crates/zh_logic/src/combat/locomotor_set.rs`,
//! wire them in `crates/zh_logic/src/combat/mod.rs`, replace the `#[ignore]` test below with
//! the full 1,000-seed assertion loop against `VECTORS_JSON`, remove `#[ignore]`, and run
//! `python3 scripts/port_map.py --write`.

use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32};

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
#[ignore = "PKT-143/144 Locomotor & LocomotorSet not yet ported — create crates/zh_logic/src/combat/{locomotor,locomotor_set}.rs, wire combat/mod.rs, replace body with 1,000-seed assertions, remove #[ignore], and run python3 scripts/port_map.py --write"]
fn test_pkt143_locomotor_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}
