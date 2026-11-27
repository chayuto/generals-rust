//! `PKT-315`: `DefaultProductionExitUpdateModuleData` and
//! `DefaultProductionExitUpdate` (`DefaultProductionExitUpdate.h`/`.cpp`), plus
//! the door geometry that the supply center and queue exits repeat verbatim.
//!
//! Packet: `packets/phase5/PKT-315_DefaultProductionExitUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt315_379_357_367_exits.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, CommandSourceType, ExitDoorType, ExitFacet, ModuleData, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, NameKeyType, Real, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// What an exit reads from the producing object.
#[derive(Debug, Clone, Copy)]
pub struct Producer {
    pub transform: Matrix3D,
    pub angle: Real,
    pub layer: PathfindLayerEnum,
}

pub fn producer(sim: &dyn SimView, owner: ObjectSlot) -> Option<Producer> {
    Some(Producer {
        transform: sim.object_transform(owner)?,
        angle: sim.object_orientation(owner)?,
        layer: sim.object_layer(owner)?,
    })
}

/// `getExitPosition`: the create point in world space, height untouched.
pub fn exit_position(
    sim: &dyn SimView,
    owner: ObjectSlot,
    unit_create_point: &Coord3D,
    exit_position: &mut Coord3D,
) -> Bool {
    let Some(me) = producer(sim, owner) else {
        return FALSE;
    };
    *exit_position = Matrix3D::transform_vector(&me.transform, unit_create_point);
    TRUE
}

/// `getNaturalRallyPoint`: with `offset`, the point is first pushed two path cells
/// further from the producer's origin.
pub fn natural_rally_point(
    sim: &dyn SimView,
    owner: ObjectSlot,
    natural: &Coord3D,
    rally_point: &mut Coord3D,
    offset: Bool,
) -> Bool {
    let Some(me) = producer(sim, owner) else {
        return FALSE;
    };
    let mut p = *natural;
    if offset != FALSE {
        let mut push = p;
        push.normalize_ww_vector3();
        let length = 2.0 * PATHFIND_CELL_SIZE_F;
        p.x += push.x * length;
        p.y += push.y * length;
        p.z += push.z * length;
    }
    *rally_point = Matrix3D::transform_vector(&me.transform, &p);
    TRUE
}

/// The rally leg shared by the default and queue exits: only a ground mover with a
/// reachable rally point gets it appended.
pub fn push_ground_rally_point(
    sim: &mut dyn SimApi,
    new_obj: ObjectSlot,
    rally_point: Coord3D,
    exit_path: &mut Vec<Coord3D>,
) {
    let mut tmp = rally_point;
    if sim.has_ai(new_obj) != FALSE
        && sim.ai_is_doing_ground_movement(new_obj) != FALSE
        && sim.pathfinder_adjust_destination(new_obj, &mut tmp) != FALSE
    {
        exit_path.push(tmp);
    }
}

pub fn follow_exit_path(
    sim: &mut dyn SimApi,
    new_obj: ObjectSlot,
    exit_path: &[Coord3D],
    producer: ObjectSlot,
) {
    if sim.has_ai(new_obj) != FALSE {
        sim.ai_follow_exit_production_path(
            new_obj,
            exit_path,
            Some(producer),
            CommandSourceType::FromAi,
        );
    }
}

/// INI template data matching `DefaultProductionExitUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultProductionExitUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub unit_create_point: Coord3D,
    pub natural_rally_point: Coord3D,
    pub use_spawn_rally_point: Bool,
}

impl DefaultProductionExitUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            unit_create_point: Coord3D::default(),
            natural_rally_point: Coord3D::default(),
            use_spawn_rally_point: FALSE,
        }
    }
}

impl ModuleData for DefaultProductionExitUpdateModuleData {
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

/// Runtime instance of `DefaultProductionExitUpdate`.
pub struct DefaultProductionExitUpdate {
    pub data: DefaultProductionExitUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub rally_point: Coord3D,
    pub rally_point_exists: Bool,
}

impl DefaultProductionExitUpdate {
    pub fn new(data: DefaultProductionExitUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            rally_point: Coord3D::default(),
            rally_point_exists: FALSE,
        }
    }
}

impl ExitFacet for DefaultProductionExitUpdate {
    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        _exit_door: ExitDoorType,
    ) {
        let Some(me) = producer(sim, ctx.owner_slot) else {
            return;
        };
        let mut create_point =
            Matrix3D::transform_vector(&me.transform, &self.data.unit_create_point);
        create_point.z = sim.get_layer_height(create_point.x, create_point.y, me.layer);

        sim.set_object_position(new_obj, &create_point);
        sim.set_object_orientation(new_obj, me.angle);
        sim.set_object_layer(new_obj, me.layer);
        sim.pathfinder_add_object(new_obj);

        let mut natural = Coord3D::default();
        natural_rally_point(
            sim,
            ctx.owner_slot,
            &self.data.natural_rally_point,
            &mut natural,
            TRUE,
        );
        let mut exit_path = vec![natural];
        if self.rally_point_exists != FALSE {
            push_ground_rally_point(sim, new_obj, self.rally_point, &mut exit_path);
        }
        follow_exit_path(sim, new_obj, &exit_path, ctx.owner_slot);
    }

    fn set_rally_point(&mut self, pos: &Coord3D) {
        self.rally_point = *pos;
        self.rally_point_exists = TRUE;
    }

    fn get_rally_point(&self) -> Option<&Coord3D> {
        (self.rally_point_exists != FALSE).then_some(&self.rally_point)
    }

    fn use_spawn_rally_point(&self) -> Bool {
        self.data.use_spawn_rally_point
    }

    fn get_natural_rally_point(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rally_point: &mut Coord3D,
        offset: Bool,
    ) -> Bool {
        natural_rally_point(
            sim,
            ctx.owner_slot,
            &self.data.natural_rally_point,
            rally_point,
            offset,
        )
    }

    fn get_exit_position(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        exit_pos: &mut Coord3D,
    ) -> Bool {
        exit_position(sim, ctx.owner_slot, &self.data.unit_create_point, exit_pos)
    }
}

impl UpdateFacet for DefaultProductionExitUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for DefaultProductionExitUpdate {
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
        xfer.xfer_coord3d(&mut self.rally_point)?;
        xfer.xfer_bool(&mut self.rally_point_exists)
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

pub fn parse_default_production_exit_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = DefaultProductionExitUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("UnitCreatePoint") {
            r.parse_coord3d(&mut data.unit_create_point)
        } else if field.eq_ignore_ascii_case("NaturalRallyPoint") {
            r.parse_coord3d(&mut data.natural_rally_point)
        } else if field.eq_ignore_ascii_case("UseSpawnRallyPoint") {
            r.parse_bool(&mut data.use_spawn_rally_point)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_default_production_exit_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<DefaultProductionExitUpdateModuleData>()
        .expect("DefaultProductionExitUpdate data type mismatch")
        .clone();
    Box::new(DefaultProductionExitUpdate::new(typed))
}
