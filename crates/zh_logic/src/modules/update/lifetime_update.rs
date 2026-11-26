//! `PKT-339`: `LifetimeUpdateModuleData` and `LifetimeUpdate`
//! (`LifetimeUpdate.h`, `LifetimeUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-339_LifetimeUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_a.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::FALSE;
use zh_common::types::{Int, NameKeyType, UnsignedInt, UpdateSleepTime};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `LifetimeUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct LifetimeUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub min_frames: UnsignedInt,
    pub max_frames: UnsignedInt,
}

impl LifetimeUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            min_frames: 0,
            max_frames: 0,
        }
    }
}

impl ModuleData for LifetimeUpdateModuleData {
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
        FacetMask(0x001)
    }
}

/// Runtime instance of `LifetimeUpdate`.
pub struct LifetimeUpdate {
    pub data: LifetimeUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub die_frame: UnsignedInt,
}

impl LifetimeUpdate {
    pub fn new(data: LifetimeUpdateModuleData) -> Self {
        Self {
            die_frame: 0,
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `calcSleepDelay`.
    fn calc_sleep_delay(
        &mut self,
        sim: &mut dyn SimApi,
        min_frames: UnsignedInt,
        max_frames: UnsignedInt,
    ) -> UnsignedInt {
        let delay = (sim.rng_logic_int(min_frames as Int, max_frames as Int) as UnsignedInt).max(1);
        self.die_frame = sim.frame().wrapping_add(delay);
        delay
    }

    /// `setLifetimeRange`.
    pub fn set_lifetime_range(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        min_frames: UnsignedInt,
        max_frames: UnsignedInt,
    ) {
        let delay = self.calc_sleep_delay(sim, min_frames, max_frames);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(delay));
    }
}

impl UpdateFacet for LifetimeUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        // `Object::kill` with its default damage and death types.
        kill_object(
            sim,
            ctx.owner_slot,
            DamageType::Unresistable,
            DeathType::Normal,
        );
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for LifetimeUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    /// C++ constructor: hulks obey the script's lifetime override.
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        let hulk_override = sim.hulk_max_lifetime_override();
        let is_hulk = sim.object_is_kind_of(ctx.owner_slot, KindOfType::Hulk) != FALSE;
        let (min_frames, max_frames) = if is_hulk && hulk_override != -1 {
            (hulk_override as UnsignedInt, hulk_override as UnsignedInt)
        } else {
            (self.data.min_frames, self.data.max_frames)
        };
        self.set_lifetime_range(sim, ctx, min_frames, max_frames);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.die_frame)
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
}

pub fn parse_lifetime_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = LifetimeUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("MinLifetime") {
            r.parse_duration_unsigned_int(&mut data.min_frames)
        } else if field.eq_ignore_ascii_case("MaxLifetime") {
            r.parse_duration_unsigned_int(&mut data.max_frames)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_lifetime_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<LifetimeUpdateModuleData>()
        .expect("LifetimeUpdate data type mismatch")
        .clone();
    Box::new(LifetimeUpdate::new(typed))
}
