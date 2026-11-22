//! Gate V2 Golden Vector integrity guard & offload contract for `PKT-406` (`WP-6.1` `Radar`).
//!
//! Active test `test_pkt406_oracle_integrity` verifies the committed C++ Golden Vector oracle
//! (`docs/golden_vectors/PKT-406.json`). For `env:ephemeral-web` offload: create
//! `crates/zh_logic/src/player/radar.rs` + `mod.rs` (exported via `pub mod player;` in
//! `crates/zh_logic/src/lib.rs`), replace the `#[ignore]` test below with the full 1,000-seed
//! assertion loop against `VECTORS_JSON`, remove `#[ignore]`, and run `scripts/port_map.py --write`.

use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32, get_u32_array};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-406.json");

#[test]
fn test_pkt406_oracle_integrity() {
    assert!(VECTORS_JSON.contains("\"packet\": \"PKT-406\""));
    assert!(VECTORS_JSON.contains("\"module\": \"Radar\""));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("Radar.h"));
    assert!(VECTORS_JSON.contains("Radar.cpp"));

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000, "PKT-406 oracle must contain 1,000 seeds");
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(idx as u32));
        assert!(get_u32(line, "xcrc").is_some());
        assert!(get_u32(line, "xbcrc").is_some());
    }

    // Verify Seed-0 self-diagnosing checkpoints exist for single-field mismatch localization.
    let s0 = lines[0];
    let samp = get_u32_array::<2>(s0, "s0_samp").expect("Seed 0 must include s0_samp");
    assert!(samp[0] != 0 && samp[1] != 0);
    assert!(get_str(s0, "s0_xhex").map_or(false, |h| h.len() == 4499 * 2));
    assert!(get_i32(s0, "xlen") == Some(4499));
}

#[test]
#[ignore = "PKT-406 Radar not yet implemented — implement crates/zh_logic/src/player/radar.rs, replace body with 1,000-seed parity assertions over VECTORS_JSON, remove #[ignore], and run python3 scripts/port_map.py --write"]
fn test_pkt406_1000_seed_cpp_parity() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}
