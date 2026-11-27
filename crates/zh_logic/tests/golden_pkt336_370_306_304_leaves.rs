//! `PKT-336` `HordeUpdate`, `PKT-370` `SpectreGunshipDeploymentUpdate`,
//! `PKT-306` `AutoFindHealingUpdate` and `PKT-304` `AssistedTargetingUpdate`
//! against the reference.
//!
//! Vectors: `docs/golden_vectors/PKT336_370_306_304_Leaves.json`, written by
//! `tests/cpp_harness/gen_pkt336_370_306_304_leaves.cpp`. Every row is compared
//! exactly.
//!
//! The AI is a test stand-in: it records orders, answers `is_idle` and the
//! turret question from flags, and makes the PRNG draws the real AI makes when
//! it is built.

use std::any::Any;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};

use zh_common::golden_json::{extract_vector_lines, get_i32_vec, get_str};
use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{
    Bool, NameKeyType, ObjectId, Real, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::module::AiUpdateFacet;
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::horde_update::TerrainDecalType;
use zh_logic::modules::{BehaviorModule, CommandSourceType, ModuleData, ModuleType};
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::rules::combat_types::WeaponStatus;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{
    ClientSink, RulesRegistry, SimApi, SimContext, SimView, SimWorld,
    PRIVATE_STATUS_EFFECTIVELY_DEAD,
};
use zh_logic::spatial::terrain::{find_closest_edge_point, find_farthest_edge_point};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT336_370_306_304_Leaves.json");

/// Neutral, A, B, in player order.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];

// -----------------------------------------------------------------------------
// AI stand-in
// -----------------------------------------------------------------------------

thread_local! {
    static ORDERS: RefCell<String> = const { RefCell::new(String::new()) }; // determinism-ok: test spy, not simulation state
}

#[derive(Debug, Clone)]
struct AiData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    /// Weapon slot bits on the main turret.
    turret_slots: u8,
}

impl ModuleData for AiData {
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
        FacetMask(0x001 | FacetMask::AI_UPDATE.0)
    }
}

struct Ai {
    data: AiData,
    busy: bool,
    turret_enabled: bool,
}

impl AiUpdateFacet for Ai {
    fn is_idle(&self) -> Bool {
        Bool::from(!self.busy)
    }

    fn ai_get_healed(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        heal_unit: ObjectSlot,
        cmd_source: CommandSourceType,
    ) {
        let target = sim.object(heal_unit).expect("heal unit").id;
        ORDERS.with(|o| {
            let _ = write!(
                o.borrow_mut(),
                " H {} {} {}",
                ctx.owner_id.0,
                target.0,
                cmd_source as i32
            );
        });
    }

    fn ai_attack_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        victim: ObjectSlot,
        max_shots_to_fire: i32,
        cmd_source: CommandSourceType,
    ) {
        let target = sim.object(victim).expect("victim").id;
        ORDERS.with(|o| {
            let _ = write!(
                o.borrow_mut(),
                " A {} {} {max_shots_to_fire} {}",
                ctx.owner_id.0,
                target.0,
                cmd_source as i32
            );
        });
    }

    fn is_weapon_slot_enabled(&self, slot: WeaponSlotType) -> Bool {
        let on_turret = self.data.turret_slots & (1 << slot as u8) != 0;
        Bool::from(!on_turret || self.turret_enabled)
    }
}

impl BehaviorModule for Ai {
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
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

/// `Turret ... ControlledWeaponSlots = A B ... End` is all it reads.
fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AiData {
        tag_key,
        module_name_key,
        turret_slots: 0,
    };
    reader.read_block_fields(|field, r| {
        if !field.eq_ignore_ascii_case("Turret") {
            return Err(IniError::UnknownToken);
        }
        r.read_block_fields(|inner, r| {
            if !inner.eq_ignore_ascii_case("ControlledWeaponSlots") {
                return Err(IniError::UnknownToken);
            }
            while let Ok(token) = r.next_token(SEPS_DEFAULT) {
                let bit = ["PRIMARY", "SECONDARY", "TERTIARY"]
                    .iter()
                    .position(|n| token.eq_ignore_ascii_case(n))
                    .expect("slot name");
                data.turret_slots |= 1 << bit;
            }
            Ok(())
        })
    })?;
    Ok(Box::new(data))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let data = data.as_any().downcast_ref::<AiData>().expect("ai data");
    Box::new(Ai {
        data: data.clone(),
        busy: false,
        turret_enabled: true,
    })
}

// -----------------------------------------------------------------------------
// Drawable and terrain
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
struct Decal {
    kind: TerrainDecalType,
    w: Real,
    h: Real,
    fade_target: Real,
    fade_rate: Real,
}

impl Default for Decal {
    fn default() -> Self {
        Self {
            kind: TerrainDecalType::None,
            w: 0.0,
            h: 0.0,
            fade_target: 0.0,
            fade_rate: 0.0,
        }
    }
}

#[derive(Default)]
struct Drawn {
    decals: BTreeMap<i32, Decal>,
    /// Laser id to (parent, target, start, end).
    lasers: BTreeMap<i32, (ObjectId, ObjectId, Coord3D, Coord3D)>,
}

type Shared = Arc<Mutex<Drawn>>; // determinism-ok: test spy, not simulation state

struct Sink(Shared);

impl ClientSink for Sink {
    fn on_terrain_decal(&mut self, id: ObjectId, decal: TerrainDecalType) {
        let mut drawn = self.0.lock().expect("drawn");
        drawn.decals.entry(id.0).or_default().kind = decal;
    }
    fn on_terrain_decal_size(&mut self, id: ObjectId, x: Real, y: Real) {
        let mut drawn = self.0.lock().expect("drawn");
        let d = drawn.decals.entry(id.0).or_default();
        (d.w, d.h) = (x, y);
    }
    fn on_terrain_decal_fade_target(&mut self, id: ObjectId, target: Real, rate: Real) {
        let mut drawn = self.0.lock().expect("drawn");
        let d = drawn.decals.entry(id.0).or_default();
        if d.fade_target != target {
            (d.fade_target, d.fade_rate) = (target, rate);
        }
    }
    fn on_laser_init(
        &mut self,
        laser: ObjectId,
        parent: ObjectId,
        target: ObjectId,
        start: Coord3D,
        end: Coord3D,
    ) {
        let mut drawn = self.0.lock().expect("drawn");
        drawn.lasers.insert(laser.0, (parent, target, start, end));
    }
}

fn slope(x: Real, y: Real) -> Real {
    x * 0.25 + y * 0.125
}

struct Slope;

impl ThingTerrainAccess for Slope {
    fn get_ground_height(&self, x: Real, y: Real, _normal: Option<&mut Coord3D>) -> Real {
        slope(x, y)
    }
    fn align_on_terrain(
        &self,
        _angle: Real,
        _pos: &Coord3D,
        _stick_to_ground: Bool,
        _mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
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
}

// -----------------------------------------------------------------------------
// Fixture: the same data the harness loads
// -----------------------------------------------------------------------------

struct HordeVariant {
    kinds: &'static str,
    rate: &'static str,
    filter: &'static str,
    count: i32,
    radius: i32,
    rub: i32,
    allies_only: i32,
    exact: bool,
    action: &'static str,
    nationalism: i32,
    geometry: i32,
    flag: bool,
}

#[allow(clippy::too_many_arguments)]
const fn hv(
    kinds: &'static str,
    rate: &'static str,
    filter: &'static str,
    count: i32,
    radius: i32,
    rub: i32,
    allies_only: i32,
    exact: bool,
    action: &'static str,
    nationalism: i32,
    geometry: i32,
    flag: bool,
) -> HordeVariant {
    HordeVariant {
        kinds,
        rate,
        filter,
        count,
        radius,
        rub,
        allies_only,
        exact,
        action,
        nationalism,
        geometry,
        flag,
    }
}

const HORDE: [HordeVariant; 10] = [
    hv(
        "INFANTRY", "100", "INFANTRY", 3, 40, 20, -1, false, "", -1, 5, true,
    ),
    hv(
        "INFANTRY", "", "INFANTRY", 2, 30, -1, 1, false, "HORDE", 1, 5, false,
    ),
    hv(
        "INFANTRY",
        "200",
        "INFANTRY",
        5,
        60,
        40,
        0,
        false,
        "HORDE_FIXED",
        -1,
        10,
        false,
    ),
    hv(
        "VEHICLE", "100", "VEHICLE", 3, 40, 30, -1, false, "", -1, 10, true,
    ),
    hv(
        "VEHICLE",
        "300",
        "VEHICLE",
        2,
        25,
        5,
        0,
        true,
        "HORDE_FIXED",
        0,
        5,
        false,
    ),
    hv(
        "VEHICLE PORTABLE_STRUCTURE",
        "100",
        "VEHICLE",
        2,
        40,
        20,
        -1,
        false,
        "",
        -1,
        5,
        false,
    ),
    hv(
        "INFANTRY VEHICLE",
        "100",
        "INFANTRY VEHICLE",
        2,
        40,
        20,
        -1,
        false,
        "",
        -1,
        5,
        false,
    ),
    hv("INFANTRY", "33", "", 4, 50, 20, 0, false, "", -1, 20, false),
    hv("VEHICLE", "100", "", 0, 20, 20, 0, false, "", -1, 5, false),
    hv(
        "INFANTRY",
        "100",
        "INFANTRY",
        1,
        0,
        50,
        -1,
        true,
        "HORDE_FIXED",
        1,
        5,
        false,
    ),
];

const SCAN_RATE: [&str; 4] = ["", "33", "100", "300"];
const SCAN_RANGE: [i32; 4] = [0, 50, 100, 150];
const NEVER_HEAL: [&str; 4] = ["", "0.5", "1.0", "0.25"];

fn body() -> &'static str {
    "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n"
}

fn ai(extra: &str, loco: &str) -> String {
    format!(
        "  Behavior = OracleLeafAI ModuleTag_AI\n{extra}  End\n  Locomotor = SET_NORMAL {loco}\n"
    )
}

fn geometry(radius: i32) -> String {
    format!("  Geometry = CYLINDER\n  GeometryMajorRadius = {radius}\n  GeometryHeight = 10\n")
}

fn yes(b: bool) -> &'static str {
    if b {
        "Yes"
    } else {
        "No"
    }
}

fn object_ini() -> String {
    let mut s = String::from("Object P_Prop\n  KindOf = PRELOAD\nEnd\n");
    for (v, h) in HORDE.iter().enumerate() {
        for has_ai in 0..2 {
            let _ = write!(
                s,
                "Object P336_{v}_{has_ai}\n  KindOf = PRELOAD {}\n",
                h.kinds
            );
            s += body();
            s += &geometry(h.geometry);
            if has_ai == 1 {
                s += &ai("", "P_Ground");
            }
            s += "  Behavior = HordeUpdate ModuleTag_Horde\n";
            if !h.rate.is_empty() {
                let _ = writeln!(s, "    UpdateRate = {}", h.rate);
            }
            if !h.filter.is_empty() {
                let _ = writeln!(s, "    KindOf = {}", h.filter);
            }
            let _ = writeln!(s, "    Count = {}\n    Radius = {}", h.count, h.radius);
            if h.rub >= 0 {
                let _ = writeln!(s, "    RubOffRadius = {}", h.rub);
            }
            if h.allies_only >= 0 {
                let _ = writeln!(s, "    AlliesOnly = {}", yes(h.allies_only != 0));
            }
            if h.exact {
                s += "    ExactMatch = Yes\n";
            }
            if !h.action.is_empty() {
                let _ = writeln!(s, "    Action = {}", h.action);
            }
            if h.nationalism >= 0 {
                let _ = writeln!(s, "    AllowedNationalism = {}", yes(h.nationalism != 0));
            }
            if h.flag {
                s += "    FlagSubObjectNames = FLAG01 FLAG02\n";
            }
            s += "  End\nEnd\n";
        }
    }
    let _ = write!(
        s,
        "Object P336_Outsider\n  KindOf = PRELOAD INFANTRY VEHICLE\n{}{}End\n",
        body(),
        geometry(5)
    );

    for v in 0..8 {
        let pad = if v == 3 { " HEAL_PAD" } else { "" };
        let _ = write!(s, "Object P306_{v}\n  KindOf = PRELOAD INFANTRY{pad}\n");
        if v != 5 {
            s += body();
        }
        s += &geometry(5);
        if v != 6 {
            s += &ai("", "P_Ground");
        }
        s += "  Behavior = AutoFindHealingUpdate ModuleTag_Find\n";
        if !SCAN_RATE[v % 4].is_empty() {
            let _ = writeln!(s, "    ScanRate = {}", SCAN_RATE[v % 4]);
        }
        let _ = writeln!(s, "    ScanRange = {}", SCAN_RANGE[(v + 1) % 4]);
        if !NEVER_HEAL[(v / 2) % 4].is_empty() {
            let _ = writeln!(s, "    NeverHeal = {}", NEVER_HEAL[(v / 2) % 4]);
        }
        if v % 2 == 1 {
            s += "    AlwaysHeal = 0.9\n";
        }
        s += "  End\nEnd\n";
    }
    for (name, kinds) in [
        ("P306_Pad", "STRUCTURE HEAL_PAD"),
        ("P306_Other", "STRUCTURE"),
    ] {
        let _ = write!(
            s,
            "Object {name}\n  KindOf = PRELOAD {kinds}\n{}{}End\n",
            body(),
            geometry(15)
        );
    }

    for g in 0..3 {
        let _ = write!(
            s,
            "Object P370_Gunship{g}\n  KindOf = PRELOAD VEHICLE AIRCRAFT SELECTABLE\n{}{}{}",
            body(),
            geometry(10),
            ai("", &format!("P_Gun{g}"))
        );
        if g != 1 {
            let _ = write!(
                s,
                "  Behavior = SpecialAbility ModuleTag_SP\n    SpecialPowerTemplate = P_SP0\n    UpdateModuleStartsAttack = {}\n  End\n",
                yes(g == 2)
            );
        }
        s += "End\n";
    }
    // power list, starts-attack, the update's power, create location, gunship
    // The last two carry a second update, for P_SP0 with another gunship.
    let spectres: [(&str, bool, i32, i32, i32); 8] = [
        ("0", true, 0, -1, 0),
        ("0", false, 0, 0, 1),
        ("10", true, 0, 1, 2),
        ("00", true, 0, 2, 0),
        ("01", false, 0, 3, 2),
        ("10", false, 1, 0, 0),
        ("01", false, 1, 2, 0),
        ("0", true, 0, 1, 0),
    ];
    let locations = [
        "CREATE_AT_EDGE_NEAR_SOURCE",
        "CREATE_AT_EDGE_FARTHEST_FROM_SOURCE",
        "CREATE_AT_EDGE_NEAR_TARGET",
        "CREATE_AT_EDGE_FARTHEST_FROM_TARGET",
    ];
    for (v, (powers, starts, mine, loc, gunship)) in spectres.into_iter().enumerate() {
        let _ = write!(
            s,
            "Object P370_Source{v}\n  KindOf = PRELOAD STRUCTURE\n{}{}",
            body(),
            geometry(30)
        );
        for (k, power) in powers.chars().enumerate() {
            let _ = write!(
                s,
                "  Behavior = SpecialAbility ModuleTag_SP{k}\n    SpecialPowerTemplate = P_SP{power}\n    UpdateModuleStartsAttack = {}\n  End\n",
                yes(starts)
            );
        }
        let _ = write!(
            s,
            "  Behavior = SpectreGunshipDeploymentUpdate ModuleTag_Spectre\n    SpecialPowerTemplate = P_SP{mine}\n    GunshipTemplateName = P370_Gunship{gunship}\n    AttackAreaRadius = 150\n"
        );
        if loc >= 0 {
            let _ = writeln!(s, "    CreateLocation = {}", locations[loc as usize]);
        }
        s += "  End\n";
        if v >= 6 {
            s += "  Behavior = SpectreGunshipDeploymentUpdate ModuleTag_Second\n    SpecialPowerTemplate = P_SP0\n    GunshipTemplateName = P370_Gunship2\n  End\n";
        }
        s += "End\n";
    }

    s += "Object P304_Laser\n  KindOf = PRELOAD INERT\n  ClientUpdate = LaserUpdate ModuleTag_Laser\n  End\nEnd\n";
    s += "Object P304_Plain\n  KindOf = PRELOAD INERT\nEnd\n";
    let _ = write!(
        s,
        "Object P304_Victim\n  KindOf = PRELOAD STRUCTURE\n{}{}End\n",
        body(),
        geometry(10)
    );
    for c in 0..2 {
        let _ = write!(
            s,
            "Object P304_Bunker{c}\n  KindOf = PRELOAD STRUCTURE\n{}{}  Behavior = TransportContain ModuleTag_Contain\n    Slots = 4\n    PassengersAllowedToFire = {}\n    AllowInsideKindOf = INFANTRY\n  End\nEnd\n",
            body(),
            geometry(20),
            yes(c == 1)
        );
    }

    // kinds, weapons, AI, enter guard, clip, slot, from, to, own hold, turret
    type Assist = (
        &'static str,
        u8,
        bool,
        bool,
        i32,
        &'static str,
        &'static str,
        &'static str,
        bool,
        u8,
    );
    const L: &str = "P304_Laser";
    const P: &str = "P304_Plain";
    const A: &str = "PRIMARY";
    const B: &str = "SECONDARY";
    let assists: [Assist; 14] = [
        ("STRUCTURE", 3, true, false, 4, A, L, L, false, 0),
        (
            "STRUCTURE CAN_ATTACK",
            3,
            true,
            false,
            -1,
            B,
            P,
            L,
            false,
            0,
        ),
        (
            "STRUCTURE",
            2,
            true,
            false,
            1,
            B,
            "Nothing_Here",
            "",
            false,
            0,
        ),
        ("STRUCTURE CAN_ATTACK", 0, true, false, 2, A, L, P, false, 0),
        (
            "STRUCTURE CAN_ATTACK",
            3,
            false,
            false,
            3,
            A,
            L,
            L,
            false,
            0,
        ),
        ("INFANTRY", 1, false, true, 5, A, "", L, false, 0),
        (
            "STRUCTURE PORTABLE_STRUCTURE",
            3,
            true,
            false,
            2,
            "TERTIARY",
            "",
            "",
            false,
            0,
        ),
        (
            "STRUCTURE SPAWNS_ARE_THE_WEAPONS",
            3,
            true,
            false,
            2,
            A,
            "",
            "",
            false,
            0,
        ),
        ("INFANTRY", 0, true, false, 2, A, "", "", false, 0),
        ("INFANTRY", 3, false, false, 2, A, "", "", true, 0),
        ("STRUCTURE", 3, true, false, 2, A, "", "", false, 1),
        ("STRUCTURE", 3, true, false, 2, B, L, "", false, 2),
        ("STRUCTURE", 2, true, false, 2, B, "", "", false, 1),
        ("INFANTRY", 3, true, false, 7, B, L, L, false, 0),
    ];
    for (v, (kinds, weapons, has_ai, guard, clip, slot, from, to, hold, turret)) in
        assists.into_iter().enumerate()
    {
        let _ = write!(
            s,
            "Object P304_{v}\n  KindOf = PRELOAD {kinds}\n{}{}",
            body(),
            geometry(8)
        );
        if guard {
            s += "  EnterGuard = Yes\n";
        }
        if weapons != 0 {
            s += "  WeaponSet\n    Conditions = None\n";
            if weapons & 1 != 0 {
                s += "    Weapon = PRIMARY P_Weapon0\n";
            }
            if weapons & 2 != 0 {
                s += "    Weapon = SECONDARY P_Weapon1\n";
            }
            s += "  End\n";
        }
        if has_ai {
            let turret = match turret {
                0 => String::new(),
                1 => "    Turret\n      ControlledWeaponSlots = PRIMARY\n    End\n".to_owned(),
                _ => "    Turret\n      ControlledWeaponSlots = PRIMARY SECONDARY\n    End\n"
                    .to_owned(),
            };
            s += &ai(&turret, "P_Ground");
        }
        if hold {
            s += "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 4\n    PassengersAllowedToFire = Yes\n    AllowInsideKindOf = INFANTRY\n  End\n";
        }
        s += "  Behavior = AssistedTargetingUpdate ModuleTag_Assist\n";
        if clip >= 0 {
            let _ = writeln!(s, "    AssistingClipSize = {clip}");
        }
        let _ = writeln!(s, "    AssistingWeaponSlot = {slot}");
        if !from.is_empty() {
            let _ = writeln!(s, "    LaserFromAssisted = {from}");
        }
        if !to.is_empty() {
            let _ = writeln!(s, "    LaserToTarget = {to}");
        }
        s += "  End\nEnd\n";
    }
    let _ = write!(
        s,
        "Object P304_Rider\n  KindOf = PRELOAD INFANTRY\n  TransportSlotCount = 1\n{}{}End\n",
        body(),
        geometry(2)
    );
    s
}

fn reader<'a>(name: &str, text: &'a str) -> IniReader<'a> {
    IniReader::new(name, text.as_bytes(), IniLoadType::Overwrite)
}

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "OracleLeafAI",
        ModuleType::Behavior,
        FacetMask(0x001 | FacetMask::AI_UPDATE.0),
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    let upgrades = "Upgrade Upgrade_Nationalism\n  Type = PLAYER\nEnd\nUpgrade Upgrade_Fanaticism\n  Type = PLAYER\nEnd\n";
    rules
        .upgrade_center
        .load_from_ini(&mut reader("upgrade.ini", upgrades), &mut rules.name_keys)
        .expect("upgrades");
    let common = "  TurnRate = 180\n  Acceleration = 60\n  Braking = 60\n";
    let mut locos = vec![(
        "P_Ground".to_owned(),
        "  Surfaces = GROUND\n  Speed = 60\n".to_owned(),
    )];
    for (g, height) in [70, 0, 125].into_iter().enumerate() {
        locos.push((
            format!("P_Gun{g}"),
            format!("  Surfaces = AIR\n  Speed = 60\n  MinSpeed = 0\n  Lift = 100\n  PreferredHeight = {height}\n"),
        ));
    }
    for (name, extra) in locos {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(&format!("Locomotor {name}\n{extra}{common}End\n"))
            .expect("locomotor");
        t.name = name;
        rules.locomotor_store.add_template(t);
    }
    for w in 0..2 {
        let mut t = WeaponTemplate::new();
        t.parse_ini_block(&format!(
            "Weapon P_Weapon{w}\n  PrimaryDamage = 1\n  AttackRange = 100\n  DelayBetweenShots = 500\n  ClipSize = {}\n  ClipReloadTime = 1000\nEnd",
            if w == 1 { 3 } else { 0 }
        ))
        .expect("weapon");
        rules.weapon_store.add_template(t);
    }
    let powers = "SpecialPower P_SP0\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 3000\nEnd\nSpecialPower P_SP1\n  Enum = SPECIAL_PARADROP_AMERICA\n  ReloadTime = 7000\nEnd\n";
    rules
        .special_power_store
        .load_from_ini(
            &mut reader("sp.ini", powers),
            &rules.science_store,
            &mut rules.name_keys,
        )
        .expect("special powers");
    let ini = object_ini();
    let mut r = reader("object.ini", &ini);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut r) {
        panic!("thing templates, line {}: {e:?}", r.get_line_num());
    }
    rules
}

fn relate(sim: &mut SimContext, a: i32, b: i32, rel: Relationship) {
    sim.world.player_relationships.insert((a, b), rel);
    sim.world.player_relationships.insert((b, a), rel);
}

fn reset(sim: &mut SimContext) -> Shared {
    let drawn = Shared::default();
    sim.client = Box::new(Sink(drawn.clone()));
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
        w.player_is_human.insert(team, FALSE);
    }
    w.team_is_neutral.insert(TEAMS[0], TRUE);
    w.neutral_team_id = TEAMS[0];
    w.local_player_index = 0;
    w.special_power_uses_delay = TRUE;
    for i in 0..3 {
        relate(sim, i, i, Relationship::Allies);
    }
    relate(sim, 0, 1, Relationship::Neutral);
    relate(sim, 0, 2, Relationship::Neutral);
    relate(sim, 1, 2, Relationship::Enemies);
    ORDERS.with(|o| o.borrow_mut().clear());
    drawn
}

// -----------------------------------------------------------------------------
// Shared helpers
// -----------------------------------------------------------------------------

/// `newObject`, then `setPosition`, as the harness's spawn does.
fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = sim.spawn_object(tid, team, Coord3D::new(0.0, 0.0, 0.0), 0.0);
    sim.set_object_position(slot, &pos);
    slot
}

fn at(x: i32, y: i32, z: i32) -> Coord3D {
    Coord3D::new(x as Real, y as Real, z as Real)
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    modules_named(sim, owner, name).first().copied()
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

fn module_state(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let save = save_hex(sim, slot);
    let mut xc = XferCrc::new();
    sim.world
        .modules
        .payload(slot)
        .expect("payload")
        .crc(&mut xc);
    format!("{save} {} {}", xc.get_crc(), next_call_frame(sim, slot))
}

fn update(sim: &mut SimContext, m: ModuleSlot) -> UpdateSleepTime {
    sim.dispatch_module(m, |b, s, c| b.as_update_mut().expect("update").update(s, c))
        .expect("dispatch")
}

/// The harness's single-module scheduler step.
fn step(sim: &mut SimContext, m: ModuleSlot, frame: u32) {
    if next_call_frame(sim, m) > frame {
        return;
    }
    let sleep = update(sim, m);
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

fn set_dead(sim: &mut SimContext, o: ObjectSlot, dead: bool) {
    let obj = sim.world.objects.get_mut(o).expect("object");
    if dead {
        obj.private_status |= PRIVATE_STATUS_EFFECTIVELY_DEAD;
    } else {
        obj.private_status &= !PRIVATE_STATUS_EFFECTIVELY_DEAD;
    }
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

fn table(line: &str, key: &str) -> Vec<Vec<i64>> {
    let text = get_str(line, key).expect(key);
    if text.is_empty() {
        return Vec::new();
    }
    text.split(';')
        .map(|row| row.split(' ').map(|n| n.parse().expect("number")).collect())
        .collect()
}

struct Case<'a> {
    index: usize,
    setup: Vec<i32>,
    units: Vec<Vec<i64>>,
    ops: Vec<Vec<i64>>,
    rows: Vec<&'a str>,
}

impl Case<'_> {
    fn check(&self, row: usize, got: &str) {
        assert_eq!(
            got, self.rows[row],
            "vector {} row {row} (setup {:?}, units {:?}, ops {:?})",
            self.index, self.setup, self.units, self.ops
        );
    }
}

fn cases(kind: i32) -> Vec<Case<'static>> {
    extract_vector_lines(VECTORS)
        .into_iter()
        .enumerate()
        .filter(|(_, line)| zh_common::golden_json::get_i32(line, "kind") == Some(kind))
        .map(|(index, line)| {
            let case = Case {
                index,
                setup: get_i32_vec(line, "setup").expect("setup"),
                units: table(line, "units"),
                ops: table(line, "ops"),
                rows: get_str(line, "rows").expect("rows").split('#').collect(),
            };
            assert_eq!(case.rows.len(), case.ops.len() + 1);
            case
        })
        .collect()
}

// -----------------------------------------------------------------------------
// HordeUpdate
// -----------------------------------------------------------------------------

struct Unit {
    slot: ObjectSlot,
    horde: Option<ModuleSlot>,
}

fn horde_row(sim: &mut SimContext, units: &[Unit], drawn: &Shared) -> String {
    let mut out = sim.world.rng.get_game_logic_random_seed_crc().to_string();
    for u in units {
        out += " |";
        match u.horde {
            Some(m) => {
                let state = module_state(sim, m);
                let horde = sim
                    .world
                    .modules
                    .payload(m)
                    .and_then(|p| p.as_horde())
                    .expect("horde facet");
                let _ = write!(
                    out,
                    " {state} {}{}",
                    horde.is_in_horde(),
                    horde.is_true_horde_member()
                );
            }
            None => out += " -",
        }
        let obj = sim.world.objects.get(u.slot).expect("unit");
        let decal = drawn
            .lock()
            .expect("drawn")
            .decals
            .get(&obj.id.0)
            .copied()
            .unwrap_or_default();
        let _ = write!(
            out,
            " {} {} {} {} {} {}",
            obj.weapon_bonus_condition,
            decal.kind as i32,
            decal.w.to_bits(),
            decal.h.to_bits(),
            decal.fade_target.to_bits(),
            decal.fade_rate.to_bits()
        );
    }
    out
}

fn set_upgrade(sim: &mut SimContext, which: i64, on: bool) {
    let name = if which & 1 == 1 {
        "Upgrade_Fanaticism"
    } else {
        "Upgrade_Nationalism"
    };
    let key = sim.rules.name_keys.name_to_key(name);
    let mask = sim
        .rules
        .upgrade_center
        .find_upgrade_by_key(key)
        .expect("upgrade")
        .upgrade_mask()
        .clone();
    let team = TEAMS[if which < 2 { 1 } else { 2 }];
    let player = sim.player_index(team);
    let owned = sim.world.player_upgrades.entry(player).or_default();
    if on {
        owned.set_flags(&mask);
    } else {
        owned.clear_flags(&mask);
    }
}

#[test]
fn test_pkt336_horde_update_golden() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    let cases = cases(0);
    assert!(cases.len() >= 400);
    for case in &cases {
        let drawn = reset(&mut sim);
        let &[seed, start, relation, upgrades, base] = case.setup.as_slice() else {
            panic!("setup");
        };
        let mut frame = start as u32;
        sim.world.frame = frame;
        let rel = [
            Relationship::Enemies,
            Relationship::Neutral,
            Relationship::Allies,
        ][relation as usize];
        relate(&mut sim, 1, 2, rel);
        for k in 0..4 {
            set_upgrade(&mut sim, k, (upgrades >> k) & 1 != 0);
        }
        sim.world.next_object_id = base;
        sim.world.rng.init_random_with_seed(seed as u32);

        let units: Vec<Unit> = case
            .units
            .iter()
            .map(|u| {
                let &[variant, has_ai, team, x, y, z] = u.as_slice() else {
                    panic!("unit");
                };
                let name = if variant < 0 {
                    "P336_Outsider".to_owned()
                } else {
                    format!("P336_{variant}_{has_ai}")
                };
                let slot = spawn(
                    &mut sim,
                    &name,
                    TEAMS[team as usize],
                    at(x as i32, y as i32, z as i32),
                );
                Unit {
                    slot,
                    horde: module_named(&mut sim, slot, "HordeUpdate"),
                }
            })
            .collect();
        sim.update_partition();

        // Binding the drawable hides the flag sub objects (not in the vectors:
        // client state).
        for (unit, spec) in units.iter().zip(&case.units) {
            let hidden = &sim
                .world
                .objects
                .get(unit.slot)
                .expect("unit")
                .sub_objects_hidden;
            let flag = spec[0] >= 0 && HORDE[spec[0] as usize].flag;
            assert_eq!(hidden.contains("FLAG01") && hidden.contains("FLAG02"), flag);
        }

        let row = horde_row(&mut sim, &units, &drawn);
        case.check(0, &row);
        for (k, op) in case.ops.iter().enumerate() {
            let &[code, a, b, c] = op.as_slice() else {
                panic!("op");
            };
            let mut ret = 0;
            sim.world.frame = frame;
            match code {
                0 => {
                    for _ in 0..a {
                        frame += 1;
                        sim.world.frame = frame;
                        for u in &units {
                            if let Some(m) = u.horde {
                                step(&mut sim, m, frame);
                            }
                        }
                    }
                }
                1 => {
                    let slot = units[a as usize].slot;
                    let z = sim
                        .world
                        .objects
                        .get(slot)
                        .expect("unit")
                        .thing
                        .get_position()
                        .z;
                    sim.set_object_position(slot, &at(b as i32, c as i32, z as i32));
                    sim.update_partition();
                }
                2 => {
                    let slot = units[a as usize].slot;
                    let dead = sim
                        .world
                        .objects
                        .get(slot)
                        .expect("unit")
                        .is_effectively_dead();
                    set_dead(&mut sim, slot, dead == FALSE);
                }
                3 => set_upgrade(&mut sim, a, b != 0),
                4 => sim.world.draw_icon_ui = Bool::from(a != 0),
                5 => ret = update(&mut sim, units[a as usize].horde.expect("horde")).0,
                _ => panic!("op code"),
            }
            let row = format!("{} = {ret}", horde_row(&mut sim, &units, &drawn));
            case.check(k + 1, &row);
        }
    }
}

// -----------------------------------------------------------------------------
// AutoFindHealingUpdate
// -----------------------------------------------------------------------------

#[test]
fn test_pkt306_auto_find_healing_update_golden() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    let cases = cases(2);
    assert!(cases.len() >= 400);
    for case in &cases {
        reset(&mut sim);
        let &[variant, human, start, base, x, y, seeker_first] = case.setup.as_slice() else {
            panic!("setup");
        };
        let mut frame = start as u32;
        sim.world.frame = frame;
        sim.world
            .player_is_human
            .insert(TEAMS[1], Bool::from(human != 0));
        sim.world.next_object_id = base;
        sim.world.rng.init_random_with_seed(7);
        let make_seeker =
            |sim: &mut SimContext| spawn(sim, &format!("P306_{variant}"), TEAMS[1], at(x, y, 0));
        let early = (seeker_first != 0).then(|| make_seeker(&mut sim));
        let pads: Vec<ObjectSlot> = case
            .units
            .iter()
            .enumerate()
            .map(|(i, u)| {
                let &[is_pad, px, py, team] = u.as_slice() else {
                    panic!("pad");
                };
                spawn(
                    &mut sim,
                    if is_pad != 0 {
                        "P306_Pad"
                    } else {
                        "P306_Other"
                    },
                    TEAMS[team as usize],
                    at(px as i32, py as i32, 5 * i as i32),
                )
            })
            .collect();
        let seeker = early.unwrap_or_else(|| make_seeker(&mut sim));
        sim.update_partition();
        let m = module_named(&mut sim, seeker, "AutoFindHealingUpdate").expect("module");
        let body_slot = sim.world.objects.get(seeker).expect("seeker").body_slot;
        let ai_slot = sim.world.objects.get(seeker).expect("seeker").ai_slot;

        let row = |sim: &mut SimContext, ret: u32| {
            let state = module_state(sim, m);
            let health = if body_slot.is_some() {
                sim.object_health(seeker).to_bits()
            } else {
                0
            };
            let orders = ORDERS.with(|o| std::mem::take(&mut *o.borrow_mut()));
            format!("{state} {ret} {health}{orders}")
        };
        let got = row(&mut sim, 0);
        case.check(0, &got);
        for (k, op) in case.ops.iter().enumerate() {
            let &[code, a, b, c] = op.as_slice() else {
                panic!("op");
            };
            let mut ret = 0;
            sim.world.frame = frame;
            match code {
                0 => {
                    for _ in 0..a {
                        frame += 1;
                        sim.world.frame = frame;
                        step(&mut sim, m, frame);
                    }
                }
                1 => {
                    if let Some(body) = body_slot {
                        sim.dispatch_module(body, |module, sim, ctx| {
                            module.as_body_mut().expect("body").internal_change_health(
                                sim,
                                ctx,
                                a as Real * 0.25,
                            );
                        })
                        .expect("body");
                    }
                }
                2 => {
                    if let Some(ai) = ai_slot {
                        sim.dispatch_module(ai, |module, _, _| {
                            module
                                .as_any_mut()
                                .and_then(|m| m.downcast_mut::<Ai>())
                                .expect("ai")
                                .busy = a != 0;
                        })
                        .expect("ai");
                    }
                }
                3 => {
                    sim.world
                        .player_is_human
                        .insert(TEAMS[1], Bool::from(a != 0));
                }
                4 => {
                    sim.set_object_position(
                        pads[a as usize],
                        &at(b as i32, c as i32, 5 * a as i32),
                    );
                    sim.update_partition();
                }
                5 => ret = update(&mut sim, m).0,
                _ => panic!("op code"),
            }
            let got = row(&mut sim, ret);
            case.check(k + 1, &got);
        }
    }
}

// -----------------------------------------------------------------------------
// SpectreGunshipDeploymentUpdate
// -----------------------------------------------------------------------------

struct Selection {
    seen: usize,
    by_player: [Vec<i32>; 3],
}

fn spectre_row(
    sim: &mut SimContext,
    source: ObjectSlot,
    m: ModuleSlot,
    powers: &[ModuleSlot],
    selection: &mut Selection,
    ret: u32,
) -> String {
    let seed = sim.world.rng.get_game_logic_random_seed_crc();
    let state = module_state(sim, m);
    let mask = sim
        .world
        .modules
        .payload(m)
        .and_then(|p| p.as_update())
        .expect("update")
        .disabled_types_to_process();
    let mut out = format!("{seed} {state} {ret} ");
    let bits: Vec<i32> = (0..zh_logic::rules::disabled_types::DISABLED_COUNT as i32)
        .map(|d| i32::from(mask.test(d) != FALSE))
        .collect();
    let _ = write!(out, "{}", bits.iter().sum::<i32>());
    for b in bits {
        let _ = write!(out, "{b}");
    }
    for &p in powers {
        let _ = write!(out, " {}", save_hex(sim, p));
    }
    let mut others: Vec<(i32, ObjectSlot)> = sim
        .world_object_slots()
        .into_iter()
        .filter(|&s| s != source)
        .map(|s| (sim.world.objects.get(s).expect("object").id.0, s))
        .collect();
    others.sort_unstable_by_key(|&(id, _)| id);
    for (id, slot) in others {
        let o = sim.world.objects.get(slot).expect("object");
        let player = sim.player_index(o.team_id);
        let _ = write!(out, " G {id} {} {player}", o.producer_id.0);
        put(&mut out, o.thing.get_position());
        let _ = write!(out, " {}", o.thing.get_orientation().to_bits());
        for p in modules_named(sim, slot, "SpecialAbility") {
            let _ = write!(out, " {}", save_hex(sim, p));
        }
    }
    for &(id, create_new, mask, _) in &sim.world.selected_objects[selection.seen..] {
        assert_eq!(create_new, TRUE);
        for (player, held) in selection.by_player.iter_mut().enumerate() {
            if mask & (1 << player) != 0 {
                *held = vec![id.0];
            }
        }
    }
    selection.seen = sim.world.selected_objects.len();
    // The special power module reaches the update itself, not the stand-in.
    assert!(sim.world.special_ability_events.is_empty());
    for held in &selection.by_player {
        out.push(' ');
        if held.is_empty() {
            out.push('-');
        }
        for id in held {
            let _ = write!(out, "{id},");
        }
    }
    out
}

#[test]
fn test_pkt370_spectre_gunship_deployment_update_golden() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    sim.terrain = Box::new(Slope);
    let cases = cases(1);
    assert!(cases.len() >= 400);
    for case in &cases {
        reset(&mut sim);
        let &[variant, start, base, team, sx, sy] = case.setup.as_slice() else {
            panic!("setup");
        };
        let mut frame = start as u32;
        sim.world.frame = frame;
        sim.world.next_object_id = base;
        sim.world.rng.init_random_with_seed(11);
        let source = spawn(
            &mut sim,
            &format!("P370_Source{variant}"),
            TEAMS[team as usize],
            at(sx, sy, 7),
        );
        let m = module_named(&mut sim, source, "SpectreGunshipDeploymentUpdate").expect("module");
        let powers = modules_named(&mut sim, source, "SpecialAbility");
        let mut selection = Selection {
            seen: 0,
            by_player: Default::default(),
        };
        let row = spectre_row(&mut sim, source, m, &powers, &mut selection, 0);
        case.check(0, &row);
        for (k, op) in case.ops.iter().enumerate() {
            let &[code, a, b, c, d] = op.as_slice() else {
                panic!("op");
            };
            let mut ret = 0;
            sim.world.frame = frame;
            match code {
                0 => {
                    let target = at(b as i32, c as i32, 3);
                    sim.dispatch_module(powers[a as usize], |module, sim, ctx| {
                        module
                            .as_special_power_mut()
                            .expect("power")
                            .do_special_power_at_location(sim, ctx, &target, -100.0, d as u32);
                    })
                    .expect("fire");
                }
                1 => {
                    for _ in 0..a {
                        frame += 1;
                        sim.world.frame = frame;
                        step(&mut sim, m, frame);
                    }
                }
                2 => {
                    sim.dispatch_module(powers[a as usize], |module, sim, ctx| {
                        module
                            .as_special_power_mut()
                            .expect("power")
                            .pause_countdown(sim, ctx, Bool::from(b != 0));
                    })
                    .expect("pause");
                }
                3 => match a {
                    0 | 1 => {
                        let status = if a == 0 {
                            ObjectStatusType::Sold
                        } else {
                            ObjectStatusType::UnderConstruction
                        };
                        if b != 0 {
                            sim.set_status(source, status);
                        } else {
                            sim.clear_status(source, status);
                        }
                    }
                    2 => set_dead(&mut sim, source, b != 0),
                    _ => {
                        if b != 0 {
                            sim.set_disabled(source, DisabledType::Hacked);
                        } else {
                            let _ = sim.clear_disabled(source, DisabledType::Hacked);
                        }
                    }
                },
                4 => ret = update(&mut sim, m).0,
                _ => panic!("op code"),
            }
            let row = spectre_row(&mut sim, source, m, &powers, &mut selection, ret);
            case.check(k + 1, &row);
        }
    }
}

// -----------------------------------------------------------------------------
// AssistedTargetingUpdate
// -----------------------------------------------------------------------------

struct Scene {
    me: ObjectSlot,
    requester: ObjectSlot,
    victim: ObjectSlot,
    module: ModuleSlot,
    /// Objects that are not lasers.
    cast: BTreeSet<i32>,
}

fn assist_row(sim: &mut SimContext, s: &Scene, drawn: &Shared, ret: u32) -> String {
    let seed = sim.world.rng.get_game_logic_random_seed_crc();
    let state = module_state(sim, s.module);
    let obj = sim.world.objects.get(s.me).expect("assistant");
    let set = &obj.weapon_set;
    let any = set.has_any_weapon() != FALSE
        && set
            .get_weapon_in_weapon_slot(set.get_cur_weapon_slot())
            .is_some();
    let mut out = format!(
        "{seed} {state} {ret} {}{}{}",
        u32::from(any),
        if any {
            set.get_cur_weapon_slot() as i32
        } else {
            0
        },
        u32::from(set.is_cur_weapon_locked() != FALSE)
    );
    for slot in [
        WeaponSlotType::Primary,
        WeaponSlotType::Secondary,
        WeaponSlotType::Tertiary,
    ] {
        match set.get_weapon_in_weapon_slot(slot) {
            Some(w) => {
                let _ = write!(out, " {}:{}", w.status as i32, w.ammo_in_clip);
            }
            None => out += " -",
        }
    }
    out += &ORDERS.with(|o| std::mem::take(&mut *o.borrow_mut()));

    let mut lasers: Vec<(i32, ObjectSlot)> = sim
        .world_object_slots()
        .into_iter()
        .map(|slot| (sim.world.objects.get(slot).expect("object").id.0, slot))
        .filter(|(id, _)| !s.cast.contains(id))
        .collect();
    lasers.sort_unstable_by_key(|&(id, _)| id);
    for &(id, slot) in &lasers {
        let o = sim.world.objects.get(slot).expect("laser");
        let pos = *o.thing.get_position();
        let _ = write!(out, " L {id} {}", sim.player_index(o.team_id));
        put(&mut out, &pos);
        match drawn.lock().expect("drawn").lasers.remove(&id) {
            Some((parent, target, start, end)) => {
                // Client only: the start is `from`'s position, where the laser sits.
                assert_eq!(start, pos);
                assert_ne!(target, ObjectId::INVALID);
                let _ = write!(out, " {}", parent.0);
                put(&mut out, &end);
            }
            None => out += " -",
        }
    }
    for (_, slot) in lasers {
        sim.destroy_object_now(slot);
    }
    sim.process_destroy_list();
    out
}

fn stage_weapon(sim: &mut SimContext, o: ObjectSlot, op: &[i64], frame: u32) {
    let [slot, status, ammo, again, pre] = [op[1], op[2], op[3], op[4], op[5]];
    let slot = [
        WeaponSlotType::Primary,
        WeaponSlotType::Secondary,
        WeaponSlotType::Tertiary,
    ][slot as usize];
    let obj = sim.world.objects.get_mut(o).expect("assistant");
    let Some(weapon) = obj.weapon_set.get_weapon_in_weapon_slot_mut(slot) else {
        return;
    };
    weapon.status = [
        WeaponStatus::ReadyToFire,
        WeaponStatus::OutOfAmmo,
        WeaponStatus::BetweenFiringShots,
        WeaponStatus::ReloadingClip,
    ][status as usize];
    weapon.ammo_in_clip = ammo as u32;
    weapon.when_we_can_fire_again = (i64::from(frame) + again) as u32;
    weapon.when_pre_attack_finished = if pre == -100 {
        0
    } else {
        (i64::from(frame) + pre) as u32
    };
}

#[test]
fn test_pkt304_assisted_targeting_update_golden() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    sim.terrain = Box::new(Slope);
    let cases = cases(3);
    assert!(cases.len() >= 400);
    for case in &cases {
        let drawn = reset(&mut sim);
        let &[variant, start, base, container] = case.setup.as_slice() else {
            panic!("setup");
        };
        let mut frame = start as u32;
        sim.world.frame = frame;
        sim.world.next_object_id = base;
        sim.world.rng.init_random_with_seed(13);
        let me = spawn(
            &mut sim,
            &format!("P304_{variant}"),
            TEAMS[1],
            at(400, 400, 4),
        );
        let requester = spawn(&mut sim, "P304_Victim", TEAMS[1], at(450, 410, 9));
        let victim = spawn(&mut sim, "P304_Victim", TEAMS[2], at(520, 470, 2));
        let mut cast_slots = vec![me, requester, victim];
        if container != 0 {
            let name = format!("P304_Bunker{}", container - 1);
            let bunker = spawn(&mut sim, &name, TEAMS[1], at(600, 600, 0));
            sim.contain_add(bunker, me);
            cast_slots.push(bunker);
        }
        let module = module_named(&mut sim, me, "AssistedTargetingUpdate").expect("module");
        let cast = cast_slots
            .iter()
            .map(|&s| sim.world.objects.get(s).expect("object").id.0)
            .collect();
        let mut s = Scene {
            me,
            requester,
            victim,
            module,
            cast,
        };
        let mut riders: Vec<ObjectSlot> = Vec::new();
        let row = assist_row(&mut sim, &s, &drawn, 0);
        case.check(0, &row);
        for (k, op) in case.ops.iter().enumerate() {
            let (code, a, b) = (op[0], op[1], op[2]);
            let mut ret = 0;
            sim.world.frame = frame;
            match code {
                0 => {
                    ret = u32::from(sim.request_assist_from(s.me, s.requester, s.victim) != FALSE);
                }
                1 => {
                    let (requester, victim) = (s.requester, s.victim);
                    sim.dispatch_module(s.module, |m, sim, ctx| {
                        m.as_assisted_targeting_mut()
                            .expect("assist")
                            .assist_attack(sim, ctx, requester, victim);
                    })
                    .expect("assist");
                }
                2 => stage_weapon(&mut sim, s.me, op, frame),
                3 => {
                    if a < 4 {
                        let status = [
                            ObjectStatusType::NoAttack,
                            ObjectStatusType::UnderConstruction,
                            ObjectStatusType::Sold,
                            ObjectStatusType::CanAttack,
                        ][a as usize];
                        if b != 0 {
                            sim.set_status(s.me, status);
                        } else {
                            sim.clear_status(s.me, status);
                        }
                    } else {
                        let kind = [
                            DisabledType::Subdued,
                            DisabledType::Hacked,
                            DisabledType::Emp,
                        ][a as usize - 4];
                        if b != 0 {
                            sim.set_disabled(s.me, kind);
                        } else {
                            let _ = sim.clear_disabled(s.me, kind);
                        }
                    }
                }
                4 => {
                    for _ in 0..a {
                        frame += 1;
                        sim.world.frame = frame;
                        step(&mut sim, s.module, frame);
                    }
                }
                5 => {
                    if let Some(ai) = sim.world.objects.get(s.me).expect("me").ai_slot {
                        sim.dispatch_module(ai, |m, _, _| {
                            m.as_any_mut()
                                .and_then(|m| m.downcast_mut::<Ai>())
                                .expect("ai")
                                .turret_enabled = a != 0;
                        })
                        .expect("ai");
                    }
                }
                6 => {
                    if variant == 9 {
                        if a != 0 && riders.len() < 3 {
                            let rider = spawn(&mut sim, "P304_Rider", TEAMS[1], at(400, 400, 0));
                            s.cast
                                .insert(sim.world.objects.get(rider).expect("rider").id.0);
                            sim.contain_add(s.me, rider);
                            riders.push(rider);
                        } else if a == 0 {
                            if let Some(rider) = riders.pop() {
                                sim.contain_remove(s.me, rider, FALSE);
                            }
                        }
                    }
                }
                7 => ret = update(&mut sim, s.module).0,
                _ => panic!("op code"),
            }
            // A request answers whether it attacked, as the harness does.
            let row = assist_row(&mut sim, &s, &drawn, ret);
            case.check(k + 1, &row);
        }
    }
}

// -----------------------------------------------------------------------------
// Map edge points
// -----------------------------------------------------------------------------

#[test]
fn test_pkt370_edge_points_golden() {
    let cases = cases(4);
    assert!(cases.len() >= 200);
    for case in &cases {
        let &[lx, ly, hx, hy] = case.setup.as_slice() else {
            panic!("setup");
        };
        let extent = Region3D {
            lo: at(lx, ly, 0),
            hi: at(hx, hy, 100),
        };
        case.check(0, "-");
        for (k, op) in case.ops.iter().enumerate() {
            let p = at(op[0] as i32, op[1] as i32, 5);
            let mut out = String::new();
            put(&mut out, &find_closest_edge_point(&Slope, &extent, &p));
            put(&mut out, &find_farthest_edge_point(&Slope, &extent, &p));
            case.check(k + 1, &out);
        }
    }
}
