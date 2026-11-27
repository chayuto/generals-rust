//! `PKT-332`: `HackInternetAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration, real, uint};
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::types::{Real, UnsignedInt};

module_data! {
    /// `HackInternetAIUpdateModuleData`.
    HackInternetAIUpdateModuleData = "HackInternetAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "UnpackTime" => unpack_time: UnsignedInt = 0, duration;
    "PackTime" => pack_time: UnsignedInt = 0, duration;
    "PackUnpackVariationFactor" => pack_unpack_variation_factor: Real = 0.0, real;
    "CashUpdateDelay" => cash_update_delay: UnsignedInt = 0, duration;
    "CashUpdateDelayFast" => cash_update_delay_fast: UnsignedInt = 0, duration;
    "RegularCashAmount" => regular_cash_amount: UnsignedInt = 0, uint;
    "VeteranCashAmount" => veteran_cash_amount: UnsignedInt = 0, uint;
    "EliteCashAmount" => elite_cash_amount: UnsignedInt = 0, uint;
    "HeroicCashAmount" => heroic_cash_amount: UnsignedInt = 0, uint;
    "XpPerCashUpdate" => xp_per_cash_update: UnsignedInt = 0, uint;
}
