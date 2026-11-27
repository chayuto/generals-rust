//! `PKT-142`: Shot history, continuous-fire ramp/cooldown, auto-reload-when-idle, and
//! looping fire-sound tracker (`GameLogic/FiringTracker.h`, `GameLogic/Object/FiringTracker.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSlot};
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::{BehaviorModule, ModuleData, ModuleSnapshotHeader, UpdateFacet};
use crate::object::weapon::WeaponTemplate;
use crate::object::weapon_actions;
use crate::rules::disabled_types::DisabledMaskType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::scheduler::{SleepyUpdatePhase, UpdateModuleScheduleState};
use crate::sim::{SimApi, SimView};
use zh_common::ini::LOGICFRAMES_PER_SECOND;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, UnsignedInt, UpdateSleepTime, XferVersion, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// Audio handle sentinel matching `AHSV_NoSound` (`Common/AudioHandleSpecialValues.h`).
pub const AHSV_NO_SOUND: UnsignedInt = 0;

/// Immutable module metadata for [`FiringTracker`] (`FiringTrackerModuleData`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiringTrackerModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl Default for FiringTrackerModuleData {
    fn default() -> Self {
        Self {
            tag_key: NameKeyType::INVALID,
            module_name_key: NameKeyType::INVALID,
        }
    }
}

impl ModuleData for FiringTrackerModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::UPDATE
    }

    fn update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Final
    }
}

/// Snapshot of the `Weapon` / `WeaponTemplate` properties inspected by `FiringTracker::shotFired`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FiredWeaponInfo {
    pub auto_reload_when_idle_frames: UnsignedInt,
    pub continuous_fire_coast_frames: UnsignedInt,
    pub possible_next_shot_frame: UnsignedInt,
    pub continuous_fire_one_shots_needed: Int,
    pub continuous_fire_two_shots_needed: Int,
    pub fire_sound_loop_time: UnsignedInt,
    pub fire_sound_key: NameKeyType,
}

impl FiredWeaponInfo {
    /// Builds [`FiredWeaponInfo`] from a [`WeaponTemplate`] and runtime weapon timing/audio data.
    #[inline]
    pub fn from_template(
        tmpl: &WeaponTemplate,
        possible_next_shot_frame: UnsignedInt,
        fire_sound_loop_time: UnsignedInt,
        fire_sound_key: NameKeyType,
    ) -> Self {
        Self {
            auto_reload_when_idle_frames: tmpl.auto_reload_when_idle_frames,
            continuous_fire_coast_frames: tmpl.continuous_fire_coast_frames,
            possible_next_shot_frame,
            continuous_fire_one_shots_needed: tmpl.continuous_fire_one_shots_needed,
            continuous_fire_two_shots_needed: tmpl.continuous_fire_two_shots_needed,
            fire_sound_loop_time,
            fire_sound_key,
        }
    }
}

/// Host abstraction decoupling [`FiringTracker`] from concrete `Object`/`GameLogic`/`AudioManager`.
pub trait FiringTrackerHost {
    fn current_frame(&self) -> UnsignedInt;
    fn object_id(&self) -> ObjectId;
    fn victim_has_faerie_fire(&self, victim_id: ObjectId) -> Bool;
    fn test_weapon_bonus_condition(&self, cond: WeaponBonusConditionType) -> Bool;
    fn set_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType);
    fn clear_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType);
    fn clear_and_set_model_condition_flags(
        &mut self,
        clr: &ModelConditionFlags,
        set: &ModelConditionFlags,
    );
    fn reload_all_ammo(&mut self, now: Bool);
    fn is_audio_playing(&self, handle: UnsignedInt) -> Bool;
    fn add_fire_sound(&mut self, sound_key: NameKeyType, object_id: ObjectId) -> UnsignedInt;
    fn add_rapid_fire_voice(&mut self, object_id: ObjectId) -> UnsignedInt;
    fn remove_audio_event(&mut self, handle: UnsignedInt);
    fn set_wake_frame(&mut self, wake_delay: UpdateSleepTime);
}

/// Tracks consecutive shots, continuous-fire speed-up/cooldown, idle auto-reload, and looping
/// fire audio (`FiringTracker` in `GameLogic/FiringTracker.h/.cpp`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FiringTracker {
    module_name_key: NameKeyType,
    schedule: UpdateModuleScheduleState,
    consecutive_shots: Int,
    victim_id: ObjectId,
    frame_to_start_cooldown: UnsignedInt,
    frame_to_force_reload: UnsignedInt,
    frame_to_stop_looping_sound: UnsignedInt,
    audio_handle: UnsignedInt,
    /// Where the module is filed, for the live sleep schedule its save carries.
    mod_slot: Option<ModuleSlot>,
}

impl Default for FiringTracker {
    fn default() -> Self {
        Self::new(NameKeyType::INVALID, 0)
    }
}

impl FiringTracker {
    /// Constructs a new `FiringTracker` sleeping forever (`UPDATE_SLEEP_FOREVER`), matching C++
    /// `FiringTracker::FiringTracker`.
    pub fn new(module_name_key: NameKeyType, current_frame: UnsignedInt) -> Self {
        let mut schedule = UpdateModuleScheduleState::new(SleepyUpdatePhase::Final);
        schedule.set_next_call_frame(current_frame.wrapping_add(UpdateSleepTime::FOREVER.0));
        Self {
            module_name_key,
            schedule,
            consecutive_shots: 0,
            victim_id: ObjectId::INVALID,
            frame_to_start_cooldown: 0,
            frame_to_force_reload: 0,
            frame_to_stop_looping_sound: 0,
            audio_handle: AHSV_NO_SOUND,
            mod_slot: None,
        }
    }

    /// [`FiringTracker::shot_fired`] on the tracker's own object.
    pub fn shot_fired_by(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        weapon_fired: &FiredWeaponInfo,
        victim_id: ObjectId,
    ) {
        self.mod_slot = Some(ctx.mod_slot);
        let mut host = SimFiringTrackerHost { sim, ctx };
        self.shot_fired(&mut host, weapon_fired, victim_id);
    }

    /// Releases any active looping audio handle (`FiringTracker::~FiringTracker`).
    pub fn on_destroy_audio(&mut self, host: &mut dyn FiringTrackerHost) {
        host.remove_audio_event(self.audio_handle);
        self.audio_handle = AHSV_NO_SOUND;
    }

    /// Returns the last victim ID shot at (`FiringTracker::getLastShotVictim`).
    #[inline]
    pub const fn get_last_shot_victim(&self) -> ObjectId {
        self.victim_id
    }

    /// Returns consecutive shots at `victim_id` (`FiringTracker::getNumConsecutiveShotsAtVictim`).
    #[inline]
    pub const fn get_num_consecutive_shots_at_victim(&self, victim: Option<ObjectId>) -> Int {
        match victim {
            Some(vid) if vid.0 == self.victim_id.0 => self.consecutive_shots,
            _ => 0,
        }
    }

    #[inline]
    pub const fn consecutive_shots(&self) -> Int {
        self.consecutive_shots
    }

    #[inline]
    pub const fn frame_to_start_cooldown(&self) -> UnsignedInt {
        self.frame_to_start_cooldown
    }

    #[inline]
    pub const fn frame_to_force_reload(&self) -> UnsignedInt {
        self.frame_to_force_reload
    }

    #[inline]
    pub const fn frame_to_stop_looping_sound(&self) -> UnsignedInt {
        self.frame_to_stop_looping_sound
    }

    #[inline]
    pub const fn audio_handle(&self) -> UnsignedInt {
        self.audio_handle
    }

    #[inline]
    pub const fn schedule(&self) -> &UpdateModuleScheduleState {
        &self.schedule
    }

    #[inline]
    pub fn schedule_mut(&mut self) -> &mut UpdateModuleScheduleState {
        &mut self.schedule
    }

    /// `FiringTracker::getDisabledTypesToProcess`: never disabled (`DISABLEDMASK_ALL`) so disabled
    /// units continue to spin down.
    #[inline]
    pub fn disabled_types_to_process(&self) -> DisabledMaskType {
        DisabledMaskType::new_set_all()
    }

    /// `FiringTracker::getUpdatePhase`: runs in `PHASE_FINAL` after normal update modules.
    #[inline]
    pub const fn update_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::Final
    }

    /// Records a weapon discharge (`FiringTracker::shotFired`).
    pub fn shot_fired(
        &mut self,
        host: &mut dyn FiringTrackerHost,
        weapon_fired: &FiredWeaponInfo,
        victim_id: ObjectId,
    ) {
        let now = host.current_frame();

        if victim_id != ObjectId::INVALID && host.victim_has_faerie_fire(victim_id) != 0 {
            if host.test_weapon_bonus_condition(WeaponBonusConditionType::TargetFaerieFire) == 0 {
                host.set_weapon_bonus_condition(WeaponBonusConditionType::TargetFaerieFire);
            }
        } else if host.test_weapon_bonus_condition(WeaponBonusConditionType::TargetFaerieFire) != 0
        {
            host.clear_weapon_bonus_condition(WeaponBonusConditionType::TargetFaerieFire);
        }

        if victim_id == self.victim_id {
            self.consecutive_shots = self.consecutive_shots.wrapping_add(1);
        } else if now < self.frame_to_start_cooldown {
            self.consecutive_shots = self.consecutive_shots.wrapping_add(1);
            self.victim_id = victim_id;
        } else {
            self.consecutive_shots = 1;
            self.victim_id = victim_id;
        }

        let auto_reload_delay = weapon_fired.auto_reload_when_idle_frames;
        if auto_reload_delay > 0 {
            self.frame_to_force_reload = now.wrapping_add(auto_reload_delay);
        }

        let coast = weapon_fired.continuous_fire_coast_frames;
        if coast != 0 {
            self.frame_to_start_cooldown = weapon_fired
                .possible_next_shot_frame
                .wrapping_add(coast);
        } else {
            self.frame_to_start_cooldown = 0;
        }

        let shots_needed_one = weapon_fired.continuous_fire_one_shots_needed;
        let shots_needed_two = weapon_fired.continuous_fire_two_shots_needed;

        if host.test_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean) != 0 {
            if self.consecutive_shots < shots_needed_one {
                self.cool_down(host);
            } else if self.consecutive_shots > shots_needed_two {
                self.speed_up(host);
            }
        } else if host.test_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast)
            != 0
        {
            if self.consecutive_shots < shots_needed_two {
                self.cool_down(host);
            }
        } else if self.consecutive_shots > shots_needed_one {
            self.speed_up(host);
        }

        let fire_sound_loop_time = weapon_fired.fire_sound_loop_time;
        let my_id = host.object_id();
        if fire_sound_loop_time != 0 {
            if self.frame_to_stop_looping_sound == 0 || host.is_audio_playing(self.audio_handle) == 0
            {
                self.audio_handle = host.add_fire_sound(weapon_fired.fire_sound_key, my_id);
            }
            self.frame_to_stop_looping_sound = now.wrapping_add(fire_sound_loop_time);
        } else {
            let _ = host.add_fire_sound(weapon_fired.fire_sound_key, my_id);
            self.frame_to_stop_looping_sound = 0;
        }

        let wake = self.calc_time_to_sleep(now);
        self.schedule
            .set_next_call_frame(now.wrapping_add(wake.0));
        host.set_wake_frame(wake);
    }

    /// Periodic update (`FiringTracker::update`).
    pub fn update_with_host(&mut self, host: &mut dyn FiringTrackerHost) -> UpdateSleepTime {
        let now = host.current_frame();

        if self.frame_to_force_reload != 0 && now >= self.frame_to_force_reload {
            host.reload_all_ammo(TRUE);
            self.frame_to_force_reload = 0;
        }

        if self.frame_to_stop_looping_sound != 0 && now >= self.frame_to_stop_looping_sound {
            host.remove_audio_event(self.audio_handle);
            self.audio_handle = AHSV_NO_SOUND;
            self.frame_to_stop_looping_sound = 0;
        }

        if self.frame_to_start_cooldown != 0 && now > self.frame_to_start_cooldown {
            self.frame_to_start_cooldown = now.wrapping_add(LOGICFRAMES_PER_SECOND as UnsignedInt);
            self.cool_down(host);
            return UpdateSleepTime(LOGICFRAMES_PER_SECOND as UnsignedInt);
        }

        self.calc_time_to_sleep(now)
    }

    /// Computes minimum sleep duration across active reload, cooldown, and looping-sound timers
    /// (`FiringTracker::calcTimeToSleep`).
    pub fn calc_time_to_sleep(&self, now: UnsignedInt) -> UpdateSleepTime {
        if self.frame_to_stop_looping_sound == 0
            && self.frame_to_start_cooldown == 0
            && self.frame_to_force_reload == 0
        {
            return UpdateSleepTime::FOREVER;
        }

        let mut sleep_time: UnsignedInt = UpdateSleepTime::FOREVER.0;

        if self.frame_to_stop_looping_sound != 0 {
            if self.frame_to_stop_looping_sound <= now {
                sleep_time = UpdateSleepTime::NONE.0;
            } else {
                let dt = self.frame_to_stop_looping_sound - now;
                if dt < sleep_time {
                    sleep_time = dt;
                }
            }
        }

        if self.frame_to_start_cooldown != 0 {
            if self.frame_to_start_cooldown <= now {
                sleep_time = UpdateSleepTime::NONE.0;
            } else {
                let dt = self.frame_to_start_cooldown - now;
                if dt < sleep_time {
                    sleep_time = dt;
                }
            }
        }

        if self.frame_to_force_reload != 0 {
            if self.frame_to_force_reload <= now {
                sleep_time = UpdateSleepTime::NONE.0;
            } else {
                let dt = self.frame_to_force_reload - now;
                if dt < sleep_time {
                    sleep_time = dt;
                }
            }
        }

        UpdateSleepTime(sleep_time)
    }

    fn speed_up(&mut self, host: &mut dyn FiringTrackerHost) {
        let mut clr = ModelConditionFlags::new();
        let mut set = ModelConditionFlags::new();

        if host.test_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast) != 0 {
            // Already at fastest tier; C++ emits empty clear/set call below.
        } else if host.test_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean)
            != 0
        {
            let _ = host.add_rapid_fire_voice(host.object_id());

            host.set_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast);
            set.set(ModelConditionFlagType::ContinuousFireFast as Int, 1);

            host.clear_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean);
            clr.set(ModelConditionFlagType::ContinuousFireMean as Int, 1);
            clr.set(ModelConditionFlagType::ContinuousFireSlow as Int, 1);
        } else {
            host.set_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean);
            set.set(ModelConditionFlagType::ContinuousFireMean as Int, 1);

            host.clear_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast);
            clr.set(ModelConditionFlagType::ContinuousFireFast as Int, 1);
            clr.set(ModelConditionFlagType::ContinuousFireSlow as Int, 1);
        }

        host.clear_and_set_model_condition_flags(&clr, &set);
    }

    fn cool_down(&mut self, host: &mut dyn FiringTrackerHost) {
        let mut clr = ModelConditionFlags::new();
        let mut set = ModelConditionFlags::new();

        if host.test_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast) != 0
            || host.test_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean) != 0
        {
            set.set(ModelConditionFlagType::ContinuousFireSlow as Int, 1);

            host.clear_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast);
            host.clear_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean);
            clr.set(ModelConditionFlagType::ContinuousFireFast as Int, 1);
            clr.set(ModelConditionFlagType::ContinuousFireMean as Int, 1);
        } else {
            host.clear_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireFast);
            host.clear_weapon_bonus_condition(WeaponBonusConditionType::ContinuousFireMean);
            clr.set(ModelConditionFlagType::ContinuousFireFast as Int, 1);
            clr.set(ModelConditionFlagType::ContinuousFireMean as Int, 1);
            clr.set(ModelConditionFlagType::ContinuousFireSlow as Int, 1);

            self.frame_to_start_cooldown = 0;
        }

        host.clear_and_set_model_condition_flags(&clr, &set);

        self.consecutive_shots = 0;
        self.victim_id = ObjectId::INVALID;
    }

    /// `FiringTracker::crc` -> delegates to `UpdateModule::crc` (empty in C++).
    pub fn crc_snapshot(&self, _xfer: &mut dyn Xfer) {}

    /// `FiringTracker::xfer` preserving exact C++ field order, versions, and `retail-xfer-save` guard.
    pub fn xfer_snapshot(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        #[cfg(feature = "retail-xfer-save")]
        const CURRENT_VERSION: XferVersion = 1;
        #[cfg(not(feature = "retail-xfer-save"))]
        const CURRENT_VERSION: XferVersion = 2;

        let mut version: XferVersion = CURRENT_VERSION;
        xfer.xfer_version(&mut version, CURRENT_VERSION)?;

        ModuleSnapshotHeader::xfer_update_module_header(xfer, &mut self.schedule)?;

        xfer.xfer_int(&mut self.consecutive_shots)?;
        xfer.xfer_object_id(&mut self.victim_id)?;
        xfer.xfer_unsigned_int(&mut self.frame_to_start_cooldown)?;

        if version >= 2 {
            xfer.xfer_unsigned_int(&mut self.frame_to_force_reload)?;
            xfer.xfer_unsigned_int(&mut self.frame_to_stop_looping_sound)?;
        }

        if xfer.mode() == XferMode::Load {
            self.schedule.set_update_phase(SleepyUpdatePhase::Final);
        }
        Ok(())
    }
}

impl BehaviorModule for FiringTracker {
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.crc_snapshot(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        if xfer.mode() == XferMode::Save {
            if let Some(live) = self.mod_slot.and_then(|s| sim.module_schedule(s)) {
                self.schedule = live;
            }
        }
        self.xfer_snapshot(xfer)
    }

    fn as_firing_tracker(&self) -> Option<&FiringTracker> {
        Some(self)
    }

    fn as_firing_tracker_mut(&mut self) -> Option<&mut FiringTracker> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
}

impl UpdateFacet for FiringTracker {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.mod_slot = Some(ctx.mod_slot);
        let mut host = SimFiringTrackerHost { sim, ctx };
        self.update_with_host(&mut host)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        self.disabled_types_to_process()
    }

    fn get_update_phase(&self) -> SleepyUpdatePhase {
        self.update_phase()
    }
}

struct SimFiringTrackerHost<'a> {
    sim: &'a mut dyn SimApi,
    ctx: ModuleContext,
}

impl FiringTrackerHost for SimFiringTrackerHost<'_> {
    fn current_frame(&self) -> UnsignedInt {
        self.sim.frame()
    }

    fn object_id(&self) -> ObjectId {
        self.ctx.owner_id
    }

    fn victim_has_faerie_fire(&self, victim_id: ObjectId) -> Bool {
        if let Some(slot) = self.sim.find_object(victim_id) {
            if let Some(obj) = self.sim.object(slot) {
                return obj
                    .status
                    .test(crate::rules::object_status_types::ObjectStatusType::FaerieFire as Int);
            }
        }
        zh_common::types::FALSE
    }

    fn test_weapon_bonus_condition(&self, cond: WeaponBonusConditionType) -> Bool {
        if let Some(obj) = self.sim.object(self.ctx.owner_slot) {
            Bool::from((obj.weapon_bonus_condition & (1u32 << (cond as u32))) != 0)
        } else {
            zh_common::types::FALSE
        }
    }

    fn set_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType) {
        self.sim.set_weapon_bonus_condition(self.ctx.owner_slot, cond);
    }

    fn clear_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType) {
        self.sim.clear_weapon_bonus_condition(self.ctx.owner_slot, cond);
    }

    fn clear_and_set_model_condition_flags(
        &mut self,
        _clr: &ModelConditionFlags,
        _set: &ModelConditionFlags,
    ) {
        // Headless SimApi models drawable condition flags on the client side.
    }

    fn reload_all_ammo(&mut self, now: Bool) {
        weapon_actions::reload_all_ammo(self.sim, self.ctx.owner_slot, now);
    }

    fn is_audio_playing(&self, _handle: UnsignedInt) -> Bool {
        zh_common::types::FALSE
    }

    fn add_fire_sound(&mut self, _sound_key: NameKeyType, _object_id: ObjectId) -> UnsignedInt {
        AHSV_NO_SOUND
    }

    fn add_rapid_fire_voice(&mut self, _object_id: ObjectId) -> UnsignedInt {
        AHSV_NO_SOUND
    }

    fn remove_audio_event(&mut self, _handle: UnsignedInt) {}

    fn set_wake_frame(&mut self, wake_delay: UpdateSleepTime) {
        self.sim.set_wake_frame(self.ctx.mod_slot, wake_delay);
    }
}
