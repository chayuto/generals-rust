//! `PKT-345`: `NeutronMissileUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{angular_velocity, duration, fx_list, real};
use crate::modules::module_data::{radius_decal, RadiusDecalTemplate};
use zh_common::types::{Real, UnsignedInt};

module_data! {
    /// `NeutronMissileUpdateModuleData`.
    NeutronMissileUpdateModuleData = "NeutronMissileUpdate";
    "DistanceToTravelBeforeTurning" => initial_dist: Real = 0.0, real;
    "MaxTurnRate" => max_turn_rate: Real = 999.0, angular_velocity;
    "ForwardDamping" => forward_damping: Real = 0.0, real;
    "RelativeSpeed" => relative_speed: Real = 1.0, real;
    "TargetFromDirectlyAbove" => target_from_directly_above: Real = 0.0, real;
    "LaunchFX" => launch_fx: Option<String> = None, fx_list;
    "SpecialSpeedTime" => special_speed_time: UnsignedInt = 0, duration;
    "SpecialSpeedHeight" => special_speed_height: Real = 0.0, real;
    "SpecialAccelFactor" => special_accel_factor: Real = 1.0, real;
    "SpecialJitterDistance" => special_jitter_distance: Real = 0.0, real;
    "IgnitionFX" => ignition_fx: Option<String> = None, fx_list;
    "DeliveryDecal" => delivery_decal: RadiusDecalTemplate = RadiusDecalTemplate::default(),
        radius_decal;
    "DeliveryDecalRadius" => delivery_decal_radius: Real = 0.0, real;
}
