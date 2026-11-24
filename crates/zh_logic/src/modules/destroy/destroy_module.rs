//! `PKT-170`: Base `DestroyModule` interface state and `Snapshot`
//! (`DestroyModule.h` / `DestroyModule.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{ModuleData, ModuleSnapshotHeader};
use crate::sim::SimApi;
use zh_common::types::{NameKeyType, XferVersion};
use zh_common::xfer::{Xfer, XferError};

/// Shared `ModuleData` for `DestroyModule` subclasses with no extra fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DestroyModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ModuleData for DestroyModuleData {
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
        FacetMask::DESTROY
    }
}

/// Shared runtime state and C++ `DestroyModule` serialization (`DestroyModule.cpp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DestroyModuleBase {
    pub module_name_key: NameKeyType,
}

impl DestroyModuleBase {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self { module_name_key }
    }

    /// Serializes `DestroyModule::xfer` (version `1` + `BehaviorModule::xfer`).
    pub fn xfer_base(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        ModuleSnapshotHeader::xfer_behavior_module_header(xfer)
    }

    /// Serializes a subclass header (`xferVersion(1)` + `DestroyModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer_base(xfer)
    }
}

/// Abstract base `DestroyModule` wrapper used by standalone `PKT-170` verification tests.
#[derive(Debug, Clone)]
pub struct DestroyModule {
    pub base: DestroyModuleBase,
}

impl DestroyModule {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: DestroyModuleBase::new(module_name_key),
        }
    }

    #[inline]
    pub fn on_destroy(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.base.xfer_base(xfer)
    }
}
