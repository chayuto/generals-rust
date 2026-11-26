//! `PKT-331`: `FloatUpdateModuleData` and `FloatUpdate`
//! (`FloatUpdate.h`, `FloatUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-331_FloatUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_a.rs`.
//!
//! Not modelled: the client-only drawable bobbing (instance matrix).
//! Where there is no water, C++ reads an uninitialised `waterZ`; this port starts it at zero.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, Real, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `FloatUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct FloatUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub enabled: Bool,
}

impl FloatUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            enabled: FALSE,
        }
    }
}

impl ModuleData for FloatUpdateModuleData {
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

/// Runtime instance of `FloatUpdate`.
pub struct FloatUpdate {
    pub data: FloatUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub enabled: Bool,
}

impl FloatUpdate {
    pub fn new(data: FloatUpdateModuleData) -> Self {
        Self {
            enabled: data.enabled,
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `setEnabled`.
    pub fn set_enabled(&mut self, enabled: Bool) {
        self.enabled = enabled;
    }
}

impl UpdateFacet for FloatUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.enabled != FALSE {
            let owner = ctx.owner_slot;
            if let Some(pos) = sim.object(owner).map(|o| *o.thing.get_position()) {
                let mut water_z: Real = 0.0;
                let _ = sim
                    .terrain()
                    .is_underwater(pos.x, pos.y, Some(&mut water_z), None);
                sim.set_object_position(owner, &Coord3D::new(pos.x, pos.y, water_z));
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for FloatUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.enabled)
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

pub fn parse_float_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = FloatUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("Enabled") {
            r.parse_bool(&mut data.enabled)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_float_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<FloatUpdateModuleData>()
        .expect("FloatUpdate data type mismatch")
        .clone();
    Box::new(FloatUpdate::new(typed))
}
