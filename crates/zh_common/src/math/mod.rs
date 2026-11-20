//! Deterministic C++ promotion, `GameMath` fdlibm IEEE-754 binary32 primitives, `Coord3D`,
//! and Phase 2 math modules (`PKT-015`..`PKT-023`).

pub mod bez_fwd_iterator;
pub mod bezier_segment;
pub mod discrete_circle;
pub mod partition_solver;

use crate::types::{Bool, Real, FALSE, TRUE};

/// Replicates C++ compound mixed-type assignment `Int a; Real s; a *= s;`
/// (`a = static_cast<Int>(static_cast<Real>(a) * s)` with truncation toward zero).
#[inline]
pub fn cpp_mul_i32_f32(a: i32, s: f32) -> i32 {
    ((a as f32) * s) as i32
}

/// Replicates C++ `#define INT_TO_REAL(x) ((Real)(x))` when applied to an `f32` expression.
#[inline]
pub const fn int_to_real_f32(x: f32) -> f32 {
    x
}

/// Exact integer-bit-manipulation fdlibm `gm_sqrtf` matching `GameMath` (`e_sqrtf.c`).
pub fn gm_sqrtf(x: f32) -> f32 {
    const ONE: f32 = 1.0;
    const TINY: f32 = 1.0e-30;
    let sign: i32 = 0x8000_0000u32 as i32;
    let mut ix: i32 = x.to_bits() as i32;

    if (ix & 0x7f80_0000) == 0x7f80_0000 {
        return x * x + x;
    }
    if ix <= 0 {
        if (ix & !sign) == 0 {
            return x;
        } else if ix < 0 {
            return (x - x) / (x - x);
        }
    }

    let mut m: i32 = ix >> 23;
    if m == 0 {
        let mut i: i32 = 0;
        while (ix & 0x0080_0000) == 0 {
            ix <<= 1;
            i += 1;
        }
        m -= i - 1;
    }
    m -= 127;
    ix = (ix & 0x007f_ffff) | 0x0080_0000;
    if (m & 1) != 0 {
        ix = ix.wrapping_add(ix);
    }
    m >>= 1;

    ix = ix.wrapping_add(ix);
    let mut q: i32 = 0;
    let mut s: i32 = 0;
    let mut r: u32 = 0x0100_0000;

    while r != 0 {
        let t: i32 = s.wrapping_add(r as i32);
        if t <= ix {
            s = t.wrapping_add(r as i32);
            ix = ix.wrapping_sub(t);
            q = q.wrapping_add(r as i32);
        }
        ix = ix.wrapping_add(ix);
        r >>= 1;
    }

    if ix != 0 {
        let z = ONE - TINY;
        if z >= ONE {
            let z2 = ONE + TINY;
            if z2 > ONE {
                q = q.wrapping_add(2);
            } else {
                q = q.wrapping_add(q & 1);
            }
        }
    }
    ix = (q >> 1).wrapping_add(0x3f00_0000);
    ix = ix.wrapping_add(m << 23);
    f32::from_bits(ix as u32)
}

/// Exact integer-bit-manipulation fdlibm `gm_floorf` matching `GameMath` (`s_floorf.c`).
pub fn gm_floorf(x: f32) -> f32 {
    const HUGE: f32 = 1.0e30;
    let mut i0: i32 = x.to_bits() as i32;
    let j0: i32 = ((i0 >> 23) & 0xff) - 0x7f;
    if j0 < 23 {
        if j0 < 0 {
            if HUGE + x > 0.0 {
                if i0 >= 0 {
                    i0 = 0;
                } else if (i0 & 0x7fff_ffff) != 0 {
                    i0 = 0xbf80_0000u32 as i32;
                }
            }
        } else {
            let i: u32 = 0x007f_ffffu32 >> j0;
            if ((i0 as u32) & i) == 0 {
                return x;
            }
            if HUGE + x > 0.0 {
                if i0 < 0 {
                    i0 = i0.wrapping_add((0x0080_0000u32 >> j0) as i32);
                }
                i0 &= !(i as i32);
            }
        }
    } else if j0 == 0x80 {
        return x + x;
    } else {
        return x;
    }
    f32::from_bits(i0 as u32)
}

/// Exact integer-bit-manipulation fdlibm `gm_ceilf` matching `GameMath` (`s_ceilf.c`).
pub fn gm_ceilf(x: f32) -> f32 {
    const HUGE: f32 = 1.0e30;
    let mut i0: i32 = x.to_bits() as i32;
    let j0: i32 = ((i0 >> 23) & 0xff) - 0x7f;
    if j0 < 23 {
        if j0 < 0 {
            if HUGE + x > 0.0 {
                if i0 < 0 {
                    i0 = 0x8000_0000u32 as i32;
                } else if i0 != 0 {
                    i0 = 0x3f80_0000;
                }
            }
        } else {
            let i: u32 = 0x007f_ffffu32 >> j0;
            if ((i0 as u32) & i) == 0 {
                return x;
            }
            if HUGE + x > 0.0 {
                if i0 > 0 {
                    i0 = i0.wrapping_add((0x0080_0000u32 >> j0) as i32);
                }
                i0 &= !(i as i32);
            }
        }
    } else if j0 == 0x80 {
        return x + x;
    } else {
        return x;
    }
    f32::from_bits(i0 as u32)
}

/// 2D floating-point coordinate matching `Core/Libraries/Source/Lib/Include/Lib/Coord2D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Coord2D {
    pub x: Real,
    pub y: Real,
}

/// 3D floating-point coordinate matching `Core/Libraries/Source/Lib/Include/Lib/Coord3D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Coord3D {
    pub x: Real,
    pub y: Real,
    pub z: Real,
}

impl Coord3D {
    #[inline]
    pub const fn new(x: Real, y: Real, z: Real) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn as_coord2d(&self) -> Coord2D {
        Coord2D {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn length(&self) -> Real {
        gm_sqrtf((self.x * self.x + self.y * self.y) + self.z * self.z)
    }

    #[inline]
    pub fn length_sqr(&self) -> Real {
        (self.x * self.x + self.y * self.y) + self.z * self.z
    }

    #[inline]
    pub fn normalize(&mut self) {
        let len = self.length();
        if len != 0.0 {
            self.x /= len;
            self.y /= len;
            self.z /= len;
        }
    }

    #[inline]
    pub fn cross_product(a: &Coord3D, b: &Coord3D, r: &mut Coord3D) {
        r.x = a.y * b.z - a.z * b.y;
        r.y = a.z * b.x - a.x * b.z;
        r.z = a.x * b.y - a.y * b.x;
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.z = 0.0;
    }

    #[inline]
    pub fn add(&mut self, a: &Coord3D) {
        self.x += a.x;
        self.y += a.y;
        self.z += a.z;
    }

    #[inline]
    pub fn sub(&mut self, a: &Coord3D) {
        self.x -= a.x;
        self.y -= a.y;
        self.z -= a.z;
    }

    #[inline]
    pub fn set(&mut self, a: &Coord3D) {
        self.x = a.x;
        self.y = a.y;
        self.z = a.z;
    }

    #[inline]
    pub fn set_xyz(&mut self, ax: Real, ay: Real, az: Real) {
        self.x = ax;
        self.y = ay;
        self.z = az;
    }

    #[inline]
    pub fn scale(&mut self, s: Real) {
        self.x *= s;
        self.y *= s;
        self.z *= s;
    }

    #[inline]
    pub fn equals(&self, r: &Coord3D) -> Bool {
        if self.x == r.x && self.y == r.y && self.z == r.z {
            TRUE
        } else {
            FALSE
        }
    }
}
