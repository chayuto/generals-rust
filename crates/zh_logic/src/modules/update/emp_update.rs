//! `PKT-324`: `EMPUpdate` and `LeafletDropBehavior` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{boolean, duration, kind_of, particle_system, real, rgb};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfMaskType;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::RGBColor;
use zh_common::types::{Bool, Real, UnsignedInt, FALSE};

/// `parseBitString32` with `TheWeaponAffectsMaskNames`.
fn weapon_affects(
    v: &mut UnsignedInt,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    const NAMES: [&str; 7] = [
        "SELF",
        "ALLIES",
        "ENEMIES",
        "NEUTRALS",
        "SUICIDE",
        "NOT_SIMILAR",
        "NOT_AIRBORNE",
    ];
    r.parse_bit_string32(v, &NAMES)
}

module_data! {
    /// `EMPUpdateModuleData`.
    EMPUpdateModuleData = "EMPUpdate";
    "Lifetime" => life_frames: UnsignedInt = 1, duration;
    "StartFadeTime" => start_fade_frame: UnsignedInt = 0, duration;
    "StartScale" => start_scale: Real = 1.0, real;
    "DisabledDuration" => disabled_duration: UnsignedInt = 0, duration;
    "TargetScaleMax" => target_scale_max: Real = 1.0, real;
    "TargetScaleMin" => target_scale_min: Real = 1.0, real;
    "StartColor" => start_color: RGBColor = RGBColor { red: 1.0, green: 1.0, blue: 1.0 }, rgb;
    "EndColor" => end_color: RGBColor = RGBColor { red: 0.0, green: 0.0, blue: 0.0 }, rgb;
    "DisableFXParticleSystem" => disable_fx_particle_system: Option<String> = None,
        particle_system;
    "SparksPerCubicFoot" => sparks_per_cubic_foot: Real = 0.001, real;
    "EffectRadius" => effect_radius: Real = 200.0, real;
    "DoesNotAffect" => reject_mask: UnsignedInt = 0, weapon_affects;
    "DoesNotAffectMyOwnBuildings" => does_not_affect_my_own_buildings: Bool = FALSE, boolean;
    "VictimRequiredKindOf" => victim_kind_of: KindOfMaskType = KindOfMaskType::new(), kind_of;
    "VictimForbiddenKindOf" => victim_kind_of_not: KindOfMaskType = KindOfMaskType::new(),
        kind_of;
}

module_data! {
    /// `LeafletDropBehaviorModuleData`.
    LeafletDropBehaviorModuleData = "LeafletDropBehavior";
    "Delay" => delay_frames: UnsignedInt = 1, duration;
    "DisabledDuration" => disabled_duration: UnsignedInt = 0, duration;
    "AffectRadius" => radius: Real = 60.0, real;
    "LeafletFXParticleSystem" => leaflet_fx_particle_system: Option<String> = None,
        particle_system;
}
