//! `PKT-253`: Base `UpgradeMux`, `UpgradeMuxData`, `UpgradeModuleData`, and `UpgradeModule`
//! (`UpgradeModule.h`, `UpgradeModule.cpp`, `Module.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, ModuleSnapshotHeader, UpgradeFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{RulesRegistry, SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, TeamId, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Shared template configuration parsed from `UpgradeMuxData::getFieldParse()` (`Module.h`/`Module.cpp`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpgradeMuxData {
    pub trigger_upgrade_names: Vec<String>,
    pub activation_upgrade_names: Vec<String>,
    pub conflicting_upgrade_names: Vec<String>,
    pub removal_upgrade_names: Vec<String>,
    pub fx_list_upgrade: Option<String>,
    pub requires_all_triggers: Bool,
}

impl UpgradeMuxData {
    #[inline]
    pub const fn new() -> Self {
        Self {
            trigger_upgrade_names: Vec::new(),
            activation_upgrade_names: Vec::new(),
            conflicting_upgrade_names: Vec::new(),
            removal_upgrade_names: Vec::new(),
            fx_list_upgrade: None,
            requires_all_triggers: FALSE,
        }
    }

    /// Parses one `UpgradeMuxData` INI field, returning `None` if `field` belongs to a subclass.
    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("TriggeredBy") {
            let res = reader.parse_ascii_string_vector(&mut self.activation_upgrade_names);
            if res.is_ok() {
                self.trigger_upgrade_names = self.activation_upgrade_names.clone();
            }
            Some(res)
        } else if field.eq_ignore_ascii_case("ConflictsWith") {
            Some(reader.parse_ascii_string_vector(&mut self.conflicting_upgrade_names))
        } else if field.eq_ignore_ascii_case("RemovesUpgrades") {
            Some(reader.parse_ascii_string_vector(&mut self.removal_upgrade_names))
        } else if field.eq_ignore_ascii_case("FXListUpgrade") {
            let res = (|| {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    self.fx_list_upgrade = None;
                } else {
                    let resolved = parse_ctx
                        .fx_list_names
                        .iter()
                        .find(|n| n.eq_ignore_ascii_case(tok))
                        .cloned()
                        .unwrap_or_else(|| tok.to_owned());
                    self.fx_list_upgrade = Some(resolved);
                }
                Ok(())
            })();
            Some(res)
        } else if field.eq_ignore_ascii_case("RequiresAllTriggers") {
            Some(reader.parse_bool(&mut self.requires_all_triggers))
        } else {
            None
        }
    }

    /// Resolves `TriggeredBy` and `ConflictsWith` upgrade masks (`UpgradeMuxData::getUpgradeActivationMasks`).
    pub fn get_upgrade_activation_masks(
        &self,
        rules: &RulesRegistry,
        activation: &mut UpgradeMaskType,
        conflicting: &mut UpgradeMaskType,
    ) {
        activation.clear();
        conflicting.clear();
        let empty = UpgradeMaskType::new();

        for name in &self.activation_upgrade_names {
            if let Some(tmpl) = rules.upgrade_center.lookup_upgrade(name, &rules.name_keys) {
                activation.clear_and_set(&empty, tmpl.upgrade_mask());
            }
        }
        for name in &self.conflicting_upgrade_names {
            if let Some(tmpl) = rules.upgrade_center.lookup_upgrade(name, &rules.name_keys) {
                conflicting.clear_and_set(&empty, tmpl.upgrade_mask());
            }
        }
    }

    #[inline]
    pub const fn requires_all_activation_upgrades(&self) -> Bool {
        self.requires_all_triggers
    }

    /// Executes `FXListUpgrade` at `owner`'s position if configured (`UpgradeMuxData::performUpgradeFX`).
    pub fn perform_upgrade_fx(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if let Some(ref fx_name) = self.fx_list_upgrade {
            let pos = sim
                .object(owner)
                .map(|o| *o.thing.get_position())
                .unwrap_or_default();
            sim.trigger_fx_list(fx_name, &pos);
        }
    }

    /// Removes all upgrades listed in `RemovesUpgrades` from `owner` (`UpgradeMuxData::muxDataProcessUpgradeRemoval`).
    pub fn process_upgrade_removal(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        for name in &self.removal_upgrade_names {
            sim.remove_upgrade(owner, name);
        }
    }

    /// Case-insensitive trigger membership test (`UpgradeMuxData::isTriggeredBy`).
    pub fn is_triggered_by(&self, upgrade: &str) -> Bool {
        let names = if self.trigger_upgrade_names.is_empty() {
            &self.activation_upgrade_names
        } else {
            &self.trigger_upgrade_names
        };
        Bool::from(names.iter().any(|n| n.eq_ignore_ascii_case(upgrade)))
    }
}

/// Mix-in execution state matching C++ `UpgradeMux` (`UpgradeModule.h` / `UpgradeModule.cpp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct UpgradeMux {
    pub upgrade_executed: Bool,
}

impl UpgradeMux {
    #[inline]
    pub const fn new() -> Self {
        Self {
            upgrade_executed: FALSE,
        }
    }

    #[inline]
    pub const fn is_already_upgraded(&self) -> Bool {
        self.upgrade_executed
    }

    #[inline]
    pub fn set_upgrade_executed(&mut self, executed: Bool) {
        self.upgrade_executed = executed;
    }

    /// Evaluates whether `key_mask` triggers activation without conflicts (`UpgradeMux::wouldUpgrade`).
    pub fn would_upgrade(
        &self,
        rules: &RulesRegistry,
        data: &UpgradeMuxData,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        let mut activation = UpgradeMaskType::new();
        let mut conflicting = UpgradeMaskType::new();
        data.get_upgrade_activation_masks(rules, &mut activation, &mut conflicting);

        if activation.any() != FALSE
            && key_mask.any() != FALSE
            && self.upgrade_executed == FALSE
            && key_mask.test_for_any(&conflicting) == FALSE
        {
            if data.requires_all_activation_upgrades() != FALSE {
                if key_mask.test_for_all(&activation) != FALSE {
                    return TRUE;
                }
            } else if key_mask.test_for_any(&activation) != FALSE {
                return TRUE;
            }
        }
        FALSE
    }

    /// Evaluates trigger and conflict masks regardless of `upgrade_executed` (`UpgradeMux::testUpgradeConditions`).
    pub fn test_upgrade_conditions(
        &self,
        rules: &RulesRegistry,
        data: &UpgradeMuxData,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        let mut activation = UpgradeMaskType::new();
        let mut conflicting = UpgradeMaskType::new();
        data.get_upgrade_activation_masks(rules, &mut activation, &mut conflicting);

        if key_mask.test_for_none(&conflicting) != FALSE {
            if activation.any() != FALSE {
                if data.requires_all_activation_upgrades() != FALSE {
                    if key_mask.test_for_all(&activation) != FALSE {
                        return TRUE;
                    }
                } else if key_mask.test_for_any(&activation) != FALSE {
                    return TRUE;
                }
            } else {
                return TRUE;
            }
        }
        FALSE
    }

    /// Resets `upgrade_executed` if `key_mask` overlaps `activation` (`UpgradeMux::resetUpgrade`).
    pub fn reset_upgrade(
        &mut self,
        rules: &RulesRegistry,
        data: &UpgradeMuxData,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        let mut activation = UpgradeMaskType::new();
        let mut conflicting = UpgradeMaskType::new();
        data.get_upgrade_activation_masks(rules, &mut activation, &mut conflicting);
        if key_mask.test_for_any(&activation) != FALSE && self.upgrade_executed != FALSE {
            self.upgrade_executed = FALSE;
            return TRUE;
        }
        FALSE
    }

    /// Performs upgrade FX, removal, `upgrade_impl`, and marks executed (`UpgradeMux::giveSelfUpgrade`).
    pub fn give_self_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        data: &UpgradeMuxData,
        upgrade_impl: impl FnOnce(&mut dyn SimApi, ModuleContext),
    ) {
        data.perform_upgrade_fx(sim, ctx.owner_slot);
        data.process_upgrade_removal(sim, ctx.owner_slot);
        upgrade_impl(sim, ctx);
        self.upgrade_executed = TRUE;
    }

    /// Checks `would_upgrade` and triggers `give_self_upgrade` on match (`UpgradeMux::attemptUpgrade`).
    pub fn attempt_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        data: &UpgradeMuxData,
        key_mask: &UpgradeMaskType,
        upgrade_impl: impl FnOnce(&mut dyn SimApi, ModuleContext),
    ) -> Bool {
        if self.would_upgrade(sim.rules(), data, key_mask) != FALSE {
            self.give_self_upgrade(sim, ctx, data, upgrade_impl);
            return TRUE;
        }
        FALSE
    }

    /// Re-runs `upgrade_impl` only if already executed (`UpgradeMux::forceRefreshUpgrade`).
    pub fn force_refresh_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        upgrade_impl: impl FnOnce(&mut dyn SimApi, ModuleContext),
    ) {
        if self.upgrade_executed != FALSE {
            upgrade_impl(sim, ctx);
        }
    }

    /// Serializes `UpgradeMux::upgradeMuxCRC` (`xfer_upgrade_mux` in CRC mode).
    pub fn crc(&self, xfer: &mut dyn Xfer) {
        let mut exec = self.upgrade_executed;
        let _ = ModuleSnapshotHeader::xfer_upgrade_mux(xfer, &mut exec);
    }

    /// Serializes `UpgradeMux::upgradeMuxXfer`.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_upgrade_mux(xfer, &mut self.upgrade_executed)
    }
}

/// Shared template data for `UpgradeModule` subclasses with no extra INI fields.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
}

impl ModuleData for UpgradeModuleData {
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
        FacetMask::UPGRADE
    }
}

/// Helper parsing `UpgradeModuleData` base fields and delegating remaining fields to `parse_extra`.
pub fn parse_upgrade_data_with(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
    mut parse_extra: impl FnMut(&str, &mut IniReader<'_>, &mut ModuleParseContext<'_>) -> Result<(), IniError>,
) -> Result<UpgradeModuleData, IniError> {
    let mut base = UpgradeModuleData {
        tag_key,
        module_name_key,
        mux_data: UpgradeMuxData::new(),
    };
    ini.read_block_fields(|field, reader| {
        if let Some(res) = base.mux_data.parse_field(field, reader, parse_ctx) {
            res
        } else {
            parse_extra(field, reader, parse_ctx)
        }
    })?;
    Ok(base)
}

/// Standard parser for `UpgradeModule` subclasses with no subclass INI fields.
pub fn parse_upgrade_module_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let data = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |_field, _reader, _ctx| Err(IniError::UnknownToken),
    )?;
    Ok(Box::new(data))
}

/// Base runtime state and snapshot wrapper matching C++ `UpgradeModule` (`UpgradeModule.h` / `UpgradeModule.cpp`).
#[derive(Debug, Clone)]
pub struct UpgradeModule {
    pub module_name_key: NameKeyType,
    pub mux: UpgradeMux,
    pub mux_data: UpgradeMuxData,
}

impl UpgradeModule {
    #[inline]
    pub fn new(module_name_key: NameKeyType, mux_data: UpgradeMuxData) -> Self {
        Self {
            module_name_key,
            mux: UpgradeMux::new(),
            mux_data,
        }
    }

    #[inline]
    pub const fn with_name_key(module_name_key: NameKeyType) -> Self {
        Self {
            module_name_key,
            mux: UpgradeMux::new(),
            mux_data: UpgradeMuxData::new(),
        }
    }

    #[inline]
    pub fn is_triggered_by(&self, upgrade: &str) -> Bool {
        self.mux_data.is_triggered_by(upgrade)
    }

    #[inline]
    pub fn crc(&self, xfer: &mut dyn Xfer) {
        self.mux.crc(xfer);
    }

    /// Serializes `UpgradeModule::xfer` (`version(1)` + `BehaviorModule::xfer` + `UpgradeMux::upgradeMuxXfer`).
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_upgrade_module_header(xfer, &mut self.mux.upgrade_executed)
    }

    /// Serializes a concrete subclass with no additional fields (`version(1)` + `UpgradeModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer(xfer)
    }
}

impl BehaviorModule for UpgradeModule {
    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
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
        UpgradeModule::crc(self, xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        UpgradeModule::xfer(self, xfer)
    }

    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        Some(self)
    }

    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        Some(self)
    }
}

impl UpgradeFacet for UpgradeModule {
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

/// Internal implementation trait shared by all 20 concrete `UpgradeModule` subclasses.
pub(crate) trait UpgradeBehaviorImpl: Send {
    fn base(&self) -> &UpgradeModule;
    fn base_mut(&mut self) -> &mut UpgradeModule;
    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext);

    fn is_sub_objects_upgrade(&self) -> Bool {
        FALSE
    }
    fn on_delete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn on_capture(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_team: TeamId,
        _new_team: TeamId,
    ) {
    }
    fn on_disabled_edge(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _disabled: Bool) {}
    fn load_post_process(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
}

macro_rules! impl_upgrade_behavior {
    ($ty:ty) => {
        impl $crate::modules::module::BehaviorModule for $ty {
            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self)
                    .module_name_key
            }

            fn on_delete(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::on_delete(
                    self, sim, ctx,
                );
            }

            fn on_capture(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                old_team: zh_common::types::TeamId,
                new_team: zh_common::types::TeamId,
            ) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::on_capture(
                    self, sim, ctx, old_team, new_team,
                );
            }

            fn on_disabled_edge(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                disabled: zh_common::types::Bool,
            ) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::on_disabled_edge(
                    self, sim, ctx, disabled,
                );
            }

            fn apply_deferred_op(
                &mut self,
                op: $crate::arena::DeferredSelfOp,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                if let $crate::arena::DeferredSelfOp::UpgradeResetKey(key) = op {
                    if let Some(mask) = sim
                        .rules()
                        .upgrade_center
                        .find_upgrade_by_key(key)
                        .map(|t| t.upgrade_mask().clone())
                    {
                        let _ = <Self as $crate::modules::module::UpgradeFacet>::reset_upgrade(
                            self, sim, ctx, &mask,
                        );
                    }
                }
            }

            fn crc(&self, xfer: &mut dyn zh_common::xfer::Xfer) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self).crc(xfer);
            }

            fn xfer(
                &mut self,
                xfer: &mut dyn zh_common::xfer::Xfer,
                _sim: &dyn $crate::sim::SimView,
            ) -> Result<(), zh_common::xfer::XferError> {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base_mut(self)
                    .xfer_subclass(xfer)
            }

            fn load_post_process(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::load_post_process(
                    self, sim, ctx,
                );
            }

            fn as_upgrade(&self) -> Option<&dyn $crate::modules::module::UpgradeFacet> {
                Some(self)
            }

            fn as_upgrade_mut(&mut self) -> Option<&mut dyn $crate::modules::module::UpgradeFacet> {
                Some(self)
            }
        }

        impl $crate::modules::module::UpgradeFacet for $ty {
            fn is_already_upgraded(&self) -> zh_common::types::Bool {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self)
                    .mux
                    .is_already_upgraded()
            }

            fn attempt_upgrade(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
                key_mask: &$crate::rules::upgrade::UpgradeMaskType,
            ) -> zh_common::types::Bool {
                if self.would_upgrade(sim, ctx, key_mask) != zh_common::types::FALSE {
                    self.give_self_upgrade(sim, ctx);
                    return zh_common::types::TRUE;
                }
                zh_common::types::FALSE
            }

            fn would_upgrade(
                &self,
                sim: &dyn $crate::sim::SimView,
                _ctx: $crate::arena::ModuleContext,
                key_mask: &$crate::rules::upgrade::UpgradeMaskType,
            ) -> zh_common::types::Bool {
                let base =
                    $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self);
                base.mux.would_upgrade(sim.rules(), &base.mux_data, key_mask)
            }

            fn reset_upgrade(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                _ctx: $crate::arena::ModuleContext,
                key_mask: &$crate::rules::upgrade::UpgradeMaskType,
            ) -> zh_common::types::Bool {
                let rules_ptr: *const $crate::sim::RulesRegistry = sim.rules();
                // SAFETY: `rules` is immutable and disjoint from `self`.
                let base =
                    $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base_mut(self);
                let rules = unsafe { &*rules_ptr };
                base.mux.reset_upgrade(rules, &base.mux_data, key_mask)
            }

            fn test_upgrade_conditions(
                &self,
                sim: &dyn $crate::sim::SimView,
                _ctx: $crate::arena::ModuleContext,
                key_mask: &$crate::rules::upgrade::UpgradeMaskType,
            ) -> zh_common::types::Bool {
                let base =
                    $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self);
                base.mux
                    .test_upgrade_conditions(sim.rules(), &base.mux_data, key_mask)
            }

            fn get_upgrade_activation_masks(
                &self,
                sim: &dyn $crate::sim::SimView,
                _ctx: $crate::arena::ModuleContext,
                activation: &mut $crate::rules::upgrade::UpgradeMaskType,
                conflicting: &mut $crate::rules::upgrade::UpgradeMaskType,
            ) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self)
                    .mux_data
                    .get_upgrade_activation_masks(sim.rules(), activation, conflicting);
            }

            fn perform_upgrade_fx(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self)
                    .mux_data
                    .perform_upgrade_fx(sim, ctx.owner_slot);
            }

            fn is_sub_objects_upgrade(&self) -> zh_common::types::Bool {
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::is_sub_objects_upgrade(
                    self,
                )
            }

            fn give_self_upgrade(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                {
                    let base =
                        $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base(self);
                    base.mux_data.perform_upgrade_fx(sim, ctx.owner_slot);
                    base.mux_data.process_upgrade_removal(sim, ctx.owner_slot);
                }
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::upgrade_implementation(
                    self, sim, ctx,
                );
                $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::base_mut(self)
                    .mux
                    .set_upgrade_executed(zh_common::types::TRUE);
            }

            fn force_refresh_upgrade(
                &mut self,
                sim: &mut dyn $crate::sim::SimApi,
                ctx: $crate::arena::ModuleContext,
            ) {
                if self.is_already_upgraded() != zh_common::types::FALSE {
                    $crate::modules::upgrade::upgrade_module::UpgradeBehaviorImpl::upgrade_implementation(
                        self, sim, ctx,
                    );
                }
            }
        }
    };
}

pub(crate) use impl_upgrade_behavior;
