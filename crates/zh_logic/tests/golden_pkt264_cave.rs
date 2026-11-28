//! `PKT-264` `CaveContain` against `docs/golden_vectors/PKT264_Cave.json`.
//!
//! Recorded by `tests/cpp_harness/gen_pkt264_cave.cpp`, which lists what is left out. The
//! operations and the preconditions below mirror `apply` there.

use std::any::Any;
use std::fmt::Write as _;
use zh_common::golden_json::{extract_vector_lines, get_i32_vec, get_str, get_u32};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, NameKeyType, ObjectId, Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferError, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, CaveFacet, CommandSourceType, ContainFacet, ModuleData,
    ModuleType,
};
use zh_logic::modules::module_factory::ModuleParseContext;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld, TunnelTrackerState};

// -----------------------------------------------------------------------------
// Stand-in for the unported AIUpdateInterface (construction draws only)
// -----------------------------------------------------------------------------

const AI_MASK: FacetMask = FacetMask(0x001 | FacetMask::AI_UPDATE.0);

struct StubData {
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
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
        AI_MASK
    }
}

struct AiStandIn(NameKeyType);

impl AiUpdateFacet for AiStandIn {}

impl BehaviorModule for AiStandIn {
    fn module_name_key(&self) -> NameKeyType {
        self.0
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
}

fn parse_ai(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(StubData {
        tag_key,
        module_name_key,
    }))
}

fn create_ai(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    Box::new(AiStandIn(data.module_name_key()))
}

// -----------------------------------------------------------------------------
// Fixture (mirrors `fixture_ini` in the harness; the stealth units' `StealthUpdate` is
// unported and left out, and nothing here runs it)
// -----------------------------------------------------------------------------

fn object(s: &mut String, name: &str, kind_of: &str, health: &str, rest: &str) {
    let _ = write!(
        s,
        "Object {name}\n  KindOf = PRELOAD {kind_of}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n{rest}End\n"
    );
}

const UNITS: [&str; 9] = [
    "P264_Inf",
    "P264_Hero",
    "P264_Veh",
    "P264_Air",
    "P264_Stealth",
    "P264_StealthHero",
    "P264_DestroyDie",
    "P264_Ai",
    "P264_Trans",
];

fn objects_ini() -> String {
    let mut s = String::new();
    let destroy_die = "  Behavior = DestroyDie ModuleTag_Die\n  End\n";
    for (i, index) in [0, 0, 0, 1, 1, 2, 4, 0].into_iter().enumerate() {
        let mut contain =
            format!("  Behavior = CaveContain ModuleTag_Contain\n    CaveIndex = {index}\n");
        if i == 2 {
            contain.push_str("    DeathTypes = NONE +SUICIDED\n");
        }
        contain.push_str("  End\n");
        let modules = match i {
            4 | 7 => format!("{contain}{destroy_die}"),
            5 => format!("{destroy_die}{contain}"),
            _ => contain,
        };
        object(
            &mut s,
            &format!("P264_Cave{i}"),
            "STRUCTURE",
            "500.0",
            &modules,
        );
    }
    let ai = "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n";
    let transport = "  Behavior = TransportContain ModuleTag_Contain\n    Slots = 8\n    AllowInsideKindOf = INFANTRY\n  End\n";
    object(
        &mut s,
        "P264_Inf",
        "INFANTRY\n  TransportSlotCount = 1",
        "100.0",
        "",
    );
    object(&mut s, "P264_Hero", "INFANTRY HERO", "200.0", "");
    object(&mut s, "P264_Veh", "VEHICLE", "300.0", "");
    object(&mut s, "P264_Air", "VEHICLE AIRCRAFT", "150.0", "");
    object(
        &mut s,
        "P264_Stealth",
        "INFANTRY STEALTH_GARRISON",
        "90.0",
        "",
    );
    object(
        &mut s,
        "P264_StealthHero",
        "INFANTRY HERO STEALTH_GARRISON",
        "95.0",
        "",
    );
    object(&mut s, "P264_DestroyDie", "INFANTRY", "60.0", destroy_die);
    object(&mut s, "P264_Ai", "INFANTRY", "110.0", ai);
    object(&mut s, "P264_Trans", "VEHICLE", "250.0", transport);
    s
}

fn build_rules() -> RulesRegistry {
    let mut rules = RulesRegistry::new();
    rules.module_registry.register_module(
        &mut rules.name_keys,
        "AIUpdateInterface",
        ModuleType::Behavior,
        AI_MASK,
        SleepyUpdatePhase::Initial,
        true,
        parse_ai,
        create_ai,
    );
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    let text = objects_ini();
    let mut ini = IniReader::new("object.ini", text.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    rules
}

// -----------------------------------------------------------------------------
// World
// -----------------------------------------------------------------------------

/// Neutral, A, B, and a second team of A's player, with the reference's `TeamID`s.
const TEAMS: [TeamId; 4] = [TeamId(3), TeamId(1), TeamId(2), TeamId(4)];

fn reset_world(sim: &mut SimContext, local_player: Int) {
    sim.world = SimWorld::new();
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    let w = &mut sim.world;
    w.team_players.insert(TEAMS[3], 1);
    for (player, team) in TEAMS.into_iter().take(3).enumerate() {
        w.team_players.insert(team, player as Int);
        w.player_default_teams.insert(player as Int, team);
    }
    w.team_is_neutral.insert(TEAMS[0], TRUE);
    w.neutral_team_id = TEAMS[0];
    w.local_player_index = local_player;
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

struct Scene {
    caves: usize,
    units: usize,
    slots: Vec<ObjectSlot>,
    ids: Vec<ObjectId>,
    /// Cleared when the harness would null its pointer.
    live: Vec<bool>,
    orders: u32,
}

impl Scene {
    fn get(&self, i: usize) -> Option<ObjectSlot> {
        self.live[i].then(|| self.slots[i])
    }
    fn cave(&self, i: u32) -> Option<ObjectSlot> {
        self.get(i as usize)
    }
    fn unit(&self, i: u32) -> Option<ObjectSlot> {
        self.get(self.caves + i as usize)
    }
}

fn spawn_at(sim: &mut SimContext, s: &mut Scene, name: &str, team: TeamId, x: u32, y: u32) {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = sim.spawn_object_with_status(
        tid,
        team,
        Coord3D::default(),
        0.0,
        ObjectStatusMaskType::new(),
    );
    sim.set_object_position(slot, &Coord3D::new(x as f32, y as f32, 0.0));
    s.slots.push(slot);
    s.ids.push(sim.object(slot).expect("spawned").id);
    s.live.push(true);
}

fn contain_slot(sim: &SimContext, owner: ObjectSlot) -> ModuleSlot {
    sim.object(owner)
        .and_then(|o| o.contain_slot)
        .expect("contain module")
}

fn facet<R>(
    sim: &SimContext,
    owner: ObjectSlot,
    f: impl FnOnce(&dyn ContainFacet, ModuleContext) -> R,
) -> R {
    let ms = contain_slot(sim, owner);
    let ctx = sim.world.modules.meta(ms).context(ms);
    let module = sim.world.modules.payload(ms).expect("payload");
    f(module.as_contain().expect("contain facet"), ctx)
}

fn dispatch_contain(
    sim: &mut SimContext,
    owner: ObjectSlot,
    f: impl FnOnce(&mut dyn ContainFacet, &mut dyn SimApi, ModuleContext),
) {
    let ms = contain_slot(sim, owner);
    sim.dispatch_module(ms, |m, s, c| {
        f(m.as_contain_mut().expect("contain facet"), s, c)
    })
    .expect("dispatch");
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn save_module(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.into_bytes()
}

/// The reference library saves version 1 of the tracker, which ends with the tunnel count; the
/// default build writes version 2, which drops it.
fn tracker_hex(state: &mut TunnelTrackerState) -> String {
    let mut xs = XferSave::new();
    zh_common::xfer::Snapshot::xfer(state, &mut xs).expect("xfer tracker");
    let mut bytes = xs.into_bytes();
    if cfg!(not(feature = "retail-xfer-save")) {
        assert_eq!(bytes[0], 2);
        bytes[0] = 1;
        bytes.extend_from_slice(&(state.tunnel_ids.len() as u32).to_le_bytes());
    }
    hex(&bytes)
}

/// `m_caveIndex`, which the module saves before the original team.
fn cave_index(sim: &mut SimContext, cave: ObjectSlot) -> Int {
    let ms = contain_slot(sim, cave);
    let bytes = save_module(sim, ms);
    let at = bytes.len() - 8;
    Int::from_le_bytes(bytes[at..at + 4].try_into().expect("cave index"))
}

fn has_tracker(sim: &mut SimContext, cave: ObjectSlot) -> bool {
    let index = cave_index(sim, cave);
    sim.cave_tracker(index).is_some()
}

/// A cave whose index has a tracker.
fn live_cave(sim: &mut SimContext, s: &Scene, i: u32) -> Option<ObjectSlot> {
    s.cave(i).filter(|&c| has_tracker(sim, c))
}

fn trackers(sim: &SimContext) -> Vec<Option<TunnelTrackerState>> {
    (0..sim.world.cave_system.len())
        .map(|i| sim.cave_tracker(i as Int).cloned())
        .collect()
}

fn dispatch_cave(
    sim: &mut SimContext,
    owner: ObjectSlot,
    f: impl FnOnce(&mut dyn CaveFacet, &mut dyn SimApi, ModuleContext),
) {
    let ms = contain_slot(sim, owner);
    sim.dispatch_module(ms, |m, s, c| f(m.as_cave_mut().expect("cave facet"), s, c))
        .expect("dispatch");
}

fn observe(sim: &mut SimContext, s: &Scene) -> [String; 5] {
    let all = trackers(sim);
    let mut sys = format!("{}:", all.len());
    for tracker in all {
        let Some(mut t) = tracker else {
            sys.push_str("-;");
            continue;
        };
        let _ = write!(
            sys,
            "{} {} {} {} {}",
            tracker_hex(&mut t),
            t.contain_list_size,
            t.hero_units_contained,
            t.tunnel_count_effective(),
            t.tunnel_ids.len()
        );
        for id in &t.tunnel_ids {
            let _ = write!(sys, " {}", id.0);
        }
        let _ = write!(sys, " {}", t.contain_list.len());
        for id in &t.contain_list {
            let _ = write!(sys, " {}", id.0);
        }
        sys.push(';');
    }

    let mut module = String::new();
    for i in 0..s.caves {
        match s.get(i) {
            Some(slot) => {
                let ms = contain_slot(sim, slot);
                let _ = write!(module, "{} ", hex(&save_module(sim, ms)));
            }
            None => module.push_str("- "),
        }
    }

    let mut obj = String::new();
    for i in 0..s.slots.len() {
        let Some(slot) = s.get(i) else {
            obj.push_str("-;");
            continue;
        };
        let health = sim.object_health(slot);
        let o = sim.object(slot).expect("live object");
        let pos = o.thing.get_position();
        let team = TEAMS
            .iter()
            .position(|&t| t == o.team_id)
            .unwrap_or_else(|| {
                assert_eq!(o.team_id, TeamId::INVALID, "unknown team");
                4
            });
        let contained = o
            .contained_by
            .and_then(|c| sim.object(c))
            .map_or(0, |c| c.id.0);
        let status = |t: ObjectStatusType| u8::from(o.status.test(t.as_i32()) != FALSE);
        let detected = sim
            .world
            .stealth_detected_events
            .iter()
            .any(|e| e.0 == o.id);
        let _ = write!(
            obj,
            "{} {} {} {} {} {} {} {} {} {} {} {} {} {} {};",
            u8::from(o.is_destroyed()),
            u8::from(o.is_effectively_dead() != FALSE),
            team,
            contained,
            o.contained_by_frame,
            status(ObjectStatusType::Masked),
            status(ObjectStatusType::Unselectable),
            u8::from(o.disabled_mask.test(DisabledType::Held as Int) != FALSE),
            u8::from(sim.world.partition.is_registered(o.id)),
            u8::from(o.is_undetected_defector() != FALSE),
            u8::from(detected),
            health.to_bits(),
            pos.x.to_bits(),
            pos.y.to_bits(),
            pos.z.to_bits()
        );
    }

    let mut qry = String::new();
    for i in 0..s.caves {
        let Some(slot) = s.get(i) else {
            qry.push_str("-;");
            continue;
        };
        let ms = contain_slot(sim, slot);
        let mut build = FALSE;
        sim.dispatch_module(ms, |m, _, _| {
            build = m
                .as_create_mut()
                .expect("create facet")
                .should_do_on_build_complete();
        })
        .expect("dispatch");
        let tracked = has_tracker(sim, slot);
        facet(sim, slot, |m, ctx| {
            let _ = write!(
                qry,
                "{} {} {} {}{}{}{}{}{}{}{}",
                m.get_hero_units_contained(sim, ctx),
                m.get_stealth_units_contained(),
                m.get_player_entered_mask(),
                m.is_kick_out_on_capture(sim, ctx),
                m.is_tunnel_contain(),
                m.is_bustable(),
                m.is_garrisonable(sim, ctx),
                m.is_heal_contain(),
                m.is_immune_to_clear_building_attacks(),
                m.is_special_zero_slot_container(),
                build,
            );
            if !tracked {
                qry.push_str(" n;");
                return;
            }
            let _ = write!(
                qry,
                " {} {}",
                m.get_contain_count(sim, ctx),
                m.get_contain_max(sim, ctx) as u32 as Int
            );
            let (mut contained, mut valid, mut valid_full) = (0u32, 0u32, 0u32);
            for k in 0..s.units {
                let Some(unit) = s.get(s.caves + k) else {
                    continue;
                };
                if m.is_contained(sim, ctx, s.ids[s.caves + k]) != FALSE {
                    contained |= 1 << k;
                }
                if m.is_valid_container_for(sim, ctx, unit, FALSE) != FALSE {
                    valid |= 1 << k;
                }
                if m.is_valid_container_for(sim, ctx, unit, TRUE) != FALSE {
                    valid_full |= 1 << k;
                }
            }
            let _ = write!(qry, " {contained} {valid} {valid_full}");
            for id in m.get_contained_items(sim, ctx) {
                let _ = write!(qry, " {}", id.0);
            }
            qry.push(';');
        });
    }

    let misc = format!(
        "{} {}",
        sim.world.rng.get_game_logic_random_seed_crc(),
        s.orders
    );
    [sys, module, obj, qry, misc]
}

fn flush_is_safe(sim: &SimContext, s: &Scene) -> bool {
    let destroyed = |slot: ObjectSlot| sim.object(slot).is_some_and(|o| o.is_destroyed());
    for tracker in trackers(sim).into_iter().flatten() {
        for id in tracker.contain_list {
            if sim.find_object(id).is_some_and(destroyed) {
                return false;
            }
        }
    }
    !(0..s.slots.len()).filter_map(|i| s.get(i)).any(|slot| {
        sim.object(slot)
            .and_then(|o| o.contained_by)
            .is_some_and(destroyed)
    })
}

fn hurt(sim: &mut SimContext, slot: ObjectSlot, amount: u32, source: ObjectId) {
    let mut d = DamageInfo::default();
    d.input.damage_type = DamageType::Explosion;
    d.input.death_type = DeathType::Normal;
    d.input.amount = amount as f32;
    d.input.source_id = source;
    sim.attempt_damage(slot, &mut d);
}

/// The AI orders `issue` gives, as a mask over `s.slots`.
fn record_orders(
    sim: &mut SimContext,
    s: &mut Scene,
    cave: u32,
    source: Option<CommandSourceType>,
    seed: u32,
    issue: impl FnOnce(&mut dyn ContainFacet, &mut dyn SimApi, ModuleContext),
) {
    let slot = s.slots[cave as usize];
    sim.world.ai_exit_orders.clear();
    sim.world.ai_commands.clear();
    dispatch_contain(sim, slot, issue);
    let cave_id = s.ids[cave as usize];
    let ordered: Vec<ObjectId> = match source {
        Some(source) => sim
            .world
            .ai_exit_orders
            .iter()
            .map(|&(rider, container, src)| {
                assert_eq!(container, cave_id, "exit order names the cave");
                assert_eq!(src, source);
                rider
            })
            .collect(),
        None => sim.world.ai_commands.iter().map(|c| c.0).collect(),
    };
    for (k, id) in s.ids.iter().enumerate() {
        // The reference AI ignores commands to the dead.
        let heard = s
            .get(k)
            .and_then(|slot| sim.object(slot))
            .is_some_and(|o| o.ai_slot.is_some() && o.is_effectively_dead() == FALSE);
        if heard && ordered.contains(id) {
            s.orders |= 1 << k;
        }
    }
    sim.world.rng.init_random_with_seed(seed);
}

fn apply(sim: &mut SimContext, s: &mut Scene, op: [u32; 4]) {
    let [code, a, b, c] = op;
    s.orders = 0;
    match code {
        0 => {
            if let (Some(t), Some(u)) = (live_cave(sim, s, b), s.unit(a)) {
                sim.contain_add(t, u);
            }
        }
        1 => {
            if let (Some(t), Some(u)) = (live_cave(sim, s, b), s.unit(a)) {
                sim.contain_remove(t, u, u8::from(c != 0));
            }
        }
        2 => {
            if let Some(t) = s.cave(a) {
                let building = sim.object(t).is_some_and(|o| {
                    o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
                });
                if building || has_tracker(sim, t) {
                    let death = if b != 0 {
                        DeathType::Suicided
                    } else {
                        DeathType::Normal
                    };
                    kill_object(sim, t, DamageType::Unresistable, death);
                }
            }
        }
        3 => {
            if let Some(t) = s.cave(a) {
                sim.destroy_object(t);
            }
        }
        4 => {
            if live_cave(sim, s, a).is_some() {
                let source = Some(CommandSourceType::FromAi);
                record_orders(sim, s, a, source, c, |m, sim, ctx| m.on_selling(sim, ctx));
            }
        }
        5 => {
            if let Some(t) = s.cave(a) {
                sim.set_object_team(t, TEAMS[b as usize]);
            }
        }
        6 => {
            if let Some(t) = s.cave(a) {
                sim.dispatch_on_build_complete(t);
            }
        }
        7 => {
            if let Some(t) = live_cave(sim, s, a) {
                dispatch_cave(sim, t, |m, sim, ctx| {
                    m.try_to_set_cave_index(sim, ctx, b as Int)
                });
            }
        }
        8 => {
            let target = if a as usize >= s.caves {
                s.get(a as usize)
            } else {
                live_cave(sim, s, a)
            };
            if let Some(o) = target {
                let mut source = ObjectId::INVALID;
                if c != 0 {
                    source = s.ids[c as usize - 1];
                    // A hit from or on a cave left without a team names no source.
                    let teamless = |slot: ObjectSlot| {
                        sim.object(slot)
                            .is_some_and(|o| o.team_id == TeamId::INVALID)
                    };
                    if s.get(c as usize - 1)
                        .is_some_and(|from| teamless(from) || teamless(o))
                    {
                        source = ObjectId::INVALID;
                    }
                }
                hurt(sim, o, b, source);
            }
        }
        9 => {
            if let Some(u) = s.unit(a) {
                sim.destroy_object(u);
            }
        }
        10 => {
            if let Some(u) = s.unit(a) {
                kill_object(sim, u, DamageType::Unresistable, DeathType::Normal);
            }
        }
        11 => {
            if let Some(t) = live_cave(sim, s, a) {
                sim.contain_remove_all(t, u8::from(b != 0));
            }
        }
        12 => {
            if let Some(t) = live_cave(sim, s, a) {
                sim.contain_kill_all(t);
            }
        }
        13 => {
            if let Some(t) = live_cave(sim, s, a) {
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::Explosion;
                d.input.death_type = DeathType::Normal;
                d.input.amount = b as f32;
                sim.contain_harm_and_force_exit_all(t, &d);
            }
        }
        14 => {
            if let Some(t) = live_cave(sim, s, a) {
                if !sim.object(t).expect("cave").is_destroyed() {
                    let ms = contain_slot(sim, t);
                    sim.dispatch_module(ms, |m, sim, ctx| {
                        m.as_update_mut().expect("update facet").update(sim, ctx);
                    })
                    .expect("dispatch update");
                }
            }
        }
        15 => sim.world.frame += a,
        16 => {
            if flush_is_safe(sim, s) {
                for i in 0..s.slots.len() {
                    if s.get(i)
                        .is_some_and(|slot| sim.object(slot).is_some_and(|o| o.is_destroyed()))
                    {
                        s.live[i] = false;
                    }
                }
                sim.process_destroy_list();
            }
        }
        17 | 18 => {
            let target = if code == 17 { s.unit(a) } else { s.cave(a) };
            if let Some(o) = target {
                let status = match (code, b) {
                    (17, 0) => ObjectStatusType::Stealthed,
                    (17, _) => ObjectStatusType::Detected,
                    (_, 0) => ObjectStatusType::UnderConstruction,
                    (_, _) => ObjectStatusType::Sold,
                };
                if c != 0 {
                    sim.set_status(o, status);
                } else {
                    sim.clear_status(o, status);
                }
            }
        }
        19 => {
            if let Some(t) = live_cave(sim, s, a) {
                dispatch_cave(sim, t, |m, sim, ctx| {
                    m.recalc_apparent_controlling_player(sim, ctx)
                });
            }
        }
        20 => {
            if live_cave(sim, s, a).is_some() {
                let source = Some(CommandSourceType::FromScript);
                record_orders(sim, s, a, source, c, |m, sim, ctx| {
                    m.order_all_passengers_to_exit(sim, ctx, CommandSourceType::FromScript, FALSE);
                });
            }
        }
        21 => {
            if live_cave(sim, s, a).is_some() {
                record_orders(sim, s, a, None, c, |m, sim, ctx| {
                    m.order_all_passengers_to_idle(sim, ctx);
                });
            }
        }
        22 => {
            if let Some(u) = s.unit(a) {
                sim.set_object_team(u, TEAMS[b as usize]);
            }
        }
        _ => panic!("unknown op {code}"),
    }
}

// -----------------------------------------------------------------------------
// Vectors
// -----------------------------------------------------------------------------

fn fnv32(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |h, b| {
        (h ^ u32::from(b)).wrapping_mul(0x0100_0193)
    })
}

const SECTIONS: [&str; 5] = ["system", "modules", "objects", "queries", "misc"];

/// The whole `CaveSystem` snapshot, which cannot be saved with an unused index in it.
fn check_final_system(sim: &SimContext, want: &str) -> Result<(), String> {
    let mut system = sim.world.cave_system.clone();
    let mut xs = XferSave::new();
    let saved = Snapshot::xfer(&mut system, &mut xs);
    if want == "gap" {
        return match saved {
            Err(XferError::InvalidData) => Ok(()),
            other => Err(format!("system with a gap saved: {other:?}")),
        };
    }
    saved.map_err(|e| format!("system save: {e:?}"))?;
    let bytes = xs.into_bytes();

    // Version and count, then every tracker.
    let mut head = vec![1u8];
    head.extend_from_slice(&(system.len() as u16).to_le_bytes());
    let mut raw = hex(&head);
    let mut converted = raw.clone();
    for mut tracker in trackers(sim).into_iter().flatten() {
        let mut one = XferSave::new();
        Snapshot::xfer(&mut tracker, &mut one).map_err(|e| format!("{e:?}"))?;
        raw.push_str(&hex(&one.into_bytes()));
        converted.push_str(&tracker_hex(&mut tracker));
    }
    if hex(&bytes) != raw || converted != want {
        return Err(format!("final system {converted}"));
    }
    Ok(())
}

fn run_vector(
    sim: &mut SimContext,
    line: &str,
    local_player: Int,
    seed0: &[&str],
    dump: bool,
) -> Result<(), String> {
    let num = |key: &str| get_u32(line, key).unwrap_or_else(|| panic!("missing {key}"));
    let array = |key: &str| -> Vec<u32> {
        let values = get_i32_vec(line, key).unwrap_or_else(|| panic!("missing {key}"));
        values.into_iter().map(|v| v as u32).collect()
    };
    let p = array("p");
    let ops = array("ops");
    let want_rows = get_str(line, "rows").expect("rows");

    reset_world(sim, local_player);
    sim.world.frame = num("frame");
    sim.world.rng.init_random_with_seed(num("seed"));
    sim.rules.global_data.max_tunnel_capacity = num("cap") as Int;

    let mut s = Scene {
        caves: p[0] as usize,
        units: 0,
        slots: Vec::new(),
        ids: Vec::new(),
        live: Vec::new(),
        orders: 0,
    };
    let mut at = 1;
    for _ in 0..s.caves {
        let name = format!("P264_Cave{}", p[at]);
        spawn_at(
            sim,
            &mut s,
            &name,
            TEAMS[p[at + 1] as usize],
            p[at + 2],
            p[at + 3],
        );
        if p[at + 4] != 0 {
            sim.dispatch_on_build_complete(*s.slots.last().expect("cave"));
        }
        at += 5;
    }
    s.units = p[at] as usize;
    at += 1;
    let mut transports = Vec::new();
    for _ in 0..s.units {
        let kind = p[at] as usize;
        spawn_at(
            sim,
            &mut s,
            UNITS[kind],
            TEAMS[p[at + 1] as usize],
            p[at + 2],
            p[at + 3],
        );
        let slot = *s.slots.last().expect("slot");
        if p[at + 4] != 0 {
            hurt(sim, slot, p[at + 4], ObjectId::INVALID);
        }
        if kind == 4 || kind == 5 {
            sim.set_status(slot, ObjectStatusType::Stealthed);
        }
        if kind == 8 {
            transports.push(s.slots.len() - 1);
        }
        at += 5;
    }
    for k in transports {
        let carrier = s.slots[k];
        let team = sim.object(carrier).expect("carrier").team_id;
        spawn_at(sim, &mut s, "P264_Inf", team, 100, 100);
        let rider = *s.slots.last().expect("rider");
        sim.contain_add(carrier, rider);
    }
    sim.update_partition();

    // `InitRandom` after the setup, as the harness does.
    sim.world.rng.init_random_with_seed(num("seed"));

    let check = |sim: &mut SimContext, s: &Scene, row: usize, op: Option<&[u32]>| {
        let got = observe(sim, s);
        if dump {
            for text in &got {
                eprintln!("row {row}: {text}");
            }
        }
        let want = &want_rows[row * 40..row * 40 + 40];
        for (k, text) in got.iter().enumerate() {
            if format!("{:08x}", fnv32(text)) != want[k * 8..k * 8 + 8] {
                let reference = seed0
                    .get(row)
                    .and_then(|l| get_str(l, SECTIONS[k]))
                    .unwrap_or("(rerun both with PKT264_DUMP)");
                // Show a window around the first byte that differs.
                let at = text
                    .bytes()
                    .zip(reference.bytes())
                    .position(|(a, b)| a != b)
                    .unwrap_or_else(|| text.len().min(reference.len()));
                let window = |t: &str| {
                    let end = t.len().min(at + 100);
                    t[at.saturating_sub(60).min(end)..end].to_owned()
                };
                return Err(format!(
                    "row {row} (after op {op:?}) section {} at {at}:\n  got  {}\n  want {}",
                    SECTIONS[k],
                    window(text),
                    window(reference)
                ));
            }
        }
        Ok(())
    };

    assert_eq!(want_rows.len(), (ops.len() / 4 + 1) * 40, "row count");
    check(sim, &s, 0, None)?;
    for (i, op) in ops.chunks_exact(4).enumerate() {
        apply(sim, &mut s, [op[0], op[1], op[2], op[3]]);
        check(sim, &s, i + 1, Some(op))?;
    }

    if sim.world.rng.get_game_logic_random_seed_crc() != num("rng") {
        return Err("rng".to_owned());
    }
    check_final_system(sim, get_str(line, "final_system").expect("final_system"))?;
    let mut final_modules = String::new();
    let mut crc = 0u32;
    for i in 0..s.caves {
        let Some(slot) = s.get(i) else {
            continue;
        };
        let ms = contain_slot(sim, slot);
        final_modules.push_str(&hex(&save_module(sim, ms)));
        final_modules.push(' ');
        let mut xc = XferCrc::new();
        sim.world.modules.payload(ms).expect("payload").crc(&mut xc);
        crc = crc.wrapping_mul(31).wrapping_add(xc.get_crc());
    }
    if fnv32(&final_modules) != num("final_modules") {
        return Err(format!("final modules {final_modules}"));
    }
    if crc != num("crc") {
        return Err(format!("crc {crc} want {}", num("crc")));
    }
    Ok(())
}

#[test]
fn pkt264_matches_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT264_Cave.json");
    let local_player = get_u32(
        json.lines()
            .find(|l| l.starts_with("\"local_player\""))
            .map(|l| format!("{{{l}"))
            .expect("local_player")
            .as_str(),
        "local_player",
    )
    .expect("local_player") as Int;
    let seed0: Vec<&str> = json
        .lines()
        .filter(|l| l.starts_with("{\"row\":"))
        .collect();
    let team_ids: Vec<String> = TEAMS.iter().map(|t| t.0.to_string()).collect();
    assert!(json.contains(&format!("\"team_ids\": [{}]", team_ids.join(","))));
    let vectors = extract_vector_lines(json);
    assert_eq!(vectors.len(), 1000);
    let dump: Option<usize> = std::env::var("PKT264_DUMP")
        .ok()
        .and_then(|v| v.parse().ok());

    let mut sim = SimContext::new();
    sim.rules = build_rules();
    let mut failures = Vec::new();
    for (i, line) in vectors.iter().enumerate() {
        let reference: &[&str] = if i == 0 { &seed0 } else { &[] };
        if let Err(e) = run_vector(&mut sim, line, local_player, reference, dump == Some(i)) {
            failures.push(format!("vector {i}: {e}"));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} vectors differ; first:\n{}",
        failures.len(),
        vectors.len(),
        failures[0]
    );
}
