//! `PKT-367`: `SpawnPointProductionExitUpdateModuleData` and
//! `SpawnPointProductionExitUpdate` (`SpawnPointProductionExitUpdate.h`/`.cpp`).
//!
//! Packet: `packets/phase5/PKT-367_SpawnPointProductionExitUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt315_379_357_367_exits.rs`.
//!
//! The spawn points are the model bones `name01`..`name10`. Without them the exit
//! has no spawn points, which is what the reference does for such a drawable.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot, ThingTemplateId};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ExitDoorType, ExitFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::default_production_exit_update::producer;
use crate::rules::disabled_types::DisabledType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, NameKeyType, ObjectId, Real, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

pub const MAX_SPAWN_POINTS: usize = 10;

/// INI template data matching `SpawnPointProductionExitUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpawnPointProductionExitUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub spawn_point_bone_name: String,
}

impl SpawnPointProductionExitUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            spawn_point_bone_name: String::new(),
        }
    }
}

impl ModuleData for SpawnPointProductionExitUpdateModuleData {
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
        FacetMask(0x001 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// Runtime instance of `SpawnPointProductionExitUpdate`. Only the occupiers are
/// saved; the points are read from the model again after a load.
pub struct SpawnPointProductionExitUpdate {
    pub data: SpawnPointProductionExitUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub bones_initialized: Bool,
    pub spawn_point_count: usize,
    pub world_coord_spawn_points: [Coord3D; MAX_SPAWN_POINTS],
    pub world_angle_spawn_points: [Real; MAX_SPAWN_POINTS],
    pub spawn_point_occupier: [ObjectId; MAX_SPAWN_POINTS],
}

impl SpawnPointProductionExitUpdate {
    pub fn new(data: SpawnPointProductionExitUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            bones_initialized: FALSE,
            spawn_point_count: 0,
            world_coord_spawn_points: [Coord3D::default(); MAX_SPAWN_POINTS],
            world_angle_spawn_points: [0.0; MAX_SPAWN_POINTS],
            spawn_point_occupier: [ObjectId::INVALID; MAX_SPAWN_POINTS],
        }
    }

    fn initialize_bone_positions(&mut self, sim: &dyn SimView, owner: ObjectSlot) {
        if sim.object_has_drawable(owner) == FALSE {
            return;
        }
        let Some(me) = producer(sim, owner) else {
            return;
        };
        let bones = sim.pristine_bone_transforms_indexed(
            owner,
            &self.data.spawn_point_bone_name,
            1,
            MAX_SPAWN_POINTS,
        );
        self.spawn_point_count = bones.len().min(MAX_SPAWN_POINTS);
        for (i, bone) in bones.iter().take(MAX_SPAWN_POINTS).enumerate() {
            let world = Matrix3D::mul(&me.transform, bone);
            let t = world.get_translation();
            self.world_coord_spawn_points[i] = Coord3D::new(t.x, t.y, 0.0);
            self.world_angle_spawn_points[i] = world.get_z_rotation();
        }
        self.bones_initialized = TRUE;
    }

    fn revalidate_occupiers(&mut self, sim: &dyn SimView) {
        for id in &mut self.spawn_point_occupier[..self.spawn_point_count] {
            if *id != ObjectId::INVALID && sim.find_object(*id).is_none() {
                *id = ObjectId::INVALID;
            }
        }
    }

    fn free_point(&self) -> Option<usize> {
        self.spawn_point_occupier[..self.spawn_point_count]
            .iter()
            .position(|&id| id == ObjectId::INVALID)
    }
}

impl ExitFacet for SpawnPointProductionExitUpdate {
    fn reserve_door_for_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _tmpl_id: ThingTemplateId,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        if self.bones_initialized == FALSE {
            self.initialize_bone_positions(sim, ctx.owner_slot);
        }
        if self.bones_initialized == FALSE {
            return ExitDoorType::DoorNoneAvailable;
        }
        self.revalidate_occupiers(sim);
        match self.free_point() {
            Some(_) => ExitDoorType::Door1,
            None => ExitDoorType::DoorNoneAvailable,
        }
    }

    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        _exit_door: ExitDoorType,
    ) {
        if self.bones_initialized == FALSE {
            self.initialize_bone_positions(sim, ctx.owner_slot);
        }
        let Some(me) = producer(sim, ctx.owner_slot) else {
            return;
        };
        // Unlike a reservation, this does not drop dead occupiers first.
        let Some(index) = self.free_point() else {
            return;
        };
        let mut create_point = self.world_coord_spawn_points[index];
        create_point.z = sim.get_layer_height(create_point.x, create_point.y, me.layer);
        let create_angle = self.world_angle_spawn_points[index];
        self.spawn_point_occupier[index] = sim.object(new_obj).map_or(ObjectId::INVALID, |o| o.id);

        sim.set_object_position(new_obj, &create_point);
        sim.set_object_orientation(new_obj, create_angle);
        sim.set_object_layer(new_obj, me.layer);
        sim.pathfinder_add_object(new_obj);
        sim.set_disabled(new_obj, DisabledType::Held);
    }
}

impl UpdateFacet for SpawnPointProductionExitUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for SpawnPointProductionExitUpdate {
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
        let mut bytes = [0_u8; 4 * MAX_SPAWN_POINTS];
        for (chunk, id) in bytes.chunks_exact_mut(4).zip(self.spawn_point_occupier) {
            chunk.copy_from_slice(&id.0.to_le_bytes());
        }
        xfer.xfer_user(&mut bytes)?;
        for (chunk, id) in bytes.chunks_exact(4).zip(&mut self.spawn_point_occupier) {
            id.0 = i32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        }
        Ok(())
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

    fn as_exit(&self) -> Option<&dyn ExitFacet> {
        Some(self)
    }

    fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
        Some(self)
    }
}

pub fn parse_spawn_point_production_exit_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SpawnPointProductionExitUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("SpawnPointBoneName") {
            r.parse_ascii_string(&mut data.spawn_point_bone_name)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_spawn_point_production_exit_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SpawnPointProductionExitUpdateModuleData>()
        .expect("SpawnPointProductionExitUpdate data type mismatch")
        .clone();
    Box::new(SpawnPointProductionExitUpdate::new(typed))
}
