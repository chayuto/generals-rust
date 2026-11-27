//! `PKT-293`: `RebuildHoleBehavior` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{ascii, duration_real, percent};
use zh_common::types::Real;

module_data! {
    /// `RebuildHoleBehaviorModuleData`.
    RebuildHoleBehaviorModuleData = "RebuildHoleBehavior";
    "WorkerObjectName" => worker_template_name: String = String::new(), ascii;
    "WorkerRespawnDelay" => worker_respawn_delay: Real = 0.0, duration_real;
    "HoleHealthRegen%PerSecond" =>
        hole_health_regen_percent_per_second: Real = 0.1, percent;
}
