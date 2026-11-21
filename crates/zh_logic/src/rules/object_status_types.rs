//! `PKT-129`: Zero Hour `ObjectStatusType` & `ObjectStatusMaskType` (`ObjectStatusTypes.h/.cpp`).

#![allow(unused_variables, dead_code)]

use zh_common::types::{Bool, Int};
use zh_common::xfer::bit_flags::BitFlags;

pub const OBJECT_STATUS_COUNT: usize = 45;

pub type ObjectStatusMaskType = BitFlags<OBJECT_STATUS_COUNT>;

/// Skeleton enum: populate all 45 Zero Hour `ObjectStatusTypes` variants (`PKT-129`).
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ObjectStatusType {
    #[default]
    None = 0,
    Destroyed = 1,
    CanAttack = 2,
    UnderConstruction = 3,
    Unselectable = 4,
    NoCollisions = 5,
    NoAttack = 6,
    AirborneTarget = 7,
    Parachuting = 8,
    Repulsor = 9,
    Hijacked = 10,
    Aflame = 11,
    Burned = 12,
    Wet = 13,
    IsFiringWeapon = 14,
    Braking = 15,
    Stealthed = 16,
    Detected = 17,
    CanStealth = 18,
    Sold = 19,
    UndergoingRepair = 20,
    Reconstructing = 21,
    Masked = 22,
    IsAttacking = 23,
    IsUsingAbility = 24,
    IsAimingWeapon = 25,
    NoAttackFromAi = 26,
    IgnoringStealth = 27,
    IsCarbomb = 28,
    DeckHeightOffset = 29,
    Rider1 = 30,
    Rider2 = 31,
    Rider3 = 32,
    Rider4 = 33,
    Rider5 = 34,
    Rider6 = 35,
    Rider7 = 36,
    Rider8 = 37,
    FaerieFire = 38,
    MissileKillingSelf = 39,
    ReassignParking = 40,
    BoobyTrapped = 41,
    Immobile = 42,
    Disguised = 43,
    Deployed = 44,
    Count = OBJECT_STATUS_COUNT as i32,
}

impl ObjectStatusType {
    pub fn from_i32(v: Int) -> Option<Self> {
        if v < 0 || v >= OBJECT_STATUS_COUNT as Int {
            return None;
        }
        Some(match v {
            0 => Self::None,
            1 => Self::Destroyed,
            2 => Self::CanAttack,
            3 => Self::UnderConstruction,
            4 => Self::Unselectable,
            5 => Self::NoCollisions,
            6 => Self::NoAttack,
            7 => Self::AirborneTarget,
            8 => Self::Parachuting,
            9 => Self::Repulsor,
            10 => Self::Hijacked,
            11 => Self::Aflame,
            12 => Self::Burned,
            13 => Self::Wet,
            14 => Self::IsFiringWeapon,
            15 => Self::Braking,
            16 => Self::Stealthed,
            17 => Self::Detected,
            18 => Self::CanStealth,
            19 => Self::Sold,
            20 => Self::UndergoingRepair,
            21 => Self::Reconstructing,
            22 => Self::Masked,
            23 => Self::IsAttacking,
            24 => Self::IsUsingAbility,
            25 => Self::IsAimingWeapon,
            26 => Self::NoAttackFromAi,
            27 => Self::IgnoringStealth,
            28 => Self::IsCarbomb,
            29 => Self::DeckHeightOffset,
            30 => Self::Rider1,
            31 => Self::Rider2,
            32 => Self::Rider3,
            33 => Self::Rider4,
            34 => Self::Rider5,
            35 => Self::Rider6,
            36 => Self::Rider7,
            37 => Self::Rider8,
            38 => Self::FaerieFire,
            39 => Self::MissileKillingSelf,
            40 => Self::ReassignParking,
            41 => Self::BoobyTrapped,
            42 => Self::Immobile,
            43 => Self::Disguised,
            44 => Self::Deployed,
            _ => unreachable!(),
        })
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

/// Skeleton table: populate exact 45 `ObjectStatusMaskType::s_bitNameList` entries.
pub const OBJECT_STATUS_NAMES: [&str; OBJECT_STATUS_COUNT] = [
    "NONE",
    "DESTROYED",
    "CAN_ATTACK",
    "UNDER_CONSTRUCTION",
    "UNSELECTABLE",
    "NO_COLLISIONS",
    "NO_ATTACK",
    "AIRBORNE_TARGET",
    "PARACHUTING",
    "REPULSOR",
    "HIJACKED",
    "AFLAME",
    "BURNED",
    "WET",
    "IS_FIRING_WEAPON",
    "IS_BRAKING",
    "STEALTHED",
    "DETECTED",
    "CAN_STEALTH",
    "SOLD",
    "UNDERGOING_REPAIR",
    "RECONSTRUCTING",
    "MASKED",
    "IS_ATTACKING",
    "USING_ABILITY",
    "IS_AIMING_WEAPON",
    "NO_ATTACK_FROM_AI",
    "IGNORING_STEALTH",
    "IS_CARBOMB",
    "DECK_HEIGHT_OFFSET",
    "STATUS_RIDER1",
    "STATUS_RIDER2",
    "STATUS_RIDER3",
    "STATUS_RIDER4",
    "STATUS_RIDER5",
    "STATUS_RIDER6",
    "STATUS_RIDER7",
    "STATUS_RIDER8",
    "FAERIE_FIRE",
    "KILLING_SELF",
    "REASSIGN_PARKING",
    "BOOBY_TRAPPED",
    "IMMOBILE",
    "DISGUISED",
    "DEPLOYED",
];

pub fn make_object_status_mask(t: ObjectStatusType) -> ObjectStatusMaskType {
    ObjectStatusMaskType::from_single_index(t as Int)
}

pub fn test_object_status_mask(mask: &ObjectStatusMaskType, t: ObjectStatusType) -> Bool {
    mask.test(t as Int)
}

pub fn object_status_mask_none() -> ObjectStatusMaskType {
    ObjectStatusMaskType::new()
}
