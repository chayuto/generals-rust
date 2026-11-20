//! Gate V2-Golden test suite for `PKT-004` (`BitFlags`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-004`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32};
use zh_common::xfer::bit_flags::{ArmorSetFlags, BitFlags, ARMOR_SET_BIT_NAMES};

const PKT004_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-004.json");

#[test]
fn test_pkt004_bitflags_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT004_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-004.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let w0_a = get_u32(line, "w0_a").expect("w0_a");
        let w1_a = get_u32(line, "w1_a").expect("w1_a");
        let w2_a = get_u32(line, "w2_a").expect("w2_a");
        let w0_b = get_u32(line, "w0_b").expect("w0_b");
        let w1_b = get_u32(line, "w1_b").expect("w1_b");
        let w2_b = get_u32(line, "w2_b").expect("w2_b");

        let mut big_a = BitFlags::<96>::new();
        let mut big_b = BitFlags::<96>::new();
        for i in 0..32 {
            if ((w0_a >> i) & 1) != 0 {
                big_a.set(i, 1);
            }
            if ((w1_a >> i) & 1) != 0 {
                big_a.set(32 + i, 1);
            }
            if ((w2_a >> i) & 1) != 0 {
                big_a.set(64 + i, 1);
            }
            if ((w0_b >> i) & 1) != 0 {
                big_b.set(i, 1);
            }
            if ((w1_b >> i) & 1) != 0 {
                big_b.set(32 + i, 1);
            }
            if ((w2_b >> i) & 1) != 0 {
                big_b.set(64 + i, 1);
            }
        }
        if seed == 0 {
            big_a.clear();
            big_b.clear();
        } else if seed == 1 {
            big_a = BitFlags::<96>::new_set_all();
        }

        assert_eq!(big_a.count(), get_i32(line, "count_a").unwrap(), "seed {} count_a", seed);
        assert_eq!(
            big_a.count_intersection(&big_b),
            get_i32(line, "count_inter").unwrap(),
            "seed {} count_inter",
            seed
        );
        assert_eq!(
            big_a.count_inverse_intersection(&big_b),
            get_i32(line, "count_inv_inter").unwrap(),
            "seed {} count_inv_inter",
            seed
        );
        assert_eq!(
            big_a.any_intersection_with(&big_b) as u32,
            get_u32(line, "any_inter").unwrap(),
            "seed {} any_inter",
            seed
        );
        assert_eq!(
            big_a.test_for_all(&big_b) as u32,
            get_u32(line, "test_all").unwrap(),
            "seed {} test_all",
            seed
        );
        assert_eq!(
            big_a.test_for_none(&big_b) as u32,
            get_u32(line, "test_none").unwrap(),
            "seed {} test_none",
            seed
        );
        assert_eq!(
            big_a.to_unsigned_int(),
            get_u32(line, "u32_a").unwrap(),
            "seed {} u32_a",
            seed
        );
        assert_eq!(
            big_a.to_hex_string(),
            get_str(line, "hex_a").unwrap(),
            "seed {} hex_a",
            seed
        );

        let mut mutated = big_a.clone();
        mutated.clear_and_set(&big_b, &BitFlags::<96>::from_single_index((seed % 96) as i32));
        assert_eq!(
            mutated.to_hex_string(),
            get_str(line, "hex_mut").unwrap(),
            "seed {} hex_mut",
            seed
        );

        let armor_name =
            ArmorSetFlags::get_name_from_single_bit(&ARMOR_SET_BIT_NAMES, ((seed % 10) as i32) - 1);
        let lookup_idx = ArmorSetFlags::get_single_bit_from_name(
            &ARMOR_SET_BIT_NAMES,
            armor_name.unwrap_or("NONEXISTENT"),
        );
        assert_eq!(
            lookup_idx,
            get_i32(line, "lookup_idx").unwrap(),
            "seed {} lookup_idx",
            seed
        );
    }
}
