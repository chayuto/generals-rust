//! `PKT-354` `ProductionUpdate` against vectors from the reference module
//! (`tests/cpp_harness/gen_pkt354_production.cpp`). The exit interface, parking place and
//! create module below are the harness's three scripted stand-ins.
//!
//! The tests after the golden one check, without a reference, what the harness leaves out.

use std::cell::RefCell;
use std::fmt::Write as _;
use zh_common::core::dict::Dict;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{
    DeferredSelfOp, FacetMask, ModuleContext, ModuleSlot, ObjectSlot, ThingTemplateId,
};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::economy::energy::Energy;
use zh_logic::economy::handicap::Handicap;
use zh_logic::modules::module::{
    BehaviorModule, CreateFacet, ExitDoorType, ExitFacet, ModuleData, ModuleType,
    ParkingPlaceFacet, StubModuleData,
};
use zh_logic::modules::module::UpdateFacet;
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::production_update::{
    ProductionUpdate, ProductionUpdateModuleData, PRODUCTION_UNIT, PRODUCTION_UPGRADE,
};
use zh_logic::object::object_script_status_bits::{
    OBJECT_STATUS_SCRIPT_DISABLED, OBJECT_STATUS_SCRIPT_UNPOWERED,
};
use zh_logic::object::thing_template::BuildableStatus;
use zh_logic::rules::command_set::{CommandButton, CommandButtonType};
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceTextLookup;
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const TEAM: TeamId = TeamId(1);
const OTHER_TEAM: TeamId = TeamId(2);

const UPGRADES: [&str; 5] = ["UpgP1", "UpgP2", "UpgO1", "UpgO2", "UpgO3"];
const UNITS: [&str; 12] = [
    "U0", "U1", "U2", "U3", "U4", "U5", "U6", "U7", "U8", "U9", "UE", "UX",
];
const SET_UNITS: [&str; 11] = [
    "U0", "U1", "U2", "U3", "U4", "U5", "U6", "U7", "U8", "UE", "U9",
];

// (production module fields, has exit, has parking)
const VARIANTS: [(&str, bool, bool); 11] = [
    ("", true, false),
    ("NumDoorAnimations = 1\nDoorOpeningTime = 100\nDoorWaitOpenTime = 200\nDoorCloseTime = 100\nConstructionCompleteDuration = 100\n", true, false),
    ("NumDoorAnimations = 4\nDoorOpeningTime = 0\nDoorWaitOpenTime = 0\nDoorCloseTime = 300\nQuantityModifier = U1\n", true, false),
    ("NumDoorAnimations = 2\nDoorOpeningTime = 66\nDoorWaitOpenTime = 33\nDoorCloseTime = 400\nQuantityModifier = U1 3\nQuantityModifier = U1 5\nQuantityModifier = U0 2\nConstructionCompleteDuration = 300\n", true, false),
    ("MaxQueueEntries = 2\nQuantityModifier = U5 2\nQuantityModifier = U3\nQuantityModifier = NoSuchThing 7\nDisabledTypesToProcess = DISABLED_EMP\n", true, false),
    ("MaxQueueEntries = -1\nQuantityModifier = U1 0\nQuantityModifier = U2 -1\nNumDoorAnimations = -1\nDisabledTypesToProcess = +DISABLED_EMP\n", true, false),
    ("MaxQueueEntries = 0\n", true, false),
    ("NumDoorAnimations = 1\nDoorOpeningTime = 100\nDoorWaitOpenTime = 100\nDoorCloseTime = 100\n", false, false),
    ("NumDoorAnimations = 4\nDoorOpeningTime = 66\nDoorWaitOpenTime = 66\nDoorCloseTime = 66\nQuantityModifier = U0 4\n", true, true),
    ("QuantityModifier = U1 3\nQuantityModifier = U0 2\n", true, true),
    ("MaxQueueEntries = 3\n", false, true),
];

// ---- the scripted stand-ins -----------------------------------------------------------
#[derive(Default)]
struct Script {
    door: i32,
    budget: i32,
    hold: i32,
    hold_flag: i32,
    park_reserve: bool,
    park_space: bool,
    log: String,
}

thread_local! {
    static SCRIPT: RefCell<Script> = RefCell::new(Script::default()); // determinism-ok: test stand-in
}

fn script<R>(f: impl FnOnce(&mut Script) -> R) -> R {
    SCRIPT.with(|s| f(&mut s.borrow_mut()))
}

fn clear_log() {
    script(|s| s.log.clear());
}

fn template_name(sim: &dyn SimView, id: ThingTemplateId) -> String {
    sim.rules()
        .thing_factory
        .find_by_template_id_final(id.0 as u16)
        .expect("template")
        .name_string
        .clone()
}

macro_rules! stand_in {
    ($ty:ident, $name:literal, $facet:ident, $accessor:ident, $accessor_mut:ident) => {
        struct $ty(NameKeyType);

        impl BehaviorModule for $ty {
            fn module_name_key(&self) -> NameKeyType {
                self.0
            }
            fn xfer(
                &mut self,
                _: &mut dyn zh_common::xfer::Xfer,
                _: &dyn SimView,
            ) -> Result<(), zh_common::xfer::XferError> {
                Ok(())
            }
            fn $accessor(&self) -> Option<&dyn $facet> {
                Some(self)
            }
            fn $accessor_mut(&mut self) -> Option<&mut dyn $facet> {
                Some(self)
            }
        }
    };
}

stand_in!(
    ScriptedExit,
    "ScriptedExit",
    ExitFacet,
    as_exit,
    as_exit_mut
);
stand_in!(
    ScriptedParking,
    "ScriptedParking",
    ParkingPlaceFacet,
    as_parking_place,
    as_parking_place_mut
);

struct ScriptedBuilt(NameKeyType);

impl BehaviorModule for ScriptedBuilt {
    fn module_name_key(&self) -> NameKeyType {
        self.0
    }
    fn xfer(
        &mut self,
        _: &mut dyn zh_common::xfer::Xfer,
        _: &dyn SimView,
    ) -> Result<(), zh_common::xfer::XferError> {
        Ok(())
    }
    fn as_create_mut(&mut self) -> Option<&mut dyn CreateFacet> {
        Some(self)
    }
}

impl ExitFacet for ScriptedExit {
    fn reserve_door_for_exit(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        tmpl: ThingTemplateId,
        _specific: Option<ObjectSlot>,
    ) -> ExitDoorType {
        let name = template_name(sim, tmpl);
        script(|s| {
            let _ = write!(s.log, "R{name};");
            if s.budget == 0 {
                return ExitDoorType::DoorNoneAvailable;
            }
            if s.budget > 0 {
                s.budget -= 1;
            }
            ExitDoorType::from_i32(s.door).expect("door")
        })
    }

    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        door: ExitDoorType,
    ) {
        let factory_id = sim.object(ctx.owner_slot).expect("factory").id;
        let unit = sim.object(new_obj).expect("unit");
        // `newObject` leaves the unit at the origin until the exit moves it.
        assert_eq!(unit.thing.get_orientation(), 0.0);
        let (producer, team) = (unit.producer_id, unit.team_id);
        let name = template_name(
            sim,
            ThingTemplateId(
                unit.thing
                    .template_override()
                    .get_final(&sim.rules().thing_factory)
                    .expect("template")
                    .get_template_id()
                    .into(),
            ),
        );
        let (hold, flag) = script(|s| {
            let _ = write!(
                s.log,
                "X{},{name},{}{};",
                door.as_i32(),
                u8::from(producer == factory_id),
                u8::from(team == TEAM)
            );
            (s.hold, s.hold_flag)
        });
        if hold != -9 {
            // Doors outside the enum are ignored by the reference too.
            if let Some(door) = ExitDoorType::from_i32(hold) {
                sim.production_set_hold_door_open(ctx.owner_slot, door, Bool::from(flag != 0));
            }
        }
    }

    fn unreserve_door_for_exit(&mut self, door: ExitDoorType) {
        script(|s| {
            let _ = write!(s.log, "U{};", door.as_i32());
        });
    }
}

impl ParkingPlaceFacet for ScriptedParking {
    fn should_reserve_door_when_queued(&self, _sim: &dyn SimView, _tmpl: ThingTemplateId) -> Bool {
        script(|s| Bool::from(s.park_reserve))
    }
    fn has_available_space_for(&self, _sim: &dyn SimView, _tmpl: ThingTemplateId) -> Bool {
        script(|s| Bool::from(s.park_space))
    }
}

impl CreateFacet for ScriptedBuilt {
    fn on_create(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn on_build_complete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let unit = sim.object(ctx.owner_slot).expect("unit");
        let hijacked = unit.status.test(ObjectStatusType::Hijacked.as_i32());
        let name = unit
            .thing
            .template_override()
            .get_final(&sim.rules().thing_factory)
            .expect("template")
            .name_string
            .clone();
        script(|s| {
            let _ = write!(s.log, "B{name},{};", u8::from(hijacked != FALSE));
        });
    }
}

fn parse_stub(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, zh_common::ini::IniError> {
    reader.read_block_fields(|_, _| Err(zh_common::ini::IniError::UnknownToken))?;
    Ok(Box::new(StubModuleData {
        module_name: String::new(),
        tag_key,
        module_name_key,
        facet_mask: FacetMask::NONE,
        update_phase: SleepyUpdatePhase::Normal,
        is_ai_module: false,
    }))
}

fn make_exit(data: &dyn ModuleData, _: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(ScriptedExit(data.module_name_key()))
}
fn make_parking(data: &dyn ModuleData, _: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(ScriptedParking(data.module_name_key()))
}
fn make_built(data: &dyn ModuleData, _: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(ScriptedBuilt(data.module_name_key()))
}

// ---- fixture ----------------------------------------------------------------------------
const UPGRADE_INI: &str = "Upgrade UpgP1\n  Type = PLAYER\n  BuildTime = 0.2\n  BuildCost = 300\nEnd\n\
Upgrade UpgP2\n  Type = PLAYER\n  BuildTime = 0.0\n  BuildCost = 50\n  AcademyClassify = ACT_UPGRADE_RADAR\nEnd\n\
Upgrade UpgO1\n  Type = OBJECT\n  BuildTime = 0.1\n  BuildCost = 400\nEnd\n\
Upgrade UpgO2\n  Type = OBJECT\n  BuildTime = 0.3\n  BuildCost = 100\nEnd\n\
Upgrade UpgO3\n  Type = OBJECT\n  BuildTime = 0.1\n  BuildCost = 0\nEnd\n";

// An `InactiveBody` object is effectively dead, which `canBuildMoreOfType` skips.
const LIVE: &str = "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n  End\n";
const DEAD: &str = "  Body = InactiveBody ModuleTag_Body\n  End\n";

fn unit_ini(name: &str, body: &str) -> String {
    let health = if name == "U1" || name == "U3" {
        DEAD
    } else {
        LIVE
    };
    format!(
        "Object {name}\n{body}{health}  Behavior = StatusBitsUpgrade ModuleTag_Upg\n    TriggeredBy = UpgP1\n    StatusToSet = HIJACKED\n  End\n  Behavior = ScriptedBuilt ModuleTag_Built\n  End\nEnd\n"
    )
}

fn templates_ini() -> String {
    let mut s = String::from(
        "Object P_Pre\n  KindOf = STRUCTURE\n  Body = InactiveBody ModuleTag_Body\n  End\nEnd\n",
    );
    for (name, body) in [
        ("U0", "  KindOf = VEHICLE SCORE\n  BuildCost = 500\n  BuildTime = 0.2\n"),
        ("U1", "  KindOf = INFANTRY\n  BuildCost = 150\n  BuildTime = 0.0\n"),
        ("U2", "  KindOf = INFANTRY SCORE_CREATE HERO\n  BuildCost = 70\n  BuildTime = 0.3\n  Prerequisites\n    Object = P_Pre\n  End\n"),
        ("U3", "  KindOf = DOZER VEHICLE\n  BuildCost = 0\n  BuildTime = 0.1\n"),
        ("U4", "  KindOf = VEHICLE HARVESTER\n  BuildCost = 60\n  BuildTime = 0.1\n  MaxSimultaneousOfType = 3\n  MaxSimultaneousLinkKey = P354Link\n"),
        ("UL", "  KindOf = VEHICLE\n  MaxSimultaneousLinkKey = P354Link\n"),
        ("U5", "  KindOf = INFANTRY\n  BuildCost = 90\n  BuildTime = 0.1\n  BuildVariations = U0 U1 U3\n  BuildCompletion = PLACED_BY_PLAYER\n"),
        ("U6", "  KindOf = INFANTRY\n  BuildCost = 33\n  BuildTime = 0.4\n  Prerequisites\n    Science = SCIENCE_P354\n    Object = P_Pre U0\n  End\n"),
        ("U7", "  KindOf = VEHICLE SCORE\n  BuildCost = 40\n  BuildTime = 0.1\n  Buildable = Only_By_AI\n"),
        ("U8", "  KindOf = STRUCTURE\n  BuildCost = 1000\n  BuildTime = 0.2\n  Buildable = Ignore_Prerequisites\n  MaxSimultaneousOfType = 1\n  Prerequisites\n    Object = P_Pre\n  End\n"),
        ("U9", "  KindOf = INFANTRY\n  BuildCost = 20\n  BuildTime = 0.1\n  Buildable = No\n  BuildVariations = UE\n"),
        ("UE", "  KindOf = INFANTRY\n  BuildCost = 25\n  BuildTime = 0.1\n  MaxSimultaneousOfType = DeterminedBySuperweaponRestriction\n"),
        ("UX", "  KindOf = INFANTRY\n  BuildCost = 25\n  BuildTime = 0.1\n"),
    ] {
        s.push_str(&unit_ini(name, body));
    }
    for (i, (body, has_exit, has_parking)) in VARIANTS.iter().enumerate() {
        let _ = write!(
            s,
            "Object F{i}\n  KindOf = STRUCTURE\n  CommandSet = P354Set\n{LIVE}  Behavior = ProductionUpdate ModuleTag_Prod\n{body}  End\n"
        );
        if *has_exit {
            s.push_str("  Behavior = ScriptedExit ModuleTag_Exit\n  End\n");
        }
        if *has_parking {
            s.push_str("  Behavior = ScriptedParking ModuleTag_Park\n  End\n");
        }
        s.push_str("  Behavior = StatusBitsUpgrade ModuleTag_O1\n    TriggeredBy = UpgO1\n    StatusToSet = HIJACKED\n  End\n  Behavior = StatusBitsUpgrade ModuleTag_O2\n    TriggeredBy = UpgO2 UpgP2\n    RequiresAllTriggers = Yes\n    StatusToSet = CAN_STEALTH\n  End\n  Behavior = StatusBitsUpgrade ModuleTag_P1\n    TriggeredBy = UpgP1\n    StatusToSet = BOOBY_TRAPPED\n  End\nEnd\n");
    }
    s
}

struct SciText;

impl ScienceTextLookup for SciText {
    fn fetch_label(&mut self, label: &str) -> String {
        label.to_owned()
    }
}

fn button(command: CommandButtonType, object: &str, upgrade: &str) -> CommandButton {
    CommandButton {
        command,
        object: object.to_owned(),
        upgrade: upgrade.to_owned(),
    }
}

fn reader(text: &str) -> IniReader<'_> {
    IniReader::new("p354.ini", text.as_bytes(), IniLoadType::Overwrite)
}

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    for (name, mask, create) in [
        (
            "ScriptedExit",
            FacetMask::EXIT_INTERFACE,
            make_exit as fn(&dyn ModuleData, ModuleContext) -> Box<dyn BehaviorModule>,
        ),
        ("ScriptedParking", FacetMask::PARKING_PLACE, make_parking),
        ("ScriptedBuilt", FacetMask::CREATE, make_built),
    ] {
        rules.module_registry.register_module(
            &mut rules.name_keys,
            name,
            ModuleType::Behavior,
            mask,
            SleepyUpdatePhase::Normal,
            false,
            parse_stub,
            create,
        );
    }
    rules
        .science_store
        .load_from_ini(
            &mut reader("Science SCIENCE_P354\nEnd\n"),
            &mut rules.name_keys,
            &mut SciText,
        )
        .expect("science");
    rules
        .upgrade_center
        .load_from_ini(&mut reader(UPGRADE_INI), &mut rules.name_keys)
        .expect("upgrades");
    rules
        .load_thing_templates_from_ini(&mut reader(&templates_ini()))
        .expect("templates");
    rules.thing_factory.post_process_load();

    let mut set: Vec<CommandButton> = SET_UNITS
        .iter()
        .map(|&u| {
            let kind = if u == "U8" {
                CommandButtonType::DozerConstruct
            } else {
                CommandButtonType::UnitBuild
            };
            button(kind, u, "")
        })
        .collect();
    set.extend(
        UPGRADES
            .iter()
            .map(|&u| button(CommandButtonType::Other, "", u)),
    );
    rules.command_sets.insert("P354Set", set);
    rules.command_sets.insert(
        "P354Alt",
        vec![
            button(CommandButtonType::UnitBuild, "U1", ""),
            button(CommandButtonType::UnitBuild, "U3", ""),
            button(CommandButtonType::Other, "", "UpgP2"),
        ],
    );
    rules
}

fn world() -> SimWorld {
    let mut w = SimWorld::new();
    w.player_default_teams.insert(0, TeamId(0));
    w.player_default_teams.insert(1, TEAM);
    w.player_default_teams.insert(2, OTHER_TEAM);
    w
}

fn tid(sim: &SimContext, name: &str) -> ThingTemplateId {
    sim.rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"))
}

fn spawn_on(sim: &mut SimContext, name: &str, team: TeamId) -> ObjectSlot {
    let id = tid(sim, name);
    sim.spawn_object(id, team, Coord3D::default(), 0.0)
}

fn upgrade_key(sim: &SimContext, i: usize) -> NameKeyType {
    sim.rules
        .name_keys
        .lookup_key(UPGRADES[i])
        .expect("upgrade")
}

fn upgrade_mask(sim: &SimContext, i: usize) -> UpgradeMaskType {
    sim.rules
        .upgrade_center
        .find_upgrade_by_key(upgrade_key(sim, i))
        .expect("upgrade")
        .upgrade_mask()
        .clone()
}

fn production_slot(sim: &SimContext, owner: ObjectSlot) -> ModuleSlot {
    sim.world
        .objects
        .get(owner)
        .expect("factory")
        .behaviors
        .iter()
        .find(|(_, m)| m.contains(FacetMask::PRODUCTION_UPDATE))
        .expect("production module")
        .0
}

fn with_module<R>(
    sim: &mut SimContext,
    owner: ObjectSlot,
    f: impl FnOnce(&mut ProductionUpdate, &mut dyn SimApi, ModuleContext) -> R,
) -> R {
    let ms = production_slot(sim, owner);
    sim.dispatch_module(ms, |m, s, c| {
        let p = m
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<ProductionUpdate>())
            .expect("production");
        f(p, s, c)
    })
    .expect("dispatch")
}

fn save(sim: &mut SimContext, owner: ObjectSlot) -> Vec<u8> {
    let ms = production_slot(sim, owner);
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(ms, generation, payload);
    xs.as_bytes().to_vec()
}

fn load(sim: &mut SimContext, owner: ObjectSlot, bytes: &[u8]) -> bool {
    let ms = production_slot(sim, owner);
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).expect("payload");
    let mut xl = XferLoad::new(bytes);
    let ok = payload.xfer(&mut xl, sim).is_ok();
    sim.world.modules.restore_payload(ms, generation, payload);
    ok
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn bit(b: Bool) -> char {
    if b == FALSE {
        '0'
    } else {
        '1'
    }
}

struct Run {
    factory: ObjectSlot,
    extras: Vec<ObjectSlot>,
    variant: usize,
}

impl Run {
    /// The harness's `row`.
    fn row(&self, sim: &mut SimContext, ret: i32) -> String {
        let log = script(|s| std::mem::take(&mut s.log));
        let o = self.factory;
        let money = sim.player_money(TEAM);
        let mut s = format!("{ret} {} {money} ", if log.is_empty() { "-" } else { &log });
        for i in 0..UPGRADES.len() {
            let mask = upgrade_mask(sim, i);
            s.push(bit(sim.player_has_upgrade(TEAM, &mask)));
            s.push(bit(sim.player_has_upgrade_in_production(TEAM, &mask)));
        }
        s.push(' ');
        for i in 0..UPGRADES.len() {
            let (mask, key) = (upgrade_mask(sim, i), upgrade_key(sim, i));
            s.push(bit(sim.object_has_upgrade(o, &mask)));
            s.push(bit(sim.affected_by_upgrade(o, key)));
            s.push(bit(sim.can_produce_upgrade(o, key)));
            let module = sim.production_module(o).expect("production");
            s.push(bit(module.is_upgrade_in_queue(key)));
        }
        let obj = sim.world.objects.get(o).expect("factory");
        for status in [
            ObjectStatusType::Hijacked,
            ObjectStatusType::CanStealth,
            ObjectStatusType::BoobyTrapped,
        ] {
            s.push(bit(obj.status.test(status.as_i32())));
        }
        s.push(' ');
        let first = ModelConditionFlagType::Door1Opening.as_i32();
        for flag in (first..first + 16).chain([
            ModelConditionFlagType::ActivelyConstructing.as_i32(),
            ModelConditionFlagType::ConstructionComplete.as_i32(),
        ]) {
            s.push(bit(obj.model_condition.test(flag)));
        }
        let _ = write!(s, " {} ", sim.world.rng.get_game_logic_random_seed_crc());
        for name in UNITS {
            let id = tid(sim, name);
            let _ = write!(s, "{}", sim.production_can_make_unit(o, id, None) as i32);
        }
        let module = sim.production_module(o).expect("production");
        let _ = write!(
            s,
            "{},{}",
            module.can_queue_upgrade() as i32,
            module.production_count()
        );
        for name in UNITS {
            let n = module.count_unit_type_in_queue(tid(sim, name));
            let _ = write!(s, ",{n}");
        }
        let score = sim
            .world
            .score_keepers
            .get(&TEAM.0)
            .map_or((0, 0, 0), |k| {
                (
                    k.total_money_spent,
                    k.total_units_built,
                    k.total_buildings_built,
                )
            });
        let _ = write!(s, " {},{},{} ", score.0, score.1, score.2);
        let a = sim
            .world
            .player_academy
            .get(&TEAM.0)
            .cloned()
            .unwrap_or_default();
        for v in [
            a.supply_centers_built,
            u32::from(a.researched_radar),
            a.peons_built,
            a.idle_building_units_max_frames,
            a.last_unit_built_frame,
            a.upgrades_purchased,
            a.gatherers_built,
            a.heroes_built,
            a.last_income_frame,
            a.max_frames_between_income,
        ] {
            let _ = write!(s, "{v},");
        }
        let factory_id = sim.world.objects.get(o).expect("factory").id;
        let done: Vec<_> = sim
            .world
            .completed_upgrades
            .iter()
            .filter(|(player, _, _)| *player == 1)
            .collect();
        let _ = write!(s, " {}", done.len());
        for (_, name, source) in done {
            let _ = write!(s, ",{name}:{}", u8::from(*source == factory_id));
        }
        let ms = production_slot(sim, o);
        let mut xc = XferCrc::new();
        sim.world.modules.payload(ms).expect("payload").crc(&mut xc);
        let _ = write!(s, " {} {}", xc.get_crc(), hex(&save(sim, o)));
        s
    }

    /// The harness's `apply`.
    fn apply(&mut self, sim: &mut SimContext, code: u32, a: u32, b: u32) -> i32 {
        let o = self.factory;
        let (x, y) = (a as usize, b);
        match code {
            0 => {
                let tmpl = tid(sim, UNITS[x]);
                return sim
                    .with_production(o, |p, s, c| {
                        let id = p.request_unique_unit_id();
                        p.queue_create_unit(s, c, tmpl, id)
                    })
                    .expect("queue")
                    .into();
            }
            1 | 3 => {
                let key = upgrade_key(sim, x);
                return sim
                    .with_production(o, |p, s, c| {
                        if code == 1 {
                            p.queue_upgrade(s, c, key)
                        } else {
                            p.cancel_upgrade(s, c, key)
                        }
                    })
                    .expect("upgrade")
                    .into();
            }
            2 => {
                return sim
                    .with_production(o, |p, s, c| p.cancel_unit_create(s, c, a as i32))
                    .expect("cancel")
                    .into();
            }
            4 => {
                let tmpl = tid(sim, UNITS[x]);
                sim.with_production(o, |p, s, c| p.cancel_all_units_of_type(s, c, tmpl));
            }
            5 => {
                sim.with_production(o, |p, s, c| p.cancel_and_refund_all_production(s, c));
            }
            6 => sim.dispatch_on_die(o, &DamageInfo::default()),
            7 => {
                let ms = production_slot(sim, o);
                for _ in 0..a {
                    sim.world.frame = sim.world.frame.wrapping_add(1);
                    let sleep = sim
                        .dispatch_module(ms, |m, s, c| {
                            m.as_update_mut().expect("update").update(s, c)
                        })
                        .expect("dispatch");
                    assert_eq!(sleep.0, 1);
                }
            }
            8 => {
                // `setHoldDoorOpen` ignores doors outside 0..4.
                if let Some(door) = ExitDoorType::from_i32(a as i32 - 2) {
                    sim.production_set_hold_door_open(o, door, Bool::from(b != 0));
                }
            }
            9 => script(|s| {
                s.door = a as i32 - 2;
                s.budget = b as i32 - 1;
            }),
            10 => script(|s| {
                s.hold = if a == 0 { -9 } else { a as i32 - 3 };
                s.hold_flag = b as i32;
            }),
            11 => {
                sim.world.player_money.insert(TEAM, a);
            }
            12 => {
                let (prod, cons) = ENERGY[x];
                let until = if y == 0 { 0 } else { sim.world.frame + y };
                let energy = Energy::from_parts(prod, cons, until, Some(1));
                sim.world.player_energy.insert(TEAM, energy);
            }
            13 => {
                if a != 0 {
                    sim.set_status(o, ObjectStatusType::Sold);
                } else {
                    sim.clear_status(o, ObjectStatusType::Sold);
                }
            }
            14 => {
                let w = &mut sim.world;
                for (set, allowed) in [
                    (&mut w.teams_cannot_build_units, a != 0),
                    (&mut w.teams_cannot_build_base, b != 0),
                ] {
                    if allowed {
                        set.remove(&TEAM);
                    } else {
                        set.insert(TEAM);
                    }
                }
            }
            15 => {
                let flag = if a != 0 {
                    OBJECT_STATUS_SCRIPT_UNPOWERED
                } else {
                    OBJECT_STATUS_SCRIPT_DISABLED
                };
                let obj = sim.object_mut(o).expect("factory");
                if b != 0 {
                    obj.script_status |= flag;
                } else {
                    obj.script_status &= !flag;
                }
            }
            16 => {
                const EXTRA: [&str; 6] = ["P_Pre", "U4", "UL", "U0", "U2", "U8"];
                let name = if a == 6 {
                    format!("F{b}")
                } else {
                    EXTRA[x].to_owned()
                };
                let extra = spawn_on(sim, &name, TEAM);
                if a != 6 && b != 0 {
                    sim.set_status(extra, ObjectStatusType::UnderConstruction);
                }
                self.extras.push(extra);
                clear_log();
            }
            17 => {
                if !self.extras.is_empty() {
                    let extra = self.extras.remove(x % self.extras.len());
                    sim.destroy_object(extra);
                    sim.process_destroy_list();
                    clear_log();
                }
            }
            18 => {
                let other = self
                    .extras
                    .iter()
                    .copied()
                    .find(|&e| sim.production_module(e).is_some());
                if let Some(other) = other {
                    let tmpl = tid(sim, "U4");
                    let queued = sim
                        .with_production(other, |p, s, c| {
                            let id = p.request_unique_unit_id();
                            p.queue_create_unit(s, c, tmpl, id)
                        })
                        .expect("queue");
                    clear_log();
                    return queued.into();
                }
            }
            19 => {
                let before = save(sim, o);
                let fresh = spawn_on(sim, &format!("F{}", self.variant), OTHER_TEAM);
                assert!(load(sim, fresh, &before), "load");
                let after = save(sim, fresh);
                sim.destroy_object(fresh);
                sim.process_destroy_list();
                clear_log();
                // The header holds the live schedule, which a load alone does not apply.
                return i32::from(before[20..] == after[20..]);
            }
            20 => {
                let science = sim
                    .rules
                    .science_store
                    .get_science_from_internal_name("SCIENCE_P354", &mut sim.rules.name_keys);
                let known = sim.world.player_sciences.entry(TEAM).or_default();
                known.clear();
                if a != 0 {
                    known.insert(science);
                }
            }
            21 => script(|s| {
                s.park_reserve = a != 0;
                s.park_space = b != 0;
            }),
            22 => sim.grant_player_upgrade(TEAM, UPGRADES[x]),
            23 => {
                let mask = upgrade_mask(sim, x);
                let obj = sim.object_mut(o).expect("factory");
                obj.upgrades_completed
                    .clear_and_set(&UpgradeMaskType::new(), &mask);
                sim.update_upgrade_modules(o);
            }
            24 => {
                let name = if a != 0 { "P354Alt" } else { "" };
                name.clone_into(&mut sim.object_mut(o).expect("factory").command_set_override);
            }
            25 => {
                let status = [
                    BuildableStatus::Yes,
                    BuildableStatus::IgnorePrerequisites,
                    BuildableStatus::No,
                    BuildableStatus::OnlyByAi,
                ][y as usize];
                sim.world
                    .buildable_overrides
                    .insert(UNITS[x].to_owned(), status);
            }
            26 => {
                if !self.extras.is_empty() {
                    let extra = self.extras[x % self.extras.len()];
                    let obj = sim.object_mut(extra).expect("extra");
                    obj.set_effectively_dead(Bool::from(b != 0));
                }
            }
            _ => panic!("op {code}"),
        }
        0
    }
}

const PENALTY: [f32; 4] = [0.0, 0.4, 1.0, 3.0];
const MIN_SPEED: [f32; 3] = [0.0, 0.2, 0.5];
const MAX_SPEED: [f32; 3] = [0.0, 0.8, 0.95];
const MULTIPLE: [f32; 3] = [0.0, 0.85, 1.3];
const HANDICAP: [f32; 3] = [1.0, 0.5, 1.5];
const CHANGE: [f32; 3] = [0.0, -0.25, 0.1];
const ENERGY: [(i32, i32); 6] = [(0, 0), (5, 0), (10, 10), (5, 10), (1, 100), (20, 10)];

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

fn numbers(text: &str) -> Vec<u32> {
    text.split(['[', ']', ','])
        .filter(|t| !t.is_empty())
        .map(|t| t.parse().expect("number"))
        .collect()
}

/// Sets up the player and the factory of one vector.
fn start(sim: &mut SimContext, variant: usize, frame: u32, cfg: &[u32]) -> Run {
    let c: Vec<usize> = cfg.iter().map(|&v| v as usize).collect();
    sim.world = world();
    sim.world.frame = frame;
    sim.world.rng.init_random_with_seed(cfg[0]);
    script(|s| {
        *s = Script {
            budget: -1,
            hold: -9,
            park_space: true,
            ..Script::default()
        };
    });
    let global = &mut sim.rules.global_data;
    global.low_energy_penalty_modifier = PENALTY[c[1]];
    global.min_low_energy_production_speed = MIN_SPEED[c[2]];
    global.max_low_energy_production_speed = MAX_SPEED[c[3]];
    global.multiple_factory = MULTIPLE[c[4]];
    let (handicap, cost, time, kind) = (c[5], c[6], c[7], c[8]);
    if handicap != 0 {
        let mut dict = Dict::new();
        for (name, value) in [
            ("HANDICAP_BUILDCOST_GENERIC", HANDICAP[handicap]),
            ("HANDICAP_BUILDTIME_BUILDINGS", HANDICAP[handicap]),
            ("HANDICAP_BUILDTIME_GENERIC", HANDICAP[3 - handicap]),
        ] {
            dict.set_real(sim.rules.name_keys.name_to_key(name), value);
        }
        let mut h = Handicap::new();
        h.read_from_dict(&dict, &mut sim.rules.name_keys);
        sim.world.player_handicaps.insert(TEAM, h);
    }
    let key = (TEAM, "U0".to_owned());
    if cost != 0 {
        sim.world
            .player_cost_change_percent
            .insert(key.clone(), CHANGE[cost]);
    }
    if time != 0 {
        sim.world
            .player_time_change_percent
            .insert(key, CHANGE[time]);
    }
    if kind != 0 {
        let vehicles = KindOfMaskType::from_single_index(KindOfType::Vehicle as i32);
        sim.add_cost_modifier(TEAM, &vehicles, CHANGE[kind]);
    }
    sim.world
        .player_is_human
        .insert(TEAM, if cfg[9] != 0 { TRUE } else { FALSE });
    for (name, status) in [
        ("U7", BuildableStatus::OnlyByAi),
        ("U8", BuildableStatus::IgnorePrerequisites),
        ("U9", BuildableStatus::No),
    ] {
        sim.world
            .buildable_overrides
            .insert(name.to_owned(), status);
    }
    sim.world.player_money.insert(TEAM, cfg[10]);
    // A player without any power entry stands for zero production and consumption.
    let (prod, cons) = ENERGY[c[11]];
    if (prod, cons) != (0, 0) {
        let energy = Energy::from_parts(prod, cons, 0, Some(1));
        sim.world.player_energy.insert(TEAM, energy);
    }
    let factory = spawn_on(sim, &format!("F{variant}"), TEAM);
    clear_log();
    Run {
        factory,
        extras: Vec::new(),
        variant,
    }
}

#[test]
#[cfg(not(any(feature = "retail-crc", feature = "retail-xfer-save")))]
fn production_update_matches_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT354_ProductionUpdate.json");
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"v\""))
        .collect();
    assert_eq!(lines.len(), 1000);
    let mut sim = SimContext::new();
    sim.rules = rules();
    let mut checked = 0;
    for (index, line) in lines.iter().enumerate() {
        let variant = get_u32(line, "variant").expect("variant") as usize;
        let frame = get_u32(line, "frame").expect("frame");
        let cfg = numbers(between(line, "\"cfg\":", ",\"disabled\""));
        let mut run = start(&mut sim, variant, frame, &cfg);
        let disabled = with_module(&mut sim, run.factory, |p, _, _| {
            p.data.disabled_types_to_process.to_unsigned_int()
        });
        assert_eq!(
            disabled,
            get_u32(line, "disabled").expect("disabled"),
            "vector {index}"
        );
        let got = run.row(&mut sim, 0);
        let want = get_str(line, "spawn").expect("spawn");
        assert_eq!(got, want, "vector {index} variant {variant} spawn");

        let ops = numbers(between(line, "\"ops\":", ",\"rows\""));
        let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
            .split("\",\"")
            .collect();
        assert_eq!(ops.len(), rows.len() * 3, "vector {index}");
        for (k, (op, want)) in ops.chunks(3).zip(&rows).enumerate() {
            let ret = run.apply(&mut sim, op[0], op[1], op[2]);
            let got = run.row(&mut sim, ret);
            assert_eq!(
                &got, want,
                "vector {index} variant {variant} op {k} {op:?} cfg {cfg:?}"
            );
            checked += 1;
        }
    }
    assert!(checked > 8000, "only {checked} rows");
}

// ---- without a reference ----------------------------------------------------------------

fn scene(variant: usize) -> (SimContext, ObjectSlot) {
    let mut sim = SimContext::new();
    sim.rules = rules();
    // seed, tuning (none), no handicap or changes, computer player, money, no power
    let cfg = [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 10_000, 5];
    let run = start(&mut sim, variant, 100, &cfg);
    (sim, run.factory)
}

fn queue(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> Bool {
    let tmpl = tid(sim, name);
    sim.with_production(owner, |p, s, c| {
        let id = p.request_unique_unit_id();
        p.queue_create_unit(s, c, tmpl, id)
    })
    .expect("production")
}

#[test]
fn special_power_template_skips_the_build_checks() {
    let (mut sim, factory) = scene(6);
    let tmpl = tid(&sim, "UX");
    assert_eq!(queue(&mut sim, factory, "UX"), FALSE);
    sim.with_production(factory, |p, _, _| {
        p.set_special_power_construction_template(Some(tmpl));
    });
    // `canMakeUnit` now answers yes, and the queue limit of zero still holds.
    assert_eq!(sim.production_can_make_unit(factory, tmpl, None) as i32, 0);
    assert_eq!(queue(&mut sim, factory, "UX"), FALSE);
    let (mut sim, factory) = scene(0);
    sim.with_production(factory, |p, _, _| {
        p.set_special_power_construction_template(Some(tmpl));
    });
    assert_eq!(queue(&mut sim, factory, "UX"), TRUE);
}

#[test]
fn defecting_refunds_the_queue_first() {
    let (mut sim, factory) = scene(0);
    assert_eq!(queue(&mut sim, factory, "U0"), TRUE);
    assert_eq!(sim.player_money(TEAM), 9_500);
    sim.defect_object(factory, OTHER_TEAM, 0, FALSE);
    assert_eq!(sim.player_money(TEAM), 10_000);
    assert_eq!(sim.player_money(OTHER_TEAM), 0);
    assert_eq!(
        sim.production_module(factory)
            .expect("p")
            .production_count(),
        0
    );
}

#[test]
fn death_of_an_executing_module_still_refunds() {
    let (mut sim, factory) = scene(0);
    assert_eq!(queue(&mut sim, factory, "U0"), TRUE);
    with_module(&mut sim, factory, |_, s, c| {
        s.dispatch_on_die(c.owner_slot, &DamageInfo::default());
    });
    assert_eq!(sim.player_money(TEAM), 10_000);
}

#[test]
fn hold_door_from_inside_the_update_is_applied_after_it() {
    let (mut sim, factory) = scene(1);
    with_module(&mut sim, factory, |p, s, c| {
        s.production_set_hold_door_open(c.owner_slot, ExitDoorType::Door2, TRUE);
        assert_eq!(p.doors[1].hold_open, FALSE);
    });
    with_module(&mut sim, factory, |p, _, _| {
        assert_eq!(p.doors[1].hold_open, TRUE);
        assert_eq!(p.doors[1].opened_frame, 100);
    });
}

#[test]
fn saved_queue_loads_back_and_rejects_bad_data() {
    let (mut sim, factory) = scene(0);
    assert_eq!(queue(&mut sim, factory, "U0"), TRUE);
    let key = upgrade_key(&sim, 0);
    sim.with_production(factory, |p, s, c| p.queue_upgrade(s, c, key));
    let bytes = save(&mut sim, factory);

    let other = spawn_on(&mut sim, "F0", TEAM);
    assert!(load(&mut sim, other, &bytes));
    let (queue_a, queue_b) = (
        with_module(&mut sim, factory, |p, _, _| p.queue.clone()),
        with_module(&mut sim, other, |p, _, _| p.queue.clone()),
    );
    assert_eq!(queue_a, queue_b);
    assert_eq!(queue_b[0].production_type, PRODUCTION_UNIT);
    assert_eq!(queue_b[1].production_type, PRODUCTION_UPGRADE);
    // A queue that is not empty cannot be loaded into.
    assert!(!load(&mut sim, other, &bytes));

    for name in ["U0", "UpgP1"] {
        let at = bytes
            .windows(name.len())
            .position(|w| w == name.as_bytes())
            .expect("name");
        let mut bad = bytes.clone();
        bad[at] = b'Z';
        let fresh = spawn_on(&mut sim, "F0", TEAM);
        assert!(!load(&mut sim, fresh, &bad), "{name}");
    }
}

#[test]
fn default_data_and_parsed_modifiers() {
    let (mut sim, factory) = scene(3);
    let data = with_module(&mut sim, factory, |p, _, _| p.data.clone());
    let quantities: Vec<_> = data
        .quantity_modifiers
        .iter()
        .map(|m| (m.template_name.as_str(), m.quantity))
        .collect();
    assert_eq!(quantities, [("U1", 3), ("U1", 5), ("U0", 2)]);
    assert_eq!(data.door_closing_time, 12);
    let key = sim.rules.name_keys.name_to_key("ProductionUpdate");
    let fresh = ProductionUpdateModuleData::new(key, key);
    assert_eq!(fresh.max_queue_entries, 9);
    assert_eq!(fresh.disabled_types_to_process.to_unsigned_int(), 8);
    assert_eq!(
        (
            fresh.num_door_animations,
            fresh.door_opening_time,
            fresh.door_wait_open_time,
            fresh.door_closing_time,
            fresh.construction_complete_duration,
        ),
        (0, 0, 0, 0, 0)
    );
    assert!(fresh.quantity_modifiers.is_empty());
}

#[test]
fn an_upgrade_granted_to_the_object_clears_the_in_production_mark() {
    let (mut sim, factory) = scene(0);
    let key = upgrade_key(&sim, 0);
    let mask = upgrade_mask(&sim, 0);
    sim.with_production(factory, |p, s, c| p.queue_upgrade(s, c, key));
    assert_eq!(sim.player_has_upgrade_in_production(TEAM, &mask), TRUE);
    sim.grant_upgrade(factory, "UpgP1", FALSE);
    assert_eq!(sim.player_has_upgrade_in_production(TEAM, &mask), FALSE);
    assert_eq!(sim.player_has_upgrade(TEAM, &mask), TRUE);
    // The queued research can no longer be cancelled; refunding everything drops it.
    let cancelled = sim.with_production(factory, |p, s, c| p.cancel_upgrade(s, c, key));
    assert_eq!(cancelled, Some(FALSE));
    sim.with_production(factory, |p, s, c| p.cancel_and_refund_all_production(s, c));
    assert_eq!(sim.player_money(TEAM), 9_700);
    assert_eq!(
        sim.production_module(factory)
            .expect("p")
            .production_count(),
        0
    );
}

#[test]
fn a_cancel_by_id_zero_skips_an_upgrade_entry() {
    let (mut sim, factory) = scene(0);
    let key = upgrade_key(&sim, 0);
    sim.with_production(factory, |p, s, c| p.queue_upgrade(s, c, key));
    // The reference reads the upgrade pointer as a thing template here.
    let cancelled = sim.with_production(factory, |p, s, c| p.cancel_unit_create(s, c, 0));
    assert_eq!(cancelled, Some(FALSE));
    assert_eq!(
        sim.production_module(factory)
            .expect("p")
            .production_count(),
        1
    );
}

#[test]
fn one_refund_pass_stops_after_a_hundred_entries() {
    let (mut sim, factory) = scene(5);
    for _ in 0..101 {
        assert_eq!(queue(&mut sim, factory, "U3"), TRUE);
    }
    sim.with_production(factory, |p, s, c| p.cancel_and_refund_all_production(s, c));
    assert_eq!(
        sim.production_module(factory)
            .expect("p")
            .production_count(),
        1
    );
}

#[test]
fn a_new_unit_is_noted_for_the_script_engine() {
    let (mut sim, factory) = scene(0);
    assert_eq!(queue(&mut sim, factory, "U1"), TRUE);
    assert_eq!(sim.world.object_count_changed_frame, 0);
    sim.world.frame = 101;
    with_module(&mut sim, factory, |p, s, c| {
        p.update(s, c);
    });
    assert_eq!(sim.world.object_count_changed_frame, 101);
    assert_eq!(sim.world.units_produced.len(), 1);
}

#[test]
fn only_hold_door_and_die_ops_are_taken() {
    let (mut sim, factory) = scene(0);
    assert_eq!(queue(&mut sim, factory, "U0"), TRUE);
    with_module(&mut sim, factory, |p, s, c| {
        p.apply_deferred_op(DeferredSelfOp::SetIndestructible(TRUE), s, c);
        assert_eq!(p.queue.len(), 1);
    });
}
