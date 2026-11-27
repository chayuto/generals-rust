//! Target finding: `AI::findClosestEnemy`, `findClosestAlly`, `findClosestRepulsor` and
//! `getAdjustedVisionRangeForObject`, with the partition filters they use.
//!
//! Source: `GameLogic/AI/AI.cpp`, the `PartitionFilter*::allow` functions of
//! `GameLogic/Object/PartitionManager.cpp` and `AttackPriorityInfo::getPriority`
//! of `GameLogic/ScriptEngine/ScriptEngine.cpp`. Part of PKT-421 (`AI`, no packet
//! document yet), slice S12 of `docs/AI_PORT_PLAN.md`; the rest of the class is
//! not here.
//! Golden test: `tests/golden_s12_targeting.rs`
//! (harness `tests/cpp_harness/gen_s12_targeting.cpp`).
//!
//! What these functions ask of code that is not ported goes through
//! [`TargetingSeams`].
//!
//! [`AiData`] is the whole of `TAiData` with `AI::parseAiDataDefinition` and
//! `TAiData::crc`, checked by `tests/golden_game_data_load.rs`
//! (harness `tests/cpp_harness/gen_game_data_load.cpp`).

use std::collections::BTreeMap;

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniLoadType, IniReader, LOGICFRAMES_PER_SECOND, SEPS_DEFAULT};
use zh_common::math::{gm_sqrtf, Coord2D, Coord3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError};

use crate::arena::ObjectSlot;
use crate::modules::module::CommandSourceType;
use crate::object::weapon_fire::compute_bonus;
use crate::object::weapon_set::{CanAttackResult, WeaponSlotType};
use crate::rules::kind_of::{kindof_mask_fs, kindof_mask_none, KindOfType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::science::{ScienceStore, ScienceType, SCIENCE_INVALID};
use crate::sim::{SimView, PRIVATE_STATUS_OFF_MAP};
use crate::spatial::partition_query::DistanceCalculationType;
use crate::spatial::sides_list::BuildListInfo;

/// `AI::findClosestEnemy` qualifiers.
pub const CAN_SEE: UnsignedInt = 1 << 0;
pub const CAN_ATTACK: UnsignedInt = 1 << 1;
pub const IGNORE_INSIGNIFICANT_BUILDINGS: UnsignedInt = 1 << 2;
pub const ATTACK_BUILDINGS: UnsignedInt = 1 << 3;
pub const WITHIN_ATTACK_RANGE: UnsignedInt = 1 << 4;
pub const UNFOGGED: UnsignedInt = 1 << 5;

/// `getAdjustedVisionRangeForObject` factors.
pub const AI_VISIONFACTOR_OWNERTYPE: Int = 0x01;
pub const AI_VISIONFACTOR_MOOD: Int = 0x02;
pub const AI_VISIONFACTOR_GUARDINNER: Int = 0x04;

/// `AbleToAttackType::ATTACK_NEW_TARGET`.
pub const ATTACK_NEW_TARGET: UnsignedInt = 0;

/// `MoodMatrixParameters`.
pub const MM_CONTROLLER_PLAYER: UnsignedInt = 0x0000_0001;
pub const MM_CONTROLLER_AI: UnsignedInt = 0x0000_0002;
pub const MM_UNIT_TYPE_NON_TURRETED: UnsignedInt = 0x0000_0010;
pub const MM_UNIT_TYPE_TURRETED: UnsignedInt = 0x0000_0020;
pub const MM_UNIT_TYPE_AIR: UnsignedInt = 0x0000_0040;
pub const MM_MOOD_SLEEP: UnsignedInt = 0x0000_0100;
pub const MM_MOOD_PASSIVE: UnsignedInt = 0x0000_0200;
pub const MM_MOOD_NORMAL: UnsignedInt = 0x0000_0400;
pub const MM_MOOD_ALERT: UnsignedInt = 0x0000_0800;
pub const MM_MOOD_AGGRESSIVE: UnsignedInt = 0x0000_1000;
pub const MM_MOOD_BITMASK: UnsignedInt =
    MM_MOOD_SLEEP | MM_MOOD_PASSIVE | MM_MOOD_NORMAL | MM_MOOD_ALERT | MM_MOOD_AGGRESSIVE;

const WWMATH_EPSILON: Real = 0.0001;

/// `MAX_AI_UPGRADES`.
pub const MAX_AI_UPGRADES: usize = 20;

/// `AISideInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct AiSideInfo {
    pub side: String,
    pub easy: Int,
    pub normal: Int,
    pub hard: Int,
    /// `m_skillSet1`..`m_skillSet5`.
    pub skill_sets: [Vec<ScienceType>; 5],
    pub base_defense_structure1: String,
}

impl AiSideInfo {
    fn new(side: &str) -> Self {
        Self {
            side: side.to_owned(),
            easy: 0,
            normal: 1,
            hard: 2,
            skill_sets: Default::default(),
            base_defense_structure1: String::new(),
        }
    }
}

/// `AISideBuildList`.
#[derive(Debug, Clone, PartialEq)]
pub struct AiSideBuildList {
    pub side: String,
    pub build_list: Vec<BuildListInfo>,
}

/// `TAiData`, with the defaults of its constructor.
#[derive(Debug, Clone, PartialEq)]
pub struct AiData {
    /// `StructureSeconds`.
    pub structure_seconds: Real,
    /// `TeamSeconds`.
    pub team_seconds: Real,
    /// `Wealthy`.
    pub resources_wealthy: Int,
    /// `Poor`.
    pub resources_poor: Int,
    /// `ForceIdleMSEC`.
    pub force_idle_frames_count: UnsignedInt,
    /// `StructuresWealthyRate`.
    pub structures_wealthy_mod: Real,
    /// `TeamsWealthyRate`.
    pub team_wealthy_mod: Real,
    /// `StructuresPoorRate`.
    pub structures_poor_mod: Real,
    /// `TeamsPoorRate`.
    pub team_poor_mod: Real,
    /// `TeamResourcesToStart`.
    pub team_resources_to_build: Real,
    /// `GuardInnerModifierAI`.
    pub guard_inner_modifier_ai: Real,
    /// `GuardOuterModifierAI`.
    pub guard_outer_modifier_ai: Real,
    /// `GuardInnerModifierHuman`.
    pub guard_inner_modifier_human: Real,
    /// `GuardOuterModifierHuman`.
    pub guard_outer_modifier_human: Real,
    /// `GuardChaseUnitsDuration`.
    pub guard_chase_unit_frames: UnsignedInt,
    /// `GuardEnemyScanRate`.
    pub guard_enemy_scan_rate: UnsignedInt,
    /// `GuardEnemyReturnScanRate`.
    pub guard_enemy_return_scan_rate: UnsignedInt,
    /// `SkirmishGroupFudgeDistance`.
    pub skirmish_group_fudge_value: Real,
    /// `RepulsedDistance`.
    pub repulsed_distance: Real,
    /// `EnableRepulsors`.
    pub enable_repulsors: Bool,
    /// `AlertRangeModifier`.
    pub alert_range_modifier: Real,
    /// `AggressiveRangeModifier`.
    pub aggressive_range_modifier: Real,
    /// `ForceSkirmishAI`.
    pub force_skirmish_ai: Bool,
    /// `RotateSkirmishBases`.
    pub rotate_skirmish_bases: Bool,
    /// `AttackUsesLineOfSight`.
    pub attack_uses_line_of_sight: Bool,
    /// `AttackIgnoreInsignificantBuildings`.
    pub attack_ignore_insignificant_buildings: Bool,
    /// `AttackPriorityDistanceModifier`.
    pub attack_priority_distance_modifier: Real,
    /// `MaxRecruitRadius`.
    pub max_recruit_distance: Real,
    /// `SkirmishBaseDefenseExtraDistance`.
    pub skirmish_base_defense_extra_distance: Real,
    /// `WallHeight`.
    pub wall_height: Real,
    /// `MinInfantryForGroup`.
    pub min_infantry_for_group: Int,
    /// `MinVehiclesForGroup`.
    pub min_vehicles_for_group: Int,
    /// `MinDistanceForGroup`.
    pub min_distance_for_group: Real,
    /// `DistanceRequiresGroup`.
    pub distance_requires_group: Real,
    /// `MinClumpDensity`.
    pub min_clump_density: Real,
    /// `InfantryPathfindDiameter`.
    pub infantry_pathfind_diameter: Int,
    /// `VehiclePathfindDiameter`.
    pub vehicle_pathfind_diameter: Int,
    /// `RebuildDelayTimeSeconds`.
    pub rebuild_delay_seconds: Int,
    /// `SupplyCenterSafeRadius`.
    pub supply_center_safe_radius: Real,
    /// `AIDozerBoredRadiusModifier`.
    pub ai_dozer_bored_radius_modifier: Real,
    /// `AICrushesInfantry`.
    pub ai_crushes_infantry: Bool,
    /// `MaxRetaliationDistance`.
    pub max_retaliate_distance: Real,
    /// `RetaliationFriendsRadius`.
    pub retaliate_friends_radius: Real,
    /// `m_sideInfo`, from the head of the list.
    pub side_info: Vec<AiSideInfo>,
    /// `m_sideBuildLists`, from the head of the list.
    pub side_build_lists: Vec<AiSideBuildList>,
}

impl Default for AiData {
    fn default() -> Self {
        Self {
            structure_seconds: 0.0,
            team_seconds: 0.0,
            resources_wealthy: 0,
            resources_poor: 0,
            force_idle_frames_count: 1,
            structures_wealthy_mod: 0.0,
            team_wealthy_mod: 0.0,
            structures_poor_mod: 0.0,
            team_poor_mod: 0.0,
            team_resources_to_build: 0.0,
            guard_inner_modifier_ai: 0.0,
            guard_outer_modifier_ai: 0.0,
            guard_inner_modifier_human: 0.0,
            guard_outer_modifier_human: 0.0,
            guard_chase_unit_frames: 0,
            guard_enemy_scan_rate: LOGICFRAMES_PER_SECOND as UnsignedInt / 2,
            guard_enemy_return_scan_rate: LOGICFRAMES_PER_SECOND as UnsignedInt,
            skirmish_group_fudge_value: 0.0,
            repulsed_distance: 0.0,
            enable_repulsors: FALSE,
            alert_range_modifier: 0.0,
            aggressive_range_modifier: 0.0,
            force_skirmish_ai: FALSE,
            rotate_skirmish_bases: FALSE,
            attack_uses_line_of_sight: TRUE,
            attack_ignore_insignificant_buildings: FALSE,
            attack_priority_distance_modifier: 0.0,
            max_recruit_distance: 0.0,
            skirmish_base_defense_extra_distance: 0.0,
            wall_height: 0.0,
            min_infantry_for_group: 3,
            min_vehicles_for_group: 4,
            min_distance_for_group: 100.0,
            distance_requires_group: 0.0,
            min_clump_density: 0.5,
            infantry_pathfind_diameter: 6,
            vehicle_pathfind_diameter: 6,
            rebuild_delay_seconds: 10,
            supply_center_safe_radius: 250.0,
            ai_dozer_bored_radius_modifier: 2.0,
            ai_crushes_infantry: TRUE,
            max_retaliate_distance: 210.0,
            retaliate_friends_radius: 120.0,
            side_info: Vec::new(),
            side_build_lists: Vec::new(),
        }
    }
}

impl AiData {
    /// The body of `AI::parseAiDataDefinition`, from the line after `AIData`. With
    /// `CreateOverrides` the C++ pushes a copy first; no chain of the replaced data is
    /// kept here. Keys outside the tables are appended to `ignored`.
    pub fn parse_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        if ini.get_load_type() == IniLoadType::CreateOverrides {
            self.reverse_lists_for_override();
        }
        ini.read_block_fields(|key, ini| {
            match key {
                "SideInfo" => self.parse_side_info(ini, science_store, name_keys, ignored)?,
                "SkirmishBuildList" => self.parse_skirmish_build_list(ini, ignored)?,
                _ => {
                    if !self.parse_field(key, ini)? {
                        ignored.push(key.to_owned());
                    }
                }
            }
            Ok(())
        })
    }

    /// `AI::newOverride` copies both lists by pushing at the head, so their order
    /// turns around.
    fn reverse_lists_for_override(&mut self) {
        self.side_info.reverse();
        self.side_build_lists.reverse();
    }

    fn parse_field(&mut self, key: &str, ini: &mut IniReader<'_>) -> Result<bool, IniError> {
        match key {
            "StructureSeconds" => ini.parse_real(&mut self.structure_seconds)?,
            "TeamSeconds" => ini.parse_real(&mut self.team_seconds)?,
            "Wealthy" => ini.parse_int(&mut self.resources_wealthy)?,
            "Poor" => ini.parse_int(&mut self.resources_poor)?,
            "ForceIdleMSEC" => ini.parse_duration_unsigned_int(&mut self.force_idle_frames_count)?,
            "StructuresWealthyRate" => ini.parse_real(&mut self.structures_wealthy_mod)?,
            "TeamsWealthyRate" => ini.parse_real(&mut self.team_wealthy_mod)?,
            "StructuresPoorRate" => ini.parse_real(&mut self.structures_poor_mod)?,
            "TeamsPoorRate" => ini.parse_real(&mut self.team_poor_mod)?,
            "TeamResourcesToStart" => ini.parse_real(&mut self.team_resources_to_build)?,
            "GuardInnerModifierAI" => ini.parse_real(&mut self.guard_inner_modifier_ai)?,
            "GuardOuterModifierAI" => ini.parse_real(&mut self.guard_outer_modifier_ai)?,
            "GuardInnerModifierHuman" => ini.parse_real(&mut self.guard_inner_modifier_human)?,
            "GuardOuterModifierHuman" => ini.parse_real(&mut self.guard_outer_modifier_human)?,
            "GuardChaseUnitsDuration" => ini.parse_duration_unsigned_int(&mut self.guard_chase_unit_frames)?,
            "GuardEnemyScanRate" => ini.parse_duration_unsigned_int(&mut self.guard_enemy_scan_rate)?,
            "GuardEnemyReturnScanRate" => ini.parse_duration_unsigned_int(&mut self.guard_enemy_return_scan_rate)?,
            "SkirmishGroupFudgeDistance" => ini.parse_real(&mut self.skirmish_group_fudge_value)?,
            "RepulsedDistance" => ini.parse_real(&mut self.repulsed_distance)?,
            "EnableRepulsors" => ini.parse_bool(&mut self.enable_repulsors)?,
            "AlertRangeModifier" => ini.parse_real(&mut self.alert_range_modifier)?,
            "AggressiveRangeModifier" => ini.parse_real(&mut self.aggressive_range_modifier)?,
            "ForceSkirmishAI" => ini.parse_bool(&mut self.force_skirmish_ai)?,
            "RotateSkirmishBases" => ini.parse_bool(&mut self.rotate_skirmish_bases)?,
            "AttackUsesLineOfSight" => ini.parse_bool(&mut self.attack_uses_line_of_sight)?,
            "AttackIgnoreInsignificantBuildings" => ini.parse_bool(&mut self.attack_ignore_insignificant_buildings)?,
            "AttackPriorityDistanceModifier" => ini.parse_real(&mut self.attack_priority_distance_modifier)?,
            "MaxRecruitRadius" => ini.parse_real(&mut self.max_recruit_distance)?,
            "SkirmishBaseDefenseExtraDistance" => ini.parse_real(&mut self.skirmish_base_defense_extra_distance)?,
            "WallHeight" => ini.parse_real(&mut self.wall_height)?,
            "MinInfantryForGroup" => ini.parse_int(&mut self.min_infantry_for_group)?,
            "MinVehiclesForGroup" => ini.parse_int(&mut self.min_vehicles_for_group)?,
            "MinDistanceForGroup" => ini.parse_real(&mut self.min_distance_for_group)?,
            "DistanceRequiresGroup" => ini.parse_real(&mut self.distance_requires_group)?,
            "MinClumpDensity" => ini.parse_real(&mut self.min_clump_density)?,
            "InfantryPathfindDiameter" => ini.parse_int(&mut self.infantry_pathfind_diameter)?,
            "VehiclePathfindDiameter" => ini.parse_int(&mut self.vehicle_pathfind_diameter)?,
            "RebuildDelayTimeSeconds" => ini.parse_int(&mut self.rebuild_delay_seconds)?,
            "SupplyCenterSafeRadius" => ini.parse_real(&mut self.supply_center_safe_radius)?,
            "AIDozerBoredRadiusModifier" => ini.parse_real(&mut self.ai_dozer_bored_radius_modifier)?,
            "AICrushesInfantry" => ini.parse_bool(&mut self.ai_crushes_infantry)?,
            "MaxRetaliationDistance" => ini.parse_real(&mut self.max_retaliate_distance)?,
            "RetaliationFriendsRadius" => ini.parse_real(&mut self.retaliate_friends_radius)?,
            _ => return Ok(false),
        }
        Ok(true)
    }

    /// `AI::parseSideInfo`; a side seen before is parsed into again.
    fn parse_side_info(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let side = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let at = match self.side_info.iter().position(|s| s.side == side) {
            Some(at) => at,
            None => {
                self.side_info.insert(0, AiSideInfo::new(&side));
                0
            }
        };
        let info = &mut self.side_info[at];
        ini.read_block_fields(|key, ini| {
            let set = match key {
                "ResourceGatherersEasy" => return ini.parse_int(&mut info.easy),
                "ResourceGatherersNormal" => return ini.parse_int(&mut info.normal),
                "ResourceGatherersHard" => return ini.parse_int(&mut info.hard),
                "BaseDefenseStructure1" => {
                    return ini.parse_ascii_string(&mut info.base_defense_structure1)
                }
                "SkillSet1" => 0,
                "SkillSet2" => 1,
                "SkillSet3" => 2,
                "SkillSet4" => 3,
                "SkillSet5" => 4,
                _ => {
                    ignored.push(key.to_owned());
                    return Ok(());
                }
            };
            parse_skill_set(&mut info.skill_sets[set], ini, science_store, name_keys, ignored)
        })
    }

    /// `AI::parseSkirmishBuildList` and `TAiData::addFactionBuildList`.
    fn parse_skirmish_build_list(
        &mut self,
        ini: &mut IniReader<'_>,
        ignored: &mut Vec<String>,
    ) -> Result<(), IniError> {
        let side = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let mut build_list = Vec::new();
        ini.read_block_fields(|key, ini| {
            if key == "Structure" {
                build_list.push(parse_structure(ini, ignored)?);
            } else {
                ignored.push(key.to_owned());
            }
            Ok(())
        })?;
        match self.side_build_lists.iter_mut().find(|l| l.side == side) {
            Some(list) => list.build_list = build_list,
            None => self
                .side_build_lists
                .insert(0, AiSideBuildList { side, build_list }),
        }
        Ok(())
    }
}

/// `AI::parseSkillSet`: only a science with a purchase cost takes a place.
fn parse_skill_set(
    skills: &mut Vec<ScienceType>,
    ini: &mut IniReader<'_>,
    science_store: &ScienceStore,
    name_keys: &mut NameKeyGenerator,
    ignored: &mut Vec<String>,
) -> Result<(), IniError> {
    skills.clear();
    ini.read_block_fields(|key, ini| {
        if key != "Science" {
            ignored.push(key.to_owned());
        } else if skills.len() < MAX_AI_UPGRADES {
            let token = ini.next_token(SEPS_DEFAULT)?;
            let science = science_store.friend_lookup_science(token, name_keys)?;
            if science != SCIENCE_INVALID && science_store.get_science_purchase_cost(science) != 0 {
                skills.push(science);
            }
        }
        Ok(())
    })
}

/// `BuildListInfo::parseStructure`.
fn parse_structure(
    ini: &mut IniReader<'_>,
    ignored: &mut Vec<String>,
) -> Result<BuildListInfo, IniError> {
    let mut info = BuildListInfo::new();
    info.template_name = ini.next_token(SEPS_DEFAULT)?.to_owned();
    ini.read_block_fields(|key, ini| {
        match key {
            "Name" => ini.parse_ascii_string(&mut info.building_name)?,
            "Location" => {
                let mut at = Coord2D { x: 0.0, y: 0.0 };
                ini.parse_coord2d(&mut at)?;
                info.location.x = at.x;
                info.location.y = at.y;
            }
            "Rebuilds" => {
                let mut rebuilds = 0;
                ini.parse_int(&mut rebuilds)?;
                info.num_rebuilds = rebuilds as UnsignedInt;
            }
            "Angle" => ini.parse_angle_real(&mut info.angle)?,
            "InitiallyBuilt" => ini.parse_bool(&mut info.is_initially_built)?,
            "RallyPointOffset" => ini.parse_coord2d(&mut info.rally_point_offset)?,
            "AutomaticallyBuild" => ini.parse_bool(&mut info.automatically_build)?,
            _ => ignored.push(key.to_owned()),
        }
        Ok(())
    })?;
    Ok(info)
}

impl Snapshot for AiData {
    /// `TAiData::crc`.
    fn crc(&self, xfer: &mut dyn Xfer) {
        let mut v = self.structure_seconds;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.team_seconds;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.resources_wealthy;
        let _ = xfer.xfer_int(&mut v);
        let mut v = self.resources_poor;
        let _ = xfer.xfer_int(&mut v);
        let mut v = self.force_idle_frames_count;
        let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.structures_wealthy_mod;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.team_wealthy_mod;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.structures_poor_mod;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.team_poor_mod;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.team_resources_to_build;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.guard_inner_modifier_ai;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.guard_outer_modifier_ai;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.guard_inner_modifier_human;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.guard_outer_modifier_human;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.guard_chase_unit_frames;
        let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.guard_enemy_scan_rate;
        let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.guard_enemy_return_scan_rate;
        let _ = xfer.xfer_unsigned_int(&mut v);
        let mut v = self.alert_range_modifier;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.aggressive_range_modifier;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.attack_priority_distance_modifier;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.max_recruit_distance;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.skirmish_base_defense_extra_distance;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.repulsed_distance;
        let _ = xfer.xfer_real(&mut v);
        let mut v = self.enable_repulsors;
        let _ = xfer.xfer_bool(&mut v);
    }

    /// `TAiData::xfer`: a version byte only.
    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1;
        xfer.xfer_version(&mut version, 1)
    }
}

/// An attack priority set other than the default one. The C++ keys the map by
/// the final override of the template; a template name stands for it here.
#[derive(Debug, Clone, PartialEq)]
pub struct AttackPriorityInfo {
    pub name: String,
    pub default_priority: Int,
    pub priorities: BTreeMap<String, Int>,
}

impl AttackPriorityInfo {
    /// `AttackPriorityInfo::getPriority`. `None` is a null template.
    pub fn get_priority(&self, template_name: Option<&str>) -> Int {
        template_name
            .and_then(|name| self.priorities.get(name))
            .copied()
            .unwrap_or(self.default_priority)
    }

    /// `getPriority` of the template of `object`.
    pub fn priority_of(&self, sim: &dyn SimView, object: ObjectSlot) -> Int {
        let name = sim.object(object).and_then(|o| {
            o.thing
                .template_override()
                .get_final(&sim.rules().thing_factory)
                .map(|t| t.get_name())
        });
        self.get_priority(name)
    }
}

/// The answers targeting needs from code that is not ported. A test answers them
/// from a tape of what the reference did.
pub trait TargetingSeams {
    /// `Object::getAbleToAttackSpecificObject(ATTACK_NEW_TARGET, other, CMD_FROM_AI)`.
    fn able_to_attack_specific_object(
        &mut self,
        sim: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> CanAttackResult {
        sim.able_to_attack_specific_object(me, ATTACK_NEW_TARGET, other, CommandSourceType::FromAi)
    }

    /// `Weapon::isWithinAttackRange(me, other)` of any weapon `me` holds
    /// (`PartitionFilterWithinAttackRange`).
    fn within_attack_range(&mut self, sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot)
        -> Bool;

    /// `Object::getLargestWeaponRange`, ported; see [`largest_weapon_range`].
    fn largest_weapon_range(&mut self, sim: &dyn SimView, me: ObjectSlot) -> Real {
        largest_weapon_range(sim, me)
    }

    /// `Pathfinder::isViewBlockedByObstacle`.
    fn view_blocked_by_obstacle(
        &mut self,
        sim: &dyn SimView,
        me: ObjectSlot,
        other: ObjectSlot,
    ) -> Bool;

    /// `other->getShroudedStatus(player_index) == OBJECTSHROUD_CLEAR`.
    fn is_clear_of_fog(&mut self, sim: &dyn SimView, other: ObjectSlot, player_index: Int) -> Bool;

    /// The target of the team of `me`: `Team::getTeamTargetObject` if the team
    /// prototype has `m_attackCommonTarget`, `None` otherwise.
    fn team_target_object(&mut self, sim: &dyn SimView, me: ObjectSlot) -> Option<ObjectSlot>;
}

/// `Object::getLargestWeaponRange`: the longest attack range of the weapons held,
/// under their current bonuses; -1 without a weapon.
pub fn largest_weapon_range(sim: &dyn SimView, object: ObjectSlot) -> Real {
    let mut largest = -1.0;
    let Some(record) = sim.object(object) else {
        return largest;
    };
    for slot in [
        WeaponSlotType::Primary,
        WeaponSlotType::Secondary,
        WeaponSlotType::Tertiary,
    ] {
        let Some(weapon) = record.weapon_set.get_weapon_in_weapon_slot(slot) else {
            continue;
        };
        let bonus = compute_bonus(weapon, sim, object, 0);
        let range = weapon.get_template().get_attack_range(&bonus);
        if range > largest {
            largest = range;
        }
    }
    largest
}

fn has_status(sim: &dyn SimView, object: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(object)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn is_kind(sim: &dyn SimView, object: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(object, kind) != FALSE
}

fn is_dead(sim: &dyn SimView, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn is_off_map(sim: &dyn SimView, object: ObjectSlot) -> bool {
    sim.object(object)
        .is_some_and(|o| o.private_status & PRIVATE_STATUS_OFF_MAP != 0)
}

fn is_enemy(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    sim.player_relationship(me, other) == Relationship::Enemies
}

fn able_to_attack(sim: &dyn SimView, object: ObjectSlot) -> bool {
    sim.object_is_able_to_attack(object) != FALSE
}

/// `myPlayer->getRelationship(otherPlayer->getDefaultTeam()) == ENEMIES`, where
/// `myPlayer` controls `me`.
fn player_is_enemy_of_player(sim: &dyn SimView, me: ObjectSlot, other_player: Int) -> bool {
    sim.object(me).is_some_and(|o| {
        sim.team_relationship_to_player(o.team_id, other_player) == Relationship::Enemies
    })
}

/// `PartitionFilterLiveMapEnemies`.
fn allow_live_map_enemy(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    !is_dead(sim, other)
        && is_off_map(sim, other) == is_off_map(sim, me)
        && is_enemy(sim, me, other)
}

/// `PartitionFilterRejectBuildings`.
fn allow_reject_buildings(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    let Some((my_team, other_team)) = sim
        .object(me)
        .zip(sim.object(other))
        .map(|(a, b)| (a.team_id, b.team_id))
    else {
        return false;
    };
    if !is_kind(sim, other, KindOfType::Structure) {
        return true;
    }
    let apparent = if sim.object_has_contain(other) != FALSE {
        sim.contain_apparent_controlling_player(other, sim.player_index(my_team))
    } else {
        None
    };
    let other_player = apparent.unwrap_or_else(|| sim.player_index(other_team));
    if !player_is_enemy_of_player(sim, me, other_player) {
        return false;
    }
    // A computer player acquires every enemy building.
    if sim.player_is_human(my_team) == FALSE {
        return true;
    }
    is_kind(sim, other, KindOfType::FsBaseDefense)
        || (sim.object_has_contain(other) != FALSE && able_to_attack(sim, other))
}

/// `PartitionFilterInsignificantBuildings(true, false)`: non buildings pass,
/// insignificant buildings do not.
fn allow_insignificant_buildings(sim: &dyn SimView, other: ObjectSlot) -> bool {
    if !is_kind(sim, other, KindOfType::Structure) {
        return true;
    }
    let faction =
        sim.object_is_kind_of_multi(other, &kindof_mask_none(), &kindof_mask_fs()) == FALSE;
    if !faction
        && sim.object_has_contain(other) != FALSE
        && (sim.contain_is_garrisonable(other) == FALSE || sim.contain_count(other) == 0)
    {
        return false;
    }
    true
}

/// `PartitionFilterStealthedAndUndetected(me, false)`: `true` keeps `other`.
fn allow_not_stealthed(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    let stealthed = has_status(sim, other, ObjectStatusType::Stealthed);
    let detected = has_status(sim, other, ObjectStatusType::Detected);
    if stealthed && !detected {
        if !is_kind(sim, other, KindOfType::Disguiser) {
            return false;
        }
        if has_status(sim, other, ObjectStatusType::Disguised) {
            if let Some(update) = sim.stealth_params(other) {
                if update.disguised_template.is_some() && update.disguised_player_index >= 0 {
                    // Disguised as an enemy of ours, it counts as not stealthed.
                    return player_is_enemy_of_player(sim, me, update.disguised_player_index);
                }
            }
        }
    } else if sim.object_has_contain(other) != FALSE {
        let items = sim.contain_items(other);
        if let Some(&first) = items.first() {
            let observer = sim.object(me).map_or(0, |o| sim.player_index(o.team_id));
            let apparent = sim.contain_apparent_controlling_player(other, observer);
            if sim.contain_stealth_units_contained(other) == sim.contain_count(other) {
                let undetected = sim
                    .find_object(first)
                    .is_some_and(|m| !has_status(sim, m, ObjectStatusType::Detected));
                if undetected && apparent.is_some_and(|p| player_is_enemy_of_player(sim, me, p)) {
                    return false;
                }
            }
        }
    }
    true
}

/// `PartitionFilterLineOfSight`.
fn allow_line_of_sight(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    me: ObjectSlot,
    other: ObjectSlot,
) -> bool {
    sim.is_clear_line_of_sight(me, other) != FALSE
        && seams.view_blocked_by_obstacle(sim, me, other) == FALSE
}

/// The highest priority among `curr` and the objects inside `enemy`.
fn priority_with_contained(
    sim: &dyn SimView,
    info: &AttackPriorityInfo,
    enemy: ObjectSlot,
    current: Int,
) -> Int {
    if sim.object_has_contain(enemy) == FALSE {
        return current;
    }
    sim.contain_items(enemy)
        .into_iter()
        .filter_map(|id: ObjectId| sim.find_object(id))
        .fold(current, |best, item| best.max(info.priority_of(sim, item)))
}

/// `AI::findClosestEnemy`. `info` is `None` for a null and for the default
/// attack priority set, which behave alike.
#[allow(clippy::too_many_arguments)]
pub fn find_closest_enemy(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    ai_data: &AiData,
    me: ObjectSlot,
    range: Real,
    qualifiers: UnsignedInt,
    info: Option<&AttackPriorityInfo>,
    mut optional_filter: Option<&mut dyn FnMut(ObjectSlot) -> bool>,
) -> Option<ObjectSlot> {
    if qualifiers & CAN_ATTACK != 0 && !able_to_attack(sim, me) {
        return None;
    }
    let player = sim.object(me).map(|o| sim.player_index(o.team_id))?;

    let mut filter = |other: ObjectSlot| -> bool {
        if !allow_live_map_enemy(sim, me, other) {
            return false;
        }
        if qualifiers & ATTACK_BUILDINGS == 0 && !allow_reject_buildings(sim, me, other) {
            return false;
        }
        if qualifiers & WITHIN_ATTACK_RANGE != 0
            && seams.within_attack_range(sim, me, other) == FALSE
        {
            return false;
        }
        if qualifiers & CAN_SEE != 0 && !allow_line_of_sight(sim, seams, me, other) {
            return false;
        }
        if qualifiers & CAN_ATTACK != 0
            && !matches!(
                seams.able_to_attack_specific_object(sim, me, other),
                CanAttackResult::Possible | CanAttackResult::PossibleAfterMoving
            )
        {
            return false;
        }
        if qualifiers & UNFOGGED != 0 && seams.is_clear_of_fog(sim, other, player) == FALSE {
            return false;
        }
        if qualifiers & IGNORE_INSIGNIFICANT_BUILDINGS != 0
            && !allow_insignificant_buildings(sim, other)
        {
            return false;
        }
        if !allow_not_stealthed(sim, me, other) {
            return false;
        }
        optional_filter.as_mut().is_none_or(|f| f(other))
    };

    let rule = DistanceCalculationType::FromBoundingSphere2d;
    let Some(info) = info else {
        return sim.closest_object_from(me, range, rule, &mut filter);
    };

    let mut best = None;
    let mut effective_priority: Int = 0;
    let mut actual_priority: Int = 0;
    for enemy in sim.objects_from_near_to_far(me, range, rule, &mut filter) {
        let mut current = info.priority_of(sim, enemy);
        if current == 0 {
            continue;
        }
        // A container is worth the best of what it holds.
        current = priority_with_contained(sim, info, enemy, current);

        let dist_sqr = sim.partition_distance_squared(me, Some(enemy), &Coord3D::default(), rule);
        let dist = gm_sqrtf(dist_sqr);
        let divisor = if ai_data.attack_priority_distance_modifier < WWMATH_EPSILON {
            WWMATH_EPSILON
        } else {
            ai_data.attack_priority_distance_modifier
        };
        let modifier = (dist / divisor) as Int;
        let modified = current.wrapping_sub(modifier).max(1);
        if modified > effective_priority {
            effective_priority = modified;
            actual_priority = current;
            best = Some(enemy);
        }
        if modified == effective_priority && current > actual_priority {
            effective_priority = modified;
            actual_priority = current;
            best = Some(enemy);
        }
    }
    best
}

/// `AI::findClosestAlly`.
pub fn find_closest_ally(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    me: ObjectSlot,
    range: Real,
    qualifiers: UnsignedInt,
) -> Option<ObjectSlot> {
    let mut filter = |other: ObjectSlot| -> bool {
        allow_reject_buildings(sim, me, other)
            && sim.player_relationship(me, other) == Relationship::Allies
            && !is_dead(sim, other)
            && is_off_map(sim, other) == is_off_map(sim, me)
            && (qualifiers & CAN_SEE == 0 || allow_line_of_sight(sim, seams, me, other))
    };
    sim.closest_object_from(
        me,
        range,
        DistanceCalculationType::FromBoundingSphere2d,
        &mut filter,
    )
}

/// `PartitionFilterRepulsor`.
fn allow_repulsor(sim: &dyn SimView, me: ObjectSlot, other: ObjectSlot) -> bool {
    if other == me {
        return false;
    }
    if has_status(sim, other, ObjectStatusType::Repulsor) {
        return true;
    }
    if is_dead(sim, other) || !is_enemy(sim, me, other) {
        return false;
    }
    if is_kind(sim, other, KindOfType::Structure) {
        return able_to_attack(sim, other);
    }
    !is_kind(sim, other, KindOfType::Inert) && able_to_attack(sim, other)
}

/// `AI::findClosestRepulsor`.
pub fn find_closest_repulsor(
    sim: &dyn SimView,
    ai_data: &AiData,
    me: ObjectSlot,
    range: Real,
) -> Option<ObjectSlot> {
    if ai_data.enable_repulsors == FALSE {
        return None;
    }
    let mut filter = |other: ObjectSlot| -> bool {
        // The second filter rejects what is stealthed and neither detected nor disguised.
        allow_repulsor(sim, me, other)
            && !(has_status(sim, other, ObjectStatusType::Stealthed)
                && !has_status(sim, other, ObjectStatusType::Detected)
                && !has_status(sim, other, ObjectStatusType::Disguised))
    };
    sim.closest_object_from(
        me,
        range,
        DistanceCalculationType::FromBoundingSphere2d,
        &mut filter,
    )
}

/// `AI::getAdjustedVisionRangeForObject`. `mood_matrix` is the
/// `getMoodMatrixValue` of the object's AI, `None` without an AI.
pub fn adjusted_vision_range(
    sim: &dyn SimView,
    seams: &mut dyn TargetingSeams,
    ai_data: &AiData,
    object: ObjectSlot,
    mood_matrix: Option<UnsignedInt>,
    factors: Int,
) -> Real {
    let mut range = sim.object_vision_range(object);
    let Some(mood) = mood_matrix else {
        return 0.0;
    };

    if factors & AI_VISIONFACTOR_OWNERTYPE != 0 {
        let inner = factors & AI_VISIONFACTOR_GUARDINNER != 0;
        range *= match (mood & MM_CONTROLLER_PLAYER != 0, inner) {
            (true, true) => ai_data.guard_inner_modifier_human,
            (true, false) => ai_data.guard_outer_modifier_human,
            (false, true) => ai_data.guard_inner_modifier_ai,
            (false, false) => ai_data.guard_outer_modifier_ai,
        };
    }

    if sim.object(object).is_some_and(|o| o.contained_by.is_some()) {
        return seams.largest_weapon_range(sim, object);
    }
    if factors & AI_VISIONFACTOR_MOOD != 0 && mood & MM_CONTROLLER_PLAYER == 0 {
        match mood & MM_MOOD_BITMASK {
            MM_MOOD_SLEEP => return 0.0,
            MM_MOOD_ALERT => range *= ai_data.alert_range_modifier,
            MM_MOOD_AGGRESSIVE => range *= ai_data.aggressive_range_modifier,
            _ => {}
        }
    }
    range
}
