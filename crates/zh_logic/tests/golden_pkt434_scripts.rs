//! Gate V2 Golden Vector parity tests for `PKT-434` (`WP-6.4` `Scripts`).
//!
//! Active structural check (`test_pkt434_oracle_structure_and_constants`) verifies the committed
//! C++ oracle and constants. Remove the two `#[ignore]` attributes once `PKT-434` is implemented.

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::crc::Crc;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_str, get_u32,
};
use zh_common::io::data_chunk::{CachedFileInputStream, DataChunkOutput};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, Real, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::rules::kind_of::{KindOfMaskType, KINDOF_COUNT, KINDOF_NAMES};
use zh_logic::rules::object_status_types::{OBJECT_STATUS_COUNT, OBJECT_STATUS_NAMES};
use zh_logic::scripts::{
    Condition, OrCondition, Parameter, ParameterType, Script, ScriptAction, ScriptGroup,
    ScriptList, ScriptTemplateProvider, ScriptTemplateRegistry, ACTION_NUM_ITEMS, ANY_TEAM,
    BORDER_COLORS, BORDER_COLORS_SIZE, CONDITION_NUM_ITEMS, MAX_PARMS, PARAMETER_NUM_ITEMS,
    SHAKE_INTENSITIES, SHELL_SCRIPT_HOOK_TOTAL, SURFACES, THE_OBJECT_FLAGS_NAMES,
    THE_SHELL_HOOK_NAMES, THIS_PLAYER, THIS_TEAM,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-434.json");

fn splitmix32(s: &mut u32) -> u32 {
    *s = s.wrapping_add(0x9e37_79b9);
    let mut z = *s;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

fn crc_str(s: &str) -> u32 {
    let mut c = Crc::new();
    c.compute_crc(s.as_bytes(), s.len() as Int);
    c.get()
}

fn rot_crc(acc: u32, v: u32) -> u32 {
    acc.rotate_left(1).wrapping_add(v)
}

fn init_deterministic_templates(
    nkg: &mut NameKeyGenerator,
    reg: &mut ScriptTemplateRegistry,
) {
    for i in 0..CONDITION_NUM_ITEMS {
        let ct = &mut reg.conditions[i as usize];
        let iname = format!("COND_INTERNAL_{i}");
        let u0 = format!("C{i}(");
        let u1 = ",".to_string();
        let u2 = ")".to_string();
        ct.internal_name = iname.clone();
        ct.internal_name_key = nkg.name_to_key(&iname);
        ct.ui_name = iname;

        if i == 0 || i == 3 {
            // CONDITION_FALSE (0), CONDITION_TRUE (3)
            ct.num_parameters = 0;
            ct.num_ui_strings = 1;
            ct.ui_strings[0] = if i == 0 {
                "FALSE_COND".to_string()
            } else {
                "TRUE_COND".to_string()
            };
        } else if matches!(i, 7 | 17 | 18 | 39 | 40 | 41 | 42) {
            // TEAM_INSIDE_AREA_PARTIALLY (7), TEAM_INSIDE_AREA_ENTIRELY (17), TEAM_OUTSIDE_AREA_ENTIRELY (18),
            // TEAM_ENTERED_AREA_ENTIRELY (39), TEAM_ENTERED_AREA_PARTIALLY (40), TEAM_EXITED_AREA_ENTIRELY (41),
            // TEAM_EXITED_AREA_PARTIALLY (42)
            ct.num_parameters = 3;
            ct.parameters[0] = ParameterType::Team;
            ct.parameters[1] = ParameterType::TriggerArea;
            ct.parameters[2] = ParameterType::SurfacesAllowed;
            ct.num_ui_strings = 3;
            ct.ui_strings[0] = u0;
            ct.ui_strings[1] = u1;
            ct.ui_strings[2] = u2;
        } else if i == 84 {
            // SKIRMISH_SPECIAL_POWER_READY (84)
            ct.num_parameters = 2;
            ct.parameters[0] = ParameterType::Side;
            ct.parameters[1] = ParameterType::SpecialPower;
            ct.num_ui_strings = 3;
            ct.ui_strings[0] = u0;
            ct.ui_strings[1] = u1;
            ct.ui_strings[2] = u2;
        } else {
            let np = 1 + (i % 3);
            ct.num_parameters = np;
            for p in 0..np {
                ct.parameters[p as usize] =
                    ParameterType::from_i32((i * 7 + p * 13) % PARAMETER_NUM_ITEMS);
            }
            ct.num_ui_strings = np;
            ct.ui_strings[0] = u0;
            if np >= 2 {
                ct.ui_strings[1] = u1;
            }
            if np >= 3 {
                ct.ui_strings[2] = u2;
            }
        }
    }

    for i in 0..ACTION_NUM_ITEMS {
        let at = &mut reg.actions[i as usize];
        let iname = format!("ACT_INTERNAL_{i}");
        let u0 = format!("A{i}[");
        let u1 = ";".to_string();
        let u2 = "]".to_string();
        at.internal_name = iname.clone();
        at.internal_name_key = nkg.name_to_key(&iname);
        at.ui_name = iname;

        match i {
            5 | 250 | 254 => {
                // NO_OP (5), SKIRMISH_BUILD_BASE_DEFENSE_FRONT (250), SKIRMISH_BUILD_BASE_DEFENSE_FLANK (254)
                at.num_parameters = 0;
                at.num_ui_strings = 1;
                at.ui_strings[0] = u0;
            }
            102 | 146 | 229 | 230 => {
                // MAP_REVEAL_ALL (102), MAP_SHROUD_ALL (146), MAP_REVEAL_ALL_PERM (229), MAP_REVEAL_ALL_UNDO_PERM (230)
                at.num_parameters = 1;
                at.parameters[0] = ParameterType::Side;
                at.num_ui_strings = 1;
                at.ui_strings[0] = u0;
            }
            251 => {
                // SKIRMISH_FIRE_SPECIAL_POWER_AT_MOST_COST (251)
                at.num_parameters = 2;
                at.parameters[0] = ParameterType::Side;
                at.parameters[1] = ParameterType::SpecialPower;
                at.num_ui_strings = 2;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u2;
            }
            45 | 46 => {
                // NAMED_SET_ATTITUDE (45), TEAM_SET_ATTITUDE (46)
                at.num_parameters = 2;
                at.parameters[0] = if i == 45 {
                    ParameterType::Unit
                } else {
                    ParameterType::Team
                };
                at.parameters[1] = ParameterType::AiMood;
                at.num_ui_strings = 2;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u2;
            }
            83 => {
                // SPEECH_PLAY (83)
                at.num_parameters = 2;
                at.parameters[0] = ParameterType::Dialog;
                at.parameters[1] = ParameterType::Boolean;
                at.num_ui_strings = 2;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u2;
            }
            36 => {
                // TEAM_FOLLOW_WAYPOINTS (36)
                at.num_parameters = 3;
                at.parameters[0] = ParameterType::Team;
                at.parameters[1] = ParameterType::WaypointPath;
                at.parameters[2] = ParameterType::Boolean;
                at.num_ui_strings = 3;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u1;
                at.ui_strings[2] = u2;
            }
            90 | 145 => {
                // MAP_REVEAL_AT_WAYPOINT (90), MAP_SHROUD_AT_WAYPOINT (145)
                at.num_parameters = 3;
                at.parameters[0] = ParameterType::Waypoint;
                at.parameters[1] = ParameterType::Real;
                at.parameters[2] = ParameterType::Side;
                at.num_ui_strings = 3;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u1;
                at.ui_strings[2] = u2;
            }
            26 | 27 => {
                // CAMERA_MOD_SET_FINAL_ZOOM (26), CAMERA_MOD_SET_FINAL_PITCH (27)
                at.num_parameters = 3;
                at.parameters[0] = ParameterType::Real;
                at.parameters[1] = ParameterType::Percent;
                at.parameters[2] = ParameterType::Percent;
                at.num_ui_strings = 3;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u1;
                at.ui_strings[2] = u2;
            }
            18 | 19 | 117 | 118 => {
                // ROTATE_CAMERA (18), RESET_CAMERA (19), ZOOM_CAMERA (117), PITCH_CAMERA (118)
                at.num_parameters = 4;
                at.parameters[0] = ParameterType::Waypoint;
                at.parameters[1] = ParameterType::Real;
                at.parameters[2] = ParameterType::Real;
                at.parameters[3] = ParameterType::Real;
                at.num_ui_strings = 3;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u1;
                at.ui_strings[2] = u2;
            }
            14 | 17 | 181 | 206 => {
                // MOVE_CAMERA_TO (14), MOVE_CAMERA_ALONG_WAYPOINT_PATH (17),
                // CAMERA_LOOK_TOWARD_OBJECT (181), CAMERA_LOOK_TOWARD_WAYPOINT (206)
                at.num_parameters = 5;
                at.parameters[0] = ParameterType::Waypoint;
                at.parameters[1] = ParameterType::Real;
                at.parameters[2] = ParameterType::Real;
                at.parameters[3] = ParameterType::Real;
                at.parameters[4] = if i == 206 {
                    ParameterType::Boolean
                } else {
                    ParameterType::Real
                };
                at.num_ui_strings = 3;
                at.ui_strings[0] = u0;
                at.ui_strings[1] = u1;
                at.ui_strings[2] = u2;
            }
            _ => {
                let np = 1 + (i % 3);
                at.num_parameters = np;
                for p in 0..np {
                    at.parameters[p as usize] =
                        ParameterType::from_i32((i * 5 + p * 11) % PARAMETER_NUM_ITEMS);
                }
                at.num_ui_strings = np;
                at.ui_strings[0] = u0;
                if np >= 2 {
                    at.ui_strings[1] = u1;
                }
                if np >= 3 {
                    at.ui_strings[2] = u2;
                }
            }
        }
    }
}

fn populate_parameter(p: &mut Parameter, ptype_idx: Int, rng: &mut u32) {
    let pt = p.param_type;
    let r = splitmix32(rng);
    match pt {
        ParameterType::Coord3D => {
            let c = Coord3D {
                x: (((r % 2001) as Int) - 1000) as Real * 0.25,
                y: ((((r >> 11) % 2001) as Int) - 1000) as Real * 0.25,
                z: ((((r >> 21) % 401) as Int) - 200) as Real * 0.5,
            };
            p.set_coord3d(&c);
        }
        ParameterType::Real | ParameterType::Angle | ParameterType::Percent => {
            p.real_val = (((r % 1001) as Int) - 500) as Real * 0.125;
            p.int_val = 0;
        }
        ParameterType::Int | ParameterType::Color => {
            p.int_val = (r & 0x00FF_FFFF) as Int;
        }
        ParameterType::Boolean => {
            p.int_val = (r & 1) as Int;
        }
        ParameterType::Comparison => {
            p.int_val = (r % 6) as Int;
        }
        ParameterType::Relation => {
            p.int_val = (r % 3) as Int;
        }
        ParameterType::AiMood => {
            p.int_val = ((r % 5) as Int) - 2;
        }
        ParameterType::RadarEventType => {
            const RADAR_EVENTS: [Int; 5] = [1, 2, 3, 4, 6];
            p.int_val = RADAR_EVENTS[(r % 5) as usize];
        }
        ParameterType::LeftOrRight => {
            p.int_val = (r % 4) as Int;
        }
        ParameterType::Boundary => {
            p.int_val = (r % 16) as Int;
        }
        ParameterType::Buildable => {
            p.int_val = ((r % 5) as Int) - 1;
        }
        ParameterType::SurfacesAllowed => {
            p.int_val = (r % 5) as Int;
        }
        ParameterType::ShakeIntensity => {
            p.int_val = (r % 7) as Int;
        }
        ParameterType::KindOfParam => {
            p.int_val = (r % (KINDOF_COUNT as u32)) as Int;
            p.string_val = KindOfMaskType::get_name_from_single_bit(&KINDOF_NAMES, p.int_val)
                .unwrap_or("")
                .to_string();
        }
        ParameterType::ObjectStatus => {
            if (r % 7) == 0 {
                p.string_val.clear();
            } else {
                let bit = (r % (OBJECT_STATUS_COUNT as u32)) as usize;
                p.string_val = OBJECT_STATUS_NAMES[bit].to_string();
            }
        }
        ParameterType::ObjectType => {
            const OBJ_TYPES: [&str; 4] = [
                "AmericaTankCrusader",
                "FundamentalistTankScorpion",
                "ChinaTankOverlord",
                "GLATankScorpion",
            ];
            p.string_val =
                OBJ_TYPES[((r.wrapping_add(ptype_idx as u32)) % 4) as usize].to_string();
        }
        ParameterType::Upgrade => {
            const UPGRADES: [&str; 5] = [
                "Upgrade_AmericaRangerCaptureBuilding",
                "Upgrade_ChinaRedguardCaptureBuilding",
                "Upgrade_GLARebelCaptureBuilding",
                "Upgrade_InfantryCaptureBuilding",
                "Upgrade_ChinaNuclearTanks",
            ];
            p.string_val = UPGRADES[(r % 5) as usize].to_string();
        }
        ParameterType::Side => {
            const SIDES: [&str; 4] = ["PlyrAmerica", "PlyrChina", "PlyrGLA", THIS_PLAYER];
            p.string_val = SIDES[(r % 4) as usize].to_string();
        }
        ParameterType::Team => {
            const TEAMS: [&str; 4] = [THIS_TEAM, ANY_TEAM, "teamPlyrAmerica", "AttackTeam1"];
            p.string_val = TEAMS[(r % 4) as usize].to_string();
        }
        _ => {
            if (r % 9) == 0 {
                p.string_val.clear();
            } else {
                p.string_val = format!("P_{}_{}", pt.as_i32(), r % 100);
            }
        }
    }
}

fn hash_scripts_slice(
    mut h: u32,
    scripts: &[Script],
    provider: &dyn ScriptTemplateProvider,
) -> u32 {
    for s in scripts {
        h = rot_crc(h, crc_str(&s.script_name));
        h = rot_crc(h, crc_str(&s.comment));
        h = rot_crc(h, crc_str(&s.condition_comment));
        h = rot_crc(h, crc_str(&s.action_comment));
        h = rot_crc(h, s.is_active as u32);
        h = rot_crc(h, s.is_one_shot as u32);
        h = rot_crc(h, s.easy as u32);
        h = rot_crc(h, s.normal as u32);
        h = rot_crc(h, s.hard as u32);
        h = rot_crc(h, s.is_subroutine as u32);
        h = rot_crc(h, s.delay_evaluation_seconds as u32);
        h = rot_crc(h, crc_str(&s.get_ui_text(provider)));
        for orc in &s.or_conditions {
            for c in &orc.conditions {
                h = rot_crc(h, c.condition_type as u32);
                h = rot_crc(h, c.parms.len() as u32);
                for pm in &c.parms {
                    h = rot_crc(h, pm.param_type.as_i32() as u32);
                    h = rot_crc(h, pm.int_val as u32);
                    h = rot_crc(h, pm.real_val.to_bits());
                    h = rot_crc(h, crc_str(&pm.string_val));
                    if pm.param_type == ParameterType::Coord3D {
                        let pos = pm.get_coord3d();
                        h = rot_crc(h, pos.x.to_bits());
                        h = rot_crc(h, pos.y.to_bits());
                        h = rot_crc(h, pos.z.to_bits());
                    }
                    if pm.param_type == ParameterType::ObjectStatus {
                        for bi in 0..OBJECT_STATUS_COUNT {
                            if pm.object_status.test(bi as Int) != FALSE {
                                h = rot_crc(h, (bi + 100) as u32);
                            }
                        }
                    }
                }
            }
        }
        for branch in [&s.actions, &s.actions_false] {
            for a in branch {
                h = rot_crc(h, a.action_type as u32);
                h = rot_crc(h, a.parms.len() as u32);
                for pm in &a.parms {
                    h = rot_crc(h, pm.param_type.as_i32() as u32);
                    h = rot_crc(h, pm.int_val as u32);
                    h = rot_crc(h, pm.real_val.to_bits());
                    h = rot_crc(h, crc_str(&pm.string_val));
                }
            }
        }
    }
    h
}

fn hash_script_list_tree(list: &ScriptList, provider: &dyn ScriptTemplateProvider) -> u32 {
    let mut h = hash_scripts_slice(0x811c_9dc5_u32, &list.scripts, provider);
    for g in &list.groups {
        h = rot_crc(h, crc_str(&g.group_name));
        h = rot_crc(h, g.is_group_active as u32);
        h = rot_crc(h, g.is_group_subroutine as u32);
        h = hash_scripts_slice(h, &g.scripts, provider);
    }
    h
}

#[test]
fn test_pkt434_oracle_structure_and_constants() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-434\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    for required in ["Scripts.h", "Scripts.cpp", "BorderColors.h", "ShellHooks.h"] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }

    assert_eq!(MAX_PARMS, 12);
    assert_eq!(PARAMETER_NUM_ITEMS, 52);
    assert_eq!(CONDITION_NUM_ITEMS, 109);
    assert_eq!(ACTION_NUM_ITEMS, 344);
    assert_eq!(SHELL_SCRIPT_HOOK_TOTAL, 29);
    assert_eq!(BORDER_COLORS_SIZE, 8);
    assert_eq!(SURFACES.len(), 3);
    assert_eq!(SHAKE_INTENSITIES.len(), 6);
    assert_eq!(THE_OBJECT_FLAGS_NAMES.len(), 7);
    assert_eq!(THE_SHELL_HOOK_NAMES.len(), 29);
    assert_eq!(BORDER_COLORS[0].color_name, "Orange");
    assert_eq!(BORDER_COLORS[7].color_name, "Pink");

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
#[ignore = "PKT-434: remove when Scripts is implemented"]
fn test_pkt434_legacy_parameter_and_action_healing_boundaries() {
    let mut nkg = NameKeyGenerator::new();
    nkg.init();
    let mut reg = ScriptTemplateRegistry::new();
    init_deterministic_templates(&mut nkg, &mut reg);

    // Parameter qualify rules
    let mut p_side = Parameter::new(ParameterType::Side, 0);
    p_side.string_val = "PlyrAmerica".to_string();
    p_side.qualify("_Q", "PlyrAmerica_Q", "Player2");
    assert_eq!(p_side.string_val, "Player2");

    let mut p_team_this = Parameter::new(ParameterType::Team, 0);
    p_team_this.string_val = THIS_TEAM.to_string();
    p_team_this.qualify("_Q", "PlyrAmerica_Q", "Player2");
    assert_eq!(p_team_this.string_val, THIS_TEAM);

    let mut p_team_other = Parameter::new(ParameterType::Team, 0);
    p_team_other.string_val = "TeamAlpha".to_string();
    p_team_other.qualify("_Q", "PlyrAmerica_Q", "Player2");
    assert_eq!(p_team_other.string_val, "TeamAlpha_Q");
}

#[test]
#[ignore = "PKT-434: remove when Scripts is implemented"]
fn test_pkt434_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);

        let mut rng = seed.wrapping_mul(0x7477_9640).wrapping_add(0x9e37_79b9);

        let mut nkg = NameKeyGenerator::new();
        nkg.init();
        let mut reg = ScriptTemplateRegistry::new();
        init_deterministic_templates(&mut nkg, &mut reg);

        let sweep_pt = ParameterType::from_i32(
            ((seed.wrapping_add(splitmix32(&mut rng) % 7)) % (PARAMETER_NUM_ITEMS as u32)) as Int,
        );
        assert_eq!(sweep_pt.as_i32(), get_i32(line, "sweep_pt").unwrap());
        let mut sweep_param = Parameter::new(sweep_pt, 0);
        populate_parameter(&mut sweep_param, seed as Int, &mut rng);
        let sweep_ui = sweep_param.get_ui_text();
        assert_eq!(
            crc_str(&sweep_ui),
            get_u32(line, "param_ui_crc").unwrap(),
            "seed {seed} param_ui_crc ({sweep_pt:?} -> {sweep_ui})"
        );

        let num_lists = (1 + (seed % 2)) as usize;
        assert_eq!(num_lists as i32, get_i32(line, "num_lists").unwrap());
        let mut lists = Vec::with_capacity(num_lists);

        for li in 0..num_lists {
            let mut sl = ScriptList::new();
            let num_top_scripts = 1 + (splitmix32(&mut rng) % 2) as usize;
            for si in 0..num_top_scripts {
                let mut scr = Script::new();
                scr.script_name = format!("Scr_{seed}_{li}_{si}");
                scr.comment = format!("Cmt_{seed}_{si}");
                scr.condition_comment = "CondCmt".to_string();
                scr.action_comment = "ActCmt".to_string();
                scr.is_active = if (splitmix32(&mut rng) & 1) != 0 {
                    TRUE
                } else {
                    FALSE
                };
                scr.is_one_shot = if (splitmix32(&mut rng) & 1) != 0 {
                    TRUE
                } else {
                    FALSE
                };
                scr.easy = if (splitmix32(&mut rng) & 1) != 0 {
                    TRUE
                } else {
                    FALSE
                };
                scr.normal = if (splitmix32(&mut rng) & 1) != 0 {
                    TRUE
                } else {
                    FALSE
                };
                scr.hard = if (splitmix32(&mut rng) & 1) != 0 {
                    TRUE
                } else {
                    FALSE
                };
                scr.is_subroutine = if (splitmix32(&mut rng) & 1) != 0 {
                    TRUE
                } else {
                    FALSE
                };
                scr.delay_evaluation_seconds = (splitmix32(&mut rng) % 30) as Int;
                scr.frame_to_evaluate_at = splitmix32(&mut rng) % 1000;

                let num_ors = 1 + (splitmix32(&mut rng) % 2) as usize;
                for oi in 0..num_ors {
                    let mut orc = OrCondition::new();
                    let num_ands = 1 + (splitmix32(&mut rng) % 2) as usize;
                    for ai in 0..num_ands {
                        let ct = ((seed * 11 + (li * 7 + si * 5 + oi * 3 + ai) as u32)
                            % (CONDITION_NUM_ITEMS as u32)) as Int;
                        let mut cond = Condition::new(ct, &reg);
                        for pi in 0..cond.parms.len() {
                            populate_parameter(&mut cond.parms[pi], pi as Int, &mut rng);
                        }
                        orc.conditions.push(cond);
                    }
                    scr.or_conditions.push(orc);
                }

                let num_acts = 1 + (splitmix32(&mut rng) % 2) as usize;
                for ai in 0..num_acts {
                    let at = ((seed * 17 + (li * 13 + si * 7 + ai * 3) as u32)
                        % (ACTION_NUM_ITEMS as u32)) as Int;
                    let mut act = ScriptAction::new(at, &reg);
                    for pi in 0..act.parms.len() {
                        populate_parameter(&mut act.parms[pi], pi as Int, &mut rng);
                    }
                    scr.actions.push(act);
                }

                if (splitmix32(&mut rng) & 1) != 0 {
                    let fat = ((seed * 23 + (si + 1) as u32) % (ACTION_NUM_ITEMS as u32)) as Int;
                    let mut fact = ScriptAction::new(fat, &reg);
                    for pi in 0..fact.parms.len() {
                        populate_parameter(&mut fact.parms[pi], pi as Int, &mut rng);
                    }
                    scr.actions_false.push(fact);
                }

                let insert_ndx = (splitmix32(&mut rng) % 3) as Int;
                sl.add_script(scr, insert_ndx);
            }

            let mut grp = ScriptGroup::new();
            grp.group_name = format!("Grp_{seed}_{li}");
            grp.is_group_active = if (splitmix32(&mut rng) & 1) != 0 {
                TRUE
            } else {
                FALSE
            };
            grp.is_group_subroutine = if (splitmix32(&mut rng) & 1) != 0 {
                TRUE
            } else {
                FALSE
            };

            let mut gscr = Script::new();
            gscr.script_name = format!("GScr_{seed}_{li}");
            gscr.is_active = if (splitmix32(&mut rng) & 1) != 0 {
                TRUE
            } else {
                FALSE
            };
            gscr.is_one_shot = if (splitmix32(&mut rng) & 1) != 0 {
                TRUE
            } else {
                FALSE
            };

            let mut gor = OrCondition::new();
            let gct = ((seed * 31 + li as u32) % (CONDITION_NUM_ITEMS as u32)) as Int;
            let mut gcond = Condition::new(gct, &reg);
            for pi in 0..gcond.parms.len() {
                populate_parameter(&mut gcond.parms[pi], pi as Int, &mut rng);
            }
            gor.conditions.push(gcond);
            gscr.or_conditions.push(gor);

            let gat = ((seed * 37 + li as u32) % (ACTION_NUM_ITEMS as u32)) as Int;
            let mut gact = ScriptAction::new(gat, &reg);
            for pi in 0..gact.parms.len() {
                populate_parameter(&mut gact.parms[pi], pi as Int, &mut rng);
            }
            gscr.actions.push(gact);

            grp.add_script(gscr, 0);
            sl.add_group(grp, 0);
            lists.push(sl);
        }

        let mut chunk_bytes = Vec::new();
        {
            let mut chunk_out = DataChunkOutput::new(&mut chunk_bytes);
            ScriptList::write_scripts_data_chunk(&mut chunk_out, &mut lists, &mut nkg, &reg)
                .unwrap();
            chunk_out.finish().unwrap();
        }

        let mut stream = CachedFileInputStream::from_bytes(&chunk_bytes);
        let mut read_lists =
            ScriptList::parse_scripts_data_chunk(&mut stream, &mut nkg, &reg).unwrap();
        assert_eq!(read_lists.len(), num_lists, "seed {seed} read_lists len");

        assert_eq!(
            hash_script_list_tree(&lists[0], &reg),
            get_u32(line, "tree_hash_orig").unwrap(),
            "seed {seed} tree_hash_orig"
        );
        assert_eq!(
            hash_script_list_tree(&read_lists[0], &reg),
            get_u32(line, "tree_hash_parsed").unwrap(),
            "seed {seed} tree_hash_parsed"
        );

        let mut reparsed_bytes = Vec::new();
        {
            let mut chunk_out2 = DataChunkOutput::new(&mut reparsed_bytes);
            ScriptList::write_scripts_data_chunk(&mut chunk_out2, &mut read_lists, &mut nkg, &reg)
                .unwrap();
            chunk_out2.finish().unwrap();
        }
        assert_eq!(
            reparsed_bytes.len() as u32,
            get_u32(line, "chunk_size").unwrap(),
            "seed {seed} chunk_size"
        );
        let mut reparsed_crc = Crc::new();
        reparsed_crc.compute_crc(&reparsed_bytes, reparsed_bytes.len() as Int);
        assert_eq!(
            reparsed_crc.get(),
            get_u32(line, "reparsed_chunk_crc").unwrap(),
            "seed {seed} reparsed_chunk_crc"
        );

        let mut dup_list = read_lists[0].duplicate(&reg);
        assert_eq!(
            hash_script_list_tree(&dup_list, &reg),
            get_u32(line, "tree_hash_parsed").unwrap(),
            "seed {seed} dup_list hash"
        );

        let qual_list =
            read_lists[0].duplicate_and_qualify("_Q", "PlyrAmerica_Q", "Player2", &reg);
        assert_eq!(
            hash_script_list_tree(&qual_list, &reg),
            get_u32(line, "tree_hash_qual").unwrap(),
            "seed {seed} tree_hash_qual"
        );

        for s in &mut read_lists[0].scripts {
            s.is_active = Bool::from(s.is_active == FALSE);
        }
        for g in &mut read_lists[0].groups {
            g.is_group_active = Bool::from(g.is_group_active == FALSE);
            for s in &mut g.scripts {
                s.is_active = Bool::from(s.is_active == FALSE);
            }
        }

        let exp_xfer_bytes = decode_hex_bytes(get_str(line, "xfer_hex").unwrap()).unwrap();
        let mut xsave = XferSave::new();
        xsave.xfer_snapshot(&mut read_lists[0]).unwrap();
        let got_xfer_bytes = xsave.into_bytes();
        assert_eq!(got_xfer_bytes, exp_xfer_bytes, "seed {seed} xfer_hex");

        let mut xcrc = XferCrc::new();
        xcrc.xfer_snapshot(&mut read_lists[0]).unwrap();
        assert_eq!(
            xcrc.get_crc(),
            get_u32(line, "xfer_crc").unwrap(),
            "seed {seed} xfer_crc"
        );

        let mut xload = XferLoad::new(&got_xfer_bytes);
        xload.xfer_snapshot(&mut dup_list).unwrap();
        assert_eq!(
            hash_script_list_tree(&dup_list, &reg),
            hash_script_list_tree(&read_lists[0], &reg),
            "seed {seed} xload active state roundtrip"
        );
    }
}
