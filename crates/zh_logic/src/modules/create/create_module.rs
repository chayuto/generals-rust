//! `PKT-168`: Base `CreateModule` interface state, common `ModuleData`, and `Snapshot`
//! (`CreateModule.h` / `CreateModule.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{ModuleData, ModuleSnapshotHeader};
use crate::sim::SimApi;
use zh_common::types::{Bool, NameKeyType, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Shared trivial `ModuleData` for `CreateModule` subclasses with no subclass INI fields.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ModuleData for CreateModuleData {
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
        FacetMask::CREATE
    }
}

/// Shared runtime state and C++ `CreateModule` base methods (`CreateModule.cpp`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CreateModuleBase {
    pub module_name_key: NameKeyType,
    pub need_to_run_on_build_complete: Bool,
}

impl CreateModuleBase {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            module_name_key,
            need_to_run_on_build_complete: TRUE,
        }
    }

    #[inline]
    pub const fn should_do_on_build_complete(&self) -> Bool {
        self.need_to_run_on_build_complete
    }

    #[inline]
    pub fn on_build_complete(&mut self) {
        self.need_to_run_on_build_complete = FALSE;
    }

    /// Serializes `CreateModule::xfer` (version `1` + `BehaviorModule::xfer` + `m_needToRunOnBuildComplete`).
    pub fn xfer_base(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        ModuleSnapshotHeader::xfer_behavior_module_header(xfer)?;
        xfer.xfer_bool(&mut self.need_to_run_on_build_complete)
    }

    /// Serializes a trivial subclass with no additional fields (`xferVersion(1)` + `CreateModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer_base(xfer)
    }
}

/// Abstract base `CreateModule` wrapper used by standalone `PKT-168` verification tests.
#[derive(Debug, Clone)]
pub struct CreateModule {
    pub base: CreateModuleBase,
}

impl CreateModule {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: CreateModuleBase::new(module_name_key),
        }
    }

    #[inline]
    pub fn on_create(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    #[inline]
    pub fn on_build_complete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.base.on_build_complete();
    }

    #[inline]
    pub const fn should_do_on_build_complete(&self) -> Bool {
        self.base.should_do_on_build_complete()
    }

    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        self.base.xfer_base(xfer)
    }
}
