//! `PKT-307` / `PKT-308`: `BaseRegenerateUpdateModuleData` and `BaseRegenerateUpdate`
//! (`BaseRegenerateUpdate.h`, `BaseRenerateUpdate.cpp`; the manifest keeps the file spellings).
//!
//! Packet: `packets/phase5/PKT-308_BaseRenerateUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_a.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, DamageFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, Real, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

const HEAL_RATE: u32 = 3;
const LOGICFRAMES_PER_SECOND: Int = 30;

/// INI template data matching `BaseRegenerateUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct BaseRegenerateUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl BaseRegenerateUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
        }
    }
}

impl ModuleData for BaseRegenerateUpdateModuleData {
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

/// Runtime instance of `BaseRegenerateUpdate`.
pub struct BaseRegenerateUpdate {
    pub data: BaseRegenerateUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
}

impl BaseRegenerateUpdate {
    pub fn new(data: BaseRegenerateUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }
}

impl UpdateFacet for BaseRegenerateUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let Some((under_construction, sold, source_id)) = sim.object(owner).map(|o| {
            (
                o.status.test(ObjectStatusType::UnderConstruction as Int) != FALSE,
                o.status.test(ObjectStatusType::Sold as Int) != FALSE,
                o.id,
            )
        }) else {
            return UpdateSleepTime::FOREVER;
        };
        if under_construction {
            return UpdateSleepTime::NONE;
        }
        if sold {
            return UpdateSleepTime::FOREVER;
        }
        let max_health = sim.object_max_health(owner);
        if max_health == sim.object_health(owner) {
            return UpdateSleepTime::FOREVER;
        }
        let percent = sim.rules().global_data.base_regen_health_percent_per_second;
        let mut healing = DamageInfo::default();
        healing.input.damage_type = DamageType::Healing;
        healing.input.death_type = DeathType::NoneType;
        healing.input.source_id = source_id;
        healing.input.amount =
            HEAL_RATE as Real * (max_health * percent) / LOGICFRAMES_PER_SECOND as Real;
        sim.attempt_healing(owner, &mut healing);
        UpdateSleepTime(HEAL_RATE)
    }
}

impl DamageFacet for BaseRegenerateUpdate {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let global = &sim.rules().global_data;
        let (percent, delay) = (
            global.base_regen_health_percent_per_second,
            global.base_regen_delay,
        );
        let sleep = if percent > 0.0 && dmg.input.damage_type != DamageType::Healing {
            UpdateSleepTime(delay)
        } else {
            UpdateSleepTime::FOREVER
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
    }
}

impl BehaviorModule for BaseRegenerateUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        let sleep = if sim.rules().global_data.base_regen_health_percent_per_second == 0.0 {
            UpdateSleepTime::FOREVER
        } else {
            UpdateSleepTime::NONE
        };
        sim.set_wake_frame(ctx.mod_slot, sleep);
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

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }
}

pub fn parse_base_regenerate_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let data = BaseRegenerateUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(data))
}

pub fn create_base_regenerate_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BaseRegenerateUpdateModuleData>()
        .expect("BaseRegenerateUpdate data type mismatch")
        .clone();
    Box::new(BaseRegenerateUpdate::new(typed))
}
