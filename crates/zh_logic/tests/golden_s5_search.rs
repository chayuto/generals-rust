//! Replays `docs/golden_vectors/S5_Search.json`
//! (`tests/cpp_harness/gen_s5_search.cpp`): the open and closed lists, the
//! neighbour checks, the ground A* search, path building, the cost search, the
//! switch to the fixed code, the hierarchical searches, `findPath`, the ground
//! path and the request queue, with the whole search state after every call.

mod pathfind_scene;

use std::collections::BTreeMap;

use pathfind_scene::{rows, Scene, Tokens};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::ObjectId;
use zh_common::xfer::XferCrc;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::sim::{SimApi, SimContext};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S5_Search.json");

#[test]
fn searches_match_row_for_row() {
    let mut scene = Scene::default();
    let mut checked: BTreeMap<&str, u32> = BTreeMap::new();
    let mut found = 0;
    let mut scenario = "";
    for (line, row) in rows(VECTORS).into_iter().enumerate() {
        // `setup` and `records` have nothing before the separator.
        let (input, want) = match row.split_once(" = ") {
            Some((input, want)) => (input, Some(want)),
            None => (row.strip_suffix(" =").unwrap_or(row), None),
        };
        let mut t = Tokens(input.split_ascii_whitespace());
        let op = t.text();
        if op == "scenario" {
            scenario = row;
        }
        if scene.setup(op, &mut t) {
            continue;
        }
        let got = scene.call(op, &mut t);
        let want = want.unwrap_or("");
        if op == "records" {
            let (got, want): (Vec<&str>, Vec<&str>) =
                (got.split(' ').collect(), want.split(' ').collect());
            for (a, b) in got.iter().zip(&want) {
                assert_eq!(a, b, "{scenario} row {line}: record");
            }
            assert_eq!(got.len(), want.len(), "{scenario} row {line}: records");
        } else {
            assert_eq!(got, want, "{scenario} row {line}: {input:.200}");
        }
        if op == "find" && !want.starts_with("null") {
            found += 1;
        }
        *checked.entry(op).or_default() += 1;
    }
    for (op, least) in [
        ("find", 400),
        ("cost", 400),
        ("examine", 200),
        ("adjust", 60),
        ("clip", 400),
        ("chain", 5),
        ("lists", 4),
        ("cap", 3),
        ("grand", 20),
        ("cut", 10),
        ("setup", 40),
        ("bridge", 20),
        ("hier", 170),
        ("path", 170),
        ("exists", 60),
        ("ground", 170),
        ("serve", 200),
        ("queue", 700),
        ("frameend", 80),
        ("hoard", 30),
        ("records", 10),
    ] {
        assert!(
            checked.get(op).is_some_and(|&c| c >= least),
            "too few {op} rows: {checked:?}"
        );
    }
    assert!(found > 120, "only {found} paths found");
}

fn pathfinder_crc(sim: &SimContext) -> u32 {
    let mut xfer = XferCrc::new();
    let pathfinder = sim.world.pathfinder.as_deref().expect("map");
    pathfinder.crc(&mut xfer).expect("crc");
    xfer.get_crc()
}

/// A world with a pathfinder queues on it, and the frame step serves it.
#[test]
fn sim_queues_paths_on_the_map() {
    let mut sim = SimContext::new();
    let extent = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(200.0, 200.0, 100.0),
    };
    sim.new_pathfind_map(&extent, 12.0);
    let empty = pathfinder_crc(&sim);
    assert_eq!(SimApi::queue_for_path(&mut sim, ObjectId(77)), 1);
    let queued = pathfinder_crc(&sim);
    assert_ne!(queued, empty);
    // A frame that has to recalculate the zones takes no request.
    let mut frames = 0;
    while pathfinder_crc(&sim) == queued {
        run_frame(&mut sim, &mut NoFrameSeams);
        frames += 1;
        assert!(frames < 400, "request never taken");
    }
}
