//! `PKT-133`: Science DAG definitions, prerequisites, and purchase queries
//! (`GeneralsMD/Code/GameEngine/Include/Common/Science.h` &
//! `GeneralsMD/Code/GameEngine/Source/Common/RTS/Science.cpp`).
//!
//! Reference C++ packet: `WP-3.2b` (`docs/golden_vectors/WP3_2b_TechTree.json`)
//! Golden Vector test: `crates/zh_logic/tests/golden_wp3_2b_tech_tree.rs`

use std::collections::BTreeMap;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{FieldParseEntry, FieldUserData, IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, NameKeyType, UnsignedInt, FALSE, TRUE};

/// Interned science identifier (`NameKeyType::0`), matching C++ `enum ScienceType : Int`.
pub type ScienceType = Int;

/// Sentinel invalid science identifier matching `SCIENCE_INVALID = -1`.
pub const SCIENCE_INVALID: ScienceType = -1;

/// Ordered list of science identifiers (`ScienceVec`).
pub type ScienceVec = Vec<ScienceType>;

/// Callback interface replacing `const Player*` science ownership queries.
pub trait SciencePlayerAccess {
    fn has_science(&self, st: ScienceType) -> bool;
}

/// Callback interface replacing `TheGameText->fetch(label)` for translated INI labels.
pub trait ScienceTextLookup {
    fn fetch_label(&mut self, label: &str) -> String;
}

/// Data fields carried by a single override layer of `ScienceInfo`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScienceInfoData {
    pub science: ScienceType,
    pub name: String,
    pub description: String,
    pub root_sciences: ScienceVec,
    pub prereq_sciences: ScienceVec,
    pub science_purchase_point_cost: Int,
    pub grantable: Bool,
}

impl Default for ScienceInfoData {
    fn default() -> Self {
        Self {
            science: SCIENCE_INVALID,
            name: String::new(),
            description: String::new(),
            root_sciences: Vec::new(),
            prereq_sciences: Vec::new(),
            science_purchase_point_cost: 0,
            grantable: TRUE,
        }
    }
}

/// Science definition with `Overridable` chain semantics (`map.ini` overrides).
#[derive(Debug, Clone)]
struct ScienceInfo {
    base: ScienceInfoData,
    base_is_override: bool,
    overrides: Vec<ScienceInfoData>,
}

impl ScienceInfo {
    fn final_override(&self) -> &ScienceInfoData {
        self.overrides.last().unwrap_or(&self.base)
    }

    fn delete_overrides(&mut self) -> bool {
        if self.base_is_override {
            false
        } else {
            self.overrides.clear();
            true
        }
    }
}

#[derive(Debug, Clone)]
enum ScienceParseOp {
    Prereqs(Vec<String>),
    Cost(Int),
    Grantable(Bool),
    DisplayName(String),
    Description(String),
}

#[derive(Debug, Default)]
struct ScienceParseTarget {
    ops: Vec<ScienceParseOp>,
}

fn parse_science_vector_tokens(
    ini: &mut IniReader<'_>,
    target: &mut ScienceParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut tokens = Vec::new();
    while let Some(tok) = ini.next_token_or_null(SEPS_DEFAULT) {
        if tok.eq_ignore_ascii_case("None") {
            tokens.clear();
            break;
        }
        tokens.push(tok.to_owned());
    }
    target.ops.push(ScienceParseOp::Prereqs(tokens));
    Ok(())
}

fn parse_purchase_cost(
    ini: &mut IniReader<'_>,
    target: &mut ScienceParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0;
    ini.parse_int(&mut val)?;
    target.ops.push(ScienceParseOp::Cost(val));
    Ok(())
}

fn parse_grantable(
    ini: &mut IniReader<'_>,
    target: &mut ScienceParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = FALSE;
    ini.parse_bool(&mut val)?;
    target.ops.push(ScienceParseOp::Grantable(val));
    Ok(())
}

fn parse_display_name(
    ini: &mut IniReader<'_>,
    target: &mut ScienceParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    target.ops.push(ScienceParseOp::DisplayName(tok));
    Ok(())
}

fn parse_description(
    ini: &mut IniReader<'_>,
    target: &mut ScienceParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    target.ops.push(ScienceParseOp::Description(tok));
    Ok(())
}

static SCIENCE_FIELD_PARSE: &[FieldParseEntry<ScienceParseTarget>] = &[
    FieldParseEntry {
        token: Some("PrerequisiteSciences"),
        parse: parse_science_vector_tokens,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("SciencePurchasePointCost"),
        parse: parse_purchase_cost,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("IsGrantable"),
        parse: parse_grantable,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DisplayName"),
        parse: parse_display_name,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Description"),
        parse: parse_description,
        user_data: FieldUserData::None,
    },
];

/// Global store of registered `ScienceInfo` definitions (`TheScienceStore`).
#[derive(Debug, Clone, Default)]
pub struct ScienceStore {
    sciences: Vec<ScienceInfo>,
}

impl ScienceStore {
    /// Creates an empty `ScienceStore`.
    pub fn new() -> Self {
        Self {
            sciences: Vec::new(),
        }
    }

    /// Initializes `ScienceStore` (`ScienceStore::init`).
    pub fn init(&mut self) {
        self.sciences.clear();
    }

    /// Removes override-only sciences and deletes per-map overrides (`ScienceStore::reset`).
    pub fn reset(&mut self) {
        self.sciences.retain_mut(|si| si.delete_overrides());
    }

    /// Subsystem update tick (no-op matching C++).
    pub fn update(&mut self) {}

    fn find_science_info(&self, st: ScienceType) -> Option<&ScienceInfoData> {
        for si in &self.sciences {
            let final_ov = si.final_override();
            if final_ov.science == st {
                return Some(final_ov);
            }
        }
        None
    }

    fn add_root_sciences_into(&self, info: &ScienceInfoData, out: &mut ScienceVec) {
        if info.prereq_sciences.is_empty() {
            if !out.contains(&info.science) {
                out.push(info.science);
            }
        } else {
            for &prereq in &info.prereq_sciences {
                if let Some(si) = self.find_science_info(prereq) {
                    self.add_root_sciences_into(si, out);
                }
            }
        }
    }

    /// Returns `true` iff `st` refers to an existing science entry.
    pub fn is_valid_science(&self, st: ScienceType) -> bool {
        self.find_science_info(st).is_some()
    }

    /// Returns `true` iff `st` exists and `m_grantable` is set (`FALSE` -> `false`).
    pub fn is_science_grantable(&self, st: ScienceType) -> bool {
        self.find_science_info(st)
            .is_some_and(|si| si.grantable != FALSE)
    }

    /// Retrieves localized `(display_name, description)` for `st`, if present.
    pub fn get_name_and_description(&self, st: ScienceType) -> Option<(&str, &str)> {
        self.find_science_info(st)
            .map(|si| (si.name.as_str(), si.description.as_str()))
    }

    /// Checks whether `player` owns every direct prerequisite of `st`.
    pub fn player_has_prereqs_for_science(
        &self,
        player: &dyn SciencePlayerAccess,
        st: ScienceType,
    ) -> bool {
        let Some(si) = self.find_science_info(st) else {
            return false;
        };
        si.prereq_sciences
            .iter()
            .all(|&prereq| player.has_science(prereq))
    }

    /// Checks whether `player` owns every computed root science required by `st`.
    pub fn player_has_root_prereqs_for_science(
        &self,
        player: &dyn SciencePlayerAccess,
        st: ScienceType,
    ) -> bool {
        let Some(si) = self.find_science_info(st) else {
            return false;
        };
        si.root_sciences
            .iter()
            .all(|&root| player.has_science(root))
    }

    /// Returns the General's point purchase cost for `st`, or `0` if invalid/free.
    pub fn get_science_purchase_cost(&self, st: ScienceType) -> Int {
        self.find_science_info(st)
            .map_or(0, |si| si.science_purchase_point_cost)
    }

    /// Converts an internal ASCII name to a `ScienceType` without checking validity.
    pub fn get_science_from_internal_name(
        &self,
        name: &str,
        name_keys: &mut NameKeyGenerator,
    ) -> ScienceType {
        if name.is_empty() {
            SCIENCE_INVALID
        } else {
            name_keys.name_to_key(name).0
        }
    }

    /// Returns the internal ASCII name for `science`, or `""` when `SCIENCE_INVALID`.
    pub fn get_internal_name_for_science<'a>(
        &self,
        science: ScienceType,
        name_keys: &'a NameKeyGenerator,
    ) -> &'a str {
        if science == SCIENCE_INVALID {
            ""
        } else {
            name_keys.key_to_name(NameKeyType(science))
        }
    }

    /// Partitions unowned purchasable sciences into `(purchasable, potentially_purchasable)`.
    pub fn get_purchasable_sciences(
        &self,
        player: &dyn SciencePlayerAccess,
    ) -> (ScienceVec, ScienceVec) {
        let mut purchasable = Vec::new();
        let mut potentially_purchasable = Vec::new();

        for entry in &self.sciences {
            let si = entry.final_override();
            if si.science_purchase_point_cost == 0 {
                continue;
            }
            if player.has_science(si.science) {
                continue;
            }
            if self.player_has_prereqs_for_science(player, si.science) {
                purchasable.push(si.science);
            } else if self.player_has_root_prereqs_for_science(player, si.science) {
                potentially_purchasable.push(si.science);
            }
        }

        (purchasable, potentially_purchasable)
    }

    /// Looks up a valid science name for INI scanning, interning the name even on error.
    pub fn friend_lookup_science(
        &self,
        science_name: &str,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<ScienceType, IniError> {
        let nkt = name_keys.name_to_key(science_name);
        let st = nkt.0;
        if !self.is_valid_science(st) {
            return Err(IniError::InvalidData);
        }
        Ok(st)
    }

    /// Returns all currently known science names in registration order.
    pub fn friend_get_science_names(&self, name_keys: &NameKeyGenerator) -> Vec<String> {
        self.sciences
            .iter()
            .map(|entry| {
                let si = entry.final_override();
                name_keys.key_to_name(NameKeyType(si.science)).to_owned()
            })
            .collect()
    }

    /// Builds a key->name index helper for callers that map science names externally.
    pub fn science_name_map(&self, name_keys: &NameKeyGenerator) -> BTreeMap<ScienceType, String> {
        let mut out = BTreeMap::new();
        for entry in &self.sciences {
            let st = entry.final_override().science;
            out.insert(st, name_keys.key_to_name(NameKeyType(st)).to_owned());
        }
        out
    }

    /// Parses one `Science <Name> ... End` INI block (`ScienceStore::friend_parseScienceDefinition`).
    pub fn parse_science_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
        text_lookup: &mut dyn ScienceTextLookup,
    ) -> Result<ScienceType, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let nkt = name_keys.name_to_key(&name);
        let st = nkt.0;

        let existing_idx = self.sciences.iter().position(|entry| entry.base.science == st);

        if ini.get_load_type() != IniLoadType::CreateOverrides && existing_idx.is_some() {
            return Err(IniError::InvalidData);
        }

        let mut parsed = ScienceParseTarget::default();
        ini.init_from_ini(&mut parsed, SCIENCE_FIELD_PARSE)?;

        let mut data = match (ini.get_load_type(), existing_idx) {
            (IniLoadType::CreateOverrides, Some(idx)) => self.sciences[idx].final_override().clone(),
            _ => ScienceInfoData::default(),
        };

        for op in parsed.ops {
            match op {
                ScienceParseOp::Prereqs(tokens) => {
                    data.prereq_sciences.clear();
                    for tok in tokens {
                        data.prereq_sciences
                            .push(self.friend_lookup_science(&tok, name_keys)?);
                    }
                }
                ScienceParseOp::Cost(val) => data.science_purchase_point_cost = val,
                ScienceParseOp::Grantable(val) => data.grantable = val,
                ScienceParseOp::DisplayName(label) => {
                    let translated = text_lookup.fetch_label(&label);
                    if translated.is_empty() {
                        return Err(IniError::InvalidData);
                    }
                    data.name = translated;
                }
                ScienceParseOp::Description(label) => {
                    let translated = text_lookup.fetch_label(&label);
                    if translated.is_empty() {
                        return Err(IniError::InvalidData);
                    }
                    data.description = translated;
                }
            }
        }

        data.science = st;
        // Preserve C++ behavior (`Science.cpp:203,225`): `*newInfo = *info` copies existing
        // `m_rootSciences` before `addRootSciences` appends any roots not yet present.
        let mut updated_roots = data.root_sciences.clone();
        self.add_root_sciences_into(&data, &mut updated_roots);
        data.root_sciences = updated_roots;

        match (ini.get_load_type(), existing_idx) {
            (IniLoadType::CreateOverrides, Some(idx)) => {
                self.sciences[idx].overrides.push(data);
            }
            (IniLoadType::CreateOverrides, None) => {
                self.sciences.push(ScienceInfo {
                    base: data,
                    base_is_override: true,
                    overrides: Vec::new(),
                });
            }
            (_, None) => {
                self.sciences.push(ScienceInfo {
                    base: data,
                    base_is_override: false,
                    overrides: Vec::new(),
                });
            }
            (_, Some(_)) => unreachable!("duplicate already rejected above"),
        }

        Ok(st)
    }

    /// Loads all `Science` blocks from `ini`.
    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
        text_lookup: &mut dyn ScienceTextLookup,
    ) -> Result<UnsignedInt, IniError> {
        ini.load_blocks(None, |block_name, reader| {
            if block_name.eq_ignore_ascii_case("Science") {
                let _block_kw = reader.next_token(SEPS_DEFAULT)?;
                self.parse_science_definition(reader, name_keys, text_lookup)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}
