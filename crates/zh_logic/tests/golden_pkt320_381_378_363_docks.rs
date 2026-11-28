//! `PKT-320` `DockUpdate`, `PKT-381` `SupplyWarehouseDockUpdate`, `PKT-378`
//! `SupplyCenterDockUpdate` and `PKT-363` `RepairDockUpdate` against vectors from
//! the reference modules (`tests/cpp_harness/gen_pkt320_381_378_363_docks.cpp`).
//!
//! The reference dockers run the real supply truck AIs. Those are unported, so
//! `Truck` below stands in for the few calls a dock makes on them.

use std::any::Any;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, DockFacet, ModuleData, ModuleType, SupplyTruckAiFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::supply_warehouse_dock_update::SupplyWarehouseDockUpdate;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const TEAMS: [TeamId; 2] = [TeamId(1), TeamId(2)];
const ANGLES: [Real; 4] = [0.0, 1.570_796_4, 0.7, -2.3];
type Real = f32;

// -----------------------------------------------------------------------------
// The stand-in dockers
// -----------------------------------------------------------------------------

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

#[derive(Clone)]
struct StubData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    mask: FacetMask,
    supply: bool,
    max_boxes: Int,
    boost: Int,
}

impl ModuleData for StubData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }
    fn facet_mask(&self) -> FacetMask {
        self.mask
    }
}

/// `SupplyTruckAIUpdate`'s box handling, or a plain `AIUpdateInterface`.
struct Truck {
    data: StubData,
    boxes: Int,
    force_wanting: Bool,
}

impl SupplyTruckAiFacet for Truck {
    fn number_boxes(&self) -> Int {
        self.boxes
    }
    fn max_boxes(&self) -> Int {
        self.data.max_boxes
    }
    /// `ChinookAIUpdate::getUpgradedSupplyBoost`: only the second team has the upgrade.
    fn upgraded_supply_boost(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int {
        let upgraded = sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.team_id == TEAMS[1]);
        if upgraded {
            self.data.boost
        } else {
            0
        }
    }
    fn lose_one_box(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> Bool {
        if self.boxes == 0 {
            return FALSE;
        }
        self.boxes -= 1;
        TRUE
    }
    fn gain_one_box(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _remaining: Int,
    ) -> Bool {
        if self.boxes >= self.data.max_boxes {
            return FALSE;
        }
        self.boxes += 1;
        TRUE
    }
    fn set_force_wanting_state(&mut self, wanting: Bool) {
        self.force_wanting = wanting;
    }
}

impl AiUpdateFacet for Truck {
    fn supply_truck(&mut self) -> Option<&mut dyn SupplyTruckAiFacet> {
        if self.data.supply {
            Some(self)
        } else {
            None
        }
    }
}

impl BehaviorModule for Truck {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.ai_construct(ctx.owner_slot);
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.ai_init_state_machine(ctx.owner_slot);
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
    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

fn parse_stub(
    reader: &mut IniReader<'_>,
    mut data: StubData,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("MaxBoxes") {
            r.parse_int(&mut data.max_boxes)
        } else if field.eq_ignore_ascii_case("UpgradedSupplyBoost") {
            r.parse_int(&mut data.boost)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

fn stub(tag_key: NameKeyType, module_name_key: NameKeyType, mask: FacetMask) -> StubData {
    StubData {
        tag_key,
        module_name_key,
        mask,
        supply: false,
        max_boxes: 0,
        boost: 0,
    }
}

fn parse_truck(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stub(
        r,
        StubData {
            supply: true,
            ..stub(t, n, AI_MASK)
        },
    )
}

fn parse_plain_ai(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stub(r, stub(t, n, AI_MASK))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let data = data
        .as_any()
        .downcast_ref::<StubData>()
        .expect("stub")
        .clone();
    Box::new(Truck {
        data,
        boxes: 0,
        force_wanting: FALSE,
    })
}

// -----------------------------------------------------------------------------
// Fixture: the text `fixture_ini` in the harness builds
// -----------------------------------------------------------------------------

struct DockType {
    kind_of: &'static str,
    geometry: &'static str,
    approach: Option<&'static str>,
    extra: &'static str,
}

const fn dock(
    kind_of: &'static str,
    geometry: &'static str,
    approach: Option<&'static str>,
    extra: &'static str,
) -> DockType {
    DockType {
        kind_of,
        geometry,
        approach,
        extra,
    }
}

const CYL: &str = "CYLINDER 30 30 20";
const BOX: &str = "BOX 40 24 20";
const SPH: &str = "SPHERE 25 25 25";

const WAREHOUSES: [DockType; 7] = [
    dock(
        "STRUCTURE SUPPLY_SOURCE",
        CYL,
        Some("3"),
        "    StartingBoxes = 3\n",
    ),
    dock(
        "STRUCTURE SUPPLY_SOURCE",
        BOX,
        Some("-1"),
        "    StartingBoxes = 20\n    DeleteWhenEmpty = No\n",
    ),
    dock("STRUCTURE", SPH, None, ""),
    dock(
        "STRUCTURE SUPPLY_SOURCE",
        CYL,
        Some("1"),
        "    StartingBoxes = 2\n    DeleteWhenEmpty = Yes\n    AllowsPassthrough = No\n",
    ),
    dock(
        "STRUCTURE IGNORE_DOCKING_BONES",
        BOX,
        Some("2"),
        "    StartingBoxes = 0\n",
    ),
    dock(
        "STRUCTURE SUPPLY_SOURCE",
        CYL,
        Some("10"),
        "    StartingBoxes = 40\n",
    ),
    dock(
        "STRUCTURE",
        "CYLINDER 220 220 20",
        Some("2"),
        "    StartingBoxes = 5\n",
    ),
];
const CENTERS: [DockType; 4] = [
    dock(
        "STRUCTURE",
        BOX,
        Some("-1"),
        "    GrantTemporaryStealth = 3000\n",
    ),
    dock("STRUCTURE", CYL, Some("3"), ""),
    dock("STRUCTURE", SPH, None, "    AllowsPassthrough = No\n"),
    dock(
        "STRUCTURE SUPPLY_SOURCE",
        CYL,
        Some("2"),
        "    GrantTemporaryStealth = 100\n",
    ),
];
const REPAIRS: [DockType; 6] = [
    dock("STRUCTURE", CYL, None, "    TimeForFullHeal = 1000\n"),
    dock("STRUCTURE", BOX, Some("1"), ""),
    dock("STRUCTURE", CYL, Some("5"), "    TimeForFullHeal = 5000\n"),
    dock("STRUCTURE", SPH, Some("-1"), "    TimeForFullHeal = 333\n"),
    dock(
        "STRUCTURE SUPPLY_SOURCE",
        BOX,
        Some("3"),
        "    TimeForFullHeal = 0\n    AllowsPassthrough = No\n",
    ),
    dock(
        "STRUCTURE IGNORE_DOCKING_BONES",
        CYL,
        Some("-1"),
        "    TimeForFullHeal = 100\n",
    ),
];
const DOCK_TYPES: [&[DockType]; 3] = [&WAREHOUSES, &CENTERS, &REPAIRS];
const DOCK_MODULES: [&str; 3] = [
    "SupplyWarehouseDockUpdate",
    "SupplyCenterDockUpdate",
    "RepairDockUpdate",
];

fn geometry(spec: &str) -> String {
    let p: Vec<&str> = spec.split(' ').collect();
    format!(
        "  Geometry = {}\n  GeometryMajorRadius = {}\n  GeometryMinorRadius = {}\n  GeometryHeight = {}\n  GeometryIsSmall = No\n",
        p[0], p[1], p[2], p[3]
    )
}

fn body(health: &str) -> String {
    format!(
        "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n"
    )
}

fn objects_ini() -> String {
    let mut s = String::new();
    for (k, types) in DOCK_TYPES.iter().enumerate() {
        for (v, d) in types.iter().enumerate() {
            let approach = d.approach.map_or_else(String::new, |a| {
                format!("    NumberApproachPositions = {a}\n")
            });
            let _ = write!(
                s,
                "Object P320_Dock{k}_{v}\n  KindOf = PRELOAD {}\n{}{}  Behavior = {} ModuleTag_Dock\n{approach}{}  End\nEnd\n",
                d.kind_of,
                geometry(d.geometry),
                body("1000.0"),
                DOCK_MODULES[k],
                d.extra
            );
        }
    }
    let ai = |module: &str, fields: &str, loco: &str| {
        format!(
            "  Behavior = {module} ModuleTag_AI\n{fields}  End\n  Locomotor = SET_NORMAL {loco}\n"
        )
    };
    let units = [
        (
            "VEHICLE HARVESTER",
            "CYLINDER 8 8 10",
            "300.0",
            ai("SupplyTruckAIUpdate", "    MaxBoxes = 4\n", "P320_Ground"),
        ),
        (
            "VEHICLE AIRCRAFT HARVESTER",
            "SPHERE 12 12 12",
            "250.0",
            ai(
                "ChinookAIUpdate",
                "    MaxBoxes = 8\n    UpgradedSupplyBoost = 60\n",
                "P320_Air",
            ),
        ),
        (
            "INFANTRY DOZER HARVESTER",
            "CYLINDER 4 4 8",
            "100.0",
            ai("SupplyTruckAIUpdate", "    MaxBoxes = 1\n", "P320_Ground"),
        ),
        (
            "VEHICLE",
            "BOX 10 6 5",
            "180.0",
            ai("SupplyTruckAIUpdate", "    MaxBoxes = 3\n", "P320_Ground")
                + "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = No\n  End\n",
        ),
        ("VEHICLE", "BOX 7 5 6", "500.0", String::new()),
        (
            "VEHICLE",
            "CYLINDER 6 6 8",
            "400.0",
            ai("AIUpdateInterface", "", "P320_Ground"),
        ),
    ];
    let carrier = |boxes: u32| {
        (
            "VEHICLE AIRCRAFT",
            "SPHERE 9 9 9",
            "100.0",
            ai(
                "ChinookAIUpdate",
                &format!("    MaxBoxes = {boxes}\n    UpgradedSupplyBoost = 60\n"),
                "P320_Air",
            ),
        )
    };
    let carriers = [carrier(1), carrier(0)];
    for (i, (kind_of, geom, health, rest)) in units.iter().chain(&carriers).enumerate() {
        let _ = write!(
            s,
            "Object P320_Docker{i}\n  KindOf = PRELOAD {kind_of}\n{}{}{rest}End\n",
            geometry(geom),
            body(health)
        );
    }
    s
}

fn context() -> SimContext {
    let mut rules = RulesRegistry::new();
    for name in [
        "SupplyTruckAIUpdate",
        "ChinookAIUpdate",
        "AIUpdateInterface",
    ] {
        let parse = if name == "AIUpdateInterface" {
            parse_plain_ai
        } else {
            parse_truck
        };
        rules.module_registry.register_module(
            &mut rules.name_keys,
            name,
            ModuleType::Behavior,
            AI_MASK,
            SleepyUpdatePhase::Initial,
            true,
            parse,
            create_ai,
        );
    }
    for (name, block) in [
        (
            "P320_Ground",
            "Locomotor P320_Ground\n  Surfaces = GROUND\n  Speed = 30\n  TurnRate = 180\n  Acceleration = 30\n  Braking = 30\nEnd\n",
        ),
        (
            "P320_Air",
            "Locomotor P320_Air\n  Surfaces = AIR\n  Speed = 60\n  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n  MinSpeed = 0\n  Lift = 100\nEnd\n",
        ),
    ] {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = name.to_owned();
        rules.locomotor_store.add_template(t);
    }
    let ini = objects_ini();
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    let mut sim = SimContext::new();
    sim.rules = rules;
    sim
}

// -----------------------------------------------------------------------------
// The scene of one vector
// -----------------------------------------------------------------------------

struct Scene {
    dock: ObjectSlot,
    module: ModuleSlot,
    /// `None` once destroyed.
    dockers: Vec<Option<ObjectSlot>>,
    centre: (Int, Int),
    frame: u32,
    last_save: String,
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D, angle: Real) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object(tid, team, pos, angle)
}

fn at(centre: (Int, Int), dx: Int, dy: Int) -> Coord3D {
    Coord3D::new((centre.0 + dx) as Real, (centre.1 + dy) as Real, 0.0)
}

fn numbers(text: &str) -> Vec<i64> {
    text.split_whitespace()
        .map(|t| t.parse().expect("number"))
        .collect()
}

fn build_scene(sim: &mut SimContext, line: &str) -> Scene {
    let field = |k: &str| get_u32(line, k).expect("field");
    sim.world = SimWorld::new();
    sim.world.frame = field("frame");
    sim.world.next_object_id = field("id") as Int;
    let signed = |k: &str| {
        let rest = &line[line.find(&format!("\"{k}\":")).expect(k) + k.len() + 3..];
        let end = rest.find(',').expect(k);
        rest[..end].parse::<Int>().expect(k)
    };
    let centre = (signed("cx"), signed("cy"));
    let dock = spawn(
        sim,
        &format!("P320_Dock{}_{}", field("kind"), field("variant")),
        TEAMS[field("team") as usize],
        at(centre, 0, 0),
        ANGLES[field("angle") as usize],
    );
    let dockers = get_str(line, "dockers")
        .expect("dockers")
        .split(';')
        .map(|d| {
            let v = numbers(d);
            let mut place = at(centre, v[2] as Int, v[3] as Int);
            place.z = v[4] as Real;
            Some(spawn(
                sim,
                &format!("P320_Docker{}", v[0]),
                TEAMS[v[1] as usize],
                place,
                0.0,
            ))
        })
        .collect();
    let module = sim.dock_slot(dock).expect("dock module");
    sim.world.rng.init_random_with_seed(field("seed"));
    Scene {
        dock,
        module,
        dockers,
        centre,
        frame: field("frame"),
        last_save: String::new(),
    }
}

fn model_bits(sim: &SimContext, o: ObjectSlot) -> u32 {
    use ModelConditionFlagType as Mc;
    let flags = &sim.world.objects.get(o).expect("object").model_condition;
    [
        Mc::Docking,
        Mc::DockingBeginning,
        Mc::DockingActive,
        Mc::DockingEnding,
        Mc::Moving,
    ]
    .iter()
    .enumerate()
    .map(|(i, f)| u32::from(flags.test(f.as_i32()) != 0) << i)
    .sum()
}

fn truck(sim: &SimContext, o: ObjectSlot) -> Option<&Truck> {
    let ai = sim.world.objects.get(o)?.ai_slot?;
    sim.world.modules.payload(ai)?.as_any()?.downcast_ref()
}

fn docker_state(sim: &SimContext, out: &mut Vec<u32>, o: ObjectSlot) {
    let obj = sim.world.objects.get(o).expect("docker");
    let p = *obj.thing.get_position();
    let status = |s: ObjectStatusType| u32::from(obj.status.test(s.as_i32()) != 0);
    out.extend([
        model_bits(sim, o),
        p.x.to_bits(),
        p.y.to_bits(),
        p.z.to_bits(),
        sim.object_health(o).to_bits(),
        status(ObjectStatusType::CanStealth)
            | status(ObjectStatusType::Stealthed) << 1
            | u32::from(obj.is_effectively_dead() != FALSE) << 2,
        truck(sim, o)
            .filter(|t| t.data.supply)
            .map_or(0, |t| t.boxes as u32),
        truck(sim, o).map_or(0, |t| u32::from(t.force_wanting)),
    ]);
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

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

/// The harness's row.
fn row(sim: &mut SimContext, s: &mut Scene, ret: &[u32], d: i64) -> String {
    let bytes = hex(&save(sim, s.module));
    let mut out = if bytes == s.last_save {
        "=".to_owned()
    } else {
        bytes.clone()
    };
    s.last_save = bytes;
    let payload = sim.world.modules.payload(s.module).expect("payload");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    let boxes = payload
        .as_any()
        .and_then(|a| a.downcast_ref::<SupplyWarehouseDockUpdate>())
        .map_or(0, |w| w.boxes_stored as u32);
    let mut v = vec![
        xc.get_crc(),
        next_call_frame(sim, s.module),
        model_bits(sim, s.dock),
        sim.world.rng.get_game_logic_random_seed_crc(),
    ];
    for team in TEAMS {
        // Each team here is its own player.
        let player = sim.world.team_players.get(&team).copied().unwrap_or(team.0);
        v.push(sim.money_of(player));
        v.push(
            sim.world.players.player(player).map(|p| &p.score_keeper)
                .map_or(0, |k| k.total_money_earned) as u32,
        );
    }
    let destroyed = sim
        .world
        .objects
        .get(s.dock)
        .is_none_or(|o| o.is_destroyed());
    v.push(boxes);
    v.push(u32::from(destroyed));
    let mut all = Vec::new();
    for &o in s.dockers.iter().flatten() {
        docker_state(sim, &mut all, o);
    }
    v.push(
        all.iter()
            .fold(2_166_136_261_u32, |h, &x| (h ^ x).wrapping_mul(16_777_619)),
    );
    if let Some(&Some(o)) = usize::try_from(d).ok().and_then(|d| s.dockers.get(d)) {
        docker_state(sim, &mut v, o);
    }
    for x in v {
        let _ = write!(out, " {x}");
    }
    out.push_str(" |");
    for x in ret {
        let _ = write!(out, " {x}");
    }
    out
}

fn with_dock<R>(
    sim: &mut SimContext,
    slot: ModuleSlot,
    f: impl FnOnce(&mut dyn DockFacet, &mut dyn SimApi, ModuleContext) -> R,
) -> R {
    sim.dispatch_module(slot, |m, sim, ctx| {
        f(m.as_dock_mut().expect("dock"), sim, ctx)
    })
    .expect("dispatch")
}

fn push_pos(ret: &mut Vec<u32>, p: Coord3D) {
    ret.extend([p.x.to_bits(), p.y.to_bits(), p.z.to_bits()]);
}

/// `set_bones` in the harness.
fn set_bones(sim: &mut SimContext, slot: ModuleSlot, pattern: u32) {
    const T: [[Real; 3]; 4] = [
        [0.0, 0.0, 0.0],
        [40.0, 0.0, 0.0],
        [-12.5, 33.0, 4.0],
        [0.25, -60.0, 0.0],
    ];
    let at = |i: u32| {
        let t = T[i as usize];
        Coord3D::new(t[0], t[1], t[2])
    };
    let (a, b, c) = (pattern & 3, (pattern >> 2) & 3, (pattern >> 4) & 3);
    with_dock(sim, slot, |dock, _, _| {
        let s = dock.dock_state_mut();
        s.enter_position = at(a);
        s.dock_position = at(b);
        s.exit_position = at(c);
        for (i, p) in s.approach_positions.iter_mut().enumerate() {
            *p = at((c + i as u32) & 3);
        }
        match (pattern >> 6) & 3 {
            1 => s.number_approach_position_bones = -1,
            2 => s.number_approach_position_bones = 2,
            3 => s.number_approach_position_bones = 0,
            _ => {}
        }
        match (pattern >> 8) & 3 {
            1 => s.number_approach_positions = -1,
            2 => s.number_approach_positions = s.approach_positions.len() as Int + 1,
            _ => {}
        }
    });
}

/// The harness's scheduled frames.
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

fn apply(sim: &mut SimContext, s: &mut Scene, code: i64, d: i64, a: u32) -> Vec<u32> {
    let mut ret = Vec::new();
    let m = s.module;
    let owner = s.dock;
    let docker = usize::try_from(d)
        .ok()
        .map(|d| s.dockers[d].expect("docker alive"));
    let id = docker.map_or(ObjectId::INVALID, |o| {
        sim.world.objects.get(o).expect("docker").id
    });
    let o = || docker.expect("operation needs a docker");
    let index = a as Int - 1;
    let flag = Bool::from(a != 0);
    match code {
        0 => ret.push(with_dock(sim, m, |k, _, _| {
            u32::from(k.is_clear_to_approach(id))
        })),
        1 | 2 => {
            let o = o();
            let got = with_dock(sim, m, |k, sim, ctx| {
                if code == 1 {
                    k.reserve_approach_position(sim, ctx, o)
                } else {
                    k.advance_approach_position(sim, ctx, o, index)
                }
            });
            ret.push(u32::from(got.is_some()));
            if let Some((pos, i)) = got {
                push_pos(&mut ret, pos);
                ret.push(i as u32);
            }
        }
        3 => ret.push(with_dock(sim, m, |k, _, _| {
            u32::from(k.is_clear_to_enter(id))
        })),
        4 => ret.push(with_dock(sim, m, |k, _, _| {
            u32::from(k.is_clear_to_advance(id, index))
        })),
        5..=7 => {
            let o = o();
            let pos = with_dock(sim, m, |k, sim, ctx| match code {
                5 => k.get_enter_position(sim, ctx, o),
                6 => k.get_dock_position(sim, ctx, o),
                _ => k.get_exit_position(sim, ctx, o),
            });
            push_pos(&mut ret, pos);
        }
        8 => with_dock(sim, m, |k, _, _| k.on_approach_reached(id)),
        9..=12 => {
            let o = o();
            with_dock(sim, m, |k, sim, ctx| match code {
                9 => k.on_enter_reached(sim, ctx, o),
                10 => k.on_dock_reached(sim, ctx, o),
                11 => k.on_exit_reached(sim, ctx, o),
                _ => k.cancel_dock(sim, ctx, o),
            });
        }
        13 => ret.push(
            sim.dispatch_module(m, |b, s, c| {
                b.as_update_mut().expect("update").update(s, c).0
            })
            .expect("dispatch"),
        ),
        14 => sim.set_dock_open(owner, flag),
        15 => sim.set_dock_crippled(owner, flag),
        16 => {
            let o = o();
            let drone = (a != 0).then(|| s.dockers[a as usize - 1].expect("drone"));
            ret.push(with_dock(sim, m, |k, sim, ctx| {
                u32::from(k.action(sim, ctx, o, drone))
            }));
            if let Some(drone) = drone {
                docker_state(sim, &mut ret, drone);
            }
        }
        17 => {
            ret.push(u32::from(sim.dock_is_open(owner)));
            with_dock(sim, m, |k, _, _| {
                ret.push(u32::from(k.is_allow_passthrough_type()));
                ret.push(u32::from(k.is_rally_point_after_dock_type()));
            });
        }
        18 => {
            let pos = at(s.centre, (a & 0xffff) as Int - 200, (a >> 16) as Int - 200);
            sim.set_object_position(o(), &pos);
        }
        19 => {
            let mut got = FALSE;
            sim.with_supply_truck(o(), &mut |t, sim, ctx| {
                got = if a != 0 {
                    t.lose_one_box(sim, ctx)
                } else {
                    t.gain_one_box(sim, ctx, 5)
                };
            });
            ret.push(u32::from(got));
        }
        20 => {
            sim.dispatch_module(m, |b, sim, _| {
                b.as_any_mut()
                    .and_then(|x| x.downcast_mut::<SupplyWarehouseDockUpdate>())
                    .expect("warehouse")
                    .set_cash_value(sim, a as Int - 100);
            })
            .expect("dispatch");
        }
        21 => {
            for _ in 0..a {
                s.frame += 1;
                step(sim, m, s.frame);
            }
        }
        22 => set_bones(sim, m, a),
        23 => with_dock(sim, m, |k, sim, ctx| {
            k.dock_state_mut().load_dock_positions(sim, ctx.owner_slot);
        }),
        24 => {
            if a == 8 {
                sim.set_model_condition_state(o(), ModelConditionFlagType::Moving);
            } else {
                let target = if a & 1 != 0 { o() } else { owner };
                let status = if a & 2 != 0 {
                    ObjectStatusType::Stealthed
                } else {
                    ObjectStatusType::CanStealth
                };
                if a & 4 != 0 {
                    sim.set_status(target, status);
                } else {
                    sim.clear_status(target, status);
                }
            }
        }
        25 => {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Unresistable;
            dmg.input.death_type = DeathType::Normal;
            dmg.input.source_id = ObjectId::INVALID;
            dmg.input.amount = a as Real;
            sim.attempt_damage(o(), &mut dmg);
        }
        26 => sim.receive_temporary_stealth_grant(o(), a),
        27 => {
            sim.destroy_object(o());
            sim.process_destroy_list();
            s.dockers[d as usize] = None;
        }
        _ => panic!("op {code}"),
    }
    ret
}

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

fn vector_lines() -> Vec<&'static str> {
    include_str!("../../../docs/golden_vectors/PKT320_381_378_363_Docks.json")
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"kind\""))
        .collect()
}

#[test]
fn docks_match_reference() {
    let lines = vector_lines();
    assert_eq!(lines.len(), 1200);
    let mut sim = context();
    let mut checked = 0;
    let mut failures = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let mut s = build_scene(&mut sim, line);
        let spawned = row(&mut sim, &mut s, &[], -1);
        let expected = get_str(line, "spawn").expect("spawn");
        if spawned != expected {
            failures.push(format!(
                "vector {index} spawn\n got {spawned}\nwant {expected}"
            ));
            continue;
        }
        let ops: Vec<&str> = get_str(line, "ops").expect("ops").split(';').collect();
        let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
            .split("\",\"")
            .collect();
        assert_eq!(ops.len(), rows.len(), "vector {index}");
        for (k, (op, expected)) in ops.iter().zip(&rows).enumerate() {
            let v = numbers(op);
            sim.world.frame = s.frame;
            let ret = apply(&mut sim, &mut s, v[0], v[1], v[2] as u32);
            let got = row(&mut sim, &mut s, &ret, v[1]);
            if got != *expected {
                failures.push(format!(
                    "vector {index} (kind {}) op {k} ({op})\n got {got}\nwant {expected}",
                    get_u32(line, "kind").expect("kind")
                ));
                break;
            }
            checked += 1;
        }
    }
    assert!(
        failures.is_empty(),
        "{} vectors differ; first ones:\n{}",
        failures.len(),
        failures[..failures.len().min(3)].join("\n")
    );
    assert!(checked > 18_000, "only {checked} rows");
}

#[test]
fn only_a_supply_truck_ai_is_found() {
    let mut sim = context();
    let none = |sim: &mut SimContext, name: &str| {
        let o = spawn(sim, name, TEAMS[0], at((500, 500), 0, 0), 0.0);
        sim.with_supply_truck(o, &mut |_, _, _| panic!("not a supply truck"))
    };
    // No AI, and an AI that is not a supply truck.
    assert_eq!(none(&mut sim, "P320_Docker4"), FALSE);
    assert_eq!(none(&mut sim, "P320_Docker5"), FALSE);
    let o = spawn(
        &mut sim,
        "P320_Docker0",
        TEAMS[0],
        at((500, 500), 0, 0),
        0.0,
    );
    assert_eq!(sim.with_supply_truck(o, &mut |_, _, _| {}), TRUE);
    // An object without a dock counts as open and ignores both setters.
    assert_eq!(sim.dock_slot(o), None);
    sim.set_dock_open(o, FALSE);
    sim.set_dock_crippled(o, TRUE);
    assert_eq!(sim.dock_is_open(o), TRUE);
}

/// A save loads into a second dock, and the load pass re-reads the (absent) bones.
#[test]
fn saved_docks_load_back() {
    let mut sim = context();
    for kind in 0..3 {
        let line = format!(
            "{{\"kind\":{kind},\"variant\":0,\"team\":0,\"angle\":2,\"cx\":500,\"cy\":500,\"frame\":50,\"seed\":7,\"id\":1,\"dockers\":\"0 0 40 0 0;0 0 -40 0 0\"}}"
        );
        let mut s = build_scene(&mut sim, &line);
        let m = s.module;
        for code in [1, 8, 13, 9, 16, 15, 14] {
            apply(&mut sim, &mut s, code, 0, 0);
        }
        apply(&mut sim, &mut s, 22, -1, 1 | (2 << 2) | (3 << 4));
        let changed = save(&mut sim, m);
        let other = spawn(
            &mut sim,
            &format!("P320_Dock{kind}_0"),
            TEAMS[0],
            at((500, 500), 100, 100),
            0.0,
        );
        let target = sim.dock_slot(other).expect("dock");
        assert_ne!(save(&mut sim, target), changed, "kind {kind}");
        let (mut payload, _ctx, generation) =
            sim.world.modules.take_payload(target).expect("payload");
        let mut load = XferLoad::new(&changed);
        payload.xfer(&mut load, &sim).expect("xfer load");
        sim.world
            .modules
            .restore_payload(target, generation, payload);
        let state = |sim: &SimContext, slot| {
            let d = sim.world.modules.payload(slot).and_then(|p| p.as_dock());
            let d = d.expect("dock").dock_state();
            (
                d.enter_position,
                d.dock_position,
                d.exit_position,
                d.approach_positions.clone(),
                d.approach_position_owners.clone(),
                d.approach_position_reached.clone(),
                (
                    d.active_docker,
                    d.docker_inside,
                    d.dock_crippled,
                    d.dock_open,
                ),
            )
        };
        assert_eq!(state(&sim, target), state(&sim, m), "kind {kind}");
        // The module's own fields round-trip; the header before them holds the live
        // schedule, which a load alone does not change.
        let again = save(&mut sim, target);
        assert_eq!(again.len(), changed.len(), "kind {kind}");
        assert_eq!(
            again[again.len() - 60..],
            changed[changed.len() - 60..],
            "kind {kind}"
        );

        sim.dispatch_module(target, |b, sim, ctx| b.load_post_process(sim, ctx))
            .expect("dispatch");
        let after = state(&sim, target);
        assert_eq!(after.0, state(&sim, m).0, "stored bone positions stay");
        // Variant 0 of the supply center has a dynamic list, which a load leaves alone.
        let zeroed = after.3.iter().all(|&p| p == Coord3D::default());
        assert_eq!(zeroed, kind != 1, "approach positions: kind {kind}");
    }
}
