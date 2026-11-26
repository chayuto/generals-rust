//! `PKT-322`: `DynamicGeometryInfoUpdateModuleData` and `DynamicGeometryInfoUpdate`
//! (`DynamicGeometryInfoUpdate.h`, `DynamicGeometryInfoUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-322_DynamicGeometryInfoUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_b.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::geometry::GeometryInfo;
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `FORWARD` of `DynamicGeometryDirection`.
pub const DIRECTION_FORWARD: Int = 1;

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicGeometryInfoUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub initial_delay: UnsignedInt,
    pub initial_height: Real,
    pub initial_major_radius: Real,
    pub initial_minor_radius: Real,
    pub final_height: Real,
    pub final_major_radius: Real,
    pub final_minor_radius: Real,
    pub transition_time: UnsignedInt,
    pub reverse_at_transition_time: Bool,
}

impl DynamicGeometryInfoUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            initial_delay: 0,
            initial_height: 0.0,
            initial_major_radius: 0.0,
            initial_minor_radius: 0.0,
            final_height: 0.0,
            final_major_radius: 0.0,
            final_minor_radius: 0.0,
            transition_time: 1,
            reverse_at_transition_time: FALSE,
        }
    }
}

impl ModuleData for DynamicGeometryInfoUpdateModuleData {
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

/// Runtime instance of `DynamicGeometryInfoUpdate`.
pub struct DynamicGeometryInfoUpdate {
    pub data: DynamicGeometryInfoUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub starting_delay_countdown: UnsignedInt,
    pub time_active: UnsignedInt,
    pub started: Bool,
    pub finished: Bool,
    pub reverse_at_transition_time: Bool,
    /// Never assigned by the C++ constructor or `update`; the pooled object is
    /// zero-filled, so the four saved bytes are zero unless a load sets them.
    pub direction: Int,
    pub switched_directions: Bool,
    pub initial_height: Real,
    pub initial_major_radius: Real,
    pub initial_minor_radius: Real,
    pub final_height: Real,
    pub final_major_radius: Real,
    pub final_minor_radius: Real,
}

impl DynamicGeometryInfoUpdate {
    pub fn new(data: DynamicGeometryInfoUpdateModuleData) -> Self {
        Self {
            schedule: UpdateScheduleTracker::new(),
            starting_delay_countdown: data.initial_delay.max(1),
            time_active: 0,
            started: FALSE,
            finished: FALSE,
            reverse_at_transition_time: data.reverse_at_transition_time,
            direction: 0,
            switched_directions: FALSE,
            initial_height: data.initial_height,
            initial_major_radius: data.initial_major_radius,
            initial_minor_radius: data.initial_minor_radius,
            final_height: data.final_height,
            final_major_radius: data.final_major_radius,
            final_minor_radius: data.final_minor_radius,
            data,
        }
    }
}

impl UpdateFacet for DynamicGeometryInfoUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.finished != FALSE {
            return UpdateSleepTime::NONE;
        }
        if self.started == FALSE {
            self.starting_delay_countdown = self.starting_delay_countdown.wrapping_sub(1);
            if self.starting_delay_countdown > 0 {
                return UpdateSleepTime::NONE;
            }
            self.started = TRUE;
        }

        let mut ratio: Real = 1.0;
        if self.data.transition_time > 0 {
            ratio = self.time_active as Real / self.data.transition_time as Real;
        }
        let new_height = self.initial_height + (ratio * (self.final_height - self.initial_height));
        let new_major = self.initial_major_radius
            + (ratio * (self.final_major_radius - self.initial_major_radius));
        let new_minor = self.initial_minor_radius
            + (ratio * (self.final_minor_radius - self.initial_minor_radius));

        if let Some(old) = sim.object(ctx.owner_slot).map(|o| o.geometry.clone()) {
            let geom = GeometryInfo::new(
                old.get_geom_type(),
                old.get_is_small(),
                new_height,
                new_major,
                new_minor,
            );
            sim.set_geometry_info(ctx.owner_slot, &geom);
        }

        self.time_active = self.time_active.wrapping_add(1);
        if self.time_active > self.data.transition_time {
            if self.reverse_at_transition_time == TRUE {
                self.switched_directions = TRUE;
                self.time_active = 0;
                self.reverse_at_transition_time = FALSE;
                self.initial_height = self.data.final_height;
                self.initial_major_radius = self.data.final_major_radius;
                self.initial_minor_radius = self.data.final_minor_radius;
                self.final_height = self.data.initial_height;
                self.final_major_radius = self.data.initial_major_radius;
                self.final_minor_radius = self.data.initial_minor_radius;
            } else {
                self.finished = TRUE;
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for DynamicGeometryInfoUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.starting_delay_countdown)?;
        xfer.xfer_unsigned_int(&mut self.time_active)?;
        xfer.xfer_bool(&mut self.started)?;
        xfer.xfer_bool(&mut self.finished)?;
        xfer.xfer_bool(&mut self.reverse_at_transition_time)?;
        let mut direction = self.direction.to_le_bytes();
        xfer.xfer_user(&mut direction)?;
        self.direction = Int::from_le_bytes(direction);
        xfer.xfer_bool(&mut self.switched_directions)?;
        xfer.xfer_real(&mut self.initial_height)?;
        xfer.xfer_real(&mut self.initial_major_radius)?;
        xfer.xfer_real(&mut self.initial_minor_radius)?;
        xfer.xfer_real(&mut self.final_height)?;
        xfer.xfer_real(&mut self.final_major_radius)?;
        xfer.xfer_real(&mut self.final_minor_radius)?;
        Ok(())
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
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

pub fn create_dynamic_geometry_info_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<DynamicGeometryInfoUpdateModuleData>()
        .expect("DynamicGeometryInfoUpdate data type mismatch")
        .clone();
    Box::new(DynamicGeometryInfoUpdate::new(typed))
}

pub fn parse_dynamic_geometry_info_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = DynamicGeometryInfoUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("InitialDelay") {
            r.parse_duration_unsigned_int(&mut data.initial_delay)
        } else if field.eq_ignore_ascii_case("InitialHeight") {
            r.parse_real(&mut data.initial_height)
        } else if field.eq_ignore_ascii_case("InitialMajorRadius") {
            r.parse_real(&mut data.initial_major_radius)
        } else if field.eq_ignore_ascii_case("InitialMinorRadius") {
            r.parse_real(&mut data.initial_minor_radius)
        } else if field.eq_ignore_ascii_case("FinalHeight") {
            r.parse_real(&mut data.final_height)
        } else if field.eq_ignore_ascii_case("FinalMajorRadius") {
            r.parse_real(&mut data.final_major_radius)
        } else if field.eq_ignore_ascii_case("FinalMinorRadius") {
            r.parse_real(&mut data.final_minor_radius)
        } else if field.eq_ignore_ascii_case("TransitionTime") {
            r.parse_duration_unsigned_int(&mut data.transition_time)
        } else if field.eq_ignore_ascii_case("ReverseAtTransitionTime") {
            r.parse_bool(&mut data.reverse_at_transition_time)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}
