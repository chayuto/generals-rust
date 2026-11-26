//! `PKT-330`: `FlammableUpdateModuleData` and `FlammableUpdate`
//! (`FlammableUpdate.h`, `FlammableUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-330_FlammableUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt330_327_fire.rs`.
//!
//! The burning sound is client audio: its name is parsed and nothing is played.

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, DamageFacet, FlammableFacet, ModuleData, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// Version 2 appends the flame source; `RETAIL_COMPATIBLE_XFER_SAVE` stays at 1.
#[cfg(feature = "retail-xfer-save")]
const XFER_VERSION: XferVersion = 1;
#[cfg(not(feature = "retail-xfer-save"))]
const XFER_VERSION: XferVersion = 2;

const FS_NORMAL: Int = 0;
const FS_AFLAME: Int = 1;
const FS_BURNED: Int = 2;

/// INI template data matching `FlammableUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct FlammableUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub burned_delay: UnsignedInt,
    pub aflame_duration: UnsignedInt,
    pub aflame_damage_delay: UnsignedInt,
    pub aflame_damage_amount: Int,
    pub burning_sound_name: String,
    pub flame_damage_limit: Real,
    pub flame_damage_expiration_delay: UnsignedInt,
}

impl FlammableUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            burned_delay: 0,
            aflame_duration: 0,
            aflame_damage_delay: 0,
            aflame_damage_amount: 0,
            burning_sound_name: String::new(),
            flame_damage_limit: 20.0,
            // `LOGICFRAMES_PER_SECOND * 2`.
            flame_damage_expiration_delay: 60,
        }
    }
}

impl ModuleData for FlammableUpdateModuleData {
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
        FacetMask(0x005)
    }
}

/// Runtime instance of `FlammableUpdate`.
pub struct FlammableUpdate {
    pub data: FlammableUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// `FlammabilityStatusType`: 0 normal, 1 aflame, 2 burned.
    pub status: Int,
    pub aflame_end_frame: UnsignedInt,
    pub burned_end_frame: UnsignedInt,
    pub damage_end_frame: UnsignedInt,
    pub flame_damage_limit: Real,
    pub flame_source: ObjectId,
    pub last_flame_damage_dealt: UnsignedInt,
}

impl FlammableUpdate {
    pub fn new(data: FlammableUpdateModuleData) -> Self {
        Self {
            status: FS_NORMAL,
            aflame_end_frame: 0,
            burned_end_frame: 0,
            damage_end_frame: 0,
            flame_damage_limit: data.flame_damage_limit,
            flame_source: ObjectId::INVALID,
            last_flame_damage_dealt: 0,
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `calcSleepTime`.
    fn calc_sleep_time(&self, now: UnsignedInt) -> UpdateSleepTime {
        if self.status == FS_AFLAME && self.aflame_end_frame != 0 && self.aflame_end_frame > now {
            let mut soonest = self.aflame_end_frame;
            for frame in [self.burned_end_frame, self.damage_end_frame] {
                if frame != 0 && frame < soonest && frame > now {
                    soonest = frame;
                }
            }
            UpdateSleepTime(soonest - now)
        } else {
            UpdateSleepTime::FOREVER
        }
    }

    /// `doAflameDamage`. The damage reaches this module's own `onDamage`, which cannot be
    /// dispatched while the module is executing. It is applied afterwards with the object
    /// status of its turn in the callback list; it changes only this module.
    fn do_aflame_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let mut info = DamageInfo::default();
        info.input.amount = self.data.aflame_damage_amount as Real;
        info.input.source_id = self.flame_source;
        info.input.damage_type = DamageType::Flame;
        info.input.death_type = DeathType::Burned;
        if let Some(status) = sim.attempt_damage_watching(ctx.owner_slot, &mut info, ctx.mod_slot) {
            self.damaged(sim, ctx, &info, &status);
        }
    }

    /// `onDamage`, with the object status it sees.
    fn damaged(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
        object_status: &ObjectStatusMaskType,
    ) {
        if !matches!(
            dmg.input.damage_type,
            DamageType::Flame | DamageType::ParticleBeam
        ) {
            return;
        }
        let now = sim.frame();
        if now.wrapping_sub(self.data.flame_damage_expiration_delay) > self.last_flame_damage_dealt
        {
            self.flame_damage_limit = self.data.flame_damage_limit;
        }
        self.last_flame_damage_dealt = now;
        #[cfg(feature = "retail-crc")]
        {
            self.flame_source = sim
                .object(ctx.owner_slot)
                .map_or(ObjectId::INVALID, |o| o.id);
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            self.flame_source = dmg.input.source_id;
        }
        if object_status.test(ObjectStatusType::Aflame.as_i32()) == FALSE
            && object_status.test(ObjectStatusType::Burned.as_i32()) == FALSE
        {
            self.flame_damage_limit -= dmg.output.actual_damage_dealt;
            if self.flame_damage_limit <= 0.0 {
                self.try_to_ignite(sim, ctx);
            }
        }
    }
}

impl FlammableFacet for FlammableUpdate {
    fn would_ignite(&self) -> Bool {
        if self.status == FS_NORMAL {
            TRUE
        } else {
            FALSE
        }
    }

    fn try_to_ignite(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.status != FS_NORMAL {
            return;
        }
        let owner = ctx.owner_slot;
        sim.set_status(owner, ObjectStatusType::Aflame);
        sim.set_body_aflame(owner, TRUE);
        sim.set_model_condition_state(owner, ModelConditionFlagType::Aflame);
        sim.start_fire_spreading(owner);
        self.status = FS_AFLAME;

        let now = sim.frame();
        self.aflame_end_frame = now.wrapping_add(self.data.aflame_duration);
        let after = |delay: UnsignedInt| {
            if delay != 0 {
                now.wrapping_add(delay)
            } else {
                0
            }
        };
        self.burned_end_frame = after(self.data.burned_delay);
        self.damage_end_frame = after(self.data.aflame_damage_delay);
        sim.set_wake_frame(ctx.mod_slot, self.calc_sleep_time(now));
    }
}

impl DamageFacet for FlammableUpdate {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let Some(status) = sim.object(ctx.owner_slot).map(|o| o.status.clone()) else {
            return;
        };
        self.damaged(sim, ctx, dmg, &status);
    }
}

impl UpdateFacet for FlammableUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        if self.damage_end_frame != 0 && now >= self.damage_end_frame {
            self.damage_end_frame = now.wrapping_add(self.data.aflame_damage_delay);
            self.do_aflame_damage(sim, ctx);
        }
        if self.burned_end_frame != 0 && now >= self.burned_end_frame {
            sim.set_status(owner, ObjectStatusType::Burned);
            sim.set_model_condition_state(owner, ModelConditionFlagType::Smoldering);
        }
        if self.aflame_end_frame != 0 && now >= self.aflame_end_frame {
            let burned = sim
                .object(owner)
                .is_some_and(|o| o.status.test(ObjectStatusType::Burned.as_i32()) != FALSE);
            self.status = if burned { FS_BURNED } else { FS_NORMAL };
            sim.clear_status(owner, ObjectStatusType::Aflame);
            sim.set_body_aflame(owner, FALSE);
            sim.clear_model_condition_state(owner, ModelConditionFlagType::Aflame);
        }
        self.calc_sleep_time(sim.frame())
    }
}

impl BehaviorModule for FlammableUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, XFER_VERSION)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let mut status = self.status.to_le_bytes();
        xfer.xfer_user(&mut status)?;
        self.status = Int::from_le_bytes(status);
        xfer.xfer_unsigned_int(&mut self.aflame_end_frame)?;
        xfer.xfer_unsigned_int(&mut self.burned_end_frame)?;
        xfer.xfer_unsigned_int(&mut self.damage_end_frame)?;
        xfer.xfer_real(&mut self.flame_damage_limit)?;
        xfer.xfer_unsigned_int(&mut self.last_flame_damage_dealt)?;
        if version >= 2 {
            xfer.xfer_object_id(&mut self.flame_source)?;
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

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }

    fn as_flammable(&self) -> Option<&dyn FlammableFacet> {
        Some(self)
    }

    fn as_flammable_mut(&mut self) -> Option<&mut dyn FlammableFacet> {
        Some(self)
    }
}

pub fn parse_flammable_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = FlammableUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("BurnedDelay") {
            r.parse_duration_unsigned_int(&mut data.burned_delay)
        } else if field.eq_ignore_ascii_case("AflameDuration") {
            r.parse_duration_unsigned_int(&mut data.aflame_duration)
        } else if field.eq_ignore_ascii_case("AflameDamageDelay") {
            r.parse_duration_unsigned_int(&mut data.aflame_damage_delay)
        } else if field.eq_ignore_ascii_case("AflameDamageAmount") {
            r.parse_int(&mut data.aflame_damage_amount)
        } else if field.eq_ignore_ascii_case("BurningSoundName") {
            r.parse_ascii_string(&mut data.burning_sound_name)
        } else if field.eq_ignore_ascii_case("FlameDamageLimit") {
            r.parse_real(&mut data.flame_damage_limit)
        } else if field.eq_ignore_ascii_case("FlameDamageExpiration") {
            r.parse_duration_unsigned_int(&mut data.flame_damage_expiration_delay)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_flammable_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<FlammableUpdateModuleData>()
        .expect("FlammableUpdate data type mismatch")
        .clone();
    Box::new(FlammableUpdate::new(typed))
}
