//! `PKT-309`: `BattlePlanUpdateModuleData`, `BattlePlanUpdate` (`BattlePlanUpdate.h`,
//! `BattlePlanUpdate.cpp`) and the battle plan side of `Player` it drives
//! (`Player::changeBattlePlan` and the object bonus helpers).
//!
//! Packet: `packets/phase5/PKT-309_BattlePlanUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt341_350_309_support.rs`.
//!
//! Client work is not emitted: unpack/pack/idle/announcement sounds, the message label,
//! the radar event and the drawable's animation loop duration. `createVisionObject` has
//! no caller in the reference (its only call is commented out), so it is not ported.
//! Turret orders go to `AIUpdateInterface` ([`AiUpdateFacet`]), which is unported.
//!
//! [`AiUpdateFacet`]: crate::modules::module::AiUpdateFacet

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::WhichTurretType;
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, CommandSourceType, MaxHealthChangeType, ModuleData, SpecialPowerCaller,
    SpecialPowerUpdateFacet, UpdateFacet, MAX_HEALTH_CHANGE_NAMES,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::rules::combat_types::WeaponSetType;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `CommandOption` bits that pick a plan.
const OPTION_ONE: UnsignedInt = 0x0000_2000;
const OPTION_TWO: UnsignedInt = 0x0000_4000;
const OPTION_THREE: UnsignedInt = 0x0000_8000;

/// `BattlePlanStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum BattlePlanStatus {
    #[default]
    None = 0,
    Bombardment = 1,
    HoldTheLine = 2,
    SearchAndDestroy = 3,
}

impl BattlePlanStatus {
    fn from_i32(v: Int) -> Self {
        match v {
            1 => Self::Bombardment,
            2 => Self::HoldTheLine,
            3 => Self::SearchAndDestroy,
            _ => Self::None,
        }
    }
}

/// `TransitionStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum TransitionStatus {
    #[default]
    Idle = 0,
    Unpacking = 1,
    Active = 2,
    Packing = 3,
}

impl TransitionStatus {
    fn from_i32(v: Int) -> Self {
        match v {
            1 => Self::Unpacking,
            2 => Self::Active,
            3 => Self::Packing,
            _ => Self::Idle,
        }
    }
}

/// `BattlePlanBonusesData`.
#[derive(Debug, Clone, PartialEq)]
pub struct BattlePlanBonuses {
    pub armor_scalar: Real,
    pub bombardment: Int,
    pub search_and_destroy: Int,
    pub hold_the_line: Int,
    pub sight_range_scalar: Real,
    pub valid_kind_of: KindOfMaskType,
    pub invalid_kind_of: KindOfMaskType,
}

impl Default for BattlePlanBonuses {
    fn default() -> Self {
        Self {
            armor_scalar: 1.0,
            bombardment: 0,
            search_and_destroy: 0,
            hold_the_line: 0,
            sight_range_scalar: 1.0,
            valid_kind_of: KindOfMaskType::new(),
            invalid_kind_of: KindOfMaskType::new(),
        }
    }
}

/// `Player`'s battle plan members.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PlayerBattlePlans {
    pub bombardment_plans: Int,
    pub hold_the_line_plans: Int,
    pub search_and_destroy_plans: Int,
    /// `m_battlePlanBonuses`, allocated by the first change that adds or removes one.
    pub bonuses: Option<BattlePlanBonuses>,
}

impl PlayerBattlePlans {
    /// `getNumBattlePlansActive`.
    pub fn num_active(&self) -> Int {
        self.bombardment_plans + self.hold_the_line_plans + self.search_and_destroy_plans
    }
}

/// `localApplyBattlePlanBonusesToObject`.
pub(crate) fn apply_bonuses_to_object(sim: &mut dyn SimApi, obj: ObjectSlot, bonus: &BattlePlanBonuses) {
    let Some((projectile, producer)) = sim.object(obj).map(|o| {
        let projectile = o
            .thing
            .is_kind_of(&sim.rules().thing_factory, KindOfType::Projectile);
        (projectile != FALSE, o.producer_id)
    }) else {
        return;
    };
    let validate = if projectile {
        sim.find_object(producer)
    } else {
        Some(obj)
    };
    let qualifies = validate.and_then(|v| sim.object(v)).is_some_and(|o| {
        let factory = &sim.rules().thing_factory;
        o.thing.is_any_kind_of(factory, &bonus.valid_kind_of) != FALSE
            && o.thing.is_any_kind_of(factory, &bonus.invalid_kind_of) == FALSE
    });
    if !qualifies {
        return;
    }
    if !projectile {
        if bonus.armor_scalar != 1.0 {
            sim.apply_damage_scalar(obj, bonus.armor_scalar);
        }
        if bonus.sight_range_scalar != 1.0 {
            let vision = sim.object_vision_range(obj) * bonus.sight_range_scalar;
            sim.set_vision_range(obj, vision);
            let clearing = sim.object_shroud_clearing_range(obj) * bonus.sight_range_scalar;
            sim.set_shroud_clearing_range(obj, clearing);
        }
    }
    use WeaponBonusConditionType as W;
    for (count, condition) in [
        (bonus.bombardment, W::BattleplanBombardment),
        (bonus.hold_the_line, W::BattleplanHoldTheLine),
        (bonus.search_and_destroy, W::BattleplanSearchAndDestroy),
    ] {
        if count > 0 {
            sim.set_weapon_bonus_condition(obj, condition);
        } else {
            sim.clear_weapon_bonus_condition(obj, condition);
        }
    }
}

/// `Player::applyBattlePlanBonusesForPlayerObjects`.
fn apply_bonuses_to_player(sim: &mut dyn SimApi, team: TeamId, bonus: &BattlePlanBonuses) {
    let player = sim.player_index(team);
    let plans = sim.player_battle_plans_mut(player);
    let total = match &mut plans.bonuses {
        None => {
            plans.bonuses = Some(bonus.clone());
            bonus.clone()
        }
        Some(total) => {
            total.armor_scalar *= bonus.armor_scalar;
            total.sight_range_scalar *= bonus.sight_range_scalar;
            total.bombardment = (total.bombardment + bonus.bombardment).max(0);
            total.hold_the_line = (total.hold_the_line + bonus.hold_the_line).max(0);
            total.search_and_destroy = (total.search_and_destroy + bonus.search_and_destroy).max(0);
            total.clone()
        }
    };
    let applied = if cfg!(feature = "retail-crc") {
        bonus.clone()
    } else {
        BattlePlanBonuses {
            armor_scalar: bonus.armor_scalar,
            sight_range_scalar: bonus.sight_range_scalar,
            ..total
        }
    };
    for obj in sim.controlling_player_objects(team) {
        apply_bonuses_to_object(sim, obj, &applied);
    }
}

/// `1.0f / std::max(scalar, 0.01f)`.
fn inverse_scalar(scalar: Real) -> Real {
    1.0 / if scalar < 0.01 { 0.01 } else { scalar }
}

/// `Player::changeBattlePlan` for the player controlling `team`.
pub fn change_battle_plan(
    sim: &mut dyn SimApi,
    team: TeamId,
    plan: BattlePlanStatus,
    delta: Int,
    bonus: &BattlePlanBonuses,
) {
    let player = sim.player_index(team);
    let plans = sim.player_battle_plans_mut(player);
    let count = match plan {
        BattlePlanStatus::None => return,
        BattlePlanStatus::Bombardment => &mut plans.bombardment_plans,
        BattlePlanStatus::HoldTheLine => &mut plans.hold_the_line_plans,
        BattlePlanStatus::SearchAndDestroy => &mut plans.search_and_destroy_plans,
    };
    *count = count.wrapping_add(delta);
    let count = *count;
    if count == 1 && delta == 1 {
        apply_bonuses_to_player(sim, team, bonus);
    } else if count == 0 && delta == -1 {
        let mut inverted = bonus.clone();
        inverted.armor_scalar = inverse_scalar(bonus.armor_scalar);
        inverted.sight_range_scalar = inverse_scalar(bonus.sight_range_scalar);
        for value in [
            &mut inverted.bombardment,
            &mut inverted.hold_the_line,
            &mut inverted.search_and_destroy,
        ] {
            if *value > 0 {
                *value = -1;
            }
        }
        apply_bonuses_to_player(sim, team, &inverted);
    }
}

/// The battle plan part of `Player::becomingTeamMember` and `Object::initObject`:
/// `obj` joins (`yes`) or leaves a team of a player with active plans.
pub fn battle_plan_team_member(sim: &mut dyn SimApi, team: TeamId, obj: ObjectSlot, yes: Bool) {
    let player = sim.player_index(team);
    let Some(plans) = sim.player_battle_plans(player) else {
        return;
    };
    if plans.num_active() <= 0 {
        return;
    }
    let Some(total) = plans.bonuses.clone() else {
        return;
    };
    let bonus = if yes != FALSE {
        total
    } else {
        /// `ALL_PLANS`.
        const ALL_PLANS: Int = 1_000_000;
        BattlePlanBonuses {
            armor_scalar: inverse_scalar(total.armor_scalar),
            sight_range_scalar: inverse_scalar(total.sight_range_scalar),
            bombardment: -ALL_PLANS,
            search_and_destroy: -ALL_PLANS,
            hold_the_line: -ALL_PLANS,
            ..total
        }
    };
    apply_bonuses_to_object(sim, obj, &bonus);
}

/// INI template data matching `BattlePlanUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct BattlePlanUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub special_power_name: String,
    pub bombardment_plan_animation_frames: UnsignedInt,
    pub hold_the_line_plan_animation_frames: UnsignedInt,
    pub search_and_destroy_plan_animation_frames: UnsignedInt,
    /// Never initialised by the reference constructor; module data comes from zeroed
    /// memory, so it is zero.
    pub transition_idle_frames: UnsignedInt,
    pub bombardment_unpack_name: String,
    pub bombardment_pack_name: String,
    pub bombardment_message_label: String,
    pub bombardment_announcement_name: String,
    pub search_and_destroy_unpack_name: String,
    pub search_and_destroy_idle_name: String,
    pub search_and_destroy_pack_name: String,
    pub search_and_destroy_message_label: String,
    pub search_and_destroy_announcement_name: String,
    pub hold_the_line_unpack_name: String,
    pub hold_the_line_pack_name: String,
    pub hold_the_line_message_label: String,
    pub hold_the_line_announcement_name: String,
    pub battle_plan_paralyze_frames: UnsignedInt,
    pub valid_member_kind_of: KindOfMaskType,
    pub invalid_member_kind_of: KindOfMaskType,
    pub hold_the_line_armor_damage_scalar: Real,
    pub search_and_destroy_sight_range_scalar: Real,
    pub strategy_center_search_and_destroy_sight_range_scalar: Real,
    pub strategy_center_search_and_destroy_detects_stealth: Bool,
    pub strategy_center_hold_the_line_max_health_scalar: Real,
    pub strategy_center_hold_the_line_max_health_change_type: MaxHealthChangeType,
    pub vision_object_name: String,
}

impl BattlePlanUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            special_power_name: String::new(),
            bombardment_plan_animation_frames: 0,
            hold_the_line_plan_animation_frames: 0,
            search_and_destroy_plan_animation_frames: 0,
            transition_idle_frames: 0,
            bombardment_unpack_name: String::new(),
            bombardment_pack_name: String::new(),
            bombardment_message_label: String::new(),
            bombardment_announcement_name: String::new(),
            search_and_destroy_unpack_name: String::new(),
            search_and_destroy_idle_name: String::new(),
            search_and_destroy_pack_name: String::new(),
            search_and_destroy_message_label: String::new(),
            search_and_destroy_announcement_name: String::new(),
            hold_the_line_unpack_name: String::new(),
            hold_the_line_pack_name: String::new(),
            hold_the_line_message_label: String::new(),
            hold_the_line_announcement_name: String::new(),
            battle_plan_paralyze_frames: 0,
            valid_member_kind_of: KindOfMaskType::new(),
            invalid_member_kind_of: KindOfMaskType::new(),
            hold_the_line_armor_damage_scalar: 1.0,
            search_and_destroy_sight_range_scalar: 1.0,
            strategy_center_search_and_destroy_sight_range_scalar: 1.0,
            strategy_center_search_and_destroy_detects_stealth: TRUE,
            strategy_center_hold_the_line_max_health_scalar: 1.0,
            strategy_center_hold_the_line_max_health_change_type:
                MaxHealthChangeType::PreserveRatio,
            vision_object_name: String::new(),
        }
    }
}

impl ModuleData for BattlePlanUpdateModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x001 | FacetMask::SPECIAL_POWER_UPDATE.0)
    }
}

/// Runtime instance of `BattlePlanUpdate`.
pub struct BattlePlanUpdate {
    pub data: BattlePlanUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub current_plan: BattlePlanStatus,
    pub desired_plan: BattlePlanStatus,
    pub plan_affecting_army: BattlePlanStatus,
    pub status: TransitionStatus,
    pub next_ready_frame: UnsignedInt,
    pub invalid_settings: Bool,
    pub centering_turret: Bool,
    pub bonuses: BattlePlanBonuses,
    pub vision_object_id: ObjectId,
    /// The owner's special power module's template, found when the object is made.
    pub has_special_power_module: Option<crate::arena::ModuleSlot>,
}

/// The door model conditions of one plan.
fn door_flag(plan: BattlePlanStatus, status: TransitionStatus) -> Option<ModelConditionFlagType> {
    use ModelConditionFlagType as F;
    let doors = match plan {
        BattlePlanStatus::None => return None,
        BattlePlanStatus::Bombardment => [F::Door1Opening, F::Door1WaitingToClose, F::Door1Closing],
        BattlePlanStatus::HoldTheLine => [F::Door2Opening, F::Door2WaitingToClose, F::Door2Closing],
        BattlePlanStatus::SearchAndDestroy => {
            [F::Door3Opening, F::Door3WaitingToClose, F::Door3Closing]
        }
    };
    match status {
        TransitionStatus::Idle => None,
        TransitionStatus::Unpacking => Some(doors[0]),
        TransitionStatus::Active => Some(doors[1]),
        TransitionStatus::Packing => Some(doors[2]),
    }
}

impl BattlePlanUpdate {
    pub fn new(data: BattlePlanUpdateModuleData) -> Self {
        let bonuses = BattlePlanBonuses {
            valid_kind_of: data.valid_member_kind_of.clone(),
            invalid_kind_of: data.invalid_member_kind_of.clone(),
            ..BattlePlanBonuses::default()
        };
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            current_plan: BattlePlanStatus::None,
            desired_plan: BattlePlanStatus::None,
            plan_affecting_army: BattlePlanStatus::None,
            status: TransitionStatus::Idle,
            next_ready_frame: 0,
            invalid_settings: FALSE,
            centering_turret: FALSE,
            bonuses,
            vision_object_id: ObjectId::INVALID,
            has_special_power_module: None,
        }
    }

    /// `getActiveBattlePlan`.
    pub fn active_battle_plan(&self) -> BattlePlanStatus {
        if self.status == TransitionStatus::Active {
            self.plan_affecting_army
        } else {
            BattlePlanStatus::None
        }
    }

    /// `getCommandOption`.
    pub fn command_option(&self) -> UnsignedInt {
        match self.desired_plan {
            BattlePlanStatus::Bombardment => OPTION_ONE,
            BattlePlanStatus::HoldTheLine => OPTION_TWO,
            BattlePlanStatus::SearchAndDestroy => OPTION_THREE,
            BattlePlanStatus::None => 0,
        }
    }

    fn enable_turret(sim: &mut dyn SimApi, owner: ObjectSlot, enable: Bool) {
        let turret = sim.ai_turret_for_cur_weapon(owner);
        if turret != WhichTurretType::Invalid {
            sim.ai_set_turret_enabled(owner, turret, enable);
        }
    }

    fn recenter_turret(sim: &mut dyn SimApi, owner: ObjectSlot) {
        let turret = sim.ai_turret_for_cur_weapon(owner);
        if turret != WhichTurretType::Invalid {
            sim.ai_recenter_turret(owner, turret);
        }
    }

    fn is_turret_in_natural_position(sim: &dyn SimView, owner: ObjectSlot) -> bool {
        let turret = sim.ai_turret_for_cur_weapon(owner);
        turret != WhichTurretType::Invalid
            && sim.ai_turret_in_natural_position(owner, turret) != FALSE
    }

    fn team_of(sim: &dyn SimView, owner: ObjectSlot) -> Option<TeamId> {
        sim.object(owner)
            .map(|o| o.team_id)
            .filter(|&t| t != TeamId::INVALID)
    }

    /// `setStatus`.
    fn set_status(
        &mut self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        new_status: TransitionStatus,
    ) {
        if self.status == new_status {
            return;
        }
        if let Some(flag) = door_flag(self.current_plan, self.status) {
            sim.clear_model_condition_state(owner, flag);
        }
        let now = sim.frame();
        match new_status {
            TransitionStatus::Idle => {
                self.current_plan = BattlePlanStatus::None;
                self.next_ready_frame = now.wrapping_add(self.data.transition_idle_frames);
            }
            TransitionStatus::Active => {
                self.set_battle_plan(sim, owner, self.current_plan);
                if let Some(flag) = door_flag(self.current_plan, new_status) {
                    sim.set_model_condition_state(owner, flag);
                }
            }
            TransitionStatus::Unpacking | TransitionStatus::Packing => {
                if new_status == TransitionStatus::Packing {
                    self.set_battle_plan(sim, owner, BattlePlanStatus::None);
                }
                let frames = match self.current_plan {
                    BattlePlanStatus::None => None,
                    BattlePlanStatus::Bombardment => {
                        Some(self.data.bombardment_plan_animation_frames)
                    }
                    BattlePlanStatus::HoldTheLine => {
                        Some(self.data.hold_the_line_plan_animation_frames)
                    }
                    BattlePlanStatus::SearchAndDestroy => {
                        Some(self.data.search_and_destroy_plan_animation_frames)
                    }
                };
                if let (Some(frames), Some(flag)) =
                    (frames, door_flag(self.current_plan, new_status))
                {
                    sim.set_model_condition_state(owner, flag);
                    self.next_ready_frame = now.wrapping_add(frames);
                }
            }
        }
        self.status = new_status;
    }

    /// `paralyzeTroop` over the player's objects.
    fn paralyze_troops(&self, sim: &mut dyn SimApi, team: TeamId) {
        for obj in sim.controlling_player_objects(team) {
            let hit = sim.object(obj).is_some_and(|o| {
                let factory = &sim.rules().thing_factory;
                o.thing
                    .is_any_kind_of(factory, &self.data.valid_member_kind_of)
                    != FALSE
                    && o.thing
                        .is_any_kind_of(factory, &self.data.invalid_member_kind_of)
                        == FALSE
            });
            if hit {
                let until = sim
                    .frame()
                    .wrapping_add(self.data.battle_plan_paralyze_frames);
                sim.set_disabled_until(obj, DisabledType::Paralyzed, until);
            }
        }
    }

    /// `setBattlePlan`.
    fn set_battle_plan(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, plan: BattlePlanStatus) {
        let Some(team) = Self::team_of(&*sim, owner) else {
            return;
        };
        let health_scalar = self.data.strategy_center_hold_the_line_max_health_scalar;
        let health_change = self
            .data
            .strategy_center_hold_the_line_max_health_change_type;
        let sight_scalar = self
            .data
            .strategy_center_search_and_destroy_sight_range_scalar;
        let detects = self.data.strategy_center_search_and_destroy_detects_stealth;
        let previous = self.plan_affecting_army;
        if previous != BattlePlanStatus::None {
            change_battle_plan(sim, team, previous, -1, &self.bonuses);
        }
        match previous {
            BattlePlanStatus::HoldTheLine => {
                if health_scalar != 1.0 {
                    let max = sim.object_max_health(owner) * 1.0 / health_scalar;
                    sim.set_max_health(owner, max, health_change);
                }
            }
            BattlePlanStatus::SearchAndDestroy => {
                if sight_scalar != 1.0 {
                    let vision = sim.object_vision_range(owner) * 1.0 / sight_scalar;
                    sim.set_vision_range(owner, vision);
                    let clearing = sim.object_shroud_clearing_range(owner) * 1.0 / sight_scalar;
                    sim.set_shroud_clearing_range(owner, clearing);
                }
                if detects != FALSE {
                    sim.set_stealth_detector_enabled(owner, FALSE);
                }
            }
            BattlePlanStatus::None | BattlePlanStatus::Bombardment => {}
        }

        self.bonuses.armor_scalar = 1.0;
        self.bonuses.sight_range_scalar = 1.0;
        self.bonuses.bombardment = 0;
        self.bonuses.search_and_destroy = 0;
        self.bonuses.hold_the_line = 0;

        match plan {
            BattlePlanStatus::None => self.paralyze_troops(sim, team),
            BattlePlanStatus::Bombardment => self.bonuses.bombardment = 1,
            BattlePlanStatus::HoldTheLine => {
                if health_scalar != 0.0 {
                    let max = sim.object_max_health(owner) * health_scalar;
                    sim.set_max_health(owner, max, health_change);
                }
                self.bonuses.armor_scalar = self.data.hold_the_line_armor_damage_scalar;
                self.bonuses.hold_the_line = 1;
            }
            BattlePlanStatus::SearchAndDestroy => {
                if sight_scalar != 1.0 {
                    let vision = sim.object_vision_range(owner) * sight_scalar;
                    sim.set_vision_range(owner, vision);
                    let clearing = sim.object_shroud_clearing_range(owner) * sight_scalar;
                    sim.set_shroud_clearing_range(owner, clearing);
                }
                if detects != FALSE {
                    sim.set_stealth_detector_enabled(owner, TRUE);
                }
                self.bonuses.search_and_destroy = 1;
                self.bonuses.sight_range_scalar = self.data.search_and_destroy_sight_range_scalar;
            }
        }
        if plan != BattlePlanStatus::None {
            change_battle_plan(sim, team, plan, 1, &self.bonuses);
        }
        self.plan_affecting_army = plan;
    }
}

impl SpecialPowerUpdateFacet for BattlePlanUpdate {
    fn initiate_intent_to_do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        mut caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        _target_obj: Option<ObjectSlot>,
        _target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> Bool {
        // The C++ dereferences a null module here when the power was never found.
        let Some(module) = self.has_special_power_module else {
            return FALSE;
        };
        let mut module_template = None;
        caller.with(sim, module, &mut |spm, sim, spm_ctx| {
            module_template = Some(spm.get_special_power_id(&*sim, spm_ctx));
        });
        if module_template != Some(template_id) {
            return FALSE;
        }
        self.desired_plan = if command_options & OPTION_ONE != 0 {
            BattlePlanStatus::Bombardment
        } else if command_options & OPTION_TWO != 0 {
            BattlePlanStatus::HoldTheLine
        } else if command_options & OPTION_THREE != 0 {
            BattlePlanStatus::SearchAndDestroy
        } else {
            return FALSE;
        };
        sim.record_battle_plan_selected(ctx.owner_slot);
        TRUE
    }

    fn is_active(&self) -> Bool {
        Bool::from(self.status != TransitionStatus::Idle)
    }
}

impl UpdateFacet for BattlePlanUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        if self.invalid_settings != FALSE {
            return UpdateSleepTime::NONE;
        }
        if self.next_ready_frame > sim.frame() {
            return UpdateSleepTime::NONE;
        }
        match self.status {
            TransitionStatus::Idle => {
                if self.desired_plan != BattlePlanStatus::None {
                    self.current_plan = self.desired_plan;
                    self.set_status(sim, owner, TransitionStatus::Unpacking);
                }
            }
            TransitionStatus::Unpacking => {
                self.set_status(sim, owner, TransitionStatus::Active);
                if self.current_plan == BattlePlanStatus::Bombardment {
                    Self::enable_turret(sim, owner, TRUE);
                }
            }
            TransitionStatus::Active => {
                if self.current_plan != self.desired_plan {
                    if self.current_plan != BattlePlanStatus::Bombardment {
                        self.set_status(sim, owner, TransitionStatus::Packing);
                    } else if sim.object_has_ai(owner) != FALSE {
                        if Self::is_turret_in_natural_position(&*sim, owner) {
                            self.set_status(sim, owner, TransitionStatus::Packing);
                            self.centering_turret = FALSE;
                            Self::enable_turret(sim, owner, FALSE);
                        } else if self.centering_turret == FALSE {
                            sim.ai_idle(owner, CommandSourceType::FromAi);
                            if !cfg!(feature = "retail-crc") {
                                Self::enable_turret(sim, owner, FALSE);
                            }
                            Self::recenter_turret(sim, owner);
                            self.centering_turret = TRUE;
                        }
                    }
                }
            }
            TransitionStatus::Packing => self.set_status(sim, owner, TransitionStatus::Idle),
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for BattlePlanUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        if self.data.special_power_name.is_empty() {
            self.invalid_settings = TRUE;
            return;
        }
        let template_id = sim
            .rules()
            .special_power_store
            .find_special_power_template(&self.data.special_power_name)
            .map(|t| t.id());
        self.has_special_power_module =
            template_id.and_then(|id| sim.find_special_power_module(owner, id));
        sim.set_weapon_set_flag(owner, WeaponSetType::Veteran, TRUE);
        if sim.object_has_ai(owner) != FALSE {
            sim.lock_weapon_slot(
                owner,
                WeaponSlotType::Primary,
                WeaponLockType::LockedTemporarily,
            );
        }
        Self::enable_turret(sim, owner, FALSE);
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if cfg!(feature = "retail-crc") {
            return;
        }
        change_battle_plan(sim, old_team, self.plan_affecting_army, -1, &self.bonuses);
        change_battle_plan(sim, new_team, self.plan_affecting_army, 1, &self.bonuses);
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.vision_object_id != ObjectId::INVALID {
            if let Some(obj) = sim.find_object(self.vision_object_id) {
                sim.destroy_object(obj);
            }
        }
        if let Some(team) = Self::team_of(&*sim, ctx.owner_slot) {
            if self.plan_affecting_army != BattlePlanStatus::None {
                change_battle_plan(sim, team, self.plan_affecting_army, -1, &self.bonuses);
            }
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// The `SpecialPowerUpdateModule` version byte is skipped: the reference calls
    /// `UpdateModule::xfer` directly.
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let user = |xfer: &mut dyn Xfer, value: Int| -> Result<Int, XferError> {
            let mut bytes = value.to_le_bytes();
            xfer.xfer_user(&mut bytes)?;
            Ok(Int::from_le_bytes(bytes))
        };
        self.current_plan = BattlePlanStatus::from_i32(user(xfer, self.current_plan as Int)?);
        self.desired_plan = BattlePlanStatus::from_i32(user(xfer, self.desired_plan as Int)?);
        self.plan_affecting_army =
            BattlePlanStatus::from_i32(user(xfer, self.plan_affecting_army as Int)?);
        self.status = TransitionStatus::from_i32(user(xfer, self.status as Int)?);
        xfer.xfer_unsigned_int(&mut self.next_ready_frame)?;
        xfer.xfer_bool(&mut self.invalid_settings)?;
        xfer.xfer_bool(&mut self.centering_turret)?;
        xfer.xfer_real(&mut self.bonuses.armor_scalar)?;
        xfer.xfer_int(&mut self.bonuses.bombardment)?;
        xfer.xfer_int(&mut self.bonuses.search_and_destroy)?;
        xfer.xfer_int(&mut self.bonuses.hold_the_line)?;
        xfer.xfer_real(&mut self.bonuses.sight_range_scalar)?;
        self.bonuses.valid_kind_of.xfer_named(xfer, &KINDOF_NAMES)?;
        self.bonuses
            .invalid_kind_of
            .xfer_named(xfer, &KINDOF_NAMES)?;
        xfer.xfer_object_id(&mut self.vision_object_id)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_special_power_update_mut(&mut self) -> Option<&mut dyn SpecialPowerUpdateFacet> {
        Some(self)
    }
}

pub fn parse_battle_plan_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = BattlePlanUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let d = &mut data;
        let key = field.to_ascii_lowercase();
        match key.as_str() {
            "specialpowertemplate" => {
                r.parse_ascii_string(&mut d.special_power_name)?;
                if d.special_power_name.eq_ignore_ascii_case("None") {
                    d.special_power_name.clear();
                }
                Ok(())
            }
            "bombardmentplananimationtime" => {
                r.parse_duration_unsigned_int(&mut d.bombardment_plan_animation_frames)
            }
            "holdthelineplananimationtime" => {
                r.parse_duration_unsigned_int(&mut d.hold_the_line_plan_animation_frames)
            }
            "searchanddestroyplananimationtime" => {
                r.parse_duration_unsigned_int(&mut d.search_and_destroy_plan_animation_frames)
            }
            "transitionidletime" => r.parse_duration_unsigned_int(&mut d.transition_idle_frames),
            "bombardmentplanunpacksoundname" => {
                r.parse_ascii_string(&mut d.bombardment_unpack_name)
            }
            "bombardmentplanpacksoundname" => r.parse_ascii_string(&mut d.bombardment_pack_name),
            "bombardmentmessagelabel" => r.parse_ascii_string(&mut d.bombardment_message_label),
            "bombardmentannouncementname" => {
                r.parse_ascii_string(&mut d.bombardment_announcement_name)
            }
            "searchanddestroyplanunpacksoundname" => {
                r.parse_ascii_string(&mut d.search_and_destroy_unpack_name)
            }
            "searchanddestroyplanidleloopsoundname" => {
                r.parse_ascii_string(&mut d.search_and_destroy_idle_name)
            }
            "searchanddestroyplanpacksoundname" => {
                r.parse_ascii_string(&mut d.search_and_destroy_pack_name)
            }
            "searchanddestroymessagelabel" => {
                r.parse_ascii_string(&mut d.search_and_destroy_message_label)
            }
            "searchanddestroyannouncementname" => {
                r.parse_ascii_string(&mut d.search_and_destroy_announcement_name)
            }
            "holdthelineplanunpacksoundname" => {
                r.parse_ascii_string(&mut d.hold_the_line_unpack_name)
            }
            "holdthelineplanpacksoundname" => r.parse_ascii_string(&mut d.hold_the_line_pack_name),
            "holdthelinemessagelabel" => r.parse_ascii_string(&mut d.hold_the_line_message_label),
            "holdthelineannouncementname" => {
                r.parse_ascii_string(&mut d.hold_the_line_announcement_name)
            }
            "validmemberkindof" => d.valid_member_kind_of.parse_from_ini(r, &KINDOF_NAMES),
            "invalidmemberkindof" => d.invalid_member_kind_of.parse_from_ini(r, &KINDOF_NAMES),
            "battleplanchangeparalyzetime" => {
                r.parse_duration_unsigned_int(&mut d.battle_plan_paralyze_frames)
            }
            "holdthelineplanarmordamagescalar" => {
                r.parse_real(&mut d.hold_the_line_armor_damage_scalar)
            }
            "searchanddestroyplansightrangescalar" => {
                r.parse_real(&mut d.search_and_destroy_sight_range_scalar)
            }
            "strategycentersearchanddestroysightrangescalar" => {
                r.parse_real(&mut d.strategy_center_search_and_destroy_sight_range_scalar)
            }
            "strategycentersearchanddestroydetectsstealth" => {
                r.parse_bool(&mut d.strategy_center_search_and_destroy_detects_stealth)
            }
            "strategycenterholdthelinemaxhealthscalar" => {
                r.parse_real(&mut d.strategy_center_hold_the_line_max_health_scalar)
            }
            "strategycenterholdthelinemaxhealthchangetype" => {
                let mut index = 0;
                r.parse_index_list(&mut index, &MAX_HEALTH_CHANGE_NAMES)?;
                d.strategy_center_hold_the_line_max_health_change_type =
                    MaxHealthChangeType::from_i32(index).ok_or(IniError::InvalidData)?;
                Ok(())
            }
            "visionobjectname" => r.parse_ascii_string(&mut d.vision_object_name),
            _ => Err(IniError::UnknownToken),
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_battle_plan_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BattlePlanUpdateModuleData>()
        .expect("BattlePlanUpdate data type mismatch")
        .clone();
    Box::new(BattlePlanUpdate::new(typed))
}
