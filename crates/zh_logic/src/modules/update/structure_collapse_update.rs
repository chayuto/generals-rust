//! `PKT-376`: `StructureCollapseUpdate` module data. The behaviour is not ported.

use crate::modules::die::die_module::DieMuxData;
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration_int, int, real};
use crate::modules::module_data::{parse, push, DataPart, FieldList, FieldText};
use crate::modules::module_factory::ModuleParseContext;
use std::collections::BTreeSet;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Int, Real};

/// `TheStructureCollapsePhaseNames`.
pub const STRUCTURE_COLLAPSE_PHASE_NAMES: [&str; 4] = ["INITIAL", "DELAY", "BURST", "FINAL"];

/// The `OCL` and `FXList` lines, per phase; `None` is a name that resolves to nothing.
#[derive(Debug, Clone, Default)]
pub struct CollapseEffects {
    pub ocls: [Vec<Option<String>>; 4],
    pub fxs: [Vec<Option<String>>; 4],
}

/// A phase name, then one or more names appended to that phase.
fn phase_list(
    lists: &mut [Vec<Option<String>>; 4],
    known: &BTreeSet<String>,
    reader: &mut IniReader<'_>,
) -> Result<(), IniError> {
    let mut phase: Int = 0;
    reader.parse_index_list(&mut phase, &STRUCTURE_COLLAPSE_PHASE_NAMES)?;
    let list = lists.get_mut(phase as usize).ok_or(IniError::InvalidData)?;
    list.push(parse::lookup(reader.next_token(SEPS_DEFAULT)?, known));
    while let Some(token) = reader.next_token_or_null(SEPS_DEFAULT) {
        list.push(parse::lookup(token, known));
    }
    Ok(())
}

impl DataPart for CollapseEffects {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        match field {
            "OCL" => Some(phase_list(&mut self.ocls, ctx.ocl_names, reader)),
            "FXList" => Some(phase_list(&mut self.fxs, ctx.fx_list_names, reader)),
            _ => None,
        }
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (name, lists) in [("OCL", &self.ocls), ("FXList", &self.fxs)] {
            let text = lists.each_ref().map(FieldText::field_text).join(";");
            push(out, name, &text);
        }
    }
}

module_data! {
    /// `StructureCollapseUpdateModuleData`. `m_oclCount` and `m_fxCount` stay 1: no
    /// field sets them.
    StructureCollapseUpdateModuleData = "StructureCollapseUpdate";
    part effects: CollapseEffects = |_, _| CollapseEffects::default();
    part die_mux: DieMuxData = |_, _| DieMuxData::new();
    "MinCollapseDelay" => min_collapse_delay: Int = 0, duration_int;
    "MaxCollapseDelay" => max_collapse_delay: Int = 0, duration_int;
    "MinBurstDelay" => min_burst_delay: Int = 9999, duration_int;
    "MaxBurstDelay" => max_burst_delay: Int = 0, duration_int;
    "CollapseDamping" => collapse_damping: Real = 0.0, real;
    "MaxShudder" => max_shudder: Real = 0.0, real;
    "BigBurstFrequency" => big_burst_frequency: Int = 0, int;
}
