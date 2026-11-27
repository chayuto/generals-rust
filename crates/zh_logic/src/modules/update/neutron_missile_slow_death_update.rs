//! `PKT-344`: `NeutronMissileSlowDeathBehavior` module data. The behaviour is not ported.

use crate::modules::behavior::slow_death_behavior::SlowDeathBehaviorModuleData;
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{fx_list, real};
use crate::modules::module_data::{push, DataPart, FieldList};
use crate::modules::module_factory::ModuleParseContext;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, Real};

pub const MAX_NEUTRON_BLASTS: usize = 9;

/// `BlastInfo`.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BlastInfo {
    pub enabled: Bool,
    pub delay: Real,
    pub scorch_delay: Real,
    pub inner_radius: Real,
    pub outer_radius: Real,
    pub max_damage: Real,
    pub min_damage: Real,
    pub topple_speed: Real,
    pub push_force: Real,
}

/// `Blast1...` to `Blast9...`.
#[derive(Debug, Clone, Default)]
pub struct NeutronBlasts(pub [BlastInfo; MAX_NEUTRON_BLASTS]);

fn blast_field(field: &str) -> Option<(usize, &str)> {
    let rest = field.strip_prefix("Blast")?;
    let digit = rest.chars().next()?.to_digit(10)?;
    let index = (digit as usize).checked_sub(1)?;
    Some((index, rest.get(1..)?))
}

impl DataPart for NeutronBlasts {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let (index, member) = blast_field(field)?;
        let blast = &mut self.0[index];
        Some(match member {
            "Enabled" => reader.parse_bool(&mut blast.enabled),
            "Delay" => reader.parse_duration_real(&mut blast.delay),
            "ScorchDelay" => reader.parse_duration_real(&mut blast.scorch_delay),
            "InnerRadius" => reader.parse_real(&mut blast.inner_radius),
            "OuterRadius" => reader.parse_real(&mut blast.outer_radius),
            "MaxDamage" => reader.parse_real(&mut blast.max_damage),
            "MinDamage" => reader.parse_real(&mut blast.min_damage),
            "ToppleSpeed" => reader.parse_real(&mut blast.topple_speed),
            "PushForce" => reader.parse_real(&mut blast.push_force),
            _ => return None,
        })
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (i, blast) in self.0.iter().enumerate() {
            let name = |member: &str| format!("Blast{}{member}", i + 1);
            push(out, &name("Enabled"), &blast.enabled);
            push(out, &name("Delay"), &blast.delay);
            push(out, &name("ScorchDelay"), &blast.scorch_delay);
            push(out, &name("InnerRadius"), &blast.inner_radius);
            push(out, &name("OuterRadius"), &blast.outer_radius);
            push(out, &name("MaxDamage"), &blast.max_damage);
            push(out, &name("MinDamage"), &blast.min_damage);
            push(out, &name("ToppleSpeed"), &blast.topple_speed);
            push(out, &name("PushForce"), &blast.push_force);
        }
    }
}

module_data! {
    /// `NeutronMissileSlowDeathBehaviorModuleData`.
    NeutronMissileSlowDeathBehaviorModuleData = "NeutronMissileSlowDeathBehavior";
    part blasts: NeutronBlasts = |_, _| NeutronBlasts::default();
    part slow_death: SlowDeathBehaviorModuleData = SlowDeathBehaviorModuleData::new;
    "ScorchMarkSize" => scorch_size: Real = 0.0, real;
    "FXList" => fx_list_name: Option<String> = None, fx_list;
}
