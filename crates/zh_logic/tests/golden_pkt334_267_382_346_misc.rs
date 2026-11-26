//! `PKT-334` `HelicopterSlowDeathBehavior`, `PKT-267` `DumbProjectileBehavior`,
//! `PKT-382` `TensileFormationUpdate` and `PKT-346` `OCLUpdate` against vectors from the
//! reference modules (`tests/cpp_harness/gen_pkt334_267_382_346_misc.cpp`).
//!
//! The rules the reference vectors cannot observe (the pathfind walls, the bonus flags of
//! a detonation, the drop zone reset order) are checked without a reference.

use std::collections::BTreeSet;

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{
    Bool, NameKeyType, ObjectId, Relationship, TeamId, UpdateSleepTime, VeterancyLevel, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::behavior::dumb_projectile_behavior::DumbProjectileBehavior;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, ModuleData, ModuleType, ParkingPlaceFacet, UpdateFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::helicopter_slow_death_update::HelicopterSlowDeathBehavior;
use zh_logic::modules::update::ocl_update::OclUpdate;
use zh_logic::modules::update::physics_update;
use zh_logic::modules::update::tensile_formation_update::TensileFormationUpdate;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const TEAM_NEUTRAL: TeamId = TeamId(3);
/// Teams by player index.
const TEAMS: [TeamId; 3] = [TEAM_NEUTRAL, TeamId(1), TeamId(2)];

/// The generator's `StepTerrain`.
struct StepTerrain;

impl ThingTerrainAccess for StepTerrain {
    fn get_ground_height(&self, x: f32, y: f32, normal: Option<&mut Coord3D>) -> f32 {
        if let Some(n) = normal {
            n.x = ((x / 25.0) as i32 % 5) as f32 * 0.1;
            n.y = ((y / 35.0) as i32 % 5) as f32 * -0.1;
            n.z = 1.0 - (((x + y) / 45.0) as i32 % 4) as f32 * 0.05;
        }
        x * 0.01
            + y * 0.02
            + ((x / 40.0) as i32 % 3) as f32 * 4.0
            + ((y / 60.0) as i32 % 4) as f32 * 3.0
    }

    fn align_on_terrain(
        &self,
        angle: f32,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        let mut pos = *pos;
        if stick_to_ground != FALSE {
            pos.z = self.get_ground_height(pos.x, pos.y, None);
        }
        zh_logic::object::thing::make_align_to_normal_matrix(
            angle,
            &pos,
            &Coord3D::new(0.0, 0.0, 1.0),
            mtx,
        );
        PathfindLayerEnum::Ground
    }

    fn is_underwater(
        &self,
        _x: f32,
        _y: f32,
        _water_z: Option<&mut f32>,
        _terrain_z: Option<&mut f32>,
    ) -> Bool {
        FALSE
    }
}

fn golden() -> &'static str {
    include_str!("../../../docs/golden_vectors/PKT334_267_382_346_Misc.json")
}

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

/// A stand-in for the unported `AIUpdateInterface`: the modules here only need its
/// presence; locomotor settings are kept on the object record.
#[derive(Debug, Clone)]
struct StubAi {
    tag_key: NameKeyType,
    name_key: NameKeyType,
}

impl ModuleData for StubAi {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn facet_mask(&self) -> FacetMask {
        AI_MASK
    }
}

impl BehaviorModule for StubAi {
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn crc(&self, _xfer: &mut dyn Xfer) {}
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }
}

impl UpdateFacet for StubAi {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl AiUpdateFacet for StubAi {}

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(StubAi { tag_key, name_key }))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(
        data.as_any()
            .downcast_ref::<StubAi>()
            .expect("stub")
            .clone(),
    )
}

const PARKING_MASK: FacetMask = FacetMask(0x001 | FacetMask::PARKING_PLACE.0);

/// A stand-in for the unported `ParkingPlaceBehavior`: only the reservation query.
#[derive(Debug, Clone)]
struct StubParking {
    tag_key: NameKeyType,
    name_key: NameKeyType,
    reserved: Option<ObjectId>,
}

impl ModuleData for StubParking {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn facet_mask(&self) -> FacetMask {
        PARKING_MASK
    }
}

impl BehaviorModule for StubParking {
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn crc(&self, _xfer: &mut dyn Xfer) {}
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
        Some(self)
    }
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
    fn as_parking_place(&self) -> Option<&dyn ParkingPlaceFacet> {
        Some(self)
    }
}

impl UpdateFacet for StubParking {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl ParkingPlaceFacet for StubParking {
    fn has_reserved_space(&self, id: ObjectId) -> Bool {
        Bool::from(self.reserved == Some(id))
    }
}

fn parse_parking(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, r| {
        r.next_token_or_null(SEPS_DEFAULT);
        Ok(())
    })?;
    Ok(Box::new(StubParking {
        tag_key,
        name_key,
        reserved: None,
    }))
}

fn create_parking(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(
        data.as_any()
            .downcast_ref::<StubParking>()
            .expect("stub")
            .clone(),
    )
}

/// Loads the generator's INI text, block by block.
fn rules(ini: &str) -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "ParkingPlaceBehavior",
        ModuleType::Behavior,
        PARKING_MASK,
        SleepyUpdatePhase::Normal,
        false,
        parse_parking,
        create_parking,
    );
    let mut objects = String::new();
    let mut block = String::new();
    for line in ini.lines() {
        block.push_str(line);
        block.push('\n');
        if line != "End" {
            continue;
        }
        if block.starts_with("Weapon ") {
            let mut weapon = WeaponTemplate::new();
            weapon.parse_ini_block(&block).expect("weapon");
            rules.weapon_store.add_template(weapon);
        } else if block.starts_with("Locomotor ") {
            let mut locomotor = LocomotorTemplate::new();
            locomotor.parse_ini_block(&block).expect("locomotor");
            rules.locomotor_store.add_template(locomotor);
        } else {
            objects.push_str(&block);
        }
        block.clear();
    }
    let mut r = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut r) {
        panic!("fixture line {}: {e:?}", r.get_line_num());
    }
    rules
}

fn new_world(frame: u32, seed: u32) -> SimWorld {
    let mut world = SimWorld::new();
    world.frame = frame;
    world.rng.init_random_with_seed(seed);
    for (player, &team) in TEAMS.iter().enumerate() {
        world.player_default_teams.insert(player as i32, team);
        world.team_players.insert(team, player as i32);
        world.team_is_neutral.insert(team, Bool::from(player == 0));
    }
    for (team, side) in [(TEAMS[1], "America"), (TEAMS[2], "China")] {
        world.playable_side_teams.insert(team);
        world.team_sides.insert(team, side.to_string());
    }
    world.neutral_team_id = TEAM_NEUTRAL;
    world.local_player_index = 1;
    for (a, b, relationship) in [
        (1, 2, Relationship::Enemies),
        (2, 1, Relationship::Enemies),
        (1, 0, Relationship::Neutral),
        (0, 1, Relationship::Neutral),
        (2, 0, Relationship::Neutral),
        (0, 2, Relationship::Neutral),
    ] {
        world.player_relationships.insert((a, b), relationship);
    }
    world
}

/// The space separated fields of a setup or op string.
struct Fields<'a>(std::str::SplitAsciiWhitespace<'a>);

impl<'a> Fields<'a> {
    fn new(s: &'a str) -> Self {
        Self(s.split_ascii_whitespace())
    }

    fn word(&mut self) -> &'a str {
        self.0.next().expect("field")
    }

    fn num(&mut self) -> u32 {
        self.word().parse().expect("number")
    }

    fn real(&mut self) -> f32 {
        f32::from_bits(self.num())
    }

    fn pos(&mut self) -> Coord3D {
        Coord3D::new(self.real(), self.real(), self.real())
    }
}

/// The generator reseeds after each vector's setup.
fn reseed(sim: &mut SimContext, v: &Vector) {
    sim.world.rng.init_random_with_seed(v.num("seed"));
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D, angle: f32) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = sim.spawn_object(tid, team, Coord3D::default(), 0.0);
    sim.set_object_position(slot, &pos);
    sim.set_object_orientation(slot, angle);
    slot
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> ModuleSlot {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner).expect("object");
    obj.behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
        .unwrap_or_else(|| panic!("missing module {name}"))
}

fn module<T: 'static>(sim: &SimContext, slot: ModuleSlot) -> &T {
    sim.world
        .modules
        .payload(slot)
        .and_then(|p| p.as_any())
        .and_then(|a| a.downcast_ref::<T>())
        .expect("module type")
}

fn save_bytes(sim: &mut SimContext, slot: ModuleSlot) -> (Vec<u8>, u32) {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    sim.world.modules.restore_payload(slot, generation, payload);
    (xs.as_bytes().to_vec(), xc.get_crc())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// "<save> <crc> <update return> <next call frame>"
fn head(sim: &mut SimContext, slot: ModuleSlot, ret: u32) -> String {
    let (bytes, crc) = save_bytes(sim, slot);
    head_of(sim, slot, &bytes, crc, ret)
}

fn head_of(sim: &SimContext, slot: ModuleSlot, bytes: &[u8], crc: u32, ret: u32) -> String {
    let next = sim
        .module_schedule(slot)
        .expect("schedule")
        .next_call_frame();
    format!("{} {crc} {ret} {next}", hex(bytes))
}

/// The projectile save as the reference library writes it: version 1, without the flight
/// path step that version 2 appends.
fn retail_projectile_save(sim: &mut SimContext, slot: ModuleSlot) -> (Vec<u8>, u32, i32) {
    let (mut bytes, crc) = save_bytes(sim, slot);
    let step = module::<DumbProjectileBehavior>(sim, slot).current_flight_path_step;
    if cfg!(feature = "retail-xfer-save") {
        return (bytes, crc, step);
    }
    assert_eq!(bytes[0], 2);
    assert_eq!(bytes.split_off(bytes.len() - 4), step.to_le_bytes());
    bytes[0] = 1;
    (bytes, crc, step)
}

fn update(sim: &mut SimContext, m: ModuleSlot) -> UpdateSleepTime {
    sim.dispatch_module(m, |b, s, c| b.as_update_mut().expect("update").update(s, c))
        .expect("dispatch")
}

fn matrix_of(sim: &SimContext, o: ObjectSlot) -> String {
    let m = sim
        .world
        .objects
        .get(o)
        .expect("object")
        .thing
        .get_transform_matrix();
    m.rows
        .iter()
        .flatten()
        .map(|v| v.to_bits().to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn point(p: &Coord3D) -> String {
    format!("{},{},{}", p.x.to_bits(), p.y.to_bits(), p.z.to_bits())
}

fn record(sim: &SimContext, o: ObjectSlot) -> &zh_logic::sim::ObjectRecord {
    sim.world.objects.get(o).expect("object")
}

fn flag(b: bool) -> char {
    if b {
        '1'
    } else {
        '0'
    }
}

fn condition(sim: &SimContext, o: ObjectSlot, f: ModelConditionFlagType) -> char {
    flag(record(sim, o).model_condition.test(f.as_i32()) != FALSE)
}

fn dashed(s: String) -> String {
    if s.is_empty() {
        "-".to_string()
    } else {
        s
    }
}

fn take_fx(sim: &mut SimContext) -> String {
    dashed(
        sim.world
            .triggered_fx_lists
            .drain(..)
            // The reference `FXList` is client side and drops effects at shrouded
            // positions, which is everywhere off the 1000 x 1000 map.
            .filter(|(_, p)| (0.0..1000.0).contains(&p.x) && (0.0..1000.0).contains(&p.y))
            .map(|(name, pos)| {
                let list: u32 = name.strip_prefix("M_F").expect("FX").parse().expect("FX");
                format!("{list},{};", point(&pos))
            })
            .collect(),
    )
}

/// `with_target`: the reference passed a secondary position.
fn take_ocl(sim: &mut SimContext, with_target: bool) -> String {
    dashed(
        sim.world
            .spawned_ocls
            .drain(..)
            .map(|(name, creator, pos, target)| {
                let list: u32 = name.strip_prefix("M_O").expect("OCL").parse().expect("OCL");
                let secondary = if with_target {
                    point(&target)
                } else {
                    "-".to_string()
                };
                format!("{list},{},{},{secondary};", creator.0, point(&pos))
            })
            .collect(),
    )
}

fn seed_of(sim: &SimContext) -> u32 {
    sim.world.rng.get_game_logic_random_seed_crc()
}

struct Vector<'a> {
    line: &'a str,
    index: usize,
    setup: &'a str,
    ops: Vec<&'a str>,
    rows: Vec<&'a str>,
}

fn strings<'a>(line: &'a str, start: &str, end: &str) -> Vec<&'a str> {
    let rest = &line[line.find(start).expect("list") + start.len()..];
    let body = &rest[..rest.find(end).expect("list end")];
    if body.is_empty() {
        Vec::new()
    } else {
        body[1..body.len() - 1].split("\",\"").collect()
    }
}

impl<'a> Vector<'a> {
    fn new(line: &'a str, index: usize) -> Self {
        Self {
            line,
            index,
            setup: get_str(line, "setup").expect("setup"),
            ops: strings(line, "\"ops\":[", "],\"rows\""),
            rows: strings(line, "\"rows\":[", "]}"),
        }
    }

    fn num(&self, key: &str) -> u32 {
        get_u32(self.line, key).unwrap_or_else(|| panic!("vector {} lacks {key}", self.index))
    }

    fn check(&self, k: usize, actual: &str) {
        let expected = self.rows[k];
        if actual == expected {
            return;
        }
        let diff: Vec<String> = expected
            .split(' ')
            .zip(actual.split(' '))
            .enumerate()
            .filter(|(_, (e, a))| e != a)
            .take(4)
            .map(|(i, (e, a))| format!("field {i}: want {e} got {a}"))
            .collect();
        panic!(
            "vector {} row {k} op '{}' setup '{}': {}",
            self.index,
            self.ops[k],
            self.setup,
            diff.join("; ")
        );
    }
}

fn run_heli(sim: &mut SimContext, v: &Vector) {
    let mut s = Fields::new(v.setup);
    let variant = s.num();
    let team = s.num() as usize;
    let pos = s.pos();
    let angle = s.real();
    let have_bone = s.num() != 0;
    let bone = s.pos();
    let name = format!("M_Heli{variant}");

    sim.world.pristine_bones.clear();
    if have_bone {
        let mut mtx = Matrix3D::identity();
        mtx.set_translation(&bone);
        sim.world
            .pristine_bones
            .insert((name.clone(), "Blade".to_string()), vec![mtx]);
    }
    let o = spawn(sim, &name, TEAMS[team], pos, angle);
    let mut tp = Coord3D::new(10.0, 10.0, 0.0);
    let shrub = spawn(sim, "M_Shrub", TEAM_NEUTRAL, tp, 0.0);
    tp.x = 30.0;
    let rock = spawn(sim, "M_Rock", TEAM_NEUTRAL, tp, 0.0);
    sim.update_partition();
    let m = module_named(sim, o, "HelicopterSlowDeathBehavior");
    let mut known: BTreeSet<ObjectSlot> = sim.world_object_slots().into_iter().collect();
    let mut frame = v.num("frame");
    reseed(sim, v);

    for (k, op) in v.ops.iter().enumerate() {
        let mut f = Fields::new(op);
        let mut ret = 0;
        match f.num() {
            code @ (0 | 1) => {
                frame += f.num();
                sim.world.frame = frame;
                if code == 0 {
                    sim.dispatch_module(m, |b, s, c| {
                        b.as_slow_death_mut().expect("slow death").begin_slow_death(
                            s,
                            c,
                            &DamageInfo::default(),
                        )
                    })
                    .expect("dispatch");
                } else if record(sim, o).is_effectively_dead() == FALSE {
                    kill_object(sim, o, DamageType::Unresistable, DeathType::Normal);
                }
            }
            2 => {
                frame += f.num();
                sim.world.frame = frame;
                ret = update(sim, m).0;
            }
            3 => {
                let id = match f.num() {
                    0 => ObjectId::INVALID,
                    1 => record(sim, shrub).id,
                    _ => record(sim, rock).id,
                };
                physics_update::physics_mut(sim, o)
                    .expect("physics")
                    .last_collidee = id;
            }
            4 => {
                let mut p = *record(sim, o).thing.get_position();
                p.z = f.real();
                sim.set_object_position(o, &p);
            }
            5 => {
                let level = match f.num() {
                    0 => VeterancyLevel::Regular,
                    1 => VeterancyLevel::Veteran,
                    2 => VeterancyLevel::Elite,
                    _ => VeterancyLevel::Heroic,
                };
                sim.object_mut(o)
                    .expect("object")
                    .experience_tracker
                    .current_level = level;
            }
            other => panic!("op {other}"),
        }

        let mut row = format!("{} {} ", head(sim, m, ret), matrix_of(sim, o));
        let physics = physics_update::physics(sim, o).expect("physics");
        row += &format!(
            "{} {} ",
            point(&physics.accel),
            physics.motive_force_expires
        );
        let obj = record(sim, o);
        let activated = module::<HelicopterSlowDeathBehavior>(sim, m)
            .core
            .is_slow_death_activated()
            != FALSE;
        if obj.ai_slot.is_some() && activated {
            row += &format!(
                "{},{} ",
                obj.loco_max_lift.to_bits(),
                obj.loco_max_braking.to_bits()
            );
        } else {
            row += "- ";
        }
        row += &format!(
            "{}{}{} {} ",
            flag(obj.disabled_mask.test(DisabledType::Held as i32) != FALSE),
            condition(sim, o, ModelConditionFlagType::SpecialDamaged),
            flag(obj.is_destroyed()),
            obj.layer as i32
        );
        let mut made = String::new();
        for slot in sim.world_object_slots() {
            if known.insert(slot) {
                let team = record(sim, slot).team_id;
                let player = TEAMS.iter().position(|&t| t == team).expect("team");
                made += &format!("{player},{};", matrix_of(sim, slot));
            }
        }
        let fx = take_fx(sim);
        let ocl = take_ocl(sim, false);
        row += &format!("{} {fx} {ocl} {}", dashed(made), seed_of(sim));
        v.check(k, &row);
    }
}

fn fnv(path: &[Coord3D]) -> u32 {
    let mut h: u32 = 2_166_136_261;
    for p in path {
        for x in [p.x.to_bits(), p.y.to_bits(), p.z.to_bits()] {
            h = (h ^ x).wrapping_mul(16_777_619);
        }
    }
    h
}

fn run_projectile(sim: &mut SimContext, v: &Vector) {
    let mut s = Fields::new(v.setup);
    let variant = s.num();
    let proj_team = s.num() as usize;
    let base = s.pos();
    let angle = s.real();
    let launcher_kind = s.num();
    let launcher_team = s.num() as usize;
    let lp = s.pos();
    let langle = s.real();
    let vel = s.pos();
    let flag_bits = s.num();
    let target_kind = s.num();
    let target_team = s.num() as usize;
    let tp = s.pos();
    let burned = s.num() != 0;
    let victim_team = s.num() as usize;
    let vp = s.pos();
    let reserved = s.num() != 0;
    let soldier_text = s.word();
    let contained = s.num() != 0;
    let proj_name = format!("M_Proj{variant}");

    let launcher = spawn(
        sim,
        &format!("M_Launcher{launcher_kind}"),
        TEAMS[launcher_team],
        lp,
        langle,
    );
    if let Some(p) = physics_update::physics_mut(sim, launcher) {
        p.vel = vel;
    }
    sim.object_mut(launcher)
        .expect("launcher")
        .weapon_bonus_condition |= flag_bits;

    let target = spawn(
        sim,
        &format!("M_Target{target_kind}"),
        TEAMS[target_team],
        tp,
        0.0,
    );
    if burned {
        sim.set_status(target, ObjectStatusType::Burned);
    }
    let victim = spawn(sim, "M_Target3", TEAMS[victim_team], vp, 0.0);
    if reserved {
        let id = record(sim, victim).id;
        let parking = module_named(sim, target, "ParkingPlaceBehavior");
        sim.dispatch_module(parking, |m, _, _| {
            m.as_any_mut()
                .and_then(|a| a.downcast_mut::<StubParking>())
                .expect("parking")
                .reserved = Some(id);
        })
        .expect("dispatch");
    }

    let mut soldiers = Vec::new();
    let mut parts = soldier_text
        .split(',')
        .map(|n| n.parse::<u32>().expect("n")); // determinism-ok: generator-written integers
    let count = parts.next().expect("count");
    for _ in 0..count {
        let kind = parts.next().expect("kind");
        let team = parts.next().expect("team") as usize;
        let soldier = spawn(sim, &format!("M_Soldier{kind}"), TEAMS[team], tp, 0.0);
        sim.contain_add(target, soldier);
        soldiers.push(soldier);
    }
    if contained {
        sim.contain_add(target, launcher);
    }

    sim.update_partition();

    let o = spawn(sim, &proj_name, TEAMS[proj_team], base, angle);
    let mut current = o;
    let mut m = module_named(sim, o, "DumbProjectileBehavior");
    let mut frame = v.num("frame");
    reseed(sim, v);

    for (k, op) in v.ops.iter().enumerate() {
        let mut f = Fields::new(op);
        let mut ret = 0;
        sim.world.frame = frame;
        match f.num() {
            code @ (0 | 1) => {
                let vic = match f.num() {
                    0 => None,
                    1 => Some(victim),
                    _ => Some(target),
                };
                let aim = f.pos();
                let weapon = f.num();
                let with_launcher = f.num() != 0;
                let name = (weapon < 8).then(|| format!("M_W{weapon}"));
                sim.dispatch_module(m, |b, s, c| {
                    let p = b.as_projectile_mut().expect("projectile");
                    if code == 0 {
                        p.projectile_launch_at_object_or_position(
                            s,
                            c,
                            vic,
                            &aim,
                            with_launcher.then_some(launcher),
                            WeaponSlotType::Primary,
                            0,
                            name.as_deref(),
                            None,
                        );
                    } else {
                        p.projectile_fire_at_object_or_position(
                            s,
                            c,
                            vic,
                            &aim,
                            name.as_deref(),
                            None,
                        );
                    }
                })
                .expect("dispatch");
            }
            2 => {
                frame += f.num();
                sim.world.frame = frame;
                ret = update(sim, m).0;
            }
            3 => {
                let p = f.pos();
                sim.set_object_position(victim, &p);
            }
            6 => {
                let layer = match f.num() {
                    1 => PathfindLayerEnum::Ground,
                    2 => PathfindLayerEnum::Bridge1,
                    _ => PathfindLayerEnum::Bridge2,
                };
                sim.set_object_layer(current, layer);
            }
            4 => {
                let other = match f.num() {
                    0 => None,
                    1 => Some(target),
                    2 => Some(victim),
                    3 => Some(launcher),
                    _ => Some(soldiers[0]),
                };
                if !record(sim, current).is_destroyed() {
                    ret = sim
                        .dispatch_module(m, |b, s, c| {
                            b.as_projectile_mut()
                                .expect("projectile")
                                .projectile_handle_collision(s, c, other)
                        })
                        .expect("dispatch") as u32;
                }
            }
            5 => {
                // Version 2 restores the step on load.
                let (full, _) = save_bytes(sim, m);
                let step = module::<DumbProjectileBehavior>(sim, m).current_flight_path_step;
                let (bytes, _, _) = retail_projectile_save(sim, m);
                let copy = spawn(sim, &proj_name, TEAMS[proj_team], base, angle);
                let cm = module_named(sim, copy, "DumbProjectileBehavior");
                let (mut payload, ctx, generation) =
                    sim.world.modules.take_payload(cm).expect("payload");
                payload
                    .xfer(&mut XferLoad::new(&full), sim)
                    .expect("xfer load");
                let loaded = payload
                    .as_any()
                    .and_then(|a| a.downcast_ref::<DumbProjectileBehavior>())
                    .expect("projectile")
                    .current_flight_path_step;
                if !cfg!(feature = "retail-xfer-save") {
                    assert_eq!(loaded, step, "vector {}", v.index);
                }
                payload
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<DumbProjectileBehavior>())
                    .expect("projectile")
                    .current_flight_path_step = 0;
                // The reference loads its own version 1 bytes.
                payload
                    .xfer(&mut XferLoad::new(&bytes), sim)
                    .expect("xfer load");
                payload.load_post_process(sim, ctx);
                sim.world.modules.restore_payload(cm, generation, payload);
                let mtx = *record(sim, current).thing.get_transform_matrix();
                let pos = *record(sim, current).thing.get_position();
                sim.set_object_transform_matrix(copy, &mtx);
                sim.set_object_position(copy, &pos);
                current = copy;
                m = cm;
            }
            other => panic!("op {other}"),
        }

        let (bytes, crc, step) = retail_projectile_save(sim, m);
        let mut row = format!(
            "{} {} {} {step} ",
            head_of(sim, m, &bytes, crc, ret),
            matrix_of(sim, current),
            record(sim, current).layer as i32
        );
        let current_id = record(sim, current).id;
        let (data_flags, detonated, path_len, path_hash) = {
            let d = module::<DumbProjectileBehavior>(sim, m);
            (
                d.extra_bonus_flags,
                d.has_detonated != FALSE,
                d.flight_path.len(),
                fnv(&d.flight_path),
            )
        };
        let obj = record(sim, current);
        row += &format!(
            "{}{}{}{} ",
            flag(obj.is_destroyed()),
            flag(obj.is_effectively_dead() != FALSE),
            flag(obj.status.test(ObjectStatusType::NoCollisions.as_i32()) != FALSE),
            flag(detonated)
        );
        let sink = obj.experience_tracker.experience_sink.0;
        row += &format!("{} ", sim.object_health(current).to_bits());
        row += &format!("{path_len},{path_hash} ");
        match physics_update::physics(sim, current) {
            Some(p) => {
                row += &format!(
                    "{},{},{},{},{} ",
                    point(&p.vel),
                    p.ignore_collisions_with.0,
                    p.pitch_rate.to_bits(),
                    p.yaw_rate.to_bits(),
                    p.roll_rate.to_bits()
                )
            }
            None => row += "- ",
        }
        row += &format!("{sink} {data_flags} ");

        let mut shots: Vec<(u32, String)> = sim
            .world
            .projectile_detonations
            .drain(..)
            .map(|(source, name, pos, bonus)| {
                // The reference cannot show these two.
                assert_eq!(source, current_id, "vector {}", v.index);
                assert_eq!(bonus, data_flags, "vector {}", v.index);
                let w: u32 = name
                    .strip_prefix("M_W")
                    .expect("weapon")
                    .parse()
                    .expect("w");
                (w, format!("{w},{};", point(&pos)))
            })
            .collect();
        shots.sort_by_key(|s| s.0);
        row += &format!("{} ", dashed(shots.into_iter().map(|s| s.1).collect()));
        let dead: String = soldiers
            .iter()
            .map(|&s| flag(record(sim, s).is_effectively_dead() != FALSE))
            .collect();
        row += &format!("{} ", dashed(dead));
        for player in 0..3 {
            row += &sim
                .world
                .cleared_garrisons
                .get(&player)
                .copied()
                .unwrap_or(0)
                .to_string();
        }
        row += &format!(" {} {}", take_fx(sim), seed_of(sim));
        v.check(k, &row);
    }
}

/// Weapon fires other than detonations would be a bug.
fn assert_quiet(sim: &SimContext, index: usize) {
    assert!(sim.world.weapon_fires.is_empty(), "vector {index}");
}

fn run_tensile(sim: &mut SimContext, v: &Vector) {
    let mut s = Fields::new(v.setup);
    let members = s.num() as usize;
    let mut all = Vec::new();
    let mut mods = Vec::new();
    for _ in 0..members {
        let kind = s.num();
        let pos = s.pos();
        let o = spawn(sim, &format!("M_Tensile{kind}"), TEAM_NEUTRAL, pos, 0.0);
        mods.push(module_named(sim, o, "TensileFormationUpdate"));
        all.push(o);
    }
    let mut trees = Vec::new();
    for _ in 0..s.num() {
        let kind = s.num();
        let pos = s.pos();
        let tree = spawn(sim, &format!("M_Tree{kind}"), TEAM_NEUTRAL, pos, 0.0);
        trees.push((tree, module_named(sim, tree, "ToppleUpdate")));
    }
    sim.update_partition();
    reseed(sim, v);

    for (k, op) in v.ops.iter().enumerate() {
        let mut f = Fields::new(op);
        let code = f.num();
        let who = f.num() as usize;
        let mut ret = 0;
        sim.world.frame = v.num("frame");
        match code {
            0 => {
                let amount = [10.0, 49.0, 51.0, 75.0, 95.0, 200.0][f.num() as usize];
                let mut dmg = DamageInfo::default();
                dmg.input.damage_type = DamageType::Unresistable;
                dmg.input.death_type = DeathType::Normal;
                dmg.input.amount = amount;
                sim.attempt_damage(all[who], &mut dmg);
            }
            1 => {
                for _ in 0..f.num() {
                    ret = update(sim, mods[who]).0;
                    sim.update_partition();
                    if ret == UpdateSleepTime::FOREVER.0 {
                        break;
                    }
                }
            }
            2 => {
                let p = f.pos();
                sim.set_object_position(all[who], &p);
                sim.update_partition();
            }
            other => panic!("op {other}"),
        }

        let o = all[who];
        let mut row = format!(
            "{} {} {}{}{} ",
            head(sim, mods[who], ret),
            matrix_of(sim, o),
            condition(sim, o, ModelConditionFlagType::Moving),
            condition(sim, o, ModelConditionFlagType::Freefall),
            condition(sim, o, ModelConditionFlagType::PostCollapse),
        );
        for &x in &all {
            row += &format!(
                "{},{};",
                sim.object_health(x).to_bits(),
                sim.object_body_damage_state(x) as i32
            );
        }
        row.push(' ');
        let mut tree_text = String::new();
        for &(_, tm) in &trees {
            let (bytes, _) = save_bytes(sim, tm);
            let h = hex(&bytes).bytes().fold(2_166_136_261u32, |h, c| {
                (h ^ u32::from(c)).wrapping_mul(16_777_619)
            });
            tree_text += &format!("{h};");
        }
        row += &format!("{} {}", dashed(tree_text), seed_of(sim));
        v.check(k, &row);

        // Walls: made at construction, removed when damage sets a disabled one off, and
        // made again when the slide ends.
        for (i, (&o, &m)) in all.iter().zip(&mods).enumerate() {
            let t = module::<TensileFormationUpdate>(sim, m);
            let sliding = t.data.enabled == FALSE && t.enabled == TRUE && t.life <= 300;
            assert_eq!(
                sim.world.pathfind_walls.contains(&record(sim, o).id),
                !sliding,
                "vector {} row {k} member {i}",
                v.index
            );
        }
    }
}

fn run_ocl(sim: &mut SimContext, v: &Vector) {
    let mut s = Fields::new(v.setup);
    let variant = s.num();
    let team = s.num() as usize;
    let pos = s.pos();
    let angle = s.real();
    for team in TEAMS {
        let color = s.num() as i32;
        sim.world.team_colors.insert(team, color);
    }
    let o = spawn(sim, &format!("M_Ocl{variant}"), TEAMS[team], pos, angle);
    let m = module_named(sim, o, "OCLUpdate");
    let mut frame = v.num("frame");
    reseed(sim, v);

    for (k, op) in v.ops.iter().enumerate() {
        let mut f = Fields::new(op);
        let mut ret = 0;
        sim.world.frame = frame;
        match f.num() {
            // The sabotage path: `OCLUpdate::resetTimer` through the object.
            0 => sim.reset_supply_dropzone_timer(o),
            1 => {
                if f.num() != 0 {
                    sim.set_disabled(o, DisabledType::Held);
                } else {
                    sim.clear_disabled(o, DisabledType::Held);
                }
            }
            2 => {
                let team = TEAMS[f.num() as usize];
                sim.set_object_team(o, team);
            }
            3 => {
                let team = TEAMS[f.num() as usize];
                let color = f.num() as i32;
                sim.world.team_colors.insert(team, color);
            }
            4 => {
                if f.num() != 0 {
                    sim.set_status(o, ObjectStatusType::UnderConstruction);
                } else {
                    sim.clear_status(o, ObjectStatusType::UnderConstruction);
                }
            }
            5 => {
                frame += f.num();
                sim.world.frame = frame;
                ret = update(sim, m).0;
            }
            other => panic!("op {other}"),
        }
        let mut row = head(sim, m, ret);
        let ocl = module::<OclUpdate>(sim, m);
        row += &format!(
            " {} {} ",
            ocl.countdown_percent(sim).to_bits(),
            ocl.remaining_frames(sim)
        );
        let made = take_ocl(sim, true);
        row += &format!("{made} {}", seed_of(sim));
        v.check(k, &row);
    }
}

fn fixture_text() -> String {
    let line = golden()
        .lines()
        .find(|l| l.starts_with("\"ini\":\""))
        .expect("ini line");
    line["\"ini\":\"".len()..line.rfind('"').expect("quote")].replace("\\n", "\n")
}

fn context() -> SimContext {
    let mut sim = SimContext::new();
    sim.rules = rules(&fixture_text());
    sim.terrain = Box::new(StepTerrain);
    sim
}

#[test]
fn misc_modules_match_reference() {
    let lines: Vec<&str> = golden()
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"kind\""))
        .collect();
    assert_eq!(lines.len(), 1600);
    let mut sim = context();
    for (index, &line) in lines.iter().enumerate() {
        let v = Vector::new(line, index);
        assert_eq!(v.ops.len(), v.rows.len(), "vector {index}");
        sim.world = new_world(v.num("frame"), v.num("seed"));
        sim.init_partition(
            20.0,
            Region3D {
                lo: Coord3D::new(0.0, 0.0, 0.0),
                hi: Coord3D::new(1000.0, 1000.0, 100.0),
            },
        );
        match v.num("kind") {
            0 => run_heli(&mut sim, &v),
            1 => run_projectile(&mut sim, &v),
            2 => run_tensile(&mut sim, &v),
            _ => run_ocl(&mut sim, &v),
        }
        assert_quiet(&sim, index);
    }
}
