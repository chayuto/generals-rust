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
        let mut circle = Self {
            edges: Vec::new(),
            y_pos: y_center,
            y_pos_doubled: y_center.wrapping_shl(1),
        };

        let reserve_count = radius.wrapping_shl(1);
        if reserve_count > 0 {
            circle.edges.reserve(reserve_count as usize);
        }

        circle.generate_edge_pairs(x_center, y_center, radius);
        circle.remove_duplicates();
        circle
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
        for edge in &self.edges {
            draw_fn(edge.x_start, edge.x_end, edge.y_pos);
            if edge.y_pos != self.y_pos {
                draw_fn(
                    edge.x_start,
                    edge.x_end,
                    self.y_pos_doubled.wrapping_sub(edge.y_pos),
                );
            }
        }
    }

    fn generate_edge_pairs(&mut self, x_center: Int, y_center: Int, radius: Int) {
        let mut x = 0;
        let mut y = radius;
        let mut d = (1i32.wrapping_sub(radius)).wrapping_shl(1);

        while y >= 0 {
            let edge = HorzLine {
                y_pos: y_center.wrapping_add(y),
                x_start: x_center.wrapping_sub(x),
                x_end: x_center.wrapping_add(x),
            };
            self.edges.push(edge);

            if d.wrapping_add(y) > 0 {
                y = y.wrapping_sub(1);
                d = d.wrapping_sub(y.wrapping_shl(1).wrapping_sub(1));
            }

            if x > d {
                x = x.wrapping_add(1);
                d = d.wrapping_add(x.wrapping_shl(1).wrapping_add(1));
            }
        }
    }

    fn remove_duplicates(&mut self) {
        let mut it = 0usize;
        while it + 1 < self.edges.len() {
            if self.edges[it].y_pos == self.edges[it + 1].y_pos {
                self.edges.remove(it);
            } else {
                it += 1;
            }
        }
    }
}
