//! Deterministic C++ promotion, `GameMath` fdlibm IEEE-754 binary32 primitives, `Coord3D`,
//! and Phase 2 math modules (`PKT-015`..`PKT-023`).

pub mod bez_fwd_iterator;
pub mod bezier_segment;
pub mod discrete_circle;
pub mod gamemath;
pub mod geometry;
pub mod partition_solver;

pub use gamemath::{
    clamp_f32, fast_float_ceil, fast_float_floor, fast_float_trunc, float_to_int_chop,
    float_to_int_floor, gm_acosf, gm_asinf, gm_atan2f, gm_atanf, gm_copysignf, gm_cosf,
    gm_fabsf, gm_lrintf, gm_lroundf, gm_rintf, gm_roundf, gm_scalbnf, gm_sinf, gm_sqrt,
    gm_tanf, inverse_lerp_f32, lerp_f32, normalize_angle, real_to_int_ceil, real_to_int_floor,
    wrap_f32, PI, TWO_PI, WWMATH_EPSILON,
};

use crate::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

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

impl Coord2D {
    #[inline]
    pub const fn new(x: Real, y: Real) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
    }

    #[inline]
    pub fn is_val(&self, value: Real) -> bool {
        self.x == value && self.y == value
    }

    #[inline]
    pub fn length(&self) -> Real {
        gm_sqrtf(self.x * self.x + self.y * self.y)
    }

    #[inline]
    pub fn length_sqr(&self) -> Real {
        self.x * self.x + self.y * self.y
    }

    #[inline]
    pub fn normalize(&mut self) {
        let len = self.length();
        if len != 0.0 {
            self.x /= len;
            self.y /= len;
        }
    }

    #[inline]
    pub fn to_angle(&self) -> Real {
        let len = self.length();
        if len == 0.0 {
            return 0.0;
        }
        let mut c = self.x / len;
        if c < -1.0 {
            c = -1.0;
        } else if c > 1.0 {
            c = 1.0;
        }
        if self.y < 0.0 {
            -gm_acosf(c)
        } else {
            gm_acosf(c)
        }
    }

    #[inline]
    pub fn add(&mut self, a: &Coord2D) {
        self.x += a.x;
        self.y += a.y;
    }

    #[inline]
    pub fn sub(&mut self, a: &Coord2D) {
        self.x -= a.x;
        self.y -= a.y;
    }

    #[inline]
    pub fn set(&mut self, a: &Coord2D) {
        self.x = a.x;
        self.y = a.y;
    }

    #[inline]
    pub fn set_xy(&mut self, ax: Real, ay: Real) {
        self.x = ax;
        self.y = ay;
    }

    #[inline]
    pub fn update_min(&mut self, other: &Coord2D) {
        if self.x > other.x {
            self.x = other.x;
        }
        if self.y > other.y {
            self.y = other.y;
        }
    }

    #[inline]
    pub fn update_max(&mut self, other: &Coord2D) {
        if self.x < other.x {
            self.x = other.x;
        }
        if self.y < other.y {
            self.y = other.y;
        }
    }
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

    /// Normalizes via reciprocal multiplication `x * (1.0 / gm_sqrtf(len2))` matching
    /// `WWMath::Vector3::Normalize` (`vector3.h:417`).
    #[inline]
    pub fn normalize_ww_vector3(&mut self) {
        let len2 = (self.x * self.x + self.y * self.y) + self.z * self.z;
        if len2 != 0.0 {
            let oolen = 1.0_f32 / gm_sqrtf(len2);
            self.x *= oolen;
            self.y *= oolen;
            self.z *= oolen;
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
    pub fn is_val(&self, value: Real) -> bool {
        self.x == value && self.y == value && self.z == value
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

    #[inline]
    pub fn update_min(&mut self, other: &Coord3D) {
        if self.x > other.x {
            self.x = other.x;
        }
        if self.y > other.y {
            self.y = other.y;
        }
        if self.z > other.z {
            self.z = other.z;
        }
    }

    #[inline]
    pub fn update_max(&mut self, other: &Coord3D) {
        if self.x < other.x {
            self.x = other.x;
        }
        if self.y < other.y {
            self.y = other.y;
        }
        if self.z < other.z {
            self.z = other.z;
        }
    }
}

/// 2D integer coordinate matching `Core/Libraries/Source/Lib/Include/Lib/ICoord2D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ICoord2D {
    pub x: Int,
    pub y: Int,
}

impl ICoord2D {
    #[inline]
    pub const fn new(x: Int, y: Int) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0;
        self.y = 0;
    }

    #[inline]
    pub fn is_val(&self, value: Int) -> bool {
        self.x == value && self.y == value
    }

    #[inline]
    pub fn length(&self) -> Int {
        gm_sqrt(self.length_sqr() as f64) as Int
    }

    #[inline]
    pub fn length_sqr(&self) -> Int {
        self.x.wrapping_mul(self.x).wrapping_add(self.y.wrapping_mul(self.y))
    }

    #[inline]
    pub fn add(&mut self, a: &ICoord2D) {
        self.x = self.x.wrapping_add(a.x);
        self.y = self.y.wrapping_add(a.y);
    }

    #[inline]
    pub fn sub(&mut self, a: &ICoord2D) {
        self.x = self.x.wrapping_sub(a.x);
        self.y = self.y.wrapping_sub(a.y);
    }

    #[inline]
    pub fn update_min(&mut self, other: &ICoord2D) {
        if self.x > other.x {
            self.x = other.x;
        }
        if self.y > other.y {
            self.y = other.y;
        }
    }

    #[inline]
    pub fn update_max(&mut self, other: &ICoord2D) {
        if self.x < other.x {
            self.x = other.x;
        }
        if self.y < other.y {
            self.y = other.y;
        }
    }
}

/// 3D integer coordinate matching `Core/Libraries/Source/Lib/Include/Lib/ICoord3D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ICoord3D {
    pub x: Int,
    pub y: Int,
    pub z: Int,
}

impl ICoord3D {
    #[inline]
    pub const fn new(x: Int, y: Int, z: Int) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub const fn as_icoord2d(&self) -> ICoord2D {
        ICoord2D {
            x: self.x,
            y: self.y,
        }
    }

    #[inline]
    pub fn zero(&mut self) {
        self.x = 0;
        self.y = 0;
        self.z = 0;
    }

    #[inline]
    pub fn is_val(&self, value: Int) -> bool {
        self.x == value && self.y == value && self.z == value
    }

    #[inline]
    pub fn length(&self) -> Int {
        gm_sqrt(self.length_sqr() as f64) as Int
    }

    #[inline]
    pub fn length_sqr(&self) -> Int {
        self.x
            .wrapping_mul(self.x)
            .wrapping_add(self.y.wrapping_mul(self.y))
            .wrapping_add(self.z.wrapping_mul(self.z))
    }

    #[inline]
    pub fn add(&mut self, a: &ICoord3D) {
        self.x = self.x.wrapping_add(a.x);
        self.y = self.y.wrapping_add(a.y);
        self.z = self.z.wrapping_add(a.z);
    }

    #[inline]
    pub fn sub(&mut self, a: &ICoord3D) {
        self.x = self.x.wrapping_sub(a.x);
        self.y = self.y.wrapping_sub(a.y);
        self.z = self.z.wrapping_sub(a.z);
    }

    #[inline]
    pub fn update_min(&mut self, other: &ICoord3D) {
        if self.x > other.x {
            self.x = other.x;
        }
        if self.y > other.y {
            self.y = other.y;
        }
        if self.z > other.z {
            self.z = other.z;
        }
    }

    #[inline]
    pub fn update_max(&mut self, other: &ICoord3D) {
        if self.x < other.x {
            self.x = other.x;
        }
        if self.y < other.y {
            self.y = other.y;
        }
        if self.z < other.z {
            self.z = other.z;
        }
    }
}

/// 2D axis-aligned region matching `Region2D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Region2D {
    pub lo: Coord2D,
    pub hi: Coord2D,
}

impl Region2D {
    #[inline]
    pub fn zero(&mut self) {
        self.lo.zero();
        self.hi.zero();
    }

    #[inline]
    pub fn width(&self) -> Real {
        self.hi.x - self.lo.x
    }

    #[inline]
    pub fn height(&self) -> Real {
        self.hi.y - self.lo.y
    }

    #[inline]
    pub fn intersect_with(&mut self, other: &Region2D) {
        self.lo.update_max(&other.lo);
        self.hi.update_min(&other.hi);
    }

    #[inline]
    pub fn unite_with(&mut self, other: &Region2D) {
        self.lo.update_min(&other.lo);
        self.hi.update_max(&other.hi);
    }

    #[inline]
    pub fn unite_with_point(&mut self, point: &Coord2D) {
        self.lo.update_min(point);
        self.hi.update_max(point);
    }

    #[inline]
    pub fn contains_xy(&self, x: Real, y: Real) -> Bool {
        if self.lo.x <= x && x <= self.hi.x && self.lo.y <= y && y <= self.hi.y {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_within_xy(&self, x: Real, y: Real) -> Bool {
        if self.lo.x < x && x < self.hi.x && self.lo.y < y && y < self.hi.y {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_point(&self, point: &Coord2D) -> Bool {
        self.contains_xy(point.x, point.y)
    }

    #[inline]
    pub fn contains_within_point(&self, point: &Coord2D) -> Bool {
        self.contains_within_xy(point.x, point.y)
    }

    #[inline]
    pub fn contains_region(&self, other: &Region2D) -> Bool {
        if self.lo.x <= other.lo.x
            && other.hi.x <= self.hi.x
            && self.lo.y <= other.lo.y
            && other.hi.y <= self.hi.y
        {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_within_region(&self, other: &Region2D) -> Bool {
        if self.lo.x < other.lo.x
            && other.hi.x < self.hi.x
            && self.lo.y < other.lo.y
            && other.hi.y < self.hi.y
        {
            TRUE
        } else {
            FALSE
        }
    }
}

/// 3D axis-aligned region matching `Region3D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Region3D {
    pub lo: Coord3D,
    pub hi: Coord3D,
}

impl Region3D {
    #[inline]
    pub fn zero(&mut self) {
        self.lo.zero();
        self.hi.zero();
    }

    #[inline]
    pub fn width(&self) -> Real {
        self.hi.x - self.lo.x
    }

    #[inline]
    pub fn height(&self) -> Real {
        self.hi.y - self.lo.y
    }

    #[inline]
    pub fn depth(&self) -> Real {
        self.hi.z - self.lo.z
    }

    #[inline]
    pub fn intersect_with(&mut self, other: &Region3D) {
        self.lo.update_max(&other.lo);
        self.hi.update_min(&other.hi);
    }

    #[inline]
    pub fn unite_with(&mut self, other: &Region3D) {
        self.lo.update_min(&other.lo);
        self.hi.update_max(&other.hi);
    }

    #[inline]
    pub fn unite_with_point(&mut self, point: &Coord3D) {
        self.lo.update_min(point);
        self.hi.update_max(point);
    }

    #[inline]
    pub fn set_xy(&mut self, region: &Region2D) {
        self.lo.x = region.lo.x;
        self.lo.y = region.lo.y;
        self.hi.x = region.hi.x;
        self.hi.y = region.hi.y;
    }

    pub fn set_xy_from_points(&mut self, points: &[Coord3D]) {
        if points.is_empty() {
            return;
        }
        self.lo.x = points[0].x;
        self.lo.y = points[0].y;
        self.hi.x = points[0].x;
        self.hi.y = points[0].y;
        for pt in &points[1..] {
            if pt.x < self.lo.x {
                self.lo.x = pt.x;
            }
            if pt.y < self.lo.y {
                self.lo.y = pt.y;
            }
            if pt.x > self.hi.x {
                self.hi.x = pt.x;
            }
            if pt.y > self.hi.y {
                self.hi.y = pt.y;
            }
        }
    }

    pub fn set_from_points(&mut self, points: &[Coord3D]) {
        if points.is_empty() {
            return;
        }
        self.lo = points[0];
        self.hi = points[0];
        for pt in &points[1..] {
            self.unite_with_point(pt);
        }
    }

    #[inline]
    pub fn contains_xyz(&self, x: Real, y: Real, z: Real) -> Bool {
        if self.lo.x <= x
            && x <= self.hi.x
            && self.lo.y <= y
            && y <= self.hi.y
            && self.lo.z <= z
            && z <= self.hi.z
        {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_within_xyz(&self, x: Real, y: Real, z: Real) -> Bool {
        if self.lo.x < x
            && x < self.hi.x
            && self.lo.y < y
            && y < self.hi.y
            && self.lo.z < z
            && z < self.hi.z
        {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_coord3d(&self, point: &Coord3D) -> Bool {
        self.contains_xyz(point.x, point.y, point.z)
    }

    #[inline]
    pub fn contains_within_coord3d(&self, point: &Coord3D) -> Bool {
        self.contains_within_xyz(point.x, point.y, point.z)
    }

    #[inline]
    pub fn contains_coord2d(&self, point: &Coord2D) -> Bool {
        if self.lo.x <= point.x && point.x <= self.hi.x && self.lo.y <= point.y && point.y <= self.hi.y {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_within_coord2d(&self, point: &Coord2D) -> Bool {
        if self.lo.x < point.x && point.x < self.hi.x && self.lo.y < point.y && point.y < self.hi.y {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_region(&self, other: &Region3D) -> Bool {
        if self.lo.x <= other.lo.x
            && other.hi.x <= self.hi.x
            && self.lo.y <= other.lo.y
            && other.hi.y <= self.hi.y
            && self.lo.z <= other.lo.z
            && other.hi.z <= self.hi.z
        {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_within_region(&self, other: &Region3D) -> Bool {
        if self.lo.x < other.lo.x
            && other.hi.x < self.hi.x
            && self.lo.y < other.lo.y
            && other.hi.y < self.hi.y
            && self.lo.z < other.lo.z
            && other.hi.z < self.hi.z
        {
            TRUE
        } else {
            FALSE
        }
    }
}

/// 2D integer axis-aligned region matching `IRegion2D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IRegion2D {
    pub lo: ICoord2D,
    pub hi: ICoord2D,
}

impl IRegion2D {
    #[inline]
    pub fn zero(&mut self) {
        self.lo.zero();
        self.hi.zero();
    }

    #[inline]
    pub fn width(&self) -> Int {
        self.hi.x.wrapping_sub(self.lo.x)
    }

    #[inline]
    pub fn height(&self) -> Int {
        self.hi.y.wrapping_sub(self.lo.y)
    }

    #[inline]
    pub fn intersect_with(&mut self, other: &IRegion2D) {
        self.lo.update_max(&other.lo);
        self.hi.update_min(&other.hi);
    }

    #[inline]
    pub fn unite_with(&mut self, other: &IRegion2D) {
        self.lo.update_min(&other.lo);
        self.hi.update_max(&other.hi);
    }

    #[inline]
    pub fn unite_with_point(&mut self, point: &ICoord2D) {
        self.lo.update_min(point);
        self.hi.update_max(point);
    }

    #[inline]
    pub fn contains_xy(&self, x: Int, y: Int) -> Bool {
        if self.lo.x <= x && x <= self.hi.x && self.lo.y <= y && y <= self.hi.y {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn contains_within_xy(&self, x: Int, y: Int) -> Bool {
        if self.lo.x < x && x < self.hi.x && self.lo.y < y && y < self.hi.y {
            TRUE
        } else {
            FALSE
        }
    }
}

/// 3D integer axis-aligned region matching `IRegion3D.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct IRegion3D {
    pub lo: ICoord3D,
    pub hi: ICoord3D,
}

impl IRegion3D {
    #[inline]
    pub fn zero(&mut self) {
        self.lo.zero();
        self.hi.zero();
    }

    #[inline]
    pub fn width(&self) -> Int {
        self.hi.x.wrapping_sub(self.lo.x)
    }

    #[inline]
    pub fn height(&self) -> Int {
        self.hi.y.wrapping_sub(self.lo.y)
    }

    #[inline]
    pub fn depth(&self) -> Int {
        self.hi.z.wrapping_sub(self.lo.z)
    }

    #[inline]
    pub fn intersect_with(&mut self, other: &IRegion3D) {
        self.lo.update_max(&other.lo);
        self.hi.update_min(&other.hi);
    }

    #[inline]
    pub fn unite_with(&mut self, other: &IRegion3D) {
        self.lo.update_min(&other.lo);
        self.hi.update_max(&other.hi);
    }

    #[inline]
    pub fn unite_with_point(&mut self, point: &ICoord3D) {
        self.lo.update_min(point);
        self.hi.update_max(point);
    }
}

/// Real-valued range matching `RealRange.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RealRange {
    pub lo: Real,
    pub hi: Real,
}

impl RealRange {
    #[inline]
    pub const fn new(lo: Real, hi: Real) -> Self {
        Self { lo, hi }
    }

    #[inline]
    pub fn zero(&mut self) {
        self.lo = 0.0;
        self.hi = 0.0;
    }

    #[inline]
    pub fn combine(&mut self, other: &RealRange) {
        if other.lo < self.lo {
            self.lo = other.lo;
        }
        if other.hi > self.hi {
            self.hi = other.hi;
        }
    }
}

/// RGB floating-point color matching `RGBColor.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RGBColor {
    pub red: Real,
    pub green: Real,
    pub blue: Real,
}

impl RGBColor {
    #[inline]
    pub fn get_as_int(&self) -> Int {
        let r = ((self.red as f64) * 255.0) as Int;
        let g = ((self.green as f64) * 255.0) as Int;
        let b = ((self.blue as f64) * 255.0) as Int;
        (r << 16) | (g << 8) | b
    }

    #[inline]
    pub fn set_from_int(&mut self, c: Int) {
        self.red = ((c >> 16) & 0xff) as Real / 255.0;
        self.green = ((c >> 8) & 0xff) as Real / 255.0;
        self.blue = (c & 0xff) as Real / 255.0;
    }
}

/// RGBA floating-point color matching `RGBColor.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct RGBAColorReal {
    pub red: Real,
    pub green: Real,
    pub blue: Real,
    pub alpha: Real,
}

/// RGBA integer color matching `RGBColor.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RGBAColorInt {
    pub red: UnsignedInt,
    pub green: UnsignedInt,
    pub blue: UnsignedInt,
    pub alpha: UnsignedInt,
}

/// 3x4 row-major affine matrix matching `WWMath::Matrix3D` (`matrix3d.h:381`).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Matrix3D {
    pub rows: [[Real; 4]; 3],
}

impl Default for Matrix3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl Matrix3D {
    #[inline]
    pub const fn identity() -> Self {
        Self {
            rows: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
            ],
        }
    }

    #[inline]
    pub fn make_identity(&mut self) {
        *self = Self::identity();
    }

    #[inline]
    #[allow(clippy::too_many_arguments)]
    pub fn set(
        &mut self,
        m11: Real,
        m12: Real,
        m13: Real,
        m14: Real,
        m21: Real,
        m22: Real,
        m23: Real,
        m24: Real,
        m31: Real,
        m32: Real,
        m33: Real,
        m34: Real,
    ) {
        self.rows[0] = [m11, m12, m13, m14];
        self.rows[1] = [m21, m22, m23, m24];
        self.rows[2] = [m31, m32, m33, m34];
    }

    #[inline]
    pub fn set_axes(&mut self, x: &Coord3D, y: &Coord3D, z: &Coord3D, pos: &Coord3D) {
        self.rows[0] = [x.x, y.x, z.x, pos.x];
        self.rows[1] = [x.y, y.y, z.y, pos.y];
        self.rows[2] = [x.z, y.z, z.z, pos.z];
    }

    #[inline]
    pub const fn get_x_translation(&self) -> Real {
        self.rows[0][3]
    }

    #[inline]
    pub const fn get_y_translation(&self) -> Real {
        self.rows[1][3]
    }

    #[inline]
    pub const fn get_z_translation(&self) -> Real {
        self.rows[2][3]
    }

    #[inline]
    pub const fn get_translation(&self) -> Coord3D {
        Coord3D::new(self.rows[0][3], self.rows[1][3], self.rows[2][3])
    }

    #[inline]
    pub fn set_x_translation(&mut self, x: Real) {
        self.rows[0][3] = x;
    }

    #[inline]
    pub fn set_y_translation(&mut self, y: Real) {
        self.rows[1][3] = y;
    }

    #[inline]
    pub fn set_z_translation(&mut self, z: Real) {
        self.rows[2][3] = z;
    }

    #[inline]
    pub fn set_translation(&mut self, t: &Coord3D) {
        self.rows[0][3] = t.x;
        self.rows[1][3] = t.y;
        self.rows[2][3] = t.z;
    }

    #[inline]
    pub const fn get_x_vector(&self) -> Coord3D {
        Coord3D::new(self.rows[0][0], self.rows[1][0], self.rows[2][0])
    }

    #[inline]
    pub const fn get_y_vector(&self) -> Coord3D {
        Coord3D::new(self.rows[0][1], self.rows[1][1], self.rows[2][1])
    }

    #[inline]
    pub const fn get_z_vector(&self) -> Coord3D {
        Coord3D::new(self.rows[0][2], self.rows[1][2], self.rows[2][2])
    }

    #[inline]
    pub fn get_z_rotation(&self) -> Real {
        gm_atan2f(self.rows[1][0], self.rows[0][0])
    }

    #[inline]
    pub fn rotate_z(&mut self, theta: Real) {
        let c = gm_cosf(theta);
        let s = gm_sinf(theta);
        for row in &mut self.rows {
            let t1 = row[0];
            let t2 = row[1];
            row[0] = c * t1 + s * t2;
            row[1] = -s * t1 + c * t2;
        }
    }

    #[inline]
    pub fn translate(&mut self, x: Real, y: Real, z: Real) {
        for row in &mut self.rows {
            row[3] += row[0] * x + row[1] * y + row[2] * z;
        }
    }

    #[inline]
    pub fn translate_coord(&mut self, t: &Coord3D) {
        self.translate(t.x, t.y, t.z);
    }

    #[inline]
    pub fn transform_vector(tm: &Self, v: &Coord3D) -> Coord3D {
        Coord3D::new(
            tm.rows[0][0] * v.x + tm.rows[0][1] * v.y + tm.rows[0][2] * v.z + tm.rows[0][3],
            tm.rows[1][0] * v.x + tm.rows[1][1] * v.y + tm.rows[1][2] * v.z + tm.rows[1][3],
            tm.rows[2][0] * v.x + tm.rows[2][1] * v.y + tm.rows[2][2] * v.z + tm.rows[2][3],
        )
    }

    #[inline]
    pub fn mul(a: &Self, b: &Self) -> Self {
        let mut out = Self::identity();
        for r in 0..3 {
            let ar = a.rows[r];
            for c in 0..3 {
                out.rows[r][c] = ar[0] * b.rows[0][c] + ar[1] * b.rows[1][c] + ar[2] * b.rows[2][c];
            }
            out.rows[r][3] =
                ar[0] * b.rows[0][3] + ar[1] * b.rows[1][3] + ar[2] * b.rows[2][3] + ar[3];
        }
        out
    }
}
