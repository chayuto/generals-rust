//! `PKT-015`: Deterministic `BezFwdIterator` translation target.
//!
//! Reference C++ packet: `packets/phase2/PKT-015_BezFwdIterator.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt016.rs` (`docs/golden_vectors/PKT-016.json`)

#![allow(unused_variables)]

use crate::math::bezier_segment::BezierSegment;
use crate::math::Coord3D;
use crate::types::{Bool, Int, Real};

#[derive(Debug, Clone, Default)]
pub struct BezFwdIterator {
    pub step: Int,
    pub steps_desired: Int,
    pub bez_seg: BezierSegment,
    pub curr_point: Coord3D,
    pub dq: Coord3D,
    pub ddq: Coord3D,
    pub dddq: Coord3D,
}

impl BezFwdIterator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_segment(steps_desired: Int, bez_seg: &BezierSegment) -> Self {
        Self {
            steps_desired,
            bez_seg: *bez_seg,
            ..Self::default()
        }
    }

    pub fn start(&mut self) {
        self.step = 0;
        if self.steps_desired <= 1 {
            return;
        }
        let d = 1.0 / (self.steps_desired - 1) as Real;
        let d2 = d * d;
        let d3 = d * d2;
        let axes = [
            [
                self.bez_seg.control_points[0].x,
                self.bez_seg.control_points[1].x,
                self.bez_seg.control_points[2].x,
                self.bez_seg.control_points[3].x,
            ],
            [
                self.bez_seg.control_points[0].y,
                self.bez_seg.control_points[1].y,
                self.bez_seg.control_points[2].y,
                self.bez_seg.control_points[3].y,
            ],
            [
                self.bez_seg.control_points[0].z,
                self.bez_seg.control_points[1].z,
                self.bez_seg.control_points[2].z,
                self.bez_seg.control_points[3].z,
            ],
        ];
        let mut coeffs = [[0.0; 4]; 3];
        for axis in 0..3 {
            coeffs[axis] = crate::math::bezier_segment::BezierMath::vec4_transform(
                axes[axis],
                &crate::math::bezier_segment::BEZ_BASIS_MATRIX,
            );
        }
        self.curr_point = self.bez_seg.control_points[0];
        for axis in 0..3 {
            let a = coeffs[axis][0];
            let b = coeffs[axis][1];
            let c = coeffs[axis][2];
            let d1 = a * d3 + b * d2 + c * d;
            let dd = 6.0 * a * d3 + 2.0 * b * d2;
            let ddd = 6.0 * a * d3;
            match axis {
                0 => {
                    self.dq.x = d1;
                    self.ddq.x = dd;
                    self.dddq.x = ddd;
                }
                1 => {
                    self.dq.y = d1;
                    self.ddq.y = dd;
                    self.dddq.y = ddd;
                }
                _ => {
                    self.dq.z = d1;
                    self.ddq.z = dd;
                    self.dddq.z = ddd;
                }
            }
        }
    }

    pub fn done(&self) -> Bool {
        if self.step >= self.steps_desired {
            crate::types::TRUE
        } else {
            crate::types::FALSE
        }
    }

    pub fn get_current(&self) -> &Coord3D {
        &self.curr_point
    }

    pub fn next(&mut self) {
        self.curr_point.add(&self.dq);
        self.dq.add(&self.ddq);
        self.ddq.add(&self.dddq);
        self.step += 1;
    }
}
