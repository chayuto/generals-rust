//! `PKT-269`: `FireWeaponWhenDeadBehaviorModuleData` and `FireWeaponWhenDeadBehavior`
//! (`FireWeaponWhenDeadBehavior.h`, `FireWeaponWhenDeadBehavior.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{xfer_behavior_module_header, xfer_module_version};
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpgradeFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{UpgradeMux, UpgradeMuxData};
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `FireWeaponWhenDeadBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FireWeaponWhenDeadBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_mux: UpgradeMuxData,
    pub die_mux: DieMuxData,
    pub initially_active: Bool,
    pub death_weapon: Option<String>,
}

impl FireWeaponWhenDeadBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            upgrade_mux: UpgradeMuxData::new(),
            die_mux: DieMuxData::new(),
            initially_active: FALSE,
            death_weapon: None,
        }
    }
}

impl ModuleData for FireWeaponWhenDeadBehaviorModuleData {
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
        FacetMask(0x082)
    }
}

/// Runtime instance of `FireWeaponWhenDeadBehavior` (`PKT-269`).
pub struct FireWeaponWhenDeadBehavior {
    pub data: FireWeaponWhenDeadBehaviorModuleData,
    pub upgrade: UpgradeMux,
}

impl FireWeaponWhenDeadBehavior {
    pub fn new(data: FireWeaponWhenDeadBehaviorModuleData) -> Self {
        Self {
            data,
            upgrade: UpgradeMux::new(),
        }
    }

    #[inline]
    pub fn is_upgrade_active(&self) -> Bool {
        self.upgrade.is_already_upgraded()
    }
}

impl BehaviorModule for FireWeaponWhenDeadBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.data.initially_active != FALSE {
            self.upgrade
                .give_self_upgrade(sim, ctx, &self.data.upgrade_mux, |_, _| {});
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.upgrade.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        xfer_behavior_module_header(xfer)?;
        self.upgrade.xfer(xfer)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        Some(self)
    }

    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        Some(self)
    }
}

impl UpgradeFacet for FireWeaponWhenDeadBehavior {
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
        self.upgrade
            .attempt_upgrade(sim, ctx, mux, key_mask, |_, _| {})
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
        self.upgrade.give_self_upgrade(sim, ctx, mux, |_, _| {});
    }

    fn force_refresh_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.upgrade.force_refresh_upgrade(sim, ctx, |_, _| {});
    }
}

impl DieFacet for FireWeaponWhenDeadBehavior {
    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.data.die_mux.is_die_applicable(sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.is_upgrade_active() == FALSE {
            return;
        }
        let owner = ctx.owner_slot;
        if self.data.die_mux.is_die_applicable(sim, owner, dmg) == FALSE {
            return;
        }
        let Some(obj) = sim.object(owner) else {
            return;
        };
        if obj.status.test(ObjectStatusType::UnderConstruction as i32) != FALSE {
            return;
        }
        let obj_upgrades = obj.upgrades_completed.clone();
        let team_id = obj.team_id;
        let pos = *obj.thing.get_position();

        let mut activation = UpgradeMaskType::new();
        let mut conflicting = UpgradeMaskType::new();
        self.data.upgrade_mux.get_upgrade_activation_masks(
            sim.rules(),
            &mut activation,
            &mut conflicting,
        );

        if obj_upgrades.test_for_any(&conflicting) != FALSE {
            return;
        }
        if sim.player_has_upgrade(team_id, &conflicting) != FALSE {
            return;
        }

        if let Some(ref wpn_name) = self.data.death_weapon {
            sim.fire_weapon_template(owner, wpn_name, None, &pos);
        }
    }
}

pub fn parse_fire_weapon_when_dead_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = FireWeaponWhenDeadBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("StartsActive") {
            r.parse_bool(&mut data.initially_active)
        } else if field.eq_ignore_ascii_case("DeathWeapon") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.death_weapon = if tok.eq_ignore_ascii_case("None") {
                None
            } else {
                parse_ctx
                    .weapon_store
                    .find_weapon_template(tok)
                    .map(|wt| wt.name.clone())
            };
            Ok(())
        } else if let Some(res) = data.upgrade_mux.parse_field(field, r, parse_ctx) {
            res
        } else if let Some(res) = data.die_mux.parse_field(field, r) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_fire_weapon_when_dead_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<FireWeaponWhenDeadBehaviorModuleData>()
        .expect("FireWeaponWhenDeadBehavior data type mismatch")
        .clone();
    Box::new(FireWeaponWhenDeadBehavior::new(typed))
}
