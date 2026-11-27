//! `PKT-347`: `ParticleUplinkCannonUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{ascii, duration, fx_list, real, special_power, uint};
use crate::modules::module_factory::ModuleParseContext;
use zh_common::ini::{IniError, IniReader, DAMAGE_TYPE_NAMES, DEATH_TYPE_NAMES};
use zh_common::types::{Int, Real, UnsignedInt};

/// `DamageTypeFlags::parseSingleBitFromINI`.
fn damage_type_index(
    v: &mut Int,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    r.parse_index_list(v, &DAMAGE_TYPE_NAMES)
}

fn death_type_index(
    v: &mut Int,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    r.parse_index_list(v, &DEATH_TYPE_NAMES)
}

module_data! {
    /// `ParticleUplinkCannonUpdateModuleData`.
    ParticleUplinkCannonUpdateModuleData = "ParticleUplinkCannonUpdate";
    "SpecialPowerTemplate" => special_power_template: Option<String> = None, special_power;
    "BeginChargeTime" => begin_charge_frames: UnsignedInt = 0, duration;
    "RaiseAntennaTime" => raise_antenna_frames: UnsignedInt = 0, duration;
    "ReadyDelayTime" => ready_delay_frames: UnsignedInt = 0, duration;
    "WidthGrowTime" => width_grow_frames: UnsignedInt = 0, duration;
    "BeamTravelTime" => beam_travel_frames: UnsignedInt = 0, duration;
    "TotalFiringTime" => total_firing_frames: UnsignedInt = 0, duration;
    "RevealRange" => reveal_range: Real = 0.0, real;
    "OuterEffectBoneName" => outer_effect_base_bone_name: String = String::new(), ascii;
    "OuterEffectNumBones" => outer_effect_num_bones: UnsignedInt = 0, uint;
    "OuterNodesLightFlareParticleSystem" =>
        outer_nodes_light_flare: String = String::new(), ascii;
    "OuterNodesMediumFlareParticleSystem" =>
        outer_nodes_medium_flare: String = String::new(), ascii;
    "OuterNodesIntenseFlareParticleSystem" =>
        outer_nodes_intense_flare: String = String::new(), ascii;
    "ConnectorBoneName" => connector_bone_name: String = String::new(), ascii;
    "ConnectorMediumLaserName" => connector_medium_laser_name: String = String::new(), ascii;
    "ConnectorIntenseLaserName" => connector_intense_laser_name: String = String::new(), ascii;
    "ConnectorMediumFlare" => connector_medium_flare: String = String::new(), ascii;
    "ConnectorIntenseFlare" => connector_intense_flare: String = String::new(), ascii;
    "FireBoneName" => fire_bone_name: String = String::new(), ascii;
    "LaserBaseLightFlareParticleSystemName" =>
        laser_base_light_flare: String = String::new(), ascii;
    "LaserBaseMediumFlareParticleSystemName" =>
        laser_base_medium_flare: String = String::new(), ascii;
    "LaserBaseIntenseFlareParticleSystemName" =>
        laser_base_intense_flare: String = String::new(), ascii;
    "ParticleBeamLaserName" => particle_beam_laser_name: String = String::new(), ascii;
    "SwathOfDeathDistance" => swath_of_death_distance: Real = 0.0, real;
    "SwathOfDeathAmplitude" => swath_of_death_amplitude: Real = 0.0, real;
    "TotalScorchMarks" => total_scorch_marks: UnsignedInt = 0, uint;
    "ScorchMarkScalar" => scorch_mark_scalar: Real = 1.0, real;
    "BeamLaunchFX" => beam_launch_fx: Option<String> = None, fx_list;
    "DelayBetweenLaunchFX" => frames_between_launch_fx_refresh: UnsignedInt = 30, duration;
    "GroundHitFX" => ground_hit_fx: Option<String> = None, fx_list;
    "DamagePerSecond" => damage_per_second: Real = 0.0, real;
    "TotalDamagePulses" => total_damage_pulses: UnsignedInt = 0, uint;
    /// `DAMAGE_LASER`.
    "DamageType" => damage_type: Int = 7, damage_type_index;
    /// `DEATH_LASERED`.
    "DeathType" => death_type: Int = 9, death_type_index;
    "DamageRadiusScalar" => damage_radius_scalar: Real = 1.0, real;
    "PoweringUpSoundLoop" => powerup_sound_name: String = String::new(), ascii;
    "UnpackToIdleSoundLoop" => unpack_to_ready_sound_name: String = String::new(), ascii;
    "FiringToPackSoundLoop" => firing_to_idle_sound_name: String = String::new(), ascii;
    "GroundAnnihilationSoundLoop" => annihilation_sound_name: String = String::new(), ascii;
    "DamagePulseRemnantObjectName" =>
        damage_pulse_remnant_object_name: String = String::new(), ascii;
    "ManualDrivingSpeed" => manual_driving_speed: Real = 0.0, real;
    "ManualFastDrivingSpeed" => manual_fast_driving_speed: Real = 0.0, real;
    "DoubleClickToFastDriveDelay" => double_click_to_fast_drive_delay: UnsignedInt = 500,
        duration;
}
