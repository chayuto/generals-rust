//! `PKT-372`: `SpyVisionUpdate` (`SpyVisionUpdate.h` / `SpyVisionUpdate.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet, UpgradeFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{UpgradeMux, UpgradeMuxData};
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{
    Bool, NameKeyType, TeamId, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `SpyVisionUpdate` (`SpyVisionUpdateModuleData`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpyVisionUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_mux: UpgradeMuxData,
    pub needs_upgrade: Bool,
    pub self_powered: Bool,
    pub self_powered_duration: UnsignedInt,
    pub self_powered_interval: UnsignedInt,
    pub spy_on_kindof: KindOfMaskType,
}

impl SpyVisionUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut spy_on_kindof = KindOfMaskType::new();
        spy_on_kindof.flip();
        Self {
            tag_key,
            module_name_key,
            upgrade_mux: UpgradeMuxData::new(),
            needs_upgrade: FALSE,
            self_powered: FALSE,
            self_powered_duration: 0,
            self_powered_interval: 0,
            spy_on_kindof,
        }
    }
}

impl ModuleData for SpyVisionUpdateModuleData {
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
        FacetMask(0x081)
    }
}

pub fn parse_spy_vision_update_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SpyVisionUpdateModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if field.eq_ignore_ascii_case("NeedsUpgrade") {
            reader.parse_bool(&mut data.needs_upgrade)
        } else if field.eq_ignore_ascii_case("SelfPowered") {
            reader.parse_bool(&mut data.self_powered)
        } else if field.eq_ignore_ascii_case("SelfPoweredDuration") {
            reader.parse_duration_unsigned_int(&mut data.self_powered_duration)
        } else if field.eq_ignore_ascii_case("SelfPoweredInterval") {
            reader.parse_duration_unsigned_int(&mut data.self_powered_interval)
        } else if field.eq_ignore_ascii_case("SpyOnKindof") {
            data.spy_on_kindof.parse_from_ini(reader, &KINDOF_NAMES)
        } else if let Some(res) = data.upgrade_mux.parse_field(field, reader, parse_ctx) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

/// Runtime instance of `SpyVisionUpdate`.
pub struct SpyVisionUpdate {
    pub data: SpyVisionUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub upgrade: UpgradeMux,
    pub deactivate_frame: UnsignedInt,
    pub disabled_until_frame: UnsignedInt,
    pub currently_active: Bool,
    pub reset_timers_next_update: Bool,
}

impl SpyVisionUpdate {
    pub fn new(data: SpyVisionUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            upgrade: UpgradeMux::new(),
            deactivate_frame: 0,
            disabled_until_frame: 0,
            currently_active: FALSE,
            reset_timers_next_update: FALSE,
        }
    }

    fn do_activation_work(&mut self, sim: &mut dyn SimApi, actor_team: TeamId, setting: Bool) {
        if actor_team == TeamId::INVALID {
            return;
        }
        sim.set_enemy_units_vision_spied(actor_team, setting, &self.data.spy_on_kindof);
        self.currently_active = setting;
    }

    /// Activates spy vision for `duration` frames (`0` = permanent, `SpyVisionUpdate::activateSpyVision`).
    pub fn activate_spy_vision(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        duration: UnsignedInt,
    ) {
        let now = sim.frame();
        self.deactivate_frame = if duration == 0 {
            UnsignedInt::MAX
        } else {
            now.wrapping_add(duration)
        };

        if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            self.do_activation_work(sim, team, TRUE);
        }

        let sleep = if duration == 0 {
            UpdateSleepTime::FOREVER
        } else {
            UpdateSleepTime(duration)
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }

    /// Disables or wakes spy vision cleanly (`SpyVisionUpdate::setDisabledUntilFrame`).
    pub fn set_disabled_until_frame(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        frame: UnsignedInt,
    ) {
        let now = sim.frame();
        if frame > now {
            if self.currently_active != FALSE {
                if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                    self.do_activation_work(sim, team, FALSE);
                }
            }
            self.disabled_until_frame = frame;
            self.reset_timers_next_update = TRUE;
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(self.disabled_until_frame - now));
        } else {
            self.disabled_until_frame = now;
            self.reset_timers_next_update = TRUE;
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        }
    }

    #[inline]
    pub const fn get_disabled_until_frame(&self) -> UnsignedInt {
        self.disabled_until_frame
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.data.needs_upgrade != FALSE && self.upgrade.is_already_upgraded() == FALSE {
            self.activate_spy_vision(sim, ctx, self.data.self_powered_duration);
        }
    }
}

impl UpdateFacet for SpyVisionUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let now = sim.frame();

        if self.reset_timers_next_update != FALSE {
            self.reset_timers_next_update = FALSE;

            if self.data.self_powered != FALSE {
                if self.data.self_powered_interval == 0 {
                    if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                        self.do_activation_work(sim, team, TRUE);
                    }
                    return UpdateSleepTime::FOREVER;
                } else {
                    return UpdateSleepTime(self.data.self_powered_interval);
                }
            }
        }

        if self.currently_active != FALSE && self.deactivate_frame <= now {
            if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                self.do_activation_work(sim, team, FALSE);
            }
            self.deactivate_frame = 0;
        } else if self.currently_active == FALSE && self.data.self_powered != FALSE {
            if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                self.do_activation_work(sim, team, TRUE);
            }
            self.deactivate_frame = if self.data.self_powered_duration == 0 {
                UnsignedInt::MAX
            } else {
                now.wrapping_add(self.data.self_powered_duration)
            };
        }

        if self.data.self_powered != FALSE {
            if self.currently_active != FALSE {
                return UpdateSleepTime(self.data.self_powered_duration);
            } else {
                return UpdateSleepTime(self.data.self_powered_interval);
            }
        }

        UpdateSleepTime::FOREVER
    }
}

impl UpgradeFacet for SpyVisionUpdate {
    fn is_already_upgraded(&self) -> Bool {
        self.upgrade.is_already_upgraded()
    }

    fn attempt_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        if self.would_upgrade(sim, ctx, key_mask) != FALSE {
            self.give_self_upgrade(sim, ctx);
            return TRUE;
        }
        FALSE
    }

    fn would_upgrade(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .would_upgrade(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn reset_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .reset_upgrade(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn test_upgrade_conditions(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .test_upgrade_conditions(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn get_upgrade_activation_masks(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        activation: &mut UpgradeMaskType,
        conflicting: &mut UpgradeMaskType,
    ) {
        self.data
            .upgrade_mux
            .get_upgrade_activation_masks(sim.rules(), activation, conflicting);
    }

    fn perform_upgrade_fx(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.data.upgrade_mux.perform_upgrade_fx(sim, ctx.owner_slot);
    }

    fn is_sub_objects_upgrade(&self) -> Bool {
        FALSE
    }

    fn give_self_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.data.upgrade_mux.perform_upgrade_fx(sim, ctx.owner_slot);
        self.data
            .upgrade_mux
            .process_upgrade_removal(sim, ctx.owner_slot);
        self.upgrade_implementation(sim, ctx);
        self.upgrade.set_upgrade_executed(TRUE);
    }

    fn force_refresh_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.upgrade.is_already_upgraded() != FALSE {
            self.upgrade_implementation(sim, ctx);
        }
    }
}

impl BehaviorModule for SpyVisionUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.currently_active != FALSE {
            if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                self.do_activation_work(sim, team, FALSE);
            }
        }
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if self.currently_active != FALSE {
            self.do_activation_work(sim, old_team, FALSE);
            self.do_activation_work(sim, new_team, TRUE);
        }
    }

    fn on_disabled_edge(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, disabled: Bool) {
        if disabled != FALSE {
            self.set_disabled_until_frame(sim, ctx, UnsignedInt::MAX);
        } else {
            self.set_disabled_until_frame(sim, ctx, 0);
        }
    }

    fn apply_deferred_op(
        &mut self,
        op: DeferredSelfOp,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) {
        if let DeferredSelfOp::UpgradeResetKey(key) = op {
            if let Some(mask) = sim
                .rules()
                .upgrade_center
                .find_upgrade_by_key(key)
                .map(|t| t.upgrade_mask().clone())
            {
                let _ = UpgradeFacet::reset_upgrade(self, sim, ctx, &mask);
            }
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.deactivate_frame)?;
        xfer.xfer_bool(&mut self.currently_active)?;
        if version >= 2 {
            xfer.xfer_bool(&mut self.reset_timers_next_update)?;
            xfer.xfer_unsigned_int(&mut self.disabled_until_frame)?;
        }
        Ok(())
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

    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        Some(self)
    }

    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        Some(self)
    }
}

pub fn create_spy_vision_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SpyVisionUpdateModuleData>()
        .expect("SpyVisionUpdateModuleData mismatch")
        .clone();
    Box::new(SpyVisionUpdate::new(typed))
}
