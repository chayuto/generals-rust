//! `PKT-316`: `DeletionUpdateModuleData` and `DeletionUpdate`
//! (`DeletionUpdate.h`, `DeletionUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-316_DeletionUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_a.rs`.
//!
//! The `RTS_DEBUG && CRISS_CROSS_GEOMETRY` developer branch is not ported.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, UnsignedInt, UpdateSleepTime};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `DeletionUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct DeletionUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub min_frames: UnsignedInt,
    pub max_frames: UnsignedInt,
}

impl DeletionUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            min_frames: 0,
            max_frames: 0,
        }
    }
}

impl ModuleData for DeletionUpdateModuleData {
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

/// Runtime instance of `DeletionUpdate`.
pub struct DeletionUpdate {
    pub data: DeletionUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub die_frame: UnsignedInt,
}

impl DeletionUpdate {
    pub fn new(data: DeletionUpdateModuleData) -> Self {
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

impl UpdateFacet for DeletionUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        sim.destroy_object(ctx.owner_slot);
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for DeletionUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    /// C++ constructor: draws the lifetime and sleeps until it expires.
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        let (min_frames, max_frames) = (self.data.min_frames, self.data.max_frames);
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

pub fn parse_deletion_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = DeletionUpdateModuleData::new(tag_key, module_name_key);
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

pub fn create_deletion_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<DeletionUpdateModuleData>()
        .expect("DeletionUpdate data type mismatch")
        .clone();
    Box::new(DeletionUpdate::new(typed))
}
