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
        for i in 0..WEAPON_BONUS_FIELD_COUNT {
            bonus.fields[i] += self.fields[i] - 1.0_f32;
        }
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
        for c in 0..WEAPON_BONUS_CONDITION_COUNT {
            if (flags & (1u32 << c)) != 0 {
                self.entries[c].append_bonuses(bonus);
            }
        }
    }

    pub fn parse_weapon_bonus_set(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let condition = IniReader::scan_index_list(
            ini.next_token(zh_common::ini::SEPS_DEFAULT)?,
            &THE_WEAPON_BONUS_NAMES,
        )?;
        let field = IniReader::scan_index_list(
            ini.next_token(zh_common::ini::SEPS_DEFAULT)?,
            &THE_WEAPON_BONUS_FIELD_NAMES,
        )?;
        let value = IniReader::scan_percent_to_real(ini.next_token(zh_common::ini::SEPS_PERCENT)?)?;
        if let Some(c) = WeaponBonusConditionType::from_i32(condition) {
            if let Some(f) = WeaponBonusField::from_i32(field) {
                let _ = c;
                self.entries[condition as usize].set_field(f, value);
            }
        }
        Ok(())
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
        let mut g = Self {
            command_line_data: CommandLineData::default(),
            map_name: String::new(),
            move_hint_name: String::new(),
            use_trees: FALSE,
            use_tree_sway: TRUE,
            use_draw_module_lod: FALSE,
            use_heat_effects: TRUE,
            use_fps_limit: TRUE,
            dump_asset_usage: FALSE,
            frames_per_second_limit: 60,
            skirmish_tick_rate: LOGICFRAMES_PER_SECOND,
            chip_set_type: 0,
            headless: FALSE,
            check_for_updates: TRUE,
            windowed: FALSE,
            x_resolution: DEFAULT_DISPLAY_WIDTH,
            y_resolution: DEFAULT_DISPLAY_HEIGHT,
            max_shell_screens: 0,
            use_cloud_map: FALSE,
            use_3way_terrain_blends: 1,
            use_light_map: FALSE,
            bilinear_terrain_tex: FALSE,
            trilinear_terrain_tex: FALSE,
            multi_pass_terrain: FALSE,
            adjust_cliff_textures: FALSE,
            stretch_terrain: FALSE,
            use_half_height_map: FALSE,
            draw_entire_terrain: FALSE,
            terrain_lod: TerrainLod::Automatic,
            enable_dynamic_lod: TRUE,
            enable_static_lod: TRUE,
            terrain_lod_target_time_ms: 0,
            use_alternate_mouse: FALSE,
            use_right_mouse_scroll_with_alternate_mouse: TRUE,
            client_retaliation_mode_enabled: TRUE,
            double_click_attack_move: FALSE,
            right_mouse_always_scrolls: FALSE,
            jpeg_quality: DEFAULT_JPEG_QUALITY,
            use_water_plane: FALSE,
            use_cloud_plane: FALSE,
            use_shadow_volumes: FALSE,
            use_shadow_volumes_for_skins: TRUE,
            shadows_for_projectiles: TRUE,
            shadows_for_props: TRUE,
            advanced_shadows: TRUE,
            advanced_lighting: TRUE,
            extended_shadow_frustum_culling: TRUE,
            natural_sun_elevation: TRUE,
            bridge_shadows: TRUE,
            water_reflections: TRUE,
            explosion_dynamic_lights: TRUE,
            vehicle_headlights: TRUE,
            enable_post_processing: TRUE,
            post_process_bloom: TRUE,
            post_process_hdr: TRUE,
            post_process_fxaa: FALSE,
            air_unit_ground_indicator: TRUE,
            use_shadow_decals: FALSE,
            texture_reduction_factor: -1,
            enable_behind_building_markers: TRUE,
            water_position_x: 0.0,
            water_position_y: 0.0,
            water_position_z: 0.0,
            water_extent_x: 0.0,
            water_extent_y: 0.0,
            water_type: 0,
            show_soft_water_edge: TRUE,
            using_water_track_editor: FALSE,
            is_world_builder: FALSE,
            feather_water: 0,
            vertex_water_available_maps: std::array::from_fn(|_| String::new()),
            vertex_water_height_clamp_low: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_height_clamp_hi: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_angle: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_x_position: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_y_position: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_z_position: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_x_grid_cells: [0; MAX_WATER_GRID_SETTINGS],
            vertex_water_y_grid_cells: [0; MAX_WATER_GRID_SETTINGS],
            vertex_water_grid_size: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_attenuation_a: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_attenuation_b: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_attenuation_c: [0.0; MAX_WATER_GRID_SETTINGS],
            vertex_water_attenuation_range: [0.0; MAX_WATER_GRID_SETTINGS],
            downwind_angle: -0.785_f32,
            sky_box_position_z: 0.0,
            draw_sky_box: FALSE as Real,
            sky_box_scale: 4.5,
            viewport_height_scale: 0.80,
            camera_pitch: 0.0,
            camera_yaw: 0.0,
            camera_height: 0.0,
            max_camera_height: 300.0,
            min_camera_height: 100.0,
            terrain_draw_distance_scale: 1.0,
            terrain_height_at_edge_of_map: 0.0,
            unit_damaged_thresh: 0.5,
            unit_really_damaged_thresh: 0.1,
            ground_stiffness: 0.5,
            structure_stiffness: 0.5,
            gravity: -1.0,
            stealth_friendly_opacity: 0.5,
            default_occlusion_delay: LOGICFRAMES_PER_SECOND as UnsignedInt * 3,
            preload_assets: FALSE,
            preload_everything: FALSE,
            preload_report: FALSE,
            partition_cell_size: 0.0,
            ammo_pip_world_offset: Coord3D::default(),
            container_pip_world_offset: Coord3D::default(),
            ammo_pip_screen_offset: Coord2D::default(),
            container_pip_screen_offset: Coord2D::default(),
            ammo_pip_scale_factor: 1.0,
            container_pip_scale_factor: 1.0,
            historic_damage_limit: 0,
            max_terrain_tracks: 0,
            max_tank_track_edges: 100,
            max_tank_track_opaque_edges: 25,
            max_tank_track_fade_delay: 300_000,
            level_gain_animation_name: String::new(),
            level_gain_animation_display_time_in_seconds: 0.0,
            level_gain_animation_z_rise_per_second: 0.0,
            get_healed_animation_name: String::new(),
            get_healed_animation_display_time_in_seconds: 0.0,
            get_healed_animation_z_rise_per_second: 0.0,
            time_of_day: TimeOfDay::Afternoon,
            weather: Weather::Normal,
            make_track_marks: FALSE,
            hide_garrison_flags: FALSE,
            force_models_to_follow_time_of_day: TRUE,
            force_models_to_follow_weather: TRUE,
            terrain_lighting: [[TerrainLighting::default(); MAX_GLOBAL_LIGHTS]; TIME_OF_DAY_COUNT],
            terrain_objects_lighting: [[TerrainLighting::default(); MAX_GLOBAL_LIGHTS];
                TIME_OF_DAY_COUNT],
            terrain_ambient: [RGBColor::default(); MAX_GLOBAL_LIGHTS],
            terrain_diffuse: [RGBColor::default(); MAX_GLOBAL_LIGHTS],
            terrain_light_pos: [Coord3D::default(); MAX_GLOBAL_LIGHTS],
            infantry_light_scale: [1.0; TIME_OF_DAY_COUNT],
            script_override_infantry_light_scale: -1.0,
            solo_player_health_bonus_for_difficulty: [[1.0; DIFFICULTY_COUNT]; PLAYERTYPE_COUNT],
            max_visible_translucent_objects: 512,
            max_visible_occluder_objects: 512,
            max_visible_occludee_objects: 512,
            max_visible_non_occluder_or_occludee_objects: 512,
            occluded_luminance_scale: 0.5,
            num_global_lights: 3,
            max_road_segments: 0,
            max_road_vertex: 0,
            max_road_index: 0,
            max_road_types: 0,
            audio_on: TRUE,
            music_on: TRUE,
            sounds_on: TRUE,
            sounds_3d_on: TRUE,
            speech_on: TRUE,
            video_on: TRUE,
            disable_camera_movement: FALSE,
            use_fx: TRUE,
            show_client_physics: TRUE,
            show_terrain_normals: FALSE,
            no_draw: 0,
            debug_ai: 0,
            debug_supply_center_placement: FALSE,
            debug_ai_obstacles: FALSE,
            show_object_health: FALSE,
            script_debug: FALSE,
            particle_edit: FALSE,
            display_debug: FALSE,
            win_cursors: TRUE,
            constant_debug_update: FALSE,
            show_team_dot: FALSE,
            force_benchmark: FALSE,
            fixed_seed: -1,
            particle_scale: 1.0,
            auto_fire_particle_small_prefix: String::new(),
            auto_fire_particle_small_system: String::new(),
            auto_fire_particle_small_max: 0,
            auto_fire_particle_medium_prefix: String::new(),
            auto_fire_particle_medium_system: String::new(),
            auto_fire_particle_medium_max: 0,
            auto_fire_particle_large_prefix: String::new(),
            auto_fire_particle_large_system: String::new(),
            auto_fire_particle_large_max: 0,
            auto_smoke_particle_small_prefix: String::new(),
            auto_smoke_particle_small_system: String::new(),
            auto_smoke_particle_small_max: 0,
            auto_smoke_particle_medium_prefix: String::new(),
            auto_smoke_particle_medium_system: String::new(),
            auto_smoke_particle_medium_max: 0,
            auto_smoke_particle_large_prefix: String::new(),
            auto_smoke_particle_large_system: String::new(),
            auto_smoke_particle_large_max: 0,
            auto_aflame_particle_prefix: String::new(),
            auto_aflame_particle_system: String::new(),
            auto_aflame_particle_max: 0,
            net_min_players: 1,
            default_ip: 0,
            firewall_behavior: FIREWALL_TYPE_UNKNOWN,
            firewall_port_override: 0,
            firewall_port_allocation_delta: 0,
            base_value_per_supply_box: 100,
            build_speed: 0.0,
            min_dist_from_edge_of_map_for_build: 0.0,
            supply_build_border: 0.0,
            allowed_height_variation_for_building: 0.0,
            min_low_energy_production_speed: 0.0,
            max_low_energy_production_speed: 0.0,
            low_energy_penalty_modifier: 0.0,
            multiple_factory: 0.0,
            refund_percent: 0.0,
            command_center_heal_range: 0.0,
            command_center_heal_amount: 0.0,
            max_line_build_objects: 0,
            max_tunnel_capacity: 0,
            horizontal_scroll_speed_factor: 1.0,
            vertical_scroll_speed_factor: 1.0,
            scroll_amount_cutoff: 10.0,
            camera_adjust_speed: 0.1,
            enforce_max_camera_height: TRUE,
            build_map_cache: FALSE,
            initial_file: String::new(),
            pending_file: String::new(),
            load_save_game: String::new(),
            load_replay_game: String::new(),
            simulate_replays: Vec::new(),
            simulate_replay_jobs: SIMULATE_REPLAYS_SEQUENTIAL,
            max_particle_count: 0,
            max_field_particle_count: 30,
            weapon_bonus_set: Some(WeaponBonusSet::new()),
            health_bonus: [1.0; LEVEL_COUNT],
            default_structure_rubble_height: 1.0,
            shell_map_name: "Maps\\ShellMap1\\ShellMap1.map".to_string(),
            shell_map_on: TRUE,
            play_intro: TRUE,
            play_sizzle: TRUE,
            load_screen_render: FALSE,
            keyboard_scroll_factor: 0.5,
            mouse_scroll_factor: 1.0,
            keyboard_default_scroll_factor: 0.5,
            draw_scroll_anchor: FALSE,
            move_scroll_anchor: FALSE,
            animate_windows: TRUE,
            incremental_agp_buf: FALSE,
            ini_crc: 0,
            exe_crc: 0,
            movement_penalty_damage_state: BodyDamageType::ReallyDamaged,
            group_select_min_select_size: 5,
            group_select_volume_base: 0.5,
            group_select_volume_increment: 0.02,
            max_unit_select_sounds: 8,
            selection_flash_saturation_factor: 0.5,
            selection_flash_house_color: FALSE,
            camera_audible_radius: 500.0,
            group_move_click_to_gather_factor: 1.0,
            anti_alias_level: MULTISAMPLE_MODE_NONE,
            texture_filtering_mode: TEXTURE_FILTER_BILINEAR,
            texture_anisotropy_level: TEXTURE_FILTER_ANISOTROPIC_2X,
            language_filter_pref: TRUE,
            load_screen_demo: FALSE,
            disable_render: FALSE,
            save_camera_in_replay: FALSE,
            use_camera_in_replay: FALSE,
            enable_player_observer: FALSE,
            network_latency_font_size: 8,
            render_fps_font_size: 8,
            system_time_font_size: 8,
            game_time_font_size: 8,
            player_info_list_font_size: 8,
            show_money_per_minute: FALSE,
            allow_money_per_minute_for_player: FALSE,
            game_window_transition_speed_multiplier: 1.0,
            shake_subtle_intensity: 0.5,
            shake_normal_intensity: 1.0,
            shake_strong_intensity: 2.5,
            shake_severe_intensity: 5.0,
            shake_cine_extreme_intensity: 8.0,
            shake_cine_insane_intensity: 12.0,
            max_shake_intensity: 10.0,
            max_shake_range: 150.0,
            sell_percentage: 1.0,
            base_regen_health_percent_per_second: 0.0,
            base_regen_delay: 0,
            hot_key_text_color: 0xffff_ff00,
            special_power_view_object_name: String::new(),
            object_placement_opacity: 0.45,
            object_placement_shadows: TRUE,
            standard_public_bones: Vec::new(),
            standard_minefield_density: 0.01,
            standard_minefield_distance: 40.0,
            show_metrics: FALSE,
            default_starting_cash: Money::new(),
            debug_show_graphical_framerate: FALSE,
            power_bar_base: 7,
            power_bar_intervals: 3.0,
            power_bar_yellow_range: 5,
            display_gamma: 1.0,
            unlook_persist_duration: 30,
            should_update_tga_to_dds: FALSE,
            double_click_time_ms: DEFAULT_DOUBLE_CLICK_TIME_MS,
            shroud_color: RGBColor {
                red: 1.0,
                green: 1.0,
                blue: 1.0,
            },
            clear_alpha: 255,
            fog_alpha: 127,
            shroud_alpha: 0,
            network_fps_history_length: 30,
            network_latency_history_length: 200,
            network_cushion_history_length: 10,
            network_run_ahead_metrics_time: 500,
            network_keep_alive_delay: 20,
            network_run_ahead_slack: 10,
            network_disconnect_time: 5000,
            network_player_timeout_time: 60000,
            network_disconnect_screen_notify_time: 15000,
            keyboard_camera_rotate_speed: 0.1,
            play_stats: -1,
            tivo_fast_mode: FALSE,
            shroud_on: TRUE,
            is_breakable_movie: FALSE,
            break_the_movie: FALSE,
            mod_dir: String::new(),
            mod_big: String::new(),
            asset_path: String::new(),
            generals_asset_path: String::new(),
            loose_files_win: FALSE,
            user_data_dir: String::new(),
        };
        g.infantry_light_scale[0] = 0.0;
        g.infantry_light_scale[1] = 1.5;
        g.infantry_light_scale[2] = 1.5;
        g.infantry_light_scale[3] = 1.5;
        g.infantry_light_scale[4] = 1.5;
        for i in 0..MAX_GLOBAL_LIGHTS {
            g.terrain_light_pos[i].z = -1.0;
            for tod in 0..TIME_OF_DAY_COUNT {
                g.terrain_lighting[tod][i].light_pos.z = -1.0;
                g.terrain_objects_lighting[tod][i].light_pos.z = -1.0;
            }
        }
        g.set_time_of_day(TimeOfDay::Afternoon.as_i32());
        g
    }

    pub fn is_use_shadow_volumes_for_skins(&self) -> Bool {
        if self.advanced_shadows != 0 && self.use_shadow_volumes_for_skins != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_shadows_for_projectiles(&self) -> Bool {
        if self.advanced_shadows != 0 && self.shadows_for_projectiles != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_shadows_for_props(&self) -> Bool {
        if self.advanced_shadows != 0 && self.shadows_for_props != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_extended_shadow_frustum_culling(&self) -> Bool {
        if self.advanced_shadows != 0 && self.extended_shadow_frustum_culling != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_natural_sun_elevation(&self) -> Bool {
        if self.advanced_shadows != 0 && self.natural_sun_elevation != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_bridge_shadows(&self) -> Bool {
        if self.advanced_shadows != 0 && self.bridge_shadows != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_water_reflections(&self) -> Bool {
        self.water_reflections
    }

    pub fn is_explosion_dynamic_lights(&self) -> Bool {
        if self.advanced_lighting != 0 && self.explosion_dynamic_lights != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_vehicle_headlights(&self) -> Bool {
        if self.advanced_lighting != 0 && self.vehicle_headlights != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_enable_post_processing(&self) -> Bool {
        if self.enable_post_processing != 0
            && (self.post_process_bloom != 0
                || self.post_process_hdr != 0
                || self.post_process_fxaa != 0)
        {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_post_process_bloom(&self) -> Bool {
        if self.enable_post_processing != 0 && self.post_process_bloom != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_post_process_hdr(&self) -> Bool {
        if self.enable_post_processing != 0 && self.post_process_hdr != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_post_process_fxaa(&self) -> Bool {
        if self.enable_post_processing != 0 && self.post_process_fxaa != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn is_air_unit_ground_indicator(&self) -> Bool {
        self.air_unit_ground_indicator
    }

    pub fn set_time_of_day(&mut self, tod_raw: Int) -> Bool {
        if tod_raw >= TIME_OF_DAY_COUNT as Int || tod_raw < TIME_OF_DAY_FIRST {
            return FALSE;
        }
        self.time_of_day = TimeOfDay::from_i32(tod_raw).unwrap_or(TimeOfDay::Invalid);
        for i in 0..MAX_GLOBAL_LIGHTS {
            self.terrain_ambient[i] = self.terrain_lighting[tod_raw as usize][i].ambient;
            self.terrain_diffuse[i] = self.terrain_lighting[tod_raw as usize][i].diffuse;
            self.terrain_light_pos[i] = self.terrain_lighting[tod_raw as usize][i].light_pos;
        }
        TRUE
    }

    pub fn parse_ini_block(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        loop {
            ini.read_line(None)?;
            let field = match std::str::from_utf8(ini.current_line_bytes()) {
                Ok(s) => s.split_ascii_whitespace().next().map(str::to_string),
                Err(_) => None,
            };
            let Some(field) = field else {
                if ini.is_eof() {
                    break;
                }
                continue;
            };
            if field.eq_ignore_ascii_case("End")
                || IniReader::is_end_of_block(ini.current_line_bytes()) != FALSE
            {
                break;
            }
            let _ = ini.next_token(zh_common::ini::SEPS_DEFAULT)?;
            match field.as_str() {
                "Windowed" => {
                    ini.parse_bool(&mut self.windowed)?;
                }
                "CheckForUpdates" => {
                    ini.parse_bool(&mut self.check_for_updates)?;
                }
                "XResolution" => {
                    ini.parse_int(&mut self.x_resolution)?;
                }
                "YResolution" => {
                    ini.parse_int(&mut self.y_resolution)?;
                }
                "MapName" => {
                    ini.parse_ascii_string(&mut self.map_name)?;
                }
                "MoveHintName" => {
                    ini.parse_ascii_string(&mut self.move_hint_name)?;
                }
                "UseTrees" => {
                    ini.parse_bool(&mut self.use_trees)?;
                }
                "UseFPSLimit" => {
                    ini.parse_bool(&mut self.use_fps_limit)?;
                }
                "DumpAssetUsage" => {
                    ini.parse_bool(&mut self.dump_asset_usage)?;
                }
                "FramesPerSecondLimit" => {
                    ini.parse_int(&mut self.frames_per_second_limit)?;
                }
                "SkirmishTickRate" | "TickRate" => {
                    ini.parse_int(&mut self.skirmish_tick_rate)?;
                }
                "ChipsetType" => {
                    ini.parse_int(&mut self.chip_set_type)?;
                }
                "MaxShellScreens" => {
                    ini.parse_int(&mut self.max_shell_screens)?;
                }
                "UseCloudMap" => {
                    ini.parse_bool(&mut self.use_cloud_map)?;
                }
                "UseLightMap" => {
                    ini.parse_bool(&mut self.use_light_map)?;
                }
                "BilinearTerrainTex" => {
                    ini.parse_bool(&mut self.bilinear_terrain_tex)?;
                }
                "TrilinearTerrainTex" => {
                    ini.parse_bool(&mut self.trilinear_terrain_tex)?;
                }
                "MultiPassTerrain" => {
                    ini.parse_bool(&mut self.multi_pass_terrain)?;
                }
                "AdjustCliffTextures" => {
                    ini.parse_bool(&mut self.adjust_cliff_textures)?;
                }
                "Use3WayTerrainBlends" => {
                    ini.parse_int(&mut self.use_3way_terrain_blends)?;
                }
                "StretchTerrain" => {
                    ini.parse_bool(&mut self.stretch_terrain)?;
                }
                "UseHalfHeightMap" => {
                    ini.parse_bool(&mut self.use_half_height_map)?;
                }
                "DrawEntireTerrain" => {
                    ini.parse_bool(&mut self.draw_entire_terrain)?;
                }
                "RightMouseAlwaysScrolls" => {
                    ini.parse_bool(&mut self.right_mouse_always_scrolls)?;
                }
                "UseWaterPlane" => {
                    ini.parse_bool(&mut self.use_water_plane)?;
                }
                "UseCloudPlane" => {
                    ini.parse_bool(&mut self.use_cloud_plane)?;
                }
                "DownwindAngle" => {
                    ini.parse_real(&mut self.downwind_angle)?;
                }
                "UseShadowVolumes" => {
                    ini.parse_bool(&mut self.use_shadow_volumes)?;
                }
                "UseShadowVolumesForSkins" => {
                    ini.parse_bool(&mut self.use_shadow_volumes_for_skins)?;
                }
                "ShadowsForProjectiles" => {
                    ini.parse_bool(&mut self.shadows_for_projectiles)?;
                }
                "ShadowsForProps" => {
                    ini.parse_bool(&mut self.shadows_for_props)?;
                }
                "AdvancedShadows" => {
                    ini.parse_bool(&mut self.advanced_shadows)?;
                }
                "AdvancedLighting" => {
                    ini.parse_bool(&mut self.advanced_lighting)?;
                }
                "ExtendedShadowFrustumCulling" => {
                    ini.parse_bool(&mut self.extended_shadow_frustum_culling)?;
                }
                "NaturalSunElevation" => {
                    ini.parse_bool(&mut self.natural_sun_elevation)?;
                }
                "BridgeShadows" => {
                    ini.parse_bool(&mut self.bridge_shadows)?;
                }
                "WaterReflections" => {
                    ini.parse_bool(&mut self.water_reflections)?;
                }
                "ExplosionDynamicLights" => {
                    ini.parse_bool(&mut self.explosion_dynamic_lights)?;
                }
                "VehicleHeadlights" => {
                    ini.parse_bool(&mut self.vehicle_headlights)?;
                }
                "EnablePostProcessing" => {
                    ini.parse_bool(&mut self.enable_post_processing)?;
                }
                "PostProcessBloom" => {
                    ini.parse_bool(&mut self.post_process_bloom)?;
                }
                "PostProcessHDR" => {
                    ini.parse_bool(&mut self.post_process_hdr)?;
                }
                "PostProcessFXAA" => {
                    ini.parse_bool(&mut self.post_process_fxaa)?;
                }
                "AirUnitGroundIndicator" => {
                    ini.parse_bool(&mut self.air_unit_ground_indicator)?;
                }
                "UseShadowDecals" => {
                    ini.parse_bool(&mut self.use_shadow_decals)?;
                }
                "TextureReductionFactor" => {
                    ini.parse_int(&mut self.texture_reduction_factor)?;
                }
                "UseBehindBuildingMarker" => {
                    ini.parse_bool(&mut self.enable_behind_building_markers)?;
                }
                "WaterPositionX" => {
                    ini.parse_real(&mut self.water_position_x)?;
                }
                "WaterPositionY" => {
                    ini.parse_real(&mut self.water_position_y)?;
                }
                "WaterPositionZ" => {
                    ini.parse_real(&mut self.water_position_z)?;
                }
                "WaterExtentX" => {
                    ini.parse_real(&mut self.water_extent_x)?;
                }
                "WaterExtentY" => {
                    ini.parse_real(&mut self.water_extent_y)?;
                }
                "WaterType" => {
                    ini.parse_int(&mut self.water_type)?;
                }
                "FeatherWater" => {
                    ini.parse_int(&mut self.feather_water)?;
                }
                "ShowSoftWaterEdge" => {
                    ini.parse_bool(&mut self.show_soft_water_edge)?;
                }
                "SkyBoxPositionZ" => {
                    ini.parse_real(&mut self.sky_box_position_z)?;
                }
                "SkyBoxScale" => {
                    ini.parse_real(&mut self.sky_box_scale)?;
                }
                "DrawSkyBox" => {
                    let mut b = FALSE;
                    ini.parse_bool(&mut b)?;
                    self.draw_sky_box =
                        f32::from_bits((self.draw_sky_box.to_bits() & 0xffff_ff00) | b as u32);
                }
                "ViewportHeightScale" => {
                    ini.parse_real(&mut self.viewport_height_scale)?;
                }
                "CameraPitch" => {
                    ini.parse_real(&mut self.camera_pitch)?;
                }
                "CameraYaw" => {
                    ini.parse_real(&mut self.camera_yaw)?;
                }
                "CameraHeight" => {
                    ini.parse_real(&mut self.camera_height)?;
                }
                "MaxCameraHeight" => {
                    ini.parse_real(&mut self.max_camera_height)?;
                }
                "MinCameraHeight" => {
                    ini.parse_real(&mut self.min_camera_height)?;
                }
                "TerrainDrawDistanceScale" => {
                    ini.parse_real(&mut self.terrain_draw_distance_scale)?;
                }
                "TerrainHeightAtEdgeOfMap" => {
                    ini.parse_real(&mut self.terrain_height_at_edge_of_map)?;
                }
                "UnitDamagedThreshold" => {
                    ini.parse_real(&mut self.unit_damaged_thresh)?;
                }
                "UnitReallyDamagedThreshold" => {
                    ini.parse_real(&mut self.unit_really_damaged_thresh)?;
                }
                "GroundStiffness" => {
                    ini.parse_real(&mut self.ground_stiffness)?;
                }
                "StructureStiffness" => {
                    ini.parse_real(&mut self.structure_stiffness)?;
                }
                "Gravity" => {
                    ini.parse_acceleration_real(&mut self.gravity)?;
                }
                "StealthFriendlyOpacity" => {
                    ini.parse_percent_to_real(&mut self.stealth_friendly_opacity)?;
                }
                "DefaultOcclusionDelay" => {
                    ini.parse_duration_unsigned_int(&mut self.default_occlusion_delay)?;
                }
                "PartitionCellSize" => {
                    ini.parse_real(&mut self.partition_cell_size)?;
                }
                "AmmoPipScaleFactor" => {
                    ini.parse_real(&mut self.ammo_pip_scale_factor)?;
                }
                "ContainerPipScaleFactor" => {
                    ini.parse_real(&mut self.container_pip_scale_factor)?;
                }
                "AmmoPipWorldOffset" => {
                    ini.parse_coord3d(&mut self.ammo_pip_world_offset)?;
                }
                "ContainerPipWorldOffset" => {
                    ini.parse_coord3d(&mut self.container_pip_world_offset)?;
                }
                "AmmoPipScreenOffset" => {
                    ini.parse_coord2d(&mut self.ammo_pip_screen_offset)?;
                }
                "ContainerPipScreenOffset" => {
                    ini.parse_coord2d(&mut self.container_pip_screen_offset)?;
                }
                "HistoricDamageLimit" => {
                    ini.parse_duration_unsigned_int(&mut self.historic_damage_limit)?;
                }
                "MaxTerrainTracks" => {
                    ini.parse_int(&mut self.max_terrain_tracks)?;
                }
                "MakeTrackMarks" => {
                    ini.parse_bool(&mut self.make_track_marks)?;
                }
                "HideGarrisonFlags" => {
                    ini.parse_bool(&mut self.hide_garrison_flags)?;
                }
                "ForceModelsToFollowTimeOfDay" => {
                    ini.parse_bool(&mut self.force_models_to_follow_time_of_day)?;
                }
                "ForceModelsToFollowWeather" => {
                    ini.parse_bool(&mut self.force_models_to_follow_weather)?;
                }
                "LevelGainAnimationName" => {
                    ini.parse_ascii_string(&mut self.level_gain_animation_name)?;
                }
                "LevelGainAnimationTime" => {
                    ini.parse_real(&mut self.level_gain_animation_display_time_in_seconds)?;
                }
                "LevelGainAnimationZRise" => {
                    ini.parse_real(&mut self.level_gain_animation_z_rise_per_second)?;
                }
                "GetHealedAnimationName" => {
                    ini.parse_ascii_string(&mut self.get_healed_animation_name)?;
                }
                "GetHealedAnimationTime" => {
                    ini.parse_real(&mut self.get_healed_animation_display_time_in_seconds)?;
                }
                "GetHealedAnimationZRise" => {
                    ini.parse_real(&mut self.get_healed_animation_z_rise_per_second)?;
                }
                "NumberGlobalLights" => {
                    ini.parse_int(&mut self.num_global_lights)?;
                }
                "MaxTranslucentObjects" => {
                    ini.parse_int(&mut self.max_visible_translucent_objects)?;
                }
                "OccludedColorLuminanceScale" => {
                    ini.parse_real(&mut self.occluded_luminance_scale)?;
                }
                "MaxRoadSegments" => {
                    ini.parse_int(&mut self.max_road_segments)?;
                }
                "MaxRoadVertex" => {
                    ini.parse_int(&mut self.max_road_vertex)?;
                }
                "MaxRoadIndex" => {
                    ini.parse_int(&mut self.max_road_index)?;
                }
                "MaxRoadTypes" => {
                    ini.parse_int(&mut self.max_road_types)?;
                }
                "ValuePerSupplyBox" => {
                    ini.parse_int(&mut self.base_value_per_supply_box)?;
                }
                "AudioOn" => {
                    ini.parse_bool(&mut self.audio_on)?;
                }
                "MusicOn" => {
                    ini.parse_bool(&mut self.music_on)?;
                }
                "SoundsOn" => {
                    ini.parse_bool(&mut self.sounds_on)?;
                }
                "Sounds3DOn" => {
                    ini.parse_bool(&mut self.sounds_3d_on)?;
                }
                "SpeechOn" => {
                    ini.parse_bool(&mut self.speech_on)?;
                }
                "VideoOn" => {
                    ini.parse_bool(&mut self.video_on)?;
                }
                "DisableCameraMovements" => {
                    ini.parse_bool(&mut self.disable_camera_movement)?;
                }
                "DebugAI" => {
                    let mut b = FALSE;
                    ini.parse_bool(&mut b)?;
                    self.debug_ai = (self.debug_ai & !0xff) | b as i32;
                }
                "DebugAIObstacles" => {
                    ini.parse_bool(&mut self.debug_ai_obstacles)?;
                }
                "ShowClientPhysics" => {
                    ini.parse_bool(&mut self.show_client_physics)?;
                }
                "ShowTerrainNormals" => {
                    ini.parse_bool(&mut self.show_terrain_normals)?;
                }
                "ShowObjectHealth" => {
                    ini.parse_bool(&mut self.show_object_health)?;
                }
                "ParticleScale" => {
                    ini.parse_real(&mut self.particle_scale)?;
                }
                "AutoFireParticleSmallPrefix" => {
                    ini.parse_ascii_string(&mut self.auto_fire_particle_small_prefix)?;
                }
                "AutoFireParticleSmallSystem" => {
                    ini.parse_ascii_string(&mut self.auto_fire_particle_small_system)?;
                }
                "AutoFireParticleSmallMax" => {
                    ini.parse_int(&mut self.auto_fire_particle_small_max)?;
                }
                "AutoFireParticleMediumPrefix" => {
                    ini.parse_ascii_string(&mut self.auto_fire_particle_medium_prefix)?;
                }
                "AutoFireParticleMediumSystem" => {
                    ini.parse_ascii_string(&mut self.auto_fire_particle_medium_system)?;
                }
                "AutoFireParticleMediumMax" => {
                    ini.parse_int(&mut self.auto_fire_particle_medium_max)?;
                }
                "AutoFireParticleLargePrefix" => {
                    ini.parse_ascii_string(&mut self.auto_fire_particle_large_prefix)?;
                }
                "AutoFireParticleLargeSystem" => {
                    ini.parse_ascii_string(&mut self.auto_fire_particle_large_system)?;
                }
                "AutoFireParticleLargeMax" => {
                    ini.parse_int(&mut self.auto_fire_particle_large_max)?;
                }
                "AutoSmokeParticleSmallPrefix" => {
                    ini.parse_ascii_string(&mut self.auto_smoke_particle_small_prefix)?;
                }
                "AutoSmokeParticleSmallSystem" => {
                    ini.parse_ascii_string(&mut self.auto_smoke_particle_small_system)?;
                }
                "AutoSmokeParticleSmallMax" => {
                    ini.parse_int(&mut self.auto_smoke_particle_small_max)?;
                }
                "AutoSmokeParticleMediumPrefix" => {
                    ini.parse_ascii_string(&mut self.auto_smoke_particle_medium_prefix)?;
                }
                "AutoSmokeParticleMediumSystem" => {
                    ini.parse_ascii_string(&mut self.auto_smoke_particle_medium_system)?;
                }
                "AutoSmokeParticleMediumMax" => {
                    ini.parse_int(&mut self.auto_smoke_particle_medium_max)?;
                }
                "AutoSmokeParticleLargePrefix" => {
                    ini.parse_ascii_string(&mut self.auto_smoke_particle_large_prefix)?;
                }
                "AutoSmokeParticleLargeSystem" => {
                    ini.parse_ascii_string(&mut self.auto_smoke_particle_large_system)?;
                }
                "AutoSmokeParticleLargeMax" => {
                    ini.parse_int(&mut self.auto_smoke_particle_large_max)?;
                }
                "AutoAflameParticlePrefix" => {
                    ini.parse_ascii_string(&mut self.auto_aflame_particle_prefix)?;
                }
                "AutoAflameParticleSystem" => {
                    ini.parse_ascii_string(&mut self.auto_aflame_particle_system)?;
                }
                "AutoAflameParticleMax" => {
                    ini.parse_int(&mut self.auto_aflame_particle_max)?;
                }
                "BuildSpeed" => {
                    ini.parse_real(&mut self.build_speed)?;
                }
                "MinDistFromEdgeOfMapForBuild" => {
                    ini.parse_real(&mut self.min_dist_from_edge_of_map_for_build)?;
                }
                "SupplyBuildBorder" => {
                    ini.parse_real(&mut self.supply_build_border)?;
                }
                "AllowedHeightVariationForBuilding" => {
                    ini.parse_real(&mut self.allowed_height_variation_for_building)?;
                }
                "MinLowEnergyProductionSpeed" => {
                    ini.parse_real(&mut self.min_low_energy_production_speed)?;
                }
                "MaxLowEnergyProductionSpeed" => {
                    ini.parse_real(&mut self.max_low_energy_production_speed)?;
                }
                "LowEnergyPenaltyModifier" => {
                    ini.parse_real(&mut self.low_energy_penalty_modifier)?;
                }
                "MultipleFactory" => {
                    ini.parse_real(&mut self.multiple_factory)?;
                }
                "RefundPercent" => {
                    ini.parse_percent_to_real(&mut self.refund_percent)?;
                }
                "CommandCenterHealRange" => {
                    ini.parse_real(&mut self.command_center_heal_range)?;
                }
                "CommandCenterHealAmount" => {
                    ini.parse_real(&mut self.command_center_heal_amount)?;
                }
                "StandardMinefieldDensity" => {
                    ini.parse_real(&mut self.standard_minefield_density)?;
                }
                "StandardMinefieldDistance" => {
                    ini.parse_real(&mut self.standard_minefield_distance)?;
                }
                "MaxLineBuildObjects" => {
                    ini.parse_int(&mut self.max_line_build_objects)?;
                }
                "MaxTunnelCapacity" => {
                    ini.parse_int(&mut self.max_tunnel_capacity)?;
                }
                "MaxParticleCount" => {
                    ini.parse_int(&mut self.max_particle_count)?;
                }
                "MaxFieldParticleCount" => {
                    ini.parse_int(&mut self.max_field_particle_count)?;
                }
                "HorizontalScrollSpeedFactor" => {
                    ini.parse_real(&mut self.horizontal_scroll_speed_factor)?;
                }
                "VerticalScrollSpeedFactor" => {
                    ini.parse_real(&mut self.vertical_scroll_speed_factor)?;
                }
                "ScrollAmountCutoff" => {
                    ini.parse_real(&mut self.scroll_amount_cutoff)?;
                }
                "CameraAdjustSpeed" => {
                    ini.parse_real(&mut self.camera_adjust_speed)?;
                }
                "EnforceMaxCameraHeight" => {
                    ini.parse_bool(&mut self.enforce_max_camera_height)?;
                }
                "KeyboardScrollSpeedFactor" => {
                    ini.parse_real(&mut self.keyboard_scroll_factor)?;
                }
                "MouseScrollSpeedFactor" => {
                    ini.parse_real(&mut self.mouse_scroll_factor)?;
                }
                "KeyboardDefaultScrollSpeedFactor" => {
                    ini.parse_real(&mut self.keyboard_default_scroll_factor)?;
                }
                "KeyboardCameraRotateSpeed" => {
                    ini.parse_real(&mut self.keyboard_camera_rotate_speed)?;
                }
                "DefaultStructureRubbleHeight" => {
                    ini.parse_real(&mut self.default_structure_rubble_height)?;
                }
                "FixedSeed" => {
                    ini.parse_int(&mut self.fixed_seed)?;
                }
                "ShellMapName" => {
                    ini.parse_ascii_string(&mut self.shell_map_name)?;
                }
                "ShellMapOn" => {
                    ini.parse_bool(&mut self.shell_map_on)?;
                }
                "PlayIntro" => {
                    ini.parse_bool(&mut self.play_intro)?;
                }
                "FirewallBehavior" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.firewall_behavior = v as UnsignedInt;
                }
                "FirewallPortOverride" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.firewall_port_override = (v as UnsignedShort) as UnsignedInt;
                }
                "FirewallPortAllocationDelta" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.firewall_port_allocation_delta = v as Short;
                }
                "GroupSelectMinSelectSize" => {
                    ini.parse_int(&mut self.group_select_min_select_size)?;
                }
                "GroupSelectVolumeBase" => {
                    ini.parse_real(&mut self.group_select_volume_base)?;
                }
                "GroupSelectVolumeIncrement" => {
                    ini.parse_real(&mut self.group_select_volume_increment)?;
                }
                "MaxUnitSelectSounds" => {
                    ini.parse_int(&mut self.max_unit_select_sounds)?;
                }
                "SelectionFlashSaturationFactor" => {
                    ini.parse_real(&mut self.selection_flash_saturation_factor)?;
                }
                "SelectionFlashHouseColor" => {
                    ini.parse_bool(&mut self.selection_flash_house_color)?;
                }
                "CameraAudibleRadius" => {
                    ini.parse_real(&mut self.camera_audible_radius)?;
                }
                "GroupMoveClickToGatherAreaFactor" => {
                    ini.parse_real(&mut self.group_move_click_to_gather_factor)?;
                }
                "ShakeSubtleIntensity" => {
                    ini.parse_real(&mut self.shake_subtle_intensity)?;
                }
                "ShakeNormalIntensity" => {
                    ini.parse_real(&mut self.shake_normal_intensity)?;
                }
                "ShakeStrongIntensity" => {
                    ini.parse_real(&mut self.shake_strong_intensity)?;
                }
                "ShakeSevereIntensity" => {
                    ini.parse_real(&mut self.shake_severe_intensity)?;
                }
                "ShakeCineExtremeIntensity" => {
                    ini.parse_real(&mut self.shake_cine_extreme_intensity)?;
                }
                "ShakeCineInsaneIntensity" => {
                    ini.parse_real(&mut self.shake_cine_insane_intensity)?;
                }
                "MaxShakeIntensity" => {
                    ini.parse_real(&mut self.max_shake_intensity)?;
                }
                "MaxShakeRange" => {
                    ini.parse_real(&mut self.max_shake_range)?;
                }
                "SellPercentage" => {
                    ini.parse_percent_to_real(&mut self.sell_percentage)?;
                }
                "BaseRegenHealthPercentPerSecond" => {
                    ini.parse_percent_to_real(&mut self.base_regen_health_percent_per_second)?;
                }
                "BaseRegenDelay" => {
                    ini.parse_duration_unsigned_int(&mut self.base_regen_delay)?;
                }
                "SpecialPowerViewObject" => {
                    ini.parse_ascii_string(&mut self.special_power_view_object_name)?;
                }
                "ObjectPlacementOpacity" => {
                    ini.parse_real(&mut self.object_placement_opacity)?;
                }
                "ObjectPlacementShadows" => {
                    ini.parse_bool(&mut self.object_placement_shadows)?;
                }
                "ShowMetrics" => {
                    ini.parse_bool(&mut self.show_metrics)?;
                }
                "ShroudColor" => {
                    ini.parse_rgb_color(&mut self.shroud_color)?;
                }
                "ClearAlpha" => {
                    ini.parse_unsigned_byte(&mut self.clear_alpha)?;
                }
                "FogAlpha" => {
                    ini.parse_unsigned_byte(&mut self.fog_alpha)?;
                }
                "ShroudAlpha" => {
                    ini.parse_unsigned_byte(&mut self.shroud_alpha)?;
                }
                "HotKeyTextColor" => {
                    ini.parse_color_int(&mut self.hot_key_text_color)?;
                }
                "PowerBarBase" => {
                    ini.parse_int(&mut self.power_bar_base)?;
                }
                "PowerBarIntervals" => {
                    ini.parse_real(&mut self.power_bar_intervals)?;
                }
                "PowerBarYellowRange" => {
                    ini.parse_int(&mut self.power_bar_yellow_range)?;
                }
                "UnlookPersistDuration" => {
                    ini.parse_duration_unsigned_int(&mut self.unlook_persist_duration)?;
                }
                "NetworkFPSHistoryLength" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_fps_history_length = v as UnsignedInt;
                }
                "NetworkLatencyHistoryLength" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_latency_history_length = v as UnsignedInt;
                }
                "NetworkRunAheadMetricsTime" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_run_ahead_metrics_time = v as UnsignedInt;
                }
                "NetworkCushionHistoryLength" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_cushion_history_length = v as UnsignedInt;
                }
                "NetworkRunAheadSlack" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_run_ahead_slack = v as UnsignedInt;
                }
                "NetworkKeepAliveDelay" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_keep_alive_delay = v as UnsignedInt;
                }
                "NetworkDisconnectTime" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_disconnect_time = v as UnsignedInt;
                }
                "NetworkPlayerTimeoutTime" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_player_timeout_time = v as UnsignedInt;
                }
                "NetworkDisconnectScreenNotifyTime" => {
                    let mut v = 0;
                    ini.parse_int(&mut v)?;
                    self.network_disconnect_screen_notify_time = v as UnsignedInt;
                }
                "PlayStats" => {
                    ini.parse_int(&mut self.play_stats)?;
                }
                "TerrainLODTargetTimeMS" => {
                    ini.parse_int(&mut self.terrain_lod_target_time_ms)?;
                }
                "TerrainLOD" => {
                    let mut v = 0;
                    ini.parse_index_list(&mut v, &TERRAIN_LOD_NAMES)?;
                    self.terrain_lod = TerrainLod::from_i32(v).unwrap_or(TerrainLod::Invalid);
                }
                "TimeOfDay" => {
                    let mut v = 0;
                    ini.parse_index_list(&mut v, &TIME_OF_DAY_NAMES)?;
                    self.time_of_day = TimeOfDay::from_i32(v).unwrap_or(TimeOfDay::Invalid);
                }
                "Weather" => {
                    let mut v = 0;
                    ini.parse_index_list(&mut v, &WEATHER_NAMES)?;
                    self.weather = Weather::from_i32(v).unwrap_or(Weather::Normal);
                }
                "MovementPenaltyDamageState" => {
                    let mut v = 0;
                    ini.parse_index_list(&mut v, &THE_BODY_DAMAGE_TYPE_NAMES)?;
                    self.movement_penalty_damage_state =
                        BodyDamageType::from_i32(v).unwrap_or(BodyDamageType::Pristine);
                }
                "HealthBonus_Veteran" => ini.parse_percent_to_real(&mut self.health_bonus[1])?,
                "HealthBonus_Elite" => ini.parse_percent_to_real(&mut self.health_bonus[2])?,
                "HealthBonus_Heroic" => ini.parse_percent_to_real(&mut self.health_bonus[3])?,
                "HumanSoloPlayerHealthBonus_Easy" => ini.parse_percent_to_real(
                    &mut self.solo_player_health_bonus_for_difficulty[0][0],
                )?,
                "HumanSoloPlayerHealthBonus_Normal" => ini.parse_percent_to_real(
                    &mut self.solo_player_health_bonus_for_difficulty[0][1],
                )?,
                "HumanSoloPlayerHealthBonus_Hard" => ini.parse_percent_to_real(
                    &mut self.solo_player_health_bonus_for_difficulty[0][2],
                )?,
                "AISoloPlayerHealthBonus_Easy" => ini.parse_percent_to_real(
                    &mut self.solo_player_health_bonus_for_difficulty[1][0],
                )?,
                "AISoloPlayerHealthBonus_Normal" => ini.parse_percent_to_real(
                    &mut self.solo_player_health_bonus_for_difficulty[1][1],
                )?,
                "AISoloPlayerHealthBonus_Hard" => ini.parse_percent_to_real(
                    &mut self.solo_player_health_bonus_for_difficulty[1][2],
                )?,
                "StandardPublicBone" => {
                    ini.parse_ascii_string_vector_append(&mut self.standard_public_bones)?
                }
                "DefaultStartingCash" => {
                    let v = IniReader::scan_unsigned_int(
                        ini.next_token(zh_common::ini::SEPS_DEFAULT)?,
                    )?;
                    self.default_starting_cash.set_starting_cash(v);
                }
                "WeaponBonus" => self
                    .weapon_bonus_set
                    .as_mut()
                    .unwrap()
                    .parse_weapon_bonus_set(ini)?,
                "VertexWaterAvailableMaps1" => {
                    ini.parse_ascii_string(&mut self.vertex_water_available_maps[0])?
                }
                "VertexWaterHeightClampLow1" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_low[0])?
                }
                "VertexWaterHeightClampHi1" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_hi[0])?
                }
                "VertexWaterAngle1" => ini.parse_angle_real(&mut self.vertex_water_angle[0])?,
                "VertexWaterXPosition1" => ini.parse_real(&mut self.vertex_water_x_position[0])?,
                "VertexWaterYPosition1" => ini.parse_real(&mut self.vertex_water_y_position[0])?,
                "VertexWaterZPosition1" => ini.parse_real(&mut self.vertex_water_z_position[0])?,
                "VertexWaterXGridCells1" => {
                    ini.parse_int(&mut self.vertex_water_x_grid_cells[0])?
                }
                "VertexWaterYGridCells1" => {
                    ini.parse_int(&mut self.vertex_water_y_grid_cells[0])?
                }
                "VertexWaterGridSize1" => ini.parse_real(&mut self.vertex_water_grid_size[0])?,
                "VertexWaterAttenuationA1" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_a[0])?
                }
                "VertexWaterAttenuationB1" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_b[0])?
                }
                "VertexWaterAttenuationC1" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_c[0])?
                }
                "VertexWaterAttenuationRange1" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_range[0])?
                }
                "VertexWaterAvailableMaps2" => {
                    ini.parse_ascii_string(&mut self.vertex_water_available_maps[1])?
                }
                "VertexWaterHeightClampLow2" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_low[1])?
                }
                "VertexWaterHeightClampHi2" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_hi[1])?
                }
                "VertexWaterAngle2" => ini.parse_angle_real(&mut self.vertex_water_angle[1])?,
                "VertexWaterXPosition2" => ini.parse_real(&mut self.vertex_water_x_position[1])?,
                "VertexWaterYPosition2" => ini.parse_real(&mut self.vertex_water_y_position[1])?,
                "VertexWaterZPosition2" => ini.parse_real(&mut self.vertex_water_z_position[1])?,
                "VertexWaterXGridCells2" => {
                    ini.parse_int(&mut self.vertex_water_x_grid_cells[1])?
                }
                "VertexWaterYGridCells2" => {
                    ini.parse_int(&mut self.vertex_water_y_grid_cells[1])?
                }
                "VertexWaterGridSize2" => ini.parse_real(&mut self.vertex_water_grid_size[1])?,
                "VertexWaterAttenuationA2" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_a[1])?
                }
                "VertexWaterAttenuationB2" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_b[1])?
                }
                "VertexWaterAttenuationC2" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_c[1])?
                }
                "VertexWaterAttenuationRange2" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_range[1])?
                }
                "VertexWaterAvailableMaps3" => {
                    ini.parse_ascii_string(&mut self.vertex_water_available_maps[2])?
                }
                "VertexWaterHeightClampLow3" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_low[2])?
                }
                "VertexWaterHeightClampHi3" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_hi[2])?
                }
                "VertexWaterAngle3" => ini.parse_angle_real(&mut self.vertex_water_angle[2])?,
                "VertexWaterXPosition3" => ini.parse_real(&mut self.vertex_water_x_position[2])?,
                "VertexWaterYPosition3" => ini.parse_real(&mut self.vertex_water_y_position[2])?,
                "VertexWaterZPosition3" => ini.parse_real(&mut self.vertex_water_z_position[2])?,
                "VertexWaterXGridCells3" => {
                    ini.parse_int(&mut self.vertex_water_x_grid_cells[2])?
                }
                "VertexWaterYGridCells3" => {
                    ini.parse_int(&mut self.vertex_water_y_grid_cells[2])?
                }
                "VertexWaterGridSize3" => ini.parse_real(&mut self.vertex_water_grid_size[2])?,
                "VertexWaterAttenuationA3" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_a[2])?
                }
                "VertexWaterAttenuationB3" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_b[2])?
                }
                "VertexWaterAttenuationC3" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_c[2])?
                }
                "VertexWaterAttenuationRange3" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_range[2])?
                }
                "VertexWaterAvailableMaps4" => {
                    ini.parse_ascii_string(&mut self.vertex_water_available_maps[3])?
                }
                "VertexWaterHeightClampLow4" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_low[3])?
                }
                "VertexWaterHeightClampHi4" => {
                    ini.parse_real(&mut self.vertex_water_height_clamp_hi[3])?
                }
                "VertexWaterAngle4" => ini.parse_angle_real(&mut self.vertex_water_angle[3])?,
                "VertexWaterXPosition4" => ini.parse_real(&mut self.vertex_water_x_position[3])?,
                "VertexWaterYPosition4" => ini.parse_real(&mut self.vertex_water_y_position[3])?,
                "VertexWaterZPosition4" => ini.parse_real(&mut self.vertex_water_z_position[3])?,
                "VertexWaterXGridCells4" => {
                    ini.parse_int(&mut self.vertex_water_x_grid_cells[3])?
                }
                "VertexWaterYGridCells4" => {
                    ini.parse_int(&mut self.vertex_water_y_grid_cells[3])?
                }
                "VertexWaterGridSize4" => ini.parse_real(&mut self.vertex_water_grid_size[3])?,
                "VertexWaterAttenuationA4" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_a[3])?
                }
                "VertexWaterAttenuationB4" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_b[3])?
                }
                "VertexWaterAttenuationC4" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_c[3])?
                }
                "VertexWaterAttenuationRange4" => {
                    ini.parse_real(&mut self.vertex_water_attenuation_range[3])?
                }
                "TerrainLightingMorningAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[1][0].ambient)?
                }
                "TerrainObjectsLightingMorningAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[1][0].ambient)?
                }
                "TerrainLightingMorningAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[1][1].ambient)?
                }
                "TerrainObjectsLightingMorningAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[1][1].ambient)?
                }
                "TerrainLightingMorningAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[1][2].ambient)?
                }
                "TerrainObjectsLightingMorningAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[1][2].ambient)?
                }
                "TerrainLightingMorningDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[1][0].diffuse)?
                }
                "TerrainObjectsLightingMorningDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[1][0].diffuse)?
                }
                "TerrainLightingMorningDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[1][1].diffuse)?
                }
                "TerrainObjectsLightingMorningDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[1][1].diffuse)?
                }
                "TerrainLightingMorningDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[1][2].diffuse)?
                }
                "TerrainObjectsLightingMorningDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[1][2].diffuse)?
                }
                "TerrainLightingMorningLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[1][0].light_pos)?
                }
                "TerrainObjectsLightingMorningLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[1][0].light_pos)?
                }
                "TerrainLightingMorningLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[1][1].light_pos)?
                }
                "TerrainObjectsLightingMorningLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[1][1].light_pos)?
                }
                "TerrainLightingMorningLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[1][2].light_pos)?
                }
                "TerrainObjectsLightingMorningLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[1][2].light_pos)?
                }
                "TerrainLightingAfternoonAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[2][0].ambient)?
                }
                "TerrainObjectsLightingAfternoonAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[2][0].ambient)?
                }
                "TerrainLightingAfternoonAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[2][1].ambient)?
                }
                "TerrainObjectsLightingAfternoonAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[2][1].ambient)?
                }
                "TerrainLightingAfternoonAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[2][2].ambient)?
                }
                "TerrainObjectsLightingAfternoonAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[2][2].ambient)?
                }
                "TerrainLightingAfternoonDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[2][0].diffuse)?
                }
                "TerrainObjectsLightingAfternoonDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[2][0].diffuse)?
                }
                "TerrainLightingAfternoonDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[2][1].diffuse)?
                }
                "TerrainObjectsLightingAfternoonDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[2][1].diffuse)?
                }
                "TerrainLightingAfternoonDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[2][2].diffuse)?
                }
                "TerrainObjectsLightingAfternoonDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[2][2].diffuse)?
                }
                "TerrainLightingAfternoonLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[2][0].light_pos)?
                }
                "TerrainObjectsLightingAfternoonLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[2][0].light_pos)?
                }
                "TerrainLightingAfternoonLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[2][1].light_pos)?
                }
                "TerrainObjectsLightingAfternoonLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[2][1].light_pos)?
                }
                "TerrainLightingAfternoonLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[2][2].light_pos)?
                }
                "TerrainObjectsLightingAfternoonLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[2][2].light_pos)?
                }
                "TerrainLightingEveningAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[3][0].ambient)?
                }
                "TerrainObjectsLightingEveningAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[3][0].ambient)?
                }
                "TerrainLightingEveningAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[3][1].ambient)?
                }
                "TerrainObjectsLightingEveningAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[3][1].ambient)?
                }
                "TerrainLightingEveningAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[3][2].ambient)?
                }
                "TerrainObjectsLightingEveningAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[3][2].ambient)?
                }
                "TerrainLightingEveningDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[3][0].diffuse)?
                }
                "TerrainObjectsLightingEveningDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[3][0].diffuse)?
                }
                "TerrainLightingEveningDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[3][1].diffuse)?
                }
                "TerrainObjectsLightingEveningDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[3][1].diffuse)?
                }
                "TerrainLightingEveningDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[3][2].diffuse)?
                }
                "TerrainObjectsLightingEveningDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[3][2].diffuse)?
                }
                "TerrainLightingEveningLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[3][0].light_pos)?
                }
                "TerrainObjectsLightingEveningLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[3][0].light_pos)?
                }
                "TerrainLightingEveningLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[3][1].light_pos)?
                }
                "TerrainObjectsLightingEveningLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[3][1].light_pos)?
                }
                "TerrainLightingEveningLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[3][2].light_pos)?
                }
                "TerrainObjectsLightingEveningLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[3][2].light_pos)?
                }
                "TerrainLightingNightAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[4][0].ambient)?
                }
                "TerrainObjectsLightingNightAmbient" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[4][0].ambient)?
                }
                "TerrainLightingNightAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[4][1].ambient)?
                }
                "TerrainObjectsLightingNightAmbient2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[4][1].ambient)?
                }
                "TerrainLightingNightAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[4][2].ambient)?
                }
                "TerrainObjectsLightingNightAmbient3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[4][2].ambient)?
                }
                "TerrainLightingNightDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[4][0].diffuse)?
                }
                "TerrainObjectsLightingNightDiffuse" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[4][0].diffuse)?
                }
                "TerrainLightingNightDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[4][1].diffuse)?
                }
                "TerrainObjectsLightingNightDiffuse2" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[4][1].diffuse)?
                }
                "TerrainLightingNightDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_lighting[4][2].diffuse)?
                }
                "TerrainObjectsLightingNightDiffuse3" => {
                    ini.parse_rgb_color(&mut self.terrain_objects_lighting[4][2].diffuse)?
                }
                "TerrainLightingNightLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[4][0].light_pos)?
                }
                "TerrainObjectsLightingNightLightPos" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[4][0].light_pos)?
                }
                "TerrainLightingNightLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[4][1].light_pos)?
                }
                "TerrainObjectsLightingNightLightPos2" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[4][1].light_pos)?
                }
                "TerrainLightingNightLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_lighting[4][2].light_pos)?
                }
                "TerrainObjectsLightingNightLightPos3" => {
                    ini.parse_coord3d(&mut self.terrain_objects_lighting[4][2].light_pos)?
                }
                "InfantryLightMorningScale" => ini.parse_real(&mut self.infantry_light_scale[1])?,
                "InfantryLightAfternoonScale" => {
                    ini.parse_real(&mut self.infantry_light_scale[2])?
                }
                "InfantryLightEveningScale" => ini.parse_real(&mut self.infantry_light_scale[3])?,
                "InfantryLightNightScale" => ini.parse_real(&mut self.infantry_light_scale[4])?,
                _ => return Err(IniError::UnknownToken),
            }
            if ini.is_eof() {
                break;
            }
        }
        Ok(())
    }

    pub fn apply_option_preferences(&mut self, prefs: &OptionPreferencesOverrides) {
        self.use_alternate_mouse = prefs.use_alternate_mouse;
        self.use_right_mouse_scroll_with_alternate_mouse =
            prefs.use_right_mouse_scroll_with_alternate_mouse;
        self.client_retaliation_mode_enabled = prefs.client_retaliation_mode_enabled;
        self.double_click_attack_move = prefs.double_click_attack_move;
        self.jpeg_quality = prefs.jpeg_quality;
        self.keyboard_scroll_factor = prefs.keyboard_scroll_factor;
        self.draw_scroll_anchor = prefs.draw_scroll_anchor;
        self.move_scroll_anchor = prefs.move_scroll_anchor;
        self.default_ip = prefs.default_ip;
        self.firewall_behavior = prefs.firewall_behavior as UnsignedInt;
        self.firewall_port_allocation_delta = prefs.firewall_port_allocation_delta;
        self.firewall_port_override = prefs.firewall_port_override as UnsignedInt;
        self.save_camera_in_replay = prefs.save_camera_in_replay;
        self.use_camera_in_replay = prefs.use_camera_in_replay;
        self.enable_player_observer = prefs.enable_player_observer;
        self.network_latency_font_size = prefs.network_latency_font_size;
        self.render_fps_font_size = prefs.render_fps_font_size;
        self.system_time_font_size = prefs.system_time_font_size;
        self.game_time_font_size = prefs.game_time_font_size;
        self.player_info_list_font_size = prefs.player_info_list_font_size;
        self.show_money_per_minute = prefs.show_money_per_minute;
        self.game_window_transition_speed_multiplier =
            prefs.game_window_transition_speed_multiplier;
        self.skirmish_tick_rate = prefs.skirmish_tick_rate;
        self.advanced_shadows = prefs.advanced_shadows;
        self.advanced_lighting = prefs.advanced_lighting;
        self.use_shadow_volumes_for_skins = prefs.use_shadow_volumes_for_skins;
        self.extended_shadow_frustum_culling = prefs.extended_shadow_frustum_culling;
        self.natural_sun_elevation = prefs.natural_sun_elevation;
        self.shadows_for_projectiles = prefs.shadows_for_projectiles;
        self.shadows_for_props = prefs.shadows_for_props;
        self.bridge_shadows = prefs.bridge_shadows;
        self.water_reflections = prefs.water_reflections;
        self.explosion_dynamic_lights = prefs.explosion_dynamic_lights;
        self.vehicle_headlights = prefs.vehicle_headlights;
        self.enable_post_processing = prefs.enable_post_processing;
        self.post_process_bloom = prefs.post_process_bloom;
        self.post_process_hdr = prefs.post_process_hdr;
        self.post_process_fxaa = prefs.post_process_fxaa;
        self.air_unit_ground_indicator = prefs.air_unit_ground_indicator;
        self.anti_alias_level = prefs.anti_aliasing;
        self.texture_filtering_mode = prefs.texture_filter_mode;
        self.texture_anisotropy_level = prefs.texture_anisotropy_level;
        self.x_resolution = prefs.x_resolution;
        self.y_resolution = prefs.y_resolution;
        if let Some(v) = prefs.max_camera_height {
            self.max_camera_height = if !v.is_finite() || v < 100.0 {
                100.0
            } else if v > 1000.0 {
                1000.0
            } else {
                v
            };
        }
        if let Some(v) = prefs.terrain_draw_distance_scale {
            self.terrain_draw_distance_scale = if !v.is_finite() || v < 1.0 {
                1.0
            } else if v > 2.0 {
                2.0
            } else {
                v
            };
        }
        let val = prefs.gamma_value as Int;
        if val < 50 {
            self.display_gamma = if val <= 0 {
                0.6
            } else {
                1.0_f32 - 0.4_f32 * ((50 - val) as Real) / 50.0_f32
            };
        } else if val > 50 {
            self.display_gamma = 1.0_f32 + 1.0_f32 * ((val - 50) as Real) / 50.0_f32;
        }
    }

    pub fn parse_custom_definition(&mut self, has_fullviewport_dat: Bool) {
        if has_fullviewport_dat != FALSE {
            self.viewport_height_scale = 1.0;
        }
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
        let mut crc = Crc::new();
        if let Some(reader) = exe_reader {
            let reader = reader;
            let mut read_count = 0u32;
            loop {
                let mut chunk = [0u8; 65536];
                let n = reader(&mut chunk);
                if n == 0 {
                    break;
                }
                read_count = read_count.wrapping_add(1);
                crc.compute_crc(&chunk[..n.min(chunk.len())], n as Int);
                if read_count > 1000 {
                    break;
                }
            }
        }
        if let Some(v) = version_number {
            crc.compute_crc(&v.to_le_bytes(), 4);
        }
        if let Some(bytes) = skirmish_scripts {
            for chunk in bytes.chunks(65536) {
                crc.compute_crc(chunk, chunk.len() as Int);
            }
        }
        if let Some(bytes) = multiplayer_scripts {
            for chunk in bytes.chunks(65536) {
                crc.compute_crc(chunk, chunk.len() as Int);
            }
        }
        self.exe_crc = crc.get();
        self.exe_crc
    }

    pub fn compute_exe_crc(
        &mut self,
        exe_bytes: Option<&[u8]>,
        version_number: Option<UnsignedInt>,
        skirmish_scripts: Option<&[u8]>,
        multiplayer_scripts: Option<&[u8]>,
    ) -> UnsignedInt {
        let mut crc = Crc::new();
        if let Some(bytes) = exe_bytes {
            let mut offset = 0usize;
            let mut read_count = 0u32;
            while offset < bytes.len() {
                let end = (offset + 65536).min(bytes.len());
                crc.compute_crc(&bytes[offset..end], (end - offset) as Int);
                offset = end;
                read_count = read_count.wrapping_add(1);
                if read_count > 1000 {
                    break;
                }
            }
        }
        if let Some(v) = version_number {
            crc.compute_crc(&v.to_le_bytes(), 4);
        }
        if let Some(bytes) = skirmish_scripts {
            for chunk in bytes.chunks(65536) {
                crc.compute_crc(chunk, chunk.len() as Int);
            }
        }
        if let Some(bytes) = multiplayer_scripts {
            for chunk in bytes.chunks(65536) {
                crc.compute_crc(chunk, chunk.len() as Int);
            }
        }
        self.exe_crc = crc.get();
        self.exe_crc
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
        Self {
            stack: vec![GlobalData::new()],
        }
    }

    pub fn current(&self) -> &GlobalData {
        self.stack.last().expect("GlobalData stack is never empty")
    }

    pub fn current_mut(&mut self) -> &mut GlobalData {
        self.stack
            .last_mut()
            .expect("GlobalData stack is never empty")
    }

    pub fn override_depth(&self) -> usize {
        self.stack.len().saturating_sub(1)
    }

    pub fn new_override(&mut self) {
        let current = self
            .stack
            .last()
            .expect("GlobalData stack is never empty")
            .clone();
        self.stack.push(current);
    }

    pub fn reset(&mut self) {
        if self.stack.len() > 1 {
            self.stack.truncate(1);
        }
    }

    pub fn parse_game_data_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        prefs: Option<&OptionPreferencesOverrides>,
    ) -> Result<(), IniError> {
        if ini.get_load_type() == IniLoadType::CreateOverrides {
            self.new_override();
        }
        self.current_mut().parse_ini_block(ini)?;
        if let Some(prefs) = prefs {
            self.current_mut().apply_option_preferences(prefs);
        }
        Ok(())
    }
}
