//! The rule data of the installed game, loaded by [`GameDataLoader`] and compared with the
//! reference parsers (`tests/cpp_harness/gen_game_data_load.cpp`), and the same comparison
//! line by line over the synthetic `tests/cpp_harness/game_data_probe*.ini`.
//!
//! The retail tests need the game (`ZH_DIR`, default `$HOME/Games/generalszh`) and pass
//! without checking anything when it is absent. The vectors hold digests and counts only.
//! `GAME_DATA_DUMP=<file>` writes the compared lines, for a local diff against the
//! oracle's `ORACLE_DUMP`. Every other test uses inputs written for it.

use std::collections::BTreeMap;

use zh_common::golden_json::{get_str, get_u32};
use zh_common::ini::IniLoadType;
use zh_common::io::file::{MemoryFileHost, OsDiskFileHost};
use zh_common::xfer::{Snapshot, XferCrc, XferSave};
use zh_logic::ai::ai::AiData;
use zh_logic::combat::locomotor::LocomotorTemplate;
use zh_common::ini::ini_terrain_road::TerrainRoadType;
use zh_logic::sim::RulesRegistry;
use zh_logic::gamelogic::game_engine::{
    file_directory_list, mount_game_archives, BlockStatus, GameDataLoader,
};
use zh_logic::object::object_creation_list::{ObjectCreationNugget, RadiusDecalTemplate};
use zh_logic::object::weapon::WeaponTemplate;
use zh_logic::rules::command_set::CommandButtonType;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/GameDataLoad.json");
/// `(category, field)` pairs that match the reference; the rest are listed as differing.
const MATCHING: &str = include_str!("../../../docs/golden_vectors/GameDataLoad_status.txt");

fn game_dir() -> Option<String> {
    let dir = std::env::var("ZH_DIR").unwrap_or_else(|_| {
        format!(
            "{}/Games/generalszh",
            std::env::var("HOME").unwrap_or_default()
        )
    });
    std::path::Path::new(&dir)
        .join("INIZH.big")
        .exists()
        .then_some(dir)
}

fn real(v: f32) -> String {
    format!("{:08x}", v.to_bits())
}

fn science_names(loader: &mut GameDataLoader) -> BTreeMap<i32, String> {
    let r = &mut loader.rules;
    let mut names = BTreeMap::new();
    for (kind, name) in &loader.report.entries {
        if *kind == "Science" {
            let st = r
                .science_store
                .get_science_from_internal_name(name, &mut r.name_keys);
            names.insert(st, name.clone());
        }
    }
    names
}

fn xy(c: zh_common::math::Coord2D) -> String {
    format!("{},{}", real(c.x), real(c.y))
}

fn xyz(c: zh_common::math::Coord3D) -> String {
    format!("{},{},{}", real(c.x), real(c.y), real(c.z))
}

fn list(names: &[String]) -> String {
    names.iter().map(|n| format!("{n},")).collect()
}

fn decal(t: &RadiusDecalTemplate) -> String {
    format!(
        "{},{},{},{},{},{},{}",
        t.name,
        t.shadow_type,
        real(t.min_opacity),
        real(t.max_opacity),
        t.opacity_throb_time,
        t.color,
        t.only_visible_to_owning_player as i64
    )
}

/// Weapons, locomotors, `AIData`, creation lists, roads and bridges.
fn rule_dump(loader: &mut GameDataLoader) -> Vec<String> {
    let sciences = science_names(loader);
    let r = &loader.rules;
    let mut out = Vec::new();
    for (kind, name) in &loader.report.entries {
        let mut put = |cat: &str, field: &str, value: String| {
            out.push(format!("{cat}|{name}|{field}={value}"));
        };
        match *kind {
            "Weapon" => {
                let w = r.weapon_store.find_weapon_template(name).unwrap();
                macro_rules! f {
                    ($($n:ident)*) => { $(put("weapon", stringify!($n), real(w.$n));)* };
                }
                macro_rules! n {
                    ($($n:ident)*) => { $(put("weapon", stringify!($n), (w.$n as i64).to_string());)* };
                }
                macro_rules! t {
                    ($($n:ident)*) => { $(put("weapon", stringify!($n), w.$n.clone());)* };
                }
                f!(primary_damage primary_damage_radius secondary_damage secondary_damage_radius
                   shock_wave_amount shock_wave_radius shock_wave_taper_off attack_range
                   minimum_attack_range request_assist_range aim_delta scatter_radius
                   scatter_target_scalar scatter_radius_vs_infantry weapon_speed min_weapon_speed
                   weapon_recoil min_target_pitch max_target_pitch radius_damage_angle
                   continue_attack_range historic_bonus_radius);
                n!(damage_type death_type scale_weapon_speed clip_size clip_reload_time
                   min_delay_between_shots max_delay_between_shots continuous_fire_one_shots_needed
                   continuous_fire_two_shots_needed continuous_fire_coast_frames
                   auto_reload_when_idle_frames shots_per_barrel damage_dealt_at_self_position
                   reload_type prefire_type pre_attack_delay affects_mask collide_mask anti_mask
                   leech_range_weapon capable_of_following_waypoint is_shows_ammo_pips
                   allow_attack_garrisoned_buildings play_fx_when_stealthed missile_calls_on_die
                   suspend_fx_delay damage_status_type historic_bonus_time historic_bonus_count);
                t!(projectile_name projectile_stream_name laser_name laser_bone_name);
                let points = w
                    .scatter_targets
                    .iter()
                    .map(|c| real(c.x) + &real(c.y))
                    .collect();
                put("weapon", "scatter_targets", points);
                put("weapon", "fire_sound", w.fire_sound.clone());
                put("weapon", "fire_sound_loop_time", w.fire_sound_loop_time.to_string());
                put("weapon", "fire_fx", list(&w.fire_fx));
                put("weapon", "projectile_detonation_fx", list(&w.projectile_detonation_fx));
                put("weapon", "projectile_exhaust", list(&w.projectile_exhaust));
                put("weapon", "fire_ocl", list(&w.fire_ocl));
                put("weapon", "projectile_detonation_ocl", list(&w.projectile_detonation_ocl));
                put("weapon", "historic_bonus_weapon", w.historic_bonus_weapon.clone());
                let bonus = w
                    .extra_bonus
                    .entries
                    .iter()
                    .flat_map(|b| b.fields.iter().map(|&v| real(v)))
                    .collect();
                put("weapon", "extra_bonus", bonus);
            }
            "Locomotor" => {
                let l = r.locomotor_store.find_by_name(name).unwrap();
                macro_rules! f {
                    ($($n:ident)*) => { $(put("locomotor", stringify!($n), real(l.$n));)* };
                }
                macro_rules! n {
                    ($($n:ident)*) => { $(put("locomotor", stringify!($n), (l.$n as i64).to_string());)* };
                }
                f!(max_speed max_speed_damaged min_speed max_turn_rate max_turn_rate_damaged
                   acceleration acceleration_damaged lift lift_damaged braking min_turn_speed
                   preferred_height preferred_height_damping circling_radius speed_limit_z
                   extra_2d_friction max_thrust_angle accel_pitch_limit decel_pitch_limit
                   bounce_kick pitch_stiffness roll_stiffness pitch_damping roll_damping
                   pitch_by_z_vel_coef thrust_roll wobble_rate min_wobble max_wobble
                   forward_vel_coef lateral_vel_coef forward_accel_coef lateral_accel_coef
                   uniform_axial_damping turn_pivot_offset close_enough_dist
                   ultra_accurate_slide_factor wheel_turn_angle maximum_wheel_extension
                   maximum_wheel_compression wander_width_factor wander_length_factor
                   wander_about_point_radius rudder_correction_degree rudder_correction_rate
                   elevator_correction_degree elevator_correction_rate);
                n!(surfaces behavior_z appearance move_priority airborne_targeting_height
                   is_close_enough_dist3d locomotor_works_when_dead allow_motive_force_airborne
                   apply_2d_friction_airborne downhill_only stick_to_ground can_move_backward
                   has_suspension);
            }
            _ => {}
        }
    }
    dump_ai_data(r, &sciences, &mut out);
    dump_creation_lists(r, &mut out);
    for road in &r.terrain_roads.roads {
        dump_road("road", road, &mut out);
    }
    for bridge in &r.terrain_roads.bridges {
        dump_road("bridge", bridge, &mut out);
    }
    out.sort();
    out
}

fn dump_ai_data(r: &RulesRegistry, sciences: &BTreeMap<i32, String>, out: &mut Vec<String>) {
    let d = &r.ai_data;
    let mut put = |field: &str, value: String| out.push(format!("ai_data|AIData|{field}={value}"));
    macro_rules! f {
        ($($c:ident: $n:ident)*) => { $(put(stringify!($c), real(d.$n));)* };
    }
    macro_rules! n {
        ($($c:ident: $n:ident)*) => { $(put(stringify!($c), (d.$n as i64).to_string());)* };
    }
    f!(structureSeconds: structure_seconds teamSeconds: team_seconds
       structuresWealthyMod: structures_wealthy_mod teamWealthyMod: team_wealthy_mod
       structuresPoorMod: structures_poor_mod teamPoorMod: team_poor_mod
       teamResourcesToBuild: team_resources_to_build
       guardInnerModifierAI: guard_inner_modifier_ai guardOuterModifierAI: guard_outer_modifier_ai
       guardInnerModifierHuman: guard_inner_modifier_human
       guardOuterModifierHuman: guard_outer_modifier_human
       skirmishGroupFudgeValue: skirmish_group_fudge_value repulsedDistance: repulsed_distance
       alertRangeModifier: alert_range_modifier aggressiveRangeModifier: aggressive_range_modifier
       attackPriorityDistanceModifier: attack_priority_distance_modifier
       maxRecruitDistance: max_recruit_distance
       skirmishBaseDefenseExtraDistance: skirmish_base_defense_extra_distance
       wallHeight: wall_height minDistanceForGroup: min_distance_for_group
       distanceRequiresGroup: distance_requires_group minClumpDensity: min_clump_density
       supplyCenterSafeRadius: supply_center_safe_radius
       aiDozerBoredRadiusModifier: ai_dozer_bored_radius_modifier
       maxRetaliateDistance: max_retaliate_distance
       retaliateFriendsRadius: retaliate_friends_radius);
    n!(resourcesWealthy: resources_wealthy resourcesPoor: resources_poor
       forceIdleFramesCount: force_idle_frames_count guardChaseUnitFrames: guard_chase_unit_frames
       guardEnemyScanRate: guard_enemy_scan_rate
       guardEnemyReturnScanRate: guard_enemy_return_scan_rate enableRepulsors: enable_repulsors
       forceSkirmishAI: force_skirmish_ai rotateSkirmishBases: rotate_skirmish_bases
       attackUsesLineOfSight: attack_uses_line_of_sight
       attackIgnoreInsignificantBuildings: attack_ignore_insignificant_buildings
       minInfantryForGroup: min_infantry_for_group minVehiclesForGroup: min_vehicles_for_group
       infantryPathfindDiameter: infantry_pathfind_diameter
       vehiclePathfindDiameter: vehicle_pathfind_diameter
       rebuildDelaySeconds: rebuild_delay_seconds aiCrushesInfantry: ai_crushes_infantry);
    let mut crc = XferCrc::new();
    d.crc(&mut crc);
    put("crc", crc.get_crc().to_string());
    for (at, side) in d.side_info.iter().enumerate() {
        let mut value = format!(
            "{};{},{},{};{};",
            side.side, side.easy, side.normal, side.hard, side.base_defense_structure1
        );
        for set in &side.skill_sets {
            for science in set {
                value += &format!("{},", sciences[science]);
            }
            value += ";";
        }
        out.push(format!("ai_side|{}|info={value}", 1000 + at));
    }
    for (at, faction) in d.side_build_lists.iter().enumerate() {
        let mut value = format!("{};", faction.side);
        for b in &faction.build_list {
            value += &format!(
                "{},{},{},{},{},{},{},{},{},{},{},{};",
                b.template_name,
                b.building_name,
                xyz(b.location),
                xy(b.rally_point_offset),
                real(b.angle),
                b.is_initially_built as i64,
                b.num_rebuilds,
                b.automatically_build as i64,
                b.health,
                b.whiner as i64,
                b.unsellable as i64,
                b.repairable as i64
            );
        }
        out.push(format!("ai_build|{}|list={value}", 1000 + at));
    }
}

fn dump_creation_lists(r: &RulesRegistry, out: &mut Vec<String>) {
    for (name, ocl) in r.ocl_store.iter() {
        out.push(format!("ocl|{name}|count={}", ocl.nuggets.len()));
        for (at, nugget) in ocl.nuggets.iter().enumerate() {
            let mut put = |field: &str, value: String| {
                out.push(format!("nugget|{name}#{at}|{field}={value}"));
            };
            macro_rules! f {
                ($g:ident, $($n:ident)*) => { $(put(stringify!($n), real($g.$n));)* };
            }
            macro_rules! n {
                ($g:ident, $($n:ident)*) => { $(put(stringify!($n), ($g.$n as i64).to_string());)* };
            }
            macro_rules! t {
                ($g:ident, $($n:ident)*) => { $(put(stringify!($n), $g.$n.clone());)* };
            }
            match nugget {
                ObjectCreationNugget::Generic(g) => {
                    put("type", "generic".into());
                    put("names", list(&g.names));
                    let anims = g
                        .anim_sets
                        .iter()
                        .map(|a| format!("{},{},{};", a.initial, a.flying, a.final_anim))
                        .collect();
                    put("anim_sets", anims);
                    put("offset", xyz(g.offset));
                    t!(g, fx_final bounce_sound put_in_container particle_system_name
                       fade_sound_name);
                    f!(g, mass extra_bounciness extra_friction disposition_intensity spin_rate
                       yaw_rate roll_rate pitch_rate min_mag max_mag min_pitch max_pitch
                       min_health max_health min_distance_a_formation min_distance_b_formation
                       max_distance_formation);
                    n!(g, names_are_objects debris_to_generate disposition min_frames max_frames
                       shadow_type min_lod_required invulnerable_time fade_frames object_count
                       requires_live_player contain_inside_source_object preserve_layer
                       ok_to_change_model_color orient_in_force_direction spread_formation fade_in
                       fade_out ignore_primary_obstacle inherits_veterancy dies_on_bad_land
                       skip_if_significantly_airborne);
                }
                ObjectCreationNugget::ApplyRandomForce(g) => {
                    put("type", "force".into());
                    f!(g, spin_rate min_mag max_mag min_pitch max_pitch);
                }
                ObjectCreationNugget::DeliverPayload(g) => {
                    put("type", "payload".into());
                    t!(g, transport_name put_in_container);
                    n!(g, start_at_preferred_height start_at_max_speed formation_size
                       delay_delivery_frames_max);
                    f!(g, formation_spacing convergence_factor error_radius);
                    let payload = g
                        .payload
                        .iter()
                        .map(|p| format!("{},{};", p.name, p.count))
                        .collect();
                    put("payload", payload);
                    let d = &g.data;
                    t!(d, visible_drop_bone_name visible_sub_object_name
                       visible_payload_template_name strafe_fx visible_payload_weapon);
                    f!(d, dist_to_target pre_open_distance dive_start_distance dive_end_distance
                       exit_pitch_rate strafe_length delivery_decal_radius);
                    n!(d, max_attempts drop_delay fire_weapon self_destruct_object visible_num_bones
                       strafing_weapon_slot visible_items_dropped_per_interval
                       inherit_transport_velocity is_parachute_directly);
                    put("drop_offset", xyz(d.drop_offset));
                    put("drop_variance", xyz(d.drop_variance));
                    put("delivery_decal", decal(&d.delivery_decal));
                }
                ObjectCreationNugget::FireWeapon(weapon) => {
                    put("type", "fire_weapon".into());
                    put("weapon", weapon.clone());
                }
                ObjectCreationNugget::Attack(g) => {
                    put("type", "attack".into());
                    n!(g, number_of_shots weapon_slot);
                    put("delivery_decal", decal(&g.delivery_decal));
                    f!(g, delivery_decal_radius);
                }
            }
        }
    }
}

fn dump_road(cat: &str, road: &TerrainRoadType, out: &mut Vec<String>) {
    let name = &road.name;
    let mut put = |field: &str, value: String| out.push(format!("{cat}|{name}|{field}={value}"));
    macro_rules! f {
        ($($n:ident)*) => { $(put(stringify!($n), real(road.$n));)* };
    }
    macro_rules! n {
        ($($n:ident)*) => { $(put(stringify!($n), (road.$n as i64).to_string());)* };
    }
    macro_rules! t {
        ($($n:ident)*) => { $(put(stringify!($n), road.$n.clone());)* };
    }
    f!(road_width road_width_in_texture bridge_scale transition_effects_height);
    n!(id is_bridge num_fx_per_type);
    t!(texture scaffold_object_name scaffold_support_object_name bridge_model_name
       bridge_model_name_damaged bridge_model_name_really_damaged bridge_model_name_broken
       texture_damaged texture_really_damaged texture_broken);
    let color = road.radar_color;
    put(
        "radar_color",
        real(color.red) + &real(color.green) + &real(color.blue),
    );
    put("tower_object_name", list(&road.tower_object_name));
    let sounds = (0..road.damage_to_sound.len())
        .map(|s| format!("{},{};", road.damage_to_sound[s], road.repaired_to_sound[s]))
        .collect();
    put("sounds", sounds);
    let mut effects = String::new();
    for s in 0..road.damage_to_ocl.len() {
        for i in 0..road.damage_to_ocl[s].len() {
            effects += &format!(
                "{},{},{},{};",
                road.damage_to_ocl[s][i],
                road.damage_to_fx[s][i],
                road.repaired_to_ocl[s][i],
                road.repaired_to_fx[s][i]
            );
        }
    }
    put("effects", effects);
}

fn dump(loader: &mut GameDataLoader) -> Vec<String> {
    let mut out = rule_dump(loader);
    let science_names = science_names(loader);
    let entries: Vec<(&str, String)> = loader.report.entries.iter().cloned().collect();
    let r = &mut loader.rules;
    for (kind, name) in &entries {
        let mut put = |cat: &str, field: &str, value: String| {
            out.push(format!("{cat}|{name}|{field}={value}"));
        };
        match *kind {
            "Armor" => {
                let a = r
                    .armor_store
                    .find_armor_template(&mut r.name_keys, name)
                    .unwrap();
                let all = (0..)
                    .map_while(zh_logic::rules::combat_types::DamageType::from_i32)
                    .take(zh_logic::rules::combat_types::DAMAGE_NUM_TYPES)
                    .map(|t| real(a.coefficient(t)))
                    .collect();
                put("armor", "coefficients", all);
            }
            "Science" => {
                let st = r
                    .science_store
                    .get_science_from_internal_name(name, &mut r.name_keys);
                put(
                    "science",
                    "cost",
                    r.science_store.get_science_purchase_cost(st).to_string(),
                );
                put(
                    "science",
                    "grantable",
                    (r.science_store.is_science_grantable(st) as i64).to_string(),
                );
            }
            "SpecialPower" => {
                let p = r
                    .special_power_store
                    .find_special_power_template(name)
                    .unwrap();
                let science = science_names
                    .get(&p.required_science())
                    .cloned()
                    .unwrap_or_default();
                put("special_power", "id", p.id().to_string());
                put(
                    "special_power",
                    "type",
                    (p.special_power_type() as i64).to_string(),
                );
                put("special_power", "reload_time", p.reload_time().to_string());
                put("special_power", "required_science", science);
                put(
                    "special_power",
                    "detection_time",
                    p.detection_time().to_string(),
                );
                put(
                    "special_power",
                    "view_object_duration",
                    p.view_object_duration().to_string(),
                );
                put(
                    "special_power",
                    "view_object_range",
                    real(p.view_object_range()),
                );
                put(
                    "special_power",
                    "radius_cursor_radius",
                    real(p.radius_cursor_radius()),
                );
                put(
                    "special_power",
                    "public_timer",
                    (p.has_public_timer() as i64).to_string(),
                );
                put(
                    "special_power",
                    "shared_n_sync",
                    (p.is_shared_n_sync() as i64).to_string(),
                );
            }
            "CommandSet" => {
                let slots = r
                    .command_sets
                    .find(name)
                    .unwrap()
                    .iter()
                    .map(|b| {
                        let kind = match b.command {
                            CommandButtonType::UnitBuild => 'u',
                            CommandButtonType::DozerConstruct => 'd',
                            CommandButtonType::Other => 'o',
                        };
                        format!("{kind}:{}:{};", b.object, b.upgrade)
                    })
                    .collect();
                put("command_set", "slots", slots);
            }
            _ => {}
        }
    }
    // Includes the veterancy upgrades, which no file defines.
    for name in r.upgrade_center.get_upgrade_names() {
        let u = r
            .upgrade_center
            .find_upgrade(&name, &mut r.name_keys)
            .unwrap();
        out.push(format!(
            "upgrade|{name}|build_frames={}",
            u.calc_time_to_build(None)
        ));
        out.push(format!(
            "upgrade|{name}|cost={}",
            u.calc_cost_to_build(None)
        ));
    }
    for id in 1..=u16::MAX {
        let Some(t) = r.thing_factory.find_by_template_id(id) else {
            continue;
        };
        let name = &t.name_string;
        let mut put = |field: &str, value: String| {
            out.push(format!("template|{name}|{field}={value}"));
        };
        put("template_id", t.template_id.to_string());
        put("build_cost", t.build_cost.to_string());
        put("build_time", real(t.build_time));
        put("refund_value", t.refund_value.to_string());
        put("vision_range", real(t.vision_range));
        put("shroud_clearing_range", real(t.shroud_clearing_range));
        put("energy_production", t.energy_production.to_string());
        put("energy_bonus", t.energy_bonus.to_string());
        put("crusher_level", t.crusher_level.to_string());
        put("crushable_level", t.crushable_level.to_string());
        put("transport_slot_count", t.transport_slot_count.to_string());
        put("threat_value", t.threat_value.to_string());
        put("is_trainable", (t.is_trainable as i64).to_string());
        put("asset_scale", real(t.asset_scale));
        put("command_set", t.command_set_string.clone());
        let kinds = (0..zh_logic::rules::kind_of::KINDOF_NAMES.len() as i32)
            .map(|i| {
                if t.kindof.test(i) as i64 != 0 {
                    '1'
                } else {
                    '0'
                }
            })
            .collect();
        put("kindof", kinds);
        let xp = (0..t.skill_point_values.len())
            .map(|i| {
                format!(
                    "{},{},{};",
                    t.skill_point_values[i], t.experience_values[i], t.experience_required[i]
                )
            })
            .collect();
        put("experience", xp);
    }
    out.sort();
    out
}

/// `(category, field) -> (count, FNV-1a 64 over the sorted lines)`, as the oracle does.
fn digests(lines: &[String]) -> BTreeMap<(String, String), (u32, u64)> {
    let mut sums: BTreeMap<(String, String), (u32, u64)> = BTreeMap::new();
    for line in lines {
        let (cat, rest) = line.split_once('|').unwrap();
        let (_, rest) = rest.split_once('|').unwrap();
        let (field, _) = rest.split_once('=').unwrap();
        let sum = sums
            .entry((cat.to_owned(), field.to_owned()))
            .or_insert((0, 0xcbf2_9ce4_8422_2325));
        sum.0 += 1;
        for byte in line.bytes().chain(std::iter::once(b'\n')) {
            sum.1 = (sum.1 ^ u64::from(byte)).wrapping_mul(0x100_0000_01b3);
        }
    }
    sums
}

#[test]
fn retail_rule_data_matches_the_reference() {
    let Some(dir) = game_dir() else {
        eprintln!("skipped: the game is not installed");
        return;
    };
    let fs = mount_game_archives(&OsDiskFileHost, &dir);
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);

    // Nothing may fail, and the counts per block type and status are fixed.
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures.first()
    );
    // Only the partial `CommandButton` reader leaves keys of the retail data unread.
    let unread: Vec<&String> = loader
        .report
        .ignored_fields
        .keys()
        .filter(|k| !k.starts_with("CommandButton."))
        .collect();
    assert!(unread.is_empty(), "{unread:?}");
    let mut counts = String::new();
    for ((kind, status), n) in &loader.report.blocks {
        counts += &format!("blocks {kind} {} {n}\n", status.label());
    }
    for (kind, n) in [("draw", 0), ("client", 1), ("logic", 2)].map(|(kind, _)| {
        let n: u32 = loader
            .report
            .stub_modules
            .iter()
            .filter(|(k, _)| k.starts_with(kind))
            .map(|(_, n)| n)
            .sum();
        (kind, n)
    }) {
        counts += &format!("unparsed_module_data {kind} {n}\n");
    }
    counts += &format!("files {}\n", loader.report.files.len());
    counts += &format!(
        "ignored_fields {}\n",
        loader.report.ignored_fields.values().sum::<u32>()
    );

    let lines = dump(&mut loader);
    if let Ok(path) = std::env::var("GAME_DATA_DUMP") {
        std::fs::write(path, lines.join("\n") + "\n").unwrap();
    }
    let ours = digests(&lines);

    let mut status = String::new();
    let mut rows = 0;
    for line in VECTORS.lines().filter(|l| l.starts_with("{\"cat\"")) {
        let key = (
            get_str(line, "cat").unwrap().to_owned(),
            get_str(line, "field").unwrap().to_owned(),
        );
        let count = get_u32(line, "count").unwrap();
        let fnv = u64::from_str_radix(get_str(line, "fnv").unwrap(), 16).unwrap();
        let verdict = match ours.get(&key) {
            Some(&(n, sum)) if n == count && sum == fnv => "match",
            Some(&(n, _)) if n == count => "values_differ",
            _ => "count_differs",
        };
        status += &format!("{} {} {verdict}\n", key.0, key.1);
        rows += 1;
    }
    assert_eq!(rows, ours.len(), "category and field set");
    let report = counts + &status;
    if let Ok(path) = std::env::var("GAME_DATA_STATUS") {
        std::fs::write(path, &report).unwrap();
    }
    assert_eq!(report, MATCHING);
}

fn load(text: &str, load_type: IniLoadType) -> GameDataLoader {
    let mut loader = GameDataLoader::new();
    loader.load_bytes("test.ini", text.as_bytes(), load_type);
    loader
}

fn count(loader: &GameDataLoader, kind: &str, status: BlockStatus) -> u32 {
    loader
        .report
        .blocks
        .get(&(kind.to_owned(), status))
        .copied()
        .unwrap_or(0)
}

#[test]
fn every_block_is_counted_by_what_happened_to_it() {
    let loader = load(
        "Armor Plate ; comment\n  Armor = FLAME 50%\nEnd\n\
         NotABlock x\n  Foo = 1\nEnd\n\
         Video Intro\n  Filename = a\nEnd\n\
         FXList FX_Test\nEnd\n\
         Weather\nEnd\n\
         ReallyLowMHz 600\n\
         Armor Broken\n  NoSuchField = 1\nEnd\n\
         Science SCIENCE_Test\n  SciencePurchasePointCost = 3\nEnd\n",
        IniLoadType::Overwrite,
    );
    assert_eq!(count(&loader, "Armor", BlockStatus::Loaded), 1);
    assert_eq!(count(&loader, "Armor", BlockStatus::Failed), 1);
    assert_eq!(count(&loader, "NotABlock", BlockStatus::Unknown), 1);
    assert_eq!(count(&loader, "Video", BlockStatus::Client), 1);
    assert_eq!(count(&loader, "ReallyLowMHz", BlockStatus::Client), 1);
    assert_eq!(count(&loader, "FXList", BlockStatus::NameOnly), 1);
    assert_eq!(count(&loader, "Weather", BlockStatus::Unported), 1);
    assert_eq!(count(&loader, "Science", BlockStatus::Loaded), 1);
    assert!(loader.rules.fx_list_names.contains("FX_Test"));
    let failure = &loader.report.failures[0];
    assert_eq!((failure.name.as_str(), failure.line), ("Broken", 15));
}

#[test]
fn weapon_and_command_blocks_report_what_they_ignore() {
    let loader = load(
        "Weapon Gun\n  PrimaryDamage = 5.0\n  ScatterTarget = X: 0.5 Y:-0.25\n  MadeUp = 1\nEnd\n\
         CommandButton Build\n  Command = UNIT_BUILD\n  Object = Tank\n  TextLabel = x\nEnd\n\
         CommandSet Set\n  2 = Build\n  1 = Build\nEnd\n",
        IniLoadType::Overwrite,
    );
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    let gun = loader
        .rules
        .weapon_store
        .find_weapon_template("Gun")
        .unwrap();
    assert_eq!(gun.primary_damage, 5.0);
    assert_eq!(
        (gun.scatter_targets[0].x, gun.scatter_targets[0].y),
        (0.5, -0.25)
    );
    assert_eq!(loader.report.ignored_fields.get("Weapon.MadeUp"), Some(&1));
    assert_eq!(
        loader.report.ignored_fields.get("CommandButton.TextLabel"),
        Some(&1)
    );
    let set = loader.rules.command_sets.find("Set").unwrap();
    assert_eq!(set.len(), 2);
    assert_eq!(set[0].command, CommandButtonType::UnitBuild);
    assert_eq!(set[0].object, "Tank");
}

#[test]
fn unparsed_module_data_with_nested_blocks_keeps_the_object_whole() {
    let loader = load(
        "Object Thing\n\
         \x20 Draw = W3DModelDraw Tag01\n\
         \x20   DefaultConditionState\n\
         \x20     Turret = Bone\n\
         \x20   End\n\
         \x20   ConditionState = MOVING\n\
         \x20   End\n\
         \x20 End\n\
         \x20 BuildCost = 700\n\
         End\n",
        IniLoadType::Overwrite,
    );
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    assert_eq!(
        loader.report.stub_modules.get("draw:W3DModelDraw"),
        Some(&1)
    );
    let thing = loader.rules.thing_factory.find_by_template_id(1).unwrap();
    assert_eq!(thing.build_cost, 700);
}

#[test]
fn map_overrides_replace_the_final_template_only() {
    let mut loader = load(
        "Object Thing\n  BuildCost = 1\nEnd\n",
        IniLoadType::Overwrite,
    );
    loader.load_bytes(
        "map.ini",
        b"Object Thing\n  BuildCost = 2\nEnd\n",
        IniLoadType::CreateOverrides,
    );
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    let factory = &loader.rules.thing_factory;
    assert_eq!(factory.find_by_template_id(1).unwrap().build_cost, 1);
    assert_eq!(factory.find_by_template_id_final(1).unwrap().build_cost, 2);
}

/// A `.big` archive holding `files`.
fn big(files: &[(&str, &str)]) -> Vec<u8> {
    let table: usize = files.iter().map(|(name, _)| 8 + name.len() + 1).sum();
    let mut head = b"BIGF".to_vec();
    head.extend(0u32.to_le_bytes());
    head.extend((files.len() as u32).to_be_bytes());
    head.extend(0u32.to_be_bytes());
    let mut offset = 16 + table;
    let mut body: Vec<u8> = Vec::new();
    for (name, text) in files {
        head.extend((offset as u32).to_be_bytes());
        head.extend((text.len() as u32).to_be_bytes());
        head.extend(name.as_bytes());
        head.push(0);
        offset += text.len();
        body.extend(text.as_bytes());
    }
    head.extend(body);
    head
}

#[test]
fn archives_overlay_and_load_in_directory_order() {
    let armor = |name: &str| format!("Armor {name}\nEnd\n");
    let object = |cost: u32| format!("Object Thing\n  BuildCost = {cost}\nEnd\n");
    let (a, b, p, nested, base) = (
        armor("FromA"),
        armor("FromB"),
        armor("FromPatch"),
        armor("FromNested"),
        armor("FromBase"),
    );
    let (sub, extra, one, two) = (armor("Sub"), armor("Extra"), object(1), object(2));
    let mut host = MemoryFileHost::default();
    let mut add = |path: &str, files: &[(&str, &str)]| host.add_file(path, big(files), 0, 0);
    add(
        "root/A.big",
        &[
            ("Data\\INI\\Armor.ini", &a),
            ("Data\\INI\\DamageFX.ini", &a),
            ("Data\\INI\\Armor\\Deeper\\Sub.ini", &sub),
            ("Data\\INI\\Armor\\Zz.ini", &extra),
            ("Data\\INI\\Object.ini", &one),
            ("Maps\\M\\map.ini", &two),
        ],
    );
    add(
        "root/B.big",
        &[
            ("Data\\INI\\DamageFX.ini", &b),
            ("Data\\INI\\Weapon.ini", &b),
        ],
    );
    add("root/PatchX.big", &[("Data\\INI\\Weapon.ini", &p)]);
    add(
        "root/ZH_Generals/Patch.big",
        &[("Data\\INI\\DamageFX.ini", &nested)],
    );
    add(
        "root/ZH_Generals/INI.big",
        &[
            ("Data\\INI\\Armor.ini", &base),
            ("Data\\INI\\Locomotor.ini", &base),
        ],
    );
    let fs = mount_game_archives(&host, "root");

    assert_eq!(
        file_directory_list(&fs, "Data\\INI\\Armor"),
        [
            "Data\\INI\\Armor.ini",
            "Data\\INI\\Armor\\zz.ini",
            "Data\\INI\\Armor\\deeper\\sub.ini"
        ]
    );
    let mut loader = GameDataLoader::new();
    loader.load_startup(&fs);
    let names: Vec<&str> = loader
        .report
        .entries
        .iter()
        .filter(|(kind, _)| *kind == "Armor")
        .map(|(_, name)| name.as_str())
        .collect();
    // The first archive wins over later ones and over the base game, a patch over both.
    assert_eq!(names, ["Extra", "FromA", "FromBase", "FromPatch", "Sub"]);
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    assert_eq!(loader.report.total(BlockStatus::Loaded), 7);

    loader.load_map_ini(&fs, "Maps\\M");
    loader.load_map_ini(&fs, "Maps\\Absent");
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    let factory = &loader.rules.thing_factory;
    assert_eq!(factory.find_by_template_id(1).unwrap().build_cost, 1);
    assert_eq!(factory.find_by_template_id_final(1).unwrap().build_cost, 2);
}

#[test]
fn reading_resumes_at_the_block_after_an_unknown_line() {
    let mut loader = GameDataLoader::new();
    loader.load_bytes(
        "test.ini",
        b"Armor First\nEnd\nStray\x80 line\nArmor Second\nEnd\nReallyLowMHz 1\nOther\n",
        IniLoadType::Overwrite,
    );
    assert_eq!(count(&loader, "Stray", BlockStatus::Unknown), 1);
    assert_eq!(count(&loader, "Armor", BlockStatus::Loaded), 2);
    assert_eq!(loader.report.total(BlockStatus::Unknown), 2);
    assert_eq!(count(&loader, "ReallyLowMHz", BlockStatus::Client), 1);
}

const PROBE: &str = include_str!("../../../tests/cpp_harness/game_data_probe.ini");
const PROBE_OVERRIDE: &str =
    include_str!("../../../tests/cpp_harness/game_data_probe_override.ini");

#[test]
fn probe_rule_data_matches_the_reference_line_by_line() {
    let mut loader = load(PROBE, IniLoadType::Overwrite);
    loader.load_bytes(
        "override.ini",
        PROBE_OVERRIDE.as_bytes(),
        IniLoadType::CreateOverrides,
    );
    assert!(
        loader.report.failures.is_empty(),
        "{:?}",
        loader.report.failures
    );
    // Keys outside the field tables are skipped, as the reference skips them, and counted.
    let ignored: Vec<(&str, u32)> = loader
        .report
        .ignored_fields
        .iter()
        .map(|(field, &n)| (field.as_str(), n))
        .collect();
    assert_eq!(
        ignored,
        [
            ("AIData.MadeUpAIKey", 1),
            ("Bridge.MadeUpBridgeKey", 1),
            ("Locomotor.MadeUpLocomotorKey", 1),
            ("ObjectCreationList.MadeUpNuggetKey", 1),
            ("Weapon.MadeUpWeaponKey", 1),
            ("Weapon.primarydamage", 1),
        ]
    );
    assert_eq!(count(&loader, "Weapon", BlockStatus::Partial), 1);
    let want: Vec<&str> = VECTORS
        .lines()
        .filter(|l| l.starts_with("{\"probe\""))
        .map(|l| get_str(l, "probe").unwrap())
        .collect();
    let mut got = rule_dump(&mut loader);
    // `TAiData` as its constructor leaves it.
    let mut defaults = Vec::new();
    dump_ai_data(&RulesRegistry::new(), &BTreeMap::new(), &mut defaults);
    got.extend(
        defaults
            .iter()
            .map(|l| l.replacen("ai_data", "ai_default", 1)),
    );
    got.sort();
    // The vector reader has no escapes: the oracle writes a quote as a backtick.
    for line in &mut got {
        *line = line.replace('"', "`");
    }
    for (got, want) in got.iter().zip(&want) {
        assert_eq!(got, want);
    }
    assert_eq!(got.len(), want.len());
}

#[test]
fn template_blocks_take_their_name_from_the_header() {
    let mut weapon = WeaponTemplate::default();
    weapon.parse_ini_block("Weapon Named\nEnd\n").unwrap();
    assert_eq!(weapon.name, "Named");
    assert!(weapon.parse_ini_block("Armor Named\nEnd\n").is_err());
    let mut locomotor = LocomotorTemplate::new();
    locomotor
        .parse_ini_block("Locomotor Walker\nEnd\n")
        .unwrap();
    assert_eq!(locomotor.name, "Walker");
}

/// `TAiData::xfer` writes its version, 1, and nothing else.
#[test]
fn ai_data_saves_a_version_byte_only() {
    let mut data = AiData::default();
    let mut save = XferSave::new();
    data.xfer(&mut save).unwrap();
    assert_eq!(save.as_bytes(), [1]);
}

#[test]
fn repeated_and_broken_definitions_fail_as_the_reference_does() {
    let loader = load(
        "Weapon Twice\nEnd\nWeapon Twice\nEnd\n\
         Road Dirt\nEnd\nRoad Dirt\nEnd\n\
         Bridge Span\nEnd\nBridge Span\nEnd\n\
         Bridge Odd\n  TransitionToOCL = Transition:Break ToState:DAMAGED EffectNum:1 OCL:x\nEnd\n\
         Bridge Four\n  TransitionToFX = Transition:Damage ToState:DAMAGED EffectNum:4 FX:x\nEnd\n\
         Bridge Zero\n  TransitionToFX = Transition:Damage ToState:DAMAGED EffectNum:0 FX:x\nEnd\n\
         ObjectCreationList Light\n  CreateDebris\n    Mass = 0\n  End\nEnd\n\
         Weapon Open\n  PrimaryDamage = 1\n",
        IniLoadType::Overwrite,
    );
    let failed: Vec<(&str, &str)> = loader
        .report
        .failures
        .iter()
        .map(|f| (f.block.as_str(), f.name.as_str()))
        .collect();
    assert_eq!(
        failed,
        [
            ("Weapon", "Twice"),
            ("Road", "Dirt"),
            ("Bridge", "Span"),
            ("Bridge", "Odd"),
            ("Bridge", "Four"),
            ("Bridge", "Zero"),
            ("ObjectCreationList", "Light"),
            ("Weapon", "Open"),
        ]
    );
}
