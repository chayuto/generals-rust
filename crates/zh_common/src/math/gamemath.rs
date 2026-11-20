//! Pure-Rust, bit-exact port of the `GameMath` (TheSuperHackers/GameMath, SunPro fdlibm)
//! single-precision transcendental functions used by `Trig.cpp` / `WWMath` under
//! `USE_DETERMINISTIC_MATH` (`WP-2.2a`).
//!
//! Source of truth: `vendor/GeneralsX/build/<preset>/_deps/gamemath-src/{float,double}/*.c`
//! (Copyright (C) 1993 by Sun Microsystems, Inc. — "Permission to use, copy, modify, and
//! distribute this software is freely granted, provided that this notice is preserved.").
//!
//! Translation rules:
//! - Only IEEE-754 `+ - * /` and integer bit manipulation are used. Rust never contracts
//!   `a*b+c` into FMA, matching `libgm.a` which is built with `-ffp-contract=off`.
//! - `GET_FLOAT_WORD` -> `to_bits() as i32`; `SET_FLOAT_WORD` -> `f32::from_bits(w as u32)`.
//! - C `(int32_t)f` conversions are only applied on in-range values in fdlibm, so Rust `as i32`
//!   (saturating) is equivalent.
//! - NaN *payload/sign* produced by `x - x` on Inf differs between x86 (`0xffc00000`) and ARM
//!   (`0x7fc00000`) for the C reference as well; Golden tests therefore compare NaN-ness only.

use super::{gm_ceilf, gm_floorf, gm_sqrtf};

#[inline]
fn word(x: f32) -> i32 {
    x.to_bits() as i32
}

#[inline]
fn from_word(w: i32) -> f32 {
    f32::from_bits(w as u32)
}

/// `gm_fabsf` (`s_fabsf.c`).
#[inline]
pub fn gm_fabsf(x: f32) -> f32 {
    f32::from_bits(x.to_bits() & 0x7fff_ffff)
}

/// `gm_copysignf` (`s_copysignf.c`).
#[inline]
pub fn gm_copysignf(x: f32, y: f32) -> f32 {
    f32::from_bits((x.to_bits() & 0x7fff_ffff) | (y.to_bits() & 0x8000_0000))
}

/// `gm_scalbnf` / `gm_ldexpf` (`s_scalbnf.c`).
pub fn gm_scalbnf(mut x: f32, n: i32) -> f32 {
    const TWO25: f32 = 3.355_443_200e+07;
    const TWOM25: f32 = 2.980_232_238_8e-08;
    const HUGE: f32 = 1.0e+30;
    const TINY: f32 = 1.0e-30;
    let mut ix = word(x);
    let mut k = (ix & 0x7f80_0000) >> 23;
    if k == 0 {
        if (ix & 0x7fff_ffff) == 0 {
            return x;
        }
        x *= TWO25;
        ix = word(x);
        k = ((ix & 0x7f80_0000) >> 23) - 25;
        if n < -50000 {
            return TINY * x;
        }
    }
    if k == 0xff {
        return x + x;
    }
    k = k.wrapping_add(n);
    if k > 0xfe {
        return HUGE * gm_copysignf(HUGE, x);
    }
    if k > 0 {
        return from_word((ix & (0x807f_ffffu32 as i32)) | (k << 23));
    }
    if k <= -25 {
        if n > 50000 {
            return HUGE * gm_copysignf(HUGE, x);
        } else {
            return TINY * gm_copysignf(TINY, x);
        }
    }
    k += 25;
    x = from_word((ix & (0x807f_ffffu32 as i32)) | (k << 23));
    x * TWOM25
}

/// `gm_rintf` (`s_rintf.c`), round-to-nearest-even via the `2^23` trick.
pub fn gm_rintf(x: f32) -> f32 {
    const TWO23: [f32; 2] = [8.388_608_000_0e+06, -8.388_608_000_0e+06];
    let i0 = word(x);
    let sx = ((i0 >> 31) & 1) as usize;
    let j0 = ((i0 >> 23) & 0xff) - 0x7f;
    if j0 < 23 {
        if j0 < 0 {
            if (i0 & 0x7fff_ffff) == 0 {
                return x;
            }
            let w = TWO23[sx] + x;
            return w - TWO23[sx];
        }
        let w = TWO23[sx] + x;
        return w - TWO23[sx];
    }
    if j0 == 0x80 {
        x + x
    } else {
        x
    }
}

/// `gm_roundf` (`s_roundf.c`), round half away from zero.
pub fn gm_roundf(x: f32) -> f32 {
    let hx = x.to_bits();
    if (hx & 0x7fff_ffff) == 0x7f80_0000 {
        return x + x;
    }
    if (hx & 0x8000_0000) == 0 {
        let mut t = gm_floorf(x);
        if t - x <= -0.5 {
            t += 1.0;
        }
        t
    } else {
        let mut t = gm_floorf(-x);
        if t + x <= -0.5 {
            t += 1.0;
        }
        -t
    }
}

// ------------------------------------------------------------------------------------------
// Kernels (k_sinf.c, k_cosf.c, k_tanf.c)
// ------------------------------------------------------------------------------------------

fn kernel_sinf(x: f32, y: f32, iy: i32) -> f32 {
    const HALF: f32 = 0.5;
    const S1: f32 = f32::from_bits(0xbe2a_aaab); // -0xaaaaab.0p-26
    const S2: f32 = f32::from_bits(0x3c08_88ba); //  0x8888ba.0p-30
    const S3: f32 = f32::from_bits(0xb950_2cb0); // -0xd02cb0.0p-36
    const S4: f32 = f32::from_bits(0x363e_18ff); //  0xbe18ff.0p-42
    let z = x * x;
    let v = z * x;
    let r = S2 + z * (S3 + z * S4);
    if iy == 0 {
        x + v * (S1 + z * r)
    } else {
        x - ((z * (HALF * y - v * r) - y) - v * S1)
    }
}

fn kernel_cosf(x: f32, y: f32) -> f32 {
    const ONE: f32 = 1.0;
    const C1: f32 = f32::from_bits(0x3d2a_aaa5); //  0xaaaaa5.0p-28
    const C2: f32 = f32::from_bits(0xbab6_0615); // -0xb60615.0p-33
    const C3: f32 = f32::from_bits(0x37cc_f47d); //  0xccf47d.0p-39
    let z = x * x;
    let r = z * (C1 + z * (C2 + z * C3));
    let hz = 0.5_f32 * z;
    let w = ONE - hz;
    w + (((ONE - w) - hz) + (z * r - x * y))
}

fn kernel_tanf(mut x: f32, mut y: f32, iy: i32) -> f32 {
    const PIO4: f32 = 7.853_981_256_5e-01;
    const PIO4LO: f32 = 3.774_894_707_9e-08;
    const T: [f32; 13] = [
        3.333_333_432_7e-01,
        1.333_333_402_9e-01,
        5.396_825_447_7e-02,
        2.186_948_806_0e-02,
        8.863_239_549_1e-03,
        3.592_079_039_7e-03,
        1.456_209_458_4e-03,
        5.880_412_645_6e-04,
        2.464_631_397_7e-04,
        7.817_944_424_5e-05,
        7.140_725_210_8e-05,
        -1.855_863_774_8e-05,
        2.590_730_582_6e-05,
    ];
    let hx = word(x);
    let ix = hx & 0x7fff_ffff;
    if ix >= 0x3f2c_a140 {
        if hx < 0 {
            x = -x;
            y = -y;
        }
        let z = PIO4 - x;
        let w = PIO4LO - y;
        x = z + w;
        y = 0.0;
    }
    let z = x * x;
    let mut w = z * z;
    let mut r = T[1] + w * (T[3] + w * T[5]);
    let mut v = z * (T[2] + w * T[4]);
    let mut s = z * x;
    r = y + z * (s * (r + v) + y);
    r += T[0] * s;
    w = x + r;
    if ix >= 0x3f2c_a140 {
        v = iy as f32;
        return ((1 - ((hx >> 30) & 2)) as f32) * (v - 2.0_f32 * (x - (w * w / (w + v) - r)));
    }
    if iy == 1 {
        w
    } else {
        let mut z = w;
        let i = word(z);
        z = from_word(i & (0xffff_f000u32 as i32));
        v = r - (z - x);
        let a = -1.0_f32 / w;
        let mut t = a;
        let i = word(t);
        t = from_word(i & (0xffff_f000u32 as i32));
        s = 1.0_f32 + t * z;
        t + a * (s + t * v)
    }
}

// ------------------------------------------------------------------------------------------
// Argument reduction (k_rem_pio2f.c, e_rem_pio2f.c)
// ------------------------------------------------------------------------------------------

const IPIO2: [i32; 198] = [
    0xA2, 0xF9, 0x83, 0x6E, 0x4E, 0x44, 0x15, 0x29, 0xFC, 0x27, 0x57, 0xD1, 0xF5, 0x34, 0xDD,
    0xC0, 0xDB, 0x62, 0x95, 0x99, 0x3C, 0x43, 0x90, 0x41, 0xFE, 0x51, 0x63, 0xAB, 0xDE, 0xBB,
    0xC5, 0x61, 0xB7, 0x24, 0x6E, 0x3A, 0x42, 0x4D, 0xD2, 0xE0, 0x06, 0x49, 0x2E, 0xEA, 0x09,
    0xD1, 0x92, 0x1C, 0xFE, 0x1D, 0xEB, 0x1C, 0xB1, 0x29, 0xA7, 0x3E, 0xE8, 0x82, 0x35, 0xF5,
    0x2E, 0xBB, 0x44, 0x84, 0xE9, 0x9C, 0x70, 0x26, 0xB4, 0x5F, 0x7E, 0x41, 0x39, 0x91, 0xD6,
    0x39, 0x83, 0x53, 0x39, 0xF4, 0x9C, 0x84, 0x5F, 0x8B, 0xBD, 0xF9, 0x28, 0x3B, 0x1F, 0xF8,
    0x97, 0xFF, 0xDE, 0x05, 0x98, 0x0F, 0xEF, 0x2F, 0x11, 0x8B, 0x5A, 0x0A, 0x6D, 0x1F, 0x6D,
    0x36, 0x7E, 0xCF, 0x27, 0xCB, 0x09, 0xB7, 0x4F, 0x46, 0x3F, 0x66, 0x9E, 0x5F, 0xEA, 0x2D,
    0x75, 0x27, 0xBA, 0xC7, 0xEB, 0xE5, 0xF1, 0x7B, 0x3D, 0x07, 0x39, 0xF7, 0x8A, 0x52, 0x92,
    0xEA, 0x6B, 0xFB, 0x5F, 0xB1, 0x1F, 0x8D, 0x5D, 0x08, 0x56, 0x03, 0x30, 0x46, 0xFC, 0x7B,
    0x6B, 0xAB, 0xF0, 0xCF, 0xBC, 0x20, 0x9A, 0xF4, 0x36, 0x1D, 0xA9, 0xE3, 0x91, 0x61, 0x5E,
    0xE6, 0x1B, 0x08, 0x65, 0x99, 0x85, 0x5F, 0x14, 0xA0, 0x68, 0x40, 0x8D, 0xFF, 0xD8, 0x80,
    0x4D, 0x73, 0x27, 0x31, 0x06, 0x06, 0x15, 0x56, 0xCA, 0x73, 0xA8, 0xC9, 0x60, 0xE2, 0x7B,
    0xC0, 0x8C, 0x6B,
];

const PIO2_TABLE: [f32; 11] = [
    1.570_312_500_0e+00,
    4.577_636_718_8e-04,
    2.598_762_512_2e-05,
    7.543_712_854_4e-08,
    6.002_665_031_7e-11,
    7.389_644_451_9e-13,
    5.384_581_669_4e-15,
    5.637_851_296_9e-18,
    8.300_922_883_1e-20,
    3.275_635_225_7e-22,
    6.333_101_564_9e-25,
];

/// `__kernel_rem_pio2f(x, y, e0, nx, prec = 2)` — only `prec == 2` is used by `e_rem_pio2f.c`.
fn kernel_rem_pio2f(x: &[f32; 3], y: &mut [f32; 2], e0: i32, nx: i32) -> i32 {
    const TWO8: f32 = 2.560_000_000_0e+02;
    const TWON8: f32 = 3.906_250_000_0e-03;
    const JK: i32 = 9; // init_jk[2]
    let jp = JK;

    let mut iq = [0i32; 20];
    let mut f = [0f32; 20];
    let mut fq = [0f32; 20];
    let mut q = [0f32; 20];

    let jx = nx - 1;
    let mut jv = (e0 - 3) / 8;
    if jv < 0 {
        jv = 0;
    }
    let mut q0 = e0 - 8 * (jv + 1);

    let mut j = jv - jx;
    let m = jx + JK;
    let mut i = 0;
    while i <= m {
        f[i as usize] = if j < 0 { 0.0 } else { IPIO2[j as usize] as f32 };
        i += 1;
        j += 1;
    }

    for i in 0..=JK {
        let mut fw = 0.0_f32;
        for j in 0..=jx {
            fw += x[j as usize] * f[(jx + i - j) as usize];
        }
        q[i as usize] = fw;
    }

    let mut jz = JK;
    let mut z;
    let mut n;
    let mut ih;
    loop {
        // distill q[] into iq[] reversingly
        let mut i = 0i32;
        let mut j = jz;
        z = q[jz as usize];
        while j > 0 {
            let fw = ((TWON8 * z) as i32) as f32;
            iq[i as usize] = (z - TWO8 * fw) as i32;
            z = q[(j - 1) as usize] + fw;
            i += 1;
            j -= 1;
        }

        // compute n
        z = gm_scalbnf(z, q0);
        z -= 8.0_f32 * gm_floorf(z * 0.125_f32);
        n = z as i32;
        z -= n as f32;
        ih = 0;
        if q0 > 0 {
            let idx = (jz - 1) as usize;
            let i = iq[idx] >> (8 - q0);
            n += i;
            iq[idx] -= i << (8 - q0);
            ih = iq[idx] >> (7 - q0);
        } else if q0 == 0 {
            ih = iq[(jz - 1) as usize] >> 7;
        } else if z >= 0.5_f32 {
            ih = 2;
        }

        if ih > 0 {
            n += 1;
            let mut carry = 0;
            for i in 0..jz {
                let j = iq[i as usize];
                if carry == 0 {
                    if j != 0 {
                        carry = 1;
                        iq[i as usize] = 0x100 - j;
                    }
                } else {
                    iq[i as usize] = 0xff - j;
                }
            }
            if q0 > 0 {
                match q0 {
                    1 => iq[(jz - 1) as usize] &= 0x7f,
                    2 => iq[(jz - 1) as usize] &= 0x3f,
                    _ => {}
                }
            }
            if ih == 2 {
                z = 1.0_f32 - z;
                if carry != 0 {
                    z -= gm_scalbnf(1.0, q0);
                }
            }
        }

        // check if recomputation is needed
        if z == 0.0 {
            let mut jj = 0;
            let mut i = jz - 1;
            while i >= JK {
                jj |= iq[i as usize];
                i -= 1;
            }
            if jj == 0 {
                let mut k = 1;
                while iq[(JK - k) as usize] == 0 {
                    k += 1;
                }
                for i in (jz + 1)..=(jz + k) {
                    f[(jx + i) as usize] = IPIO2[(jv + i) as usize] as f32;
                    let mut fw = 0.0_f32;
                    for j in 0..=jx {
                        fw += x[j as usize] * f[(jx + i - j) as usize];
                    }
                    q[i as usize] = fw;
                }
                jz += k;
                continue;
            }
        }
        break;
    }

    // chop off zero terms
    if z == 0.0 {
        jz -= 1;
        q0 -= 8;
        while iq[jz as usize] == 0 {
            jz -= 1;
            q0 -= 8;
        }
    } else {
        z = gm_scalbnf(z, -q0);
        if z >= TWO8 {
            let fw = ((TWON8 * z) as i32) as f32;
            iq[jz as usize] = (z - TWO8 * fw) as i32;
            jz += 1;
            q0 += 8;
            iq[jz as usize] = fw as i32;
        } else {
            iq[jz as usize] = z as i32;
        }
    }

    // convert integer "bit" chunk to floating-point value
    let mut fw = gm_scalbnf(1.0, q0);
    let mut i = jz;
    while i >= 0 {
        q[i as usize] = fw * (iq[i as usize] as f32);
        fw *= TWON8;
        i -= 1;
    }

    // compute PIo2[0,...,jp]*q[jz,...,0]
    let mut i = jz;
    while i >= 0 {
        let mut fw = 0.0_f32;
        let mut k = 0;
        while k <= jp && k <= jz - i {
            fw += PIO2_TABLE[k as usize] * q[(i + k) as usize];
            k += 1;
        }
        fq[(jz - i) as usize] = fw;
        i -= 1;
    }

    // compress fq[] into y[] (prec 2)
    let mut fw = 0.0_f32;
    let mut i = jz;
    while i >= 0 {
        fw += fq[i as usize];
        i -= 1;
    }
    y[0] = if ih == 0 { fw } else { -fw };
    fw = fq[0] - fw;
    for i in 1..=jz {
        fw += fq[i as usize];
    }
    y[1] = if ih == 0 { fw } else { -fw };
    n & 7
}

const NPIO2_HW: [i32; 32] = [
    0x3fc90f00, 0x40490f00, 0x4096cb00, 0x40c90f00, 0x40fb5300, 0x4116cb00, 0x412fed00,
    0x41490f00, 0x41623100, 0x417b5300, 0x418a3a00, 0x4196cb00, 0x41a35c00, 0x41afed00,
    0x41bc7e00, 0x41c90f00, 0x41d5a000, 0x41e23100, 0x41eec200, 0x41fb5300, 0x4203f200,
    0x420a3a00, 0x42108300, 0x4216cb00, 0x421d1400, 0x42235c00, 0x4229a500, 0x422fed00,
    0x42363600, 0x423c7e00, 0x4242c700, 0x42490f00,
];

/// `__ieee754_rem_pio2f(x, y)` — returns `n` and writes `x rem pi/2` into `y[0] + y[1]`.
fn rem_pio2f(x: f32, y: &mut [f32; 2]) -> i32 {
    const HALF: f32 = 5.000_000_000_0e-01;
    const TWO8: f32 = 2.560_000_000_0e+02;
    const INVPIO2: f32 = 6.366_198_062_9e-01;
    const PIO2_1: f32 = 1.570_785_522_5e+00;
    const PIO2_1T: f32 = 1.080_433_412_4e-05;
    const PIO2_2: f32 = 1.080_427_318_8e-05;
    const PIO2_2T: f32 = 6.077_099_934_4e-11;
    const PIO2_3: f32 = 6.077_094_383_3e-11;
    const PIO2_3T: f32 = 6.123_234_262_9e-17;

    let hx = word(x);
    let ix = hx & 0x7fff_ffff;
    if ix <= 0x3f49_0fd8 {
        y[0] = x;
        y[1] = 0.0;
        return 0;
    }
    if ix < 0x4016_cbe4 {
        if hx > 0 {
            let mut z = x - PIO2_1;
            if (ix & (0xfffe_0000u32 as i32)) != 0x3fc8_0000 {
                y[0] = z - PIO2_1T;
                y[1] = (z - y[0]) - PIO2_1T;
            } else {
                z -= PIO2_2;
                y[0] = z - PIO2_2T;
                y[1] = (z - y[0]) - PIO2_2T;
            }
            return 1;
        } else {
            let mut z = x + PIO2_1;
            if (ix & (0xfffe_0000u32 as i32)) != 0x3fc8_0000 {
                y[0] = z + PIO2_1T;
                y[1] = (z - y[0]) + PIO2_1T;
            } else {
                z += PIO2_2;
                y[0] = z + PIO2_2T;
                y[1] = (z - y[0]) + PIO2_2T;
            }
            return -1;
        }
    }
    if ix <= 0x4349_0f80 {
        let mut t = gm_fabsf(x);
        let n = (t * INVPIO2 + HALF) as i32;
        let fn_ = n as f32;
        let mut r = t - fn_ * PIO2_1;
        let mut w = fn_ * PIO2_1T;
        if n < 32 && (ix & (0xffe0_0000u32 as i32)) != (NPIO2_HW[(n - 1) as usize] & (0xffe0_0000u32 as i32)) {
            y[0] = r - w;
        } else {
            let j = ix >> 23;
            y[0] = r - w;
            let high = y[0].to_bits();
            let mut i = j - ((high >> 23) & 0xff) as i32;
            if i > 8 {
                t = r;
                w = fn_ * PIO2_2;
                r = t - w;
                w = fn_ * PIO2_2T - ((t - r) - w);
                y[0] = r - w;
                let high = y[0].to_bits();
                i = j - ((high >> 23) & 0xff) as i32;
                if i > 25 {
                    t = r;
                    w = fn_ * PIO2_3;
                    r = t - w;
                    w = fn_ * PIO2_3T - ((t - r) - w);
                    y[0] = r - w;
                }
            }
        }
        y[1] = (r - y[0]) - w;
        if hx < 0 {
            y[0] = -y[0];
            y[1] = -y[1];
            return -n;
        }
        return n;
    }
    if ix >= 0x7f80_0000 {
        let nan = x - x;
        y[0] = nan;
        y[1] = nan;
        return 0;
    }
    let e0 = (ix >> 23) - 134;
    let mut z = from_word(ix - (e0 << 23));
    let mut tx = [0f32; 3];
    for v in tx.iter_mut().take(2) {
        *v = (z as i32) as f32;
        z = (z - *v) * TWO8;
    }
    tx[2] = z;
    let mut nx = 3;
    while tx[(nx - 1) as usize] == 0.0 {
        nx -= 1;
    }
    let n = kernel_rem_pio2f(&tx, y, e0, nx);
    if hx < 0 {
        y[0] = -y[0];
        y[1] = -y[1];
        return -n;
    }
    n
}

// ------------------------------------------------------------------------------------------
// Public trig (s_sinf.c, s_cosf.c, s_tanf.c, e_asinf.c, e_acosf.c, s_atanf.c, e_atan2f.c)
// ------------------------------------------------------------------------------------------

/// `gm_sinf` (`s_sinf.c`).
pub fn gm_sinf(x: f32) -> f32 {
    let ix = word(x) & 0x7fff_ffff;
    if ix <= 0x3f49_0fd8 {
        if ix < 0x3980_0000 && (x as i32) == 0 {
            return x;
        }
        return kernel_sinf(x, 0.0, 0);
    } else if ix >= 0x7f80_0000 {
        return x - x;
    }
    let mut y = [0f32; 2];
    let n = rem_pio2f(x, &mut y);
    match n & 3 {
        0 => kernel_sinf(y[0], y[1], 1),
        1 => kernel_cosf(y[0], y[1]),
        2 => -kernel_sinf(y[0], y[1], 1),
        _ => -kernel_cosf(y[0], y[1]),
    }
}

/// `gm_cosf` (`s_cosf.c`).
pub fn gm_cosf(x: f32) -> f32 {
    let ix = word(x) & 0x7fff_ffff;
    if ix <= 0x3f49_0fd8 {
        if ix < 0x3980_0000 && (x as i32) == 0 {
            return 1.0;
        }
        return kernel_cosf(x, 0.0);
    } else if ix >= 0x7f80_0000 {
        return x - x;
    }
    let mut y = [0f32; 2];
    let n = rem_pio2f(x, &mut y);
    match n & 3 {
        0 => kernel_cosf(y[0], y[1]),
        1 => -kernel_sinf(y[0], y[1], 1),
        2 => -kernel_cosf(y[0], y[1]),
        _ => kernel_sinf(y[0], y[1], 1),
    }
}

/// `gm_tanf` (`s_tanf.c`).
pub fn gm_tanf(x: f32) -> f32 {
    let ix = word(x) & 0x7fff_ffff;
    if ix <= 0x3f49_0fda {
        return kernel_tanf(x, 0.0, 1);
    } else if ix >= 0x7f80_0000 {
        return x - x;
    }
    let mut y = [0f32; 2];
    let n = rem_pio2f(x, &mut y);
    kernel_tanf(y[0], y[1], 1 - ((n & 1) << 1))
}

const P_S0: f32 = 1.666_666_716_3e-01;
const P_S1: f32 = -3.255_658_149_7e-01;
const P_S2: f32 = 2.012_125_253_7e-01;
const P_S3: f32 = -4.005_553_573_4e-02;
const P_S4: f32 = 7.915_350_142_9e-04;
const P_S5: f32 = 3.479_330_916_9e-05;
const Q_S1: f32 = -2.403_394_937_5e+00;
const Q_S2: f32 = 2.020_945_787_4e+00;
const Q_S3: f32 = -6.882_839_798_9e-01;
const Q_S4: f32 = 7.703_815_400_6e-02;
const PIO2_HI: f32 = 1.570_796_251_3e+00;
const PIO2_LO: f32 = 7.549_789_415_9e-08;

#[inline]
fn asin_p(t: f32) -> f32 {
    t * (P_S0 + t * (P_S1 + t * (P_S2 + t * (P_S3 + t * (P_S4 + t * P_S5)))))
}

#[inline]
fn asin_q(t: f32) -> f32 {
    1.0_f32 + t * (Q_S1 + t * (Q_S2 + t * (Q_S3 + t * Q_S4)))
}

/// `gm_asinf` (`e_asinf.c`).
pub fn gm_asinf(x: f32) -> f32 {
    const ONE: f32 = 1.0;
    const HUGE: f32 = 1.000e+30;
    const PIO4_HI: f32 = 7.853_981_852_5e-01;
    let hx = word(x);
    let ix = hx & 0x7fff_ffff;
    if ix == 0x3f80_0000 {
        return x * PIO2_HI + x * PIO2_LO;
    } else if ix > 0x3f80_0000 {
        return (x - x) / (x - x);
    } else if ix < 0x3f00_0000 {
        let mut t = 0.0_f32;
        if ix < 0x3200_0000 {
            if HUGE + x > ONE {
                return x;
            }
        } else {
            t = x * x;
        }
        let p = asin_p(t);
        let q = asin_q(t);
        let w = p / q;
        return x + x * w;
    }
    let w = ONE - gm_fabsf(x);
    let t = w * 0.5_f32;
    let mut p = asin_p(t);
    let mut q = asin_q(t);
    let s = gm_sqrtf(t);
    let t = if ix >= 0x3F79_999A {
        let w = p / q;
        PIO2_HI - (2.0_f32 * (s + s * w) - PIO2_LO)
    } else {
        let iw = word(s);
        let w = from_word(iw & (0xffff_f000u32 as i32));
        let c = (t - w * w) / (s + w);
        let r = p / q;
        p = 2.0_f32 * s * r - (PIO2_LO - 2.0_f32 * c);
        q = PIO4_HI - 2.0_f32 * w;
        PIO4_HI - (p - q)
    };
    if hx > 0 {
        t
    } else {
        -t
    }
}

/// `gm_acosf` (`e_acosf.c`).
pub fn gm_acosf(x: f32) -> f32 {
    const ONE: f32 = 1.0;
    const PI: f32 = 3.141_592_502_6e+00;
    let hx = word(x);
    let ix = hx & 0x7fff_ffff;
    if ix == 0x3f80_0000 {
        if hx > 0 {
            return 0.0;
        } else {
            return PI + 2.0_f32 * PIO2_LO;
        }
    } else if ix > 0x3f80_0000 {
        return (x - x) / (x - x);
    }
    if ix < 0x3f00_0000 {
        if ix <= 0x2300_0000 {
            return PIO2_HI + PIO2_LO;
        }
        let z = x * x;
        let r = asin_p(z) / asin_q(z);
        PIO2_HI - (x - (PIO2_LO - x * r))
    } else if hx < 0 {
        let z = (ONE + x) * 0.5_f32;
        let p = asin_p(z);
        let q = asin_q(z);
        let s = gm_sqrtf(z);
        let r = p / q;
        let w = r * s - PIO2_LO;
        PI - 2.0_f32 * (s + w)
    } else {
        let z = (ONE - x) * 0.5_f32;
        let s = gm_sqrtf(z);
        let idf = word(s);
        let df = from_word(idf & (0xffff_f000u32 as i32));
        let c = (z - df * df) / (s + df);
        let p = asin_p(z);
        let q = asin_q(z);
        let r = p / q;
        let w = r * s + c;
        2.0_f32 * (df + w)
    }
}

/// `gm_atanf` (`s_atanf.c`).
pub fn gm_atanf(mut x: f32) -> f32 {
    const ATANHI: [f32; 4] = [4.636_476_039_9e-01, 7.853_981_256_5e-01, 9.827_936_887_7e-01, 1.570_796_251_3e+00];
    const ATANLO: [f32; 4] = [5.012_158_244_0e-09, 3.774_894_707_9e-08, 3.447_321_717_0e-08, 7.549_789_415_9e-08];
    const AT: [f32; 11] = [
        3.333_333_432_7e-01,
        -2.000_000_029_8e-01,
        1.428_571_492_4e-01,
        -1.111_111_044_9e-01,
        9.090_887_010_1e-02,
        -7.691_875_845_2e-02,
        6.661_073_118_4e-02,
        -5.833_570_286_6e-02,
        4.976_877_942_7e-02,
        -3.653_157_129_9e-02,
        1.628_581_993_3e-02,
    ];
    const ONE: f32 = 1.0;
    const HUGE: f32 = 1.0e30;

    let hx = word(x);
    let ix = hx & 0x7fff_ffff;
    let id: i32;
    if ix >= 0x5080_0000 {
        if ix > 0x7f80_0000 {
            return x + x;
        }
        if hx > 0 {
            return ATANHI[3] + ATANLO[3];
        } else {
            return -ATANHI[3] - ATANLO[3];
        }
    }
    if ix < 0x3ee0_0000 {
        if ix < 0x3100_0000 && HUGE + x > ONE {
            return x;
        }
        id = -1;
    } else {
        x = gm_fabsf(x);
        if ix < 0x3f98_0000 {
            if ix < 0x3f30_0000 {
                id = 0;
                x = (2.0_f32 * x - ONE) / (2.0_f32 + x);
            } else {
                id = 1;
                x = (x - ONE) / (x + ONE);
            }
        } else if ix < 0x401c_0000 {
            id = 2;
            x = (x - 1.5_f32) / (ONE + 1.5_f32 * x);
        } else {
            id = 3;
            x = -1.0_f32 / x;
        }
    }
    let z = x * x;
    let w = z * z;
    let s1 = z * (AT[0] + w * (AT[2] + w * (AT[4] + w * (AT[6] + w * (AT[8] + w * AT[10])))));
    let s2 = w * (AT[1] + w * (AT[3] + w * (AT[5] + w * (AT[7] + w * AT[9]))));
    if id < 0 {
        x - x * (s1 + s2)
    } else {
        let id = id as usize;
        let z = ATANHI[id] - ((x * (s1 + s2) - ATANLO[id]) - x);
        if hx < 0 {
            -z
        } else {
            z
        }
    }
}

/// `gm_atan2f(y, x)` (`e_atan2f.c`).
pub fn gm_atan2f(y: f32, x: f32) -> f32 {
    const TINY: f32 = 1.0e-30;
    const ZERO: f32 = 0.0;
    const PI_O_4: f32 = 7.853_981_852_5e-01;
    const PI_O_2: f32 = 1.570_796_370_5e+00;
    const PI: f32 = 3.141_592_741_0e+00;
    const PI_LO: f32 = -8.742_277_657_3e-08;

    let hx = word(x);
    let ix = hx & 0x7fff_ffff;
    let hy = word(y);
    let iy = hy & 0x7fff_ffff;
    if ix > 0x7f80_0000 || iy > 0x7f80_0000 {
        return x + y;
    }
    if hx == 0x3f80_0000 {
        return gm_atanf(y);
    }
    let mut m = ((hy >> 31) & 1) | ((hx >> 30) & 2);

    if iy == 0 {
        match m {
            0 | 1 => return y,
            2 => return PI + TINY,
            _ => return -PI - TINY,
        }
    }
    if ix == 0 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }
    if ix == 0x7f80_0000 {
        if iy == 0x7f80_0000 {
            match m {
                0 => return PI_O_4 + TINY,
                1 => return -PI_O_4 - TINY,
                2 => return 3.0_f32 * PI_O_4 + TINY,
                _ => return -3.0_f32 * PI_O_4 - TINY,
            }
        } else {
            match m {
                0 => return ZERO,
                1 => return -ZERO,
                2 => return PI + TINY,
                _ => return -PI - TINY,
            }
        }
    }
    if iy == 0x7f80_0000 {
        return if hy < 0 { -PI_O_2 - TINY } else { PI_O_2 + TINY };
    }

    let k = (iy - ix) >> 23;
    let z = if k > 26 {
        m &= 1;
        PI_O_2 + 0.5_f32 * PI_LO
    } else if hx < 0 && k < -26 {
        0.0
    } else {
        gm_atanf(gm_fabsf(y / x))
    };
    match m {
        0 => z,
        1 => -z,
        2 => PI - (z - PI_LO),
        _ => (z - PI_LO) - PI,
    }
}

/// `gm_sqrt` double-precision (`double/e_sqrt.c`), used by `Sqrt(double)` / `ICoord2D::length()`.
pub fn gm_sqrt(x: f64) -> f64 {
    const ONE: f64 = 1.0;
    const TINY: f64 = 1.0e-300;
    const SIGN_U: u32 = 0x8000_0000;
    let sign: i32 = SIGN_U as i32;

    let bits = x.to_bits();
    let mut ix0 = (bits >> 32) as i32;
    let mut ix1 = bits as u32;

    if (ix0 & 0x7ff0_0000) == 0x7ff0_0000 {
        return x * x + x;
    }
    if ix0 <= 0 {
        if (((ix0 & !sign) as u32) | ix1) == 0 {
            return x;
        } else if ix0 < 0 {
            return (x - x) / (x - x);
        }
    }
    let mut m = ix0 >> 20;
    if m == 0 {
        while ix0 == 0 {
            m -= 21;
            ix0 |= (ix1 >> 11) as i32;
            ix1 <<= 21;
        }
        let mut i: i32 = 0;
        while (ix0 & 0x0010_0000) == 0 {
            ix0 <<= 1;
            i += 1;
        }
        m -= i - 1;
        // C shifts by (32 - i) which is 32 when i == 0; hardware masks the count mod 32.
        ix0 |= ix1.wrapping_shr((32 - i) as u32) as i32;
        ix1 = ix1.wrapping_shl(i as u32);
    }
    m -= 1023;
    ix0 = (ix0 & 0x000f_ffff) | 0x0010_0000;
    if (m & 1) != 0 {
        ix0 = ix0.wrapping_add(ix0).wrapping_add(((ix1 & SIGN_U) >> 31) as i32);
        ix1 = ix1.wrapping_add(ix1);
    }
    m >>= 1;

    ix0 = ix0.wrapping_add(ix0).wrapping_add(((ix1 & SIGN_U) >> 31) as i32);
    ix1 = ix1.wrapping_add(ix1);
    let mut q: i32 = 0;
    let mut q1: u32 = 0;
    let mut s0: i32 = 0;
    let mut s1: u32 = 0;
    let mut r: u32 = 0x0020_0000;

    while r != 0 {
        let t = s0.wrapping_add(r as i32);
        if t <= ix0 {
            s0 = t.wrapping_add(r as i32);
            ix0 = ix0.wrapping_sub(t);
            q = q.wrapping_add(r as i32);
        }
        ix0 = ix0.wrapping_add(ix0).wrapping_add(((ix1 & SIGN_U) >> 31) as i32);
        ix1 = ix1.wrapping_add(ix1);
        r >>= 1;
    }

    r = SIGN_U;
    while r != 0 {
        let t1 = s1.wrapping_add(r);
        let t = s0;
        if t < ix0 || (t == ix0 && t1 <= ix1) {
            s1 = t1.wrapping_add(r);
            if (t1 & SIGN_U) == SIGN_U && (s1 & SIGN_U) == 0 {
                s0 = s0.wrapping_add(1);
            }
            ix0 = ix0.wrapping_sub(t);
            if ix1 < t1 {
                ix0 = ix0.wrapping_sub(1);
            }
            ix1 = ix1.wrapping_sub(t1);
            q1 = q1.wrapping_add(r);
        }
        ix0 = ix0.wrapping_add(ix0).wrapping_add(((ix1 & SIGN_U) >> 31) as i32);
        ix1 = ix1.wrapping_add(ix1);
        r >>= 1;
    }

    if ((ix0 as u32) | ix1) != 0 {
        let z = ONE - TINY;
        if z >= ONE {
            let z = ONE + TINY;
            if q1 == 0xffff_ffff {
                q1 = 0;
                q = q.wrapping_add(1);
            } else if z > ONE {
                if q1 == 0xffff_fffe {
                    q = q.wrapping_add(1);
                }
                q1 = q1.wrapping_add(2);
            } else {
                q1 = q1.wrapping_add(q1 & 1);
            }
        }
    }
    let mut hi = (q >> 1).wrapping_add(0x3fe0_0000);
    let mut lo = q1 >> 1;
    if (q & 1) == 1 {
        lo |= SIGN_U;
    }
    hi = hi.wrapping_add(m << 20);
    f64::from_bits(((hi as u32 as u64) << 32) | lo as u64)
}

// ------------------------------------------------------------------------------------------
// Engine-level wrappers (Trig.cpp, wwmath.h) under USE_DETERMINISTIC_MATH
// ------------------------------------------------------------------------------------------

/// `WWMATH_PI` (`wwmath.h:56`).
pub const WWMATH_PI: f32 = 3.141_592_654;
/// `WWMATH_TWO_PI` (`wwmath.h:57`).
pub const WWMATH_TWO_PI: f32 = 6.283_185_308;

/// `WWMath::Normalize_Angle` (`wwmath.h:1072`): normalizes to `[-PI, PI)`.
#[inline]
pub fn normalize_angle(angle: f32) -> f32 {
    angle - (WWMATH_TWO_PI * gm_floorf((angle + WWMATH_PI) / WWMATH_TWO_PI))
}

/// `Sin` (`Trig.cpp`) / `WWMath::Sin(float)`.
#[inline]
pub fn sin(x: f32) -> f32 {
    gm_sinf(x)
}
/// `Cos` (`Trig.cpp`) / `WWMath::Cos(float)`.
#[inline]
pub fn cos(x: f32) -> f32 {
    gm_cosf(x)
}
/// `Tan` (`Trig.cpp`) / `WWMath::Tan(float)`.
#[inline]
pub fn tan(x: f32) -> f32 {
    gm_tanf(x)
}
/// `ASin` (`Trig.cpp`) / `WWMath::Asin(float)`.
#[inline]
pub fn asin(x: f32) -> f32 {
    gm_asinf(x)
}
/// `ACos` (`Trig.cpp`) / `WWMath::Acos(float)`.
#[inline]
pub fn acos(x: f32) -> f32 {
    gm_acosf(x)
}
/// `WWMath::Atan2(float y, float x)`.
#[inline]
pub fn atan2(y: f32, x: f32) -> f32 {
    gm_atan2f(y, x)
}
/// `WWMath::Ceilf`.
#[inline]
pub fn ceilf(x: f32) -> f32 {
    gm_ceilf(x)
}

/// `PI` (`Core/Libraries/Source/Lib/Include/Precompiled/BaseMacros.h:34`).
pub const PI: f32 = 3.141_592_653_59;
/// `TWO_PI` (`Core/Libraries/Source/Lib/Include/Precompiled/BaseMacros.h:35`).
pub const TWO_PI: f32 = 6.283_185_307_18;
/// `WWMATH_EPSILON` (`wwmath.h:53`).
pub const WWMATH_EPSILON: f32 = 0.0001;

/// `fast_float_trunc` (`BaseFunctions.h:80`).
#[inline]
pub fn fast_float_trunc(f: f32) -> f32 {
    let mut x = f.to_bits();
    let mut exp = (x >> 23) as u8;
    let mut mask: i32 = if exp < 127 {
        0
    } else {
        0xff80_0000u32 as i32
    };
    exp = exp.wrapping_sub(127);
    mask >>= (exp & 31) as u32;
    x &= mask as u32;
    f32::from_bits(x)
}

/// `fast_float_floor` (`BaseFunctions.h:111`), reproducing VC6 round-to-nearest retail quirk.
#[inline]
pub fn fast_float_floor(mut f: f32) -> f32 {
    const ALMOST1: f32 = f32::from_bits((126 << 23) | 0x7f_ffff);
    if (f.to_bits() & 0x8000_0000) != 0 {
        f -= ALMOST1;
    }
    fast_float_trunc(f)
}

/// `fast_float_ceil` (`BaseFunctions.h:124`), reproducing VC6 round-to-nearest retail quirk.
#[inline]
pub fn fast_float_ceil(mut f: f32) -> f32 {
    const ALMOST1: f32 = f32::from_bits((126 << 23) | 0x7f_ffff);
    if (f.to_bits() & 0x8000_0000) == 0 {
        f += ALMOST1;
    }
    fast_float_trunc(f)
}

/// `gm_lrintf`: rounds using IEEE round-to-nearest-even (`gm_rintf`) and converts to `i32`.
#[inline]
pub fn gm_lrintf(x: f32) -> i32 {
    gm_rintf(x) as i32
}

/// `gm_lroundf`: rounds half away from zero (`gm_roundf`) and converts to `i32`.
#[inline]
pub fn gm_lroundf(x: f32) -> i32 {
    gm_roundf(x) as i32
}

/// `REAL_TO_INT_FLOOR` (`BaseFunctions.h:136,140`).
#[inline]
pub fn real_to_int_floor(x: f32) -> i32 {
    #[cfg(feature = "retail-crc")]
    {
        gm_lroundf(fast_float_floor(x))
    }
    #[cfg(not(feature = "retail-crc"))]
    {
        gm_lroundf(gm_floorf(x))
    }
}

/// `REAL_TO_INT_CEIL` (`BaseFunctions.h:135,139`).
#[inline]
pub fn real_to_int_ceil(x: f32) -> i32 {
    #[cfg(feature = "retail-crc")]
    {
        gm_lroundf(fast_float_ceil(x))
    }
    #[cfg(not(feature = "retail-crc"))]
    {
        gm_lroundf(gm_ceilf(x))
    }
}

/// `WWMath::Float_To_Int_Chop` (`wwmath.h:1046`).
#[inline]
pub fn float_to_int_chop(f: f32) -> i32 {
    let a = f.to_bits() as i32;
    let sign = a >> 31;
    let mantissa = (a & ((1 << 23) - 1)) | (1 << 23);
    let exponent = ((a & 0x7fff_ffff) >> 23) - 127;
    let shift = 31i32.wrapping_sub(exponent) as u32;
    let r = (((mantissa as u32) << 8).wrapping_shr(shift)) as i32;
    ((r ^ sign).wrapping_sub(sign)) & !(exponent >> 31)
}

/// `WWMath::Float_To_Int_Floor` (`wwmath.h:1056`).
#[inline]
pub fn float_to_int_floor(f: f32) -> i32 {
    let mut a = f.to_bits() as i32;
    let sign = a >> 31;
    a &= 0x7fff_ffff;
    let exponent = (a >> 23) - 127;
    let expsign = !(exponent >> 31);
    let shift = 31i32.wrapping_sub(exponent) as u32;
    let imask = 1u32.wrapping_shl(shift).wrapping_sub(1) as i32;
    let mantissa = a & ((1 << 23) - 1);
    let r = (((mantissa | (1 << 23)) as u32) << 8).wrapping_shr(shift) as i32;
    let adj = if ((mantissa << 8) & imask) == 0 { 1i32 } else { 0i32 };
    let adj_term = (adj & (expsign ^ (a.wrapping_sub(1) >> 31))) & sign;
    ((r & expsign) ^ sign).wrapping_add(adj_term)
}

/// `WWMath::Lerp` (`wwmath.h:949`).
#[inline]
pub fn lerp_f32(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// `WWMath::Inverse_Lerp` (`wwmath.h:959`).
#[inline]
pub fn inverse_lerp_f32(a: f32, b: f32, v: f32) -> f32 {
    if a == b {
        0.0
    } else {
        (v - a) / (b - a)
    }
}

/// `WWMath::Clamp` (`wwmath.h:887`).
#[inline]
pub fn clamp_f32(val: f32, min: f32, max: f32) -> f32 {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

/// `WWMath::Wrap` (`wwmath.h:908`).
#[inline]
pub fn wrap_f32(mut val: f32, min: f32, max: f32) -> f32 {
    if val >= max {
        val -= max - min;
    }
    if val < min {
        val += max - min;
    }
    if val < min {
        val = min;
    }
    if val > max {
        val = max;
    }
    val
}
