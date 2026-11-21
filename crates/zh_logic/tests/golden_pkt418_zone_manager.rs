//! Gate V2 Golden Vector parity tests for `PKT-418` (`PathfindZoneManager`)
//! & `PKT-420` (`ZoneBlock`) (`WP-6.2`).
//!
//! When implementing `PKT-418` & `PKT-420`, remove both `#[ignore]` attributes
//! below so all 1,000 stateful zone calculation, block equivalency, passability,
//! bridge connectivity, and incremental `update_zones_for_modify` vectors run in CI.

use zh_common::golden_json::{
    extract_vector_lines, get_i32, get_i32_array, get_i32_vec, get_u16_vec, get_u32,
};
use zh_common::math::{ICoord2D, IRegion2D};
use zh_common::types::{Bool, ObjectId, UnsignedInt, FALSE, TRUE};
use zh_logic::pathfinder::{
    CellType, PathfindLayerEnum, PathfindZoneManager, PathfindZoneMap, ZoneBlock, ZoneLayerSlot,
    ZoneStorageType, INITIAL_ZONES, LAYER_COUNT, MAX_ZONES, UNINITIALIZED_ZONE, ZONE_BLOCK_SIZE,
    ZONE_UPDATE_FREQUENCY,
};
use zh_logic::rules::combat_types::{LocomotorSurfaceType, NO_SURFACES};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-418.json");

#[test]
fn test_pkt418_oracle_structure_and_constants() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    for required in [
        "PathfindZoneManager.h",
        "PathfindZoneManager.cpp",
        "ZoneBlock.h",
        "ZoneBlock.cpp",
    ] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }

    assert_eq!(INITIAL_ZONES, 256);
    assert_eq!(ZONE_BLOCK_SIZE, 10);
    assert_eq!(UNINITIALIZED_ZONE, 0);
    assert_eq!(MAX_ZONES, 24_000);
    assert_eq!(LAYER_COUNT, 16);
    assert_eq!(ZONE_UPDATE_FREQUENCY, 300);

    for i in 0..=15 {
        let layer = PathfindLayerEnum::from_i32(i).unwrap();
        assert_eq!(layer.as_i32(), i);
    }
    assert!(PathfindLayerEnum::from_i32(16).is_none());

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_pkt418_zone_block_helpers_and_boundaries() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for line in &lines {
        let seed = get_u32(line, "seed").unwrap();
        let pairs = get_i32_array::<4>(line, "zb_pairs").unwrap();
        let exp_hier = get_u16_vec(line, "zb_hier").unwrap();
        let exp_pre = get_u16_vec(line, "zb_pre_flat").unwrap();
        let exp_post = get_u16_vec(line, "zb_post_flat").unwrap();

        let mut hier: Vec<ZoneStorageType> = (0..12).collect();
        let mut eq: Vec<ZoneStorageType> = (0..12).collect();
        ZoneBlock::resolve_zones(pairs[0], pairs[1], &mut hier);
        ZoneBlock::resolve_zones(pairs[2], pairs[3], &mut eq);
        assert_eq!(hier, exp_hier, "seed {seed} zb_hier");
        assert_eq!(eq, exp_pre, "seed {seed} zb_pre_flat");

        ZoneBlock::flatten_zones(&mut eq, &hier);
        assert_eq!(eq, exp_post, "seed {seed} zb_post_flat");
    }

    let zb = ZoneBlock::new();
    assert_eq!(
        zb.get_effective_zone(
            LocomotorSurfaceType::Ground.as_i32(),
            FALSE,
            UNINITIALIZED_ZONE
        ),
        UNINITIALIZED_ZONE
    );
    assert_eq!(
        zb.get_effective_zone(LocomotorSurfaceType::Air.as_i32(), FALSE, 5),
        1
    );
    let all_main = LocomotorSurfaceType::Ground.as_i32()
        | LocomotorSurfaceType::Water.as_i32()
        | LocomotorSurfaceType::Cliff.as_i32();
    assert_eq!(zb.get_effective_zone(all_main, FALSE, 5), 1);
    assert_eq!(zb.get_effective_zone(NO_SURFACES, FALSE, 5), 0);
}

#[test]
fn test_pkt418_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);

        let w = get_i32(line, "w").unwrap();
        let h = get_i32(line, "h").unwrap();
        let cell_types = get_i32_vec(line, "cell_types").unwrap();
        let cell_fence = get_i32_vec(line, "cell_fence").unwrap();
        let cell_connect = get_i32_vec(line, "cell_connect").unwrap();

        let mut map = PathfindZoneMap::new(w, h);
        for x in 0..w {
            for y in 0..h {
                let idx = (x as usize) * (h as usize) + (y as usize);
                let ct = CellType::from_i32(cell_types[idx]).unwrap();
                let cell = map.cell_mut(x, y);
                cell.cell_type = ct;
                if ct == CellType::Obstacle {
                    cell.obstacle_id = ObjectId(77);
                    cell.obstacle_is_fence = cell_fence[idx] != 0;
                }
                let conn = cell_connect[idx];
                if conn != 0 {
                    cell.set_connect_layer(PathfindLayerEnum::from_i32(conn).unwrap());
                }
            }
        }

        let layer_unused = get_i32_vec(line, "layer_unused").unwrap();
        let layer_destroyed = get_i32_vec(line, "layer_destroyed").unwrap();
        let layer_sx = get_i32_vec(line, "layer_sx").unwrap();
        let layer_sy = get_i32_vec(line, "layer_sy").unwrap();
        let layer_ex = get_i32_vec(line, "layer_ex").unwrap();
        let layer_ey = get_i32_vec(line, "layer_ey").unwrap();

        let mut layers: [ZoneLayerSlot; LAYER_COUNT] = std::array::from_fn(|_| ZoneLayerSlot::default());
        for li in 0..LAYER_COUNT {
            layers[li].unused = layer_unused[li] != 0;
            layers[li].destroyed = layer_destroyed[li] != 0;
            layers[li].start_cell = ICoord2D::new(layer_sx[li], layer_sy[li]);
            layers[li].end_cell = ICoord2D::new(layer_ex[li], layer_ey[li]);
        }

        let frames = get_i32_vec(line, "frames").unwrap();
        let frame0 = frames[0] as UnsignedInt;
        let frame1 = frames[1] as UnsignedInt;
        let need_exp = get_i32_array::<4>(line, "need_calc").unwrap();

        let mut zm = PathfindZoneManager::new();
        assert_eq!(zm.need_to_calculate_zones(frame0) as i32, need_exp[0], "seed {seed} need_init");
        zm.mark_zones_dirty(frame0);
        assert_eq!(zm.need_to_calculate_zones(frame1) as i32, need_exp[1], "seed {seed} need_dirty");

        let global_bounds = IRegion2D {
            lo: ICoord2D::new(0, 0),
            hi: ICoord2D::new(w - 1, h - 1),
        };
        zm.allocate_blocks(&global_bounds);
        let ext = zm.get_extent();
        let ext_exp = get_i32_array::<2>(line, "extent").unwrap();
        assert_eq!([ext.x, ext.y], ext_exp, "seed {seed} extent");

        zm.calculate_zones(&mut map, &mut layers, &global_bounds);
        assert_eq!(
            zm.need_to_calculate_zones(frame1) as i32,
            need_exp[2],
            "seed {seed} need_after_calc"
        );
        let frame2 = frame0.wrapping_add(5);
        zm.mark_zones_dirty(frame2);
        assert_eq!(
            zm.need_to_calculate_zones(frame2) as i32,
            need_exp[3],
            "seed {seed} need_after_dirty2"
        );

        let zones_calc_exp = get_i32_vec(line, "cell_zones_calc").unwrap();
        for x in 0..w {
            for y in 0..h {
                let idx = (x as usize) * (h as usize) + (y as usize);
                assert_eq!(
                    map.cell(x, y).zone() as i32,
                    zones_calc_exp[idx],
                    "seed {seed} calc cell ({x},{y})"
                );
            }
        }

        let layer_zones_exp = get_i32_vec(line, "layer_zones").unwrap();
        let layer_applied_exp = get_i32_vec(line, "layer_applied").unwrap();
        for li in 0..LAYER_COUNT {
            assert_eq!(layers[li].zone, layer_zones_exp[li], "seed {seed} layer {li} zone");
            assert_eq!(
                layers[li].applied_zone, layer_applied_exp[li],
                "seed {seed} layer {li} applied_zone"
            );
        }

        let bridge_calc_exp = get_i32_vec(line, "blocks_bridge_calc").unwrap();
        for bx in 0..ext.x {
            for by in 0..ext.y {
                let bidx = (bx as usize) * (ext.y as usize) + (by as usize);
                assert_eq!(
                    zm.interacts_with_bridge(bx * 10, by * 10) as i32,
                    bridge_calc_exp[bidx],
                    "seed {seed} bridge calc ({bx},{by})"
                );
            }
        }

        let eff_q = get_i32_vec(line, "eff_q").unwrap();
        let eff_r = get_i32_vec(line, "eff_r").unwrap();
        for (triple, &exp) in eff_q.chunks_exact(3).zip(eff_r.iter()) {
            let got = zm.get_effective_zone(
                triple[0],
                Bool::from(triple[1] != 0),
                triple[2] as ZoneStorageType,
            );
            assert_eq!(got as i32, exp, "seed {seed} get_effective_zone {:?}", triple);
        }

        let terr_q = get_i32_vec(line, "terr_q").unwrap();
        let terr_r = get_i32_vec(line, "terr_r").unwrap();
        for (&z, &exp) in terr_q.iter().zip(terr_r.iter()) {
            assert_eq!(
                zm.get_effective_terrain_zone(z as ZoneStorageType) as i32,
                exp,
                "seed {seed} get_effective_terrain_zone {z}"
            );
        }

        let bz_q = get_i32_vec(line, "bz_q").unwrap();
        let bz_r = get_i32_vec(line, "bz_r").unwrap();
        for (quad, &exp) in bz_q.chunks_exact(4).zip(bz_r.iter()) {
            let got = zm.get_block_zone(quad[0], Bool::from(quad[1] != 0), quad[2], quad[3], &map);
            assert_eq!(got as i32, exp, "seed {seed} get_block_zone {:?}", quad);
        }

        let pass_ops = get_i32_array::<9>(line, "pass_ops").unwrap();
        zm.clear_passable_flags();
        assert_eq!(zm.is_passable(0, 0) as i32, pass_ops[2], "seed {seed} clear");
        zm.set_all_passable();
        assert_eq!(zm.is_passable(0, 0) as i32, pass_ops[3], "seed {seed} set_all");
        zm.set_passable(pass_ops[0], pass_ops[1], FALSE);
        assert_eq!(
            zm.is_passable(pass_ops[0], pass_ops[1]) as i32,
            pass_ops[4],
            "seed {seed} set_passable"
        );
        assert_eq!(zm.clip_is_passable(-1, 0) as i32, pass_ops[5]);
        assert_eq!(zm.clip_is_passable(w + 20, h + 20) as i32, pass_ops[6]);
        assert_eq!(zm.is_passable(-1, 0) as i32, pass_ops[7]);
        zm.set_bridge(pass_ops[0], pass_ops[1], TRUE);
        assert_eq!(
            zm.interacts_with_bridge(pass_ops[0], pass_ops[1]) as i32,
            pass_ops[8]
        );

        let mb = get_i32_array::<4>(line, "mod_bounds").unwrap();
        for x in mb[0]..=mb[2] {
            for y in mb[1]..=mb[3] {
                map.cell_mut(x, y).set_zone(UNINITIALIZED_ZONE);
            }
        }
        let mod_bounds = IRegion2D {
            lo: ICoord2D::new(mb[0], mb[1]),
            hi: ICoord2D::new(mb[2], mb[3]),
        };
        zm.update_zones_for_modify(&mut map, &mut layers, &mod_bounds, &global_bounds);
        let zones_mod_exp = get_i32_vec(line, "cell_zones_mod").unwrap();
        for x in 0..w {
            for y in 0..h {
                let idx = (x as usize) * (h as usize) + (y as usize);
                assert_eq!(
                    map.cell(x, y).zone() as i32,
                    zones_mod_exp[idx],
                    "seed {seed} mod cell ({x},{y})"
                );
            }
        }
    }
}
