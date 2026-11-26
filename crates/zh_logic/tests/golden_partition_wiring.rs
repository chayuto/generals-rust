//! The partition grid and collisions as wired into [`SimContext`], against
//! `tests/cpp_harness/gen_partition_wiring.cpp`, which drives the same steps
//! through the C++ engine's object creation, `Object` setters,
//! `GameLogic::destroyObject`, `PartitionManager::update` and range iterators.
//! The harness header describes the operation format.
//!
//! The objects are driven through the public `SimContext` API only. After every
//! operation the test also reads `OFF_MAP` and the grid's dirty and
//! registration order. Not covered: see the harness header.

use std::sync::{Arc, Mutex};
use zh_common::golden_json::{decode_hex_bytes, extract_vector_lines, get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, NameKeyType, Real, TeamId};
use zh_common::xfer::{Xfer, XferError};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleDataId, ObjectSlot};
use zh_logic::modules::module::{BehaviorModule, CollideFacet};
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld, PRIVATE_STATUS_OFF_MAP};

const DOC: &str = include_str!("../../../docs/golden_vectors/PARTITION_WIRING.json");

#[derive(Clone, Copy)]
struct Reaction {
    kind: u32,
    target: usize,
    geom: [u32; 4],
    to: Coord3D,
    angle: Real,
}

#[derive(Default)]
struct Shared {
    /// By creation order; `None` once deleted.
    objects: Vec<Option<ObjectSlot>>,
    reactions: Vec<Option<Reaction>>,
    /// self, other, loc and normal bits.
    events: Vec<(usize, usize, [u32; 6])>,
}

impl Shared {
    fn index_of(&self, slot: ObjectSlot) -> usize {
        self.objects
            .iter()
            .position(|&s| s == Some(slot))
            .expect("known object")
    }
}

fn geometry(g: [u32; 4]) -> GeometryInfo {
    GeometryInfo::new(
        GeometryType::from_i32(Int::try_from(g[0]).unwrap()),
        0,
        Real::from_bits(g[1]),
        Real::from_bits(g[2]),
        Real::from_bits(g[3]),
    )
}

struct Recorder(Arc<Mutex<Shared>>); // determinism-ok: test recorder

impl BehaviorModule for Recorder {
    fn module_name_key(&self) -> NameKeyType {
        NameKeyType::default()
    }
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_collide(&self) -> Option<&dyn CollideFacet> {
        Some(self)
    }
    fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
        Some(self)
    }
}

impl CollideFacet for Recorder {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    ) {
        let me = ctx.owner_slot;
        let other = other.expect("other");
        let (loc, n) = (loc.expect("loc"), normal.expect("normal"));
        let (reaction, third) = {
            let mut shared = self.0.lock().unwrap();
            let (s, o) = (shared.index_of(me), shared.index_of(other));
            let values = [loc.x, loc.y, loc.z, n.x, n.y, n.z].map(Real::to_bits);
            shared.events.push((s, o, values));
            let reaction = shared.reactions[s];
            let third = reaction.and_then(|r| shared.objects.get(r.target).copied().flatten());
            (reaction, third)
        };
        let Some(r) = reaction else {
            return;
        };
        match r.kind {
            0 => {}
            1 => sim.destroy_object(me),
            2 => sim.destroy_object(other),
            3 => sim.set_status(me, ObjectStatusType::NoCollisions),
            4 => sim.set_status(other, ObjectStatusType::NoCollisions),
            5 => sim.set_object_position(me, &r.to),
            6 => sim.set_object_position(other, &r.to),
            7 => sim.set_object_orientation(other, r.angle),
            8 => sim.set_status(other, ObjectStatusType::UnderConstruction),
            9 => sim.set_geometry_info(other, &geometry(r.geom)),
            10 => {
                if let Some(t) = third {
                    sim.destroy_object(t);
                }
            }
            11 => {
                if let Some(t) = third {
                    sim.set_object_position(t, &r.to);
                }
            }
            k => panic!("reaction {k}"),
        }
    }
}

struct Run {
    sim: SimContext,
    shared: Arc<Mutex<Shared>>, // determinism-ok: test recorder
}

fn word(w: &str) -> u32 {
    u32::from_str_radix(w, 16).unwrap_or_else(|_| panic!("hex {w}"))
}

fn real(w: &str) -> Real {
    Real::from_bits(word(w))
}

fn coord(w: &[&str]) -> Coord3D {
    Coord3D::new(real(w[0]), real(w[1]), real(w[2]))
}

fn region(lox: Real, loy: Real, hix: Real, hiy: Real) -> Region3D {
    Region3D {
        lo: Coord3D::new(lox, loy, 0.0),
        hi: Coord3D::new(hix, hiy, 100.0),
    }
}

impl Run {
    fn slot(&self, index: usize) -> ObjectSlot {
        self.shared.lock().unwrap().objects[index].expect("live object")
    }

    fn live(&self) -> Vec<Option<ObjectSlot>> {
        self.shared.lock().unwrap().objects.clone()
    }

    fn place(&self, slot: ObjectSlot) -> String {
        let thing = &self.sim.world.objects.get(slot).expect("object").thing;
        let p = thing.get_position();
        [p.x, p.y, p.z, thing.get_orientation()]
            .map(|v| format!("{:x}", v.to_bits()))
            .join(" ")
    }

    fn names(&self, slots: impl IntoIterator<Item = ObjectSlot>) -> String {
        let shared = self.shared.lock().unwrap();
        slots
            .into_iter()
            .map(|s| shared.index_of(s).to_string())
            .collect()
    }

    fn state(&self) -> String {
        let off: String = self
            .live()
            .iter()
            .map(|s| match s {
                None => '-',
                Some(s) => {
                    let status = self.sim.world.objects.get(*s).unwrap().private_status;
                    if status & PRIVATE_STATUS_OFF_MAP != 0 {
                        '1'
                    } else {
                        '0'
                    }
                }
            })
            .collect();
        let objects = &self.sim.world.objects;
        let slot = |id| objects.slot_by_id(id).expect("registered object exists");
        let dirty = self.names(self.sim.world.partition.dirty_objects().map(slot));
        let modules = self.names(self.sim.world.partition.registered_objects().map(slot));
        format!("{off} {dirty} {modules}")
    }

    /// Runs one operation and returns its result in the harness's format; for
    /// `f`, only the places (the events stay in `shared`).
    fn apply(&mut self, op: &str, a: &[&str]) -> String {
        let index = || a[0].parse::<usize>().expect("index");
        match op {
            "s" => {
                let name = format!("PW_Object{}", a[0]);
                let tid = self
                    .sim
                    .rules
                    .thing_factory
                    .find_template_id(&name)
                    .expect("template");
                let slot = self
                    .sim
                    .spawn_object(tid, TeamId(2), coord(&a[1..]), real(a[4]));
                self.sim.world.attach_module(
                    slot,
                    NameKeyType::default(),
                    ModuleDataId::default(),
                    FacetMask::COLLIDE,
                    SleepyUpdatePhase::Normal,
                    Box::new(Recorder(self.shared.clone())),
                );
                let mut shared = self.shared.lock().unwrap();
                shared.objects.push(Some(slot));
                shared.reactions.push(None);
                drop(shared);
                self.place(slot)
            }
            "m" => {
                let slot = self.slot(index());
                self.sim.set_object_position(slot, &coord(&a[1..]));
                self.place(slot)
            }
            "t" => {
                let slot = self.slot(index());
                self.sim.set_object_orientation(slot, real(a[1]));
                self.place(slot)
            }
            "g" => {
                let slot = self.slot(index());
                // type is decimal, the sizes are float bits
                let g = [a[1].parse().unwrap(), word(a[2]), word(a[3]), word(a[4])];
                self.sim.set_geometry_info(slot, &geometry(g));
                String::new()
            }
            "u" | "n" => {
                let slot = self.slot(index());
                let status = if op == "u" {
                    ObjectStatusType::UnderConstruction
                } else {
                    ObjectStatusType::NoCollisions
                };
                if a[1] == "1" {
                    self.sim.set_status(slot, status);
                } else {
                    self.sim.clear_status(slot, status);
                }
                String::new()
            }
            "d" => {
                let slot = self.slot(index());
                self.sim.destroy_object_now(slot);
                String::new()
            }
            "p" => {
                let gone: Vec<bool> = self
                    .live()
                    .iter()
                    .map(|s| {
                        s.is_some_and(|s| self.sim.world.objects.get(s).unwrap().is_destroyed())
                    })
                    .collect();
                self.sim.process_destroy_list();
                let mut shared = self.shared.lock().unwrap();
                for (object, gone) in shared.objects.iter_mut().zip(gone) {
                    if gone {
                        *object = None;
                    }
                }
                String::new()
            }
            "f" => {
                self.shared.lock().unwrap().events.clear();
                self.sim.update_partition();
                let places: Vec<String> = self
                    .live()
                    .into_iter()
                    .flatten()
                    .map(|s| self.place(s))
                    .collect();
                places.join(" ")
            }
            "b" => {
                let r = region(real(a[1]), real(a[2]), real(a[3]), real(a[4]));
                self.sim.init_partition(real(a[0]), r);
                String::new()
            }
            "r" => {
                let reaction = Reaction {
                    kind: a[1].parse().unwrap(),
                    target: a[2].parse().unwrap(),
                    // type is decimal, the sizes are float bits
                    geom: [a[3].parse().unwrap(), word(a[4]), word(a[5]), word(a[6])],
                    to: coord(&a[7..]),
                    angle: real(a[10]),
                };
                self.shared.lock().unwrap().reactions[index()] = Some(reaction);
                String::new()
            }
            "q" => {
                let found = self.sim.find_objects_in_range(&coord(a), real(a[3]));
                self.names(found)
            }
            "o" => {
                let found = self
                    .sim
                    .find_objects_in_range_of(self.slot(index()), real(a[1]));
                self.names(found)
            }
            "w" => {
                let found = self
                    .sim
                    .find_objects_in_range_bounding_sphere_3d(self.slot(index()), real(a[1]));
                self.names(found)
            }
            _ => panic!("operation {op}"),
        }
    }

    /// `Err` holds what the port did instead.
    fn step(&mut self, row: &str) -> Result<(), String> {
        let (head, tail) = row.split_once('=').expect("=");
        let (want, want_state) = tail.split_once('!').expect("!");
        let words: Vec<&str> = head.split(' ').collect();
        let (op, args) = (words[0], &words[1..]);
        let got = self.apply(op, args);
        if op == "f" {
            let (events, places) = want.split_once('/').expect("/");
            self.check_events(events)?;
            if got != places {
                return Err(format!("places {got}"));
            }
        } else if got != want {
            return Err(format!("result {got:?}"));
        }
        let state = self.state();
        if state != want_state {
            return Err(format!("state {state:?}"));
        }
        Ok(())
    }

    fn check_events(&self, want: &str) -> Result<(), String> {
        let shared = self.shared.lock().unwrap();
        let want: Vec<&str> = want.split('|').filter(|e| !e.is_empty()).collect();
        let mut same = want.len() == shared.events.len();
        for (event, &(s, o, values)) in want.iter().zip(&shared.events) {
            let f: Vec<&str> = event.split(',').collect();
            let mask: u32 = f[2].parse().unwrap();
            same &= f[0] == s.to_string() && f[1] == o.to_string();
            same &= (0..6).all(|k| mask >> k & 1 == 0 || word(f[3 + k]) == values[k]);
        }
        if same {
            return Ok(());
        }
        let got: Vec<String> = shared
            .events
            .iter()
            .map(|(s, o, v)| format!("{s},{o},{:x?}", v))
            .collect();
        Err(format!("events {got:?}"))
    }
}

#[test]
fn partition_wiring_matches_reference() {
    let ini_hex = DOC
        .lines()
        .find_map(|l| get_str(l, "ini_hex"))
        .expect("ini_hex");
    let text = decode_hex_bytes(ini_hex).expect("ini bytes");
    let mut sim = SimContext::new();
    sim.rules = RulesRegistry::new();
    let mut ini = IniReader::new("object.ini", &text, IniLoadType::Overwrite);
    if let Err(e) = sim.rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    let mut run = Run {
        sim,
        shared: Arc::default(),
    };

    let declared = |key| {
        DOC.lines()
            .find_map(|l| get_u32(l, key))
            .unwrap_or_else(|| panic!("{key}")) as usize
    };
    let lines = extract_vector_lines(DOC);
    assert_eq!(lines.len(), declared("vector_count"), "vector count");

    let mut failures = Vec::new();
    let (mut rows, mut events) = (0_usize, 0_usize);
    for (index, line) in lines.iter().enumerate() {
        run.sim.world = SimWorld::new();
        run.sim
            .init_partition(20.0, region(0.0, 0.0, 1000.0, 1000.0));
        *run.shared.lock().unwrap() = Shared::default();
        let script = get_str(line, "v").expect("script");
        for (n, row) in script.split(';').enumerate() {
            rows += 1;
            let outcome = run.step(row);
            if row.starts_with('f') {
                events += run.shared.lock().unwrap().events.len();
            }
            if let Err(got) = outcome {
                failures.push(format!(
                    "vector {index} row {n}\n  want {row}\n  got  {got}"
                ));
                break;
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} vectors differ:\n{}",
        failures.len(),
        lines.len(),
        failures
            .iter()
            .take(8)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(rows > lines.len() * 20, "rows {rows}");
    assert_eq!(events, declared("event_count"), "event count");
}
