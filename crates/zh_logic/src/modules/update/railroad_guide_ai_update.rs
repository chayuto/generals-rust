//! `PKT-362`: `RailroadBehavior` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{
    ascii, audio, boolean, duration, real, string_vec_append,
};
use crate::modules::update::physics_update::PhysicsBehaviorModuleData;
use zh_common::types::{Bool, Real, UnsignedInt, FALSE};

module_data! {
    /// `RailroadBehaviorModuleData`.
    RailroadBehaviorModuleData = "RailroadBehavior";
    part physics: PhysicsBehaviorModuleData = PhysicsBehaviorModuleData::new;
    "PathPrefixName" => path_prefix_name: String = String::new(), ascii;
    "CrashFXTemplateName" => crash_fx_template_name: String = String::new(), ascii;
    "IsLocomotive" => is_locomotive: Bool = FALSE, boolean;
    "CarriageTemplateName" => carriage_template_names: Vec<String> = Vec::new(),
        string_vec_append;
    "BigMetalBounceSound" => big_metal_impact_sound: String = String::new(), audio;
    "SmallMetalBounceSound" => small_metal_impact_sound: String = String::new(), audio;
    "MeatyBounceSound" => meaty_impact_sound: String = String::new(), audio;
    "RunningGarrisonSpeedMax" => running_garrison_speed_max: Real = 1.0, real;
    "KillSpeedMin" => kill_speed_min: Real = 1.0, real;
    "SpeedMax" => speed_max: Real = 4.0, real;
    "Acceleration" => acceleration: Real = 1.01, real;
    "Braking" => braking: Real = 0.99, real;
    "WaitAtStationTime" => wait_at_station_time: UnsignedInt = 150, duration;
    "RunningSound" => running_sound: String = String::new(), audio;
    "ClicketyClackSound" => clickety_clack_sound: String = String::new(), audio;
    "WhistleSound" => whistle_sound: String = String::new(), audio;
    "Friction" => friction: Real = 0.97, real;
}
