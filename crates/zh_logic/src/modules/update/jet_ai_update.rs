//! `PKT-337`: `JetAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{
    angle, ascii, boolean, duration, locomotor_set, percent, real,
};
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

module_data! {
    /// `JetAIUpdateModuleData`.
    JetAIUpdateModuleData = "JetAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "OutOfAmmoDamagePerSecond" => out_of_ammo_damage_per_second: Real = 0.0, percent;
    "NeedsRunway" => needs_runway: Bool = TRUE, boolean;
    "KeepsParkingSpaceWhenAirborne" =>
        keeps_parking_space_when_airborne: Bool = TRUE, boolean;
    "TakeoffDistForMaxLift" => takeoff_dist_for_max_lift: Real = 0.0, percent;
    "TakeoffPause" => takeoff_pause: UnsignedInt = 0, duration;
    "MinHeight" => min_height: Real = 0.0, real;
    "ParkingOffset" => parking_offset: Real = 0.0, real;
    "SneakyOffsetWhenAttacking" => sneaky_offset_when_attacking: Real = 0.0, real;
    "AttackLocomotorType" => attacking_loco: Int = 0, locomotor_set;
    "AttackLocomotorPersistTime" => attack_loco_persist_time: UnsignedInt = 0, duration;
    "AttackersMissPersistTime" => attackers_miss_persist_time: UnsignedInt = 0, duration;
    "ReturnForAmmoLocomotorType" => returning_loco: Int = 0, locomotor_set;
    "LockonTime" => lockon_time: UnsignedInt = 0, duration;
    "LockonCursor" => lockon_cursor: String = String::new(), ascii;
    "LockonInitialDist" => lockon_initial_dist: Real = 100.0, real;
    "LockonFreq" => lockon_freq: Real = 0.5, real;
    /// The default is 720 as written, not converted to radians.
    "LockonAngleSpin" => lockon_angle_spin: Real = 720.0, angle;
    "LockonBlinky" => lockon_blinky: Bool = FALSE, boolean;
    "ReturnToBaseIdleTime" => return_to_base_idle_time: UnsignedInt = 0, duration;
}
