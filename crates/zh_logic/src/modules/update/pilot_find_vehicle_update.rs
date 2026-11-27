//! `PKT-349`: `PilotFindVehicleUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration, real};
use zh_common::types::{Real, UnsignedInt};

module_data! {
    /// `PilotFindVehicleUpdateModuleData`.
    PilotFindVehicleUpdateModuleData = "PilotFindVehicleUpdate";
    "ScanRate" => scan_frames: UnsignedInt = 0, duration;
    "ScanRange" => scan_range: Real = 0.0, real;
    "MinHealth" => min_health: Real = 0.5, real;
}
