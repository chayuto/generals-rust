//! Gate V2-Golden test suite for `WP-5.2-B` (`UpgradeModule` hierarchy:
//! `UpgradeMux`, `UpgradeModule`, and all 20 concrete subclasses, 1,000 C++ reference vectors).

use zh_common::crc::Crc;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Int, NameKeyType, ObjectId, Real, TeamId, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleContext, ModuleDataId, ModuleSlot, ObjectSlot};
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::gamelogic::global_data::WeaponBonusConditionType;
use zh_logic::modules::module::UpgradeFacet;
use zh_logic::modules::upgrade::{
    ExperienceScalarUpgradeModuleData, UpgradeModule, UpgradeMuxData,
};
use zh_logic::rules::combat_types::{ArmorSetType, WeaponSetType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceTextLookup;
use zh_logic::rules::upgrade::UpgradeMaskType;
use zh_logic::sim::{MockSimApi, SimApi, SimContext, SimView};

const WP52_UPGRADE_JSON: &str = include_str!("../../../docs/golden_vectors/WP52_Upgrade.json");

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

fn find_upgrade_module(sim: &SimContext, owner: ObjectSlot, idx: usize) -> ModuleSlot {
    sim.object(owner)
        .expect("object exists")
        .behaviors
        .iter()
        .filter(|(_, m)| m.contains(FacetMask::UPGRADE))
        .nth(idx)
        .map(|&(s, _)| s)
        .expect("UpgradeModule attached")
}

fn find_sp_module(sim: &SimContext, owner: ObjectSlot) -> ModuleSlot {
    sim.object(owner)
        .expect("object exists")
        .behaviors
        .iter()
        .find(|(_, m)| m.contains(FacetMask::SPECIAL_POWER))
        .map(|&(s, _)| s)
        .expect("SpecialPowerModule attached")
}

fn upgrade_already(sim: &mut SimContext, slot: ModuleSlot) -> i32 {
    sim.dispatch_module(slot, |m, _, _| {
        i32::from(m.as_upgrade().unwrap().is_already_upgraded() != FALSE)
    })
    .unwrap()
}

fn upgrade_xfer_and_crc(sim: &mut SimContext, slot: ModuleSlot) -> (Vec<u8>, u32, u32) {
    sim.dispatch_module(slot, |m, sim_a, _ctx| {
        let mut xs = XferSave::new();
        m.xfer(&mut xs, sim_a).expect("upgrade xfer save");
        let bytes = xs.into_bytes();
        let xfer_crc = bytes_crc(&bytes);
        let mut xc = XferCrc::new();
        m.crc(&mut xc);
        (bytes, xc.get_crc(), xfer_crc)
    })
    .unwrap()
}

fn object_shroud_range(sim: &SimContext, slot: ObjectSlot) -> Real {
    let obj = sim.object(slot).unwrap();
    if obj.shroud_clearing_range_override >= 0.0 {
        return obj.shroud_clearing_range_override;
    }
    obj.thing
        .template_override()
        .get_final(&sim.rules.thing_factory)
        .map_or(0.0, |t| t.friend_calc_shroud_clearing_range().max(0.0))
}

fn object_command_set_string(sim: &SimContext, slot: ObjectSlot) -> String {
    let obj = sim.object(slot).unwrap();
    if !obj.command_set_override.is_empty() {
        return obj.command_set_override.clone();
    }
    obj.thing
        .template_override()
        .get_final(&sim.rules.thing_factory)
        .map(|t| t.friend_get_command_set_string().to_owned())
        .unwrap_or_default()
}

fn id_of(name: &str, sim: &SimContext) -> zh_logic::arena::ThingTemplateId {
    sim.rules.thing_factory.find_template_id(name).unwrap()
}

#[test]
fn test_wp52_upgrade_oracle_integrity_and_base_contract() {
    assert!(WP52_UPGRADE_JSON.contains("\"packet\": \"WP-5.2-B\""));
    assert!(WP52_UPGRADE_JSON.contains("\"seed_count\": 1000"));
    assert!(WP52_UPGRADE_JSON.contains("\"seed0_diagnostics\":"));
    let lines = extract_vector_lines(WP52_UPGRADE_JSON);
    assert_eq!(lines.len(), 1000);

    // Abstract UpgradeModule Xfer / CRC / isTriggeredBy contract check.
    let mut mux_data = UpgradeMuxData::new();
    mux_data.activation_upgrade_names.push("UpgAlpha".to_owned());
    mux_data.trigger_upgrade_names.push("UpgAlpha".to_owned());
    let mut um = UpgradeModule::new(NameKeyType(1), mux_data);
    assert_eq!(um.is_already_upgraded(), FALSE);
    assert_eq!(um.is_triggered_by("upgaLPHA"), TRUE);
    assert_eq!(um.is_triggered_by("UpgOther"), FALSE);

    let mut xs = XferSave::new();
    um.xfer(&mut xs).expect("base UpgradeModule save");
    assert_eq!(xs.as_bytes(), &[1, 1, 1, 1, 1, 0]);
    let mut xc = XferCrc::new();
    UpgradeModule::crc(&um, &mut xc);
    assert_eq!(xc.get_crc(), 2);

    um.mux.set_upgrade_executed(TRUE);
    let mut xs2 = XferSave::new();
    um.xfer(&mut xs2).expect("base UpgradeModule save upgraded");
    assert_eq!(xs2.as_bytes(), &[1, 1, 1, 1, 1, 1]);
    let mut xc2 = XferCrc::new();
    UpgradeModule::crc(&um, &mut xc2);
    assert_eq!(xc2.get_crc(), 3);

    let mut um2 = UpgradeModule::with_name_key(NameKeyType(1));
    let mut xl = XferLoad::new(xs2.as_bytes());
    um2.xfer(&mut xl).expect("base UpgradeModule load");
    assert_eq!(um2.is_already_upgraded(), TRUE);

    let mut dummy = MockSimApi::new();
    let dummy_ctx = ModuleContext {
        mod_slot: ModuleSlot::INVALID,
        owner_slot: ObjectSlot::INVALID,
        owner_id: ObjectId::INVALID,
        tag_key: NameKeyType(1),
        template_data_id: ModuleDataId::INVALID,
    };
    um2.force_refresh_upgrade(&mut dummy, dummy_ctx);
}

#[test]
fn test_wp52_power_plant_load_post_process_and_disabled_edge() {
    let mut sim = SimContext::new();
    let upg_ini = "\
Upgrade UpgA
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
";
    sim.rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new("upg.ini", upg_ini.as_bytes(), IniLoadType::Overwrite),
            &mut sim.rules.name_keys,
        )
        .unwrap();
    let obj_ini = "\
Object PPPlant
  EnergyProduction = 10
  EnergyBonus = 7
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = PowerPlantUpgrade ModuleTag_PP
    TriggeredBy = UpgA
  End
End
";
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "pp.ini",
            obj_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .unwrap();

    let team = TeamId(1);
    let o = sim.spawn_object(id_of("PPPlant", &sim), team, Coord3D::default(), 0.0);
    // Base EnergyProduction (10) vs bonus (7):
    sim.adjust_player_power(team, 10, TRUE);
    sim.grant_upgrade(o, "UpgA", FALSE);
    assert_eq!(sim.player_energy_production(team), 17);

    // Disabled edge removes base (10) + bonus (7), clear restores both.
    sim.set_disabled(o, DisabledType::Emp);
    assert_eq!(sim.player_energy_production(team), 0);
    assert_ne!(sim.clear_disabled(o, DisabledType::Emp), FALSE);
    assert_eq!(sim.player_energy_production(team), 17);

    // Clear player_energy and verify load_post_process restores the bonus when enabled.
    sim.world.player_energy.clear();
    let pp_slot = find_upgrade_module(&sim, o, 0);
    sim.dispatch_module(pp_slot, |m, sim_a, ctx| {
        m.load_post_process(sim_a, ctx);
    })
    .unwrap();
    assert_eq!(sim.player_energy_production(team), 7);

    sim.destroy_object_now(o);
    sim.process_destroy_list();
}

#[test]
fn test_wp52_status_bits_upgrade_active_set_and_clear() {
    let mut sim = SimContext::new();
    let upg_ini = "\
Upgrade UpgA
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
";
    sim.rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new("upg.ini", upg_ini.as_bytes(), IniLoadType::Overwrite),
            &mut sim.rules.name_keys,
        )
        .unwrap();
    let obj_ini = "\
Object StatActive
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = StatusBitsUpgrade ModuleTag_Stat
    TriggeredBy = UpgA
    StatusToSet = CAN_STEALTH IMMOBILE
    StatusToClear = NO_ATTACK
  End
End
";
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "stat.ini",
            obj_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .unwrap();

    let o = sim.spawn_object(id_of("StatActive", &sim), TeamId(1), Coord3D::default(), 0.0);
    sim.set_status(o, ObjectStatusType::NoAttack);
    sim.grant_upgrade(o, "UpgA", FALSE);
    let obj = sim.object(o).unwrap();
    assert_ne!(obj.status.test(ObjectStatusType::CanStealth.as_i32()), FALSE);
    assert_ne!(obj.status.test(ObjectStatusType::Immobile.as_i32()), FALSE);
    assert_eq!(obj.status.test(ObjectStatusType::NoAttack.as_i32()), FALSE);

    sim.destroy_object_now(o);
    sim.process_destroy_list();
}

#[test]
fn test_wp52_upgrade_1000_golden_vectors() {
    let lines = extract_vector_lines(WP52_UPGRADE_JSON);
    assert_eq!(lines.len(), 1000);

    let mut sim = SimContext::new();

    let upg_ini = "\
Upgrade UpgA
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade UpgB
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade UpgConflict
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade UpgRemovable
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade UpgPlayerAlt
  Type = PLAYER
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade UpgObjAlt
  Type = OBJECT
  BuildTime = 1.0
  BuildCost = 100
End
Upgrade Upgrade_AmericaChemicalSuits
  Type = PLAYER
  BuildTime = 1.0
  BuildCost = 100
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
Science SCI_WP52_A
  SciencePurchasePointCost = 1
  IsGrantable = Yes
End
Science SCI_WP52_B
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

    let sp_ini = "\
SpecialPower SpWP52_A
  Enum = SPECIAL_CASH_HACK
  ReloadTime = 1000
End
";
    sim.rules
        .special_power_store
        .load_from_ini(
            &mut IniReader::new("sp.ini", sp_ini.as_bytes(), IniLoadType::Overwrite),
            &sim.rules.science_store,
            &mut sim.rules.name_keys,
        )
        .expect("load special power");

    sim.rules.fx_list_names.insert("FX_WP52_Upg".to_owned());
    sim.rules.ocl_names.insert("OCL_WP52_Test".to_owned());
    for loco_name in ["LocoNormal", "LocoUpgraded"] {
        let mut lt = LocomotorTemplate::new();
        lt.name = loco_name.to_owned();
        sim.rules.locomotor_store.add_template(lt);
    }

    let shared_objs_ini = "\
Object WP52_Spawned
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
End
Object WP52_ReplTarget
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 250
    InitialHealth = 250
  End
  Behavior = SupplyCenterCreate ModuleTag_SC
  End
End
";
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "shared.ini",
            shared_objs_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("load shared templates");

    let team_a = TeamId(1);
    let team_b = TeamId(2);

    let ut_a_mask = sim
        .rules
        .upgrade_center
        .lookup_upgrade("UpgA", &sim.rules.name_keys)
        .unwrap()
        .upgrade_mask()
        .clone();
    let ut_b_mask = sim
        .rules
        .upgrade_center
        .lookup_upgrade("UpgB", &sim.rules.name_keys)
        .unwrap()
        .upgrade_mask()
        .clone();
    let mut ab_mask = ut_a_mask.clone();
    ab_mask.clear_and_set(&UpgradeMaskType::new(), &ut_b_mask);

    let sci_a = sim.rules.name_keys.lookup_key("SCI_WP52_A").unwrap().0;
    let sci_b = sim.rules.name_keys.lookup_key("SCI_WP52_B").unwrap().0;

    let change_type_names = [
        "SAME_CURRENTHEALTH",
        "PRESERVE_RATIO",
        "ADD_CURRENT_HEALTH_TOO",
        "FULLY_HEAL",
    ];
    let mc_flag_names = [
        "USER_1",
        "USER_2",
        "WEAPONSET_PLAYER_UPGRADE",
        "ARMORSET_CRATEUPGRADE_ONE",
    ];
    let mc_flag_enums = [
        ModelConditionFlagType::User1,
        ModelConditionFlagType::User2,
        ModelConditionFlagType::WeaponSetPlayerUpgrade,
        ModelConditionFlagType::ArmorSetCrateUpgradeOne,
    ];
    let kind_of_names = ["INFANTRY", "VEHICLE", "STRUCTURE"];
    let kind_of_enums = [
        KindOfType::Infantry,
        KindOfType::Vehicle,
        KindOfType::Structure,
    ];
    let pct_strs = ["-15%", "20%", "-25.5%", "10%"];

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed") as usize;
        assert_eq!(seed, idx);

        let frame0 = get_u32(line, "frame").unwrap();
        let req_all = get_i32(line, "req_all").unwrap() != 0;
        let give_conflict_first = get_i32(line, "conflict_first").unwrap() != 0;
        let chem_armor = get_i32(line, "chem_armor").unwrap() != 0;
        let cmd_alt_mode = get_i32(line, "cmd_alt_mode").unwrap();
        let cost_ko_idx = get_i32(line, "cost_ko_idx").unwrap() as usize;
        let cost_pct_idx = get_i32(line, "cost_pct_idx").unwrap() as usize;
        let cost_capture = get_i32(line, "cost_capture").unwrap() != 0;
        let xp_add = f32::from_bits(get_u32(line, "xp_add_bits").unwrap());
        let grant_sci_b = get_i32(line, "grant_sci_b").unwrap() != 0;
        let hp_change_idx = get_i32(line, "hp_change_idx").unwrap() as usize;
        let hp_add = f32::from_bits(get_u32(line, "hp_add_bits").unwrap());
        let hp_dmg = f32::from_bits(get_u32(line, "hp_dmg_bits").unwrap());
        let mc_idx = get_i32(line, "mc_idx").unwrap() as usize;
        let shroud_range = f32::from_bits(get_u32(line, "shroud_bits").unwrap());
        let radar_dp = get_i32(line, "radar_dp").unwrap() != 0;
        let radar_dis_del = get_i32(line, "radar_dis_del").unwrap() != 0;
        let sp_ready = get_i32(line, "sp_ready").unwrap() != 0;
        let sub_conflict = get_i32(line, "sub_conflict").unwrap() != 0;

        let want_mux = get_i32_vec(line, "mux").unwrap();
        let want_xp = get_i32_vec(line, "xp_bits").unwrap();
        let want_mux_crc = get_u32(line, "mux_crc").unwrap();
        let want_mux_xfer_crc = get_u32(line, "mux_xfer_crc").unwrap();
        let want_mux_xfer = decode_hex_bytes(get_str(line, "mux_xfer").unwrap()).unwrap();
        let want_shroud = get_i32_vec(line, "shroud_out").unwrap();
        let want_shroud_xfer = decode_hex_bytes(get_str(line, "shroud_xfer").unwrap()).unwrap();
        let want_armor_flag = get_i32(line, "armor_flag").unwrap();
        let want_armor_xfer = decode_hex_bytes(get_str(line, "armor_xfer").unwrap()).unwrap();
        let want_cmd_str = get_str(line, "cmd_str").unwrap();
        let want_cost = get_i32_vec(line, "cost_bits").unwrap();
        let want_cost_pre_del = get_i32(line, "cost_pre_del").unwrap();
        let want_sci = get_i32_vec(line, "sci").unwrap();
        let want_loco = get_i32_vec(line, "loco").unwrap();
        let want_hp = get_i32_vec(line, "hp_bits").unwrap();
        let want_mc_set = get_i32(line, "mc_set").unwrap();
        let want_ocl = get_i32_vec(line, "ocl").unwrap();
        let want_pass = get_i32_vec(line, "pass").unwrap();
        let want_pp = get_i32_vec(line, "pp").unwrap();
        let want_rad = get_i32_vec(line, "rad").unwrap();
        let want_repl = get_i32_vec(line, "repl").unwrap();
        let want_stat = get_i32_vec(line, "stat").unwrap();
        let want_stl = get_i32(line, "stl").unwrap();
        let want_sub = get_i32_vec(line, "sub").unwrap();
        let want_sub_xfer = decode_hex_bytes(get_str(line, "sub_xfer").unwrap()).unwrap();
        let want_sp = get_i32_vec(line, "sp").unwrap();
        let want_wb = get_i32(line, "wb").unwrap();
        let want_ws = get_i32(line, "ws").unwrap();

        sim.world.frame = frame0;
        sim.world.player_upgrades.clear();
        sim.world.player_sciences.clear();
        sim.world.player_radars.clear();
        sim.world.player_energy.clear();
        sim.world.player_cost_modifiers.clear();
        sim.world.registered_supply_centers.clear();
        sim.world.spawned_ocls.clear();
        sim.world.triggered_fx_lists.clear();

        let pfx = format!("U52_{seed}_");
        let t_mux = format!("{pfx}Mux");
        let t_shroud = format!("{pfx}Shroud");
        let t_armor = format!("{pfx}Armor");
        let t_cmd = format!("{pfx}Cmd");
        let t_cost = format!("{pfx}Cost");
        let t_sci = format!("{pfx}Sci");
        let t_loco = format!("{pfx}Loco");
        let t_hp = format!("{pfx}HP");
        let t_mc = format!("{pfx}MC");
        let t_ocl = format!("{pfx}OCL");
        let t_pass = format!("{pfx}Pass");
        let t_pp = format!("{pfx}PP");
        let t_rad = format!("{pfx}Rad");
        let t_repl = format!("{pfx}Repl");
        let t_stat = format!("{pfx}Stat");
        let t_stl = format!("{pfx}Stl");
        let t_sub = format!("{pfx}Sub");
        let t_sp = format!("{pfx}SP");
        let t_wb = format!("{pfx}WB");
        let t_ws = format!("{pfx}WS");

        let cmd_trigger_alt_line = if cmd_alt_mode == 1 {
            "    TriggerAlt = UpgPlayerAlt\n"
        } else if cmd_alt_mode >= 2 {
            "    TriggerAlt = UpgObjAlt\n"
        } else {
            ""
        };
        let ini_text = format!(
            "Object {t_mux}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ExperienceScalarUpgrade ModuleTag_Mux
    TriggeredBy = UpgA UpgB
    ConflictsWith = UpgConflict
    RemovesUpgrades = UpgRemovable
    FXListUpgrade = FX_WP52_Upg
    RequiresAllTriggers = {req_all_str}
    AddXPScalar = {xp_add}
  End
  Behavior = StatusBitsUpgrade ModuleTag_Rem
    TriggeredBy = UpgRemovable
    StatusToSet = IMMOBILE
  End
End
Object {t_shroud}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ActiveShroudUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    NewShroudRange = {shroud_range}
  End
End
Object {t_armor}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ArmorUpgrade ModuleTag_Upg
    TriggeredBy = {armor_trig}
  End
End
Object {t_cmd}
  CommandSet = CmdBase
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = CommandSetUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    CommandSet = CmdMain
    CommandSetAlt = CmdAlt
{cmd_trigger_alt_line}  End
End
Object {t_cost}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = CostModifierUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    EffectKindOf = {cost_ko}
    Percentage = {cost_pct}
  End
End
Object {t_sci}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = GrantScienceUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    GrantScience = {grant_sci}
  End
End
Object {t_loco}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL LocoNormal
  Locomotor = SET_NORMAL_UPGRADED LocoUpgraded
  Behavior = LocomotorSetUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
  End
End
Object {t_hp}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = MaxHealthUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    AddMaxHealth = {hp_add}
    ChangeType = {hp_ct}
  End
End
Object {t_mc}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ModelConditionUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    ConditionFlag = {mc_flag}
  End
End
Object {t_ocl}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ObjectCreationUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    UpgradeObject = OCL_WP52_Test
  End
End
Object {t_pass}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = OpenContain ModuleTag_Contain
    ContainMax = 4
  End
  Behavior = PassengersFireUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
  End
End
Object {t_pp}
  EnergyBonus = 7
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = PowerPlantUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
  End
End
Object {t_rad}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = RadarUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    DisableProof = {rad_dp_str}
  End
End
Object {t_repl}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = ReplaceObjectUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    ReplaceObject = WP52_ReplTarget
  End
End
Object {t_stat}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = StatusBitsUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    StatusToSet = CAN_STEALTH IMMOBILE
    StatusToClear = UNDER_CONSTRUCTION
  End
End
Object {t_stl}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = StealthUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
  End
End
Object {t_sub}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SubObjectsUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    ConflictsWith = UpgConflict
    ShowSubObjects = TurretA Barrel1
    ShowSubObjects = Barrel2
    HideSubObjects = OldMeshA
    HideSubObjects = OldMeshB
  End
End
Object {t_sp}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = CashHackSpecialPower ModuleTag_SP
    SpecialPowerTemplate = SpWP52_A
    StartsPaused = Yes
    StartsReady = {sp_ready_str}
  End
  Behavior = UnpauseSpecialPowerUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
    SpecialPowerTemplate = SpWP52_A
  End
End
Object {t_wb}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = WeaponBonusUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
  End
End
Object {t_ws}
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = WeaponSetUpgrade ModuleTag_Upg
    TriggeredBy = UpgA
  End
End
",
            req_all_str = if req_all { "Yes" } else { "No" },
            armor_trig = if chem_armor {
                "Upgrade_AmericaChemicalSuits"
            } else {
                "UpgA"
            },
            cost_ko = kind_of_names[cost_ko_idx],
            cost_pct = pct_strs[cost_pct_idx],
            grant_sci = if grant_sci_b {
                "SCI_WP52_B"
            } else {
                "SCI_WP52_A"
            },
            hp_ct = change_type_names[hp_change_idx],
            mc_flag = mc_flag_names[mc_idx],
            rad_dp_str = if radar_dp { "Yes" } else { "No" },
            sp_ready_str = if sp_ready { "Yes" } else { "No" },
        );
        sim.rules
            .load_thing_templates_from_ini(&mut IniReader::new(
                "seed.ini",
                ini_text.as_bytes(),
                IniLoadType::Overwrite,
            ))
            .expect("load seed templates");

        // --- Exercise 1: Mux + ExperienceScalarUpgrade + Removable StatusBitsUpgrade ---
        let o_mux = sim.spawn_object(id_of(&t_mux, &sim), team_a, Coord3D::default(), 0.0);
        let um_mux = find_upgrade_module(&sim, o_mux, 0);
        let um_rem = find_upgrade_module(&sim, o_mux, 1);
        sim.grant_upgrade(o_mux, "UpgRemovable", FALSE);
        let rem_pre = upgrade_already(&mut sim, um_rem);
        if give_conflict_first {
            sim.grant_upgrade(o_mux, "UpgConflict", FALSE);
        }
        let (would_a, would_ab, cond_ab, is_sub_false, trig_a_true, trig_x_false) = sim
            .dispatch_module(um_mux, |m, sim_a, ctx| {
                let u = m.as_upgrade().unwrap();
                let wa = i32::from(u.would_upgrade(sim_a, ctx, &ut_a_mask) != FALSE);
                let wab = i32::from(u.would_upgrade(sim_a, ctx, &ab_mask) != FALSE);
                let cab = i32::from(u.test_upgrade_conditions(sim_a, ctx, &ab_mask) != FALSE);
                let sub_f = i32::from(u.is_sub_objects_upgrade() != FALSE);
                let md = sim_a
                    .module_data(ctx.template_data_id)
                    .unwrap()
                    .as_any()
                    .downcast_ref::<ExperienceScalarUpgradeModuleData>()
                    .unwrap();
                let ta = i32::from(md.mux_data.is_triggered_by("UpgA") != FALSE);
                let tx = i32::from(md.mux_data.is_triggered_by("UpgUnknown") != FALSE);
                (wa, wab, cab, sub_f, ta, tx)
            })
            .unwrap();

        sim.grant_upgrade(o_mux, "UpgA", FALSE);
        let upg_after_a = upgrade_already(&mut sim, um_mux);
        sim.grant_upgrade(o_mux, "UpgB", FALSE);
        let upg_after_ab = upgrade_already(&mut sim, um_mux);
        let rem_after = upgrade_already(&mut sim, um_rem);
        let xp_post1 = sim
            .object(o_mux)
            .unwrap()
            .experience_tracker
            .experience_scalar
            .to_bits() as i32;
        sim.dispatch_module(um_mux, |m, sim_a, ctx| {
            m.as_upgrade_mut().unwrap().force_refresh_upgrade(sim_a, ctx);
        })
        .unwrap();
        let xp_post2 = sim
            .object(o_mux)
            .unwrap()
            .experience_tracker
            .experience_scalar
            .to_bits() as i32;
        if upg_after_ab != 0 {
            assert!(
                sim.world
                    .triggered_fx_lists
                    .iter()
                    .any(|(fx, _)| fx == "FX_WP52_Upg"),
                "seed {seed} expected FXListUpgrade trigger"
            );
        }

        let (mux_bytes, mux_crc, mux_xfer_crc) = upgrade_xfer_and_crc(&mut sim, um_mux);
        sim.remove_upgrade(o_mux, "UpgA");
        let upg_after_reset = upgrade_already(&mut sim, um_mux);

        assert_eq!(
            [
                would_a,
                would_ab,
                cond_ab,
                is_sub_false,
                upg_after_a,
                upg_after_ab,
                rem_pre,
                rem_after,
                upg_after_reset,
                trig_a_true,
                trig_x_false,
            ],
            want_mux[..],
            "seed {seed} mux"
        );
        assert_eq!([xp_post1, xp_post2], want_xp[..], "seed {seed} xp");
        assert_eq!(mux_crc, want_mux_crc, "seed {seed} mux_crc");
        assert_eq!(mux_xfer_crc, want_mux_xfer_crc, "seed {seed} mux_xfer_crc");
        assert_eq!(mux_bytes, want_mux_xfer, "seed {seed} mux_xfer");

        // --- Exercise 2: ActiveShroudUpgrade ---
        let o_shroud = sim.spawn_object(id_of(&t_shroud, &sim), team_a, Coord3D::default(), 0.0);
        let shroud_pre = object_shroud_range(&sim, o_shroud).to_bits() as i32;
        sim.grant_upgrade(o_shroud, "UpgA", FALSE);
        let shroud_post = object_shroud_range(&sim, o_shroud).to_bits() as i32;
        let um_shroud = find_upgrade_module(&sim, o_shroud, 0);
        let (shroud_bytes, _, _) = upgrade_xfer_and_crc(&mut sim, um_shroud);
        assert_eq!(
            [shroud_pre, shroud_post],
            want_shroud[..],
            "seed {seed} shroud"
        );
        assert_eq!(shroud_bytes, want_shroud_xfer, "seed {seed} shroud_xfer");

        // --- Exercise 3: ArmorUpgrade ---
        let o_armor = sim.spawn_object(id_of(&t_armor, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(
            o_armor,
            if chem_armor {
                "Upgrade_AmericaChemicalSuits"
            } else {
                "UpgA"
            },
            FALSE,
        );
        let bs_armor = sim.object(o_armor).unwrap().body_slot.unwrap();
        let armor_flag = sim
            .dispatch_module(bs_armor, |m, _, _| {
                i32::from(
                    m.as_body()
                        .unwrap()
                        .test_armor_set_flag(ArmorSetType::PlayerUpgrade)
                        != FALSE,
                )
            })
            .unwrap();
        let um_armor = find_upgrade_module(&sim, o_armor, 0);
        let (armor_bytes, _, _) = upgrade_xfer_and_crc(&mut sim, um_armor);
        assert_eq!(armor_flag, want_armor_flag, "seed {seed} armor_flag");
        assert_eq!(armor_bytes, want_armor_xfer, "seed {seed} armor_xfer");

        // --- Exercise 4: CommandSetUpgrade ---
        let o_cmd = sim.spawn_object(id_of(&t_cmd, &sim), team_a, Coord3D::default(), 0.0);
        if cmd_alt_mode == 1 {
            sim.grant_player_upgrade(team_a, "UpgPlayerAlt");
        } else if cmd_alt_mode == 2 {
            sim.grant_upgrade(o_cmd, "UpgObjAlt", FALSE);
        }
        sim.grant_upgrade(o_cmd, "UpgA", FALSE);
        let cmd_str = object_command_set_string(&sim, o_cmd);
        assert_eq!(cmd_str, want_cmd_str, "seed {seed} cmd_str");

        // --- Exercise 5: CostModifierUpgrade ---
        let o_cost = sim.spawn_object(id_of(&t_cost, &sim), team_a, Coord3D::default(), 0.0);
        let mut ko_test = KindOfMaskType::new();
        ko_test.set(kind_of_enums[cost_ko_idx].as_i32(), 1);
        let cost_pre = sim.production_cost_change(team_a, &ko_test).to_bits() as i32;
        sim.grant_upgrade(o_cost, "UpgA", FALSE);
        let cost_post = sim.production_cost_change(team_a, &ko_test).to_bits() as i32;
        let mut cost_cap_a = cost_post;
        let mut cost_cap_b = sim.production_cost_change(team_b, &ko_test).to_bits() as i32;
        if cost_capture {
            sim.capture_object(o_cost, team_b);
            cost_cap_a = sim.production_cost_change(team_a, &ko_test).to_bits() as i32;
            cost_cap_b = sim.production_cost_change(team_b, &ko_test).to_bits() as i32;
        }
        let um_cost = find_upgrade_module(&sim, o_cost, 0);
        let cost_pre_del = upgrade_already(&mut sim, um_cost);
        sim.destroy_object_now(o_cost);
        sim.process_destroy_list();
        let cost_after_del_a = sim.production_cost_change(team_a, &ko_test).to_bits() as i32;
        let cost_after_del_b = sim.production_cost_change(team_b, &ko_test).to_bits() as i32;
        assert_eq!(
            [
                cost_pre,
                cost_post,
                cost_cap_a,
                cost_cap_b,
                cost_after_del_a,
                cost_after_del_b,
            ],
            want_cost[..],
            "seed {seed} cost_bits"
        );
        assert_eq!(cost_pre_del, want_cost_pre_del, "seed {seed} cost_pre_del");

        // --- Exercise 6: GrantScienceUpgrade ---
        let o_sci = sim.spawn_object(id_of(&t_sci, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(o_sci, "UpgA", FALSE);
        let has_sci_a = i32::from(sim.player_has_science(team_a, sci_a) != FALSE);
        let has_sci_b = i32::from(sim.player_has_science(team_a, sci_b) != FALSE);
        assert_eq!([has_sci_a, has_sci_b], want_sci[..], "seed {seed} sci");

        // --- Exercise 7: LocomotorSetUpgrade ---
        let o_loco = sim.spawn_object(id_of(&t_loco, &sim), team_a, Coord3D::default(), 0.0);
        let loco_pre = sim.object(o_loco).unwrap().cur_locomotor_set.as_i32();
        sim.grant_upgrade(o_loco, "UpgA", FALSE);
        let loco_post = sim.object(o_loco).unwrap().cur_locomotor_set.as_i32();
        assert_eq!([loco_pre, loco_post], want_loco[..], "seed {seed} loco");

        // --- Exercise 8: MaxHealthUpgrade ---
        let o_hp = sim.spawn_object(id_of(&t_hp, &sim), team_a, Coord3D::default(), 0.0);
        let bs_hp = sim.object(o_hp).unwrap().body_slot.unwrap();
        sim.dispatch_module(bs_hp, |m, sim_a, ctx| {
            m.as_body_mut()
                .unwrap()
                .internal_change_health(sim_a, ctx, -hp_dmg);
        })
        .unwrap();
        sim.grant_upgrade(o_hp, "UpgA", FALSE);
        let (hp_post, max_hp_post) = sim
            .dispatch_module(bs_hp, |m, _, _| {
                let b = m.as_body().unwrap();
                (b.get_health().to_bits() as i32, b.get_max_health().to_bits() as i32)
            })
            .unwrap();
        assert_eq!([hp_post, max_hp_post], want_hp[..], "seed {seed} hp");

        // --- Exercise 9: ModelConditionUpgrade ---
        let o_mc = sim.spawn_object(id_of(&t_mc, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(o_mc, "UpgA", FALSE);
        let mc_set = i32::from(
            sim.object(o_mc)
                .unwrap()
                .model_condition
                .test(mc_flag_enums[mc_idx].as_i32())
                != FALSE,
        );
        assert_eq!(mc_set, want_mc_set, "seed {seed} mc_set");

        // --- Exercise 10: ObjectCreationUpgrade ---
        let o_ocl = sim.spawn_object(id_of(&t_ocl, &sim), team_a, Coord3D::default(), 0.0);
        let o_ocl_id = sim.object(o_ocl).unwrap().id;
        sim.world.spawned_ocls.clear();
        sim.grant_upgrade(o_ocl, "UpgA", FALSE);
        if want_ocl[1] - want_ocl[0] == 1 {
            assert_eq!(
                sim.world.spawned_ocls.len(),
                1,
                "seed {seed} ocl spawned count"
            );
            assert_eq!(sim.world.spawned_ocls[0].0, "OCL_WP52_Test");
            assert_eq!(sim.world.spawned_ocls[0].1, o_ocl_id);
        }

        // --- Exercise 11: PassengersFireUpgrade ---
        let o_pass = sim.spawn_object(id_of(&t_pass, &sim), team_a, Coord3D::default(), 0.0);
        let pass_pre = i32::from(sim.object(o_pass).unwrap().passengers_allowed_to_fire != FALSE);
        sim.grant_upgrade(o_pass, "UpgA", FALSE);
        let pass_post = i32::from(sim.object(o_pass).unwrap().passengers_allowed_to_fire != FALSE);
        assert_eq!([pass_pre, pass_post], want_pass[..], "seed {seed} pass");

        // --- Exercise 12: PowerPlantUpgrade ---
        let o_pp = sim.spawn_object(id_of(&t_pp, &sim), team_a, Coord3D::default(), 0.0);
        let pp_pre_a = sim.player_energy_production(team_a);
        sim.grant_upgrade(o_pp, "UpgA", FALSE);
        let pp_post_a = sim.player_energy_production(team_a);
        sim.capture_object(o_pp, team_b);
        let pp_cap_a = sim.player_energy_production(team_a);
        let pp_cap_b = sim.player_energy_production(team_b);
        sim.destroy_object_now(o_pp);
        sim.process_destroy_list();
        let pp_del_b = sim.player_energy_production(team_b);
        assert_eq!(
            [pp_pre_a, pp_post_a, pp_cap_a, pp_cap_b, pp_del_b],
            want_pp[..],
            "seed {seed} pp"
        );

        // --- Exercise 13: RadarUpgrade ---
        let o_rad = sim.spawn_object(id_of(&t_rad, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(o_rad, "UpgA", FALSE);
        let (rad_a, dprad_a) = sim.player_radar_counts(team_a);
        sim.capture_object(o_rad, team_b);
        let (rad_cap_a, _) = sim.player_radar_counts(team_a);
        let (rad_cap_b, _) = sim.player_radar_counts(team_b);
        if radar_dis_del {
            sim.set_disabled(o_rad, DisabledType::Emp);
        }
        sim.destroy_object_now(o_rad);
        sim.process_destroy_list();
        let (rad_del_b, _) = sim.player_radar_counts(team_b);
        assert_eq!(
            [rad_a, dprad_a, rad_cap_a, rad_cap_b, rad_del_b],
            want_rad[..],
            "seed {seed} rad"
        );

        // --- Exercise 14: ReplaceObjectUpgrade ---
        let o_repl = sim.spawn_object(
            id_of(&t_repl, &sim),
            team_a,
            Coord3D::new(120.0, 240.0, 0.0),
            1.25,
        );
        let sc_before = sim.world.registered_supply_centers.len() as i32;
        sim.grant_upgrade(o_repl, "UpgA", FALSE);
        let repl_old_dest = i32::from(sim.object(o_repl).unwrap().is_destroyed());
        let sc_after = sim.world.registered_supply_centers.len() as i32;
        assert_eq!(
            [repl_old_dest, sc_before, sc_after],
            want_repl[..],
            "seed {seed} repl"
        );

        // --- Exercise 15: StatusBitsUpgrade ---
        let o_stat = sim.spawn_object(id_of(&t_stat, &sim), team_a, Coord3D::default(), 0.0);
        sim.set_status(o_stat, ObjectStatusType::UnderConstruction);
        sim.grant_upgrade(o_stat, "UpgA", FALSE);
        let stat_obj = sim.object(o_stat).unwrap();
        let stat_stealth = i32::from(stat_obj.status.test(ObjectStatusType::CanStealth.as_i32()) != FALSE);
        let stat_immob = i32::from(stat_obj.status.test(ObjectStatusType::Immobile.as_i32()) != FALSE);
        let stat_uc = i32::from(
            stat_obj
                .status
                .test(ObjectStatusType::UnderConstruction.as_i32())
                != FALSE,
        );
        assert_eq!(
            [stat_stealth, stat_immob, stat_uc],
            want_stat[..],
            "seed {seed} stat"
        );

        // --- Exercise 16: StealthUpgrade ---
        let o_stl = sim.spawn_object(id_of(&t_stl, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(o_stl, "UpgA", FALSE);
        let stl_flag = i32::from(
            sim.object(o_stl)
                .unwrap()
                .status
                .test(ObjectStatusType::CanStealth.as_i32())
                != FALSE,
        );
        assert_eq!(stl_flag, want_stl, "seed {seed} stl");

        // --- Exercise 17: SubObjectsUpgrade ---
        let o_sub = sim.spawn_object(id_of(&t_sub, &sim), team_a, Coord3D::default(), 0.0);
        let um_sub = find_upgrade_module(&sim, o_sub, 0);
        let is_sub_true = sim
            .dispatch_module(um_sub, |m, _, _| {
                i32::from(m.as_upgrade().unwrap().is_sub_objects_upgrade() != FALSE)
            })
            .unwrap();
        if sub_conflict {
            sim.grant_upgrade(o_sub, "UpgConflict", FALSE);
        }
        sim.grant_upgrade(o_sub, "UpgA", FALSE);
        let sub_upg_done = upgrade_already(&mut sim, um_sub);
        let (sub_bytes, _, _) = upgrade_xfer_and_crc(&mut sim, um_sub);
        assert_eq!([is_sub_true, sub_upg_done], want_sub[..], "seed {seed} sub");
        assert_eq!(sub_bytes, want_sub_xfer, "seed {seed} sub_xfer");
        if sub_upg_done != 0 {
            let sub_obj = sim.object(o_sub).unwrap();
            assert!(sub_obj.sub_objects_shown.contains("TurretA"));
            assert!(sub_obj.sub_objects_shown.contains("Barrel1"));
            assert!(sub_obj.sub_objects_shown.contains("Barrel2"));
            assert!(sub_obj.sub_objects_hidden.contains("OldMeshA"));
            assert!(sub_obj.sub_objects_hidden.contains("OldMeshB"));
        }

        // --- Exercise 18: UnpauseSpecialPowerUpgrade ---
        let o_sp = sim.spawn_object(id_of(&t_sp, &sim), team_a, Coord3D::default(), 0.0);
        let sp_mod = find_sp_module(&sim, o_sp);
        let sp_paused_pre = sim
            .dispatch_module(sp_mod, |m, _, _| {
                i32::from(m.as_special_power().unwrap().is_paused() != FALSE)
            })
            .unwrap();
        sim.grant_upgrade(o_sp, "UpgA", FALSE);
        let (sp_paused_post, sp_ready_post) = sim
            .dispatch_module(sp_mod, |m, sim_a, ctx| {
                let sp = m.as_special_power().unwrap();
                (
                    i32::from(sp.is_paused() != FALSE),
                    sp.get_ready_frame(sim_a, ctx) as i32,
                )
            })
            .unwrap();
        assert_eq!(
            [sp_paused_pre, sp_paused_post, sp_ready_post],
            want_sp[..],
            "seed {seed} sp"
        );

        // --- Exercise 19 & 20: WeaponBonusUpgrade & WeaponSetUpgrade ---
        let o_wb = sim.spawn_object(id_of(&t_wb, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(o_wb, "UpgA", FALSE);
        let wb_bit = WeaponBonusConditionType::PlayerUpgrade.as_i32();
        let wb_flag = i32::from((sim.object(o_wb).unwrap().weapon_bonus_condition & (1u32 << wb_bit)) != 0);
        assert_eq!(wb_flag, want_wb, "seed {seed} wb");

        let o_ws = sim.spawn_object(id_of(&t_ws, &sim), team_a, Coord3D::default(), 0.0);
        sim.grant_upgrade(o_ws, "UpgA", FALSE);
        let ws_flag = i32::from(
            sim.object(o_ws)
                .unwrap()
                .weapon_set_flags
                .test(WeaponSetType::PlayerUpgrade.as_i32())
                != FALSE,
        );
        assert_eq!(ws_flag, want_ws, "seed {seed} ws");

        // Verify XferLoad roundtrip & seed0_diagnostics checkpoints for seed 0.
        if seed == 0 {
            let s0_line = WP52_UPGRADE_JSON
                .lines()
                .find(|l| l.contains("\"seed0_diagnostics\":"))
                .expect("seed0_diagnostics line");
            let sub_slice = |key: &str| -> &str {
                let pat = format!("\"{key}\":{{");
                let start = s0_line.find(&pat).expect("section start") + pat.len() - 1;
                let rest = &s0_line[start..];
                let end = rest.find('}').expect("section end");
                &rest[..=end]
            };
            let s_mux = sub_slice("mux");
            assert_eq!(get_i32(s_mux, "would_a").unwrap(), would_a);
            assert_eq!(get_i32(s_mux, "would_ab").unwrap(), would_ab);
            assert_eq!(get_i32(s_mux, "cond_ab").unwrap(), cond_ab);
            assert_eq!(get_i32(s_mux, "upg_a").unwrap(), upg_after_a);
            assert_eq!(get_i32(s_mux, "upg_ab").unwrap(), upg_after_ab);
            assert_eq!(get_i32(s_mux, "rem_pre").unwrap(), rem_pre);
            assert_eq!(get_i32(s_mux, "rem_after").unwrap(), rem_after);
            assert_eq!(get_i32(s_mux, "xp_post1_bits").unwrap(), xp_post1);
            assert_eq!(get_i32(s_mux, "xp_post2_bits").unwrap(), xp_post2);
            assert_eq!(get_i32(s_mux, "upg_reset").unwrap(), upg_after_reset);
            assert_eq!(get_i32(s_mux, "trig_a").unwrap(), trig_a_true);
            assert_eq!(get_i32(s_mux, "trig_x").unwrap(), trig_x_false);
            assert_eq!(decode_hex_bytes(get_str(s_mux, "xfer").unwrap()).unwrap(), mux_bytes);
            assert_eq!(get_u32(s_mux, "crc").unwrap(), mux_crc);
            let s_pp = sub_slice("pp");
            assert_eq!(get_i32(s_pp, "post_a").unwrap(), pp_post_a);
            assert_eq!(get_i32(s_pp, "cap_b").unwrap(), pp_cap_b);
            assert_eq!(get_i32(s_pp, "del_b").unwrap(), pp_del_b);
            let s_rad = sub_slice("rad");
            assert_eq!(get_i32(s_rad, "dp_a").unwrap(), dprad_a);
            assert_eq!(get_i32(s_rad, "cap_b").unwrap(), rad_cap_b);
            assert_eq!(get_i32(s_rad, "del_b").unwrap(), rad_del_b);
            let s_sp = sub_slice("sp");
            assert_eq!(get_i32(s_sp, "paused_pre").unwrap(), sp_paused_pre);
            assert_eq!(get_i32(s_sp, "paused_post").unwrap(), sp_paused_post);
            assert_eq!(get_i32(s_sp, "ready").unwrap(), sp_ready_post);
            let s_sub = sub_slice("sub");
            assert_eq!(get_i32(s_sub, "is_sub").unwrap(), is_sub_true);
            assert_eq!(get_i32(s_sub, "done").unwrap(), sub_upg_done);
            assert_eq!(decode_hex_bytes(get_str(s_sub, "xfer").unwrap()).unwrap(), sub_bytes);

            for bytes in [&mux_bytes, &shroud_bytes, &armor_bytes, &sub_bytes] {
                let mut tmp = UpgradeModule::with_name_key(NameKeyType(1));
                let mut ver: XferVersion = 1;
                let mut xl = XferLoad::new(bytes);
                xl.xfer_version(&mut ver, 1).expect("subclass ver");
                tmp.xfer(&mut xl).expect("upgrade xfer load");
                assert_eq!(tmp.is_already_upgraded(), TRUE);
            }
        }

        // Destroy all live objects before next seed.
        let to_kill: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
        for s in to_kill {
            sim.destroy_object_now(s);
        }
        sim.process_destroy_list();
    }
}
