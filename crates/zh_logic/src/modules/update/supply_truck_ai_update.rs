//! `PKT-380`: `SupplyTruckAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{audio, duration, int, real};
use crate::modules::module_data::{DataOnly, DataPart, FieldList};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, Real, UnsignedInt};

module_data! {
    /// `SupplyTruckAIUpdateModuleData`.
    SupplyTruckAIUpdateModuleData = "SupplyTruckAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "MaxBoxes" => max_boxes: Int = 0, int;
    "SupplyCenterActionDelay" => center_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseActionDelay" => warehouse_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseScanDistance" => warehouse_scan_distance: Real = 100.0, real;
    "SuppliesDepletedVoice" => supplies_depleted_voice: String = String::new(), audio;
}

/// The base of `ChinookAIUpdateModuleData`; the last part.
impl DataPart for SupplyTruckAIUpdateModuleData {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        self.parse_field(field, reader, ctx)
    }

    fn part_fields(&self, gravity: Real, out: &mut FieldList) {
        self.fields(gravity, out);
    }
}
