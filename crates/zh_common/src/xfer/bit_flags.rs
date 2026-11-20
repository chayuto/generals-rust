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
        todo!("PKT-004: Implement BitFlags::new_set_all from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn from_unsigned_int(value: UnsignedInt) -> Self {
        todo!("PKT-004: Implement BitFlags::from_unsigned_int from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn from_single_index(idx1: Int) -> Self {
        todo!("PKT-004: Implement BitFlags::from_single_index from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn from_indices(idxs: &[Int]) -> Self {
        todo!("PKT-004: Implement BitFlags::from_indices from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn set(&mut self, i: Int, val: Int) {
        todo!("PKT-004: Implement BitFlags::set from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn test(&self, i: Int) -> Bool {
        todo!("PKT-004: Implement BitFlags::test from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn test_for_any(&self, that: &Self) -> Bool {
        todo!("PKT-004: Implement BitFlags::test_for_any from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn test_for_all(&self, that: &Self) -> Bool {
        todo!("PKT-004: Implement BitFlags::test_for_all from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn test_for_none(&self, that: &Self) -> Bool {
        todo!("PKT-004: Implement BitFlags::test_for_none from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn size(&self) -> Int {
        todo!("PKT-004: Implement BitFlags::size from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn count(&self) -> Int {
        todo!("PKT-004: Implement BitFlags::count from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn any(&self) -> Bool {
        todo!("PKT-004: Implement BitFlags::any from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn flip(&mut self) {
        todo!("PKT-004: Implement BitFlags::flip from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn clear(&mut self) {
        todo!("PKT-004: Implement BitFlags::clear from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn count_intersection(&self, that: &Self) -> Int {
        todo!("PKT-004: Implement BitFlags::count_intersection from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn count_inverse_intersection(&self, that: &Self) -> Int {
        todo!("PKT-004: Implement BitFlags::count_inverse_intersection from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn any_intersection_with(&self, that: &Self) -> Bool {
        todo!("PKT-004: Implement BitFlags::any_intersection_with from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn clear_flags(&mut self, that: &Self) {
        todo!("PKT-004: Implement BitFlags::clear_flags from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn set_flags(&mut self, that: &Self) {
        todo!("PKT-004: Implement BitFlags::set_flags from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn clear_and_set(&mut self, flags_to_clear: &Self, flags_to_set: &Self) {
        todo!("PKT-004: Implement BitFlags::clear_and_set from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn test_set_and_clear(&self, must_be_set: &Self, must_be_clear: &Self) -> Bool {
        todo!("PKT-004: Implement BitFlags::test_set_and_clear from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn to_unsigned_int(&self) -> UnsignedInt {
        todo!("PKT-004: Implement BitFlags::to_unsigned_int from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn to_hex_string(&self) -> String {
        todo!("PKT-004: Implement BitFlags::to_hex_string from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn get_name_from_single_bit<'a>(names: &'a [&'a str], i: Int) -> Option<&'a str> {
        todo!("PKT-004: Implement BitFlags::get_name_from_single_bit from packets/phase2/PKT-004_BitFlags.md")
    }

    pub fn get_single_bit_from_name(names: &[&str], token: &str) -> Int {
        todo!("PKT-004: Implement BitFlags::get_single_bit_from_name from packets/phase2/PKT-004_BitFlags.md")
    }
}
