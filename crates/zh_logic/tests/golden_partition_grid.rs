//! Golden test for `spatial::partition_grid`, the filing half of
//! `PartitionManager`. Oracle: `tests/cpp_harness/gen_partition_grid.cpp`, which
//! also describes the dump format. Recorded without the retail features.

use zh_common::golden_json::get_i32;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::ObjectId;
use zh_logic::spatial::partition_grid::{PartitionGrid, PartitionObject};

const JSON: &str = include_str!("../../../docs/golden_vectors/PARTITION_GRID.json");

/// The subset of JSON the vector lines use: integers, escape-free strings,
/// arrays and one flat object per vector.
#[derive(Debug)]
enum Value {
    Number(i64),
    Text(String),
    List(Vec<Value>),
}

impl Value {
    fn parse(bytes: &[u8], at: &mut usize) -> Value {
        match bytes[*at] {
            b'[' => {
                *at += 1;
                let mut items = Vec::new();
                while bytes[*at] != b']' {
                    items.push(Value::parse(bytes, at));
                    if bytes[*at] == b',' {
                        *at += 1;
                    }
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

    fn real(&self) -> f32 {
        f32::from_bits(self.number() as u32)
    }

    fn text(&self) -> &str {
        match self {
            Value::Text(s) => s,
            other => panic!("expected a string, got {other:?}"),
        }
    }
}

fn section(name: &str) -> impl Iterator<Item = &'static str> {
    let head = format!("\"{name}\": [\n");
    let start = JSON.find(&head).expect("section") + head.len();
    JSON[start..]
        .lines()
        .take_while(|line| !line.starts_with(']'))
        .map(|line| line.trim_end_matches(','))
}

fn fnv(s: &str) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in s.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("#{}:{hash:016x}", s.len())
}

#[track_caller]
fn check_dump(expected: &str, actual: &str, context: &str) {
    let matches = if expected.starts_with('#') {
        expected == fnv(actual)
    } else {
        expected == actual
    };
    assert!(
        matches,
        "{context}\n  expected {expected}\n  actual   {actual}\n  (hash of actual {})",
        fnv(actual)
    );
}

struct GridConfig {
    cell_size: f32,
    extent: Region3D,
}

/// Reads a grid row and checks the sizes and the radius-ring table.
fn check_grid(index: usize, row: &Value) -> GridConfig {
    let row = row.list();
    let config = GridConfig {
        cell_size: row[0].real(),
        extent: Region3D {
            lo: Coord3D {
                x: row[1].real(),
                y: row[2].real(),
                z: 0.0,
            },
            hi: Coord3D {
                x: row[3].real(),
                y: row[4].real(),
                z: 100.0,
            },
        },
    };
    let grid = PartitionGrid::new(config.cell_size, config.extent);
    assert_eq!(
        grid.cell_size().to_bits(),
        row[5].real().to_bits(),
        "grid {index} cell size"
    );
    assert_eq!(
        (
            i64::from(grid.cell_count_x()),
            i64::from(grid.cell_count_y())
        ),
        (row[6].number(), row[7].number()),
        "grid {index} cell counts"
    );
    assert_eq!(
        i64::from(grid.max_gco_radius()),
        row[8].number(),
        "grid {index} max radius"
    );
    let rings = row[9].list();
    assert_eq!(rings.len() as i64, row[8].number() + 1);
    for (radius, expected) in rings.iter().enumerate() {
        let actual: String = grid
            .radius_offsets(radius as i32)
            .iter()
            .map(|(x, y)| format!("{x}.{y} "))
            .collect();
        check_dump(
            expected.text(),
            &actual,
            &format!("grid {index} ring {radius}"),
        );
    }
    config
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
        (
            info.get_major_radius().to_bits(),
            info.get_minor_radius().to_bits()
        ),
        (fields[5].real().to_bits(), fields[6].real().to_bits()),
        "{context} geometry radii"
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

fn ids(list: impl Iterator<Item = ObjectId>) -> String {
    list.map(|id| id.0.to_string()).collect()
}

fn cell_dump(grid: &PartitionGrid) -> String {
    let total = (grid.cell_count_x() * grid.cell_count_y()) as usize;
    let mut out = String::new();
    for index in 0..total {
        if grid.cell_object_count(index) == 0 {
            continue;
        }
        let members = ids(grid.cell_objects(index));
        assert_eq!(members.len(), grid.cell_object_count(index));
        out.push_str(&format!("{index}:{members},"));
    }
    out
}

fn object_dump(grid: &PartitionGrid, count: usize) -> String {
    let mut out = String::new();
    for id in (1..=count as i32).map(ObjectId) {
        if !grid.is_registered(id) {
            continue;
        }
        let center = grid
            .object_center_cell(id)
            .map_or("-".to_string(), |c| c.to_string());
        let cells: Vec<String> = grid.object_cells(id).iter().map(usize::to_string).collect();
        let capacity = grid.object_cell_capacity(id).expect("filed object");
        out.push_str(&format!(
            "{}@{center}/{capacity}:{},",
            id.0,
            cells.join(".")
        ));
    }
    out
}

/// Returns the number of updates checked.
fn run_vector(index: usize, line: &str, grids: &[GridConfig]) -> usize {
    let grid_index = get_i32(line, "g").expect("grid index") as usize;
    let config = &grids[grid_index];
    let mut grid = PartitionGrid::new(config.cell_size, config.extent);

    let objects_at = line.find("\"o\":").unwrap() + 4;
    let script_at = line.find("\"s\":").unwrap() + 4;
    let mut at = 0;
    let objects = Value::parse(&line.as_bytes()[objects_at..], &mut at);
    let mut at = 0;
    let script = Value::parse(&line.as_bytes()[script_at..], &mut at);

    let mut objects: Vec<PartitionObject> = objects
        .list()
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let row = row.list();
            let mut object = PartitionObject {
                id: ObjectId(i as i32 + 1),
                pos: Coord3D::default(),
                angle: 0.0,
                geometry: geometry(&row[..7], &format!("vector {index} object {}", i + 1)),
                is_immobile: 0,
            };
            place(&mut object, &row[7..]);
            object
        })
        .collect();

    let mut updates = 0;
    for (step, row) in script.list().iter().enumerate() {
        let row = row.list();
        let code = row[0].number();
        let context = format!("vector {index} (grid {grid_index}) step {step} op {code}");
        let target = if code == 7 {
            0
        } else {
            row[1].number() as usize - 1
        };
        let id = ObjectId(target as i32 + 1);
        match code {
            0 => grid.register(id),
            1 => grid.unregister(id),
            2 | 3 => {
                place(&mut objects[target], &row[2..6]);
                grid.make_dirty(id);
            }
            4 => {
                objects[target].geometry = geometry(&row[2..9], &context);
                if grid.is_registered(id) {
                    grid.unregister(id);
                    grid.register(id);
                }
            }
            5 => grid.make_dirty(id),
            6 => grid.make_dirty_for_collision_check(id),
            7 => {
                let lookup = |id: ObjectId| objects.get(id.0 as usize - 1).cloned();
                let changed = ids(grid.update(&lookup).into_iter());
                assert!(grid.updated_since_last_reset());
                check_dump(
                    row[1].text(),
                    &cell_dump(&grid),
                    &format!("{context}: cells"),
                );
                check_dump(
                    row[2].text(),
                    &object_dump(&grid, objects.len()),
                    &format!("{context}: objects"),
                );
                assert_eq!(row[3].text(), changed, "{context}: center cell changes");
                updates += 1;
            }
            other => panic!("op {other}"),
        }
        let lists = &row[row.len() - 2..];
        assert_eq!(
            lists[0].text(),
            ids(grid.dirty_objects()),
            "{context}: dirty list"
        );
        assert_eq!(
            lists[1].text(),
            ids(grid.registered_objects()),
            "{context}: module list"
        );
    }
    updates
}

#[test]
fn golden_partition_grid() {
    let grids: Vec<GridConfig> = section("grids")
        .enumerate()
        .map(|(i, line)| check_grid(i, &Value::parse(line.as_bytes(), &mut 0)))
        .collect();
    assert_eq!(grids.len(), 90);

    let mut vectors = 0;
    let mut updates = 0;
    for (i, line) in section("vectors").enumerate() {
        updates += run_vector(i, line, &grids);
        vectors += 1;
    }
    assert_eq!(vectors, 1200);
    assert!(updates > 3000, "only {updates} updates");
}

#[test]
fn registering_twice_and_unknown_ids_do_nothing() {
    let extent = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D {
            x: 100.0,
            y: 100.0,
            z: 0.0,
        },
    };
    let mut grid = PartitionGrid::new(40.0, extent);
    grid.register(ObjectId(1));
    grid.register(ObjectId(2));
    grid.register(ObjectId(1));
    grid.make_dirty(ObjectId(3));
    grid.unregister(ObjectId(3));
    assert_eq!(ids(grid.registered_objects()), "21");
    assert_eq!(ids(grid.dirty_objects()), "21");
    // An object the caller does not know leaves the dirty list without filing.
    assert!(grid.update(&|_| None).is_empty());
    assert_eq!(ids(grid.dirty_objects()), "");
    assert!(grid.object_cells(ObjectId(1)).is_empty());
}
