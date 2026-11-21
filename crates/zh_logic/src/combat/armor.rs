//! `PKT-138`: Damage multiplier lookup tables and armor store (`Armor.h` / `Armor.cpp`).

use crate::rules::combat_types::{DamageType, DAMAGE_NUM_TYPES};
use std::collections::BTreeMap;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{FieldParseEntry, FieldUserData, IniError, IniReader, DAMAGE_TYPE_NAMES, SEPS_DEFAULT};
use zh_common::types::{NameKeyType, Real, UnsignedInt};

/// Per-damage-type multipliers read from an `Armor <Name>` INI block.
#[derive(Debug, Clone, PartialEq)]
pub struct ArmorTemplate {
    damage_coefficient: [Real; DAMAGE_NUM_TYPES],
}

impl Default for ArmorTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl ArmorTemplate {
    /// Creates an armor template with all damage multipliers initialized to `1.0` (`100%`).
    pub fn new() -> Self {
        Self {
            damage_coefficient: [1.0_f32; DAMAGE_NUM_TYPES],
        }
    }

    /// Resets all damage multipliers back to `1.0` (`100%`).
    pub fn clear(&mut self) {
        self.damage_coefficient = [1.0_f32; DAMAGE_NUM_TYPES];
    }

    /// Returns the raw coefficient configured for `damage_type`.
    #[inline]
    pub fn coefficient(&self, damage_type: DamageType) -> Real {
        self.damage_coefficient[damage_type.as_i32() as usize]
    }

    /// Sets the coefficient for a single `damage_type`.
    #[inline]
    pub fn set_coefficient(&mut self, damage_type: DamageType, coeff: Real) {
        self.damage_coefficient[damage_type.as_i32() as usize] = coeff;
    }

    /// Sets every damage coefficient to `coeff` (corresponding to `Armor = Default <pct>`).
    pub fn set_default_coefficient(&mut self, coeff: Real) {
        self.damage_coefficient = [coeff; DAMAGE_NUM_TYPES];
    }

    /// Adjusts `damage` for `damage_type`, preserving `Unresistable` and `SubdualUnresistable`
    /// verbatim and clamping negative resistable results to `+0.0` (matching C++ `damage < 0.0f`).
    pub fn adjust_damage(&self, damage_type: DamageType, mut damage: Real) -> Real {
        if damage_type == DamageType::Unresistable
            || damage_type == DamageType::SubdualUnresistable
        {
            return damage;
        }

        damage *= self.damage_coefficient[damage_type.as_i32() as usize];
        if damage < 0.0_f32 {
            damage = 0.0_f32;
        }
        damage
    }

    /// Parses one `Armor = <DamageType|Default> <percent>` INI field entry.
    pub fn parse_armor_coefficients(
        ini: &mut IniReader<'_>,
        target: &mut ArmorTemplate,
        _user_data: &FieldUserData,
    ) -> Result<(), IniError> {
        let damage_name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let pct_token = ini.next_token(SEPS_DEFAULT)?;
        let pct = IniReader::scan_percent_to_real(pct_token)?;

        if damage_name.eq_ignore_ascii_case("Default") {
            target.set_default_coefficient(pct);
            return Ok(());
        }

        let idx = IniReader::scan_index_list(&damage_name, &DAMAGE_TYPE_NAMES)?;
        target.damage_coefficient[idx as usize] = pct;
        Ok(())
    }
}

/// Lightweight handle referencing an `ArmorTemplate` (or unarmored pass-through when `None`).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Armor<'a> {
    template: Option<&'a ArmorTemplate>,
}

impl<'a> Armor<'a> {
    /// Constructs an `Armor` wrapping `template` (`None` passes all damage unchanged).
    #[inline]
    pub const fn new(template: Option<&'a ArmorTemplate>) -> Self {
        Self { template }
    }

    /// Constructs an unarmored handle that leaves damage untouched.
    #[inline]
    pub const fn none() -> Self {
        Self { template: None }
    }

    /// Adjusts `damage` using the referenced template, or returns `damage` untouched if `None`.
    #[inline]
    pub fn adjust_damage(&self, damage_type: DamageType, damage: Real) -> Real {
        match self.template {
            Some(tmpl) => tmpl.adjust_damage(damage_type, damage),
            None => damage,
        }
    }

    /// Clears the template reference to `None`.
    #[inline]
    pub fn clear(&mut self) {
        self.template = None;
    }

    /// Returns the underlying template reference, if any.
    #[inline]
    pub const fn template(&self) -> Option<&'a ArmorTemplate> {
        self.template
    }
}

static ARMOR_FIELD_PARSE: &[FieldParseEntry<ArmorTemplate>] = &[FieldParseEntry {
    token: Some("Armor"),
    parse: ArmorTemplate::parse_armor_coefficients,
    user_data: FieldUserData::None,
}];

/// Deterministic store mapping interned armor template keys to `ArmorTemplate` definitions.
#[derive(Debug, Clone, Default)]
pub struct ArmorStore {
    armor_templates: BTreeMap<NameKeyType, ArmorTemplate>,
}

impl ArmorStore {
    /// Creates an empty `ArmorStore`.
    pub fn new() -> Self {
        Self {
            armor_templates: BTreeMap::new(),
        }
    }

    /// Subsystem lifecycle `init` (no-op matching C++).
    pub fn init(&mut self) {}

    /// Subsystem lifecycle `reset` (no-op matching C++; call `clear` to wipe templates).
    pub fn reset(&mut self) {}

    /// Subsystem lifecycle `update` (no-op matching C++).
    pub fn update(&mut self) {}

    /// Removes all stored armor templates.
    pub fn clear(&mut self) {
        self.armor_templates.clear();
    }

    /// Looks up a template by its interned `NameKeyType`.
    pub fn find_armor_template_by_key(&self, key: NameKeyType) -> Option<&ArmorTemplate> {
        self.armor_templates.get(&key)
    }

    /// Looks up a template by name via `name_keys`.
    pub fn find_armor_template(
        &self,
        name_keys: &mut NameKeyGenerator,
        name: &str,
    ) -> Option<&ArmorTemplate> {
        let key = name_keys.name_to_key(name);
        self.find_armor_template_by_key(key)
    }

    /// Creates an `Armor` handle from an optional template reference.
    #[inline]
    pub const fn make_armor<'a>(&self, tmpl: Option<&'a ArmorTemplate>) -> Armor<'a> {
        Armor::new(tmpl)
    }

    /// Parses one `Armor <Name> ... End` INI block and inserts/overwrites it in `self`.
    pub fn parse_armor_definition(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<NameKeyType, IniError> {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let key = name_keys.name_to_key(&name);
        let tmpl = self.armor_templates.entry(key).or_default();
        tmpl.clear();
        ini.init_from_ini(tmpl, ARMOR_FIELD_PARSE)?;
        Ok(key)
    }

    /// Loads `Armor` blocks from an `IniReader` stream matching `INI::load` dispatch.
    pub fn load_from_ini(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<UnsignedInt, IniError> {
        ini.load_blocks(None, |block_name, reader| {
            if block_name.eq_ignore_ascii_case("Armor") {
                let _block_kw = reader.next_token(SEPS_DEFAULT)?;
                self.parse_armor_definition(reader, name_keys)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}
