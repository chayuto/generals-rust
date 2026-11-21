//! Gate V2-Golden test suite for `PKT-038` (`Dict`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-038`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

use zh_common::core::dict::{Dict, DictDataType};
use zh_common::golden_json::{extract_vector_lines, get_i32, get_u32};
use zh_common::types::{Bool, NameKeyType, Real, FALSE, TRUE};

const PKT038_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-038.json");

fn splitmix32(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x9e37_79b9);
    let mut z = *state;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[test]
fn test_pkt038_dict_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT038_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-038.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let mut rng = seed.wrapping_mul(0x94d0_49bb).wrapping_add(0x2468_ACE0);
        let mut d = Dict::new();
        let mut d_other = Dict::new();

        if seed == 0 {
            d.set_int(NameKeyType(5), 42);
            d.clear();
        } else {
            for step in 0..16 {
                let op = splitmix32(&mut rng);
                let key = NameKeyType(((op % 12) + 1) as i32);
                match (op >> 8) % 7 {
                    0 => {
                        let b = if ((op >> 16) & 1) != 0 { TRUE } else { FALSE };
                        d.set_bool(key, b);
                    }
                    1 => {
                        let v = ((op >> 4) as i32).wrapping_sub(50000);
                        d.set_int(key, v);
                    }
                    2 => {
                        let rv = ((((op >> 4) % 10001) as i32) - 5000) as Real * 0.125;
                        d.set_real(key, rv);
                    }
                    3 => {
                        let s = format!("V_{}_{}", key.0, step);
                        d.set_ascii_string(key, &s);
                    }
                    4 => {
                        let w = [
                            b'U' as u16,
                            (b'0' as i32 + (key.0 % 10)) as u16,
                            (0x0410 + step) as u16,
                        ];
                        d.set_unicode_string(key, &w);
                    }
                    5 => {
                        d.remove(key);
                    }
                    _ => {
                        if (op & 0x200) != 0 {
                            d_other.set_int(
                                key,
                                seed.wrapping_mul(100).wrapping_add(step as u32) as i32,
                            );
                        } else {
                            d_other.remove(key);
                        }
                        d.copy_pair_from(&d_other, key);
                    }
                }
            }
        }

        let mut d_clone = d.clone();
        d_clone.set_int(NameKeyType(13), seed as i32);

        let pair_count = d.get_pair_count();
        assert_eq!(pair_count, get_i32(line, "pair_count").unwrap(), "seed {seed} pair_count");
        assert_eq!(d_clone.get_pair_count(), get_i32(line, "clone_count").unwrap(), "seed {seed} clone_count");

        let mut state_crc = 0u32;
        for i in 0..pair_count {
            let k = d.get_nth_key(i);
            let dt = d.get_nth_type(i);
            state_crc = rot_crc(state_crc, k.0 as u32);
            state_crc = rot_crc(state_crc, dt.as_i32() as u32);
            match dt {
                DictDataType::Bool => {
                    state_crc = rot_crc(state_crc, d.get_nth_bool(i) as u32);
                }
                DictDataType::Int => {
                    state_crc = rot_crc(state_crc, d.get_nth_int(i) as u32);
                }
                DictDataType::Real => {
                    state_crc = rot_crc(state_crc, d.get_nth_real(i).to_bits());
                }
                DictDataType::AsciiString => {
                    let s = d.get_nth_ascii_string(i);
                    state_crc = rot_crc(state_crc, s.len() as u32);
                    for &b in s.as_bytes() {
                        state_crc = rot_crc(state_crc, b as u32);
                    }
                }
                DictDataType::UnicodeString => {
                    let u = d.get_nth_unicode_string(i);
                    state_crc = rot_crc(state_crc, u.len() as u32);
                    for &unit in &u {
                        state_crc = rot_crc(state_crc, unit as u32);
                    }
                }
                DictDataType::None => {}
            }
        }
        assert_eq!(state_crc, get_u32(line, "state_crc").unwrap(), "seed {seed} state_crc");

        let probe_key = NameKeyType(get_i32(line, "probe_key").unwrap());
        let want_type = DictDataType::from_i32(get_i32(line, "probe_type").unwrap());
        assert_eq!(d.get_type(probe_key), want_type, "seed {seed} probe_type");
        assert_eq!(
            d.known(probe_key, want_type) as u32,
            if want_type != DictDataType::None { 1 } else { 0 },
            "seed {seed} known"
        );
        let mut exists: Bool = 0;
        let probe_int = d.get_int(probe_key, Some(&mut exists));
        assert_eq!(probe_int, get_i32(line, "probe_int").unwrap(), "seed {seed} probe_int");
        assert_eq!(exists as u32, get_u32(line, "probe_exists").unwrap(), "seed {seed} probe_exists");
    }
}
