//! `PKT-358` `RadarUpdate` and `PKT-351` `PowerPlantUpdate` against vectors from the
//! reference modules (`tests/cpp_harness/gen_pkt358_351_radar_power.cpp`).
//!
//! The upgrade modules that call these two are checked here without a reference.

use std::fmt::Write as _;
use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{TeamId, UpdateSleepTime};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{ModuleSlot, ObjectSlot};
use zh_logic::modules::update::power_plant_update::PowerPlantUpdate;
use zh_logic::modules::update::radar_update::RadarUpdate;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::scheduler::UPDATE_SLEEP_FOREVER;
use zh_logic::sim::{RulesRegistry, SimApi, SimContext, SimView, SimWorld};

// `None` leaves the field out, giving the default.
const TIMES: [Option<&str>; 7] = [
    Some("0"),
    Some("1"),
    Some("100"),
    Some("599"),
    Some("1000"),
    Some("4000"),
    None,
];
const TEAM: TeamId = TeamId(1);

fn fixture() -> String {
    let mut s = String::new();
    for (i, t) in TIMES.iter().enumerate() {
        let line = |field: &str| t.map_or_else(String::new, |t| format!("    {field} = {t}\n"));
        let _ = write!(
            s,
            "Object P358_Radar{i}\n  KindOf = PRELOAD STRUCTURE\n  Body = InactiveBody ModuleTag_Body\n  End\n  Behavior = RadarUpdate ModuleTag_Radar\n{}  End\nEnd\n\
             Object P351_Power{i}\n  KindOf = PRELOAD STRUCTURE\n  Body = InactiveBody ModuleTag_Body\n  End\n  Behavior = PowerPlantUpdate ModuleTag_Rods\n{}  End\nEnd\n",
            line("RadarExtendTime"),
            line("RodsExtendTime")
        );
    }
    s
}

fn context(ini: &str) -> SimContext {
    let mut sim = SimContext::new();
    let mut rules = RulesRegistry::new();
    let mut reader = IniReader::new("object.ini", ini.as_bytes(), IniLoadType::Overwrite);
    rules
        .load_thing_templates_from_ini(&mut reader)
        .expect("templates");
    sim.rules = rules;
    sim
}

fn spawn(sim: &mut SimContext, name: &str) -> ObjectSlot {
    let tid = sim
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    sim.spawn_object(tid, TEAM, Coord3D::default(), 0.0)
}

fn module_named(sim: &mut SimContext, owner: ObjectSlot, name: &str) -> ModuleSlot {
    let key = sim.rules.name_keys.name_to_key(name);
    let obj = sim.world.objects.get(owner).expect("object");
    obj.behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .find(|&ms| {
            sim.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
        .unwrap_or_else(|| panic!("missing module {name}"))
}

fn save(sim: &mut SimContext, slot: ModuleSlot) -> Vec<u8> {
    let (mut payload, _ctx, generation) = sim.world.modules.take_payload(slot).expect("payload");
    let mut xs = XferSave::new();
    payload.xfer(&mut xs, sim).expect("xfer save");
    sim.world.modules.restore_payload(slot, generation, payload);
    xs.as_bytes().to_vec()
}

fn next_call_frame(sim: &SimContext, slot: ModuleSlot) -> u32 {
    sim.module_schedule(slot)
        .expect("schedule")
        .next_call_frame()
}

/// The harness's row for one module.
fn row(sim: &mut SimContext, power: bool, o: ObjectSlot, m: ModuleSlot, ret: u32) -> String {
    let bytes = save(sim, m);
    let mut xc = XferCrc::new();
    sim.world.modules.payload(m).expect("payload").crc(&mut xc);
    let flags = if power {
        [
            ModelConditionFlagType::PowerPlantUpgrading,
            ModelConditionFlagType::PowerPlantUpgraded,
        ]
    } else {
        [
            ModelConditionFlagType::RadarExtending,
            ModelConditionFlagType::RadarUpgraded,
        ]
    };
    let obj = sim.world.objects.get(o).expect("object");
    let bits: String = flags
        .iter()
        .map(|f| {
            if obj.model_condition.test(f.as_i32()) != 0 {
                '1'
            } else {
                '0'
            }
        })
        .collect();
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!(
        "{hex} {} {bits} {ret} {}",
        xc.get_crc(),
        next_call_frame(sim, m)
    )
}

/// The harness's `step`.
fn step(sim: &mut SimContext, m: ModuleSlot, frame: u32) {
    sim.world.frame = frame;
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

fn between<'a>(line: &'a str, start: &str, end: &str) -> &'a str {
    let rest = &line[line.find(start).expect(start) + start.len()..];
    &rest[..rest.find(end).expect(end)]
}

#[test]
fn radar_and_power_plant_match_reference() {
    let json = include_str!("../../../docs/golden_vectors/PKT358_351_RadarPower.json");
    let lines: Vec<&str> = json
        .lines()
        .map(|l| l.trim_end_matches(','))
        .filter(|l| l.starts_with("{\"kind\""))
        .collect();
    assert_eq!(lines.len(), 1000);
    let mut sim = context(&fixture());
    let mut checked = 0;
    for (index, line) in lines.iter().enumerate() {
        let power = get_u32(line, "kind").expect("kind") == 1;
        let variant = get_u32(line, "variant").expect("variant");
        let mut frame = get_u32(line, "frame").expect("frame");
        sim.world = SimWorld::new();
        sim.world.frame = frame;
        let (name, module) = if power {
            (format!("P351_Power{variant}"), "PowerPlantUpdate")
        } else {
            (format!("P358_Radar{variant}"), "RadarUpdate")
        };
        let o = spawn(&mut sim, &name);
        let m = module_named(&mut sim, o, module);
        assert_eq!(
            row(&mut sim, power, o, m, 0),
            get_str(line, "spawn").expect("spawn"),
            "vector {index} spawn"
        );

        let ops = between(line, "\"ops\":[", "],\"rows\"");
        let numbers: Vec<u32> = ops
            .split(['[', ']', ','])
            .filter(|t| !t.is_empty())
            .map(|t| t.parse().expect("op number"))
            .collect();
        let rows: Vec<&str> = between(line, "\"rows\":[\"", "\"]}")
            .split("\",\"")
            .collect();
        assert_eq!(numbers.len(), rows.len() * 2, "vector {index}");
        for (k, (op, expected)) in numbers.chunks(2).zip(&rows).enumerate() {
            let (code, a) = (op[0], op[1]);
            let mut ret = 0;
            sim.world.frame = frame;
            match code {
                0 if power => sim.extend_power_plant(o),
                0 | 1 if !power => sim.extend_radar_update(o),
                1 => sim.retract_power_plant(o),
                2 => {
                    for _ in 0..a {
                        frame += 1;
                        step(&mut sim, m, frame);
                    }
                }
                3 => {
                    ret = sim
                        .dispatch_module(m, |b, s, c| {
                            b.as_update_mut().expect("update").update(s, c).0
                        })
                        .expect("dispatch");
                }
                _ => panic!("op {code}"),
            }
            assert_eq!(
                row(&mut sim, power, o, m, ret),
                *expected,
                "vector {index} op {k} ({code},{a})"
            );
            checked += 1;
        }
    }
    assert!(checked > 5000, "only {checked} rows");
}

#[test]
fn saved_state_loads_back() {
    let mut sim = context(&fixture());
    // The last bytes of a save are the module's own fields; the header before them
    // holds the live schedule, which a load alone does not change.
    for (name, module, own) in [
        ("P358_Radar4", "RadarUpdate", 6),
        ("P351_Power4", "PowerPlantUpdate", 1),
    ] {
        sim.world = SimWorld::new();
        sim.world.frame = 50;
        let o = spawn(&mut sim, name);
        let m = module_named(&mut sim, o, module);
        let fresh = save(&mut sim, m);
        sim.extend_radar_update(o);
        sim.extend_power_plant(o);
        let changed = save(&mut sim, m);
        let tail = |bytes: &[u8]| bytes[bytes.len() - own..].to_vec();
        assert_ne!(tail(&fresh), tail(&changed), "{module}");

        let other = spawn(&mut sim, name);
        let target = module_named(&mut sim, other, module);
        let (mut payload, _ctx, generation) =
            sim.world.modules.take_payload(target).expect("payload");
        let mut load = XferLoad::new(&changed);
        payload.xfer(&mut load, &sim).expect("xfer load");
        sim.world
            .modules
            .restore_payload(target, generation, payload);
        assert_eq!(tail(&save(&mut sim, target)), tail(&changed), "{module}");
    }
}

#[test]
fn upgrades_reach_the_update_modules() {
    let ini = "\
Object Radar
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = RadarUpdate ModuleTag_Radar
    RadarExtendTime = 1000
  End
  Behavior = RadarUpdate ModuleTag_Radar2
    RadarExtendTime = 1000
  End
  Behavior = RadarUpgrade ModuleTag_Up
    TriggeredBy = UpgA
  End
End
Object Plant
  Body = InactiveBody ModuleTag_Body
  End
  Behavior = PowerPlantUpdate ModuleTag_Rods
    RodsExtendTime = 600
  End
  Behavior = PowerPlantUpdate ModuleTag_Rods2
    RodsExtendTime = 600
  End
  Behavior = PowerPlantUpgrade ModuleTag_Up
    TriggeredBy = UpgA
  End
End
";
    let mut sim = SimContext::new();
    sim.rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new(
                "upg.ini",
                b"Upgrade UpgA\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n",
                IniLoadType::Overwrite,
            ),
            &mut sim.rules.name_keys,
        )
        .expect("upgrades");
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "object.ini",
            ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("templates");
    sim.world.frame = 10;

    let radar = spawn(&mut sim, "Radar");
    sim.grant_upgrade(radar, "UpgA", 0);
    let m = module_named(&mut sim, radar, "RadarUpdate");
    let done = sim
        .dispatch_module(m, |b, _, _| {
            let r = b
                .as_any()
                .and_then(|a| a.downcast_ref::<RadarUpdate>())
                .expect("radar");
            (r.radar_active, r.extend_done_frame)
        })
        .expect("dispatch");
    assert_eq!(done, (1, 40));
    // `findUpdateModule` stops at the first one.
    let active: Vec<u8> = sim
        .world
        .objects
        .get(radar)
        .expect("radar")
        .behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|ms| {
            sim.dispatch_module(ms, |b, _, _| {
                b.as_any()
                    .and_then(|a| a.downcast_ref::<RadarUpdate>())
                    .map(|r| r.radar_active)
            })
            .expect("dispatch")
        })
        .collect();
    assert_eq!(active, [1, 0]);

    // Every module with the interface is told, not only the first.
    let plant = spawn(&mut sim, "Plant");
    sim.grant_upgrade(plant, "UpgA", 0);
    let slots: Vec<ModuleSlot> = sim
        .world
        .objects
        .get(plant)
        .expect("plant")
        .behaviors
        .iter()
        .map(|&(ms, _)| ms)
        .collect();
    let extended = slots
        .into_iter()
        .filter_map(|ms| {
            sim.dispatch_module(ms, |b, _, _| {
                b.as_any()
                    .and_then(|a| a.downcast_ref::<PowerPlantUpdate>())
                    .map(|p| p.extended)
            })
            .expect("dispatch")
        })
        .collect::<Vec<_>>();
    assert_eq!(extended, [1, 1]);
}
