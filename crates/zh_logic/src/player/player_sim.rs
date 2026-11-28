//! What the sim does with the state a `Player` holds, reached from one of its
//! teams (`Object::getControllingPlayer`), and `Player::onPowerBrownOutChange`.
//!
//! Golden vectors: `docs/golden_vectors/PlayerHoldings.json`
//! (`tests/cpp_harness/gen_player_holdings.cpp`,
//! `crates/zh_logic/tests/golden_player_holdings.rs`).

use zh_common::types::{Bool, Int, TeamId, UnsignedInt, FALSE};

use crate::economy::energy::{Energy, EnergyOwnerCallbacks};
use crate::player::player::Player;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimContext, SimView};

/// Holds what an [`Energy`] call reports. Every such call reports once, as its
/// last step, so acting on it afterwards keeps the reference order.
#[derive(Default)]
struct BrownOut(Option<Bool>);

impl EnergyOwnerCallbacks for BrownOut {
    fn on_power_brown_out_change(&mut self, now_brownout: Bool) {
        self.0 = Some(now_brownout);
    }
}

impl SimContext {
    /// The controlling player of a member of `team_id`; a null `Team*` has none.
    pub fn player_of_team(&self, team_id: TeamId) -> Option<&Player> {
        self.controlling_player(team_id)
            .and_then(|p| self.world.players.player(p))
    }

    pub fn player_of_team_mut(&mut self, team_id: TeamId) -> Option<&mut Player> {
        let player = self.controlling_player(team_id)?;
        self.world.players.player_mut(player)
    }

    /// A call on a player's `Energy`, with the `onPowerBrownOutChange` it makes.
    pub(crate) fn change_energy(
        &mut self,
        player: Int,
        change: impl FnOnce(&mut Energy, UnsignedInt, &mut dyn EnergyOwnerCallbacks),
    ) {
        let frame = self.world.frame;
        let mut told = BrownOut::default();
        change(self.energy_of(player), frame, &mut told);
        if let Some(brownout) = told.0 {
            self.on_power_brown_out_change(player, brownout);
        }
    }

    /// The power sabotage part of `Player::update`: once the outage is over the
    /// power state is brought up to date. `true` when it did.
    pub fn expire_power_sabotage(&mut self, player: Int) -> bool {
        let now = self.world.frame;
        let Some(energy) = self.world.player_energy.get_mut(&player) else {
            return false;
        };
        let till = energy.get_power_sabotaged_till_frame();
        if till == 0 || now <= till {
            return false;
        }
        energy.set_power_sabotaged_till_frame(0);
        let short = Bool::from(energy.has_sufficient_power(now) == FALSE);
        self.on_power_brown_out_change(player, short);
        true
    }

    /// `Player::onPowerBrownOutChange`: radar off or on, then every `POWERED`
    /// object of the player.
    pub fn on_power_brown_out_change(&mut self, player: Int, brownout: Bool) {
        if let Some(p) = self.world.players.player_mut(player) {
            if brownout != FALSE {
                p.disable_radar();
            } else {
                p.enable_radar();
            }
        }
        // `iterateObjects(doPowerDisable)`.
        for slot in self.player_object_slots(player) {
            if self.object_is_kind_of(slot, KindOfType::Powered) == FALSE {
                continue;
            }
            if brownout != FALSE {
                self.set_disabled(slot, DisabledType::Underpowered);
            } else {
                let _ = self.clear_disabled(slot, DisabledType::Underpowered);
            }
        }
    }
}
