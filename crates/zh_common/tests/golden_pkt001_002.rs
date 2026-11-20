//! Gate V2-Golden Test Oracle for `PKT-001` (`crc`) and `PKT-002` (`RandomValue`).
//! Verifies 1,000 C++-recorded Golden Test Vectors per module.

use zh_common::crc::Crc;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_hex_u32, get_i32, get_str, get_u32, get_u32_array,
};
use zh_common::random_value::PrngState;

const PKT001_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-001.json");
const PKT002_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-002.json");

#[test]
fn gate_v2_golden_pkt001_crc_1000_vectors() {
    let lines = extract_vector_lines(PKT001_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-001.json");

    for line in lines {
        let seed = get_i32(line, "seed").expect("seed");
        let chunk1 = decode_hex_bytes(get_str(line, "chunk1_hex").expect("chunk1_hex")).expect("hex1");
        let passed_len1 = get_i32(line, "passed_len1").expect("passed_len1");
        let chunk2 = decode_hex_bytes(get_str(line, "chunk2_hex").expect("chunk2_hex")).expect("hex2");
        let expected_crc1 = get_hex_u32(line, "expected_crc1").expect("expected_crc1");
        let expected_crc2 = get_hex_u32(line, "expected_crc2").expect("expected_crc2");

        let mut c = Crc::new();
        c.compute_crc(&chunk1, passed_len1);
        assert_eq!(
            c.get(),
            expected_crc1,
            "PKT-001 CRC mismatch after chunk1 at seed {}: got 0x{:08X}, expected 0x{:08X}",
            seed,
            c.get(),
            expected_crc1
        );

        c.compute_crc(&chunk2, chunk2.len() as i32);
        assert_eq!(
            c.get(),
            expected_crc2,
            "PKT-001 CRC mismatch after chunk2 at seed {}: got 0x{:08X}, expected 0x{:08X}",
            seed,
            c.get(),
            expected_crc2
        );
    }
}

#[test]
fn gate_v2_golden_pkt002_random_value_1000_vectors() {
    let lines = extract_vector_lines(PKT002_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-002.json");

    for line in lines {
        let seed_index = get_i32(line, "seed_index").expect("seed_index");
        let init_seed = get_u32(line, "init_seed").expect("init_seed");
        let lo_i = get_i32(line, "lo_i").expect("lo_i");
        let hi_i = get_i32(line, "hi_i").expect("hi_i");
        let lo_f = f32::from_bits(get_hex_u32(line, "lo_f_bits").expect("lo_f_bits"));
        let hi_f = f32::from_bits(get_hex_u32(line, "hi_f_bits").expect("hi_f_bits"));

        let init_crc = get_hex_u32(line, "init_crc").expect("init_crc");
        let unch_i = get_i32(line, "unch_i").expect("unch_i");
        let unch_f_bits = get_hex_u32(line, "unch_f_bits").expect("unch_f_bits");
        let crc_after_unch = get_hex_u32(line, "crc_after_unch").expect("crc_after_unch");
        let logic_i1 = get_i32(line, "logic_i1").expect("logic_i1");
        let crc_after_i1 = get_hex_u32(line, "crc_after_i1").expect("crc_after_i1");
        let logic_f1_bits = get_hex_u32(line, "logic_f1_bits").expect("logic_f1_bits");
        let logic_i2 = get_i32(line, "logic_i2").expect("logic_i2");
        let logic_f2_bits = get_hex_u32(line, "logic_f2_bits").expect("logic_f2_bits");
        let final_crc = get_hex_u32(line, "final_crc").expect("final_crc");
        let client_i = get_i32(line, "client_i").expect("client_i");
        let client_f_bits = get_hex_u32(line, "client_f_bits").expect("client_f_bits");
        let audio_i = get_i32(line, "audio_i").expect("audio_i");
        let audio_f_bits = get_hex_u32(line, "audio_f_bits").expect("audio_f_bits");
        let expected_words = get_u32_array::<6>(line, "state_words").expect("state_words");

        let mut rng = PrngState::new();
        rng.init_random_with_seed(init_seed);
        assert_eq!(rng.get_game_logic_random_seed(), init_seed);
        assert_eq!(
            rng.get_game_logic_random_seed_crc(),
            init_crc,
            "PKT-002 init_crc mismatch at seed_index {}",
            seed_index
        );

        assert_eq!(
            rng.get_game_logic_random_value_unchanged(lo_i, hi_i),
            unch_i,
            "PKT-002 unch_i mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_value_real_unchanged(lo_f, hi_f).to_bits(),
            unch_f_bits,
            "PKT-002 unch_f_bits mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_seed_crc(),
            crc_after_unch,
            "PKT-002 crc_after_unch mismatch at seed_index {}",
            seed_index
        );

        assert_eq!(
            rng.get_game_logic_random_value(lo_i, hi_i),
            logic_i1,
            "PKT-002 logic_i1 mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_seed_crc(),
            crc_after_i1,
            "PKT-002 crc_after_i1 mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_value_real(lo_f, hi_f).to_bits(),
            logic_f1_bits,
            "PKT-002 logic_f1_bits mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_value(-100, 100),
            logic_i2,
            "PKT-002 logic_i2 mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_value_real(0.0, 1.0).to_bits(),
            logic_f2_bits,
            "PKT-002 logic_f2_bits mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_logic_random_seed_crc(),
            final_crc,
            "PKT-002 final_crc mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.logic_seed, expected_words,
            "PKT-002 logic_seed words mismatch at seed_index {}",
            seed_index
        );

        assert_eq!(
            rng.get_game_client_random_value(lo_i, hi_i),
            client_i,
            "PKT-002 client_i mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_client_random_value_real(lo_f, hi_f).to_bits(),
            client_f_bits,
            "PKT-002 client_f_bits mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_audio_random_value(lo_i, hi_i),
            audio_i,
            "PKT-002 audio_i mismatch at seed_index {}",
            seed_index
        );
        assert_eq!(
            rng.get_game_audio_random_value_real(lo_f, hi_f).to_bits(),
            audio_f_bits,
            "PKT-002 audio_f_bits mismatch at seed_index {}",
            seed_index
        );
    }
}
