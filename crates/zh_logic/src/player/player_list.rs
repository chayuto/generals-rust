//! `PKT-405` `PlayerList` (`PlayerList.cpp`): the sixteen players, game start, update order.
//!
//! Golden test: `tests/golden_players_teams.rs`.

use zh_common::types::{
    Int, NameKeyType, Relationship, TeamId, UnsignedInt, UnsignedShort, XferVersion,
};
use zh_common::xfer::{Xfer, XferError};

use crate::gamelogic::global_data::PlayerType;
use crate::player::player::{Player, MAX_PLAYER_COUNT};
use crate::player::seams::{PlayerEnv, PlayerIndex, PlayerWorld};
use crate::player::team::TeamFactory;
use crate::spatial::sides_list::{SidesList, WellKnownSidesKeys};

/// Network game slots.
pub const MAX_SLOTS: usize = 8;

pub const ALLOW_SAME_PLAYER: UnsignedInt = 0x01;
pub const ALLOW_ALLIES: UnsignedInt = 0x02;
pub const ALLOW_ENEMIES: UnsignedInt = 0x04;
pub const ALLOW_NEUTRAL: UnsignedInt = 0x08;

/// `AsciiString::nextToken` with its default separators.
fn tokens(list: &str) -> impl Iterator<Item = &str> {
    list.split([' ', '\n', '\r', '\t'])
        .filter(|t| !t.is_empty())
}

#[derive(Debug, Clone)]
pub struct PlayerList {
    players: Vec<Player>,
    local: Option<PlayerIndex>,
    player_count: Int,
    slot_indices: [Int; MAX_PLAYER_COUNT],
    slot_to_player: [Int; MAX_SLOTS],
}

impl PlayerList {
    /// The constructor: sixteen players, then `init`.
    pub fn new(env: &mut PlayerEnv<'_>) -> Self {
        let players = (0..MAX_PLAYER_COUNT as Int)
            .map(|i| Player::new(i, env))
            .collect();
        let mut list = Self {
            players,
            local: None,
            player_count: 0,
            slot_indices: [-1; MAX_PLAYER_COUNT],
            slot_to_player: [-1; MAX_SLOTS],
        };
        list.init(env);
        list
    }

    pub fn init(&mut self, env: &mut PlayerEnv<'_>) {
        self.player_count = 1;
        for player in &mut self.players {
            player.init(None, env);
        }
        self.slot_indices = [-1; MAX_PLAYER_COUNT];
        self.slot_to_player = [-1; MAX_SLOTS];
        self.set_local_player(0, env.world);
    }

    pub fn reset(&mut self, teams: &mut TeamFactory, env: &mut PlayerEnv<'_>) {
        for player in &mut self.players {
            player.delete_player_ai(env.world);
        }
        teams.clear(self, env);
        self.init(env);
    }

    /// `newGame`: a player for every named side, the teams, then the relations
    /// and default teams. Build lists move from the sides to the players.
    pub fn new_game(
        &mut self,
        sides: &mut SidesList,
        teams: &mut TeamFactory,
        env: &mut PlayerEnv<'_>,
    ) {
        self.reset(teams, env);
        let keys = WellKnownSidesKeys::intern(env.name_keys);

        let mut set_local = false;
        let mut i = 0;
        while i < sides.sides.len() {
            // `initFromDict` never changes the side dictionaries.
            let dict = sides.sides[i].dict.clone();
            if !dict.get_ascii_string(keys.player_name, None).is_empty()
                && (self.player_count as usize) < MAX_PLAYER_COUNT
            {
                let index = self.player_count;
                self.player_count += 1;
                self.players[index as usize].init_from_dict(&dict, sides, env);

                let is_local_key = env.name_keys.name_to_key("multiplayerIsLocal");
                if dict.get_bool(is_local_key, None) != zh_common::types::FALSE {
                    self.set_local_player(index, env.world);
                    set_local = true;
                }
                if !set_local
                    && !env.world.is_network()
                    && dict.get_bool(keys.player_is_human, None) != zh_common::types::FALSE
                {
                    self.set_local_player(index, env.world);
                    set_local = true;
                }
                self.players[index as usize].build_list =
                    std::mem::take(&mut sides.sides[i].build_list);
            }
            i += 1;
        }

        if !set_local {
            for index in 0..sides.num_sides().min(MAX_PLAYER_COUNT as Int) {
                if index != 0 {
                    self.players[index as usize].set_player_type(
                        PlayerType::Human,
                        false,
                        env.world,
                    );
                    self.set_local_player(index, env.world);
                    break;
                }
            }
        }

        teams.init_from_sides(sides, self, env);

        for side in &sides.sides {
            let name = side.dict.get_ascii_string(keys.player_name, None);
            let key = env.name_keys.name_to_key(&name);
            let Some(player) = self.find_player_with_name_key(key) else {
                continue;
            };
            for (list, r) in [
                (keys.player_enemies, Relationship::Enemies),
                (keys.player_allies, Relationship::Allies),
            ] {
                let list = side.dict.get_ascii_string(list, None);
                for token in tokens(&list) {
                    let key = env.name_keys.name_to_key(token);
                    if let Some(other) = self.find_player_with_name_key(key) {
                        self.players[player as usize].set_player_relationship(other, r);
                    }
                }
            }
            let p = &mut self.players[player as usize];
            p.set_player_relationship(player, Relationship::Allies);
            if player != 0 {
                p.set_player_relationship(0, Relationship::Neutral);
            }
            self.set_default_team(player, teams, env);
        }

        if (0..MAX_SLOTS).any(|slot| env.world.game_slot_occupied(slot)) {
            self.assign_slot_indices(env);
        }
    }

    /// `Player::setDefaultTeam`: the team `team<playerName>`, activated.
    pub fn set_default_team(
        &mut self,
        player: PlayerIndex,
        teams: &mut TeamFactory,
        env: &mut PlayerEnv<'_>,
    ) {
        let Some(p) = self.player(player) else {
            return;
        };
        let name = format!("team{}", p.name);
        if let Some(team) = teams.find_team(&name, self, env) {
            self.players[player as usize].default_team = Some(team);
            if let Some(t) = teams.team_mut(team) {
                t.set_active();
            }
        }
    }

    fn assign_slot_indices(&mut self, env: &mut PlayerEnv<'_>) {
        for slot in 0..MAX_SLOTS {
            if !env.world.game_slot_occupied(slot) {
                continue;
            }
            let key = env.name_keys.name_to_key(&format!("player{slot}"));
            if let Some(player) = self.find_player_with_name_key(key) {
                self.set_slot_index(player, slot as Int);
            }
        }
    }

    fn set_slot_index(&mut self, player: PlayerIndex, slot: Int) {
        let old = self.slot_indices[player as usize];
        if let Some(entry) = usize::try_from(old)
            .ok()
            .and_then(|o| self.slot_to_player.get_mut(o))
        {
            *entry = -1;
        }
        self.slot_indices[player as usize] = slot;
        if let Some(entry) = usize::try_from(slot)
            .ok()
            .and_then(|s| self.slot_to_player.get_mut(s))
        {
            *entry = player;
        }
    }

    pub fn slot_index(&self, player: PlayerIndex) -> Int {
        usize::try_from(player)
            .ok()
            .and_then(|p| self.slot_indices.get(p))
            .copied()
            .unwrap_or(-1)
    }

    pub fn player_from_slot_index(&self, slot: Int) -> Option<PlayerIndex> {
        let player = usize::try_from(slot)
            .ok()
            .and_then(|s| self.slot_to_player.get(s))
            .copied()?;
        (0..self.player_count).contains(&player).then_some(player)
    }

    pub fn player_count(&self) -> Int {
        self.player_count
    }

    /// `getNthPlayer`: any of the sixteen.
    pub fn player(&self, i: PlayerIndex) -> Option<&Player> {
        self.players.get(usize::try_from(i).ok()?)
    }

    pub fn player_mut(&mut self, i: PlayerIndex) -> Option<&mut Player> {
        self.players.get_mut(usize::try_from(i).ok()?)
    }

    pub fn local_player(&self) -> PlayerIndex {
        self.local.unwrap_or(0)
    }

    pub fn set_local_player(&mut self, player: PlayerIndex, world: &mut dyn PlayerWorld) {
        if self.local != Some(player) {
            if let Some(old) = self.local {
                world.becoming_local_player(old, false);
            }
            self.local = Some(player);
            world.becoming_local_player(player, true);
        }
    }

    pub fn find_player_with_name_key(&self, key: NameKeyType) -> Option<PlayerIndex> {
        self.active().find(|p| p.name_key == key).map(|p| p.index)
    }

    /// Skirmish players are renamed without a new name key.
    pub fn find_player_with_name(&self, name: &str) -> Option<PlayerIndex> {
        self.active().find(|p| p.name == name).map(|p| p.index)
    }

    /// The first `getPlayerCount` players.
    pub fn active(&self) -> impl Iterator<Item = &Player> {
        self.players.iter().take(self.player_count as usize)
    }

    pub fn player_from_mask(&self, mask: UnsignedShort) -> Option<PlayerIndex> {
        self.players
            .iter()
            .find(|p| p.mask() == mask)
            .map(|p| p.index)
    }

    /// `getEachPlayerFromMask`: clears the returned player's bit.
    pub fn each_player_from_mask(&self, mask: &mut UnsignedShort) -> Option<PlayerIndex> {
        match self.players.iter().find(|p| p.mask() & *mask != 0) {
            Some(p) => {
                *mask &= !p.mask();
                Some(p.index)
            }
            None => {
                *mask = 0;
                None
            }
        }
    }

    /// `getPlayersWithRelationship`: how `source` sees each default team.
    pub fn players_with_relationship(
        &self,
        source: PlayerIndex,
        allowed: UnsignedInt,
        teams: &TeamFactory,
    ) -> UnsignedShort {
        let mut mask = 0;
        let Some(src) = self.player(source).filter(|_| allowed != 0) else {
            return mask;
        };
        if allowed & ALLOW_SAME_PLAYER != 0 {
            mask |= src.mask();
        }
        for player in self.active().filter(|p| p.index != source) {
            let team = player.default_team;
            let owner = team.and_then(|t| teams.controlling_player(t));
            let wanted = match src.relationship(team, owner) {
                Relationship::Enemies => ALLOW_ENEMIES,
                Relationship::Allies => ALLOW_ALLIES,
                Relationship::Neutral => ALLOW_NEUTRAL,
            };
            if allowed & wanted != 0 {
                mask |= player.mask();
            }
        }
        mask
    }

    /// `validateTeam`: the named team, else the neutral player's default team.
    pub fn validate_team(
        &mut self,
        owner: &str,
        teams: &mut TeamFactory,
        env: &mut PlayerEnv<'_>,
    ) -> Option<TeamId> {
        match teams.find_team(owner, self, env) {
            Some(team) => Some(team),
            None => self.players[0].default_team,
        }
    }

    pub fn team_about_to_be_deleted(&mut self, team: TeamId) {
        for player in &mut self.players {
            player.remove_team_relationship(Some(team));
        }
    }

    /// `PlayerList::update`: `Player::update` for all sixteen in order.
    pub fn update(&mut self, teams: &mut TeamFactory, env: &mut PlayerEnv<'_>) {
        for i in 0..MAX_PLAYER_COUNT as Int {
            self.update_player(i, teams, env);
        }
    }

    /// `Player::update`.
    fn update_player(&mut self, i: PlayerIndex, teams: &mut TeamFactory, env: &mut PlayerEnv<'_>) {
        let at = i as usize;
        if self.players[at].ai.is_some() {
            env.world.ai_update(i);
        }
        let mut n = 0;
        while let Some(&prototype) = self.players[at].team_prototypes.get(n) {
            for team in teams.instance_ids(prototype) {
                teams.update_generic_scripts(team, self, env);
            }
            n += 1;
        }

        let frame = env.world.frame();
        let sabotaged_till = self.players[at].energy.get_power_sabotaged_till_frame();
        if sabotaged_till != 0 && frame > sabotaged_till {
            let energy = &mut self.players[at].energy;
            energy.set_power_sabotaged_till_frame(0);
            let brownout = energy.has_sufficient_power(frame) == zh_common::types::FALSE;
            env.world.power_brownout_changed(i, brownout);
        }
        env.world.academy_update(i);
        if self.local == Some(i) {
            env.world.local_player_update(i);
        }
        self.players[at].money.update_income_bucket(frame);
    }

    pub fn new_map(&mut self, world: &mut dyn PlayerWorld) {
        for player in &mut self.players {
            player.new_map(world);
        }
    }

    /// `updateTeamStates`: every player's prototypes in list order.
    pub fn update_team_states(&mut self, teams: &mut TeamFactory, env: &mut PlayerEnv<'_>) {
        for at in 0..MAX_PLAYER_COUNT {
            let mut n = 0;
            while let Some(&prototype) = self.players[at].team_prototypes.get(n) {
                teams.update_prototype_state(prototype, self, env);
                n += 1;
            }
        }
    }

    /// `PlayerList::crc`.
    pub fn crc(&self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut count = self.player_count;
        xfer.xfer_int(&mut count)?;
        for player in self.active() {
            player.crc(xfer)?;
        }
        Ok(())
    }

    pub fn xfer(
        &mut self,
        xfer: &mut dyn Xfer,
        teams: &TeamFactory,
        env: &mut PlayerEnv<'_>,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut count = self.player_count;
        xfer.xfer_int(&mut count)?;
        if count != self.player_count {
            return Err(XferError::InvalidData);
        }
        for player in self.players.iter_mut().take(count as usize) {
            player.xfer(xfer, teams, env)?;
        }
        Ok(())
    }

    /// `loadPostProcess`: the original fills the slot table with 255.
    pub fn load_post_process(&mut self) {
        self.slot_indices = [255; MAX_PLAYER_COUNT];
    }
}
