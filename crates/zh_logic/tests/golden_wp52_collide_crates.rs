//! Gate V2-Golden test suite for `WP-5.2-D2` (remaining 10 `CollideModule` /
//! `CrateCollide` subclasses: `ConvertToCarBombCrateCollide`,
//! `ConvertToHijackedVehicleCrateCollide`, `FireWeaponCollide`, `HealCrateCollide`,
//! `MoneyCrateCollide`, `SalvageCrateCollide`, `ShroudCrateCollide`,
//! `SquishCollide`, `UnitCrateCollide`, `VeterancyCrateCollide` across 1,000 C++ vectors).

use std::collections::BTreeSet;
use std::path::Path;
use zh_common::golden_json::{
    decode_hex_bytes, encode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str,
    get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{
    Bool, NameKeyType, Real, Relationship, TeamId, VeterancyLevel, XferVersion,
    FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::arena::{FacetMask, ModuleDataId, ModuleSidecar, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_logic::combat::locomotor_set::LocomotorSetType;
use zh_logic::modules::module::{
    AiUpdateFacet, BehaviorModule, ModuleSnapshotHeader, ModuleType, 
};
use zh_logic::object::weapon::{Weapon, WeaponTemplate};
use zh_logic::object::weapon_set::WeaponSlotType;
use zh_logic::rules::combat_types::{ArmorSetType, DamageType, DeathType, WeaponSetType};
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::modules::update::physics_behavior::physics_mut;
use zh_logic::sim::{armor_set_flags_from_u8, MockSimApi, SimApi, SimView};

const STATIC_OBJS_INI: &str = "\
Object Cr52_SpawnedUnit
  KindOf = INFANTRY
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_Decoy
  KindOf = INFANTRY
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_HealInf
  KindOf = INFANTRY
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_HealBldg
  KindOf = STRUCTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
End
Object Cr52_VictimCar
  KindOf = VEHICLE
  VisionRange = 120
  ShroudClearingRange = 140
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  WeaponSet
    Weapon = PRIMARY Wpn_CollideSingle
  End
  WeaponSet
    Conditions = CARBOMB
    Weapon = PRIMARY Wpn_CarBombBlast
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  ExperienceValue = 10 20 30 40
End
Object Cr52_PlainCar
  KindOf = VEHICLE
  CrusherLevel = 2
  Geometry = CYLINDER
  GeometryMajorRadius = 15
  GeometryHeight = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = PhysicsBehavior ModuleTag_Physics
  End
End
Object Cr52_BoxTank
  KindOf = VEHICLE
  CrusherLevel = 2
  Geometry = BOX
  GeometryMajorRadius = 20
  GeometryMinorRadius = 10
  GeometryHeight = 10
  VisionRange = 120
  ShroudClearingRange = 130
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  ExperienceValue = 10 20 30 40
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300
    InitialHealth = 300
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = PhysicsBehavior ModuleTag_Physics
  End
  Behavior = EjectPilotDie ModuleTag_Eject
    AirCreationList = OCL_EjectEmpty
    GroundCreationList = OCL_EjectEmpty
    InvulnerableTime = 30
  End
End
Object Cr52_NonCrusherVeh
  KindOf = VEHICLE
  CrusherLevel = 0
  Geometry = CYLINDER
  GeometryMajorRadius = 15
  GeometryHeight = 10
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = PhysicsBehavior ModuleTag_Physics
  End
End
Object Cr52_Transport
  KindOf = VEHICLE TRANSPORT
  VisionRange = 120
  ShroudClearingRange = 120
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = TransportContain ModuleTag_Contain
    ContainMax = 2
    AllowInsideKindOf = INFANTRY
  End
End
Object Cr52_Boat
  KindOf = VEHICLE BOAT
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_Drone
  KindOf = VEHICLE DRONE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_Immune
  KindOf = VEHICLE IMMUNE_TO_CAPTURE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_AirUnit
  KindOf = AIRCRAFT VEHICLE
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL Loco_Air
End
Object Cr52_TrainableVeh
  KindOf = VEHICLE
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  ExperienceValue = 10 20 30 40
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL Loco_Ground
End
Object Cr52_NonTrainableVeh
  KindOf = VEHICLE
  IsTrainable = No
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL Loco_Ground
End
Object Cr52_SalvagerAll
  KindOf = VEHICLE SALVAGER WEAPON_SALVAGER ARMOR_SALVAGER
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_SalvagerWpn
  KindOf = VEHICLE SALVAGER WEAPON_SALVAGER
  IsTrainable = Yes
  ExperienceRequired = 0 50 100 200
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_SalvagerPlain
  KindOf = VEHICLE SALVAGER
  IsTrainable = No
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200
    InitialHealth = 200
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
End
Object Cr52_Mod_Heal
  KindOf = IMMOBILE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 10
    InitialHealth = 10
  End
  Behavior = HealCrateCollide ModuleTag_Collide
    ExecuteFX = FX_CrateExec
  End
End
Object Cr52_Mod_Shroud
  KindOf = IMMOBILE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 10
    InitialHealth = 10
  End
  Behavior = ShroudCrateCollide ModuleTag_Collide
    ExecuteFX = FX_CrateExec
  End
End
Object Cr52_Mod_CarBomb
  KindOf = INFANTRY
  VisionRange = 180
  ShroudClearingRange = 220
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = ConvertToCarBombCrateCollide ModuleTag_Collide
    FXList = FX_CarBomb
    ExecuteFX = FX_CrateExec
  End
End
Object Cr52_Mod_Hijack
  KindOf = INFANTRY
  VisionRange = 160
  ShroudClearingRange = 190
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 50
    InitialHealth = 50
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = HijackerUpdate ModuleTag_Hijack
  End
  Behavior = ConvertToHijackedVehicleCrateCollide ModuleTag_Collide
    ExecuteFX = FX_CrateExec
  End
End
Object Cr52_Mod_Squish
  KindOf = INFANTRY
  Geometry = CYLINDER
  GeometryMajorRadius = 5
  GeometryHeight = 5
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = SquishCollide ModuleTag_Collide
  End
End
Object Cr52_Mod_SquishHijacker
  KindOf = INFANTRY
  Geometry = CYLINDER
  GeometryMajorRadius = 5
  GeometryHeight = 5
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100
    InitialHealth = 100
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Behavior = HijackerUpdate ModuleTag_Hijack
  End
  Behavior = SquishCollide ModuleTag_Collide
  End
End
";

const WPN_LASER_INI: &str = "\
Weapon Wpn_CollideLaser
  PrimaryDamage = 10
  AttackRange = 100
  DamageType = SMALL_ARMS
  DeathType = NORMAL
  WeaponSpeed = 999999
  ClipSize = 3
  ClipReloadTime = 900
  DelayBetweenShots = 60
  AutoReloadsClip = Yes
  LeechRangeWeapon = Yes
End";

const WPN_SINGLE_INI: &str = "\
Weapon Wpn_CollideSingle
  PrimaryDamage = 20
  AttackRange = 100
  DamageType = EXPLOSION
  DeathType = EXPLODED
  WeaponSpeed = 999999
  ClipSize = 1
  ClipReloadTime = 300
  DelayBetweenShots = 30
  AutoReloadsClip = No
End";

const WPN_CARBOMB_INI: &str = "\
Weapon Wpn_CarBombBlast
  PrimaryDamage = 500
  AttackRange = 100
  DamageType = EXPLOSION
  DeathType = EXPLODED
  WeaponSpeed = 999999
End";

const LOCO_GROUND_INI: &str = "\
Locomotor Loco_Ground
  Surfaces = GROUND
  Speed = 30
  SpeedDamaged = 20
  TurnRate = 90
  TurnRateDamaged = 90
  Acceleration = 10
  Braking = 10
  Appearance = TREADS
End";

const LOCO_AIR_INI: &str = "\
Locomotor Loco_Air
  Surfaces = AIR
  Speed = 60
  SpeedDamaged = 40
  TurnRate = 120
  TurnRateDamaged = 120
  Acceleration = 20
  Braking = 20
  Appearance = WINGS
End";

struct StubAiBehavior {
    key: NameKeyType,
    locomotor_set: LocomotorSetType,
}

impl AiUpdateFacet for StubAiBehavior {}

impl BehaviorModule for StubAiBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.key
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        ModuleSidecar::Ai {
            moving: FALSE,
            attacking: FALSE,
            idle: TRUE,
            cur_locomotor_set: self.locomotor_set as u8,
        }
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

struct StubHijackerBehavior(NameKeyType);

impl BehaviorModule for StubHijackerBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.0
    }

    fn xfer(&mut self, _xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        Ok(())
    }
}

fn attach_stub_ai(api: &mut MockSimApi, slot: ObjectSlot) {
    let key = api.ctx.rules.name_keys.name_to_key("AIUpdateInterface");
    api.ctx.world.attach_module(
        slot,
        NameKeyType(902),
        ModuleDataId::INVALID,
        FacetMask::AI_UPDATE,
        SleepyUpdatePhase::Normal,
        Box::new(StubAiBehavior {
            key,
            locomotor_set: LocomotorSetType::Normal,
        }),
    );
}

fn attach_stub_hijacker(api: &mut MockSimApi, slot: ObjectSlot) {
    let key = api.ctx.rules.name_keys.name_to_key("HijackerUpdate");
    api.ctx.world.attach_module(
        slot,
        NameKeyType(903),
        ModuleDataId::INVALID,
        FacetMask(0x001),
        SleepyUpdatePhase::Normal,
        Box::new(StubHijackerBehavior(key)),
    );
}

/// `if (PhysicsBehavior* phys = obj->getPhysics()) phys->addVelocityTo(&vel);`
fn add_velocity(api: &mut MockSimApi, slot: ObjectSlot, velocity: Coord3D) {
    if let Some(p) = physics_mut(api, slot) {
        p.add_velocity(&velocity);
    }
}

fn spawn(api: &mut MockSimApi, name: &str, team: TeamId, pos: Coord3D, angle: Real) -> ObjectSlot {
    let id = api
        .ctx
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    let slot = api.spawn_object(id, team, pos, angle);
    let has_ai = !matches!(name, "Cr52_HealBldg" | "Cr52_Mod_Heal" | "Cr52_Mod_Shroud")
        && !name.starts_with("Cr52_Fire_")
        && !name.starts_with("Cr52_Money_")
        && !name.starts_with("Cr52_Salv_")
        && !name.starts_with("Cr52_Unit_");
    if has_ai {
        attach_stub_ai(api, slot);
    }
    slot
}

fn destroy_if_live(api: &mut MockSimApi, slot: ObjectSlot) {
    if api.object(slot).is_some_and(|o| !o.is_destroyed()) {
        api.ctx.destroy_object_now(slot);
    }
}

fn first_collide_module(api: &MockSimApi, slot: ObjectSlot) -> ModuleSlot {
    api.object(slot)
        .expect("object alive")
        .behaviors
        .iter()
        .find(|(_, mask)| mask.contains(FacetMask::COLLIDE))
        .map(|&(ms, _)| ms)
        .expect("missing Collide module")
}

fn collide_flags(api: &mut MockSimApi, ms: ModuleSlot) -> (i32, i32, i32, i32) {
    api.ctx
        .dispatch_module(ms, |m, _sim, _ctx| {
            let c = m.as_collide().expect("collide facet");
            (
                i32::from(c.is_car_bomb_crate_collide() != FALSE),
                i32::from(c.is_hijacked_vehicle_crate_collide() != FALSE),
                i32::from(c.is_salvage_crate_collide() != FALSE),
                i32::from(c.is_railroad() != FALSE),
            )
        })
        .expect("dispatch collide flags")
}

fn collide_would(api: &mut MockSimApi, ms: ModuleSlot, target: Option<ObjectSlot>) -> i32 {
    let Some(t) = target else {
        return 0;
    };
    let ok = api
        .ctx
        .dispatch_module(ms, |m, sim, ctx| {
            m.as_collide()
                .expect("collide facet")
                .would_like_to_collide_with(sim, ctx, t)
        })
        .expect("dispatch would");
    i32::from(ok != FALSE)
}

fn collide_exec(
    api: &mut MockSimApi,
    ms: ModuleSlot,
    target: Option<ObjectSlot>,
    loc: &Coord3D,
    normal: &Coord3D,
) {
    api.ctx
        .dispatch_module(ms, |m, sim, ctx| {
            m.as_collide_mut()
                .expect("collide facet")
                .on_collide(sim, ctx, target, Some(loc), Some(normal));
        })
        .expect("dispatch on_collide");
}

fn snapshot_mod(api: &mut MockSimApi, ms: ModuleSlot) -> (u32, String) {
    let mut xc = XferCrc::new();
    let mut xs = XferSave::new();
    api.ctx
        .dispatch_module(ms, |m, sim, _ctx| {
            m.crc(&mut xc);
            m.xfer(&mut xs, sim).expect("xfer save");
        })
        .expect("dispatch snapshot");
    (xc.get_crc(), encode_hex_bytes(xs.as_bytes()))
}

fn snapshot_load_and_resave(api: &mut MockSimApi, ms: ModuleSlot, xfer_hex: &str) -> String {
    let bytes = decode_hex_bytes(xfer_hex).expect("valid xfer hex");
    api.ctx
        .dispatch_module(ms, |m, sim, _ctx| {
            let mut xl = XferLoad::new(&bytes);
            m.xfer(&mut xl, sim).expect("xfer load");
            assert_eq!(xl.position(), bytes.len(), "unconsumed trailing xfer bytes");
            let mut xs = XferSave::new();
            m.xfer(&mut xs, sim).expect("xfer resave");
            encode_hex_bytes(xs.as_bytes())
        })
        .expect("dispatch load/resave")
}

fn check_crc(actual: u32, expected: u32, label: &str, seed: u32) {
    #[cfg(not(feature = "retail-crc"))]
    assert_eq!(actual, expected, "seed {seed} {label} crc mismatch");
    #[cfg(feature = "retail-crc")]
    assert_eq!(
        actual.swap_bytes(),
        expected,
        "seed {seed} {label} retail-crc mismatch"
    );
}

fn check_xfer(actual_hex: &str, expected_hex: &str, rt_hex: &str, label: &str, seed: u32) {
    #[cfg(not(feature = "retail-xfer-save"))]
    assert_eq!(actual_hex, expected_hex, "seed {seed} {label} xfer mismatch");
    #[cfg(not(feature = "retail-xfer-save"))]
    assert_eq!(rt_hex, expected_hex, "seed {seed} {label} rt_xfer mismatch");
    #[cfg(feature = "retail-xfer-save")]
    {
        let _ = expected_hex;
        assert_eq!(rt_hex, actual_hex, "seed {seed} {label} retail rt mismatch");
    }
}

fn is_destroyed_i32(api: &MockSimApi, slot: ObjectSlot) -> i32 {
    i32::from(api.object(slot).map_or(true, |o| o.is_destroyed()))
}

fn body_health(api: &MockSimApi, slot: ObjectSlot) -> Real {
    let bs = api.object(slot).unwrap().body_slot.unwrap();
    match api.module_sidecar(bs) {
        ModuleSidecar::Body { health, .. } => health,
        _ => panic!("expected Body sidecar"),
    }
}

fn apply_unresistable_damage(api: &mut MockSimApi, slot: ObjectSlot, amount: Real) {
    let mut d = DamageInfo::default();
    d.input.damage_type = DamageType::Unresistable;
    d.input.death_type = DeathType::NoneType;
    d.input.amount = amount;
    api.ctx.attempt_damage_on(slot, &mut d);
}

fn object_shroud_range(api: &MockSimApi, slot: ObjectSlot) -> Real {
    api.object_shroud_clearing_range(slot)
}

fn test_salvage_armor_mask(api: &MockSimApi, slot: ObjectSlot) -> u32 {
    let obj = api.object(slot).unwrap();
    let flags = if let Some(bs) = obj.body_slot {
        if let ModuleSidecar::Body { armor_set_bits, .. } = api.module_sidecar(bs) {
            armor_set_flags_from_u8(armor_set_bits)
        } else {
            obj.armor_set_flags.clone()
        }
    } else {
        obj.armor_set_flags.clone()
    };
    let one = u32::from(flags.test(ArmorSetType::CrateUpgradeOne.as_i32()) != FALSE);
    let two = u32::from(flags.test(ArmorSetType::CrateUpgradeTwo.as_i32()) != FALSE);
    one | (two << 1)
}

fn vet_level_from_i32(v: i32) -> VeterancyLevel {
    VeterancyLevel::from_i32(v)
}

fn rel_from_i32(v: i32) -> Relationship {
    match v {
        0 => Relationship::Enemies,
        2 => Relationship::Allies,
        _ => Relationship::Neutral,
    }
}

fn parse_i32_vec_as_u32(line: &str, key: &str) -> Vec<u32> {
    get_i32_vec(line, key)
        .unwrap_or_else(|| panic!("missing array {key}"))
        .into_iter()
        .map(|v| v as u32)
        .collect()
}

fn verify_seed0_diagnostics(diag_json: &str) {
    assert!(diag_json.contains("\"cb\":{\"is_cb\":1,\"is_rr\":0,\"would\":1,\"dest\":1,\"ws\":1,\"st\":1,\"team_b\":1,\"vet\":2,\"shroud_bits\":1130102784"));
    assert!(diag_json.contains("\"hj\":{\"is_hj\":1,\"would\":1,\"dest\":0,\"st\":1,\"team_b\":1,\"tgt_vet\":2,\"jkr_vet\":2,\"masked\":1,\"shroud_bits\":1128136704"));
    assert!(diag_json.contains("\"fw\":{\"ammo\":2,\"wstatus\":2,\"next_fire\":189,\"last_fire\":187,\"ever\":0"));
    assert!(diag_json.contains("\"hl\":{\"would\":1,\"dest\":1,\"u1\":1112014848,\"u2\":1128792064,\"u3\":1128792064,\"u4\":1101004800,\"en\":1103626240"));
    assert!(diag_json.contains("\"mn\":{\"would\":1,\"dest\":1,\"money\":600"));
    assert!(diag_json.contains("\"sv\":{\"is_sv\":1,\"would\":1,\"dest\":1,\"armor\":2,\"wpn\":0,\"vet\":0,\"exp\":0,\"money\":0,\"rng_crc\":2708529444"));
    assert!(diag_json.contains("\"sh\":{\"would\":1,\"dest\":1"));
    assert!(diag_json.contains("\"sq\":{\"dead\":1,\"hp\":0,\"def\":0"));
    assert!(diag_json.contains("\"un\":{\"would\":1,\"dest\":1,\"spawned\":2,\"px\":1128792064,\"py\":1128792064,\"rng_crc\":2888267676"));
    assert!(diag_json.contains("\"vt\":{\"would\":1,\"dest\":1,\"tgt_vet\":0,\"tgt_exp\":0,\"in_vet\":2,\"in_exp\":100,\"masked_vet\":2,\"out_vet\":0,\"enemy_vet\":0"));
}

#[test]
fn wp52_collide_crates_module_registry_parity() {
    let api = MockSimApi::new();
    for name in [
        "ConvertToCarBombCrateCollide",
        "ConvertToHijackedVehicleCrateCollide",
        "FireWeaponCollide",
        "HealCrateCollide",
        "MoneyCrateCollide",
        "SalvageCrateCollide",
        "ShroudCrateCollide",
        "SquishCollide",
        "UnitCrateCollide",
        "VeterancyCrateCollide",
    ] {
        assert!(
            api.ctx
                .rules
                .module_registry
                .find_descriptor(name, ModuleType::Behavior)
                .is_some_and(|d| d.parse_data.is_some() && d.create_module.is_some()),
            "expected CollideModule {name} to be registered"
        );
    }
}

#[test]
fn wp52_collide_crates_golden_vectors() {
    let json_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/golden_vectors/WP52_CollideCrates.json");
    let json_text = std::fs::read_to_string(&json_path).expect("read WP52_CollideCrates.json");
    let vec_idx = json_text.find("\"vectors\"").expect("vectors key");
    verify_seed0_diagnostics(&json_text[..vec_idx]);

    let lines = extract_vector_lines(&json_text);
    assert_eq!(lines.len(), 1000, "expected 1000 golden vector seeds");

    let mut api = MockSimApi::new();
    // The harness world: 20-unit partition cells over its 1000 by 1000 terrain.
    api.ctx.init_partition(
        20.0,
        Region3D {
            lo: Coord3D::new(0.0, 0.0, 0.0),
            hi: Coord3D::new(1000.0, 1000.0, 100.0),
        },
    );
    api.ctx.rules.fx_list_names.insert("FX_CrateExec".to_owned());
    api.ctx.rules.fx_list_names.insert("FX_CarBomb".to_owned());
    api.ctx.rules.ocl_names.insert("OCL_EjectEmpty".to_owned());

    api.ctx
        .rules
        .upgrade_center
        .load_from_ini(
            &mut IniReader::new(
                "upg_d2.ini",
                b"Upgrade Upg_MoneyBoost\n  Type = PLAYER\nEnd\n",
                IniLoadType::Overwrite,
            ),
            &mut api.ctx.rules.name_keys,
        )
        .expect("load Upg_MoneyBoost");

    for wpn_text in [WPN_LASER_INI, WPN_SINGLE_INI, WPN_CARBOMB_INI] {
        let mut wt = WeaponTemplate::new();
        wt.parse_ini_block(wpn_text).expect("parse weapon");
        api.ctx.rules.weapon_store.add_template(wt);
    }

    for loco_text in [LOCO_GROUND_INI, LOCO_AIR_INI] {
        let mut lt = LocomotorTemplate::new();
        lt.parse_ini_block(loco_text).expect("parse locomotor");
        api.ctx.rules.locomotor_store.add_template(lt);
    }

    api.ctx
        .rules
        .load_thing_templates_from_ini(&mut IniReader::new(
            "static_d2.ini",
            STATIC_OBJS_INI.as_bytes(),
            IniLoadType::Overwrite,
        ))
        .expect("load static templates");

    let team_a = TeamId(0);
    let team_b = TeamId(1);
    let team_c = TeamId(2);
    api.ctx.world.team_is_neutral.insert(team_a, TRUE);
    api.ctx.world.team_is_neutral.insert(team_b, FALSE);
    api.ctx.world.team_is_neutral.insert(team_c, FALSE);
    api.ctx.world.team_players.insert(team_a, 0);
    api.ctx.world.team_players.insert(team_b, 1);
    api.ctx.world.team_players.insert(team_c, 2);
    api.ctx
        .world
        .player_relationships
        .insert((0, 1), Relationship::Neutral);
    api.ctx
        .world
        .player_relationships
        .insert((1, 0), Relationship::Neutral);
    api.ctx
        .world
        .player_relationships
        .insert((0, 2), Relationship::Neutral);
    api.ctx
        .world
        .player_relationships
        .insert((2, 0), Relationship::Neutral);

    let upg_mask = api
        .ctx
        .rules
        .upgrade_center
        .lookup_upgrade("Upg_MoneyBoost", &api.ctx.rules.name_keys)
        .expect("Upg_MoneyBoost registered")
        .upgrade_mask()
        .clone();

    let col_loc = Coord3D {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    let col_norm = Coord3D {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };

    for line in &lines {
        let seed = get_u32(line, "seed").expect("seed");
        let frame0 = get_u32(line, "frame").expect("frame");
        api.ctx.world.frame = frame0;

        let rel_bc = rel_from_i32(get_i32(line, "rel_bc").expect("rel_bc"));
        api.ctx.world.player_relationships.insert((1, 2), rel_bc);
        api.ctx.world.player_relationships.insert((2, 1), rel_bc);
        api.ctx.world.player_upgrades.clear();

        let cb_sel = get_u32(line, "cb_sel").expect("cb_sel");
        let cb_goal = get_i32(line, "cb_goal").expect("cb_goal") != 0;
        let cb_vet = get_i32(line, "cb_vet").expect("cb_vet");

        let hj_sel = get_u32(line, "hj_sel").expect("hj_sel");
        let hj_goal = get_i32(line, "hj_goal").expect("hj_goal") != 0;
        let hj_jvet = get_i32(line, "hj_jvet").expect("hj_jvet");
        let hj_tvet = get_i32(line, "hj_tvet").expect("hj_tvet");

        let fw_wpn = get_u32(line, "fw_wpn").expect("fw_wpn");
        let fw_once = get_i32(line, "fw_once").expect("fw_once") != 0;
        let fw_req = get_u32(line, "fw_req").expect("fw_req");
        let fw_forb = get_u32(line, "fw_forb").expect("fw_forb");
        let fw_st = get_u32(line, "fw_st").expect("fw_st");
        let fw_gnd = get_i32(line, "fw_gnd").expect("fw_gnd") != 0;

        let hl_pick = get_u32(line, "hl_pick").expect("hl_pick");

        let mn_prov = get_u32(line, "mn_prov").expect("mn_prov");
        let mn_bcfg = get_i32(line, "mn_bcfg").expect("mn_bcfg") != 0;
        let mn_bamt = get_i32(line, "mn_bamt").expect("mn_bamt");
        let mn_upg = get_i32(line, "mn_upg").expect("mn_upg") != 0;
        let mn_m0 = get_u32(line, "mn_m0").expect("mn_m0");

        let sv_sel = get_u32(line, "sv_sel").expect("sv_sel");
        let sv_arm = get_u32(line, "sv_arm").expect("sv_arm");
        let sv_wpn = get_u32(line, "sv_wpn").expect("sv_wpn");
        let sv_vet = get_i32(line, "sv_vet").expect("sv_vet");
        let sv_wpct = get_u32(line, "sv_wpct").expect("sv_wpct");
        let sv_lpct = get_u32(line, "sv_lpct").expect("sv_lpct");
        let sv_min = get_u32(line, "sv_min").expect("sv_min");
        let sv_max = get_u32(line, "sv_max").expect("sv_max");
        let sv_rng = get_u32(line, "sv_rng").expect("sv_rng");

        let sq_kind = get_u32(line, "sq_kind").expect("sq_kind");
        let sq_vel = get_u32(line, "sq_vel").expect("sq_vel");
        let sq_dist = get_u32(line, "sq_dist").expect("sq_dist");
        let sq_ang = f32::from_bits(get_u32(line, "sq_ang").expect("sq_ang"));
        let sq_hjk = get_i32(line, "sq_hjk").expect("sq_hjk") != 0;
        let sq_def = get_i32(line, "sq_def").expect("sq_def") != 0;
        let sq_gnd = get_i32(line, "sq_gnd").expect("sq_gnd") != 0;

        let un_cnt = get_u32(line, "un_cnt").expect("un_cnt");
        let un_ok = get_i32(line, "un_ok").expect("un_ok") != 0;
        let un_rng = get_u32(line, "un_rng").expect("un_rng");

        let vt_rng = get_u32(line, "vt_rng").expect("vt_rng");
        let vt_add = get_i32(line, "vt_add").expect("vt_add") != 0;
        let vt_plt = get_i32(line, "vt_plt").expect("vt_plt") != 0;
        let vt_ovet = get_i32(line, "vt_ovet").expect("vt_ovet");
        let vt_sel = get_u32(line, "vt_sel").expect("vt_sel");
        let vt_goal = get_i32(line, "vt_goal").expect("vt_goal") != 0;

        let t_fire = format!("Cr52_Fire_{seed}");
        let t_money = format!("Cr52_Money_{seed}");
        let t_salv = format!("Cr52_Salv_{seed}");
        let t_unit = format!("Cr52_Unit_{seed}");
        let t_vet = format!("Cr52_Vet_{seed}");

        let fw_req_line = match fw_req {
            1 => "    RequiredStatus = AIRBORNE_TARGET\n",
            2 => "    RequiredStatus = HIJACKED\n",
            _ => "",
        };
        let fw_forb_line = match fw_forb {
            1 => "    ForbiddenStatus = UNDER_CONSTRUCTION\n",
            2 => "    ForbiddenStatus = HIJACKED\n",
            _ => "",
        };
        let boost_line = if mn_bcfg {
            format!("    UpgradedBoost = UpgradeType:Upg_MoneyBoost Boost:{mn_bamt}\n")
        } else {
            String::new()
        };
        let dyn_ini = format!(
            "Object {t_fire}\n\
               KindOf = IMMOBILE\n\
               Body = ActiveBody ModuleTag_Body\n\
                 MaxHealth = 100\n\
                 InitialHealth = 100\n\
               End\n\
               Behavior = FireWeaponCollide ModuleTag_Collide\n\
                 CollideWeapon = {fw_wpn_name}\n\
                 FireOnce = {fw_once_s}\n\
                 {fw_req_line}\
                 {fw_forb_line}\
               End\n\
             End\n\
             Object {t_money}\n\
               KindOf = IMMOBILE\n\
               Body = ActiveBody ModuleTag_Body\n\
                 MaxHealth = 10\n\
                 InitialHealth = 10\n\
               End\n\
               Behavior = MoneyCrateCollide ModuleTag_Collide\n\
                 MoneyProvided = {mn_prov}\n\
                 {boost_line}\
                 ExecuteFX = FX_CrateExec\n\
               End\n\
             End\n\
             Object {t_salv}\n\
               KindOf = IMMOBILE\n\
               Body = ActiveBody ModuleTag_Body\n\
                 MaxHealth = 10\n\
                 InitialHealth = 10\n\
               End\n\
               Behavior = SalvageCrateCollide ModuleTag_Collide\n\
                 WeaponChance = {sv_wpct}%\n\
                 LevelChance = {sv_lpct}%\n\
                 MoneyChance = 75%\n\
                 MinMoney = {sv_min}\n\
                 MaxMoney = {sv_max}\n\
                 ExecuteFX = FX_CrateExec\n\
               End\n\
             End\n\
             Object {t_unit}\n\
               KindOf = IMMOBILE\n\
               Body = ActiveBody ModuleTag_Body\n\
                 MaxHealth = 10\n\
                 InitialHealth = 10\n\
               End\n\
               Behavior = UnitCrateCollide ModuleTag_Collide\n\
                 UnitCount = {un_cnt}\n\
                 UnitName = {un_name}\n\
                 ExecuteFX = FX_CrateExec\n\
               End\n\
             End\n\
             Object {t_vet}\n\
               KindOf = INFANTRY\n\
               IsTrainable = Yes\n\
               ExperienceRequired = 0 50 100 200\n\
               Body = ActiveBody ModuleTag_Body\n\
                 MaxHealth = 50\n\
                 InitialHealth = 50\n\
               End\n\
               Behavior = AIUpdateInterface ModuleTag_AI\n\
               End\n\
               Behavior = VeterancyCrateCollide ModuleTag_Collide\n\
                 EffectRange = {vt_rng}\n\
                 AddsOwnerVeterancy = {vt_add_s}\n\
                 IsPilot = {vt_plt_s}\n\
                 ExecuteFX = FX_CrateExec\n\
               End\n\
             End\n",
            fw_wpn_name = if fw_wpn == 0 {
                "Wpn_CollideLaser"
            } else {
                "Wpn_CollideSingle"
            },
            fw_once_s = if fw_once { "Yes" } else { "No" },
            un_name = if un_ok {
                "Cr52_SpawnedUnit"
            } else {
                "Cr52_DoesNotExist"
            },
            vt_add_s = if vt_add { "Yes" } else { "No" },
            vt_plt_s = if vt_plt { "Yes" } else { "No" },
        );
        api.ctx
            .rules
            .load_thing_templates_from_ini(&mut IniReader::new(
                "dyn_d2.ini",
                dyn_ini.as_bytes(),
                IniLoadType::Overwrite,
            ))
            .expect("load dynamic templates");

        // 1. ConvertToCarBombCrateCollide (PKT-201)
        let cb_crate = spawn(
            &mut api,
            "Cr52_Mod_CarBomb",
            team_b,
            Coord3D { x: 10.0, y: 10.0, z: 0.0 },
            0.0,
        );
        api.set_veterancy_level(cb_crate, vet_level_from_i32(cb_vet), FALSE, FALSE);
        let cb_decoy = spawn(
            &mut api,
            "Cr52_Decoy",
            team_c,
            Coord3D { x: 30.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let cb_pos = Coord3D { x: 10.0, y: 10.0, z: 0.0 };
        let cb_tgt = match cb_sel {
            0 => spawn(&mut api, "Cr52_VictimCar", team_c, cb_pos, 0.0),
            1 => spawn(&mut api, "Cr52_PlainCar", team_c, cb_pos, 0.0),
            2 => spawn(&mut api, "Cr52_Boat", team_c, cb_pos, 0.0),
            3 => spawn(&mut api, "Cr52_AirUnit", team_c, cb_pos, 0.0),
            4 => {
                let o = spawn(&mut api, "Cr52_VictimCar", team_c, cb_pos, 0.0);
                api.set_status(o, ObjectStatusType::IsCarbomb);
                o
            }
            5 => {
                let o = spawn(&mut api, "Cr52_VictimCar", team_c, cb_pos, 0.0);
                api.set_weapon_set_flag(o, WeaponSetType::Carbomb, TRUE);
                o
            }
            _ => {
                let o = spawn(&mut api, "Cr52_VictimCar", team_c, cb_pos, 0.0);
                api.object_mut(o).unwrap().set_effectively_dead(TRUE);
                o
            }
        };
        api.object_mut(cb_crate).unwrap().ai_goal_object =
            Some(if cb_goal { cb_tgt } else { cb_decoy });
        let m_cb = first_collide_module(&api, cb_crate);
        let (cb_is_cb, _, _, cb_is_rr) = collide_flags(&mut api, m_cb);
        let cb_would = collide_would(&mut api, m_cb, Some(cb_tgt));
        collide_exec(&mut api, m_cb, Some(cb_tgt), &col_loc, &col_norm);
        let cb_dest = is_destroyed_i32(&api, cb_crate);
        let cb_has_ws = i32::from(
            api.object(cb_tgt)
                .unwrap()
                .weapon_set_flags
                .test(WeaponSetType::Carbomb.as_i32())
                != FALSE,
        );
        let cb_has_st = i32::from(
            api.object(cb_tgt)
                .unwrap()
                .status
                .test(ObjectStatusType::IsCarbomb.as_i32())
                != FALSE,
        );
        let cb_team_b = i32::from(api.object(cb_tgt).unwrap().team_id == team_b);
        let cb_tgt_vet = api.object(cb_tgt).unwrap().experience_tracker.current_level.as_i32();
        let cb_shroud_bits = object_shroud_range(&api, cb_tgt).to_bits();
        let (cb_crc, cb_xfer) = snapshot_mod(&mut api, m_cb);
        let cb_rt = snapshot_load_and_resave(&mut api, m_cb, &cb_xfer);
        let exp_cb = parse_i32_vec_as_u32(line, "cb");
        assert_eq!(
            vec![
                cb_is_cb as u32,
                cb_is_rr as u32,
                cb_would as u32,
                cb_dest as u32,
                cb_has_ws as u32,
                cb_has_st as u32,
                cb_team_b as u32,
                cb_tgt_vet as u32,
                cb_shroud_bits,
            ],
            exp_cb,
            "seed {seed} cb mismatch"
        );
        check_crc(cb_crc, get_u32(line, "cb_crc").expect("cb_crc"), "cb", seed);
        check_xfer(
            &cb_xfer,
            get_str(line, "cb_xfer").expect("cb_xfer"),
            &cb_rt,
            "cb",
            seed,
        );
        let exp_cb_rt = get_str(line, "cb_rt").expect("cb_rt");
        #[cfg(not(feature = "retail-xfer-save"))]
        assert_eq!(cb_rt, exp_cb_rt, "seed {seed} cb_rt mismatch");
        #[cfg(feature = "retail-xfer-save")]
        let _ = exp_cb_rt;

        destroy_if_live(&mut api, cb_crate);
        destroy_if_live(&mut api, cb_tgt);
        destroy_if_live(&mut api, cb_decoy);
        api.ctx.process_destroy_list();

        // 2. ConvertToHijackedVehicleCrateCollide (PKT-202)
        let hj_pos = Coord3D { x: 20.0, y: 10.0, z: 0.0 };
        let hj_crate = spawn(&mut api, "Cr52_Mod_Hijack", team_b, hj_pos, 0.0);
        attach_stub_hijacker(&mut api, hj_crate);
        api.set_veterancy_level(hj_crate, vet_level_from_i32(hj_jvet), FALSE, FALSE);
        let hj_decoy = spawn(
            &mut api,
            "Cr52_Decoy",
            team_c,
            Coord3D { x: 40.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let mut hj_pass: Option<ObjectSlot> = None;
        let hj_tgt = match hj_sel {
            0 => spawn(&mut api, "Cr52_BoxTank", team_c, hj_pos, 0.0),
            1 => spawn(&mut api, "Cr52_PlainCar", team_c, hj_pos, 0.0),
            2 => spawn(&mut api, "Cr52_Transport", team_c, hj_pos, 0.0),
            3 => {
                let o = spawn(&mut api, "Cr52_Transport", team_c, hj_pos, 0.0);
                let p = spawn(&mut api, "Cr52_Decoy", team_c, hj_pos, 0.0);
                api.contain_add(o, p);
                hj_pass = Some(p);
                o
            }
            4 => spawn(&mut api, "Cr52_Immune", team_c, hj_pos, 0.0),
            5 => spawn(&mut api, "Cr52_AirUnit", team_c, hj_pos, 0.0),
            6 => spawn(&mut api, "Cr52_Boat", team_c, hj_pos, 0.0),
            7 => spawn(&mut api, "Cr52_Drone", team_c, hj_pos, 0.0),
            8 => {
                let o = spawn(&mut api, "Cr52_BoxTank", team_c, hj_pos, 0.0);
                api.set_status(o, ObjectStatusType::Hijacked);
                o
            }
            _ => {
                spawn(&mut api, "Cr52_BoxTank", team_c, hj_pos, 0.0)
            }
        };
        api.set_veterancy_level(hj_tgt, vet_level_from_i32(hj_tvet), FALSE, FALSE);
        if hj_sel >= 9 {
            api.object_mut(hj_tgt).unwrap().set_effectively_dead(TRUE);
        }
        api.object_mut(hj_crate).unwrap().ai_goal_object =
            Some(if hj_goal { hj_tgt } else { hj_decoy });
        let m_hj = first_collide_module(&api, hj_crate);
        let (_, hj_is_hj, _, _) = collide_flags(&mut api, m_hj);
        let hj_would = collide_would(&mut api, m_hj, Some(hj_tgt));
        collide_exec(&mut api, m_hj, Some(hj_tgt), &col_loc, &col_norm);
        let hj_dest = is_destroyed_i32(&api, hj_crate);
        let hj_has_st = i32::from(
            api.object(hj_tgt)
                .unwrap()
                .status
                .test(ObjectStatusType::Hijacked.as_i32())
                != FALSE,
        );
        let hj_team_b = i32::from(api.object(hj_tgt).unwrap().team_id == team_b);
        let hj_tgt_vet_after = api.object(hj_tgt).unwrap().experience_tracker.current_level.as_i32();
        let hj_jkr_vet_after = api
            .object(hj_crate)
            .unwrap()
            .experience_tracker
            .current_level
            .as_i32();
        let hj_jkr_masked = i32::from(
            api.object(hj_crate)
                .unwrap()
                .status
                .test(ObjectStatusType::Masked.as_i32())
                != FALSE,
        );
        let hj_shroud_bits = object_shroud_range(&api, hj_tgt).to_bits();
        let (hj_crc, hj_xfer) = snapshot_mod(&mut api, m_hj);
        let hj_rt = snapshot_load_and_resave(&mut api, m_hj, &hj_xfer);
        let exp_hj = parse_i32_vec_as_u32(line, "hj");
        assert_eq!(
            vec![
                hj_is_hj as u32,
                hj_would as u32,
                hj_dest as u32,
                hj_has_st as u32,
                hj_team_b as u32,
                hj_tgt_vet_after as u32,
                hj_jkr_vet_after as u32,
                hj_jkr_masked as u32,
                hj_shroud_bits,
            ],
            exp_hj,
            "seed {seed} hj mismatch"
        );
        check_crc(hj_crc, get_u32(line, "hj_crc").expect("hj_crc"), "hj", seed);
        check_xfer(
            &hj_xfer,
            get_str(line, "hj_xfer").expect("hj_xfer"),
            &hj_rt,
            "hj",
            seed,
        );

        if let Some(p) = hj_pass {
            destroy_if_live(&mut api, p);
        }
        destroy_if_live(&mut api, hj_crate);
        destroy_if_live(&mut api, hj_tgt);
        destroy_if_live(&mut api, hj_decoy);
        api.ctx.process_destroy_list();

        // 3. FireWeaponCollide (PKT-215)
        let fw_pos = Coord3D { x: 30.0, y: 10.0, z: 0.0 };
        let fw_obj = spawn(&mut api, &t_fire, team_b, fw_pos, 0.0);
        let fw_tgt = spawn(&mut api, "Cr52_PlainCar", team_c, fw_pos, 0.0);
        match fw_st {
            1 => api.set_status(fw_obj, ObjectStatusType::AirborneTarget),
            2 => api.set_status(fw_obj, ObjectStatusType::UnderConstruction),
            3 => api.set_status(fw_obj, ObjectStatusType::Hijacked),
            _ => {}
        }
        let m_fw = first_collide_module(&api, fw_obj);
        let fw_arg = if fw_gnd { None } else { Some(fw_tgt) };
        collide_exec(&mut api, m_fw, fw_arg, &col_loc, &col_norm);
        api.ctx.world.frame = frame0 + 5;
        collide_exec(&mut api, m_fw, fw_arg, &col_loc, &col_norm);
        let now_fw = api.ctx.world.frame;
        let (fw_crc, fw_xfer) = snapshot_mod(&mut api, m_fw);
        let fw_rt = snapshot_load_and_resave(&mut api, m_fw, &fw_xfer);
        let (fw_ammo, fw_wstatus, fw_next_fire, fw_last_fire, fw_ever) = {
            let fw_bytes = decode_hex_bytes(&fw_xfer).expect("fw_xfer hex");
            let mut xl = XferLoad::new(&fw_bytes);
            let mut ver: XferVersion = 1;
            xl.xfer_version(&mut ver, 1).expect("fw xfer version");
            ModuleSnapshotHeader::xfer_collide_module_header(&mut xl).expect("fw collide header");
            let mut has_w: Bool = FALSE;
            xl.xfer_bool(&mut has_w).expect("fw weapon_present");
            let wpn_name = if fw_wpn == 0 {
                "Wpn_CollideLaser"
            } else {
                "Wpn_CollideSingle"
            };
            let wt = api
                .ctx
                .rules
                .weapon_store
                .find_weapon_template(wpn_name)
                .expect("weapon template")
                .clone();
            let mut w = Weapon::new(wt, WeaponSlotType::Primary, 0);
            if has_w != FALSE {
                w.xfer(&mut xl).expect("fw weapon xfer");
            }
            let mut ever: Bool = FALSE;
            xl.xfer_bool(&mut ever).expect("fw ever_fired");
            assert_eq!(xl.position(), fw_bytes.len(), "fw xfer length");
            if has_w != FALSE {
                (
                    w.ammo_in_clip as i32,
                    w.get_status(now_fw).as_i32(),
                    w.when_we_can_fire_again as i32,
                    w.last_fire_frame as i32,
                    i32::from(ever != FALSE),
                )
            } else {
                (0, -1, 0, 0, i32::from(ever != FALSE))
            }
        };
        let exp_fw = get_i32_vec(line, "fw").expect("fw vec");
        assert_eq!(
            vec![fw_ammo, fw_wstatus, fw_next_fire, fw_last_fire, fw_ever],
            exp_fw,
            "seed {seed} fw mismatch"
        );
        check_crc(fw_crc, get_u32(line, "fw_crc").expect("fw_crc"), "fw", seed);
        check_xfer(
            &fw_xfer,
            get_str(line, "fw_xfer").expect("fw_xfer"),
            &fw_rt,
            "fw",
            seed,
        );
        let exp_fw_rt = get_str(line, "fw_rt").expect("fw_rt");
        #[cfg(not(feature = "retail-xfer-save"))]
        assert_eq!(fw_rt, exp_fw_rt, "seed {seed} fw_rt mismatch");
        #[cfg(feature = "retail-xfer-save")]
        let _ = exp_fw_rt;

        destroy_if_live(&mut api, fw_obj);
        destroy_if_live(&mut api, fw_tgt);
        api.ctx.process_destroy_list();
        api.ctx.world.frame = frame0;

        // 4. HealCrateCollide (PKT-219)
        let hl_crate = spawn(
            &mut api,
            "Cr52_Mod_Heal",
            team_b,
            Coord3D { x: 40.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let hl_u1 = spawn(
            &mut api,
            "Cr52_HealInf",
            team_c,
            Coord3D { x: 41.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let hl_u2 = spawn(
            &mut api,
            "Cr52_PlainCar",
            team_c,
            Coord3D { x: 42.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let hl_u3 = spawn(
            &mut api,
            "Cr52_HealBldg",
            team_c,
            Coord3D { x: 43.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let hl_u4 = spawn(
            &mut api,
            "Cr52_HealInf",
            team_c,
            Coord3D { x: 44.0, y: 10.0, z: 0.0 },
            0.0,
        );
        let hl_en = spawn(
            &mut api,
            "Cr52_HealInf",
            team_b,
            Coord3D { x: 45.0, y: 10.0, z: 0.0 },
            0.0,
        );
        apply_unresistable_damage(&mut api, hl_u1, 30.0);
        apply_unresistable_damage(&mut api, hl_u2, 120.0);
        apply_unresistable_damage(&mut api, hl_u3, 100.0);
        apply_unresistable_damage(&mut api, hl_u4, 30.0);
        api.object_mut(hl_u4).unwrap().set_effectively_dead(TRUE);
        apply_unresistable_damage(&mut api, hl_en, 25.0);
        let hl_picker = match hl_pick {
            0 => hl_u2,
            1 => hl_en,
            _ => hl_u4,
        };
        let m_hl = first_collide_module(&api, hl_crate);
        let hl_would = collide_would(&mut api, m_hl, Some(hl_picker));
        collide_exec(&mut api, m_hl, Some(hl_picker), &col_loc, &col_norm);
        let hl_dest = is_destroyed_i32(&api, hl_crate);
        let (hl_crc, hl_xfer) = snapshot_mod(&mut api, m_hl);
        let hl_rt = snapshot_load_and_resave(&mut api, m_hl, &hl_xfer);
        let exp_hl = parse_i32_vec_as_u32(line, "hl");
        assert_eq!(
            vec![
                hl_would as u32,
                hl_dest as u32,
                body_health(&api, hl_u1).to_bits(),
                body_health(&api, hl_u2).to_bits(),
                body_health(&api, hl_u3).to_bits(),
                body_health(&api, hl_u4).to_bits(),
                body_health(&api, hl_en).to_bits(),
            ],
            exp_hl,
            "seed {seed} hl mismatch"
        );
        check_crc(hl_crc, get_u32(line, "hl_crc").expect("hl_crc"), "hl", seed);
        check_xfer(
            &hl_xfer,
            get_str(line, "hl_xfer").expect("hl_xfer"),
            &hl_rt,
            "hl",
            seed,
        );
        for o in [hl_crate, hl_u1, hl_u2, hl_u3, hl_u4, hl_en] {
            destroy_if_live(&mut api, o);
        }
        api.ctx.process_destroy_list();

        // 5. MoneyCrateCollide (PKT-224)
        api.ctx.world.player_money.insert(team_c, mn_m0);
        if mn_upg {
            api.ctx
                .world
                .player_upgrades
                .entry(team_c)
                .or_default()
                .set_flags(&upg_mask);
        }
        let mn_pos = Coord3D { x: 50.0, y: 10.0, z: 0.0 };
        let mn_crate = spawn(&mut api, &t_money, team_b, mn_pos, 0.0);
        let mn_picker = spawn(&mut api, "Cr52_PlainCar", team_c, mn_pos, 0.0);
        let m_mn = first_collide_module(&api, mn_crate);
        let mn_would = collide_would(&mut api, m_mn, Some(mn_picker));
        collide_exec(&mut api, m_mn, Some(mn_picker), &col_loc, &col_norm);
        let mn_dest = is_destroyed_i32(&api, mn_crate);
        let mn_money_after = api.player_money(team_c);
        let (mn_crc, mn_xfer) = snapshot_mod(&mut api, m_mn);
        let mn_rt = snapshot_load_and_resave(&mut api, m_mn, &mn_xfer);
        let exp_mn = parse_i32_vec_as_u32(line, "mn");
        assert_eq!(
            vec![mn_would as u32, mn_dest as u32, mn_money_after],
            exp_mn,
            "seed {seed} mn mismatch"
        );
        check_crc(mn_crc, get_u32(line, "mn_crc").expect("mn_crc"), "mn", seed);
        check_xfer(
            &mn_xfer,
            get_str(line, "mn_xfer").expect("mn_xfer"),
            &mn_rt,
            "mn",
            seed,
        );
        destroy_if_live(&mut api, mn_crate);
        destroy_if_live(&mut api, mn_picker);
        api.ctx.process_destroy_list();

        // 6. SalvageCrateCollide (PKT-240)
        api.ctx.world.player_money.insert(team_c, 0);
        let sv_pos = Coord3D { x: 60.0, y: 10.0, z: 0.0 };
        let sv_crate = spawn(&mut api, &t_salv, team_b, sv_pos, 0.0);
        let sv_tmpl = match sv_sel {
            0 => "Cr52_SalvagerAll",
            1 => "Cr52_SalvagerWpn",
            2 => "Cr52_SalvagerPlain",
            _ => "Cr52_PlainCar",
        };
        let sv_picker = spawn(&mut api, sv_tmpl, team_c, sv_pos, 0.0);
        if sv_arm == 1 {
            api.set_armor_set_flag(sv_picker, ArmorSetType::CrateUpgradeOne, TRUE);
        } else if sv_arm == 2 {
            api.set_armor_set_flag(sv_picker, ArmorSetType::CrateUpgradeTwo, TRUE);
        }
        if sv_wpn == 1 {
            api.set_weapon_set_flag(sv_picker, WeaponSetType::CrateUpgradeOne, TRUE);
        } else if sv_wpn == 2 {
            api.set_weapon_set_flag(sv_picker, WeaponSetType::CrateUpgradeTwo, TRUE);
        }
        api.set_veterancy_level(sv_picker, vet_level_from_i32(sv_vet), FALSE, FALSE);
        let m_sv = first_collide_module(&api, sv_crate);
        let (_, _, sv_is_sv, _) = collide_flags(&mut api, m_sv);
        let sv_would = collide_would(&mut api, m_sv, Some(sv_picker));
        api.ctx.world.rng.init_random_with_seed(sv_rng);
        collide_exec(&mut api, m_sv, Some(sv_picker), &col_loc, &col_norm);
        let sv_post_rng_crc = api.ctx.world.rng.get_game_logic_random_seed_crc();
        let sv_dest = is_destroyed_i32(&api, sv_crate);
        let sv_armor_mask = test_salvage_armor_mask(&api, sv_picker);
        let sv_wpn_mask = {
            let wf = &api.object(sv_picker).unwrap().weapon_set_flags;
            u32::from(wf.test(WeaponSetType::CrateUpgradeOne.as_i32()) != FALSE)
                | (u32::from(wf.test(WeaponSetType::CrateUpgradeTwo.as_i32()) != FALSE) << 1)
        };
        let sv_vet_after = api
            .object(sv_picker)
            .unwrap()
            .experience_tracker
            .current_level
            .as_i32() as u32;
        let sv_exp_after = api
            .object(sv_picker)
            .unwrap()
            .experience_tracker
            .current_experience as u32;
        let sv_money_after = api.player_money(team_c);
        let (sv_crc, sv_xfer) = snapshot_mod(&mut api, m_sv);
        let sv_rt = snapshot_load_and_resave(&mut api, m_sv, &sv_xfer);
        let exp_sv = parse_i32_vec_as_u32(line, "sv");
        assert_eq!(
            vec![
                sv_is_sv as u32,
                sv_would as u32,
                sv_dest as u32,
                sv_armor_mask,
                sv_wpn_mask,
                sv_vet_after,
                sv_exp_after,
                sv_money_after,
                sv_post_rng_crc,
            ],
            exp_sv,
            "seed {seed} sv mismatch"
        );
        check_crc(sv_crc, get_u32(line, "sv_crc").expect("sv_crc"), "sv", seed);
        check_xfer(
            &sv_xfer,
            get_str(line, "sv_xfer").expect("sv_xfer"),
            &sv_rt,
            "sv",
            seed,
        );
        destroy_if_live(&mut api, sv_crate);
        destroy_if_live(&mut api, sv_picker);
        api.ctx.process_destroy_list();

        // 7. ShroudCrateCollide (PKT-241)
        let sh_pos = Coord3D { x: 70.0, y: 10.0, z: 0.0 };
        let sh_crate = spawn(&mut api, "Cr52_Mod_Shroud", team_b, sh_pos, 0.0);
        let sh_picker = spawn(&mut api, "Cr52_PlainCar", team_c, sh_pos, 0.0);
        let m_sh = first_collide_module(&api, sh_crate);
        let sh_would = collide_would(&mut api, m_sh, Some(sh_picker));
        collide_exec(&mut api, m_sh, Some(sh_picker), &col_loc, &col_norm);
        assert_eq!(
            api.ctx.world.partition.shroud_status(2, 0, 0),
            zh_logic::spatial::CellShroudStatus::Fogged,
            "seed {seed} ShroudCrateCollide should reveal map for picker's player"
        );
        let sh_dest = is_destroyed_i32(&api, sh_crate);
        let (sh_crc, sh_xfer) = snapshot_mod(&mut api, m_sh);
        let sh_rt = snapshot_load_and_resave(&mut api, m_sh, &sh_xfer);
        let exp_sh = get_i32_vec(line, "sh").expect("sh vec");
        assert_eq!(vec![sh_would, sh_dest], exp_sh, "seed {seed} sh mismatch");
        check_crc(sh_crc, get_u32(line, "sh_crc").expect("sh_crc"), "sh", seed);
        check_xfer(
            &sh_xfer,
            get_str(line, "sh_xfer").expect("sh_xfer"),
            &sh_rt,
            "sh",
            seed,
        );
        destroy_if_live(&mut api, sh_crate);
        destroy_if_live(&mut api, sh_picker);
        api.ctx.process_destroy_list();

        // 8. SquishCollide (PKT-246)
        let sq_self_pos = Coord3D {
            x: 100.0,
            y: 100.0,
            z: 0.0,
        };
        let sq_crusher_pos = if sq_dist == 0 {
            Coord3D { x: 92.0, y: 96.0, z: 0.0 }
        } else {
            Coord3D { x: 50.0, y: 50.0, z: 0.0 }
        };
        let sq_self = spawn(
            &mut api,
            if sq_hjk {
                "Cr52_Mod_SquishHijacker"
            } else {
                "Cr52_Mod_Squish"
            },
            team_b,
            sq_self_pos,
            0.0,
        );
        if sq_hjk {
            attach_stub_hijacker(&mut api, sq_self);
        }
        if sq_def {
            api.object_mut(sq_self).unwrap().set_undetected_defector(TRUE);
        }
        let sq_cr_tmpl = match sq_kind {
            0 => "Cr52_PlainCar",
            1 => "Cr52_BoxTank",
            _ => "Cr52_NonCrusherVeh",
        };
        let sq_crusher = spawn(&mut api, sq_cr_tmpl, team_c, sq_crusher_pos, sq_ang);
        let sq_vel_vec = match sq_vel {
            0 => Coord3D { x: 8.0, y: 4.0, z: 0.0 },
            1 => Coord3D { x: -8.0, y: -4.0, z: 0.0 },
            _ => Coord3D::default(),
        };
        add_velocity(&mut api, sq_crusher, sq_vel_vec);
        let sq_decoy = spawn(
            &mut api,
            "Cr52_Decoy",
            team_c,
            Coord3D { x: 120.0, y: 100.0, z: 0.0 },
            0.0,
        );
        api.object_mut(sq_self).unwrap().ai_goal_object =
            Some(if sq_hjk { sq_crusher } else { sq_decoy });
        let m_sq = first_collide_module(&api, sq_self);
        let sq_arg = if sq_gnd { None } else { Some(sq_crusher) };
        collide_exec(&mut api, m_sq, sq_arg, &col_loc, &col_norm);
        let sq_dead = i32::from(api.object(sq_self).unwrap().is_effectively_dead() != FALSE);
        let sq_hp = body_health(&api, sq_self).to_bits();
        let sq_is_def = i32::from(api.object(sq_self).unwrap().is_undetected_defector() != FALSE);
        let (sq_crc, sq_xfer) = snapshot_mod(&mut api, m_sq);
        let sq_rt = snapshot_load_and_resave(&mut api, m_sq, &sq_xfer);
        let exp_sq = parse_i32_vec_as_u32(line, "sq");
        assert_eq!(
            vec![sq_dead as u32, sq_hp, sq_is_def as u32],
            exp_sq,
            "seed {seed} sq mismatch"
        );
        check_crc(sq_crc, get_u32(line, "sq_crc").expect("sq_crc"), "sq", seed);
        check_xfer(
            &sq_xfer,
            get_str(line, "sq_xfer").expect("sq_xfer"),
            &sq_rt,
            "sq",
            seed,
        );
        destroy_if_live(&mut api, sq_self);
        destroy_if_live(&mut api, sq_crusher);
        destroy_if_live(&mut api, sq_decoy);
        api.ctx.process_destroy_list();

        // 9. UnitCrateCollide (PKT-250)
        let un_pos = Coord3D {
            x: 200.0,
            y: 200.0,
            z: 0.0,
        };
        let un_crate = spawn(&mut api, &t_unit, team_b, un_pos, 0.0);
        let un_picker = spawn(&mut api, "Cr52_PlainCar", team_c, un_pos, 0.75);
        let un_before: BTreeSet<ObjectSlot> = api.world_object_slots().into_iter().collect();
        let m_un = first_collide_module(&api, un_crate);
        let un_would = collide_would(&mut api, m_un, Some(un_picker));
        api.ctx.world.rng.init_random_with_seed(un_rng);
        collide_exec(&mut api, m_un, Some(un_picker), &col_loc, &col_norm);
        let un_dest = is_destroyed_i32(&api, un_crate);
        let un_created: Vec<ObjectSlot> = api
            .world_object_slots()
            .into_iter()
            .filter(|s| {
                !un_before.contains(s) && api.object(*s).is_some_and(|o| !o.is_destroyed())
            })
            .collect();
        // Advance 5 GameLogic PRNG steps per spawned Cr52_SpawnedUnit to match C++
        // `newObject` + unported `PartitionManager::findPositionAround` draws.
        for _ in 0..(un_created.len() * 5) {
            let _ = api.rng_logic_real(0.0, 1.0);
        }
        let un_post_rng_crc = api.ctx.world.rng.get_game_logic_random_seed_crc();
        let un_first_pos = un_created
            .first()
            .and_then(|s| api.object(*s).map(|o| *o.thing.get_position()))
            .unwrap_or_default();
        let (un_crc, un_xfer) = snapshot_mod(&mut api, m_un);
        let un_rt = snapshot_load_and_resave(&mut api, m_un, &un_xfer);
        let exp_un = parse_i32_vec_as_u32(line, "un");
        assert_eq!(
            vec![
                un_would as u32,
                un_dest as u32,
                un_created.len() as u32,
                un_first_pos.x.to_bits(),
                un_first_pos.y.to_bits(),
                un_post_rng_crc,
            ],
            exp_un,
            "seed {seed} un mismatch"
        );
        check_crc(un_crc, get_u32(line, "un_crc").expect("un_crc"), "un", seed);
        check_xfer(
            &un_xfer,
            get_str(line, "un_xfer").expect("un_xfer"),
            &un_rt,
            "un",
            seed,
        );
        for o in un_created {
            destroy_if_live(&mut api, o);
        }
        destroy_if_live(&mut api, un_crate);
        destroy_if_live(&mut api, un_picker);
        api.ctx.process_destroy_list();

        // 10. VeterancyCrateCollide (PKT-254)
        let vt_pos = Coord3D {
            x: 300.0,
            y: 300.0,
            z: 0.0,
        };
        let vt_crate = spawn(&mut api, &t_vet, team_b, vt_pos, 0.0);
        api.set_veterancy_level(vt_crate, vet_level_from_i32(vt_ovet), FALSE, FALSE);
        let vt_decoy = spawn(
            &mut api,
            "Cr52_Decoy",
            team_c,
            Coord3D { x: 500.0, y: 500.0, z: 0.0 },
            0.0,
        );
        let vt_tgt = match vt_sel {
            0 => spawn(&mut api, "Cr52_TrainableVeh", team_b, vt_pos, 0.0),
            1 => spawn(&mut api, "Cr52_TrainableVeh", team_c, vt_pos, 0.0),
            2 => spawn(&mut api, "Cr52_AirUnit", team_b, vt_pos, 0.0),
            3 => spawn(&mut api, "Cr52_NonTrainableVeh", team_b, vt_pos, 0.0),
            4 => spawn(
                &mut api,
                "Cr52_TrainableVeh",
                team_b,
                Coord3D { x: 300.0, y: 300.0, z: 40.0 },
                0.0,
            ),
            _ => {
                let o = spawn(&mut api, "Cr52_TrainableVeh", team_b, vt_pos, 0.0);
                api.set_veterancy_level(o, VeterancyLevel::Heroic, FALSE, FALSE);
                o
            }
        };
        let vt_tgt_team = api.object(vt_tgt).unwrap().team_id;
        let vt_other_team = if vt_tgt_team == team_b { team_c } else { team_b };
        let vt_in = spawn(
            &mut api,
            "Cr52_TrainableVeh",
            vt_tgt_team,
            Coord3D { x: 350.0, y: 300.0, z: 0.0 },
            0.0,
        );
        let vt_masked = spawn(
            &mut api,
            "Cr52_TrainableVeh",
            vt_tgt_team,
            Coord3D { x: 340.0, y: 300.0, z: 0.0 },
            0.0,
        );
        api.set_status(vt_masked, ObjectStatusType::Masked);
        let vt_out = spawn(
            &mut api,
            "Cr52_TrainableVeh",
            vt_tgt_team,
            Coord3D { x: 600.0, y: 300.0, z: 0.0 },
            0.0,
        );
        let vt_enemy = spawn(
            &mut api,
            "Cr52_TrainableVeh",
            vt_other_team,
            Coord3D { x: 320.0, y: 300.0, z: 0.0 },
            0.0,
        );

        // The harness updates the partition here.
        api.ctx.update_partition();
        api.object_mut(vt_crate).unwrap().ai_goal_object =
            Some(if vt_goal { vt_tgt } else { vt_decoy });
        let m_vt = first_collide_module(&api, vt_crate);
        let vt_would = collide_would(&mut api, m_vt, Some(vt_tgt));
        collide_exec(&mut api, m_vt, Some(vt_tgt), &col_loc, &col_norm);
        let vt_dest = is_destroyed_i32(&api, vt_crate);
        let vt_tgt_vet = api.object(vt_tgt).unwrap().experience_tracker.current_level.as_i32();
        let vt_tgt_exp = api.object(vt_tgt).unwrap().experience_tracker.current_experience;
        let vt_in_vet = api.object(vt_in).unwrap().experience_tracker.current_level.as_i32();
        let vt_in_exp = api.object(vt_in).unwrap().experience_tracker.current_experience;
        let vt_masked_vet = api
            .object(vt_masked)
            .unwrap()
            .experience_tracker
            .current_level
            .as_i32();
        let vt_out_vet = api.object(vt_out).unwrap().experience_tracker.current_level.as_i32();
        let vt_enemy_vet = api
            .object(vt_enemy)
            .unwrap()
            .experience_tracker
            .current_level
            .as_i32();
        let (vt_crc, vt_xfer) = snapshot_mod(&mut api, m_vt);
        let vt_rt = snapshot_load_and_resave(&mut api, m_vt, &vt_xfer);
        let exp_vt = get_i32_vec(line, "vt").expect("vt vec");
        assert_eq!(
            vec![
                vt_would,
                vt_dest,
                vt_tgt_vet,
                vt_tgt_exp,
                vt_in_vet,
                vt_in_exp,
                vt_masked_vet,
                vt_out_vet,
                vt_enemy_vet,
            ],
            exp_vt,
            "seed {seed} vt mismatch"
        );
        check_crc(vt_crc, get_u32(line, "vt_crc").expect("vt_crc"), "vt", seed);
        check_xfer(
            &vt_xfer,
            get_str(line, "vt_xfer").expect("vt_xfer"),
            &vt_rt,
            "vt",
            seed,
        );
        for o in [vt_crate, vt_decoy, vt_tgt, vt_in, vt_masked, vt_out, vt_enemy] {
            destroy_if_live(&mut api, o);
        }
        api.ctx.process_destroy_list();
    }
}
