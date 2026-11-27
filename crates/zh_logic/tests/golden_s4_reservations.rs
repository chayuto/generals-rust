//! Slice S4, goal reservations and destination adjustment, against the
//! reference library (`tests/cpp_harness/gen_s4_reservations.cpp`).

use std::collections::{BTreeMap, VecDeque};

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, ICoord2D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::{
    AdjustUnit, AllyMover, AllyUnit, Footprint, GoalMover, MoveAlliesWorld, NoPathCost, Path,
    PathAgent, PathCost, PathUnit, PathUnits, PathWorld, PathfindLayerEnum, Pathfinder,
};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S4_Reservations.json");

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
    fn equals(&mut self) {
        assert_eq!(self.text(), "=");
    }
    fn rest(&mut self) -> Vec<&'a str> {
        self.0.by_ref().collect()
    }
}

fn split(row: &str) -> (&str, Tokens<'_>) {
    let mut tokens = Tokens(row.split_ascii_whitespace());
    (tokens.text(), tokens)
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

// ---- terrain ------------------------------------------------------------------

#[derive(Default)]
struct Tables {
    w: Int,
    h: Int,
    cliff: Vec<u8>,
    water: Vec<u8>,
    /// Cells where a destination is above the ground.
    up: Vec<u8>,
    bridge_end: Vec<u8>,
}

impl Tables {
    fn at(&self, x: Real, y: Real) -> usize {
        let i = real_to_int_floor(x / 10.0).clamp(0, self.w + 1);
        let j = real_to_int_floor(y / 10.0).clamp(0, self.h + 1);
        (j * (self.w + 2) + i) as usize
    }
    fn interacts(&self, pos: &Coord3D) -> bool {
        self.bridge_end[self.at(pos.x, pos.y)] == b'1'
    }
}

impl ThingTerrainAccess for Tables {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(normal) = normal {
            *normal = Coord3D::new(0.0, 0.0, 1.0);
        }
        x * 0.01 + y * 0.003
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
    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum {
        if self.up[self.at(pos.x, pos.y)] == b'1' {
            PathfindLayerEnum::Wall
        } else {
            PathfindLayerEnum::Ground
        }
    }
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        self.get_ground_height(x, y, normal) + 3.0 * (layer as Int as Real)
    }
}

// ---- units --------------------------------------------------------------------

#[derive(Clone)]
struct Unit {
    mover: GoalMover,
    position: Coord3D,
    crushable_level: u8,
    dozer_or_harvester: bool,
    aircraft: bool,
    projectile: bool,
    human: bool,
}

impl Unit {
    fn adjust(&self, surfaces: Int) -> AdjustUnit {
        AdjustUnit {
            agent: self.mover.agent.clone(),
            position: self.position,
            surfaces,
            aircraft: self.aircraft,
            projectile: self.projectile,
            human: self.human,
        }
    }
}

#[derive(Default, Clone, Copy)]
struct AllyState {
    moving: bool,
    attacking: bool,
    using_ability: bool,
    busy: bool,
    move_when_told: bool,
    path: Option<usize>,
}

#[derive(Default)]
struct Units {
    all: BTreeMap<Int, Unit>,
    /// The order the reference dumps them in.
    order: Vec<Int>,
    /// (unit, other) -> (ally, can crush or squish).
    relations: BTreeMap<(Int, Int), (bool, bool)>,
}

impl PathUnits for Units {
    fn unit(&self, id: ObjectId) -> Option<PathUnit> {
        self.all.get(&id.0).map(|u| PathUnit {
            has_ai: u.mover.agent.has_ai,
            crushable_level: u.crushable_level,
            infantry: u.mover.agent.infantry,
        })
    }
    fn is_ally(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.relations[&(agent.0, other.0)].0
    }
    fn can_crush_or_squish(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.relations[&(agent.0, other.0)].1
    }
}

/// `moveAllies` against the recorded orders, in order.
struct Allies<'a> {
    units: &'a Units,
    states: &'a mut BTreeMap<Int, AllyState>,
    paths: &'a BTreeMap<usize, Path>,
    tape: &'a mut VecDeque<String>,
}

fn ally_mover(unit: &Unit) -> AllyMover {
    AllyMover {
        agent: unit.mover.agent.clone(),
        dozer_or_harvester: unit.dozer_or_harvester,
    }
}

impl MoveAlliesWorld for Allies<'_> {
    fn ally(&mut self, mover: ObjectId, other: ObjectId) -> Option<AllyUnit> {
        let unit = self.units.all.get(&other.0)?;
        let state = self.states.get(&other.0).copied().unwrap_or_default();
        Some(AllyUnit {
            ally: self.units.is_ally(mover, other),
            infantry: unit.mover.agent.infantry,
            has_ai: unit.mover.agent.has_ai,
            moving: state.moving,
            attacking: state.attacking,
            using_ability: state.using_ability,
            busy: state.busy,
        })
    }

    fn move_away_from_unit(
        &mut self,
        pathfinder: &mut Pathfinder,
        other: ObjectId,
        mover: ObjectId,
    ) {
        let expected = self.tape.pop_front().expect("an order");
        assert_eq!(format!("away {} {}", other.0, mover.0), expected);
        let state = self.states.entry(other.0).or_default();
        if state.move_when_told {
            state.moving = true;
        }
        let Some(path) = state.path else { return };
        let paths = self.paths;
        let nested = ally_mover(&self.units.all[&other.0]);
        let ran = pathfinder.move_allies(self, &nested, &paths[&path]);
        let expected = self.tape.pop_front().expect("a nested answer");
        assert_eq!(format!("nested {} = {}", other.0, n(ran)), expected);
    }
}

/// `checkPathCost` from the table the reference filled for every cell.
struct Costs {
    from: Coord3D,
    table: BTreeMap<(u32, u32), Int>,
}

impl PathCost for Costs {
    fn check_path_cost(
        &mut self,
        _pathfinder: &mut Pathfinder,
        _unit: &AdjustUnit,
        from: &Coord3D,
        to: &Coord3D,
    ) -> Int {
        assert_eq!(c3(from), c3(&self.from), "cost from");
        self.table[&(to.x.to_bits(), to.y.to_bits())]
    }
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
    states: BTreeMap<Int, AllyState>,
    paths: BTreeMap<usize, Path>,
    costs: Option<(bool, Vec<(Int, usize)>)>,
    seen: BTreeMap<String, String>,
}

struct Walls<'a>(&'a BTreeMap<Int, Footprint>);

impl PathWorld for Walls<'_> {
    fn find_object(&self, id: ObjectId) -> Option<Footprint> {
        self.0.get(&id.0).cloned()
    }
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
    fn setup(&mut self, op: &str, t: &mut Tokens) -> bool {
        match op {
            "scenario" | "newmap" => {}
            "end" => *self = Scene::default(),
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
            "tables" => {
                self.terrain.up = t.text().bytes().collect();
                self.terrain.bridge_end = t.text().bytes().collect();
            }
            "wallpiece" => self.pf.add_wall_piece(t.id()),
            "obj" => {
                let footprint = read_footprint(t);
                self.footprints.insert(footprint.id.0, footprint);
            }
            "order" => {
                let all: Vec<Footprint> = t
                    .rest()
                    .iter()
                    .map(|id| self.footprints[&id.parse::<Int>().expect("id")].clone())
                    .collect();
                self.pf.new_map(
                    &self.terrain,
                    &self.extent,
                    self.wall_height,
                    &all,
                    &Walls(&self.footprints),
                    100,
                );
            }
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
                let immobile = t.flag();
                let ground_movement = t.flag();
                let unmanned_helicopter = t.flag();
                let dozer_or_harvester = t.flag();
                let aircraft = t.flag();
                let projectile = t.flag();
                let human = t.flag();
                let position = t.point();
                let agent = PathAgent {
                    id: ObjectId(id),
                    has_ai,
                    ignored_obstacle,
                    aircraft_adjusts,
                    crusher_level,
                    radius,
                    layer,
                    infantry,
                };
                let mover = GoalMover {
                    agent,
                    immobile,
                    ground_movement,
                    unmanned_helicopter,
                    destination_layer: PathfindLayerEnum::Ground,
                    goal_cell: ICoord2D::new(-1, -1),
                    cur_cell: ICoord2D::new(-1, -1),
                };
                // The reference puts every unit on the ground layer afterwards.
                let mut mover = mover;
                mover.agent.layer = PathfindLayerEnum::Ground;
                self.units.order.push(id);
                self.units.all.insert(
                    id,
                    Unit {
                        mover,
                        position,
                        crushable_level,
                        dozer_or_harvester,
                        aircraft,
                        projectile,
                        human,
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
            "logical" => {
                let mut region = self.extent;
                region.lo.x = t.real();
                region.lo.y = t.real();
                region.hi.x = t.real();
                region.hi.y = t.real();
                assert!(self.pf.begin_path_queue(&region, 100));
            }
            _ => return false,
        }
        true
    }

    /// The cells and units that changed since the last call, as the harness
    /// prints them.
    fn diff(&mut self) -> String {
        let mut now = Vec::new();
        for layer in [PathfindLayerEnum::Ground, PathfindLayerEnum::Wall] {
            for x in 0..=self.terrain.w + 1 {
                for y in 0..=self.terrain.h + 1 {
                    let Some(idx) = self.pf.get_cell(layer, x, y) else {
                        continue;
                    };
                    let cell = self.pf.map().grid().cell(idx);
                    let value = format!(
                        "{}.{}.{}.{}.{}.{}",
                        cell.flags() as Int,
                        self.pf.goal_unit(idx).0,
                        self.pf.unit_present(idx).0,
                        n(cell.aircraft_goal),
                        self.pf.goal_aircraft(idx).0,
                        n(cell.info_idx.is_some()),
                    );
                    now.push((format!("{}.{x}.{y}", layer as Int), value));
                }
            }
        }
        for id in &self.units.order {
            let m = &self.units.all[id].mover;
            let mut value = format!("{}.{}", m.destination_layer as Int, m.agent.layer as Int);
            if m.agent.has_ai {
                value += &format!(
                    ".{}.{}.{}.{}",
                    m.goal_cell.x, m.goal_cell.y, m.cur_cell.x, m.cur_cell.y
                );
            }
            now.push((format!("u{id}"), value));
        }
        let mut out = String::from("d");
        for (key, value) in now {
            let old = self
                .seen
                .entry(key.clone())
                .or_insert_with(|| "0.0.0.0.0.0".to_owned());
            if *old != value {
                out += &format!(" {key}={value}");
                *old = value;
            }
        }
        out
    }

    /// Applies `act` to a unit's mover. The bridge question is answered from
    /// where the unit stands.
    fn with_mover(
        &mut self,
        id: Int,
        act: impl FnOnce(&mut Pathfinder, &mut GoalMover, &mut dyn FnMut(PathfindLayerEnum) -> bool),
    ) {
        let unit = self.units.all.get_mut(&id).expect("unit");
        let interacts = self.terrain.interacts(&unit.position);
        act(&mut self.pf, &mut unit.mover, &mut |_| interacts);
    }

    fn set_layer(&mut self, id: Int, layer: PathfindLayerEnum) {
        let position = self.units.all[&id].position;
        if self.units.all[&id].mover.agent.layer == layer {
            return;
        }
        self.with_mover(id, |pf, mover, bridge| {
            pf.remove_pos(mover);
            mover.agent.layer = layer;
            pf.update_pos(mover, &position, bridge);
        });
    }

    fn operation(&mut self, op: &str, t: &mut Tokens) -> bool {
        match op {
            "goal" => {
                let id = t.int();
                let pos = t.point();
                let layer = t.layer();
                self.with_mover(id, |pf, m, bridge| pf.update_goal(m, &pos, layer, bridge));
            }
            "move" => {
                let id = t.int();
                let pos = t.point();
                self.units.all.get_mut(&id).expect("unit").position = pos;
                self.with_mover(id, |pf, m, bridge| pf.update_pos(m, &pos, bridge));
            }
            "rmgoal" => {
                let id = t.int();
                self.with_mover(id, |pf, m, _| pf.remove_goal(m));
            }
            "rmpos" => {
                let id = t.int();
                self.with_mover(id, |pf, m, _| pf.remove_pos(m));
            }
            "rmunit" => {
                let id = t.int();
                self.with_mover(id, |pf, m, _| pf.remove_unit_from_map(m));
            }
            "setlayer" => {
                let id = t.int();
                let layer = t.layer();
                self.set_layer(id, layer);
            }
            "uplayer" => {
                let id = t.int();
                let asked = t.layer();
                t.equals();
                let expected = t.layer();
                let interacts = self.terrain.interacts(&self.units.all[&id].position);
                let layer = Pathfinder::layer_for_update(asked, &mut |_| interacts);
                assert_eq!(layer, expected, "layer");
                self.set_layer(id, layer);
            }
            "airgoal" => {
                let id = t.int();
                let pos = t.point();
                self.with_mover(id, |pf, m, _| pf.update_aircraft_goal(m, &pos));
            }
            "ground" => {
                let id = t.int();
                let flag = t.flag();
                self.units
                    .all
                    .get_mut(&id)
                    .expect("unit")
                    .mover
                    .ground_movement = flag;
            }
            "ignore" => {
                let id = t.int();
                let ignored = t.id();
                self.units
                    .all
                    .get_mut(&id)
                    .expect("unit")
                    .mover
                    .agent
                    .ignored_obstacle = ignored;
            }
            "forget" => {
                let id = t.int();
                self.units.all.get_mut(&id).expect("unit").mover.goal_cell = ICoord2D::new(-1, -1);
            }
            _ => return false,
        }
        true
    }

    /// Answers a question the way the harness printed it.
    fn query(&mut self, row: &str, tape: &mut VecDeque<String>) -> Option<String> {
        let (op, mut t) = split(row);
        let head = row.split(" = ").next().expect("question");
        let answer = match op {
            "snap" => {
                let unit = &self.units.all[&t.int()];
                let mut pos = t.point();
                self.pf
                    .snap_position(&self.terrain, Some(&unit.mover.agent), &mut pos);
                c3(&pos)
            }
            "snapgoal" => {
                let unit = &self.units.all[&t.int()];
                let mut pos = t.point();
                self.pf.snap_closest_goal_position(
                    &self.terrain,
                    &self.units,
                    &unit.mover.agent,
                    &mut pos,
                );
                c3(&pos)
            }
            "goalpos" => {
                let unit = &self.units.all[&t.int()];
                let mut pos = t.point();
                let has = self.pf.goal_position(
                    &self.terrain,
                    &unit.mover.agent,
                    unit.mover.goal_cell,
                    &mut pos,
                );
                format!("{} {}", n(has), c3(&pos))
            }
            "adjust" => {
                let unit = self.units.all[&t.int()].adjust(t.int());
                let mut pos = t.point();
                let ok = self.pf.adjust_destination(
                    &self.terrain,
                    &self.units,
                    &mut NoPathCost,
                    &unit,
                    &mut pos,
                    None,
                );
                format!("{} {}", n(ok), c3(&pos))
            }
            "adjustgroup" => {
                let unit = self.units.all[&t.int()].adjust(t.int());
                let mut pos = t.point();
                let group = t.point();
                let mut costs = self.cost_table(group);
                let ok = self.pf.adjust_destination(
                    &self.terrain,
                    &self.units,
                    &mut costs,
                    &unit,
                    &mut pos,
                    Some(&group),
                );
                format!("{} {}", n(ok), c3(&pos))
            }
            "possible" => {
                let unit = self.units.all[&t.int()].adjust(t.int());
                let mut pos = t.point();
                let ok = self.pf.adjust_to_possible_destination(
                    &self.terrain,
                    &self.units,
                    &unit,
                    &mut pos,
                );
                format!("{} {}", n(ok), c3(&pos))
            }
            "land" => {
                let unit = &self.units.all[&t.int()];
                let mut pos = t.point();
                let ok = self.pf.adjust_to_landing_destination(
                    &self.terrain,
                    &self.units,
                    &self.extent,
                    &unit.mover.agent,
                    &unit.position,
                    &mut pos,
                );
                format!("{} {}", n(ok), c3(&pos))
            }
            "tighten" => {
                let unit = self.units.all[&t.int()].adjust(t.int());
                let mut from = t.point();
                let to = t.point();
                self.pf
                    .tighten_path(&self.terrain, &self.units, &unit, &mut from, &to);
                c3(&from)
            }
            "uplayer" => return None,
            "target" => {
                let unit = &self.units.all[&t.int()];
                let mut pos = t.point();
                let ok = self.pf.adjust_target_destination(
                    &self.terrain,
                    &self.units,
                    &unit.mover.agent,
                    &mut |at| {
                        let row = tape.pop_front().expect("a range question");
                        let (kind, mut r) = split(&row);
                        assert_eq!(kind, "range");
                        assert_eq!(c3(at), c3(&r.point()), "range question");
                        r.int() as Bool
                    },
                    &mut pos,
                );
                return Some(format!("targeted = {} {}", n(ok), c3(&pos)));
            }
            "allies" => {
                let mover = ally_mover(&self.units.all[&t.int()]);
                let mut world = Allies {
                    units: &self.units,
                    states: &mut self.states,
                    paths: &self.paths,
                    tape,
                };
                let ran = self.pf.move_allies(&mut world, &mover, &self.paths[&0]);
                return Some(format!("allied = {}", n(ran)));
            }
            "madest" => {
                let unit = self.units.all[&t.int()].clone();
                let to = t.point();
                let mut world = Allies {
                    units: &self.units,
                    states: &mut self.states,
                    paths: &self.paths,
                    tape,
                };
                self.pf.move_allies_away_from_destination(
                    &self.terrain,
                    &mut world,
                    &unit.mover.agent,
                    &unit.position,
                    &to,
                );
                return Some("madested".to_owned());
            }
            _ => return None,
        };
        Some(format!("{head} = {answer}"))
    }

    fn cost_table(&mut self, from: Coord3D) -> Costs {
        let (center, runs) = self.costs.take().expect("a cost table");
        let mut values = runs
            .iter()
            .flat_map(|&(cost, count)| std::iter::repeat_n(cost, count));
        let mut table = BTreeMap::new();
        for x in 0..=self.terrain.w + 1 {
            for y in 0..=self.terrain.h + 1 {
                let to = self.pf.adjust_coord_to_cell(
                    &self.terrain,
                    x,
                    y,
                    center,
                    PathfindLayerEnum::Ground,
                );
                let cost = values.next().expect("a cost for every cell");
                table.insert((to.x.to_bits(), to.y.to_bits()), cost);
            }
        }
        assert_eq!(values.next(), None);
        Costs { from, table }
    }

    fn ally_row(&mut self, op: &str, t: &mut Tokens) -> bool {
        match op {
            "ally" => {
                let id = t.int();
                let state = AllyState {
                    moving: t.flag(),
                    attacking: t.flag(),
                    using_ability: t.flag(),
                    busy: t.flag(),
                    move_when_told: t.flag(),
                    path: None,
                };
                self.states.insert(id, state);
            }
            "path" => {
                let index = t.int() as usize;
                if index == 0 {
                    self.paths.clear();
                }
                let mut path = Path::default();
                path.set_blocked_by_ally(t.int() as Bool);
                let rest = t.rest();
                for node in rest.chunks(4) {
                    let text = node.join(" ");
                    let mut node = Tokens(text.split_ascii_whitespace());
                    let pos = node.point();
                    path.append_node(&pos, node.layer());
                }
                self.paths.insert(index, path);
            }
            "nest" => {
                let id = t.int();
                let index = t.int() as usize;
                self.states.get_mut(&id).expect("state").path = Some(index);
            }
            "costs" => {
                let center = t.flag();
                let runs = t
                    .rest()
                    .iter()
                    .map(|run| {
                        let (cost, count) = run.split_once('*').expect("run");
                        (cost.parse().expect("cost"), count.parse().expect("count"))
                    })
                    .collect();
                self.costs = Some((center, runs));
            }
            _ => return false,
        }
        true
    }
}

#[test]
fn s4_reservations_match_the_reference() {
    let rows: Vec<&str> = VECTORS
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.strip_prefix('"')?.strip_suffix('"')
        })
        .collect();

    let mut scene = Scene::default();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    let mut index = 0;
    while index < rows.len() {
        let row = rows[index];
        index += 1;
        let (op, mut t) = split(row);
        *counts.entry(op).or_default() += 1;
        if op == "d" {
            assert_eq!(scene.diff(), row, "row {index}");
            continue;
        }
        if scene.setup(op, &mut t) || scene.ally_row(op, &mut t) || scene.operation(op, &mut t) {
            continue;
        }
        // Calls record the questions they make before their answer.
        let mut tape = VecDeque::new();
        let mut expected = row;
        let last = match op {
            "target" => Some("targeted"),
            "allies" => Some("allied"),
            "madest" => Some("madested"),
            _ => None,
        };
        if let Some(last) = last {
            while !rows[index].starts_with(last) {
                tape.push_back(rows[index].to_owned());
                index += 1;
            }
            expected = rows[index];
            index += 1;
        }
        let got = scene
            .query(row, &mut tape)
            .unwrap_or_else(|| panic!("unknown row {index}: {row}"));
        assert_eq!(got, expected, "row {index}: {row}");
        assert!(tape.is_empty(), "row {index}: unasked {tape:?}");
    }
    // The vectors must keep exercising every operation.
    for op in [
        "goal",
        "move",
        "rmgoal",
        "rmpos",
        "rmunit",
        "setlayer",
        "uplayer",
        "airgoal",
        "snap",
        "snapgoal",
        "goalpos",
        "adjust",
        "adjustgroup",
        "possible",
        "land",
        "target",
        "tighten",
        "allies",
        "madest",
    ] {
        assert!(counts.get(op).copied().unwrap_or(0) >= 30, "{op}");
    }
}
