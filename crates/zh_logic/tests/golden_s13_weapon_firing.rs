//! AI port slice S13 (PKT-399): range queries, approach target, line of sight questions,
//! damage estimate, firing from the weapon set with the firing tracker, scatter and infantry
//! inaccuracy, assistance, disarm and assault shots, reloads, bonus retimes and
//! `FireWeaponPower`. Vectors from `tests/cpp_harness/gen_s13_weapon_firing.cpp`.
//!
//! Every row is compared whole; the first vectors keep their text to locate a difference.

use std::any::Any;
use std::cell::RefCell;
use std::collections::BTreeSet;
use std::fmt::Write as _;

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Matrix3D, Region3D};
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferError, XferSave};

use zh_logic::ai::seams::{shared, AiSeams};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::gamelogic::global_data::WeaponBonusConditionType;
use zh_logic::modules::module::{AiUpdateFacet, BehaviorModule, CommandSourceType, ModuleData};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::modules::ModuleType;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::{Weapon, WeaponTemplate};
use zh_logic::object::weapon_set::{WeaponLockType, WeaponSlotType};
use zh_logic::object::{weapon_actions, weapon_effects, weapon_fire, weapon_queries};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::RulesRegistry;
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};

const GOLDEN: &str = include_str!("../../../docs/golden_vectors/S13_WeaponFiring.json");

/// The harness spawns on player A, player B or the neutral player.
const TEAMS: [TeamId; 3] = [TeamId(2), TeamId(3), TeamId(1)];
const ANGLES: [f32; 8] = [0.0, 0.5, 1.5707964, 2.5, 3.1415927, -1.0, -2.0, 4.5];
const OFFSETS: [f32; 4] = [0.0, 0.0, 0.7, -2.1];
const CONDITIONS: [WeaponBonusConditionType; 4] = [
    WeaponBonusConditionType::Garrisoned,
    WeaponBonusConditionType::Horde,
    WeaponBonusConditionType::ContinuousFireMean,
    WeaponBonusConditionType::ContinuousFireFast,
];
const SLOTS: [WeaponSlotType; 3] = [
    WeaponSlotType::Primary,
    WeaponSlotType::Secondary,
    WeaponSlotType::Tertiary,
];

thread_local! {
    static LOG: RefCell<String> = const { RefCell::new(String::new()) }; // determinism-ok: test log
}

fn log(entry: &str) {
    LOG.with(|l| l.borrow_mut().push_str(entry));
}

fn take_log() -> String {
    LOG.with(|l| std::mem::take(&mut *l.borrow_mut()))
}

// -----------------------------------------------------------------------------
// Recording AI, seams and terrain (the harness's overrides)
// -----------------------------------------------------------------------------

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
}

impl AiUpdateFacet for Ai {
    fn is_idle(&self) -> Bool {
        TRUE
    }
    fn ai_attack_object(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        target: ObjectSlot,
        max_shots: Int,
        src: CommandSourceType,
    ) {
        let target = sim.object(target).map_or(0, |o| o.id.0);
        log(&format!(
            "C{} attack {} {target} {max_shots};",
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
        log(&format!(
            "C{} attackpos {} {} {} {} {max_shots};",
            self.id.0,
            src as i32,
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits()
        ));
    }
}

impl BehaviorModule for Ai {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }
    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.id = ctx.owner_id;
        // The state machine's draws happen, but no order is acted on afterwards.
        sim.ai_init_state_machine(ctx.owner_slot);
        if let Some(o) = sim.object_mut(ctx.owner_slot) {
            o.ai_state_machine_ready = FALSE;
        }
    }
    fn crc(&self, _xfer: &mut dyn Xfer) {}
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
    })
}

#[derive(Default)]
struct Seams {
    sneaky: BTreeSet<ObjectId>,
}

impl AiSeams for Seams {
    fn sneaky_targeting_offset(&mut self, victim: ObjectId) -> Option<Coord3D> {
        self.sneaky
            .contains(&victim)
            .then(|| Coord3D::new(13.0, -7.5, 2.0))
    }
    fn begin_assault(&mut self, owner: ObjectId, victim: Option<ObjectId>) {
        log(&format!("B{},{};", owner.0, victim.map_or(0, |v| v.0)));
    }
}

struct Terrain;

impl ThingTerrainAccess for Terrain {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        ((x / 40.0) as i32 % 3) as Real * 4.0 + ((y / 60.0) as i32 % 4) as Real * 3.0
    }

    fn is_clear_line_of_sight(&self, a: &Coord3D, b: &Coord3D) -> Bool {
        let v = [a.x, a.y, a.z, b.x, b.y, b.z].map(Real::to_bits);
        let hash = v
            .iter()
            .fold(2_166_136_261_u32, |h, &u| (h ^ u).wrapping_mul(16_777_619));
        let clear = (hash >> 7) & 1;
        log(&format!(
            "L{},{},{},{},{},{}={clear};",
            v[0], v[1], v[2], v[3], v[4], v[5]
        ));
        Bool::from(clear != 0)
    }

    fn align_on_terrain(
        &self,
        angle: Real,
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
        _x: Real,
        _y: Real,
        _water_z: Option<&mut Real>,
        _terrain_z: Option<&mut Real>,
    ) -> Bool {
        FALSE
    }
}

// -----------------------------------------------------------------------------
// World
// -----------------------------------------------------------------------------

/// The text of a JSON string whose only escapes are `\n`, `\"` and `\\`.
fn header(key: &str) -> String {
    let start = format!("\"{key}\":");
    let line = GOLDEN
        .lines()
        .find(|l| l.starts_with(&start))
        .unwrap_or_else(|| panic!("missing {key}"));
    get_str(line, key)
        .unwrap_or_else(|| panic!("missing {key}"))
        .replace("\\n", "\n")
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
    let armor = header("armor_ini");
    let mut ini = IniReader::new("armor.ini", armor.as_bytes(), IniLoadType::Overwrite);
    rules
        .armor_store
        .load_from_ini(&mut ini, &mut rules.name_keys)
        .expect("armor");
    let power = "SpecialPower E_Power\n  Enum = SPECIAL_DAISY_CUTTER\nEnd\n";
    let mut ini = IniReader::new("power.ini", power.as_bytes(), IniLoadType::Overwrite);
    ini.load_blocks(None, |_, reader| {
        let _kw = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
        rules.special_power_store.parse_special_power_definition(
            reader,
            &rules.science_store,
            &mut rules.name_keys,
        )
    })
    .expect("special power");

    let mut block = String::new();
    for line in header("weapon_ini").lines() {
        block.push_str(line);
        block.push('\n');
        if line == "End" {
            let mut weapon = WeaponTemplate::new();
            weapon.parse_ini_block(&block).expect("weapon");
            rules.weapon_store.add_template(weapon);
            block.clear();
        }
    }
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    rules.global_data.gravity = -1.0;
    let objects = header("object_ini");
    let mut ini = IniReader::new("object.ini", objects.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("object ini line {}: {e:?}", ini.get_line_num());
    }
    rules
}

fn new_world(frame: u32, seed: u32) -> SimWorld {
    let mut world = SimWorld::new();
    world.frame = frame;
    world.rng.init_random_with_seed(seed);
    for (player, team) in [TeamId(1), TeamId(2), TeamId(3)].into_iter().enumerate() {
        world.player_default_teams.insert(player as i32, team);
        world.team_players.insert(team, player as i32);
        world.team_is_neutral.insert(team, Bool::from(player == 0));
    }
    world.neutral_team_id = TeamId(1);
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

/// The arrays of numbers of `key`: `[[..],..]`.
fn arrays(line: &str, key: &str) -> Vec<Vec<i64>> {
    let start = format!("\"{key}\":[[");
    let from = line.find(&start).unwrap_or_else(|| panic!("missing {key}")) + start.len();
    let to = from + line[from..].find("]]").expect("end of arrays");
    line[from..to]
        .split("],[")
        .map(|a| a.split(',').map(|n| n.parse().expect("number")).collect())
        .collect()
}

fn rows(line: &str) -> Vec<&str> {
    let start = line.find("\"rows\":[\"").expect("rows") + 9;
    let end = line.rfind("\"]}").expect("end of rows");
    line[start..end].split("\",\"").collect()
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, c| {
        (h ^ u64::from(c)).wrapping_mul(0x100_0000_01b3)
    })
}

fn live_objects(sim: &SimContext) -> Vec<ObjectSlot> {
    (1..200)
        .filter_map(|id| sim.world.objects.slot_by_id(ObjectId(id)))
        .collect()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().fold(String::new(), |mut s, b| {
        let _ = write!(s, "{b:02x}");
        s
    })
}

fn module_named(sim: &SimContext, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
    let key = sim.rules.name_keys.lookup_key(name)?;
    sim.object(owner)?
        .behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|m| m.module_name_key() == key)
        })
}

fn module_state(sim: &mut SimContext, slot: ModuleSlot) -> (Vec<u8>, u32) {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut save = XferSave::new();
    payload.xfer(&mut save, sim).expect("save");
    let mut crc = XferCrc::new();
    payload.crc(&mut crc);
    sim.world.modules.restore_payload(slot, generation, payload);
    (save.as_bytes().to_vec(), crc.get_crc())
}

/// The reference build saves version 1 of the tracker, which lacks its last two members
/// (the default build here writes version 2; `OPEN_DECISIONS` item 2). They are compared
/// from the version 2 bytes, so nothing of the state is left out.
fn tracker_state(sim: &mut SimContext, slot: ModuleSlot) -> String {
    let (mut save, crc) = module_state(sim, slot);
    assert_eq!(save[0], 2, "tracker save version");
    let tail = save.split_off(save.len() - 8);
    save[0] = 1;
    let word = |at: usize| u32::from_le_bytes(tail[at..at + 4].try_into().expect("word"));
    format!("{}.{crc}.{}.{}", hex(&save), word(0), word(4))
}

fn state(sim: &mut SimContext, answer: &str) -> String {
    let mut out = format!(
        "R{};{answer}{}",
        sim.world.rng.get_game_logic_random_seed_crc(),
        take_log()
    );
    for slot in live_objects(sim) {
        let health = sim.object_health(slot);
        let o = sim.object(slot).expect("object");
        let pos = *o.thing.get_position();
        let _ = write!(
            out,
            "O{},{},{},{},{},{}",
            o.id.0,
            health.to_bits(),
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits(),
            i32::from(o.is_undetected_defector())
        );
        let set = &o.weapon_set;
        let cur = set.get_cur_weapon_slot();
        if set.has_any_weapon() != FALSE && set.get_weapon_in_weapon_slot(cur).is_some() {
            let _ = write!(
                out,
                ",c{}{}",
                cur.as_i32(),
                i32::from(set.is_cur_weapon_locked())
            );
        }
        let weapons: Vec<_> = set.weapons.iter().flatten().cloned().collect();
        for mut weapon in weapons {
            let mut save = XferSave::new();
            Snapshot::xfer(&mut weapon, &mut save).expect("weapon save");
            let _ = write!(out, ",w{}", hex(save.as_bytes()));
        }
        if let Some(ms) = module_named(sim, slot, "FiringTracker") {
            let _ = write!(out, ",n{}", tracker_state(sim, ms));
        }
        if let Some(ms) = module_named(sim, slot, "FireWeaponPower") {
            let (save, crc) = module_state(sim, ms);
            let _ = write!(out, ",W{}.{crc}", hex(&save));
        }
        out.push(';');
    }
    let mines = (0..3).fold(0, |acc, p| {
        acc * 100
            + sim
                .world.players.player(p).map(|p| &p.academy)
                .map_or(0, |a| a.mines_cleared)
    });
    let _ = write!(out, "M{mines};");
    out
}

/// The answers to every question about one weapon, in the harness's order.
fn queries(
    sim: &mut SimContext,
    s: ObjectSlot,
    wslot: WeaponSlotType,
    t: Option<ObjectSlot>,
    pos: &Coord3D,
    angle: f32,
) -> String {
    let wslot = if sim
        .object(s)
        .is_some_and(|o| o.weapon_set.get_weapon_in_weapon_slot(wslot).is_some())
    {
        wslot
    } else {
        WeaponSlotType::Primary
    };
    let now = sim.world.frame;
    let own_pos = *sim.object(s).expect("shooter").thing.get_position();
    weapon_actions::with_weapon_in_slot(sim, s, wslot, |w, sim| {
        let sim: &dyn SimView = sim;
        let b = |v: Bool| i32::from(v);
        let range = weapon_queries::attack_range(w, sim, s);
        let radius = weapon_queries::primary_damage_radius(w, sim, s);
        let ready = w.get_percent_ready_to_fire(now);
        let mut out = format!(
            "Q{},{},{},{},{}",
            range.to_bits(),
            radius.to_bits(),
            ready.to_bits(),
            weapon_queries::attack_distance(w, sim, s, t).to_bits(),
            weapon_queries::attack_distance(w, sim, s, None).to_bits()
        );
        let target_pos = t
            .and_then(|t| sim.object(t))
            .map_or(*pos, |o| *o.thing.get_position());
        let _ = write!(
            out,
            ",a{}{}{}{}",
            b(weapon_queries::is_within_attack_range_of_position(
                w, sim, s, pos
            )),
            b(weapon_queries::is_too_close_to_position(w, sim, s, pos)),
            b(weapon_queries::is_goal_pos_within_attack_range(
                w, sim, s, pos, None, pos
            )),
            b(weapon_queries::is_source_at_goal_within_attack_range(
                w,
                sim,
                s,
                pos,
                None,
                &target_pos
            ))
        );
        let (done, approach) = weapon_queries::compute_approach_target(w, sim, s, None, pos, angle);
        let _ = write!(
            out,
            ",p{}.{}.{}.{}",
            b(done),
            approach.x.to_bits(),
            approach.y.to_bits(),
            approach.z.to_bits()
        );
        let delay = weapon_actions::pre_attack_delay(w, sim, s, None);
        // Against a position the estimate does not look at the victim.
        let damage = weapon_queries::estimate_weapon_damage(w, sim, s, None);
        let _ = write!(out, ",d{delay},e{}", damage.to_bits());
        let first = weapon_queries::is_clear_firing_los_terrain_to_position(sim, s, pos);
        let second =
            weapon_queries::is_clear_goal_firing_los_terrain_to_position(sim, s, pos, &own_pos);
        let _ = write!(out, ",l{}{}", b(first), b(second));
        if let Some(t) = t.filter(|&t| t != s) {
            let at = *sim.object(t).expect("target").thing.get_position();
            let _ = write!(
                out,
                ",A{}{}{}{}{}",
                b(weapon_queries::is_within_attack_range(w, sim, s, t)),
                b(weapon_queries::is_too_close(w, sim, s, t)),
                b(weapon_queries::is_goal_pos_within_attack_range(
                    w,
                    sim,
                    s,
                    pos,
                    Some(t),
                    &at
                )),
                b(weapon_queries::is_source_at_goal_within_attack_range(
                    w,
                    sim,
                    s,
                    pos,
                    Some(t),
                    &at
                )),
                b(weapon_queries::is_within_target_pitch(w, sim, s, t))
            );
            let (done, approach) =
                weapon_queries::compute_approach_target(w, sim, s, Some(t), &at, angle);
            let _ = write!(
                out,
                ",P{}.{}.{}.{}",
                b(done),
                approach.x.to_bits(),
                approach.y.to_bits(),
                approach.z.to_bits()
            );
            let delay = weapon_actions::pre_attack_delay(w, sim, s, Some(t));
            let damage = weapon_queries::estimate_weapon_damage(w, sim, s, Some(t));
            let _ = write!(out, ",D{delay},E{}", damage.to_bits());
            let first = weapon_queries::is_clear_firing_los_terrain(sim, s, t);
            let second = weapon_queries::is_clear_goal_firing_los_terrain(sim, s, pos, t);
            let _ = write!(out, ",L{}{}", b(first), b(second));
            let collides = weapon_fire::should_projectile_collide_with(
                sim,
                Some(&w.template.name),
                None,
                s,
                t,
                ObjectId::INVALID,
            );
            let _ = write!(out, ",K{}", i32::from(collides));
        }
        out.push(';');
        out
    })
    .expect("weapon")
}

fn use_power(sim: &mut SimContext, s: ObjectSlot, mode: i64, t: Option<ObjectSlot>, pos: &Coord3D) {
    let Some(ms) = module_named(sim, s, "FireWeaponPower") else {
        return;
    };
    let _ = sim.dispatch_module(ms, |m, sim, ctx| {
        let power = m.as_special_power_mut().expect("power");
        match (mode, t) {
            (0, _) => power.do_special_power(sim, ctx, 0),
            (1, _) => power.do_special_power_at_location(sim, ctx, pos, 0.0, 0),
            (_, Some(t)) => power.do_special_power_at_object(sim, ctx, t, 0),
            _ => {}
        }
    });
}

fn run(sim: &mut SimContext, ids: &[ObjectId], op: &[i64], frame: &mut u32) -> String {
    let find = |sim: &SimContext, i: i64| sim.world.objects.slot_by_id(ids[i as usize]);
    match op[0] {
        4 => {
            *frame += op[1] as u32;
            sim.world.frame = *frame;
            weapon_effects::update_delayed_damage(sim);
        }
        9 => {
            if let Some(o) = find(sim, op[1]) {
                let (x, y) = (op[2] as f32, op[3] as f32);
                let z = Terrain.get_ground_height(x, y, None);
                sim.set_object_position(o, &Coord3D::new(x, y, z));
            }
        }
        12 => {
            if let (Some(o), Some(anchor)) = (find(sim, op[1]), find(sim, op[2])) {
                let mut pos = *sim.object(anchor).expect("anchor").thing.get_position();
                pos.x += op[3] as f32 * 0.5;
                if o != anchor && pos.x < 990.0 {
                    sim.set_object_position(o, &pos);
                }
            }
        }
        kind => {
            let Some(s) = find(sim, op[1]) else {
                return String::new();
            };
            match kind {
                13 => {
                    let Some(t) = find(sim, op[3]).filter(|&t| t != s) else {
                        return String::new();
                    };
                    let (me, them) = (
                        sim.object(s).expect("shooter"),
                        sim.object(t).expect("target"),
                    );
                    let slot = SLOTS[op[2] as usize];
                    let Some(weapon) =
                        me.weapon_set.get_weapon_in_weapon_slot(slot).or_else(|| {
                            me.weapon_set
                                .get_weapon_in_weapon_slot(WeaponSlotType::Primary)
                        })
                    else {
                        return String::new();
                    };
                    let mut limit = if op[4] % 2 != 0 {
                        weapon.template.get_minimum_attack_range()
                    } else {
                        weapon_queries::attack_range(weapon, sim, s)
                    };
                    if op[4] == 2 {
                        limit -= 2.5;
                    }
                    if op[4] == 3 {
                        limit += 2.5;
                    }
                    limit += (op[5] - 1) as f32 * 0.5;
                    let mine = me.geometry.get_bounding_circle_radius();
                    let mut pos = *me.thing.get_position();
                    let mut at = pos;
                    pos.x += limit + mine + them.geometry.get_bounding_circle_radius();
                    if pos.x >= 990.0 || pos.x < 0.0 {
                        return String::new();
                    }
                    sim.set_object_position(t, &pos);
                    sim.update_partition();
                    at.x += limit + mine;
                    return queries(sim, s, slot, Some(t), &at, OFFSETS[op[7] as usize]);
                }
                14 => {
                    let set = &sim.object(s).expect("shooter").weapon_set;
                    let next = set
                        .get_weapon_in_weapon_slot(set.get_cur_weapon_slot())
                        .map(|w| w.when_we_can_fire_again);
                    let (Some(next), Some(t)) = (next, find(sim, op[2]).filter(|&t| t != s)) else {
                        return String::new();
                    };
                    let before = op[3] as u32;
                    if next < 0x7fff_ffff && next >= *frame + before {
                        *frame = next - before;
                        sim.world.frame = *frame;
                    }
                    weapon_actions::pre_fire_current_weapon(sim, s, Some(t));
                }
                10 => sim
                    .world
                    .objects
                    .get_mut(s)
                    .expect("object")
                    .set_undetected_defector(TRUE),
                0 | 11 => {
                    let pos = if kind == 0 {
                        Coord3D::new(op[4] as f32 + 0.25, op[5] as f32, op[6] as f32)
                    } else {
                        let mut pos = *sim.object(s).expect("shooter").thing.get_position();
                        let offset = op[4] as f32 * 0.5;
                        if op[5] != 0 {
                            pos.y += offset;
                        } else {
                            pos.x += offset;
                        }
                        pos
                    };
                    let has_primary = sim.object(s).is_some_and(|o| {
                        o.weapon_set
                            .get_weapon_in_weapon_slot(WeaponSlotType::Primary)
                            .is_some()
                    });
                    if has_primary {
                        let t = find(sim, op[3]);
                        let angle = OFFSETS[op[7] as usize];
                        return queries(sim, s, SLOTS[op[2] as usize], t, &pos, angle);
                    }
                }
                1 => {
                    if let Some(t) = find(sim, op[2]).filter(|&t| t != s) {
                        weapon_actions::fire_current_weapon_at_object(sim, s, t);
                    }
                }
                2 => {
                    let pos = Coord3D::new(op[2] as f32, op[3] as f32, op[4] as f32);
                    weapon_actions::fire_current_weapon_at_position(sim, s, &pos);
                }
                3 => {
                    if let Some(t) = find(sim, op[2]).filter(|&t| t != s) {
                        weapon_actions::pre_fire_current_weapon(sim, s, Some(t));
                    }
                }
                5 => weapon_actions::reload_all_ammo(sim, s, Bool::from(op[2] != 0)),
                6 => {
                    let t = find(sim, op[3]);
                    let pos = Coord3D::new(op[4] as f32, op[5] as f32, 0.0);
                    use_power(sim, s, op[2], t, &pos);
                }
                7 => {
                    let condition = CONDITIONS[op[2] as usize];
                    if op[3] != 0 {
                        sim.set_weapon_bonus_condition(s, condition);
                    } else {
                        sim.clear_weapon_bonus_condition(s, condition);
                    }
                }
                8 => {
                    let lock = if op[3] == 1 {
                        WeaponLockType::LockedTemporarily
                    } else {
                        WeaponLockType::LockedPermanently
                    };
                    let o = sim.world.objects.get_mut(s).expect("object");
                    o.weapon_set.set_weapon_lock(SLOTS[op[2] as usize], lock);
                }
                other => panic!("op {other}"),
            }
        }
    }
    String::new()
}

fn spawn_units(sim: &mut SimContext, units: &[Vec<i64>]) -> (Vec<ObjectId>, BTreeSet<ObjectId>) {
    let mut ids = Vec::new();
    let mut sneaky = BTreeSet::new();
    for u in units {
        let kind = u[0];
        let name = if kind < 8 {
            format!("E_S{kind}")
        } else {
            format!("E_T{}", kind - 8)
        };
        let template = sim
            .rules
            .thing_factory
            .find_template_id(&name)
            .unwrap_or_else(|| panic!("missing template {name}"));
        let (x, y) = (u[2] as f32, u[3] as f32);
        let z = Terrain.get_ground_height(x, y, None) + u[4] as f32;
        let slot = sim.spawn_object(
            template,
            TEAMS[u[1] as usize],
            Coord3D::new(x, y, z),
            ANGLES[u[5] as usize],
        );
        for (bit, &condition) in CONDITIONS.iter().enumerate() {
            if u[6] & (1 << bit) != 0 {
                sim.set_weapon_bonus_condition(slot, condition);
            }
        }
        if u[7] & 1 != 0 {
            sim.set_status(slot, ObjectStatusType::AirborneTarget);
        }
        if u[7] & 2 != 0 {
            sim.set_status(slot, ObjectStatusType::UnderConstruction);
        }
        let o = sim.world.objects.get_mut(slot).expect("spawned");
        if u[7] & 4 != 0 {
            sneaky.insert(o.id);
        }
        if u[7] & 8 != 0 {
            o.set_undetected_defector(TRUE);
        }
        let id = o.id;
        ids.push(id);
    }
    (ids, sneaky)
}

/// Replays vectors until the first differences; returns them with the row texts seen.
fn replay(lines: &[&str]) -> (Vec<String>, String) {
    let mut sim = SimContext::new();
    sim.rules = rules();
    sim.terrain = Box::new(Terrain);
    let mut failed = Vec::new();
    let mut seen = String::new();
    for &line in lines {
        let index = get_u32(line, "vector").expect("vector");
        let mut frame = get_u32(line, "frame").expect("frame");
        sim.world = new_world(frame, get_u32(line, "seed").expect("seed"));
        sim.init_partition(
            20.0,
            Region3D {
                lo: Coord3D::new(0.0, 0.0, 0.0),
                hi: Coord3D::new(1000.0, 1000.0, 100.0),
            },
        );
        let ops = arrays(line, "ops");
        let expected = rows(line);
        assert_eq!(expected.len(), ops.len() + 1, "vector {index}");
        take_log();
        let (ids, sneaky) = spawn_units(&mut sim, &arrays(line, "units"));
        sim.ai_seams = shared(Seams { sneaky });
        sim.update_partition();
        for (step, want) in expected.iter().enumerate() {
            let answer = match step.checked_sub(1) {
                Some(i) => {
                    let answer = run(&mut sim, &ids, &ops[i], &mut frame);
                    sim.process_destroy_list();
                    sim.update_partition();
                    answer
                }
                None => String::new(),
            };
            let got = state(&mut sim, &answer);
            seen.push_str(&got);
            let same = match want.strip_prefix('#') {
                Some(hash) => fnv(&got).to_string() == hash,
                None => got == *want,
            };
            if !same {
                let op = step.checked_sub(1).map(|i| &ops[i]);
                let detail = if want.starts_with('#') {
                    format!("got {got}")
                } else {
                    let at = got
                        .bytes()
                        .zip(want.bytes())
                        .position(|(a, b)| a != b)
                        .unwrap_or(got.len().min(want.len()));
                    let from = at.saturating_sub(120);
                    format!(
                        "at {at}\n got ..{}\nwant ..{}",
                        &got[from..got.len().min(at + 120)],
                        &want[from..want.len().min(at + 120)]
                    )
                };
                failed.push(format!("vector {index} row {step} op {op:?}: {detail}"));
                break;
            }
        }
        if failed.len() >= 5 {
            break;
        }
    }
    (failed, seen)
}

#[test]
fn s13_weapon_firing_matches_the_reference() {
    // The vectors come from the default build of the reference.
    if cfg!(any(feature = "retail-crc", feature = "retail-xfer-save")) {
        return;
    }
    let lines: Vec<&str> = GOLDEN
        .lines()
        .filter(|l| l.starts_with("{\"vector\""))
        .collect();
    assert_eq!(
        lines.len(),
        get_u32(GOLDEN.lines().next().expect("header"), "vector_count").expect("count") as usize
    );
    let (failed, seen) = replay(&lines);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
    // The vectors must reach what they claim to cover.
    for (what, mark, least) in [
        ("queries", ";Q", 1000),
        ("sight questions", ";L", 1000),
        ("attack orders", " attack 2 ", 50),
        ("position orders", " attackpos 2 ", 100),
        ("assault orders", ";B", 20),
    ] {
        let count = seen.matches(mark).count();
        assert!(count >= least, "{what}: {count}");
    }
    let disarmed = seen
        .split('R')
        .filter(|r| r.contains(";M") && !r.contains(";M0;"));
    assert!(disarmed.count() >= 100, "disarm shots that clear a mine");
}

// -----------------------------------------------------------------------------
// Questions only a seam answers. The reference run has no bridge record and no
// aircraft that adjusts its destination, so these have no vectors.
// -----------------------------------------------------------------------------

#[derive(Default)]
struct Answers {
    points: [Coord3D; 2],
}

impl AiSeams for Answers {
    fn bridge_attack_points(&mut self, _bridge: ObjectId, _pos: &Coord3D) -> [Coord3D; 2] {
        self.points
    }
    fn is_aircraft_that_adjusts_destination(&mut self, _owner: ObjectId) -> Bool {
        TRUE
    }
    fn adjust_target_destination(
        &mut self,
        owner: ObjectId,
        target: Option<ObjectId>,
        target_pos: &Coord3D,
        weapon: &str,
        approach: Coord3D,
    ) -> Coord3D {
        log(&format!(
            "{} {target:?} {} {weapon};",
            owner.0, target_pos.x
        ));
        Coord3D::new(approach.x, approach.y, 77.0)
    }
    fn map_extent(&mut self, extent: Region3D, maximum_pathfind: bool) -> Region3D {
        log(&format!("extent {maximum_pathfind};"));
        extent
    }
}

#[test]
fn bridge_points_and_destination_adjustment_come_from_the_seams() {
    let mut sim = SimContext::new();
    sim.rules = rules();
    sim.terrain = Box::new(Terrain);
    sim.world = new_world(1000, 1);
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    // A shooter and a bridge far from it.
    let units = [
        vec![0, 0, 300, 300, 0, 0, 0, 0],
        vec![17, 1, 600, 300, 0, 0, 0, 0],
    ];
    let (ids, _) = spawn_units(&mut sim, &units);
    sim.update_partition();
    let find = |sim: &SimContext, i: usize| sim.world.objects.slot_by_id(ids[i]).expect("unit");
    let (shooter, bridge) = (find(&sim, 0), find(&sim, 1));
    let mut template = WeaponTemplate::new();
    template.name = "E_Hand".to_owned();
    template.primary_damage = 5.0;
    template.attack_range = 100.0;
    template.weapon_speed = 1.0;
    sim.rules.weapon_store.add_template(template.clone());
    let weapon = Weapon::new(template, WeaponSlotType::Primary, 0);
    let far = Coord3D::new(500.0, 300.0, 0.0);
    let near = Coord3D::new(350.0, 300.0, 0.0);

    for points in [[far, near], [near, far], [far, far]] {
        sim.ai_seams = shared(Answers { points });
        let reachable = points.contains(&near);
        let within = weapon_queries::is_within_attack_range(&weapon, &sim, shooter, bridge);
        assert_eq!(within != FALSE, reachable);
        let from = *sim.object(shooter).expect("shooter").thing.get_position();
        let goal = weapon_queries::is_goal_pos_within_attack_range(
            &weapon,
            &sim,
            shooter,
            &from,
            Some(bridge),
            &far,
        );
        assert_eq!(goal != FALSE, reachable);

        // The shot lands, later, on the nearer point.
        sim.world.weapon_store_state.delayed.clear();
        let mut shot = weapon.clone();
        shot.ammo_in_clip = 5;
        weapon_fire::fire_weapon_at_object(&mut shot, &mut sim, shooter, bridge);
        let landed = sim.world.weapon_store_state.delayed.first().map(|d| d.pos);
        assert_eq!(landed, reachable.then_some(near));
    }

    take_log();
    let pos = Coord3D::new(600.0, 300.0, 0.0);
    let (_, approach) =
        weapon_queries::compute_approach_target(&weapon, &sim, shooter, None, &pos, 0.0);
    assert_eq!(approach.z, 77.0);
    assert_eq!(take_log(), format!("{} None 600 E_Hand;", ids[0].0));

    // Too close: the spot is clipped to the playable extent, not the pathfind one.
    let mut template = weapon.template.clone();
    template.minimum_attack_range = 50.0;
    let weapon = Weapon::new(template, WeaponSlotType::Primary, 0);
    let close = Coord3D::new(310.0, 300.0, 0.0);
    weapon_queries::compute_approach_target(&weapon, &sim, shooter, None, &close, 0.0);
    assert_eq!(take_log(), "extent false;");
}
