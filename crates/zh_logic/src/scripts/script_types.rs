//! Condition and action numbers, and the templates `ScriptEngine::init` registers
//! for them (internal name and parameter types). Part of `PKT-433`
//! (`ScriptEngine.cpp`, `Scripts.h`); checked row by row against the reference by
//! `tests/golden_script_engine.rs`.

use zh_common::types::Int;

/// `(type, internal name, parameter types)`; types not listed have no template.
pub type TemplateRow = (Int, &'static str, &'static [Int]);

/// `Condition::ConditionType`.
pub mod condition_type {
    use zh_common::types::Int;

    pub const CONDITION_FALSE: Int = 0;
    pub const COUNTER: Int = 1;
    pub const FLAG: Int = 2;
    pub const CONDITION_TRUE: Int = 3;
    pub const TIMER_EXPIRED: Int = 4;
    pub const PLAYER_ALL_DESTROYED: Int = 5;
    pub const PLAYER_ALL_BUILDFACILITIES_DESTROYED: Int = 6;
    pub const TEAM_INSIDE_AREA_PARTIALLY: Int = 7;
    pub const TEAM_DESTROYED: Int = 8;
    pub const CAMERA_MOVEMENT_FINISHED: Int = 9;
    pub const TEAM_HAS_UNITS: Int = 10;
    pub const TEAM_STATE_IS: Int = 11;
    pub const TEAM_STATE_IS_NOT: Int = 12;
    pub const NAMED_INSIDE_AREA: Int = 13;
    pub const NAMED_OUTSIDE_AREA: Int = 14;
    pub const NAMED_DESTROYED: Int = 15;
    pub const NAMED_NOT_DESTROYED: Int = 16;
    pub const TEAM_INSIDE_AREA_ENTIRELY: Int = 17;
    pub const TEAM_OUTSIDE_AREA_ENTIRELY: Int = 18;
    pub const NAMED_ATTACKED_BY_OBJECTTYPE: Int = 19;
    pub const TEAM_ATTACKED_BY_OBJECTTYPE: Int = 20;
    pub const NAMED_ATTACKED_BY_PLAYER: Int = 21;
    pub const TEAM_ATTACKED_BY_PLAYER: Int = 22;
    pub const BUILT_BY_PLAYER: Int = 23;
    pub const NAMED_CREATED: Int = 24;
    pub const TEAM_CREATED: Int = 25;
    pub const PLAYER_HAS_CREDITS: Int = 26;
    pub const NAMED_DISCOVERED: Int = 27;
    pub const TEAM_DISCOVERED: Int = 28;
    pub const MISSION_ATTEMPTS: Int = 29;
    pub const NAMED_OWNED_BY_PLAYER: Int = 30;
    pub const TEAM_OWNED_BY_PLAYER: Int = 31;
    pub const PLAYER_HAS_N_OR_FEWER_BUILDINGS: Int = 32;
    pub const PLAYER_HAS_POWER: Int = 33;
    pub const NAMED_REACHED_WAYPOINTS_END: Int = 34;
    pub const TEAM_REACHED_WAYPOINTS_END: Int = 35;
    pub const NAMED_SELECTED: Int = 36;
    pub const NAMED_ENTERED_AREA: Int = 37;
    pub const NAMED_EXITED_AREA: Int = 38;
    pub const TEAM_ENTERED_AREA_ENTIRELY: Int = 39;
    pub const TEAM_ENTERED_AREA_PARTIALLY: Int = 40;
    pub const TEAM_EXITED_AREA_ENTIRELY: Int = 41;
    pub const TEAM_EXITED_AREA_PARTIALLY: Int = 42;
    pub const MULTIPLAYER_ALLIED_VICTORY: Int = 43;
    pub const MULTIPLAYER_ALLIED_DEFEAT: Int = 44;
    pub const MULTIPLAYER_PLAYER_DEFEAT: Int = 45;
    pub const PLAYER_HAS_NO_POWER: Int = 46;
    pub const HAS_FINISHED_VIDEO: Int = 47;
    pub const HAS_FINISHED_SPEECH: Int = 48;
    pub const HAS_FINISHED_AUDIO: Int = 49;
    pub const BUILDING_ENTERED_BY_PLAYER: Int = 50;
    pub const ENEMY_SIGHTED: Int = 51;
    pub const UNIT_HEALTH: Int = 52;
    pub const BRIDGE_REPAIRED: Int = 53;
    pub const BRIDGE_BROKEN: Int = 54;
    pub const NAMED_DYING: Int = 55;
    pub const NAMED_TOTALLY_DEAD: Int = 56;
    pub const PLAYER_HAS_OBJECT_COMPARISON: Int = 57;
    pub const OBSOLETE_SCRIPT_1: Int = 58;
    pub const OBSOLETE_SCRIPT_2: Int = 59;
    pub const PLAYER_TRIGGERED_SPECIAL_POWER: Int = 60;
    pub const PLAYER_COMPLETED_SPECIAL_POWER: Int = 61;
    pub const PLAYER_MIDWAY_SPECIAL_POWER: Int = 62;
    pub const PLAYER_TRIGGERED_SPECIAL_POWER_FROM_NAMED: Int = 63;
    pub const PLAYER_COMPLETED_SPECIAL_POWER_FROM_NAMED: Int = 64;
    pub const PLAYER_MIDWAY_SPECIAL_POWER_FROM_NAMED: Int = 65;
    pub const DEFUNCT_PLAYER_SELECTED_GENERAL: Int = 66;
    pub const DEFUNCT_PLAYER_SELECTED_GENERAL_FROM_NAMED: Int = 67;
    pub const PLAYER_BUILT_UPGRADE: Int = 68;
    pub const PLAYER_BUILT_UPGRADE_FROM_NAMED: Int = 69;
    pub const PLAYER_DESTROYED_N_BUILDINGS_PLAYER: Int = 70;
    pub const UNIT_COMPLETED_SEQUENTIAL_EXECUTION: Int = 71;
    pub const TEAM_COMPLETED_SEQUENTIAL_EXECUTION: Int = 72;
    pub const PLAYER_HAS_COMPARISON_UNIT_TYPE_IN_TRIGGER_AREA: Int = 73;
    pub const PLAYER_HAS_COMPARISON_UNIT_KIND_IN_TRIGGER_AREA: Int = 74;
    pub const UNIT_EMPTIED: Int = 75;
    pub const TYPE_SIGHTED: Int = 76;
    pub const NAMED_BUILDING_IS_EMPTY: Int = 77;
    pub const PLAYER_HAS_N_OR_FEWER_FACTION_BUILDINGS: Int = 78;
    pub const UNIT_HAS_OBJECT_STATUS: Int = 79;
    pub const TEAM_ALL_HAS_OBJECT_STATUS: Int = 80;
    pub const TEAM_SOME_HAVE_OBJECT_STATUS: Int = 81;
    pub const PLAYER_POWER_COMPARE_PERCENT: Int = 82;
    pub const PLAYER_EXCESS_POWER_COMPARE_VALUE: Int = 83;
    pub const SKIRMISH_SPECIAL_POWER_READY: Int = 84;
    pub const SKIRMISH_VALUE_IN_AREA: Int = 85;
    pub const SKIRMISH_PLAYER_FACTION: Int = 86;
    pub const SKIRMISH_SUPPLIES_VALUE_WITHIN_DISTANCE: Int = 87;
    pub const SKIRMISH_TECH_BUILDING_WITHIN_DISTANCE: Int = 88;
    pub const SKIRMISH_COMMAND_BUTTON_READY_ALL: Int = 89;
    pub const SKIRMISH_COMMAND_BUTTON_READY_PARTIAL: Int = 90;
    pub const SKIRMISH_UNOWNED_FACTION_UNIT_EXISTS: Int = 91;
    pub const SKIRMISH_PLAYER_HAS_PREREQUISITE_TO_BUILD: Int = 92;
    pub const SKIRMISH_PLAYER_HAS_COMPARISON_GARRISONED: Int = 93;
    pub const SKIRMISH_PLAYER_HAS_COMPARISON_CAPTURED_UNITS: Int = 94;
    pub const SKIRMISH_NAMED_AREA_EXIST: Int = 95;
    pub const SKIRMISH_PLAYER_HAS_UNITS_IN_AREA: Int = 96;
    pub const SKIRMISH_PLAYER_HAS_BEEN_ATTACKED_BY_PLAYER: Int = 97;
    pub const SKIRMISH_PLAYER_IS_OUTSIDE_AREA: Int = 98;
    pub const SKIRMISH_PLAYER_HAS_DISCOVERED_PLAYER: Int = 99;
    pub const PLAYER_ACQUIRED_SCIENCE: Int = 100;
    pub const PLAYER_HAS_SCIENCEPURCHASEPOINTS: Int = 101;
    pub const PLAYER_CAN_PURCHASE_SCIENCE: Int = 102;
    pub const MUSIC_TRACK_HAS_COMPLETED: Int = 103;
    pub const PLAYER_LOST_OBJECT_TYPE: Int = 104;
    pub const SUPPLY_SOURCE_SAFE: Int = 105;
    pub const SUPPLY_SOURCE_ATTACKED: Int = 106;
    pub const START_POSITION_IS: Int = 107;
    pub const NAMED_HAS_FREE_CONTAINER_SLOTS: Int = 108;
    pub const NUM_ITEMS: Int = 109;
}

/// `ScriptAction::ScriptActionType`.
pub mod action_type {
    use zh_common::types::Int;

    pub const DEBUG_MESSAGE_BOX: Int = 0;
    pub const SET_FLAG: Int = 1;
    pub const SET_COUNTER: Int = 2;
    pub const VICTORY: Int = 3;
    pub const DEFEAT: Int = 4;
    pub const NO_OP: Int = 5;
    pub const SET_TIMER: Int = 6;
    pub const PLAY_SOUND_EFFECT: Int = 7;
    pub const ENABLE_SCRIPT: Int = 8;
    pub const DISABLE_SCRIPT: Int = 9;
    pub const CALL_SUBROUTINE: Int = 10;
    pub const PLAY_SOUND_EFFECT_AT: Int = 11;
    pub const DAMAGE_MEMBERS_OF_TEAM: Int = 12;
    pub const MOVE_TEAM_TO: Int = 13;
    pub const MOVE_CAMERA_TO: Int = 14;
    pub const INCREMENT_COUNTER: Int = 15;
    pub const DECREMENT_COUNTER: Int = 16;
    pub const MOVE_CAMERA_ALONG_WAYPOINT_PATH: Int = 17;
    pub const ROTATE_CAMERA: Int = 18;
    pub const RESET_CAMERA: Int = 19;
    pub const SET_MILLISECOND_TIMER: Int = 20;
    pub const CAMERA_MOD_FREEZE_TIME: Int = 21;
    pub const SET_VISUAL_SPEED_MULTIPLIER: Int = 22;
    pub const CREATE_OBJECT: Int = 23;
    pub const SUSPEND_BACKGROUND_SOUNDS: Int = 24;
    pub const RESUME_BACKGROUND_SOUNDS: Int = 25;
    pub const CAMERA_MOD_SET_FINAL_ZOOM: Int = 26;
    pub const CAMERA_MOD_SET_FINAL_PITCH: Int = 27;
    pub const CAMERA_MOD_FREEZE_ANGLE: Int = 28;
    pub const CAMERA_MOD_SET_FINAL_SPEED_MULTIPLIER: Int = 29;
    pub const CAMERA_MOD_SET_ROLLING_AVERAGE: Int = 30;
    pub const CAMERA_MOD_FINAL_LOOK_TOWARD: Int = 31;
    pub const CAMERA_MOD_LOOK_TOWARD: Int = 32;
    pub const TEAM_ATTACK_TEAM: Int = 33;
    pub const CREATE_REINFORCEMENT_TEAM: Int = 34;
    pub const MOVE_CAMERA_TO_SELECTION: Int = 35;
    pub const TEAM_FOLLOW_WAYPOINTS: Int = 36;
    pub const TEAM_SET_STATE: Int = 37;
    pub const MOVE_NAMED_UNIT_TO: Int = 38;
    pub const NAMED_ATTACK_NAMED: Int = 39;
    pub const CREATE_NAMED_ON_TEAM_AT_WAYPOINT: Int = 40;
    pub const CREATE_UNNAMED_ON_TEAM_AT_WAYPOINT: Int = 41;
    pub const NAMED_APPLY_ATTACK_PRIORITY_SET: Int = 42;
    pub const TEAM_APPLY_ATTACK_PRIORITY_SET: Int = 43;
    pub const SET_BASE_CONSTRUCTION_SPEED: Int = 44;
    pub const NAMED_SET_ATTITUDE: Int = 45;
    pub const TEAM_SET_ATTITUDE: Int = 46;
    pub const NAMED_ATTACK_AREA: Int = 47;
    pub const NAMED_ATTACK_TEAM: Int = 48;
    pub const TEAM_ATTACK_AREA: Int = 49;
    pub const TEAM_ATTACK_NAMED: Int = 50;
    pub const TEAM_LOAD_TRANSPORTS: Int = 51;
    pub const NAMED_ENTER_NAMED: Int = 52;
    pub const TEAM_ENTER_NAMED: Int = 53;
    pub const NAMED_EXIT_ALL: Int = 54;
    pub const TEAM_EXIT_ALL: Int = 55;
    pub const NAMED_FOLLOW_WAYPOINTS: Int = 56;
    pub const NAMED_GUARD: Int = 57;
    pub const TEAM_GUARD: Int = 58;
    pub const NAMED_HUNT: Int = 59;
    pub const TEAM_HUNT: Int = 60;
    pub const PLAYER_SELL_EVERYTHING: Int = 61;
    pub const PLAYER_DISABLE_BASE_CONSTRUCTION: Int = 62;
    pub const PLAYER_DISABLE_FACTORIES: Int = 63;
    pub const PLAYER_DISABLE_UNIT_CONSTRUCTION: Int = 64;
    pub const PLAYER_ENABLE_BASE_CONSTRUCTION: Int = 65;
    pub const PLAYER_ENABLE_FACTORIES: Int = 66;
    pub const PLAYER_ENABLE_UNIT_CONSTRUCTION: Int = 67;
    pub const CAMERA_MOVE_HOME: Int = 68;
    pub const BUILD_TEAM: Int = 69;
    pub const NAMED_DAMAGE: Int = 70;
    pub const NAMED_DELETE: Int = 71;
    pub const TEAM_DELETE: Int = 72;
    pub const NAMED_KILL: Int = 73;
    pub const TEAM_KILL: Int = 74;
    pub const PLAYER_KILL: Int = 75;
    pub const DISPLAY_TEXT: Int = 76;
    pub const CAMEO_FLASH: Int = 77;
    pub const NAMED_FLASH: Int = 78;
    pub const TEAM_FLASH: Int = 79;
    pub const MOVIE_PLAY_FULLSCREEN: Int = 80;
    pub const MOVIE_PLAY_RADAR: Int = 81;
    pub const SOUND_PLAY_NAMED: Int = 82;
    pub const SPEECH_PLAY: Int = 83;
    pub const PLAYER_TRANSFER_OWNERSHIP_PLAYER: Int = 84;
    pub const NAMED_TRANSFER_OWNERSHIP_PLAYER: Int = 85;
    pub const PLAYER_RELATES_PLAYER: Int = 86;
    pub const RADAR_CREATE_EVENT: Int = 87;
    pub const RADAR_DISABLE: Int = 88;
    pub const RADAR_ENABLE: Int = 89;
    pub const MAP_REVEAL_AT_WAYPOINT: Int = 90;
    pub const TEAM_AVAILABLE_FOR_RECRUITMENT: Int = 91;
    pub const TEAM_COLLECT_NEARBY_FOR_TEAM: Int = 92;
    pub const TEAM_MERGE_INTO_TEAM: Int = 93;
    pub const DISABLE_INPUT: Int = 94;
    pub const ENABLE_INPUT: Int = 95;
    pub const PLAYER_HUNT: Int = 96;
    pub const SOUND_AMBIENT_PAUSE: Int = 97;
    pub const SOUND_AMBIENT_RESUME: Int = 98;
    pub const MUSIC_SET_TRACK: Int = 99;
    pub const SET_TREE_SWAY: Int = 100;
    pub const DEBUG_STRING: Int = 101;
    pub const MAP_REVEAL_ALL: Int = 102;
    pub const TEAM_GARRISON_SPECIFIC_BUILDING: Int = 103;
    pub const EXIT_SPECIFIC_BUILDING: Int = 104;
    pub const TEAM_GARRISON_NEAREST_BUILDING: Int = 105;
    pub const TEAM_EXIT_ALL_BUILDINGS: Int = 106;
    pub const NAMED_GARRISON_SPECIFIC_BUILDING: Int = 107;
    pub const NAMED_GARRISON_NEAREST_BUILDING: Int = 108;
    pub const NAMED_EXIT_BUILDING: Int = 109;
    pub const PLAYER_GARRISON_ALL_BUILDINGS: Int = 110;
    pub const PLAYER_EXIT_ALL_BUILDINGS: Int = 111;
    pub const TEAM_WANDER: Int = 112;
    pub const TEAM_PANIC: Int = 113;
    pub const SETUP_CAMERA: Int = 114;
    pub const CAMERA_LETTERBOX_BEGIN: Int = 115;
    pub const CAMERA_LETTERBOX_END: Int = 116;
    pub const ZOOM_CAMERA: Int = 117;
    pub const PITCH_CAMERA: Int = 118;
    pub const CAMERA_FOLLOW_NAMED: Int = 119;
    pub const OVERSIZE_TERRAIN: Int = 120;
    pub const CAMERA_FADE_ADD: Int = 121;
    pub const CAMERA_FADE_SUBTRACT: Int = 122;
    pub const CAMERA_FADE_SATURATE: Int = 123;
    pub const CAMERA_FADE_MULTIPLY: Int = 124;
    pub const CAMERA_BW_MODE_BEGIN: Int = 125;
    pub const CAMERA_BW_MODE_END: Int = 126;
    pub const DRAW_SKYBOX_BEGIN: Int = 127;
    pub const DRAW_SKYBOX_END: Int = 128;
    pub const SET_ATTACK_PRIORITY_THING: Int = 129;
    pub const SET_ATTACK_PRIORITY_KIND_OF: Int = 130;
    pub const SET_DEFAULT_ATTACK_PRIORITY: Int = 131;
    pub const CAMERA_STOP_FOLLOW: Int = 132;
    pub const CAMERA_MOTION_BLUR: Int = 133;
    pub const CAMERA_MOTION_BLUR_JUMP: Int = 134;
    pub const CAMERA_MOTION_BLUR_FOLLOW: Int = 135;
    pub const CAMERA_MOTION_BLUR_END_FOLLOW: Int = 136;
    pub const FREEZE_TIME: Int = 137;
    pub const UNFREEZE_TIME: Int = 138;
    pub const SHOW_MILITARY_CAPTION: Int = 139;
    pub const CAMERA_SET_AUDIBLE_DISTANCE: Int = 140;
    pub const SET_STOPPING_DISTANCE: Int = 141;
    pub const NAMED_SET_STOPPING_DISTANCE: Int = 142;
    pub const SET_FPS_LIMIT: Int = 143;
    pub const MUSIC_SET_VOLUME: Int = 144;
    pub const MAP_SHROUD_AT_WAYPOINT: Int = 145;
    pub const MAP_SHROUD_ALL: Int = 146;
    pub const SET_RANDOM_TIMER: Int = 147;
    pub const SET_RANDOM_MSEC_TIMER: Int = 148;
    pub const STOP_TIMER: Int = 149;
    pub const RESTART_TIMER: Int = 150;
    pub const ADD_TO_MSEC_TIMER: Int = 151;
    pub const SUB_FROM_MSEC_TIMER: Int = 152;
    pub const TEAM_TRANSFER_TO_PLAYER: Int = 153;
    pub const PLAYER_SET_MONEY: Int = 154;
    pub const PLAYER_GIVE_MONEY: Int = 155;
    pub const DISABLE_SPECIAL_POWER_DISPLAY: Int = 156;
    pub const ENABLE_SPECIAL_POWER_DISPLAY: Int = 157;
    pub const NAMED_HIDE_SPECIAL_POWER_DISPLAY: Int = 158;
    pub const NAMED_SHOW_SPECIAL_POWER_DISPLAY: Int = 159;
    pub const DISPLAY_COUNTDOWN_TIMER: Int = 160;
    pub const HIDE_COUNTDOWN_TIMER: Int = 161;
    pub const ENABLE_COUNTDOWN_TIMER_DISPLAY: Int = 162;
    pub const DISABLE_COUNTDOWN_TIMER_DISPLAY: Int = 163;
    pub const NAMED_STOP_SPECIAL_POWER_COUNTDOWN: Int = 164;
    pub const NAMED_START_SPECIAL_POWER_COUNTDOWN: Int = 165;
    pub const NAMED_SET_SPECIAL_POWER_COUNTDOWN: Int = 166;
    pub const NAMED_ADD_SPECIAL_POWER_COUNTDOWN: Int = 167;
    pub const NAMED_FIRE_SPECIAL_POWER_AT_WAYPOINT: Int = 168;
    pub const NAMED_FIRE_SPECIAL_POWER_AT_NAMED: Int = 169;
    pub const REFRESH_RADAR: Int = 170;
    pub const CAMERA_TETHER_NAMED: Int = 171;
    pub const CAMERA_STOP_TETHER_NAMED: Int = 172;
    pub const CAMERA_SET_DEFAULT: Int = 173;
    pub const NAMED_STOP: Int = 174;
    pub const TEAM_STOP: Int = 175;
    pub const TEAM_STOP_AND_DISBAND: Int = 176;
    pub const RECRUIT_TEAM: Int = 177;
    pub const TEAM_SET_OVERRIDE_RELATION_TO_TEAM: Int = 178;
    pub const TEAM_REMOVE_OVERRIDE_RELATION_TO_TEAM: Int = 179;
    pub const TEAM_REMOVE_ALL_OVERRIDE_RELATIONS: Int = 180;
    pub const CAMERA_LOOK_TOWARD_OBJECT: Int = 181;
    pub const NAMED_FIRE_WEAPON_FOLLOWING_WAYPOINT_PATH: Int = 182;
    pub const TEAM_SET_OVERRIDE_RELATION_TO_PLAYER: Int = 183;
    pub const TEAM_REMOVE_OVERRIDE_RELATION_TO_PLAYER: Int = 184;
    pub const PLAYER_SET_OVERRIDE_RELATION_TO_TEAM: Int = 185;
    pub const PLAYER_REMOVE_OVERRIDE_RELATION_TO_TEAM: Int = 186;
    pub const UNIT_EXECUTE_SEQUENTIAL_SCRIPT: Int = 187;
    pub const UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING: Int = 188;
    pub const UNIT_STOP_SEQUENTIAL_SCRIPT: Int = 189;
    pub const TEAM_EXECUTE_SEQUENTIAL_SCRIPT: Int = 190;
    pub const TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING: Int = 191;
    pub const TEAM_STOP_SEQUENTIAL_SCRIPT: Int = 192;
    pub const UNIT_GUARD_FOR_FRAMECOUNT: Int = 193;
    pub const UNIT_IDLE_FOR_FRAMECOUNT: Int = 194;
    pub const TEAM_GUARD_FOR_FRAMECOUNT: Int = 195;
    pub const TEAM_IDLE_FOR_FRAMECOUNT: Int = 196;
    pub const WATER_CHANGE_HEIGHT: Int = 197;
    pub const NAMED_USE_COMMANDBUTTON_ABILITY_ON_NAMED: Int = 198;
    pub const NAMED_USE_COMMANDBUTTON_ABILITY_AT_WAYPOINT: Int = 199;
    pub const WATER_CHANGE_HEIGHT_OVER_TIME: Int = 200;
    pub const MAP_SWITCH_BORDER: Int = 201;
    pub const TEAM_GUARD_POSITION: Int = 202;
    pub const TEAM_GUARD_OBJECT: Int = 203;
    pub const TEAM_GUARD_AREA: Int = 204;
    pub const OBJECT_FORCE_SELECT: Int = 205;
    pub const CAMERA_LOOK_TOWARD_WAYPOINT: Int = 206;
    pub const UNIT_DESTROY_ALL_CONTAINED: Int = 207;
    pub const RADAR_FORCE_ENABLE: Int = 208;
    pub const RADAR_REVERT_TO_NORMAL: Int = 209;
    pub const SCREEN_SHAKE: Int = 210;
    pub const TECHTREE_MODIFY_BUILDABILITY_OBJECT: Int = 211;
    pub const WAREHOUSE_SET_VALUE: Int = 212;
    pub const OBJECT_CREATE_RADAR_EVENT: Int = 213;
    pub const TEAM_CREATE_RADAR_EVENT: Int = 214;
    pub const DISPLAY_CINEMATIC_TEXT: Int = 215;
    pub const DEBUG_CRASH_BOX: Int = 216;
    pub const SOUND_DISABLE_TYPE: Int = 217;
    pub const SOUND_ENABLE_TYPE: Int = 218;
    pub const SOUND_ENABLE_ALL: Int = 219;
    pub const AUDIO_OVERRIDE_VOLUME_TYPE: Int = 220;
    pub const AUDIO_RESTORE_VOLUME_TYPE: Int = 221;
    pub const AUDIO_RESTORE_VOLUME_ALL_TYPE: Int = 222;
    pub const INGAME_POPUP_MESSAGE: Int = 223;
    pub const SET_CAVE_INDEX: Int = 224;
    pub const NAMED_SET_HELD: Int = 225;
    pub const NAMED_SET_TOPPLE_DIRECTION: Int = 226;
    pub const UNIT_MOVE_TOWARDS_NEAREST_OBJECT_TYPE: Int = 227;
    pub const TEAM_MOVE_TOWARDS_NEAREST_OBJECT_TYPE: Int = 228;
    pub const MAP_REVEAL_ALL_PERM: Int = 229;
    pub const MAP_REVEAL_ALL_UNDO_PERM: Int = 230;
    pub const NAMED_SET_REPULSOR: Int = 231;
    pub const TEAM_SET_REPULSOR: Int = 232;
    pub const TEAM_WANDER_IN_PLACE: Int = 233;
    pub const TEAM_INCREASE_PRIORITY: Int = 234;
    pub const TEAM_DECREASE_PRIORITY: Int = 235;
    pub const DISPLAY_COUNTER: Int = 236;
    pub const HIDE_COUNTER: Int = 237;
    pub const TEAM_USE_COMMANDBUTTON_ABILITY_ON_NAMED: Int = 238;
    pub const TEAM_USE_COMMANDBUTTON_ABILITY_AT_WAYPOINT: Int = 239;
    pub const NAMED_USE_COMMANDBUTTON_ABILITY: Int = 240;
    pub const TEAM_USE_COMMANDBUTTON_ABILITY: Int = 241;
    pub const NAMED_FLASH_WHITE: Int = 242;
    pub const TEAM_FLASH_WHITE: Int = 243;
    pub const SKIRMISH_BUILD_BUILDING: Int = 244;
    pub const SKIRMISH_FOLLOW_APPROACH_PATH: Int = 245;
    pub const IDLE_ALL_UNITS: Int = 246;
    pub const RESUME_SUPPLY_TRUCKING: Int = 247;
    pub const NAMED_CUSTOM_COLOR: Int = 248;
    pub const SKIRMISH_MOVE_TO_APPROACH_PATH: Int = 249;
    pub const SKIRMISH_BUILD_BASE_DEFENSE_FRONT: Int = 250;
    pub const SKIRMISH_FIRE_SPECIAL_POWER_AT_MOST_COST: Int = 251;
    pub const NAMED_RECEIVE_UPGRADE: Int = 252;
    pub const PLAYER_REPAIR_NAMED_STRUCTURE: Int = 253;
    pub const SKIRMISH_BUILD_BASE_DEFENSE_FLANK: Int = 254;
    pub const SKIRMISH_BUILD_STRUCTURE_FRONT: Int = 255;
    pub const SKIRMISH_BUILD_STRUCTURE_FLANK: Int = 256;
    pub const SKIRMISH_ATTACK_NEAREST_GROUP_WITH_VALUE: Int = 257;
    pub const SKIRMISH_PERFORM_COMMANDBUTTON_ON_MOST_VALUABLE_OBJECT: Int = 258;
    pub const SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_ALL: Int = 259;
    pub const SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_PARTIAL: Int = 260;
    pub const TEAM_SPIN_FOR_FRAMECOUNT: Int = 261;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NAMED: Int = 262;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_UNIT: Int = 263;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_GARRISONED_BUILDING: Int = 264;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_KINDOF: Int = 265;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_BUILDING: Int = 266;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_BUILDING_CLASS: Int = 267;
    pub const TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_OBJECTTYPE: Int = 268;
    pub const TEAM_PARTIAL_USE_COMMANDBUTTON: Int = 269;
    pub const TEAM_CAPTURE_NEAREST_UNOWNED_FACTION_UNIT: Int = 270;
    pub const PLAYER_CREATE_TEAM_FROM_CAPTURED_UNITS: Int = 271;
    pub const PLAYER_ADD_SKILLPOINTS: Int = 272;
    pub const PLAYER_ADD_RANKLEVEL: Int = 273;
    pub const PLAYER_SET_RANKLEVEL: Int = 274;
    pub const PLAYER_SET_RANKLEVELLIMIT: Int = 275;
    pub const PLAYER_GRANT_SCIENCE: Int = 276;
    pub const PLAYER_PURCHASE_SCIENCE: Int = 277;
    pub const TEAM_HUNT_WITH_COMMAND_BUTTON: Int = 278;
    pub const TEAM_WAIT_FOR_NOT_CONTAINED_ALL: Int = 279;
    pub const TEAM_WAIT_FOR_NOT_CONTAINED_PARTIAL: Int = 280;
    pub const TEAM_FOLLOW_WAYPOINTS_EXACT: Int = 281;
    pub const NAMED_FOLLOW_WAYPOINTS_EXACT: Int = 282;
    pub const TEAM_SET_EMOTICON: Int = 283;
    pub const NAMED_SET_EMOTICON: Int = 284;
    pub const AI_PLAYER_BUILD_SUPPLY_CENTER: Int = 285;
    pub const AI_PLAYER_BUILD_UPGRADE: Int = 286;
    pub const OBJECTLIST_ADDOBJECTTYPE: Int = 287;
    pub const OBJECTLIST_REMOVEOBJECTTYPE: Int = 288;
    pub const MAP_REVEAL_PERMANENTLY_AT_WAYPOINT: Int = 289;
    pub const MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT: Int = 290;
    pub const NAMED_SET_STEALTH_ENABLED: Int = 291;
    pub const TEAM_SET_STEALTH_ENABLED: Int = 292;
    pub const EVA_SET_ENABLED_DISABLED: Int = 293;
    pub const OPTIONS_SET_OCCLUSION_MODE: Int = 294;
    pub const LOCALDEFEAT: Int = 295;
    pub const OPTIONS_SET_DRAWICON_UI_MODE: Int = 296;
    pub const OPTIONS_SET_PARTICLE_CAP_MODE: Int = 297;
    pub const PLAYER_SCIENCE_AVAILABILITY: Int = 298;
    pub const UNIT_AFFECT_OBJECT_PANEL_FLAGS: Int = 299;
    pub const TEAM_AFFECT_OBJECT_PANEL_FLAGS: Int = 300;
    pub const PLAYER_SELECT_SKILLSET: Int = 301;
    pub const SCRIPTING_OVERRIDE_HULK_LIFETIME: Int = 302;
    pub const NAMED_FACE_NAMED: Int = 303;
    pub const NAMED_FACE_WAYPOINT: Int = 304;
    pub const TEAM_FACE_NAMED: Int = 305;
    pub const TEAM_FACE_WAYPOINT: Int = 306;
    pub const COMMANDBAR_REMOVE_BUTTON_OBJECTTYPE: Int = 307;
    pub const COMMANDBAR_ADD_BUTTON_OBJECTTYPE_SLOT: Int = 308;
    pub const UNIT_SPAWN_NAMED_LOCATION_ORIENTATION: Int = 309;
    pub const PLAYER_AFFECT_RECEIVING_EXPERIENCE: Int = 310;
    pub const PLAYER_EXCLUDE_FROM_SCORE_SCREEN: Int = 311;
    pub const TEAM_GUARD_SUPPLY_CENTER: Int = 312;
    pub const ENABLE_SCORING: Int = 313;
    pub const DISABLE_SCORING: Int = 314;
    pub const SOUND_SET_VOLUME: Int = 315;
    pub const SPEECH_SET_VOLUME: Int = 316;
    pub const DISABLE_BORDER_SHROUD: Int = 317;
    pub const ENABLE_BORDER_SHROUD: Int = 318;
    pub const OBJECT_ALLOW_BONUSES: Int = 319;
    pub const SOUND_REMOVE_ALL_DISABLED: Int = 320;
    pub const SOUND_REMOVE_TYPE: Int = 321;
    pub const TEAM_GUARD_IN_TUNNEL_NETWORK: Int = 322;
    pub const QUICKVICTORY: Int = 323;
    pub const SET_INFANTRY_LIGHTING_OVERRIDE: Int = 324;
    pub const RESET_INFANTRY_LIGHTING_OVERRIDE: Int = 325;
    pub const TEAM_DELETE_LIVING: Int = 326;
    pub const RESIZE_VIEW_GUARDBAND: Int = 327;
    pub const DELETE_ALL_UNMANNED: Int = 328;
    pub const CHOOSE_VICTIM_ALWAYS_USES_NORMAL: Int = 329;
    pub const CAMERA_ENABLE_SLAVE_MODE: Int = 330;
    pub const CAMERA_DISABLE_SLAVE_MODE: Int = 331;
    pub const CAMERA_ADD_SHAKER_AT: Int = 332;
    pub const SET_TRAIN_HELD: Int = 333;
    pub const NAMED_SET_EVAC_LEFT_OR_RIGHT: Int = 334;
    pub const ENABLE_OBJECT_SOUND: Int = 335;
    pub const DISABLE_OBJECT_SOUND: Int = 336;
    pub const NAMED_USE_COMMANDBUTTON_ABILITY_USING_WAYPOINT_PATH: Int = 337;
    pub const NAMED_SET_UNMANNED_STATUS: Int = 338;
    pub const TEAM_SET_UNMANNED_STATUS: Int = 339;
    pub const NAMED_SET_BOOBYTRAPPED: Int = 340;
    pub const TEAM_SET_BOOBYTRAPPED: Int = 341;
    pub const SHOW_WEATHER: Int = 342;
    pub const AI_PLAYER_BUILD_TYPE_NEAREST_TEAM: Int = 343;
    pub const NUM_ITEMS: Int = 344;
}

pub const CONDITION_TEMPLATES: &[TemplateRow] = &[
    (0, "CONDITION_FALSE", &[]),
    (1, "COUNTER", &[4, 6, 0]),
    (2, "FLAG", &[5, 8]),
    (3, "CONDITION_TRUE", &[]),
    (4, "TIMER_EXPIRED", &[4]),
    (5, "PLAYER_ALL_DESTROYED", &[11]),
    (6, "PLAYER_ALL_BUILDFACILITIES_DESTROYED", &[11]),
    (7, "TEAM_INSIDE_AREA_PARTIALLY", &[3, 9, 36]),
    (8, "TEAM_DESTROYED", &[3]),
    (9, "CAMERA_MOVEMENT_FINISHED", &[]),
    (10, "TEAM_HAS_UNITS", &[3]),
    (11, "TEAM_STATE_IS", &[3, 18]),
    (12, "TEAM_STATE_IS_NOT", &[3, 18]),
    (13, "NAMED_INSIDE_AREA", &[14, 9]),
    (14, "NAMED_OUTSIDE_AREA", &[14, 9]),
    (15, "NAMED_DESTROYED", &[14]),
    (16, "NAMED_NOT_DESTROYED", &[14]),
    (17, "TEAM_INSIDE_AREA_ENTIRELY", &[3, 9, 36]),
    (18, "TEAM_OUTSIDE_AREA_ENTIRELY", &[3, 9, 36]),
    (19, "NAMED_ATTACKED_BY_OBJECTTYPE", &[14, 15]),
    (20, "TEAM_ATTACKED_BY_OBJECTTYPE", &[3, 15]),
    (21, "NAMED_ATTACKED_BY_PLAYER", &[14, 11]),
    (22, "TEAM_ATTACKED_BY_PLAYER", &[3, 11]),
    (23, "BUILT_BY_PLAYER", &[15, 11]),
    (24, "NAMED_CREATED", &[14]),
    (25, "TEAM_CREATED", &[3]),
    (26, "PLAYER_HAS_CREDITS", &[0, 6, 11]),
    (27, "NAMED_DISCOVERED", &[14, 11]),
    (28, "TEAM_DISCOVERED", &[3, 11]),
    (29, "MISSION_ATTEMPTS", &[11, 6, 0]),
    (30, "NAMED_OWNED_BY_PLAYER", &[14, 11]),
    (31, "TEAM_OWNED_BY_PLAYER", &[3, 11]),
    (32, "PLAYER_HAS_N_OR_FEWER_BUILDINGS", &[11, 0]),
    (33, "PLAYER_HAS_POWER", &[11]),
    (34, "NAMED_REACHED_WAYPOINTS_END", &[14, 24]),
    (35, "TEAM_REACHED_WAYPOINTS_END", &[3, 24]),
    (36, "NAMED_SELECTED", &[14]),
    (37, "NAMED_ENTERED_AREA", &[14, 9]),
    (38, "NAMED_EXITED_AREA", &[14, 9]),
    (39, "TEAM_ENTERED_AREA_ENTIRELY", &[3, 9, 36]),
    (40, "TEAM_ENTERED_AREA_PARTIALLY", &[3, 9, 36]),
    (41, "TEAM_EXITED_AREA_ENTIRELY", &[3, 9, 36]),
    (42, "TEAM_EXITED_AREA_PARTIALLY", &[3, 9, 36]),
    (43, "MULTIPLAYER_ALLIED_VICTORY", &[]),
    (44, "MULTIPLAYER_ALLIED_DEFEAT", &[]),
    (45, "MULTIPLAYER_PLAYER_DEFEAT", &[]),
    (46, "PLAYER_HAS_NO_POWER", &[11]),
    (47, "HAS_FINISHED_VIDEO", &[23]),
    (48, "HAS_FINISHED_SPEECH", &[21]),
    (49, "HAS_FINISHED_AUDIO", &[12]),
    (50, "BUILDING_ENTERED_BY_PLAYER", &[11, 14]),
    (51, "ENEMY_SIGHTED", &[14, 19, 11]),
    (52, "UNIT_HEALTH", &[14, 6, 0]),
    (53, "BRIDGE_REPAIRED", &[26]),
    (54, "BRIDGE_BROKEN", &[26]),
    (55, "NAMED_DYING", &[14]),
    (56, "NAMED_TOTALLY_DEAD", &[14]),
    (57, "PLAYER_HAS_OBJECT_COMPARISON", &[11, 6, 0, 15]),
    (60, "PLAYER_TRIGGERED_SPECIAL_POWER", &[11, 30]),
    (61, "PLAYER_COMPLETED_SPECIAL_POWER", &[11, 30]),
    (62, "PLAYER_MIDWAY_SPECIAL_POWER", &[11, 30]),
    (63, "PLAYER_TRIGGERED_SPECIAL_POWER_FROM_NAMED", &[11, 30, 14]),
    (64, "PLAYER_COMPLETED_SPECIAL_POWER_FROM_NAMED", &[11, 30, 14]),
    (65, "PLAYER_MIDWAY_SPECIAL_POWER_FROM_NAMED", &[11, 30, 14]),
    (68, "PLAYER_BUILT_UPGRADE", &[11, 32]),
    (69, "PLAYER_BUILT_UPGRADE_FROM_NAMED", &[11, 32, 14]),
    (70, "PLAYER_DESTROYED_N_BUILDINGS_PLAYER", &[11, 0, 11]),
    (73, "PLAYER_HAS_COMPARISON_UNIT_TYPE_IN_TRIGGER_AREA", &[11, 6, 0, 15, 9]),
    (74, "PLAYER_HAS_COMPARISON_UNIT_KIND_IN_TRIGGER_AREA", &[11, 6, 0, 27, 9]),
    (75, "UNIT_EMPTIED", &[14]),
    (76, "TYPE_SIGHTED", &[14, 15, 11]),
    (77, "NAMED_BUILDING_IS_EMPTY", &[14]),
    (78, "PLAYER_HAS_N_OR_FEWER_FACTION_BUILDINGS", &[11, 0]),
    (79, "UNIT_HAS_OBJECT_STATUS", &[14, 40]),
    (80, "TEAM_ALL_HAS_OBJECT_STATUS", &[3, 40]),
    (81, "TEAM_SOME_HAVE_OBJECT_STATUS", &[3, 40]),
    (82, "PLAYER_POWER_COMPARE_PERCENT", &[11, 6, 0]),
    (83, "PLAYER_EXCESS_POWER_COMPARE_VALUE", &[11, 6, 0]),
    (84, "SKIRMISH_SPECIAL_POWER_READY", &[11, 30]),
    (85, "SKIRMISH_VALUE_IN_AREA", &[11, 6, 0, 9]),
    (86, "SKIRMISH_PLAYER_FACTION", &[11, 46]),
    (87, "SKIRMISH_SUPPLIES_VALUE_WITHIN_DISTANCE", &[11, 1, 9, 1]),
    (88, "SKIRMISH_TECH_BUILDING_WITHIN_DISTANCE", &[11, 1, 9]),
    (89, "SKIRMISH_COMMAND_BUTTON_READY_ALL", &[11, 3, 41]),
    (90, "SKIRMISH_COMMAND_BUTTON_READY_PARTIAL", &[11, 3, 41]),
    (91, "SKIRMISH_UNOWNED_FACTION_UNIT_EXISTS", &[11, 6, 0]),
    (92, "SKIRMISH_PLAYER_HAS_PREREQUISITE_TO_BUILD", &[11, 15]),
    (93, "SKIRMISH_PLAYER_HAS_COMPARISON_GARRISONED", &[11, 6, 0]),
    (94, "SKIRMISH_PLAYER_HAS_COMPARISON_CAPTURED_UNITS", &[11, 6, 0]),
    (95, "SKIRMISH_NAMED_AREA_EXIST", &[11, 9]),
    (96, "SKIRMISH_PLAYER_HAS_UNITS_IN_AREA", &[11, 9]),
    (97, "SKIRMISH_PLAYER_HAS_BEEN_ATTACKED_BY_PLAYER", &[11, 11]),
    (98, "SKIRMISH_PLAYER_IS_OUTSIDE_AREA", &[11, 9]),
    (99, "SKIRMISH_PLAYER_HAS_DISCOVERED_PLAYER", &[11, 11]),
    (100, "PLAYER_ACQUIRED_SCIENCE", &[11, 31]),
    (101, "PLAYER_HAS_SCIENCEPURCHASEPOINTS", &[11, 0]),
    (102, "PLAYER_CAN_PURCHASE_SCIENCE", &[11, 31]),
    (103, "MUSIC_TRACK_HAS_COMPLETED", &[22, 0]),
    (104, "PLAYER_LOST_OBJECT_TYPE", &[11, 15]),
    (105, "SUPPLY_SOURCE_SAFE", &[11, 0]),
    (106, "SUPPLY_SOURCE_ATTACKED", &[11]),
    (107, "START_POSITION_IS", &[11, 0]),
    (108, "NAMED_HAS_FREE_CONTAINER_SLOTS", &[14]),
];

pub const ACTION_TEMPLATES: &[TemplateRow] = &[
    (0, "DEBUG_MESSAGE_BOX", &[10]),
    (1, "SET_FLAG", &[5, 8]),
    (2, "SET_COUNTER", &[4, 0]),
    (3, "VICTORY", &[]),
    (4, "DEFEAT", &[]),
    (5, "NO_OP", &[]),
    (6, "SET_TIMER", &[4, 0]),
    (7, "PLAY_SOUND_EFFECT", &[12]),
    (8, "ENABLE_SCRIPT", &[2]),
    (9, "DISABLE_SCRIPT", &[2]),
    (10, "CALL_SUBROUTINE", &[13]),
    (11, "PLAY_SOUND_EFFECT_AT", &[12, 7]),
    (12, "DAMAGE_MEMBERS_OF_TEAM", &[3, 1]),
    (13, "MOVE_TEAM_TO", &[3, 7]),
    (14, "MOVE_CAMERA_TO", &[7, 1, 1, 1, 1]),
    (15, "INCREMENT_COUNTER", &[0, 4]),
    (16, "DECREMENT_COUNTER", &[0, 4]),
    (17, "MOVE_CAMERA_ALONG_WAYPOINT_PATH", &[7, 1, 1, 1, 1]),
    (18, "ROTATE_CAMERA", &[1, 1, 1, 1]),
    (19, "RESET_CAMERA", &[7, 1, 1, 1]),
    (20, "SET_MILLISECOND_TIMER", &[4, 1]),
    (21, "CAMERA_MOD_FREEZE_TIME", &[]),
    (22, "SET_VISUAL_SPEED_MULTIPLIER", &[0]),
    (23, "CREATE_OBJECT", &[15, 3, 16, 17]),
    (24, "SUSPEND_BACKGROUND_SOUNDS", &[]),
    (25, "RESUME_BACKGROUND_SOUNDS", &[]),
    (26, "CAMERA_MOD_SET_FINAL_ZOOM", &[1, 51, 51]),
    (27, "CAMERA_MOD_SET_FINAL_PITCH", &[1, 51, 51]),
    (28, "CAMERA_MOD_FREEZE_ANGLE", &[]),
    (29, "CAMERA_MOD_SET_FINAL_SPEED_MULTIPLIER", &[0]),
    (30, "CAMERA_MOD_SET_ROLLING_AVERAGE", &[0]),
    (31, "CAMERA_MOD_FINAL_LOOK_TOWARD", &[7]),
    (32, "CAMERA_MOD_LOOK_TOWARD", &[7]),
    (33, "TEAM_ATTACK_TEAM", &[3, 3]),
    (34, "CREATE_REINFORCEMENT_TEAM", &[3, 7]),
    (35, "MOVE_CAMERA_TO_SELECTION", &[]),
    (36, "TEAM_FOLLOW_WAYPOINTS", &[3, 24, 8]),
    (37, "TEAM_SET_STATE", &[3, 18]),
    (38, "MOVE_NAMED_UNIT_TO", &[14, 7]),
    (39, "NAMED_ATTACK_NAMED", &[14, 14]),
    (40, "CREATE_NAMED_ON_TEAM_AT_WAYPOINT", &[14, 15, 3, 7]),
    (41, "CREATE_UNNAMED_ON_TEAM_AT_WAYPOINT", &[15, 3, 7]),
    (42, "NAMED_APPLY_ATTACK_PRIORITY_SET", &[14, 28]),
    (43, "TEAM_APPLY_ATTACK_PRIORITY_SET", &[3, 28]),
    (44, "SET_BASE_CONSTRUCTION_SPEED", &[11, 0]),
    (45, "NAMED_SET_ATTITUDE", &[14, 20]),
    (46, "TEAM_SET_ATTITUDE", &[3, 20]),
    (47, "NAMED_ATTACK_AREA", &[14, 9]),
    (48, "NAMED_ATTACK_TEAM", &[14, 3]),
    (49, "TEAM_ATTACK_AREA", &[3, 9]),
    (50, "TEAM_ATTACK_NAMED", &[3, 14]),
    (51, "TEAM_LOAD_TRANSPORTS", &[3]),
    (52, "NAMED_ENTER_NAMED", &[14, 14]),
    (53, "TEAM_ENTER_NAMED", &[3, 14]),
    (54, "NAMED_EXIT_ALL", &[14]),
    (55, "TEAM_EXIT_ALL", &[3]),
    (56, "NAMED_FOLLOW_WAYPOINTS", &[14, 24]),
    (57, "NAMED_GUARD", &[14]),
    (58, "TEAM_GUARD", &[3]),
    (59, "NAMED_HUNT", &[14]),
    (60, "TEAM_HUNT", &[3]),
    (61, "PLAYER_SELL_EVERYTHING", &[11]),
    (62, "PLAYER_DISABLE_BASE_CONSTRUCTION", &[11]),
    (63, "PLAYER_DISABLE_FACTORIES", &[11, 15]),
    (64, "PLAYER_DISABLE_UNIT_CONSTRUCTION", &[11]),
    (65, "PLAYER_ENABLE_BASE_CONSTRUCTION", &[11]),
    (66, "PLAYER_ENABLE_FACTORIES", &[11, 15]),
    (67, "PLAYER_ENABLE_UNIT_CONSTRUCTION", &[11]),
    (68, "CAMERA_MOVE_HOME", &[]),
    (69, "BUILD_TEAM", &[3]),
    (70, "NAMED_DAMAGE", &[14, 0]),
    (71, "NAMED_DELETE", &[14]),
    (72, "TEAM_DELETE", &[3]),
    (73, "NAMED_KILL", &[14]),
    (74, "TEAM_KILL", &[3]),
    (75, "PLAYER_KILL", &[11]),
    (76, "DISPLAY_TEXT", &[25]),
    (77, "CAMEO_FLASH", &[38, 0]),
    (78, "NAMED_FLASH", &[14, 0]),
    (79, "TEAM_FLASH", &[3, 0]),
    (80, "MOVIE_PLAY_FULLSCREEN", &[23]),
    (81, "MOVIE_PLAY_RADAR", &[23]),
    (82, "SOUND_PLAY_NAMED", &[10, 14]),
    (83, "SPEECH_PLAY", &[21, 8]),
    (84, "PLAYER_TRANSFER_OWNERSHIP_PLAYER", &[11, 11]),
    (85, "NAMED_TRANSFER_OWNERSHIP_PLAYER", &[14, 11]),
    (86, "PLAYER_RELATES_PLAYER", &[11, 11, 19]),
    (87, "RADAR_CREATE_EVENT", &[16, 29]),
    (88, "RADAR_DISABLE", &[]),
    (89, "RADAR_ENABLE", &[]),
    (90, "MAP_REVEAL_AT_WAYPOINT", &[7, 1, 11]),
    (91, "TEAM_AVAILABLE_FOR_RECRUITMENT", &[3, 8]),
    (92, "TEAM_COLLECT_NEARBY_FOR_TEAM", &[3]),
    (93, "TEAM_MERGE_INTO_TEAM", &[3, 3]),
    (94, "DISABLE_INPUT", &[]),
    (95, "ENABLE_INPUT", &[]),
    (96, "PLAYER_HUNT", &[11]),
    (97, "SOUND_AMBIENT_PAUSE", &[]),
    (98, "SOUND_AMBIENT_RESUME", &[]),
    (99, "MUSIC_SET_TRACK", &[22, 8, 8]),
    (100, "SET_TREE_SWAY", &[17, 17, 17, 0, 1]),
    (101, "DEBUG_STRING", &[10]),
    (102, "MAP_REVEAL_ALL", &[11]),
    (103, "TEAM_GARRISON_SPECIFIC_BUILDING", &[3, 14]),
    (104, "EXIT_SPECIFIC_BUILDING", &[14]),
    (105, "TEAM_GARRISON_NEAREST_BUILDING", &[3]),
    (106, "TEAM_EXIT_ALL_BUILDINGS", &[3]),
    (107, "NAMED_GARRISON_SPECIFIC_BUILDING", &[14, 14]),
    (108, "NAMED_GARRISON_NEAREST_BUILDING", &[14]),
    (109, "NAMED_EXIT_BUILDING", &[14]),
    (110, "PLAYER_GARRISON_ALL_BUILDINGS", &[11]),
    (111, "PLAYER_EXIT_ALL_BUILDINGS", &[11]),
    (112, "TEAM_WANDER", &[3, 24]),
    (113, "TEAM_PANIC", &[3, 24]),
    (114, "SETUP_CAMERA", &[7, 1, 1, 7]),
    (115, "CAMERA_LETTERBOX_BEGIN", &[]),
    (116, "CAMERA_LETTERBOX_END", &[]),
    (117, "ZOOM_CAMERA", &[1, 1, 1, 1]),
    (118, "PITCH_CAMERA", &[1, 1, 1, 1]),
    (119, "CAMERA_FOLLOW_NAMED", &[14, 8]),
    (120, "OVERSIZE_TERRAIN", &[0]),
    (121, "CAMERA_FADE_ADD", &[1, 1, 0, 0, 0]),
    (122, "CAMERA_FADE_SUBTRACT", &[1, 1, 0, 0, 0]),
    (123, "CAMERA_FADE_SATURATE", &[1, 1, 0, 0, 0]),
    (124, "CAMERA_FADE_MULTIPLY", &[1, 1, 0, 0, 0]),
    (125, "CAMERA_BW_MODE_BEGIN", &[0]),
    (126, "CAMERA_BW_MODE_END", &[0]),
    (127, "DRAW_SKYBOX_BEGIN", &[]),
    (128, "DRAW_SKYBOX_END", &[]),
    (129, "SET_ATTACK_PRIORITY_THING", &[28, 15, 0]),
    (130, "SET_ATTACK_PRIORITY_KIND_OF", &[28, 27, 0]),
    (131, "SET_DEFAULT_ATTACK_PRIORITY", &[28, 0]),
    (132, "CAMERA_STOP_FOLLOW", &[]),
    (133, "CAMERA_MOTION_BLUR", &[8, 8]),
    (134, "CAMERA_MOTION_BLUR_JUMP", &[7, 8]),
    (135, "CAMERA_MOTION_BLUR_FOLLOW", &[0]),
    (136, "CAMERA_MOTION_BLUR_END_FOLLOW", &[]),
    (137, "FREEZE_TIME", &[]),
    (138, "UNFREEZE_TIME", &[]),
    (139, "SHOW_MILITARY_CAPTION", &[10, 0]),
    (140, "CAMERA_SET_AUDIBLE_DISTANCE", &[1]),
    (141, "SET_STOPPING_DISTANCE", &[3, 1]),
    (142, "NAMED_SET_STOPPING_DISTANCE", &[14, 1]),
    (143, "SET_FPS_LIMIT", &[0]),
    (144, "MUSIC_SET_VOLUME", &[1]),
    (145, "MAP_SHROUD_AT_WAYPOINT", &[7, 1, 11]),
    (146, "MAP_SHROUD_ALL", &[11]),
    (147, "SET_RANDOM_TIMER", &[4, 0, 0]),
    (148, "SET_RANDOM_MSEC_TIMER", &[4, 1, 1]),
    (149, "STOP_TIMER", &[4]),
    (150, "RESTART_TIMER", &[4]),
    (151, "ADD_TO_MSEC_TIMER", &[1, 4]),
    (152, "SUB_FROM_MSEC_TIMER", &[1, 4]),
    (153, "TEAM_TRANSFER_TO_PLAYER", &[3, 11]),
    (154, "PLAYER_SET_MONEY", &[11, 0]),
    (155, "PLAYER_GIVE_MONEY", &[11, 0]),
    (156, "DISABLE_SPECIAL_POWER_DISPLAY", &[]),
    (157, "ENABLE_SPECIAL_POWER_DISPLAY", &[]),
    (158, "NAMED_HIDE_SPECIAL_POWER_DISPLAY", &[14]),
    (159, "NAMED_SHOW_SPECIAL_POWER_DISPLAY", &[14]),
    (160, "DISPLAY_COUNTDOWN_TIMER", &[4, 25]),
    (161, "HIDE_COUNTDOWN_TIMER", &[4]),
    (162, "ENABLE_COUNTDOWN_TIMER_DISPLAY", &[]),
    (163, "DISABLE_COUNTDOWN_TIMER_DISPLAY", &[]),
    (164, "NAMED_STOP_SPECIAL_POWER_COUNTDOWN", &[14, 30]),
    (165, "NAMED_START_SPECIAL_POWER_COUNTDOWN", &[14, 30]),
    (166, "NAMED_SET_SPECIAL_POWER_COUNTDOWN", &[14, 30, 0]),
    (167, "NAMED_ADD_SPECIAL_POWER_COUNTDOWN", &[14, 30, 0]),
    (168, "NAMED_FIRE_SPECIAL_POWER_AT_WAYPOINT", &[14, 30, 7]),
    (169, "NAMED_FIRE_SPECIAL_POWER_AT_NAMED", &[14, 30, 14]),
    (170, "REFRESH_RADAR", &[]),
    (171, "CAMERA_TETHER_NAMED", &[14, 8, 1]),
    (172, "CAMERA_STOP_TETHER_NAMED", &[]),
    (173, "CAMERA_SET_DEFAULT", &[1, 1, 1]),
    (174, "NAMED_STOP", &[14]),
    (175, "TEAM_STOP", &[3]),
    (176, "TEAM_STOP_AND_DISBAND", &[3]),
    (177, "RECRUIT_TEAM", &[3, 1]),
    (178, "TEAM_SET_OVERRIDE_RELATION_TO_TEAM", &[3, 3, 19]),
    (179, "TEAM_REMOVE_OVERRIDE_RELATION_TO_TEAM", &[3, 3]),
    (180, "TEAM_REMOVE_ALL_OVERRIDE_RELATIONS", &[3]),
    (181, "CAMERA_LOOK_TOWARD_OBJECT", &[14, 1, 1, 1, 1]),
    (182, "NAMED_FIRE_WEAPON_FOLLOWING_WAYPOINT_PATH", &[14, 24]),
    (183, "TEAM_SET_OVERRIDE_RELATION_TO_PLAYER", &[3, 11, 19]),
    (184, "TEAM_REMOVE_OVERRIDE_RELATION_TO_PLAYER", &[3, 11]),
    (185, "PLAYER_SET_OVERRIDE_RELATION_TO_TEAM", &[11, 3, 19]),
    (186, "PLAYER_REMOVE_OVERRIDE_RELATION_TO_TEAM", &[11, 3]),
    (187, "UNIT_EXECUTE_SEQUENTIAL_SCRIPT", &[14, 2]),
    (188, "UNIT_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING", &[14, 2, 0]),
    (189, "UNIT_STOP_SEQUENTIAL_SCRIPT", &[14]),
    (190, "TEAM_EXECUTE_SEQUENTIAL_SCRIPT", &[3, 2]),
    (191, "TEAM_EXECUTE_SEQUENTIAL_SCRIPT_LOOPING", &[3, 2, 0]),
    (192, "TEAM_STOP_SEQUENTIAL_SCRIPT", &[3]),
    (193, "UNIT_GUARD_FOR_FRAMECOUNT", &[14, 0]),
    (194, "UNIT_IDLE_FOR_FRAMECOUNT", &[14, 0]),
    (195, "TEAM_GUARD_FOR_FRAMECOUNT", &[3, 0]),
    (196, "TEAM_IDLE_FOR_FRAMECOUNT", &[3, 0]),
    (197, "WATER_CHANGE_HEIGHT", &[9, 1]),
    (198, "NAMED_USE_COMMANDBUTTON_ABILITY_ON_NAMED", &[14, 33, 14]),
    (199, "NAMED_USE_COMMANDBUTTON_ABILITY_AT_WAYPOINT", &[14, 33, 7]),
    (200, "WATER_CHANGE_HEIGHT_OVER_TIME", &[9, 1, 1, 1]),
    (201, "MAP_SWITCH_BORDER", &[34]),
    (202, "TEAM_GUARD_POSITION", &[3, 7]),
    (203, "TEAM_GUARD_OBJECT", &[3, 14]),
    (204, "TEAM_GUARD_AREA", &[3, 9]),
    (205, "OBJECT_FORCE_SELECT", &[3, 15, 8, 21]),
    (206, "CAMERA_LOOK_TOWARD_WAYPOINT", &[7, 1, 1, 1, 8]),
    (207, "UNIT_DESTROY_ALL_CONTAINED", &[14]),
    (208, "RADAR_FORCE_ENABLE", &[]),
    (209, "RADAR_REVERT_TO_NORMAL", &[]),
    (210, "SCREEN_SHAKE", &[37]),
    (211, "TECHTREE_MODIFY_BUILDABILITY_OBJECT", &[15, 35]),
    (212, "WAREHOUSE_SET_VALUE", &[14, 0]),
    (213, "OBJECT_CREATE_RADAR_EVENT", &[14, 29]),
    (214, "TEAM_CREATE_RADAR_EVENT", &[3, 29]),
    (215, "DISPLAY_CINEMATIC_TEXT", &[25, 39, 0]),
    (216, "DEBUG_CRASH_BOX", &[10]),
    (217, "SOUND_DISABLE_TYPE", &[12]),
    (218, "SOUND_ENABLE_TYPE", &[12]),
    (219, "SOUND_ENABLE_ALL", &[]),
    (220, "AUDIO_OVERRIDE_VOLUME_TYPE", &[12, 1]),
    (221, "AUDIO_RESTORE_VOLUME_TYPE", &[12]),
    (222, "AUDIO_RESTORE_VOLUME_ALL_TYPE", &[]),
    (223, "INGAME_POPUP_MESSAGE", &[25, 0, 0, 0, 8]),
    (224, "SET_CAVE_INDEX", &[14, 0]),
    (225, "NAMED_SET_HELD", &[14, 8]),
    (226, "NAMED_SET_TOPPLE_DIRECTION", &[14, 16]),
    (227, "UNIT_MOVE_TOWARDS_NEAREST_OBJECT_TYPE", &[14, 15, 9]),
    (228, "TEAM_MOVE_TOWARDS_NEAREST_OBJECT_TYPE", &[3, 15, 9]),
    (229, "MAP_REVEAL_ALL_PERM", &[11]),
    (230, "MAP_REVEAL_ALL_UNDO_PERM", &[11]),
    (231, "NAMED_SET_REPULSOR", &[14, 8]),
    (232, "TEAM_SET_REPULSOR", &[3, 8]),
    (233, "TEAM_WANDER_IN_PLACE", &[3]),
    (234, "TEAM_INCREASE_PRIORITY", &[3]),
    (235, "TEAM_DECREASE_PRIORITY", &[3]),
    (236, "DISPLAY_COUNTER", &[4, 25]),
    (237, "HIDE_COUNTER", &[4]),
    (238, "TEAM_USE_COMMANDBUTTON_ABILITY_ON_NAMED", &[3, 41, 14]),
    (239, "TEAM_USE_COMMANDBUTTON_ABILITY_AT_WAYPOINT", &[3, 41, 7]),
    (240, "NAMED_USE_COMMANDBUTTON_ABILITY", &[14, 33]),
    (241, "TEAM_USE_COMMANDBUTTON_ABILITY", &[3, 41]),
    (242, "NAMED_FLASH_WHITE", &[14, 0]),
    (243, "TEAM_FLASH_WHITE", &[3, 0]),
    (244, "SKIRMISH_BUILD_BUILDING", &[15]),
    (245, "SKIRMISH_FOLLOW_APPROACH_PATH", &[3, 42, 8]),
    (246, "IDLE_ALL_UNITS", &[]),
    (247, "RESUME_SUPPLY_TRUCKING", &[]),
    (248, "NAMED_CUSTOM_COLOR", &[14, 43]),
    (249, "SKIRMISH_MOVE_TO_APPROACH_PATH", &[3, 42]),
    (250, "SKIRMISH_BUILD_BASE_DEFENSE_FRONT", &[]),
    (251, "SKIRMISH_FIRE_SPECIAL_POWER_AT_MOST_COST", &[11, 30]),
    (253, "PLAYER_REPAIR_NAMED_STRUCTURE", &[11, 14]),
    (254, "SKIRMISH_BUILD_BASE_DEFENSE_FLANK", &[]),
    (255, "SKIRMISH_BUILD_STRUCTURE_FRONT", &[15]),
    (256, "SKIRMISH_BUILD_STRUCTURE_FLANK", &[15]),
    (257, "SKIRMISH_ATTACK_NEAREST_GROUP_WITH_VALUE", &[3, 6, 0]),
    (258, "SKIRMISH_PERFORM_COMMANDBUTTON_ON_MOST_VALUABLE_OBJECT", &[3, 41, 1, 8]),
    (259, "SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_ALL", &[11, 3, 41]),
    (260, "SKIRMISH_WAIT_FOR_COMMANDBUTTON_AVAILABLE_PARTIAL", &[11, 3, 41]),
    (261, "TEAM_SPIN_FOR_FRAMECOUNT", &[3, 0]),
    (262, "TEAM_ALL_USE_COMMANDBUTTON_ON_NAMED", &[3, 41]),
    (263, "TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_UNIT", &[3, 41]),
    (264, "TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_GARRISONED_BUILDING", &[3, 41]),
    (265, "TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_KINDOF", &[3, 41, 27]),
    (266, "TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_BUILDING", &[3, 41]),
    (267, "TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_ENEMY_BUILDING_CLASS", &[3, 41, 27]),
    (268, "TEAM_ALL_USE_COMMANDBUTTON_ON_NEAREST_OBJECTTYPE", &[3, 41, 15]),
    (269, "TEAM_PARTIAL_USE_COMMANDBUTTON", &[1, 3, 41]),
    (270, "TEAM_CAPTURE_NEAREST_UNOWNED_FACTION_UNIT", &[3]),
    (271, "PLAYER_CREATE_TEAM_FROM_CAPTURED_UNITS", &[11, 3]),
    (272, "PLAYER_ADD_SKILLPOINTS", &[11, 0]),
    (273, "PLAYER_ADD_RANKLEVEL", &[11, 0]),
    (274, "PLAYER_SET_RANKLEVEL", &[11, 0]),
    (275, "PLAYER_SET_RANKLEVELLIMIT", &[0]),
    (276, "PLAYER_GRANT_SCIENCE", &[11, 31]),
    (277, "PLAYER_PURCHASE_SCIENCE", &[11, 31]),
    (278, "TEAM_HUNT_WITH_COMMAND_BUTTON", &[3, 41]),
    (279, "TEAM_WAIT_FOR_NOT_CONTAINED_ALL", &[3]),
    (280, "TEAM_WAIT_FOR_NOT_CONTAINED_PARTIAL", &[3]),
    (281, "TEAM_FOLLOW_WAYPOINTS_EXACT", &[3, 24, 8]),
    (282, "NAMED_FOLLOW_WAYPOINTS_EXACT", &[14, 24]),
    (283, "TEAM_SET_EMOTICON", &[3, 44, 1]),
    (284, "NAMED_SET_EMOTICON", &[14, 44, 1]),
    (285, "AI_PLAYER_BUILD_SUPPLY_CENTER", &[11, 15, 0]),
    (286, "AI_PLAYER_BUILD_UPGRADE", &[11, 32]),
    (287, "OBJECTLIST_ADDOBJECTTYPE", &[47, 15]),
    (288, "OBJECTLIST_REMOVEOBJECTTYPE", &[47, 15]),
    (289, "MAP_REVEAL_PERMANENTLY_AT_WAYPOINT", &[7, 1, 11, 48]),
    (290, "MAP_UNDO_REVEAL_PERMANENTLY_AT_WAYPOINT", &[48]),
    (291, "NAMED_SET_STEALTH_ENABLED", &[14, 8]),
    (292, "TEAM_SET_STEALTH_ENABLED", &[3, 8]),
    (293, "EVA_SET_ENABLED_DISABLED", &[8]),
    (294, "OPTIONS_SET_OCCLUSION_MODE", &[8]),
    (295, "LOCALDEFEAT", &[]),
    (296, "OPTIONS_SET_DRAWICON_UI_MODE", &[8]),
    (297, "OPTIONS_SET_PARTICLE_CAP_MODE", &[8]),
    (298, "PLAYER_SCIENCE_AVAILABILITY", &[11, 31, 49]),
    (299, "UNIT_AFFECT_OBJECT_PANEL_FLAGS", &[14, 45, 8]),
    (300, "TEAM_AFFECT_OBJECT_PANEL_FLAGS", &[3, 45, 8]),
    (301, "PLAYER_SELECT_SKILLSET", &[11, 0]),
    (302, "SCRIPTING_OVERRIDE_HULK_LIFETIME", &[1]),
    (303, "NAMED_FACE_NAMED", &[14, 14]),
    (304, "NAMED_FACE_WAYPOINT", &[14, 7]),
    (305, "TEAM_FACE_NAMED", &[3, 14]),
    (306, "TEAM_FACE_WAYPOINT", &[3, 7]),
    (307, "COMMANDBAR_REMOVE_BUTTON_OBJECTTYPE", &[38, 15]),
    (308, "COMMANDBAR_ADD_BUTTON_OBJECTTYPE_SLOT", &[38, 15, 0]),
    (309, "UNIT_SPAWN_NAMED_LOCATION_ORIENTATION", &[14, 15, 3, 16, 17]),
    (310, "PLAYER_AFFECT_RECEIVING_EXPERIENCE", &[11, 1]),
    (311, "PLAYER_EXCLUDE_FROM_SCORE_SCREEN", &[11]),
    (312, "TEAM_GUARD_SUPPLY_CENTER", &[3, 0]),
    (313, "ENABLE_SCORING", &[]),
    (314, "DISABLE_SCORING", &[]),
    (315, "SOUND_SET_VOLUME", &[1]),
    (316, "SPEECH_SET_VOLUME", &[1]),
    (317, "DISABLE_BORDER_SHROUD", &[]),
    (318, "ENABLE_BORDER_SHROUD", &[]),
    (319, "OBJECT_ALLOW_BONUSES", &[8]),
    (320, "SOUND_REMOVE_ALL_DISABLED", &[]),
    (321, "SOUND_REMOVE_TYPE", &[12]),
    (322, "TEAM_GUARD_IN_TUNNEL_NETWORK", &[3]),
    (323, "QUICKVICTORY", &[]),
    (324, "SET_INFANTRY_LIGHTING_OVERRIDE", &[1]),
    (325, "RESET_INFANTRY_LIGHTING_OVERRIDE", &[]),
    (326, "TEAM_DELETE_LIVING", &[3]),
    (327, "RESIZE_VIEW_GUARDBAND", &[1, 1]),
    (328, "DELETE_ALL_UNMANNED", &[]),
    (329, "CHOOSE_VICTIM_ALWAYS_USES_NORMAL", &[8]),
    (330, "CAMERA_ENABLE_SLAVE_MODE", &[10, 10]),
    (331, "CAMERA_DISABLE_SLAVE_MODE", &[]),
    (332, "CAMERA_ADD_SHAKER_AT", &[7, 1, 1, 1]),
    (333, "SET_TRAIN_HELD", &[14, 8]),
    (334, "NAMED_SET_EVAC_LEFT_OR_RIGHT", &[14, 50]),
    (335, "ENABLE_OBJECT_SOUND", &[14]),
    (336, "DISABLE_OBJECT_SOUND", &[14]),
    (337, "NAMED_USE_COMMANDBUTTON_ABILITY_USING_WAYPOINT_PATH", &[14, 33, 24]),
    (338, "NAMED_SET_UNMANNED_STATUS", &[14]),
    (339, "TEAM_SET_UNMANNED_STATUS", &[3]),
    (340, "NAMED_SET_BOOBYTRAPPED", &[15, 14]),
    (341, "TEAM_SET_BOOBYTRAPPED", &[15, 3]),
    (342, "SHOW_WEATHER", &[8]),
    (343, "AI_PLAYER_BUILD_TYPE_NEAREST_TEAM", &[11, 15, 3]),
];
