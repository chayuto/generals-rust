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
        todo!("PKT-023: Implement PartitionSolver::new from packets/phase2/PKT-023_PartitionSolver.md")
    }

    pub fn solve(&mut self) {
        todo!("PKT-023: Implement PartitionSolver::solve from packets/phase2/PKT-023_PartitionSolver.md")
    }

    pub fn get_solution(&self) -> &[PairObjectId] {
        &self.best_solution
    }
}
