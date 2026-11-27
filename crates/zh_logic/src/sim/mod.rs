//! `WP-5.0` / `WP-5.1`: Core simulation spine (`SimContext`, `RulesRegistry`, `SimWorld`,
//! `ObjectRecord`, `SimView`, `SimApi`, `ClientSink`, and `MockSimApi`).
//!
//! Enforces strict three-layer separation (`docs/ARCHITECTURE_SPEC.md` §1.2–§1.5):
//! 1. [`RulesRegistry`] (immutable during a match; reset truncates map overrides)
//! 2. [`SimWorld`] (all mutable state hashed by `GameLogic::getCRC`)
//! 3. [`ClientSink`] (write-only visual/audio notifications, no-op in headless simulation)

mod production;

use crate::combat::WhichTurretType;
use crate::modules::update::battle_plan_update::{self, PlayerBattlePlans};
use crate::rules::AcademyClassificationType;
use crate::modules::module::{CanMakeType, ProductionFacet};
use std::collections::{BTreeMap, BTreeSet};

use crate::arena::{
    DeferredSelfOp, FacetMask, ModuleArena, ModuleContext, ModuleDataId, ModuleMeta,
    ModuleSidecar, ModuleSlot, ObjectArena, ObjectSlot, TakeError, ThingTemplateId,
};
use crate::combat::armor::{Armor, ArmorStore};
use crate::combat::damage::{DamageInfo, PlayerMaskType, PLAYERMASK_ALL};
use crate::combat::damage_fx::DamageFXStore;
use crate::combat::locomotor::{Locomotor, LocomotorStore};
use crate::combat::locomotor_set::LocomotorSetType;
use crate::economy::energy::{Energy, EnergyObjectInfo, NoopEnergyCallbacks};
use crate::object::object_script_status_bits::OBJECT_STATUS_SCRIPT_UNSELLABLE;
use crate::economy::score_keeper::ScoreKeeper;
use crate::economy::MAX_PLAYER_COUNT;
use crate::gamelogic::global_data::{BodyDamageType, GlobalData, WeaponBonusConditionType};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::deletion_update::DeletionUpdate;
use crate::modules::update::ocl_update::OclUpdate;
use crate::modules::update::physics_update::{self, PhysicsState};
use crate::modules::update::spy_vision_update::SpyVisionUpdate;
use crate::modules::{
    AttitudeType, BehaviorModule, BridgeTowerType, CommandSourceType, ExitDoorType,
    MaxHealthChangeType, ModuleData, ModuleDataStore, ModuleRegistry, ModuleType,
    JetAiState, ParkingPlaceInfo, StealthParams,
};
use crate::modules::module::SupplyTruckAiFacet;
use crate::ai::seams::{NoPathServices, PathServices};
use crate::modules::module::AiUpdateFacet;
use crate::rules::special_power::{SpecialPowerObjectAccess, SpecialPowerTemplate};
use crate::modules::update::special_ability_update::{AbilityFeedback, SpecialAbilityUpdateModuleData};
use crate::rules::command_button::{CommandButtonId, CommandStore, ControlBarOverrideKey};
use crate::modules::module::SpawnBehaviorFacet;
use crate::modules::update::prone_update::ProneUpdate;
use crate::object::experience_tracker::{ExperienceParentAccess, ExperienceTrackerState};
use crate::object::thing::{make_align_to_normal_matrix, Thing, ThingTerrainAccess};
use crate::object::thing_factory::ThingFactory;
use crate::object::thing_template::{
    DefaultModuleMaskLookup, GameTextLookup, ThingTemplate, ThingTemplateParseContext,
};
use crate::object::weapon::WeaponStore;
use crate::object::weapon_set::{
    CanAttackResult, WeaponLockType, WeaponSet, WeaponSetConditionType, WeaponSlotType,
    WEAPONSLOT_COUNT,
};
use crate::pathfinder::pathfind_cell::CellType as PathfindCellType;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{
    WeaponStatus,
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
use crate::spatial::cave_system::{CaveSystem, CaveTunnelTracker};
use crate::spatial::crate_system::CrateSystem;
use crate::spatial::partition_collide::{
    geom_collides_with_geom, process_contacts, CollideInfo, CollisionHost,
};
use crate::spatial::partition_grid::{PartitionGrid, PartitionObject};
use crate::modules::module::{SpecialPowerCaller, SpecialPowerFacet};
use crate::modules::update::horde_update::TerrainDecalType;
use crate::spatial::partition_query::{DistanceCalculationType, IterOrderType, QueryCenter};
use crate::spatial::partition_shroud::{ShroudStatusStoreRestore, SightingInfo};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{
    div_safe_f32, gm_cosf, gm_sinf, gm_sqrtf, real_to_int_ceil, real_to_int_floor, Coord3D, Matrix3D,
    Region3D,
};
use zh_common::random_value::PrngState;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort,
    UpdateSleepTime, VeterancyLevel, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub type FindPositionFlags = UnsignedInt;
/// `TheWeaponSetTypeToModelConditionTypeMap` (`WeaponSet.h`).
fn weapon_set_model_condition(flag: WeaponSetType) -> Option<ModelConditionFlagType> {
    use ModelConditionFlagType as Mc;
    Some(match flag {
        WeaponSetType::Veteran => Mc::WeaponSetVeteran,
        WeaponSetType::Elite => Mc::WeaponSetElite,
        WeaponSetType::Hero => Mc::WeaponSetHero,
        WeaponSetType::PlayerUpgrade => Mc::WeaponSetPlayerUpgrade,
        WeaponSetType::CrateUpgradeOne => Mc::WeaponSetCrateUpgradeOne,
        WeaponSetType::CrateUpgradeTwo => Mc::WeaponSetCrateUpgradeTwo,
        WeaponSetType::VehicleHijack
        | WeaponSetType::Carbomb
        | WeaponSetType::MineClearingDetail
        | WeaponSetType::Count => return None,
        WeaponSetType::Rider1 => Mc::Rider1,
        WeaponSetType::Rider2 => Mc::Rider2,
        WeaponSetType::Rider3 => Mc::Rider3,
        WeaponSetType::Rider4 => Mc::Rider4,
        WeaponSetType::Rider5 => Mc::Rider5,
        WeaponSetType::Rider6 => Mc::Rider6,
        WeaponSetType::Rider7 => Mc::Rider7,
        WeaponSetType::Rider8 => Mc::Rider8,
    })
}

/// `IDLE_COUNTDOWN_DELAY` (`AIStates.cpp`, two seconds of logic frames).
const AI_IDLE_COUNTDOWN_DELAY: Int = 60;

pub const FPF_NONE: UnsignedInt = 0;
pub const FPF_IGNORE_WATER: UnsignedInt = 1 << 0;
pub const FPF_WATER_ONLY: UnsignedInt = 1 << 1;
pub const FPF_IGNORE_ALL_OBJECTS: UnsignedInt = 1 << 2;
pub const FPF_IGNORE_ALLY_OR_NEUTRAL_UNITS: UnsignedInt = 1 << 3;
pub const FPF_IGNORE_ENEMY_UNITS: UnsignedInt = 1 << 4;
pub const FPF_IGNORE_ALLY_OR_NEUTRAL_STRUCTURES: UnsignedInt = 1 << 5;
pub const FPF_IGNORE_ENEMY_STRUCTURES: UnsignedInt = 1 << 6;
pub const FPF_USE_HIGHEST_LAYER: UnsignedInt = 1 << 7;
pub const FPF_CLEAR_CELLS_ONLY: UnsignedInt = 1 << 8;

/// Legal-position search parameters matching `PartitionManager.h` `FindPositionOptions`.
#[derive(Debug, Clone, Copy)]
pub struct FindPositionOptions {
    pub flags: UnsignedInt,
    pub min_radius: Real,
    pub max_radius: Real,
    pub start_angle: Option<Real>,
    pub max_z_delta: Real,
    pub ignore_object: Option<ObjectSlot>,
    pub source_to_path_to_dest: Option<ObjectSlot>,
    pub relationship_object: Option<ObjectSlot>,
}

impl Default for FindPositionOptions {
    fn default() -> Self {
        Self {
            flags: FPF_NONE,
            min_radius: 0.0,
            max_radius: 0.0,
            start_angle: None,
            max_z_delta: 1e10,
            ignore_object: None,
            source_to_path_to_dest: None,
            relationship_object: None,
        }
    }
}

/// Per-player or per-cave shared tunnel containment tracker (`TunnelTracker.h` / `TunnelTracker.cpp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunnelTrackerState {
    pub tunnel_ids: Vec<ObjectId>,
    pub contain_list: Vec<ObjectId>,
    pub contain_list_size: Int,
    pub hero_units_contained: UnsignedInt,
    pub tunnel_count: UnsignedInt,
    pub frames_for_full_heal: UnsignedInt,
    pub needs_full_heal_time_update: Bool,
    pub cur_nemesis_id: ObjectId,
    pub nemesis_timestamp: UnsignedInt,
}

impl Default for TunnelTrackerState {
    fn default() -> Self {
        Self::new()
    }
}

impl TunnelTrackerState {
    pub const fn new() -> Self {
        Self {
            tunnel_ids: Vec::new(),
            contain_list: Vec::new(),
            contain_list_size: 0,
            hero_units_contained: 0,
            tunnel_count: 0,
            frames_for_full_heal: 0,
            needs_full_heal_time_update: FALSE,
            cur_nemesis_id: ObjectId::INVALID,
            nemesis_timestamp: 0,
        }
    }

    pub fn on_tunnel_created(&mut self, tunnel_id: ObjectId) {
        self.tunnel_ids.push(tunnel_id);
        #[cfg(feature = "retail-crc")]
        {
            self.tunnel_count = self.tunnel_count.wrapping_add(1);
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            self.tunnel_count = self.tunnel_ids.len() as UnsignedInt;
        }
        self.needs_full_heal_time_update = TRUE;
    }

    /// Removes `tunnel_id` and returns `TRUE` when zero entrances remain (signaling kill-all).
    /// The world half is `tunnel_contain::tracker_on_tunnel_destroyed`.
    pub fn on_tunnel_destroyed(&mut self, tunnel_id: ObjectId) -> Bool {
        #[cfg(feature = "retail-crc")]
        {
            self.tunnel_ids.retain(|&id| id != tunnel_id);
            self.tunnel_count = self.tunnel_count.wrapping_sub(1);
            Bool::from(self.tunnel_count == 0)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            if let Some(pos) = self.tunnel_ids.iter().position(|&id| id == tunnel_id) {
                self.tunnel_ids.remove(pos);
                self.needs_full_heal_time_update = TRUE;
            }
            self.tunnel_count = self.tunnel_ids.len() as UnsignedInt;
            Bool::from(self.tunnel_ids.is_empty())
        }
    }

    #[inline]
    pub fn tunnel_count_effective(&self) -> UnsignedInt {
        #[cfg(feature = "retail-crc")]
        {
            self.tunnel_count
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            self.tunnel_ids.len() as UnsignedInt
        }
    }

    pub fn add_to_contain_list(&mut self, obj_id: ObjectId, is_hero: Bool) {
        self.contain_list.push(obj_id);
        self.contain_list_size += 1;
        if is_hero != FALSE {
            self.hero_units_contained = self.hero_units_contained.saturating_add(1);
        }
    }

    pub fn remove_from_contain(&mut self, obj_id: ObjectId, is_hero: Bool) -> Bool {
        if let Some(pos) = self.contain_list.iter().position(|&id| id == obj_id) {
            self.contain_list.remove(pos);
            self.contain_list_size -= 1;
            if is_hero != FALSE {
                self.hero_units_contained = self.hero_units_contained.saturating_sub(1);
            }
            return TRUE;
        }
        FALSE
    }

    #[inline]
    pub fn is_in_container(&self, obj_id: ObjectId) -> Bool {
        Bool::from(self.contain_list.contains(&obj_id))
    }
}

impl CaveTunnelTracker for TunnelTrackerState {
    #[inline]
    fn contain_count(&self) -> UnsignedInt {
        if self.contain_list_size > 0 {
            self.contain_list_size as UnsignedInt
        } else {
            0
        }
    }
}

impl Snapshot for TunnelTrackerState {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        #[cfg(feature = "retail-xfer-save")]
        let current_version: XferVersion = 1;
        #[cfg(not(feature = "retail-xfer-save"))]
        let current_version: XferVersion = 2;

        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;
        xfer.xfer_stl_object_id_vec(&mut self.tunnel_ids)?;
        xfer.xfer_int(&mut self.contain_list_size)?;

        if xfer.mode() == XferMode::Save || xfer.mode() == XferMode::Crc {
            for id in &mut self.contain_list {
                xfer.xfer_object_id(id)?;
            }
        } else {
            self.contain_list.clear();
            let count = if self.contain_list_size > 0 {
                self.contain_list_size as usize
            } else {
                0
            };
            for _ in 0..count {
                let mut id = ObjectId::INVALID;
                xfer.xfer_object_id(&mut id)?;
                self.contain_list.push(id);
            }
            self.needs_full_heal_time_update = TRUE;
        }

        if version <= 1 {
            #[cfg(feature = "retail-crc")]
            {
                xfer.xfer_unsigned_int(&mut self.tunnel_count)?;
            }
            #[cfg(not(feature = "retail-crc"))]
            {
                let mut tunnel_count = self.tunnel_ids.len() as UnsignedInt;
                xfer.xfer_unsigned_int(&mut tunnel_count)?;
                if xfer.mode() == XferMode::Load {
                    self.tunnel_count = tunnel_count;
                }
            }
        } else if xfer.mode() == XferMode::Load {
            self.tunnel_count = self.tunnel_ids.len() as UnsignedInt;
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

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

/// The answers of `Object`'s attack queries (`WeaponSet`, unported).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AttackAbility {
    pub able: Bool,
    pub specific_object: CanAttackResult,
    pub weapon_against_target: CanAttackResult,
}

/// Write-only client/audio notification sink (`GameClient` boundary in `SimContext`).
pub trait ClientSink: Send {
    /// `Drawable::setTerrainDecal`. A drawable ignores the type it already has.
    fn on_terrain_decal(&mut self, _id: ObjectId, _decal: TerrainDecalType) {}
    /// `Drawable::setTerrainDecalSize`.
    fn on_terrain_decal_size(&mut self, _id: ObjectId, _x: Real, _y: Real) {}
    /// `Drawable::setTerrainDecalFadeTarget`. A drawable ignores the target it has.
    fn on_terrain_decal_fade_target(&mut self, _id: ObjectId, _target: Real, _rate: Real) {}
    /// `LaserUpdate::initLaser` on the drawable of `laser`.
    fn on_laser_init(
        &mut self,
        _laser: ObjectId,
        _parent: ObjectId,
        _target: ObjectId,
        _start: Coord3D,
        _end: Coord3D,
    ) {
    }
    /// Sounds, EVA lines, radar events and drawable effects of a special ability.
    fn on_special_ability_feedback(&mut self, _source: ObjectId, _feedback: &AbilityFeedback<'_>) {}
    fn on_object_created(&mut self, _id: ObjectId, _template_id: ThingTemplateId) {}
    fn on_object_destroyed(&mut self, _id: ObjectId) {}
    fn on_damage(&mut self, _target: ObjectId, _amount: Real) {}
    fn on_veterancy_changed(&mut self, _target: ObjectId, _old_level: Int, _new_level: Int) {}
    /// The welding sparks and sound of a repairing slave (`SlavedUpdate`).
    fn on_welding_sparks(&mut self, _slave: ObjectId, _system: &str, _bone: &str, _frames: Int) {}
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
    pub command_sets: crate::rules::command_set::CommandSetStore,
    pub damage_fx_store: DamageFXStore,
    pub special_power_store: SpecialPowerStore,
    pub crate_system: CrateSystem,
    pub global_data: GlobalData,
    pub fx_list_names: BTreeSet<String>,
    pub damage_fx_names: BTreeSet<String>,
    pub ocl_names: BTreeSet<String>,
    pub particle_system_names: BTreeSet<String>,
    /// `AIData::m_attackPriorityDistanceModifier` (`AI` is unported).
    pub attack_priority_distance_modifier: Real,
    /// `TheControlBar`'s buttons and sets.
    pub command_store: CommandStore,
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
        crate::modules::behavior::register_behavior_modules(&mut module_registry, &mut name_keys);
        crate::modules::contain::register_contain_modules(&mut module_registry, &mut name_keys);
        crate::modules::update::register_update_modules(&mut module_registry, &mut name_keys);

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
            command_sets: Default::default(),
            damage_fx_store: DamageFXStore::new(),
            special_power_store: SpecialPowerStore::new(),
            crate_system,
            global_data: GlobalData::new(),
            fx_list_names: BTreeSet::new(),
            damage_fx_names: BTreeSet::new(),
            ocl_names: BTreeSet::new(),
            particle_system_names: BTreeSet::new(),
            command_store: CommandStore::default(),
            attack_priority_distance_modifier: 0.0,
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

/// `CONSTRUCTION_COMPLETE` (`BuildAssistant.h`).
pub const CONSTRUCTION_COMPLETE: Real = -1.0;

/// Slot-stored object state (`Object` in `ObjectArena<ObjectRecord>`), embedding [`Thing`].
#[derive(Debug, Clone)]
pub struct ObjectRecord {
    pub id: ObjectId,
    pub thing: Thing,
    pub team_id: TeamId,
    pub status: ObjectStatusMaskType,
    pub private_status: u8,
    /// `Object::m_scriptStatus` (`OBJECT_STATUS_SCRIPT_*`).
    pub script_status: u8,
    pub geometry: GeometryInfo,
    pub disabled_mask: DisabledMaskType,
    pub disabled_till_frame: [UnsignedInt; DISABLED_COUNT],
    pub upgrades_completed: UpgradeMaskType,
    pub model_condition: ModelConditionFlags,
    pub special_model_condition_until: UnsignedInt,
    pub weapon_bonus_condition: UnsignedInt,
    pub experience_tracker: ExperienceTrackerState,
    pub weapon_set: WeaponSet,
    /// `Drawable::getBarrelCount` per slot. It comes from the model, which the port does not load.
    pub weapon_barrel_count: [Int; WEAPONSLOT_COUNT],
    pub armor_set_flags: ArmorSetFlags,
    pub layer: PathfindLayerEnum,
    pub contained_by: Option<ObjectSlot>,
    pub contained_by_frame: UnsignedInt,
    /// `AIUpdateInterface::m_lastCommandSource`; the AI update is unported.
    pub ai_last_command_source: CommandSourceType,
    pub producer_id: ObjectId,
    pub builder_id: ObjectId,
    pub sole_healing_benefactor_id: ObjectId,
    pub sole_healing_benefactor_expiration_frame: UnsignedInt,
    pub last_weapon_condition: [WeaponSetConditionType; WEAPONSLOT_COUNT],
    pub behaviors: Vec<(ModuleSlot, FacetMask)>,
    pub body_slot: Option<ModuleSlot>,
    pub contain_slot: Option<ModuleSlot>,
    /// `isWeaponBonusPassedToPassengers` of the contain module. It is module data, kept here
    /// so that it can be read while that module is executing.
    pub contain_passes_weapon_bonus: Bool,
    pub stealth_slot: Option<ModuleSlot>,
    pub ai_slot: Option<ModuleSlot>,
    pub physics_slot: Option<ModuleSlot>,
    pub bridge_slot: Option<ModuleSlot>,
    pub bridge_tower_slot: Option<ModuleSlot>,
    pub bridge_scaffold_slot: Option<ModuleSlot>,
    pub parking_slot: Option<ModuleSlot>,
    pub spawn_slot: Option<ModuleSlot>,
    pub projectile_slot: Option<ModuleSlot>,
    pub countermeasures_slot: Option<ModuleSlot>,
    pub exit_slot: Option<ModuleSlot>,
    pub rebuild_hole_slot: Option<ModuleSlot>,
    pub overcharge_slot: Option<ModuleSlot>,
    pub land_mine_slot: Option<ModuleSlot>,
    pub cave_slot: Option<ModuleSlot>,
    pub slaved_slot: Option<ModuleSlot>,
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
    pub partition_last_look: SightingInfo,
    pub partition_reveal_all_last_look: SightingInfo,
    pub partition_last_shroud: SightingInfo,
    pub vision_spied_by: [Int; MAX_PLAYER_COUNT],
    pub vision_spied_mask: PlayerMaskType,
    pub vision_range: Real,
    pub shroud_clearing_range: Real,
    pub shroud_range: Real,
    pub passengers_allowed_to_fire: Bool,
    pub sub_objects_shown: BTreeSet<String>,
    pub sub_objects_hidden: BTreeSet<String>,
    pub ai_goal_object: Option<ObjectSlot>,
    pub ai_dead: Bool,
    pub ai_enter_target: Option<ObjectSlot>,
    /// `Object::m_physics`: the members of the object's `PhysicsBehavior`.
    pub physics: Option<Box<PhysicsState>>,
    pub ai_attitude: AttitudeType,
    pub ai_free_to_exit: Bool,
    pub ai_current_victim: Option<ObjectSlot>,
    pub ai_current_victim_pos: Option<Coord3D>,
    pub ai_has_locomotor: Bool,
    pub loco_max_lift: Real,
    pub loco_max_turn_rate: Real,
    pub loco_max_braking: Real,
    pub loco_ultra_accurate: Bool,
    pub loco_close_enough_dist: Real,
    pub loco_close_enough_dist_3d: Bool,
    /// `AIUpdateInterface::m_curLocomotorSet` (`None` = `LOCOMOTORSET_INVALID`).
    pub ai_locomotor_set: Option<LocomotorSetType>,
    /// `AIUpdateInterface::m_curLocomotor`.
    pub ai_locomotor: Option<Box<Locomotor>>,
    /// Set once the AI behaviour state machine exists (`AIUpdateInterface::onObjectCreated`).
    pub ai_state_machine_ready: Bool,
    /// `Object::m_constructionPercent`.
    pub construction_percent: Real,
    /// `Object::m_healthBoxOffset`.
    pub health_box_offset: Coord3D,
    /// Copy of the body's last damage type, read while the body is executing.
    pub body_last_damage_type: Option<DamageType>,
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
            script_status: 0,
            geometry,
            disabled_mask: DisabledMaskType::new(),
            disabled_till_frame: [0; DISABLED_COUNT],
            upgrades_completed: UpgradeMaskType::new(),
            model_condition: ModelConditionFlags::new(),
            special_model_condition_until: 0,
            weapon_bonus_condition: 0,
            experience_tracker: ExperienceTrackerState::default(),
            weapon_set: WeaponSet::new(),
            weapon_barrel_count: [0; WEAPONSLOT_COUNT],
            armor_set_flags: ArmorSetFlags::new(),
            layer: PathfindLayerEnum::Ground,
            contained_by: None,
            contained_by_frame: 0,
            ai_last_command_source: CommandSourceType::FromAi,
            producer_id: ObjectId::INVALID,
            builder_id: ObjectId::INVALID,
            sole_healing_benefactor_id: ObjectId::INVALID,
            sole_healing_benefactor_expiration_frame: 0,
            last_weapon_condition: [WeaponSetConditionType::None; WEAPONSLOT_COUNT],
            behaviors: Vec::new(),
            body_slot: None,
            contain_slot: None,
            contain_passes_weapon_bonus: FALSE,
            stealth_slot: None,
            ai_slot: None,
            physics_slot: None,
            bridge_slot: None,
            bridge_tower_slot: None,
            bridge_scaffold_slot: None,
            parking_slot: None,
            spawn_slot: None,
            projectile_slot: None,
            countermeasures_slot: None,
            exit_slot: None,
            rebuild_hole_slot: None,
            overcharge_slot: None,
            land_mine_slot: None,
            cave_slot: None,
            slaved_slot: None,
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
            partition_last_look: SightingInfo::default(),
            partition_reveal_all_last_look: SightingInfo::default(),
            partition_last_shroud: SightingInfo::default(),
            vision_spied_by: [0; MAX_PLAYER_COUNT],
            vision_spied_mask: 0,
            vision_range: 0.0,
            shroud_clearing_range: 0.0,
            shroud_range: 0.0,
            passengers_allowed_to_fire: FALSE,
            sub_objects_shown: BTreeSet::new(),
            sub_objects_hidden: BTreeSet::new(),
            ai_goal_object: None,
            ai_dead: FALSE,
            ai_enter_target: None,
            physics: None,
            ai_attitude: AttitudeType::Normal,
            ai_free_to_exit: TRUE,
            ai_current_victim: None,
            ai_current_victim_pos: None,
            ai_has_locomotor: FALSE,
            loco_max_lift: 1e10,
            loco_max_turn_rate: 1e10,
            loco_max_braking: 1e10,
            loco_ultra_accurate: FALSE,
            loco_close_enough_dist: 1.0,
            loco_close_enough_dist_3d: FALSE,
            ai_locomotor_set: None,
            ai_locomotor: None,
            ai_state_machine_ready: FALSE,
            health_box_offset: Coord3D::default(),
            construction_percent: CONSTRUCTION_COMPLETE,
            body_last_damage_type: None,
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

    /// Serializes the vision, shroud, and sighting fields of `Object::xfer`
    /// (`Object.cpp:4228-4257`) in exact wire order.
    pub fn xfer_vision_and_shroud(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        xfer.xfer_snapshot(&mut self.partition_last_look)?;
        xfer.xfer_snapshot(&mut self.partition_reveal_all_last_look)?;
        xfer.xfer_snapshot(&mut self.partition_last_shroud)?;

        let mut spied_bytes = [0u8; 4 * MAX_PLAYER_COUNT];
        for (val, chunk) in self
            .vision_spied_by
            .iter()
            .zip(spied_bytes.chunks_exact_mut(4))
        {
            chunk.copy_from_slice(&val.to_le_bytes());
        }
        xfer.xfer_user(&mut spied_bytes)?;
        for (val, chunk) in self
            .vision_spied_by
            .iter_mut()
            .zip(spied_bytes.chunks_exact(4))
        {
            *val = Int::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }

        let mut mask_bytes = self.vision_spied_mask.to_le_bytes();
        xfer.xfer_user(&mut mask_bytes)?;
        self.vision_spied_mask = PlayerMaskType::from_le_bytes(mask_bytes);

        xfer.xfer_real(&mut self.vision_range)?;
        xfer.xfer_real(&mut self.shroud_clearing_range)?;
        xfer.xfer_real(&mut self.shroud_range)
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
    /// `GameLogic::getDrawIconUI`.
    fn draw_icon_ui(&self) -> Bool;
    /// `isTrueHordeMember` of the object's `HordeUpdateInterface`, if it has one.
    fn horde_member_state(&self, owner: ObjectSlot) -> Option<Bool>;
    /// `AIUpdateInterface::isIdle`; `TRUE` without an AI.
    fn ai_is_idle(&self, owner: ObjectSlot) -> Bool;
    /// `getCurLocomotor()->getPreferredHeight()`.
    fn ai_cur_locomotor_preferred_height(&self, owner: ObjectSlot) -> Option<Real>;
    /// `TerrainLogic::getExtent`.
    fn terrain_extent(&self) -> Region3D;
    /// `Object::isAbleToAttack`.
    fn object_is_able_to_attack(&self, owner: ObjectSlot) -> Bool;
    /// `Drawable::findClientUpdateModule` on the object's drawable.
    fn object_has_client_update(&self, owner: ObjectSlot, name: &str) -> Bool;
    /// `ContainModuleInterface::getApparentControllingPlayer`, as a player index.
    fn contain_apparent_controlling_player(&self, container: ObjectSlot, observer: Int) -> Option<Int>;
    /// `AIUpdateInterface::isMoving`.
    fn ai_is_moving(&self, owner: ObjectSlot) -> Bool;
    /// The owner's `getAttackInfo()->getPriority(victim)`; `None` without attack info.
    fn ai_attack_priority(&self, owner: ObjectSlot, victim: ObjectSlot) -> Option<Int>;
    /// The answers of the owner's `JetAIUpdate`.
    fn object_jet_ai(&self, owner: ObjectSlot) -> Option<JetAiState>;
    /// `AIUpdateInterface::getWhichTurretForCurWeapon`; invalid without an AI.
    fn ai_turret_for_cur_weapon(&self, owner: ObjectSlot) -> WhichTurretType;
    fn ai_turret_in_natural_position(&self, owner: ObjectSlot, turret: WhichTurretType) -> Bool;
    /// The battle plan members of the player with this index.
    fn player_battle_plans(&self, player: Int) -> Option<&PlayerBattlePlans>;
    /// `Object::findSpecialAbilityUpdate`. Reads module data, so it also finds a running module.
    fn find_special_ability_update(&self, owner: ObjectSlot, power_type: SpecialPowerType) -> Option<ModuleSlot>;
    /// `SpecialPowerUpdateInterface::isActive`.
    fn special_ability_is_active(&self, module: ModuleSlot) -> Bool;
    /// `(isReady(), getReadyFrame())` of the owner's special power module.
    fn special_power_module_ready(&self, owner: ObjectSlot, template_id: UnsignedInt) -> Option<(Bool, UnsignedInt)>;
    /// An `ActionManager::can...` answer. The action manager is unported; the world holds them.
    fn action_manager_allows(&self, actor: ObjectSlot, target: ObjectSlot, query: ActionQuery) -> Bool;
    /// `PartitionFilterLineOfSight`: terrain only, the pathfinder's view blocking is unported.
    fn is_clear_line_of_sight(&self, owner: ObjectSlot, other: ObjectSlot) -> Bool;
    /// Whether `iterateObjectsInRange(owner, radius, FROM_BOUNDINGSPHERE_2D)` yields `target`.
    fn is_in_range_bounding_2d(&self, owner: ObjectSlot, target: ObjectSlot, radius: Real) -> Bool;
    /// `iterateObjectsInRange(center, radius, FROM_CENTER_2D, ITER_SORTED_NEAR_TO_FAR)`.
    fn find_objects_near_to_far(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot>;
    /// `getClosestObject(center, FROM_CENTER_2D)` among the objects of `kind` that the player
    /// controlling `team_id` owns.
    fn find_closest_team_object_of_kind(&self, center: &Coord3D, radius: Real, kind: KindOfType, team_id: TeamId) -> Option<ObjectSlot>;
    /// Whether the owner's template lists a behavior or client update module called `name`.
    fn object_has_template_module(&self, owner: ObjectSlot, name: &str) -> Bool;
    /// `CommandSet::getCommandButton`, including the script overrides.
    fn command_set_button(&self, set_name: &str, index: usize) -> Option<CommandButtonId>;
    /// `Object::getCommandSetString`.
    fn object_command_set_string(&self, owner: ObjectSlot) -> String;
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
    /// The condition flags `Weapon::computeBonus` reads: the object's own, plus its
    /// container's when that container passes weapon bonuses to passengers.
    fn weapon_bonus_condition_flags(&self, owner: ObjectSlot) -> UnsignedInt;
    /// `Object::isReloadTimeShared`.
    fn is_reload_time_shared(&self, owner: ObjectSlot) -> Bool;
    /// `Drawable::getBarrelCount`.
    fn weapon_barrel_count(&self, owner: ObjectSlot, wslot: WeaponSlotType) -> Int;
    /// `Object::getLastShotFiredFrame`.
    fn last_shot_fired_frame(&self, owner: ObjectSlot) -> UnsignedInt;
    fn adjust_damage_by_armor(
        &self,
        target: ObjectSlot,
        armor_set_flags: &ArmorSetFlags,
        damage_type: DamageType,
        amount: Real,
    ) -> Real;
    fn ai_enable_repulsors(&self) -> Bool;
    fn has_module_by_name_key(&self, owner: ObjectSlot, mod_name_key: NameKeyType) -> bool;

    /// `wouldIgnite` of the owner's `FlammableUpdate`; `FALSE` without one.
    fn object_would_ignite(&self, owner: ObjectSlot) -> Bool;

    /// `getClosestObject(owner, range, FROM_CENTER_3D)` over the objects that would ignite.
    fn closest_would_ignite_object(&self, owner: ObjectSlot, range: Real) -> Option<ObjectSlot>;

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
    /// `BodyModuleInterface::getHealth`.
    fn object_health(&self, owner: ObjectSlot) -> Real;
    /// `BodyModuleInterface::getDamageState`.
    fn object_body_damage_state(&self, owner: ObjectSlot) -> BodyDamageType;
    /// `DockUpdateInterface::isDockOpen`; `TRUE` for an object without a dock.
    fn dock_is_open(&self, owner: ObjectSlot) -> Bool;
    /// `StealthUpdate::isTemporaryGrant` of the owner.
    fn stealth_is_temporary_grant(&self, owner: ObjectSlot) -> Bool;
    /// `Player::isPlayableSide` of the player controlling `team_id`.
    fn player_is_playable_side(&self, team_id: TeamId) -> Bool;
    fn object_has_upgrade(&self, owner: ObjectSlot, mask: &UpgradeMaskType) -> Bool;
    fn object_can_crush(&self, crusher: ObjectSlot, victim: ObjectSlot) -> Bool;
    fn object_is_above_terrain(&self, owner: ObjectSlot) -> Bool;
    fn object_has_ai(&self, owner: ObjectSlot) -> Bool;
    fn object_ai_goal_object(&self, owner: ObjectSlot) -> Option<ObjectSlot>;
    /// `PartitionManager::iterateObjectsInRange(pos, radius, FROM_CENTER_2D)` with
    /// `ITER_FASTEST` and no filters; the caller applies its own. Only objects filed
    /// by the last [`SimContext::update_partition`] and strictly closer than `radius`.
    fn find_objects_in_range(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot>;
    /// `PartitionManager::iteratePotentialCollisions`: the objects `geom` at `pos` would
    /// collide with.
    fn find_potential_collisions(
        &self,
        pos: &Coord3D,
        geom: &GeometryInfo,
        angle: Real,
        use_2d: Bool,
    ) -> Vec<ObjectSlot>;
    /// `isClearingMines` of the owner's AI module. `FALSE` without one.
    fn ai_is_clearing_mines(&self, owner: ObjectSlot) -> Bool;
    /// The same search from an object, which is never part of the result.
    fn find_objects_in_range_of(&self, owner: ObjectSlot, radius: Real) -> Vec<ObjectSlot>;
    /// The object form of the search under `FROM_BOUNDINGSPHERE_3D`.
    fn find_objects_in_range_bounding_sphere_3d(
        &self,
        owner: ObjectSlot,
        radius: Real,
    ) -> Vec<ObjectSlot>;
    fn find_special_power_module(
        &self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
    ) -> Option<ModuleSlot>;
    fn shared_special_power_ready_frame(&self, team_id: TeamId, template_id: UnsignedInt) -> UnsignedInt;
    fn player_captured_object_count(&self, team_id: TeamId) -> Int;
    fn special_power_uses_delay(&self) -> Bool;
    fn player_energy_production(&self, team_id: TeamId) -> Int;
    fn player_radar_counts(&self, team_id: TeamId) -> (Int, Int);
    fn production_cost_change(&self, team_id: TeamId, kindof: &KindOfMaskType) -> Real;
    fn world_object_slots(&self) -> Vec<ObjectSlot>;
    fn object_is_significantly_above_terrain(&self, owner: ObjectSlot) -> Bool;
    fn player_is_active(&self, team_id: TeamId) -> Bool;
    fn player_bounty_percent(&self, team_id: TeamId) -> Real;
    fn object_is_trainable(&self, owner: ObjectSlot) -> Bool;
    fn object_crusher_level(&self, owner: ObjectSlot) -> Int;
    fn object_crushable_level(&self, owner: ObjectSlot) -> Int;

    fn contain_count(&self, owner: ObjectSlot) -> UnsignedInt;
    fn contain_max(&self, owner: ObjectSlot) -> Int;
    fn is_valid_container_for(
        &self,
        container: ObjectSlot,
        obj: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool;
    /// `Player::getTunnelSystem` of the player at `player_index`.
    fn tunnel_tracker(&self, player_index: Int) -> Option<&TunnelTrackerState>;
    /// `GlobalData::m_maxTunnelCapacity`.
    fn max_tunnel_capacity(&self) -> Int;
    /// `getLastDamageInfo()->in.m_sourceID` and `getLastDamageTimestamp` of the body.
    fn object_last_damage(&self, owner: ObjectSlot) -> Option<(ObjectId, UnsignedInt)>;
    fn cave_tracker(&self, cave_index: Int) -> Option<&TunnelTrackerState>;
    fn cave_can_switch_index(&self, old_index: Int, new_index: Int) -> Bool;
    fn bridge_is_passable(&self, bridge_id: ObjectId) -> Bool;
    fn neutral_team_id(&self) -> TeamId;
    fn player_index(&self, team_id: TeamId) -> Int;
    fn player_mask(&self, team_id: TeamId) -> UnsignedShort;
    fn ai_is_in_dead_state(&self, owner: ObjectSlot) -> Bool;
    fn object_ai_enter_target(&self, owner: ObjectSlot) -> Option<ObjectSlot>;
    fn object_height_above_terrain(&self, owner: ObjectSlot) -> Real;
    fn slow_death_scale(&self) -> Real;
    fn hulk_max_lifetime_override(&self) -> Int;
    fn physics_last_collidee(&self, owner: ObjectSlot) -> ObjectId;
    /// `Object::getDrawable() != nullptr`: drawables are bound after `onObjectCreated`.
    fn object_has_drawable(&self, owner: ObjectSlot) -> Bool;
    fn local_player_index(&self) -> Int;
    /// `Player::getDefaultTeam()` of the player controlling `team_id`.
    fn player_default_team(&self, team_id: TeamId) -> TeamId;
    /// `BuildAssistant::canMakeUnit`.
    fn production_can_make_unit(&self, builder: ObjectSlot, tmpl: ThingTemplateId, own: Option<&dyn ProductionFacet>) -> CanMakeType;
    /// `ThingTemplate::calcCostToBuild`.
    fn unit_cost_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Int;
    /// `ThingTemplate::calcTimeToBuild`.
    fn unit_time_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Int;
    /// `Player::allowedToBuild`.
    fn player_allowed_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Bool;
    /// `Object::canProduceUpgrade`.
    fn can_produce_upgrade(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool;
    /// `Object::affectedByUpgrade`.
    fn affected_by_upgrade(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool;
    /// `Player::hasUpgradeInProduction`.
    fn player_has_upgrade_in_production(&self, team_id: TeamId, mask: &UpgradeMaskType) -> Bool;
    /// Whether a parking place wants a door at queue time, and whether it is full.
    fn parking_door_requirement(&self, owner: ObjectSlot, tmpl: ThingTemplateId) -> (Bool, Bool);
    /// Whether `Object::getObjectExitInterface` finds one.
    fn object_has_exit_interface(&self, owner: ObjectSlot) -> Bool;
    fn player_is_skirmish_ai(&self, team_id: TeamId) -> Bool;
    /// `Player::getRelationship` from the controller of `team_id` toward `player_index`.
    fn team_relationship_to_player(&self, team_id: TeamId, player_index: Int) -> Relationship;
    fn contain_is_special_zero_slot(&self, owner: ObjectSlot) -> Bool;
    /// `ContainModuleInterface::isBustable`.
    fn contain_is_bustable(&self, owner: ObjectSlot) -> Bool;
    /// `ContainModuleInterface::isEnclosingContainerFor`.
    fn contain_is_enclosing_container_for(&self, owner: ObjectSlot, rider: ObjectSlot) -> Bool;
    /// `BridgeBehaviorInterface::getTowerID`; `None` if the object has no such interface.
    fn bridge_tower_id(&self, owner: ObjectSlot, tower_type: BridgeTowerType) -> Option<ObjectId>;
    fn contain_is_special_overlord_style(&self, owner: ObjectSlot) -> Bool;
    fn contain_items(&self, owner: ObjectSlot) -> Vec<ObjectId>;
    fn contain_is_passenger_allowed_to_fire(&self, owner: ObjectSlot, id: ObjectId) -> Bool;
    fn exit_use_spawn_rally_point(&self, owner: ObjectSlot) -> Bool;
    fn has_physics(&self, owner: ObjectSlot) -> Bool;
    fn physics_velocity(&self, owner: ObjectSlot) -> Coord3D;
    fn physics_mass(&self, owner: ObjectSlot) -> Real;
    fn physics_center_of_mass_offset(&self, owner: ObjectSlot) -> Real;
    /// `BodyModuleInterface::getFrontCrushed` and `getBackCrushed`.
    fn object_crushed_ends(&self, owner: ObjectSlot) -> (Bool, Bool);
    /// `OpenContain::m_containList`, which tunnels and caves leave empty.
    fn contain_items_for_mass(&self, owner: ObjectSlot) -> Vec<ObjectId>;
    /// `AIUpdateInterface::getIgnoredObstacleID`; invalid without an AI.
    fn ai_ignored_obstacle_id(&self, owner: ObjectSlot) -> ObjectId;
    /// `Object::getCarrierDeckHeight`.
    fn carrier_deck_height(&self, owner: ObjectSlot) -> Real;
    fn has_ai(&self, owner: ObjectSlot) -> Bool;
    fn ai_free_to_exit(&self, owner: ObjectSlot) -> Bool;
    fn ai_current_victim(&self, owner: ObjectSlot) -> Option<ObjectSlot>;
    fn ai_current_victim_pos(&self, owner: ObjectSlot) -> Option<Coord3D>;
    fn ai_has_cur_locomotor(&self, owner: ObjectSlot) -> Bool;
    fn valid_movement_terrain(&self, layer: PathfindLayerEnum, rider: ObjectSlot, pos: &Coord3D) -> Bool;
    fn pathfind_cell_type(
        &self,
        layer: PathfindLayerEnum,
        cell_x: Int,
        cell_y: Int,
    ) -> PathfindCellType;
    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum;
    fn get_layer_height(&self, x: Real, y: Real, layer: PathfindLayerEnum) -> Real;
    /// `Pathfinder::snapPosition`: moves `pos` onto the owner's footprint grid.
    fn pathfinder_snap_position(&self, owner: ObjectSlot, pos: &mut Coord3D);
    /// `AIUpdateInterface::isDoingGroundMovement`.
    fn ai_is_doing_ground_movement(&self, owner: ObjectSlot) -> Bool;
    /// `Drawable::getPristineBonePositions` transforms from `SimWorld::pristine_bones`.
    fn pristine_bone_transforms(&self, owner: ObjectSlot, bone_prefix: &str) -> Vec<Matrix3D>;
    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        only_healthy_bridges: Bool,
    ) -> PathfindLayerEnum;
    fn object_vision_range(&self, owner: ObjectSlot) -> Real;
    fn object_shroud_clearing_range(&self, owner: ObjectSlot) -> Real;
    fn object_shroud_range(&self, owner: ObjectSlot) -> Real;
    fn contain_is_garrisonable(&self, owner: ObjectSlot) -> Bool;
    /// `PartitionManager::estimateTerrainExtremesAlongLine`, the high extreme only. `None`
    /// when no cell of the line is on the map.
    fn estimate_highest_terrain_along_line(&self, start: &Coord3D, end: &Coord3D) -> Option<Real>;
    /// `PartitionManager::getClosestObject(pos, radius, FROM_CENTER_2D)` without filters.
    fn find_closest_object_to_position(&self, pos: &Coord3D, radius: Real) -> Option<ObjectSlot>;
    /// `isImmuneToClearBuildingAttacks` of the object's contain module.
    fn contain_is_immune_to_clear(&self, owner: ObjectSlot) -> Bool;
    /// `ParkingPlaceBehaviorInterface::hasReservedSpace` on any module of `airfield`.
    fn object_has_reserved_space(&self, airfield: ObjectSlot, id: ObjectId) -> Bool;
    /// `Player::getPlayerColor` of the team's player.
    fn player_color(&self, team_id: TeamId) -> Int;
    /// `PlayerTemplate::getSide` of the team's player.
    fn player_side(&self, team_id: TeamId) -> String;
    /// `getStealth()` and its accessors. `None` without a `StealthUpdate`, or while it runs.
    fn stealth_params(&self, owner: ObjectSlot) -> Option<StealthParams>;
    /// `*getBodyModule()->getLastDamageTimestamp()`.
    fn object_last_damage_timestamp(&self, owner: ObjectSlot) -> Option<UnsignedInt>;
    /// `getWeaponInWeaponSlot(slot)->getLastShotFrame()`.
    fn weapon_slot_last_shot_frame(&self, owner: ObjectSlot, slot: usize) -> Option<UnsignedInt>;
    fn object_has_contain(&self, owner: ObjectSlot) -> Bool;
    fn contain_stealth_units_contained(&self, owner: ObjectSlot) -> UnsignedInt;
    fn contain_is_rider_change(&self, owner: ObjectSlot) -> Bool;
    /// Whether `ThingFactory::findTemplate` knows the name.
    fn thing_template_exists(&self, name: &str) -> Bool;
    /// `SpawnBehaviorInterface::getClosestSlave`; `None` without a spawn behavior.
    fn spawn_closest_slave(&self, owner: ObjectSlot, pos: &Coord3D) -> Option<Option<ObjectSlot>>;
    /// `ContainModuleInterface::getClosestRider`; `None` without a contain.
    fn contain_closest_rider(&self, owner: ObjectSlot, pos: &Coord3D)
        -> Option<Option<ObjectSlot>>;
    /// `SlavedUpdateInterface::isSelfTasking`; `FALSE` without a slaved update.
    fn slaved_is_self_tasking(&self, owner: ObjectSlot) -> Bool;
    /// The last node of `AIUpdateInterface::getPath`; `None` without a path.
    fn ai_path_destination(&self, owner: ObjectSlot) -> Option<Coord3D>;
    /// `Object::getAbleToAttackSpecificObject`, from `SimWorld::attack_abilities`.
    fn able_to_attack_specific_object(
        &self,
        owner: ObjectSlot,
        attack_type: UnsignedInt,
        target: ObjectSlot,
        cmd_source: CommandSourceType,
    ) -> CanAttackResult;
    /// `Object::getAbleToUseWeaponAgainstTarget`, from `SimWorld::attack_abilities`.
    fn able_to_use_weapon_against_target(
        &self,
        owner: ObjectSlot,
        attack_type: UnsignedInt,
        victim: Option<ObjectSlot>,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    ) -> CanAttackResult;
    /// `AIUpdateInterface::getLastCommandSource`.
    fn ai_last_command_source(&self, owner: ObjectSlot) -> CommandSourceType;
    /// The controlling player's objects, in `Player::iterateObjects` order.
    fn controlling_player_objects(&self, team_id: TeamId) -> Vec<ObjectSlot>;
    /// The objects of every player who counts `team_id`'s player as an enemy, in player order.
    fn enemy_player_objects(&self, team_id: TeamId) -> Vec<ObjectSlot>;

    /// `Object::getConstructionPercent`.
    fn object_construction_percent(&self, owner: ObjectSlot) -> Real;
    /// `Object::isLogicallyVisible`, for the local player.
    fn object_is_logically_visible(&self, owner: ObjectSlot) -> Bool;
    /// The damage type of `BodyModuleInterface::getLastDamageInfo`. `None` is the
    /// null pointer of a body that keeps none.
    fn object_last_damage_type(&self, owner: ObjectSlot) -> Option<DamageType>;
    /// `ScriptEngine::adjustToppleDirection`.
    fn adjust_topple_direction(&self, owner: ObjectSlot, direction: &mut Coord3D);
}

/// Mutable simulation interface passed into all [`BehaviorModule`] facet callbacks.
pub trait SimApi: SimView {
    fn ai_set_turret_enabled(&mut self, owner: ObjectSlot, turret: WhichTurretType, enabled: Bool);
    fn ai_recenter_turret(&mut self, owner: ObjectSlot, turret: WhichTurretType);
    /// `BodyModuleInterface::applyDamageScalar`.
    fn apply_damage_scalar(&mut self, owner: ObjectSlot, scalar: Real);
    fn player_battle_plans_mut(&mut self, player: Int) -> &mut PlayerBattlePlans;
    /// `AcademyStats::recordBattlePlanSelected` for the owner's player.
    fn record_battle_plan_selected(&mut self, owner: ObjectSlot);
    /// Runs `f` on the special power module in `module`.
    fn with_special_power_module(
        &mut self,
        module: ModuleSlot,
        f: &mut dyn FnMut(&mut dyn SpecialPowerFacet, &mut dyn SimApi, ModuleContext),
    );
    /// `initiateIntentToDoSpecialPower` on the update module in `update`; `None`
    /// when that module has no `SpecialPowerUpdateInterface` here.
    fn initiate_special_power_update(
        &mut self,
        update: ModuleSlot,
        caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        target_obj: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> Option<Bool>;
    /// `GameLogic::selectObject`; selection state is recorded, not kept.
    fn select_object(&mut self, owner: ObjectSlot, create_new: Bool, player_mask: UnsignedShort);
    /// `AICommandInterface::aiGetHealed`.
    fn ai_get_healed(&mut self, owner: ObjectSlot, heal_unit: ObjectSlot, cmd_source: CommandSourceType);
    /// `AICommandInterface::aiAttackObject`.
    fn ai_attack_object(
        &mut self,
        owner: ObjectSlot,
        victim: ObjectSlot,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    );
    /// The tail of the weapon's assistance request (`makeAssistanceRequest`):
    /// `assistant` attacks `victim` if it is free to. `TRUE` when it did.
    fn request_assist_from(
        &mut self,
        assistant: ObjectSlot,
        requesting: ObjectSlot,
        victim: ObjectSlot,
    ) -> Bool;
    fn ai_busy(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_move_to_object(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_face_object(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_face_position(&mut self, owner: ObjectSlot, pos: &Coord3D, cmd_source: CommandSourceType);
    fn ai_enter(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_ignore_obstacle(&mut self, owner: ObjectSlot, obstacle: ObjectId);
    /// The module loop of `SpecialPowerModule::initiateIntentToDoSpecialPower`.
    /// `SpecialAbilityUpdate::onExit` of another module.
    fn exit_special_ability(&mut self, module: ModuleSlot, cleanup: Bool);
    /// `ExperienceTracker::addExperiencePoints`, following the experience sink.
    fn add_experience_points(&mut self, owner: ObjectSlot, gain: Int);
    /// `Player::addSkillPoints`. Player ranks are unported; the world totals them.
    fn add_skill_points(&mut self, team_id: TeamId, points: Int);
    /// `AcademyStats::recordSpecialPowerUsed` on the controlling player.
    fn record_special_power_used(
        &mut self,
        owner: ObjectSlot,
        classification: AcademyClassificationType,
    );
    /// `AcademyStats::recordBuildingCapture`.
    fn record_building_capture(&mut self, team_id: TeamId);
    /// `StickyBombUpdate::initStickyBomb` of the charge.
    fn sticky_bomb_init(&mut self, charge: ObjectSlot, target: ObjectSlot, bomber: ObjectSlot);
    /// `StickyBombUpdate::detonate`; `FALSE` without the module.
    fn sticky_bomb_detonate(&mut self, charge: ObjectSlot) -> Bool;
    /// `Object::checkAndDetonateBoobyTrap`.
    fn check_and_detonate_booby_trap(&mut self, owner: ObjectSlot, victim: ObjectSlot) -> Bool;
    /// `Object::doSpecialPowerAtObject`.
    fn do_special_power_at_object(&mut self, owner: ObjectSlot, template_id: UnsignedInt, target: ObjectSlot, command_options: UnsignedInt, forced: Bool);
    /// `startPowerRecharge` of the owner's special power module.
    fn special_power_start_recharge(&mut self, owner: ObjectSlot, template_id: UnsignedInt);
    /// `markSpecialPowerTriggered(nullptr)` of the owner's special power module.
    fn special_power_mark_triggered(&mut self, owner: ObjectSlot, template_id: UnsignedInt);
    /// `GeometryInfo::makeRandomOffsetWithinFootprint` on the logic stream.
    fn random_offset_within_footprint(&mut self, geometry: &GeometryInfo, offset: &mut Coord3D);
    /// `CommandButtonHuntUpdate::setCommandButton`, as the hunt scripts call it.
    fn set_command_button_hunt(&mut self, owner: ObjectSlot, button_name: &str);
    /// Applies a shared reload time to every weapon in the owner's weapon set.
    fn share_weapon_reload(&mut self, owner: ObjectSlot, when: UnsignedInt, status: WeaponStatus);
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
    /// Files `module` back at `slot`, which is executing, while `f` runs, so that callbacks
    /// reach it as they reach a C++ module further up the stack. Returns it with what they
    /// changed, or `None` if the slot was freed. See `behavior_module::lend_self`.
    fn lend_module(
        &mut self,
        slot: ModuleSlot,
        module: Box<dyn BehaviorModule>,
        f: &mut dyn FnMut(&mut dyn SimApi),
    ) -> Option<Box<dyn BehaviorModule>>;
    /// `BodyModuleInterface::internalChangeHealth` on the owner's body.
    fn body_internal_change_health(&mut self, owner: ObjectSlot, delta: Real);
    /// `setScootParms` on the first `LandMineInterface` module of the owner.
    fn set_land_mine_scoot_parms(&mut self, owner: ObjectSlot, start: &Coord3D, end: &Coord3D);
    /// `AutoHealBehavior::stopHealing` on the owner's first `AutoHealBehavior`.
    fn stop_auto_healing(&mut self, owner: ObjectSlot);
    /// `attempt_damage` by a module on its own owner. A module cannot be called back while it
    /// executes: this returns the owner's status when the `onDamage` loop reached `watch`.
    fn attempt_damage_watching(
        &mut self,
        target: ObjectSlot,
        dmg: &mut DamageInfo,
        watch: ModuleSlot,
    ) -> Option<ObjectStatusMaskType>;
    /// `tryToIgnite` on the owner's `FlammableUpdate`, if it has one.
    fn try_to_ignite(&mut self, owner: ObjectSlot);
    /// `startFireSpreading` on the owner's `FireSpreadUpdate`, if it has one.
    fn start_fire_spreading(&mut self, owner: ObjectSlot);
    /// `BodyModuleInterface::setAflame`, which only rebuilds client effects.
    fn set_body_aflame(&mut self, owner: ObjectSlot, setting: Bool);
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
    /// `Money::deposit` without income tracking.
    fn deposit_player_money(&mut self, team_id: TeamId, amount: UnsignedInt);
    /// `Player::addUpgrade` with `UPGRADE_STATUS_IN_PRODUCTION`.
    fn player_add_upgrade_in_production(&mut self, team_id: TeamId, upgrade: NameKeyType);
    /// `Player::removeUpgrade`.
    fn player_remove_upgrade(&mut self, team_id: TeamId, upgrade: NameKeyType);
    /// A factory finished researching an upgrade.
    fn complete_production_upgrade(&mut self, owner: ObjectSlot, upgrade: NameKeyType);
    /// `ExitInterface::reserveDoorForExit`.
    fn reserve_exit_door(&mut self, owner: ObjectSlot, tmpl: ThingTemplateId) -> ExitDoorType;
    /// As `reserve_exit_door` with no template, for callers that must know whether
    /// there is an exit interface at all (`None` if not, or while it runs).
    fn try_reserve_exit_door(&mut self, owner: ObjectSlot) -> Option<ExitDoorType>;
    /// `ExitInterface::unreserveDoorForExit`.
    fn unreserve_exit_door(&mut self, owner: ObjectSlot, door: ExitDoorType);
    /// `ThingFactory::newObject`.
    fn factory_new_object(&mut self, tmpl: ThingTemplateId, team_id: TeamId) -> ObjectSlot;
    /// `Player::onUnitCreated`.
    fn production_unit_created(&mut self, factory: ObjectSlot, unit: ObjectSlot);
    /// `AcademyStats::recordProduction`.
    fn record_unit_production(&mut self, factory: ObjectSlot, unit: ObjectSlot);
    /// `ProductionUpdateInterface::setHoldDoorOpen` of `owner`.
    fn production_set_hold_door_open(&mut self, owner: ObjectSlot, door: ExitDoorType, hold: Bool);
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
    fn set_vision_range(&mut self, owner: ObjectSlot, range: Real);
    fn set_shroud_clearing_range(&mut self, owner: ObjectSlot, range: Real);
    fn set_shroud_range(&mut self, owner: ObjectSlot, range: Real);
    fn set_vision_spied(&mut self, owner: ObjectSlot, setting: Bool, by_whom: Int);
    fn handle_partition_cell_maintenance(&mut self, owner: ObjectSlot);
    fn set_enemy_units_vision_spied(
        &mut self,
        actor_team: TeamId,
        setting: Bool,
        which_units: &KindOfMaskType,
    );
    fn activate_spy_vision(&mut self, owner: ObjectSlot, duration: UnsignedInt);
    fn disable_internet_center_spy_vision(&mut self, victim: ObjectSlot, until_frame: UnsignedInt);
    fn set_deletion_lifetime(
        &mut self,
        owner: ObjectSlot,
        min_frames: UnsignedInt,
        max_frames: UnsignedInt,
    );
    fn grant_stealth(&mut self, owner: ObjectSlot);
    fn set_passengers_allowed_to_fire(&mut self, owner: ObjectSlot, allowed: Bool);
    fn add_radar(&mut self, team_id: TeamId, disable_proof: Bool);
    fn remove_radar(&mut self, team_id: TeamId, disable_proof: Bool);
    fn disable_radar(&mut self, team_id: TeamId, frames: UnsignedInt);
    /// `RadarUpdate::extendRadar` on the owner's first `RadarUpdate`, if any.
    fn extend_radar_update(&mut self, owner: ObjectSlot);
    /// `PowerPlantUpdateInterface::extendRods(TRUE)` on every module that has it.
    fn extend_power_plant(&mut self, owner: ObjectSlot);
    /// `PowerPlantUpdateInterface::extendRods(FALSE)`.
    fn retract_power_plant(&mut self, owner: ObjectSlot);
    /// `DockUpdateInterface::setDockOpen`.
    fn set_dock_open(&mut self, owner: ObjectSlot, open: Bool);
    /// `DockUpdateInterface::setDockCrippled`.
    fn set_dock_crippled(&mut self, owner: ObjectSlot, crippled: Bool);
    /// Runs `f` on the `SupplyTruckAIInterface` of the owner's AI module. `FALSE`
    /// when there is none.
    fn with_supply_truck(
        &mut self,
        owner: ObjectSlot,
        f: &mut dyn FnMut(&mut dyn SupplyTruckAiFacet, &mut dyn SimApi, ModuleContext),
    ) -> Bool;
    /// `ScoreKeeper::addMoneyEarned` of the player controlling `team_id`.
    fn add_money_earned(&mut self, team_id: TeamId, amount: Int);
    /// `StealthUpdate::receiveGrant(TRUE, frames)`; nothing without a `StealthUpdate`.
    fn receive_temporary_stealth_grant(&mut self, owner: ObjectSlot, frames: UnsignedInt);
    fn add_power_bonus(&mut self, team_id: TeamId, owner: ObjectSlot);
    fn remove_power_bonus(&mut self, team_id: TeamId, owner: ObjectSlot);
    fn adjust_player_power(&mut self, team_id: TeamId, delta: Int, adding: Bool);
    fn sabotage_power(&mut self, team_id: TeamId, frames: UnsignedInt);
    fn pause_special_power(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        pause: Bool,
    );
    fn unpause_special_power(&mut self, owner: ObjectSlot, template_id: UnsignedInt);
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
    fn capture_object(&mut self, owner: ObjectSlot, new_team: TeamId);
    fn update_upgrade_modules(&mut self, owner: ObjectSlot);
    fn set_crushed(&mut self, owner: ObjectSlot, front: Option<Bool>, back: Option<Bool>);
    fn set_special_power_creator(&mut self, owner: ObjectSlot, creator_id: ObjectId);
    fn set_geometry_info(&mut self, owner: ObjectSlot, geom: &GeometryInfo);
    fn set_demoralized(&mut self, owner: ObjectSlot, frames: UnsignedInt);
    fn reset_supply_dropzone_timer(&mut self, owner: ObjectSlot);
    fn eject_pilot_dying(&mut self, owner: ObjectSlot);
    fn dispatch_attempt_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType);
    fn dispatch_reset_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType);

    fn contain_add(&mut self, container: ObjectSlot, passenger: ObjectSlot);
    fn contain_remove(
        &mut self,
        container: ObjectSlot,
        passenger: ObjectSlot,
        expose_stealthed: Bool,
    );
    fn contain_remove_all(&mut self, container: ObjectSlot, expose_stealthed: Bool);
    fn contain_kill_all(&mut self, container: ObjectSlot);
    /// `ContainModuleInterface::harmAndForceExitAllContained`.
    fn contain_harm_and_force_exit_all(&mut self, container: ObjectSlot, dmg: &DamageInfo);
    fn set_contained_by(&mut self, passenger: ObjectSlot, container: Option<ObjectSlot>);
    /// `PartitionManager::registerObject` (`present`) or `unRegisterObject`. Both do
    /// nothing if the object is already in that state. A newly registered object is
    /// not seen by searches until the next [`SimContext::update_partition`].
    fn set_in_partition(&mut self, owner: ObjectSlot, present: Bool);
    /// `BodyModuleInterface::setDamageState`.
    fn set_body_damage_state(&mut self, owner: ObjectSlot, new_state: BodyDamageType);
    fn set_object_position(&mut self, owner: ObjectSlot, pos: &Coord3D);
    fn set_object_orientation(&mut self, owner: ObjectSlot, angle: Real);
    /// `Object::setTransformMatrix`.
    fn set_object_transform_matrix(&mut self, owner: ObjectSlot, mtx: &Matrix3D);
    /// `AIUpdateInterface::processCollision`: whether the collision still pushes.
    fn ai_process_collision(&mut self, owner: ObjectSlot, other: ObjectSlot) -> Bool;
    /// `ProjectileUpdateInterface::projectileHandleCollision`.
    fn projectile_handle_collision(&mut self, owner: ObjectSlot, other: Option<ObjectSlot>)
        -> Bool;
    fn set_object_layer(&mut self, owner: ObjectSlot, layer: PathfindLayerEnum);
    fn tunnel_tracker_mut(&mut self, player_index: Int) -> &mut TunnelTrackerState;
    fn cave_register(&mut self, cave_index: Int);
    fn cave_unregister(&mut self, cave_index: Int);
    /// `CaveInterface::setOriginalTeam` on `cave`'s cave module. `FALSE` when the object
    /// has none to call. The executing module cannot be reached this way.
    fn cave_set_original_team(&mut self, cave: ObjectSlot, team: TeamId) -> Bool;
    fn cave_tracker_mut(&mut self, cave_index: Int) -> Option<&mut TunnelTrackerState>;
    fn launch_projectile(
        &mut self,
        proj: ObjectSlot,
        victim: Option<ObjectSlot>,
        victim_pos: &Coord3D,
        launcher: ObjectSlot,
        wslot: WeaponSlotType,
        weapon_name: &str,
    );
    fn report_missile_to_countermeasures(&mut self, target: ObjectSlot, missile: ObjectSlot);
    fn divert_projectile_to_target(&mut self, missile: ObjectSlot, frames: UnsignedInt);
    fn set_bridge_passable(&mut self, bridge_id: ObjectId, passable: Bool);
    fn notify_bridge_tower_damage(&mut self, bridge_slot: ObjectSlot, dmg: &mut DamageInfo);
    fn link_bridge_tower(
        &mut self,
        bridge_id: ObjectId,
        tower_type: BridgeTowerType,
        tower_id: ObjectId,
    );
    fn reserve_parking_space(
        &mut self,
        airfield: ObjectSlot,
        aircraft_id: ObjectId,
        offset: Real,
    ) -> Option<ParkingPlaceInfo>;
    fn release_parking_space(&mut self, airfield: ObjectSlot, aircraft_id: ObjectId);
    fn notify_spawn_slave_died(
        &mut self,
        master: ObjectSlot,
        slave_id: ObjectId,
        dmg: &DamageInfo,
    );
    fn start_rebuild_hole(
        &mut self,
        hole: ObjectSlot,
        rebuild_tmpl: ThingTemplateId,
        spawner_id: ObjectId,
    );
    fn record_mine_detonation(&mut self, mine_id: ObjectId, trigger_id: ObjectId);
    fn order_passenger(&mut self, passenger_id: ObjectId, order: &str);
    fn ai_mark_as_dead(&mut self, owner: ObjectSlot);
    fn deselect_object(
        &mut self,
        owner: ObjectSlot,
        player_mask: UnsignedShort,
        affect_drag_selection: Bool,
    );
    fn trigger_slow_death_with_caller(
        &mut self,
        owner: ObjectSlot,
        caller_slot: ModuleSlot,
        caller_applicable: Bool,
        caller_prob: Int,
        dmg: &DamageInfo,
    ) -> Bool;
    fn apply_fling_physics(&mut self, owner: ObjectSlot, force: &Coord3D, orientation: Real);
    fn notify_slaver_die(&mut self, owner: ObjectSlot);
    fn mark_stealth_detected(&mut self, owner: ObjectSlot, frames: UnsignedInt);
    /// `StealthUpdate::receiveGrant`.
    fn receive_stealth_grant(&mut self, owner: ObjectSlot, active: Bool, frames: UnsignedInt);
    /// `GameClientRandomValueReal`.
    fn rng_client_real(&mut self, lo: Real, hi: Real) -> Real;
    /// `PhysicsBehavior::resetDynamicPhysics`.
    fn reset_dynamic_physics(&mut self, owner: ObjectSlot);
    /// `AcademyStats::recordVehicleDisguised` on the controlling player.
    fn record_vehicle_disguised(&mut self, owner: ObjectSlot);
    /// `recalcApparentControllingPlayer` of the object's contain module.
    fn contain_recalc_apparent_controlling_player(&mut self, container: ObjectSlot);
    /// `SpawnBehaviorInterface::revealSlaves`.
    fn spawn_reveal_slaves(&mut self, owner: ObjectSlot);
    /// `areAllSlavesStealthed`; `None` without a spawn behavior.
    fn spawn_all_slaves_stealthed(&mut self, owner: ObjectSlot) -> Option<Bool>;
    /// The owner's `StealthUpdate::allowedToStealth(owner)`; `FALSE` without one.
    fn stealth_allowed_to_stealth(&mut self, owner: ObjectSlot) -> Bool;
    fn spawn_give_slaves_stealth_upgrade(&mut self, owner: ObjectSlot, grant_stealth: Bool);
    /// `ThingFactory::newObject(findTemplate(name), team)`, build variations included.
    fn new_object_from_template(&mut self, name: &str, team: TeamId) -> Option<ObjectSlot>;
    /// `Player::onUnitCreated` of the unit's controlling player.
    fn on_unit_created(&mut self, factory: ObjectSlot, unit: ObjectSlot);
    fn exit_object_by_budding(
        &mut self,
        owner: ObjectSlot,
        new_obj: ObjectSlot,
        bud_host: Option<ObjectSlot>,
    );
    /// `SlavedUpdateInterface::onEnslave`.
    fn slaved_on_enslave(&mut self, slave: ObjectSlot, slaver: ObjectSlot);
    /// `SlavedUpdateInterface::onSlaverDamage`.
    fn slaved_on_slaver_damage(&mut self, slave: ObjectSlot, dmg: &DamageInfo);
    /// `BodyModuleInterface::setInitialHealth`.
    fn set_initial_health(&mut self, owner: ObjectSlot, initial_percent: Int);
    /// `AIUpdateInterface::aiForceAttackObject`.
    fn ai_force_attack_object(
        &mut self,
        owner: ObjectSlot,
        victim: ObjectSlot,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    );
    /// `AIUpdateInterface::aiAttackPosition`.
    fn ai_attack_position(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    );
    /// `AIUpdateInterface::aiGoProne`.
    fn ai_go_prone(&mut self, owner: ObjectSlot, dmg: &DamageInfo, cmd_source: CommandSourceType);
    /// `Locomotor::setUsePreciseZPos` on the owner's current locomotor.
    fn set_locomotor_use_precise_z_pos(&mut self, owner: ObjectSlot, precise: Bool);
    /// `StealthUpdate::disguiseAsObject`.
    fn stealth_disguise_as_object(&mut self, owner: ObjectSlot, target: Option<ObjectSlot>);
    /// `setSDEnabled` on the owner's first `StealthDetectorUpdate`.
    fn set_stealth_detector_enabled(&mut self, owner: ObjectSlot, enabled: Bool);
    fn order_ai_exit(
        &mut self,
        rider: ObjectSlot,
        container: ObjectSlot,
        cmd_source: CommandSourceType,
    );
    fn contain_process_damage_to_contained(&mut self, container: ObjectSlot, percent_damage: Real);
    fn exit_object_via_door(&mut self, container: ObjectSlot, obj: ObjectSlot, door: ExitDoorType);
    fn set_object_team(&mut self, owner: ObjectSlot, new_team: TeamId);
    fn set_physics_allow_to_fall(&mut self, owner: ObjectSlot, allow: Bool);
    fn physics_clear_acceleration(&mut self, owner: ObjectSlot);
    fn physics_scrub_velocity_2d(&mut self, owner: ObjectSlot, desired_velocity: Real);
    fn physics_apply_force(&mut self, owner: ObjectSlot, force: &Coord3D);
    fn physics_apply_shock(&mut self, owner: ObjectSlot, force: &Coord3D);
    fn physics_apply_motive_force(&mut self, owner: ObjectSlot, force: &Coord3D);
    fn physics_apply_random_rotation(&mut self, owner: ObjectSlot);
    fn physics_set_roll_rate(&mut self, owner: ObjectSlot, rate: Real);
    fn physics_set_pitch_rate(&mut self, owner: ObjectSlot, rate: Real);
    /// `Locomotor::setMaxBraking` on the current locomotor.
    fn set_locomotor_max_braking(&mut self, owner: ObjectSlot, max_braking: Real);
    /// `PhysicsBehavior::setYawRate`.
    fn physics_set_yaw_rate(&mut self, owner: ObjectSlot, rate: Real);
    /// `Pathfinder::createAWallFromMyFootprint`. Only the membership is recorded.
    fn pathfinder_create_wall(&mut self, owner: ObjectSlot);
    /// `Pathfinder::removeWallFromMyFootprint`.
    fn pathfinder_remove_wall(&mut self, owner: ObjectSlot);
    /// `fireWeaponTemplate` with `isProjectileDetonation`, which is unported: recorded.
    fn fire_projectile_detonation(&mut self, owner: ObjectSlot, weapon_name: &str, pos: &Coord3D, bonus_flags: UnsignedInt);
    /// `AcademyStats::recordClearedGarrisonedBuilding` on the controlling player.
    fn record_cleared_garrisoned_building(&mut self, owner: ObjectSlot);
    fn physics_set_is_in_free_fall(&mut self, owner: ObjectSlot, in_free_fall: Bool);
    fn ai_idle(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_hunt(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_move_to_position(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    );
    fn ai_follow_path(
        &mut self,
        owner: ObjectSlot,
        path: &[Coord3D],
        ignore_obj: Option<ObjectSlot>,
        cmd_source: CommandSourceType,
    );
    fn ai_hack_internet(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType);
    fn ai_set_attitude(&mut self, owner: ObjectSlot, attitude: AttitudeType);
    fn ai_wake_up_and_attempt_to_target(&mut self, owner: ObjectSlot);
    fn set_locomotor_max_lift(&mut self, owner: ObjectSlot, max_lift: Real);
    fn set_locomotor_max_turn_rate(&mut self, owner: ObjectSlot, max_turn_rate: Real);
    fn set_locomotor_ultra_accurate(&mut self, owner: ObjectSlot, ultra: Bool);
    fn set_locomotor_close_enough_dist(&mut self, owner: ObjectSlot, dist: Real);
    fn set_locomotor_close_enough_dist_3d(&mut self, owner: ObjectSlot, is_3d: Bool);
    fn find_position_around(
        &mut self,
        center: &Coord3D,
        options: &FindPositionOptions,
    ) -> Option<Coord3D>;
    fn pathfinder_add_object(&mut self, owner: ObjectSlot);
    /// `Pathfinder::adjustDestination` as on open ground; the pathfinder is unported.
    fn pathfinder_adjust_destination(&mut self, owner: ObjectSlot, dest: &mut Coord3D) -> Bool;
    /// `AIUpdateInterface::aiFollowExitProductionPath`. Recorded only.
    fn ai_follow_exit_production_path(
        &mut self,
        owner: ObjectSlot,
        path: &[Coord3D],
        ignore_obj: Option<ObjectSlot>,
        cmd_source: CommandSourceType,
    );
    fn pathfinder_adjust_to_possible_destination(
        &mut self,
        owner: ObjectSlot,
        dest: &mut Coord3D,
    ) -> Bool;
    fn pathfinder_update_goal(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        layer: PathfindLayerEnum,
    );
    /// PRNG-visible part of the `AIUpdateInterface` constructor: `chooseLocomotorSet(NORMAL)`.
    fn ai_construct(&mut self, owner: ObjectSlot);
    /// PRNG-visible part of `AIUpdateInterface::onObjectCreated` (state machine creation).
    fn ai_init_state_machine(&mut self, owner: ObjectSlot);
    /// `AIUpdateInterface::chooseLocomotorSet`.
    fn ai_choose_locomotor_set(&mut self, owner: ObjectSlot, set_type: LocomotorSetType);
    fn client(&mut self) -> &mut dyn ClientSink;

    /// The state of a resident module, for its own entry points and its callers.
    /// `None` while a `&mut self` method of it is running.
    fn resident_state_mut(&mut self, module: ModuleSlot) -> Option<&mut dyn std::any::Any>;
    /// `TheAI->pathfinder()`.
    fn paths(&mut self) -> &mut dyn PathServices;

    /// `GameClientRandomValue`.
    fn rng_client_int(&mut self, lo: Int, hi: Int) -> Int;
    fn set_object_construction_percent(&mut self, owner: ObjectSlot, percent: Real);
    /// `Object::topple`.
    fn topple_object(
        &mut self,
        owner: ObjectSlot,
        direction: &Coord3D,
        speed: Real,
        options: UnsignedInt,
    );
    /// `Pathfinder::removeObjectFromPathfindMap`.
    fn pathfinder_remove_object(&mut self, owner: ObjectSlot);
    /// `BoneFXUpdate::stopAllBoneFX` (the module is unported: the call is recorded).
    fn stop_all_bone_fx(&mut self, owner: ObjectSlot);
    /// `InGameUI::addFloatingText` of an "add cash" message.
    fn add_floating_money_text(&mut self, owner: ObjectSlot, pos: &Coord3D, amount: Int);
    /// The mask of the template in the `static` of `AutoDepositUpdate`'s
    /// `getUpgradedSupplyBoost`: the first call looks `name` up, later calls ignore it.
    fn auto_deposit_boost_upgrade(&mut self, name: &str) -> Option<UpgradeMaskType>;
}

/// An `aiFollowExitProductionPath` order; the AI state it drives is unported.
#[derive(Debug, Clone, PartialEq)]
pub struct ExitPathOrder {
    pub object: ObjectId,
    pub path: Vec<Coord3D>,
    pub ignore: ObjectId,
    pub cmd_source: CommandSourceType,
}

impl SimContext {
    /// `Pathfinder::getRadiusAndCenter` and `worldToCell`: the grid cell a goal for
    /// `owner` at `pos` falls in, and whether the unit centers in it.
    fn pathfind_goal_cell(&self, owner: ObjectSlot, pos: &Coord3D) -> (Int, Int, bool) {
        let cell = crate::pathfinder::pathfind_constants::PATHFIND_CELL_SIZE_F;
        let mut center = true;
        if let Some(obj) = self.world.objects.get(owner) {
            let mut diameter = 2.0 * obj.geometry.get_bounding_circle_radius();
            if diameter > cell && diameter < 2.0 * cell {
                diameter = 2.0 * cell;
            }
            let radius = real_to_int_floor(diameter / cell + 0.3).max(1);
            // An odd cell count centers; so does a footprint clipped to the maximum.
            center = radius & 1 != 0 || radius / 2 > 2;
        }
        let shift = if center { 0.0 } else { cell / 2.0 };
        let hi = self.world.map_extent_hi;
        let index = |v: Real, hi: Real| {
            let last = real_to_int_floor(hi / cell).wrapping_sub(1);
            real_to_int_floor((v + shift) / cell).min(last).max(0)
        };
        (index(pos.x, hi.x), index(pos.y, hi.y), center)
    }

    /// `Pathfinder::adjustCoordToCell`.
    fn adjust_coord_to_cell(
        &self,
        cell_x: Int,
        cell_y: Int,
        center: bool,
        pos: &mut Coord3D,
        layer: PathfindLayerEnum,
    ) {
        let size = crate::pathfinder::pathfind_constants::PATHFIND_CELL_SIZE_F;
        let coord = |c: Int| {
            if center {
                (c as Real + 0.5) * size
            } else {
                // The 0.05 is a double in the reference.
                ((f64::from(c as Real) + 0.05) * f64::from(size)) as Real
            }
        };
        pos.x = coord(cell_x);
        pos.y = coord(cell_y);
        pos.z = self.get_layer_height(pos.x, pos.y, layer);
    }
}

/// The smallest change of position or angle `Object::reactToTransformChange` acts on.
const TRANSFORM_THRESHOLD: Real = 0.01;

/// `PartitionCellSize` in the retail `GameData.ini`.
pub const DEFAULT_PARTITION_CELL_SIZE: Real = 40.0;

/// What the partition grid reads from the object `id`.
fn partition_object(
    objects: &ObjectArena<ObjectRecord>,
    factory: &ThingFactory,
    id: ObjectId,
) -> Option<PartitionObject> {
    let obj = objects.get(objects.slot_by_id(id)?)?;
    let tmpl = obj.thing.template_override().get_final(factory);
    Some(PartitionObject {
        id,
        pos: *obj.thing.get_position(),
        angle: obj.thing.get_orientation(),
        geometry: obj.geometry.clone(),
        is_immobile: tmpl.map_or(FALSE, |t| t.is_kind_of(KindOfType::Immobile)),
        build_cost: tmpl.map_or(0, |t| Int::from(t.get_build_cost())),
    })
}

/// What `canUseSpecialPower` asks of an object.
struct PowerUser<'a> {
    sim: &'a SimContext,
    team_id: TeamId,
    has_module: bool,
}

impl SpecialPowerObjectAccess for PowerUser<'_> {
    fn has_special_power_module(&self, _template: &SpecialPowerTemplate) -> bool {
        self.has_module
    }
    fn controlling_player_has_science(&self, science: ScienceType) -> bool {
        self.sim.player_has_science(self.team_id, science) != FALSE
    }
}

/// A question the unported `ActionManager` answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ActionQuery {
    /// `canDoSpecialPowerAtObject` with the template of this id.
    SpecialPowerAtObject(UnsignedInt),
    HijackVehicle,
    SabotageBuilding,
    ConvertToCarBomb,
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
    pub team_members: BTreeMap<TeamId, Vec<ObjectSlot>>,
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
    /// The module `attempt_damage_watching` waits for, and the status seen at its turn.
    pub damage_watch: Option<(ModuleSlot, Option<ObjectStatusMaskType>)>,
    pub triggered_fx_lists: Vec<(String, Coord3D)>,
    pub triggered_damage_fx: Vec<(String, DamageType, Real, ObjectId, ObjectId)>,
    pub player_money: BTreeMap<TeamId, UnsignedInt>,
    pub player_is_human: BTreeMap<TeamId, Bool>,
    pub team_is_neutral: BTreeMap<TeamId, Bool>,
    pub player_cost_modifiers: BTreeMap<TeamId, Vec<(KindOfMaskType, Real, Int)>>,
    pub player_radars: BTreeMap<TeamId, (Int, Int)>,
    pub player_energy: BTreeMap<TeamId, Energy>,
    pub player_radar_disabled_until: BTreeMap<TeamId, UnsignedInt>,
    pub player_power_sabotaged_until: BTreeMap<TeamId, UnsignedInt>,
    pub player_bounty_percent: BTreeMap<TeamId, Real>,
    pub player_captured_objects: BTreeMap<TeamId, Int>,
    pub shared_special_power_ready_frames: BTreeMap<(TeamId, UnsignedInt), UnsignedInt>,
    pub special_power_uses_delay: Bool,
    /// `Player::getScoreKeeper()` by player index.
    pub score_keepers: BTreeMap<Int, ScoreKeeper>,
    /// `ScriptEngine::m_toppleDirections`, keyed by the named object's id (objects
    /// have no name yet).
    pub script_topple_directions: BTreeMap<ObjectId, Coord3D>,
    /// `BoneFXUpdate::stopAllBoneFX` calls.
    pub bone_fx_stops: Vec<ObjectId>,
    /// "Add cash" floating texts: object, position, amount.
    pub floating_money_texts: Vec<(ObjectId, Coord3D, Int)>,
    /// The `static const UpgradeTemplate*` of `AutoDepositUpdate`: unset until the
    /// first lookup, then its result for good.
    pub auto_deposit_boost_upgrade: Option<Option<UpgradeMaskType>>,
    /// Teams whose controlling player is a playable side (`Player::isPlayableSide`).
    pub playable_side_teams: BTreeSet<TeamId>,
    pub car_bombed_objects: Vec<(ObjectId, ObjectId)>,
    pub hijacked_objects: Vec<(ObjectId, ObjectId)>,
    pub dam_break_events: Vec<ObjectId>,
    pub rebuild_hole_events: Vec<(ObjectId, ObjectId, Real)>,
    pub special_power_triggered_events: Vec<(TeamId, String, ObjectId)>,
    pub special_power_completed_events: Vec<(ObjectId, SpecialPowerType)>,
    pub public_timer_events: Vec<(ObjectId, UnsignedInt, UnsignedInt, Bool)>,
    pub weapon_fires: Vec<(ObjectId, String, Option<ObjectId>, Coord3D)>,
    pub crate_spawns: Vec<(String, ObjectId, Option<ObjectId>)>,
    pub special_ability_events: Vec<(ObjectId, Option<ObjectId>, Option<Coord3D>)>,
    pub cleanup_area_events: Vec<(ObjectId, Coord3D)>,
    pub baikonur_launch_events: Vec<ObjectId>,
    pub stealth_granted: BTreeSet<ObjectId>,
    pub player_inactive: BTreeSet<TeamId>,
    pub demoralized_events: Vec<(ObjectId, UnsignedInt)>,
    pub supply_dropzone_resets: Vec<ObjectId>,
    pub neutral_team_id: TeamId,
    /// Keyed by player index.
    pub tunnel_trackers: BTreeMap<Int, TunnelTrackerState>,
    pub cave_system: CaveSystem<TunnelTrackerState>,
    pub bridge_links: BTreeMap<ObjectId, [ObjectId; 4]>,
    pub bridge_passable: BTreeMap<ObjectId, Bool>,
    pub mine_detonations: Vec<(ObjectId, ObjectId)>,
    pub projectile_launches: Vec<(ObjectId, Option<ObjectId>, Coord3D)>,
    pub countermeasure_reports: Vec<(ObjectId, ObjectId)>,
    pub parking_reservations: BTreeMap<ObjectId, ObjectId>,
    /// `Player`'s battle plan members by player index.
    pub player_battle_plans: BTreeMap<Int, PlayerBattlePlans>,
    pub passenger_orders: Vec<(ObjectId, String)>,
    pub slow_death_scale: Real,
    pub hulk_max_lifetime_override: Int,
    pub deselected_objects: Vec<(ObjectId, UnsignedShort, Bool)>,
    pub stealth_detected_events: Vec<(ObjectId, UnsignedInt)>,
    /// Footprints registered by `createAWallFromMyFootprint`.
    pub pathfind_walls: BTreeSet<ObjectId>,
    /// Unported detonation damage: source, weapon, position, extra bonus flags.
    pub projectile_detonations: Vec<(ObjectId, String, Coord3D, UnsignedInt)>,
    /// `Player::m_color` by team.
    pub team_colors: BTreeMap<TeamId, Int>,
    /// `PlayerTemplate::getSide` by team.
    pub team_sides: BTreeMap<TeamId, String>,
    pub slaved_die_notifications: Vec<ObjectId>,
    pub ai_exit_orders: Vec<(ObjectId, ObjectId, CommandSourceType)>,
    pub ai_commands: Vec<(ObjectId, String)>,
    /// `GameLogic::m_drawIconUI`.
    pub draw_icon_ui: Bool,
    /// `GameLogic::selectObject` calls: object, new selection, player mask, local.
    pub selected_objects: Vec<(ObjectId, Bool, UnsignedShort, Bool)>,
    /// What each object's weapons answer; stands in for the unported `WeaponSet` queries.
    pub attack_abilities: BTreeMap<ObjectId, AttackAbility>,
    pub exit_path_orders: Vec<ExitPathOrder>,
    /// Pristine model bones by template name and bone prefix, as a model loader would
    /// report them. No entry means the model has no such bones.
    pub pristine_bones: BTreeMap<(String, String), Vec<Matrix3D>>,
    pub pathfind_map_objects: BTreeSet<ObjectId>,
    pub pathfind_goals: BTreeMap<ObjectId, (Coord3D, PathfindLayerEnum)>,
    pub blocked_pathfind_cells: BTreeMap<(Int, Int, Int), PathfindCellType>,
    pub valid_movement_terrain_override: Option<Bool>,
    pub map_extent_hi: Coord3D,
    /// `ThePartitionManager`'s cell grid. Sized by [`SimContext::init_partition`].
    pub partition: PartitionGrid,
    /// `ThePlayerList->getLocalPlayer()` index (only used for apparent-controller visuals).
    pub local_player_index: Int,
    /// `Player::getDefaultTeam()` by player index.
    pub player_default_teams: BTreeMap<Int, TeamId>,
    /// Teams controlled by a skirmish AI player (`Player::isSkirmishAIPlayer`).
    pub skirmish_ai_teams: BTreeSet<TeamId>,
    /// What the unported `ActionManager` would allow: (actor, target, question).
    pub action_permissions: BTreeSet<(ObjectId, ObjectId, ActionQuery)>,
    /// `GameLogic::m_controlBarOverrides`: (command set, slot) to button.
    pub control_bar_overrides: BTreeMap<ControlBarOverrideKey, Option<CommandButtonId>>,
    /// Skill points awarded by modules, by player index.
    pub skill_points_awarded: BTreeMap<Int, Int>,
    /// `Player::m_upgradesInProgress`.
    pub player_upgrades_in_progress: BTreeMap<TeamId, UpgradeMaskType>,
    /// `Player::m_handicap`; absent means every handicap is 1.
    pub player_handicaps: BTreeMap<TeamId, crate::economy::handicap::Handicap>,
    /// `Player::m_productionCostChanges` by template name.
    pub player_cost_change_percent: BTreeMap<(TeamId, String), Real>,
    /// `Player::m_productionTimeChanges` by template name.
    pub player_time_change_percent: BTreeMap<(TeamId, String), Real>,
    /// Teams whose player has `m_canBuildBase` cleared.
    pub teams_cannot_build_base: BTreeSet<TeamId>,
    /// Teams whose player has `m_canBuildUnits` cleared.
    pub teams_cannot_build_units: BTreeSet<TeamId>,
    /// `Player::getAcademyStats()` by player index.
    pub player_academy: BTreeMap<Int, crate::economy::academy_stats::AcademyStats>,
    /// `GameLogic::m_thingTemplateBuildableOverrides` by template name.
    pub buildable_overrides: BTreeMap<String, crate::object::thing_template::BuildableStatus>,
    /// `GameLogic::getSuperweaponRestriction`.
    pub superweapon_restriction: UnsignedShort,
    /// `GameLogic::isScoringEnabled`.
    pub scoring_enabled: Bool,
    /// `ScriptEngine::notifyOfCompletedUpgrade`: player index, upgrade, source object.
    pub completed_upgrades: Vec<(Int, String, ObjectId)>,
    /// `ScriptEngine::m_frameObjectCountChanged`, as far as production sets it.
    pub object_count_changed_frame: UnsignedInt,
    /// `AIPlayer::onUnitProduced` calls (factory, unit); the AI player is unported.
    pub units_produced: Vec<(ObjectId, ObjectId)>,
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
            team_members: BTreeMap::new(),
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
            damage_watch: None,
            triggered_fx_lists: Vec::new(),
            triggered_damage_fx: Vec::new(),
            player_money: BTreeMap::new(),
            player_is_human: BTreeMap::new(),
            team_is_neutral: BTreeMap::new(),
            player_cost_modifiers: BTreeMap::new(),
            player_radars: BTreeMap::new(),
            player_energy: BTreeMap::new(),
            player_radar_disabled_until: BTreeMap::new(),
            player_power_sabotaged_until: BTreeMap::new(),
            player_bounty_percent: BTreeMap::new(),
            player_captured_objects: BTreeMap::new(),
            shared_special_power_ready_frames: BTreeMap::new(),
            special_power_uses_delay: TRUE,
            score_keepers: BTreeMap::new(),
            script_topple_directions: BTreeMap::new(),
            bone_fx_stops: Vec::new(),
            floating_money_texts: Vec::new(),
            auto_deposit_boost_upgrade: None,
            playable_side_teams: BTreeSet::new(),
            car_bombed_objects: Vec::new(),
            hijacked_objects: Vec::new(),
            dam_break_events: Vec::new(),
            rebuild_hole_events: Vec::new(),
            special_power_triggered_events: Vec::new(),
            special_power_completed_events: Vec::new(),
            public_timer_events: Vec::new(),
            weapon_fires: Vec::new(),
            crate_spawns: Vec::new(),
            special_ability_events: Vec::new(),
            cleanup_area_events: Vec::new(),
            baikonur_launch_events: Vec::new(),
            stealth_granted: BTreeSet::new(),
            player_inactive: BTreeSet::new(),
            demoralized_events: Vec::new(),
            supply_dropzone_resets: Vec::new(),
            neutral_team_id: TeamId(0),
            tunnel_trackers: BTreeMap::new(),
            cave_system: CaveSystem::new(),
            bridge_links: BTreeMap::new(),
            bridge_passable: BTreeMap::new(),
            mine_detonations: Vec::new(),
            projectile_launches: Vec::new(),
            countermeasure_reports: Vec::new(),
            parking_reservations: BTreeMap::new(),
            player_battle_plans: BTreeMap::new(),
            passenger_orders: Vec::new(),
            slow_death_scale: 1.0,
            hulk_max_lifetime_override: -1,
            deselected_objects: Vec::new(),
            stealth_detected_events: Vec::new(),
            pathfind_walls: BTreeSet::new(),
            projectile_detonations: Vec::new(),
            team_colors: BTreeMap::new(),
            team_sides: BTreeMap::new(),
            slaved_die_notifications: Vec::new(),
            ai_exit_orders: Vec::new(),
            ai_commands: Vec::new(),
            draw_icon_ui: TRUE,
            selected_objects: Vec::new(),
            attack_abilities: BTreeMap::new(),
            exit_path_orders: Vec::new(),
            pristine_bones: BTreeMap::new(),
            pathfind_map_objects: BTreeSet::new(),
            pathfind_goals: BTreeMap::new(),
            blocked_pathfind_cells: BTreeMap::new(),
            valid_movement_terrain_override: None,
            map_extent_hi: Coord3D::new(1000.0, 1000.0, 100.0),
            partition: PartitionGrid::new(
                DEFAULT_PARTITION_CELL_SIZE,
                Region3D {
                    lo: Coord3D::new(0.0, 0.0, 0.0),
                    hi: Coord3D::new(1000.0, 1000.0, 100.0),
                },
            ),
            local_player_index: 0,
            player_default_teams: BTreeMap::new(),
            skirmish_ai_teams: BTreeSet::new(),
            action_permissions: BTreeSet::new(),
            control_bar_overrides: BTreeMap::new(),
            skill_points_awarded: BTreeMap::new(),
            player_upgrades_in_progress: BTreeMap::new(),
            player_handicaps: BTreeMap::new(),
            player_cost_change_percent: BTreeMap::new(),
            player_time_change_percent: BTreeMap::new(),
            teams_cannot_build_base: BTreeSet::new(),
            teams_cannot_build_units: BTreeSet::new(),
            player_academy: BTreeMap::new(),
            buildable_overrides: BTreeMap::new(),
            superweapon_restriction: 0,
            scoring_enabled: TRUE,
            completed_upgrades: Vec::new(),
            object_count_changed_frame: 0,
            units_produced: Vec::new(),
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
        let passes_weapon_bonus = module
            .as_contain()
            .map_or(FALSE, |c| c.is_weapon_bonus_passed_to_passengers());
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
                obj.contain_passes_weapon_bonus = passes_weapon_bonus;
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
            if facet_mask.contains(FacetMask::BRIDGE_BEHAVIOR) && obj.bridge_slot.is_none() {
                obj.bridge_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::BRIDGE_TOWER) && obj.bridge_tower_slot.is_none() {
                obj.bridge_tower_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::BRIDGE_SCAFFOLD)
                && obj.bridge_scaffold_slot.is_none()
            {
                obj.bridge_scaffold_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::PARKING_PLACE) && obj.parking_slot.is_none() {
                obj.parking_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::SPAWN_BEHAVIOR) && obj.spawn_slot.is_none() {
                obj.spawn_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::PROJECTILE_UPDATE) && obj.projectile_slot.is_none() {
                obj.projectile_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::COUNTERMEASURES) && obj.countermeasures_slot.is_none()
            {
                obj.countermeasures_slot = Some(mod_slot);
            }
            // `getObjectExitInterface` asks the update modules first; a container's exit
            // is reached through `contain_slot` when there is none.
            if facet_mask.contains(FacetMask::EXIT_INTERFACE)
                && !facet_mask.contains(FacetMask::CONTAIN)
                && obj.exit_slot.is_none()
            {
                obj.exit_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::REBUILD_HOLE) && obj.rebuild_hole_slot.is_none() {
                obj.rebuild_hole_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::OVERCHARGE) && obj.overcharge_slot.is_none() {
                obj.overcharge_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::LAND_MINE) && obj.land_mine_slot.is_none() {
                obj.land_mine_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::CAVE_INTERFACE) && obj.cave_slot.is_none() {
                obj.cave_slot = Some(mod_slot);
            }
            if facet_mask.contains(FacetMask::SLAVED_UPDATE) && obj.slaved_slot.is_none() {
                obj.slaved_slot = Some(mod_slot);
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

    /// Prepends `slot` to `team`'s member list (`Team::prependTo_TeamMemberList`).
    pub fn team_member_prepend(&mut self, team: TeamId, slot: ObjectSlot) {
        let list = self.team_members.entry(team).or_default();
        list.retain(|&s| s != slot);
        list.insert(0, slot);
    }

    /// Removes `slot` from `team`'s member list (`Team::removeFrom_TeamMemberList`).
    pub fn team_member_remove(&mut self, team: TeamId, slot: ObjectSlot) {
        if let Some(list) = self.team_members.get_mut(&team) {
            list.retain(|&s| s != slot);
        }
    }
}

/// Top-level three-layer simulation context (`RulesRegistry` + `SimWorld` + `ClientSink`).
pub struct SimContext {
    pub rules: RulesRegistry,
    pub world: SimWorld,
    pub terrain: Box<dyn ThingTerrainAccess>,
    pub client: Box<dyn ClientSink>,
    /// The path queue, until the pathfinder is ported into the world.
    pub paths: Box<dyn PathServices>,
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
            paths: Box::new(NoPathServices),
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

    fn attack_ability(&self, owner: ObjectSlot) -> AttackAbility {
        self.world
            .objects
            .id_of(owner)
            .and_then(|id| self.world.attack_abilities.get(&id).copied())
            .unwrap_or_default()
    }

    /// Sends an order to the owner's AI module, which is where the AI port reacts to it.
    fn order_ai(
        &mut self,
        owner: ObjectSlot,
        order: &str,
        f: impl FnOnce(&mut dyn AiUpdateFacet, &mut dyn SimApi, ModuleContext),
    ) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world.ai_commands.push((obj_id, order.to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    f(ai, sim, ctx);
                }
            });
        }
    }

    /// Runs `f` on the spawn behavior of an object whose spawns are its weapons.
    fn with_weapon_slaves(
        &mut self,
        owner: ObjectSlot,
        f: impl FnOnce(&mut dyn SpawnBehaviorFacet, &mut dyn SimApi),
    ) {
        if self.object_is_kind_of(owner, KindOfType::SpawnsAreTheWeapons) == FALSE {
            return;
        }
        let Some(ss) = self.world.objects.get(owner).and_then(|o| o.spawn_slot) else {
            return;
        };
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, _| {
                if let Some(s) = m.as_spawn_behavior_mut() {
                    f(s, sim);
                }
            });
        }
    }

    /// The `AutoDepositUpdate` part of `Player::becomingTeamMember(obj, true)`.
    fn award_capture_bonus(&mut self, owner: ObjectSlot) {
        if self.is_neutral_controlled(owner) != FALSE {
            return;
        }
        // `findUpdateModule("AutoDepositUpdate")`.
        let Some(ms) = self.module_by_name(owner, "AutoDepositUpdate") else {
            return;
        };
        let _ = self.dispatch_module(ms, |m, sim, ctx| {
            if let Some(deposit) = m.as_auto_deposit_mut() {
                deposit.award_initial_capture_bonus(sim, ctx);
            }
        });
    }

    /// `Object::findModule` by module name.
    fn module_by_name(&mut self, owner: ObjectSlot, name: &str) -> Option<ModuleSlot> {
        let key = self.rules.name_keys.name_to_key(name);
        self.behavior_slots(owner).into_iter().find(|&ms| {
            self.world
                .modules
                .payload(ms)
                .is_some_and(|p| p.module_name_key() == key)
        })
    }

    /// Collects all object slots controlled by `player_idx` in `Player::iterateObjects`
    /// (`defaultTeam` first, then other teams owned by `player_idx` in `team_members` order).
    fn player_object_slots(&self, player_idx: Int) -> Vec<ObjectSlot> {
        let default_team = self.world.player_default_teams.get(&player_idx).copied();
        let mut out = Vec::new();
        if let Some(def_team) = default_team {
            if let Some(members) = self.world.team_members.get(&def_team) {
                out.extend_from_slice(members);
            }
        }
        for (&team_id, members) in &self.world.team_members {
            if Some(team_id) != default_team && self.player_index_for_team(team_id) == player_idx {
                out.extend_from_slice(members);
            }
        }
        out
    }

    /// Sets the map extent, rebuilds the partition grid with `cell_size` over it,
    /// preserves fogged and permanently revealed cells (`TerrainLogic::setActiveBoundary`),
    /// and re-registers every object in world list order (`friend_prepareForMapBoundaryAdjust`
    /// and `friend_notifyOfNewMapBoundary`).
    pub fn init_partition(&mut self, cell_size: Real, extent: Region3D) {
        let mut store = ShroudStatusStoreRestore::default();
        let players = self.partition_player_count();
        self.world
            .partition
            .process_all_pending_undo_shroud_reveals(players);
        self.world.partition.store_fogged_cells(&mut store, TRUE);

        let slots: Vec<ObjectSlot> = self.world.objects.world_iter().collect();
        for &slot in &slots {
            if let Some(id) = self.world.objects.id_of(slot) {
                self.world.partition.unregister(id);
            }
            if let Some(obj) = self.world.objects.get_mut(slot) {
                obj.partition_last_look.reset();
                obj.partition_reveal_all_last_look.reset();
                obj.partition_last_shroud.reset();
            }
        }

        self.world.partition.store_fogged_cells(&mut store, FALSE);
        self.world.map_extent_hi = extent.hi;
        self.world.partition = PartitionGrid::new(cell_size, extent);
        self.world.partition.restore_fogged_cells(&store, FALSE);

        for &slot in &slots {
            if let Some(id) = self.world.objects.id_of(slot) {
                self.world.partition.register(id);
            }
            self.handle_partition_cell_maintenance(slot);
            self.update_off_map(slot);
        }

        self.world.partition.restore_fogged_cells(&store, TRUE);
    }

    /// Sets or clears `OFF_MAP` from the object's position: on the edge counts as off.
    fn update_off_map(&mut self, owner: ObjectSlot) {
        let extent = *self.world.partition.world_extents();
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let pos = obj.thing.get_position();
        let inside = extent.lo.x < pos.x
            && pos.x < extent.hi.x
            && extent.lo.y < pos.y
            && pos.y < extent.hi.y;
        if inside {
            obj.private_status &= !PRIVATE_STATUS_OFF_MAP;
        } else {
            obj.private_status |= PRIVATE_STATUS_OFF_MAP;
        }
    }

    /// `PartitionManager::update`: re-files the dirty objects, refreshes shroud for any
    /// whose center cell changed (`Object::onPartitionCellChange`), runs the contacts, and
    /// drains due delayed unlooks (`processPendingUndoShroudRevealQueue`). Call it after
    /// [`Self::step_sleepy_updates`] and before [`Self::process_destroy_list`].
    pub fn update_partition(&mut self) {
        let changed = {
            let SimContext { rules, world, .. } = self;
            let SimWorld {
                partition, objects, ..
            } = world;
            partition.update(&|id| partition_object(objects, &rules.thing_factory, id))
        };
        for id in changed {
            if let Some(slot) = self.world.objects.slot_by_id(id) {
                self.handle_partition_cell_maintenance(slot);
            }
        }
        process_contacts(self);
        let frame = self.world.frame;
        let players = self.partition_player_count();
        self.world
            .partition
            .process_pending_undo_shroud_reveals(frame, players);
    }

    /// Number of active player slots derived from `player_default_teams` (`max_index + 1`).
    pub fn partition_player_count(&self) -> usize {
        self.world
            .player_default_teams
            .keys()
            .copied()
            .filter(|&p| (0..MAX_PLAYER_COUNT as Int).contains(&p))
            .max()
            .map_or(0, |m| (m + 1) as usize)
    }

    fn controlling_player_index(&self, team_id: TeamId) -> Option<usize> {
        let idx = self.player_index_for_team(team_id);
        if (0..MAX_PLAYER_COUNT as Int).contains(&idx)
            && self.world.player_default_teams.contains_key(&idx)
        {
            Some(idx as usize)
        } else {
            None
        }
    }

    /// `PlayerList::getPlayersWithRelationship` derived from `player_default_teams`.
    fn players_with_relationship(
        &self,
        team_id: TeamId,
        allow_same: bool,
        allow_allies: bool,
        allow_enemies: bool,
        allow_neutral: bool,
    ) -> PlayerMaskType {
        if !allow_same && !allow_allies && !allow_enemies && !allow_neutral {
            return 0;
        }
        let Some(src_idx) = self.controlling_player_index(team_id) else {
            return 0;
        };
        let mut mask: PlayerMaskType = 0;
        if allow_same {
            mask |= 1u16 << (src_idx as u32);
        }
        for (&idx, &def_team) in &self.world.player_default_teams {
            if !(0..MAX_PLAYER_COUNT as Int).contains(&idx) || idx as usize == src_idx {
                continue;
            }
            let target_player = self.player_index_for_team(def_team);
            let include = match self.team_relationship_to_player(team_id, target_player) {
                Relationship::Enemies => allow_enemies,
                Relationship::Allies => allow_allies,
                Relationship::Neutral => allow_neutral,
            };
            if include {
                mask |= 1u16 << (idx as u32);
            }
        }
        mask
    }

    /// `Object::look`.
    fn object_look(&mut self, owner: ObjectSlot) {
        let Some((
            team_id,
            last_invalid,
            destroyed,
            eff_dead,
            contained_by,
            pos,
            vision_spied_mask,
            under_construction,
            stealthed_and_not_detected,
            reveal_all_range,
        )) = self.world.objects.get(owner).map(|o| {
            let reveal_all_range = o
                .thing
                .template_override()
                .get_final(&self.rules.thing_factory)
                .map_or(0.0, |t| t.get_shroud_reveal_to_all_range());
            let stealthed = o.status.test(ObjectStatusType::Stealthed.as_i32()) != FALSE;
            let detected = o.status.test(ObjectStatusType::Detected.as_i32()) != FALSE;
            let disguised = o.status.test(ObjectStatusType::Disguised.as_i32()) != FALSE;
            (
                o.team_id,
                o.partition_last_look.is_invalid(),
                o.is_destroyed(),
                o.is_effectively_dead() != FALSE,
                o.contained_by,
                *o.thing.get_position(),
                o.vision_spied_mask,
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE,
                stealthed && !detected && !disguised,
                reveal_all_range,
            )
        })
        else {
            return;
        };

        if !last_invalid {
            return;
        }
        if self.controlling_player_index(team_id).is_none() {
            return;
        }
        if destroyed || eff_dead {
            return;
        }
        if let Some(container) = contained_by {
            let has_contain = self
                .world
                .objects
                .get(container)
                .is_some_and(|c| c.contain_slot.is_some());
            if has_contain && self.contain_is_garrisonable(container) == FALSE {
                return;
            }
        }

        let player_count = self.partition_player_count();
        let shroud_clearing_range = self.object_shroud_clearing_range(owner);
        if shroud_clearing_range > 0.0 {
            let looking_mask = if self.object_is_kind_of(owner, KindOfType::RevealToAll) != FALSE {
                PLAYERMASK_ALL
            } else {
                let mut mask: PlayerMaskType = 0;
                for (&idx, &def_team) in self.world.player_default_teams.iter().rev() {
                    if !(0..MAX_PLAYER_COUNT as Int).contains(&idx) {
                        continue;
                    }
                    let target_player = self.player_index_for_team(def_team);
                    if self.team_relationship_to_player(team_id, target_player)
                        == Relationship::Allies
                    {
                        mask |= 1u16 << (idx as u32);
                    }
                }
                mask | vision_spied_mask
            };
            self.world.partition.do_shroud_reveal(
                pos.x,
                pos.y,
                shroud_clearing_range,
                looking_mask,
                player_count,
            );
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.partition_last_look.center = pos;
                obj.partition_last_look.for_whom = looking_mask;
                obj.partition_last_look.how_far = shroud_clearing_range;
            }
        }

        if reveal_all_range > 0.0 && !under_construction && !stealthed_and_not_detected {
            let the_players_mask =
                self.players_with_relationship(team_id, false, false, true, true);
            self.world.partition.do_shroud_reveal(
                pos.x,
                pos.y,
                reveal_all_range,
                the_players_mask,
                player_count,
            );
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.partition_reveal_all_last_look.center = pos;
                obj.partition_reveal_all_last_look.for_whom = the_players_mask;
                obj.partition_reveal_all_last_look.how_far = reveal_all_range;
            }
        }
    }

    /// `Object::unlook`.
    fn object_unlook(&mut self, owner: ObjectSlot) {
        let Some((last_look, reveal_all)) = self.world.objects.get(owner).map(|o| {
            (
                o.partition_last_look.clone(),
                o.partition_reveal_all_last_look.clone(),
            )
        }) else {
            return;
        };
        if last_look.is_invalid() {
            return;
        }
        let frame = self.world.frame;
        let persist = self.rules.global_data.unlook_persist_duration;
        self.world.partition.queue_undo_shroud_reveal(
            last_look.center.x,
            last_look.center.y,
            last_look.how_far,
            last_look.for_whom,
            frame,
            persist,
        );
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.partition_last_look.reset();
        }
        if !reveal_all.is_invalid() {
            self.world.partition.queue_undo_shroud_reveal(
                reveal_all.center.x,
                reveal_all.center.y,
                reveal_all.how_far,
                reveal_all.for_whom,
                frame,
                persist,
            );
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.partition_reveal_all_last_look.reset();
            }
        }
    }

    /// `Object::shroud`.
    fn object_shroud(&mut self, owner: ObjectSlot) {
        let Some((team_id, last_invalid, under_construction, eff_dead, shroud_range, pos)) =
            self.world.objects.get(owner).map(|o| {
                (
                    o.team_id,
                    o.partition_last_shroud.is_invalid(),
                    o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE,
                    o.is_effectively_dead() != FALSE,
                    o.shroud_range,
                    *o.thing.get_position(),
                )
            })
        else {
            return;
        };
        if !last_invalid {
            return;
        }
        if self.controlling_player_index(team_id).is_none() {
            return;
        }
        if !under_construction && !eff_dead && shroud_range > 0.0 {
            let mut shrouding_mask: PlayerMaskType = 0;
            for (&idx, &def_team) in self.world.player_default_teams.iter().rev() {
                if !(0..MAX_PLAYER_COUNT as Int).contains(&idx) {
                    continue;
                }
                let target_player = self.player_index_for_team(def_team);
                if self.team_relationship_to_player(team_id, target_player) != Relationship::Allies
                {
                    shrouding_mask |= 1u16 << (idx as u32);
                }
            }
            let player_count = self.partition_player_count();
            self.world.partition.do_shroud_cover(
                pos.x,
                pos.y,
                shroud_range,
                shrouding_mask,
                player_count,
            );
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.partition_last_shroud.center = pos;
                obj.partition_last_shroud.for_whom = shrouding_mask;
                obj.partition_last_shroud.how_far = shroud_range;
            }
        }
    }

    /// `Object::unshroud`.
    fn object_unshroud(&mut self, owner: ObjectSlot) {
        let Some(last) = self
            .world
            .objects
            .get(owner)
            .map(|o| o.partition_last_shroud.clone())
        else {
            return;
        };
        if last.is_invalid() {
            return;
        }
        let player_count = self.partition_player_count();
        self.world.partition.undo_shroud_cover(
            last.center.x,
            last.center.y,
            last.how_far,
            last.for_whom,
            player_count,
        );
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.partition_last_shroud.reset();
        }
    }

    /// `Object::handleShroud`.
    fn handle_shroud(&mut self, owner: ObjectSlot) {
        self.object_unlook(owner);
        self.object_unshroud(owner);
        self.object_shroud(owner);
        self.object_look(owner);
    }

    /// `PartitionManager::revealMapForPlayerPermanently`.
    pub fn reveal_map_for_player_permanently(&mut self, team_id: TeamId) {
        let idx = self.player_index_for_team(team_id);
        if (0..MAX_PLAYER_COUNT as Int).contains(&idx) {
            self.world
                .partition
                .reveal_map_for_player_permanently(idx as usize);
        }
    }

    /// `PartitionManager::undoRevealMapForPlayerPermanently`.
    pub fn undo_reveal_map_for_player_permanently(&mut self, team_id: TeamId) {
        let idx = self.player_index_for_team(team_id);
        if (0..MAX_PLAYER_COUNT as Int).contains(&idx) {
            let players = self.partition_player_count();
            self.world
                .partition
                .undo_reveal_map_for_player_permanently(idx as usize, players);
        }
    }

    /// `PartitionManager::shroudMapForPlayer`.
    pub fn shroud_map_for_player(&mut self, team_id: TeamId) {
        let idx = self.player_index_for_team(team_id);
        if (0..MAX_PLAYER_COUNT as Int).contains(&idx) {
            let players = self.partition_player_count();
            self.world
                .partition
                .shroud_map_for_player(idx as usize, players);
        }
    }

    /// `Team::setControllingPlayer`: updates the team's controlling player and
    /// refreshes shroud (or runs `onCapture` in non-retail builds) for all members.
    pub fn set_team_controlling_player(&mut self, team_id: TeamId, new_player: Int) {
        let old_player = self.player_index_for_team(team_id);
        self.world.team_players.insert(team_id, new_player);
        let slots: Vec<ObjectSlot> = self
            .world
            .team_members
            .get(&team_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|&slot| {
                self.world
                    .objects
                    .get(slot)
                    .is_some_and(|o| o.team_id == team_id)
            })
            .collect();
        #[cfg(feature = "retail-crc")]
        let _ = old_player;
        #[cfg(not(feature = "retail-crc"))]
        let (old_owner_team, new_owner_team) = (
            self.world
                .player_default_teams
                .get(&old_player)
                .copied()
                .unwrap_or(team_id),
            self.world
                .player_default_teams
                .get(&new_player)
                .copied()
                .unwrap_or(team_id),
        );
        for slot in slots {
            #[cfg(feature = "retail-crc")]
            self.handle_partition_cell_maintenance(slot);
            #[cfg(not(feature = "retail-crc"))]
            {
                let beh_slots: Vec<ModuleSlot> = self
                    .world
                    .objects
                    .get(slot)
                    .map_or_else(Vec::new, |o| o.behaviors.iter().map(|&(ms, _)| ms).collect());
                if old_player != new_player
                    && self.world.player_relationships.get(&(old_player, new_player))
                        != Some(&Relationship::Allies)
                {
                    self.ai_idle(slot, CommandSourceType::FromAi);
                }
                *self
                    .world
                    .player_captured_objects
                    .entry(new_owner_team)
                    .or_insert(0) += 1;
                self.record_object_captured(slot, new_owner_team);
                for ms in beh_slots {
                    if self.world.modules.is_alive(ms) {
                        let _ = self.dispatch_module(ms, |m, sim, ctx| {
                            m.on_capture(sim, ctx, old_owner_team, new_owner_team);
                        });
                    }
                }
                self.handle_partition_cell_maintenance(slot);
                self.clear_unsellable(slot);
            }
        }
    }

    fn search_partition(
        &self,
        center: QueryCenter<'_>,
        radius: Real,
        rule: DistanceCalculationType,
    ) -> Vec<ObjectSlot> {
        let objects = &self.world.objects;
        let factory = &self.rules.thing_factory;
        self.world
            .partition
            .iterate_objects_in_range(
                center,
                radius,
                rule,
                IterOrderType::Fastest,
                &|id| partition_object(objects, factory, id),
                &mut |_| true,
            )
            .into_iter()
            .filter_map(|(id, _)| objects.slot_by_id(id))
            .collect()
    }

    fn search_partition_from_object(
        &self,
        owner: ObjectSlot,
        radius: Real,
        rule: DistanceCalculationType,
    ) -> Vec<ObjectSlot> {
        let Some(me) = self.world.objects.id_of(owner).and_then(|id| {
            partition_object(&self.world.objects, &self.rules.thing_factory, id)
        }) else {
            return Vec::new();
        };
        self.search_partition(QueryCenter::Object(&me), radius, rule)
    }

    /// `Object::isAbleToAttack`.
    fn is_able_to_attack(&self, owner: ObjectSlot) -> bool {
        let Some(me) = self.world.objects.get(owner) else {
            return false;
        };
        let status = |s: ObjectStatusType| me.status.test(s.as_i32()) != FALSE;
        let kind = |k: KindOfType| self.object_is_kind_of(owner, k) != FALSE;
        let disabled = |d: DisabledType| me.is_disabled_by_type(d) != FALSE;
        if status(ObjectStatusType::NoAttack) {
            return false;
        }
        if let Some(container) = me.contained_by {
            if self.object_has_contain(container) != FALSE
                && self.contain_is_passenger_allowed_to_fire(container, me.id) == FALSE
            {
                return false;
            }
        }
        if status(ObjectStatusType::UnderConstruction)
            || status(ObjectStatusType::Sold)
            || disabled(DisabledType::Subdued)
        {
            return false;
        }

        if kind(KindOfType::PortableStructure) || kind(KindOfType::SpawnsAreTheWeapons) {
            if disabled(DisabledType::Hacked) || disabled(DisabledType::Emp) {
                return false;
            }
            // A stinger soldier is subdued with its site.
            if kind(KindOfType::Infantry) {
                let slaver = me
                    .slaved_slot
                    .and_then(|ms| self.world.modules.payload(ms)?.as_slaved_update())
                    .map(|slaved| slaved.get_slaver_id())
                    .and_then(|id| self.world.objects.slot_by_id(id))
                    .and_then(|slot| self.world.objects.get(slot));
                if slaver.is_some_and(|o| o.is_disabled_by_type(DisabledType::Subdued) != FALSE) {
                    return false;
                }
            }
        }

        let ai = me
            .ai_slot
            .and_then(|ms| self.world.modules.payload(ms)?.as_ai());
        // Every weapon on a disabled turret means no attack.
        if let Some(ai) = ai {
            if !kind(KindOfType::CanAttack) {
                let mut any_weapon = false;
                let mut any_enabled = false;
                for slot in [
                    WeaponSlotType::Primary,
                    WeaponSlotType::Secondary,
                    WeaponSlotType::Tertiary,
                ] {
                    if me.weapon_set.get_weapon_in_weapon_slot(slot).is_none() {
                        continue;
                    }
                    any_weapon = true;
                    if ai.is_weapon_slot_enabled(slot) != FALSE {
                        any_enabled = true;
                        break;
                    }
                }
                if any_weapon && !any_enabled {
                    return false;
                }
            }
        }

        if kind(KindOfType::CanAttack) || status(ObjectStatusType::CanAttack) {
            return true;
        }
        if self.object_has_contain(owner) != FALSE
            && self.contain_is_passenger_allowed_to_fire(owner, me.id) != FALSE
            && self.contain_count(owner) > 0
        {
            return true;
        }
        if me.ai_slot.is_some() && me.weapon_set.has_any_weapon() != FALSE {
            return true;
        }
        let slaves_attack = me
            .spawn_slot
            .and_then(|ms| self.world.modules.payload(ms)?.as_spawn_behavior())
            .is_some_and(|spawn| spawn.can_any_slaves_attack(self) != FALSE);
        if slaves_attack {
            return true;
        }
        me.thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .is_some_and(|t| t.is_enter_guard() != FALSE)
    }

    /// The bit change of `Object::setStatus`. A change of `UnderConstruction`
    /// re-files the object (it changes what the object can see). The mine
    /// detonation the original runs there is not ported.
    fn change_status(&mut self, owner: ObjectSlot, status: ObjectStatusType, value: Int) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let was_set = obj.status.test(status.as_i32()) != FALSE;
        obj.status.set(status.as_i32(), value);
        let id = obj.id;
        let changed = was_set != (value != 0);
        if changed
            && matches!(
                status,
                ObjectStatusType::Stealthed
                    | ObjectStatusType::Detected
                    | ObjectStatusType::Disguised
            )
        {
            let reveal_all = obj
                .thing
                .template_override()
                .get_final(&self.rules.thing_factory)
                .map_or(0.0, |t| t.get_shroud_reveal_to_all_range());
            if reveal_all > 0.0 {
                self.handle_partition_cell_maintenance(owner);
            }
        }
        if status == ObjectStatusType::UnderConstruction && changed {
            self.world.partition.make_dirty(id);
        }
    }

    /// The `aiIdle` at the top of `Object::onCapture`. Retail always idles on a change of owner;
    /// later builds let a unit carry on when the new owner is an ally.
    fn idle_on_capture(&mut self, owner: ObjectSlot, old_team: TeamId, new_team: TeamId) {
        let old_player = self.player_index_for_team(old_team);
        let new_player = self.player_index_for_team(new_team);
        if old_player == new_player {
            return;
        }
        #[cfg(not(feature = "retail-crc"))]
        if self.world.player_relationships.get(&(old_player, new_player))
            == Some(&Relationship::Allies)
        {
            return;
        }
        self.ai_idle(owner, CommandSourceType::FromAi);
    }

    fn clear_unsellable(&mut self, owner: ObjectSlot) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.script_status &= !OBJECT_STATUS_SCRIPT_UNSELLABLE;
        }
    }

    fn behavior_slots(&self, owner: ObjectSlot) -> Vec<ModuleSlot> {
        self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors.iter().map(|&(ms, _)| ms).collect()
        })
    }

    fn extend_rods(&mut self, owner: ObjectSlot, extend: Bool) {
        for ms in self.behavior_slots(owner) {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(f) = m.as_power_plant_mut() {
                    f.extend_rods(sim, ctx, extend);
                }
            });
        }
    }

    /// `Object::getDockUpdateInterface`: the first behavior with a dock facet. A dock
    /// whose module is executing is not found.
    pub fn dock_slot(&self, owner: ObjectSlot) -> Option<ModuleSlot> {
        self.world.objects.get(owner)?.behaviors.iter().find_map(|&(ms, mask)| {
            (mask.contains(FacetMask::DOCK_UPDATE)
                && self.world.modules.payload(ms)?.as_dock().is_some())
            .then_some(ms)
        })
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
                match op {
                    DeferredSelfOp::Delete => payload.on_delete(self, ctx),
                    other => payload.apply_deferred_op(other, self, ctx),
                }
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

    /// `Object::reactToTransformChange`: re-file the owner in the partition, tell its
    /// contain module that it moved, and on a change of position update `OFF_MAP`.
    fn react_to_transform_change(&mut self, owner: ObjectSlot, old_pos: &Coord3D, old_angle: Real) {
        const THRESH: Real = TRANSFORM_THRESHOLD;
        let Some(obj) = self.world.objects.get(owner) else {
            return;
        };
        let pos = obj.thing.get_position();
        let moved = zh_common::math::gm_fabsf(pos.x - old_pos.x) > THRESH
            || zh_common::math::gm_fabsf(pos.y - old_pos.y) > THRESH
            || zh_common::math::gm_fabsf(pos.z - old_pos.z) > THRESH;
        let turned = zh_common::math::gm_fabsf(obj.thing.get_orientation() - old_angle) > THRESH;
        if !moved && !turned {
            return;
        }
        let contain_slot = obj.contain_slot;
        let id = obj.id;
        self.world.partition.make_dirty(id);
        if let Some(cs) = contain_slot {
            if !self.world.modules.is_executing(cs) {
                let _ = self.dispatch_module(cs, |m, sim, ctx| {
                    if let Some(c) = m.as_contain_mut() {
                        c.contain_react_to_transform_change(sim, ctx);
                    }
                });
            }
        }
        if moved {
            self.update_off_map(owner);
        }
    }

    fn add_experience_to(&mut self, owner: ObjectSlot, gain: Int) {
        let Some((sink, scalar, tmpl_ov, obj_id, body_slot)) =
            self.world.objects.get(owner).map(|o| {
                (
                    o.experience_tracker.experience_sink,
                    o.experience_tracker.experience_scalar,
                    o.thing.template_override(),
                    o.id,
                    o.body_slot,
                )
            })
        else {
            return;
        };
        if sink != ObjectId::INVALID {
            if let Some(sink_slot) = self.world.objects.slot_by_id(sink) {
                let scaled = zh_common::math::cpp_mul_i32_f32(gain, scalar);
                self.add_experience_to(sink_slot, scaled);
                return;
            }
        }
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
            obj.experience_tracker
                .add_experience_points(&mut parent, gain, TRUE);
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

    fn detonate_booby_trap(&mut self, owner: ObjectSlot, victim: ObjectSlot) -> Bool {
        const BOOBY_TRAP_SCAN_RANGE: Real = 25.0;
        let Some((id, pos, radius, off_map, trapped)) = self.world.objects.get(owner).map(|o| {
            (
                o.id,
                *o.thing.get_position(),
                o.geometry.get_bounding_circle_radius(),
                o.private_status & PRIVATE_STATUS_OFF_MAP,
                o.status.test(ObjectStatusType::BoobyTrapped.as_i32()),
            )
        }) else {
            return FALSE;
        };
        if trapped == FALSE {
            return FALSE;
        }
        let trap = self
            .find_objects_near_to_far(&pos, BOOBY_TRAP_SCAN_RANGE + radius)
            .into_iter()
            .find(|&s| {
                self.object_is_kind_of(s, KindOfType::BoobyTrap) != FALSE
                    && self.world.objects.get(s).is_some_and(|o| {
                        o.private_status & PRIVATE_STATUS_OFF_MAP == off_map && o.producer_id == id
                    })
            });
        let Some(trap) = trap else {
            return FALSE;
        };
        if self.object_has_template_module(trap, "StickyBombUpdate") == FALSE {
            return FALSE;
        }
        let teams = self
            .world
            .objects
            .get(trap)
            .zip(self.world.objects.get(victim))
            .map(|(t, v)| (t.team_id, v.team_id));
        if let Some((trap_team, victim_team)) = teams {
            let victim_player = self.player_index_for_team(victim_team);
            if self.team_relationship_to_player(trap_team, victim_player) == Relationship::Allies {
                return FALSE;
            }
        }
        self.sticky_bomb_detonate(trap)
    }

    fn special_power_at_object(
        &mut self,
        owner: ObjectSlot,
        template_id: UnsignedInt,
        target: ObjectSlot,
        command_options: UnsignedInt,
        forced: Bool,
    ) {
        let Some(obj) = self.world.objects.get(owner) else {
            return;
        };
        if obj.is_disabled() != FALSE {
            return;
        }
        let module = self.find_special_power_module(owner, template_id);
        if forced == FALSE {
            let store = &self.rules.special_power_store;
            let user = PowerUser {
                sim: self,
                team_id: obj.team_id,
                has_module: module.is_some(),
            };
            let template = store.find_special_power_template_by_id(template_id);
            if store.can_use_special_power(Some(&user), template) == FALSE {
                return;
            }
        }
        if let Some(ms) = module {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(sp) = m.as_special_power_mut() {
                    sp.do_special_power_at_object(sim, ctx, target, command_options);
                }
            });
        }
    }

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
        // `Object::onDestroy`: say goodbye to whoever contains us, notify modules (`on_delete`), then unlook/unshroud.
        if let Some(container) = self.world.objects.get(slot).and_then(|o| o.contained_by) {
            self.contain_remove(container, slot, FALSE);
        }
        let beh_slots: Vec<ModuleSlot> = self
            .world
            .objects
            .get(slot)
            .map_or_else(Vec::new, |o| o.behaviors.iter().map(|&(ms, _)| ms).collect());
        for &ms in &beh_slots {
            if !self.world.modules.is_alive(ms) {
                continue;
            }
            if self.world.modules.is_executing(ms) {
                self.world
                    .modules
                    .push_deferred_op(ms, DeferredSelfOp::Delete);
            } else {
                let child_ids = self
                    .world
                    .modules
                    .payload(ms)
                    .map_or_else(Vec::new, |m| m.delete_child_objects());
                for id in child_ids {
                    if let Some(child_slot) = self.world.objects.slot_by_id(id) {
                        self.destroy_object_now(child_slot);
                    }
                }
                if self.world.modules.is_alive(ms) {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        m.on_delete(sim, ctx);
                    });
                }
            }
        }
        self.handle_partition_cell_maintenance(slot);
        self.client.on_object_destroyed(obj_id);
    }

    /// Destroys all queued objects (`GameLogic::processDestroyList`), running two-pass heap removal
    /// and arena despawn (`on_delete` already ran inside `destroy_object_now` / `Object::onDestroy`).
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

            let modules = &mut self.world.modules;
            self.world
                .sleepy_updates
                .erase_for_owner(modules, |m, ms| m.meta(ms).owner_slot == slot);

            // `Object::~Object` leaves its team before it deletes the modules.
            if let Some(team) = self.world.objects.get(slot).map(|o| o.team_id) {
                self.adjust_team_power(slot, team, FALSE);
            }

            for &ms in &beh_slots {
                if self.world.modules.is_alive(ms) {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        m.on_destruct(sim, ctx);
                    });
                }
            }

            for ms in beh_slots {
                let _ = self.world.modules.free(ms);
            }
            // `Object::~Object`.
            if let Some(id) = self.world.objects.id_of(slot) {
                self.world.partition.unregister(id);
            }
            if let Some(team) = self.world.objects.get(slot).map(|o| o.team_id) {
                self.world.team_member_remove(team, slot);
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
            let resident = self.world.modules.payload(u).and_then(|m| m.resident());
            let proc_bits = match &resident {
                Some(entries) => entries.disabled_types_to_process.to_unsigned_int() as u64,
                None => self
                    .world
                    .modules
                    .payload(u)
                    .and_then(|m| m.as_update())
                    .map_or(0, |upd| upd.disabled_types_to_process().to_unsigned_int() as u64),
            };

            let mut sleep_len = UpdateSleepTime::NONE;
            if should_process_disabled_update(disabled_bits, proc_bits) {
                self.world.sleepy_updates.set_cur_update_module(Some(u));
                if let Some(entries) = resident {
                    let ctx = self.world.modules.meta(u).context(u);
                    sleep_len = (entries.update)(self, ctx);
                } else if let Ok(ret) = self.dispatch_module(u, |m, sim, ctx| {
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
    fn build_first_weapon_set(&mut self, owner: ObjectSlot) {
        let frame = self.world.frame;
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        if obj.weapon_set.cur_weapon_template_set.is_some() {
            return;
        }
        let set = obj
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .and_then(|t| t.find_weapon_template_set(&obj.weapon_set_flags));
        if let Some(set) = set {
            obj.weapon_set.init_from_template_set(set, frame);
        }
    }

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
                let vision_range = t.friend_calc_vision_range();
                let mut shroud_clearing_range = t.friend_calc_shroud_clearing_range();
                if shroud_clearing_range == -1.0 {
                    shroud_clearing_range = vision_range;
                }
                (
                    t.get_template_geometry_info().clone(),
                    t.is_trainable(),
                    t.get_behavior_module_info().nuggets().to_vec(),
                    t.find_weapon_template_set(&crate::rules::combat_types::WeaponSetFlags::new())
                        .cloned(),
                    vision_range,
                    shroud_clearing_range,
                )
            });
        let (geom, trainable, nuggets, wts, vision_range, shroud_clearing_range) = match tmpl_data
        {
            Some((g, tr, n, w, vr, scr)) => (g, tr, n, w, vr, scr),
            None => (GeometryInfo::default(), FALSE, Vec::new(), None, 0.0, 0.0),
        };

        let mut record = ObjectRecord::new(id, tmpl, team, geom);
        record.status = initial_status;
        record.experience_tracker.set_trainable(trainable);
        record.vision_range = vision_range;
        record.shroud_clearing_range = shroud_clearing_range;
        record.shroud_range = 0.0;
        let mut dummy_reactor = crate::object::thing::NoopTransformReactor;
        let has_template = self
            .rules
            .thing_factory
            .find_by_template_id_final(tmpl.0 as u16)
            .is_some();
        if has_template {
            // `setPosition` then `setOrientation`, so that the angle is stored as
            // given (normalized) rather than read back from the matrix.
            let factory = &self.rules.thing_factory;
            let terrain = self.terrain.as_ref();
            record
                .thing
                .set_position(factory, &pos, terrain, &mut dummy_reactor);
            record
                .thing
                .set_orientation(factory, angle, terrain, &mut dummy_reactor);
        } else {
            // A thing without a template has no kind-of bits to consult.
            let mut mtx = zh_common::math::Matrix3D::identity();
            self.terrain.align_on_terrain(angle, &pos, FALSE, &mut mtx);
            record.thing.set_transform_matrix(&mtx, &mut dummy_reactor);
        }
        if !has_template {
            let slot = self.world.objects.spawn(id, record, true);
            self.world.team_member_prepend(team, slot);
            self.world.partition.register(id);
            self.client.on_object_created(id, tmpl);
            return slot;
        }

        // Attach helpers + INI behavior modules while unlinked so constructor initial
        // sleep frames are preserved when pushing UPDATE facets into SleepyUpdateHeap.
        let slot = self.world.objects.spawn(id, record, false);
        self.world.team_member_prepend(team, slot);
        self.adjust_team_power(slot, team, TRUE);
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
            let mod_slot = self.world.attach_module(
                slot,
                nug.tag_key,
                nug.data_id,
                desc.facet_mask,
                desc.update_phase,
                module,
            );
            let _ = self.dispatch_module(mod_slot, |m, sim, ctx| {
                m.on_construct(sim, ctx);
            });
        }

        let behaviors: Vec<(ModuleSlot, FacetMask)> = self
            .world
            .objects
            .get(slot)
            .map_or_else(Vec::new, |o| o.behaviors.clone());
        // Run `on_object_created` while unlinked (matching C++ `Object::Object`) so
        // constructor-equivalent `set_wake_frame` calls set `next_call_frame` prior to
        // `register_update_module`.
        for &(ms, _) in &behaviors {
            if self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    m.on_object_created(sim, ctx);
                });
            }
        }
        self.world.objects.link_into_world(slot);
        // `sendObjectCreated` binds the drawable after the modules' `onObjectCreated`:
        // `friend_bindToDrawable` gives it the weapon set flags held by then.
        if let Some(obj) = self.world.objects.get_mut(slot) {
            for i in 0..WeaponSetType::Count as Int {
                let Some(flag) = WeaponSetType::from_i32(i) else {
                    continue;
                };
                if let Some(condition) = weapon_set_model_condition(flag) {
                    if obj.weapon_set_flags.test(i) != FALSE {
                        obj.set_model_condition_state(condition);
                    } else {
                        obj.clear_model_condition_state(condition);
                    }
                }
            }
        }
        for &(ms, mask) in &behaviors {
            if mask.contains(FacetMask::UPDATE) {
                self.world.sleepy_updates.register_update_module(
                    &mut self.world.modules,
                    ms,
                    self.world.frame,
                );
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

        // `ThingFactory::newObject` registers with the partition after the create
        // modules and before `initObject`.
        self.world.partition.register(id);

        // `initObject`: a new object takes its player's active battle plan bonuses.
        battle_plan_update::battle_plan_team_member(self, team, slot, TRUE);

        // C++ builds the weapon set last (`updateWeaponSet` closes `initObject`), so module
        // constructors neither see its weapons nor share a reload with them.
        if let Some(ref set) = wts {
            let frame = self.world.frame;
            if let Some(o) = self.world.objects.get_mut(slot) {
                // A weapon set flag set while the object was being made built it already.
                if o.weapon_set.cur_weapon_template_set.is_none() {
                    o.weapon_set.init_from_template_set(set, frame);
                }
            }
        }

        // The original creates the object at the origin and positions it afterwards, so
        // `OFF_MAP` is only evaluated for a position that differs from the origin.
        let placed = self.world.objects.get(slot).is_some_and(|o| {
            let p = o.thing.get_position();
            [p.x, p.y, p.z]
                .iter()
                .any(|&v| zh_common::math::gm_fabsf(v) > TRANSFORM_THRESHOLD)
        });
        if placed {
            self.update_off_map(slot);
        }

        self.client.on_object_created(id, tmpl);
        slot
    }

    /// Grants a player-level upgrade (`Player::addUpgrade`) and refreshes all live
    /// objects belonging to `team_id`.
    pub fn grant_player_upgrade(&mut self, team_id: TeamId, upgrade_name: &str) {
        let Some(upgrade_mask) = self
            .rules
            .upgrade_center
            .find_upgrade(upgrade_name, &mut self.rules.name_keys)
            .map(|ut| ut.upgrade_mask().clone())
        else {
            return;
        };
        let empty_mask = UpgradeMaskType::new();
        if let Some(pending) = self.world.player_upgrades_in_progress.get_mut(&team_id) {
            pending.clear_and_set(&upgrade_mask, &empty_mask);
        }
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
            self.update_upgrade_modules(pslot);
        }
    }

    fn energy_object_info(&self, owner: ObjectSlot) -> Option<EnergyObjectInfo> {
        let obj = self.world.objects.get(owner)?;
        let tmpl = obj
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)?;
        Some(EnergyObjectInfo {
            energy_production: tmpl.get_energy_production(),
            energy_bonus: tmpl.get_energy_bonus(),
            is_disabled: obj.is_disabled(),
        })
    }

    /// The energy half of `Player::becomingTeamMember`, which every `Object::setTeam`
    /// runs (also at construction and destruction): unless under construction,
    /// `Object::friend_adjustPowerForPlayer`. A disabled producer is not counted.
    /// `Player::onPowerBrownOutChange` is unported.
    fn adjust_team_power(&mut self, owner: ObjectSlot, team_id: TeamId, incoming: Bool) {
        let under_construction = self.world.objects.get(owner).is_none_or(|o| {
            o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
        });
        let Some(info) = self.energy_object_info(owner) else {
            return;
        };
        if under_construction
            || team_id == TeamId::INVALID
            || info.energy_production == 0
            || (info.is_disabled != FALSE && info.energy_production > 0)
        {
            return;
        }
        let frame = self.world.frame;
        let energy = self.world.player_energy.entry(team_id).or_default();
        if incoming != FALSE {
            energy.object_entering_influence(Some(info), frame, &mut NoopEnergyCallbacks);
        } else {
            energy.object_leaving_influence(Some(info), frame, &mut NoopEnergyCallbacks);
        }
    }

    /// `Object::onDisabledEdge`: every behaviour, then the power adjustment.
    fn on_disabled_edge(&mut self, owner: ObjectSlot, becoming_disabled: Bool) {
        let behaviors: Vec<ModuleSlot> = self
            .world
            .objects
            .get(owner)
            .map_or_else(Vec::new, |o| o.behaviors.iter().map(|&(ms, _)| ms).collect());
        for ms in behaviors {
            if self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    m.on_disabled_edge(sim, ctx, becoming_disabled);
                });
            }
        }
        self.adjust_power_on_disabled_edge(owner, becoming_disabled);
    }

    /// `Object::pauseAllSpecialPowers`.
    fn pause_all_special_powers(&mut self, owner: ObjectSlot, pause: Bool) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::SPECIAL_POWER))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
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
    }

    /// `WeaponSet::weaponSetOnWeaponBonusChange`.
    fn weapon_set_on_weapon_bonus_change(&mut self, owner: ObjectSlot) {
        for wslot in [
            WeaponSlotType::Primary,
            WeaponSlotType::Secondary,
            WeaponSlotType::Tertiary,
        ] {
            crate::object::weapon_fire::on_weapon_bonus_change(self, owner, wslot);
        }
    }

    fn adjust_power_on_disabled_edge(&mut self, owner: ObjectSlot, becoming_disabled: Bool) {
        let Some((team_id, under_const, power)) = self.world.objects.get(owner).and_then(|o| {
            let tmpl = o
                .thing
                .template_override()
                .get_final(&self.rules.thing_factory)?;
            Some((
                o.team_id,
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE,
                tmpl.get_energy_production(),
            ))
        }) else {
            return;
        };
        #[cfg(feature = "retail-crc")]
        let _ = under_const;
        #[cfg(not(feature = "retail-crc"))]
        if under_const {
            return;
        }
        if power > 0 {
            self.adjust_player_power(team_id, power, Bool::from(becoming_disabled == FALSE));
        }
    }

    fn select_and_trigger_slow_death(
        &mut self,
        owner: ObjectSlot,
        caller_slot: Option<ModuleSlot>,
        caller_applicable: Bool,
        caller_prob: Int,
        dmg: &DamageInfo,
    ) -> Bool {
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
            if caller_slot == Some(ms) {
                if caller_applicable != FALSE {
                    total = total.wrapping_add(caller_prob);
                    applicable.push((ms, caller_prob));
                }
            } else if let Ok(Some(w)) = self.dispatch_module(ms, |m, sim, ctx| {
                let sd = m.as_slow_death()?;
                if sd.is_die_applicable(sim, ctx, dmg) != FALSE {
                    Some(sd.get_probability_modifier(sim, ctx, dmg))
                } else {
                    None
                }
            }) {
                total = total.wrapping_add(w);
                applicable.push((ms, w));
            }
        }
        if total <= 0 {
            return FALSE;
        }
        let mut roll = self.rng_logic_int(1, total);
        for (ms, w) in applicable {
            roll = roll.wrapping_sub(w);
            if roll <= 0 {
                if caller_slot == Some(ms) {
                    return TRUE;
                }
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    if let Some(sd) = m.as_slow_death_mut() {
                        sd.begin_slow_death(sim, ctx, dmg);
                    }
                });
                return FALSE;
            }
        }
        FALSE
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
    fn contain_apparent_controlling_player(&self, container: ObjectSlot, observer: Int) -> Option<Int> {
        let ms = self.world.objects.get(container)?.contain_slot?;
        let contain = self.world.modules.payload(ms)?.as_contain()?;
        let ctx = self.world.modules.meta(ms).context(ms);
        contain.apparent_controlling_player_index(self, ctx, observer)
    }

    fn ai_is_idle(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot)
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_ai())
            .map_or(TRUE, |ai| ai.is_idle())
    }

    fn ai_is_moving(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot)
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_ai())
            .map_or(FALSE, |ai| ai.is_moving())
    }

    fn ai_attack_priority(&self, owner: ObjectSlot, victim: ObjectSlot) -> Option<Int> {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot)
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_ai())
            .and_then(|ai| ai.attack_priority(self, victim))
    }

    fn find_special_ability_update(&self, owner: ObjectSlot, power_type: SpecialPowerType) -> Option<ModuleSlot> {
        let obj = self.world.objects.get(owner)?;
        obj.behaviors
            .iter()
            .filter(|&&(_, mask)| mask.contains(FacetMask::SPECIAL_POWER_UPDATE))
            .map(|&(ms, _)| ms)
            .find(|&ms| {
                let data_id = self.world.modules.meta(ms).context(ms).template_data_id;
                self.module_data(data_id)
                    .and_then(|d| d.as_any().downcast_ref::<SpecialAbilityUpdateModuleData>())
                    .is_some_and(|d| d.power_type(self.rules()) == power_type)
            })
    }

    fn special_ability_is_active(&self, module: ModuleSlot) -> Bool {
        self.world
            .modules
            .payload(module)
            .and_then(|m| m.as_special_power_update())
            .map_or(FALSE, |f| f.is_active())
    }

    fn object_jet_ai(&self, owner: ObjectSlot) -> Option<JetAiState> {
        let obj = self.world.objects.get(owner)?;
        obj.behaviors.iter().find_map(|&(ms, _)| {
            self.world.modules.payload(ms)?.as_ai()?.jet_ai()
        })
    }

    fn ai_turret_for_cur_weapon(&self, owner: ObjectSlot) -> WhichTurretType {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return WhichTurretType::Invalid;
        };
        let Some(ai) = self.world.modules.payload(ms).and_then(|m| m.as_ai()) else {
            return WhichTurretType::Invalid;
        };
        let ctx = self.world.modules.meta(ms).context(ms);
        ai.which_turret_for_cur_weapon(self, ctx)
    }

    fn ai_turret_in_natural_position(&self, owner: ObjectSlot, turret: WhichTurretType) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot)
            .and_then(|ms| self.world.modules.payload(ms)?.as_ai())
            .map_or(FALSE, |ai| ai.is_turret_in_natural_position(turret))
    }

    fn player_battle_plans(&self, player: Int) -> Option<&PlayerBattlePlans> {
        self.world.player_battle_plans.get(&player)
    }

    fn special_power_module_ready(&self, owner: ObjectSlot, template_id: UnsignedInt) -> Option<(Bool, UnsignedInt)> {
        let ms = self.find_special_power_module(owner, template_id)?;
        let sp = self.world.modules.payload(ms)?.as_special_power()?;
        let ctx = self.world.modules.meta(ms).context(ms);
        Some((sp.is_ready(self, ctx), sp.get_ready_frame(self, ctx)))
    }

    fn action_manager_allows(&self, actor: ObjectSlot, target: ObjectSlot, query: ActionQuery) -> Bool {
        let ids = self.world.objects.id_of(actor).zip(self.world.objects.id_of(target));
        Bool::from(ids.is_some_and(|(a, t)| self.world.action_permissions.contains(&(a, t, query))))
    }

    fn is_clear_line_of_sight(&self, owner: ObjectSlot, other: ObjectSlot) -> Bool {
        let top = |slot| {
            self.world.objects.get(slot).map(|o| {
                let mut pos = *o.thing.get_position();
                pos.z += o.geometry.get_max_height_above_position();
                pos
            })
        };
        match (top(owner), top(other)) {
            (Some(a), Some(b)) => self.terrain.is_clear_line_of_sight(&a, &b),
            _ => FALSE,
        }
    }

    fn is_in_range_bounding_2d(&self, owner: ObjectSlot, target: ObjectSlot, radius: Real) -> Bool {
        Bool::from(
            self.search_partition_from_object(owner, radius, DistanceCalculationType::FromBoundingSphere2d)
                .contains(&target),
        )
    }

    fn find_objects_near_to_far(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot> {
        let objects = &self.world.objects;
        let factory = &self.rules.thing_factory;
        self.world
            .partition
            .iterate_objects_in_range(
                QueryCenter::Position(*center),
                radius,
                DistanceCalculationType::FromCenter2d,
                IterOrderType::SortedNearToFar,
                &|id| partition_object(objects, factory, id),
                &mut |_| true,
            )
            .into_iter()
            .filter_map(|(id, _)| objects.slot_by_id(id))
            .collect()
    }

    fn find_closest_team_object_of_kind(&self, center: &Coord3D, radius: Real, kind: KindOfType, team_id: TeamId) -> Option<ObjectSlot> {
        let objects = &self.world.objects;
        let factory = &self.rules.thing_factory;
        let player = self.player_index_for_team(team_id);
        let closest = self.world.partition.get_closest_object(
            QueryCenter::Position(*center),
            radius,
            DistanceCalculationType::FromCenter2d,
            &|id| partition_object(objects, factory, id),
            &mut |id| {
                objects.slot_by_id(id).is_some_and(|s| {
                    self.object_is_kind_of(s, kind) != FALSE
                        && objects
                            .get(s)
                            .is_some_and(|o| self.player_index_for_team(o.team_id) == player)
                })
            },
        );
        closest.id.and_then(|id| objects.slot_by_id(id))
    }

    fn object_has_template_module(&self, owner: ObjectSlot, name: &str) -> Bool {
        Bool::from(self.world.objects.get(owner).is_some_and(|o| {
            o.thing
                .template_override()
                .get_final(&self.rules.thing_factory)
                .is_some_and(|t| {
                    t.get_behavior_module_info()
                        .nuggets()
                        .iter()
                        .chain(t.get_client_update_module_info().nuggets())
                        .any(|n| n.name == name)
                })
        }))
    }

    fn command_set_button(&self, set_name: &str, index: usize) -> Option<CommandButtonId> {
        if let Some(over) = self
            .world
            .control_bar_overrides
            .get(&(set_name.to_owned(), index as Int))
        {
            return *over;
        }
        self.rules
            .command_store
            .find_command_set(set_name)
            .and_then(|set| set.buttons.get(index).copied().flatten())
    }

    fn object_command_set_string(&self, owner: ObjectSlot) -> String {
        let Some(o) = self.world.objects.get(owner) else {
            return String::new();
        };
        if !o.command_set_override.is_empty() {
            return o.command_set_override.clone();
        }
        o.thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .map_or_else(String::new, |t| t.friend_get_command_set_string().to_owned())
    }

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

    /// `a`'s `Object::getRelationship(b)`; the argument order matters.
    fn player_relationship(&self, a: ObjectSlot, b: ObjectSlot) -> Relationship {
        let Some(oa) = self.world.objects.get(a) else {
            return Relationship::Neutral;
        };
        let Some(ob) = self.world.objects.get(b) else {
            return Relationship::Neutral;
        };
        // An undetected defector auto-acquires nobody, and everybody takes it for their own.
        if oa.is_undetected_defector() != FALSE {
            return Relationship::Neutral;
        }
        if ob.is_undetected_defector() != FALSE {
            return Relationship::Allies;
        }
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

    fn weapon_bonus_condition_flags(&self, owner: ObjectSlot) -> UnsignedInt {
        let Some(obj) = self.world.objects.get(owner) else {
            return 0;
        };
        let mut flags = obj.weapon_bonus_condition;
        if let Some(container) = obj.contained_by.and_then(|c| self.world.objects.get(c)) {
            if container.contain_passes_weapon_bonus != FALSE {
                flags |= container.weapon_bonus_condition;
            }
        }
        flags
    }

    fn is_reload_time_shared(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .map_or(FALSE, |o| o.weapon_set.is_shared_reload_time())
    }

    fn weapon_barrel_count(&self, owner: ObjectSlot, wslot: WeaponSlotType) -> Int {
        self.world
            .objects
            .get(owner)
            .map_or(0, |o| o.weapon_barrel_count[wslot.as_i32() as usize])
    }

    fn last_shot_fired_frame(&self, owner: ObjectSlot) -> UnsignedInt {
        self.world.objects.get(owner).map_or(0, |o| {
            o.weapon_set
                .weapons
                .iter()
                .flatten()
                .fold(0, |recent, w| recent.max(w.last_fire_frame))
        })
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

    fn object_would_ignite(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|obj| {
                obj.behaviors.iter().find_map(|&(ms, _)| {
                    self.world
                        .modules
                        .payload(ms)
                        .and_then(|m| m.as_flammable())
                })
            })
            .map_or(FALSE, |f| f.would_ignite())
    }

    fn closest_would_ignite_object(&self, owner: ObjectSlot, range: Real) -> Option<ObjectSlot> {
        let objects = &self.world.objects;
        let factory = &self.rules.thing_factory;
        let me = objects
            .id_of(owner)
            .and_then(|id| partition_object(objects, factory, id))?;
        let found = self.world.partition.get_closest_object(
            QueryCenter::Object(&me),
            range,
            DistanceCalculationType::FromCenter3d,
            &|id| partition_object(objects, factory, id),
            &mut |id| {
                objects
                    .slot_by_id(id)
                    .is_some_and(|slot| self.object_would_ignite(slot) != FALSE)
            },
        );
        found.id.and_then(|id| objects.slot_by_id(id))
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

    fn object_health(&self, owner: ObjectSlot) -> Real {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return 0.0;
        };
        if let Some(b) = self.world.modules.payload(bs).and_then(|p| p.as_body()) {
            return b.get_health();
        }
        match self.world.modules.sidecar(bs) {
            ModuleSidecar::Body { health, .. } => health,
            _ => 0.0,
        }
    }

    fn object_body_damage_state(&self, owner: ObjectSlot) -> BodyDamageType {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.body_slot)
            .and_then(|bs| self.world.modules.payload(bs))
            .and_then(|p| p.as_body())
            .map_or(BodyDamageType::Pristine, |b| b.get_damage_state())
    }

    fn dock_is_open(&self, owner: ObjectSlot) -> Bool {
        self.dock_slot(owner)
            .and_then(|ds| self.world.modules.payload(ds))
            .and_then(|p| p.as_dock())
            .map_or(TRUE, |d| d.is_dock_open())
    }

    fn stealth_is_temporary_grant(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.stealth_slot)
            .and_then(|slot| self.world.modules.payload(slot))
            .and_then(|m| m.as_stealth())
            .and_then(|st| st.stealth_params())
            .map_or(FALSE, |p| p.temporary_grant)
    }

    fn player_is_playable_side(&self, team_id: TeamId) -> Bool {
        Bool::from(self.world.playable_side_teams.contains(&team_id))
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
        if self.world.objects.get(victim).is_none() {
            return FALSE;
        }
        // An unmanned vehicle crushes nobody; its riders were just booted out around it.
        if oc.is_disabled_by_type(DisabledType::Unmanned) != FALSE {
            return FALSE;
        }
        // Order matters: whether the crusher considers the victim an ally.
        if self.player_relationship(crusher, victim) == Relationship::Allies {
            return FALSE;
        }
        let level = self.object_crusher_level(crusher) as u8;
        let crushable = self.object_crushable_level(victim) as u8;
        Bool::from(level != 0 && level > crushable)
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
        self.search_partition(
            QueryCenter::Position(*center),
            radius,
            DistanceCalculationType::FromCenter2d,
        )
    }

    fn find_potential_collisions(
        &self,
        pos: &Coord3D,
        geom: &GeometryInfo,
        angle: Real,
        use_2d: Bool,
    ) -> Vec<ObjectSlot> {
        let rule = if use_2d != FALSE {
            DistanceCalculationType::FromBoundingSphere2d
        } else {
            DistanceCalculationType::FromBoundingSphere3d
        };
        let probe = CollideInfo {
            pos: *pos,
            geometry: geom.clone(),
            angle,
        };
        // "just a little slop"
        let radius = geom.get_bounding_sphere_radius() * 1.1;
        let mut found = self.search_partition(QueryCenter::Position(*pos), radius, rule);
        found.retain(|&other| {
            self.world.objects.get(other).is_some_and(|o| {
                let them = CollideInfo {
                    pos: *o.thing.get_position(),
                    geometry: o.geometry.clone(),
                    angle: o.thing.get_orientation(),
                };
                geom_collides_with_geom(&probe, &them)
            })
        });
        found
    }

    fn ai_is_clearing_mines(&self, owner: ObjectSlot) -> Bool {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return FALSE;
        };
        let ctx = self.world.modules.meta(ms).context(ms);
        self.world
            .modules
            .payload(ms)
            .and_then(|m| m.as_ai())
            .map_or(FALSE, |ai| ai.is_clearing_mines(self, ctx))
    }

    fn find_objects_in_range_of(&self, owner: ObjectSlot, radius: Real) -> Vec<ObjectSlot> {
        self.search_partition_from_object(owner, radius, DistanceCalculationType::FromCenter2d)
    }

    fn find_objects_in_range_bounding_sphere_3d(
        &self,
        owner: ObjectSlot,
        radius: Real,
    ) -> Vec<ObjectSlot> {
        self.search_partition_from_object(
            owner,
            radius,
            DistanceCalculationType::FromBoundingSphere3d,
        )
    }

    fn draw_icon_ui(&self) -> Bool {
        self.world.draw_icon_ui
    }

    fn horde_member_state(&self, owner: ObjectSlot) -> Option<Bool> {
        let obj = self.world.objects.get(owner)?;
        obj.behaviors
            .iter()
            .filter(|&&(_, mask)| mask.contains(FacetMask::HORDE_UPDATE))
            .find_map(|&(ms, _)| self.world.modules.payload(ms)?.as_horde())
            .map(|horde| horde.is_true_horde_member())
    }

    fn ai_cur_locomotor_preferred_height(&self, owner: ObjectSlot) -> Option<Real> {
        let obj = self.world.objects.get(owner)?;
        obj.ai_locomotor.as_ref().map(|l| l.preferred_height)
    }

    fn terrain_extent(&self) -> Region3D {
        *self.world.partition.world_extents()
    }

    fn object_is_able_to_attack(&self, owner: ObjectSlot) -> Bool {
        // An entry in `attack_abilities` answers for weapons that are not ported;
        // without one the object is asked.
        self.world
            .objects
            .id_of(owner)
            .and_then(|id| self.world.attack_abilities.get(&id))
            .map_or_else(|| Bool::from(self.is_able_to_attack(owner)), |entry| entry.able)
    }

    fn object_has_client_update(&self, owner: ObjectSlot, name: &str) -> Bool {
        if self.object_has_drawable(owner) == FALSE {
            return FALSE;
        }
        let found = self.world.objects.get(owner).and_then(|o| {
            o.thing
                .template_override()
                .get_final(&self.rules.thing_factory)
        });
        Bool::from(found.is_some_and(|t| {
            t.get_client_update_module_info()
                .nuggets()
                .iter()
                .any(|n| n.name == name)
        }))
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

    fn player_energy_production(&self, team_id: TeamId) -> Int {
        self.world
            .player_energy
            .get(&team_id)
            .map_or(0, |e| e.get_production(self.world.frame))
    }

    fn player_radar_counts(&self, team_id: TeamId) -> (Int, Int) {
        self.world
            .player_radars
            .get(&team_id)
            .copied()
            .unwrap_or((0, 0))
    }

    fn production_cost_change(&self, team_id: TeamId, kindof: &KindOfMaskType) -> Real {
        let mut start = 1.0_f32;
        if let Some(list) = self.world.player_cost_modifiers.get(&team_id) {
            for (ko_mask, percent, _refs) in list {
                if kindof.test_for_all(ko_mask) != FALSE {
                    start *= 1.0_f32 + *percent;
                }
            }
        }
        start
    }

    fn world_object_slots(&self) -> Vec<ObjectSlot> {
        self.world.objects.world_iter().collect()
    }

    fn object_is_significantly_above_terrain(&self, owner: ObjectSlot) -> Bool {
        self.world.objects.get(owner).map_or(FALSE, |o| {
            o.thing
                .is_significantly_above_terrain(self.terrain.as_ref(), self.rules.global_data.gravity)
        })
    }

    fn player_is_active(&self, team_id: TeamId) -> Bool {
        Bool::from(!self.world.player_inactive.contains(&team_id))
    }

    fn player_bounty_percent(&self, team_id: TeamId) -> Real {
        self.world
            .player_bounty_percent
            .get(&team_id)
            .copied()
            .unwrap_or(0.0)
    }

    fn object_is_trainable(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .map_or(FALSE, |o| o.experience_tracker.is_trainable())
    }

    fn object_crusher_level(&self, owner: ObjectSlot) -> Int {
        self.world
            .objects
            .get(owner)
            .and_then(|o| {
                o.thing
                    .template_override()
                    .get_final(&self.rules.thing_factory)
                    .map(|t| t.get_crusher_level() as Int)
            })
            .unwrap_or(0)
    }

    fn object_crushable_level(&self, owner: ObjectSlot) -> Int {
        self.world
            .objects
            .get(owner)
            .and_then(|o| {
                o.thing
                    .template_override()
                    .get_final(&self.rules.thing_factory)
                    .map(|t| t.get_crushable_level() as Int)
            })
            .unwrap_or(0)
    }

    fn contain_count(&self, owner: ObjectSlot) -> UnsignedInt {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return 0;
        };
        if let Some(c) = self.world.modules.payload(cs).and_then(|p| p.as_contain()) {
            return c.get_contain_count(self, self.world.modules.meta(cs).context(cs));
        }
        match self.world.modules.sidecar(cs) {
            ModuleSidecar::Contain { count, .. } => count,
            _ => 0,
        }
    }

    fn contain_max(&self, owner: ObjectSlot) -> Int {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return -1;
        };
        if let Some(c) = self.world.modules.payload(cs).and_then(|p| p.as_contain()) {
            return c.get_contain_max(self, self.world.modules.meta(cs).context(cs));
        }
        match self.world.modules.sidecar(cs) {
            ModuleSidecar::Contain { max, .. } => max,
            _ => -1,
        }
    }

    fn is_valid_container_for(
        &self,
        container: ObjectSlot,
        obj: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        if let Some(c) = self.world.modules.payload(cs).and_then(|p| p.as_contain()) {
            let ctx = self.world.modules.meta(cs).context(cs);
            return c.is_valid_container_for(self, ctx, obj, check_capacity);
        }
        FALSE
    }

    fn tunnel_tracker(&self, player_index: Int) -> Option<&TunnelTrackerState> {
        self.world.tunnel_trackers.get(&player_index)
    }

    fn max_tunnel_capacity(&self) -> Int {
        self.rules.global_data.max_tunnel_capacity
    }

    fn object_last_damage(&self, owner: ObjectSlot) -> Option<(ObjectId, UnsignedInt)> {
        let bs = self.world.objects.get(owner)?.body_slot?;
        let body = self.world.modules.payload(bs)?.as_body()?;
        let source = body.get_last_damage_info()?.input.source_id;
        Some((source, body.get_last_damage_timestamp()?))
    }

    fn cave_tracker(&self, cave_index: Int) -> Option<&TunnelTrackerState> {
        self.world
            .cave_system
            .get_tunnel_tracker_for_cave_index(cave_index)
    }

    fn cave_can_switch_index(&self, old_index: Int, new_index: Int) -> Bool {
        self.world
            .cave_system
            .can_switch_index_to_index(old_index, new_index)
    }

    fn bridge_is_passable(&self, bridge_id: ObjectId) -> Bool {
        self.world
            .bridge_passable
            .get(&bridge_id)
            .copied()
            .unwrap_or(TRUE)
    }

    fn neutral_team_id(&self) -> TeamId {
        self.world.neutral_team_id
    }

    fn player_index(&self, team_id: TeamId) -> Int {
        self.player_index_for_team(team_id)
    }

    fn player_mask(&self, team_id: TeamId) -> UnsignedShort {
        let idx = self.player_index_for_team(team_id);
        if (0..16).contains(&idx) {
            1u16 << (idx as u32)
        } else {
            0
        }
    }

    fn ai_is_in_dead_state(&self, owner: ObjectSlot) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        if obj.ai_slot.is_none() {
            return FALSE;
        }
        if let Some(ai_ms) = obj.ai_slot {
            if let Some(ai) = self.world.modules.payload(ai_ms).and_then(|p| p.as_ai()) {
                if ai.is_ai_in_dead_state() != FALSE {
                    return TRUE;
                }
            }
        }
        obj.ai_dead
    }

    fn object_ai_enter_target(&self, owner: ObjectSlot) -> Option<ObjectSlot> {
        let obj = self.world.objects.get(owner)?;
        if let Some(ai_ms) = obj.ai_slot {
            if let Some(ai) = self.world.modules.payload(ai_ms).and_then(|p| p.as_ai()) {
                if let Some(tgt) = ai.get_enter_target() {
                    return Some(tgt);
                }
            }
        }
        obj.ai_enter_target
    }

    fn object_height_above_terrain(&self, owner: ObjectSlot) -> Real {
        let Some(obj) = self.world.objects.get(owner) else {
            return 0.0;
        };
        let pos = obj.thing.get_position();
        pos.z - self.terrain.get_ground_height(pos.x, pos.y, None)
    }

    fn slow_death_scale(&self) -> Real {
        self.world.slow_death_scale
    }

    fn hulk_max_lifetime_override(&self) -> Int {
        self.world.hulk_max_lifetime_override
    }

    fn physics_last_collidee(&self, owner: ObjectSlot) -> ObjectId {
        physics_update::physics(self, owner).map_or(ObjectId::INVALID, |p| p.last_collidee)
    }

    fn object_has_drawable(&self, owner: ObjectSlot) -> Bool {
        Bool::from(self.world.objects.is_in_world_list(owner))
    }

    fn local_player_index(&self) -> Int {
        self.world.local_player_index
    }

    fn production_can_make_unit(&self, builder: ObjectSlot, tmpl: ThingTemplateId, own: Option<&dyn ProductionFacet>) -> CanMakeType {
        self.can_make_unit_for(builder, tmpl, own)
    }

    fn unit_cost_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Int {
        self.unit_cost_for(team_id, tmpl)
    }

    fn unit_time_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Int {
        self.unit_time_for(team_id, tmpl)
    }

    fn player_allowed_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Bool {
        self.allowed_to_build_for(team_id, tmpl)
    }

    fn can_produce_upgrade(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool {
        self.can_produce_upgrade_for(owner, upgrade)
    }

    fn affected_by_upgrade(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool {
        self.affected_by_upgrade_for(owner, upgrade)
    }

    fn player_has_upgrade_in_production(&self, team_id: TeamId, mask: &UpgradeMaskType) -> Bool {
        Bool::from(
            self.world
                .player_upgrades_in_progress
                .get(&team_id)
                .is_some_and(|owned| owned.test_for_all(mask) != FALSE),
        )
    }

    fn parking_door_requirement(&self, owner: ObjectSlot, tmpl: ThingTemplateId) -> (Bool, Bool) {
        self.parking_requirement_for(owner, tmpl)
    }

    fn object_has_exit_interface(&self, owner: ObjectSlot) -> Bool {
        self.has_exit_interface(owner)
    }

    fn player_default_team(&self, team_id: TeamId) -> TeamId {
        let idx = self.player_index_for_team(team_id);
        self.world
            .player_default_teams
            .get(&idx)
            .copied()
            .unwrap_or(team_id)
    }

    fn player_is_skirmish_ai(&self, team_id: TeamId) -> Bool {
        Bool::from(self.world.skirmish_ai_teams.contains(&team_id))
    }

    fn team_relationship_to_player(&self, team_id: TeamId, player_index: Int) -> Relationship {
        let pa = self.player_index_for_team(team_id);
        if pa == player_index {
            return Relationship::Allies;
        }
        self.world
            .player_relationships
            .get(&(pa, player_index))
            .copied()
            .unwrap_or(Relationship::Neutral)
    }

    fn contain_is_special_zero_slot(&self, owner: ObjectSlot) -> Bool {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        self.world
            .modules
            .payload(cs)
            .and_then(|p| p.as_contain())
            .map_or(FALSE, |c| c.is_special_zero_slot_container())
    }

    fn contain_is_bustable(&self, owner: ObjectSlot) -> Bool {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        self.world
            .modules
            .payload(cs)
            .and_then(|p| p.as_contain())
            .map_or(FALSE, |c| c.is_bustable())
    }

    fn contain_is_enclosing_container_for(&self, owner: ObjectSlot, rider: ObjectSlot) -> Bool {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        self.world
            .modules
            .payload(cs)
            .and_then(|p| p.as_contain())
            .map_or(FALSE, |c| c.is_enclosing_container_for(rider))
    }

    fn bridge_tower_id(&self, owner: ObjectSlot, tower_type: BridgeTowerType) -> Option<ObjectId> {
        let bs = self.world.objects.get(owner)?.bridge_slot?;
        self.world
            .modules
            .payload(bs)
            .and_then(|p| p.as_bridge())
            .map(|b| b.get_tower_id(tower_type))
    }

    fn contain_is_special_overlord_style(&self, owner: ObjectSlot) -> Bool {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        self.world
            .modules
            .payload(cs)
            .and_then(|p| p.as_contain())
            .map_or(FALSE, |c| c.is_special_overlord_style_container())
    }

    fn contain_items(&self, owner: ObjectSlot) -> Vec<ObjectId> {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return Vec::new();
        };
        self.world
            .modules
            .payload(cs)
            .and_then(|p| p.as_contain())
            .map_or_else(Vec::new, |c| {
                let ctx = self.world.modules.meta(cs).context(cs);
                c.get_contained_items(self, ctx).to_vec()
            })
    }

    fn contain_is_passenger_allowed_to_fire(&self, owner: ObjectSlot, id: ObjectId) -> Bool {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        if let Some(c) = self.world.modules.payload(cs).and_then(|p| p.as_contain()) {
            let ctx = self.world.modules.meta(cs).context(cs);
            return c.is_passenger_allowed_to_fire(self, ctx, id);
        }
        FALSE
    }

    fn exit_use_spawn_rally_point(&self, owner: ObjectSlot) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        let Some(es) = obj.exit_slot.or(obj.contain_slot) else {
            return FALSE;
        };
        self.world
            .modules
            .payload(es)
            .and_then(|p| p.as_exit())
            .map_or(FALSE, |e| e.use_spawn_rally_point())
    }

    fn has_physics(&self, owner: ObjectSlot) -> Bool {
        Bool::from(self.world.objects.get(owner).is_some_and(|o| o.physics_slot.is_some()))
    }

    fn physics_velocity(&self, owner: ObjectSlot) -> Coord3D {
        physics_update::physics(self, owner).map_or_else(Coord3D::default, |p| p.vel)
    }

    fn physics_mass(&self, owner: ObjectSlot) -> Real {
        physics_update::get_mass(self, owner)
    }

    fn physics_center_of_mass_offset(&self, owner: ObjectSlot) -> Real {
        physics_update::physics(self, owner).map_or(0.0, |p| p.data.center_of_mass_offset)
    }

    fn object_crushed_ends(&self, owner: ObjectSlot) -> (Bool, Bool) {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.body_slot)
            .and_then(|bs| self.world.modules.payload(bs))
            .and_then(|m| m.as_body())
            .map_or((FALSE, FALSE), |b| (b.get_front_crushed(), b.get_back_crushed()))
    }

    fn contain_items_for_mass(&self, owner: ObjectSlot) -> Vec<ObjectId> {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return Vec::new();
        };
        self.world
            .modules
            .payload(cs)
            .and_then(|p| p.as_contain())
            .map_or_else(Vec::new, |c| c.contain_list_for_mass().to_vec())
    }

    fn ai_ignored_obstacle_id(&self, owner: ObjectSlot) -> ObjectId {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot)
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_ai())
            .map_or(ObjectId::INVALID, |ai| ai.get_ignored_obstacle_id())
    }

    fn carrier_deck_height(&self, owner: ObjectSlot) -> Real {
        let Some(obj) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| self.world.objects.slot_by_id(o.producer_id))
            .and_then(|producer| self.world.objects.get(producer))
        else {
            return 0.0;
        };
        obj.behaviors
            .iter()
            .filter_map(|&(ms, _)| self.world.modules.payload(ms))
            .find_map(|m| m.as_parking_place())
            .map_or(0.0, |pp| pp.get_landing_deck_height_offset())
    }

    fn has_ai(&self, owner: ObjectSlot) -> Bool {
        Bool::from(self.world.objects.get(owner).is_some_and(|o| o.ai_slot.is_some()))
    }

    fn ai_free_to_exit(&self, owner: ObjectSlot) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return TRUE;
        };
        if obj.ai_free_to_exit == FALSE {
            return FALSE;
        }
        if let Some(ai_ms) = obj.ai_slot {
            if let Some(ai) = self.world.modules.payload(ai_ms).and_then(|m| m.as_ai()) {
                let ctx = self.world.modules.meta(ai_ms).context(ai_ms);
                return ai.is_free_to_exit(self, ctx);
            }
        }
        TRUE
    }

    fn ai_current_victim(&self, owner: ObjectSlot) -> Option<ObjectSlot> {
        let obj = self.world.objects.get(owner)?;
        if let Some(ai_ms) = obj.ai_slot {
            if let Some(ai) = self.world.modules.payload(ai_ms).and_then(|m| m.as_ai()) {
                if let Some(v) = ai.get_current_victim() {
                    return Some(v);
                }
            }
        }
        obj.ai_current_victim
    }

    fn ai_current_victim_pos(&self, owner: ObjectSlot) -> Option<Coord3D> {
        let obj = self.world.objects.get(owner)?;
        if let Some(ai_ms) = obj.ai_slot {
            if let Some(ai) = self.world.modules.payload(ai_ms).and_then(|m| m.as_ai()) {
                if let Some(pos) = ai.get_current_victim_pos() {
                    return Some(pos);
                }
            }
        }
        obj.ai_current_victim_pos
    }

    fn ai_has_cur_locomotor(&self, owner: ObjectSlot) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        if let Some(ai_ms) = obj.ai_slot {
            if let Some(ai) = self.world.modules.payload(ai_ms).and_then(|m| m.as_ai()) {
                if ai.has_cur_locomotor() != FALSE {
                    return TRUE;
                }
            }
        }
        obj.ai_has_locomotor
    }

    fn valid_movement_terrain(&self, layer: PathfindLayerEnum, _rider: ObjectSlot, pos: &Coord3D) -> Bool {
        if let Some(ov) = self.world.valid_movement_terrain_override {
            return ov;
        }
        if pos.x < 0.0
            || pos.y < 0.0
            || pos.x > self.world.map_extent_hi.x
            || pos.y > self.world.map_extent_hi.y
        {
            return FALSE;
        }
        let cell_x = real_to_int_floor(pos.x / 10.0_f32);
        let cell_y = real_to_int_floor(pos.y / 10.0_f32);
        Bool::from(self.pathfind_cell_type(layer, cell_x, cell_y) != PathfindCellType::Impassable)
    }

    fn pathfind_cell_type(
        &self,
        layer: PathfindLayerEnum,
        cell_x: Int,
        cell_y: Int,
    ) -> PathfindCellType {
        if let Some(&ct) = self
            .world
            .blocked_pathfind_cells
            .get(&(layer as Int, cell_x, cell_y))
        {
            return ct;
        }
        let max_cx = real_to_int_floor(self.world.map_extent_hi.x / 10.0_f32);
        let max_cy = real_to_int_floor(self.world.map_extent_hi.y / 10.0_f32);
        if cell_x < 0 || cell_y < 0 || cell_x >= max_cx || cell_y >= max_cy {
            PathfindCellType::Impassable
        } else {
            PathfindCellType::Clear
        }
    }

    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum {
        self.terrain.get_layer_for_destination(pos)
    }

    fn get_layer_height(&self, x: Real, y: Real, layer: PathfindLayerEnum) -> Real {
        self.terrain.get_layer_height(x, y, layer, None, TRUE)
    }

    fn pathfinder_snap_position(&self, owner: ObjectSlot, pos: &mut Coord3D) {
        let (cell_x, cell_y, center) = self.pathfind_goal_cell(owner, pos);
        self.adjust_coord_to_cell(cell_x, cell_y, center, pos, PathfindLayerEnum::Ground);
    }

    fn ai_is_doing_ground_movement(&self, owner: ObjectSlot) -> Bool {
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        if obj.is_disabled_by_type(DisabledType::Unmanned) != FALSE
            && self.object_is_kind_of(owner, KindOfType::ProducedAtHelipad) != FALSE
        {
            return TRUE;
        }
        let air = crate::rules::combat_types::LocomotorSurfaceType::Air.as_i32();
        // `LocomotorSet::getValidSurfaces` of the chosen set.
        let valid_surfaces = obj.ai_locomotor_set.map_or(0, |set| {
            obj.thing
                .template_override()
                .get_final(&self.rules.thing_factory)
                .map_or(0, |t| {
                    t.get_locomotor_set(set)
                        .iter()
                        .filter_map(|name| self.rules.locomotor_store.find_by_name(name))
                        .fold(0, |mask, l| mask | l.surfaces)
                })
        });
        if valid_surfaces == air {
            return FALSE;
        }
        let Some(loco) = obj.ai_locomotor.as_ref() else {
            return FALSE;
        };
        if loco.get_legal_surfaces() & air != 0
            || obj.is_disabled_by_type(DisabledType::Held) != FALSE
        {
            return FALSE;
        }
        let allowed_to_fall =
            crate::modules::update::physics_update::physics(self, owner).is_some_and(|p| {
                p.flag(crate::modules::update::physics_update::ALLOW_TO_FALL)
            });
        Bool::from(!(allowed_to_fall && self.object_is_above_terrain(owner) != FALSE))
    }

    fn pristine_bone_transforms(&self, owner: ObjectSlot, bone_prefix: &str) -> Vec<Matrix3D> {
        self.world
            .objects
            .get(owner)
            .and_then(|o| {
                o.thing
                    .template_override()
                    .get_final(&self.rules.thing_factory)
            })
            .and_then(|t| {
                self.world
                    .pristine_bones
                    .get(&(t.get_name().to_owned(), bone_prefix.to_owned()))
            })
            .cloned()
            .unwrap_or_default()
    }

    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        only_healthy_bridges: Bool,
    ) -> PathfindLayerEnum {
        self.terrain
            .get_highest_layer_for_destination(pos, only_healthy_bridges)
    }

    fn object_vision_range(&self, owner: ObjectSlot) -> Real {
        self.world
            .objects
            .get(owner)
            .map_or(0.0, |o| o.vision_range)
    }

    fn object_shroud_clearing_range(&self, owner: ObjectSlot) -> Real {
        let Some(obj) = self.world.objects.get(owner) else {
            return 0.0;
        };
        if obj.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE {
            obj.geometry.get_bounding_circle_radius()
        } else {
            obj.shroud_clearing_range
        }
    }

    fn object_shroud_range(&self, owner: ObjectSlot) -> Real {
        self.world
            .objects
            .get(owner)
            .map_or(0.0, |o| o.shroud_range)
    }

    fn stealth_params(&self, owner: ObjectSlot) -> Option<StealthParams> {
        let ss = self.world.objects.get(owner)?.stealth_slot?;
        self.world.modules.payload(ss)?.as_stealth()?.stealth_params()
    }

    fn object_last_damage_timestamp(&self, owner: ObjectSlot) -> Option<UnsignedInt> {
        let bs = self.world.objects.get(owner)?.body_slot?;
        self.world.modules.payload(bs)?.as_body()?.get_last_damage_timestamp()
    }

    fn weapon_slot_last_shot_frame(&self, owner: ObjectSlot, slot: usize) -> Option<UnsignedInt> {
        let weapon = self.world.objects.get(owner)?.weapon_set.weapons.get(slot)?;
        weapon.as_ref().map(|w| w.last_fire_frame)
    }

    fn object_has_contain(&self, owner: ObjectSlot) -> Bool {
        Bool::from(self.world.objects.get(owner).is_some_and(|o| o.contain_slot.is_some()))
    }

    fn contain_stealth_units_contained(&self, owner: ObjectSlot) -> UnsignedInt {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.contain_slot)
            .and_then(|cs| self.world.modules.payload(cs))
            .and_then(|p| p.as_contain())
            .map_or(0, |c| c.get_stealth_units_contained())
    }

    fn contain_is_rider_change(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.contain_slot)
            .and_then(|cs| self.world.modules.payload(cs))
            .and_then(|p| p.as_contain())
            .map_or(FALSE, |c| c.is_rider_change_contain())
    }

    fn thing_template_exists(&self, name: &str) -> Bool {
        Bool::from(self.rules.thing_factory.find_template_id(name).is_some())
    }

    fn spawn_closest_slave(&self, owner: ObjectSlot, pos: &Coord3D) -> Option<Option<ObjectSlot>> {
        let ss = self.world.objects.get(owner)?.spawn_slot?;
        let spawn = self.world.modules.payload(ss)?.as_spawn_behavior()?;
        Some(spawn.get_closest_slave(self, pos))
    }

    fn contain_closest_rider(
        &self,
        owner: ObjectSlot,
        pos: &Coord3D,
    ) -> Option<Option<ObjectSlot>> {
        let cs = self.world.objects.get(owner)?.contain_slot?;
        let contain = self.world.modules.payload(cs)?.as_contain()?;
        Some(contain.get_closest_rider(self, pos))
    }

    fn slaved_is_self_tasking(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.slaved_slot)
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_slaved_update())
            .map_or(FALSE, |s| s.is_self_tasking())
    }

    fn ai_path_destination(&self, owner: ObjectSlot) -> Option<Coord3D> {
        let ms = self.world.objects.get(owner)?.ai_slot?;
        self.world.modules.payload(ms)?.as_ai()?.get_path_destination()
    }

    fn able_to_attack_specific_object(
        &self,
        owner: ObjectSlot,
        _attack_type: UnsignedInt,
        _target: ObjectSlot,
        _cmd_source: CommandSourceType,
    ) -> CanAttackResult {
        self.attack_ability(owner).specific_object
    }

    fn able_to_use_weapon_against_target(
        &self,
        owner: ObjectSlot,
        _attack_type: UnsignedInt,
        _victim: Option<ObjectSlot>,
        _pos: &Coord3D,
        _cmd_source: CommandSourceType,
    ) -> CanAttackResult {
        self.attack_ability(owner).weapon_against_target
    }

    fn ai_last_command_source(&self, owner: ObjectSlot) -> CommandSourceType {
        self.world
            .objects
            .get(owner)
            .map_or(CommandSourceType::FromAi, |o| o.ai_last_command_source)
    }

    fn controlling_player_objects(&self, team_id: TeamId) -> Vec<ObjectSlot> {
        self.player_object_slots(self.player_index_for_team(team_id))
    }

    fn enemy_player_objects(&self, team_id: TeamId) -> Vec<ObjectSlot> {
        let them = self.player_index_for_team(team_id);
        (0..MAX_PLAYER_COUNT as Int)
            .filter(|&n| {
                self.world.player_relationships.get(&(n, them)) == Some(&Relationship::Enemies)
            })
            .flat_map(|n| self.player_object_slots(n))
            .collect()
    }

    fn estimate_highest_terrain_along_line(
        &self,
        start: &Coord3D,
        end: &Coord3D,
    ) -> Option<Real> {
        crate::spatial::terrain::estimate_highest_terrain_along_line(
            &self.world.partition,
            self.terrain.as_ref(),
            start,
            end,
        )
    }

    fn find_closest_object_to_position(&self, pos: &Coord3D, radius: Real) -> Option<ObjectSlot> {
        let objects = &self.world.objects;
        let factory = &self.rules.thing_factory;
        let found = self.world.partition.get_closest_object(
            QueryCenter::Position(*pos),
            radius,
            DistanceCalculationType::FromCenter2d,
            &|id| partition_object(objects, factory, id),
            &mut |_| true,
        );
        found.id.and_then(|id| objects.slot_by_id(id))
    }

    fn contain_is_immune_to_clear(&self, owner: ObjectSlot) -> Bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.contain_slot)
            .and_then(|cs| self.world.modules.payload(cs))
            .and_then(|p| p.as_contain())
            .map_or(FALSE, |c| c.is_immune_to_clear_building_attacks())
    }

    fn object_has_reserved_space(&self, airfield: ObjectSlot, id: ObjectId) -> Bool {
        let found = self.world.objects.get(airfield).is_some_and(|o| {
            o.behaviors.iter().any(|&(ms, _)| {
                self.world
                    .modules
                    .payload(ms)
                    .and_then(|p| p.as_parking_place())
                    .is_some_and(|p| p.has_reserved_space(id) != FALSE)
            })
        });
        Bool::from(found)
    }

    fn player_color(&self, team_id: TeamId) -> Int {
        self.world.team_colors.get(&team_id).copied().unwrap_or(0)
    }

    fn player_side(&self, team_id: TeamId) -> String {
        self.world.team_sides.get(&team_id).cloned().unwrap_or_default()
    }

    fn contain_is_garrisonable(&self, owner: ObjectSlot) -> Bool {
        let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) else {
            return FALSE;
        };
        if let Some(c) = self.world.modules.payload(cs).and_then(|p| p.as_contain()) {
            return c.is_garrisonable(self, self.world.modules.meta(cs).context(cs));
        }
        match self.world.modules.sidecar(cs) {
            ModuleSidecar::Contain { garrisonable, .. } => garrisonable,
            _ => FALSE,
        }
    }

    fn object_construction_percent(&self, owner: ObjectSlot) -> Real {
        self.world
            .objects
            .get(owner)
            .map_or(CONSTRUCTION_COMPLETE, |o| o.construction_percent)
    }

    fn object_is_logically_visible(&self, owner: ObjectSlot) -> Bool {
        // `getOuterObject`.
        let mut outer = owner;
        while let Some(container) = self.world.objects.get(outer).and_then(|o| o.contained_by) {
            outer = container;
        }
        if self.object_is_kind_of(outer, KindOfType::Disguiser) != FALSE {
            return TRUE;
        }
        let Some(obj) = self.world.objects.get(outer) else {
            return TRUE;
        };
        if obj.status.test(ObjectStatusType::Stealthed.as_i32()) == FALSE
            || obj.status.test(ObjectStatusType::Detected.as_i32()) != FALSE
        {
            return TRUE;
        }
        let local = self.world.local_player_index;
        let owner_player = self.player_index_for_team(obj.team_id);
        let relationship = if owner_player == local {
            Relationship::Allies
        } else {
            self.world
                .player_relationships
                .get(&(local, owner_player))
                .copied()
                .unwrap_or(Relationship::Neutral)
        };
        let active = self
            .world
            .player_default_teams
            .get(&local)
            .is_none_or(|&team| self.player_is_active(team) != FALSE);
        Bool::from(!active || relationship == Relationship::Allies)
    }

    fn object_last_damage_type(&self, owner: ObjectSlot) -> Option<DamageType> {
        let obj = self.world.objects.get(owner)?;
        match self.world.modules.payload(obj.body_slot?) {
            Some(body) => Some(body.as_body()?.get_last_damage_info()?.input.damage_type),
            // The body is running (a death inside its own damage handling).
            None => obj.body_last_damage_type,
        }
    }

    fn adjust_topple_direction(&self, owner: ObjectSlot, direction: &mut Coord3D) {
        let scripted = self
            .world
            .objects
            .id_of(owner)
            .and_then(|id| self.world.script_topple_directions.get(&id));
        if let Some(&scripted) = scripted {
            *direction = scripted;
            direction.normalize();
        }
    }
}

impl SimContext {
    fn try_find_position(
        &self,
        center: &Coord3D,
        dist: Real,
        angle: Real,
        options: &FindPositionOptions,
    ) -> Option<Coord3D> {
        let mut pos = Coord3D::new(
            dist * gm_cosf(angle) + center.x,
            dist * gm_sinf(angle) + center.y,
            0.0,
        );
        let mut layer = PathfindLayerEnum::Ground;
        if (options.flags & FPF_USE_HIGHEST_LAYER) != 0 {
            pos.z = 99999.0;
            layer = self.terrain.get_highest_layer_for_destination(&pos, FALSE);
            pos.z = self.terrain.get_layer_height(pos.x, pos.y, layer, None, TRUE);
            if layer != PathfindLayerEnum::Ground {
                pos.z += 1.0;
            }
        } else {
            pos.z = self.terrain.get_ground_height(pos.x, pos.y, None);
        }

        if zh_common::math::gm_fabsf(pos.z - center.z) > options.max_z_delta {
            return None;
        }
        let cell_x = real_to_int_floor(pos.x / 10.0_f32);
        let cell_y = real_to_int_floor(pos.y / 10.0_f32);
        let ctype = self.pathfind_cell_type(layer, cell_x, cell_y);
        if ctype == PathfindCellType::Impassable {
            return None;
        }
        if (options.flags & FPF_CLEAR_CELLS_ONLY) != 0 && ctype != PathfindCellType::Clear {
            return None;
        }

        if (options.flags & FPF_IGNORE_WATER) == 0 {
            let underwater = self.terrain.is_underwater(pos.x, pos.y, None, None) != FALSE;
            if (options.flags & FPF_WATER_ONLY) != 0 {
                if !underwater || layer != PathfindLayerEnum::Ground {
                    return None;
                }
            } else if underwater && layer == PathfindLayerEnum::Ground {
                return None;
            }
        }

        if (options.flags & FPF_IGNORE_ALL_OBJECTS) == 0 {
            // `iteratePotentialCollisions` with a sphere of radius 5.
            let probe = CollideInfo {
                pos,
                geometry: GeometryInfo::new(GeometryType::Sphere, TRUE, 5.0, 5.0, 5.0),
                angle,
            };
            for other in self.world_object_slots() {
                if Some(other) == options.ignore_object || Some(other) == options.source_to_path_to_dest {
                    continue;
                }
                let Some(o) = self.world.objects.get(other) else {
                    continue;
                };
                if o.is_destroyed() || o.contained_by.is_some() {
                    continue;
                }
                let opos = o.thing.get_position();
                let dx = opos.x - pos.x;
                let dy = opos.y - pos.y;
                // Within 1.1 sphere radii of the other's bounding circle, and touching.
                let gap = gm_sqrtf(dx * dx + dy * dy) - o.geometry.get_bounding_circle_radius();
                if gap >= 5.0_f32 * 1.1_f32 {
                    continue;
                }
                let shape = CollideInfo {
                    pos: *opos,
                    geometry: o.geometry.clone(),
                    angle: o.thing.get_orientation(),
                };
                if !geom_collides_with_geom(&probe, &shape) {
                    continue;
                }
                if let Some(rel_slot) = options.relationship_object {
                    let rel = self.player_relationship(rel_slot, other);
                    let is_unit = o
                        .thing
                        .is_kind_of(&self.rules.thing_factory, KindOfType::Infantry)
                        != FALSE
                        || o
                            .thing
                            .is_kind_of(&self.rules.thing_factory, KindOfType::Vehicle)
                            != FALSE;
                    let is_struct = o
                        .thing
                        .is_kind_of(&self.rules.thing_factory, KindOfType::Structure)
                        != FALSE;
                    if rel != Relationship::Enemies {
                        if (options.flags & FPF_IGNORE_ALLY_OR_NEUTRAL_UNITS) != 0 && is_unit {
                            continue;
                        }
                        if (options.flags & FPF_IGNORE_ALLY_OR_NEUTRAL_STRUCTURES) != 0 && is_struct
                        {
                            continue;
                        }
                    } else {
                        if (options.flags & FPF_IGNORE_ENEMY_UNITS) != 0 && is_unit {
                            continue;
                        }
                        if (options.flags & FPF_IGNORE_ENEMY_STRUCTURES) != 0 && is_struct {
                            continue;
                        }
                    }
                }
                return None;
            }
        }

        Some(pos)
    }
}

impl CollisionHost for SimContext {
    fn grid_mut(&mut self) -> &mut PartitionGrid {
        &mut self.world.partition
    }

    fn partition_object(&self, id: ObjectId) -> Option<PartitionObject> {
        partition_object(&self.world.objects, &self.rules.thing_factory, id)
    }

    fn is_no_collide(&self, id: ObjectId) -> bool {
        self.world.objects.slot_by_id(id).is_some_and(|slot| {
            self.object_is_kind_of(slot, KindOfType::NoCollide) != FALSE
        })
    }

    fn has_no_collisions_status(&self, id: ObjectId) -> bool {
        self.world
            .objects
            .slot_by_id(id)
            .and_then(|slot| self.world.objects.get(slot))
            .is_some_and(|o| o.status.test(ObjectStatusType::NoCollisions.as_i32()) != FALSE)
    }

    fn is_destroyed(&self, id: ObjectId) -> bool {
        self.world
            .objects
            .slot_by_id(id)
            .and_then(|slot| self.world.objects.get(slot))
            .is_none_or(ObjectRecord::is_destroyed)
    }

    fn on_collide(&mut self, id: ObjectId, other: ObjectId, loc: &Coord3D, normal: &Coord3D) {
        let Some(slot) = self.world.objects.slot_by_id(id) else {
            return;
        };
        let other = self.world.objects.slot_by_id(other);
        self.dispatch_on_collide(slot, other, Some(loc), Some(normal));
    }
}

impl SimApi for SimContext {
    fn ai_busy(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        self.order_ai(owner, "aiBusy", |ai, sim, ctx| ai.ai_busy(sim, ctx, cmd_source));
    }

    fn ai_move_to_object(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType) {
        self.order_ai(owner, "aiMoveToObject", |ai, sim, ctx| {
            ai.ai_move_to_object(sim, ctx, target, cmd_source)
        });
    }

    fn ai_attack_object(&mut self, owner: ObjectSlot, target: ObjectSlot, max_shots: Int, cmd_source: CommandSourceType) {
        self.order_ai(owner, "aiAttackObject", |ai, sim, ctx| {
            ai.ai_attack_object(sim, ctx, target, max_shots, cmd_source)
        });
    }

    fn ai_face_object(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType) {
        self.order_ai(owner, "aiFaceObject", |ai, sim, ctx| {
            ai.ai_face_object(sim, ctx, target, cmd_source)
        });
    }

    fn ai_face_position(&mut self, owner: ObjectSlot, pos: &Coord3D, cmd_source: CommandSourceType) {
        self.order_ai(owner, "aiFacePosition", |ai, sim, ctx| {
            ai.ai_face_position(sim, ctx, pos, cmd_source)
        });
    }

    fn ai_enter(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType) {
        self.order_ai(owner, "aiEnter", |ai, sim, ctx| ai.ai_enter(sim, ctx, target, cmd_source));
    }

    fn ai_ignore_obstacle(&mut self, owner: ObjectSlot, obstacle: ObjectId) {
        let Some(ai_ms) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return;
        };
        let _ = self.dispatch_module(ai_ms, |m, _, _| {
            if let Some(ai) = m.as_ai_mut() {
                ai.ignore_obstacle(obstacle);
            }
        });
    }

    fn exit_special_ability(&mut self, module: ModuleSlot, cleanup: Bool) {
        let _ = self.dispatch_module(module, |m, sim, ctx| {
            if let Some(f) = m.as_special_power_update_mut() {
                f.on_exit(sim, ctx, cleanup);
            }
        });
    }

    fn add_experience_points(&mut self, owner: ObjectSlot, gain: Int) {
        self.add_experience_to(owner, gain);
    }

    fn add_skill_points(&mut self, team_id: TeamId, points: Int) {
        let player = self.player_index_for_team(team_id);
        let total = self.world.skill_points_awarded.entry(player).or_insert(0);
        *total = total.wrapping_add(points);
    }

    fn record_special_power_used(
        &mut self,
        owner: ObjectSlot,
        classification: AcademyClassificationType,
    ) {
        let Some(team) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        self.academy(team).record_special_power_used(classification);
    }

    fn record_building_capture(&mut self, team_id: TeamId) {
        self.academy(team_id).record_building_capture();
    }

    fn sticky_bomb_init(&mut self, charge: ObjectSlot, target: ObjectSlot, bomber: ObjectSlot) {
        for ms in self.behavior_slots(charge) {
            let found = self.dispatch_module(ms, |m, sim, ctx| {
                m.as_sticky_bomb_mut()
                    .map(|f| f.init_sticky_bomb(sim, ctx, target, bomber))
                    .is_some()
            });
            if found == Ok(true) {
                break;
            }
        }
    }

    fn sticky_bomb_detonate(&mut self, charge: ObjectSlot) -> Bool {
        for ms in self.behavior_slots(charge) {
            let found = self.dispatch_module(ms, |m, sim, ctx| {
                m.as_sticky_bomb_mut()
                    .map(|f| f.detonate(sim, ctx))
                    .is_some()
            });
            if found == Ok(true) {
                return TRUE;
            }
        }
        FALSE
    }

    fn check_and_detonate_booby_trap(&mut self, owner: ObjectSlot, victim: ObjectSlot) -> Bool {
        self.detonate_booby_trap(owner, victim)
    }

    fn do_special_power_at_object(&mut self, owner: ObjectSlot, template_id: UnsignedInt, target: ObjectSlot, command_options: UnsignedInt, forced: Bool) {
        self.special_power_at_object(owner, template_id, target, command_options, forced);
    }

    fn special_power_start_recharge(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        if let Some(ms) = self.find_special_power_module(owner, template_id) {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(sp) = m.as_special_power_mut() {
                    sp.start_power_recharge(sim, ctx);
                }
            });
        }
    }

    fn special_power_mark_triggered(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        if let Some(ms) = self.find_special_power_module(owner, template_id) {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(sp) = m.as_special_power_mut() {
                    sp.mark_special_power_triggered(sim, ctx, None);
                }
            });
        }
    }

    fn random_offset_within_footprint(&mut self, geometry: &GeometryInfo, offset: &mut Coord3D) {
        geometry.make_random_offset_within_footprint(offset, &mut self.world.rng);
    }

    fn set_command_button_hunt(&mut self, owner: ObjectSlot, button_name: &str) {
        for ms in self.behavior_slots(owner) {
            let found = self.dispatch_module(ms, |m, sim, ctx| {
                m.as_command_button_hunt_mut()
                    .map(|f| f.set_command_button(sim, ctx, button_name))
                    .is_some()
            });
            if found == Ok(true) {
                break;
            }
        }
    }

    fn share_weapon_reload(&mut self, owner: ObjectSlot, when: UnsignedInt, status: WeaponStatus) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            for w in obj.weapon_set.weapons.iter_mut().flatten() {
                w.set_possible_next_shot_frame(when);
                w.status = status;
            }
        }
    }

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
            self.world.sleepy_updates.restore_module_schedule(
                &mut self.world.modules,
                slot,
                schedule.next_call_frame(),
            );
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

    fn try_to_ignite(&mut self, owner: ObjectSlot) {
        let found = self.world.objects.get(owner).and_then(|obj| {
            obj.behaviors.iter().map(|&(ms, _)| ms).find(|&ms| {
                self.world
                    .modules
                    .payload(ms)
                    .is_some_and(|m| m.as_flammable().is_some())
            })
        });
        if let Some(ms) = found {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(f) = m.as_flammable_mut() {
                    f.try_to_ignite(sim, ctx);
                }
            });
        }
    }

    fn start_fire_spreading(&mut self, owner: ObjectSlot) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors.iter().map(|&(ms, _)| ms).collect()
        });
        for ms in slots {
            let mut handled = false;
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(f) = m.as_fire_spread_mut() {
                    f.start_fire_spreading(sim, ctx);
                    handled = true;
                }
            });
            if handled {
                break;
            }
        }
    }

    fn set_body_aflame(&mut self, owner: ObjectSlot, setting: Bool) {
        // A body that is executing is skipped: the call changes nothing in the simulation.
        if let Some(body) = self.world.objects.get(owner).and_then(|o| o.body_slot) {
            let _ = self.dispatch_module(body, |m, sim, ctx| {
                if let Some(b) = m.as_body_mut() {
                    b.set_aflame(sim, ctx, setting);
                }
            });
        }
    }

    fn lend_module(
        &mut self,
        slot: ModuleSlot,
        module: Box<dyn BehaviorModule>,
        f: &mut dyn FnMut(&mut dyn SimApi),
    ) -> Option<Box<dyn BehaviorModule>> {
        if !self.world.modules.is_executing(slot) {
            f(self);
            return Some(module);
        }
        let generation = self.world.modules.meta(slot).generation;
        self.world.modules.restore_payload(slot, generation, module);
        f(self);
        let taken = self.world.modules.take_payload(slot).ok();
        taken.map(|(module, _, _)| module)
    }

    fn body_internal_change_health(&mut self, owner: ObjectSlot, delta: Real) {
        let Some(body_slot) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return;
        };
        let _ = self.dispatch_module(body_slot, |m, sim, ctx| {
            if let Some(body) = m.as_body_mut() {
                body.internal_change_health(sim, ctx, delta);
            }
        });
    }

    fn set_land_mine_scoot_parms(&mut self, owner: ObjectSlot, start: &Coord3D, end: &Coord3D) {
        for ms in self.behavior_slots(owner) {
            let mut handled = false;
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(f) = m.as_land_mine_mut() {
                    f.set_scoot_parms(sim, ctx, start, end);
                    handled = true;
                }
            });
            if handled {
                break;
            }
        }
    }

    fn stop_auto_healing(&mut self, owner: ObjectSlot) {
        for ms in self.behavior_slots(owner) {
            let mut handled = false;
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(f) = m.as_auto_heal_mut() {
                    f.stop_healing(sim, ctx);
                    handled = true;
                }
            });
            if handled {
                break;
            }
        }
    }

    fn attempt_damage_watching(
        &mut self,
        target: ObjectSlot,
        dmg: &mut DamageInfo,
        watch: ModuleSlot,
    ) -> Option<ObjectStatusMaskType> {
        let outer = self.world.damage_watch.replace((watch, None));
        self.attempt_damage_on(target, dmg);
        std::mem::replace(&mut self.world.damage_watch, outer).and_then(|(_, status)| status)
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
            let status = self.world.objects.get(owner).map(|o| o.status.clone());
            if let Some((watch, seen)) = self.world.damage_watch.as_mut() {
                if *watch == ms {
                    *seen = status;
                }
            }
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
            if self.world.modules.is_executing(ms) {
                // The module killed its own owner: deliver `onDie` once it returns.
                let _ = self.world.modules.push_deferred_op(
                    ms,
                    DeferredSelfOp::Die {
                        damage_type: dmg.input.damage_type.as_i32() as u8,
                        death_type: dmg.input.death_type.as_i32() as u8,
                        source_id: dmg.input.source_id,
                        amount: dmg.input.amount,
                        kill: dmg.input.kill,
                        actual_damage_dealt: dmg.output.actual_damage_dealt,
                        actual_damage_clipped: dmg.output.actual_damage_clipped,
                    },
                );
                continue;
            }
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_die_mut() {
                    d.on_die(sim, ctx, dmg);
                }
            });
        }
        // An object that was spawned tells its spawner that it has died.
        if let Some((slave_id, producer_id)) = self
            .world
            .objects
            .get(owner)
            .map(|o| (o.id, o.producer_id))
        {
            if let Some(spawner) = self.find_object(producer_id) {
                self.notify_spawn_slave_died(spawner, slave_id, dmg);
            }
        }
        self.handle_partition_cell_maintenance(owner);
    }

    fn deposit_player_money(&mut self, team_id: TeamId, amount: UnsignedInt) {
        self.deposit_money(team_id, amount);
    }

    fn player_add_upgrade_in_production(&mut self, team_id: TeamId, upgrade: NameKeyType) {
        self.add_upgrade_in_production(team_id, upgrade);
    }

    fn player_remove_upgrade(&mut self, team_id: TeamId, upgrade: NameKeyType) {
        self.remove_player_upgrade_of(team_id, upgrade);
    }

    fn complete_production_upgrade(&mut self, owner: ObjectSlot, upgrade: NameKeyType) {
        self.complete_upgrade_of(owner, upgrade);
    }

    fn reserve_exit_door(&mut self, owner: ObjectSlot, tmpl: ThingTemplateId) -> ExitDoorType {
        self.reserve_exit_door_of(owner, tmpl)
    }

    fn try_reserve_exit_door(&mut self, owner: ObjectSlot) -> Option<ExitDoorType> {
        let es = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.exit_slot.or(o.contain_slot))?;
        if self.world.modules.is_executing(es) {
            return None;
        }
        // The callers pass no template and no particular object.
        self.dispatch_module(es, |m, sim, ctx| {
            m.as_exit_mut()
                .map(|e| e.reserve_door_for_exit(sim, ctx, ThingTemplateId(0), None))
        })
        .ok()
        .flatten()
    }

    fn unreserve_exit_door(&mut self, owner: ObjectSlot, door: ExitDoorType) {
        self.unreserve_exit_door_of(owner, door);
    }

    fn factory_new_object(&mut self, tmpl: ThingTemplateId, team_id: TeamId) -> ObjectSlot {
        self.factory_new_object_of(tmpl, team_id)
    }

    fn production_unit_created(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        self.unit_created(factory, unit);
    }

    fn record_unit_production(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        self.record_production_of(factory, unit);
    }

    fn production_set_hold_door_open(&mut self, owner: ObjectSlot, door: ExitDoorType, hold: Bool) {
        self.hold_production_door(owner, door, hold);
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
        // `FOREVER` (`GameCommon.h`).
        self.set_disabled_until(owner, disabled_type, 0x3fff_ffff);
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
        let Some(obj) = self.world.objects.get(owner) else {
            return;
        };
        let was_disabled = obj.is_disabled() != FALSE;
        let slot_idx = idx as usize;
        let changed = obj.disabled_till_frame[slot_idx] != till_frame;
        // Each newly set type pauses once more; `HELD` never pauses special powers.
        if changed
            && disabled_type != DisabledType::Held
            && obj.is_disabled_by_type(disabled_type) == FALSE
        {
            self.pause_all_special_powers(owner, TRUE);
        }
        if changed {
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.disabled_till_frame[slot_idx] = till_frame;
                obj.disabled_mask.set(idx, i32::from(till_frame > now));
            }
            self.with_weapon_slaves(owner, |spawn, sim| {
                spawn.order_slaves_disabled_until(sim, disabled_type, till_frame);
            });
        }
        // The edge runs whenever the object was not disabled on entry, even if this call
        // leaves it enabled (`till_frame` not in the future).
        if !was_disabled {
            self.on_disabled_edge(owner, TRUE);
        }
    }

    fn clear_disabled(&mut self, owner: ObjectSlot, disabled_type: DisabledType) -> Bool {
        let idx = disabled_type.as_i32();
        if !(0..DISABLED_COUNT as Int).contains(&idx) {
            return FALSE;
        }
        let Some(obj) = self.world.objects.get(owner) else {
            return FALSE;
        };
        if obj.is_disabled_by_type(disabled_type) == FALSE {
            return FALSE;
        }
        if disabled_type != DisabledType::Held {
            self.pause_all_special_powers(owner, FALSE);
        }
        self.with_weapon_slaves(owner, |spawn, sim| {
            spawn.order_slaves_to_clear_disabled(sim, disabled_type);
        });
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return FALSE;
        };
        obj.disabled_till_frame[idx as usize] = 0;
        obj.disabled_mask.set(idx, 0);
        // "In case we just edged": an expired type is cleared here, which can run the edge
        // below a second time.
        crate::gamelogic::frame::check_disabled_status(self, owner);
        if self
            .world
            .objects
            .get(owner)
            .is_some_and(|o| o.is_disabled() == FALSE)
        {
            self.on_disabled_edge(owner, FALSE);
        }
        TRUE
    }

    fn grant_upgrade(&mut self, owner: ObjectSlot, upgrade_name: &str, record_stats: Bool) {
        let Some((upgrade_type, upgrade_mask, classification)) = self
            .rules
            .upgrade_center
            .find_upgrade(upgrade_name, &mut self.rules.name_keys)
            .map(|ut| {
                (
                    ut.upgrade_type(),
                    ut.upgrade_mask().clone(),
                    ut.academy_classification_type(),
                )
            })
        else {
            return;
        };
        let Some(team_id) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        let empty_mask = UpgradeMaskType::new();
        if upgrade_type == UpgradeType::Player {
            if let Some(pending) = self.world.player_upgrades_in_progress.get_mut(&team_id) {
                pending.clear_and_set(&upgrade_mask, &empty_mask);
            }
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
                self.update_upgrade_modules(pslot);
            }
        } else {
            if let Some(obj) = self.world.objects.get_mut(owner) {
                obj.upgrades_completed
                    .clear_and_set(&empty_mask, &upgrade_mask);
            }
            self.update_upgrade_modules(owner);
        }
        if record_stats != FALSE {
            // `AcademyStats::recordUpgrade`, granted.
            self.academy(team_id).record_upgrade(classification, TRUE);
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
        self.change_status(owner, status, 1);
        if status == ObjectStatusType::Repulsor {
            let until = self.world.frame.saturating_add(2 * 30);
            self.trigger_repulsor(owner, until);
        }
    }

    fn clear_status(&mut self, owner: ObjectSlot, status: ObjectStatusType) {
        self.change_status(owner, status, 0);
    }

    fn set_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    ) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let old = obj.weapon_bonus_condition;
        obj.set_weapon_bonus_condition(cond);
        if old != obj.weapon_bonus_condition {
            self.weapon_set_on_weapon_bonus_change(owner);
        }
    }

    fn clear_weapon_bonus_condition(
        &mut self,
        owner: ObjectSlot,
        cond: WeaponBonusConditionType,
    ) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let old = obj.weapon_bonus_condition;
        obj.clear_weapon_bonus_condition(cond);
        if old != obj.weapon_bonus_condition {
            self.weapon_set_on_weapon_bonus_change(owner);
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
        let Some((old_team, contained, under_const, sold, ai_slot, contain_slot)) =
            self.world.objects.get(owner).map(|obj| {
                (
                    obj.team_id,
                    obj.contained_by.is_some(),
                    obj.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE,
                    obj.status.test(ObjectStatusType::Sold.as_i32()) != FALSE,
                    obj.ai_slot,
                    obj.contain_slot,
                )
            })
        else {
            return;
        };
        // An object without a team has no controlling player.
        if contained || under_const || sold || old_team == TeamId::INVALID {
            return;
        }
        if !self.world.player_default_teams.is_empty()
            && self.controlling_player_index(old_team).is_none()
        {
            return;
        }
        let my_team = self.player_default_team(old_team);
        if my_team == new_team {
            return;
        }
        let _ = self.with_production(owner, |p, sim, ctx| {
            p.cancel_and_refund_all_production(sim, ctx);
        });
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.set_undetected_defector(Bool::from(detection_frames > 0));
        }
        self.trigger_defection(owner, detection_frames, with_fx);
        self.set_object_team(owner, new_team);
        self.handle_partition_cell_maintenance(owner);
        self.ai_idle(owner, CommandSourceType::FromAi);
        if let Some(ai_ms) = ai_slot {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_goal_none(sim, ctx);
                }
            });
        }
        let kick_out = contain_slot.is_some_and(|cs| {
            self.world
                .modules
                .payload(cs)
                .and_then(|p| p.as_contain())
                .is_some_and(|c| {
                    let ctx = self.world.modules.meta(cs).context(cs);
                    c.is_kick_out_on_capture(self, ctx) != FALSE
                })
        });
        if kick_out {
            self.contain_remove_all(owner, TRUE);
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
        let _ = self.select_and_trigger_slow_death(owner, None, FALSE, 0, dmg);
    }

    fn add_player_money(&mut self, team_id: TeamId, amount: Int) -> Int {
        // The same `Money::deposit`; a negative amount converts as in C++.
        self.deposit_money(team_id, amount as UnsignedInt);
        amount
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
        let list = self.world.player_cost_modifiers.entry(team_id).or_default();
        if let Some((_, _, refs)) = list
            .iter_mut()
            .find(|(k, p, _)| k == kindof && p.to_bits() == percent.to_bits())
        {
            *refs += 1;
            return;
        }
        list.push((kindof.clone(), percent, 1));
    }

    fn remove_cost_modifier(&mut self, team_id: TeamId, kindof: &KindOfMaskType, percent: Real) {
        if let Some(list) = self.world.player_cost_modifiers.get_mut(&team_id) {
            if let Some(pos) = list
                .iter()
                .position(|(k, p, _)| k == kindof && p.to_bits() == percent.to_bits())
            {
                list[pos].2 -= 1;
                if list[pos].2 <= 0 {
                    list.remove(pos);
                }
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
        let has_drawable = self.world.objects.is_in_world_list(owner);
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.weapon_set_flags.set(flag.as_i32(), i32::from(set != FALSE));
            // `Object::setWeaponSetFlag` mirrors the flag onto the drawable.
            if let Some(condition) = weapon_set_model_condition(flag).filter(|_| has_drawable) {
                if set != FALSE {
                    obj.set_model_condition_state(condition);
                } else {
                    obj.clear_model_condition_state(condition);
                }
            }
        }
        // `updateWeaponSet` builds the weapons at the first flag, also from a module's
        // `onObjectCreated`. Later changes of set are not modelled.
        if !has_drawable {
            self.build_first_weapon_set(owner);
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

    fn set_vision_range(&mut self, owner: ObjectSlot, range: Real) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.vision_range = range;
        }
    }

    fn set_shroud_clearing_range(&mut self, owner: ObjectSlot, range: Real) {
        let Some(pos) = self.world.objects.get_mut(owner).and_then(|obj| {
            if range == obj.shroud_clearing_range {
                None
            } else {
                obj.shroud_clearing_range = range;
                Some(*obj.thing.get_position())
            }
        }) else {
            return;
        };
        if pos.x != 0.0 || pos.y != 0.0 || pos.z != 0.0 {
            self.handle_partition_cell_maintenance(owner);
        }
    }

    fn set_shroud_range(&mut self, owner: ObjectSlot, range: Real) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.shroud_range = range;
        }
    }

    fn set_vision_spied(&mut self, owner: ObjectSlot, setting: Bool, by_whom: Int) {
        if !(0..MAX_PLAYER_COUNT as Int).contains(&by_whom) {
            return;
        }
        let idx = by_whom as usize;
        let need_refresh = {
            let Some(obj) = self.world.objects.get_mut(owner) else {
                return;
            };
            let refresh = if setting != FALSE {
                obj.vision_spied_by[idx] = obj.vision_spied_by[idx].wrapping_add(1);
                obj.vision_spied_by[idx] == 1
            } else {
                obj.vision_spied_by[idx] = obj.vision_spied_by[idx].wrapping_sub(1);
                obj.vision_spied_by[idx] == 0
            };
            if refresh {
                let mut working_mask: PlayerMaskType = 0;
                for (i, &count) in obj.vision_spied_by.iter().enumerate() {
                    if count > 0 {
                        working_mask |= 1u16 << (i as u32);
                    }
                }
                obj.vision_spied_mask = working_mask;
            }
            refresh
        };
        if need_refresh {
            self.handle_partition_cell_maintenance(owner);
        }
    }

    fn handle_partition_cell_maintenance(&mut self, owner: ObjectSlot) {
        self.handle_shroud(owner);
    }

    fn set_enemy_units_vision_spied(
        &mut self,
        actor_team: TeamId,
        setting: Bool,
        which_units: &KindOfMaskType,
    ) {
        if actor_team == TeamId::INVALID {
            return;
        }
        let actor_player_idx = self.player_index_for_team(actor_team);
        let actor_def_team = self.player_default_team(actor_team);
        let players: Vec<Int> = self.world.player_default_teams.keys().copied().collect();
        for target_idx in players {
            if self.team_relationship_to_player(actor_def_team, target_idx)
                == Relationship::Enemies
            {
                for slot in self.player_object_slots(target_idx) {
                    if self.world.objects.get(slot).is_some_and(|o| {
                        o.thing
                            .template_override()
                            .get_final(&self.rules.thing_factory)
                            .is_some_and(|t| t.is_any_kind_of(which_units) != FALSE)
                    }) {
                        self.set_vision_spied(slot, setting, actor_player_idx);
                    }
                }
            }
        }
    }

    fn activate_spy_vision(&mut self, owner: ObjectSlot, duration: UnsignedInt) {
        let Some(sv_key) = self.rules.name_keys.lookup_key("SpyVisionUpdate") else {
            return;
        };
        let slot = self.world.objects.get(owner).and_then(|obj| {
            obj.behaviors.iter().find_map(|&(ms, mask)| {
                if !mask.contains(FacetMask::UPDATE) {
                    return None;
                }
                let name_key = self
                    .world
                    .modules
                    .payload(ms)
                    .map(|m| m.module_name_key());
                (name_key == Some(sv_key)).then_some(ms)
            })
        });
        if let Some(ms) = slot {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(sv) = m.as_any_mut().and_then(|a| a.downcast_mut::<SpyVisionUpdate>())
                {
                    sv.activate_spy_vision(sim, ctx, duration);
                }
            });
        }
    }

    fn disable_internet_center_spy_vision(&mut self, victim: ObjectSlot, until_frame: UnsignedInt) {
        let Some(victim_team) = self.world.objects.get(victim).map(|o| o.team_id) else {
            return;
        };
        if victim_team == TeamId::INVALID {
            return;
        }
        let victim_player_idx = self.player_index_for_team(victim_team);
        for slot in self.player_object_slots(victim_player_idx) {
            if self.object_is_kind_of(slot, KindOfType::FsInternetCenter) == FALSE {
                continue;
            }
            let beh_slots: Vec<ModuleSlot> = self
                .world
                .objects
                .get(slot)
                .map_or_else(Vec::new, |o| o.behaviors.iter().map(|&(ms, _)| ms).collect());
            for ms in beh_slots {
                if self.world.modules.is_alive(ms)
                    && self
                        .world
                        .modules
                        .payload(ms)
                        .and_then(|m| m.as_any())
                        .is_some_and(|a| a.is::<SpyVisionUpdate>())
                {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        if let Some(sv) =
                            m.as_any_mut().and_then(|a| a.downcast_mut::<SpyVisionUpdate>())
                        {
                            sv.set_disabled_until_frame(sim, ctx, until_frame);
                        }
                    });
                }
            }
        }
    }

    fn set_deletion_lifetime(
        &mut self,
        owner: ObjectSlot,
        min_frames: UnsignedInt,
        max_frames: UnsignedInt,
    ) {
        let Some(del_key) = self.rules.name_keys.lookup_key("DeletionUpdate") else {
            return;
        };
        let slot = self.world.objects.get(owner).and_then(|obj| {
            obj.behaviors.iter().find_map(|&(ms, _)| {
                let name_key = self
                    .world
                    .modules
                    .payload(ms)
                    .map(|m| m.module_name_key());
                (name_key == Some(del_key)).then_some(ms)
            })
        });
        if let Some(ms) = slot {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(del) = m
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<DeletionUpdate>())
                {
                    del.set_lifetime_range(sim, ctx, min_frames, max_frames);
                }
            });
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
        if let Some(cs) = self.world.objects.get(owner).and_then(|o| o.contain_slot) {
            if !self.world.modules.is_executing(cs) {
                let _ = self.dispatch_module(cs, |m, _, _| {
                    if let Some(c) = m.as_contain_mut() {
                        c.set_passenger_allowed_to_fire(allowed);
                    }
                });
            }
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

    fn retract_power_plant(&mut self, owner: ObjectSlot) {
        self.extend_rods(owner, FALSE);
    }

    fn set_dock_open(&mut self, owner: ObjectSlot, open: Bool) {
        let Some(ds) = self.dock_slot(owner) else {
            return;
        };
        let _ = self.dispatch_module(ds, |m, _, _| {
            if let Some(d) = m.as_dock_mut() {
                d.set_dock_open(open);
            }
        });
    }

    fn set_dock_crippled(&mut self, owner: ObjectSlot, crippled: Bool) {
        let Some(ds) = self.dock_slot(owner) else {
            return;
        };
        let _ = self.dispatch_module(ds, |m, sim, ctx| {
            if let Some(d) = m.as_dock_mut() {
                d.set_dock_crippled(sim, ctx, crippled);
            }
        });
    }

    fn with_supply_truck(
        &mut self,
        owner: ObjectSlot,
        f: &mut dyn FnMut(&mut dyn SupplyTruckAiFacet, &mut dyn SimApi, ModuleContext),
    ) -> Bool {
        let Some(ai) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return FALSE;
        };
        self.dispatch_module(ai, |m, sim, ctx| {
            match m.as_ai_mut().and_then(|ai| ai.supply_truck()) {
                Some(truck) => {
                    f(truck, sim, ctx);
                    TRUE
                }
                None => FALSE,
            }
        })
        .unwrap_or(FALSE)
    }

    fn add_money_earned(&mut self, team_id: TeamId, amount: Int) {
        let player = self.player_index_for_team(team_id);
        self.world
            .score_keepers
            .entry(player)
            .or_default()
            .add_money_earned(amount);
    }

    fn receive_temporary_stealth_grant(&mut self, owner: ObjectSlot, frames: UnsignedInt) {
        self.receive_stealth_grant(owner, TRUE, frames);
    }

    fn extend_power_plant(&mut self, owner: ObjectSlot) {
        self.extend_rods(owner, TRUE);
    }

    fn extend_radar_update(&mut self, owner: ObjectSlot) {
        for ms in self.behavior_slots(owner) {
            let mut handled = false;
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(f) = m.as_radar_mut() {
                    f.extend_radar(sim, ctx);
                    handled = true;
                }
            });
            if handled {
                break;
            }
        }
    }

    fn add_power_bonus(&mut self, team_id: TeamId, owner: ObjectSlot) {
        let info = self.energy_object_info(owner);
        let now = self.world.frame;
        self.world
            .player_energy
            .entry(team_id)
            .or_default()
            .add_power_bonus(info, now, &mut NoopEnergyCallbacks);
    }

    fn remove_power_bonus(&mut self, team_id: TeamId, owner: ObjectSlot) {
        let info = self.energy_object_info(owner);
        let now = self.world.frame;
        self.world
            .player_energy
            .entry(team_id)
            .or_default()
            .remove_power_bonus(info, now, &mut NoopEnergyCallbacks);
    }

    fn adjust_player_power(&mut self, team_id: TeamId, delta: Int, adding: Bool) {
        let now = self.world.frame;
        self.world
            .player_energy
            .entry(team_id)
            .or_default()
            .adjust_power(delta, adding, now, &mut NoopEnergyCallbacks);
    }

    fn sabotage_power(&mut self, team_id: TeamId, frames: UnsignedInt) {
        let until = self.world.frame.saturating_add(frames);
        self.world
            .player_power_sabotaged_until
            .insert(team_id, until);
        self.world
            .player_energy
            .entry(team_id)
            .or_default()
            .set_power_sabotaged_till_frame(until);
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

    fn unpause_special_power(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        let now = self.world.frame;
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::SPECIAL_POWER))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let (match_ok, sidecar_starts_ready) = if let Some(p) = self.world.modules.payload(ms) {
                let ctx = self.world.modules.meta(ms).context(ms);
                let ok = p
                    .as_special_power()
                    .is_some_and(|sp| sp.is_module_for_power(self, ctx, template_id) != FALSE);
                (ok, FALSE)
            } else {
                match self.world.modules.sidecar(ms) {
                    ModuleSidecar::SpecialPower {
                        template_id: tid,
                        starts_ready,
                        ..
                    } if tid == template_id => (true, starts_ready),
                    _ => (false, FALSE),
                }
            };
            if match_ok {
                if self.world.modules.is_executing(ms) {
                    let _ = self
                        .world
                        .modules
                        .push_deferred_op(ms, DeferredSelfOp::SpecialPowerPause { pause: FALSE });
                    if sidecar_starts_ready != FALSE {
                        let _ = self.world.modules.push_deferred_op(
                            ms,
                            DeferredSelfOp::SpecialPowerSetReadyFrame(now),
                        );
                    }
                } else {
                    let _ = self.dispatch_module(ms, |m, sim, ctx| {
                        if let Some(sp) = m.as_special_power_mut() {
                            sp.pause_countdown(sim, ctx, FALSE);
                            if sp.starts_ready(sim, ctx) != FALSE {
                                let cur = sim.frame();
                                sp.set_ready_frame(sim, ctx, cur);
                            }
                        }
                    });
                }
            }
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

    fn reveal_map_for_player(&mut self, team_id: TeamId) {
        let idx = self.player_index_for_team(team_id);
        if (0..MAX_PLAYER_COUNT as Int).contains(&idx) {
            self.world.partition.reveal_map_for_player(idx as usize);
        }
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
        if let Some(dead_tmpl) = self
            .world
            .objects
            .get(dead_obj)
            .and_then(|o| o.thing.template_override().get_non_overloaded_key())
        {
            self.start_rebuild_hole(hole_obj, dead_tmpl, dead_id);
        }
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
            // Tested before each module, in case a collide module sets it.
            if self.world.objects.get(owner).is_some_and(|o| {
                o.status.test(ObjectStatusType::NoCollisions.as_i32()) != FALSE
            }) {
                break;
            }
            // `PhysicsBehavior` keeps its members on the object, so it also answers
            // from inside its own update (the landing collision).
            if physics_update::physics(self, owner).is_some_and(|p| p.mod_slot == ms) {
                physics_update::on_collide(self, owner, other, loc, normal);
                continue;
            }
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(c) = m.as_collide_mut() {
                    c.on_collide(sim, ctx, other, loc, normal);
                }
            });
        }
    }

    fn capture_object(&mut self, owner: ObjectSlot, new_team: TeamId) {
        // `Player::becomingTeamMember(obj, false)`, while the old team still controls it.
        if let Some(old) = self.world.objects.get(owner).map(|o| o.team_id) {
            if old != new_team {
                self.adjust_team_power(owner, old, FALSE);
            }
        }
        let Some((old_team, beh_slots)) = self.world.objects.get_mut(owner).and_then(|obj| {
            if obj.team_id == new_team {
                return None;
            }
            let old = obj.team_id;
            obj.team_id = new_team;
            let slots: Vec<ModuleSlot> = obj.behaviors.iter().map(|&(ms, _)| ms).collect();
            Some((old, slots))
        }) else {
            return;
        };
        // `Player::becomingTeamMember(obj, false)`.
        if old_team != TeamId::INVALID {
            battle_plan_update::battle_plan_team_member(self, old_team, owner, FALSE);
        }
        self.world.team_member_remove(old_team, owner);
        // `TeamId::INVALID` is a null `Team*`: `setTeam` runs `onCapture` only between two
        // teams.
        if new_team == TeamId::INVALID {
            return;
        }
        self.world.team_member_prepend(new_team, owner);
        // `Player::becomingTeamMember(obj, true)`, before `onCapture`.
        self.adjust_team_power(owner, new_team, TRUE);
        self.award_capture_bonus(owner);
        battle_plan_update::battle_plan_team_member(self, new_team, owner, TRUE);
        if old_team == TeamId::INVALID {
            return;
        }
        self.idle_on_capture(owner, old_team, new_team);
        *self.world.player_captured_objects.entry(new_team).or_insert(0) += 1;
        self.record_object_captured(owner, new_team);
        for ms in beh_slots {
            if self.world.modules.is_alive(ms) {
                let _ = self.dispatch_module(ms, |m, sim, ctx| {
                    m.on_capture(sim, ctx, old_team, new_team);
                });
            }
        }
        self.handle_partition_cell_maintenance(owner);
        self.clear_unsellable(owner);
    }

    fn set_crushed(&mut self, owner: ObjectSlot, front: Option<Bool>, back: Option<Bool>) {
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
                mut front_crushed,
                mut back_crushed,
                indestructible,
                current_subdual_damage,
                subdued,
                last_damage_ts,
                last_healing_ts,
                armor_set_bits,
            } = self.world.modules.sidecar(bs)
            {
                if let Some(f) = front {
                    front_crushed = f;
                }
                if let Some(bk) = back {
                    back_crushed = bk;
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
                .push_deferred_op(bs, DeferredSelfOp::SetCrushed { front, back });
            return;
        }
        let _ = self.dispatch_module(bs, |m, _, _| {
            if let Some(b) = m.as_body_mut() {
                if let Some(f) = front {
                    b.set_front_crushed(f);
                }
                if let Some(bk) = back {
                    b.set_back_crushed(bk);
                }
            }
        });
    }

    fn set_special_power_creator(&mut self, owner: ObjectSlot, creator_id: ObjectId) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::DIE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, _, _| {
                if let Some(d) = m.as_die_mut() {
                    d.set_creator(creator_id);
                }
            });
        }
    }

    fn set_geometry_info(&mut self, owner: ObjectSlot, geom: &GeometryInfo) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.geometry = geom.clone();
        let id = obj.id;
        // The number of cells an object may occupy is fixed per registration.
        if self.world.partition.is_registered(id) {
            self.world.partition.unregister(id);
            self.world.partition.register(id);
        }
    }

    fn set_demoralized(&mut self, owner: ObjectSlot, frames: UnsignedInt) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.demoralized_events.push((id, frames));
        }
    }

    fn reset_supply_dropzone_timer(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.supply_dropzone_resets.push(id);
        }
        // `findUpdateModule("OCLUpdate")->resetTimer()`.
        if let Some(ms) = self.module_by_name(owner, "OCLUpdate") {
            let _ = self.dispatch_module(ms, |m, sim, _| {
                if let Some(ocl) = m.as_any_mut().and_then(|a| a.downcast_mut::<OclUpdate>()) {
                    ocl.reset_timer(sim);
                }
            });
        }
    }

    fn eject_pilot_dying(&mut self, owner: ObjectSlot) {
        let slots: Vec<ModuleSlot> = self.world.objects.get(owner).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::EJECT_PILOT_DIE))
                .map(|&(s, _)| s)
                .collect()
        });
        for ms in slots {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(d) = m.as_die_mut() {
                    d.eject_pilot_dying(sim, ctx);
                }
            });
        }
    }

    fn update_upgrade_modules(&mut self, owner: ObjectSlot) {
        let Some((team_id, obj_mask, under_const, destroyed)) = self.world.objects.get(owner).map(|o| {
            (
                o.team_id,
                o.upgrades_completed.clone(),
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE,
                o.is_destroyed(),
            )
        }) else {
            return;
        };
        if under_const || destroyed {
            return;
        }
        let empty_mask = UpgradeMaskType::new();
        let mut mask_to_check = self
            .world
            .player_upgrades
            .get(&team_id)
            .cloned()
            .unwrap_or_default();
        mask_to_check.clear_and_set(&empty_mask, &obj_mask);
        self.dispatch_attempt_upgrade(owner, &mask_to_check);
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
                    if u.is_already_upgraded() == FALSE {
                        let _ = u.attempt_upgrade(sim, ctx, mask);
                    }
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

    fn contain_add(&mut self, container: ObjectSlot, passenger: ObjectSlot) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if self.world.modules.is_executing(cs) {
            let _ = self
                .world
                .modules
                .push_deferred_op(cs, DeferredSelfOp::ContainAdd(passenger));
            return;
        }
        let _ = self.dispatch_module(cs, |m, sim, ctx| {
            if let Some(c) = m.as_contain_mut() {
                c.add_to_contain(sim, ctx, passenger);
            }
        });
    }

    fn contain_remove(
        &mut self,
        container: ObjectSlot,
        passenger: ObjectSlot,
        expose_stealthed: Bool,
    ) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if self.world.modules.is_executing(cs) {
            let _ = self.world.modules.push_deferred_op(
                cs,
                DeferredSelfOp::ContainRemove {
                    obj: passenger,
                    expose_stealthed,
                },
            );
            return;
        }
        let _ = self.dispatch_module(cs, |m, sim, ctx| {
            if let Some(c) = m.as_contain_mut() {
                c.remove_from_contain(sim, ctx, passenger, expose_stealthed);
            }
        });
    }

    fn contain_remove_all(&mut self, container: ObjectSlot, expose_stealthed: Bool) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if self.world.modules.is_executing(cs) {
            let _ = self
                .world
                .modules
                .push_deferred_op(cs, DeferredSelfOp::ContainRemoveAll { expose_stealthed });
            return;
        }
        let _ = self.dispatch_module(cs, |m, sim, ctx| {
            if let Some(c) = m.as_contain_mut() {
                c.remove_all_contained(sim, ctx, expose_stealthed);
            }
        });
    }

    fn contain_kill_all(&mut self, container: ObjectSlot) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if !self.world.modules.is_executing(cs) {
            let _ = self.dispatch_module(cs, |m, sim, ctx| {
                if let Some(c) = m.as_contain_mut() {
                    c.kill_all_contained(sim, ctx);
                }
            });
        }
    }

    fn contain_harm_and_force_exit_all(&mut self, container: ObjectSlot, dmg: &DamageInfo) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if !self.world.modules.is_executing(cs) {
            let _ = self.dispatch_module(cs, |m, sim, ctx| {
                if let Some(c) = m.as_contain_mut() {
                    c.harm_and_force_exit_all_contained(sim, ctx, dmg);
                }
            });
        }
    }

    fn set_body_damage_state(&mut self, owner: ObjectSlot, new_state: BodyDamageType) {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return;
        };
        if !self.world.modules.is_executing(bs) {
            let _ = self.dispatch_module(bs, |m, sim, ctx| {
                if let Some(b) = m.as_body_mut() {
                    b.set_damage_state(sim, ctx, new_state);
                }
            });
        }
    }

    fn set_contained_by(&mut self, passenger: ObjectSlot, container: Option<ObjectSlot>) {
        let frame = self.world.frame;
        if let Some(obj) = self.world.objects.get_mut(passenger) {
            obj.contained_by = container;
            obj.contained_by_frame = if container.is_some() { frame } else { 0 };
        }
    }

    fn set_in_partition(&mut self, owner: ObjectSlot, present: Bool) {
        let Some(id) = self.world.objects.id_of(owner) else {
            return;
        };
        if present != FALSE {
            self.world.partition.register(id);
        } else {
            self.world.partition.unregister(id);
        }
    }

    fn set_object_position(&mut self, owner: ObjectSlot, pos: &Coord3D) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let old_pos = *obj.thing.get_position();
        let old_angle = obj.thing.get_orientation();
        let mut dummy = crate::object::thing::NoopTransformReactor;
        obj.thing
            .set_position(&self.rules.thing_factory, pos, self.terrain.as_ref(), &mut dummy);
        self.react_to_transform_change(owner, &old_pos, old_angle);
    }

    fn set_object_orientation(&mut self, owner: ObjectSlot, angle: Real) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let old_pos = *obj.thing.get_position();
        let old_angle = obj.thing.get_orientation();
        let mut dummy = crate::object::thing::NoopTransformReactor;
        obj.thing
            .set_orientation(&self.rules.thing_factory, angle, self.terrain.as_ref(), &mut dummy);
        self.react_to_transform_change(owner, &old_pos, old_angle);
    }

    fn set_object_layer(&mut self, owner: ObjectSlot, layer: PathfindLayerEnum) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.layer = layer;
        }
    }

    fn set_object_transform_matrix(&mut self, owner: ObjectSlot, mtx: &Matrix3D) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        let old_pos = *obj.thing.get_position();
        let old_angle = obj.thing.get_orientation();
        let mut dummy = crate::object::thing::NoopTransformReactor;
        obj.thing.set_transform_matrix(mtx, &mut dummy);
        let pos = *obj.thing.get_position();
        // `Object::reactToTransformChange` deletes an object at a NaN position.
        if pos.x.is_nan() || pos.y.is_nan() || pos.z.is_nan() {
            self.destroy_object_now(owner);
        }
        self.react_to_transform_change(owner, &old_pos, old_angle);
    }

    fn ai_process_collision(&mut self, owner: ObjectSlot, other: ObjectSlot) -> Bool {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return FALSE;
        };
        self.dispatch_module(ms, |m, sim, ctx| {
            m.as_ai_mut()
                .map_or(FALSE, |ai| ai.process_collision(sim, ctx, other))
        })
        .unwrap_or(FALSE)
    }

    fn projectile_handle_collision(
        &mut self,
        owner: ObjectSlot,
        other: Option<ObjectSlot>,
    ) -> Bool {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.projectile_slot) else {
            return FALSE;
        };
        self.dispatch_module(ms, |m, sim, ctx| {
            m.as_projectile_mut()
                .map_or(FALSE, |p| p.projectile_handle_collision(sim, ctx, other))
        })
        .unwrap_or(FALSE)
    }

    fn tunnel_tracker_mut(&mut self, player_index: Int) -> &mut TunnelTrackerState {
        self.world.tunnel_trackers.entry(player_index).or_default()
    }

    fn cave_register(&mut self, cave_index: Int) {
        self.world.cave_system.register_new_cave(cave_index);
    }

    fn cave_unregister(&mut self, cave_index: Int) {
        self.world.cave_system.unregister_cave(cave_index);
    }

    fn cave_set_original_team(&mut self, cave: ObjectSlot, team: TeamId) -> Bool {
        let slots: Vec<ModuleSlot> = self.world.objects.get(cave).map_or_else(Vec::new, |o| {
            o.behaviors
                .iter()
                .filter(|(_, m)| m.contains(FacetMask::CAVE_INTERFACE))
                .map(|&(s, _)| s)
                .collect()
        });
        let Some(&ms) = slots.first() else {
            return FALSE;
        };
        let mut found = FALSE;
        let _ = self.dispatch_module(ms, |m, _, _| {
            if let Some(c) = m.as_cave_mut() {
                c.set_original_team(team);
                found = TRUE;
            }
        });
        found
    }

    fn cave_tracker_mut(&mut self, cave_index: Int) -> Option<&mut TunnelTrackerState> {
        self.world
            .cave_system
            .get_tunnel_tracker_for_cave_index_mut(cave_index)
    }

    fn launch_projectile(
        &mut self,
        proj: ObjectSlot,
        victim: Option<ObjectSlot>,
        victim_pos: &Coord3D,
        launcher: ObjectSlot,
        wslot: WeaponSlotType,
        weapon_name: &str,
    ) {
        let proj_id = self.world.objects.id_of(proj).unwrap_or(ObjectId::INVALID);
        let victim_id = victim.and_then(|v| self.world.objects.id_of(v));
        self.world
            .projectile_launches
            .push((proj_id, victim_id, *victim_pos));

        if let Some(ps) = self.world.objects.get(proj).and_then(|o| o.projectile_slot) {
            let det_name = if weapon_name.is_empty() {
                None
            } else {
                Some(weapon_name)
            };
            let _ = self.dispatch_module(ps, |m, sim, ctx| {
                if let Some(p) = m.as_projectile_mut() {
                    p.projectile_launch_at_object_or_position(
                        sim,
                        ctx,
                        victim,
                        victim_pos,
                        Some(launcher),
                        wslot,
                        0,
                        det_name,
                        None,
                    );
                }
            });
        }
    }

    fn report_missile_to_countermeasures(&mut self, target: ObjectSlot, missile: ObjectSlot) {
        let target_id = self.world.objects.id_of(target).unwrap_or(ObjectId::INVALID);
        let missile_id = self
            .world
            .objects
            .id_of(missile)
            .unwrap_or(ObjectId::INVALID);
        self.world
            .countermeasure_reports
            .push((target_id, missile_id));
        if let Some(cm_slot) = self
            .world
            .objects
            .get(target)
            .and_then(|o| o.countermeasures_slot)
        {
            if !self.world.modules.is_executing(cm_slot) {
                let _ = self.dispatch_module(cm_slot, |m, sim, ctx| {
                    if let Some(cm) = m.as_countermeasures_mut() {
                        cm.report_missile_for_countermeasures(sim, ctx, missile);
                    }
                });
            }
        }
    }

    fn divert_projectile_to_target(&mut self, missile: ObjectSlot, frames: UnsignedInt) {
        if let Some(ps) = self
            .world
            .objects
            .get(missile)
            .and_then(|o| o.projectile_slot)
        {
            let _ = self.dispatch_module(ps, |m, _, _| {
                if let Some(p) = m.as_projectile_mut() {
                    p.set_frames_till_countermeasure_diversion_occurs(frames);
                }
            });
        }
    }

    fn set_bridge_passable(&mut self, bridge_id: ObjectId, passable: Bool) {
        self.world.bridge_passable.insert(bridge_id, passable);
    }

    fn notify_bridge_tower_damage(&mut self, bridge_slot: ObjectSlot, dmg: &mut DamageInfo) {
        let Some(bs) = self.world.objects.get(bridge_slot).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(bs) {
            return;
        }
        self.attempt_damage_on(bridge_slot, dmg);
    }

    fn link_bridge_tower(
        &mut self,
        bridge_id: ObjectId,
        tower_type: BridgeTowerType,
        tower_id: ObjectId,
    ) {
        let entry = self
            .world
            .bridge_links
            .entry(bridge_id)
            .or_insert([ObjectId::INVALID; 4]);
        let idx = tower_type.as_i32();
        if (0..4).contains(&idx) {
            entry[idx as usize] = tower_id;
        }
    }

    fn reserve_parking_space(
        &mut self,
        airfield: ObjectSlot,
        aircraft_id: ObjectId,
        offset: Real,
    ) -> Option<ParkingPlaceInfo> {
        let ps = self.world.objects.get(airfield).and_then(|o| o.parking_slot)?;
        let airfield_id = self
            .world
            .objects
            .id_of(airfield)
            .unwrap_or(ObjectId::INVALID);
        let mut info = ParkingPlaceInfo::default();
        let reserved = self
            .dispatch_module(ps, |m, sim, ctx| {
                m.as_parking_place_mut()
                    .map_or(FALSE, |pp| pp.reserve_space(sim, ctx, aircraft_id, offset, Some(&mut info)))
            })
            .unwrap_or(FALSE);
        if reserved != FALSE {
            self.world
                .parking_reservations
                .insert(aircraft_id, airfield_id);
            Some(info)
        } else {
            None
        }
    }

    fn release_parking_space(&mut self, airfield: ObjectSlot, aircraft_id: ObjectId) {
        self.world.parking_reservations.remove(&aircraft_id);
        let Some(ps) = self.world.objects.get(airfield).and_then(|o| o.parking_slot) else {
            return;
        };
        if self.world.modules.is_executing(ps) {
            let _ = self
                .world
                .modules
                .push_deferred_op(ps, DeferredSelfOp::ParkingRelease(aircraft_id));
            return;
        }
        let _ = self.dispatch_module(ps, |m, sim, ctx| {
            if let Some(pp) = m.as_parking_place_mut() {
                pp.release_space(sim, ctx, aircraft_id);
            }
        });
    }

    fn notify_spawn_slave_died(
        &mut self,
        master: ObjectSlot,
        slave_id: ObjectId,
        dmg: &DamageInfo,
    ) {
        let Some(ss) = self.world.objects.get(master).and_then(|o| o.spawn_slot) else {
            return;
        };
        if self.world.modules.is_executing(ss) {
            let _ = self
                .world
                .modules
                .push_deferred_op(
                    ss,
                    DeferredSelfOp::SpawnNotifySlaveDied {
                        slave_id,
                        source_id: dmg.input.source_id,
                    },
                );
            return;
        }
        let _ = self.dispatch_module(ss, |m, sim, ctx| {
            if let Some(sb) = m.as_spawn_behavior_mut() {
                sb.on_spawn_death(sim, ctx, slave_id, Some(dmg));
            }
        });
    }

    fn start_rebuild_hole(
        &mut self,
        hole: ObjectSlot,
        rebuild_tmpl: ThingTemplateId,
        spawner_id: ObjectId,
    ) {
        let Some(rs) = self
            .world
            .objects
            .get(hole)
            .and_then(|o| o.rebuild_hole_slot)
        else {
            return;
        };
        if !self.world.modules.is_executing(rs) {
            let _ = self.dispatch_module(rs, |m, sim, ctx| {
                if let Some(rh) = m.as_rebuild_hole_mut() {
                    rh.start_rebuild_process(sim, ctx, rebuild_tmpl, spawner_id);
                }
            });
        }
    }

    fn record_mine_detonation(&mut self, mine_id: ObjectId, trigger_id: ObjectId) {
        self.world.mine_detonations.push((mine_id, trigger_id));
    }

    fn order_passenger(&mut self, passenger_id: ObjectId, order: &str) {
        self.world
            .passenger_orders
            .push((passenger_id, order.to_owned()));
    }

    fn ai_mark_as_dead(&mut self, owner: ObjectSlot) {
        let ai_slot = match self.world.objects.get_mut(owner) {
            Some(obj) if obj.ai_slot.is_some() => {
                obj.ai_dead = TRUE;
                obj.ai_slot
            }
            _ => return,
        };
        if let Some(ms) = ai_slot {
            if !self.world.modules.is_executing(ms) {
                let _ = self.dispatch_module(ms, |m, _, _| {
                    if let Some(ai) = m.as_ai_mut() {
                        ai.mark_as_dead();
                    }
                });
            }
        }
    }

    fn deselect_object(
        &mut self,
        owner: ObjectSlot,
        player_mask: UnsignedShort,
        affect_drag_selection: Bool,
    ) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world
                .deselected_objects
                .push((id, player_mask, affect_drag_selection));
        }
    }

    fn trigger_slow_death_with_caller(
        &mut self,
        owner: ObjectSlot,
        caller_slot: ModuleSlot,
        caller_applicable: Bool,
        caller_prob: Int,
        dmg: &DamageInfo,
    ) -> Bool {
        self.select_and_trigger_slow_death(
            owner,
            Some(caller_slot),
            caller_applicable,
            caller_prob,
            dmg,
        )
    }

    fn apply_fling_physics(&mut self, owner: ObjectSlot, force: &Coord3D, orientation: Real) {
        if physics_update::physics(self, owner).is_none() {
            return;
        }
        self.set_physics_allow_to_fall(owner, TRUE);
        physics_update::apply_force(self, owner, force);
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.extra_bounciness = -1.0;
            p.extra_friction = -3.0_f32 * (1.0_f32 / 30.0_f32);
            p.set_flag(physics_update::ALLOW_BOUNCE, TRUE);
        }
        physics_update::set_angles(self, owner, orientation, 0.0, 0.0);
    }

    fn notify_slaver_die(&mut self, owner: ObjectSlot) {
        let Some(obj) = self.world.objects.get(owner) else {
            return;
        };
        let slaved_slot = obj.slaved_slot;
        if let Some(ss) = slaved_slot {
            if !self.world.modules.is_executing(ss) {
                let _ = self.dispatch_module(ss, |m, sim, ctx| {
                    if let Some(su) = m.as_slaved_update_mut() {
                        su.on_slaver_die(sim, ctx, None);
                    }
                });
            }
        }
    }

    fn mark_stealth_detected(&mut self, owner: ObjectSlot, frames: UnsignedInt) {
        let Some(obj) = self.world.objects.get(owner) else {
            return;
        };
        let obj_id = obj.id;
        let stealth_slot = obj.stealth_slot;
        self.world.stealth_detected_events.push((obj_id, frames));
        if let Some(ss) = stealth_slot {
            if !self.world.modules.is_executing(ss) {
                let _ = self.dispatch_module(ss, |m, sim, ctx| {
                    if let Some(st) = m.as_stealth_mut() {
                        st.mark_as_detected(sim, ctx, frames);
                    }
                });
            }
        }
    }

    fn receive_stealth_grant(&mut self, owner: ObjectSlot, active: Bool, frames: UnsignedInt) {
        let Some((obj_id, Some(ss))) = self
            .world
            .objects
            .get(owner)
            .map(|o| (o.id, o.stealth_slot))
        else {
            return;
        };
        if active != FALSE {
            self.world.stealth_granted.insert(obj_id);
        }
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, ctx| {
                if let Some(st) = m.as_stealth_mut() {
                    st.receive_grant(sim, ctx, active, frames);
                }
            });
        }
    }

    fn rng_client_real(&mut self, lo: Real, hi: Real) -> Real {
        self.world.rng.get_game_client_random_value_real(lo, hi)
    }

    fn reset_dynamic_physics(&mut self, owner: ObjectSlot) {
        physics_update::reset_dynamic_physics(self, owner);
    }

    fn record_vehicle_disguised(&mut self, owner: ObjectSlot) {
        let Some(team) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        self.academy(team).record_vehicle_disguised();
    }

    fn contain_recalc_apparent_controlling_player(&mut self, container: ObjectSlot) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if !self.world.modules.is_executing(cs) {
            let _ = self.dispatch_module(cs, |m, sim, ctx| {
                if let Some(c) = m.as_contain_mut() {
                    c.recalc_apparent_controlling_player(sim, ctx);
                }
            });
        }
    }

    fn spawn_all_slaves_stealthed(&mut self, owner: ObjectSlot) -> Option<Bool> {
        let ss = self.world.objects.get(owner)?.spawn_slot?;
        if self.world.modules.is_executing(ss) {
            return None;
        }
        self.dispatch_module(ss, |m, sim, _| {
            m.as_spawn_behavior().map(|s| s.are_all_slaves_stealthed(sim))
        })
        .ok()
        .flatten()
    }

    fn stealth_allowed_to_stealth(&mut self, owner: ObjectSlot) -> Bool {
        let Some(ss) = self.world.objects.get(owner).and_then(|o| o.stealth_slot) else {
            return FALSE;
        };
        if self.world.modules.is_executing(ss) {
            return FALSE;
        }
        self.dispatch_module(ss, |m, sim, ctx| {
            m.as_stealth_mut()
                .map_or(FALSE, |st| st.allowed_to_stealth(sim, ctx, owner))
        })
        .unwrap_or(FALSE)
    }

    fn spawn_give_slaves_stealth_upgrade(&mut self, owner: ObjectSlot, grant_stealth: Bool) {
        let Some(ss) = self.world.objects.get(owner).and_then(|o| o.spawn_slot) else {
            return;
        };
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, _| {
                if let Some(s) = m.as_spawn_behavior_mut() {
                    s.give_slaves_stealth_upgrade(sim, grant_stealth);
                }
            });
        }
    }

    fn new_object_from_template(&mut self, name: &str, team: TeamId) -> Option<ObjectSlot> {
        let tmpl = self.rules.thing_factory.find_template_id(name)?;
        Some(self.factory_new_object_of(tmpl, team))
    }

    fn on_unit_created(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        self.unit_created(factory, unit);
    }

    fn exit_object_by_budding(
        &mut self,
        owner: ObjectSlot,
        new_obj: ObjectSlot,
        bud_host: Option<ObjectSlot>,
    ) {
        let Some(es) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.exit_slot.or(o.contain_slot))
        else {
            return;
        };
        if !self.world.modules.is_executing(es) {
            let _ = self.dispatch_module(es, |m, sim, ctx| {
                if let Some(e) = m.as_exit_mut() {
                    e.exit_object_by_budding(sim, ctx, new_obj, bud_host);
                }
            });
        }
    }

    fn slaved_on_enslave(&mut self, slave: ObjectSlot, slaver: ObjectSlot) {
        let Some(ss) = self.world.objects.get(slave).and_then(|o| o.slaved_slot) else {
            return;
        };
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, ctx| {
                if let Some(su) = m.as_slaved_update_mut() {
                    su.on_enslave(sim, ctx, Some(slaver));
                }
            });
        }
    }

    fn slaved_on_slaver_damage(&mut self, slave: ObjectSlot, dmg: &DamageInfo) {
        let Some(ss) = self.world.objects.get(slave).and_then(|o| o.slaved_slot) else {
            return;
        };
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, ctx| {
                if let Some(su) = m.as_slaved_update_mut() {
                    su.on_slaver_damage(sim, ctx, dmg);
                }
            });
        }
    }

    fn set_initial_health(&mut self, owner: ObjectSlot, initial_percent: Int) {
        let Some(bs) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return;
        };
        if !self.world.modules.is_executing(bs) {
            let _ = self.dispatch_module(bs, |m, sim, ctx| {
                if let Some(b) = m.as_body_mut() {
                    b.set_initial_health(sim, ctx, initial_percent);
                }
            });
        }
    }

    fn ai_force_attack_object(
        &mut self,
        owner: ObjectSlot,
        victim: ObjectSlot,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    ) {
        self.order_ai(owner, "aiForceAttackObject", |ai, sim, ctx| {
            ai.ai_force_attack_object(sim, ctx, victim, max_shots_to_fire, cmd_source);
        });
    }

    fn ai_attack_position(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    ) {
        let pos = *pos;
        self.order_ai(owner, "aiAttackPosition", |ai, sim, ctx| {
            ai.ai_attack_position(sim, ctx, &pos, max_shots_to_fire, cmd_source);
        });
    }

    fn ai_go_prone(&mut self, owner: ObjectSlot, dmg: &DamageInfo, _cmd_source: CommandSourceType) {
        if self.world.objects.get(owner).is_none_or(|o| o.ai_slot.is_none()) {
            return;
        }
        self.order_ai(owner, "aiGoProne", |_, _, _| {});
        // `AIUpdateInterface::isAllowedToRespondToAiCommands` drops commands to the dead
        // (its sleeping mood test is left to the AI port); `privateGoProne` then only
        // forwards to the `ProneUpdate`.
        if self
            .world
            .objects
            .get(owner)
            .is_some_and(|o| o.is_effectively_dead() != FALSE)
        {
            return;
        }
        let Some(ms) = self.module_by_name(owner, "ProneUpdate") else {
            return;
        };
        if !self.world.modules.is_executing(ms) {
            let _ = self.dispatch_module(ms, |m, sim, ctx| {
                if let Some(prone) = m
                    .as_any_mut()
                    .and_then(|a| a.downcast_mut::<ProneUpdate>())
                {
                    prone.go_prone(sim, ctx, dmg);
                }
            });
        }
    }

    fn set_locomotor_use_precise_z_pos(&mut self, owner: ObjectSlot, precise: Bool) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        if let Some(loco) = obj.ai_locomotor.as_mut() {
            loco.set_use_precise_z_pos(precise);
        }
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_use_precise_z_pos(precise);
                }
            });
        }
    }

    fn spawn_reveal_slaves(&mut self, owner: ObjectSlot) {
        let Some(ss) = self.world.objects.get(owner).and_then(|o| o.spawn_slot) else {
            return;
        };
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, _| {
                if let Some(s) = m.as_spawn_behavior_mut() {
                    s.reveal_slaves(sim);
                }
            });
        }
    }

    fn stealth_disguise_as_object(&mut self, owner: ObjectSlot, target: Option<ObjectSlot>) {
        let Some(ss) = self.world.objects.get(owner).and_then(|o| o.stealth_slot) else {
            return;
        };
        if !self.world.modules.is_executing(ss) {
            let _ = self.dispatch_module(ss, |m, sim, ctx| {
                if let Some(st) = m.as_stealth_mut() {
                    st.disguise_as_object(sim, ctx, target);
                }
            });
        }
    }

    fn set_stealth_detector_enabled(&mut self, owner: ObjectSlot, enabled: Bool) {
        for ms in self.behavior_slots(owner) {
            let found = self.dispatch_module(ms, |m, sim, ctx| {
                m.as_stealth_detector_mut()
                    .map(|f| f.set_sd_enabled(sim, ctx, enabled))
                    .is_some()
            });
            if found == Ok(true) {
                break;
            }
        }
    }

    fn order_ai_exit(
        &mut self,
        rider: ObjectSlot,
        container: ObjectSlot,
        cmd_source: CommandSourceType,
    ) {
        let rider_id = self.world.objects.id_of(rider).unwrap_or(ObjectId::INVALID);
        let container_id = self
            .world
            .objects
            .id_of(container)
            .unwrap_or(ObjectId::INVALID);
        self.world
            .ai_exit_orders
            .push((rider_id, container_id, cmd_source));
        if let Some(ai_ms) = self.world.objects.get(rider).and_then(|o| o.ai_slot) {
            if !self.world.modules.is_executing(ai_ms) {
                let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                    if let Some(ai) = m.as_ai_mut() {
                        ai.ai_exit(sim, ctx, container, cmd_source);
                    }
                });
            }
        }
    }

    fn contain_process_damage_to_contained(&mut self, container: ObjectSlot, percent_damage: Real) {
        let Some(cs) = self.world.objects.get(container).and_then(|o| o.contain_slot) else {
            return;
        };
        if self.world.modules.is_executing(cs) {
            let _ = self
                .world
                .modules
                .push_deferred_op(cs, DeferredSelfOp::ContainProcessDamage(percent_damage));
            return;
        }
        let _ = self.dispatch_module(cs, |m, sim, ctx| {
            if let Some(c) = m.as_contain_mut() {
                c.process_damage_to_contained(sim, ctx, percent_damage);
            }
        });
    }

    fn exit_object_via_door(&mut self, container: ObjectSlot, obj: ObjectSlot, door: ExitDoorType) {
        let Some(es) = self
            .world
            .objects
            .get(container)
            .and_then(|o| o.exit_slot.or(o.contain_slot))
        else {
            return;
        };
        if !self.world.modules.is_executing(es) {
            let _ = self.dispatch_module(es, |m, sim, ctx| {
                if let Some(e) = m.as_exit_mut() {
                    e.exit_object_via_door(sim, ctx, obj, door);
                }
            });
        }
    }

    /// `Object::setTeam`: a dead player's team is redirected to the neutral default team.
    fn set_object_team(&mut self, owner: ObjectSlot, new_team: TeamId) {
        let team = if self.player_is_active(new_team) == FALSE {
            self.player_default_team(self.neutral_team_id())
        } else {
            new_team
        };
        self.capture_object(owner, team);
    }

    fn set_physics_allow_to_fall(&mut self, owner: ObjectSlot, allow: Bool) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.set_flag(physics_update::ALLOW_TO_FALL, allow);
        }
    }

    fn physics_clear_acceleration(&mut self, owner: ObjectSlot) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.accel = Coord3D::default();
        }
    }

    fn physics_scrub_velocity_2d(&mut self, owner: ObjectSlot, desired_velocity: Real) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.scrub_velocity_2d(desired_velocity);
        }
    }

    fn physics_apply_force(&mut self, owner: ObjectSlot, force: &Coord3D) {
        physics_update::apply_force(self, owner, force);
    }

    fn physics_apply_shock(&mut self, owner: ObjectSlot, force: &Coord3D) {
        physics_update::apply_shock(self, owner, force);
    }

    fn physics_apply_motive_force(&mut self, owner: ObjectSlot, force: &Coord3D) {
        physics_update::apply_motive_force(self, owner, force);
    }

    fn physics_apply_random_rotation(&mut self, owner: ObjectSlot) {
        physics_update::apply_random_rotation(self, owner);
    }

    fn physics_set_roll_rate(&mut self, owner: ObjectSlot, rate: Real) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.set_roll_rate(rate);
        }
    }

    fn set_locomotor_max_braking(&mut self, owner: ObjectSlot, max_braking: Real) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.loco_max_braking = max_braking;
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_max_braking(max_braking);
                }
            });
        }
    }

    fn physics_set_yaw_rate(&mut self, owner: ObjectSlot, rate: Real) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.set_yaw_rate(rate);
        }
    }

    fn pathfinder_create_wall(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.pathfind_walls.insert(id);
        }
    }

    fn pathfinder_remove_wall(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.pathfind_walls.remove(&id);
        }
    }

    fn fire_projectile_detonation(
        &mut self,
        owner: ObjectSlot,
        weapon_name: &str,
        pos: &Coord3D,
        bonus_flags: UnsignedInt,
    ) {
        let owner_id = self.world.objects.id_of(owner).unwrap_or(ObjectId::INVALID);
        self.world.projectile_detonations.push((
            owner_id,
            weapon_name.to_owned(),
            *pos,
            bonus_flags,
        ));
    }

    fn record_cleared_garrisoned_building(&mut self, owner: ObjectSlot) {
        let Some(team) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        self.academy(team).record_cleared_garrisoned_building();
    }

    fn physics_set_pitch_rate(&mut self, owner: ObjectSlot, rate: Real) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.set_pitch_rate(rate);
        }
    }

    fn physics_set_is_in_free_fall(&mut self, owner: ObjectSlot, in_free_fall: Bool) {
        if let Some(p) = physics_update::physics_mut(self, owner) {
            p.set_flag(physics_update::IS_IN_FREEFALL, in_free_fall);
        }
    }

    fn ai_idle(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world.ai_commands.push((obj_id, "aiIdle".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.ai_idle(sim, ctx, cmd_source);
                }
            });
        }
        // `AIUpdateInterface::privateIdle`: re-enter `AI_IDLE` (one PRNG draw), then idle
        // everything this object contains. `isAllowedToRespondToAiCommands` drops commands to
        // the dead first.
        let ready = self.world.objects.get(owner).is_some_and(|o| {
            o.ai_state_machine_ready != FALSE
                && o.is_effectively_dead() == FALSE
                && o.thing
                    .is_kind_of(&self.rules.thing_factory, KindOfType::Projectile)
                    == FALSE
        });
        if ready {
            let _ = self.rng_logic_int(0, AI_IDLE_COUNTDOWN_DELAY);
            for id in self.contain_items(owner) {
                if let Some(rider) = self.find_object(id) {
                    self.ai_idle(rider, cmd_source);
                }
            }
        }
    }

    fn ai_hunt(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world.ai_commands.push((obj_id, "aiHunt".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.ai_hunt(sim, ctx, cmd_source);
                }
            });
        }
    }

    fn ai_move_to_position(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    ) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world
            .ai_commands
            .push((obj_id, "aiMoveToPosition".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.ai_move_to_position(sim, ctx, pos, cmd_source);
                }
            });
        }
    }

    fn ai_follow_path(
        &mut self,
        owner: ObjectSlot,
        path: &[Coord3D],
        ignore_obj: Option<ObjectSlot>,
        cmd_source: CommandSourceType,
    ) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world
            .ai_commands
            .push((obj_id, "aiFollowPath".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.ai_follow_path(sim, ctx, path, ignore_obj, cmd_source);
                }
            });
        }
    }

    fn ai_hack_internet(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world
            .ai_commands
            .push((obj_id, "aiHackInternet".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.ai_hack_internet(sim, ctx, cmd_source);
                }
            });
        }
    }

    fn ai_set_attitude(&mut self, owner: ObjectSlot, attitude: AttitudeType) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.ai_attitude = attitude;
        let obj_id = obj.id;
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        self.world
            .ai_commands
            .push((obj_id, "setAttitude".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_attitude(attitude);
                }
            });
        }
    }

    fn ai_wake_up_and_attempt_to_target(&mut self, owner: ObjectSlot) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world
            .ai_commands
            .push((obj_id, "wakeUpAndAttemptToTarget".to_owned()));
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.wake_up_and_attempt_to_target(sim, ctx);
                }
            });
        }
    }

    fn set_locomotor_max_lift(&mut self, owner: ObjectSlot, max_lift: Real) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.loco_max_lift = max_lift;
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_max_lift(max_lift);
                }
            });
        }
    }

    fn set_locomotor_max_turn_rate(&mut self, owner: ObjectSlot, max_turn_rate: Real) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.loco_max_turn_rate = max_turn_rate;
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_max_turn_rate(max_turn_rate);
                }
            });
        }
    }

    fn set_locomotor_ultra_accurate(&mut self, owner: ObjectSlot, ultra: Bool) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.loco_ultra_accurate = ultra;
        if let Some(loco) = obj.ai_locomotor.as_mut() {
            loco.set_ultra_accurate(ultra);
        }
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_ultra_accurate(ultra);
                }
            });
        }
    }

    fn set_locomotor_close_enough_dist(&mut self, owner: ObjectSlot, dist: Real) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.loco_close_enough_dist = dist;
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_close_enough_dist(dist);
                }
            });
        }
    }

    fn set_locomotor_close_enough_dist_3d(&mut self, owner: ObjectSlot, is_3d: Bool) {
        let Some(obj) = self.world.objects.get_mut(owner) else {
            return;
        };
        obj.loco_close_enough_dist_3d = is_3d;
        let Some(ai_ms) = obj.ai_slot else {
            return;
        };
        if !self.world.modules.is_executing(ai_ms) {
            let _ = self.dispatch_module(ai_ms, |m, _, _| {
                if let Some(ai) = m.as_ai_mut() {
                    ai.set_locomotor_close_enough_dist_3d(is_3d);
                }
            });
        }
    }

    fn find_position_around(
        &mut self,
        center: &Coord3D,
        options: &FindPositionOptions,
    ) -> Option<Coord3D> {
        let extent_hi = self.world.map_extent_hi;
        if center.x < 0.0 || center.x > extent_hi.x || center.y < 0.0 || center.y > extent_hi.y {
            return Some(*center);
        }
        let two_pi = zh_common::math::TWO_PI;
        let start_angle = match options.start_angle {
            Some(a) => a,
            None => self.rng_logic_real(0.0, two_pi),
        };
        let ring_spacing = 5.0_f32;
        let mut dist = options.min_radius;
        while dist <= options.max_radius {
            let angle_spacing = if dist == options.min_radius {
                two_pi
            } else {
                (ring_spacing / (dist + 1.0_f32)) * (two_pi / 6.0_f32)
            };
            let samples = real_to_int_ceil((two_pi / angle_spacing) / 2.0_f32);
            for i in 0..samples {
                let angle_pos = start_angle + angle_spacing * (i as Real);
                if let Some(found) = self.try_find_position(center, dist, angle_pos, options) {
                    return Some(found);
                }
                if i != 0 {
                    let angle_neg = start_angle - angle_spacing * (i as Real);
                    if let Some(found) = self.try_find_position(center, dist, angle_neg, options) {
                        return Some(found);
                    }
                }
            }
            dist += ring_spacing;
        }
        None
    }

    fn pathfinder_add_object(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.pathfind_map_objects.insert(id);
        }
    }

    fn pathfinder_adjust_destination(&mut self, owner: ObjectSlot, dest: &mut Coord3D) -> Bool {
        if self.object_is_kind_of(owner, KindOfType::Projectile) != FALSE {
            return TRUE;
        }
        let layer = self.get_layer_for_destination(dest);
        let (cell_x, cell_y, center) = self.pathfind_goal_cell(owner, dest);
        // Only the goal cell is tried: no spiral search, unit footprints or zones.
        if self.pathfind_cell_type(layer, cell_x, cell_y) != PathfindCellType::Clear {
            return FALSE;
        }
        self.adjust_coord_to_cell(cell_x, cell_y, center, dest, layer);
        TRUE
    }

    fn ai_follow_exit_production_path(
        &mut self,
        owner: ObjectSlot,
        path: &[Coord3D],
        ignore_obj: Option<ObjectSlot>,
        cmd_source: CommandSourceType,
    ) {
        let Some(obj_id) = self.world.objects.id_of(owner) else {
            return;
        };
        let ignore = ignore_obj
            .and_then(|slot| self.world.objects.id_of(slot))
            .unwrap_or(ObjectId::INVALID);
        self.world
            .ai_commands
            .push((obj_id, "aiFollowExitProductionPath".to_owned()));
        self.world.exit_path_orders.push(ExitPathOrder {
            object: obj_id,
            path: path.to_vec(),
            ignore,
            cmd_source,
        });
    }

    fn pathfinder_adjust_to_possible_destination(
        &mut self,
        owner: ObjectSlot,
        dest: &mut Coord3D,
    ) -> Bool {
        let layer = self.get_layer_for_destination(dest);
        if self.valid_movement_terrain(layer, owner, dest) != FALSE {
            return TRUE;
        }
        let opts = FindPositionOptions {
            flags: FPF_IGNORE_ALL_OBJECTS,
            min_radius: 0.0,
            max_radius: 40.0,
            start_angle: Some(0.0),
            ..FindPositionOptions::default()
        };
        if let Some(found) = self.find_position_around(dest, &opts) {
            *dest = found;
            TRUE
        } else {
            FALSE
        }
    }

    fn pathfinder_update_goal(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        layer: PathfindLayerEnum,
    ) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.pathfind_goals.insert(id, (*pos, layer));
        }
    }

    fn ai_set_turret_enabled(&mut self, owner: ObjectSlot, turret: WhichTurretType, enabled: Bool) {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return;
        };
        let _ = self.dispatch_module(ms, |m, _, _| {
            if let Some(ai) = m.as_ai_mut() {
                ai.set_turret_enabled(turret, enabled);
            }
        });
    }

    fn ai_recenter_turret(&mut self, owner: ObjectSlot, turret: WhichTurretType) {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.ai_slot) else {
            return;
        };
        let _ = self.dispatch_module(ms, |m, _, _| {
            if let Some(ai) = m.as_ai_mut() {
                ai.recenter_turret(turret);
            }
        });
    }

    fn apply_damage_scalar(&mut self, owner: ObjectSlot, scalar: Real) {
        let Some(ms) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return;
        };
        if self.world.modules.is_executing(ms) {
            let _ = self
                .world
                .modules
                .push_deferred_op(ms, DeferredSelfOp::ApplyDamageScalar(scalar));
            return;
        }
        let _ = self.dispatch_module(ms, |m, _, _| {
            if let Some(body) = m.as_body_mut() {
                body.apply_damage_scalar(scalar);
            }
        });
    }

    fn player_battle_plans_mut(&mut self, player: Int) -> &mut PlayerBattlePlans {
        self.world.player_battle_plans.entry(player).or_default()
    }

    fn record_battle_plan_selected(&mut self, owner: ObjectSlot) {
        let Some(team) = self.world.objects.get(owner).map(|o| o.team_id) else {
            return;
        };
        self.academy(team).record_battle_plan_selected();
    }

    fn with_special_power_module(
        &mut self,
        module: ModuleSlot,
        f: &mut dyn FnMut(&mut dyn SpecialPowerFacet, &mut dyn SimApi, ModuleContext),
    ) {
        let _ = self.dispatch_module(module, |m, sim, ctx| {
            if let Some(spm) = m.as_special_power_mut() {
                f(spm, sim, ctx);
            }
        });
    }

    fn initiate_special_power_update(
        &mut self,
        update: ModuleSlot,
        caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        target_obj: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> Option<Bool> {
        self.dispatch_module(update, |m, sim, ctx| {
            m.as_special_power_update_mut().map(|spu| {
                if spu.does_pass_science_test(sim, ctx) == FALSE {
                    return FALSE;
                }
                spu.initiate_intent_to_do_special_power(
                    sim,
                    ctx,
                    caller,
                    template_id,
                    target_obj,
                    target_pos,
                    command_options,
                )
            })
        })
        .ok()
        .flatten()
    }

    fn select_object(&mut self, owner: ObjectSlot, create_new: Bool, player_mask: UnsignedShort) {
        let Some((id, team)) = self.world.objects.get(owner).map(|o| (o.id, o.team_id)) else {
            return;
        };
        let local = Bool::from(self.player_index(team) == self.world.local_player_index);
        self.world
            .selected_objects
            .push((id, create_new, player_mask, local));
    }

    fn ai_get_healed(
        &mut self,
        owner: ObjectSlot,
        heal_unit: ObjectSlot,
        cmd_source: CommandSourceType,
    ) {
        let Some((obj_id, ai_ms)) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.ai_slot.map(|ms| (o.id, ms)))
        else {
            return;
        };
        self.world
            .ai_commands
            .push((obj_id, "aiGetHealed".to_owned()));
        let _ = self.dispatch_module(ai_ms, |m, sim, ctx| {
            if let Some(ai) = m.as_ai_mut() {
                ai.ai_get_healed(sim, ctx, heal_unit, cmd_source);
            }
        });
    }

    fn request_assist_from(
        &mut self,
        assistant: ObjectSlot,
        requesting: ObjectSlot,
        victim: ObjectSlot,
    ) -> Bool {
        let key = self.rules.name_keys.lookup_key("AssistedTargetingUpdate");
        let module = self.world.objects.get(assistant).and_then(|o| {
            o.behaviors.iter().map(|&(ms, _)| ms).find(|&ms| {
                self.world
                    .modules
                    .payload(ms)
                    .is_some_and(|p| Some(p.module_name_key()) == key)
            })
        });
        let Some(module) = module else {
            return FALSE;
        };
        self.dispatch_module(module, |m, sim, ctx| {
            let Some(assist) = m.as_assisted_targeting_mut() else {
                return FALSE;
            };
            if assist.is_free_to_assist(sim, ctx) == FALSE {
                return FALSE;
            }
            assist.assist_attack(sim, ctx, requesting, victim);
            TRUE
        })
        .unwrap_or(FALSE)
    }

    fn ai_construct(&mut self, owner: ObjectSlot) {
        self.ai_choose_locomotor_set(owner, LocomotorSetType::Normal);
    }

    fn ai_init_state_machine(&mut self, owner: ObjectSlot) {
        if self.world.objects.get(owner).is_none() {
            return;
        }
        // `AIStateMachine` owns three attack-move / follow-waypoint states that each build an
        // idle sub-machine, then enters its own default state: four `AIIdleState::onEnter`.
        for _ in 0..4 {
            let _ = self.rng_logic_int(0, AI_IDLE_COUNTDOWN_DELAY);
        }
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.ai_state_machine_ready = TRUE;
        }
    }

    fn ai_choose_locomotor_set(&mut self, owner: ObjectSlot, set_type: LocomotorSetType) {
        let Some(obj) = self.world.objects.get(owner) else {
            return;
        };
        if obj.ai_locomotor_set == Some(set_type) {
            return;
        }
        let names: Vec<String> = obj
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .map_or_else(Vec::new, |t| t.get_locomotor_set(set_type).to_vec());
        if names.is_empty() {
            return;
        }
        let frame = self.world.frame;
        let mut current: Option<Box<Locomotor>> = None;
        for name in &names {
            let Some(tmpl) = self.rules.locomotor_store.find_by_name(name) else {
                continue;
            };
            let loco = Locomotor::new(tmpl, &mut self.world.rng, frame);
            // `chooseBestLocomotorForPosition` is unported: the first entry stays current.
            if current.is_none() {
                current = Some(Box::new(loco));
            }
        }
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.ai_has_locomotor = Bool::from(current.is_some());
            obj.ai_locomotor = current;
            obj.ai_locomotor_set = Some(set_type);
            obj.cur_locomotor_set = set_type;
        }
    }

    fn client(&mut self) -> &mut dyn ClientSink {
        self.client.as_mut()
    }

    fn resident_state_mut(&mut self, module: ModuleSlot) -> Option<&mut dyn std::any::Any> {
        self.world.modules.payload_mut(module)?.as_any_mut()
    }

    fn paths(&mut self) -> &mut dyn PathServices {
        self.paths.as_mut()
    }

    fn rng_client_int(&mut self, lo: Int, hi: Int) -> Int {
        self.world.rng.get_game_client_random_value(lo, hi)
    }

    fn set_object_construction_percent(&mut self, owner: ObjectSlot, percent: Real) {
        if let Some(obj) = self.world.objects.get_mut(owner) {
            obj.construction_percent = percent;
        }
    }

    fn topple_object(
        &mut self,
        owner: ObjectSlot,
        direction: &Coord3D,
        speed: Real,
        options: UnsignedInt,
    ) {
        // `findModule("ToppleUpdate")`. A running module is not entered again.
        let Some(ms) = self.module_by_name(owner, "ToppleUpdate") else {
            return;
        };
        let _ = self.dispatch_module(ms, |m, sim, ctx| {
            if let Some(t) = m.as_topple_mut() {
                if t.is_able_to_be_toppled() != FALSE {
                    t.apply_toppling_force(sim, ctx, direction, speed, options);
                }
            }
        });
    }

    fn pathfinder_remove_object(&mut self, owner: ObjectSlot) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.pathfind_map_objects.remove(&id);
        }
    }

    fn stop_all_bone_fx(&mut self, owner: ObjectSlot) {
        let key = self.rules.name_keys.name_to_key("BoneFXUpdate");
        let has_module = self.has_module_by_name_key(owner, key);
        if let (true, Some(id)) = (has_module, self.world.objects.id_of(owner)) {
            self.world.bone_fx_stops.push(id);
        }
    }

    fn add_floating_money_text(&mut self, owner: ObjectSlot, pos: &Coord3D, amount: Int) {
        if let Some(id) = self.world.objects.id_of(owner) {
            self.world.floating_money_texts.push((id, *pos, amount));
        }
    }

    fn auto_deposit_boost_upgrade(&mut self, name: &str) -> Option<UpgradeMaskType> {
        if self.world.auto_deposit_boost_upgrade.is_none() {
            let mask = self
                .rules
                .upgrade_center
                .find_upgrade(name, &mut self.rules.name_keys)
                .map(|t| t.upgrade_mask().clone());
            self.world.auto_deposit_boost_upgrade = Some(mask);
        }
        self.world.auto_deposit_boost_upgrade.clone().flatten()
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
    fn object_jet_ai(&self, owner: ObjectSlot) -> Option<JetAiState> {
        self.ctx.object_jet_ai(owner)
    }

    fn ai_turret_for_cur_weapon(&self, owner: ObjectSlot) -> WhichTurretType {
        self.ctx.ai_turret_for_cur_weapon(owner)
    }

    fn ai_turret_in_natural_position(&self, owner: ObjectSlot, turret: WhichTurretType) -> Bool {
        self.ctx.ai_turret_in_natural_position(owner, turret)
    }

    fn player_battle_plans(&self, player: Int) -> Option<&PlayerBattlePlans> {
        self.ctx.player_battle_plans(player)
    }

    fn draw_icon_ui(&self) -> Bool {
        self.ctx.draw_icon_ui()
    }

    fn horde_member_state(&self, owner: ObjectSlot) -> Option<Bool> {
        self.ctx.horde_member_state(owner)
    }

    fn contain_apparent_controlling_player(&self, container: ObjectSlot, observer: Int) -> Option<Int> {
        self.ctx.contain_apparent_controlling_player(container, observer)
    }

    fn thing_template_exists(&self, name: &str) -> Bool {
        self.ctx.thing_template_exists(name)
    }

    fn spawn_closest_slave(&self, owner: ObjectSlot, pos: &Coord3D) -> Option<Option<ObjectSlot>> {
        self.ctx.spawn_closest_slave(owner, pos)
    }

    fn contain_closest_rider(
        &self,
        owner: ObjectSlot,
        pos: &Coord3D,
    ) -> Option<Option<ObjectSlot>> {
        self.ctx.contain_closest_rider(owner, pos)
    }

    fn slaved_is_self_tasking(&self, owner: ObjectSlot) -> Bool {
        self.ctx.slaved_is_self_tasking(owner)
    }

    fn ai_is_idle(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_is_idle(owner)
    }

    fn ai_cur_locomotor_preferred_height(&self, owner: ObjectSlot) -> Option<Real> {
        self.ctx.ai_cur_locomotor_preferred_height(owner)
    }

    fn terrain_extent(&self) -> Region3D {
        self.ctx.terrain_extent()
    }

    fn ai_path_destination(&self, owner: ObjectSlot) -> Option<Coord3D> {
        self.ctx.ai_path_destination(owner)
    }

    fn object_is_able_to_attack(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_is_able_to_attack(owner)
    }

    fn object_has_client_update(&self, owner: ObjectSlot, name: &str) -> Bool {
        self.ctx.object_has_client_update(owner, name)
    }

    fn ai_is_moving(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_is_moving(owner)
    }

    fn ai_attack_priority(&self, owner: ObjectSlot, victim: ObjectSlot) -> Option<Int> {
        self.ctx.ai_attack_priority(owner, victim)
    }

    fn find_special_ability_update(&self, owner: ObjectSlot, power_type: SpecialPowerType) -> Option<ModuleSlot> {
        self.ctx.find_special_ability_update(owner, power_type)
    }

    fn special_ability_is_active(&self, module: ModuleSlot) -> Bool {
        self.ctx.special_ability_is_active(module)
    }

    fn special_power_module_ready(&self, owner: ObjectSlot, template_id: UnsignedInt) -> Option<(Bool, UnsignedInt)> {
        self.ctx.special_power_module_ready(owner, template_id)
    }

    fn action_manager_allows(&self, actor: ObjectSlot, target: ObjectSlot, query: ActionQuery) -> Bool {
        self.ctx.action_manager_allows(actor, target, query)
    }

    fn is_clear_line_of_sight(&self, owner: ObjectSlot, other: ObjectSlot) -> Bool {
        self.ctx.is_clear_line_of_sight(owner, other)
    }

    fn is_in_range_bounding_2d(&self, owner: ObjectSlot, target: ObjectSlot, radius: Real) -> Bool {
        self.ctx.is_in_range_bounding_2d(owner, target, radius)
    }

    fn find_objects_near_to_far(&self, center: &Coord3D, radius: Real) -> Vec<ObjectSlot> {
        self.ctx.find_objects_near_to_far(center, radius)
    }

    fn find_closest_team_object_of_kind(&self, center: &Coord3D, radius: Real, kind: KindOfType, team_id: TeamId) -> Option<ObjectSlot> {
        self.ctx.find_closest_team_object_of_kind(center, radius, kind, team_id)
    }

    fn object_has_template_module(&self, owner: ObjectSlot, name: &str) -> Bool {
        self.ctx.object_has_template_module(owner, name)
    }

    fn command_set_button(&self, set_name: &str, index: usize) -> Option<CommandButtonId> {
        self.ctx.command_set_button(set_name, index)
    }

    fn object_command_set_string(&self, owner: ObjectSlot) -> String {
        self.ctx.object_command_set_string(owner)
    }

    fn able_to_attack_specific_object(&self, owner: ObjectSlot, attack_type: UnsignedInt, target: ObjectSlot, cmd_source: CommandSourceType) -> CanAttackResult {
        self.ctx.able_to_attack_specific_object(owner, attack_type, target, cmd_source)
    }

    fn able_to_use_weapon_against_target(&self, owner: ObjectSlot, attack_type: UnsignedInt, victim: Option<ObjectSlot>, pos: &Coord3D, cmd_source: CommandSourceType) -> CanAttackResult {
        self.ctx.able_to_use_weapon_against_target(owner, attack_type, victim, pos, cmd_source)
    }

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

    fn weapon_bonus_condition_flags(&self, owner: ObjectSlot) -> UnsignedInt {
        self.ctx.weapon_bonus_condition_flags(owner)
    }

    fn is_reload_time_shared(&self, owner: ObjectSlot) -> Bool {
        self.ctx.is_reload_time_shared(owner)
    }

    fn weapon_barrel_count(&self, owner: ObjectSlot, wslot: WeaponSlotType) -> Int {
        self.ctx.weapon_barrel_count(owner, wslot)
    }

    fn last_shot_fired_frame(&self, owner: ObjectSlot) -> UnsignedInt {
        self.ctx.last_shot_fired_frame(owner)
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

    fn object_would_ignite(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_would_ignite(owner)
    }

    fn closest_would_ignite_object(&self, owner: ObjectSlot, range: Real) -> Option<ObjectSlot> {
        self.ctx.closest_would_ignite_object(owner, range)
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

    fn object_health(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_health(owner)
    }

    fn object_body_damage_state(&self, owner: ObjectSlot) -> BodyDamageType {
        self.ctx.object_body_damage_state(owner)
    }

    fn stealth_is_temporary_grant(&self, owner: ObjectSlot) -> Bool {
        self.ctx.stealth_is_temporary_grant(owner)
    }

    fn dock_is_open(&self, owner: ObjectSlot) -> Bool {
        self.ctx.dock_is_open(owner)
    }

    fn player_is_playable_side(&self, team_id: TeamId) -> Bool {
        self.ctx.player_is_playable_side(team_id)
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

    fn find_potential_collisions(
        &self,
        pos: &Coord3D,
        geom: &GeometryInfo,
        angle: Real,
        use_2d: Bool,
    ) -> Vec<ObjectSlot> {
        self.ctx.find_potential_collisions(pos, geom, angle, use_2d)
    }

    fn ai_is_clearing_mines(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_is_clearing_mines(owner)
    }

    fn find_objects_in_range_of(&self, owner: ObjectSlot, radius: Real) -> Vec<ObjectSlot> {
        self.ctx.find_objects_in_range_of(owner, radius)
    }

    fn find_objects_in_range_bounding_sphere_3d(
        &self,
        owner: ObjectSlot,
        radius: Real,
    ) -> Vec<ObjectSlot> {
        self.ctx
            .find_objects_in_range_bounding_sphere_3d(owner, radius)
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

    fn player_energy_production(&self, team_id: TeamId) -> Int {
        self.ctx.player_energy_production(team_id)
    }

    fn player_radar_counts(&self, team_id: TeamId) -> (Int, Int) {
        self.ctx.player_radar_counts(team_id)
    }

    fn production_cost_change(&self, team_id: TeamId, kindof: &KindOfMaskType) -> Real {
        self.ctx.production_cost_change(team_id, kindof)
    }

    fn world_object_slots(&self) -> Vec<ObjectSlot> {
        self.ctx.world_object_slots()
    }

    fn object_is_significantly_above_terrain(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_is_significantly_above_terrain(owner)
    }

    fn player_is_active(&self, team_id: TeamId) -> Bool {
        self.ctx.player_is_active(team_id)
    }

    fn player_bounty_percent(&self, team_id: TeamId) -> Real {
        self.ctx.player_bounty_percent(team_id)
    }

    fn object_is_trainable(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_is_trainable(owner)
    }

    fn object_crusher_level(&self, owner: ObjectSlot) -> Int {
        self.ctx.object_crusher_level(owner)
    }

    fn object_crushable_level(&self, owner: ObjectSlot) -> Int {
        self.ctx.object_crushable_level(owner)
    }

    fn contain_count(&self, owner: ObjectSlot) -> UnsignedInt {
        self.ctx.contain_count(owner)
    }

    fn contain_max(&self, owner: ObjectSlot) -> Int {
        self.ctx.contain_max(owner)
    }

    fn is_valid_container_for(
        &self,
        container: ObjectSlot,
        obj: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        self.ctx
            .is_valid_container_for(container, obj, check_capacity)
    }

    fn tunnel_tracker(&self, player_index: Int) -> Option<&TunnelTrackerState> {
        self.ctx.tunnel_tracker(player_index)
    }

    fn max_tunnel_capacity(&self) -> Int {
        self.ctx.max_tunnel_capacity()
    }

    fn object_last_damage(&self, owner: ObjectSlot) -> Option<(ObjectId, UnsignedInt)> {
        self.ctx.object_last_damage(owner)
    }

    fn cave_tracker(&self, cave_index: Int) -> Option<&TunnelTrackerState> {
        self.ctx.cave_tracker(cave_index)
    }

    fn cave_can_switch_index(&self, old_index: Int, new_index: Int) -> Bool {
        self.ctx.cave_can_switch_index(old_index, new_index)
    }

    fn bridge_is_passable(&self, bridge_id: ObjectId) -> Bool {
        self.ctx.bridge_is_passable(bridge_id)
    }

    fn neutral_team_id(&self) -> TeamId {
        self.ctx.neutral_team_id()
    }

    fn player_index(&self, team_id: TeamId) -> Int {
        self.ctx.player_index(team_id)
    }

    fn player_mask(&self, team_id: TeamId) -> UnsignedShort {
        self.ctx.player_mask(team_id)
    }

    fn ai_is_in_dead_state(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_is_in_dead_state(owner)
    }

    fn object_ai_enter_target(&self, owner: ObjectSlot) -> Option<ObjectSlot> {
        self.ctx.object_ai_enter_target(owner)
    }

    fn object_height_above_terrain(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_height_above_terrain(owner)
    }

    fn slow_death_scale(&self) -> Real {
        self.ctx.slow_death_scale()
    }

    fn hulk_max_lifetime_override(&self) -> Int {
        self.ctx.hulk_max_lifetime_override()
    }

    fn physics_last_collidee(&self, owner: ObjectSlot) -> ObjectId {
        self.ctx.physics_last_collidee(owner)
    }

    fn object_has_drawable(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_has_drawable(owner)
    }

    fn local_player_index(&self) -> Int {
        self.ctx.local_player_index()
    }

    fn production_can_make_unit(&self, builder: ObjectSlot, tmpl: ThingTemplateId, own: Option<&dyn ProductionFacet>) -> CanMakeType {
        self.ctx.production_can_make_unit(builder, tmpl, own)
    }

    fn unit_cost_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Int {
        self.ctx.unit_cost_to_build(team_id, tmpl)
    }

    fn unit_time_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Int {
        self.ctx.unit_time_to_build(team_id, tmpl)
    }

    fn player_allowed_to_build(&self, team_id: TeamId, tmpl: ThingTemplateId) -> Bool {
        self.ctx.player_allowed_to_build(team_id, tmpl)
    }

    fn can_produce_upgrade(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool {
        self.ctx.can_produce_upgrade(owner, upgrade)
    }

    fn affected_by_upgrade(&self, owner: ObjectSlot, upgrade: NameKeyType) -> Bool {
        self.ctx.affected_by_upgrade(owner, upgrade)
    }

    fn player_has_upgrade_in_production(&self, team_id: TeamId, mask: &UpgradeMaskType) -> Bool {
        self.ctx.player_has_upgrade_in_production(team_id, mask)
    }

    fn parking_door_requirement(&self, owner: ObjectSlot, tmpl: ThingTemplateId) -> (Bool, Bool) {
        self.ctx.parking_door_requirement(owner, tmpl)
    }

    fn object_has_exit_interface(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_has_exit_interface(owner)
    }

    fn player_default_team(&self, team_id: TeamId) -> TeamId {
        self.ctx.player_default_team(team_id)
    }

    fn player_is_skirmish_ai(&self, team_id: TeamId) -> Bool {
        self.ctx.player_is_skirmish_ai(team_id)
    }

    fn team_relationship_to_player(&self, team_id: TeamId, player_index: Int) -> Relationship {
        self.ctx.team_relationship_to_player(team_id, player_index)
    }

    fn contain_is_special_zero_slot(&self, owner: ObjectSlot) -> Bool {
        self.ctx.contain_is_special_zero_slot(owner)
    }

    fn contain_is_bustable(&self, owner: ObjectSlot) -> Bool {
        self.ctx.contain_is_bustable(owner)
    }

    fn contain_is_enclosing_container_for(&self, owner: ObjectSlot, rider: ObjectSlot) -> Bool {
        self.ctx.contain_is_enclosing_container_for(owner, rider)
    }

    fn bridge_tower_id(&self, owner: ObjectSlot, tower_type: BridgeTowerType) -> Option<ObjectId> {
        self.ctx.bridge_tower_id(owner, tower_type)
    }

    fn contain_is_special_overlord_style(&self, owner: ObjectSlot) -> Bool {
        self.ctx.contain_is_special_overlord_style(owner)
    }

    fn contain_items(&self, owner: ObjectSlot) -> Vec<ObjectId> {
        self.ctx.contain_items(owner)
    }

    fn contain_is_passenger_allowed_to_fire(&self, owner: ObjectSlot, id: ObjectId) -> Bool {
        self.ctx.contain_is_passenger_allowed_to_fire(owner, id)
    }

    fn exit_use_spawn_rally_point(&self, owner: ObjectSlot) -> Bool {
        self.ctx.exit_use_spawn_rally_point(owner)
    }

    fn has_physics(&self, owner: ObjectSlot) -> Bool {
        self.ctx.has_physics(owner)
    }

    fn physics_velocity(&self, owner: ObjectSlot) -> Coord3D {
        self.ctx.physics_velocity(owner)
    }

    fn physics_mass(&self, owner: ObjectSlot) -> Real {
        self.ctx.physics_mass(owner)
    }

    fn physics_center_of_mass_offset(&self, owner: ObjectSlot) -> Real {
        self.ctx.physics_center_of_mass_offset(owner)
    }

    fn object_crushed_ends(&self, owner: ObjectSlot) -> (Bool, Bool) {
        self.ctx.object_crushed_ends(owner)
    }

    fn contain_items_for_mass(&self, owner: ObjectSlot) -> Vec<ObjectId> {
        self.ctx.contain_items_for_mass(owner)
    }

    fn ai_ignored_obstacle_id(&self, owner: ObjectSlot) -> ObjectId {
        self.ctx.ai_ignored_obstacle_id(owner)
    }

    fn carrier_deck_height(&self, owner: ObjectSlot) -> Real {
        self.ctx.carrier_deck_height(owner)
    }

    fn has_ai(&self, owner: ObjectSlot) -> Bool {
        self.ctx.has_ai(owner)
    }

    fn ai_free_to_exit(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_free_to_exit(owner)
    }

    fn ai_current_victim(&self, owner: ObjectSlot) -> Option<ObjectSlot> {
        self.ctx.ai_current_victim(owner)
    }

    fn ai_current_victim_pos(&self, owner: ObjectSlot) -> Option<Coord3D> {
        self.ctx.ai_current_victim_pos(owner)
    }

    fn ai_has_cur_locomotor(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_has_cur_locomotor(owner)
    }

    fn valid_movement_terrain(
        &self,
        layer: PathfindLayerEnum,
        rider: ObjectSlot,
        pos: &Coord3D,
    ) -> Bool {
        self.ctx.valid_movement_terrain(layer, rider, pos)
    }

    fn pathfind_cell_type(
        &self,
        layer: PathfindLayerEnum,
        cell_x: Int,
        cell_y: Int,
    ) -> PathfindCellType {
        self.ctx.pathfind_cell_type(layer, cell_x, cell_y)
    }

    fn get_layer_for_destination(&self, pos: &Coord3D) -> PathfindLayerEnum {
        self.ctx.get_layer_for_destination(pos)
    }

    fn get_layer_height(&self, x: Real, y: Real, layer: PathfindLayerEnum) -> Real {
        self.ctx.get_layer_height(x, y, layer)
    }

    fn pathfinder_snap_position(&self, owner: ObjectSlot, pos: &mut Coord3D) {
        self.ctx.pathfinder_snap_position(owner, pos);
    }

    fn ai_is_doing_ground_movement(&self, owner: ObjectSlot) -> Bool {
        self.ctx.ai_is_doing_ground_movement(owner)
    }

    fn pristine_bone_transforms(&self, owner: ObjectSlot, bone_prefix: &str) -> Vec<Matrix3D> {
        self.ctx.pristine_bone_transforms(owner, bone_prefix)
    }

    fn get_highest_layer_for_destination(
        &self,
        pos: &Coord3D,
        only_healthy_bridges: Bool,
    ) -> PathfindLayerEnum {
        self.ctx
            .get_highest_layer_for_destination(pos, only_healthy_bridges)
    }

    fn object_vision_range(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_vision_range(owner)
    }

    fn object_shroud_clearing_range(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_shroud_clearing_range(owner)
    }

    fn object_shroud_range(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_shroud_range(owner)
    }

    fn stealth_params(&self, owner: ObjectSlot) -> Option<StealthParams> {
        self.ctx.stealth_params(owner)
    }

    fn object_last_damage_timestamp(&self, owner: ObjectSlot) -> Option<UnsignedInt> {
        self.ctx.object_last_damage_timestamp(owner)
    }

    fn weapon_slot_last_shot_frame(&self, owner: ObjectSlot, slot: usize) -> Option<UnsignedInt> {
        self.ctx.weapon_slot_last_shot_frame(owner, slot)
    }

    fn object_has_contain(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_has_contain(owner)
    }

    fn contain_stealth_units_contained(&self, owner: ObjectSlot) -> UnsignedInt {
        self.ctx.contain_stealth_units_contained(owner)
    }

    fn contain_is_rider_change(&self, owner: ObjectSlot) -> Bool {
        self.ctx.contain_is_rider_change(owner)
    }

    fn ai_last_command_source(&self, owner: ObjectSlot) -> CommandSourceType {
        self.ctx.ai_last_command_source(owner)
    }

    fn controlling_player_objects(&self, team_id: TeamId) -> Vec<ObjectSlot> {
        self.ctx.controlling_player_objects(team_id)
    }

    fn enemy_player_objects(&self, team_id: TeamId) -> Vec<ObjectSlot> {
        self.ctx.enemy_player_objects(team_id)
    }

    fn estimate_highest_terrain_along_line(&self, start: &Coord3D, end: &Coord3D) -> Option<Real> {
        self.ctx.estimate_highest_terrain_along_line(start, end)
    }

    fn find_closest_object_to_position(&self, pos: &Coord3D, radius: Real) -> Option<ObjectSlot> {
        self.ctx.find_closest_object_to_position(pos, radius)
    }

    fn contain_is_immune_to_clear(&self, owner: ObjectSlot) -> Bool {
        self.ctx.contain_is_immune_to_clear(owner)
    }

    fn object_has_reserved_space(&self, airfield: ObjectSlot, id: ObjectId) -> Bool {
        self.ctx.object_has_reserved_space(airfield, id)
    }

    fn player_color(&self, team_id: TeamId) -> Int {
        self.ctx.player_color(team_id)
    }

    fn player_side(&self, team_id: TeamId) -> String {
        self.ctx.player_side(team_id)
    }

    fn contain_is_garrisonable(&self, owner: ObjectSlot) -> Bool {
        self.ctx.contain_is_garrisonable(owner)
    }

    fn object_construction_percent(&self, owner: ObjectSlot) -> Real {
        self.ctx.object_construction_percent(owner)
    }

    fn object_is_logically_visible(&self, owner: ObjectSlot) -> Bool {
        self.ctx.object_is_logically_visible(owner)
    }

    fn object_last_damage_type(&self, owner: ObjectSlot) -> Option<DamageType> {
        self.ctx.object_last_damage_type(owner)
    }

    fn adjust_topple_direction(&self, owner: ObjectSlot, direction: &mut Coord3D) {
        self.ctx.adjust_topple_direction(owner, direction);
    }
}

impl SimApi for MockSimApi {
    fn ai_set_turret_enabled(&mut self, owner: ObjectSlot, turret: WhichTurretType, enabled: Bool) {
        self.ctx.ai_set_turret_enabled(owner, turret, enabled);
    }

    fn ai_recenter_turret(&mut self, owner: ObjectSlot, turret: WhichTurretType) {
        self.ctx.ai_recenter_turret(owner, turret);
    }

    fn apply_damage_scalar(&mut self, owner: ObjectSlot, scalar: Real) {
        self.ctx.apply_damage_scalar(owner, scalar);
    }

    fn player_battle_plans_mut(&mut self, player: Int) -> &mut PlayerBattlePlans {
        self.ctx.player_battle_plans_mut(player)
    }

    fn record_battle_plan_selected(&mut self, owner: ObjectSlot) {
        self.ctx.record_battle_plan_selected(owner);
    }

    fn with_special_power_module(
        &mut self,
        module: ModuleSlot,
        f: &mut dyn FnMut(&mut dyn SpecialPowerFacet, &mut dyn SimApi, ModuleContext),
    ) {
        self.ctx.with_special_power_module(module, f);
    }

    fn initiate_special_power_update(
        &mut self,
        update: ModuleSlot,
        caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        target_obj: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> Option<Bool> {
        self.ctx.initiate_special_power_update(
            update,
            caller,
            template_id,
            target_obj,
            target_pos,
            command_options,
        )
    }

    fn select_object(&mut self, owner: ObjectSlot, create_new: Bool, player_mask: UnsignedShort) {
        self.ctx.select_object(owner, create_new, player_mask);
    }

    fn ai_get_healed(
        &mut self,
        owner: ObjectSlot,
        heal_unit: ObjectSlot,
        cmd_source: CommandSourceType,
    ) {
        self.ctx.ai_get_healed(owner, heal_unit, cmd_source);
    }

    fn ai_attack_object(
        &mut self,
        owner: ObjectSlot,
        victim: ObjectSlot,
        max_shots_to_fire: Int,
        cmd_source: CommandSourceType,
    ) {
        self.ctx
            .ai_attack_object(owner, victim, max_shots_to_fire, cmd_source);
    }

    fn request_assist_from(
        &mut self,
        assistant: ObjectSlot,
        requesting: ObjectSlot,
        victim: ObjectSlot,
    ) -> Bool {
        self.ctx.request_assist_from(assistant, requesting, victim)
    }

    fn ai_busy(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_busy(owner, cmd_source);
    }

    fn ai_move_to_object(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_move_to_object(owner, target, cmd_source);
    }

    fn ai_face_object(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_face_object(owner, target, cmd_source);
    }

    fn ai_face_position(&mut self, owner: ObjectSlot, pos: &Coord3D, cmd_source: CommandSourceType) {
        self.ctx.ai_face_position(owner, pos, cmd_source);
    }

    fn ai_enter(&mut self, owner: ObjectSlot, target: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_enter(owner, target, cmd_source);
    }

    fn ai_ignore_obstacle(&mut self, owner: ObjectSlot, obstacle: ObjectId) {
        self.ctx.ai_ignore_obstacle(owner, obstacle);
    }

    fn exit_special_ability(&mut self, module: ModuleSlot, cleanup: Bool) {
        self.ctx.exit_special_ability(module, cleanup);
    }

    fn add_experience_points(&mut self, owner: ObjectSlot, gain: Int) {
        self.ctx.add_experience_points(owner, gain);
    }

    fn add_skill_points(&mut self, team_id: TeamId, points: Int) {
        self.ctx.add_skill_points(team_id, points);
    }

    fn record_special_power_used(
        &mut self,
        owner: ObjectSlot,
        classification: AcademyClassificationType,
    ) {
        self.ctx.record_special_power_used(owner, classification);
    }

    fn record_building_capture(&mut self, team_id: TeamId) {
        self.ctx.record_building_capture(team_id);
    }

    fn sticky_bomb_init(&mut self, charge: ObjectSlot, target: ObjectSlot, bomber: ObjectSlot) {
        self.ctx.sticky_bomb_init(charge, target, bomber);
    }

    fn sticky_bomb_detonate(&mut self, charge: ObjectSlot) -> Bool {
        self.ctx.sticky_bomb_detonate(charge)
    }

    fn check_and_detonate_booby_trap(&mut self, owner: ObjectSlot, victim: ObjectSlot) -> Bool {
        self.ctx.check_and_detonate_booby_trap(owner, victim)
    }

    fn do_special_power_at_object(&mut self, owner: ObjectSlot, template_id: UnsignedInt, target: ObjectSlot, command_options: UnsignedInt, forced: Bool) {
        self.ctx.do_special_power_at_object(owner, template_id, target, command_options, forced);
    }

    fn special_power_start_recharge(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        self.ctx.special_power_start_recharge(owner, template_id);
    }

    fn special_power_mark_triggered(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        self.ctx.special_power_mark_triggered(owner, template_id);
    }

    fn random_offset_within_footprint(&mut self, geometry: &GeometryInfo, offset: &mut Coord3D) {
        self.ctx.random_offset_within_footprint(geometry, offset);
    }

    fn set_command_button_hunt(&mut self, owner: ObjectSlot, button_name: &str) {
        self.ctx.set_command_button_hunt(owner, button_name);
    }

    fn spawn_all_slaves_stealthed(&mut self, owner: ObjectSlot) -> Option<Bool> {
        self.ctx.spawn_all_slaves_stealthed(owner)
    }

    fn stealth_allowed_to_stealth(&mut self, owner: ObjectSlot) -> Bool {
        self.ctx.stealth_allowed_to_stealth(owner)
    }

    fn spawn_give_slaves_stealth_upgrade(&mut self, owner: ObjectSlot, grant_stealth: Bool) {
        self.ctx.spawn_give_slaves_stealth_upgrade(owner, grant_stealth);
    }

    fn new_object_from_template(&mut self, name: &str, team: TeamId) -> Option<ObjectSlot> {
        self.ctx.new_object_from_template(name, team)
    }

    fn on_unit_created(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        self.ctx.on_unit_created(factory, unit);
    }

    fn reserve_exit_door(&mut self, owner: ObjectSlot, tmpl: ThingTemplateId) -> ExitDoorType {
        self.ctx.reserve_exit_door(owner, tmpl)
    }

    fn try_reserve_exit_door(&mut self, owner: ObjectSlot) -> Option<ExitDoorType> {
        self.ctx.try_reserve_exit_door(owner)
    }

    fn exit_object_by_budding(&mut self, owner: ObjectSlot, new_obj: ObjectSlot, bud_host: Option<ObjectSlot>) {
        self.ctx.exit_object_by_budding(owner, new_obj, bud_host);
    }

    fn slaved_on_enslave(&mut self, slave: ObjectSlot, slaver: ObjectSlot) {
        self.ctx.slaved_on_enslave(slave, slaver);
    }

    fn slaved_on_slaver_damage(&mut self, slave: ObjectSlot, dmg: &DamageInfo) {
        self.ctx.slaved_on_slaver_damage(slave, dmg);
    }

    fn set_initial_health(&mut self, owner: ObjectSlot, initial_percent: Int) {
        self.ctx.set_initial_health(owner, initial_percent);
    }

    fn ai_force_attack_object(&mut self, owner: ObjectSlot, victim: ObjectSlot, max_shots_to_fire: Int, cmd_source: CommandSourceType) {
        self.ctx.ai_force_attack_object(owner, victim, max_shots_to_fire, cmd_source);
    }

    fn ai_attack_position(&mut self, owner: ObjectSlot, pos: &Coord3D, max_shots_to_fire: Int, cmd_source: CommandSourceType) {
        self.ctx.ai_attack_position(owner, pos, max_shots_to_fire, cmd_source);
    }

    fn ai_go_prone(&mut self, owner: ObjectSlot, dmg: &DamageInfo, cmd_source: CommandSourceType) {
        self.ctx.ai_go_prone(owner, dmg, cmd_source);
    }

    fn set_locomotor_use_precise_z_pos(&mut self, owner: ObjectSlot, precise: Bool) {
        self.ctx.set_locomotor_use_precise_z_pos(owner, precise);
    }

    fn share_weapon_reload(&mut self, owner: ObjectSlot, when: UnsignedInt, status: WeaponStatus) {
        self.ctx.share_weapon_reload(owner, when, status);
    }

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

    fn try_to_ignite(&mut self, owner: ObjectSlot) {
        self.ctx.try_to_ignite(owner);
    }

    fn start_fire_spreading(&mut self, owner: ObjectSlot) {
        self.ctx.start_fire_spreading(owner);
    }

    fn set_body_aflame(&mut self, owner: ObjectSlot, setting: Bool) {
        self.ctx.set_body_aflame(owner, setting);
    }

    fn lend_module(
        &mut self,
        slot: ModuleSlot,
        module: Box<dyn BehaviorModule>,
        f: &mut dyn FnMut(&mut dyn SimApi),
    ) -> Option<Box<dyn BehaviorModule>> {
        self.ctx.lend_module(slot, module, f)
    }

    fn body_internal_change_health(&mut self, owner: ObjectSlot, delta: Real) {
        self.ctx.body_internal_change_health(owner, delta);
    }

    fn set_land_mine_scoot_parms(&mut self, owner: ObjectSlot, start: &Coord3D, end: &Coord3D) {
        self.ctx.set_land_mine_scoot_parms(owner, start, end);
    }

    fn stop_auto_healing(&mut self, owner: ObjectSlot) {
        self.ctx.stop_auto_healing(owner);
    }

    fn attempt_damage_watching(
        &mut self,
        target: ObjectSlot,
        dmg: &mut DamageInfo,
        watch: ModuleSlot,
    ) -> Option<ObjectStatusMaskType> {
        self.ctx.attempt_damage_watching(target, dmg, watch)
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

    fn deposit_player_money(&mut self, team_id: TeamId, amount: UnsignedInt) {
        self.ctx.deposit_player_money(team_id, amount);
    }

    fn player_add_upgrade_in_production(&mut self, team_id: TeamId, upgrade: NameKeyType) {
        self.ctx.player_add_upgrade_in_production(team_id, upgrade);
    }

    fn player_remove_upgrade(&mut self, team_id: TeamId, upgrade: NameKeyType) {
        self.ctx.player_remove_upgrade(team_id, upgrade);
    }

    fn complete_production_upgrade(&mut self, owner: ObjectSlot, upgrade: NameKeyType) {
        self.ctx.complete_production_upgrade(owner, upgrade);
    }

    fn unreserve_exit_door(&mut self, owner: ObjectSlot, door: ExitDoorType) {
        self.ctx.unreserve_exit_door(owner, door);
    }

    fn factory_new_object(&mut self, tmpl: ThingTemplateId, team_id: TeamId) -> ObjectSlot {
        self.ctx.factory_new_object(tmpl, team_id)
    }

    fn production_unit_created(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        self.ctx.production_unit_created(factory, unit);
    }

    fn record_unit_production(&mut self, factory: ObjectSlot, unit: ObjectSlot) {
        self.ctx.record_unit_production(factory, unit);
    }

    fn production_set_hold_door_open(&mut self, owner: ObjectSlot, door: ExitDoorType, hold: Bool) {
        self.ctx.production_set_hold_door_open(owner, door, hold);
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

    fn set_vision_range(&mut self, owner: ObjectSlot, range: Real) {
        self.ctx.set_vision_range(owner, range);
    }

    fn set_shroud_clearing_range(&mut self, owner: ObjectSlot, range: Real) {
        self.ctx.set_shroud_clearing_range(owner, range);
    }

    fn set_shroud_range(&mut self, owner: ObjectSlot, range: Real) {
        self.ctx.set_shroud_range(owner, range);
    }

    fn set_vision_spied(&mut self, owner: ObjectSlot, setting: Bool, by_whom: Int) {
        self.ctx.set_vision_spied(owner, setting, by_whom);
    }

    fn handle_partition_cell_maintenance(&mut self, owner: ObjectSlot) {
        self.ctx.handle_partition_cell_maintenance(owner);
    }

    fn set_enemy_units_vision_spied(
        &mut self,
        actor_team: TeamId,
        setting: Bool,
        which_units: &KindOfMaskType,
    ) {
        self.ctx
            .set_enemy_units_vision_spied(actor_team, setting, which_units);
    }

    fn activate_spy_vision(&mut self, owner: ObjectSlot, duration: UnsignedInt) {
        self.ctx.activate_spy_vision(owner, duration);
    }

    fn disable_internet_center_spy_vision(&mut self, victim: ObjectSlot, until_frame: UnsignedInt) {
        self.ctx
            .disable_internet_center_spy_vision(victim, until_frame);
    }

    fn set_deletion_lifetime(
        &mut self,
        owner: ObjectSlot,
        min_frames: UnsignedInt,
        max_frames: UnsignedInt,
    ) {
        self.ctx
            .set_deletion_lifetime(owner, min_frames, max_frames);
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

    fn retract_power_plant(&mut self, owner: ObjectSlot) {
        self.ctx.retract_power_plant(owner);
    }

    fn with_supply_truck(
        &mut self,
        owner: ObjectSlot,
        f: &mut dyn FnMut(&mut dyn SupplyTruckAiFacet, &mut dyn SimApi, ModuleContext),
    ) -> Bool {
        self.ctx.with_supply_truck(owner, f)
    }

    fn add_money_earned(&mut self, team_id: TeamId, amount: Int) {
        self.ctx.add_money_earned(team_id, amount);
    }

    fn receive_temporary_stealth_grant(&mut self, owner: ObjectSlot, frames: UnsignedInt) {
        self.ctx.receive_temporary_stealth_grant(owner, frames);
    }

    fn set_dock_open(&mut self, owner: ObjectSlot, open: Bool) {
        self.ctx.set_dock_open(owner, open);
    }

    fn set_dock_crippled(&mut self, owner: ObjectSlot, crippled: Bool) {
        self.ctx.set_dock_crippled(owner, crippled);
    }

    fn extend_power_plant(&mut self, owner: ObjectSlot) {
        self.ctx.extend_power_plant(owner);
    }

    fn extend_radar_update(&mut self, owner: ObjectSlot) {
        self.ctx.extend_radar_update(owner);
    }

    fn add_power_bonus(&mut self, team_id: TeamId, owner: ObjectSlot) {
        self.ctx.add_power_bonus(team_id, owner);
    }

    fn remove_power_bonus(&mut self, team_id: TeamId, owner: ObjectSlot) {
        self.ctx.remove_power_bonus(team_id, owner);
    }

    fn adjust_player_power(&mut self, team_id: TeamId, delta: Int, adding: Bool) {
        self.ctx.adjust_player_power(team_id, delta, adding);
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

    fn unpause_special_power(&mut self, owner: ObjectSlot, template_id: UnsignedInt) {
        self.ctx.unpause_special_power(owner, template_id);
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

    fn capture_object(&mut self, owner: ObjectSlot, new_team: TeamId) {
        self.ctx.capture_object(owner, new_team);
    }

    fn update_upgrade_modules(&mut self, owner: ObjectSlot) {
        self.ctx.update_upgrade_modules(owner);
    }

    fn set_crushed(&mut self, owner: ObjectSlot, front: Option<Bool>, back: Option<Bool>) {
        self.ctx.set_crushed(owner, front, back);
    }

    fn set_special_power_creator(&mut self, owner: ObjectSlot, creator_id: ObjectId) {
        self.ctx.set_special_power_creator(owner, creator_id);
    }

    fn set_geometry_info(&mut self, owner: ObjectSlot, geom: &GeometryInfo) {
        self.ctx.set_geometry_info(owner, geom);
    }

    fn set_demoralized(&mut self, owner: ObjectSlot, frames: UnsignedInt) {
        self.ctx.set_demoralized(owner, frames);
    }

    fn reset_supply_dropzone_timer(&mut self, owner: ObjectSlot) {
        self.ctx.reset_supply_dropzone_timer(owner);
    }

    fn eject_pilot_dying(&mut self, owner: ObjectSlot) {
        self.ctx.eject_pilot_dying(owner);
    }

    fn dispatch_attempt_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType) {
        self.ctx.dispatch_attempt_upgrade(owner, mask);
    }

    fn dispatch_reset_upgrade(&mut self, owner: ObjectSlot, mask: &UpgradeMaskType) {
        self.ctx.dispatch_reset_upgrade(owner, mask);
    }

    fn contain_add(&mut self, container: ObjectSlot, passenger: ObjectSlot) {
        self.ctx.contain_add(container, passenger);
    }

    fn contain_remove(
        &mut self,
        container: ObjectSlot,
        passenger: ObjectSlot,
        expose_stealthed: Bool,
    ) {
        self.ctx
            .contain_remove(container, passenger, expose_stealthed);
    }

    fn contain_remove_all(&mut self, container: ObjectSlot, expose_stealthed: Bool) {
        self.ctx.contain_remove_all(container, expose_stealthed);
    }

    fn contain_kill_all(&mut self, container: ObjectSlot) {
        self.ctx.contain_kill_all(container);
    }

    fn contain_harm_and_force_exit_all(&mut self, container: ObjectSlot, dmg: &DamageInfo) {
        self.ctx.contain_harm_and_force_exit_all(container, dmg);
    }

    fn set_body_damage_state(&mut self, owner: ObjectSlot, new_state: BodyDamageType) {
        self.ctx.set_body_damage_state(owner, new_state);
    }

    fn set_contained_by(&mut self, passenger: ObjectSlot, container: Option<ObjectSlot>) {
        self.ctx.set_contained_by(passenger, container);
    }

    fn set_in_partition(&mut self, owner: ObjectSlot, present: Bool) {
        self.ctx.set_in_partition(owner, present);
    }

    fn set_object_position(&mut self, owner: ObjectSlot, pos: &Coord3D) {
        self.ctx.set_object_position(owner, pos);
    }

    fn set_object_orientation(&mut self, owner: ObjectSlot, angle: Real) {
        self.ctx.set_object_orientation(owner, angle);
    }

    fn set_object_transform_matrix(&mut self, owner: ObjectSlot, mtx: &Matrix3D) {
        self.ctx.set_object_transform_matrix(owner, mtx);
    }

    fn ai_process_collision(&mut self, owner: ObjectSlot, other: ObjectSlot) -> Bool {
        self.ctx.ai_process_collision(owner, other)
    }

    fn projectile_handle_collision(
        &mut self,
        owner: ObjectSlot,
        other: Option<ObjectSlot>,
    ) -> Bool {
        self.ctx.projectile_handle_collision(owner, other)
    }

    fn set_object_layer(&mut self, owner: ObjectSlot, layer: PathfindLayerEnum) {
        self.ctx.set_object_layer(owner, layer);
    }

    fn tunnel_tracker_mut(&mut self, player_index: Int) -> &mut TunnelTrackerState {
        self.ctx.tunnel_tracker_mut(player_index)
    }

    fn cave_register(&mut self, cave_index: Int) {
        self.ctx.cave_register(cave_index);
    }

    fn cave_unregister(&mut self, cave_index: Int) {
        self.ctx.cave_unregister(cave_index);
    }

    fn cave_set_original_team(&mut self, cave: ObjectSlot, team: TeamId) -> Bool {
        self.ctx.cave_set_original_team(cave, team)
    }

    fn cave_tracker_mut(&mut self, cave_index: Int) -> Option<&mut TunnelTrackerState> {
        self.ctx.cave_tracker_mut(cave_index)
    }

    fn launch_projectile(
        &mut self,
        proj: ObjectSlot,
        victim: Option<ObjectSlot>,
        victim_pos: &Coord3D,
        launcher: ObjectSlot,
        wslot: WeaponSlotType,
        weapon_name: &str,
    ) {
        self.ctx
            .launch_projectile(proj, victim, victim_pos, launcher, wslot, weapon_name);
    }

    fn report_missile_to_countermeasures(&mut self, target: ObjectSlot, missile: ObjectSlot) {
        self.ctx
            .report_missile_to_countermeasures(target, missile);
    }

    fn divert_projectile_to_target(&mut self, missile: ObjectSlot, frames: UnsignedInt) {
        self.ctx.divert_projectile_to_target(missile, frames);
    }

    fn set_bridge_passable(&mut self, bridge_id: ObjectId, passable: Bool) {
        self.ctx.set_bridge_passable(bridge_id, passable);
    }

    fn notify_bridge_tower_damage(&mut self, bridge_slot: ObjectSlot, dmg: &mut DamageInfo) {
        self.ctx.notify_bridge_tower_damage(bridge_slot, dmg);
    }

    fn link_bridge_tower(
        &mut self,
        bridge_id: ObjectId,
        tower_type: BridgeTowerType,
        tower_id: ObjectId,
    ) {
        self.ctx.link_bridge_tower(bridge_id, tower_type, tower_id);
    }

    fn reserve_parking_space(
        &mut self,
        airfield: ObjectSlot,
        aircraft_id: ObjectId,
        offset: Real,
    ) -> Option<ParkingPlaceInfo> {
        self.ctx
            .reserve_parking_space(airfield, aircraft_id, offset)
    }

    fn release_parking_space(&mut self, airfield: ObjectSlot, aircraft_id: ObjectId) {
        self.ctx.release_parking_space(airfield, aircraft_id);
    }

    fn notify_spawn_slave_died(
        &mut self,
        master: ObjectSlot,
        slave_id: ObjectId,
        dmg: &DamageInfo,
    ) {
        self.ctx.notify_spawn_slave_died(master, slave_id, dmg);
    }

    fn start_rebuild_hole(
        &mut self,
        hole: ObjectSlot,
        rebuild_tmpl: ThingTemplateId,
        spawner_id: ObjectId,
    ) {
        self.ctx
            .start_rebuild_hole(hole, rebuild_tmpl, spawner_id);
    }

    fn record_mine_detonation(&mut self, mine_id: ObjectId, trigger_id: ObjectId) {
        self.ctx.record_mine_detonation(mine_id, trigger_id);
    }

    fn order_passenger(&mut self, passenger_id: ObjectId, order: &str) {
        self.ctx.order_passenger(passenger_id, order);
    }

    fn ai_mark_as_dead(&mut self, owner: ObjectSlot) {
        self.ctx.ai_mark_as_dead(owner);
    }

    fn deselect_object(
        &mut self,
        owner: ObjectSlot,
        player_mask: UnsignedShort,
        affect_drag_selection: Bool,
    ) {
        self.ctx
            .deselect_object(owner, player_mask, affect_drag_selection);
    }

    fn trigger_slow_death_with_caller(
        &mut self,
        owner: ObjectSlot,
        caller_slot: ModuleSlot,
        caller_applicable: Bool,
        caller_prob: Int,
        dmg: &DamageInfo,
    ) -> Bool {
        self.ctx.trigger_slow_death_with_caller(
            owner,
            caller_slot,
            caller_applicable,
            caller_prob,
            dmg,
        )
    }

    fn apply_fling_physics(&mut self, owner: ObjectSlot, force: &Coord3D, orientation: Real) {
        self.ctx.apply_fling_physics(owner, force, orientation);
    }

    fn notify_slaver_die(&mut self, owner: ObjectSlot) {
        self.ctx.notify_slaver_die(owner);
    }

    fn mark_stealth_detected(&mut self, owner: ObjectSlot, frames: UnsignedInt) {
        self.ctx.mark_stealth_detected(owner, frames);
    }

    fn receive_stealth_grant(&mut self, owner: ObjectSlot, active: Bool, frames: UnsignedInt) {
        self.ctx.receive_stealth_grant(owner, active, frames);
    }

    fn rng_client_real(&mut self, lo: Real, hi: Real) -> Real {
        self.ctx.rng_client_real(lo, hi)
    }

    fn reset_dynamic_physics(&mut self, owner: ObjectSlot) {
        self.ctx.reset_dynamic_physics(owner);
    }

    fn record_vehicle_disguised(&mut self, owner: ObjectSlot) {
        self.ctx.record_vehicle_disguised(owner);
    }

    fn contain_recalc_apparent_controlling_player(&mut self, container: ObjectSlot) {
        self.ctx.contain_recalc_apparent_controlling_player(container);
    }

    fn spawn_reveal_slaves(&mut self, owner: ObjectSlot) {
        self.ctx.spawn_reveal_slaves(owner);
    }

    fn stealth_disguise_as_object(&mut self, owner: ObjectSlot, target: Option<ObjectSlot>) {
        self.ctx.stealth_disguise_as_object(owner, target);
    }

    fn set_stealth_detector_enabled(&mut self, owner: ObjectSlot, enabled: Bool) {
        self.ctx.set_stealth_detector_enabled(owner, enabled);
    }

    fn order_ai_exit(
        &mut self,
        rider: ObjectSlot,
        container: ObjectSlot,
        cmd_source: CommandSourceType,
    ) {
        self.ctx.order_ai_exit(rider, container, cmd_source);
    }

    fn contain_process_damage_to_contained(&mut self, container: ObjectSlot, percent_damage: Real) {
        self.ctx
            .contain_process_damage_to_contained(container, percent_damage);
    }

    fn exit_object_via_door(&mut self, container: ObjectSlot, obj: ObjectSlot, door: ExitDoorType) {
        self.ctx.exit_object_via_door(container, obj, door);
    }

    fn set_object_team(&mut self, owner: ObjectSlot, new_team: TeamId) {
        self.ctx.set_object_team(owner, new_team);
    }

    fn set_physics_allow_to_fall(&mut self, owner: ObjectSlot, allow: Bool) {
        self.ctx.set_physics_allow_to_fall(owner, allow);
    }

    fn physics_clear_acceleration(&mut self, owner: ObjectSlot) {
        self.ctx.physics_clear_acceleration(owner);
    }

    fn physics_scrub_velocity_2d(&mut self, owner: ObjectSlot, desired_velocity: Real) {
        self.ctx
            .physics_scrub_velocity_2d(owner, desired_velocity);
    }

    fn physics_apply_force(&mut self, owner: ObjectSlot, force: &Coord3D) {
        self.ctx.physics_apply_force(owner, force);
    }

    fn physics_apply_shock(&mut self, owner: ObjectSlot, force: &Coord3D) {
        self.ctx.physics_apply_shock(owner, force);
    }

    fn physics_apply_motive_force(&mut self, owner: ObjectSlot, force: &Coord3D) {
        self.ctx.physics_apply_motive_force(owner, force);
    }

    fn physics_apply_random_rotation(&mut self, owner: ObjectSlot) {
        self.ctx.physics_apply_random_rotation(owner);
    }

    fn physics_set_roll_rate(&mut self, owner: ObjectSlot, rate: Real) {
        self.ctx.physics_set_roll_rate(owner, rate);
    }

    fn set_locomotor_max_braking(&mut self, owner: ObjectSlot, max_braking: Real) {
        self.ctx.set_locomotor_max_braking(owner, max_braking);
    }

    fn physics_set_yaw_rate(&mut self, owner: ObjectSlot, rate: Real) {
        self.ctx.physics_set_yaw_rate(owner, rate);
    }

    fn pathfinder_create_wall(&mut self, owner: ObjectSlot) {
        self.ctx.pathfinder_create_wall(owner);
    }

    fn pathfinder_remove_wall(&mut self, owner: ObjectSlot) {
        self.ctx.pathfinder_remove_wall(owner);
    }

    fn fire_projectile_detonation(&mut self, owner: ObjectSlot, weapon_name: &str, pos: &Coord3D, bonus_flags: UnsignedInt) {
        self.ctx.fire_projectile_detonation(owner, weapon_name, pos, bonus_flags);
    }

    fn record_cleared_garrisoned_building(&mut self, owner: ObjectSlot) {
        self.ctx.record_cleared_garrisoned_building(owner);
    }

    fn physics_set_pitch_rate(&mut self, owner: ObjectSlot, rate: Real) {
        self.ctx.physics_set_pitch_rate(owner, rate);
    }

    fn physics_set_is_in_free_fall(&mut self, owner: ObjectSlot, in_free_fall: Bool) {
        self.ctx.physics_set_is_in_free_fall(owner, in_free_fall);
    }

    fn ai_idle(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_idle(owner, cmd_source);
    }

    fn ai_hunt(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_hunt(owner, cmd_source);
    }

    fn ai_move_to_position(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        cmd_source: CommandSourceType,
    ) {
        self.ctx.ai_move_to_position(owner, pos, cmd_source);
    }

    fn ai_follow_path(
        &mut self,
        owner: ObjectSlot,
        path: &[Coord3D],
        ignore_obj: Option<ObjectSlot>,
        cmd_source: CommandSourceType,
    ) {
        self.ctx
            .ai_follow_path(owner, path, ignore_obj, cmd_source);
    }

    fn ai_hack_internet(&mut self, owner: ObjectSlot, cmd_source: CommandSourceType) {
        self.ctx.ai_hack_internet(owner, cmd_source);
    }

    fn ai_set_attitude(&mut self, owner: ObjectSlot, attitude: AttitudeType) {
        self.ctx.ai_set_attitude(owner, attitude);
    }

    fn ai_wake_up_and_attempt_to_target(&mut self, owner: ObjectSlot) {
        self.ctx.ai_wake_up_and_attempt_to_target(owner);
    }

    fn set_locomotor_max_lift(&mut self, owner: ObjectSlot, max_lift: Real) {
        self.ctx.set_locomotor_max_lift(owner, max_lift);
    }

    fn set_locomotor_max_turn_rate(&mut self, owner: ObjectSlot, max_turn_rate: Real) {
        self.ctx.set_locomotor_max_turn_rate(owner, max_turn_rate);
    }

    fn set_locomotor_ultra_accurate(&mut self, owner: ObjectSlot, ultra: Bool) {
        self.ctx.set_locomotor_ultra_accurate(owner, ultra);
    }

    fn set_locomotor_close_enough_dist(&mut self, owner: ObjectSlot, dist: Real) {
        self.ctx.set_locomotor_close_enough_dist(owner, dist);
    }

    fn set_locomotor_close_enough_dist_3d(&mut self, owner: ObjectSlot, is_3d: Bool) {
        self.ctx.set_locomotor_close_enough_dist_3d(owner, is_3d);
    }

    fn find_position_around(
        &mut self,
        center: &Coord3D,
        options: &FindPositionOptions,
    ) -> Option<Coord3D> {
        self.ctx.find_position_around(center, options)
    }

    fn pathfinder_add_object(&mut self, owner: ObjectSlot) {
        self.ctx.pathfinder_add_object(owner);
    }

    fn pathfinder_adjust_destination(&mut self, owner: ObjectSlot, dest: &mut Coord3D) -> Bool {
        self.ctx.pathfinder_adjust_destination(owner, dest)
    }

    fn ai_follow_exit_production_path(
        &mut self,
        owner: ObjectSlot,
        path: &[Coord3D],
        ignore_obj: Option<ObjectSlot>,
        cmd_source: CommandSourceType,
    ) {
        self.ctx
            .ai_follow_exit_production_path(owner, path, ignore_obj, cmd_source);
    }

    fn pathfinder_adjust_to_possible_destination(
        &mut self,
        owner: ObjectSlot,
        dest: &mut Coord3D,
    ) -> Bool {
        self.ctx
            .pathfinder_adjust_to_possible_destination(owner, dest)
    }

    fn pathfinder_update_goal(
        &mut self,
        owner: ObjectSlot,
        pos: &Coord3D,
        layer: PathfindLayerEnum,
    ) {
        self.ctx.pathfinder_update_goal(owner, pos, layer);
    }

    fn ai_construct(&mut self, owner: ObjectSlot) {
        self.ctx.ai_construct(owner);
    }

    fn ai_init_state_machine(&mut self, owner: ObjectSlot) {
        self.ctx.ai_init_state_machine(owner);
    }

    fn ai_choose_locomotor_set(&mut self, owner: ObjectSlot, set_type: LocomotorSetType) {
        self.ctx.ai_choose_locomotor_set(owner, set_type);
    }

    fn client(&mut self) -> &mut dyn ClientSink {
        self.ctx.client()
    }

    fn resident_state_mut(&mut self, module: ModuleSlot) -> Option<&mut dyn std::any::Any> {
        self.ctx.resident_state_mut(module)
    }

    fn paths(&mut self) -> &mut dyn PathServices {
        self.ctx.paths()
    }

    fn rng_client_int(&mut self, lo: Int, hi: Int) -> Int {
        self.ctx.rng_client_int(lo, hi)
    }

    fn set_object_construction_percent(&mut self, owner: ObjectSlot, percent: Real) {
        self.ctx.set_object_construction_percent(owner, percent);
    }

    fn topple_object(
        &mut self,
        owner: ObjectSlot,
        direction: &Coord3D,
        speed: Real,
        options: UnsignedInt,
    ) {
        self.ctx.topple_object(owner, direction, speed, options);
    }

    fn pathfinder_remove_object(&mut self, owner: ObjectSlot) {
        self.ctx.pathfinder_remove_object(owner);
    }

    fn stop_all_bone_fx(&mut self, owner: ObjectSlot) {
        self.ctx.stop_all_bone_fx(owner);
    }

    fn add_floating_money_text(&mut self, owner: ObjectSlot, pos: &Coord3D, amount: Int) {
        self.ctx.add_floating_money_text(owner, pos, amount);
    }

    fn auto_deposit_boost_upgrade(&mut self, name: &str) -> Option<UpgradeMaskType> {
        self.ctx.auto_deposit_boost_upgrade(name)
    }
}
