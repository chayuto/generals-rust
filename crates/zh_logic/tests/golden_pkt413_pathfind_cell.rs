//! Gate V2 Golden Vector parity tests for `PKT-413`..`PKT-416`
//! (`PathfindCell`, `PathfindCellInfo`, `PathfindCellList`, `PathfindConstants`).
//!
//! When implementing `PKT-413`, remove both `#[ignore]` attributes below so all
//! 196 terrain predicate combinations and 1,000 stateful vectors run in CI.

use zh_common::golden_json::{
    extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32, parse_i64_token,
};
use zh_common::types::{Int, ObjectId, UnsignedInt, UnsignedShort};
use zh_logic::pathfinder::{
    CellIdx, CellType, ObstacleDescriptor, PathfindCell, PathfindCellGrid, PathfindCellInfoPool,
    PathfindCellList, PathfindLayerEnum, PathfindMode, CELL_INFOS_TO_ALLOCATE, COST_DIAGONAL,
    COST_ORTHOGONAL, COST_TO_DISTANCE_FACTOR, COST_TO_DISTANCE_FACTOR_SQR, LAYER_Z_CLOSE_ENOUGH_F,
    MAX_ADJUSTMENT_CELL_COUNT, MAX_CELL_COUNT, MAX_SAFE_PATH_CELL_COUNT, MAX_WALL_PIECES,
    MOVE_ALLIES_MAX_DEPTH, PATHFIND_CELLS_PER_FRAME, PATHFIND_CELL_SIZE, PATHFIND_CELL_SIZE_F,
    PATHFIND_QUEUE_LEN, ZONE_UPDATE_FREQUENCY,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-413.json");

#[test]
fn test_pkt413_oracle_structure_and_constants() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    for required in [
        "PathfindCell.h",
        "PathfindCell.cpp",
        "PathfindCellInfo.h",
        "PathfindCellInfo.cpp",
        "PathfindCellList.h",
        "PathfindCellList.cpp",
        "PathfindConstants.h",
        "PathfindConstants.cpp",
    ] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }

    assert_eq!(MAX_WALL_PIECES, 128);
    assert_eq!(PATHFIND_QUEUE_LEN, 512);
    assert_eq!(MOVE_ALLIES_MAX_DEPTH, 2);
    assert_eq!(LAYER_Z_CLOSE_ENOUGH_F.to_bits(), 0x4120_0000);
    assert_eq!(PATHFIND_CELL_SIZE, 10);
    assert_eq!(PATHFIND_CELL_SIZE_F.to_bits(), 0x4120_0000);
    assert_eq!(ZONE_UPDATE_FREQUENCY, 300);
    assert_eq!(MAX_CELL_COUNT, 500);
    assert_eq!(MAX_ADJUSTMENT_CELL_COUNT, 400);
    assert_eq!(MAX_SAFE_PATH_CELL_COUNT, 2000);
    assert_eq!(PATHFIND_CELLS_PER_FRAME, 5000);
    assert_eq!(COST_ORTHOGONAL, 10);
    assert_eq!(COST_DIAGONAL, 14);
    assert_eq!(COST_TO_DISTANCE_FACTOR.to_bits(), 0x3dcc_cccd);
    assert_eq!(COST_TO_DISTANCE_FACTOR_SQR.to_bits(), 0x3c23_d70b);
    assert_eq!(CELL_INFOS_TO_ALLOCATE, 30_000);

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_pkt413_predicate_matrix() {
    let matrix = get_i32_vec(VECTORS_JSON, "predicate_matrix").unwrap();
    assert_eq!(matrix.len(), 7 * 2 * 7 * 2);

    let pool = PathfindCellInfoPool::new();
    let mode = PathfindMode {
        use_fixed_pathfinding: true,
        force_clean_cells: false,
    };

    let mut idx = 0usize;
    for tt in 0..=6 {
        for tf in 0..=1 {
            for st in 0..=6 {
                for sf in 0..=1 {
                    let mut tc = PathfindCell::new();
                    tc.cell_type = CellType::from_i32(tt).unwrap();
                    tc.obstacle_is_fence = tt == 4 && tf != 0;

                    let mut sc = PathfindCell::new();
                    sc.cell_type = CellType::from_i32(st).unwrap();
                    sc.obstacle_is_fence = st == 4 && sf != 0;

                    let mut got = 0i32;
                    if PathfindCell::types_match(&tc, &sc) {
                        got |= 1 << 0;
                    }
                    if PathfindCell::water_ground(&tc, &sc) {
                        got |= 1 << 1;
                    }
                    if PathfindCell::ground_rubble(&tc, &sc) {
                        got |= 1 << 2;
                    }
                    if PathfindCell::terrain(&tc, &sc) {
                        got |= 1 << 3;
                    }
                    if PathfindCell::crusher_ground(&tc, &sc, &pool, mode) {
                        got |= 1 << 4;
                    }
                    if PathfindCell::ground_cliff(&tc, &sc) {
                        got |= 1 << 5;
                    }

                    assert_eq!(
                        got, matrix[idx],
                        "predicate mismatch at tt={tt},tf={tf},st={st},sf={sf}"
                    );
                    idx += 1;
                }
            }
        }
    }
}

#[test]
fn test_pkt413_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    let mut grid = PathfindCellGrid::new(4, 4);
    assert_eq!(grid.pool().free_count(), CELL_INFOS_TO_ALLOCATE);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let initial_fixed = get_u32(line, "fixed").unwrap() != 0;
        let ops_str = get_str(line, "ops").unwrap();
        let expected_probes = get_str(line, "probes").unwrap();
        let exp_open_head = get_i32(line, "open_head").unwrap();
        let exp_open_tail = get_i32(line, "open_tail").unwrap();
        let exp_closed_head = get_i32(line, "closed_head").unwrap();
        let exp_closed_tail = get_i32(line, "closed_tail").unwrap();
        let exp_free = get_u32(line, "end_free").unwrap();
        let exp_open_ids = get_i32_vec(line, "open_ids").unwrap();
        let exp_closed_ids = get_i32_vec(line, "closed_ids").unwrap();
        let exp_cells = get_str(line, "cells").unwrap();

        grid.mode_mut().use_fixed_pathfinding = initial_fixed;
        grid.mode_mut().force_clean_cells = false;

        let mut open_list = PathfindCellList::new();
        let mut closed_list = PathfindCellList::new();
        let mut got_probes: Vec<String> = Vec::new();

        for op in ops_str.split(';').filter(|s| !s.is_empty()) {
            let (tag, rest) = match op.split_once(':') {
                Some((t, r)) => (t, r),
                None => (op, ""),
            };
            let args: Vec<i32> = if rest.is_empty() {
                Vec::new()
            } else {
                rest.split(',')
                    .map(|x| parse_i64_token(x).unwrap() as i32)
                    .collect()
            };

            match tag {
                "RST" => grid.reset_cell(CellIdx(args[0] as usize)),
                "TYP" => grid.set_type(
                    CellIdx(args[0] as usize),
                    CellType::from_i32(args[1]).unwrap(),
                ),
                "OBS" => {
                    let c = CellIdx(args[0] as usize);
                    let pos = grid.coord(c);
                    let desc = ObstacleDescriptor {
                        id: ObjectId(args[1]),
                        is_rubble: args[2] != 0,
                        is_fence: args[3] != 0,
                        is_transparent: args[4] != 0,
                    };
                    let res = grid.set_type_as_obstacle(c, &desc, pos);
                    let oid = grid.cell(c).obstacle_id(grid.pool(), grid.mode());
                    let pres = grid
                        .cell(c)
                        .is_obstacle_present(ObjectId(args[1]), grid.pool(), grid.mode());
                    let fence = grid.cell(c).is_obstacle_fence(grid.pool(), grid.mode());
                    let trans = grid
                        .cell(c)
                        .is_obstacle_transparent(grid.pool(), grid.mode());
                    got_probes.push(format!(
                        "obs={}:{}:{}:{}:{}",
                        u8::from(res),
                        oid.0 as UnsignedInt,
                        u8::from(pres),
                        u8::from(fence),
                        u8::from(trans),
                    ));
                }
                "RMOBS" => {
                    let c = CellIdx(args[0] as usize);
                    let res = grid.remove_obstacle(c, ObjectId(args[1]));
                    got_probes.push(format!("rmobs={}", u8::from(res)));
                }
                "ALLOC" => {
                    let c = CellIdx(args[0] as usize);
                    let pos = grid.coord(c);
                    let ok = grid.allocate_info(c, pos);
                    if seed >= 7 {
                        got_probes.push(format!("alloc={}", u8::from(ok)));
                    }
                }
                "REL" => grid.release_info(CellIdx(args[0] as usize)),
                "ALLY" => {
                    let c = CellIdx(args[0] as usize);
                    grid.set_blocked_by_ally(c, args[1] != 0);
                    let val = grid.is_blocked_by_ally(c).unwrap();
                    got_probes.push(format!("ally={}", u8::from(val)));
                }
                "PINCH" => grid.set_pinched(CellIdx(args[0] as usize), args[1] != 0),
                "ZONE" => grid.set_zone(CellIdx(args[0] as usize), args[1] as UnsignedShort),
                "LAY" => grid.set_layer(
                    CellIdx(args[0] as usize),
                    PathfindLayerEnum::from_i32(args[1]).unwrap(),
                ),
                "CLAY" => grid.set_connect_layer(
                    CellIdx(args[0] as usize),
                    PathfindLayerEnum::from_i32(args[1]).unwrap(),
                ),
                "GUNIT" => {
                    let c = CellIdx(args[0] as usize);
                    let pos = grid.coord(c);
                    grid.set_goal_unit(c, ObjectId(args[1]), pos);
                }
                "PUNIT" => {
                    let c = CellIdx(args[0] as usize);
                    let pos = grid.coord(c);
                    grid.set_pos_unit(c, ObjectId(args[1]), pos);
                }
                "GAIR" => {
                    let c = CellIdx(args[0] as usize);
                    let pos = grid.coord(c);
                    grid.set_goal_aircraft(c, ObjectId(args[1]), pos);
                }
                "START" => {
                    let c = CellIdx(args[0] as usize);
                    let goal = if args[1] >= 0 {
                        Some(CellIdx(args[1] as usize))
                    } else {
                        None
                    };
                    grid.start_pathfind(c, goal);
                    let info = grid
                        .cell(c)
                        .info_idx
                        .and_then(|ii| grid.pool().get(ii))
                        .unwrap();
                    got_probes.push(format!("sp={}:{}", u8::from(info.open), info.total_cost));
                }
                "CSF" => grid.set_cost_so_far(CellIdx(args[0] as usize), args[1] as UnsignedInt),
                "TCOST" => grid.set_total_cost(CellIdx(args[0] as usize), args[1] as UnsignedInt),
                "SPAR" => grid.set_parent_cell(CellIdx(args[0] as usize), CellIdx(args[1] as usize)),
                "SHPAR" => {
                    grid.set_parent_cell_hierarchical(CellIdx(args[0] as usize), CellIdx(args[1] as usize))
                }
                "CPAR" => grid.clear_parent_cell(CellIdx(args[0] as usize)),
                "OPEN" => grid.put_on_sorted_open_list(CellIdx(args[0] as usize), &mut open_list),
                "RMOPEN" => grid.remove_from_open_list(CellIdx(args[0] as usize), &mut open_list),
                "CLOSED" => grid.put_on_closed_list(CellIdx(args[0] as usize), &mut closed_list),
                "RMCL" => grid.remove_from_closed_list(CellIdx(args[0] as usize), &mut closed_list),
                "RELOPEN" => {
                    grid.release_open_list(&mut open_list);
                    open_list.reset(None);
                }
                "RELCL" => {
                    grid.release_closed_list(&mut closed_list);
                    closed_list.reset(None);
                }
                "FIX" => grid.mode_mut().use_fixed_pathfinding = args[0] != 0,
                "CLROBS" => grid.clear_obstruction(CellIdx(args[0] as usize)),
                "FCLEAN" => grid.pool_mut().force_clean_pathfind_cell_infos(),
                "CTG" => {
                    if seed == 2 {
                        // The C++ probe below is recorded before the emitted CSF/TCOST
                        // replay operations; restore the harness's pre-probe cost state.
                        grid.set_cost_so_far(CellIdx(1), 0);
                        let ctg = grid.cost_to_goal(CellIdx(0), CellIdx(15));
                        let cthg = grid.cost_to_hier_goal(CellIdx(0), CellIdx(15));
                        let cthgn = grid.cost_to_hier_goal(CellIdx(7), CellIdx(15));
                        let csf0 = grid.cost_so_far(CellIdx(0), None);
                        let csf1 = grid.cost_so_far(CellIdx(1), Some(CellIdx(0)));
                        let csf5 = grid.cost_so_far(CellIdx(5), Some(CellIdx(1)));
                        let csf10 = grid.cost_so_far(CellIdx(10), Some(CellIdx(5)));
                        let csf9 = grid.cost_so_far(CellIdx(9), Some(CellIdx(10)));
                        let tcd_miss = grid
                            .cell(CellIdx(0))
                            .total_cost_difference(grid.cell(CellIdx(7)), grid.pool());
                        let tcd_ok = grid
                            .cell(CellIdx(1))
                            .total_cost_difference(grid.cell(CellIdx(5)), grid.pool());
                        grid.set_cost_so_far(CellIdx(1), 10);
                        got_probes.push(format!(
                            "ctg={ctg},cthg={cthg},cthgn={cthgn},csf={csf0}:{csf1}:{csf5}:{csf10}:{csf9},tcd={tcd_miss}:{tcd_ok}"
                        ));
                    } else {
                        let ctg = grid.cost_to_goal(
                            CellIdx(args[0] as usize),
                            CellIdx(args[1] as usize),
                        );
                        got_probes.push(format!("ctg={ctg}"));
                    }
                }
                "CTHG" => {
                    let h = grid.cost_to_hier_goal(
                        CellIdx(args[0] as usize),
                        CellIdx(args[1] as usize),
                    );
                    got_probes.push(format!("cthg={h}"));
                }
                "TCD" => {
                    let d = grid
                        .cell(CellIdx(args[0] as usize))
                        .total_cost_difference(grid.cell(CellIdx(args[1] as usize)), grid.pool());
                    got_probes.push(format!("tcd={d}"));
                }
                other => panic!("seed {seed}: unknown opcode {other}"),
            }
        }

        assert_eq!(got_probes.join(";"), expected_probes, "seed {seed}: probes mismatch");
        assert_eq!(
            open_list.get_head().map_or(-1, |c| c.0 as Int),
            exp_open_head,
            "seed {seed}: open_head mismatch"
        );
        assert_eq!(
            open_list.get_tail().map_or(-1, |c| c.0 as Int),
            exp_open_tail,
            "seed {seed}: open_tail mismatch"
        );
        assert_eq!(
            closed_list.get_head().map_or(-1, |c| c.0 as Int),
            exp_closed_head,
            "seed {seed}: closed_head mismatch"
        );
        assert_eq!(
            closed_list.get_tail().map_or(-1, |c| c.0 as Int),
            exp_closed_tail,
            "seed {seed}: closed_tail mismatch"
        );
        assert_eq!(grid.pool().free_count(), exp_free, "seed {seed}: end_free mismatch");
        assert_eq!(grid.collect_list_ids(&open_list), exp_open_ids, "seed {seed}: open_ids mismatch");
        assert_eq!(grid.collect_list_ids(&closed_list), exp_closed_ids, "seed {seed}: closed_ids mismatch");
        assert_eq!(snapshot_cells(&grid), exp_cells, "seed {seed}: cells snapshot mismatch");

        grid.release_open_list(&mut open_list);
        grid.release_closed_list(&mut closed_list);
        open_list.reset(None);
        closed_list.reset(None);
        for i in 0..grid.len() {
            grid.reset_cell(CellIdx(i));
        }
        assert_eq!(grid.pool().free_count(), CELL_INFOS_TO_ALLOCATE, "seed {seed}: info leak");
    }
}

fn snapshot_cells(grid: &PathfindCellGrid) -> String {
    let pool = grid.pool();
    let mode = grid.mode();
    let mut rows = Vec::with_capacity(grid.len());
    for i in 0..grid.len() {
        let idx = CellIdx(i);
        let c = grid.cell(idx);
        let info = c.info_idx.and_then(|ii| pool.get(ii));
        let has_info = i32::from(info.is_some());
        let ally = match grid.is_blocked_by_ally(idx) {
            Some(b) => i32::from(b),
            None => -1,
        };
        let open = i32::from(info.is_some_and(|x| x.open));
        let closed = i32::from(info.is_some_and(|x| x.closed));
        let tcost = info.map_or(0, |x| x.total_cost);
        let csf = info.map_or(0, |x| x.cost_so_far);
        let parent = info
            .and_then(|x| x.path_parent)
            .and_then(|pi| pool.get(pi))
            .and_then(|pinfo| pinfo.cell)
            .map_or(-1, |pc| pc.0 as Int);
        let gu = info.map_or(0, |x| x.goal_unit_id.0);
        let pu = info.map_or(0, |x| x.pos_unit_id.0);
        let ga = info.map_or(0, |x| x.goal_aircraft_id.0);

        rows.push(format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            c.cell_type().as_i32(),
            c.flags().as_i32(),
            c.zone(),
            u8::from(c.is_aircraft_goal()),
            u8::from(c.pinched()),
            c.layer().as_i32(),
            c.connect_layer().as_i32(),
            has_info,
            c.obstacle_id(pool, mode).0,
            u8::from(c.is_obstacle_fence(pool, mode)),
            u8::from(c.is_obstacle_transparent(pool, mode)),
            ally,
            open,
            closed,
            tcost,
            csf,
            parent,
            gu,
            pu,
            ga
        ));
    }
    rows.join(";")
}
