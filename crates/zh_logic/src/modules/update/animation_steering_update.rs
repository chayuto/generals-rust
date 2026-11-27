//! `PKT-302`: `AnimationSteeringUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::duration;
use zh_common::types::UnsignedInt;

module_data! {
    /// `AnimationSteeringUpdateModuleData`.
    AnimationSteeringUpdateModuleData = "AnimationSteeringUpdate";
    "MinTransitionTime" => transition_frames: UnsignedInt = 0, duration;
}
