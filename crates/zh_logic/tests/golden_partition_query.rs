//! Golden test for `spatial::partition_query`, the searches of
//! `PartitionManager`. Oracle: `tests/cpp_harness/gen_partition_query.cpp`, which
//! also describes the row format. Recorded without the retail features.

use zh_common::golden_json::get_i32;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::ObjectId;
use zh_logic::spatial::partition_grid::{PartitionGrid, PartitionObject};
use zh_logic::spatial::partition_query::{DistanceCalculationType, IterOrderType, QueryCenter};

const JSON: &str = include_str!("../../../docs/golden_vectors/PARTITION_QUERY.json");

/// The subset of JSON the world rows use: integers, escape-free strings and
/// arrays.
#[derive(Debug)]
enum Value {
    Number(i64),
    Text(String),
    List(Vec<Value>),
}

fn skip_separators(bytes: &[u8], at: &mut usize) {
    while matches!(bytes[*at], b',' | b'\n' | b' ') {
        *at += 1;
    }
}

impl Value {
    fn parse(bytes: &[u8], at: &mut usize) -> Value {
        skip_separators(bytes, at);
        match bytes[*at] {
            b'[' => {
                *at += 1;
                let mut items = Vec::new();
                loop {
                    skip_separators(bytes, at);
                    if bytes[*at] == b']' {
                        break;
                    }
                    items.push(Value::parse(bytes, at));
                }
                *at += 1;
                Value::List(items)
            }
            b'"' => {
                let start = *at + 1;
                let mut end = start;
                while bytes[end] != b'"' {
                    end += 1;
                }
                *at = end + 1;
                Value::Text(String::from_utf8(bytes[start..end].to_vec()).unwrap())
            }
            _ => {
                let start = *at;
                while bytes[*at] == b'-' || bytes[*at].is_ascii_digit() {
                    *at += 1;
                }
                let text = std::str::from_utf8(&bytes[start..*at]).unwrap();
                Value::Number(text.parse().unwrap())
            }
        }
    }

    fn list(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            other => panic!("expected a list, got {other:?}"),
        }
    }

    fn number(&self) -> i64 {
        match self {
            Value::Number(n) => *n,
            other => panic!("expected a number, got {other:?}"),
        }
    }

    fn bits(&self) -> u32 {
        self.number() as u32
    }

    fn real(&self) -> f32 {
        f32::from_bits(self.bits())
    }

    fn text(&self) -> &str {
        match self {
            Value::Text(s) => s,
            other => panic!("expected a string, got {other:?}"),
        }
    }
}

fn geometry(fields: &[Value], context: &str) -> GeometryInfo {
    let geom = match fields[0].number() {
        0 => GeometryType::Sphere,
        1 => GeometryType::Cylinder,
        2 => GeometryType::Box,
        other => panic!("geometry type {other}"),
    };
    let info = GeometryInfo::new(
        geom,
        u8::from(fields[1].number() != 0),
        fields[2].real(),
        fields[3].real(),
        fields[4].real(),
    );
    assert_eq!(
        [
            info.get_bounding_circle_radius().to_bits(),
            info.get_bounding_sphere_radius().to_bits(),
            info.get_z_delta_to_center_position().to_bits(),
        ],
        [fields[5].bits(), fields[6].bits(), fields[7].bits()],
        "{context}: derived geometry"
    );
    info
}

fn place(object: &mut PartitionObject, fields: &[Value]) {
    object.pos = Coord3D {
        x: fields[0].real(),
        y: fields[1].real(),
        z: fields[2].real(),
    };
    object.angle = fields[3].real();
}

fn letter(id: ObjectId) -> char {
    char::from(b'a' + (id.0 - 1) as u8)
}

fn index(letter: u8) -> usize {
    usize::from(letter - b'a')
}

/// Runs one search row and compares the filter calls, then the result.
fn check_search(grid: &PartitionGrid, objects: &[PartitionObject], row: &[Value], context: &str) {
    let kind = row[1].number();
    let center = match usize::try_from(row[2].number()) {
        Ok(i) => QueryCenter::Object(&objects[i]),
        Err(_) => QueryCenter::Position(Coord3D {
            x: row[3].real(),
            y: row[4].real(),
            z: row[5].real(),
        }),
    };
    let rule = match row[6].number() {
        0 => DistanceCalculationType::FromCenter2d,
        1 => DistanceCalculationType::FromCenter3d,
        2 => DistanceCalculationType::FromBoundingSphere2d,
        3 => DistanceCalculationType::FromBoundingSphere3d,
        other => panic!("rule {other}"),
    };
    let max_dist = row[7].real();
    let order = match row[8].number() {
        0 => IterOrderType::Fastest,
        1 => IterOrderType::SortedNearToFar,
        2 => IterOrderType::SortedFarToNear,
        3 => IterOrderType::SortedCheapToExpensive,
        4 => IterOrderType::SortedExpensiveToCheap,
        other => panic!("order {other}"),
    };
    let reject = row[9].number() as u32;
    let offered = row[10].text();
    let expected_ids = row[11].text();
    let values: Vec<u32> = row[12].list().iter().map(Value::bits).collect();

    let lookup = |id: ObjectId| objects.get((id.0 - 1) as usize).cloned();
    let mut calls = String::new();
    let mut filter = |id: ObjectId| {
        calls.push(letter(id));
        (reject >> (id.0 - 1)) & 1 == 0
    };
    let (found, got): (String, Vec<u32>) = match kind {
        0 => {
            let closest = grid.get_closest_object(center, max_dist, rule, &lookup, &mut filter);
            (
                closest.id.map(letter).into_iter().collect(),
                [closest.dist, closest.vec.x, closest.vec.y, closest.vec.z]
                    .map(f32::to_bits)
                    .to_vec(),
            )
        }
        1 => {
            let list =
                grid.iterate_objects_in_range(center, max_dist, rule, order, &lookup, &mut filter);
            (
                list.iter().map(|&(id, _)| letter(id)).collect(),
                list.iter().map(|&(_, n)| n.to_bits()).collect(),
            )
        }
        2 => {
            let list = grid.iterate_all_objects(&mut filter);
            (
                list.iter().map(|&id| letter(id)).collect(),
                vec![0; list.len()],
            )
        }
        other => panic!("kind {other}"),
    };
    assert_eq!(calls, offered, "{context}: filter calls");
    assert_eq!(found, expected_ids, "{context}: ids");
    assert_eq!(got, values, "{context}: values");
    assert!(offered.bytes().all(|l| index(l) < objects.len()));
}

/// Replays one world and returns the number of searches it checked.
fn check_world(world: &Value, number: usize) -> usize {
    let world = world.list();
    let g = world[0].list();
    let extent = Region3D {
        lo: Coord3D {
            x: g[1].real(),
            y: g[2].real(),
            z: 0.0,
        },
        hi: Coord3D {
            x: g[3].real(),
            y: g[4].real(),
            z: 100.0,
        },
    };
    let mut grid = PartitionGrid::new(g[0].real(), extent);
    let mut objects: Vec<PartitionObject> = world[1]
        .list()
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let row = row.list();
            let mut object = PartitionObject {
                id: ObjectId(i as i32 + 1),
                pos: Coord3D::default(),
                angle: 0.0,
                geometry: geometry(&row[..8], &format!("world {number} object {i}")),
                is_immobile: 0,
                build_cost: row[12].number() as i32,
            };
            place(&mut object, &row[8..12]);
            object
        })
        .collect();

    let mut searches = 0;
    for (step, row) in world[2].list().iter().enumerate() {
        let row = row.list();
        let code = row[0].number();
        let context = format!("world {number} step {step}");
        match code {
            0 => grid.register(ObjectId(row[1].number() as i32 + 1)),
            1 => grid.unregister(ObjectId(row[1].number() as i32 + 1)),
            2 => {
                let target = row[1].number() as usize;
                place(&mut objects[target], &row[2..6]);
                grid.make_dirty(ObjectId(target as i32 + 1));
            }
            // Always between an unregister and a register.
            4 => objects[row[1].number() as usize].geometry = geometry(&row[2..10], &context),
            7 => {
                grid.update(&|id| objects.get((id.0 - 1) as usize).cloned());
            }
            8 => {
                check_search(&grid, &objects, row, &context);
                searches += 1;
            }
            other => panic!("{context}: op {other}"),
        }
    }
    searches
}

#[test]
fn searches_match_the_reference() {
    let expected = get_i32(JSON, "vector_count").expect("vector_count") as usize;
    assert!(expected >= 1000);
    let start = JSON.find("\"worlds\": ").expect("worlds") + "\"worlds\": ".len();
    let mut at = 0;
    let worlds = Value::parse(&JSON.as_bytes()[start..], &mut at);
    let searches: usize = worlds
        .list()
        .iter()
        .enumerate()
        .map(|(number, world)| check_world(world, number))
        .sum();
    assert_eq!(searches, expected);
}
