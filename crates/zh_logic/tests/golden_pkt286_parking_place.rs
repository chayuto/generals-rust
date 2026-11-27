//! `PKT-286` `ParkingPlaceBehavior` against vectors from the reference module
//! (`tests/cpp_harness/gen_pkt286_parking_place.cpp`), with the real `ProductionUpdate`.
//!
//! The AI modules are stand-ins that record their orders. Loading a save and the module's
//! use through the production exit seams are checked here without a reference.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, NameKeyType, ObjectId, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot, ThingTemplateId};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::behavior::parking_place_behavior::{
    ParkingPlaceBehavior, ParkingPlaceBehaviorModuleData,
};
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CommandSourceType, ExitDoorType, JetAiState, ModuleData,
    ModuleType, ParkingPlaceInfo, RunwayReservationType,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

// Neutral, A and B are players 0, 1 and 2.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];

#[rustfmt::skip]
const LAYOUTS: [(i32, i32, bool, bool, &str, &str, &str, bool); 8] = [
    (2, 2, true, false, "5", "50", "0", true),   (2, 2, true, true, "10", "0", "0", true),
    (2, 1, true, false, "0", "20", "15", false), (1, 2, true, false, "3.3", "35.5", "-7", true),
    (4, 1, true, true, "7", "10", "3", true),    (0, 0, false, false, "12", "0", "0", false),
    (1, 4, true, false, "2", "0", "0", true),    (3, 1, true, false, "1000", "5", "0", true),
];

// (extra kind of, AI: 0 none, 1 plain, 2 jet, parking offset)
#[rustfmt::skip]
const CRAFTS: [(&str, u8, &str); 6] = [
    ("", 2, "0"),  ("", 2, "-12.5"),
    ("", 1, ""),   (" PRODUCED_AT_HELIPAD", 1, ""),
    ("", 0, ""),   (" PRODUCED_AT_HELIPAD", 2, "7.25"),
];

thread_local! {
    static ORDERS: RefCell<String> = const { RefCell::new(String::new()) }; // determinism-ok: test stand-in recorder
    static JET_FLAGS: RefCell<BTreeMap<ObjectId, u32>> = const { RefCell::new(BTreeMap::new()) }; // determinism-ok: test stand-in recorder
}

#[derive(Debug, Clone)]
struct AiData {
    tag: NameKeyType,
    name: NameKeyType,
    jet: bool,
    parking_offset: f32,
}

impl ModuleData for AiData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag
    }
    fn module_name_key(&self) -> NameKeyType {
        self.name
    }
    fn facet_mask(&self) -> FacetMask {
        AI_MASK
    }
}

struct Ai {
    data: AiData,
    owner: ObjectId,
}

fn coord(c: &Coord3D) -> String {
    format!("{},{},{}", c.x.to_bits(), c.y.to_bits(), c.z.to_bits())
}

fn order(text: String) {
    ORDERS.with_borrow_mut(|o| o.push_str(&text));
}

impl AiUpdateFacet for Ai {
    fn jet_ai(&self) -> Option<JetAiState> {
        self.data.jet.then(|| JetAiState {
            takeoff_or_landing_in_progress: Bool::from(
                JET_FLAGS.with_borrow(|f| f.get(&self.owner).copied().unwrap_or(0)) != 0,
            ),
            parking_offset: self.data.parking_offset,
        })
    }
    fn ai_move_to_position(
        &mut self,
        _sim: &mut dyn SimApi,
        ctx: ModuleContext,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    ) {
        order(format!(
            "{{{}:1:{}:{}}}",
            ctx.owner_id.0,
            cmd_source as i32,
            coord(pos)
        ));
    }
    fn ai_idle(
        &mut self,
        _sim: &mut dyn SimApi,
        ctx: ModuleContext,
        cmd_source: CommandSourceType,
    ) {
        order(format!("{{{}:3:{}}}", ctx.owner_id.0, cmd_source as i32));
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.data.name
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.owner = ctx.owner_id;
        sim.ai_construct(ctx.owner_slot);
        // The state machine the C++ AI builds draws four times; the recording AI never lets
        // an order reach it, so it is not marked ready.
        for _ in 0..4 {
            let _ = sim.rng_logic_int(0, 1);
        }
    }
    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_ai(&self) -> Option<&dyn AiUpdateFacet> {
        Some(self)
    }
    fn as_ai_mut(&mut self) -> Option<&mut dyn AiUpdateFacet> {
        Some(self)
    }
}

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

fn parse_ai_data(
    reader: &mut IniReader<'_>,
    mut data: AiData,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|field, r| {
        if field == "ParkingOffset" {
            r.parse_real(&mut data.parking_offset)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

fn parse_plain(
    r: &mut IniReader<'_>,
    tag: NameKeyType,
    name: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_ai_data(
        r,
        AiData {
            tag,
            name,
            jet: false,
            parking_offset: 0.0,
        },
    )
}

fn parse_jet(
    r: &mut IniReader<'_>,
    tag: NameKeyType,
    name: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_ai_data(
        r,
        AiData {
            tag,
            name,
            jet: true,
            parking_offset: 0.0,
        },
    )
}

fn create_ai(data: &dyn ModuleData, ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(Ai {
        data: data
            .as_any()
            .downcast_ref::<AiData>()
            .expect("AI data")
            .clone(),
        owner: ctx.owner_id,
    })
}

fn yes_no(v: bool) -> &'static str {
    if v {
        "Yes"
    } else {
        "No"
    }
}

fn fixture() -> String {
    let mut s = String::new();
    for (i, (rows, cols, runways, hangars, heal, approach, deck, prod)) in
        LAYOUTS.iter().enumerate()
    {
        let _ = write!(
            s,
            "Object P286_Field{i}\n  KindOf = PRELOAD STRUCTURE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 1000\n    InitialHealth = 1000\n  End\n"
        );
        if *prod {
            s.push_str(
                "  Behavior = ProductionUpdate ModuleTag_Prod\n    NumDoorAnimations = 4\n    DoorOpeningTime = 100\n    DoorWaitOpenTime = 200\n    DoorCloseTime = 100\n  End\n",
            );
        }
        let _ = write!(
            s,
            "  Behavior = ParkingPlaceBehavior ModuleTag_Park\n    NumRows = {rows}\n    NumCols = {cols}\n    HasRunways = {}\n    ParkInHangars = {}\n    HealAmountPerSecond = {heal}\n    ApproachHeight = {approach}\n    LandingDeckHeightOffset = {deck}\n  End\nEnd\n",
            yes_no(*runways),
            yes_no(*hangars)
        );
    }
    for (i, (kind, ai, offset)) in CRAFTS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P286_Craft{i}\n  KindOf = PRELOAD VEHICLE AIRCRAFT{kind}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
        );
        if *ai == 1 {
            s.push_str("  Behavior = OracleParkAI ModuleTag_AI\n  End\n");
        }
        if *ai == 2 {
            let _ = write!(
                s,
                "  Behavior = OracleParkJet ModuleTag_AI\n    ParkingOffset = {offset}\n  End\n"
            );
        }
        s.push_str("End\n");
    }
    s
}

fn context() -> SimContext {
    let mut rules = RulesRegistry::new();
    for (name, jet) in [("OracleParkAI", false), ("OracleParkJet", true)] {
        rules.module_registry.register_module(
            &mut rules.name_keys,
            name,
            ModuleType::Behavior,
            AI_MASK,
            SleepyUpdatePhase::Initial,
            true,
            if jet { parse_jet } else { parse_plain },
            create_ai,
        );
    }
    let ini = fixture();
    rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "object.ini",
            ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("templates");
    let mut sim = SimContext::new();
    sim.rules = rules;
    sim
}

fn reset(sim: &mut SimContext) {
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as i32);
        w.player_default_teams.insert(player as i32, team);
    }
    w.team_is_neutral.insert(TEAMS[0], TRUE);
    w.neutral_team_id = TEAMS[0];
    w.local_player_index = 5;
    for i in 0..3 {
        for j in 0..3 {
            let rel = if i == j {
                Relationship::Allies
            } else if i == 0 || j == 0 {
                Relationship::Neutral
            } else {
                Relationship::Enemies
            };
            w.player_relationships.insert((i, j), rel);
        }
    }
    ORDERS.with_borrow_mut(String::clear);
    JET_FLAGS.with_borrow_mut(BTreeMap::clear);
}

fn template(sim: &SimContext, name: &str) -> ThingTemplateId {
    sim.rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"))
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.name_to_key(name);
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

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_bits(s: &str) -> f32 {
    f32::from_bits(s.parse().expect("float bits"))
}

struct Scene {
    field: ObjectSlot,
    field_id: ObjectId,
    park: ModuleSlot,
    prod: Option<ModuleSlot>,
    crafts: Vec<(ObjectId, ObjectSlot, usize)>,
}

/// The harness's row, before its fields are compared with the previous one.
fn row(sim: &mut SimContext, sc: &Scene, ret: &str) -> Vec<String> {
    let mut out = vec![hex(&save(sim, sc.park))];
    let mut xc = XferCrc::new();
    sim.world
        .modules
        .payload(sc.park)
        .expect("payload")
        .crc(&mut xc);
    out.push(xc.get_crc().to_string());
    out.push(match sc.prod {
        Some(p) => hex(&save(sim, p)),
        None => "-".to_owned(),
    });
    use ModelConditionFlagType as F;
    let field = sim.world.objects.get(sc.field).expect("field");
    out.push(
        [
            F::Door1Opening,
            F::Door2Opening,
            F::Door3Opening,
            F::Door4Opening,
        ]
        .iter()
        .map(|f| {
            if field.model_condition.test(f.as_i32()) != 0 {
                '1'
            } else {
                '0'
            }
        })
        .collect(),
    );
    out.push(sim.world.rng.get_game_logic_random_seed_crc().to_string());
    out.push(if ret.is_empty() { "-" } else { ret }.to_owned());
    let mut orders = ORDERS.with_borrow_mut(std::mem::take);
    for o in std::mem::take(&mut sim.world.exit_path_orders) {
        let _ = write!(
            orders,
            "{{{}:2:{}:{}",
            o.object.0, o.cmd_source as i32, o.ignore.0
        );
        for c in &o.path {
            let _ = write!(orders, ":{}", coord(c));
        }
        orders.push('}');
    }
    sim.world.ai_commands.clear();
    out.push(if orders.is_empty() {
        "-".to_owned()
    } else {
        orders
    });
    let crafts: Vec<String> = sc
        .crafts
        .iter()
        .map(|&(id, _, _)| {
            let Some(slot) = sim.find_object(id) else {
                return "x".to_owned();
            };
            let o = sim.world.objects.get(slot).expect("craft");
            format!(
                "{},{},{},{},{},{},{}",
                i32::from(o.is_effectively_dead() != FALSE),
                coord(o.thing.get_position()),
                o.thing.get_orientation().to_bits(),
                sim.object_health(slot).to_bits(),
                o.producer_id.0,
                i32::from(o.status.test(ObjectStatusType::DeckHeightOffset.as_i32()) != FALSE),
                sim.player_index(o.team_id)
            )
        })
        .collect();
    out.push(crafts.join(";"));
    out
}

fn assert_row(actual: &[String], last: &mut Vec<String>, expected: &str, what: &str) {
    let expected: Vec<&str> = expected.split(' ').collect();
    assert_eq!(actual.len(), expected.len(), "{what}: {expected:?}");
    let names = [
        "save",
        "crc",
        "production",
        "doors",
        "rng",
        "result",
        "orders",
        "crafts",
    ];
    for (i, (a, e)) in actual.iter().zip(&expected).enumerate() {
        let e = if *e == "~" { last[i].as_str() } else { e };
        assert_eq!(a, e, "{what}: {}", names[i]);
    }
    *last = actual.to_vec();
}

fn info_text(p: &ParkingPlaceInfo) -> String {
    [
        coord(&p.parking_space),
        p.parking_orientation.to_bits().to_string(),
        coord(&p.runway_prep),
        coord(&p.runway_start),
        coord(&p.runway_end),
        coord(&p.runway_exit),
        coord(&p.runway_landing_start),
        coord(&p.runway_landing_end),
        coord(&p.runway_approach),
        coord(&p.hangar_internal),
        p.runway_takeoff_dist.to_bits().to_string(),
        p.hangar_internal_orient.to_bits().to_string(),
    ]
    .join(",")
}

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

/// Runs `f` with the parking facet of the field.
fn parking<R>(
    sim: &mut SimContext,
    park: ModuleSlot,
    f: impl FnOnce(&mut dyn BehaviorModule, &mut dyn SimApi, ModuleContext) -> R,
) -> R {
    sim.dispatch_module(park, f).expect("dispatch")
}

macro_rules! pp {
    ($sim:expr, $sc:expr, $p:ident, $s:ident, $c:ident, $body:expr) => {
        parking($sim, $sc.park, |m, $s, $c| {
            let $p = m.as_parking_place_mut().expect("parking");
            $body
        })
    };
}

macro_rules! exit {
    ($sim:expr, $sc:expr, $p:ident, $s:ident, $c:ident, $body:expr) => {
        parking($sim, $sc.park, |m, $s, $c| {
            let $p = m.as_exit_mut().expect("exit");
            $body
        })
    };
}

#[allow(clippy::too_many_lines)]
fn apply(sim: &mut SimContext, sc: &Scene, frame: &mut u32, op: [i64; 4]) -> String {
    let [code, a, b, c] = op;
    // `a` is the aircraft, except for the frame count of a step.
    let who = if code == 12 { 0 } else { a as usize };
    let (id, slot, kind) = sc.crafts[who];
    let tmpl = template(sim, &format!("P286_Craft{kind}"));
    let b32 = b as i32;
    let c32 = c as i32;
    sim.world.frame = *frame;
    match code {
        0 => exit!(sim, sc, e, s, ctx, {
            e.reserve_door_for_exit(s, ctx, tmpl, None)
                .as_i32()
                .to_string()
        }),
        1 => {
            let door = ExitDoorType::from_i32(b32).expect("door");
            exit!(
                sim,
                sc,
                e,
                s,
                ctx,
                e.exit_object_via_door(s, ctx, slot, door)
            );
            String::new()
        }
        2 => {
            let door = ExitDoorType::from_i32(b32).expect("door");
            exit!(sim, sc, e, _s, _ctx, e.unreserve_door_for_exit(door));
            String::new()
        }
        3 => {
            let query = if b != 0 { id } else { ObjectId::INVALID };
            let busy = exit!(sim, sc, e, s, ctx, e.is_exit_busy(&*s, ctx));
            pp!(sim, sc, p, s, _ctx, {
                format!(
                    "{}{}{},{},{},{},{},{busy}",
                    p.has_available_space_for(&*s, tmpl),
                    p.should_reserve_door_when_queued(&*s, tmpl),
                    p.has_reserved_space(query),
                    p.get_space_index(query),
                    p.get_runway_count(),
                    p.get_approach_height().to_bits(),
                    p.get_landing_deck_height_offset().to_bits()
                )
            })
        }
        4 => {
            let offset = [0.0f32, 0.0, 5.5, -20.0][b as usize];
            let mut info = ParkingPlaceInfo::default();
            let want = c != 0;
            let ok = pp!(sim, sc, p, s, ctx, {
                p.reserve_space(s, ctx, id, offset, want.then_some(&mut info))
            });
            if want {
                format!("{ok},{}", info_text(&info))
            } else {
                ok.to_string()
            }
        }
        5 => {
            pp!(sim, sc, p, s, ctx, p.release_space(s, ctx, id));
            String::new()
        }
        6 => pp!(
            sim,
            sc,
            p,
            s,
            ctx,
            p.reserve_runway(s, ctx, id, b as Bool).to_string()
        ),
        7 => {
            pp!(sim, sc, p, s, ctx, p.release_runway(s, ctx, id));
            String::new()
        }
        8 => {
            let query = if b != 0 { id } else { ObjectId::INVALID };
            pp!(
                sim,
                sc,
                p,
                s,
                ctx,
                p.get_runway_index(s, ctx, query).to_string()
            )
        }
        9 => {
            let rtype = if c != 0 {
                RunwayReservationType::Landing
            } else {
                RunwayReservationType::Takeoff
            };
            pp!(sim, sc, p, s, ctx, {
                let holder = p.get_runway_reservation(s, ctx, b32, rtype);
                let mut info = ParkingPlaceInfo::default();
                p.calc_pp_info(&*s, ctx, id, &mut info);
                format!("{},{}", holder.0, info_text(&info))
            })
        }
        10 => {
            pp!(sim, sc, p, s, ctx, {
                p.transfer_runway_reservation_to_next_in_line_for_takeoff(s, ctx, id)
            });
            String::new()
        }
        11 => {
            pp!(sim, sc, p, s, ctx, p.set_healee(s, ctx, slot, b as Bool));
            String::new()
        }
        12 => {
            for _ in 0..a {
                *frame += 1;
                sim.world.frame = *frame;
                let sleep = parking(sim, sc.park, |m, s, ctx| {
                    m.as_update_mut().expect("update").update(s, ctx)
                });
                assert_eq!(sleep.0, 1);
            }
            String::new()
        }
        13 => {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Unresistable;
            dmg.input.death_type = DeathType::Normal;
            dmg.input.amount = b as f32;
            sim.attempt_damage(slot, &mut dmg);
            String::new()
        }
        14 => {
            pp!(sim, sc, p, s, ctx, p.kill_all_parked_units(s, ctx));
            String::new()
        }
        15 => {
            let team = TEAMS[b as usize];
            pp!(sim, sc, p, s, ctx, {
                p.defect_all_parked_units(s, ctx, team, c as u32)
            });
            String::new()
        }
        16 => {
            parking(sim, sc.park, |m, s, ctx| {
                m.as_die_mut()
                    .expect("die")
                    .on_die(s, ctx, &DamageInfo::default());
            });
            String::new()
        }
        17 => {
            if b != 0 {
                sim.destroy_object(slot);
                sim.process_destroy_list();
            } else {
                zh_logic::modules::contain::open_contain::kill_object(
                    sim,
                    slot,
                    DamageType::Unresistable,
                    DeathType::Normal,
                );
            }
            String::new()
        }
        18 => {
            let mut p = *sim
                .world
                .objects
                .get(slot)
                .expect("craft")
                .thing
                .get_position();
            p.z = if b != 0 { 40.0 } else { 0.0 };
            sim.set_object_position(slot, &p);
            String::new()
        }
        19 => {
            JET_FLAGS.with_borrow_mut(|f| f.insert(id, b as u32));
            String::new()
        }
        20 => {
            if b == 0 {
                let p = Coord3D::new((c32 / 1000) as f32, (c32 % 1000) as f32, 0.0);
                exit!(sim, sc, e, _s, _ctx, e.set_rally_point(&p));
            }
            exit!(sim, sc, e, s, ctx, {
                let mut p1 = Coord3D::default();
                let mut p2 = Coord3D::default();
                let r1 = e.get_exit_position(&*s, ctx, &mut p1);
                let r2 = e.get_natural_rally_point(&*s, ctx, &mut p2, TRUE);
                let rally = e.get_rally_point().map_or_else(|| "n".to_owned(), coord);
                format!("{r1},{},{r2},{},{rally}", coord(&p1), coord(&p2))
            })
        }
        21 => {
            if b == 0 {
                sim.clear_status(sc.field, ObjectStatusType::UnderConstruction);
                sim.clear_status(sc.field, ObjectStatusType::Sold);
            } else if b == 1 {
                sim.set_status(sc.field, ObjectStatusType::UnderConstruction);
            } else {
                sim.set_status(sc.field, ObjectStatusType::Sold);
            }
            String::new()
        }
        _ => panic!("op {code}"),
    }
}

fn matrix(values: &str) -> Matrix3D {
    let v: Vec<f32> = values.split(',').map(from_bits).collect();
    let mut m = Matrix3D::identity();
    for r in 0..3 {
        for c in 0..4 {
            m.rows[r][c] = v[r * 4 + c];
        }
    }
    m
}

fn build(sim: &mut SimContext, line: &str) -> Scene {
    let layout = get_u32(line, "layout").expect("layout") as usize;
    sim.world.frame = get_u32(line, "frame").expect("frame");
    sim.world
        .rng
        .init_random_with_seed(get_u32(line, "seed").expect("seed"));
    let name = format!("P286_Field{layout}");
    for bone in get_str(line, "bones")
        .expect("bones")
        .split(';')
        .filter(|b| !b.is_empty())
    {
        let (bone_name, values) = bone.split_once('=').expect("bone");
        sim.world
            .pristine_bones
            .insert((name.clone(), bone_name.to_owned()), vec![matrix(values)]);
    }
    let at: Vec<f32> = get_str(line, "field")
        .expect("field")
        .split(',')
        .map(from_bits)
        .collect();
    let tid = template(sim, &name);
    let field = sim.spawn_object(tid, TEAMS[1], Coord3D::new(at[0], at[1], at[2]), at[3]);
    let field_id = sim.world.objects.id_of(field).expect("field id");
    let park = module_named(sim, field, "ParkingPlaceBehavior").expect("parking");
    let prod = module_named(sim, field, "ProductionUpdate");
    let mut crafts = Vec::new();
    for craft in get_str(line, "crafts").expect("crafts").split(';') {
        let f: Vec<&str> = craft.split(',').collect();
        let kind: usize = f[0].parse().expect("kind");
        let team: usize = f[1].parse().expect("team");
        let p = Coord3D::new(from_bits(f[2]), from_bits(f[3]), from_bits(f[4]));
        let tid = template(sim, &format!("P286_Craft{kind}"));
        let slot = sim.spawn_object(tid, TEAMS[team], p, 0.5);
        if f[5] == "1" {
            sim.world.objects.get_mut(slot).expect("craft").producer_id = field_id;
        }
        let id = sim.world.objects.id_of(slot).expect("craft id");
        crafts.push((id, slot, kind));
    }
    if get_u32(line, "construction").expect("construction") == 1 {
        sim.set_status(field, ObjectStatusType::UnderConstruction);
    }
    ORDERS.with_borrow_mut(String::clear);
    sim.world.ai_commands.clear();
    sim.world.exit_path_orders.clear();
    Scene {
        field,
        field_id,
        park,
        prod,
        crafts,
    }
}

#[test]
fn parking_place_matches_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT286_ParkingPlace.json");
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"layout\""))
        .collect();
    assert_eq!(lines.len(), 1000);
    let mut sim = context();
    let mut checked = 0;
    let mut next_id = 1;
    for (index, line) in lines.iter().enumerate() {
        reset(&mut sim);
        // The harness's ids run on from vector to vector.
        sim.world.next_object_id = next_id;
        let sc = build(&mut sim, line);
        next_id = sim.world.next_object_id;
        assert!(sc.field_id.0 > 0);
        let mut frame = sim.world.frame;
        let mut last = Vec::new();
        let actual = row(&mut sim, &sc, "");
        assert_row(
            &actual,
            &mut last,
            get_str(line, "spawn").expect("spawn"),
            &format!("vector {index} spawn"),
        );
        let ops: Vec<i64> = between(line, "\"ops\":[", "],\"rows\"")
            .split(['[', ']', ','])
            .filter(|t| !t.is_empty())
            .map(|t| t.parse().expect("op number"))
            .collect();
        let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
            .split("\",\"")
            .collect();
        assert_eq!(ops.len(), rows.len() * 4, "vector {index}");
        for (k, (op, expected)) in ops.chunks(4).zip(&rows).enumerate() {
            let op = [op[0], op[1], op[2], op[3]];
            let ret = apply(&mut sim, &sc, &mut frame, op);
            let actual = row(&mut sim, &sc, &ret);
            assert_row(
                &actual,
                &mut last,
                expected,
                &format!("vector {index} op {k} {op:?}"),
            );
            checked += 1;
        }
    }
    assert!(checked > 20_000, "only {checked} rows");
}

fn load(sim: &mut SimContext, slot: ModuleSlot, bytes: &[u8]) {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    payload
        .xfer(&mut XferLoad::new(bytes), sim)
        .expect("xfer load");
    sim.world.modules.restore_payload(slot, generation, payload);
}

fn state<R>(
    sim: &mut SimContext,
    slot: ModuleSlot,
    f: impl FnOnce(&ParkingPlaceBehavior) -> R,
) -> R {
    sim.dispatch_module(slot, |m, _, _| {
        f(m.as_any()
            .and_then(|a| a.downcast_ref::<ParkingPlaceBehavior>())
            .expect("parking"))
    })
    .expect("dispatch")
}

fn pair(sim: &mut SimContext, layout: usize) -> (ObjectSlot, ModuleSlot) {
    let tid = template(sim, &format!("P286_Field{layout}"));
    let field = sim.spawn_object(tid, TEAMS[1], Coord3D::new(300.0, 300.0, 0.0), 0.0);
    let park = module_named(sim, field, "ParkingPlaceBehavior").expect("parking");
    (field, park)
}

#[test]
fn saved_state_loads_back() {
    let mut sim = context();
    reset(&mut sim);
    sim.world.frame = 50;
    let (_, source) = pair(&mut sim, 0);
    let tid = template(&sim, "P286_Craft0");
    let craft = sim.spawn_object(tid, TEAMS[1], Coord3D::default(), 0.0);
    let id = sim.world.objects.id_of(craft).expect("id");
    sim.dispatch_module(source, |m, s, ctx| {
        let p = m.as_parking_place_mut().expect("parking");
        assert_eq!(p.reserve_space(s, ctx, id, 0.0, None), TRUE);
        assert_eq!(p.reserve_runway(s, ctx, id, TRUE), TRUE);
        p.set_healee(s, ctx, craft, TRUE);
        m.as_exit_mut()
            .expect("exit")
            .set_rally_point(&Coord3D::new(1.0, 2.0, 3.0));
    })
    .expect("dispatch");
    let saved = save(&mut sim, source);

    // A fresh module builds its spaces on load and takes the saved occupants.
    let (_, target) = pair(&mut sim, 0);
    assert_eq!(state(&mut sim, target, |p| p.spaces.len()), 0);
    load(&mut sim, target, &saved);
    state(&mut sim, target, |p| {
        assert_eq!(p.spaces.len(), 4);
        assert_eq!(p.spaces[0].object_in_space, id);
        assert_eq!(p.runways[0].in_use_by, id);
        assert_eq!(p.healing.len(), 1);
        assert_eq!(p.healing[0].heal_start_frame, 50);
        assert_eq!(p.next_heal_frame, 56);
        assert_eq!(p.heli_rally_point, Coord3D::new(1.0, 2.0, 3.0));
        assert_eq!(p.heli_rally_point_exists, TRUE);
    });
    assert_eq!(save(&mut sim, target), saved);

    // Spaces the layout does not have are read and dropped.
    let (_, small) = pair(&mut sim, 2);
    load(&mut sim, small, &saved);
    state(&mut sim, small, |p| {
        assert_eq!(p.spaces.len(), 2);
        assert_eq!(p.runways.len(), 1);
        assert_eq!(p.spaces[0].object_in_space, id);
        assert_eq!(p.healing.len(), 1);
        assert_eq!(p.next_heal_frame, 56);
    });

    // Version 2 has no heal frame; a load sets it to zero.
    let mut old = saved[..saved.len() - 4].to_vec();
    old[0] = 2;
    let (_, target) = pair(&mut sim, 0);
    load(&mut sim, target, &old);
    state(&mut sim, target, |p| {
        assert_eq!(p.next_heal_frame, 0);
        assert_eq!(p.heli_rally_point_exists, TRUE);
    });
}

#[test]
fn production_reaches_the_module() {
    let mut sim = context();
    reset(&mut sim);
    sim.world.frame = 20;
    let (field, park) = pair(&mut sim, 0);
    let jet = template(&sim, "P286_Craft0");
    let heli = template(&sim, "P286_Craft3");
    assert_eq!(
        sim.reserve_exit_door(field, heli),
        ExitDoorType::DoorNoneNeeded
    );
    assert_eq!(sim.reserve_exit_door(field, jet), ExitDoorType::Door1);
    assert_eq!(sim.reserve_exit_door(field, jet), ExitDoorType::Door2);
    sim.unreserve_exit_door(field, ExitDoorType::Door1);
    assert_eq!(sim.reserve_exit_door(field, jet), ExitDoorType::Door1);

    let unit = sim.spawn_object(jet, TEAMS[1], Coord3D::default(), 0.0);
    let id = sim.world.objects.id_of(unit).expect("id");
    sim.exit_object_via_door(field, unit, ExitDoorType::Door2);
    assert_eq!(state(&mut sim, park, |p| p.spaces[1].object_in_space), id);
    assert_eq!(sim.object_has_reserved_space(field, id), TRUE);
    sim.release_parking_space(field, id);
    assert_eq!(sim.object_has_reserved_space(field, id), FALSE);
    assert!(sim.reserve_parking_space(field, id, 0.0).is_some());
    assert_eq!(sim.object_has_reserved_space(field, id), TRUE);

    // A release from inside the module's own call is applied when it returns.
    sim.dispatch_module(park, |_, s, _| s.release_parking_space(field, id))
        .expect("dispatch");
    assert_eq!(sim.object_has_reserved_space(field, id), FALSE);
}

/// Fields the INI leaves out keep the reference constructor's values, and building the
/// spaces releases every door.
#[test]
fn data_defaults_and_door_release() {
    let d = ParkingPlaceBehaviorModuleData::new(NameKeyType::default(), NameKeyType::default());
    assert_eq!((d.num_rows, d.num_cols), (0, 0));
    assert_eq!(
        (
            d.heal_amount,
            d.approach_height,
            d.landing_deck_height_offset
        ),
        (0.0, 0.0, 0.0)
    );
    assert_eq!((d.has_runways, d.park_in_hangars), (FALSE, FALSE));

    let mut sim = context();
    reset(&mut sim);
    sim.world.frame = 20;
    let (field, park) = pair(&mut sim, 0);
    let production = module_named(&mut sim, field, "ProductionUpdate").expect("production");
    let before = save(&mut sim, production);
    sim.production_set_hold_door_open(field, ExitDoorType::Door3, TRUE);
    assert_ne!(save(&mut sim, production), before);
    let held = save(&mut sim, production);
    parking(&mut sim, park, |m, s, ctx| {
        m.as_update_mut().expect("update").update(s, ctx);
    });
    assert_ne!(save(&mut sim, production), held);
}
