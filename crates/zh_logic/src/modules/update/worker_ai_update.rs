//! `PKT-389`: `WorkerAIUpdate` (AI plan slice S16): construction, `xfer` and the
//! module data. The behaviour is not ported.
//!
//! Packet doc: `packets/phase5/PKT-389_WorkerAIUpdate.md`. Golden test:
//! `crates/zh_logic/tests/golden_s16_dozer_worker.rs`.
//!
//! The module is an `AIUpdateInterface` with a [`DozerBrain`], a supply truck
//! machine and a machine choosing between the two.
//!
//! Missing: building, repairing and supply gathering, the orders and the class's own
//! frame update (the base update runs). So the three machines stay in the states they
//! are built in, and only those states can be written.

use std::any::Any;
use std::sync::Arc;

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{audio, duration, duration_real, int, percent, real};
use crate::modules::update::ai_update::{AiUpdate, AiUpdateModuleData, DerivedAi};
use crate::modules::update::dozer_ai_update::{xfer_resting_machine, DozerBrain};
use zh_common::types::{Bool, Int, ObjectId, Real, UnsignedInt, XferVersion, FALSE};
use zh_common::xfer::{Xfer, XferError, XferMode};

module_data! {
    /// `WorkerAIUpdateModuleData`.
    WorkerAIUpdateModuleData = "WorkerAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "MaxBoxes" => max_boxes: Int = 0, int;
    "RepairHealthPercentPerSecond" =>
        repair_health_percent_per_second: Real = 0.0, percent;
    "BoredTime" => bored_time: Real = 0.0, duration_real;
    "BoredRange" => bored_range: Real = 0.0, real;
    "SupplyCenterActionDelay" => center_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseActionDelay" => warehouse_delay: UnsignedInt = 0, duration;
    "SupplyWarehouseScanDistance" => warehouse_scan_distance: Real = 100.0, real;
    "SuppliesDepletedVoice" => supplies_depleted_voice: String = String::new(), audio;
    "UpgradedSupplyBoost" => upgraded_supply_boost: Int = 0, int;
}

/// `ST_BUSY`, the first state the supply truck machine defines.
const ST_BUSY: UnsignedInt = 1;
/// `AS_DOZER`.
const AS_DOZER: UnsignedInt = 0;

/// The members `WorkerAIUpdate` adds to its base.
#[derive(Debug, Clone)]
pub struct WorkerAi {
    pub data: Arc<WorkerAIUpdateModuleData>,
    pub brain: DozerBrain,
    /// The constructor never sets it; the module's memory starts zeroed.
    pub preferred_dock: ObjectId,
    pub number_boxes: Int,
    pub force_pending: Bool,
}

impl WorkerAi {
    /// What `WorkerAIUpdate::xfer` writes after the base class. The states of the
    /// two machines here write nothing, and the machines' `crc` is empty.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, version: XferVersion) -> Result<(), XferError> {
        let crc = xfer.mode() == XferMode::Crc;
        self.brain.xfer(xfer, version)?;
        if !crc {
            xfer_resting_machine(xfer, ST_BUSY, |_| Ok(()))?;
        }
        xfer.xfer_object_id(&mut self.preferred_dock)?;
        xfer.xfer_int(&mut self.number_boxes)?;
        xfer.xfer_bool(&mut self.force_pending)?;
        if !crc {
            xfer_resting_machine(xfer, AS_DOZER, |_| Ok(()))?;
        }
        Ok(())
    }
}

pub fn create_worker_ai_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let data: &dyn Any = data.as_any();
    let typed = data
        .downcast_ref::<WorkerAIUpdateModuleData>()
        .expect("WorkerAIUpdate data type mismatch");
    let mut ai = AiUpdate::new(Arc::new(typed.ai.clone()));
    ai.derived = DerivedAi::Worker(Box::new(WorkerAi {
        data: Arc::new(typed.clone()),
        brain: DozerBrain::default(),
        preferred_dock: ObjectId::INVALID,
        number_boxes: 0,
        force_pending: FALSE,
    }));
    Box::new(ai)
}
