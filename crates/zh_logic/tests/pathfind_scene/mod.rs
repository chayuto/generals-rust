//! The scene the pathfinder golden suites rebuild from their rows.
//! Each suite uses its own part of it.
#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, ICoord2D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real};
use zh_common::xfer::XferCrc;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::{
    AdjustUnit, CellIdx, Footprint, GoalObject, HierarchicalSearch, InfoIdx, NeighborSearch, Path,
    PathAgent, PathUnit, PathUnits, PathWorld, PathfindCellList, PathfindLayerEnum, Pathfinder,
    SearchPathCost, SearchUnit, WallSurface,
};
use zh_logic::terrain::{nearest_layer, Bridge, BridgeGeometry, ElevatedLayers, TerrainLogic};

pub const INFO_COUNT: usize = 30_000;

// ---- tokens -------------------------------------------------------------------

pub struct Tokens<'a>(pub std::str::SplitAsciiWhitespace<'a>);

impl<'a> Tokens<'a> {
    pub fn text(&mut self) -> &'a str {
        self.0.next().expect("token")
    }
    pub fn int(&mut self) -> Int {
        self.text().parse().expect("number")
    }
    pub fn id(&mut self) -> ObjectId {
        ObjectId(self.int())
    }
    pub fn flag(&mut self) -> bool {
        self.int() != 0
    }
    pub fn real(&mut self) -> Real {
        Real::from_bits(u32::from_str_radix(self.text(), 16).expect("bits"))
    }
    pub fn point(&mut self) -> Coord3D {
        Coord3D::new(self.real(), self.real(), self.real())
    }
    pub fn at(&mut self) -> ICoord2D {
        ICoord2D::new(self.int(), self.int())
    }
    pub fn pair(&mut self) -> Option<ICoord2D> {
        let (x, y) = self.0.next()?.split_once(':')?;
        Some(ICoord2D::new(x.parse().ok()?, y.parse().ok()?))
    }
    pub fn layer(&mut self) -> PathfindLayerEnum {
        PathfindLayerEnum::from_i32(self.int()).expect("layer")
    }
    pub fn rest(&mut self) -> Vec<&'a str> {
        self.0.by_ref().collect()
    }
}

pub fn f(v: Real) -> String {
    format!("{:08x}", v.to_bits())
}

pub fn c3(c: &Coord3D) -> String {
    format!("{} {} {}", f(c.x), f(c.y), f(c.z))
}

pub struct Fnv(u64);

impl Fnv {
    pub fn new() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
    pub fn add(&mut self, v: i64) {
        self.0 = (self.0 ^ v as u64).wrapping_mul(0x100_0000_01b3);
    }
    pub fn hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

// ---- terrain and units ----------------------------------------------------------

pub const HEIGHTS: [Real; 6] = [0.0, 3.0, 9.5, 10.0, 14.0, 25.0];

#[derive(Default)]
pub struct Tables {
    pub w: Int,
    pub h: Int,
    pub cliff: Vec<u8>,
    pub water: Vec<u8>,
    pub height: Vec<u8>,
    pub wall_height: Real,
    pub wall: WallSurface,
    /// Answers the layer questions of maps with bridges, whose ground is flat.
    pub bridged: Option<TerrainLogic>,
}

impl Tables {
    pub fn at(&self, x: Real, y: Real) -> usize {
        let i = real_to_int_floor(x / 10.0).clamp(0, self.w + 1);
        let j = real_to_int_floor(y / 10.0).clamp(0, self.h + 1);
        (j * (self.w + 2) + i) as usize
    }
}

impl ThingTerrainAccess for Tables {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(normal) = normal {
            *normal = Coord3D::new(0.0, 0.0, 1.0);
        }
        HEIGHTS[usize::from(self.height[self.at(x, y)] - b'0')]
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
        if let Some(bridged) = &self.bridged {
            return bridged.get_layer_for_destination(pos);
        }
        nearest_layer(
            pos,
            self.get_ground_height(pos.x, pos.y, None),
            self.wall_height,
            || self.wall.is_point_on_wall(pos) != 0,
            &[],
        )
    }
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        let above = (layer as Int - PathfindLayerEnum::Ground as Int) as Real * 7.5;
        self.get_ground_height(x, y, normal) + above
    }
}

#[derive(Debug, Clone)]
pub struct UnitInfo {
    pub agent: PathAgent,
    pub crushable: u8,
    pub human: bool,
    pub dozer: bool,
    pub vehicle: bool,
}

#[derive(Default)]
pub struct Units {
    pub info: BTreeMap<Int, UnitInfo>,
    pub relations: BTreeMap<(Int, Int), (bool, bool)>,
    pub friendly: BTreeSet<(Int, Int)>,
    pub goal_objects: BTreeMap<Int, GoalObject>,
    pub idle: BTreeSet<Int>,
}

impl PathUnits for Units {
    fn unit(&self, id: ObjectId) -> Option<PathUnit> {
        self.info.get(&id.0).map(|u| PathUnit {
            has_ai: true,
            crushable_level: u.crushable,
            infantry: u.agent.infantry,
        })
    }
    fn is_ally(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.relations[&(agent.0, other.0)].0
    }
    fn can_crush_or_squish(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.relations[&(agent.0, other.0)].1
    }
    fn is_non_enemy_object(&self, agent: ObjectId, other: ObjectId) -> bool {
        self.friendly.contains(&(agent.0, other.0))
    }
    fn goal_object(&self, id: ObjectId) -> Option<GoalObject> {
        self.goal_objects.get(&id.0).copied()
    }
    fn is_idle(&self, id: ObjectId) -> bool {
        self.idle.contains(&id.0)
    }
}

pub struct Walls<'a>(&'a BTreeMap<Int, Footprint>);

impl PathWorld for Walls<'_> {
    fn find_object(&self, id: ObjectId) -> Option<Footprint> {
        self.0.get(&id.0).cloned()
    }
}

// ---- scene ----------------------------------------------------------------------

#[derive(Default)]
pub struct Scene {
    pub pf: Pathfinder,
    pub terrain: Tables,
    pub extent: Region3D,
    pub wall_height: Real,
    pub footprints: BTreeMap<Int, Footprint>,
    pub units: Units,
    pub cells: Vec<CellIdx>,
    pub keys: BTreeMap<usize, i64>,
    /// Cells holding records to leave few free.
    pub hoard: Vec<CellIdx>,
    /// Unit id to the search its queued request runs, and whom it queues.
    pub plans: BTreeMap<Int, (Coord3D, Coord3D, ObjectId, Int)>,
    /// Whether this frame's `processPathfindQueue` takes requests.
    pub serving: bool,
}

pub fn read_footprint(t: &mut Tokens) -> Footprint {
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

/// The unit a call searches for, as the row names it.
pub struct Mover {
    pub unit: SearchUnit,
}

impl Scene {
    pub fn mover(&mut self, t: &mut Tokens) -> Mover {
        let id = t.int();
        let surfaces = t.int();
        let downhill_only = t.flag();
        let can_path_through_units = t.flag();
        let layer = t.layer();
        self.pf.ignore_obstacle_id = t.id();
        let info = &self.units.info[&id];
        let mut agent = info.agent.clone();
        agent.layer = layer;
        Mover {
            unit: SearchUnit {
                agent,
                surfaces,
                downhill_only,
                human: info.human,
                can_path_through_units,
                dozer: info.dozer,
                vehicle: info.vehicle,
            },
        }
    }

    pub fn done(&mut self) {
        self.pf.ignore_obstacle_id = ObjectId::INVALID;
    }

    pub fn index_cells(&mut self) {
        let ground = PathfindLayerEnum::Ground;
        for layer in [ground, PathfindLayerEnum::Wall] {
            for x in 0..self.terrain.w {
                for y in 0..self.terrain.h {
                    let Some(idx) = self.pf.get_cell(layer, x, y) else {
                        continue;
                    };
                    if layer != ground && Some(idx) == self.pf.get_cell(ground, x, y) {
                        continue;
                    }
                    let key = i64::from(layer as Int * 1_000_000 + x * 1000 + y);
                    self.cells.push(idx);
                    self.keys.insert(idx.0, key);
                }
            }
        }
    }

    pub fn key(&self, cell: Option<CellIdx>) -> i64 {
        cell.map_or(-1, |c| self.keys.get(&c.0).copied().unwrap_or(-2))
    }

    /// The records some call has taken: all but those still in order at the end
    /// of the free list.
    pub fn used_records(&self) -> usize {
        let pool = self.pf.map().pool();
        let mut list = Vec::new();
        let mut next = pool.first_free();
        while let Some(i) = next.filter(|_| list.len() <= INFO_COUNT) {
            list.push(i.0);
            next = pool.get(i).expect("free record").next_free;
        }
        if list.last() != Some(&(INFO_COUNT - 1)) {
            return INFO_COUNT;
        }
        let mut k = list.len() - 1;
        while k > 0 && list[k - 1] + 1 == list[k] {
            k -= 1;
        }
        list[k]
    }

    pub fn record(&self, i: usize) -> [i64; 14] {
        let pool = self.pf.map().pool();
        let v = pool.get(InfoIdx(i)).expect("record");
        let parent = v
            .path_parent
            .filter(|_| !v.is_free)
            .map_or(-1, |p| self.key(pool.get(p).expect("parent").cell));
        let link = |l: Option<InfoIdx>| l.map_or(-1, |l| l.0 as i64);
        let flags = i64::from(v.is_free)
            + 2 * i64::from(v.blocked_by_ally)
            + 4 * i64::from(v.obstacle_is_fence)
            + 8 * i64::from(v.obstacle_is_transparent)
            + 16 * i64::from(v.open)
            + 32 * i64::from(v.closed);
        [
            i as i64,
            self.key(v.cell),
            parent,
            link(v.next_open),
            link(v.prev_open),
            i64::from(v.total_cost),
            i64::from(v.cost_so_far),
            i64::from(v.pos.x),
            i64::from(v.pos.y),
            i64::from(v.goal_unit_id.0),
            i64::from(v.pos_unit_id.0),
            i64::from(v.goal_aircraft_id.0),
            i64::from(v.obstacle_id.0),
            flags,
        ]
    }

    pub fn records(&self) -> Vec<String> {
        (0..self.used_records())
            .map(|i| self.record(i).map(|v| v.to_string()).join(","))
            .collect()
    }

    /// The digests the harness writes after every call.
    pub fn state(&self) -> String {
        let mut pool_hash = Fnv::new();
        for i in 0..self.used_records() {
            for v in self.record(i) {
                pool_hash.add(v);
            }
        }
        let pool = self.pf.map().pool();
        let mut free = Fnv::new();
        let mut next = pool.first_free();
        let mut walked = 0;
        while let Some(i) = next.filter(|_| walked <= INFO_COUNT) {
            free.add(i.0 as i64);
            next = pool.get(i).expect("free record").next_free;
            walked += 1;
        }
        let mode = self.pf.map().mode();
        let mut cells = Fnv::new();
        for &idx in &self.cells {
            let grid = self.pf.map().grid();
            let c = grid.cell(idx);
            cells.add(c.info_idx.map_or(-1, |i| i.0 as i64));
            cells.add(c.cell_type as i64);
            cells.add(c.flags as i64);
            cells.add(c.connects_to_layer as i64);
            cells.add(i64::from(c.pinched));
            cells.add(i64::from(c.aircraft_goal));
            cells.add(i64::from(c.obstacle_id(grid.pool(), mode).0));
            let blocked = (c.info_idx.is_some() || mode.use_fixed_pathfinding)
                && grid.is_blocked_by_ally(idx) == Some(true);
            cells.add(i64::from(blocked));
        }
        let mut pass = String::new();
        for x in (0..self.terrain.w).step_by(10) {
            for y in (0..self.terrain.h).step_by(10) {
                pass.push(if self.pf.zones().is_passable(x, y) != 0 {
                    '1'
                } else {
                    '0'
                });
            }
        }
        format!(
            "{} {} {} {} {} {} {} {}",
            u8::from(mode.use_fixed_pathfinding),
            u8::from(mode.force_clean_cells),
            u8::from(self.pf.is_tunneling),
            self.pf.cumulative_cells_allocated,
            pool_hash.hex(),
            free.hex(),
            cells.hex(),
            pass
        )
    }

    pub fn path_dump(path: Option<&Path>) -> String {
        let Some(path) = path else {
            return "null".to_string();
        };
        let mut nodes = Vec::new();
        let mut cursor = path.first_node();
        while let Some(i) = cursor {
            nodes.push(i);
            cursor = path.node(i).next();
        }
        let mut out = format!("{} {}", nodes.len(), path.blocked_by_ally());
        for &i in &nodes {
            let node = path.node(i);
            let next = node
                .next_optimized()
                .and_then(|n| nodes.iter().position(|&m| m == n))
                .map_or(-1, |p| p as Int);
            out += &format!(
                " / {} {} {} {}",
                c3(node.position()),
                node.layer() as Int,
                u8::from(node.can_optimize()),
                next
            );
        }
        out
    }

    pub fn open_list_dump(&self) -> String {
        let pool = self.pf.map().pool();
        let grid = self.pf.map().grid();
        let mut out = String::new();
        let mut cursor = self.pf.open_list.get_head();
        let mut walked = 0;
        while let Some(c) = cursor.filter(|_| walked < 4000) {
            let info = pool
                .get(grid.cell(c).info_idx.expect("listed cell"))
                .expect("record");
            out += &format!(
                " {}:{}:{}",
                self.key(Some(c)),
                info.total_cost,
                info.cost_so_far
            );
            cursor = info
                .next_open
                .and_then(|n| pool.get(n))
                .and_then(|n| n.cell);
            walked += 1;
        }
        out
    }

    pub fn cell(&self, layer: PathfindLayerEnum, at: ICoord2D) -> CellIdx {
        self.pf.get_cell(layer, at.x, at.y).expect("cell")
    }

    /// The map and its occupants.
    pub fn setup(&mut self, op: &str, t: &mut Tokens) -> bool {
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
                self.terrain.height = t.text().bytes().collect();
                self.terrain.wall_height = self.wall_height;
            }
            "wallpiece" => self.pf.add_wall_piece(t.id()),
            "plan" => {
                let id = t.int();
                let (from, to) = (t.point(), t.point());
                self.plans.insert(id, (from, to, t.id(), t.int()));
            }
            "fill" => assert!(self.pf.queue_for_path(t.id())),
            "frame" => {
                let region = Region3D {
                    lo: t.point(),
                    hi: t.point(),
                };
                let frame = t.int() as u32;
                self.serving = self.pf.begin_path_queue(&region, frame);
            }
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
                self.pf.new_map(
                    &self.terrain,
                    &self.extent,
                    self.wall_height,
                    &all,
                    &Walls(&self.footprints),
                    100,
                );
                self.terrain.wall = self.pf.wall_surface();
                if let Some(bridged) = &mut self.terrain.bridged {
                    bridged.set_elevated_layers(Box::new(self.pf.wall_surface()));
                }
                self.index_cells();
            }
            "unit" => {
                let id = t.int();
                let ignored_obstacle = t.id();
                let aircraft_adjusts = t.flag();
                let crusher_level = t.int() as u8;
                let crushable = t.int() as u8;
                let radius = t.real();
                let layer = t.layer();
                let infantry = t.flag();
                let (human, dozer, vehicle) = (t.flag(), t.flag(), t.flag());
                let agent = PathAgent {
                    id: ObjectId(id),
                    has_ai: true,
                    ignored_obstacle,
                    aircraft_adjusts,
                    crusher_level,
                    radius,
                    layer,
                    infantry,
                };
                self.units.info.insert(
                    id,
                    UnitInfo {
                        agent,
                        crushable,
                        human,
                        dozer,
                        vehicle,
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
            "friendly" => {
                let agent = t.int();
                for other in t.rest() {
                    self.units
                        .friendly
                        .insert((agent, other.parse().expect("id")));
                }
            }
            "goalobj" => {
                let id = t.int();
                let object = GoalObject {
                    position: t.point(),
                    layer: t.layer(),
                    airfield: t.flag(),
                };
                self.units.goal_objects.insert(id, object);
            }
            "idle" => {
                self.units.idle.clear();
                for id in t.rest() {
                    self.units.idle.insert(id.parse().expect("id"));
                }
            }
            "goal" | "pos" | "air" => {
                let layer = t.layer();
                let at = t.at();
                let id = t.id();
                let idx = self.cell(layer, at);
                let grid = self.pf.map_mut().grid_mut();
                match op {
                    "goal" => grid.set_goal_unit(idx, id, at),
                    "pos" => grid.set_pos_unit(idx, id, at),
                    _ => grid.set_goal_aircraft(idx, id, at),
                }
            }
            "connect" => {
                let at = t.at();
                let top = self.cell(PathfindLayerEnum::Wall, at);
                let ground = self.cell(PathfindLayerEnum::Ground, at);
                let grid = self.pf.map_mut().grid_mut();
                grid.set_connect_layer(ground, PathfindLayerEnum::Wall);
                grid.set_connect_layer(top, PathfindLayerEnum::Ground);
            }
            "begin" => {
                let region = Region3D {
                    lo: t.point(),
                    hi: t.point(),
                };
                let frame = t.int() as u32;
                self.pf.begin_path_queue(&region, frame);
            }
            "allpass" => self.pf.zones_mut().set_all_passable(),
            "somepass" => {
                self.pf.zones_mut().clear_passable_flags();
                while let Some(at) = t.pair() {
                    self.pf.zones_mut().set_passable(at.x, at.y, 1);
                }
            }
            "scenario" => *self = Scene::default(),
            "end" => {}
            _ => return false,
        }
        true
    }

    /// Makes the harness's call; the answer and the state after it.
    pub fn call(&mut self, op: &str, t: &mut Tokens) -> String {
        match op {
            "find" => {
                let m = self.mover(t);
                let (from, to) = (t.point(), t.point());
                let path =
                    self.pf
                        .internal_find_path(&self.terrain, &self.units, &m.unit, &from, &to);
                let out = format!("{} | {}", Self::path_dump(path.as_ref()), self.state());
                self.done();
                out
            }
            "cost" => {
                let m = self.mover(t);
                let (from, to) = (t.point(), t.point());
                self.pf.is_tunneling = t.flag();
                let crusher = m.unit.agent.crusher_level > 0;
                let cost =
                    self.pf
                        .check_path_cost(&self.terrain, crusher, m.unit.surfaces, &from, &to);
                let out = format!("{} | {}", cost, self.state());
                self.pf.is_tunneling = false;
                self.done();
                out
            }
            "examine" => self.examine(t),
            "adjust" => {
                let m = self.mover(t);
                let position = t.point();
                let (mut dest, group) = (t.point(), t.point());
                let unit = AdjustUnit {
                    agent: m.unit.agent.clone(),
                    position,
                    surfaces: m.unit.surfaces,
                    aircraft: false,
                    projectile: false,
                    human: m.unit.human,
                };
                let mut cost = SearchPathCost {
                    terrain: &self.terrain,
                };
                let ok = self.pf.adjust_destination(
                    &self.terrain,
                    &self.units,
                    &mut cost,
                    &unit,
                    &mut dest,
                    Some(&group),
                );
                let out = format!("{} {} | {}", u8::from(ok), c3(&dest), self.state());
                self.done();
                out
            }
            "clip" => {
                let (mut a, mut b) = (t.point(), t.point());
                self.pf.clip(&mut a, &mut b);
                format!("{} {}", c3(&a), c3(&b))
            }
            "chain" => {
                let mut words: Vec<&str> = t.rest();
                let tail = words.split_off(words.len() - 4);
                let real = |s: &str| Real::from_bits(u32::from_str_radix(s, 16).expect("bits"));
                let from = Coord3D::new(real(tail[0]), real(tail[1]), real(tail[2]));
                let center = tail[3] == "1";
                let cells: Vec<(CellIdx, ICoord2D)> = words
                    .iter()
                    .map(|w| {
                        let (x, y) = w.split_once(':').expect("cell");
                        let at = ICoord2D::new(x.parse().expect("x"), y.parse().expect("y"));
                        (self.cell(PathfindLayerEnum::Ground, at), at)
                    })
                    .collect();
                let grid = self.pf.map_mut().grid_mut();
                for &(idx, at) in &cells {
                    grid.allocate_info(idx, at);
                }
                for pair in cells.windows(2) {
                    grid.set_parent_cell(pair[0].0, pair[1].0);
                }
                grid.release_info(cells.last().expect("chain").0);
                let mut path = Path::new();
                self.pf
                    .prepend_cells(&self.terrain, &mut path, &from, cells[0].0, center);
                format!("{} | {}", Self::path_dump(Some(&path)), self.state())
            }
            "lists" => {
                let open = t.flag();
                let (bare_at, held_at) = (t.at(), t.at());
                let bare = self.cell(PathfindLayerEnum::Ground, bare_at);
                let held = self.cell(PathfindLayerEnum::Ground, held_at);
                let mut other = PathfindCellList::new();
                let grid = self.pf.map_mut().grid_mut();
                grid.allocate_info(held, held_at);
                if open {
                    grid.put_on_closed_list(held, &mut other);
                    self.pf.closed_list = other;
                    self.pf.open_list.reset(Some(bare));
                } else {
                    grid.put_on_sorted_open_list(held, &mut other);
                    self.pf.open_list = other;
                    self.pf.closed_list.reset(Some(bare));
                }
                self.pf.clean_open_and_closed_lists();
                self.pf.map_mut().grid_mut().release_info(held);
                self.state()
            }
            "cap" => {
                let mut cells = Vec::new();
                for x in 0..self.terrain.w {
                    for y in 0..self.terrain.h {
                        let at = ICoord2D::new(x, y);
                        let idx = self.cell(PathfindLayerEnum::Ground, at);
                        if cells.len() < 5100 && self.pf.map().grid().cell(idx).info_idx.is_none() {
                            cells.push((idx, at));
                        }
                    }
                }
                let mut list = PathfindCellList::new();
                let grid = self.pf.map_mut().grid_mut();
                for (i, &(idx, at)) in cells.iter().enumerate() {
                    grid.allocate_info(idx, at);
                    grid.set_total_cost(idx, if i < 3 { 1 } else { 7 });
                    grid.put_on_sorted_open_list(idx, &mut list);
                }
                let mut order = Vec::new();
                let mut cursor = list.get_head();
                while let Some(c) = cursor {
                    order.push(c);
                    let pool = self.pf.map().pool();
                    let info = self.pf.map().grid().cell(c).info_idx.expect("listed");
                    cursor = pool
                        .get(info)
                        .and_then(|i| i.next_open)
                        .and_then(|n| pool.get(n))
                        .and_then(|n| n.cell);
                }
                let mut hash = Fnv::new();
                for &c in &order {
                    hash.add(self.key(Some(c)));
                }
                self.pf.map_mut().grid_mut().release_open_list(&mut list);
                format!(
                    "{} {} {} | {}",
                    cells.len(),
                    order.len(),
                    hash.hex(),
                    self.state()
                )
            }
            "grand" => {
                let at: [ICoord2D; 4] = std::array::from_fn(|_| t.at());
                let cells = at.map(|p| self.cell(PathfindLayerEnum::Ground, p));
                let [grand, parent, cell, taker] = cells;
                let cost = t.int() as u32;
                let grid = self.pf.map_mut().grid_mut();
                grid.allocate_info(grand, at[0]);
                grid.allocate_info(parent, at[1]);
                grid.allocate_info(cell, at[2]);
                grid.set_parent_cell_hierarchical(parent, grand);
                grid.set_cost_so_far(parent, cost);
                let held = grid.cost_so_far(cell, Some(parent));
                grid.release_info(grand);
                let released = grid.cost_so_far(cell, Some(parent));
                grid.allocate_info(taker, at[3]);
                let reused = grid.cost_so_far(cell, Some(parent));
                let digests = self.state();
                let grid = self.pf.map_mut().grid_mut();
                grid.clear_parent_cell(parent);
                for idx in [taker, cell, parent] {
                    grid.release_info(idx);
                }
                format!("{held} {released} {reused} {digests}")
            }
            "cut" => {
                let at: [ICoord2D; 4] = std::array::from_fn(|_| t.at());
                let cells = at.map(|p| self.cell(PathfindLayerEnum::Ground, p));
                let [first, second, gone, added] = cells;
                let mut list = PathfindCellList::new();
                let grid = self.pf.map_mut().grid_mut();
                for (idx, p) in cells.into_iter().zip(at) {
                    grid.allocate_info(idx, p);
                }
                grid.set_total_cost(first, 10);
                grid.set_total_cost(second, 20);
                grid.set_total_cost(added, 30);
                grid.put_on_sorted_open_list(first, &mut list);
                grid.put_on_sorted_open_list(second, &mut list);
                let stale = grid.cell(gone).info_idx.expect("record");
                grid.release_info(gone);
                let link = grid.cell(second).info_idx.expect("record");
                grid.pool_mut().get_mut(link).expect("record").next_open = Some(stale);
                grid.put_on_sorted_open_list(added, &mut list);
                let mut out = String::new();
                let mut cursor = list.get_head();
                let mut walked = 0;
                while let Some(c) = cursor.filter(|_| walked < 8) {
                    out += &format!(" {}", self.key(Some(c)));
                    let grid = self.pf.map().grid();
                    let info = grid.cell(c).info_idx.expect("listed");
                    cursor = grid
                        .pool()
                        .get(info)
                        .and_then(|i| i.next_open)
                        .and_then(|n| grid.pool().get(n))
                        .and_then(|n| n.cell);
                    walked += 1;
                }
                let digests = self.state();
                let grid = self.pf.map_mut().grid_mut();
                grid.release_open_list(&mut list);
                grid.release_info(gone);
                format!("{} | {digests}", out.trim_start())
            }
            "bridge" => {
                let id = t.id();
                let [from, to, fl, fr, tl, tr] = std::array::from_fn(|_| t.point());
                let geometry = BridgeGeometry::from_corners(from, to, fl, fr, tl, tr);
                let bridged = self.terrain.bridged.get_or_insert_with(TerrainLogic::new);
                bridged.add_bridge(Bridge::new(geometry, "S5NoRoad", id), &mut self.pf);
                let layer = bridged.bridges().iter().next().expect("bridge").layer;
                (layer as Int).to_string()
            }
            "hier" => {
                let m = self.mover(t);
                let (human, crusher, closest_ok) = (t.flag(), t.flag(), t.flag());
                let (from, to) = (t.point(), t.point());
                let search = HierarchicalSearch {
                    human,
                    surfaces: m.unit.surfaces,
                    crusher,
                    closest_ok,
                };
                let path = self
                    .pf
                    .find_hierarchical_path(&self.terrain, &search, &from, &to);
                let out = format!("{} | {}", Self::path_dump(path.as_ref()), self.state());
                self.done();
                out
            }
            "path" => {
                let m = self.mover(t);
                let (from, to) = (t.point(), t.point());
                let path = self
                    .pf
                    .find_path(&self.terrain, &self.units, &m.unit, &from, &to);
                let out = format!("{} | {}", Self::path_dump(path.as_ref()), self.state());
                self.done();
                out
            }
            "exists" => {
                let m = self.mover(t);
                let (from, to) = (t.point(), t.point());
                let ignore = t.id();
                let found = self.pf.slow_does_path_exist(
                    &self.terrain,
                    &self.units,
                    &m.unit,
                    &from,
                    &to,
                    ignore,
                );
                let out = format!(
                    "{} {} | {}",
                    u8::from(found),
                    self.pf.ignore_obstacle_id.0,
                    self.state()
                );
                self.done();
                out
            }
            "ground" => {
                let diameter = t.int();
                let crusher = t.flag();
                let (from, to) = (t.point(), t.point());
                let path = self.pf.find_ground_path(
                    &self.terrain,
                    &self.units,
                    &from,
                    &to,
                    diameter,
                    crusher,
                );
                format!("{} | {}", Self::path_dump(path.as_ref()), self.state())
            }
            "queue" => {
                let ok = self.pf.queue_for_path(t.id());
                format!("{} {}", u8::from(ok), self.crc())
            }
            "serve" => self.serve(t.int()),
            "frameend" => {
                // What is left has no AI, unless the frame's cells are spent.
                while let Some(id) = self.next_request() {
                    assert!(!self.plans.contains_key(&id.0), "unit {id:?} left unserved");
                    self.pf.finish_path_request();
                }
                format!("{} {}", self.state(), self.crc())
            }
            "hoard" => {
                let left = t.int() as u32;
                for x in (0..self.terrain.w).rev().filter(|x| !(36..50).contains(x)) {
                    for y in (0..self.terrain.h).rev() {
                        if self.pf.map().pool().free_count() <= left {
                            break;
                        }
                        let at = ICoord2D::new(x, y);
                        let idx = self.cell(PathfindLayerEnum::Ground, at);
                        if self.pf.map().grid().cell(idx).info_idx.is_none() {
                            self.pf.map_mut().grid_mut().allocate_info(idx, at);
                            self.hoard.push(idx);
                        }
                    }
                }
                format!("{} {}", self.pf.map().pool().free_count(), self.state())
            }
            "unhoard" => {
                for idx in std::mem::take(&mut self.hoard) {
                    self.pf.map_mut().grid_mut().release_info(idx);
                }
                self.state()
            }
            "retail" => {
                self.pf.map_mut().mode_mut().use_fixed_pathfinding = false;
                self.state()
            }
            "setup" => self.state(),
            "records" => self.records().join(" "),
            other => panic!("unknown row {other}"),
        }
    }

    pub fn crc(&self) -> u32 {
        let mut xfer = XferCrc::new();
        self.pf.crc(&mut xfer).expect("crc");
        xfer.get_crc()
    }

    pub fn next_request(&mut self) -> Option<ObjectId> {
        if !self.serving {
            return None;
        }
        self.pf.next_path_request()
    }

    /// The next queued unit runs its scripted search, as `doPathfind` did.
    pub fn serve(&mut self, expected: Int) -> String {
        let id = loop {
            let id = self.next_request().expect("a request to serve");
            if self.plans.contains_key(&id.0) {
                break id.0;
            }
            self.pf.finish_path_request();
        };
        assert_eq!(id, expected, "served out of order");
        let before = self.pf.cumulative_cells_allocated;
        let (from, to, again, surfaces) = self.plans[&id];
        let info = &self.units.info[&id];
        let unit = SearchUnit {
            agent: info.agent.clone(),
            surfaces,
            downhill_only: false,
            human: info.human,
            can_path_through_units: false,
            dozer: info.dozer,
            vehicle: info.vehicle,
        };
        let path = self
            .pf
            .find_path(&self.terrain, &self.units, &unit, &from, &to);
        let mut queued = "-".to_string();
        if again != ObjectId::INVALID {
            queued = u8::from(self.pf.queue_for_path(again)).to_string();
            self.plans.get_mut(&id).expect("plan").2 = ObjectId::INVALID;
        }
        let out = format!(
            "{before} {} {queued} {} {}",
            Self::path_dump(path.as_ref()),
            self.pf.cumulative_cells_allocated,
            self.crc()
        );
        self.pf.finish_path_request();
        out
    }

    pub fn examine(&mut self, t: &mut Tokens) -> String {
        let mut m = self.mover(t);
        let (at, goal_at) = (t.at(), t.at());
        let (tunnel, human, center_in_cell, no_goal) = (t.flag(), t.flag(), t.flag(), t.flag());
        let radius = t.int();
        let attack_distance = t.int();
        let start = t.at();
        m.unit.human = human;
        let parent = self.cell(m.unit.agent.layer, at);
        let goal = self.cell(PathfindLayerEnum::Ground, goal_at);
        let mut open = PathfindCellList::new();
        let mut closed = PathfindCellList::new();
        let fixed = self.pf.map().mode().use_fixed_pathfinding;
        let grid = self.pf.map_mut().grid_mut();
        grid.allocate_info(goal, goal_at);
        grid.allocate_info(parent, at);
        grid.start_pathfind(parent, Some(goal));
        if fixed {
            grid.put_on_sorted_open_list(parent, &mut open);
        } else {
            open.reset(Some(parent));
        }
        grid.remove_from_open_list(parent, &mut open);
        grid.put_on_closed_list(parent, &mut closed);
        self.pf.open_list = open;
        self.pf.closed_list = closed;
        self.pf.is_tunneling = tunnel;
        let search = NeighborSearch {
            unit: &m.unit,
            goal: (!no_goal).then_some(goal),
            center_in_cell,
            radius,
            start,
            attack_distance,
        };
        let count = self
            .pf
            .examine_neighboring_cells(&self.terrain, &self.units, parent, &search);
        let opened = self.open_list_dump();
        let tunnel_after = u8::from(self.pf.is_tunneling);
        self.pf.is_tunneling = false;
        self.pf.clean_open_and_closed_lists();
        let grid = self.pf.map_mut().grid_mut();
        grid.release_info(parent);
        grid.release_info(goal);
        let out = format!("{count} {tunnel_after}{opened} | {}", self.state());
        self.done();
        out
    }
}

pub fn rows(vectors: &'static str) -> Vec<&'static str> {
    vectors
        .lines()
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.strip_prefix('"')?.strip_suffix('"')
        })
        .collect()
}
