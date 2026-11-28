//! `PKT-368` `SpecialAbilityUpdate` and `PKT-314` `CommandButtonHuntUpdate` against
//! vectors from the reference modules (`tests/cpp_harness/gen_pkt368_314_ability.cpp`).

use std::any::Any;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::sync::{Arc, Mutex};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::modules::module::{AiUpdateFacet, StickyBombFacet};
use zh_logic::modules::module::{SpecialPowerCaller, SpecialPowerFacet};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::command_button_hunt_update::CommandButtonHuntUpdate;
use zh_logic::modules::update::special_ability_update::{AbilityFeedback, SpecialAbilityUpdate};
use zh_logic::modules::{BehaviorModule, CommandSourceType, ModuleData, ModuleType};
use zh_logic::object::thing::{make_align_to_normal_matrix, ThingTerrainAccess};
use zh_logic::object::weapon::{Weapon, WeaponTemplate};
use zh_logic::object::weapon_set::{WeaponLockType, WeaponSlotType};
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::rules::command_button::{CommandButton, CommandStore, GuiCommandType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceTextLookup;
use zh_logic::scheduler::{SleepyUpdatePhase, UPDATE_SLEEP_FOREVER};
use zh_logic::sim::{
    ActionQuery, ClientSink, RulesRegistry, SimApi, SimContext, SimView, SimWorld,
    PRIVATE_STATUS_EFFECTIVELY_DEAD,
};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT368_314_Ability.json");

/// Neutral, A, B by player index.
const TEAMS: [TeamId; 3] = [TeamId(3), TeamId(1), TeamId(2)];

const OFFSETS: [[f32; 2]; 16] = [
    [0.0, 0.0],
    [4.0, 0.0],
    [6.0, 1.0],
    [10.0, 0.0],
    [0.0, 12.0],
    [-18.0, 0.0],
    [25.0, 20.0],
    [-30.0, -9.0],
    [45.0, 0.0],
    [0.0, -70.0],
    [-100.0, 5.0],
    [130.0, 40.0],
    [2.0, 2.0],
    [-7.0, 0.0],
    [200.0, 10.0],
    [37.0, 0.0],
];

// ---- a small JSON reader ----------------------------------------------------

#[derive(Debug)]
enum Json {
    Num(i64),
    Str(String),
    Arr(Vec<Json>),
    Obj(BTreeMap<String, Json>),
}

impl Json {
    fn parse(text: &[u8], at: &mut usize) -> Json {
        let skip = |at: &mut usize| {
            while matches!(text[*at], b' ' | b'\n' | b',' | b':') {
                *at += 1;
            }
        };
        skip(at);
        match text[*at] {
            b'{' => {
                *at += 1;
                let mut map = BTreeMap::new();
                loop {
                    skip(at);
                    if text[*at] == b'}' {
                        *at += 1;
                        return Json::Obj(map);
                    }
                    let key = Json::parse(text, at).str().to_owned();
                    map.insert(key, Json::parse(text, at));
                }
            }
            b'[' => {
                *at += 1;
                let mut list = Vec::new();
                loop {
                    skip(at);
                    if text[*at] == b']' {
                        *at += 1;
                        return Json::Arr(list);
                    }
                    list.push(Json::parse(text, at));
                }
            }
            b'"' => {
                let start = *at + 1;
                let mut end = start;
                while text[end] != b'"' {
                    end += 1;
                }
                *at = end + 1;
                Json::Str(String::from_utf8_lossy(&text[start..end]).into_owned())
            }
            _ => {
                let start = *at;
                while text[*at] == b'-' || text[*at].is_ascii_digit() {
                    *at += 1;
                }
                Json::Num(
                    std::str::from_utf8(&text[start..*at])
                        .unwrap()
                        .parse()
                        .unwrap(),
                )
            }
        }
    }
    fn get(&self, key: &str) -> &Json {
        match self {
            Json::Obj(m) => &m[key],
            other => panic!("not an object: {other:?}"),
        }
    }
    fn arr(&self) -> &[Json] {
        match self {
            Json::Arr(a) => a,
            other => panic!("not an array: {other:?}"),
        }
    }
    fn str(&self) -> &str {
        match self {
            Json::Str(s) => s,
            other => panic!("not a string: {other:?}"),
        }
    }
    fn num(&self) -> i64 {
        match self {
            Json::Num(n) => *n,
            other => panic!("not a number: {other:?}"),
        }
    }
    fn at(&self, i: usize) -> i64 {
        self.arr()[i].num()
    }
}

// ---- test doubles -------------------------------------------------------------

#[derive(Default)]
struct Log {
    ai: Vec<String>,
    audio: Vec<String>,
    text: Vec<String>,
    eva: [u8; 3],
    particles: Vec<String>,
}

type Shared = Arc<Mutex<Log>>; // determinism-ok: test recorder

fn bits(f: Real) -> String {
    format!("{:08x}", f.to_bits())
}

fn pos_bits(p: &Coord3D) -> String {
    format!("{}{}{}", bits(p.x), bits(p.y), bits(p.z))
}

/// The harness's scripted AI.
struct ScriptedAi {
    name_key: NameKeyType,
    log: Shared,
    idle: bool,
    moving: bool,
    ignored: ObjectId,
    /// Default priority and per template priorities, when attack info is set.
    priorities: Option<(Int, BTreeMap<String, Int>)>,
}

/// The one object whose orders are logged.
const USER_ID: ObjectId = ObjectId(1);

impl ScriptedAi {
    fn order(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        mut text: String,
        src: CommandSourceType,
    ) {
        if ctx.owner_id == USER_ID {
            if let Some((name, rest)) = text.split_once('|') {
                text = format!("{name}:{}{rest}", src as i32);
            }
            self.log.lock().unwrap().ai.push(text);
        }
        if let Some(o) = sim.object_mut(ctx.owner_slot) {
            o.ai_last_command_source = src;
        }
    }
}

fn id_of(sim: &dyn SimView, slot: ObjectSlot) -> i32 {
    sim.object(slot).map_or(0, |o| o.id.0)
}

impl AiUpdateFacet for ScriptedAi {
    fn is_idle(&self) -> Bool {
        Bool::from(self.idle)
    }
    fn is_moving(&self) -> Bool {
        Bool::from(!self.idle && self.moving)
    }
    fn get_ignored_obstacle_id(&self) -> ObjectId {
        self.ignored
    }
    fn ignore_obstacle(&mut self, obstacle: ObjectId) {
        self.ignored = obstacle;
    }
    fn ai_idle(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, src: CommandSourceType) {
        self.order(sim, ctx, "idle|".into(), src);
    }
    fn ai_hunt(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, src: CommandSourceType) {
        self.order(sim, ctx, "hunt|".into(), src);
    }
    fn ai_busy(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, src: CommandSourceType) {
        self.order(sim, ctx, "busy|".into(), src);
    }
    fn ai_enter(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        t: ObjectSlot,
        src: CommandSourceType,
    ) {
        let text = format!("enter|:{}", id_of(sim, t));
        self.order(sim, ctx, text, src);
    }
    fn ai_move_to_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        t: ObjectSlot,
        src: CommandSourceType,
    ) {
        let text = format!("moveObj|:{}", id_of(sim, t));
        self.order(sim, ctx, text, src);
    }
    fn ai_move_to_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        p: &Coord3D,
        src: CommandSourceType,
    ) {
        self.order(sim, ctx, format!("movePos|:{}", pos_bits(p)), src);
    }
    fn ai_attack_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        t: ObjectSlot,
        max_shots: Int,
        src: CommandSourceType,
    ) {
        let text = format!("attack|:{}:{}", id_of(sim, t), max_shots as u32);
        self.order(sim, ctx, text, src);
    }
    fn ai_face_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        t: ObjectSlot,
        src: CommandSourceType,
    ) {
        let text = format!("faceObj|:{}", id_of(sim, t));
        self.order(sim, ctx, text, src);
    }
    fn ai_face_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        p: &Coord3D,
        src: CommandSourceType,
    ) {
        self.order(sim, ctx, format!("facePos|:{}", pos_bits(p)), src);
    }
    fn attack_priority(&self, sim: &dyn SimView, victim: ObjectSlot) -> Option<Int> {
        let (default, map) = self.priorities.as_ref()?;
        let name = sim.object(victim).and_then(|o| {
            o.thing
                .template_override()
                .get_final(&sim.rules().thing_factory)
                .map(|t| t.get_name().to_owned())
        })?;
        Some(map.get(&name).copied().unwrap_or(*default))
    }
}

impl BehaviorModule for ScriptedAi {
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
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
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

thread_local! {
    static AI_LOG: Shared = Shared::default();
}

/// What `StickyBombUpdate` does that the vectors see.
struct StickyDouble {
    name_key: NameKeyType,
    target: ObjectId,
}

fn is_booby_trap(sim: &dyn SimView, slot: ObjectSlot) -> bool {
    sim.object_is_kind_of(slot, KindOfType::BoobyTrap) != FALSE
}

fn template_name(sim: &dyn SimView, slot: ObjectSlot) -> String {
    sim.object(slot)
        .and_then(|o| {
            o.thing
                .template_override()
                .get_final(&sim.rules().thing_factory)
                .map(|t| t.get_name().to_owned())
        })
        .unwrap_or_default()
}

fn set_dead(sim: &mut dyn SimApi, slot: ObjectSlot, dead: bool) {
    if let Some(o) = sim.object_mut(slot) {
        if dead {
            o.private_status |= PRIVATE_STATUS_EFFECTIVELY_DEAD;
        } else {
            o.private_status &= !PRIVATE_STATUS_EFFECTIVELY_DEAD;
        }
    }
}

impl StickyDouble {
    fn init(
        &mut self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        target: ObjectSlot,
        bomber: Option<ObjectSlot>,
    ) {
        let Some((target_id, mut pos)) =
            sim.object(target).map(|o| (o.id, *o.thing.get_position()))
        else {
            return;
        };
        self.target = target_id;
        if let Some(o) = sim.object_mut(owner) {
            o.producer_id = target_id;
        }
        let bomber_pos = bomber
            .and_then(|b| sim.object(b))
            .map(|o| *o.thing.get_position());
        match bomber_pos {
            Some(p) if sim.object_is_kind_of(target, KindOfType::Immobile) != FALSE => {
                pos = Coord3D::new(p.x, p.y, 0.0);
            }
            _ => pos.z += 10.0,
        }
        sim.set_object_position(owner, &pos);
        if is_booby_trap(sim, owner) {
            sim.set_status(target, ObjectStatusType::BoobyTrapped);
        }
    }
}

impl StickyBombFacet for StickyDouble {
    fn init_sticky_bomb(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        bomber: ObjectSlot,
    ) {
        self.init(sim, ctx.owner_slot, target, Some(bomber));
    }
    fn detonate(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        // `kill()`, with the harness's die module in the way.
        set_dead(sim, owner, true);
        let victim = match template_name(sim, owner).as_str() {
            "AB_TrapKill" => sim.object(owner).map(|o| o.producer_id),
            "AB_TrapKillUser" => Some(USER_ID),
            _ => None,
        };
        if let Some(victim) = victim.and_then(|v| sim.find_object(v)) {
            set_dead(sim, victim, true);
        }
        sim.destroy_object(owner);
    }
}

impl BehaviorModule for StickyDouble {
    fn module_name_key(&self) -> NameKeyType {
        self.name_key
    }
    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if is_booby_trap(sim, ctx.owner_slot) {
            if let Some(target) = sim.find_object(self.target) {
                sim.clear_status(target, ObjectStatusType::BoobyTrapped);
            }
        }
    }
    fn xfer(&mut self, _x: &mut dyn Xfer, _s: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
    fn as_sticky_bomb_mut(&mut self) -> Option<&mut dyn StickyBombFacet> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

struct StubData(NameKeyType, NameKeyType, FacetMask);

impl ModuleData for StubData {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn tag_key(&self) -> NameKeyType {
        self.0
    }
    fn module_name_key(&self) -> NameKeyType {
        self.1
    }
    fn facet_mask(&self) -> FacetMask {
        self.2
    }
}

fn skip_block(ini: &mut IniReader<'_>) -> Result<(), IniError> {
    ini.read_block_fields(|_, reader| {
        while reader
            .next_token_or_null(zh_common::ini::SEPS_DEFAULT)
            .is_some()
        {}
        Ok(())
    })
}

fn parse_ai(
    ini: &mut IniReader<'_>,
    tag: NameKeyType,
    name: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    skip_block(ini)?;
    Ok(Box::new(StubData(tag, name, FacetMask::AI_UPDATE)))
}

fn parse_sticky(
    ini: &mut IniReader<'_>,
    tag: NameKeyType,
    name: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    skip_block(ini)?;
    Ok(Box::new(StubData(tag, name, FacetMask(0x001))))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(ScriptedAi {
        name_key: data.module_name_key(),
        log: AI_LOG.with(Arc::clone),
        idle: true,
        moving: false,
        ignored: ObjectId::INVALID,
        priorities: None,
    })
}

fn create_sticky(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(StickyDouble {
        name_key: data.module_name_key(),
        target: ObjectId::INVALID,
    })
}

/// Where each object was, for the floating texts.
type Positions = Arc<Mutex<BTreeMap<ObjectId, Coord3D>>>; // determinism-ok: test recorder

/// Records what the harness's audio, UI, EVA and particle doubles record.
struct Client(Shared, Positions);

impl ClientSink for Client {
    fn on_special_ability_feedback(&mut self, source: ObjectId, event: &AbilityFeedback<'_>) {
        let mut log = self.0.lock().unwrap();
        match event {
            AbilityFeedback::Sound(name) => log.audio.push((*name).to_owned()),
            AbilityFeedback::CompletionVoice(key) => log.audio.push(
                match *key {
                    "VoiceCaptureBuildingComplete" => "AS_VoiceCapture",
                    "VoiceDisableVehicleComplete" => "AS_VoiceDisable",
                    "VoiceStealCashComplete" => "AS_VoiceCash",
                    _ => "AS_VoiceTask",
                }
                .to_owned(),
            ),
            AbilityFeedback::CaptureFlash(_) => log.audio.push("AS_Tick".to_owned()),
            AbilityFeedback::EvaBuildingBeingStolen => log.eva[0] = 1,
            AbilityFeedback::EvaBuildingStolen => log.eva[1] = 1,
            AbilityFeedback::EvaCashStolen => log.eva[2] = 1,
            AbilityFeedback::CashStolen { target, .. } => {
                let positions = self.1.lock().unwrap();
                let mut at = |id: ObjectId, rise: Real, color: u32| {
                    let mut p = positions[&id];
                    p.z += rise;
                    log.text.push(format!("{}:{color}", pos_bits(&p)));
                };
                at(source, 20.0, 0xFF00_FF00);
                at(*target, 30.0, 0xFFFF_0000);
            }
            AbilityFeedback::DisableEffect {
                offset, lifetime, ..
            } => {
                log.particles
                    .push(format!("{}:{lifetime}", pos_bits(offset)));
            }
            _ => {}
        }
    }
}

/// Flat ground with the harness's wall at x = 500.
struct Wall(Real);

impl ThingTerrainAccess for Wall {
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
        _stick: Bool,
        mtx: &mut zh_common::math::Matrix3D,
    ) -> PathfindLayerEnum {
        make_align_to_normal_matrix(angle, pos, &Coord3D::new(0.0, 0.0, 1.0), mtx);
        PathfindLayerEnum::Ground
    }
    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        water: Option<&mut Real>,
        ground: Option<&mut Real>,
    ) -> Bool {
        if let Some(w) = water {
            *w = 0.0;
        }
        if let Some(g) = ground {
            *g = 0.0;
        }
        FALSE
    }
    fn is_clear_line_of_sight(&self, a: &Coord3D, b: &Coord3D) -> Bool {
        Bool::from(
            !(self.0 > 0.0 && (a.x - 500.0) * (b.x - 500.0) < 0.0 && (a.z + b.z) * 0.5 < self.0),
        )
    }
}

struct NoText;

impl ScienceTextLookup for NoText {
    fn fetch_label(&mut self, label: &str) -> String {
        label.to_owned()
    }
}

// ---- fixtures -----------------------------------------------------------------

fn command_type(name: &str) -> GuiCommandType {
    match name {
        "SPECIAL_POWER" => GuiCommandType::SpecialPower,
        "FIRE_WEAPON" => GuiCommandType::FireWeapon,
        "SWITCH_WEAPON" => GuiCommandType::SwitchWeapon,
        "HIJACK_VEHICLE" => GuiCommandType::HijackVehicle,
        "CONVERT_TO_CARBOMB" => GuiCommandType::ConvertToCarBomb,
        "SABOTAGE_BUILDING" => GuiCommandType::SabotageBuilding,
        "STOP" => GuiCommandType::Stop,
        other => panic!("command {other}"),
    }
}

fn build_rules(root: &Json) -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    for (name, mask, ai, parse, create) in [
        (
            "AIUpdateInterface",
            FacetMask::AI_UPDATE,
            true,
            parse_ai as _,
            create_ai as _,
        ),
        (
            "StickyBombUpdate",
            FacetMask(0x001),
            false,
            parse_sticky
                as fn(
                    &mut IniReader<'_>,
                    NameKeyType,
                    NameKeyType,
                    &mut ModuleParseContext<'_>,
                ) -> Result<Box<dyn ModuleData>, IniError>,
            create_sticky as fn(&dyn ModuleData, ModuleContext) -> Box<dyn BehaviorModule>,
        ),
    ] {
        rules.module_registry.register_module(
            &mut rules.name_keys,
            name,
            ModuleType::Behavior,
            mask,
            SleepyUpdatePhase::Normal,
            ai,
            parse,
            create,
        );
    }
    let mut sciences = String::new();
    for name in ["SCIENCE_AB_HAVE", "SCIENCE_AB_LACK"] {
        let _ = write!(
            sciences,
            "Science {name}\n  PrerequisiteSciences = None\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n"
        );
    }
    let mut reader = IniReader::new("science.ini", sciences.as_bytes(), IniLoadType::Overwrite);
    rules
        .science_store
        .load_from_ini(&mut reader, &mut rules.name_keys, &mut NoText)
        .expect("sciences");
    let mut powers = String::new();
    for (i, p) in root.get("powers").arr().iter().enumerate() {
        let science = match p.get("science").num() {
            1 => "  RequiredScience = SCIENCE_AB_HAVE\n",
            2 => "  RequiredScience = SCIENCE_AB_LACK\n",
            _ => "",
        };
        let _ = write!(
            powers,
            "SpecialPower AB_SP{i}\n  Enum = {}\n  ReloadTime = {}\n{science}End\n",
            p.get("type").str(),
            // Back to milliseconds.
            p.get("reload").num() * 1000 / 30
        );
    }
    let mut reader = IniReader::new("powers.ini", powers.as_bytes(), IniLoadType::Overwrite);
    rules
        .special_power_store
        .load_from_ini(&mut reader, &rules.science_store, &mut rules.name_keys)
        .expect("special powers");
    for (i, p) in root.get("powers").arr().iter().enumerate() {
        let t = rules
            .special_power_store
            .find_special_power_template(&format!("AB_SP{i}"))
            .expect("power");
        assert_eq!(
            i64::from(t.reload_time()),
            p.get("reload").num(),
            "reload of power {i}"
        );
    }
    rules.particle_system_names.insert("AB_Fx".to_owned());

    let mut store = CommandStore::default();
    for b in root.get("buttons").arr() {
        let mut button =
            CommandButton::new(b.get("name").str(), command_type(b.get("command").str()));
        if b.get("power").num() >= 0 {
            let name = format!("AB_SP{}", b.get("power").num());
            button.special_power_id = rules
                .special_power_store
                .find_special_power_template(&name)
                .map(|t| t.id());
        }
        if b.get("context").num() != 0 {
            button.options = zh_logic::rules::command_button::CONTEXTMODE_COMMAND;
        }
        if b.get("slot").num() == 1 {
            button.weapon_slot = WeaponSlotType::Secondary;
        }
        store.add_button(button);
    }
    let users = root.get("users").arr();
    for (i, u) in users.iter().enumerate() {
        let set: Vec<(usize, usize)> = u
            .get("set")
            .arr()
            .iter()
            .map(|e| (e.at(0) as usize, e.at(1) as usize))
            .collect();
        store.add_set(&format!("AB_Set{i}"), &set);
    }
    rules.command_store = store;

    let mut objects = root.get("shared").str().replace('|', "\n");
    for u in users {
        objects.push_str(&u.get("ini").str().replace('|', "\n"));
    }
    let mut reader = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut reader) {
        let line = reader.get_line_num() as usize;
        panic!(
            "thing templates, line {line}: {e:?}: {:?}",
            objects.lines().nth(line.saturating_sub(1))
        );
    }
    rules
}

// ---- the scene ------------------------------------------------------------------

struct Scene<'a> {
    sim: &'a mut SimContext,
    log: Shared,
    positions: Positions,
    user: ObjectId,
    /// The object the hunter's experience feeds.
    sink: ObjectId,
    fixed: Vec<ObjectId>,
    highest_fixed: i32,
    abilities: Vec<ModuleSlot>,
    /// The `SpecialAbility` power modules.
    powers: Vec<ModuleSlot>,
    hunt: Option<ModuleSlot>,
    previous: [String; 6],
    rotate: usize,
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object(tid, team, pos, 0.0)
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

fn save(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    let mut s = String::new();
    for b in xs.as_bytes() {
        let _ = write!(s, "{b:02x}");
    }
    s
}

/// Saves a module and loads the bytes back into it.
fn load_back(sim: &mut SimContext, slot: ModuleSlot) {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    let mut xl = XferLoad::new(xs.as_bytes());
    payload.xfer(&mut xl, sim).expect("xfer load");
    sim.world.modules.restore_payload(slot, generation, payload);
}

/// Rust only: an ability's save loads into a fresh module and saves the same bytes.
fn round_trip(sim: &mut SimContext, slot: ModuleSlot) {
    let (mut payload, ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut first = XferSave::new();
    payload.xfer(&mut first, sim).expect("xfer save");
    let data = payload
        .as_any()
        .and_then(|a| a.downcast_ref::<SpecialAbilityUpdate>())
        .expect("ability")
        .data
        .clone();
    sim.world.modules.restore_payload(slot, generation, payload);
    let mut fresh = SpecialAbilityUpdate::new(data);
    fresh.schedule.bind(ctx);
    fresh
        .xfer(&mut XferLoad::new(first.as_bytes()), sim)
        .expect("xfer load");
    let mut second = XferSave::new();
    fresh.xfer(&mut second, sim).expect("second save");
    assert_eq!(first.as_bytes(), second.as_bytes());
}

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

fn step(sim: &mut SimContext, m: ModuleSlot, frame: u32) {
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

fn object_state(sim: &SimContext, slot: ObjectSlot) -> String {
    let o = sim.object(slot).expect("object");
    let status = (0..45)
        .filter(|&b| o.status.test(b) != FALSE)
        .fold(0u64, |acc, b| acc | (1 << b));
    format!(
        "{},{status},{}{}{},{},{}",
        sim.player_index(o.team_id),
        u8::from(o.is_effectively_dead() != FALSE),
        u8::from(o.is_destroyed()),
        u8::from(o.is_disabled_by_type(DisabledType::Hacked) != FALSE),
        // `getDisabledUntil`: zero unless the type is set.
        if o.is_disabled_by_type(DisabledType::Hacked) != FALSE {
            o.disabled_till_frame[DisabledType::Hacked.as_i32() as usize]
        } else {
            0
        },
        if o.contain_slot.is_some() {
            sim.contain_count(slot)
        } else {
            0
        },
    )
}

impl Scene<'_> {
    fn user_slot(&self) -> Option<ObjectSlot> {
        self.sim.find_object(self.user)
    }

    fn slot(&self, index: usize) -> Option<ObjectSlot> {
        self.sim.find_object(self.fixed[index])
    }

    /// Positions for the floating texts, which are placed when the cash moves.
    fn remember_positions(&mut self) {
        let mut map = self.positions.lock().unwrap();
        for id in std::iter::once(self.user).chain(self.fixed.iter().copied()) {
            if let Some(o) = self.sim.find_object(id).and_then(|s| self.sim.object(s)) {
                let p = *o.thing.get_position();
                map.insert(id, p);
            }
        }
    }

    fn ai<R>(&mut self, f: impl FnOnce(&mut ScriptedAi) -> R) -> Option<R> {
        let ms = self
            .user_slot()
            .and_then(|u| self.sim.object(u))
            .and_then(|o| o.ai_slot)?;
        let payload = self.sim.world.modules.payload_mut(ms)?;
        payload.as_any_mut()?.downcast_mut::<ScriptedAi>().map(f)
    }

    fn probe(&self) -> usize {
        self.rotate % self.sim.rules.command_store.button_count()
    }

    fn observe(&mut self, perms: &str, ret: u32) -> Vec<String> {
        let mut sec: [String; 6] = Default::default();
        let probe = self.probe();
        let user = self.user_slot();
        if let Some(u) = user {
            for m in self.abilities.clone() {
                let saved = save(self.sim, m);
                let next = next_call_frame(self.sim, m);
                let payload = self.sim.world.modules.payload(m).expect("payload");
                let ability = payload
                    .as_any()
                    .and_then(|a| a.downcast_ref::<SpecialAbilityUpdate>())
                    .expect("ability");
                let facet = payload.as_special_power_update().expect("facet");
                let button = self.sim.rules.command_store.button(probe).expect("button");
                let in_use = facet.is_power_currently_in_use(self.sim, None);
                let probed = if button.special_power_id.is_some() {
                    facet.is_power_currently_in_use(self.sim, Some(button))
                } else {
                    2
                };
                let mut found = String::new();
                for &id in &self.fixed {
                    if self.sim.find_object(id).is_none() {
                        continue;
                    }
                    if let Some(s) = facet.find_special_object_with_producer_id(self.sim, id) {
                        let _ = write!(found, "{}>{} ", id.0, id_of(self.sim, s));
                    }
                }
                let _ = write!(
                    sec[0],
                    "{saved},{next},{in_use},{probed},{},{},{found};",
                    facet.special_object_count(),
                    ability.data.max_special_objects
                );
            }
            for m in self.powers.clone() {
                let saved = save(self.sim, m);
                sec[0].push_str(&saved);
                sec[0].push('+');
            }
            if let Some(h) = self.hunt {
                let saved = save(self.sim, h);
                let held = self
                    .current_button()
                    .and_then(|b| self.sim.rules.command_store.button(b))
                    .map_or("-", |b| b.name.as_str());
                sec[1] = format!("{saved},{},{held}", next_call_frame(self.sim, h));
            }
            let physics = self.sim.object(u).and_then(|o| o.physics_slot);
            let stealth = self.sim.object(u).and_then(|o| o.stealth_slot);
            let physics = physics.map_or_else(|| "-".to_owned(), |m| save(self.sim, m));
            let stealth = stealth.map_or_else(|| "-".to_owned(), |m| save(self.sim, m));
            let state = object_state(self.sim, u);
            let ignored = self.ai(|a| a.ignored.0).unwrap_or(0);
            let sink_xp = self
                .sim
                .find_object(self.sink)
                .and_then(|s| self.sim.object(s))
                .map_or(0, |o| o.experience_tracker.current_experience);
            let o = self.sim.object(u).expect("user");
            let flag =
                |f: ModelConditionFlagType| u8::from(o.model_condition.test(f.as_i32()) != FALSE);
            let weapon_slot = o.weapon_set.cur_weapon;
            let has_weapon = o
                .weapon_set
                .get_weapon_in_weapon_slot(weapon_slot)
                .is_some();
            sec[2] = format!(
                "{},{},{}{}{}{},{state},{}{},{},{},{physics},{stealth},{ignored},{sink_xp}",
                pos_bits(o.thing.get_position()),
                bits(o.thing.get_orientation()),
                flag(ModelConditionFlagType::Unpacking),
                flag(ModelConditionFlagType::Packing),
                flag(ModelConditionFlagType::FiringA),
                flag(ModelConditionFlagType::RaisingFlag),
                if has_weapon { weapon_slot.as_i32() } else { 9 },
                u8::from(o.weapon_set.cur_weapon_locked_status != WeaponLockType::NotLocked),
                o.experience_tracker.current_experience,
                o.experience_tracker.current_level.as_i32(),
            );
        }
        for &id in &self.fixed {
            match self.sim.find_object(id) {
                Some(s) => sec[3].push_str(&object_state(self.sim, s)),
                None => sec[3].push('-'),
            }
            sec[3].push(';');
        }
        for raw in self.highest_fixed + 1..self.sim.world.next_object_id {
            let Some(s) = self.sim.find_object(ObjectId(raw)) else {
                continue;
            };
            let name = template_name(self.sim, s);
            let o = self.sim.object(s).expect("object");
            let physics = o.physics.as_ref().map_or_else(
                || "-".to_owned(),
                |p| format!("{}{}", bits(p.pitch_rate), u8::from(p.flags & 4 != 0)),
            );
            let _ = write!(
                sec[4],
                "{raw},{name},{},{},{},{},{},{physics};",
                o.producer_id.0,
                o.experience_tracker.experience_sink.0,
                pos_bits(o.thing.get_position()),
                bits(o.thing.get_orientation()),
                u8::from(o.is_destroyed()),
            );
        }
        let money = |team: TeamId| self.sim.player_money(team);
        sec[5] = format!(
            "{},{},{},{},{}",
            money(TEAMS[1]),
            money(TEAMS[2]),
            self.sim
                .world.players.player(self.sim.player_index(TEAMS[1])).map(|p| &p.score_keeper)
                .map_or(0, |k| k.total_money_earned),
            self.sim
                .world
                .skill_points_awarded
                .get(&1)
                .copied()
                .unwrap_or(0),
            self.sim
                .world.players.player(1).map(|p| &p.academy)
                .map_or(0, |a| a.structures_captured),
        );

        let events = {
            let mut log = self.log.lock().unwrap();
            let mut e = String::new();
            for s in log.ai.drain(..) {
                let _ = write!(e, "{s} ");
            }
            e.push('/');
            for s in log.audio.drain(..) {
                let _ = write!(e, "{s} ");
            }
            e.push('/');
            for s in log.text.drain(..) {
                let _ = write!(e, "{s} ");
            }
            e.push('/');
            for flag in std::mem::take(&mut log.eva) {
                let _ = write!(e, "{flag}");
            }
            e.push('/');
            for s in log.particles.drain(..) {
                let _ = write!(e, "{s} ");
            }
            e
        };

        let mut row = vec![perms.to_owned()];
        for (k, s) in sec.iter().enumerate() {
            row.push(if *s == self.previous[k] {
                "=".to_owned()
            } else {
                s.clone()
            });
            self.previous[k].clone_from(s);
        }
        row.push(events);
        row.push(
            self.sim
                .world
                .rng
                .get_game_logic_random_seed_crc()
                .to_string(),
        );
        row.push(ret.to_string());
        row
    }

    /// Fills the action manager's answers from a row's first field.
    fn permit(&mut self, perms: &str, button: Option<usize>) {
        self.sim.world.action_permissions.clear();
        let Some(button) = button.and_then(|b| self.sim.rules.command_store.button(b)) else {
            return;
        };
        let power = button.special_power_id;
        let enter = match button.command {
            GuiCommandType::HijackVehicle => Some(ActionQuery::HijackVehicle),
            GuiCommandType::SabotageBuilding => Some(ActionQuery::SabotageBuilding),
            GuiCommandType::ConvertToCarBomb => Some(ActionQuery::ConvertToCarBomb),
            _ => None,
        };
        for token in perms.split_whitespace() {
            let other = ObjectId(token[1..].parse().expect("id"));
            let query = if token.starts_with('p') {
                ActionQuery::SpecialPowerAtObject(power.expect("power"))
            } else {
                enter.expect("enter command")
            };
            self.sim
                .world
                .action_permissions
                .insert((self.user, other, query));
        }
    }

    fn current_button(&self) -> Option<usize> {
        let payload = self.sim.world.modules.payload(self.hunt?)?;
        payload
            .as_any()
            .and_then(|a| a.downcast_ref::<CommandButtonHuntUpdate>())
            .and_then(|h| h.command_button)
    }

    fn run_frame(&mut self, perms: &str) {
        self.sim.world.frame += 1;
        let frame = self.sim.world.frame;
        if self.user_slot().is_some() {
            if let Some(h) = self.hunt {
                let button = self.current_button();
                self.permit(perms, button);
                step(self.sim, h, frame);
            }
            let destroyed = self
                .user_slot()
                .and_then(|u| self.sim.object(u))
                .is_none_or(|o| o.is_destroyed());
            if !destroyed {
                for m in self.abilities.clone() {
                    self.remember_positions();
                    step(self.sim, m, frame);
                }
            }
        }
        self.sim.update_partition();
        self.sim.process_destroy_list();
    }
}

fn check(vector: usize, row: usize, want: &str, got: &[String], previous_op: &str) {
    let want: Vec<&str> = want.split('|').collect();
    const NAMES: [&str; 10] = [
        "permissions",
        "abilities",
        "hunt",
        "user",
        "targets",
        "spawned",
        "players",
        "events",
        "rng",
        "ret",
    ];
    for (k, name) in NAMES.iter().enumerate() {
        assert_eq!(
            got[k], want[k],
            "vector {vector} row {row} ({previous_op}): {name} differs\n"
        );
    }
}

#[test]
fn golden_special_ability_and_command_button_hunt() {
    let mut at = 0;
    let root = Json::parse(VECTORS.as_bytes(), &mut at);
    let log = AI_LOG.with(Arc::clone);
    let mut sim = SimContext::new();
    sim.rules = build_rules(&root);
    let have = sim
        .rules
        .science_store
        .get_science_from_internal_name("SCIENCE_AB_HAVE", &mut sim.rules.name_keys);
    let users = root.get("users").arr();
    let mut loaded = 0;
    let vectors = root.get("vectors").arr();
    assert_eq!(vectors.len() as i64, root.get("vector_count").num());
    assert!(vectors.len() >= 1000);

    for (index, v) in vectors.iter().enumerate() {
        let get = |k: &str| v.get(k).num();
        let user = get("user") as usize;
        let positions = Arc::new(Mutex::new(BTreeMap::new()));
        sim.client = Box::new(Client(Arc::clone(&log), Arc::clone(&positions)));
        sim.terrain = Box::new(Wall(get("wall") as Real));
        sim.rules.ai_data.attack_priority_distance_modifier = get("modifier") as Real;
        sim.world = SimWorld::new();
        sim.init_partition(
            20.0,
            Region3D {
                lo: Coord3D::new(0.0, 0.0, 0.0),
                hi: Coord3D::new(1000.0, 1000.0, 100.0),
            },
        );
        {
            let w = &mut sim.world;
            for (player, team) in TEAMS.into_iter().enumerate() {
                w.team_players.insert(team, player as i32);
                w.player_default_teams.insert(player as i32, team);
            }
            w.team_is_neutral.insert(TEAMS[0], TRUE);
            w.neutral_team_id = TEAMS[0];
            w.local_player_index = 2;
            let ab = [
                Relationship::Enemies,
                Relationship::Neutral,
                Relationship::Allies,
            ][get("rel") as usize];
            for i in 0..3 {
                for j in 0..3 {
                    let rel = if i == j {
                        Relationship::Allies
                    } else if i == 0 || j == 0 {
                        Relationship::Neutral
                    } else {
                        ab
                    };
                    w.set_player_relation(i, j, rel);
                }
            }
            let owner = w.team_players[&TEAMS[1]];
            w.players.player_mut(owner).expect("player").push_science(have);
            w.players.player_mut(w.team_players[&TEAMS[1]]).expect("player").money.set_starting_cash(100);
            w.players.player_mut(w.team_players[&TEAMS[2]]).expect("player").money.set_starting_cash(get("money") as UnsignedInt);
            w.frame = get("frame") as u32;
            w.rng.init_random_with_seed(get("seed") as u32);
        }

        let mut ids = Vec::new();
        for s in v.get("setup").arr() {
            let s = s.arr();
            let pos = Coord3D::new(s[2].num() as Real, s[3].num() as Real, 0.0);
            let slot = spawn(&mut sim, s[0].str(), TEAMS[s[1].num() as usize], pos);
            ids.push(sim.object(slot).expect("object").id);
        }
        let me = sim.find_object(ids[0]).expect("user");
        assert_eq!(ids[0], USER_ID);
        let weapons = users[user].get("weapons").num();
        let frame = sim.world.frame;
        for (bit, wslot) in [WeaponSlotType::Primary, WeaponSlotType::Secondary]
            .into_iter()
            .enumerate()
        {
            if weapons & (1 << bit) != 0 {
                sim.object_mut(me).expect("user").weapon_set.weapons[bit] =
                    Some(Weapon::new(WeaponTemplate::default(), wslot, frame));
            }
        }
        for r in v.get("riders").arr() {
            let house = sim.find_object(ids[1 + r.at(0) as usize]).expect("house");
            let rider = sim.find_object(ids[1 + r.at(1) as usize]).expect("rider");
            sim.contain_add(house, rider);
        }
        for t in v.get("traps").arr() {
            let target = sim.find_object(ids[1 + t.at(0) as usize]).expect("target");
            let trap = sim.find_object(ids[1 + t.at(1) as usize]).expect("trap");
            let ms = modules_named(&mut sim, trap, "StickyBombUpdate")[0];
            sim.dispatch_module(ms, |m, sim, ctx| {
                let double = m
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<StickyDouble>());
                double
                    .expect("sticky")
                    .init(sim, ctx.owner_slot, target, None);
            })
            .expect("trap");
        }
        sim.update_partition();
        log.lock().unwrap().ai.clear();
        log.lock().unwrap().audio.clear();
        sim.world.rng.init_random_with_seed(get("seed") as u32 + 1);

        let abilities = modules_named(&mut sim, me, "SpecialAbilityUpdate");
        let hunt = modules_named(&mut sim, me, "CommandButtonHuntUpdate")
            .first()
            .copied();
        let powers = modules_named(&mut sim, me, "SpecialAbility");
        let sink = usize::try_from(get("sink")).map_or(ObjectId::INVALID, |t| ids[1 + t]);
        if let Some(o) = sim.object_mut(me) {
            o.experience_tracker.experience_sink = sink;
            o.experience_tracker.experience_scalar = get("scalar") as Real;
        }
        let highest_fixed = ids.last().expect("ids").0;
        let mut sc = Scene {
            sim: &mut sim,
            log: Arc::clone(&log),
            positions,
            user: ids[0],
            sink,
            fixed: ids[1..].to_vec(),
            highest_fixed,
            abilities,
            powers,
            hunt,
            previous: Default::default(),
            rotate: 0,
        };

        let rows: Vec<&str> = v.get("rows").arr().iter().map(Json::str).collect();
        let mut next_row = 0;
        let got = sc.observe("", 0);
        check(index, next_row, rows[next_row], &got, "start");
        next_row += 1;

        for op in v.get("ops").arr() {
            let a: Vec<i64> = op.arr().iter().map(Json::num).collect();
            let what = format!("op {a:?}");
            let me = sc.user_slot().expect("user");
            let ability_power = |sc: &Scene<'_>, k: usize| {
                let payload = sc
                    .sim
                    .world
                    .modules
                    .payload(sc.abilities[k])
                    .expect("payload");
                let ability = payload
                    .as_any()
                    .and_then(|a| a.downcast_ref::<SpecialAbilityUpdate>())
                    .expect("ability");
                ability.data.special_power_id
            };
            let power_id = |sc: &Scene<'_>, p: i64| {
                sc.sim
                    .rules
                    .special_power_store
                    .find_special_power_template(&format!("AB_SP{p}"))
                    .expect("power")
                    .id()
            };
            sc.remember_positions();
            let mut perms = String::new();
            let mut ret = 0;
            match a[0] {
                0..=2 => {
                    let id = ability_power(&sc, a[1] as usize);
                    let ms = sc.sim.find_special_power_module(me, id).expect("spm");
                    let target = if a[0] == 0 {
                        sc.slot(a[2] as usize)
                    } else {
                        None
                    };
                    let pos = Coord3D::new(a[2] as Real, a[3] as Real, 0.0);
                    sc.sim
                        .dispatch_module(ms, |m, sim, ctx| {
                            let sp = m.as_special_power_mut().expect("special power");
                            match a[0] {
                                0 => sp.do_special_power_at_object(
                                    sim,
                                    ctx,
                                    target.expect("target"),
                                    0,
                                ),
                                1 => sp.do_special_power_at_location(sim, ctx, &pos, 0.0, 0),
                                _ => sp.do_special_power(sim, ctx, 0),
                            }
                        })
                        .expect("special power");
                }
                3 => {
                    let template = power_id(&sc, a[2]);
                    let target = if a[3] == 0 {
                        sc.slot(a[4] as usize)
                    } else {
                        None
                    };
                    let pos = Coord3D::new(a[4] as Real, a[5] as Real, 0.0);
                    let pos = (a[3] == 1).then_some(pos);
                    ret = sc
                        .sim
                        .dispatch_module(sc.abilities[a[1] as usize], |m, sim, ctx| {
                            m.as_special_power_update_mut()
                                .expect("facet")
                                .initiate_intent_to_do_special_power(
                                    sim,
                                    ctx,
                                    SpecialPowerCaller {
                                        ctx,
                                        module: &mut NoCaller,
                                    },
                                    template,
                                    target,
                                    pos.as_ref(),
                                    0,
                                )
                        })
                        .expect("initiate")
                        .into();
                }
                4 => {
                    for _ in 0..a[1] {
                        let want = rows[next_row].split('|').next().expect("perms").to_owned();
                        let ran_hunt = sc.hunt.is_some_and(|h| {
                            sc.user_slot().is_some()
                                && next_call_frame(sc.sim, h) <= sc.sim.world.frame + 1
                        });
                        // The permissions are the reference's input; check only that both
                        // sides expect the hunt to run.
                        let perms = if ran_hunt {
                            want.clone()
                        } else {
                            String::new()
                        };
                        sc.run_frame(&perms);
                        sc.rotate += 1;
                        let got = sc.observe(&perms, 0);
                        check(index, next_row, rows[next_row], &got, &what);
                        next_row += 1;
                    }
                    continue;
                }
                5 => {
                    sc.ai(|ai| {
                        ai.idle = a[1] != 0;
                        ai.moving = a[2] != 0;
                    });
                }
                6 => {
                    let target = sc.slot((a[1] / 2) as usize).expect("target");
                    let (mover, anchor) = if a[1] % 2 != 0 {
                        (target, me)
                    } else {
                        (me, target)
                    };
                    let at = *sc.sim.object(anchor).expect("anchor").thing.get_position();
                    let off = OFFSETS[a[2] as usize];
                    let pos = Coord3D::new(at.x + off[0], at.y + off[1], a[3] as Real);
                    sc.sim.set_object_position(mover, &pos);
                    sc.sim.update_partition();
                }
                7 => {
                    let target = sc.slot(a[1] as usize).expect("target");
                    let status = ObjectStatusType::from_i32(a[2] as i32).expect("status");
                    if a[3] != 0 {
                        sc.sim.set_status(target, status);
                    } else {
                        sc.sim.clear_status(target, status);
                    }
                }
                8 => {
                    match a[1] {
                        0 => {
                            let t = sc.slot(a[2] as usize).expect("target");
                            set_dead(sc.sim, t, true);
                        }
                        1 => {
                            let t = sc.slot(a[2] as usize).expect("target");
                            sc.sim.destroy_object(t);
                        }
                        _ => {
                            let special =
                                sc.sim.find_object(ObjectId(a[2] as i32)).expect("special");
                            sc.sim.destroy_object(special);
                        }
                    }
                    sc.sim.process_destroy_list();
                }
                9 => {
                    let src = match a[1] {
                        0 => CommandSourceType::FromPlayer,
                        1 => CommandSourceType::FromScript,
                        2 => CommandSourceType::FromAi,
                        other => panic!("source {other}"),
                    };
                    sc.sim.object_mut(me).expect("user").ai_last_command_source = src;
                }
                10 => {
                    let target = sc.slot(a[1] as usize).expect("target");
                    sc.sim.set_object_team(target, TEAMS[a[2] as usize]);
                }
                11 => set_dead(sc.sim, me, a[1] != 0),
                12 => {
                    sc.sim.destroy_object(me);
                    sc.sim.process_destroy_list();
                }
                13 => {
                    let name = if a[1] < 0 {
                        "AB_Unknown".to_owned()
                    } else {
                        sc.sim
                            .rules
                            .command_store
                            .button(a[1] as usize)
                            .expect("button")
                            .name
                            .clone()
                    };
                    perms = rows[next_row].split('|').next().expect("perms").to_owned();
                    let button = (a[1] >= 0).then_some(a[1] as usize);
                    sc.permit(&perms, button);
                    sc.sim.set_command_button_hunt(me, &name);
                }
                14 => {
                    ret = sc
                        .sim
                        .dispatch_module(sc.abilities[a[1] as usize], |m, sim, ctx| {
                            m.as_update_mut().expect("update").update(sim, ctx).0
                        })
                        .expect("update");
                }
                15 => {
                    if a[1] == 0 {
                        let key = (format!("AB_Set{user}"), a[2] as Int);
                        let button = (a[3] >= 0).then_some(a[3] as usize);
                        sc.sim.world.control_bar_overrides.insert(key, button);
                    } else {
                        let name = if a[2] < 0 {
                            "AB_NoSet".to_owned()
                        } else {
                            format!("AB_Set{}", a[2])
                        };
                        sc.sim.set_command_set(me, &name);
                    }
                }
                16 => {
                    let until = sc.sim.world.frame + a[1] as u32;
                    sc.sim.set_disabled_until(me, DisabledType::Hacked, until);
                }
                17 => {
                    let names: Vec<String> = [
                        "AB_Big",
                        "AB_SmallCyl",
                        "AB_SmallBox",
                        "AB_Vehicle",
                        "AB_Garrison",
                        "AB_Infantry",
                        "AB_Truck",
                        "AB_Ball",
                        "AB_Edge",
                    ]
                    .map(str::to_owned)
                    .to_vec();
                    let map: BTreeMap<String, Int> = names
                        .into_iter()
                        .zip(&a[3..])
                        .filter(|(_, &v)| v >= 0)
                        .map(|(n, &v)| (n, v as Int))
                        .collect();
                    sc.ai(|ai| ai.priorities = (a[1] != 0).then_some((a[2] as Int, map)));
                }
                18 => {
                    let id = ability_power(&sc, a[1] as usize);
                    sc.sim.special_power_start_recharge(me, id);
                }
                19 => {
                    let truck = sc.slot(a[1] as usize).expect("truck");
                    let like = if a[2] < 0 {
                        me
                    } else {
                        sc.slot(a[2] as usize).expect("like")
                    };
                    sc.sim.stealth_disguise_as_object(truck, Some(like));
                    sc.sim.set_status(truck, ObjectStatusType::Stealthed);
                    sc.sim.set_status(truck, ObjectStatusType::Disguised);
                }
                20 => load_back(sc.sim, sc.hunt.expect("hunt")),
                21 => {
                    let target = if a[1] < 0 {
                        me
                    } else {
                        sc.slot(a[1] as usize).expect("target")
                    };
                    let pos = Coord3D::new(-3.0, a[2] as Real, 0.0);
                    sc.sim.set_object_position(target, &pos);
                    sc.sim.update_partition();
                }
                other => panic!("op {other}"),
            }
            sc.rotate += 1;
            let got = sc.observe(&perms, ret);
            check(index, next_row, rows[next_row], &got, &what);
            next_row += 1;
        }
        assert_eq!(next_row, rows.len(), "vector {index}: row count");
        if sc.user_slot().is_some() {
            for m in sc.abilities.clone() {
                round_trip(sc.sim, m);
                loaded += 1;
            }
        }
    }
    assert!(loaded > 500, "{loaded} modules loaded back");
}

/// The calling special power module of a direct call; nothing here calls back into it.
struct NoCaller;

impl SpecialPowerFacet for NoCaller {
    fn is_ready(&self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> Bool {
        unreachable!("no special power module in this call")
    }
    fn get_ready_frame(&self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UnsignedInt {
        unreachable!("no special power module in this call")
    }
    fn get_raw_ready_frame(&self) -> UnsignedInt {
        unreachable!("no special power module in this call")
    }
    fn set_ready_frame(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _frame: UnsignedInt) {
        unreachable!("no special power module in this call")
    }
}
