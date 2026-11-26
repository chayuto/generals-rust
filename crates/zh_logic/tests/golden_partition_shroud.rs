//! Golden test for `spatial::partition_shroud`, the shroud half of
//! `PartitionManager`. Oracle: `tests/cpp_harness/gen_partition_shroud.cpp`,
//! which also describes the row format. Recorded without the retail features.

use zh_common::math::{Coord3D, Region3D};
use zh_common::xfer::{Snapshot, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::spatial::partition_grid::PartitionGrid;
use zh_logic::spatial::partition_shroud::{CellShroudStatus, ShroudStatusStoreRestore};

const JSON: &str = include_str!("../../../docs/golden_vectors/PARTITION_SHROUD.json");
const VECTORS: usize = 700;

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
                Value::Number(
                    std::str::from_utf8(&bytes[start..*at])
                        .unwrap()
                        .parse()
                        .unwrap(),
                )
            }
        }
    }

    fn list(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            other => panic!("not a list: {other:?}"),
        }
    }

    fn number(&self) -> i64 {
        match self {
            Value::Number(n) => *n,
            other => panic!("not a number: {other:?}"),
        }
    }

    fn text(&self) -> &str {
        match self {
            Value::Text(text) => text,
            other => panic!("not a string: {other:?}"),
        }
    }
}

/// The value of `"key":` in a vector line.
fn field(line: &str, key: &str) -> Value {
    let marker = format!("\"{key}\":");
    let mut at = line.find(&marker).expect("field") + marker.len();
    Value::parse(line.as_bytes(), &mut at)
}

fn real(value: &Value) -> f32 {
    f32::from_bits(value.number() as u32)
}

fn new_grid(g: &[Value]) -> PartitionGrid {
    let corner = |x: &Value, y: &Value, z| Coord3D {
        x: real(x),
        y: real(y),
        z,
    };
    PartitionGrid::new(
        real(&g[0]),
        Region3D {
            lo: corner(&g[1], &g[2], 0.0),
            hi: corner(&g[3], &g[4], 100.0),
        },
    )
}

fn crc_of(grid: &PartitionGrid) -> u32 {
    let mut xfer = XferCrc::new();
    grid.crc(&mut xfer);
    xfer.get_crc()
}

fn save_of(grid: &mut PartitionGrid) -> Vec<u8> {
    let mut xfer = XferSave::new();
    grid.xfer(&mut xfer).expect("save");
    xfer.into_bytes()
}

fn digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{}:{hash:016x}", bytes.len())
}

fn levels(grid: &PartitionGrid) -> String {
    let mut out = String::new();
    let mut index = 0;
    while let Some(cell) = grid.shroud_cell(index) {
        for player in 0..16 {
            let level = cell.level(player);
            if level.current_shroud != 1 || level.active_shroud_level != 0 {
                out += &format!(
                    "{index}.{player}={}/{} ",
                    level.current_shroud, level.active_shroud_level
                );
            }
        }
        index += 1;
    }
    out
}

fn vector_lines() -> impl Iterator<Item = &'static str> {
    JSON.lines().filter(|line| line.starts_with("{\"g\":"))
}

#[test]
fn golden_partition_shroud() {
    let mut vectors = 0;
    let mut rows = 0;
    for (v, line) in vector_lines().enumerate() {
        let geometry = field(line, "g");
        let mut geometry_now = geometry.list();
        let mut grid = new_grid(geometry_now);
        let players = field(line, "players").number() as usize;
        let persist = field(line, "persist").number() as u32;
        let mut frame = field(line, "frame").number() as u32;
        let mut store = ShroudStatusStoreRestore::default();
        let script = field(line, "s");
        for (r, row) in script.list().iter().enumerate() {
            let row = row.list();
            let op = row[0].number();
            let at = format!("vector {v} row {r} op {op}");
            let args = match op {
                0..=4 | 12 => 4,
                14 => 5,
                6 | 7 => 0,
                _ => 1,
            };
            let a = &row[1..=args];
            match op {
                0..=4 => {
                    let (x, y, radius) = (real(&a[0]), real(&a[1]), real(&a[2]));
                    let mask = a[3].number() as u16;
                    match op {
                        0 => grid.do_shroud_reveal(x, y, radius, mask, players),
                        1 => grid.undo_shroud_reveal(x, y, radius, mask, players),
                        2 => grid.do_shroud_cover(x, y, radius, mask, players),
                        3 => grid.undo_shroud_cover(x, y, radius, mask, players),
                        _ => grid.queue_undo_shroud_reveal(x, y, radius, mask, frame, persist),
                    }
                }
                5 => frame = a[0].number() as u32,
                6 => {
                    grid.update(&|_| None);
                    grid.process_pending_undo_shroud_reveals(frame, players);
                }
                7 => grid.process_all_pending_undo_shroud_reveals(players),
                8 => grid.reveal_map_for_player(a[0].number() as usize),
                9 => grid.reveal_map_for_player_permanently(a[0].number() as usize),
                10 => {
                    grid.undo_reveal_map_for_player_permanently(a[0].number() as usize, players);
                }
                11 => grid.shroud_map_for_player(a[0].number() as usize, players),
                12 => {
                    let pos = Coord3D {
                        x: real(&a[1]),
                        y: real(&a[2]),
                        z: 0.0,
                    };
                    let status = match grid.shroud_status_at(a[0].number() as i32, &pos) {
                        CellShroudStatus::Clear => 0,
                        CellShroudStatus::Fogged => 1,
                        CellShroudStatus::Shrouded => 2,
                    };
                    assert_eq!(status, a[3].number(), "status, {at}");
                }
                13 => grid.store_fogged_cells(&mut store, a[0].number() as u8),
                14 => {
                    geometry_now = a;
                    grid = new_grid(geometry_now);
                }
                15 => grid.restore_fogged_cells(&store, a[0].number() as u8),
                _ => panic!("unknown op, {at}"),
            }
            if row.len() > args + 3 {
                assert_eq!(levels(&grid), row[args + 3].text(), "levels, {at}");
            }
            assert_eq!(crc_of(&grid), row[args + 1].number() as u32, "crc, {at}");
            let saved = save_of(&mut grid);
            assert_eq!(digest(&saved), row[args + 2].text(), "save, {at}");

            // A load into a fresh grid gives the same save back.
            let mut loaded = new_grid(geometry_now);
            let mut load = XferLoad::new(&saved);
            loaded.xfer(&mut load).expect("load");
            assert_eq!(load.position(), saved.len(), "load length, {at}");
            assert_eq!(save_of(&mut loaded), saved, "load, {at}");
            rows += 1;
        }
        vectors += 1;
    }
    assert_eq!(vectors, VECTORS);
    assert!(rows > 10 * VECTORS, "only {rows} rows");
}

#[test]
fn load_refuses_another_map() {
    let region = |size| Region3D {
        lo: Coord3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        hi: Coord3D {
            x: size,
            y: size,
            z: 0.0,
        },
    };
    let saved = save_of(&mut PartitionGrid::new(10.0, region(40.0)));
    for mut other in [
        PartitionGrid::new(20.0, region(80.0)),
        PartitionGrid::new(10.0, region(50.0)),
    ] {
        assert_eq!(
            other.xfer(&mut XferLoad::new(&saved)),
            Err(XferError::InvalidData)
        );
    }
}
