//! Gate V2-Golden test for `PKT-372` (`SpyVisionUpdate`) and `PKT-323`
//! (`DynamicShroudClearingRangeUpdate`), including `SpyVisionSpecialPower` (`PKT-245`),
//! `SabotageInternetCenterCrateCollide` (`PKT-234`), and `PartitionManager` integration.
//!
//! Oracle: `tests/cpp_harness/gen_shroud_updates.cpp`
//! Golden vectors: `docs/golden_vectors/SHROUD_UPDATES.json` (300 vectors).

use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{Int, Real, Relationship, TeamId, UnsignedInt, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleSlot, ObjectSlot};
use zh_logic::modules::update::spy_vision_update::SpyVisionUpdate;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::sim::{SimApi, SimContext, SimView, SimWorld};
use zh_logic::spatial::partition_grid::PartitionGrid;

const JSON: &str = include_str!("../../../docs/golden_vectors/SHROUD_UPDATES.json");
const VECTORS: usize = 300;

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
    let mut at = line.find(&marker).unwrap_or_else(|| panic!("missing {key}")) + marker.len();
    Value::parse(line.as_bytes(), &mut at)
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}

fn check_crc(actual: u32, expected: u32, label: &str) {
    #[cfg(not(feature = "retail-crc"))]
    assert_eq!(actual, expected, "{label} crc mismatch");
    #[cfg(feature = "retail-crc")]
    assert_eq!(actual.swap_bytes(), expected, "{label} retail-crc mismatch");
}

fn crc_grid(grid: &PartitionGrid) -> u32 {
    let mut xfer = XferCrc::new();
    grid.crc(&mut xfer);
    xfer.get_crc()
}

fn snapshot_mod(sim: &mut SimContext, ms: ModuleSlot) -> (u32, String) {
    let mut xc = XferCrc::new();
    let mut xs = XferSave::new();
    sim.dispatch_module(ms, |m, s, _ctx| {
        m.crc(&mut xc);
        m.xfer(&mut xs, s).expect("mod xfer save");
    })
    .expect("dispatch mod snapshot");
    (xc.get_crc(), encode_hex(xs.as_bytes()))
}

fn xfer_roundtrip_mod(sim: &mut SimContext, ms: ModuleSlot) {
    let mut xs = XferSave::new();
    sim.dispatch_module(ms, |m, s, _ctx| {
        m.xfer(&mut xs, s).expect("mod xfer save rt");
    })
    .expect("dispatch mod rt save");
    let bytes = xs.into_bytes();
    sim.dispatch_module(ms, |m, s, ctx| {
        let mut xl = XferLoad::new(&bytes);
        m.xfer(&mut xl, s).expect("mod xfer load rt");
        assert_eq!(xl.position(), bytes.len(), "xfer load trailing bytes");
        m.load_post_process(s, ctx);
    })
    .expect("dispatch mod rt load");
}

fn step_frame(sim: &mut SimContext) {
    sim.world.frame += 1;
    sim.step_sleepy_updates();
    sim.update_partition();
    sim.process_destroy_list();
}

fn def_team(p: usize) -> TeamId {
    TeamId((p as i32) + 1)
}

fn alt_team(p: usize) -> TeamId {
    TeamId((p as i32) + 101)
}

fn find_sv(sim: &SimContext, owner: ObjectSlot) -> ModuleSlot {
    let sv_key = sim
        .rules
        .name_keys
        .lookup_key("SpyVisionUpdate")
        .expect("SpyVisionUpdate key");
    sim.object(owner)
        .expect("owner")
        .behaviors
        .iter()
        .find_map(|&(ms, _)| {
            (sim.world.modules.payload(ms).map(|m| m.module_name_key()) == Some(sv_key))
                .then_some(ms)
        })
        .expect("missing SpyVisionUpdate")
}

fn find_dscru(sim: &SimContext, owner: ObjectSlot) -> ModuleSlot {
    let ds_key = sim
        .rules
        .name_keys
        .lookup_key("DynamicShroudClearingRangeUpdate")
        .expect("DynamicShroudClearingRangeUpdate key");
    sim.object(owner)
        .expect("owner")
        .behaviors
        .iter()
        .find_map(|&(ms, _)| {
            (sim.world.modules.payload(ms).map(|m| m.module_name_key()) == Some(ds_key))
                .then_some(ms)
        })
        .expect("missing DynamicShroudClearingRangeUpdate")
}

fn find_first_facet(sim: &SimContext, owner: ObjectSlot, mask: FacetMask) -> ModuleSlot {
    sim.object(owner)
        .expect("owner")
        .behaviors
        .iter()
        .find_map(|&(ms, m)| m.contains(mask).then_some(ms))
        .unwrap_or_else(|| panic!("missing facet {mask:?}"))
}

fn setup_base_rules(sim: &mut SimContext) {
    let base_ini = "\
Object SuRiderInf
  KindOf = INFANTRY
  TransportSlotCount = 1
  VisionRange = 50
  ShroudClearingRange = 50
  Geometry = SPHERE
  GeometryMajorRadius = 5
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
End
Object SuTargetVeh
  KindOf = VEHICLE
  VisionRange = 70
  ShroudClearingRange = 70
  Geometry = SPHERE
  GeometryMajorRadius = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
End
Object SuTargetInf
  KindOf = INFANTRY
  VisionRange = 60
  ShroudClearingRange = 60
  Geometry = SPHERE
  GeometryMajorRadius = 6
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 80
    InitialHealth = 80
  End
End
Object SuTargetStruct
  KindOf = STRUCTURE
  VisionRange = 80
  ShroudClearingRange = 80
  Geometry = BOX
  GeometryMajorRadius = 15
  GeometryMinorRadius = 15
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300
    InitialHealth = 300
  End
End
";
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "base.ini",
            base_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("load base objects");
}

fn check_spy_slice(sim: &SimContext, slot: Option<ObjectSlot>, want: &[Value], label: &str) {
    let got: [Int; 4] = match slot
        .and_then(|s| sim.object(s))
        .filter(|o| !o.is_destroyed())
    {
        None => [-1, 0, 0, 0],
        Some(obj) => [
            obj.vision_spied_mask as Int,
            obj.vision_spied_by[1],
            obj.vision_spied_by[2],
            obj.vision_spied_by[3],
        ],
    };
    let expected = [
        want[0].number() as Int,
        want[1].number() as Int,
        want[2].number() as Int,
        want[3].number() as Int,
    ];
    assert_eq!(got, expected, "{label} spy slice");
}

fn check_sv_mod(
    sim: &mut SimContext,
    obj_slot: Option<ObjectSlot>,
    want_crc: u32,
    want_hex: &str,
    want_dis_until: UnsignedInt,
    label: &str,
) {
    match obj_slot.filter(|&s| sim.object(s).is_some_and(|o| !o.is_destroyed())) {
        None => {
            assert_eq!(want_crc, 0, "{label} dead sv want_crc");
            assert_eq!(want_hex, "", "{label} dead sv want_hex");
            assert_eq!(want_dis_until, 0, "{label} dead sv want_dis");
        }
        Some(s) => {
            let ms = find_sv(sim, s);
            let (crc, hex) = snapshot_mod(sim, ms);
            check_crc(crc, want_crc, label);
            assert_eq!(hex, want_hex, "{label} xfer");
            let dis_until = sim
                .dispatch_module(ms, |m, _sim, _ctx| {
                    m.as_any()
                        .and_then(|a| a.downcast_ref::<SpyVisionUpdate>())
                        .expect("SpyVisionUpdate")
                        .get_disabled_until_frame()
                })
                .expect("dis_until");
            assert_eq!(dis_until, want_dis_until, "{label} disabled_until");
        }
    }
}

fn assert_sv_checkpoint(
    sim: &mut SimContext,
    o0: Option<ObjectSlot>,
    o1: Option<ObjectSlot>,
    o2: Option<ObjectSlot>,
    o3: Option<ObjectSlot>,
    o4: Option<ObjectSlot>,
    cp: &[Value],
    at: &str,
) {
    check_sv_mod(
        sim,
        o0,
        cp[0].number() as u32,
        cp[1].text(),
        cp[2].number() as UnsignedInt,
        &format!("{at} sv0"),
    );
    check_sv_mod(
        sim,
        o1,
        cp[3].number() as u32,
        cp[4].text(),
        cp[5].number() as UnsignedInt,
        &format!("{at} sv1"),
    );
    check_crc(crc_grid(&sim.world.partition), cp[6].number() as u32, &format!("{at} pm"));
    check_spy_slice(sim, o2, cp[7].list(), &format!("{at} o2"));
    check_spy_slice(sim, o3, cp[8].list(), &format!("{at} o3"));
    check_spy_slice(sim, o4, cp[9].list(), &format!("{at} o4"));
}

#[test]
fn golden_shroud_updates() {
    let mut sim = SimContext::new();
    setup_base_rules(&mut sim);

    let mut vector_count = 0usize;
    let mut sv_checkpoints = 0usize;
    let mut ds_checkpoints = 0usize;

    for line in JSON.lines().filter(|l| l.starts_with("{\"seed\":")) {
        sim.world = SimWorld::new();

        let seed = field(line, "seed").number() as usize;
        let persist = field(line, "persist").number() as UnsignedInt;
        let frame0 = field(line, "frame0").number() as UnsignedInt;
        let rels = field(line, "rels");
        let rel_list = rels.list();
        let base_ms = field(line, "base_ms").number() as UnsignedInt;
        let bonus_ms = field(line, "bonus_ms").number() as UnsignedInt;
        let max_ms = field(line, "max_ms").number() as UnsignedInt;
        let riders = field(line, "riders").number() as usize;
        let sv2_needs_upg = field(line, "sv2_needs_upg").number() != 0;
        let sv2_self = field(line, "sv2_self").number() != 0;
        let self_dur_ms = field(line, "self_dur_ms").number() as UnsignedInt;
        let self_int_ms = field(line, "self_int_ms").number() as UnsignedInt;
        let spy_kind = field(line, "spy_kind").text().to_owned();
        let sab_ms = field(line, "sab_ms").number() as UnsignedInt;
        let test_dis = field(line, "test_dis").number() != 0;
        let clear_hack = field(line, "clear_hack").number() != 0;
        let steps_v = field(line, "steps");
        let steps = steps_v.list();
        let t_step1 = steps[0].number() as UnsignedInt;
        let t_step2 = steps[1].number() as UnsignedInt;
        let t_step3 = steps[2].number() as UnsignedInt;

        let dscru_vis = Real::from_bits(field(line, "dscru_vis").number() as u32);
        let dscru_scr = Real::from_bits(field(line, "dscru_scr").number() as u32);
        let gd_ms = field(line, "gd_ms").number() as UnsignedInt;
        let gt_ms = field(line, "gt_ms").number() as UnsignedInt;
        let sd_ms = field(line, "sd_ms").number() as UnsignedInt;
        let st_ms = field(line, "st_ms").number() as UnsignedInt;
        let gi_ms = field(line, "gi_ms").number() as UnsignedInt;
        let ci_ms = field(line, "ci_ms").number() as UnsignedInt;
        let fv = Real::from_bits(field(line, "fv").number() as u32);

        let sp_name = format!("SpSu_{seed}");
        let upg_name = format!("UpgSu_{seed}");
        let t_sv1 = format!("SuSv1_{seed}");
        let t_sv2 = format!("SuSv2_{seed}");
        let t_sab = format!("SuSab_{seed}");
        let t_dscru = format!("SuDscru_{seed}");

        let upg_ini = format!("Upgrade {upg_name}\n  Type = OBJECT\nEnd\n");
        sim.rules
            .upgrade_center
            .load_from_ini(
                &mut IniReader::new("u.ini", upg_ini.as_bytes(), IniLoadType::Overwrite),
                &mut sim.rules.name_keys,
            )
            .expect("upgrade ini");
        let sp_ini = format!(
            "SpecialPower {sp_name}\n  Enum = SPECIAL_SPY_SATELLITE\n  ReloadTime = 1\nEnd\n"
        );
        sim.rules
            .special_power_store
            .load_from_ini(
                &mut IniReader::new("sp.ini", sp_ini.as_bytes(), IniLoadType::Overwrite),
                &sim.rules.science_store,
                &mut sim.rules.name_keys,
            )
            .expect("sp ini");

        let spy_kind_line = if spy_kind.is_empty() {
            String::new()
        } else {
            format!("    SpyOnKindof = {spy_kind}\n")
        };
        let obj_ini = format!(
            "Object {t_sv1}
  KindOf = STRUCTURE FS_INTERNET_CENTER
  VisionRange = 40
  ShroudClearingRange = 40
  Geometry = BOX
  GeometryMajorRadius = 15
  GeometryMinorRadius = 15
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300
    InitialHealth = 300
  End
  Behavior = InternetHackContain ModuleTag_Contain
    ContainMax = 5
    Slots = 5
    AllowInsideKindOf = INFANTRY
  End
  Behavior = SpyVisionUpdate ModuleTag_SVU
    NeedsUpgrade = No
    SelfPowered = No
  End
  Behavior = SpyVisionSpecialPower ModuleTag_SP
    SpecialPowerTemplate = {sp_name}
    BaseDuration = {base_ms}
    BonusDurationPerCaptured = {bonus_ms}
    MaxDuration = {max_ms}
  End
End
Object {t_sv2}
  KindOf = STRUCTURE FS_INTERNET_CENTER
  VisionRange = 40
  ShroudClearingRange = 40
  Geometry = BOX
  GeometryMajorRadius = 15
  GeometryMinorRadius = 15
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300
    InitialHealth = 300
  End
  Behavior = SpyVisionUpdate ModuleTag_SVU
    NeedsUpgrade = {}
    TriggeredBy = {upg_name}
    SelfPowered = {}
    SelfPoweredDuration = {self_dur_ms}
    SelfPoweredInterval = {self_int_ms}
{spy_kind_line}  End
End
Object {t_sab}
  KindOf = INFANTRY
  VisionRange = 30
  ShroudClearingRange = 30
  Geometry = SPHERE
  GeometryMajorRadius = 8
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SabotageInternetCenterCrateCollide ModuleTag_Collide
    SabotageDuration = {sab_ms}
    BuildingPickup = Yes
  End
End
Object {t_dscru}
  KindOf = IMMOBILE
  VisionRange = {dscru_vis}
  ShroudClearingRange = {dscru_scr}
  Geometry = SPHERE
  GeometryMajorRadius = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = DynamicShroudClearingRangeUpdate ModuleTag_DSCRU
    GrowDelay = {gd_ms}
    GrowTime = {gt_ms}
    ShrinkDelay = {sd_ms}
    ShrinkTime = {st_ms}
    GrowInterval = {gi_ms}
    ChangeInterval = {ci_ms}
    FinalVision = {fv}
    GridDecalTemplate
      Texture = SCCA10
      OpacityMin = 25%
      OpacityMax = 50%
      OpacityThrobTime = 500
      Color = R:255 G:255 B:0 A:255
      OnlyVisibleToOwningPlayer = Yes
    End
  End
End
",
            if sv2_needs_upg { "Yes" } else { "No" },
            if sv2_self { "Yes" } else { "No" },
        );
        sim.rules
            .load_thing_templates_from_ini(&mut IniReader::new(
                "dyn.ini",
                obj_ini.as_bytes(),
                IniLoadType::Overwrite,
            ))
            .expect("load seed templates");

        sim.rules.global_data.unlook_persist_duration = persist;
        sim.world.frame = frame0;

        let players = 4usize;
        for p in 0..players {
            let dt = def_team(p);
            let at = alt_team(p);
            sim.world.player_default_teams.insert(p as Int, dt);
            sim.world.team_players.insert(dt, p as Int);
            sim.world.team_players.insert(at, p as Int);
        }
        sim.world.team_is_neutral.insert(def_team(0), TRUE);
        sim.world.team_is_neutral.insert(alt_team(0), TRUE);
        for i in 0..players {
            for j in 0..players {
                let r = match rel_list[i * players + j].number() {
                    0 => Relationship::Enemies,
                    2 => Relationship::Allies,
                    _ => Relationship::Neutral,
                };
                sim.world.set_player_relation(i as Int, j as Int, r);
            }
        }
        let extent = Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(400.0, 400.0, 100.0),
        };
        sim.init_partition(20.0, extent);

        let tid_sv1 = sim.rules.thing_factory.find_template_id(&t_sv1).unwrap();
        let tid_sv2 = sim.rules.thing_factory.find_template_id(&t_sv2).unwrap();
        let tid_veh = sim.rules.thing_factory.find_template_id("SuTargetVeh").unwrap();
        let tid_inf = sim.rules.thing_factory.find_template_id("SuTargetInf").unwrap();
        let tid_str = sim.rules.thing_factory.find_template_id("SuTargetStruct").unwrap();
        let tid_sab = sim.rules.thing_factory.find_template_id(&t_sab).unwrap();
        let rider_tid = sim.rules.thing_factory.find_template_id("SuRiderInf").unwrap();
        let tid_dscru = sim.rules.thing_factory.find_template_id(&t_dscru).unwrap();

        let mut o0 = Some(sim.spawn_object(tid_sv1, def_team(1), Coord3D::new(60.0, 60.0, 0.0), 0.0));
        let o1 = Some(sim.spawn_object(tid_sv2, def_team(1), Coord3D::new(100.0, 60.0, 0.0), 0.0));
        let o2 = Some(sim.spawn_object(tid_veh, def_team(3), Coord3D::new(220.0, 220.0, 0.0), 0.0));
        let o3 = Some(sim.spawn_object(tid_inf, alt_team(3), Coord3D::new(300.0, 220.0, 0.0), 0.0));
        let o4 = Some(sim.spawn_object(tid_str, def_team(2), Coord3D::new(220.0, 300.0, 0.0), 0.0));
        let o5 = sim.spawn_object(tid_sab, def_team(2), Coord3D::new(360.0, 60.0, 0.0), 0.0);

        let c0 = find_first_facet(&sim, o0.unwrap(), FacetMask::CONTAIN);
        for _ in 0..riders {
            let passenger = sim.spawn_object(rider_tid, def_team(1), Coord3D::new(60.0, 60.0, 0.0), 0.0);
            sim.dispatch_module(c0, |m, s, ctx| {
                m.as_contain_mut().unwrap().add_to_contain(s, ctx, passenger);
            })
            .expect("add rider");
        }

        let sp0 = find_first_facet(&sim, o0.unwrap(), FacetMask::SPECIAL_POWER);
        let col5 = find_first_facet(&sim, o5, FacetMask::COLLIDE);

        let sv_cps_v = field(line, "sv_cps");
        let sv_cps = sv_cps_v.list();
        assert_eq!(sv_cps.len(), 8);

        sim.update_partition();
        step_frame(&mut sim);
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[0].list(), &format!("seed {seed} sv_cp 0"));
        sv_checkpoints += 1;

        if test_dis {
            let until = sim.world.frame + 4;
            sim.set_disabled_until(o0.unwrap(), DisabledType::Emp, until);
            sim.dispatch_module(sp0, |m, s, ctx| {
                m.as_special_power_mut().unwrap().do_special_power(s, ctx, 0);
            })
            .expect("disabled sp");
            sim.clear_disabled(o0.unwrap(), DisabledType::Emp);
        }
        sim.dispatch_module(sp0, |m, s, ctx| {
            m.as_special_power_mut().unwrap().do_special_power(s, ctx, 0);
        })
        .expect("sp");
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[1].list(), &format!("seed {seed} sv_cp 1"));
        sv_checkpoints += 1;

        for _ in 0..t_step1 {
            step_frame(&mut sim);
        }
        sim.grant_upgrade(o1.unwrap(), &upg_name, zh_common::types::FALSE);
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[2].list(), &format!("seed {seed} sv_cp 2"));
        sv_checkpoints += 1;

        let col_loc = *sim.object(o0.unwrap()).unwrap().thing.get_position();
        let col_norm = Coord3D::new(0.0, 0.0, 1.0);
        sim.dispatch_module(col5, |m, s, ctx| {
            m.as_collide_mut()
                .unwrap()
                .on_collide(s, ctx, o0, Some(&col_loc), Some(&col_norm));
        })
        .expect("sabotage collide");
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[3].list(), &format!("seed {seed} sv_cp 3"));
        sv_checkpoints += 1;

        for _ in 0..t_step2 {
            step_frame(&mut sim);
        }
        if clear_hack
            && sim
                .object(o0.unwrap())
                .is_some_and(|o| !o.is_destroyed())
        {
            sim.clear_disabled(o0.unwrap(), DisabledType::Hacked);
        }
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[4].list(), &format!("seed {seed} sv_cp 4"));
        sv_checkpoints += 1;

        if sim
            .object(o1.unwrap())
            .is_some_and(|o| !o.is_destroyed())
        {
            sim.capture_object(o1.unwrap(), def_team(2));
        }
        for _ in 0..t_step3 {
            step_frame(&mut sim);
        }
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[5].list(), &format!("seed {seed} sv_cp 5"));
        sv_checkpoints += 1;

        if sim
            .object(o1.unwrap())
            .is_some_and(|o| !o.is_destroyed())
        {
            let sv1_ms = find_sv(&sim, o1.unwrap());
            xfer_roundtrip_mod(&mut sim, sv1_ms);
        }

        sim.destroy_object_now(o0.unwrap());
        o0 = None;
        for _ in 0..(persist + 2) {
            step_frame(&mut sim);
        }
        assert_sv_checkpoint(&mut sim, o0, o1, o2, o3, o4, sv_cps[6].list(), &format!("seed {seed} sv_cp 6"));
        sv_checkpoints += 1;

        let to_kill: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
        for s in to_kill {
            sim.destroy_object_now(s);
        }
        sim.process_destroy_list();
        for _ in 0..(persist + 2) {
            step_frame(&mut sim);
        }
        assert_sv_checkpoint(&mut sim, None, None, None, None, None, sv_cps[7].list(), &format!("seed {seed} sv_cp 7"));
        sv_checkpoints += 1;

        // Part B: DynamicShroudClearingRangeUpdate
        let od = sim.spawn_object(tid_dscru, def_team(1), Coord3D::new(180.0, 180.0, 0.0), 0.0);
        let ds_ms = find_dscru(&sim, od);

        let dscru_frames = (sd_ms + st_ms) / 33 + 16;
        let total_run = (dscru_frames + 10).max(30);
        let rt_frame = (total_run / 2).max(2);

        let ds_cps_v = field(line, "ds_cps");
        let ds_cps = ds_cps_v.list();
        let mut ds_idx = 0usize;

        sim.update_partition();
        let assert_ds = |sim: &mut SimContext, idx: &mut usize| {
            let row = ds_cps[*idx].list();
            let want_frame = row[0].number() as UnsignedInt;
            let want_range_bits = row[1].number() as u32;
            let want_crc = row[2].number() as u32;
            let want_hex = row[3].text();
            let want_pm = row[4].number() as u32;

            assert_eq!(sim.world.frame, want_frame, "seed {seed} ds_cp {idx} frame");
            let range = sim.object_shroud_clearing_range(od);
            assert_eq!(
                range.to_bits(),
                want_range_bits,
                "seed {seed} ds_cp {idx} range (got {range}, want {})",
                Real::from_bits(want_range_bits)
            );
            let (crc, hex) = snapshot_mod(sim, ds_ms);
            check_crc(crc, want_crc, &format!("seed {seed} ds_cp {idx}"));
            assert_eq!(hex, want_hex, "seed {seed} ds_cp {idx} xfer");
            check_crc(
                crc_grid(&sim.world.partition),
                want_pm,
                &format!("seed {seed} ds_cp {idx} pm"),
            );
            *idx += 1;
        };

        assert_ds(&mut sim, &mut ds_idx);
        ds_checkpoints += 1;

        let mut s0_bits = Vec::new();
        for step in 1..=total_run {
            step_frame(&mut sim);
            if step == rt_frame {
                xfer_roundtrip_mod(&mut sim, ds_ms);
            }
            if seed == 0 {
                s0_bits.push(sim.object_shroud_clearing_range(od).to_bits() as i64);
            }
            if step == 1
                || step == 3
                || step == 6
                || step == rt_frame
                || step == total_run / 3
                || step == (total_run * 2) / 3
                || step == total_run - 2
                || step == total_run
            {
                assert_ds(&mut sim, &mut ds_idx);
                ds_checkpoints += 1;
            }
        }
        assert_eq!(ds_idx, ds_cps.len(), "seed {seed} ds_cps count");

        if seed == 0 {
            let diag_line = JSON
                .lines()
                .find(|l| l.contains("\"seed0_diagnostics\":"))
                .expect("seed0_diagnostics");
            let diag = field(diag_line, "dscru_range_bits");
            let want_bits: Vec<i64> = diag.list().iter().map(Value::number).collect();
            assert_eq!(s0_bits, want_bits, "seed 0 dscru_range_bits");
        }

        let to_kill_b: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
        for s in to_kill_b {
            sim.destroy_object_now(s);
        }
        sim.process_destroy_list();
        for _ in 0..(persist + 2) {
            step_frame(&mut sim);
        }

        vector_count += 1;
    }

    assert_eq!(vector_count, VECTORS);
    assert_eq!(sv_checkpoints, VECTORS * 8);
    assert!(ds_checkpoints >= VECTORS * 8);
}
