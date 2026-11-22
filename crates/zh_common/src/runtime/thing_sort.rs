//! `PKT-078`: Editor sorting buckets (`GeneralsMD/Code/GameEngine/Include/Common/ThingSort.h`).

use crate::types::Int;

pub use crate::ini::IniError;
use crate::ini::IniReader;

/// Editor sorting bucket categories (`EditorSortingType` in `ThingSort.h`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EditorSortingType {
    #[default]
    None = 0,
    Structure = 1,
    Infantry = 2,
    Vehicle = 3,
    Shrubbery = 4,
    MiscManMade = 5,
    MiscNatural = 6,
    Debris = 7,
    System = 8,
    Audio = 9,
    Test = 10,
    ForReview = 11,
    Road = 12,
    Waypoint = 13,
    NumSortingTypes = 14,
}

impl EditorSortingType {
    pub const FIRST: Int = 0;
    pub const COUNT: usize = 14;

    #[inline]
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::None),
            1 => Some(Self::Structure),
            2 => Some(Self::Infantry),
            3 => Some(Self::Vehicle),
            4 => Some(Self::Shrubbery),
            5 => Some(Self::MiscManMade),
            6 => Some(Self::MiscNatural),
            7 => Some(Self::Debris),
            8 => Some(Self::System),
            9 => Some(Self::Audio),
            10 => Some(Self::Test),
            11 => Some(Self::ForReview),
            12 => Some(Self::Road),
            13 => Some(Self::Waypoint),
            _ => None,
        }
    }

    #[inline]
    pub const fn as_i32(self) -> Int {
        self as Int
    }

    /// Returns the canonical INI / editor string name for valid sorting buckets.
    #[inline]
    pub const fn as_name(self) -> Option<&'static str> {
        let idx = self.as_i32();
        if idx >= 0 && (idx as usize) < Self::COUNT {
            Some(EDITOR_SORTING_NAMES[idx as usize])
        } else {
            None
        }
    }
}

/// Canonical C++ `EditorSortingNames` table (`ThingSort.h:96`).
pub const EDITOR_SORTING_NAMES: [&str; EditorSortingType::COUNT] = [
    "NONE",
    "STRUCTURE",
    "INFANTRY",
    "VEHICLE",
    "SHRUBBERY",
    "MISC_MAN_MADE",
    "MISC_NATURAL",
    "DEBRIS",
    "SYSTEM",
    "AUDIO",
    "TEST",
    "FOR_REVIEW",
    "ROAD",
    "WAYPOINT",
];

/// Parses an `EditorSortingType` token using case-insensitive INI lookup.
pub fn parse_editor_sorting_type(token: &str) -> Result<EditorSortingType, IniError> {
    let idx = IniReader::scan_index_list(token, &EDITOR_SORTING_NAMES)?;
    EditorSortingType::from_i32(idx).ok_or(IniError::InvalidData)
}
