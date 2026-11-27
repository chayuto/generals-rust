//! `PKT-321`: `DozerAIUpdate` (AI plan slice S16): construction, `xfer` and the
//! module data. The behaviour is not ported.
//!
//! Packet doc: `packets/phase5/PKT-321_DozerAIUpdate.md`. Golden test:
//! `crates/zh_logic/tests/golden_s16_dozer_worker.rs`.
//!
//! The module is an `AIUpdateInterface` with a [`DozerBrain`], which a
//! `WorkerAIUpdate` has too.
//!
//! Missing: building, repairing and mine clearing, the orders and the class's own
//! frame update (the base update runs). So the machines stay in the states they are
//! built in, and only those states can be written.

use std::any::Any;
use std::sync::Arc;

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration_real, percent, real};
use crate::modules::update::ai_update::{AiUpdate, AiUpdateModuleData, DerivedAi};
use crate::rules::combat_types::WeaponSetType;
use crate::sim::SimApi;
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, ObjectId, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError, XferMode};

module_data! {
    /// `DozerAIUpdateModuleData`.
    DozerAIUpdateModuleData = "DozerAIUpdate";
    part ai: AiUpdateModuleData = AiUpdateModuleData::new;
    "RepairHealthPercentPerSecond" =>
        repair_health_percent_per_second: Real = 0.0, percent;
    "BoredTime" => bored_time: Real = 0.0, duration_real;
    "BoredRange" => bored_range: Real = 0.0, real;
}

/// `DOZER_NUM_TASKS`: build, repair, fortify.
pub const DOZER_NUM_TASKS: usize = 3;
/// `DOZER_NUM_DOCK_POINTS`: start, action, end.
pub const DOZER_NUM_DOCK_POINTS: usize = 3;
/// `DOZER_TASK_INVALID`.
pub const DOZER_TASK_INVALID: Int = -1;
/// `DOZER_PRIMARY_IDLE`.
const DOZER_PRIMARY_IDLE: UnsignedInt = 0;

/// `DozerTaskInfo`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DozerTaskInfo {
    pub target: ObjectId,
    pub order_frame: UnsignedInt,
}

impl DozerTaskInfo {
    /// The members as `xferUser` writes the struct.
    fn bytes(self) -> [u8; 8] {
        let mut bytes = [0; 8];
        bytes[..4].copy_from_slice(&self.target.0.to_le_bytes());
        bytes[4..].copy_from_slice(&self.order_frame.to_le_bytes());
        bytes
    }
}

/// `DozerDockPointInfo`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DozerDockPoint {
    pub valid: Bool,
    pub location: Coord3D,
}

/// The members of `DozerPrimaryIdleState`.
#[derive(Debug, Clone, Copy, Default)]
pub struct DozerIdleState {
    pub idle_too_long_timestamp: UnsignedInt,
    pub idle_player_number: Int,
    pub is_marked_as_idle: Bool,
}

/// The dozer members of `DozerAIUpdate` and `WorkerAIUpdate`.
#[derive(Debug, Clone)]
pub struct DozerBrain {
    pub tasks: [DozerTaskInfo; DOZER_NUM_TASKS],
    pub current_task: Int,
    pub previous_task: Int,
    pub previous_task_info: DozerTaskInfo,
    pub dock_points: [[DozerDockPoint; DOZER_NUM_DOCK_POINTS]; DOZER_NUM_TASKS],
    /// `DOZER_SELECT_BUILD_DOCK_LOCATION` (0) at first.
    pub build_sub_task: Int,
    pub idle: DozerIdleState,
}

impl Default for DozerBrain {
    fn default() -> Self {
        Self {
            tasks: [DozerTaskInfo::default(); DOZER_NUM_TASKS],
            current_task: DOZER_TASK_INVALID,
            previous_task: DOZER_TASK_INVALID,
            previous_task_info: DozerTaskInfo::default(),
            dock_points: [[DozerDockPoint::default(); DOZER_NUM_DOCK_POINTS]; DOZER_NUM_TASKS],
            build_sub_task: 0,
            idle: DozerIdleState::default(),
        }
    }
}

/// `xfer` of a machine resting in the state it was built in, with no goal: the
/// class's version, then `StateMachine::xfer`, with `state` writing that state.
pub(super) fn xfer_resting_machine(
    xfer: &mut dyn Xfer,
    default_state: UnsignedInt,
    state: impl FnOnce(&mut dyn Xfer) -> Result<(), XferError>,
) -> Result<(), XferError> {
    for _ in 0..2 {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
    }
    let mut sleep_till: UnsignedInt = 0;
    xfer.xfer_unsigned_int(&mut sleep_till)?;
    let (mut default_id, mut current_id) = (default_state, default_state);
    xfer.xfer_unsigned_int(&mut default_id)?;
    xfer.xfer_unsigned_int(&mut current_id)?;
    let mut all_states = FALSE;
    xfer.xfer_bool(&mut all_states)?;
    state(xfer)?;
    let mut goal_object = ObjectId::INVALID;
    xfer.xfer_object_id(&mut goal_object)?;
    xfer.xfer_coord3d(&mut Coord3D::default())?;
    let (mut locked, mut default_inited) = (FALSE, TRUE);
    xfer.xfer_bool(&mut locked)?;
    xfer.xfer_bool(&mut default_inited)
}

/// `xferUser` of an enum.
fn xfer_user_int(xfer: &mut dyn Xfer, value: Int) -> Result<(), XferError> {
    xfer.xfer_user(&mut value.to_le_bytes())
}

impl DozerBrain {
    /// The dozer half of `createMachines`: the primary machine enters its idle
    /// state, which asks for the mine clearing weapon set.
    pub fn create_machine(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.idle.idle_too_long_timestamp = sim.frame();
        self.idle.is_marked_as_idle = FALSE;
        sim.set_weapon_set_flag(ctx.owner_slot, WeaponSetType::MineClearingDetail, TRUE);
    }

    /// What `DozerAIUpdate::xfer` and `WorkerAIUpdate::xfer` write after the base
    /// class, up to the build sub task. The machine's `crc` is empty.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, version: XferVersion) -> Result<(), XferError> {
        let mut count = DOZER_NUM_TASKS as Int;
        xfer.xfer_int(&mut count)?;
        for task in &mut self.tasks {
            xfer.xfer_object_id(&mut task.target)?;
            xfer.xfer_unsigned_int(&mut task.order_frame)?;
        }
        if xfer.mode() != XferMode::Crc {
            let idle = &mut self.idle;
            xfer_resting_machine(xfer, DOZER_PRIMARY_IDLE, |x| {
                let mut state_version: XferVersion = 1;
                x.xfer_version(&mut state_version, 1)?;
                x.xfer_unsigned_int(&mut idle.idle_too_long_timestamp)?;
                x.xfer_int(&mut idle.idle_player_number)?;
                x.xfer_bool(&mut idle.is_marked_as_idle)
            })?;
        }
        xfer_user_int(xfer, self.current_task)?;
        if version >= 2 {
            xfer_user_int(xfer, self.previous_task)?;
            xfer.xfer_user(&mut self.previous_task_info.bytes())?;
        }
        let mut points = DOZER_NUM_DOCK_POINTS as Int;
        xfer.xfer_int(&mut points)?;
        for point in self.dock_points.iter_mut().flatten() {
            xfer.xfer_bool(&mut point.valid)?;
            xfer.xfer_coord3d(&mut point.location)?;
        }
        xfer_user_int(xfer, self.build_sub_task)
    }
}

/// The members `DozerAIUpdate` adds to its base.
#[derive(Debug, Clone)]
pub struct DozerAi {
    pub data: Arc<DozerAIUpdateModuleData>,
    pub brain: DozerBrain,
}

pub fn create_dozer_ai_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let data: &dyn Any = data.as_any();
    let typed = data
        .downcast_ref::<DozerAIUpdateModuleData>()
        .expect("DozerAIUpdate data type mismatch");
    let mut ai = AiUpdate::new(Arc::new(typed.ai.clone()));
    ai.derived = DerivedAi::Dozer(Box::new(DozerAi {
        data: Arc::new(typed.clone()),
        brain: DozerBrain::default(),
    }));
    Box::new(ai)
}
