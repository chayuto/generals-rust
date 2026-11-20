//! `PKT-016`: Deterministic `BezierSegment` & `BezierMath` translation target.
//!
//! Reference C++ packet: `packets/phase2/PKT-016_BezierSegment.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt016.rs` (`docs/golden_vectors/PKT-016.json`)
//!
//! Note: In `BezierSegment::getApproximateLength`, C++ `INT_TO_REAL(length0 + length1)` is defined
//! in `BaseMacros.h:80` as `#define INT_TO_REAL(x) ((Real)(x))` (i.e. a direct `(Real)` cast with
//! NO integer truncation!).

#![allow(unused_variables)]

use crate::math::Coord3D;
use crate::types::{Int, Real};

pub const USUAL_TOLERANCE: Real = 1.0;

pub const BEZ_BASIS_MATRIX: [[Real; 4]; 4] = [
    [-1.0, 3.0, -3.0, 1.0],
    [3.0, -6.0, 3.0, 0.0],
    [-3.0, 3.0, 0.0, 0.0],
    [1.0, 0.0, 0.0, 0.0],
];

pub struct BezierMath;

impl BezierMath {
    pub fn vec4_transform(v: [Real; 4], m: &[[Real; 4]; 4]) -> [Real; 4] {
        [
            ((v[0] * m[0][0] + v[1] * m[1][0]) + v[2] * m[2][0]) + v[3] * m[3][0],
            ((v[0] * m[0][1] + v[1] * m[1][1]) + v[2] * m[2][1]) + v[3] * m[3][1],
            ((v[0] * m[0][2] + v[1] * m[1][2]) + v[2] * m[2][2]) + v[3] * m[3][2],
            ((v[0] * m[0][3] + v[1] * m[1][3]) + v[2] * m[2][3]) + v[3] * m[3][3],
        ]
    }

    pub fn vec4_dot(a: [Real; 4], b: [Real; 4]) -> Real {
        ((a[0] * b[0] + a[1] * b[1]) + a[2] * b[2]) + a[3] * b[3]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BezierSegment {
    pub control_points: [Coord3D; 4],
}

impl BezierSegment {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_coords(cp: &[Real; 12]) -> Self {
        Self::from_points(
            Coord3D::new(cp[0], cp[1], cp[2]),
            Coord3D::new(cp[3], cp[4], cp[5]),
            Coord3D::new(cp[6], cp[7], cp[8]),
            Coord3D::new(cp[9], cp[10], cp[11]),
        )
    }

    pub fn from_points(cp0: Coord3D, cp1: Coord3D, cp2: Coord3D, cp3: Coord3D) -> Self {
        Self {
            control_points: [cp0, cp1, cp2, cp3],
        }
    }

    pub fn evaluate_bez_segment_at_t(&self, t_value: Real, out_result: &mut Coord3D) {
        let t2 = t_value * t_value;
        let t_vec = [t_value * t2, t2, t_value, 1.0];
        let x = [
            self.control_points[0].x,
            self.control_points[1].x,
            self.control_points[2].x,
            self.control_points[3].x,
        ];
        let y = [
            self.control_points[0].y,
            self.control_points[1].y,
            self.control_points[2].y,
            self.control_points[3].y,
        ];
        let z = [
            self.control_points[0].z,
            self.control_points[1].z,
            self.control_points[2].z,
            self.control_points[3].z,
        ];
        let t_result = BezierMath::vec4_transform(t_vec, &BEZ_BASIS_MATRIX);
        out_result.x = BezierMath::vec4_dot(x, t_result);
        out_result.y = BezierMath::vec4_dot(y, t_result);
        out_result.z = BezierMath::vec4_dot(z, t_result);
    }

    pub fn get_segment_points(&self, num_segments: Int, out_result: &mut Vec<Coord3D>) {
        out_result.clear();
        out_result.resize(num_segments as usize, Coord3D::default());
        let mut iter =
            crate::math::bez_fwd_iterator::BezFwdIterator::with_segment(num_segments, self);
        iter.start();
        let mut i = 0usize;
        while iter.done() == crate::types::FALSE {
            out_result[i] = *iter.get_current();
            i += 1;
            iter.next();
        }
    }

    pub fn get_approximate_length(&self, within_tolerance: Real) -> Real {
        let p0p1 = Coord3D::new(
            self.control_points[1].x - self.control_points[0].x,
            self.control_points[1].y - self.control_points[0].y,
            self.control_points[1].z - self.control_points[0].z,
        );
        let p1p2 = Coord3D::new(
            self.control_points[2].x - self.control_points[1].x,
            self.control_points[2].y - self.control_points[1].y,
            self.control_points[2].z - self.control_points[1].z,
        );
        let p2p3 = Coord3D::new(
            self.control_points[3].x - self.control_points[2].x,
            self.control_points[3].y - self.control_points[2].y,
            self.control_points[3].z - self.control_points[2].z,
        );
        let p0p3 = Coord3D::new(
            self.control_points[3].x - self.control_points[0].x,
            self.control_points[3].y - self.control_points[0].y,
            self.control_points[3].z - self.control_points[0].z,
        );
        let length0 = p0p3.length();
        let length1 = p0p1.length() + p1p2.length() + p2p3.length();
        if length1 - length0 > within_tolerance {
            let mut seg1 = Self::default();
            let mut seg2 = Self::default();
            self.split_segment_at_t(0.5, &mut seg1, &mut seg2);
            return seg1.get_approximate_length(within_tolerance)
                + seg2.get_approximate_length(within_tolerance);
        }
        (length0 + length1) / 2.0
    }

    pub fn split_segment_at_t(
        &self,
        t_value: Real,
        out_seg1: &mut BezierSegment,
        out_seg2: &mut BezierSegment,
    ) {
        let mut p0p1 = Coord3D::new(
            self.control_points[1].x - self.control_points[0].x,
            self.control_points[1].y - self.control_points[0].y,
            self.control_points[1].z - self.control_points[0].z,
        );
        let mut p1p2 = Coord3D::new(
            self.control_points[2].x - self.control_points[1].x,
            self.control_points[2].y - self.control_points[1].y,
            self.control_points[2].z - self.control_points[1].z,
        );
        let mut p2p3 = Coord3D::new(
            self.control_points[3].x - self.control_points[2].x,
            self.control_points[3].y - self.control_points[2].y,
            self.control_points[3].z - self.control_points[2].z,
        );
        p0p1.scale(t_value);
        p1p2.scale(t_value);
        p2p3.scale(t_value);
        p0p1.add(&self.control_points[0]);
        p1p2.add(&self.control_points[1]);
        p2p3.add(&self.control_points[2]);
        let mut tri_left = Coord3D::new(p1p2.x - p0p1.x, p1p2.y - p0p1.y, p1p2.z - p0p1.z);
        let mut tri_right = Coord3D::new(p2p3.x - p1p2.x, p2p3.y - p1p2.y, p2p3.z - p1p2.z);
        tri_left.scale(t_value);
        tri_right.scale(t_value);
        tri_left.add(&p0p1);
        tri_right.add(&p1p2);
        out_seg1.control_points[0] = self.control_points[0];
        out_seg1.control_points[1] = p0p1;
        out_seg1.control_points[2] = tri_left;
        self.evaluate_bez_segment_at_t(t_value, &mut out_seg1.control_points[3]);
        out_seg2.control_points[0] = out_seg1.control_points[3];
        out_seg2.control_points[1] = tri_right;
        out_seg2.control_points[2] = p2p3;
        out_seg2.control_points[3] = self.control_points[3];
    }
}
