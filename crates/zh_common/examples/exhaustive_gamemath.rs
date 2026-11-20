//! Exhaustive 2^32-input digest for the pure-Rust `GameMath` port (`WP-2.2a`).
//!
//! Counterpart of `tests/cpp_harness/exhaustive_gamemath.c` (linked against the reference
//! `libgm.a`). Both print `<fn> <fnv1a64-digest>`; identical digests prove bit-exact parity
//! over every `f32` input (NaN results canonicalized to `0x7fc00000`).
//!
//! Usage: `cargo run --release -p zh_common --example exhaustive_gamemath -- sinf`

use zh_common::math::{
    gm_acosf, gm_asinf, gm_atan2f, gm_atanf, gm_ceilf, gm_cosf, gm_fabsf, gm_floorf, gm_rintf,
    gm_roundf, gm_sinf, gm_sqrtf, gm_tanf,
};

fn lookup(name: &str) -> Option<fn(f32) -> f32> {
    Some(match name {
        "sinf" => gm_sinf,
        "cosf" => gm_cosf,
        "tanf" => gm_tanf,
        "asinf" => gm_asinf,
        "acosf" => gm_acosf,
        "atanf" => gm_atanf,
        "sqrtf" => |x| gm_sqrtf(gm_fabsf(x)),
        "floorf" => gm_floorf,
        "ceilf" => gm_ceilf,
        "rintf" => gm_rintf,
        "roundf" => gm_roundf,
        "atan2f_y1" => |x| gm_atan2f(1.0, x),
        "atan2f_xm1" => |y| gm_atan2f(y, -1.0),
        _ => return None,
    })
}

fn main() {
    let name = std::env::args().nth(1).expect("usage: exhaustive_gamemath <fn>");
    let f = lookup(&name).unwrap_or_else(|| panic!("unknown fn {name}"));
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut b: u32 = 0;
    loop {
        let mut rb = f(f32::from_bits(b)).to_bits();
        if (rb & 0x7f80_0000) == 0x7f80_0000 && (rb & 0x007f_ffff) != 0 {
            rb = 0x7fc0_0000;
        }
        h ^= rb as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
        b = b.wrapping_add(1);
        if b == 0 {
            break;
        }
    }
    println!("{name} {h:016x}");
}
