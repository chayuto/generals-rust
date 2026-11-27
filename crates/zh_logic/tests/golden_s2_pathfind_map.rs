//! Slice S2, the pathfind map, against the reference library
//! (`tests/cpp_harness/gen_s2_pathfind_map.cpp`).

use std::collections::BTreeMap;

use zh_common::math::gamemath::real_to_int_floor;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord2D, Coord3D, ICoord2D, Matrix3D, Region2D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real};
use zh_common::xfer::XferCrc;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::{Footprint, PathWorld, PathfindLayerEnum, Pathfinder};
use zh_logic::terrain::{
    highest_layer, line_in_region, nearest_layer, Bridge, BridgeGeometry, BridgeWorld, DeckObject,
    ElevatedLayers, TerrainLogic,
};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S2_PathfindMap.json");

// ---- tokens -------------------------------------------------------------------

struct Tokens<'a>(std::str::SplitAsciiWhitespace<'a>);

impl<'a> Tokens<'a> {
    fn text(&mut self) -> &'a str {
        self.0.next().expect("token")
    }
    fn int(&mut self) -> i64 {
        self.text().parse().expect("number")
    }
    fn id(&mut self) -> ObjectId {
        ObjectId(self.int() as i32)
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
    fn region(&mut self) -> Region2D {
        Region2D {
            lo: Coord2D {
                x: self.real(),
                y: self.real(),
            },
            hi: Coord2D {
                x: self.real(),
                y: self.real(),
            },
        }
    }
    fn rest(&mut self) -> Vec<&'a str> {
        self.0.by_ref().collect()
    }
}

fn f(v: Real) -> String {
    format!("{:08x}", v.to_bits())
}

fn digit(b: bool) -> char {
    if b {
        '1'
    } else {
        '0'
    }
}

fn layer_of(raw: i64) -> PathfindLayerEnum {
    PathfindLayerEnum::from_i32(raw as Int).expect("layer")
}

// ---- terrain ------------------------------------------------------------------

/// The harness's tables: one entry per cell with a border of one.
#[derive(Default, Clone)]
struct Tables {
    w: i64,
    h: i64,
    cliff: Vec<u8>,
    water: Vec<u8>,
    ground: Vec<Real>,
}

impl Tables {
    fn at(&self, x: Real, y: Real) -> usize {
        let i = i64::from(real_to_int_floor(x / 10.0)).clamp(0, self.w + 1);
        let j = i64::from(real_to_int_floor(y / 10.0)).clamp(0, self.h + 1);
        (j * (self.w + 2) + i) as usize
    }
}

impl ThingTerrainAccess for Tables {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        self.ground[self.at(x, y)]
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
}

// ---- objects ------------------------------------------------------------------

#[derive(Default)]
struct Objects {
    footprints: BTreeMap<i32, Footprint>,
    /// Units on the wall layer or on a deck, in list order.
    units: Vec<(DeckObject, bool)>,
}

impl PathWorld for Objects {
    fn find_object(&self, id: ObjectId) -> Option<Footprint> {
        self.footprints.get(&id.0).cloned()
    }
    fn objects_on_wall_layer(&self) -> Vec<(ObjectId, Coord3D)> {
        self.units
            .iter()
            .filter(|(u, _)| u.layer == PathfindLayerEnum::Wall)
            .map(|(u, _)| (u.id, u.position))
            .collect()
    }
}

impl Objects {
    /// The ids among `ids` that were alive, as the reference reports them.
    fn alive(&self, ids: &[ObjectId]) -> Vec<String> {
        ids.iter()
            .filter(|id| self.units.iter().any(|(u, alive)| u.id == **id && *alive))
            .map(|id| id.0.to_string())
            .collect()
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
    let _layer = t.int();
    let bit = |b: i64| flags & b != 0;
    // The template's radii as parsed: `GeometryInfo::set` would give a
    // cylinder one radius, the INI fields need not.
    let mut geometry = GeometryInfo::new(geom_type, small, height, major, minor);
    geometry.set_minor_radius(minor);
    Footprint {
        id,
        position,
        orientation,
        geometry,
        fence_width,
        fence_x_offset,
        mine: bit(1),
        projectile: bit(2),
        bridge_tower: bit(4),
        defensive_wall: bit(8),
        blast_crater: bit(16),
        walk_on_top_of_wall: bit(32),
        structure: bit(64),
        see_through: bit(128),
        mobile: bit(256),
        rubble: bit(512),
        height_above_terrain,
    }
}

struct Damage<'a> {
    objects: &'a Objects,
    decks: &'a BTreeMap<i32, BodyDamageType>,
    fallen: Vec<ObjectId>,
}

impl BridgeWorld for Damage<'_> {
    fn bridge_damage_state(&self, id: ObjectId) -> Option<BodyDamageType> {
        Some(
            self.decks
                .get(&id.0)
                .copied()
                .unwrap_or(BodyDamageType::Pristine),
        )
    }
    fn objects(&self) -> Vec<DeckObject> {
        self.objects.units.iter().map(|(u, _)| *u).collect()
    }
    fn kill_by_fall(&mut self, id: ObjectId) {
        self.fallen.push(id);
    }
}

// ---- state digests ------------------------------------------------------------

#[derive(Clone, Copy)]
struct Fnv(u64);

impl Fnv {
    fn new() -> Self {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn add(&mut self, v: i64) {
        for byte in (v as i32).to_le_bytes() {
            self.0 ^= u64::from(byte);
            self.0 = self.0.wrapping_mul(0x100_0000_01b3);
        }
    }
}

fn state_fields(pf: &Pathfinder) -> Vec<(&'static str, String)> {
    let extent = pf.extent();
    let logical = pf.logical_extent();
    let map = pf.map();
    let zones = pf.zones();

    let mut meta = Fnv::new();
    for v in [extent.lo, extent.hi, logical.lo, logical.hi] {
        meta.add(v.x.into());
        meta.add(v.y.into());
    }
    meta.add(pf.is_map_ready().into());
    meta.add(zones.max_zone.into());
    meta.add(zones.next_frame_to_calculate_zones.into());
    meta.add(pf.wall_pieces().len() as i64);
    for id in pf.wall_pieces() {
        meta.add(id.0.into());
    }

    let [mut type_, mut pinched, mut zone, mut connect, mut layer, mut obstacle, mut obsflags, mut info] =
        [Fnv::new(); 8];
    for i in 0..=extent.hi.x {
        for j in 0..=extent.hi.y {
            let c = map.cell(i, j);
            type_.add(c.cell_type() as i64);
            pinched.add(c.pinched().into());
            zone.add(c.zone().into());
            connect.add(c.connect_layer() as i64);
            layer.add(c.layer() as i64);
            obstacle.add(c.obstacle_id(map.pool(), map.mode()).0.into());
            let fence = c.is_obstacle_fence(map.pool(), map.mode());
            let glass = c.is_obstacle_transparent(map.pool(), map.mode());
            obsflags.add(i64::from(fence) | (i64::from(glass) << 1));
            info.add(c.has_info().into());
        }
    }

    let mut lmeta = Fnv::new();
    let [mut ltype, mut lpinched, mut lzone, mut lconnect, mut llayer, mut lget] = [Fnv::new(); 6];
    for k in 0..=15 {
        let l = pf.layer(layer_of(k));
        let (x0, y0, w, h) = l.rect();
        let (start, end) = (l.start_cell_index(), l.end_cell_index());
        lmeta.add(l.is_unused().into());
        lmeta.add(l.is_destroyed().into());
        lmeta.add(l.zone().into());
        for v in [start.x, start.y, end.x, end.y, w, h] {
            lmeta.add(v.into());
        }
        if w > 0 {
            lmeta.add(x0.into());
            lmeta.add(y0.into());
        }
        for i in 0..w {
            for j in 0..h {
                let c = map.grid().cell(l.local(i, j));
                ltype.add(c.cell_type() as i64);
                lpinched.add(c.pinched().into());
                lzone.add(c.zone().into());
                lconnect.add(c.connect_layer() as i64);
                llayer.add(c.layer() as i64);
            }
        }
        if w > 0 {
            for i in x0 - 1..=x0 + w {
                for j in y0 - 1..=y0 + h {
                    lget.add(l.get_cell(map, i, j).is_some().into());
                }
            }
        }
    }

    // Ground, ground and cliff, ground and water, ground and rubble, water,
    // air, cliff, ground with cliff and water.
    let masks = [1u32, 5, 3, 17, 2, 8, 4, 7];
    let (mut equiv, mut block, mut bridges) = (Fnv::new(), Fnv::new(), Fnv::new());
    for mask in masks {
        for crusher in 0..2u8 {
            for i in 0..=extent.hi.x {
                for j in 0..=extent.hi.y {
                    let z = map.cell(i, j).zone();
                    equiv.add(zones.get_effective_zone(mask as _, crusher, z).into());
                    block.add(zones.get_block_zone(mask as _, crusher, i, j, map).into());
                }
            }
        }
    }
    for i in 0..=extent.hi.x {
        for j in 0..=extent.hi.y {
            equiv.add(
                zones
                    .get_effective_terrain_zone(map.cell(i, j).zone())
                    .into(),
            );
            bridges.add(zones.interacts_with_bridge(i, j).into());
        }
    }

    let mut crc = XferCrc::new();
    pf.crc(&mut crc).expect("crc");
    let hex = |h: Fnv| format!("{:016x}", h.0);
    vec![
        ("meta", hex(meta)),
        ("type", hex(type_)),
        ("pinched", hex(pinched)),
        ("connect", hex(connect)),
        ("layer", hex(layer)),
        ("obstacle", hex(obstacle)),
        ("obsflags", hex(obsflags)),
        ("info", hex(info)),
        ("lmeta", hex(lmeta)),
        ("ltype", hex(ltype)),
        ("lpinched", hex(lpinched)),
        ("lzone", hex(lzone)),
        ("lconnect", hex(lconnect)),
        ("llayer", hex(llayer)),
        ("lget", hex(lget)),
        ("zone", hex(zone)),
        ("equiv", hex(equiv)),
        ("block", hex(block)),
        ("bridges", hex(bridges)),
        ("crc", format!("{:08x}", crc.get_crc())),
    ]
}

fn picture(pf: &Pathfinder) -> String {
    let extent = pf.extent();
    let mut out = String::new();
    for i in 0..=extent.hi.x {
        for j in 0..=extent.hi.y {
            let c = pf.map().cell(i, j);
            let v = c.cell_type() as u8 + if c.pinched() { 10 } else { 0 };
            out.push((b'0' + v) as char);
        }
    }
    out
}

// ---- replay -------------------------------------------------------------------

#[derive(Default)]
struct Scene {
    pf: Pathfinder,
    terrain: TerrainLogic,
    tables: Tables,
    extent: Region3D,
    wall_height: Real,
    objects: Objects,
    order: Vec<i32>,
    decks: BTreeMap<i32, BodyDamageType>,
    shape: Option<BridgeGeometry>,
}

impl Scene {
    fn footprint(&self, id: ObjectId) -> Footprint {
        self.objects.footprints[&id.0].clone()
    }

    fn classify(&mut self, id: ObjectId, insert: bool, frame: u32) -> Vec<ObjectId> {
        let fp = self.footprint(id);
        self.pf
            .classify_object_footprint(&fp, insert, &self.objects, frame)
    }

    fn query(&mut self, t: &mut Tokens) -> (String, String) {
        let p = t.point();
        let r = t.region();
        let minor = t.real();
        assert_eq!(t.text(), "=");
        let want = t.rest().join(" ");
        let wall = self.pf.wall_surface();
        let on_wall = self.pf.is_point_on_wall(&p);
        assert_eq!(
            wall.is_point_on_wall(&p) != 0,
            on_wall,
            "wall surface at {p:?}"
        );
        self.terrain.set_elevated_layers(Box::new(wall));
        let ground = self.tables.get_ground_height(p.x, p.y, None);
        let bridges = self.terrain.bridges();
        let decks = bridges.decks_at(&p);
        let height = self.pf.wall_height();
        let mut got = format!(
            "{} {} {} {} {}",
            bridges.find_at(&p).map_or(0, |b| b.layer as Int),
            nearest_layer(&p, ground, height, || on_wall, &decks) as Int,
            highest_layer(&p, ground, height, || on_wall, &decks, false) as Int,
            highest_layer(&p, ground, height, || on_wall, &decks, true) as Int,
            Int::from(on_wall),
        );
        for b in bridges.iter() {
            let g = &b.geometry;
            let mut normal = Coord3D::default();
            let h = g.height(&p, Some(&mut normal));
            let interacts = |health| {
                self.terrain.object_interacts_with_bridge_layer(
                    &p,
                    minor,
                    PathfindLayerEnum::Ground,
                    b.layer,
                    health,
                )
            };
            got.push_str(&format!(
                " | {}{}{}{} {} {} {} {} {}{}{}",
                digit(g.is_point_on(&p)),
                digit(g.is_cell_on_end(&r)),
                digit(g.is_cell_on_side(&r)),
                digit(g.is_cell_entry_point(&r)),
                f(h),
                f(normal.x),
                f(normal.y),
                f(normal.z),
                digit(interacts(false)),
                digit(interacts(true)),
                digit(bridges.interacts_with_end(&p, minor, b.layer)),
            ));
        }
        (got, want)
    }

    /// Replays one row; returns `(got, want)` where the row states a result.
    fn replay(&mut self, row: &str) -> Option<(String, String)> {
        let mut t = Tokens(row.split_ascii_whitespace());
        let op = t.text();
        match op {
            "map" => {
                self.extent = Region3D::default();
                self.extent.hi = Coord3D::new(t.real(), t.real(), 100.0);
                self.wall_height = t.real();
                self.pf.reset(self.wall_height);
            }
            "terrain" => {
                self.tables.w = t.int();
                self.tables.h = t.int();
                self.tables.cliff = t.text().bytes().collect();
                self.tables.water = t.text().bytes().collect();
            }
            "heights" => {
                self.tables.ground = t
                    .rest()
                    .iter()
                    .flat_map(|run| {
                        let (n, bits) = run.split_once(':').expect("run");
                        let v = Real::from_bits(u32::from_str_radix(bits, 16).expect("bits"));
                        std::iter::repeat_n(v, n.parse().expect("count"))
                    })
                    .collect();
            }
            "obj" => {
                let fp = read_footprint(&mut t);
                self.objects.footprints.insert(fp.id.0, fp);
            }
            "order" => self.order = t.rest().iter().map(|s| s.parse().expect("id")).collect(),
            "newmap" => {
                let frame = t.int() as u32;
                let all: Vec<Footprint> = self
                    .order
                    .iter()
                    .map(|id| self.objects.footprints[id].clone())
                    .collect();
                self.pf.new_map(
                    &self.tables,
                    &self.extent,
                    self.wall_height,
                    &all,
                    &self.objects,
                    frame,
                );
            }
            "add" => {
                let id = t.id();
                let frame = t.int() as u32;
                assert!(self.classify(id, true, frame).is_empty());
            }
            "remove" => {
                let id = t.id();
                let frame = t.int() as u32;
                assert_eq!(t.text(), "=");
                let want = t.rest().join(" ");
                let fallers = self.classify(id, false, frame);
                return Some((self.objects.alive(&fallers).join(" "), want));
            }
            "wallon" | "walloff" => {
                let fp = self.footprint(t.id());
                self.pf
                    .classify_footprint_cells(&fp, op == "wallon", t.int() as u32);
            }
            "zones" => {
                let frame = t.int() as u32;
                self.pf.begin_path_queue(&self.extent, frame);
            }
            "recalc" => self.pf.force_map_recalculation(&self.tables, &self.objects),
            "bridgestate" => {
                let layer = layer_of(t.int());
                let repaired = t.flag();
                self.pf
                    .change_bridge_state(layer, repaired, &self.tables, t.int() as u32);
            }
            "decks" => {
                let id = t.id();
                let state = BodyDamageType::from_i32(t.int() as Int).expect("state");
                self.decks.insert(id.0, state);
            }
            "unit" => {
                let unit = DeckObject {
                    id: t.id(),
                    position: t.point(),
                    minor_radius: t.real(),
                    layer: layer_of(t.int()),
                };
                let alive = t.flag();
                self.objects.units.push((unit, alive));
            }
            "unitsgone" => self.objects.units.clear(),
            "damage" => {
                let frame = t.int() as u32;
                assert_eq!(t.text(), "=");
                let want = t.rest().join(" ");
                let mut world = Damage {
                    objects: &self.objects,
                    decks: &self.decks,
                    fallen: Vec::new(),
                };
                let mut changes = Vec::new();
                self.terrain
                    .bridges_mut()
                    .update_damage_states(&mut world, &mut |l, r| changes.push((l, r)));
                let fallen = world.fallen;
                for (layer, repaired) in changes {
                    self.pf
                        .change_bridge_state(layer, repaired, &self.tables, frame);
                }
                let mut got = vec!["fallen".to_string()];
                got.extend(self.objects.alive(&fallen));
                got.push(";".to_string());
                let bridges = self.terrain.bridges();
                for b in bridges.iter() {
                    got.push(format!(
                        "{}{}{}",
                        digit(bridges.is_repaired(b.object_id)),
                        digit(bridges.is_broken(b.object_id)),
                        b.damage_state as Int
                    ));
                }
                return Some((got.join(" "), want));
            }
            "wallpiece" => self.pf.add_wall_piece(t.id()),
            "unwallpiece" => self.pf.remove_wall_piece(t.id()),
            "bridge" => {
                let id = t.id();
                let [from, to, fl, fr, tl, tr] = std::array::from_fn(|_| t.point());
                assert_eq!(t.text(), "=");
                let geometry = BridgeGeometry::from_corners(from, to, fl, fr, tl, tr);
                self.terrain
                    .add_bridge(Bridge::new(geometry, "S2NoRoad", id), &mut self.pf);
                let got = self.terrain.bridges().iter().next().expect("bridge").layer;
                return Some(((got as Int).to_string(), t.text().to_string()));
            }
            "landmark" => {
                let id = t.id();
                let pos = t.point();
                let (angle, major, minor) = (t.real(), t.real(), t.real());
                assert_eq!(t.text(), "=");
                let want = t.rest().join(" ");
                let g = BridgeGeometry::from_box(&pos, angle, major, minor);
                self.terrain
                    .add_bridge(Bridge::new(g, "S2Landmark", id), &mut self.pf);
                let layer = self.terrain.bridges().iter().next().expect("bridge").layer;
                let mut got = (layer as Int).to_string();
                for c in [
                    g.from,
                    g.to,
                    g.from_left,
                    g.from_right,
                    g.to_left,
                    g.to_right,
                ] {
                    got.push_str(&format!(" {} {} {}", f(c.x), f(c.y), f(c.z)));
                }
                return Some((got, want));
            }
            "state" => {
                let got = state_fields(&self.pf);
                let want = t.rest();
                assert_eq!(got.len(), want.len());
                for ((name, value), item) in got.iter().zip(&want) {
                    let (key, expected) = item.split_once('=').expect("field");
                    assert_eq!(*name, key);
                    if value != expected {
                        return Some((format!("{name}={value}"), (*item).to_string()));
                    }
                }
            }
            "picture" => {
                let want = t.text().to_string();
                return Some((picture(&self.pf), want));
            }
            "query" => return Some(self.query(&mut t)),
            "connects" => {
                let layer = layer_of(t.int());
                let mask = t.int() as u32;
                let (a, b) = (t.int() as Int, t.int() as Int);
                assert_eq!(t.text(), "=");
                let got = self.pf.layer(layer).connects_zones(
                    self.pf.zones(),
                    self.pf.map(),
                    &self.pf.extent(),
                    mask as _,
                    a,
                    b,
                );
                return Some((digit(got).to_string(), t.text().to_string()));
            }
            "shape" => {
                let [from, to, fl, fr, tl, tr] = std::array::from_fn(|_| t.point());
                self.shape = Some(BridgeGeometry::from_corners(from, to, fl, fr, tl, tr));
            }
            "probe" => {
                let p = t.point();
                let r = t.region();
                assert_eq!(t.text(), "=");
                let want = t.rest().join(" ");
                let g = self.shape.expect("shape");
                let mut got = format!(
                    "{}{}{}{}",
                    digit(g.is_point_on(&p)),
                    digit(g.is_cell_on_end(&r)),
                    digit(g.is_cell_on_side(&r)),
                    digit(g.is_cell_entry_point(&r))
                );
                if want.ends_with(" -") {
                    got.push_str(" -");
                } else {
                    let mut n = Coord3D::default();
                    let h = g.height(&p, Some(&mut n));
                    got.push_str(&format!(" {} {} {} {}", f(h), f(n.x), f(n.y), f(n.z)));
                }
                return Some((got, want));
            }
            "line" => {
                let a = Coord2D {
                    x: t.real(),
                    y: t.real(),
                };
                let b = Coord2D {
                    x: t.real(),
                    y: t.real(),
                };
                let clip = t.region();
                assert_eq!(t.text(), "=");
                let got = digit(line_in_region(&a, &b, &clip)).to_string();
                return Some((got, t.text().to_string()));
            }
            "extent" => self.extent.hi = Coord3D::new(t.real(), t.real(), 100.0),
            "reset" => {
                assert_eq!(t.text(), "=");
                self.pf.reset(self.wall_height);
                assert_eq!(self.pf.cell_type(PathfindLayerEnum::Ground, 0, 0), None);
                assert!(!self.pf.begin_path_queue(&self.extent, 100_000));
                let mut crc = XferCrc::new();
                self.pf.crc(&mut crc).expect("crc");
                return Some((format!("{:08x}", crc.get_crc()), t.text().to_string()));
            }
            other => panic!("unknown row kind {other}"),
        }
        None
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
fn pathfind_map_matches_row_for_row() {
    let rows = rows();
    assert!(rows.len() > 15_000, "{} rows", rows.len());
    let mut scene = Scene::default();
    let mut name = "";
    let mut failed = Vec::new();
    let mut failed_scenes = 0;
    let mut broken = false;
    for (n, row) in rows.iter().enumerate() {
        if row.starts_with("scenario ") {
            // One pathfinder, reset between scenarios as the reference's is:
            // a layer keeps its destroyed flag until a bridge takes it again.
            let pf = std::mem::take(&mut scene.pf);
            scene = Scene {
                pf,
                ..Scene::default()
            };
            name = row;
            broken = false;
            continue;
        }
        // After a mismatch the rest of a scenario follows from it.
        if broken {
            continue;
        }
        if let Some((got, want)) = scene.replay(row) {
            if got != want {
                broken = name != "scenario -1";
                failed_scenes += 1;
                if failed.len() < 8 {
                    let head: String = rows[n.saturating_sub(1)].chars().take(60).collect();
                    let kind = row.split(' ').next().unwrap_or("");
                    let (got, want) = first_difference(&got, &want);
                    failed.push(format!(
                        "{name} row {n} {kind} after '{head}': got {got} want {want}"
                    ));
                }
            }
        }
    }
    assert!(
        failed.is_empty(),
        "{failed_scenes} mismatches\n{}",
        failed.join("\n")
    );
}

/// The two strings from a little before they part.
fn first_difference(got: &str, want: &str) -> (String, String) {
    let at = got
        .bytes()
        .zip(want.bytes())
        .position(|(a, b)| a != b)
        .unwrap_or(got.len().min(want.len()));
    let cut = |s: &str| {
        let from = at.saturating_sub(20);
        format!(
            "[{at}] {}",
            s.get(from..s.len().min(at + 60)).unwrap_or_default()
        )
    };
    (cut(got), cut(want))
}

#[test]
fn terrain_logic_closes_a_rubbled_bridge_layer() {
    use zh_logic::pathfinder::NoPathWorld;

    struct Rubble(BodyDamageType);
    impl BridgeWorld for Rubble {
        fn bridge_damage_state(&self, _id: ObjectId) -> Option<BodyDamageType> {
            Some(self.0)
        }
    }

    // Flat ground at height 0 under a deck at height 30.
    let mut terrain = TerrainLogic::new();
    let mut pf = Pathfinder::new(0.0);
    let corner = |x, y| Coord3D::new(x, y, 30.0);
    let geometry = BridgeGeometry::from_corners(
        corner(40.0, 55.0),
        corner(160.0, 55.0),
        corner(40.0, 70.0),
        corner(40.0, 40.0),
        corner(160.0, 70.0),
        corner(160.0, 40.0),
    );
    terrain.add_bridge(Bridge::new(geometry, "road", ObjectId(7)), &mut pf);
    let extent = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(200.0, 120.0, 100.0),
    };
    pf.new_map(&terrain, &extent, 0.0, &[], &NoPathWorld, 0);
    let layer = PathfindLayerEnum::Bridge1;
    assert!(!pf.layer(layer).is_destroyed());
    assert_eq!(pf.layer(layer).start_cell_index(), ICoord2D::new(3, 5));

    terrain.update_bridge_damage_states(&mut Rubble(BodyDamageType::Rubble), &mut pf, 10);
    assert!(pf.layer(layer).is_destroyed());
    assert!(terrain.bridges().is_broken(ObjectId(7)));
    assert!(pf.zones().next_frame_to_calculate_zones <= 310);

    terrain.update_bridge_damage_states(&mut Rubble(BodyDamageType::Pristine), &mut pf, 11);
    assert!(!pf.layer(layer).is_destroyed());
    assert!(terrain.bridges().is_repaired(ObjectId(7)));

    // The queue step starts a new cell budget (only searches spend it).
    pf.calculate_zones();
    pf.cumulative_cells_allocated = 9;
    assert!(pf.begin_path_queue(&extent, 12));
    assert_eq!(pf.cumulative_cells_allocated, 0);
    assert_eq!(pf.logical_extent().hi, ICoord2D::new(19, 11));
}

/// The calls modules make (`pathfinder_add_object` and friends) reach the map.
#[test]
fn sim_footprint_calls_change_the_map() {
    use zh_common::ini::{IniLoadType, IniReader};
    use zh_common::types::TeamId;
    use zh_logic::pathfinder::{CellType, PathfindEffects};
    use zh_logic::sim::{SimApi, SimContext, SimView};
    use zh_logic::terrain::TerrainEffects;

    let mut sim = SimContext::new();
    let ini = "Object Block\n  KindOf = STRUCTURE IMMOBILE\n  Geometry = BOX\n  \
               GeometryMajorRadius = 15.0\n  GeometryMinorRadius = 15.0\n  \
               GeometryHeight = 10.0\n  GeometryIsSmall = No\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    let template = sim
        .rules
        .thing_factory
        .find_template_id("Block")
        .expect("Block");
    let slot = sim.spawn_object(template, TeamId::default(), Coord3D::default(), 0.0);
    sim.set_object_position(slot, &Coord3D::new(105.0, 105.0, 0.0));

    let extent = Region3D {
        lo: Coord3D::default(),
        hi: Coord3D::new(200.0, 200.0, 100.0),
    };
    let at = |sim: &SimContext| sim.pathfind_cell_type(PathfindLayerEnum::Ground, 10, 10);

    // Without a map the calls only record membership.
    sim.pathfinder_add_object(slot);
    assert_eq!(at(&sim), CellType::Clear);

    // A new map classifies the objects already there.
    sim.new_pathfind_map(&extent, 0.0);
    assert_eq!(at(&sim), CellType::Obstacle);
    sim.pathfinder_remove_object(slot);
    assert_eq!(at(&sim), CellType::Clear);
    sim.pathfinder_add_object(slot);
    assert_eq!(at(&sim), CellType::Obstacle);
    sim.pathfinder_remove_object(slot);

    sim.pathfinder_create_wall(slot);
    assert_eq!(at(&sim), CellType::Obstacle);
    sim.pathfinder_remove_wall(slot);
    assert_eq!(at(&sim), CellType::Clear);

    // The terrain's water change reclassifies the cells.
    struct Flooded;
    impl ThingTerrainAccess for Flooded {
        fn get_ground_height(&self, _x: Real, _y: Real, _n: Option<&mut Coord3D>) -> Real {
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
            _x: Real,
            _y: Real,
            _water_z: Option<&mut Real>,
            _terrain_z: Option<&mut Real>,
        ) -> Bool {
            1
        }
        fn is_clear_line_of_sight(&self, _a: &Coord3D, _b: &Coord3D) -> Bool {
            1
        }
    }
    sim.terrain = Box::new(Flooded);
    sim.force_pathfind_recalculation();
    assert_eq!(at(&sim), CellType::Water);

    // A dry `TerrainLogic` through its effects seam.
    let mut pathfinder = sim.world.pathfinder.take().expect("map");
    let mut effects = PathfindEffects {
        pathfinder: &mut pathfinder,
        world: &sim,
    };
    effects.force_map_recalculation(&TerrainLogic::new());
    assert_eq!(
        pathfinder.cell_type(PathfindLayerEnum::Ground, 10, 10),
        Some(CellType::Clear)
    );
}
