//! Second set of joins: minefields under physics vehicles, toppling and helicopter slow
//! death, battle plans, special abilities with stealth, hordes, parking with production
//! and the first set's scene, all saving and loading module state mid-way.
//!
//! Vectors: `docs/golden_vectors/JOINS2_Modules.json`
//! (`tests/cpp_harness/gen_joins2_modules.cpp`). No ported module is replaced; the only
//! stand-in is the AI, as in the harness. The AI stand-in and the observation are those of
//! `golden_joins_modules.rs`. To see a mismatch in full, run the harness with
//! `JOINS_HEX=1` into a scratch file and this test with `JOINS_HEX=1` and
//! `JOINS_VECTORS=<that file>`; `JOINS_ONLY=<n>` runs one vector.
//!
//! On each load the C++ bytes are loaded twice: into a module newly built from its data,
//! which must save them again, and into the live module, which then runs on.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Relationship, TeamId, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::behavior::spawn_behavior::SpawnBehavior;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CommandSourceType, ModuleData, ModuleType, ProductionFacet,
    SupplyTruckAiFacet, UpdateFacet,
};
use zh_logic::modules::module_factory::{ModuleDataParseFn, ModuleParseContext};
use zh_logic::modules::update::production_update::ProductionUpdate;
use zh_logic::modules::update::special_ability_update::SpecialAbilityUpdate;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::command_set::{CommandButton, CommandButtonType};
use zh_logic::rules::disabled_types::{DisabledMaskType, DisabledType};
use zh_logic::rules::model_state::MODELCONDITION_COUNT;
use zh_logic::rules::object_status_types::OBJECT_STATUS_COUNT;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const COMMITTED: &str = include_str!("../../../docs/golden_vectors/JOINS2_Modules.json");
const FULL_VECTORS: usize = 7;
const TEAMS: [TeamId; 3] = [TeamId(10), TeamId(11), TeamId(12)];
const PRODUCIBLE: [&str; 6] = ["J_Tank", "J_Inf", "J_Truck", "J_Plain", "J_Var", "J_Mob"];
const EXTRA: [&str; 6] = [
    "J_Plant",
    "J_Tech",
    "J_Detector",
    "J_Truck",
    "J_Tank",
    "J_Hive1",
];
const UPGRADES: [&str; 3] = ["J_UpgPlant", "J_UpgBoost", "J_UpgSlave"];
const DISABLED: [DisabledType; 3] = [
    DisabledType::Emp,
    DisabledType::Hacked,
    DisabledType::Paralyzed,
];
const ANGLES: [f32; 8] = [
    0.0,
    1.570_796_4,
    3.141_592_7,
    -1.570_796_4,
    0.5,
    -2.75,
    6.0,
    0.001,
];

// ---- the AI stand-in ---------------------------------------------------------------

thread_local! {
    static TAGS: RefCell<BTreeMap<i32, &'static str>> = const { RefCell::new(BTreeMap::new()) }; // determinism-ok: test cache
    static ORDERS: RefCell<String> = const { RefCell::new(String::new()) }; // determinism-ok: test log
}

fn order(line: String) {
    ORDERS.with(|o| {
        let mut o = o.borrow_mut();
        o.push_str(&line);
        o.push(';');
    });
}

#[derive(Clone)]
struct AiData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    truck: bool,
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
        AI_MASK
    }
}

struct Ai {
    data: AiData,
    id: ObjectId,
    idle: Bool,
    boxes: Int,
    wanting: Bool,
    ignored: ObjectId,
}

fn bits3(c: &Coord3D) -> String {
    format!("{} {} {}", c.x.to_bits(), c.y.to_bits(), c.z.to_bits())
}

impl AiUpdateFacet for Ai {
    fn is_idle(&self) -> Bool {
        self.idle
    }
    fn supply_truck(&mut self) -> Option<&mut dyn SupplyTruckAiFacet> {
        if self.data.truck {
            Some(self)
        } else {
            None
        }
    }
    fn ai_idle(&mut self, _s: &mut dyn SimApi, _c: ModuleContext, src: CommandSourceType) {
        order(format!("{} idle {}", self.id.0, src as i32));
    }
    fn ai_exit(
        &mut self,
        sim: &mut dyn SimApi,
        _c: ModuleContext,
        container: ObjectSlot,
        src: CommandSourceType,
    ) {
        let other = sim.object(container).map_or(0, |o| o.id.0);
        order(format!("{} exit {} {other}", self.id.0, src as i32));
    }
    fn ai_move_to_position(
        &mut self,
        _s: &mut dyn SimApi,
        _c: ModuleContext,
        pos: &Coord3D,
        src: CommandSourceType,
    ) {
        order(format!("{} move {} {}", self.id.0, src as i32, bits3(pos)));
    }
    fn ai_force_attack_object(
        &mut self,
        sim: &mut dyn SimApi,
        _c: ModuleContext,
        victim: ObjectSlot,
        max_shots: Int,
        src: CommandSourceType,
    ) {
        let victim = sim.object(victim).map_or(0, |o| o.id.0);
        order(format!(
            "{} force {} {victim} {max_shots}",
            self.id.0, src as i32
        ));
    }
    fn ai_attack_position(
        &mut self,
        _s: &mut dyn SimApi,
        _c: ModuleContext,
        pos: &Coord3D,
        max_shots: Int,
        src: CommandSourceType,
    ) {
        order(format!(
            "{} attackpos {} {} {max_shots}",
            self.id.0,
            src as i32,
            bits3(pos)
        ));
    }
    fn ai_move_to_object(
        &mut self,
        sim: &mut dyn SimApi,
        _c: ModuleContext,
        target: ObjectSlot,
        src: CommandSourceType,
    ) {
        let target = sim.object(target).map_or(0, |o| o.id.0);
        order(format!("{} moveobj {} {target}", self.id.0, src as i32));
    }
    fn ai_attack_object(
        &mut self,
        sim: &mut dyn SimApi,
        _c: ModuleContext,
        target: ObjectSlot,
        _max_shots: Int,
        src: CommandSourceType,
    ) {
        let target = sim.object(target).map_or(0, |o| o.id.0);
        order(format!("{} attack {} {target}", self.id.0, src as i32));
    }
    fn ai_busy(&mut self, _s: &mut dyn SimApi, _c: ModuleContext, src: CommandSourceType) {
        order(format!("{} busy {}", self.id.0, src as i32));
    }
    fn ai_face_object(
        &mut self,
        sim: &mut dyn SimApi,
        _c: ModuleContext,
        target: ObjectSlot,
        src: CommandSourceType,
    ) {
        let target = sim.object(target).map_or(0, |o| o.id.0);
        order(format!("{} faceobj {} {target}", self.id.0, src as i32));
    }
    fn ai_face_position(
        &mut self,
        _s: &mut dyn SimApi,
        _c: ModuleContext,
        pos: &Coord3D,
        src: CommandSourceType,
    ) {
        order(format!(
            "{} facepos {} {}",
            self.id.0,
            src as i32,
            bits3(pos)
        ));
    }
    fn ignore_obstacle(&mut self, obstacle: ObjectId) {
        self.ignored = obstacle;
    }
    fn get_ignored_obstacle_id(&self) -> ObjectId {
        self.ignored
    }
}

impl SupplyTruckAiFacet for Ai {
    fn number_boxes(&self) -> i32 {
        self.boxes
    }
    fn max_boxes(&self) -> i32 {
        4
    }
    fn upgraded_supply_boost(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> i32 {
        20
    }
    fn lose_one_box(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> Bool {
        if self.boxes == 0 {
            return FALSE;
        }
        self.boxes -= 1;
        TRUE
    }
    fn gain_one_box(&mut self, _s: &mut dyn SimApi, _c: ModuleContext, _left: i32) -> Bool {
        FALSE
    }
    fn set_force_wanting_state(&mut self, wanting: Bool) {
        self.wanting = wanting;
    }
}

impl UpdateFacet for Ai {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
    fn disabled_types_to_process(&self) -> DisabledMaskType {
        let mut mask = DisabledMaskType::new();
        for i in 0..32 {
            mask.set(i, 1);
        }
        mask
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.id = ctx.owner_id;
        sim.ai_construct(ctx.owner_slot);
        // `AIUpdateInterface`'s constructor ends with `setWakeFrame(UPDATE_SLEEP_NONE)`.
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        // The state machine's draws happen, but no order is acted on afterwards.
        sim.ai_init_state_machine(ctx.owner_slot);
        if let Some(o) = sim.object_mut(ctx.owner_slot) {
            o.ai_state_machine_ready = FALSE;
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
    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }
    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

fn parse_ai_data(
    reader: &mut IniReader<'_>,
    data: AiData,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(data))
}

fn parse_ai(
    r: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_ai_data(
        r,
        AiData {
            tag_key,
            module_name_key,
            truck: false,
        },
    )
}

fn parse_truck(
    r: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_ai_data(
        r,
        AiData {
            tag_key,
            module_name_key,
            truck: true,
        },
    )
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let data = data.as_any().downcast_ref::<AiData>().expect("ai").clone();
    Box::new(Ai {
        data,
        id: ObjectId::INVALID,
        idle: TRUE,
        boxes: 0,
        wanting: FALSE,
        ignored: ObjectId::INVALID,
    })
}

// ---- rules and world ---------------------------------------------------------------

/// The committed vectors, or the file `JOINS_VECTORS` names.
fn vectors() -> &'static str {
    static VECTORS: std::sync::OnceLock<String> = std::sync::OnceLock::new();
    VECTORS.get_or_init(|| match std::env::var("JOINS_VECTORS") {
        Ok(path) => std::fs::read_to_string(path).expect("JOINS_VECTORS"),
        Err(_) => COMMITTED.to_owned(),
    })
}

fn text_field(name: &str) -> String {
    get_str(vectors(), name).expect(name).replace('~', "\n")
}

fn reader(text: &str) -> IniReader<'_> {
    IniReader::new("joins.ini", text.as_bytes(), IniLoadType::Overwrite)
}

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    for (name, parse) in [
        ("JoinAI", parse_ai as ModuleDataParseFn),
        ("JoinTruckAI", parse_truck),
        ("J2AI", parse_ai),
    ] {
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
    rules
        .upgrade_center
        .load_from_ini(&mut reader(&text_field("upgrades")), &mut rules.name_keys)
        .expect("upgrades");
    let mut locomotor = LocomotorTemplate::new();
    locomotor
        .parse_ini_block(&text_field("locomotor"))
        .expect("locomotor");
    rules.locomotor_store.add_template(locomotor);
    let mut air = LocomotorTemplate::new();
    air.parse_ini_block(&text_field("locomotor2"))
        .expect("air locomotor");
    rules.locomotor_store.add_template(air);
    for block in text_field("weapons").split_inclusive("End\n") {
        let mut weapon = WeaponTemplate::new();
        weapon.parse_ini_block(block.trim_end()).expect("weapon");
        rules.weapon_store.add_template(weapon);
    }
    rules
        .special_power_store
        .load_from_ini(
            &mut reader(&text_field("powers")),
            &rules.science_store,
            &mut rules.name_keys,
        )
        .expect("powers");
    let objects = text_field("fixture");
    let mut r = reader(&objects);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut r) {
        panic!("thing templates, line {}: {e:?}", r.get_line_num());
    }
    rules.thing_factory.post_process_load();
    rules.command_sets.insert(
        "J_Set",
        PRODUCIBLE
            .iter()
            .map(|&u| CommandButton {
                command: CommandButtonType::UnitBuild,
                object: u.to_owned(),
                upgrade: String::new(),
            })
            .collect(),
    );
    rules.command_sets.insert(
        "J2_Set",
        JETS.iter()
            .map(|&u| CommandButton {
                command: CommandButtonType::UnitBuild,
                object: u.to_owned(),
                upgrade: String::new(),
            })
            .collect(),
    );
    rules.global_data.gravity = -1.0;
    rules.global_data.base_value_per_supply_box = 75;
    rules
}

fn world(sim: &mut SimContext) {
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    sim.world.frame = 1;
    let w = &mut sim.world;
    for (player, team) in TEAMS.into_iter().enumerate() {
        w.team_players.insert(team, player as i32);
        w.player_default_teams.insert(player as i32, team);
        let money = &mut w.players.player_mut(player as i32).expect("player").money;
        // As the harness does: without a template the money names player 0.
        money.set_player_index(player as i32);
        money.set_starting_cash(5000);
        w.set_team_human(team, FALSE);
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
            w.set_player_relation(i, j, rel);
        }
    }
}

// ---- observation -------------------------------------------------------------------

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn digest(text: &str) -> String {
    let mut h: u32 = 2_166_136_261;
    for b in text.bytes() {
        h ^= u32::from(b);
        h = h.wrapping_mul(16_777_619);
    }
    format!("{h:08x}")
}

fn shown(text: &str, full: bool) -> String {
    if full || text.len() <= 24 {
        text.to_owned()
    } else {
        format!("#{}", digest(text))
    }
}

fn template_name(sim: &SimContext, o: ObjectSlot) -> String {
    let rec = sim.world.objects.get(o).expect("object");
    rec.thing
        .template_override()
        .get_final(&sim.rules.thing_factory)
        .map_or_else(String::new, |t| t.get_name().to_owned())
}

/// `NameKeyGenerator::key_to_name` searches every key; the tags are few, so keep them.
fn tag_name(sim: &SimContext, tag: NameKeyType) -> &'static str {
    TAGS.with(|tags| {
        *tags.borrow_mut().entry(tag.0).or_insert_with(|| {
            Box::leak(
                sim.rules
                    .name_keys
                    .key_to_name(tag)
                    .to_owned()
                    .into_boxed_str(),
            )
        })
    })
}

fn modules_of(sim: &SimContext, o: ObjectSlot) -> Vec<(ModuleSlot, &'static str)> {
    sim.world
        .objects
        .get(o)
        .expect("object")
        .behaviors
        .iter()
        .map(|&(ms, _)| (ms, tag_name(sim, sim.world.modules.meta(ms).tag_key)))
        .collect()
}

fn module_tagged(sim: &SimContext, o: ObjectSlot, tag: &str) -> Option<ModuleSlot> {
    modules_of(sim, o)
        .into_iter()
        .find(|&(_, t)| t == tag)
        .map(|(ms, _)| ms)
}

fn with_ai<R>(sim: &mut SimContext, o: ObjectSlot, f: impl FnOnce(&mut Ai) -> R) -> Option<R> {
    let ms = module_tagged(sim, o, "ModuleTag_AI")?;
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).ok()?;
    let out = payload
        .as_any_mut()
        .and_then(|a| a.downcast_mut::<Ai>())
        .map(f);
    sim.world.modules.restore_payload(ms, generation, payload);
    out
}

fn objects_by_id(sim: &SimContext) -> Vec<(i32, ObjectSlot)> {
    let mut out: Vec<(i32, ObjectSlot)> = sim
        .world
        .objects
        .world_iter()
        .map(|s| (sim.world.objects.get(s).expect("object").id.0, s))
        .collect();
    out.sort_by_key(|&(id, _)| id);
    out
}

fn object_row(sim: &mut SimContext, o: ObjectSlot, in_hex: bool) -> String {
    let name = template_name(sim, o);
    let health = sim.object_health(o).to_bits();
    let team = sim.world.objects.get(o).expect("object").team_id;
    let player = TEAMS
        .iter()
        .position(|&t| t == team)
        .map_or(-1, |p| p as i32);
    let rec = sim.world.objects.get(o).expect("object");
    let mut status: u64 = 0;
    for b in 0..OBJECT_STATUS_COUNT {
        if rec.status.test(b as i32) != FALSE {
            status |= 1 << b;
        }
    }
    let p = *rec.thing.get_position();
    let mut s = format!(
        "{name} {player} {status} {} {} {} {} {health} {} [",
        rec.disabled_mask.to_unsigned_int(),
        u8::from(rec.is_effectively_dead() != FALSE),
        bits3(&p),
        rec.thing.get_orientation().to_bits(),
        rec.producer_id.0,
    );
    for b in 0..MODELCONDITION_COUNT {
        if rec.model_condition.test(b as i32) != FALSE {
            let _ = write!(s, "{b},");
        }
    }
    s.push(']');
    for (ms, tag) in modules_of(sim, o) {
        if !tag.starts_with("ModuleTag_") || tag == "ModuleTag_AI" || tag.ends_with("Helper") {
            continue;
        }
        let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).expect("payload");
        let mut xs = XferSave::new();
        payload.xfer(&mut xs, sim).expect("xfer save");
        let mut xc = XferCrc::new();
        payload.crc(&mut xc);
        sim.world.modules.restore_payload(ms, generation, payload);
        let mut bytes = xs.as_bytes().to_vec();
        // The reference saves SpawnBehavior version 2: no burst countdown, which
        // follows the three module versions and a flag in version 3.
        let mut countdown = None;
        if tag == "ModuleTag_Spawn" {
            assert_eq!(bytes[0], 3);
            bytes[0] = 2;
            let field: Vec<u8> = bytes.drain(5..9).collect();
            countdown = Some(u32::from_le_bytes(field.try_into().expect("four bytes")));
        }
        let save = hex(&bytes);
        let save = if in_hex { save } else { digest(&save) };
        let _ = write!(s, " {}={save}/{}", &tag[10..], xc.get_crc());
        if let Some(countdown) = countdown {
            let _ = write!(s, "+{countdown}");
        }
    }
    if let Some(Some((boxes, wanting))) =
        with_ai(sim, o, |ai| ai.data.truck.then_some((ai.boxes, ai.wanting)))
    {
        let _ = write!(
            s,
            " truck {boxes}{}",
            if wanting != FALSE { 'w' } else { '-' }
        );
    }
    s
}

fn players_row(sim: &SimContext) -> String {
    let mut s = String::new();
    for (p, team) in TEAMS.into_iter().enumerate() {
        let p = p as i32;
        let frame = sim.world.frame;
        let energy = sim.world.players.player(p).map(|p| &p.energy);
        let (prod, cons) =
            energy.map_or((0, 0), |e| (e.get_production(frame), e.get_consumption()));
        let sk = sim.world.players.player(p).map(|p| p.score_keeper.clone()).unwrap_or_else(|| zh_logic::player::player::Player::blank(-1).score_keeper);
        let a = sim
            .world.players.player(p).map(|p| p.academy.clone()).unwrap_or_else(|| zh_logic::player::player::Player::blank(-1).academy);
        let _ = write!(
            s,
            "{},{prod},{cons},{},{},{},{},{},{},{},{},{},{},{},{};",
            sim.player_money(team),
            sk.total_money_earned,
            sk.total_money_spent,
            sk.total_units_built,
            sk.total_buildings_built,
            sk.total_tech_buildings_captured,
            sk.total_faction_buildings_captured,
            a.last_income_frame,
            a.max_frames_between_income,
            a.supply_centers_built,
            a.peons_built,
            a.gatherers_built,
            a.last_unit_built_frame,
        );
    }
    s
}

// ---- operations --------------------------------------------------------------------

struct Run {
    full: bool,
    in_hex: bool,
    last: Vec<(i32, String)>,
    last_players: String,
}

impl Run {
    fn row(&mut self, sim: &mut SimContext, ret: i32) -> String {
        let mut s = format!("{ret} {}@", sim.world.frame);
        let mut now = Vec::new();
        for (id, o) in objects_by_id(sim) {
            now.push((id, object_row(sim, o, self.in_hex)));
        }
        for (id, text) in &now {
            if !self.last.iter().any(|(i, t)| i == id && t == text) {
                let _ = write!(s, "{id}:{}^", shown(text, self.full));
            }
        }
        for (id, _) in &self.last {
            if !now.iter().any(|(i, _)| i == id) {
                let _ = write!(s, "-{id}^");
            }
        }
        self.last = now;
        let players = players_row(sim);
        let players_shown = if players == self.last_players {
            "=".to_owned()
        } else {
            shown(&players, self.full)
        };
        self.last_players = players;

        let orders = ORDERS.with(|o| std::mem::take(&mut *o.borrow_mut()));
        let mut paths = String::new();
        for p in std::mem::take(&mut sim.world.exit_path_orders) {
            let _ = write!(
                paths,
                "{} {} {}",
                p.object.0, p.ignore.0, p.cmd_source as i32
            );
            for c in &p.path {
                let _ = write!(paths, " {}", bits3(c));
            }
            paths.push(';');
        }
        let mut prone = String::new();
        for (id, what) in std::mem::take(&mut sim.world.ai_commands) {
            if what == "aiGoProne" {
                let _ = write!(prone, "{};", id.0);
            }
        }
        let mut texts = String::new();
        for (_, pos, amount) in std::mem::take(&mut sim.world.floating_money_texts) {
            let _ = write!(texts, "{} {amount};", bits3(&pos));
        }
        let _ = write!(
            s,
            "@{players_shown}@{}@{}@{}@{}@{}",
            shown(&orders, self.full),
            shown(&paths, self.full),
            shown(&prone, self.full),
            shown(&texts, self.full),
            sim.world.rng.get_game_logic_random_seed_crc()
        );
        s
    }
}

fn spawn(sim: &mut SimContext, name: &str, team: usize, x: f32, y: f32, angle: f32) -> ObjectSlot {
    let tmpl = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let o = sim.factory_new_object(tmpl, TEAMS[team]);
    sim.set_object_position(o, &Coord3D::new(x, y, 0.0));
    sim.set_object_orientation(o, angle);
    o
}

fn run_frame(sim: &mut SimContext) {
    sim.step_sleepy_updates();
    sim.update_partition();
    sim.process_destroy_list();
    sim.world.frame += 1;
}

fn live(sim: &SimContext) -> Vec<ObjectSlot> {
    objects_by_id(sim)
        .into_iter()
        .map(|(_, s)| s)
        .filter(|&s| !sim.world.objects.get(s).expect("object").is_destroyed())
        .collect()
}

fn live_of(sim: &mut SimContext, what: &str) -> Vec<ObjectSlot> {
    let mut out = Vec::new();
    for o in live(sim) {
        let rec = sim.world.objects.get(o).expect("object");
        let has_ai = rec.ai_slot.is_some();
        let has_physics = rec.physics_slot.is_some();
        let keep = match what {
            "factory" => module_tagged(sim, o, "ModuleTag_Prod").is_some(),
            "exit" => module_tagged(sim, o, "ModuleTag_Exit").is_some(),
            "truck" => with_ai(sim, o, |ai| ai.data.truck) == Some(true),
            "dock" => module_tagged(sim, o, "ModuleTag_Dock").is_some(),
            "ai" => has_ai,
            "physics" => has_physics,
            "plant" => template_name(sim, o) == "J_Plant",
            _ => panic!("kind {what}"),
        };
        if keep {
            out.push(o);
        }
    }
    out
}

fn choose(list: &[ObjectSlot], a: u32) -> Option<ObjectSlot> {
    (!list.is_empty()).then(|| list[a as usize % list.len()])
}

fn with_production<R>(
    sim: &mut SimContext,
    owner: ObjectSlot,
    f: impl FnOnce(&mut ProductionUpdate, &mut dyn SimApi, ModuleContext) -> R,
) -> R {
    let ms = module_tagged(sim, owner, "ModuleTag_Prod").expect("production");
    sim.dispatch_module(ms, |m, s, c| {
        let p = m
            .as_any_mut()
            .and_then(|a| a.downcast_mut::<ProductionUpdate>())
            .expect("production");
        f(p, s, c)
    })
    .expect("dispatch")
}

fn id_of(sim: &SimContext, o: ObjectSlot) -> i32 {
    sim.world.objects.get(o).expect("object").id.0
}

/// The harness's `apply`; -1 when the operation had nothing to act on.
fn apply(sim: &mut SimContext, op: [u32; 4]) -> i32 {
    let [code, a, b, c] = op;
    match code {
        0 => {
            for _ in 0..a {
                run_frame(sim);
            }
            0
        }
        1 => {
            let Some(f) = choose(&live_of(sim, "factory"), a) else {
                return -1;
            };
            let tmpl = sim
                .rules
                .thing_factory
                .find_template_id(PRODUCIBLE[b as usize])
                .expect("unit");
            with_production(sim, f, |p, s, ctx| {
                let id = p.request_unique_unit_id();
                i32::from(p.queue_create_unit(s, ctx, tmpl, id) != FALSE)
            })
        }
        2 => {
            let Some(f) = choose(&live_of(sim, "factory"), a) else {
                return -1;
            };
            with_production(sim, f, |p, s, ctx| {
                p.cancel_unit_create(s, ctx, b as Int);
            });
            0
        }
        3 => {
            let player = sim.player_index(TEAMS[a as usize]);
            sim.set_player_cash(player, b);
            0
        }
        4 => {
            let Some(f) = choose(&live_of(sim, "exit"), a) else {
                return -1;
            };
            let ms = module_tagged(sim, f, "ModuleTag_Exit").expect("exit");
            let pos = Coord3D::new(b as f32, c as f32, 0.0);
            sim.dispatch_module(ms, |m, _, _| {
                m.as_exit_mut().expect("exit").set_rally_point(&pos);
            })
            .expect("dispatch");
            0
        }
        5 => {
            let (Some(d), Some(t)) = (
                choose(&live_of(sim, "dock"), a),
                choose(&live_of(sim, "truck"), b),
            ) else {
                return -1;
            };
            with_ai(sim, t, |ai| ai.boxes = c as Int).expect("truck");
            let ms = module_tagged(sim, d, "ModuleTag_Dock").expect("dock");
            sim.dispatch_module(ms, |m, s, ctx| {
                i32::from(m.as_dock_mut().expect("dock").action(s, ctx, t, None) != FALSE)
            })
            .expect("dispatch")
        }
        6 => {
            let Some(o) = choose(&live(sim), a) else {
                return -1;
            };
            sim.set_object_position(o, &Coord3D::new(b as f32, c as f32, 0.0));
            id_of(sim, o)
        }
        7 => {
            let Some(o) = choose(&live_of(sim, "physics"), a) else {
                return -1;
            };
            let f = Coord3D::new(
                (b as i32 - 8) as f32 * 0.5,
                (c as i32 - 8) as f32 * 0.5,
                (b % 3) as f32 * 1.0,
            );
            sim.physics_apply_force(o, &f);
            id_of(sim, o)
        }
        8 => {
            let Some(o) = choose(&live(sim), a) else {
                return -1;
            };
            let id = id_of(sim, o);
            sim.destroy_object(o);
            id
        }
        9 => {
            let Some(o) = choose(&live(sim), a) else {
                return -1;
            };
            let id = id_of(sim, o);
            let mut d = DamageInfo::default();
            d.input.damage_type = DamageType::Unresistable;
            d.input.death_type = DeathType::Normal;
            d.input.amount = b as f32;
            sim.attempt_damage(o, &mut d);
            id
        }
        10 => {
            let Some(o) = choose(&live(sim), a) else {
                return -1;
            };
            if c != 0 {
                sim.set_disabled(o, DISABLED[b as usize]);
            } else {
                sim.clear_disabled(o, DISABLED[b as usize]);
            }
            id_of(sim, o)
        }
        11 => {
            let Some(o) = choose(&live(sim), a) else {
                return -1;
            };
            sim.set_object_team(o, TEAMS[b as usize]);
            id_of(sim, o)
        }
        12 => {
            let Some(o) = choose(&live_of(sim, "plant"), a) else {
                return -1;
            };
            let key = sim.rules.name_keys.name_to_key(UPGRADES[0]);
            let mask = sim
                .rules
                .upgrade_center
                .find_upgrade_by_key(key)
                .expect("upgrade")
                .upgrade_mask()
                .clone();
            if sim.object_has_upgrade(o, &mask) != FALSE {
                return 0;
            }
            sim.grant_upgrade(o, UPGRADES[0], FALSE);
            id_of(sim, o)
        }
        13 => {
            sim.grant_player_upgrade(TEAMS[a as usize], UPGRADES[b as usize]);
            0
        }
        14 => {
            let Some(o) = choose(&live_of(sim, "ai"), a) else {
                return -1;
            };
            with_ai(sim, o, |ai| ai.idle = Bool::from(b != 0)).expect("ai");
            let rec = sim.world.objects.get_mut(o).expect("object");
            rec.ai_last_command_source = if c != 0 {
                CommandSourceType::FromPlayer
            } else {
                CommandSourceType::FromAi
            };
            rec.id.0
        }
        15 => {
            let x = 250.0 + (c % 100) as f32;
            let y = 250.0 + (c / 100) as f32;
            let o = spawn(sim, EXTRA[a as usize], b as usize, x, y, 0.0);
            id_of(sim, o)
        }
        _ => panic!("op {code}"),
    }
}

fn numbers(text: &str) -> Vec<u32> {
    text.split(',')
        .filter(|t| !t.is_empty())
        .map(|t| t.trim().parse().expect("number"))
        .collect()
}

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let from = line.find(start).expect(start) + start.len();
    let to = line[from..].find(end).expect(end) + from;
    &line[from..to]
}

/// The first differing section and, inside the objects, the first differing entry.
fn explain(got: &str, want: &str) -> String {
    const NAMES: [&str; 8] = [
        "return/frame",
        "objects",
        "players",
        "orders",
        "paths",
        "prone",
        "texts",
        "rng",
    ];
    for (i, (g, w)) in got.split('@').zip(want.split('@')).enumerate() {
        if g == w {
            continue;
        }
        let sep = if i == 1 { '^' } else { ';' };
        let gs: Vec<&str> = g.split(sep).collect();
        let ws: Vec<&str> = w.split(sep).collect();
        let k = gs
            .iter()
            .zip(&ws)
            .position(|(a, b)| a != b)
            .unwrap_or(gs.len().min(ws.len()));
        return format!(
            "section {} entry {k}\n  got  {}\n  want {}",
            NAMES.get(i).copied().unwrap_or("?"),
            gs.get(k).copied().unwrap_or("<none>"),
            ws.get(k).copied().unwrap_or("<none>")
        );
    }
    "equal".to_owned()
}

// ---- the second set ----------------------------------------------------------------

const JETS: [&str; 2] = ["J2_Jet0", "J2_Jet1"];

fn extra_row(sim: &mut SimContext) -> String {
    let mut s = String::new();
    for (id, o) in objects_by_id(sim) {
        let max = sim.object_max_health(o);
        let scalar = match sim.world.objects.get(o).expect("object").body_slot {
            Some(body) => sim
                .dispatch_module(body, |m, _, _| {
                    m.as_body_mut().expect("body").get_damage_scalar()
                })
                .expect("dispatch"),
            None => 1.0,
        };
        let ignored = with_ai(sim, o, |ai| ai.ignored.0).unwrap_or(0);
        let rec = sim.world.objects.get(o).expect("object");
        // `getDisabledUntil` reports a type only while its bit is set.
        let until = |t: DisabledType| {
            if rec.disabled_mask.test(t.as_i32()) != 0 {
                rec.disabled_till_frame[t as usize]
            } else {
                0
            }
        };
        let _ = write!(
            s,
            "{id}:{},{},{},{},{},{},{},{ignored};",
            max.to_bits(),
            scalar.to_bits(),
            rec.vision_range.to_bits(),
            rec.shroud_clearing_range.to_bits(),
            rec.weapon_bonus_condition,
            until(DisabledType::Paralyzed),
            until(DisabledType::Hacked),
        );
    }
    s.push('|');
    for (player, team) in TEAMS.into_iter().enumerate() {
        let chose = sim
            .world.players.player(sim.player_index(team)).map(|p| &p.academy)
            .is_some_and(|a| a.chose_a_strategy_for_center != FALSE);
        let plans = sim
            .player_battle_plans(player as i32)
            .cloned()
            .unwrap_or_default();
        let _ = write!(
            s,
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
        s.push(';');
    }
    s.push('|');
    for (_, name, _, pos) in std::mem::take(&mut sim.world.weapon_fires) {
        assert_eq!(name, "J2_Det");
        let _ = write!(
            s,
            "{},{},{};",
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits()
        );
    }
    s
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len() / 2)
        .map(|i| u8::from_str_radix(&text[i * 2..i * 2 + 2], 16).expect("hex"))
        .collect()
}

/// The save of a module in the reference's form (see `object_row`).
fn reference_save(sim: &SimContext, payload: &mut dyn BehaviorModule, spawn: bool) -> Vec<u8> {
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    let mut bytes = xs.as_bytes().to_vec();
    if spawn {
        bytes[0] = 2;
        bytes.drain(5..9);
    }
    bytes
}

#[derive(Default)]
struct Loads {
    count: BTreeMap<String, usize>,
    failures: BTreeMap<String, String>,
    /// Bytes still to be loaded into newly built objects: template, tag, bytes, the
    /// reference's save afterwards, where.
    pending: Vec<(String, String, String, String, String)>,
}

/// The harness empties the lists a load requires empty; do the same.
fn empty_lists(payload: &mut dyn BehaviorModule) {
    let Some(any) = payload.as_any_mut() else {
        return;
    };
    if let Some(m) = any.downcast_mut::<ProductionUpdate>() {
        m.queue.clear();
    } else if let Some(m) = any.downcast_mut::<SpawnBehavior>() {
        m.spawn_ids.clear();
    } else if let Some(m) = any.downcast_mut::<SpecialAbilityUpdate>() {
        m.special_object_ids.clear();
    }
}

/// Loads `bytes` into the module in `ms`, post-processes it and compares what it saves.
fn load_into(
    sim: &mut SimContext,
    ms: ModuleSlot,
    spawn: bool,
    bytes: &str,
    after: &str,
) -> Result<(), String> {
    let want = unhex(bytes);
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).expect("payload");
    empty_lists(payload.as_mut());
    let mut load = XferLoad::new(&want);
    let result = payload.xfer(&mut load, sim);
    let used = load.position();
    sim.world.modules.restore_payload(ms, generation, payload);
    if result.is_err() || used != want.len() {
        return Err(format!(
            "{result:?}, read {used} of {}\n  {bytes}",
            want.len()
        ));
    }
    sim.dispatch_module(ms, |m, s, c| m.load_post_process(s, c))
        .expect("post process");
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).expect("payload");
    let again = hex(&reference_save(sim, payload.as_mut(), spawn));
    sim.world.modules.restore_payload(ms, generation, payload);
    if again != after {
        return Err(format!("loaded {bytes}\n  got  {again}\n  want {after}"));
    }
    Ok(())
}

/// One saved module: load the reference bytes into the live module, which then runs on.
fn load_module(sim: &mut SimContext, entry: &str, loads: &mut Loads, at: &str) {
    let (id, rest) = entry.split_once(':').expect("id");
    let (tag, rest) = rest.split_once('=').expect("tag");
    let (bytes, after) = rest.split_once('>').expect("after");
    let o = objects_by_id(sim)
        .into_iter()
        .find(|&(i, _)| i.to_string() == id)
        .expect("object")
        .1;
    let ms = module_tagged(sim, o, &format!("ModuleTag_{tag}")).expect("module");
    *loads.count.entry(tag.to_owned()).or_default() += 1;
    loads.pending.push((
        template_name(sim, o),
        tag.to_owned(),
        bytes.to_owned(),
        after.to_owned(),
        at.to_owned(),
    ));
    if let Err(why) = load_into(sim, ms, tag == "Spawn", bytes, after) {
        loads
            .failures
            .entry(format!("{tag} live"))
            .or_insert(format!("{at}: {why}"));
    }
}

/// The same bytes into the module of a newly built object, as a game load would. Run
/// once the vector is over: building objects changes the world.
fn load_new(sim: &mut SimContext, loads: &mut Loads) {
    for (template, tag, bytes, after, at) in std::mem::take(&mut loads.pending) {
        let o = spawn(sim, &template, 1, 600.0, 600.0, 0.0);
        let ms = module_tagged(sim, o, &format!("ModuleTag_{tag}")).expect("module");
        if let Err(why) = load_into(sim, ms, tag == "Spawn", &bytes, &after) {
            loads
                .failures
                .entry(format!("{tag} new"))
                .or_insert(format!("{at}: {why}"));
        }
        sim.destroy_object(o);
        sim.process_destroy_list();
    }
}

/// The harness's `Scene2::apply`.
fn apply2(sim: &mut SimContext, templates: &[&str], op: [u32; 4]) -> i32 {
    let [code, a, b, c] = op;
    match code {
        9 => {
            let Some(o) = choose(&live(sim), a) else {
                return -1;
            };
            let id = id_of(sim, o);
            let mut d = DamageInfo::default();
            d.input.damage_type = if c != 0 {
                DamageType::Crush
            } else {
                DamageType::Unresistable
            };
            d.input.death_type = if c != 0 {
                DeathType::Crushed
            } else {
                DeathType::Normal
            };
            d.input.amount = b as f32;
            sim.attempt_damage(o, &mut d);
            id
        }
        16 => choose(&live(sim), a).map_or(-1, |o| id_of(sim, o)),
        17 => {
            let (Some(o), Some(t)) = (choose(&live_of(sim, "physics"), a), choose(&live(sim), b))
            else {
                return -1;
            };
            let k = c as f32 / 64.0;
            let to = *sim.world.objects.get(t).expect("t").thing.get_position();
            let from = *sim.world.objects.get(o).expect("o").thing.get_position();
            let f = Coord3D::new((to.x - from.x) * k, (to.y - from.y) * k, 0.0);
            sim.physics_apply_force(o, &f);
            id_of(sim, o)
        }
        18 | 19 => {
            let tag = format!("ModuleTag_Sp{}", if code == 18 { c } else { 0 });
            let with: Vec<ObjectSlot> = live(sim)
                .into_iter()
                .filter(|&o| module_tagged(sim, o, &tag).is_some())
                .collect();
            let Some(o) = choose(&with, a) else {
                return -1;
            };
            let ms = module_tagged(sim, o, &tag).expect("power");
            if code == 19 {
                sim.with_special_power_module(ms, &mut |p, s, ctx| p.do_special_power(s, ctx, b));
            } else {
                let Some(t) = choose(&live(sim), b) else {
                    return -1;
                };
                if t == o {
                    return -1;
                }
                sim.with_special_power_module(ms, &mut |p, s, ctx| {
                    p.do_special_power_at_object(s, ctx, t, 0);
                });
            }
            id_of(sim, o)
        }
        20 => {
            let Some(near) = choose(&live(sim), c) else {
                return -1;
            };
            let p = *sim
                .world
                .objects
                .get(near)
                .expect("near")
                .thing
                .get_position();
            let x = p.x + (c % 17) as f32 - 8.0;
            let y = p.y + (c % 11) as f32 - 5.0;
            let o = spawn(sim, templates[a as usize], b as usize, x, y, 0.5);
            id_of(sim, o)
        }
        21 => {
            let (Some(o), Some(t)) = (choose(&live_of(sim, "physics"), a), choose(&live(sim), b))
            else {
                return -1;
            };
            let mut p = *sim.world.objects.get(t).expect("t").thing.get_position();
            p.x += (c % 7) as f32 - 3.0;
            p.y += (c / 7 % 7) as f32 - 3.0;
            sim.set_object_position(o, &p);
            id_of(sim, o)
        }
        22 => {
            let parks: Vec<ObjectSlot> = live(sim)
                .into_iter()
                .filter(|&o| module_tagged(sim, o, "ModuleTag_Park").is_some())
                .collect();
            let Some(f) = choose(&parks, a) else {
                return -1;
            };
            let tmpl = sim
                .rules
                .thing_factory
                .find_template_id(JETS[b as usize])
                .expect("jet");
            with_production(sim, f, |p, s, ctx| {
                let id = p.request_unique_unit_id();
                i32::from(p.queue_create_unit(s, ctx, tmpl, id) != FALSE)
            })
        }
        _ => apply(sim, op),
    }
}

#[test]
#[cfg(not(any(feature = "retail-crc", feature = "retail-xfer-save")))]
fn second_joined_modules_match_reference() {
    const COUNT: usize = 1400;
    const FAMILIES: [&str; 7] = [
        "mines", "topple", "plan", "ability", "horde", "parking", "first",
    ];
    let in_hex = std::env::var_os("JOINS_HEX").is_some();
    let only: Option<usize> = std::env::var("JOINS_ONLY")
        .ok()
        .and_then(|v| v.parse().ok());
    let lines: Vec<&str> = vectors()
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"v\""))
        .collect();
    assert_eq!(lines.len(), COUNT);
    let names = text_field("templates");
    let templates: Vec<&str> = names.split_whitespace().collect();
    let mut sim = SimContext::new();
    sim.rules = rules();
    let mut checked = 0;
    let mut failed = [0usize; 7];
    let mut first: [Option<String>; 7] = Default::default();
    let mut loads = Loads::default();
    for (v, line) in lines.iter().enumerate() {
        if only.is_some_and(|o| o != v) {
            continue;
        }
        let family = v % 7;
        let seed = get_u32(line, "seed").expect("seed");
        let layout = numbers(between(line, "\"layout\":[", "]"));
        let ops = numbers(between(line, "\"ops\":[", "]"));
        let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
            .split("\",\"")
            .collect();
        let mut saved = between(line, "\"loads\":[\"", "\"],\"rows\"").split("\",\"");
        assert_eq!(ops.len(), (rows.len() - 1) * 4, "vector {v}");

        world(&mut sim);
        sim.world.rng.init_random_with_seed(seed);
        ORDERS.with(|o| o.borrow_mut().clear());
        for e in layout.chunks(6) {
            let name = templates[e[0] as usize];
            let (x, y) = (e[2] as f32, e[3] as f32);
            let o = spawn(&mut sim, name, e[1] as usize, x, y, ANGLES[e[5] as usize]);
            if e[4] != 0 {
                sim.set_object_position(o, &Coord3D::new(x, y, e[4] as f32));
            }
        }

        let mut run = Run {
            full: in_hex || v < FULL_VECTORS,
            in_hex,
            last: Vec::new(),
            last_players: String::new(),
        };
        for (k, want) in rows.iter().enumerate() {
            let (ret, op) = if k == 0 {
                (0, [99; 4])
            } else {
                let op: [u32; 4] = ops[(k - 1) * 4..k * 4].try_into().expect("op");
                let ret = apply2(&mut sim, &templates, op);
                if op[0] == 16 && ret != -1 {
                    let at = format!("vector {v} row {k}");
                    for entry in saved.next().expect("loads").split_whitespace() {
                        load_module(&mut sim, entry, &mut loads, &at);
                    }
                }
                (ret, op)
            };
            let base = run.row(&mut sim, ret);
            let extra = extra_row(&mut sim);
            let extra = if run.full { extra } else { digest(&extra) };
            let got = format!("{base}@{extra}");
            if got != *want {
                if only.is_some() {
                    println!("DBG{got}\nDBW{want}");
                }
                failed[family] += 1;
                first[family].get_or_insert_with(|| {
                    let tail = |s: &str| s.rsplit('@').next().unwrap_or("").to_owned();
                    let mut why = explain(&got, want);
                    if why.starts_with("section ?") {
                        why = format!("extra\n  got  {}\n  want {}", tail(&got), tail(want));
                    }
                    format!("vector {v} row {k} op {op:?}\n{why}")
                });
                break;
            }
            checked += 1;
        }
        load_new(&mut sim, &mut loads);
    }
    let mut report = String::new();
    for (f, name) in FAMILIES.iter().enumerate() {
        if failed[f] != 0 {
            let _ = writeln!(
                report,
                "{name}: {} vectors differ; {}",
                failed[f],
                first[f].as_deref().unwrap_or("")
            );
        }
    }
    for (name, why) in &loads.failures {
        let _ = writeln!(report, "load {name}: {why}");
    }
    assert!(report.is_empty(), "{checked} rows agree\n{report}");
    if only.is_none() {
        assert!(checked > 30_000, "only {checked} rows");
        assert!(loads.count.len() >= 27, "{:?}", loads.count);
    }
}
