//! `PKT-002`: 6-Word ADC Cascade PRNG (`Core/GameEngine/Source/Common/RandomValue.cpp`).
//!
//! Target module for `WP-1.2a` (`PKT-002`).

use crate::crc::Crc;
use crate::types::{Int, Real, UnsignedInt};

pub const INITIAL_SEED_WORDS: [UnsignedInt; 6] = [
    0xf22d_0e56,
    0x8831_26e9,
    0xc624_dd2f,
    0x0702_c49c,
    0x9e35_3f7d,
    0x6fdf_3b64,
];

pub const THE_MULT_FACTOR: Real = 1.0f32 / (u32::MAX as Real);

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

#[inline]
fn seed_random(seed_val: UnsignedInt, seed: &mut [UnsignedInt; 6]) {
    let mut ax = seed_val;
    ax = ax.wrapping_add(0xf22d_0e56);
    seed[0] = ax;
    ax = ax.wrapping_add(0x8831_26e9_u32.wrapping_sub(0xf22d_0e56));
    seed[1] = ax;
    ax = ax.wrapping_add(0xc624_dd2f_u32.wrapping_sub(0x8831_26e9));
    seed[2] = ax;
    ax = ax.wrapping_add(0x0702_c49c_u32.wrapping_sub(0xc624_dd2f));
    seed[3] = ax;
    ax = ax.wrapping_add(0x9e35_3f7d_u32.wrapping_sub(0x0702_c49c));
    seed[4] = ax;
    ax = ax.wrapping_add(0x6fdf_3b64_u32.wrapping_sub(0x9e35_3f7d));
    seed[5] = ax;
}

#[inline]
fn random_value(seed: &mut [UnsignedInt; 6]) -> UnsignedInt {
    let mut c = 0u32;

    macro_rules! adc {
        ($sum:ident, $a:expr, $b:expr, $c:expr) => {
            let $sum = ($a).wrapping_add($b).wrapping_add($c);
            $c = if $sum < ($a) || $sum < ($b) { 1 } else { 0 };
        };
    }

    adc!(ax0, seed[5], seed[4], c);
    let ax = ax0;
    seed[4] = ax;

    adc!(ax1, ax, seed[3], c);
    let ax = ax1;
    seed[3] = ax;

    adc!(ax2, ax, seed[2], c);
    let ax = ax2;
    seed[2] = ax;

    adc!(ax3, ax, seed[1], c);
    let ax = ax3;
    seed[1] = ax;

    adc!(ax4, ax, seed[0], c);
    let _ = c;
    let mut ax = ax4;
    seed[0] = ax;

    if seed[5] == u32::MAX {
        seed[5] = 0;
        if seed[4] == u32::MAX {
            seed[4] = 0;
            if seed[3] == u32::MAX {
                seed[3] = 0;
                if seed[2] == u32::MAX {
                    seed[2] = 0;
                    if seed[1] == u32::MAX {
                        seed[1] = 0;
                        seed[0] = seed[0].wrapping_add(1);
                        ax = ax.wrapping_add(1);
                    } else {
                        seed[1] = seed[1].wrapping_add(1);
                    }
                } else {
                    seed[2] = seed[2].wrapping_add(1);
                }
            } else {
                seed[3] = seed[3].wrapping_add(1);
            }
        } else {
            seed[4] = seed[4].wrapping_add(1);
        }
    } else {
        seed[5] = seed[5].wrapping_add(1);
    }

    ax
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

    pub fn init_random_with_seed(&mut self, seed: UnsignedInt) {
        seed_random(seed, &mut self.audio_seed);
        seed_random(seed, &mut self.client_seed);
        seed_random(seed, &mut self.logic_seed);
        self.logic_base_seed = seed;
    }

    pub fn get_game_logic_random_seed(&self) -> UnsignedInt {
        self.logic_base_seed
    }

    pub fn get_game_logic_random_seed_crc(&self) -> UnsignedInt {
        let mut c = Crc::new();
        c.compute_crc(
            unsafe {
                core::slice::from_raw_parts(
                    self.logic_seed.as_ptr() as *const u8,
                    core::mem::size_of_val(&self.logic_seed),
                )
            },
            core::mem::size_of_val(&self.logic_seed) as Int,
        );
        c.get()
    }

    pub fn get_game_logic_random_value(&mut self, lo: Int, hi: Int) -> Int {
        #[cfg(feature = "retail-crc")]
        {
            let delta = hi.wrapping_sub(lo).wrapping_add(1) as UnsignedInt;
            if delta == 0 {
                return hi;
            }
            let rval = ((random_value(&mut self.logic_seed) % delta) as Int).wrapping_add(lo);
            rval
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if lo >= hi {
                return hi;
            }
            let delta = hi.wrapping_sub(lo).wrapping_add(1) as UnsignedInt;
            let rval = ((random_value(&mut self.logic_seed) % delta) as Int).wrapping_add(lo);
            rval
        }
    }

    pub fn get_game_logic_random_value_real(&mut self, lo: Real, hi: Real) -> Real {
        #[cfg(feature = "retail-crc")]
        {
            let delta = hi - lo;
            if delta <= 0.0 {
                return hi;
            }
            let rval = ((random_value(&mut self.logic_seed) as Real) * THE_MULT_FACTOR) * delta + lo;
            rval
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if lo >= hi {
                return hi;
            }
            let delta = hi - lo;
            let rval = ((random_value(&mut self.logic_seed) as Real) * THE_MULT_FACTOR) * delta + lo;
            rval
        }
    }

    pub fn get_game_logic_random_value_unchanged(&self, lo: Int, hi: Int) -> Int {
        #[cfg(feature = "retail-crc")]
        {
            let mut temp = self.clone();
            temp.get_game_logic_random_value(lo, hi)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if lo >= hi {
                return hi;
            }
            let mut temp_seed = self.logic_seed;
            let delta = hi.wrapping_sub(lo).wrapping_add(1) as UnsignedInt;
            let rval = ((random_value(&mut temp_seed) % delta) as Int).wrapping_add(lo);
            rval
        }
    }

    pub fn get_game_logic_random_value_real_unchanged(&self, lo: Real, hi: Real) -> Real {
        #[cfg(feature = "retail-crc")]
        {
            let mut temp = self.clone();
            temp.get_game_logic_random_value_real(lo, hi)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if lo >= hi {
                return hi;
            }
            let mut temp_seed = self.logic_seed;
            let delta = hi - lo;
            let rval = ((random_value(&mut temp_seed) as Real) * THE_MULT_FACTOR) * delta + lo;
            rval
        }
    }

    pub fn get_game_client_random_value(&mut self, lo: Int, hi: Int) -> Int {
        if lo >= hi {
            return hi;
        }
        let delta = hi.wrapping_sub(lo).wrapping_add(1) as UnsignedInt;
        let rval = ((random_value(&mut self.client_seed) % delta) as Int).wrapping_add(lo);
        rval
    }

    pub fn get_game_client_random_value_real(&mut self, lo: Real, hi: Real) -> Real {
        if lo >= hi {
            return hi;
        }
        let delta = hi - lo;
        let rval = ((random_value(&mut self.client_seed) as Real) * THE_MULT_FACTOR) * delta + lo;
        rval
    }

    pub fn get_game_audio_random_value(&mut self, lo: Int, hi: Int) -> Int {
        if lo >= hi {
            return hi;
        }
        let delta = hi.wrapping_sub(lo).wrapping_add(1) as UnsignedInt;
        let rval = ((random_value(&mut self.audio_seed) % delta) as Int).wrapping_add(lo);
        rval
    }

    pub fn get_game_audio_random_value_real(&mut self, lo: Real, hi: Real) -> Real {
        if lo >= hi {
            return hi;
        }
        let delta = hi - lo;
        let rval = ((random_value(&mut self.audio_seed) as Real) * THE_MULT_FACTOR) * delta + lo;
        rval
    }
}

static mut GLOBAL_PRNG: PrngState = PrngState::new();

pub fn init_random_time() {
    unsafe {
        *core::ptr::addr_of_mut!(GLOBAL_PRNG) = PrngState::new();
    }
}

pub fn init_random(seed: UnsignedInt) {
    unsafe {
        (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).init_random_with_seed(seed);
    }
}

pub fn get_game_logic_random_seed() -> UnsignedInt {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).get_game_logic_random_seed() }
}

pub fn get_game_logic_random_seed_crc() -> UnsignedInt {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).get_game_logic_random_seed_crc() }
}

pub fn get_game_logic_random_value(lo: Int, hi: Int) -> Int {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).get_game_logic_random_value(lo, hi) }
}

pub fn get_game_logic_random_value_real(lo: Real, hi: Real) -> Real {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).get_game_logic_random_value_real(lo, hi) }
}

pub fn get_game_logic_random_value_unchanged(lo: Int, hi: Int) -> Int {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).get_game_logic_random_value_unchanged(lo, hi) }
}

pub fn get_game_logic_random_value_real_unchanged(lo: Real, hi: Real) -> Real {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).get_game_logic_random_value_real_unchanged(lo, hi) }
}

pub fn get_game_client_random_value(lo: Int, hi: Int) -> Int {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).get_game_client_random_value(lo, hi) }
}

pub fn get_game_client_random_value_real(lo: Real, hi: Real) -> Real {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).get_game_client_random_value_real(lo, hi) }
}

pub fn get_game_audio_random_value(lo: Int, hi: Int) -> Int {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).get_game_audio_random_value(lo, hi) }
}

pub fn get_game_audio_random_value_real(lo: Real, hi: Real) -> Real {
    unsafe { (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).get_game_audio_random_value_real(lo, hi) }
}

pub fn get_logic_state_words() -> [UnsignedInt; 6] {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).logic_seed }
}

pub fn set_logic_state_words(words: [UnsignedInt; 6], base_seed: UnsignedInt) {
    unsafe {
        (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).logic_seed = words;
        (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).logic_base_seed = base_seed;
    }
}

pub fn get_client_state_words() -> [UnsignedInt; 6] {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).client_seed }
}

pub fn set_client_state_words(words: [UnsignedInt; 6]) {
    unsafe {
        (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).client_seed = words;
    }
}

pub fn get_audio_state_words() -> [UnsignedInt; 6] {
    unsafe { (*core::ptr::addr_of!(GLOBAL_PRNG)).audio_seed }
}

pub fn set_audio_state_words(words: [UnsignedInt; 6]) {
    unsafe {
        (*core::ptr::addr_of_mut!(GLOBAL_PRNG)).audio_seed = words;
    }
}
