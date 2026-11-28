//! `PKT-341`: `MissileLauncherBuildingUpdateModuleData` and `MissileLauncherBuildingUpdate`
//! (`MissileLauncherBuildingUpdate.h`, `MissileLauncherBuildingUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-341_MissileLauncherBuildingUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt341_350_309_support.rs`.
//!
//! The open-door audio loop and the drawable's animation loop duration are client state
//! and are not emitted.

use crate::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, ModuleData, SpecialPowerCaller, SpecialPowerUpdateFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `DoorStateType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum DoorState {
    #[default]
    Closed = 0,
    Opening = 1,
    Open = 2,
    WaitingToClose = 3,
    Closing = 4,
}

impl DoorState {
    fn from_i32(v: Int) -> Self {
        match v {
            1 => Self::Opening,
            2 => Self::Open,
            3 => Self::WaitingToClose,
            4 => Self::Closing,
            _ => Self::Closed,
        }
    }
}

/// INI template data matching `MissileLauncherBuildingUpdateModuleData`. An FX list is
/// kept by name, empty for none.
#[derive(Debug, Clone, PartialEq)]
pub struct MissileLauncherBuildingUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub special_power_name: String,
    pub door_open_time: UnsignedInt,
    pub door_wait_open_time: UnsignedInt,
    pub door_closing_time: UnsignedInt,
    pub opening_fx: String,
    pub open_fx: String,
    pub waiting_to_close_fx: String,
    pub closing_fx: String,
    pub closed_fx: String,
    pub open_idle_audio: String,
}

impl MissileLauncherBuildingUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            special_power_name: String::new(),
            door_open_time: 0,
            door_wait_open_time: 0,
            door_closing_time: 0,
            opening_fx: String::new(),
            open_fx: String::new(),
            waiting_to_close_fx: String::new(),
            closing_fx: String::new(),
            closed_fx: String::new(),
            open_idle_audio: String::new(),
        }
    }
}

impl ModuleData for MissileLauncherBuildingUpdateModuleData {
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

/// Runtime instance of `MissileLauncherBuildingUpdate`.
pub struct MissileLauncherBuildingUpdate {
    pub data: MissileLauncherBuildingUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// `m_specialPowerModule`, found on first use. Not saved.
    pub special_power_module: Option<ModuleSlot>,
    pub door_state: DoorState,
    pub timeout_state: DoorState,
    pub timeout_frame: UnsignedInt,
}

fn door_flags(flags: &[ModelConditionFlagType]) -> ModelConditionFlags {
    let mut mask = ModelConditionFlags::new();
    for flag in flags {
        mask.set(flag.as_i32(), 1);
    }
    mask
}

impl MissileLauncherBuildingUpdate {
    pub fn new(data: MissileLauncherBuildingUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            special_power_module: None,
            door_state: DoorState::Closed,
            timeout_state: DoorState::Closed,
            timeout_frame: 0,
        }
    }

    fn template_id(&self, sim: &dyn SimView) -> Option<UnsignedInt> {
        sim.rules()
            .special_power_store
            .find_special_power_template(&self.data.special_power_name)
            .map(|t| t.id())
    }

    /// `(isReady(), getReadyFrame())` of the cached special power module.
    fn power_ready(&self, sim: &mut dyn SimApi) -> (Bool, UnsignedInt) {
        let mut out = (FALSE, 0);
        if let Some(module) = self.special_power_module {
            sim.with_special_power_module(module, &mut |spm, sim, ctx| {
                out = (spm.is_ready(sim, ctx), spm.get_ready_frame(sim, ctx));
            });
        }
        out
    }

    /// `switchToState`.
    pub fn switch_to_state(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, dst: DoorState) {
        use ModelConditionFlagType as F;
        if self.door_state == dst {
            return;
        }
        let now = sim.frame();
        let d = &self.data;
        let (clr, set, fx): (&[F], &[F], &String) = match dst {
            DoorState::Closed => (
                &[
                    F::Door1WaitingToClose,
                    F::Door1Closing,
                    F::Door1Opening,
                    F::Door1WaitingOpen,
                ],
                &[],
                &d.closed_fx,
            ),
            DoorState::Opening => (
                &[F::Door1WaitingToClose, F::Door1Closing, F::Door1WaitingOpen],
                &[F::Door1Opening],
                &d.opening_fx,
            ),
            DoorState::Open => (
                &[F::Door1WaitingToClose, F::Door1Closing, F::Door1Opening],
                &[F::Door1WaitingOpen],
                &d.open_fx,
            ),
            DoorState::WaitingToClose => (
                &[F::Door1Closing, F::Door1Opening, F::Door1WaitingOpen],
                &[F::Door1WaitingToClose],
                &d.waiting_to_close_fx,
            ),
            DoorState::Closing => (
                &[F::Door1WaitingToClose, F::Door1WaitingOpen, F::Door1Opening],
                &[F::Door1Closing],
                &d.closing_fx,
            ),
        };
        let fx = fx.clone();
        match dst {
            DoorState::Closed => {
                self.timeout_frame = 0;
                self.timeout_state = DoorState::Closed;
            }
            DoorState::Opening => {
                self.timeout_frame = self.power_ready(sim).1.wrapping_sub(1);
                self.timeout_state = DoorState::Open;
            }
            DoorState::Open => {
                self.timeout_frame = 0;
                self.timeout_state = DoorState::Open;
            }
            DoorState::WaitingToClose => {
                self.timeout_frame = now.wrapping_add(self.data.door_wait_open_time);
                self.timeout_state = DoorState::Closing;
            }
            DoorState::Closing => {
                self.timeout_frame = now.wrapping_add(self.data.door_closing_time);
                let delta = self.power_ready(sim).1.wrapping_sub(now) as Int;
                // `delta / 2` is signed, then converted for the unsigned comparison.
                let halfway = now.wrapping_add((delta / 2) as UnsignedInt);
                if self.timeout_frame > halfway {
                    self.timeout_frame = halfway;
                }
                self.timeout_state = DoorState::Closed;
            }
        }
        if !fx.is_empty() {
            let pos = sim
                .object(owner)
                .map_or_else(Coord3D::default, |o| *o.thing.get_position());
            sim.trigger_fx_list(&fx, &pos);
        }
        self.door_state = dst;
        sim.clear_and_set_model_condition_flags(owner, &door_flags(clr), &door_flags(set));
    }
}

impl SpecialPowerUpdateFacet for MissileLauncherBuildingUpdate {
    fn initiate_intent_to_do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        mut caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        _target_obj: Option<ObjectSlot>,
        _target_pos: Option<&Coord3D>,
        _command_options: UnsignedInt,
    ) -> Bool {
        let own = self.template_id(&*sim);
        if own != Some(template_id) {
            return FALSE;
        }
        let owner = ctx.owner_slot;
        let under_construction = sim
            .object(owner)
            .is_some_and(|o| o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE);
        if under_construction {
            let module = self
                .special_power_module
                .or_else(|| sim.find_special_power_module(owner, template_id));
            if let Some(module) = module {
                caller.with(sim, module, &mut |spm, sim, spm_ctx| {
                    spm.set_ready_frame(sim, spm_ctx, 0xFFFF_FFFF);
                });
            }
            return FALSE;
        }
        if self.special_power_module.is_none() {
            self.special_power_module = sim.find_special_power_module(owner, template_id);
        }
        let Some(module) = self.special_power_module else {
            return FALSE;
        };
        let mut module_template = None;
        caller.with(sim, module, &mut |spm, sim, spm_ctx| {
            module_template = Some(spm.get_special_power_id(&*sim, spm_ctx));
        });
        if module_template != Some(template_id) {
            return FALSE;
        }
        self.switch_to_state(sim, owner, DoorState::WaitingToClose);
        TRUE
    }

    fn is_active(&self) -> Bool {
        Bool::from(self.door_state != self.timeout_state)
    }
}

impl UpdateFacet for MissileLauncherBuildingUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        if sim
            .object(owner)
            .is_some_and(|o| o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE)
        {
            return UpdateSleepTime::NONE;
        }
        if self.special_power_module.is_none() {
            if let Some(id) = self.template_id(&*sim) {
                self.special_power_module = sim.find_special_power_module(owner, id);
            }
        }
        if self.special_power_module.is_some() {
            let ready_frame = self.power_ready(sim).1;
            let when_to_start_opening = if ready_frame >= self.data.door_open_time {
                ready_frame - self.data.door_open_time
            } else {
                0
            };
            if self.timeout_frame != 0 && now > self.timeout_frame {
                self.switch_to_state(sim, owner, self.timeout_state);
            }
            if self.door_state != DoorState::Open && self.power_ready(sim).0 != FALSE {
                self.switch_to_state(sim, owner, DoorState::Open);
            } else if self.door_state == DoorState::Closed && now >= when_to_start_opening {
                self.switch_to_state(sim, owner, DoorState::Opening);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for MissileLauncherBuildingUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// The `SpecialPowerUpdateModule` version byte is skipped: the reference calls
    /// `UpdateModule::xfer` directly.
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let mut door = (self.door_state as Int).to_le_bytes();
        xfer.xfer_user(&mut door)?;
        self.door_state = DoorState::from_i32(Int::from_le_bytes(door));
        let mut timeout = (self.timeout_state as Int).to_le_bytes();
        xfer.xfer_user(&mut timeout)?;
        self.timeout_state = DoorState::from_i32(Int::from_le_bytes(timeout));
        xfer.xfer_unsigned_int(&mut self.timeout_frame)
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

pub fn parse_missile_launcher_building_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = MissileLauncherBuildingUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        let name = |r: &mut IniReader<'_>, out: &mut String| {
            r.parse_ascii_string(out)?;
            if out.eq_ignore_ascii_case("None") {
                out.clear();
            }
            Ok(())
        };
        if is("SpecialPowerTemplate") {
            name(r, &mut data.special_power_name)
        } else if is("DoorOpenTime") {
            r.parse_duration_unsigned_int(&mut data.door_open_time)
        } else if is("DoorWaitOpenTime") {
            r.parse_duration_unsigned_int(&mut data.door_wait_open_time)
        } else if is("DoorCloseTime") {
            r.parse_duration_unsigned_int(&mut data.door_closing_time)
        } else if is("DoorOpeningFX") {
            name(r, &mut data.opening_fx)
        } else if is("DoorOpenFX") {
            name(r, &mut data.open_fx)
        } else if is("DoorWaitingToCloseFX") {
            name(r, &mut data.waiting_to_close_fx)
        } else if is("DoorClosingFX") {
            name(r, &mut data.closing_fx)
        } else if is("DoorClosedFX") {
            name(r, &mut data.closed_fx)
        } else if is("DoorOpenIdleAudio") {
            name(r, &mut data.open_idle_audio)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_missile_launcher_building_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<MissileLauncherBuildingUpdateModuleData>()
        .expect("MissileLauncherBuildingUpdate data type mismatch")
        .clone();
    Box::new(MissileLauncherBuildingUpdate::new(typed))
}
