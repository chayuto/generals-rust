// SPDX-License-Identifier: GPL-3.0-or-later
//! `PKT-143` locomotor movement (AI plan slice S7) against the reference
//! (`tests/cpp_harness/gen_s7_locomotor_movement.cpp`). The calls to unported code are
//! replayed from the tape the harness recorded: another order or argument fails.

use std::collections::{BTreeMap, VecDeque};

use zh_common::golden_json::get_str;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, Real, TeamId};
use zh_common::xfer::{Snapshot, XferCrc, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::{Locomotor, LocomotorSeams, LocomotorTemplate, LocomotorWorld};
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::update::physics_update::{self as pb, physics_mut, IS_STUNNED};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::LocomotorSurfaceTypeMask;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimWorld};

fn float(tok: &str) -> Real {
    Real::from_bits(u32::from_str_radix(tok, 16).expect("float bits"))
}

/// Bits, with NaN canonical.
fn hx(v: Real) -> String {
    let bits = if v.is_nan() { 0x7fc0_0000 } else { v.to_bits() };
    format!("{bits:08x}")
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn strings(line: &str, start: &str, end: &str) -> Vec<String> {
    let from = line.find(start).expect(start) + start.len();
    let to = from + line[from..].find(end).expect(end);
    line[from..to]
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// The harness's synthetic terrain.
#[derive(Clone, Copy)]
struct Terrain {
    base: Real,
    slope: Real,
    step: Real,
    water: bool,
    water_z: Real,
}

impl ThingTerrainAccess for Terrain {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        let mut h = self.base;
        if x > 500.0 {
            h += (x - 500.0) * self.slope;
        }
        if y > 506.0 {
            h += self.step;
        }
        h
    }
    fn align_on_terrain(
        &self,
        _angle: Real,
        _pos: &Coord3D,
        _stick: Bool,
        _mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        unreachable!("no object sticks to the slope")
    }
    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(w) = water_z {
            *w = self.water_z;
        }
        if let Some(t) = terrain_z {
            *t = self.get_ground_height(x, y, None);
        }
        Bool::from(self.water && x < 504.0)
    }
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        let h = self.get_ground_height(x, y, normal);
        if layer == PathfindLayerEnum::Ground {
            h
        } else {
            h + 7.0
        }
    }
}

/// Answers the seams from the recorded calls.
#[derive(Default)]
struct Tape {
    calls: VecDeque<String>,
    errors: Vec<String>,
}

impl Tape {
    fn answer(&mut self, call: String) -> String {
        let Some(want) = self.calls.pop_front() else {
            self.errors.push(format!("unexpected call {call}"));
            return "0".to_owned();
        };
        let (args, result) = want.split_once('=').expect("tape entry");
        if args != call {
            self.errors.push(format!("called {call}, recorded {args}"));
        }
        result.to_owned()
    }
}

impl LocomotorSeams for Tape {
    fn valid_movement_terrain(
        &mut self,
        layer: PathfindLayerEnum,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> Bool {
        let call = format!(
            "V,{},{},{},{},{}",
            layer as i32,
            surfaces,
            hx(pos.x),
            hx(pos.y),
            hx(pos.z)
        );
        Bool::from(self.answer(call) == "1")
    }
    fn ground_or_structure_height(&mut self, x: Real, y: Real) -> Real {
        let result = self.answer(format!("G,{},{}", hx(x), hx(y)));
        if result == "0" {
            0.0
        } else {
            float(&result)
        }
    }
}

struct Run {
    sim: SimContext,
    slot: ObjectSlot,
    module: ModuleSlot,
    loco: Locomotor,
    tape: Tape,
}

impl Run {
    /// The save bytes of the physics module, with NaN members made canonical, and its CRC.
    fn physics_state(&mut self) -> (String, u32) {
        let (mut payload, _, generation) = self
            .sim
            .world
            .modules
            .take_payload(self.module)
            .expect("payload");
        let mut xs = XferSave::new();
        payload.xfer(&mut xs, &self.sim).expect("save");
        let mut xc = XferCrc::new();
        payload.crc(&mut xc);
        self.sim
            .world
            .modules
            .restore_payload(self.module, generation, payload);
        let mut bytes = xs.as_bytes().to_vec();
        let start = bytes.len() - 88;
        for off in [0, 4, 8, 12, 16, 20, 24, 28, 32, 36, 40, 44, 60, 76, 80, 84] {
            let at = start + off;
            let v = f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
            if v.is_nan() {
                bytes[at..at + 4].copy_from_slice(&0x7fc0_0000_u32.to_le_bytes());
            }
        }
        (hex(&bytes), xc.get_crc())
    }

    /// The fields of a row, apart from the return value and the tape.
    fn state(&mut self) -> [String; 7] {
        let (phys_save, phys_crc) = self.physics_state();
        let mut xs = XferSave::new();
        self.loco.xfer(&mut xs).expect("save");
        let mut xc = XferCrc::new();
        self.loco.crc(&mut xc);
        let obj = self.sim.world.objects.get(self.slot).expect("object");
        let mut transform: String = obj
            .thing
            .get_transform_matrix()
            .rows
            .iter()
            .flatten()
            .map(|&v| hx(v) + ",")
            .collect();
        transform.push_str(&hx(obj.thing.get_orientation()));
        let braking = obj.status.test(ObjectStatusType::Braking as Int) != 0;
        let water = obj
            .model_condition
            .test(ModelConditionFlagType::OverWater.as_i32())
            != 0;
        let bits = format!("{}{}", u8::from(braking), u8::from(water));
        [
            transform,
            hex(xs.as_bytes()),
            xc.get_crc().to_string(),
            phys_save,
            phys_crc.to_string(),
            bits,
            self.sim
                .world
                .rng
                .get_game_logic_random_seed_crc()
                .to_string(),
        ]
    }

    /// Applies one recorded operation and returns what the harness returned for it.
    fn apply(&mut self, op: &str) -> i64 {
        let tok: Vec<&str> = op.split(' ').collect();
        let f = |i: usize| float(tok[i]);
        let n = |i: usize| tok[i].parse::<i64>().expect("number");
        let c = |i: usize| Coord3D::new(float(tok[i]), float(tok[i + 1]), float(tok[i + 2]));
        let slot = self.slot;
        let mut world = LocomotorWorld {
            sim: &mut self.sim,
            owner: slot,
            seams: &mut self.tape,
        };
        let loco = &mut self.loco;
        match tok[0] {
            "T" => {
                let mut blocked = n(6) as Bool;
                loco.move_towards_position(&mut world, &c(1), f(4), f(5), &mut blocked);
                return i64::from(blocked);
            }
            "A" => loco.move_towards_angle(&mut world, f(1)),
            "M" => return i64::from(loco.maintain_current_position(&mut world)),
            "R" => {
                let mut rel = 0.0;
                let turn = loco.rotate_towards_position(&mut world, &c(1), Some(&mut rel));
                let rel_bits = hx(rel);
                let rel_bits = i64::from_str_radix(&rel_bits, 16).expect("bits");
                return i64::from(turn) * 10_000_000_000 + rel_bits;
            }
            "Z" => return i64::from(loco.handle_behavior_z(&mut world, &c(1))),
            "P" => return i64::from(pb::update(&mut self.sim, slot).0),
            "O" => loco.set_physics_options(&mut self.sim, slot),
            "S" => loco.start_move(self.sim.world.frame),
            "F" => {
                let on = n(2) as Bool;
                match n(1) {
                    0 => loco.set_ultra_accurate(on),
                    1 => loco.set_use_precise_z_pos(on),
                    2 => loco.set_no_slow_down(on),
                    3 => loco.set_allow_invalid_position(on),
                    _ => loco.set_close_enough_dist3d(on),
                }
            }
            "V" => {
                let v = f(2);
                match n(1) {
                    0 => loco.set_max_speed(v),
                    1 => loco.set_max_acceleration(v),
                    2 => loco.set_max_braking(v),
                    3 => loco.set_max_turn_rate(v),
                    4 => loco.set_max_lift(v),
                    5 => loco.set_preferred_height(v),
                    _ => loco.set_close_enough_dist(v),
                }
            }
            "W" => {
                let p = physics_mut(&mut self.sim, slot).expect("physics");
                p.vel = c(1);
                p.vel_mag.set(-1.0);
            }
            "Q" => {
                self.sim.set_object_position(slot, &c(1));
                self.sim.set_object_orientation(slot, f(4));
            }
            "D" => {
                let state = BodyDamageType::from_i32(n(1) as Int).expect("damage state");
                self.sim.set_body_damage_state(slot, state);
            }
            "K" => {
                let on = n(2) != 0;
                match n(1) {
                    0 => physics_mut(&mut self.sim, slot)
                        .expect("physics")
                        .set_flag(IS_STUNNED, Bool::from(on)),
                    1 => {
                        if on {
                            self.sim.set_disabled(slot, DisabledType::Held);
                        } else {
                            self.sim.clear_disabled(slot, DisabledType::Held);
                        }
                    }
                    which @ (2 | 3) => {
                        let status = if which == 2 {
                            ObjectStatusType::Braking
                        } else {
                            ObjectStatusType::DeckHeightOffset
                        };
                        if on {
                            self.sim.set_status(slot, status);
                        } else {
                            self.sim.clear_status(slot, status);
                        }
                    }
                    _ => {
                        let layer = if on {
                            PathfindLayerEnum::from_i32(PathfindLayerEnum::Ground as i32 + 1)
                                .expect("layer")
                        } else {
                            PathfindLayerEnum::Ground
                        };
                        self.sim.set_object_layer(slot, layer);
                    }
                }
            }
            "N" => self.sim.world.frame = self.sim.world.frame.wrapping_add(n(1) as u32),
            other => panic!("unknown operation {other}"),
        }
        0
    }
}

/// What parsing and `validate` leave in the template, as the harness prints it.
fn parsed_fields(t: &LocomotorTemplate) -> String {
    let reals = [
        t.max_speed,
        t.max_speed_damaged,
        t.min_speed,
        t.max_turn_rate,
        t.max_turn_rate_damaged,
        t.acceleration,
        t.acceleration_damaged,
        t.lift,
        t.lift_damaged,
        t.braking,
        t.min_turn_speed,
        t.preferred_height,
        t.preferred_height_damping,
        t.circling_radius,
        t.speed_limit_z,
        t.extra_2d_friction,
        t.max_thrust_angle,
        t.accel_pitch_limit,
        t.decel_pitch_limit,
        t.turn_pivot_offset,
        t.close_enough_dist,
        t.ultra_accurate_slide_factor,
        t.wander_width_factor,
        t.wander_length_factor,
    ];
    let mut s: String = reals.iter().map(|&v| hx(v) + " ").collect();
    let flags: String = [
        t.apply_2d_friction_airborne,
        t.stick_to_ground,
        t.allow_motive_force_airborne,
        t.downhill_only,
        t.can_move_backward,
        t.is_close_enough_dist3d,
    ]
    .iter()
    .map(|&f| if f != 0 { '1' } else { '0' })
    .collect();
    s.push_str(&format!(
        "{} {} {} {flags}",
        t.surfaces,
        t.behavior_z.as_i32(),
        t.appearance.as_i32()
    ));
    s
}

const FIELDS: [&str; 9] = [
    "return",
    "transform",
    "locomotor save",
    "locomotor crc",
    "physics save",
    "physics crc",
    "status bits",
    "random seed",
    "tape",
];

#[test]
fn locomotor_movement_matches_reference() {
    let json = include_str!("../../../docs/golden_vectors/S7_LocomotorMovement.json");
    let units = get_str(json, "units").expect("units").replace("\\n", "\n");
    let template_block = &json[json.find("\"templates\":[").expect("templates")
        ..json.find("\"vector_count\"").expect("count")];
    let templates: Vec<LocomotorTemplate> = template_block
        .lines()
        .filter(|l| l.starts_with("\"Locomotor"))
        .map(|l| {
            let text = l
                .trim_end_matches(',')
                .trim_matches('"')
                .replace("\\n", "\n");
            let text = text.trim_end_matches(['"', ']']);
            let mut t = LocomotorTemplate::new();
            t.parse_ini_block(text).expect("locomotor template");
            t
        })
        .collect();
    assert_eq!(templates.len(), 387);
    let parsed: Vec<&str> = json
        .lines()
        .filter_map(|l| l.trim_end_matches(',').strip_prefix("\"p "))
        .map(|l| l.trim_end_matches([']', ',']).trim_end_matches('"'))
        .collect();
    assert_eq!(parsed.len(), templates.len());
    for (t, want) in templates.iter().zip(&parsed) {
        assert_eq!(&parsed_fields(t), want, "template {}", t.name);
    }
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"v\":"))
        .collect();
    assert_eq!(lines.len(), 1200);

    let mut rules = RulesRegistry::new();
    let mut ini = IniReader::new("object.ini", units.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut ini)
        .expect("units");
    let mut sim = SimContext::new();
    sim.rules = rules;

    let mut failures: BTreeMap<String, (usize, String)> = BTreeMap::new();
    let mut rows_checked = 0;
    // The first 54 vectors drive to one goal point; count those that get there.
    let mut arrivals = 0;
    for (v, line) in lines.iter().enumerate() {
        let setup: Vec<&str> = get_str(line, "setup").expect("setup").split(' ').collect();
        let num = |i: usize| setup[i].parse::<u32>().expect("number"); // determinism-ok: test input
        let ops = strings(line, "\"ops\":[", "],\"rows\"");
        let rows = strings(line, "\"rows\":[", "]}");
        assert_eq!(rows.len(), ops.len(), "vector {v}");

        let terrain = Terrain {
            base: float(setup[4]),
            slope: float(setup[5]),
            step: float(setup[6]),
            water: setup[7] != "0",
            water_z: float(setup[8]),
        };
        sim.world = SimWorld::new();
        let frame = num(12);
        sim.world.frame = frame;
        sim.terrain = Box::new(terrain);
        sim.rules.global_data.gravity = -1.0;
        sim.rules.global_data.movement_penalty_damage_state =
            BodyDamageType::from_i32(num(3) as Int).expect("penalty state");
        let tid = sim
            .rules
            .thing_factory
            .find_template_id(&format!("S7_U{}", setup[1]))
            .expect("unit template");
        let pos = Coord3D::new(float(setup[13]), float(setup[14]), float(setup[15]));
        let slot = sim.spawn_object(tid, TeamId(1), pos, float(setup[16]));
        let module = sim
            .world
            .objects
            .get(slot)
            .and_then(|o| o.physics.as_ref())
            .expect("physics")
            .mod_slot;
        sim.rules.global_data.gravity = float(setup[2]);
        sim.world.rng.init_random_with_seed(num(11));
        let loco = Locomotor::new(&templates[num(0) as usize], &mut sim.world.rng, frame);
        let mut run = Run {
            sim,
            slot,
            module,
            loco,
            tape: Tape::default(),
        };

        let mut previous = get_str(line, "spawn").expect("spawn").to_owned();
        for k in 0..=rows.len() {
            let (ret, want) = if k == 0 {
                (0, previous.clone())
            } else {
                let want = if rows[k - 1] == "=" {
                    previous.clone()
                } else {
                    rows[k - 1].clone()
                };
                let tape = want.rsplit('|').next().expect("tape");
                run.tape.calls = tape
                    .split(';')
                    .filter(|e| !e.is_empty())
                    .map(str::to_owned)
                    .collect();
                (run.apply(&ops[k - 1]), want)
            };
            let want_fields: Vec<&str> = want.split('|').collect();
            assert_eq!(want_fields.len(), FIELDS.len(), "vector {v} row {k}");
            let state = run.state();
            let tape_error = if let Some(e) = run.tape.errors.first() {
                Some(e.clone())
            } else {
                run.tape
                    .calls
                    .front()
                    .map(|c| format!("recorded call not made: {c}"))
            };
            let mut got: Vec<String> = vec![ret.to_string()];
            got.extend(state);
            got.push(match &tape_error {
                Some(e) => e.clone(),
                None => want_fields[8].to_owned(),
            });
            rows_checked += 1;
            let wrong: Vec<usize> = (0..FIELDS.len())
                .filter(|&i| got[i] != want_fields[i])
                .collect();
            if !wrong.is_empty() {
                let op = if k == 0 { "spawn" } else { &ops[k - 1] };
                let t = &run.loco.template;
                for &i in &wrong {
                    let entry = failures.entry(FIELDS[i].to_owned()).or_insert_with(|| {
                        (
                            0,
                            format!(
                                "vector {v} row {k} op {op:?} {:?} {:?}\n  got  {}\n  want {}",
                                t.appearance, t.behavior_z, got[i], want_fields[i]
                            ),
                        )
                    });
                    entry.0 += 1;
                }
                // Later rows of this vector start from a wrong state.
                break;
            }
            previous = want;
        }
        if v < 54 && ops.iter().any(|op| op == "M") {
            arrivals += 1;
        }
        sim = run.sim;
    }
    assert!(
        failures.is_empty(),
        "first failure of each field (count of vectors):\n{}",
        failures
            .iter()
            .map(|(field, (n, first))| format!("[{field}] x{n}: {first}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(rows_checked > 15_000, "{rows_checked}");
    assert!(arrivals >= 25, "{arrivals}");
}
