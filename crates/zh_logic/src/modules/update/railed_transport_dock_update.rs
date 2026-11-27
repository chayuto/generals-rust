//! `PKT-361`: `RailedTransportDockUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration, real};
use crate::modules::update::dock_update::DockUpdateModuleData;
use zh_common::types::{Real, UnsignedInt};

module_data! {
    /// `RailedTransportDockUpdateModuleData`.
    RailedTransportDockUpdateModuleData = "RailedTransportDockUpdate";
    part dock: DockUpdateModuleData = DockUpdateModuleData::new;
    "PullInsideDuration" => pull_inside_duration_frames: UnsignedInt = 0, duration;
    "PushOutsideDuration" => push_outside_duration_frames: UnsignedInt = 0, duration;
    "ToleranceDistance" => tolerance_distance: Real = 50.0, real;
}
