//! `PKT-283`: `OverchargeBehaviorModuleData` and `OverchargeBehavior`
//! (`OverchargeBehavior.h`, `OverchargeBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-283_OverchargeBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c1.rs`.
//!
//! Not modelled: the "overcharge exhausted" message and radar event shown to the local player.

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, DamageFacet, ModuleData, OverchargeFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Real, TeamId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

const LOGICFRAMES_PER_SECOND_REAL: Real = 30.0;

/// INI template data matching `OverchargeBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct OverchargeBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub health_percent_to_drain_per_second: Real,
    pub not_allowed_when_health_below_percent: Real,
}

impl OverchargeBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            health_percent_to_drain_per_second: 0.0,
            not_allowed_when_health_below_percent: 0.0,
        }
    }
}

impl ModuleData for OverchargeBehaviorModuleData {
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
        FacetMask(0x005 | FacetMask::OVERCHARGE.0)
    }
}

/// Runtime instance of `OverchargeBehavior` (`PKT-283`).
pub struct OverchargeBehavior {
    pub data: OverchargeBehaviorModuleData,
    pub overcharge_active: Bool,
    pub schedule: UpdateScheduleTracker,
}

impl OverchargeBehavior {
    pub fn new(data: OverchargeBehaviorModuleData) -> Self {
        Self {
            data,
            overcharge_active: FALSE,
            schedule: UpdateScheduleTracker::new(),
        }
    }
}

impl OverchargeFacet for OverchargeBehavior {
    fn toggle(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let next = Bool::from(self.overcharge_active == FALSE);
        self.enable(sim, ctx, next);
    }

    fn enable(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, enable: Bool) {
        let owner = ctx.owner_slot;
        let team = sim.object(owner).map(|o| o.team_id);
        if enable == FALSE {
            if self.overcharge_active == TRUE {
                // `PowerPlantUpdateInterface::extendRods(FALSE)`.
                sim.retract_power_plant(owner);
                if let Some(team) = team {
                    sim.remove_power_bonus(team, owner);
                }
                self.overcharge_active = FALSE;
                sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
            }
        } else if self.overcharge_active == FALSE {
            // `PowerPlantUpdateInterface::extendRods(TRUE)`.
            sim.extend_power_plant(owner);
            if let Some(team) = team {
                sim.add_power_bonus(team, owner);
            }
            self.overcharge_active = TRUE;
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        }
    }

    fn is_overcharge_active(&self) -> Bool {
        self.overcharge_active
    }
}

impl UpdateFacet for OverchargeBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.overcharge_active != FALSE {
            let owner = ctx.owner_slot;
            let mut damage = DamageInfo::default();
            damage.input.amount = (sim.object_max_health(owner)
                * self.data.health_percent_to_drain_per_second)
                / LOGICFRAMES_PER_SECOND_REAL;
            damage.input.source_id = sim.object(owner).map_or(ctx.owner_id, |o| o.id);
            damage.input.damage_type = DamageType::Penalty;
            damage.input.death_type = DeathType::Normal;
            sim.attempt_damage(owner, &mut damage);

            if sim.object_health(owner)
                < sim.object_max_health(owner) * self.data.not_allowed_when_health_below_percent
            {
                self.enable(sim, ctx, FALSE);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl DamageFacet for OverchargeBehavior {
    fn on_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}
}

impl BehaviorModule for OverchargeBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.overcharge_active == FALSE {
            return;
        }
        if let Some(team) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            sim.remove_power_bonus(team, ctx.owner_slot);
        }
        self.overcharge_active = FALSE;
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if self.overcharge_active == FALSE {
            return;
        }
        if sim
            .object(ctx.owner_slot)
            .is_none_or(|o| o.is_disabled() != FALSE)
        {
            return;
        }
        sim.remove_power_bonus(old_team, ctx.owner_slot);
        sim.add_power_bonus(new_team, ctx.owner_slot);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.overcharge_active)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
        // The power bonus is not saved state, so it is re-applied.
        if self.overcharge_active != FALSE {
            let team = sim
                .object(ctx.owner_slot)
                .filter(|o| o.is_disabled() == FALSE)
                .map(|o| o.team_id);
            if let Some(team) = team {
                sim.add_power_bonus(team, ctx.owner_slot);
            }
        }
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

    fn as_overcharge(&self) -> Option<&dyn OverchargeFacet> {
        Some(self)
    }

    fn as_overcharge_mut(&mut self) -> Option<&mut dyn OverchargeFacet> {
        Some(self)
    }
}

pub fn parse_overcharge_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = OverchargeBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("HealthPercentToDrainPerSecond") {
            r.parse_percent_to_real(&mut data.health_percent_to_drain_per_second)
        } else if field.eq_ignore_ascii_case("NotAllowedWhenHealthBelowPercent") {
            r.parse_percent_to_real(&mut data.not_allowed_when_health_below_percent)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_overcharge_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<OverchargeBehaviorModuleData>()
        .expect("OverchargeBehavior data type mismatch")
        .clone();
    Box::new(OverchargeBehavior::new(typed))
}
