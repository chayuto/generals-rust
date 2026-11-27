//! `PKT-431`: `ScriptActions`, the actions that need only the engine's state and
//! single calls of [`ScriptWorld`]. Reference:
//! `GameLogic/ScriptEngine/ScriptActions.cpp`. Golden test:
//! `tests/golden_script_engine.rs`.
//!
//! The actions `ScriptEngine::executeActions` carries out itself are in
//! `script_engine.rs`. [`PORTED_ACTIONS`] lists both; any other type goes to
//! the host as unported. A ported action that changes players, teams or objects
//! does so through the host.

use zh_common::types::Int;

use super::script_engine::{
    parm, NamedReveal, ScriptEngine, ScriptUiCommand, ScriptWorld, SequentialScript,
};
use super::script_types::action_type as at;
use super::scripts::ScriptAction;

/// Every action type carried out by the engine or this module.
pub const PORTED_ACTIONS: &[Int] = &[
    // `ScriptEngine::executeActions`.
    at::SET_COUNTER,
    at::SET_TREE_SWAY,
    at::INCREMENT_COUNTER,
    at::DECREMENT_COUNTER,
    at::SET_FLAG,
    at::STOP_TIMER,
    at::RESTART_TIMER,
    at::SET_TIMER,
    at::SET_MILLISECOND_TIMER,
    at::SET_RANDOM_TIMER,
    at::SET_RANDOM_MSEC_TIMER,
    at::ADD_TO_MSEC_TIMER,
    at::SUB_FROM_MSEC_TIMER,
    at::ENABLE_SCRIPT,
    at::DISABLE_SCRIPT,
    at::CALL_SUBROUTINE,
    at::CAMERA_FADE_ADD,
    at::CAMERA_FADE_SUBTRACT,
    at::CAMERA_FADE_SATURATE,
    at::CAMERA_FADE_MULTIPLY,
    at::SET_ATTACK_PRIORITY_THING,
    at::SET_ATTACK_PRIORITY_KIND_OF,
    at::SET_DEFAULT_ATTACK_PRIORITY,
    at::NO_OP,
    at::TEAM_APPLY_ATTACK_PRIORITY_SET,
    at::MUSIC_SET_TRACK,
    at::SHOW_MILITARY_CAPTION,
    at::WAREHOUSE_SET_VALUE,
    at::AUDIO_OVERRIDE_VOLUME_TYPE,
    at::SHOW_WEATHER,
    // `ScriptActions::executeAction`.
    at::OBJECTLIST_ADDOBJECTTYPE,
    at::OBJECTLIST_REMOVEOBJECTTYPE,
    at::TEAM_EXECUTE_SEQUENTIAL_SCRIPT,
    at::TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING,
    at::UNIT_EXECUTE_SEQUENTIAL_SCRIPT,
    at::UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING,
    at::TEAM_STOP_SEQUENTIAL_SCRIPT,
    at::UNIT_STOP_SEQUENTIAL_SCRIPT,
    at::TEAM_GUARD_FOR_FRAMECOUNT,
    at::TEAM_IDLE_FOR_FRAMECOUNT,
    at::UNIT_GUARD_FOR_FRAMECOUNT,
    at::UNIT_IDLE_FOR_FRAMECOUNT,
    at::PLAYER_SET_MONEY,
    at::PLAYER_GIVE_MONEY,
    at::PLAYER_GRANT_SCIENCE,
    at::PLAYER_ENABLE_UNIT_CONSTRUCTION,
    at::PLAYER_DISABLE_UNIT_CONSTRUCTION,
    at::PLAYER_ENABLE_BASE_CONSTRUCTION,
    at::PLAYER_DISABLE_BASE_CONSTRUCTION,
    at::PLAYER_RELATES_PLAYER,
    at::PLAYER_KILL,
    at::TEAM_SET_STATE,
    at::TEAM_DELETE,
    at::TEAM_DELETE_LIVING,
    at::TEAM_KILL,
    at::NAMED_DELETE,
    at::NAMED_KILL,
    at::NAMED_SET_TOPPLE_DIRECTION,
    at::MAP_REVEAL_PERMANENTLY_AT_WAYPOINT,
    at::MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT,
    at::OBJECT_ALLOW_BONUSES,
    at::CHOOSE_VICTIM_ALWAYS_USES_NORMAL,
    at::FREEZE_TIME,
    at::UNFREEZE_TIME,
    at::CAMERA_LETTERBOX_BEGIN,
    at::CAMERA_LETTERBOX_END,
    at::VICTORY,
    at::QUICKVICTORY,
    at::DEFEAT,
    at::LOCALDEFEAT,
    at::SKIRMISH_BUILD_BUILDING,
    at::AI_PLAYER_BUILD_SUPPLY_CENTER,
];

impl ScriptEngine {
    /// `ScriptActions::executeAction`; `false` if the type is not ported.
    pub(crate) fn execute_world_action(
        &mut self,
        world: &mut dyn ScriptWorld,
        a: &mut ScriptAction,
    ) -> bool {
        let first = parm(&a.parms, 0).string_val;
        let second = parm(&a.parms, 1);
        let third = parm(&a.parms, 2);
        match a.action_type {
            at::OBJECTLIST_ADDOBJECTTYPE => {
                self.maintain_object_type_list(&first, &second.string_val, true)
            }
            at::OBJECTLIST_REMOVEOBJECTTYPE => {
                self.maintain_object_type_list(&first, &second.string_val, false)
            }
            at::TEAM_EXECUTE_SEQUENTIAL_SCRIPT => {
                self.team_start_sequential(world, &first, &second.string_val, 0)
            }
            at::TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING => self.team_start_sequential(
                world,
                &first,
                &second.string_val,
                third.int_val.wrapping_sub(1),
            ),
            at::UNIT_EXECUTE_SEQUENTIAL_SCRIPT => {
                self.unit_start_sequential(&first, &second.string_val, 0)
            }
            at::UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING => self.unit_start_sequential(
                &first,
                &second.string_val,
                third.int_val.wrapping_sub(1),
            ),
            // The current player's.
            at::SKIRMISH_BUILD_BUILDING => {
                if let Some(p) = self.current_player {
                    world.ai_build_building(p, &first);
                }
            }
            at::AI_PLAYER_BUILD_SUPPLY_CENTER => {
                if let Some(p) = self.player_from_name(world, &first) {
                    world.ai_build_by_supplies(p, third.int_val, &second.string_val);
                }
            }
            at::TEAM_STOP_SEQUENTIAL_SCRIPT => {
                if let Some(team) = self.team_named(world, &first) {
                    // The same as the team being destroyed.
                    self.notify_of_team_destruction(team);
                }
            }
            at::UNIT_STOP_SEQUENTIAL_SCRIPT => {
                if let Some(unit) = self.unit_named(&first) {
                    self.remove_sequential_scripts_of_object(unit);
                }
            }
            // The guard action idles the team too.
            at::TEAM_GUARD_FOR_FRAMECOUNT | at::TEAM_IDLE_FOR_FRAMECOUNT => {
                if let Some(team) = self.team_named(world, &first) {
                    world.team_group_idle(team);
                    self.set_sequential_timer_team(team, second.int_val);
                }
            }
            at::UNIT_IDLE_FOR_FRAMECOUNT | at::UNIT_GUARD_FOR_FRAMECOUNT => {
                if let Some(unit) = self.unit_named(&first) {
                    if world.object_ai_is_idle(unit).is_some() {
                        if a.action_type == at::UNIT_IDLE_FOR_FRAMECOUNT {
                            world.object_ai_idle(unit);
                        } else {
                            world.object_ai_guard_here(unit, true);
                        }
                        self.set_sequential_timer_object(unit, second.int_val);
                    }
                }
            }
            at::TEAM_APPLY_ATTACK_PRIORITY_SET => {
                if let Some(team) = self.team_named(world, &first) {
                    // An unknown set is the default one, which has no name.
                    let name = self.attack_info(&second.string_val).name.clone();
                    let named = (!name.is_empty()).then_some(name.as_str());
                    world.set_team_attack_priority(team, named, &name);
                }
            }
            at::WAREHOUSE_SET_VALUE => {
                if let Some(unit) = self.unit_named(&first) {
                    world.set_warehouse_cash(unit, second.int_val);
                }
            }
            at::MUSIC_SET_TRACK => {
                self.current_track_name = first.clone();
                world.ui(ScriptUiCommand::Client(a.action_type));
            }
            at::SHOW_MILITARY_CAPTION | at::AUDIO_OVERRIDE_VOLUME_TYPE | at::SHOW_WEATHER => {
                world.ui(ScriptUiCommand::Client(a.action_type));
            }
            at::PLAYER_SET_MONEY => {
                if let Some(p) = self.player_from_name(world, &first) {
                    let held = world.player_money(p);
                    world.money_withdraw(p, held);
                    world.money_deposit(p, second.int_val as u32, false, false);
                }
            }
            at::PLAYER_GIVE_MONEY => {
                if let Some(p) = self.player_from_name(world, &first) {
                    let money = second.int_val;
                    if money < 0 {
                        world.money_withdraw(p, money.wrapping_neg() as u32);
                    } else {
                        world.money_deposit(p, money as u32, true, true);
                    }
                }
            }
            at::PLAYER_GRANT_SCIENCE => {
                if let Some(p) = self.player_from_name(world, &first) {
                    if let Some(science) = world.science_from_name(&second.string_val) {
                        if world.grant_science(p, science) {
                            self.notify_of_acquired_science(p, science);
                        }
                    }
                }
            }
            at::PLAYER_ENABLE_UNIT_CONSTRUCTION | at::PLAYER_DISABLE_UNIT_CONSTRUCTION => {
                if let Some(p) = self.player_from_name(world, &first) {
                    world.set_can_build_units(
                        p,
                        a.action_type == at::PLAYER_ENABLE_UNIT_CONSTRUCTION,
                    );
                }
            }
            at::PLAYER_ENABLE_BASE_CONSTRUCTION | at::PLAYER_DISABLE_BASE_CONSTRUCTION => {
                if let Some(p) = self.player_from_name(world, &first) {
                    world.set_can_build_base(
                        p,
                        a.action_type == at::PLAYER_ENABLE_BASE_CONSTRUCTION,
                    );
                }
            }
            at::PLAYER_RELATES_PLAYER => {
                // The second player is looked up first.
                let to = self.player_from_name(world, &second.string_val);
                let from = self.player_from_name(world, &first);
                if let (Some(from), Some(to)) = (from, to) {
                    world.set_player_relationship(from, to, third.int_val);
                }
            }
            at::PLAYER_KILL => {
                if let Some(p) = self.player_from_name(world, &first) {
                    world.kill_player(p);
                }
            }
            at::TEAM_SET_STATE => {
                if let Some(team) = self.team_named(world, &first) {
                    world.set_team_state(team, &second.string_val);
                }
            }
            at::TEAM_DELETE | at::TEAM_DELETE_LIVING => {
                if let Some(team) = self.team_named(world, &first) {
                    world.delete_team(team, a.action_type == at::TEAM_DELETE_LIVING);
                }
            }
            at::TEAM_KILL => {
                if let Some(team) = self.team_named(world, &first) {
                    world.kill_team(team);
                }
            }
            at::NAMED_DELETE => {
                if let Some(unit) = self.unit_named(&first) {
                    world.destroy_object(unit);
                }
            }
            at::NAMED_KILL => {
                if let Some(unit) = self.unit_named(&first) {
                    world.kill_object(unit);
                }
            }
            at::NAMED_SET_TOPPLE_DIRECTION => {
                self.set_topple_direction(&first, second.get_coord3d())
            }
            at::MAP_REVEAL_PERMANENTLY_AT_WAYPOINT => {
                let name = parm(&a.parms, 3).string_val;
                self.create_named_map_reveal(NamedReveal {
                    reveal_name: name.clone(),
                    waypoint_name: first,
                    radius: second.real_val,
                    player_name: third.string_val,
                });
                self.apply_named_map_reveal(world, &name, false);
            }
            at::MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT => {
                self.apply_named_map_reveal(world, &first, true);
                self.remove_named_map_reveal(&first);
            }
            at::OBJECT_ALLOW_BONUSES => {
                let enable = parm(&a.parms, 0).int_val != 0;
                world.set_all_difficulty_bonuses(enable);
                self.objects_receive_difficulty_bonus = enable;
            }
            at::CHOOSE_VICTIM_ALWAYS_USES_NORMAL => {
                self.choose_victim_always_uses_normal = parm(&a.parms, 0).int_val != 0;
            }
            at::FREEZE_TIME => self.freeze_by_script = true,
            at::UNFREEZE_TIME => self.freeze_by_script = false,
            at::CAMERA_LETTERBOX_BEGIN | at::CAMERA_LETTERBOX_END => {
                let begin = a.action_type == at::CAMERA_LETTERBOX_BEGIN;
                self.letter_box_active = begin;
                world.ui(ScriptUiCommand::LetterBox(begin));
            }
            at::VICTORY | at::DEFEAT => {
                let won = a.action_type == at::VICTORY;
                world.ui(ScriptUiCommand::CloseWindows);
                world.ui(ScriptUiCommand::DisableInput);
                world.ui(if won {
                    ScriptUiCommand::ShowVictory
                } else {
                    ScriptUiCommand::ShowDefeat
                });
                world.ui(ScriptUiCommand::SetVictorious(won));
                self.start_end_game_timer();
            }
            at::QUICKVICTORY => {
                world.ui(ScriptUiCommand::CloseWindows);
                world.ui(ScriptUiCommand::DisableInput);
                world.ui(ScriptUiCommand::SetVictorious(true));
                self.start_quick_end_game_timer();
            }
            at::LOCALDEFEAT => {
                self.shown_mp_local_defeat_window = true;
                world.ui(ScriptUiCommand::CloseWindows);
                world.ui(ScriptUiCommand::ShowLocalDefeat);
                world.ui(ScriptUiCommand::SetVictorious(false));
                self.start_close_window_timer();
            }
            _ => return false,
        }
        true
    }

    /// `doTeamStartSequentialScript`: the team is idled first.
    fn team_start_sequential(
        &mut self,
        world: &mut dyn ScriptWorld,
        team: &str,
        script: &str,
        loops: Int,
    ) {
        let Some(team) = self.team_named(world, team) else {
            return;
        };
        let Some(script) = self.find_script(script) else {
            return;
        };
        world.team_group_idle(team);
        self.append_sequential_script(SequentialScript {
            team: Some(team),
            script: Some(script),
            times_to_loop: loops,
            ..SequentialScript::default()
        });
    }

    /// `doUnitStartSequentialScript`.
    fn unit_start_sequential(&mut self, unit: &str, script: &str, loops: Int) {
        let Some(object_id) = self.unit_named(unit) else {
            return;
        };
        let Some(script) = self.find_script(script) else {
            return;
        };
        self.append_sequential_script(SequentialScript {
            object_id,
            script: Some(script),
            times_to_loop: loops,
            ..SequentialScript::default()
        });
    }

    /// Whether the player of the local machine has been shown its defeat.
    pub fn has_shown_mp_local_defeat_window(&self) -> bool {
        self.shown_mp_local_defeat_window
    }
}
