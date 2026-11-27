//! `PKT-260` `BridgeBehavior` with the landmark half of the terrain's bridges against
//! the reference: 120 bridges, each with its towers and a few units around the deck,
//! under scripted damage, healing, scaffolding and death
//! (`tests/cpp_harness/gen_bridge_behavior.cpp`, `docs/golden_vectors/PKT-260.json`).
//!
//! Every field of every `at` row is compared. The ground is flat, and its layer
//! heights follow the shipped terrain (`W3DTerrainLogic::getLayerHeight`).

use zh_common::golden_json as json;
use zh_common::ini::IniLoadType;
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::game_engine::GameDataLoader;
use zh_logic::modules::module::BridgeTowerType;
use zh_logic::modules::update::physics_update::{physics, ALLOW_TO_FALL};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::pathfinder::Pathfinder;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};
use zh_logic::terrain::bridge::point_inside_area_2d;
use zh_logic::terrain::TerrainLogic;

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/PKT-260.json");

const NEUTRAL: TeamId = TeamId(1);
const TEAM_A: TeamId = TeamId(2);
const UNITS: [&str; 4] = ["BB_Walker", "BB_Roller", "BB_Flyer", "BB_TowerA"];
const TOWERS: [BridgeTowerType; 4] = [
    BridgeTowerType::FromLeft,
    BridgeTowerType::FromRight,
    BridgeTowerType::ToLeft,
    BridgeTowerType::ToRight,
];

// ---- terrain ------------------------------------------------------------------

/// Flat ground at zero under the sim's bridges.
struct FlatTerrain(TerrainLogic);

impl ThingTerrainAccess for FlatTerrain {
    fn terrain_logic(&self) -> Option<&TerrainLogic> {
        Some(&self.0)
    }
    fn terrain_logic_mut(&mut self) -> Option<&mut TerrainLogic> {
        Some(&mut self.0)
    }
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        self.0.align_on_terrain(angle, pos, stick_to_ground, mtx)
    }
    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        FALSE
    }
    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum {
        self.0.get_layer_for_destination(pos)
    }
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        clip: Bool,
    ) -> Real {
        let mut normal = normal;
        let height = self.get_ground_height(x, y, normal.as_deref_mut());
        if layer == PathfindLayerEnum::Ground {
            return height;
        }
        let loc = Coord3D::new(x, y, height);
        match self.0.bridges().find_layer_at(&loc, layer, clip != FALSE) {
            Some(bridge) => {
                let deck = bridge.geometry.height(&loc, normal);
                if deck > height {
                    deck
                } else {
                    height
                }
            }
            None => height,
        }
    }
}

// ---- rows ---------------------------------------------------------------------

fn bits(word: &str) -> Real {
    Real::from_bits(u32::from_str_radix(word, 16).expect("float bits"))
}

fn f32_hex(value: Real) -> String {
    format!("{:08x}", value.to_bits())
}

#[derive(Clone)]
struct Fnv(u32);

impl Fnv {
    fn new() -> Self {
        Self(0x811c_9dc5)
    }
    fn bytes(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ u32::from(b)).wrapping_mul(0x0100_0193);
        }
    }
    fn add(&mut self, v: u32) {
        self.bytes(&v.to_le_bytes());
    }
    fn real(&mut self, v: Real) {
        self.add(v.to_bits());
    }
}

// ---- world --------------------------------------------------------------------

struct Case {
    sim: SimContext,
    bridge: ObjectSlot,
    module: ModuleSlot,
    units: Vec<ObjectId>,
}

fn new_sim(ini: &str, frame: u32, seed: u32) -> SimContext {
    let mut loader = GameDataLoader::new();
    loader.load_bytes("bridges.ini", ini.as_bytes(), IniLoadType::Overwrite);
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    let mut sim = SimContext::new();
    sim.rules = loader.rules;
    sim.rules.global_data.unit_damaged_thresh = 0.7;
    sim.rules.global_data.unit_really_damaged_thresh = 0.35;
    sim.world = SimWorld::new();
    let w = &mut sim.world;
    for (player, team) in [NEUTRAL, TEAM_A].into_iter().enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(NEUTRAL, TRUE);
    w.neutral_team_id = NEUTRAL;
    w.local_player_index = 1;
    for i in 0..2 {
        for j in 0..2 {
            let rel = if i == j {
                Relationship::Allies
            } else {
                Relationship::Neutral
            };
            w.player_relationships.insert((i, j), rel);
        }
    }
    w.frame = frame;
    w.next_object_id = 1;
    w.rng.init_random_with_seed(seed);
    sim.terrain = Box::new(FlatTerrain(TerrainLogic::new()));
    sim.init_partition(20.0, extent());
    sim.world.pathfinder = Some(Box::new(Pathfinder::new(0.0)));
    sim
}

fn extent() -> Region3D {
    Region3D {
        lo: Coord3D::new(0.0, 0.0, 0.0),
        hi: Coord3D::new(600.0, 600.0, 100.0),
    }
}

fn make(sim: &mut SimContext, name: &str, team: TeamId) -> ObjectSlot {
    let template = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("no template {name}"));
    sim.factory_new_object(template, team)
}

fn set_indestructible(sim: &mut SimContext, slot: ObjectSlot, on: bool) {
    let body = sim.object(slot).and_then(|o| o.body_slot).expect("body");
    sim.dispatch_module(body, |m, sim, ctx| {
        m.as_body_mut()
            .expect("body facet")
            .set_indestructible(sim, ctx, Bool::from(on));
    })
    .expect("dispatch");
}

/// `case <index> <template> <x> <y> <z> <angle> <indestructible> <seed> <frame>`.
fn new_case(ini: &str, w: &[&str]) -> Case {
    let index: u32 = w[1].parse().expect("index");
    let seed: u32 = w[8].parse().expect("seed");
    let frame: u32 = w[9].parse().expect("frame");
    let indestructible = w[7] == "1";
    let mut sim = new_sim(ini, frame, seed);
    let bridge = make(&mut sim, w[2], TEAM_A);
    sim.set_object_orientation(bridge, bits(w[6]));
    sim.set_object_position(bridge, &Coord3D::new(bits(w[3]), bits(w[4]), bits(w[5])));
    if indestructible {
        set_indestructible(&mut sim, bridge, true);
    }
    sim.add_landmark_bridge(bridge);
    sim.new_pathfind_map(&extent(), 0.0);
    if indestructible && index % 3 != 0 {
        set_indestructible(&mut sim, bridge, index % 2 == 0);
    }
    let module = sim
        .object(bridge)
        .and_then(|o| o.bridge_slot)
        .expect("BridgeBehavior");
    Case {
        sim,
        bridge,
        module,
        units: Vec::new(),
    }
}

/// `unit <kind> <x> <y> <z> <on deck>`.
fn spawn_unit(c: &mut Case, w: &[&str]) {
    let kind: usize = w[1].parse().expect("kind");
    let pos = Coord3D::new(bits(w[2]), bits(w[3]), bits(w[4]));
    let layer = c
        .sim
        .bridge_list()
        .and_then(|l| l.iter().next())
        .expect("bridge")
        .layer;
    let unit = make(&mut c.sim, UNITS[kind], TEAM_A);
    c.sim.set_object_position(unit, &pos);
    c.sim.set_object_orientation(unit, 0.0);
    c.sim.set_object_layer(
        unit,
        if w[5] == "1" {
            layer
        } else {
            PathfindLayerEnum::Ground
        },
    );
    c.units.push(c.sim.object(unit).expect("unit").id);
}

fn save(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("take");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

fn tower_id(c: &Case, which: usize) -> ObjectId {
    c.sim
        .world
        .modules
        .payload(c.module)
        .and_then(|m| m.as_bridge())
        .expect("facet")
        .get_tower_id(TOWERS[which])
}

fn body_row(sim: &SimContext, id: Option<ObjectSlot>) -> String {
    let Some(slot) = id else {
        return "-".to_owned();
    };
    let obj = sim.object(slot).expect("object");
    format!(
        "{}:{}:{}",
        f32_hex(sim.object_health(slot)),
        sim.object_body_damage_state(slot) as i32,
        u8::from(obj.is_effectively_dead() != FALSE),
    )
}

fn deck_digest(sim: &SimContext) -> String {
    let bridge = sim
        .bridge_list()
        .and_then(|l| l.iter().next())
        .expect("bridge");
    let pathfinder = sim.world.pathfinder.as_ref().expect("pathfinder");
    let bounds = &bridge.geometry.bounds;
    let (x0, y0) = (
        (bounds.lo.x / 10.0) as i32 - 2,
        (bounds.lo.y / 10.0) as i32 - 2,
    );
    let (x1, y1) = (
        (bounds.hi.x / 10.0) as i32 + 3,
        (bounds.hi.y / 10.0) as i32 + 3,
    );
    let mut d = Fnv::new();
    for y in y0..y1 {
        for x in x0..x1 {
            for layer in [bridge.layer, PathfindLayerEnum::Ground] {
                d.add(pathfinder.get_cell(layer, x, y).map_or(9999, |idx| {
                    let cell = pathfinder.map().grid().cell(idx);
                    cell.cell_type() as u32 * 100 + cell.connect_layer() as u32
                }));
            }
        }
    }
    format!("{:08x}", d.0)
}

fn objects_digest(c: &mut Case) -> String {
    let slots: Vec<ObjectSlot> = c.sim.world.objects.world_iter().collect();
    let mut digest = Fnv::new().0;
    let count = slots.len();
    for slot in slots {
        let scaffold = c.sim.object(slot).and_then(|o| o.bridge_scaffold_slot);
        let piece = scaffold.map(|m| save(&mut c.sim, m));
        let obj = c.sim.object(slot).expect("object");
        let template = obj
            .thing
            .template_override()
            .get_final(&c.sim.rules.thing_factory)
            .map_or("", |t| t.get_name());
        let pos = obj.thing.get_position();
        let body = obj.body_slot;
        let mut one = Fnv::new();
        one.add(obj.id.0 as u32);
        one.bytes(template.as_bytes());
        one.real(pos.x);
        one.real(pos.y);
        one.real(pos.z);
        one.real(obj.thing.get_orientation());
        one.add(obj.layer as u32);
        one.add(body.map_or(0, |_| c.sim.object_health(slot).to_bits()));
        one.add(u32::from(obj.is_effectively_dead() != FALSE));
        one.add(u32::from(obj.is_destroyed()));
        let indestructible = body
            .and_then(|m| c.sim.world.modules.payload(m))
            .and_then(|m| m.as_body())
            .is_some_and(|b| b.is_indestructible() != FALSE);
        one.add(u32::from(indestructible));
        one.add(u32::from(
            physics(&c.sim, slot).is_some_and(|p| p.flag(ALLOW_TO_FALL)),
        ));
        if let Some(bytes) = piece {
            one.bytes(
                bytes[bytes.len() - 60..]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>()
                    .as_bytes(),
            );
        }
        digest ^= one.0;
    }
    format!("{count}:{digest:08x}")
}

fn log_digest(entries: impl Iterator<Item = (String, Coord3D)>) -> String {
    let mut d = Fnv::new();
    let mut count = 0;
    for (name, p) in entries {
        let line = format!("{name}:{}{}{};", f32_hex(p.x), f32_hex(p.y), f32_hex(p.z));
        d.bytes(line.as_bytes());
        count += 1;
    }
    format!("{count}:{:08x}", d.0)
}

fn state_row(c: &mut Case) -> String {
    let rng = c.sim.world.rng.get_game_logic_random_seed_crc();
    let mut r = format!(
        "at rng={rng:08x} bridge={} towers=",
        body_row(&c.sim, Some(c.bridge))
    );
    for which in 0..4 {
        let id = tower_id(c, which);
        let row = body_row(&c.sim, c.sim.world.objects.slot_by_id(id));
        r += &format!("{}/{row},", id.0);
    }
    let bridge = c
        .sim
        .bridge_list()
        .and_then(|l| l.iter().next())
        .expect("bridge");
    r += &format!(
        " record={}:{}:{}:{}",
        bridge.damage_state as i32,
        u8::from(bridge.damage_state_changed),
        bridge.object_id.0,
        bridge.layer as i32,
    );
    r += &format!(" deck={}", deck_digest(&c.sim));
    let facet = c
        .sim
        .world
        .modules
        .payload(c.module)
        .and_then(|m| m.as_bridge())
        .expect("facet");
    r += &format!(
        " scaffold={}:{}",
        facet.is_scaffold_present(),
        facet.is_scaffold_in_motion(&c.sim)
    );
    let bytes = save(&mut c.sim, c.module);
    let mut crc = XferCrc::new();
    c.sim
        .world
        .modules
        .payload(c.module)
        .expect("payload")
        .crc(&mut crc);
    r += " save=";
    for b in bytes {
        r += &format!("{b:02x}");
    }
    r += &format!(" crc={:08x}", crc.get_crc());
    r += &format!(" objects={}", objects_digest(c));
    let fx = c.sim.world.triggered_fx_lists.iter().cloned();
    r += &format!(" fx={}", log_digest(fx));
    let ocl = c.sim.world.spawned_ocls.iter().map(|e| (e.0.clone(), e.2));
    r += &format!(" ocl={}", log_digest(ocl));
    r
}

fn damage(
    sim: &mut SimContext,
    target: ObjectSlot,
    kind: DamageType,
    amount: Real,
    from: ObjectId,
) {
    let mut info = DamageInfo::default();
    info.input.damage_type = kind;
    info.input.death_type = DeathType::Normal;
    info.input.source_id = from;
    info.input.amount = amount;
    sim.attempt_damage(target, &mut info);
}

fn heal(sim: &mut SimContext, target: ObjectSlot, amount: Real, from: ObjectId) {
    let mut info = DamageInfo::default();
    info.input.damage_type = DamageType::Healing;
    info.input.death_type = DeathType::NoneType;
    info.input.source_id = sim
        .world
        .objects
        .slot_by_id(from)
        .map_or(ObjectId::INVALID, |_| from);
    info.input.amount = amount;
    sim.attempt_healing(target, &mut info);
}

fn kill(sim: &mut SimContext, target: ObjectSlot) {
    let mut info = DamageInfo::default();
    info.input.damage_type = DamageType::Unresistable;
    info.input.death_type = DeathType::Normal;
    info.input.source_id = ObjectId::INVALID;
    info.input.amount = sim.object_max_health(target);
    info.input.kill = TRUE;
    sim.attempt_damage(target, &mut info);
}

/// `op <code> <tower> <source> <amount> <unresistable>`.
fn apply(c: &mut Case, w: &[&str]) {
    let code: u32 = w[1].parse().expect("code");
    let which: usize = w[2].parse().expect("tower");
    let source: u32 = w[3].parse().expect("source");
    let amount = bits(w[4]);
    let kind = if w[5] == "1" {
        DamageType::Unresistable
    } else {
        DamageType::Explosion
    };
    let tower = c.sim.world.objects.slot_by_id(tower_id(c, which));
    let from = match source {
        1 => tower_id(c, (which + 1) % 4),
        2 => c.units.first().copied().unwrap_or(ObjectId::INVALID),
        3 => c.sim.object(c.bridge).expect("bridge").id,
        _ => ObjectId::INVALID,
    };
    let (bridge, module) = (c.bridge, c.module);
    match code {
        0 => damage(&mut c.sim, bridge, kind, amount, from),
        1 => heal(&mut c.sim, bridge, amount, from),
        2 => {
            if let Some(tower) = tower {
                damage(&mut c.sim, tower, kind, amount * 0.4, from);
            }
        }
        3 => {
            if let Some(tower) = tower {
                heal(&mut c.sim, tower, amount * 0.4, from);
            }
        }
        4 | 5 => {
            c.sim
                .dispatch_module(module, |m, sim, ctx| {
                    let facet = m.as_bridge_mut().expect("facet");
                    if code == 4 {
                        facet.create_scaffolding(sim, ctx);
                    } else {
                        facet.remove_scaffolding(sim, ctx);
                    }
                })
                .expect("dispatch");
        }
        6 | 9 => {
            for _ in 0..=source {
                c.sim.world.frame += 1;
                c.sim
                    .dispatch_module(module, |m, sim, ctx| {
                        m.as_update_mut().expect("update").update(sim, ctx);
                    })
                    .expect("dispatch");
            }
        }
        7 => kill(&mut c.sim, bridge),
        8 => {
            if let Some(tower) = tower {
                kill(&mut c.sim, tower);
            }
        }
        _ => panic!("unknown op {code}"),
    }
}

/// The fixture INI and the rows.
fn golden() -> (String, Vec<&'static str>) {
    let ini = json::get_str(GOLDEN, "ini")
        .expect("ini")
        .replace("\\n", "\n");
    let rows = GOLDEN
        .lines()
        .skip_while(|line| !line.contains("\"rows\""))
        .filter_map(|line| {
            let line = line.trim().trim_end_matches(',');
            line.strip_prefix('"')?.strip_suffix('"')
        })
        .collect();
    (ini, rows)
}

#[test]
fn bridges_match_the_reference() {
    let (ini, rows) = golden();
    let mut case: Option<Case> = None;
    let mut name = String::new();
    let mut step = 0;
    let (mut cases, mut compared, mut areas) = (0, 0, 0);
    for &row in &rows {
        let words: Vec<&str> = row.split(' ').collect();
        match words[0] {
            "case" => {
                // The reference reads the INI once; a fresh sim per case here.
                // `index % 4 == 2` (no definition) never builds a scaffold.
                case = Some(new_case(&ini, &words));
                name = row.to_owned();
                step = 0;
                cases += 1;
            }
            // `area <point> <four corners> <inside>`.
            "area" => {
                let at = |i: usize| Coord3D::new(bits(words[i]), bits(words[i + 1]), 0.0);
                let area = [at(3), at(5), at(7), at(9)];
                let inside = point_inside_area_2d(&at(1), &area);
                assert_eq!(words[11], u8::from(inside).to_string(), "{row}");
                areas += 1;
            }
            "unit" => spawn_unit(case.as_mut().expect("case"), &words),
            "op" => {
                let c = case.as_mut().expect("case");
                step += 1;
                let variant: u32 = name
                    .split(' ')
                    .nth(1)
                    .and_then(|w| w.parse().ok())
                    .expect("index");
                if !(words[1] == "4" && variant % 4 == 2) {
                    apply(c, &words);
                }
            }
            "at" => {
                let c = case.as_mut().expect("case");
                c.sim.update_partition();
                let got = state_row(c);
                let differing: Vec<(&str, &str)> = row
                    .split(' ')
                    .zip(got.split(' '))
                    .filter(|(a, b)| a != b)
                    .collect();
                assert!(
                    differing.is_empty(),
                    "{name}, step {step}: (reference, port) {differing:?}"
                );
                assert_eq!(row, got, "{name}, step {step}");
                compared += 1;
            }
            other => panic!("unknown row {other}"),
        }
    }
    assert_eq!((cases, compared, areas), (120, 120 * 17, 600));
}

#[test]
fn xfer_loads_what_it_saves_and_hands_the_terrain_its_ids() {
    let (ini, rows) = golden();
    let first: Vec<&str> = rows[0].split(' ').collect();
    let mut c = new_case(&ini, &first);
    let module = c.module;
    c.sim
        .dispatch_module(module, |m, sim, ctx| {
            m.as_bridge_mut()
                .expect("facet")
                .create_scaffolding(sim, ctx);
        })
        .expect("dispatch");
    let saved = save(&mut c.sim, module);
    let towers: Vec<ObjectId> = (0..4).map(|i| tower_id(&c, i)).collect();

    // Version, update header, four towers, scaffold flag and count, ids, death frame.
    let count = c.sim.world.objects.world_iter().count() - 1 - towers.len();
    assert!(count > 0);

    let fresh = make(&mut c.sim, "BB_Full", TEAM_A);
    let fresh_module = c
        .sim
        .object(fresh)
        .and_then(|o| o.bridge_slot)
        .expect("module");
    let pos = *c.sim.object(c.bridge).expect("bridge").thing.get_position();
    c.sim.set_object_position(fresh, &pos);
    let fresh_id = c.sim.object(fresh).expect("fresh").id;
    let (mut payload, ctx, generation) = c
        .sim
        .world
        .modules
        .take_payload(fresh_module)
        .expect("take");
    let mut load = zh_common::xfer::XferLoad::new(&saved);
    payload.xfer(&mut load, &c.sim).expect("load");
    let mut again = XferSave::new();
    payload
        .xfer(&mut again, &SimContext::new())
        .expect("resave");
    assert_eq!(again.as_bytes(), &saved[..]);
    payload.load_post_process(&mut c.sim, ctx);
    c.sim
        .world
        .modules
        .restore_payload(fresh_module, generation, payload);

    let bridge = c
        .sim
        .bridge_list()
        .and_then(|l| l.iter().next())
        .expect("bridge");
    assert_eq!(bridge.object_id, fresh_id);
    assert_eq!(bridge.tower_ids.to_vec(), towers);
}

/// The module's destructor takes the towers with it.
#[test]
fn destroying_the_bridge_destroys_its_towers() {
    let (ini, rows) = golden();
    let first: Vec<&str> = rows[0].split(' ').collect();
    let mut c = new_case(&ini, &first);
    let towers: Vec<ObjectId> = (0..4).map(|i| tower_id(&c, i)).collect();
    assert!(towers.iter().all(|&id| c.sim.find_object(id).is_some()));
    c.sim.destroy_object(c.bridge);
    c.sim.process_destroy_list();
    c.sim.process_destroy_list();
    assert!(towers.iter().all(|&id| c.sim.find_object(id).is_none()));
}
