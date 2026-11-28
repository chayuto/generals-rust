//! Gate V2-Golden test suite for `WP-5.1-C` (`CreateModule` hierarchy:
//! `CreateModule`, `GrantUpgradeCreate`, `LockWeaponCreate`, `PreorderCreate`,
//! `SpecialPowerCreate`, `SupplyCenterCreate`, `SupplyWarehouseCreate`, `VeterancyGainCreate`,
//! 1,000 C++ reference vectors).

use zh_common::crc::Crc;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleDataId, ModuleSlot, ObjectSlot};
use zh_logic::modules::create::{CreateModule, VeterancyGainCreateModuleData};
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use zh_logic::rules::science::{ScienceTextLookup, ScienceType, SCIENCE_INVALID};
use zh_logic::sim::{SimApi, SimContext, SimView};

const WP51_CREATE_JSON: &str = include_str!("../../../docs/golden_vectors/WP51_Create.json");

struct TestScienceText;

impl ScienceTextLookup for TestScienceText {
    fn fetch_label(&mut self, label: &str) -> String {
        format!("TXT:{label}")
    }
}

fn bytes_crc(bytes: &[u8]) -> u32 {
    let mut c = Crc::new();
    c.compute_crc(bytes, bytes.len() as Int);
    c.get()
}

fn inspect_create_module(
    sim: &mut SimContext,
    owner: ObjectSlot,
) -> (Bool, Vec<u8>, u32, u32) {
    let slot = sim
        .object(owner)
        .expect("object exists")
        .behaviors
        .iter()
        .find(|(_, m)| m.contains(FacetMask::CREATE))
        .map(|&(s, _)| s)
        .expect("CreateModule attached");
    sim.dispatch_module(slot, |m, sim_api, _ctx| {
        let need = m
            .as_create_mut()
            .expect("create facet")
            .should_do_on_build_complete();
        let mut xs = XferSave::new();
        m.xfer(&mut xs, sim_api).expect("create xfer save");
        let bytes = xs.into_bytes();
        let xfer_crc = bytes_crc(&bytes);
        let mut xc = XferCrc::new();
        m.crc(&mut xc);
        (need, bytes, xc.get_crc(), xfer_crc)
    })
    .expect("dispatch create module")
}

#[test]
fn test_wp51_create_oracle_integrity() {
    assert!(WP51_CREATE_JSON.contains("\"packet\": \"WP-5.1-C\""));
    assert!(WP51_CREATE_JSON.contains("\"seed_count\": 1000"));
    assert!(WP51_CREATE_JSON.contains("\"seed0_diagnostics\":"));
    assert_eq!(extract_vector_lines(WP51_CREATE_JSON).len(), 1000);
}

#[test]
fn test_wp51_create_1000_golden_vectors() {
    let lines = extract_vector_lines(WP51_CREATE_JSON);
    assert_eq!(lines.len(), 1000);

    // Verify abstract base CreateModule Xfer layout.
    {
        let mut cm = CreateModule::new(NameKeyType(1));
        assert_eq!(cm.should_do_on_build_complete(), TRUE);
        let mut xs = XferSave::new();
        cm.xfer(&mut xs).expect("base create save");
        assert_eq!(xs.as_bytes(), &[1, 1, 1, 1, 1]);
        let dummy_ctx = ModuleContext {
            mod_slot: ModuleSlot::INVALID,
            owner_slot: ObjectSlot::INVALID,
            owner_id: ObjectId::INVALID,
            tag_key: NameKeyType(1),
            template_data_id: ModuleDataId::INVALID,
        };
        cm.on_build_complete(&mut zh_logic::sim::MockSimApi::new(), dummy_ctx);
        assert_eq!(cm.should_do_on_build_complete(), FALSE);
        let mut xs2 = XferSave::new();
        cm.xfer(&mut xs2).expect("base create save post-build");
        assert_eq!(xs2.as_bytes(), &[1, 1, 1, 1, 0]);
        let mut xl = XferLoad::new(&[1, 1, 1, 1, 1]);
        cm.xfer(&mut xl).expect("base create load");
        assert_eq!(cm.should_do_on_build_complete(), TRUE);
    }

    let mut sim = SimContext::new();
    let upg_ini = "\
Upgrade UpgPlayerTest
  Type = PLAYER
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade UpgObjectTest
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 50
End
";
    sim.rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new("upg.ini", upg_ini.as_bytes(), IniLoadType::Overwrite),
            &mut sim.rules.name_keys,
        )
        .expect("load upgrades");

    let sci_ini = "\
Science SCI_WP51_A
  SciencePurchasePointCost = 1
  IsGrantable = Yes
End
Science SCI_WP51_B
  SciencePurchasePointCost = 1
  IsGrantable = Yes
End
";
    let mut text = TestScienceText;
    sim.rules
        .science_store
        .load_from_ini(
            &mut IniReader::new("sci.ini", sci_ini.as_bytes(), IniLoadType::Overwrite),
            &mut sim.rules.name_keys,
            &mut text,
        )
        .expect("load sciences");

    let wpn_blocks = [
        "\
Weapon CreateWpnPrimary
  PrimaryDamage = 10
  AttackRange = 100
  DamageType = SMALL_ARMS
  DeathType = NORMAL
  WeaponSpeed = 999999
  ClipSize = 1
  DelayBetweenShots = 100
End",
        "\
Weapon CreateWpnSecondary
  PrimaryDamage = 20
  AttackRange = 150
  DamageType = EXPLOSION
  DeathType = NORMAL
  WeaponSpeed = 999999
  ClipSize = 1
  DelayBetweenShots = 100
End",
        "\
Weapon CreateWpnTertiary
  PrimaryDamage = 30
  AttackRange = 200
  DamageType = ARMOR_PIERCING
  DeathType = NORMAL
  WeaponSpeed = 999999
  ClipSize = 1
  DelayBetweenShots = 100
End",
    ];
    for block in wpn_blocks {
        let mut wt = WeaponTemplate::new();
        wt.parse_ini_block(block).expect("parse weapon");
        sim.rules.weapon_store.add_template(wt);
    }

    let upg_player_mask = sim
        .rules
        .upgrade_center
        .find_upgrade("UpgPlayerTest", &mut sim.rules.name_keys)
        .expect("UpgPlayerTest")
        .upgrade_mask()
        .clone();
    let upg_object_mask = sim
        .rules
        .upgrade_center
        .find_upgrade("UpgObjectTest", &mut sim.rules.name_keys)
        .expect("UpgObjectTest")
        .upgrade_mask()
        .clone();
    let sci_a = sim
        .rules
        .science_store
        .friend_lookup_science("SCI_WP51_A", &mut sim.rules.name_keys)
        .expect("SCI_WP51_A");
    let sci_b = sim
        .rules
        .science_store
        .friend_lookup_science("SCI_WP51_B", &mut sim.rules.name_keys)
        .expect("SCI_WP51_B");

    let vet_names = ["REGULAR", "VETERAN", "ELITE", "HEROIC"];
    let slot_names = ["PRIMARY", "SECONDARY", "TERTIARY"];
    let team_a = TeamId(1);

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed") as usize;
        assert_eq!(seed, idx);

        let frame = get_u32(line, "frame").expect("frame");
        let upg_name = get_str(line, "upg").expect("upg");
        let exempt_uc = get_i32(line, "exempt_uc").expect("exempt_uc") != 0;
        let start_uc = get_i32(line, "start_uc").expect("start_uc") != 0;
        let build = get_i32(line, "build").expect("build") != 0;
        let build2 = get_i32(line, "build2").expect("build2") != 0;
        let lock_slot = get_i32(line, "lock_slot").expect("lock_slot") as usize;
        let preorder = get_i32(line, "preorder").expect("preorder") != 0;
        let vet_lvl = get_i32(line, "vet_lvl").expect("vet_lvl") as usize;
        let sci_name = get_str(line, "sci_name").expect("sci_name");
        let sci_a_known = get_i32(line, "sci_a_known").expect("sci_a_known") != 0;
        let sci_b_known = get_i32(line, "sci_b_known").expect("sci_b_known") != 0;

        let want_g_obj = get_i32_vec(line, "g_obj").expect("g_obj");
        let want_g_ply = get_i32_vec(line, "g_ply").expect("g_ply");
        let want_g_need = get_i32_vec(line, "g_need").expect("g_need");
        let want_grant_crc = get_u32(line, "grant_crc").expect("grant_crc");
        let want_grant_xfer = decode_hex_bytes(get_str(line, "grant_xfer").unwrap()).unwrap();
        let want_lock_slot = get_i32(line, "lock_slot_out").expect("lock_slot_out");
        let want_lock_need = get_i32(line, "lock_need").expect("lock_need");
        let want_lock_xfer = decode_hex_bytes(get_str(line, "lock_xfer").unwrap()).unwrap();
        let want_pre_flag = get_i32(line, "pre_flag").expect("pre_flag");
        let want_pre_need = get_i32(line, "pre_need").expect("pre_need");
        let want_pre_xfer = decode_hex_bytes(get_str(line, "pre_xfer").unwrap()).unwrap();
        let want_sp_need = get_i32(line, "sp_need").expect("sp_need");
        let want_sp_xfer = decode_hex_bytes(get_str(line, "sp_xfer").unwrap()).unwrap();
        let want_sc = get_i32_vec(line, "sc").expect("sc");
        let want_sw = get_i32(line, "sw").expect("sw");
        let want_sc_xfer = decode_hex_bytes(get_str(line, "sc_xfer").unwrap()).unwrap();
        let want_sw_xfer = decode_hex_bytes(get_str(line, "sw_xfer").unwrap()).unwrap();
        let want_vet_lvl = get_i32(line, "vet_lvl").expect("out vet_lvl"); // Note: matches "vet_lvl" key check below
        let _ = want_vet_lvl;
        let want_vet_need = get_i32_vec(line, "vet_need").expect("vet_need");
        let want_vet_xfer = decode_hex_bytes(get_str(line, "vet_xfer").unwrap()).unwrap();
        let want_xfer_crcs: Vec<u32> = get_i32_vec(line, "xfer_crcs")
            .expect("xfer_crcs")
            .into_iter()
            .map(|v| v as u32)
            .collect();

        // Sync world clock & player state matching C++ seed setup.
        sim.world.frame = frame;
        sim.world.player_upgrades.clear();
        sim.player_of_team_mut(team_a).expect("player").is_preorder = preorder;
        sim.world.registered_supply_centers.clear();
        sim.world.registered_supply_warehouses.clear();
        let sci_set = &mut sim.player_of_team_mut(team_a).expect("player").sciences;
        sci_set.clear();
        if sci_a_known {
            sci_set.push(sci_a);
        }
        if sci_b_known {
            sci_set.push(sci_b);
        }

        let pfx = format!("C51_{seed}_");
        let t_grant = format!("{pfx}Grant");
        let t_lock = format!("{pfx}Lock");
        let t_pre = format!("{pfx}Pre");
        let t_sp = format!("{pfx}SP");
        let t_sc = format!("{pfx}SC");
        let t_sw = format!("{pfx}SW");
        let t_vet = format!("{pfx}Vet");

        let exempt_line = if exempt_uc {
            "    ExemptStatus = UNDER_CONSTRUCTION\n"
        } else {
            ""
        };
        let sci_line = if !sci_name.is_empty() {
            format!("    ScienceRequired = {sci_name}\n")
        } else {
            String::new()
        };
        let ini_text = format!(
            "Object {t_grant}\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = GrantUpgradeCreate ModuleTag_Create\n\
                UpgradeToGrant = {upg_name}\n\
            {exempt_line}\
              End\nEnd\n\
            Object {t_lock}\n\
              WeaponSet\n    Conditions = None\n\
                Weapon = PRIMARY CreateWpnPrimary\n\
                Weapon = SECONDARY CreateWpnSecondary\n\
                Weapon = TERTIARY CreateWpnTertiary\n  End\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = LockWeaponCreate ModuleTag_Create\n\
                SlotToLock = {}\n  End\nEnd\n\
            Object {t_pre}\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = PreorderCreate ModuleTag_Create\n  End\nEnd\n\
            Object {t_sp}\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = SpecialPowerCreate ModuleTag_Create\n  End\nEnd\n\
            Object {t_sc}\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = SupplyCenterCreate ModuleTag_Create\n  End\nEnd\n\
            Object {t_sw}\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = SupplyWarehouseCreate ModuleTag_Create\n  End\nEnd\n\
            Object {t_vet}\n\
              IsTrainable = Yes\n\
              ExperienceRequired = 0 100 200 400\n\
              ExperienceValue = 10 20 30 40\n\
              Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
              Behavior = VeterancyGainCreate ModuleTag_Create\n\
                StartingLevel = {}\n\
            {sci_line}\
              End\nEnd\n",
            slot_names[lock_slot], vet_names[vet_lvl]
        );
        let mut reader = IniReader::new("create.ini", ini_text.as_bytes(), IniLoadType::Overwrite);
        sim.rules
            .load_thing_templates_from_ini(&mut reader)
            .expect("load WP-5.1-C templates");

        // 1) GrantUpgradeCreate
        let id_grant = sim.rules.thing_factory.find_template_id(&t_grant).unwrap();
        let mut init_st = ObjectStatusMaskType::new();
        if start_uc {
            init_st.set(ObjectStatusType::UnderConstruction.as_i32(), 1);
        }
        let o_grant = sim.spawn_object_with_status(id_grant, team_a, Coord3D::default(), 0.0, init_st);
        let has_obj_pre = sim
            .object(o_grant)
            .unwrap()
            .upgrades_completed
            .test_for_any(&upg_object_mask)
            != FALSE;
        let has_ply_pre = sim.player_has_upgrade(team_a, &upg_player_mask) != FALSE;
        let (g_need_pre, _, _, _) = inspect_create_module(&mut sim, o_grant);
        if build {
            sim.dispatch_on_build_complete(o_grant);
        }
        let has_obj_post = sim
            .object(o_grant)
            .unwrap()
            .upgrades_completed
            .test_for_any(&upg_object_mask)
            != FALSE;
        let has_ply_post = sim.player_has_upgrade(team_a, &upg_player_mask) != FALSE;
        let (g_need_post, _, _, _) = inspect_create_module(&mut sim, o_grant);
        if build2 {
            sim.dispatch_on_build_complete(o_grant);
        }
        let (_, g_bytes, g_crc, g_xcrc) = inspect_create_module(&mut sim, o_grant);
        assert_eq!(i32::from(has_obj_pre), want_g_obj[0], "seed {seed} g_obj[0]");
        assert_eq!(i32::from(has_obj_post), want_g_obj[1], "seed {seed} g_obj[1]");
        assert_eq!(i32::from(has_ply_pre), want_g_ply[0], "seed {seed} g_ply[0]");
        assert_eq!(i32::from(has_ply_post), want_g_ply[1], "seed {seed} g_ply[1]");
        assert_eq!(i32::from(g_need_pre != FALSE), want_g_need[0], "seed {seed} g_need[0]");
        assert_eq!(i32::from(g_need_post != FALSE), want_g_need[1], "seed {seed} g_need[1]");
        assert_eq!(g_crc, want_grant_crc, "seed {seed} grant_crc");
        assert_eq!(g_bytes, want_grant_xfer, "seed {seed} grant_xfer");
        assert_eq!(g_xcrc, want_xfer_crcs[0], "seed {seed} grant xfer_crc");

        // 2) LockWeaponCreate
        let id_lock = sim.rules.thing_factory.find_template_id(&t_lock).unwrap();
        let o_lock = sim.spawn_object(id_lock, team_a, Coord3D::default(), 0.0);
        if build {
            sim.dispatch_on_build_complete(o_lock);
        }
        let locked_slot = sim
            .object(o_lock)
            .unwrap()
            .weapon_set
            .get_cur_weapon_slot()
            .as_i32();
        let (l_need, l_bytes, _, l_xcrc) = inspect_create_module(&mut sim, o_lock);
        assert_eq!(locked_slot, want_lock_slot, "seed {seed} lock_slot_out");
        assert_eq!(i32::from(l_need != FALSE), want_lock_need, "seed {seed} lock_need");
        assert_eq!(l_bytes, want_lock_xfer, "seed {seed} lock_xfer");
        assert_eq!(l_xcrc, want_xfer_crcs[1], "seed {seed} lock xfer_crc");

        // 3) PreorderCreate
        let id_pre = sim.rules.thing_factory.find_template_id(&t_pre).unwrap();
        let o_pre = sim.spawn_object(id_pre, team_a, Coord3D::default(), 0.0);
        if build {
            sim.dispatch_on_build_complete(o_pre);
        }
        let pre_flag = sim
            .object(o_pre)
            .unwrap()
            .model_condition
            .test(ModelConditionFlagType::Preorder.as_i32())
            != FALSE;
        let (p_need, p_bytes, _, p_xcrc) = inspect_create_module(&mut sim, o_pre);
        assert_eq!(i32::from(pre_flag), want_pre_flag, "seed {seed} pre_flag");
        assert_eq!(i32::from(p_need != FALSE), want_pre_need, "seed {seed} pre_need");
        assert_eq!(p_bytes, want_pre_xfer, "seed {seed} pre_xfer");
        assert_eq!(p_xcrc, want_xfer_crcs[2], "seed {seed} pre xfer_crc");

        // 4) SpecialPowerCreate
        let id_sp = sim.rules.thing_factory.find_template_id(&t_sp).unwrap();
        let o_sp = sim.spawn_object(id_sp, team_a, Coord3D::default(), 0.0);
        if build {
            sim.dispatch_on_build_complete(o_sp);
        }
        let (sp_need, sp_bytes, _, sp_xcrc) = inspect_create_module(&mut sim, o_sp);
        assert_eq!(i32::from(sp_need != FALSE), want_sp_need, "seed {seed} sp_need");
        assert_eq!(sp_bytes, want_sp_xfer, "seed {seed} sp_xfer");
        assert_eq!(sp_xcrc, want_xfer_crcs[3], "seed {seed} sp xfer_crc");

        // 5) SupplyCenterCreate & 6) SupplyWarehouseCreate
        let id_sc = sim.rules.thing_factory.find_template_id(&t_sc).unwrap();
        let o_sc = sim.spawn_object(id_sc, team_a, Coord3D::default(), 0.0);
        let sc_pre = sim.world.registered_supply_centers.len() as i32;
        if build {
            sim.dispatch_on_build_complete(o_sc);
        }
        let sc_post = sim.world.registered_supply_centers.len() as i32;
        let (_, sc_bytes, _, sc_xcrc) = inspect_create_module(&mut sim, o_sc);

        let id_sw = sim.rules.thing_factory.find_template_id(&t_sw).unwrap();
        let o_sw = sim.spawn_object(id_sw, team_a, Coord3D::default(), 0.0);
        let sw_pre = sim.world.registered_supply_warehouses.len() as i32;
        let (_, sw_bytes, _, sw_xcrc) = inspect_create_module(&mut sim, o_sw);

        assert_eq!([sc_pre, sc_post], [want_sc[0], want_sc[1]], "seed {seed} sc");
        assert_eq!(sw_pre, want_sw, "seed {seed} sw");
        assert_eq!(sc_bytes, want_sc_xfer, "seed {seed} sc_xfer");
        assert_eq!(sw_bytes, want_sw_xfer, "seed {seed} sw_xfer");
        assert_eq!(sc_xcrc, want_xfer_crcs[4], "seed {seed} sc xfer_crc");
        assert_eq!(sw_xcrc, want_xfer_crcs[5], "seed {seed} sw xfer_crc");

        // 7) VeterancyGainCreate
        let id_vet = sim.rules.thing_factory.find_template_id(&t_vet).unwrap();
        let o_vet = sim.spawn_object(id_vet, team_a, Coord3D::default(), 0.0);
        let got_vet_lvl = sim
            .object(o_vet)
            .unwrap()
            .experience_tracker
            .get_veterancy_level()
            .as_i32();
        let (v_need_pre, _, _, _) = inspect_create_module(&mut sim, o_vet);
        if build {
            sim.dispatch_on_build_complete(o_vet);
        }
        let (v_need_post, v_bytes, _, v_xcrc) = inspect_create_module(&mut sim, o_vet);

        // Extract exact "vet_lvl" from "out" section of the vector line.
        let out_slice = &line[line.find("\"out\":").unwrap()..];
        let want_out_vet_lvl = get_i32(out_slice, "vet_lvl").expect("out vet_lvl");
        assert_eq!(got_vet_lvl, want_out_vet_lvl, "seed {seed} out vet_lvl");
        assert_eq!(i32::from(v_need_pre != FALSE), want_vet_need[0], "seed {seed} vet_need[0]");
        assert_eq!(i32::from(v_need_post != FALSE), want_vet_need[1], "seed {seed} vet_need[1]");
        assert_eq!(v_bytes, want_vet_xfer, "seed {seed} vet_xfer");
        assert_eq!(v_xcrc, want_xfer_crcs[6], "seed {seed} vet xfer_crc");

        // Verify parsed ScienceRequired matches expected interned ScienceType.
        let vet_tmpl = sim.rules.thing_factory.find_template(&t_vet).unwrap();
        let vet_nug = vet_tmpl
            .get_behavior_module_info()
            .get_nugget_with_tag("ModuleTag_Create")
            .unwrap();
        let vet_md = sim
            .rules
            .module_data_store
            .get(vet_nug.data_id)
            .unwrap()
            .as_any()
            .downcast_ref::<VeterancyGainCreateModuleData>()
            .unwrap();
        let expected_sci: ScienceType = match sci_name {
            "SCI_WP51_A" => sci_a,
            "SCI_WP51_B" => sci_b,
            _ => SCIENCE_INVALID,
        };
        assert_eq!(vet_md.science_required, expected_sci, "seed {seed} science_required");

        for s in [o_grant, o_lock, o_pre, o_sp, o_sc, o_sw, o_vet] {
            sim.destroy_object_now(s);
        }
        sim.process_destroy_list();
    }
}
