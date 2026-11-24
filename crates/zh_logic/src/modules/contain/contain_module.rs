//! `PKT-265`: Base `ContainModule` interfaces, helper types, and `ContainModuleData`
//! (`ContainModule.h`, `ContainModule.cpp`).

use crate::arena::FacetMask;
use crate::modules::module::{ModuleData, ObjectEnterExitType};
use std::any::Any;
use zh_common::types::{Bool, NameKeyType, FALSE};

pub const CONTAIN_MAX_UNKNOWN: i32 = -1;

/// Base metadata marker for containment modules (`ContainModule.h`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ContainModuleData {
    #[inline]
    pub const fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
        }
    }
}

impl ModuleData for ContainModuleData {
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
        FacetMask::CONTAIN
    }
}

/// Checks whether `wants` represents a non-idle enter or exit intent (`ObjectEnterExitType`).
#[inline]
pub const fn is_wanting_enter_or_exit(wants: ObjectEnterExitType) -> Bool {
    if !matches!(wants, ObjectEnterExitType::WantsNeither) {
        1
    } else {
        FALSE
    }
}
