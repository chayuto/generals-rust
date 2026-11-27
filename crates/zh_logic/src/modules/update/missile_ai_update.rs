//! `PKT-340`: `MissileAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{
    boolean, duration, duration_int, fx_list, kind_of, real, uint_int, velocity,
};
use crate::modules::update::ai_update::AiUpdateModuleData;
use crate::rules::kind_of::KindOfMaskType;
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

module_data! {
    /// `MissileAIUpdateModuleData`.
    MissileAIUpdateModuleData = "MissileAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "TryToFollowTarget" => try_to_follow_target: Bool = TRUE, boolean;
    "FuelLifetime" => fuel_lifetime: UnsignedInt = 0, duration;
    "IgnitionDelay" => ignition_delay: UnsignedInt = 0, duration;
    "InitialVelocity" => initial_vel: Real = 0.0, velocity;
    "DistanceToTravelBeforeTurning" => initial_dist: Real = 0.0, real;
    "DistanceToTargetBeforeDiving" => dive_distance: Real = 0.0, real;
    "DistanceToTargetForLock" => lock_distance: Real = 75.0, real;
    "IgnitionFX" => ignition_fx: Option<String> = None, fx_list;
    "UseWeaponSpeed" => use_weapon_speed: Bool = FALSE, boolean;
    "DetonateOnNoFuel" => detonate_on_no_fuel: Bool = FALSE, boolean;
    "DistanceScatterWhenJammed" => distance_scatter_when_jammed: Real = 75.0, real;
    "GarrisonHitKillRequiredKindOf" =>
        garrison_hit_kill_kind_of: KindOfMaskType = KindOfMaskType::new(), kind_of;
    "GarrisonHitKillForbiddenKindOf" =>
        garrison_hit_kill_kind_of_not: KindOfMaskType = KindOfMaskType::new(), kind_of;
    "GarrisonHitKillCount" => garrison_hit_kill_count: Int = 0, uint_int;
    "GarrisonHitKillFX" => garrison_hit_kill_fx: Option<String> = None, fx_list;
    "DetonateCallsKill" => detonate_calls_kill: Bool = FALSE, boolean;
    "KillSelfDelay" => kill_self_delay: Int = 3, duration_int;
}
