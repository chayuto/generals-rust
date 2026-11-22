//! `PKT-139`: Condition-keyed armor template sets (`GeneralsMD/Code/GameEngine/Include/GameLogic/ArmorSet.h`).

use std::collections::BTreeSet;

use crate::combat::armor::{ArmorStore, ArmorTemplate};
use crate::player::sparse_match_finder::SparseMatchable;
use crate::rules::combat_types::{ArmorSetFlags, ARMOR_SET_NAMES};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{FieldUserData, IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::Int;

/// Pairs an `ArmorSetFlags` condition mask with an optional `ArmorTemplate` and `DamageFX` name.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ArmorTemplateSet {
    pub types: ArmorSetFlags,
    pub template: Option<ArmorTemplate>,
    pub damage_fx_name: Option<String>,
}

struct ArmorSetParseState<'a, 'b> {
    set: &'a mut ArmorTemplateSet,
    armor_store: &'b ArmorStore,
    damage_fx_names: &'b BTreeSet<String>,
    name_keys: &'b mut NameKeyGenerator,
}

impl ArmorTemplateSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.types.clear();
        self.template = None;
        self.damage_fx_name = None;
    }

    #[inline]
    pub fn get_armor_template(&self) -> Option<&ArmorTemplate> {
        self.template.as_ref()
    }

    #[inline]
    pub fn get_damage_fx_name(&self) -> Option<&str> {
        self.damage_fx_name.as_deref()
    }

    #[inline]
    pub fn has_damage_fx(&self) -> bool {
        self.damage_fx_name.is_some()
    }

    /// Parses an `ArmorSet ... End` INI block matching C++ `ArmorTemplateSet::parseArmorTemplateSet`.
    pub fn parse_armor_template_set(
        &mut self,
        ini: &mut IniReader<'_>,
        armor_store: &ArmorStore,
        damage_fx_names: &BTreeSet<String>,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), IniError> {
        let mut state = ArmorSetParseState {
            set: self,
            armor_store,
            damage_fx_names,
            name_keys,
        };
        ini.read_block_fields(|field, reader| {
            if field.eq_ignore_ascii_case("Conditions") {
                parse_conditions(reader, &mut state, &FieldUserData::None)
            } else if field.eq_ignore_ascii_case("Armor") {
                parse_armor(reader, &mut state, &FieldUserData::None)
            } else if field.eq_ignore_ascii_case("DamageFX") {
                parse_damage_fx(reader, &mut state, &FieldUserData::None)
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}

impl SparseMatchable<ArmorSetFlags> for ArmorTemplateSet {
    #[inline]
    fn get_conditions_yes_count(&self) -> Int {
        1
    }

    #[inline]
    fn get_nth_conditions_yes(&self, _i: Int) -> &ArmorSetFlags {
        &self.types
    }
}

fn parse_conditions(
    ini: &mut IniReader<'_>,
    state: &mut ArmorSetParseState<'_, '_>,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    state.set.types.parse_from_ini(ini, &ARMOR_SET_NAMES)
}

fn parse_armor(
    ini: &mut IniReader<'_>,
    state: &mut ArmorSetParseState<'_, '_>,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let token = ini.next_token(SEPS_DEFAULT)?;
    if token.eq_ignore_ascii_case("None") {
        state.set.template = None;
    } else {
        state.set.template = state
            .armor_store
            .find_armor_template(state.name_keys, token)
            .cloned();
    }
    Ok(())
}

fn parse_damage_fx(
    ini: &mut IniReader<'_>,
    state: &mut ArmorSetParseState<'_, '_>,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let token = ini.next_token(SEPS_DEFAULT)?;
    if token.eq_ignore_ascii_case("None") {
        state.set.damage_fx_name = None;
    } else if let Some(found) = state
        .damage_fx_names
        .iter()
        .find(|name| name.eq_ignore_ascii_case(token))
    {
        state.set.damage_fx_name = Some(found.clone());
    } else {
        state.set.damage_fx_name = None;
    }
    Ok(())
}
