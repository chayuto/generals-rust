//! `PKT-141`: Damage audio-visual effect tables (`Common/DamageFX.h`, `Common/DamageFX.cpp`,
//! `Common/INI/INIDamageFX.cpp`).

use crate::object::thing_template::LEVEL_COUNT;
use crate::rules::combat_types::{DamageType, DAMAGE_NUM_TYPES};
use std::collections::BTreeMap;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniReader, DAMAGE_TYPE_NAMES, SEPS_DEFAULT, VETERANCY_NAMES};
use zh_common::types::{NameKeyType, Real, UnsignedInt, VeterancyLevel};

/// Per-`(DamageType, VeterancyLevel)` damage FX thresholds, FXList handles, and throttle times
/// (`DamageFX::DFX` in `Common/DamageFX.h`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DamageFxSlot {
    /// If `damage_amount >= amount_for_major_fx`, use `major_damage_fx_list`.
    pub amount_for_major_fx: Real,
    /// Interned `FXList` key for major damage effects (`None` for null / `"None"`).
    pub major_damage_fx_list: Option<NameKeyType>,
    /// Interned `FXList` key for minor damage effects (`None` for null / `"None"`).
    pub minor_damage_fx_list: Option<NameKeyType>,
    /// Throttle duration in logic frames (`INI::parseDurationUnsignedInt`).
    pub damage_fx_throttle_time: UnsignedInt,
}

impl Default for DamageFxSlot {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl DamageFxSlot {
    /// Constructs a cleared `DamageFxSlot` matching C++ `DFX::clear()`.
    #[inline]
    pub const fn new() -> Self {
        Self {
            amount_for_major_fx: 0.0_f32,
            major_damage_fx_list: None,
            minor_damage_fx_list: None,
            damage_fx_throttle_time: 0,
        }
    }

    /// Resets all slot fields to defaults (`DFX::clear()`).
    #[inline]
    pub fn clear(&mut self) {
        self.amount_for_major_fx = 0.0_f32;
        self.major_damage_fx_list = None;
        self.minor_damage_fx_list = None;
        self.damage_fx_throttle_time = 0;
    }
}

/// Shared immutable audio/visual reaction matrix indexed by `[DamageType][VeterancyLevel]`
/// (`DamageFX` in `Common/DamageFX.h/.cpp`).
#[derive(Debug, Clone, PartialEq)]
pub struct DamageFX {
    entries: [[DamageFxSlot; LEVEL_COUNT]; DAMAGE_NUM_TYPES],
}

impl Default for DamageFX {
    fn default() -> Self {
        Self::new()
    }
}

impl DamageFX {
    /// Creates a cleared `DamageFX` table (`0.0f`, `None`, `None`, `0` across all 38 x 4 slots).
    pub fn new() -> Self {
        Self {
            entries: [[DamageFxSlot::new(); LEVEL_COUNT]; DAMAGE_NUM_TYPES],
        }
    }

    /// Resets every `(DamageType, VeterancyLevel)` entry (`DamageFX::clear()`).
    pub fn clear(&mut self) {
        for row in &mut self.entries {
            for slot in row {
                slot.clear();
            }
        }
    }

    /// Returns a reference to the slot at `(damage_type, veterancy)`.
    #[inline]
    pub fn slot(&self, damage_type: DamageType, veterancy: VeterancyLevel) -> &DamageFxSlot {
        &self.entries[damage_type.as_i32() as usize][veterancy as usize]
    }

    /// Returns a mutable reference to the slot at `(damage_type, veterancy)`.
    #[inline]
    pub fn slot_mut(
        &mut self,
        damage_type: DamageType,
        veterancy: VeterancyLevel,
    ) -> &mut DamageFxSlot {
        &mut self.entries[damage_type.as_i32() as usize][veterancy as usize]
    }

    /// Returns the throttle time in frames for `(t, source_veterancy)`
    /// (`DamageFX::getDamageFXThrottleTime`). `None` source defaults to `VeterancyLevel::Regular`.
    #[inline]
    pub fn get_damage_fx_throttle_time(
        &self,
        t: DamageType,
        source_veterancy: Option<VeterancyLevel>,
    ) -> UnsignedInt {
        let vet = source_veterancy.unwrap_or(VeterancyLevel::Regular);
        self.entries[t.as_i32() as usize][vet as usize].damage_fx_throttle_time
    }

    /// Resolves the `FXList` key to play for `(t, damage_amount, source_veterancy)`
    /// (`DamageFX::getDamageFXList`).
    ///
    /// Returns `None` when `damage_amount == 0.0` (by design for faux zero-damage weapons).
    #[inline]
    pub fn get_damage_fx_list(
        &self,
        t: DamageType,
        damage_amount: Real,
        source_veterancy: Option<VeterancyLevel>,
    ) -> Option<NameKeyType> {
        if damage_amount == 0.0_f32 {
            return None;
        }
        let vet = source_veterancy.unwrap_or(VeterancyLevel::Regular);
        let dfx = &self.entries[t.as_i32() as usize][vet as usize];
        if damage_amount >= dfx.amount_for_major_fx {
            dfx.major_damage_fx_list
        } else {
            dfx.minor_damage_fx_list
        }
    }

    /// Executes the damage FX lookup and invokes `on_fx` if a non-null `FXList` applies
    /// (`DamageFX::doDamageFX`).
    #[inline]
    pub fn do_damage_fx<F>(
        &self,
        t: DamageType,
        damage_amount: Real,
        source_veterancy: Option<VeterancyLevel>,
        mut on_fx: F,
    ) where
        F: FnMut(NameKeyType),
    {
        if let Some(fx) = self.get_damage_fx_list(t, damage_amount, source_veterancy) {
            on_fx(fx);
        }
    }

    /// Parses one `DamageFX` INI field line into `self` (`DamageFX::getFieldParse()`).
    pub fn parse_field<F>(
        &mut self,
        ini: &mut IniReader<'_>,
        field: &str,
        mut resolve_fx: F,
    ) -> Result<(), IniError>
    where
        F: FnMut(&str) -> Option<NameKeyType>,
    {
        let (has_vet, kind) = if field.eq_ignore_ascii_case("AmountForMajorFX") {
            (false, DfxFieldKind::Amount)
        } else if field.eq_ignore_ascii_case("MajorFX") {
            (false, DfxFieldKind::MajorFx)
        } else if field.eq_ignore_ascii_case("MinorFX") {
            (false, DfxFieldKind::MinorFx)
        } else if field.eq_ignore_ascii_case("ThrottleTime") {
            (false, DfxFieldKind::ThrottleTime)
        } else if field.eq_ignore_ascii_case("VeterancyAmountForMajorFX") {
            (true, DfxFieldKind::Amount)
        } else if field.eq_ignore_ascii_case("VeterancyMajorFX") {
            (true, DfxFieldKind::MajorFx)
        } else if field.eq_ignore_ascii_case("VeterancyMinorFX") {
            (true, DfxFieldKind::MinorFx)
        } else if field.eq_ignore_ascii_case("VeterancyThrottleTime") {
            (true, DfxFieldKind::ThrottleTime)
        } else {
            return Err(IniError::UnknownToken);
        };

        let (vet_first, vet_last) = if has_vet {
            let vet_tok = ini.next_token(SEPS_DEFAULT)?;
            let idx = IniReader::scan_index_list(vet_tok, &VETERANCY_NAMES)? as usize;
            (idx, idx)
        } else {
            (0, LEVEL_COUNT - 1)
        };

        let dmg_tok = ini.next_token(SEPS_DEFAULT)?;
        let (dmg_first, dmg_last) = if dmg_tok.eq_ignore_ascii_case("Default") {
            (0, DAMAGE_NUM_TYPES - 1)
        } else {
            let idx = IniReader::scan_index_list(dmg_tok, &DAMAGE_TYPE_NAMES)? as usize;
            (idx, idx)
        };

        match kind {
            DfxFieldKind::Amount => {
                let amt_tok = ini.next_token(SEPS_DEFAULT)?;
                let amt = IniReader::scan_real(amt_tok)?;
                for dt in dmg_first..=dmg_last {
                    for v in vet_first..=vet_last {
                        self.entries[dt][v].amount_for_major_fx = amt;
                    }
                }
            }
            DfxFieldKind::MajorFx => {
                let fx = parse_fx_list_token(ini, &mut resolve_fx)?;
                for dt in dmg_first..=dmg_last {
                    for v in vet_first..=vet_last {
                        self.entries[dt][v].major_damage_fx_list = fx;
                    }
                }
            }
            DfxFieldKind::MinorFx => {
                let fx = parse_fx_list_token(ini, &mut resolve_fx)?;
                for dt in dmg_first..=dmg_last {
                    for v in vet_first..=vet_last {
                        self.entries[dt][v].minor_damage_fx_list = fx;
                    }
                }
            }
            DfxFieldKind::ThrottleTime => {
                let mut t: UnsignedInt = 0;
                ini.parse_duration_unsigned_int(&mut t)?;
                for dt in dmg_first..=dmg_last {
                    for v in vet_first..=vet_last {
                        self.entries[dt][v].damage_fx_throttle_time = t;
                    }
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum DfxFieldKind {
    Amount,
    MajorFx,
    MinorFx,
    ThrottleTime,
}

fn parse_fx_list_token<F>(
    ini: &mut IniReader<'_>,
    resolve_fx: &mut F,
) -> Result<Option<NameKeyType>, IniError>
where
    F: FnMut(&str) -> Option<NameKeyType>,
{
    let tok = ini.next_token(SEPS_DEFAULT)?;
    if tok.eq_ignore_ascii_case("None") {
        Ok(None)
    } else {
        Ok(resolve_fx(tok))
    }
}

/// Deterministic store mapping interned `DamageFX` names to [`DamageFX`] definitions
/// (`DamageFXStore` in `Common/DamageFX.h/.cpp`).
#[derive(Debug, Clone, Default)]
pub struct DamageFXStore {
    entries: BTreeMap<NameKeyType, DamageFX>,
}

impl DamageFXStore {
    /// Creates an empty `DamageFXStore`.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }

    /// Subsystem lifecycle `init` (no-op matching C++).
    pub fn init(&mut self) {}

    /// Subsystem lifecycle `reset` (no-op matching C++; call `clear` to wipe entries).
    pub fn reset(&mut self) {}

    /// Subsystem lifecycle `update` (no-op matching C++).
    pub fn update(&mut self) {}

    /// Removes all stored `DamageFX` entries.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Finds a `DamageFX` entry by interned `NameKeyType`.
    #[inline]
    pub fn find_damage_fx(&self, namekey: NameKeyType) -> Option<&DamageFX> {
        self.entries.get(&namekey)
    }

    /// Finds a `DamageFX` entry by string name via `name_keys`.
    #[inline]
    pub fn find_damage_fx_by_name(
        &self,
        name_keys: &mut NameKeyGenerator,
        name: &str,
    ) -> Option<&DamageFX> {
        let key = name_keys.name_to_key(name);
        self.find_damage_fx(key)
    }

    /// Parses a single `DamageFX <Name> ... End` block (`DamageFXStore::parseDamageFXDefinition` /
    /// `INI::parseDamageFXDefinition`).
    pub fn parse_damage_fx_definition<F>(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
        mut resolve_fx: F,
    ) -> Result<NameKeyType, IniError>
    where
        F: FnMut(&str) -> Option<NameKeyType>,
    {
        let name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let key = name_keys.name_to_key(&name);
        let dfx = self.entries.entry(key).or_default();
        dfx.clear();
        ini.read_block_fields(|field, reader| dfx.parse_field(reader, field, &mut resolve_fx))?;
        Ok(key)
    }

    /// Loads all `DamageFX` blocks from an `IniReader` stream.
    pub fn load_from_ini<F>(
        &mut self,
        ini: &mut IniReader<'_>,
        name_keys: &mut NameKeyGenerator,
        mut resolve_fx: F,
    ) -> Result<UnsignedInt, IniError>
    where
        F: FnMut(&str) -> Option<NameKeyType>,
    {
        ini.load_blocks(None, |block_name, reader| {
            if block_name.eq_ignore_ascii_case("DamageFX") {
                let _block_kw = reader.next_token(SEPS_DEFAULT)?;
                self.parse_damage_fx_definition(reader, name_keys, &mut resolve_fx)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        })
    }
}
