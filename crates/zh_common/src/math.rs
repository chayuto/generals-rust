//! Deterministic C++ promotion and IEEE-754 binary32 math helpers.

/// Replicates C++ compound mixed-type assignment `Int a; Real s; a *= s;`
/// (`a = static_cast<Int>(static_cast<Real>(a) * s)` with truncation toward zero).
#[inline]
pub fn cpp_mul_i32_f32(a: i32, s: f32) -> i32 {
    ((a as f32) * s) as i32
}
