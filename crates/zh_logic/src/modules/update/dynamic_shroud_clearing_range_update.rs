//! `PKT-323`: `DynamicShroudClearingRangeUpdate`
//! (`DynamicShroudClearingRangeUpdate.h` / `DynamicShroudClearingRangeUpdate.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, TeamId, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `DynamicShroudClearingRangeUpdate`.
#[derive(Debug, Clone, PartialEq)]
pub struct DynamicShroudClearingRangeUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub shrink_delay: UnsignedInt,
    pub shrink_time: UnsignedInt,
    pub grow_delay: UnsignedInt,
    pub grow_time: UnsignedInt,
    pub final_vision: Real,
    pub change_interval: UnsignedInt,
    pub grow_interval: UnsignedInt,
    pub do_spy_sat_fx: Bool,
}

impl DynamicShroudClearingRangeUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            shrink_delay: 0,
            shrink_time: 0,
            grow_delay: 0,
            grow_time: 0,
            final_vision: 0.0,
            change_interval: 0,
            grow_interval: 0,
            do_spy_sat_fx: FALSE,
        }
    }
}

impl ModuleData for DynamicShroudClearingRangeUpdateModuleData {
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
        FacetMask::UPDATE
    }
}

pub fn parse_dynamic_shroud_clearing_range_update_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = DynamicShroudClearingRangeUpdateModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if field.eq_ignore_ascii_case("ChangeInterval") {
            reader.parse_duration_unsigned_int(&mut data.change_interval)
        } else if field.eq_ignore_ascii_case("GrowInterval") {
            reader.parse_duration_unsigned_int(&mut data.grow_interval)
        } else if field.eq_ignore_ascii_case("ShrinkDelay") {
            reader.parse_duration_unsigned_int(&mut data.shrink_delay)
        } else if field.eq_ignore_ascii_case("ShrinkTime") {
            reader.parse_duration_unsigned_int(&mut data.shrink_time)
        } else if field.eq_ignore_ascii_case("GrowDelay") {
            reader.parse_duration_unsigned_int(&mut data.grow_delay)
        } else if field.eq_ignore_ascii_case("GrowTime") {
            reader.parse_duration_unsigned_int(&mut data.grow_time)
        } else if field.eq_ignore_ascii_case("FinalVision") {
            reader.parse_real(&mut data.final_vision)
        } else if field.eq_ignore_ascii_case("GridDecalTemplate") {
            reader.skip_block()
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DscruState {
    #[default]
    NotStartedYet,
    Growing,
    Sustaining,
    Shrinking,
    DoneForever,
    Sleeping,
}

/// Runtime instance of `DynamicShroudClearingRangeUpdate`.
pub struct DynamicShroudClearingRangeUpdate {
    pub data: DynamicShroudClearingRangeUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub state: DscruState,
    pub state_count_down: Int,
    pub total_frames: Int,
    pub grow_start_deadline: UnsignedInt,
    pub sustain_deadline: UnsignedInt,
    pub shrink_start_deadline: UnsignedInt,
    pub done_forever_frame: UnsignedInt,
    pub change_interval_countdown: UnsignedInt,
    pub decals_created: Bool,
    pub vision_change_per_interval: Real,
    pub native_clearing_range: Real,
    pub current_clearing_range: Real,
}

impl DynamicShroudClearingRangeUpdate {
    pub fn new(data: DynamicShroudClearingRangeUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            state: DscruState::NotStartedYet,
            state_count_down: 0,
            total_frames: 1,
            grow_start_deadline: 0,
            sustain_deadline: 0,
            shrink_start_deadline: 0,
            done_forever_frame: 0,
            change_interval_countdown: 0,
            decals_created: FALSE,
            vision_change_per_interval: 0.0,
            native_clearing_range: 200.0,
            current_clearing_range: 0.0,
        }
    }
}

impl UpdateFacet for DynamicShroudClearingRangeUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.state == DscruState::Sleeping {
            return UpdateSleepTime::NONE;
        }

        let current_frame = sim.frame();
        if self.decals_created == FALSE {
            self.decals_created = TRUE;
        }

        if self.state_count_down <= 0 || current_frame > self.done_forever_frame {
            self.state = DscruState::DoneForever;
        } else if (self.state_count_down as UnsignedInt) <= self.shrink_start_deadline {
            self.state = DscruState::Shrinking;
        } else if (self.state_count_down as UnsignedInt) <= self.sustain_deadline {
            self.state = DscruState::Sustaining;
        } else if (self.state_count_down as UnsignedInt) <= self.grow_start_deadline {
            self.state = DscruState::Growing;
        }

        match self.state {
            DscruState::NotStartedYet | DscruState::Sleeping => {}
            DscruState::Growing => {
                let gt = self.data.grow_time as Real;
                let denom = if gt > 1.0 { gt } else { 1.0 };
                self.current_clearing_range += self.native_clearing_range / denom;
                if self.current_clearing_range >= self.native_clearing_range {
                    self.state = DscruState::Sustaining;
                }
            }
            DscruState::Sustaining => {
                self.current_clearing_range = self.native_clearing_range;
            }
            DscruState::Shrinking => {
                let st = self.data.shrink_time as Real;
                let denom = if st > 1.0 { st } else { 1.0 };
                self.current_clearing_range -=
                    (self.native_clearing_range - self.data.final_vision) / denom;
            }
            DscruState::DoneForever => {
                self.current_clearing_range = self.data.final_vision;
            }
        }

        if self.state_count_down > 0 {
            self.state_count_down -= 1;
        }

        if self.change_interval_countdown > 0 {
            self.change_interval_countdown -= 1;
        } else {
            self.change_interval_countdown = if self.state == DscruState::Growing {
                self.data.grow_interval
            } else {
                self.data.change_interval
            };
            sim.set_shroud_clearing_range(ctx.owner_slot, self.current_clearing_range);
            if self.state == DscruState::DoneForever {
                self.state = DscruState::Sleeping;
            }
        }

        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for DynamicShroudClearingRangeUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.change_interval_countdown = 0;
        self.vision_change_per_interval = 0.0;

        let total = self.data.shrink_delay.wrapping_add(self.data.shrink_time);
        self.state_count_down = total as Int;
        self.total_frames = if self.state_count_down > 1 {
            self.state_count_down
        } else {
            1
        };
        let countdown_u32 = self.state_count_down as UnsignedInt;
        self.shrink_start_deadline = countdown_u32.wrapping_sub(self.data.shrink_delay);
        self.grow_start_deadline = countdown_u32.wrapping_sub(self.data.grow_delay);
        self.sustain_deadline = self.grow_start_deadline.wrapping_sub(self.data.grow_time);
        self.done_forever_frame = sim.frame().wrapping_add(countdown_u32);

        self.decals_created = FALSE;
        self.native_clearing_range = 200.0;
        self.current_clearing_range = 0.0;
        self.state = DscruState::NotStartedYet;

        if sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.team_id != TeamId::INVALID)
        {
            self.native_clearing_range = sim.object_shroud_clearing_range(ctx.owner_slot);
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_int(&mut self.state_count_down)?;
        xfer.xfer_int(&mut self.total_frames)?;
        xfer.xfer_unsigned_int(&mut self.grow_start_deadline)?;
        xfer.xfer_unsigned_int(&mut self.sustain_deadline)?;
        xfer.xfer_unsigned_int(&mut self.shrink_start_deadline)?;
        xfer.xfer_unsigned_int(&mut self.done_forever_frame)?;
        xfer.xfer_unsigned_int(&mut self.change_interval_countdown)?;
        xfer.xfer_bool(&mut self.decals_created)?;
        xfer.xfer_real(&mut self.vision_change_per_interval)?;
        xfer.xfer_real(&mut self.native_clearing_range)?;
        xfer.xfer_real(&mut self.current_clearing_range)
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

pub fn create_dynamic_shroud_clearing_range_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<DynamicShroudClearingRangeUpdateModuleData>()
        .expect("DynamicShroudClearingRangeUpdateModuleData mismatch")
        .clone();
    Box::new(DynamicShroudClearingRangeUpdate::new(typed))
}
