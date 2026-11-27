//! `PKT-313`: `CleanupHazardUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration, real, weapon_slot};
use zh_common::types::{Int, Real, UnsignedInt};

module_data! {
    /// `CleanupHazardUpdateModuleData`.
    CleanupHazardUpdateModuleData = "CleanupHazardUpdate";
    "WeaponSlot" => weapon_slot_index: Int = 0, weapon_slot;
    "ScanRate" => scan_frames: UnsignedInt = 0, duration;
    "ScanRange" => scan_range: Real = 0.0, real;
}
