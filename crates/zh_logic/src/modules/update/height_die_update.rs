//! `PKT-333`: `HeightDieUpdate`
//! (`HeightDieUpdate.h`, `HeightDieUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-333_HeightDieUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp54_c.rs`.
//!
//! Destroying attached particle systems is client work; only the flag recording
//! it is kept. The structure scan does not model the reference partition cell
//! walk: see `SimView::find_objects_in_range_bounding_sphere_3d`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

#[derive(Debug, Clone, PartialEq)]
pub struct HeightDieUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub target_height_above_terrain: Real,
    pub target_height_includes_structures: Bool,
    pub only_when_moving_down: Bool,
    pub destroy_attached_particles_at_height: Real,
    pub snap_to_ground_on_death: Bool,
    pub initial_delay: UnsignedInt,
}

impl HeightDieUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            target_height_above_terrain: 0.0,
            target_height_includes_structures: FALSE,
            only_when_moving_down: FALSE,
            destroy_attached_particles_at_height: -1.0,
            snap_to_ground_on_death: FALSE,
            initial_delay: 0,
        }
    }
}

impl ModuleData for HeightDieUpdateModuleData {
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

/// Runtime instance of `HeightDieUpdate`.
pub struct HeightDieUpdate {
    pub data: HeightDieUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub has_died: Bool,
    pub particles_destroyed: Bool,
    pub last_position: Coord3D,
    pub earliest_death_frame: UnsignedInt,
}

fn position(sim: &dyn SimApi, owner: ObjectSlot) -> Option<Coord3D> {
    sim.object(owner).map(|obj| *obj.thing.get_position())
}

impl HeightDieUpdate {
    pub fn new(data: HeightDieUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            has_died: FALSE,
            particles_destroyed: FALSE,
            last_position: Coord3D::new(-1.0, -1.0, -1.0),
            earliest_death_frame: UnsignedInt::MAX,
        }
    }

    /// The height of the tallest structure touching the owner's bounding circle.
    fn tallest_structure(sim: &dyn SimApi, owner: ObjectSlot) -> Real {
        let Some(range) = sim
            .object(owner)
            .map(|obj| obj.geometry.get_bounding_circle_radius())
        else {
            return 0.0;
        };
        let mut tallest: Real = 0.0;
        for slot in sim.find_objects_in_range_bounding_sphere_3d(owner, range) {
            if sim.object_is_kind_of(slot, KindOfType::Structure) == FALSE {
                continue;
            }
            if let Some(obj) = sim.object(slot) {
                let height = obj.geometry.get_max_height_above_position();
                if height > tallest {
                    tallest = height;
                }
            }
        }
        tallest
    }
}

impl UpdateFacet for HeightDieUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let now = sim.frame();
        if self.earliest_death_frame == UnsignedInt::MAX {
            self.earliest_death_frame = now.wrapping_add(self.data.initial_delay);
        }
        if self.earliest_death_frame > now {
            return UpdateSleepTime::NONE;
        }
        let Some(obj) = sim.object(owner) else {
            return UpdateSleepTime::NONE;
        };
        let contained = obj.contained_by.is_some();
        let mut pos = *obj.thing.get_position();
        if contained {
            self.last_position = pos;
            return UpdateSleepTime::NONE;
        }

        let mut direction_ok = true;
        if self.has_died == FALSE {
            if self.data.only_when_moving_down != FALSE && pos.z >= self.last_position.z {
                direction_ok = false;
            }
            let mut terrain_height = sim.terrain().get_ground_height(pos.x, pos.y, None);
            if self.data.target_height_includes_structures != FALSE {
                let layer = sim.get_highest_layer_for_destination(&pos, FALSE);
                if layer != PathfindLayerEnum::Ground {
                    let layer_height = sim.get_layer_height(pos.x, pos.y, layer);
                    if layer_height > terrain_height {
                        terrain_height = layer_height;
                    }
                }
            }
            let mut target_height = terrain_height + self.data.target_height_above_terrain;
            if self.data.target_height_includes_structures != FALSE {
                let tallest = Self::tallest_structure(sim, owner);
                if tallest > self.data.target_height_above_terrain {
                    target_height = tallest + terrain_height;
                }
            }
            if pos.z < target_height && direction_ok {
                if self.data.snap_to_ground_on_death != FALSE || pos.z < terrain_height {
                    let ground = Coord3D::new(pos.x, pos.y, terrain_height);
                    sim.set_object_position(owner, &ground);
                }
                kill_object(sim, owner, DamageType::Unresistable, DeathType::Normal);
                self.has_died = TRUE;
                // The reference holds a pointer to the live position.
                pos = position(sim, owner).unwrap_or(pos);
            }
        }

        if self.particles_destroyed == FALSE
            && pos.z < self.data.destroy_attached_particles_at_height
            && (self.has_died != FALSE || direction_ok)
        {
            self.particles_destroyed = TRUE;
        }
        self.last_position = pos;
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for HeightDieUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.has_died)?;
        xfer.xfer_bool(&mut self.particles_destroyed)?;
        xfer.xfer_coord3d(&mut self.last_position)?;
        if version >= 2 {
            xfer.xfer_unsigned_int(&mut self.earliest_death_frame)?;
        } else {
            self.earliest_death_frame = 0;
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

pub fn create_height_die_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<HeightDieUpdateModuleData>()
        .expect("HeightDieUpdate data type mismatch")
        .clone();
    Box::new(HeightDieUpdate::new(typed))
}

pub fn parse_height_die_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = HeightDieUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("TargetHeight") {
            r.parse_real(&mut data.target_height_above_terrain)
        } else if field.eq_ignore_ascii_case("TargetHeightIncludesStructures") {
            r.parse_bool(&mut data.target_height_includes_structures)
        } else if field.eq_ignore_ascii_case("OnlyWhenMovingDown") {
            r.parse_bool(&mut data.only_when_moving_down)
        } else if field.eq_ignore_ascii_case("DestroyAttachedParticlesAtHeight") {
            r.parse_real(&mut data.destroy_attached_particles_at_height)
        } else if field.eq_ignore_ascii_case("SnapToGroundOnDeath") {
            r.parse_bool(&mut data.snap_to_ground_on_death)
        } else if field.eq_ignore_ascii_case("InitialDelay") {
            r.parse_duration_unsigned_int(&mut data.initial_delay)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}
