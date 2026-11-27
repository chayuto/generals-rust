//! `PKT-318`: `DemoTrapUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{boolean, duration, kind_of, real, weapon, weapon_slot};
use crate::rules::kind_of::KindOfMaskType;
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE};

module_data! {
    /// `DemoTrapUpdateModuleData`.
    DemoTrapUpdateModuleData = "DemoTrapUpdate";
    "DefaultProximityMode" => defaults_to_proximity_mode: Bool = FALSE, boolean;
    "DetonationWeaponSlot" => detonation_weapon_slot: Int = 0, weapon_slot;
    "ProximityModeWeaponSlot" => proximity_mode_weapon_slot: Int = 0, weapon_slot;
    "ManualModeWeaponSlot" => manual_mode_weapon_slot: Int = 0, weapon_slot;
    "TriggerDetonationRange" => trigger_detonation_range: Real = 0.0, real;
    "IgnoreTargetTypes" => ignore_kind_of: KindOfMaskType = KindOfMaskType::new(), kind_of;
    "ScanRate" => scan_frames: UnsignedInt = 0, duration;
    "AutoDetonationWithFriendsInvolved" => friendly_detonation: Bool = FALSE, boolean;
    "DetonationWeapon" => detonation_weapon: Option<String> = None, weapon;
    "DetonateWhenKilled" => detonate_when_killed: Bool = FALSE, boolean;
}
