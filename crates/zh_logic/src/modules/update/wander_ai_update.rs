//! `PKT-386`: `WanderAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::update::ai_update::AiUpdateModuleData;

module_data! {
    /// `AIUpdateModuleData` as `WanderAIUpdate` takes it.
    WanderAIUpdateModuleData = "WanderAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
}
