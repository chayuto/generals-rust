//! `Player::attemptToPurchaseScience` and what `Player::addScience` does to the
//! world (part of `Player.cpp`; the player's own state is in `player.rs`).
//!
//! Golden vectors: `docs/golden_vectors/SciencePurchase.json`
//! (`tests/cpp_harness/gen_science_purchase.cpp`,
//! `crates/zh_logic/tests/golden_science_purchase.rs`).
//!
//! The sim still holds sciences by team, so a new science is entered for every
//! team of the player. `TheControlBar` calls are the client's.

use crate::arena::{FacetMask, ModuleSlot};
use crate::player::player::Player;
use crate::rules::science::{SciencePlayerAccess, ScienceType, SCIENCE_INVALID};
use crate::scripts::script_engine::ScriptEngine;
use crate::sim::{SimApi, SimContext};
use zh_common::types::Int;

impl SciencePlayerAccess for Player {
    fn has_science(&self, science: ScienceType) -> bool {
        Player::has_science(self, science)
    }
}

/// `Player::attemptToPurchaseScience` for the player at `index`. The script
/// engine hears of the science when one is given.
pub fn attempt_to_purchase_science(
    sim: &mut SimContext,
    scripts: Option<&mut ScriptEngine>,
    index: Int,
    science: ScienceType,
) -> bool {
    let Some(player) = sim.world.players.player_mut(index) else {
        return false;
    };
    // `isCapableOfPurchasingScience`.
    if science == SCIENCE_INVALID
        || player.has_science(science)
        || player.sciences_disabled.contains(&science)
        || player.sciences_hidden.contains(&science)
        || !sim
            .rules
            .science_store
            .player_has_prereqs_for_science(player, science)
    {
        return false;
    }
    let cost = sim.rules.science_store.get_science_purchase_cost(science);
    if cost == 0 || cost > player.science_purchase_points {
        return false;
    }
    // `addSciencePurchasePoints(-cost)`.
    player.science_purchase_points = player.science_purchase_points.wrapping_sub(cost).max(0);
    player.sciences.push(science);
    science_added_to_world(sim, index, science);
    if let Some(scripts) = scripts {
        scripts.notify_of_acquired_science(index, science);
    }
    sim.world
        .player_academy
        .entry(index)
        .or_default()
        .record_generals_points_spent(cost);
    true
}

/// The rest of `Player::addScience`: the sim's copy of the player's sciences,
/// then every special power on the player's objects that needs this science
/// starts over, ready now.
pub fn science_added_to_world(sim: &mut SimContext, index: Int, science: ScienceType) {
    let owned = sim.world.team_players.iter();
    let teams: Vec<_> = owned
        .filter(|&(_, &player)| player == index)
        .map(|(&team, _)| team)
        .chain(sim.world.player_default_teams.get(&index).copied())
        .collect();
    for team in teams {
        sim.grant_player_science(team, science);
    }
    // `m_playerTeamPrototypes`, each prototype's teams, each team's members.
    let listed: Vec<_> = sim
        .world
        .player_team_prototypes
        .get(&index)
        .into_iter()
        .flatten()
        .flatten()
        .copied()
        .collect();
    let modules: Vec<ModuleSlot> = listed
        .into_iter()
        .filter_map(|team| sim.world.team_members.get(&team))
        .flatten()
        .filter_map(|&slot| sim.world.objects.get(slot))
        .flat_map(|object| &object.behaviors)
        .filter(|(_, facets)| facets.contains(FacetMask::SPECIAL_POWER))
        .map(|&(module, _)| module)
        .collect();
    for module in modules {
        let _ = sim.dispatch_module(module, |m, sim, ctx| {
            let Some(power) = m.as_special_power_mut() else {
                return;
            };
            if power.get_required_science(sim, ctx) == science {
                power.on_special_power_creation(sim, ctx);
                let now = sim.frame();
                power.set_ready_frame(sim, ctx, now);
            }
        });
    }
}
