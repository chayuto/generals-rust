//! `PKT-326`: `FireOCLAfterWeaponCooldownUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration, ocl, uint, weapon_slot};
use crate::modules::upgrade::upgrade_module::UpgradeMuxData;
use zh_common::types::{Int, UnsignedInt};

module_data! {
    /// `FireOCLAfterWeaponCooldownUpdateModuleData`.
    FireOCLAfterWeaponCooldownUpdateModuleData = "FireOCLAfterWeaponCooldownUpdate";
    part upgrade_mux: UpgradeMuxData = |_, _| UpgradeMuxData::new();
    "WeaponSlot" => weapon_slot_index: Int = 0, weapon_slot;
    "OCL" => ocl_name: Option<String> = None, ocl;
    "MinShotsToCreateOCL" => min_shots_required: UnsignedInt = 1, uint;
    "OCLLifetimePerSecond" => ocl_lifetime_per_second: UnsignedInt = 1000, uint;
    "OCLLifetimeMaxCap" => ocl_max_frames: UnsignedInt = 1000, duration;
}
