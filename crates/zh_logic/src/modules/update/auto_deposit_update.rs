//! `PKT-305`: `AutoDepositUpdateModuleData` and `AutoDepositUpdate`
//! (`AutoDepositUpdate.h`, `AutoDepositUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-305_AutoDepositUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt383_377_305_topple.rs`.
//!
//! The floating "+$n" text is client-only: the world records its position and
//! amount. Its jitter draws from the client PRNG, which `StructureToppleUpdate`
//! reads back into the simulation, so the draws are kept.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{AutoDepositFacet, BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView, CONSTRUCTION_COMPLETE};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_COLON};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, TeamId, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `AutoDepositUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoDepositUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub deposit_frame: UnsignedInt,
    pub deposit_amount: Int,
    pub initial_capture_bonus: Int,
    pub is_actual_money: Bool,
    /// `UpgradedBoost`: upgrade name and extra amount, in INI order.
    pub upgrade_boost: Vec<(String, Int)>,
}

impl AutoDepositUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            deposit_frame: 0,
            deposit_amount: 0,
            initial_capture_bonus: 0,
            is_actual_money: TRUE,
            upgrade_boost: Vec::new(),
        }
    }
}

impl ModuleData for AutoDepositUpdateModuleData {
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

/// Runtime instance of `AutoDepositUpdate`.
pub struct AutoDepositUpdate {
    pub data: AutoDepositUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub deposit_on_frame: UnsignedInt,
    pub award_initial_capture_bonus: Bool,
    pub initialized: Bool,
}

impl AutoDepositUpdate {
    pub fn new(data: AutoDepositUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            deposit_on_frame: 0,
            award_initial_capture_bonus: FALSE,
            initialized: FALSE,
        }
    }

    /// `getUpgradedSupplyBoost`. The reference caches the upgrade template in a
    /// function-local static, so every entry of every instance tests the template
    /// that was looked up first.
    fn upgraded_supply_boost(&self, sim: &mut dyn SimApi, team: TeamId) -> Int {
        for (name, amount) in &self.data.upgrade_boost {
            if let Some(mask) = sim.auto_deposit_boost_upgrade(name) {
                if sim.player_has_upgrade(team, &mask) != FALSE {
                    return *amount;
                }
            }
        }
        0
    }
}

impl AutoDepositFacet for AutoDepositUpdate {
    fn award_initial_capture_bonus(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.deposit_on_frame = sim.frame().wrapping_add(self.data.deposit_frame);
        let bonus = self.data.initial_capture_bonus;
        if self.award_initial_capture_bonus == FALSE || bonus <= 0 {
            return;
        }
        let Some((team, mut pos)) = sim
            .object(ctx.owner_slot)
            .map(|o| (o.team_id, *o.thing.get_position()))
        else {
            return;
        };
        sim.deposit_player_money(team, bonus as UnsignedInt, TRUE);
        sim.add_money_earned(team, bonus);
        pos.z += 10.0;
        sim.add_floating_money_text(ctx.owner_slot, &pos, bonus);
        self.award_initial_capture_bonus = FALSE;
    }
}

impl UpdateFacet for AutoDepositUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if sim.frame() < self.deposit_on_frame {
            return UpdateSleepTime::NONE;
        }
        let owner = ctx.owner_slot;
        if self.initialized == FALSE {
            self.award_initial_capture_bonus = TRUE;
            self.initialized = TRUE;
        }
        self.deposit_on_frame = sim.frame().wrapping_add(self.data.deposit_frame);
        if sim.is_neutral_controlled(owner) != FALSE || self.data.deposit_amount <= 0 {
            return UpdateSleepTime::NONE;
        }
        if sim.object_construction_percent(owner) != CONSTRUCTION_COMPLETE {
            return UpdateSleepTime::NONE;
        }
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return UpdateSleepTime::NONE;
        };
        let money = self
            .data
            .deposit_amount
            .wrapping_add(self.upgraded_supply_boost(sim, team));
        if self.data.is_actual_money != FALSE {
            // A negative sum wraps, as through `Money::deposit(UnsignedInt)`. The
            // score counts the base amount only.
            sim.deposit_player_money(team, money as UnsignedInt, TRUE);
            sim.add_money_earned(team, self.data.deposit_amount);
        }
        if money > 0 && sim.object_is_logically_visible(owner) != FALSE {
            let Some((mut pos, major, minor)) = sim.object(owner).map(|o| {
                (
                    *o.thing.get_position(),
                    o.geometry.get_major_radius(),
                    o.geometry.get_minor_radius(),
                )
            }) else {
                return UpdateSleepTime::NONE;
            };
            pos.z += 10.0;
            if sim.object_is_kind_of(owner, KindOfType::Structure) != FALSE {
                let width: Real = major * 0.3;
                let depth: Real = minor * 0.3;
                pos.x += sim.rng_client_int((-width) as Int, width as Int) as Real;
                pos.y += sim.rng_client_int((-depth) as Int, depth as Int) as Real;
            }
            sim.add_floating_money_text(owner, &pos, money);
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for AutoDepositUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        self.deposit_on_frame = sim.frame().wrapping_add(self.data.deposit_frame);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.deposit_on_frame)?;
        xfer.xfer_bool(&mut self.award_initial_capture_bonus)?;
        if version > 1 {
            xfer.xfer_bool(&mut self.initialized)?;
        }
        Ok(())
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

    fn as_auto_deposit_mut(&mut self) -> Option<&mut dyn AutoDepositFacet> {
        Some(self)
    }
}

/// `parseUpgradePair`: `UpgradeType:<name> Boost:<amount>`.
fn parse_upgrade_pair(r: &mut IniReader<'_>) -> Result<(String, Int), IniError> {
    let name = r.next_sub_token("UpgradeType")?.to_string();
    let amount = IniReader::scan_int(r.next_sub_token("Boost")?)?;
    Ok((name, amount))
}

pub fn parse_auto_deposit_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AutoDepositUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("DepositTiming") {
            r.parse_duration_unsigned_int(&mut data.deposit_frame)
        } else if field.eq_ignore_ascii_case("DepositAmount") {
            r.parse_int(&mut data.deposit_amount)
        } else if field.eq_ignore_ascii_case("InitialCaptureBonus") {
            r.parse_int(&mut data.initial_capture_bonus)
        } else if field.eq_ignore_ascii_case("ActualMoney") {
            r.parse_bool(&mut data.is_actual_money)
        } else if field.eq_ignore_ascii_case("UpgradedBoost") {
            let _ = SEPS_COLON;
            data.upgrade_boost.push(parse_upgrade_pair(r)?);
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_auto_deposit_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<AutoDepositUpdateModuleData>()
        .expect("AutoDepositUpdate data type mismatch")
        .clone();
    Box::new(AutoDepositUpdate::new(typed))
}
