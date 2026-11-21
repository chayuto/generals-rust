//! Gate V2 Golden Vector parity tests for `PKT-116` (`WP-3.1` `AcademyStats`).
//!
//! Active structural check (`test_pkt116_oracle_structure_and_constants`) verifies the committed
//! C++ oracle and constants. Remove the two `#[ignore]` attributes once `PKT-116` is implemented.

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_str, get_u32, parse_i64_token,
};
use zh_common::random_value::PrngState;
use zh_common::types::{Bool, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::economy::academy_stats::academy_kind_bits::{
    KINDOF_DOZER, KINDOF_FS_SUPPLY_CENTER, KINDOF_HERO, KINDOF_INFANTRY, KINDOF_MONEY_HACKER,
};
use zh_logic::economy::{
    AcademyAdviceInfo, AcademyObjectInfo, AcademyPlayerContext, AcademyStats,
    FRAMES_BETWEEN_UPDATES, MAX_ADVICE_TIPS,
};
use zh_logic::rules::{AcademyClassificationType, ACADEMY_CLASSIFICATION_TYPE_NAMES};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-116.json");

fn parse_ops_16x4(line: &str) -> Vec<[i32; 4]> {
    let pat = "\"ops\":[";
    let start = line.find(pat).unwrap() + pat.len();
    let rest = &line[start..];
    let end = rest.find("],\"is_first\"").unwrap();
    let inner = &rest[..end];
    let mut out = Vec::with_capacity(16);
    for tuple in inner.split("],[") {
        let clean = tuple.trim_matches(|c| c == '[' || c == ']');
        let mut row = [0i32; 4];
        for (i, part) in clean.split(',').enumerate() {
            row[i] = parse_i64_token(part).unwrap() as i32;
        }
        out.push(row);
    }
    out
}

#[test]
fn test_pkt116_oracle_structure_and_constants() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-116\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("AcademyStats.h"));
    assert!(VECTORS_JSON.contains("AcademyStats.cpp"));

    assert_eq!(MAX_ADVICE_TIPS, 1);
    assert_eq!(FRAMES_BETWEEN_UPDATES, 30);
    assert_eq!(
        ACADEMY_CLASSIFICATION_TYPE_NAMES,
        ["ACT_NONE", "ACT_UPGRADE_RADAR", "ACT_SUPERPOWER"]
    );

    let mut stats = AcademyStats::new();
    stats.record_building_capture();
    stats.record_generals_points_spent(3);
    stats.record_mine();
    stats.record_vehicle_sniped();
    assert_eq!(stats.structures_captured, 1);
    assert_eq!(stats.generals_points_spent, 3);
    assert_eq!(stats.get_mines(), 1);
    assert_eq!(stats.get_vehicles_sniped(), 1);
    assert_eq!(stats.had_a_supply_center(), FALSE);

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
#[ignore = "PKT-116: remove when AcademyStats is implemented"]
fn test_pkt116_academy_stats_boundaries_and_xfer_roundtrip() {
    let mut prng = PrngState::new();
    prng.init_random_with_seed(42);

    // Unknown side ("Observer") should set unknown_side = TRUE and return FALSE from calculate_academy_advice
    let obs_ctx = AcademyPlayerContext {
        base_side: Some("Observer"),
        current_frame: 100,
        current_money: 5000,
        has_sufficient_power: TRUE,
        objects: &[],
        dozer_supply_center_cost: None,
        use_alternate_mouse: FALSE,
        local_player_mines: 0,
        neutral_player_vehicles_sniped: 0,
    };
    let mut stats = AcademyStats::new();
    stats.init(&obs_ctx);
    assert_eq!(stats.unknown_side, TRUE);
    assert_eq!(stats.is_first_update(), TRUE);
    let mut info = AcademyAdviceInfo::default();
    assert_eq!(
        stats.calculate_academy_advice(&mut info, &obs_ctx, &mut prng),
        FALSE
    );

    // Known side ("GLA") round-trip XferSave / XferLoad
    let gla_ctx = AcademyPlayerContext {
        base_side: Some("GLA"),
        current_frame: 10,
        current_money: 2000,
        has_sufficient_power: TRUE,
        objects: &[],
        dozer_supply_center_cost: None,
        use_alternate_mouse: FALSE,
        local_player_mines: 2,
        neutral_player_vehicles_sniped: 1,
    };
    stats.init(&gla_ctx);
    assert_eq!(stats.unknown_side, FALSE);
    assert_eq!(stats.next_update_frame, 40);
    stats.record_building_capture();
    stats.record_salvage_collected();

    let mut xsave = XferSave::new();
    xsave.xfer_snapshot(&mut stats).unwrap();
    let bytes = xsave.into_bytes();

    let mut loaded = AcademyStats::new();
    let mut xload = XferLoad::new(&bytes);
    xload.xfer_snapshot(&mut loaded).unwrap();
    assert_eq!(stats, loaded);
}

#[test]
#[ignore = "PKT-116: remove when AcademyStats is implemented"]
fn test_pkt116_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
    let sides = [
        Some("USA"),
        Some("China"),
        Some("GLA"),
        Some("Observer"),
        None,
    ];

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);

        let mut prng = PrngState::new();
        prng.init_random_with_seed(seed);

        let side_idx = get_i32(line, "side_idx").unwrap() as usize;
        let init_obj_mask = get_i32(line, "init_obj_mask").unwrap();
        let init_frame = get_u32(line, "init_frame").unwrap();
        let alt_mouse = get_u32(line, "alt_mouse").unwrap() as Bool;
        let local_mines = get_u32(line, "local_mines").unwrap();
        let neut_sniped = get_u32(line, "neut_sniped").unwrap();

        let mut init_objs = Vec::new();
        if (init_obj_mask & 1) != 0 {
            init_objs.push(AcademyObjectInfo {
                kind_mask: 1u32 << KINDOF_DOZER,
                is_tunnel_contain: FALSE,
            });
        }
        if (init_obj_mask & 2) != 0 {
            init_objs.push(AcademyObjectInfo {
                kind_mask: 1u32 << KINDOF_FS_SUPPLY_CENTER,
                is_tunnel_contain: FALSE,
            });
        }
        if (init_obj_mask & 4) != 0 {
            init_objs.push(AcademyObjectInfo {
                kind_mask: (1u32 << KINDOF_INFANTRY) | (1u32 << KINDOF_HERO),
                is_tunnel_contain: FALSE,
            });
        }

        let mut ctx = AcademyPlayerContext {
            base_side: sides[side_idx],
            current_frame: init_frame,
            current_money: 0,
            has_sufficient_power: TRUE,
            objects: &init_objs,
            dozer_supply_center_cost: None,
            use_alternate_mouse: alt_mouse,
            local_player_mines: local_mines,
            neutral_player_vehicles_sniped: neut_sniped,
        };

        let mut stats = AcademyStats::new();
        stats.init(&ctx);
        for _ in 0..local_mines {
            stats.record_mine();
        }

        let ops = parse_ops_16x4(line);
        assert_eq!(ops.len(), 16);

        for [op_kind, a1, a2, frame_step] in ops {
            ctx.current_frame = ctx.current_frame.wrapping_add(frame_step as UnsignedInt);
            match op_kind {
                0 => {
                    ctx.current_money = a1 as UnsignedInt;
                    ctx.has_sufficient_power = if a2 != 0 { TRUE } else { FALSE };
                    stats.update(&ctx);
                }
                1 => {
                    let prod = AcademyObjectInfo {
                        kind_mask: a1 as UnsignedInt,
                        is_tunnel_contain: if a2 != 0 { TRUE } else { FALSE },
                    };
                    stats.record_production(&prod, ctx.current_frame);
                    if seed % 5 == 1 || seed % 5 == 2 {
                        let mil = AcademyObjectInfo {
                            kind_mask: (1u32 << KINDOF_INFANTRY)
                                | (1u32 << KINDOF_FS_SUPPLY_CENTER),
                            is_tunnel_contain: FALSE,
                        };
                        stats.record_production(&mil, ctx.current_frame);
                        let dozer2 = AcademyObjectInfo {
                            kind_mask: 1u32 << KINDOF_DOZER,
                            is_tunnel_contain: FALSE,
                        };
                        stats.record_production(&dozer2, ctx.current_frame);
                    }
                    let _ = KINDOF_MONEY_HACKER;
                }
                2 => {
                    let cls =
                        AcademyClassificationType::from_i32(a1).unwrap_or(AcademyClassificationType::None);
                    stats.record_upgrade(cls, if a2 != 0 { TRUE } else { FALSE });
                }
                3 => {
                    let cls =
                        AcademyClassificationType::from_i32(a1).unwrap_or(AcademyClassificationType::None);
                    stats.record_special_power_used(cls);
                }
                4 => stats.record_income(ctx.current_frame),
                5 => {
                    stats.record_building_capture();
                    stats.record_generals_points_spent(a1);
                }
                6 => {
                    stats.record_building_garrisoned();
                    stats.record_drag_selection();
                }
                7 => {
                    stats.record_strategy_center();
                    if a1 != 0 {
                        stats.record_battle_plan_selected();
                    }
                }
                8 => {
                    stats.record_unit_entered_tunnel_network();
                    stats.record_control_groups_used();
                }
                9 => {
                    stats.record_cleared_garrisoned_building();
                    stats.record_salvage_collected();
                }
                10 => {
                    stats.record_guard_ability_used();
                    stats.record_double_click_attack_move_order_given();
                }
                11 => {
                    stats.record_vehicle_disguised();
                    stats.record_firestorm_created();
                }
                12 => stats.record_mine_cleared(),
                13 => {
                    stats.record_mine();
                    stats.record_vehicle_sniped();
                }
                _ => unreachable!(),
            }
        }

        let exp_pre_hex = get_str(line, "pre_xfer_hex").unwrap();
        let exp_pre_bytes = decode_hex_bytes(exp_pre_hex).unwrap();
        let mut xsave_pre = XferSave::new();
        xsave_pre.xfer_snapshot(&mut stats).unwrap();
        assert_eq!(
            xsave_pre.into_bytes(),
            exp_pre_bytes,
            "seed {seed} pre_xfer_hex"
        );

        let mut xcrc_pre = XferCrc::new();
        xcrc_pre.xfer_snapshot(&mut stats).unwrap();
        assert_eq!(
            xcrc_pre.get_crc(),
            get_u32(line, "pre_crc").unwrap(),
            "seed {seed} pre_crc"
        );

        let mut info = AcademyAdviceInfo::default();
        let has_advice = stats.calculate_academy_advice(&mut info, &ctx, &mut prng);
        assert_eq!(
            has_advice as u32,
            get_u32(line, "has_advice").unwrap(),
            "seed {seed} has_advice"
        );
        assert_eq!(
            info.num_tips,
            get_u32(line, "num_tips").unwrap(),
            "seed {seed} num_tips"
        );
        let got_tip0 = if info.num_tips > 0 {
            info.advice[0].trim()
        } else {
            ""
        };
        assert_eq!(got_tip0, get_str(line, "tip0").unwrap(), "seed {seed} tip0");

        let post_rng = prng.get_game_client_random_value(0, 1_000_000);
        assert_eq!(
            post_rng,
            get_i32(line, "post_client_rng").unwrap(),
            "seed {seed} post_client_rng"
        );

        assert_eq!(
            stats.is_first_update() as u32,
            get_u32(line, "is_first").unwrap(),
            "seed {seed} is_first"
        );
        assert_eq!(
            stats.had_a_supply_center() as u32,
            get_u32(line, "had_supply").unwrap(),
            "seed {seed} had_supply"
        );
        assert_eq!(
            stats.get_mines(),
            get_u32(line, "mines").unwrap(),
            "seed {seed} mines"
        );
        assert_eq!(
            stats.get_vehicles_sniped(),
            get_u32(line, "sniped").unwrap(),
            "seed {seed} sniped"
        );

        let exp_post_hex = get_str(line, "post_xfer_hex").unwrap();
        let exp_post_bytes = decode_hex_bytes(exp_post_hex).unwrap();
        let mut xsave_post = XferSave::new();
        xsave_post.xfer_snapshot(&mut stats).unwrap();
        let post_bytes = xsave_post.into_bytes();
        assert_eq!(post_bytes, exp_post_bytes, "seed {seed} post_xfer_hex");

        let mut loaded = AcademyStats::new();
        let mut xload = XferLoad::new(&post_bytes);
        xload.xfer_snapshot(&mut loaded).unwrap();
        assert_eq!(loaded, stats, "seed {seed} xfer roundtrip");
    }
}
