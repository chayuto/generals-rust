//! Golden test for `spatial::partition_collide`: the shape tests and
//! `processContactList`. Oracle: `tests/cpp_harness/gen_partition_collide.cpp`,
//! which also describes the vector format.

use zh_common::golden_json::get_i32;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::ObjectId;
use zh_logic::spatial::partition_collide::{
    collides_with, geom_collides_with_geom, is_colliding, process_contacts, CollideInfo,
    CollisionHost,
};
use zh_logic::spatial::partition_grid::{PartitionGrid, PartitionObject};

const JSON: &str = include_str!("../../../docs/golden_vectors/PARTITION_COLLIDE.json");

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
    fn num(&self) -> i64 {
        match self {
            Value::Number(n) => *n,
            other => panic!("expected a number, got {other:?}"),
        }
    }
    fn index(&self) -> usize {
        usize::try_from(self.num()).unwrap()
    }
    fn bits(&self) -> u32 {
        u32::try_from(self.num()).unwrap()
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

fn parse(line: &str) -> Value {
    Value::parse(line.as_bytes(), &mut 0)
}

/// The lines of the array that starts at `"name": [`.
fn block(name: &str) -> Vec<&'static str> {
    let opening = format!("\"{name}\": [");
    let mut lines = JSON.lines().skip_while(|line| *line != opening);
    assert!(lines.next().is_some(), "no {name} block");
    lines
        .take_while(|line| !line.starts_with(']'))
        .map(|line| line.trim_end_matches(','))
        .collect()
}

fn geometry(v: &[Value]) -> GeometryInfo {
    let geom_type = GeometryType::from_i32(i32::try_from(v[0].num()).unwrap());
    GeometryInfo::new(geom_type, 0, v[1].real(), v[2].real(), v[3].real())
}

fn coord(v: &[Value]) -> Coord3D {
    Coord3D::new(v[0].real(), v[1].real(), v[2].real())
}

/// Compares the values the C++ wrote; the ones it left uninitialised are zero here.
fn assert_values(loc: &Coord3D, normal: &Coord3D, mask: u32, expected: &[Value], what: &str) {
    let actual = [loc.x, loc.y, loc.z, normal.x, normal.y, normal.z];
    for (k, value) in actual.iter().enumerate() {
        if mask >> k & 1 == 1 {
            assert_eq!(value.to_bits(), expected[k].bits(), "{what} value {k}");
        } else {
            assert_eq!(value.to_bits() & 0x7fff_ffff, 0, "{what} unwritten {k}");
        }
    }
}

#[test]
fn shape_pairs() {
    let rows = block("pairs");
    assert_eq!(rows.len() as i32, get_i32(JSON, "pair_count").unwrap());
    let mut hits = [0usize; 9];
    let mut misses = [0usize; 9];
    for (n, line) in rows.iter().enumerate() {
        let row = parse(line);
        let v = row.list();
        let info = |at: usize| CollideInfo {
            geometry: geometry(&v[at..at + 4]),
            pos: coord(&v[at + 4..at + 7]),
            angle: v[at + 7].real(),
        };
        let (a, b) = (info(0), info(8));
        let combo = v[0].index() * 3 + v[8].index();
        let result = collides_with(&a, &b);
        assert_eq!(result.is_some(), v[16].num() == 1, "pair {n} combo {combo}");
        assert_eq!(
            geom_collides_with_geom(&a, &b),
            v[17].num() == 1,
            "pair {n} combo {combo} geom"
        );
        match result {
            Some(c) => {
                hits[combo] += 1;
                let what = format!("pair {n} combo {combo}");
                assert_values(&c.loc, &c.normal, v[18].bits(), &v[19..25], &what);
            }
            None => misses[combo] += 1,
        }
    }
    assert!(hits.iter().chain(&misses).all(|&count| count > 100));
}

#[derive(Debug, Clone)]
struct Object {
    variant: i64,
    geometry: GeometryInfo,
    pos: Coord3D,
    angle: f32,
    destroyed: bool,
    no_collisions: bool,
}

#[derive(Debug, Clone, Copy)]
struct Reaction {
    kind: i64,
    target: usize,
    pos: Coord3D,
    angle: f32,
}

#[derive(Debug)]
struct Event {
    id: usize,
    other: usize,
    loc: Coord3D,
    normal: Coord3D,
}

struct Host {
    grid: PartitionGrid,
    objects: Vec<Object>,
    reactions: Vec<Reaction>,
    events: Vec<Event>,
}

fn id_of(slot: usize) -> ObjectId {
    ObjectId(slot as i32 + 1)
}

fn slot_of(id: ObjectId) -> usize {
    (id.0 - 1) as usize
}

fn partition_object(objects: &[Object], id: ObjectId) -> Option<PartitionObject> {
    objects.get(slot_of(id)).map(|o| PartitionObject {
        id,
        pos: o.pos,
        angle: o.angle,
        geometry: o.geometry.clone(),
        is_immobile: u8::from(o.variant & 1 != 0),
        build_cost: 0,
    })
}

impl Host {
    fn update(&mut self) {
        let objects = &self.objects;
        self.grid.update(&|id| partition_object(objects, id));
        process_contacts(self);
        assert_eq!(self.grid.pending_contacts().count(), 0);
    }

    fn move_object(&mut self, slot: usize, pos: Coord3D, angle: f32) {
        self.objects[slot].pos = pos;
        self.objects[slot].angle = angle;
        self.grid.make_dirty(id_of(slot));
    }

    fn react(&mut self, slot: usize, other: usize) {
        let r = self.reactions[slot];
        match r.kind {
            0 => {}
            1 => self.objects[slot].destroyed = true,
            2 => self.objects[other].destroyed = true,
            3 => self.objects[slot].no_collisions = true,
            4 => self.objects[other].no_collisions = true,
            5 => self.grid.unregister(id_of(slot)),
            6 => self.grid.unregister(id_of(other)),
            7 => self.move_object(slot, r.pos, r.angle),
            8 => self.move_object(other, r.pos, r.angle),
            9 => {
                self.grid.unregister(id_of(other));
                self.grid.register(id_of(other));
            }
            10 => self.grid.unregister(id_of(r.target)),
            11 => self.move_object(r.target, r.pos, r.angle),
            12 => self.grid.make_dirty(id_of(other)),
            13 => self.grid.register(id_of(r.target)),
            kind => panic!("reaction {kind}"),
        }
    }

    fn digits(ids: impl Iterator<Item = ObjectId>) -> String {
        ids.map(|id| slot_of(id).to_string()).collect()
    }
}

impl CollisionHost for Host {
    fn grid_mut(&mut self) -> &mut PartitionGrid {
        &mut self.grid
    }
    fn partition_object(&self, id: ObjectId) -> Option<PartitionObject> {
        partition_object(&self.objects, id)
    }
    fn is_no_collide(&self, id: ObjectId) -> bool {
        self.objects[slot_of(id)].variant & 2 != 0
    }
    fn has_no_collisions_status(&self, id: ObjectId) -> bool {
        self.objects[slot_of(id)].no_collisions
    }
    fn is_destroyed(&self, id: ObjectId) -> bool {
        self.objects[slot_of(id)].destroyed
    }
    fn on_collide(&mut self, id: ObjectId, other: ObjectId, loc: &Coord3D, normal: &Coord3D) {
        let (slot, other) = (slot_of(id), slot_of(other));
        // `Object::onCollide` tests this before its collide module.
        if self.objects[slot].no_collisions {
            return;
        }
        self.events.push(Event {
            id: slot,
            other,
            loc: *loc,
            normal: *normal,
        });
        self.react(slot, other);
    }
}

fn reaction(v: &[Value]) -> Reaction {
    Reaction {
        kind: v[0].num(),
        target: v[1].index(),
        pos: coord(&v[2..5]),
        angle: v[5].real(),
    }
}

fn grids() -> Vec<(f32, Region3D)> {
    block("grids")
        .iter()
        .map(|line| {
            let row = parse(line);
            let v = row.list();
            let extent = Region3D {
                lo: Coord3D::new(v[1].real(), v[2].real(), 0.0),
                hi: Coord3D::new(v[3].real(), v[4].real(), 100.0),
            };
            (v[0].real(), extent)
        })
        .collect()
}

/// The text between `"key":` and the next key, which holds one array.
fn member<'a>(line: &'a str, key: &str) -> &'a str {
    let start = line.find(&format!("\"{key}\":")).unwrap() + key.len() + 3;
    &line[start..]
}

#[test]
fn contact_processing_scripts() {
    let grids = grids();
    let rows = block("vectors");
    assert_eq!(rows.len() as i32, get_i32(JSON, "vector_count").unwrap());
    let mut events_seen = 0usize;
    for (n, line) in rows.iter().enumerate() {
        let (cell_size, extent) = grids[parse(member(line, "g")).index()];
        let objects = parse(member(line, "o"))
            .list()
            .iter()
            .map(|o| {
                let v = o.list();
                Object {
                    variant: v[0].num(),
                    geometry: geometry(&v[1..5]),
                    pos: coord(&v[5..8]),
                    angle: v[8].real(),
                    destroyed: false,
                    no_collisions: false,
                }
            })
            .collect();
        let reactions = parse(member(line, "r"))
            .list()
            .iter()
            .map(|r| reaction(r.list()))
            .collect();
        let mut host = Host {
            grid: PartitionGrid::new(cell_size, extent),
            objects,
            reactions,
            events: Vec::new(),
        };

        for (k, step) in parse(member(line, "s")).list().iter().enumerate() {
            let v = step.list();
            let what = format!("vector {n} step {k} op {}", v[0].num());
            match v[0].num() {
                0 => host.grid.register(id_of(v[1].index())),
                1 => host.grid.unregister(id_of(v[1].index())),
                2 => host.move_object(v[1].index(), coord(&v[2..5]), v[5].real()),
                3 => host.grid.make_dirty(id_of(v[1].index())),
                4 => host
                    .grid
                    .make_dirty_for_collision_check(id_of(v[1].index())),
                5 => host.objects[v[1].index()].no_collisions = v[2].num() == 1,
                6 => host.objects[v[1].index()].destroyed = v[2].num() == 1,
                7 => {
                    host.events.clear();
                    host.update();
                    let expected = v[1].list();
                    let pairs: Vec<(usize, usize)> =
                        host.events.iter().map(|e| (e.id, e.other)).collect();
                    let expected_pairs: Vec<(usize, usize)> = expected
                        .iter()
                        .map(|e| (e.list()[0].index(), e.list()[1].index()))
                        .collect();
                    assert_eq!(pairs, expected_pairs, "{what}");
                    for (event, e) in host.events.iter().zip(expected) {
                        let e = e.list();
                        assert_values(&event.loc, &event.normal, e[2].bits(), &e[3..9], &what);
                    }
                    events_seen += expected.len();
                }
                8 => host.reactions[v[1].index()] = reaction(&v[2..8]),
                9 => {
                    let hit = is_colliding(&mut host, id_of(v[1].index()), id_of(v[2].index()));
                    assert_eq!(hit, v[3].num() == 1, "{what}");
                }
                op => panic!("{what}: unknown {op}"),
            }
            assert_eq!(
                Host::digits(host.grid.dirty_objects()),
                v[v.len() - 2].text(),
                "{what} dirty list"
            );
            assert_eq!(
                Host::digits(host.grid.registered_objects()),
                v[v.len() - 1].text(),
                "{what} module list"
            );
        }
    }
    assert_eq!(events_seen as i32, get_i32(JSON, "event_count").unwrap());
}

/// Three coincident spheres: every pair is a contact.
fn stacked_host() -> Host {
    let object = Object {
        variant: 0,
        geometry: GeometryInfo::new(GeometryType::Sphere, 0, 2.0, 2.0, 2.0),
        pos: Coord3D::new(15.0, 15.0, 0.0),
        angle: 0.0,
        destroyed: false,
        no_collisions: false,
    };
    let idle = Reaction {
        kind: 0,
        target: 0,
        pos: Coord3D::default(),
        angle: 0.0,
    };
    let extent = Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(100.0, 100.0, 100.0),
    };
    let mut host = Host {
        grid: PartitionGrid::new(10.0, extent),
        objects: vec![object; 3],
        reactions: vec![idle; 3],
        events: Vec::new(),
    };
    for slot in 0..3 {
        host.grid.register(id_of(slot));
    }
    host
}

fn gather(host: &mut Host) -> usize {
    let objects = &host.objects;
    host.grid.update(&|id| partition_object(objects, id));
    host.grid.pending_contacts().count()
}

// The C++ has no gap between gathering and processing, so these two follow its
// contact list code rather than an oracle row.

#[test]
fn unregistering_before_processing_cancels_contacts() {
    let mut host = stacked_host();
    assert_eq!(gather(&mut host), 3);
    host.grid.unregister(id_of(1));
    host.grid.register(id_of(1));
    assert_eq!(host.grid.pending_contacts().count(), 1);
    process_contacts(&mut host);
    assert!(host.events.iter().all(|e| e.id != 1 && e.other != 1));
    assert_eq!(host.events.len(), 2);
}

#[test]
fn next_update_discards_unprocessed_contacts() {
    let mut host = stacked_host();
    assert_eq!(gather(&mut host), 3);
    assert_eq!(gather(&mut host), 0);
}
