//! `PKT-384`: `TransportAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::update::ai_update::AiUpdateModuleData;

module_data! {
    /// `AIUpdateModuleData` as `TransportAIUpdate` takes it.
    TransportAIUpdateModuleData = "TransportAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
}
