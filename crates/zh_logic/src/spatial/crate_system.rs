//! `PKT-151` & `PKT-094`: `CrateTemplate`, `CrateSystem`, `INICrate`
//! (`Core/GameEngine/Include/GameLogic/CrateSystem.h`,
//! `Core/GameEngine/Source/GameLogic/System/CrateSystem.cpp`, &
//! `Core/GameEngine/Source/Common/INI/INICrate.cpp`).

use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::rules::science::{ScienceStore, ScienceType, SCIENCE_INVALID};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{
    IniError, IniLoadType, IniReader, SEPS_DEFAULT, VETERANCY_NAMES,
};
use zh_common::types::{Bool, Int, Real, FALSE};

/// Sentinel `VeterancyLevel::LEVEL_INVALID` matching C++ enum in `GameCommon.h`.
pub const VETERANCY_LEVEL_INVALID: Int = 4;

/// Weighted crate spawn entry in `CrateTemplate::m_possibleCrates`.
#[derive(Debug, Clone, PartialEq)]
pub struct CrateCreationEntry {
    pub crate_name: String,
    pub crate_chance: Real,
}

/// Overridable crate spawn rules matching C++ `CrateTemplate`.
#[derive(Debug, Clone, PartialEq)]
pub struct CrateTemplate {
    pub name: String,
    pub creation_chance: Real,
    pub veterancy_level: Int,
    pub killed_by_type_kindof: KindOfMaskType,
    pub killer_science: ScienceType,
    pub possible_crates: Vec<CrateCreationEntry>,
    pub is_owned_by_maker: Bool,
    is_override: bool,
    next_override: Option<Box<CrateTemplate>>,
}

impl Default for CrateTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl CrateTemplate {
    pub fn new() -> Self {
        Self {
            name: String::new(),
            creation_chance: 0.0,
            veterancy_level: VETERANCY_LEVEL_INVALID,
            killed_by_type_kindof: KindOfMaskType::new(),
            killer_science: SCIENCE_INVALID,
            possible_crates: Vec::new(),
            is_owned_by_maker: FALSE,
            is_override: false,
            next_override: None,
        }
    }

    #[inline]
    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_string();
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn mark_as_override(&mut self) {
        self.is_override = true;
    }

    #[inline]
    pub fn is_override(&self) -> bool {
        self.is_override
    }

    pub fn final_override(&self) -> &CrateTemplate {
        let mut cur = self;
        while let Some(next) = &cur.next_override {
            cur = next;
        }
        cur
    }

    pub fn final_override_mut(&mut self) -> &mut CrateTemplate {
        if self.next_override.is_some() {
            self.next_override
                .as_deref_mut()
                .unwrap()
                .final_override_mut()
        } else {
            self
        }
    }

    /// Recursively removes override nodes matching `Overridable::deleteOverrides`.
    /// Returns `false` if this root node is itself an override and should be dropped.
    pub fn delete_overrides(&mut self) -> bool {
        if self.is_override {
            false
        } else {
            if let Some(next) = &mut self.next_override {
                if !next.delete_overrides() {
                    self.next_override = None;
                }
            }
            true
        }
    }

    /// Parses a `KindOfMaskType` bitstring field (`NONE`, `+BIT`/`-BIT`, or `BIT ...`).
    pub fn parse_kindof_mask(
        reader: &mut IniReader<'_>,
        store: &mut KindOfMaskType,
    ) -> Result<(), IniError> {
        let mut found_normal = false;
        let mut found_add_or_sub = false;
        while let Some(token) = reader.next_token_or_null(SEPS_DEFAULT) {
            if token.eq_ignore_ascii_case("NONE") {
                if found_normal || found_add_or_sub {
                    return Err(IniError::InvalidNameList);
                }
                store.clear();
                break;
            }
            if let Some(name) = token.strip_prefix('+') {
                if found_normal {
                    return Err(IniError::InvalidNameList);
                }
                let bit = IniReader::scan_index_list(name, &KINDOF_NAMES)?;
                store.set(bit, 1);
                found_add_or_sub = true;
            } else if let Some(name) = token.strip_prefix('-') {
                if found_normal {
                    return Err(IniError::InvalidNameList);
                }
                let bit = IniReader::scan_index_list(name, &KINDOF_NAMES)?;
                store.set(bit, 0);
                found_add_or_sub = true;
            } else {
                if found_add_or_sub {
                    return Err(IniError::InvalidNameList);
                }
                if !found_normal {
                    store.clear();
                }
                let bit = IniReader::scan_index_list(token, &KINDOF_NAMES)?;
                store.set(bit, 1);
                found_normal = true;
            }
        }
        Ok(())
    }

    pub fn parse_from_ini(
        &mut self,
        reader: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), IniError> {
        loop {
            reader.read_line(None)?;
            let field = reader
                .next_token_or_null(SEPS_DEFAULT)
                .map(str::to_string);

            if let Some(field_name) = field {
                if field_name.eq_ignore_ascii_case("End")
                    || IniReader::is_end_of_block(reader.current_line_bytes()) != FALSE
                {
                    if reader.is_eof() && !field_name.eq_ignore_ascii_case("End") {
                        return Err(IniError::MissingEndToken);
                    }
                    break;
                }
                if field_name.eq_ignore_ascii_case("CreationChance") {
                    reader.parse_real(&mut self.creation_chance)?;
                } else if field_name.eq_ignore_ascii_case("VeterancyLevel") {
                    reader.parse_index_list(&mut self.veterancy_level, &VETERANCY_NAMES)?;
                } else if field_name.eq_ignore_ascii_case("KilledByType") {
                    Self::parse_kindof_mask(reader, &mut self.killed_by_type_kindof)?;
                } else if field_name.eq_ignore_ascii_case("CrateObject") {
                    let crate_name = reader.next_token(SEPS_DEFAULT)?.to_string();
                    let mut crate_chance: Real = 0.0;
                    reader.parse_real(&mut crate_chance)?;
                    self.possible_crates.push(CrateCreationEntry {
                        crate_name,
                        crate_chance,
                    });
                } else if field_name.eq_ignore_ascii_case("KillerScience") {
                    let tok = reader.next_token(SEPS_DEFAULT)?.to_string();
                    self.killer_science = science_store.friend_lookup_science(&tok, name_keys)?;
                } else if field_name.eq_ignore_ascii_case("OwnedByMaker") {
                    reader.parse_bool(&mut self.is_owned_by_maker)?;
                } else {
                    return Err(IniError::UnknownToken);
                }
            }

            if reader.is_eof() {
                return Err(IniError::MissingEndToken);
            }
        }
        Ok(())
    }
}

/// Collection of crate templates with `OVERRIDE<CrateTemplate>` lifecycle (`CrateSystem`).
#[derive(Debug, Clone, Default)]
pub struct CrateSystem {
    crate_templates: Vec<CrateTemplate>,
}

impl CrateSystem {
    pub fn new() -> Self {
        Self {
            crate_templates: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.reset();
    }

    pub fn reset(&mut self) {
        self.crate_templates.retain_mut(|t| t.delete_overrides());
    }

    pub fn update(&mut self) {}

    pub fn find_crate_template(&self, name: &str) -> Option<&CrateTemplate> {
        self.crate_templates
            .iter()
            .find(|t| t.name == name)
            .map(|t| t.final_override())
    }

    pub fn friend_find_crate_template_mut(&mut self, name: &str) -> Option<&mut CrateTemplate> {
        self.crate_templates
            .iter_mut()
            .find(|t| t.name == name)
            .map(|t| t.final_override_mut())
    }

    /// Allocates a new `CrateTemplate`, inheriting from `"DefaultCrate"`'s final override if present.
    pub fn new_crate_template(&mut self, name: &str) -> Option<usize> {
        if name.is_empty() {
            return None;
        }
        let mut ct = if let Some(def) = self.find_crate_template("DefaultCrate") {
            let mut cloned = def.clone();
            cloned.next_override = None;
            cloned
        } else {
            CrateTemplate::new()
        };
        ct.set_name(name);
        self.crate_templates.push(ct);
        Some(self.crate_templates.len() - 1)
    }

    pub fn parse_crate_template_definition(
        &mut self,
        reader: &mut IniReader<'_>,
        load_type: IniLoadType,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), IniError> {
        let name = reader.next_token(SEPS_DEFAULT)?.to_string();
        if let Some(base_idx) = self.crate_templates.iter().position(|t| t.name == name) {
            if load_type != IniLoadType::CreateOverrides {
                return Err(IniError::InvalidData);
            }
            let final_ov = self.crate_templates[base_idx].final_override_mut();
            let mut new_ov = final_ov.clone();
            new_ov.next_override = None;
            new_ov.mark_as_override();
            final_ov.next_override = Some(Box::new(new_ov));
            let target = self.crate_templates[base_idx].final_override_mut();
            target.parse_from_ini(reader, science_store, name_keys)
        } else {
            let idx = self
                .new_crate_template(&name)
                .ok_or(IniError::InvalidData)?;
            if load_type == IniLoadType::CreateOverrides {
                self.crate_templates[idx].mark_as_override();
            }
            self.crate_templates[idx].parse_from_ini(reader, science_store, name_keys)
        }
    }

    pub fn load_from_ini(
        &mut self,
        ini_bytes: &[u8],
        load_type: IniLoadType,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), IniError> {
        let mut reader = IniReader::new("crate.ini", ini_bytes, load_type);
        reader.load_blocks(None, |block, r| {
            let _block_kw = r.next_token(SEPS_DEFAULT)?;
            if block.eq_ignore_ascii_case("CrateData") {
                self.parse_crate_template_definition(r, load_type, science_store, name_keys)
            } else {
                Err(IniError::UnknownToken)
            }
        })?;
        Ok(())
    }
}
