//! Gate V2 Differential Golden Vector Harness for `WP-5.2-D1` `CollideModule` & Sabotage crates.

use std::fmt::Write as _;
use std::path::Path;

use zh_common::golden_json::{
    decode_hex_bytes, encode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str,
    get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, ObjectId, Real, Relationship, TeamId, UnsignedInt, FALSE, TRUE,
};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};

use zh_logic::arena::{FacetMask, ModuleDataId, ModuleSlot, ObjectSlot};
use zh_logic::modules::module::{AiUpdateFacet, BehaviorModule};
use zh_logic::rules::disabled_types::DisabledType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::rules::science::ScienceTextLookup;
use zh_logic::scheduler::SleepyUpdatePhase;
use zh_logic::sim::{MockSimApi, SimApi, SimView};
use zh_common::types::NameKeyType;
use zh_common::xfer::{Xfer, XferError};

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
        if block_name.eq_ignore_ascii_case("Science") {
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

fn spawn(api: &mut MockSimApi, name: &str, team: TeamId, pos: Coord3D) -> ObjectSlot {
    let id = api
        .ctx
        .rules
        .thing_factory
        .find_template_id(name)
        .unwrap_or_else(|| panic!("missing template {name}"));
    api.spawn_object(id, team, pos, 0.0)
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

fn sp_ready_frames(api: &mut MockSimApi, slot: ObjectSlot) -> Vec<UnsignedInt> {
    let slots: Vec<ModuleSlot> = api
        .object(slot)
        .expect("object alive")
        .behaviors
        .iter()
        .filter(|(_, mask)| mask.contains(FacetMask::SPECIAL_POWER))
        .map(|&(ms, _)| ms)
        .collect();
    slots
        .into_iter()
        .map(|ms| {
            api.ctx
                .dispatch_module(ms, |m, sim, ctx| {
                    m.as_special_power()
                        .expect("sp facet")
                        .get_ready_frame(sim, ctx)
                })
                .expect("dispatch sp")
        })
        .collect()
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

fn collide_is_sabotage(api: &mut MockSimApi, ms: ModuleSlot) -> bool {
    api.ctx
        .dispatch_module(ms, |m, _sim, _ctx| {
            m.as_collide()
                .expect("collide facet")
                .is_sabotage_building_crate_collide()
                != FALSE
        })
        .expect("dispatch is_sab")
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

fn is_destroyed_i32(api: &MockSimApi, slot: ObjectSlot) -> i32 {
    i32::from(api.object(slot).map_or(true, |o| o.is_destroyed()))
}

fn is_hacked_i32(api: &MockSimApi, slot: ObjectSlot) -> i32 {
    i32::from(
        api.object(slot)
            .is_some_and(|o| o.is_disabled_by_type(DisabledType::Hacked) != FALSE),
    )
}

fn hacked_until(api: &MockSimApi, slot: ObjectSlot) -> UnsignedInt {
    api.object(slot)
        .map_or(0, |o| o.disabled_till_frame[DisabledType::Hacked.as_i32() as usize])
}

struct StubAiBehavior(NameKeyType);

impl AiUpdateFacet for StubAiBehavior {}

impl BehaviorModule for StubAiBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.0
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

fn attach_stub_ai(api: &mut MockSimApi, slot: ObjectSlot) {
    api.ctx.world.attach_module(
        slot,
        NameKeyType(902),
        ModuleDataId::INVALID,
        FacetMask::AI_UPDATE,
        SleepyUpdatePhase::Normal,
        Box::new(StubAiBehavior(NameKeyType(902))),
    );
}

fn spawn_sab_unit(
    api: &mut MockSimApi,
    tmpl: &str,
    team_b: TeamId,
    target: ObjectSlot,
    decoy: ObjectSlot,
    sab_goal_match: bool,
    col_loc: Coord3D,
) -> (ObjectSlot, ModuleSlot) {
    let s = spawn(api, tmpl, team_b, col_loc);
    attach_stub_ai(api, s);
    api.object_mut(s).unwrap().ai_goal_object = Some(if sab_goal_match { target } else { decoy });
    let ms = first_collide_module(api, s);
    (s, ms)
}

#[test]
fn wp52_collide_sabotage_golden_vectors() {
    let json_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/golden_vectors/WP52_CollideSabotage.json");
    let json_text = std::fs::read_to_string(&json_path).expect("read WP52_CollideSabotage.json");
    assert!(
        json_text.contains("\"seed0_diagnostics\""),
        "missing seed0_diagnostics section"
    );
    let lines = extract_vector_lines(&json_text);
    assert_eq!(lines.len(), 1000, "expected 1000 golden vector seeds");

    let mut api = MockSimApi::new();
    api.ctx
        .rules
        .fx_list_names
        .insert("FX_CollideExec".to_owned());
    api.ctx
        .rules
        .ocl_names
        .insert("OCL_DropzoneTest".to_owned());

    let team_a = TeamId(0); // Neutral
    let team_b = TeamId(1); // Player A (saboteurs/crates)
    let team_c = TeamId(2); // Player B (targets)
    api.ctx.world.team_is_neutral.insert(team_a, TRUE);
    api.ctx.world.team_is_neutral.insert(team_b, FALSE);
    api.ctx.world.team_is_neutral.insert(team_c, FALSE);
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

    load_rules_ini(
        &mut api,
        "Science SCI_WP52_COL\n\
           SciencePurchasePointCost = 1\n\
           IsGrantable = Yes\n\
         End\n\
         SpecialPower SpWP52_Col1\n\
           Enum = SPECIAL_CASH_HACK\n\
           ReloadTime = 1000\n\
         End\n\
         SpecialPower SpWP52_Col2\n\
           Enum = SPECIAL_SPY_SATELLITE\n\
           ReloadTime = 2000\n\
         End\n",
    );

    let sci_col = api
        .ctx
        .rules
        .science_store
        .friend_lookup_science("SCI_WP52_COL", &mut api.ctx.rules.name_keys)
        .expect("SCI_WP52_COL");

    load_objects_only(
        &mut api,
        "Object Col_DummyItem\n\
           KindOf = INFANTRY\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 50\n\
             InitialHealth = 50\n\
           End\n\
         End\n\
         Object Col_TargetParachute\n\
           KindOf = PARACHUTE INFANTRY\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 100\n\
             InitialHealth = 100\n\
           End\n\
           Behavior = AIUpdateInterface ModuleTag_AI\n\
           End\n\
         End\n\
         Object Col_TargetCmdCenter\n\
           KindOf = STRUCTURE COMMANDCENTER\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
           Behavior = CashHackSpecialPower ModuleTag_SP1\n\
             SpecialPowerTemplate = SpWP52_Col1\n\
             MoneyAmount = 500\n\
           End\n\
           Behavior = SpyVisionSpecialPower ModuleTag_SP2\n\
             SpecialPowerTemplate = SpWP52_Col2\n\
           End\n\
         End\n\
         Object Col_TargetFake\n\
           KindOf = STRUCTURE FS_FAKE\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 150\n\
             InitialHealth = 150\n\
           End\n\
         End\n\
         Object Col_TargetInternet\n\
           KindOf = STRUCTURE FS_INTERNET_CENTER\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
           Behavior = InternetHackContain ModuleTag_Contain\n\
             Slots = 4\n\
             AllowInsideKindOf = INFANTRY\n\
           End\n\
         End\n\
         Object Col_TargetBarracks\n\
           KindOf = STRUCTURE FS_BARRACKS\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
         End\n\
         Object Col_TargetWarfactory\n\
           KindOf = STRUCTURE FS_WARFACTORY\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
         End\n\
         Object Col_TargetAirfield\n\
           KindOf = STRUCTURE FS_AIRFIELD\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
         End\n\
         Object Col_TargetCarrier\n\
           KindOf = STRUCTURE FS_AIRFIELD AIRCRAFT_CARRIER\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
         End\n\
         Object Col_TargetPower\n\
           KindOf = STRUCTURE FS_POWER\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
         End\n\
         Object Col_TargetSuperweapon\n\
           KindOf = STRUCTURE FS_SUPERWEAPON\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 300\n\
             InitialHealth = 300\n\
           End\n\
           Behavior = CashHackSpecialPower ModuleTag_SP1\n\
             SpecialPowerTemplate = SpWP52_Col1\n\
             MoneyAmount = 500\n\
           End\n\
           Behavior = SpyVisionSpecialPower ModuleTag_SP2\n\
             SpecialPowerTemplate = SpWP52_Col2\n\
           End\n\
         End\n\
         Object Col_TargetStrategy\n\
           KindOf = STRUCTURE FS_STRATEGY_CENTER\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 300\n\
             InitialHealth = 300\n\
           End\n\
           Behavior = CashHackSpecialPower ModuleTag_SP1\n\
             SpecialPowerTemplate = SpWP52_Col1\n\
             MoneyAmount = 500\n\
           End\n\
         End\n\
         Object Col_TargetSupplyCenter\n\
           KindOf = STRUCTURE FS_SUPPLY_CENTER\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
         End\n\
         Object Col_TargetDropzone\n\
           KindOf = STRUCTURE FS_SUPPLY_DROPZONE\n\
           Body = ActiveBody ModuleTag_Body\n\
             MaxHealth = 200\n\
             InitialHealth = 200\n\
           End\n\
           Behavior = OCLUpdate ModuleTag_OCL\n\
             OCL = OCL_DropzoneTest\n\
             MinDelay = 3000\n\
             MaxDelay = 3000\n\
           End\n\
         End\n",
    );

    let col_loc = Coord3D::new(0.0, 0.0, 0.0);
    let col_norm = Coord3D::new(0.0, 0.0, 1.0);

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap_or(idx as u32);
        let frame0 = get_u32(line, "frame").unwrap();
        let rel_bc = match get_i32(line, "rel_bc").unwrap() {
            0 => Relationship::Enemies,
            1 => Relationship::Neutral,
            _ => Relationship::Allies,
        };
        let pc_human = Bool::from(get_i32(line, "pc_human").unwrap() != 0);
        let pb_human = Bool::from(get_i32(line, "pb_human").unwrap() != 0);
        let pc_has_sci = get_i32(line, "pc_has_sci").unwrap() != 0;

        let forbid_owner = get_i32(line, "forbid_owner").unwrap() != 0;
        let building_pickup = get_i32(line, "building_pickup").unwrap() != 0;
        let human_only = get_i32(line, "human_only").unwrap() != 0;
        let allow_multi = get_i32(line, "allow_multi").unwrap() != 0;
        let req_science = get_i32(line, "req_science").unwrap() != 0;
        let use_fx_a = get_i32(line, "use_fx_a").unwrap() != 0;
        let req_k_mode = get_u32(line, "req_k_mode").unwrap();
        let forb_k_mode = get_u32(line, "forb_k_mode").unwrap();
        let crate_above = get_i32(line, "crate_above").unwrap() != 0;
        let crate_has_ai = get_i32(line, "crate_has_ai").unwrap() != 0;
        let goal_mode_a = get_u32(line, "goal_mode_a").unwrap();
        let target_same_owner = get_i32(line, "target_same_owner").unwrap() != 0;
        let target_sel_a = get_u32(line, "target_sel_a").unwrap();

        let sab_dur_ms = get_u32(line, "sab_dur_ms").unwrap();
        let pow_dur_ms = get_u32(line, "pow_dur_ms").unwrap();
        let steal_amt = get_u32(line, "steal_amt").unwrap();
        let pb_money0 = get_u32(line, "pb_money0").unwrap();
        let pc_money0 = get_u32(line, "pc_money0").unwrap();
        let use_fx_b = get_i32(line, "use_fx_b").unwrap() != 0;
        let sab_goal_match = get_i32(line, "sab_goal_match").unwrap() != 0;
        let sab_status_mode = get_u32(line, "sab_status_mode").unwrap();
        let fac_kind_sel = get_u32(line, "fac_kind_sel").unwrap();
        let sw_kind_sel = get_u32(line, "sw_kind_sel").unwrap();

        api.ctx.world.frame = frame0;
        api.ctx.world.player_relationships.insert((1, 2), rel_bc);
        api.ctx.world.player_relationships.insert((2, 1), rel_bc);
        api.ctx.world.player_is_human.insert(team_b, pb_human);
        api.ctx.world.player_is_human.insert(team_c, pc_human);
        let owner = api.ctx.player_of_team_mut(team_b).expect("player");
        owner.sciences.clear();
        let sci_c = &mut api.ctx.player_of_team_mut(team_c).expect("player").sciences;
        sci_c.clear();
        if pc_has_sci {
            sci_c.push(sci_col);
        }

        let t_probe = format!("Sab52_Probe_{seed}");
        let t_cmd_name = format!("Sab52_Cmd_{seed}");
        let t_fake_name = format!("Sab52_Fake_{seed}");
        let t_net_name = format!("Sab52_Net_{seed}");
        let t_fac_name = format!("Sab52_Fac_{seed}");
        let t_pow_name = format!("Sab52_Pow_{seed}");
        let t_sw_name = format!("Sab52_Sw_{seed}");
        let t_sup_name = format!("Sab52_Sup_{seed}");
        let t_drop_name = format!("Sab52_Drop_{seed}");

        let mut dyn_ini = String::new();
        let _ = writeln!(dyn_ini, "Object {t_probe}");
        let _ = writeln!(dyn_ini, "  KindOf = INFANTRY");
        let _ = writeln!(
            dyn_ini,
            "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End"
        );
        if crate_has_ai {
            let _ = writeln!(dyn_ini, "  Behavior = AIUpdateInterface ModuleTag_AI\n  End");
        }
        let _ = writeln!(
            dyn_ini,
            "  Behavior = SabotageMilitaryFactoryCrateCollide ModuleTag_Collide"
        );
        let _ = writeln!(dyn_ini, "    SabotageDuration = {sab_dur_ms}");
        let _ = writeln!(
            dyn_ini,
            "    ForbidOwnerPlayer = {}",
            if forbid_owner { "Yes" } else { "No" }
        );
        let _ = writeln!(
            dyn_ini,
            "    BuildingPickup = {}",
            if building_pickup { "Yes" } else { "No" }
        );
        let _ = writeln!(
            dyn_ini,
            "    HumanOnly = {}",
            if human_only { "Yes" } else { "No" }
        );
        let _ = writeln!(
            dyn_ini,
            "    AllowMultiPickup = {}",
            if allow_multi { "Yes" } else { "No" }
        );
        if req_k_mode == 1 {
            let _ = writeln!(dyn_ini, "    RequiredKindOf = STRUCTURE");
        } else if req_k_mode == 2 {
            let _ = writeln!(dyn_ini, "    RequiredKindOf = INFANTRY");
        }
        if forb_k_mode == 1 {
            let _ = writeln!(dyn_ini, "    ForbiddenKindOf = FS_AIRFIELD");
        }
        if req_science {
            let _ = writeln!(dyn_ini, "    PickupScience = SCI_WP52_COL");
        }
        if use_fx_a {
            let _ = writeln!(dyn_ini, "    ExecuteFX = FX_CollideExec");
        }
        let _ = writeln!(
            dyn_ini,
            "    ExecuteAnimation = AnimProbe\n\
                 ExecuteAnimationTime = 1.5\n\
                 ExecuteAnimationZRise = 2.5\n\
                 ExecuteAnimationFades = No\n\
               End\nEnd"
        );

        let mut append_sab = |name: &str, mod_name: &str, extra: &str| {
            let _ = writeln!(dyn_ini, "Object {name}");
            let _ = writeln!(dyn_ini, "  KindOf = INFANTRY");
            let _ = writeln!(
                dyn_ini,
                "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 100\n    InitialHealth = 100\n  End"
            );
            let _ = writeln!(dyn_ini, "  Behavior = AIUpdateInterface ModuleTag_AI\n  End");
            let _ = writeln!(dyn_ini, "  Behavior = {mod_name} ModuleTag_Collide");
            let _ = writeln!(dyn_ini, "    BuildingPickup = Yes");
            if use_fx_b {
                let _ = writeln!(dyn_ini, "    ExecuteFX = FX_CollideExec");
            }
            if !extra.is_empty() {
                let _ = write!(dyn_ini, "{extra}");
            }
            let _ = writeln!(dyn_ini, "  End\nEnd");
        };
        let dur_line = format!("    SabotageDuration = {sab_dur_ms}\n");
        let pow_line = format!("    SabotagePowerDuration = {pow_dur_ms}\n");
        let steal_line = format!("    StealCashAmount = {steal_amt}\n");
        append_sab(&t_cmd_name, "SabotageCommandCenterCrateCollide", "");
        append_sab(&t_fake_name, "SabotageFakeBuildingCrateCollide", "");
        append_sab(&t_net_name, "SabotageInternetCenterCrateCollide", &dur_line);
        append_sab(&t_fac_name, "SabotageMilitaryFactoryCrateCollide", &dur_line);
        append_sab(&t_pow_name, "SabotagePowerPlantCrateCollide", &pow_line);
        append_sab(&t_sw_name, "SabotageSuperweaponCrateCollide", "");
        append_sab(&t_sup_name, "SabotageSupplyCenterCrateCollide", &steal_line);
        append_sab(&t_drop_name, "SabotageSupplyDropzoneCrateCollide", &steal_line);

        load_objects_only(&mut api, &dyn_ini);

        // =========================================================================
        // Part A: CrateCollide base validity, multi-pickup guard, AI goal, snapshot
        // =========================================================================
        let target_team_a = if target_same_owner { team_b } else { team_c };
        let decoy_a = spawn(&mut api, "Col_DummyItem", team_c, col_loc);
        let crate_pos = Coord3D::new(0.0, 0.0, if crate_above { 25.0 } else { 0.0 });
        let crate_slot = spawn(&mut api, &t_probe, team_b, crate_pos);
        let target_a = match target_sel_a {
            0 => None,
            1 => Some(spawn(&mut api, "Col_TargetBarracks", team_a, col_loc)),
            2 => {
                let s = spawn(&mut api, "Col_TargetParachute", target_team_a, col_loc);
                attach_stub_ai(&mut api, s);
                Some(s)
            }
            3 => {
                let s = spawn(&mut api, "Col_TargetBarracks", target_team_a, col_loc);
                api.object_mut(s).unwrap().set_effectively_dead(TRUE);
                Some(s)
            }
            4 => Some(spawn(&mut api, "Col_DummyItem", target_team_a, col_loc)),
            6 => {
                let s = spawn(&mut api, "Col_TargetBarracks", target_team_a, col_loc);
                api.set_status(s, ObjectStatusType::UnderConstruction);
                Some(s)
            }
            7 => {
                let s = spawn(&mut api, "Col_TargetBarracks", target_team_a, col_loc);
                api.set_status(s, ObjectStatusType::Sold);
                Some(s)
            }
            8 => Some(spawn(&mut api, "Col_TargetAirfield", target_team_a, col_loc)),
            _ => Some(spawn(&mut api, "Col_TargetBarracks", target_team_a, col_loc)),
        };
        if crate_has_ai {
            attach_stub_ai(&mut api, crate_slot);
            api.object_mut(crate_slot).unwrap().ai_goal_object = match goal_mode_a {
                1 => target_a,
                2 => Some(decoy_a),
                _ => None,
            };
        }

        let cm_a = first_collide_module(&api, crate_slot);
        api.ctx.world.triggered_fx_lists.clear();
        let a_would_ground = collide_would(&mut api, cm_a, None);
        let a_would_1 = collide_would(&mut api, cm_a, target_a);
        collide_exec(&mut api, cm_a, target_a, &col_loc, &col_norm);
        let a_dest_1 = is_destroyed_i32(&api, crate_slot);
        let a_would_2 = collide_would(&mut api, cm_a, target_a);
        let a_until_1 = target_a.map_or(0, |t| hacked_until(&api, t));
        collide_exec(&mut api, cm_a, target_a, &col_loc, &col_norm);
        let a_dest_2 = is_destroyed_i32(&api, crate_slot);
        let a_hacked_2 = target_a.map_or(0, |t| is_hacked_i32(&api, t));
        let a_until_2 = target_a.map_or(0, |t| hacked_until(&api, t));

        if use_fx_a && a_dest_1 != 0 {
            assert!(
                api.ctx
                    .world
                    .triggered_fx_lists
                    .iter()
                    .any(|(n, _)| n == "FX_CollideExec"),
                "seed {seed} expected FX_CollideExec"
            );
        }

        let (a_crc, a_xfer) = snapshot_mod(&mut api, cm_a);
        let load_bytes = decode_hex_bytes(&a_xfer).expect("decode a_xfer");
        let mut xl = XferLoad::new(&load_bytes);
        api.ctx
            .dispatch_module(cm_a, |m, sim, _ctx| {
                m.xfer(&mut xl, sim).expect("xfer load");
            })
            .expect("dispatch load");
        let (_, a_rt_xfer) = snapshot_mod(&mut api, cm_a);

        let exp_a = get_i32_vec(line, "a").unwrap();
        assert_eq!(
            vec![
                a_would_ground,
                a_would_1,
                a_dest_1,
                a_would_2,
                a_dest_2,
                a_hacked_2,
                a_until_1 as i32,
                a_until_2 as i32,
            ],
            exp_a,
            "seed {seed} Part A CrateCollide mismatch"
        );
        check_crc(a_crc, get_u32(line, "a_crc").unwrap(), "a", seed);
        assert_eq!(a_xfer, get_str(line, "a_xfer").unwrap(), "seed {seed} a_xfer");
        assert_eq!(
            a_rt_xfer,
            get_str(line, "a_rt_xfer").unwrap(),
            "seed {seed} a_rt_xfer"
        );

        api.destroy_object(crate_slot);
        if let Some(t) = target_a {
            api.destroy_object(t);
        }
        api.destroy_object(decoy_a);
        api.ctx.process_destroy_list();

        // =========================================================================
        // Part B: All 8 Sabotage* CrateCollide modules
        // =========================================================================
        api.ctx.world.frame = frame0;
        let decoy_b = spawn(&mut api, "Col_DummyItem", team_c, col_loc);
        let t_cmd = spawn(&mut api, "Col_TargetCmdCenter", team_c, col_loc);
        let t_fake = spawn(&mut api, "Col_TargetFake", team_c, col_loc);
        let t_net = spawn(&mut api, "Col_TargetInternet", team_c, col_loc);
        let t_hacker = spawn(&mut api, "Col_DummyItem", team_c, col_loc);
        api.object_mut(t_hacker).unwrap().contained_by = Some(t_net);
        let fac_tmpl = match fac_kind_sel {
            0 => "Col_TargetBarracks",
            1 => "Col_TargetWarfactory",
            2 => "Col_TargetAirfield",
            _ => "Col_TargetCarrier",
        };
        let t_fac = spawn(&mut api, fac_tmpl, team_c, col_loc);
        let t_carrier = spawn(&mut api, "Col_TargetCarrier", team_c, col_loc);
        let t_pow = spawn(&mut api, "Col_TargetPower", team_c, col_loc);
        let t_sw = spawn(
            &mut api,
            if sw_kind_sel == 0 {
                "Col_TargetSuperweapon"
            } else {
                "Col_TargetStrategy"
            },
            team_c,
            col_loc,
        );
        let t_sup = spawn(&mut api, "Col_TargetSupplyCenter", team_c, col_loc);
        let t_drop = spawn(&mut api, "Col_TargetDropzone", team_c, col_loc);

        let all_t = [t_cmd, t_fake, t_net, t_fac, t_pow, t_sw, t_sup, t_drop];
        if sab_status_mode == 4 {
            for &o in &all_t {
                api.set_status(o, ObjectStatusType::UnderConstruction);
            }
        } else if sab_status_mode == 5 {
            for &o in &all_t {
                api.set_status(o, ObjectStatusType::Sold);
            }
        }

        let sab_frame = frame0 + 50;
        api.ctx.world.frame = sab_frame;
        let player_b = api.ctx.player_index(team_b);
        let player_c = api.ctx.player_index(team_c);
        api.ctx
            .world
            .player_energy
            .entry(player_c)
            .or_default()
            .set_power_sabotaged_till_frame(0);

        // 1. SabotageCommandCenterCrateCollide
        let (s_cmd, m_cmd) = spawn_sab_unit(
            &mut api,
            &t_cmd_name,
            team_b,
            t_cmd,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let cmd_is_sab = collide_is_sabotage(&mut api, m_cmd);
        let cmd_would_bad = collide_would(&mut api, m_cmd, Some(t_fake));
        let cmd_would_ok = collide_would(&mut api, m_cmd, Some(t_cmd));
        collide_exec(&mut api, m_cmd, Some(t_cmd), &col_loc, &col_norm);
        let cmd_dest = is_destroyed_i32(&api, s_cmd);
        let cmd_sps = sp_ready_frames(&mut api, t_cmd);
        let (cmd_crc, cmd_xfer) = snapshot_mod(&mut api, m_cmd);
        assert_eq!(
            vec![
                cmd_would_bad,
                cmd_would_ok,
                cmd_dest,
                cmd_sps.first().copied().unwrap_or(0) as i32,
                cmd_sps.get(1).copied().unwrap_or(0) as i32,
            ],
            get_i32_vec(line, "cmd").unwrap(),
            "seed {seed} cmd mismatch"
        );
        check_crc(cmd_crc, get_u32(line, "cmd_crc").unwrap(), "cmd", seed);
        assert_eq!(cmd_xfer, get_str(line, "cmd_xfer").unwrap(), "seed {seed} cmd_xfer");

        // 2. SabotageFakeBuildingCrateCollide
        let (s_fake, m_fake) = spawn_sab_unit(
            &mut api,
            &t_fake_name,
            team_b,
            t_fake,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let fake_is_sab = collide_is_sabotage(&mut api, m_fake);
        let fake_would_bad = collide_would(&mut api, m_fake, Some(t_cmd));
        let fake_would_ok = collide_would(&mut api, m_fake, Some(t_fake));
        collide_exec(&mut api, m_fake, Some(t_fake), &col_loc, &col_norm);
        let fake_dest = is_destroyed_i32(&api, s_fake);
        let fake_t_dead = i32::from(api.object(t_fake).unwrap().is_effectively_dead() != FALSE);
        let fake_t_hp: Real = api
            .ctx
            .dispatch_module(api.object(t_fake).unwrap().body_slot.unwrap(), |m, _, _| {
                m.as_body().unwrap().get_health()
            })
            .unwrap();
        let (fake_crc, fake_xfer) = snapshot_mod(&mut api, m_fake);
        assert_eq!(
            vec![
                fake_would_bad,
                fake_would_ok,
                fake_dest,
                fake_t_dead,
                fake_t_hp.to_bits() as i32,
            ],
            get_i32_vec(line, "fake").unwrap(),
            "seed {seed} fake mismatch"
        );
        check_crc(fake_crc, get_u32(line, "fake_crc").unwrap(), "fake", seed);
        assert_eq!(fake_xfer, get_str(line, "fake_xfer").unwrap(), "seed {seed} fake_xfer");

        // 3. SabotageInternetCenterCrateCollide
        let (s_net, m_net) = spawn_sab_unit(
            &mut api,
            &t_net_name,
            team_b,
            t_net,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let net_is_sab = collide_is_sabotage(&mut api, m_net);
        let net_would_bad = collide_would(&mut api, m_net, Some(t_fake));
        let net_would_ok = collide_would(&mut api, m_net, Some(t_net));
        collide_exec(&mut api, m_net, Some(t_net), &col_loc, &col_norm);
        let net_dest = is_destroyed_i32(&api, s_net);
        let net_t_hacked = is_hacked_i32(&api, t_net);
        let net_t_until = hacked_until(&api, t_net);
        let net_h_hacked = is_hacked_i32(&api, t_hacker);
        let net_h_until = hacked_until(&api, t_hacker);
        let (net_crc, net_xfer) = snapshot_mod(&mut api, m_net);
        assert_eq!(
            vec![
                net_would_bad,
                net_would_ok,
                net_dest,
                net_t_hacked,
                net_t_until as i32,
                net_h_hacked,
                net_h_until as i32,
            ],
            get_i32_vec(line, "net").unwrap(),
            "seed {seed} net mismatch"
        );
        check_crc(net_crc, get_u32(line, "net_crc").unwrap(), "net", seed);
        assert_eq!(net_xfer, get_str(line, "net_xfer").unwrap(), "seed {seed} net_xfer");

        // 4. SabotageMilitaryFactoryCrateCollide
        let (s_fac, m_fac) = spawn_sab_unit(
            &mut api,
            &t_fac_name,
            team_b,
            t_fac,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let fac_is_sab = collide_is_sabotage(&mut api, m_fac);
        let fac_would_carrier = collide_would(&mut api, m_fac, Some(t_carrier));
        let fac_would_ok = collide_would(&mut api, m_fac, Some(t_fac));
        collide_exec(&mut api, m_fac, Some(t_fac), &col_loc, &col_norm);
        let fac_dest = is_destroyed_i32(&api, s_fac);
        let fac_t_hacked = is_hacked_i32(&api, t_fac);
        let fac_t_until = hacked_until(&api, t_fac);
        let (fac_crc, fac_xfer) = snapshot_mod(&mut api, m_fac);
        assert_eq!(
            vec![
                fac_would_carrier,
                fac_would_ok,
                fac_dest,
                fac_t_hacked,
                fac_t_until as i32,
            ],
            get_i32_vec(line, "fac").unwrap(),
            "seed {seed} fac mismatch"
        );
        check_crc(fac_crc, get_u32(line, "fac_crc").unwrap(), "fac", seed);
        assert_eq!(fac_xfer, get_str(line, "fac_xfer").unwrap(), "seed {seed} fac_xfer");

        // 5. SabotagePowerPlantCrateCollide
        let (s_pow, m_pow) = spawn_sab_unit(
            &mut api,
            &t_pow_name,
            team_b,
            t_pow,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let pow_is_sab = collide_is_sabotage(&mut api, m_pow);
        let pow_would_bad = collide_would(&mut api, m_pow, Some(t_fake));
        let pow_would_ok = collide_would(&mut api, m_pow, Some(t_pow));
        collide_exec(&mut api, m_pow, Some(t_pow), &col_loc, &col_norm);
        let pow_dest = is_destroyed_i32(&api, s_pow);
        let pow_until = api
            .ctx
            .world
            .player_energy
            .get(&player_c)
            .map_or(0, |e| e.get_power_sabotaged_till_frame());
        let (pow_crc, pow_xfer) = snapshot_mod(&mut api, m_pow);
        assert_eq!(
            vec![pow_would_bad, pow_would_ok, pow_dest, pow_until as i32],
            get_i32_vec(line, "pow").unwrap(),
            "seed {seed} pow mismatch"
        );
        check_crc(pow_crc, get_u32(line, "pow_crc").unwrap(), "pow", seed);
        assert_eq!(pow_xfer, get_str(line, "pow_xfer").unwrap(), "seed {seed} pow_xfer");

        // 6. SabotageSuperweaponCrateCollide
        let (s_sw, m_sw) = spawn_sab_unit(
            &mut api,
            &t_sw_name,
            team_b,
            t_sw,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let sw_is_sab = collide_is_sabotage(&mut api, m_sw);
        let sw_would_bad = collide_would(&mut api, m_sw, Some(t_cmd));
        let sw_would_ok = collide_would(&mut api, m_sw, Some(t_sw));
        collide_exec(&mut api, m_sw, Some(t_sw), &col_loc, &col_norm);
        let sw_dest = is_destroyed_i32(&api, s_sw);
        let sw_sps = sp_ready_frames(&mut api, t_sw);
        let (sw_crc, sw_xfer) = snapshot_mod(&mut api, m_sw);
        assert_eq!(
            vec![
                sw_would_bad,
                sw_would_ok,
                sw_dest,
                sw_sps.first().copied().unwrap_or(0) as i32,
                sw_sps.get(1).copied().unwrap_or(0) as i32,
            ],
            get_i32_vec(line, "sw").unwrap(),
            "seed {seed} sw mismatch"
        );
        check_crc(sw_crc, get_u32(line, "sw_crc").unwrap(), "sw", seed);
        assert_eq!(sw_xfer, get_str(line, "sw_xfer").unwrap(), "seed {seed} sw_xfer");

        // 7. SabotageSupplyCenterCrateCollide
        api.ctx.set_player_cash(player_b, pb_money0);
        api.ctx.set_player_cash(player_c, pc_money0);
        let (s_sup, m_sup) = spawn_sab_unit(
            &mut api,
            &t_sup_name,
            team_b,
            t_sup,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let sup_is_sab = collide_is_sabotage(&mut api, m_sup);
        let sup_would_bad = collide_would(&mut api, m_sup, Some(t_drop));
        let sup_would_ok = collide_would(&mut api, m_sup, Some(t_sup));
        collide_exec(&mut api, m_sup, Some(t_sup), &col_loc, &col_norm);
        let sup_dest = is_destroyed_i32(&api, s_sup);
        let sup_pb_money = api.player_money(team_b);
        let sup_pc_money = api.player_money(team_c);
        let (sup_crc, sup_xfer) = snapshot_mod(&mut api, m_sup);
        assert_eq!(
            vec![
                sup_would_bad,
                sup_would_ok,
                sup_dest,
                sup_pb_money as i32,
                sup_pc_money as i32,
            ],
            get_i32_vec(line, "sup").unwrap(),
            "seed {seed} sup mismatch"
        );
        check_crc(sup_crc, get_u32(line, "sup_crc").unwrap(), "sup", seed);
        assert_eq!(sup_xfer, get_str(line, "sup_xfer").unwrap(), "seed {seed} sup_xfer");

        // 8. SabotageSupplyDropzoneCrateCollide
        api.ctx.set_player_cash(player_b, pb_money0);
        api.ctx.set_player_cash(player_c, pc_money0);
        api.ctx.world.supply_dropzone_resets.clear();
        let (s_drop, m_drop) = spawn_sab_unit(
            &mut api,
            &t_drop_name,
            team_b,
            t_drop,
            decoy_b,
            sab_goal_match,
            col_loc,
        );
        let drop_is_sab = collide_is_sabotage(&mut api, m_drop);
        let drop_would_bad = collide_would(&mut api, m_drop, Some(t_sup));
        let drop_would_ok = collide_would(&mut api, m_drop, Some(t_drop));
        collide_exec(&mut api, m_drop, Some(t_drop), &col_loc, &col_norm);
        let drop_dest = is_destroyed_i32(&api, s_drop);
        let drop_pb_money = api.player_money(team_b);
        let drop_pc_money = api.player_money(team_c);
        let drop_id = api.object(t_drop).map_or(ObjectId::INVALID, |o| o.id);
        let drop_reset_recorded = api.ctx.world.supply_dropzone_resets.contains(&drop_id);
        let exp_drop = get_i32_vec(line, "drop").unwrap();
        assert_eq!(
            vec![
                drop_would_bad,
                drop_would_ok,
                drop_dest,
                drop_pb_money as i32,
                drop_pc_money as i32,
            ],
            exp_drop[..5].to_vec(),
            "seed {seed} drop mismatch"
        );
        assert_eq!(
            drop_reset_recorded,
            drop_dest != 0,
            "seed {seed} dropzone reset recording mismatch"
        );
        if drop_dest != 0 {
            assert_eq!(exp_drop[5], sab_frame as i32, "seed {seed} drop ocl_start");
            assert_eq!(
                exp_drop[6],
                (sab_frame + 90) as i32,
                "seed {seed} drop ocl_next"
            );
        }
        let (drop_crc, drop_xfer) = snapshot_mod(&mut api, m_drop);
        check_crc(drop_crc, get_u32(line, "drop_crc").unwrap(), "drop", seed);
        assert_eq!(drop_xfer, get_str(line, "drop_xfer").unwrap(), "seed {seed} drop_xfer");

        let is_sab_mask = u32::from(cmd_is_sab)
            | (u32::from(fake_is_sab) << 1)
            | (u32::from(net_is_sab) << 2)
            | (u32::from(fac_is_sab) << 3)
            | (u32::from(pow_is_sab) << 4)
            | (u32::from(sw_is_sab) << 5)
            | (u32::from(sup_is_sab) << 6)
            | (u32::from(drop_is_sab) << 7);
        assert_eq!(
            is_sab_mask,
            get_u32(line, "is_sab_mask").unwrap(),
            "seed {seed} is_sab_mask mismatch"
        );

        for slot in [
            t_hacker, s_cmd, s_fake, s_net, s_fac, s_pow, s_sw, s_sup, s_drop, t_cmd, t_fake,
            t_net, t_fac, t_carrier, t_pow, t_sw, t_sup, t_drop, decoy_b,
        ] {
            api.destroy_object(slot);
        }
        api.ctx.process_destroy_list();
    }
}
