//! `PKT-397`: Global template registry and lifecycle manager (`ThingFactory.cpp`).

use std::collections::BTreeMap;

use crate::arena::ThingTemplateId;
use crate::object::overridable::OverridableEntry;
use crate::object::r#override::OverrideStore;
use crate::object::thing_template::{ThingTemplate, ThingTemplateParseContext};
use crate::rules::production_prerequisite::{ThingTemplateRef, ThingTemplateRegistry};
use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::types::{UnsignedInt, UnsignedShort, TRUE};

/// Read-only registry snapshot used during `resolve_names` to avoid aliasing mutable template entries.
struct RegistrySnapshot {
    by_name: BTreeMap<String, UnsignedShort>,
    display_names: BTreeMap<UnsignedShort, Vec<u16>>,
}

impl ThingTemplateRegistry for RegistrySnapshot {
    fn find_template(&self, name: &str) -> Option<ThingTemplateRef> {
        self.by_name
            .get(name)
            .map(|&id| ThingTemplateRef(id as usize))
    }

    fn display_name_utf16(&self, tmpl: ThingTemplateRef) -> &[u16] {
        self.display_names
            .get(&(tmpl.0 as UnsignedShort))
            .map_or(&[], Vec::as_slice)
    }
}

/// Manages `ThingTemplate` definitions, reskins, overrides, and match resets (`ThingFactory`).
#[derive(Debug, Default)]
pub struct ThingFactory {
    /// Stored in creation order; `entries.last()` corresponds to C++ `m_firstTemplate` (head-prepend).
    entries: Vec<OverridableEntry<ThingTemplate>>,
    /// Exact case-sensitive lookup matching C++ `ThingTemplateHashMap` (`ThingFactory.h:44`, `STLTypedefs.h:224`, `ThingFactory.cpp:303`).
    by_name: BTreeMap<String, usize>,
    by_id: BTreeMap<UnsignedShort, usize>,
    display_cache: BTreeMap<UnsignedShort, Vec<u16>>,
    next_template_id: UnsignedShort,
}

impl ThingFactory {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_name: BTreeMap::new(),
            by_id: BTreeMap::new(),
            display_cache: BTreeMap::new(),
            next_template_id: 1,
        }
    }

    pub fn init(&mut self) {}

    pub fn update(&mut self) {}

    #[inline]
    pub const fn next_template_id(&self) -> UnsignedShort {
        self.next_template_id
    }

    /// Returns the most recently added base template, matching C++ `m_firstTemplate`.
    #[inline]
    pub fn first_template(&self) -> Option<&ThingTemplate> {
        self.entries.last().map(OverridableEntry::get_base)
    }

    #[inline]
    pub fn find_template_id(&self, name: &str) -> Option<ThingTemplateId> {
        self.by_name
            .get(name)
            .map(|&idx| ThingTemplateId(self.entries[idx].get_base().get_template_id() as u32))
    }

    /// Finds a template by database name without resolving overrides (`ThingFactory::findTemplate`).
    #[inline]
    pub fn find_template(&self, name: &str) -> Option<&ThingTemplate> {
        self.by_name
            .get(name)
            .map(|&idx| self.entries[idx].get_base())
    }

    #[inline]
    pub fn find_template_final(&self, name: &str) -> Option<&ThingTemplate> {
        self.by_name
            .get(name)
            .map(|&idx| self.entries[idx].get_final_override())
    }

    #[inline]
    pub fn find_template_final_mut(&mut self, name: &str) -> Option<&mut ThingTemplate> {
        let &idx = self.by_name.get(name)?;
        Some(self.entries[idx].get_final_override_mut())
    }

    #[inline]
    pub fn find_by_template_id(&self, id: UnsignedShort) -> Option<&ThingTemplate> {
        self.by_id
            .get(&id)
            .map(|&idx| self.entries[idx].get_base())
    }

    #[inline]
    pub fn find_by_template_id_final(&self, id: UnsignedShort) -> Option<&ThingTemplate> {
        self.by_id
            .get(&id)
            .map(|&idx| self.entries[idx].get_final_override())
    }

    /// Creates a new base template, copying from `DefaultThingTemplate` if present (`ThingFactory.cpp:303`).
    pub fn new_template(
        &mut self,
        name: &str,
        default_occlusion_delay: UnsignedInt,
    ) -> usize {
        let mut tmpl = if let Some(&def_idx) = self.by_name.get("DefaultThingTemplate") {
            let mut copy = self.entries[def_idx].get_base().clone();
            copy.set_copied_from_default();
            copy
        } else {
            ThingTemplate::new(default_occlusion_delay)
        };
        let id = self.next_template_id;
        self.next_template_id = self.next_template_id.wrapping_add(1);
        tmpl.template_id = id;
        tmpl.name_string = name.to_owned();

        let idx = self.entries.len();
        self.entries.push(OverridableEntry::new(tmpl));
        self.by_name.insert(name.to_owned(), idx);
        self.by_id.insert(id, idx);
        idx
    }

    /// Appends an override node onto the entry at `idx` (`ThingFactory::newOverride`).
    pub fn new_override(&mut self, idx: usize) -> &mut ThingTemplate {
        let mut child_copy = self.entries[idx].get_final_override().clone();
        child_copy.set_copied_from_default();
        self.entries[idx].append_override(child_copy)
    }

    /// Parses one `Object` or `ObjectReskin` block (`ThingFactory::parseObjectDefinition`).
    pub fn parse_object_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        name: &str,
        reskin_from: &str,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<(), IniError> {
        let (idx, target_is_override) = if let Some(&existing_idx) = self.by_name.get(name) {
            if ini.get_load_type() != IniLoadType::CreateOverrides {
                return Err(IniError::InvalidData);
            }
            let _ = self.new_override(existing_idx);
            (existing_idx, true)
        } else {
            let created_idx = self.new_template(name, ctx.default_occlusion_delay);
            if ini.get_load_type() == IniLoadType::CreateOverrides {
                self.entries[created_idx].mark_as_override();
            }
            (created_idx, false)
        };

        if !reskin_from.is_empty() {
            let Some(&reskin_idx) = self.by_name.get(reskin_from) else {
                return Err(IniError::InvalidData);
            };
            let reskin_snapshot = self.entries[reskin_idx].get_base().clone();
            let reskin_id = reskin_snapshot.get_template_id();
            let t = if target_is_override {
                self.entries[idx].get_final_override_mut()
            } else {
                self.entries[idx].get_base_mut()
            };
            t.copy_from(&reskin_snapshot);
            t.set_copied_from_default();
            t.reskinned_from_id = Some(reskin_id);
            t.parse_object_fields(ini, ctx, true)?;
            t.validate();
        } else {
            let t = if target_is_override {
                self.entries[idx].get_final_override_mut()
            } else {
                self.entries[idx].get_base_mut()
            };
            t.parse_object_fields(ini, ctx, false)?;
            t.validate();
        }

        if ini.get_load_type() == IniLoadType::CreateOverrides {
            self.refresh_display_cache();
            self.resolve_one(idx, target_is_override);
        }
        Ok(())
    }

    /// Parses `Object` and `ObjectReskin` top-level blocks from an INI stream.
    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        ctx: &mut ThingTemplateParseContext<'_>,
    ) -> Result<UnsignedInt, IniError> {
        ini.load_blocks(None, |block, reader| {
            if block.eq_ignore_ascii_case("Object") {
                let _kw = reader.next_token(SEPS_DEFAULT)?;
                let name = reader.next_token(SEPS_DEFAULT)?.to_owned();
                self.parse_object_definition(reader, &name, "", ctx)
            } else if block.eq_ignore_ascii_case("ObjectReskin") {
                let _kw = reader.next_token(SEPS_DEFAULT)?;
                let name = reader.next_token(SEPS_DEFAULT)?.to_owned();
                let reskin_from = reader.next_token(SEPS_DEFAULT)?.to_owned();
                self.parse_object_definition(reader, &name, &reskin_from, ctx)
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }

    fn build_snapshot(&self) -> RegistrySnapshot {
        let mut by_name = BTreeMap::new();
        for (name, &idx) in &self.by_name {
            by_name.insert(name.clone(), self.entries[idx].get_base().get_template_id());
        }
        RegistrySnapshot {
            by_name,
            display_names: self.display_cache.clone(),
        }
    }

    fn refresh_display_cache(&mut self) {
        self.display_cache.clear();
        for entry in &self.entries {
            let t = entry.get_base();
            self.display_cache
                .insert(t.get_template_id(), t.get_display_name().to_vec());
        }
    }

    fn resolve_one(&mut self, idx: usize, resolve_final: bool) {
        let snapshot = self.build_snapshot();
        let target = if resolve_final {
            self.entries[idx].get_final_override_mut()
        } else {
            self.entries[idx].get_base_mut()
        };
        let facility_refs = target.resolve_names(&snapshot);
        for fac_ref in facility_refs {
            if let Some(&fac_idx) = self.by_id.get(&(fac_ref.0 as UnsignedShort)) {
                self.entries[fac_idx]
                    .get_final_override_mut()
                    .is_build_facility = TRUE;
            }
        }
    }

    /// Post-load resolution pass across all registered templates (`ThingFactory::postProcessLoad`).
    pub fn post_process_load(&mut self) {
        self.refresh_display_cache();
        let len = self.entries.len();
        for idx in (0..len).rev() {
            self.resolve_one(idx, false);
        }
    }

    /// Cleans up map/scenario overrides and restores `next_template_id` (`ThingFactory::reset`).
    pub fn reset(&mut self) {
        self.entries.retain_mut(OverridableEntry::delete_overrides);
        self.by_name.clear();
        self.by_id.clear();
        for (idx, entry) in self.entries.iter().enumerate() {
            let base = entry.get_base();
            self.by_name.insert(base.name_string.clone(), idx);
            self.by_id.insert(base.template_id, idx);
        }
        self.refresh_display_cache();
        self.next_template_id = self
            .entries
            .last()
            .map_or(1, |e| e.get_base().get_template_id().wrapping_add(1));
    }
}

impl OverrideStore<ThingTemplateId, ThingTemplate> for ThingFactory {
    #[inline]
    fn get_overridable(&self, key: ThingTemplateId) -> Option<&OverridableEntry<ThingTemplate>> {
        self.by_id
            .get(&(key.0 as UnsignedShort))
            .map(|&idx| &self.entries[idx])
    }

    #[inline]
    fn get_overridable_mut(
        &mut self,
        key: ThingTemplateId,
    ) -> Option<&mut OverridableEntry<ThingTemplate>> {
        let &idx = self.by_id.get(&(key.0 as UnsignedShort))?;
        Some(&mut self.entries[idx])
    }
}

impl ThingTemplateRegistry for ThingFactory {
    fn find_template(&self, name: &str) -> Option<ThingTemplateRef> {
        self.by_name
            .get(name)
            .map(|&idx| ThingTemplateRef(self.entries[idx].get_base().get_template_id() as usize))
    }

    fn display_name_utf16(&self, tmpl: ThingTemplateRef) -> &[u16] {
        self.display_cache
            .get(&(tmpl.0 as UnsignedShort))
            .map_or(&[], Vec::as_slice)
    }
}
