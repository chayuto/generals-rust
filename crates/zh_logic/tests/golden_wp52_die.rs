//! Gate V2 Differential Golden Vector Harness for `WP-5.2-A` `DieModule` hierarchy.

use std::fmt::Write as _;
use std::path::Path;

use zh_common::golden_json::{
    decode_hex_bytes, encode_hex_bytes, extract_vector_lines, get_i32, get_i32_array, get_str,
    get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, ObjectId, Real, Relationship, TeamId, VeterancyLevel, FALSE, TRUE,
};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};

use zh_logic::arena::{FacetMask, ModuleSlot, ObjectSlot};
use zh_logic::combat::damage::DamageInfo;
use zh_logic::rules::combat_types::{DamageType, DeathType};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceTextLookup;
use zh_logic::rules::special_power_types::SpecialPowerType;
use zh_logic::sim::{MockSimApi, SimApi, SimView};

struct NoopSciLookup;
impl ScienceTextLookup for NoopSciLookup {
    fn fetch_label(&mut self, _label: &str) -> String {
        String::new()
    }
}

fn load_rules_ini(api: &mut MockSimApi, text: &str) {
    let bytes = text.as_bytes();
    let mut r = IniReader::new("rules.ini", bytes, IniLoadType::Overwrite);
    r.load_blocks(None, |block_name, reader| {
        if block_name.eq_ignore_ascii_case("Upgrade") {
            let _kw = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
            api.ctx
                .rules
                .upgrade_center
                .parse_upgrade_definition(reader, &mut api.ctx.rules.name_keys)
        } else if block_name.eq_ignore_ascii_case("Science") {
            let _kw = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
            let mut noop = NoopSciLookup;
            api.ctx.rules.science_store.parse_science_definition(
                reader,
                &mut api.ctx.rules.name_keys,
                &mut noop,
            )?;
            Ok(())
        } else if block_name.eq_ignore_ascii_case("SpecialPower") {
            let _kw = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
            api.ctx.rules.special_power_store.parse_special_power_definition(
                reader,
                &api.ctx.rules.science_store,
                &mut api.ctx.rules.name_keys,
            )
        } else if block_name.eq_ignore_ascii_case("CrateData") {
            let _kw = reader.next_token(zh_common::ini::SEPS_DEFAULT)?;
            api.ctx.rules.crate_system.parse_crate_template_definition(
                reader,
                IniLoadType::Overwrite,
                &api.ctx.rules.science_store,
                &mut api.ctx.rules.name_keys,
            )
        } else {
            Err(zh_common::ini::IniError::UnknownToken)
        }
    })
    .expect("parse rules INI");
}

fn load_objects_only(api: &mut MockSimApi, text: &str) {
    let mut r_obj = IniReader::new("mem.ini", text.as_bytes(), IniLoadType::Overwrite);
    api.ctx
        .rules
        .load_thing_templates_from_ini(&mut r_obj)
        .expect("parse Object INI");
}

fn spawn(api: &mut MockSimApi, name: &str, team: TeamId, pos: Coord3D, angle: Real) -> ObjectSlot {
    let id = api
        .ctx
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    api.spawn_object(id, team, pos, angle)
}

fn first_die_module(api: &MockSimApi, slot: ObjectSlot) -> ModuleSlot {
    api.object(slot)
        .expect("object alive")
        .behaviors
        .iter()
        .find(|(_, mask)| mask.contains(FacetMask::DIE))
        .map(|&(ms, _)| ms)
        .expect("missing Die module")
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

fn get_str_vec<'a>(obj_line: &'a str, key: &str) -> Vec<&'a str> {
    let pattern = format!("\"{}\":", key);
    let Some(start) = obj_line.find(&pattern) else {
        return Vec::new();
    };
    let rest = obj_line[start + pattern.len()..].trim_start();
    if !rest.starts_with('[') {
        return Vec::new();
    }
    let Some(end) = rest.find(']') else {
        return Vec::new();
    };
    let inner = rest[1..end].trim();
    if inner.is_empty() {
        return Vec::new();
    }
    inner
        .split(',')
        .map(|part| part.trim().trim_matches('"'))
        .collect()
}

fn get_i32_vec(obj_line: &str, key: &str) -> Vec<i32> {
    zh_common::golden_json::get_i32_vec(obj_line, key).unwrap_or_default()
}

#[test]
fn wp52_die_golden_vectors() {
    let json_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/golden_vectors/WP52_Die.json");
    let json_text = std::fs::read_to_string(&json_path).expect("read WP52_Die.json");
    let lines = extract_vector_lines(&json_text);
    assert_eq!(lines.len(), 1000, "expected 1000 golden vector seeds");

    let mut api = MockSimApi::new();
    api.ctx.rules.fx_list_names.insert("FX_DieMain".to_owned());
    api.ctx.rules.fx_list_names.insert("FX_DieUpg".to_owned());
    api.ctx.rules.ocl_names.insert("OCL_DieTest".to_owned());
    api.ctx.rules.ocl_names.insert("OCL_EjectAir".to_owned());
    api.ctx.rules.ocl_names.insert("OCL_EjectGround".to_owned());

    // The neutral player's team: `TeamId(0)` is a null `Team*`, which has no player.
    let team_a = TeamId(1);
    api.ctx.world.team_players.insert(team_a, 0);
    api.ctx.world.neutral_team_id = team_a;
    let team_b = TeamId(2); // Active non-neutral
    api.ctx.world.team_is_neutral.insert(team_a, TRUE);
    api.ctx.world.team_is_neutral.insert(team_b, FALSE);
    api.ctx
        .world.set_player_relation(0, 2, Relationship::Enemies);
    api.ctx
        .world.set_player_relation(2, 0, Relationship::Enemies);

    let rules_ini = "Upgrade UpgDieA\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
         Upgrade UpgDieConflict\n  Type = PLAYER\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
         Upgrade UpgDieRemove\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
         Upgrade UpgDieDrone\n  Type = OBJECT\n  BuildTime = 1.0\n  BuildCost = 100\nEnd\n\
         Science SCI_WP52_DIE\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n\
         SpecialPower SpWP52_Die\n  Enum = SPECIAL_CASH_HACK\n  ReloadTime = 1000\nEnd\n\
         CrateData Crate_Basic\n\
           CreationChance = 0.50\n\
           CrateObject = WP52_CrateItemA 0.30\n\
           CrateObject = WP52_CrateItemB 0.50\n\
           OwnedByMaker = No\n\
         End\n\
         CrateData Crate_VetElite\n\
           CreationChance = 0.80\n\
           VeterancyLevel = ELITE\n\
           CrateObject = WP52_CrateItemB 1.0\n\
           OwnedByMaker = Yes\n\
         End\n\
         CrateData Crate_KillerInfSci\n\
           CreationChance = 0.90\n\
           KilledByType = INFANTRY\n\
           KillerScience = SCI_WP52_DIE\n\
           CrateObject = WP52_CrateItemC 1.0\n\
           OwnedByMaker = No\n\
         End\n";
    load_rules_ini(&mut api, rules_ini);

    let mut base_ini = String::from(
        "Object WP52_DieSpawned\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 80\n    InitialHealth = 80\n  End\nEnd\n\
         Object WP52_CrateItemA\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\nEnd\n\
         Object WP52_CrateItemB\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\nEnd\n\
         Object WP52_CrateItemC\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 10\n    InitialHealth = 10\n  End\nEnd\n\
         Object WP52_WaveGuide\n  KindOf = WAVEGUIDE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n\
         Object WP52_NonWave\n  KindOf = VEHICLE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n\
         Object WP52_KillerInf\n  KindOf = INFANTRY\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n\
         Object WP52_KillerVeh\n  KindOf = VEHICLE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\nEnd\n\
         Object WP52_RebuildHole\n\
           KindOf = STRUCTURE\n\
           Geometry = BOX\n\
           GeometryMajorRadius = 15.0\n\
           GeometryMinorRadius = 15.0\n\
           GeometryHeight = 10.0\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
         End\n\
         Object D52_Crate_0\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = CreateCrateDie ModuleTag_Die\n\
             CrateData = Crate_Basic\n\
           End\n\
         End\n\
         Object D52_Crate_1\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = CreateCrateDie ModuleTag_Die\n\
             CrateData = Crate_VetElite\n\
           End\n\
         End\n\
         Object D52_Crate_2\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = CreateCrateDie ModuleTag_Die\n\
             CrateData = Crate_KillerInfSci\n\
           End\n\
         End\n\
         Object D52_Crate_3\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = CreateCrateDie ModuleTag_Die\n\
             CrateData = Crate_VetElite\n\
             CrateData = Crate_Basic\n\
           End\n\
         End\n\
         Object D52_CreateObj\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = CreateObjectDie ModuleTag_Die\n\
             DeathTypes = NONE +NORMAL +EXPLODED\n\
             CreationList = OCL_DieTest\n\
             TransferPreviousHealth = No\n\
             TransferSelection = No\n\
           End\n\
         End\n\
         Object D52_Dam\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = DamDie ModuleTag_Die\n\
             DeathTypes = NONE +EXPLODED\n\
           End\n\
         End\n\
         Object D52_Eject\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           UnitSpecificSounds\n    VoiceEject = NoSound\n    SoundEject = NoSound\n  End\n\
           Behavior = EjectPilotDie ModuleTag_Die\n\
             AirCreationList = OCL_EjectAir\n\
             GroundCreationList = OCL_EjectGround\n\
             InvulnerableTime = 30\n\
           End\n\
         End\n\
         Object D52_SP\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = SpecialPowerCompletionDie ModuleTag_Die\n\
             SpecialPowerTemplate = SpWP52_Die\n\
           End\n\
         End\n\
         Object D52_UpgDie\n\
           Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n\
           Behavior = UpgradeDie ModuleTag_Die\n\
             UpgradeToRemove = UpgDieDrone\n\
           End\n\
         End\n\
         Object D52_FX_00\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = FXListDie ModuleTag_Die\n    StartsActive = No\n    DeathFX = FX_DieMain\n    OrientToObject = No\n    TriggeredBy = UpgDieA\n    ConflictsWith = UpgDieConflict\n    RemovesUpgrades = UpgDieRemove\n    FXListUpgrade = FX_DieUpg\n  End\nEnd\n\
         Object D52_FX_01\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = FXListDie ModuleTag_Die\n    StartsActive = No\n    DeathFX = FX_DieMain\n    OrientToObject = Yes\n    TriggeredBy = UpgDieA\n    ConflictsWith = UpgDieConflict\n    RemovesUpgrades = UpgDieRemove\n    FXListUpgrade = FX_DieUpg\n  End\nEnd\n\
         Object D52_FX_10\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = FXListDie ModuleTag_Die\n    StartsActive = Yes\n    DeathFX = FX_DieMain\n    OrientToObject = No\n    TriggeredBy = UpgDieA\n    ConflictsWith = UpgDieConflict\n    RemovesUpgrades = UpgDieRemove\n    FXListUpgrade = FX_DieUpg\n  End\nEnd\n\
         Object D52_FX_11\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = FXListDie ModuleTag_Die\n    StartsActive = Yes\n    DeathFX = FX_DieMain\n    OrientToObject = Yes\n    TriggeredBy = UpgDieA\n    ConflictsWith = UpgDieConflict\n    RemovesUpgrades = UpgDieRemove\n    FXListUpgrade = FX_DieUpg\n  End\nEnd\n\
         Object D52_Crush_0\n  Geometry = BOX\n  GeometryMajorRadius = 10.0\n  GeometryMinorRadius = 5.0\n  GeometryHeight = 5.0\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = CrushDie ModuleTag_Die\n    TotalCrushSound = NoSound\n    BackEndCrushSound = NoSound\n    FrontEndCrushSound = NoSound\n  End\nEnd\n\
         Object D52_Crush_1\n  Geometry = BOX\n  GeometryMajorRadius = 10.0\n  GeometryMinorRadius = 5.0\n  GeometryHeight = 5.0\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = CrushDie ModuleTag_Die\n    TotalCrushSound = UnitCrushSound\n    BackEndCrushSound = UnitCrushSound\n    FrontEndCrushSound = UnitCrushSound\n    TotalCrushSoundPercent = 50\n    BackEndCrushSoundPercent = 75\n    FrontEndCrushSoundPercent = 25\n  End\nEnd\n",
    );

    let death_filters = ["ALL", "NONE +NORMAL +EXPLODED", "ALL -CRUSHED -LASERED"];
    let vet_filters = ["ALL", "NONE +VETERAN +HEROIC", "ALL -REGULAR"];
    let exempt_filters = ["NONE", "UNDER_CONSTRUCTION", "STEALTHED"];
    let req_filters = ["NONE", "IMMOBILE", "STEALTHED"];
    for (df, df_s) in death_filters.iter().enumerate() {
        for (vf, vf_s) in vet_filters.iter().enumerate() {
            for (ef, ef_s) in exempt_filters.iter().enumerate() {
                for (rf, rf_s) in req_filters.iter().enumerate() {
                    let _ = write!(
                        base_ini,
                        "Object D52_Keep_{df}{vf}{ef}{rf}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = KeepObjectDie ModuleTag_Die\n    DeathTypes = {df_s}\n    VeterancyLevels = {vf_s}\n    ExemptStatus = {ef_s}\n    RequiredStatus = {rf_s}\n  End\nEnd\n\
                         Object D52_Dest_{df}{vf}{ef}{rf}\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End\n  Behavior = DestroyDie ModuleTag_Die\n    DeathTypes = {df_s}\n    VeterancyLevels = {vf_s}\n    ExemptStatus = {ef_s}\n    RequiredStatus = {rf_s}\n  End\nEnd\n"
                    );
                }
            }
        }
    }
    load_objects_only(&mut api, &base_ini);

    let death_types = [
        DeathType::Normal,
        DeathType::Crushed,
        DeathType::Exploded,
        DeathType::Lasered,
        DeathType::Burned,
    ];
    let vet_levels = [
        VeterancyLevel::Regular,
        VeterancyLevel::Veteran,
        VeterancyLevel::Elite,
        VeterancyLevel::Heroic,
    ];

    let mut r_nk = std::mem::take(&mut api.ctx.rules.name_keys);
    let sci_die = api
        .ctx
        .rules
        .science_store
        .friend_lookup_science("SCI_WP52_DIE", &mut r_nk)
        .expect("SCI_WP52_DIE");
    api.ctx.rules.name_keys = r_nk;
    let upg_conf_mask = api
        .ctx
        .rules
        .upgrade_center
        .lookup_upgrade("UpgDieConflict", &api.ctx.rules.name_keys)
        .expect("UpgDieConflict")
        .upgrade_mask()
        .clone();
    let upg_rem_mask = api
        .ctx
        .rules
        .upgrade_center
        .lookup_upgrade("UpgDieRemove", &api.ctx.rules.name_keys)
        .expect("UpgDieRemove")
        .upgrade_mask()
        .clone();
    let upg_drone_mask = api
        .ctx
        .rules
        .upgrade_center
        .lookup_upgrade("UpgDieDrone", &api.ctx.rules.name_keys)
        .expect("UpgDieDrone")
        .upgrade_mask()
        .clone();

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let frame0 = get_u32(line, "frame").expect("frame");
        api.ctx.world.frame = frame0;

    for p in 0..16 {
        api.ctx.world.players.player_mut(p).expect("player").upgrades_completed.clear();
    }
        for p in 0..16 {
            let player = api.ctx.world.players.player_mut(p).expect("player");
            player.sciences.clear();
        }

        let df = get_i32(line, "df").unwrap();
        let vf = get_i32(line, "vf").unwrap();
        let ef = get_i32(line, "ef").unwrap();
        let rf = get_i32(line, "rf").unwrap();
        let obj_vet_idx = get_i32(line, "obj_vet").unwrap() as usize;
        let obj_status_bits = get_u32(line, "obj_status").unwrap();
        let dmg_death_idx = get_i32(line, "dmg_death").unwrap() as usize;

        let fx_starts_active = get_i32(line, "fx_starts_active").unwrap() != 0;
        let fx_orient = get_i32(line, "fx_orient").unwrap() != 0;
        let fx_give_upg = get_i32(line, "fx_give_upg").unwrap() != 0;
        let fx_conf_obj = get_i32(line, "fx_conf_obj").unwrap() != 0;
        let fx_conf_ply = get_i32(line, "fx_conf_ply").unwrap() != 0;

        let crate_killer_mode = get_i32(line, "crate_killer_mode").unwrap();
        let crate_vet_idx = get_i32(line, "crate_vet").unwrap() as usize;
        let crate_tmpl_idx = get_i32(line, "crate_tmpl").unwrap();
        let crate_rng_seed = get_u32(line, "crate_rng_seed").unwrap();

        let crush_is_crush = get_i32(line, "crush_is_crush").unwrap() != 0;
        let crush_has_dealer = get_i32(line, "crush_has_dealer").unwrap() != 0;
        let crush_pre_front = get_i32(line, "crush_pre_front").unwrap() != 0;
        let crush_pre_back = get_i32(line, "crush_pre_back").unwrap() != 0;
        let crush_named_sound = get_i32(line, "crush_named_sound").unwrap() != 0;
        let crush_angle = f32::from_bits(get_u32(line, "crush_angle_bits").unwrap());
        let dealer_dx = f32::from_bits(get_u32(line, "dealer_dx_bits").unwrap());
        let dealer_dy = f32::from_bits(get_u32(line, "dealer_dy_bits").unwrap());
        let crush_rng_seed = get_u32(line, "crush_rng_seed").unwrap();

        let eject_air = get_i32(line, "eject_air").unwrap() != 0;

        let rh_neutral = get_i32(line, "rh_neutral").unwrap() != 0;
        let rh_under_const = get_i32(line, "rh_under_const").unwrap() != 0;
        let rh_max_hp = f32::from_bits(get_u32(line, "rh_max_hp_bits").unwrap());
        let rh_major_rad = f32::from_bits(get_u32(line, "rh_major_rad_bits").unwrap());
        let rh_transfer_atk = if get_i32(line, "rh_transfer_atk").unwrap() != 0 {
            "Yes"
        } else {
            "No"
        };

        let sp_set_creator = get_i32(line, "sp_set_creator").unwrap() != 0;
        let sp_creator_id = get_i32(line, "sp_creator_id").unwrap();
        let ud_has_producer = get_i32(line, "ud_has_producer").unwrap() != 0;
        let ud_has_upgrade = get_i32(line, "ud_has_upgrade").unwrap() != 0;

        let rh_tmpl = format!("D52_RH_{seed}");
        let rh_ini = format!(
            "Object {rh_tmpl}\n  KindOf = STRUCTURE\n  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 500\n    InitialHealth = 500\n  End\n  Behavior = RebuildHoleExposeDie ModuleTag_Die\n    HoleName = WP52_RebuildHole\n    HoleMaxHealth = {rh_max_hp}\n    TransferAttackers = {rh_transfer_atk}\n  End\nEnd\n"
        );
        load_objects_only(&mut api, &rh_ini);

        let mut dmg = DamageInfo::default();
        dmg.input.damage_type = DamageType::Explosion;
        dmg.input.death_type = death_types[dmg_death_idx];
        dmg.input.amount = 1000.0;
        dmg.input.source_id = ObjectId::INVALID;

        // 1) DieMuxData + KeepObjectDie + DestroyDie
        let keep_name = format!("D52_Keep_{df}{vf}{ef}{rf}");
        let dest_name = format!("D52_Dest_{df}{vf}{ef}{rf}");
        let o_keep = spawn(&mut api, &keep_name, team_b, Coord3D::default(), 0.0);
        let o_dest = spawn(&mut api, &dest_name, team_b, Coord3D::default(), 0.0);
        for slot in [o_keep, o_dest] {
            api.set_veterancy_level(slot, vet_levels[obj_vet_idx], FALSE, FALSE);
            if (obj_status_bits & 1) != 0 {
                api.set_status(slot, ObjectStatusType::UnderConstruction);
            }
            if (obj_status_bits & 2) != 0 {
                api.set_status(slot, ObjectStatusType::Stealthed);
            }
            if (obj_status_bits & 4) != 0 {
                api.set_status(slot, ObjectStatusType::Immobile);
            }
        }
        api.dispatch_on_die(o_keep, &dmg);
        api.dispatch_on_die(o_dest, &dmg);

        let keep_dest = api.object(o_keep).is_none_or(|o| o.is_destroyed());
        let dest_dest = api.object(o_dest).is_none_or(|o| o.is_destroyed());
        assert_eq!(
            i32::from(keep_dest),
            get_i32(line, "keep_dest").unwrap(),
            "seed {seed} keep_dest"
        );
        assert_eq!(
            i32::from(dest_dest),
            get_i32(line, "dest_dest").unwrap(),
            "seed {seed} dest_dest"
        );

        let keep_mod = first_die_module(&api, o_keep);
        let (keep_crc, keep_hex) = snapshot_mod(&mut api, keep_mod);
        check_crc(keep_crc, get_u32(line, "keep_crc").unwrap(), "keep", seed);
        assert_eq!(keep_hex, get_str(line, "keep_xfer").unwrap(), "seed {seed} keep_xfer");

        let dest_mod = first_die_module(&api, o_dest);
        let (dest_crc, dest_hex) = snapshot_mod(&mut api, dest_mod);
        check_crc(dest_crc, get_u32(line, "dest_crc").unwrap(), "dest", seed);
        assert_eq!(dest_hex, get_str(line, "dest_xfer").unwrap(), "seed {seed} dest_xfer");

        // 2) FXListDie
        let fx_name = format!(
            "D52_FX_{}{}",
            u8::from(fx_starts_active),
            u8::from(fx_orient)
        );
        let o_fx = spawn(&mut api, &fx_name, team_b, Coord3D::new(15.0, 25.0, 0.0), 0.0);
        api.grant_upgrade(o_fx, "UpgDieRemove", FALSE);
        let fx_rem_pre = api.object_has_upgrade(o_fx, &upg_rem_mask) != FALSE;
        let fx_mod = first_die_module(&api, o_fx);
        let fx_active_init = api
            .ctx
            .dispatch_module(fx_mod, |m, _, _| {
                m.as_upgrade().unwrap().is_already_upgraded() != FALSE
            })
            .unwrap();
        if fx_give_upg {
            api.grant_upgrade(o_fx, "UpgDieA", FALSE);
        }
        let fx_active_after = api
            .ctx
            .dispatch_module(fx_mod, |m, _, _| {
                m.as_upgrade().unwrap().is_already_upgraded() != FALSE
            })
            .unwrap();
        let fx_rem_after = api.object_has_upgrade(o_fx, &upg_rem_mask) != FALSE;
        if fx_conf_obj {
            api.grant_upgrade(o_fx, "UpgDieConflict", FALSE);
        }
        if fx_conf_ply {
            let player_b = api.ctx.player_index(team_b);
            api.ctx
                .world.players.player_mut(player_b).expect("player").upgrades_completed
                .set_flags(&upg_conf_mask);
        }
        api.ctx.world.triggered_fx_lists.clear();
        let mut fx_dmg = DamageInfo::default();
        fx_dmg.input.damage_type = DamageType::Explosion;
        fx_dmg.input.death_type = DeathType::Normal;
        fx_dmg.input.amount = 50.0;
        api.dispatch_on_die(o_fx, &fx_dmg);
        let fx_fired = api
            .ctx
            .world
            .triggered_fx_lists
            .iter()
            .any(|(n, _)| n == "FX_DieMain");
        assert_eq!(
            [
                i32::from(fx_active_init),
                i32::from(fx_active_after),
                i32::from(fx_rem_pre),
                i32::from(fx_rem_after),
                i32::from(fx_fired),
            ],
            get_i32_array::<5>(line, "fx").unwrap(),
            "seed {seed} fx"
        );
        let (fx_crc, fx_hex) = snapshot_mod(&mut api, fx_mod);
        check_crc(fx_crc, get_u32(line, "fx_crc").unwrap(), "fx", seed);
        assert_eq!(fx_hex, get_str(line, "fx_xfer").unwrap(), "seed {seed} fx_xfer");

        // 3) CreateObjectDie
        api.ctx.world.spawned_ocls.clear();
        let o_co = spawn(
            &mut api,
            "D52_CreateObj",
            team_b,
            Coord3D::new(30.0, 40.0, 0.0),
            0.0,
        );
        api.dispatch_on_die(o_co, &dmg);
        let co_spawned = i32::from(
            api.ctx
                .world
                .spawned_ocls
                .iter()
                .any(|(n, _, _, _)| n == "OCL_DieTest"),
        );
        assert_eq!(co_spawned, get_i32(line, "co_spawned").unwrap(), "seed {seed} co_spawned");
        let co_mod = first_die_module(&api, o_co);
        let (co_crc, co_hex) = snapshot_mod(&mut api, co_mod);
        check_crc(co_crc, get_u32(line, "co_crc").unwrap(), "co", seed);
        assert_eq!(co_hex, get_str(line, "co_xfer").unwrap(), "seed {seed} co_xfer");

        // 4) EjectPilotDie
        api.ctx.world.spawned_ocls.clear();
        let ej_z = if eject_air { 50.0 } else { 0.0 };
        let o_ej = spawn(
            &mut api,
            "D52_Eject",
            team_b,
            Coord3D::new(10.0, 20.0, ej_z),
            0.0,
        );
        api.dispatch_on_die(o_ej, &fx_dmg);
        let expected_ej_ocl = match get_str(line, "ej_spawned").unwrap() {
            "WP52_CrateItemA" => "OCL_EjectAir",
            "WP52_CrateItemB" => "OCL_EjectGround",
            other => panic!("unexpected ej_spawned {other}"),
        };
        assert!(
            api.ctx
                .world
                .spawned_ocls
                .iter()
                .any(|(n, _, _, _)| n == expected_ej_ocl),
            "seed {seed} ej_spawned expected {expected_ej_ocl}, got {:?}",
            api.ctx.world.spawned_ocls
        );
        let ej_mod = first_die_module(&api, o_ej);
        let (ej_crc, ej_hex) = snapshot_mod(&mut api, ej_mod);
        check_crc(ej_crc, get_u32(line, "ej_crc").unwrap(), "ej", seed);
        assert_eq!(ej_hex, get_str(line, "ej_xfer").unwrap(), "seed {seed} ej_xfer");

        // 5) CreateCrateDie
        api.ctx.world.crate_spawns.clear();
        let crate_tmpl_name = format!("D52_Crate_{crate_tmpl_idx}");
        let o_cr = spawn(
            &mut api,
            &crate_tmpl_name,
            team_b,
            Coord3D::new(60.0, 70.0, 0.0),
            0.0,
        );
        api.set_veterancy_level(o_cr, vet_levels[crate_vet_idx], FALSE, FALSE);
        let o_killer = match crate_killer_mode {
            1 => Some(spawn(
                &mut api,
                "WP52_KillerInf",
                team_b,
                Coord3D::new(62.0, 70.0, 0.0),
                0.0,
            )),
            2 => Some(spawn(
                &mut api,
                "WP52_KillerInf",
                team_a,
                Coord3D::new(62.0, 70.0, 0.0),
                0.0,
            )),
            3 => {
                api.ctx
                    .player_of_team_mut(team_a)
                    .expect("player")
                    .push_science(sci_die);
                Some(spawn(
                    &mut api,
                    "WP52_KillerInf",
                    team_a,
                    Coord3D::new(62.0, 70.0, 0.0),
                    0.0,
                ))
            }
            4 => {
                api.ctx
                    .player_of_team_mut(team_a)
                    .expect("player")
                    .push_science(sci_die);
                Some(spawn(
                    &mut api,
                    "WP52_KillerVeh",
                    team_a,
                    Coord3D::new(62.0, 70.0, 0.0),
                    0.0,
                ))
            }
            _ => None,
        };
        let mut cr_dmg = fx_dmg.clone();
        cr_dmg.input.source_id = o_killer
            .and_then(|s| api.ctx.world.objects.id_of(s))
            .unwrap_or(ObjectId::INVALID);
        api.ctx.world.rng.init_random_with_seed(crate_rng_seed);
        api.dispatch_on_die(o_cr, &cr_dmg);
        let got_crates: Vec<&str> = api
            .ctx
            .world
            .crate_spawns
            .iter()
            .map(|(n, _, _)| n.as_str())
            .collect();
        let exp_crates = get_str_vec(line, "crates");
        assert_eq!(got_crates, exp_crates, "seed {seed} crates");
        // Advance 1 GameLogicRandomValueReal per spawned crate to match C++ orientation roll.
        for _ in 0..got_crates.len() {
            let _ = api.rng_logic_real(0.0, 6.2831855);
        }
        assert_eq!(
            api.ctx.world.rng.get_game_logic_random_seed_crc(),
            get_u32(line, "crate_post_rng_crc").unwrap(),
            "seed {seed} crate_post_rng_crc"
        );
        let cr_mod = first_die_module(&api, o_cr);
        let (cr_crc, cr_hex) = snapshot_mod(&mut api, cr_mod);
        check_crc(cr_crc, get_u32(line, "cr_crc").unwrap(), "cr", seed);
        assert_eq!(cr_hex, get_str(line, "cr_xfer").unwrap(), "seed {seed} cr_xfer");

        // 6) CrushDie
        let crush_tmpl = if crush_named_sound {
            "D52_Crush_1"
        } else {
            "D52_Crush_0"
        };
        let o_crush = spawn(
            &mut api,
            crush_tmpl,
            team_b,
            Coord3D::new(100.0, 100.0, 0.0),
            crush_angle,
        );
        api.set_crushed(
            o_crush,
            Some(Bool::from(crush_pre_front)),
            Some(Bool::from(crush_pre_back)),
        );
        let o_crusher = if crush_has_dealer {
            Some(spawn(
                &mut api,
                "WP52_KillerVeh",
                team_a,
                Coord3D::new(100.0 + dealer_dx, 100.0 + dealer_dy, 0.0),
                0.0,
            ))
        } else {
            None
        };
        let mut crush_dmg = DamageInfo::default();
        crush_dmg.input.damage_type = if crush_is_crush {
            DamageType::Crush
        } else {
            DamageType::Explosion
        };
        crush_dmg.input.death_type = DeathType::Crushed;
        crush_dmg.input.amount = 50.0;
        crush_dmg.input.source_id = o_crusher
            .and_then(|s| api.ctx.world.objects.id_of(s))
            .unwrap_or(ObjectId::INVALID);

        api.ctx.world.rng.init_random_with_seed(crush_rng_seed);
        api.dispatch_on_die(o_crush, &crush_dmg);
        let (crush_post_front, crush_post_back) = {
            let bs = api.object(o_crush).unwrap().body_slot.unwrap();
            match api.module_sidecar(bs) {
                zh_logic::arena::ModuleSidecar::Body {
                    front_crushed,
                    back_crushed,
                    ..
                } => (front_crushed != FALSE, back_crushed != FALSE),
                _ => (false, false),
            }
        };
        let crush_mc_front = api
            .object(o_crush)
            .unwrap()
            .model_condition
            .test(ModelConditionFlagType::FrontCrushed.as_i32())
            != FALSE;
        let crush_mc_back = api
            .object(o_crush)
            .unwrap()
            .model_condition
            .test(ModelConditionFlagType::BackCrushed.as_i32())
            != FALSE;
        assert_eq!(
            [
                i32::from(crush_post_front),
                i32::from(crush_post_back),
                i32::from(crush_mc_front),
                i32::from(crush_mc_back),
            ],
            get_i32_array::<4>(line, "crush").unwrap(),
            "seed {seed} crush"
        );
        assert_eq!(
            api.ctx.world.rng.get_game_logic_random_seed_crc(),
            get_u32(line, "crush_post_rng_crc").unwrap(),
            "seed {seed} crush_post_rng_crc"
        );
        let crush_mod = first_die_module(&api, o_crush);
        let (crush_crc, crush_hex) = snapshot_mod(&mut api, crush_mod);
        check_crc(crush_crc, get_u32(line, "crush_crc").unwrap(), "crush", seed);
        assert_eq!(
            crush_hex,
            get_str(line, "crush_xfer").unwrap(),
            "seed {seed} crush_xfer"
        );

        // 7) DamDie
        api.ctx.world.dam_break_events.clear();
        let o_wg1 = spawn(&mut api, "WP52_WaveGuide", team_a, Coord3D::default(), 0.0);
        let o_wg2 = spawn(&mut api, "WP52_WaveGuide", team_b, Coord3D::default(), 0.0);
        let o_nw = spawn(&mut api, "WP52_NonWave", team_b, Coord3D::default(), 0.0);
        api.set_disabled(o_wg1, DisabledType::Default);
        api.set_disabled(o_wg2, DisabledType::Default);
        api.set_disabled(o_nw, DisabledType::Default);
        let o_dam = spawn(&mut api, "D52_Dam", team_b, Coord3D::default(), 0.0);
        api.dispatch_on_die(o_dam, &dmg);
        let wg1_dis = api
            .object(o_wg1)
            .unwrap()
            .is_disabled_by_type(DisabledType::Default)
            != FALSE;
        let wg2_dis = api
            .object(o_wg2)
            .unwrap()
            .is_disabled_by_type(DisabledType::Default)
            != FALSE;
        let nw_dis = api
            .object(o_nw)
            .unwrap()
            .is_disabled_by_type(DisabledType::Default)
            != FALSE;
        assert_eq!(
            [i32::from(wg1_dis), i32::from(wg2_dis), i32::from(nw_dis)],
            get_i32_array::<3>(line, "dam").unwrap(),
            "seed {seed} dam"
        );
        assert_eq!(
            !api.ctx.world.dam_break_events.is_empty(),
            !wg1_dis,
            "seed {seed} dam_break_events"
        );
        let dam_mod = first_die_module(&api, o_dam);
        let (dam_crc, dam_hex) = snapshot_mod(&mut api, dam_mod);
        check_crc(dam_crc, get_u32(line, "dam_crc").unwrap(), "dam", seed);
        assert_eq!(dam_hex, get_str(line, "dam_xfer").unwrap(), "seed {seed} dam_xfer");

        // 8) RebuildHoleExposeDie
        api.ctx.world.rebuild_hole_events.clear();
        let rh_team = if rh_neutral { team_a } else { team_b };
        let o_rh = spawn(
            &mut api,
            &rh_tmpl,
            rh_team,
            Coord3D::new(200.0, 200.0, 0.0),
            0.5,
        );
        let mut rh_geom = api.object(o_rh).unwrap().geometry.clone();
        rh_geom.set_major_radius(rh_major_rad);
        api.set_geometry_info(o_rh, &rh_geom);
        if rh_under_const {
            api.set_status(o_rh, ObjectStatusType::UnderConstruction);
        }
        api.dispatch_on_die(o_rh, &fx_dmg);
        let rh_spawned = !api.ctx.world.rebuild_hole_events.is_empty();
        assert_eq!(
            i32::from(rh_spawned),
            get_i32(line, "rh_spawned").unwrap(),
            "seed {seed} rh_spawned"
        );
        if rh_spawned {
            let (_, hole_id, _) = api.ctx.world.rebuild_hole_events[0];
            let hole_slot = api.find_object(hole_id).expect("hole spawned");
            let hole_rad = api.object(hole_slot).unwrap().geometry.get_major_radius();
            let hole_max = api.object_max_health(hole_slot);
            let hole_bs = api.object(hole_slot).unwrap().body_slot.unwrap();
            let hole_hp = api
                .ctx
                .dispatch_module(hole_bs, |m, _, _| m.as_body().unwrap().get_health())
                .unwrap();
            assert_eq!(
                hole_max.to_bits(),
                get_u32(line, "rh_hole_max_hp_bits").unwrap(),
                "seed {seed} rh_hole_max_hp"
            );
            assert_eq!(
                hole_hp.to_bits(),
                get_u32(line, "rh_hole_hp_bits").unwrap(),
                "seed {seed} rh_hole_hp"
            );
            assert_eq!(
                hole_rad.to_bits(),
                get_u32(line, "rh_hole_rad_bits").unwrap(),
                "seed {seed} rh_hole_rad"
            );
        }
        let rh_mod = first_die_module(&api, o_rh);
        let (rh_crc, rh_hex) = snapshot_mod(&mut api, rh_mod);
        check_crc(rh_crc, get_u32(line, "rh_crc").unwrap(), "rh", seed);
        assert_eq!(rh_hex, get_str(line, "rh_xfer").unwrap(), "seed {seed} rh_xfer");

        // 9) SpecialPowerCompletionDie
        api.ctx.world.special_power_completed_events.clear();
        let o_sp = spawn(&mut api, "D52_SP", team_b, Coord3D::default(), 0.0);
        let sp_mod = first_die_module(&api, o_sp);
        if sp_set_creator {
            api.ctx
                .dispatch_module(sp_mod, |m, _, _| {
                    let d = m.as_die_mut().unwrap();
                    d.set_creator(ObjectId(sp_creator_id));
                    d.set_creator(ObjectId(9999));
                })
                .unwrap();
        }
        let (sp_crc, sp_hex) = snapshot_mod(&mut api, sp_mod);
        check_crc(sp_crc, get_u32(line, "sp_crc").unwrap(), "sp", seed);
        assert_eq!(sp_hex, get_str(line, "sp_xfer").unwrap(), "seed {seed} sp_xfer");

        let o_sp_rt = spawn(&mut api, "D52_SP", team_b, Coord3D::default(), 0.0);
        let sp_rt_mod = first_die_module(&api, o_sp_rt);
        let bytes = decode_hex_bytes(&sp_hex).expect("sp hex");
        api.ctx
            .dispatch_module(sp_rt_mod, |m, sim, _| {
                let mut xl = XferLoad::new(&bytes);
                m.xfer(&mut xl, sim).expect("sp load");
            })
            .unwrap();
        let (_, sp_rt_hex) = snapshot_mod(&mut api, sp_rt_mod);
        assert_eq!(
            sp_rt_hex,
            get_str(line, "sp_rt_xfer").unwrap(),
            "seed {seed} sp_rt_xfer"
        );

        api.dispatch_on_die(o_sp, &fx_dmg);
        if sp_set_creator {
            assert_eq!(
                api.ctx.world.special_power_completed_events,
                vec![(ObjectId(sp_creator_id), SpecialPowerType::CashHack)],
                "seed {seed} sp_completed"
            );
        } else {
            assert!(
                api.ctx.world.special_power_completed_events.is_empty(),
                "seed {seed} sp_completed empty"
            );
        }
        let (_, sp_post_hex) = snapshot_mod(&mut api, sp_mod);
        assert_eq!(
            sp_post_hex,
            get_str(line, "sp_post_xfer").unwrap(),
            "seed {seed} sp_post_xfer"
        );

        // 10) UpgradeDie
        let o_prod = if ud_has_producer {
            let p = spawn(&mut api, "WP52_NonWave", team_b, Coord3D::default(), 0.0);
            if ud_has_upgrade {
                api.grant_upgrade(p, "UpgDieDrone", FALSE);
            }
            Some(p)
        } else {
            None
        };
        let ud_prod_pre = o_prod.is_some_and(|p| api.object_has_upgrade(p, &upg_drone_mask) != FALSE);
        let o_ud = spawn(&mut api, "D52_UpgDie", team_b, Coord3D::default(), 0.0);
        if let Some(p) = o_prod {
            let pid = api.ctx.world.objects.id_of(p).unwrap();
            api.ctx.world.objects.get_mut(o_ud).unwrap().producer_id = pid;
        }
        api.dispatch_on_die(o_ud, &fx_dmg);
        let ud_prod_post = o_prod.is_some_and(|p| api.object_has_upgrade(p, &upg_drone_mask) != FALSE);
        assert_eq!(
            vec![i32::from(ud_prod_pre), i32::from(ud_prod_post)],
            get_i32_vec(line, "ud"),
            "seed {seed} ud"
        );
        let ud_mod = first_die_module(&api, o_ud);
        let (ud_crc, ud_hex) = snapshot_mod(&mut api, ud_mod);
        check_crc(ud_crc, get_u32(line, "ud_crc").unwrap(), "ud", seed);
        assert_eq!(ud_hex, get_str(line, "ud_xfer").unwrap(), "seed {seed} ud_xfer");

        // Clean up objects before next seed
        for s in api.world_object_slots() {
            api.ctx.destroy_object_now(s);
        }
        api.ctx.world.spawned_ocls.clear();
        api.ctx.world.triggered_fx_lists.clear();
        api.ctx.world.crate_spawns.clear();
        api.ctx.world.dam_break_events.clear();
        api.ctx.world.rebuild_hole_events.clear();
        api.ctx.world.special_power_completed_events.clear();
    }
}
