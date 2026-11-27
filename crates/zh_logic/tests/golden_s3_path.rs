//! Slice S3, path objects and movement-surface queries, against the reference
//! library (`tests/cpp_harness/gen_s3_path.cpp`).

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, VecDeque};

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, ICoord2D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::{
    CheckMovementInfo, Footprint, Path, PathAgent, PathQuery, PathUnit, PathUnits,
    PathfindLayerEnum, Pathfinder, TallBuilding, TallBuildings, TerrainSight, ViewParty,
};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S3_Path.json");

// ---- tokens -------------------------------------------------------------------

struct Tokens<'a>(std::str::SplitAsciiWhitespace<'a>);

impl<'a> Tokens<'a> {
    fn text(&mut self) -> &'a str {
        self.0.next().expect("token")
    }
    fn int(&mut self) -> Int {
        self.text().parse().expect("number")
    }
    fn id(&mut self) -> ObjectId {
        ObjectId(self.int())
    }
    fn flag(&mut self) -> bool {
        self.int() != 0
    }
    fn real(&mut self) -> Real {
        Real::from_bits(u32::from_str_radix(self.text(), 16).expect("bits"))
    }
    fn point(&mut self) -> Coord3D {
        Coord3D::new(self.real(), self.real(), self.real())
    }
    fn layer(&mut self) -> PathfindLayerEnum {
        PathfindLayerEnum::from_i32(self.int()).expect("layer")
    }
    fn rest(&mut self) -> Vec<&'a str> {
        self.0.by_ref().collect()
    }
}

fn f(v: Real) -> String {
    format!("{:08x}", v.to_bits())
}

fn c3(c: &Coord3D) -> String {
    format!("{} {} {}", f(c.x), f(c.y), f(c.z))
}

fn n(b: bool) -> Int {
    Int::from(b)
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

// ---- terrain ------------------------------------------------------------------

#[derive(Default)]
struct Tables {
    w: Int,
    h: Int,
    cliff: Vec<u8>,
    water: Vec<u8>,
    /// Answers of `getLayerForDestination` for the next positions asked.
    layers: RefCell<Vec<(Coord3D, PathfindLayerEnum)>>, // determinism-ok: test fake behind a &self trait
    asked: Cell<u32>,
}

impl Tables {
    fn at(&self, x: Real, y: Real) -> usize {
        let i = real_to_int_floor(x / 10.0).clamp(0, self.w + 1);
        let j = real_to_int_floor(y / 10.0).clamp(0, self.h + 1);
        (j * (self.w + 2) + i) as usize
    }
}

impl ThingTerrainAccess for Tables {
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(normal) = normal {
            *normal = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }
    fn align_on_terrain(
        &self,
        _angle: Real,
        _pos: &Coord3D,
        _stick: Bool,
        _mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        PathfindLayerEnum::Ground
    }
    fn is_cliff_cell(&self, x: Real, y: Real) -> Bool {
        Bool::from(self.cliff[self.at(x, y)] == b'1')
    }
    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        Bool::from(self.water[self.at(x, y)] == b'1')
    }
    fn is_clear_line_of_sight(&self, _a: &Coord3D, _b: &Coord3D) -> Bool {
        1
    }
    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum {
        self.asked.set(self.asked.get() + 1);
        self.layers
            .borrow()
            .iter()
            .find(|(p, _)| p == pos)
            .map_or(PathfindLayerEnum::Ground, |(_, l)| *l)
    }
}

// ---- units and buildings ------------------------------------------------------

#[derive(Default)]
struct Units {
    agents: BTreeMap<Int, PathAgent>,
    units: BTreeMap<Int, PathUnit>,
    /// (agent, other) -> (ally, can crush or squish).
    relations: BTreeMap<(Int, Int), (bool, bool)>,
}

impl PathUnits for Units {
    fn unit(&self, id: ObjectId) -> Option<PathUnit> {
        self.units.get(&id.0).copied()
    }
    fn is_ally(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.relations[&(agent.0, other.0)].0
    }
    fn can_crush_or_squish(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.relations[&(agent.0, other.0)].1
    }
}

#[derive(Default)]
struct Buildings {
    tall: BTreeMap<Int, TallBuilding>,
    /// The reference's partition searches, in order.
    tape: RefCell<VecDeque<(String, Int)>>, // determinism-ok: test fake behind a &self trait
}

impl TallBuildings for Buildings {
    fn tall_building(&self, id: ObjectId) -> Option<TallBuilding> {
        self.tall.get(&id.0).copied()
    }
    fn closest_tall_building(&self, pos: &Coord3D, radius: Real) -> Option<TallBuilding> {
        let (args, id) = self.tape.borrow_mut().pop_front().expect("a search");
        assert_eq!(
            format!("{} {}", c3(pos), f(radius)),
            args,
            "search arguments"
        );
        self.tall.get(&id).copied()
    }
}

// ---- paths --------------------------------------------------------------------

fn order(path: &Path) -> Vec<usize> {
    let mut nodes = Vec::new();
    let mut cursor = path.first_node();
    while let Some(i) = cursor {
        nodes.push(i);
        cursor = path.node(i).next();
    }
    nodes
}

fn dump(path: &Path) -> String {
    let nodes = order(path);
    let index = |i: Option<usize>| {
        i.and_then(|i| nodes.iter().position(|&n| n == i))
            .map_or(-1, |p| p as Int)
    };
    let mut out = format!("{} {}", nodes.len(), index(path.last_node()));
    for &i in &nodes {
        let node = path.node(i);
        let (dir, dist) = node.optimized_segment();
        out += &format!(
            " | {} {} {} {} {} {} {} {} {}",
            c3(node.position()),
            node.layer() as Int,
            n(node.can_optimize()),
            index(node.next_optimized()),
            f(dist),
            f(dir.x),
            f(dir.y),
            c3(&path.direction_vector(i)),
            index(node.previous()),
        );
    }
    out
}

fn save(path: &mut Path) -> String {
    let mut xfer = XferSave::new();
    path.xfer(&mut xfer).expect("save");
    hex(xfer.as_bytes())
}

// ---- replay -------------------------------------------------------------------

#[derive(Default)]
struct Scene {
    pf: Pathfinder,
    terrain: Tables,
    extent: Region3D,
    wall_height: Real,
    footprints: BTreeMap<Int, Footprint>,
    units: Units,
    buildings: Buildings,
    path: Path,
}

fn read_footprint(t: &mut Tokens) -> Footprint {
    let id = t.id();
    let position = t.point();
    let orientation = t.real();
    let geom_type = match t.int() {
        0 => GeometryType::Sphere,
        1 => GeometryType::Cylinder,
        _ => GeometryType::Box,
    };
    let small = t.int() as Bool;
    let height = t.real();
    let major = t.real();
    let minor = t.real();
    let fence_width = t.real();
    let fence_x_offset = t.real();
    let flags = t.int();
    let height_above_terrain = t.real();
    let bit = |b: Int| flags & b != 0;
    let mut geometry = GeometryInfo::new(geom_type, small, height, major, minor);
    geometry.set_minor_radius(minor);
    Footprint {
        id,
        position,
        orientation,
        geometry,
        fence_width,
        fence_x_offset,
        defensive_wall: bit(8),
        walk_on_top_of_wall: bit(32),
        structure: bit(64),
        see_through: bit(128),
        mobile: bit(256),
        height_above_terrain,
        ..Footprint::default()
    }
}

impl Scene {
    fn agent(&self, id: Int) -> Option<&PathAgent> {
        (id != 0).then(|| &self.units.agents[&id])
    }

    fn view_party(&self, id: Int, needs: bool, layer: PathfindLayerEnum) -> ViewParty {
        ViewParty {
            id: ObjectId(id),
            position: Coord3D::default(),
            layer,
            container: ObjectId::INVALID,
            slaver: ObjectId::INVALID,
            needs_line_of_sight: needs,
            significantly_above_terrain: false,
        }
    }

    /// The map and its occupants.
    fn setup(&mut self, op: &str, t: &mut Tokens) -> bool {
        match op {
            "map" => {
                self.extent = Region3D::default();
                self.extent.hi = Coord3D::new(t.real(), t.real(), 100.0);
                self.wall_height = t.real();
                self.pf = Pathfinder::new(self.wall_height);
            }
            "terrain" => {
                self.terrain.w = t.int();
                self.terrain.h = t.int();
                self.terrain.cliff = t.text().bytes().collect();
                self.terrain.water = t.text().bytes().collect();
            }
            "wallpiece" => self.pf.add_wall_piece(t.id()),
            "obj" => {
                let fp = read_footprint(t);
                self.footprints.insert(fp.id.0, fp);
            }
            "order" => {
                let all: Vec<Footprint> = t
                    .rest()
                    .iter()
                    .map(|id| self.footprints[&id.parse::<Int>().expect("id")].clone())
                    .collect();
                struct Walls<'a>(&'a BTreeMap<Int, Footprint>);
                impl zh_logic::pathfinder::PathWorld for Walls<'_> {
                    fn find_object(&self, id: ObjectId) -> Option<Footprint> {
                        self.0.get(&id.0).cloned()
                    }
                }
                let frame = 100;
                self.pf.new_map(
                    &self.terrain,
                    &self.extent,
                    self.wall_height,
                    &all,
                    &Walls(&self.footprints),
                    frame,
                );
            }
            "newmap" => {}
            "unit" => {
                let id = t.int();
                let has_ai = t.flag();
                let ignored_obstacle = t.id();
                let aircraft_adjusts = t.flag();
                let crusher_level = t.int() as u8;
                let crushable_level = t.int() as u8;
                let radius = t.real();
                let layer = t.layer();
                let infantry = t.flag();
                self.units.agents.insert(
                    id,
                    PathAgent {
                        id: ObjectId(id),
                        has_ai,
                        ignored_obstacle,
                        aircraft_adjusts,
                        crusher_level,
                        radius,
                        layer,
                        infantry,
                    },
                );
                self.units.units.insert(
                    id,
                    PathUnit {
                        has_ai,
                        crushable_level,
                        infantry,
                    },
                );
            }
            "rel" => {
                let agent = t.int();
                for pair in t.rest() {
                    let (other, bits) = pair.split_once(':').expect("pair");
                    let bits = bits.as_bytes();
                    self.units.relations.insert(
                        (agent, other.parse().expect("id")),
                        (bits[0] == b'1', bits[1] == b'1'),
                    );
                }
            }
            "goal" | "pos" | "air" => {
                let layer = t.layer();
                let at = ICoord2D::new(t.int(), t.int());
                let id = t.id();
                let idx = self.pf.get_cell(layer, at.x, at.y).expect("cell");
                let grid = self.pf.map_mut().grid_mut();
                match op {
                    "goal" => grid.set_goal_unit(idx, id, at),
                    "pos" => grid.set_pos_unit(idx, id, at),
                    _ => grid.set_goal_aircraft(idx, id, at),
                }
            }
            "tall" => {
                let id = t.int();
                let building = TallBuilding {
                    id: ObjectId(id),
                    position: t.point(),
                    radius: t.real(),
                };
                self.buildings.tall.insert(id, building);
            }
            "closest" => {
                let args = format!("{} {} {} {}", t.text(), t.text(), t.text(), t.text());
                let id = t.int();
                self.buildings.tape.borrow_mut().push_back((args, id));
            }
            "end" => *self = Scene::default(),
            "scenario" => {}
            _ => return false,
        }
        true
    }

    /// Answers a map question the way the harness printed it.
    fn query(&mut self, op: &str, t: &mut Tokens) -> Option<String> {
        Some(match op {
            "terr" => {
                let layer = t.layer();
                let surfaces = t.int();
                n(self.pf.valid_movement_terrain(layer, surfaces, &t.point())).to_string()
            }
            "valid" => {
                let crusher = t.flag();
                self.pf.ignore_obstacle_id = t.id();
                let layer = t.layer();
                let surfaces = t.int();
                let ok = self
                    .pf
                    .valid_movement_position(crusher, layer, surfaces, &t.point());
                self.pf.ignore_obstacle_id = ObjectId::INVALID;
                n(ok).to_string()
            }
            "best" => {
                let layer = t.layer();
                let pos = t.point();
                let mask = self.pf.locomotor_surfaces_at(layer, &pos);
                let mut best = -1;
                loop {
                    let token = t.text();
                    if token == "=" {
                        break;
                    }
                    let have: Int = token.parse().expect("surfaces");
                    if best == -1 && have & mask != 0 {
                        best = have;
                    }
                }
                best.to_string()
            }
            "dest" => {
                let agent = t.int();
                let layer = t.layer();
                let (x, y, radius, center) = (t.int(), t.int(), t.int(), t.flag());
                n(self.pf.check_destination(
                    &self.units,
                    self.agent(agent),
                    x,
                    y,
                    layer,
                    radius,
                    center,
                ))
                .to_string()
            }
            "move" => {
                let agent = t.int();
                let layer = t.layer();
                // The reference leaves the counts it was handed at zero.
                let mut info = CheckMovementInfo {
                    layer,
                    cell: ICoord2D::new(t.int(), t.int()),
                    radius: t.int(),
                    center_in_cell: t.flag(),
                    consider_transient: t.flag(),
                    ally_fixed_count: 9,
                    enemy_fixed: true,
                    ally_moving: true,
                    ally_goal: true,
                };
                let ok = self
                    .pf
                    .check_for_movement(&self.units, self.agent(agent), &mut info);
                format!(
                    "{} {} {} {} {}",
                    n(ok),
                    info.ally_fixed_count,
                    n(info.enemy_fixed),
                    n(info.ally_moving),
                    n(info.ally_goal)
                )
            }
            "clear" => {
                let crusher = t.flag();
                let (x, y) = (t.int(), t.int());
                let layer = t.layer();
                let diameter = t.int();
                self.pf
                    .clear_cell_for_diameter(&self.units, crusher, x, y, layer, diameter)
                    .to_string()
            }
            "cells" => {
                let a = ICoord2D::new(t.int(), t.int());
                let b = ICoord2D::new(t.int(), t.int());
                let layer = t.layer();
                let mut left = t.int();
                let mut seen = String::new();
                let mut previous = None;
                let ret = self
                    .pf
                    .iterate_cells_along_line(a, b, layer, &mut |from, to, x, y| {
                        let kind = match from {
                            None => "n",
                            Some(_) if from == previous => "p",
                            Some(_) => "x",
                        };
                        seen += &format!(" {x}:{y}:{kind}");
                        previous = Some(to);
                        left -= 1;
                        if left == 0 {
                            7
                        } else {
                            0
                        }
                    });
                format!("{ret}{seen}")
            }
            "line" => {
                let agent = t.int();
                let surfaces = t.int();
                let layer = t.layer();
                let (from, to) = (t.point(), t.point());
                let (blocked, pinched) = (t.flag(), t.flag());
                n(self.pf.is_line_passable(
                    &self.units,
                    self.agent(agent),
                    surfaces,
                    layer,
                    &from,
                    &to,
                    blocked,
                    pinched,
                ))
                .to_string()
            }
            "ground" => {
                let crusher = t.flag();
                let from = t.point();
                let layer = t.layer();
                let to = t.point();
                let diameter = t.int();
                n(self.pf.is_ground_path_passable(
                    &self.units,
                    crusher,
                    &from,
                    layer,
                    &to,
                    diameter,
                ))
                .to_string()
            }
            "quick" => {
                let surfaces = t.int();
                let (from, to) = (t.point(), t.point());
                let (from_layer, to_layer) = (t.layer(), t.layer());
                *self.terrain.layers.borrow_mut() = vec![(from, from_layer), (to, to_layer)];
                let a = self
                    .pf
                    .quick_path_exists(&self.terrain, surfaces, &from, &to);
                let b = self
                    .pf
                    .quick_path_exists_for_ui(&self.terrain, surfaces, &from, &to);
                format!("{} {}", n(a), n(b))
            }
            "view" | "sight" => {
                let uses = t.flag();
                let (aid, needs, alayer) = (t.int(), t.flag(), t.layer());
                let vid = t.int();
                let vlayer = t.int();
                let (from, to) = (t.point(), t.point());
                let mut attacker = self.view_party(aid, needs, alayer);
                let victim = (vid != 0).then(|| {
                    let layer = PathfindLayerEnum::from_i32(vlayer).expect("layer");
                    self.view_party(vid, false, layer)
                });
                let blocked = if op == "view" {
                    self.pf.is_attack_view_blocked(
                        uses,
                        &attacker,
                        &from,
                        victim.as_ref(),
                        &to,
                        TerrainSight::NoWeapon,
                    )
                } else {
                    let mut victim = victim.expect("victim");
                    attacker.position = from;
                    victim.position = to;
                    victim.significantly_above_terrain = t.flag();
                    self.pf
                        .is_view_blocked(uses, &attacker, &victim, TerrainSight::NoWeapon)
                };
                n(blocked).to_string()
            }
            "segment" => {
                let from = t.point();
                let mut to = t.point();
                let ignore = t.id();
                let around = self.pf.segment_intersects_tall_building(
                    &self.buildings,
                    &from,
                    &mut to,
                    ignore,
                );
                let mut out = format!("{} {}", n(around.is_some()), c3(&to));
                for p in around.iter().flatten() {
                    out += &format!(" {}", c3(p));
                }
                out
            }
            "circle" => {
                let (from, to) = (t.point(), t.point());
                let radius = t.real();
                let adjusted =
                    Pathfinder::circle_clips_tall_building(&self.buildings, &from, &to, radius);
                format!("{} {}", n(adjusted.is_some()), c3(&adjusted.unwrap_or(to)))
            }
            "aircraft" => {
                let (at, to) = (t.point(), t.point());
                let winged = t.flag();
                let path =
                    self.pf
                        .aircraft_path(&self.buildings, &at, &to, winged, ObjectId::INVALID);
                assert!(path.is_optimized());
                dump(&path)
            }
            _ => return None,
        })
    }

    /// Replays a path operation; the answer is empty for a plain step.
    fn path_op(&mut self, op: &str, t: &mut Tokens) -> String {
        match op {
            "path" => self.path = Path::new(),
            "append" | "prepend" => {
                let pos = t.point();
                let layer = t.layer();
                if op == "append" {
                    self.path.append_node(&pos, layer);
                } else {
                    self.path.prepend_node(&pos, layer);
                }
            }
            "canopt" => {
                let node = order(&self.path)[t.int() as usize];
                self.path.set_can_optimize(node, t.flag());
            }
            "dump" => return dump(&self.path),
            "opt" => {
                let agent = t.int();
                let surfaces = t.int();
                let blocked = t.flag();
                let query = PathQuery {
                    pathfinder: &self.pf,
                    units: &self.units,
                    agent: (agent != 0).then(|| &self.units.agents[&agent]),
                };
                self.path.optimize(&query, surfaces, blocked);
            }
            "gopt" => {
                let crusher = t.flag();
                let diameter = t.int();
                self.path
                    .optimize_ground_path(&self.pf, &self.units, crusher, diameter);
            }
            "blocked" => {
                self.path.set_blocked_by_ally(t.int() as Bool);
                return self.path.blocked_by_ally().to_string();
            }
            "cpop" => {
                let mut agent = self.units.agents[&t.int()].clone();
                agent.layer = t.layer();
                let surfaces = t.int();
                let pos = t.point();
                let query = PathQuery {
                    pathfinder: &self.pf,
                    units: &self.units,
                    agent: Some(&agent),
                };
                let out = self.path.compute_point_on_path(&query, surfaces, &pos);
                return format!(
                    "{} {} {} {}",
                    f(out.dist_along_path),
                    c3(&out.pos_on_path),
                    out.layer as Int,
                    c3(&self.path.peek_cached_point_on_path())
                );
            }
            "flight" => {
                let (dist, goal) = self.path.compute_flight_dist_to_goal(&t.point());
                return format!("{} {}", f(dist), c3(&goal));
            }
            "empty" => {
                let pos = t.point();
                let query = PathQuery {
                    pathfinder: &self.pf,
                    units: &self.units,
                    agent: None,
                };
                let out = self.path.compute_point_on_path(&query, 1, &pos);
                let (dist, goal) = self.path.compute_flight_dist_to_goal(&pos);
                return format!(
                    "{} {} {} {} {}",
                    f(out.dist_along_path),
                    c3(&out.pos_on_path),
                    out.layer as Int,
                    f(dist),
                    c3(&goal)
                );
            }
            "update" => {
                let pos = t.point();
                let layer = t.layer();
                *self.terrain.layers.borrow_mut() = vec![(pos, layer)];
                self.path.update_last_node(&self.terrain, &pos);
            }
            "setopt" => {
                let nodes = order(&self.path);
                let (i, j) = (t.int(), t.int());
                let next = (j >= 0).then(|| nodes[j as usize]);
                self.path.set_next_optimized(nodes[i as usize], next);
            }
            "save" => {
                let bytes = save(&mut self.path);
                return format!("{bytes} {}", XferCrc::new().get_crc());
            }
            "load" => {
                let text = t.text();
                let bytes: Vec<u8> = (0..text.len())
                    .step_by(2)
                    .map(|i| u8::from_str_radix(&text[i..i + 2], 16).expect("hex"))
                    .collect();
                let mut loaded = Path::new();
                let mut xfer = XferLoad::new(&bytes);
                loaded.xfer(&mut xfer).expect("load");
                assert_eq!(xfer.position(), bytes.len(), "whole save read");
                // The following `resave` row is of the loaded path.
                self.path = loaded;
                return dump(&self.path);
            }
            "resave" => return save(&mut self.path),
            other => panic!("unknown row {other}"),
        }
        String::new()
    }
}

fn rows() -> Vec<&'static str> {
    VECTORS
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.strip_prefix('"')?.strip_suffix('"')
        })
        .collect()
}

#[test]
fn paths_and_movement_queries_match_row_for_row() {
    let mut scene = Scene::default();
    let mut checked: BTreeMap<&str, u32> = BTreeMap::new();
    for (line, row) in rows().into_iter().enumerate() {
        let (input, want) = match row.split_once(" = ") {
            Some((input, want)) => (input, Some(want)),
            None => (row, None),
        };
        let mut t = Tokens(input.split_ascii_whitespace());
        let op = t.text();
        // `best` reads up to the separator.
        if op == "best" {
            t = Tokens(row.split_ascii_whitespace());
            t.text();
        }
        if scene.setup(op, &mut t) {
            continue;
        }
        let got = match scene.query(op, &mut t) {
            Some(got) => got,
            None => scene.path_op(op, &mut t),
        };
        if let Some(want) = want {
            assert_eq!(got, want, "row {line}: {row:.200}");
            *checked.entry(op).or_default() += 1;
        }
        assert!(
            scene.buildings.tape.borrow().is_empty(),
            "row {line}: searches left on the tape"
        );
    }
    let total: u32 = checked.values().sum();
    assert!(total > 15_000, "only {total} rows checked: {checked:?}");
    for op in [
        "terr", "valid", "best", "dest", "move", "clear", "cells", "line", "ground", "quick",
        "view", "sight", "segment", "circle", "aircraft", "dump", "cpop", "flight", "empty",
        "blocked", "save", "load", "resave",
    ] {
        assert!(checked.get(op).is_some_and(|&c| c > 0), "no {op} rows");
    }
}

/// Flat ground with water east of x = 100.
struct HalfFlooded;

impl ThingTerrainAccess for HalfFlooded {
    fn get_ground_height(&self, _x: Real, _y: Real, _normal: Option<&mut Coord3D>) -> Real {
        0.0
    }
    fn align_on_terrain(
        &self,
        _angle: Real,
        _pos: &Coord3D,
        _stick: Bool,
        _mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        PathfindLayerEnum::Ground
    }
    fn is_cliff_cell(&self, _x: Real, _y: Real) -> Bool {
        0
    }
    fn is_underwater(
        &self,
        x: Real,
        _y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        Bool::from(x >= 100.0)
    }
    fn is_clear_line_of_sight(&self, _a: &Coord3D, _b: &Coord3D) -> Bool {
        1
    }
}

#[test]
fn sim_asks_the_map_about_movement_surfaces() {
    use zh_common::ini::{IniLoadType, IniReader};
    use zh_common::random_value::PrngState;
    use zh_common::types::TeamId;
    use zh_logic::combat::locomotor::{Locomotor, LocomotorTemplate};
    use zh_logic::sim::{SimApi, SimContext, SimView};

    let mut sim = SimContext::new();
    let ini = "Object Rider\n  KindOf = VEHICLE\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 5.0\n  GeometryHeight = 10.0\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    let template = sim
        .rules
        .thing_factory
        .find_template_id("Rider")
        .expect("Rider");
    let rider = sim.spawn_object(template, TeamId::default(), Coord3D::default(), 0.0);
    sim.terrain = Box::new(HalfFlooded);
    let water = Coord3D::new(155.0, 55.0, 0.0);
    let land = Coord3D::new(55.0, 55.0, 0.0);
    let ask = |sim: &SimContext, pos: &Coord3D| {
        sim.valid_movement_terrain(PathfindLayerEnum::Ground, rider, pos)
    };

    let mut boat = LocomotorTemplate::new();
    boat.surfaces = 2;
    let locomotor = Locomotor::new(&boat, &mut PrngState::default(), 0);
    sim.world
        .objects
        .get_mut(rider)
        .expect("rider")
        .ai_locomotor = Some(Box::new(locomotor));

    // Without a map the stand-in knows no water.
    assert!(sim.pathfinder().is_none());
    assert_eq!(ask(&sim, &land), 1);

    sim.new_pathfind_map(
        &Region3D {
            lo: Coord3D::default(),
            hi: Coord3D::new(200.0, 200.0, 100.0),
        },
        0.0,
    );
    assert!(sim.pathfinder().is_some());
    assert_eq!(ask(&sim, &water), 1);
    assert_eq!(ask(&sim, &land), 0);

    // A rider without a locomotor object keeps the stand-in.
    sim.world
        .objects
        .get_mut(rider)
        .expect("rider")
        .ai_locomotor = None;
    assert_eq!(ask(&sim, &land), 1);
}

#[test]
fn locomotion_and_the_ai_ask_the_map_before_the_seams() {
    use zh_common::ini::{IniLoadType, IniReader};
    use zh_common::types::TeamId;
    use zh_logic::combat::locomotor::{LocomotorSeams, LocomotorTemplate, LocomotorWorld};
    use zh_logic::combat::locomotor_set::LocomotorSetType;
    use zh_logic::modules::update::ai_update::choose_good_locomotor;
    use zh_logic::sim::{SimApi, SimContext, SimView};

    struct Refuse;
    impl LocomotorSeams for Refuse {
        fn valid_movement_terrain(
            &mut self,
            _layer: PathfindLayerEnum,
            _surfaces: Int,
            _pos: &Coord3D,
        ) -> Bool {
            0
        }
        fn ground_or_structure_height(&mut self, _x: Real, _y: Real) -> Real {
            0.0
        }
    }

    let mut sim = SimContext::new();
    for (name, surfaces) in [("Legs", "GROUND"), ("Fins", "WATER")] {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(&format!(
            "Locomotor {name}\n  Surfaces = {surfaces}\n  Appearance = TWO_LEGS\nEnd\n"
        ))
        .expect("locomotor");
        t.name = name.to_owned();
        sim.rules.locomotor_store.add_template(t);
    }
    let ini = "Object Swimmer\n  KindOf = INFANTRY\n  Geometry = CYLINDER\n  \
               GeometryMajorRadius = 5.0\n  GeometryHeight = 10.0\n  \
               Behavior = AIUpdateInterface ModuleTag_AI\n  End\n  \
               Locomotor = SET_NORMAL Legs Fins\n  Locomotor = SET_WANDER Legs Fins\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.terrain = Box::new(HalfFlooded);
    let template = sim
        .rules
        .thing_factory
        .find_template_id("Swimmer")
        .expect("Swimmer");
    let slot = sim.spawn_object(template, TeamId::default(), Coord3D::default(), 0.0);
    let water = Coord3D::new(155.0, 55.0, 0.0);
    let land = Coord3D::new(55.0, 55.0, 0.0);
    let ask = |sim: &mut SimContext, pos: &Coord3D| {
        let mut seams = Refuse;
        LocomotorWorld {
            sim,
            owner: slot,
            seams: &mut seams,
        }
        .valid_movement_terrain(PathfindLayerEnum::Ground, 2, pos)
    };
    assert_eq!(ask(&mut sim, &water), 0, "the seam answers without a map");
    sim.new_pathfind_map(
        &Region3D {
            lo: Coord3D::default(),
            hi: Coord3D::new(200.0, 200.0, 100.0),
        },
        0.0,
    );
    assert_eq!(ask(&mut sim, &water), 1);
    assert_eq!(ask(&mut sim, &land), 0);

    // `chooseGoodLocomotorFromCurrentSet` picks by the cell under the object.
    let surfaces = |sim: &SimContext| {
        sim.cur_locomotor(slot)
            .expect("locomotor")
            .get_legal_surfaces()
    };
    let module = sim.object(slot).and_then(|o| o.ai_slot).expect("AI");
    let context = sim.world.modules.meta(module).context(module);
    sim.set_object_position(slot, &water);
    choose_good_locomotor(&mut sim, context);
    assert_eq!(surfaces(&sim), 2, "swimming");
    sim.set_object_position(slot, &land);
    choose_good_locomotor(&mut sim, context);
    assert_eq!(surfaces(&sim), 1, "walking");

    // `chooseLocomotorSet` goes the same way.
    sim.ai_choose_locomotor_set(slot, LocomotorSetType::Wander);
    sim.set_object_position(slot, &water);
    sim.ai_choose_locomotor_set(slot, LocomotorSetType::Normal);
    assert_eq!(surfaces(&sim), 2);
}

/// Retail compatible pathfinding keeps a cell's parent link on release; the
/// fixed code clears it.
#[test]
fn releasing_an_occupied_cell_keeps_its_parent_until_pathfinding_is_fixed() {
    let mut pathfinder = Pathfinder::new(0.0);
    pathfinder.new_map(
        &HalfFlooded,
        &Region3D {
            lo: Coord3D::default(),
            hi: Coord3D::new(100.0, 100.0, 100.0),
        },
        0.0,
        &[],
        &zh_logic::pathfinder::NoPathWorld,
        0,
    );
    let (at, parent_at) = (ICoord2D::new(2, 2), ICoord2D::new(3, 2));
    let cell = pathfinder
        .get_cell(PathfindLayerEnum::Ground, at.x, at.y)
        .expect("cell");
    let parent = pathfinder
        .get_cell(PathfindLayerEnum::Ground, parent_at.x, parent_at.y)
        .expect("parent");
    let grid = pathfinder.map_mut().grid_mut();
    grid.set_pos_unit(cell, ObjectId(7), at);
    assert!(grid.allocate_info(parent, parent_at));
    grid.set_parent_cell(cell, parent);
    let linked = |grid: &zh_logic::pathfinder::PathfindCellGrid| {
        let info = grid.cell(cell).info_idx.expect("info");
        grid.pool().get(info).expect("record").path_parent.is_some()
    };
    grid.release_info(cell);
    assert!(linked(grid));
    grid.mode_mut().use_fixed_pathfinding = true;
    grid.release_info(cell);
    assert!(!linked(grid));
}

/// The weapon's answer about the terrain is an input, so no vector covers it.
#[test]
fn a_weapon_blocked_by_terrain_blocks_the_view() {
    let pathfinder = Pathfinder::new(0.0);
    let party = ViewParty {
        id: ObjectId(1),
        position: Coord3D::default(),
        layer: PathfindLayerEnum::Ground,
        container: ObjectId::INVALID,
        slaver: ObjectId::INVALID,
        needs_line_of_sight: true,
        significantly_above_terrain: false,
    };
    let blocked = |sight| pathfinder.is_view_blocked(true, &party, &party, sight);
    assert!(blocked(TerrainSight::Weapon {
        terrain_blocked: true
    }));
    assert!(!blocked(TerrainSight::Weapon {
        terrain_blocked: false
    }));
    let contact = blocked(TerrainSight::Contact {
        terrain_blocked: true,
    });
    assert_eq!(contact, cfg!(feature = "retail-crc"));
}
