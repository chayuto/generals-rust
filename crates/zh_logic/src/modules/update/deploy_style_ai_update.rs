//! `PKT-319`: `DeployStyleAIUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{boolean, duration};
use crate::modules::update::ai_update::AiUpdateModuleData;
use zh_common::types::{Bool, UnsignedInt, FALSE};

module_data! {
    /// `DeployStyleAIUpdateModuleData`.
    DeployStyleAIUpdateModuleData = "DeployStyleAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "UnpackTime" => unpack_time: UnsignedInt = 0, duration;
    "PackTime" => pack_time: UnsignedInt = 0, duration;
    "ResetTurretBeforePacking" => reset_turret_before_packing: Bool = FALSE, boolean;
    "TurretsFunctionOnlyWhenDeployed" =>
        turrets_function_only_when_deployed: Bool = FALSE, boolean;
    "TurretsMustCenterBeforePacking" =>
        turrets_must_center_before_packing: Bool = FALSE, boolean;
    "ManualDeployAnimations" => manual_deploy_animations: Bool = FALSE, boolean;
}
