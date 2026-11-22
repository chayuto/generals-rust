//! `PKT-442`: Global game configuration container, `GameData.ini` parser, override stack,
//! `OptionPreferences` overrides, and `exe`/`.scb` CRC computation
//! (`GeneralsMD/Code/GameEngine/Source/Common/GlobalData.cpp`).
//!
//! Reference C++ packet: `packets/phase7/PKT-442_GlobalData.md`
//! Golden Vector test: `crates/zh_logic/tests/golden_pkt442_global_data.rs` (`docs/golden_vectors/PKT-442.json`)

#![allow(unused_variables, dead_code, unused_imports)]

use crate::economy::money::Money;
use crate::rules::combat_types::WeaponBonusConditionFlags;
use zh_common::crc::Crc;
use zh_common::ini::{Color, IniError, IniLoadType, IniReader, LOGICFRAMES_PER_SECOND};
use zh_common::math::{Coord2D, Coord3D, RGBColor};
use zh_common::types::{
    Bool, Int, Real, Short, UnsignedByte, UnsignedInt, UnsignedShort, FALSE, TRUE,
};

pub const MAX_GLOBAL_LIGHTS: usize = 3;
pub const MAX_WATER_GRID_SETTINGS: usize = 4;
pub const SIMULATE_REPLAYS_SEQUENTIAL: Int = -1;

pub const DEFAULT_DISPLAY_WIDTH: Int = 800;
pub const DEFAULT_DISPLAY_HEIGHT: Int = 600;
pub const DEFAULT_JPEG_QUALITY: Int = 80;
pub const DEFAULT_DOUBLE_CLICK_TIME_MS: UnsignedInt = 500;

pub const PLAYERTYPE_COUNT: usize = 2;
pub const DIFFICULTY_COUNT: usize = 3;
pub const LEVEL_COUNT: usize = 4;

pub const MULTISAMPLE_MODE_NONE: UnsignedInt = 0;
pub const TEXTURE_FILTER_BILINEAR: UnsignedInt = 2;
pub const TEXTURE_FILTER_ANISOTROPIC_2X: UnsignedInt = 2;
pub const FIREWALL_TYPE_UNKNOWN: UnsignedInt = 0;

pub const TERRAIN_LOD_COUNT: usize = 7;
pub const TIME_OF_DAY_COUNT: usize = 5;
pub const TIME_OF_DAY_FIRST: Int = 1;
pub const WEATHER_COUNT: usize = 2;
pub const BODY_DAMAGE_TYPE_COUNT: usize = 4;
pub const WEAPON_BONUS_CONDITION_COUNT: usize = 27;
pub const WEAPON_BONUS_FIELD_COUNT: usize = 5;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TerrainLod {
    Invalid = 0,
    Min = 1,
    NoClouds = 2,
    NoWater = 3,
    Max = 4,
    #[default]
    Automatic = 5,
    Disable = 6,
}

impl TerrainLod {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::Min),
            2 => Some(Self::NoClouds),
            3 => Some(Self::NoWater),
            4 => Some(Self::Max),
            5 => Some(Self::Automatic),
            6 => Some(Self::Disable),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const TERRAIN_LOD_NAMES: [&str; TERRAIN_LOD_COUNT] = [
    "NONE",
    "MIN",
    "NO_CLOUDS",
    "NO_WATER",
    "MAX",
    "AUTOMATIC",
    "DISABLE",
];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum TimeOfDay {
    Invalid = 0,
    Morning = 1,
    #[default]
    Afternoon = 2,
    Evening = 3,
    Night = 4,
}

impl TimeOfDay {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::Morning),
            2 => Some(Self::Afternoon),
            3 => Some(Self::Evening),
            4 => Some(Self::Night),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const TIME_OF_DAY_NAMES: [&str; TIME_OF_DAY_COUNT] =
    ["NONE", "MORNING", "AFTERNOON", "EVENING", "NIGHT"];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum Weather {
    #[default]
    Normal = 0,
    Snowy = 1,
}

impl Weather {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Normal),
            1 => Some(Self::Snowy),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const WEATHER_NAMES: [&str; WEATHER_COUNT] = ["NORMAL", "SNOWY"];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum BodyDamageType {
    Pristine = 0,
    Damaged = 1,
    #[default]
    ReallyDamaged = 2,
    Rubble = 3,
}

impl BodyDamageType {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Pristine),
            1 => Some(Self::Damaged),
            2 => Some(Self::ReallyDamaged),
            3 => Some(Self::Rubble),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const THE_BODY_DAMAGE_TYPE_NAMES: [&str; BODY_DAMAGE_TYPE_COUNT] =
    ["PRISTINE", "DAMAGED", "REALLYDAMAGED", "RUBBLE"];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum AiDebugOptions {
    #[default]
    None = 0,
    Paths = 1,
    Terrain = 2,
    Cells = 3,
    GroundPaths = 4,
    Zones = 5,
    End = 6,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum PlayerType {
    #[default]
    Human = 0,
    Computer = 1,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum GameDifficulty {
    #[default]
    Easy = 0,
    Normal = 1,
    Hard = 2,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum WeaponBonusConditionType {
    #[default]
    Garrisoned = 0,
    Horde = 1,
    ContinuousFireMean = 2,
    ContinuousFireFast = 3,
    Nationalism = 4,
    PlayerUpgrade = 5,
    DroneSpotting = 6,
    DemoralizedObsolete = 7,
    Enthusiastic = 8,
    Veteran = 9,
    Elite = 10,
    Hero = 11,
    BattleplanBombardment = 12,
    BattleplanHoldTheLine = 13,
    BattleplanSearchAndDestroy = 14,
    Subliminal = 15,
    SoloHumanEasy = 16,
    SoloHumanNormal = 17,
    SoloHumanHard = 18,
    SoloAiEasy = 19,
    SoloAiNormal = 20,
    SoloAiHard = 21,
    TargetFaerieFire = 22,
    Fanaticism = 23,
    FrenzyOne = 24,
    FrenzyTwo = 25,
    FrenzyThree = 26,
}

impl WeaponBonusConditionType {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Garrisoned),
            1 => Some(Self::Horde),
            2 => Some(Self::ContinuousFireMean),
            3 => Some(Self::ContinuousFireFast),
            4 => Some(Self::Nationalism),
            5 => Some(Self::PlayerUpgrade),
            6 => Some(Self::DroneSpotting),
            7 => Some(Self::DemoralizedObsolete),
            8 => Some(Self::Enthusiastic),
            9 => Some(Self::Veteran),
            10 => Some(Self::Elite),
            11 => Some(Self::Hero),
            12 => Some(Self::BattleplanBombardment),
            13 => Some(Self::BattleplanHoldTheLine),
            14 => Some(Self::BattleplanSearchAndDestroy),
            15 => Some(Self::Subliminal),
            16 => Some(Self::SoloHumanEasy),
            17 => Some(Self::SoloHumanNormal),
            18 => Some(Self::SoloHumanHard),
            19 => Some(Self::SoloAiEasy),
            20 => Some(Self::SoloAiNormal),
            21 => Some(Self::SoloAiHard),
            22 => Some(Self::TargetFaerieFire),
            23 => Some(Self::Fanaticism),
            24 => Some(Self::FrenzyOne),
            25 => Some(Self::FrenzyTwo),
            26 => Some(Self::FrenzyThree),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const THE_WEAPON_BONUS_NAMES: [&str; WEAPON_BONUS_CONDITION_COUNT] = [
    "GARRISONED",
    "HORDE",
    "CONTINUOUS_FIRE_MEAN",
    "CONTINUOUS_FIRE_FAST",
    "NATIONALISM",
    "PLAYER_UPGRADE",
    "DRONE_SPOTTING",
    "DEMORALIZED_OBSOLETE",
    "ENTHUSIASTIC",
    "VETERAN",
    "ELITE",
    "HERO",
    "BATTLEPLAN_BOMBARDMENT",
    "BATTLEPLAN_HOLDTHELINE",
    "BATTLEPLAN_SEARCHANDDESTROY",
    "SUBLIMINAL",
    "SOLO_HUMAN_EASY",
    "SOLO_HUMAN_NORMAL",
    "SOLO_HUMAN_HARD",
    "SOLO_AI_EASY",
    "SOLO_AI_NORMAL",
    "SOLO_AI_HARD",
    "TARGET_FAERIE_FIRE",
    "FANATICISM",
    "FRENZY_ONE",
    "FRENZY_TWO",
    "FRENZY_THREE",
];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum WeaponBonusField {
    #[default]
    Damage = 0,
    Radius = 1,
    Range = 2,
    RateOfFire = 3,
    PreAttack = 4,
}

impl WeaponBonusField {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Damage),
            1 => Some(Self::Radius),
            2 => Some(Self::Range),
            3 => Some(Self::RateOfFire),
            4 => Some(Self::PreAttack),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const THE_WEAPON_BONUS_FIELD_NAMES: [&str; WEAPON_BONUS_FIELD_COUNT] =
    ["DAMAGE", "RADIUS", "RANGE", "RATE_OF_FIRE", "PRE_ATTACK"];

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WeaponBonus {
    pub fields: [Real; WEAPON_BONUS_FIELD_COUNT],
}

impl Default for WeaponBonus {
    fn default() -> Self {
        Self::new()
    }
}

impl WeaponBonus {
    pub const fn new() -> Self {
        Self {
            fields: [1.0; WEAPON_BONUS_FIELD_COUNT],
        }
    }

    pub fn clear(&mut self) {
        self.fields = [1.0; WEAPON_BONUS_FIELD_COUNT];
    }

    #[inline]
    pub fn get_field(&self, f: WeaponBonusField) -> Real {
        self.fields[f.as_i32() as usize]
    }

    #[inline]
    pub fn set_field(&mut self, f: WeaponBonusField, v: Real) {
        self.fields[f.as_i32() as usize] = v;
    }

    pub fn append_bonuses(&self, bonus: &mut WeaponBonus) {
        todo!("PKT-442: implement WeaponBonus::append_bonuses matching Weapon.cpp:3581-3587")
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WeaponBonusSet {
    pub entries: [WeaponBonus; WEAPON_BONUS_CONDITION_COUNT],
}

impl Default for WeaponBonusSet {
    fn default() -> Self {
        Self::new()
    }
}

impl WeaponBonusSet {
    pub const fn new() -> Self {
        Self {
            entries: [WeaponBonus::new(); WEAPON_BONUS_CONDITION_COUNT],
        }
    }

    pub fn append_bonuses(&self, flags: WeaponBonusConditionFlags, bonus: &mut WeaponBonus) {
        todo!("PKT-442: implement WeaponBonusSet::append_bonuses matching Weapon.cpp:3612-3624")
    }

    pub fn parse_weapon_bonus_set(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        todo!("PKT-442: implement WeaponBonusSet::parse_weapon_bonus_set matching Weapon.cpp:3604-3609")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TerrainLighting {
    pub ambient: RGBColor,
    pub diffuse: RGBColor,
    pub light_pos: Coord3D,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandLineData {
    pub has_parsed_command_line_for_startup: Bool,
    pub has_parsed_command_line_for_engine_init: Bool,
    pub parsed_arguments: Vec<Bool>,
}

/// Option preferences override values applied at the end of `GlobalData::parse_game_data_definition`.
#[derive(Debug, Clone, PartialEq)]
pub struct OptionPreferencesOverrides {
    pub use_alternate_mouse: Bool,
    pub use_right_mouse_scroll_with_alternate_mouse: Bool,
    pub client_retaliation_mode_enabled: Bool,
    pub double_click_attack_move: Bool,
    pub jpeg_quality: Int,
    pub keyboard_scroll_factor: Real,
    pub draw_scroll_anchor: Bool,
    pub move_scroll_anchor: Bool,
    pub default_ip: UnsignedInt,
    pub firewall_behavior: Int,
    pub firewall_port_allocation_delta: Short,
    pub firewall_port_override: UnsignedShort,
    pub save_camera_in_replay: Bool,
    pub use_camera_in_replay: Bool,
    pub enable_player_observer: Bool,
    pub network_latency_font_size: Int,
    pub render_fps_font_size: Int,
    pub system_time_font_size: Int,
    pub game_time_font_size: Int,
    pub player_info_list_font_size: Int,
    pub show_money_per_minute: Bool,
    pub game_window_transition_speed_multiplier: Real,
    pub max_camera_height: Option<Real>,
    pub terrain_draw_distance_scale: Option<Real>,
    pub skirmish_tick_rate: Int,
    pub advanced_shadows: Bool,
    pub advanced_lighting: Bool,
    pub use_shadow_volumes_for_skins: Bool,
    pub extended_shadow_frustum_culling: Bool,
    pub natural_sun_elevation: Bool,
    pub shadows_for_projectiles: Bool,
    pub shadows_for_props: Bool,
    pub bridge_shadows: Bool,
    pub water_reflections: Bool,
    pub explosion_dynamic_lights: Bool,
    pub vehicle_headlights: Bool,
    pub enable_post_processing: Bool,
    pub post_process_bloom: Bool,
    pub post_process_hdr: Bool,
    pub post_process_fxaa: Bool,
    pub air_unit_ground_indicator: Bool,
    pub anti_aliasing: UnsignedInt,
    pub texture_filter_mode: UnsignedInt,
    pub texture_anisotropy_level: UnsignedInt,
    pub gamma_value: Real,
    pub x_resolution: Int,
    pub y_resolution: Int,
}

impl Default for OptionPreferencesOverrides {
    fn default() -> Self {
        Self {
            use_alternate_mouse: FALSE,
            use_right_mouse_scroll_with_alternate_mouse: TRUE,
            client_retaliation_mode_enabled: TRUE,
            double_click_attack_move: FALSE,
            jpeg_quality: DEFAULT_JPEG_QUALITY,
            keyboard_scroll_factor: 0.5,
            draw_scroll_anchor: FALSE,
            move_scroll_anchor: TRUE,
            default_ip: 0,
            firewall_behavior: FIREWALL_TYPE_UNKNOWN as Int,
            firewall_port_allocation_delta: 0,
            firewall_port_override: 0,
            save_camera_in_replay: TRUE,
            use_camera_in_replay: TRUE,
            enable_player_observer: TRUE,
            network_latency_font_size: 8,
            render_fps_font_size: 8,
            system_time_font_size: 8,
            game_time_font_size: 8,
            player_info_list_font_size: 8,
            show_money_per_minute: FALSE,
            game_window_transition_speed_multiplier: 1.0,
            max_camera_height: None,
            terrain_draw_distance_scale: None,
            skirmish_tick_rate: LOGICFRAMES_PER_SECOND,
            advanced_shadows: TRUE,
            advanced_lighting: TRUE,
            use_shadow_volumes_for_skins: TRUE,
            extended_shadow_frustum_culling: TRUE,
            natural_sun_elevation: TRUE,
            shadows_for_projectiles: TRUE,
            shadows_for_props: TRUE,
            bridge_shadows: TRUE,
            water_reflections: TRUE,
            explosion_dynamic_lights: TRUE,
            vehicle_headlights: TRUE,
            enable_post_processing: TRUE,
            post_process_bloom: TRUE,
            post_process_hdr: TRUE,
            post_process_fxaa: FALSE,
            air_unit_ground_indicator: TRUE,
            anti_aliasing: MULTISAMPLE_MODE_NONE,
            texture_filter_mode: TEXTURE_FILTER_BILINEAR,
            texture_anisotropy_level: TEXTURE_FILTER_ANISOTROPIC_2X,
            gamma_value: 50.0,
            x_resolution: DEFAULT_DISPLAY_WIDTH,
            y_resolution: DEFAULT_DISPLAY_HEIGHT,
        }
    }
}

/// Pure-Rust Release mirror of C++ `GlobalData` (`GlobalData.h` / `GlobalData.cpp`).
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalData {
    pub command_line_data: CommandLineData,

    pub map_name: String,
    pub move_hint_name: String,
    pub use_trees: Bool,
    pub use_tree_sway: Bool,
    pub use_draw_module_lod: Bool,
    pub use_heat_effects: Bool,
    pub use_fps_limit: Bool,
    pub dump_asset_usage: Bool,
    pub frames_per_second_limit: Int,
    pub skirmish_tick_rate: Int,
    pub chip_set_type: Int,

    pub headless: Bool,
    pub check_for_updates: Bool,

    pub windowed: Bool,
    pub x_resolution: Int,
    pub y_resolution: Int,
    pub max_shell_screens: Int,
    pub use_cloud_map: Bool,
    pub use_3way_terrain_blends: Int,
    pub use_light_map: Bool,
    pub bilinear_terrain_tex: Bool,
    pub trilinear_terrain_tex: Bool,
    pub multi_pass_terrain: Bool,
    pub adjust_cliff_textures: Bool,
    pub stretch_terrain: Bool,
    pub use_half_height_map: Bool,
    pub draw_entire_terrain: Bool,
    pub terrain_lod: TerrainLod,
    pub enable_dynamic_lod: Bool,
    pub enable_static_lod: Bool,
    pub terrain_lod_target_time_ms: Int,
    pub use_alternate_mouse: Bool,
    pub use_right_mouse_scroll_with_alternate_mouse: Bool,
    pub client_retaliation_mode_enabled: Bool,
    pub double_click_attack_move: Bool,
    pub right_mouse_always_scrolls: Bool,
    pub jpeg_quality: Int,
    pub use_water_plane: Bool,
    pub use_cloud_plane: Bool,
    pub use_shadow_volumes: Bool,
    pub use_shadow_volumes_for_skins: Bool,
    pub shadows_for_projectiles: Bool,
    pub shadows_for_props: Bool,
    pub advanced_shadows: Bool,
    pub advanced_lighting: Bool,
    pub extended_shadow_frustum_culling: Bool,
    pub natural_sun_elevation: Bool,
    pub bridge_shadows: Bool,
    pub water_reflections: Bool,
    pub explosion_dynamic_lights: Bool,
    pub vehicle_headlights: Bool,
    pub enable_post_processing: Bool,
    pub post_process_bloom: Bool,
    pub post_process_hdr: Bool,
    pub post_process_fxaa: Bool,
    pub air_unit_ground_indicator: Bool,

    pub use_shadow_decals: Bool,
    pub texture_reduction_factor: Int,
    pub enable_behind_building_markers: Bool,
    pub water_position_x: Real,
    pub water_position_y: Real,
    pub water_position_z: Real,
    pub water_extent_x: Real,
    pub water_extent_y: Real,
    pub water_type: Int,
    pub show_soft_water_edge: Bool,
    pub using_water_track_editor: Bool,
    pub is_world_builder: Bool,
    pub feather_water: Int,

    pub vertex_water_available_maps: [String; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_height_clamp_low: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_height_clamp_hi: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_angle: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_x_position: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_y_position: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_z_position: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_x_grid_cells: [Int; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_y_grid_cells: [Int; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_grid_size: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_attenuation_a: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_attenuation_b: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_attenuation_c: [Real; MAX_WATER_GRID_SETTINGS],
    pub vertex_water_attenuation_range: [Real; MAX_WATER_GRID_SETTINGS],

    pub downwind_angle: Real,
    pub sky_box_position_z: Real,
    /// Note: C++ stores `Real m_drawSkyBox`, populated by `INI::parseBool` writing byte 0.
    pub draw_sky_box: Real,
    pub sky_box_scale: Real,
    pub viewport_height_scale: Real,
    pub camera_pitch: Real,
    pub camera_yaw: Real,
    pub camera_height: Real,
    pub max_camera_height: Real,
    pub min_camera_height: Real,
    pub terrain_draw_distance_scale: Real,

    pub terrain_height_at_edge_of_map: Real,
    pub unit_damaged_thresh: Real,
    pub unit_really_damaged_thresh: Real,
    pub ground_stiffness: Real,
    pub structure_stiffness: Real,
    pub gravity: Real,
    pub stealth_friendly_opacity: Real,
    pub default_occlusion_delay: UnsignedInt,

    pub preload_assets: Bool,
    pub preload_everything: Bool,
    pub preload_report: Bool,

    pub partition_cell_size: Real,

    pub ammo_pip_world_offset: Coord3D,
    pub container_pip_world_offset: Coord3D,
    pub ammo_pip_screen_offset: Coord2D,
    pub container_pip_screen_offset: Coord2D,
    pub ammo_pip_scale_factor: Real,
    pub container_pip_scale_factor: Real,

    pub historic_damage_limit: UnsignedInt,

    pub max_terrain_tracks: Int,
    pub max_tank_track_edges: Int,
    pub max_tank_track_opaque_edges: Int,
    pub max_tank_track_fade_delay: Int,

    pub level_gain_animation_name: String,
    pub level_gain_animation_display_time_in_seconds: Real,
    pub level_gain_animation_z_rise_per_second: Real,

    pub get_healed_animation_name: String,
    pub get_healed_animation_display_time_in_seconds: Real,
    pub get_healed_animation_z_rise_per_second: Real,

    pub time_of_day: TimeOfDay,
    pub weather: Weather,
    pub make_track_marks: Bool,
    pub hide_garrison_flags: Bool,
    pub force_models_to_follow_time_of_day: Bool,
    pub force_models_to_follow_weather: Bool,

    pub terrain_lighting: [[TerrainLighting; MAX_GLOBAL_LIGHTS]; TIME_OF_DAY_COUNT],
    pub terrain_objects_lighting: [[TerrainLighting; MAX_GLOBAL_LIGHTS]; TIME_OF_DAY_COUNT],
    pub terrain_ambient: [RGBColor; MAX_GLOBAL_LIGHTS],
    pub terrain_diffuse: [RGBColor; MAX_GLOBAL_LIGHTS],
    pub terrain_light_pos: [Coord3D; MAX_GLOBAL_LIGHTS],

    pub infantry_light_scale: [Real; TIME_OF_DAY_COUNT],
    pub script_override_infantry_light_scale: Real,

    pub solo_player_health_bonus_for_difficulty: [[Real; DIFFICULTY_COUNT]; PLAYERTYPE_COUNT],

    pub max_visible_translucent_objects: Int,
    pub max_visible_occluder_objects: Int,
    pub max_visible_occludee_objects: Int,
    pub max_visible_non_occluder_or_occludee_objects: Int,
    pub occluded_luminance_scale: Real,

    pub num_global_lights: Int,
    pub max_road_segments: Int,
    pub max_road_vertex: Int,
    pub max_road_index: Int,
    pub max_road_types: Int,

    pub audio_on: Bool,
    pub music_on: Bool,
    pub sounds_on: Bool,
    pub sounds_3d_on: Bool,
    pub speech_on: Bool,
    pub video_on: Bool,
    pub disable_camera_movement: Bool,

    pub use_fx: Bool,
    pub show_client_physics: Bool,
    pub show_terrain_normals: Bool,

    pub no_draw: UnsignedInt,
    /// Note: C++ stores `AIDebugOptions m_debugAI` (`Int`), populated by `INI::parseBool` writing byte 0.
    pub debug_ai: Int,
    pub debug_supply_center_placement: Bool,
    pub debug_ai_obstacles: Bool,
    pub show_object_health: Bool,
    pub script_debug: Bool,
    pub particle_edit: Bool,
    pub display_debug: Bool,
    pub win_cursors: Bool,
    pub constant_debug_update: Bool,
    pub show_team_dot: Bool,

    pub force_benchmark: Bool,
    pub fixed_seed: Int,
    pub particle_scale: Real,

    pub auto_fire_particle_small_prefix: String,
    pub auto_fire_particle_small_system: String,
    pub auto_fire_particle_small_max: Int,
    pub auto_fire_particle_medium_prefix: String,
    pub auto_fire_particle_medium_system: String,
    pub auto_fire_particle_medium_max: Int,
    pub auto_fire_particle_large_prefix: String,
    pub auto_fire_particle_large_system: String,
    pub auto_fire_particle_large_max: Int,
    pub auto_smoke_particle_small_prefix: String,
    pub auto_smoke_particle_small_system: String,
    pub auto_smoke_particle_small_max: Int,
    pub auto_smoke_particle_medium_prefix: String,
    pub auto_smoke_particle_medium_system: String,
    pub auto_smoke_particle_medium_max: Int,
    pub auto_smoke_particle_large_prefix: String,
    pub auto_smoke_particle_large_system: String,
    pub auto_smoke_particle_large_max: Int,
    pub auto_aflame_particle_prefix: String,
    pub auto_aflame_particle_system: String,
    pub auto_aflame_particle_max: Int,

    pub net_min_players: Int,
    pub default_ip: UnsignedInt,
    pub firewall_behavior: UnsignedInt,
    pub firewall_port_override: UnsignedInt,
    pub firewall_port_allocation_delta: Short,

    pub base_value_per_supply_box: Int,
    pub build_speed: Real,
    pub min_dist_from_edge_of_map_for_build: Real,
    pub supply_build_border: Real,
    pub allowed_height_variation_for_building: Real,
    pub min_low_energy_production_speed: Real,
    pub max_low_energy_production_speed: Real,
    pub low_energy_penalty_modifier: Real,
    pub multiple_factory: Real,
    pub refund_percent: Real,

    pub command_center_heal_range: Real,
    pub command_center_heal_amount: Real,
    pub max_line_build_objects: Int,
    pub max_tunnel_capacity: Int,
    pub horizontal_scroll_speed_factor: Real,
    pub vertical_scroll_speed_factor: Real,
    pub scroll_amount_cutoff: Real,
    pub camera_adjust_speed: Real,
    pub enforce_max_camera_height: Bool,
    pub build_map_cache: Bool,
    pub initial_file: String,
    pub pending_file: String,
    pub load_save_game: String,
    pub load_replay_game: String,

    pub simulate_replays: Vec<String>,
    pub simulate_replay_jobs: Int,

    pub max_particle_count: Int,
    pub max_field_particle_count: Int,
    pub weapon_bonus_set: Option<WeaponBonusSet>,
    pub health_bonus: [Real; LEVEL_COUNT],
    pub default_structure_rubble_height: Real,

    pub shell_map_name: String,
    pub shell_map_on: Bool,
    pub play_intro: Bool,
    pub play_sizzle: Bool,
    pub load_screen_render: Bool,

    pub keyboard_scroll_factor: Real,
    pub mouse_scroll_factor: Real,
    pub keyboard_default_scroll_factor: Real,
    pub draw_scroll_anchor: Bool,
    pub move_scroll_anchor: Bool,
    pub animate_windows: Bool,
    pub incremental_agp_buf: Bool,

    pub ini_crc: UnsignedInt,
    pub exe_crc: UnsignedInt,

    pub movement_penalty_damage_state: BodyDamageType,

    pub group_select_min_select_size: Int,
    pub group_select_volume_base: Real,
    pub group_select_volume_increment: Real,
    pub max_unit_select_sounds: Int,

    pub selection_flash_saturation_factor: Real,
    pub selection_flash_house_color: Bool,

    pub camera_audible_radius: Real,
    pub group_move_click_to_gather_factor: Real,

    pub anti_alias_level: UnsignedInt,
    pub texture_filtering_mode: UnsignedInt,
    pub texture_anisotropy_level: UnsignedInt,

    pub language_filter_pref: Bool,
    pub load_screen_demo: Bool,
    pub disable_render: Bool,

    pub save_camera_in_replay: Bool,
    pub use_camera_in_replay: Bool,
    pub enable_player_observer: Bool,

    pub network_latency_font_size: Int,
    pub render_fps_font_size: Int,
    pub system_time_font_size: Int,
    pub game_time_font_size: Int,
    pub player_info_list_font_size: Int,

    pub show_money_per_minute: Bool,
    pub allow_money_per_minute_for_player: Bool,
    pub game_window_transition_speed_multiplier: Real,

    pub shake_subtle_intensity: Real,
    pub shake_normal_intensity: Real,
    pub shake_strong_intensity: Real,
    pub shake_severe_intensity: Real,
    pub shake_cine_extreme_intensity: Real,
    pub shake_cine_insane_intensity: Real,
    pub max_shake_intensity: Real,
    pub max_shake_range: Real,

    pub sell_percentage: Real,
    pub base_regen_health_percent_per_second: Real,
    pub base_regen_delay: UnsignedInt,

    pub hot_key_text_color: Color,
    pub special_power_view_object_name: String,

    pub object_placement_opacity: Real,
    pub object_placement_shadows: Bool,
    pub standard_public_bones: Vec<String>,
    pub standard_minefield_density: Real,
    pub standard_minefield_distance: Real,

    pub show_metrics: Bool,
    pub default_starting_cash: Money,
    pub debug_show_graphical_framerate: Bool,

    pub power_bar_base: Int,
    pub power_bar_intervals: Real,
    pub power_bar_yellow_range: Int,
    pub display_gamma: Real,
    pub unlook_persist_duration: UnsignedInt,
    pub should_update_tga_to_dds: Bool,
    pub double_click_time_ms: UnsignedInt,

    pub shroud_color: RGBColor,
    pub clear_alpha: UnsignedByte,
    pub fog_alpha: UnsignedByte,
    pub shroud_alpha: UnsignedByte,

    pub network_fps_history_length: UnsignedInt,
    pub network_latency_history_length: UnsignedInt,
    pub network_cushion_history_length: UnsignedInt,
    pub network_run_ahead_metrics_time: UnsignedInt,
    pub network_keep_alive_delay: UnsignedInt,
    pub network_run_ahead_slack: UnsignedInt,
    pub network_disconnect_time: UnsignedInt,
    pub network_player_timeout_time: UnsignedInt,
    pub network_disconnect_screen_notify_time: UnsignedInt,

    pub keyboard_camera_rotate_speed: Real,
    pub play_stats: Int,
    pub tivo_fast_mode: Bool,
    pub shroud_on: Bool,
    pub is_breakable_movie: Bool,
    pub break_the_movie: Bool,

    pub mod_dir: String,
    pub mod_big: String,
    pub asset_path: String,
    pub generals_asset_path: String,
    pub loose_files_win: Bool,
    pub user_data_dir: String,
}

impl Default for GlobalData {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalData {
    pub fn new() -> Self {
        todo!("PKT-442: implement GlobalData::new matching GlobalData::GlobalData()")
    }

    pub fn is_use_shadow_volumes_for_skins(&self) -> Bool {
        todo!("PKT-442: implement is_use_shadow_volumes_for_skins")
    }

    pub fn is_shadows_for_projectiles(&self) -> Bool {
        todo!("PKT-442: implement is_shadows_for_projectiles")
    }

    pub fn is_shadows_for_props(&self) -> Bool {
        todo!("PKT-442: implement is_shadows_for_props")
    }

    pub fn is_extended_shadow_frustum_culling(&self) -> Bool {
        todo!("PKT-442: implement is_extended_shadow_frustum_culling")
    }

    pub fn is_natural_sun_elevation(&self) -> Bool {
        todo!("PKT-442: implement is_natural_sun_elevation")
    }

    pub fn is_bridge_shadows(&self) -> Bool {
        todo!("PKT-442: implement is_bridge_shadows")
    }

    pub fn is_water_reflections(&self) -> Bool {
        todo!("PKT-442: implement is_water_reflections")
    }

    pub fn is_explosion_dynamic_lights(&self) -> Bool {
        todo!("PKT-442: implement is_explosion_dynamic_lights")
    }

    pub fn is_vehicle_headlights(&self) -> Bool {
        todo!("PKT-442: implement is_vehicle_headlights")
    }

    pub fn is_enable_post_processing(&self) -> Bool {
        todo!("PKT-442: implement is_enable_post_processing")
    }

    pub fn is_post_process_bloom(&self) -> Bool {
        todo!("PKT-442: implement is_post_process_bloom")
    }

    pub fn is_post_process_hdr(&self) -> Bool {
        todo!("PKT-442: implement is_post_process_hdr")
    }

    pub fn is_post_process_fxaa(&self) -> Bool {
        todo!("PKT-442: implement is_post_process_fxaa")
    }

    pub fn is_air_unit_ground_indicator(&self) -> Bool {
        todo!("PKT-442: implement is_air_unit_ground_indicator")
    }

    pub fn set_time_of_day(&mut self, tod_raw: Int) -> Bool {
        todo!("PKT-442: implement GlobalData::set_time_of_day matching GlobalData.cpp:1155-1171")
    }

    pub fn parse_ini_block(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        todo!("PKT-442: implement GlobalData::parse_ini_block matching s_GlobalDataFieldParseTable")
    }

    pub fn apply_option_preferences(&mut self, prefs: &OptionPreferencesOverrides) {
        todo!("PKT-442: implement GlobalData::apply_option_preferences matching GlobalData.cpp:1272-1361")
    }

    pub fn parse_custom_definition(&mut self, has_fullviewport_dat: Bool) {
        todo!("PKT-442: implement GlobalData::parse_custom_definition matching GlobalData.cpp:1364-1371")
    }

    pub fn compute_exe_crc_with_reader<F>(
        &mut self,
        exe_reader: Option<&mut F>,
        version_number: Option<UnsignedInt>,
        skirmish_scripts: Option<&[u8]>,
        multiplayer_scripts: Option<&[u8]>,
    ) -> UnsignedInt
    where
        F: FnMut(&mut [u8]) -> usize,
    {
        todo!("PKT-442: implement GlobalData::compute_exe_crc_with_reader matching GlobalData.cpp:1373-1476")
    }

    pub fn compute_exe_crc(
        &mut self,
        exe_bytes: Option<&[u8]>,
        version_number: Option<UnsignedInt>,
        skirmish_scripts: Option<&[u8]>,
        multiplayer_scripts: Option<&[u8]>,
    ) -> UnsignedInt {
        todo!("PKT-442: implement GlobalData::compute_exe_crc matching GlobalData.cpp:1373-1476")
    }
}

/// Manages base and override stack for `GlobalData` (`GlobalData::newOverride` / `reset`).
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDataStore {
    stack: Vec<GlobalData>,
}

impl Default for GlobalDataStore {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalDataStore {
    pub fn new() -> Self {
        todo!("PKT-442: implement GlobalDataStore::new")
    }

    pub fn current(&self) -> &GlobalData {
        todo!("PKT-442: implement GlobalDataStore::current")
    }

    pub fn current_mut(&mut self) -> &mut GlobalData {
        todo!("PKT-442: implement GlobalDataStore::current_mut")
    }

    pub fn override_depth(&self) -> usize {
        todo!("PKT-442: implement GlobalDataStore::override_depth")
    }

    pub fn new_override(&mut self) {
        todo!("PKT-442: implement GlobalDataStore::new_override matching GlobalData.cpp:1178-1199")
    }

    pub fn reset(&mut self) {
        todo!("PKT-442: implement GlobalDataStore::reset matching GlobalData.cpp:1211-1240")
    }

    pub fn parse_game_data_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        prefs: Option<&OptionPreferencesOverrides>,
    ) -> Result<(), IniError> {
        todo!("PKT-442: implement GlobalDataStore::parse_game_data_definition matching GlobalData.cpp:1245-1362")
    }
}
