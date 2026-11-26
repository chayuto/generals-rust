//! `PKT-374`: `StealthUpdateModuleData` and `StealthUpdate`
//! (`StealthUpdate.h`, `StealthUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-374_StealthUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt374_373_stealth.rs`.
//!
//! Left out because they are client only and reach neither saved state nor the logic
//! PRNG: drawable opacity and stealth look, the second material pass hint, the stealth
//! and disguise sounds, control bar and selection updates, indicator colours, and the
//! radar remove/add around a disguise change.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot, ThingTemplateId};
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{
    BehaviorModule, CommandSourceType, ModuleData, StealthFacet, StealthParams, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::object_script_status_bits::OBJECT_STATUS_SCRIPT_UNSTEALTHED;
use crate::rules::disabled_types::{DisabledMaskType, DisabledType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::{
    ObjectStatusMaskType, ObjectStatusType, OBJECT_STATUS_NAMES,
};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_sqrtf, PI};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const STEALTH_NOT_WHILE_ATTACKING: UnsignedInt = 0x0000_0001;
pub const STEALTH_NOT_WHILE_MOVING: UnsignedInt = 0x0000_0002;
pub const STEALTH_NOT_WHILE_USING_ABILITY: UnsignedInt = 0x0000_0004;
pub const STEALTH_NOT_WHILE_FIRING_PRIMARY: UnsignedInt = 0x0000_0008;
pub const STEALTH_NOT_WHILE_FIRING_SECONDARY: UnsignedInt = 0x0000_0010;
pub const STEALTH_NOT_WHILE_FIRING_TERTIARY: UnsignedInt = 0x0000_0020;
pub const STEALTH_ONLY_WITH_BLACK_MARKET: UnsignedInt = 0x0000_0040;
pub const STEALTH_NOT_WHILE_TAKING_DAMAGE: UnsignedInt = 0x0000_0080;
pub const STEALTH_NOT_WHILE_RIDERS_ATTACKING: UnsignedInt = 0x0000_0100;
pub const STEALTH_NOT_WHILE_FIRING_WEAPON: UnsignedInt = STEALTH_NOT_WHILE_FIRING_PRIMARY
    | STEALTH_NOT_WHILE_FIRING_SECONDARY
    | STEALTH_NOT_WHILE_FIRING_TERTIARY;

/// `TheStealthLevelNames`.
pub const STEALTH_LEVEL_NAMES: [&str; 9] = [
    "ATTACKING",
    "MOVING",
    "USING_ABILITY",
    "FIRING_PRIMARY",
    "FIRING_SECONDARY",
    "FIRING_TERTIARY",
    "NO_BLACK_MARKET",
    "TAKING_DAMAGE",
    "RIDERS_ATTACKING",
];

/// `TheEvaMessageNames`: only used to reject unknown names, as the original does.
const EVA_MESSAGE_NAMES: [&str; 53] = [
    "LOWPOWER",
    "INSUFFICIENTFUNDS",
    "SUPERWEAPONDETECTED_OWN_PARTICLECANNON",
    "SUPERWEAPONDETECTED_OWN_NUKE",
    "SUPERWEAPONDETECTED_OWN_SCUDSTORM",
    "SUPERWEAPONDETECTED_ALLY_PARTICLECANNON",
    "SUPERWEAPONDETECTED_ALLY_NUKE",
    "SUPERWEAPONDETECTED_ALLY_SCUDSTORM",
    "SUPERWEAPONDETECTED_ENEMY_PARTICLECANNON",
    "SUPERWEAPONDETECTED_ENEMY_NUKE",
    "SUPERWEAPONDETECTED_ENEMY_SCUDSTORM",
    "SUPERWEAPONLAUNCHED_OWN_PARTICLECANNON",
    "SUPERWEAPONLAUNCHED_OWN_NUKE",
    "SUPERWEAPONLAUNCHED_OWN_SCUDSTORM",
    "SUPERWEAPONLAUNCHED_ALLY_PARTICLECANNON",
    "SUPERWEAPONLAUNCHED_ALLY_NUKE",
    "SUPERWEAPONLAUNCHED_ALLY_SCUDSTORM",
    "SUPERWEAPONLAUNCHED_ENEMY_PARTICLECANNON",
    "SUPERWEAPONLAUNCHED_ENEMY_NUKE",
    "SUPERWEAPONLAUNCHED_ENEMY_SCUDSTORM",
    "SUPERWEAPONREADY_OWN_PARTICLECANNON",
    "SUPERWEAPONREADY_OWN_NUKE",
    "SUPERWEAPONREADY_OWN_SCUDSTORM",
    "SUPERWEAPONREADY_ALLY_PARTICLECANNON",
    "SUPERWEAPONREADY_ALLY_NUKE",
    "SUPERWEAPONREADY_ALLY_SCUDSTORM",
    "SUPERWEAPONREADY_ENEMY_PARTICLECANNON",
    "SUPERWEAPONREADY_ENEMY_NUKE",
    "SUPERWEAPONREADY_ENEMY_SCUDSTORM",
    "BUILDINGLOST",
    "BASEUNDERATTACK",
    "ALLYUNDERATTACK",
    "BEACONDETECTED",
    "ENEMYBLACKLOTUSDETECTED",
    "ENEMYJARMENKELLDETECTED",
    "ENEMYCOLONELBURTONDETECTED",
    "OWNBLACKLOTUSDETECTED",
    "OWNJARMENKELLDETECTED",
    "OWNCOLONELBURTONDETECTED",
    "UNITLOST",
    "GENERALLEVELUP",
    "VEHICLESTOLEN",
    "BUILDINGSTOLEN",
    "CASHSTOLEN",
    "UPGRADECOMPLETE",
    "BUILDINGBEINGSTOLEN",
    "BUILDINGSABOTAGED",
    "SUPERWEAPONLAUNCHED_OWN_GPS_SCRAMBLER",
    "SUPERWEAPONLAUNCHED_ALLY_GPS_SCRAMBLER",
    "SUPERWEAPONLAUNCHED_ENEMY_GPS_SCRAMBLER",
    "SUPERWEAPONLAUNCHED_OWN_SNEAK_ATTACK",
    "SUPERWEAPONLAUNCHED_ALLY_SNEAK_ATTACK",
    "SUPERWEAPONLAUNCHED_ENEMY_SNEAK_ATTACK",
];

const FOREVER: UnsignedInt = 0x3fff_ffff;

/// INI template data matching `StealthUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct StealthUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub hint_detectable_states: ObjectStatusMaskType,
    pub required_status: ObjectStatusMaskType,
    pub forbidden_status: ObjectStatusMaskType,
    pub disguise_reveal_fx: Option<String>,
    pub disguise_fx: Option<String>,
    pub stealth_speed: Real,
    pub friendly_opacity_min: Real,
    pub friendly_opacity_max: Real,
    pub reveal_distance_from_target: Real,
    pub disguise_transition_frames: UnsignedInt,
    pub disguise_reveal_transition_frames: UnsignedInt,
    pub pulse_frames: UnsignedInt,
    pub stealth_delay: UnsignedInt,
    pub stealth_level: UnsignedInt,
    pub black_market_check_frames: UnsignedInt,
    pub enemy_detection_eva_event: Option<String>,
    pub own_detection_eva_event: Option<String>,
    pub innate_stealth: Bool,
    pub order_idle_enemies_to_attack_me_upon_reveal: Bool,
    pub team_disguised: Bool,
    pub use_rider_stealth: Bool,
    pub granted_by_special_power: Bool,
}

impl StealthUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            hint_detectable_states: ObjectStatusMaskType::new(),
            required_status: ObjectStatusMaskType::new(),
            forbidden_status: ObjectStatusMaskType::new(),
            disguise_reveal_fx: None,
            disguise_fx: None,
            stealth_speed: 0.0,
            friendly_opacity_min: 0.5,
            friendly_opacity_max: 1.0,
            reveal_distance_from_target: 0.0,
            disguise_transition_frames: 0,
            disguise_reveal_transition_frames: 0,
            pulse_frames: 30,
            stealth_delay: UnsignedInt::MAX,
            stealth_level: 0,
            black_market_check_frames: 0,
            enemy_detection_eva_event: None,
            own_detection_eva_event: None,
            innate_stealth: TRUE,
            order_idle_enemies_to_attack_me_upon_reveal: FALSE,
            team_disguised: FALSE,
            use_rider_stealth: FALSE,
            granted_by_special_power: FALSE,
        }
    }
}

impl ModuleData for StealthUpdateModuleData {
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
        FacetMask(0x001 | FacetMask::STEALTH_UPDATE.0)
    }
}

/// Runtime instance of `StealthUpdate`.
pub struct StealthUpdate {
    pub data: StealthUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub stealth_allowed_frame: UnsignedInt,
    pub detection_expires_frame: UnsignedInt,
    /// Not saved.
    pub next_black_market_check_frame: UnsignedInt,
    pub enabled: Bool,
    pub pulse_phase_rate: Real,
    pub pulse_phase: Real,
    pub disguise_as_player_index: Int,
    pub disguise_as_template: Option<ThingTemplateId>,
    pub disguise_transition_frames: UnsignedInt,
    pub disguise_halfpoint_reached: Bool,
    pub transitioning_to_disguise: Bool,
    pub disguised: Bool,
    pub frames_granted: UnsignedInt,
    /// Not saved: set by `loadPostProcess`.
    pub xfer_restore_disguise: Bool,
}

impl StealthUpdate {
    pub fn new(data: StealthUpdateModuleData) -> Self {
        let enabled = Bool::from(data.team_disguised == FALSE);
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            stealth_allowed_frame: 0,
            detection_expires_frame: 0,
            next_black_market_check_frame: 0,
            enabled,
            pulse_phase_rate: 0.2,
            pulse_phase: 0.0,
            disguise_as_player_index: -1,
            disguise_as_template: None,
            disguise_transition_frames: 0,
            disguise_halfpoint_reached: FALSE,
            transitioning_to_disguise: FALSE,
            disguised: FALSE,
            frames_granted: 0,
            xfer_restore_disguise: FALSE,
        }
    }

    fn params(&self) -> StealthParams {
        StealthParams {
            stealth_delay: self.data.stealth_delay,
            stealth_level: self.data.stealth_level,
            order_idle_enemies_on_reveal: self.data.order_idle_enemies_to_attack_me_upon_reveal,
            disguised_template: self.disguise_as_template,
            disguised_player_index: self.disguise_as_player_index,
            granted_by_special_power: self.data.granted_by_special_power,
            temporary_grant: Bool::from(self.frames_granted > 0),
        }
    }

    fn status(sim: &dyn SimView, slot: ObjectSlot, status: ObjectStatusType) -> bool {
        sim.object(slot)
            .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
    }

    /// `calcStealthOwner`: the first rider with `UseRiderStealth`, else the owner.
    fn calc_stealth_owner(&self, sim: &dyn SimView, owner: ObjectSlot) -> ObjectSlot {
        if self.data.use_rider_stealth != FALSE {
            if let Some(&id) = sim.contain_items(owner).first() {
                if let Some(rider) = sim.find_object(id) {
                    return rider;
                }
            }
        }
        owner
    }

    /// `calcStealthOwner()->getStealth()` when the owner is another object.
    ///
    /// The original reads the delay and the reveal order from uninitialised locals when a
    /// rider has no `StealthUpdate`; this uses the owner's own data there.
    fn owner_params(
        &self,
        sim: &dyn SimView,
        owner: ObjectSlot,
        stealth_owner: ObjectSlot,
    ) -> StealthParams {
        if owner == stealth_owner {
            self.params()
        } else {
            sim.stealth_params(stealth_owner)
                .unwrap_or_else(|| self.params())
        }
    }

    fn calc_sleep_time(&self) -> UpdateSleepTime {
        if self.enabled != FALSE {
            UpdateSleepTime::NONE
        } else {
            UpdateSleepTime::FOREVER
        }
    }

    fn has_black_market(sim: &dyn SimView, owner: ObjectSlot) -> bool {
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return false;
        };
        sim.controlling_player_objects(team)
            .into_iter()
            .any(|slot| {
                sim.object_is_kind_of(slot, KindOfType::FsBlackMarket) != FALSE
                    && sim.object(slot).is_some_and(|o| {
                        o.is_effectively_dead() == FALSE
                            && o.status.test(ObjectStatusType::UnderConstruction.as_i32()) == FALSE
                            && o.status.test(ObjectStatusType::Sold.as_i32()) == FALSE
                    })
            })
    }

    /// `allowedToStealth`.
    fn check_allowed_to_stealth(
        &mut self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        stealth_owner: ObjectSlot,
    ) -> bool {
        let now = sim.frame();
        let mut flags = self.data.stealth_level;
        if owner != stealth_owner {
            if let Some(params) = sim.stealth_params(stealth_owner) {
                flags = params.stealth_level;
            }
        }

        if sim.object_is_kind_of(owner, KindOfType::SpawnsAreTheWeapons) != FALSE
            && sim.spawn_all_slaves_stealthed(owner) == Some(FALSE)
        {
            sim.spawn_reveal_slaves(owner);
            return false;
        }

        if flags & STEALTH_NOT_WHILE_ATTACKING != 0
            && Self::status(sim, owner, ObjectStatusType::IsFiringWeapon)
        {
            return false;
        }
        if flags & STEALTH_NOT_WHILE_USING_ABILITY != 0
            && Self::status(sim, owner, ObjectStatusType::IsUsingAbility)
        {
            return false;
        }

        if flags & STEALTH_ONLY_WITH_BLACK_MARKET != 0 && self.next_black_market_check_frame < now {
            let jitter = sim.rng_logic_int(0, 10) as UnsignedInt;
            self.next_black_market_check_frame = self
                .next_black_market_check_frame
                .wrapping_add(self.data.black_market_check_frames.wrapping_add(jitter));
            if !Self::has_black_market(sim, owner) {
                return false;
            }
        }

        if !Self::status(sim, stealth_owner, ObjectStatusType::CanStealth) {
            return false;
        }

        if flags & STEALTH_NOT_WHILE_TAKING_DAMAGE != 0 {
            if let Some(stamp) = sim.object_last_damage_timestamp(owner) {
                if stamp >= now.wrapping_sub(1) {
                    #[cfg(feature = "retail-crc")]
                    let hurt = stamp != 0xffff_ffff;
                    #[cfg(not(feature = "retail-crc"))]
                    let hurt = true;
                    if hurt {
                        return false;
                    }
                }
            }
        }

        let Some(status) = sim.object(owner).map(|o| o.status.clone()) else {
            return false;
        };
        if status.test_for_all(&self.data.required_status) == FALSE {
            return false;
        }
        if status.test_for_any(&self.data.forbidden_status) != FALSE {
            return false;
        }

        if flags & STEALTH_NOT_WHILE_FIRING_WEAPON != 0
            && Self::status(sim, owner, ObjectStatusType::IsFiringWeapon)
        {
            if flags & STEALTH_NOT_WHILE_FIRING_WEAPON == STEALTH_NOT_WHILE_FIRING_WEAPON {
                return false;
            }
            let last_frame = now.wrapping_sub(1);
            for (slot, flag) in [
                STEALTH_NOT_WHILE_FIRING_PRIMARY,
                STEALTH_NOT_WHILE_FIRING_SECONDARY,
                STEALTH_NOT_WHILE_FIRING_TERTIARY,
            ]
            .into_iter()
            .enumerate()
            {
                if flags & flag != 0
                    && sim
                        .weapon_slot_last_shot_frame(owner, slot)
                        .is_some_and(|f| f >= last_frame)
                {
                    return false;
                }
            }
        }

        if let Some(container) = sim.object(owner).and_then(|o| o.contained_by) {
            if sim.object_has_contain(container) != FALSE
                && sim.contain_is_garrisonable(container) == FALSE
            {
                return false;
            }
        }

        if flags & STEALTH_NOT_WHILE_RIDERS_ATTACKING != 0
            && sim.object_has_contain(owner) != FALSE
            && sim.contain_is_passenger_allowed_to_fire(owner, ObjectId::INVALID) != FALSE
            && sim.contain_items(owner).into_iter().any(|id| {
                sim.find_object(id)
                    .is_some_and(|r| Self::status(sim, r, ObjectStatusType::IsAttacking))
            })
        {
            return false;
        }

        if flags & STEALTH_NOT_WHILE_MOVING != 0 && sim.has_physics(owner) != FALSE {
            let v = sim.physics_velocity(owner);
            if gm_sqrtf(v.x * v.x + v.y * v.y + v.z * v.z) > self.data.stealth_speed {
                return false;
            }
        }

        !sim.object(owner)
            .is_some_and(|o| o.script_status & OBJECT_STATUS_SCRIPT_UNSTEALTHED != 0)
    }

    /// `markAsDetected`.
    fn detect(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, num_frames: UnsignedInt) {
        let owner = ctx.owner_slot;
        let stealth_owner = self.calc_stealth_owner(sim, owner);
        let params = self.owner_params(sim, owner, stealth_owner);
        let Some(this_team) = sim.object(owner).map(|o| o.team_id) else {
            return;
        };

        if self.disguise_as_template.is_some() {
            self.disguise(sim, ctx, None);
        }

        let now = sim.frame();
        if num_frames == 0 {
            self.detection_expires_frame = now.wrapping_add(params.stealth_delay);
        } else if self.detection_expires_frame < now.wrapping_add(num_frames) {
            self.detection_expires_frame = now.wrapping_add(num_frames);
        }

        if params.order_idle_enemies_on_reveal != FALSE {
            let Some(here) = sim.object(owner).map(|o| *o.thing.get_position()) else {
                return;
            };
            for enemy in sim.enemy_player_objects(this_team) {
                if sim.object_has_ai(enemy) == FALSE {
                    continue;
                }
                let vision = sim.object_vision_range(enemy);
                let Some(mut delta) = sim.object(enemy).map(|o| *o.thing.get_position()) else {
                    continue;
                };
                delta.x -= here.x;
                delta.y -= here.y;
                delta.z -= here.z;
                if delta.length() > vision {
                    continue;
                }
                sim.ai_wake_up_and_attempt_to_target(enemy);
            }
        }
    }

    /// `disguiseAsObject`.
    fn disguise(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, target: Option<ObjectSlot>) {
        let owner = ctx.owner_slot;
        let target = target.and_then(|slot| {
            let record = sim.object(slot)?;
            let template = record
                .thing
                .get_template(&sim.rules().thing_factory)
                .get_template_id();
            Some((slot, ThingTemplateId(u32::from(template)), record.team_id))
        });
        if let Some((slot, template, team)) = target {
            let theirs = if slot == owner {
                Some(self.params())
            } else {
                sim.stealth_params(slot)
            };
            match theirs.and_then(|p| Some((p.disguised_template?, p.disguised_player_index))) {
                Some((disguise, player)) => {
                    self.disguise_as_template = Some(disguise);
                    self.disguise_as_player_index = player;
                }
                None => {
                    self.disguise_as_template = Some(template);
                    self.disguise_as_player_index = sim.player_index(team);
                }
            }
            self.enabled = TRUE;
            self.transitioning_to_disguise = TRUE;
            self.disguise_transition_frames = self.data.disguise_transition_frames;
            self.disguise_halfpoint_reached = FALSE;
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        } else if self.disguised != FALSE {
            self.disguise_as_template = None;
            self.disguise_as_player_index = 0;
            self.disguise_transition_frames = self.data.disguise_reveal_transition_frames;
            self.transitioning_to_disguise = FALSE;
            self.disguise_halfpoint_reached = FALSE;
        }
    }

    /// `changeVisualDisguise`: the model swap itself is the client's.
    fn change_visual_disguise(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) else {
            return;
        };
        if self.disguise_as_template.is_some() {
            // The original dereferences `getPhysics()` unchecked here.
            sim.reset_dynamic_physics(owner);
            if let Some(fx) = &self.data.disguise_fx {
                sim.trigger_fx_list(fx, &pos);
            }
            self.disguised = TRUE;
            sim.set_status(owner, ObjectStatusType::Disguised);
            sim.set_model_condition_state(owner, ModelConditionFlagType::Disguised);
            sim.record_vehicle_disguised(owner);
        } else if self.disguise_as_player_index != -1 {
            self.disguise_as_player_index = -1;
            sim.reset_dynamic_physics(owner);
            if let Some(fx) = &self.data.disguise_reveal_fx {
                sim.trigger_fx_list(fx, &pos);
            }
            self.disguised = FALSE;
            sim.clear_status(owner, ObjectStatusType::Disguised);
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Disguised);
        }
        self.xfer_restore_disguise = FALSE;
    }

    /// `receiveGrant`.
    fn grant(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        active: Bool,
        frames: UnsignedInt,
    ) {
        let owner = ctx.owner_slot;
        if sim.object(owner).is_none() || self.data.team_disguised != FALSE {
            return;
        }
        self.enabled = active;
        if active != FALSE {
            sim.set_status(owner, ObjectStatusType::CanStealth);
            sim.set_status(owner, ObjectStatusType::Stealthed);
            self.stealth_allowed_frame = sim.frame();
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
            self.frames_granted = frames;
        } else {
            sim.clear_status(owner, ObjectStatusType::CanStealth);
            sim.clear_status(owner, ObjectStatusType::Stealthed);
            self.stealth_allowed_frame = FOREVER;
            self.frames_granted = 0;
        }

        if sim.contain_is_rider_change(owner) != FALSE {
            let rider = sim.contain_items(owner).first().copied();
            if let Some(rider) = rider.and_then(|id| sim.find_object(id)) {
                sim.receive_stealth_grant(rider, active, frames);
            }
        }
    }
}

impl StealthFacet for StealthUpdate {
    fn is_disguised(&self) -> Bool {
        Bool::from(self.disguise_as_template.is_some())
    }

    fn stealth_params(&self) -> Option<StealthParams> {
        Some(self.params())
    }

    fn mark_as_detected(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, frames: UnsignedInt) {
        self.detect(sim, ctx, frames);
    }

    fn receive_grant(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        active: Bool,
        frames: UnsignedInt,
    ) {
        self.grant(sim, ctx, active, frames);
    }

    fn disguise_as_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: Option<ObjectSlot>,
    ) {
        self.disguise(sim, ctx, target);
    }

    fn allowed_to_stealth(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        stealth_owner: ObjectSlot,
    ) -> Bool {
        Bool::from(self.check_allowed_to_stealth(sim, ctx.owner_slot, stealth_owner))
    }
}

impl UpdateFacet for StealthUpdate {
    fn disabled_types_to_process(&self) -> DisabledMaskType {
        let mut mask = DisabledMaskType::new();
        mask.set(DisabledType::Held.as_i32(), 1);
        mask
    }

    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        if self.xfer_restore_disguise == TRUE {
            self.change_visual_disguise(sim, owner);
        }

        let stealth_owner = self.calc_stealth_owner(sim, owner);
        let stealth_delay = self.owner_params(sim, owner, stealth_owner).stealth_delay;
        let now = sim.frame();

        if self.enabled == FALSE {
            return self.calc_sleep_time();
        }

        if sim.object_has_drawable(owner) != FALSE
            && sim.object_is_kind_of(owner, KindOfType::Mine) == FALSE
        {
            if self.disguise_transition_frames != 0 {
                self.disguise_transition_frames -= 1;
                let total = if self.transitioning_to_disguise != FALSE {
                    self.data.disguise_transition_frames
                } else {
                    self.data.disguise_reveal_transition_frames
                };
                let factor = 1.0 - (self.disguise_transition_frames as Real / total as Real);
                if factor >= 0.5 && self.disguise_halfpoint_reached == FALSE {
                    self.change_visual_disguise(sim, owner);
                    self.disguise_halfpoint_reached = TRUE;
                }
                if self.disguise_transition_frames == 0 && self.transitioning_to_disguise == FALSE {
                    self.enabled = FALSE;
                    sim.clear_status(owner, ObjectStatusType::Stealthed);
                    sim.clear_status(owner, ObjectStatusType::Detected);
                    return self.calc_sleep_time();
                }
            } else {
                self.pulse_phase += self.pulse_phase_rate;
            }
        }

        let reveal = self.data.reveal_distance_from_target;
        if reveal > 0.0 && sim.object_has_ai(owner) != FALSE {
            if let Some(target) = sim.ai_current_victim(owner) {
                let here = sim.object(owner).map(|o| *o.thing.get_position());
                let there = sim.object(target).map(|o| *o.thing.get_position());
                if let (Some(here), Some(there)) = (here, there) {
                    let dx = there.x - here.x;
                    let dy = there.y - here.y;
                    if dx * dx + dy * dy <= reveal * reveal {
                        self.detect(sim, ctx, 0);
                        return self.calc_sleep_time();
                    }
                }
            }
        }

        if self.frames_granted > 0 {
            self.frames_granted -= 1;
            if sim.object_has_ai(owner) != FALSE
                && sim.ai_last_command_source(owner) == CommandSourceType::FromPlayer
            {
                self.grant(sim, ctx, FALSE, 0);
            }
            if self.frames_granted == 0 {
                self.grant(sim, ctx, FALSE, 0);
            }
        }

        if self.check_allowed_to_stealth(sim, owner, stealth_owner) {
            if self.stealth_allowed_frame > now {
                return self.calc_sleep_time();
            }
            sim.set_status(owner, ObjectStatusType::Stealthed);
        } else {
            self.stealth_allowed_frame = now.wrapping_add(stealth_delay);
            sim.clear_status(owner, ObjectStatusType::Stealthed);
        }

        let was_detected = Self::status(sim, owner, ObjectStatusType::Detected);
        let detected = self.detection_expires_frame > now;
        if detected {
            sim.set_status(owner, ObjectStatusType::Detected);
        } else {
            sim.clear_status(owner, ObjectStatusType::Detected);
        }

        if was_detected != detected {
            if let Some(container) = sim.object(owner).and_then(|o| o.contained_by) {
                if sim.contain_is_garrisonable(container) != FALSE {
                    sim.contain_recalc_apparent_controlling_player(container);
                }
            }
        }

        self.calc_sleep_time()
    }
}

impl BehaviorModule for StealthUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.stealth_allowed_frame = sim.frame().wrapping_add(self.data.stealth_delay);
        self.pulse_phase = sim.rng_client_real(0.0, PI);
        if self.data.innate_stealth != FALSE {
            sim.set_status(ctx.owner_slot, ObjectStatusType::CanStealth);
        }
        let sleep = if self.data.granted_by_special_power != FALSE {
            UpdateSleepTime::FOREVER
        } else {
            UpdateSleepTime::NONE
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.stealth_allowed_frame)?;
        xfer.xfer_unsigned_int(&mut self.detection_expires_frame)?;
        xfer.xfer_bool(&mut self.enabled)?;
        xfer.xfer_real(&mut self.pulse_phase_rate)?;
        xfer.xfer_real(&mut self.pulse_phase)?;
        xfer.xfer_int(&mut self.disguise_as_player_index)?;

        let factory = &sim.rules().thing_factory;
        let mut name = self
            .disguise_as_template
            .and_then(|id| factory.find_by_template_id(id.0 as u16))
            .map_or_else(String::new, |t| t.get_name().to_owned());
        xfer.xfer_ascii_string(&mut name)?;
        if xfer.mode() == XferMode::Load {
            self.disguise_as_template = None;
            if !name.is_empty() {
                let id = factory.find_template_id(&name);
                self.disguise_as_template = Some(id.ok_or(XferError::InvalidData)?);
            }
        }

        xfer.xfer_unsigned_int(&mut self.disguise_transition_frames)?;
        xfer.xfer_bool(&mut self.disguise_halfpoint_reached)?;
        xfer.xfer_bool(&mut self.transitioning_to_disguise)?;
        xfer.xfer_bool(&mut self.disguised)?;
        if version >= 2 {
            xfer.xfer_unsigned_int(&mut self.frames_granted)?;
        }
        Ok(())
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
        if self.disguise_as_template.is_some() {
            self.xfer_restore_disguise = TRUE;
        }
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

    fn as_stealth(&self) -> Option<&dyn StealthFacet> {
        Some(self)
    }

    fn as_stealth_mut(&mut self) -> Option<&mut dyn StealthFacet> {
        Some(self)
    }
}

fn parse_eva(reader: &mut IniReader<'_>, store: &mut Option<String>) -> Result<(), IniError> {
    let tok = reader.next_token(SEPS_DEFAULT)?;
    if !EVA_MESSAGE_NAMES
        .iter()
        .any(|name| name.eq_ignore_ascii_case(tok))
    {
        return Err(IniError::InvalidData);
    }
    *store = Some(tok.to_owned());
    Ok(())
}

pub fn parse_stealth_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = StealthUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("StealthDelay") {
            r.parse_duration_unsigned_int(&mut data.stealth_delay)
        } else if is("MoveThresholdSpeed") {
            r.parse_velocity_real(&mut data.stealth_speed)
        } else if is("StealthForbiddenConditions") {
            r.parse_bit_string32(&mut data.stealth_level, &STEALTH_LEVEL_NAMES)
        } else if is("HintDetectableConditions") {
            data.hint_detectable_states
                .parse_from_ini(r, &OBJECT_STATUS_NAMES)
        } else if is("RequiredStatus") {
            data.required_status.parse_from_ini(r, &OBJECT_STATUS_NAMES)
        } else if is("ForbiddenStatus") {
            data.forbidden_status
                .parse_from_ini(r, &OBJECT_STATUS_NAMES)
        } else if is("FriendlyOpacityMin") {
            r.parse_percent_to_real(&mut data.friendly_opacity_min)
        } else if is("FriendlyOpacityMax") {
            r.parse_percent_to_real(&mut data.friendly_opacity_max)
        } else if is("PulseFrequency") {
            r.parse_duration_unsigned_int(&mut data.pulse_frames)
        } else if is("DisguisesAsTeam") {
            r.parse_bool(&mut data.team_disguised)
        } else if is("RevealDistanceFromTarget") {
            r.parse_real(&mut data.reveal_distance_from_target)
        } else if is("OrderIdleEnemiesToAttackMeUponReveal") {
            r.parse_bool(&mut data.order_idle_enemies_to_attack_me_upon_reveal)
        } else if is("DisguiseFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.disguise_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if is("DisguiseRevealFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.disguise_reveal_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if is("DisguiseTransitionTime") {
            r.parse_duration_unsigned_int(&mut data.disguise_transition_frames)
        } else if is("DisguiseRevealTransitionTime") {
            r.parse_duration_unsigned_int(&mut data.disguise_reveal_transition_frames)
        } else if is("InnateStealth") {
            r.parse_bool(&mut data.innate_stealth)
        } else if is("UseRiderStealth") {
            r.parse_bool(&mut data.use_rider_stealth)
        } else if is("EnemyDetectionEvaEvent") {
            parse_eva(r, &mut data.enemy_detection_eva_event)
        } else if is("OwnDetectionEvaEvent") {
            parse_eva(r, &mut data.own_detection_eva_event)
        } else if is("BlackMarketCheckDelay") {
            r.parse_duration_unsigned_int(&mut data.black_market_check_frames)
        } else if is("GrantedBySpecialPower") {
            r.parse_bool(&mut data.granted_by_special_power)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_stealth_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<StealthUpdateModuleData>()
        .expect("StealthUpdate data type mismatch")
        .clone();
    Box::new(StealthUpdate::new(typed))
}
