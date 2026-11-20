//! `PKT-017`: Deterministic `DiscreteCircle` Bresenham scanline generator translation target.
//!
//! Reference C++ packet: `packets/phase2/PKT-017_DiscreteCircle.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt017.rs` (`docs/golden_vectors/PKT-017.json`)

#![allow(unused_variables, unused_mut, dead_code)]

use crate::types::Int;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HorzLine {
    pub y_pos: Int,
    pub x_start: Int,
    pub x_end: Int,
}

#[derive(Debug, Clone, Default)]
pub struct DiscreteCircle {
    edges: Vec<HorzLine>,
    y_pos: Int,
    y_pos_doubled: Int,
}

impl DiscreteCircle {
    pub fn new(x_center: Int, y_center: Int, radius: Int) -> Self {
        todo!("PKT-017: Implement DiscreteCircle::new from packets/phase2/PKT-017_DiscreteCircle.md")
    }

    pub fn get_edges(&self) -> &[HorzLine] {
        &self.edges
    }

    pub fn get_edge_count(&self) -> Int {
        self.edges.len() as Int
    }

    pub fn draw_circle<F>(&self, mut draw_fn: F)
    where
        F: FnMut(Int, Int, Int),
    {
        todo!("PKT-017: Implement DiscreteCircle::draw_circle from packets/phase2/PKT-017_DiscreteCircle.md")
    }
}
