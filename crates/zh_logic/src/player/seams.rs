//! What players and teams need from systems outside this module.
//!
//! No method of [`PlayerWorld`] has a default: a host must say what it does with
//! each. Calls that can run scripts receive the player list and team factory, as
//! the script may create teams.

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::math::Coord3D;
use zh_common::types::{Int, ObjectId, TeamId, UnsignedInt, UnsignedShort};
use zh_common::xfer::{Xfer, XferError};

use crate::player::player_list::PlayerList;
use crate::player::team::TeamFactory;
use crate::rules::player_template::PlayerTemplateStore;
use crate::rules::rank_info::RankInfoStore;
use crate::rules::science::{ScienceStore, ScienceType};
use crate::rules::upgrade::UpgradeMaskType;
use crate::spatial::sides_list::SidesList;

/// Index into `ThePlayerList`.
pub type PlayerIndex = Int;

/// Which AI object `Player::setPlayerType` makes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiKind {
    /// `AIPlayer`.
    Standard,
    /// `AISkirmishPlayer`.
    Skirmish,
}

/// A sub-snapshot of `Player::xfer` owned by an unported system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerPart {
    /// The name and snapshot of each upgrade instance.
    Upgrades,
    Ai,
    ResourceGathering,
    TunnelTracker,
}

/// A skirmish player's script list to copy onto its side
/// (`ScriptList::duplicateAndQualify`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkirmishScripts<'a> {
    /// Side that receives the copy.
    pub side: Int,
    /// Skirmish side copied from.
    pub skirmish_side: Int,
    pub qualifier: &'a str,
    pub template_player_name: &'a str,
    pub player_name: &'a str,
}

pub trait PlayerWorld {
    // Game flow and settings.
    fn frame(&self) -> UnsignedInt;
    /// `TheGameInfo`'s starting cash, or `GlobalData::m_defaultStartingCash`.
    fn starting_cash(&self) -> UnsignedInt;
    /// `TheNetwork != nullptr`.
    fn is_network(&self) -> bool;
    /// `TheGameInfo` exists and its slot `slot` is occupied.
    fn game_slot_occupied(&self, slot: usize) -> bool;
    /// `ScriptEngine::getGlobalDifficulty`.
    fn global_difficulty(&self) -> Int;
    /// `AIData::m_forceSkirmishAI`.
    fn force_skirmish_ai(&self) -> bool;
    /// `TheGameText->fetch("SCIENCE:GeneralName")`.
    fn general_name(&self) -> Vec<u16>;
    /// Location of the last waypoint in `TheTerrainLogic`'s list with this name.
    fn waypoint_location(&self, name: &str) -> Option<Coord3D>;
    fn waypoint_exists(&self, id: UnsignedInt) -> bool;

    // Client.
    fn becoming_local_player(&mut self, player: PlayerIndex, yes: bool);
    /// The retaliation mode message `Player::update` posts for the local player.
    fn local_player_update(&mut self, player: PlayerIndex);

    // AI players.
    fn create_ai(&mut self, player: PlayerIndex, kind: AiKind);
    fn delete_ai(&mut self, player: PlayerIndex);
    fn set_ai_difficulty(&mut self, player: PlayerIndex, difficulty: Int);
    fn ai_update(&mut self, player: PlayerIndex);
    fn ai_new_map(&mut self, player: PlayerIndex);
    fn ai_pre_team_destroy(&mut self, player: PlayerIndex, team: TeamId);
    fn qualify_skirmish_scripts(&mut self, sides: &mut SidesList, copy: &SkirmishScripts<'_>);

    // Resource gathering, tunnels, academy, power.
    /// New `ResourceGatheringManager` and `TunnelTracker`.
    fn create_economy_trackers(&mut self, player: PlayerIndex);
    fn delete_economy_trackers(&mut self, player: PlayerIndex);
    fn academy_init(&mut self, player: PlayerIndex);
    fn academy_update(&mut self, player: PlayerIndex);
    fn academy_record_income(&mut self, player: PlayerIndex);
    /// `Player::onPowerBrownOutChange`.
    fn power_brownout_changed(&mut self, player: PlayerIndex, brownout: bool);

    // Sciences and upgrades.
    /// Special powers of the player's objects and
    /// `ScriptEngine::notifyOfAcquiredScience`.
    fn science_added(&mut self, player: PlayerIndex, science: ScienceType);
    fn upgrade_count(&self, player: PlayerIndex) -> UnsignedShort;
    fn delete_upgrades(&mut self, player: PlayerIndex);
    /// Names and masks in `UpgradeCenter::firstUpgradeTemplate` order.
    fn upgrade_masks(&self) -> Vec<(String, UpgradeMaskType)>;

    // Teams, scripts and objects.
    /// `ScriptEngine::notifyOfTeamDestruction`.
    fn notify_team_destroyed(&mut self, team: TeamId);
    /// `Object::setTeam(nullptr)` on a member of a team being deleted.
    fn detach_member(&mut self, object: ObjectId);
    /// `Object::onCapture` (`handlePartitionCellMaintenance` in retail CRC builds).
    fn member_changed_player(&mut self, object: ObjectId, old: PlayerIndex, new: PlayerIndex);
    /// Runs the action of the script `condition` for a team with
    /// `executeActionsOnCreate`.
    fn execute_team_create_actions(
        &mut self,
        players: &mut PlayerList,
        teams: &mut TeamFactory,
        condition: &str,
    );
    /// `findScriptByName` and `Script::duplicate`; the host keeps the copy.
    fn duplicate_script(&mut self, name: &str) -> Option<UnsignedInt>;
    fn evaluate_script(
        &mut self,
        players: &mut PlayerList,
        teams: &mut TeamFactory,
        script: UnsignedInt,
        team: TeamId,
    ) -> bool;
    fn script_is_one_shot(&self, script: UnsignedInt) -> bool;
    /// `friend_executeAction` of a duplicated script.
    fn execute_script_action(
        &mut self,
        players: &mut PlayerList,
        teams: &mut TeamFactory,
        script: UnsignedInt,
        team: TeamId,
    );
    /// `ScriptEngine::runScript`.
    fn run_script(
        &mut self,
        players: &mut PlayerList,
        teams: &mut TeamFactory,
        name: &str,
        team: TeamId,
    );
    fn member_is_effectively_dead(&self, object: ObjectId) -> bool;
    /// `None` for an object without an `AIUpdateInterface`.
    fn member_ai_idle(&self, object: ObjectId) -> Option<bool>;
    /// An enemy within the member's vision range (the enemy sighted query).
    fn member_sees_enemy(&mut self, object: ObjectId) -> bool;

    // Save games.
    fn xfer_player_part(
        &mut self,
        player: PlayerIndex,
        part: PlayerPart,
        xfer: &mut dyn Xfer,
    ) -> Result<(), XferError>;
}

/// The rule data and services every player or team operation may need.
pub struct PlayerEnv<'a> {
    pub templates: &'a PlayerTemplateStore,
    pub ranks: &'a RankInfoStore,
    pub sciences: &'a ScienceStore,
    pub name_keys: &'a mut NameKeyGenerator,
    pub world: &'a mut dyn PlayerWorld,
}
