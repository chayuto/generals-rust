//! `PKT-355`: `ProjectileStreamUpdate`
//! (`ProjectileStreamUpdate.h`, `ProjectileStreamUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-355_ProjectileStreamUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_b.rs`.
//!
//! `add_projectile` is called by weapon code that is not ported yet, so nothing
//! in the simulation calls it. `set_position` and `get_all_points` are likewise
//! public for the callers that will need them.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Int, NameKeyType, ObjectId, Real, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

pub const MAX_PROJECTILE_STREAM: usize = 20;

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectileStreamUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ProjectileStreamUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
        }
    }
}

impl ModuleData for ProjectileStreamUpdateModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x001)
    }
}

/// Runtime instance of `ProjectileStreamUpdate`.
pub struct ProjectileStreamUpdate {
    pub data: ProjectileStreamUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub projectile_ids: [ObjectId; MAX_PROJECTILE_STREAM],
    pub next_free_index: Int,
    pub first_valid_index: Int,
    pub owning_object: ObjectId,
    pub target_object: ObjectId,
    pub target_position: Coord3D,
}

fn next_index(index: Int) -> Int {
    index.wrapping_add(1) % MAX_PROJECTILE_STREAM as Int
}

impl ProjectileStreamUpdate {
    pub fn new(data: ProjectileStreamUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            projectile_ids: [ObjectId::INVALID; MAX_PROJECTILE_STREAM],
            next_free_index: 0,
            first_valid_index: 0,
            owning_object: ObjectId::INVALID,
            target_object: ObjectId::INVALID,
            target_position: Coord3D::default(),
        }
    }

    fn push(&mut self, id: ObjectId) {
        self.projectile_ids[self.next_free_index as usize] = id;
        self.next_free_index = next_index(self.next_free_index);
    }

    /// `addProjectile`. A hole is left in the stream when the target changes.
    pub fn add_projectile(
        &mut self,
        source_id: ObjectId,
        new_id: ObjectId,
        victim_id: ObjectId,
        victim_pos: Option<&Coord3D>,
    ) {
        if self.owning_object == ObjectId::INVALID {
            self.owning_object = source_id;
        }
        if victim_id != ObjectId::INVALID {
            if victim_id != self.target_object {
                self.push(ObjectId::INVALID);
                self.target_object = victim_id;
            }
            self.target_position.zero();
        } else if let Some(pos) = victim_pos {
            if self.target_position != *pos {
                self.push(ObjectId::INVALID);
                self.target_position = *pos;
            }
            self.target_object = ObjectId::INVALID;
        }
        self.push(new_id);
    }

    fn cull_front_of_list(&mut self, sim: &dyn SimApi) {
        while self.first_valid_index != self.next_free_index
            && sim
                .find_object(self.projectile_ids[self.first_valid_index as usize])
                .is_none()
        {
            self.first_valid_index = next_index(self.first_valid_index);
        }
    }

    fn consider_dying(&self, sim: &dyn SimApi) -> bool {
        self.first_valid_index == self.next_free_index
            && self.owning_object != ObjectId::INVALID
            && sim.find_object(self.owning_object).is_none()
    }

    /// `getAllPoints`.
    pub fn get_all_points(&self, sim: &dyn SimView) -> Vec<Coord3D> {
        let mut points = Vec::new();
        let mut index = self.first_valid_index;
        let owner = sim.find_object(self.owning_object);
        while index != self.next_free_index {
            let projectile = sim
                .find_object(self.projectile_ids[index as usize])
                .and_then(|slot| sim.object(slot));
            let mut point = Coord3D::default();
            if let Some(projectile) = projectile {
                point = *projectile.thing.get_position();
                if let Some(slot) = owner {
                    if let Some(obj) = sim.object(slot) {
                        if sim.object_is_kind_of(slot, KindOfType::Vehicle) != FALSE {
                            let pos = obj.thing.get_position();
                            let my_top: Real =
                                obj.geometry.get_max_height_above_position() + pos.z + 0.5;
                            let delta = Coord3D::new(pos.x - point.x, pos.y - point.y, 0.0);
                            if delta.length() <= obj.geometry.get_major_radius() * 1.5 {
                                point.z = if point.z > my_top { point.z } else { my_top };
                            }
                        }
                    }
                }
            }
            points.push(point);
            index = next_index(index);
        }
        points
    }

    /// `setPosition`.
    pub fn set_position(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, pos: &Coord3D) {
        sim.set_object_position(ctx.owner_slot, pos);
    }
}

impl UpdateFacet for ProjectileStreamUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.cull_front_of_list(sim);
        if self.consider_dying(sim) {
            sim.destroy_object(ctx.owner_slot);
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for ProjectileStreamUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let mut raw = [0_u8; MAX_PROJECTILE_STREAM * 4];
        for (chunk, id) in raw.chunks_exact_mut(4).zip(&self.projectile_ids) {
            chunk.copy_from_slice(&id.0.to_le_bytes());
        }
        xfer.xfer_user(&mut raw)?;
        for (chunk, id) in raw.chunks_exact(4).zip(&mut self.projectile_ids) {
            id.0 = Int::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        xfer.xfer_int(&mut self.next_free_index)?;
        xfer.xfer_int(&mut self.first_valid_index)?;
        xfer.xfer_object_id(&mut self.owning_object)?;
        if version >= 2 {
            xfer.xfer_object_id(&mut self.target_object)?;
            xfer.xfer_coord3d(&mut self.target_position)?;
        }
        Ok(())
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
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
}

pub fn create_projectile_stream_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<ProjectileStreamUpdateModuleData>()
        .expect("ProjectileStreamUpdate data type mismatch")
        .clone();
    Box::new(ProjectileStreamUpdate::new(typed))
}

pub fn parse_projectile_stream_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(ProjectileStreamUpdateModuleData::new(
        tag_key,
        module_name_key,
    )))
}
