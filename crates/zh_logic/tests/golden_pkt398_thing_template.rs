//! Gate V2 Golden Vector test suite for `WP-5.5` Core Templates & Base `Thing`
//! (`PKT-078` `ThingSort`, `PKT-139` `ArmorSet`, `PKT-392` `ObjectScriptStatusBits`,
//!  `PKT-393` `ObjectTypes`, `PKT-394` `Overridable`, `PKT-395` `Override`,
//!  `PKT-396` `Thing`, `PKT-397` `ThingFactory`, `PKT-398` `ThingTemplate`,
//!  `PKT-408` `SparseMatchFinder`).

use std::collections::{BTreeMap, BTreeSet};

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    encode_hex_bytes, extract_vector_lines, get_i32, get_i32_array, get_str, get_u32, get_u32_array,
};
use zh_common::ini::{IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::runtime::thing_sort::EditorSortingType;
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedShort, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::combat::armor::ArmorStore;
use zh_logic::rules::combat_types::{ArmorSetFlags, DamageType, WeaponSetFlags, ARMORSET_COUNT};
use zh_logic::economy::{Energy, Handicap, HandicapType, NoopEnergyCallbacks};
use zh_logic::object::object_script_status_bits::{
    OBJECT_STATUS_SCRIPT_DISABLED, OBJECT_STATUS_SCRIPT_TARGETABLE,
    OBJECT_STATUS_SCRIPT_UNPOWERED, OBJECT_STATUS_SCRIPT_UNSELLABLE,
    OBJECT_STATUS_SCRIPT_UNSTEALTHED,
};
use zh_logic::object::object_types::ObjectTypes;
use zh_logic::object::thing::{
    make_align_to_normal_matrix, Thing, ThingTerrainAccess, ThingTransformReactor,
};
use zh_logic::object::thing_factory::ThingFactory;
use zh_logic::object::thing_template::{
    BuildableStatus, DefaultModuleMaskLookup, GameTextLookup, ThingTemplate,
    ThingTemplateGlobalTunables, ThingTemplateParseContext, ThingTemplatePlayerView, LEVEL_COUNT,
};
use zh_logic::object::weapon::{WeaponStore, WeaponTemplate};
use zh_logic::pathfinder::pathfind_constants::PathfindLayerEnum;
use zh_logic::rules::kind_of::{KindOfMaskType, KindOfType};
use zh_logic::rules::production_prerequisite::{PrereqPlayerView, ThingTemplateRef};
use zh_logic::rules::science::{ScienceStore, ScienceTextLookup, ScienceType};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-398.json");

struct OracleGameText;

impl GameTextLookup for OracleGameText {
    fn fetch_utf16(&mut self, label: &str) -> Vec<u16> {
        let mut s = String::from("TXT:");
        s.push_str(label);
        s.bytes().map(|b| b as u16).collect()
    }
}

impl ScienceTextLookup for OracleGameText {
    fn fetch_label(&mut self, label: &str) -> String {
        format!("TXT:{label}")
    }
}

struct OracleTerrainLogic;

impl ThingTerrainAccess for OracleTerrainLogic {
    fn get_ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        let hz = 10.0_f32 + 0.05_f32 * x - 0.03_f32 * y;
        if let Some(n) = normal {
            n.x = -0.05_f32;
            n.y = 0.03_f32;
            n.z = 1.0_f32;
            n.normalize();
        }
        hz
    }

    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        stick_to_ground: Bool,
        mtx: &mut Matrix3D,
    ) -> PathfindLayerEnum {
        let mut terrain_normal = Coord3D::default();
        let terrain_at_pos = self.get_ground_height(pos.x, pos.y, Some(&mut terrain_normal));
        make_align_to_normal_matrix(angle, pos, &terrain_normal, mtx);
        if stick_to_ground != FALSE {
            mtx.set_z_translation(terrain_at_pos);
        }
        PathfindLayerEnum::Ground
    }

    fn is_underwater(
        &self,
        x: Real,
        y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        let hz = self.get_ground_height(x, y, None);
        let wz = if x < 0.0_f32 { 12.0_f32 } else { -5.0_f32 };
        if let Some(w) = water_z {
            *w = wz;
        }
        if let Some(t) = terrain_z {
            *t = hz;
        }
        if wz > hz { TRUE } else { FALSE }
    }
}

#[derive(Default)]
struct OracleTransformReactor {
    react_count: i32,
}

impl ThingTransformReactor for OracleTransformReactor {
    fn react_to_transform_change(
        &mut self,
        _old_mtx: &Matrix3D,
        _old_pos: &Coord3D,
        _old_angle: Real,
    ) {
        self.react_count += 1;
    }
}

struct OraclePlayerView {
    production_cost_changes: BTreeMap<String, Real>,
    production_time_changes: BTreeMap<String, Real>,
    kindof_cost_percent: (KindOfMaskType, Real),
    handicap: Handicap,
    energy: Energy,
}

impl PrereqPlayerView for OraclePlayerView {
    fn has_science(&self, _st: ScienceType) -> bool {
        false
    }

    fn count_objects_by_thing_template(
        &self,
        _tmpls: &[Option<ThingTemplateRef>],
        counts: &mut [Int],
    ) {
        counts.fill(0);
    }
}

impl ThingTemplatePlayerView for OraclePlayerView {
    fn as_prereq_view(&self) -> Option<&dyn PrereqPlayerView> {
        Some(self)
    }

    fn get_production_cost_change_percent(&self, template_name: &str) -> Real {
        self.production_cost_changes
            .get(template_name)
            .copied()
            .unwrap_or(0.0)
    }

    fn get_production_cost_change_based_on_kind_of(&self, kindof: &KindOfMaskType) -> Real {
        let none = KindOfMaskType::new();
        if kindof.test_set_and_clear(&self.kindof_cost_percent.0, &none) != FALSE {
            1.0_f32 + self.kindof_cost_percent.1
        } else {
            1.0_f32
        }
    }

    fn get_production_time_change_percent(&self, template_name: &str) -> Real {
        self.production_time_changes
            .get(template_name)
            .copied()
            .unwrap_or(0.0)
    }

    fn get_handicap(&self, ht: HandicapType, tmpl: &ThingTemplate) -> Real {
        let is_struct = tmpl.is_kind_of(KindOfType::Structure) != FALSE;
        self.handicap.get_handicap(ht, is_struct)
    }

    fn get_energy_supply_ratio(&self) -> Real {
        self.energy.get_energy_supply_ratio(0)
    }

    fn count_objects_by_template_ref(&self, _tmpl: ThingTemplateRef) -> Int {
        0
    }
}

#[inline]
fn fnv1a_32(data: &[u8], mut h: u32) -> u32 {
    for &b in data {
        h ^= b as u32;
        h = h.wrapping_mul(16777619);
    }
    h
}

#[inline]
fn mix_u32(h: u32, v: u32) -> u32 {
    fnv1a_32(&v.to_le_bytes(), h)
}

#[inline]
fn mix_i32(h: u32, v: i32) -> u32 {
    fnv1a_32(&v.to_le_bytes(), h)
}

#[inline]
fn mix_f32(h: u32, v: f32) -> u32 {
    mix_u32(h, v.to_bits())
}

fn mix_str(mut h: u32, s: &str) -> u32 {
    let bytes = s.as_bytes();
    h = mix_u32(h, bytes.len() as u32);
    if !bytes.is_empty() {
        h = fnv1a_32(bytes, h);
    }
    h
}

fn mix_wstr(mut h: u32, ws: &[u16]) -> u32 {
    h = mix_u32(h, ws.len() as u32);
    for &ch in ws {
        h = mix_u32(h, ch as u32);
    }
    h
}

fn armor_coef_u32(at: &zh_logic::combat::armor::ArmorTemplate) -> u32 {
    let mut h = mix_f32(2166136261, at.adjust_damage(DamageType::Explosion, 100.0));
    h = mix_f32(h, at.adjust_damage(DamageType::SmallArms, 100.0));
    h = mix_f32(h, at.adjust_damage(DamageType::ArmorPiercing, 100.0));
    h
}

fn make_armor_flags(mask: u32) -> ArmorSetFlags {
    let mut f = ArmorSetFlags::new();
    for i in 0..ARMORSET_COUNT.min(8) {
        if ((mask >> i) & 1) != 0 {
            f.set(i as Int, 1);
        }
    }
    f
}

fn make_weapon_flags(mask: u32) -> WeaponSetFlags {
    let mut f = WeaponSetFlags::new();
    for i in 0..8 {
        if ((mask >> i) & 1) != 0 {
            f.set(i as Int, 1);
        }
    }
    f
}

fn join_mods(tt: &ThingTemplate) -> String {
    let mut acc = String::new();
    let mi = tt.get_behavior_module_info();
    for i in 0..mi.get_count() as usize {
        if i > 0 {
            acc.push(',');
        }
        acc.push_str(mi.get_nth_name(i));
        acc.push(':');
        acc.push_str(mi.get_nth_tag(i));
    }
    acc
}

fn digest_thing_template(
    tt: &ThingTemplate,
    name_keys: &NameKeyGenerator,
    swr: UnsignedShort,
) -> u32 {
    let mut h = 2166136261u32;
    h = mix_u32(h, tt.get_template_id() as u32);
    h = mix_str(h, tt.get_name());
    h = mix_wstr(h, tt.get_display_name());
    h = mix_i32(h, tt.get_default_radar_priority() as Int);
    h = mix_i32(h, tt.get_raw_transport_slot_count());
    h = mix_f32(h, tt.get_fence_width());
    h = mix_f32(h, tt.get_fence_x_offset());
    h = mix_i32(h, if tt.is_bridge() != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.is_bridge_like() != FALSE { 1 } else { 0 });
    h = mix_f32(h, tt.friend_calc_vision_range());
    h = mix_f32(h, tt.friend_calc_shroud_clearing_range());
    h = mix_f32(h, tt.get_shroud_reveal_to_all_range());
    h = mix_f32(h, tt.get_placement_view_angle());
    h = mix_f32(h, tt.get_factory_exit_width());
    h = mix_f32(h, tt.get_factory_extra_bib_width());
    for lvl in 0..LEVEL_COUNT {
        h = mix_i32(h, tt.get_skill_point_value(lvl));
        h = mix_i32(h, tt.get_experience_value(lvl));
        h = mix_i32(h, tt.get_experience_required(lvl));
    }
    h = mix_i32(h, if tt.is_trainable() != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.is_enter_guard() != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.is_hijack_guard() != FALSE { 1 } else { 0 });
    h = mix_str(h, tt.get_default_owning_side());
    h = mix_i32(h, tt.get_prereq_count());
    h = mix_i32(h, tt.get_buildable(None).as_i32());
    h = mix_u32(h, tt.get_build_cost() as u32);
    h = mix_f32(h, tt.get_build_time());
    h = mix_i32(h, tt.get_refund_value());
    h = mix_i32(h, tt.get_build_completion().as_i32());
    h = mix_i32(h, tt.get_energy_production());
    h = mix_i32(h, tt.get_energy_bonus());
    h = mix_i32(h, if tt.is_prerequisite() != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.is_build_facility() != FALSE { 1 } else { 0 });
    h = mix_u32(h, tt.get_display_color());
    h = mix_i32(h, tt.get_editor_sorting().as_i32());
    h = mix_i32(h, if tt.is_kind_of(KindOfType::Structure) != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.is_kind_of(KindOfType::Vehicle) != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.is_kind_of(KindOfType::Infantry) != FALSE { 1 } else { 0 });
    h = mix_i32(
        h,
        if tt.is_kind_of(KindOfType::StickToTerrainSlope) != FALSE {
            1
        } else {
            0
        },
    );
    h = mix_str(h, tt.friend_get_command_set_string());
    h = mix_u32(h, tt.get_build_variations().len() as u32);
    for bv in tt.get_build_variations() {
        h = mix_str(h, bv);
    }
    let mi = tt.get_behavior_module_info();
    h = mix_i32(h, mi.get_count());
    for i in 0..mi.get_count() as usize {
        h = mix_str(h, mi.get_nth_name(i));
        h = mix_str(h, mi.get_nth_tag(i));
    }
    for i in 0..5 {
        h = mix_str(h, tt.get_upgrade_cameo_name(i));
    }
    h = mix_f32(h, tt.get_asset_scale());
    h = mix_f32(h, tt.get_instance_scale_fuzziness());
    h = mix_f32(h, tt.get_structure_rubble_height());
    let gi = tt.get_template_geometry_info();
    h = mix_i32(h, gi.get_geom_type().as_i32());
    h = mix_i32(h, if gi.get_is_small() != FALSE { 1 } else { 0 });
    h = mix_f32(h, gi.get_major_radius());
    h = mix_f32(h, gi.get_minor_radius());
    h = mix_f32(h, gi.get_bounding_circle_radius());
    h = mix_i32(h, tt.get_shadow_type());
    h = mix_f32(h, tt.get_shadow_size_x());
    h = mix_f32(h, tt.get_shadow_size_y());
    h = mix_f32(h, tt.get_shadow_offset_x());
    h = mix_f32(h, tt.get_shadow_offset_y());
    h = mix_str(h, tt.get_shadow_texture_name());
    h = mix_u32(h, tt.get_occlusion_delay());
    h = mix_u32(h, tt.get_threat_value());
    h = mix_u32(h, tt.get_max_simultaneous_of_type(swr));
    let link_key = tt.get_max_simultaneous_link_key();
    let link_str = if link_key == NameKeyType::INVALID {
        ""
    } else {
        name_keys.key_to_name(link_key)
    };
    h = mix_str(h, link_str);
    h = mix_i32(h, tt.get_crushable_level() as Int);
    h = mix_i32(h, tt.get_crusher_level() as Int);
    h = mix_i32(
        h,
        if tt.can_possibly_have_any_weapon() != FALSE {
            1
        } else {
            0
        },
    );
    h = mix_i32(h, if tt.is_buildable_item() != FALSE { 1 } else { 0 });
    h = mix_i32(h, if tt.get_per_unit_fx("FXTest").is_some() { 1 } else { 0 });
    h = mix_str(h, tt.get_per_unit_sound("MoveTest"));
    h
}

fn parse_str_array(line: &str, key: &str) -> Vec<String> {
    let needle = format!("\"{key}\":");
    let start = line.find(&needle).expect("str array key");
    let rest = &line[start + needle.len()..];
    let lbr = rest.find('[').expect("array [");
    let rbr = rest.find(']').expect("array ]");
    let inner = &rest[lbr + 1..rbr];
    let mut out = Vec::new();
    let mut in_q = false;
    let mut cur = String::new();
    for ch in inner.chars() {
        match ch {
            '"' => {
                if in_q {
                    out.push(std::mem::take(&mut cur));
                }
                in_q = !in_q;
            }
            c if in_q => cur.push(c),
            _ => {}
        }
    }
    out
}

fn parse_shared_setup(
    shared_ini: &str,
    name_keys: &mut NameKeyGenerator,
) -> (
    ScienceStore,
    ArmorStore,
    WeaponStore,
    BTreeSet<String>,
    BTreeSet<String>,
) {
    let mut science_store = ScienceStore::new();
    science_store.init();
    let mut armor_store = ArmorStore::new();
    armor_store.init();
    let mut weapon_store = WeaponStore::new();
    let mut fx_list_names = BTreeSet::new();
    let mut damage_fx_names = BTreeSet::new();
    let mut game_text = OracleGameText;

    // Extract Weapon ... End blocks for `WeaponTemplate::parse_ini_block`,
    // and feed Science/FXList/DamageFX/Armor blocks to `IniReader`.
    let mut weapon_blocks: Vec<String> = Vec::new();
    let mut non_weapon_ini = String::new();
    let mut in_weapon = false;
    let mut cur_wpn = String::new();
    for raw_line in shared_ini.lines() {
        let trimmed = raw_line.trim();
        if !in_weapon && trimmed.starts_with("Weapon ") {
            in_weapon = true;
            cur_wpn.clear();
            cur_wpn.push_str(trimmed);
            cur_wpn.push('\n');
        } else if in_weapon {
            cur_wpn.push_str(trimmed);
            cur_wpn.push('\n');
            if trimmed.eq_ignore_ascii_case("End") {
                weapon_blocks.push(std::mem::take(&mut cur_wpn));
                in_weapon = false;
            }
        } else {
            non_weapon_ini.push_str(raw_line);
            non_weapon_ini.push('\n');
        }
    }

    for block in &weapon_blocks {
        let mut wt = WeaponTemplate::new();
        wt.parse_ini_block(block).expect("parse shared Weapon INI");
        weapon_store.add_template(wt);
    }

    let mut ini = IniReader::new("shared_setup.ini", non_weapon_ini.as_bytes(), IniLoadType::Overwrite);
    ini.load_blocks(None, |block, reader| {
        if block.eq_ignore_ascii_case("Science") {
            let _kw = reader.next_token(SEPS_DEFAULT)?;
            science_store.parse_science_definition(reader, name_keys, &mut game_text)?;
            Ok(())
        } else if block.eq_ignore_ascii_case("FXList") {
            let _kw = reader.next_token(SEPS_DEFAULT)?;
            let name = reader.next_token(SEPS_DEFAULT)?.to_owned();
            fx_list_names.insert(name);
            reader.skip_block()
        } else if block.eq_ignore_ascii_case("DamageFX") {
            let _kw = reader.next_token(SEPS_DEFAULT)?;
            let name = reader.next_token(SEPS_DEFAULT)?.to_owned();
            damage_fx_names.insert(name);
            reader.skip_block()
        } else if block.eq_ignore_ascii_case("Armor") {
            let _kw = reader.next_token(SEPS_DEFAULT)?;
            armor_store.parse_armor_definition(reader, name_keys)?;
            Ok(())
        } else {
            panic!("unexpected shared setup block: {block}");
        }
    })
    .expect("parse shared setup blocks");

    (
        science_store,
        armor_store,
        weapon_store,
        fx_list_names,
        damage_fx_names,
    )
}

#[test]
fn test_pkt398_oracle_integrity() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-398\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("ThingTemplate.cpp"));
    assert!(VECTORS_JSON.contains("ThingFactory.cpp"));
    assert!(VECTORS_JSON.contains("Thing.cpp"));
    assert!(VECTORS_JSON.contains("ObjectTypes.cpp"));

    let status_bits = get_i32_array::<5>(VECTORS_JSON, "status_bits").expect("status_bits");
    assert_eq!(
        status_bits,
        [
            OBJECT_STATUS_SCRIPT_DISABLED as Int,
            OBJECT_STATUS_SCRIPT_UNPOWERED as Int,
            OBJECT_STATUS_SCRIPT_UNSELLABLE as Int,
            OBJECT_STATUS_SCRIPT_UNSTEALTHED as Int,
            OBJECT_STATUS_SCRIPT_TARGETABLE as Int,
        ]
    );
    assert_eq!(
        get_i32(VECTORS_JSON, "editor_sorting_count"),
        Some(EditorSortingType::COUNT as Int)
    );

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000, "PKT-398 oracle must contain 1,000 seeds");
    for (idx, line) in lines.iter().enumerate() {
        assert_eq!(get_u32(line, "seed"), Some(idx as u32));
        assert!(get_str(line, "ini").is_some());
        assert!(get_str(line, "ovr_ini").is_some());
        assert!(line.contains("\"tt_crcs\":"));
        assert!(line.contains("\"ot_out\":"));
    }

    let s0 = lines[0];
    for key in [
        "\"s0_default_tmpl\"",
        "\"s0_unit_tmpl\"",
        "\"s0_reskin_tmpl\"",
        "\"s0_override_tmpl\"",
        "\"s0_post_reset\"",
        "\"s0_sparse\"",
        "\"s0_build_math\"",
        "\"s0_thing\"",
        "\"s0_thing_slope\"",
        "\"s0_obj_types\"",
    ] {
        assert!(s0.contains(key), "Seed 0 must include {key}");
    }
}

#[test]
fn test_pkt398_1000_golden_vectors() {
    let shared_ini = get_str(VECTORS_JSON, "shared_ini")
        .expect("setup.shared_ini")
        .replace("\\n", "\n");

    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let (science_store, armor_store, weapon_store, fx_list_names, damage_fx_names) =
        parse_shared_setup(&shared_ini, &mut name_keys);

    let handicap_names = [
        "HANDICAP_BUILDCOST_GENERIC",
        "HANDICAP_BUILDCOST_BUILDINGS",
        "HANDICAP_BUILDTIME_GENERIC",
        "HANDICAP_BUILDTIME_BUILDINGS",
    ];
    let handicap_keys = handicap_names.map(|n| name_keys.name_to_key(n));
    let mut game_text = OracleGameText;
    let module_masks = DefaultModuleMaskLookup;
    let terrain = OracleTerrainLogic;

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (seed, line) in lines.iter().enumerate() {
        let occ = get_u32(line, "occ").expect("occ");
        let gravity = f32::from_bits(get_u32(line, "grav").expect("grav"));
        let epen = f32::from_bits(get_u32(line, "epen").expect("epen"));
        let espd = get_u32_array::<3>(line, "espd").expect("espd");
        let min_e_spd = f32::from_bits(espd[0]);
        let max_e_spd = f32::from_bits(espd[1]);
        let mult_fac = f32::from_bits(espd[2]);
        let swr = get_u32(line, "swr").expect("swr") as UnsignedShort;

        let init_ini = get_str(line, "ini").expect("ini").replace("\\n", "\n");
        let ovr_ini = get_str(line, "ovr_ini")
            .expect("ovr_ini")
            .replace("\\n", "\n");

        let mut factory = ThingFactory::new();
        {
            let mut ctx = ThingTemplateParseContext {
                name_keys: &mut name_keys,
                armor_store: &armor_store,
                weapon_store: &weapon_store,
                science_store: &science_store,
                fx_list_names: &fx_list_names,
                damage_fx_names: &damage_fx_names,
                game_text: &mut game_text,
                module_masks: &module_masks,
                default_occlusion_delay: occ,
                locomotor_store: None,
                module_registry: None,
                module_data_store: None,
                upgrade_center: None,
                ocl_names: None,
                particle_system_names: None,
                special_power_store: None,
                crate_system: None,
            };
            let mut ini_reader =
                IniReader::new("unit_init.ini", init_ini.as_bytes(), IniLoadType::Overwrite);
            factory
                .load_from_ini(&mut ini_reader, &mut ctx)
                .expect("load init_ini");
        }
        factory.post_process_load();

        let pre_ovr_crc = digest_thing_template(
            factory.find_template("UnitMain").expect("UnitMain"),
            &name_keys,
            swr,
        );
        let reskin_crc = digest_thing_template(
            factory.find_template("UnitReskin").expect("UnitReskin"),
            &name_keys,
            swr,
        );

        if seed == 0 {
            let def_tt = factory.find_template("DefaultThingTemplate").unwrap();
            assert_eq!(def_tt.get_template_id(), get_u32(line, "tid").unwrap() as u16);
            assert_eq!(
                def_tt.get_behavior_module_info().get_count(),
                get_i32(line, "mods").unwrap()
            );
            assert_eq!(
                join_mods(def_tt),
                get_str(line, "mod_list").unwrap()
            );

            let unit_tt = factory.find_template("UnitMain").unwrap();
            assert_eq!(
                unit_tt.get_shadow_texture_name(),
                get_str(line, "shadow_tex").unwrap()
            );
            assert_eq!(
                unit_tt.get_skill_point_value(0),
                get_i32(line, "skill0").unwrap()
            );
            assert_eq!(
                unit_tt.get_build_variations().len() as i32,
                get_i32(line, "bvar").unwrap()
            );
            assert_eq!(
                unit_tt.can_possibly_have_any_weapon() as i32,
                get_i32(line, "any_wpn").unwrap()
            );

            let r_tt = factory.find_template("UnitReskin").unwrap();
            assert_eq!(r_tt.get_fence_width().to_bits(), get_u32(line, "fence_w").unwrap());
            assert_eq!(
                r_tt.get_template_geometry_info().get_geom_type().as_i32(),
                get_i32(line, "geom").unwrap()
            );
            assert_eq!(
                r_tt.get_template_geometry_info().get_major_radius().to_bits(),
                get_u32(line, "rad").unwrap()
            );
        }

        // Apply overrides
        {
            let mut ctx = ThingTemplateParseContext {
                name_keys: &mut name_keys,
                armor_store: &armor_store,
                weapon_store: &weapon_store,
                science_store: &science_store,
                fx_list_names: &fx_list_names,
                damage_fx_names: &damage_fx_names,
                game_text: &mut game_text,
                module_masks: &module_masks,
                default_occlusion_delay: occ,
                locomotor_store: None,
                module_registry: None,
                module_data_store: None,
                upgrade_center: None,
                ocl_names: None,
                particle_system_names: None,
                special_power_store: None,
                crate_system: None,
            };
            let mut ovr_reader = IniReader::new(
                "map_ovr.ini",
                ovr_ini.as_bytes(),
                IniLoadType::CreateOverrides,
            );
            factory
                .load_from_ini(&mut ovr_reader, &mut ctx)
                .expect("load ovr_ini");
        }

        let main_final = factory.find_template_final("UnitMain").unwrap();
        let post_ovr_crc = digest_thing_template(main_final, &name_keys, swr);

        if seed == 0 {
            assert_eq!(main_final.get_build_cost() as i32, get_i32(line, "ovr_cost").unwrap());
            assert_eq!(main_final.get_build_time().to_bits(), get_u32(line, "ovr_time").unwrap());
            assert_eq!(
                main_final.friend_calc_vision_range().to_bits(),
                get_u32(line, "ovr_vision").unwrap()
            );
            assert_eq!(
                main_final.get_behavior_module_info().get_count(),
                get_i32(line, "ovr_mods").unwrap()
            );
            assert_eq!(
                main_final.get_prereq_count(),
                get_i32(line, "ovr_prereqs").unwrap()
            );
        }

        let main_base = factory.find_template("UnitMain").unwrap();
        let reskin_tt = factory.find_template("UnitReskin").unwrap();
        let reskin2_tt = factory.find_template("UnitReskin2").unwrap();
        let alt_tt = factory.find_template("AltVarUnit").unwrap();
        let fac_a = factory.find_template("FacilityA").unwrap();
        let fac_b = factory.find_template("FacilityB").unwrap();
        let map_only = factory.find_template("MapOnlyObj");

        let mut eq_mask = 0;
        if main_base.is_equivalent_to(main_final) != FALSE {
            eq_mask |= 1;
        }
        if main_base.is_equivalent_to(reskin_tt) != FALSE {
            eq_mask |= 2;
        }
        if reskin_tt.is_equivalent_to(reskin2_tt) != FALSE {
            eq_mask |= 4;
        }
        if main_base.is_equivalent_to(alt_tt) != FALSE {
            eq_mask |= 8;
        }
        if main_base.is_equivalent_to(fac_a) != FALSE {
            eq_mask |= 16;
        }
        if fac_a.is_build_facility() != FALSE {
            eq_mask |= 32;
        }
        if fac_b.is_build_facility() != FALSE {
            eq_mask |= 64;
        }
        if map_only.is_some() {
            eq_mask |= 128;
        }
        assert_eq!(eq_mask, get_i32(line, "eq_mask").unwrap(), "seed {seed} eq_mask");

        let mut buildable_ovr = BTreeMap::new();
        if seed % 4 == 0 {
            buildable_ovr.insert("UnitMain".to_owned(), BuildableStatus::OnlyByAi);
        }
        assert_eq!(
            main_final.get_buildable(Some(&buildable_ovr)).as_i32(),
            get_i32(line, "ovr_b").unwrap(),
            "seed {seed} ovr_b"
        );

        // SparseMatchFinder queries (test mutable memoizing path too)
        let am_q = get_u32_array::<4>(line, "am_q").unwrap();
        let wm_q = get_u32_array::<4>(line, "wm_q").unwrap();
        let mut am_hash = 2166136261u32;
        let mut wm_hash = 2166136261u32;
        for q in 0..4 {
            let af = make_armor_flags(am_q[q]);
            let ats_mut = factory
                .find_template_final_mut("UnitMain")
                .unwrap()
                .find_armor_template_set_mut(&af)
                .cloned();
            let ats_const = factory
                .find_template_final("UnitMain")
                .unwrap()
                .find_armor_template_set(&af);
            assert_eq!(ats_mut.as_ref(), ats_const, "seed {seed} armor finder cache");
            let ac = ats_const
                .and_then(|s| s.get_armor_template())
                .map_or(0, armor_coef_u32);
            am_hash = mix_u32(am_hash, ac);
            am_hash = mix_i32(
                am_hash,
                if ats_const.is_some_and(zh_logic::combat::armor_set::ArmorTemplateSet::has_damage_fx)
                {
                    1
                } else {
                    0
                },
            );

            let wf = make_weapon_flags(wm_q[q]);
            let wts_mut = factory
                .find_template_final_mut("UnitMain")
                .unwrap()
                .find_weapon_template_set_mut(&wf)
                .cloned();
            let wts_const = factory
                .find_template_final("UnitMain")
                .unwrap()
                .find_weapon_template_set(&wf);
            assert_eq!(wts_mut.as_ref(), wts_const, "seed {seed} weapon finder cache");
            wm_hash = mix_i32(wm_hash, if wts_const.is_some() { 1 } else { 0 });
            if let Some(wts) = wts_const {
                wm_hash = mix_i32(wm_hash, if wts.is_shared_reload_time() != FALSE { 1 } else { 0 });
                wm_hash = mix_i32(
                    wm_hash,
                    if wts.is_weapon_lock_shared_across_sets() != FALSE {
                        1
                    } else {
                        0
                    },
                );
                wm_hash = mix_i32(
                    wm_hash,
                    if wts.get_nth(zh_logic::object::weapon_set::WeaponSlotType::Primary).is_some()
                    {
                        1
                    } else {
                        0
                    },
                );
                wm_hash = mix_i32(
                    wm_hash,
                    if wts.get_nth(zh_logic::object::weapon_set::WeaponSlotType::Secondary).is_some()
                    {
                        1
                    } else {
                        0
                    },
                );
            }
        }
        let want_sm = get_u32_array::<2>(line, "sm_h").unwrap();
        assert_eq!([am_hash, wm_hash], want_sm, "seed {seed} sparse hashes");

        // Build cost & build time math
        let p_pcts = get_u32_array::<3>(line, "p_pcts").unwrap();
        let p_hc = get_u32_array::<4>(line, "p_hc").unwrap();
        let p_en = get_i32_array::<2>(line, "p_en").unwrap();

        let mut hc_dict = Dict::new();
        for i in 0..4 {
            hc_dict.set_real(handicap_keys[i], f32::from_bits(p_hc[i]));
        }
        let mut handicap = Handicap::new();
        handicap.read_from_dict(&hc_dict, &mut name_keys);

        let mut energy = Energy::new();
        energy.init(None);
        let mut noop = NoopEnergyCallbacks;
        energy.adjust_power(p_en[0], TRUE, 0, &mut noop);
        energy.adjust_power(-p_en[1], TRUE, 0, &mut noop);

        let mut ko_mask = KindOfMaskType::new();
        ko_mask.set(
            if seed % 3 == 0 {
                KindOfType::Structure.as_i32()
            } else {
                KindOfType::Vehicle.as_i32()
            },
            1,
        );
        let mut cost_map = BTreeMap::new();
        let mut time_map = BTreeMap::new();
        cost_map.insert("UnitMain".to_owned(), f32::from_bits(p_pcts[0]));
        time_map.insert("UnitMain".to_owned(), f32::from_bits(p_pcts[1]));

        let player_view = OraclePlayerView {
            production_cost_changes: cost_map,
            production_time_changes: time_map,
            kindof_cost_percent: (ko_mask, f32::from_bits(p_pcts[2])),
            handicap,
            energy,
        };
        let tunables = ThingTemplateGlobalTunables {
            low_energy_penalty_modifier: epen,
            min_low_energy_production_speed: min_e_spd,
            max_low_energy_production_speed: max_e_spd,
            multiple_factory: mult_fac,
        };
        let main_final = factory.find_template_final("UnitMain").unwrap();
        let calc_cost = main_final.calc_cost_to_build(&player_view);
        let calc_time = main_final.calc_time_to_build(&player_view, &tunables);
        let want_build = get_i32_array::<2>(line, "build").unwrap();
        if seed == 0 {
            assert_eq!(
                player_view.get_energy_supply_ratio().to_bits(),
                get_u32(line, "eratio").unwrap(),
                "seed 0 eratio"
            );
            assert_eq!(
                player_view.get_handicap(HandicapType::BuildCost, main_final).to_bits(),
                get_u32(line, "hc_cost").unwrap(),
                "seed 0 hc_cost"
            );
            assert_eq!(
                player_view.get_handicap(HandicapType::BuildTime, main_final).to_bits(),
                get_u32(line, "hc_time").unwrap(),
                "seed 0 hc_time"
            );
        }
        assert_eq!([calc_cost, calc_time], want_build, "seed {seed} build math");

        // Thing & slope Thing tests
        let th_in = get_u32_array::<8>(line, "th_in").unwrap();
        let th_x = f32::from_bits(th_in[0]);
        let th_y = f32::from_bits(th_in[1]);
        let th_z = f32::from_bits(th_in[2]);
        let th_z1 = f32::from_bits(th_in[3]);
        let th_ang = f32::from_bits(th_in[4]);
        let bone_in = Coord3D::new(
            f32::from_bits(th_in[5]),
            f32::from_bits(th_in[6]),
            f32::from_bits(th_in[7]),
        );

        let main_id = factory.find_template_id("UnitMain").unwrap();
        let mut th = Thing::new(main_id);
        let mut reactor = OracleTransformReactor::default();

        let pos_before = th.is_positioned();
        let c0 = Coord3D::new(th_x, th_y, th_z);
        th.set_position(&factory, &c0, &terrain, &mut reactor);
        let pos_after = th.is_positioned();
        let h_terr0 = th.get_height_above_terrain(&terrain);
        let h_water0 = th.get_height_above_terrain_or_water(&terrain);
        let is_above0 = th.is_above_terrain(&terrain);
        let sig_above0 = th.is_significantly_above_terrain(&terrain, gravity);
        th.set_position_z(&factory, th_z1, &terrain, &mut reactor);
        let h_terr1 = th.get_height_above_terrain(&terrain);
        let h_water1 = th.get_height_above_terrain_or_water(&terrain);
        th.set_orientation(&factory, th_ang, &terrain, &mut reactor);
        let dir2 = th.get_unit_direction_vector_2d();
        let dir3 = th.get_unit_direction_vector_3d();

        let mut bone_mtx = Matrix3D::identity();
        bone_mtx.rotate_z(0.25_f32);
        bone_mtx.translate(2.0_f32, -1.0_f32, 3.0_f32);
        let mut world_pt = Coord3D::default();
        let mut world_mtx = Matrix3D::identity();
        th.convert_bone_pos_to_world_pos(
            Some(&bone_in),
            Some(&bone_mtx),
            Some(&mut world_pt),
            Some(&mut world_mtx),
        );
        let xform_pt = th.transform_point(&bone_in);

        let th_mtx_raw = get_u32_array::<12>(line, "th_mtx").unwrap();
        let mut custom_mx = Matrix3D::identity();
        custom_mx.set(
            f32::from_bits(th_mtx_raw[0]),
            f32::from_bits(th_mtx_raw[1]),
            f32::from_bits(th_mtx_raw[2]),
            f32::from_bits(th_mtx_raw[3]),
            f32::from_bits(th_mtx_raw[4]),
            f32::from_bits(th_mtx_raw[5]),
            f32::from_bits(th_mtx_raw[6]),
            f32::from_bits(th_mtx_raw[7]),
            f32::from_bits(th_mtx_raw[8]),
            f32::from_bits(th_mtx_raw[9]),
            f32::from_bits(th_mtx_raw[10]),
            f32::from_bits(th_mtx_raw[11]),
        );
        th.set_transform_matrix(&custom_mx, &mut reactor);
        let cached_ang_after_mx = th.get_orientation();
        let cached_pos_after_mx = *th.get_position();

        let mut th_hash = mix_i32(2166136261, if pos_before != FALSE { 1 } else { 0 });
        th_hash = mix_i32(th_hash, if pos_after != FALSE { 1 } else { 0 });
        th_hash = mix_f32(th_hash, h_terr0);
        th_hash = mix_f32(th_hash, h_water0);
        th_hash = mix_i32(th_hash, if is_above0 != FALSE { 1 } else { 0 });
        th_hash = mix_i32(th_hash, if sig_above0 != FALSE { 1 } else { 0 });
        th_hash = mix_f32(th_hash, h_terr1);
        th_hash = mix_f32(th_hash, h_water1);
        th_hash = mix_f32(th_hash, dir2.x);
        th_hash = mix_f32(th_hash, dir2.y);
        th_hash = mix_f32(th_hash, dir3.x);
        th_hash = mix_f32(th_hash, dir3.y);
        th_hash = mix_f32(th_hash, dir3.z);
        th_hash = mix_f32(th_hash, world_pt.x);
        th_hash = mix_f32(th_hash, world_pt.y);
        th_hash = mix_f32(th_hash, world_pt.z);
        th_hash = mix_f32(th_hash, xform_pt.x);
        th_hash = mix_f32(th_hash, xform_pt.y);
        th_hash = mix_f32(th_hash, xform_pt.z);
        th_hash = mix_f32(th_hash, cached_ang_after_mx);
        th_hash = mix_f32(th_hash, cached_pos_after_mx.x);
        th_hash = mix_f32(th_hash, cached_pos_after_mx.y);
        th_hash = mix_f32(th_hash, cached_pos_after_mx.z);
        th_hash = mix_i32(th_hash, reactor.react_count);

        let slope_id = factory.find_template_id("SlopeUnit").unwrap();
        let mut slope_th = Thing::new(slope_id);
        let mut slope_reactor = OracleTransformReactor::default();
        slope_th.set_position(&factory, &c0, &terrain, &mut slope_reactor);
        slope_th.set_position_z(&factory, th_z1, &terrain, &mut slope_reactor);
        slope_th.set_orientation(&factory, th_ang, &terrain, &mut slope_reactor);
        let slope_dir3 = slope_th.get_unit_direction_vector_3d();

        let mut slope_hash = mix_f32(2166136261, slope_th.get_position().x);
        slope_hash = mix_f32(slope_hash, slope_th.get_position().y);
        slope_hash = mix_f32(slope_hash, slope_th.get_position().z);
        slope_hash = mix_f32(slope_hash, slope_th.get_orientation());
        slope_hash = mix_f32(slope_hash, slope_dir3.x);
        slope_hash = mix_f32(slope_hash, slope_dir3.y);
        slope_hash = mix_f32(slope_hash, slope_dir3.z);
        slope_hash = mix_i32(slope_hash, slope_reactor.react_count);

        if seed == 0 {
            assert_eq!(h_terr0.to_bits(), get_u32(line, "ht0").unwrap(), "seed 0 ht0");
            assert_eq!(h_water0.to_bits(), get_u32(line, "hw0").unwrap(), "seed 0 hw0");
            assert_eq!(h_terr1.to_bits(), get_u32(line, "ht1").unwrap(), "seed 0 ht1");
            assert_eq!(
                [dir2.x.to_bits(), dir2.y.to_bits()],
                get_u32_array::<2>(line, "dir2").unwrap(),
                "seed 0 dir2"
            );
            assert_eq!(
                [dir3.x.to_bits(), dir3.y.to_bits(), dir3.z.to_bits()],
                get_u32_array::<3>(line, "dir3").unwrap(),
                "seed 0 dir3"
            );
            assert_eq!(
                [world_pt.x.to_bits(), world_pt.y.to_bits(), world_pt.z.to_bits()],
                get_u32_array::<3>(line, "world_pt").unwrap(),
                "seed 0 world_pt"
            );
            assert_eq!(
                [xform_pt.x.to_bits(), xform_pt.y.to_bits(), xform_pt.z.to_bits()],
                get_u32_array::<3>(line, "xform_pt").unwrap(),
                "seed 0 xform_pt"
            );
            assert_eq!(
                [
                    cached_pos_after_mx.x.to_bits(),
                    cached_pos_after_mx.y.to_bits(),
                    cached_pos_after_mx.z.to_bits(),
                ],
                get_u32_array::<3>(line, "pos_mx").unwrap(),
                "seed 0 pos_mx"
            );
            assert_eq!(
                cached_ang_after_mx.to_bits(),
                get_u32(line, "ang_mx").unwrap(),
                "seed 0 ang_mx"
            );
            assert_eq!(
                slope_th.get_position().z.to_bits(),
                get_u32(line, "pz").unwrap(),
                "seed 0 slope pz"
            );
            assert_eq!(
                [
                    slope_dir3.x.to_bits(),
                    slope_dir3.y.to_bits(),
                    slope_dir3.z.to_bits(),
                ],
                get_u32_array::<3>(line, "d3").unwrap(),
                "seed 0 slope d3"
            );
        }

        let want_th = get_u32_array::<2>(line, "th_h").unwrap();
        assert_eq!([th_hash, slope_hash], want_th, "seed {seed} Thing hashes");

        // ObjectTypes tests
        let ot_names = parse_str_array(line, "ot_names");
        let ot_rm = get_str(line, "ot_rm").unwrap();
        let mut ot = ObjectTypes::with_name(format!("Group_{seed}"));
        for nm in &ot_names {
            ot.add_object_type(nm);
        }
        ot.add_object_type(&ot_names[0]);
        ot.remove_object_type(ot_rm);

        let mut ot_in_mask = 0;
        if ot.is_in_set_template(factory.find_template("UnitMain")) != FALSE {
            ot_in_mask |= 1;
        }
        if ot.is_in_set_template(factory.find_template("FacilityA")) != FALSE {
            ot_in_mask |= 2;
        }
        if ot.is_in_set_template(factory.find_template("UnitReskin")) != FALSE {
            ot_in_mask |= 4;
        }
        if ot.is_in_set_template(factory.find_template("AltVarUnit")) != FALSE {
            ot_in_mask |= 8;
        }
        let mut resolved = Vec::new();
        let mut counts = Vec::new();
        let ot_prep_cnt = ot.prep_for_player_counting(&factory, &mut resolved, &mut counts);

        let mut otsave = XferSave::new();
        otsave.xfer_snapshot(&mut ot).unwrap();
        let save_bytes = otsave.into_bytes();
        let mut crc_acc = zh_common::crc::Crc::new();
        crc_acc.compute_crc(&save_bytes, save_bytes.len() as Int);
        let save_crc = crc_acc.get();

        let mut ot_loaded = ObjectTypes::new();
        let mut otload = XferLoad::new(&save_bytes);
        otload.xfer_snapshot(&mut ot_loaded).unwrap();

        let mut otxcrc = XferCrc::new();
        otxcrc.xfer_snapshot(&mut ot_loaded).unwrap();

        if seed == 0 {
            assert_eq!(
                encode_hex_bytes(&save_bytes),
                get_str(line, "ot_xhex").unwrap(),
                "seed 0 ot_xhex"
            );
        }
        let want_ot = get_u32_array::<5>(line, "ot_out").unwrap();
        assert_eq!(
            [
                ot_in_mask as u32,
                ot_prep_cnt as u32,
                save_bytes.len() as u32,
                save_crc,
                otxcrc.get_crc(),
            ],
            want_ot,
            "seed {seed} ObjectTypes outputs"
        );

        // Reset factory & check post-reset invariants
        factory.reset();
        let after_reset_main = factory.find_template("UnitMain").unwrap();
        let after_reset_map_only = factory.find_template("MapOnlyObj");
        let post_reset_crc = digest_thing_template(after_reset_main, &name_keys, swr);
        let map_gone = if after_reset_map_only.is_none() { 1 } else { 0 };
        let rst_nid = factory.next_template_id();

        if seed == 0 {
            assert_eq!(
                after_reset_main.get_build_cost() as i32,
                get_i32(line, "restored_cost").unwrap()
            );
            assert_eq!(
                after_reset_main.get_behavior_module_info().get_count(),
                get_i32(line, "restored_mods").unwrap()
            );
            assert_eq!(rst_nid as i32, get_i32(line, "next_id").unwrap());
        }

        let want_tt_crcs = get_u32_array::<4>(line, "tt_crcs").unwrap();
        assert_eq!(
            [pre_ovr_crc, reskin_crc, post_ovr_crc, post_reset_crc],
            want_tt_crcs,
            "seed {seed} ThingTemplate CRCs"
        );
        assert_eq!(map_gone, get_i32(line, "map_gone").unwrap(), "seed {seed} map_gone");
        assert_eq!(rst_nid as i32, get_i32(line, "rst_nid").unwrap(), "seed {seed} rst_nid");
    }
}
