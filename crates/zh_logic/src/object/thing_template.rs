//! `PKT-398`: Core entity blueprint (`Core/GameEngine/Source/Common/Thing/ThingTemplate.cpp`).

use std::collections::{BTreeMap, BTreeSet};

use crate::arena::{ModuleDataId, ThingTemplateId};
use crate::combat::armor::ArmorStore;
use crate::combat::armor_set::ArmorTemplateSet;
use crate::combat::locomotor::LocomotorStore;
use crate::combat::locomotor_set::{LocomotorSetType, LOCOMOTORSET_COUNT, LOCOMOTOR_SET_NAMES};
use crate::economy::handicap::HandicapType;
use crate::modules::{ModuleDataStore, ModuleParseContext, ModuleRegistry};
use crate::object::weapon::WeaponStore;
use crate::object::weapon_set::WeaponTemplateSet;
use crate::player::radar::RadarPriorityType;
use crate::player::sparse_match_finder::SparseMatchFinder;
use crate::rules::combat_types::{ArmorSetFlags, WeaponSetFlags};
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
use crate::rules::production_prerequisite::{
    PrereqPlayerView, ProductionPrerequisite, ThingTemplateRef, ThingTemplateRegistry,
};
use crate::rules::science::{ScienceStore, SCIENCE_INVALID};
use crate::rules::upgrade::UpgradeCenter;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{
    Color, IniError, IniLoadType, IniReader, LOGICFRAMES_PER_SECONDS_REAL, SEPS_DEFAULT,
};
use zh_common::math::geometry::{GeometryInfo, GeometryType, GEOMETRY_NAMES};
use zh_common::runtime::thing_sort::{parse_editor_sorting_type, EditorSortingType};
use zh_common::types::{
    Bool, Byte, Int, NameKeyType, Real, UnsignedByte, UnsignedInt, UnsignedShort, FALSE, TRUE,
};

pub const LEVEL_COUNT: usize = 4;
pub const MAX_UPGRADE_CAMEO_UPGRADES: usize = 5;
pub const USE_EXP_VALUE_FOR_SKILL_VALUE: Int = -999;

pub const RADAR_PRIORITY_NAMES: [&str; 5] = [
    "INVALID",
    "NOT_ON_RADAR",
    "STRUCTURE",
    "UNIT",
    "LOCAL_UNIT_ONLY",
];

pub const MODULEINTERFACE_UPDATE: Int = 0x0000_0001;
pub const MODULEINTERFACE_DIE: Int = 0x0000_0002;
pub const MODULEINTERFACE_DAMAGE: Int = 0x0000_0004;
pub const MODULEINTERFACE_CREATE: Int = 0x0000_0008;
pub const MODULEINTERFACE_COLLIDE: Int = 0x0000_0010;
pub const MODULEINTERFACE_BODY: Int = 0x0000_0020;
pub const MODULEINTERFACE_CONTAIN: Int = 0x0000_0040;
pub const MODULEINTERFACE_UPGRADE: Int = 0x0000_0080;
pub const MODULEINTERFACE_SPECIAL_POWER: Int = 0x0000_0100;
pub const MODULEINTERFACE_DESTROY: Int = 0x0000_0200;
pub const MODULEINTERFACE_DRAW: Int = 0x0000_0400;
pub const MODULEINTERFACE_CLIENT_UPDATE: Int = 0x0000_0800;

/// Module category enumeration matching `Module.h`.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleType {
    #[default]
    Behavior = 0,
    Draw = 1,
    ClientUpdate = 2,
}

/// Internal module parsing mode during object INI parsing (`ThingTemplate.h`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleParseMode {
    #[default]
    Normal = 0,
    AddRemoveReplace = 1,
    Inheritable = 2,
    OverrideableByLikeKind = 3,
}

/// Unit/structure placement behavior upon build completion (`ThingTemplate.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildCompletionType {
    Invalid = 0,
    #[default]
    AppearsAtRallyPoint = 1,
    PlacedByPlayer = 2,
}

impl BuildCompletionType {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::AppearsAtRallyPoint),
            2 => Some(Self::PlacedByPlayer),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const BUILD_COMPLETION_NAMES: [&str; 3] =
    ["INVALID", "APPEARS_AT_RALLY_POINT", "PLACED_BY_PLAYER"];

/// Unit buildability status (`ThingTemplate.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BuildableStatus {
    #[default]
    Yes = 0,
    IgnorePrerequisites = 1,
    No = 2,
    OnlyByAi = 3,
}

impl BuildableStatus {
    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Yes),
            1 => Some(Self::IgnorePrerequisites),
            2 => Some(Self::No),
            3 => Some(Self::OnlyByAi),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const BUILDABLE_STATUS_NAMES: [&str; 4] = ["Yes", "Ignore_Prerequisites", "No", "Only_By_AI"];

/// Shadow bitflags (`GameClient/Shadow.h`).
pub const SHADOW_NONE: Int = 0x00;
pub const SHADOW_DECAL: Int = 0x01;
pub const SHADOW_VOLUME: Int = 0x02;
pub const SHADOW_PROJECTION: Int = 0x04;
pub const SHADOW_DYNAMIC_PROJECTION: Int = 0x08;
pub const SHADOW_DIRECTIONAL_PROJECTION: Int = 0x10;
pub const SHADOW_ALPHA_DECAL: Int = 0x20;
pub const SHADOW_ADDITIVE_DECAL: Int = 0x40;

pub const THE_SHADOW_NAMES: [&str; 7] = [
    "SHADOW_DECAL",
    "SHADOW_VOLUME",
    "SHADOW_PROJECTION",
    "SHADOW_DYNAMIC_PROJECTION",
    "SHADOW_DIRECTIONAL_PROJECTION",
    "SHADOW_ALPHA_DECAL",
    "SHADOW_ADDITIVE_DECAL",
];

/// Single registered module descriptor inside `ModuleInfo` (`ThingTemplate.h`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleNugget {
    pub name: String,
    pub module_tag: String,
    pub tag_key: NameKeyType,
    pub data_id: ModuleDataId,
    pub interface_mask: Int,
    pub is_ai_module_data: bool,
    pub copied_from_default: bool,
    pub inheritable: bool,
    pub overrideable_by_like_kind: bool,
}

/// Ordered module list (`ModuleInfo` in `ThingTemplate.h/.cpp`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ModuleInfo {
    info: Vec<ModuleNugget>,
}

impl ModuleInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.info.clear();
    }

    #[inline]
    pub fn get_count(&self) -> Int {
        self.info.len() as Int
    }

    #[inline]
    pub fn get_nth_name(&self, i: usize) -> &str {
        self.info.get(i).map_or("", |n| n.name.as_str())
    }

    #[inline]
    pub fn get_nth_tag(&self, i: usize) -> &str {
        self.info.get(i).map_or("", |n| n.module_tag.as_str())
    }

    #[inline]
    pub fn get_nugget_with_tag(&self, tag: &str) -> Option<&ModuleNugget> {
        self.info.iter().find(|n| n.module_tag == tag)
    }

    #[inline]
    pub fn nuggets(&self) -> &[ModuleNugget] {
        &self.info
    }

    #[inline]
    pub fn set_copied_from_default(&mut self, v: bool) {
        for n in &mut self.info {
            n.copied_from_default = v;
        }
    }

    pub fn add_module_info(
        &mut self,
        name: String,
        module_tag: String,
        interface_mask: Int,
        is_ai_module_data: bool,
        inheritable: bool,
        overrideable_by_like_kind: bool,
    ) {
        self.add_module_info_with_data(
            name,
            module_tag,
            NameKeyType::INVALID,
            ModuleDataId::INVALID,
            interface_mask,
            is_ai_module_data,
            inheritable,
            overrideable_by_like_kind,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn add_module_info_with_data(
        &mut self,
        name: String,
        module_tag: String,
        tag_key: NameKeyType,
        data_id: ModuleDataId,
        interface_mask: Int,
        is_ai_module_data: bool,
        inheritable: bool,
        overrideable_by_like_kind: bool,
    ) {
        self.info.push(ModuleNugget {
            name,
            module_tag,
            tag_key,
            data_id,
            interface_mask,
            is_ai_module_data,
            copied_from_default: false,
            inheritable,
            overrideable_by_like_kind,
        });
    }

    pub fn clear_module_data_with_tag(&mut self, tag_to_clear: &str) -> Option<String> {
        let mut cleared_name = None;
        let mut i = 0;
        while i < self.info.len() {
            if self.info[i].module_tag == tag_to_clear {
                cleared_name = Some(self.info[i].name.clone());
                self.info.remove(i);
            } else {
                i += 1;
            }
        }
        cleared_name
    }

    pub fn clear_ai_module_info(&mut self) -> bool {
        let before = self.info.len();
        self.info.retain(|n| !n.is_ai_module_data);
        self.info.len() != before
    }

    /// Mirrors `ModuleInfo::clearCopiedFromDefaultEntries` (`ThingTemplate.cpp:1416-1491`).
    pub fn clear_copied_from_default_entries(
        &mut self,
        interface_mask: Int,
        new_name: &str,
        kindof: &KindOfMaskType,
        is_trainable: Bool,
    ) -> bool {
        let immune_mask = KindOfMaskType::from_indices(&[
            KindOfType::Aircraft.as_i32(),
            KindOfType::Shrubbery.as_i32(),
            KindOfType::OptimizedTree.as_i32(),
            KindOfType::Structure.as_i32(),
            KindOfType::DrawableOnly.as_i32(),
            KindOfType::MobNexus.as_i32(),
            KindOfType::IgnoredInGui.as_i32(),
            KindOfType::ClearedByBuild.as_i32(),
            KindOfType::DefensiveWall.as_i32(),
            KindOfType::BallisticMissile.as_i32(),
            KindOfType::SupplySource.as_i32(),
            KindOfType::Boat.as_i32(),
            KindOfType::Inert.as_i32(),
            KindOfType::Bridge.as_i32(),
            KindOfType::LandmarkBridge.as_i32(),
            KindOfType::BridgeTower.as_i32(),
        ]);
        let disallowed = kindof.test_for_any(&immune_mask) != FALSE;

        let candidate_mask = KindOfMaskType::from_indices(&[
            KindOfType::Score.as_i32(),
            KindOfType::Vehicle.as_i32(),
            KindOfType::Infantry.as_i32(),
            KindOfType::PortableStructure.as_i32(),
        ]);
        let candidate = kindof.test_for_any(&candidate_mask) != FALSE;

        let mut ret = false;
        let mut i = 0;
        while i < self.info.len() {
            let nugget = &self.info[i];
            if (nugget.interface_mask & interface_mask) != 0 && nugget.copied_from_default {
                if nugget.inheritable {
                    #[cfg(not(feature = "retail-crc"))]
                    if nugget.module_tag == "ModuleTag_DefaultAutoHealBehavior"
                        && is_trainable == FALSE
                    {
                        self.info.remove(i);
                        ret = true;
                        continue;
                    }
                    #[cfg(feature = "retail-crc")]
                    let _ = is_trainable;
                    i += 1;
                } else if nugget.overrideable_by_like_kind {
                    if nugget.name == new_name || disallowed || !candidate {
                        self.info.remove(i);
                        ret = true;
                    } else {
                        i += 1;
                    }
                } else {
                    self.info.remove(i);
                    ret = true;
                }
            } else {
                i += 1;
            }
        }
        ret
    }
}

/// Localized display label lookup adapter (`TheGameText->fetch`).
pub trait GameTextLookup {
    fn fetch_utf16(&mut self, label: &str) -> Vec<u16>;
}

/// Module interface mask & AI-module metadata adapter (`TheModuleFactory`).
pub trait ModuleMaskLookup {
    fn find_module_interface_mask(&self, name: &str, mod_type: ModuleType) -> Int;
    fn is_ai_module_name(&self, _name: &str) -> bool {
        false
    }
}

/// Built-in Zero Hour module mask lookup table covering core Behavior/Body/Draw/ClientUpdate types.
#[derive(Debug, Clone, Default)]
pub struct DefaultModuleMaskLookup;

impl ModuleMaskLookup for DefaultModuleMaskLookup {
    fn find_module_interface_mask(&self, name: &str, mod_type: ModuleType) -> Int {
        match mod_type {
            ModuleType::Draw => MODULEINTERFACE_DRAW,
            ModuleType::ClientUpdate => MODULEINTERFACE_CLIENT_UPDATE,
            ModuleType::Behavior => match name {
                "InactiveBody" | "ActiveBody" | "StructureBody" | "ImmortalBody" | "HighlanderBody"
                | "HiveStructureBody" => MODULEINTERFACE_BODY,
                "AutoHealBehavior" => {
                    MODULEINTERFACE_UPDATE | MODULEINTERFACE_UPGRADE | MODULEINTERFACE_DAMAGE
                }
                "LifetimeUpdate" | "PhysicsBehavior" | "ProductionUpdate" | "SpecialPowerUpdate" => {
                    MODULEINTERFACE_UPDATE
                }
                "DestroyDie" | "KeepObjectDie" | "CrushDie" | "CreateObjectDie"
                | "FXListDie" | "SlowDeathBehavior" => MODULEINTERFACE_DIE,
                _ => {
                    if name.ends_with("Body") {
                        MODULEINTERFACE_BODY
                    } else if name.ends_with("Die") {
                        MODULEINTERFACE_DIE
                    } else {
                        MODULEINTERFACE_UPDATE
                    }
                }
            },
        }
    }

    fn is_ai_module_name(&self, name: &str) -> bool {
        name.contains("AIUpdate")
    }
}

/// Dependencies supplied while parsing `ThingTemplate` INI blocks.
pub struct ThingTemplateParseContext<'a> {
    pub name_keys: &'a mut NameKeyGenerator,
    pub armor_store: &'a ArmorStore,
    pub weapon_store: &'a WeaponStore,
    pub science_store: &'a ScienceStore,
    pub fx_list_names: &'a BTreeSet<String>,
    pub damage_fx_names: &'a BTreeSet<String>,
    pub game_text: &'a mut dyn GameTextLookup,
    pub module_masks: &'a dyn ModuleMaskLookup,
    pub default_occlusion_delay: UnsignedInt,
    pub locomotor_store: Option<&'a LocomotorStore>,
    pub module_registry: Option<&'a ModuleRegistry>,
    pub module_data_store: Option<&'a mut ModuleDataStore>,
    pub upgrade_center: Option<&'a UpgradeCenter>,
    pub ocl_names: Option<&'a BTreeSet<String>>,
    pub particle_system_names: Option<&'a BTreeSet<String>>,
}

/// Cost/time/facility parameters provided by a `Player` to `ThingTemplate::calc_cost_to_build`
/// and `ThingTemplate::calc_time_to_build`.
pub trait ThingTemplatePlayerView {
    fn as_prereq_view(&self) -> Option<&dyn PrereqPlayerView>;
    fn get_production_cost_change_percent(&self, template_name: &str) -> Real;
    fn get_production_cost_change_based_on_kind_of(&self, kindof: &KindOfMaskType) -> Real;
    fn get_production_time_change_percent(&self, template_name: &str) -> Real;
    fn get_handicap(&self, ht: HandicapType, tmpl: &ThingTemplate) -> Real;
    fn get_energy_supply_ratio(&self) -> Real;
    fn count_objects_by_template_ref(&self, tmpl: ThingTemplateRef) -> Int;
}

/// Global tuning parameters consumed by `ThingTemplate::calc_time_to_build`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ThingTemplateGlobalTunables {
    pub low_energy_penalty_modifier: Real,
    pub min_low_energy_production_speed: Real,
    pub max_low_energy_production_speed: Real,
    pub multiple_factory: Real,
}

/// Core entity blueprint (`Core/GameEngine/Include/Common/ThingTemplate.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct ThingTemplate {
    pub display_name: Vec<u16>,
    pub name_string: String,
    pub default_owning_side: String,
    pub command_set_string: String,
    pub selected_portrait_image_name: String,
    pub button_image_name: String,
    pub upgrade_cameo_upgrade_names: [String; MAX_UPGRADE_CAMEO_UPGRADES],
    pub shadow_texture_name: String,
    pub module_being_replaced_name: String,
    pub module_being_replaced_tag: String,

    pub geometry_info: GeometryInfo,
    pub kindof: KindOfMaskType,
    pub audio_events: BTreeMap<String, String>,
    pub behavior_module_info: ModuleInfo,
    pub draw_module_info: ModuleInfo,
    pub client_update_module_info: ModuleInfo,

    pub skill_point_values: [Int; LEVEL_COUNT],
    pub experience_values: [Int; LEVEL_COUNT],
    pub experience_required: [Int; LEVEL_COUNT],

    pub prereq_info: Vec<ProductionPrerequisite>,
    pub build_variations: Vec<String>,
    pub weapon_template_sets: Vec<WeaponTemplateSet>,
    pub weapon_template_set_finder: SparseMatchFinder<WeaponTemplateSet, WeaponSetFlags, true>,
    pub armor_template_sets: Vec<ArmorTemplateSet>,
    pub armor_template_set_finder: SparseMatchFinder<ArmorTemplateSet, ArmorSetFlags, true>,
    pub locomotor_sets: [Vec<String>; LOCOMOTORSET_COUNT],
    pub per_unit_sounds: BTreeMap<String, String>,
    pub per_unit_fx: BTreeMap<String, Option<String>>,

    pub reskinned_from_id: Option<UnsignedShort>,

    pub fence_width: Real,
    pub fence_x_offset: Real,
    pub vision_range: Real,
    pub shroud_clearing_range: Real,
    pub shroud_reveal_to_all_range: Real,
    pub placement_view_angle: Real,
    pub factory_exit_width: Real,
    pub factory_extra_bib_width: Real,
    pub build_time: Real,
    pub asset_scale: Real,
    pub instance_scale_fuzziness: Real,
    pub shadow_size_x: Real,
    pub shadow_size_y: Real,
    pub shadow_offset_x: Real,
    pub shadow_offset_y: Real,

    pub energy_production: Int,
    pub energy_bonus: Int,
    pub display_color: Color,
    pub occlusion_delay: UnsignedInt,
    pub max_simultaneous_link_key: NameKeyType,

    pub template_id: UnsignedShort,
    pub build_cost: UnsignedShort,
    pub refund_value: UnsignedShort,
    pub threat_value: UnsignedShort,
    pub max_simultaneous_of_type: UnsignedShort,

    pub max_simultaneous_determined_by_superweapon_restriction: Bool,
    pub is_prerequisite: Bool,
    pub is_bridge: Bool,
    pub is_build_facility: Bool,
    pub is_trainable: Bool,
    pub enter_guard: Bool,
    pub hijack_guard: Bool,
    pub is_forbidden: Bool,
    pub armor_copied_from_default: Bool,
    pub weapons_copied_from_default: Bool,

    pub radar_priority: RadarPriorityType,
    pub transport_slot_count: UnsignedByte,
    pub buildable: BuildableStatus,
    pub build_completion: BuildCompletionType,
    pub editor_sorting: EditorSortingType,
    pub structure_rubble_height: UnsignedByte,
    pub shadow_type: Int,
    pub module_parsing_mode: ModuleParseMode,
    pub crusher_level: UnsignedByte,
    pub crushable_level: UnsignedByte,
}

impl Default for ThingTemplate {
    fn default() -> Self {
        Self::new(90)
    }
}

impl ThingTemplate {
    /// Constructs a new `ThingTemplate` with exact C++ constructor defaults (`ThingTemplate.cpp:2001`).
    pub fn new(default_occlusion_delay: UnsignedInt) -> Self {
        Self {
            display_name: Vec::new(),
            name_string: String::new(),
            default_owning_side: String::new(),
            command_set_string: String::new(),
            selected_portrait_image_name: String::new(),
            button_image_name: String::new(),
            upgrade_cameo_upgrade_names: std::array::from_fn(|_| String::new()),
            shadow_texture_name: String::new(),
            module_being_replaced_name: String::new(),
            module_being_replaced_tag: String::new(),

            geometry_info: GeometryInfo::new(GeometryType::Sphere, FALSE, 1.0, 1.0, 1.0),
            kindof: KindOfMaskType::new(),
            audio_events: BTreeMap::new(),
            behavior_module_info: ModuleInfo::new(),
            draw_module_info: ModuleInfo::new(),
            client_update_module_info: ModuleInfo::new(),

            skill_point_values: [USE_EXP_VALUE_FOR_SKILL_VALUE; LEVEL_COUNT],
            experience_values: [0; LEVEL_COUNT],
            experience_required: [0; LEVEL_COUNT],

            prereq_info: Vec::new(),
            build_variations: Vec::new(),
            weapon_template_sets: Vec::new(),
            weapon_template_set_finder: SparseMatchFinder::new(),
            armor_template_sets: Vec::new(),
            armor_template_set_finder: SparseMatchFinder::new(),
            locomotor_sets: std::array::from_fn(|_| Vec::new()),
            per_unit_sounds: BTreeMap::new(),
            per_unit_fx: BTreeMap::new(),

            reskinned_from_id: None,

            fence_width: 0.0,
            fence_x_offset: 0.0,
            vision_range: 0.0,
            shroud_clearing_range: -1.0,
            shroud_reveal_to_all_range: -1.0,
            placement_view_angle: 0.0,
            factory_exit_width: 0.0,
            factory_extra_bib_width: 0.0,
            build_time: 1.0,
            asset_scale: 0.0,
            instance_scale_fuzziness: 0.0,
            shadow_size_x: 0.0,
            shadow_size_y: 0.0,
            shadow_offset_x: 0.0,
            shadow_offset_y: 0.0,

            energy_production: 0,
            energy_bonus: 0,
            display_color: 0,
            occlusion_delay: default_occlusion_delay,
            max_simultaneous_link_key: NameKeyType::INVALID,

            template_id: 0,
            build_cost: 0,
            refund_value: 0,
            threat_value: 0,
            max_simultaneous_of_type: 0,

            max_simultaneous_determined_by_superweapon_restriction: FALSE,
            is_prerequisite: FALSE,
            is_bridge: FALSE,
            is_build_facility: FALSE,
            is_trainable: FALSE,
            enter_guard: FALSE,
            hijack_guard: FALSE,
            is_forbidden: FALSE,
            armor_copied_from_default: FALSE,
            weapons_copied_from_default: FALSE,

            radar_priority: RadarPriorityType::Invalid,
            transport_slot_count: 0,
            buildable: BuildableStatus::Yes,
            build_completion: BuildCompletionType::AppearsAtRallyPoint,
            editor_sorting: EditorSortingType::None,
            structure_rubble_height: 0,
            shadow_type: SHADOW_NONE,
            module_parsing_mode: ModuleParseMode::Normal,
            crusher_level: 0,
            crushable_level: 255,
        }
    }

    #[inline]
    pub const fn get_template_id(&self) -> UnsignedShort {
        self.template_id
    }

    #[inline]
    pub fn get_name(&self) -> &str {
        &self.name_string
    }

    #[inline]
    pub fn get_display_name(&self) -> &[u16] {
        &self.display_name
    }

    #[inline]
    pub const fn get_default_radar_priority(&self) -> RadarPriorityType {
        self.radar_priority
    }

    #[inline]
    pub const fn get_raw_transport_slot_count(&self) -> Int {
        self.transport_slot_count as Int
    }

    #[inline]
    pub const fn get_fence_width(&self) -> Real {
        self.fence_width
    }

    #[inline]
    pub const fn get_fence_x_offset(&self) -> Real {
        self.fence_x_offset
    }

    #[inline]
    pub const fn is_bridge(&self) -> Bool {
        self.is_bridge
    }

    #[inline]
    pub fn is_bridge_like(&self) -> Bool {
        if self.is_bridge != FALSE || self.is_kind_of(KindOfType::WalkOnTopOfWall) != FALSE {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn is_kind_of(&self, t: KindOfType) -> Bool {
        self.kindof.test(t.as_i32())
    }

    #[inline]
    pub fn is_kind_of_multi(
        &self,
        must_be_set: &KindOfMaskType,
        must_be_clear: &KindOfMaskType,
    ) -> Bool {
        self.kindof.test_set_and_clear(must_be_set, must_be_clear)
    }

    #[inline]
    pub fn is_any_kind_of(&self, any_kind_of: &KindOfMaskType) -> Bool {
        self.kindof.test_for_any(any_kind_of)
    }

    #[inline]
    pub const fn friend_calc_vision_range(&self) -> Real {
        self.vision_range
    }

    #[inline]
    pub const fn friend_calc_shroud_clearing_range(&self) -> Real {
        self.shroud_clearing_range
    }

    #[inline]
    pub const fn get_shroud_reveal_to_all_range(&self) -> Real {
        self.shroud_reveal_to_all_range
    }

    #[inline]
    pub const fn get_placement_view_angle(&self) -> Real {
        self.placement_view_angle
    }

    #[inline]
    pub const fn get_factory_exit_width(&self) -> Real {
        self.factory_exit_width
    }

    #[inline]
    pub const fn get_factory_extra_bib_width(&self) -> Real {
        self.factory_extra_bib_width
    }

    #[inline]
    pub const fn get_skill_point_value(&self, level: usize) -> Int {
        let value = self.skill_point_values[level];
        if value == USE_EXP_VALUE_FOR_SKILL_VALUE {
            self.experience_values[level]
        } else {
            value
        }
    }

    #[inline]
    pub const fn get_experience_value(&self, level: usize) -> Int {
        self.experience_values[level]
    }

    #[inline]
    pub const fn get_experience_required(&self, level: usize) -> Int {
        self.experience_required[level]
    }

    #[inline]
    pub const fn is_trainable(&self) -> Bool {
        self.is_trainable
    }

    #[inline]
    pub const fn is_enter_guard(&self) -> Bool {
        self.enter_guard
    }

    #[inline]
    pub const fn is_hijack_guard(&self) -> Bool {
        self.hijack_guard
    }

    #[inline]
    pub fn get_default_owning_side(&self) -> &str {
        &self.default_owning_side
    }

    #[inline]
    pub fn get_prereq_count(&self) -> Int {
        self.prereq_info.len() as Int
    }

    #[inline]
    pub fn get_nth_prereq(&self, i: usize) -> Option<&ProductionPrerequisite> {
        self.prereq_info.get(i)
    }

    pub fn get_buildable(&self, overrides: Option<&BTreeMap<String, BuildableStatus>>) -> BuildableStatus {
        if let Some(map) = overrides {
            if let Some(&bs) = map.get(&self.name_string) {
                return bs;
            }
        }
        self.buildable
    }

    #[inline]
    pub const fn get_build_cost(&self) -> UnsignedShort {
        self.build_cost
    }

    #[inline]
    pub const fn get_build_time(&self) -> Real {
        self.build_time
    }

    #[inline]
    pub const fn get_refund_value(&self) -> Int {
        self.refund_value as Int
    }

    #[inline]
    pub const fn get_build_completion(&self) -> BuildCompletionType {
        self.build_completion
    }

    #[inline]
    pub const fn get_energy_production(&self) -> Int {
        self.energy_production
    }

    #[inline]
    pub const fn get_energy_bonus(&self) -> Int {
        self.energy_bonus
    }

    #[inline]
    pub const fn is_prerequisite(&self) -> Bool {
        self.is_prerequisite
    }

    #[inline]
    pub const fn is_build_facility(&self) -> Bool {
        self.is_build_facility
    }

    #[inline]
    pub const fn get_display_color(&self) -> Color {
        self.display_color
    }

    #[inline]
    pub const fn get_editor_sorting(&self) -> EditorSortingType {
        self.editor_sorting
    }

    #[inline]
    pub fn friend_get_command_set_string(&self) -> &str {
        &self.command_set_string
    }

    #[inline]
    pub fn get_build_variations(&self) -> &[String] {
        &self.build_variations
    }

    #[inline]
    pub fn get_behavior_module_info(&self) -> &ModuleInfo {
        &self.behavior_module_info
    }

    #[inline]
    pub fn get_draw_module_info(&self) -> &ModuleInfo {
        &self.draw_module_info
    }

    #[inline]
    pub fn get_client_update_module_info(&self) -> &ModuleInfo {
        &self.client_update_module_info
    }

    #[inline]
    pub fn get_upgrade_cameo_name(&self, n: usize) -> &str {
        &self.upgrade_cameo_upgrade_names[n]
    }

    #[inline]
    pub const fn get_asset_scale(&self) -> Real {
        self.asset_scale
    }

    #[inline]
    pub const fn get_instance_scale_fuzziness(&self) -> Real {
        self.instance_scale_fuzziness
    }

    #[inline]
    pub const fn get_structure_rubble_height(&self) -> Real {
        self.structure_rubble_height as Real
    }

    #[inline]
    pub const fn get_template_geometry_info(&self) -> &GeometryInfo {
        &self.geometry_info
    }

    #[inline]
    pub const fn get_shadow_type(&self) -> Int {
        self.shadow_type
    }

    #[inline]
    pub const fn get_shadow_size_x(&self) -> Real {
        self.shadow_size_x
    }

    #[inline]
    pub const fn get_shadow_size_y(&self) -> Real {
        self.shadow_size_y
    }

    #[inline]
    pub const fn get_shadow_offset_x(&self) -> Real {
        self.shadow_offset_x
    }

    #[inline]
    pub const fn get_shadow_offset_y(&self) -> Real {
        self.shadow_offset_y
    }

    #[inline]
    pub fn get_shadow_texture_name(&self) -> &str {
        &self.shadow_texture_name
    }

    #[inline]
    pub const fn get_occlusion_delay(&self) -> UnsignedInt {
        self.occlusion_delay
    }

    #[inline]
    pub const fn get_threat_value(&self) -> UnsignedInt {
        self.threat_value as UnsignedInt
    }

    #[inline]
    pub const fn get_max_simultaneous_of_type(
        &self,
        superweapon_restriction: UnsignedShort,
    ) -> UnsignedInt {
        if self.max_simultaneous_determined_by_superweapon_restriction != FALSE {
            superweapon_restriction as UnsignedInt
        } else {
            self.max_simultaneous_of_type as UnsignedInt
        }
    }

    #[inline]
    pub const fn get_max_simultaneous_link_key(&self) -> NameKeyType {
        self.max_simultaneous_link_key
    }

    #[inline]
    pub const fn get_crushable_level(&self) -> UnsignedByte {
        self.crushable_level
    }

    #[inline]
    pub const fn get_crusher_level(&self) -> UnsignedByte {
        self.crusher_level
    }

    #[inline]
    pub fn can_possibly_have_any_weapon(&self) -> Bool {
        if self
            .weapon_template_sets
            .iter()
            .any(|ws| ws.has_any_weapons() != FALSE)
        {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub const fn is_buildable_item(&self) -> Bool {
        if self.build_cost != 0 { TRUE } else { FALSE }
    }

    #[inline]
    pub fn get_per_unit_fx(&self, name: &str) -> Option<&str> {
        if name.is_empty() {
            return None;
        }
        self.per_unit_fx.get(name).and_then(|v| v.as_deref())
    }

    #[inline]
    pub fn get_per_unit_sound(&self, name: &str) -> &str {
        if name.is_empty() {
            return "";
        }
        self.per_unit_sounds.get(name).map_or("", String::as_str)
    }

    #[inline]
    pub fn find_armor_template_set(&self, t: &ArmorSetFlags) -> Option<&ArmorTemplateSet> {
        self.armor_template_set_finder
            .find_best_info(&self.armor_template_sets, t)
    }

    #[inline]
    pub fn find_armor_template_set_mut(&mut self, t: &ArmorSetFlags) -> Option<&ArmorTemplateSet> {
        self.armor_template_set_finder
            .find_best_info_mut(&self.armor_template_sets, t)
    }

    #[inline]
    pub fn find_weapon_template_set(&self, t: &WeaponSetFlags) -> Option<&WeaponTemplateSet> {
        self.weapon_template_set_finder
            .find_best_info(&self.weapon_template_sets, t)
    }

    #[inline]
    pub fn find_weapon_template_set_mut(&mut self, t: &WeaponSetFlags) -> Option<&WeaponTemplateSet> {
        self.weapon_template_set_finder
            .find_best_info_mut(&self.weapon_template_sets, t)
    }

    #[inline]
    pub fn get_locomotor_set(&self, set: LocomotorSetType) -> &[String] {
        self.locomotor_sets
            .get(set as usize)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    pub fn set_copied_from_default(&mut self) {
        self.armor_copied_from_default = TRUE;
        self.weapons_copied_from_default = TRUE;
        self.behavior_module_info.set_copied_from_default(true);
        self.draw_module_info.set_copied_from_default(true);
        self.client_update_module_info.set_copied_from_default(true);
    }

    /// Copies all template fields from `other`, preserving `self.template_id` and `self.name_string`.
    pub fn copy_from(&mut self, other: &Self) {
        let id = self.template_id;
        let name = std::mem::take(&mut self.name_string);
        *self = other.clone();
        self.template_id = id;
        self.name_string = name;
    }

    /// Sets default `shadow_texture_name` when empty (`ThingTemplate.cpp:2160-2175`).
    pub fn validate(&mut self) {
        if self.shadow_texture_name.is_empty() {
            match self.geometry_info.get_geom_type() {
                GeometryType::Sphere | GeometryType::Cylinder => {
                    self.shadow_texture_name = "shadow".to_owned();
                }
                GeometryType::Box => {
                    self.shadow_texture_name = "shadows".to_owned();
                }
            }
        }
    }

    /// Resolves prerequisite unit names and marks build facilities (`ThingTemplate.cpp:2272-2314`).
    pub fn resolve_names(
        &mut self,
        registry: &dyn ThingTemplateRegistry,
    ) -> Vec<ThingTemplateRef> {
        let mut facility_refs = Vec::new();
        for prereq in &mut self.prereq_info {
            prereq.resolve_names(registry);
            let mut buf = [None; 32];
            let count = prereq.get_all_possible_build_facility_templates(&mut buf);
            for slot in buf.iter().take(count as usize) {
                if let Some(r) = *slot {
                    facility_refs.push(r);
                }
            }
        }
        if self.is_kind_of(KindOfType::CommandCenter) != FALSE {
            self.is_build_facility = TRUE;
        }
        facility_refs
    }

    /// Returns `Some(prereq[0].get_existing_build_facility_template)` if prereqs exist.
    pub fn get_build_facility_template(
        &self,
        player: Option<&dyn PrereqPlayerView>,
    ) -> Option<ThingTemplateRef> {
        let view = player?;
        self.prereq_info
            .first()
            .and_then(|p| p.get_existing_build_facility_template(view))
    }

    /// Mirrors `ThingTemplate::isEquivalentTo` (`ThingTemplate.cpp:2497-2538`).
    pub fn is_equivalent_to(&self, other: &Self) -> Bool {
        if self.template_id == other.template_id {
            return TRUE;
        }
        if self.reskinned_from_id == Some(other.template_id)
            || other.reskinned_from_id == Some(self.template_id)
        {
            return TRUE;
        }
        if self.reskinned_from_id.is_some() && self.reskinned_from_id == other.reskinned_from_id {
            return TRUE;
        }
        for bv in &self.build_variations {
            if bv.eq_ignore_ascii_case(&other.name_string) {
                return TRUE;
            }
        }
        for bv in &other.build_variations {
            if bv.eq_ignore_ascii_case(&self.name_string) {
                return TRUE;
            }
        }
        FALSE
    }

    /// Calculates production cost in credits (`ThingTemplate.cpp:2551-2560`).
    pub fn calc_cost_to_build(&self, player: &dyn ThingTemplatePlayerView) -> Int {
        let mut faction_modifier: Real =
            1.0_f32 + player.get_production_cost_change_percent(&self.name_string);
        faction_modifier *= player.get_production_cost_change_based_on_kind_of(&self.kindof);
        let hc = player.get_handicap(HandicapType::BuildCost, self);
        (((self.build_cost as Int as Real) * faction_modifier) * hc) as Int
    }

    /// Calculates production duration in logic frames (`ThingTemplate.cpp:2567-2619`).
    pub fn calc_time_to_build(
        &self,
        player: &dyn ThingTemplatePlayerView,
        global_data: &ThingTemplateGlobalTunables,
    ) -> Int {
        let mut build_time: Int = (self.build_time * LOGICFRAMES_PER_SECONDS_REAL) as Int;
        let hc_time = player.get_handicap(HandicapType::BuildTime, self);
        build_time = ((build_time as Real) * hc_time) as Int;

        let faction_modifier: Real =
            1.0_f32 + player.get_production_time_change_percent(&self.name_string);
        build_time = ((build_time as Real) * faction_modifier) as Int;

        let mut energy_percent = player.get_energy_supply_ratio();
        if energy_percent > 1.0_f32 {
            energy_percent = 1.0_f32;
        }
        let mut energy_short = 1.0_f32 - energy_percent;
        energy_short *= global_data.low_energy_penalty_modifier;
        let mut penalty_rate = 1.0_f32 - energy_short;
        if penalty_rate < global_data.min_low_energy_production_speed {
            penalty_rate = global_data.min_low_energy_production_speed;
        }
        if energy_percent < 1.0_f32 && global_data.max_low_energy_production_speed < penalty_rate {
            penalty_rate = global_data.max_low_energy_production_speed;
        }
        if penalty_rate <= 0.0_f32 {
            penalty_rate = 0.01_f32;
        }
        build_time = ((build_time as Real) / penalty_rate) as Int;

        if self.build_completion == BuildCompletionType::AppearsAtRallyPoint {
            if let Some(fac_tmpl) = self.get_build_facility_template(player.as_prereq_view()) {
                let count = player.count_objects_by_template_ref(fac_tmpl);
                let factory_mult = global_data.multiple_factory;
                if factory_mult > 0.0_f32 {
                    let mut i: Int = 0;
                    while i < count - 1 {
                        build_time = ((build_time as Real) * factory_mult) as Int;
                        i += 1;
                    }
                }
            }
        }
        build_time
    }

    /// Parses all `Object` or `ObjectReskin` fields up to `End`.
    pub fn parse_object_fields(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
        reskin_only: bool,
    ) -> Result<(), IniError> {
        ini.read_block_fields(|field, reader| {
            self.parse_single_object_field(field, reader, ctx, reskin_only)
        })
    }

    fn parse_single_object_field(
        &mut self,
        field: &str,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
        reskin_only: bool,
    ) -> Result<(), IniError> {
        // Common subset shared by both s_objectFieldParseTable and s_objectReskinFieldParseTable.
        match field {
            "Draw" => return self.parse_module_name(ini, ModuleType::Draw, false, ctx),
            "Geometry" => {
                let tok = ini.next_token(SEPS_DEFAULT)?;
                let idx = IniReader::scan_index_list(tok, &GEOMETRY_NAMES)?;
                self.geometry_info.set_geom_type(GeometryType::from_i32(idx));
                return Ok(());
            }
            "GeometryMajorRadius" => {
                let v = IniReader::scan_real(ini.next_token(SEPS_DEFAULT)?)?;
                self.geometry_info.set_major_radius(v);
                return Ok(());
            }
            "GeometryMinorRadius" => {
                let v = IniReader::scan_real(ini.next_token(SEPS_DEFAULT)?)?;
                self.geometry_info.set_minor_radius(v);
                return Ok(());
            }
            "GeometryHeight" => {
                let v = IniReader::scan_real(ini.next_token(SEPS_DEFAULT)?)?;
                self.geometry_info.set_height(v);
                return Ok(());
            }
            "GeometryIsSmall" => {
                let v = IniReader::scan_bool(ini.next_token(SEPS_DEFAULT)?)?;
                self.geometry_info.set_is_small(v);
                return Ok(());
            }
            "FenceWidth" => return ini.parse_real(&mut self.fence_width),
            "FenceXOffset" => return ini.parse_real(&mut self.fence_x_offset),
            "MaxSimultaneousOfType" => return self.parse_max_simultaneous(ini),
            "MaxSimultaneousLinkKey" => {
                let tok = ini.next_token(SEPS_DEFAULT)?;
                self.max_simultaneous_link_key = ctx.name_keys.name_to_key(tok);
                return Ok(());
            }
            _ => {}
        }

        if reskin_only {
            return Err(IniError::UnknownToken);
        }

        match field {
            "DisplayName" => {
                let label = ini.next_token(SEPS_DEFAULT)?;
                self.display_name = ctx.game_text.fetch_utf16(label);
                Ok(())
            }
            "RadarPriority" => {
                let tok = ini.next_token(SEPS_DEFAULT)?;
                let idx = IniReader::scan_index_list(tok, &RADAR_PRIORITY_NAMES)?;
                self.radar_priority =
                    RadarPriorityType::from_i32(idx).ok_or(IniError::OutOfRange)?;
                Ok(())
            }
            "TransportSlotCount" => ini.parse_unsigned_byte(&mut self.transport_slot_count),
            "IsBridge" => ini.parse_bool(&mut self.is_bridge),
            "ArmorSet" => self.parse_armor_template_set(ini, ctx),
            "WeaponSet" => self.parse_weapon_template_set(ini, ctx),
            "VisionRange" => ini.parse_real(&mut self.vision_range),
            "ShroudClearingRange" => ini.parse_real(&mut self.shroud_clearing_range),
            "ShroudRevealToAllRange" => ini.parse_real(&mut self.shroud_reveal_to_all_range),
            "PlacementViewAngle" => ini.parse_angle_real(&mut self.placement_view_angle),
            "FactoryExitWidth" => ini.parse_real(&mut self.factory_exit_width),
            "FactoryExtraBibWidth" => ini.parse_real(&mut self.factory_extra_bib_width),
            "SkillPointValue" => Self::parse_int_list(ini, &mut self.skill_point_values),
            "ExperienceValue" => Self::parse_int_list(ini, &mut self.experience_values),
            "ExperienceRequired" => Self::parse_int_list(ini, &mut self.experience_required),
            "IsTrainable" => ini.parse_bool(&mut self.is_trainable),
            "EnterGuard" => ini.parse_bool(&mut self.enter_guard),
            "HijackGuard" => ini.parse_bool(&mut self.hijack_guard),
            "Side" => ini.parse_ascii_string(&mut self.default_owning_side),
            "Prerequisites" => self.parse_prerequisites(ini, ctx),
            "Buildable" => {
                let tok = ini.next_token(SEPS_DEFAULT)?;
                let idx = IniReader::scan_index_list(tok, &BUILDABLE_STATUS_NAMES)?;
                self.buildable = BuildableStatus::from_i32(idx).ok_or(IniError::OutOfRange)?;
                Ok(())
            }
            "BuildCost" => ini.parse_unsigned_short(&mut self.build_cost),
            "BuildTime" => ini.parse_real(&mut self.build_time),
            "RefundValue" => ini.parse_unsigned_short(&mut self.refund_value),
            "BuildCompletion" => {
                let tok = ini.next_token(SEPS_DEFAULT)?;
                let idx = IniReader::scan_index_list(tok, &BUILD_COMPLETION_NAMES)?;
                self.build_completion =
                    BuildCompletionType::from_i32(idx).ok_or(IniError::OutOfRange)?;
                Ok(())
            }
            "EnergyProduction" => ini.parse_int(&mut self.energy_production),
            "EnergyBonus" => ini.parse_int(&mut self.energy_bonus),
            "IsForbidden" => ini.parse_bool(&mut self.is_forbidden),
            "IsPrerequisite" => ini.parse_bool(&mut self.is_prerequisite),
            "DisplayColor" => ini.parse_color_int(&mut self.display_color),
            "EditorSorting" => {
                let tok = ini.next_token(SEPS_DEFAULT)?;
                self.editor_sorting = parse_editor_sorting_type(tok)?;
                Ok(())
            }
            "KindOf" => self.kindof.parse_from_ini(ini, &KINDOF_NAMES),
            "CommandSet" => ini.parse_ascii_string(&mut self.command_set_string),
            "BuildVariations" => ini.parse_ascii_string_vector(&mut self.build_variations),
            "Behavior" => self.parse_module_name(ini, ModuleType::Behavior, false, ctx),
            "Body" => self.parse_module_name(ini, ModuleType::Behavior, true, ctx),
            "ClientUpdate" => self.parse_module_name(ini, ModuleType::ClientUpdate, false, ctx),
            "SelectPortrait" => ini.parse_ascii_string(&mut self.selected_portrait_image_name),
            "ButtonImage" => ini.parse_ascii_string(&mut self.button_image_name),
            "UpgradeCameo1" => ini.parse_ascii_string(&mut self.upgrade_cameo_upgrade_names[0]),
            "UpgradeCameo2" => ini.parse_ascii_string(&mut self.upgrade_cameo_upgrade_names[1]),
            "UpgradeCameo3" => ini.parse_ascii_string(&mut self.upgrade_cameo_upgrade_names[2]),
            "UpgradeCameo4" => ini.parse_ascii_string(&mut self.upgrade_cameo_upgrade_names[3]),
            "UpgradeCameo5" => ini.parse_ascii_string(&mut self.upgrade_cameo_upgrade_names[4]),
            "UnitSpecificSounds" => self.parse_per_unit_sounds(ini),
            "UnitSpecificFX" => self.parse_per_unit_fx(ini, ctx),
            "Scale" => ini.parse_real(&mut self.asset_scale),
            "Shadow" => {
                let mut b: Byte = self.shadow_type as Byte;
                ini.parse_bit_string8(&mut b, &THE_SHADOW_NAMES)?;
                self.shadow_type = (b as UnsignedByte) as Int;
                Ok(())
            }
            "ShadowSizeX" => ini.parse_real(&mut self.shadow_size_x),
            "ShadowSizeY" => ini.parse_real(&mut self.shadow_size_y),
            "ShadowOffsetX" => ini.parse_real(&mut self.shadow_offset_x),
            "ShadowOffsetY" => ini.parse_real(&mut self.shadow_offset_y),
            "ShadowTexture" => ini.parse_ascii_string(&mut self.shadow_texture_name),
            "OcclusionDelay" => ini.parse_duration_unsigned_int(&mut self.occlusion_delay),
            "AddModule" => self.parse_add_module(ini, ctx),
            "RemoveModule" => self.parse_remove_module(ini),
            "ReplaceModule" => self.parse_replace_module(ini, ctx),
            "InheritableModule" => self.parse_inheritable_module(ini, ctx),
            "OverrideableByLikeKind" => self.parse_overrideable_by_like_kind(ini, ctx),
            "Locomotor" => self.parse_locomotor_set(ini, ctx),
            "InstanceScaleFuzziness" => ini.parse_real(&mut self.instance_scale_fuzziness),
            "StructureRubbleHeight" => ini.parse_unsigned_byte(&mut self.structure_rubble_height),
            "ThreatValue" => ini.parse_unsigned_short(&mut self.threat_value),
            "CrusherLevel" => ini.parse_unsigned_byte(&mut self.crusher_level),
            "CrushableLevel" => ini.parse_unsigned_byte(&mut self.crushable_level),
            _ if field.starts_with("Voice") || field.starts_with("Sound") => {
                let val = ini.next_ascii_string()?;
                self.audio_events.insert(field.to_owned(), val);
                Ok(())
            }
            _ => Err(IniError::UnknownToken),
        }
    }

    fn parse_int_list(ini: &mut IniReader<'_>, list: &mut [Int; LEVEL_COUNT]) -> Result<(), IniError> {
        for slot in list.iter_mut() {
            *slot = IniReader::scan_int(ini.next_token(SEPS_DEFAULT)?)?;
        }
        Ok(())
    }

    fn parse_armor_template_set(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        if self.armor_copied_from_default == TRUE {
            self.armor_copied_from_default = FALSE;
            self.armor_template_sets.clear();
        }
        let mut ats = ArmorTemplateSet::new();
        ats.parse_armor_template_set(
            ini,
            ctx.armor_store,
            ctx.damage_fx_names,
            ctx.name_keys,
        )?;
        self.armor_template_sets.push(ats);
        self.armor_template_set_finder.clear();
        Ok(())
    }

    fn parse_weapon_template_set(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        if self.weapons_copied_from_default == TRUE {
            self.weapons_copied_from_default = FALSE;
            self.weapon_template_sets.clear();
        }
        let mut wts = WeaponTemplateSet::new();
        wts.parse_weapon_template_set_ini(ini, ctx.weapon_store, &self.name_string)?;
        self.weapon_template_sets.push(wts);
        self.weapon_template_set_finder.clear();
        Ok(())
    }

    fn parse_prerequisites(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        if ini.get_load_type() == IniLoadType::CreateOverrides {
            self.prereq_info.clear();
        }
        ini.read_block_fields(|field, reader| {
            if field.eq_ignore_ascii_case("Object") {
                let mut prereq = ProductionPrerequisite::new();
                let first = reader.next_token(SEPS_DEFAULT)?;
                prereq.add_unit_prereq(first, FALSE);
                while let Some(tok) = reader.next_token_or_null(SEPS_DEFAULT) {
                    prereq.add_unit_prereq(tok, TRUE);
                }
                self.prereq_info.push(prereq);
                Ok(())
            } else if field.eq_ignore_ascii_case("Science") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                let st = if tok.eq_ignore_ascii_case("None") {
                    SCIENCE_INVALID
                } else {
                    ctx.science_store.friend_lookup_science(tok, ctx.name_keys)?
                };
                let mut prereq = ProductionPrerequisite::new();
                prereq.add_science_prereq(st);
                self.prereq_info.push(prereq);
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }

    fn parse_per_unit_fx(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        self.per_unit_fx.clear();
        ini.read_block_fields(|field, reader| {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            let fx = if tok.eq_ignore_ascii_case("None") {
                None
            } else {
                ctx.fx_list_names
                    .iter()
                    .find(|n| n.eq_ignore_ascii_case(tok))
                    .cloned()
            };
            self.per_unit_fx.insert(field.to_owned(), fx);
            Ok(())
        })
    }

    fn parse_per_unit_sounds(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        self.per_unit_sounds.clear();
        ini.read_block_fields(|field, reader| {
            let tok = reader.next_token_or_null(SEPS_DEFAULT).unwrap_or("");
            self.per_unit_sounds.insert(field.to_owned(), tok.to_owned());
            Ok(())
        })
    }

    fn parse_locomotor_set(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let set_tok = ini.next_token(SEPS_DEFAULT)?;
        let set_idx = IniReader::scan_index_list(set_tok, &LOCOMOTOR_SET_NAMES)? as usize;
        if !self.locomotor_sets[set_idx].is_empty()
            && ini.get_load_type() != IniLoadType::CreateOverrides
        {
            return Err(IniError::InvalidData);
        }
        self.locomotor_sets[set_idx].clear();

        let first = ini.next_token(SEPS_DEFAULT)?;
        if !first.is_empty() && !first.eq_ignore_ascii_case("None") {
            if let Some(loco_store) = ctx.locomotor_store {
                if loco_store.find_by_name(first).is_none() {
                    return Err(IniError::InvalidData);
                }
            }
            self.locomotor_sets[set_idx].push(first.to_owned());
        }
        while let Some(tok) = ini.next_token_or_null(SEPS_DEFAULT) {
            if tok.is_empty() || tok.eq_ignore_ascii_case("None") {
                continue;
            }
            if let Some(loco_store) = ctx.locomotor_store {
                if loco_store.find_by_name(tok).is_none() {
                    return Err(IniError::InvalidData);
                }
            }
            self.locomotor_sets[set_idx].push(tok.to_owned());
        }
        Ok(())
    }

    fn parse_module_name(
        &mut self,
        ini: &mut IniReader<'_>,
        mod_type: ModuleType,
        is_body: bool,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let mod_name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let module_tag = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let interface_mask = if let Some(reg) = ctx.module_registry {
            reg.find_module_interface_mask(&mod_name, mod_type)
        } else {
            ctx.module_masks
                .find_module_interface_mask(&mod_name, mod_type)
        };
        if is_body && (interface_mask & MODULEINTERFACE_BODY) == 0 {
            return Err(IniError::InvalidData);
        }
        if !is_body && (interface_mask & MODULEINTERFACE_BODY) != 0 {
            return Err(IniError::InvalidData);
        }

        if ini.get_load_type() == IniLoadType::CreateOverrides {
            if self.module_parsing_mode != ModuleParseMode::AddRemoveReplace {
                return Err(IniError::InvalidData);
            }
        } else {
            self.behavior_module_info.clear_copied_from_default_entries(
                interface_mask,
                &mod_name,
                &self.kindof,
                self.is_trainable,
            );
            self.draw_module_info.clear_copied_from_default_entries(
                interface_mask,
                &mod_name,
                &self.kindof,
                self.is_trainable,
            );
            self.client_update_module_info
                .clear_copied_from_default_entries(
                    interface_mask,
                    &mod_name,
                    &self.kindof,
                    self.is_trainable,
                );
        }

        if self.module_parsing_mode == ModuleParseMode::AddRemoveReplace {
            if !self.module_being_replaced_name.is_empty()
                && self.module_being_replaced_name != mod_name
            {
                return Err(IniError::InvalidData);
            }
            if !self.module_being_replaced_tag.is_empty()
                && self.module_being_replaced_tag == module_tag
            {
                return Err(IniError::InvalidData);
            }
        }

        let tag_key = ctx.name_keys.name_to_key(&module_tag);
        let mod_name_key = ctx.name_keys.name_to_key(&mod_name);
        let data_id = match (ctx.module_registry, ctx.module_data_store.as_deref_mut()) {
            (Some(reg), Some(store)) => {
                let empty_upgrades = UpgradeCenter::new();
                let empty_names = BTreeSet::new();
                let mut parse_ctx = ModuleParseContext {
                    name_keys: ctx.name_keys,
                    science_store: ctx.science_store,
                    upgrade_center: ctx.upgrade_center.unwrap_or(&empty_upgrades),
                    armor_store: ctx.armor_store,
                    weapon_store: ctx.weapon_store,
                    fx_list_names: ctx.fx_list_names,
                    damage_fx_names: ctx.damage_fx_names,
                    ocl_names: ctx.ocl_names.unwrap_or(&empty_names),
                    particle_system_names: ctx.particle_system_names.unwrap_or(&empty_names),
                };
                reg.parse_module_data(
                    ini,
                    &mod_name,
                    mod_type,
                    tag_key,
                    mod_name_key,
                    store,
                    ThingTemplateId(self.template_id as u32),
                    &mut parse_ctx,
                )?
            }
            _ => {
                ini.skip_block()?;
                ModuleDataId::INVALID
            }
        };

        let is_ai = if let Some(reg) = ctx.module_registry {
            reg.is_ai_module_name(&mod_name)
        } else {
            ctx.module_masks.is_ai_module_name(&mod_name)
        };
        let target = match mod_type {
            ModuleType::Behavior => &mut self.behavior_module_info,
            ModuleType::Draw => &mut self.draw_module_info,
            ModuleType::ClientUpdate => &mut self.client_update_module_info,
        };
        if is_ai {
            let _ = target.clear_ai_module_info();
        }
        let inheritable = self.module_parsing_mode == ModuleParseMode::Inheritable;
        let oblk = self.module_parsing_mode == ModuleParseMode::OverrideableByLikeKind;
        target.add_module_info_with_data(
            mod_name,
            module_tag,
            tag_key,
            data_id,
            interface_mask,
            is_ai,
            inheritable,
            oblk,
        );
        Ok(())
    }

    fn parse_add_module(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let old = self.module_parsing_mode;
        if old != ModuleParseMode::Normal {
            return Err(IniError::InvalidData);
        }
        self.module_parsing_mode = ModuleParseMode::AddRemoveReplace;
        let res = self.parse_object_fields(ini, ctx, false);
        self.module_parsing_mode = old;
        res
    }

    fn parse_remove_module(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let old = self.module_parsing_mode;
        if old != ModuleParseMode::Normal {
            return Err(IniError::InvalidData);
        }
        self.module_parsing_mode = ModuleParseMode::AddRemoveReplace;
        let mod_to_remove = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let removed = self.remove_module_info(&mod_to_remove).is_some();
        self.module_parsing_mode = old;
        if removed {
            Ok(())
        } else {
            Err(IniError::InvalidData)
        }
    }

    fn parse_replace_module(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let old = self.module_parsing_mode;
        if old != ModuleParseMode::Normal {
            return Err(IniError::InvalidData);
        }
        self.module_parsing_mode = ModuleParseMode::AddRemoveReplace;
        let mod_to_remove = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let Some(removed_name) = self.remove_module_info(&mod_to_remove) else {
            self.module_parsing_mode = old;
            return Err(IniError::InvalidData);
        };
        self.module_being_replaced_name = removed_name;
        self.module_being_replaced_tag = mod_to_remove;
        let res = self.parse_object_fields(ini, ctx, false);
        self.module_being_replaced_name.clear();
        self.module_being_replaced_tag.clear();
        self.module_parsing_mode = old;
        res
    }

    fn parse_inheritable_module(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let old = self.module_parsing_mode;
        if old != ModuleParseMode::Normal {
            return Err(IniError::InvalidData);
        }
        self.module_parsing_mode = ModuleParseMode::Inheritable;
        let res = self.parse_object_fields(ini, ctx, false);
        self.module_parsing_mode = old;
        res
    }

    fn parse_overrideable_by_like_kind(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let old = self.module_parsing_mode;
        if old != ModuleParseMode::Normal {
            return Err(IniError::InvalidData);
        }
        self.module_parsing_mode = ModuleParseMode::OverrideableByLikeKind;
        let res = self.parse_object_fields(ini, ctx, false);
        self.module_parsing_mode = old;
        res
    }

    fn remove_module_info(&mut self, tag: &str) -> Option<String> {
        self.behavior_module_info
            .clear_module_data_with_tag(tag)
            .or_else(|| self.draw_module_info.clear_module_data_with_tag(tag))
            .or_else(|| {
                self.client_update_module_info
                    .clear_module_data_with_tag(tag)
            })
    }

    fn parse_max_simultaneous(&mut self, ini: &mut IniReader<'_>) -> Result<(), IniError> {
        let token = ini.next_token(SEPS_DEFAULT)?;
        if token.eq_ignore_ascii_case("DeterminedBySuperweaponRestriction") {
            self.max_simultaneous_determined_by_superweapon_restriction = TRUE;
            self.max_simultaneous_of_type = 0;
            Ok(())
        } else {
            let value = IniReader::scan_int(token)?;
            if !(0..=65535).contains(&value) {
                return Err(IniError::OutOfRange);
            }
            self.max_simultaneous_of_type = value as UnsignedShort;
            self.max_simultaneous_determined_by_superweapon_restriction = FALSE;
            Ok(())
        }
    }
}
