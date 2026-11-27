//! `PKT-284`: `OverlordContain` module data. The behaviour is not ported.

use crate::modules::contain::transport_contain::TransportContainModuleData;
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{boolean, string_vec_append};
use zh_common::types::{Bool, TRUE};

module_data! {
    /// `OverlordContainModuleData`.
    OverlordContainModuleData = "OverlordContain";
    part transport: TransportContainModuleData = TransportContainModuleData::new;
    "PayloadTemplateName" => payload_template_names: Vec<String> = Vec::new(),
        string_vec_append;
    "ExperienceSinkForRider" => experience_sink_for_rider: Bool = TRUE, boolean;
}
