//! Gate V2-Golden test suite for `WP-5.1-A` (`ObjectHelper` hierarchy:
//! `ObjectHelper`, `ObjectDefectionHelper`, `ObjectRepulsorHelper`, `ObjectSMCHelper`,
//! `ObjectWeaponStatusHelper`, `StatusDamageHelper`, `SubdualDamageHelper`,
//! `TempWeaponBonusHelper`, 1,000 C++ reference vectors).

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::XferSave;
use zh_logic::arena::ModuleSlot;
use zh_logic::combat::damage::DamageInfo;
use zh_logic::gamelogic::global_data::WeaponBonusConditionType;
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::combat_types::DamageType;
use zh_logic::rules::model_state::ModelConditionFlagType;
use zh_logic::rules::object_status_types::ObjectStatusType;
use zh_logic::sim::{SimApi, SimContext, SimView};

const WP51_HELPER_JSON: &str = include_str!("../../../docs/golden_vectors/WP51_Helper.json");

fn extract_str_array(line: &str, key: &str) -> Vec<String> {
    let pat = format!("\"{key}\":");
    let start = line.find(&pat).expect("missing key") + pat.len();
    let rest = line[start..].trim_start();
    assert!(rest.starts_with('['));
    let end = rest.find(']').expect("closing ]");
    let inner = &rest[1..end];
    let mut out = Vec::new();
    let bytes = inner.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            i += 1;
            let s = i;
            while i < bytes.len() && bytes[i] != b'"' {
                i += 1;
            }
            out.push(inner[s..i].to_owned());
        }
        i += 1;
    }
    out
}

fn helper_xfer_hex(sim: &mut SimContext, slot_opt: Option<ModuleSlot>) -> Vec<u8> {
    let Some(ms) = slot_opt else {
        return Vec::new();
    };
    sim.dispatch_module(ms, |m, sim_v, _| {
        let mut xs = XferSave::new();
        m.xfer(&mut xs, sim_v).expect("helper xfer save");
        xs.into_bytes()
    })
    .expect("dispatch helper xfer")
}

fn helper_pri(sim: &SimContext, slot_opt: Option<ModuleSlot>) -> u32 {
    slot_opt
        .and_then(|ms| sim.module_schedule(ms))
        .map_or(0, |s| s.priority())
}

fn step_helper_update(sim: &mut SimContext, slot_opt: Option<ModuleSlot>) -> u32 {
    let ms = slot_opt.expect("helper slot present");
    sim.dispatch_module(ms, |m, sim_a, ctx| {
        m.as_update_mut()
            .expect("update facet")
            .update(sim_a, ctx)
            .0
    })
    .expect("dispatch helper update")
}

#[test]
fn test_wp51_helper_oracle_integrity() {
    assert!(WP51_HELPER_JSON.contains("\"packet\": \"WP-5.1-A\""));
    assert!(WP51_HELPER_JSON.contains("\"seed_count\": 1000"));
    assert!(WP51_HELPER_JSON.contains("\"seed0_diagnostics\":"));
    assert_eq!(extract_vector_lines(WP51_HELPER_JSON).len(), 1000);
}

#[test]
fn test_wp51_helper_1000_golden_vectors() {
    let lines = extract_vector_lines(WP51_HELPER_JSON);
    assert_eq!(lines.len(), 1000);

    let mut sim = SimContext::new();
    let mut wt = WeaponTemplate::new();
    wt.parse_ini_block(
        "\
Weapon HelperWpn
  PrimaryDamage = 15
  AttackRange = 100
  DamageType = SMALL_ARMS
  DeathType = NORMAL
  WeaponSpeed = 999999
  ClipSize = 2
  ClipReloadTime = 1000
  DelayBetweenShots = 100
End",
    )
    .expect("parse HelperWpn");
    sim.rules.weapon_store.add_template(wt);

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed") as usize;
        assert_eq!(seed, idx);

        let frame0 = get_u32(line, "frame").expect("frame");
        let repulsors = get_i32(line, "repulsors").expect("repulsors") != 0;
        let inactive_body = get_i32(line, "inactive_body").expect("inactive_body") != 0;
        let can_repulse = get_i32(line, "can_repulse").expect("can_repulse") != 0;
        let shrub = get_i32(line, "shrub").expect("shrub") != 0;
        let has_wpn = get_i32(line, "has_wpn").expect("has_wpn") != 0;
        let smc_dur = get_u32(line, "smc_dur").expect("smc_dur");
        let status_enum = ObjectStatusType::from_i32(get_i32(line, "status_enum").unwrap()).unwrap();
        let status_dur = f32::from_bits(get_u32(line, "status_dur").unwrap());
        let bonus_enum =
            WeaponBonusConditionType::from_i32(get_i32(line, "bonus_enum").unwrap()).unwrap();
        let bonus_dur = get_u32(line, "bonus_dur").unwrap();
        let def_dur = get_u32(line, "def_dur").unwrap();
        let def_blow = get_i32(line, "def_blow").unwrap() != 0;
        let def_steps = get_u32(line, "def_steps").unwrap();
        let sub_cap = f32::from_bits(get_u32(line, "sub_cap").unwrap());
        let sub_rate_f = get_u32(line, "sub_rate_f").unwrap();
        let sub_heal = f32::from_bits(get_u32(line, "sub_heal").unwrap());
        let sub_dmg = f32::from_bits(get_u32(line, "sub_dmg").unwrap());
        let sub_steps = get_u32(line, "sub_steps").unwrap();

        let want_present = get_i32_vec(line, "present").unwrap();
        let want_pri_init = get_i32_vec(line, "pri_init").unwrap();
        let want_x0 = extract_str_array(line, "x0");
        let want_smc = get_i32_vec(line, "smc").unwrap();
        let want_smc_x1 = decode_hex_bytes(get_str(line, "smc_x1").unwrap()).unwrap();
        let want_sd = get_i32_vec(line, "sd").unwrap();
        let want_sd_x1 = decode_hex_bytes(get_str(line, "sd_x1").unwrap()).unwrap();
        let want_sd_x2 = decode_hex_bytes(get_str(line, "sd_x2").unwrap()).unwrap();
        let want_rep = get_i32_vec(line, "rep").unwrap();
        let want_def = get_i32_vec(line, "def").unwrap();
        let want_def_x1 = decode_hex_bytes(get_str(line, "def_x1").unwrap()).unwrap();
        let want_def_x2 = decode_hex_bytes(get_str(line, "def_x2").unwrap()).unwrap();
        let want_twb = get_i32_vec(line, "twb").unwrap();
        let want_twb_x1 = decode_hex_bytes(get_str(line, "twb_x1").unwrap()).unwrap();
        let want_twb_x2 = decode_hex_bytes(get_str(line, "twb_x2").unwrap()).unwrap();
        let want_sub = get_i32_vec(line, "sub").unwrap();
        let want_sub_x1 = decode_hex_bytes(get_str(line, "sub_x1").unwrap()).unwrap();
        let want_sub_x2 = decode_hex_bytes(get_str(line, "sub_x2").unwrap()).unwrap();
        let want_ws = get_i32_vec(line, "ws").unwrap();

        sim.world.frame = frame0;
        sim.world.ai_enable_repulsors = if repulsors { TRUE } else { FALSE };

        let obj_name = format!("H51_Unit_{seed}");
        let kind_extra = format!(
            "{}{}{}",
            if can_repulse { " CAN_BE_REPULSED" } else { "" },
            if shrub { " SHRUBBERY" } else { "" },
            if has_wpn { " CAN_ATTACK" } else { "" }
        );
        let wpn_block = if has_wpn {
            "  WeaponSet\n    Conditions = None\n    Weapon = PRIMARY HelperWpn\n  End\n"
        } else {
            ""
        };
        let body_block = if inactive_body {
            "  Body = InactiveBody ModuleTag_Body\n  End\n".to_owned()
        } else {
            let sub_rate_ms = (sub_rate_f * 100) / 3;
            format!(
                "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n    SubdualDamageCap = {sub_cap}\n    SubdualDamageHealRate = {sub_rate_ms}\n    SubdualDamageHealAmount = {sub_heal}\n  End\n"
            )
        };
        let ini_text = format!(
            "Object {obj_name}\n  KindOf = SELECTABLE{kind_extra}\n{wpn_block}{body_block}End\n"
        );
        let mut reader = IniReader::new("helper.ini", ini_text.as_bytes(), IniLoadType::Overwrite);
        sim.rules
            .load_thing_templates_from_ini(&mut reader)
            .expect("load helper template");
        let tmpl_id = sim.rules.thing_factory.find_template_id(&obj_name).unwrap();
        let obj = sim.spawn_object(tmpl_id, TeamId(1), Coord3D::default(), 0.0);
        if !inactive_body {
            assert_eq!(sim.get_subdual_damage_heal_rate(obj), sub_rate_f, "seed {seed} sub_rate_f");
        }

        let (s_smc, s_sd, s_sub, s_rep, s_def, s_ws, s_twb) = {
            let r = sim.object(obj).unwrap();
            (
                r.smc_helper_slot,
                r.status_damage_helper_slot,
                r.subdual_damage_helper_slot,
                r.repulsor_helper_slot,
                r.defection_helper_slot,
                r.weapon_status_helper_slot,
                r.temp_weapon_bonus_helper_slot,
            )
        };
        let slots = [s_smc, s_sd, s_sub, s_rep, s_def, s_ws, s_twb];
        let got_present: Vec<i32> = slots.iter().map(|s| i32::from(s.is_some())).collect();
        assert_eq!(got_present, want_present, "seed {seed} present");

        for (i, &s) in slots.iter().enumerate() {
            assert_eq!(
                helper_pri(&sim, s),
                want_pri_init[i] as u32,
                "seed {seed} pri_init[{i}]"
            );
            assert_eq!(
                helper_xfer_hex(&mut sim, s),
                decode_hex_bytes(&want_x0[i]).unwrap(),
                "seed {seed} x0[{i}]"
            );
        }

        // 1) ObjectSMCHelper
        sim.set_special_model_condition_state(obj, ModelConditionFlagType::SpecialCheering, smc_dur);
        assert_eq!(helper_pri(&sim, s_smc), want_smc[0] as u32, "seed {seed} smc_pri1");
        assert_eq!(helper_xfer_hex(&mut sim, s_smc), want_smc_x1, "seed {seed} smc_x1");
        let smc_ret = step_helper_update(&mut sim, s_smc);
        let smc_flag_after = sim
            .object(obj)
            .unwrap()
            .model_condition
            .test(ModelConditionFlagType::SpecialCheering.as_i32())
            != FALSE;
        assert_eq!(smc_ret, want_smc[1] as u32, "seed {seed} smc_ret");
        assert_eq!(i32::from(smc_flag_after), want_smc[2], "seed {seed} smc_flag_after");

        // 2) StatusDamageHelper
        if s_sd.is_some() {
            sim.trigger_status_damage(obj, status_enum, status_dur);
            let sd_set = sim
                .object(obj)
                .unwrap()
                .status
                .test(status_enum.as_i32())
                != FALSE;
            assert_eq!(helper_pri(&sim, s_sd), want_sd[0] as u32, "seed {seed} sd_pri1");
            assert_eq!(i32::from(sd_set), want_sd[2], "seed {seed} sd_set");
            assert_eq!(helper_xfer_hex(&mut sim, s_sd), want_sd_x1, "seed {seed} sd_x1");
            sim.world.frame = frame0 + 1000;
            let sd_ret = step_helper_update(&mut sim, s_sd);
            let sd_clr = sim
                .object(obj)
                .unwrap()
                .status
                .test(status_enum.as_i32())
                != FALSE;
            assert_eq!(sd_ret, want_sd[1] as u32, "seed {seed} sd_ret");
            assert_eq!(i32::from(sd_clr), want_sd[3], "seed {seed} sd_clr");
            assert_eq!(helper_xfer_hex(&mut sim, s_sd), want_sd_x2, "seed {seed} sd_x2");
            sim.world.frame = frame0;
        }

        // 3) ObjectRepulsorHelper
        if s_rep.is_some() {
            sim.set_status(obj, ObjectStatusType::Repulsor);
            let rep_set = sim
                .object(obj)
                .unwrap()
                .status
                .test(ObjectStatusType::Repulsor.as_i32())
                != FALSE;
            assert_eq!(helper_pri(&sim, s_rep), want_rep[0] as u32, "seed {seed} rep_pri1");
            assert_eq!(i32::from(rep_set), want_rep[2], "seed {seed} rep_set");
            let rep_ret = step_helper_update(&mut sim, s_rep);
            let rep_clr = sim
                .object(obj)
                .unwrap()
                .status
                .test(ObjectStatusType::Repulsor.as_i32())
                != FALSE;
            assert_eq!(rep_ret, want_rep[1] as u32, "seed {seed} rep_ret");
            assert_eq!(i32::from(rep_clr), want_rep[3], "seed {seed} rep_clr");
        }

        // 4) ObjectDefectionHelper
        if s_def.is_some() {
            sim.defect_object(obj, TeamId(2), def_dur, TRUE);
            let def_pre = sim.object(obj).unwrap().is_undetected_defector() != FALSE;
            assert_eq!(helper_pri(&sim, s_def), want_def[0] as u32, "seed {seed} def_pri1");
            assert_eq!(i32::from(def_pre), want_def[2], "seed {seed} def_pre");
            assert_eq!(helper_xfer_hex(&mut sim, s_def), want_def_x1, "seed {seed} def_x1");
            let mut def_ret = 0;
            for k in 0..def_steps {
                sim.world.frame = frame0 + 1 + k;
                if def_blow && k == 1 {
                    sim.set_status(obj, ObjectStatusType::IsFiringWeapon);
                }
                def_ret = step_helper_update(&mut sim, s_def);
            }
            let def_post = sim.object(obj).unwrap().is_undetected_defector() != FALSE;
            assert_eq!(def_ret, want_def[1] as u32, "seed {seed} def_ret");
            assert_eq!(i32::from(def_post), want_def[3], "seed {seed} def_post");
            assert_eq!(helper_xfer_hex(&mut sim, s_def), want_def_x2, "seed {seed} def_x2");
            sim.world.frame = frame0;
        }

        // 5) TempWeaponBonusHelper
        if s_twb.is_some() {
            sim.trigger_temp_weapon_bonus(obj, bonus_enum, bonus_dur);
            let twb_mask1 = sim.object(obj).unwrap().weapon_bonus_condition;
            assert_eq!(helper_pri(&sim, s_twb), want_twb[0] as u32, "seed {seed} twb_pri1");
            assert_eq!(twb_mask1, want_twb[2] as u32, "seed {seed} twb_mask1");
            assert_eq!(helper_xfer_hex(&mut sim, s_twb), want_twb_x1, "seed {seed} twb_x1");
            sim.world.frame = frame0 + bonus_dur + 1;
            let twb_ret = step_helper_update(&mut sim, s_twb);
            let twb_mask2 = sim.object(obj).unwrap().weapon_bonus_condition;
            assert_eq!(twb_ret, want_twb[1] as u32, "seed {seed} twb_ret");
            assert_eq!(twb_mask2, want_twb[3] as u32, "seed {seed} twb_mask2");
            assert_eq!(helper_xfer_hex(&mut sim, s_twb), want_twb_x2, "seed {seed} twb_x2");
            sim.world.frame = frame0;
        }

        // 6) SubdualDamageHelper
        if s_sub.is_some() {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::SubdualUnresistable;
            dmg.input.amount = sub_dmg;
            sim.attempt_damage(obj, &mut dmg);
            assert_eq!(helper_pri(&sim, s_sub), want_sub[0] as u32, "seed {seed} sub_pri1");
            assert_eq!(helper_xfer_hex(&mut sim, s_sub), want_sub_x1, "seed {seed} sub_x1");
            let mut sub_ret = 0;
            for k in 0..sub_steps {
                sim.world.frame = frame0 + 1 + k;
                sub_ret = step_helper_update(&mut sim, s_sub);
            }
            let bs = sim.object(obj).unwrap().body_slot.unwrap();
            let sub_after = sim
                .world
                .modules
                .payload(bs)
                .and_then(|p| p.as_body())
                .unwrap()
                .get_current_subdual_damage();
            assert_eq!(sub_ret, want_sub[1] as u32, "seed {seed} sub_ret");
            assert_eq!(sub_after.to_bits(), want_sub[2] as u32, "seed {seed} sub_after");
            assert_eq!(helper_xfer_hex(&mut sim, s_sub), want_sub_x2, "seed {seed} sub_x2");
            sim.world.frame = frame0;
        }

        // 7) ObjectWeaponStatusHelper
        if s_ws.is_some() {
            let phase = sim
                .module_schedule(s_ws.unwrap())
                .unwrap()
                .next_call_phase() as i32;
            let ws_ret = step_helper_update(&mut sim, s_ws);
            assert_eq!(phase, want_ws[0], "seed {seed} ws_phase");
            assert_eq!(ws_ret, want_ws[1] as u32, "seed {seed} ws_ret");
        } else {
            assert_eq!(want_ws, [-1, 0]);
        }

        sim.destroy_object_now(obj);
        sim.process_destroy_list();
    }
    let _ = UpdateSleepTime::NONE;
}
