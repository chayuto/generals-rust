//! `PKT-432`: `ScriptConditions`, the conditions that need only counts and
//! states [`ScriptWorld`] can report. Reference:
//! `GameLogic/ScriptEngine/ScriptConditions.cpp`. Golden test:
//! `tests/golden_script_engine.rs`.
//!
//! [`PORTED_CONDITIONS`] lists what is evaluated here or by the engine; any
//! other type returns `None` and goes to the host as unported. The reference's
//! quirks are kept: the player of a parameter is looked up once and cached as a
//! mask, and the object count conditions keep their answer until the object
//! count of the game changes.

use zh_common::types::{Int, ObjectId, Real, UnsignedInt};

use super::script_engine::SCRIPT_PLAYER_COUNT;
use super::script_engine::{compare, parm, ScriptEngine, ScriptPlayer, ScriptWorld};
use super::script_types::condition_type as ct;
use super::scripts::{Condition, Parameter, THIS_PLAYER_ENEMY, THIS_TEAM};

/// Every condition type evaluated by the engine or this module.
pub const PORTED_CONDITIONS: &[Int] = &[
    ct::CONDITION_FALSE,
    ct::COUNTER,
    ct::FLAG,
    ct::CONDITION_TRUE,
    ct::TIMER_EXPIRED,
    ct::PLAYER_ALL_DESTROYED,
    ct::PLAYER_ALL_BUILDFACILITIES_DESTROYED,
    ct::TEAM_DESTROYED,
    ct::TEAM_HAS_UNITS,
    ct::TEAM_STATE_IS,
    ct::TEAM_STATE_IS_NOT,
    ct::TEAM_CREATED,
    ct::TEAM_OWNED_BY_PLAYER,
    ct::NAMED_DESTROYED,
    ct::NAMED_NOT_DESTROYED,
    ct::NAMED_DYING,
    ct::NAMED_TOTALLY_DEAD,
    ct::NAMED_CREATED,
    ct::NAMED_OWNED_BY_PLAYER,
    ct::UNIT_HEALTH,
    ct::PLAYER_HAS_CREDITS,
    ct::PLAYER_HAS_POWER,
    ct::PLAYER_HAS_NO_POWER,
    ct::PLAYER_POWER_COMPARE_PERCENT,
    ct::PLAYER_EXCESS_POWER_COMPARE_VALUE,
    ct::PLAYER_HAS_N_OR_FEWER_BUILDINGS,
    ct::PLAYER_HAS_N_OR_FEWER_FACTION_BUILDINGS,
    ct::PLAYER_HAS_OBJECT_COMPARISON,
    ct::BUILT_BY_PLAYER,
    ct::PLAYER_LOST_OBJECT_TYPE,
    ct::PLAYER_ACQUIRED_SCIENCE,
    ct::PLAYER_TRIGGERED_SPECIAL_POWER,
    ct::PLAYER_TRIGGERED_SPECIAL_POWER_FROM_NAMED,
    ct::PLAYER_MIDWAY_SPECIAL_POWER,
    ct::PLAYER_MIDWAY_SPECIAL_POWER_FROM_NAMED,
    ct::PLAYER_COMPLETED_SPECIAL_POWER,
    ct::PLAYER_COMPLETED_SPECIAL_POWER_FROM_NAMED,
    ct::PLAYER_BUILT_UPGRADE,
    ct::PLAYER_BUILT_UPGRADE_FROM_NAMED,
    ct::HAS_FINISHED_VIDEO,
    ct::HAS_FINISHED_SPEECH,
    ct::HAS_FINISHED_AUDIO,
    ct::SKIRMISH_PLAYER_FACTION,
    ct::SKIRMISH_PLAYER_HAS_BEEN_ATTACKED_BY_PLAYER,
    ct::START_POSITION_IS,
    ct::MISSION_ATTEMPTS,
    ct::MUSIC_TRACK_HAS_COMPLETED,
    ct::DEFUNCT_PLAYER_SELECTED_GENERAL,
    ct::DEFUNCT_PLAYER_SELECTED_GENERAL_FROM_NAMED,
    ct::SUPPLY_SOURCE_SAFE,
    ct::SUPPLY_SOURCE_ATTACKED,
];

#[derive(Clone, Copy)]
enum PowerEvent {
    Triggered,
    Midway,
    Completed,
    Upgrade,
}

impl ScriptEngine {
    /// `playerFromParam`: the mask kept in the parameter, else the name.
    fn player_from_parm(
        &self,
        world: &dyn ScriptWorld,
        parms: &mut [Parameter],
        i: usize,
    ) -> Option<ScriptPlayer> {
        let p = parms.get_mut(i)?;
        let mask = p.int_val as UnsignedInt;
        if mask != 0 {
            return (0..SCRIPT_PLAYER_COUNT as Int).find(|&n| 1u32 << n == mask);
        }
        let player = self.player_from_name(world, &p.string_val);
        match player {
            Some(n) => {
                if p.string_val != THIS_PLAYER_ENEMY {
                    p.int_val = 1 << n;
                }
            }
            None => p.int_val = 0xFFFF_0000u32 as Int,
        }
        player
    }

    /// `objectTypesFromParam` followed by `prepForPlayerCounting`: the names of
    /// the list, or the name itself, that are templates.
    fn counted_templates(&self, world: &dyn ScriptWorld, name: &str) -> Vec<String> {
        if name.is_empty() {
            return Vec::new();
        }
        let names: Vec<String> = match self.object_types(name) {
            Some(types) => (0..types.get_list_size())
                .map(|i| types.get_nth_in_list(i).to_string())
                .collect(),
            None => vec![name.to_string()],
        };
        names
            .into_iter()
            .filter(|n| world.has_thing_template(n))
            .collect()
    }

    /// The cached answer of a count condition, while the object count of the
    /// game is unchanged.
    fn cached_count(&self, c: &Condition) -> Option<bool> {
        if c.custom_data == 0 || self.frame_object_count_changed != c.custom_frame {
            return None;
        }
        match c.custom_data {
            1 => Some(true),
            -1 => Some(false),
            _ => None,
        }
    }

    fn keep_count(&self, c: &mut Condition, answer: bool) -> bool {
        c.custom_data = if answer { 1 } else { -1 };
        c.custom_frame = self.frame_object_count_changed;
        answer
    }

    fn player_has_object_comparison(&self, world: &dyn ScriptWorld, c: &mut Condition) -> bool {
        if let Some(answer) = self.cached_count(c) {
            return answer;
        }
        let Some(player) = self.player_from_parm(world, &mut c.parms, 0) else {
            return false;
        };
        let templates = self.counted_templates(world, &parm(&c.parms, 3).string_val);
        let count = count_templates(world, player, &templates, false);
        let answer = compare(parm(&c.parms, 1).int_val, count, parm(&c.parms, 2).int_val);
        self.keep_count(c, answer)
    }

    fn built_by_player(&self, world: &dyn ScriptWorld, c: &mut Condition) -> bool {
        if let Some(answer) = self.cached_count(c) {
            return answer;
        }
        let Some(player) = self.player_from_parm(world, &mut c.parms, 1) else {
            return false;
        };
        // A list name that is not also a template never counts.
        let name = parm(&c.parms, 0).string_val;
        if !world.has_thing_template(&name) {
            return false;
        }
        let templates = self.counted_templates(world, &name);
        if templates.is_empty() {
            return false;
        }
        let answer = count_templates(world, player, &templates, false) != 0;
        self.keep_count(c, answer)
    }

    fn player_lost_object_type(&mut self, world: &dyn ScriptWorld, c: &mut Condition) -> bool {
        let Some(player) = self.player_from_parm(world, &mut c.parms, 0) else {
            return false;
        };
        let name = parm(&c.parms, 1).string_val;
        let templates = self.counted_templates(world, &name);
        if templates.is_empty() {
            return false;
        }
        let sum = count_templates(world, player, &templates, true);
        let before = self.object_count(world, player, &name);
        if sum != before {
            self.set_object_count(player, &name, sum);
        }
        sum < before
    }

    fn special_event(
        &mut self,
        world: &dyn ScriptWorld,
        c: &mut Condition,
        event: PowerEvent,
        from_named: bool,
    ) -> bool {
        let player = self.player_from_parm(world, &mut c.parms, 0);
        let name = parm(&c.parms, 1).string_val;
        let mut source = ObjectId::INVALID;
        if from_named {
            let Some(unit) = self.unit_named(&parm(&c.parms, 2).string_val) else {
                return false;
            };
            source = unit;
        }
        let Some(player) = player else {
            return false;
        };
        let lists = match event {
            PowerEvent::Triggered => &mut self.triggered_special_powers,
            PowerEvent::Midway => &mut self.midway_special_powers,
            PowerEvent::Completed => &mut self.finished_special_powers,
            PowerEvent::Upgrade => &mut self.completed_upgrades,
        };
        let list = &mut lists[player as usize];
        let found = list
            .iter()
            .position(|(n, from)| *n == name && (source == ObjectId::INVALID || source == *from));
        match found {
            Some(i) => {
                list.remove(i);
                true
            }
            None => false,
        }
    }

    fn team_has_units(&self, world: &dyn ScriptWorld, name: &str) -> bool {
        if name == THIS_TEAM {
            return self
                .team_named(world, name)
                .is_some_and(|t| world.team_has_any_units(t));
        }
        if let Some(this) = self.team_named(world, THIS_TEAM) {
            if world.team_name(this) == name {
                return world.team_has_any_units(this);
            }
        }
        world
            .team_prototype(name)
            .is_some_and(|p| p.instances.iter().any(|&t| world.team_has_any_units(t)))
    }

    /// `ScriptConditions::evaluateCondition`; `None` if the type is not ported.
    pub(crate) fn evaluate_world_condition(
        &mut self,
        world: &mut dyn ScriptWorld,
        c: &mut Condition,
    ) -> Option<bool> {
        let world: &dyn ScriptWorld = world;
        let first = parm(&c.parms, 0);
        let answer = match c.condition_type {
            ct::PLAYER_ALL_DESTROYED => self
                .player_from_parm(world, &mut c.parms, 0)
                .map_or(true, |p| !world.player_has_any_objects(p)),
            ct::PLAYER_ALL_BUILDFACILITIES_DESTROYED => self
                .player_from_parm(world, &mut c.parms, 0)
                .map_or(true, |p| !world.player_has_any_build_facility(p)),
            ct::TEAM_DESTROYED => self
                .team_named(world, &first.string_val)
                .is_some_and(|t| !world.team_has_any_objects(t)),
            ct::TEAM_HAS_UNITS => self.team_has_units(world, &first.string_val),
            ct::TEAM_STATE_IS | ct::TEAM_STATE_IS_NOT => {
                let wanted = parm(&c.parms, 1).string_val;
                let not = c.condition_type == ct::TEAM_STATE_IS_NOT;
                self.team_named(world, &first.string_val)
                    .is_some_and(|t| (world.team_state(t) == wanted) != not)
            }
            ct::TEAM_CREATED => self
                .team_named(world, &first.string_val)
                .is_some_and(|t| world.team_is_created(t)),
            ct::TEAM_OWNED_BY_PLAYER => {
                let player = self.player_from_parm(world, &mut c.parms, 1);
                match (player, self.team_named(world, &first.string_val)) {
                    (Some(p), Some(t)) => world.team_controlling_player(t) == Some(p),
                    _ => false,
                }
            }
            ct::NAMED_DESTROYED => match self.unit_named(&first.string_val) {
                Some(unit) => world.object_is_effectively_dead(unit),
                None => self.did_unit_exist(&first.string_val),
            },
            ct::NAMED_NOT_DESTROYED => self
                .unit_named(&first.string_val)
                .is_some_and(|u| !world.object_is_effectively_dead(u)),
            ct::NAMED_DYING => self
                .unit_named(&first.string_val)
                .is_some_and(|u| world.object_is_effectively_dead(u)),
            ct::NAMED_TOTALLY_DEAD => {
                self.unit_named(&first.string_val).is_none()
                    && self.did_unit_exist(&first.string_val)
            }
            ct::NAMED_CREATED => self.unit_named(&first.string_val).is_some(),
            ct::NAMED_OWNED_BY_PLAYER => {
                let player = self.player_from_parm(world, &mut c.parms, 1);
                match (player, self.unit_named(&first.string_val)) {
                    (Some(p), Some(u)) => world.object_controlling_player(u) == Some(p),
                    _ => false,
                }
            }
            ct::UNIT_HEALTH => {
                let health = self
                    .unit_named(&first.string_val)
                    .and_then(|u| world.object_health(u));
                match health {
                    Some((current, initial)) => {
                        let percent = ((current * 100.0 + initial / 2.0) / initial) as Int;
                        compare(
                            parm(&c.parms, 1).int_val,
                            percent,
                            parm(&c.parms, 2).int_val,
                        )
                    }
                    None => false,
                }
            }
            ct::PLAYER_HAS_CREDITS => {
                // The credits are on the left, and both sides compare unsigned.
                match self.player_from_parm(world, &mut c.parms, 2) {
                    Some(p) => compare(
                        parm(&c.parms, 1).int_val,
                        first.int_val as UnsignedInt,
                        world.player_money(p),
                    ),
                    None => false,
                }
            }
            ct::PLAYER_HAS_POWER | ct::PLAYER_HAS_NO_POWER => {
                let powered = self
                    .player_from_parm(world, &mut c.parms, 0)
                    .is_some_and(|p| world.player_power(p).sufficient);
                powered != (c.condition_type == ct::PLAYER_HAS_NO_POWER)
            }
            ct::PLAYER_POWER_COMPARE_PERCENT => {
                match self.player_from_parm(world, &mut c.parms, 0) {
                    Some(p) => {
                        let test = parm(&c.parms, 2).int_val as Real / 100.0;
                        compare(
                            parm(&c.parms, 1).int_val,
                            world.player_power(p).supply_ratio,
                            test,
                        )
                    }
                    None => false,
                }
            }
            ct::PLAYER_EXCESS_POWER_COMPARE_VALUE => {
                match self.player_from_parm(world, &mut c.parms, 0) {
                    Some(p) => {
                        let power = world.player_power(p);
                        compare(
                            parm(&c.parms, 1).int_val,
                            power.production.wrapping_sub(power.consumption),
                            parm(&c.parms, 2).int_val,
                        )
                    }
                    None => false,
                }
            }
            // The count is the second parameter, the player the first.
            ct::PLAYER_HAS_N_OR_FEWER_BUILDINGS => {
                let count = parm(&c.parms, 1).int_val;
                self.player_from_parm(world, &mut c.parms, 0)
                    .is_some_and(|p| count >= world.player_count_buildings(p))
            }
            ct::PLAYER_HAS_N_OR_FEWER_FACTION_BUILDINGS => {
                let count = parm(&c.parms, 1).int_val;
                self.player_from_parm(world, &mut c.parms, 0)
                    .is_some_and(|p| count >= world.player_count_faction_buildings(p))
            }
            ct::PLAYER_HAS_OBJECT_COMPARISON => self.player_has_object_comparison(world, c),
            ct::BUILT_BY_PLAYER => self.built_by_player(world, c),
            ct::PLAYER_LOST_OBJECT_TYPE => self.player_lost_object_type(world, c),
            ct::PLAYER_ACQUIRED_SCIENCE => {
                let player = self.player_from_parm(world, &mut c.parms, 0);
                let science = world.science_from_name(&parm(&c.parms, 1).string_val);
                match (player, science) {
                    (Some(p), Some(science)) => {
                        let list = &mut self.acquired_sciences[p as usize];
                        match list.iter().position(|&s| s == science) {
                            Some(i) => {
                                list.remove(i);
                                true
                            }
                            None => false,
                        }
                    }
                    _ => false,
                }
            }
            ct::PLAYER_TRIGGERED_SPECIAL_POWER => {
                self.special_event(world, c, PowerEvent::Triggered, false)
            }
            ct::PLAYER_TRIGGERED_SPECIAL_POWER_FROM_NAMED => {
                self.special_event(world, c, PowerEvent::Triggered, true)
            }
            ct::PLAYER_MIDWAY_SPECIAL_POWER => {
                self.special_event(world, c, PowerEvent::Midway, false)
            }
            ct::PLAYER_MIDWAY_SPECIAL_POWER_FROM_NAMED => {
                self.special_event(world, c, PowerEvent::Midway, true)
            }
            ct::PLAYER_COMPLETED_SPECIAL_POWER => {
                self.special_event(world, c, PowerEvent::Completed, false)
            }
            ct::PLAYER_COMPLETED_SPECIAL_POWER_FROM_NAMED => {
                self.special_event(world, c, PowerEvent::Completed, true)
            }
            ct::PLAYER_BUILT_UPGRADE => self.special_event(world, c, PowerEvent::Upgrade, false),
            ct::PLAYER_BUILT_UPGRADE_FROM_NAMED => {
                self.special_event(world, c, PowerEvent::Upgrade, true)
            }
            ct::HAS_FINISHED_VIDEO => self.video_complete(&first.string_val),
            ct::HAS_FINISHED_SPEECH => self.sound_complete(world, true, &first.string_val),
            ct::HAS_FINISHED_AUDIO => self.sound_complete(world, false, &first.string_val),
            ct::SKIRMISH_PLAYER_FACTION => {
                let side = parm(&c.parms, 1).string_val;
                self.player_from_parm(world, &mut c.parms, 0)
                    .is_some_and(|p| world.player_side(p) == side)
            }
            ct::SKIRMISH_PLAYER_HAS_BEEN_ATTACKED_BY_PLAYER => {
                match self.player_from_parm(world, &mut c.parms, 0) {
                    Some(p) => self
                        .player_from_parm(world, &mut c.parms, 1)
                        .is_some_and(|by| world.player_attacked_by(p, by)),
                    None => false,
                }
            }
            ct::START_POSITION_IS => {
                let wanted = parm(&c.parms, 1).int_val.wrapping_sub(1);
                self.player_from_parm(world, &mut c.parms, 0)
                    .is_some_and(|p| wanted == world.player_start_index(p))
            }
            ct::MUSIC_TRACK_HAS_COMPLETED => {
                world.music_track_completed(&first.string_val, parm(&c.parms, 1).int_val)
            }
            ct::SUPPLY_SOURCE_SAFE => {
                let frame = world.frame();
                let any_changes = frame > c.custom_frame;
                if !any_changes {
                    if c.custom_data == -1 {
                        return Some(false);
                    }
                    if c.custom_data == 1 {
                        return Some(true);
                    }
                }
                c.custom_frame = frame.wrapping_add(60);
                let min_supply = parm(&c.parms, 1).int_val;
                match self.player_from_parm(world, &mut c.parms, 0) {
                    Some(p) => {
                        let is_safe = world.ai_is_supply_source_safe(p, min_supply);
                        c.custom_data = if is_safe { 1 } else { -1 };
                        is_safe
                    }
                    None => false,
                }
            }
            ct::SUPPLY_SOURCE_ATTACKED => self
                .player_from_parm(world, &mut c.parms, 0)
                .is_some_and(|p| world.ai_is_supply_source_attacked(p)),
            ct::MISSION_ATTEMPTS
            | ct::DEFUNCT_PLAYER_SELECTED_GENERAL
            | ct::DEFUNCT_PLAYER_SELECTED_GENERAL_FROM_NAMED => false,
            _ => return None,
        };
        Some(answer)
    }
}

/// `countObjectsByThingTemplate` summed; nothing to ask for no templates.
fn count_templates(
    world: &dyn ScriptWorld,
    player: ScriptPlayer,
    templates: &[String],
    ignore_dead: bool,
) -> Int {
    if templates.is_empty() {
        return 0;
    }
    world
        .player_count_objects_by_template(player, templates, ignore_dead)
        .iter()
        .fold(0, |a: Int, &b| a.wrapping_add(b))
}
