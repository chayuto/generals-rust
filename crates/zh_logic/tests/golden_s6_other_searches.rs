//! The other path searches against `tests/cpp_harness/gen_s6_search.cpp`:
//! `findClosestPath`, `patchPath`, `getMoveAwayFromPath`, `findSafePath` and
//! `findAttackPath`. Every row is replayed in order on the scene the S5 suite
//! builds, comparing paths and the state of every cell.
//!
//! The weapon and view answers of the attack search come from the table each
//! row carries.

mod pathfind_scene;

use std::collections::BTreeMap;

use pathfind_scene::{c3, rows, Scene, Tokens};
use zh_common::math::Coord3D;
use zh_common::types::Real;
use zh_logic::pathfinder::{AttackProbe, AttackRequest, AttackVictim, Path, Pathfinder};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S6_Search.json");

/// A path as a row gives it: positions, layers and next optimised nodes.
fn read_path(t: &mut Tokens) -> Option<Path> {
    let count = t.int();
    if count == 0 {
        return None;
    }
    let mut path = Path::new();
    let mut next = Vec::new();
    for _ in 0..count {
        let (position, layer) = (t.point(), t.layer());
        path.append_node(&position, layer);
        next.push(t.int());
    }
    let mut nodes = Vec::new();
    let mut cursor = path.first_node();
    while let Some(node) = cursor {
        nodes.push(node);
        cursor = path.node(node).next();
    }
    assert_eq!(nodes.len(), count as usize, "nodes were merged");
    for (&node, &to) in nodes.iter().zip(&next) {
        if to >= 0 {
            path.set_next_optimized(node, Some(nodes[to as usize]));
        }
    }
    Some(path)
}

/// The harness's answers for every position the weapon accepts: whether the
/// view from it is blocked.
struct Table {
    distance: Real,
    in_range: BTreeMap<(u32, u32), bool>,
}

fn key(position: &Coord3D) -> (u32, u32) {
    (position.x.to_bits(), position.y.to_bits())
}

impl AttackProbe for Table {
    fn attack_distance(&mut self) -> Real {
        self.distance
    }
    fn in_range(&mut self, position: &Coord3D) -> bool {
        self.in_range.contains_key(&key(position))
    }
    fn view_blocked(&mut self, _pathfinder: &Pathfinder, position: &Coord3D) -> bool {
        self.in_range[&key(position)]
    }
}

/// Makes the call of one of this suite's rows.
fn search(scene: &mut Scene, op: &str, t: &mut Tokens) -> Option<String> {
    let out = match op {
        "closest" => {
            let m = scene.mover(t);
            let (from, mut to) = (t.point(), t.point());
            let (blocked, multiplier) = (t.flag(), t.real());
            let path = scene.pf.find_closest_path(
                &scene.terrain,
                &scene.units,
                &m.unit,
                &from,
                &mut to,
                blocked,
                multiplier,
            );
            format!(
                "{} {} {} | {}",
                Scene::path_dump(path.as_ref()),
                c3(&to),
                scene.pf.ignore_obstacle_id.0,
                scene.state()
            )
        }
        "patch" => {
            let m = scene.mover(t);
            let position = t.point();
            let blocked = t.flag();
            let original = read_path(t).expect("original path");
            let path = scene.pf.patch_path(
                &scene.terrain,
                &scene.units,
                &m.unit,
                &position,
                &original,
                blocked,
            );
            format!("{} | {}", Scene::path_dump(path.as_ref()), scene.state())
        }
        "moveaway" => {
            let m = scene.mover(t);
            let position = t.point();
            scene.pf.is_tunneling = t.flag();
            let other = t.int();
            let avoid = read_path(t).expect("path to avoid");
            let avoid2 = read_path(t);
            let other = scene.units.info[&other].agent.clone();
            let path = scene.pf.get_move_away_from_path(
                &scene.terrain,
                &scene.units,
                &m.unit,
                &position,
                Some(&other),
                &avoid,
                avoid2.as_ref(),
            );
            let out = format!("{} | {}", Scene::path_dump(path.as_ref()), scene.state());
            scene.pf.is_tunneling = false;
            out
        }
        "safe" => {
            let m = scene.mover(t);
            let position = t.point();
            scene.pf.is_tunneling = t.flag();
            let (first, second, radius) = (t.point(), t.point(), t.real());
            let path = scene.pf.find_safe_path(
                &scene.terrain,
                &scene.units,
                &m.unit,
                &position,
                &first,
                &second,
                radius,
            );
            let out = format!("{} | {}", Scene::path_dump(path.as_ref()), scene.state());
            scene.pf.is_tunneling = false;
            out
        }
        "attack" => {
            let m = scene.mover(t);
            let position = t.point();
            scene.pf.is_tunneling = t.flag();
            let (has_victim, above) = (t.flag(), t.flag());
            let (victim_at, target) = (t.point(), t.point());
            let distance = t.real();
            let mut table = Table {
                distance,
                in_range: BTreeMap::new(),
            };
            for _ in 0..t.int() {
                let at = (t.real().to_bits(), t.real().to_bits());
                table.in_range.insert(at, t.flag());
            }
            let request = AttackRequest {
                unit: &m.unit,
                position,
                from: position,
                victim: has_victim.then_some(AttackVictim {
                    position: victim_at,
                    significantly_above_terrain: above,
                }),
                victim_position: target,
            };
            let path =
                scene
                    .pf
                    .find_attack_path(&scene.terrain, &scene.units, &mut table, &request);
            let out = format!("{} | {}", Scene::path_dump(path.as_ref()), scene.state());
            scene.pf.is_tunneling = false;
            out
        }
        _ => return None,
    };
    scene.done();
    Some(out)
}

#[test]
fn searches_match_row_for_row() {
    let mut scene = Scene::default();
    let mut checked: BTreeMap<&str, (u32, u32)> = BTreeMap::new();
    let mut scenario = "";
    for (line, row) in rows(VECTORS).into_iter().enumerate() {
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
        let got = match search(&mut scene, op, &mut t) {
            Some(got) => got,
            None => scene.call(op, &mut t),
        };
        let want = want.unwrap_or("");
        assert_eq!(got, want, "{scenario} row {line}: {input:.300}");
        let count = checked.entry(op).or_default();
        if want.starts_with("null") {
            count.1 += 1;
        } else {
            count.0 += 1;
        }
    }
    // Rows that found a path, and rows that did not.
    for (op, found, missed) in [
        ("closest", 400, 30),
        ("patch", 200, 250),
        ("moveaway", 200, 10),
        ("safe", 150, 40),
        ("attack", 180, 300),
    ] {
        assert!(
            checked
                .get(op)
                .is_some_and(|&(f, m)| f >= found && m >= missed),
            "too few {op} rows: {checked:?}"
        );
    }
}
