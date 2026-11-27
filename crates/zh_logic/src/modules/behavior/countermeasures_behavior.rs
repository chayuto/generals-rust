//! `PKT-266`: `CountermeasuresBehavior` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{angle, ascii, boolean, duration, percent, real, uint};
use crate::modules::upgrade::upgrade_module::UpgradeMuxData;
use zh_common::types::{Bool, Real, UnsignedInt, FALSE};

module_data! {
    /// `CountermeasuresBehaviorModuleData`.
    CountermeasuresBehaviorModuleData = "CountermeasuresBehavior";
    part upgrade_mux: UpgradeMuxData = |_, _| UpgradeMuxData::new();
    "FlareTemplateName" => flare_template_name: String = String::new(), ascii;
    "FlareBoneBaseName" => flare_bone_base_name: String = String::new(), ascii;
    "VolleySize" => volley_size: UnsignedInt = 0, uint;
    "VolleyArcAngle" => volley_arc_angle: Real = 0.0, angle;
    "VolleyVelocityFactor" => volley_velocity_factor: Real = 1.0, real;
    "DelayBetweenVolleys" => frames_between_volleys: UnsignedInt = 0, duration;
    "NumberOfVolleys" => number_of_volleys: UnsignedInt = 0, uint;
    "ReloadTime" => reload_frames: UnsignedInt = 0, duration;
    "EvasionRate" => evasion_rate: Real = 0.0, percent;
    "MustReloadAtAirfield" => must_reload_at_airfield: Bool = FALSE, boolean;
    "MissileDecoyDelay" => missile_decoy_frames: UnsignedInt = 0, duration;
    /// The constructor does not set it; the reference's allocator zeroes it.
    "ReactionLaunchLatency" => countermeasure_reaction_frames: UnsignedInt = 0, duration;
}
