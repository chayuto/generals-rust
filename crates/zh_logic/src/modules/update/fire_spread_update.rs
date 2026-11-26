//! `PKT-327`: `FireSpreadUpdateModuleData` and `FireSpreadUpdate`
//! (`FireSpreadUpdate.h`, `FireSpreadUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-327_FireSpreadUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt330_327_fire.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, FireSpreadFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `FireSpreadUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct FireSpreadUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub ocl_embers: Option<String>,
    pub min_spread_try_delay: UnsignedInt,
    pub max_spread_try_delay: UnsignedInt,
    pub spread_try_range: Real,
}

impl FireSpreadUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            ocl_embers: None,
            min_spread_try_delay: 0,
            max_spread_try_delay: 0,
            spread_try_range: 0.0,
        }
    }
}

impl ModuleData for FireSpreadUpdateModuleData {
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

/// Runtime instance of `FireSpreadUpdate`.
pub struct FireSpreadUpdate {
    pub data: FireSpreadUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
}

impl FireSpreadUpdate {
    pub fn new(data: FireSpreadUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `calcNextSpreadDelay`.
    fn calc_next_spread_delay(&self, sim: &mut dyn SimApi) -> UnsignedInt {
        let delay = sim.rng_logic_int(
            self.data.min_spread_try_delay as Int,
            self.data.max_spread_try_delay as Int,
        ) as UnsignedInt;
        delay.max(1)
    }

    fn is_aflame(sim: &dyn SimView, ctx: ModuleContext) -> bool {
        sim.object(ctx.owner_slot)
            .is_some_and(|o| o.status.test(ObjectStatusType::Aflame.as_i32()) != FALSE)
    }
}

impl FireSpreadFacet for FireSpreadUpdate {
    fn start_fire_spreading(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if !Self::is_aflame(sim, ctx) {
            return;
        }
        let delay = self.calc_next_spread_delay(sim);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(delay));
    }
}

impl UpdateFacet for FireSpreadUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if !Self::is_aflame(sim, ctx) {
            return UpdateSleepTime::FOREVER;
        }
        if let Some(ocl) = &self.data.ocl_embers {
            if let Some(pos) = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position()) {
                sim.spawn_ocl(ocl, ctx.owner_slot, &pos, &pos);
            }
        }
        if self.data.spread_try_range != 0.0 {
            if let Some(target) =
                sim.closest_would_ignite_object(ctx.owner_slot, self.data.spread_try_range)
            {
                sim.try_to_ignite(target);
            }
        }
        UpdateSleepTime(self.calc_next_spread_delay(sim))
    }
}

impl BehaviorModule for FireSpreadUpdate {
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
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)
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

    fn as_fire_spread_mut(&mut self) -> Option<&mut dyn FireSpreadFacet> {
        Some(self)
    }
}

pub fn parse_fire_spread_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = FireSpreadUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("OCLEmbers") {
            let token = r.next_token(SEPS_DEFAULT)?;
            data.ocl_embers = if token.eq_ignore_ascii_case("None") {
                None
            } else {
                Some(
                    parse_ctx
                        .ocl_names
                        .iter()
                        .find(|n| n.eq_ignore_ascii_case(token))
                        .cloned()
                        .unwrap_or_else(|| token.to_owned()),
                )
            };
            Ok(())
        } else if field.eq_ignore_ascii_case("MinSpreadDelay") {
            r.parse_duration_unsigned_int(&mut data.min_spread_try_delay)
        } else if field.eq_ignore_ascii_case("MaxSpreadDelay") {
            r.parse_duration_unsigned_int(&mut data.max_spread_try_delay)
        } else if field.eq_ignore_ascii_case("SpreadTryRange") {
            r.parse_real(&mut data.spread_try_range)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_fire_spread_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<FireSpreadUpdateModuleData>()
        .expect("FireSpreadUpdate data type mismatch")
        .clone();
    Box::new(FireSpreadUpdate::new(typed))
}
