//! `PKT-335`: `HijackerUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::ascii;

module_data! {
    /// `HijackerUpdateModuleData`.
    HijackerUpdateModuleData = "HijackerUpdate";
    "AttachToTargetBone" => attach_to_bone: String = String::new(), ascii;
    "ParachuteName" => parachute_name: String = String::new(), ascii;
}
