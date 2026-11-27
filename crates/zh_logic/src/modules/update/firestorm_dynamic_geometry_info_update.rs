//! `PKT-328`: `FirestormDynamicGeometryInfoUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration_real, fx_list, real};
use crate::modules::module_data::{parse, push, DataPart, FieldList};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::dynamic_geometry_info_update::DynamicGeometryInfoUpdateModuleData;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::Real;

pub const MAX_FIRESTORM_SYSTEMS: usize = 16;

/// `ParticleSystem1` to `ParticleSystem16`.
#[derive(Debug, Clone, Default)]
pub struct FirestormSystems(pub [Option<String>; MAX_FIRESTORM_SYSTEMS]);

fn system_index(field: &str) -> Option<usize> {
    let number = field.strip_prefix("ParticleSystem")?;
    (0..MAX_FIRESTORM_SYSTEMS).find(|i| number == (i + 1).to_string())
}

impl DataPart for FirestormSystems {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let index = system_index(field)?;
        Some(parse::particle_system(&mut self.0[index], reader, ctx))
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (i, system) in self.0.iter().enumerate() {
            push(out, &format!("ParticleSystem{}", i + 1), system);
        }
    }
}

module_data! {
    /// `FirestormDynamicGeometryInfoUpdateModuleData`.
    FirestormDynamicGeometryInfoUpdateModuleData = "FirestormDynamicGeometryInfoUpdate";
    part particles: FirestormSystems = |_, _| FirestormSystems::default();
    part geometry: DynamicGeometryInfoUpdateModuleData =
        DynamicGeometryInfoUpdateModuleData::new;
    "DelayBetweenDamageFrames" => delay_between_damage_frames: Real = 0.0, duration_real;
    "DamageAmount" => damage_amount: Real = 0.0, real;
    "MaxHeightForDamage" => max_height_for_damage: Real = 20.0, real;
    "FXList" => fx_list_name: Option<String> = None, fx_list;
    "ParticleOffsetZ" => particle_offset_z: Real = 0.0, real;
    "ScorchSize" => scorch_size: Real = 0.0, real;
}
