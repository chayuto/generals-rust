//! `PKT-139`, `PKT-140` (enums/masks), `PKT-144` (surface types), `PKT-146`–`PKT-149`.

#![allow(unused_variables, dead_code)]

use zh_common::ini::{DAMAGE_TYPE_NAMES as INI_DAMAGE_NAMES, DEATH_TYPE_NAMES as INI_DEATH_NAMES};
use zh_common::types::{Bool, Int, UnsignedInt, FALSE};
use zh_common::xfer::bit_flags::{ArmorSetFlags as CommonArmorSetFlags, ARMOR_SET_BIT_NAMES};
use zh_common::xfer::bit_flags::BitFlags;

pub const ARMORSET_COUNT: usize = 8;
pub type ArmorSetFlags = CommonArmorSetFlags;
pub const ARMOR_SET_NAMES: [&str; ARMORSET_COUNT] = ARMOR_SET_BIT_NAMES;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ArmorSetType {
    #[default]
    Veteran = 0,
    Count = ARMORSET_COUNT as i32,
}

impl ArmorSetType {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        todo!("PKT-139: implement ArmorSetType::from_i32")
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
    NumTypes = DAMAGE_NUM_TYPES as i32,
}

impl DamageType {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        todo!("PKT-140: implement DamageType::from_i32")
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

pub fn is_subdual_damage(dt: Int) -> bool {
    let _ = dt;
    todo!("PKT-140: implement IsSubdualDamage")
}

pub fn is_health_damaging_damage(dt: Int) -> bool {
    let _ = dt;
    todo!("PKT-140: implement IsHealthDamagingDamage")
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
    NumTypes = DEATH_NUM_TYPES as i32,
}

impl DeathType {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        todo!("PKT-140: implement DeathType::from_i32")
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
    [""; LOCOMOTOR_SURFACE_COUNT];

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

pub type WeaponBonusConditionFlags = UnsignedInt;

pub const WEAPONSET_COUNT: usize = 17;
pub type WeaponSetFlags = BitFlags<WEAPONSET_COUNT>;
pub const WEAPON_SET_NAMES: [&str; WEAPONSET_COUNT] = [""; WEAPONSET_COUNT];

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponSetType {
    #[default]
    Veteran = 0,
    Count = WEAPONSET_COUNT as i32,
}

impl WeaponSetType {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        todo!("PKT-148: implement WeaponSetType::from_i32")
    }
}

pub const WEAPON_STATUS_COUNT: usize = 5;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponStatus {
    #[default]
    ReadyToFire = 0,
    Count = WEAPON_STATUS_COUNT as i32,
}

impl WeaponStatus {
    pub fn from_i32(v: Int) -> Option<Self> {
        let _ = v;
        let _ = FALSE;
        todo!("PKT-149: implement WeaponStatus::from_i32")
    }
}
