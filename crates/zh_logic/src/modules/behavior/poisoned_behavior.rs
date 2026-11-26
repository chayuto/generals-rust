//! `PKT-287`: `PoisonedBehaviorModuleData` and `PoisonedBehavior`
//! (`PoisonedBehavior.h`, `PoisonedBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-287_PoisonedBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c1.rs`.
//!
//! Not modelled: the drawable's poisoned tint (client only).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, DamageFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

#[cfg(feature = "retail-xfer-save")]
const XFER_VERSION: zh_common::types::XferVersion = 2;
#[cfg(not(feature = "retail-xfer-save"))]
const XFER_VERSION: zh_common::types::XferVersion = 3;

/// INI template data matching `PoisonedBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PoisonedBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub poison_damage_interval: UnsignedInt,
    pub poison_duration: UnsignedInt,
}

impl PoisonedBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            poison_damage_interval: 0,
            poison_duration: 0,
        }
    }
}

impl ModuleData for PoisonedBehaviorModuleData {
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

/// Runtime instance of `PoisonedBehavior` (`PKT-287`).
pub struct PoisonedBehavior {
    pub data: PoisonedBehaviorModuleData,
    pub poison_damage_frame: UnsignedInt,
    pub poison_overall_stop_frame: UnsignedInt,
    pub poison_damage_amount: Real,
    pub poison_source: ObjectId,
    pub death_type: DeathType,
    pub schedule: UpdateScheduleTracker,
}

impl PoisonedBehavior {
    pub fn new(data: PoisonedBehaviorModuleData) -> Self {
        Self {
            data,
            poison_damage_frame: 0,
            poison_overall_stop_frame: 0,
            poison_damage_amount: 0.0,
            poison_source: ObjectId::INVALID,
            death_type: DeathType::Poisoned,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `PoisonedBehavior::calcSleepTime` with `UpdateModule::frameToSleepTime`.
    fn calc_sleep_time(&self, now: UnsignedInt) -> UpdateSleepTime {
        if self.poison_overall_stop_frame == 0 || self.poison_overall_stop_frame == now {
            return UpdateSleepTime::FOREVER;
        }
        let frame = self.poison_damage_frame.min(self.poison_overall_stop_frame);
        if frame > now {
            UpdateSleepTime(frame - now)
        } else {
            UpdateSleepTime::NONE
        }
    }

    fn start_poisoned_effects(&mut self, now: UnsignedInt, dmg: &DamageInfo) {
        self.poison_damage_amount = dmg.output.actual_damage_dealt;
        #[cfg(not(feature = "retail-crc"))]
        {
            self.poison_source = dmg.input.source_id;
        }
        self.poison_overall_stop_frame = now.wrapping_add(self.data.poison_duration);

        // Re-poisoning keeps a running damage timer.
        let next = now.wrapping_add(self.data.poison_damage_interval);
        self.poison_damage_frame = if self.poison_damage_frame != 0 {
            self.poison_damage_frame.min(next)
        } else {
            next
        };
        self.death_type = dmg.input.death_type;
    }

    fn stop_poisoned_effects(&mut self) {
        self.poison_damage_frame = 0;
        self.poison_overall_stop_frame = 0;
        self.poison_damage_amount = 0.0;
        self.poison_source = ObjectId::INVALID;
    }
}

impl DamageFacet for PoisonedBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        if dmg.input.damage_type == DamageType::Poison {
            let now = sim.frame();
            self.start_poisoned_effects(now, dmg);
            sim.set_wake_frame(ctx.mod_slot, self.calc_sleep_time(now));
        }
    }

    fn on_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &mut DamageInfo) {
        self.stop_poisoned_effects();
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }
}

impl UpdateFacet for PoisonedBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let now = sim.frame();
        if self.poison_overall_stop_frame == 0 {
            return UpdateSleepTime::FOREVER;
        }
        let owner = ctx.owner_slot;

        if self.poison_damage_frame != 0 && now >= self.poison_damage_frame {
            let mut damage = DamageInfo::default();
            damage.input.amount = self.poison_damage_amount;
            damage.input.source_id = self.poison_source;
            // Not poison, as that would infect us again; the effect still reads as poison.
            damage.input.damage_type = DamageType::Unresistable;
            damage.input.damage_fx_override = DamageType::Poison;
            damage.input.death_type = self.death_type;
            sim.attempt_damage(owner, &mut damage);
            self.poison_damage_frame = now.wrapping_add(self.data.poison_damage_interval);
        }

        // A unit killed by the poison keeps pulsing.
        if self.poison_overall_stop_frame != 0
            && now >= self.poison_overall_stop_frame
            && sim
                .object(owner)
                .is_some_and(|o| o.is_effectively_dead() == FALSE)
        {
            self.stop_poisoned_effects();
        }

        self.calc_sleep_time(now)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

impl BehaviorModule for PoisonedBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, XFER_VERSION)?;
        // `xfer_module_version` reports the version that was actually loaded.
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.poison_damage_frame)?;
        xfer.xfer_unsigned_int(&mut self.poison_overall_stop_frame)?;
        xfer.xfer_real(&mut self.poison_damage_amount)?;
        if version >= 2 {
            // `xferUser` of the four-byte enum.
            let mut death = (self.death_type as Int).to_le_bytes();
            xfer.xfer_user(&mut death)?;
            if let Some(d) = DeathType::from_i32(Int::from_le_bytes(death)) {
                self.death_type = d;
            }
        }
        if version >= 3 {
            xfer.xfer_object_id(&mut self.poison_source)?;
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
}

pub fn parse_poisoned_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = PoisonedBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("PoisonDamageInterval") {
            r.parse_duration_unsigned_int(&mut data.poison_damage_interval)
        } else if field.eq_ignore_ascii_case("PoisonDuration") {
            r.parse_duration_unsigned_int(&mut data.poison_duration)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_poisoned_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<PoisonedBehaviorModuleData>()
        .expect("PoisonedBehavior data type mismatch")
        .clone();
    Box::new(PoisonedBehavior::new(typed))
}
