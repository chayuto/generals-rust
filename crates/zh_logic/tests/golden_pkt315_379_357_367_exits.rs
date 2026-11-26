//! `PKT-315` `DefaultProductionExitUpdate`, `PKT-379`
//! `SupplyCenterProductionExitUpdate`, `PKT-357` `QueueProductionExitUpdate` and
//! `PKT-367` `SpawnPointProductionExitUpdate` against vectors from the reference
//! modules (`tests/cpp_harness/gen_pkt315_379_357_367_exits.cpp`).
//!
//! The AIs of the exited units are unported, so `Ai` below stands in; so does
//! The stealth module is the real `StealthUpdate`.
use std::any::Any;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, NameKeyType, ObjectId, Real, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot, ThingTemplateId};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CommandSourceType, ExitDoorType, ModuleData, ModuleType,
    SupplyTruckAiFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::physics_update::physics_mut;
use zh_logic::modules::update::stealth_update::StealthUpdate;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::pathfinder::pathfind_cell::CellType;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

// -----------------------------------------------------------------------------
// Stand-in AI and stealth modules
// -----------------------------------------------------------------------------

#[derive(Clone)]
struct StubData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    mask: FacetMask,
    truck: bool,
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

struct Ai {
    data: StubData,
    force_wanting: Bool,
    move_order: Option<(Coord3D, CommandSourceType)>,
}

impl SupplyTruckAiFacet for Ai {
    fn number_boxes(&self) -> i32 {
        0
    }
    fn max_boxes(&self) -> i32 {
        0
    }
    fn upgraded_supply_boost(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> i32 {
        0
    }
    fn lose_one_box(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn gain_one_box(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _left: i32) -> Bool {
        FALSE
    }
    fn set_force_wanting_state(&mut self, wanting: Bool) {
        self.force_wanting = wanting;
    }
}

impl AiUpdateFacet for Ai {
    fn supply_truck(&mut self) -> Option<&mut dyn SupplyTruckAiFacet> {
        if self.data.truck {
            Some(self)
        } else {
            None
        }
    }
    fn ai_move_to_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    ) {
        self.move_order = Some((*pos, cmd_source));
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.ai_construct(ctx.owner_slot);
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

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

fn parse_stub(reader: &mut IniReader<'_>, data: StubData) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(data))
}

fn stub(tag_key: NameKeyType, module_name_key: NameKeyType, mask: FacetMask) -> StubData {
    StubData {
        tag_key,
        module_name_key,
        mask,
        truck: false,
    }
}

fn parse_ai(
    r: &mut IniReader<'_>,
    t: NameKeyType,
    n: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_stub(r, stub(t, n, AI_MASK))
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
            truck: true,
            ..stub(t, n, AI_MASK)
        },
    )
}

fn stub_data(data: &dyn ModuleData) -> StubData {
    data.as_any()
        .downcast_ref::<StubData>()
        .expect("stub")
        .clone()
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(Ai {
        data: stub_data(data),
        force_wanting: FALSE,
        move_order: None,
    })
}

// -----------------------------------------------------------------------------
// Terrain and fixture, as in the harness
// -----------------------------------------------------------------------------

struct Slope(bool);

impl ThingTerrainAccess for Slope {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        if self.0 {
            x * 0.01 + y * 0.02
        } else {
            0.0
        }
    }
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        zh_logic::object::thing::make_align_to_normal_matrix(
            angle,
            pos,
            &Coord3D::new(0.0, 0.0, 1.0),
            mtx,
        );
        PathfindLayerEnum::Ground
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
    fn get_layer_height(
        &self,
        x: Real,
        y: Real,
        layer: PathfindLayerEnum,
        normal: Option<&mut Coord3D>,
        _clip: Bool,
    ) -> Real {
        self.get_ground_height(x, y, normal) + 2.0 * (layer as i32 - 1) as Real
    }
}

/// `KindOf`, radius, AI (1 plain, 2 supply truck), locomotor, physics, stealth.
type UnitSpec = (&'static str, u32, u32, Option<&'static str>, bool, u32);

const UNITS: [UnitSpec; 11] = [
    ("PRELOAD", 3, 0, None, false, 0),
    ("PRELOAD", 7, 1, Some("P_Ground"), true, 0),
    ("PRELOAD", 12, 1, Some("P_Air"), true, 0),
    ("PRELOAD", 3, 1, Some("P_Both"), true, 0),
    ("PRELOAD", 15, 1, None, false, 0),
    ("PRELOAD", 30, 2, Some("P_Ground"), true, 1),
    ("PRELOAD PRODUCED_AT_HELIPAD", 6, 1, Some("P_Both"), true, 0),
    ("PRELOAD PRODUCED_AT_HELIPAD", 3, 1, Some("P_Air"), true, 0),
    ("PRELOAD", 12, 1, Some("P_Ground"), false, 2),
    ("PRELOAD", 7, 0, None, true, 0),
    ("PRELOAD PROJECTILE", 10, 1, Some("P_Ground"), false, 0),
];

const KINDS: [&str; 4] = [
    "DefaultProductionExitUpdate",
    "SupplyCenterProductionExitUpdate",
    "QueueProductionExitUpdate",
    "SpawnPointProductionExitUpdate",
];
const CREATE: [&str; 6] = [
    "X:0 Y:0 Z:0",
    "X:12.5 Y:-3.25 Z:0",
    "X:-20 Y:7 Z:15.5",
    "X:3 Y:4 Z:-2",
    "X:0.1 Y:0.2 Z:0",
    "X:33 Y:0 Z:0",
];
const NATURAL: [&str; 6] = [
    "X:0 Y:0 Z:0",
    "X:40 Y:0 Z:0",
    "X:-13.5 Y:22.25 Z:6",
    "X:0.3 Y:0.4 Z:0",
    "X:1 Y:1 Z:1",
    "X:17.7 Y:-31.9 Z:0",
];
const BONE_NAME: &str = "SpawnPoint";

fn producer_fields(kind: usize, v: usize) -> String {
    let mut s = String::new();
    if kind != 3 {
        let _ = writeln!(s, "    UnitCreatePoint = {}", CREATE[v]);
        let _ = writeln!(s, "    NaturalRallyPoint = {}", NATURAL[(v + kind) % 6]);
    }
    if kind == 0 && v % 2 == 1 {
        s += "    UseSpawnRallyPoint = Yes\n";
    }
    if kind == 1 {
        let grant = [
            None,
            Some("0"),
            Some("1"),
            Some("1000"),
            Some("5000"),
            Some("333"),
        ];
        if let Some(g) = grant[v] {
            let _ = writeln!(s, "    GrantTemporaryStealth = {g}");
        }
    }
    if kind == 2 {
        let delay = [
            None,
            Some("0"),
            Some("1"),
            Some("100"),
            Some("300"),
            Some("1000"),
        ];
        let burst = [0, 2, 0, 1, 0, 3];
        if let Some(d) = delay[v] {
            let _ = writeln!(s, "    ExitDelay = {d}");
        }
        if burst[v] != 0 {
            let _ = writeln!(s, "    InitialBurst = {}", burst[v]);
        }
        if v >= 3 {
            s += "    AllowAirborneCreation = Yes\n";
        }
    }
    if kind == 3 {
        let _ = writeln!(s, "    SpawnPointBoneName = {BONE_NAME}");
    }
    s
}

fn fixture_ini() -> String {
    let mut s = String::new();
    let body = "  Body = InactiveBody ModuleTag_Body\n  End\n";
    for (i, (kind_of, radius, ai, loco, physics, stealth)) in UNITS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P_Unit{i}\n  KindOf = {kind_of}\n  Geometry = CYLINDER\n  \
             GeometryMajorRadius = {radius}\n  GeometryHeight = 4\n  GeometryIsSmall = No\n{body}"
        );
        if *physics {
            let sign = if i % 2 == 1 { "-" } else { "" };
            let _ = write!(
                s,
                "  Behavior = PhysicsBehavior ModuleTag_Physics\n    Mass = {}.5\n    \
                 CenterOfMassOffset = {sign}{}.25\n  End\n",
                i + 2,
                i + 1
            );
        }
        if *ai != 0 {
            let name = if *ai == 2 {
                "OracleExitTruck"
            } else {
                "OracleExitAI"
            };
            let _ = write!(s, "  Behavior = {name} ModuleTag_AI\n  End\n");
            if let Some(l) = loco {
                let _ = writeln!(s, "  Locomotor = SET_NORMAL {l}");
            }
        }
        if *stealth != 0 {
            let innate = if *stealth == 2 { "Yes" } else { "No" };
            let _ = write!(
                s,
                "  Behavior = StealthUpdate ModuleTag_Stealth\n    InnateStealth = {innate}\n  End\n"
            );
        }
        s += "End\n";
    }
    for (k, module) in KINDS.iter().enumerate() {
        for v in 0..6 {
            for p in 0..2 {
                let _ = write!(s, "Object P_Exit{k}_{v}_{p}\n  KindOf = PRELOAD\n{body}");
                if p == 1 {
                    s += "  Behavior = PhysicsBehavior ModuleTag_Physics\n    Mass = 40\n  End\n";
                }
                let _ = write!(
                    s,
                    "  Behavior = {module} ModuleTag_Exit\n{}  End\nEnd\n",
                    producer_fields(k, v)
                );
            }
        }
    }
    s
}

fn context() -> SimContext {
    let mut rules = RulesRegistry::new();
    for (name, truck) in [("OracleExitAI", false), ("OracleExitTruck", true)] {
        rules.module_registry.register_module(
            &mut rules.name_keys,
            name,
            ModuleType::Behavior,
            AI_MASK,
            SleepyUpdatePhase::Initial,
            true,
            if truck { parse_truck } else { parse_ai },
            create_ai,
        );
    }
    let common = "  Speed = 60\n  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n";
    for (name, surfaces, extra) in [
        ("P_Ground", "GROUND", ""),
        ("P_Air", "AIR", "  MinSpeed = 0\n  Lift = 100\n"),
        ("P_Both", "GROUND AIR", "  MinSpeed = 0\n  Lift = 100\n"),
    ] {
        let block = format!("Locomotor {name}\n  Surfaces = {surfaces}\n{common}{extra}End\n");
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(&block).expect("locomotor");
        t.name = name.to_owned();
        rules.locomotor_store.add_template(t);
    }
    let ini = fixture_ini();
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    let mut sim = SimContext::new();
    sim.rules = rules;
    sim
}

// -----------------------------------------------------------------------------
// Observation
// -----------------------------------------------------------------------------

struct Scene {
    producer: ObjectSlot,
    module: ModuleSlot,
    spawned: Vec<ObjectSlot>,
    frame: u32,
}

fn real(bits: u32) -> Real {
    Real::from_bits(bits)
}

fn put(out: &mut String, c: &Coord3D) {
    let _ = write!(
        out,
        " {} {} {}",
        c.x.to_bits(),
        c.y.to_bits(),
        c.z.to_bits()
    );
}

fn template(sim: &SimContext, name: &str) -> ThingTemplateId {
    sim.rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"))
}

fn save_hex(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

fn row(sim: &mut SimContext, s: &Scene, door: i32) -> String {
    let save = save_hex(sim, s.module);
    let next = next_call_frame(sim, s.module);
    let (payload, ctx, generation) = sim.world.modules.take_payload(s.module).expect("payload");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    let mut out = format!("{save} {} {next} {door}", xc.get_crc());
    let exit = payload.as_exit().expect("exit facet");
    let mut c = Coord3D::new(-1.0, -1.0, -1.0);
    let ok = exit.get_exit_position(sim, ctx, &mut c);
    let _ = write!(out, " {}", ok as u32);
    put(&mut out, &c);
    for offset in [TRUE, FALSE] {
        let mut c = Coord3D::new(-1.0, -1.0, -1.0);
        let ok = exit.get_natural_rally_point(sim, ctx, &mut c, offset);
        let _ = write!(out, " {}", ok as u32);
        put(&mut out, &c);
    }
    match exit.get_rally_point() {
        Some(r) => {
            out += " 1";
            put(&mut out, r);
        }
        None => out += " 0",
    }
    let _ = write!(
        out,
        " {}{}",
        exit.use_spawn_rally_point() as u32,
        exit.is_exit_busy(sim, ctx) as u32
    );
    sim.world
        .modules
        .restore_payload(s.module, generation, payload);
    out
}

fn ai_module(sim: &SimContext, o: ObjectSlot) -> Option<&Ai> {
    let ai = sim.world.objects.get(o)?.ai_slot?;
    sim.world.modules.payload(ai)?.as_any()?.downcast_ref()
}

fn unit_row(sim: &mut SimContext, unit: ObjectSlot) -> String {
    let stealth = sim.world.objects.get(unit).expect("unit").stealth_slot;
    let granted = stealth.map_or(0, |slot| {
        sim.dispatch_module(slot, |m, _, _| {
            m.as_any()
                .and_then(|a| a.downcast_ref::<StealthUpdate>())
                .expect("stealth")
                .frames_granted
        })
        .expect("dispatch stealth")
    });
    let physics = sim.world.objects.get(unit).expect("unit").physics_slot;
    let physics = physics.map_or_else(|| "-".to_owned(), |slot| save_hex(sim, slot));
    let obj = sim.world.objects.get(unit).expect("unit");
    let id = obj.id;
    let status = |s: ObjectStatusType| u32::from(obj.status.test(s.as_i32()) != 0);
    let mut out = format!(" U {}", id.0);
    put(&mut out, obj.thing.get_position());
    let _ = write!(
        out,
        " {} {} {} {}{} {} {} {physics}",
        obj.thing.get_orientation().to_bits(),
        obj.layer as i32,
        u32::from(obj.is_disabled_by_type(DisabledType::Held) != FALSE),
        status(ObjectStatusType::Stealthed),
        status(ObjectStatusType::CanStealth),
        granted,
        ai_module(sim, unit).map_or(0, |a| a.force_wanting as u32),
    );
    let follow = sim.world.exit_path_orders.iter().find(|o| o.object == id);
    let moved = ai_module(sim, unit).and_then(|a| a.move_order);
    assert!(
        sim.world.exit_path_orders.len() <= 1,
        "more than one path order in an exit"
    );
    match (follow, moved) {
        (Some(order), None) => {
            let _ = write!(
                out,
                " 1 {} {} {}",
                order.cmd_source as i32,
                order.ignore.0,
                order.path.len()
            );
            for c in &order.path {
                put(&mut out, c);
            }
        }
        (None, Some((pos, source))) => {
            let _ = write!(out, " 2 {} 0 1", source as i32);
            put(&mut out, &pos);
        }
        (None, None) => out += " 0 0 0 0",
        _ => panic!("two commands"),
    }
    out
}

// -----------------------------------------------------------------------------
// Replay
// -----------------------------------------------------------------------------

fn numbers(text: &str) -> Vec<u32> {
    text.split_whitespace()
        .map(|t| t.parse().expect("number"))
        .collect()
}

fn build_scene(sim: &mut SimContext, line: &str) -> Scene {
    let field = |k: &str| get_u32(line, k).expect("field");
    sim.world = SimWorld::new();
    sim.world.frame = field("frame");
    sim.world.next_object_id = field("id") as i32;
    sim.terrain = Box::new(Slope(field("slope") == 1));
    let name = format!(
        "P_Exit{}_{}_{}",
        field("kind"),
        field("variant"),
        field("phys")
    );
    let bones = numbers(get_str(line, "bones").expect("bones"));
    if !bones.is_empty() {
        let transforms = bones
            .chunks_exact(12)
            .map(|m| {
                let mut out = Matrix3D::identity();
                for r in 0..3 {
                    for c in 0..4 {
                        out.rows[r][c] = real(m[r * 4 + c]);
                    }
                }
                out
            })
            .collect();
        sim.world
            .pristine_bones
            .insert((name.clone(), BONE_NAME.to_owned()), transforms);
    }
    let tid = template(sim, &name);
    let producer = sim.spawn_object(
        tid,
        TeamId(1 + field("team") as i32),
        Coord3D::new(real(field("x")), real(field("y")), real(field("z"))),
        real(field("angle")),
    );
    sim.set_object_layer(producer, layer(field("layer")));
    let module = sim
        .world
        .objects
        .get(producer)
        .and_then(|o| o.exit_slot)
        .expect("exit module");
    Scene {
        producer,
        module,
        spawned: Vec::new(),
        frame: field("frame"),
    }
}

fn layer(v: u32) -> PathfindLayerEnum {
    match v {
        1 => PathfindLayerEnum::Ground,
        2 => PathfindLayerEnum::Bridge1,
        3 => PathfindLayerEnum::Bridge2,
        4 => PathfindLayerEnum::Bridge3,
        _ => panic!("layer {v}"),
    }
}

fn spawn_unit(sim: &mut SimContext, which: u32, flags: u32) -> ObjectSlot {
    let tid = template(sim, &format!("P_Unit{which}"));
    let unit = sim.spawn_object(tid, TeamId(1), Coord3D::new(500.0, 500.0, 0.0), 1.0);
    if flags & 4 != 0 {
        sim.set_physics_allow_to_fall(unit, TRUE);
    }
    if flags & 8 != 0 {
        sim.receive_temporary_stealth_grant(unit, 90);
    }
    if flags & 16 != 0 {
        sim.set_status(unit, ObjectStatusType::CanStealth);
    }
    if flags & 1 != 0 {
        sim.set_disabled(unit, DisabledType::Held);
    }
    if flags & 2 != 0 {
        sim.set_disabled(unit, DisabledType::Unmanned);
    }
    unit
}

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

fn apply(sim: &mut SimContext, s: &mut Scene, op: &[u32]) -> String {
    let (code, a, b, c, d, e) = (op[0], op[1], op[2], op[3], op[4], op[5]);
    sim.world.frame = s.frame;
    sim.world.exit_path_orders.clear();
    let mut door = 99;
    let mut extra = String::new();
    match code {
        0 | 5 => {
            let unit = spawn_unit(sim, a, b);
            let mut host = None;
            if code == 0 {
                sim.exit_object_via_door(s.producer, unit, ExitDoorType::Door1);
            } else {
                if c != 0 {
                    let hx = real(e);
                    let tid = template(sim, "P_Unit0");
                    let h = sim.spawn_object(
                        tid,
                        TeamId(1),
                        Coord3D::new(hx, 1000.0 - hx, hx * 0.1),
                        hx * 0.003,
                    );
                    sim.set_object_layer(h, layer(d));
                    host = Some(h);
                }
                sim.dispatch_module(s.module, |m, sim, ctx| {
                    m.as_exit_mut()
                        .expect("exit")
                        .exit_object_by_budding(sim, ctx, unit, host);
                })
                .expect("dispatch");
            }
            extra = unit_row(sim, unit);
            if let Some(h) = host {
                sim.destroy_object(h);
            }
            if is_spawn_point(sim, s) {
                s.spawned.push(unit);
            } else {
                sim.destroy_object(unit);
            }
            sim.process_destroy_list();
        }
        1 => {
            let rally = Coord3D::new(real(a), real(b), real(c));
            sim.dispatch_module(s.module, |m, _, _| {
                m.as_exit_mut().expect("exit").set_rally_point(&rally);
            })
            .expect("dispatch");
        }
        2 => {
            if b != 0 {
                let sleep = sim
                    .dispatch_module(s.module, |m, sim, ctx| {
                        m.as_update_mut().expect("update").update(sim, ctx)
                    })
                    .expect("dispatch");
                door = 100 + sleep.0 as i32;
            } else {
                for _ in 0..a {
                    s.frame += 1;
                    step(sim, s.module, s.frame);
                }
            }
        }
        3 => {
            let tid = template(sim, "P_Unit0");
            door = sim
                .dispatch_module(s.module, |m, sim, ctx| {
                    let exit = m.as_exit_mut().expect("exit");
                    let door = exit.reserve_door_for_exit(sim, ctx, tid, None);
                    exit.unreserve_door_for_exit(door);
                    door.as_i32()
                })
                .expect("dispatch");
        }
        4 => {
            sim.set_object_position(s.producer, &Coord3D::new(real(a), real(b), real(c)));
            sim.set_object_orientation(s.producer, real(d));
        }
        6 => {
            if a != 0 {
                sim.set_status(s.producer, ObjectStatusType::Stealthed);
            } else {
                sim.clear_status(s.producer, ObjectStatusType::Stealthed);
            }
            if let Some(p) = physics_mut(sim, s.producer) {
                p.vel = Coord3D::new(real(b), real(c), real(d));
            }
        }
        7 => {
            let victim = s.spawned.remove(a as usize);
            sim.destroy_object(victim);
            sim.process_destroy_list();
        }
        _ => panic!("op {code}"),
    }
    row(sim, s, door) + &extra
}

fn is_spawn_point(sim: &SimContext, s: &Scene) -> bool {
    sim.world.modules.payload(s.module).is_some_and(|p| {
        p.as_any().is_some_and(|a| {
            a.is::<
                zh_logic::modules::update::spawn_point_production_exit_update::SpawnPointProductionExitUpdate,
            >()
        })
    })
}

fn vector_lines() -> Vec<&'static str> {
    include_str!("../../../docs/golden_vectors/PKT315_379_357_367_Exits.json")
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"kind\""))
        .collect()
}

/// The first token at which two rows differ, for the failure message.
fn first_difference(got: &str, want: &str) -> String {
    let (g, w): (Vec<_>, Vec<_>) = (got.split(' ').collect(), want.split(' ').collect());
    let at = (0..g.len().max(w.len()))
        .find(|&i| g.get(i) != w.get(i))
        .unwrap_or(0);
    format!(
        "token {at}: got {:?} want {:?}\n got {got}\nwant {want}",
        g.get(at),
        w.get(at)
    )
}

#[test]
fn exits_match_reference() {
    let lines = vector_lines();
    assert_eq!(lines.len(), 1200);
    let mut sim = context();
    let mut checked = 0;
    let mut failures = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        let mut s = build_scene(&mut sim, line);
        let spawned = row(&mut sim, &s, 0);
        let expected = get_str(line, "spawn").expect("spawn");
        checked += 1;
        if spawned != expected {
            failures.push(format!(
                "vector {index} spawn {}",
                first_difference(&spawned, expected)
            ));
            continue;
        }
        let ops: Vec<&str> = get_str(line, "ops").expect("ops").split(';').collect();
        let rows: Vec<&str> = get_str(line, "rows").expect("rows").split('|').collect();
        assert_eq!(ops.len(), rows.len(), "vector {index}");
        for (k, (op, expected)) in ops.iter().zip(&rows).enumerate() {
            let got = apply(&mut sim, &mut s, &numbers(op));
            checked += 1;
            if got != *expected {
                failures.push(format!(
                    "vector {index} kind {} variant {} op {k} [{op}] {}",
                    get_u32(line, "kind").expect("kind"),
                    get_u32(line, "variant").expect("variant"),
                    first_difference(&got, expected)
                ));
                break;
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} vectors differ ({checked} rows):\n{}",
        failures.len(),
        lines.len(),
        failures
            .iter()
            .take(4)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
    assert!(checked > 9000, "{checked} rows");
}

/// Branches the harness cannot reach: a refused rally goal, and a producer that
/// is not in the world.
#[test]
fn blocked_rally_goal_is_left_off_the_path() {
    let mut sim = context();
    sim.world = SimWorld::new();
    sim.world.frame = 10;
    let tid = template(&sim, "P_Exit0_1_0");
    let producer = sim.spawn_object(tid, TeamId(1), Coord3D::new(200.0, 200.0, 0.0), 0.0);
    let rally = Coord3D::new(305.0, 405.0, 0.0);
    let module = sim
        .world
        .objects
        .get(producer)
        .and_then(|o| o.exit_slot)
        .expect("exit module");
    sim.dispatch_module(module, |m, _, _| {
        m.as_exit_mut().expect("exit").set_rally_point(&rally);
    })
    .expect("dispatch");
    for (blocked, points) in [(false, 2), (true, 1)] {
        if blocked {
            sim.world.blocked_pathfind_cells.insert(
                // A radius 7 unit does not center, so its cell is half a cell on.
                (PathfindLayerEnum::Ground as i32, 31, 41),
                CellType::Impassable,
            );
        }
        sim.world.exit_path_orders.clear();
        let unit = spawn_unit(&mut sim, 1, 0);
        sim.exit_object_via_door(producer, unit, ExitDoorType::Door1);
        assert_eq!(sim.world.exit_path_orders[0].path.len(), points);
        assert!(sim
            .world
            .pathfind_map_objects
            .contains(&sim.world.objects.get(unit).expect("unit").id));
    }
    assert_eq!(sim.world.exit_path_orders[0].ignore, ObjectId(1));
    assert_eq!(
        sim.world.exit_path_orders[0].cmd_source,
        CommandSourceType::FromAi
    );
}

fn exit_module(sim: &SimContext, producer: ObjectSlot) -> ModuleSlot {
    sim.world
        .objects
        .get(producer)
        .and_then(|o| o.exit_slot)
        .expect("exit module")
}

/// `Object::getObjectExitInterface` asks the update modules before the container,
/// whatever order the template lists them in.
#[test]
fn production_exit_wins_over_an_earlier_container() {
    let mut sim = context();
    let ini = "Object P_Both\n  KindOf = PRELOAD STRUCTURE\n  \
        Body = InactiveBody ModuleTag_Body\n  End\n  \
        Behavior = OpenContain ModuleTag_Contain\n  End\n  \
        Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n    \
        UseSpawnRallyPoint = Yes\n  End\nEnd\n";
    let mut reader = IniReader::new("both.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("template");
    sim.world = SimWorld::new();
    let tid = template(&sim, "P_Both");
    let producer = sim.spawn_object(tid, TeamId(1), Coord3D::new(200.0, 200.0, 0.0), 0.0);
    assert!(sim
        .world
        .objects
        .get(producer)
        .is_some_and(|o| o.contain_slot.is_some()));
    assert_eq!(sim.exit_use_spawn_rally_point(producer), TRUE);
}

fn load(sim: &mut SimContext, slot: ModuleSlot, bytes: &[u8]) {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xl = XferLoad::new(bytes);
    payload.xfer(&mut xl, sim).expect("xfer load");
    sim.world.modules.restore_payload(slot, generation, payload);
}

/// The harness does not load saves: a save read into a second producer must
/// write the same bytes again.
#[test]
fn saved_state_loads_back() {
    let mut sim = context();
    for kind in 0..4 {
        sim.world = SimWorld::new();
        sim.world.frame = 10;
        let name = format!("P_Exit{kind}_5_0");
        let mut bone = Matrix3D::identity();
        bone.rows[0][3] = 4.0;
        sim.world
            .pristine_bones
            .insert((name.clone(), BONE_NAME.to_owned()), vec![bone; 3]);
        let tid = template(&sim, &name);
        let used = sim.spawn_object(tid, TeamId(1), Coord3D::new(200.0, 200.0, 0.0), 0.0);
        let fresh = sim.spawn_object(tid, TeamId(1), Coord3D::new(300.0, 300.0, 0.0), 0.0);
        let (used_module, fresh_module) = (exit_module(&sim, used), exit_module(&sim, fresh));
        sim.dispatch_module(used_module, |m, _, _| {
            m.as_exit_mut()
                .expect("exit")
                .set_rally_point(&Coord3D::new(1.0, 2.0, 3.0));
        })
        .expect("dispatch");
        for _ in 0..2 {
            let unit = spawn_unit(&mut sim, 0, 0);
            sim.exit_object_via_door(used, unit, ExitDoorType::Door1);
        }
        let saved = save_hex(&mut sim, used_module);
        assert_ne!(saved, save_hex(&mut sim, fresh_module), "kind {kind}");
        let bytes: Vec<u8> = (0..saved.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&saved[i..i + 2], 16).expect("hex"))
            .collect();
        load(&mut sim, fresh_module, &bytes);
        assert_eq!(save_hex(&mut sim, fresh_module), saved, "kind {kind}");
    }
}

/// Parachuting units ask the building this before using its exit.
#[test]
fn spawn_rally_flag_follows_the_ini() {
    let mut sim = context();
    sim.world = SimWorld::new();
    let tid = template(&sim, "P_Exit0_0_0");
    let producer = sim.spawn_object(tid, TeamId(1), Coord3D::new(200.0, 200.0, 0.0), 0.0);
    assert_eq!(sim.exit_use_spawn_rally_point(producer), FALSE);
    let tid = template(&sim, "P_Exit0_1_0");
    let other = sim.spawn_object(tid, TeamId(1), Coord3D::new(300.0, 300.0, 0.0), 0.0);
    assert_eq!(sim.exit_use_spawn_rally_point(other), TRUE);
}
