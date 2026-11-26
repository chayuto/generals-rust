//! Gate V2-Golden test for spatial grid Stage 4b: object-side vision,
//! shroud clearing, active shroud jamming, `SightingInfo` persistence, refresh
//! sites, map-wide reveals, and boundary store/restore.
//!
//! Oracle: `tests/cpp_harness/gen_partition_object_shroud.cpp`
//! Golden vectors: `docs/golden_vectors/PARTITION_OBJECT_SHROUD.json` (500 vectors).

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, NameKeyType, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ObjectSlot, ThingTemplateId};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::economy::MAX_PLAYER_COUNT;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::sim::{ObjectRecord, SimApi, SimContext, SimView, SimWorld};
use zh_logic::spatial::partition_grid::PartitionGrid;

const JSON: &str = include_str!("../../../docs/golden_vectors/PARTITION_OBJECT_SHROUD.json");
const VECTORS: usize = 500;

#[derive(Debug)]
enum Value {
    Number(i64),
    Text(String),
    List(Vec<Value>),
}

impl Value {
    fn parse(bytes: &[u8], at: &mut usize) -> Value {
        while *at < bytes.len() && bytes[*at].is_ascii_whitespace() {
            *at += 1;
        }
        match bytes[*at] {
            b'[' => {
                *at += 1;
                let mut items = Vec::new();
                while bytes[*at] != b']' {
                    items.push(Value::parse(bytes, at));
                    while *at < bytes.len() && bytes[*at].is_ascii_whitespace() {
                        *at += 1;
                    }
                    if bytes[*at] == b',' {
                        *at += 1;
                    }
                }
                *at += 1;
                Value::List(items)
            }
            b'"' => {
                let start = *at + 1;
                let mut end = start;
                while bytes[end] != b'"' {
                    end += 1;
                }
                *at = end + 1;
                Value::Text(String::from_utf8(bytes[start..end].to_vec()).unwrap())
            }
            _ => {
                let start = *at;
                while *at < bytes.len() && (bytes[*at] == b'-' || bytes[*at].is_ascii_digit()) {
                    *at += 1;
                }
                Value::Number(
                    std::str::from_utf8(&bytes[start..*at])
                        .unwrap()
                        .parse()
                        .unwrap(),
                )
            }
        }
    }

    fn list(&self) -> &[Value] {
        match self {
            Value::List(items) => items,
            other => panic!("not a list: {other:?}"),
        }
    }

    fn number(&self) -> i64 {
        match self {
            Value::Number(n) => *n,
            other => panic!("not a number: {other:?}"),
        }
    }

    fn text(&self) -> &str {
        match self {
            Value::Text(text) => text,
            other => panic!("not a string: {other:?}"),
        }
    }
}

fn field(line: &str, key: &str) -> Value {
    let marker = format!("\"{key}\":");
    let mut at = line.find(&marker).expect("field") + marker.len();
    Value::parse(line.as_bytes(), &mut at)
}

fn real(value: &Value) -> Real {
    Real::from_bits(value.number() as u32)
}

fn region_from_vals(g: &[Value]) -> (Real, Region3D) {
    let cs = real(&g[0]);
    let lo = Coord3D::new(real(&g[1]), real(&g[2]), 0.0);
    let hi = Coord3D::new(real(&g[3]), real(&g[4]), 100.0);
    (cs, Region3D { lo, hi })
}

fn digest(bytes: &[u8]) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    for &byte in bytes {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{}:{hash:016x}", bytes.len())
}

fn crc_of(grid: &PartitionGrid) -> u32 {
    let mut xfer = XferCrc::new();
    grid.crc(&mut xfer);
    xfer.get_crc()
}

fn save_grid(grid: &mut PartitionGrid) -> Vec<u8> {
    let mut xfer = XferSave::new();
    grid.xfer(&mut xfer).expect("save grid");
    xfer.into_bytes()
}

fn save_obj_vision(obj: &mut ObjectRecord) -> Vec<u8> {
    let mut xfer = XferSave::new();
    obj.xfer_vision_and_shroud(&mut xfer)
        .expect("save object vision");
    xfer.into_bytes()
}

const INI_TEXT: &str = "\
Object PosViewObj
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = DeletionUpdate ModuleTag_Del
    MinLifetime = 5
    MaxLifetime = 5
  End
End
Object PosUnitInherit
  KindOf = INFANTRY
  TransportSlotCount = 1
  VisionRange = 90
  ShroudClearingRange = -1
  Geometry = SPHERE
  GeometryMajorRadius = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object PosUnitExplicit
  KindOf = VEHICLE
  TransportSlotCount = 1
  VisionRange = 90
  ShroudClearingRange = 140
  Geometry = CYLINDER
  GeometryMajorRadius = 15
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ActiveShroudUpgrade ModuleTag_Upg
    TriggeredBy = Upg_Shroud
    NewShroudRange = 110
  End
End
Object PosUnitZero
  KindOf = INFANTRY
  TransportSlotCount = 1
  VisionRange = 80
  ShroudClearingRange = 0
  ShroudRevealToAllRange = 55
  Geometry = SPHERE
  GeometryMajorRadius = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object PosRevealAll
  KindOf = INFANTRY REVEAL_TO_ALL
  TransportSlotCount = 1
  VisionRange = 75
  ShroudClearingRange = 75
  Geometry = SPHERE
  GeometryMajorRadius = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object PosRevealEnemy
  KindOf = INFANTRY
  TransportSlotCount = 1
  VisionRange = 60
  ShroudClearingRange = 85
  ShroudRevealToAllRange = 50
  Geometry = SPHERE
  GeometryMajorRadius = 12
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
End
Object PosGarrison
  KindOf = STRUCTURE
  VisionRange = 70
  ShroudClearingRange = 70
  Geometry = BOX
  GeometryMajorRadius = 20
  GeometryMinorRadius = 20
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300
    InitialHealth = 300
  End
  Behavior = GarrisonContain ModuleTag_Contain
    ContainMax = 4
    IsEnclosingContainer = Yes
  End
End
Object PosTransport
  KindOf = VEHICLE
  VisionRange = 70
  ShroudClearingRange = 70
  Geometry = SPHERE
  GeometryMajorRadius = 15
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = TransportContain ModuleTag_Contain
    ContainMax = 4
  End
End
Object PosShroudCrate
  KindOf = CRATE
  VisionRange = 50
  ShroudClearingRange = 50
  Geometry = SPHERE
  GeometryMajorRadius = 8
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
  Behavior = ShroudCrateCollide ModuleTag_Collide
  End
End
Object PosSpLauncher
  KindOf = STRUCTURE
  VisionRange = 60
  ShroudClearingRange = 60
  Geometry = BOX
  GeometryMajorRadius = 15
  GeometryMinorRadius = 15
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = OCLSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_View
    OCL = OCL_Empty
    CreateLocation = CREATE_AT_LOCATION
  End
End
";

const TEMPLATE_NAMES: [&str; 9] = [
    "PosUnitInherit",
    "PosUnitExplicit",
    "PosUnitZero",
    "PosRevealAll",
    "PosRevealEnemy",
    "PosGarrison",
    "PosTransport",
    "PosShroudCrate",
    "PosSpLauncher",
];

fn setup_rules(sim: &mut SimContext) -> Vec<ThingTemplateId> {
    let upg_ini = b"Upgrade Upg_Shroud\n  Type = OBJECT\nEnd\n";
    sim.rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new("u.ini", upg_ini, IniLoadType::Overwrite),
            &mut sim.rules.name_keys,
        )
        .expect("upgrade ini");
    sim.rules.ocl_names.insert("OCL_Empty".to_owned());
    let sp_ini = b"SpecialPower Sp_View\n  Enum = SPECIAL_SPY_SATELLITE\n  ReloadTime = 1\n  ViewObjectRange = 120\n  ViewObjectDuration = 15\nEnd\n";
    sim.rules
        .special_power_store
        .load_from_ini(
            &mut IniReader::new("sp.ini", sp_ini, IniLoadType::Overwrite),
            &sim.rules.science_store,
            &mut sim.rules.name_keys,
        )
        .expect("sp ini");
    sim.rules.global_data.special_power_view_object_name = "PosViewObj".to_owned();
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "t.ini",
            INI_TEXT.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("templates ini");
    TEMPLATE_NAMES
        .iter()
        .map(|name| {
            sim.rules
                .thing_factory
                .find_template_id(name)
                .unwrap_or_else(|| panic!("missing template {name}"))
        })
        .collect()
}

fn team_for(idx: usize, players: usize) -> TeamId {
    if idx < players {
        TeamId((idx as i32) + 1)
    } else {
        TeamId((idx - players) as i32 + 101)
    }
}

#[test]
fn golden_partition_object_shroud() {
    let mut sim = SimContext::new();
    let tmpl_ids = setup_rules(&mut sim);

    let mut vectors = 0;
    let mut rows = 0;

    for (v, line) in JSON
        .lines()
        .filter(|l| l.starts_with("{\"g\":"))
        .enumerate()
    {
        sim.world = SimWorld::new();
        let g = field(line, "g");
        let (mut cur_cs, mut cur_extent) = region_from_vals(g.list());
        let players = field(line, "players").number() as usize;
        let persist = field(line, "persist").number() as UnsignedInt;
        let frame = field(line, "frame").number() as UnsignedInt;
        let rels = field(line, "rels");
        let rel_list = rels.list();

        sim.rules.global_data.unlook_persist_duration = persist;
        sim.world.frame = frame;

        for p in 0..players {
            let def_team = team_for(p, players);
            let alt_team = team_for(players + p, players);
            sim.world.player_default_teams.insert(p as Int, def_team);
            sim.world.team_players.insert(def_team, p as Int);
            sim.world.team_players.insert(alt_team, p as Int);
        }
        for i in 0..players {
            for j in 0..players {
                let r = match rel_list[i * players + j].number() {
                    0 => Relationship::Enemies,
                    2 => Relationship::Allies,
                    _ => Relationship::Neutral,
                };
                sim.world.player_relationships.insert((i as Int, j as Int), r);
            }
        }
        sim.init_partition(cur_cs, cur_extent.clone());

        let mut objs: Vec<Option<ObjectSlot>> = Vec::new();
        let script = field(line, "s");
        for (r, row) in script.list().iter().enumerate() {
            let row = row.list();
            let op = row[0].number();
            let at = format!("vector {v} row {r} op {op}");
            let arg_count = match op {
                0 => 6,
                1 | 20 => 4,
                21 => 5,
                7 | 8 | 12 => 3,
                2 | 3 | 4 | 9 | 10 | 11 | 13 | 19 | 22 => 2,
                5 | 6 | 14 | 15 | 17 => 1,
                16 | 18 => 0,
                _ => panic!("unknown op {op}, {at}"),
            };
            let a = &row[1..=arg_count];
            match op {
                0 => {
                    let tmpl_idx = a[0].number() as usize;
                    let team_idx = a[1].number() as usize;
                    let pos = Coord3D::new(real(&a[2]), real(&a[3]), real(&a[4]));
                    let under_const = a[5].number() != 0;
                    let mut st = ObjectStatusMaskType::new();
                    if under_const {
                        st.set(ObjectStatusType::UnderConstruction.as_i32(), 1);
                    }
                    let slot = sim.spawn_object_with_status(
                        tmpl_ids[tmpl_idx],
                        team_for(team_idx, players),
                        pos,
                        0.0,
                        st,
                    );
                    objs.push(Some(slot));
                }
                1 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    let pos = Coord3D::new(real(&a[1]), real(&a[2]), real(&a[3]));
                    sim.set_object_position(slot, &pos);
                }
                2 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    sim.set_vision_range(slot, real(&a[1]));
                }
                3 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    sim.set_shroud_clearing_range(slot, real(&a[1]));
                }
                4 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    sim.set_shroud_range(slot, real(&a[1]));
                }
                5 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    sim.handle_partition_cell_maintenance(slot);
                }
                6 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    sim.grant_upgrade(slot, "Upg_Shroud", FALSE);
                }
                7 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    let setting = if a[1].number() != 0 { TRUE } else { FALSE };
                    let by_whom = a[2].number() as Int;
                    sim.set_vision_spied(slot, setting, by_whom);
                }
                8 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    let st = match a[1].number() {
                        0 => ObjectStatusType::Stealthed,
                        1 => ObjectStatusType::Detected,
                        2 => ObjectStatusType::Disguised,
                        _ => ObjectStatusType::UnderConstruction,
                    };
                    if a[2].number() != 0 {
                        sim.set_status(slot, st);
                    } else {
                        sim.clear_status(slot, st);
                    }
                }
                9 => {
                    let c_slot = objs[a[0].number() as usize].expect("live");
                    let r_slot = objs[a[1].number() as usize].expect("live");
                    sim.contain_add(c_slot, r_slot);
                }
                10 => {
                    let c_slot = objs[a[0].number() as usize].expect("live");
                    let r_slot = objs[a[1].number() as usize].expect("live");
                    sim.contain_remove(c_slot, r_slot, FALSE);
                }
                11 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    let tidx = a[1].number() as usize;
                    sim.set_object_team(slot, team_for(tidx, players));
                }
                12 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    let tidx = a[1].number() as usize;
                    let det = a[2].number() as UnsignedInt;
                    sim.defect_object(slot, team_for(tidx, players), det, FALSE);
                }
                13 => {
                    let tidx = a[0].number() as usize;
                    let np = a[1].number() as Int;
                    sim.set_team_controlling_player(team_for(tidx, players), np);
                }
                14 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    let max_hp = sim.object_max_health(slot);
                    let mut d = DamageInfo::default();
                    d.input.damage_type = DamageType::Unresistable;
                    d.input.death_type = DeathType::Normal;
                    d.input.amount = max_hp;
                    d.input.kill = TRUE;
                    sim.attempt_damage(slot, &mut d);
                }
                15 => {
                    let slot = objs[a[0].number() as usize].expect("live");
                    sim.destroy_object_now(slot);
                }
                16 => {
                    sim.process_destroy_list();
                    for entry in &mut objs {
                        if let Some(slot) = *entry {
                            if sim.object(slot).is_none() {
                                *entry = None;
                            }
                        }
                    }
                }
                17 => {
                    sim.world.frame = a[0].number() as UnsignedInt;
                }
                18 => {
                    sim.update_partition();
                }
                19 => {
                    let crate_slot = objs[a[0].number() as usize].expect("live");
                    let picker_slot = objs[a[1].number() as usize].expect("live");
                    let pos = *sim.object(crate_slot).unwrap().thing.get_position();
                    let norm = Coord3D::new(0.0, 0.0, 1.0);
                    let ms = sim
                        .object(crate_slot)
                        .unwrap()
                        .behaviors
                        .iter()
                        .find(|(_, m)| m.contains(FacetMask::COLLIDE))
                        .unwrap()
                        .0;
                    let _ = sim.dispatch_module(ms, |m, sim_a, ctx| {
                        m.as_collide_mut().unwrap().on_collide(
                            sim_a,
                            ctx,
                            Some(picker_slot),
                            Some(&pos),
                            Some(&norm),
                        );
                    });
                }
                20 => {
                    let sp_slot = objs[a[0].number() as usize].expect("live");
                    let loc = Coord3D::new(real(&a[1]), real(&a[2]), real(&a[3]));
                    let prev_head = sim.world.objects.world_iter().next();
                    let ms = sim
                        .object(sp_slot)
                        .unwrap()
                        .behaviors
                        .iter()
                        .find(|(_, m)| m.contains(FacetMask::SPECIAL_POWER))
                        .unwrap()
                        .0;
                    let _ = sim.dispatch_module(ms, |m, sim_a, ctx| {
                        m.as_special_power_mut()
                            .unwrap()
                            .do_special_power_at_location(sim_a, ctx, &loc, 0.0, 0);
                    });
                    let new_head = sim.world.objects.world_iter().next();
                    if new_head != prev_head {
                        if let Some(view_slot) = new_head {
                            objs.push(Some(view_slot));
                        }
                    }
                }
                21 => {
                    (cur_cs, cur_extent) = region_from_vals(a);
                    sim.init_partition(cur_cs, cur_extent.clone());
                }
                22 => {
                    let mk = a[0].number();
                    let p = a[1].number() as usize;
                    match mk {
                        0 => sim.world.partition.reveal_map_for_player(p),
                        1 => sim.world.partition.reveal_map_for_player_permanently(p),
                        2 => sim
                            .world
                            .partition
                            .undo_reveal_map_for_player_permanently(p, players),
                        _ => sim.world.partition.shroud_map_for_player(p, players),
                    }
                }
                _ => unreachable!(),
            }

            let want_crc = row[arg_count + 1].number() as u32;
            let want_pm_save = row[arg_count + 2].text();
            let want_objs = row[arg_count + 3].list();

            assert_eq!(crc_of(&sim.world.partition), want_crc, "pm crc, {at}");
            let saved_pm = save_grid(&mut sim.world.partition);
            assert_eq!(digest(&saved_pm), want_pm_save, "pm xfer, {at}");

            let mut loaded_pm = PartitionGrid::new(cur_cs, cur_extent.clone());
            let mut load = XferLoad::new(&saved_pm);
            loaded_pm.xfer(&mut load).expect("load pm");
            assert_eq!(load.position(), saved_pm.len(), "pm load len, {at}");
            assert_eq!(save_grid(&mut loaded_pm), saved_pm, "pm roundtrip, {at}");

            assert_eq!(objs.len(), want_objs.len(), "obj count, {at}");
            for (i, (slot_opt, want_hash)) in objs.iter().zip(want_objs.iter()).enumerate() {
                let want_hash = want_hash.text();
                match slot_opt {
                    None => assert_eq!(want_hash, "", "deleted obj {i}, {at}"),
                    Some(slot) => {
                        let obj = sim.world.objects.get_mut(*slot).expect("obj alive");
                        let saved_obj = save_obj_vision(obj);
                        assert_eq!(digest(&saved_obj), want_hash, "obj {i} vision xfer, {at}");

                        let mut scratch = ObjectRecord::new(
                            obj.id,
                            ThingTemplateId(0),
                            obj.team_id,
                            obj.geometry.clone(),
                        );
                        let mut xl = XferLoad::new(&saved_obj);
                        scratch
                            .xfer_vision_and_shroud(&mut xl)
                            .expect("load obj vision");
                        assert_eq!(xl.position(), saved_obj.len(), "obj {i} load len, {at}");
                        assert_eq!(
                            save_obj_vision(&mut scratch),
                            saved_obj,
                            "obj {i} roundtrip, {at}"
                        );
                    }
                }
            }

            rows += 1;
        }
        vectors += 1;
    }

    assert_eq!(vectors, VECTORS);
    assert!(rows > 15 * VECTORS, "only {rows} rows");
    let _ = MAX_PLAYER_COUNT;
    let _ = NameKeyType::default();
}
