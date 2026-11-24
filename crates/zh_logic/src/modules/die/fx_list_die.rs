//! `PKT-217`: `FXListDie` (`FXListDie.h` / `FXListDie.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{parse_die_data_with, DieModule, DieMuxData};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpgradeFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{UpgradeMux, UpgradeMuxData};
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `FXListDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FXListDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub default_death_fx: Option<String>,
    pub mux_data: UpgradeMuxData,
    pub orient_to_object: Bool,
    pub initially_active: Bool,
}

impl ModuleData for FXListDieModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(FacetMask::DIE.0 | FacetMask::UPGRADE.0)
    }
}

pub fn parse_fx_list_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut default_death_fx: Option<String> = None;
    let mut mux_data = UpgradeMuxData::new();
    let mut orient_to_object: Bool = TRUE;
    let mut initially_active: Bool = TRUE;
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if let Some(res) = mux_data.parse_field(field, reader, ctx) {
                res
            } else if field.eq_ignore_ascii_case("StartsActive") {
                reader.parse_bool(&mut initially_active)
            } else if field.eq_ignore_ascii_case("DeathFX") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    default_death_fx = None;
                } else {
                    let resolved = ctx
                        .fx_list_names
                        .iter()
                        .find(|n| n.eq_ignore_ascii_case(tok))
                        .cloned()
                        .unwrap_or_else(|| tok.to_owned());
                    default_death_fx = Some(resolved);
                }
                Ok(())
            } else if field.eq_ignore_ascii_case("OrientToObject") {
                reader.parse_bool(&mut orient_to_object)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(FXListDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        default_death_fx,
        mux_data,
        orient_to_object,
        initially_active,
    }))
}

#[derive(Debug, Clone)]
pub struct FXListDie {
    base: DieModule,
    mux: UpgradeMux,
    mux_data: UpgradeMuxData,
    pub default_death_fx: Option<String>,
    pub orient_to_object: Bool,
    pub initially_active: Bool,
}

impl FXListDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<FXListDieModuleData>()
            .expect("FXListDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            mux: UpgradeMux::new(),
            mux_data: typed.mux_data.clone(),
            default_death_fx: typed.default_death_fx.clone(),
            orient_to_object: typed.orient_to_object,
            initially_active: typed.initially_active,
        })
    }

    #[inline]
    pub fn is_triggered_by(&self, upgrade: &str) -> Bool {
        self.mux_data.is_triggered_by(upgrade)
    }

    #[inline]
    pub fn is_upgrade_active(&self) -> Bool {
        self.mux.is_already_upgraded()
    }
}

impl BehaviorModule for FXListDie {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.initially_active != FALSE {
            self.give_self_upgrade(sim, ctx);
        }
    }

    fn apply_deferred_op(
        &mut self,
        op: DeferredSelfOp,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) {
        if let DeferredSelfOp::UpgradeResetKey(key) = op {
            if let Some(mask) = sim
                .rules()
                .upgrade_center
                .find_upgrade_by_key(key)
                .map(|t| t.upgrade_mask().clone())
            {
                let _ = self.reset_upgrade(sim, ctx, &mask);
            }
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.base.crc(xfer);
    }

    /// Serializes `FXListDie::xfer` (`version(1)` + `DieModule::xfer` ONLY; omits `UpgradeMux`).
    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        self.base.xfer_subclass(xfer)
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

impl DieFacet for FXListDie {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.is_upgrade_active() == FALSE {
            return;
        }
        if self.base.is_die_applicable(sim, ctx, dmg) == FALSE {
            return;
        }

        let mut activation = UpgradeMaskType::new();
        let mut conflicting = UpgradeMaskType::new();
        self.mux_data.get_upgrade_activation_masks(
            sim.rules(),
            &mut activation,
            &mut conflicting,
        );

        if sim.object_has_upgrade(ctx.owner_slot, &conflicting) != FALSE {
            return;
        }
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            if sim.player_has_upgrade(team_id, &conflicting) != FALSE {
                return;
            }
        }

        if let Some(ref fx_name) = self.default_death_fx {
            let pos = sim
                .object(ctx.owner_slot)
                .map(|o| *o.thing.get_position())
                .unwrap_or_default();
            sim.trigger_fx_list(fx_name, &pos);
        }
    }

    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.base.is_die_applicable(sim, ctx, dmg)
    }
}

impl UpgradeFacet for FXListDie {
    fn is_already_upgraded(&self) -> Bool {
        self.mux.is_already_upgraded()
    }

    fn attempt_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.mux
            .attempt_upgrade(sim, ctx, &self.mux_data, key_mask, |_, _| {})
    }

    fn would_upgrade(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.mux.would_upgrade(sim.rules(), &self.mux_data, key_mask)
    }

    fn reset_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.mux.reset_upgrade(sim.rules(), &self.mux_data, key_mask)
    }

    fn test_upgrade_conditions(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.mux
            .test_upgrade_conditions(sim.rules(), &self.mux_data, key_mask)
    }

    fn get_upgrade_activation_masks(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        activation: &mut UpgradeMaskType,
        conflicting: &mut UpgradeMaskType,
    ) {
        self.mux_data
            .get_upgrade_activation_masks(sim.rules(), activation, conflicting);
    }

    fn perform_upgrade_fx(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.mux_data.perform_upgrade_fx(sim, ctx.owner_slot);
    }

    fn give_self_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.mux.give_self_upgrade(sim, ctx, &self.mux_data, |_, _| {});
    }

    fn force_refresh_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.mux.force_refresh_upgrade(sim, ctx, |_, _| {});
    }
}
