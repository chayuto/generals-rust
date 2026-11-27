//! `PKT-371`: `SpectreGunshipUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{
    ascii, duration, particle_system, real, special_power, weapon,
};
use crate::modules::module_data::{radius_decal, RadiusDecalTemplate};
use zh_common::types::{Real, UnsignedInt};

module_data! {
    /// `SpectreGunshipUpdateModuleData`.
    SpectreGunshipUpdateModuleData = "SpectreGunshipUpdate";
    "SpecialPowerTemplate" => special_power_template: Option<String> = None, special_power;
    "GattlingTemplateName" => gattling_template_name: String = String::new(), ascii;
    "HowitzerFiringRate" => howitzer_firing_rate: UnsignedInt = 10, duration;
    "OrbitTime" => orbit_frames: UnsignedInt = 0, duration;
    "HowitzerFollowLag" => howitzer_follow_lag: UnsignedInt = 0, duration;
    "AttackAreaRadius" => attack_area_radius: Real = 200.0, real;
    "StrafingIncrement" => strafing_increment: Real = 20.0, real;
    "OrbitInsertionSlope" => orbit_insertion_slope: Real = 0.7, real;
    "RandomOffsetForHowitzer" => random_offset_for_howitzer: Real = 20.0, real;
    "TargetingReticleRadius" => targeting_reticle_radius: Real = 25.0, real;
    "GunshipOrbitRadius" => gunship_orbit_radius: Real = 250.0, real;
    "HowitzerWeaponTemplate" => howitzer_weapon: Option<String> = None, weapon;
    "GattlingStrafeFXParticleSystem" => gattling_strafe_fx: Option<String> = None,
        particle_system;
    "AttackAreaDecal" => attack_area_decal: RadiusDecalTemplate = RadiusDecalTemplate::default(),
        radius_decal;
    "TargetingReticleDecal" =>
        targeting_reticle_decal: RadiusDecalTemplate = RadiusDecalTemplate::default(),
        radius_decal;
}
