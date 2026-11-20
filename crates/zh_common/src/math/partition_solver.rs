//! `PKT-023`: Deterministic `PartitionSolver` translation target.
//!
//! Reference C++ packet: `packets/phase2/PKT-023_PartitionSolver.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt023.rs` (`docs/golden_vectors/PKT-023.json`)

#![allow(unused_variables, dead_code)]

use crate::types::{ObjectId, UnsignedInt};

pub type PairObjectIdAndUInt = (ObjectId, UnsignedInt);
pub type PairObjectId = (ObjectId, ObjectId);
pub type EntriesVec = Vec<PairObjectIdAndUInt>;
pub type SpacesVec = Vec<PairObjectIdAndUInt>;
pub type SolutionVec = Vec<PairObjectId>;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SolutionType {
    #[default]
    PreferFastSolution = 0,
    PreferCorrectSolution = 0x7FFF_FFFF,
}

#[derive(Debug, Clone)]
pub struct PartitionSolver {
    how_to_solve: SolutionType,
    data: EntriesVec,
    spaces_for_data: SpacesVec,
    current_solution: SolutionVec,
    current_solution_leftovers: UnsignedInt,
    best_solution: SolutionVec,
}

impl PartitionSolver {
    pub fn new(elements: &[PairObjectIdAndUInt], spaces: &[PairObjectIdAndUInt], solve_how: SolutionType) -> Self {
        Self {
            how_to_solve: solve_how,
            data: elements.to_vec(),
            spaces_for_data: spaces.to_vec(),
            current_solution: Vec::new(),
            current_solution_leftovers: 0,
            best_solution: Vec::new(),
        }
    }

    pub fn solve(&mut self) {
        self.best_solution.clear();
        self.current_solution.clear();
        self.current_solution_leftovers = 0x7fff_ffff;

        self.data.sort_by(|a, b| b.1.cmp(&a.1));
        self.spaces_for_data.sort_by(|a, b| b.1.cmp(&a.1));

        if self.how_to_solve == SolutionType::PreferFastSolution {
            let mut spaces_available = self.spaces_for_data.clone();
            for &(entry_id, entry_size) in &self.data {
                for j in 0..spaces_available.len() {
                    let (space_id, space_size) = spaces_available[j];
                    if entry_size <= space_size {
                        spaces_available[j].1 -= entry_size;
                        self.best_solution.push((entry_id, space_id));
                        break;
                    }
                }
            }
        }
    }

    pub fn get_solution(&self) -> &[PairObjectId] {
        &self.best_solution
    }
}
