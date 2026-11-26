//! `PKT-268`: `FireWeaponWhenDamagedBehaviorModuleData` and `FireWeaponWhenDamagedBehavior`
//! (`FireWeaponWhenDamagedBehavior.h`, `FireWeaponWhenDamagedBehavior.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, DamageFacet, ModuleData, UpdateFacet, UpgradeFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{UpgradeMux, UpgradeMuxData};
use crate::object::weapon::{Weapon, WeaponTemplate};
use crate::object::weapon_fire;
use crate::object::weapon_set::WeaponSlotType;
use crate::rules::combat_types::{
    get_damage_type_flag, parse_damage_type_flags, DamageTypeFlags, WeaponStatus,
};
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, Real, UpdateSleepTime, FALSE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// INI template data matching `FireWeaponWhenDamagedBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct FireWeaponWhenDamagedBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_mux: UpgradeMuxData,
    pub initially_active: Bool,
    pub damage_types: DamageTypeFlags,
    pub damage_amount: Real,
    pub reaction_weapon_pristine: Option<WeaponTemplate>,
    pub reaction_weapon_damaged: Option<WeaponTemplate>,
    pub reaction_weapon_really_damaged: Option<WeaponTemplate>,
    pub reaction_weapon_rubble: Option<WeaponTemplate>,
    pub continuous_weapon_pristine: Option<WeaponTemplate>,
    pub continuous_weapon_damaged: Option<WeaponTemplate>,
    pub continuous_weapon_really_damaged: Option<WeaponTemplate>,
    pub continuous_weapon_rubble: Option<WeaponTemplate>,
}

impl FireWeaponWhenDamagedBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            upgrade_mux: UpgradeMuxData::new(),
            initially_active: FALSE,
            damage_types: DamageTypeFlags::new_set_all(),
            damage_amount: 0.0,
            reaction_weapon_pristine: None,
            reaction_weapon_damaged: None,
            reaction_weapon_really_damaged: None,
            reaction_weapon_rubble: None,
            continuous_weapon_pristine: None,
            continuous_weapon_damaged: None,
            continuous_weapon_really_damaged: None,
            continuous_weapon_rubble: None,
        }
    }
}

impl ModuleData for FireWeaponWhenDamagedBehaviorModuleData {
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

/// Runtime instance of `FireWeaponWhenDamagedBehavior` (`PKT-268`).
pub struct FireWeaponWhenDamagedBehavior {
    pub data: FireWeaponWhenDamagedBehaviorModuleData,
    pub upgrade: UpgradeMux,
    pub schedule: UpdateScheduleTracker,
    pub reaction_weapon_pristine: Option<Weapon>,
    pub reaction_weapon_damaged: Option<Weapon>,
    pub reaction_weapon_really_damaged: Option<Weapon>,
    pub reaction_weapon_rubble: Option<Weapon>,
    pub continuous_weapon_pristine: Option<Weapon>,
    pub continuous_weapon_damaged: Option<Weapon>,
    pub continuous_weapon_really_damaged: Option<Weapon>,
    pub continuous_weapon_rubble: Option<Weapon>,
}

impl FireWeaponWhenDamagedBehavior {
    pub fn new(data: FireWeaponWhenDamagedBehaviorModuleData) -> Self {
        Self {
            data,
            upgrade: UpgradeMux::new(),
            schedule: UpdateScheduleTracker::new(),
            reaction_weapon_pristine: None,
            reaction_weapon_damaged: None,
            reaction_weapon_really_damaged: None,
            reaction_weapon_rubble: None,
            continuous_weapon_pristine: None,
            continuous_weapon_damaged: None,
            continuous_weapon_really_damaged: None,
            continuous_weapon_rubble: None,
        }
    }

    #[inline]
    pub fn is_upgrade_active(&self) -> Bool {
        self.upgrade.is_already_upgraded()
    }

    /// The constructor's `allocateNewWeapon` + `reloadAmmo`.
    fn init_and_reload_weapon(
        tmpl: &Option<WeaponTemplate>,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
    ) -> Option<Weapon> {
        let mut w = Weapon::new(tmpl.as_ref()?.clone(), WeaponSlotType::Primary, sim.frame());
        weapon_fire::reload_ammo(&mut w, sim, owner);
        Some(w)
    }

    /// Force-fires the weapon at the owner's position if it is ready.
    fn force_fire_weapon_slot(wpn: &mut Option<Weapon>, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let Some(w) = wpn.as_mut() else {
            return;
        };
        if w.get_status(sim.frame()) != WeaponStatus::ReadyToFire {
            return;
        }
        let pos = sim
            .object(owner)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        weapon_fire::force_fire_weapon(w, sim, owner, &pos);
    }

    fn xfer_opt_weapon(
        xfer: &mut dyn Xfer,
        sim: &dyn SimView,
        slot: &mut Option<Weapon>,
    ) -> Result<(), XferError> {
        let mut present = Bool::from(slot.is_some());
        xfer.xfer_bool(&mut present)?;
        if present != FALSE {
            if xfer.mode() == XferMode::Load && slot.is_none() {
                *slot = Some(Weapon::new(
                    WeaponTemplate::default(),
                    WeaponSlotType::Primary,
                    sim.frame(),
                ));
            }
            if let Some(w) = slot.as_mut() {
                w.xfer(xfer)?;
                if xfer.mode() == XferMode::Load && !w.template_name.is_empty() {
                    if let Some(t) = sim.rules().weapon_store.find_weapon_template(&w.template_name) {
                        w.template = t.clone();
                    }
                }
            }
        } else if xfer.mode() == XferMode::Load {
            *slot = None;
        }
        Ok(())
    }

    fn body_damage_state(sim: &dyn SimView, owner: ObjectSlot) -> BodyDamageType {
        let Some(o) = sim.object(owner) else {
            return BodyDamageType::Pristine;
        };
        let Some(body_slot) = o.body_slot else {
            return BodyDamageType::Pristine;
        };
        let ModuleSidecar::Body { damage_state, .. } = sim.module_sidecar(body_slot) else {
            return BodyDamageType::Pristine;
        };
        BodyDamageType::from_i32(damage_state as i32).unwrap_or(BodyDamageType::Pristine)
    }
}

impl BehaviorModule for FireWeaponWhenDamagedBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        let owner = ctx.owner_slot;

        self.reaction_weapon_pristine =
            Self::init_and_reload_weapon(&self.data.reaction_weapon_pristine, sim, owner);
        self.reaction_weapon_damaged =
            Self::init_and_reload_weapon(&self.data.reaction_weapon_damaged, sim, owner);
        self.reaction_weapon_really_damaged =
            Self::init_and_reload_weapon(&self.data.reaction_weapon_really_damaged, sim, owner);
        self.reaction_weapon_rubble =
            Self::init_and_reload_weapon(&self.data.reaction_weapon_rubble, sim, owner);

        self.continuous_weapon_pristine =
            Self::init_and_reload_weapon(&self.data.continuous_weapon_pristine, sim, owner);
        self.continuous_weapon_damaged =
            Self::init_and_reload_weapon(&self.data.continuous_weapon_damaged, sim, owner);
        self.continuous_weapon_really_damaged =
            Self::init_and_reload_weapon(&self.data.continuous_weapon_really_damaged, sim, owner);
        self.continuous_weapon_rubble =
            Self::init_and_reload_weapon(&self.data.continuous_weapon_rubble, sim, owner);

        if self.data.initially_active != FALSE {
            self.upgrade
                .give_self_upgrade(sim, ctx, &self.data.upgrade_mux, |_, _| {});
        }

        let has_continuous = self.data.continuous_weapon_pristine.is_some()
            || self.data.continuous_weapon_damaged.is_some()
            || self.data.continuous_weapon_really_damaged.is_some()
            || self.data.continuous_weapon_rubble.is_some();

        if self.is_upgrade_active() != FALSE && has_continuous {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        } else {
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
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

        Self::xfer_opt_weapon(xfer, sim, &mut self.reaction_weapon_pristine)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.reaction_weapon_damaged)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.reaction_weapon_really_damaged)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.reaction_weapon_rubble)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.continuous_weapon_pristine)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.continuous_weapon_damaged)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.continuous_weapon_really_damaged)?;
        Self::xfer_opt_weapon(xfer, sim, &mut self.continuous_weapon_rubble)
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

    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        Some(self)
    }

    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        Some(self)
    }
}

impl DamageFacet for FireWeaponWhenDamagedBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        if self.is_upgrade_active() == FALSE {
            return;
        }
        if get_damage_type_flag(&self.data.damage_types, dmg.input.damage_type.as_i32()) == FALSE {
            return;
        }
        if dmg.output.actual_damage_dealt < self.data.damage_amount {
            return;
        }
        let bdt = Self::body_damage_state(sim, ctx.owner_slot);
        match bdt {
            BodyDamageType::Rubble => {
                Self::force_fire_weapon_slot(&mut self.reaction_weapon_rubble, sim, ctx.owner_slot)
            }
            BodyDamageType::ReallyDamaged => Self::force_fire_weapon_slot(
                &mut self.reaction_weapon_really_damaged,
                sim,
                ctx.owner_slot,
            ),
            BodyDamageType::Damaged => {
                Self::force_fire_weapon_slot(&mut self.reaction_weapon_damaged, sim, ctx.owner_slot)
            }
            _ => Self::force_fire_weapon_slot(
                &mut self.reaction_weapon_pristine,
                sim,
                ctx.owner_slot,
            ),
        }
    }
}

impl UpgradeFacet for FireWeaponWhenDamagedBehavior {
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

impl UpdateFacet for FireWeaponWhenDamagedBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.is_upgrade_active() == FALSE {
            return UpdateSleepTime::FOREVER;
        }
        let bdt = Self::body_damage_state(sim, ctx.owner_slot);
        match bdt {
            BodyDamageType::Rubble => Self::force_fire_weapon_slot(
                &mut self.continuous_weapon_rubble,
                sim,
                ctx.owner_slot,
            ),
            BodyDamageType::ReallyDamaged => Self::force_fire_weapon_slot(
                &mut self.continuous_weapon_really_damaged,
                sim,
                ctx.owner_slot,
            ),
            BodyDamageType::Damaged => Self::force_fire_weapon_slot(
                &mut self.continuous_weapon_damaged,
                sim,
                ctx.owner_slot,
            ),
            _ => Self::force_fire_weapon_slot(
                &mut self.continuous_weapon_pristine,
                sim,
                ctx.owner_slot,
            ),
        }
        UpdateSleepTime::NONE
    }
}

fn parse_opt_weapon_template(
    reader: &mut IniReader<'_>,
    parse_ctx: &ModuleParseContext<'_>,
) -> Result<Option<WeaponTemplate>, IniError> {
    let tok = reader.next_token(SEPS_DEFAULT)?;
    if tok.eq_ignore_ascii_case("None") {
        return Ok(None);
    }
    Ok(parse_ctx.weapon_store.find_weapon_template(tok).cloned())
}

pub fn parse_fire_weapon_when_damaged_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = FireWeaponWhenDamagedBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("StartsActive") {
            r.parse_bool(&mut data.initially_active)
        } else if field.eq_ignore_ascii_case("ReactionWeaponPristine") {
            data.reaction_weapon_pristine = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ReactionWeaponDamaged") {
            data.reaction_weapon_damaged = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ReactionWeaponReallyDamaged") {
            data.reaction_weapon_really_damaged = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ReactionWeaponRubble") {
            data.reaction_weapon_rubble = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ContinuousWeaponPristine") {
            data.continuous_weapon_pristine = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ContinuousWeaponDamaged") {
            data.continuous_weapon_damaged = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ContinuousWeaponReallyDamaged") {
            data.continuous_weapon_really_damaged = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("ContinuousWeaponRubble") {
            data.continuous_weapon_rubble = parse_opt_weapon_template(r, parse_ctx)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("DamageTypes") {
            data.damage_types = parse_damage_type_flags(r)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("DamageAmount") {
            r.parse_real(&mut data.damage_amount)
        } else if let Some(res) = data.upgrade_mux.parse_field(field, r, parse_ctx) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_fire_weapon_when_damaged_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<FireWeaponWhenDamagedBehaviorModuleData>()
        .expect("FireWeaponWhenDamagedBehavior data type mismatch")
        .clone();
    Box::new(FireWeaponWhenDamagedBehavior::new(typed))
}
