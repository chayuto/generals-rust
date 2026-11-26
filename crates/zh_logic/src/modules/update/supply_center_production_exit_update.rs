//! `PKT-379`: `SupplyCenterProductionExitUpdateModuleData` and
//! `SupplyCenterProductionExitUpdate` (`SupplyCenterProductionExitUpdate.h`/`.cpp`).
//!
//! Packet: `packets/phase5/PKT-379_SupplyCenterProductionExitUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt315_379_357_367_exits.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ExitDoorType, ExitFacet, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::default_production_exit_update::{
    exit_position, follow_exit_path, natural_rally_point, producer,
};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::{Coord3D, Matrix3D};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `SupplyCenterProductionExitUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SupplyCenterProductionExitUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub unit_create_point: Coord3D,
    pub natural_rally_point: Coord3D,
    /// `GrantTemporaryStealth`, in frames.
    pub grant_temporary_stealth_frames: UnsignedInt,
}

impl SupplyCenterProductionExitUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            unit_create_point: Coord3D::default(),
            natural_rally_point: Coord3D::default(),
            grant_temporary_stealth_frames: 0,
        }
    }
}

impl ModuleData for SupplyCenterProductionExitUpdateModuleData {
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

/// Runtime instance of `SupplyCenterProductionExitUpdate`.
pub struct SupplyCenterProductionExitUpdate {
    pub data: SupplyCenterProductionExitUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// The constructor leaves this unset; pool memory is zeroed.
    pub rally_point: Coord3D,
    pub rally_point_exists: Bool,
}

impl SupplyCenterProductionExitUpdate {
    pub fn new(data: SupplyCenterProductionExitUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            rally_point: Coord3D::default(),
            rally_point_exists: FALSE,
        }
    }

    /// A stealthed center hides the new unit for a while. The reference crashes on
    /// a unit without a `StealthUpdate`; here such a unit gets nothing.
    fn grant_stealth(&self, sim: &mut dyn SimApi, owner: ObjectSlot, new_obj: ObjectSlot) {
        let frames = self.data.grant_temporary_stealth_frames;
        if frames == 0 {
            return;
        }
        let status = |sim: &dyn SimApi, slot: ObjectSlot, s: ObjectStatusType| {
            sim.object(slot)
                .is_some_and(|o| o.status.test(s.as_i32()) != 0)
        };
        if !status(sim, owner, ObjectStatusType::Stealthed) {
            return;
        }
        if sim.stealth_is_temporary_grant(new_obj) != FALSE
            || !status(sim, new_obj, ObjectStatusType::CanStealth)
        {
            sim.receive_temporary_stealth_grant(new_obj, frames);
        }
    }
}

impl ExitFacet for SupplyCenterProductionExitUpdate {
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
        // The ground height, not the layer height, and the layer is not copied.
        create_point.z = sim
            .terrain()
            .get_ground_height(create_point.x, create_point.y, None);

        sim.set_object_position(new_obj, &create_point);
        sim.set_object_orientation(new_obj, me.angle);
        sim.pathfinder_add_object(new_obj);

        // No two-cell offset and no destination adjustment here.
        let natural = Matrix3D::transform_vector(&me.transform, &self.data.natural_rally_point);
        let mut exit_path = vec![natural];
        if self.rally_point_exists != FALSE {
            exit_path.push(self.rally_point);
        }
        if sim.has_ai(new_obj) != FALSE {
            follow_exit_path(sim, new_obj, &exit_path, ctx.owner_slot);
            sim.with_supply_truck(new_obj, &mut |truck, _, _| {
                truck.set_force_wanting_state(TRUE);
            });
        }
        self.grant_stealth(sim, ctx.owner_slot, new_obj);
    }

    fn set_rally_point(&mut self, pos: &Coord3D) {
        self.rally_point = *pos;
        self.rally_point_exists = TRUE;
    }

    fn get_rally_point(&self) -> Option<&Coord3D> {
        (self.rally_point_exists != FALSE).then_some(&self.rally_point)
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

impl UpdateFacet for SupplyCenterProductionExitUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for SupplyCenterProductionExitUpdate {
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

pub fn parse_supply_center_production_exit_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SupplyCenterProductionExitUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("UnitCreatePoint") {
            r.parse_coord3d(&mut data.unit_create_point)
        } else if field.eq_ignore_ascii_case("NaturalRallyPoint") {
            r.parse_coord3d(&mut data.natural_rally_point)
        } else if field.eq_ignore_ascii_case("GrantTemporaryStealth") {
            r.parse_duration_unsigned_int(&mut data.grant_temporary_stealth_frames)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_supply_center_production_exit_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SupplyCenterProductionExitUpdateModuleData>()
        .expect("SupplyCenterProductionExitUpdate data type mismatch")
        .clone();
    Box::new(SupplyCenterProductionExitUpdate::new(typed))
}
