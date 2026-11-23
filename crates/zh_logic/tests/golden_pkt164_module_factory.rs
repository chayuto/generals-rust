//! Gate V2 Golden Vector test suite for `WP-5.0` Base `Module` + `ModuleFactory` + Trap 4 + `SimContext`
//! (`PKT-163` `Module.h`/`Module.cpp`/`BehaviorModule.h` and `PKT-164` `ModuleFactory.h`/`ModuleFactory.cpp`).

use std::collections::BTreeSet;
use std::sync::atomic::{AtomicBool, AtomicU32, AtomicUsize, Ordering};
use std::sync::Arc;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    encode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::ini::{IniError, IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Int, NameKeyType, Real, TeamId, UpdateSleepTime, FALSE};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{
    DeferredSelfOp, FacetMask, ModuleContext, ModuleDataId, ModuleSidecar, ObjectSlot, TakeError,
};
use zh_logic::combat::armor::ArmorStore;
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::{LocomotorStore, LocomotorTemplate};
use zh_logic::combat::locomotor_set::LocomotorSetType;
use zh_logic::gamelogic::global_data::BodyDamageType;
use zh_logic::modules::{
    BehaviorModule, BodyFacet, DestroyFacet, ModuleDataStore, ModuleRegistry, ModuleSnapshotHeader,
    ModuleType, StubModuleData, UpdateFacet, BUILTIN_ZH_MODULES, W3D_DRAW_STUB_NAMES,
};
use zh_logic::object::thing_factory::ThingFactory;
use zh_logic::object::thing_template::{
    DefaultModuleMaskLookup, GameTextLookup, ThingTemplateParseContext,
};
use zh_logic::object::weapon::WeaponStore;
use zh_logic::rules::science::ScienceStore;
use zh_logic::scheduler::{SleepyUpdatePhase, UpdateModuleScheduleState};
use zh_logic::sim::{SimApi, SimContext, SimView};

const PKT_164_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-164.json");

/// Exact 205 candidate module names used by `tests/cpp_harness/gen_pkt164_module_factory.cpp`
/// to populate `seed_0_catalog` (including `#ifdef`-disabled and `W3DModelDraw` entries that
/// advance C++ `NameKeyGenerator::m_nextID` via `findModuleTemplate` prior to seed `0..999`).
const CANDIDATE_MODULE_NAMES: [&str; 205] = [
    "AIUpdateInterface",
    "ActiveBody",
    "ActiveShroudUpgrade",
    "AnimatedParticleSysBoneClientUpdate",
    "AnimationSteeringUpdate",
    "ArmorUpgrade",
    "AssaultTransportAIUpdate",
    "AssistedTargetingUpdate",
    "AutoDepositUpdate",
    "AutoFindHealingUpdate",
    "AutoHealBehavior",
    "BaikonurLaunchPower",
    "BaseRegenerateUpdate",
    "BattleBusSlowDeathBehavior",
    "BattlePlanUpdate",
    "BeaconClientUpdate",
    "BoneFXDamage",
    "BoneFXUpdate",
    "BridgeBehavior",
    "BridgeScaffoldBehavior",
    "BridgeTowerBehavior",
    "BunkerBusterBehavior",
    "CashBountyPower",
    "CashHackSpecialPower",
    "CaveContain",
    "CheckpointUpdate",
    "ChinookAIUpdate",
    "CleanupAreaPower",
    "CleanupHazardUpdate",
    "CommandButtonHuntUpdate",
    "CommandSetUpgrade",
    "ConvertToCarBombCrateCollide",
    "ConvertToHijackedVehicleCrateCollide",
    "CostModifierUpgrade",
    "CountermeasuresBehavior",
    "CreateCrateDie",
    "CreateObjectDie",
    "CrushDie",
    "DamDie",
    "DefaultProductionExitUpdate",
    "DefectorSpecialPower",
    "DeletionUpdate",
    "DeliverPayloadAIUpdate",
    "DemoTrapUpdate",
    "DemoralizeSpecialPower",
    "DeployStyleAIUpdate",
    "DestroyDie",
    "DozerAIUpdate",
    "DumbProjectileBehavior",
    "DynamicGeometryInfoUpdate",
    "DynamicShroudClearingRangeUpdate",
    "EMPUpdate",
    "EjectPilotDie",
    "EnemyNearUpdate",
    "ExperienceScalarUpgrade",
    "FXListDie",
    "FireOCLAfterWeaponCooldownUpdate",
    "FireSpreadUpdate",
    "FireWeaponCollide",
    "FireWeaponPower",
    "FireWeaponUpdate",
    "FireWeaponWhenDamagedBehavior",
    "FireWeaponWhenDeadBehavior",
    "FirestormDynamicGeometryInfoUpdate",
    "FlammableUpdate",
    "FlightDeckBehavior",
    "FloatUpdate",
    "GarrisonContain",
    "GenerateMinefieldBehavior",
    "GrantScienceUpgrade",
    "GrantStealthBehavior",
    "GrantUpgradeCreate",
    "HackInternetAIUpdate",
    "HealContain",
    "HealCrateCollide",
    "HeightDieUpdate",
    "HelicopterSlowDeathBehavior",
    "HelixContain",
    "HighlanderBody",
    "HijackerUpdate",
    "HiveStructureBody",
    "HordeUpdate",
    "ImmortalBody",
    "InactiveBody",
    "InstantDeathBehavior",
    "InternetHackContain",
    "JetAIUpdate",
    "JetSlowDeathBehavior",
    "KeepObjectDie",
    "LaserUpdate",
    "LeafletDropBehavior",
    "LifetimeUpdate",
    "LockWeaponCreate",
    "LocomotorSetUpgrade",
    "MaxHealthUpgrade",
    "MinefieldBehavior",
    "MissileAIUpdate",
    "MissileLauncherBuildingUpdate",
    "MobMemberSlavedUpdate",
    "MobNexusContain",
    "ModelConditionUpgrade",
    "MoneyCrateCollide",
    "NeutronBlastBehavior",
    "NeutronMissileSlowDeathBehavior",
    "NeutronMissileUpdate",
    "OCLSpecialPower",
    "OCLUpdate",
    "ObjectCreationUpgrade",
    "OpenContain",
    "OverchargeBehavior",
    "OverlordContain",
    "POWTruckAIUpdate",
    "POWTruckBehavior",
    "ParachuteContain",
    "ParkingPlaceBehavior",
    "ParticleUplinkCannonUpdate",
    "PassengersFireUpgrade",
    "PhysicsBehavior",
    "PilotFindVehicleUpdate",
    "PointDefenseLaserUpdate",
    "PoisonedBehavior",
    "PowerPlantUpdate",
    "PowerPlantUpgrade",
    "PreorderCreate",
    "PrisonBehavior",
    "PrisonDockUpdate",
    "ProductionUpdate",
    "ProjectileStreamUpdate",
    "ProneUpdate",
    "PropagandaCenterBehavior",
    "PropagandaTowerBehavior",
    "QueueProductionExitUpdate",
    "RadarUpdate",
    "RadarUpgrade",
    "RadiusDecalUpdate",
    "RailedTransportAIUpdate",
    "RailedTransportContain",
    "RailedTransportDockUpdate",
    "RailroadBehavior",
    "RebuildHoleBehavior",
    "RebuildHoleExposeDie",
    "RepairDockUpdate",
    "ReplaceObjectUpgrade",
    "RiderChangeContain",
    "SabotageCommandCenterCrateCollide",
    "SabotageFakeBuildingCrateCollide",
    "SabotageInternetCenterCrateCollide",
    "SabotageMilitaryFactoryCrateCollide",
    "SabotagePowerPlantCrateCollide",
    "SabotageSuperweaponCrateCollide",
    "SabotageSupplyCenterCrateCollide",
    "SabotageSupplyDropzoneCrateCollide",
    "SalvageCrateCollide",
    "ShroudCrateCollide",
    "SlavedUpdate",
    "SlowDeathBehavior",
    "SmartBombTargetHomingUpdate",
    "SpawnBehavior",
    "SpawnPointProductionExitUpdate",
    "SpecialAbility",
    "SpecialAbilityUpdate",
    "SpecialPowerCompletionDie",
    "SpecialPowerCreate",
    "SpectreGunshipDeploymentUpdate",
    "SpectreGunshipUpdate",
    "SpyVisionSpecialPower",
    "SpyVisionUpdate",
    "SquishCollide",
    "StatusBitsUpgrade",
    "StealthDetectorUpdate",
    "StealthUpdate",
    "StealthUpgrade",
    "StickyBombUpdate",
    "StructureBody",
    "StructureCollapseUpdate",
    "StructureToppleUpdate",
    "SubObjectsUpgrade",
    "SupplyCenterCreate",
    "SupplyCenterDockUpdate",
    "SupplyCenterProductionExitUpdate",
    "SupplyTruckAIUpdate",
    "SupplyWarehouseCreate",
    "SupplyWarehouseCripplingBehavior",
    "SupplyWarehouseDockUpdate",
    "SwayClientUpdate",
    "TechBuildingBehavior",
    "TensileFormationUpdate",
    "ToppleUpdate",
    "TransitionDamageFX",
    "TransportAIUpdate",
    "TransportContain",
    "TunnelContain",
    "UndeadBody",
    "UnitCrateCollide",
    "UnpauseSpecialPowerUpgrade",
    "UpgradeDie",
    "VeterancyCrateCollide",
    "VeterancyGainCreate",
    "W3DModelDraw",
    "WanderAIUpdate",
    "WaveGuideUpdate",
    "WeaponBonusUpdate",
    "WeaponBonusUpgrade",
    "WeaponSetUpgrade",
    "WorkerAIUpdate",
];

fn module_type_from_i32(raw: Int) -> ModuleType {
    match raw {
        0 => ModuleType::Behavior,
        1 => ModuleType::Draw,
        2 => ModuleType::ClientUpdate,
        _ => panic!("unexpected module_type {raw}"),
    }
}

fn parse_i32_dec(token: &str) -> Option<i32> {
    let bytes = token.trim().as_bytes();
    if bytes.is_empty() {
        return None;
    }
    let (neg, digits) = if bytes[0] == b'-' {
        (true, &bytes[1..])
    } else {
        (false, bytes)
    };
    if digits.is_empty() {
        return None;
    }
    let mut acc: i64 = 0;
    for &b in digits {
        if !b.is_ascii_digit() {
            return None;
        }
        acc = acc.checked_mul(10)?.checked_add((b - b'0') as i64)?;
    }
    if neg {
        acc = -acc;
    }
    i32::try_from(acc).ok()
}

fn parse_queries_field(line: &str) -> Vec<(String, Int)> {
    let needle = "\"queries\":[";
    let start = line.find(needle).expect("queries array missing") + needle.len();
    let rest = &line[start..];
    let mut out = Vec::new();
    let bytes = rest.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b']' => break,
            b'[' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                assert!(i < bytes.len(), "malformed query string start");
                i += 1;
                let name_start = i;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                assert!(i < bytes.len(), "malformed query string end");
                let name = &rest[name_start..i];
                i += 1;
                while i < bytes.len() && (bytes[i] == b',' || bytes[i] == b' ') {
                    i += 1;
                }
                let num_start = i;
                while i < bytes.len() && bytes[i] != b']' {
                    i += 1;
                }
                let mtype = parse_i32_dec(&rest[num_start..i]).expect("valid module_type");
                out.push((name.to_owned(), mtype));
                i += 1;
            }
            _ => i += 1,
        }
    }
    out
}

#[inline]
fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[test]
fn test_pkt164_oracle_integrity_and_catalog() {
    assert!(PKT_164_JSON.contains("\"packet_id\": \"PKT-164\""));
    assert!(PKT_164_JSON.contains("\"vector_count\": 1000"));
    assert!(PKT_164_JSON.contains("\"source_sha256\""));
    assert_eq!(BUILTIN_ZH_MODULES.len(), 198);
    assert_eq!(
        get_i32(PKT_164_JSON, "registered_count"),
        Some(BUILTIN_ZH_MODULES.len() as Int)
    );

    // Module::xfer / Module::crc + header helpers
    let mut mod_save = XferSave::new();
    ModuleSnapshotHeader::xfer_module_header(&mut mod_save).unwrap();
    let mod_bytes = mod_save.into_bytes();
    assert_eq!(
        encode_hex_bytes(&mod_bytes),
        get_str(PKT_164_JSON, "module_xfer_hex").unwrap()
    );
    let mut mod_load = XferLoad::new(&mod_bytes);
    ModuleSnapshotHeader::xfer_module_header(&mut mod_load).unwrap();
    let mod_crc = XferCrc::new();
    assert_eq!(mod_crc.get_crc(), get_u32(PKT_164_JSON, "module_crc").unwrap());

    // Derived snapshot header round-trips (ObjectModule -> BehaviorModule -> UpdateModule / BodyModule)
    let mut obj_save = XferSave::new();
    ModuleSnapshotHeader::xfer_object_module_header(&mut obj_save).unwrap();
    let obj_bytes = obj_save.into_bytes();
    let mut obj_load = XferLoad::new(&obj_bytes);
    ModuleSnapshotHeader::xfer_object_module_header(&mut obj_load).unwrap();

    let mut beh_save = XferSave::new();
    ModuleSnapshotHeader::xfer_behavior_module_header(&mut beh_save).unwrap();
    let beh_bytes = beh_save.into_bytes();
    let mut beh_load = XferLoad::new(&beh_bytes);
    ModuleSnapshotHeader::xfer_behavior_module_header(&mut beh_load).unwrap();

    let mut upd_sched = UpdateModuleScheduleState::new(SleepyUpdatePhase::Normal);
    upd_sched.set_next_call_frame(42);
    let mut upd_save = XferSave::new();
    ModuleSnapshotHeader::xfer_update_module_header(&mut upd_save, &mut upd_sched).unwrap();
    let upd_bytes = upd_save.into_bytes();
    let mut upd_sched_loaded = UpdateModuleScheduleState::default();
    let mut upd_load = XferLoad::new(&upd_bytes);
    ModuleSnapshotHeader::xfer_update_module_header(&mut upd_load, &mut upd_sched_loaded).unwrap();
    assert_eq!(upd_sched_loaded, upd_sched);

    let mut unsched = UpdateModuleScheduleState::new(SleepyUpdatePhase::Normal);
    let mut unsched_save = XferSave::new();
    ModuleSnapshotHeader::xfer_update_module_header(&mut unsched_save, &mut unsched).unwrap();
    let unsched_bytes = unsched_save.into_bytes();
    let mut unsched_loaded = UpdateModuleScheduleState::default();
    let mut unsched_load = XferLoad::new(&unsched_bytes);
    ModuleSnapshotHeader::xfer_update_module_header(&mut unsched_load, &mut unsched_loaded).unwrap();
    assert_eq!(unsched_loaded.next_call_frame(), 0);
    assert_eq!(unsched_loaded.priority(), SleepyUpdatePhase::Normal as u32);

    let mut dmg_scalar: Real = 1.25;
    let mut body_save = XferSave::new();
    ModuleSnapshotHeader::xfer_body_module_header(&mut body_save, &mut dmg_scalar).unwrap();
    let body_bytes = body_save.into_bytes();
    let mut dmg_scalar_loaded: Real = 0.0;
    let mut body_load = XferLoad::new(&body_bytes);
    ModuleSnapshotHeader::xfer_body_module_header(&mut body_load, &mut dmg_scalar_loaded).unwrap();
    assert_eq!(dmg_scalar_loaded.to_bits(), dmg_scalar.to_bits());

    // ModuleFactory::xfer / ModuleFactory::crc
    let mut nk = NameKeyGenerator::new();
    nk.init();
    let mut reg = ModuleRegistry::with_builtin_zh_modules(&mut nk);
    assert_eq!(reg.len(), 198);

    let mut mf_save = XferSave::new();
    mf_save.xfer_snapshot(&mut reg).unwrap();
    let mf_bytes = mf_save.into_bytes();
    assert_eq!(
        encode_hex_bytes(&mf_bytes),
        get_str(PKT_164_JSON, "factory_xfer_hex").unwrap()
    );
    let mut mf_load = XferLoad::new(&mf_bytes);
    mf_load.xfer_snapshot(&mut reg).unwrap();
    let mut mf_crc = XferCrc::new();
    Snapshot::crc(&reg, &mut mf_crc);
    mf_crc.xfer_snapshot(&mut reg).unwrap();
    assert_eq!(
        mf_crc.get_crc(),
        get_u32(PKT_164_JSON, "factory_crc").unwrap()
    );

    // Exhaustively verify all 198 C++ seed_0_catalog entries.
    let mut catalog_count = 0usize;
    for line in PKT_164_JSON.lines() {
        if line.contains("\"decorated_key\":") && !line.contains("\"seed\":") {
            catalog_count += 1;
            let name = get_str(line, "name").expect("catalog name");
            let mtype_raw = get_i32(line, "module_type").expect("catalog module_type");
            let mtype = module_type_from_i32(mtype_raw);
            let want_mask = get_i32(line, "interface_mask").expect("catalog interface_mask");
            let want_key = get_i32(line, "decorated_key").expect("catalog decorated_key");

            let desc = reg
                .find_descriptor(&name, mtype)
                .unwrap_or_else(|| panic!("missing builtin module {name} ({mtype_raw})"));
            assert_eq!(desc.decorated_key.0, want_key, "key mismatch for {name}");
            assert_eq!(
                reg.find_module_interface_mask(&name, mtype),
                want_mask,
                "mask mismatch for {name}"
            );
            let by_key = reg
                .find_by_decorated_key(NameKeyType(want_key))
                .expect("lookup by decorated key");
            assert_eq!(by_key.name, name);
        }
    }
    assert_eq!(catalog_count, 198);

    // Subtle C++ special cases
    let laser = reg
        .find_descriptor("LaserUpdate", ModuleType::ClientUpdate)
        .expect("LaserUpdate is ClientUpdate");
    assert_eq!(laser.facet_mask.to_cpp_interface_mask(), 0x800);
    assert!(reg
        .find_descriptor("LaserUpdate", ModuleType::Behavior)
        .is_none());

    let flight_deck = reg
        .find_descriptor("FlightDeckBehavior", ModuleType::Behavior)
        .expect("FlightDeckBehavior");
    assert!(flight_deck.is_ai_module);
    assert_eq!(flight_deck.update_phase, SleepyUpdatePhase::Initial);
    assert!(flight_deck.facet_mask.contains(FacetMask::AI_UPDATE));

    reg.register_w3d_draw_stubs(&mut nk);
    for draw_name in W3D_DRAW_STUB_NAMES {
        let d = reg
            .find_descriptor(draw_name, ModuleType::Draw)
            .unwrap_or_else(|| panic!("missing W3D draw stub {draw_name}"));
        assert_eq!(d.facet_mask, FacetMask::DRAW);
        assert_eq!(reg.find_module_interface_mask(draw_name, ModuleType::Draw), 0x400);
    }
}

#[test]
fn test_pkt164_1000_golden_vectors() {
    let mut nk = NameKeyGenerator::new();
    nk.init();
    let reg = ModuleRegistry::with_builtin_zh_modules(&mut nk);

    // Replay the exact C++ generator NameKeyGenerator warm-up prior to Seed 0
    // (205 candidate module names x 3 ModuleTypes, then ModuleTag_01 / ModuleTag_02).
    let types = [
        ModuleType::Behavior,
        ModuleType::Draw,
        ModuleType::ClientUpdate,
    ];
    for name in CANDIDATE_MODULE_NAMES {
        for mt in types {
            let _ = ModuleRegistry::make_decorated_name_key(&mut nk, name, mt);
        }
    }
    let _ = nk.name_to_key("ModuleTag_01");
    let _ = nk.name_to_key("ModuleTag_02");

    let lines = extract_vector_lines(PKT_164_JSON);
    assert_eq!(lines.len(), 1000);

    for (seed, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(seed as u32));
        let queries = parse_queries_field(line);
        let want_masks = get_i32_vec(line, "masks").expect("masks array");
        let want_keys = get_i32_vec(line, "keys").expect("keys array");
        let want_crc = get_u32(line, "query_crc").expect("query_crc");

        assert_eq!(queries.len(), 8);
        assert_eq!(want_masks.len(), 8);
        assert_eq!(want_keys.len(), 8);

        let mut crc: u32 = 0;
        for (q_idx, (qname, mtype_raw)) in queries.iter().enumerate() {
            let mt = module_type_from_i32(*mtype_raw);
            let mask = reg.find_module_interface_mask(qname, mt);
            let dkey = ModuleRegistry::make_decorated_name_key(&mut nk, qname, mt).0;
            assert_eq!(
                mask, want_masks[q_idx],
                "seed {seed} query {q_idx} ({qname}, {mtype_raw}) mask"
            );
            assert_eq!(
                dkey, want_keys[q_idx],
                "seed {seed} query {q_idx} ({qname}, {mtype_raw}) decorated_key"
            );
            crc = rot_crc(crc, mask as u32);
            crc = rot_crc(crc, dkey as u32);
        }
        assert_eq!(crc, want_crc, "seed {seed} query_crc");
    }
}

struct StubGameText;

impl GameTextLookup for StubGameText {
    fn fetch_utf16(&mut self, label: &str) -> Vec<u16> {
        label.bytes().map(|b| b as u16).collect()
    }
}

#[test]
fn test_module_data_store_and_thing_template_trap4_ini() {
    let mut nk = NameKeyGenerator::new();
    nk.init();

    let mut reg = ModuleRegistry::with_builtin_zh_modules(&mut nk);
    reg.register_w3d_draw_stubs(&mut nk);

    let mut loco_store = LocomotorStore::new();
    let mut loco = LocomotorTemplate::new();
    loco.parse_ini_block(
        "Locomotor BasicTankLoco\n  Surfaces = GROUND\n  Speed = 30.0\n  Appearance = TREADS\nEnd\n",
    )
    .expect("parse BasicTankLoco");
    loco_store.add_template(loco);

    let mut data_store = ModuleDataStore::new();
    let armor_store = ArmorStore::new();
    let weapon_store = WeaponStore::new();
    let mut science_store = ScienceStore::new();
    science_store.init();
    let fx_names = BTreeSet::new();
    let damage_fx_names = BTreeSet::new();
    let mut game_text = StubGameText;
    let default_masks = DefaultModuleMaskLookup;
    let mut factory = ThingFactory::new();

    let base_ini = "\
Object TestTank
  Locomotor = SET_NORMAL BasicTankLoco
  Draw = W3DModelDraw ModuleTag_Draw
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 500.0
  End
  Behavior = DestroyDie ModuleTag_Die
  End
  ClientUpdate = LaserUpdate ModuleTag_Laser
  End
End
";
    {
        let mut reader = IniReader::new("base.ini", base_ini.as_bytes(), IniLoadType::Overwrite);
        let mut ctx = ThingTemplateParseContext {
            name_keys: &mut nk,
            armor_store: &armor_store,
            weapon_store: &weapon_store,
            science_store: &science_store,
            fx_list_names: &fx_names,
            damage_fx_names: &damage_fx_names,
            game_text: &mut game_text,
            module_masks: &default_masks,
            default_occlusion_delay: 0,
            locomotor_store: Some(&loco_store),
            module_registry: Some(&reg),
            module_data_store: Some(&mut data_store),
        };
        factory.load_from_ini(&mut reader, &mut ctx).expect("parse TestTank");
    }

    let tmpl_id = factory.find_template_id("TestTank").expect("TestTank id");
    let (body_tag_key, body_data_id) = {
        let tt = factory.find_template("TestTank").expect("TestTank template");
        assert_eq!(tt.get_locomotor_set(LocomotorSetType::Normal), &["BasicTankLoco"]);

        let body_nugget = tt
            .get_behavior_module_info()
            .get_nugget_with_tag("ModuleTag_Body")
            .expect("ModuleTag_Body nugget");
        assert_ne!(body_nugget.tag_key, NameKeyType::INVALID);
        assert_ne!(body_nugget.data_id, ModuleDataId::INVALID);
        assert_eq!(
            data_store.find_for_template(tmpl_id, body_nugget.tag_key),
            Some(body_nugget.data_id)
        );
        let stored = data_store.get(body_nugget.data_id).expect("stored ModuleData");
        assert!(stored.as_any().downcast_ref::<StubModuleData>().is_some());
        assert!(stored.facet_mask().contains(FacetMask::BODY));

        assert!(tt
            .get_draw_module_info()
            .get_nugget_with_tag("ModuleTag_Draw")
            .is_some());
        assert!(tt
            .get_behavior_module_info()
            .get_nugget_with_tag("ModuleTag_Die")
            .is_some());
        assert!(tt
            .get_client_update_module_info()
            .get_nugget_with_tag("ModuleTag_Laser")
            .is_some());

        (body_nugget.tag_key, body_nugget.data_id)
    };

    // Duplicate Locomotor set rejected in Overwrite mode
    let dup_ini = "\
Object DupLocoTank
  Locomotor = SET_NORMAL BasicTankLoco
  Locomotor = SET_NORMAL BasicTankLoco
End
";
    {
        let mut reader = IniReader::new("dup.ini", dup_ini.as_bytes(), IniLoadType::Overwrite);
        let mut ctx = ThingTemplateParseContext {
            name_keys: &mut nk,
            armor_store: &armor_store,
            weapon_store: &weapon_store,
            science_store: &science_store,
            fx_list_names: &fx_names,
            damage_fx_names: &damage_fx_names,
            game_text: &mut game_text,
            module_masks: &default_masks,
            default_occlusion_delay: 0,
            locomotor_store: Some(&loco_store),
            module_registry: Some(&reg),
            module_data_store: Some(&mut data_store),
        };
        assert_eq!(factory.load_from_ini(&mut reader, &mut ctx), Err(IniError::InvalidData));
    }

    // Unknown locomotor rejected
    let bad_loco_ini = "\
Object BadLocoTank
  Locomotor = SET_NORMAL MissingLoco
End
";
    {
        let mut reader = IniReader::new("bad_loco.ini", bad_loco_ini.as_bytes(), IniLoadType::Overwrite);
        let mut ctx = ThingTemplateParseContext {
            name_keys: &mut nk,
            armor_store: &armor_store,
            weapon_store: &weapon_store,
            science_store: &science_store,
            fx_list_names: &fx_names,
            damage_fx_names: &damage_fx_names,
            game_text: &mut game_text,
            module_masks: &default_masks,
            default_occlusion_delay: 0,
            locomotor_store: Some(&loco_store),
            module_registry: Some(&reg),
            module_data_store: Some(&mut data_store),
        };
        assert_eq!(factory.load_from_ini(&mut reader, &mut ctx), Err(IniError::InvalidData));
    }

    // Bare module without AddModule rejected in CreateOverrides mode
    let bare_override_ini = "\
Object TestTank
  Behavior = InstantDeathBehavior ModuleTag_BareOvr
  End
End
";
    {
        let mut reader = IniReader::new(
            "bare_ovr.ini",
            bare_override_ini.as_bytes(),
            IniLoadType::CreateOverrides,
        );
        let mut ctx = ThingTemplateParseContext {
            name_keys: &mut nk,
            armor_store: &armor_store,
            weapon_store: &weapon_store,
            science_store: &science_store,
            fx_list_names: &fx_names,
            damage_fx_names: &damage_fx_names,
            game_text: &mut game_text,
            module_masks: &default_masks,
            default_occlusion_delay: 0,
            locomotor_store: Some(&loco_store),
            module_registry: Some(&reg),
            module_data_store: Some(&mut data_store),
        };
        assert_eq!(factory.load_from_ini(&mut reader, &mut ctx), Err(IniError::InvalidData));
    }

    // mark_base_loaded + CreateOverrides (Locomotor + AddModule) + reset cleanup
    factory.reset();
    data_store.mark_base_loaded();
    let base_len = data_store.len();
    let map_ini = "\
Object TestTank
  Locomotor = SET_NORMAL BasicTankLoco
  AddModule
    Behavior = InstantDeathBehavior ModuleTag_MapDie
    End
  End
End
";
    {
        let mut reader = IniReader::new("map.ini", map_ini.as_bytes(), IniLoadType::CreateOverrides);
        let mut ctx = ThingTemplateParseContext {
            name_keys: &mut nk,
            armor_store: &armor_store,
            weapon_store: &weapon_store,
            science_store: &science_store,
            fx_list_names: &fx_names,
            damage_fx_names: &damage_fx_names,
            game_text: &mut game_text,
            module_masks: &default_masks,
            default_occlusion_delay: 0,
            locomotor_store: Some(&loco_store),
            module_registry: Some(&reg),
            module_data_store: Some(&mut data_store),
        };
        factory
            .load_from_ini(&mut reader, &mut ctx)
            .expect("parse TestTank override");
    }
    let map_die_tag = nk.name_to_key("ModuleTag_MapDie");
    assert!(data_store.len() > base_len);
    assert!(data_store.find_for_template(tmpl_id, map_die_tag).is_some());

    data_store.reset();
    factory.reset();
    assert_eq!(data_store.len(), base_len);
    assert!(data_store.find_for_template(tmpl_id, map_die_tag).is_none());
    assert_eq!(
        data_store.find_for_template(tmpl_id, body_tag_key),
        Some(body_data_id)
    );
}

struct TestBodyModule {
    mod_name_key: NameKeyType,
    health: Real,
    max_health: Real,
    prev_health: Real,
    reentered_ok: Arc<AtomicBool>,
    sidecar_seen_health_bits: Arc<AtomicU32>,
    deferred_applied: Arc<AtomicUsize>,
}

impl BehaviorModule for TestBodyModule {
    fn module_name_key(&self) -> NameKeyType {
        self.mod_name_key
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        if let DeferredSelfOp::ChangeHealth { delta, .. } = op {
            self.prev_health = self.health;
            self.health += delta;
            if self.health < 0.0 {
                self.health = 0.0;
            }
            self.deferred_applied.fetch_add(1, Ordering::Relaxed);
        }
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        ModuleSidecar::Body {
            health: self.health,
            max_health: self.max_health,
            prev_health: self.prev_health,
            damage_scalar: 1.0,
            damage_state: if self.health <= 0.0 {
                BodyDamageType::Rubble.as_i32() as u8
            } else {
                BodyDamageType::Pristine.as_i32() as u8
            },
            front_crushed: FALSE,
            back_crushed: FALSE,
            indestructible: FALSE,
        }
    }

    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl UpdateFacet for TestBodyModule {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        // Trigger same-slot re-entrant attempt_damage while TestBodyModule is taken out.
        let mut inner_dmg = DamageInfo::default();
        inner_dmg.input.amount = 15.0;
        sim.attempt_damage(ctx.owner_slot, &mut inner_dmg);

        if let ModuleSidecar::Body { health, .. } = sim.module_sidecar(ctx.mod_slot) {
            self.sidecar_seen_health_bits
                .store(health.to_bits(), Ordering::Relaxed);
            if (health - 60.0).abs() < 1e-4 {
                self.reentered_ok.store(true, Ordering::Relaxed);
            }
        }
        UpdateSleepTime::FOREVER
    }
}

impl BodyFacet for TestBodyModule {
    fn attempt_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, dmg: &mut DamageInfo) {
        self.prev_health = self.health;
        self.health = (self.health - dmg.input.amount).max(0.0);
    }

    fn get_health(&self) -> Real {
        self.health
    }

    fn get_max_health(&self) -> Real {
        self.max_health
    }

    fn get_previous_health(&self) -> Real {
        self.prev_health
    }
}

struct CascadeDeleteModule {
    mod_name_key: NameKeyType,
    child_to_destroy: Option<ObjectSlot>,
    destroy_observed: Arc<AtomicUsize>,
    delete_observed: Arc<AtomicUsize>,
}

impl BehaviorModule for CascadeDeleteModule {
    fn module_name_key(&self) -> NameKeyType {
        self.mod_name_key
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.delete_observed.fetch_add(1, Ordering::Relaxed);
        if let Some(child) = self.child_to_destroy {
            sim.destroy_object(child);
        }
    }

    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }

    fn as_destroy_mut(&mut self) -> Option<&mut dyn DestroyFacet> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
}

impl DestroyFacet for CascadeDeleteModule {
    fn on_destroy(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.destroy_observed.fetch_add(1, Ordering::Relaxed);
    }
}

impl UpdateFacet for CascadeDeleteModule {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

#[test]
fn test_sim_context_reentrancy_sidecar_and_cascading_destroy() {
    let mut sim = SimContext::new();
    let tag_body = sim.rules.name_keys.name_to_key("TestBodyTag");
    let tag_parent = sim.rules.name_keys.name_to_key("TestParentTag");
    let tag_child = sim.rules.name_keys.name_to_key("TestChildTag");
    let mod_key = sim.rules.name_keys.name_to_key("TestModule");

    let tank = <SimContext as SimApi>::spawn_object(
        &mut sim,
        Default::default(),
        TeamId(1),
        Coord3D::default(),
        0.0,
    );

    let reentered_ok = Arc::new(AtomicBool::new(false));
    let sidecar_seen = Arc::new(AtomicU32::new(0));
    let deferred_applied = Arc::new(AtomicUsize::new(0));
    let body_slot = sim.world.attach_module(
        tank,
        tag_body,
        ModuleDataId::INVALID,
        FacetMask(FacetMask::BODY.0 | FacetMask::UPDATE.0),
        SleepyUpdatePhase::Normal,
        Box::new(TestBodyModule {
            mod_name_key: mod_key,
            health: 100.0,
            max_health: 100.0,
            prev_health: 100.0,
            reentered_ok: Arc::clone(&reentered_ok),
            sidecar_seen_health_bits: Arc::clone(&sidecar_seen),
            deferred_applied: Arc::clone(&deferred_applied),
        }),
    );

    // Non-reentrant attempt_damage updates health to 75 and refreshes sidecar.
    let mut dmg = DamageInfo::default();
    dmg.input.amount = 25.0;
    sim.attempt_damage_on(tank, &mut dmg);
    match sim.world.modules.sidecar(body_slot) {
        ModuleSidecar::Body { health, .. } => assert!((health - 75.0).abs() < 1e-4),
        other => panic!("unexpected sidecar: {other:?}"),
    }

    // Re-entrant TakeError check + sidecar visibility + DeferredSelfOp flush during sleepy update.
    sim.step_sleepy_updates();
    assert!(reentered_ok.load(Ordering::Relaxed));
    assert_eq!(deferred_applied.load(Ordering::Relaxed), 1);
    assert_eq!(
        f32::from_bits(sidecar_seen.load(Ordering::Relaxed)),
        60.0
    );
    let final_health = sim
        .world
        .modules
        .payload(body_slot)
        .and_then(|m| m.as_body())
        .expect("restored body payload")
        .get_health();
    assert!((final_health - 60.0).abs() < 1e-4);

    // Explicit same-slot dispatch error check when module payload is temporarily taken.
    let (taken, _, gen_id) = sim.world.modules.take_payload(body_slot).unwrap();
    assert_eq!(
        sim.dispatch_module(body_slot, |_, _, _| ()).unwrap_err(),
        TakeError::Reentered
    );
    sim.world.modules.restore_payload(body_slot, gen_id, taken);

    // Destroy tank cleanly before cascading parent/child test.
    sim.destroy_object_now(tank);
    sim.process_destroy_list();

    // Cascading destroy_object inside on_delete + ObjectArena generation ABA guard.
    let child = <SimContext as SimApi>::spawn_object(
        &mut sim,
        Default::default(),
        TeamId(1),
        Coord3D::default(),
        0.0,
    );
    let parent = <SimContext as SimApi>::spawn_object(
        &mut sim,
        Default::default(),
        TeamId(1),
        Coord3D::default(),
        0.0,
    );
    let parent_gen = sim.world.objects.generation(parent).expect("parent gen");
    let child_gen = sim.world.objects.generation(child).expect("child gen");
    let child_id = sim.world.objects.get(child).unwrap().id;

    let destroy_count = Arc::new(AtomicUsize::new(0));
    let delete_count = Arc::new(AtomicUsize::new(0));

    let parent_mod = sim.world.attach_module(
        parent,
        tag_parent,
        ModuleDataId::INVALID,
        FacetMask(FacetMask::DESTROY.0 | FacetMask::UPDATE.0),
        SleepyUpdatePhase::Normal,
        Box::new(CascadeDeleteModule {
            mod_name_key: mod_key,
            child_to_destroy: Some(child),
            destroy_observed: Arc::clone(&destroy_count),
            delete_observed: Arc::clone(&delete_count),
        }),
    );
    let child_mod = sim.world.attach_module(
        child,
        tag_child,
        ModuleDataId::INVALID,
        FacetMask(FacetMask::DESTROY.0 | FacetMask::UPDATE.0),
        SleepyUpdatePhase::Normal,
        Box::new(CascadeDeleteModule {
            mod_name_key: mod_key,
            child_to_destroy: None,
            destroy_observed: Arc::clone(&destroy_count),
            delete_observed: Arc::clone(&delete_count),
        }),
    );

    sim.destroy_object_now(parent);
    assert_eq!(destroy_count.load(Ordering::Relaxed), 1);
    sim.process_destroy_list();

    // Parent on_delete triggered child destroy, so both executed on_destroy + on_delete.
    assert_eq!(destroy_count.load(Ordering::Relaxed), 2);
    assert_eq!(delete_count.load(Ordering::Relaxed), 2);
    assert!(!sim.world.modules.is_alive(parent_mod));
    assert!(!sim.world.modules.is_alive(child_mod));
    assert!(sim.world.sleepy_updates.is_empty());
    sim.world
        .sleepy_updates
        .validate_invariants(&sim.world.modules)
        .expect("heap valid after cascading destroy");

    // Stale (slot, gen) rejected after respawn reuses slot.
    assert!(!sim.world.objects.is_slot_current(parent, parent_gen));
    assert!(!sim.world.objects.is_slot_current(child, child_gen));
    assert!(sim.world.objects.slot_by_id(child_id).is_none());

    let respawned = <SimContext as SimApi>::spawn_object(
        &mut sim,
        Default::default(),
        TeamId(1),
        Coord3D::default(),
        0.0,
    );
    assert_eq!(respawned, child);
    assert!(sim.world.objects.get_checked(child, child_gen).is_none());
    assert!(!sim.world.objects.is_slot_current(child, child_gen));
}
