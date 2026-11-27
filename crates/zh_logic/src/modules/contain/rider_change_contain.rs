//! `PKT-294`: `RiderChangeContain` module data. The behaviour is not ported.

use crate::combat::locomotor_set::LOCOMOTOR_SET_NAMES;
use crate::modules::contain::transport_contain::TransportContainModuleData;
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::duration;
use crate::modules::module_data::{push, DataPart, FieldList};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::WEAPON_SET_NAMES;
use crate::rules::model_state::MODEL_CONDITION_NAMES;
use crate::rules::object_status_types::OBJECT_STATUS_NAMES;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, Real, UnsignedInt};

pub const MAX_RIDERS: usize = 8;

/// `RiderInfo`; the enum members are indices into their name tables.
#[derive(Debug, Clone, Default)]
pub struct RiderInfo {
    pub template_name: String,
    pub weapon_set_flag: Int,
    pub model_condition_flag_type: Int,
    pub object_status_type: Int,
    pub command_set: String,
    pub locomotor_set_type: Int,
}

/// `Rider1` to `Rider8`.
#[derive(Debug, Clone, Default)]
pub struct Riders(pub [RiderInfo; MAX_RIDERS]);

fn model_condition(
    v: &mut Int,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    r.parse_index_list(v, &MODEL_CONDITION_NAMES)
}

/// `parseRiderInfo`: template, model condition, weapon set flag, status, command set,
/// locomotor set.
fn rider(info: &mut RiderInfo, r: &mut IniReader<'_>) -> Result<(), IniError> {
    info.template_name = r.next_token(SEPS_DEFAULT)?.to_owned();
    r.parse_index_list(&mut info.model_condition_flag_type, &MODEL_CONDITION_NAMES)?;
    r.parse_index_list(&mut info.weapon_set_flag, &WEAPON_SET_NAMES)?;
    r.parse_index_list(&mut info.object_status_type, &OBJECT_STATUS_NAMES)?;
    info.command_set = r.next_token(SEPS_DEFAULT)?.to_owned();
    r.parse_index_list(&mut info.locomotor_set_type, &LOCOMOTOR_SET_NAMES)
}

fn rider_index(field: &str) -> Option<usize> {
    let number = field.strip_prefix("Rider")?;
    (0..MAX_RIDERS).find(|i| number == (i + 1).to_string())
}

impl DataPart for Riders {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let index = rider_index(field)?;
        Some(rider(&mut self.0[index], reader))
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (i, info) in self.0.iter().enumerate() {
            let text = format!(
                "{}|{}|{}|{}|{}|{}",
                info.template_name,
                info.model_condition_flag_type,
                info.weapon_set_flag,
                info.object_status_type,
                info.command_set,
                info.locomotor_set_type
            );
            push(out, &format!("Rider{}", i + 1), &text);
        }
    }
}

module_data! {
    /// `RiderChangeContainModuleData`.
    RiderChangeContainModuleData = "RiderChangeContain";
    part riders: Riders = |_, _| Riders::default();
    part transport: TransportContainModuleData = TransportContainModuleData::new;
    "ScuttleDelay" => scuttle_frames: UnsignedInt = 0, duration;
    /// `MODELCONDITION_TOPPLED`.
    "ScuttleStatus" => scuttle_state: Int = 0, model_condition;
}
