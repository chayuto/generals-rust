//! Gate V2-Golden test suite for `WP-3.1b` Economy/Stats Leaves
//! (`BattleHonors`, `ScoreKeeper`, `SkirmishBattleHonors`, and `StatsCollector`,
//!  1,000 C++ reference vectors).

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32,
};
use zh_common::types::{Bool, Int, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferLoad, XferSave};
use zh_logic::economy::battle_honors::*;
use zh_logic::economy::score_keeper::{ScoreKeeper, ScoreObjectView, ScoreTemplateDesc};
use zh_logic::economy::skirmish_battle_honors::{
    MemoryPreferencesStorage, SkirmishBattleHonors,
};
use zh_logic::economy::stats_collector::{
    MemoryStatsSink, StatsCollector, StatsContext, StatsObjectInfo,
};
use zh_logic::gamelogic::global_data::GameDifficulty;
use zh_logic::gamelogic::message_stream::GameMessageType;
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};

const WP3_1B_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_1b_Stats.json");

#[inline]
fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

fn unescape_json_str(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn get_json_string(line: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{key}\":\"");
    let start = line.find(&pattern)? + pattern.len();
    let bytes = line.as_bytes();
    let mut i = start;
    while i < bytes.len() {
        if bytes[i] == b'\\' {
            i += 2;
            continue;
        }
        if bytes[i] == b'"' {
            return Some(unescape_json_str(&line[start..i]));
        }
        i += 1;
    }
    None
}

fn make_kind_of(k1: KindOfType, k2: Option<KindOfType>) -> KindOfMaskType {
    let mut m = KindOfMaskType::default();
    if (k1 as Int) >= 0 {
        m.set(k1 as Int, 1);
    }
    if let Some(k) = k2 {
        if (k as Int) >= 0 {
            m.set(k as Int, 1);
        }
    }
    m
}

fn build_template_catalog() -> Vec<ScoreTemplateDesc> {
    vec![
        ScoreTemplateDesc {
            id: 0,
            name: "Tmpl_Bld_Score".to_string(),
            kind_of: make_kind_of(KindOfType::Structure, Some(KindOfType::Score)),
            equivalent_root_id: 0,
        },
        ScoreTemplateDesc {
            id: 1,
            name: "Tmpl_Bld_Create".to_string(),
            kind_of: make_kind_of(KindOfType::Structure, Some(KindOfType::ScoreCreate)),
            equivalent_root_id: 1,
        },
        ScoreTemplateDesc {
            id: 2,
            name: "Tmpl_Bld_Destroy".to_string(),
            kind_of: make_kind_of(KindOfType::Structure, Some(KindOfType::ScoreDestroy)),
            equivalent_root_id: 2,
        },
        ScoreTemplateDesc {
            id: 3,
            name: "Tmpl_Bld_Tech".to_string(),
            kind_of: make_kind_of(KindOfType::Structure, Some(KindOfType::TechBuilding)),
            equivalent_root_id: 3,
        },
        ScoreTemplateDesc {
            id: 4,
            name: "Tmpl_Inf_Score".to_string(),
            kind_of: make_kind_of(KindOfType::Infantry, Some(KindOfType::Score)),
            equivalent_root_id: 4,
        },
        ScoreTemplateDesc {
            id: 5,
            name: "Tmpl_Veh_ScoreCreate".to_string(),
            kind_of: make_kind_of(KindOfType::Vehicle, Some(KindOfType::ScoreCreate)),
            equivalent_root_id: 5,
        },
        ScoreTemplateDesc {
            id: 6,
            name: "Tmpl_Veh_ScoreDest".to_string(),
            kind_of: make_kind_of(KindOfType::Vehicle, Some(KindOfType::ScoreDestroy)),
            equivalent_root_id: 6,
        },
        ScoreTemplateDesc {
            id: 7,
            name: "Tmpl_Air_Score".to_string(),
            kind_of: make_kind_of(KindOfType::Aircraft, Some(KindOfType::Score)),
            equivalent_root_id: 7,
        },
    ]
}

fn difficulty_from_i32(d: Int) -> GameDifficulty {
    match d {
        0 => GameDifficulty::Easy,
        1 => GameDifficulty::Normal,
        2 => GameDifficulty::Hard,
        _ => panic!("invalid difficulty {d}"),
    }
}

fn msg_type_from_i32(t: Int) -> GameMessageType {
    match t {
        1047 => GameMessageType::MsgQueueUnitCreate,
        1049 => GameMessageType::MsgDozerConstruct,
        1050 => GameMessageType::MsgDozerConstructLine,
        1068 => GameMessageType::MsgDoMoveto,
        1059 => GameMessageType::MsgDoAttackObject,
        _ => panic!("unexpected msg type {t}"),
    }
}

#[test]
fn test_wp3_1b_stats_seed0_diagnostics() {
    assert_eq!(BATTLE_HONOR_LADDER_CHAMP, 1);
    assert_eq!(BATTLE_HONOR_STREAK, 2);
    assert_eq!(BATTLE_HONOR_LOYALTY_USA, 32);
    assert_eq!(BATTLE_HONOR_LOYALTY_CHINA, 64);
    assert_eq!(BATTLE_HONOR_BATTLE_TANK, 128);
    assert_eq!(BATTLE_HONOR_AIR_WING, 256);
    assert_eq!(BATTLE_HONOR_LOYALTY_GLA, 512);
    assert_eq!(BATTLE_HONOR_ENDURANCE, 1024);
    assert_eq!(BATTLE_HONOR_CAMPAIGN_USA, 2048);
    assert_eq!(BATTLE_HONOR_CAMPAIGN_CHINA, 4096);
    assert_eq!(BATTLE_HONOR_CAMPAIGN_GLA, 8192);
    assert_eq!(BATTLE_HONOR_BLITZ5, 16384);
    assert_eq!(BATTLE_HONOR_BLITZ10, 32768);
    assert_eq!(BATTLE_HONOR_FAIR_PLAY, 65536);
    assert_eq!(BATTLE_HONOR_APOCALYPSE, 131072);
    assert_eq!(BATTLE_HONOR_OFFICERSCLUB, 262144);
    assert_eq!(BATTLE_HONOR_DOMINATION, 524288);
    assert_eq!(BATTLE_HONOR_CHALLENGE_MODE, 1048576);
    assert_eq!(BATTLE_HONOR_ULTIMATE, 2097152);
    assert_eq!(BATTLE_HONOR_GLOBAL_GENERAL, 4194304);
    assert_eq!(BATTLE_HONOR_DOMINATION_ONLINE, 8388608);
    assert_eq!(BATTLE_HONOR_STREAK_ONLINE, 16777216);
    assert_eq!(BATTLE_HONOR_CHALLENGE, 33554432);
    assert_eq!(BATTLE_HONOR_NOT_GAINED, 134217728);
    assert_eq!(BH_CHALLENGE_MASK_1, 1);
    assert_eq!(BH_CHALLENGE_MASK_7, 64);
    assert_eq!(MAX_BATTLE_HONOR_COLUMNS, 4);
    assert_eq!(MAX_BATTLE_HONOR_IMAGE_WIDTH, 40);
    assert_eq!(MAX_BATTLE_HONOR_IMAGE_HEIGHT, 41);

    assert!(WP3_1B_JSON.contains("\"vector_count\": 1000"));
    assert!(WP3_1B_JSON.contains("\"source_sha256\":"));
    assert!(WP3_1B_JSON.contains("\"seed_0_diagnostics\":"));
}

#[test]
fn test_wp3_1b_stats_1000_golden_vectors() {
    let lines = extract_vector_lines(WP3_1B_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 vectors in WP3_1b_Stats.json");

    let catalog = build_template_catalog();
    let maps = [
        "Tournament Desert",
        "Twilight Flame",
        "Fallen Empire",
        "Lone Eagle",
    ];
    let gens = [
        "AmericaAirForceGeneral",
        "ChinaTankGeneral",
        "GLAToxinGeneral",
        "America",
        "",
    ];

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed");
        assert_eq!(seed as usize, idx);

        // ---------------- 1. BattleHonors (PKT-117) ----------------
        let bh_raw = get_u32(line, "bh_raw").expect("bh_raw");
        let bh_flags = BattleHonorFlags::new(bh_raw).mask_defined();
        assert_eq!(bh_flags.bits(), get_u32(line, "bh_masked").expect("bh_masked"), "seed {seed}");
        let expected_chal = get_u32(line, "bh_chal").expect("bh_chal");
        assert_eq!(expected_chal & !BH_CHALLENGE_ALL_MASK, 0, "seed {seed}");

        // ---------------- 2. SkirmishBattleHonors (PKT-123) ----------------
        let s_in = get_i32_vec(line, "s_in").expect("s_in");
        assert_eq!(s_in.len(), 18);
        let mut storage = MemoryPreferencesStorage::default();
        if seed == 0 {
            let pre = concat!(
                "  Wins = 7 \n",
                "SCUD = YES\n",
                "PPC = TrUe\n",
                "Nuke = 0\n",
                "LastHouse = AmericaAirForceGeneral\n",
                "EmptySkip = \n",
            );
            storage.files.insert("SkirmishStats.ini".to_string(), pre.to_string());
        }

        let mut sbh = SkirmishBattleHonors::from_storage(1, &storage);
        if seed == 0 {
            assert_eq!(sbh.get_wins(), 7);
            assert_eq!(sbh.built_scud(), TRUE);
            assert_eq!(sbh.built_particle_cannon(), TRUE);
            assert_eq!(sbh.built_nuke(), FALSE);
        }

        sbh.set_wins(s_in[0]);
        sbh.set_losses(s_in[1]);
        sbh.set_win_streak(s_in[2]);
        sbh.set_best_win_streak(s_in[3]);
        sbh.set_challenge_medals(s_in[4]);
        sbh.set_num_games_loyal(s_in[5]);
        let s_super = s_in[6] as u32;
        if (s_super & 1) != 0 {
            sbh.set_built_scud();
        }
        if (s_super & 2) != 0 {
            sbh.set_built_particle_cannon();
        }
        if (s_super & 4) != 0 {
            sbh.set_built_nuke();
        }
        sbh.set_honors(s_in[7]);
        sbh.set_honors(s_in[8]);
        let end_diff = s_in[9];
        let end_ais = s_in[10];
        let map_idx = s_in[11] as usize;
        let gen_idx = s_in[12] as usize;
        sbh.set_endurance_medal(maps[map_idx], end_diff, end_ais);
        sbh.set_last_general(gens[gen_idx]);
        let usa_m = s_in[13] as u32;
        let chi_m = s_in[14] as u32;
        let gla_m = s_in[15] as u32;
        for d in 0..3 {
            if (usa_m & (1 << d)) != 0 {
                sbh.set_usa_campaign_complete(difficulty_from_i32(d));
            }
            if (chi_m & (1 << d)) != 0 {
                sbh.set_china_campaign_complete(difficulty_from_i32(d));
            }
            if (gla_m & (1 << d)) != 0 {
                sbh.set_gla_campaign_complete(difficulty_from_i32(d));
            }
        }
        let chal_idx = s_in[16];
        let chal_diff = difficulty_from_i32(s_in[17]);
        sbh.set_challenge_campaign_complete(chal_idx, chal_diff);

        assert_eq!(sbh.write_to_storage(&mut storage), TRUE, "seed {seed}");
        let expected_ini = get_json_string(line, "s_ini").expect("s_ini");
        assert_eq!(
            storage.files.get("SkirmishStats.ini").map(|s| s.as_str()),
            Some(expected_ini.as_str()),
            "seed {seed} ini mismatch"
        );

        let sbh2 = SkirmishBattleHonors::from_storage(1, &storage);
        let s_out = get_i32_vec(line, "s_out").expect("s_out");
        assert_eq!(s_out.len(), 11);
        assert_eq!(sbh2.get_wins(), s_out[0], "seed {seed}");
        assert_eq!(sbh2.get_losses(), s_out[1], "seed {seed}");
        assert_eq!(sbh2.get_win_streak(), s_out[2], "seed {seed}");
        assert_eq!(sbh2.get_best_win_streak(), s_out[3], "seed {seed}");
        assert_eq!(sbh2.get_challenge_medals(), s_out[4], "seed {seed}");
        assert_eq!(sbh2.built_scud() as Int, s_out[5], "seed {seed}");
        assert_eq!(sbh2.built_particle_cannon() as Int, s_out[6], "seed {seed}");
        assert_eq!(sbh2.built_nuke() as Int, s_out[7], "seed {seed}");
        assert_eq!(sbh2.get_honors(), s_out[8], "seed {seed}");
        assert_eq!(sbh2.get_endurance_medal(maps[map_idx], end_diff), s_out[9], "seed {seed}");
        assert_eq!(sbh2.get_num_games_loyal(), s_out[10], "seed {seed}");

        let mut camp_bits: u32 = 0;
        for d in 0..3 {
            let diff = difficulty_from_i32(d);
            if sbh2.get_usa_campaign_complete(diff) != FALSE {
                camp_bits |= 1 << d;
            }
            if sbh2.get_china_campaign_complete(diff) != FALSE {
                camp_bits |= 1 << (d + 3);
            }
            if sbh2.get_gla_campaign_complete(diff) != FALSE {
                camp_bits |= 1 << (d + 6);
            }
        }
        if sbh2.get_challenge_campaign_complete(chal_idx, chal_diff) != FALSE {
            camp_bits |= 1 << 9;
        }
        assert_eq!(camp_bits, get_u32(line, "s_camp").expect("s_camp"), "seed {seed}");
        let expected_last = get_json_string(line, "s_last").expect("s_last");
        assert_eq!(sbh2.get_last_general(), expected_last, "seed {seed}");

        // ---------------- 3. ScoreKeeper (PKT-122) ----------------
        let sk_player = get_i32(line, "sk_player").expect("sk_player");
        let sk_ops = get_i32_vec(line, "sk_ops").expect("sk_ops");
        assert_eq!(sk_ops.len(), 64);

        let mut sk = ScoreKeeper::new();
        sk.reset(sk_player);
        let mut scoring_enabled: Bool = TRUE;
        let mut sk_tcrc: u32 = 0;

        for step in 0..16 {
            let kind = sk_ops[step * 4];
            let a0 = sk_ops[step * 4 + 1];
            let a1 = sk_ops[step * 4 + 2];
            let a2 = sk_ops[step * 4 + 3];

            match kind {
                0 => {
                    scoring_enabled = if a0 != 0 { TRUE } else { FALSE };
                }
                1 => sk.add_money_earned(a0),
                2 => sk.add_money_spent(a0),
                3..=7 => {
                    let tmpl = &catalog[a0 as usize];
                    let owner = if kind == 6 { a1 } else { sk_player };
                    let under_constr = if (kind == 6 && a2 != 0) || (kind == 7 && a1 != 0) {
                        TRUE
                    } else {
                        FALSE
                    };
                    let view = ScoreObjectView {
                        template: tmpl,
                        controlling_player_idx: owner,
                        under_construction: under_constr,
                    };
                    match kind {
                        3 => sk.add_object_built(&view, scoring_enabled),
                        4 => sk.remove_object_built(&view, scoring_enabled),
                        5 => sk.add_object_captured(&view, scoring_enabled),
                        6 => sk.add_object_destroyed(&view, scoring_enabled),
                        7 => sk.add_object_lost(&view, scoring_enabled),
                        _ => unreachable!(),
                    }
                }
                _ => panic!("unknown sk_op {kind}"),
            }

            sk_tcrc = rot_crc(sk_tcrc, sk.calculate_score() as u32);
            sk_tcrc = rot_crc(sk_tcrc, sk.total_units_built as u32);
            sk_tcrc = rot_crc(sk_tcrc, sk.total_buildings_built as u32);
            sk_tcrc = rot_crc(sk_tcrc, sk.get_total_units_destroyed() as u32);
            sk_tcrc = rot_crc(sk_tcrc, sk.get_total_buildings_destroyed() as u32);
            sk_tcrc = rot_crc(sk_tcrc, sk.total_units_lost as u32);
            sk_tcrc = rot_crc(sk_tcrc, sk.total_buildings_lost as u32);
        }

        assert_eq!(sk_tcrc, get_u32(line, "sk_tcrc").expect("sk_tcrc"), "seed {seed}");
        assert_eq!(sk.calculate_score(), get_i32(line, "sk_score").expect("sk_score"), "seed {seed}");

        let sk_stats = get_i32_vec(line, "sk_stats").expect("sk_stats");
        assert_eq!(sk_stats.len(), 13);
        assert_eq!(sk.total_money_earned, sk_stats[0], "seed {seed}");
        assert_eq!(sk.total_money_spent, sk_stats[1], "seed {seed}");
        assert_eq!(sk.total_units_built, sk_stats[2], "seed {seed}");
        assert_eq!(sk.total_units_lost, sk_stats[3], "seed {seed}");
        assert_eq!(sk.total_buildings_built, sk_stats[4], "seed {seed}");
        assert_eq!(sk.total_buildings_lost, sk_stats[5], "seed {seed}");
        assert_eq!(sk.get_total_units_destroyed(), sk_stats[6], "seed {seed}");
        assert_eq!(sk.get_total_buildings_destroyed(), sk_stats[7], "seed {seed}");
        assert_eq!(sk.total_tech_buildings_captured, sk_stats[8], "seed {seed}");
        assert_eq!(sk.total_faction_buildings_captured, sk_stats[9], "seed {seed}");

        let mask_veh_valid = make_kind_of(KindOfType::Vehicle, None);
        let mask_veh_invalid = make_kind_of(KindOfType::ScoreDestroy, None);
        assert_eq!(
            sk.get_total_units_built_masked(&mask_veh_valid, &mask_veh_invalid),
            sk_stats[10],
            "seed {seed}"
        );
        assert_eq!(sk.get_total_objects_built(&catalog[0]), sk_stats[11], "seed {seed}");
        assert_eq!(sk.get_total_objects_built(&catalog[4]), sk_stats[12], "seed {seed}");

        let expected_sk_save = decode_hex_bytes(get_str(line, "sk_save_hex").expect("sk_save_hex"))
            .expect("hex decode sk_save_hex");
        let mut sk_save = XferSave::new();
        sk_save.xfer_snapshot(&mut sk).expect("sk_save");
        assert_eq!(sk_save.as_bytes(), expected_sk_save.as_slice(), "seed {seed}");

        let mut sk_loaded = ScoreKeeper::new();
        let mut sk_load = XferLoad::new(&expected_sk_save);
        sk_loaded
            .xfer_with_resolver(&mut sk_load, |name| {
                catalog.iter().find(|t| t.name == name).cloned()
            })
            .expect("sk_load");
        assert_eq!(sk_loaded, sk, "seed {seed}");

        let mut sk_resave = XferSave::new();
        sk_resave.xfer_snapshot(&mut sk_loaded).expect("sk_resave");
        assert_eq!(sk_resave.as_bytes(), expected_sk_save.as_slice(), "seed {seed}");

        // ---------------- 4. StatsCollector (PKT-124) ----------------
        let sc_play_stats = get_i32(line, "sc_play_stats").expect("sc_play_stats");
        let sc_local_p = get_i32(line, "sc_local_p").expect("sc_local_p");
        let sc_money = get_u32(line, "sc_money").expect("sc_money");
        let sc_start_f = get_u32(line, "sc_start_f").expect("sc_start_f");
        let sc_end_f = get_u32(line, "sc_end_f").expect("sc_end_f");
        let sc_ops = get_i32_vec(line, "sc_ops").expect("sc_ops");
        assert_eq!(sc_ops.len(), 48);

        let k_inf = make_kind_of(KindOfType::Infantry, Some(KindOfType::Score));
        let k_veh1 = make_kind_of(KindOfType::Vehicle, Some(KindOfType::ScoreCreate));
        let k_veh2 = make_kind_of(KindOfType::Vehicle, Some(KindOfType::ScoreDestroy));
        let k_bld = make_kind_of(KindOfType::Structure, Some(KindOfType::Score));

        let local_side = if sc_local_p % 2 == 0 { "China" } else { "America" };
        let world_objects = [
            StatsObjectInfo {
                kind_of: &k_inf,
                is_neutral_controlled: false,
                controlling_side: local_side,
                is_local_player: true,
            },
            StatsObjectInfo {
                kind_of: &k_veh1,
                is_neutral_controlled: false,
                controlling_side: local_side,
                is_local_player: true,
            },
            StatsObjectInfo {
                kind_of: &k_veh2,
                is_neutral_controlled: false,
                controlling_side: "China",
                is_local_player: false,
            },
            StatsObjectInfo {
                kind_of: &k_inf,
                is_neutral_controlled: true,
                controlling_side: "",
                is_local_player: false,
            },
            StatsObjectInfo {
                kind_of: &k_inf,
                is_neutral_controlled: false,
                controlling_side: "Civilian",
                is_local_player: false,
            },
            StatsObjectInfo {
                kind_of: &k_bld,
                is_neutral_controlled: false,
                controlling_side: "America",
                is_local_player: false,
            },
        ];

        let mut sc = StatsCollector::new(sc_start_f);
        let mut sink = MemoryStatsSink::default();
        let init_ctx = StatsContext {
            frame: sc_start_f,
            play_stats: sc_play_stats,
            map_name: "Maps/Tournament Desert/Tournament Desert.map",
            local_player_side: local_side,
            local_player_money: sc_money,
        };
        sc.reset(&init_ctx, &world_objects, "_oracle", &mut sink);

        let mut sc_tcrc: u32 = 0;
        for step in 0..12 {
            let kind = sc_ops[step * 4];
            let a0 = sc_ops[step * 4 + 1];
            let a1 = sc_ops[step * 4 + 2];
            let frame = sc_ops[step * 4 + 3] as UnsignedInt;

            match kind {
                0 => sc.collect_msg_stats(sc_local_p, a1, msg_type_from_i32(a0)),
                1 => sc.increment_scroll_move_count(),
                2 => sc.increment_attack_count(),
                3 => sc.increment_build_count(),
                4 => sc.increment_move_count(),
                5 => sc.start_scroll_time(frame),
                6 => sc.end_scroll_time(frame),
                7 => {
                    let ctx = StatsContext {
                        frame,
                        play_stats: sc_play_stats,
                        map_name: "Maps/Tournament Desert/Tournament Desert.map",
                        local_player_side: local_side,
                        local_player_money: sc_money,
                    };
                    sc.update(&ctx, &world_objects, &mut sink);
                }
                _ => panic!("unknown sc_op {kind}"),
            }

            sc_tcrc = rot_crc(sc_tcrc, sc.build_commands);
            sc_tcrc = rot_crc(sc_tcrc, sc.move_commands);
            sc_tcrc = rot_crc(sc_tcrc, sc.attack_commands);
            sc_tcrc = rot_crc(sc_tcrc, sc.scroll_map_commands);
            sc_tcrc = rot_crc(sc_tcrc, sc.ai_units);
            sc_tcrc = rot_crc(sc_tcrc, sc.player_units);
            sc_tcrc = rot_crc(sc_tcrc, sc.scroll_time);
            sc_tcrc = rot_crc(sc_tcrc, sc.time_count as u32);
            sc_tcrc = rot_crc(sc_tcrc, sc.last_update as u32);
        }

        let end_ctx = StatsContext {
            frame: sc_end_f,
            play_stats: sc_play_stats,
            map_name: "Maps/Tournament Desert/Tournament Desert.map",
            local_player_side: local_side,
            local_player_money: sc_money,
        };
        sc.write_file_end(&end_ctx, &mut sink);

        assert_eq!(sc_tcrc, get_u32(line, "sc_tcrc").expect("sc_tcrc"), "seed {seed}");
        let sc_out = get_i32_vec(line, "sc_out").expect("sc_out");
        assert_eq!(sc_out.len(), 12);
        assert_eq!(sc.build_commands, sc_out[0] as u32, "seed {seed}");
        assert_eq!(sc.move_commands, sc_out[1] as u32, "seed {seed}");
        assert_eq!(sc.attack_commands, sc_out[2] as u32, "seed {seed}");
        assert_eq!(sc.scroll_map_commands, sc_out[3] as u32, "seed {seed}");
        assert_eq!(sc.ai_units, sc_out[4] as u32, "seed {seed}");
        assert_eq!(sc.player_units, sc_out[5] as u32, "seed {seed}");
        assert_eq!(sc.scroll_begin_time, sc_out[6] as u32, "seed {seed}");
        assert_eq!(sc.scroll_time, sc_out[7] as u32, "seed {seed}");
        assert_eq!(sc.is_scrolling as i32, sc_out[8], "seed {seed}");
        assert_eq!(sc.time_count, sc_out[9], "seed {seed}");
        assert_eq!(sc.last_update, sc_out[10], "seed {seed}");
        assert_eq!(sc.start_frame, sc_out[11], "seed {seed}");

        let expected_sc_norm = get_json_string(line, "sc_norm").expect("sc_norm");
        assert_eq!(sink.contents, expected_sc_norm, "seed {seed} stats log mismatch");
    }
}
