//! `PKT-002`: 6-Word ADC Cascade PRNG (`Core/GameEngine/Source/Common/RandomValue.cpp`).
//!
//! Target module for `WP-1.2a` (`PKT-002`).

use crate::types::{Int, Real, UnsignedInt};

pub const INITIAL_SEED_WORDS: [UnsignedInt; 6] = [
    0xf22d_0e56,
    0x8831_26e9,
    0xc624_dd2f,
    0x0702_c49c,
    0x9e35_3f7d,
    0x6fdf_3b64,
];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DistributionType {
    #[default]
    Constant = 0,
    Uniform = 1,
    Gaussian = 2,
    Triangular = 3,
    LowBias = 4,
    HighBias = 5,
    Count = 6,
}

/// Complete deterministic 3-stream PRNG state (`theGameLogicSeed`, `theGameClientSeed`, `theGameAudioSeed`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrngState {
    pub audio_seed: [UnsignedInt; 6],
    pub client_seed: [UnsignedInt; 6],
    pub logic_seed: [UnsignedInt; 6],
    pub logic_base_seed: UnsignedInt,
}

impl Default for PrngState {
    fn default() -> Self {
        Self::new()
    }
}

impl PrngState {
    #[inline]
    pub const fn new() -> Self {
        Self {
            audio_seed: INITIAL_SEED_WORDS,
            client_seed: INITIAL_SEED_WORDS,
            logic_seed: INITIAL_SEED_WORDS,
            logic_base_seed: 0,
        }
    }

    pub fn init_random_with_seed(&mut self, _seed: UnsignedInt) {
        // Pending PKT-002 translation in WP-1.2a
    }

    pub fn get_game_logic_random_seed(&self) -> UnsignedInt {
        self.logic_base_seed
    }

    pub fn get_game_logic_random_seed_crc(&self) -> UnsignedInt {
        // Pending PKT-002 translation in WP-1.2a
        0
    }

    pub fn get_game_logic_random_value(&mut self, lo: Int, hi: Int) -> Int {
        let _ = lo;
        hi
    }

    pub fn get_game_logic_random_value_real(&mut self, lo: Real, hi: Real) -> Real {
        let _ = lo;
        hi
    }

    pub fn get_game_logic_random_value_unchanged(&self, lo: Int, hi: Int) -> Int {
        let _ = lo;
        hi
    }

    pub fn get_game_logic_random_value_real_unchanged(&self, lo: Real, hi: Real) -> Real {
        let _ = lo;
        hi
    }

    pub fn get_game_client_random_value(&mut self, lo: Int, hi: Int) -> Int {
        let _ = lo;
        hi
    }

    pub fn get_game_client_random_value_real(&mut self, lo: Real, hi: Real) -> Real {
        let _ = lo;
        hi
    }

    pub fn get_game_audio_random_value(&mut self, lo: Int, hi: Int) -> Int {
        let _ = lo;
        hi
    }

    pub fn get_game_audio_random_value_real(&mut self, lo: Real, hi: Real) -> Real {
        let _ = lo;
        hi
    }
}

pub fn init_random_time() {}
pub fn init_random(seed: UnsignedInt) {
    let _ = seed;
}
pub fn get_game_logic_random_seed() -> UnsignedInt {
    0
}
pub fn get_game_logic_random_seed_crc() -> UnsignedInt {
    0
}
pub fn get_game_logic_random_value(lo: Int, hi: Int) -> Int {
    let _ = lo;
    hi
}
pub fn get_game_logic_random_value_real(lo: Real, hi: Real) -> Real {
    let _ = lo;
    hi
}
pub fn get_game_logic_random_value_unchanged(lo: Int, hi: Int) -> Int {
    let _ = lo;
    hi
}
pub fn get_game_logic_random_value_real_unchanged(lo: Real, hi: Real) -> Real {
    let _ = lo;
    hi
}
pub fn get_game_client_random_value(lo: Int, hi: Int) -> Int {
    let _ = lo;
    hi
}
pub fn get_game_client_random_value_real(lo: Real, hi: Real) -> Real {
    let _ = lo;
    hi
}
pub fn get_game_audio_random_value(lo: Int, hi: Int) -> Int {
    let _ = lo;
    hi
}
pub fn get_game_audio_random_value_real(lo: Real, hi: Real) -> Real {
    let _ = lo;
    hi
}
pub fn get_logic_state_words() -> [UnsignedInt; 6] {
    INITIAL_SEED_WORDS
}
pub fn set_logic_state_words(_words: [UnsignedInt; 6], _base_seed: UnsignedInt) {}
pub fn get_client_state_words() -> [UnsignedInt; 6] {
    INITIAL_SEED_WORDS
}
pub fn set_client_state_words(_words: [UnsignedInt; 6]) {}
pub fn get_audio_state_words() -> [UnsignedInt; 6] {
    INITIAL_SEED_WORDS
}
pub fn set_audio_state_words(_words: [UnsignedInt; 6]) {}
