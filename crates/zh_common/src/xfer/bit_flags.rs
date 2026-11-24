//! `PKT-004`: Deterministic `BitFlags<NUMBITS>` translation target.
//!
//! Reference C++ packet: `packets/phase2/PKT-004_BitFlags.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt004.rs` (`docs/golden_vectors/PKT-004.json`)
//!
//! Instructions for `env:ephemeral-web` agent:
//! 1. Read `packets/phase2/PKT-004_BitFlags.md`.
//! 2. Implement all methods of `BitFlags<NUMBITS>` below plus `get_name_from_single_bit` /
//!    `get_single_bit_from_name` using `ARMOR_SET_BIT_NAMES` and `MODEL_CONDITION_BIT_NAMES`.
//! 3. Remove `#[ignore]` in `crates/zh_common/tests/golden_pkt004.rs` and verify:
//!    `python3 scripts/lint_determinism.py && cargo test --workspace`

#![allow(unused_variables)]

use crate::types::{Bool, Int, UnsignedInt};

pub const ARMOR_SET_BIT_NAMES: [&str; 8] = [
    "VETERAN",
    "ELITE",
    "HERO",
    "PLAYER_UPGRADE",
    "WEAK_VERSUS_BASEDEFENSES",
    "SECOND_LIFE",
    "CRATE_UPGRADE_ONE",
    "CRATE_UPGRADE_TWO",
];

pub const MODEL_CONDITION_BIT_NAMES: [&str; 117] = [
    "TOPPLED",
    "FRONTCRUSHED",
    "BACKCRUSHED",
    "DAMAGED",
    "REALLYDAMAGED",
    "RUBBLE",
    "SPECIAL_DAMAGED",
    "NIGHT",
    "SNOW",
    "PARACHUTING",
    "GARRISONED",
    "ENEMYNEAR",
    "WEAPONSET_VETERAN",
    "WEAPONSET_ELITE",
    "WEAPONSET_HERO",
    "WEAPONSET_CRATEUPGRADE_ONE",
    "WEAPONSET_CRATEUPGRADE_TWO",
    "WEAPONSET_PLAYER_UPGRADE",
    "DOOR_1_OPENING",
    "DOOR_1_CLOSING",
    "DOOR_1_WAITING_OPEN",
    "DOOR_1_WAITING_TO_CLOSE",
    "DOOR_2_OPENING",
    "DOOR_2_CLOSING",
    "DOOR_2_WAITING_OPEN",
    "DOOR_2_WAITING_TO_CLOSE",
    "DOOR_3_OPENING",
    "DOOR_3_CLOSING",
    "DOOR_3_WAITING_OPEN",
    "DOOR_3_WAITING_TO_CLOSE",
    "DOOR_4_OPENING",
    "DOOR_4_CLOSING",
    "DOOR_4_WAITING_OPEN",
    "DOOR_4_WAITING_TO_CLOSE",
    "ATTACKING",
    "PREATTACK_A",
    "FIRING_A",
    "BETWEEN_FIRING_SHOTS_A",
    "RELOADING_A",
    "PREATTACK_B",
    "FIRING_B",
    "BETWEEN_FIRING_SHOTS_B",
    "RELOADING_B",
    "PREATTACK_C",
    "FIRING_C",
    "BETWEEN_FIRING_SHOTS_C",
    "RELOADING_C",
    "TURRET_ROTATE",
    "POST_COLLAPSE",
    "MOVING",
    "DYING",
    "AWAITING_CONSTRUCTION",
    "PARTIALLY_CONSTRUCTED",
    "ACTIVELY_BEING_CONSTRUCTED",
    "PRONE",
    "FREEFALL",
    "ACTIVELY_CONSTRUCTING",
    "CONSTRUCTION_COMPLETE",
    "RADAR_EXTENDING",
    "RADAR_UPGRADED",
    "PANICKING",
    "AFLAME",
    "SMOLDERING",
    "BURNED",
    "DOCKING",
    "DOCKING_BEGINNING",
    "DOCKING_ACTIVE",
    "DOCKING_ENDING",
    "CARRYING",
    "FLOODED",
    "LOADED",
    "JETAFTERBURNER",
    "JETEXHAUST",
    "PACKING",
    "UNPACKING",
    "DEPLOYED",
    "OVER_WATER",
    "POWER_PLANT_UPGRADED",
    "CLIMBING",
    "SOLD",
    "RAPPELLING",
    "ARMED",
    "POWER_PLANT_UPGRADING",
    "SPECIAL_CHEERING",
    "CONTINUOUS_FIRE_SLOW",
    "CONTINUOUS_FIRE_MEAN",
    "CONTINUOUS_FIRE_FAST",
    "RAISING_FLAG",
    "CAPTURED",
    "EXPLODED_FLAILING",
    "EXPLODED_BOUNCING",
    "SPLATTED",
    "USING_WEAPON_A",
    "USING_WEAPON_B",
    "USING_WEAPON_C",
    "PREORDER",
    "CENTER_TO_LEFT",
    "LEFT_TO_CENTER",
    "CENTER_TO_RIGHT",
    "RIGHT_TO_CENTER",
    "RIDER1",
    "RIDER2",
    "RIDER3",
    "RIDER4",
    "RIDER5",
    "RIDER6",
    "RIDER7",
    "RIDER8",
    "STUNNED_FLAILING",
    "STUNNED",
    "SECOND_LIFE",
    "JAMMED",
    "ARMORSET_CRATEUPGRADE_ONE",
    "ARMORSET_CRATEUPGRADE_TWO",
    "USER_1",
    "USER_2",
    "DISGUISED",
];

pub type ArmorSetFlags = BitFlags<8>;
pub type ModelConditionFlags = BitFlags<117>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BitFlags<const NUMBITS: usize> {
    bits: [bool; NUMBITS],
}

impl<const NUMBITS: usize> Default for BitFlags<NUMBITS> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const NUMBITS: usize> BitFlags<NUMBITS> {
    pub const NUM_BITS: usize = NUMBITS;

    pub const fn new() -> Self {
        Self {
            bits: [false; NUMBITS],
        }
    }

    pub fn new_set_all() -> Self {
        Self {
            bits: [true; NUMBITS],
        }
    }

    pub fn from_unsigned_int(value: UnsignedInt) -> Self {
        let mut result = Self::new();
        let limit = NUMBITS.min(32);
        for i in 0..limit {
            result.bits[i] = ((value >> i) & 1) != 0;
        }
        result
    }

    pub fn from_single_index(idx1: Int) -> Self {
        let mut result = Self::new();
        if idx1 >= 0 {
            let i = idx1 as usize;
            if i < NUMBITS {
                result.bits[i] = true;
            }
        }
        result
    }

    pub fn from_indices(idxs: &[Int]) -> Self {
        let mut result = Self::new();
        for &idx in idxs {
            if idx >= 0 {
                let i = idx as usize;
                if i < NUMBITS {
                    result.bits[i] = true;
                }
            }
        }
        result
    }

    pub fn set(&mut self, i: Int, val: Int) {
        if i >= 0 {
            let i = i as usize;
            if i < NUMBITS {
                self.bits[i] = val != 0;
            }
        }
    }

    pub fn test(&self, i: Int) -> Bool {
        if i >= 0 {
            let i = i as usize;
            if i < NUMBITS {
                return Bool::from(self.bits[i]);
            }
        }
        0
    }

    pub fn test_for_any(&self, that: &Self) -> Bool {
        u8::from(self.bits.iter().zip(that.bits.iter()).any(|(&a, &b)| a && b))
    }

    pub fn test_for_all(&self, that: &Self) -> Bool {
        u8::from(self
            .bits
            .iter()
            .zip(that.bits.iter())
            .all(|(&a, &b)| !b || a))
    }

    pub fn test_for_none(&self, that: &Self) -> Bool {
        u8::from(!self
            .bits
            .iter()
            .zip(that.bits.iter())
            .any(|(&a, &b)| a && b))
    }

    pub fn size(&self) -> Int {
        NUMBITS as Int
    }

    pub fn count(&self) -> Int {
        self.bits.iter().filter(|&&bit| bit).count() as Int
    }

    pub fn any(&self) -> Bool {
        u8::from(self.bits.iter().any(|&bit| bit))
    }

    pub fn flip(&mut self) {
        for bit in &mut self.bits {
            *bit = !*bit;
        }
    }

    pub fn clear(&mut self) {
        self.bits.fill(false);
    }

    pub fn count_intersection(&self, that: &Self) -> Int {
        self.bits
            .iter()
            .zip(that.bits.iter())
            .filter(|(&a, &b)| a && b)
            .count() as Int
    }

    pub fn count_inverse_intersection(&self, that: &Self) -> Int {
        self.bits
            .iter()
            .zip(that.bits.iter())
            .filter(|(&a, &b)| !a && b)
            .count() as Int
    }

    pub fn any_intersection_with(&self, that: &Self) -> Bool {
        u8::from(self.bits.iter().zip(that.bits.iter()).any(|(&a, &b)| a && b))
    }

    pub fn clear_flags(&mut self, that: &Self) {
        for (bit, clear) in self.bits.iter_mut().zip(that.bits.iter()) {
            if *clear {
                *bit = false;
            }
        }
    }

    pub fn set_flags(&mut self, that: &Self) {
        for (bit, set) in self.bits.iter_mut().zip(that.bits.iter()) {
            if *set {
                *bit = true;
            }
        }
    }

    pub fn clear_and_set(&mut self, flags_to_clear: &Self, flags_to_set: &Self) {
        self.clear_flags(flags_to_clear);
        self.set_flags(flags_to_set);
    }

    pub fn test_set_and_clear(&self, must_be_set: &Self, must_be_clear: &Self) -> Bool {
        u8::from(self.test_for_none(must_be_clear) != 0 && self.test_for_all(must_be_set) != 0)
    }

    pub fn to_unsigned_int(&self) -> UnsignedInt {
        let mut value = 0u32;
        for i in 0..NUMBITS.min(32) {
            if self.bits[i] {
                value |= 1u32 << i;
            }
        }
        value
    }

    pub fn to_hex_string(&self) -> String {
        if NUMBITS == 0 {
            return "0".to_string();
        }

        let num_chunks = (NUMBITS + 63) / 64;
        let mut result = String::new();
        let mut printed_any = false;

        for chunk in (0..num_chunks).rev() {
            let mut value = 0u64;
            let base = chunk * 64;
            let limit = 64.min(NUMBITS.saturating_sub(base));
            for bit in 0..limit {
                if self.bits[base + bit] {
                    value |= 1u64 << bit;
                }
            }

            if value != 0 || chunk == 0 || printed_any {
                if printed_any {
                    result.push_str(&format!("{value:016X}"));
                } else if value == 0 {
                    result.push('0');
                } else {
                    result.push_str(&format!("{value:X}"));
                }
                printed_any = true;
            }
        }

        if !printed_any {
            "0".to_string()
        } else {
            result
        }
    }

    pub fn get_name_from_single_bit<'a>(names: &'a [&'a str], i: Int) -> Option<&'a str> {
        if i >= 0 {
            let i = i as usize;
            if i < NUMBITS && i < names.len() {
                return Some(names[i]);
            }
        }
        None
    }

    pub fn get_single_bit_from_name(names: &[&str], token: &str) -> Int {
        for (i, &name) in names.iter().enumerate() {
            if i < NUMBITS && name.eq_ignore_ascii_case(token) {
                return i as Int;
            }
        }
        -1
    }

    /// Parses bitflags from an `IniReader` token stream matching C++ `BitFlags::parse` (`BitFlagsIO.h`).
    pub fn parse_from_ini(
        &mut self,
        ini: &mut crate::ini::IniReader<'_>,
        bit_names: &[&str],
    ) -> Result<(), crate::ini::IniError> {
        let mut found_normal = false;
        let mut found_add_or_sub = false;
        while let Some(token) = ini.next_token_or_null(crate::ini::SEPS_DEFAULT) {
            if token.eq_ignore_ascii_case("NONE") {
                if found_normal || found_add_or_sub {
                    return Err(crate::ini::IniError::InvalidNameList);
                }
                self.clear();
                break;
            }
            if let Some(name) = token.strip_prefix('+') {
                if found_normal {
                    return Err(crate::ini::IniError::InvalidNameList);
                }
                let bit_index = crate::ini::IniReader::scan_index_list(name, bit_names)?;
                self.set(bit_index, 1);
                found_add_or_sub = true;
            } else if let Some(name) = token.strip_prefix('-') {
                if found_normal {
                    return Err(crate::ini::IniError::InvalidNameList);
                }
                let bit_index = crate::ini::IniReader::scan_index_list(name, bit_names)?;
                self.set(bit_index, 0);
                found_add_or_sub = true;
            } else {
                if found_add_or_sub {
                    return Err(crate::ini::IniError::InvalidNameList);
                }
                if !found_normal {
                    self.clear();
                }
                let bit_index = crate::ini::IniReader::scan_index_list(token, bit_names)?;
                self.set(bit_index, 1);
                found_normal = true;
            }
        }
        Ok(())
    }

    /// Serializes bitflags matching C++ `BitFlags::xfer` (`BitFlagsIO.h:156-232`).
    pub fn xfer_named(
        &mut self,
        xfer: &mut dyn crate::xfer::Xfer,
        bit_names: &[&str],
    ) -> Result<(), crate::xfer::XferError> {
        let mut version: crate::types::XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;

        match xfer.mode() {
            crate::xfer::XferMode::Save => {
                let mut count = self.count();
                xfer.xfer_int(&mut count)?;
                for i in 0..NUMBITS {
                    if let Some(name) = (self.test(i as Int) != 0)
                        .then(|| Self::get_name_from_single_bit(bit_names, i as Int))
                        .flatten()
                    {
                        let mut s = name.to_owned();
                        xfer.xfer_ascii_string(&mut s)?;
                    }
                }
                Ok(())
            }
            crate::xfer::XferMode::Load => {
                self.clear();
                let mut count: Int = 0;
                xfer.xfer_int(&mut count)?;
                if count < 0 {
                    return Err(crate::xfer::XferError::InvalidData);
                }
                for _ in 0..count {
                    let mut name = String::new();
                    xfer.xfer_ascii_string(&mut name)?;
                    let idx = Self::get_single_bit_from_name(bit_names, &name);
                    if idx < 0 {
                        return Err(crate::xfer::XferError::InvalidData);
                    }
                    self.set(idx, 1);
                }
                Ok(())
            }
            crate::xfer::XferMode::Crc => {
                let mut raw = self.to_unsigned_int().to_le_bytes();
                xfer.xfer_user(&mut raw)
            }
        }
    }
}

impl<const NUMBITS: usize> PartialOrd for BitFlags<NUMBITS> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<const NUMBITS: usize> Ord for BitFlags<NUMBITS> {
    /// Deterministic bit-index lexicographic ordering matching C++ `SparseMatchFinder::MapHelper`.
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        for i in 0..NUMBITS {
            let ord = self.bits[i].cmp(&other.bits[i]);
            if ord != std::cmp::Ordering::Equal {
                return ord;
            }
        }
        std::cmp::Ordering::Equal
    }
}
