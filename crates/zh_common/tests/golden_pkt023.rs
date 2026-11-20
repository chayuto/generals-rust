//! Gate V2-Golden test suite for `PKT-023` (`PartitionSolver`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-023`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

use zh_common::golden_json::{extract_vector_lines, get_u32};
use zh_common::math::partition_solver::{PartitionSolver, SolutionType};
use zh_common::types::ObjectId;

const PKT023_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-023.json");

fn splitmix32(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x9e37_79b9);
    let mut z = *state;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

fn rotate_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[test]
#[ignore = "Un-ignore when implementing PKT-023 (crates/zh_common/src/math/partition_solver.rs)"]
fn test_pkt023_partition_solver_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT023_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-023.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let mut rng = seed.wrapping_mul(0x9e37_79b9).wrapping_add(0x5555_5555);
        let n_entries = (splitmix32(&mut rng) % 7) + 1;
        let n_spaces = (splitmix32(&mut rng) % 4) + 1;

        let mut entries = Vec::with_capacity(n_entries as usize);
        let mut spaces = Vec::with_capacity(n_spaces as usize);
        for i in 0..n_entries {
            let id = ObjectId((100 + i) as i32);
            let sz = ((splitmix32(&mut rng) % 30) + 1) * 8 + i;
            entries.push((id, sz));
        }
        for j in 0..n_spaces {
            let id = ObjectId((500 + j) as i32);
            let sz = ((splitmix32(&mut rng) % 60) + 10) * 8 + j;
            spaces.push((id, sz));
        }

        let mut solver = PartitionSolver::new(&entries, &spaces, SolutionType::PreferFastSolution);
        solver.solve();
        let sol = solver.get_solution();

        assert_eq!(
            sol.len() as u32,
            get_u32(line, "sol_len").unwrap(),
            "seed {} sol_len",
            seed
        );

        let mut sol_crc: u32 = 0;
        for &(entry_id, space_id) in sol {
            sol_crc = rotate_crc(sol_crc, entry_id.0 as u32);
            sol_crc = rotate_crc(sol_crc, space_id.0 as u32);
        }
        assert_eq!(
            sol_crc,
            get_u32(line, "sol_crc").unwrap(),
            "seed {} sol_crc",
            seed
        );
    }
}
