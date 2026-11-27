//! `PKT-317`: `DeliverPayloadAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{ascii, coord3d, duration, int, real};
use crate::modules::module_data::{radius_decal, RadiusDecalTemplate};
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::math::Coord3D;
use zh_common::types::{Int, Real, UnsignedInt};

module_data! {
    /// `DeliverPayloadAIUpdateModuleData`.
    DeliverPayloadAIUpdateModuleData = "DeliverPayloadAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "DoorDelay" => door_delay: UnsignedInt = 0, duration;
    "PutInContainer" => put_in_container_name: String = String::new(), ascii;
    "DeliveryDistance" => max_distance_to_target: Real = 0.0, real;
    "MaxAttempts" => max_number_attempts: Int = 0, int;
    "DropDelay" => drop_delay: UnsignedInt = 0, duration;
    "DropOffset" => drop_offset: Coord3D = Coord3D::default(), coord3d;
    "DropVariance" => drop_variance: Coord3D = Coord3D::default(), coord3d;
    "DeliveryDecal" => delivery_decal: RadiusDecalTemplate = RadiusDecalTemplate::default(),
        radius_decal;
    "DeliveryDecalRadius" => delivery_decal_radius: Real = 0.0, real;
}
