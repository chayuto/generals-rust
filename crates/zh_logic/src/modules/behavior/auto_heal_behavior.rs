//! `PKT-257`: `AutoHealBehaviorModuleData` and `AutoHealBehavior`
//! (`AutoHealBehavior.h`, `AutoHealBehavior.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::module::{AutoHealFacet, 
    BehaviorModule, DamageFacet, ModuleData, UpdateFacet, UpgradeFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{UpgradeMux, UpgradeMuxData};
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::{DisabledMaskType, DisabledType};
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{RulesRegistry, SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

const INVALID_PARTICLE_SYSTEM_ID: UnsignedInt = 0;
const FOREVER_FRAME: UnsignedInt = 0x3fff_ffff;

/// INI template data matching `AutoHealBehaviorModuleData` (`AutoHealBehavior.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct AutoHealBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_mux: UpgradeMuxData,
    pub initially_active: Bool,
    pub single_burst: Bool,
    pub healing_amount: Int,
    pub healing_delay: UnsignedInt,
    pub start_healing_delay: UnsignedInt,
    pub radius: Real,
    pub affects_whole_player: Bool,
    pub skip_self_for_healing: Bool,
    pub kind_of: KindOfMaskType,
    pub forbidden_kind_of: KindOfMaskType,
    pub radius_particle_system: Option<String>,
    pub unit_heal_pulse_particle_system: Option<String>,
}

impl AutoHealBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut kind_of = KindOfMaskType::new();
        kind_of.flip();
        Self {
            tag_key,
            module_name_key,
            upgrade_mux: UpgradeMuxData::new(),
            initially_active: FALSE,
            single_burst: FALSE,
            healing_amount: 0,
            healing_delay: u32::MAX,
            start_healing_delay: 0,
            radius: 0.0,
            affects_whole_player: FALSE,
            skip_self_for_healing: FALSE,
            kind_of,
            forbidden_kind_of: KindOfMaskType::new(),
            radius_particle_system: None,
            unit_heal_pulse_particle_system: None,
        }
    }
}

impl ModuleData for AutoHealBehaviorModuleData {
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
        FacetMask(0x085)
    }
}

/// Runtime instance of `AutoHealBehavior` (`PKT-257`).
pub struct AutoHealBehavior {
    pub data: AutoHealBehaviorModuleData,
    pub radius_particle_system_id: UnsignedInt,
    pub soonest_heal_frame: UnsignedInt,
    pub stopped: Bool,
    pub upgrade: UpgradeMux,
    pub schedule: UpdateScheduleTracker,
}

impl AutoHealBehavior {
    pub fn new(data: AutoHealBehaviorModuleData) -> Self {
        Self {
            data,
            radius_particle_system_id: INVALID_PARTICLE_SYSTEM_ID,
            soonest_heal_frame: 0,
            stopped: FALSE,
            upgrade: UpgradeMux::new(),
            schedule: UpdateScheduleTracker::new(),
        }
    }

    #[inline]
    pub fn is_upgrade_active(&self) -> Bool {
        self.upgrade.is_already_upgraded()
    }

    /// Stops all future healing updates (`AutoHealBehavior::stopHealing`).
    pub fn stop_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.stopped = TRUE;
        self.soonest_heal_frame = FOREVER_FRAME;
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    /// Resets activation state so upgrade can fire again (`AutoHealBehavior::undoUpgrade`).
    pub fn undo_upgrade(&mut self) {
        self.soonest_heal_frame = 0;
        self.upgrade.set_upgrade_executed(FALSE);
    }

    fn pulse_heal_object(
        &mut self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        target: ObjectSlot,
    ) {
        if self.stopped != FALSE {
            return;
        }
        #[cfg(feature = "retail-crc")]
        let use_direct = self.data.radius == 0.0;
        #[cfg(not(feature = "retail-crc"))]
        let use_direct = self.data.radius == 0.0 || self.data.single_burst != FALSE;

        if use_direct {
            let source_id = sim.object(owner).map_or(Default::default(), |o| o.id);
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Healing;
            dmg.input.death_type = DeathType::NoneType;
            dmg.input.source_id = source_id;
            dmg.input.amount = self.data.healing_amount as Real;
            sim.attempt_healing(target, &mut dmg);
        } else {
            let source_id = sim.object(owner).map_or(Default::default(), |o| o.id);
            let _ = sim.attempt_healing_from_sole_benefactor(
                target,
                self.data.healing_amount as Real,
                source_id,
                self.data.healing_delay,
            );
        }
        self.soonest_heal_frame = sim.frame().wrapping_add(self.data.healing_delay);
    }

    fn can_heal_candidate(&self, sim: &dyn SimView, target: ObjectSlot, owner: ObjectSlot) -> bool {
        let Some(cand) = sim.object(target) else {
            return false;
        };
        if cand.is_effectively_dead() != FALSE {
            return false;
        }
        if self.data.skip_self_for_healing != FALSE && target == owner {
            return false;
        }
        let tf = &sim.rules().thing_factory;
        if cand.thing.is_any_kind_of(tf, &self.data.kind_of) == FALSE {
            return false;
        }
        if cand.thing.is_any_kind_of(tf, &self.data.forbidden_kind_of) != FALSE {
            return false;
        }
        let Some(body_slot) = cand.body_slot else {
            return false;
        };
        let ModuleSidecar::Body {
            health, max_health, ..
        } = sim.module_sidecar(body_slot)
        else {
            return false;
        };
        health < max_health
    }
}

impl BehaviorModule for AutoHealBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        if self.data.initially_active != FALSE {
            self.upgrade
                .give_self_upgrade(sim, ctx, &self.data.upgrade_mux, |_, _| {});
            let delay = self.data.healing_delay;
            let roll = sim.rng_logic_int(1, delay as Int) as UnsignedInt;
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(roll));
        } else {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
        }
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        if let DeferredSelfOp::UpgradeResetKey(key) = op {
            if self.data.upgrade_mux.is_triggered_by_key_name(key) {
                self.upgrade.set_upgrade_executed(FALSE);
            }
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
        self.upgrade.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        self.upgrade.xfer(xfer)?;
        xfer.xfer_unsigned_int(&mut self.radius_particle_system_id)?;
        xfer.xfer_unsigned_int(&mut self.soonest_heal_frame)?;
        xfer.xfer_bool(&mut self.stopped)
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

    fn as_auto_heal_mut(&mut self) -> Option<&mut dyn AutoHealFacet> {
        Some(self)
    }

    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        Some(self)
    }

    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        Some(self)
    }
}

impl AutoHealFacet for AutoHealBehavior {
    fn stop_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        AutoHealBehavior::stop_healing(self, sim, ctx);
    }
}

impl DamageFacet for AutoHealBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &mut DamageInfo) {
        if self.stopped != FALSE {
            return;
        }
        if self.is_upgrade_active() != FALSE && self.data.radius == 0.0 {
            if self.data.start_healing_delay > 0 {
                sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(self.data.start_healing_delay));
            } else if sim.frame() > self.soonest_heal_frame {
                sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
            }
        }
    }
}

impl UpgradeFacet for AutoHealBehavior {
    fn is_already_upgraded(&self) -> Bool {
        self.upgrade.is_already_upgraded()
    }

    fn attempt_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        let mux = &self.data.upgrade_mux;
        self.upgrade.attempt_upgrade(sim, ctx, mux, key_mask, |s, c| {
            s.set_wake_frame(c.mod_slot, UpdateSleepTime::NONE);
        })
    }

    fn would_upgrade(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .would_upgrade(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn reset_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .reset_upgrade(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn test_upgrade_conditions(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .test_upgrade_conditions(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn get_upgrade_activation_masks(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        activation: &mut UpgradeMaskType,
        conflicting: &mut UpgradeMaskType,
    ) {
        self.data
            .upgrade_mux
            .get_upgrade_activation_masks(sim.rules(), activation, conflicting);
    }

    fn perform_upgrade_fx(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.data.upgrade_mux.perform_upgrade_fx(sim, ctx.owner_slot);
    }

    fn is_sub_objects_upgrade(&self) -> Bool {
        FALSE
    }

    fn give_self_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let mux = &self.data.upgrade_mux;
        self.upgrade.give_self_upgrade(sim, ctx, mux, |s, c| {
            s.set_wake_frame(c.mod_slot, UpdateSleepTime::NONE);
        });
    }

    fn force_refresh_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.upgrade.force_refresh_upgrade(sim, ctx, |s, c| {
            s.set_wake_frame(c.mod_slot, UpdateSleepTime::NONE);
        });
    }
}

impl UpdateFacet for AutoHealBehavior {
    fn disabled_types_to_process(&self) -> DisabledMaskType {
        let mut mask = DisabledMaskType::new();
        mask.set(DisabledType::Held as Int, TRUE as Int);
        mask
    }

    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.stopped != FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let owner = ctx.owner_slot;
        let Some(owner_rec) = sim.object(owner) else {
            return UpdateSleepTime::FOREVER;
        };
        if self.is_upgrade_active() == FALSE || owner_rec.is_effectively_dead() != FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let owner_team = owner_rec.team_id;
        let owner_pos = *owner_rec.thing.get_position();
        let owner_off_map = (owner_rec.private_status & PRIVATE_STATUS_OFF_MAP) != 0;

        if self.data.affects_whole_player != FALSE {
            let owner_player_idx = sim.player_index(owner_team);
            let mut to_heal = Vec::new();
            for slot in sim.world_object_slots() {
                let Some(cand) = sim.object(slot) else {
                    continue;
                };
                if (cand.private_status & PRIVATE_STATUS_OFF_MAP) != 0 {
                    continue;
                }
                if sim.player_index(cand.team_id) != owner_player_idx {
                    continue;
                }
                if self.can_heal_candidate(sim, slot, owner) {
                    to_heal.push(slot);
                }
            }
            for target in to_heal {
                self.pulse_heal_object(sim, owner, target);
            }
            return UpdateSleepTime(self.data.healing_delay);
        } else if self.data.radius == 0.0 {
            let needs_heal = owner_rec
                .body_slot
                .is_some_and(|bs| matches!(sim.module_sidecar(bs), ModuleSidecar::Body { health, max_health, .. } if health < max_health));
            if needs_heal {
                self.pulse_heal_object(sim, owner, owner);
                return UpdateSleepTime(self.data.healing_delay);
            }
            return UpdateSleepTime::FOREVER;
        } else {
            let mut to_heal = Vec::new();
            for slot in sim.find_objects_in_range(&owner_pos, self.data.radius) {
                if sim.player_relationship(owner, slot) != Relationship::Allies {
                    continue;
                }
                let Some(cand) = sim.object(slot) else {
                    continue;
                };
                let cand_off_map = (cand.private_status & PRIVATE_STATUS_OFF_MAP) != 0;
                if cand_off_map != owner_off_map {
                    continue;
                }
                if self.can_heal_candidate(sim, slot, owner) {
                    to_heal.push(slot);
                }
            }
            for target in to_heal {
                self.pulse_heal_object(sim, owner, target);
            }
            if self.data.single_burst != FALSE {
                UpdateSleepTime::FOREVER
            } else {
                UpdateSleepTime(self.data.healing_delay)
            }
        }
    }
}

trait UpgradeMuxDataTriggerExt {
    fn is_triggered_by_key_name(&self, key: NameKeyType) -> bool;
}

impl UpgradeMuxDataTriggerExt for UpgradeMuxData {
    fn is_triggered_by_key_name(&self, _key: NameKeyType) -> bool {
        let _ = self;
        false
    }
}

pub fn parse_auto_heal_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AutoHealBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("StartsActive") {
            r.parse_bool(&mut data.initially_active)
        } else if field.eq_ignore_ascii_case("SingleBurst") {
            r.parse_bool(&mut data.single_burst)
        } else if field.eq_ignore_ascii_case("HealingAmount") {
            r.parse_int(&mut data.healing_amount)
        } else if field.eq_ignore_ascii_case("HealingDelay") {
            r.parse_duration_unsigned_int(&mut data.healing_delay)
        } else if field.eq_ignore_ascii_case("StartHealingDelay") {
            r.parse_duration_unsigned_int(&mut data.start_healing_delay)
        } else if field.eq_ignore_ascii_case("Radius") {
            r.parse_real(&mut data.radius)
        } else if field.eq_ignore_ascii_case("AffectsWholePlayer") {
            r.parse_bool(&mut data.affects_whole_player)
        } else if field.eq_ignore_ascii_case("SkipSelfForHealing") {
            r.parse_bool(&mut data.skip_self_for_healing)
        } else if field.eq_ignore_ascii_case("KindOf") {
            data.kind_of.parse_from_ini(r, &KINDOF_NAMES)
        } else if field.eq_ignore_ascii_case("ForbiddenKindOf") {
            data.forbidden_kind_of.parse_from_ini(r, &KINDOF_NAMES)
        } else if field.eq_ignore_ascii_case("RadiusParticleSystemName") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.radius_particle_system =
                resolve_optional_asset_name(tok, parse_ctx.particle_system_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("UnitHealPulseParticleSystemName") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.unit_heal_pulse_particle_system =
                resolve_optional_asset_name(tok, parse_ctx.particle_system_names);
            Ok(())
        } else if let Some(res) = data.upgrade_mux.parse_field(field, r, parse_ctx) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_auto_heal_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<AutoHealBehaviorModuleData>()
        .expect("AutoHealBehavior data type mismatch")
        .clone();
    Box::new(AutoHealBehavior::new(typed))
}

#[allow(dead_code)]
fn _assert_rules_ref(_: &RulesRegistry) {}
