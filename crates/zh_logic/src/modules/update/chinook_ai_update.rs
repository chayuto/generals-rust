//! `PKT-312`: `ChinookAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{
    angular_velocity, ascii, boolean, duration, int, real, rgb, uint, velocity,
};
use crate::modules::module_data::FieldText;
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::supply_truck_ai_update::SupplyTruckAIUpdateModuleData;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::RGBColor;
use zh_common::types::{Bool, Int, Real, UnsignedInt, TRUE};

/// `m_rappelSpeed`. `None` when the data does not set it: the reference's default is
/// `|gravity| * LOGICFRAMES_PER_SECOND * 0.5`, from the gravity in force at parse time.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RappelSpeed(pub Option<Real>);

impl RappelSpeed {
    pub fn value(self, gravity: Real) -> Real {
        self.0.unwrap_or(gravity.abs() * 30.0 * 0.5)
    }
}

impl FieldText for RappelSpeed {
    fn field_text(&self) -> String {
        self.0.map(|v| v.field_text()).unwrap_or_default()
    }

    fn field_text_at(&self, gravity: Real) -> String {
        self.value(gravity).field_text()
    }
}

fn rappel(
    v: &mut RappelSpeed,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    let mut speed = 0.0;
    r.parse_velocity_real(&mut speed)?;
    *v = RappelSpeed(Some(speed));
    Ok(())
}

module_data! {
    /// `ChinookAIUpdateModuleData`.
    ChinookAIUpdateModuleData = "ChinookAIUpdate";
    part supply: SupplyTruckAIUpdateModuleData = SupplyTruckAIUpdateModuleData::new;
    "RappelSpeed" => rappel_speed: RappelSpeed = RappelSpeed(None), rappel;
    "RopeDropSpeed" => rope_drop_speed: Real = 1e10, velocity;
    "RopeName" => rope_name: String = "GenericRope".to_owned(), ascii;
    "RopeFinalHeight" => rope_final_height: Real = 0.0, real;
    "RopeWidth" => rope_width: Real = 0.5, real;
    "RopeWobbleLen" => rope_wobble_len: Real = 10.0, real;
    "RopeWobbleAmplitude" => rope_wobble_amp: Real = 1.0, real;
    "RopeWobbleRate" => rope_wobble_rate: Real = 0.1, angular_velocity;
    "RopeColor" => rope_color: RGBColor = RGBColor { red: 0.9, green: 0.8, blue: 0.7 }, rgb;
    "NumRopes" => num_ropes: UnsignedInt = 4, uint;
    "PerRopeDelayMin" => per_rope_delay_min: UnsignedInt = 0x7fff_ffff, duration;
    "PerRopeDelayMax" => per_rope_delay_max: UnsignedInt = 0x7fff_ffff, duration;
    "MinDropHeight" => min_drop_height: Real = 30.0, real;
    "WaitForRopesToDrop" => wait_for_ropes_to_drop: Bool = TRUE, boolean;
    "RotorWashParticleSystem" => rotor_wash_particle_system: String = String::new(), ascii;
    "UpgradedSupplyBoost" => upgraded_supply_boost: Int = 0, int;
}
