//! One logic frame: `GameLogic::update` (`GameLogic.cpp`) with `AI::update`
//! (`AI.cpp`), in the reference order. Slice S0 of `docs/AI_PORT_PLAN.md`; the
//! trace test is `tests/golden_s0_frame_driver.rs`.
//!
//! What is not ported is a method of [`FrameSeams`], called where the reference
//! calls it. Left out because it is not world state: the replay CRC message, the
//! stats collector, the client frame number and profiling, and the empty
//! `LocomotorStore::update`.

use crate::object::weapon_effects;
use crate::arena::ObjectSlot;
use crate::economy::MAX_PLAYER_COUNT;
use crate::rules::disabled_types::{DisabledType, DISABLED_COUNT};
use crate::sim::{SimApi, SimContext};
use zh_common::types::{Bool, Int, ObjectId, FALSE};

/// The unported parts of the frame, in call order. Each default does nothing.
pub trait FrameSeams {
    /// `TheScriptEngine->UPDATE()`.
    fn script_engine(&mut self, _sim: &mut SimContext) {}
    /// `TheGameEngine->isTimeFrozen()`: the frame ends here, uncounted.
    fn is_time_frozen(&mut self, _sim: &SimContext) -> Bool {
        FALSE
    }
    /// `TheTerrainLogic->UPDATE()` (water levels).
    fn terrain_logic(&mut self, _sim: &mut SimContext) {}
    /// `TheRecorder->UPDATE()`.
    fn recorder(&mut self, _sim: &mut SimContext) {}
    /// `processCommandList(TheCommandList)`.
    fn command_list(&mut self, _sim: &mut SimContext) {}
    /// `Player::m_ai->update()`, the computer player.
    fn player_ai(&mut self, _sim: &mut SimContext, _player: Int) {}
    /// `Team::updateGenericScripts` of the player's teams.
    fn team_generic_scripts(&mut self, _sim: &mut SimContext, _player: Int) {}
    /// `Player::onPowerBrownOutChange`, when a power sabotage runs out.
    fn power_brown_out_change(&mut self, _sim: &mut SimContext, _player: Int) {}
    /// The rest of `Player::update`: academy statistics, the retaliation mode
    /// message of the local player and the income bucket.
    fn player_upkeep(&mut self, _sim: &mut SimContext, _player: Int) {}
    /// `TheBuildAssistant->UPDATE()`: the sell list.
    fn build_assistant(&mut self, _sim: &mut SimContext) {}
    /// `TheWeaponStore->UPDATE()`: delayed damage.
    fn weapon_store(&mut self, _sim: &mut SimContext) {}
    /// `TheVictoryConditions->UPDATE()`.
    fn victory_conditions(&mut self, _sim: &mut SimContext) {}
}

/// A frame with every unported part empty.
pub struct NoFrameSeams;

impl FrameSeams for NoFrameSeams {}

/// `GameLogic::update`. The path queue runs through [`SimContext::paths`].
pub fn run_frame(sim: &mut SimContext, seams: &mut dyn FrameSeams) {
    seams.script_engine(sim);
    if seams.is_time_frozen(sim) != FALSE {
        return;
    }
    seams.terrain_logic(sim);
    seams.recorder(sim);
    seams.command_list(sim);
    sim.step_sleepy_updates();
    // `AI::update`.
    process_path_queue(sim);
    update_players(sim, seams);

    seams.build_assistant(sim);
    sim.update_partition();
    sim.process_destroy_list();
    // `TheWeaponStore->UPDATE()`.
    weapon_effects::update_delayed_damage(sim);
    seams.weapon_store(sim);
    seams.victory_conditions(sim);
    check_disabled_statuses(sim);
    sim.world.frame = sim.world.frame.wrapping_add(1);
}

/// `Pathfinder::processPathfindQueue`, less the queue itself.
fn process_path_queue(sim: &mut SimContext) {
    sim.paths.begin_queue();
    while let Some(id) = sim.paths.next_request() {
        do_pathfind(sim, id);
    }
}

/// `AIUpdateInterface::doPathfind` of the object, if it still has an AI.
fn do_pathfind(sim: &mut SimContext, id: ObjectId) {
    let Some(ms) = sim
        .world
        .objects
        .slot_by_id(id)
        .and_then(|slot| sim.world.objects.get(slot))
        .and_then(|o| o.ai_slot)
    else {
        return;
    };
    let Some(entries) = sim.world.modules.payload(ms).and_then(|m| m.resident()) else {
        return;
    };
    let ctx = sim.world.modules.meta(ms).context(ms);
    (entries.do_pathfind)(sim, ctx);
}

/// `PlayerList::update`.
fn update_players(sim: &mut SimContext, seams: &mut dyn FrameSeams) {
    for player in 0..MAX_PLAYER_COUNT as Int {
        seams.player_ai(sim, player);
        seams.team_generic_scripts(sim, player);
        expire_power_sabotage(sim, seams, player);
        seams.player_upkeep(sim, player);
    }
}

/// The power sabotage part of `Player::update`.
fn expire_power_sabotage(sim: &mut SimContext, seams: &mut dyn FrameSeams, player: Int) {
    let now = sim.world.frame;
    let Some(energy) = sim.world.player_energy.get_mut(&player) else {
        return;
    };
    let till = energy.get_power_sabotaged_till_frame();
    if till != 0 && now > till {
        energy.set_power_sabotaged_till_frame(0);
        // The reference brings the objects' power state up to date here; the
        // world keeps no brown-out state yet.
        seams.power_brown_out_change(sim, player);
    }
}

/// The last loop of `GameLogic::update`: disabled states that have run out.
fn check_disabled_statuses(sim: &mut SimContext) {
    let slots: Vec<ObjectSlot> = sim.world.objects.world_iter().collect();
    for slot in slots {
        if sim
            .world
            .objects
            .get(slot)
            .is_some_and(|o| o.is_disabled() != FALSE)
        {
            check_disabled_status(sim, slot);
        }
    }
}

/// `Object::checkDisabledStatus`. `Object::clearDisabled` runs it too.
pub fn check_disabled_status(sim: &mut SimContext, owner: ObjectSlot) {
    let now = sim.world.frame;
    for index in 0..DISABLED_COUNT as Int {
        let Some(disabled_type) = DisabledType::from_i32(index) else {
            continue;
        };
        let due = sim.world.objects.get(owner).is_some_and(|o| {
            o.is_disabled_by_type(disabled_type) != FALSE
                && now >= o.disabled_till_frame[index as usize]
        });
        if due {
            sim.clear_disabled(owner, disabled_type);
            if let Some(o) = sim.world.objects.get_mut(owner) {
                o.disabled_mask.set(index, 0);
            }
        }
    }
}
