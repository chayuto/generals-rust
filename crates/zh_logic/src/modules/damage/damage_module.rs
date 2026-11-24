//! `PKT-169`: Base `DamageModule` interface state, `DamageModuleData`, and `Snapshot`
//! (`DamageModule.h` / `DamageModule.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::module::{ModuleData, ModuleSnapshotHeader};
use crate::sim::SimApi;
use zh_common::types::{NameKeyType, XferVersion};
use zh_common::xfer::{Xfer, XferError};

/// Shared trivial `ModuleData` for `DamageModule` subclasses with no extra fields (`BoneFXDamage`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ModuleData for DamageModuleData {
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
        FacetMask::DAMAGE
    }
}

/// Shared runtime state and C++ `DamageModule` serialization (`DamageModule.cpp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DamageModuleBase {
    pub module_name_key: NameKeyType,
}

impl DamageModuleBase {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self { module_name_key }
    }

    /// Serializes `DamageModule::xfer` (version `1` + `BehaviorModule::xfer`).
    pub fn xfer_base(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        ModuleSnapshotHeader::xfer_behavior_module_header(xfer)
    }

    /// Serializes a subclass header (`xferVersion(1)` + `DamageModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer_base(xfer)
    }
}

/// Abstract base `DamageModule` wrapper used by standalone `PKT-169` verification tests.
#[derive(Debug, Clone)]
pub struct DamageModule {
    pub base: DamageModuleBase,
}

impl DamageModule {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: DamageModuleBase::new(module_name_key),
        }
    }

    #[inline]
    pub fn on_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}

    #[inline]
    pub fn on_healing(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dmg: &mut DamageInfo,
    ) {
    }

    #[inline]
    pub fn on_body_damage_state_change(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _dmg: &DamageInfo,
        _old_state: BodyDamageType,
        _new_state: BodyDamageType,
    ) {
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.base.xfer_base(xfer)
    }
}
