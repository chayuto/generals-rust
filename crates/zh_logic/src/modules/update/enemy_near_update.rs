//! `PKT-325`: `EnemyNearUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::duration;
use zh_common::types::UnsignedInt;

module_data! {
    /// `EnemyNearUpdateModuleData`.
    EnemyNearUpdateModuleData = "EnemyNearUpdate";
    "ScanDelayTime" => enemy_scan_delay_time: UnsignedInt = 30, duration;
}
