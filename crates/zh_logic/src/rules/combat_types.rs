//! `PKT-139`, `PKT-140` (enums/masks), `PKT-144` (surface types), `PKT-146`–`PKT-149`.

#![allow(unused_variables, dead_code)]

use zh_common::ini::{
    IniError, IniReader, DAMAGE_TYPE_NAMES as INI_DAMAGE_NAMES, DEATH_TYPE_NAMES as INI_DEATH_NAMES,
    SEPS_DEFAULT,
};
use zh_common::types::{Bool, Int, UnsignedInt};
use zh_common::xfer::bit_flags::BitFlags;
use zh_common::xfer::bit_flags::{ArmorSetFlags as CommonArmorSetFlags, ARMOR_SET_BIT_NAMES};

pub const ARMORSET_COUNT: usize = 8;
pub type ArmorSetFlags = CommonArmorSetFlags;
pub const ARMOR_SET_NAMES: [&str; ARMORSET_COUNT] = ARMOR_SET_BIT_NAMES;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArmorSetType {
    #[default]
    Veteran = 0,
    Elite = 1,
    Hero = 2,
    PlayerUpgrade = 3,
    WeakVersusBaseDefenses = 4,
    SecondLife = 5,
    CrateUpgradeOne = 6,
    CrateUpgradeTwo = 7,
    Count = ARMORSET_COUNT as i32,
}

impl ArmorSetType {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Veteran),
            1 => Some(Self::Elite),
            2 => Some(Self::Hero),
            3 => Some(Self::PlayerUpgrade),
            4 => Some(Self::WeakVersusBaseDefenses),
            5 => Some(Self::SecondLife),
            6 => Some(Self::CrateUpgradeOne),
            7 => Some(Self::CrateUpgradeTwo),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const DAMAGE_NUM_TYPES: usize = 38;
pub type DamageTypeFlags = BitFlags<DAMAGE_NUM_TYPES>;
pub const DAMAGE_TYPE_NAMES: [&str; DAMAGE_NUM_TYPES] = INI_DAMAGE_NAMES;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DamageType {
    #[default]
    Explosion = 0,
    Crush = 1,
    ArmorPiercing = 2,
    SmallArms = 3,
    Gattling = 4,
    Radiation = 5,
    Flame = 6,
    Laser = 7,
    Sniper = 8,
    Poison = 9,
    Healing = 10,
    Unresistable = 11,
    Water = 12,
    Deploy = 13,
    Surrender = 14,
    Hack = 15,
    KillPilot = 16,
    Penalty = 17,
    Falling = 18,
    Melee = 19,
    Disarm = 20,
    HazardCleanup = 21,
    ParticleBeam = 22,
    Toppling = 23,
    InfantryMissile = 24,
    AuroraBomb = 25,
    LandMine = 26,
    JetMissiles = 27,
    StealthJetMissiles = 28,
    MolotovCocktail = 29,
    ComancheVulcan = 30,
    SubdualMissile = 31,
    SubdualVehicle = 32,
    SubdualBuilding = 33,
    SubdualUnresistable = 34,
    Microwave = 35,
    KillGarrisoned = 36,
    Status = 37,
    NumTypes = DAMAGE_NUM_TYPES as i32,
}

impl DamageType {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Explosion),
            1 => Some(Self::Crush),
            2 => Some(Self::ArmorPiercing),
            3 => Some(Self::SmallArms),
            4 => Some(Self::Gattling),
            5 => Some(Self::Radiation),
            6 => Some(Self::Flame),
            7 => Some(Self::Laser),
            8 => Some(Self::Sniper),
            9 => Some(Self::Poison),
            10 => Some(Self::Healing),
            11 => Some(Self::Unresistable),
            12 => Some(Self::Water),
            13 => Some(Self::Deploy),
            14 => Some(Self::Surrender),
            15 => Some(Self::Hack),
            16 => Some(Self::KillPilot),
            17 => Some(Self::Penalty),
            18 => Some(Self::Falling),
            19 => Some(Self::Melee),
            20 => Some(Self::Disarm),
            21 => Some(Self::HazardCleanup),
            22 => Some(Self::ParticleBeam),
            23 => Some(Self::Toppling),
            24 => Some(Self::InfantryMissile),
            25 => Some(Self::AuroraBomb),
            26 => Some(Self::LandMine),
            27 => Some(Self::JetMissiles),
            28 => Some(Self::StealthJetMissiles),
            29 => Some(Self::MolotovCocktail),
            30 => Some(Self::ComancheVulcan),
            31 => Some(Self::SubdualMissile),
            32 => Some(Self::SubdualVehicle),
            33 => Some(Self::SubdualBuilding),
            34 => Some(Self::SubdualUnresistable),
            35 => Some(Self::Microwave),
            36 => Some(Self::KillGarrisoned),
            37 => Some(Self::Status),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub fn get_damage_type_flag(flags: &DamageTypeFlags, dt: Int) -> Bool {
    flags.test(dt)
}

pub fn set_damage_type_flag(mut flags: DamageTypeFlags, dt: Int) -> DamageTypeFlags {
    flags.set(dt, 1);
    flags
}

pub fn clear_damage_type_flag(mut flags: DamageTypeFlags, dt: Int) -> DamageTypeFlags {
    flags.set(dt, 0);
    flags
}

/// Parses `DamageTypeFlags` (`INI::parseDamageTypeFlags` in `INI.cpp:2062-2090`).
pub fn parse_damage_type_flags(ini: &mut IniReader<'_>) -> Result<DamageTypeFlags, IniError> {
    let mut flags = DamageTypeFlags::new_set_all();
    let mut first = true;
    while let Some(tok) = ini.next_token_or_null(SEPS_DEFAULT) {
        if tok.eq_ignore_ascii_case("ALL") {
            flags = DamageTypeFlags::new_set_all();
        } else if tok.eq_ignore_ascii_case("NONE") {
            flags.clear();
        } else if let Some(name) = tok.strip_prefix('+') {
            let idx = IniReader::scan_index_list(name, &DAMAGE_TYPE_NAMES)?;
            flags.set(idx, 1);
        } else if let Some(name) = tok.strip_prefix('-') {
            let idx = IniReader::scan_index_list(name, &DAMAGE_TYPE_NAMES)?;
            flags.set(idx, 0);
        } else {
            return Err(IniError::UnknownToken);
        }
        first = false;
    }
    if first {
        return Err(IniError::InvalidData);
    }
    Ok(flags)
}

pub fn is_subdual_damage(dt: Int) -> bool {
    matches!(dt, 31 | 32 | 33 | 34)
}

pub fn is_health_damaging_damage(dt: Int) -> bool {
    !matches!(dt, 37 | 31 | 32 | 33 | 34 | 16 | 36)
}

pub const DEATH_NUM_TYPES: usize = 21;
pub type DeathTypeFlags = UnsignedInt;
pub const DEATH_TYPE_FLAGS_ALL: DeathTypeFlags = 0xffff_ffff;
pub const DEATH_TYPE_FLAGS_NONE: DeathTypeFlags = 0x0000_0000;
pub const DEATH_TYPE_NAMES: [&str; DEATH_NUM_TYPES] = INI_DEATH_NAMES;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DeathType {
    #[default]
    Normal = 0,
    NoneType = 1,
    Crushed = 2,
    Burned = 3,
    Exploded = 4,
    Poisoned = 5,
    Toppled = 6,
    Flooded = 7,
    Suicided = 8,
    Lasered = 9,
    Detonated = 10,
    Splatted = 11,
    PoisonedBeta = 12,
    Extra2 = 13,
    Extra3 = 14,
    Extra4 = 15,
    Extra5 = 16,
    Extra6 = 17,
    Extra7 = 18,
    Extra8 = 19,
    PoisonedGamma = 20,
    NumTypes = DEATH_NUM_TYPES as i32,
}

impl DeathType {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Normal),
            1 => Some(Self::NoneType),
            2 => Some(Self::Crushed),
            3 => Some(Self::Burned),
            4 => Some(Self::Exploded),
            5 => Some(Self::Poisoned),
            6 => Some(Self::Toppled),
            7 => Some(Self::Flooded),
            8 => Some(Self::Suicided),
            9 => Some(Self::Lasered),
            10 => Some(Self::Detonated),
            11 => Some(Self::Splatted),
            12 => Some(Self::PoisonedBeta),
            13 => Some(Self::Extra2),
            14 => Some(Self::Extra3),
            15 => Some(Self::Extra4),
            16 => Some(Self::Extra5),
            17 => Some(Self::Extra6),
            18 => Some(Self::Extra7),
            19 => Some(Self::Extra8),
            20 => Some(Self::PoisonedGamma),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub fn get_death_type_flag(flags: DeathTypeFlags, dt: Int) -> Bool {
    Bool::from((flags & (1u32 << dt)) != 0)
}

pub fn set_death_type_flag(flags: DeathTypeFlags, dt: Int) -> DeathTypeFlags {
    flags | (1u32 << dt)
}

pub fn clear_death_type_flag(flags: DeathTypeFlags, dt: Int) -> DeathTypeFlags {
    flags & !(1u32 << dt)
}

pub const LOCOMOTOR_SURFACE_COUNT: usize = 5;
pub type LocomotorSurfaceTypeMask = Int;
pub const NO_SURFACES: LocomotorSurfaceTypeMask = 0x0000;
pub const ALL_SURFACES: LocomotorSurfaceTypeMask = 0xffff;
pub const LOCOMOTOR_SURFACE_TYPE_NAMES: [&str; LOCOMOTOR_SURFACE_COUNT] =
    ["GROUND", "WATER", "CLIFF", "AIR", "RUBBLE"];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LocomotorSurfaceType {
    #[default]
    Ground = 1 << 0,
    Water = 1 << 1,
    Cliff = 1 << 2,
    Air = 1 << 3,
    Rubble = 1 << 4,
}

impl LocomotorSurfaceType {
    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub type WeaponBonusConditionFlags = UnsignedInt;

pub const WEAPONSET_COUNT: usize = 17;
pub type WeaponSetFlags = BitFlags<WEAPONSET_COUNT>;
pub const WEAPON_SET_NAMES: [&str; WEAPONSET_COUNT] = [
    "VETERAN",
    "ELITE",
    "HERO",
    "PLAYER_UPGRADE",
    "CRATEUPGRADE_ONE",
    "CRATEUPGRADE_TWO",
    "VEHICLE_HIJACK",
    "CARBOMB",
    "MINE_CLEARING_DETAIL",
    "WEAPON_RIDER1",
    "WEAPON_RIDER2",
    "WEAPON_RIDER3",
    "WEAPON_RIDER4",
    "WEAPON_RIDER5",
    "WEAPON_RIDER6",
    "WEAPON_RIDER7",
    "WEAPON_RIDER8",
];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponSetType {
    #[default]
    Veteran = 0,
    Elite = 1,
    Hero = 2,
    PlayerUpgrade = 3,
    CrateUpgradeOne = 4,
    CrateUpgradeTwo = 5,
    VehicleHijack = 6,
    Carbomb = 7,
    MineClearingDetail = 8,
    Rider1 = 9,
    Rider2 = 10,
    Rider3 = 11,
    Rider4 = 12,
    Rider5 = 13,
    Rider6 = 14,
    Rider7 = 15,
    Rider8 = 16,
    Count = WEAPONSET_COUNT as i32,
}

impl WeaponSetType {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::Veteran),
            1 => Some(Self::Elite),
            2 => Some(Self::Hero),
            3 => Some(Self::PlayerUpgrade),
            4 => Some(Self::CrateUpgradeOne),
            5 => Some(Self::CrateUpgradeTwo),
            6 => Some(Self::VehicleHijack),
            7 => Some(Self::Carbomb),
            8 => Some(Self::MineClearingDetail),
            9 => Some(Self::Rider1),
            10 => Some(Self::Rider2),
            11 => Some(Self::Rider3),
            12 => Some(Self::Rider4),
            13 => Some(Self::Rider5),
            14 => Some(Self::Rider6),
            15 => Some(Self::Rider7),
            16 => Some(Self::Rider8),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}

pub const WEAPON_STATUS_COUNT: usize = 5;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponStatus {
    #[default]
    ReadyToFire = 0,
    OutOfAmmo = 1,
    BetweenFiringShots = 2,
    ReloadingClip = 3,
    PreAttack = 4,
    Count = WEAPON_STATUS_COUNT as i32,
}

impl WeaponStatus {
    pub fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::ReadyToFire),
            1 => Some(Self::OutOfAmmo),
            2 => Some(Self::BetweenFiringShots),
            3 => Some(Self::ReloadingClip),
            4 => Some(Self::PreAttack),
            _ => None,
        }
    }

    pub const fn as_i32(self) -> Int {
        self as Int
    }
}
