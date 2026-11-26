//! `PKT-320`: the abstract `DockUpdate` base (`DockUpdate.h`, `DockUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-320_DockUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt320_381_378_363_docks.rs`.
//!
//! The reference reads the dock, enter, exit and waiting positions from model bones.
//! This port loads no models, so it behaves as the reference does for a model that
//! has none of those bones: the stored positions are left alone, the bone count
//! becomes zero and the fixed approach positions are zeroed.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::sim::{FindPositionOptions, SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{gm_sqrtf, Coord3D};
use zh_common::types::{Bool, Int, NameKeyType, ObjectId, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// `DEFAULT_APPROACH_VECTOR_SIZE`.
const DEFAULT_APPROACH_VECTOR_SIZE: usize = 10;
/// `DYNAMIC_APPROACH_VECTOR_FLAG`.
const DYNAMIC_APPROACH_VECTOR_FLAG: Int = -1;

/// `DockUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct DockUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub number_approach_positions: Int,
    pub allows_passthrough: Bool,
}

impl DockUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            number_approach_positions: 0,
            allows_passthrough: TRUE,
        }
    }

    pub const FACET_MASK: FacetMask = FacetMask(0x001 | FacetMask::DOCK_UPDATE.0);

    /// `buildFieldParse`; `Err(UnknownToken)` when the field is not a base one.
    pub fn parse_field(&mut self, field: &str, r: &mut IniReader<'_>) -> Result<(), IniError> {
        if field.eq_ignore_ascii_case("NumberApproachPositions") {
            r.parse_int(&mut self.number_approach_positions)
        } else if field.eq_ignore_ascii_case("AllowsPassthrough") {
            r.parse_bool(&mut self.allows_passthrough)
        } else {
            Err(IniError::UnknownToken)
        }
    }
}

/// The members of `DockUpdate`.
pub struct DockUpdateState {
    pub data: DockUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub enter_position: Coord3D,
    pub dock_position: Coord3D,
    pub exit_position: Coord3D,
    pub number_approach_positions: Int,
    /// Not saved. `-1` until the positions are loaded.
    pub number_approach_position_bones: Int,
    pub positions_loaded: Bool,
    pub approach_positions: Vec<Coord3D>,
    pub approach_position_owners: Vec<ObjectId>,
    pub approach_position_reached: Vec<Bool>,
    pub active_docker: ObjectId,
    pub docker_inside: Bool,
    pub dock_crippled: Bool,
    pub dock_open: Bool,
}

fn flags(list: &[ModelConditionFlagType]) -> ModelConditionFlags {
    let mut out = ModelConditionFlags::new();
    for f in list {
        out.set(f.as_i32(), 1);
    }
    out
}

fn object_id(sim: &dyn SimView, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).map_or(ObjectId::INVALID, |o| o.id)
}

fn position(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

/// `Object::convertBonePosToWorldPos`.
fn bone_to_world(sim: &dyn SimView, owner: ObjectSlot, bone: &Coord3D) -> Coord3D {
    let mut world = *bone;
    if let Some(o) = sim.object(owner) {
        o.thing
            .convert_bone_pos_to_world_pos(Some(bone), None, Some(&mut world), None);
    }
    world
}

impl DockUpdateState {
    pub fn new(data: DockUpdateModuleData) -> Self {
        let number = data.number_approach_positions;
        // A count below -1 crashes the reference.
        let len = if number == DYNAMIC_APPROACH_VECTOR_FLAG {
            DEFAULT_APPROACH_VECTOR_SIZE
        } else {
            usize::try_from(number).unwrap_or(0)
        };
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            enter_position: Coord3D::default(),
            dock_position: Coord3D::default(),
            exit_position: Coord3D::default(),
            number_approach_positions: number,
            number_approach_position_bones: -1,
            positions_loaded: FALSE,
            approach_positions: vec![Coord3D::default(); len],
            approach_position_owners: vec![ObjectId::INVALID; len],
            approach_position_reached: vec![FALSE; len],
            active_docker: ObjectId::INVALID,
            docker_inside: FALSE,
            dock_crippled: FALSE,
            dock_open: TRUE,
        }
    }

    /// `loadDockPositions` for a model without the dock bones.
    pub fn load_dock_positions(&mut self, sim: &dyn SimView, owner: ObjectSlot) {
        if sim.object_has_drawable(owner) == FALSE {
            return;
        }
        self.number_approach_position_bones = 0;
        if sim.object_is_kind_of(owner, KindOfType::IgnoreDockingBones) == FALSE
            && self.number_approach_positions != DYNAMIC_APPROACH_VECTOR_FLAG
            && usize::try_from(self.number_approach_positions)
                .is_ok_and(|n| n == self.approach_positions.len())
        {
            // The reference copies from a zeroed array of ten. Past ten it reads
            // beyond the array; zero stands in for that.
            self.approach_positions.fill(Coord3D::default());
        }
        self.positions_loaded = TRUE;
    }

    fn ensure_positions(&mut self, sim: &dyn SimView, owner: ObjectSlot) {
        if self.positions_loaded == FALSE {
            self.load_dock_positions(sim, owner);
        }
    }

    pub fn is_clear_to_approach(&self, docker: ObjectId) -> Bool {
        if self.number_approach_positions == DYNAMIC_APPROACH_VECTOR_FLAG {
            return TRUE;
        }
        Bool::from(
            self.approach_position_owners
                .iter()
                .any(|&o| o == ObjectId::INVALID || o == docker),
        )
    }

    /// `computeApproachPosition`. The search draws from the logic PRNG.
    fn compute_approach_position(
        &mut self,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        index: usize,
        for_whom: ObjectSlot,
    ) -> Coord3D {
        self.ensure_positions(sim, owner);
        let bone = self
            .approach_positions
            .get(index)
            .copied()
            .unwrap_or_default();
        let mut working = bone_to_world(sim, owner, &bone);
        if self.number_approach_position_bones == 0 {
            let ours = position(sim, owner);
            let theirs = position(sim, for_whom);
            let (mut x, mut y, mut z) = (theirs.x - ours.x, theirs.y - ours.y, theirs.z - ours.z);
            // `Vector3::Normalize`.
            let len2 = x * x + y * y + z * z;
            if len2 != 0.0 {
                let inv = 1.0_f32 / gm_sqrtf(len2);
                x *= inv;
                y *= inv;
                z *= inv;
            }
            let half = sim
                .object(owner)
                .map_or(0.0, |o| o.geometry.get_major_radius())
                / 2.0;
            working.x += x * half;
            working.y += y * half;
            working.z += z * half;
        }
        let options = FindPositionOptions {
            min_radius: 0.0,
            max_radius: 100.0,
            source_to_path_to_dest: Some(for_whom),
            ignore_object: crate::modules::contain::transport_contain::is_using_airborne_locomotor(
                sim, for_whom,
            )
            .then_some(owner),
            ..FindPositionOptions::default()
        };
        sim.find_position_around(&working, &options)
            .unwrap_or(working)
    }

    /// `reserveApproachPosition`: the position and index given to `docker`, or `None`.
    pub fn reserve_approach_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Option<(Coord3D, Int)> {
        let owner = ctx.owner_slot;
        self.ensure_positions(sim, owner);
        let id = object_id(sim, docker);
        let found = self
            .approach_position_owners
            .iter()
            .position(|&o| o == id || o == ObjectId::INVALID);
        let index = match found {
            Some(i) => {
                self.approach_position_owners[i] = id;
                i
            }
            None if self.number_approach_positions == DYNAMIC_APPROACH_VECTOR_FLAG => {
                self.approach_positions.push(Coord3D::default());
                self.approach_position_owners.push(ObjectId::INVALID);
                self.approach_position_reached.push(FALSE);
                self.load_dock_positions(sim, owner);
                let i = self.approach_position_owners.len() - 1;
                self.approach_position_owners[i] = id;
                i
            }
            None => return None,
        };
        let pos = self.compute_approach_position(sim, owner, index, docker);
        Some((pos, index as Int))
    }

    /// `advanceApproachPosition` from `index`: the new position and index, or `None`.
    pub fn advance_approach_position(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
        index: Int,
    ) -> Option<(Coord3D, Int)> {
        let owner = ctx.owner_slot;
        self.ensure_positions(sim, owner);
        if index <= 0 {
            return None;
        }
        let his = index as usize;
        // An index past the end is a read past the vector in the reference.
        if his >= self.approach_position_owners.len()
            || self.approach_position_owners[his - 1] != ObjectId::INVALID
        {
            return None;
        }
        let previous = his - 1;
        self.approach_position_owners[previous] = object_id(sim, docker);
        self.approach_position_owners[his] = ObjectId::INVALID;
        for i in [previous, his] {
            if let Some(r) = self.approach_position_reached.get_mut(i) {
                *r = FALSE;
            }
        }
        let pos = self.compute_approach_position(sim, owner, previous, docker);
        Some((pos, previous as Int))
    }

    pub fn is_clear_to_enter(&self, docker: ObjectId) -> Bool {
        Bool::from(docker == self.active_docker)
    }

    pub fn is_clear_to_advance(&self, docker: ObjectId, index: Int) -> Bool {
        let Ok(i) = usize::try_from(index) else {
            return FALSE;
        };
        let correct = self.approach_position_owners.get(i) == Some(&docker);
        let reached = self.approach_position_reached.get(i) == Some(&TRUE);
        let next_free =
            i > 0 && self.approach_position_owners.get(i - 1) == Some(&ObjectId::INVALID);
        Bool::from(correct && reached && next_free)
    }

    pub fn get_enter_position(
        &mut self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Coord3D {
        self.ensure_positions(sim, ctx.owner_slot);
        if self.enter_position == Coord3D::default() {
            if crate::modules::contain::transport_contain::is_using_airborne_locomotor(sim, docker)
            {
                return position(sim, ctx.owner_slot);
            }
            return position(sim, docker);
        }
        bone_to_world(sim, ctx.owner_slot, &self.enter_position)
    }

    /// `getDockPosition`. The test is on the enter position, as in the reference.
    pub fn get_dock_position(
        &mut self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Coord3D {
        self.ensure_positions(sim, ctx.owner_slot);
        if self.enter_position == Coord3D::default() {
            return position(sim, docker);
        }
        bone_to_world(sim, ctx.owner_slot, &self.dock_position)
    }

    pub fn get_exit_position(
        &mut self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) -> Coord3D {
        self.ensure_positions(sim, ctx.owner_slot);
        if self.enter_position == Coord3D::default() {
            return position(sim, docker);
        }
        bone_to_world(sim, ctx.owner_slot, &self.exit_position)
    }

    pub fn on_approach_reached(&mut self, docker: ObjectId) {
        if let Some(i) = self
            .approach_position_owners
            .iter()
            .position(|&o| o == docker)
        {
            if let Some(r) = self.approach_position_reached.get_mut(i) {
                *r = TRUE;
            }
        }
    }

    pub fn on_enter_reached(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) {
        use ModelConditionFlagType as Mc;
        let clear = flags(&[Mc::DockingEnding]);
        let set = flags(&[Mc::DockingBeginning, Mc::Docking]);
        sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
        sim.clear_and_set_model_condition_flags(docker, &clear, &set);
        self.docker_inside = TRUE;
        let id = object_id(sim, docker);
        if let Some(i) = self.approach_position_owners.iter().position(|&o| o == id) {
            self.approach_position_owners[i] = ObjectId::INVALID;
            if let Some(r) = self.approach_position_reached.get_mut(i) {
                *r = FALSE;
            }
        }
    }

    pub fn on_dock_reached(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) {
        use ModelConditionFlagType as Mc;
        let clear = flags(&[Mc::DockingBeginning]);
        let set = flags(&[Mc::DockingActive]);
        sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
        sim.clear_and_set_model_condition_flags(docker, &clear, &set);
    }

    pub fn on_exit_reached(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        docker: ObjectSlot,
    ) {
        use ModelConditionFlagType as Mc;
        let clear = flags(&[Mc::DockingActive, Mc::Docking]);
        let set = flags(&[Mc::DockingEnding]);
        sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
        sim.clear_and_set_model_condition_flags(docker, &clear, &set);
        self.docker_inside = FALSE;
        if object_id(sim, docker) == self.active_docker {
            self.active_docker = ObjectId::INVALID;
        }
    }

    pub fn cancel_dock(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, docker: ObjectSlot) {
        use ModelConditionFlagType as Mc;
        let id = object_id(sim, docker);
        for i in 0..self.approach_position_owners.len() {
            if self.approach_position_owners[i] == id {
                self.approach_position_owners[i] = ObjectId::INVALID;
                if let Some(r) = self.approach_position_reached.get_mut(i) {
                    *r = FALSE;
                }
            }
        }
        if self.active_docker == id {
            let docking = sim.find_object(self.active_docker);
            self.active_docker = ObjectId::INVALID;
            self.docker_inside = FALSE;
            let clear = flags(&[
                Mc::DockingEnding,
                Mc::DockingBeginning,
                Mc::DockingActive,
                Mc::Docking,
            ]);
            let set = ModelConditionFlags::new();
            sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
            if let Some(other) = docking {
                sim.clear_and_set_model_condition_flags(other, &clear, &set);
            }
        }
    }

    /// `DockUpdate::update`.
    pub fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.active_docker == ObjectId::INVALID && self.dock_crippled == FALSE {
            if let Some(i) = self
                .approach_position_reached
                .iter()
                .position(|&r| r != FALSE)
            {
                self.active_docker = self
                    .approach_position_owners
                    .get(i)
                    .copied()
                    .unwrap_or(ObjectId::INVALID);
            }
        } else if sim.object_is_kind_of(ctx.owner_slot, KindOfType::SupplySource) != FALSE {
            if let Some(docker) = sim.find_object(self.active_docker) {
                if sim.object_is_kind_of(docker, KindOfType::Dozer) != FALSE
                    && sim.object_is_kind_of(docker, KindOfType::Harvester) != FALSE
                    && sim.object_has_drawable(docker) != FALSE
                    && sim.object(docker).is_some_and(|o| {
                        o.model_condition
                            .test(ModelConditionFlagType::DockingBeginning.as_i32())
                            != FALSE
                    })
                {
                    sim.clear_model_condition_state(docker, ModelConditionFlagType::Moving);
                }
            }
        }
        UpdateSleepTime::NONE
    }

    /// `DockUpdate::crc`.
    pub fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// `DockUpdate::xfer`.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_coord3d(&mut self.enter_position)?;
        xfer.xfer_coord3d(&mut self.dock_position)?;
        xfer.xfer_coord3d(&mut self.exit_position)?;
        xfer.xfer_int(&mut self.number_approach_positions)?;
        xfer.xfer_bool(&mut self.positions_loaded)?;

        let mut size = self.approach_positions.len() as Int;
        xfer.xfer_int(&mut size)?;
        self.approach_positions
            .resize(usize::try_from(size).unwrap_or(0), Coord3D::default());
        for p in &mut self.approach_positions {
            xfer.xfer_coord3d(p)?;
        }
        size = self.approach_position_owners.len() as Int;
        xfer.xfer_int(&mut size)?;
        self.approach_position_owners
            .resize(usize::try_from(size).unwrap_or(0), ObjectId::INVALID);
        for id in &mut self.approach_position_owners {
            xfer.xfer_object_id(id)?;
        }
        size = self.approach_position_reached.len() as Int;
        xfer.xfer_int(&mut size)?;
        self.approach_position_reached
            .resize(usize::try_from(size).unwrap_or(0), FALSE);
        for r in &mut self.approach_position_reached {
            xfer.xfer_bool(r)?;
        }

        xfer.xfer_object_id(&mut self.active_docker)?;
        xfer.xfer_bool(&mut self.docker_inside)?;
        xfer.xfer_bool(&mut self.dock_crippled)?;
        xfer.xfer_bool(&mut self.dock_open)
    }

    /// `DockUpdate::loadPostProcess`.
    pub fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
        if self.positions_loaded != FALSE {
            self.load_dock_positions(sim, ctx.owner_slot);
        }
    }
}

/// The `BehaviorModule` methods every dock shares. `$state` is the field that holds
/// the [`DockUpdateState`].
macro_rules! dock_behavior_common {
    ($state:ident) => {
        fn module_name_key(&self) -> zh_common::types::NameKeyType {
            self.$state.data.module_name_key
        }

        fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
            self.$state.schedule.bind(ctx);
        }

        fn crc(&self, xfer: &mut dyn Xfer) {
            self.$state.crc(xfer);
        }

        fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
            self.$state.load_post_process(sim, ctx);
        }

        fn as_any(&self) -> Option<&dyn std::any::Any> {
            Some(self)
        }

        fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
            Some(self)
        }

        fn as_update(&self) -> Option<&dyn crate::modules::module::UpdateFacet> {
            Some(self)
        }

        fn as_update_mut(&mut self) -> Option<&mut dyn crate::modules::module::UpdateFacet> {
            Some(self)
        }

        fn as_dock(&self) -> Option<&dyn crate::modules::module::DockFacet> {
            Some(self)
        }

        fn as_dock_mut(&mut self) -> Option<&mut dyn crate::modules::module::DockFacet> {
            Some(self)
        }
    };
}
pub(crate) use dock_behavior_common;

/// `ModuleData` for a dock's data struct that keeps the base in `$base`.
macro_rules! dock_module_data {
    ($ty:ty, $($base:ident).+) => {
        impl crate::modules::module::ModuleData for $ty {
            fn as_any(&self) -> &dyn std::any::Any {
                self
            }

            fn tag_key(&self) -> zh_common::types::NameKeyType {
                self.$($base).+.tag_key
            }

            fn module_name_key(&self) -> zh_common::types::NameKeyType {
                self.$($base).+.module_name_key
            }

            fn facet_mask(&self) -> crate::arena::FacetMask {
                crate::modules::update::dock_update::DockUpdateModuleData::FACET_MASK
            }
        }
    };
}
pub(crate) use dock_module_data;
