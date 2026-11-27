//! `PKT-342`: `MobMemberSlavedUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{int, real, uint};
use zh_common::types::{Int, Real, UnsignedInt};

module_data! {
    /// `MobMemberSlavedUpdateModuleData`.
    MobMemberSlavedUpdateModuleData = "MobMemberSlavedUpdate";
    "MustCatchUpRadius" => must_catch_up_radius: Int = 50, int;
    "CatchUpCrisisBailTime" => catch_up_crisis_bail_time: UnsignedInt = 999_999, uint;
    "NoNeedToCatchUpRadius" => no_need_to_catch_up_radius: Int = 25, int;
    "Squirrelliness" => squirrelliness_ratio: Real = 0.0, real;
}
