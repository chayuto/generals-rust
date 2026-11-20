//! Gate V2-Golden test suite for `PKT-064` (`NameKeyGenerator`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-064`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

use zh_common::core::name_key_generator::{
    calc_hash_for_lowercase_string, calc_hash_for_string, NameKeyGenerator, SOCKET_COUNT,
};
use zh_common::golden_json::{extract_vector_lines, get_i32, get_str, get_u32};
use zh_common::types::NameKeyType;

const PKT064_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-064.json");

const SAMPLE_TOKENS: [&str; 16] = [
    "AmericaTankCrusader",
    "americatankcrusader",
    "GLATankScorpion",
    "ChinaTankOverlord",
    "Upgrade_ChinaNuclearMissile",
    "SCIENCE_PaladinTank",
    "Weapon_CrusaderTankGun",
    "Locomotor_FourWheels",
    "ObjectCreationList_Debris",
    "FXList_ExplosionMedium",
    "Data/English/Language9x.ini",
    "Data/Audio/Tracks/GLA_02.mp3",
    "SpecialPower_ParticleUplinkCannon",
    "Armor_CompositeTankArmor",
    "Behavior_AutoHealUpdate",
    "ModuleTag_01",
];

fn splitmix32(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x9e37_79b9);
    let mut z = *state;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

fn rotate_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[test]
#[ignore = "Un-ignore when implementing PKT-064 (crates/zh_common/src/core/name_key_generator.rs)"]
fn test_pkt064_name_key_generator_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT064_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-064.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let mut rng = seed.wrapping_mul(0x85eb_ca6b).wrapping_add(0x1234_5678);
        let r0 = splitmix32(&mut rng);
        assert_eq!(r0, get_u32(line, "r0").unwrap(), "seed {} r0", seed);

        let custom_tok = format!("Tok_{}_{:X}", seed % 137, r0 & 0xFFFF);
        let raw_hash = calc_hash_for_string(&custom_tok);
        let lower_hash = calc_hash_for_lowercase_string(&custom_tok);
        let socket_idx = raw_hash % (SOCKET_COUNT as u32);
        let lower_socket_idx = lower_hash % (SOCKET_COUNT as u32);

        assert_eq!(raw_hash, get_u32(line, "raw_hash").unwrap(), "seed {} raw_hash", seed);
        assert_eq!(
            lower_hash,
            get_u32(line, "lower_hash").unwrap(),
            "seed {} lower_hash",
            seed
        );
        assert_eq!(
            socket_idx,
            get_u32(line, "socket_idx").unwrap(),
            "seed {} socket_idx",
            seed
        );
        assert_eq!(
            lower_socket_idx,
            get_u32(line, "lower_socket_idx").unwrap(),
            "seed {} lower_socket_idx",
            seed
        );

        let mut gen = NameKeyGenerator::new();
        gen.init();

        let mut seq_crc: u32 = 0;
        let mut last_key = NameKeyType::INVALID;
        for step in 0..12 {
            let op = splitmix32(&mut rng);
            let tok = if step == 5 || step == 10 {
                custom_tok.as_str()
            } else {
                SAMPLE_TOKENS[(op & 15) as usize]
            };
            let k = if (op & 0x100) != 0 {
                gen.name_to_lowercase_key(tok)
            } else {
                gen.name_to_key(tok)
            };
            seq_crc = rotate_crc(seq_crc, k.0 as u32);
            last_key = k;
        }

        assert_eq!(seq_crc, get_u32(line, "seq_crc").unwrap(), "seed {} seq_crc", seed);
        assert_eq!(
            last_key.0,
            get_i32(line, "last_key").unwrap(),
            "seed {} last_key",
            seed
        );
        assert_eq!(
            gen.key_to_name(last_key),
            get_str(line, "recovered").unwrap(),
            "seed {} recovered",
            seed
        );
    }
}
