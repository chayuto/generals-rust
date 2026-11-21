//! `PKT-132`: General's rank progression store (`RankInfo.h` / `RankInfo.cpp`).
//!
//! Reference C++ packet: `WP-3.2b` (`docs/golden_vectors/WP3_2b_TechTree.json`)
//! Golden Vector test: `crates/zh_logic/tests/golden_wp3_2b_tech_tree.rs`

use crate::rules::science::{ScienceStore, ScienceTextLookup, ScienceVec};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{FieldParseEntry, FieldUserData, IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, UnsignedInt};

/// Single rank tier definition (`RankInfo` / `Overridable`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RankInfoData {
    pub rank_name: String,
    pub skill_points_needed: Int,
    pub science_purchase_points_granted: Int,
    pub sciences_granted: ScienceVec,
}

#[derive(Debug, Clone)]
struct RankInfoEntry {
    base: RankInfoData,
    overrides: Vec<RankInfoData>,
}

impl RankInfoEntry {
    fn final_override(&self) -> &RankInfoData {
        self.overrides.last().unwrap_or(&self.base)
    }

    fn delete_overrides(&mut self) {
        self.overrides.clear();
    }
}

#[derive(Debug, Clone)]
enum RankParseOp {
    RankName(String),
    SkillPointsNeeded(Int),
    SciencesGranted(Vec<String>),
    SciencePurchasePointsGranted(UnsignedInt),
}

#[derive(Debug, Default)]
struct RankParseTarget {
    ops: Vec<RankParseOp>,
}

fn parse_rank_name(
    ini: &mut IniReader<'_>,
    target: &mut RankParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_DEFAULT)?.to_owned();
    target.ops.push(RankParseOp::RankName(tok));
    Ok(())
}

fn parse_skill_points_needed(
    ini: &mut IniReader<'_>,
    target: &mut RankParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0;
    ini.parse_int(&mut val)?;
    target.ops.push(RankParseOp::SkillPointsNeeded(val));
    Ok(())
}

fn parse_sciences_granted(
    ini: &mut IniReader<'_>,
    target: &mut RankParseTarget,
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
    target.ops.push(RankParseOp::SciencesGranted(tokens));
    Ok(())
}

fn parse_science_purchase_points_granted(
    ini: &mut IniReader<'_>,
    target: &mut RankParseTarget,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut val = 0u32;
    ini.parse_unsigned_int(&mut val)?;
    target
        .ops
        .push(RankParseOp::SciencePurchasePointsGranted(val));
    Ok(())
}

static RANK_FIELD_PARSE: &[FieldParseEntry<RankParseTarget>] = &[
    FieldParseEntry {
        token: Some("RankName"),
        parse: parse_rank_name,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("SkillPointsNeeded"),
        parse: parse_skill_points_needed,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("SciencesGranted"),
        parse: parse_sciences_granted,
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("SciencePurchasePointsGranted"),
        parse: parse_science_purchase_points_granted,
        user_data: FieldUserData::None,
    },
];

/// Global store of 1-based rank progression definitions (`TheRankInfoStore`).
#[derive(Debug, Clone, Default)]
pub struct RankInfoStore {
    rank_infos: Vec<RankInfoEntry>,
}

impl RankInfoStore {
    /// Creates an empty `RankInfoStore`.
    pub fn new() -> Self {
        Self {
            rank_infos: Vec::new(),
        }
    }

    /// Initializes `RankInfoStore` (`RankInfoStore::init`).
    pub fn init(&mut self) {
        self.rank_infos.clear();
    }

    /// Deletes per-map overrides while keeping base rank tiers (`RankInfoStore::reset`).
    pub fn reset(&mut self) {
        for ri in &mut self.rank_infos {
            ri.delete_overrides();
        }
    }

    /// Subsystem update tick (no-op matching C++).
    pub fn update(&mut self) {}

    /// Returns the total number of defined rank levels.
    #[inline]
    pub fn get_rank_level_count(&self) -> Int {
        self.rank_infos.len() as Int
    }

    /// Returns rank info for 1-based `level` (`1..=get_rank_level_count()`), or `None`.
    pub fn get_rank_info(&self, level: Int) -> Option<&RankInfoData> {
        if level >= 1 && level <= self.get_rank_level_count() {
            Some(self.rank_infos[(level - 1) as usize].final_override())
        } else {
            None
        }
    }

    /// Parses one `Rank <Level> ... End` INI block (`RankInfoStore::friend_parseRankDefinition`).
    pub fn parse_rank_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
        text_lookup: &mut dyn ScienceTextLookup,
    ) -> Result<(), IniError> {
        let rank = IniReader::scan_int(ini.next_token(SEPS_DEFAULT)?)?;
        let is_override = ini.get_load_type() == IniLoadType::CreateOverrides;

        if is_override {
            if rank < 1 || rank > self.get_rank_level_count() {
                return Err(IniError::InvalidData);
            }
        } else if rank != self.get_rank_level_count() + 1 {
            return Err(IniError::InvalidData);
        }

        let mut parsed = RankParseTarget::default();
        ini.init_from_ini(&mut parsed, RANK_FIELD_PARSE)?;

        let mut data = if is_override {
            self.rank_infos[(rank - 1) as usize].final_override().clone()
        } else {
            RankInfoData::default()
        };

        for op in parsed.ops {
            match op {
                RankParseOp::RankName(label) => {
                    let translated = text_lookup.fetch_label(&label);
                    if translated.is_empty() {
                        return Err(IniError::InvalidData);
                    }
                    data.rank_name = translated;
                }
                RankParseOp::SkillPointsNeeded(pts) => data.skill_points_needed = pts,
                RankParseOp::SciencesGranted(tokens) => {
                    data.sciences_granted.clear();
                    for tok in tokens {
                        data.sciences_granted
                            .push(science_store.friend_lookup_science(&tok, name_keys)?);
                    }
                }
                RankParseOp::SciencePurchasePointsGranted(pts) => {
                    // C++ stores INI::parseUnsignedInt output directly in Int field.
                    data.science_purchase_points_granted = pts as Int;
                }
            }
        }

        if is_override {
            self.rank_infos[(rank - 1) as usize].overrides.push(data);
        } else {
            self.rank_infos.push(RankInfoEntry {
                base: data,
                overrides: Vec::new(),
            });
        }

        Ok(())
    }

    /// Loads all `Rank` blocks from `ini`.
    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        science_store: &ScienceStore,
        name_keys: &mut NameKeyGenerator,
        text_lookup: &mut dyn ScienceTextLookup,
    ) -> Result<UnsignedInt, IniError> {
        ini.load_blocks(None, |block_name, reader| {
            if block_name.eq_ignore_ascii_case("Rank") {
                let _block_kw = reader.next_token(SEPS_DEFAULT)?;
                self.parse_rank_definition(reader, science_store, name_keys, text_lookup)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}
