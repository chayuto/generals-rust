//! `PKT-270`: `FlightDeckBehavior` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{ascii, duration, int, real};
use crate::modules::module_data::{parse, push, DataPart, FieldList};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, Real, UnsignedInt};

pub const MAX_RUNWAYS: usize = 2;

/// `RunwayDefinition`.
#[derive(Debug, Clone, Default)]
pub struct RunwayDefinition {
    pub spaces_bone_names: Vec<String>,
    pub taxi_bone_names: Vec<String>,
    pub creation_bone_names: Vec<String>,
    /// Start and end.
    pub takeoff_bone_names: [String; 2],
    pub landing_bone_names: [String; 2],
    pub catapult_particle_system: Option<String>,
}

/// The `Runway1...` and `Runway2...` fields.
#[derive(Debug, Clone, Default)]
pub struct Runways(pub [RunwayDefinition; MAX_RUNWAYS]);

fn runway_field(field: &str) -> Option<(usize, &str)> {
    if let Some(member) = field.strip_prefix("Runway1") {
        Some((0, member))
    } else {
        field.strip_prefix("Runway2").map(|member| (1, member))
    }
}

/// `parseRunwayStrip`: up to two names; what is missing keeps its value.
fn strip(names: &mut [String; 2], r: &mut IniReader<'_>) -> Result<(), IniError> {
    for name in names {
        match r.next_token_or_null(SEPS_DEFAULT) {
            Some(token) => *name = token.to_owned(),
            None => break,
        }
    }
    Ok(())
}

impl DataPart for Runways {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let (index, member) = runway_field(field)?;
        let runway = &mut self.0[index];
        Some(match member {
            "Spaces" => reader.parse_ascii_string_vector(&mut runway.spaces_bone_names),
            "Takeoff" => strip(&mut runway.takeoff_bone_names, reader),
            "Landing" => strip(&mut runway.landing_bone_names, reader),
            "Taxi" => reader.parse_ascii_string_vector(&mut runway.taxi_bone_names),
            "Creation" => reader.parse_ascii_string_vector(&mut runway.creation_bone_names),
            "CatapultSystem" => {
                parse::particle_system(&mut runway.catapult_particle_system, reader, ctx)
            }
            _ => return None,
        })
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (i, runway) in self.0.iter().enumerate() {
            let name = |member: &str| format!("Runway{}{member}", i + 1);
            push(out, &name("Spaces"), &runway.spaces_bone_names);
            push(out, &name("Takeoff"), &runway.takeoff_bone_names.join(","));
            push(out, &name("Landing"), &runway.landing_bone_names.join(","));
            push(out, &name("Taxi"), &runway.taxi_bone_names);
            push(out, &name("Creation"), &runway.creation_bone_names);
            push(
                out,
                &name("CatapultSystem"),
                &runway.catapult_particle_system,
            );
        }
    }
}

module_data! {
    /// `FlightDeckBehaviorModuleData`. Members the constructor leaves alone are zero
    /// from the reference's allocator.
    FlightDeckBehaviorModuleData = "FlightDeckBehavior";
    part runways: Runways = |_, _| Runways::default();
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    /// The reference stores `NumRunways` in `m_numCols`.
    "NumRunways" => num_cols: Int = 0, int;
    /// The reference stores `NumSpacesPerRunway` in `m_numRows`.
    "NumSpacesPerRunway" => num_rows: Int = 0, int;
    "ApproachHeight" => approach_height: Real = 0.0, real;
    "LandingDeckHeightOffset" => landing_deck_height_offset: Real = 0.0, real;
    "HealAmountPerSecond" => heal_amount: Real = 0.0, real;
    "ParkingCleanupPeriod" => cleanup_frames: UnsignedInt = 0, duration;
    "HumanFollowPeriod" => human_follow_frames: UnsignedInt = 0, duration;
    "PayloadTemplate" => thing_template_name: String = String::new(), ascii;
    "ReplacementDelay" => replacement_frames: UnsignedInt = 0, duration;
    "DockAnimationDelay" => dock_animation_frames: UnsignedInt = 0, duration;
    "LaunchWaveDelay" => launch_wave_frames: UnsignedInt = 0, duration;
    "LaunchRampDelay" => launch_ramp_frames: UnsignedInt = 0, duration;
    "LowerRampDelay" => lower_ramp_frames: UnsignedInt = 0, duration;
    "CatapultFireDelay" => catapult_fire_frames: UnsignedInt = 0, duration;
}
