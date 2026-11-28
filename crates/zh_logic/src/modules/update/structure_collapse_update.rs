//! `PKT-376`: `StructureCollapseUpdateModuleData` and `StructureCollapseUpdate`
//! (`StructureCollapseUpdate.h`, `StructureCollapseUpdate.cpp`).

use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::damage::{DamageInfo, PLAYERMASK_ALL};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, UpdateFacet};
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::{duration_int, int, real};
use crate::modules::module_data::{parse, parse_data, push, DataPart, FieldList, FieldText};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use std::collections::BTreeSet;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `TheStructureCollapsePhaseNames`.
pub const STRUCTURE_COLLAPSE_PHASE_NAMES: [&str; 4] = ["INITIAL", "DELAY", "BURST", "FINAL"];

pub const SCPHASE_INITIAL: usize = 0;
pub const SCPHASE_DELAY: usize = 1;
pub const SCPHASE_BURST: usize = 2;
pub const SCPHASE_FINAL: usize = 3;

pub const COLLAPSESTATE_STANDING: Int = 0;
pub const COLLAPSESTATE_WAITINGFORCOLLAPSESTART: Int = 1;
pub const COLLAPSESTATE_COLLAPSING: Int = 2;
pub const COLLAPSESTATE_DONE: Int = 3;

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

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

/// Runtime instance of `StructureCollapseUpdate`.
#[derive(Debug, Clone)]
pub struct StructureCollapseUpdate {
    pub data: StructureCollapseUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub collapse_frame: UnsignedInt,
    pub burst_frame: UnsignedInt,
    pub collapse_state: Int,
    pub collapse_velocity: Real,
    pub current_height: Real,
}

impl StructureCollapseUpdate {
    pub fn new(data: StructureCollapseUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            collapse_frame: 0,
            burst_frame: 0,
            collapse_state: COLLAPSESTATE_STANDING,
            collapse_velocity: 0.0,
            current_height: 0.0,
        }
    }

    fn do_phase_stuff(
        &self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        phase: usize,
        target: &Coord3D,
    ) {
        // `m_fxCount` and `m_oclCount` are always 1, so `buildNonDupRandomIndexList` draws once.
        let fxs = &self.data.effects.fxs[phase];
        if !fxs.is_empty() {
            let idx = sim.rng_logic_int(0, (fxs.len() as Int) - 1) as usize;
            if let Some(Some(fx)) = fxs.get(idx) {
                sim.trigger_fx_list(fx, target);
            }
        }
        let ocls = &self.data.effects.ocls[phase];
        if !ocls.is_empty() {
            let idx = sim.rng_logic_int(0, (ocls.len() as Int) - 1) as usize;
            if let Some(Some(ocl)) = ocls.get(idx) {
                let angle = sim.object(owner).map_or(0.0, |o| o.thing.get_orientation());
                sim.create_ocl(
                    ocl,
                    crate::object::ocl_runtime::OclCreateArgs::position_angle(
                        owner, *target, None, angle,
                    ),
                );
            }
        }
    }

    fn begin_structure_collapse(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        let delay = sim.rng_logic_int(self.data.min_collapse_delay, self.data.max_collapse_delay);
        self.collapse_frame = now.wrapping_add(delay as UnsignedInt);
        let pos = position_of(sim, owner);
        self.do_phase_stuff(sim, owner, SCPHASE_INITIAL, &pos);
        self.collapse_state = COLLAPSESTATE_WAITINGFORCOLLAPSESTART;
        self.current_height = 0.0;
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn do_collapse_done_stuff(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        sim.stop_all_bone_fx(owner);
    }
}

impl DieFacet for StructureCollapseUpdate {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let owner = ctx.owner_slot;
        if self.is_die_applicable(sim, ctx, dmg) == FALSE {
            return;
        }
        sim.ai_mark_as_dead(owner);
        sim.deselect_object(owner, PLAYERMASK_ALL, TRUE);
        self.begin_structure_collapse(sim, ctx);
    }

    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> zh_common::types::Bool {
        self.data
            .die_mux
            .is_die_applicable(sim, ctx.owner_slot, dmg)
    }
}

impl UpdateFacet for StructureCollapseUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.collapse_state == COLLAPSESTATE_STANDING {
            return UpdateSleepTime::FOREVER;
        }
        let owner = ctx.owner_slot;
        if self.collapse_state == COLLAPSESTATE_WAITINGFORCOLLAPSESTART {
            let now = sim.frame();
            let _ = sim.rng_client_real(-self.data.max_shudder, self.data.max_shudder);
            let _ = sim.rng_client_real(-self.data.max_shudder, self.data.max_shudder);
            if now >= self.collapse_frame {
                self.collapse_state = COLLAPSESTATE_COLLAPSING;
                let pos = position_of(sim, owner);
                self.do_phase_stuff(sim, owner, SCPHASE_BURST, &pos);
                let burst_delay =
                    sim.rng_logic_int(self.data.min_burst_delay, self.data.max_burst_delay);
                self.burst_frame = now.wrapping_add(burst_delay as UnsignedInt);
            }
        }
        // Intentional fallthrough from `WAITINGFORCOLLAPSESTART` on the start frame.
        if self.collapse_state == COLLAPSESTATE_COLLAPSING {
            let now = sim.frame();
            self.current_height -= self.collapse_velocity;
            let gravity = sim.rules().global_data.gravity;
            // C++ promotes `m_gravity * (1.0 - m_collapseDamping)` to `double` before narrowing.
            self.collapse_velocity = (f64::from(self.collapse_velocity)
                - f64::from(gravity) * (1.0_f64 - f64::from(self.data.collapse_damping)))
                as Real;
            let _ = sim.rng_client_real(-self.data.max_shudder, self.data.max_shudder);
            let _ = sim.rng_client_real(-self.data.max_shudder, self.data.max_shudder);
            if now >= self.burst_frame {
                let pos = position_of(sim, owner);
                let phase = if sim.rng_logic_int(1, self.data.big_burst_frequency) == 1 {
                    SCPHASE_BURST
                } else {
                    SCPHASE_DELAY
                };
                self.do_phase_stuff(sim, owner, phase, &pos);
                let burst_delay =
                    sim.rng_logic_int(self.data.min_burst_delay, self.data.max_burst_delay);
                self.burst_frame = self.burst_frame.wrapping_add(burst_delay as UnsignedInt);
            }
            let max_height = sim.object(owner).map_or(0.0, |o| {
                o.thing
                    .get_template(&sim.rules().thing_factory)
                    .get_template_geometry_info()
                    .get_max_height_above_position()
            });
            if self.current_height + max_height <= 0.0 {
                self.collapse_state = COLLAPSESTATE_DONE;
                let pos = position_of(sim, owner);
                self.do_phase_stuff(sim, owner, SCPHASE_FINAL, &pos);
                self.do_collapse_done_stuff(sim, owner);
                sim.clear_model_condition_state(owner, ModelConditionFlagType::Rubble);
                sim.set_model_condition_state(owner, ModelConditionFlagType::PostCollapse);
                if let Some(angle) = sim.object(owner).map(|o| o.thing.get_orientation()) {
                    sim.set_object_orientation(owner, angle);
                }
                return UpdateSleepTime::FOREVER;
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for StructureCollapseUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.collapse_frame)?;
        xfer.xfer_unsigned_int(&mut self.burst_frame)?;
        let mut state = self.collapse_state.to_le_bytes();
        xfer.xfer_user(&mut state)?;
        self.collapse_state = Int::from_le_bytes(state);
        xfer.xfer_real(&mut self.collapse_velocity)?;
        xfer.xfer_real(&mut self.current_height)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

pub fn parse_structure_collapse_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_data::<StructureCollapseUpdateModuleData>(reader, tag_key, module_name_key, ctx)
}

pub fn create_structure_collapse_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<StructureCollapseUpdateModuleData>()
        .expect("StructureCollapseUpdate data type mismatch")
        .clone();
    Box::new(StructureCollapseUpdate::new(typed))
}
