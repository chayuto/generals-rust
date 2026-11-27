//! The `WWMath` operations bone transforms go through (`matrix3d.h`, `quat.cpp`,
//! `wwmath.h`), in the reference evaluation order. Tables are those built with
//! deterministic math and `RETAIL_COMPATIBLE_CRC=0`.

use std::sync::OnceLock;

use zh_common::math::{gm_acosf, gm_lrintf, gm_sinf};

const PI: f32 = std::f32::consts::PI;
const EPSILON: f32 = 0.0001;
const TABLE: usize = 1024;

struct Tables {
    acos: [f32; TABLE],
    sin: [f32; TABLE],
}

fn tables() -> &'static Tables {
    static TABLES: OnceLock<Tables> = OnceLock::new();
    TABLES.get_or_init(|| {
        let mut t = Tables {
            acos: [0.0; TABLE],
            sin: [0.0; TABLE],
        };
        for (a, v) in t.acos.iter_mut().enumerate() {
            let cv = (a as i32 - 512) as f32 * (1.0 / 512.0);
            *v = gm_acosf(cv);
        }
        for (a, v) in t.sin.iter_mut().enumerate() {
            let cv = a as f32 * 2.0 * PI / TABLE as f32;
            *v = gm_sinf(cv);
        }
        t
    })
}

/// `WWMath::Float_To_Long`.
pub fn float_to_long(f: f32) -> i32 {
    gm_lrintf(f) as i32
}

/// `WWMath::Float_To_Int_Floor`, shift counts wrapping as on the reference CPU.
fn float_to_int_floor(f: f32) -> i32 {
    let mut a = f.to_bits() as i32;
    let sign = a >> 31;
    a &= 0x7fff_ffff;
    let exponent = (a >> 23) - 127;
    let expsign = !(exponent >> 31);
    let shift = 31i32.wrapping_sub(exponent) as u32;
    let imask = 1i32.wrapping_shl(shift).wrapping_sub(1);
    let mantissa = a & ((1 << 23) - 1);
    let r = ((mantissa | (1 << 23)) as u32)
        .wrapping_shl(8)
        .wrapping_shr(shift) as i32;
    let exact = i32::from(mantissa.wrapping_shl(8) & imask == 0);
    ((r & expsign) ^ sign).wrapping_add((exact & (expsign ^ (a.wrapping_sub(1) >> 31))) & sign)
}

/// `WWMath::Lerp`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

fn fast_acos(mut val: f32) -> f32 {
    if val.abs() > 0.975 {
        return gm_acosf(val);
    }
    val *= 512.0;
    let idx0 = float_to_int_floor(val);
    let frac = val - idx0 as f32;
    let i = (idx0 + 512) as usize;
    let t = &tables().acos;
    (1.0 - frac) * t[i] + frac * t[i + 1]
}

fn fast_sin(mut val: f32) -> f32 {
    val *= TABLE as f32 / (2.0 * PI);
    let idx0 = float_to_int_floor(val);
    let idx1 = idx0.wrapping_add(1);
    let frac = val - idx0 as f32;
    let t = &tables().sin;
    let i0 = idx0 as u32 as usize & (TABLE - 1);
    let i1 = idx1 as u32 as usize & (TABLE - 1);
    (1.0 - frac) * t[i0] + frac * t[i1]
}

#[cfg(test)]
mod tests {
    use super::float_to_int_floor;

    #[test]
    fn floors_exact_and_negative_values() {
        let cases = [
            (0.0, 0),
            (0.5, 0),
            (3.75, 3),
            (-0.5, -1),
            (-1.0, -1),
            (-2.0, -2),
            (-2.5, -3),
            (511.0, 511),
            (-511.0, -511),
        ];
        for (f, floor) in cases {
            assert_eq!(float_to_int_floor(f), floor, "{f}");
        }
    }
}

/// X, Y, Z, W.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quat(pub [f32; 4]);

impl Quat {
    pub const IDENTITY: Quat = Quat([0.0, 0.0, 0.0, 1.0]);
}

/// `Fast_Slerp`.
pub fn fast_slerp(p: Quat, q: Quat, mut alpha: f32) -> Quat {
    let (p, q) = (p.0, q.0);
    let mut cos_t = p[0] * q[0] + p[1] * q[1] + p[2] * q[2] + p[3] * q[3];
    let flip = cos_t < 0.0;
    if flip {
        cos_t = -cos_t;
    }
    let beta;
    if 1.0 - cos_t < EPSILON * EPSILON {
        beta = 1.0 - alpha;
    } else {
        let theta = fast_acos(cos_t);
        let oo_sin_t = 1.0 / fast_sin(theta);
        beta = fast_sin(theta - alpha * theta) * oo_sin_t;
        alpha = fast_sin(alpha * theta) * oo_sin_t;
    }
    if flip {
        alpha = -alpha;
    }
    Quat(std::array::from_fn(|i| beta * p[i] + alpha * q[i]))
}

/// Three rows of four: rotation and, in the last column, translation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Matrix3D(pub [[f32; 4]; 3]);

impl Matrix3D {
    pub const IDENTITY: Matrix3D = Matrix3D([
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ]);

    /// `Build_Matrix3D`: the diagonal and cross terms are formed in `double`.
    pub fn from_quat(q: Quat) -> Matrix3D {
        let q = q.0;
        let two = |v: f32| (2.0 * f64::from(v)) as f32;
        let one_minus_two = |v: f32| (1.0 - 2.0 * f64::from(v)) as f32;
        Matrix3D([
            [
                one_minus_two(q[1] * q[1] + q[2] * q[2]),
                two(q[0] * q[1] - q[2] * q[3]),
                two(q[2] * q[0] + q[1] * q[3]),
                0.0,
            ],
            [
                two(q[0] * q[1] + q[2] * q[3]),
                (1.0 - f64::from(2.0f32 * (q[2] * q[2] + q[0] * q[0]))) as f32,
                two(q[1] * q[2] - q[0] * q[3]),
                0.0,
            ],
            [
                two(q[2] * q[0] - q[1] * q[3]),
                two(q[1] * q[2] + q[0] * q[3]),
                one_minus_two(q[1] * q[1] + q[0] * q[0]),
                0.0,
            ],
        ])
    }

    /// `Translate`: post-multiplies by a translation.
    pub fn translate(&mut self, t: [f32; 3]) {
        for r in &mut self.0 {
            r[3] += r[0] * t[0] + r[1] * t[1] + r[2] * t[2];
        }
    }

    /// `a * b` (`Matrix3D::Multiply` and `postMul`).
    pub fn mul(a: &Matrix3D, b: &Matrix3D) -> Matrix3D {
        let mut out = Matrix3D::IDENTITY;
        for (r, row) in out.0.iter_mut().enumerate() {
            let a = a.0[r];
            for (c, v) in row.iter_mut().enumerate() {
                *v = a[0] * b.0[0][c] + a[1] * b.0[1][c] + a[2] * b.0[2][c];
            }
            row[3] += a[3];
        }
        out
    }

    /// `Scale(float)`: scales the three axis columns.
    pub fn scale(&mut self, s: f32) {
        for r in &mut self.0 {
            r[0] *= s;
            r[1] *= s;
            r[2] *= s;
        }
    }
}
