//! `PKT-433`: `ScriptEngine`, which runs mission and computer player scripts.
//! Reference: `GameLogic/ScriptEngine/ScriptEngine.cpp` and `ScriptEngine.h`.
//! Golden test: `tests/golden_script_engine.rs`.
//!
//! The engine owns the script lists of `TheSidesList` and everything `xfer`
//! saves. It reaches players, teams and objects through [`ScriptWorld`]. What
//! the reference keeps as `Parameter` caches (counter and flag numbers, player
//! masks) and `Condition` custom data is written back into the lists, as there.
//!
//! A condition or action that is not ported is not answered here: it goes to
//! [`ScriptWorld::unported_condition`] or [`ScriptWorld::unported_action`] and
//! is counted in [`ScriptEngine::unported_conditions`] and
//! [`ScriptEngine::unported_actions`].
//!
//! Left out because it is not game state: the debug window and particle editor
//! hooks, debug messages, profiling, and `ScriptActions::update` and
//! `ScriptConditions::update`, which are empty.
//!
//! Known difference: the reference iterates an attack priority set by
//! `ThingTemplate` address. Here it is ordered by the template number the world
//! gives, so only the save bytes of a set with two or more entries can differ.

use std::collections::BTreeMap;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{convert_duration_from_msecs_to_frames, MSEC_PER_LOGICFRAME_REAL};
use zh_common::math::gamemath::{gm_cosf, gm_sinf, real_to_int_ceil};
use zh_common::math::Coord3D;
use zh_common::types::{Int, ObjectId, Real, UnsignedInt, UnsignedShort, XferVersion};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

use super::script_types::{
    action_type as at, condition_type as ct, ACTION_TEMPLATES, CONDITION_TEMPLATES,
};
use super::scripts::{
    Condition, Parameter, ParameterType, Script, ScriptAction, ScriptList, ScriptTemplate,
    ScriptTemplateRegistry, LOCAL_PLAYER, TEAM_THE_PLAYER, THE_PLAYER, THIS_OBJECT, THIS_PLAYER,
    THIS_PLAYER_ENEMY, THIS_TEAM,
};
use crate::gamelogic::global_data::GameDifficulty;
use crate::object::object_types::ObjectTypes;
use crate::rules::science::ScienceType;

pub const MAX_COUNTERS: usize = 256;
pub const MAX_FLAGS: usize = 256;
pub const MAX_ATTACK_PRIORITIES: usize = 256;
pub const SCRIPT_PLAYER_COUNT: usize = 16;
pub const ATTACK_PRIORITY_DEFAULT: Int = 1;
const FRAMES_TO_SHOW_WIN_LOSE_MESSAGE: Int = 120;
const FRAMES_TO_FADE_IN_AT_START: Int = 33;
const MAX_SPIN_COUNT: Int = 20;
const LOGICFRAMES_PER_SECOND: Int = 30;
const PI: Real = std::f32::consts::PI;
const NONE_STRING: &str = "<none>";

/// `TeamID`.
pub type ScriptTeamId = UnsignedInt;
/// Index into `ThePlayerList`.
pub type ScriptPlayer = Int;

/// `TeamPrototype` as the engine reads it.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TeamPrototypeInfo {
    pub singleton: bool,
    pub max_instances: Int,
    /// `iterate_TeamInstanceList` order.
    pub instances: Vec<ScriptTeamId>,
}

/// The readings of `Player::getEnergy` the conditions use.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PowerReading {
    pub production: Int,
    pub consumption: Int,
    pub sufficient: bool,
    pub supply_ratio: Real,
}

/// Calls into the client and the game flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptUiCommand {
    /// `ScriptActions::closeWindows` and `GameLogic::closeWindows`.
    CloseWindows,
    DisableInput,
    ShowVictory,
    ShowDefeat,
    ShowLocalDefeat,
    /// `CampaignManager::SetVictorious`.
    SetVictorious(bool),
    /// `rts::enableLetterBox`.
    LetterBox(bool),
}

/// What the script engine needs from the rest of the game. No method has a
/// default: a host must say what it does with each.
pub trait ScriptWorld {
    fn frame(&self) -> UnsignedInt;
    /// `GameLogicRandomValue(lo, hi)`.
    fn logic_random(&mut self, lo: Int, hi: Int) -> Int;

    // Players. Indices are those of `ThePlayerList`.
    fn player_count(&self) -> Int;
    fn find_player(&self, name: &str) -> Option<ScriptPlayer>;
    fn local_player(&self) -> ScriptPlayer;
    /// A challenge campaign mission is running.
    fn is_challenge_campaign(&self) -> bool;
    fn player_default_team(&self, player: ScriptPlayer) -> Option<ScriptTeamId>;
    fn player_name(&self, player: ScriptPlayer) -> String;
    fn player_is_human(&self, player: ScriptPlayer) -> bool;
    fn player_is_skirmish_ai(&self, player: ScriptPlayer) -> bool;
    fn player_is_active(&self, player: ScriptPlayer) -> bool;
    /// The difficulty of the player's AI; `None` for a player without one, who
    /// plays at the engine's.
    fn player_difficulty(&self, player: ScriptPlayer) -> Option<GameDifficulty>;
    fn player_current_enemy(&self, player: ScriptPlayer) -> Option<ScriptPlayer>;
    fn player_start_index(&self, player: ScriptPlayer) -> Int;
    fn player_side(&self, player: ScriptPlayer) -> String;
    fn player_money(&self, player: ScriptPlayer) -> UnsignedInt;
    fn player_power(&self, player: ScriptPlayer) -> PowerReading;
    fn player_has_any_objects(&self, player: ScriptPlayer) -> bool;
    fn player_has_any_build_facility(&self, player: ScriptPlayer) -> bool;
    fn player_count_buildings(&self, player: ScriptPlayer) -> Int;
    /// `countObjects` with `STRUCTURE` and `MP_COUNT_FOR_VICTORY` set.
    fn player_count_faction_buildings(&self, player: ScriptPlayer) -> Int;
    /// `countObjectsByThingTemplate` over the named templates together.
    fn player_count_objects_by_template(
        &self,
        player: ScriptPlayer,
        templates: &[String],
        ignore_dead: bool,
    ) -> Vec<Int>;
    fn player_attacked_by(&self, player: ScriptPlayer, attacker: ScriptPlayer) -> bool;

    /// `Money::withdraw`.
    fn money_withdraw(&mut self, player: ScriptPlayer, amount: UnsignedInt);
    /// `Money::deposit`.
    fn money_deposit(
        &mut self,
        player: ScriptPlayer,
        amount: UnsignedInt,
        play_sound: bool,
        count_in_score: bool,
    );
    fn set_can_build_units(&mut self, player: ScriptPlayer, can: bool);
    fn set_can_build_base(&mut self, player: ScriptPlayer, can: bool);
    fn set_player_relationship(&mut self, from: ScriptPlayer, to: ScriptPlayer, relationship: Int);
    /// `Player::grantScience`; true if the player did not have it. The host
    /// must not report it back: the engine records it itself.
    fn grant_science(&mut self, player: ScriptPlayer, science: ScienceType) -> bool;
    /// `Player::killPlayer`.
    fn kill_player(&mut self, player: ScriptPlayer);

    // Teams.
    fn team_prototype(&self, name: &str) -> Option<TeamPrototypeInfo>;
    fn team_name(&self, team: ScriptTeamId) -> String;
    fn team_exists(&self, team: ScriptTeamId) -> bool;
    fn team_is_active(&self, team: ScriptTeamId) -> bool;
    fn team_is_created(&self, team: ScriptTeamId) -> bool;
    fn team_state(&self, team: ScriptTeamId) -> String;
    fn team_controlling_player(&self, team: ScriptTeamId) -> Option<ScriptPlayer>;
    fn team_has_any_objects(&self, team: ScriptTeamId) -> bool;
    fn team_has_any_units(&self, team: ScriptTeamId) -> bool;
    /// `iterate_TeamMemberList` order.
    fn team_members(&self, team: ScriptTeamId) -> Vec<ObjectId>;
    /// `ThePlayerList->updateTeamStates`.
    fn update_team_states(&mut self);
    /// `getTeamAsAIGroup` followed by `AIGroup::isIdle`.
    fn team_group_is_idle(&mut self, team: ScriptTeamId) -> bool;
    /// `AIGroup::isGroupAiDead`.
    fn team_group_is_ai_dead(&mut self, team: ScriptTeamId) -> bool;
    /// `AIGroup::groupIdle(CMD_FROM_SCRIPT)`.
    fn team_group_idle(&mut self, team: ScriptTeamId);
    fn set_team_state(&mut self, team: ScriptTeamId, state: &str);
    fn delete_team(&mut self, team: ScriptTeamId, ignore_dead: bool);
    fn kill_team(&mut self, team: ScriptTeamId);

    // Objects.
    /// Every named object in the order of the game logic object list.
    fn named_objects(&self) -> Vec<(String, ObjectId)>;
    fn object_exists(&self, id: ObjectId) -> bool;
    fn object_name(&self, id: ObjectId) -> String;
    fn object_is_effectively_dead(&self, id: ObjectId) -> bool;
    fn object_controlling_player(&self, id: ObjectId) -> Option<ScriptPlayer>;
    /// Body health and initial health, if the object has a body.
    fn object_health(&self, id: ObjectId) -> Option<(Real, Real)>;
    /// `None` if the object has no `AIUpdateInterface`, else `isIdle`.
    fn object_ai_is_idle(&self, id: ObjectId) -> Option<bool>;
    /// `aiIdle(CMD_FROM_SCRIPT)`.
    fn object_ai_idle(&mut self, id: ObjectId);
    /// `aiGuardPosition` of the object's own position, normal guard mode.
    fn object_ai_guard_here(&mut self, id: ObjectId, choose_normal_locomotor: bool);
    fn set_object_name(&mut self, id: ObjectId, name: &str);
    /// Gives `to` the custom indicator color of `from`, or none if it has none.
    fn copy_indicator_color(&mut self, from: ObjectId, to: ObjectId);
    fn destroy_object(&mut self, id: ObjectId);
    /// `Object::kill`.
    fn kill_object(&mut self, id: ObjectId);
    /// `setReceivingDifficultyBonus` on every object.
    fn set_all_difficulty_bonuses(&mut self, enable: bool);

    // Things, sciences, terrain, audio.
    fn has_thing_template(&self, name: &str) -> bool;
    /// The template's number in creation order.
    fn thing_template_index(&self, name: &str) -> Option<u32>;
    fn thing_template_name(&self, index: u32) -> String;
    /// Numbers of every template with the `KindOfType`.
    fn thing_templates_of_kind(&self, kind: Int) -> Vec<u32>;
    fn science_from_name(&self, name: &str) -> Option<ScienceType>;
    fn science_name(&self, science: ScienceType) -> String;
    fn waypoint_location(&self, name: &str) -> Option<Coord3D>;
    fn do_shroud_reveal(&mut self, at: Coord3D, radius: Real, player: ScriptPlayer);
    fn undo_shroud_reveal(&mut self, at: Coord3D, radius: Real, player: ScriptPlayer);
    fn audio_length_ms(&self, event: &str) -> Real;

    // The client and the game flow.
    fn ui(&mut self, command: ScriptUiCommand);
    fn exit_game(&mut self);

    // Not ported.
    /// `ScriptConditions::evaluateSkirmishCommandButtonIsReady` for a team's
    /// sequential script.
    fn command_button_ready(&mut self, team: &Parameter, button: &Parameter, all: bool) -> bool;
    /// `ScriptConditions::evaluateTeamIsContained`.
    fn team_is_contained(&mut self, team: &Parameter, all: bool) -> bool;
    /// A condition this port does not evaluate. The result is the host's.
    fn unported_condition(&mut self, condition: &Condition) -> bool;
    /// An action this port does not carry out.
    fn unported_action(&mut self, action: &ScriptAction);
}

/// `TCounter`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Counter {
    pub value: Int,
    pub name: String,
    pub is_countdown_timer: bool,
}

/// `TFlag`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Flag {
    pub value: bool,
    pub name: String,
}

/// `AttackPriorityInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct AttackPriorityInfo {
    pub name: String,
    pub default_priority: Int,
    /// Thing template number to priority.
    pub priorities: BTreeMap<u32, Int>,
}

impl Default for AttackPriorityInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            default_priority: ATTACK_PRIORITY_DEFAULT,
            priorities: BTreeMap::new(),
        }
    }
}

impl AttackPriorityInfo {
    /// `getPriority`.
    pub fn priority(&self, template: Option<u32>) -> Int {
        template
            .and_then(|t| self.priorities.get(&t).copied())
            .unwrap_or(self.default_priority)
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, world: &dyn ScriptWorld) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_ascii_string(&mut self.name)?;
        xfer.xfer_int(&mut self.default_priority)?;
        let mut count = self.priorities.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Load {
            for _ in 0..count {
                let mut name = String::new();
                xfer.xfer_ascii_string(&mut name)?;
                let index = world
                    .thing_template_index(&name)
                    .ok_or(XferError::InvalidData)?;
                let mut priority = 0;
                xfer.xfer_int(&mut priority)?;
                self.priorities.insert(index, priority);
            }
        } else {
            for (&index, priority) in self.priorities.iter() {
                let mut name = world.thing_template_name(index);
                xfer.xfer_ascii_string(&mut name)?;
                let mut priority = *priority;
                xfer.xfer_int(&mut priority)?;
            }
        }
        Ok(())
    }
}

/// Where a script sits in the side lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptRef {
    pub side: usize,
    pub group: Option<usize>,
    pub index: usize,
}

/// `SequentialScript`. A chain is a vector, head first.
#[derive(Debug, Clone, PartialEq)]
pub struct SequentialScript {
    pub team: Option<ScriptTeamId>,
    pub object_id: ObjectId,
    pub script: Option<ScriptRef>,
    pub current_instruction: Int,
    pub times_to_loop: Int,
    pub frames_to_wait: Int,
    pub dont_advance_instruction: bool,
}

impl Default for SequentialScript {
    fn default() -> Self {
        Self {
            team: None,
            object_id: ObjectId::INVALID,
            script: None,
            current_instruction: -1,
            times_to_loop: 0,
            frames_to_wait: -1,
            dont_advance_instruction: false,
        }
    }
}

/// `NamedReveal`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NamedReveal {
    pub reveal_name: String,
    pub waypoint_name: String,
    pub radius: Real,
    pub player_name: String,
}

/// `BreezeInfo`, which the client's trees read.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BreezeInfo {
    pub direction: Real,
    pub direction_x: Real,
    pub direction_y: Real,
    pub intensity: Real,
    pub lean: Real,
    pub randomness: Real,
    pub period: i16,
    pub version: i16,
}

/// `ScriptEngine::TFade`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptFade {
    #[default]
    None = 0,
    Subtract = 1,
    Add = 2,
    Saturate = 3,
    Multiply = 4,
}

impl ScriptFade {
    fn from_i32(v: Int) -> Self {
        match v {
            1 => Self::Subtract,
            2 => Self::Add,
            3 => Self::Saturate,
            4 => Self::Multiply,
            _ => Self::None,
        }
    }
}

/// `std::list<std::pair<AsciiString, ObjectID>>` per player.
pub type PlayerEvents = [Vec<(String, ObjectId)>; SCRIPT_PLAYER_COUNT];

/// `ScriptEngine`.
pub struct ScriptEngine {
    /// `TheSidesList`'s script list of each side.
    pub sides: Vec<Option<ScriptList>>,
    pub counters: Vec<Counter>,
    pub num_counters: Int,
    pub flags: Vec<Flag>,
    pub num_flags: Int,
    pub attack_priorities: Vec<AttackPriorityInfo>,
    pub num_attack_info: Int,
    pub end_game_timer: Int,
    pub close_window_timer: Int,
    pub letter_box_active: bool,
    pub calling_team: Option<ScriptTeamId>,
    pub calling_object: Option<ObjectId>,
    pub condition_team: Option<ScriptTeamId>,
    pub condition_object: Option<ObjectId>,
    pub named_objects: Vec<(String, Option<ObjectId>)>,
    pub first_update: bool,
    pub current_player: Option<ScriptPlayer>,
    pub current_track_name: String,
    pub fade: ScriptFade,
    pub min_fade: Real,
    pub max_fade: Real,
    pub cur_fade_value: Real,
    pub cur_fade_frame: Int,
    pub fade_frames_increase: Int,
    pub fade_frames_hold: Int,
    pub fade_frames_decrease: Int,
    pub frame_object_count_changed: UnsignedInt,
    pub object_counts: [BTreeMap<String, Int>; SCRIPT_PLAYER_COUNT],
    pub completed_video: Vec<String>,
    pub testing_speech: Vec<(String, UnsignedInt)>,
    pub testing_audio: Vec<(String, UnsignedInt)>,
    pub ui_interactions: Vec<String>,
    pub triggered_special_powers: PlayerEvents,
    pub midway_special_powers: PlayerEvents,
    pub finished_special_powers: PlayerEvents,
    pub completed_upgrades: PlayerEvents,
    pub acquired_sciences: [Vec<ScienceType>; SCRIPT_PLAYER_COUNT],
    pub topple_directions: Vec<(String, Coord3D)>,
    pub named_reveals: Vec<NamedReveal>,
    pub breeze: BreezeInfo,
    pub difficulty: GameDifficulty,
    pub freeze_by_script: bool,
    pub object_type_lists: Vec<ObjectTypes>,
    pub objects_receive_difficulty_bonus: bool,
    pub choose_victim_always_uses_normal: bool,
    pub shown_mp_local_defeat_window: bool,
    pub sequential: Vec<Vec<SequentialScript>>,
    /// Condition type to the times it went to the host as unported.
    pub unported_conditions: BTreeMap<Int, u64>,
    /// Action type to the times it went to the host as unported.
    pub unported_actions: BTreeMap<Int, u64>,
}

impl Default for ScriptEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// The templates `ScriptEngine::init` registers, for reading script chunks.
pub fn script_template_registry(names: &mut NameKeyGenerator) -> ScriptTemplateRegistry {
    let mut registry = ScriptTemplateRegistry::new();
    for (rows, templates) in [
        (CONDITION_TEMPLATES, &mut registry.conditions),
        (ACTION_TEMPLATES, &mut registry.actions),
    ] {
        for &(index, name, parms) in rows {
            let t: &mut ScriptTemplate = &mut templates[index as usize];
            t.internal_name = name.to_string();
            t.internal_name_key = names.name_to_key(name);
            t.num_parameters = parms.len() as Int;
            for (slot, &p) in t.parameters.iter_mut().zip(parms) {
                *slot = ParameterType::from_i32(p);
            }
        }
    }
    registry
}

/// Parameter `i`, or an unset one where the reference would read past the end.
pub(crate) fn parm(parms: &[Parameter], i: usize) -> Parameter {
    parms
        .get(i)
        .cloned()
        .unwrap_or_else(|| Parameter::new(ParameterType::Int, 0))
}

impl ScriptEngine {
    /// The constructor followed by `reset`, as `init` and the first load leave it.
    pub fn new() -> Self {
        let mut engine = Self {
            sides: Vec::new(),
            counters: vec![Counter::default(); MAX_COUNTERS],
            num_counters: 0,
            flags: vec![Flag::default(); MAX_FLAGS],
            num_flags: 0,
            attack_priorities: vec![AttackPriorityInfo::default(); MAX_ATTACK_PRIORITIES],
            num_attack_info: 0,
            end_game_timer: 0,
            close_window_timer: 0,
            letter_box_active: false,
            calling_team: None,
            calling_object: None,
            condition_team: None,
            condition_object: None,
            named_objects: Vec::new(),
            first_update: true,
            current_player: None,
            current_track_name: String::new(),
            fade: ScriptFade::None,
            min_fade: 0.0,
            max_fade: 0.0,
            cur_fade_value: 0.0,
            cur_fade_frame: 0,
            fade_frames_increase: 0,
            fade_frames_hold: 0,
            fade_frames_decrease: 0,
            frame_object_count_changed: 0,
            object_counts: Default::default(),
            completed_video: Vec::new(),
            testing_speech: Vec::new(),
            testing_audio: Vec::new(),
            ui_interactions: Vec::new(),
            triggered_special_powers: Default::default(),
            midway_special_powers: Default::default(),
            finished_special_powers: Default::default(),
            completed_upgrades: Default::default(),
            acquired_sciences: Default::default(),
            topple_directions: Vec::new(),
            named_reveals: Vec::new(),
            breeze: BreezeInfo::default(),
            difficulty: GameDifficulty::Normal,
            freeze_by_script: false,
            object_type_lists: Vec::new(),
            objects_receive_difficulty_bonus: true,
            choose_victim_always_uses_normal: false,
            shown_mp_local_defeat_window: false,
            sequential: Vec::new(),
            unported_conditions: BTreeMap::new(),
            unported_actions: BTreeMap::new(),
        };
        engine.reset();
        engine
    }

    /// `reset`. The script lists are dropped with `TheSidesList`'s.
    pub fn reset(&mut self) {
        self.num_counters = 1;
        self.num_attack_info = 1;
        self.num_flags = 1;
        self.end_game_timer = -1;
        self.close_window_timer = -1;
        self.letter_box_active = false;
        self.calling_team = None;
        self.calling_object = None;
        self.condition_team = None;
        self.condition_object = None;
        self.current_player = None;
        self.frame_object_count_changed = 0;
        self.shown_mp_local_defeat_window = false;
        self.counters.fill(Counter::default());
        self.flags.fill(Flag::default());
        let direction = PI / 3.0;
        self.breeze = BreezeInfo {
            direction,
            direction_x: gm_sinf(direction),
            direction_y: gm_cosf(direction),
            intensity: 0.07 * PI / 4.0,
            lean: 0.07 * PI / 4.0,
            randomness: 0.2,
            period: (LOGICFRAMES_PER_SECOND * 5) as i16,
            version: 0,
        };
        self.freeze_by_script = false;
        self.objects_receive_difficulty_bonus = true;
        self.choose_victim_always_uses_normal = false;
        self.sequential.clear();
        self.object_type_lists.clear();
        self.named_reveals.clear();
        self.named_objects.clear();
        self.clear_event_lists();
        self.attack_priorities.fill(AttackPriorityInfo::default());
        for counts in &mut self.object_counts {
            counts.clear();
        }
        self.topple_directions.clear();
        self.sides.clear();
    }

    fn clear_event_lists(&mut self) {
        self.completed_video.clear();
        self.testing_speech.clear();
        self.testing_audio.clear();
        self.ui_interactions.clear();
        for i in 0..SCRIPT_PLAYER_COUNT {
            self.triggered_special_powers[i].clear();
            self.midway_special_powers[i].clear();
            self.finished_special_powers[i].clear();
            self.acquired_sciences[i].clear();
            self.completed_upgrades[i].clear();
        }
    }

    /// `newMap`, with the side script lists of the new map.
    pub fn new_map(&mut self, world: &mut dyn ScriptWorld, sides: Vec<Option<ScriptList>>) {
        self.sides = sides;
        self.num_counters = 1;
        self.counters.fill(Counter::default());
        self.num_flags = 1;
        self.flags.fill(Flag::default());
        self.end_game_timer = -1;
        self.close_window_timer = -1;
        self.clear_event_lists();
        for list in self.sides.iter_mut().flatten() {
            for script in list.scripts.iter_mut() {
                check_conditions_for_team_names(world, script);
            }
            for group in list.groups.iter_mut() {
                for script in group.scripts.iter_mut() {
                    check_conditions_for_team_names(world, script);
                }
            }
        }
        self.first_update = true;
        self.start_fade_in();
    }

    fn start_fade_in(&mut self) {
        self.fade = ScriptFade::Multiply;
        self.cur_fade_frame = 0;
        self.min_fade = 1.0;
        self.max_fade = 0.0;
        self.fade_frames_increase = 0;
        self.fade_frames_hold = 0;
        self.fade_frames_decrease = FRAMES_TO_FADE_IN_AT_START;
        self.cur_fade_value = 0.0;
    }

    /// `update`, once per logic frame.
    pub fn update(&mut self, world: &mut dyn ScriptWorld) {
        if self.first_update {
            self.create_named_cache(world);
            self.first_update = false;
        }
        if self.close_window_timer > 0 {
            self.close_window_timer -= 1;
            if self.close_window_timer < 1 {
                world.ui(ScriptUiCommand::CloseWindows);
            }
        }
        if self.end_game_timer > 0 {
            self.end_game_timer -= 1;
            if self.end_game_timer < 1 {
                world.exit_game();
            }
        }
        if self.fade != ScriptFade::None {
            self.update_fades();
        }
        if self.end_game_timer >= 0 {
            return;
        }
        for i in 1..self.num_counters as usize {
            let counter = &mut self.counters[i];
            if counter.is_countdown_timer && counter.value >= 0 {
                counter.value -= 1;
            }
        }
        for side in 0..self.sides.len() {
            self.current_player = Some(side as Int);
            if self.sides[side].is_none() {
                continue;
            }
            self.execute_list(world, side, None);
            let groups = self.sides[side].as_ref().map_or(0, |l| l.groups.len());
            for group in 0..groups {
                let Some(g) = self.sides[side].as_ref().and_then(|l| l.groups.get(group)) else {
                    continue;
                };
                if !on(g.is_group_active) || on(g.is_group_subroutine) {
                    continue;
                }
                self.execute_list(world, side, Some(group));
            }
            self.current_player = None;
        }
        world.update_team_states();
        self.ui_interactions.clear();
        self.evaluate_sequential_scripts(world);
    }

    fn update_fades(&mut self) {
        self.cur_fade_frame = self.cur_fade_frame.wrapping_add(1);
        let mut fade = self.cur_fade_frame;
        if fade <= self.fade_frames_increase {
            let factor = self.cur_fade_frame as Real / self.fade_frames_increase as Real;
            self.cur_fade_value = self.min_fade + factor * (self.max_fade - self.min_fade);
            return;
        }
        fade = fade.wrapping_sub(self.fade_frames_increase);
        if fade <= self.fade_frames_hold {
            self.cur_fade_value = self.max_fade;
            return;
        }
        fade = fade.wrapping_sub(self.fade_frames_hold);
        if fade <= self.fade_frames_decrease {
            let mut divisor = self.fade_frames_decrease.wrapping_add(1);
            if divisor == 0 {
                divisor = 1;
            }
            let factor = fade as Real / divisor as Real;
            self.cur_fade_value = self.max_fade + factor * (self.min_fade - self.max_fade);
            return;
        }
        self.fade = ScriptFade::None;
    }

    // Scripts and groups.

    /// The script at `r`.
    pub fn script(&self, r: ScriptRef) -> Option<&Script> {
        let list = self.sides.get(r.side)?.as_ref()?;
        match r.group {
            Some(g) => list.groups.get(g)?.scripts.get(r.index),
            None => list.scripts.get(r.index),
        }
    }

    pub(crate) fn script_mut(&mut self, r: ScriptRef) -> Option<&mut Script> {
        let list = self.sides.get_mut(r.side)?.as_mut()?;
        match r.group {
            Some(g) => list.groups.get_mut(g)?.scripts.get_mut(r.index),
            None => list.scripts.get_mut(r.index),
        }
    }

    /// `findScript`: the first of the name, side by side.
    pub fn find_script(&self, name: &str) -> Option<ScriptRef> {
        for (side, list) in self.sides.iter().enumerate() {
            let Some(list) = list else { continue };
            if let Some(index) = list.scripts.iter().position(|s| s.script_name == name) {
                return Some(ScriptRef {
                    side,
                    group: None,
                    index,
                });
            }
            for (g, group) in list.groups.iter().enumerate() {
                if let Some(index) = group.scripts.iter().position(|s| s.script_name == name) {
                    return Some(ScriptRef {
                        side,
                        group: Some(g),
                        index,
                    });
                }
            }
        }
        None
    }

    /// `findGroup`: side and group number.
    pub fn find_group(&self, name: &str) -> Option<(usize, usize)> {
        for (side, list) in self.sides.iter().enumerate() {
            let Some(list) = list else { continue };
            if let Some(g) = list.groups.iter().position(|g| g.group_name == name) {
                return Some((side, g));
            }
        }
        None
    }

    /// `executeScripts` over a side's own scripts or one group's.
    fn execute_list(&mut self, world: &mut dyn ScriptWorld, side: usize, group: Option<usize>) {
        let mut index = 0;
        loop {
            let r = ScriptRef { side, group, index };
            let Some(script) = self.script(r) else { return };
            if !on(script.is_subroutine) {
                self.execute_script(world, r);
            }
            index += 1;
        }
    }

    /// `executeScript`.
    pub fn execute_script(&mut self, world: &mut dyn ScriptWorld, r: ScriptRef) {
        let current_player = self.current_player;
        let global = self.difficulty;
        let frame = world.frame();
        let Some(script) = self.script_mut(r) else {
            return;
        };
        script.cur_time = 0.0;
        if !on(script.is_active) {
            return;
        }
        let difficulty = current_player
            .and_then(|p| world.player_difficulty(p))
            .unwrap_or(global);
        let allowed = match difficulty {
            GameDifficulty::Easy => on(script.easy),
            GameDifficulty::Normal => on(script.normal),
            GameDifficulty::Hard => on(script.hard),
        };
        if !allowed || frame < script.frame_to_evaluate_at {
            return;
        }
        let delay = script.delay_evaluation_seconds;
        if delay > 0 {
            script.frame_to_evaluate_at =
                frame.wrapping_add(delay.wrapping_mul(LOGICFRAMES_PER_SECOND) as UnsignedInt);
        }
        let prototype = if script.condition_team_name.is_empty() {
            None
        } else {
            world.team_prototype(&script.condition_team_name)
        };
        let saved = self.condition_team;
        match prototype {
            Some(p) if !p.instances.is_empty() => {
                // The reference does not deactivate a one shot script that ran
                // its false actions here, and keeps going through the teams.
                for team in p.instances {
                    self.condition_team = Some(team);
                    if self.evaluate_conditions(world, r, None, None) {
                        self.execute_actions_of(world, r, false);
                        self.deactivate_if_one_shot(r);
                    } else {
                        self.execute_actions_of(world, r, true);
                    }
                }
            }
            _ => {
                self.condition_team = None;
                if self.evaluate_conditions(world, r, None, None) {
                    self.execute_actions_of(world, r, false);
                    self.deactivate_if_one_shot(r);
                } else if self.script(r).is_some_and(|s| !s.actions_false.is_empty()) {
                    self.execute_actions_of(world, r, true);
                    self.deactivate_if_one_shot(r);
                }
            }
        }
        self.condition_team = saved;
    }

    fn deactivate_if_one_shot(&mut self, r: ScriptRef) {
        if let Some(script) = self.script_mut(r) {
            if on(script.is_one_shot) {
                script.is_active = 0;
            }
        }
    }

    /// `evaluateConditions`: any of the or terms, each an and of conditions.
    pub fn evaluate_conditions(
        &mut self,
        world: &mut dyn ScriptWorld,
        r: ScriptRef,
        this_team: Option<ScriptTeamId>,
        player: Option<ScriptPlayer>,
    ) -> bool {
        let saved_team = self.calling_team;
        let saved_player = self.current_player;
        self.calling_team = this_team;
        let player = match this_team {
            Some(t) => world.team_controlling_player(t),
            None => player,
        };
        self.current_player = player.or(self.current_player);

        let mut result = false;
        let ors = self.script(r).map_or(0, |s| s.or_conditions.len());
        for o in 0..ors {
            let ands = self
                .script(r)
                .map_or(0, |s| s.or_conditions[o].conditions.len());
            if ands == 0 {
                continue;
            }
            let mut term = true;
            for a in 0..ands {
                let Some(mut condition) = self
                    .script(r)
                    .map(|s| s.or_conditions[o].conditions[a].clone())
                else {
                    break;
                };
                let value = self.evaluate_condition(world, &mut condition);
                if let Some(script) = self.script_mut(r) {
                    script.or_conditions[o].conditions[a] = condition;
                }
                if !value {
                    term = false;
                    break;
                }
            }
            if term {
                result = true;
                break;
            }
        }

        self.calling_team = saved_team;
        self.current_player = saved_player;
        result
    }

    /// `evaluateCondition`.
    pub fn evaluate_condition(
        &mut self,
        world: &mut dyn ScriptWorld,
        condition: &mut Condition,
    ) -> bool {
        match condition.condition_type {
            ct::CONDITION_FALSE => false,
            ct::CONDITION_TRUE => true,
            ct::COUNTER => self.evaluate_counter(condition),
            ct::FLAG => self.evaluate_flag(condition),
            ct::TIMER_EXPIRED => self.evaluate_timer(condition),
            _ => match self.evaluate_world_condition(world, condition) {
                Some(value) => value,
                None => {
                    *self
                        .unported_conditions
                        .entry(condition.condition_type)
                        .or_insert(0) += 1;
                    world.unported_condition(condition)
                }
            },
        }
    }

    // Actions.

    fn execute_actions_of(&mut self, world: &mut dyn ScriptWorld, r: ScriptRef, otherwise: bool) {
        let mut i = 0;
        while self.execute_listed_action(world, r, otherwise, i) {
            i += 1;
        }
    }

    /// Executes one action of a script, keeping its parameter caches.
    fn execute_listed_action(
        &mut self,
        world: &mut dyn ScriptWorld,
        r: ScriptRef,
        otherwise: bool,
        i: usize,
    ) -> bool {
        let list = |s: &Script| {
            let l = if otherwise {
                &s.actions_false
            } else {
                &s.actions
            };
            l.get(i).cloned()
        };
        let Some(mut action) = self.script(r).and_then(list) else {
            return false;
        };
        self.execute_action(world, &mut action);
        if let Some(script) = self.script_mut(r) {
            let l = if otherwise {
                &mut script.actions_false
            } else {
                &mut script.actions
            };
            if let Some(slot) = l.get_mut(i) {
                *slot = action;
            }
        }
        true
    }

    /// `friend_executeAction` of a script's actions for a team.
    pub fn execute_actions_for_team(
        &mut self,
        world: &mut dyn ScriptWorld,
        r: ScriptRef,
        team: Option<ScriptTeamId>,
    ) {
        let saved_team = self.calling_team;
        let saved_player = self.current_player;
        self.calling_team = team;
        self.current_player = team.and_then(|t| world.team_controlling_player(t));
        self.execute_actions_of(world, r, false);
        self.calling_team = saved_team;
        self.current_player = saved_player;
    }

    /// One turn of the `executeActions` loop.
    pub fn execute_action(&mut self, world: &mut dyn ScriptWorld, action: &mut ScriptAction) {
        match action.action_type {
            at::SET_COUNTER => self.set_counter(action),
            at::SET_TREE_SWAY => self.set_sway(action),
            at::INCREMENT_COUNTER => self.change_counter(action, true),
            at::DECREMENT_COUNTER => self.change_counter(action, false),
            at::SET_FLAG => self.set_flag(action),
            at::STOP_TIMER => self.pause_timer(action),
            at::RESTART_TIMER => self.restart_timer(action),
            at::SET_TIMER => self.set_timer(world, action, false, false),
            at::SET_MILLISECOND_TIMER => self.set_timer(world, action, true, false),
            at::SET_RANDOM_TIMER => self.set_timer(world, action, false, true),
            at::SET_RANDOM_MSEC_TIMER => self.set_timer(world, action, true, true),
            at::ADD_TO_MSEC_TIMER => self.adjust_timer(action, true),
            at::SUB_FROM_MSEC_TIMER => self.adjust_timer(action, false),
            at::ENABLE_SCRIPT => self.enable_script(&parm(&action.parms, 0).string_val),
            at::DISABLE_SCRIPT => self.disable_script(&parm(&action.parms, 0).string_val),
            at::CALL_SUBROUTINE => {
                let name = parm(&action.parms, 0).string_val;
                self.call_subroutine(world, &name);
            }
            at::CAMERA_FADE_ADD => self.set_fade(action, ScriptFade::Add),
            at::CAMERA_FADE_SUBTRACT => self.set_fade(action, ScriptFade::Subtract),
            at::CAMERA_FADE_SATURATE => self.set_fade(action, ScriptFade::Saturate),
            at::CAMERA_FADE_MULTIPLY => self.set_fade(action, ScriptFade::Multiply),
            at::SET_ATTACK_PRIORITY_THING => self.set_priority_thing(world, action),
            at::SET_ATTACK_PRIORITY_KIND_OF => self.set_priority_kind(world, action),
            at::SET_DEFAULT_ATTACK_PRIORITY => self.set_priority_default(action),
            at::NO_OP => {}
            _ => {
                if !self.execute_world_action(world, action) {
                    *self.unported_actions.entry(action.action_type).or_insert(0) += 1;
                    world.unported_action(action);
                }
            }
        }
    }

    fn enable_script(&mut self, name: &str) {
        self.set_group_active(name, true);
        self.set_script_active(name, true);
    }

    fn disable_script(&mut self, name: &str) {
        self.set_script_active(name, false);
        self.set_group_active(name, false);
    }

    fn set_group_active(&mut self, name: &str, active: bool) {
        if let Some((side, g)) = self.find_group(name) {
            if let Some(list) = self.sides[side].as_mut() {
                list.groups[g].is_group_active = u8::from(active);
            }
        }
    }

    fn set_script_active(&mut self, name: &str, active: bool) {
        if let Some(r) = self.find_script(name) {
            if let Some(script) = self.script_mut(r) {
                script.is_active = u8::from(active);
            }
        }
    }

    /// `callSubroutine`: a subroutine group of the name, else a subroutine script.
    pub fn call_subroutine(&mut self, world: &mut dyn ScriptWorld, name: &str) {
        if let Some((side, g)) = self.find_group(name) {
            let Some(group) = self.sides[side].as_ref().map(|l| &l.groups[g]) else {
                return;
            };
            if on(group.is_group_subroutine) && on(group.is_group_active) {
                self.execute_list(world, side, Some(g));
            }
        } else if let Some(r) = self.find_script(name) {
            if self.script(r).is_some_and(|s| on(s.is_subroutine)) {
                self.execute_script(world, r);
            }
        }
    }

    /// `runScript`, which team and AI behaviour call by name. As there, the
    /// condition team is left cleared.
    pub fn run_script(
        &mut self,
        world: &mut dyn ScriptWorld,
        name: &str,
        this_team: Option<ScriptTeamId>,
    ) {
        if name.is_empty() || name == NONE_STRING {
            return;
        }
        let saved_player = self.current_player;
        let saved_calling = self.calling_team;
        self.calling_team = this_team;
        self.condition_team = None;
        self.current_player = this_team.and_then(|t| world.team_controlling_player(t));
        self.call_subroutine(world, name);
        self.current_player = saved_player;
        self.calling_team = saved_calling;
    }

    /// `runObjectScript`.
    pub fn run_object_script(
        &mut self,
        world: &mut dyn ScriptWorld,
        name: &str,
        this_object: Option<ObjectId>,
    ) {
        if name.is_empty() || name == NONE_STRING {
            return;
        }
        let saved = self.calling_object;
        self.calling_object = this_object;
        self.call_subroutine(world, name);
        self.calling_object = saved;
    }

    // Counters, flags and timers.

    /// `allocateCounter`; 0 when the table is full.
    pub fn allocate_counter(&mut self, name: &str) -> Int {
        let n = self.num_counters as usize;
        if let Some(i) = (1..n).find(|&i| self.counters[i].name == name) {
            return i as Int;
        }
        if n < MAX_COUNTERS {
            self.counters[n].name = name.to_string();
            self.num_counters += 1;
            return n as Int;
        }
        0
    }

    /// `getCounter`.
    pub fn counter(&self, name: &str) -> Option<&Counter> {
        (1..self.num_counters as usize)
            .map(|i| &self.counters[i])
            .find(|c| c.name == name)
    }

    /// `allocateFlag`; 0 when the table is full.
    pub fn allocate_flag(&mut self, name: &str) -> Int {
        let n = self.num_flags as usize;
        if let Some(i) = (1..n).find(|&i| self.flags[i].name == name) {
            return i as Int;
        }
        if n < MAX_FLAGS {
            self.flags[n].name = name.to_string();
            self.num_flags += 1;
            return n as Int;
        }
        0
    }

    /// The counter a parameter names, looked up once and kept in the parameter.
    fn counter_index(&mut self, parms: &mut [Parameter], i: usize) -> usize {
        let Some(p) = parms.get_mut(i) else { return 0 };
        if p.int_val == 0 {
            p.int_val = self.allocate_counter(&p.string_val);
        }
        (p.int_val as usize).min(MAX_COUNTERS - 1)
    }

    fn flag_index(&mut self, parms: &mut [Parameter], i: usize) -> usize {
        let Some(p) = parms.get_mut(i) else { return 0 };
        if p.int_val == 0 {
            p.int_val = self.allocate_flag(&p.string_val);
        }
        (p.int_val as usize).min(MAX_FLAGS - 1)
    }

    fn evaluate_counter(&mut self, c: &mut Condition) -> bool {
        let i = self.counter_index(&mut c.parms, 0);
        let value = parm(&c.parms, 2).int_val;
        compare(parm(&c.parms, 1).int_val, self.counters[i].value, value)
    }

    fn set_counter(&mut self, a: &mut ScriptAction) {
        let i = self.counter_index(&mut a.parms, 0);
        self.counters[i].value = parm(&a.parms, 1).int_val;
    }

    fn change_counter(&mut self, a: &mut ScriptAction, add: bool) {
        let value = parm(&a.parms, 0).int_val;
        let i = self.counter_index(&mut a.parms, 1);
        let old = self.counters[i].value;
        self.counters[i].value = if add {
            old.wrapping_add(value)
        } else {
            old.wrapping_sub(value)
        };
    }

    fn evaluate_flag(&mut self, c: &mut Condition) -> bool {
        let i = self.flag_index(&mut c.parms, 0);
        let wanted = parm(&c.parms, 1).int_val != 0;
        if wanted == self.flags[i].value {
            return true;
        }
        let name = parm(&c.parms, 0).string_val;
        self.ui_interactions.contains(&name)
    }

    fn set_flag(&mut self, a: &mut ScriptAction) {
        let i = self.flag_index(&mut a.parms, 0);
        self.flags[i].value = parm(&a.parms, 1).int_val != 0;
    }

    /// Sets every flag called `name` followed by a player number back to false.
    pub fn clear_flag(&mut self, name: &str) {
        for j in 0..SCRIPT_PLAYER_COUNT {
            let full = format!("{name}{j}");
            for i in 1..self.num_flags as usize {
                if self.flags[i].name == full {
                    self.flags[i].value = false;
                }
            }
        }
    }

    /// `clearTeamFlags`.
    pub fn clear_team_flags(&mut self) {
        for name in [
            "USA Team is Building",
            "USA Air Team Is Building",
            "USA Inf Team Is Building",
            "China Team is Building",
            "China Air Team Is Building",
            "China Inf Team Is Building",
            "GLA Team is Building",
            "GLA Inf Team is Building",
        ] {
            self.clear_flag(name);
        }
    }

    fn evaluate_timer(&mut self, c: &mut Condition) -> bool {
        let i = self.counter_index(&mut c.parms, 0);
        self.counters[i].is_countdown_timer && self.counters[i].value < 1
    }

    fn set_timer(
        &mut self,
        world: &mut dyn ScriptWorld,
        a: &mut ScriptAction,
        milliseconds: bool,
        random: bool,
    ) {
        let i = self.counter_index(&mut a.parms, 0);
        if milliseconds {
            let mut value = parm(&a.parms, 1).real_val;
            if random {
                // The integer generator, so both bounds lose their fraction.
                let high = parm(&a.parms, 2).real_val;
                value = world.logic_random(value as Int, high as Int) as Real;
            }
            self.counters[i].value =
                real_to_int_ceil(convert_duration_from_msecs_to_frames(value * 1000.0));
        } else {
            let mut value = parm(&a.parms, 1).int_val;
            if random {
                value = world.logic_random(value, parm(&a.parms, 2).int_val);
            }
            self.counters[i].value = value;
        }
        self.counters[i].is_countdown_timer = true;
    }

    fn pause_timer(&mut self, a: &mut ScriptAction) {
        let i = self.counter_index(&mut a.parms, 0);
        self.counters[i].is_countdown_timer = false;
    }

    fn restart_timer(&mut self, a: &mut ScriptAction) {
        let i = self.counter_index(&mut a.parms, 0);
        if self.counters[i].value > 0 {
            self.counters[i].is_countdown_timer = true;
        }
    }

    /// `adjustTimer` in seconds; the engine has no frame count form of it.
    fn adjust_timer(&mut self, a: &mut ScriptAction, add: bool) {
        let i = self.counter_index(&mut a.parms, 1);
        let mut value = parm(&a.parms, 0).real_val;
        if !add {
            value = -value;
        }
        let frames = real_to_int_ceil(convert_duration_from_msecs_to_frames(value * 1000.0));
        self.counters[i].value = self.counters[i].value.wrapping_add(frames);
    }

    fn set_sway(&mut self, a: &mut ScriptAction) {
        let b = &mut self.breeze;
        b.version = b.version.wrapping_add(1);
        b.direction = parm(&a.parms, 0).real_val;
        b.direction_x = gm_sinf(b.direction);
        b.direction_y = gm_cosf(b.direction);
        b.intensity = parm(&a.parms, 1).real_val;
        b.lean = parm(&a.parms, 2).real_val;
        b.period = parm(&a.parms, 3).int_val as i16;
        if b.period < 1 {
            b.period = 1;
        }
        b.randomness = parm(&a.parms, 4).real_val;
    }

    fn set_fade(&mut self, a: &mut ScriptAction, fade: ScriptFade) {
        self.fade = fade;
        self.cur_fade_frame = 0;
        self.min_fade = parm(&a.parms, 0).real_val;
        self.max_fade = parm(&a.parms, 1).real_val;
        self.fade_frames_increase = parm(&a.parms, 2).int_val;
        self.fade_frames_hold = parm(&a.parms, 3).int_val;
        self.fade_frames_decrease = parm(&a.parms, 4).int_val;
        self.cur_fade_value = self.min_fade;
        if self.fade_frames_increase == 0 {
            self.update_fades();
        }
    }

    // Attack priority sets.

    /// `findAttackInfo` with `addIfNotFound`.
    fn attack_info_mut(&mut self, name: &str) -> Option<&mut AttackPriorityInfo> {
        let n = self.num_attack_info as usize;
        if let Some(i) = (1..n).find(|&i| self.attack_priorities[i].name == name) {
            return Some(&mut self.attack_priorities[i]);
        }
        if n < MAX_ATTACK_PRIORITIES {
            self.attack_priorities[n].name = name.to_string();
            self.num_attack_info += 1;
            return Some(&mut self.attack_priorities[n]);
        }
        None
    }

    /// `getAttackInfo`: the set of the name, else the default set.
    pub fn attack_info(&self, name: &str) -> &AttackPriorityInfo {
        let n = self.num_attack_info as usize;
        let i = (1..n)
            .find(|&i| self.attack_priorities[i].name == name)
            .unwrap_or(0);
        &self.attack_priorities[i]
    }

    fn set_priority_thing(&mut self, world: &mut dyn ScriptWorld, a: &mut ScriptAction) {
        let set = parm(&a.parms, 0).string_val;
        let argument = parm(&a.parms, 1).string_val;
        let priority = parm(&a.parms, 2).int_val;
        let names = match self.object_types(&argument) {
            Some(types) => (0..types.get_list_size())
                .map(|i| types.get_nth_in_list(i).to_string())
                .collect(),
            None => vec![argument],
        };
        for name in names {
            let Some(template) = world.thing_template_index(&name) else {
                return;
            };
            let Some(info) = self.attack_info_mut(&set) else {
                return;
            };
            info.priorities.insert(template, priority);
        }
    }

    fn set_priority_kind(&mut self, world: &mut dyn ScriptWorld, a: &mut ScriptAction) {
        let set = parm(&a.parms, 0).string_val;
        let kind = parm(&a.parms, 1).int_val;
        let priority = parm(&a.parms, 2).int_val;
        let Some(info) = self.attack_info_mut(&set) else {
            return;
        };
        for template in world.thing_templates_of_kind(kind) {
            info.priorities.insert(template, priority);
        }
    }

    fn set_priority_default(&mut self, a: &mut ScriptAction) {
        let set = parm(&a.parms, 0).string_val;
        let priority = parm(&a.parms, 1).int_val;
        if let Some(info) = self.attack_info_mut(&set) {
            info.default_priority = priority;
        }
    }

    // Object type lists and object counts.

    /// `getObjectTypes`.
    pub fn object_types(&self, list: &str) -> Option<&ObjectTypes> {
        self.object_type_lists
            .iter()
            .find(|t| t.get_list_name() == list)
    }

    /// `doObjectTypeListMaintenance`.
    pub fn maintain_object_type_list(&mut self, list: &str, object_type: &str, add: bool) {
        let i = match self
            .object_type_lists
            .iter()
            .position(|t| t.get_list_name() == list)
        {
            Some(i) => i,
            None => {
                self.object_type_lists.push(ObjectTypes::with_name(list));
                self.object_type_lists.len() - 1
            }
        };
        let types = &mut self.object_type_lists[i];
        if add {
            types.add_object_type(object_type);
        } else {
            types.remove_object_type(object_type);
        }
        if types.get_list_size() == 0 {
            self.object_type_lists.remove(i);
        }
    }

    /// `getObjectCount`; 0 for a player that is out of the game.
    pub fn object_count(&self, world: &dyn ScriptWorld, player: ScriptPlayer, name: &str) -> Int {
        if !world.player_is_active(player) {
            return 0;
        }
        self.object_counts[player as usize]
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    pub fn set_object_count(&mut self, player: ScriptPlayer, name: &str, count: Int) {
        self.object_counts[player as usize].insert(name.to_string(), count);
    }

    // Names.

    /// `getPlayerFromAsciiString`.
    pub fn player_from_name(&self, world: &dyn ScriptWorld, name: &str) -> Option<ScriptPlayer> {
        if name == LOCAL_PLAYER || (name == THE_PLAYER && world.is_challenge_campaign()) {
            return Some(world.local_player());
        }
        if name == THIS_PLAYER {
            return self.current_player;
        }
        if name == THIS_PLAYER_ENEMY {
            return self.skirmish_enemy(world);
        }
        world.find_player(name)
    }

    /// `getSkirmishEnemyPlayer`: the current enemy, else the first human.
    pub fn skirmish_enemy(&self, world: &dyn ScriptWorld) -> Option<ScriptPlayer> {
        let current = self.current_player?;
        if let Some(enemy) = world.player_current_enemy(current) {
            return Some(enemy);
        }
        let challenge = world.is_challenge_campaign();
        (0..world.player_count()).find(|&p| {
            world.player_is_human(p) && !(challenge && world.player_name(p) == THE_PLAYER)
        })
    }

    /// `getTeamNamed`.
    pub fn team_named(&self, world: &dyn ScriptWorld, name: &str) -> Option<ScriptTeamId> {
        if name == TEAM_THE_PLAYER && world.is_challenge_campaign() {
            return world.player_default_team(world.local_player());
        }
        if name == THIS_TEAM {
            return self.calling_team.or(self.condition_team);
        }
        for team in [self.calling_team, self.condition_team]
            .into_iter()
            .flatten()
        {
            if world.team_name(team) == name {
                return Some(team);
            }
        }
        let prototype = world.team_prototype(name)?;
        let first = prototype.instances.first().copied()?;
        if prototype.singleton && !world.team_is_active(first) {
            return None;
        }
        Some(first)
    }

    /// `getUnitNamed`.
    pub fn unit_named(&self, name: &str) -> Option<ObjectId> {
        if name == THIS_OBJECT {
            return self.calling_object.or(self.condition_object);
        }
        self.named_objects
            .iter()
            .find(|(n, _)| n == name)
            .and_then(|(_, id)| *id)
    }

    /// `didUnitExist`: the name is known and its object is gone.
    pub fn did_unit_exist(&self, name: &str) -> bool {
        self.named_objects
            .iter()
            .find(|(n, _)| n == name)
            .is_some_and(|(_, id)| id.is_none())
    }

    /// `createNamedCache`.
    pub fn create_named_cache(&mut self, world: &dyn ScriptWorld) {
        self.named_objects = world
            .named_objects()
            .into_iter()
            .filter(|(name, _)| !name.is_empty())
            .map(|(name, id)| (name, Some(id)))
            .collect();
    }

    /// `addObjectToCache`.
    pub fn add_object_to_cache(&mut self, name: &str, id: ObjectId) {
        if name.is_empty() {
            return;
        }
        for entry in self.named_objects.iter_mut() {
            if entry.0 == name {
                if entry.1.is_none() {
                    entry.1 = Some(id);
                }
                return;
            }
            if entry.1 == Some(id) {
                entry.0 = name.to_string();
                return;
            }
        }
        self.named_objects.push((name.to_string(), Some(id)));
    }

    /// `removeObjectFromCache`: the name stays, without an object.
    pub fn remove_object_from_cache(&mut self, id: ObjectId) {
        if let Some(entry) = self.named_objects.iter_mut().find(|e| e.1 == Some(id)) {
            entry.1 = None;
        }
    }

    /// `transferObjectName`.
    pub fn transfer_object_name(&mut self, world: &mut dyn ScriptWorld, name: &str, to: ObjectId) {
        if name.is_empty() {
            return;
        }
        if !world.object_name(to).is_empty() {
            self.remove_object_from_cache(to);
        }
        world.set_object_name(to, name);
        if let Some(entry) = self.named_objects.iter_mut().find(|e| e.0 == name) {
            if let Some(old) = entry.1 {
                world.copy_indicator_color(old, to);
            }
            entry.1 = Some(to);
        }
    }

    /// `notifyOfObjectDestruction`.
    pub fn notify_of_object_destruction(&mut self, name: &str, id: ObjectId) {
        if !name.is_empty() {
            self.remove_object_from_cache(id);
        }
        if self.condition_object == Some(id) {
            self.condition_object = None;
        }
        if self.calling_object == Some(id) {
            self.calling_object = None;
        }
    }

    /// `notifyOfObjectCreationOrDestruction`.
    pub fn notify_of_object_creation_or_destruction(&mut self, frame: UnsignedInt) {
        self.frame_object_count_changed = frame;
    }

    /// `notifyOfTeamDestruction`.
    pub fn notify_of_team_destruction(&mut self, team: ScriptTeamId) {
        self.sequential
            .retain(|chain| chain.first().map_or(false, |s| s.team != Some(team)));
        if self.calling_team == Some(team) {
            self.calling_team = None;
        }
        if self.condition_team == Some(team) {
            self.condition_team = None;
        }
    }

    // Events reported by the game and tested by conditions.

    pub fn notify_of_completed_video(&mut self, name: &str) {
        self.completed_video.push(name.to_string());
    }

    pub fn notify_of_triggered_special_power(
        &mut self,
        player: ScriptPlayer,
        name: &str,
        source: ObjectId,
    ) {
        self.triggered_special_powers[player as usize].push((name.to_string(), source));
    }

    pub fn notify_of_midway_special_power(
        &mut self,
        player: ScriptPlayer,
        name: &str,
        source: ObjectId,
    ) {
        self.midway_special_powers[player as usize].push((name.to_string(), source));
    }

    pub fn notify_of_completed_special_power(
        &mut self,
        player: ScriptPlayer,
        name: &str,
        source: ObjectId,
    ) {
        self.finished_special_powers[player as usize].push((name.to_string(), source));
    }

    pub fn notify_of_completed_upgrade(
        &mut self,
        player: ScriptPlayer,
        name: &str,
        source: ObjectId,
    ) {
        self.completed_upgrades[player as usize].push((name.to_string(), source));
    }

    pub fn notify_of_acquired_science(&mut self, player: ScriptPlayer, science: ScienceType) {
        self.acquired_sciences[player as usize].push(science);
    }

    /// `signalUIInteract`: a flag condition of this name is true this frame.
    pub fn signal_ui_interact(&mut self, hook: &str) {
        self.ui_interactions.insert(0, hook.to_string());
    }

    pub(crate) fn video_complete(&mut self, name: &str) -> bool {
        match self.completed_video.iter().position(|v| v == name) {
            Some(i) => {
                self.completed_video.remove(i);
                true
            }
            None => false,
        }
    }

    /// `isSpeechComplete` and `isAudioComplete`: the first question starts the
    /// clock for the length of the event.
    pub(crate) fn sound_complete(
        &mut self,
        world: &dyn ScriptWorld,
        speech: bool,
        name: &str,
    ) -> bool {
        let list = if speech {
            &mut self.testing_speech
        } else {
            &mut self.testing_audio
        };
        let i = match list.iter().position(|(n, _)| n == name) {
            Some(i) => i,
            None => {
                let frames =
                    (world.audio_length_ms(name) / MSEC_PER_LOGICFRAME_REAL) as UnsignedInt;
                list.insert(0, (name.to_string(), frames.wrapping_add(world.frame())));
                0
            }
        };
        if world.frame() >= list[i].1 {
            list.remove(i);
            return true;
        }
        false
    }

    // Reveals, topple directions, the end of the game and frozen time.

    /// `createNamedMapReveal`: a name already defined is kept.
    pub fn create_named_map_reveal(&mut self, reveal: NamedReveal) {
        if self
            .named_reveals
            .iter()
            .all(|r| r.reveal_name != reveal.reveal_name)
        {
            self.named_reveals.push(reveal);
        }
    }

    /// `doNamedMapReveal`, or `undoNamedMapReveal` when `undo`.
    pub fn apply_named_map_reveal(&mut self, world: &mut dyn ScriptWorld, name: &str, undo: bool) {
        let Some(reveal) = self.named_reveals.iter().find(|r| r.reveal_name == name) else {
            return;
        };
        let Some(at) = world.waypoint_location(&reveal.waypoint_name) else {
            return;
        };
        let Some(player) = self.player_from_name(world, &reveal.player_name) else {
            return;
        };
        if undo {
            world.undo_shroud_reveal(at, reveal.radius, player);
        } else {
            world.do_shroud_reveal(at, reveal.radius, player);
        }
    }

    pub fn remove_named_map_reveal(&mut self, name: &str) {
        if let Some(i) = self
            .named_reveals
            .iter()
            .position(|r| r.reveal_name == name)
        {
            self.named_reveals.remove(i);
        }
    }

    /// `setToppleDirection`.
    pub fn set_topple_direction(&mut self, name: &str, direction: Coord3D) {
        if name.is_empty() {
            return;
        }
        match self.topple_directions.iter_mut().find(|e| e.0 == name) {
            Some(entry) => entry.1 = direction,
            None => self
                .topple_directions
                .insert(0, (name.to_string(), direction)),
        }
    }

    /// `adjustToppleDirection` of the object of this name.
    pub fn adjust_topple_direction(&self, name: &str, direction: &mut Coord3D) {
        if name.is_empty() {
            return;
        }
        if let Some(entry) = self.topple_directions.iter().find(|e| e.0 == name) {
            *direction = entry.1;
            direction.normalize();
        }
    }

    pub fn start_end_game_timer(&mut self) {
        self.end_game_timer = FRAMES_TO_SHOW_WIN_LOSE_MESSAGE;
    }

    pub fn start_quick_end_game_timer(&mut self) {
        self.end_game_timer = 1;
    }

    pub fn start_close_window_timer(&mut self) {
        self.close_window_timer = FRAMES_TO_SHOW_WIN_LOSE_MESSAGE;
    }

    /// `isTimeFrozenScript`.
    pub fn is_time_frozen(&self) -> bool {
        self.freeze_by_script
    }

    // Sequential scripts.

    /// `appendSequentialScript`: queued behind a chain of the same object or
    /// team, from the first instruction.
    pub fn append_sequential_script(&mut self, mut script: SequentialScript) {
        script.current_instruction = -1;
        let found = self.sequential.iter_mut().find(|chain| {
            chain.first().is_some_and(|s| {
                (script.object_id != ObjectId::INVALID && script.object_id == s.object_id)
                    || (script.team.is_some() && script.team == s.team)
            })
        });
        match found {
            Some(chain) => chain.push(script),
            None => self.sequential.push(vec![script]),
        }
    }

    /// `removeAllSequentialScripts(Object *)`.
    pub fn remove_sequential_scripts_of_object(&mut self, id: ObjectId) {
        self.sequential
            .retain(|chain| chain.first().map_or(false, |s| s.object_id != id));
    }

    /// `setSequentialTimer`, on the first chain of the object.
    pub fn set_sequential_timer_object(&mut self, id: ObjectId, frames: Int) {
        if let Some(s) = self
            .sequential
            .iter_mut()
            .filter_map(|c| c.first_mut())
            .find(|s| s.object_id == id)
        {
            s.frames_to_wait = frames;
        }
    }

    /// `setSequentialTimer`, on the first chain of the team.
    pub fn set_sequential_timer_team(&mut self, team: ScriptTeamId, frames: Int) {
        if let Some(s) = self
            .sequential
            .iter_mut()
            .filter_map(|c| c.first_mut())
            .find(|s| s.team == Some(team))
        {
            s.frames_to_wait = frames;
        }
    }

    /// `cleanupSequentialScript`: drops the head, or the whole chain.
    fn cleanup_sequential(&mut self, i: usize, whole_chain: bool) {
        let Some(chain) = self.sequential.get_mut(i) else {
            return;
        };
        if !whole_chain && chain.len() > 1 {
            chain.remove(0);
        } else {
            self.sequential.remove(i);
        }
    }

    /// `evaluateAndProgressAllSequentialScripts`.
    fn evaluate_sequential_scripts(&mut self, world: &mut dyn ScriptWorld) {
        let mut i = 0;
        let mut previous = usize::MAX;
        let mut spins = 0;
        while i < self.sequential.len() {
            if i == previous {
                spins += 1;
            } else {
                spins = 0;
            }
            if spins > MAX_SPIN_COUNT {
                i += 1;
                continue;
            }
            previous = i;
            if !self.progress_sequential(world, i) {
                i += 1;
            }
        }
        self.current_player = None;
    }

    /// One turn of the loop for chain `i`; true if it is to be looked at again
    /// without moving on.
    fn progress_sequential(&mut self, world: &mut dyn ScriptWorld, i: usize) -> bool {
        let Some(mut head) = self.sequential[i].first().cloned() else {
            self.sequential.remove(i);
            return true;
        };
        let team = head.team;
        let object = Some(head.object_id).filter(|&id| world.object_exists(id));
        if object.is_none() && team.is_none() {
            self.cleanup_sequential(i, false);
            return true;
        }
        self.current_player = match (object, team) {
            (Some(o), _) => world.object_controlling_player(o),
            (None, Some(t)) => world.team_controlling_player(t),
            (None, None) => None,
        };
        if self
            .current_player
            .is_some_and(|p| !world.player_is_skirmish_ai(p))
        {
            self.current_player = None;
        }

        let ai_idle = object.and_then(|o| world.object_ai_is_idle(o));
        let group_idle = team.map(|t| world.team_group_is_idle(t));
        if ai_idle.is_none() && group_idle.is_none() {
            return false;
        }
        let idle = ai_idle.is_some_and(bool::from) || group_idle.unwrap_or(false);
        if !(head.frames_to_wait == 0 || (head.frames_to_wait < 0 && idle)) {
            if head.frames_to_wait > 0 {
                self.sequential[i][0].frames_to_wait = head.frames_to_wait - 1;
            }
            return false;
        }

        if head.dont_advance_instruction {
            head.dont_advance_instruction = false;
        } else {
            head.current_instruction = head.current_instruction.wrapping_add(1);
        }
        let script = head.script;
        let instruction = head.current_instruction;
        let action_type = script
            .and_then(|r| self.script(r))
            .and_then(|s| s.actions.get(usize::try_from(instruction).ok()?))
            .map(|a| (a.action_type, a.parms.clone()));
        let (Some(r), Some((action_type, parms))) = (script, action_type) else {
            // Past the last instruction: queue it again if it loops.
            if head.times_to_loop != 0 {
                if head.times_to_loop != -1 {
                    head.times_to_loop -= 1;
                }
                head.frames_to_wait = -1;
                self.sequential[i][0] = head.clone();
                self.append_sequential_script(head);
            } else {
                self.sequential[i][0] = head;
            }
            self.cleanup_sequential(i, false);
            return true;
        };

        self.condition_team = team;
        self.condition_object = object;
        head.frames_to_wait = -1;
        self.sequential[i][0] = head;
        let wait = match action_type {
            at::SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_ALL
            | at::SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_PARTIAL => {
                let all = action_type == at::SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_ALL;
                let (team_parm, button) = (parm(&parms, 1), parm(&parms, 2));
                Some(!world.command_button_ready(&team_parm, &button, all))
            }
            at::TEAM_WAIT_FOR_NOT_CONTAINED_ALL | at::TEAM_WAIT_FOR_NOT_CONTAINED_PARTIAL => {
                let all = action_type == at::TEAM_WAIT_FOR_NOT_CONTAINED_ALL;
                Some(world.team_is_contained(&parm(&parms, 0), all))
            }
            _ => None,
        };
        match wait {
            Some(wait) => {
                if wait {
                    if let Some(s) = self.sequential.get_mut(i).and_then(|c| c.first_mut()) {
                        s.dont_advance_instruction = true;
                    }
                }
            }
            None => {
                self.execute_listed_action(world, r, false, instruction as usize);
            }
        }

        // The action may have changed the chains.
        let Some(head) = self.sequential.get(i).and_then(|c| c.first()) else {
            return true;
        };
        if head.dont_advance_instruction {
            return false;
        }
        let mut again = false;
        let mut group_ai_dead = false;
        // The reference asks the object's AI again, and a team's new group.
        if object
            .and_then(|o| world.object_ai_is_idle(o))
            .is_some_and(bool::from)
        {
            again = true;
        }
        if let Some(t) = team {
            if world.team_group_is_idle(t) {
                again = true;
            }
        }
        if again {
            if object.is_some_and(|o| world.object_is_effectively_dead(o)) {
                self.cleanup_sequential(i, true);
                return true;
            }
            if let Some(t) = team {
                group_ai_dead = world.team_group_is_ai_dead(t);
            }
            if group_ai_dead {
                self.cleanup_sequential(i, true);
                return true;
            }
        }
        again
    }

    // Save and load.

    /// `xfer`. `crc` is empty in the reference.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, world: &dyn ScriptWorld) -> Result<(), XferError> {
        let current: XferVersion = if cfg!(feature = "retail-xfer-save") {
            5
        } else {
            6
        };
        let mut version = current;
        xfer.xfer_version(&mut version, current)?;
        let loading = xfer.mode() == XferMode::Load;

        let mut count = self.sequential.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if loading {
            if !self.sequential.is_empty() {
                return Err(XferError::InvalidData);
            }
            self.sequential = vec![vec![SequentialScript::default()]; count as usize];
        }
        // Only the head of each chain is saved.
        for i in 0..self.sequential.len() {
            let mut head = self.sequential[i].first().cloned().unwrap_or_default();
            self.xfer_sequential(xfer, world, &mut head)?;
            if loading {
                self.sequential[i] = vec![head];
            }
        }

        let mut size = self.num_counters as UnsignedShort;
        xfer.xfer_unsigned_short(&mut size)?;
        if size as usize > MAX_COUNTERS {
            return Err(XferError::InvalidData);
        }
        for c in self.counters.iter_mut().take(size as usize) {
            xfer.xfer_int(&mut c.value)?;
            xfer.xfer_ascii_string(&mut c.name)?;
            xfer_flag(xfer, &mut c.is_countdown_timer)?;
        }
        xfer.xfer_int(&mut self.num_counters)?;
        let mut size = self.num_flags as UnsignedShort;
        xfer.xfer_unsigned_short(&mut size)?;
        if size as usize > MAX_FLAGS {
            return Err(XferError::InvalidData);
        }
        for f in self.flags.iter_mut().take(size as usize) {
            xfer_flag(xfer, &mut f.value)?;
            xfer.xfer_ascii_string(&mut f.name)?;
        }
        xfer.xfer_int(&mut self.num_flags)?;
        let mut size = self.num_attack_info as UnsignedShort;
        xfer.xfer_unsigned_short(&mut size)?;
        if size as usize > MAX_ATTACK_PRIORITIES {
            return Err(XferError::InvalidData);
        }
        for info in self.attack_priorities.iter_mut().take(size as usize) {
            info.xfer(xfer, world)?;
        }
        xfer.xfer_int(&mut self.num_attack_info)?;
        xfer.xfer_int(&mut self.end_game_timer)?;
        xfer.xfer_int(&mut self.close_window_timer)?;

        let mut count = self.named_objects.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if loading {
            self.named_objects = vec![(String::new(), None); count as usize];
        }
        for (name, id) in self.named_objects.iter_mut() {
            xfer.xfer_ascii_string(name)?;
            let mut raw = id.unwrap_or(ObjectId::INVALID);
            xfer.xfer_object_id(&mut raw)?;
            if loading {
                if raw != ObjectId::INVALID && !world.object_exists(raw) {
                    return Err(XferError::InvalidData);
                }
                *id = Some(raw).filter(|&r| r != ObjectId::INVALID);
            }
        }

        xfer_flag(xfer, &mut self.first_update)?;
        let mut fade = self.fade as Int;
        xfer.xfer_enum_i32(&mut fade)?;
        self.fade = ScriptFade::from_i32(fade);
        xfer.xfer_real(&mut self.min_fade)?;
        xfer.xfer_real(&mut self.max_fade)?;
        xfer.xfer_real(&mut self.cur_fade_value)?;
        xfer.xfer_int(&mut self.cur_fade_frame)?;
        xfer.xfer_int(&mut self.fade_frames_increase)?;
        xfer.xfer_int(&mut self.fade_frames_hold)?;
        xfer.xfer_int(&mut self.fade_frames_decrease)?;
        xfer_strings(xfer, &mut self.completed_video)?;
        xfer_timed(xfer, &mut self.testing_speech)?;
        xfer_timed(xfer, &mut self.testing_audio)?;
        xfer_strings(xfer, &mut self.ui_interactions)?;
        for lists in [
            &mut self.triggered_special_powers,
            &mut self.midway_special_powers,
            &mut self.finished_special_powers,
            &mut self.completed_upgrades,
        ] {
            xfer_player_count(xfer)?;
            for list in lists.iter_mut() {
                xfer_events(xfer, list)?;
            }
        }
        xfer_player_count(xfer)?;
        for list in self.acquired_sciences.iter_mut() {
            xfer_sciences(xfer, world, list)?;
        }
        xfer_topples(xfer, &mut self.topple_directions)?;

        let b = &mut self.breeze;
        xfer.xfer_real(&mut b.direction)?;
        xfer.xfer_real(&mut b.direction_x)?;
        xfer.xfer_real(&mut b.direction_y)?;
        xfer.xfer_real(&mut b.intensity)?;
        xfer.xfer_real(&mut b.lean)?;
        xfer.xfer_real(&mut b.randomness)?;
        xfer.xfer_short(&mut b.period)?;
        xfer.xfer_short(&mut b.version)?;
        let mut difficulty = self.difficulty as Int;
        xfer.xfer_enum_i32(&mut difficulty)?;
        self.difficulty = match difficulty {
            0 => GameDifficulty::Easy,
            1 => GameDifficulty::Normal,
            _ => GameDifficulty::Hard,
        };
        xfer_flag(xfer, &mut self.freeze_by_script)?;

        if version >= 2 {
            let mut count = self.named_reveals.len() as UnsignedShort;
            xfer.xfer_unsigned_short(&mut count)?;
            if loading {
                if !self.named_reveals.is_empty() {
                    return Err(XferError::InvalidData);
                }
                self.named_reveals = vec![NamedReveal::default(); count as usize];
            }
            for r in self.named_reveals.iter_mut() {
                xfer.xfer_ascii_string(&mut r.reveal_name)?;
                xfer.xfer_ascii_string(&mut r.waypoint_name)?;
                xfer.xfer_real(&mut r.radius)?;
                xfer.xfer_ascii_string(&mut r.player_name)?;
            }
            let mut count = self.object_type_lists.len() as UnsignedShort;
            xfer.xfer_unsigned_short(&mut count)?;
            if loading {
                if !self.object_type_lists.is_empty() {
                    return Err(XferError::InvalidData);
                }
                self.object_type_lists = vec![ObjectTypes::new(); count as usize];
            }
            for types in self.object_type_lists.iter_mut() {
                types.xfer(xfer)?;
            }
        }
        if version >= 3 {
            xfer_flag(xfer, &mut self.objects_receive_difficulty_bonus)?;
        } else {
            self.objects_receive_difficulty_bonus = true;
        }
        if version >= 4 {
            xfer.xfer_ascii_string(&mut self.current_track_name)?;
        }
        if version >= 5 {
            xfer_flag(xfer, &mut self.choose_victim_always_uses_normal)?;
        } else {
            self.choose_victim_always_uses_normal = false;
        }
        if version >= 6 {
            xfer_flag(xfer, &mut self.letter_box_active)?;
        } else if loading {
            self.letter_box_active = false;
        }
        if loading && self.fade == ScriptFade::None {
            self.start_fade_in();
        }
        Ok(())
    }

    fn xfer_sequential(
        &self,
        xfer: &mut dyn Xfer,
        world: &dyn ScriptWorld,
        s: &mut SequentialScript,
    ) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut team = s.team.unwrap_or(0);
        xfer.xfer_unsigned_int(&mut team)?;
        if xfer.mode() == XferMode::Load {
            if team != 0 && !world.team_exists(team) {
                return Err(XferError::InvalidData);
            }
            s.team = Some(team).filter(|&t| t != 0);
        }
        xfer.xfer_object_id(&mut s.object_id)?;
        let mut name = s
            .script
            .and_then(|r| self.script(r))
            .map(|sc| sc.script_name.clone())
            .unwrap_or_default();
        xfer.xfer_ascii_string(&mut name)?;
        if xfer.mode() == XferMode::Load {
            s.script = self.find_script(&name);
        }
        xfer.xfer_int(&mut s.current_instruction)?;
        xfer.xfer_int(&mut s.times_to_loop)?;
        xfer.xfer_int(&mut s.frames_to_wait)?;
        xfer_flag(xfer, &mut s.dont_advance_instruction)
    }
}

/// The comparisons of `Parameter`: less, less or equal, equal, greater or
/// equal, greater, not equal. Anything else is false.
pub(crate) fn compare<T: PartialOrd>(op: Int, a: T, b: T) -> bool {
    match op {
        0 => a < b,
        1 => a <= b,
        2 => a == b,
        3 => a >= b,
        4 => a > b,
        5 => a != b,
        _ => false,
    }
}

/// `checkConditionsForTeamNames`.
fn check_conditions_for_team_names(world: &mut dyn ScriptWorld, script: &mut Script) {
    script.frame_to_evaluate_at = if script.delay_evaluation_seconds > 0 {
        world.logic_random(0, 2 * LOGICFRAMES_PER_SECOND) as UnsignedInt
    } else {
        0
    };
    let mut singleton = String::new();
    let mut multiple = String::new();
    for or in &script.or_conditions {
        for condition in &or.conditions {
            for p in &condition.parms {
                if p.param_type != ParameterType::Team {
                    continue;
                }
                let Some(proto) = world.team_prototype(&p.string_val) else {
                    continue;
                };
                if proto.singleton || proto.max_instances < 2 {
                    singleton = p.string_val.clone();
                } else if multiple.is_empty() {
                    multiple = p.string_val.clone();
                }
            }
        }
    }
    if !multiple.is_empty() {
        script.condition_team_name = multiple;
    } else if !singleton.is_empty() {
        script.condition_team_name = singleton;
    }
}

fn xfer_list_header(xfer: &mut dyn Xfer, held: UnsignedShort) -> Result<UnsignedShort, XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    let mut count = held;
    xfer.xfer_unsigned_short(&mut count)?;
    if xfer.mode() == XferMode::Load && held != 0 {
        return Err(XferError::InvalidData);
    }
    Ok(count)
}

fn xfer_list<T: Clone + Default>(
    xfer: &mut dyn Xfer,
    list: &mut Vec<T>,
    mut item: impl FnMut(&mut dyn Xfer, &mut T) -> Result<(), XferError>,
) -> Result<(), XferError> {
    let count = xfer_list_header(xfer, list.len() as UnsignedShort)?;
    if xfer.mode() == XferMode::Load {
        list.resize(count as usize, T::default());
    }
    for entry in list.iter_mut() {
        item(xfer, entry)?;
    }
    Ok(())
}

fn xfer_strings(xfer: &mut dyn Xfer, list: &mut Vec<String>) -> Result<(), XferError> {
    xfer_list(xfer, list, |x, s| x.xfer_ascii_string(s))
}

fn xfer_timed(xfer: &mut dyn Xfer, list: &mut Vec<(String, UnsignedInt)>) -> Result<(), XferError> {
    xfer_list(xfer, list, |x, e| {
        x.xfer_ascii_string(&mut e.0)?;
        x.xfer_unsigned_int(&mut e.1)
    })
}

fn xfer_events(xfer: &mut dyn Xfer, list: &mut Vec<(String, ObjectId)>) -> Result<(), XferError> {
    let count = xfer_list_header(xfer, list.len() as UnsignedShort)?;
    if xfer.mode() == XferMode::Load {
        list.resize(count as usize, (String::new(), ObjectId::INVALID));
    }
    for e in list.iter_mut() {
        xfer.xfer_ascii_string(&mut e.0)?;
        xfer.xfer_object_id(&mut e.1)?;
    }
    Ok(())
}

fn xfer_topples(xfer: &mut dyn Xfer, list: &mut Vec<(String, Coord3D)>) -> Result<(), XferError> {
    let count = xfer_list_header(xfer, list.len() as UnsignedShort)?;
    if xfer.mode() == XferMode::Load {
        let zero = Coord3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        list.resize(count as usize, (String::new(), zero));
    }
    for e in list.iter_mut() {
        xfer.xfer_ascii_string(&mut e.0)?;
        xfer.xfer_coord3d(&mut e.1)?;
    }
    Ok(())
}

fn xfer_player_count(xfer: &mut dyn Xfer) -> Result<(), XferError> {
    let mut size = SCRIPT_PLAYER_COUNT as UnsignedShort;
    xfer.xfer_unsigned_short(&mut size)?;
    if size as usize != SCRIPT_PLAYER_COUNT {
        return Err(XferError::InvalidData);
    }
    Ok(())
}

/// `Xfer::xferScienceVec`: names in a save file.
fn xfer_sciences(
    xfer: &mut dyn Xfer,
    world: &dyn ScriptWorld,
    list: &mut Vec<ScienceType>,
) -> Result<(), XferError> {
    let mut version: XferVersion = 1;
    xfer.xfer_version(&mut version, 1)?;
    let mut count = list.len() as UnsignedShort;
    xfer.xfer_unsigned_short(&mut count)?;
    match xfer.mode() {
        XferMode::Save => {
            for &science in list.iter() {
                let mut name = world.science_name(science);
                xfer.xfer_ascii_string(&mut name)?;
            }
        }
        XferMode::Load => {
            list.clear();
            for _ in 0..count {
                let mut name = String::new();
                xfer.xfer_ascii_string(&mut name)?;
                list.push(
                    world
                        .science_from_name(&name)
                        .ok_or(XferError::InvalidData)?,
                );
            }
        }
        _ => {
            for science in list.iter_mut() {
                xfer.xfer_int(science)?;
            }
        }
    }
    Ok(())
}

/// A script `Bool` as a `bool`.
pub(crate) fn on(value: impl Into<u8>) -> bool {
    value.into() != 0
}

fn xfer_flag(xfer: &mut dyn Xfer, flag: &mut bool) -> Result<(), XferError> {
    let mut byte = u8::from(*flag);
    xfer.xfer_bool(&mut byte)?;
    *flag = byte != 0;
    Ok(())
}
