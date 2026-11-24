//! `WP-5.0` / `WP-5.1`: Core simulation spine (`SimContext`, `RulesRegistry`, `SimWorld`,
//! `ObjectRecord`, `SimView`, `SimApi`, `ClientSink`, and `MockSimApi`).
//!
//! Enforces strict three-layer separation (`docs/ARCHITECTURE_SPEC.md` §1.2–§1.5):
//! 1. [`RulesRegistry`] (immutable during a match; reset truncates map overrides)
//! 2. [`SimWorld`] (all mutable state hashed by `GameLogic::getCRC`)
//! 3. [`ClientSink`] (write-only visual/audio notifications, no-op in headless simulation)

use std::collections::{BTreeMap, BTreeSet};

use crate::arena::{
    DeferredSelfOp, FacetMask, ModuleArena, ModuleContext, ModuleDataId, ModuleMeta,
    ModuleSidecar, ModuleSlot, ObjectArena, ObjectSlot, TakeError, ThingTemplateId,
};
use crate::combat::armor::{Armor, ArmorStore};
use crate::combat::damage::DamageInfo;
use crate::combat::damage_fx::DamageFXStore;
use crate::combat::locomotor::LocomotorStore;
use crate::combat::locomotor_set::LocomotorSetType;
use crate::gamelogic::global_data::{BodyDamageType, GlobalData, WeaponBonusConditionType};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::{
    BehaviorModule, MaxHealthChangeType, ModuleData, ModuleDataStore, ModuleRegistry, ModuleType,
};
use crate::object::experience_tracker::{ExperienceParentAccess, ExperienceTrackerState};
use crate::object::thing::{make_align_to_normal_matrix, Thing, ThingTerrainAccess};
use crate::object::thing_factory::ThingFactory;
use crate::object::thing_template::{
    DefaultModuleMaskLookup, GameTextLookup, ThingTemplate, ThingTemplateParseContext,
};
use crate::object::weapon::WeaponStore;
use crate::object::weapon_set::{
    WeaponLockType, WeaponSet, WeaponSetConditionType, WeaponSlotType, WEAPONSLOT_COUNT,
};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{
    is_subdual_damage, ArmorSetFlags, ArmorSetType, DamageType, WeaponSetFlags, WeaponSetType,
    ARMORSET_COUNT,
};
use crate::rules::disabled_types::{DisabledMaskType, DisabledType, DISABLED_COUNT};
use crate::rules::kind_of::{KindOfMaskType, KindOfType};
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::rules::science::{ScienceStore, ScienceType};
use crate::rules::special_power::SpecialPowerStore;
use crate::rules::special_power_types::SpecialPowerType;
use crate::rules::upgrade::{UpgradeCenter, UpgradeMaskType, UpgradeType};
use crate::scheduler::{
    should_process_disabled_update, ScheduleStore, SleepyUpdateHeap, SleepyUpdatePhase,
};
use crate::spatial::crate_system::CrateSystem;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::div_safe_f32;
use zh_common::math::geometry::GeometryInfo;
use zh_common::math::Coord3D;
use zh_common::random_value::PrngState;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime,
    VeterancyLevel, FALSE, TRUE,
};

/// `Object::ObjectPrivateStatusBits` bit flags (`Object.h:694-700`).
pub const PRIVATE_STATUS_EFFECTIVELY_DEAD: u8 = 1 << 0;
pub const PRIVATE_STATUS_UNDETECTED_DEFECTOR: u8 = 1 << 1;
pub const PRIVATE_STATUS_CAPTURED: u8 = 1 << 2;
pub const PRIVATE_STATUS_OFF_MAP: u8 = 1 << 3;

/// Per-weapon-slot clearing mask matching `s_allWeaponFireFlags[]` (`Object.cpp:125-149`).
pub fn all_weapon_fire_flags(slot: WeaponSlotType) -> ModelConditionFlags {
    let mut flags = ModelConditionFlags::new();
    match slot {
        WeaponSlotType::Primary => {
            flags.set(ModelConditionFlagType::FiringA.as_i32(), 1);
            flags.set(ModelConditionFlagType::BetweenFiringShotsA.as_i32(), 1);
            flags.set(ModelConditionFlagType::ReloadingA.as_i32(), 1);
            flags.set(ModelConditionFlagType::PreAttackA.as_i32(), 1);
            flags.set(ModelConditionFlagType::UsingWeaponA.as_i32(), 1);
        }
        WeaponSlotType::Secondary => {
            flags.set(ModelConditionFlagType::FiringB.as_i32(), 1);
            flags.set(ModelConditionFlagType::BetweenFiringShotsB.as_i32(), 1);
            flags.set(ModelConditionFlagType::ReloadingB.as_i32(), 1);
            flags.set(ModelConditionFlagType::PreAttackB.as_i32(), 1);
            flags.set(ModelConditionFlagType::UsingWeaponB.as_i32(), 1);
        }
        WeaponSlotType::Tertiary => {
            flags.set(ModelConditionFlagType::FiringC.as_i32(), 1);
            flags.set(ModelConditionFlagType::BetweenFiringShotsC.as_i32(), 1);
            flags.set(ModelConditionFlagType::ReloadingC.as_i32(), 1);
            flags.set(ModelConditionFlagType::PreAttackC.as_i32(), 1);
            flags.set(ModelConditionFlagType::UsingWeaponC.as_i32(), 1);
        }
    }
    flags
}

/// Decodes an 8-bit [`ArmorSetFlags`] bitmask stored inside [`ModuleSidecar::Body`].
#[inline]
pub fn armor_set_flags_from_u8(bits: u8) -> ArmorSetFlags {
    let mut flags = ArmorSetFlags::new();
    let mut i = 0;
    while i < ARMORSET_COUNT {
        if ((bits >> i) & 1) != 0 {
            flags.set(i as Int, 1);
        }
        i += 1;
    }
    flags
}

/// Encodes an [`ArmorSetFlags`] bitmask into an 8-bit integer for [`ModuleSidecar::Body`].
#[inline]
pub fn armor_set_flags_to_u8(flags: &ArmorSetFlags) -> u8 {
    let mut bits = 0u8;
    let mut i = 0;
    while i < ARMORSET_COUNT {
        if flags.test(i as Int) != FALSE {
            bits |= 1u8 << i;
        }
        i += 1;
    }
    bits
}

/// Headless `GameTextLookup` returning the label ASCII characters for `ThingFactory` INI parsing.
#[derive(Debug, Default)]
pub struct NoopGameText;

impl GameTextLookup for NoopGameText {
    fn fetch_utf16(&mut self, label: &str) -> Vec<u16> {
        label.bytes().map(u16::from).collect()
    }
}

/// Write-only client/audio notification sink (`GameClient` boundary in `SimContext`).
pub trait ClientSink: Send {
    fn on_object_created(&mut self, _id: ObjectId, _template_id: ThingTemplateId) {}
    fn on_object_destroyed(&mut self, _id: ObjectId) {}
    fn on_damage(&mut self, _target: ObjectId, _amount: Real) {}
    fn on_veterancy_changed(&mut self, _target: ObjectId, _old_level: Int, _new_level: Int) {}
    fn on_body_damage_state_change(
        &mut self,
        _target: ObjectId,
        _damage_state: BodyDamageType,
    ) {
    }
}

/// Headless no-op [`ClientSink`] used for Gate V2 tests and headless simulation.
#[derive(Debug, Default)]
pub struct NoopClientSink;

impl ClientSink for NoopClientSink {}

/// Flat zero-altitude terrain stub implementing [`ThingTerrainAccess`] for unit/headless tests.
#[derive(Debug, Default)]
pub struct FlatTerrain;

impl ThingTerrainAccess for FlatTerrain {
    fn get_ground_height(&self, _x: Real, _y: Real, normal: Option<&mut Coord3D>) -> Real {
        if let Some(n) = normal {
            *n = Coord3D::new(0.0, 0.0, 1.0);
        }
        0.0
    }

    fn align_on_terrain(
        &self,
        angle: Real,
        pos: &Coord3D,
        _stick_to_ground: Bool,
        mtx: &mut zh_common::math::Matrix3D,
    ) -> PathfindLayerEnum {
        make_align_to_normal_matrix(angle, pos, &Coord3D::new(0.0, 0.0, 1.0), mtx);
        PathfindLayerEnum::Ground
    }

    fn is_underwater(
        &self,
        _x: Real,
        _y: Real,
        water_z: Option<&mut Real>,
        terrain_z: Option<&mut Real>,
    ) -> Bool {
        if let Some(wz) = water_z {
            *wz = 0.0;
        }
        if let Some(tz) = terrain_z {
            *tz = 0.0;
        }
        FALSE
    }
}

/// Immutable INI-loaded simulation rules and module factories.
pub struct RulesRegistry {
    pub name_keys: NameKeyGenerator,
    pub thing_factory: ThingFactory,
    pub module_registry: ModuleRegistry,
    pub module_data_store: ModuleDataStore,
    pub armor_store: ArmorStore,
    pub weapon_store: WeaponStore,
    pub locomotor_store: LocomotorStore,
    pub science_store: ScienceStore,
    pub upgrade_center: UpgradeCenter,
    pub damage_fx_store: DamageFXStore,
    pub special_power_store: SpecialPowerStore,
    pub crate_system: CrateSystem,
    pub global_data: GlobalData,
    pub fx_list_names: BTreeSet<String>,
    pub damage_fx_names: BTreeSet<String>,
    pub ocl_names: BTreeSet<String>,
    pub particle_system_names: BTreeSet<String>,
}

impl Default for RulesRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl RulesRegistry {
    pub fn new() -> Self {
        let mut name_keys = NameKeyGenerator::new();
        name_keys.init();
        let mut module_registry = ModuleRegistry::with_builtin_zh_modules(&mut name_keys);
        // Preserve C++ 1..=198 decorated key interning order by running category registrars
        // strictly after `with_builtin_zh_modules`.
        crate::modules::body::register_body_modules(&mut module_registry, &mut name_keys);
        crate::modules::create::register_create_modules(&mut module_registry, &mut name_keys);
        crate::modules::damage::register_damage_modules(&mut module_registry, &mut name_keys);
        crate::modules::destroy::register_destroy_modules(&mut module_registry, &mut name_keys);
        crate::modules::die::register_die_modules(&mut module_registry, &mut name_keys);
        crate::modules::upgrade::register_upgrade_modules(&mut module_registry, &mut name_keys);
        crate::modules::special_power::register_special_power_modules(
            &mut module_registry,
            &mut name_keys,
        );
        crate::modules::collide::register_collide_modules(&mut module_registry, &mut name_keys);

        let mut science_store = ScienceStore::new();
        science_store.init();
        let mut upgrade_center = UpgradeCenter::new();
        upgrade_center.init(&mut name_keys);
        let mut crate_system = CrateSystem::new();
        crate_system.init();
        Self {
            name_keys,
            thing_factory: ThingFactory::new(),
            module_registry,
            module_data_store: ModuleDataStore::new(),
            armor_store: ArmorStore::new(),
            weapon_store: WeaponStore::new(),
            locomotor_store: LocomotorStore::new(),
            science_store,
            upgrade_center,
            damage_fx_store: DamageFXStore::new(),
            special_power_store: SpecialPowerStore::new(),
            crate_system,
            global_data: GlobalData::new(),
            fx_list_names: BTreeSet::new(),
            damage_fx_names: BTreeSet::new(),
            ocl_names: BTreeSet::new(),
            particle_system_names: BTreeSet::new(),
        }
    }

    /// Parses `Object ... End` INI definitions into [`Self::thing_factory`] and
    /// [`Self::module_data_store`] with all registered module parsers active.
    pub fn load_thing_templates_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
    ) -> Result<UnsignedInt, IniError> {
        let default_masks = DefaultModuleMaskLookup;
        let mut game_text = NoopGameText;
        let mut ctx = ThingTemplateParseContext {
            name_keys: &mut self.name_keys,
            armor_store: &self.armor_store,
            weapon_store: &self.weapon_store,
            science_store: &self.science_store,
            fx_list_names: &self.fx_list_names,
            damage_fx_names: &self.damage_fx_names,
            game_text: &mut game_text,
            module_masks: &default_masks,
            default_occlusion_delay: self.global_data.default_occlusion_delay,
            locomotor_store: Some(&self.locomotor_store),
            module_registry: Some(&self.module_registry),
            module_data_store: Some(&mut self.module_data_store),
            upgrade_center: Some(&self.upgrade_center),
            ocl_names: Some(&self.ocl_names),
            particle_system_names: Some(&self.particle_system_names),
            special_power_store: Some(&self.special_power_store),
            crate_system: Some(&self.crate_system),
        };
        self.thing_factory.load_from_ini(ini, &mut ctx)
    }

    /// Snapshots base INI boundary before loading map-local overrides.
    pub fn mark_base_loaded(&mut self) {
        self.module_data_store.mark_base_loaded();
    }

    /// Cleans up map-local overrides between matches (`ThingFactory::reset` + `ModuleDataStore::reset`).
    pub fn reset(&mut self) {
        self.thing_factory.reset();
        self.module_data_store.reset();
        self.crate_system.reset();
    }

    /// Constructs a standalone [`ModuleParseContext`] for direct module INI parser invocation.
    pub fn module_parse_context(&mut self) -> ModuleParseContext<'_> {
        ModuleParseContext {
            name_keys: &mut self.name_keys,
            science_store: &self.science_store,
            upgrade_center: &self.upgrade_center,
            armor_store: &self.armor_store,
            weapon_store: &self.weapon_store,
            fx_list_names: &self.fx_list_names,
            damage_fx_names: &self.damage_fx_names,
            ocl_names: &self.ocl_names,
            particle_system_names: &self.particle_system_names,
            special_power_store: &self.special_power_store,
            crate_system: &self.crate_system,
        }
    }
}

/// Slot-stored object state (`Object` in `ObjectArena<ObjectRecord>`), embedding [`Thing`].
#[derive(Debug, Clone)]
pub struct ObjectRecord {
    pub id: ObjectId,
    pub thing: Thing,
    pub team_id: TeamId,
    pub status: ObjectStatusMaskType,
    pub private_status: u8,
    pub geometry: GeometryInfo,
    pub disabled_mask: DisabledMaskType,
    pub disabled_till_frame: [UnsignedInt; DISABLED_COUNT],
    pub upgrades_completed: UpgradeMaskType,
    pub model_condition: ModelConditionFlags,
    pub special_model_condition_until: UnsignedInt,
    pub weapon_bonus_condition: UnsignedInt,
    pub experience_tracker: ExperienceTrackerState,
    pub weapon_set: WeaponSet,
    pub armor_set_flags: ArmorSetFlags,
    pub layer: PathfindLayerEnum,
    pub contained_by: Option<ObjectSlot>,
    pub producer_id: ObjectId,
    pub builder_id: ObjectId,
    pub sole_healing_benefactor_id: ObjectId,
    pub sole_healing_benefactor_expiration_frame: UnsignedInt,
    pub last_weapon_condition: [WeaponSetConditionType; WEAPONSLOT_COUNT],
    pub behaviors: Vec<(ModuleSlot, FacetMask)>,
    pub body_slot: Option<ModuleSlot>,
    pub contain_slot: Option<ModuleSlot>,
    pub stealth_slot: Option<ModuleSlot>,
    pub ai_slot: Option<ModuleSlot>,
    pub physics_slot: Option<ModuleSlot>,
    pub smc_helper_slot: Option<ModuleSlot>,
    pub status_damage_helper_slot: Option<ModuleSlot>,
    pub subdual_damage_helper_slot: Option<ModuleSlot>,
    pub repulsor_helper_slot: Option<ModuleSlot>,
    pub defection_helper_slot: Option<ModuleSlot>,
    pub weapon_status_helper_slot: Option<ModuleSlot>,
    pub firing_tracker_slot: Option<ModuleSlot>,
    pub temp_weapon_bonus_helper_slot: Option<ModuleSlot>,
    pub command_set_override: String,
    pub weapon_set_flags: WeaponSetFlags,
    pub cur_locomotor_set: LocomotorSetType,
    pub shroud_clearing_range_override: Real,
    pub passengers_allowed_to_fire: Bool,
    pub sub_objects_shown: BTreeSet<String>,
    pub sub_objects_hidden: BTreeSet<String>,
    pub ai_goal_object: Option<ObjectSlot>,
}

impl ObjectRecord {
    pub fn new(
        id: ObjectId,
        template_id: ThingTemplateId,
        team_id: TeamId,
        geometry: GeometryInfo,
    ) -> Self {
        Self {
            id,
            thing: Thing::new(template_id),
            team_id,
            status: ObjectStatusMaskType::new(),
            private_status: 0,
            geometry,
            disabled_mask: DisabledMaskType::new(),
            disabled_till_frame: [0; DISABLED_COUNT],
            upgrades_completed: UpgradeMaskType::new(),
            model_condition: ModelConditionFlags::new(),
            special_model_condition_until: 0,
            weapon_bonus_condition: 0,
            experience_tracker: ExperienceTrackerState::default(),
            weapon_set: WeaponSet::new(),
            armor_set_flags: ArmorSetFlags::new(),
            layer: PathfindLayerEnum::Ground,
            contained_by: None,
            producer_id: ObjectId::INVALID,
            builder_id: ObjectId::INVALID,
            sole_healing_benefactor_id: ObjectId::INVALID,
            sole_healing_benefactor_expiration_frame: 0,
            last_weapon_condition: [WeaponSetConditionType::None; WEAPONSLOT_COUNT],
            behaviors: Vec::new(),
            body_slot: None,
            contain_slot: None,
            stealth_slot: None,
            ai_slot: None,
            physics_slot: None,
            smc_helper_slot: None,
            status_damage_helper_slot: None,
            subdual_damage_helper_slot: None,
            repulsor_helper_slot: None,
            defection_helper_slot: None,
            weapon_status_helper_slot: None,
            firing_tracker_slot: None,
            temp_weapon_bonus_helper_slot: None,
            command_set_override: String::new(),
            weapon_set_flags: WeaponSetFlags::new(),
            cur_locomotor_set: LocomotorSetType::Normal,
            shroud_clearing_range_override: -1.0,
            passengers_allowed_to_fire: FALSE,
            sub_objects_shown: BTreeSet::new(),
            sub_objects_hidden: BTreeSet::new(),
            ai_goal_object: None,
        }
    }

    #[inline]
    pub fn is_destroyed(&self) -> bool {
        self.status.test(ObjectStatusType::Destroyed.as_i32()) != FALSE
    }

    #[inline]
    pub fn is_effectively_dead(&self) -> Bool {
        if (self.private_status & PRIVATE_STATUS_EFFECTIVELY_DEAD) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn set_effectively_dead(&mut self, dead: Bool) {
        if dead != FALSE {
            self.private_status |= PRIVATE_STATUS_EFFECTIVELY_DEAD;
        } else {
            self.private_status &= !PRIVATE_STATUS_EFFECTIVELY_DEAD;
        }
    }

    #[inline]
    pub fn is_undetected_defector(&self) -> Bool {
        if (self.private_status & PRIVATE_STATUS_UNDETECTED_DEFECTOR) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn set_undetected_defector(&mut self, status: Bool) {
        if status != FALSE {
            self.private_status |= PRIVATE_STATUS_UNDETECTED_DEFECTOR;
        } else {
            self.private_status &= !PRIVATE_STATUS_UNDETECTED_DEFECTOR;
        }
    }

    #[inline]
    pub fn is_disabled(&self) -> Bool {
        if self.disabled_mask.any() != FALSE {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn is_disabled_by_type(&self, dt: DisabledType) -> Bool {
        self.disabled_mask.test(dt.as_i32())
    }

    #[inline]
    pub fn set_model_condition_state(&mut self, flag: ModelConditionFlagType) {
        if flag != ModelConditionFlagType::Invalid {
            self.model_condition.set(flag.as_i32(), 1);
        }
    }

    #[inline]
    pub fn clear_model_condition_state(&mut self, flag: ModelConditionFlagType) {
        if flag != ModelConditionFlagType::Invalid {
            self.model_condition.set(flag.as_i32(), 0);
        }
    }

    #[inline]
    pub fn clear_and_set_model_condition_flags(
        &mut self,
        clear: &ModelConditionFlags,
        set: &ModelConditionFlags,
    ) {
        self.model_condition.clear_and_set(clear, set);
    }

    #[inline]
    pub fn clear_special_model_condition_states(&mut self) {
        self.model_condition
            .set(ModelConditionFlagType::SpecialCheering.as_i32(), 0);
        self.special_model_condition_until = 0;
    }

    #[inline]
    pub fn set_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType) {
        let idx = cond.as_i32();
        if (0..32).contains(&idx) {
            self.weapon_bonus_condition |= 1u32 << idx;
        }
    }

    #[inline]
    pub fn clear_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType) {
        let idx = cond.as_i32();
        if (0..32).contains(&idx) {
            self.weapon_bonus_condition &= !(1u32 << idx);
        }
    }

    /// Updates [`Self::model_condition`] for active weapon firing/reloading/preattack status
    /// (`Object::adjustModelConditionForWeaponStatus`, `Object.cpp:4883-4962`).
    pub fn adjust_model_condition_for_weapon_status(&mut self, now: UnsignedInt) {
        let is_attacking = self.status.test(ObjectStatusType::IsAttacking.as_i32()) != FALSE;
        let is_aiming = self
            .status
            .test(ObjectStatusType::IsAimingWeapon.as_i32())
            != FALSE;
        let is_firing = self
            .status
            .test(ObjectStatusType::IsFiringWeapon.as_i32())
            != FALSE;
        let cur_slot = self.weapon_set.get_cur_weapon_slot();

        for i in 0..WEAPONSLOT_COUNT {
            let slot = WeaponSlotType::from_i32(i as Int).unwrap_or(WeaponSlotType::Primary);
            let Some(w) = self.weapon_set.get_weapon_in_weapon_slot(slot) else {
                self.last_weapon_condition[i] = WeaponSetConditionType::None;
                continue;
            };

            let condition_to_set = if slot != cur_slot {
                WeaponSetConditionType::None
            } else if w.last_fire_frame == now {
                WeaponSetConditionType::Firing
            } else if !is_attacking {
                WeaponSetConditionType::None
            } else {
                let status = w.status;
                let base = match status {
                    crate::rules::combat_types::WeaponStatus::ReadyToFire
                    | crate::rules::combat_types::WeaponStatus::OutOfAmmo
                    | crate::rules::combat_types::WeaponStatus::Count => {
                        WeaponSetConditionType::None
                    }
                    crate::rules::combat_types::WeaponStatus::BetweenFiringShots => {
                        WeaponSetConditionType::Between
                    }
                    crate::rules::combat_types::WeaponStatus::ReloadingClip => {
                        WeaponSetConditionType::Reloading
                    }
                    crate::rules::combat_types::WeaponStatus::PreAttack => {
                        WeaponSetConditionType::Preattack
                    }
                };
                if matches!(status, crate::rules::combat_types::WeaponStatus::ReadyToFire)
                    && base == WeaponSetConditionType::None
                    && (is_aiming || is_firing)
                {
                    WeaponSetConditionType::Between
                } else {
                    base
                }
            };

            if condition_to_set != WeaponSetConditionType::Invalid
                && condition_to_set != self.last_weapon_condition[i]
            {
                self.last_weapon_condition[i] = condition_to_set;
                let clr = all_weapon_fire_flags(slot);
                let set = WeaponSet::get_model_condition_for_weapon_slot(slot, condition_to_set);
                self.model_condition.clear_and_set(&clr, &set);
            }
        }
    }
}

/// Read-only simulation view passed to `Snapshot::xfer` and const module queries.
pub trait SimView {
    fn frame(&self) -> UnsignedInt;
    fn rules(&self) -> &RulesRegistry;
    fn module_data(&self, id: ModuleDataId) -> Option<&dyn ModuleData>;
    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot>;
    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord>;
    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar;
    fn terrain(&self) -> &dyn ThingTerrainAccess;

    fn get_wake_frame(&self, slot: ModuleSlot) -> UpdateSleepTime;
    fn module_schedule(&self, slot: ModuleSlot) -> Option<crate::scheduler::UpdateModuleScheduleState>;
    fn get_subdual_damage_heal_rate(&self, owner: ObjectSlot) -> UnsignedInt;
    fn get_subdual_damage_heal_amount(&self, owner: ObjectSlot) -> Real;
    fn has_any_subdual_damage(&self, owner: ObjectSlot) -> Bool;
    fn player_relationship(&self, a: ObjectSlot, b: ObjectSlot) -> Relationship;
    fn player_has_science(&self, team_id: TeamId, science: ScienceType) -> Bool;
    fn player_is_preorder(&self, team_id: TeamId) -> Bool;
    fn player_has_upgrade(&self, team_id: TeamId, mask: &UpgradeMaskType) -> Bool;
    fn adjust_damage_by_armor(
        &self,
        target: ObjectSlot,
        armor_set_flags: &ArmorSetFlags,
        damage_type: DamageType,
        amount: Real,
    ) -> Real;
    fn ai_enable_repulsors(&self) -> Bool;
    fn has_module_by_name_key(&self, owner: ObjectSlot, mod_name_key: NameKeyType) -> bool;

    fn player_money(&self, team_id: TeamId) -> UnsignedInt;
    fn player_is_human(&self, team_id: TeamId) -> Bool;
    fn is_neutral_controlled(&self, owner: ObjectSlot) -> Bool;
    fn object_is_kind_of(&self, owner: ObjectSlot, kindof: KindOfType) -> Bool;
    fn object_is_kind_of_multi(
        &self,
        owner: ObjectSlot,
        must_be_set: &KindOfMaskType,
        must_be_clear: &KindOfMaskType,
    ) -> Bool;
    fn object_veterancy_level(&self, owner: ObjectSlot) -> VeterancyLevel;
    fn object_health_ratio(&self, owner: ObjectSlot) -> Real;
    fn object_max_health(&self, owner: ObjectSlot) -> Real;
    fn object_has_upgrade(&self, owner: ObjectSlot, mask: &UpgradeMaskType) -> Bool;
    fn object_can_crush(&self, crusher: ObjectSlot, victim: ObjectSlot) -> Bool;
    fn object_is_above_terrain(&self, owner: ObjectSlot) -> Bool;
    fn object_has_ai(&self, owner: ObjectSlot) -> Bool;
    fn object_ai_goal_object(&self, owner: ObjectSlot) -> Option<ObjectSlot>;
    fn find_objects_in_range(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot>;
    fn find_objects_in_range_bounding(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot>;
    fn find_special_power_module(
        &self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
    ) -> Option<ModuleSlot>;
    fn shared_special_power_ready_frame(&self, team_id: TeamId, template_id: UnsignedInt) -> UnsignedInt;
    fn player_captured_object_count(&self, team_id: TeamId) -> Int;
    fn special_power_uses_delay(&self) -> Bool;
}

/// Mutable simulation interface passed into all [`BehaviorModule`] facet callbacks.
pub trait SimApi: SimView {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int;
    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real;
    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord>;
    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar);
    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool;
    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime);
    fn set_module_schedule(
        &mut self,
        slot: ModuleSlot,
        schedule: crate::scheduler::UpdateModuleScheduleState,
    );
    fn spawn_object(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
    ) -> ObjectSlot;
    fn destroy_object(&mut self, slot: ObjectSlot);
    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo);
    fn attempt_healing(&mut self, target: ObjectSlot, healing: &mut DamageInfo);
    fn attempt_healing_from_sole_benefactor(
        &mut self,
        target: ObjectSlot,
        amount: Real,
        source: ObjectId,
        duration: UnsignedInt,
    ) -> Bool;
    fn dispatch_on_damage(&mut self, owner: ObjectSlot, dmg: &mut DamageInfo);
    fn dispatch_on_healing(&mut self, owner: ObjectSlot, dmg: &mut DamageInfo);
    fn dispatch_on_body_damage_state_change(
        &mut self,
        owner: ObjectSlot,
        dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    );
    fn dispatch_on_die(&mut self, owner: ObjectSlot, dmg: &DamageInfo);
    fn dispatch_on_build_complete(&mut self, owner: ObjectSlot);
    fn notify_object_killed(&mut self, victim: ObjectSlot, killer: ObjectId, dmg: &DamageInfo);
    fn record_attacked_by(&mut self, victim: ObjectSlot, attacker: ObjectId);
    fn notify_subdual_damage(&mut self, owner: ObjectSlot, amount: Real);
    fn set_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType);
    fn set_disabled_until(
        &mut self,
        owner: ObjectSlot,
        disabled_type: DisabledType,
        till_frame: UnsignedInt,
    );
    fn clear_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType) -> Bool;
    fn grant_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str, record_stats: Bool);
    fn lock_weapon_slot(
        &mut self,
        owner: ObjectSlot,
        weapon_slot: WeaponSlotType,
        lock_type: WeaponLockType,
    ) -> Bool;
    fn set_veterancy_level(
        &mut self,
        owner: ObjectSlot,
        level: VeterancyLevel,
        min_only: Bool,
        provide_feedback: Bool,
    );
    fn register_supply_center(&mut self, owner: ObjectSlot);
    fn register_supply_warehouse(&mut self, owner: ObjectSlot);
    fn on_special_power_creation(&mut self, owner: ObjectSlot);
    fn notify_bone_fx_damage_state_change(
        &mut self,
        owner: ObjectSlot,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    );
    fn set_model_condition_state(&mut self, owner: ObjectSlot, flag: ModelConditionFlagType);
    fn clear_model_condition_state(&mut self, owner: ObjectSlot, flag: ModelConditionFlagType);
    fn clear_and_set_model_condition_flags(
        &mut self,
        owner: ObjectSlot,
        clear: &ModelConditionFlags,
        set: &ModelConditionFlags,
    );
    fn set_special_model_condition_state(
        &mut self,
        owner: ObjectSlot,
        flag: ModelConditionFlagType,
        frames: UnsignedInt,
    );
    fn clear_special_model_condition_states(&mut self, owner: ObjectSlot);
    fn set_status(&mut self, owner: ObjectSlot, status: ObjectStatusType);
    fn clear_status(&mut self, owner: ObjectSlot, status: ObjectStatusType);
    fn set_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    );
    fn clear_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    );
    fn spawn_ocl(&mut self, ocl_name: &str, creator: ObjectSlot, pos: &Coord3D, target_pos: &Coord3D);
    fn trigger_fx_list(&mut self, fx_name: &str, pos: &Coord3D);
    fn trigger_damage_fx(
        &mut self,
        damage_fx_name: &str,
        damage_type: DamageType,
        damage_amount: Real,
        source: ObjectId,
        victim: ObjectSlot,
    );
    fn defect_object(
        &mut self,
        owner: ObjectSlot,
        new_team: TeamId,
        detection_frames: UnsignedInt,
        with_fx: Bool,
    );
    fn trigger_smc_helper(&mut self, owner: ObjectSlot, duration: UnsignedInt);
    fn trigger_status_damage(
        &mut self,
        owner: ObjectSlot,
        status: ObjectStatusType,
        duration: Real,
    );
    fn trigger_subdual_damage(&mut self, owner: ObjectSlot, amount: Real);
    fn trigger_defection(
        &mut self,
        owner: ObjectSlot,
        detection_frames: UnsignedInt,
        with_fx: Bool,
    );
    fn trigger_temp_weapon_bonus(
        &mut self,
        owner: ObjectSlot,
        bonus_type: WeaponBonusConditionType,
        duration: UnsignedInt,
    );
    fn trigger_weapon_status(&mut self, owner: ObjectSlot);
    fn trigger_repulsor(&mut self, owner: ObjectSlot, until_frame: UnsignedInt);
    fn trigger_slow_death(&mut self, owner: ObjectSlot, dmg: &DamageInfo);

    fn add_player_money(&mut self, team_id: TeamId, amount: Int) -> Int;
    fn withdraw_player_money(&mut self, team_id: TeamId, amount: UnsignedInt) -> UnsignedInt;
    fn grant_player_science(&mut self, team_id: TeamId, science: ScienceType) -> Bool;
    fn add_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real);
    fn remove_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real);
    fn remove_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str);
    fn set_armor_set_flag(&mut self, owner: ObjectSlot, flag: ArmorSetType, set: Bool);
    fn set_weapon_set_flag(&mut self, owner: ObjectSlot, flag: WeaponSetType, set: Bool);
    fn set_max_health(
        &mut self,
        owner: ObjectSlot,
        new_max: Real,
        change_type: MaxHealthChangeType,
    );
    fn set_experience_scalar(&mut self, owner: ObjectSlot, scalar: Real);
    fn add_experience_scalar(&mut self, owner: ObjectSlot, delta: Real);
    fn set_command_set(&mut self, owner: ObjectSlot, command_set: &str);
    fn set_locomotor_set(&mut self, owner: ObjectSlot, set_type: LocomotorSetType);
    fn set_shroud_clearing_range(&mut self, owner: ObjectSlot, range: Real);
    fn grant_stealth(&mut self, owner: ObjectSlot);
    fn set_passengers_allowed_to_fire(&mut self, owner: ObjectSlot, allowed: Bool);
    fn add_radar(&mut self, team_id: TeamId, disable_proof: Bool);
    fn remove_radar(&mut self, team_id: TeamId, disable_proof: Bool);
    fn disable_radar(&mut self, team_id: TeamId, frames: UnsignedInt);
    fn extend_power_plant(&mut self, owner: ObjectSlot);
    fn sabotage_power(&mut self, team_id: TeamId, frames: UnsignedInt);
    fn pause_special_power(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        pause: Bool,
    );
    fn reset_special_power_timer(&mut self, owner: ObjectSlot, template_id: UnsignedInt);
    fn express_shared_special_power_ready_frame(
        &mut self,
        team_id: TeamId,
        template_id: UnsignedInt,
        frame: UnsignedInt,
    );
    fn reset_shared_special_power_ready_frame(
        &mut self,
        team_id: TeamId,
        template_id: UnsignedInt,
    ) -> UnsignedInt;
    fn notify_special_power_triggered(
        &mut self,
        team_id: TeamId,
        power_name: &str,
        obj_id: ObjectId,
    );
    fn notify_special_power_complete(&mut self, obj_id: ObjectId, sp_type: SpecialPowerType);
    fn update_public_timer(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        ready_frame: UnsignedInt,
        add: Bool,
    );
    fn reveal_shroud(&mut self, team_id: TeamId, pos: &Coord3D, range: Real, duration: UnsignedInt);
    fn remove_shroud(&mut self, team_id: TeamId);
    fn reveal_map_for_player(&mut self, team_id: TeamId);
    fn fire_weapon_template(
        &mut self,
        owner: ObjectSlot,
        weapon_name: &str,
        target: Option<ObjectSlot>,
        target_pos: &Coord3D,
    );
    fn create_crate_at(
        &mut self,
        crate_tmpl_name: &str,
        killer: Option<ObjectSlot>,
        victim: ObjectSlot,
    );
    fn replace_object(&mut self, owner: ObjectSlot, new_tmpl_name: &str) -> Option<ObjectSlot>;
    fn convert_to_car_bomb(&mut self, target: ObjectSlot, crate_owner: ObjectSlot);
    fn hijack_vehicle(&mut self, target: ObjectSlot, hijacker: ObjectSlot);
    fn apply_cash_bounty_percent(&mut self, team_id: TeamId, percent: Real);
    fn trigger_dam_break(&mut self, owner: ObjectSlot);
    fn notify_rebuild_hole_exposed(
        &mut self,
        dead_obj: ObjectSlot,
        hole_obj: ObjectSlot,
        hole_max_health: Real,
    );
    fn trigger_special_ability(
        &mut self,
        owner: ObjectSlot,
        target: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
    );
    fn trigger_cleanup_area(&mut self, owner: ObjectSlot, loc: &Coord3D);
    fn trigger_baikonur_launch(&mut self, owner: ObjectSlot);
    fn update_sub_objects(&mut self, owner: ObjectSlot, shown: &[String], hidden: &[String]);
    fn dispatch_on_collide(
        &mut self,
        owner: ObjectSlot,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    );
    fn dispatch_attempt_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType);
    fn dispatch_reset_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType);
    fn client(&mut self) -> &mut dyn ClientSink;
}

/// Mutable per-match simulation state (`GameLogic`).
pub struct SimWorld {
    pub frame: UnsignedInt,
    pub rng: PrngState,
    pub objects: ObjectArena<ObjectRecord>,
    pub modules: ModuleArena<Box<dyn BehaviorModule>>,
    pub sleepy_updates: SleepyUpdateHeap<ModuleSlot>,
    pub objects_to_destroy: Vec<(ObjectSlot, u32)>,
    pub next_object_id: Int,
    pub ai_enable_repulsors: Bool,
    pub team_players: BTreeMap<TeamId, Int>,
    pub player_relationships: BTreeMap<(Int, Int), Relationship>,
    pub player_sciences: BTreeMap<TeamId, BTreeSet<ScienceType>>,
    pub player_upgrades: BTreeMap<TeamId, UpgradeMaskType>,
    pub player_preorder: BTreeMap<TeamId, Bool>,
    pub registered_supply_centers: Vec<ObjectId>,
    pub registered_supply_warehouses: Vec<ObjectId>,
    pub kills_scored: Vec<(ObjectId, ObjectId)>,
    pub upgrades_recorded: Vec<(TeamId, String)>,
    pub attacked_by: BTreeMap<TeamId, BTreeSet<Int>>,
    pub spawned_ocls: Vec<(String, ObjectId, Coord3D, Coord3D)>,
    pub triggered_fx_lists: Vec<(String, Coord3D)>,
    pub triggered_damage_fx: Vec<(String, DamageType, Real, ObjectId, ObjectId)>,
    pub player_money: BTreeMap<TeamId, UnsignedInt>,
    pub player_is_human: BTreeMap<TeamId, Bool>,
    pub team_is_neutral: BTreeMap<TeamId, Bool>,
    pub player_cost_modifiers: BTreeMap<TeamId, Vec<(KindOfMaskType, Real)>>,
    pub player_radars: BTreeMap<TeamId, (Int, Int)>,
    pub player_radar_disabled_until: BTreeMap<TeamId, UnsignedInt>,
    pub player_power_sabotaged_until: BTreeMap<TeamId, UnsignedInt>,
    pub player_bounty_percent: BTreeMap<TeamId, Real>,
    pub player_captured_objects: BTreeMap<TeamId, Int>,
    pub shared_special_power_ready_frames: BTreeMap<(TeamId, UnsignedInt), UnsignedInt>,
    pub special_power_uses_delay: Bool,
    pub power_plant_extended: BTreeSet<ObjectId>,
    pub car_bombed_objects: Vec<(ObjectId, ObjectId)>,
    pub hijacked_objects: Vec<(ObjectId, ObjectId)>,
    pub dam_break_events: Vec<ObjectId>,
    pub rebuild_hole_events: Vec<(ObjectId, ObjectId, Real)>,
    pub special_power_triggered_events: Vec<(TeamId, String, ObjectId)>,
    pub special_power_completed_events: Vec<(ObjectId, SpecialPowerType)>,
    pub public_timer_events: Vec<(ObjectId, UnsignedInt, UnsignedInt, Bool)>,
    pub shroud_reveals: Vec<(TeamId, Coord3D, Real, UnsignedInt)>,
    pub shroud_removals: Vec<TeamId>,
    pub map_reveals: Vec<TeamId>,
    pub weapon_fires: Vec<(ObjectId, String, Option<ObjectId>, Coord3D)>,
    pub crate_spawns: Vec<(String, ObjectId, Option<ObjectId>)>,
    pub special_ability_events: Vec<(ObjectId, Option<ObjectId>, Option<Coord3D>)>,
    pub cleanup_area_events: Vec<(ObjectId, Coord3D)>,
    pub baikonur_launch_events: Vec<ObjectId>,
    pub stealth_granted: BTreeSet<ObjectId>,
}

impl Default for SimWorld {
    fn default() -> Self {
        Self::new()
    }
}

impl SimWorld {
    pub fn new() -> Self {
        Self {
            frame: 1,
            rng: PrngState::new(),
            objects: ObjectArena::new(),
            modules: ModuleArena::new(),
            sleepy_updates: SleepyUpdateHeap::new(),
            objects_to_destroy: Vec::new(),
            next_object_id: 1,
            ai_enable_repulsors: TRUE,
            team_players: BTreeMap::new(),
            player_relationships: BTreeMap::new(),
            player_sciences: BTreeMap::new(),
            player_upgrades: BTreeMap::new(),
            player_preorder: BTreeMap::new(),
            registered_supply_centers: Vec::new(),
            registered_supply_warehouses: Vec::new(),
            kills_scored: Vec::new(),
            upgrades_recorded: Vec::new(),
            attacked_by: BTreeMap::new(),
            spawned_ocls: Vec::new(),
            triggered_fx_lists: Vec::new(),
            triggered_damage_fx: Vec::new(),
            player_money: BTreeMap::new(),
            player_is_human: BTreeMap::new(),
            team_is_neutral: BTreeMap::new(),
            player_cost_modifiers: BTreeMap::new(),
            player_radars: BTreeMap::new(),
            player_radar_disabled_until: BTreeMap::new(),
            player_power_sabotaged_until: BTreeMap::new(),
            player_bounty_percent: BTreeMap::new(),
            player_captured_objects: BTreeMap::new(),
            shared_special_power_ready_frames: BTreeMap::new(),
            special_power_uses_delay: TRUE,
            power_plant_extended: BTreeSet::new(),
            car_bombed_objects: Vec::new(),
            hijacked_objects: Vec::new(),
            dam_break_events: Vec::new(),
            rebuild_hole_events: Vec::new(),
            special_power_triggered_events: Vec::new(),
            special_power_completed_events: Vec::new(),
            public_timer_events: Vec::new(),
            shroud_reveals: Vec::new(),
            shroud_removals: Vec::new(),
            map_reveals: Vec::new(),
            weapon_fires: Vec::new(),
            crate_spawns: Vec::new(),
            special_ability_events: Vec::new(),
            cleanup_area_events: Vec::new(),
            baikonur_launch_events: Vec::new(),
            stealth_granted: BTreeSet::new(),
        }
    }

    pub fn allocate_object_id(&mut self) -> ObjectId {
        let id = ObjectId(self.next_object_id);
        self.next_object_id = self.next_object_id.wrapping_add(1);
        if self.next_object_id <= 0 {
            self.next_object_id = 1;
        }
        id
    }

    /// Allocates `module` into [`Self::modules`], links it to `obj_slot`, and pushes any
    /// `UPDATE` facet into [`Self::sleepy_updates`] if the object is linked in the world list.
    pub fn attach_module(
        &mut self,
        obj_slot: ObjectSlot,
        tag_key: NameKeyType,
        data_id: ModuleDataId,
        facet_mask: FacetMask,
        phase: SleepyUpdatePhase,
        module: Box<dyn BehaviorModule>,
    ) -> ModuleSlot {
        let initial_sidecar = module.sync_sidecar();
        let owner_id = self.objects.id_of(obj_slot).unwrap_or(ObjectId::INVALID);
        let mut meta = ModuleMeta::new(obj_slot, owner_id, tag_key, data_id, facet_mask, phase);
        meta.sidecar = initial_sidecar;
        let mod_slot = self.modules.allocate(meta, module);

        if let Some(obj) = self.objects.get_mut(obj_slot) {
            obj.behaviors.push((mod_slot, facet_mask));
            if facet_mask.contains(FacetMask::BODY) && obj.body_slot.is_none() {
                obj.body_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::CONTAIN) && obj.contain_slot.is_none() {
                obj.contain_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::STEALTH_UPDATE) && obj.stealth_slot.is_none() {
                obj.stealth_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::AI_UPDATE) && obj.ai_slot.is_none() {
                obj.ai_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::PHYSICS_UPDATE) && obj.physics_slot.is_none() {
                obj.physics_slot = Some(mod_slot);
            }
        }

        if facet_mask.contains(FacetMask::UPDATE) && self.objects.is_in_world_list(obj_slot) {
            self.sleepy_updates
                .register_update_module(&mut self.modules, mod_slot, self.frame);
        }

        mod_slot
    }

    /// Enqueues `(slot, generation)` for Step 15a destruction (`GameLogic::m_objectsToDestroy`).
    pub fn enqueue_destroy(&mut self, slot: ObjectSlot) {
        if let Some(gen_id) = self.objects.generation(slot) {
            if !self
                .objects_to_destroy
                .iter()
                .any(|&(s, g)| s == slot && g == gen_id)
            {
                self.objects_to_destroy.push((slot, gen_id));
            }
        }
    }
}

/// Top-level three-layer simulation context (`RulesRegistry` + `SimWorld` + `ClientSink`).
pub struct SimContext {
    pub rules: RulesRegistry,
    pub world: SimWorld,
    pub terrain: Box<dyn ThingTerrainAccess>,
    pub client: Box<dyn ClientSink>,
}

impl Default for SimContext {
    fn default() -> Self {
        Self::new()
    }
}

impl SimContext {
    pub fn new() -> Self {
        Self {
            rules: RulesRegistry::new(),
            world: SimWorld::new(),
            terrain: Box::new(FlatTerrain),
            client: Box::new(NoopClientSink),
        }
    }

    /// Resolves the controlling player index for `team_id` (`team_players` override or `team_id.0`).
    #[inline]
    fn player_index_for_team(&self, team_id: TeamId) -> Int {
        self.world
            .team_players
            .get(&team_id)
            .copied()
            .unwrap_or(team_id.0)
    }

    /// Dispatches `f` on `mod_slot` via Take-and-Restore, draining any re-entrant [`DeferredSelfOp`]s
    /// and refreshing [`ModuleSidecar`] before restoring ownership to `world.modules`.
    pub fn dispatch_module<R>(
        &mut self,
        mod_slot: ModuleSlot,
        f: impl FnOnce(&mut dyn BehaviorModule, &mut dyn SimApi, ModuleContext) -> R,
    ) -> Result<R, TakeError> {
        let (mut payload, ctx, gen_id) = self.world.modules.take_payload(mod_slot)?;
        let out = f(&mut *payload, self, ctx);

        loop {
            let ops = if self.world.modules.is_executing(mod_slot)
                && self.world.modules.meta(mod_slot).generation == gen_id
            {
                self.world.modules.drain_deferred_ops(mod_slot)
            } else {
                Vec::new()
            };
            if ops.is_empty() {
                break;
            }
            for op in ops {
                payload.apply_deferred_op(op, self, ctx);
            }
        }

        if self.world.modules.is_executing(mod_slot)
            && self.world.modules.meta(mod_slot).generation == gen_id
        {
            let sidecar = payload.sync_sidecar();
            if sidecar != ModuleSidecar::None {
                self.world.modules.set_sidecar(mod_slot, sidecar);
            }
        }
        self.world.modules.restore_payload(mod_slot, gen_id, payload);
        Ok(out)
    }

    /// Marks `slot` destroyed (`GameLogic::destroyObject`), running `DestroyFacet::on_destroy`
    /// and queueing `slot` for end-of-frame deletion in [`Self::process_destroy_list`].
    pub fn destroy_object_now(&mut self, slot: ObjectSlot) {
        let Some(obj) = self.world.objects.get(slot) else {
            return;
        };
        if obj.is_destroyed() {
            return;
        }
        let obj_id = obj.id;
        let destroy_slots: Vec<ModuleSlot> = obj
            .behaviors
            .iter()
            .filter(|(_, mask)| mask.contains(FacetMask::DESTROY))
            .map(|&(ms, _)| ms)
            .collect();
        let ai_slot = obj.ai_slot;

        for ms in destroy_slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_destroy_mut() {
                    d.on_destroy(sim, ctx);
                }
            });
        }

        if let Some(obj_mut) = self.world.objects.get_mut(slot) {
            obj_mut.status.set(ObjectStatusType::Destroyed.as_i32(), 1);
        }

        if let Some(ai_ms) = ai_slot {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_goal_none(sim, ctx);
                    ai.destroy_path();
                }
            });
        }

        self.world.enqueue_destroy(slot);
        self.client.on_object_destroyed(obj_id);
    }

    /// Destroys all queued objects (`GameLogic::processDestroyList`), handling child objects
    /// enqueued dynamically inside `on_delete()`, two-pass heap removal, and arena despawn.
    pub fn process_destroy_list(&mut self) {
        let mut i = 0;
        while i < self.world.objects_to_destroy.len() {
            let (slot, gen_id) = self.world.objects_to_destroy[i];
            i += 1;

            if !self.world.objects.is_slot_current(slot, gen_id) {
                continue;
            }
            let beh_slots: Vec<ModuleSlot> = self
                .world
                .objects
                .get(slot)
                .map_or_else(Vec::new, |o| o.behaviors.iter().map(|&(ms, _)| ms).collect());

            for &ms in &beh_slots {
                if self.world.modules.is_alive(ms) {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        m.on_delete(sim, ctx);
                    });
                }
            }

            let modules = &mut self.world.modules;
            self.world
                .sleepy_updates
                .erase_for_owner(modules, |m, ms| m.meta(ms).owner_slot == slot);

            for ms in beh_slots {
                let _ = self.world.modules.free(ms);
            }
            let _ = self.world.objects.despawn(slot);
        }
        self.world.objects_to_destroy.clear();
    }

    /// Executes all due sleepy updates for `self.world.frame` (`GameLogic.cpp:4062-4113`).
    pub fn step_sleepy_updates(&mut self) {
        let now = self.world.frame;
        while let Some(u) = self.world.sleepy_updates.peek_due(&self.world.modules, now) {
            let owner_slot = self.world.modules.meta(u).owner_slot;
            let disabled_bits = self
                .world
                .objects
                .get(owner_slot)
                .map_or(0, |o| o.disabled_mask.to_unsigned_int() as u64);
            let proc_bits = self
                .world
                .modules
                .payload(u)
                .and_then(|m| m.as_update())
                .map_or(0, |upd| upd.disabled_types_to_process().to_unsigned_int() as u64);

            let mut sleep_len = UpdateSleepTime::NONE;
            if should_process_disabled_update(disabled_bits, proc_bits) {
                self.world.sleepy_updates.set_cur_update_module(Some(u));
                if let Ok(ret) = self.dispatch_module(u, |m, sim, ctx| {
                    m.as_update_mut()
                        .map_or(UpdateSleepTime::FOREVER, |upd| upd.update(sim, ctx))
                }) {
                    sleep_len = ret;
                }
            }

            if self.world.modules.is_alive(u)
                && self.world.modules.schedule(u).index_in_logic() >= 0
            {
                self.world
                    .sleepy_updates
                    .finish_due_update(&mut self.world.modules, u, now, sleep_len);
            } else {
                self.world.sleepy_updates.set_cur_update_module(None);
            }
        }
    }

    /// Routes damage through `target`'s `BodyFacet`, handling same-slot re-entrancy via
    /// armor/scalar/subdual adjustment, [`ModuleSidecar::Body`] immediate sync, and typed
    /// [`DeferredSelfOp`] queueing.
    pub fn attempt_damage_on(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        let Some(body_slot) = self.world.objects.get(target).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(body_slot) {
            let effectively_dead = self
                .world
                .objects
                .get(target)
                .is_none_or(|o| o.is_effectively_dead() != FALSE);
            if effectively_dead {
                return;
            }
            if let ModuleSidecar::Body {
                mut health,
                max_health,
                prev_health: _,
                initial_health,
                damage_scalar,
                mut damage_state,
                front_crushed,
                back_crushed,
                indestructible,
                mut current_subdual_damage,
                subdued,
                last_damage_ts,
                last_healing_ts,
                armor_set_bits,
            } = self.world.modules.sidecar(body_slot)
            {
                if indestructible != FALSE {
                    return;
                }
                if dmg.input.damage_type == DamageType::Healing {
                    return;
                }
                let armor_flags = armor_set_flags_from_u8(armor_set_bits);
                let mut amount = self.adjust_damage_by_armor(
                    target,
                    &armor_flags,
                    dmg.input.damage_type,
                    dmg.input.amount,
                );
                if is_subdual_damage(dmg.input.damage_type.as_i32()) {
                    current_subdual_damage += amount;
                    #[cfg(not(feature = "retail-crc"))]
                    if current_subdual_damage < 0.0 {
                        current_subdual_damage = 0.0;
                    }
                    self.world.modules.set_sidecar(
                        body_slot,
                        ModuleSidecar::Body {
                            health,
                            max_health,
                            prev_health: health,
                            initial_health,
                            damage_scalar,
                            damage_state,
                            front_crushed,
                            back_crushed,
                            indestructible,
                            current_subdual_damage,
                            subdued,
                            last_damage_ts,
                            last_healing_ts,
                            armor_set_bits,
                        },
                    );
                    let _ = self
                        .world
                        .modules
                        .push_deferred_op(body_slot, DeferredSelfOp::AddSubdualDamage(amount));
                    return;
                }
                if !matches!(
                    dmg.input.damage_type,
                    DamageType::Status | DamageType::KillPilot | DamageType::KillGarrisoned
                ) {
                    if dmg.input.damage_type != DamageType::Unresistable {
                        amount *= damage_scalar;
                    }
                    if amount > 0.0 || dmg.input.kill != FALSE {
                        let applied = if dmg.input.kill != FALSE { health } else { amount };
                        let prev_health = health;
                        health -= applied;
                        if health < 0.0 {
                            health = 0.0;
                        }
                        if health > max_health && max_health > 0.0 {
                            health = max_health;
                        }
                        if health <= 0.0 {
                            damage_state = BodyDamageType::Rubble.as_i32() as u8;
                        }
                        self.world.modules.set_sidecar(
                            body_slot,
                            ModuleSidecar::Body {
                                health,
                                max_health,
                                prev_health,
                                initial_health,
                                damage_scalar,
                                damage_state,
                                front_crushed,
                                back_crushed,
                                indestructible,
                                current_subdual_damage,
                                subdued,
                                last_damage_ts,
                                last_healing_ts,
                                armor_set_bits,
                            },
                        );
                        let _ = self.world.modules.push_deferred_op(
                            body_slot,
                            DeferredSelfOp::ApplyAdjustedDamage {
                                amount: applied,
                                damage_type: dmg.input.damage_type.as_i32() as u8,
                                death_type: dmg.input.death_type.as_i32() as u8,
                                damage_fx_override: dmg.input.damage_fx_override.as_i32() as u8,
                                damage_status_type: dmg.input.damage_status_type.as_i32() as u8,
                                kill: dmg.input.kill,
                                source_id: dmg.input.source_id,
                                source_player_mask: dmg.input.source_player_mask,
                            },
                        );
                    }
                }
            }
            return;
        }

        if let Some(p) = self.world.modules.payload(body_slot) {
            let sc = p.sync_sidecar();
            if sc != ModuleSidecar::None {
                self.world.modules.set_sidecar(body_slot, sc);
            }
        }
        let _ = self.dispatch_module(body_slot, |m, sim, ctx| {
            if let Some(body) = m.as_body_mut() {
                body.attempt_damage(sim, ctx, dmg);
            }
        });
    }

    /// Routes healing through `target`'s `BodyFacet`.
    pub fn attempt_healing_on(&mut self, target: ObjectSlot, healing: &mut DamageInfo) {
        let Some(body_slot) = self.world.objects.get(target).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(body_slot) {
            let amount = healing.input.amount;
            if amount > 0.0 {
                let _ = self.world.modules.push_deferred_op(
                    body_slot,
                    DeferredSelfOp::ChangeHealth {
                        delta: amount,
                        change_type: 0,
                    },
                );
            }
            return;
        }
        if let Some(p) = self.world.modules.payload(body_slot) {
            let sc = p.sync_sidecar();
            if sc != ModuleSidecar::None {
                self.world.modules.set_sidecar(body_slot, sc);
            }
        }
        let _ = self.dispatch_module(body_slot, |m, sim, ctx| {
            if let Some(body) = m.as_body_mut() {
                body.attempt_healing(sim, ctx, healing);
            }
        });
    }

    /// Spawns an object from `tmpl` with explicit `initial_status` flags matching
    /// `ThingFactory::newObject(tmpl, team, statusBits)`.
    pub fn spawn_object_with_status(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
        initial_status: ObjectStatusMaskType,
    ) -> ObjectSlot {
        let id = self.world.allocate_object_id();
        let tmpl_data = self
            .rules
            .thing_factory
            .find_by_template_id_final(tmpl.0 as u16)
            .map(|t| {
                (
                    t.get_template_geometry_info().clone(),
                    t.is_trainable(),
                    t.get_behavior_module_info().nuggets().to_vec(),
                    t.find_weapon_template_set(&crate::rules::combat_types::WeaponSetFlags::new())
                        .cloned(),
                )
            });
        let (geom, trainable, nuggets, wts) = match tmpl_data {
            Some((g, tr, n, w)) => (g, tr, n, w),
            None => (GeometryInfo::default(), FALSE, Vec::new(), None),
        };

        let mut record = ObjectRecord::new(id, tmpl, team, geom);
        record.status = initial_status;
        record.experience_tracker.set_trainable(trainable);
        if let Some(ref set) = wts {
            record.weapon_set.init_from_template_set(set, self.world.frame);
        }
        let mut dummy_reactor = crate::object::thing::NoopTransformReactor;
        let mut mtx = zh_common::math::Matrix3D::identity();
        self.terrain.align_on_terrain(angle, &pos, FALSE, &mut mtx);
        record.thing.set_transform_matrix(&mtx, &mut dummy_reactor);

        let has_template = self
            .rules
            .thing_factory
            .find_by_template_id_final(tmpl.0 as u16)
            .is_some();
        if !has_template {
            let slot = self.world.objects.spawn(id, record, true);
            self.client.on_object_created(id, tmpl);
            return slot;
        }

        // Attach helpers + INI behavior modules while unlinked so constructor initial
        // sleep frames are preserved when pushing UPDATE facets into SleepyUpdateHeap.
        let slot = self.world.objects.spawn(id, record, false);
        crate::modules::helper::attach_object_helpers(self, slot);

        for nug in &nuggets {
            let Some(desc) = self
                .rules
                .module_registry
                .find_descriptor(&nug.name, ModuleType::Behavior)
                .cloned()
            else {
                continue;
            };
            let Some(create_fn) = desc.create_module else {
                continue;
            };
            let Some(mod_data) = self.rules.module_data_store.get(nug.data_id) else {
                continue;
            };
            let ctx = ModuleContext {
                mod_slot: ModuleSlot::INVALID,
                owner_slot: slot,
                owner_id: id,
                tag_key: nug.tag_key,
                template_data_id: nug.data_id,
            };
            let module = create_fn(mod_data, ctx);
            self.world.attach_module(
                slot,
                nug.tag_key,
                nug.data_id,
                desc.facet_mask,
                desc.update_phase,
                module,
            );
        }

        self.world.objects.link_into_world(slot);
        let behaviors: Vec<(ModuleSlot, FacetMask)> = self
            .world
            .objects
            .get(slot)
            .map_or_else(Vec::new, |o| o.behaviors.clone());
        for &(ms, mask) in &behaviors {
            if mask.contains(FacetMask::UPDATE) {
                self.world.sleepy_updates.register_update_module(
                    &mut self.world.modules,
                    ms,
                    self.world.frame,
                );
            }
        }
        for &(ms, _) in &behaviors {
            if self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    m.on_object_created(sim, ctx);
                });
            }
        }
        for &(ms, mask) in &behaviors {
            if mask.contains(FacetMask::CREATE) && self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    if let Some(c) = m.as_create_mut() {
                        c.on_create(sim, ctx);
                    }
                });
            }
        }

        self.client.on_object_created(id, tmpl);
        slot
    }
}

struct ObjectExpParent<'a> {
    tmpl: &'a ThingTemplate,
    killer_rel: Relationship,
    level_change: Option<(VeterancyLevel, VeterancyLevel, Bool)>,
}

impl<'a> ExperienceParentAccess for ObjectExpParent<'a> {
    fn is_template_trainable(&self) -> Bool {
        self.tmpl.is_trainable()
    }

    fn get_experience_required(&self, level: Int) -> Int {
        self.tmpl
            .get_experience_required(level.clamp(0, 3) as usize)
    }

    fn get_experience_value(&self, level: Int) -> Int {
        self.tmpl.get_experience_value(level.clamp(0, 3) as usize)
    }

    fn get_relationship_with_killer(&self) -> Relationship {
        self.killer_rel
    }

    fn on_veterancy_level_changed(
        &mut self,
        old_level: VeterancyLevel,
        new_level: VeterancyLevel,
        provide_feedback: Bool,
    ) {
        self.level_change = Some((old_level, new_level, provide_feedback));
    }

    fn forward_experience_to_sink(
        &mut self,
        _sink_id: ObjectId,
        _scaled_exp_gain: Int,
        _can_scale_for_bonus: Bool,
    ) -> bool {
        false
    }

    fn forward_set_exp_and_level_to_sink(
        &mut self,
        _sink_id: ObjectId,
        _experience_in: Int,
        _provide_feedback: Bool,
    ) -> bool {
        false
    }
}

impl SimView for SimContext {
    fn frame(&self) -> UnsignedInt {
        self.world.frame
    }

    fn rules(&self) -> &RulesRegistry {
        &self.rules
    }

    fn module_data(&self, id: ModuleDataId) -> Option<&dyn ModuleData> {
        self.rules.module_data_store.get(id)
    }

    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.world.objects.slot_by_id(id)
    }

    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord> {
        self.world.objects.get(slot)
    }

    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar {
        self.world.modules.sidecar(slot)
    }

    fn terrain(&self) -> &dyn ThingTerrainAccess {
        self.terrain.as_ref()
    }

    fn get_wake_frame(&self, slot: ModuleSlot) -> UpdateSleepTime {
        if !self.world.modules.is_alive(slot) {
            return UpdateSleepTime::FOREVER;
        }
        self.world
            .modules
            .schedule(slot)
            .get_wake_frame(self.world.frame)
    }

    fn module_schedule(&self, slot: ModuleSlot) -> Option<crate::scheduler::UpdateModuleScheduleState> {
        if !self.world.modules.is_alive(slot) {
            return None;
        }
        Some(*self.world.modules.schedule(slot))
    }

    fn get_subdual_damage_heal_rate(&self, owner: ObjectSlot) -> UnsignedInt {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return 0;
        };
        if let Some(body) = self.world.modules.payload(bs).and_then(|m| m.as_body()) {
            return body.get_subdual_damage_heal_rate();
        }
        if !self.world.modules.is_alive(bs) {
            return 0;
        }
        let data_id = self.world.modules.meta(bs).template_data_id;
        self.rules
            .module_data_store
            .get(data_id)
            .map_or(0, |d| d.subdual_damage_heal_rate())
    }

    fn get_subdual_damage_heal_amount(&self, owner: ObjectSlot) -> Real {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return 0.0;
        };
        if let Some(body) = self.world.modules.payload(bs).and_then(|m| m.as_body()) {
            return body.get_subdual_damage_heal_amount();
        }
        if !self.world.modules.is_alive(bs) {
            return 0.0;
        }
        let data_id = self.world.modules.meta(bs).template_data_id;
        self.rules
            .module_data_store
            .get(data_id)
            .map_or(0.0, |d| d.subdual_damage_heal_amount())
    }

    fn has_any_subdual_damage(&self, owner: ObjectSlot) -> Bool {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return FALSE;
        };
        if let Some(body) = self.world.modules.payload(bs).and_then(|m| m.as_body()) {
            return body.has_any_subdual_damage();
        }
        match self.world.modules.sidecar(bs) {
            ModuleSidecar::Body {
                current_subdual_damage,
                ..
            } => Bool::from(current_subdual_damage > 0.0),
            _ => FALSE,
        }
    }

    fn player_relationship(&self, a: ObjectSlot, b: ObjectSlot) -> Relationship {
        let Some(oa) = self.world.objects.get(a) else {
            return Relationship::Neutral;
        };
        let Some(ob) = self.world.objects.get(b) else {
            return Relationship::Neutral;
        };
        let pa = self.player_index_for_team(oa.team_id);
        let pb = self.player_index_for_team(ob.team_id);
        if pa == pb {
            return Relationship::Allies;
        }
        self.world
            .player_relationships
            .get(&(pa, pb))
            .copied()
            .unwrap_or(Relationship::Neutral)
    }

    fn player_has_science(&self, team_id: TeamId, science: ScienceType) -> Bool {
        if self
            .world
            .player_sciences
            .get(&team_id)
            .is_some_and(|set| set.contains(&science))
        {
            TRUE
        } else {
            FALSE
        }
    }

    fn player_is_preorder(&self, team_id: TeamId) -> Bool {
        self.world
            .player_preorder
            .get(&team_id)
            .copied()
            .unwrap_or(FALSE)
    }

    fn player_has_upgrade(&self, team_id: TeamId, mask: &UpgradeMaskType) -> Bool {
        if self
            .world
            .player_upgrades
            .get(&team_id)
            .is_some_and(|owned| owned.test_for_any(mask) != FALSE)
        {
            TRUE
        } else {
            FALSE
        }
    }

    fn adjust_damage_by_armor(
        &self,
        target: ObjectSlot,
        armor_set_flags: &ArmorSetFlags,
        damage_type: DamageType,
        amount: Real,
    ) -> Real {
        let Some(obj) = self.world.objects.get(target) else {
            return amount;
        };
        let Some(tmpl) = obj.thing.template_override().get_final(&self.rules.thing_factory) else {
            return amount;
        };
        let armor_tmpl = tmpl
            .find_armor_template_set(armor_set_flags)
            .and_then(|set| set.get_armor_template());
        Armor::new(armor_tmpl).adjust_damage(damage_type, amount)
    }

    fn ai_enable_repulsors(&self) -> Bool {
        self.world.ai_enable_repulsors
    }

    fn has_module_by_name_key(&self, owner: ObjectSlot, mod_name_key: NameKeyType) -> bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return false;
        };
        obj.behaviors.iter().any(|&(ms, _)| {
            self.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == mod_name_key)
        })
    }

    fn player_money(&self, team_id: TeamId) -> UnsignedInt {
        self.world.player_money.get(&team_id).copied().unwrap_or(0)
    }

    fn player_is_human(&self, team_id: TeamId) -> Bool {
        self.world
            .player_is_human
            .get(&team_id)
            .copied()
            .unwrap_or(FALSE)
    }

    fn is_neutral_controlled(&self, owner: ObjectSlot) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return TRUE;
        };
        if let Some(&neutral) = self.world.team_is_neutral.get(&obj.team_id) {
            return neutral;
        }
        Bool::from(obj.team_id.0 <= 0)
    }

    fn object_is_kind_of(&self, owner: ObjectSlot, kindof: KindOfType) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        obj.thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .map_or(FALSE, |t| t.is_kind_of(kindof))
    }

    fn object_is_kind_of_multi(
        &self,
        owner: ObjectSlot,
        must_be_set: &KindOfMaskType,
        must_be_clear: &KindOfMaskType,
    ) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        obj.thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .map_or(FALSE, |t| t.is_kind_of_multi(must_be_set, must_be_clear))
    }

    fn object_veterancy_level(&self, owner: ObjectSlot) -> VeterancyLevel {
        self.world
            .objects
            .get(owner)
            .map_or(VeterancyLevel::Regular, |o| {
                o.experience_tracker.get_veterancy_level()
            })
    }

    fn object_health_ratio(&self, owner: ObjectSlot) -> Real {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return 0.0;
        };
        if let Some(b) = self.world.modules.payload(bs).and_then(|p| p.as_body()) {
            return div_safe_f32(b.get_health(), b.get_max_health());
        }
        match self.world.modules.sidecar(bs) {
            ModuleSidecar::Body {
                health, max_health, ..
            } => div_safe_f32(health, max_health),
            _ => 0.0,
        }
    }

    fn object_max_health(&self, owner: ObjectSlot) -> Real {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return 0.0;
        };
        if let Some(b) = self.world.modules.payload(bs).and_then(|p| p.as_body()) {
            return b.get_max_health();
        }
        match self.world.modules.sidecar(bs) {
            ModuleSidecar::Body { max_health, .. } => max_health,
            _ => 0.0,
        }
    }

    fn object_has_upgrade(&self, owner: ObjectSlot, mask: &UpgradeMaskType) -> Bool {
        self.world
            .objects
            .get(owner)
            .map_or(FALSE, |o| o.upgrades_completed.test_for_any(mask))
    }

    fn object_can_crush(&self, crusher: ObjectSlot, victim: ObjectSlot) -> Bool {
        let Some(oc) = self.world.objects.get(crusher) else {
            return FALSE;
        };
        let Some(ov) = self.world.objects.get(victim) else {
            return FALSE;
        };
        let Some(tc) = oc
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)
        else {
            return FALSE;
        };
        let Some(tv) = ov
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)
        else {
            return FALSE;
        };
        let crusher_lvl = tc.get_crusher_level();
        let crushable_lvl = tv.get_crushable_level();
        Bool::from(crusher_lvl > 1 && crusher_lvl > crushable_lvl)
    }

    fn object_is_above_terrain(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .map_or(FALSE, |o| o.thing.is_above_terrain(self.terrain.as_ref()))
    }

    fn object_has_ai(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .map_or(FALSE, |o| Bool::from(o.ai_slot.is_some()))
    }

    fn object_ai_goal_object(&self, owner: ObjectSlot) -> Option<ObjectSlot> {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.ai_goal_object)
    }

    fn find_objects_in_range(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot> {
        let r2 = radius * radius;
        self.world
            .objects
            .world_iter()
            .filter(|&slot| {
                self.world.objects.get(slot).is_some_and(|obj| {
                    if obj.is_destroyed() {
                        return false;
                    }
                    let pos = obj.thing.get_position();
                    let dx = pos.x - center.x;
                    let dy = pos.y - center.y;
                    (dx * dx + dy * dy) <= r2
                })
            })
            .collect()
    }

    fn find_objects_in_range_bounding(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot> {
        self.world
            .objects
            .world_iter()
            .filter(|&slot| {
                self.world.objects.get(slot).is_some_and(|obj| {
                    if obj.is_destroyed() {
                        return false;
                    }
                    let pos = obj.thing.get_position();
                    let dx = pos.x - center.x;
                    let dy = pos.y - center.y;
                    let eff = radius + obj.geometry.get_bounding_circle_radius();
                    (dx * dx + dy * dy) <= (eff * eff)
                })
            })
            .collect()
    }

    fn find_special_power_module(
        &self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
    ) -> Option<ModuleSlot> {
        let obj = self.world.objects.get(owner)?;
        for &(ms, mask) in &obj.behaviors {
            if !mask.contains(FacetMask::SPECIAL_POWER) {
                continue;
            }
            if let Some(p) = self.world.modules.payload(ms) {
                if let Some(sp) = p.as_special_power() {
                    let ctx = self.world.modules.meta(ms).context(ms);
                    if sp.is_module_for_power(self, ctx, template_id) != FALSE {
                        return Some(ms);
                    }
                }
            } else if self.world.modules.is_executing(ms) {
                if let ModuleSidecar::SpecialPower {
                    template_id: tid, ..
                } = self.world.modules.sidecar(ms)
                {
                    if tid == template_id {
                        return Some(ms);
                    }
                }
            }
        }
        None
    }

    fn shared_special_power_ready_frame(
        &self,
        team_id: TeamId,
        template_id: UnsignedInt,
    ) -> UnsignedInt {
        self.world
            .shared_special_power_ready_frames
            .get(&(team_id, template_id))
            .copied()
            .unwrap_or(self.world.frame)
    }

    fn player_captured_object_count(&self, team_id: TeamId) -> Int {
        self.world
            .player_captured_objects
            .get(&team_id)
            .copied()
            .unwrap_or(0)
    }

    fn special_power_uses_delay(&self) -> Bool {
        self.world.special_power_uses_delay
    }
}

impl SimApi for SimContext {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int {
        self.world.rng.get_game_logic_random_value(lo, hi)
    }

    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real {
        self.world.rng.get_game_logic_random_value_real(lo, hi)
    }

    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord> {
        self.world.objects.get_mut(slot)
    }

    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar) {
        self.world.modules.set_sidecar(slot, sidecar);
    }

    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool {
        if !self.world.modules.is_alive(slot) {
            return false;
        }
        let owner_slot = self.world.modules.meta(slot).owner_slot;
        let in_world = self.world.objects.is_in_world_list(owner_slot);
        self.world.sleepy_updates.awaken_module(
            &mut self.world.modules,
            slot,
            in_world,
            self.world.frame,
            when,
        )
    }

    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime) {
        let when = self.world.frame.saturating_add(sleep.0);
        let _ = self.awaken_module(slot, when);
    }

    fn set_module_schedule(
        &mut self,
        slot: ModuleSlot,
        schedule: crate::scheduler::UpdateModuleScheduleState,
    ) {
        if self.world.modules.is_alive(slot) {
            let _ = self.awaken_module(slot, schedule.next_call_frame());
        }
    }

    fn spawn_object(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
    ) -> ObjectSlot {
        self.spawn_object_with_status(tmpl, team, pos, angle, ObjectStatusMaskType::new())
    }

    fn destroy_object(&mut self, slot: ObjectSlot) {
        self.destroy_object_now(slot);
    }

    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        self.attempt_damage_on(target, dmg);
    }

    fn attempt_healing(&mut self, target: ObjectSlot, healing: &mut DamageInfo) {
        self.attempt_healing_on(target, healing);
    }

    fn attempt_healing_from_sole_benefactor(
        &mut self,
        target: ObjectSlot,
        amount: Real,
        source: ObjectId,
        duration: UnsignedInt,
    ) -> Bool {
        if source == ObjectId::INVALID {
            return FALSE;
        }
        let now = self.world.frame;
        let Some(obj) = self.world.objects.get_mut(target) else {
            return FALSE;
        };
        if now > obj.sole_healing_benefactor_expiration_frame
            || obj.sole_healing_benefactor_id == source
        {
            obj.sole_healing_benefactor_id = source;
            obj.sole_healing_benefactor_expiration_frame = now.saturating_add(duration);
            let mut healing = DamageInfo::default();
            healing.input.damage_type = DamageType::Healing;
            healing.input.death_type = crate::rules::combat_types::DeathType::NoneType;
            healing.input.source_id = source;
            healing.input.amount = amount;
            self.attempt_healing_on(target, &mut healing);
            TRUE
        } else {
            FALSE
        }
    }

    fn dispatch_on_damage(&mut self, owner: ObjectSlot, dmg: &mut DamageInfo) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::DAMAGE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_damage_mut() {
                    d.on_damage(sim, ctx, dmg);
                }
            });
        }
        if dmg.output.actual_damage_clipped != 0.0 {
            if let Some(id) = self.world.objects.id_of(owner) {
                self.client.on_damage(id, dmg.output.actual_damage_clipped);
            }
        }
    }

    fn dispatch_on_healing(&mut self, owner: ObjectSlot, dmg: &mut DamageInfo) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::DAMAGE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_damage_mut() {
                    d.on_healing(sim, ctx, dmg);
                }
            });
        }
    }

    fn dispatch_on_body_damage_state_change(
        &mut self,
        owner: ObjectSlot,
        dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::DAMAGE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_damage_mut() {
                    d.on_body_damage_state_change(sim, ctx, dmg, old_state, new_state);
                }
            });
        }
        if let Some(id) = self.world.objects.id_of(owner) {
            self.client.on_body_damage_state_change(id, new_state);
        }
    }

    fn dispatch_on_die(&mut self, owner: ObjectSlot, dmg: &DamageInfo) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::DIE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_die_mut() {
                    d.on_die(sim, ctx, dmg);
                }
            });
        }
    }

    fn dispatch_on_build_complete(&mut self, owner: ObjectSlot) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::CREATE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(c) = m.as_create_mut() {
                    c.on_build_complete(sim, ctx);
                }
            });
        }
    }

    fn notify_object_killed(&mut self, victim: ObjectSlot, killer: ObjectId, _dmg: &DamageInfo) {
        if killer == ObjectId::INVALID {
            return;
        }
        if let Some(victim_id) = self.world.objects.id_of(victim) {
            self.world.kills_scored.push((killer, victim_id));
        }
    }

    fn record_attacked_by(&mut self, victim: ObjectSlot, attacker: ObjectId) {
        if attacker == ObjectId::INVALID {
            return;
        }
        let Some(victim_team) = self.world.objects.get(victim).map(|o| o.team_id) else {
            return;
        };
        let Some(atk_slot) = self.world.objects.slot_by_id(attacker) else {
            return;
        };
        let Some(atk_team) = self.world.objects.get(atk_slot).map(|o| o.team_id) else {
            return;
        };
        let atk_player = self.player_index_for_team(atk_team);
        self.world
            .attacked_by
            .entry(victim_team)
            .or_default()
            .insert(atk_player);
    }

    fn notify_subdual_damage(&mut self, owner: ObjectSlot, amount: Real) {
        let Some(hs) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.subdual_damage_helper_slot)
        else {
            return;
        };
        let _ = self.dispatch_module(hs, |m, sim, ctx| {
            if let Some(h) = m.as_helper_mut() {
                h.notify_subdual_damage(sim, ctx, amount);
            }
        });
    }

    fn set_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType) {
        self.set_disabled_until(owner, disabled_type, u32::MAX);
    }

    fn set_disabled_until(
        &mut self,
        owner: ObjectSlot,
        disabled_type: DisabledType,
        till_frame: UnsignedInt,
    ) {
        let now = self.world.frame;
        let idx = disabled_type.as_i32();
        if !(0..DISABLED_COUNT as Int).contains(&idx) {
            return;
        }
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let was_disabled = obj.is_disabled() != FALSE;
        let slot_idx = idx as usize;
        let mut behaviors_to_edge = Vec::new();
        if obj.disabled_till_frame[slot_idx] != till_frame {
            obj.disabled_till_frame[slot_idx] = till_frame;
            obj.disabled_mask.set(idx, i32::from(till_frame > now));
        }
        if !was_disabled && obj.is_disabled() != FALSE {
            behaviors_to_edge = obj.behaviors.iter().map(|&(ms, _)| ms).collect();
        }
        for ms in behaviors_to_edge {
            if self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    m.on_disabled_edge(sim, ctx, TRUE);
                });
            }
        }
    }

    fn clear_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType) -> Bool {
        let idx = disabled_type.as_i32();
        if !(0..DISABLED_COUNT as Int).contains(&idx) {
            return FALSE;
        }
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return FALSE;
        };
        if obj.is_disabled_by_type(disabled_type) == FALSE {
            return FALSE;
        }
        obj.disabled_till_frame[idx as usize] = 0;
        obj.disabled_mask.set(idx, 0);
        let behaviors_to_edge: Vec<ModuleSlot> = if obj.is_disabled() == FALSE {
            obj.behaviors.iter().map(|&(ms, _)| ms).collect()
        } else {
            Vec::new()
        };
        for ms in behaviors_to_edge {
            if self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    m.on_disabled_edge(sim, ctx, FALSE);
                });
            }
        }
        TRUE
    }

    fn grant_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str, record_stats: Bool) {
        let Some((upgrade_type, upgrade_mask)) = self
            .rules
            .upgrade_center
            .find_upgrade(upgrade_name, &mut self.rules.name_keys)
            .map(|ut| (ut.upgrade_type(), ut.upgrade_mask().clone()))
        else {
            return;
        };
        let Some(team_id) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        let empty_mask = UpgradeMaskType::new();
        if upgrade_type == UpgradeType::Player {
            self.world
                .player_upgrades
                .entry(team_id)
                .or_default()
                .clear_and_set(&empty_mask, &upgrade_mask);
            let player_slots: Vec<ObjectSlot> = self
                .world
                .objects
                .world_iter()
                .filter(|&slot| {
                    self.world
                        .objects
                        .get(slot)
                        .is_some_and(|o| !o.is_destroyed() && o.team_id == team_id)
                })
                .collect();
            for pslot in player_slots {
                self.dispatch_attempt_upgrade(pslot, &upgrade_mask);
            }
        } else {
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.upgrades_completed
                    .clear_and_set(&empty_mask, &upgrade_mask);
            }
            self.dispatch_attempt_upgrade(owner, &upgrade_mask);
        }
        if record_stats != FALSE {
            self.world
                .upgrades_recorded
                .push((team_id, upgrade_name.to_owned()));
        }
    }

    fn lock_weapon_slot(
        &mut self,
        owner: ObjectSlot,
        weapon_slot: WeaponSlotType,
        lock_type: WeaponLockType,
    ) -> Bool {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return FALSE;
        };
        obj.weapon_set.set_weapon_lock(weapon_slot, lock_type)
    }

    fn set_veterancy_level(
        &mut self,
        owner: ObjectSlot,
        level: VeterancyLevel,
        min_only: Bool,
        provide_feedback: Bool,
    ) {
        let Some((tmpl_ov, obj_id, body_slot)) = self
            .world
            .objects
            .get(owner)
            .map(|o| (o.thing.template_override(), o.id, o.body_slot))
        else {
            return;
        };
        let tmpl = tmpl_ov
            .get_final(&self.rules.thing_factory)
            .cloned()
            .unwrap_or_default();
        let mut parent = ObjectExpParent {
            tmpl: &tmpl,
            killer_rel: Relationship::Neutral,
            level_change: None,
        };
        if let Some(obj) = self.world.objects.get_mut(owner) {
            if min_only != FALSE {
                obj.experience_tracker
                    .set_min_veterancy_level(&mut parent, level, provide_feedback);
            } else {
                obj.experience_tracker
                    .set_veterancy_level(&mut parent, level, provide_feedback);
            }
        }
        if let Some((old_lvl, new_lvl, fb)) = parent.level_change {
            if let Some(bs) = body_slot {
                let _ = self.dispatch_module(bs, |m, sim, ctx| {
                    if let Some(b) = m.as_body_mut() {
                        b.on_veterancy_level_changed(sim, ctx, old_lvl, new_lvl, fb);
                    }
                });
            }
            self.client
                .on_veterancy_changed(obj_id, old_lvl.as_i32(), new_lvl.as_i32());
        }
    }

    fn register_supply_center(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            if !self.world.registered_supply_centers.contains(&id) {
                self.world.registered_supply_centers.push(id);
            }
        }
    }

    fn register_supply_warehouse(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            if !self.world.registered_supply_warehouses.contains(&id) {
                self.world.registered_supply_warehouses.push(id);
            }
        }
    }

    fn on_special_power_creation(&mut self, owner: ObjectSlot) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::SPECIAL_POWER))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(sp) = m.as_special_power_mut() {
                    sp.on_special_power_creation(sim, ctx);
                }
            });
        }
    }

    fn notify_bone_fx_damage_state_change(
        &mut self,
        owner: ObjectSlot,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        let bone_fx_key = self.rules.name_keys.name_to_key("BoneFXUpdate");
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::UPDATE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let is_bfx = self
                .world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == bone_fx_key);
            if is_bfx {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    if let Some(upd) = m.as_update_mut() {
                        upd.change_body_damage_state(sim, ctx, old_state, new_state);
                    }
                });
            }
        }
    }

    fn set_model_condition_state(&mut self, owner: ObjectSlot, flag: ModelConditionFlagType) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.set_model_condition_state(flag);
        }
    }

    fn clear_model_condition_state(&mut self, owner: ObjectSlot, flag: ModelConditionFlagType) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.clear_model_condition_state(flag);
        }
    }

    fn clear_and_set_model_condition_flags(
        &mut self,
        owner: ObjectSlot,
        clear: &ModelConditionFlags,
        set: &ModelConditionFlags,
    ) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.clear_and_set_model_condition_flags(clear, set);
        }
    }

    fn set_special_model_condition_state(
        &mut self,
        owner: ObjectSlot,
        flag: ModelConditionFlagType,
        frames: UnsignedInt,
    ) {
        let dur = if frames == 0 { 1 } else { frames };
        let until = self.world.frame.saturating_add(dur);
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.clear_special_model_condition_states();
            obj.set_model_condition_state(flag);
            obj.special_model_condition_until = until;
        }
        self.trigger_smc_helper(owner, until);
    }

    fn clear_special_model_condition_states(&mut self, owner: ObjectSlot) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.clear_special_model_condition_states();
        }
    }

    fn set_status(&mut self, owner: ObjectSlot, status: ObjectStatusType) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.status.set(status.as_i32(), 1);
        }
        if status == ObjectStatusType::Repulsor {
            let until = self.world.frame.saturating_add(2 * 30);
            self.trigger_repulsor(owner, until);
        }
    }

    fn clear_status(&mut self, owner: ObjectSlot, status: ObjectStatusType) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.status.set(status.as_i32(), 0);
        }
    }

    fn set_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    ) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.set_weapon_bonus_condition(cond);
        }
    }

    fn clear_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    ) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.clear_weapon_bonus_condition(cond);
        }
    }

    fn spawn_ocl(
        &mut self,
        ocl_name: &str,
        creator: ObjectSlot,
        pos: &Coord3D,
        target_pos: &Coord3D,
    ) {
        let creator_id = self.world.objects.id_of(creator).unwrap_or(ObjectId::INVALID);
        self.world
            .spawned_ocls
            .push((ocl_name.to_owned(), creator_id, *pos, *target_pos));
    }

    fn trigger_fx_list(&mut self, fx_name: &str, pos: &Coord3D) {
        self.world
            .triggered_fx_lists
            .push((fx_name.to_owned(), *pos));
    }

    fn trigger_damage_fx(
        &mut self,
        damage_fx_name: &str,
        damage_type: DamageType,
        damage_amount: Real,
        source: ObjectId,
        victim: ObjectSlot,
    ) {
        let victim_id = self.world.objects.id_of(victim).unwrap_or(ObjectId::INVALID);
        self.world.triggered_damage_fx.push((
            damage_fx_name.to_owned(),
            damage_type,
            damage_amount,
            source,
            victim_id,
        ));
    }

    fn defect_object(
        &mut self,
        owner: ObjectSlot,
        new_team: TeamId,
        detection_frames: UnsignedInt,
        with_fx: Bool,
    ) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        if obj.contained_by.is_some()
            || obj.team_id == new_team
            || obj.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
            || obj.status.test(ObjectStatusType::Sold.as_i32()) != FALSE
        {
            return;
        }
        obj.set_undetected_defector(Bool::from(detection_frames > 0));
        obj.team_id = new_team;
        let ai_slot = obj.ai_slot;
        self.trigger_defection(owner, detection_frames, with_fx);
        if let Some(ai_ms) = ai_slot {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_goal_none(sim, ctx);
                }
            });
        }
    }

    fn trigger_smc_helper(&mut self, owner: ObjectSlot, when: UnsignedInt) {
        let Some(hs) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.smc_helper_slot)
        else {
            return;
        };
        let _ = self.dispatch_module(hs, |m, sim, ctx| {
            if let Some(h) = m.as_helper_mut() {
                h.sleep_until(sim, ctx, when);
            }
        });
    }

    fn trigger_status_damage(
        &mut self,
        owner: ObjectSlot,
        status: ObjectStatusType,
        duration: Real,
    ) {
        let Some(hs) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.status_damage_helper_slot)
        else {
            return;
        };
        let _ = self.dispatch_module(hs, |m, sim, ctx| {
            if let Some(h) = m.as_helper_mut() {
                h.do_status_damage(sim, ctx, status, duration);
            }
        });
    }

    fn trigger_subdual_damage(&mut self, owner: ObjectSlot, amount: Real) {
        self.notify_subdual_damage(owner, amount);
    }

    fn trigger_defection(
        &mut self,
        owner: ObjectSlot,
        detection_frames: UnsignedInt,
        with_fx: Bool,
    ) {
        let Some(hs) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.defection_helper_slot)
        else {
            return;
        };
        let _ = self.dispatch_module(hs, |m, sim, ctx| {
            if let Some(h) = m.as_helper_mut() {
                h.start_defection_timer(sim, ctx, detection_frames, with_fx);
            }
        });
    }

    fn trigger_temp_weapon_bonus(
        &mut self,
        owner: ObjectSlot,
        bonus_type: WeaponBonusConditionType,
        duration: UnsignedInt,
    ) {
        let Some(hs) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.temp_weapon_bonus_helper_slot)
        else {
            return;
        };
        let _ = self.dispatch_module(hs, |m, sim, ctx| {
            if let Some(h) = m.as_helper_mut() {
                h.do_temp_weapon_bonus(sim, ctx, bonus_type, duration);
            }
        });
    }

    fn trigger_weapon_status(&mut self, owner: ObjectSlot) {
        let now = self.world.frame;
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.adjust_model_condition_for_weapon_status(now);
        }
    }

    fn trigger_repulsor(&mut self, owner: ObjectSlot, until_frame: UnsignedInt) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.status.set(ObjectStatusType::Repulsor.as_i32(), 1);
        }
        let Some(hs) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.repulsor_helper_slot)
        else {
            return;
        };
        let _ = self.dispatch_module(hs, |m, sim, ctx| {
            if let Some(h) = m.as_helper_mut() {
                h.sleep_until(sim, ctx, until_frame);
            }
        });
    }

    fn trigger_slow_death(&mut self, owner: ObjectSlot, dmg: &DamageInfo) {
        let beh_slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::SLOW_DEATH))
                .map(|&(s, _)| s)
                .collect()
        });
        let mut total: Int = 0;
        let mut applicable = Vec::new();
        for &ms in &beh_slots {
            if let Some(sd) = self
                .world
                .modules
                .payload(ms)
                .and_then(|p| p.as_slow_death())
            {
                if sd.is_die_applicable(dmg) != FALSE {
                    let w = sd.get_probability_modifier(dmg);
                    total = total.wrapping_add(w);
                    applicable.push((ms, w));
                }
            }
        }
        if total <= 0 {
            return;
        }
        let mut roll = self.rng_logic_int(1, total);
        for (ms, w) in applicable {
            roll = roll.wrapping_sub(w);
            if roll <= 0 {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    if let Some(sd) = m.as_slow_death_mut() {
                        sd.begin_slow_death(sim, ctx, dmg);
                    }
                });
                return;
            }
        }
    }

    fn add_player_money(&mut self, team_id: TeamId, amount: Int) -> Int {
        let entry = self.world.player_money.entry(team_id).or_insert(0);
        let cur = *entry as i64;
        let next = (cur + amount as i64).max(0).min(u32::MAX as i64);
        *entry = next as UnsignedInt;
        (next - cur) as Int
    }

    fn withdraw_player_money(&mut self, team_id: TeamId, amount: UnsignedInt) -> UnsignedInt {
        let entry = self.world.player_money.entry(team_id).or_insert(0);
        let withdrawn = (*entry).min(amount);
        *entry -= withdrawn;
        withdrawn
    }

    fn grant_player_science(&mut self, team_id: TeamId, science: ScienceType) -> Bool {
        let inserted = self
            .world
            .player_sciences
            .entry(team_id)
            .or_default()
            .insert(science);
        Bool::from(inserted)
    }

    fn add_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real) {
        self.world
            .player_cost_modifiers
            .entry(team_id)
            .or_default()
            .push((kindof.clone(), percent));
    }

    fn remove_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real) {
        if let Some(list) = self.world.player_cost_modifiers.get_mut(&team_id) {
            if let Some(pos) = list
                .iter()
                .position(|(k, p)| k == kindof && p.to_bits() == percent.to_bits())
            {
                list.remove(pos);
            }
        }
    }

    fn remove_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str) {
        let Some((upgrade_key, upgrade_mask)) = self
            .rules
            .upgrade_center
            .find_upgrade(upgrade_name, &mut self.rules.name_keys)
            .map(|ut| (ut.upgrade_name_key(), ut.upgrade_mask().clone()))
        else {
            return;
        };
        if let Some(obj) = self.world.objects.get_mut(owner) {
            let empty_set = UpgradeMaskType::new();
            obj.upgrades_completed.clear_and_set(&upgrade_mask, &empty_set);
        } else {
            return;
        }
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::UPGRADE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            if self.world.modules.is_executing(ms) {
                let _ = self
                    .world
                    .modules
                    .push_deferred_op(ms, DeferredSelfOp::UpgradeResetKey(upgrade_key));
            } else {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    if let Some(u) = m.as_upgrade_mut() {
                        let _ = u.reset_upgrade(sim, ctx, &upgrade_mask);
                    }
                });
            }
        }
    }

    fn set_armor_set_flag(&mut self, owner: ObjectSlot, flag: ArmorSetType, set: Bool) {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(bs) {
            if let ModuleSidecar::Body {
                health,
                max_health,
                prev_health,
                initial_health,
                damage_scalar,
                damage_state,
                front_crushed,
                back_crushed,
                indestructible,
                current_subdual_damage,
                subdued,
                last_damage_ts,
                last_healing_ts,
                mut armor_set_bits,
            } = self.world.modules.sidecar(bs)
            {
                let bit = flag.as_i32();
                if (0..8).contains(&bit) {
                    if set != FALSE {
                        armor_set_bits |= 1u8 << bit;
                    } else {
                        armor_set_bits &= !(1u8 << bit);
                    }
                }
                self.world.modules.set_sidecar(
                    bs,
                    ModuleSidecar::Body {
                        health,
                        max_health,
                        prev_health,
                        initial_health,
                        damage_scalar,
                        damage_state,
                        front_crushed,
                        back_crushed,
                        indestructible,
                        current_subdual_damage,
                        subdued,
                        last_damage_ts,
                        last_healing_ts,
                        armor_set_bits,
                    },
                );
            }
            let _ = self
                .world
                .modules
                .push_deferred_op(bs, DeferredSelfOp::SetArmorSetFlag { flag, set });
            return;
        }
        let _ = self.dispatch_module(bs, |m, _, _| {
            if let Some(b) = m.as_body_mut() {
                if set != FALSE {
                    b.set_armor_set_flag(flag);
                } else {
                    b.clear_armor_set_flag(flag);
                }
            }
        });
    }

    fn set_weapon_set_flag(&mut self, owner: ObjectSlot, flag: WeaponSetType, set: Bool) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.weapon_set_flags.set(flag.as_i32(), i32::from(set != FALSE));
        }
    }

    fn set_max_health(
        &mut self,
        owner: ObjectSlot,
        new_max: Real,
        change_type: MaxHealthChangeType,
    ) {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(bs) {
            if let ModuleSidecar::Body {
                mut health,
                max_health: prev_max,
                prev_health: _,
                initial_health: _,
                damage_scalar,
                damage_state,
                front_crushed,
                back_crushed,
                indestructible,
                current_subdual_damage,
                subdued,
                last_damage_ts,
                last_healing_ts,
                armor_set_bits,
            } = self.world.modules.sidecar(bs)
            {
                let prev_health = health;
                match change_type {
                    MaxHealthChangeType::PreserveRatio => {
                        let ratio = div_safe_f32(health, prev_max);
                        health = new_max * ratio;
                    }
                    MaxHealthChangeType::AddCurrentHealthToo => {
                        health += new_max - prev_max;
                    }
                    MaxHealthChangeType::SameCurrentHealth => {}
                    MaxHealthChangeType::FullyHeal => {
                        health = new_max;
                    }
                }
                if health > new_max {
                    health = new_max;
                }
                if health < 0.0 {
                    health = 0.0;
                }
                self.world.modules.set_sidecar(
                    bs,
                    ModuleSidecar::Body {
                        health,
                        max_health: new_max,
                        prev_health,
                        initial_health: new_max,
                        damage_scalar,
                        damage_state,
                        front_crushed,
                        back_crushed,
                        indestructible,
                        current_subdual_damage,
                        subdued,
                        last_damage_ts,
                        last_healing_ts,
                        armor_set_bits,
                    },
                );
            }
            let _ = self.world.modules.push_deferred_op(
                bs,
                DeferredSelfOp::SetMaxHealth {
                    new_max,
                    change_type: change_type.as_i32() as u8,
                },
            );
            return;
        }
        let _ = self.dispatch_module(bs, |m, sim, ctx| {
            if let Some(b) = m.as_body_mut() {
                b.set_max_health(sim, ctx, new_max, change_type);
            }
        });
    }

    fn set_experience_scalar(&mut self, owner: ObjectSlot, scalar: Real) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.experience_tracker.experience_scalar = scalar;
        }
    }

    fn add_experience_scalar(&mut self, owner: ObjectSlot, delta: Real) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.experience_tracker.experience_scalar += delta;
        }
    }

    fn set_command_set(&mut self, owner: ObjectSlot, command_set: &str) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.command_set_override = command_set.to_owned();
        }
    }

    fn set_locomotor_set(&mut self, owner: ObjectSlot, set_type: LocomotorSetType) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.cur_locomotor_set = set_type;
        }
    }

    fn set_shroud_clearing_range(&mut self, owner: ObjectSlot, range: Real) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.shroud_clearing_range_override = range;
        }
    }

    fn grant_stealth(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.stealth_granted.insert(id);
        }
        self.set_status(owner, ObjectStatusType::CanStealth);
    }

    fn set_passengers_allowed_to_fire(&mut self, owner: ObjectSlot, allowed: Bool) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.passengers_allowed_to_fire = allowed;
        }
    }

    fn add_radar(&mut self, team_id: TeamId, disable_proof: Bool) {
        let entry = self.world.player_radars.entry(team_id).or_insert((0, 0));
        entry.0 += 1;
        if disable_proof != FALSE {
            entry.1 += 1;
        }
    }

    fn remove_radar(&mut self, team_id: TeamId, disable_proof: Bool) {
        let entry = self.world.player_radars.entry(team_id).or_insert((0, 0));
        entry.0 = (entry.0 - 1).max(0);
        if disable_proof != FALSE {
            entry.1 = (entry.1 - 1).max(0);
        }
    }

    fn disable_radar(&mut self, team_id: TeamId, frames: UnsignedInt) {
        let until = self.world.frame.saturating_add(frames);
        self.world.player_radar_disabled_until.insert(team_id, until);
    }

    fn extend_power_plant(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.power_plant_extended.insert(id);
        }
    }

    fn sabotage_power(&mut self, team_id: TeamId, frames: UnsignedInt) {
        let until = self.world.frame.saturating_add(frames);
        self.world
            .player_power_sabotaged_until
            .insert(team_id, until);
    }

    fn pause_special_power(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        pause: Bool,
    ) {
        let Some(ms) = self.find_special_power_module(owner, template_id) else {
            return;
        };
        if self.world.modules.is_executing(ms) {
            let _ = self
                .world
                .modules
                .push_deferred_op(ms, DeferredSelfOp::SpecialPowerPause { pause });
        } else {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(sp) = m.as_special_power_mut() {
                    sp.pause_countdown(sim, ctx, pause);
                }
            });
        }
    }

    fn reset_special_power_timer(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::SPECIAL_POWER))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let match_ok = if template_id == 0 {
                true
            } else if let Some(p) = self.world.modules.payload(ms) {
                let ctx = self.world.modules.meta(ms).context(ms);
                p.as_special_power()
                    .is_some_and(|sp| sp.is_module_for_power(self, ctx, template_id) != FALSE)
            } else {
                matches!(
                    self.world.modules.sidecar(ms),
                    ModuleSidecar::SpecialPower { template_id: tid, .. } if tid == template_id
                )
            };
            if match_ok {
                if self.world.modules.is_executing(ms) {
                    let _ = self.world.modules.push_deferred_op(
                        ms,
                        DeferredSelfOp::SpecialPowerSetReadyFrame(u32::MAX),
                    );
                } else {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        if let Some(sp) = m.as_special_power_mut() {
                            sp.start_power_recharge(sim, ctx);
                        }
                    });
                }
            }
        }
    }

    fn express_shared_special_power_ready_frame(
        &mut self,
        team_id: TeamId,
        template_id: UnsignedInt,
        frame: UnsignedInt,
    ) {
        self.world
            .shared_special_power_ready_frames
            .insert((team_id, template_id), frame);
    }

    fn reset_shared_special_power_ready_frame(
        &mut self,
        team_id: TeamId,
        template_id: UnsignedInt,
    ) -> UnsignedInt {
        let reload = self
            .rules
            .special_power_store
            .find_special_power_template_by_id(template_id)
            .map_or(0, |t| t.reload_time());
        let ready = self.world.frame.saturating_add(reload);
        self.world
            .shared_special_power_ready_frames
            .insert((team_id, template_id), ready);
        ready
    }

    fn notify_special_power_triggered(
        &mut self,
        team_id: TeamId,
        power_name: &str,
        obj_id: ObjectId,
    ) {
        self.world
            .special_power_triggered_events
            .push((team_id, power_name.to_owned(), obj_id));
    }

    fn notify_special_power_complete(&mut self, obj_id: ObjectId, sp_type: SpecialPowerType) {
        self.world
            .special_power_completed_events
            .push((obj_id, sp_type));
    }

    fn update_public_timer(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        ready_frame: UnsignedInt,
        add: Bool,
    ) {
        if let Some(obj_id) = self.world.objects.id_of(owner) {
            self.world
                .public_timer_events
                .push((obj_id, template_id, ready_frame, add));
        }
    }

    fn reveal_shroud(&mut self, team_id: TeamId, pos: &Coord3D, range: Real, duration: UnsignedInt) {
        self.world
            .shroud_reveals
            .push((team_id, *pos, range, duration));
    }

    fn remove_shroud(&mut self, team_id: TeamId) {
        self.world.shroud_removals.push(team_id);
    }

    fn reveal_map_for_player(&mut self, team_id: TeamId) {
        self.world.map_reveals.push(team_id);
    }

    fn fire_weapon_template(
        &mut self,
        owner: ObjectSlot,
        weapon_name: &str,
        target: Option<ObjectSlot>,
        target_pos: &Coord3D,
    ) {
        let owner_id = self.world.objects.id_of(owner).unwrap_or(ObjectId::INVALID);
        let target_id = target.and_then(|s| self.world.objects.id_of(s));
        self.world
            .weapon_fires
            .push((owner_id, weapon_name.to_owned(), target_id, *target_pos));
    }

    fn create_crate_at(
        &mut self,
        crate_tmpl_name: &str,
        killer: Option<ObjectSlot>,
        victim: ObjectSlot,
    ) {
        let victim_id = self.world.objects.id_of(victim).unwrap_or(ObjectId::INVALID);
        let killer_id = killer.and_then(|s| self.world.objects.id_of(s));
        self.world
            .crate_spawns
            .push((crate_tmpl_name.to_owned(), victim_id, killer_id));
    }

    fn replace_object(&mut self, owner: ObjectSlot, new_tmpl_name: &str) -> Option<ObjectSlot> {
        let new_tmpl_id = self.rules.thing_factory.find_template_id(new_tmpl_name)?;
        let (team_id, mtx, pos, angle) = self.world.objects.get(owner).map(|o| {
            (
                o.team_id,
                *o.thing.get_transform_matrix(),
                *o.thing.get_position(),
                o.thing.get_orientation(),
            )
        })?;
        self.destroy_object_now(owner);
        let new_slot = self.spawn_object(new_tmpl_id, team_id, pos, angle);
        if let Some(new_obj) = self.world.objects.get_mut(new_slot) {
            let mut dummy = crate::object::thing::NoopTransformReactor;
            new_obj.thing.set_transform_matrix(&mtx, &mut dummy);
        }
        self.dispatch_on_build_complete(new_slot);
        Some(new_slot)
    }

    fn convert_to_car_bomb(&mut self, target: ObjectSlot, crate_owner: ObjectSlot) {
        let Some((new_team, owner_id)) = self
            .world
            .objects
            .get(crate_owner)
            .map(|o| (o.team_id, o.id))
        else {
            return;
        };
        let target_id = self.world.objects.id_of(target).unwrap_or(ObjectId::INVALID);
        self.set_weapon_set_flag(target, WeaponSetType::Carbomb, TRUE);
        self.set_status(target, ObjectStatusType::IsCarbomb);
        self.defect_object(target, new_team, 0, FALSE);
        self.world
            .car_bombed_objects
            .push((target_id, owner_id));
    }

    fn hijack_vehicle(&mut self, target: ObjectSlot, hijacker: ObjectSlot) {
        let Some((new_team, hijacker_id)) = self
            .world
            .objects
            .get(hijacker)
            .map(|o| (o.team_id, o.id))
        else {
            return;
        };
        let target_id = self.world.objects.id_of(target).unwrap_or(ObjectId::INVALID);
        self.set_weapon_set_flag(target, WeaponSetType::VehicleHijack, TRUE);
        self.set_status(target, ObjectStatusType::Hijacked);
        self.defect_object(target, new_team, 0, FALSE);
        self.world.hijacked_objects.push((target_id, hijacker_id));
    }

    fn apply_cash_bounty_percent(&mut self, team_id: TeamId, percent: Real) {
        self.world.player_bounty_percent.insert(team_id, percent);
    }

    fn trigger_dam_break(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.dam_break_events.push(id);
        }
    }

    fn notify_rebuild_hole_exposed(
        &mut self,
        dead_obj: ObjectSlot,
        hole_obj: ObjectSlot,
        hole_max_health: Real,
    ) {
        let dead_id = self.world.objects.id_of(dead_obj).unwrap_or(ObjectId::INVALID);
        let hole_id = self.world.objects.id_of(hole_obj).unwrap_or(ObjectId::INVALID);
        self.world
            .rebuild_hole_events
            .push((dead_id, hole_id, hole_max_health));
    }

    fn trigger_special_ability(
        &mut self,
        owner: ObjectSlot,
        target: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
    ) {
        let owner_id = self.world.objects.id_of(owner).unwrap_or(ObjectId::INVALID);
        let target_id = target.and_then(|s| self.world.objects.id_of(s));
        self.world
            .special_ability_events
            .push((owner_id, target_id, loc.copied()));
    }

    fn trigger_cleanup_area(&mut self, owner: ObjectSlot, loc: &Coord3D) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.cleanup_area_events.push((id, *loc));
        }
    }

    fn trigger_baikonur_launch(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.baikonur_launch_events.push(id);
        }
    }

    fn update_sub_objects(&mut self, owner: ObjectSlot, shown: &[String], hidden: &[String]) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            for s in shown {
                obj.sub_objects_hidden.remove(s);
                obj.sub_objects_shown.insert(s.clone());
            }
            for h in hidden {
                obj.sub_objects_shown.remove(h);
                obj.sub_objects_hidden.insert(h.clone());
            }
        }
    }

    fn dispatch_on_collide(
        &mut self,
        owner: ObjectSlot,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    ) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::COLLIDE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(c) = m.as_collide_mut() {
                    c.on_collide(sim, ctx, other, loc, normal);
                }
            });
        }
    }

    fn dispatch_attempt_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::UPGRADE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(u) = m.as_upgrade_mut() {
                    let _ = u.attempt_upgrade(sim, ctx, mask);
                }
            });
        }
    }

    fn dispatch_reset_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::UPGRADE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(u) = m.as_upgrade_mut() {
                    let _ = u.reset_upgrade(sim, ctx, mask);
                }
            });
        }
    }

    fn client(&mut self) -> &mut dyn ClientSink {
        self.client.as_mut()
    }
}

/// Lightweight [`SimApi`] wrapper used in standalone Gate V2 module tests.
pub struct MockSimApi {
    pub ctx: SimContext,
}

impl Default for MockSimApi {
    fn default() -> Self {
        Self::new()
    }
}

impl MockSimApi {
    pub fn new() -> Self {
        Self {
            ctx: SimContext::new(),
        }
    }
}

impl SimView for MockSimApi {
    fn frame(&self) -> UnsignedInt {
        self.ctx.frame()
    }

    fn rules(&self) -> &RulesRegistry {
        self.ctx.rules()
    }

    fn module_data(&self, id: ModuleDataId) -> Option<&dyn ModuleData> {
        self.ctx.module_data(id)
    }

    fn find_object(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.ctx.find_object(id)
    }

    fn object(&self, slot: ObjectSlot) -> Option<&ObjectRecord> {
        self.ctx.object(slot)
    }

    fn module_sidecar(&self, slot: ModuleSlot) -> ModuleSidecar {
        self.ctx.module_sidecar(slot)
    }

    fn terrain(&self) -> &dyn ThingTerrainAccess {
        self.ctx.terrain()
    }

    fn get_wake_frame(&self, slot: ModuleSlot) -> UpdateSleepTime {
        self.ctx.get_wake_frame(slot)
    }

    fn module_schedule(&self, slot: ModuleSlot) -> Option<crate::scheduler::UpdateModuleScheduleState> {
        self.ctx.module_schedule(slot)
    }

    fn get_subdual_damage_heal_rate(&self, owner: ObjectSlot) -> UnsignedInt {
        self.ctx.get_subdual_damage_heal_rate(owner)
    }

    fn get_subdual_damage_heal_amount(&self, owner: ObjectSlot) -> Real {
        self.ctx.get_subdual_damage_heal_amount(owner)
    }

    fn has_any_subdual_damage(&self, owner: ObjectSlot) -> Bool {
        self.ctx.has_any_subdual_damage(owner)
    }

    fn player_relationship(&self, a: ObjectSlot, b: ObjectSlot) -> Relationship {
        self.ctx.player_relationship(a, b)
    }

    fn player_has_science(&self, team_id: TeamId, science: ScienceType) -> Bool {
        self.ctx.player_has_science(team_id, science)
    }

    fn player_is_preorder(&self, team_id: TeamId) -> Bool {
        self.ctx.player_is_preorder(team_id)
    }

    fn player_has_upgrade(&self, team_id: TeamId, mask: &UpgradeMaskType) -> Bool {
        self.ctx.player_has_upgrade(team_id, mask)
    }

    fn adjust_damage_by_armor(
        &self,
        target: ObjectSlot,
        armor_set_flags: &ArmorSetFlags,
        damage_type: DamageType,
        amount: Real,
    ) -> Real {
        self.ctx
            .adjust_damage_by_armor(target, armor_set_flags, damage_type, amount)
    }

    fn ai_enable_repulsors(&self) -> Bool {
        self.ctx.ai_enable_repulsors()
    }

    fn has_module_by_name_key(&self, owner: ObjectSlot, mod_name_key: NameKeyType) -> bool {
        self.ctx.has_module_by_name_key(owner, mod_name_key)
    }

    fn player_money(&self, team_id: TeamId) -> UnsignedInt {
        self.ctx.player_money(team_id)
    }

    fn player_is_human(&self, team_id: TeamId) -> Bool {
        self.ctx.player_is_human(team_id)
    }

    fn is_neutral_controlled(&self, owner: ObjectSlot) -> Bool {
        self.ctx.is_neutral_controlled(owner)
    }

    fn object_is_kind_of(&self, owner: ObjectSlot, kindof: KindOfType) -> Bool {
        self.ctx.object_is_kind_of(owner, kindof)
    }

    fn object_is_kind_of_multi(
        &self,
        owner: ObjectSlot,
        must_be_set: &KindOfMaskType,
        must_be_clear: &KindOfMaskType,
    ) -> Bool {
        self.ctx
            .object_is_kind_of_multi(owner, must_be_set, must_be_clear)
    }

    fn object_veterancy_level(&self, owner: ObjectSlot) -> VeterancyLevel {
        self.ctx.object_veterancy_level(owner)
    }

    fn object_health_ratio(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_health_ratio(owner)
    }

    fn object_max_health(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_max_health(owner)
    }

    fn object_has_upgrade(&self, owner: ObjectSlot, mask: &UpgradeMaskType) -> Bool {
        self.ctx.object_has_upgrade(owner, mask)
    }

    fn object_can_crush(&self, crusher: ObjectSlot, victim: ObjectSlot) -> Bool {
        self.ctx.object_can_crush(crusher, victim)
    }

    fn object_is_above_terrain(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_is_above_terrain(owner)
    }

    fn object_has_ai(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_has_ai(owner)
    }

    fn object_ai_goal_object(&self, owner: ObjectSlot) -> Option<ObjectSlot> {
        self.ctx.object_ai_goal_object(owner)
    }

    fn find_objects_in_range(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot> {
        self.ctx.find_objects_in_range(center, radius)
    }

    fn find_objects_in_range_bounding(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot> {
        self.ctx.find_objects_in_range_bounding(center, radius)
    }

    fn find_special_power_module(
        &self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
    ) -> Option<ModuleSlot> {
        self.ctx.find_special_power_module(owner, template_id)
    }

    fn shared_special_power_ready_frame(
        &self,
        team_id: TeamId,
        template_id: UnsignedInt,
    ) -> UnsignedInt {
        self.ctx
            .shared_special_power_ready_frame(team_id, template_id)
    }

    fn player_captured_object_count(&self, team_id: TeamId) -> Int {
        self.ctx.player_captured_object_count(team_id)
    }

    fn special_power_uses_delay(&self) -> Bool {
        self.ctx.special_power_uses_delay()
    }
}

impl SimApi for MockSimApi {
    fn rng_logic_int(&mut self, lo: Int, hi: Int) -> Int {
        self.ctx.rng_logic_int(lo, hi)
    }

    fn rng_logic_real(&mut self, lo: Real, hi: Real) -> Real {
        self.ctx.rng_logic_real(lo, hi)
    }

    fn object_mut(&mut self, slot: ObjectSlot) -> Option<&mut ObjectRecord> {
        self.ctx.object_mut(slot)
    }

    fn set_module_sidecar(&mut self, slot: ModuleSlot, sidecar: ModuleSidecar) {
        self.ctx.set_module_sidecar(slot, sidecar);
    }

    fn awaken_module(&mut self, slot: ModuleSlot, when: UnsignedInt) -> bool {
        self.ctx.awaken_module(slot, when)
    }

    fn set_wake_frame(&mut self, slot: ModuleSlot, sleep: UpdateSleepTime) {
        self.ctx.set_wake_frame(slot, sleep);
    }

    fn set_module_schedule(
        &mut self,
        slot: ModuleSlot,
        schedule: crate::scheduler::UpdateModuleScheduleState,
    ) {
        self.ctx.set_module_schedule(slot, schedule);
    }

    fn spawn_object(
        &mut self,
        tmpl: ThingTemplateId,
        team: TeamId,
        pos: Coord3D,
        angle: Real,
    ) -> ObjectSlot {
        self.ctx.spawn_object(tmpl, team, pos, angle)
    }

    fn destroy_object(&mut self, slot: ObjectSlot) {
        self.ctx.destroy_object(slot);
    }

    fn attempt_damage(&mut self, target: ObjectSlot, dmg: &mut DamageInfo) {
        self.ctx.attempt_damage(target, dmg);
    }

    fn attempt_healing(&mut self, target: ObjectSlot, healing: &mut DamageInfo) {
        self.ctx.attempt_healing(target, healing);
    }

    fn attempt_healing_from_sole_benefactor(
        &mut self,
        target: ObjectSlot,
        amount: Real,
        source: ObjectId,
        duration: UnsignedInt,
    ) -> Bool {
        self.ctx
            .attempt_healing_from_sole_benefactor(target, amount, source, duration)
    }

    fn dispatch_on_damage(&mut self, owner: ObjectSlot, dmg: &mut DamageInfo) {
        self.ctx.dispatch_on_damage(owner, dmg);
    }

    fn dispatch_on_healing(&mut self, owner: ObjectSlot, dmg: &mut DamageInfo) {
        self.ctx.dispatch_on_healing(owner, dmg);
    }

    fn dispatch_on_body_damage_state_change(
        &mut self,
        owner: ObjectSlot,
        dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        self.ctx
            .dispatch_on_body_damage_state_change(owner, dmg, old_state, new_state);
    }

    fn dispatch_on_die(&mut self, owner: ObjectSlot, dmg: &DamageInfo) {
        self.ctx.dispatch_on_die(owner, dmg);
    }

    fn dispatch_on_build_complete(&mut self, owner: ObjectSlot) {
        self.ctx.dispatch_on_build_complete(owner);
    }

    fn notify_object_killed(&mut self, victim: ObjectSlot, killer: ObjectId, dmg: &DamageInfo) {
        self.ctx.notify_object_killed(victim, killer, dmg);
    }

    fn record_attacked_by(&mut self, victim: ObjectSlot, attacker: ObjectId) {
        self.ctx.record_attacked_by(victim, attacker);
    }

    fn notify_subdual_damage(&mut self, owner: ObjectSlot, amount: Real) {
        self.ctx.notify_subdual_damage(owner, amount);
    }

    fn set_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType) {
        self.ctx.set_disabled(owner, disabled_type);
    }

    fn set_disabled_until(
        &mut self,
        owner: ObjectSlot,
        disabled_type: DisabledType,
        till_frame: UnsignedInt,
    ) {
        self.ctx.set_disabled_until(owner, disabled_type, till_frame);
    }

    fn clear_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType) -> Bool {
        self.ctx.clear_disabled(owner, disabled_type)
    }

    fn grant_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str, record_stats: Bool) {
        self.ctx.grant_upgrade(owner, upgrade_name, record_stats);
    }

    fn lock_weapon_slot(
        &mut self,
        owner: ObjectSlot,
        weapon_slot: WeaponSlotType,
        lock_type: WeaponLockType,
    ) -> Bool {
        self.ctx.lock_weapon_slot(owner, weapon_slot, lock_type)
    }

    fn set_veterancy_level(
        &mut self,
        owner: ObjectSlot,
        level: VeterancyLevel,
        min_only: Bool,
        provide_feedback: Bool,
    ) {
        self.ctx
            .set_veterancy_level(owner, level, min_only, provide_feedback);
    }

    fn register_supply_center(&mut self, owner: ObjectSlot) {
        self.ctx.register_supply_center(owner);
    }

    fn register_supply_warehouse(&mut self, owner: ObjectSlot) {
        self.ctx.register_supply_warehouse(owner);
    }

    fn on_special_power_creation(&mut self, owner: ObjectSlot) {
        self.ctx.on_special_power_creation(owner);
    }

    fn notify_bone_fx_damage_state_change(
        &mut self,
        owner: ObjectSlot,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        self.ctx
            .notify_bone_fx_damage_state_change(owner, old_state, new_state);
    }

    fn set_model_condition_state(&mut self, owner: ObjectSlot, flag: ModelConditionFlagType) {
        self.ctx.set_model_condition_state(owner, flag);
    }

    fn clear_model_condition_state(&mut self, owner: ObjectSlot, flag: ModelConditionFlagType) {
        self.ctx.clear_model_condition_state(owner, flag);
    }

    fn clear_and_set_model_condition_flags(
        &mut self,
        owner: ObjectSlot,
        clear: &ModelConditionFlags,
        set: &ModelConditionFlags,
    ) {
        self.ctx.clear_and_set_model_condition_flags(owner, clear, set);
    }

    fn set_special_model_condition_state(
        &mut self,
        owner: ObjectSlot,
        flag: ModelConditionFlagType,
        frames: UnsignedInt,
    ) {
        self.ctx
            .set_special_model_condition_state(owner, flag, frames);
    }

    fn clear_special_model_condition_states(&mut self, owner: ObjectSlot) {
        self.ctx.clear_special_model_condition_states(owner);
    }

    fn set_status(&mut self, owner: ObjectSlot, status: ObjectStatusType) {
        self.ctx.set_status(owner, status);
    }

    fn clear_status(&mut self, owner: ObjectSlot, status: ObjectStatusType) {
        self.ctx.clear_status(owner, status);
    }

    fn set_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    ) {
        self.ctx.set_weapon_bonus_condition(owner, cond);
    }

    fn clear_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    ) {
        self.ctx.clear_weapon_bonus_condition(owner, cond);
    }

    fn spawn_ocl(
        &mut self,
        ocl_name: &str,
        creator: ObjectSlot,
        pos: &Coord3D,
        target_pos: &Coord3D,
    ) {
        self.ctx.spawn_ocl(ocl_name, creator, pos, target_pos);
    }

    fn trigger_fx_list(&mut self, fx_name: &str, pos: &Coord3D) {
        self.ctx.trigger_fx_list(fx_name, pos);
    }

    fn trigger_damage_fx(
        &mut self,
        damage_fx_name: &str,
        damage_type: DamageType,
        damage_amount: Real,
        source: ObjectId,
        victim: ObjectSlot,
    ) {
        self.ctx
            .trigger_damage_fx(damage_fx_name, damage_type, damage_amount, source, victim);
    }

    fn defect_object(
        &mut self,
        owner: ObjectSlot,
        new_team: TeamId,
        detection_frames: UnsignedInt,
        with_fx: Bool,
    ) {
        self.ctx
            .defect_object(owner, new_team, detection_frames, with_fx);
    }

    fn trigger_smc_helper(&mut self, owner: ObjectSlot, duration: UnsignedInt) {
        self.ctx.trigger_smc_helper(owner, duration);
    }

    fn trigger_status_damage(
        &mut self,
        owner: ObjectSlot,
        status: ObjectStatusType,
        duration: Real,
    ) {
        self.ctx.trigger_status_damage(owner, status, duration);
    }

    fn trigger_subdual_damage(&mut self, owner: ObjectSlot, amount: Real) {
        self.ctx.trigger_subdual_damage(owner, amount);
    }

    fn trigger_defection(
        &mut self,
        owner: ObjectSlot,
        detection_frames: UnsignedInt,
        with_fx: Bool,
    ) {
        self.ctx.trigger_defection(owner, detection_frames, with_fx);
    }

    fn trigger_temp_weapon_bonus(
        &mut self,
        owner: ObjectSlot,
        bonus_type: WeaponBonusConditionType,
        duration: UnsignedInt,
    ) {
        self.ctx
            .trigger_temp_weapon_bonus(owner, bonus_type, duration);
    }

    fn trigger_weapon_status(&mut self, owner: ObjectSlot) {
        self.ctx.trigger_weapon_status(owner);
    }

    fn trigger_repulsor(&mut self, owner: ObjectSlot, until_frame: UnsignedInt) {
        self.ctx.trigger_repulsor(owner, until_frame);
    }

    fn trigger_slow_death(&mut self, owner: ObjectSlot, dmg: &DamageInfo) {
        self.ctx.trigger_slow_death(owner, dmg);
    }

    fn add_player_money(&mut self, team_id: TeamId, amount: Int) -> Int {
        self.ctx.add_player_money(team_id, amount)
    }

    fn withdraw_player_money(&mut self, team_id: TeamId, amount: UnsignedInt) -> UnsignedInt {
        self.ctx.withdraw_player_money(team_id, amount)
    }

    fn grant_player_science(&mut self, team_id: TeamId, science: ScienceType) -> Bool {
        self.ctx.grant_player_science(team_id, science)
    }

    fn add_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real) {
        self.ctx.add_cost_modifier(team_id, kindof, percent);
    }

    fn remove_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real) {
        self.ctx.remove_cost_modifier(team_id, kindof, percent);
    }

    fn remove_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str) {
        self.ctx.remove_upgrade(owner, upgrade_name);
    }

    fn set_armor_set_flag(&mut self, owner: ObjectSlot, flag: ArmorSetType, set: Bool) {
        self.ctx.set_armor_set_flag(owner, flag, set);
    }

    fn set_weapon_set_flag(&mut self, owner: ObjectSlot, flag: WeaponSetType, set: Bool) {
        self.ctx.set_weapon_set_flag(owner, flag, set);
    }

    fn set_max_health(
        &mut self,
        owner: ObjectSlot,
        new_max: Real,
        change_type: MaxHealthChangeType,
    ) {
        self.ctx.set_max_health(owner, new_max, change_type);
    }

    fn set_experience_scalar(&mut self, owner: ObjectSlot, scalar: Real) {
        self.ctx.set_experience_scalar(owner, scalar);
    }

    fn add_experience_scalar(&mut self, owner: ObjectSlot, delta: Real) {
        self.ctx.add_experience_scalar(owner, delta);
    }

    fn set_command_set(&mut self, owner: ObjectSlot, command_set: &str) {
        self.ctx.set_command_set(owner, command_set);
    }

    fn set_locomotor_set(&mut self, owner: ObjectSlot, set_type: LocomotorSetType) {
        self.ctx.set_locomotor_set(owner, set_type);
    }

    fn set_shroud_clearing_range(&mut self, owner: ObjectSlot, range: Real) {
        self.ctx.set_shroud_clearing_range(owner, range);
    }

    fn grant_stealth(&mut self, owner: ObjectSlot) {
        self.ctx.grant_stealth(owner);
    }

    fn set_passengers_allowed_to_fire(&mut self, owner: ObjectSlot, allowed: Bool) {
        self.ctx.set_passengers_allowed_to_fire(owner, allowed);
    }

    fn add_radar(&mut self, team_id: TeamId, disable_proof: Bool) {
        self.ctx.add_radar(team_id, disable_proof);
    }

    fn remove_radar(&mut self, team_id: TeamId, disable_proof: Bool) {
        self.ctx.remove_radar(team_id, disable_proof);
    }

    fn disable_radar(&mut self, team_id: TeamId, frames: UnsignedInt) {
        self.ctx.disable_radar(team_id, frames);
    }

    fn extend_power_plant(&mut self, owner: ObjectSlot) {
        self.ctx.extend_power_plant(owner);
    }

    fn sabotage_power(&mut self, team_id: TeamId, frames: UnsignedInt) {
        self.ctx.sabotage_power(team_id, frames);
    }

    fn pause_special_power(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        pause: Bool,
    ) {
        self.ctx.pause_special_power(owner, template_id, pause);
    }

    fn reset_special_power_timer(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        self.ctx.reset_special_power_timer(owner, template_id);
    }

    fn express_shared_special_power_ready_frame(
        &mut self,
        team_id: TeamId,
        template_id: UnsignedInt,
        frame: UnsignedInt,
    ) {
        self.ctx
            .express_shared_special_power_ready_frame(team_id, template_id, frame);
    }

    fn reset_shared_special_power_ready_frame(
        &mut self,
        team_id: TeamId,
        template_id: UnsignedInt,
    ) -> UnsignedInt {
        self.ctx
            .reset_shared_special_power_ready_frame(team_id, template_id)
    }

    fn notify_special_power_triggered(
        &mut self,
        team_id: TeamId,
        power_name: &str,
        obj_id: ObjectId,
    ) {
        self.ctx
            .notify_special_power_triggered(team_id, power_name, obj_id);
    }

    fn notify_special_power_complete(&mut self, obj_id: ObjectId, sp_type: SpecialPowerType) {
        self.ctx.notify_special_power_complete(obj_id, sp_type);
    }

    fn update_public_timer(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        ready_frame: UnsignedInt,
        add: Bool,
    ) {
        self.ctx
            .update_public_timer(owner, template_id, ready_frame, add);
    }

    fn reveal_shroud(&mut self, team_id: TeamId, pos: &Coord3D, range: Real, duration: UnsignedInt) {
        self.ctx.reveal_shroud(team_id, pos, range, duration);
    }

    fn remove_shroud(&mut self, team_id: TeamId) {
        self.ctx.remove_shroud(team_id);
    }

    fn reveal_map_for_player(&mut self, team_id: TeamId) {
        self.ctx.reveal_map_for_player(team_id);
    }

    fn fire_weapon_template(
        &mut self,
        owner: ObjectSlot,
        weapon_name: &str,
        target: Option<ObjectSlot>,
        target_pos: &Coord3D,
    ) {
        self.ctx
            .fire_weapon_template(owner, weapon_name, target, target_pos);
    }

    fn create_crate_at(
        &mut self,
        crate_tmpl_name: &str,
        killer: Option<ObjectSlot>,
        victim: ObjectSlot,
    ) {
        self.ctx.create_crate_at(crate_tmpl_name, killer, victim);
    }

    fn replace_object(&mut self, owner: ObjectSlot, new_tmpl_name: &str) -> Option<ObjectSlot> {
        self.ctx.replace_object(owner, new_tmpl_name)
    }

    fn convert_to_car_bomb(&mut self, target: ObjectSlot, crate_owner: ObjectSlot) {
        self.ctx.convert_to_car_bomb(target, crate_owner);
    }

    fn hijack_vehicle(&mut self, target: ObjectSlot, hijacker: ObjectSlot) {
        self.ctx.hijack_vehicle(target, hijacker);
    }

    fn apply_cash_bounty_percent(&mut self, team_id: TeamId, percent: Real) {
        self.ctx.apply_cash_bounty_percent(team_id, percent);
    }

    fn trigger_dam_break(&mut self, owner: ObjectSlot) {
        self.ctx.trigger_dam_break(owner);
    }

    fn notify_rebuild_hole_exposed(
        &mut self,
        dead_obj: ObjectSlot,
        hole_obj: ObjectSlot,
        hole_max_health: Real,
    ) {
        self.ctx
            .notify_rebuild_hole_exposed(dead_obj, hole_obj, hole_max_health);
    }

    fn trigger_special_ability(
        &mut self,
        owner: ObjectSlot,
        target: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
    ) {
        self.ctx.trigger_special_ability(owner, target, loc);
    }

    fn trigger_cleanup_area(&mut self, owner: ObjectSlot, loc: &Coord3D) {
        self.ctx.trigger_cleanup_area(owner, loc);
    }

    fn trigger_baikonur_launch(&mut self, owner: ObjectSlot) {
        self.ctx.trigger_baikonur_launch(owner);
    }

    fn update_sub_objects(&mut self, owner: ObjectSlot, shown: &[String], hidden: &[String]) {
        self.ctx.update_sub_objects(owner, shown, hidden);
    }

    fn dispatch_on_collide(
        &mut self,
        owner: ObjectSlot,
        other: Option<ObjectSlot>,
        loc: Option<&Coord3D>,
        normal: Option<&Coord3D>,
    ) {
        self.ctx.dispatch_on_collide(owner, other, loc, normal);
    }

    fn dispatch_attempt_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType) {
        self.ctx.dispatch_attempt_upgrade(owner, mask);
    }

    fn dispatch_reset_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType) {
        self.ctx.dispatch_reset_upgrade(owner, mask);
    }

    fn client(&mut self) -> &mut dyn ClientSink {
        self.ctx.client()
    }
}
