//! Gate V2 Golden Vector integrity guard & offload contract for `PKT-406` (`WP-6.1` `Radar`).
//!
//! Active test `test_pkt406_oracle_integrity` verifies the committed C++ Golden Vector oracle
//! (`docs/golden_vectors/PKT-406.json`). For `env:ephemeral-web` offload: create
//! `crates/zh_logic/src/player/radar.rs` + `mod.rs` (exported via `pub mod player;` in
//! `crates/zh_logic/src/lib.rs`), replace the `#[ignore]` test below with the full 1,000-seed
//! assertion loop against `VECTORS_JSON`, remove `#[ignore]`, and run `scripts/port_map.py --write`.

use zh_common::crc::Crc;
use zh_common::golden_json::{
    extract_vector_lines, get_i32, get_i32_array, get_i32_vec, get_str, get_u32, get_u32_array,
};
use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferLoad, XferSave};
use zh_logic::player::radar::{Radar, RadarEventType};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-406.json");

fn splitmix32(state: &mut u32) -> u32 {
    *state = state.wrapping_add(0x9e3779b9);
    let mut z = *state;
    z = (z ^ (z >> 16)).wrapping_mul(0x85ebca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2ae35);
    z ^ (z >> 16)
}

fn harness_initial_frame(seed: usize) -> u32 {
    if seed == 0 {
        return 100;
    }
    let mut rng = (seed as u32) ^ 0x4060beef;
    if rng == 0 {
        rng = 0x12345678;
    }
    if seed % 5 != 0 {
        splitmix32(&mut rng);
        splitmix32(&mut rng);
    }
    splitmix32(&mut rng); // hi_x
    splitmix32(&mut rng); // hi_y
    splitmix32(&mut rng); // wx
    splitmix32(&mut rng); // wx fraction
    splitmix32(&mut rng); // wy
    splitmix32(&mut rng); // wy fraction
    splitmix32(&mut rng); // rx
    splitmix32(&mut rng); // ry
    splitmix32(&mut rng); // sx
    splitmix32(&mut rng); // sy
    splitmix32(&mut rng); // dw
    splitmix32(&mut rng); // dh
    splitmix32(&mut rng); // pcol
    splitmix32(&mut rng); // hide
    splitmix32(&mut rng); // force
    splitmix32(&mut rng) % 500
}

#[test]
fn test_pkt406_oracle_integrity() {
    assert!(VECTORS_JSON.contains("\"packet\": \"PKT-406\""));
    assert!(VECTORS_JSON.contains("\"module\": \"Radar\""));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("Radar.h"));
    assert!(VECTORS_JSON.contains("Radar.cpp"));

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000, "PKT-406 oracle must contain 1,000 seeds");
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(idx as u32));
        assert!(get_u32(line, "xcrc").is_some());
        assert!(get_u32(line, "xbcrc").is_some());
    }

    // Verify Seed-0 self-diagnosing checkpoints exist for single-field mismatch localization.
    let s0 = lines[0];
    let samp = get_u32_array::<2>(s0, "s0_samp").expect("Seed 0 must include s0_samp");
    assert!(samp[0] != 0 && samp[1] != 0);
    assert!(get_str(s0, "s0_xhex").map_or(false, |h| h.len() == 4499 * 2));
    assert!(get_i32(s0, "xlen") == Some(4499));
}

#[test]
fn test_pkt406_1000_seed_cpp_parity() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);

        let ext = get_u32_array::<4>(line, "ext").unwrap();
        let extent = Region3D {
            lo: Coord3D::new(f32::from_bits(ext[0]), f32::from_bits(ext[1]), 0.0),
            hi: Coord3D::new(f32::from_bits(ext[2]), f32::from_bits(ext[3]), 0.0),
        };
        let mut radar = Radar::new();
        radar.configure_extent(&extent);
        radar.set_frame(harness_initial_frame(expected_seed));
        let lp = get_i32(line, "lp").unwrap();
        radar.set_local_player_index(lp);

        let hm = get_u32(line, "hm").unwrap() as u16;
        let fm = get_u32(line, "fm").unwrap() as u16;
        for p in 0..16 {
            radar.hide(p, if ((hm >> p) & 1) != 0 { TRUE } else { FALSE });
            radar.force_on(p, if ((fm >> p) & 1) != 0 { TRUE } else { FALSE });
        }

        let wq = get_u32_array::<2>(line, "wq").unwrap();
        let world = Coord3D::new(f32::from_bits(wq[0]), f32::from_bits(wq[1]), 0.0);
        let mut ic = ICoord2D::new(0, 0);
        radar.world_to_radar(&world, &mut ic);
        let exp_w2r = get_i32_array::<2>(line, "w2r").unwrap();
        assert_eq!([ic.x, ic.y], exp_w2r, "seed {seed} world_to_radar");

        let rq = get_i32_array::<2>(line, "rq").unwrap();
        let mut world_out = Coord3D::default();
        radar.radar_to_world_2d(&ICoord2D::new(rq[0], rq[1]), &mut world_out);
        let exp_r2w = get_u32_array::<2>(line, "r2w").unwrap();
        assert_eq!(
            [world_out.x.to_bits(), world_out.y.to_bits()],
            exp_r2w,
            "seed {seed} radar_to_world_2d"
        );

        let dr = get_i32_array::<4>(line, "dr").unwrap();
        let (ul, lr) = radar.find_draw_positions(dr[0], dr[1], dr[2], dr[3]);
        let exp_ul_lr = get_i32_array::<4>(line, "ul_lr").unwrap();
        assert_eq!(
            [ul.x, ul.y, lr.x, lr.y],
            exp_ul_lr,
            "seed {seed} find_draw_positions"
        );

        let mut pvis = 0i32;
        for p in -1..=6 {
            let visible = if p < 0 {
                true
            } else {
                if p >= 5 {
                    true
                } else {
                    let prio = match p {
                        0 => zh_logic::player::radar::RadarPriorityType::Invalid,
                        1 => zh_logic::player::radar::RadarPriorityType::NotOnRadar,
                        2 => zh_logic::player::radar::RadarPriorityType::Structure,
                        3 => zh_logic::player::radar::RadarPriorityType::Unit,
                        4 => zh_logic::player::radar::RadarPriorityType::LocalUnitOnly,
                        _ => zh_logic::player::radar::RadarPriorityType::Invalid,
                    };
                    Radar::is_priority_visible(prio) == TRUE
                }
            };
            if visible {
                pvis |= 1 << (p + 1);
            }
        }
        let exp_pvis = get_i32(line, "pvis").unwrap();
        assert_eq!(pvis, exp_pvis, "seed {seed} is_priority_visible");

        let player_color = get_i32(line, "pcol").unwrap();
        let ops = get_i32_vec(line, "ops").unwrap();
        let mut try_rets = Vec::new();
        let mut idx = 0;
        while idx < ops.len() {
            let kind = ops[idx];
            let a0 = ops[idx + 1];
            let a1 = ops[idx + 2];
            let a2 = ops[idx + 3];
            let a3 = ops[idx + 4];
            idx += 5;
            match kind {
                0 => {
                    radar.set_frame(a0 as u32);
                    radar.update();
                }
                1 => {
                    let event_type =
                        RadarEventType::from_i32(a0).unwrap_or(RadarEventType::Invalid);
                    let world =
                        Coord3D::new(f32::from_bits(a1 as u32), f32::from_bits(a2 as u32), 0.0);
                    radar.create_event(&world, event_type, f32::from_bits(a3 as u32));
                }
                2 => {
                    let event_type =
                        RadarEventType::from_i32(a0).unwrap_or(RadarEventType::Invalid);
                    let world =
                        Coord3D::new(f32::from_bits(a1 as u32), f32::from_bits(a2 as u32), 0.0);
                    radar.create_player_event(
                        player_color,
                        &world,
                        event_type,
                        f32::from_bits(a3 as u32),
                    );
                }
                3 => {
                    let event_type =
                        RadarEventType::from_i32(a0).unwrap_or(RadarEventType::Invalid);
                    let pos =
                        Coord3D::new(f32::from_bits(a1 as u32), f32::from_bits(a2 as u32), 0.0);
                    try_rets.push(radar.try_event(event_type, &pos));
                }
                4 => radar.queue_terrain_refresh(),
                5 => radar.reset(),
                _ => panic!("seed {seed} unknown radar operation {kind}"),
            }
        }
        let exp_try = get_i32_vec(line, "try_rets").unwrap();
        assert_eq!(
            try_rets,
            exp_try.iter().map(|&v| v as u8).collect::<Vec<_>>(),
            "seed {seed} try_rets"
        );

        let exp_last_ok = get_i32(line, "last_ok").unwrap();
        let last = radar.get_last_event_loc();
        assert_eq!(last.is_some() as i32, exp_last_ok, "seed {seed} last_ok");
        if let Some(loc) = last {
            let exp_loc = get_u32_array::<3>(line, "last_loc").unwrap();
            assert_eq!(
                [loc.x.to_bits(), loc.y.to_bits(), loc.z.to_bits()],
                exp_loc,
                "seed {seed} last_loc"
            );
        }
        assert_eq!(
            radar.next_free_radar_event(),
            get_i32(line, "next_free").unwrap(),
            "seed {seed} next_free"
        );
        assert_eq!(
            radar.last_radar_event(),
            get_i32(line, "last_idx").unwrap(),
            "seed {seed} last_idx"
        );
        assert_eq!(
            radar.active_event_count(),
            get_i32(line, "act").unwrap() as usize,
            "seed {seed} active"
        );
        assert_eq!(
            radar.queue_terrain_refresh_frame(),
            get_i32(line, "qframe").unwrap() as u32,
            "seed {seed} qframe"
        );

        let mut h_out = 0u16;
        let mut f_out = 0u16;
        for p in 0..16 {
            if radar.is_radar_hidden(p) == TRUE {
                h_out |= 1 << p;
            }
            if radar.is_radar_forced(p) == TRUE {
                f_out |= 1 << p;
            }
        }
        assert_eq!(
            h_out,
            get_u32(line, "h_out").unwrap() as u16,
            "seed {seed} h_out"
        );
        assert_eq!(
            f_out,
            get_u32(line, "f_out").unwrap() as u16,
            "seed {seed} f_out"
        );

        let mut save = XferSave::new();
        radar.xfer(&mut save).unwrap();
        let bytes = save.as_bytes();
        assert_eq!(
            bytes.len(),
            get_i32(line, "xlen").unwrap() as usize,
            "seed {seed} xlen"
        );
        let mut xbcrc = Crc::new();
        xbcrc.compute_crc(bytes, bytes.len() as i32);
        assert_eq!(
            xbcrc.get(),
            get_u32(line, "xbcrc").unwrap(),
            "seed {seed} xbcrc"
        );

        let mut xcrc = XferCrc::new();
        radar.xfer(&mut xcrc).unwrap();
        assert_eq!(
            xcrc.get_crc(),
            get_u32(line, "xcrc").unwrap(),
            "seed {seed} xcrc"
        );

        if expected_seed == 0 {
            let s0_ev0 = get_i32_vec(line, "s0_ev0").unwrap();
            let ev = radar.event(0).unwrap();
            let flat = [
                ev.event_type.as_i32(),
                ev.active as i32,
                ev.create_frame as i32,
                ev.die_frame as i32,
                ev.fade_frame as i32,
                ev.color1.red as i32,
                ev.color1.green as i32,
                ev.color1.blue as i32,
                ev.color1.alpha as i32,
                ev.radar_loc.x,
                ev.radar_loc.y,
            ];
            assert_eq!(s0_ev0, flat[..11], "Seed 0 s0_ev0");
            let hex = get_str(line, "s0_xhex").unwrap();
            assert_eq!(hex.len(), 4499 * 2);
        }

        let mut loaded = Radar::new();
        loaded.set_local_player_index(lp);
        let mut load = XferLoad::new(bytes);
        loaded.xfer(&mut load).unwrap();
        assert_eq!(load.position(), bytes.len(), "seed {seed} xfer load length");
    }
}
