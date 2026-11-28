//! Four faithfulness fixes against vectors from the reference
//! (`tests/cpp_harness/gen_fixes_object_weapon.cpp`): weapon template angle, velocity and
//! duration parsing; `Object::canCrushOrSquish`; the special power pauses of
//! `setDisabledUntil` / `clearDisabled` with the academy count; and the weapon retiming of
//! `setWeaponBonusCondition` / `clearWeaponBonusCondition`.

use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Relationship, TeamId, FALSE, TRUE};
use zh_common::xfer::XferSave;
use zh_logic::arena::{FacetMask, ModuleSlot, ObjectSlot};
use zh_logic::gamelogic::frame::check_disabled_status;
use zh_logic::gamelogic::global_data::WeaponBonusConditionType;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::WeaponStatus;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

const VECTORS: &str = include_str!("../../../docs/golden_vectors/FIXES_ObjectWeapon.json");
const TEAM_A: TeamId = TeamId(1);
const TEAM_B: TeamId = TeamId(2);

/// The vector lines of one section, one per line.
fn section(name: &str) -> impl Iterator<Item = &'static str> {
    let start = VECTORS.find(&format!("\"{name}\":[")).expect("section");
    VECTORS[start..]
        .lines()
        .skip(1)
        .take_while(|l| l.starts_with('{'))
}

/// The strings of the array `"key":[..]`.
fn strings<'a>(line: &'a str, key: &str) -> Vec<&'a str> {
    let start = line.find(&format!("\"{key}\":[")).expect("array") + key.len() + 4;
    let rest = &line[start..];
    let body = &rest[..rest.find(']').expect("array end")];
    body.split('"').skip(1).step_by(2).collect()
}

fn numbers(text: &str) -> Vec<u32> {
    text.split(' ')
        .map(|t| t.parse().expect("number"))
        .collect()
}

// ---------------------------------------------------------------- weapon parsing

#[test]
fn weapon_template_angle_velocity_and_duration_fields() {
    let mut count = 0;
    for (v, line) in section("parse").enumerate() {
        let fields = get_str(line, "ini").expect("ini");
        let block = format!("Weapon P{v}\n  {}\nEnd", fields.replace('|', "\n  "));
        let mut t = WeaponTemplate::new();
        t.parse_ini_block(&block)
            .unwrap_or_else(|e| panic!("vector {v}: {e:?}\n{block}"));
        let got = [
            t.aim_delta.to_bits(),
            t.weapon_recoil.to_bits(),
            t.min_target_pitch.to_bits(),
            t.max_target_pitch.to_bits(),
            t.radius_damage_angle.to_bits(),
            t.weapon_speed.to_bits(),
            t.min_weapon_speed.to_bits(),
            t.clip_reload_time as u32,
            t.continuous_fire_coast_frames,
            t.auto_reload_when_idle_frames,
            t.pre_attack_delay as u32,
            t.suspend_fx_delay,
            t.min_delay_between_shots as u32,
            t.max_delay_between_shots as u32,
        ];
        let want = numbers(get_str(line, "out").expect("out"));
        assert_eq!(got.to_vec(), want, "vector {v}\n{block}");
        count += 1;
    }
    assert_eq!(count, 1000);
}

// ---------------------------------------------------------------- the world

const LEVELS: [u32; 5] = [0, 1, 2, 3, 255];

const WEAPONS: [&str; 3] = [
    "Weapon FX_WeaponA\n  ClipSize = 4\n  ClipReloadTime = 2000\n  DelayBetweenShots = 500\n  WeaponBonus = VETERAN RATE_OF_FIRE 150%\n  WeaponBonus = ELITE RATE_OF_FIRE 200%\n  WeaponBonus = HERO RATE_OF_FIRE 300%\n  WeaponBonus = PLAYER_UPGRADE DAMAGE 200%\nEnd",
    "Weapon FX_WeaponB\n  ClipSize = 0\n  ClipReloadTime = 700\n  DelayBetweenShots = Min:300 Max:900\n  WeaponBonus = ENTHUSIASTIC RATE_OF_FIRE 125%\n  WeaponBonus = VETERAN RATE_OF_FIRE 110%\nEnd",
    "Weapon FX_WeaponC\n  ClipSize = 2\n  ClipReloadTime = 1000\n  DelayBetweenShots = Min:100 Max:400\n  WeaponBonus = GARRISONED RATE_OF_FIRE 133%\n  WeaponBonus = HERO RATE_OF_FIRE 70%\nEnd",
];

fn object_ini() -> String {
    let body = |health: u32| {
        format!("  Body = ActiveBody ModuleTag_Body\n    MaxHealth = {health}\n    InitialHealth = {health}\n  End\n")
    };
    let mut s = String::new();
    for (c, crusher) in LEVELS.iter().enumerate() {
        for (v, crushable) in LEVELS.iter().enumerate() {
            let _ = write!(
                s,
                "Object FX_Crush{c}{v}\n  CrusherLevel = {crusher}\n  CrushableLevel = {crushable}\n{}End\n",
                body(100)
            );
        }
    }
    let _ = write!(
        s,
        "Object FX_Disable\n  KindOf = STRUCTURE DRONE\n  EnergyProduction = 10\n{}  \
         Behavior = SpecialAbility ModuleTag_A\n    SpecialPowerTemplate = FX_PowerSuper\n  End\n  \
         Behavior = SpecialAbility ModuleTag_B\n    SpecialPowerTemplate = FX_PowerOther\n  End\nEnd\n",
        body(100)
    );
    let gunners = [
        "    Weapon = PRIMARY FX_WeaponA\n    Weapon = SECONDARY FX_WeaponB\n    Weapon = TERTIARY FX_WeaponC\n",
        "    Weapon = PRIMARY FX_WeaponA\n    Weapon = SECONDARY FX_WeaponB\n    Weapon = TERTIARY FX_WeaponC\n    ShareWeaponReloadTime = Yes\n",
        "    Weapon = PRIMARY FX_WeaponC\n    Weapon = TERTIARY FX_WeaponB\n",
    ];
    for (i, set) in gunners.iter().enumerate() {
        let _ = write!(
            s,
            "Object FX_Gunner{i}\n  KindOf = INFANTRY\n  TransportSlotCount = 1\n{}  WeaponSet\n    Conditions = None\n{set}  End\nEnd\n",
            body(100)
        );
    }
    for (pass, yes) in ["No", "Yes"].iter().enumerate() {
        let _ = write!(
            s,
            "Object FX_Fort{pass}\n  KindOf = STRUCTURE\n  GeometryMajorRadius = 20\n  GeometryHeight = 20\n{}  \
             Behavior = GarrisonContain ModuleTag_Contain\n    ContainMax = 5\n    WeaponBonusPassedToPassengers = {yes}\n  End\nEnd\n",
            body(1000)
        );
    }
    s
}

fn context() -> SimContext {
    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    let powers = "SpecialPower FX_PowerSuper\n  Enum = SPECIAL_DAISY_CUTTER\n  ReloadTime = 3000\n  AcademyClassify = ACT_SUPERPOWER\nEnd\n\
                  SpecialPower FX_PowerOther\n  Enum = SPECIAL_CARPET_BOMB\n  ReloadTime = 1300\n  AcademyClassify = ACT_UPGRADE_RADAR\nEnd\n";
    rules
        .special_power_store
        .load_from_ini(
            &mut IniReader::new("sp.ini", powers.as_bytes(), IniLoadType::Overwrite),
            &rules.science_store,
            &mut rules.name_keys,
        )
        .expect("special powers");
    for block in WEAPONS {
        let mut w = WeaponTemplate::new();
        w.parse_ini_block(block).expect("weapon");
        rules.weapon_store.add_template(w);
    }
    let ini = object_ini();
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules = rules;
    sim
}

fn fresh_world(sim: &mut SimContext, frame: u32) {
    sim.world = SimWorld::new();
    sim.world.frame = frame;
    for pair in [(1, 2), (2, 1)] {
        sim.world.set_player_relation(pair.0, pair.1, Relationship::Enemies);
    }
}

fn spawn(sim: &mut SimContext, name: &str, team: TeamId) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object(tid, team, Coord3D::default(), 0.0)
}

// ---------------------------------------------------------------- crush

#[test]
fn object_can_crush_matches_the_reference() {
    let mut sim = context();
    let relationship = [
        Relationship::Enemies,
        Relationship::Neutral,
        Relationship::Allies,
    ];
    let (mut count, mut crushed) = (0, 0);
    for (v, line) in section("crush").enumerate() {
        let n = numbers(get_str(line, "in").expect("in"));
        let (ca, va, cb, vb, team_b, ab, ba, flags) =
            (n[0], n[1], n[2], n[3], n[4], n[5], n[6], n[7]);
        fresh_world(&mut sim, 1);
        sim.world.set_player_relation(1, 2, relationship[ab as usize]);
        sim.world.set_player_relation(2, 1, relationship[ba as usize]);
        let a = spawn(&mut sim, &format!("FX_Crush{ca}{va}"), TEAM_A);
        let team = if team_b == 1 { TEAM_A } else { TEAM_B };
        let b = spawn(&mut sim, &format!("FX_Crush{cb}{vb}"), team);
        for (bit, slot) in [(1, a), (2, b)] {
            if flags & bit != 0 {
                sim.set_disabled(slot, DisabledType::Unmanned);
            }
        }
        for (bit, slot) in [(4, a), (8, b)] {
            if flags & bit != 0 {
                sim.world
                    .objects
                    .get_mut(slot)
                    .expect("object")
                    .set_undetected_defector(TRUE);
            }
        }
        let got = format!(
            "{}{}",
            u8::from(sim.object_can_crush(a, b) != FALSE),
            u8::from(sim.object_can_crush(b, a) != FALSE)
        );
        assert_eq!(
            got,
            get_str(line, "out").expect("out"),
            "vector {v}: {line}"
        );
        crushed += got.matches('1').count();
        count += 1;
    }
    assert_eq!(count, 1500);
    assert!(crushed > 200, "{crushed}");
}

// ---------------------------------------------------------------- disable

const TYPES: [DisabledType; 8] = [
    DisabledType::Default,
    DisabledType::Hacked,
    DisabledType::Emp,
    DisabledType::Held,
    DisabledType::Paralyzed,
    DisabledType::Underpowered,
    DisabledType::Subdued,
    DisabledType::ScriptDisabled,
];

fn special_powers(sim: &SimContext, owner: ObjectSlot) -> Vec<ModuleSlot> {
    sim.world
        .objects
        .get(owner)
        .expect("object")
        .behaviors
        .iter()
        .filter(|(_, facets)| facets.contains(FacetMask::SPECIAL_POWER))
        .map(|&(ms, _)| ms)
        .collect()
}

fn disable_row(sim: &mut SimContext, owner: ObjectSlot, ret: u8) -> String {
    let powers = special_powers(sim, owner);
    let used = sim
        .world.players.player(sim.player_index(TEAM_A)).map(|p| &p.academy)
        .map_or(0, |a| a.special_powers_used);
    let power = sim.player_energy_production(TEAM_A);
    let record = sim.world.objects.get(owner).expect("object");
    let till: String = TYPES
        .iter()
        .map(|t| format!("{},", record.disabled_till_frame[t.as_i32() as usize]))
        .collect();
    let mut out = format!(
        "{} {till} {power} {used} {ret}",
        record.disabled_mask.to_unsigned_int()
    );
    for ms in powers {
        let (mut payload, _ctx, generation) = sim.world.modules.take_payload(ms).expect("payload");
        let mut xs = XferSave::new();
        payload.xfer(&mut xs, sim).expect("xfer save");
        sim.world.modules.restore_payload(ms, generation, payload);
        out.push(' ');
        for b in xs.as_bytes() {
            let _ = write!(out, "{b:02x}");
        }
    }
    out
}

#[test]
fn disabling_pauses_special_powers_per_type() {
    let mut sim = context();
    let mut count = 0;
    for (v, line) in section("disable").enumerate() {
        let mut frame = get_u32(line, "start").expect("start");
        fresh_world(&mut sim, frame);
        let o = spawn(&mut sim, "FX_Disable", TEAM_A);
        let ops = strings(line, "ops");
        let rows = strings(line, "rows");
        assert_eq!(rows.len(), ops.len() + 1, "vector {v}");
        assert_eq!(disable_row(&mut sim, o, 0), rows[0], "vector {v} spawn");
        for (k, op) in ops.iter().enumerate() {
            let n = numbers(op);
            let (code, kind, arg) = (n[0], n[1] as usize, n[2]);
            let mut ret = 0;
            match code {
                0 => sim.set_disabled(o, TYPES[kind]),
                1 => sim.set_disabled_until(o, TYPES[kind], arg),
                2 => ret = u8::from(sim.clear_disabled(o, TYPES[kind]) != FALSE),
                3 | 4 => {
                    frame += arg;
                    sim.world.frame = frame;
                    if code == 3 && sim.world.objects.get(o).expect("object").is_disabled() != FALSE
                    {
                        check_disabled_status(&mut sim, o);
                    }
                }
                _ => {
                    let ms = special_powers(&sim, o)[kind];
                    sim.dispatch_module(ms, |m, s, c| {
                        m.as_special_power_mut()
                            .expect("special power")
                            .do_special_power(s, c, 0);
                    })
                    .expect("dispatch");
                }
            }
            assert_eq!(
                disable_row(&mut sim, o, ret),
                rows[k + 1],
                "vector {v} op {k} ({op})"
            );
        }
        count += 1;
    }
    assert_eq!(count, 1000);
}

// ---------------------------------------------------------------- weapon bonus

const CONDITIONS: [WeaponBonusConditionType; 6] = [
    WeaponBonusConditionType::Garrisoned,
    WeaponBonusConditionType::PlayerUpgrade,
    WeaponBonusConditionType::Enthusiastic,
    WeaponBonusConditionType::Veteran,
    WeaponBonusConditionType::Elite,
    WeaponBonusConditionType::Hero,
];

const STATUSES: [WeaponStatus; 5] = [
    WeaponStatus::ReadyToFire,
    WeaponStatus::OutOfAmmo,
    WeaponStatus::BetweenFiringShots,
    WeaponStatus::ReloadingClip,
    WeaponStatus::PreAttack,
];

fn contained(sim: &SimContext, o: ObjectSlot) -> bool {
    sim.world
        .objects
        .get(o)
        .expect("object")
        .contained_by
        .is_some()
}

fn bonus_row(sim: &SimContext, gunner: ObjectSlot, fort: ObjectSlot) -> String {
    let g = sim.world.objects.get(gunner).expect("gunner");
    let f = sim.world.objects.get(fort).expect("fort");
    let mut out = format!(
        "{} {} {} {}",
        g.weapon_bonus_condition,
        f.weapon_bonus_condition,
        sim.world.rng.get_game_logic_random_seed_crc(),
        u8::from(g.contained_by.is_some())
    );
    for slot in &g.weapon_set.weapons {
        match slot {
            None => out.push_str(" -"),
            Some(w) => {
                let _ = write!(
                    out,
                    " {},{},{},{},{}",
                    w.status as u32,
                    w.ammo_in_clip,
                    w.when_we_can_fire_again,
                    w.when_pre_attack_finished,
                    w.when_last_reload_started
                );
            }
        }
    }
    out
}

#[test]
fn bonus_changes_retime_the_weapons() {
    let mut sim = context();
    let mut count = 0;
    for (v, line) in section("bonus").enumerate() {
        let setup = numbers(get_str(line, "setup").expect("setup"));
        let (kind, pass, seed, mut frame) = (setup[0], setup[1], setup[2], setup[3]);
        fresh_world(&mut sim, frame);
        let gunner = spawn(&mut sim, &format!("FX_Gunner{kind}"), TEAM_A);
        let fort = spawn(&mut sim, &format!("FX_Fort{pass}"), TEAM_A);
        sim.world.rng.init_random_with_seed(seed);
        let ops = strings(line, "ops");
        let rows = strings(line, "rows");
        assert_eq!(rows.len(), ops.len() + 1, "vector {v}");
        assert_eq!(bonus_row(&sim, gunner, fort), rows[0], "vector {v} spawn");
        for (k, op) in ops.iter().enumerate() {
            let n = numbers(op);
            let (code, a, b, c, d, e) = (n[0], n[1], n[2], n[3], n[4], n[5]);
            match code {
                0 => sim.set_weapon_bonus_condition(gunner, CONDITIONS[a as usize]),
                1 => sim.clear_weapon_bonus_condition(gunner, CONDITIONS[a as usize]),
                2 if b != 0 => sim.clear_weapon_bonus_condition(fort, CONDITIONS[a as usize]),
                2 => sim.set_weapon_bonus_condition(fort, CONDITIONS[a as usize]),
                3 => {
                    let record = sim.world.objects.get_mut(gunner).expect("gunner");
                    if let Some(w) = record.weapon_set.weapons[a as usize].as_mut() {
                        w.status = STATUSES[b as usize];
                        w.ammo_in_clip = c;
                        w.when_we_can_fire_again = d;
                        w.when_pre_attack_finished = e;
                    }
                }
                4 => {
                    frame += a;
                    sim.world.frame = frame;
                }
                5 => {
                    if !contained(&sim, gunner) {
                        sim.contain_add(fort, gunner);
                    }
                }
                _ => {
                    if contained(&sim, gunner) {
                        sim.contain_remove(fort, gunner, FALSE);
                    }
                }
            }
            assert_eq!(
                bonus_row(&sim, gunner, fort),
                rows[k + 1],
                "vector {v} op {k} ({op})"
            );
        }
        count += 1;
    }
    assert_eq!(count, 1000);
}
