//! `PKT-382`: `TensileFormationUpdate` (`TensileFormationUpdate.h`, `.cpp`): linked pieces
//! of a mass (an avalanche) that slide downhill once one is damaged.
//!
//! Packet: `packets/phase5/PKT-382_TensileFormationUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt334_267_382_346_misc.rs`.
//!
//! The crack sound is client audio and is not modelled.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_fabsf, Coord3D, PI};
use zh_common::types::{
    Bool, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `TensileFormationUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct TensileFormationUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub enabled: Bool,
    pub crack_sound: String,
}

impl TensileFormationUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            enabled: FALSE,
            crack_sound: String::new(),
        }
    }
}

impl ModuleData for TensileFormationUpdateModuleData {
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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct TensileLink {
    pub id: ObjectId,
    /// Offset from this object to the linked one when the links were made.
    pub tensor: Coord3D,
}

/// Runtime instance of `TensileFormationUpdate` (`PKT-382`). Only `enabled` is saved.
pub struct TensileFormationUpdate {
    pub data: TensileFormationUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub links: [TensileLink; 4],
    pub inertia: Coord3D,
    pub enabled: Bool,
    pub links_inited: Bool,
    pub motionless_counter: UnsignedInt,
    pub life: UnsignedInt,
    pub lowest_slide_elevation: Real,
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

fn scale(v: &mut Coord3D, s: Real) {
    v.x *= s;
    v.y *= s;
    v.z *= s;
}

fn set_flag(sim: &mut dyn SimApi, owner: ObjectSlot, flag: ModelConditionFlagType, on: bool) {
    if on {
        sim.set_model_condition_state(owner, flag);
    } else {
        sim.clear_model_condition_state(owner, flag);
    }
}

impl TensileFormationUpdate {
    pub fn new(data: TensileFormationUpdateModuleData) -> Self {
        let enabled = data.enabled;
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            links: [TensileLink {
                id: ObjectId::INVALID,
                tensor: Coord3D::new(0.0, 0.0, 0.0),
            }; 4],
            inertia: Coord3D::new(0.0, 0.0, 0.0),
            enabled,
            links_inited: FALSE,
            motionless_counter: 0,
            life: 0,
            lowest_slide_elevation: 255.0,
        }
    }

    /// The other members of the formation within `radius`.
    fn members(&self, sim: &dyn SimView, owner: ObjectSlot, radius: Real) -> Vec<ObjectSlot> {
        sim.find_objects_in_range_bounding_sphere_3d(owner, radius)
            .into_iter()
            .filter(|&o| sim.has_module_by_name_key(o, self.data.module_name_key))
            .collect()
    }

    /// `initLinks`: the up to four members that were each the closest seen so far.
    fn init_links(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        self.links_inited = TRUE;
        let mut closest: Real = 99999.9;
        let my_pos = position_of(sim, owner);
        for other in self.members(sim, owner, 1000.0) {
            let their = position_of(sim, other);
            let delta = Coord3D::new(their.x - my_pos.x, their.y - my_pos.y, their.z - my_pos.z);
            let distance = delta.length();
            if closest > distance {
                closest = distance;
                for t in (1..4).rev() {
                    self.links[t] = self.links[t - 1];
                }
                self.links[0] = TensileLink {
                    id: sim.object(other).map_or(ObjectId::INVALID, |o| o.id),
                    tensor: delta,
                };
            }
        }
        let angle = sim.rng_logic_real(-PI, PI);
        sim.set_object_orientation(owner, angle);
    }

    /// `propagateDislodgement`.
    fn propagate_dislodgement(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        for other in self.members(sim, owner, 100.0) {
            sim.set_body_damage_state(other, BodyDamageType::Damaged);
        }
        for link in self.links {
            if let Some(other) = sim.find_object(link.id) {
                sim.set_body_damage_state(other, BodyDamageType::Damaged);
            }
        }
    }

    /// `TensileFormationUpdate::update`.
    fn tick(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let us = ctx.owner_slot;
        if self.links_inited == FALSE {
            self.init_links(sim, us);
        }

        if self.enabled == FALSE {
            if sim.object_body_damage_state(us) >= BodyDamageType::Damaged {
                self.enabled = TRUE;
                sim.pathfinder_remove_wall(us);
            } else {
                return UpdateSleepTime(30);
            }
        }

        if sim.object_has_drawable(us) == FALSE {
            return UpdateSleepTime::NONE;
        }

        self.life = self.life.wrapping_add(1);
        if self.life > 300 {
            sim.clear_model_condition_state(us, ModelConditionFlagType::Moving);
            sim.clear_model_condition_state(us, ModelConditionFlagType::Freefall);
            sim.clear_model_condition_state(us, ModelConditionFlagType::PostCollapse);
            sim.set_body_damage_state(us, BodyDamageType::Rubble);
            sim.pathfinder_create_wall(us);
            return UpdateSleepTime::FOREVER;
        }

        if self.life % 30 == 29 {
            self.propagate_dislodgement(sim, us);
        }

        let pos = position_of(sim, us);
        let mut normal = Coord3D::new(0.0, 0.0, 1.0);
        sim.terrain()
            .get_ground_height(pos.x, pos.y, Some(&mut normal));

        let mut slope = Coord3D::new(normal.x, normal.y, 0.0);
        let steepness = 1.0 - normal.z;
        scale(&mut slope, 0.3 + steepness);
        self.inertia.x += slope.x;
        self.inertia.y += slope.y;
        self.inertia.z += slope.z;
        let friction = 0.95;
        scale(&mut self.inertia, friction);

        let mut new_pos = Coord3D::new(pos.x + self.inertia.x, pos.y + self.inertia.y, 0.0);
        new_pos.z = sim.terrain().get_ground_height(new_pos.x, new_pos.y, None);

        let radius = sim
            .object(us)
            .map_or(0.0, |o| o.geometry.get_major_radius());
        // C++ dereferences a null object when nothing is that close; nothing topples here.
        if let Some(tree) = sim.find_closest_object_to_position(&new_pos, radius) {
            if sim.object_is_kind_of(tree, KindOfType::Shrubbery) != FALSE {
                let inertia = self.inertia;
                sim.topple_object(tree, &inertia, inertia.length(), 1);
            }
        }

        // C++ also sums the normalised tensors here and never uses the sum.
        for link in self.links {
            let Some(other) = sim.find_object(link.id) else {
                continue;
            };
            let mut desired = position_of(sim, other);
            let tensor = link.tensor;
            desired.x -= tensor.x;
            desired.y -= tensor.y;
            desired.z -= tensor.z;
            new_pos.x = new_pos.x * 0.93 + desired.x * 0.07;
            new_pos.y = new_pos.y * 0.93 + desired.y * 0.07;
            let ground = sim.terrain().get_ground_height(new_pos.x, new_pos.y, None);
            new_pos.z = if self.lowest_slide_elevation < ground {
                self.lowest_slide_elevation
            } else {
                ground
            };
        }

        // The object has not moved yet: this is its position before the slide.
        let old_z = position_of(sim, us).z;
        sim.set_model_condition_state(us, ModelConditionFlagType::PostCollapse);
        set_flag(sim, us, ModelConditionFlagType::Moving, self.life < 200);
        set_flag(
            sim,
            us,
            ModelConditionFlagType::Freefall,
            gm_fabsf(old_z - new_pos.z) > 0.2 && self.life < 100,
        );

        self.lowest_slide_elevation = new_pos.z;
        sim.set_object_position(us, &new_pos);
        UpdateSleepTime::NONE
    }

    /// `setEnabled`.
    pub fn set_enabled(&mut self, enabled: Bool) {
        self.enabled = enabled;
    }
}

impl UpdateFacet for TensileFormationUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.tick(sim, ctx)
    }
}

impl BehaviorModule for TensileFormationUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.pathfinder_create_wall(ctx.owner_slot);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.enabled)
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

pub fn parse_tensile_formation_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = TensileFormationUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("Enabled") {
            r.parse_bool(&mut data.enabled)
        } else if field.eq_ignore_ascii_case("CrackSound") {
            r.next_token(SEPS_DEFAULT).map(|tok| {
                data.crack_sound = tok.to_owned();
            })
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_tensile_formation_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<TensileFormationUpdateModuleData>()
        .expect("TensileFormationUpdate data type mismatch")
        .clone();
    Box::new(TensileFormationUpdate::new(typed))
}
