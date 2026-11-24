//! `PKT-166`: Base `BodyModule` interface state, `BodyModuleData`, and `Snapshot`
//! (`BodyModule.h` / `BodyModule.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::module::{BehaviorModule, BodyFacet, ModuleData, ModuleSnapshotHeader};
use crate::sim::{SimApi, SimView};
use zh_common::types::{NameKeyType, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Shared trivial `ModuleData` for `BodyModule` subclasses with no subclass INI fields
/// (`InactiveBody`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BodyModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ModuleData for BodyModuleData {
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
        FacetMask::BODY
    }
}

/// Shared runtime state and C++ `BodyModule` serialization (`BodyModule.h` / `BodyModule.cpp`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BodyModuleBase {
    pub module_name_key: NameKeyType,
    pub damage_scalar: Real,
}

impl BodyModuleBase {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            module_name_key,
            damage_scalar: 1.0,
        }
    }

    #[inline]
    pub fn apply_damage_scalar(&mut self, scalar: Real) {
        self.damage_scalar *= scalar;
    }

    #[inline]
    pub const fn get_damage_scalar(&self) -> Real {
        self.damage_scalar
    }

    #[inline]
    pub const fn get_last_damage_timestamp() -> Option<UnsignedInt> {
        #[cfg(feature = "retail-crc")]
        {
            Some(0)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            None
        }
    }

    #[inline]
    pub const fn get_last_healing_timestamp() -> Option<UnsignedInt> {
        #[cfg(feature = "retail-crc")]
        {
            Some(0)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            None
        }
    }

    /// Serializes `BodyModule::xfer` (version `1` + `BehaviorModule::xfer` + `m_damageScalar`).
    pub fn xfer_base(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        ModuleSnapshotHeader::xfer_body_module_header(xfer, &mut self.damage_scalar)
    }

    /// Serializes a subclass header (`xferVersion(1)` + `BodyModule::xfer`).
    pub fn xfer_subclass(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.xfer_base(xfer)
    }
}

/// Abstract base `BodyModule` wrapper used by standalone `PKT-166` verification tests.
#[derive(Debug, Clone)]
pub struct BodyModule {
    pub base: BodyModuleBase,
}

impl BodyModule {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: BodyModuleBase::new(module_name_key),
        }
    }
}

impl BehaviorModule for BodyModule {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        self.base.xfer_base(xfer)
    }

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl BodyFacet for BodyModule {
    fn attempt_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}

    fn get_health(&self) -> Real {
        0.0
    }

    fn get_damage_scalar(&self) -> Real {
        self.base.get_damage_scalar()
    }

    fn apply_damage_scalar(&mut self, scalar: Real) {
        self.base.apply_damage_scalar(scalar);
    }

    fn get_last_damage_timestamp(&self) -> Option<UnsignedInt> {
        BodyModuleBase::get_last_damage_timestamp()
    }

    fn get_last_healing_timestamp(&self) -> Option<UnsignedInt> {
        BodyModuleBase::get_last_healing_timestamp()
    }

    fn is_indestructible(&self) -> zh_common::types::Bool {
        TRUE
    }

    fn is_inactive_body(&self) -> zh_common::types::Bool {
        FALSE
    }
}
