//! `PKT-119`: Per-player build cost/time handicap multiplier table
//! (`GeneralsMD/Code/GameEngine/Source/Common/RTS/Handicap.cpp`).

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::types::{Bool, Real, FALSE};

pub const HANDICAP_TYPE_COUNT: usize = 2;
pub const THING_TYPE_COUNT: usize = 2;

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HandicapType {
    BuildCost = 0,
    BuildTime = 1,
}

#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThingType {
    Generic = 0,
    Buildings = 1,
}

const HANDICAP_TYPE_NAMES: [&str; HANDICAP_TYPE_COUNT] = ["BUILDCOST", "BUILDTIME"];
const THING_TYPE_NAMES: [&str; THING_TYPE_COUNT] = ["GENERIC", "BUILDINGS"];

/// Per-player build handicap table matching C++ `Handicap`.
#[derive(Debug, Clone, PartialEq)]
pub struct Handicap {
    handicaps: [[Real; THING_TYPE_COUNT]; HANDICAP_TYPE_COUNT],
}

impl Default for Handicap {
    fn default() -> Self {
        Self::new()
    }
}

impl Handicap {
    pub fn new() -> Self {
        let mut out = Self {
            handicaps: [[1.0_f32; THING_TYPE_COUNT]; HANDICAP_TYPE_COUNT],
        };
        out.init();
        out
    }

    pub fn init(&mut self) {
        for row in &mut self.handicaps {
            for val in row.iter_mut() {
                *val = 1.0_f32;
            }
        }
    }

    /// Reads present `HANDICAP_<TYPE>_<THING>` real overrides from `dict` without resetting absent ones.
    pub fn read_from_dict(&mut self, dict: &Dict, name_keys: &mut NameKeyGenerator) {
        for (i, ht_name) in HANDICAP_TYPE_NAMES.iter().enumerate() {
            for (j, tt_name) in THING_TYPE_NAMES.iter().enumerate() {
                let key_name = format!("HANDICAP_{}_{}", ht_name, tt_name);
                let key = name_keys.name_to_key(&key_name);
                let mut exists: Bool = FALSE;
                let value = dict.get_real(key, Some(&mut exists));
                if exists != FALSE {
                    self.handicaps[i][j] = value;
                }
            }
        }
    }

    #[inline]
    pub const fn best_thing_type(is_structure: bool) -> ThingType {
        if is_structure {
            ThingType::Buildings
        } else {
            ThingType::Generic
        }
    }

    #[inline]
    pub fn get_handicap(&self, ht: HandicapType, is_structure: bool) -> Real {
        let tt = Self::best_thing_type(is_structure);
        self.handicaps[ht as usize][tt as usize]
    }
}
