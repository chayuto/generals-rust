//! `PKT-387`: `WaveGuideUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{
    angle, audio, duration_real, int, particle_system, real, velocity,
};
use zh_common::types::{Int, Real};

module_data! {
    /// `WaveGuideUpdateModuleData`.
    WaveGuideUpdateModuleData = "WaveGuideUpdate";
    "WaveDelay" => wave_delay: Real = 0.0, duration_real;
    "YSize" => y_size: Real = 0.0, real;
    "LinearWaveSpacing" => linear_wave_spacing: Real = 0.0, real;
    "WaveBendMagnitude" => wave_bend_magnitude: Real = 0.0, real;
    "WaterVelocity" => water_velocity: Real = 0.0, velocity;
    "PreferredHeight" => preferred_height: Real = 0.0, real;
    "ShorelineEffectDistance" => shoreline_effect_distance: Real = 0.0, real;
    "DamageRadius" => damage_radius: Real = 0.0, real;
    "DamageAmount" => damage_amount: Real = 0.0, real;
    "ToppleForce" => topple_force: Real = 0.0, real;
    "RandomSplashSound" => random_splash_sound: String = String::new(), audio;
    "RandomSplashSoundFrequency" => random_splash_sound_frequency: Int = 0, int;
    "BridgeParticle" => bridge_particle: Option<String> = None, particle_system;
    "BridgeParticleAngleFudge" => bridge_particle_angle_fudge: Real = 0.0, angle;
    "LoopingSound" => looping_sound: String = String::new(), audio;
}
