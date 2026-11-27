//! `PKT-360`: `RailedTransportAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::ascii;
use crate::modules::update::ai_update::AiUpdateModuleData;

module_data! {
    /// `RailedTransportAIUpdateModuleData`.
    RailedTransportAIUpdateModuleData = "RailedTransportAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "PathPrefixName" => path_prefix_name: String = String::new(), ascii;
}
