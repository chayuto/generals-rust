//! `PKT-306`: `AutoFindHealingUpdateModuleData` and `AutoFindHealingUpdate`
//! (`AutoFindHealingUpdate.h`, `AutoFindHealingUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-306_AutoFindHealingUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt336_370_306_304_leaves.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, CommandSourceType, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `AutoFindHealingUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoFindHealingUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub scan_frames: UnsignedInt,
    pub scan_range: Real,
    pub never_heal: Real,
    /// Parsed but without effect: its test sits behind a `return`.
    pub always_heal: Real,
}

impl AutoFindHealingUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            scan_frames: 0,
            scan_range: 0.0,
            never_heal: 0.95,
            always_heal: 0.25,
        }
    }
}

impl ModuleData for AutoFindHealingUpdateModuleData {
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

/// Runtime instance of `AutoFindHealingUpdate`.
pub struct AutoFindHealingUpdate {
    pub data: AutoFindHealingUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub next_scan_frames: Int,
}

impl AutoFindHealingUpdate {
    pub fn new(data: AutoFindHealingUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            next_scan_frames: 0,
        }
    }

    /// `scanClosestTarget`: the nearest heal pad of anybody, in any state. The
    /// scan is from a position, so a heal pad finds itself.
    fn scan_closest_target(&self, sim: &dyn SimView, owner: ObjectSlot) -> Option<ObjectSlot> {
        let pos = *sim.object(owner)?.thing.get_position();
        let mut best: Option<(ObjectSlot, Real)> = None;
        for other in sim.find_objects_in_range(&pos, self.data.scan_range) {
            if sim.object_is_kind_of(other, KindOfType::HealPad) == FALSE {
                continue;
            }
            let Some(there) = sim.object(other).map(|o| *o.thing.get_position()) else {
                continue;
            };
            let (dx, dy) = (there.x - pos.x, there.y - pos.y);
            let dist_sqr = dx * dx + dy * dy;
            if best.is_none_or(|(_, closest)| dist_sqr < closest) {
                best = Some((other, dist_sqr));
            }
        }
        best.map(|(slot, _)| slot)
    }
}

impl UpdateFacet for AutoFindHealingUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let Some((team, has_body)) = sim
            .object(owner)
            .map(|o| (o.team_id, o.body_slot.is_some()))
        else {
            return UpdateSleepTime::NONE;
        };
        // The computer only.
        if sim.player_is_human(team) != FALSE {
            return UpdateSleepTime::NONE;
        }

        if self.next_scan_frames > 0 {
            self.next_scan_frames -= 1;
            return UpdateSleepTime::NONE;
        }
        self.next_scan_frames = self.data.scan_frames as Int;

        if sim.has_ai(owner) == FALSE || !has_body {
            return UpdateSleepTime::NONE;
        }

        if sim.object_health(owner) > sim.object_max_health(owner) * self.data.never_heal {
            return UpdateSleepTime::NONE;
        }

        if sim.ai_is_idle(owner) == FALSE {
            return UpdateSleepTime::NONE;
        }

        if let Some(heal_unit) = self.scan_closest_target(sim, owner) {
            sim.ai_get_healed(owner, heal_unit, CommandSourceType::FromAi);
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for AutoFindHealingUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_int(&mut self.next_scan_frames)
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

pub fn parse_auto_find_healing_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AutoFindHealingUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("ScanRate") {
            r.parse_duration_unsigned_int(&mut data.scan_frames)
        } else if is("ScanRange") {
            r.parse_real(&mut data.scan_range)
        } else if is("NeverHeal") {
            r.parse_real(&mut data.never_heal)
        } else if is("AlwaysHeal") {
            r.parse_real(&mut data.always_heal)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_auto_find_healing_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<AutoFindHealingUpdateModuleData>()
        .expect("AutoFindHealingUpdate data type mismatch")
        .clone();
    Box::new(AutoFindHealingUpdate::new(typed))
}
