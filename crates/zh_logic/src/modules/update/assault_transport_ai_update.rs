//! `PKT-303`: `AssaultTransportAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::real;
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::types::Real;

module_data! {
    /// `AssaultTransportAIUpdateModuleData`.
    AssaultTransportAIUpdateModuleData = "AssaultTransportAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "MembersGetHealedAtLifeRatio" => members_get_healed_at_life_ratio: Real = 0.0, real;
    "ClearRangeRequiredToContinueAttackMove" =>
        clear_range_required_to_continue_attack_move: Real = 50.0, real;
}
