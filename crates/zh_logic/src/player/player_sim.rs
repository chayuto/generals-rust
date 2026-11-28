//! What the sim does with the state a `Player` holds, reached from one of its
//! teams (`Object::getControllingPlayer`), and `Player::onPowerBrownOutChange`.
//!
//! Golden vectors: `docs/golden_vectors/PlayerHoldings.json`
//! (`tests/cpp_harness/gen_player_holdings.cpp`,
//! `crates/zh_logic/tests/golden_player_holdings.rs`).

use zh_common::types::{Bool, Int, Relationship, TeamId, UnsignedInt, FALSE, TRUE};

use crate::economy::energy::{Energy, EnergyOwnerCallbacks};
use crate::economy::money::{MoneyCallbacks, NoopMoneyCallbacks};
use crate::gamelogic::global_data::PlayerType;
use crate::player::player::Player;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimContext, SimView, SimWorld};

/// Holds what an [`Energy`] call reports. Every such call reports once, as its
/// last step, so acting on it afterwards keeps the reference order.
#[derive(Default)]
struct BrownOut(Option<Bool>);

impl EnergyOwnerCallbacks for BrownOut {
    fn on_power_brown_out_change(&mut self, now_brownout: Bool) {
        self.0 = Some(now_brownout);
    }
}

/// Holds `Money::deposit`'s income record, its last step. The sounds it also
/// asks for are client side.
#[derive(Default)]
struct Income(Option<Int>);

impl MoneyCallbacks for Income {
    fn record_income(&mut self, player_index: Int) {
        self.0 = Some(player_index);
    }
}

impl SimWorld {
    /// `from`'s `m_playerRelations` entry for the player `to`.
    pub fn player_relation(&self, from: Int, to: Int) -> Option<Relationship> {
        self.players.player(from)?.player_relations.get(to as u32)
    }

    /// `setPlayerRelationship`. Panics without a player `from`.
    pub fn set_player_relation(&mut self, from: Int, to: Int, relationship: Relationship) {
        self.players
            .player_mut(from)
            .expect("no such player")
            .set_player_relationship(to, relationship);
    }

    /// The controlling player of `team`. Panics when it has none.
    pub fn team_player_mut(&mut self, team: TeamId) -> &mut Player {
        let index = self.team_players.get(&team).copied().unwrap_or(team.0);
        self.players.player_mut(index).expect("team without a player")
    }

    /// Makes the controlling player of `team` a human or a computer.
    pub fn set_team_human(&mut self, team: TeamId, human: Bool) {
        self.team_player_mut(team).player_type = if human != FALSE {
            PlayerType::Human
        } else {
            PlayerType::Computer
        };
    }
}

impl SimContext {
    /// `getMoney()->countMoney()`; 0 without a player.
    pub fn money_of(&self, player: Int) -> UnsignedInt {
        self.world
            .players
            .player(player)
            .map_or(0, |p| p.money.count_money())
    }

    /// `Money::setStartingCash`: a balance with no income behind it.
    pub fn set_player_cash(&mut self, player: Int, amount: UnsignedInt) {
        if let Some(p) = self.world.players.player_mut(player) {
            p.money.set_starting_cash(amount);
        }
    }

    /// `Money::deposit`.
    pub fn deposit_to_player(&mut self, player: Int, amount: UnsignedInt, track_income: Bool) {
        let mut income = Income::default();
        let Some(p) = self.world.players.player_mut(player) else {
            return;
        };
        p.money.deposit(amount, TRUE, track_income, &mut income);
        // The academy of the player the money names, which need not be its owner.
        if let Some(index) = income.0 {
            let frame = self.world.frame;
            if let Some(p) = self.world.players.player_mut(index) {
                p.academy.record_income(frame);
            }
        }
    }

    /// `Money::withdraw`: what was taken.
    pub fn withdraw_from_player(&mut self, player: Int, amount: UnsignedInt) -> UnsignedInt {
        self.world.players.player_mut(player).map_or(0, |p| {
            p.money.withdraw(amount, TRUE, &mut NoopMoneyCallbacks)
        })
    }

    /// The controlling player of a member of `team_id`; a null `Team*` has none.
    pub fn player_of_team(&self, team_id: TeamId) -> Option<&Player> {
        self.controlling_player(team_id)
            .and_then(|p| self.world.players.player(p))
    }

    pub fn player_of_team_mut(&mut self, team_id: TeamId) -> Option<&mut Player> {
        let player = self.controlling_player(team_id)?;
        self.world.players.player_mut(player)
    }

    /// The money part of `Player::update`.
    pub fn update_player_money(&mut self, player: Int) {
        let frame = self.world.frame;
        if let Some(p) = self.world.players.player_mut(player) {
            p.money.update_income_bucket(frame);
        }
    }

    /// A call on a player's `Energy`, with the `onPowerBrownOutChange` it makes.
    pub(crate) fn change_energy(
        &mut self,
        player: Int,
        change: impl FnOnce(&mut Energy, UnsignedInt, &mut dyn EnergyOwnerCallbacks),
    ) {
        let frame = self.world.frame;
        let mut told = BrownOut::default();
        let Some(p) = self.world.players.player_mut(player) else {
            return;
        };
        change(&mut p.energy, frame, &mut told);
        if let Some(brownout) = told.0 {
            self.on_power_brown_out_change(player, brownout);
        }
    }

    /// The power sabotage part of `Player::update`: once the outage is over the
    /// power state is brought up to date. `true` when it did.
    pub fn expire_power_sabotage(&mut self, player: Int) -> bool {
        let now = self.world.frame;
        let Some(energy) = self.world.players.player_mut(player).map(|p| &mut p.energy) else {
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
