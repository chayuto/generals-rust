//! `PKT-375`: `StickyBombUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{ascii, fx_list, real, weapon};
use zh_common::types::Real;

module_data! {
    /// `StickyBombUpdateModuleData`.
    StickyBombUpdateModuleData = "StickyBombUpdate";
    "AttachToTargetBone" => attach_to_bone: String = String::new(), ascii;
    "OffsetZ" => offset_z: Real = 10.0, real;
    "GeometryBasedDamageWeapon" => geometry_based_damage_weapon: Option<String> = None, weapon;
    "GeometryBasedDamageFX" => geometry_based_damage_fx: Option<String> = None, fx_list;
}
