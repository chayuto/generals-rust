# Translation Packet `PKT-190`: `TransitionDamageFX`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-190`
- **Work Package:** `WP-5.1` (Phase 5)
- **Complexity & Assigned Tier:** Class `C1` -> `tier:1a-lite`
- **Environment Access:** `env:ephemeral-web`
- **FFI Seam Pattern:** Pattern 3: RustBehaviorModuleAdapter
- **Allowed Rust Output File:** `crates/zh_logic/src/modules/damage/transition_damage_fx.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (11):**
  - `Common/Xfer.h` -> `vendor/GeneralsX/Core/GameEngine/Include/Common/Xfer.h`
  - `GameClient/Drawable.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameClient/Drawable.h`
  - `GameClient/FXList.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/FXList.h`
  - `GameClient/ParticleSys.h` -> `vendor/GeneralsX/Core/GameEngine/Include/GameClient/ParticleSys.h`
  - `GameLogic/GameLogic.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/GameLogic.h`
  - `GameLogic/Module/BodyModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/DamageModule.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/DamageModule.h`
  - `GameLogic/Module/TransitionDamageFX.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransitionDamageFX.h`
  - `GameLogic/Object.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h` -> `vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/ObjectCreationList.h`
  - `PreRTS.h`
- **INI `FieldParse` Table Entries (111):**
  - `"DamageFXTypes" => INI::parseDamageTypeFlags`
  - `"DamageOCLTypes" => INI::parseDamageTypeFlags`
  - `"DamageParticleTypes" => INI::parseDamageTypeFlags`
  - `"DamagedFXList1" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList2" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList3" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList4" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList5" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList6" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList7" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList8" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList9" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList10" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList11" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedFXList12" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList1" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList2" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList3" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList4" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList5" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList6" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList7" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList8" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList9" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList10" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList11" => TransitionDamageFXModuleData::parseFXList`
  - `"ReallyDamagedFXList12" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList1" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList2" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList3" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList4" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList5" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList6" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList7" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList8" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList9" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList10" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList11" => TransitionDamageFXModuleData::parseFXList`
  - `"RubbleFXList12" => TransitionDamageFXModuleData::parseFXList`
  - `"DamagedOCL1" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL2" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL3" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL4" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL5" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL6" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL7" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL8" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL9" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL10" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL11" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedOCL12" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL1" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL2" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL3" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL4" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL5" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL6" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL7" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL8" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL9" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL10" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL11" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"ReallyDamagedOCL12" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL1" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL2" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL3" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL4" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL5" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL6" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL7" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL8" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL9" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL10" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL11" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"RubbleOCL12" => TransitionDamageFXModuleData::parseObjectCreationList`
  - `"DamagedParticleSystem1" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem2" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem3" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem4" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem5" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem6" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem7" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem8" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem9" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem10" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem11" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"DamagedParticleSystem12" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem1" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem2" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem3" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem4" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem5" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem6" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem7" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem8" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem9" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem10" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem11" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"ReallyDamagedParticleSystem12" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem1" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem2" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem3" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem4" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem5" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem6" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem7" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem8" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem9" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem10" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem11" => TransitionDamageFXModuleData::parseParticleSystem`
  - `"RubbleParticleSystem12" => TransitionDamageFXModuleData::parseParticleSystem`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (2 calls — preserve order identically):**
  1. `xferVersion( &version, currentVersion )`
  2. `xferUser( m_particleSystemID, sizeof( ParticleSystemID ) * BODYDAMAGETYPE_COUNT * DAMAGE_MODULE_MAX_FX )`
- **PRNG Calls (1 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue( 0, boneCount - 1 )`

## 2.5 Existing Rust Symbol Index (Use These Exact Paths)
- `zh_common::core::dict`: `DICT_MAX_LEN`, `DictDataType`, `fn`, `DictValue`, `Dict`
- `zh_common::core::name_key_generator`: `SOCKET_COUNT`, `NAMEKEY_MAX`, `NameKeyGenerator`
- `zh_common::core::state_machine`: `StateId`, `MACHINE_DONE_STATE_ID`, `INVALID_STATE_ID`, `EXIT_MACHINE_WITH_SUCCESS`, `EXIT_MACHINE_WITH_FAILURE`, `STATE_CONTINUE_CODE`, `STATE_SUCCESS_CODE`, `STATE_FAILURE_CODE`, `STATE_SLEEP_FOREVER_FRAMES`, `MAX_TRANSITION_DEPTH`, `StateReturnType`, `fn`, `StateExitType`, `StateWorldView`, ... (+8)
- `zh_common::crc`: `Crc`, `fn`
- `zh_common::ini`: `INI_MAX_CHARS_PER_LINE`, `LOGICFRAMES_PER_SECOND`, `MSEC_PER_SECOND`, `LOGICFRAMES_PER_MSEC_REAL`, `MSEC_PER_LOGICFRAME_REAL`, `LOGICFRAMES_PER_SECONDS_REAL`, `SECONDS_PER_LOGICFRAME_REAL`, `RADS_PER_DEGREE`, `SEPS_DEFAULT`, `SEPS_PERCENT`, `SEPS_COLON`, `SEPS_QUOTE`, `END_TOKEN`, `Color`, ... (+22)
- `zh_common::math`: `fn`, `Coord2D`, `Coord3D`, `ICoord2D`, `ICoord3D`, `Region2D`, `Region3D`, `IRegion2D`, `IRegion3D`, `RealRange`, `RGBColor`, `RGBAColorReal`, `RGBAColorInt`, `Matrix3D`
- `zh_common::math::bez_fwd_iterator`: `BezFwdIterator`
- `zh_common::math::bezier_segment`: `USUAL_TOLERANCE`, `BEZ_BASIS_MATRIX`, `BezierMath`, `BezierSegment`
- `zh_common::math::discrete_circle`: `HorzLine`, `DiscreteCircle`
- `zh_common::math::gamemath`: `WWMATH_PI`, `WWMATH_TWO_PI`, `PI`, `TWO_PI`, `WWMATH_EPSILON`
- `zh_common::math::geometry`: `GeometryType`, `fn`, `GEOMETRY_NAMES`, `GeometryInfo`
- `zh_common::math::partition_solver`: `PairObjectIdAndUInt`, `PairObjectId`, `EntriesVec`, `SpacesVec`, `SolutionVec`, `SolutionType`, `PartitionSolver`
- `zh_common::random_value`: `INITIAL_SEED_WORDS`, `THE_MULT_FACTOR`, `DistributionType`, `PrngState`, `fn`
- `zh_common::types`: `Int`, `UnsignedInt`, `Short`, `UnsignedShort`, `Byte`, `UnsignedByte`, `Bool`, `Real`, `XferVersion`, `FALSE`, `TRUE`, `ObjectId`, `INVALID`, `TeamId`, ... (+11)
- `zh_common::xfer`: `XferMode`, `XferError`, `Xfer`, `Snapshot`, `XferCrc`, `fn`, `XferDeepCrc`, `XferSave`, `XferLoad`
- `zh_common::xfer::bit_flags`: `ARMOR_SET_BIT_NAMES`, `MODEL_CONDITION_BIT_NAMES`, `ArmorSetFlags`, `ModelConditionFlags`, `BitFlags`, `NUM_BITS`, `fn`
- `zh_logic::arena`: `ObjectSlot`, `INVALID`, `fn`, `ModuleSlot`, `ModuleDataId`, `ThingTemplateId`, `FacetMask`, `NONE`, `UPDATE`, `DIE`, `DAMAGE`, `CREATE`, `COLLIDE`, `BODY`, ... (+34)
- `zh_logic::modules::module`: `ModuleData`, `StubModuleData`, `ModuleSnapshotHeader`, `BehaviorModule`, `UpdateFacet`, `BodyFacet`, `DieFacet`, `DamageFacet`, `CreateFacet`, `CollideFacet`, `ContainFacet`, `UpgradeFacet`, `SpecialPowerFacet`, `DestroyFacet`, ... (+3)
- `zh_logic::modules::module_factory`: `ModuleDataParseFn`, `ModuleCreateFn`, `BuiltinModuleEntry`, `BUILTIN_ZH_MODULES`, `W3D_DRAW_STUB_NAMES`, `ModuleDataStore`, `ModuleDescriptor`, `ModuleRegistry`
- `zh_logic::object::experience_tracker`: `ExperienceParentAccess`, `ExperienceTrackerState`, `fn`
- `zh_logic::object::object_script_status_bits`: `ObjectScriptStatusBit`, `fn`, `OBJECT_STATUS_SCRIPT_DISABLED`, `OBJECT_STATUS_SCRIPT_UNPOWERED`, `OBJECT_STATUS_SCRIPT_UNSELLABLE`, `OBJECT_STATUS_SCRIPT_UNSTEALTHED`, `OBJECT_STATUS_SCRIPT_TARGETABLE`
- `zh_logic::object::object_types`: `ObjectTypes`
- `zh_logic::object::overridable`: `OverridableEntry`, `fn`
- `zh_logic::object::override`: `OverrideStore`, `Override`, `fn`
- `zh_logic::object::thing`: `INITIAL_THING_POS_X`, `INITIAL_THING_POS_Y`, `ThingTerrainAccess`, `ThingTransformReactor`, `NoopTransformReactor`, `Thing`, `fn`
- `zh_logic::object::thing_factory`: `ThingFactory`, `fn`
- `zh_logic::object::thing_template`: `LEVEL_COUNT`, `MAX_UPGRADE_CAMEO_UPGRADES`, `USE_EXP_VALUE_FOR_SKILL_VALUE`, `RADAR_PRIORITY_NAMES`, `MODULEINTERFACE_UPDATE`, `MODULEINTERFACE_DIE`, `MODULEINTERFACE_DAMAGE`, `MODULEINTERFACE_CREATE`, `MODULEINTERFACE_COLLIDE`, `MODULEINTERFACE_BODY`, `MODULEINTERFACE_CONTAIN`, `MODULEINTERFACE_UPGRADE`, `MODULEINTERFACE_SPECIAL_POWER`, `MODULEINTERFACE_DESTROY`, ... (+27)
- `zh_logic::object::weapon`: `NO_MAX_SHOTS_LIMIT`, `PATHFIND_CELL_SIZE_F`, `WEAPON_UNDERSIZE`, `WeaponReloadType`, `fn`, `WeaponPrefireType`, `WEAPON_RELOAD_NAMES`, `WEAPON_PREFIRE_NAMES`, `WEAPON_ANTI_AIRBORNE_VEHICLE`, `WEAPON_ANTI_GROUND`, `WEAPON_ANTI_PROJECTILE`, `WEAPON_ANTI_SMALL_MISSILE`, `WEAPON_ANTI_MINE`, `WEAPON_ANTI_AIRBORNE_INFANTRY`, ... (+22)
- `zh_logic::object::weapon_set`: `WEAPONSLOT_COUNT`, `WeaponSlotType`, `fn`, `WeaponSetConditionType`, `WeaponChoiceCriteria`, `WeaponLockType`, `CanAttackResult`, `WeaponTemplateSet`, `WEAPON_SET_TYPE_TO_MODEL_CONDITION_TYPE_MAP`, `WeaponSet`
- `zh_logic::rules`: `AcademyClassificationType`, `fn`, `ACADEMY_CLASSIFICATION_TYPE_NAMES`
- `zh_logic::rules::combat_types`: `ARMORSET_COUNT`, `ArmorSetFlags`, `ARMOR_SET_NAMES`, `ArmorSetType`, `fn`, `DAMAGE_NUM_TYPES`, `DamageTypeFlags`, `DAMAGE_TYPE_NAMES`, `DamageType`, `DEATH_NUM_TYPES`, `DeathTypeFlags`, `DEATH_TYPE_FLAGS_ALL`, `DEATH_TYPE_FLAGS_NONE`, `DEATH_TYPE_NAMES`, ... (+14)
- `zh_logic::rules::disabled_types`: `DISABLED_COUNT`, `DISABLED_ANY_SENTINEL`, `DisabledMaskType`, `DisabledType`, `fn`, `DISABLED_TYPE_NAMES`
- `zh_logic::rules::kind_of`: `KINDOF_COUNT`, `KindOfMaskType`, `KindOfType`, `fn`, `KINDOF_NAMES`
- `zh_logic::rules::model_state`: `MODELCONDITION_COUNT`, `ModelConditionFlags`, `MODEL_CONDITION_NAMES`, `ModelConditionFlagType`, `fn`
- `zh_logic::rules::multiplayer_settings`: `Color`, `PLAYERTEMPLATE_RANDOM`, `PLAYERTEMPLATE_OBSERVER`, `PLAYERTEMPLATE_MIN`, `GAME_COLOR_UNDEFINED`, `MultiplayerColorDefinition`, `MultiplayerColorList`, `MultiplayerStartingMoneyList`, `MultiplayerSettings`
- `zh_logic::rules::object_status_types`: `OBJECT_STATUS_COUNT`, `ObjectStatusMaskType`, `ObjectStatusType`, `fn`, `OBJECT_STATUS_NAMES`
- `zh_logic::rules::player_template`: `MAX_MP_STARTING_UNITS`, `VETERANCY_NAMES`, `ProductionChangeMap`, `ProductionVeterancyMap`, `PlayerTemplateImageLookup`, `PlayerTemplate`, `PlayerTemplateStore`
- `zh_logic::rules::production_prerequisite`: `UNIT_OR_WITH_PREV`, `MAX_PREREQ`, `ThingTemplateRef`, `ThingTemplateRegistry`, `PrereqPlayerView`, `PrereqTextLookup`, `ProductionPrerequisite`
- `zh_logic::rules::rank_info`: `RankInfoData`, `RankInfoStore`
- `zh_logic::rules::science`: `ScienceType`, `SCIENCE_INVALID`, `ScienceVec`, `SciencePlayerAccess`, `ScienceTextLookup`, `ScienceInfoData`, `ScienceStore`
- `zh_logic::rules::special_power`: `DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT`, `SpecialPowerObjectAccess`, `SpecialPowerTemplateData`, `SpecialPowerTemplate`, `fn`, `SpecialPowerStore`
- `zh_logic::rules::special_power_types`: `SPECIALPOWER_COUNT`, `SpecialPowerMaskType`, `SpecialPowerType`, `fn`, `SPECIAL_POWER_NAMES`
- `zh_logic::rules::upgrade`: `UPGRADE_MAX_COUNT`, `UpgradeMaskType`, `UpgradeStatusType`, `fn`, `UpgradeType`, `UPGRADE_TYPE_NAMES`, `UpgradePlayerAccess`, `Upgrade`, `UpgradeTemplate`, `UpgradeCenter`
- `zh_logic::scheduler::sleepy_heap`: `UPDATE_SLEEP_INVALID`, `UPDATE_SLEEP_NONE`, `UPDATE_SLEEP_FOREVER`, `INVALID_LOGIC_INDEX`, `MAX_SLEEPY_UPDATES_PER_OBJECT`, `SleepyUpdatePhase`, `fn`, `UpdateModuleScheduleState`, `ScheduleStore`, `SleepyUpdateHeap`
- `zh_logic::sim`: `ClientSink`, `NoopClientSink`, `FlatTerrain`, `RulesRegistry`, `ObjectRecord`, `SimView`, `SimApi`, `SimWorld`, `SimContext`, `MockSimApi`
- `zh_logic::combat::armor`: `ArmorTemplate`, `Armor`, `fn`, `ArmorStore`
- `zh_logic::combat::armor_set`: `ArmorTemplateSet`
- `zh_logic::combat::damage`: `PlayerMaskType`, `HUGE_DAMAGE_AMOUNT`, `DamageInfoInput`, `DamageInfoOutput`, `DamageInfo`
- `zh_logic::combat::damage_fx`: `DamageFxSlot`, `fn`, `DamageFX`, `DamageFXStore`
- `zh_logic::combat::firing_tracker`: `AHSV_NO_SOUND`, `FiringTrackerModuleData`, `FiredWeaponInfo`, `FiringTrackerHost`, `FiringTracker`, `fn`
- `zh_logic::combat::locomotor`: `BIGNUM`, `LocomotorAppearance`, `fn`, `LOCOMOTOR_APPEARANCE_NAMES`, `LocomotorPriority`, `LOCOMOTOR_PRIORITY_NAMES`, `LocomotorBehaviorZ`, `LOCOMOTOR_BEHAVIOR_Z_NAMES`, `LocomotorTemplate`, `LocomotorStore`, `Locomotor`
- `zh_logic::combat::locomotor_set`: `LOCOMOTORSET_COUNT`, `LocomotorSetType`, `fn`, `LOCOMOTOR_SET_NAMES`, `LocomotorSet`
- `zh_logic::combat::turret_ai`: `DEFAULT_TURN_RATE`, `DEFAULT_PITCH_RATE`, `WAIT_INDEFINITELY`, `WhichTurretType`, `MAX_TURRETS`, `fn`, `TurretStateType`, `NUM_STATES`, `TurretTargetType`, `TurretAIError`, `FireStateStep`, `TurretWorldAccess`, `TurretAIData`, `TurretStateMachine`, ... (+1)
- `zh_logic::economy`: `MAX_PLAYER_COUNT`
- `zh_logic::economy::academy_stats`: `MAX_ADVICE_TIPS`, `FRAMES_BETWEEN_UPDATES`, `DEFAULT_SUPPLY_CENTER_COST`, `KINDOF_DOZER`, `KINDOF_FS_SUPPLY_CENTER`, `KINDOF_COMMANDCENTER`, `KINDOF_INFANTRY`, `KINDOF_VEHICLE`, `KINDOF_HARVESTER`, `KINDOF_HERO`, `KINDOF_FS_STRATEGY_CENTER`, `KINDOF_MONEY_HACKER`, `KINDOF_FS_BLACK_MARKET`, `KINDOF_FS_SUPPLY_DROPZONE`, ... (+9)

---

## Compact Worker Cheat Sheet (Embedded in All Translation Packets)

### 1. Primitive Type Mapping
| C++ Type | Rust Type | Size | Notes |
|---|---|---:|---|
| `Int` | `i32` | 4 | Two's-complement 32-bit signed |
| `UnsignedInt` | `u32` | 4 | 32-bit unsigned |
| `Short` | `i16` | 2 | 16-bit signed |
| `UnsignedShort` | `u16` | 2 | 16-bit unsigned |
| `Byte` / `char` | `i8` | 1 | Signed 8-bit (`char` is signed on x86/macOS) |
| `UnsignedByte` | `u8` | 1 | Unsigned 8-bit |
| `Bool` | `u8` (`Bool`) | 1 | `FALSE = 0`, `TRUE = 1` (1 byte in `Xfer`!) |
| `Real` | `f32` | 4 | IEEE-754 binary32, round-to-nearest-even, no FMA |
| `ObjectID` / `TeamID` / `DrawableID` | `ObjectId(pub i32)` etc. | 4 | `INVALID_ID = 0` |
| `NameKeyType` | `NameKeyType(pub i32)` | 4 | `NAMEKEY_INVALID = 0` |
| `UpdateSleepTime` | `UpdateSleepTime(pub u32)` | 4 | `NONE = 1`, `FOREVER = 0x3fff_ffff` |

### 2. Ten Non-Negotiable Determinism Rules
1. **Wrapping Integer Arithmetic:** Never use bare `+`, `-`, `*`, `+=`, `-=`, `*=` on `i32`/`u32` where C++ can overflow or wrap. Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`.
2. **Unfused IEEE-754 `f32` Arithmetic:** Never call `f32::mul_add` (FMA). Keep exact C++ parenthesization and left-to-right operator grouping: `(a * b) * c + d` must not be rewritten as `a * (b * c) + d`.
3. **Deterministic Transcendentals via `GameMath`:** Never call standard `f32::sin`, `cos`, `tan`, `asin`, `acos`, `atan2`, `sqrt`, `floor`, `ceil`, `powf`, `exp`, or `ln`. Always call `zh_common::gamemath::{gm_sinf, gm_cosf, gm_tanf, gm_asinf, gm_acosf, gm_atan2f, gm_sqrtf, gm_floorf, gm_ceilf, gm_powf, gm_expf, gm_logf}`.
4. **Mixed `Int * Real` Promotion & Float-to-Int Truncation:**
   - In C++, `Int a; Real s; a *= s;` evaluates `a = static_cast<Int>(static_cast<Real>(a) * s)`.
   - In Rust, always write: `a = ((a as f32) * s) as i32;` (or `zh_common::math::cpp_mul_i32_f32(a, s)`).
   - For `REAL_TO_INT_FLOOR(x)` and `REAL_TO_INT_CEIL(x)`, call `zh_common::math::{real_to_int_floor, real_to_int_ceil}` (which route to `gm_floorf(x) as i32` / `gm_ceilf(x) as i32` in default deterministic mode, and `fast_float_floor`/`ceil` under `#[cfg(feature = "retail-crc")]`).
5. **No `HashMap` or `HashSet`:** Never import `std::collections::{HashMap, HashSet}`. Always use `std::collections::{BTreeMap, BTreeSet}` keyed by a deterministic integer ID (`ObjectId`, `NameKeyType`, `TeamId`, `u32`), never by raw pointer address.
6. **No `BinaryHeap` for Sleepy Updates:** Use `zh_logic::SleepyUpdateHeap` (strict `>` comparator, left-child tie-breaking, in-place root sifting, reverse-heap-index removal).
7. **Evaluation Order of Side-Effecting Arguments:** When a C++ function call has 2+ arguments that call `GameLogicRandomValue*` or mutate state, evaluate them into explicit `let arg0 = ...; let arg1 = ...;` bindings in left-to-right order before the call.
8. **Exact PRNG Call Parity:** `GameLogicRandomValue(lo, hi)` and `GameLogicRandomValueReal(lo, hi)` return `hi` **without advancing the seed** when `lo >= hi` (under default `RETAIL_COMPATIBLE_CRC == 0`). Never reorder, hoist, or short-circuit a branch that calls `GameLogicRandomValue*`.
9. **Exact `Xfer` & `XferCRC` Field Order and Widths:**
   - Serialize fields in the **exact** C++ `crc(Xfer*)` and `xfer(Xfer*)` order.
   - Never serialize `usize` or `isize`; cast lengths to `u16`, `u32`, or `i32` matching the C++ local variable type (`UnsignedShort`, `UnsignedInt`, `Int`).
   - Remember `Bool` is `1 byte` (`u8`), and enum values serialized via `xferUser(&e, sizeof(Enum))` are `4 bytes` (`i32`).
10. **Preserve `#if !RETAIL_COMPATIBLE_CRC` Guards:** Translate `#if RETAIL_COMPATIBLE_CRC` / `#if !RETAIL_COMPATIBLE_CRC` blocks using `#[cfg(feature = "retail-crc")]` and `#[cfg(not(feature = "retail-crc"))]`.

---

## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Module/TransitionDamageFX.h`
```cpp
// FILE: TransitionDamageFX.h /////////////////////////////////////////////////////////////////////
// Author: Colin Day, March 2002
// Desc:   Damage module capable of launching various effects on damage transitions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// USER INCLUDES //////////////////////////////////////////////////////////////////////////////////
#include "GameClient/ParticleSys.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/BodyModule.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
class Thing;
class FXList;
class ObjectCreationList;
class ParticleSystemTemplate;

// we can have this many effects of each type per body state
// NOTE: If you change this you should update the dataFieldParse[] table in the FXDamageModuleData
//       to allow for the new indices into the effect arrays
enum { DAMAGE_MODULE_MAX_FX = 12 };

typedef Char FXDamageLocType;
enum
{
	FX_DAMAGE_LOC_TYPE_BONE  = 0,
	FX_DAMAGE_LOC_TYPE_COORD = 1
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct FXLocInfo
{
	FXDamageLocType locType;			// location is bone or coord
	AsciiString boneName;					// bone name to use for effect pos
	Bool randomBone;							// if true, boneName is a prefix and we will do the effect at one of the bones with that prefix
	Coord3D loc;									// location is a specified point in space
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct FXDamageFXListInfo
{
	const FXList *fx;
	FXLocInfo locInfo;
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct FXDamageOCLInfo
{
	const ObjectCreationList *ocl;
	FXLocInfo locInfo;
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
struct FXDamageParticleSystemInfo
{
	const ParticleSystemTemplate *particleSysTemplate;
	FXLocInfo locInfo;
};

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
class TransitionDamageFXModuleData : public DamageModuleData
{

public:

	TransitionDamageFXModuleData();

	static void buildFieldParse(MultiIniFieldParse& p)
	{
    DamageModuleData::buildFieldParse(p);

		static const FieldParse dataFieldParse[] =
		{

			{ "DamageFXTypes",				INI::parseDamageTypeFlags, nullptr, offsetof( TransitionDamageFXModuleData, m_damageFXTypes ) },
			{ "DamageOCLTypes",				INI::parseDamageTypeFlags, nullptr, offsetof( TransitionDamageFXModuleData, m_damageOCLTypes ) },
			{ "DamageParticleTypes",	INI::parseDamageTypeFlags, nullptr, offsetof( TransitionDamageFXModuleData, m_damageParticleTypes ) },

			{ "DamagedFXList1",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 0 ] ) },
			{ "DamagedFXList2",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 1 ] ) },
			{ "DamagedFXList3",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 2 ] ) },
			{ "DamagedFXList4",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 3 ] ) },
			{ "DamagedFXList5",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 4 ] ) },
			{ "DamagedFXList6",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 5 ] ) },
			{ "DamagedFXList7",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 6 ] ) },
			{ "DamagedFXList8",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 7 ] ) },
			{ "DamagedFXList9",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 8 ] ) },
			{ "DamagedFXList10",							TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 9 ] ) },
			{ "DamagedFXList11",							TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 10 ] ) },
			{ "DamagedFXList12",							TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_DAMAGED ][ 11 ] ) },
			{ "ReallyDamagedFXList1",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 0 ] ) },
			{ "ReallyDamagedFXList2",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 1 ] ) },
			{ "ReallyDamagedFXList3",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 2 ] ) },
			{ "ReallyDamagedFXList4",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 3 ] ) },
			{ "ReallyDamagedFXList5",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 4 ] ) },
			{ "ReallyDamagedFXList6",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 5 ] ) },
			{ "ReallyDamagedFXList7",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 6 ] ) },
			{ "ReallyDamagedFXList8",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 7 ] ) },
			{ "ReallyDamagedFXList9",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 8 ] ) },
			{ "ReallyDamagedFXList10",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 9 ] ) },
			{ "ReallyDamagedFXList11",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 10 ] ) },
			{ "ReallyDamagedFXList12",					TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_REALLYDAMAGED ][ 11 ] ) },
			{ "RubbleFXList1",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 0 ] ) },
			{ "RubbleFXList2",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 1 ] ) },
			{ "RubbleFXList3",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 2 ] ) },
			{ "RubbleFXList4",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 3 ] ) },
			{ "RubbleFXList5",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 4 ] ) },
			{ "RubbleFXList6",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 5 ] ) },
			{ "RubbleFXList7",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 6 ] ) },
			{ "RubbleFXList8",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 7 ] ) },
			{ "RubbleFXList9",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 8 ] ) },
			{ "RubbleFXList10",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 9 ] ) },
			{ "RubbleFXList11",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 10 ] ) },
			{ "RubbleFXList12",								TransitionDamageFXModuleData::parseFXList,							nullptr,	offsetof( TransitionDamageFXModuleData, m_fxList[ BODY_RUBBLE ][ 11 ] ) },

			{ "DamagedOCL1",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 0 ] ) },
			{ "DamagedOCL2",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 1 ] ) },
			{ "DamagedOCL3",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 2 ] ) },
			{ "DamagedOCL4",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 3 ] ) },
			{ "DamagedOCL5",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 4 ] ) },
			{ "DamagedOCL6",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 5 ] ) },
			{ "DamagedOCL7",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 6 ] ) },
			{ "DamagedOCL8",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 7 ] ) },
			{ "DamagedOCL9",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 8 ] ) },
			{ "DamagedOCL10",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 9 ] ) },
			{ "DamagedOCL11",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 10 ] ) },
			{ "DamagedOCL12",									TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_DAMAGED ][ 11 ] ) },
			{ "ReallyDamagedOCL1",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 0 ] ) },
			{ "ReallyDamagedOCL2",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 1 ] ) },
			{ "ReallyDamagedOCL3",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 2 ] ) },
			{ "ReallyDamagedOCL4",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 3 ] ) },
			{ "ReallyDamagedOCL5",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 4 ] ) },
			{ "ReallyDamagedOCL6",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 5 ] ) },
			{ "ReallyDamagedOCL7",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 6 ] ) },
			{ "ReallyDamagedOCL8",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 7 ] ) },
			{ "ReallyDamagedOCL9",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 8 ] ) },
			{ "ReallyDamagedOCL10",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 9 ] ) },
			{ "ReallyDamagedOCL11",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 10 ] ) },
			{ "ReallyDamagedOCL12",						TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_REALLYDAMAGED ][ 11 ] ) },
			{ "RubbleOCL1",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 0 ] ) },
			{ "RubbleOCL2",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 1 ] ) },
			{ "RubbleOCL3",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 2 ] ) },
			{ "RubbleOCL4",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 3 ] ) },
			{ "RubbleOCL5",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 4 ] ) },
			{ "RubbleOCL6",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 5 ] ) },
			{ "RubbleOCL7",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 6 ] ) },
			{ "RubbleOCL8",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 7 ] ) },
			{ "RubbleOCL9",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 8 ] ) },
			{ "RubbleOCL10",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 9 ] ) },
			{ "RubbleOCL11",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 10 ] ) },
			{ "RubbleOCL12",										TransitionDamageFXModuleData::parseObjectCreationList,  nullptr, offsetof( TransitionDamageFXModuleData, m_OCL[ BODY_RUBBLE ][ 11 ] ) },

			{ "DamagedParticleSystem1",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 0 ] ) },
			{ "DamagedParticleSystem2",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 1 ] ) },
			{ "DamagedParticleSystem3",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 2 ] ) },
			{ "DamagedParticleSystem4",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 3 ] ) },
			{ "DamagedParticleSystem5",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 4 ] ) },
			{ "DamagedParticleSystem6",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 5 ] ) },
			{ "DamagedParticleSystem7",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 6 ] ) },
			{ "DamagedParticleSystem8",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 7 ] ) },
			{ "DamagedParticleSystem9",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 8 ] ) },
			{ "DamagedParticleSystem10",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 9 ] ) },
			{ "DamagedParticleSystem11",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 10 ] ) },
			{ "DamagedParticleSystem12",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_DAMAGED ][ 11 ] ) },
			{ "ReallyDamagedParticleSystem1",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 0 ] ) },
			{ "ReallyDamagedParticleSystem2",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 1 ] ) },
			{ "ReallyDamagedParticleSystem3",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 2 ] ) },
			{ "ReallyDamagedParticleSystem4",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 3 ] ) },
			{ "ReallyDamagedParticleSystem5",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 4 ] ) },
			{ "ReallyDamagedParticleSystem6",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 5 ] ) },
			{ "ReallyDamagedParticleSystem7",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 6 ] ) },
			{ "ReallyDamagedParticleSystem8",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 7 ] ) },
			{ "ReallyDamagedParticleSystem9",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 8 ] ) },
			{ "ReallyDamagedParticleSystem10",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 9 ] ) },
			{ "ReallyDamagedParticleSystem11",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 10 ] ) },
			{ "ReallyDamagedParticleSystem12",	TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_REALLYDAMAGED ][ 11 ] ) },
			{ "RubbleParticleSystem1",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 0 ] ) },
			{ "RubbleParticleSystem2",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 1 ] ) },
			{ "RubbleParticleSystem3",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 2 ] ) },
			{ "RubbleParticleSystem4",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 3 ] ) },
			{ "RubbleParticleSystem5",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 4 ] ) },
			{ "RubbleParticleSystem6",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 5 ] ) },
			{ "RubbleParticleSystem7",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 6 ] ) },
			{ "RubbleParticleSystem8",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 7 ] ) },
			{ "RubbleParticleSystem9",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 8 ] ) },
			{ "RubbleParticleSystem10",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 9 ] ) },
			{ "RubbleParticleSystem11",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 10 ] ) },
			{ "RubbleParticleSystem12",				TransitionDamageFXModuleData::parseParticleSystem,			nullptr, offsetof( TransitionDamageFXModuleData, m_particleSystem[ BODY_RUBBLE ][ 11 ] ) },

			{ nullptr, nullptr, nullptr, 0 }
		};
    p.add(dataFieldParse);

	}

	static void parseFXList( INI *ini, void *instance, void *store, const void *userData );
	static void parseObjectCreationList( INI *ini, void *instance, void *store, const void *userData );
	static void parseParticleSystem( INI *ini, void *instance, void *store, const void *userData );

	DamageTypeFlags m_damageFXTypes;		///< flags used to play or not play the effects
	FXDamageFXListInfo m_fxList[ BODYDAMAGETYPE_COUNT ][ DAMAGE_MODULE_MAX_FX ];
	DamageTypeFlags m_damageOCLTypes;		///< flags used to play or not play the effects
	FXDamageOCLInfo m_OCL[ BODYDAMAGETYPE_COUNT ][ DAMAGE_MODULE_MAX_FX ];
	DamageTypeFlags m_damageParticleTypes;		///< flags used to play or not play the effects
	FXDamageParticleSystemInfo m_particleSystem[ BODYDAMAGETYPE_COUNT ][ DAMAGE_MODULE_MAX_FX ];

};

//-------------------------------------------------------------------------------------------------
class TransitionDamageFX : public DamageModule
{

	MAKE_STANDARD_MODULE_MACRO_WITH_MODULE_DATA( TransitionDamageFX, TransitionDamageFXModuleData );
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( TransitionDamageFX, "TransitionDamageFX" )

public:

	TransitionDamageFX( Thing *thing, const ModuleData* moduleData );
	// virtual destructor prototype provided by memory pool declaration

	virtual void onDamage( DamageInfo *damageInfo ) override { }
	virtual void onHealing( DamageInfo *damageInfo ) override { }
	virtual void onBodyDamageStateChange( const DamageInfo* damageInfo,
																				BodyDamageType oldState,
																				BodyDamageType newState ) override;

protected:

	/// we keep a record of attached particle system so we can detach and kill them when we want to
	ParticleSystemID m_particleSystemID[ BODYDAMAGETYPE_COUNT ][ DAMAGE_MODULE_MAX_FX ];

};
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Damage/TransitionDamageFX.cpp`
```cpp
// FILE: TransitionDamageFX.cpp ///////////////////////////////////////////////////////////////////
// Author: Colin Day, March 2002
// Desc:   Damage module capable of launching various effects on damage transitions
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "Common/Xfer.h"
#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/Module/TransitionDamageFX.h"

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
TransitionDamageFXModuleData::TransitionDamageFXModuleData()
{
	Int i, j;

	for( i = 0; i < BODYDAMAGETYPE_COUNT; i++ )
	{

		for( j = 0; j < DAMAGE_MODULE_MAX_FX; j++ )
		{

			m_fxList[ i ][ j ].fx = nullptr;
			m_fxList[ i ][ j ].locInfo.loc.x = 0.0f;
			m_fxList[ i ][ j ].locInfo.loc.y = 0.0f;
			m_fxList[ i ][ j ].locInfo.loc.z = 0.0f;
			m_fxList[ i ][ j ].locInfo.locType = FX_DAMAGE_LOC_TYPE_COORD;
			m_fxList[ i ][ j ].locInfo.randomBone = FALSE;
			m_OCL[ i ][ j ].ocl = nullptr;
			m_OCL[ i ][ j ].locInfo.loc.x = 0.0f;
			m_OCL[ i ][ j ].locInfo.loc.y = 0.0f;
			m_OCL[ i ][ j ].locInfo.loc.z = 0.0f;
			m_OCL[ i ][ j ].locInfo.locType = FX_DAMAGE_LOC_TYPE_COORD;
			m_OCL[ i ][ j ].locInfo.randomBone = FALSE;
			m_particleSystem[ i ][ j ].particleSysTemplate = nullptr;
			m_particleSystem[ i ][ j ].locInfo.loc.x = 0.0f;
			m_particleSystem[ i ][ j ].locInfo.loc.y = 0.0f;
			m_particleSystem[ i ][ j ].locInfo.loc.z = 0.0f;
			m_particleSystem[ i ][ j ].locInfo.locType = FX_DAMAGE_LOC_TYPE_COORD;
			m_particleSystem[ i ][ j ].locInfo.randomBone = FALSE;

		}

	}

	m_damageFXTypes = DAMAGE_TYPE_FLAGS_ALL;
	m_damageOCLTypes = DAMAGE_TYPE_FLAGS_ALL;
	m_damageParticleTypes = DAMAGE_TYPE_FLAGS_ALL;

}

//-------------------------------------------------------------------------------------------------
/** Parse fx location info ... that is a named bone or a coord3d position */
//-------------------------------------------------------------------------------------------------
static void parseFXLocInfo( INI *ini, void *instance, FXLocInfo *locInfo )
{
	const char *token = ini->getNextToken( ini->getSepsColon() );

	if( stricmp( token, "bone" ) == 0 )
	{

		// save bone name and location type
		locInfo->boneName = AsciiString( ini->getNextToken() );
		locInfo->locType = FX_DAMAGE_LOC_TYPE_BONE;

		//
		// bones are followed by RandomBone:<Yes|No>, if random bone is yes, the bone name is
		// assumed to be a "base bone name" and we will find all the bones with that prefix
		// when picking an effect position.  If it's no, the bone name is assumed to be explicit
		//
		token = ini->getNextToken( ini->getSepsColon() );
		if( stricmp( token, "randombone" ) != 0 )
		{

			DEBUG_CRASH(( "parseFXLocInfo: Bone name not followed by RandomBone specifier\nPress IGNORE to see which INI file and line # is incorrect." ));
			throw INI_INVALID_DATA;

		}

		// parse the Bool definition
		ini->parseBool( ini, instance, &locInfo->randomBone, nullptr );

	}
	else if( stricmp( token, "loc" ) == 0 )
	{

		// save location and location type
		locInfo->loc.x = ini->scanReal( ini->getNextSubToken("X") );
		locInfo->loc.y = ini->scanReal( ini->getNextSubToken("Y") );
		locInfo->loc.z = ini->scanReal( ini->getNextSubToken("Z") );
		locInfo->locType = FX_DAMAGE_LOC_TYPE_COORD;

	}
	else
	{

		// error
		throw INI_INVALID_DATA;

	}

}

//-------------------------------------------------------------------------------------------------
/** In the form of:
	* FXListSlot = <<Bone:BoneName BoneRandom:<Yes|No>> | <Loc: X:x Y:y Z:z>> FXList:FXListName */
//-------------------------------------------------------------------------------------------------
void TransitionDamageFXModuleData::parseFXList( INI *ini, void *instance,
																								void *store, const void *userData )
{
	const char *token;
	FXDamageFXListInfo *info = (FXDamageFXListInfo *)store;

	// parse the location bone or location
	parseFXLocInfo( ini, instance, &info->locInfo );

	// make sure we have an "FXList:" token
	token = ini->getNextToken( ini->getSepsColon() );
	if( stricmp( token, "fxlist" ) != 0 )
	{

		// error
		throw INI_INVALID_DATA;

	}

	// parse the fx list name
	ini->parseFXList( ini, instance, &info->fx, nullptr );

}

//-------------------------------------------------------------------------------------------------
/** In the form of:
	* OCLSlot = <<Bone:BoneName BoneRandom:<Yes|No>> | <Loc: X:x Y:y Z:z>> OCL:OCLName */
//-------------------------------------------------------------------------------------------------
void TransitionDamageFXModuleData::parseObjectCreationList( INI *ini, void *instance,
																														void *store, const void *userData )
{
	const char *token;
	FXDamageOCLInfo *info = (FXDamageOCLInfo *)store;

	// parse the location bone or location
	parseFXLocInfo( ini, instance, &info->locInfo );

	// make sure we have an "OCL:" token
	token = ini->getNextToken( ini->getSepsColon() );
	if( stricmp( token, "ocl" ) != 0 )
	{

		// error
		throw INI_INVALID_DATA;

	}

	// parse the ocl name
	ini->parseObjectCreationList( ini, instance, store, &info->ocl );

}

//-------------------------------------------------------------------------------------------------
/** In the form of:
	* ParticleSlot = <<Bone:BoneName BoneRandom:<Yes|No>> | <Loc: X:x Y:y Z:z>> PSys:PSysName */
//-------------------------------------------------------------------------------------------------
void TransitionDamageFXModuleData::parseParticleSystem( INI *ini, void *instance,
																												void *store, const void *userData )
{
	const char *token;
	FXDamageParticleSystemInfo *info = (FXDamageParticleSystemInfo *)store;

	// parse the location bone or location
	parseFXLocInfo( ini, instance, &info->locInfo );

	// make sure we have an "PSys:" token
	token = ini->getNextToken( ini->getSepsColon() );
	if( stricmp( token, "psys" ) != 0 )
	{

		// error
		throw INI_INVALID_DATA;

	}

	// parse the particle system name
	ini->parseParticleSystemTemplate( ini, instance, store, &info->particleSysTemplate );

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
TransitionDamageFX::TransitionDamageFX( Thing *thing, const ModuleData* moduleData )
																		  : DamageModule( thing, moduleData )
{
	Int i, j;

	for( i = 0; i < BODYDAMAGETYPE_COUNT; i++ )
		for( j = 0; j < DAMAGE_MODULE_MAX_FX; j++ )
			m_particleSystemID[ i ][ j ] = INVALID_PARTICLE_SYSTEM_ID;

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
TransitionDamageFX::~TransitionDamageFX()
{

}

/*
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void TransitionDamageFX::onDelete()
{

	//
	// we would in theory delete any particle systems we have created and attached, but the
	// particle system will automatically delete itself when the object is destroyed
	//

}
*/

//-------------------------------------------------------------------------------------------------
/** Given an FXLoc info struct, return the effect position that we are supposed to use.
	* The position is local to to the object */
//-------------------------------------------------------------------------------------------------
static Coord3D getLocalEffectPos( const FXLocInfo *locInfo, Drawable *draw )
{

	DEBUG_ASSERTCRASH( locInfo, ("getLocalEffectPos: locInfo is null") );

	if( locInfo->locType == FX_DAMAGE_LOC_TYPE_BONE && draw )
	{

		if( locInfo->randomBone == FALSE )
		{
			Coord3D pos;

			// get the bone position
			Int count = draw->getPristineBonePositions( locInfo->boneName.str(), 0, &pos, nullptr, 1 );

			// sanity, if bone not found revert back to location defined in struct (which is 0,0,0)
			if( count == 0 )
				return locInfo->loc;

			// return the position retrieved
			return pos;

		}
		else
		{
		  const Int MAX_BONES = 32;
			Coord3D positions[ MAX_BONES ];

			// get the bone positions
			Int boneCount;
			boneCount = draw->getPristineBonePositions( locInfo->boneName.str(), 1, positions, nullptr, MAX_BONES );

			// sanity, if bone not found revert back to location defined in struct (which is 0,0,0)
			if( boneCount == 0 )
				return locInfo->loc;

			// pick one of the bone positions
			Int pick = GameLogicRandomValue( 0, boneCount - 1 );
			return positions[ pick ];

		}

	}
	else
		return locInfo->loc;

}

//-------------------------------------------------------------------------------------------------
/** Switching damage states */
//-------------------------------------------------------------------------------------------------
void TransitionDamageFX::onBodyDamageStateChange( const DamageInfo* damageInfo,
																									BodyDamageType oldState,
																									BodyDamageType newState )
{
	Object *damageSource = nullptr;
	Int i;
	Drawable *draw = getObject()->getDrawable();
	const TransitionDamageFXModuleData *modData = getTransitionDamageFXModuleData();

	// get the source of the damage if present
	if( damageInfo )
		damageSource = TheGameLogic->findObjectByID( damageInfo->in.m_sourceID );

	// remove any particle systems that might be emitting from our old state
	for( i = 0; i < DAMAGE_MODULE_MAX_FX; i++ )
	{

		if( m_particleSystemID[ oldState ][ i ] != INVALID_PARTICLE_SYSTEM_ID )
		{

			TheParticleSystemManager->destroyParticleSystemByID( m_particleSystemID[ oldState ][ i ] );
			m_particleSystemID[ oldState ][ i ] = INVALID_PARTICLE_SYSTEM_ID;

		}

	}

	//
	// when we are transitioning to a "worse" state we will play a set of effects for that
	// new state to make the transition
	//
	if( IS_CONDITION_WORSE( newState, oldState ) )
	{
		const ParticleSystemTemplate *pSystemT;
		Coord3D pos;

		// if we are restricted by the damage type executing effect, bail out of here
		const DamageInfo *lastDamageInfo = getObject()->getBodyModule()->getLastDamageInfo();

		for( i = 0; i < DAMAGE_MODULE_MAX_FX; i++ )
		{

			// play fx list for our new state
			if( modData->m_fxList[ newState ][ i ].fx )
			{

				if( lastDamageInfo == nullptr ||
						getDamageTypeFlag( modData->m_damageFXTypes, lastDamageInfo->in.m_damageType ) )
				{

					pos = getLocalEffectPos( &modData->m_fxList[ newState ][ i ].locInfo, draw );
					getObject()->convertBonePosToWorldPos( &pos, nullptr, &pos, nullptr );
					FXList::doFXPos( modData->m_fxList[ newState ][ i ].fx, &pos );

				}

			}

			// do any object creation list for our new state
			if( damageSource && modData->m_OCL[ newState ][ i ].ocl )
			{

				if( lastDamageInfo == nullptr ||
						getDamageTypeFlag( modData->m_damageOCLTypes, lastDamageInfo->in.m_damageType ) )
				{

					pos = getLocalEffectPos( &modData->m_OCL[ newState ][ i ].locInfo, draw );
					getObject()->convertBonePosToWorldPos( &pos, nullptr, &pos, nullptr );
					ObjectCreationList::create( modData->m_OCL[ newState ][ i ].ocl,
																			getObject(), &pos, damageSource->getPosition(), INVALID_ANGLE );

				}

			}

			// get the template of the system to create
			pSystemT = modData->m_particleSystem[ newState ][ i ].particleSysTemplate;
			if( pSystemT )
			{

				if( lastDamageInfo == nullptr ||
						getDamageTypeFlag( modData->m_damageParticleTypes, lastDamageInfo->in.m_damageType ) )
				{

					// create a new particle system based on the template provided
					ParticleSystem* pSystem = TheParticleSystemManager->createParticleSystem( pSystemT );
					if( pSystem )
					{

						// get the what is the position we're going to played the effect at
						pos = getLocalEffectPos( &modData->m_particleSystem[ newState ][ i ].locInfo, draw );

						//
						// set position on system given any bone position provided, the bone position is
						// local to the object and that's what we want for the particle system ... the
						// transformation into world space using the object position is taken care of in
						// the particle system attachToObject method
						//
						pSystem->setPosition( &pos );

						// attach to object
						pSystem->attachToObject( getObject() );

						// save the id of this particle system so we can remove it later if it still exists
						m_particleSystemID[ newState ][ i ] = pSystem->getSystemID();

					}

				}

			}

		}

	}

}

// ------------------------------------------------------------------------------------------------
/** CRC */
// ------------------------------------------------------------------------------------------------
void TransitionDamageFX::crc( Xfer *xfer )
{

	// extend base class
	DamageModule::crc( xfer );

}

// ------------------------------------------------------------------------------------------------
/** Xfer method
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void TransitionDamageFX::xfer( Xfer *xfer )
{

	// version
	XferVersion currentVersion = 1;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	// extend base class
	DamageModule::xfer( xfer );

	// particle systems ids
	xfer->xferUser( m_particleSystemID, sizeof( ParticleSystemID ) * BODYDAMAGETYPE_COUNT * DAMAGE_MODULE_MAX_FX );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void TransitionDamageFX::loadPostProcess()
{

	// extend base class
	DamageModule::loadPostProcess();

}
```
