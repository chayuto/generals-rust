//! `PKT-404` `Player` (`Player.cpp`), partial: what exists at game start and in saves.
//!
//! Golden test: `tests/golden_players_teams.rs`.
//!
//! AI, build list processing, resource gathering, tunnels, academy statistics,
//! upgrades and object bonuses live behind
//! [`PlayerWorld`](crate::player::seams::PlayerWorld).

use zh_common::core::dict::Dict;
use zh_common::runtime::game_logic_dispatch::{PlayerSquads, Squad};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort,
    XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

use crate::economy::energy::Energy;
use crate::economy::handicap::Handicap;
use crate::economy::money::{Money, MoneyCallbacks};
use crate::economy::score_keeper::ScoreKeeper;
use crate::gamelogic::global_data::PlayerType;
use crate::modules::update::battle_plan_update::BattlePlanBonuses;
use crate::player::seams::{
    AiKind, PlayerEnv, PlayerIndex, PlayerPart, PlayerWorld, SkirmishScripts,
};
use crate::player::team::{
    flag, xfer_team_id, RelationMap, TeamFactory, TeamPrototypeId, MAX_GENERIC_SCRIPTS,
};
use crate::rules::kind_of::{KindOfMaskType, KINDOF_COUNT, KINDOF_NAMES};
use crate::rules::player_template::{PlayerTemplate, ProductionChangeMap, ProductionVeterancyMap};
use crate::rules::science::{ScienceType, ScienceVec, SCIENCE_INVALID};
use crate::rules::upgrade::UpgradeMaskType;
use crate::spatial::sides_list::{BuildListInfo, SidesList, WellKnownSidesKeys};

pub const MAX_PLAYER_COUNT: usize = 16;
/// `NEUTRAL_PLAYER_COLOR`.
pub const NEUTRAL_PLAYER_COLOR: Int = -1;

/// `KindOfPercentProductionChange`.
#[derive(Debug, Clone, PartialEq)]
pub struct KindOfProductionChange {
    pub kind_of: KindOfMaskType,
    pub percent: Real,
    pub refs: UnsignedInt,
}

/// `SpecialPowerReadyTimerType`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SpecialPowerReadyTimer {
    pub template_id: UnsignedInt,
    pub ready_frame: UnsignedInt,
}

#[derive(Debug, Clone)]
pub struct Player {
    pub index: PlayerIndex,
    pub template_name_key: Option<NameKeyType>,
    pub display_name: Vec<u16>,
    pub name: String,
    pub name_key: NameKeyType,
    pub player_type: PlayerType,
    pub side: String,
    pub base_side: String,
    pub color: Int,
    pub night_color: Int,
    pub mp_start_index: Int,
    pub production_cost_changes: ProductionChangeMap,
    pub production_time_changes: ProductionChangeMap,
    pub production_veterancy_levels: ProductionVeterancyMap,
    pub handicap: Handicap,

    pub money: Money,
    pub is_preorder: bool,
    pub sciences_disabled: ScienceVec,
    pub sciences_hidden: ScienceVec,
    pub radar_count: Int,
    pub is_dead: bool,
    pub disable_proof_radar_count: Int,
    pub radar_disabled: bool,
    pub upgrades_in_progress: UpgradeMaskType,
    pub upgrades_completed: UpgradeMaskType,
    pub energy: Energy,
    /// `m_playerTeamPrototypes`.
    pub team_prototypes: Vec<TeamPrototypeId>,
    /// `m_pBuildList`, head first.
    pub build_list: Vec<BuildListInfo>,
    pub ai: Option<AiKind>,
    /// `m_resourceGatheringManager` and `m_tunnelSystem` exist.
    pub has_economy_trackers: bool,
    pub default_team: Option<TeamId>,
    pub sciences: ScienceVec,
    pub rank_level: Int,
    pub skill_points: Int,
    pub science_purchase_points: Int,
    pub level_up: Int,
    pub level_down: Int,
    pub general_name: Vec<u16>,
    pub player_relations: RelationMap,
    pub team_relations: RelationMap,
    pub can_build_units: bool,
    pub can_build_base: bool,
    pub observer: bool,
    pub skill_points_modifier: Real,
    pub list_in_score_screen: bool,
    pub attacked_by: [bool; MAX_PLAYER_COUNT],
    pub attacked_frame: UnsignedInt,
    pub cash_bounty_percent: Real,
    pub score_keeper: ScoreKeeper,
    pub production_changes: Vec<KindOfProductionChange>,
    pub special_power_timers: Vec<SpecialPowerReadyTimer>,
    /// Hotkey squads and the current selection.
    pub squads: PlayerSquads,
    pub battle_plan_bonuses: Option<BattlePlanBonuses>,
    pub bombard_battle_plans: Int,
    pub hold_the_line_battle_plans: Int,
    pub search_and_destroy_battle_plans: Int,
    pub units_should_hunt: bool,
    pub logical_retaliation_mode: bool,
}

/// Money income reaches the academy statistics of the player that owns it.
struct Income<'a>(&'a mut dyn PlayerWorld);

impl MoneyCallbacks for Income<'_> {
    fn record_income(&mut self, player_index: Int) {
        if (0..MAX_PLAYER_COUNT as Int).contains(&player_index) {
            self.0.academy_record_income(player_index);
        }
    }
}

fn dict_string(d: &Dict, env: &mut PlayerEnv<'_>, key: &str) -> String {
    d.get_ascii_string(env.name_keys.name_to_key(key), None)
}

/// A dictionary value, `None` when the key is absent.
fn dict_int(d: &Dict, env: &mut PlayerEnv<'_>, key: &str) -> Option<Int> {
    let mut exists: Bool = FALSE;
    let value = d.get_int(env.name_keys.name_to_key(key), Some(&mut exists));
    (exists != FALSE).then_some(value)
}

fn dict_bool(d: &Dict, env: &mut PlayerEnv<'_>, key: &str) -> bool {
    d.get_bool(env.name_keys.name_to_key(key), None) != FALSE
}

/// The `PlayerTemplate` named by a side's `playerFaction`.
fn faction_template<'a>(d: &Dict, env: &mut PlayerEnv<'a>) -> Option<&'a PlayerTemplate> {
    let faction = dict_string(d, env, "playerFaction");
    let key = env.name_keys.name_to_key(&faction);
    env.templates.find_player_template(key, env.name_keys)
}

fn xfer_science_vec(
    xfer: &mut dyn Xfer,
    env: &mut PlayerEnv<'_>,
    list: &mut ScienceVec,
) -> Result<(), XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    let mut count = list.len() as UnsignedShort;
    xfer.xfer_unsigned_short(&mut count)?;
    match xfer.mode() {
        XferMode::Save => {
            for &science in list.iter() {
                let mut name = env
                    .sciences
                    .get_internal_name_for_science(science, env.name_keys)
                    .to_owned();
                xfer.xfer_ascii_string(&mut name)?;
            }
        }
        XferMode::Load => {
            list.clear();
            for _ in 0..count {
                let mut name = String::new();
                xfer.xfer_ascii_string(&mut name)?;
                let science = env
                    .sciences
                    .get_science_from_internal_name(&name, env.name_keys);
                if science == SCIENCE_INVALID {
                    return Err(XferError::InvalidData);
                }
                list.push(science);
            }
        }
        XferMode::Crc => {
            for science in list.iter_mut() {
                xfer.xfer_int(science)?;
            }
        }
    }
    Ok(())
}

/// `xferUpgradeMask`, as saves use it.
fn xfer_upgrade_mask(
    xfer: &mut dyn Xfer,
    upgrades: &[(String, UpgradeMaskType)],
    mask: &mut UpgradeMaskType,
) -> Result<(), XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    match xfer.mode() {
        XferMode::Save => {
            let set = |m: &UpgradeMaskType| mask.test_for_all(m) != FALSE;
            let mut count = upgrades.iter().filter(|u| set(&u.1)).count() as UnsignedShort;
            xfer.xfer_unsigned_short(&mut count)?;
            for (name, _) in upgrades.iter().filter(|u| set(&u.1)) {
                xfer.xfer_ascii_string(&mut name.clone())?;
            }
        }
        XferMode::Load => {
            let mut count = 0;
            xfer.xfer_unsigned_short(&mut count)?;
            mask.clear();
            for _ in 0..count {
                let mut name = String::new();
                xfer.xfer_ascii_string(&mut name)?;
                let found = upgrades
                    .iter()
                    .find(|u| u.0 == name)
                    .ok_or(XferError::InvalidData)?;
                mask.set_flags(&found.1);
            }
        }
        // `Player::crc` hashes no upgrade mask.
        XferMode::Crc => return Err(XferError::InvalidData),
    }
    Ok(())
}

fn xfer_squad(xfer: &mut dyn Xfer, squad: &mut Squad) -> Result<(), XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    let mut count = squad.ids.len() as UnsignedShort;
    xfer.xfer_unsigned_short(&mut count)?;
    if xfer.mode() == XferMode::Save {
        for &id in &squad.ids {
            xfer.xfer_object_id(&mut ObjectId(id as i32))?;
        }
    } else {
        for _ in 0..count {
            let mut id = ObjectId::INVALID;
            xfer.xfer_object_id(&mut id)?;
            squad.ids.push(id.0 as u32);
        }
    }
    Ok(())
}

fn xfer_battle_plan_values(
    xfer: &mut dyn Xfer,
    bonuses: &mut BattlePlanBonuses,
) -> Result<(), XferError> {
    xfer.xfer_real(&mut bonuses.armor_scalar)?;
    xfer.xfer_real(&mut bonuses.sight_range_scalar)?;
    xfer.xfer_int(&mut bonuses.bombardment)?;
    xfer.xfer_int(&mut bonuses.hold_the_line)?;
    xfer.xfer_int(&mut bonuses.search_and_destroy)?;
    xfer_kind_of(xfer, &mut bonuses.valid_kind_of)?;
    xfer_kind_of(xfer, &mut bonuses.invalid_kind_of)
}

/// `BitFlags<KINDOF_COUNT>::xfer`. In CRC mode the reference hashes the whole
/// `std::bitset` (64-bit words), or in retail CRC builds `sizeof(this)`, a
/// pointer's worth of it.
fn xfer_kind_of(xfer: &mut dyn Xfer, mask: &mut KindOfMaskType) -> Result<(), XferError> {
    if xfer.mode() != XferMode::Crc {
        return mask.xfer_named(xfer, &KINDOF_NAMES);
    }
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    let mut raw = [0u8; KINDOF_COUNT.div_ceil(64) * 8];
    for bit in 0..KINDOF_COUNT {
        if mask.test(bit as Int) != 0 {
            raw[bit / 8] |= 1 << (bit % 8);
        }
    }
    let size = if cfg!(feature = "retail-crc") {
        8
    } else {
        raw.len()
    };
    xfer.xfer_user(&mut raw[..size])
}

/// A presence flag that must match what the player has.
fn xfer_presence(xfer: &mut dyn Xfer, present: bool) -> Result<(), XferError> {
    let mut saved = present;
    flag(xfer, &mut saved)?;
    if saved != present {
        return Err(XferError::InvalidData);
    }
    Ok(())
}

impl Player {
    /// The constructor, which ends in `init(nullptr)`.
    pub fn new(index: PlayerIndex, env: &mut PlayerEnv<'_>) -> Self {
        let mut money = Money::new();
        money.set_player_index(index);
        let mut player = Self {
            index,
            template_name_key: None,
            display_name: Vec::new(),
            name: String::new(),
            name_key: NameKeyType::default(),
            player_type: PlayerType::Computer,
            side: String::new(),
            base_side: String::new(),
            color: 0,
            night_color: 0,
            mp_start_index: 0,
            production_cost_changes: ProductionChangeMap::default(),
            production_time_changes: ProductionChangeMap::default(),
            production_veterancy_levels: ProductionVeterancyMap::default(),
            handicap: Handicap::new(),
            money,
            is_preorder: false,
            sciences_disabled: Vec::new(),
            sciences_hidden: Vec::new(),
            radar_count: 0,
            is_dead: false,
            disable_proof_radar_count: 0,
            radar_disabled: false,
            upgrades_in_progress: UpgradeMaskType::default(),
            upgrades_completed: UpgradeMaskType::default(),
            energy: Energy::new(),
            team_prototypes: Vec::new(),
            build_list: Vec::new(),
            ai: None,
            has_economy_trackers: false,
            default_team: None,
            sciences: Vec::new(),
            rank_level: 0,
            skill_points: 0,
            science_purchase_points: 0,
            level_up: 0,
            level_down: 0,
            general_name: Vec::new(),
            player_relations: RelationMap::default(),
            team_relations: RelationMap::default(),
            can_build_units: true,
            can_build_base: true,
            observer: false,
            skill_points_modifier: 1.0,
            list_in_score_screen: true,
            attacked_by: [false; MAX_PLAYER_COUNT],
            attacked_frame: 0,
            cash_bounty_percent: 0.0,
            score_keeper: ScoreKeeper::new(),
            production_changes: Vec::new(),
            special_power_timers: Vec::new(),
            squads: PlayerSquads::default(),
            battle_plan_bonuses: None,
            bombard_battle_plans: 0,
            hold_the_line_battle_plans: 0,
            search_and_destroy_battle_plans: 0,
            units_should_hunt: false,
            logical_retaliation_mode: false,
        };
        player.init(None, env);
        player
    }

    /// `PlayerMaskType` of this player.
    pub fn mask(&self) -> UnsignedShort {
        1 << self.index
    }

    /// `Player::init`; no template means the neutral player. Relations and
    /// `m_attackedBy` are not reset.
    pub fn init(&mut self, template: Option<&PlayerTemplate>, env: &mut PlayerEnv<'_>) {
        self.skill_points_modifier = 1.0;
        self.attacked_frame = 0;
        self.is_preorder = false;
        self.is_dead = false;
        self.radar_count = 0;
        self.disable_proof_radar_count = 0;
        self.radar_disabled = false;
        self.bombard_battle_plans = 0;
        self.hold_the_line_battle_plans = 0;
        self.search_and_destroy_battle_plans = 0;
        self.battle_plan_bonuses = None;
        env.world.delete_upgrades(self.index);
        self.upgrades_in_progress.clear();
        self.upgrades_completed.clear();
        self.energy.init(Some(self.index));
        self.build_list.clear();
        self.default_team = None;
        self.delete_ai(env.world);
        self.squads = PlayerSquads::default();
        self.delete_economy_trackers(env.world);
        self.can_build_base = true;
        self.can_build_units = true;
        self.observer = false;
        self.cash_bounty_percent = 0.0;
        self.list_in_score_screen = true;
        self.units_should_hunt = false;

        match template {
            Some(pt) => {
                self.side = pt.side().to_owned();
                self.base_side = pt.base_side().to_owned();
                self.production_cost_changes = pt.production_cost_changes().clone();
                self.production_time_changes = pt.production_time_changes().clone();
                self.production_veterancy_levels = pt.production_veterancy_levels().clone();
                self.color = pt.preferred_color().get_as_int() | 0xff00_0000_u32 as Int;
                self.night_color = self.color;
                self.money = pt.money().clone();
                self.money.set_player_index(self.index);
                self.handicap = pt.handicap().clone();
                if self.money.count_money() == 0 {
                    let cash = env.world.starting_cash();
                    self.money
                        .deposit(cash, FALSE, FALSE, &mut Income(&mut *env.world));
                }
                self.display_name.clear();
                self.name.clear();
                self.name_key = NameKeyType::default();
                self.player_type = PlayerType::Computer;
                self.observer = pt.is_observer() != FALSE;
                self.is_dead = self.observer;
            }
            None => {
                self.side.clear();
                self.base_side.clear();
                self.production_cost_changes.clear();
                self.production_time_changes.clear();
                self.production_veterancy_levels.clear();
                self.color = NEUTRAL_PLAYER_COLOR;
                self.night_color = NEUTRAL_PLAYER_COLOR;
                self.money.init();
                self.handicap.init();
                self.display_name.clear();
                self.name.clear();
                self.name_key = env.name_keys.name_to_key("");
                self.player_type = PlayerType::Computer;
                self.set_player_relationship(self.index, Relationship::Allies);
            }
        }
        self.score_keeper.reset(self.index);
        self.template_name_key = template.map(PlayerTemplate::name_key);

        self.reset_rank(template, env);
        self.sciences_disabled.clear();
        self.sciences_hidden.clear();
        self.special_power_timers.clear();
        self.production_changes.clear();
        env.world.academy_init(self.index);
        self.logical_retaliation_mode = false;
    }

    /// `resetRank` and `resetSciences`.
    pub fn reset_rank(&mut self, template: Option<&PlayerTemplate>, env: &mut PlayerEnv<'_>) {
        self.rank_level = 1;
        self.skill_points = 0;
        self.level_up = env
            .ranks
            .get_rank_info(2)
            .map_or(Int::MAX, |r| r.skill_points_needed);
        self.level_down = 0;
        self.sciences.clear();
        self.science_purchase_points =
            template.map_or(0, |t| t.intrinsic_science_purchase_points());
        self.science_purchase_points = self.science_purchase_points.wrapping_add(
            env.ranks
                .get_rank_info(1)
                .map_or(0, |r| r.science_purchase_points_granted),
        );
        self.general_name = env.world.general_name();

        self.sciences = template.map_or_else(Vec::new, |t| t.intrinsic_sciences().clone());
        for level in 1..=self.rank_level {
            if let Some(rank) = env.ranks.get_rank_info(level) {
                for &science in &rank.sciences_granted {
                    self.add_science(science, env.world);
                }
            }
        }
        for &science in &self.sciences {
            env.world.science_added(self.index, science);
        }
    }

    pub fn has_science(&self, science: ScienceType) -> bool {
        self.sciences.contains(&science)
    }

    /// `addScience`.
    pub fn add_science(&mut self, science: ScienceType, world: &mut dyn PlayerWorld) -> bool {
        if self.has_science(science) {
            return false;
        }
        self.sciences.push(science);
        world.science_added(self.index, science);
        true
    }

    /// `setScienceAvailability`: 1 disables, 2 hides, anything else restores.
    pub fn set_science_availability(&mut self, science: ScienceType, availability: Int) {
        if let Some(at) = self.sciences_disabled.iter().position(|&s| s == science) {
            self.sciences_disabled.remove(at);
        } else if let Some(at) = self.sciences_hidden.iter().position(|&s| s == science) {
            self.sciences_hidden.remove(at);
        }
        match availability {
            1 => self.sciences_disabled.push(science),
            2 => self.sciences_hidden.push(science),
            _ => {}
        }
    }

    /// `addKindOfProductionCostChange`: repeats of a pair are counted.
    pub fn add_production_cost_change(&mut self, kind_of: KindOfMaskType, percent: Real) {
        match self
            .production_changes
            .iter_mut()
            .find(|c| c.percent == percent && c.kind_of == kind_of)
        {
            Some(change) => change.refs = change.refs.wrapping_add(1),
            None => self.production_changes.push(KindOfProductionChange {
                kind_of,
                percent,
                refs: 1,
            }),
        }
    }

    fn delete_ai(&mut self, world: &mut dyn PlayerWorld) {
        if self.ai.take().is_some() {
            world.delete_ai(self.index);
        }
    }

    /// `deletePlayerAI`.
    pub fn delete_player_ai(&mut self, world: &mut dyn PlayerWorld) {
        self.delete_ai(world);
    }

    fn delete_economy_trackers(&mut self, world: &mut dyn PlayerWorld) {
        if std::mem::take(&mut self.has_economy_trackers) {
            world.delete_economy_trackers(self.index);
        }
    }

    /// `setPlayerType`.
    pub fn set_player_type(
        &mut self,
        player_type: PlayerType,
        skirmish: bool,
        world: &mut dyn PlayerWorld,
    ) {
        self.player_type = player_type;
        self.delete_ai(world);
        if player_type == PlayerType::Computer {
            let kind = if skirmish || world.force_skirmish_ai() {
                AiKind::Skirmish
            } else {
                AiKind::Standard
            };
            self.ai = Some(kind);
            // `AIPlayer`'s constructor turns unit production off and
            // `AISkirmishPlayer`'s turns it back on.
            self.can_build_units = kind == AiKind::Skirmish;
            world.create_ai(self.index, kind);
        }
    }

    /// `initFromDict` for a side. A skirmish computer player takes the scripts and
    /// teams of its faction's skirmish side, renamed with its start index.
    pub fn init_from_dict(&mut self, d: &Dict, sides: &mut SidesList, env: &mut PlayerEnv<'_>) {
        let template = faction_template(d, env);
        self.init(template, env);

        let display_key = env.name_keys.name_to_key("playerDisplayName");
        self.display_name = d.get_unicode_string(display_key, None);
        let player_name = dict_string(d, env, "playerName");
        self.name = player_name.clone();
        self.name_key = env.name_keys.name_to_key(&player_name);

        let mut skirmish = false;
        let mut force_human = false;
        self.mp_start_index = dict_int(d, env, "multiplayerStartIndex").unwrap_or(0);
        if dict_bool(d, env, "playerIsSkirmish") {
            for side in &sides.skirmish_sides {
                if faction_template(&side.dict, env).is_some_and(|t| t.side() == self.side) {
                    skirmish = true;
                    break;
                }
            }
            if !skirmish {
                force_human = true;
            }
        }

        if dict_bool(d, env, "playerIsHuman") || force_human {
            self.set_player_type(PlayerType::Human, false, env.world);
            if dict_bool(d, env, "playerIsPreorder") {
                self.is_preorder = true;
            }
            if sides.num_skirmish_sides() > 0 {
                // A human player takes the civilian player's scripts.
                if let Some((source, template_name)) = self.skirmish_source(sides, "Civilian", env)
                {
                    if sides.skirmish_sides[source].scripts.is_some() {
                        self.copy_scripts(sides, source, &template_name, &player_name, env);
                        sides.skirmish_sides[source].scripts = None;
                    }
                }
            }
            skirmish = false;
        } else {
            self.set_player_type(PlayerType::Computer, skirmish, env.world);
        }

        if skirmish {
            let my_side = self.side.clone();
            let found = self.skirmish_source(sides, &my_side, env);
            let difficulty = dict_int(d, env, "skirmishDifficulty")
                .unwrap_or_else(|| env.world.global_difficulty());
            if self.ai.is_some() {
                env.world.set_ai_difficulty(self.index, difficulty);
            }
            if let Some((source, template_name)) = found {
                self.name = template_name.clone();
                self.copy_scripts(sides, source, &template_name, &player_name, env);
                self.copy_skirmish_teams(sides, source, &player_name, env);
            }
        }

        // `init` left neither manager.
        env.world.create_economy_trackers(self.index);
        self.has_economy_trackers = true;

        self.handicap.read_from_dict(d, env.name_keys);
        self.player_relations.clear();
        self.team_relations.clear();
        self.attacked_by = [false; MAX_PLAYER_COUNT];

        if let Some(color) = dict_int(d, env, "playerColor") {
            self.color = color | 0xff00_0000_u32 as Int;
            self.night_color = self.color;
        }
        if let Some(color) = dict_int(d, env, "playerNightColor") {
            self.night_color = color | 0xff00_0000_u32 as Int;
        }
        if let Some(money) = dict_int(d, env, "playerStartMoney") {
            // `Money::deposit` with its default sound and income tracking.
            self.money.deposit(
                money as UnsignedInt,
                TRUE,
                TRUE,
                &mut Income(&mut *env.world),
            );
        }
        self.squads = PlayerSquads::default();
    }

    /// The first skirmish side whose faction plays `side`, and the qualified name
    /// of its player.
    fn skirmish_source(
        &self,
        sides: &SidesList,
        side: &str,
        env: &mut PlayerEnv<'_>,
    ) -> Option<(usize, String)> {
        for (i, info) in sides.skirmish_sides.iter().enumerate() {
            if faction_template(&info.dict, env).is_some_and(|t| t.side() == side) {
                let name = dict_string(&info.dict, env, "playerName");
                return Some((i, format!("{name}{}", self.mp_start_index)));
            }
        }
        None
    }

    fn copy_scripts(
        &self,
        sides: &mut SidesList,
        source: usize,
        template_name: &str,
        player_name: &str,
        env: &mut PlayerEnv<'_>,
    ) {
        env.world.qualify_skirmish_scripts(
            sides,
            &SkirmishScripts {
                side: self.index,
                skirmish_side: source as Int,
                qualifier: &self.mp_start_index.to_string(),
                template_player_name: template_name,
                player_name,
            },
        );
    }

    /// Replaces this player's teams in the sides list with renamed copies of the
    /// skirmish player's teams.
    fn copy_skirmish_teams(
        &self,
        sides: &mut SidesList,
        source: usize,
        player_name: &str,
        env: &mut PlayerEnv<'_>,
    ) {
        let keys = WellKnownSidesKeys::intern(env.name_keys);
        let mut i = 0;
        while i < sides.num_teams() {
            let owned = sides.team_rec.teams[i as usize]
                .dict
                .get_ascii_string(keys.team_owner, None)
                == player_name;
            // The last team of the list cannot be removed.
            if owned && sides.num_teams() > 1 {
                sides.remove_team(i);
            } else {
                i += 1;
            }
        }

        let original = sides.skirmish_sides[source]
            .dict
            .get_ascii_string(keys.player_name, None);
        let suffix = self.mp_start_index;
        for i in 0..sides.skirmish_team_rec.teams.len() {
            let source_dict = &sides.skirmish_team_rec.teams[i].dict;
            if source_dict.get_ascii_string(keys.team_owner, None) != original {
                continue;
            }
            let mut team = source_dict.clone();
            let new_name = format!("{}{suffix}", team.get_ascii_string(keys.team_name, None));
            if sides.find_team_info(&new_name, &keys).is_some() {
                continue;
            }
            team.set_ascii_string(keys.team_owner, player_name);
            team.set_ascii_string(keys.team_name, &new_name);

            let mut script_keys: Vec<String> = [
                "teamOnCreateScript",
                "teamOnIdleScript",
                "teamOnUnitDestroyedScript",
                "teamOnDestroyedScript",
                "teamEnemySightedScript",
                "teamAllClearScript",
                "teamProductionCondition",
            ]
            .map(str::to_owned)
            .to_vec();
            let hook = env.name_keys.name_to_key("teamGenericScriptHook");
            let hook = env.name_keys.key_to_name(hook).to_owned();
            script_keys.extend((0..MAX_GENERIC_SCRIPTS).map(|j| format!("{hook}{j}")));
            for key in script_keys {
                let key = env.name_keys.name_to_key(&key);
                let mut exists: Bool = FALSE;
                let script = team.get_ascii_string(key, Some(&mut exists));
                if exists != FALSE && !script.is_empty() {
                    team.set_ascii_string(key, &format!("{script}{suffix}"));
                }
            }
            sides.add_team(Some(&team));
        }
    }

    /// `Player::getRelationship(const Team*)`; `that_player` is the team's
    /// controlling player.
    pub fn relationship(
        &self,
        that: Option<TeamId>,
        that_player: Option<PlayerIndex>,
    ) -> Relationship {
        if let Some(that) = that {
            if let Some(r) = self.team_relations.get(that.0 as u32) {
                return r;
            }
            if let Some(r) = that_player.and_then(|p| self.player_relations.get(p as u32)) {
                return r;
            }
        }
        Relationship::Neutral
    }

    pub fn set_player_relationship(&mut self, that: PlayerIndex, r: Relationship) {
        self.player_relations.set(that as u32, r);
    }

    /// `None` removes every entry.
    pub fn remove_player_relationship(&mut self, that: Option<PlayerIndex>) -> bool {
        if self.player_relations.is_empty() {
            return false;
        }
        match that {
            Some(p) => self.player_relations.remove(p as u32),
            None => {
                self.player_relations.clear();
                true
            }
        }
    }

    pub fn set_team_relationship(&mut self, that: TeamId, r: Relationship) {
        self.team_relations.set(that.0 as u32, r);
    }

    /// `None` removes every entry.
    pub fn remove_team_relationship(&mut self, that: Option<TeamId>) -> bool {
        if self.team_relations.is_empty() {
            return false;
        }
        match that {
            Some(t) => self.team_relations.remove(t.0 as u32),
            None => {
                self.team_relations.clear();
                true
            }
        }
    }

    pub fn add_team_to_list(&mut self, prototype: TeamPrototypeId) {
        if !self.team_prototypes.contains(&prototype) {
            self.team_prototypes.push(prototype);
        }
    }

    pub fn remove_team_from_list(&mut self, prototype: TeamPrototypeId) {
        if let Some(at) = self.team_prototypes.iter().position(|&p| p == prototype) {
            self.team_prototypes.remove(at);
        }
    }

    pub fn pre_team_destroy(&mut self, team: TeamId, world: &mut dyn PlayerWorld) {
        if self.ai.is_some() {
            world.ai_pre_team_destroy(self.index, team);
        }
        if self.default_team == Some(team) {
            self.default_team = None;
        }
    }

    pub fn new_map(&mut self, world: &mut dyn PlayerWorld) {
        if self.ai.is_some() {
            world.ai_new_map(self.index);
        }
    }

    /// `Player::crc`.
    pub fn crc(&self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut present = self.battle_plan_bonuses.is_some();
        flag(xfer, &mut present)?;
        if let Some(bonuses) = &self.battle_plan_bonuses {
            xfer_battle_plan_values(xfer, &mut bonuses.clone())?;
        }
        let mut skill_points = self.skill_points;
        xfer.xfer_int(&mut skill_points)?;
        let mut points = self.science_purchase_points;
        xfer.xfer_int(&mut points)
    }

    /// `Player::xfer`. `teams` resolves prototype and team ids when loading.
    pub fn xfer(
        &mut self,
        xfer: &mut dyn Xfer,
        teams: &TeamFactory,
        env: &mut PlayerEnv<'_>,
    ) -> Result<(), XferError> {
        let load = xfer.mode() == XferMode::Load;
        let mut version: XferVersion = 8;
        xfer.xfer_version(&mut version, 8)?;
        self.money.xfer(xfer)?;
        let mut upgrade_count = env.world.upgrade_count(self.index);
        xfer.xfer_unsigned_short(&mut upgrade_count)?;
        if version >= 7 {
            flag(xfer, &mut self.is_preorder)?;
        }
        if version >= 8 {
            xfer_science_vec(xfer, env, &mut self.sciences_disabled)?;
            xfer_science_vec(xfer, env, &mut self.sciences_hidden)?;
        }
        env.world
            .xfer_player_part(self.index, PlayerPart::Upgrades, xfer)?;
        xfer.xfer_int(&mut self.radar_count)?;
        flag(xfer, &mut self.is_dead)?;
        xfer.xfer_int(&mut self.disable_proof_radar_count)?;
        flag(xfer, &mut self.radar_disabled)?;
        let upgrades = env.world.upgrade_masks();
        xfer_upgrade_mask(xfer, &upgrades, &mut self.upgrades_in_progress)?;
        xfer_upgrade_mask(xfer, &upgrades, &mut self.upgrades_completed)?;
        self.energy.xfer(xfer)?;

        let mut count = self.team_prototypes.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if load {
            self.team_prototypes.clear();
            for _ in 0..count {
                let mut id = 0;
                xfer.xfer_unsigned_int(&mut id)?;
                let id = teams
                    .find_team_prototype_by_id(id)
                    .ok_or(XferError::InvalidData)?;
                self.team_prototypes.push(id);
            }
        } else {
            for &(mut id) in &self.team_prototypes {
                xfer.xfer_unsigned_int(&mut id)?;
            }
        }

        let mut count = self.build_list.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if load {
            self.build_list.clear();
            for _ in 0..count {
                let mut info = BuildListInfo::new();
                info.xfer(xfer)?;
                self.build_list.push(info);
            }
        } else {
            for info in &mut self.build_list {
                info.xfer(xfer)?;
            }
        }

        xfer_presence(xfer, self.ai.is_some())?;
        if self.ai.is_some() {
            env.world
                .xfer_player_part(self.index, PlayerPart::Ai, xfer)?;
        }
        for part in [PlayerPart::ResourceGathering, PlayerPart::TunnelTracker] {
            xfer_presence(xfer, self.has_economy_trackers)?;
            if self.has_economy_trackers {
                env.world.xfer_player_part(self.index, part, xfer)?;
            }
        }

        let mut default_team = self.default_team.unwrap_or(TeamId::INVALID);
        xfer_team_id(xfer, &mut default_team)?;
        if load {
            self.default_team = teams.team(default_team).map(|t| t.id);
        }

        if version >= 5 {
            if load {
                self.sciences.clear();
            }
            xfer_science_vec(xfer, env, &mut self.sciences)?;
        } else {
            // Old saves hold raw ids.
            let mut count = self.sciences.len() as UnsignedShort;
            xfer.xfer_unsigned_short(&mut count)?;
            if load {
                for _ in 0..count {
                    let mut science = SCIENCE_INVALID;
                    xfer.xfer_int(&mut science)?;
                    self.sciences.push(science);
                }
            } else {
                for &(mut science) in &self.sciences {
                    xfer.xfer_int(&mut science)?;
                }
            }
        }
        xfer.xfer_int(&mut self.rank_level)?;
        xfer.xfer_int(&mut self.skill_points)?;
        xfer.xfer_int(&mut self.science_purchase_points)?;
        xfer.xfer_int(&mut self.level_up)?;
        xfer.xfer_int(&mut self.level_down)?;
        xfer.xfer_unicode_string(&mut self.general_name)?;
        self.player_relations.xfer(xfer)?;
        self.team_relations.xfer(xfer)?;
        flag(xfer, &mut self.can_build_units)?;
        flag(xfer, &mut self.can_build_base)?;
        flag(xfer, &mut self.observer)?;
        if version >= 2 {
            xfer.xfer_real(&mut self.skill_points_modifier)?;
        } else {
            self.skill_points_modifier = 1.0;
        }
        if version >= 3 {
            flag(xfer, &mut self.list_in_score_screen)?;
        } else {
            self.list_in_score_screen = true;
        }
        let mut attacked = self.attacked_by.map(u8::from);
        xfer.xfer_user(&mut attacked)?;
        self.attacked_by = attacked.map(|b| b != 0);
        xfer.xfer_real(&mut self.cash_bounty_percent)?;
        self.score_keeper.xfer(xfer)?;

        let mut count = self.production_changes.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if load {
            if !self.production_changes.is_empty() {
                return Err(XferError::InvalidData);
            }
            self.production_changes = (0..count)
                .map(|_| KindOfProductionChange {
                    kind_of: KindOfMaskType::default(),
                    percent: 0.0,
                    refs: 0,
                })
                .collect();
        }
        for change in &mut self.production_changes {
            xfer_kind_of(xfer, &mut change.kind_of)?;
            xfer.xfer_real(&mut change.percent)?;
            xfer.xfer_unsigned_int(&mut change.refs)?;
        }

        if version < 4 {
            self.special_power_timers.clear();
        } else {
            let mut count = self.special_power_timers.len() as UnsignedShort;
            xfer.xfer_unsigned_short(&mut count)?;
            if load {
                if !self.special_power_timers.is_empty() {
                    return Err(XferError::InvalidData);
                }
                self.special_power_timers = vec![SpecialPowerReadyTimer::default(); count.into()];
            }
            for timer in &mut self.special_power_timers {
                xfer.xfer_unsigned_int(&mut timer.template_id)?;
                xfer.xfer_unsigned_int(&mut timer.ready_frame)?;
            }
        }

        let mut squads = self.squads.hotkeys.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut squads)?;
        if usize::from(squads) != self.squads.hotkeys.len() {
            return Err(XferError::InvalidData);
        }
        for squad in &mut self.squads.hotkeys {
            xfer_squad(xfer, squad)?;
        }
        let mut selection = true;
        flag(xfer, &mut selection)?;
        if selection {
            xfer_squad(xfer, &mut self.squads.selection)?;
        }

        let mut present = self.battle_plan_bonuses.is_some();
        flag(xfer, &mut present)?;
        if load {
            self.battle_plan_bonuses = present.then(BattlePlanBonuses::default);
        }
        if let Some(bonuses) = &mut self.battle_plan_bonuses {
            xfer_battle_plan_values(xfer, bonuses)?;
        }
        xfer.xfer_int(&mut self.bombard_battle_plans)?;
        xfer.xfer_int(&mut self.hold_the_line_battle_plans)?;
        xfer.xfer_int(&mut self.search_and_destroy_battle_plans)?;
        if version >= 6 {
            flag(xfer, &mut self.units_should_hunt)?;
        } else {
            self.units_should_hunt = false;
        }
        Ok(())
    }
}
