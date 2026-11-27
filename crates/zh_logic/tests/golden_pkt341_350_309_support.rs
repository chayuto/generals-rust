//! `PKT-341` `MissileLauncherBuildingUpdate`, `PKT-350` `PointDefenseLaserUpdate` and
//! `PKT-309` `BattlePlanUpdate` against vectors from the reference modules
//! (`tests/cpp_harness/gen_pkt341_350_309_support.cpp`), with the real special power,
//! stealth detector and weapon firing code.
//!
//! The AI is a stand-in with a turret that records its orders. Loading a save and what the
//! vectors cannot reach are checked here without a reference.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Bool, NameKeyType, ObjectId, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot, ThingTemplateId};
use zh_logic::combat::WhichTurretType;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CommandSourceType, MaxHealthChangeType, ModuleData, ModuleType,
    SpecialPowerCaller,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::battle_plan_update::{
    BattlePlanStatus, BattlePlanUpdate, BattlePlanUpdateModuleData,
};
use zh_logic::modules::update::missile_launcher_building_update::{
    DoorState, MissileLauncherBuildingUpdate,
};
use zh_logic::modules::update::point_defense_laser_update::PointDefenseLaserUpdate;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::{DamageType, DeathType, WeaponSetType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

// Neutral, A and B are players 0, 1 and 2.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];

const FX_NAMES: [&str; 5] = [
    "P341_FxOpening",
    "P341_FxOpen",
    "P341_FxWaiting",
    "P341_FxClosing",
    "P341_FxClosed",
];

// door open, wait open, close times (ms); reload (ms); FX mask
#[rustfmt::skip]
const SILOS: [[u32; 5]; 6] = [
    [1000, 500, 1000, 3000, 31], [2000, 0, 3000, 6000, 31], [5000, 1000, 100, 2000, 0],
    [0, 0, 0, 1000, 21],         [600, 2000, 4000, 2500, 10], [100, 100, 100, 0, 31],
];

const PD_WEAPONS: [&str; 4] = [
    "  AttackRange = 60\n  DelayBetweenShots = 100\n",
    "  AttackRange = 100\n  DelayBetweenShots = Min:33 Max:300\n  AntiGround = No\n  AntiAirborneVehicle = Yes\n",
    "  AttackRange = 40\n  DelayBetweenShots = 0\n  AntiSmallMissile = Yes\n",
    "  AttackRange = 80\n  DelayBetweenShots = Min:0 Max:100\n  AntiGround = Yes\n  AntiBallisticMissile = Yes\n",
];
// weapon, scan rate (ms), primary, secondary, velocity factor
#[rustfmt::skip]
const PDS: [(usize, u32, &str, &str, &str); 5] = [
    (0, 0, "INFANTRY", "VEHICLE", "0"),
    (1, 100, "BALLISTIC_MISSILE SMALL_MISSILE", "VEHICLE AIRCRAFT", "2.5"),
    (2, 500, "SMALL_MISSILE", "INFANTRY", "0"),
    (3, 200, "VEHICLE", "INFANTRY SMALL_MISSILE", "10"),
    (0, 1000, "INFANTRY VEHICLE", "", "1"),
];
#[rustfmt::skip]
const TARGETS: [&str; 8] = [
    "INFANTRY", "VEHICLE", "INFANTRY VEHICLE", "PROJECTILE SMALL_MISSILE", "AIRCRAFT VEHICLE",
    "STRUCTURE IMMOBILE VEHICLE", "BALLISTIC_MISSILE PROJECTILE", "STRUCTURE",
];

struct Center {
    health: &'static str,
    change: &'static str,
    sight: &'static str,
    detects: bool,
    armor: &'static str,
    member_sight: &'static str,
    paralyze: u32,
    idle: u32,
    anim: u32,
    valid: &'static str,
    invalid: &'static str,
    ai: u8,
    detector: bool,
}

#[rustfmt::skip]
const CENTERS: [Center; 5] = [
    Center { health: "2.0", change: "PRESERVE_RATIO", sight: "2.0", detects: true, armor: "0.9", member_sight: "1.2", paralyze: 500, idle: 0, anim: 200, valid: "INFANTRY VEHICLE", invalid: "AIRCRAFT", ai: 2, detector: true },
    Center { health: "1.5", change: "ADD_CURRENT_HEALTH_TOO", sight: "1.5", detects: false, armor: "0.5", member_sight: "1.0", paralyze: 300, idle: 100, anim: 100, valid: "INFANTRY VEHICLE STRUCTURE", invalid: "", ai: 2, detector: true },
    Center { health: "1.0", change: "FULLY_HEAL", sight: "1.0", detects: true, armor: "1.0", member_sight: "0.0", paralyze: 1000, idle: 200, anim: 0, valid: "VEHICLE STRUCTURE", invalid: "", ai: 0, detector: true },
    Center { health: "3.0", change: "FULLY_HEAL", sight: "0.7", detects: true, armor: "0.0", member_sight: "2.0", paralyze: 0, idle: 0, anim: 66, valid: "INFANTRY", invalid: "", ai: 1, detector: false },
    Center { health: "0.0", change: "SAME_CURRENTHEALTH", sight: "3.0", detects: false, armor: "1.3", member_sight: "1.5", paralyze: 100, idle: 66, anim: 133, valid: "INFANTRY VEHICLE AIRCRAFT", invalid: "STRUCTURE", ai: 2, detector: false },
];
const MEMBERS: [&str; 5] = [
    "INFANTRY",
    "VEHICLE",
    "AIRCRAFT VEHICLE",
    "STRUCTURE",
    "PROJECTILE",
];

#[derive(Debug, Clone, Copy, Default)]
struct Turret {
    enabled: bool,
    recentering: bool,
    turned: bool,
}

thread_local! {
    static ORDERS: RefCell<String> = const { RefCell::new(String::new()) }; // determinism-ok: test stand-in recorder
    static TURRETS: RefCell<BTreeMap<ObjectId, Turret>> = const { RefCell::new(BTreeMap::new()) }; // determinism-ok: test stand-in recorder
}

#[derive(Debug, Clone)]
struct AiData {
    tag: NameKeyType,
    name: NameKeyType,
    turret: bool,
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

impl Ai {
    fn turret(&self, f: impl FnOnce(&mut Turret)) {
        TURRETS.with_borrow_mut(|t| {
            if let Some(turret) = t.get_mut(&self.owner) {
                f(turret);
            }
        });
    }
}

impl AiUpdateFacet for Ai {
    fn which_turret_for_cur_weapon(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
    ) -> WhichTurretType {
        if self.data.turret {
            WhichTurretType::Main
        } else {
            WhichTurretType::Invalid
        }
    }
    fn set_turret_enabled(&mut self, turret: WhichTurretType, enabled: Bool) {
        assert_eq!(turret, WhichTurretType::Main);
        self.turret(|t| t.enabled = enabled != FALSE);
    }
    fn recenter_turret(&mut self, turret: WhichTurretType) {
        assert_eq!(turret, WhichTurretType::Main);
        self.turret(|t| t.recentering = true);
    }
    fn is_turret_in_natural_position(&self, turret: WhichTurretType) -> Bool {
        assert_eq!(turret, WhichTurretType::Main);
        Bool::from(!TURRETS.with_borrow(|t| t.get(&self.owner).is_some_and(|t| t.turned)))
    }
    fn ai_idle(
        &mut self,
        _sim: &mut dyn SimApi,
        ctx: ModuleContext,
        cmd_source: CommandSourceType,
    ) {
        ORDERS.with_borrow_mut(|o| {
            let _ = write!(o, "{{{}:3:{}}}", ctx.owner_id.0, cmd_source as i32);
        });
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.data.name
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.owner = ctx.owner_id;
        sim.ai_construct(ctx.owner_slot);
        if self.data.turret {
            TURRETS.with_borrow_mut(|t| {
                t.insert(
                    ctx.owner_id,
                    Turret {
                        enabled: true,
                        ..Turret::default()
                    },
                )
            });
        }
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        // The state machine the C++ AI builds draws four times; the recording AI never
        // lets an order reach it, so it is not marked ready.
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

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag: NameKeyType,
    name: NameKeyType,
    _c: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AiData {
        tag,
        name,
        turret: false,
    };
    reader.read_block_fields(|field, r| {
        if field == "Turret" {
            data.turret = true;
            r.read_block_fields(|_, r| {
                let mut ignored = String::new();
                r.parse_ascii_string(&mut ignored)
            })
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
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

fn special_powers() -> String {
    let mut s = String::new();
    for (i, silo) in SILOS.iter().enumerate() {
        let _ = write!(
            s,
            "SpecialPower P341_A{i}\n  Enum = SPECIAL_SCUD_STORM\n  ReloadTime = {}\nEnd\nSpecialPower P341_B{i}\n  Enum = SPECIAL_NEUTRON_MISSILE\n  ReloadTime = 100\nEnd\n",
            silo[3]
        );
    }
    s.push_str(
        "SpecialPower P309_Power\n  Enum = SPECIAL_CHANGE_BATTLE_PLANS\n  ReloadTime = 0\nEnd\n",
    );
    s
}

fn weapon(name: &str, body: &str) -> WeaponTemplate {
    let mut t = WeaponTemplate::default();
    t.parse_ini_block(&format!("Weapon {name}\n  PrimaryDamage = 10\n{body}End\n"))
        .expect("weapon");
    t
}

fn objects() -> String {
    let mut s = String::new();
    let body = |max: u32, initial: u32| {
        format!("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {max}\n    InitialHealth = {initial}\n  End\n")
    };
    for (i, [open, wait, close, _, fx]) in SILOS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P341_Silo{i}\n  KindOf = PRELOAD STRUCTURE\n{}  Behavior = SpecialAbility ModuleTag_SpA\n    SpecialPowerTemplate = P341_A{i}\n  End\n  Behavior = SpecialAbility ModuleTag_SpB\n    SpecialPowerTemplate = P341_B{i}\n  End\n  Behavior = MissileLauncherBuildingUpdate ModuleTag_M\n    SpecialPowerTemplate = P341_A{i}\n    DoorOpenTime = {open}\n    DoorWaitOpenTime = {wait}\n    DoorCloseTime = {close}\n",
            body(1000, 1000)
        );
        let fields = [
            "DoorOpeningFX",
            "DoorOpenFX",
            "DoorWaitingToCloseFX",
            "DoorClosingFX",
            "DoorClosedFX",
        ];
        for (f, field) in fields.iter().enumerate() {
            if fx & (1 << f) != 0 {
                let _ = writeln!(s, "    {field} = {}", FX_NAMES[f]);
            }
        }
        s.push_str("  End\nEnd\n");
    }
    for (i, (w, rate, primary, secondary, factor)) in PDS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P350_Laser{i}\n  KindOf = PRELOAD STRUCTURE\n{}  Behavior = PointDefenseLaserUpdate ModuleTag_L\n    WeaponTemplate = P350_W{w}\n    ScanRate = {rate}\n    ScanRange = 150\n    PrimaryTargetTypes = {primary}\n",
            body(100, 100)
        );
        if !secondary.is_empty() {
            let _ = writeln!(s, "    SecondaryTargetTypes = {secondary}");
        }
        let _ = write!(
            s,
            "    PredictTargetVelocityFactor = {factor}\n  End\nEnd\n"
        );
    }
    for (i, kind) in TARGETS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P350_Target{i}\n  KindOf = PRELOAD {kind}\n{}End\n",
            body(100, 100)
        );
    }
    for (i, v) in CENTERS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P309_Center{i}\n  KindOf = PRELOAD STRUCTURE\n  VisionRange = 100\n  ShroudClearingRange = 130\n  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY P309_Gun\n  End\n{}",
            body(1000, 700)
        );
        if v.ai == 1 {
            s.push_str("  Behavior = OracleSupportAI ModuleTag_AI\n  End\n");
        }
        if v.ai == 2 {
            s.push_str("  Behavior = OracleSupportAI ModuleTag_AI\n    Turret\n      ControlledWeaponSlots = PRIMARY\n    End\n  End\n");
        }
        if v.detector {
            s.push_str("  Behavior = StealthDetectorUpdate ModuleTag_Det\n    DetectionRate = 500\n  End\n");
        }
        let _ = write!(
            s,
            "  Behavior = SpecialAbility ModuleTag_Sp\n    SpecialPowerTemplate = P309_Power\n  End\n  Behavior = BattlePlanUpdate ModuleTag_Plan\n    SpecialPowerTemplate = P309_Power\n    BombardmentPlanAnimationTime = {}\n    HoldTheLinePlanAnimationTime = {}\n    SearchAndDestroyPlanAnimationTime = {}\n    TransitionIdleTime = {}\n    BattlePlanChangeParalyzeTime = {}\n    ValidMemberKindOf = {}\n",
            v.anim,
            v.anim + 100,
            v.anim * 2,
            v.idle,
            v.paralyze,
            v.valid
        );
        if !v.invalid.is_empty() {
            let _ = writeln!(s, "    InvalidMemberKindOf = {}", v.invalid);
        }
        let _ = write!(
            s,
            "    HoldTheLinePlanArmorDamageScalar = {}\n    SearchAndDestroyPlanSightRangeScalar = {}\n    StrategyCenterSearchAndDestroySightRangeScalar = {}\n    StrategyCenterSearchAndDestroyDetectsStealth = {}\n    StrategyCenterHoldTheLineMaxHealthScalar = {}\n    StrategyCenterHoldTheLineMaxHealthChangeType = {}\n  End\nEnd\n",
            v.armor,
            v.member_sight,
            v.sight,
            if v.detects { "Yes" } else { "No" },
            v.health,
            v.change
        );
    }
    for (i, kind) in MEMBERS.iter().enumerate() {
        let _ = write!(
            s,
            "Object P309_Member{i}\n  KindOf = PRELOAD {kind}\n  VisionRange = 80\n  ShroudClearingRange = 90\n{}End\n",
            body(200, 150)
        );
    }
    s
}

fn context() -> SimContext {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "OracleSupportAI",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    for (i, body) in PD_WEAPONS.iter().enumerate() {
        rules
            .weapon_store
            .add_template(weapon(&format!("P350_W{i}"), body));
    }
    rules.weapon_store.add_template(weapon(
        "P309_Gun",
        "  AttackRange = 100\n  DelayBetweenShots = 100\n",
    ));
    let powers = special_powers();
    rules
        .special_power_store
        .load_from_ini(
            &mut IniReader::new("power.ini", powers.as_bytes(), IniLoadType::Overwrite),
            &rules.science_store,
            &mut rules.name_keys,
        )
        .expect("powers");
    let ini = objects();
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
    TURRETS.with_borrow_mut(BTreeMap::clear);
}

fn template(sim: &SimContext, name: &str) -> ThingTemplateId {
    sim.rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"))
}

fn spawn(sim: &mut SimContext, name: &str, team: usize, at: Coord3D) -> ObjectSlot {
    let tid = template(sim, name);
    sim.spawn_object(tid, TEAMS[team], at, 0.0)
}

fn modules_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> Vec<ModuleSlot> {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner).expect("object");
    obj.behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .filter(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
        .collect()
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
    payload
        .xfer(&mut XferLoad::new(bytes), sim)
        .expect("xfer load");
    sim.world.modules.restore_payload(slot, generation, payload);
}

fn save_crc(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let bytes = save(sim, slot);
    let mut xc = XferCrc::new();
    sim.world
        .modules
        .payload(slot)
        .expect("payload")
        .crc(&mut xc);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("{hex}:{}", xc.get_crc())
}

fn update(sim: &mut SimContext, slot: ModuleSlot) -> u32 {
    sim.dispatch_module(slot, |m, s, c| {
        m.as_update_mut().expect("update").update(s, c).0
    })
    .expect("dispatch")
}

fn flag(on: bool) -> char {
    if on {
        '1'
    } else {
        '0'
    }
}

fn or_dash(text: String) -> String {
    if text.is_empty() {
        "-".to_owned()
    } else {
        text
    }
}

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

struct Rows<'a> {
    last: Vec<String>,
    rows: std::vec::IntoIter<&'a str>,
}

impl Rows<'_> {
    fn check(&mut self, actual: Vec<String>, expected: &str, what: &str) {
        let expected: Vec<&str> = expected.split(' ').collect();
        assert_eq!(
            actual.len(),
            expected.len(),
            "{what}: {actual:?} {expected:?}"
        );
        for (i, (a, e)) in actual.iter().zip(&expected).enumerate() {
            let e = if *e == "~" { self.last[i].as_str() } else { e };
            assert_eq!(a, e, "{what}: field {i}");
        }
        self.last = actual;
    }
}

/// The vector's operations and the reader of its rows.
fn script(line: &str) -> (Vec<[i64; 4]>, Rows<'_>) {
    let ops: Vec<i64> = between(line, "\"ops\":[", "],\"rows\"")
        .split(['[', ']', ','])
        .filter(|t| !t.is_empty())
        .map(|t| t.parse().expect("op number"))
        .collect();
    let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
        .split("\",\"")
        .collect();
    assert_eq!(ops.len(), rows.len() * 4);
    (
        ops.chunks(4).map(|o| [o[0], o[1], o[2], o[3]]).collect(),
        Rows {
            last: Vec::new(),
            rows: rows.into_iter(),
        },
    )
}

// --- missile launcher -------------------------------------------------------

struct Silo {
    object: ObjectSlot,
    module: ModuleSlot,
    powers: [ModuleSlot; 2],
}

fn silo_row(sim: &mut SimContext, silo: &Silo) -> Vec<String> {
    let mut out = vec![
        save_crc(sim, silo.module),
        save_crc(sim, silo.powers[0]),
        save_crc(sim, silo.powers[1]),
    ];
    use ModelConditionFlagType as F;
    let obj = sim.world.objects.get(silo.object).expect("silo");
    out.push(
        [
            F::Door1Opening,
            F::Door1WaitingOpen,
            F::Door1WaitingToClose,
            F::Door1Closing,
        ]
        .iter()
        .map(|f| flag(obj.model_condition.test(f.as_i32()) != 0))
        .collect(),
    );
    out.push(
        sim.dispatch_module(silo.module, |m, s, _| {
            let spu = m
                .as_special_power_update_mut()
                .expect("special power update");
            format!(
                "{}{}",
                spu.is_active(),
                spu.is_power_currently_in_use(&*s, None)
            )
        })
        .expect("dispatch"),
    );
    let mut fx = String::new();
    for (name, p) in std::mem::take(&mut sim.world.triggered_fx_lists) {
        let _ = write!(
            fx,
            "{{{name}:{}:{}:{}}}",
            p.x.to_bits(),
            p.y.to_bits(),
            p.z.to_bits()
        );
    }
    out.push(or_dash(fx));
    out
}

fn do_power(sim: &mut SimContext, module: ModuleSlot, options: u32) {
    sim.with_special_power_module(module, &mut |spm, sim, ctx| {
        spm.do_special_power(sim, ctx, options);
    });
}

/// `initiateIntentToDoSpecialPower` on `update`, called by the power module `power`.
fn initiate(
    sim: &mut SimContext,
    power: ModuleSlot,
    update: ModuleSlot,
    template_id: u32,
    options: u32,
) -> Bool {
    let mut taken = FALSE;
    sim.with_special_power_module(power, &mut |module, sim, ctx| {
        let caller = SpecialPowerCaller { ctx, module };
        taken = sim
            .initiate_special_power_update(update, caller, template_id, None, None, options)
            .expect("facet");
    });
    taken
}

fn power_id(sim: &SimContext, name: &str) -> u32 {
    sim.rules
        .special_power_store
        .find_special_power_template(name)
        .expect("power")
        .id()
}

fn silo_vector(sim: &mut SimContext, line: &str) -> usize {
    let variant = get_u32(line, "variant").expect("variant");
    let mut frame = get_u32(line, "frame").expect("frame");
    sim.world.frame = frame;
    sim.world.next_object_id = get_u32(line, "id").expect("id") as i32;
    let at = Coord3D::new(
        f32::from_bits(get_u32(line, "x").expect("x")),
        f32::from_bits(get_u32(line, "y").expect("y")),
        0.0,
    );
    let object = spawn(sim, &format!("P341_Silo{variant}"), 1, at);
    let module = modules_named(sim, object, "MissileLauncherBuildingUpdate")[0];
    let powers = modules_named(sim, object, "SpecialAbility");
    let silo = Silo {
        object,
        module,
        powers: [powers[0], powers[1]],
    };
    if get_u32(line, "construction").expect("construction") == 1 {
        sim.set_status(object, ObjectStatusType::UnderConstruction);
    }
    sim.world.triggered_fx_lists.clear();
    let (ops, mut rows) = script(line);
    let actual = silo_row(sim, &silo);
    rows.check(actual, get_str(line, "spawn").expect("spawn"), "silo spawn");
    for (k, &[code, a, b, c]) in ops.iter().enumerate() {
        sim.world.frame = frame;
        match code {
            0 => {
                for _ in 0..a {
                    frame += 1;
                    sim.world.frame = frame;
                    assert_eq!(update(sim, module), 1);
                }
            }
            1 => do_power(sim, silo.powers[0], 0),
            2 => do_power(sim, silo.powers[1], 0),
            3 => {
                let when = (i64::from(frame) + a) as u32;
                sim.with_special_power_module(silo.powers[0], &mut |spm, sim, ctx| {
                    spm.set_ready_frame(sim, ctx, when);
                });
            }
            4 => sim.with_special_power_module(silo.powers[0], &mut |spm, sim, ctx| {
                spm.pause_countdown(sim, ctx, a as Bool);
            }),
            5 => {
                if a != 0 {
                    sim.set_status(object, ObjectStatusType::UnderConstruction);
                } else {
                    sim.clear_status(object, ObjectStatusType::UnderConstruction);
                }
            }
            6 => {
                let name = format!("P341_{}{variant}", if a >> 1 != 0 { "B" } else { "A" });
                let id = power_id(sim, &name);
                let taken = initiate(sim, silo.powers[0], module, id, 0);
                assert_eq!(i64::from(taken), a & 1, "initiate, op {k}");
            }
            _ => panic!("op {code}"),
        }
        let actual = silo_row(sim, &silo);
        let expected = rows.rows.next().expect("row");
        rows.check(
            actual,
            expected,
            &format!("silo op {k} {:?}", [code, a, b, c]),
        );
    }
    ops.len()
}

// --- point defense ----------------------------------------------------------

fn laser_row(
    sim: &mut SimContext,
    module: ModuleSlot,
    targets: &[ObjectId],
    ret: u32,
) -> Vec<String> {
    let mut out = vec![
        save_crc(sim, module),
        sim.world.rng.get_game_logic_random_seed_crc().to_string(),
    ];
    let mut shots = String::new();
    for (owner, _, victim, _) in std::mem::take(&mut sim.world.weapon_fires) {
        // `TERTIARY_WEAPON`; the recorded seam keeps no slot.
        let _ = write!(shots, "{{{}:2:{}}}", owner.0, victim.map_or(0, |v| v.0));
    }
    out.push(or_dash(shots));
    out.push(ret.to_string());
    out.push(
        targets
            .iter()
            .map(|&id| match sim.find_object(id) {
                None => 'x',
                Some(slot) => {
                    let dead = sim
                        .world
                        .objects
                        .get(slot)
                        .expect("target")
                        .is_effectively_dead();
                    if dead != FALSE {
                        'd'
                    } else {
                        'a'
                    }
                }
            })
            .collect(),
    );
    out
}

fn set_target_status(sim: &mut SimContext, slot: ObjectSlot, flags: i64) {
    use ObjectStatusType as S;
    for (bit, status) in [S::AirborneTarget, S::Stealthed, S::Detected, S::Disguised]
        .into_iter()
        .enumerate()
    {
        if flags & (1 << bit) != 0 {
            sim.set_status(slot, status);
        }
    }
}

fn from_bits(s: &str) -> f32 {
    f32::from_bits(s.parse().expect("float bits"))
}

fn laser_vector(sim: &mut SimContext, line: &str) -> usize {
    let variant = get_u32(line, "variant").expect("variant");
    let mut frame = get_u32(line, "frame").expect("frame");
    sim.world.frame = frame;
    sim.world.next_object_id = get_u32(line, "id").expect("id") as i32;
    sim.world
        .rng
        .init_random_with_seed(get_u32(line, "seed").expect("seed"));
    let object = spawn(
        sim,
        &format!("P350_Laser{variant}"),
        1,
        Coord3D::new(500.0, 500.0, 0.0),
    );
    let module = modules_named(sim, object, "PointDefenseLaserUpdate")[0];
    let mut targets = Vec::new();
    for target in get_str(line, "targets").expect("targets").split(';') {
        let f: Vec<&str> = target.split(',').collect();
        let at = Coord3D::new(from_bits(f[2]), from_bits(f[3]), from_bits(f[4]));
        let slot = spawn(
            sim,
            &format!("P350_Target{}", f[0]),
            f[1].parse().expect("team"),
            at,
        );
        set_target_status(sim, slot, f[5].parse().expect("status"));
        targets.push(sim.world.objects.id_of(slot).expect("id"));
    }
    sim.update_partition();
    sim.world.weapon_fires.clear();
    let (ops, mut rows) = script(line);
    let actual = laser_row(sim, module, &targets, 0);
    rows.check(
        actual,
        get_str(line, "spawn").expect("spawn"),
        "laser spawn",
    );
    for (k, &[code, a, b, c]) in ops.iter().enumerate() {
        sim.world.frame = frame;
        let mut ret = 0;
        let target = |sim: &SimContext| sim.find_object(targets[a as usize]).expect("target");
        match code {
            0 => {
                for _ in 0..a {
                    frame += 1;
                    sim.world.frame = frame;
                    ret = update(sim, module);
                }
            }
            1 => {
                let slot = target(sim);
                let z = sim
                    .world
                    .objects
                    .get(slot)
                    .expect("target")
                    .thing
                    .get_position()
                    .z;
                let p = Coord3D::new(f32::from_bits(b as u32), f32::from_bits(c as u32), z);
                sim.set_object_position(slot, &p);
                sim.update_partition();
            }
            2 => {
                let slot = target(sim);
                use ObjectStatusType as S;
                for status in [S::AirborneTarget, S::Stealthed, S::Detected, S::Disguised] {
                    sim.clear_status(slot, status);
                }
                set_target_status(sim, slot, b);
            }
            3 => {
                let slot = target(sim);
                kill_object(sim, slot, DamageType::Unresistable, DeathType::Normal);
            }
            4 => {
                let slot = target(sim);
                sim.destroy_object(slot);
                sim.process_destroy_list();
                sim.update_partition();
            }
            5 => {
                let slot = target(sim);
                sim.set_object_team(slot, TEAMS[b as usize]);
            }
            6 => kill_object(sim, object, DamageType::Unresistable, DeathType::Normal),
            _ => panic!("op {code}"),
        }
        let actual = laser_row(sim, module, &targets, ret);
        let expected = rows.rows.next().expect("row");
        rows.check(
            actual,
            expected,
            &format!("variant {variant} laser op {k} {:?}", [code, a, b, c]),
        );
    }
    ops.len()
}

// --- battle plan ------------------------------------------------------------

#[derive(Default)]
struct Plans {
    centers: Vec<ObjectId>,
    members: Vec<ObjectId>,
}

impl Plans {
    fn member(&mut self, sim: &mut SimContext, kind: i64, team: i64, producer: i64) {
        let at = Coord3D::new(200.0 + 10.0 * self.members.len() as f32, 200.0, 0.0);
        let slot = spawn(sim, &format!("P309_Member{kind}"), team as usize, at);
        let source = usize::try_from(producer)
            .ok()
            .and_then(|p| self.members.get(p))
            .copied()
            .filter(|&id| sim.find_object(id).is_some());
        if let Some(source) = source {
            sim.world.objects.get_mut(slot).expect("member").producer_id = source;
        }
        self.members
            .push(sim.world.objects.id_of(slot).expect("id"));
    }

    fn center(&mut self, sim: &mut SimContext, variant: i64, team: i64) {
        let at = Coord3D::new(300.0 + 40.0 * self.centers.len() as f32, 300.0, 0.0);
        let slot = spawn(sim, &format!("P309_Center{variant}"), team as usize, at);
        self.centers
            .push(sim.world.objects.id_of(slot).expect("id"));
    }
}

fn player_text(sim: &SimContext, player: usize) -> String {
    let chose = sim
        .world
        .player_academy
        .get(&TEAMS[player])
        .is_some_and(|a| a.chose_a_strategy_for_center != FALSE);
    let plans = sim
        .player_battle_plans(player as i32)
        .cloned()
        .unwrap_or_default();
    let mut s = format!(
        "{},{},{},{}",
        plans.bombardment_plans,
        plans.hold_the_line_plans,
        plans.search_and_destroy_plans,
        i32::from(chose)
    );
    if let Some(b) = plans.bonuses {
        let _ = write!(
            s,
            ",{},{},{},{},{}",
            b.armor_scalar.to_bits(),
            b.sight_range_scalar.to_bits(),
            b.bombardment,
            b.hold_the_line,
            b.search_and_destroy
        );
    }
    s
}

fn unit_text(sim: &mut SimContext, slot: ObjectSlot) -> String {
    let body = sim
        .world
        .objects
        .get(slot)
        .and_then(|o| o.body_slot)
        .expect("body");
    let scalar = sim
        .dispatch_module(body, |m, _, _| {
            m.as_body_mut().expect("body").get_damage_scalar()
        })
        .expect("dispatch");
    let health = sim.object_health(slot);
    let max = sim.object_max_health(slot);
    let o = sim.world.objects.get(slot).expect("unit");
    format!(
        "{},{},{},{},{},{},{},{}",
        health.to_bits(),
        max.to_bits(),
        scalar.to_bits(),
        o.vision_range.to_bits(),
        o.shroud_clearing_range.to_bits(),
        o.weapon_bonus_condition,
        // `getDisabledUntil` reports a type only while its bit is set.
        if o.disabled_mask.test(DisabledType::Paralyzed.as_i32()) != 0 {
            o.disabled_till_frame[DisabledType::Paralyzed as usize]
        } else {
            0
        },
        sim.player_index(o.team_id)
    )
}

fn plan_row(sim: &mut SimContext, plans: &Plans) -> Vec<String> {
    let mut out = vec![
        player_text(sim, 1),
        player_text(sim, 2),
        or_dash(ORDERS.with_borrow_mut(std::mem::take)),
    ];
    for &id in &plans.centers {
        let Some(slot) = sim.find_object(id) else {
            out.extend(["x", "x", "x"].map(str::to_owned));
            continue;
        };
        let module = modules_named(sim, slot, "BattlePlanUpdate")[0];
        out.push(save_crc(sim, module));
        let detector = modules_named(sim, slot, "StealthDetectorUpdate");
        out.push(match detector.first() {
            Some(&d) => save_crc(sim, d),
            None => "-".to_owned(),
        });
        let unit = unit_text(sim, slot);
        use ModelConditionFlagType as F;
        let model = &sim.world.objects.get(slot).expect("center").model_condition;
        let doors: String = [
            F::Door1Opening,
            F::Door1WaitingToClose,
            F::Door1Closing,
            F::Door2Opening,
            F::Door2WaitingToClose,
            F::Door2Closing,
            F::Door3Opening,
            F::Door3WaitingToClose,
            F::Door3Closing,
        ]
        .iter()
        .map(|f| flag(model.test(f.as_i32()) != 0))
        .collect();
        let mut text = format!("{doors},{unit}");
        let (plan, option, active) = sim
            .dispatch_module(module, |m, _, _| {
                let active = m
                    .as_special_power_update_mut()
                    .expect("special power update")
                    .is_active();
                let b = m
                    .as_any()
                    .and_then(|a| a.downcast_ref::<BattlePlanUpdate>())
                    .expect("battle plan");
                (b.active_battle_plan() as i32, b.command_option(), active)
            })
            .expect("dispatch");
        let o = sim.world.objects.get(slot).expect("center");
        let _ = write!(
            text,
            ",{}{},{plan},{option},{active}",
            flag(o.weapon_set_flags.test(WeaponSetType::Veteran.as_i32()) != 0),
            o.weapon_set.is_cur_weapon_locked()
        );
        if let Some(t) = TURRETS.with_borrow(|t| t.get(&id).copied()) {
            let _ = write!(text, ",{}{}", u8::from(t.enabled), u8::from(t.recentering));
        }
        out.push(text);
    }
    for &id in &plans.members {
        out.push(match sim.find_object(id) {
            Some(slot) => unit_text(sim, slot),
            None => "x".to_owned(),
        });
    }
    out
}

fn update_centers(sim: &mut SimContext, plans: &Plans) {
    for &id in &plans.centers {
        if let Some(slot) = sim.find_object(id) {
            let module = modules_named(sim, slot, "BattlePlanUpdate")[0];
            assert_eq!(update(sim, module), 1);
        }
    }
}

fn plan_vector(sim: &mut SimContext, line: &str) -> usize {
    let mut frame = get_u32(line, "frame").expect("frame");
    sim.world.frame = frame;
    sim.world.next_object_id = get_u32(line, "id").expect("id") as i32;
    sim.world
        .rng
        .init_random_with_seed(get_u32(line, "seed").expect("seed"));
    let mut plans = Plans::default();
    for item in get_str(line, "setup")
        .expect("setup")
        .split(';')
        .filter(|i| !i.is_empty())
    {
        let f: Vec<i64> = item[1..]
            .split(',')
            .map(|v| v.parse().expect("setup number"))
            .collect();
        if item.starts_with('m') {
            plans.member(sim, f[0], f[1], f[2]);
        } else {
            plans.center(sim, f[0], f[1]);
        }
    }
    ORDERS.with_borrow_mut(String::clear);
    let (ops, mut rows) = script(line);
    let actual = plan_row(sim, &plans);
    rows.check(actual, get_str(line, "spawn").expect("spawn"), "plan spawn");
    for (k, &[code, a, b, c]) in ops.iter().enumerate() {
        sim.world.frame = frame;
        let center = |sim: &SimContext| sim.find_object(plans.centers[a as usize]).expect("center");
        match code {
            0 => {
                for _ in 0..a {
                    frame += 1;
                    sim.world.frame = frame;
                    update_centers(sim, &plans);
                }
            }
            1 => {
                let slot = center(sim);
                let power = modules_named(sim, slot, "SpecialAbility")[0];
                do_power(sim, power, b as u32);
            }
            2 => {
                TURRETS.with_borrow_mut(|t| {
                    if let Some(t) = t.get_mut(&plans.centers[a as usize]) {
                        t.turned = b == 0;
                    }
                });
            }
            3 => {
                let slot = center(sim);
                sim.set_object_team(slot, TEAMS[b as usize]);
            }
            4 => plans.member(sim, a, b, c),
            5 => {
                if let Some(slot) = sim.find_object(plans.members[a as usize]) {
                    sim.set_object_team(slot, TEAMS[b as usize]);
                }
            }
            6 => {
                let slot = center(sim);
                sim.destroy_object(slot);
                sim.process_destroy_list();
            }
            7 => {
                if let Some(slot) = sim.find_object(plans.members[a as usize]) {
                    sim.destroy_object(slot);
                    sim.process_destroy_list();
                }
            }
            8 => {
                // `b == 0` marks the harness's full list, which does nothing.
                if b != 0 {
                    plans.center(sim, a, b);
                }
            }
            _ => panic!("op {code}"),
        }
        let actual = plan_row(sim, &plans);
        let expected = rows.rows.next().expect("row");
        rows.check(
            actual,
            expected,
            &format!("plan op {k} {:?}", [code, a, b, c]),
        );
    }
    ops.len()
}

#[test]
fn support_modules_match_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT341_350_309_Support.json");
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"kind\""))
        .collect();
    assert_eq!(lines.len(), 3000);
    let mut sim = context();
    let mut checked = [0usize; 3];
    for (index, line) in lines.iter().enumerate() {
        reset(&mut sim);
        let kind = get_u32(line, "kind").expect("kind") as usize;
        let run = [silo_vector, laser_vector, plan_vector][kind];
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run(&mut sim, line)));
        match result {
            Ok(n) => checked[kind] += n,
            Err(e) => {
                eprintln!("vector {index}");
                std::panic::resume_unwind(e);
            }
        }
    }
    assert!(checked.iter().all(|&n| n > 15_000), "only {checked:?} rows");
}

// --- checks without a reference ---------------------------------------------

fn field<T: 'static, R>(sim: &mut SimContext, slot: ModuleSlot, f: impl FnOnce(&T) -> R) -> R {
    sim.dispatch_module(slot, |m, _, _| {
        f(m.as_any()
            .and_then(|a| a.downcast_ref::<T>())
            .expect("type"))
    })
    .expect("dispatch")
}

/// The module's own fields: the header before them holds the live schedule, which a load
/// alone does not change.
fn own(bytes: &[u8]) -> &[u8] {
    &bytes[9..]
}

#[test]
fn saved_state_loads_back() {
    let mut sim = context();
    reset(&mut sim);
    sim.world.frame = 100;
    let at = Coord3D::new(300.0, 300.0, 0.0);

    let silo = spawn(&mut sim, "P341_Silo0", 1, at);
    let source = modules_named(&mut sim, silo, "MissileLauncherBuildingUpdate")[0];
    let power = modules_named(&mut sim, silo, "SpecialAbility")[0];
    let id = power_id(&sim, "P341_A0");
    assert_eq!(initiate(&mut sim, power, source, id, 0), TRUE);
    let saved = save(&mut sim, source);
    let other = spawn(&mut sim, "P341_Silo0", 1, at);
    let target = modules_named(&mut sim, other, "MissileLauncherBuildingUpdate")[0];
    load(&mut sim, target, &saved);
    let state = field(&mut sim, target, |m: &MissileLauncherBuildingUpdate| {
        (m.door_state, m.timeout_state, m.timeout_frame)
    });
    assert_eq!(state, (DoorState::WaitingToClose, DoorState::Closing, 115));
    assert_eq!(own(&save(&mut sim, target)), own(&saved));

    let laser = spawn(&mut sim, "P350_Laser2", 1, at);
    let source = modules_named(&mut sim, laser, "PointDefenseLaserUpdate")[0];
    let enemy = spawn(&mut sim, "P350_Target0", 2, Coord3D::new(310.0, 300.0, 0.0));
    let enemy_id = sim.world.objects.id_of(enemy).expect("id");
    sim.update_partition();
    update(&mut sim, source);
    let saved = save(&mut sim, source);
    let other = spawn(&mut sim, "P350_Laser2", 1, at);
    let target = modules_named(&mut sim, other, "PointDefenseLaserUpdate")[0];
    load(&mut sim, target, &saved);
    let state = field(&mut sim, target, |m: &PointDefenseLaserUpdate| {
        (m.best_target_id, m.in_range, m.next_scan_frames)
    });
    assert_eq!(state, (enemy_id, TRUE, 15));
    assert_eq!(own(&save(&mut sim, target)), own(&saved));

    let center = spawn(&mut sim, "P309_Center4", 1, at);
    let source = modules_named(&mut sim, center, "BattlePlanUpdate")[0];
    let power = modules_named(&mut sim, center, "SpecialAbility")[0];
    do_power(&mut sim, power, 0x4000);
    for frame in 101..120 {
        sim.world.frame = frame;
        update(&mut sim, source);
    }
    let saved = save(&mut sim, source);
    let other = spawn(&mut sim, "P309_Center0", 1, at);
    let target = modules_named(&mut sim, other, "BattlePlanUpdate")[0];
    load(&mut sim, target, &saved);
    field(&mut sim, target, |m: &BattlePlanUpdate| {
        assert_eq!(m.active_battle_plan(), BattlePlanStatus::HoldTheLine);
        assert_eq!(m.bonuses.hold_the_line, 1);
        assert_eq!(m.bonuses.armor_scalar, 1.3);
    });
    // The kind-of masks come from the save, not from the loading object's data.
    assert_eq!(own(&save(&mut sim, target)), own(&saved));
}

/// An aircraft's projectile takes the weapon bonuses of its launcher's kind and neither
/// armor nor sight; one without a producer takes nothing.
#[test]
fn projectiles_follow_their_producer() {
    let mut sim = context();
    reset(&mut sim);
    sim.world.frame = 10;
    let mut plans = Plans::default();
    plans.center(&mut sim, 0, 1);
    plans.member(&mut sim, 0, 1, -1);
    plans.member(&mut sim, 4, 1, 0);
    plans.member(&mut sim, 4, 1, -1);
    let center = sim.find_object(plans.centers[0]).expect("center");
    let power = modules_named(&mut sim, center, "SpecialAbility")[0];
    do_power(&mut sim, power, 0x8000);
    for frame in 11..40 {
        sim.world.frame = frame;
        update_centers(&mut sim, &plans);
    }
    let record = |sim: &SimContext, i: usize| {
        let slot = sim.find_object(plans.members[i]).expect("member");
        let o = sim.world.objects.get(slot).expect("member");
        (o.weapon_bonus_condition, o.vision_range)
    };
    assert_eq!(record(&sim, 0), (1 << 14, 80.0 * 1.2));
    assert_eq!(record(&sim, 1), (1 << 14, 80.0));
    assert_eq!(record(&sim, 2), (0, 80.0));
}

/// Fields the INI leaves out keep the reference constructor's values, and an order for
/// another power or without a plan option is refused.
#[test]
fn data_defaults_and_refused_orders() {
    let data = BattlePlanUpdateModuleData::new(NameKeyType::default(), NameKeyType::default());
    assert_eq!(
        (
            data.bombardment_plan_animation_frames,
            data.hold_the_line_plan_animation_frames,
            data.search_and_destroy_plan_animation_frames,
            data.transition_idle_frames,
            data.battle_plan_paralyze_frames,
        ),
        (0, 0, 0, 0, 0)
    );
    assert_eq!(
        (
            data.hold_the_line_armor_damage_scalar,
            data.search_and_destroy_sight_range_scalar,
            data.strategy_center_search_and_destroy_sight_range_scalar,
            data.strategy_center_hold_the_line_max_health_scalar,
        ),
        (1.0, 1.0, 1.0, 1.0)
    );
    assert_eq!(
        data.strategy_center_search_and_destroy_detects_stealth,
        TRUE
    );
    assert_eq!(
        data.strategy_center_hold_the_line_max_health_change_type,
        MaxHealthChangeType::PreserveRatio
    );

    let mut sim = context();
    reset(&mut sim);
    sim.world.frame = 10;
    let center = spawn(&mut sim, "P309_Center0", 1, Coord3D::new(300.0, 300.0, 0.0));
    let module = modules_named(&mut sim, center, "BattlePlanUpdate")[0];
    let power = modules_named(&mut sim, center, "SpecialAbility")[0];
    let own = power_id(&sim, "P309_Power");
    let other = power_id(&sim, "P341_A0");
    assert_eq!(initiate(&mut sim, power, module, other, 0x2000), FALSE);
    assert_eq!(initiate(&mut sim, power, module, own, 0), FALSE);
    assert_eq!(
        field(&mut sim, module, |m: &BattlePlanUpdate| m.desired_plan),
        BattlePlanStatus::None
    );
    assert_eq!(initiate(&mut sim, power, module, own, 0x8000), TRUE);
    assert_eq!(
        field(&mut sim, module, |m: &BattlePlanUpdate| m.desired_plan),
        BattlePlanStatus::SearchAndDestroy
    );
}
