//! `PKT-297`: `SupplyWarehouseCripplingBehaviorModuleData` and `SupplyWarehouseCripplingBehavior`
//! (`SupplyWarehouseCripplingBehavior.h`, `SupplyWarehouseCripplingBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-297_SupplyWarehouseCripplingBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c1.rs`.
//!
//! The dock update it cripples is not ported; the call is recorded through
//! `SimApi::set_dock_crippled`.

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, DamageFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `SupplyWarehouseCripplingBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SupplyWarehouseCripplingBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub self_heal_supression: UnsignedInt,
    pub self_heal_delay: UnsignedInt,
    pub self_heal_amount: Real,
}

impl SupplyWarehouseCripplingBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            self_heal_supression: 0,
            self_heal_delay: 0,
            self_heal_amount: 0.0,
        }
    }
}

impl ModuleData for SupplyWarehouseCripplingBehaviorModuleData {
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

/// Runtime instance of `SupplyWarehouseCripplingBehavior` (`PKT-297`).
pub struct SupplyWarehouseCripplingBehavior {
    pub data: SupplyWarehouseCripplingBehaviorModuleData,
    pub healing_supressed_until_frame: UnsignedInt,
    pub next_healing_frame: UnsignedInt,
    pub schedule: UpdateScheduleTracker,
}

impl SupplyWarehouseCripplingBehavior {
    pub fn new(data: SupplyWarehouseCripplingBehaviorModuleData) -> Self {
        Self {
            data,
            healing_supressed_until_frame: 0,
            next_healing_frame: 0,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    fn react_to_damage_state(
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        if new_state == BodyDamageType::ReallyDamaged {
            sim.set_dock_crippled(ctx.owner_slot, TRUE);
        } else if old_state == BodyDamageType::ReallyDamaged {
            sim.set_dock_crippled(ctx.owner_slot, FALSE);
        }
    }
}

impl DamageFacet for SupplyWarehouseCripplingBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &mut DamageInfo) {
        let now = sim.frame();
        self.healing_supressed_until_frame = now.wrapping_add(self.data.self_heal_supression);
        self.next_healing_frame = self.healing_supressed_until_frame;
        sim.set_wake_frame(
            ctx.mod_slot,
            UpdateSleepTime(self.healing_supressed_until_frame.wrapping_sub(now)),
        );
    }

    fn on_body_damage_state_change(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        Self::react_to_damage_state(sim, ctx, old_state, new_state);
    }
}

impl UpdateFacet for SupplyWarehouseCripplingBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        self.next_healing_frame = now.wrapping_add(self.data.self_heal_delay);

        let old_state = sim.object_body_damage_state(owner);
        let mut healing = DamageInfo::default();
        healing.input.damage_type = DamageType::Healing;
        healing.input.death_type = DeathType::NoneType;
        healing.input.source_id = ObjectId::INVALID;
        healing.input.amount = self.data.self_heal_amount;
        sim.attempt_healing(owner, &mut healing);
        // This module is executing, so the body's state-change callback cannot reach it
        // through dispatch; apply the same reaction here.
        let new_state = sim.object_body_damage_state(owner);
        if new_state != old_state {
            Self::react_to_damage_state(sim, ctx, old_state, new_state);
        }

        if sim.object_health(owner) == sim.object_max_health(owner) {
            return UpdateSleepTime::FOREVER;
        }
        UpdateSleepTime(self.next_healing_frame.wrapping_sub(now))
    }
}

impl BehaviorModule for SupplyWarehouseCripplingBehavior {
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
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.healing_supressed_until_frame)?;
        xfer.xfer_unsigned_int(&mut self.next_healing_frame)
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

pub fn parse_supply_warehouse_crippling_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SupplyWarehouseCripplingBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("SelfHealSupression") {
            r.parse_duration_unsigned_int(&mut data.self_heal_supression)
        } else if field.eq_ignore_ascii_case("SelfHealDelay") {
            r.parse_duration_unsigned_int(&mut data.self_heal_delay)
        } else if field.eq_ignore_ascii_case("SelfHealAmount") {
            r.parse_real(&mut data.self_heal_amount)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_supply_warehouse_crippling_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SupplyWarehouseCripplingBehaviorModuleData>()
        .expect("SupplyWarehouseCripplingBehavior data type mismatch")
        .clone();
    Box::new(SupplyWarehouseCripplingBehavior::new(typed))
}
