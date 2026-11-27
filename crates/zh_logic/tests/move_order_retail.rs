//! The first play test (AI plan slice S10) on a retail map: a game is started,
//! each of its starting units is given one move order, and the frames run until
//! they stand again. Nothing is compared with the reference. Skipped when the
//! game or its replays are not installed (see `golden_game_start.rs`).
//!
//! `cargo test -p zh_logic --test move_order_retail -- --nocapture` says what
//! each unit did. `MOVE_ORDER_REPLAY=<name>` picks another replay's game and
//! `MOVE_ORDER_REACH=<distance>` a longer or shorter order.

#[allow(dead_code)]
mod game_start_support;

use game_start_support::{replay_parms, retail_dirs};
use std::path::PathBuf;
use zh_common::io::file::OsDiskFileHost;
use zh_common::math::{gm_sqrtf, Coord3D};
use zh_common::types::{Real, FALSE};
use zh_logic::arena::ObjectSlot;
use zh_logic::gamelogic::frame::{run_frame, NoFrameSeams};
use zh_logic::gamelogic::game_engine::{mount_game_archives, GameDataLoader};
use zh_logic::gamelogic::game_logic::start_new_game;
use zh_logic::modules::update::ai_update::AiUpdate;
use zh_logic::modules::CommandSourceType;
use zh_logic::rules::kind_of::KindOfType;
use zh_logic::sim::{SimApi, SimContext, SimView};

/// How far from where it stands each unit is sent, towards the map's middle,
/// unless the environment says otherwise.
const REACH: Real = 150.0;
/// One and a half pathfind cells.
const CLOSE: Real = 15.0;

fn position(sim: &SimContext, slot: ObjectSlot) -> Coord3D {
    *sim.object(slot).expect("unit").thing.get_position()
}

fn distance(a: &Coord3D, b: &Coord3D) -> Real {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    gm_sqrtf(dx * dx + dy * dy)
}

fn ai(sim: &mut SimContext, slot: ObjectSlot) -> &mut AiUpdate {
    let module = sim.object(slot).and_then(|o| o.ai_slot).expect("AI module");
    sim.resident_state_mut(module)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .expect("AIUpdate")
}

/// The goal of the unit's move-to state.
fn headed_for(sim: &mut SimContext, slot: ObjectSlot) -> Coord3D {
    ai(sim, slot).machine.moves.move_to.goal_position
}

fn is_idle(sim: &mut SimContext, slot: ObjectSlot) -> bool {
    ai(sim, slot).is_idle() != FALSE
}

fn describe(sim: &mut SimContext, slot: ObjectSlot) -> String {
    let ai = ai(sim, slot);
    format!(
        "state {}, path {}, waiting {}, locomotor {}, blocked frames {}",
        ai.machine.current_state_id(),
        ai.path.is_some(),
        ai.waiting_for_path != FALSE,
        ai.cur_locomotor().is_some(),
        ai.blocked_frames
    )
}

#[test]
fn starting_units_ordered_to_move_get_there() {
    let Some((game_dir, user_dir)) = retail_dirs() else {
        eprintln!("skipped: the game is not installed");
        return;
    };
    let reach = std::env::var("MOVE_ORDER_REACH")
        .ok()
        .and_then(|text| text.parse().ok())
        .unwrap_or(REACH);
    let name = std::env::var("MOVE_ORDER_REPLAY")
        .unwrap_or_else(|_| "macos_2p_vanilla_map.rep".to_owned());
    let fs = mount_game_archives(&OsDiskFileHost, &game_dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let replay = PathBuf::from(&user_dir).join("Replays").join(&name);
    let Some(parms) = replay_parms(&fs, &loader, &replay, &game_dir, &user_dir) else {
        eprintln!("skipped: {name} or its map is not installed");
        return;
    };
    let mut game = start_new_game(loader, parms).expect("start");
    let sim = &mut game.sim;
    for _ in 0..30 {
        run_frame(sim, &mut NoFrameSeams);
    }

    let units: Vec<ObjectSlot> = sim
        .world
        .objects
        .world_iter()
        .filter(|&slot| {
            sim.object(slot).is_some_and(|o| o.ai_slot.is_some())
                && sim.object_is_kind_of(slot, KindOfType::Immobile) == FALSE
                && (sim.object_is_kind_of(slot, KindOfType::Dozer) != FALSE
                    || sim.object_is_kind_of(slot, KindOfType::Infantry) != FALSE)
        })
        .collect();
    let extent = sim
        .terrain()
        .playable_extent()
        .expect("the map has an extent");
    let middle = Coord3D::new(
        (extent.lo.x + extent.hi.x) / 2.0,
        (extent.lo.y + extent.hi.y) / 2.0,
        0.0,
    );
    eprintln!("{name}: {} units to order", units.len());
    assert!(!units.is_empty(), "the game has starting units");

    let mut orders = Vec::new();
    for &slot in &units {
        let from = position(sim, slot);
        let length = distance(&from, &middle);
        let mut goal = Coord3D::new(
            from.x + (middle.x - from.x) / length * reach,
            from.y + (middle.y - from.y) / length * reach,
            0.0,
        );
        goal.z = sim.terrain().get_ground_height(goal.x, goal.y, None);
        sim.ai_move_to_position(slot, &goal, CommandSourceType::FromPlayer);
        orders.push((slot, from, goal));
    }
    run_frame(sim, &mut NoFrameSeams);
    for &(slot, ..) in &orders {
        eprintln!("after one frame: {}", describe(sim, slot));
    }
    for _ in 0..(900.0_f32.max(reach * 3.0) as u32) {
        run_frame(sim, &mut NoFrameSeams);
    }

    let mut arrived = 0;
    for &(slot, from, goal) in &orders {
        let at = position(sim, slot);
        let headed = headed_for(sim, slot);
        let (idle, moved) = (is_idle(sim, slot), distance(&from, &at));
        eprintln!(
            "moved {moved:.1}; {:.1} from the spot ordered, {:.1} from the spot headed for, \
             which is {:.1} from the one ordered; {}",
            distance(&at, &goal),
            distance(&at, &headed),
            distance(&goal, &headed),
            describe(sim, slot)
        );
        // `adjustDestination` moves a goal which is blocked or already taken, so
        // the unit is held to the spot its move state ended up with.
        if idle && moved > reach / 2.0 && distance(&at, &headed) < CLOSE {
            arrived += 1;
        }
    }
    eprintln!("{arrived} of {} arrived", orders.len());
    assert_eq!(arrived, orders.len());
}
