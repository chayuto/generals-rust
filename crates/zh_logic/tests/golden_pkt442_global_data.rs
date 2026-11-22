//! Gate V2 Golden Vector parity tests for `PKT-442` (`WP-7.1` `GlobalData`).
//!
//! Active structural & INI-oracle check (`test_pkt442_oracle_structure_and_constants`) verifies
//! the committed C++ oracle, constants, and 1,000-seed synthetic INI CRC parity.
//! Remove the two `#[ignore]` attributes once `PKT-442` is implemented.

use zh_common::crc::Crc;
use zh_common::golden_json::{
    extract_vector_lines, get_i32_array, get_u32,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::{Coord2D, Coord3D, RGBColor};
use zh_common::types::{Bool, Int, Real, Short, UnsignedInt, UnsignedShort, FALSE, TRUE};
use zh_logic::gamelogic::global_data::{
    BodyDamageType, GlobalData, GlobalDataStore, OptionPreferencesOverrides, TerrainLod, TimeOfDay,
    Weather, WeaponBonus, WeaponBonusField, BODY_DAMAGE_TYPE_COUNT, MAX_GLOBAL_LIGHTS,
    MAX_WATER_GRID_SETTINGS, THE_BODY_DAMAGE_TYPE_NAMES, THE_WEAPON_BONUS_FIELD_NAMES,
    THE_WEAPON_BONUS_NAMES, TERRAIN_LOD_COUNT, TERRAIN_LOD_NAMES, TIME_OF_DAY_COUNT,
    TIME_OF_DAY_FIRST, TIME_OF_DAY_NAMES, WEATHER_COUNT, WEATHER_NAMES,
    WEAPON_BONUS_CONDITION_COUNT, WEAPON_BONUS_FIELD_COUNT, DIFFICULTY_COUNT, LEVEL_COUNT,
    PLAYERTYPE_COUNT,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-442.json");

fn splitmix32(s: &mut u32) -> u32 {
    *s = s.wrapping_add(0x9e37_79b9);
    let mut z = *s;
    z = (z ^ (z >> 16)).wrapping_mul(0x85eb_ca6b);
    z = (z ^ (z >> 13)).wrapping_mul(0xc2b2_ae35);
    z ^ (z >> 16)
}

fn crc_str(s: &str) -> u32 {
    let mut c = Crc::new();
    c.compute_crc(s.as_bytes(), s.len() as Int);
    c.get()
}

struct Hasher {
    c: Crc,
}

impl Hasher {
    fn new() -> Self {
        Self { c: Crc::new() }
    }
    fn u8(&mut self, v: u8) {
        self.c.compute_crc(&[v], 1);
    }
    fn i16(&mut self, v: Short) {
        self.c.compute_crc(&v.to_le_bytes(), 2);
    }
    fn i32(&mut self, v: Int) {
        self.c.compute_crc(&v.to_le_bytes(), 4);
    }
    fn u32(&mut self, v: UnsignedInt) {
        self.c.compute_crc(&v.to_le_bytes(), 4);
    }
    fn f32(&mut self, v: Real) {
        self.u32(v.to_bits());
    }
    fn coord2d(&mut self, v: &Coord2D) {
        self.f32(v.x);
        self.f32(v.y);
    }
    fn coord3d(&mut self, v: &Coord3D) {
        self.f32(v.x);
        self.f32(v.y);
        self.f32(v.z);
    }
    fn rgb(&mut self, v: &RGBColor) {
        self.f32(v.red);
        self.f32(v.green);
        self.f32(v.blue);
    }
    fn ascii(&mut self, s: &str) {
        let len = s.len() as u32;
        self.u32(len);
        if !s.is_empty() {
            self.c.compute_crc(s.as_bytes(), s.len() as Int);
        }
    }
    fn finish(&self) -> UnsignedInt {
        self.c.get()
    }
}

fn hash_global_data(g: &GlobalData) -> UnsignedInt {
    let mut h = Hasher::new();
    h.ascii(&g.map_name);
    h.ascii(&g.move_hint_name);
    h.u8(g.use_trees);
    h.u8(g.use_tree_sway);
    h.u8(g.use_draw_module_lod);
    h.u8(g.use_heat_effects);
    h.u8(g.use_fps_limit);
    h.u8(g.dump_asset_usage);
    h.i32(g.frames_per_second_limit);
    h.i32(g.skirmish_tick_rate);
    h.i32(g.chip_set_type);
    h.u8(g.headless);
    h.u8(g.check_for_updates);
    h.u8(g.windowed);
    h.i32(g.x_resolution);
    h.i32(g.y_resolution);
    h.i32(g.max_shell_screens);
    h.u8(g.use_cloud_map);
    h.i32(g.use_3way_terrain_blends);
    h.u8(g.use_light_map);
    h.u8(g.bilinear_terrain_tex);
    h.u8(g.trilinear_terrain_tex);
    h.u8(g.multi_pass_terrain);
    h.u8(g.adjust_cliff_textures);
    h.u8(g.stretch_terrain);
    h.u8(g.use_half_height_map);
    h.u8(g.draw_entire_terrain);
    h.i32(g.terrain_lod.as_i32());
    h.u8(g.enable_dynamic_lod);
    h.u8(g.enable_static_lod);
    h.i32(g.terrain_lod_target_time_ms);
    h.u8(g.use_alternate_mouse);
    h.u8(g.use_right_mouse_scroll_with_alternate_mouse);
    h.u8(g.client_retaliation_mode_enabled);
    h.u8(g.double_click_attack_move);
    h.u8(g.right_mouse_always_scrolls);
    h.i32(g.jpeg_quality);
    h.u8(g.use_water_plane);
    h.u8(g.use_cloud_plane);
    h.u8(g.use_shadow_volumes);
    h.u8(g.use_shadow_volumes_for_skins);
    h.u8(g.shadows_for_projectiles);
    h.u8(g.shadows_for_props);
    h.u8(g.advanced_shadows);
    h.u8(g.advanced_lighting);
    h.u8(g.extended_shadow_frustum_culling);
    h.u8(g.natural_sun_elevation);
    h.u8(g.bridge_shadows);
    h.u8(g.water_reflections);
    h.u8(g.explosion_dynamic_lights);
    h.u8(g.vehicle_headlights);
    h.u8(g.enable_post_processing);
    h.u8(g.post_process_bloom);
    h.u8(g.post_process_hdr);
    h.u8(g.post_process_fxaa);
    h.u8(g.air_unit_ground_indicator);
    h.u8(g.use_shadow_decals);
    h.i32(g.texture_reduction_factor);
    h.u8(g.enable_behind_building_markers);
    h.f32(g.water_position_x);
    h.f32(g.water_position_y);
    h.f32(g.water_position_z);
    h.f32(g.water_extent_x);
    h.f32(g.water_extent_y);
    h.i32(g.water_type);
    h.u8(g.show_soft_water_edge);
    h.u8(g.using_water_track_editor);
    h.u8(g.is_world_builder);
    h.i32(g.feather_water);

    for i in 0..MAX_WATER_GRID_SETTINGS {
        h.ascii(&g.vertex_water_available_maps[i]);
        h.f32(g.vertex_water_height_clamp_low[i]);
        h.f32(g.vertex_water_height_clamp_hi[i]);
        h.f32(g.vertex_water_angle[i]);
        h.f32(g.vertex_water_x_position[i]);
        h.f32(g.vertex_water_y_position[i]);
        h.f32(g.vertex_water_z_position[i]);
        h.i32(g.vertex_water_x_grid_cells[i]);
        h.i32(g.vertex_water_y_grid_cells[i]);
        h.f32(g.vertex_water_grid_size[i]);
        h.f32(g.vertex_water_attenuation_a[i]);
        h.f32(g.vertex_water_attenuation_b[i]);
        h.f32(g.vertex_water_attenuation_c[i]);
        h.f32(g.vertex_water_attenuation_range[i]);
    }

    h.f32(g.downwind_angle);
    h.f32(g.sky_box_position_z);
    h.f32(g.draw_sky_box);
    h.f32(g.sky_box_scale);
    h.f32(g.viewport_height_scale);
    h.f32(g.camera_pitch);
    h.f32(g.camera_yaw);
    h.f32(g.camera_height);
    h.f32(g.max_camera_height);
    h.f32(g.min_camera_height);
    h.f32(g.terrain_draw_distance_scale);
    h.f32(g.terrain_height_at_edge_of_map);
    h.f32(g.unit_damaged_thresh);
    h.f32(g.unit_really_damaged_thresh);
    h.f32(g.ground_stiffness);
    h.f32(g.structure_stiffness);
    h.f32(g.gravity);
    h.f32(g.stealth_friendly_opacity);
    h.u32(g.default_occlusion_delay);
    h.u8(g.preload_assets);
    h.u8(g.preload_everything);
    h.u8(g.preload_report);
    h.f32(g.partition_cell_size);
    h.coord3d(&g.ammo_pip_world_offset);
    h.coord3d(&g.container_pip_world_offset);
    h.coord2d(&g.ammo_pip_screen_offset);
    h.coord2d(&g.container_pip_screen_offset);
    h.f32(g.ammo_pip_scale_factor);
    h.f32(g.container_pip_scale_factor);
    h.u32(g.historic_damage_limit);
    h.i32(g.max_terrain_tracks);
    h.i32(g.max_tank_track_edges);
    h.i32(g.max_tank_track_opaque_edges);
    h.i32(g.max_tank_track_fade_delay);
    h.ascii(&g.level_gain_animation_name);
    h.f32(g.level_gain_animation_display_time_in_seconds);
    h.f32(g.level_gain_animation_z_rise_per_second);
    h.ascii(&g.get_healed_animation_name);
    h.f32(g.get_healed_animation_display_time_in_seconds);
    h.f32(g.get_healed_animation_z_rise_per_second);
    h.i32(g.time_of_day.as_i32());
    h.i32(g.weather.as_i32());
    h.u8(g.make_track_marks);
    h.u8(g.hide_garrison_flags);
    h.u8(g.force_models_to_follow_time_of_day);
    h.u8(g.force_models_to_follow_weather);

    for t in 0..TIME_OF_DAY_COUNT {
        for l in 0..MAX_GLOBAL_LIGHTS {
            h.rgb(&g.terrain_lighting[t][l].ambient);
            h.rgb(&g.terrain_lighting[t][l].diffuse);
            h.coord3d(&g.terrain_lighting[t][l].light_pos);
            h.rgb(&g.terrain_objects_lighting[t][l].ambient);
            h.rgb(&g.terrain_objects_lighting[t][l].diffuse);
            h.coord3d(&g.terrain_objects_lighting[t][l].light_pos);
        }
    }

    for l in 0..MAX_GLOBAL_LIGHTS {
        h.rgb(&g.terrain_ambient[l]);
        h.rgb(&g.terrain_diffuse[l]);
        h.coord3d(&g.terrain_light_pos[l]);
    }

    for t in (TIME_OF_DAY_FIRST as usize)..TIME_OF_DAY_COUNT {
        h.f32(g.infantry_light_scale[t]);
    }
    h.f32(g.script_override_infantry_light_scale);

    for p in 0..PLAYERTYPE_COUNT {
        for d in 0..DIFFICULTY_COUNT {
            h.f32(g.solo_player_health_bonus_for_difficulty[p][d]);
        }
    }

    h.i32(g.max_visible_translucent_objects);
    h.i32(g.max_visible_occluder_objects);
    h.i32(g.max_visible_occludee_objects);
    h.i32(g.max_visible_non_occluder_or_occludee_objects);
    h.f32(g.occluded_luminance_scale);
    h.i32(g.num_global_lights);
    h.i32(g.max_road_segments);
    h.i32(g.max_road_vertex);
    h.i32(g.max_road_index);
    h.i32(g.max_road_types);
    h.u8(g.audio_on);
    h.u8(g.music_on);
    h.u8(g.sounds_on);
    h.u8(g.sounds_3d_on);
    h.u8(g.speech_on);
    h.u8(g.video_on);
    h.u8(g.disable_camera_movement);
    h.u8(g.use_fx);
    h.u8(g.show_client_physics);
    h.u8(g.show_terrain_normals);
    h.u32(g.no_draw);
    h.i32(g.debug_ai);
    h.u8(g.debug_supply_center_placement);
    h.u8(g.debug_ai_obstacles);
    h.u8(g.show_object_health);
    h.u8(g.script_debug);
    h.u8(g.particle_edit);
    h.u8(g.display_debug);
    h.u8(g.win_cursors);
    h.u8(g.constant_debug_update);
    h.u8(g.show_team_dot);
    h.u8(g.force_benchmark);
    h.i32(g.fixed_seed);
    h.f32(g.particle_scale);
    h.ascii(&g.auto_fire_particle_small_prefix);
    h.ascii(&g.auto_fire_particle_small_system);
    h.i32(g.auto_fire_particle_small_max);
    h.ascii(&g.auto_fire_particle_medium_prefix);
    h.ascii(&g.auto_fire_particle_medium_system);
    h.i32(g.auto_fire_particle_medium_max);
    h.ascii(&g.auto_fire_particle_large_prefix);
    h.ascii(&g.auto_fire_particle_large_system);
    h.i32(g.auto_fire_particle_large_max);
    h.ascii(&g.auto_smoke_particle_small_prefix);
    h.ascii(&g.auto_smoke_particle_small_system);
    h.i32(g.auto_smoke_particle_small_max);
    h.ascii(&g.auto_smoke_particle_medium_prefix);
    h.ascii(&g.auto_smoke_particle_medium_system);
    h.i32(g.auto_smoke_particle_medium_max);
    h.ascii(&g.auto_smoke_particle_large_prefix);
    h.ascii(&g.auto_smoke_particle_large_system);
    h.i32(g.auto_smoke_particle_large_max);
    h.ascii(&g.auto_aflame_particle_prefix);
    h.ascii(&g.auto_aflame_particle_system);
    h.i32(g.auto_aflame_particle_max);
    h.i32(g.net_min_players);
    h.u32(g.default_ip);
    h.u32(g.firewall_behavior);
    h.u32(g.firewall_port_override);
    h.i16(g.firewall_port_allocation_delta);
    h.i32(g.base_value_per_supply_box);
    h.f32(g.build_speed);
    h.f32(g.min_dist_from_edge_of_map_for_build);
    h.f32(g.supply_build_border);
    h.f32(g.allowed_height_variation_for_building);
    h.f32(g.min_low_energy_production_speed);
    h.f32(g.max_low_energy_production_speed);
    h.f32(g.low_energy_penalty_modifier);
    h.f32(g.multiple_factory);
    h.f32(g.refund_percent);
    h.f32(g.command_center_heal_range);
    h.f32(g.command_center_heal_amount);
    h.i32(g.max_line_build_objects);
    h.i32(g.max_tunnel_capacity);
    h.f32(g.horizontal_scroll_speed_factor);
    h.f32(g.vertical_scroll_speed_factor);
    h.f32(g.scroll_amount_cutoff);
    h.f32(g.camera_adjust_speed);
    h.u8(g.enforce_max_camera_height);
    h.u8(g.build_map_cache);
    h.ascii(&g.initial_file);
    h.ascii(&g.pending_file);
    h.ascii(&g.load_save_game);
    h.ascii(&g.load_replay_game);
    h.u32(g.simulate_replays.len() as u32);
    h.i32(g.simulate_replay_jobs);
    h.i32(g.max_particle_count);
    h.i32(g.max_field_particle_count);

    h.u8(if g.weapon_bonus_set.is_some() { 1 } else { 0 });
    if let Some(ref wbs) = g.weapon_bonus_set {
        for c in 0..WEAPON_BONUS_CONDITION_COUNT {
            for f in 0..WEAPON_BONUS_FIELD_COUNT {
                let wf = WeaponBonusField::from_i32(f as Int).unwrap();
                h.f32(wbs.entries[c].get_field(wf));
            }
        }
    }

    for i in 0..LEVEL_COUNT {
        h.f32(g.health_bonus[i]);
    }
    h.f32(g.default_structure_rubble_height);
    h.ascii(&g.shell_map_name);
    h.u8(g.shell_map_on);
    h.u8(g.play_intro);
    h.u8(g.play_sizzle);
    h.u8(g.load_screen_render);
    h.f32(g.keyboard_scroll_factor);
    h.f32(g.mouse_scroll_factor);
    h.f32(g.keyboard_default_scroll_factor);
    h.u8(g.draw_scroll_anchor);
    h.u8(g.move_scroll_anchor);
    h.u8(g.animate_windows);
    h.u8(g.incremental_agp_buf);
    h.u32(g.ini_crc);
    h.u32(g.exe_crc);
    h.i32(g.movement_penalty_damage_state.as_i32());
    h.i32(g.group_select_min_select_size);
    h.f32(g.group_select_volume_base);
    h.f32(g.group_select_volume_increment);
    h.i32(g.max_unit_select_sounds);
    h.f32(g.selection_flash_saturation_factor);
    h.u8(g.selection_flash_house_color);
    h.f32(g.camera_audible_radius);
    h.f32(g.group_move_click_to_gather_factor);
    h.u32(g.anti_alias_level);
    h.u32(g.texture_filtering_mode);
    h.u32(g.texture_anisotropy_level);
    h.u8(g.language_filter_pref);
    h.u8(g.load_screen_demo);
    h.u8(g.disable_render);
    h.u8(g.save_camera_in_replay);
    h.u8(g.use_camera_in_replay);
    h.u8(g.enable_player_observer);
    h.i32(g.network_latency_font_size);
    h.i32(g.render_fps_font_size);
    h.i32(g.system_time_font_size);
    h.i32(g.game_time_font_size);
    h.i32(g.player_info_list_font_size);
    h.u8(g.show_money_per_minute);
    h.u8(g.allow_money_per_minute_for_player);
    h.f32(g.game_window_transition_speed_multiplier);
    h.f32(g.shake_subtle_intensity);
    h.f32(g.shake_normal_intensity);
    h.f32(g.shake_strong_intensity);
    h.f32(g.shake_severe_intensity);
    h.f32(g.shake_cine_extreme_intensity);
    h.f32(g.shake_cine_insane_intensity);
    h.f32(g.max_shake_intensity);
    h.f32(g.max_shake_range);
    h.f32(g.sell_percentage);
    h.f32(g.base_regen_health_percent_per_second);
    h.u32(g.base_regen_delay);
    h.u32(g.hot_key_text_color);
    h.ascii(&g.special_power_view_object_name);
    h.f32(g.object_placement_opacity);
    h.u8(g.object_placement_shadows);
    h.u32(g.standard_public_bones.len() as u32);
    for b in &g.standard_public_bones {
        h.ascii(b);
    }
    h.f32(g.standard_minefield_density);
    h.f32(g.standard_minefield_distance);
    h.u8(g.show_metrics);
    h.u32(g.default_starting_cash.count_money());
    h.u8(g.debug_show_graphical_framerate);
    h.i32(g.power_bar_base);
    h.f32(g.power_bar_intervals);
    h.i32(g.power_bar_yellow_range);
    h.f32(g.display_gamma);
    h.u32(g.unlook_persist_duration);
    h.u8(g.should_update_tga_to_dds);
    h.u32(g.double_click_time_ms);
    h.rgb(&g.shroud_color);
    h.u8(g.clear_alpha);
    h.u8(g.fog_alpha);
    h.u8(g.shroud_alpha);
    h.u32(g.network_fps_history_length);
    h.u32(g.network_latency_history_length);
    h.u32(g.network_cushion_history_length);
    h.u32(g.network_run_ahead_metrics_time);
    h.u32(g.network_keep_alive_delay);
    h.u32(g.network_run_ahead_slack);
    h.u32(g.network_disconnect_time);
    h.u32(g.network_player_timeout_time);
    h.u32(g.network_disconnect_screen_notify_time);
    h.f32(g.keyboard_camera_rotate_speed);
    h.i32(g.play_stats);
    h.u8(g.tivo_fast_mode);
    h.u8(g.shroud_on);
    h.u8(g.is_breakable_movie);
    h.u8(g.break_the_movie);
    h.ascii(&g.mod_dir);
    h.ascii(&g.mod_big);
    h.ascii(&g.asset_path);
    h.ascii(&g.generals_asset_path);
    h.u8(g.loose_files_win);
    h.finish()
}

fn compute_getters_mask(g: &GlobalData) -> u32 {
    let mut m = 0u32;
    if g.is_use_shadow_volumes_for_skins() != FALSE { m |= 1 << 0; }
    if g.is_shadows_for_projectiles() != FALSE { m |= 1 << 1; }
    if g.is_shadows_for_props() != FALSE { m |= 1 << 2; }
    if g.is_extended_shadow_frustum_culling() != FALSE { m |= 1 << 3; }
    if g.is_natural_sun_elevation() != FALSE { m |= 1 << 4; }
    if g.is_bridge_shadows() != FALSE { m |= 1 << 5; }
    if g.is_water_reflections() != FALSE { m |= 1 << 6; }
    if g.is_explosion_dynamic_lights() != FALSE { m |= 1 << 7; }
    if g.is_vehicle_headlights() != FALSE { m |= 1 << 8; }
    if g.is_enable_post_processing() != FALSE { m |= 1 << 9; }
    if g.is_post_process_bloom() != FALSE { m |= 1 << 10; }
    if g.is_post_process_hdr() != FALSE { m |= 1 << 11; }
    if g.is_post_process_fxaa() != FALSE { m |= 1 << 12; }
    if g.is_air_unit_ground_indicator() != FALSE { m |= 1 << 13; }
    m
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldKind {
    Bool,
    Int,
    U8,
    Real,
    AngleReal,
    AccelReal,
    Percent,
    DurationUint,
    Ascii,
    AsciiVecAppend,
    Rgb,
    ColorInt,
    Coord2D,
    Coord3D,
    TerrainLod,
    TimeOfDay,
    Weather,
    BodyDamage,
    Money,
    WeaponBonus,
    ShortInt,
}

const RELEASE_FIELDS: [(&str, FieldKind); 360] = [
    ("Windowed", FieldKind::Bool),
    ("CheckForUpdates", FieldKind::Bool),
    ("XResolution", FieldKind::Int),
    ("YResolution", FieldKind::Int),
    ("MapName", FieldKind::Ascii),
    ("MoveHintName", FieldKind::Ascii),
    ("UseTrees", FieldKind::Bool),
    ("UseFPSLimit", FieldKind::Bool),
    ("DumpAssetUsage", FieldKind::Bool),
    ("FramesPerSecondLimit", FieldKind::Int),
    ("SkirmishTickRate", FieldKind::Int),
    ("TickRate", FieldKind::Int),
    ("ChipsetType", FieldKind::Int),
    ("MaxShellScreens", FieldKind::Int),
    ("UseCloudMap", FieldKind::Bool),
    ("UseLightMap", FieldKind::Bool),
    ("BilinearTerrainTex", FieldKind::Bool),
    ("TrilinearTerrainTex", FieldKind::Bool),
    ("MultiPassTerrain", FieldKind::Bool),
    ("AdjustCliffTextures", FieldKind::Bool),
    ("Use3WayTerrainBlends", FieldKind::Int),
    ("StretchTerrain", FieldKind::Bool),
    ("UseHalfHeightMap", FieldKind::Bool),
    ("DrawEntireTerrain", FieldKind::Bool),
    ("TerrainLOD", FieldKind::TerrainLod),
    ("TerrainLODTargetTimeMS", FieldKind::Int),
    ("RightMouseAlwaysScrolls", FieldKind::Bool),
    ("UseWaterPlane", FieldKind::Bool),
    ("UseCloudPlane", FieldKind::Bool),
    ("DownwindAngle", FieldKind::Real),
    ("UseShadowVolumes", FieldKind::Bool),
    ("UseShadowVolumesForSkins", FieldKind::Bool),
    ("ShadowsForProjectiles", FieldKind::Bool),
    ("ShadowsForProps", FieldKind::Bool),
    ("AdvancedShadows", FieldKind::Bool),
    ("AdvancedLighting", FieldKind::Bool),
    ("ExtendedShadowFrustumCulling", FieldKind::Bool),
    ("NaturalSunElevation", FieldKind::Bool),
    ("BridgeShadows", FieldKind::Bool),
    ("WaterReflections", FieldKind::Bool),
    ("ExplosionDynamicLights", FieldKind::Bool),
    ("VehicleHeadlights", FieldKind::Bool),
    ("EnablePostProcessing", FieldKind::Bool),
    ("PostProcessBloom", FieldKind::Bool),
    ("PostProcessHDR", FieldKind::Bool),
    ("PostProcessFXAA", FieldKind::Bool),
    ("AirUnitGroundIndicator", FieldKind::Bool),
    ("UseShadowDecals", FieldKind::Bool),
    ("TextureReductionFactor", FieldKind::Int),
    ("UseBehindBuildingMarker", FieldKind::Bool),
    ("WaterPositionX", FieldKind::Real),
    ("WaterPositionY", FieldKind::Real),
    ("WaterPositionZ", FieldKind::Real),
    ("WaterExtentX", FieldKind::Real),
    ("WaterExtentY", FieldKind::Real),
    ("WaterType", FieldKind::Int),
    ("FeatherWater", FieldKind::Int),
    ("ShowSoftWaterEdge", FieldKind::Bool),
    ("VertexWaterAvailableMaps1", FieldKind::Ascii),
    ("VertexWaterHeightClampLow1", FieldKind::Real),
    ("VertexWaterHeightClampHi1", FieldKind::Real),
    ("VertexWaterAngle1", FieldKind::AngleReal),
    ("VertexWaterXPosition1", FieldKind::Real),
    ("VertexWaterYPosition1", FieldKind::Real),
    ("VertexWaterZPosition1", FieldKind::Real),
    ("VertexWaterXGridCells1", FieldKind::Int),
    ("VertexWaterYGridCells1", FieldKind::Int),
    ("VertexWaterGridSize1", FieldKind::Real),
    ("VertexWaterAttenuationA1", FieldKind::Real),
    ("VertexWaterAttenuationB1", FieldKind::Real),
    ("VertexWaterAttenuationC1", FieldKind::Real),
    ("VertexWaterAttenuationRange1", FieldKind::Real),
    ("VertexWaterAvailableMaps2", FieldKind::Ascii),
    ("VertexWaterHeightClampLow2", FieldKind::Real),
    ("VertexWaterHeightClampHi2", FieldKind::Real),
    ("VertexWaterAngle2", FieldKind::AngleReal),
    ("VertexWaterXPosition2", FieldKind::Real),
    ("VertexWaterYPosition2", FieldKind::Real),
    ("VertexWaterZPosition2", FieldKind::Real),
    ("VertexWaterXGridCells2", FieldKind::Int),
    ("VertexWaterYGridCells2", FieldKind::Int),
    ("VertexWaterGridSize2", FieldKind::Real),
    ("VertexWaterAttenuationA2", FieldKind::Real),
    ("VertexWaterAttenuationB2", FieldKind::Real),
    ("VertexWaterAttenuationC2", FieldKind::Real),
    ("VertexWaterAttenuationRange2", FieldKind::Real),
    ("VertexWaterAvailableMaps3", FieldKind::Ascii),
    ("VertexWaterHeightClampLow3", FieldKind::Real),
    ("VertexWaterHeightClampHi3", FieldKind::Real),
    ("VertexWaterAngle3", FieldKind::AngleReal),
    ("VertexWaterXPosition3", FieldKind::Real),
    ("VertexWaterYPosition3", FieldKind::Real),
    ("VertexWaterZPosition3", FieldKind::Real),
    ("VertexWaterXGridCells3", FieldKind::Int),
    ("VertexWaterYGridCells3", FieldKind::Int),
    ("VertexWaterGridSize3", FieldKind::Real),
    ("VertexWaterAttenuationA3", FieldKind::Real),
    ("VertexWaterAttenuationB3", FieldKind::Real),
    ("VertexWaterAttenuationC3", FieldKind::Real),
    ("VertexWaterAttenuationRange3", FieldKind::Real),
    ("VertexWaterAvailableMaps4", FieldKind::Ascii),
    ("VertexWaterHeightClampLow4", FieldKind::Real),
    ("VertexWaterHeightClampHi4", FieldKind::Real),
    ("VertexWaterAngle4", FieldKind::AngleReal),
    ("VertexWaterXPosition4", FieldKind::Real),
    ("VertexWaterYPosition4", FieldKind::Real),
    ("VertexWaterZPosition4", FieldKind::Real),
    ("VertexWaterXGridCells4", FieldKind::Int),
    ("VertexWaterYGridCells4", FieldKind::Int),
    ("VertexWaterGridSize4", FieldKind::Real),
    ("VertexWaterAttenuationA4", FieldKind::Real),
    ("VertexWaterAttenuationB4", FieldKind::Real),
    ("VertexWaterAttenuationC4", FieldKind::Real),
    ("VertexWaterAttenuationRange4", FieldKind::Real),
    ("SkyBoxPositionZ", FieldKind::Real),
    ("SkyBoxScale", FieldKind::Real),
    ("DrawSkyBox", FieldKind::Bool),
    ("ViewportHeightScale", FieldKind::Real),
    ("CameraPitch", FieldKind::Real),
    ("CameraYaw", FieldKind::Real),
    ("CameraHeight", FieldKind::Real),
    ("MaxCameraHeight", FieldKind::Real),
    ("MinCameraHeight", FieldKind::Real),
    ("TerrainDrawDistanceScale", FieldKind::Real),
    ("TerrainHeightAtEdgeOfMap", FieldKind::Real),
    ("UnitDamagedThreshold", FieldKind::Real),
    ("UnitReallyDamagedThreshold", FieldKind::Real),
    ("GroundStiffness", FieldKind::Real),
    ("StructureStiffness", FieldKind::Real),
    ("Gravity", FieldKind::AccelReal),
    ("StealthFriendlyOpacity", FieldKind::Percent),
    ("DefaultOcclusionDelay", FieldKind::DurationUint),
    ("PartitionCellSize", FieldKind::Real),
    ("AmmoPipScaleFactor", FieldKind::Real),
    ("ContainerPipScaleFactor", FieldKind::Real),
    ("AmmoPipWorldOffset", FieldKind::Coord3D),
    ("ContainerPipWorldOffset", FieldKind::Coord3D),
    ("AmmoPipScreenOffset", FieldKind::Coord2D),
    ("ContainerPipScreenOffset", FieldKind::Coord2D),
    ("HistoricDamageLimit", FieldKind::DurationUint),
    ("MaxTerrainTracks", FieldKind::Int),
    ("TimeOfDay", FieldKind::TimeOfDay),
    ("Weather", FieldKind::Weather),
    ("MakeTrackMarks", FieldKind::Bool),
    ("HideGarrisonFlags", FieldKind::Bool),
    ("ForceModelsToFollowTimeOfDay", FieldKind::Bool),
    ("ForceModelsToFollowWeather", FieldKind::Bool),
    ("LevelGainAnimationName", FieldKind::Ascii),
    ("LevelGainAnimationTime", FieldKind::Real),
    ("LevelGainAnimationZRise", FieldKind::Real),
    ("GetHealedAnimationName", FieldKind::Ascii),
    ("GetHealedAnimationTime", FieldKind::Real),
    ("GetHealedAnimationZRise", FieldKind::Real),
    ("TerrainLightingMorningAmbient", FieldKind::Rgb),
    ("TerrainLightingMorningDiffuse", FieldKind::Rgb),
    ("TerrainLightingMorningLightPos", FieldKind::Coord3D),
    ("TerrainLightingAfternoonAmbient", FieldKind::Rgb),
    ("TerrainLightingAfternoonDiffuse", FieldKind::Rgb),
    ("TerrainLightingAfternoonLightPos", FieldKind::Coord3D),
    ("TerrainLightingEveningAmbient", FieldKind::Rgb),
    ("TerrainLightingEveningDiffuse", FieldKind::Rgb),
    ("TerrainLightingEveningLightPos", FieldKind::Coord3D),
    ("TerrainLightingNightAmbient", FieldKind::Rgb),
    ("TerrainLightingNightDiffuse", FieldKind::Rgb),
    ("TerrainLightingNightLightPos", FieldKind::Coord3D),
    ("TerrainObjectsLightingMorningAmbient", FieldKind::Rgb),
    ("TerrainObjectsLightingMorningDiffuse", FieldKind::Rgb),
    ("TerrainObjectsLightingMorningLightPos", FieldKind::Coord3D),
    ("TerrainObjectsLightingAfternoonAmbient", FieldKind::Rgb),
    ("TerrainObjectsLightingAfternoonDiffuse", FieldKind::Rgb),
    ("TerrainObjectsLightingAfternoonLightPos", FieldKind::Coord3D),
    ("TerrainObjectsLightingEveningAmbient", FieldKind::Rgb),
    ("TerrainObjectsLightingEveningDiffuse", FieldKind::Rgb),
    ("TerrainObjectsLightingEveningLightPos", FieldKind::Coord3D),
    ("TerrainObjectsLightingNightAmbient", FieldKind::Rgb),
    ("TerrainObjectsLightingNightDiffuse", FieldKind::Rgb),
    ("TerrainObjectsLightingNightLightPos", FieldKind::Coord3D),
    ("TerrainLightingMorningAmbient2", FieldKind::Rgb),
    ("TerrainLightingMorningDiffuse2", FieldKind::Rgb),
    ("TerrainLightingMorningLightPos2", FieldKind::Coord3D),
    ("TerrainLightingAfternoonAmbient2", FieldKind::Rgb),
    ("TerrainLightingAfternoonDiffuse2", FieldKind::Rgb),
    ("TerrainLightingAfternoonLightPos2", FieldKind::Coord3D),
    ("TerrainLightingEveningAmbient2", FieldKind::Rgb),
    ("TerrainLightingEveningDiffuse2", FieldKind::Rgb),
    ("TerrainLightingEveningLightPos2", FieldKind::Coord3D),
    ("TerrainLightingNightAmbient2", FieldKind::Rgb),
    ("TerrainLightingNightDiffuse2", FieldKind::Rgb),
    ("TerrainLightingNightLightPos2", FieldKind::Coord3D),
    ("TerrainObjectsLightingMorningAmbient2", FieldKind::Rgb),
    ("TerrainObjectsLightingMorningDiffuse2", FieldKind::Rgb),
    ("TerrainObjectsLightingMorningLightPos2", FieldKind::Coord3D),
    ("TerrainObjectsLightingAfternoonAmbient2", FieldKind::Rgb),
    ("TerrainObjectsLightingAfternoonDiffuse2", FieldKind::Rgb),
    ("TerrainObjectsLightingAfternoonLightPos2", FieldKind::Coord3D),
    ("TerrainObjectsLightingEveningAmbient2", FieldKind::Rgb),
    ("TerrainObjectsLightingEveningDiffuse2", FieldKind::Rgb),
    ("TerrainObjectsLightingEveningLightPos2", FieldKind::Coord3D),
    ("TerrainObjectsLightingNightAmbient2", FieldKind::Rgb),
    ("TerrainObjectsLightingNightDiffuse2", FieldKind::Rgb),
    ("TerrainObjectsLightingNightLightPos2", FieldKind::Coord3D),
    ("TerrainLightingMorningAmbient3", FieldKind::Rgb),
    ("TerrainLightingMorningDiffuse3", FieldKind::Rgb),
    ("TerrainLightingMorningLightPos3", FieldKind::Coord3D),
    ("TerrainLightingAfternoonAmbient3", FieldKind::Rgb),
    ("TerrainLightingAfternoonDiffuse3", FieldKind::Rgb),
    ("TerrainLightingAfternoonLightPos3", FieldKind::Coord3D),
    ("TerrainLightingEveningAmbient3", FieldKind::Rgb),
    ("TerrainLightingEveningDiffuse3", FieldKind::Rgb),
    ("TerrainLightingEveningLightPos3", FieldKind::Coord3D),
    ("TerrainLightingNightAmbient3", FieldKind::Rgb),
    ("TerrainLightingNightDiffuse3", FieldKind::Rgb),
    ("TerrainLightingNightLightPos3", FieldKind::Coord3D),
    ("TerrainObjectsLightingMorningAmbient3", FieldKind::Rgb),
    ("TerrainObjectsLightingMorningDiffuse3", FieldKind::Rgb),
    ("TerrainObjectsLightingMorningLightPos3", FieldKind::Coord3D),
    ("TerrainObjectsLightingAfternoonAmbient3", FieldKind::Rgb),
    ("TerrainObjectsLightingAfternoonDiffuse3", FieldKind::Rgb),
    ("TerrainObjectsLightingAfternoonLightPos3", FieldKind::Coord3D),
    ("TerrainObjectsLightingEveningAmbient3", FieldKind::Rgb),
    ("TerrainObjectsLightingEveningDiffuse3", FieldKind::Rgb),
    ("TerrainObjectsLightingEveningLightPos3", FieldKind::Coord3D),
    ("TerrainObjectsLightingNightAmbient3", FieldKind::Rgb),
    ("TerrainObjectsLightingNightDiffuse3", FieldKind::Rgb),
    ("TerrainObjectsLightingNightLightPos3", FieldKind::Coord3D),
    ("NumberGlobalLights", FieldKind::Int),
    ("InfantryLightMorningScale", FieldKind::Real),
    ("InfantryLightAfternoonScale", FieldKind::Real),
    ("InfantryLightEveningScale", FieldKind::Real),
    ("InfantryLightNightScale", FieldKind::Real),
    ("MaxTranslucentObjects", FieldKind::Int),
    ("OccludedColorLuminanceScale", FieldKind::Real),
    ("MaxRoadSegments", FieldKind::Int),
    ("MaxRoadVertex", FieldKind::Int),
    ("MaxRoadIndex", FieldKind::Int),
    ("MaxRoadTypes", FieldKind::Int),
    ("ValuePerSupplyBox", FieldKind::Int),
    ("AudioOn", FieldKind::Bool),
    ("MusicOn", FieldKind::Bool),
    ("SoundsOn", FieldKind::Bool),
    ("Sounds3DOn", FieldKind::Bool),
    ("SpeechOn", FieldKind::Bool),
    ("VideoOn", FieldKind::Bool),
    ("DisableCameraMovements", FieldKind::Bool),
    ("DebugAI", FieldKind::Bool),
    ("DebugAIObstacles", FieldKind::Bool),
    ("ShowClientPhysics", FieldKind::Bool),
    ("ShowTerrainNormals", FieldKind::Bool),
    ("ShowObjectHealth", FieldKind::Bool),
    ("ParticleScale", FieldKind::Real),
    ("AutoFireParticleSmallPrefix", FieldKind::Ascii),
    ("AutoFireParticleSmallSystem", FieldKind::Ascii),
    ("AutoFireParticleSmallMax", FieldKind::Int),
    ("AutoFireParticleMediumPrefix", FieldKind::Ascii),
    ("AutoFireParticleMediumSystem", FieldKind::Ascii),
    ("AutoFireParticleMediumMax", FieldKind::Int),
    ("AutoFireParticleLargePrefix", FieldKind::Ascii),
    ("AutoFireParticleLargeSystem", FieldKind::Ascii),
    ("AutoFireParticleLargeMax", FieldKind::Int),
    ("AutoSmokeParticleSmallPrefix", FieldKind::Ascii),
    ("AutoSmokeParticleSmallSystem", FieldKind::Ascii),
    ("AutoSmokeParticleSmallMax", FieldKind::Int),
    ("AutoSmokeParticleMediumPrefix", FieldKind::Ascii),
    ("AutoSmokeParticleMediumSystem", FieldKind::Ascii),
    ("AutoSmokeParticleMediumMax", FieldKind::Int),
    ("AutoSmokeParticleLargePrefix", FieldKind::Ascii),
    ("AutoSmokeParticleLargeSystem", FieldKind::Ascii),
    ("AutoSmokeParticleLargeMax", FieldKind::Int),
    ("AutoAflameParticlePrefix", FieldKind::Ascii),
    ("AutoAflameParticleSystem", FieldKind::Ascii),
    ("AutoAflameParticleMax", FieldKind::Int),
    ("BuildSpeed", FieldKind::Real),
    ("MinDistFromEdgeOfMapForBuild", FieldKind::Real),
    ("SupplyBuildBorder", FieldKind::Real),
    ("AllowedHeightVariationForBuilding", FieldKind::Real),
    ("MinLowEnergyProductionSpeed", FieldKind::Real),
    ("MaxLowEnergyProductionSpeed", FieldKind::Real),
    ("LowEnergyPenaltyModifier", FieldKind::Real),
    ("MultipleFactory", FieldKind::Real),
    ("RefundPercent", FieldKind::Percent),
    ("CommandCenterHealRange", FieldKind::Real),
    ("CommandCenterHealAmount", FieldKind::Real),
    ("StandardMinefieldDensity", FieldKind::Real),
    ("StandardMinefieldDistance", FieldKind::Real),
    ("MaxLineBuildObjects", FieldKind::Int),
    ("MaxTunnelCapacity", FieldKind::Int),
    ("MaxParticleCount", FieldKind::Int),
    ("MaxFieldParticleCount", FieldKind::Int),
    ("HorizontalScrollSpeedFactor", FieldKind::Real),
    ("VerticalScrollSpeedFactor", FieldKind::Real),
    ("ScrollAmountCutoff", FieldKind::Real),
    ("CameraAdjustSpeed", FieldKind::Real),
    ("EnforceMaxCameraHeight", FieldKind::Bool),
    ("KeyboardScrollSpeedFactor", FieldKind::Real),
    ("MouseScrollSpeedFactor", FieldKind::Real),
    ("KeyboardDefaultScrollSpeedFactor", FieldKind::Real),
    ("KeyboardCameraRotateSpeed", FieldKind::Real),
    ("MovementPenaltyDamageState", FieldKind::BodyDamage),
    ("HealthBonus_Veteran", FieldKind::Percent),
    ("HealthBonus_Elite", FieldKind::Percent),
    ("HealthBonus_Heroic", FieldKind::Percent),
    ("HumanSoloPlayerHealthBonus_Easy", FieldKind::Percent),
    ("HumanSoloPlayerHealthBonus_Normal", FieldKind::Percent),
    ("HumanSoloPlayerHealthBonus_Hard", FieldKind::Percent),
    ("AISoloPlayerHealthBonus_Easy", FieldKind::Percent),
    ("AISoloPlayerHealthBonus_Normal", FieldKind::Percent),
    ("AISoloPlayerHealthBonus_Hard", FieldKind::Percent),
    ("WeaponBonus", FieldKind::WeaponBonus),
    ("DefaultStructureRubbleHeight", FieldKind::Real),
    ("FixedSeed", FieldKind::Int),
    ("ShellMapName", FieldKind::Ascii),
    ("ShellMapOn", FieldKind::Bool),
    ("PlayIntro", FieldKind::Bool),
    ("FirewallBehavior", FieldKind::Int),
    ("FirewallPortOverride", FieldKind::Int),
    ("FirewallPortAllocationDelta", FieldKind::ShortInt),
    ("GroupSelectMinSelectSize", FieldKind::Int),
    ("GroupSelectVolumeBase", FieldKind::Real),
    ("GroupSelectVolumeIncrement", FieldKind::Real),
    ("MaxUnitSelectSounds", FieldKind::Int),
    ("SelectionFlashSaturationFactor", FieldKind::Real),
    ("SelectionFlashHouseColor", FieldKind::Bool),
    ("CameraAudibleRadius", FieldKind::Real),
    ("GroupMoveClickToGatherAreaFactor", FieldKind::Real),
    ("ShakeSubtleIntensity", FieldKind::Real),
    ("ShakeNormalIntensity", FieldKind::Real),
    ("ShakeStrongIntensity", FieldKind::Real),
    ("ShakeSevereIntensity", FieldKind::Real),
    ("ShakeCineExtremeIntensity", FieldKind::Real),
    ("ShakeCineInsaneIntensity", FieldKind::Real),
    ("MaxShakeIntensity", FieldKind::Real),
    ("MaxShakeRange", FieldKind::Real),
    ("SellPercentage", FieldKind::Percent),
    ("BaseRegenHealthPercentPerSecond", FieldKind::Percent),
    ("BaseRegenDelay", FieldKind::DurationUint),
    ("SpecialPowerViewObject", FieldKind::Ascii),
    ("ObjectPlacementOpacity", FieldKind::Real),
    ("ObjectPlacementShadows", FieldKind::Bool),
    ("StandardPublicBone", FieldKind::AsciiVecAppend),
    ("ShowMetrics", FieldKind::Bool),
    ("DefaultStartingCash", FieldKind::Money),
    ("ShroudColor", FieldKind::Rgb),
    ("ClearAlpha", FieldKind::U8),
    ("FogAlpha", FieldKind::U8),
    ("ShroudAlpha", FieldKind::U8),
    ("HotKeyTextColor", FieldKind::ColorInt),
    ("PowerBarBase", FieldKind::Int),
    ("PowerBarIntervals", FieldKind::Real),
    ("PowerBarYellowRange", FieldKind::Int),
    ("UnlookPersistDuration", FieldKind::DurationUint),
    ("NetworkFPSHistoryLength", FieldKind::Int),
    ("NetworkLatencyHistoryLength", FieldKind::Int),
    ("NetworkRunAheadMetricsTime", FieldKind::Int),
    ("NetworkCushionHistoryLength", FieldKind::Int),
    ("NetworkRunAheadSlack", FieldKind::Int),
    ("NetworkKeepAliveDelay", FieldKind::Int),
    ("NetworkDisconnectTime", FieldKind::Int),
    ("NetworkPlayerTimeoutTime", FieldKind::Int),
    ("NetworkDisconnectScreenNotifyTime", FieldKind::Int),
    ("PlayStats", FieldKind::Int),
];

fn fmt_quarter(q: i32) -> String {
    if q < 0 {
        let aq = (-q) as u32;
        format!("-{}.{:02}", aq / 4, (aq % 4) * 25)
    } else {
        let aq = q as u32;
        format!("{}.{:02}", aq / 4, (aq % 4) * 25)
    }
}

fn format_field_line(name: &str, kind: FieldKind, rng: &mut u32) -> String {
    match kind {
        FieldKind::Bool => {
            let v = (splitmix32(rng) & 1) != 0;
            format!("  {name} = {}\n", if v { "Yes" } else { "No" })
        }
        FieldKind::Int => {
            let v = (splitmix32(rng) % 2001) as i32 - 500;
            format!("  {name} = {v}\n")
        }
        FieldKind::ShortInt => {
            let v = (splitmix32(rng) % 40001) as i32 - 20000;
            format!("  {name} = {v}\n")
        }
        FieldKind::U8 => {
            let v = splitmix32(rng) % 256;
            format!("  {name} = {v}\n")
        }
        FieldKind::Real | FieldKind::AngleReal | FieldKind::AccelReal => {
            let q = (splitmix32(rng) % 2001) as i32 - 1000;
            format!("  {name} = {}\n", fmt_quarter(q))
        }
        FieldKind::Percent => {
            let p = splitmix32(rng) % 401;
            let with_sym = (splitmix32(rng) & 1) != 0;
            format!(
                "  {name} = {}{}\n",
                fmt_quarter(p as i32),
                if with_sym { "%" } else { "" }
            )
        }
        FieldKind::DurationUint => {
            let ms = splitmix32(rng) % 30000;
            format!("  {name} = {ms}\n")
        }
        FieldKind::Ascii => {
            let tag = splitmix32(rng) % 10000;
            format!("  {name} = Asset_{tag}_Path/Item\n")
        }
        FieldKind::AsciiVecAppend => {
            let a = splitmix32(rng) % 1000;
            let b = splitmix32(rng) % 1000;
            format!("  {name} = BONE_{a} BONE_{b}\n")
        }
        FieldKind::Rgb => {
            let r = splitmix32(rng) % 256;
            let g = splitmix32(rng) % 256;
            let b = splitmix32(rng) % 256;
            format!("  {name} = R:{r} G:{g} B:{b}\n")
        }
        FieldKind::ColorInt => {
            let r = splitmix32(rng) % 256;
            let g = splitmix32(rng) % 256;
            let b = splitmix32(rng) % 256;
            let with_a = (splitmix32(rng) & 1) != 0;
            if with_a {
                let a = splitmix32(rng) % 256;
                format!("  {name} = R:{r} G:{g} B:{b} A:{a}\n")
            } else {
                format!("  {name} = R:{r} G:{g} B:{b}\n")
            }
        }
        FieldKind::Coord2D => {
            let x = fmt_quarter((splitmix32(rng) % 801) as i32 - 400);
            let y = fmt_quarter((splitmix32(rng) % 801) as i32 - 400);
            format!("  {name} = X:{x} Y:{y}\n")
        }
        FieldKind::Coord3D => {
            let x = fmt_quarter((splitmix32(rng) % 801) as i32 - 400);
            let y = fmt_quarter((splitmix32(rng) % 801) as i32 - 400);
            let z = fmt_quarter((splitmix32(rng) % 801) as i32 - 400);
            format!("  {name} = X:{x} Y:{y} Z:{z}\n")
        }
        FieldKind::TerrainLod => {
            let idx = (splitmix32(rng) as usize) % TERRAIN_LOD_COUNT;
            format!("  {name} = {}\n", TERRAIN_LOD_NAMES[idx])
        }
        FieldKind::TimeOfDay => {
            let idx = (splitmix32(rng) as usize) % TIME_OF_DAY_COUNT;
            format!("  {name} = {}\n", TIME_OF_DAY_NAMES[idx])
        }
        FieldKind::Weather => {
            let idx = (splitmix32(rng) as usize) % WEATHER_COUNT;
            format!("  {name} = {}\n", WEATHER_NAMES[idx])
        }
        FieldKind::BodyDamage => {
            let idx = (splitmix32(rng) as usize) % BODY_DAMAGE_TYPE_COUNT;
            format!("  {name} = {}\n", THE_BODY_DAMAGE_TYPE_NAMES[idx])
        }
        FieldKind::Money => {
            let cash = splitmix32(rng) % 100000;
            format!("  {name} = {cash}\n")
        }
        FieldKind::WeaponBonus => {
            let c = (splitmix32(rng) as usize) % WEAPON_BONUS_CONDITION_COUNT;
            let f = (splitmix32(rng) as usize) % WEAPON_BONUS_FIELD_COUNT;
            let pct = (splitmix32(rng) % 300) + 10;
            format!(
                "  WeaponBonus = {} {} {pct}%\n",
                THE_WEAPON_BONUS_NAMES[c], THE_WEAPON_BONUS_FIELD_NAMES[f]
            )
        }
    }
}

struct SeedInputs {
    base_ini: String,
    override_ini: String,
    multi_ini: String,
    tod_arg: Int,
    bonus_flags: UnsignedInt,
    prefs: OptionPreferencesOverrides,
    has_fullviewport_dat: Bool,
    has_exe: bool,
    use_pattern_exe: bool,
    pattern_exe_bytes: u64,
    pattern_exe_seed: u32,
    exe_bytes: Vec<u8>,
    version_num: Option<UnsignedInt>,
    skirmish_scb: Option<Vec<u8>>,
    mp_scb: Option<Vec<u8>>,
}

fn build_seed_inputs(seed: u32) -> SeedInputs {
    let mut rng = seed ^ 0x474C_4F42;
    let mut base_ini = String::from("GameData\n");
    if seed == 0 {
        for &(name, kind) in &RELEASE_FIELDS {
            base_ini.push_str(&format_field_line(name, kind, &mut rng));
        }
        for _ in 0..6 {
            base_ini.push_str(&format_field_line("WeaponBonus", FieldKind::WeaponBonus, &mut rng));
        }
        base_ini.push_str(&format_field_line(
            "StandardPublicBone",
            FieldKind::AsciiVecAppend,
            &mut rng,
        ));
    } else {
        let count = 24 + (splitmix32(&mut rng) % 16);
        for _ in 0..count {
            let idx = ((seed as usize * 19) + (splitmix32(&mut rng) as usize % 360)) % 360;
            let (name, kind) = RELEASE_FIELDS[idx];
            base_ini.push_str(&format_field_line(name, kind, &mut rng));
        }
        let wb_n = 1 + (splitmix32(&mut rng) % 4);
        for _ in 0..wb_n {
            base_ini.push_str(&format_field_line("WeaponBonus", FieldKind::WeaponBonus, &mut rng));
        }
        if (seed & 3) == 0 {
            base_ini.push_str(&format_field_line("DrawSkyBox", FieldKind::Bool, &mut rng));
            base_ini.push_str(&format_field_line("DebugAI", FieldKind::Bool, &mut rng));
            base_ini.push_str(&format_field_line(
                "FirewallPortAllocationDelta",
                FieldKind::ShortInt,
                &mut rng,
            ));
        }
        let tick_key = if (seed & 1) == 0 { "SkirmishTickRate" } else { "TickRate" };
        base_ini.push_str(&format_field_line(tick_key, FieldKind::Int, &mut rng));
    }
    base_ini.push_str("End\n");

    const TOD_ARGS: [Int; 8] = [-1, 0, 1, 2, 3, 4, 5, 99];
    let tod_arg = TOD_ARGS[(seed as usize + (splitmix32(&mut rng) as usize % 3)) % TOD_ARGS.len()];

    let bonus_flags = match seed {
        0 => 0,
        1 => 0x07ff_ffff,
        2 => 0xf800_0000,
        _ => splitmix32(&mut rng),
    };

    let mut override_ini = String::from("GameData\n");
    let ov_count = 8 + (splitmix32(&mut rng) % 8);
    for _ in 0..ov_count {
        let mut idx = (splitmix32(&mut rng) as usize) % 360;
        if RELEASE_FIELDS[idx].1 == FieldKind::WeaponBonus {
            idx = (idx + 1) % 360;
        }
        let (name, kind) = RELEASE_FIELDS[idx];
        override_ini.push_str(&format_field_line(name, kind, &mut rng));
    }
    override_ini.push_str("End\n");

    let mut multi_ini = String::from("GameData\n");
    let mu_count = 6 + (splitmix32(&mut rng) % 6);
    for _ in 0..mu_count {
        let idx = (splitmix32(&mut rng) as usize) % 360;
        let (name, kind) = RELEASE_FIELDS[idx];
        multi_ini.push_str(&format_field_line(name, kind, &mut rng));
    }
    multi_ini.push_str("End\n");

    let mut prefs = OptionPreferencesOverrides::default();
    prefs.use_alternate_mouse = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.use_right_mouse_scroll_with_alternate_mouse =
        if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.client_retaliation_mode_enabled =
        if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.double_click_attack_move =
        if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.jpeg_quality = (50 + (splitmix32(&mut rng) % 46)) as Int;
    prefs.keyboard_scroll_factor = (1 + (splitmix32(&mut rng) % 200)) as Real * 0.01;
    prefs.draw_scroll_anchor = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.move_scroll_anchor = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.default_ip = splitmix32(&mut rng);
    prefs.firewall_behavior = (splitmix32(&mut rng) % 10) as Int;
    prefs.firewall_port_allocation_delta = ((splitmix32(&mut rng) % 20001) as Int - 10000) as Short;
    prefs.firewall_port_override = (splitmix32(&mut rng) % 65536) as UnsignedShort;
    prefs.save_camera_in_replay = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.use_camera_in_replay = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.enable_player_observer = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.network_latency_font_size = (splitmix32(&mut rng) % 24) as Int;
    prefs.render_fps_font_size = (splitmix32(&mut rng) % 24) as Int;
    prefs.system_time_font_size = (splitmix32(&mut rng) % 24) as Int;
    prefs.game_time_font_size = (splitmix32(&mut rng) % 24) as Int;
    prefs.player_info_list_font_size = (splitmix32(&mut rng) % 24) as Int;
    prefs.show_money_per_minute = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.game_window_transition_speed_multiplier =
        (1 + (splitmix32(&mut rng) % 50)) as Real * 0.25;

    let max_cam_bits: [u32; 9] = [
        0x7fc0_0000,
        0x7f80_0000,
        0xff80_0000,
        (-50.0_f32).to_bits(),
        50.0_f32.to_bits(),
        100.0_f32.to_bits(),
        450.0_f32.to_bits(),
        1000.0_f32.to_bits(),
        1250.5_f32.to_bits(),
    ];
    let has_max_cam = (splitmix32(&mut rng) % 5) != 0;
    let max_cam_val = f32::from_bits(max_cam_bits[(splitmix32(&mut rng) as usize) % 9]);
    prefs.max_camera_height = if has_max_cam { Some(max_cam_val) } else { None };

    let draw_dist_bits: [u32; 9] = [
        0x7fc0_0000,
        0x7f80_0000,
        0xff80_0000,
        (-1.0_f32).to_bits(),
        0.5_f32.to_bits(),
        1.0_f32.to_bits(),
        1.5_f32.to_bits(),
        2.0_f32.to_bits(),
        3.25_f32.to_bits(),
    ];
    let has_draw_dist = (splitmix32(&mut rng) % 5) != 0;
    let draw_dist_val = f32::from_bits(draw_dist_bits[(splitmix32(&mut rng) as usize) % 9]);
    prefs.terrain_draw_distance_scale = if has_draw_dist { Some(draw_dist_val) } else { None };

    prefs.skirmish_tick_rate = (5 + (splitmix32(&mut rng) % 116)) as Int;
    prefs.advanced_shadows = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.advanced_lighting = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.use_shadow_volumes_for_skins =
        if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.extended_shadow_frustum_culling =
        if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.natural_sun_elevation = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.shadows_for_projectiles = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.shadows_for_props = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.bridge_shadows = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.water_reflections = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.explosion_dynamic_lights = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.vehicle_headlights = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.enable_post_processing = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.post_process_bloom = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.post_process_hdr = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.post_process_fxaa = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };
    prefs.air_unit_ground_indicator =
        if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };

    const MSAA_MODES: [UnsignedInt; 4] = [0, 2, 4, 8];
    const ANISO_MODES: [UnsignedInt; 4] = [2, 4, 8, 16];
    prefs.anti_aliasing = MSAA_MODES[(splitmix32(&mut rng) as usize) % 4];
    prefs.texture_filter_mode = splitmix32(&mut rng) % 5;
    prefs.texture_anisotropy_level = ANISO_MODES[(splitmix32(&mut rng) as usize) % 4];

    const GAMMA_VALS: [Real; 12] =
        [-5.0, 0.0, 0.75, 1.0, 25.0, 49.5, 50.0, 50.9, 51.0, 75.0, 100.0, 120.0];
    prefs.gamma_value = GAMMA_VALS[(seed as usize + (splitmix32(&mut rng) as usize % 4)) % 12];
    prefs.x_resolution = (800 + (splitmix32(&mut rng) % 4) * 320) as Int;
    prefs.y_resolution = (600 + (splitmix32(&mut rng) % 4) * 240) as Int;
    let has_fullviewport_dat = if (splitmix32(&mut rng) & 1) != 0 { TRUE } else { FALSE };

    let has_exe: bool;
    let mut use_pattern_exe = false;
    let mut pattern_exe_bytes = 0u64;
    let mut pattern_exe_seed = 0u32;
    let mut exe_bytes = Vec::new();
    if seed == 999 {
        has_exe = true;
        use_pattern_exe = true;
        pattern_exe_bytes = 65536u64 * 1002 + 17;
        pattern_exe_seed = 0x55AA_1234;
    } else {
        has_exe = (splitmix32(&mut rng) % 8) != 0;
        if has_exe {
            const EXE_LENS: [usize; 9] = [0, 1, 17, 255, 4096, 65535, 65536, 65537, 131072];
            let len = EXE_LENS[(splitmix32(&mut rng) as usize) % EXE_LENS.len()];
            exe_bytes.reserve(len);
            for _ in 0..len {
                exe_bytes.push((splitmix32(&mut rng) & 0xff) as u8);
            }
        }
    }

    let has_version = (splitmix32(&mut rng) % 4) != 0;
    let ver_val = splitmix32(&mut rng);
    let version_num = if has_version { Some(ver_val) } else { None };

    let has_skirmish_scb = (splitmix32(&mut rng) % 3) != 0;
    let skirmish_scb = if has_skirmish_scb {
        let slen = (splitmix32(&mut rng) as usize % 256) + if seed % 7 == 0 { 70000 } else { 0 };
        let mut v = Vec::with_capacity(slen);
        for i in 0..slen {
            v.push(((i * 37 + seed as usize) & 0xff) as u8);
        }
        Some(v)
    } else {
        None
    };

    let has_mp_scb = (splitmix32(&mut rng) % 3) != 0;
    let mp_scb = if has_mp_scb {
        let mlen = (splitmix32(&mut rng) as usize % 256) + if seed % 11 == 0 { 66000 } else { 0 };
        let mut v = Vec::with_capacity(mlen);
        for i in 0..mlen {
            v.push(((i * 53 + seed as usize * 3) & 0xff) as u8);
        }
        Some(v)
    } else {
        None
    };

    SeedInputs {
        base_ini,
        override_ini,
        multi_ini,
        tod_arg,
        bonus_flags,
        prefs,
        has_fullviewport_dat,
        has_exe,
        use_pattern_exe,
        pattern_exe_bytes,
        pattern_exe_seed,
        exe_bytes,
        version_num,
        skirmish_scb,
        mp_scb,
    }
}

fn load_gamedata_text(
    store: &mut GlobalDataStore,
    ini_text: &str,
    load_type: IniLoadType,
    prefs: Option<&OptionPreferencesOverrides>,
) {
    let mut reader = IniReader::new("GameData.ini", ini_text.as_bytes(), load_type);
    reader
        .load_blocks(None, |block, r| {
            if block == "GameData" {
                store.parse_game_data_definition(r, prefs)?;
            }
            Ok(())
        })
        .expect("synthetic GameData INI must parse cleanly");
}

#[test]
fn test_pkt442_oracle_structure_and_constants() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-442\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("GlobalData.h"));
    assert!(VECTORS_JSON.contains("GlobalData.cpp"));
    assert!(VECTORS_JSON.contains("Weapon.cpp"));

    assert_eq!(TERRAIN_LOD_NAMES.len(), TERRAIN_LOD_COUNT);
    assert_eq!(TIME_OF_DAY_NAMES.len(), TIME_OF_DAY_COUNT);
    assert_eq!(WEATHER_NAMES.len(), WEATHER_COUNT);
    assert_eq!(THE_BODY_DAMAGE_TYPE_NAMES.len(), BODY_DAMAGE_TYPE_COUNT);
    assert_eq!(THE_WEAPON_BONUS_NAMES.len(), WEAPON_BONUS_CONDITION_COUNT);
    assert_eq!(THE_WEAPON_BONUS_FIELD_NAMES.len(), WEAPON_BONUS_FIELD_COUNT);
    assert_eq!(TerrainLod::Automatic.as_i32(), 5);
    assert_eq!(TimeOfDay::Afternoon.as_i32(), 2);
    assert_eq!(Weather::Snowy.as_i32(), 1);
    assert_eq!(BodyDamageType::ReallyDamaged.as_i32(), 2);

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);
        let inp = build_seed_inputs(seed);
        assert_eq!(
            crc_str(&inp.base_ini),
            get_u32(line, "base_ini_crc").unwrap(),
            "base_ini_crc mismatch at seed {seed}"
        );
        assert_eq!(
            crc_str(&inp.override_ini),
            get_u32(line, "override_ini_crc").unwrap(),
            "override_ini_crc mismatch at seed {seed}"
        );
        assert_eq!(
            crc_str(&inp.multi_ini),
            get_u32(line, "multi_ini_crc").unwrap(),
            "multi_ini_crc mismatch at seed {seed}"
        );
    }
}

#[test]
#[ignore = "PKT-442 GlobalData skeleton — remove #[ignore] once implemented"]
fn test_pkt442_defaults_and_unit_edge_cases() {
    let lines = extract_vector_lines(VECTORS_JSON);
    let expected_defaults = get_u32(lines[0], "defaults_crc").unwrap();
    let mut store = GlobalDataStore::new();
    assert_eq!(hash_global_data(store.current()), expected_defaults);

    // DrawSkyBox / DebugAI parseBool quirk and FirewallPortAllocationDelta Short quirk
    let quirk_ini = "GameData\n\
        DrawSkyBox = Yes\n\
        DebugAI = Yes\n\
        FirewallPortAllocationDelta = -12\n\
        SkirmishTickRate = 45\n\
        TickRate = 60\n\
        DefaultStartingCash = 25000\n\
        StandardPublicBone = Muzzle01 Muzzle02\n\
        StandardPublicBone = Turret01\n\
        WeaponBonus = DEMORALIZED_OBSOLETE RATE_OF_FIRE 150%\n\
    End\n";
    load_gamedata_text(&mut store, quirk_ini, IniLoadType::Overwrite, None);
    let cur = store.current();
    assert_eq!(cur.draw_sky_box.to_bits(), 1u32);
    assert_eq!(cur.debug_ai, 1);
    assert_eq!(cur.firewall_port_allocation_delta, -12);
    assert_eq!(cur.skirmish_tick_rate, 60);
    assert_eq!(cur.default_starting_cash.count_money(), 25000);
    assert_eq!(
        cur.standard_public_bones,
        vec!["Muzzle01", "Muzzle02", "Turret01"]
    );
    let mut bonus = WeaponBonus::new();
    cur.weapon_bonus_set
        .as_ref()
        .unwrap()
        .append_bonuses(1u32 << 7, &mut bonus);
    assert!((bonus.get_field(WeaponBonusField::RateOfFire) - 1.5).abs() < 1e-6);

    // Invalid TimeOfDay returns FALSE and preserves state
    let before_tod = hash_global_data(store.current());
    assert_eq!(store.current_mut().set_time_of_day(0), FALSE);
    assert_eq!(store.current_mut().set_time_of_day(5), FALSE);
    assert_eq!(store.current_mut().set_time_of_day(-1), FALSE);
    assert_eq!(hash_global_data(store.current()), before_tod);

    // Override stack + reset
    store.new_override();
    assert_eq!(store.override_depth(), 1);
    store.current_mut().build_speed = 9.5;
    store.reset();
    assert_eq!(store.override_depth(), 0);
    assert_eq!(hash_global_data(store.current()), before_tod);

    // NaN / Inf clamping in OptionPreferences
    let mut prefs = OptionPreferencesOverrides::default();
    prefs.max_camera_height = Some(f32::NAN);
    prefs.terrain_draw_distance_scale = Some(f32::INFINITY);
    prefs.gamma_value = 0.75; // truncates to 0 -> 0.6
    store.current_mut().apply_option_preferences(&prefs);
    assert_eq!(store.current().max_camera_height, 100.0);
    assert_eq!(store.current().terrain_draw_distance_scale, 1.0);
    assert_eq!(store.current().display_gamma, 0.6);
}

#[test]
#[ignore = "PKT-442 GlobalData skeleton — remove #[ignore] once implemented"]
fn test_pkt442_1000_seed_cpp_parity() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);
        let inp = build_seed_inputs(seed);

        let mut store = GlobalDataStore::new();
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "defaults_crc").unwrap(),
            "defaults_crc mismatch at seed {seed}"
        );

        load_gamedata_text(&mut store, &inp.base_ini, IniLoadType::Overwrite, None);
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "after_base_crc").unwrap(),
            "after_base_crc mismatch at seed {seed}"
        );

        let tod_ret = store.current_mut().set_time_of_day(inp.tod_arg);
        assert_eq!(
            tod_ret as u32,
            get_u32(line, "tod_ret").unwrap(),
            "tod_ret mismatch at seed {seed}"
        );
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "after_tod_crc").unwrap(),
            "after_tod_crc mismatch at seed {seed}"
        );

        let mut bonus = WeaponBonus::new();
        store
            .current()
            .weapon_bonus_set
            .as_ref()
            .unwrap()
            .append_bonuses(inp.bonus_flags, &mut bonus);
        let got_bonus = [
            bonus.fields[0].to_bits() as i32,
            bonus.fields[1].to_bits() as i32,
            bonus.fields[2].to_bits() as i32,
            bonus.fields[3].to_bits() as i32,
            bonus.fields[4].to_bits() as i32,
        ];
        let exp_bonus = get_i32_array::<5>(line, "bonus_bits").unwrap();
        assert_eq!(got_bonus, exp_bonus, "bonus_bits mismatch at seed {seed}");

        load_gamedata_text(
            &mut store,
            &inp.override_ini,
            IniLoadType::CreateOverrides,
            None,
        );
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "after_override_crc").unwrap(),
            "after_override_crc mismatch at seed {seed}"
        );

        store.reset();
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "after_reset_crc").unwrap(),
            "after_reset_crc mismatch at seed {seed}"
        );

        load_gamedata_text(&mut store, &inp.multi_ini, IniLoadType::Multifile, None);
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "after_multifile_crc").unwrap(),
            "after_multifile_crc mismatch at seed {seed}"
        );

        load_gamedata_text(
            &mut store,
            "GameData\nEnd\n",
            IniLoadType::Overwrite,
            Some(&inp.prefs),
        );
        store
            .current_mut()
            .parse_custom_definition(inp.has_fullviewport_dat);
        assert_eq!(
            hash_global_data(store.current()),
            get_u32(line, "after_prefs_crc").unwrap(),
            "after_prefs_crc mismatch at seed {seed}"
        );
        assert_eq!(
            compute_getters_mask(store.current()),
            get_u32(line, "getters_mask").unwrap(),
            "getters_mask mismatch at seed {seed}"
        );

        let exe_crc = if inp.use_pattern_exe {
            let mut pos: u64 = 0;
            let total = inp.pattern_exe_bytes;
            let pseed = inp.pattern_exe_seed;
            let mut reader = |buf: &mut [u8]| -> usize {
                if pos >= total {
                    return 0;
                }
                let take = (total - pos).min(buf.len() as u64) as usize;
                for (i, byte) in buf[..take].iter_mut().enumerate() {
                    let idx = pos + i as u64;
                    *byte = ((idx.wrapping_mul(131))
                        .wrapping_add(pseed as u64)
                        .wrapping_add(idx >> 16)
                        & 0xff) as u8;
                }
                pos += take as u64;
                take
            };
            store.current_mut().compute_exe_crc_with_reader(
                Some(&mut reader),
                inp.version_num,
                inp.skirmish_scb.as_deref(),
                inp.mp_scb.as_deref(),
            )
        } else {
            store.current_mut().compute_exe_crc(
                if inp.has_exe {
                    Some(inp.exe_bytes.as_slice())
                } else {
                    None
                },
                inp.version_num,
                inp.skirmish_scb.as_deref(),
                inp.mp_scb.as_deref(),
            )
        };
        assert_eq!(
            exe_crc,
            get_u32(line, "exe_crc").unwrap(),
            "exe_crc mismatch at seed {seed}"
        );
    }
}
