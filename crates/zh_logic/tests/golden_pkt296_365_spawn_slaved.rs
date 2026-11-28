//! Golden vectors for `SpawnBehavior` (`PKT-296`) and `SlavedUpdate` (`PKT-365`),
//! recorded by `tests/cpp_harness/gen_pkt296_365_spawn_slaved.cpp`.
//!
//! The scene, the operations and the row format are those of the harness. The AI
//! stand-in records the orders as the harness AI does; every other module is real.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, VeterancyLevel,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::modules::behavior::spawn_behavior::{SpawnBehavior, SpawnBehaviorModuleData};
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CommandSourceType, ModuleData, ModuleType, SpawnBehaviorFacet,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::update::slaved_update::SlavedUpdate;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon_set::CanAttackResult;
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{AttackAbility, RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT296_365_SpawnSlaved.json");

/// Teams of players 0 (neutral), 1 and 2, as in the harness.
const TEAMS: [TeamId; 3] = [TeamId(1), TeamId(2), TeamId(3)];
const ATTACK_NEW_TARGET: UnsignedInt = 0;

// -----------------------------------------------------------------------------
// Recording AI stand-in
// -----------------------------------------------------------------------------

thread_local! {
    static ORDERS: RefCell<Vec<String>> = const { RefCell::new(Vec::new()) }; // determinism-ok: test log
}

fn log(order: String) {
    ORDERS.with(|o| o.borrow_mut().push(order));
}

#[derive(Debug, Clone)]
struct AiData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
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
    path: Option<Coord3D>,
}

fn bits3(c: &Coord3D) -> String {
    format!("{} {} {}", c.x.to_bits(), c.y.to_bits(), c.z.to_bits())
}

impl AiUpdateFacet for Ai {
    fn is_idle(&self) -> Bool {
        self.idle
    }
    fn get_path_destination(&self) -> Option<Coord3D> {
        self.path
    }
    fn destroy_path(&mut self) {
        self.path = None;
    }
    fn ai_idle(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, src: CommandSourceType) {
        log(format!("{} idle {}", self.id.0, src as i32));
    }
    fn ai_move_to_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        pos: &Coord3D,
        src: CommandSourceType,
    ) {
        log(format!("{} move {} {}", self.id.0, src as i32, bits3(pos)));
    }
    fn ai_force_attack_object(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        victim: ObjectSlot,
        max_shots: Int,
        src: CommandSourceType,
    ) {
        let victim = sim.object(victim).map_or(0, |o| o.id.0);
        log(format!(
            "{} force {} {victim} {max_shots}",
            self.id.0, src as i32
        ));
    }
    fn ai_attack_position(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        pos: &Coord3D,
        max_shots: Int,
        src: CommandSourceType,
    ) {
        log(format!(
            "{} attackpos {} {} {max_shots}",
            self.id.0,
            src as i32,
            bits3(pos)
        ));
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.id = ctx.owner_id;
        sim.ai_construct(ctx.owner_slot);
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
    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }
}

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(AiData {
        tag_key,
        module_name_key,
    }))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let data = data.as_any().downcast_ref::<AiData>().expect("ai").clone();
    Box::new(Ai {
        data,
        id: ObjectId::INVALID,
        idle: TRUE,
        path: None,
    })
}

// -----------------------------------------------------------------------------
// World
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
}

fn rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "OracleAI",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    let locomotors = get_str(VECTORS, "locomotors")
        .expect("locomotors")
        .replace('~', "\n");
    for block in locomotors.split_inclusive("End\n") {
        let mut t = LocomotorTemplate::new();
        t.parse_ini_block(block).expect("locomotor");
        t.name = block.split_whitespace().nth(1).expect("name").to_owned();
        rules.locomotor_store.add_template(t);
    }
    let ini = get_str(VECTORS, "objects")
        .expect("objects")
        .replace('~', "\n");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    rules
}

fn reset_world(sim: &mut SimContext) {
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
            w.set_player_relation(i, j, rel);
        }
    }
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D, angle: Real) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object(tid, team, pos, angle)
}

struct Scene {
    base: i32,
    master: i32,
    frame: u32,
    last: BTreeMap<i32, String>,
    victims: BTreeMap<i32, i32>,
}

fn find(sim: &SimContext, id: i32) -> Option<ObjectSlot> {
    sim.find_object(ObjectId(id))
}

fn scene_ids(sim: &SimContext, scene: &Scene) -> Vec<i32> {
    let mut ids: Vec<i32> = sim
        .world
        .objects
        .world_iter()
        .filter_map(|s| sim.object(s).map(|o| o.id.0))
        .filter(|&id| id >= scene.base)
        .collect();
    ids.sort_unstable();
    ids
}

/// The behaviors of an object with their module names, in module order.
fn modules(sim: &SimContext, o: ObjectSlot) -> Vec<(ModuleSlot, NameKeyType)> {
    sim.object(o).map_or_else(Vec::new, |rec| {
        rec.behaviors
            .iter()
            .filter_map(|&(ms, _)| {
                sim.world
                    .modules
                    .payload(ms)
                    .map(|m| (ms, m.module_name_key()))
            })
            .collect()
    })
}

fn module_named(sim: &mut SimContext, o: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.name_to_key(name);
    modules(sim, o)
        .into_iter()
        .find(|&(_, k)| k == key)
        .map(|(ms, _)| ms)
}

fn save_bytes(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

fn crc(sim: &mut SimContext, slot: ModuleSlot) -> u32 {
    let (payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    sim.world.modules.restore_payload(slot, generation, payload);
    xc.get_crc()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn object_row(sim: &mut SimContext, o: ObjectSlot) -> String {
    let mut out = String::new();
    let rec = sim.object(o).expect("object");
    let p = *rec.thing.get_position();
    let _ = write!(
        out,
        "{} {} {} {} {} ",
        rec.producer_id.0,
        sim.player_index(rec.team_id),
        bits3(&p),
        rec.thing.get_orientation().to_bits(),
        rec.layer as i32
    );
    use ObjectStatusType as S;
    for s in [
        S::Unselectable,
        S::Stealthed,
        S::CanStealth,
        S::Masked,
        S::NoAttack,
        S::Detected,
        S::UnderConstruction,
        S::Sold,
        S::Reconstructing,
    ] {
        out.push(if rec.status.test(s.as_i32()) != FALSE {
            '1'
        } else {
            '0'
        });
    }
    out.push(' ');
    use DisabledType as D;
    for d in [D::Held, D::Unmanned, D::Emp, D::Hacked, D::Subdued] {
        out.push(if rec.is_disabled_by_type(d) != FALSE {
            '1'
        } else {
            '0'
        });
    }
    let dead = rec.is_effectively_dead() != FALSE;
    let _ = write!(
        out,
        " {} {} {} {} {} ",
        sim.object_veterancy_level(o) as i32,
        sim.object_health(o).to_bits(),
        sim.object_max_health(o).to_bits(),
        dead as u32,
        bits3(&rec.health_box_offset)
    );
    use ModelConditionFlagType as M;
    for m in [
        M::Packing,
        M::Unpacking,
        M::FiringB,
        M::FiringC,
        M::BetweenFiringShotsB,
        M::BetweenFiringShotsC,
        M::ReloadingB,
        M::ReloadingC,
        M::Prone,
    ] {
        out.push(if rec.model_condition.test(m.as_i32()) != FALSE {
            '1'
        } else {
            '0'
        });
    }
    // `WEAPONBONUSCONDITION_DRONE_SPOTTING`
    let _ = write!(out, " {}", (rec.weapon_bonus_condition >> 6) & 1);
    match (&rec.ai_locomotor, rec.ai_locomotor_set) {
        (Some(loco), Some(set)) => {
            let _ = write!(
                out,
                " {} {} {}",
                set as i32,
                (loco.flags >> 3) & 1,
                (loco.flags >> 6) & 1
            );
        }
        _ => out += " - - -",
    }
    for name in [
        "SpawnBehavior",
        "SlavedUpdate",
        "ProneUpdate",
        "StealthUpdate",
    ] {
        match module_named(sim, o, name) {
            Some(ms) => {
                let mut bytes = save_bytes(sim, ms);
                // The reference saves version 2: no burst countdown, which follows
                // the three module versions and a flag in version 3.
                let countdown = (name == "SpawnBehavior").then(|| {
                    assert_eq!(bytes[0], 3);
                    bytes[0] = 2;
                    let field: Vec<u8> = bytes.drain(5..9).collect();
                    u32::from_le_bytes(field.try_into().expect("four bytes"))
                });
                let _ = write!(out, " {} {}", hex(&bytes), crc(sim, ms));
                if let Some(countdown) = countdown {
                    let _ = write!(out, " {countdown}");
                }
            }
            None => out += " -",
        }
    }
    out
}

/// The orders since the last row, with `aiGoProne` (which the stand-in is not told
/// about) in its place among them.
fn take_orders(sim: &mut SimContext) -> Vec<String> {
    let mut detailed = ORDERS
        .with(|o| std::mem::take(&mut *o.borrow_mut()))
        .into_iter();
    let out = std::mem::take(&mut sim.world.ai_commands)
        .into_iter()
        // The production exits have their own vectors.
        .filter(|(_, name)| name != "aiFollowExitProductionPath")
        .map(|(id, name)| {
            if name == "aiGoProne" {
                format!("{} prone", id.0)
            } else {
                detailed
                    .next()
                    .unwrap_or_else(|| panic!("no detail for {name} of {}", id.0))
            }
        })
        .collect();
    assert!(detailed.next().is_none(), "unrecorded order");
    out
}

fn row(sim: &mut SimContext, scene: &mut Scene, query: &str) -> String {
    let mut parts = Vec::new();
    let ids = scene_ids(sim, scene);
    for &id in &ids {
        let o = find(sim, id).expect("listed object");
        let now = object_row(sim, o);
        if scene.last.get(&id) != Some(&now) {
            parts.push(format!("O {id} {now}"));
            scene.last.insert(id, now);
        }
    }
    let gone: Vec<i32> = scene
        .last
        .keys()
        .copied()
        .filter(|id| ids.binary_search(id).is_err())
        .collect();
    for id in gone {
        parts.push(format!("X {id}"));
        scene.last.remove(&id);
    }
    parts.extend(take_orders(sim).into_iter().map(|c| format!("C {c}")));
    if !query.is_empty() {
        parts.push(format!("Q {query}"));
    }
    let mut tail = format!("R {}", sim.rng_logic_int(0, 1_000_000));
    for player in 0..3 {
        let built = sim
            .world.players.player(player).map(|p| &p.score_keeper)
            .map_or(0, |k| k.total_units_built);
        let _ = write!(tail, " {built}");
    }
    parts.push(tail);
    parts.join(";")
}

/// `AIUpdateInterface::getCurrentVictim` finds the object by id at each call.
fn refresh_victims(sim: &mut SimContext, scene: &Scene) {
    for (&who, &victim) in &scene.victims {
        let victim = find(sim, victim);
        if let Some(o) = find(sim, who) {
            sim.object_mut(o).expect("object").ai_current_victim = victim;
        }
    }
}

fn tick(sim: &mut SimContext, scene: &mut Scene) {
    scene.frame += 1;
    sim.world.frame = scene.frame;
    let updates = [
        "SpawnBehavior",
        "SlavedUpdate",
        "ProneUpdate",
        "QueueProductionExitUpdate",
    ]
    .map(|n| sim.rules.name_keys.name_to_key(n));
    for id in scene_ids(sim, scene) {
        let Some(o) = find(sim, id) else {
            continue;
        };
        if sim.object(o).is_none_or(|r| r.is_destroyed()) {
            continue;
        }
        for (ms, key) in modules(sim, o) {
            if !updates.contains(&key) {
                continue;
            }
            refresh_victims(sim, scene);
            let Some(rec) = sim.object(o) else {
                continue;
            };
            if rec.is_destroyed() {
                continue;
            }
            let disabled = rec.disabled_mask.clone();
            let is_disabled = rec.is_disabled() != FALSE;
            let _ = sim.dispatch_module(ms, |m, s, c| {
                let update = m.as_update_mut().expect("update");
                let mine = update.disabled_types_to_process();
                let runs = !is_disabled
                    || (0..32).all(|bit| disabled.test(bit) == FALSE || mine.test(bit) != FALSE);
                if runs {
                    let _ = update.update(s, c);
                }
            });
        }
    }
    sim.process_destroy_list();
}

fn on_spawn_behavior(
    sim: &mut SimContext,
    master: ObjectSlot,
    f: impl FnOnce(&mut dyn SpawnBehaviorFacet, &mut dyn SimApi, ModuleContext),
) {
    let ms = sim
        .object(master)
        .and_then(|o| o.spawn_slot)
        .expect("spawn behavior");
    sim.dispatch_module(ms, |m, s, c| {
        f(m.as_spawn_behavior_mut().expect("facet"), s, c)
    })
    .expect("dispatch");
}

fn ai_mut(sim: &mut SimContext, o: ObjectSlot, f: impl FnOnce(&mut Ai)) -> bool {
    let Some(ms) = sim.object(o).and_then(|r| r.ai_slot) else {
        return false;
    };
    sim.dispatch_module(ms, |m, _, _| {
        f(m.as_any_mut()
            .and_then(|a| a.downcast_mut::<Ai>())
            .expect("ai"));
    })
    .expect("dispatch");
    true
}

fn source(v: i64) -> CommandSourceType {
    match v {
        0 => CommandSourceType::FromPlayer,
        1 => CommandSourceType::FromScript,
        2 => CommandSourceType::FromAi,
        _ => panic!("command source {v}"),
    }
}

fn status(v: i64) -> ObjectStatusType {
    use ObjectStatusType as S;
    [
        S::UnderConstruction,
        S::Sold,
        S::Reconstructing,
        S::Stealthed,
    ]
    .into_iter()
    .find(|s| i64::from(s.as_i32()) == v)
    .expect("status")
}

fn disabled(v: i64) -> DisabledType {
    use DisabledType as D;
    [D::Emp, D::Hacked, D::Subdued, D::Held, D::Unmanned]
        .into_iter()
        .find(|d| i64::from(d.as_i32()) == v)
        .expect("disabled type")
}

fn attack_result(v: i64) -> CanAttackResult {
    match v {
        0 => CanAttackResult::NotPossible,
        1 => CanAttackResult::InvalidShot,
        2 => CanAttackResult::PossibleAfterMoving,
        3 => CanAttackResult::Possible,
        _ => panic!("attack result {v}"),
    }
}

fn layer(v: i64) -> PathfindLayerEnum {
    use PathfindLayerEnum as L;
    match v {
        1 => L::Ground,
        2 => L::Bridge1,
        3 => L::Bridge2,
        4 => L::Bridge3,
        _ => panic!("layer {v}"),
    }
}

fn real(v: i64) -> Real {
    Real::from_bits(v as u32)
}

fn apply(sim: &mut SimContext, scene: &mut Scene, op: &[i64]) -> String {
    let (code, a, b, c, d, e) = (op[0], op[1], op[2], op[3], op[4], op[5]);
    sim.world.frame = scene.frame;
    refresh_victims(sim, scene);
    let target = find(sim, a as i32);
    let mut query = String::new();
    match code {
        0 => {
            for _ in 0..a {
                tick(sim, scene);
            }
        }
        1 => {
            if let Some(t) = target {
                let mut dmg = DamageInfo::default();
                dmg.input.amount = real(b);
                dmg.input.damage_type = DamageType::from_i32(c as Int).expect("damage type");
                dmg.input.source_id = ObjectId(d as i32);
                dmg.input.death_type = DeathType::from_i32(e as Int).expect("death type");
                sim.attempt_damage(t, &mut dmg);
                query = format!(
                    "damage {} {}",
                    dmg.output.actual_damage_dealt.to_bits(),
                    dmg.output.actual_damage_clipped.to_bits()
                );
            }
        }
        2 => {
            if let Some(t) = target {
                sim.destroy_object(t);
            }
        }
        3 => {
            if let Some(t) = target {
                sim.set_object_position(t, &Coord3D::new(real(b), real(c), real(d)));
                if e != 0 {
                    sim.set_object_layer(t, layer(e));
                }
            }
        }
        4 => {
            if let Some(t) = target {
                if ai_mut(sim, t, |ai| ai.idle = Bool::from(b != 0)) {
                    sim.object_mut(t).expect("object").ai_last_command_source = source(c);
                    let victim = find(sim, d as i32).map_or(0, |_| d as i32);
                    scene.victims.insert(a as i32, victim);
                    let slot = find(sim, victim);
                    sim.object_mut(t).expect("object").ai_current_victim = slot;
                }
            }
        }
        5 => {
            if let Some(t) = target {
                let destination = (b != 0).then(|| Coord3D::new(real(c), real(d), 0.0));
                ai_mut(sim, t, |ai| ai.path = destination);
            }
        }
        6 => {
            if let Some(t) = target {
                if c != 0 {
                    sim.set_status(t, status(b));
                } else {
                    sim.clear_status(t, status(b));
                }
            }
        }
        7 => {
            if let Some(t) = target {
                sim.set_object_team(t, TEAMS[b as usize]);
            }
        }
        8 => {
            if let Some(t) = target {
                if c != 0 {
                    sim.set_disabled_until(t, disabled(b), d as UnsignedInt);
                } else {
                    sim.clear_disabled(t, disabled(b));
                }
            }
        }
        9 => {
            if let Some(t) = target {
                let level = [
                    VeterancyLevel::Regular,
                    VeterancyLevel::Veteran,
                    VeterancyLevel::Elite,
                    VeterancyLevel::Heroic,
                ][b as usize];
                sim.set_veterancy_level(t, level, FALSE, TRUE);
            }
        }
        10..=14 => {
            let Some(master) = find(sim, scene.master) else {
                sim.process_destroy_list();
                return query;
            };
            match code {
                10 => {
                    let pos = Coord3D::new(real(a), real(b), 0.0);
                    let enemy = find(sim, d as i32).expect("query target");
                    for ability in op[6..].chunks_exact(4) {
                        sim.world.attack_abilities.insert(
                            ObjectId(ability[0] as i32),
                            AttackAbility {
                                able: Bool::from(ability[1] != 0),
                                specific_object: attack_result(ability[2]),
                                weapon_against_target: attack_result(ability[3]),
                            },
                        );
                    }
                    on_spawn_behavior(sim, master, |sb, s, ctx| {
                        let closest = sb
                            .get_closest_slave(s, &pos)
                            .and_then(|o| s.object(o))
                            .map_or(0, |o| o.id.0);
                        let src = CommandSourceType::FromPlayer;
                        query = format!(
                            "spawn {} {} {closest} {} {} {} {}",
                            sb.get_slave_count(),
                            u32::from(sb.do_slaves_have_freedom() != FALSE),
                            u32::from(sb.may_spawn_self_task_ai(s, ctx, real(c)) != FALSE),
                            u32::from(sb.can_any_slaves_attack(s) != FALSE),
                            sb.get_can_any_slaves_attack_specific_target(
                                s,
                                ATTACK_NEW_TARGET,
                                enemy,
                                src
                            ) as i32,
                            sb.get_can_any_slaves_use_weapon_against_target(
                                s,
                                ATTACK_NEW_TARGET,
                                Some(enemy),
                                &pos,
                                src
                            ) as i32,
                        );
                        let stealthed = sb.are_all_slaves_stealthed(s);
                        let _ = write!(query, " {}", u32::from(stealthed != FALSE));
                    });
                }
                11 => {
                    let enemy = find(sim, b as i32);
                    on_spawn_behavior(sim, master, |sb, s, ctx| match (a, enemy) {
                        (0, Some(t)) => {
                            sb.order_slaves_to_attack_target(s, ctx, t, c as Int, source(d));
                        }
                        (1, Some(t)) => {
                            let pos = *s.object(t).expect("target").thing.get_position();
                            sb.order_slaves_to_attack_position(s, ctx, &pos, c as Int, source(d));
                        }
                        (2, _) => sb.order_slaves_to_go_idle(s, ctx, source(d)),
                        _ => {}
                    });
                }
                12 => on_spawn_behavior(sim, master, |sb, s, _| {
                    sb.give_slaves_stealth_upgrade(s, Bool::from(a != 0));
                }),
                13 => {
                    if a != 0 {
                        for id in scene_ids(sim, scene) {
                            let o = find(sim, id).expect("object");
                            sim.receive_stealth_grant(o, TRUE, 0);
                        }
                    }
                    on_spawn_behavior(sim, master, |sb, s, _| sb.reveal_slaves(s));
                }
                _ => on_spawn_behavior(sim, master, |sb, _, _| {
                    if a != 0 {
                        sb.start_spawning();
                    } else {
                        sb.stop_spawning();
                    }
                }),
            }
        }
        15 | 16 => {
            if let Some(ms) = target.and_then(|t| sim.object(t).and_then(|o| o.slaved_slot)) {
                let slaver = find(sim, b as i32);
                sim.dispatch_module(ms, |m, s, ctx| {
                    let su = m.as_slaved_update_mut().expect("slaved");
                    if code == 15 {
                        su.on_enslave(s, ctx, slaver);
                    } else {
                        su.on_slaver_die(s, ctx, None);
                    }
                    query = format!(
                        "slaver {} {}",
                        su.get_slaver_id().0,
                        u32::from(su.is_self_tasking() != FALSE)
                    );
                })
                .expect("dispatch");
            }
        }
        17 => {
            if let Some(t) = target {
                sim.object_mut(t).expect("object").producer_id = ObjectId::INVALID;
            }
        }
        _ => panic!("op {code}"),
    }
    sim.process_destroy_list();
    query
}

fn build_scene(sim: &mut SimContext, line: &str) -> Scene {
    let field = |k: &str| get_u32(line, k).expect("field");
    let value = |k: &str| Real::from_bits(field(k));
    reset_world(sim);
    ORDERS.with(|o| o.borrow_mut().clear());
    sim.terrain = Box::new(Slope(field("slope") == 1));
    sim.world.frame = field("frame");
    sim.world.next_object_id = field("id") as i32;
    sim.world.rng.init_random_with_seed(field("seed"));
    // `barracks` and `orphaned` may be negative or zero; read them as signed text.
    let signed = |k: &str| -> i64 {
        let tail = line
            .split_once(&format!("\"{k}\":"))
            .expect("signed field")
            .1;
        tail.split([',', '}'])
            .next()
            .and_then(|v| v.parse().ok())
            .expect("number")
    };
    let ground = |x: Real, y: Real| {
        if field("slope") == 1 {
            x * 0.01 + y * 0.02
        } else {
            0.0
        }
    };
    let (mx, my, ex, ey) = (value("x"), value("y"), value("ex"), value("ey"));
    let enemy = spawn(
        sim,
        "P_Enemy",
        TEAMS[2],
        Coord3D::new(ex, ey, ground(ex, ey)),
        0.0,
    );
    let plane = spawn(
        sim,
        "P_Plane",
        TEAMS[2],
        Coord3D::new(value("px"), value("py"), 60.0),
        0.0,
    );
    let kind = signed("barracks");
    let barracks = (kind >= 0).then(|| {
        let x = mx - 40.0;
        spawn(
            sim,
            &format!("P_Barracks{kind}"),
            TEAMS[1],
            Coord3D::new(x, my, ground(x, my)),
            0.0,
        )
    });
    let name = get_str(line, "master").expect("master");
    let master = spawn(
        sim,
        name,
        TEAMS[field("team") as usize],
        Coord3D::new(mx, my, value("z")),
        value("angle"),
    );
    let id = |sim: &SimContext, o: ObjectSlot| sim.object(o).expect("object").id;
    let base = id(sim, enemy).0;
    let master_id = id(sim, master).0;
    if let Some(b) = barracks {
        let producer = id(sim, b);
        sim.object_mut(master).expect("master").producer_id = producer;
    }
    if signed("orphaned") != 0 {
        let producer = id(sim, plane);
        sim.object_mut(master).expect("master").producer_id = producer;
        sim.destroy_object(plane);
        sim.process_destroy_list();
    }
    Scene {
        base,
        master: master_id,
        frame: field("frame"),
        last: BTreeMap::new(),
        victims: BTreeMap::new(),
    }
}

/// Saving, loading into a fresh module and saving again gives the same bytes.
fn check_reload(sim: &mut SimContext, scene: &Scene) -> usize {
    let mut checked = 0;
    for id in scene_ids(sim, scene) {
        let o = find(sim, id).expect("object");
        for name in ["SpawnBehavior", "SlavedUpdate"] {
            let Some(ms) = module_named(sim, o, name) else {
                continue;
            };
            let bytes = save_bytes(sim, ms);
            let meta = sim.world.modules.meta(ms);
            let (data_id, ctx) = (meta.template_data_id, meta.context(ms));
            let descriptor = sim
                .rules
                .module_registry
                .find_descriptor(name, ModuleType::Behavior)
                .expect("descriptor")
                .clone();
            let data = sim.rules.module_data_store.get(data_id).expect("data");
            let mut fresh = (descriptor.create_module.expect("create"))(data, ctx);
            fresh.on_construct(sim, ctx);
            fresh.xfer(&mut XferLoad::new(&bytes), sim).expect("load");
            let mut again = XferSave::new();
            // The fresh module is not bound to the arena slot's live schedule.
            fresh.load_post_process(sim, ctx);
            fresh.xfer(&mut again, sim).expect("save");
            assert_eq!(hex(again.as_bytes()), hex(&bytes), "{name} of {id}");
            checked += 1;
        }
    }
    checked
}

#[test]
fn spawn_and_slaved_golden_vectors() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    let mut vectors = 0;
    let mut reloaded = 0;
    for line in VECTORS.lines().filter(|l| l.starts_with("{\"frame\"")) {
        let mut scene = build_scene(&mut sim, line);
        let first = row(&mut sim, &mut scene, "");
        assert_eq!(
            first,
            get_str(line, "spawn").expect("spawn"),
            "vector {vectors} at spawn"
        );
        let ops = get_str(line, "ops").expect("ops");
        let mut rows = get_str(line, "rows").expect("rows").split('|');
        for (k, op) in ops.split(';').enumerate() {
            let numbers: Vec<i64> = op.split(' ').map(|n| n.parse().expect("number")).collect();
            let query = apply(&mut sim, &mut scene, &numbers);
            let got = row(&mut sim, &mut scene, &query);
            let want = rows.next().expect("row");
            if got != want {
                let g: Vec<&str> = got.split(';').collect();
                let w: Vec<&str> = want.split(';').collect();
                let at = (0..g.len().max(w.len()))
                    .find(|&i| g.get(i) != w.get(i))
                    .unwrap_or(0);
                panic!(
                    "vector {vectors} op {k} ({op}) part {at}\n got {:?}\nwant {:?}",
                    g.get(at),
                    w.get(at)
                );
            }
        }
        if vectors % 7 == 0 {
            reloaded += check_reload(&mut sim, &scene);
        }
        vectors += 1;
    }
    assert_eq!(vectors, 1200);
    assert!(reloaded > 300, "reloaded {reloaded}");
}

// -----------------------------------------------------------------------------
// Paths the harness cannot observe
// -----------------------------------------------------------------------------

/// The values a template leaves out come from the data constructors.
#[test]
fn empty_blocks_keep_the_constructor_values() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    let ini = "Object T_Bare\n  Behavior = SpawnBehavior ModuleTag_Spawn\n    \
               SpawnTemplateName = P_S5\n  End\n  Behavior = SlavedUpdate ModuleTag_Slaved\n  \
               End\nEnd\n";
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    reset_world(&mut sim);
    let o = spawn(
        &mut sim,
        "T_Bare",
        TEAMS[1],
        Coord3D::new(1.0, 1.0, 0.0),
        0.0,
    );

    let ms = module_named(&mut sim, o, "SpawnBehavior").expect("spawn");
    sim.dispatch_module(ms, |m, _, _| {
        let module = m
            .as_any()
            .and_then(|a| a.downcast_ref::<SpawnBehavior>())
            .expect("spawn behavior");
        let mut want =
            SpawnBehaviorModuleData::new(module.data.tag_key, module.data.module_name_key);
        want.spawn_template_names = vec!["P_S5".to_owned()];
        assert_eq!(module.data, want);
        assert_eq!(
            (
                want.spawn_number,
                want.spawn_replace_delay,
                want.initial_burst
            ),
            (0, 0, 0)
        );
        assert_eq!(module.one_shot_countdown, -1);
        assert_eq!(module.spawn_count, -1);
    })
    .expect("dispatch");

    let ms = module_named(&mut sim, o, "SlavedUpdate").expect("slaved");
    sim.dispatch_module(ms, |m, _, _| {
        let module = m
            .as_any()
            .and_then(|a| a.downcast_ref::<SlavedUpdate>())
            .expect("slaved update");
        let d = &module.data;
        let ints = [
            d.guard_max_range,
            d.guard_wander_range,
            d.attack_range,
            d.attack_wander_range,
            d.scout_range,
            d.scout_wander_range,
            d.dist_to_target_to_grant_range_bonus,
            d.repair_range,
            d.repair_when_health_below_percentage,
            d.min_ready_frames,
            d.max_ready_frames,
            d.min_weld_frames,
            d.max_weld_frames,
        ];
        assert_eq!(ints, [0; 13]);
        let reals = [
            d.repair_min_altitude,
            d.repair_max_altitude,
            d.repair_rate_per_second,
        ];
        assert_eq!(reals, [0.0; 3]);
        assert!(d.welding_sys_name.is_empty() && d.welding_fx_bone.is_empty());
    })
    .expect("dispatch");
}

/// A scene with a master of the given template that has spawned its slaves.
fn populated(sim: &mut SimContext, master: &str) -> (ObjectSlot, ObjectSlot, Vec<ObjectSlot>) {
    reset_world(sim);
    sim.world.frame = 100;
    let enemy = spawn(
        sim,
        "P_Enemy",
        TEAMS[2],
        Coord3D::new(400.0, 400.0, 0.0),
        0.0,
    );
    let master = spawn(sim, master, TEAMS[1], Coord3D::new(500.0, 500.0, 0.0), 0.0);
    let mut scene = Scene {
        base: 0,
        master: sim.object(master).expect("master").id.0,
        frame: 100,
        last: BTreeMap::new(),
        victims: BTreeMap::new(),
    };
    for _ in 0..40 {
        tick(sim, &mut scene);
    }
    let id = sim.object(master).expect("master").id;
    let slaves = sim
        .world
        .objects
        .world_iter()
        .filter(|&s| sim.object(s).is_some_and(|o| o.producer_id == id))
        .collect();
    (enemy, master, slaves)
}

/// The last spawn of an aggregate-health master takes the master with it, and the
/// killer is credited (`scoreTheKill`).
#[test]
fn aggregate_master_dies_with_its_last_spawn() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    let (enemy, master, slaves) = populated(&mut sim, "P_M2_0");
    assert_eq!(slaves.len(), 4);
    let enemy_id = sim.object(enemy).expect("enemy").id;
    let master_id = sim.object(master).expect("master").id;
    for (i, &slave) in slaves.iter().enumerate() {
        assert!(sim.object(master).is_some_and(|m| !m.is_destroyed()));
        let mut dmg = DamageInfo::default();
        dmg.input.damage_type = DamageType::Unresistable;
        dmg.input.source_id = enemy_id;
        dmg.input.amount = 10_000.0;
        sim.attempt_damage(slave, &mut dmg);
        let credited = sim.world.kills_scored.contains(&(enemy_id, master_id));
        assert_eq!(credited, i + 1 == slaves.len());
    }
    assert!(sim.object(master).is_some_and(|m| m.is_destroyed()));
}

/// `StealthUpgrade` on an object whose spawns are its weapons upgrades them too.
#[test]
fn stealth_upgrade_reaches_the_spawns() {
    let hive = |name: &str, kind_of: &str| {
        format!(
            "Object {name}\n  KindOf = {kind_of}\n  Body = ActiveBody ModuleTag_Body\n    \
             MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = SpawnBehavior \
             ModuleTag_Spawn\n    SpawnNumber = 2\n    SpawnTemplateName = P_S5\n  End\n  \
             Behavior = DefaultProductionExitUpdate ModuleTag_Exit\n  End\n  Behavior = \
             StealthUpgrade ModuleTag_Upgrade\n    TriggeredBy = Upgrade_Camo\n  End\nEnd\n"
        )
    };
    let mut sim = SimContext::new();
    sim.rules = rules();
    let upgrade = "Upgrade Upgrade_Camo\nEnd\n";
    let mut r = IniReader::new("upgrade.ini", upgrade.as_bytes(), IniLoadType::Overwrite);
    r.load_blocks(None, |_, reader| {
        let _ = reader.next_token(SEPS_DEFAULT)?;
        sim.rules
            .upgrade_center
            .parse_upgrade_definition(reader, &mut sim.rules.name_keys)
    })
    .expect("upgrade");
    let ini = hive("T_Weapons", "SPAWNS_ARE_THE_WEAPONS") + &hive("T_Plain", "STRUCTURE");
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    sim.rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");

    for (name, weapons) in [("T_Weapons", true), ("T_Plain", false)] {
        let (_, master, slaves) = populated(&mut sim, name);
        assert_eq!(slaves.len(), 2);
        assert_eq!(
            sim.object_is_kind_of(master, KindOfType::SpawnsAreTheWeapons) != FALSE,
            weapons
        );
        let can = |sim: &SimContext, o: ObjectSlot| {
            sim.object(o)
                .is_some_and(|r| r.status.test(ObjectStatusType::CanStealth.as_i32()) != FALSE)
        };
        assert!(slaves.iter().all(|&s| !can(&sim, s)));
        sim.grant_upgrade(master, "Upgrade_Camo", FALSE);
        assert!(can(&sim, master));
        assert!(slaves.iter().all(|&s| can(&sim, s) == weapons));
    }
}
