//! `PKT-389`: `WorkerAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{audio, duration, duration_real, int, percent, real};
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::types::{Int, Real, UnsignedInt};

module_data! {
    /// `WorkerAIUpdateModuleData`.
    WorkerAIUpdateModuleData = "WorkerAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "MaxBoxes" => max_boxes: Int = 0, int;
    "RepairHealthPercentPerSecond" =>
        repair_health_percent_per_second: Real = 0.0, percent;
    "BoredTime" => bored_time: Real = 0.0, duration_real;
    "BoredRange" => bored_range: Real = 0.0, real;
    "SupplyCenterActionDelay" => center_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseActionDelay" => warehouse_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseScanDistance" => warehouse_scan_distance: Real = 100.0, real;
    "SuppliesDepletedVoice" => supplies_depleted_voice: String = String::new(), audio;
    "UpgradedSupplyBoost" => upgraded_supply_boost: Int = 0, int;
}
