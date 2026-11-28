//! `PKT-272` `GenerateMinefieldBehavior` and `PKT-279` `MinefieldBehavior` against vectors
//! from the reference modules (`tests/cpp_harness/gen_pkt272_279_minefield.cpp`), with the
//! footprint operations, the potential collision query and the cliff flags they use.

use std::any::Any;
use std::fmt::Write as _;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{decode_hex_bytes, get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::random_value::PrngState;
use zh_common::types::{
    Bool, NameKeyType, ObjectId, Real, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::module::{AiUpdateFacet, BehaviorModule, ModuleData};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::ModuleType;
use zh_logic::object::thing::{make_align_to_normal_matrix, ThingTerrainAccess};
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};
use zh_logic::spatial::terrain::CliffMap;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT272_279_Minefield.json");
const UPGRADES: [&str; 4] = [
    "Upgrade_ChinaEMPMines",
    "P272_Trigger",
    "P272_Other",
    "P272_Conflict",
];
const BRIDGE_LO: Real = 703.25;
const BRIDGE_HI: Real = 797.75;
const BRIDGE_Z: Real = 20.0;
const PRIVATE_EFFECTIVELY_DEAD: u8 = 1;

// ---------------------------------------------------------------------------
// Terrain (the harness's `MineTerrain`)
// ---------------------------------------------------------------------------

struct Terrain {
    ground: Real,
    step: Real,
    water_x: Real,
    water_y: Real,
    cliffs: CliffMap,
}

impl ThingTerrainAccess for Terrain {
    fn get_ground_height(&self, x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        if x > 400.0 {
            self.ground + self.step
        } else {
            self.ground
        }
    }
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        make_align_to_normal_matrix(angle, pos, &Coord3D::new(0.0, 0.0, 1.0), mtx);
        PathfindLayerEnum::Ground
    }
    fn is_cliff_cell(&self, x: Real, y: Real) -> Bool {
        self.cliffs.is_cliff_cell(x, y)
    }
    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        Bool::from(x > self.water_x && y > self.water_y)
    }
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        if layer != PathfindLayerEnum::Ground {
            return BRIDGE_Z;
        }
        self.get_ground_height(x, y, normal)
    }
    /// The reference's own test for one flat bridge.
    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        _only_healthy: Bool,
    ) -> PathfindLayerEnum {
        let on = |v: Real| (BRIDGE_LO..=BRIDGE_HI).contains(&v);
        let best = pos.z - self.get_ground_height(pos.x, pos.y, None);
        let delta = pos.z - BRIDGE_Z;
        if on(pos.x) && on(pos.y) && delta >= 0.0 && delta.abs() < best.abs() {
            PathfindLayerEnum::Bridge1
        } else {
            PathfindLayerEnum::Ground
        }
    }
}

// ---------------------------------------------------------------------------
// Stand-in for the unported AI update: only the trait's defaults.
// ---------------------------------------------------------------------------

struct AiData {
    tag_key: NameKeyType,
    name_key: NameKeyType,
}

impl ModuleData for AiData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn facet_mask(&self) -> FacetMask {
        FacetMask::AI_UPDATE
    }
}

struct PlainAi(NameKeyType);

impl AiUpdateFacet for PlainAi {}

impl BehaviorModule for PlainAi {
    fn module_name_key(&self) -> NameKeyType {
        self.0
    }
    fn xfer(&mut self, _x: &mut dyn Xfer, _s: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }
}

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(AiData { tag_key, name_key }))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(PlainAi(data.module_name_key()))
}

// ---------------------------------------------------------------------------
// World
// ---------------------------------------------------------------------------

fn text(key: &str) -> String {
    let header = VECTORS.lines().next().expect("header");
    let hex = get_str(header, key).expect(key);
    String::from_utf8(decode_hex_bytes(hex).expect("hex")).expect("utf8")
}

fn context() -> SimContext {
    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        FacetMask::AI_UPDATE,
        SleepyUpdatePhase::Normal,
        true,
        parse_ai,
        create_ai,
    );
    let nk: &mut NameKeyGenerator = &mut rules.name_keys;
    for name in UPGRADES {
        rules.upgrade_center.new_upgrade(name, nk);
    }
    for block in text("weapons").split_inclusive("End\n") {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(block.trim_end()).expect("weapon");
        rules.weapon_store.add_template(weapon);
    }
    rules.fx_list_names.insert("FX_P272".to_owned());
    rules.ocl_names.insert("P279_OCL".to_owned());
    rules.global_data.gravity = -1.0;
    rules.global_data.standard_minefield_distance = 33.0;
    rules.global_data.standard_minefield_density = 0.006;
    let ini = text("ini");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules = rules;
    sim
}

/// A fresh world: teams 0 neutral, 1 A, 2 B and 3, a second team of player A.
fn new_world(sim: &mut SimContext) {
    sim.world = SimWorld::new();
    for (team, player) in [(0, 0), (1, 1), (2, 2), (3, 1)] {
        sim.world.team_players.insert(TeamId(team), player);
    }
    for player in 0..3 {
        sim.world
            .player_default_teams
            .insert(player, TeamId(player));
    }
    sim.world.set_player_relation(1, 2, Relationship::Enemies);
    sim.world.set_player_relation(2, 1, Relationship::Enemies);
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
}

fn spawn(sim: &mut SimContext, name: &str, team: u32, pos: Coord3D, angle: Real) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let o = sim.spawn_object(tid, TeamId(team as i32), Coord3D::default(), 0.0);
    sim.set_object_position(o, &pos);
    sim.set_object_orientation(o, angle);
    o
}

fn module_named(sim: &SimContext, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.lookup_key(name)?;
    let obj = sim.world.objects.get(owner)?;
    obj.behaviors.iter().map(|&(ms, _)| ms).find(|&ms| {
        sim.world
            .modules
            .payload(ms)
            .is_some_and(|p| p.module_name_key() == key)
    })
}

fn save(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

fn load(sim: &mut SimContext, slot: ModuleSlot, bytes: &[u8]) {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xl = XferLoad::new(bytes);
    payload.xfer(&mut xl, sim).expect("xfer load");
    assert_eq!(xl.position(), bytes.len(), "load consumed");
    sim.world.modules.restore_payload(slot, generation, payload);
}

fn module_crc(sim: &SimContext, slot: ModuleSlot) -> u32 {
    let mut xc = XferCrc::new();
    sim.world
        .modules
        .payload(slot)
        .expect("payload")
        .crc(&mut xc);
    xc.get_crc()
}

fn bytes_crc(bytes: &[u8]) -> u32 {
    let mut c = zh_common::crc::Crc::new();
    c.compute_crc(bytes, bytes.len() as i32);
    c.get()
}

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn bits(p: &Coord3D) -> String {
    format!("{},{},{}", p.x.to_bits(), p.y.to_bits(), p.z.to_bits())
}

fn numbers(s: &str) -> Vec<u32> {
    s.split_whitespace()
        .map(|t| t.parse::<i64>().expect("number") as u32)
        .collect()
}

fn real(v: u32) -> Real {
    Real::from_bits(v)
}

fn point(v: &[u32]) -> Coord3D {
    Coord3D::new(real(v[0]), real(v[1]), real(v[2]))
}

fn set_terrain(sim: &mut SimContext, line: &str) {
    let t = numbers(get_str(line, "terrain").expect("terrain"));
    let mut cliffs = CliffMap::new(104, 104, 2);
    for y in t[5] as i32..t[7] as i32 {
        for x in t[4] as i32..t[6] as i32 {
            cliffs.set_cliff_state(x, y, TRUE);
        }
    }
    sim.terrain = Box::new(Terrain {
        ground: real(t[0]),
        step: real(t[1]),
        water_x: real(t[2]),
        water_y: real(t[3]),
        cliffs,
    });
}

/// The harness's `step_module`.
fn step(sim: &mut SimContext, m: ModuleSlot, frame: u32) {
    sim.world.frame = frame;
    if next_call_frame(sim, m) > frame {
        return;
    }
    let sleep = sim
        .dispatch_module(m, |b, s, c| b.as_update_mut().expect("update").update(s, c))
        .expect("dispatch");
    let sleep = if sleep.0 < 1 {
        UpdateSleepTime::NONE
    } else {
        sleep
    };
    let next = frame.saturating_add(sleep.0).min(UPDATE_SLEEP_FOREVER);
    sim.world
        .modules
        .meta_mut(m)
        .schedule
        .set_next_call_frame(next);
    let index = sim.world.modules.meta(m).schedule.index_in_logic();
    if index >= 0 {
        sim.world
            .sleepy_updates
            .rebalance(&mut sim.world.modules, index as usize);
    }
}

fn effectively_dead(sim: &SimContext, o: ObjectSlot) -> bool {
    sim.world
        .objects
        .get(o)
        .expect("object")
        .is_effectively_dead()
        != FALSE
}

fn find(sim: &SimContext, id: ObjectId) -> Option<ObjectSlot> {
    sim.find_object(id)
}

/// Every object made from a mine template, oldest first.
fn mines(sim: &SimContext) -> Vec<ObjectSlot> {
    let mut v: Vec<(ObjectId, ObjectSlot)> = sim
        .world_object_slots()
        .into_iter()
        .filter_map(|s| {
            let o = sim.world.objects.get(s)?;
            let name = template_name(sim, s);
            (name.starts_with("P279_G") || name.starts_with("P279_M")).then_some((o.id, s))
        })
        .collect();
    v.sort_by_key(|&(id, _)| id);
    v.into_iter().map(|(_, s)| s).collect()
}

fn template_name(sim: &SimContext, s: ObjectSlot) -> &str {
    sim.world
        .objects
        .get(s)
        .expect("object")
        .thing
        .template_override()
        .get_final(&sim.rules.thing_factory)
        .expect("template")
        .get_name()
}

/// The harness's `mine_list`.
fn mine_list(sim: &mut SimContext) -> String {
    let mut out = String::new();
    for s in mines(sim) {
        let m = sim.world.objects.get(s).unwrap().land_mine_slot;
        let crc = m.map_or(0, |m| bytes_crc(&save(sim, m)));
        let next = m.map_or(0, |m| next_call_frame(sim, m));
        let dead = u8::from(effectively_dead(sim, s));
        let name = template_name(sim, s).to_owned();
        let o = sim.world.objects.get(s).unwrap();
        let _ = write!(
            out,
            "{},{name},{},{},{},{},{},{crc},{dead},{next};",
            o.id.0,
            o.team_id.0,
            o.producer_id.0,
            bits(o.thing.get_position()),
            o.thing.get_orientation().to_bits(),
            o.layer as i32,
        );
    }
    out
}

fn lines(kind: &str) -> Vec<&'static str> {
    let tag = format!("{{\"kind\":\"{kind}\"");
    VECTORS
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with(&tag))
        .collect()
}

fn split_list(s: &str, sep: char) -> Vec<&str> {
    s.split(sep).filter(|t| !t.is_empty()).collect()
}

#[track_caller]
fn same(got: &str, want: &str, what: std::fmt::Arguments<'_>) {
    if got != want {
        // Show the first differing field rather than the whole row.
        let (g, w): (Vec<&str>, Vec<&str>) = (
            got.split([' ', ';']).collect(),
            want.split([' ', ';']).collect(),
        );
        let i = g.iter().zip(&w).take_while(|(a, b)| a == b).count();
        panic!(
            "{what}: field {i} of {}/{}: got {:?}, want {:?}",
            g.len(),
            w.len(),
            g.get(i),
            w.get(i)
        );
    }
}

// ---------------------------------------------------------------------------
// Placement
// ---------------------------------------------------------------------------

#[test]
fn placement_matches_reference() {
    let vectors = lines("place");
    assert_eq!(vectors.len(), 600);
    let mut sim = context();
    let mut most_mines = 0;
    for (index, line) in vectors.iter().enumerate() {
        new_world(&mut sim);
        set_terrain(&mut sim, line);
        let variant = get_u32(line, "variant").expect("variant");
        let team = get_u32(line, "team").expect("team");
        let seed = get_u32(line, "seed").expect("seed");
        let pose = numbers(get_str(line, "pose").expect("pose"));
        let ops = split_list(get_str(line, "ops").expect("ops"), ';');
        let rows = split_list(get_str(line, "rows").expect("rows"), '|');
        assert_eq!(ops.len(), rows.len());

        let mut frame = get_u32(line, "frame").expect("frame");
        sim.world.frame = frame;
        sim.world.rng.init_random_with_seed(seed);

        let producer = spawn(
            &mut sim,
            &format!("P272_P{variant}"),
            team,
            point(&pose),
            real(pose[3]),
        );
        let gen = module_named(&sim, producer, "GenerateMinefieldBehavior").expect("generator");
        for block in split_list(get_str(line, "blocks").expect("blocks"), ';') {
            let b = numbers(block);
            spawn(
                &mut sim,
                &format!("P279_S{}", b[0]),
                2,
                point(&b[1..4]),
                real(b[4]),
            );
        }
        sim.update_partition();

        let mut last_list = String::new();
        let mut fx_seen = 0;
        for (k, (op, want)) in ops.iter().zip(&rows).enumerate() {
            let op = numbers(op);
            let (code, a) = (op[0], op[1]);
            sim.world.frame = frame;
            let mut ret = 0;
            let mask = |sim: &SimContext| {
                let mut mask = UpgradeMaskType::new();
                for (i, name) in UPGRADES.iter().enumerate() {
                    if a & (1 << i) != 0 {
                        let t = sim
                            .rules
                            .upgrade_center
                            .lookup_upgrade(name, &sim.rules.name_keys)
                            .expect("upgrade");
                        mask.clear_and_set(&UpgradeMaskType::new(), t.upgrade_mask());
                    }
                }
                mask
            };
            match code {
                0 => {
                    let mut info = DamageInfo::default();
                    info.input.damage_type = DamageType::Unresistable;
                    sim.dispatch_module(gen, |m, s, c| {
                        m.as_die_mut().expect("die").on_die(s, c, &info)
                    })
                    .unwrap();
                }
                1 => {
                    let t = point(&op[2..5]);
                    sim.dispatch_module(gen, |m, _, _| {
                        m.as_generate_minefield_mut()
                            .expect("target")
                            .set_minefield_target((a != 0).then_some(&t))
                    })
                    .unwrap();
                }
                2 | 4 | 10 => {
                    let mask = mask(&sim);
                    ret = sim
                        .dispatch_module(gen, |m, s, c| {
                            let up = m.as_upgrade_mut().expect("upgrade");
                            match code {
                                2 => up.attempt_upgrade(s, c, &mask),
                                4 => up.reset_upgrade(s, c, &mask),
                                _ => up.would_upgrade(s, c, &mask),
                            }
                        })
                        .unwrap() as u32;
                }
                3 => sim
                    .dispatch_module(gen, |m, s, c| {
                        m.as_upgrade_mut().unwrap().force_refresh_upgrade(s, c)
                    })
                    .unwrap(),
                5 => {
                    let t = sim
                        .rules
                        .upgrade_center
                        .lookup_upgrade(UPGRADES[0], &sim.rules.name_keys)
                        .expect("upgrade")
                        .upgrade_mask()
                        .clone();
                    let o = sim.world.objects.get_mut(producer).unwrap();
                    o.upgrades_completed
                        .clear_and_set(&UpgradeMaskType::new(), &t);
                }
                6 => {
                    ret = sim
                        .dispatch_module(gen, |m, s, c| m.as_update_mut().unwrap().update(s, c))
                        .unwrap()
                        .0;
                }
                7 => {
                    for _ in 0..a {
                        frame += 1;
                        sim.world.frame = frame;
                        for s in mines(&sim) {
                            if let Some(m) = sim.world.objects.get(s).unwrap().land_mine_slot {
                                step(&mut sim, m, frame);
                            }
                        }
                        sim.process_destroy_list();
                    }
                }
                8 => {
                    let v = mines(&sim);
                    if !v.is_empty() {
                        sim.destroy_object(v[a as usize % v.len()]);
                    }
                }
                9 => {
                    // `a` is set where the reference could not read its own save back.
                    if a == 0 {
                        let bytes = save(&mut sim, gen);
                        load(&mut sim, gen, &bytes);
                    }
                }
                11 => sim.update_partition(),
                _ => panic!("op {code}"),
            }
            sim.process_destroy_list();

            let bytes = save(&mut sim, gen);
            let list = mine_list(&mut sim);
            most_mines = most_mines.max(mines(&sim).len());
            let mut fx = String::new();
            for (name, p) in &sim.world.triggered_fx_lists[fx_seen..] {
                assert_eq!(name, "FX_P272");
                let _ = write!(fx, ".{}", bits(p));
            }
            fx_seen = sim.world.triggered_fx_lists.len();
            let shown = if list == last_list {
                "=".to_owned()
            } else {
                format!("m{list}")
            };
            let got = format!(
                "{ret} {} {} {} f{fx} {shown}",
                hex(&bytes),
                module_crc(&sim, gen),
                sim.world.rng.get_game_logic_random_seed_crc()
            );
            same(
                &got,
                want,
                format_args!("vector {index} variant {variant} op {k} ({code} {a})"),
            );
            last_list = list;
        }
    }
    assert!(
        most_mines > 255,
        "a vector passes the saved count: {most_mines}"
    );
}

// ---------------------------------------------------------------------------
// Behaviour
// ---------------------------------------------------------------------------

struct Mine {
    slot: ObjectSlot,
    module: ModuleSlot,
    heal: Option<ModuleSlot>,
    fires_seen: usize,
    ocls_seen: usize,
}

fn mine_row(sim: &mut SimContext, mine: &mut Mine) -> String {
    let bytes = save(sim, mine.module);
    let crc = module_crc(sim, mine.module);
    let next = next_call_frame(sim, mine.module);
    let heal = match mine.heal {
        Some(h) => {
            let c = bytes_crc(&save(sim, h));
            format!("{c},{}", next_call_frame(sim, h))
        }
        None => "-".to_owned(),
    };
    let health = sim.object_health(mine.slot);
    let dead = effectively_dead(sim, mine.slot);
    let o = sim.world.objects.get(mine.slot).unwrap();
    let status = |s: ObjectStatusType| u8::from(o.status.test(s.as_i32()) != FALSE);
    let rubble = u8::from(
        o.model_condition
            .test(ModelConditionFlagType::Rubble.as_i32())
            != 0,
    );
    let flags = format!(
        "{}{}{rubble}{}{}",
        status(ObjectStatusType::Masked),
        status(ObjectStatusType::NoAttackFromAi),
        u8::from(o.is_destroyed()),
        u8::from(dead),
    );
    let place = format!("{},{}", bits(o.thing.get_position()), o.layer as i32);
    let mut fires = String::new();
    for (_, name, _, p) in &sim.world.weapon_fires[mine.fires_seen..] {
        assert_eq!(name, "P279_Det");
        let _ = write!(fires, ".{}", bits(p));
    }
    mine.fires_seen = sim.world.weapon_fires.len();
    let mut ocls = String::new();
    for (name, creator, at, to) in &sim.world.spawned_ocls[mine.ocls_seen..] {
        assert_eq!(name, "P279_OCL");
        assert_eq!(at, to);
        let _ = write!(ocls, ".{},{}", creator.0, creator.0);
    }
    mine.ocls_seen = sim.world.spawned_ocls.len();
    format!(
        "{} {crc} {} {flags} {place} {next} {heal} f{fires} o{ocls} {}",
        hex(&bytes),
        health.to_bits(),
        sim.world.rng.get_game_logic_random_seed_crc()
    )
}

#[test]
fn minefield_matches_reference() {
    let vectors = lines("mine");
    assert_eq!(vectors.len(), 900);
    let mut sim = context();
    let mut destroyed = 0;
    for (index, line) in vectors.iter().enumerate() {
        new_world(&mut sim);
        set_terrain(&mut sim, line);
        let variant = get_u32(line, "variant").expect("variant");
        let team = get_u32(line, "team").expect("team");
        let producer_team = zh_common::golden_json::get_i32(line, "producer").expect("producer");
        let seed = get_u32(line, "seed").expect("seed");
        let mut frame = get_u32(line, "frame").expect("frame");
        let pose = numbers(get_str(line, "pose").expect("pose"));
        sim.world.frame = frame;
        sim.world.rng.init_random_with_seed(seed);

        let at = point(&pose);
        let producer = (producer_team >= 0).then(|| {
            spawn(
                &mut sim,
                "P279_S1",
                producer_team as u32,
                Coord3D::new(at.x + 200.0, at.y, 0.0),
                0.0,
            )
        });
        let slot = spawn(
            &mut sim,
            &format!("P279_M{variant}"),
            team,
            at,
            real(pose[3]),
        );
        let producer_id = producer.map(|p| sim.world.objects.get(p).unwrap().id);
        if let Some(id) = producer_id {
            sim.world.objects.get_mut(slot).unwrap().producer_id = id;
        }
        let mut mine = Mine {
            slot,
            module: module_named(&sim, slot, "MinefieldBehavior").expect("minefield"),
            heal: module_named(&sim, slot, "AutoHealBehavior"),
            fires_seen: 0,
            ocls_seen: 0,
        };
        let units: Vec<ObjectSlot> = split_list(get_str(line, "units").expect("units"), ';')
            .iter()
            .map(|u| {
                let u = numbers(u);
                spawn(
                    &mut sim,
                    &format!("P279_U{}", u[0]),
                    u[1],
                    point(&u[2..5]),
                    real(u[5]),
                )
            })
            .collect();
        let ids: Vec<ObjectId> = units
            .iter()
            .map(|&u| sim.world.objects.get(u).unwrap().id)
            .collect();
        let mine_id = sim.world.objects.get(slot).unwrap().id;
        let busy = numbers(get_str(line, "busy").expect("busy"));
        for (u, b) in units.iter().zip(busy.chunks(2)) {
            if b[0] != 0 {
                sim.set_status(*u, ObjectStatusType::IsAttacking);
            }
            let goal = units.get(b[1] as usize).copied();
            let record = sim.world.objects.get_mut(*u).unwrap();
            if record.ai_slot.is_some() {
                record.ai_goal_object = goal;
            }
        }
        sim.update_partition();
        sim.world.rng.init_random_with_seed(seed + 1);
        same(
            &mine_row(&mut sim, &mut mine),
            get_str(line, "spawn").expect("spawn"),
            format_args!("vector {index} variant {variant} spawn"),
        );

        let ops = split_list(get_str(line, "ops").expect("ops"), ';');
        let rows = split_list(get_str(line, "rows").expect("rows"), '|');
        assert_eq!(ops.len(), rows.len());
        for (k, (op, want)) in ops.iter().zip(&rows).enumerate() {
            let op = numbers(op);
            let (code, p) = (op[0], &op[1..]);
            sim.world.frame = frame;
            let m = mine.module;
            let unit = |sim: &SimContext, i: u32| ids.get(i as usize).and_then(|&id| find(sim, id));
            match code {
                0 => {
                    let other = unit(&sim, p[0]);
                    sim.dispatch_module(m, |b, s, c| {
                        b.as_collide_mut()
                            .expect("collide")
                            .on_collide(s, c, other, None, None)
                    })
                    .unwrap();
                }
                1 => {
                    if let Some(o) = unit(&sim, p[0]) {
                        sim.set_object_position(o, &point(&p[1..4]));
                    }
                }
                2 => {
                    let mut info = DamageInfo::default();
                    info.input.damage_type =
                        DamageType::from_i32(p[0] as i32).expect("damage type");
                    info.input.death_type = DeathType::Normal;
                    info.input.amount = real(p[1]);
                    info.input.source_id = match p[2] {
                        0 => ObjectId::INVALID,
                        1 => mine_id,
                        n => ids[n as usize - 2],
                    };
                    sim.attempt_damage(slot, &mut info);
                }
                3 => {
                    let mut info = DamageInfo::default();
                    info.input.damage_type = DamageType::Healing;
                    info.input.death_type = DeathType::NoneType;
                    info.input.source_id = mine_id;
                    info.input.amount = real(p[0]);
                    sim.attempt_healing(slot, &mut info);
                }
                4 => sim
                    .dispatch_module(m, |b, s, c| {
                        b.as_land_mine_mut().expect("land mine").disarm(s, c)
                    })
                    .unwrap(),
                5 => {
                    for _ in 0..p[0] {
                        if sim.world.objects.get(slot).unwrap().is_destroyed() {
                            break;
                        }
                        frame += 1;
                        step(&mut sim, m, frame);
                        if let Some(h) = mine.heal {
                            if !sim.world.objects.get(slot).unwrap().is_destroyed() {
                                step(&mut sim, h, frame);
                            }
                        }
                    }
                }
                6 => {
                    if let Some(o) = unit(&sim, p[0]) {
                        if p[1] != 0 {
                            sim.set_status(o, ObjectStatusType::IsAttacking);
                        } else {
                            sim.clear_status(o, ObjectStatusType::IsAttacking);
                        }
                    }
                }
                7 => {
                    let goal = unit(&sim, p[1]);
                    if let Some(o) = unit(&sim, p[0]) {
                        let record = sim.world.objects.get_mut(o).unwrap();
                        if record.ai_slot.is_some() {
                            record.ai_goal_object = goal;
                        }
                    }
                }
                8 => {
                    if let Some(o) = producer_id.and_then(|id| find(&sim, id)) {
                        if p[0] == 0 {
                            sim.destroy_object(o);
                        } else {
                            sim.world.objects.get_mut(o).unwrap().private_status |=
                                PRIVATE_EFFECTIVELY_DEAD;
                        }
                    }
                }
                9 => {
                    let (from, to) = (point(&p[0..3]), point(&p[3..6]));
                    sim.set_land_mine_scoot_parms(slot, &from, &to);
                }
                10 => {
                    if let Some(o) = unit(&sim, p[0]) {
                        sim.destroy_object(o);
                    }
                }
                11 => {
                    if let Some(o) = unit(&sim, p[0]) {
                        let r = sim.world.objects.get_mut(o).unwrap();
                        if p[1] != 0 {
                            r.private_status |= PRIVATE_EFFECTIVELY_DEAD;
                        } else {
                            r.private_status &= !PRIVATE_EFFECTIVELY_DEAD;
                        }
                    }
                }
                12 => {
                    let bytes = save(&mut sim, m);
                    load(&mut sim, m, &bytes);
                }
                13 => frame += p[0],
                14 => {
                    sim.dispatch_module(m, |b, s, c| b.as_update_mut().unwrap().update(s, c))
                        .unwrap();
                }
                _ => panic!("op {code}"),
            }
            same(
                &mine_row(&mut sim, &mut mine),
                want,
                format_args!("vector {index} variant {variant} op {k} ({code} {p:?})"),
            );
            if sim.world.objects.get(slot).unwrap().is_destroyed() {
                destroyed += 1;
                assert_eq!(k + 1, ops.len(), "vector {index}");
            }
            sim.process_destroy_list();
            sim.update_partition();
        }
    }
    assert!(destroyed > 100, "{destroyed}");
}

// ---------------------------------------------------------------------------
// Footprints, cliffs and potential collisions
// ---------------------------------------------------------------------------

fn geometry(v: &[u32]) -> GeometryInfo {
    GeometryInfo::new(
        GeometryType::from_i32(v[0] as i32),
        FALSE,
        real(v[1]),
        real(v[2]),
        real(v[3]),
    )
}

#[test]
fn footprint_operations_match_reference() {
    let vectors = lines("geom");
    assert_eq!(vectors.len(), 250);
    for (index, line) in vectors.iter().enumerate() {
        let mut g = geometry(&numbers(get_str(line, "geom").expect("geom")));
        let mut rng = PrngState::default();
        rng.init_random_with_seed(get_u32(line, "seed").expect("seed"));
        let ops = split_list(get_str(line, "ops").expect("ops"), ';');
        let rows = split_list(get_str(line, "rows").expect("rows"), '|');
        for (k, (op, want)) in ops.iter().zip(&rows).enumerate() {
            let op = numbers(op);
            let p = &op[1..];
            let mut got = String::new();
            match op[0] {
                0 => g.expand_footprint(real(p[0])),
                1 => {
                    let r = g.get_bounding_circle_radius();
                    g.set(GeometryType::Cylinder, FALSE, 1.0, r, r);
                }
                2 => {
                    let inside = g.is_point_in_footprint(&point(&p[0..3]), &point(&p[3..6]));
                    let _ = write!(got, "{} ", inside as u32);
                }
                3 => {
                    let mut q = point(&p[3..6]);
                    g.clip_point_to_footprint(&point(&p[0..3]), &mut q);
                    let _ = write!(got, "{} ", bits(&q));
                }
                _ => {
                    let mut q = Coord3D::new(1.0, 2.0, 3.0);
                    g.make_random_offset_within_footprint(&mut q, &mut rng);
                    let _ = write!(got, "{} ", bits(&q));
                }
            }
            let _ = write!(
                got,
                "{},{},{},{},{},{} {}",
                g.get_geom_type().as_i32(),
                g.get_major_radius().to_bits(),
                g.get_minor_radius().to_bits(),
                g.get_bounding_circle_radius().to_bits(),
                g.get_bounding_sphere_radius().to_bits(),
                g.get_footprint_area().to_bits(),
                rng.get_game_logic_random_seed_crc()
            );
            same(
                &got,
                want,
                format_args!("vector {index} op {k} ({})", op[0]),
            );
        }
    }
}

#[test]
fn cliff_flags_match_reference() {
    let vectors = lines("cliff");
    assert_eq!(vectors.len(), 120);
    for (index, line) in vectors.iter().enumerate() {
        let s = numbers(get_str(line, "size").expect("size"));
        let (w, h, border) = (s[0] as i32, s[1] as i32, s[2] as i32);
        let data = get_str(line, "data").expect("data");
        let map = if s[3] != 0 {
            let heights = decode_hex_bytes(data).expect("heights");
            CliffMap::from_heights(w, h, border, &heights)
        } else {
            let mut map = CliffMap::new(w, h, border);
            for (i, c) in data.bytes().enumerate() {
                let i = i as i32;
                // Set everything first, so clearing is exercised too.
                map.set_cliff_state(i % w, i / w, TRUE);
                map.set_cliff_state(i % w, i / w, Bool::from(c == b'1'));
            }
            map
        };
        let mut flags = String::new();
        for y in -1..=h {
            for x in -1..=w {
                flags.push(if map.get_cliff_state(x, y) != FALSE {
                    '1'
                } else {
                    '0'
                });
            }
        }
        assert_eq!(
            flags,
            get_str(line, "flags").expect("flags"),
            "vector {index}"
        );
        let answers: String = numbers(get_str(line, "points").expect("points"))
            .chunks(2)
            .map(|p| {
                if map.is_cliff_cell(real(p[0]), real(p[1])) != FALSE {
                    '1'
                } else {
                    '0'
                }
            })
            .collect();
        assert_eq!(
            answers,
            get_str(line, "answers").expect("answers"),
            "vector {index}"
        );
    }
}

#[test]
fn potential_collisions_match_reference() {
    const NAMES: [&str; 6] = [
        "P279_S0", "P279_S1", "P279_S2", "P279_U2", "P279_U0", "P279_G3",
    ];
    let vectors = lines("overlap");
    assert_eq!(vectors.len(), 200);
    let mut sim = context();
    for (index, line) in vectors.iter().enumerate() {
        new_world(&mut sim);
        for o in split_list(get_str(line, "objects").expect("objects"), ';') {
            let o = numbers(o);
            spawn(
                &mut sim,
                NAMES[o[0] as usize],
                1,
                point(&o[1..4]),
                real(o[4]),
            );
        }
        sim.update_partition();
        let queries = split_list(get_str(line, "queries").expect("queries"), ';');
        let answers: Vec<&str> = get_str(line, "answers")
            .expect("answers")
            .split(';')
            .collect();
        assert_eq!(queries.len(), answers.len());
        for (k, (query, want)) in queries.iter().zip(&answers).enumerate() {
            let q = numbers(query);
            let found: String = sim
                .find_potential_collisions(
                    &point(&q[4..7]),
                    &geometry(&q[0..4]),
                    real(q[7]),
                    q[8] as Bool,
                )
                .into_iter()
                .map(|s| format!("{}.", sim.world.objects.get(s).unwrap().id.0))
                .collect();
            assert_eq!(&found, want, "vector {index} query {k}");
        }
    }
}

#[test]
fn oracle_structure() {
    // The structure of the oracle: every kind is present and the header carries the fixtures.
    let header = VECTORS.lines().next().expect("header");
    assert_eq!(get_u32(header, "vector_count"), Some(2070));
    assert_eq!(get_u32(header, "bridge_layer"), Some(2));
    assert!(text("ini").contains("Behavior = GenerateMinefieldBehavior"));
}
