//! `PKT-321`: `DozerAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration_real, percent, real};
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::types::Real;

module_data! {
    /// `DozerAIUpdateModuleData`.
    DozerAIUpdateModuleData = "DozerAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "RepairHealthPercentPerSecond" =>
        repair_health_percent_per_second: Real = 0.0, percent;
    "BoredTime" => bored_time: Real = 0.0, duration_real;
    "BoredRange" => bored_range: Real = 0.0, real;
}
