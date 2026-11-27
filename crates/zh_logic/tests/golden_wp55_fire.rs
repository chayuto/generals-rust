//! WP-5.5: the weapon firing state machine, its module callers and `PKT-329` `FireWeaponUpdate`,
//! against vectors recorded from `Weapon.cpp` (`tests/cpp_harness/gen_wp55_fire.cpp`).

use std::fmt::Write as _;
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord2D, Coord3D, Matrix3D};
use zh_common::types::{Bool, Real, TeamId, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::modules::contain::open_contain::kill_object;
use zh_logic::object::thing::ThingTerrainAccess;
use zh_logic::object::weapon::{Weapon, WeaponReloadType, WeaponTemplate};
use zh_logic::object::weapon_fire;
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_logic::pathfinder::PathfindLayerEnum;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::sim::{FlatTerrain, RulesRegistry, SimApi, SimContext};

const WEAPON_NAMES: [&str; 8] = [
    "WF_STATE", "WF_SET0", "WF_SET1", "WF_SET2", "WF_MOD", "WF_CONT0", "WF_CONT1", "WF_REACT",
];
const MODULE_NAMES: [&str; 4] = [
    "FireWeaponUpdate",
    "FireWeaponWhenDeadBehavior",
    "FireWeaponWhenDamagedBehavior",
    "FireWeaponCollide",
];
const NEUTRAL: TeamId = TeamId(1);
const TEAM_A: TeamId = TeamId(2);
const TEAM_B: TeamId = TeamId(3);

// ----------------------------------------------------------------------------- JSON

#[derive(Debug)]
enum Json {
    Num(u32),
    Str(String),
    List(Vec<Json>),
    Map(Vec<(String, Json)>),
}

impl Json {
    fn get(&self, key: &str) -> &Json {
        match self {
            Json::Map(m) => &m.iter().find(|(k, _)| k == key).expect(key).1,
            _ => panic!("not a map: {key}"),
        }
    }
    fn list(&self) -> &[Json] {
        match self {
            Json::List(l) => l,
            _ => panic!("not a list"),
        }
    }
    fn num(&self) -> u32 {
        match self {
            Json::Num(n) => *n,
            _ => panic!("not a number"),
        }
    }
    fn nums(&self) -> Vec<u32> {
        self.list().iter().map(Json::num).collect()
    }
    fn str(&self) -> &str {
        match self {
            Json::Str(s) => s,
            _ => panic!("not a string"),
        }
    }
}

fn parse_json(b: &[u8], i: &mut usize) -> Json {
    let skip = |i: &mut usize| {
        while b[*i].is_ascii_whitespace() || b[*i] == b',' || b[*i] == b':' {
            *i += 1;
        }
    };
    skip(i);
    match b[*i] {
        open @ (b'[' | b'{') => {
            *i += 1;
            let mut list = Vec::new();
            let mut map = Vec::new();
            loop {
                skip(i);
                if b[*i] == b']' || b[*i] == b'}' {
                    *i += 1;
                    break;
                }
                if open == b'{' {
                    let key = parse_json(b, i).str().to_owned();
                    map.push((key, parse_json(b, i)));
                } else {
                    list.push(parse_json(b, i));
                }
            }
            if open == b'{' {
                Json::Map(map)
            } else {
                Json::List(list)
            }
        }
        b'"' => {
            let start = *i + 1;
            let end = start + b[start..].iter().position(|&c| c == b'"').expect("quote");
            *i = end + 1;
            Json::Str(String::from_utf8(b[start..end].to_vec()).expect("utf8"))
        }
        _ => {
            let mut n = 0u32;
            while b[*i].is_ascii_digit() {
                n = n * 10 + u32::from(b[*i] - b'0');
                *i += 1;
            }
            Json::Num(n)
        }
    }
}

fn load_golden() -> Json {
    let text = include_str!("../../../docs/golden_vectors/WP55_FIRE.json");
    parse_json(text.as_bytes(), &mut 0)
}

// ----------------------------------------------------------------------------- fixture

struct SlopeTerrain;

impl ThingTerrainAccess for SlopeTerrain {
    fn get_ground_height(&self, x: Real, y: Real, _normal: Option<&mut Coord3D>) -> Real {
        x * 0.25 + y * 0.125
    }
    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        FlatTerrain.align_on_terrain(angle, pos, stick_to_ground, mtx)
    }
    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        FlatTerrain.is_underwater(x, y, water_z, terrain_z)
    }
}

fn percent(pct: u32) -> Real {
    IniReader::scan_percent_to_real(&format!("{pct}%")).expect("percent")
}

/// `[clip, reload, min, max, spb, reload type, disarm, leech, scalar, suspend, pitch,
/// n, (x, y)*n, m, (condition, field, percent)*m]`.
fn template(name: &str, p: &[u32]) -> WeaponTemplate {
    let mut t = WeaponTemplate::default();
    t.name = name.to_owned();
    t.clip_size = p[0] as i32;
    t.clip_reload_time = p[1] as i32;
    t.min_delay_between_shots = p[2] as i32;
    t.max_delay_between_shots = p[3] as i32;
    t.shots_per_barrel = p[4] as i32;
    t.reload_type = WeaponReloadType::from_i32(p[5] as i32).expect("reload type");
    if p[6] != 0 {
        t.damage_type = DamageType::Disarm;
    }
    t.leech_range_weapon = if p[7] != 0 { TRUE } else { FALSE };
    t.scatter_target_scalar = Real::from_bits(p[8]);
    t.suspend_fx_delay = p[9];
    if p[10] != 0 {
        t.min_target_pitch = -0.5;
    }
    let n = p[11] as usize;
    for i in 0..n {
        t.scatter_targets.push(Coord2D {
            x: Real::from_bits(p[12 + 2 * i]),
            y: Real::from_bits(p[13 + 2 * i]),
        });
    }
    let mut at = 12 + 2 * n;
    let m = p[at] as usize;
    at += 1;
    for _ in 0..m {
        set_bonus(&mut t.extra_bonus.entries[p[at] as usize], p[at + 1], p[at + 2]);
        at += 3;
    }
    t
}

fn set_bonus(bonus: &mut zh_logic::gamelogic::global_data::WeaponBonus, field: u32, pct: u32) {
    use zh_logic::gamelogic::global_data::WeaponBonusField;
    let field = WeaponBonusField::from_i32(field as i32).expect("bonus field");
    bonus.set_field(field, percent(pct));
}

fn shooter_ini(kind: u32, variant: u32, shared: u32, update_variants: &[Json]) -> (String, String) {
    let mut s = String::new();
    let mut unit = |name: &str, modules: &str, shared: bool| {
        let _ = write!(
            s,
            "Object {name}\n  KindOf = SELECTABLE\n  WeaponSet\n    Conditions = None\n    \
             Weapon = PRIMARY WF_SET0\n    Weapon = SECONDARY WF_SET1\n    \
             Weapon = TERTIARY WF_SET2\n"
        );
        if shared {
            s.push_str("    ShareWeaponReloadTime = Yes\n");
        }
        s.push_str("  End\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n");
        s.push_str(modules);
        s.push_str("End\n");
    };
    let (name, modules) = match kind {
        3 => {
            // INI durations are milliseconds; 30 logic frames per second.
            let v = update_variants[variant as usize].nums();
            let ms = |frames: u32| frames * 1000 / 30;
            (
                "WF_Update",
                format!(
                    "  Behavior = FireWeaponUpdate ModuleTag_M\n    Weapon = WF_MOD\n    InitialDelay = {}\n    ExclusiveWeaponDelay = {}\n  End\n",
                    ms(v[0]),
                    ms(v[1])
                ),
            )
        }
        4 => ("WF_Dead", "  Behavior = FireWeaponWhenDeadBehavior ModuleTag_M\n    DeathWeapon = WF_MOD\n    StartsActive = Yes\n  End\n".to_owned()),
        5 => ("WF_Damaged", "  Behavior = FireWeaponWhenDamagedBehavior ModuleTag_M\n    StartsActive = Yes\n    ReactionWeaponPristine = WF_REACT\n    ReactionWeaponDamaged = WF_REACT\n    ContinuousWeaponPristine = WF_CONT0\n    ContinuousWeaponDamaged = WF_CONT1\n  End\n".to_owned()),
        6 => ("WF_Collide", "  Behavior = FireWeaponCollide ModuleTag_M\n    CollideWeapon = WF_MOD\n    FireOnce = No\n  End\n".to_owned()),
        _ => ("WF_Shooter", String::new()),
    };
    unit(name, &modules, shared != 0);
    unit("WF_Victim", "", false);
    s.push_str(
        "Object WF_Box\n  KindOf = SELECTABLE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100.0\n    InitialHealth = 100.0\n  End\n  Behavior = TransportContain ModuleTag_C\n    Slots = 4\n    WeaponBonusPassedToPassengers = Yes\n  End\nEnd\n",
    );
    (name.to_owned(), s)
}

fn build_rules(golden: &Json, v: &Json) -> (RulesRegistry, String) {
    let head = v.get("head").nums();
    let mut rules = RulesRegistry::new();
    rules.global_data.unit_damaged_thresh = 0.7;
    rules.global_data.unit_really_damaged_thresh = 0.35;
    let set = rules
        .global_data
        .weapon_bonus_set
        .as_mut()
        .expect("bonus set");
    for line in golden.get("global_bonus").list() {
        let l = line.nums();
        set_bonus(&mut set.entries[l[0] as usize], l[1], l[2]);
    }
    for (i, t) in v.get("tmpl").list().iter().enumerate() {
        rules
            .weapon_store
            .add_template(template(WEAPON_NAMES[i], &t.nums()));
    }
    let (name, text) = shooter_ini(head[0], head[2], head[1], golden.get("update_variants").list());
    let mut ini = IniReader::new("object.ini", text.as_bytes(), IniLoadType::Overwrite);
    if let Err(e) = rules.load_thing_templates_from_ini(&mut ini) {
        panic!("load thing templates at line {}: {e:?}", ini.get_line_num());
    }
    (rules, name)
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object_with_status(tid, team, pos, 0.0, ObjectStatusMaskType::new())
}

// ----------------------------------------------------------------------------- state capture

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn snap_weapon(w: &mut Weapon) -> (String, u32) {
    let mut xs = XferSave::new();
    w.xfer(&mut xs).expect("xfer save");
    let mut xc = XferCrc::new();
    w.crc(&mut xc);
    (hex(xs.as_bytes()), xc.get_crc())
}

fn snap_module(sim: &mut SimContext, slot: ModuleSlot) -> (String, u32) {
    let (mut payload, _ctx, generation) =
        sim.world.modules.take_payload(slot).expect("take payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    let mut xc = XferCrc::new();
    payload.crc(&mut xc);
    sim.world.modules.restore_payload(slot, generation, payload);
    (hex(xs.as_bytes()), xc.get_crc())
}

struct Shooter {
    owner: ObjectSlot,
    victim: ObjectSlot,
    container: ObjectSlot,
    module: Option<ModuleSlot>,
    standalone: Option<Weapon>,
    kind: u32,
}

impl Shooter {
    /// Runs `f` on the standalone weapon, or on a copy of a set weapon which is written back.
    fn with_weapon<R>(
        &mut self,
        sim: &mut SimContext,
        slot: u32,
        f: impl FnOnce(&mut Weapon, &mut SimContext) -> R,
    ) -> R {
        if let Some(w) = self.standalone.as_mut() {
            return f(w, sim);
        }
        let mut w = set_weapon(sim, self.owner, slot);
        let out = f(&mut w, sim);
        sim.world
            .objects
            .get_mut(self.owner)
            .expect("owner")
            .weapon_set
            .weapons[slot as usize] = Some(w);
        out
    }
}

fn set_weapon(sim: &SimContext, owner: ObjectSlot, slot: u32) -> Weapon {
    sim.world
        .objects
        .get(owner)
        .expect("owner")
        .weapon_set
        .weapons[slot as usize]
        .clone()
        .expect("set weapon")
}

/// One row: `(return value, rng CRC, snapshots, CRCs, shots)`.
type Row = (u32, u32, Vec<String>, Vec<u32>, Vec<[u32; 5]>);

fn record(sim: &mut SimContext, s: &mut Shooter, ret: u32) -> Row {
    let mut snaps = Vec::new();
    if let Some(w) = s.standalone.as_mut() {
        snaps.push(snap_weapon(w));
    }
    let slots = if s.kind == 1 { 3 } else { 1 };
    for slot in 0..slots {
        snaps.push(snap_weapon(&mut set_weapon(sim, s.owner, slot)));
    }
    if let Some(m) = s.module {
        snaps.push(snap_module(sim, m));
    }
    let victim_id = sim.world.objects.id_of(s.victim);
    let shots = std::mem::take(&mut sim.world.weapon_fires)
        .into_iter()
        .map(|(_, name, target, pos)| {
            let weapon = WEAPON_NAMES.iter().position(|n| *n == name).unwrap_or(8) as u32;
            if target.is_some() {
                assert_eq!(target, victim_id, "shot at an unexpected object");
            }
            [
                weapon,
                u32::from(target.is_some()),
                pos.x.to_bits(),
                pos.y.to_bits(),
                pos.z.to_bits(),
            ]
        })
        .collect();
    let (xfer, crc) = snaps.into_iter().unzip();
    (
        ret,
        sim.world.rng.get_game_logic_random_seed_crc(),
        xfer,
        crc,
        shots,
    )
}

fn expected_row(r: &Json) -> Row {
    let head = r.get("r").nums();
    (
        head[0],
        head[1],
        r.get("s").list().iter().map(|s| s.str().to_owned()).collect(),
        r.get("c").nums(),
        r.get("f")
            .list()
            .iter()
            .map(|f| <[u32; 5]>::try_from(f.nums()).expect("shot"))
            .collect(),
    )
}

// ----------------------------------------------------------------------------- operations

fn apply(sim: &mut SimContext, s: &mut Shooter, op: &[u32], frame: &mut u32) -> u32 {
    let (code, a, b, c) = (op[0], op[1], op[2], op[3]);
    let owner = s.owner;
    let victim = s.victim;
    sim.world.frame = *frame;
    let target = Coord3D::new(Real::from_bits(b), Real::from_bits(c), 7.5);
    match code {
        0 => {
            *frame = frame.wrapping_add(a);
            sim.world.frame = *frame;
        }
        1 => s.with_weapon(sim, a, |w, sim| weapon_fire::load_ammo_now(w, sim, owner)),
        2 => s.with_weapon(sim, a, |w, sim| weapon_fire::reload_ammo(w, sim, owner)),
        3 => {
            return u32::from(s.with_weapon(sim, a, |w, sim| {
                weapon_fire::fire_weapon_at_position(w, sim, owner, &target)
            }))
        }
        4 => {
            return u32::from(s.with_weapon(sim, a, |w, sim| {
                weapon_fire::fire_weapon_at_object(w, sim, owner, victim)
            }))
        }
        5 => s.with_weapon(sim, a, |w, sim| {
            weapon_fire::force_fire_weapon(w, sim, owner, &target);
        }),
        6 => {
            let now = *frame;
            return s.with_weapon(sim, a, |w, _| w.get_status(now).as_i32() as u32);
        }
        7 => {
            let now = *frame;
            let allow = if c != 0 { TRUE } else { FALSE };
            s.with_weapon(sim, a, |w, _| {
                w.set_clip_percent_full(Real::from_bits(b), allow, now)
            });
        }
        8 => sim.world.objects.get_mut(owner).expect("owner").weapon_bonus_condition = a,
        9 => {
            let container = s.container;
            sim.world.objects.get_mut(owner).expect("owner").contained_by =
                (a != 0).then_some(container);
            sim.world
                .objects
                .get_mut(container)
                .expect("container")
                .weapon_bonus_condition = b;
        }
        10 | 11 => {
            let name = WEAPON_NAMES.get(a as usize).copied().unwrap_or("WF_NONE");
            if code == 10 {
                weapon_fire::create_and_fire_temp_weapon(sim, owner, name, &target);
            } else {
                weapon_fire::create_and_fire_temp_weapon_at_object(sim, owner, name, victim);
            }
        }
        12 => {
            *frame = frame.wrapping_add(a);
            sim.world.frame = *frame;
            return sim
                .dispatch_module(s.module.expect("module"), |m, sim, ctx| {
                    m.as_update_mut().expect("update").update(sim, ctx).0
                })
                .expect("dispatch");
        }
        13 => {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Unresistable;
            dmg.input.death_type = DeathType::Normal;
            dmg.input.amount = Real::from_bits(a);
            dmg.input.source_id = Default::default();
            sim.attempt_damage(owner, &mut dmg);
        }
        14 => kill_object(sim, owner, DamageType::Unresistable, DeathType::Normal),
        15 => {
            sim.dispatch_module(s.module.expect("module"), |m, sim, ctx| {
                m.as_collide_mut().expect("collide").on_collide(
                    sim,
                    ctx,
                    Some(victim),
                    None,
                    None,
                );
            })
            .expect("dispatch");
        }
        16 => {
            let status = &mut sim.world.objects.get_mut(owner).expect("owner").status;
            status.set(ObjectStatusType::UnderConstruction as i32, i32::from(a != 0));
        }
        other => panic!("unknown op {other}"),
    }
    0
}

fn run(golden: &Json, v: &Json) -> Option<String> {
    let head = v.get("head").nums();
    let (kind, frame0, seed) = (head[0], head[3], head[4]);
    let (rules, name) = build_rules(golden, v);
    let mut sim = SimContext::new();
    sim.rules = rules;
    sim.terrain = Box::new(SlopeTerrain);
    for (player, team) in [NEUTRAL, TEAM_A, TEAM_B].into_iter().enumerate() {
        sim.world.team_players.insert(team, player as i32);
        sim.world.player_default_teams.insert(player as i32, team);
    }
    sim.world.team_is_neutral.insert(NEUTRAL, TRUE);
    sim.world.neutral_team_id = NEUTRAL;
    sim.world.frame = frame0;

    let at = Coord3D::new(
        Real::from_bits(head[8]),
        Real::from_bits(head[9]),
        Real::from_bits(head[10]),
    );
    let there = Coord3D::new(
        Real::from_bits(head[11]),
        Real::from_bits(head[12]),
        Real::from_bits(v.get("victim_z").num()),
    );
    // Barrel counts come from the drawable, so they are set before any weapon fires.
    let owner = spawn(&mut sim, &name, TEAM_A, at);
    sim.world
        .objects
        .get_mut(owner)
        .expect("owner")
        .weapon_barrel_count = [head[5] as i32, head[6] as i32, head[7] as i32];
    let victim = spawn(&mut sim, "WF_Victim", TEAM_B, there);
    let container = spawn(&mut sim, "WF_Box", TEAM_A, at);
    // The harness places objects without terrain alignment.
    for (slot, pos) in [(owner, at), (victim, there)] {
        let mut reactor = zh_logic::object::thing::NoopTransformReactor;
        let mut mtx = Matrix3D::identity();
        mtx.set_translation(&pos);
        sim.world
            .objects
            .get_mut(slot)
            .expect("object")
            .thing
            .set_transform_matrix(&mtx, &mut reactor);
    }

    let module = (kind >= 3).then(|| {
        let key = sim.rules.name_keys.name_to_key(MODULE_NAMES[kind as usize - 3]);
        sim.world
            .objects
            .get(owner)
            .expect("owner")
            .behaviors
            .iter()
            .map(|&(ms, _)| ms)
            .find(|&ms| {
                sim.world
                    .modules
                    .payload(ms)
                    .is_some_and(|p| p.module_name_key() == key)
            })
            .expect("module")
    });
    let standalone = (kind == 0).then(|| {
        let t = sim.rules.weapon_store.find_weapon_template("WF_STATE").expect("template");
        Weapon::new(t.clone(), WeaponSlotType::Primary, frame0)
    });
    let mut s = Shooter {
        owner,
        victim,
        container,
        module,
        standalone,
        kind,
    };

    sim.world.rng.init_random_with_seed(seed);
    // The oracle intercepts `fireWeaponTemplate`; `golden_pkt399_weapon_effects` runs it.
    sim.world.weapon_templates_stubbed = true;
    sim.world.weapon_fires.clear();
    let ops = v.get("ops").list();
    let rows = v.get("rows").list();
    assert_eq!(rows.len(), ops.len() + 1);
    let mut frame = frame0;
    for (step, expected) in rows.iter().enumerate() {
        let ret = match step.checked_sub(1) {
            Some(i) => apply(&mut sim, &mut s, &ops[i].nums(), &mut frame),
            None => 0,
        };
        let actual = record(&mut sim, &mut s, ret);
        let expected = expected_row(expected);
        if actual != expected {
            let op = step.checked_sub(1).map(|i| ops[i].nums());
            return Some(format!(
                "step {step} op {op:?} head {head:?}\n  expected {expected:?}\n  actual   {actual:?}"
            ));
        }
    }
    None
}

// ----------------------------------------------------------------------------- tests

#[test]
fn wp55_default_template_matches_constructor() {
    let d = WeaponTemplate::default();
    let f = Real::to_bits;
    let b = |x: Bool| u32::from(x);
    let actual = vec![
        f(d.primary_damage),
        f(d.primary_damage_radius),
        f(d.secondary_damage),
        f(d.secondary_damage_radius),
        f(d.attack_range),
        f(d.minimum_attack_range),
        f(d.request_assist_range),
        f(d.aim_delta),
        f(d.scatter_radius),
        f(d.scatter_target_scalar),
        f(d.shock_wave_amount),
        f(d.shock_wave_radius),
        f(d.shock_wave_taper_off),
        d.damage_type as u32,
        d.death_type as u32,
        f(d.weapon_speed),
        f(d.min_weapon_speed),
        b(d.scale_weapon_speed),
        f(d.weapon_recoil),
        f(d.min_target_pitch),
        f(d.max_target_pitch),
        f(d.radius_damage_angle),
        d.clip_size as u32,
        d.clip_reload_time as u32,
        d.min_delay_between_shots as u32,
        d.max_delay_between_shots as u32,
        d.continuous_fire_one_shots_needed as u32,
        d.continuous_fire_two_shots_needed as u32,
        d.continuous_fire_coast_frames,
        d.auto_reload_when_idle_frames,
        d.shots_per_barrel as u32,
        b(d.damage_dealt_at_self_position),
        d.reload_type as u32,
        d.prefire_type as u32,
        d.pre_attack_delay as u32,
        f(d.continue_attack_range),
        d.affects_mask as u32,
        d.collide_mask as u32,
        d.anti_mask as u32,
        b(d.leech_range_weapon),
        b(d.capable_of_following_waypoint),
        b(d.is_shows_ammo_pips),
        b(d.allow_attack_garrisoned_buildings),
        b(d.play_fx_when_stealthed),
        b(d.missile_calls_on_die),
        d.suspend_fx_delay,
        f(d.scatter_radius_vs_infantry),
    ];
    assert_eq!(actual, load_golden().get("default_template").nums());
}

#[test]
fn wp55_fire_matches_reference() {
    let golden = load_golden();
    let vectors = golden.get("vectors").list();
    assert!(vectors.len() >= 1000);
    assert_eq!(vectors.len() as u32, golden.get("vector_count").num());

    let mut failed = [0u32; 7];
    let mut first: [Option<(usize, String)>; 7] = Default::default();
    let mut shots = 0u32;
    for (i, v) in vectors.iter().enumerate() {
        shots += v
            .get("rows")
            .list()
            .iter()
            .map(|r| r.get("f").list().len() as u32)
            .sum::<u32>();
        let kind = v.get("head").nums()[0] as usize;
        if let Some(why) = run(&golden, v) {
            failed[kind] += 1;
            first[kind].get_or_insert((i, why));
        }
    }
    assert_eq!(shots, golden.get("shot_count").num());
    let report: Vec<String> = first
        .iter()
        .flatten()
        .map(|(i, why)| format!("vector {i}: {why}"))
        .collect();
    assert_eq!(failed, [0; 7], "failures per kind\n{}", report.join("\n"));
}
