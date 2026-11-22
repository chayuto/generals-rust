//! `PKT-393`: Named lists of object template names for scripts & counting (`ObjectTypes.cpp`).

use crate::arena::ThingTemplateId;
use crate::object::thing_factory::ThingFactory;
use crate::object::thing_template::ThingTemplate;
use zh_common::types::{Bool, Int, UnsignedShort, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// Script/AI named list of object template names (`GameLogic/ObjectTypes.h`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObjectTypes {
    list_name: String,
    object_types: Vec<String>,
}

impl ObjectTypes {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_name(list_name: impl Into<String>) -> Self {
        Self {
            list_name: list_name.into(),
            object_types: Vec::new(),
        }
    }

    pub fn add_object_type(&mut self, object_type: &str) {
        if self.is_in_set_name(object_type) != FALSE {
            return;
        }
        self.object_types.push(object_type.to_owned());
    }

    pub fn remove_object_type(&mut self, object_type: &str) {
        if let Some(idx) = self.object_types.iter().position(|s| s == object_type) {
            self.object_types.remove(idx);
        }
    }

    #[inline]
    pub fn get_list_name(&self) -> &str {
        &self.list_name
    }

    #[inline]
    pub fn set_list_name(&mut self, list_name: impl Into<String>) {
        self.list_name = list_name.into();
    }

    #[inline]
    pub fn is_in_set_name(&self, object_type: &str) -> Bool {
        if self.object_types.iter().any(|s| s == object_type) {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn is_in_set_template(&self, template: Option<&ThingTemplate>) -> Bool {
        match template {
            Some(t) => self.is_in_set_name(t.get_name()),
            None => FALSE,
        }
    }

    #[inline]
    pub fn get_list_size(&self) -> usize {
        self.object_types.len()
    }

    #[inline]
    pub fn get_nth_in_list(&self, index: usize) -> &str {
        self.object_types
            .get(index)
            .map_or("", String::as_str)
    }

    /// Resolves known template IDs and resizes `counts` (`ObjectTypes.cpp:222`).
    pub fn prep_for_player_counting(
        &self,
        factory: &ThingFactory,
        templates: &mut Vec<ThingTemplateId>,
        counts: &mut Vec<Int>,
    ) -> Int {
        for name in &self.object_types {
            if let Some(id) = factory.find_template_id(name) {
                templates.push(id);
            }
        }
        let ret_val = templates.len();
        counts.resize(ret_val, 0);
        ret_val as Int
    }

    /// Returns `TRUE` if `can_build` returns `TRUE` for any resolved template in the list.
    pub fn can_build_any(
        &self,
        factory: &ThingFactory,
        mut can_build: impl FnMut(&ThingTemplate) -> Bool,
    ) -> Bool {
        for name in &self.object_types {
            if let Some(tmpl) = factory.find_template_final(name) {
                if can_build(tmpl) != FALSE {
                    return TRUE;
                }
            }
        }
        FALSE
    }
}

impl Snapshot for ObjectTypes {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_ascii_string(&mut self.list_name)?;

        let mut object_types_count = self.object_types.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut object_types_count)?;

        if xfer.mode() == XferMode::Save || xfer.mode() == XferMode::Crc {
            for s in &mut self.object_types {
                xfer.xfer_ascii_string(s)?;
            }
        } else {
            if !self.object_types.is_empty() {
                return Err(XferError::InvalidData);
            }
            for _ in 0..object_types_count {
                let mut type_name = String::new();
                xfer.xfer_ascii_string(&mut type_name)?;
                self.object_types.push(type_name);
            }
        }
        Ok(())
    }
}
