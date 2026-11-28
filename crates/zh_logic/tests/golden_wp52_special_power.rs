//! Gate V2-Golden test suite for `WP-5.2-C` (`SpecialPowerModule` hierarchy:
//! `SpecialPowerModule`, `SpecialAbility`, `DemoralizeSpecialPower`, and all 9 concrete
//! special-power modules, 1,000 C++ reference vectors).

use std::collections::BTreeSet;
use zh_common::crc::Crc;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32, get_u32_array,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{NameKeyType, ObjectId, TeamId, FALSE, TRUE};
use zh_common::xfer::{XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{
    DeferredSelfOp, FacetMask, ModuleContext, ModuleDataId, ModuleSidecar, ModuleSlot, ObjectSlot,
};
use zh_logic::modules::module::{BehaviorModule, SpecialPowerFacet};
use zh_logic::modules::special_power::{
    register_demoralize_special_power, OCLSpecialPowerModuleData, OclCreateLocType,
    SpecialPowerModule, SpecialPowerModuleData, SpyVisionSpecialPower,
};
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceTextLookup;
use zh_logic::rules::special_power_types::SpecialPowerType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{MockSimApi, SimApi, SimContext, SimView};

struct DummyBehavior(NameKeyType);

impl BehaviorModule for DummyBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.0
    }

    fn xfer(&mut self, _xfer: &mut dyn zh_common::xfer::Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
}

const WP52_SP_JSON: &str = include_str!("../../../docs/golden_vectors/WP52_SpecialPower.json");

struct TestScienceText;

impl ScienceTextLookup for TestScienceText {
    fn fetch_label(&mut self, label: &str) -> String {
        format!("TXT:{label}")
    }
}

fn bytes_crc(bytes: &[u8]) -> u32 {
    let mut c = Crc::new();
    c.compute_crc(bytes, bytes.len() as i32);
    c.get()
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

fn sp_xfer_and_crc(sim: &mut SimContext, slot: ModuleSlot) -> (Vec<u8>, u32, u32) {
    sim.dispatch_module(slot, |m, sim_a, _ctx| {
        let mut xs = XferSave::new();
        m.xfer(&mut xs, sim_a).expect("sp xfer save");
        let bytes = xs.into_bytes();
        let xfer_crc = bytes_crc(&bytes);
        let mut xc = XferCrc::new();
        m.crc(&mut xc);
        (bytes, xc.get_crc(), xfer_crc)
    })
    .unwrap()
}

fn live_obj_count(sim: &SimContext) -> i32 {
    sim.world
        .objects
        .world_iter()
        .filter(|&s| sim.object(s).is_some_and(|o| !o.is_destroyed()))
        .count() as i32
}

fn id_of(name: &str, sim: &SimContext) -> zh_logic::arena::ThingTemplateId {
    sim.rules.thing_factory.find_template_id(name).unwrap()
}

fn load_base_rules(sim: &mut SimContext) {
    let sci_ini = "\
Science SCI_BOUNTY
  SciencePurchasePointCost = 1
  IsGrantable = Yes
End
Science SCI_CASH_UPG
  SciencePurchasePointCost = 1
  IsGrantable = Yes
End
Science SCI_OCL_UPG
  SciencePurchasePointCost = 1
  IsGrantable = Yes
End
Science SCI_SHARED_REQ
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

    let mut wt = WeaponTemplate::new();
    wt.parse_ini_block(
        "Weapon WP52_Wpn\n  PrimaryDamage = 15\n  AttackRange = 100\n  DamageType = SMALL_ARMS\n  DeathType = NORMAL\n  WeaponSpeed = 999999\n  ClipSize = 2\n  ClipReloadTime = 1000\n  DelayBetweenShots = 100\nEnd",
    )
    .expect("parse weapon");
    sim.rules.weapon_store.add_template(wt);

    let sp_ini = "\
SpecialPower Sp_Normal
  Enum = SPECIAL_CASH_HACK
  ReloadTime = 900
  InitiateSound = NoSound
End
SpecialPower Sp_Ready
  Enum = SPECIAL_SPY_SATELLITE
  ReloadTime = 600
  ViewObjectRange = 200
  ViewObjectDuration = 150
End
SpecialPower Sp_Shared
  Enum = SPECIAL_PARADROP_AMERICA
  ReloadTime = 1200
  SharedSyncedTimer = Yes
  PublicTimer = No
  RequiredScience = SCI_SHARED_REQ
End
SpecialPower Sp_Bounty
  Enum = SPECIAL_CASH_BOUNTY
  ReloadTime = 0
  RequiredScience = SCI_BOUNTY
End
SpecialPower Sp_Defect
  Enum = SPECIAL_DEFECTOR
  ReloadTime = 1500
  DetectionTime = 240
End
SpecialPower Sp_Baikonur
  Enum = SPECIAL_LAUNCH_BAIKONUR_ROCKET
  ReloadTime = 1800
End
SpecialPower Sp_Cleanup
  Enum = SPECIAL_CLEANUP_AREA
  ReloadTime = 0
End
SpecialPower Sp_FireWep
  Enum = SPECIAL_DAISY_CUTTER
  ReloadTime = 300
End
SpecialPower Sp_Script
  Enum = SPECIAL_CARPET_BOMB
  ReloadTime = 450
End
";
    sim.rules
        .special_power_store
        .load_from_ini(
            &mut IniReader::new("sp.ini", sp_ini.as_bytes(), IniLoadType::Overwrite),
            &sim.rules.science_store,
            &mut sim.rules.name_keys,
        )
        .expect("load special powers");

    sim.rules.ocl_names.insert("OCL_WP52_Default".to_owned());
    sim.rules.ocl_names.insert("OCL_WP52_Upgrade".to_owned());

    let objs_ini = "\
Object WP52_Detonation
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
End
Object WP52_OCLDefault
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
End
Object WP52_OCLUpgrade
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
End
Object Obj_SP_Basic
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpecialAbility ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    StartsPaused = No
    StartsReady = No
    ScriptedSpecialPowerOnly = No
  End
End
Object Obj_SP_Ready
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpecialAbility ModuleTag_SP
    SpecialPowerTemplate = Sp_Ready
    StartsPaused = No
    StartsReady = Yes
    InitiateSound = NoSound
  End
End
Object Obj_SP_Paused
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpecialAbility ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    StartsPaused = Yes
    StartsReady = Yes
  End
End
Object Obj_SP_SharedStruct
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpecialAbility ModuleTag_SP
    SpecialPowerTemplate = Sp_Shared
    StartsPaused = No
    StartsReady = No
  End
End
Object Obj_SP_Script
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpecialAbility ModuleTag_SP
    SpecialPowerTemplate = Sp_Script
    ScriptedSpecialPowerOnly = Yes
  End
End
Object Obj_Baikonur
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = BaikonurLaunchPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Baikonur
    DetonationObject = WP52_Detonation
  End
End
Object Obj_CashBounty_Low
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = CashBountyPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Bounty
    Bounty = 10%
  End
End
Object Obj_CashBounty_High
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = CashBountyPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Bounty
    Bounty = 25%
  End
End
Object Obj_CashHack
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = CashHackSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    UpgradeMoneyAmount = SCI_CASH_UPG 2500
    MoneyAmount = 1000
  End
End
Object Obj_Cleanup
  KindOf = VEHICLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = CleanupHazardUpdate ModuleTag_CHU
    WeaponSlot = PRIMARY
  End
  Behavior = CleanupAreaPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Cleanup
    MaxMoveDistanceFromLocation = 225.0
  End
End
Object Obj_Defector
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = DefectorSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Defect
    FatCursorRadius = 45.0
  End
End
Object Obj_TargetInf
  KindOf = INFANTRY SELECTABLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Obj_FireWep
  KindOf = CAN_ATTACK SELECTABLE
  WeaponSet
    Conditions = None
    Weapon = PRIMARY WP52_Wpn
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = FireWeaponPower ModuleTag_SP
    SpecialPowerTemplate = Sp_FireWep
    MaxShotsToFire = 3
  End
End
Object Obj_OCL_Loc
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = OCLSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    OCL = OCL_WP52_Default
    UpgradeOCL = SCI_OCL_UPG OCL_WP52_Upgrade
    CreateLocation = CREATE_AT_LOCATION
    ReferenceObject = WP52_Detonation
    OCLAdjustPositionToPassable = No
  End
End
Object Obj_OCL_Above
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = OCLSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    OCL = OCL_WP52_Default
    UpgradeOCL = SCI_OCL_UPG OCL_WP52_Upgrade
    CreateLocation = CREATE_ABOVE_LOCATION
    ReferenceObject = WP52_Detonation
    OCLAdjustPositionToPassable = Yes
  End
End
Object Obj_OCL_Owner
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = OCLSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    OCL = OCL_WP52_Default
    CreateLocation = USE_OWNER_OBJECT
  End
End
Object Obj_SpyVision
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpyVisionUpdate ModuleTag_SVU
    NeedsUpgrade = No
  End
  Behavior = SpyVisionSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Ready
    BaseDuration = 5000
    BonusDurationPerCaptured = 2000
    MaxDuration = 12000
  End
End
";
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "objs.ini",
            objs_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("load objects");
}

#[test]
fn test_wp52_special_power_oracle_integrity_and_base_contract() {
    assert!(WP52_SP_JSON.contains("\"packet\": \"WP-5.2-C\""));
    assert!(WP52_SP_JSON.contains("\"demoralize_compiled\": 0"));
    assert!(WP52_SP_JSON.contains("\"seed_count\": 1000"));
    assert!(WP52_SP_JSON.contains("\"seed0_diagnostics\":"));
    let lines = extract_vector_lines(WP52_SP_JSON);
    assert_eq!(lines.len(), 1000);

    // Base SpecialPowerModule Xfer / CRC / sidecar / deferred op / starts_ready / escape-path checks.
    let mut data = SpecialPowerModuleData::new(NameKeyType(1), NameKeyType(2));
    data.special_power_id = 7;
    data.starts_ready = TRUE;
    let mut sp = SpecialPowerModule::new(data);
    sp.available_on_frame = 42;
    sp.paused_count = 1;
    sp.paused_on_frame = 10;
    sp.paused_percent = 0.5;

    assert_eq!(
        sp.sync_sidecar(),
        ModuleSidecar::SpecialPower {
            template_id: 7,
            available_on_frame: 42,
            paused_count: 1,
            paused_on_frame: 10,
            paused_percent: 0.5,
            starts_ready: TRUE,
        }
    );

    let mut xs = XferSave::new();
    sp.xfer(&mut xs).expect("base SpecialPowerModule save");
    assert_eq!(
        xs.as_bytes().len(),
        1 + 1 + 1 + 1 + 4 + 4 + 4 + 4 // version(sp + beh + obj + mod) + 4 fields
    );
    let mut xc = XferCrc::new();
    sp.crc(&mut xc);
    assert_eq!(xc.get_crc(), 0);

    let mut sp_loaded = SpecialPowerModule::with_name_key(NameKeyType(2));
    let mut xl = XferLoad::new(xs.as_bytes());
    sp_loaded.xfer(&mut xl).expect("base SpecialPowerModule load");
    assert_eq!(sp_loaded.available_on_frame, 42);
    assert_eq!(sp_loaded.paused_count, 1);
    assert_eq!(sp_loaded.paused_on_frame, 10);
    assert_eq!(sp_loaded.paused_percent, 0.5);

    let mut dummy = MockSimApi::new();
    let dummy_ctx = ModuleContext {
        mod_slot: ModuleSlot::INVALID,
        owner_slot: ObjectSlot::INVALID,
        owner_id: ObjectId::INVALID,
        tag_key: NameKeyType(1),
        template_data_id: ModuleDataId::INVALID,
    };
    #[cfg(feature = "retail-crc")]
    assert_eq!(sp.starts_ready(&dummy, dummy_ctx), FALSE);
    #[cfg(not(feature = "retail-crc"))]
    assert_eq!(sp.starts_ready(&dummy, dummy_ctx), TRUE);

    sp_loaded.apply_deferred_op(
        DeferredSelfOp::SpecialPowerPause { pause: FALSE },
        &mut dummy,
        dummy_ctx,
    );
    assert_eq!(sp_loaded.is_paused(), FALSE);
    sp_loaded.apply_deferred_op(
        DeferredSelfOp::SpecialPowerSetReadyFrame(u32::MAX),
        &mut dummy,
        dummy_ctx,
    );
    assert_eq!(sp_loaded.get_raw_ready_frame(), u32::MAX);
    // Preserve 0xFFFFFFFF escape path in do_special_power_at_location.
    sp_loaded.do_special_power_at_location(&mut dummy, dummy_ctx, &Coord3D::default(), 0.0, 0);
    assert_eq!(sp_loaded.get_raw_ready_frame(), u32::MAX);

    // PublicTimer + SpecialPowerUpdate intent + create_view_object shroud reveal checks.
    let mut sim = SimContext::new();
    assert!(sim
        .rules
        .module_registry
        .find_descriptor("DemoralizeSpecialPower", zh_logic::modules::ModuleType::Behavior)
        .is_none());
    let mut text = TestScienceText;
    sim.rules
        .science_store
        .load_from_ini(
            &mut IniReader::new(
                "s.ini",
                b"Science SCI_PUB\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n",
                IniLoadType::Overwrite,
            ),
            &mut sim.rules.name_keys,
            &mut text,
        )
        .unwrap();
    sim.rules
        .special_power_store
        .load_from_ini(
            &mut IniReader::new(
                "sp.ini",
                b"SpecialPower Sp_Pub\n  Enum = SPECIAL_SPY_SATELLITE\n  ReloadTime = 600\n  SharedSyncedTimer = Yes\n  PublicTimer = Yes\n  ViewObjectRange = 175\n  ViewObjectDuration = 120\nEnd\n",
                IniLoadType::Overwrite,
            ),
            &sim.rules.science_store,
            &mut sim.rules.name_keys,
        )
        .unwrap();
    let tmpl_ini = "\
Object PubStruct
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpecialAbilityUpdate ModuleTag_SPU
  End
  Behavior = SpecialAbility ModuleTag_SP
    SpecialPowerTemplate = Sp_Pub
    UpdateModuleStartsAttack = No
  End
End
Object ViewObj
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = DeletionUpdate ModuleTag_Del
    MinLifetime = 10
    MaxLifetime = 10
  End
End
";
    sim.rules.global_data.special_power_view_object_name = "ViewObj".to_owned();
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "t.ini",
            tmpl_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .unwrap();

    let obj = sim.spawn_object(id_of("PubStruct", &sim), TeamId(1), Coord3D::default(), 0.0);
    sim.world.attach_module(
        obj,
        NameKeyType(900),
        ModuleDataId::INVALID,
        FacetMask::SPECIAL_POWER_UPDATE,
        SleepyUpdatePhase::Normal,
        Box::new(DummyBehavior(NameKeyType(900))),
    );
    assert_eq!(sim.world.public_timer_events.len(), 1);
    assert_eq!(sim.world.public_timer_events[0].3, TRUE);
    let sp_ms = find_sp_module(&sim, obj);
    let _ = sim.dispatch_module(sp_ms, |m, sim_a, ctx| {
        let sp_f = m.as_special_power_mut().unwrap();
        assert_eq!(
            sp_f.has_special_power(sim_a, ctx, SpecialPowerType::SpySatellite),
            TRUE
        );
        sp_f.do_special_power_at_location(sim_a, ctx, &Coord3D::new(40.0, 50.0, 0.0), 0.0, 0);
    });
    assert_eq!(sim.world.special_ability_events.len(), 1);
    let view_slot = sim
        .world
        .objects
        .world_iter()
        .find(|&s| s != obj)
        .expect("view object spawned");
    let view_obj = sim.object(view_slot).unwrap();
    assert_eq!(*view_obj.thing.get_position(), Coord3D::new(40.0, 50.0, 0.0));
    assert_eq!(sim.object_shroud_clearing_range(view_slot), 175.0);
    let del_slot = view_obj.behaviors.last().unwrap().0;
    assert_eq!(sim.get_wake_frame(del_slot).0, 4);
    let _ = sim.dispatch_module(sp_ms, |m, sim_a, ctx| {
        m.load_post_process(sim_a, ctx);
    });
    assert_eq!(sim.world.public_timer_events.len(), 2);
    sim.destroy_object_now(obj);
    sim.process_destroy_list();
    assert_eq!(sim.world.public_timer_events.len(), 3);
    assert_eq!(sim.world.public_timer_events[2].3, FALSE);
}

#[test]
fn test_wp52_demoralize_special_power_and_subclass_contracts() {
    let mut sim = SimContext::new();
    // The harness world: 20-unit partition cells over its 1000 by 1000 terrain.
    // The units below stand at y = 0. A position on the map edge is off the map, so
    // the map starts further out.
    sim.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(-100.0, -100.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    register_demoralize_special_power(&mut sim.rules.module_registry, &mut sim.rules.name_keys);
    sim.rules.fx_list_names.insert("FX_Demoralize".to_owned());
    load_base_rules(&mut sim);

    let extra_ini = "\
Object Obj_Demoralize
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = DemoralizeSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Normal
    BaseRange = 100.0
    BonusRangePerCaptured = 50.0
    MaxRange = 200.0
    BaseDuration = 3000
    BonusDurationPerCaptured = 1000
    MaxDuration = 4500
    FXList = FX_Demoralize
  End
End
Object Obj_TargetVeh
  KindOf = VEHICLE SELECTABLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Obj_OCL_AllLocs
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = OCLSpecialPower ModuleTag_EdgeSrc
    SpecialPowerTemplate = Sp_Normal
    OCL = OCL_WP52_Default
    CreateLocation = CREATE_AT_EDGE_NEAR_SOURCE
  End
  Behavior = OCLSpecialPower ModuleTag_EdgeTgt
    SpecialPowerTemplate = Sp_Ready
    OCL = OCL_WP52_Default
    CreateLocation = CREATE_AT_EDGE_NEAR_TARGET
  End
  Behavior = OCLSpecialPower ModuleTag_EdgeFar
    SpecialPowerTemplate = Sp_FireWep
    OCL = OCL_WP52_Default
    CreateLocation = CREATE_AT_EDGE_FARTHEST_FROM_TARGET
  End
End
Object Obj_SpyVision_Contain
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = SpyVisionSpecialPower ModuleTag_SP
    SpecialPowerTemplate = Sp_Ready
    BaseDuration = 5000
    BonusDurationPerCaptured = 2000
    MaxDuration = 12000
  End
End
";
    sim.rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "extra.ini",
            extra_ini.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .unwrap();

    // Verify all 6 OCL CreateLocation enum parses.
    let ocl_tt = sim.rules.thing_factory.find_template("Obj_OCL_AllLocs").unwrap();
    let d_src = sim
        .rules
        .module_data_store
        .get(
            ocl_tt
                .get_behavior_module_info()
                .get_nugget_with_tag("ModuleTag_EdgeSrc")
                .unwrap()
                .data_id,
        )
        .unwrap()
        .as_any()
        .downcast_ref::<OCLSpecialPowerModuleData>()
        .unwrap();
    assert_eq!(d_src.create_loc, OclCreateLocType::CreateAtEdgeNearSource);

    // DemoralizeSpecialPower with Contain occupancy = 2 -> range = min(100+100, 200) = 200,
    // duration = min(90 + 60, 135) = 135 frames.
    let team_a = TeamId(1);
    let team_b = TeamId(2);
    let team_n = TeamId(0);
    sim.world.set_player_relation(1, 2, zh_common::types::Relationship::Enemies);
    sim.world.set_player_relation(2, 1, zh_common::types::Relationship::Enemies);
    let src = sim.spawn_object(id_of("Obj_Demoralize", &sim), team_a, Coord3D::default(), 0.0);
    // The templates carry no `OpenContain` behavior: a real container would claim
    // `contain_slot` first and hide this stand-in's occupancy of 2.
    assert!(sim.object(src).unwrap().contain_slot.is_none());
    let contain_slot = sim.world.attach_module(
        src,
        NameKeyType(901),
        ModuleDataId::INVALID,
        FacetMask::CONTAIN,
        SleepyUpdatePhase::Normal,
        Box::new(DummyBehavior(NameKeyType(901))),
    );
    sim.world.modules.set_sidecar(
        contain_slot,
        ModuleSidecar::Contain {
            count: 2,
            max: 5,
            enclosing: TRUE,
            stealth_units: 0,
            garrisoned: FALSE,
            garrisonable: FALSE,
            rider_id: ObjectId::INVALID,
        },
    );

    let enemy_inf_in = sim.spawn_object(
        id_of("Obj_TargetInf", &sim),
        team_b,
        Coord3D::new(150.0, 0.0, 0.0),
        0.0,
    );
    let neutral_inf_in = sim.spawn_object(
        id_of("Obj_TargetInf", &sim),
        team_n,
        Coord3D::new(50.0, 0.0, 0.0),
        0.0,
    );
    let ally_inf_in = sim.spawn_object(
        id_of("Obj_TargetInf", &sim),
        team_a,
        Coord3D::new(20.0, 0.0, 0.0),
        0.0,
    );
    let enemy_veh_in = sim.spawn_object(
        id_of("Obj_TargetVeh", &sim),
        team_b,
        Coord3D::new(30.0, 0.0, 0.0),
        0.0,
    );
    let enemy_inf_out = sim.spawn_object(
        id_of("Obj_TargetInf", &sim),
        team_b,
        Coord3D::new(250.0, 0.0, 0.0),
        0.0,
    );
    for unit in [
        enemy_inf_in,
        neutral_inf_in,
        ally_inf_in,
        enemy_veh_in,
        enemy_inf_out,
    ] {
        sim.world.attach_module(
            unit,
            NameKeyType(902),
            ModuleDataId::INVALID,
            FacetMask::AI_UPDATE,
            SleepyUpdatePhase::Normal,
            Box::new(DummyBehavior(NameKeyType(902))),
        );
    }

    let dem_ms = find_sp_module(&sim, src);
    sim.update_partition();
    let _ = sim.dispatch_module(dem_ms, |m, sim_a, ctx| {
        m.as_special_power_mut().unwrap().do_special_power_at_location(
            sim_a,
            ctx,
            &Coord3D::default(),
            0.0,
            0,
        );
    });
    let dem_ids: BTreeSet<ObjectId> = sim
        .world
        .demoralized_events
        .iter()
        .map(|&(id, frames)| {
            assert_eq!(frames, 135);
            id
        })
        .collect();
    assert!(dem_ids.contains(&sim.object(enemy_inf_in).unwrap().id));
    assert!(dem_ids.contains(&sim.object(neutral_inf_in).unwrap().id));
    assert!(!dem_ids.contains(&sim.object(ally_inf_in).unwrap().id));
    assert!(!dem_ids.contains(&sim.object(enemy_veh_in).unwrap().id));
    assert!(!dem_ids.contains(&sim.object(enemy_inf_out).unwrap().id));
    assert_eq!(sim.world.triggered_fx_lists.len(), 1);
    assert_eq!(sim.world.triggered_fx_lists[0].0, "FX_Demoralize");

    // Verify SpyVisionSpecialPower contain-scaling duration (Base 5s=150, + 3*2s=180 = 330 <= 360).
    let sv = sim.spawn_object(
        id_of("Obj_SpyVision_Contain", &sim),
        team_a,
        Coord3D::default(),
        0.0,
    );
    let sv_contain = sim.world.attach_module(
        sv,
        NameKeyType(903),
        ModuleDataId::INVALID,
        FacetMask::CONTAIN,
        SleepyUpdatePhase::Normal,
        Box::new(DummyBehavior(NameKeyType(903))),
    );
    sim.world.modules.set_sidecar(
        sv_contain,
        ModuleSidecar::Contain {
            count: 3,
            max: 5,
            enclosing: TRUE,
            stealth_units: 0,
            garrisoned: FALSE,
            garrisonable: FALSE,
            rider_id: ObjectId::INVALID,
        },
    );
    let sv_ms = find_sp_module(&sim, sv);
    let _ = sim.dispatch_module(sv_ms, |m, sim_a, ctx| {
        let sv_mod = m
            .as_any()
            .and_then(|a| a.downcast_ref::<SpyVisionSpecialPower>())
            .expect("SpyVisionSpecialPower");
        assert_eq!(sv_mod.compute_duration(sim_a, ctx.owner_slot), 330);
        m.as_special_power_mut().unwrap().do_special_power(sim_a, ctx, 0);
    });

    // Verify BaikonurLaunchPower, CleanupAreaPower, FireWeaponPower, and OCLSpecialPower targetless/above events.
    let ocl_above = sim.spawn_object(id_of("Obj_OCL_Above", &sim), team_a, Coord3D::default(), 0.0);
    let ocl_ms = find_sp_module(&sim, ocl_above);
    let _ = sim.dispatch_module(ocl_ms, |m, sim_a, ctx| {
        m.as_special_power_mut().unwrap().do_special_power_at_location(
            sim_a,
            ctx,
            &Coord3D::new(77.0, 88.0, 10.0),
            0.0,
            0,
        );
    });
    let last_ocl = sim.world.spawned_ocls.last().unwrap();
    assert_eq!(last_ocl.0, "OCL_WP52_Default");
    assert_eq!(last_ocl.2, Coord3D::new(77.0, 88.0, 310.0));
    assert_eq!(last_ocl.3, Coord3D::new(77.0, 88.0, 10.0));

    let cln = sim.spawn_object(id_of("Obj_Cleanup", &sim), team_a, Coord3D::default(), 0.0);
    let cln_ms = find_sp_module(&sim, cln);
    let _ = sim.dispatch_module(cln_ms, |m, sim_a, ctx| {
        m.as_special_power_mut().unwrap().do_special_power_at_location(
            sim_a,
            ctx,
            &Coord3D::new(12.0, 34.0, 0.0),
            0.0,
            0,
        );
    });
    assert_eq!(sim.world.cleanup_area_events.len(), 1);
    assert_eq!(sim.world.cleanup_area_events[0].1, Coord3D::new(12.0, 34.0, 0.0));

    let fw = sim.spawn_object(id_of("Obj_FireWep", &sim), team_a, Coord3D::default(), 0.0);
    let fw_ms = find_sp_module(&sim, fw);
    let _ = sim.dispatch_module(fw_ms, |m, sim_a, ctx| {
        m.as_special_power_mut().unwrap().do_special_power_at_object(
            sim_a,
            ctx,
            enemy_inf_in,
            0,
        );
    });
    // The power reloads and orders an attack; it fires nothing itself. The order is
    // compared with the reference in `golden_s13_weapon_firing`.
    assert_eq!(sim.world.weapon_fires.len(), 0);

    let baik = sim.spawn_object(id_of("Obj_Baikonur", &sim), team_a, Coord3D::default(), 0.0);
    let baik_ms = find_sp_module(&sim, baik);
    let _ = sim.dispatch_module(baik_ms, |m, sim_a, ctx| {
        m.as_special_power_mut().unwrap().do_special_power(sim_a, ctx, 0);
    });
    assert_eq!(sim.world.baikonur_launch_events.len(), 1);
}

#[test]
fn test_wp52_special_power_1000_golden_vectors() {
    let lines = extract_vector_lines(WP52_SP_JSON);
    assert_eq!(lines.len(), 1000);

    let mut sim = SimContext::new();
    load_base_rules(&mut sim);

    let team_a = TeamId(1);
    let team_b = TeamId(2);
    sim.world.set_player_relation(1, 2, zh_common::types::Relationship::Enemies);
    sim.world.set_player_relation(2, 1, zh_common::types::Relationship::Enemies);

    let sp_normal_id = sim
        .rules
        .special_power_store
        .find_special_power_template("Sp_Normal")
        .unwrap()
        .id();
    let sp_ready_id = sim
        .rules
        .special_power_store
        .find_special_power_template("Sp_Ready")
        .unwrap()
        .id();
    let sp_shared_id = sim
        .rules
        .special_power_store
        .find_special_power_template("Sp_Shared")
        .unwrap()
        .id();
    let sp_bounty_id = sim
        .rules
        .special_power_store
        .find_special_power_template("Sp_Bounty")
        .unwrap()
        .id();
    let bounty_req_sci = sim
        .rules
        .special_power_store
        .find_special_power_template_by_id(sp_bounty_id)
        .unwrap()
        .required_science();

    let sci_bounty = sim.rules.name_keys.lookup_key("SCI_BOUNTY").unwrap().0;
    let sci_cash_upg = sim.rules.name_keys.lookup_key("SCI_CASH_UPG").unwrap().0;
    let sci_ocl_upg = sim.rules.name_keys.lookup_key("SCI_OCL_UPG").unwrap().0;

    let t_basic = id_of("Obj_SP_Basic", &sim);
    let t_ready = id_of("Obj_SP_Ready", &sim);
    let t_paused = id_of("Obj_SP_Paused", &sim);
    let t_script = id_of("Obj_SP_Script", &sim);
    let t_shared = id_of("Obj_SP_SharedStruct", &sim);
    let t_baik = id_of("Obj_Baikonur", &sim);
    let t_cb_low = id_of("Obj_CashBounty_Low", &sim);
    let t_cb_high = id_of("Obj_CashBounty_High", &sim);
    let t_ch = id_of("Obj_CashHack", &sim);
    let t_cln = id_of("Obj_Cleanup", &sim);
    let t_def = id_of("Obj_Defector", &sim);
    let t_tgt = id_of("Obj_TargetInf", &sim);
    let t_fw = id_of("Obj_FireWep", &sim);
    let t_ocl_loc = id_of("Obj_OCL_Loc", &sim);
    let t_ocl_above = id_of("Obj_OCL_Above", &sim);
    let t_sv = id_of("Obj_SpyVision", &sim);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed") as usize;
        assert_eq!(seed, expected_seed);

        let frame0 = get_u32(line, "frame").unwrap();
        let step_dt = get_u32(line, "dt").unwrap();
        let custom_ready = get_u32(line, "custom_ready").unwrap();
        let under_const = get_i32(line, "under_const").unwrap() != 0;
        let disabled_flag = get_i32(line, "disabled").unwrap() != 0;
        let has_bounty_sci = get_i32(line, "has_bounty_sci").unwrap() != 0;
        let has_cash_upg = get_i32(line, "has_cash_upg").unwrap() != 0;
        let has_ocl_upg = get_i32(line, "has_ocl_upg").unwrap() != 0;
        let ocl_above = get_i32(line, "ocl_above").unwrap() != 0;
        let ocl_reject = get_i32(line, "ocl_reject").unwrap() != 0;
        let money_a = get_u32(line, "money_a").unwrap();
        let money_b = get_u32(line, "money_b").unwrap();
        let loc_x = f32::from_bits(get_u32(line, "loc_x_bits").unwrap());
        let loc_y = f32::from_bits(get_u32(line, "loc_y_bits").unwrap());

        let want_sp: [u32; 21] = get_u32_array(line, "sp").unwrap();
        let want_sp_crc = get_u32(line, "sp_crc").unwrap();
        let want_sp_xcrc = get_u32(line, "sp_xcrc").unwrap();
        let want_sp_hex = get_str(line, "sp_xfer").unwrap();
        let want_baik: [u32; 3] = get_u32_array(line, "baik").unwrap();
        let want_baik_hex = get_str(line, "baik_xfer").unwrap();
        let want_cb: [u32; 7] = get_u32_array(line, "cb").unwrap();
        let want_ch: [u32; 4] = get_u32_array(line, "ch").unwrap();
        let want_cln = get_u32(line, "cln").unwrap();
        let want_def: [u32; 4] = get_u32_array(line, "def").unwrap();
        let want_fw = get_u32(line, "fw").unwrap();
        let want_ocl: [u32; 3] = get_u32_array(line, "ocl").unwrap();
        let want_ocl_ref = get_str(line, "ocl_ref").unwrap();
        let want_ocl_name = get_str(line, "ocl_name").unwrap();
        let want_sv = get_u32(line, "sv").unwrap();

        // Reset player state deterministically
        for p in 0..16 {
            let player = sim.world.players.player_mut(p).expect("player");
            player.sciences.clear();
            player.cash_bounty_percent = 0.0;
            player.special_power_timers.clear();
        }
        let (player_a, player_b) = (sim.player_index(team_a), sim.player_index(team_b));
        sim.set_player_cash(player_a, money_a);
        sim.set_player_cash(player_b, money_b);
        sim.player_of_team_mut(team_a)
            .expect("player")
            .express_special_power_ready_frame(sp_shared_id, frame0);

        if has_bounty_sci {
            sim.grant_player_science(team_a, sci_bounty);
        }
        if has_cash_upg {
            sim.grant_player_science(team_a, sci_cash_upg);
        }
        if has_ocl_upg {
            sim.grant_player_science(team_a, sci_ocl_upg);
        }

        sim.world.frame = frame0;

        // --- 1) SpecialPowerModule & SpecialAbility ---
        let mut init_status = Default::default();
        if under_const {
            init_status = zh_logic::rules::object_status_types::ObjectStatusMaskType::default();
            init_status.set(ObjectStatusType::UnderConstruction.as_i32(), 1);
        }
        let o_basic = sim.spawn_object_with_status(
            t_basic,
            team_a,
            Coord3D::default(),
            0.0,
            init_status,
        );
        let sp_b = find_sp_module(&sim, o_basic);
        let (sp_for_norm, sp_for_ready, sp_ready_init, sp_is_ready_init) = sim
            .dispatch_module(sp_b, |m, sim_a, ctx| {
                let s = m.as_special_power().unwrap();
                (
                    u32::from(s.is_module_for_power(sim_a, ctx, sp_normal_id) != FALSE),
                    u32::from(s.is_module_for_power(sim_a, ctx, sp_ready_id) != FALSE),
                    s.get_ready_frame(sim_a, ctx),
                    u32::from(s.is_ready(sim_a, ctx) != FALSE),
                )
            })
            .unwrap();

        sim.world.frame = frame0 + step_dt;
        let (pct_mid, pct_paused, ready_paused, still_paused) = sim
            .dispatch_module(sp_b, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                let mid = s.get_percent_ready(sim_a, ctx);
                s.pause_countdown(sim_a, ctx, TRUE);
                s.pause_countdown(sim_a, ctx, TRUE);
                let paused_pct = s.get_percent_ready(sim_a, ctx);
                let r_paused = s.get_ready_frame(sim_a, ctx);
                s.pause_countdown(sim_a, ctx, FALSE);
                let same = u32::from(s.get_percent_ready(sim_a, ctx) == paused_pct);
                (mid.to_bits(), paused_pct.to_bits(), r_paused, same)
            })
            .unwrap();

        sim.world.frame = frame0 + step_dt + 7;
        let (ready_unpaused, ready_after_set) = sim
            .dispatch_module(sp_b, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.pause_countdown(sim_a, ctx, FALSE);
                let r_unp = s.get_ready_frame(sim_a, ctx);
                s.set_ready_frame(sim_a, ctx, custom_ready);
                let r_set = s.get_ready_frame(sim_a, ctx);
                (r_unp, r_set)
            })
            .unwrap();

        let (sp_bytes, sp_crc, sp_xcrc) = sp_xfer_and_crc(&mut sim, sp_b);
        let ready_after_load = sim
            .dispatch_module(sp_b, |m, sim_a, ctx| {
                m.as_special_power_mut().unwrap().set_ready_frame(sim_a, ctx, 123456);
                let mut xl = XferLoad::new(&sp_bytes);
                m.xfer(&mut xl, sim_a).expect("sp xfer load");
                m.as_special_power().unwrap().get_ready_frame(sim_a, ctx)
            })
            .unwrap();

        let o_rdy = sim.spawn_object(t_ready, team_a, Coord3D::default(), 0.0);
        let sp_r = find_sp_module(&sim, o_rdy);
        let (rdy_starts, rdy_f0, rdy_is0) = sim
            .dispatch_module(sp_r, |m, sim_a, ctx| {
                let s = m.as_special_power().unwrap();
                (
                    u32::from(s.starts_ready(sim_a, ctx) != FALSE),
                    s.get_ready_frame(sim_a, ctx),
                    u32::from(s.is_ready(sim_a, ctx) != FALSE),
                )
            })
            .unwrap();

        let o_pau = sim.spawn_object(t_paused, team_a, Coord3D::default(), 0.0);
        let sp_p = find_sp_module(&sim, o_pau);
        let pct_pau100 = sim
            .dispatch_module(sp_p, |m, sim_a, ctx| {
                m.as_special_power().unwrap().get_percent_ready(sim_a, ctx).to_bits()
            })
            .unwrap();
        if disabled_flag {
            sim.set_disabled(o_pau, DisabledType::Emp);
        }
        let pau_ready_after_do = sim
            .dispatch_module(sp_p, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power(sim_a, ctx, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();

        let o_scr = sim.spawn_object(t_script, team_a, Coord3D::default(), 0.0);
        let sp_sc = find_sp_module(&sim, o_scr);
        let scr_t = sim
            .dispatch_module(sp_sc, |m, sim_a, ctx| {
                u32::from(m.as_special_power().unwrap().is_script_only(sim_a, ctx) != FALSE)
            })
            .unwrap();
        let scr_f = sim
            .dispatch_module(sp_b, |m, sim_a, ctx| {
                u32::from(m.as_special_power().unwrap().is_script_only(sim_a, ctx) != FALSE)
            })
            .unwrap();

        let o_shr = sim.spawn_object(t_shared, team_a, Coord3D::default(), 0.0);
        let sp_sh = find_sp_module(&sim, o_shr);
        let (shr_pre, shr_cr, shr_do) = sim
            .dispatch_module(sp_sh, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                let pre = s.get_ready_frame(sim_a, ctx);
                s.on_special_power_creation(sim_a, ctx);
                let cr = s.get_ready_frame(sim_a, ctx);
                s.do_special_power(sim_a, ctx, 0);
                let after = s.get_ready_frame(sim_a, ctx);
                (pre, cr, after)
            })
            .unwrap();

        assert_eq!(
            [
                sp_for_norm,
                sp_for_ready,
                sp_ready_init,
                sp_is_ready_init,
                pct_mid,
                pct_paused,
                ready_paused,
                still_paused,
                ready_unpaused,
                ready_after_set,
                ready_after_load,
            ],
            want_sp[0..11],
            "seed {seed} sp[0..11]"
        );
        #[cfg(not(feature = "retail-crc"))]
        assert_eq!(
            [rdy_starts, rdy_f0, rdy_is0, pct_pau100],
            want_sp[11..15],
            "seed {seed} sp[11..15]"
        );
        #[cfg(feature = "retail-crc")]
        {
            assert_eq!(rdy_starts, 0, "seed {seed} retail-crc starts_ready");
            assert_eq!(rdy_f0, frame0 + step_dt + 7 + 18, "seed {seed} retail-crc rdy_f0");
            assert_eq!(rdy_is0, 0, "seed {seed} retail-crc rdy_is0");
            assert_eq!(pct_pau100, 0.0f32.to_bits(), "seed {seed} retail-crc pct_pau100");
        }
        assert_eq!(
            [pau_ready_after_do, scr_t, scr_f, shr_pre, shr_cr, shr_do],
            want_sp[15..21],
            "seed {seed} sp[15..21]"
        );
        assert_eq!(sp_crc, want_sp_crc, "seed {seed} sp_crc");
        assert_eq!(sp_xcrc, want_sp_xcrc, "seed {seed} sp_xcrc");
        assert_eq!(sp_bytes, decode_hex_bytes(want_sp_hex).unwrap(), "seed {seed} sp_xfer");

        // --- 2) BaikonurLaunchPower ---
        let o_baik = sim.spawn_object(t_baik, team_a, Coord3D::default(), 0.0);
        let sp_baik = find_sp_module(&sim, o_baik);
        if disabled_flag {
            sim.set_disabled(o_baik, DisabledType::Emp);
        }
        let baik_loc = Coord3D::new(loc_x, loc_y, 0.0);
        let baik_ready = sim
            .dispatch_module(sp_baik, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power(sim_a, ctx, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        let baik_door = u32::from(
            sim.object(o_baik)
                .unwrap()
                .model_condition
                .test(ModelConditionFlagType::Door1Opening.as_i32())
                != FALSE,
        );
        let baik_cnt_pre = live_obj_count(&sim);
        let _ = sim.dispatch_module(sp_baik, |m, sim_a, ctx| {
            m.as_special_power_mut().unwrap().do_special_power_at_location(
                sim_a,
                ctx,
                &baik_loc,
                0.0,
                0,
            );
        });
        let baik_cnt_post = live_obj_count(&sim);
        let (baik_bytes, _, _) = sp_xfer_and_crc(&mut sim, sp_baik);
        assert_eq!(
            [baik_door, baik_ready, (baik_cnt_post - baik_cnt_pre) as u32],
            want_baik,
            "seed {seed} baik"
        );
        assert_eq!(
            baik_bytes,
            decode_hex_bytes(want_baik_hex).unwrap(),
            "seed {seed} baik_xfer"
        );

        // --- 3) CashBountyPower ---
        let cb_before = sim.player_bounty_percent(team_a).to_bits();
        let o_cb_low = sim.spawn_object(t_cb_low, team_a, Coord3D::default(), 0.0);
        let sp_cb_low = find_sp_module(&sim, o_cb_low);
        let cb_low = sim.player_bounty_percent(team_a).to_bits();
        let cb_sci_match = sim
            .dispatch_module(sp_cb_low, |m, sim_a, ctx| {
                u32::from(m.as_special_power().unwrap().get_required_science(sim_a, ctx) == bounty_req_sci)
            })
            .unwrap();
        let o_cb_high = sim.spawn_object(t_cb_high, team_a, Coord3D::default(), 0.0);
        let sp_cb_high = find_sp_module(&sim, o_cb_high);
        let cb_high = sim.player_bounty_percent(team_a).to_bits();
        if !has_bounty_sci {
            sim.grant_player_science(team_a, sci_bounty);
        }
        let (cb_r_pre, cb_r_post) = sim
            .dispatch_module(sp_cb_high, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.on_special_power_creation(sim_a, ctx);
                let pre = s.get_ready_frame(sim_a, ctx);
                s.do_special_power(sim_a, ctx, 0);
                let post = s.get_ready_frame(sim_a, ctx);
                (pre, post)
            })
            .unwrap();
        let cb_sp = sim.player_bounty_percent(team_a).to_bits();
        assert_eq!(
            [cb_before, cb_low, cb_high, cb_sp, cb_sci_match, cb_r_pre, cb_r_post],
            want_cb,
            "seed {seed} cb"
        );

        // --- 4) CashHackSpecialPower ---
        let o_ch = sim.spawn_object(t_ch, team_a, Coord3D::default(), 0.0);
        let o_ch_tgt = sim.spawn_object(
            t_tgt,
            team_b,
            Coord3D::new(loc_x + 20.0, loc_y, 0.0),
            0.0,
        );
        let sp_ch = find_sp_module(&sim, o_ch);
        if disabled_flag {
            sim.set_disabled(o_ch, DisabledType::Emp);
        }
        let ch_r_loc = sim
            .dispatch_module(sp_ch, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power_at_location(sim_a, ctx, &baik_loc, 0.0, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        let ch_r_obj = sim
            .dispatch_module(sp_ch, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power_at_object(sim_a, ctx, o_ch_tgt, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        let ch_ma = sim.player_money(team_a);
        let ch_mb = sim.player_money(team_b);
        assert_eq!(
            [ch_r_loc, ch_ma, ch_mb, ch_r_obj],
            want_ch,
            "seed {seed} ch"
        );

        // --- 5) CleanupAreaPower ---
        let o_cln = sim.spawn_object(t_cln, team_a, Coord3D::default(), 0.0);
        let sp_cln = find_sp_module(&sim, o_cln);
        if disabled_flag {
            sim.set_disabled(o_cln, DisabledType::Emp);
        }
        let cln_ready = sim
            .dispatch_module(sp_cln, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power_at_location(sim_a, ctx, &baik_loc, 0.0, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        assert_eq!(cln_ready, want_cln, "seed {seed} cln");

        // --- 6) DefectorSpecialPower ---
        let o_def = sim.spawn_object(t_def, team_a, Coord3D::default(), 0.0);
        let o_def_tgt = sim.spawn_object(
            t_tgt,
            team_b,
            Coord3D::new(loc_x, loc_y + 30.0, 0.0),
            0.0,
        );
        let sp_def = find_sp_module(&sim, o_def);
        if disabled_flag {
            sim.set_disabled(o_def, DisabledType::Emp);
        }
        let _ = sim.dispatch_module(sp_def, |m, sim_a, ctx| {
            m.as_special_power_mut().unwrap().do_special_power_at_location(
                sim_a,
                ctx,
                &baik_loc,
                0.0,
                0,
            );
        });
        let def_loc_t = u32::from(sim.object(o_def_tgt).unwrap().team_id == team_a);
        let def_ready = sim
            .dispatch_module(sp_def, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power_at_object(sim_a, ctx, o_def_tgt, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        let def_tgt_rec = sim.object(o_def_tgt).unwrap();
        let def_obj_t = u32::from(def_tgt_rec.team_id == team_a);
        let def_undet = u32::from(def_tgt_rec.is_undetected_defector() != FALSE);
        assert_eq!(
            [def_loc_t, def_obj_t, def_undet, def_ready],
            want_def,
            "seed {seed} def"
        );

        // --- 7) FireWeaponPower ---
        let o_fw = sim.spawn_object(t_fw, team_a, Coord3D::new(50.0, 50.0, 0.0), 0.0);
        let sp_fw = find_sp_module(&sim, o_fw);
        if disabled_flag {
            sim.set_disabled(o_fw, DisabledType::Emp);
        }
        let fw_ready = sim
            .dispatch_module(sp_fw, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power_at_location(sim_a, ctx, &baik_loc, 0.0, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        assert_eq!(fw_ready, want_fw, "seed {seed} fw");

        // --- 8) OCLSpecialPower ---
        let o_ocl = sim.spawn_object(
            if ocl_above { t_ocl_above } else { t_ocl_loc },
            team_a,
            Coord3D::new(10.0, 20.0, 0.0),
            0.0,
        );
        let sp_ocl = find_sp_module(&sim, o_ocl);
        let ocl_ref = sim
            .dispatch_module(sp_ocl, |m, sim_a, ctx| {
                m.as_special_power()
                    .unwrap()
                    .get_reference_thing_template_name(sim_a, ctx)
                    .unwrap_or_default()
            })
            .unwrap();
        assert_eq!(ocl_ref, want_ocl_ref, "seed {seed} ocl_ref");
        if ocl_reject {
            let _ = sim.dispatch_module(sp_ocl, |m, sim_a, ctx| {
                m.as_special_power_mut()
                    .unwrap()
                    .set_ready_frame(sim_a, ctx, u32::MAX);
            });
        }
        if disabled_flag {
            sim.set_disabled(o_ocl, DisabledType::Emp);
        }
        let spawned_before = sim.world.spawned_ocls.len();
        let ocl_ready = sim
            .dispatch_module(sp_ocl, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power_at_location(sim_a, ctx, &baik_loc, 0.0, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        let spawned_cnt = (sim.world.spawned_ocls.len() - spawned_before) as u32;
        assert_eq!(spawned_cnt, want_ocl[0], "seed {seed} ocl_cnt");
        assert_eq!(ocl_ready, want_ocl[2], "seed {seed} ocl_ready");
        if spawned_cnt > 0 {
            let last_ocl = sim.world.spawned_ocls.last().unwrap();
            let expected_ocl_name = if want_ocl_name == "WP52_OCLUpgrade" {
                "OCL_WP52_Upgrade"
            } else {
                "OCL_WP52_Default"
            };
            assert_eq!(last_ocl.0, expected_ocl_name, "seed {seed} ocl_name");
            assert_eq!(
                last_ocl.2.z,
                if ocl_above { 300.0 } else { 0.0 },
                "seed {seed} ocl creation z"
            );
        }

        // --- 9) SpyVisionSpecialPower ---
        let o_sv = sim.spawn_object(t_sv, team_a, Coord3D::default(), 0.0);
        let sp_sv = find_sp_module(&sim, o_sv);
        if disabled_flag {
            sim.set_disabled(o_sv, DisabledType::Emp);
        }
        let sv_ready = sim
            .dispatch_module(sp_sv, |m, sim_a, ctx| {
                let s = m.as_special_power_mut().unwrap();
                s.do_special_power(sim_a, ctx, 0);
                s.get_ready_frame(sim_a, ctx)
            })
            .unwrap();
        assert_eq!(sv_ready, want_sv, "seed {seed} sv");

        if seed == 0 {
            let s0_line = WP52_SP_JSON
                .lines()
                .find(|l| l.contains("\"seed0_diagnostics\":"))
                .expect("seed0_diagnostics line");
            assert!(s0_line.contains(&format!("\"ref\":\"{ocl_ref}\"")));
            assert!(s0_line.contains(&format!("\"ma\":{ch_ma}")));
            assert!(s0_line.contains(&format!("\"mb\":{ch_mb}")));
            let _ = get_i32_vec(line, "ch"); // keep helper check active
        }

        // Cleanup all live objects before next seed
        let to_kill: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
        for s in to_kill {
            sim.destroy_object_now(s);
        }
        sim.process_destroy_list();
    }
}
