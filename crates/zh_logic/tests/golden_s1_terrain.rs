//! Slice S1: map loading and terrain queries against vectors recorded by
//! `tests/cpp_harness/gen_s1_terrain.cpp`. Every row is replayed and the whole
//! line compared, floats as bits.
//!
//! `retail_maps_match` is local only: it reads the maps of an installed copy of
//! the game in place and compares digests; it passes without checking anything
//! where the archive is absent.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::Rc;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{decode_hex_bytes, encode_hex_bytes};
use zh_common::io::{BigFileSystem, FileInfo};
use zh_common::math::geometry::GeometryType;
use zh_common::math::{Coord3D, ICoord2D, Matrix3D};
use zh_common::types::{Bool, Real, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferLoad, XferSave};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::terrain::{
    BridgeDeck, ElevatedLayers, HeightMap, TerrainEffects, TerrainLogic, TerrainShape, WaterRef,
    Waypoint,
};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/S1_Terrain.json");
const RETAIL: &str = include_str!("../../../docs/golden_vectors/S1_Terrain_Retail.json");

fn f(v: Real) -> String {
    format!("{:08x}", v.to_bits())
}

fn c3(c: &Coord3D) -> String {
    format!("{} {} {}", f(c.x), f(c.y), f(c.z))
}

fn word(s: &str) -> &str {
    if s.is_empty() {
        "~"
    } else {
        s
    }
}

const FNV: u64 = 0xcbf2_9ce4_8422_2325;

fn fnv(mut h: u64, data: &[u8]) -> u64 {
    for &b in data {
        h ^= u64::from(b);
        h = h.wrapping_mul(0x100_0000_01b3);
    }
    h
}

/// The tokens of a row.
struct Tokens<'a>(std::str::SplitAsciiWhitespace<'a>);

impl<'a> Tokens<'a> {
    fn new(row: &'a str) -> Self {
        Self(row.split_ascii_whitespace())
    }
    fn text(&mut self) -> &'a str {
        self.0.next().expect("short row")
    }
    fn int(&mut self) -> i64 {
        self.text().parse().expect("integer")
    }
    fn real(&mut self) -> Real {
        Real::from_bits(u32::from_str_radix(self.text(), 16).expect("float bits"))
    }
    fn point(&mut self) -> Coord3D {
        let x = self.real();
        let y = self.real();
        let z = self.real();
        Coord3D::new(x, y, z)
    }
}

fn layer(v: i64) -> PathfindLayerEnum {
    PathfindLayerEnum::from_i32(v as i32).expect("layer")
}

/// The wall and bridge answers recorded for one row.
#[derive(Default)]
struct Tape {
    wall: Real,
    on_wall: Bool,
    decks: Vec<BridgeDeck>,
    found: Option<(Real, Coord3D)>,
}

#[derive(Clone, Default)]
struct Replay(Rc<RefCell<Tape>>); // determinism-ok: test double, single thread

impl ElevatedLayers for Replay {
    fn wall_height(&self) -> Real {
        self.0.borrow().wall
    }
    fn is_point_on_wall(&self, _pos: &Coord3D) -> Bool {
        self.0.borrow().on_wall
    }
    fn bridges_at(&self, _pos: &Coord3D) -> Vec<BridgeDeck> {
        self.0.borrow().decks.clone()
    }
    fn bridge_height(
        &self,
        _pos: &Coord3D,
        _layer: PathfindLayerEnum,
        _clip: Bool,
        normal: Option<&mut Coord3D>,
    ) -> Option<Real> {
        let (height, n) = self.0.borrow().found?;
        if let Some(out) = normal {
            *out = n;
        }
        Some(height)
    }
}

impl Replay {
    fn read(&self, t: &mut Tokens) {
        assert_eq!(t.text(), "|");
        let mut tape = Tape {
            wall: t.real(),
            on_wall: t.int() as Bool,
            ..Tape::default()
        };
        for _ in 0..t.int() {
            tape.decks.push(BridgeDeck {
                layer: layer(t.int()),
                height: t.real(),
                rubble: t.int() as Bool,
            });
        }
        if t.int() == 1 {
            tape.found = Some((t.real(), t.point()));
        }
        assert_eq!(t.text(), "=");
        *self.0.borrow_mut() = tape;
    }
}

/// The input tokens of a row, before `|` or `=`.
fn inputs(row: &str) -> &str {
    let end = row
        .find(" |")
        .or_else(|| row.find(" ="))
        .unwrap_or(row.len());
    &row[..end]
}

// ---- rows --------------------------------------------------------------------

fn cliff_digest(terrain: &TerrainLogic) -> u64 {
    let map = terrain.height_map().expect("height map");
    let mut h = FNV;
    for y in -1..=map.height() {
        for x in -1..=map.width() {
            h = fnv(h, &[map.cliffs().get_cliff_state(x, y) as u8]);
        }
    }
    h
}

fn sample_digest(terrain: &TerrainLogic) -> u64 {
    let map = terrain.height_map().expect("height map");
    // The data size can shrink below the allocation on a version 1 map.
    let size = (map.width() * map.height()) as usize;
    fnv(FNV, &map.samples()[..size])
}

fn row_map(terrain: &TerrainLogic) -> String {
    let map = terrain.height_map().expect("height map");
    let mut r = format!(
        "HM {} {} {} {} {:016x} {:016x} {}",
        map.width(),
        map.height(),
        map.border_size(),
        map.width() * map.height(),
        sample_digest(terrain),
        cliff_digest(terrain),
        map.boundaries().len()
    );
    for b in map.boundaries() {
        r += &format!(" {} {}", b.x, b.y);
    }
    r + " " + &f(map.max_height())
}

fn row_extents(terrain: &TerrainLogic) -> String {
    let e = terrain.extent();
    let b = terrain.extent_including_border();
    let p = terrain.maximum_pathfind_extent();
    format!(
        "EXT {} {} {} {} {} {}",
        c3(&e.lo),
        c3(&e.hi),
        c3(&b.lo),
        c3(&b.hi),
        c3(&p.lo),
        c3(&p.hi)
    )
}

fn rows_lists(terrain: &TerrainLogic) -> Vec<String> {
    let mut rows = Vec::new();
    for o in terrain.map_objects() {
        rows.push(format!(
            "OBJ {} {} {} {} {} {} {}",
            word(o.name()),
            c3(o.location()),
            f(o.angle()),
            o.flags(),
            o.is_waypoint(),
            o.is_light(),
            o.is_scorch()
        ));
    }
    for w in terrain.waypoints() {
        let links: Vec<u32> = terrain.waypoint_links(w).map(|l| l.id).collect();
        let mut r = format!(
            "WP {} {} {} {} {} {} {} {}",
            w.id,
            word(&w.name),
            c3(&w.location),
            word(&w.path_labels[0]),
            word(&w.path_labels[1]),
            word(&w.path_labels[2]),
            w.bi_directional,
            links.len()
        );
        for id in links {
            r += &format!(" {id}");
        }
        rows.push(r);
    }
    for p in terrain.triggers().triggers() {
        let mut r = format!(
            "TR {} {} {} {}",
            p.id(),
            word(&p.trigger_name),
            p.is_water_area,
            p.num_points()
        );
        for c in &p.points {
            r += &format!(" {} {} {}", c.x, c.y, c.z);
        }
        rows.push(r);
    }
    rows
}

fn row_height(terrain: &TerrainLogic, x: Real, y: Real) -> String {
    let mut n = Coord3D::new(-7.0, -7.0, -7.0);
    let with = terrain.get_ground_height(x, y, Some(&mut n));
    let without = terrain.get_ground_height(x, y, None);
    let max_cell = terrain
        .height_map()
        .expect("height map")
        .max_cell_height(x, y);
    format!(
        "H {} {} = {} {} {} {} {}",
        f(x),
        f(y),
        f(with),
        c3(&n),
        f(without),
        f(max_cell),
        terrain.is_cliff_cell(x, y)
    )
}

fn row_los(terrain: &TerrainLogic, a: &Coord3D, b: &Coord3D) -> String {
    format!(
        "LOS {} {} = {}",
        c3(a),
        c3(b),
        terrain.is_clear_line_of_sight(a, b)
    )
}

fn row_water(terrain: &TerrainLogic, x: Real, y: Real) -> String {
    let mut water = -12345.0;
    let mut ground = -12345.0;
    let under = terrain.is_underwater(x, y, Some(&mut water), Some(&mut ground));
    format!(
        "UW {} {} = {} {} {}",
        f(x),
        f(y),
        under,
        f(water),
        f(ground)
    )
}

fn align_outputs(
    terrain: &TerrainLogic,
    angle: Real,
    at: &Coord3D,
    stick: Bool,
) -> (PathfindLayerEnum, String) {
    let mut m = Matrix3D::identity();
    let layer = terrain.align_on_terrain(angle, at, stick, &mut m);
    let mut r = format!("{}", layer as i32);
    for row in m.rows {
        for v in row {
            r += " ";
            r += &f(v);
        }
    }
    (layer, r)
}

fn row_edges(terrain: &TerrainLogic, at: &Coord3D) -> String {
    format!(
        "EDGE {} = {} {}",
        c3(at),
        c3(&terrain.closest_edge_point(at)),
        c3(&terrain.farthest_edge_point(at))
    )
}

fn id_of(w: Option<&Waypoint>) -> String {
    w.map_or_else(|| "-".to_owned(), |w| format!("{} {}", w.id, word(&w.name)))
}

fn water_name(terrain: &TerrainLogic, water: Option<WaterRef>) -> String {
    match water {
        None => "none".to_owned(),
        Some(WaterRef::Grid) => "grid".to_owned(),
        Some(WaterRef::Trigger(i)) => terrain.triggers().triggers()[i].id().to_string(),
    }
}

fn water_by_id(terrain: &TerrainLogic, id: &str) -> Option<WaterRef> {
    let id: i32 = id.parse().expect("trigger id");
    let i = terrain
        .triggers()
        .triggers()
        .iter()
        .position(|t| t.id() == id && t.is_water_area != FALSE)
        .expect("water trigger");
    Some(WaterRef::Trigger(i))
}

// ---- water changes -------------------------------------------------------------

/// Records what a water change asks of the rest of the game.
#[derive(Default)]
struct Effects {
    victims: Vec<(Real, Real)>,
    recalculated: bool,
    damaged: Vec<bool>,
}

impl TerrainEffects for Effects {
    fn force_map_recalculation(&mut self) {
        self.recalculated = true;
    }
    /// The partition query: centres within the radius, then those underwater.
    fn damage_objects_in_water(
        &mut self,
        terrain: &TerrainLogic,
        center: &Coord3D,
        radius: Real,
        _amount: Real,
    ) {
        for (i, &(x, y)) in self.victims.iter().enumerate() {
            let (dx, dy) = (x - center.x, y - center.y);
            if dx * dx + dy * dy <= radius * radius
                && terrain.is_underwater(x, y, None, None) != FALSE
            {
                self.damaged[i] = true;
            }
        }
    }
}

fn mask(bits: &[bool]) -> String {
    bits.iter().map(|&b| if b { '1' } else { '0' }).collect()
}

fn save_bytes(terrain: &mut TerrainLogic) -> Vec<u8> {
    let mut save = XferSave::new();
    terrain.xfer(&mut save).expect("save");
    save.into_bytes()
}

fn water_state(terrain: &mut TerrainLogic, effects: &mut Effects) -> String {
    let mut r = format!(
        "{} {}",
        u8::from(effects.recalculated),
        mask(&effects.damaged)
    );
    let names: Vec<String> = terrain
        .triggers()
        .triggers()
        .iter()
        .filter(|t| t.is_water_area != FALSE)
        .map(|t| t.trigger_name.clone())
        .collect();
    for name in names {
        let water = terrain.water_by_name(&name);
        r += &format!(
            " {}:{}",
            water_name(terrain, water),
            f(terrain.water_height(water))
        );
        let Some(WaterRef::Trigger(i)) = water else {
            panic!("water area without a polygon");
        };
        for p in &terrain.triggers().triggers()[i].points {
            r += &format!(",{}", p.z);
        }
    }
    let bytes = save_bytes(terrain);

    // Loading what was saved gives the same bytes back. Where two polygons
    // share an id (the water area added to a version 1 trigger chunk does) the
    // original finds the wrong one and rejects the save.
    let ids: BTreeSet<i32> = terrain
        .triggers()
        .triggers()
        .iter()
        .map(|t| t.id())
        .collect();
    let mut load = XferLoad::new(&bytes);
    let loaded = terrain.xfer(&mut load);
    if ids.len() == terrain.triggers().triggers().len() {
        loaded.expect("load");
        assert_eq!(save_bytes(terrain), bytes);
    }

    effects.recalculated = false;
    effects.damaged.fill(false);
    r + " " + &encode_hex_bytes(&bytes)
}

// ---- the replay ----------------------------------------------------------------

struct Run {
    terrain: TerrainLogic,
    tape: Replay,
    effects: Effects,
}

impl Run {
    fn load(bytes: &[u8]) -> Self {
        let mut terrain = TerrainLogic::new();
        let mut keys = NameKeyGenerator::new();
        terrain
            .load_map(bytes, "map", &mut keys, ICoord2D { x: 70, y: 90 }, FALSE)
            .expect("map loads");
        terrain.new_map();
        let tape = Replay::default();
        terrain.set_elevated_layers(Box::new(tape.clone()));
        Self {
            terrain,
            tape,
            effects: Effects::default(),
        }
    }

    /// Performs the row's operation and returns the line it produces.
    fn replay(&mut self, row: &str, lists: &[String], list_row: &mut usize) -> String {
        let mut t = Tokens::new(row);
        let op = t.text();
        let head = inputs(row);
        let terrain = &mut self.terrain;
        match op {
            "HM" => row_map(terrain),
            "EXT" => row_extents(terrain),
            "OBJ" | "WP" | "TR" => {
                *list_row += 1;
                lists.get(*list_row - 1).cloned().unwrap_or_default()
            }
            "H" => {
                let x = t.real();
                row_height(terrain, x, t.real())
            }
            "LOS" => {
                let a = t.point();
                row_los(terrain, &a, &t.point())
            }
            "UW" => {
                let x = t.real();
                row_water(terrain, x, t.real())
            }
            "LH" => {
                let (x, y) = (t.real(), t.real());
                let (lay, clip) = (layer(t.int()), t.int() as Bool);
                self.tape.read(&mut t);
                let mut n = Coord3D::new(-7.0, -7.0, -7.0);
                let h = terrain.get_layer_height(x, y, lay, Some(&mut n), clip);
                let tape = &row[head.len()..row.find(" =").expect("outputs")];
                format!("{head}{tape} = {} {}", f(h), c3(&n))
            }
            "LAY" => {
                let at = t.point();
                self.tape.read(&mut t);
                let tape = &row[head.len()..row.find(" =").expect("outputs")];
                format!(
                    "{head}{tape} = {} {} {}",
                    terrain.get_layer_for_destination(&at) as i32,
                    terrain.get_highest_layer_for_destination(&at, FALSE) as i32,
                    terrain.get_highest_layer_for_destination(&at, TRUE) as i32
                )
            }
            "AL" => {
                let angle = t.real();
                let at = t.point();
                let stick = t.int() as Bool;
                self.tape.read(&mut t);
                let tape = &row[head.len()..row.find(" =").expect("outputs")];
                let (_, out) = align_outputs(terrain, angle, &at, stick);
                format!("{head}{tape} = {out}")
            }
            "EDGE" => row_edges(terrain, &t.point()),
            "WPN" => format!("{head} = {}", id_of(terrain.waypoint_by_name(t.text()))),
            "WPI" => format!("{head} = {}", id_of(terrain.waypoint_by_id(t.int() as u32))),
            "NEAR" => {
                let label = t.text();
                let label = if label == "~" { "" } else { label };
                let at = t.point();
                format!(
                    "{head} = {}",
                    id_of(terrain.closest_waypoint_on_path(&at, label))
                )
            }
            "PUR" => {
                let skip = t.int() as usize;
                let label = t.text();
                let label = if label == "~" { "" } else { label };
                let way = terrain.waypoints().nth(skip);
                format!("{head} = {}", TerrainLogic::is_purpose_of_path(way, label))
            }
            "AREA" => {
                let key = t.text();
                let name = if key == "Water_Grid" {
                    "Water Grid"
                } else {
                    key
                };
                let area = terrain
                    .trigger_area_by_name(name)
                    .map_or_else(|| "-".to_owned(), |a| a.id().to_string());
                let water = terrain.water_by_name(name);
                let mut r = format!("{head} = {area} {} ", water_name(terrain, water));
                match water {
                    Some(w @ WaterRef::Trigger(_)) => {
                        let rect = terrain.water_bounding_rect(w);
                        r += &format!(
                            "{} {} {}",
                            f(terrain.water_height(water)),
                            c3(&rect.lo),
                            c3(&rect.hi)
                        );
                    }
                    _ => r += "-",
                }
                r
            }
            "VICTIMS" => {
                let count = t.int() as usize;
                self.effects.victims = (0..count)
                    .map(|_| {
                        let x = t.real();
                        (x, t.real())
                    })
                    .collect();
                self.effects.damaged = vec![false; count];
                row.to_owned()
            }
            "SETW" => {
                let water = water_by_id(terrain, t.text());
                let (height, damage, force) = (t.real(), t.real(), t.int() as Bool);
                terrain.set_water_height(water, height, damage, force, &mut self.effects);
                format!("{head} = {}", water_state(terrain, &mut self.effects))
            }
            "CHG" => {
                let water = water_by_id(terrain, t.text());
                let (height, seconds, damage) = (t.real(), t.real(), t.real());
                terrain.change_water_height_over_time(water, height, seconds, damage);
                format!("{head} = {}", water_state(terrain, &mut self.effects))
            }
            "UPD" => {
                terrain.update(t.int() as u32, &mut self.effects);
                format!("{head} = {}", water_state(terrain, &mut self.effects))
            }
            "UNDER" => {
                let under: Vec<bool> = self
                    .effects
                    .victims
                    .iter()
                    .map(|&(x, y)| terrain.is_underwater(x, y, None, None) != FALSE)
                    .collect();
                format!("{head} = {}", mask(&under))
            }
            "FLAT" | "CRATER" => {
                let geom_type = match t.int() {
                    0 => GeometryType::Sphere,
                    1 => GeometryType::Cylinder,
                    _ => GeometryType::Box,
                };
                let shape = TerrainShape {
                    geom_type,
                    is_small: t.int() as Bool,
                    position: t.point(),
                    angle: t.real(),
                    major_radius: t.real(),
                    minor_radius: t.real(),
                };
                if op == "CRATER" {
                    terrain.create_crater_in_terrain(&shape);
                } else {
                    terrain.flatten_terrain(&shape);
                }
                let ground = terrain.get_ground_height(shape.position.x, shape.position.y, None);
                format!("{head} = {:016x} {}", sample_digest(terrain), f(ground))
            }
            other => panic!("unknown row kind {other}"),
        }
    }
}

/// The maps of the vector file: the file bytes and the rows.
fn maps() -> Vec<(Vec<u8>, Vec<&'static str>)> {
    let mut out: Vec<(Vec<u8>, Vec<&str>)> = Vec::new();
    for line in VECTORS.lines() {
        let line = line.trim().trim_end_matches(',');
        if let Some(at) = line.find("\"hex\": \"") {
            let hex = line[at + 8..].trim_end_matches('"');
            out.push((decode_hex_bytes(hex).expect("map bytes"), Vec::new()));
        } else if line.starts_with('"') && line.ends_with('"') && line.len() > 2 {
            out.last_mut()
                .expect("a row before its map")
                .1
                .push(&line[1..line.len() - 1]);
        }
    }
    out
}

#[test]
fn synthetic_maps_match_row_for_row() {
    let maps = maps();
    assert_eq!(maps.len(), 8);
    let mut total = 0;
    for (index, (bytes, rows)) in maps.iter().enumerate() {
        let mut run = Run::load(bytes);
        let lists = rows_lists(&run.terrain);
        let mut list_row = 0;
        let mut failed: Vec<String> = Vec::new();
        let mut failures = 0;
        for (n, row) in rows.iter().enumerate() {
            let got = run.replay(row, &lists, &mut list_row);
            if got != *row {
                failures += 1;
                if failed.len() < 4 {
                    failed.push(format!("row {n}\n  want {row}\n  got  {got}"));
                }
            }
        }
        assert_eq!(
            failures,
            0,
            "map {index}: {failures} rows differ\n{}",
            failed.join("\n")
        );
        assert_eq!(list_row, lists.len(), "map {index}: extra list entries");
        total += rows.len();
    }
    assert!(total >= 8000, "{total} rows");
}

// ---- behaviour the recorded maps do not reach ----------------------------------

#[test]
fn active_boundary_changes_only_to_a_usable_one() {
    let (bytes, _) = &maps()[0];
    let mut run = Run::load(bytes);
    let terrain = &mut run.terrain;
    let first = terrain.extent();
    assert_eq!(terrain.set_active_boundary(-1), FALSE);
    assert_eq!(terrain.set_active_boundary(3), FALSE);
    assert_eq!(terrain.set_active_boundary(0), FALSE);
    assert_eq!(terrain.set_active_boundary(2), TRUE);
    assert_eq!(terrain.active_boundary(), 2);
    assert!(terrain.extent().hi.x < first.hi.x);
    assert!(terrain.extent().hi.y < first.hi.y);
    assert_eq!(terrain.maximum_pathfind_extent().hi.x, first.hi.x);

    // A load restores the boundary.
    let saved = save_bytes(terrain);
    assert_eq!(terrain.set_active_boundary(1), TRUE);
    let mut load = XferLoad::new(&saved);
    terrain.xfer(&mut load).expect("load");
    assert_eq!(terrain.active_boundary(), 2);
}

#[test]
fn waypoint_seam_answers_by_id_and_name() {
    use zh_logic::ai::seams::MapTerrain;
    let (bytes, _) = &maps()[0];
    let run = Run::load(bytes);
    let terrain = &run.terrain;
    let first = terrain.first_waypoint().expect("waypoints");
    let info = MapTerrain::waypoint_by_id(terrain, first).expect("by id");
    assert_eq!(info.id, first);
    assert_eq!(MapTerrain::waypoint_by_name(terrain, "no such"), None);
    let linked = terrain
        .waypoints()
        .find(|w| terrain.waypoint_links(w).next().is_some())
        .expect("a linked waypoint");
    let info = MapTerrain::waypoint_by_id(terrain, linked.id).expect("by id");
    assert!(!info.links.is_empty());
    assert_eq!(terrain.get_extent(), terrain.extent());
    assert_eq!(
        terrain.get_extent_including_border(),
        terrain.extent_including_border()
    );
    assert_eq!(
        terrain.get_maximum_pathfind_extent(),
        terrain.maximum_pathfind_extent()
    );
}

#[test]
fn a_wall_is_a_layer_only_on_the_wall() {
    // Flat ground at height 10.
    let mut terrain = TerrainLogic::new();
    let size = vec![ICoord2D { x: 16, y: 16 }];
    terrain.set_height_map(HeightMap::from_samples(16, 16, 0, size, vec![16; 256]));
    let tape = Replay::default();
    terrain.set_elevated_layers(Box::new(tape.clone()));
    let run = Run {
        terrain,
        tape,
        effects: Effects::default(),
    };
    let at = Coord3D::new(100.0, 100.0, 0.0);
    let ground = run.terrain.get_ground_height(at.x, at.y, None);
    assert_eq!(ground, 10.0);
    let top = Coord3D::new(at.x, at.y, ground + 90.0);
    *run.tape.0.borrow_mut() = Tape {
        wall: ground + 80.0,
        on_wall: TRUE,
        ..Tape::default()
    };
    let t = &run.terrain;
    assert_eq!(t.get_layer_for_destination(&top), PathfindLayerEnum::Wall);
    assert_eq!(
        t.get_highest_layer_for_destination(&top, FALSE),
        PathfindLayerEnum::Wall
    );
    // Below the wall the highest layer is the ground; the nearest is still the wall.
    let below = Coord3D::new(at.x, at.y, ground + 70.0);
    assert_eq!(t.get_layer_for_destination(&below), PathfindLayerEnum::Wall);
    assert_eq!(
        t.get_highest_layer_for_destination(&below, FALSE),
        PathfindLayerEnum::Ground
    );
    assert_eq!(
        t.get_layer_height(at.x, at.y, PathfindLayerEnum::Wall, None, TRUE),
        ground + 80.0
    );
    run.tape.0.borrow_mut().on_wall = FALSE;
    assert_eq!(t.get_layer_for_destination(&top), PathfindLayerEnum::Ground);
    assert_eq!(
        t.get_layer_height(at.x, at.y, PathfindLayerEnum::Wall, None, TRUE),
        ground
    );
}

// ---- retail maps ---------------------------------------------------------------

const GRID: i64 = 96;

struct Digest {
    hash: u64,
    rows: usize,
}

impl Digest {
    fn new() -> Self {
        Self { hash: FNV, rows: 0 }
    }
    fn add(&mut self, row: &str) {
        self.hash = fnv(fnv(self.hash, row.as_bytes()), b"\n");
        self.rows += 1;
    }
    fn text(&self) -> String {
        format!("{}:{:016x}", self.rows, self.hash)
    }
}

struct Random(u32);

impl Random {
    fn pick(&mut self, n: u32) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0 % n
    }
    fn between(&mut self, lo: Real, hi: Real) -> Real {
        let unit = self.pick(1_000_000) as Real / 1_000_000.0;
        lo + unit * (hi - lo)
    }
}

fn grid_at(i: i64, samples: i32, border: i32) -> Real {
    let span = i64::from(samples) * 100 + 300;
    let tenths = i * span / (GRID - 1) - i64::from(border) * 100 - 150 + (i % 7) * 3;
    tenths as Real / 10.0
}

fn retail_digests(terrain: &TerrainLogic) -> String {
    let map = terrain.height_map().expect("height map");
    let (w, h, border) = (map.width(), map.height(), map.border_size());
    let mut digests: [Digest; 7] = std::array::from_fn(|_| Digest::new());
    let [head, objects, heights, cliffs, water, sight, align] = &mut digests;
    head.add(&row_map(terrain));
    head.add(&row_extents(terrain));
    let lists = rows_lists(terrain);
    for r in &lists {
        objects.add(r);
    }
    for y in -1..=h {
        let bits: Vec<bool> = (-1..=w)
            .map(|x| map.cliffs().get_cliff_state(x, y) != FALSE)
            .collect();
        cliffs.add(&mask(&bits));
    }
    for j in 0..GRID {
        for i in 0..GRID {
            let (x, y) = (grid_at(i, w, border), grid_at(j, h, border));
            heights.add(&row_height(terrain, x, y));
            water.add(&row_water(terrain, x, y));
        }
    }
    let mut random = Random(0x051e_7a11);
    for _ in 0..600 {
        let ax = grid_at(i64::from(random.pick(GRID as u32)), w, border);
        let ay = grid_at(i64::from(random.pick(GRID as u32)), h, border);
        let bx = ax + random.between(-300.0, 300.0);
        let by = ay + random.between(-300.0, 300.0);
        let az = terrain.get_ground_height(ax, ay, None) + random.between(0.0, 30.0);
        let bz = terrain.get_ground_height(bx, by, None) + random.between(0.0, 30.0);
        sight.add(&row_los(
            terrain,
            &Coord3D::new(ax, ay, az),
            &Coord3D::new(bx, by, bz),
        ));
        let a = Coord3D::new(ax, ay, 0.0);
        let angle = random.between(-3.0, 3.0);
        let (_, out) = align_outputs(terrain, angle, &a, TRUE);
        align.add(&format!(
            "AL {} {} 1 | 00000000 0 0 0 = {out}",
            f(angle),
            c3(&a)
        ));
        align.add(&row_edges(terrain, &a));
    }
    let count = |kind: &str| lists.iter().filter(|r| r.starts_with(kind)).count();
    format!(
        "w={w} h={h} wp={} tr={} map={} lists={} heights={} cliffs={} water={} sight={} align={}",
        count("WP "),
        count("TR "),
        head.text(),
        objects.text(),
        heights.text(),
        cliffs.text(),
        water.text(),
        sight.text(),
        align.text()
    )
}

#[test]
fn retail_maps_match() {
    let root = std::env::var("CNC_GENERALS_ZH_PATH").unwrap_or_else(|_| {
        format!(
            "{}/Games/generalszh",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    let Ok(bytes) = std::fs::read(format!("{root}/MapsZH.big")) else {
        eprintln!("retail_maps_match: no MapsZH.big under {root}; nothing checked");
        return;
    };
    let archive = BigFileSystem::open_archive_bytes("MapsZH.big", bytes, FileInfo::default())
        .expect("archive");
    let expected: Vec<(&str, &str)> = RETAIL
        .lines()
        .filter_map(|l| {
            l.trim()
                .trim_end_matches(',')
                .strip_prefix('"')?
                .strip_suffix('"')?
                .split_once(' ')
        })
        .collect();
    assert!(expected.len() >= 50, "{} maps recorded", expected.len());

    let mut names = BTreeSet::new();
    archive.get_file_list_in_directory("", "*.map", &mut names);
    let mut checked = 0;
    let mut failed = Vec::new();
    for entry in names {
        let name = entry.as_str().to_owned();
        let data = archive
            .open_file(&name)
            .expect("listed file")
            .read_entire_and_close();
        // Maps are keyed by a hash of the file, which names no map.
        let key = format!("{:016x}", fnv(FNV, &data));
        let Some((_, want)) = expected.iter().find(|(k, _)| *k == key) else {
            continue;
        };
        let mut terrain = TerrainLogic::new();
        let mut keys = NameKeyGenerator::new();
        terrain
            .load_map(&data, &name, &mut keys, ICoord2D { x: 70, y: 90 }, FALSE)
            .expect("retail map loads");
        terrain.new_map();
        let got = retail_digests(&terrain);
        if got != *want {
            let differing: Vec<&str> = got
                .split(' ')
                .zip(want.split(' '))
                .filter_map(|(g, w)| (g != w).then_some(g))
                .collect();
            failed.push(format!("map {key}: {}", differing.join(" ")));
        }
        checked += 1;
    }
    assert!(
        failed.is_empty(),
        "{} of {checked} differ:\n{}",
        failed.len(),
        failed.join("\n")
    );
    assert_eq!(checked, expected.len(), "recorded maps not in the archive");
}
