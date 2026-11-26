//! `PKT-261`: `BridgeScaffoldBehavior` (`BridgeScaffoldBehavior.h`, `BridgeScaffoldBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-261_BridgeScaffoldBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_c1.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, BridgeScaffoldFacet, ModuleData, ScaffoldTargetMotion, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, UpdateSleepTime};
use zh_common::xfer::{Xfer, XferError};

/// Template data (`UpdateModuleData`, no fields of its own).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeScaffoldBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
}

impl ModuleData for BridgeScaffoldBehaviorModuleData {
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
        FacetMask(0x001 | FacetMask::BRIDGE_SCAFFOLD.0)
    }
}

/// Runtime instance of `BridgeScaffoldBehavior` (`PKT-261`).
pub struct BridgeScaffoldBehavior {
    pub data: BridgeScaffoldBehaviorModuleData,
    pub target_motion: ScaffoldTargetMotion,
    pub create_pos: Coord3D,
    pub rise_to_pos: Coord3D,
    pub build_pos: Coord3D,
    pub lateral_speed: Real,
    pub vertical_speed: Real,
    pub target_pos: Coord3D,
    pub schedule: UpdateScheduleTracker,
}

impl BridgeScaffoldBehavior {
    pub fn new(data: BridgeScaffoldBehaviorModuleData) -> Self {
        Self {
            data,
            target_motion: ScaffoldTargetMotion::Still,
            create_pos: Coord3D::default(),
            rise_to_pos: Coord3D::default(),
            build_pos: Coord3D::default(),
            lateral_speed: 1.0,
            vertical_speed: 1.0,
            target_pos: Coord3D::default(),
            schedule: UpdateScheduleTracker::new(),
        }
    }
}

fn sub(a: &Coord3D, b: &Coord3D) -> Coord3D {
    Coord3D {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

impl BridgeScaffoldFacet for BridgeScaffoldBehavior {
    fn set_positions(&mut self, create_pos: &Coord3D, rise_pos: &Coord3D, build_pos: &Coord3D) {
        self.create_pos = *create_pos;
        self.rise_to_pos = *rise_pos;
        self.build_pos = *build_pos;
    }

    fn set_motion(&mut self, target_motion: ScaffoldTargetMotion) {
        self.target_motion = target_motion;
        match target_motion {
            ScaffoldTargetMotion::Rise | ScaffoldTargetMotion::TearDownAcross => {
                self.target_pos = self.rise_to_pos;
            }
            ScaffoldTargetMotion::BuildAcross => self.target_pos = self.build_pos,
            ScaffoldTargetMotion::Sink => self.target_pos = self.create_pos,
            ScaffoldTargetMotion::Still => {}
        }
    }

    fn get_current_motion(&self) -> ScaffoldTargetMotion {
        self.target_motion
    }

    fn reverse_motion(&mut self) {
        let next = match self.target_motion {
            ScaffoldTargetMotion::Still => ScaffoldTargetMotion::TearDownAcross,
            ScaffoldTargetMotion::Rise => ScaffoldTargetMotion::Sink,
            ScaffoldTargetMotion::BuildAcross => ScaffoldTargetMotion::TearDownAcross,
            ScaffoldTargetMotion::TearDownAcross => ScaffoldTargetMotion::BuildAcross,
            ScaffoldTargetMotion::Sink => ScaffoldTargetMotion::Rise,
        };
        self.set_motion(next);
    }

    fn set_lateral_speed(&mut self, lateral_speed: Real) {
        self.lateral_speed = lateral_speed;
    }

    fn set_vertical_speed(&mut self, vertical_speed: Real) {
        self.vertical_speed = vertical_speed;
    }
}

impl UpdateFacet for BridgeScaffoldBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.target_motion == ScaffoldTargetMotion::Still {
            return UpdateSleepTime::NONE;
        }
        let owner = ctx.owner_slot;
        let Some(our_pos) = sim.object(owner).map(|o| *o.thing.get_position()) else {
            return UpdateSleepTime::NONE;
        };

        let dir_v = sub(&self.target_pos, &our_pos);
        let mut v = dir_v;
        v.normalize();

        let (top_speed, start, end) = match self.target_motion {
            ScaffoldTargetMotion::Rise => (self.vertical_speed, self.create_pos, self.rise_to_pos),
            ScaffoldTargetMotion::Sink => (self.vertical_speed, self.rise_to_pos, self.create_pos),
            ScaffoldTargetMotion::BuildAcross => {
                (self.lateral_speed, self.rise_to_pos, self.build_pos)
            }
            ScaffoldTargetMotion::TearDownAcross => {
                (self.lateral_speed, self.build_pos, self.rise_to_pos)
            }
            ScaffoldTargetMotion::Still => return UpdateSleepTime::NONE,
        };

        // Slower at the end of the motion.
        let total_distance = sub(&end, &start).length() * 0.25;
        let our_distance = sub(&end, &our_pos).length();
        let mut speed = (our_distance / total_distance) * top_speed;
        let min_speed = top_speed * 0.08;
        if speed < min_speed {
            speed = min_speed;
        }
        if speed > top_speed {
            speed = top_speed;
        }
        if speed < 0.001 {
            speed = 0.001;
        }

        let mut new_pos = Coord3D {
            x: v.x * speed + our_pos.x,
            y: v.y * speed + our_pos.y,
            z: v.z * speed + our_pos.z,
        };

        let too_far = sub(&self.target_pos, &new_pos);
        if too_far.x * dir_v.x + too_far.y * dir_v.y + too_far.z * dir_v.z <= 0.0 {
            new_pos = self.target_pos;
            match self.target_motion {
                ScaffoldTargetMotion::Rise => self.set_motion(ScaffoldTargetMotion::BuildAcross),
                ScaffoldTargetMotion::BuildAcross => self.set_motion(ScaffoldTargetMotion::Still),
                ScaffoldTargetMotion::TearDownAcross => self.set_motion(ScaffoldTargetMotion::Sink),
                ScaffoldTargetMotion::Sink => sim.destroy_object(owner),
                ScaffoldTargetMotion::Still => {}
            }
        }

        sim.set_object_position(owner, &new_pos);
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for BridgeScaffoldBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        // `xferUser` of the four-byte enum.
        let mut motion = (self.target_motion as i32).to_le_bytes();
        xfer.xfer_user(&mut motion)?;
        self.target_motion = match i32::from_le_bytes(motion) {
            1 => ScaffoldTargetMotion::Rise,
            2 => ScaffoldTargetMotion::BuildAcross,
            3 => ScaffoldTargetMotion::TearDownAcross,
            4 => ScaffoldTargetMotion::Sink,
            _ => ScaffoldTargetMotion::Still,
        };
        xfer.xfer_coord3d(&mut self.create_pos)?;
        xfer.xfer_coord3d(&mut self.rise_to_pos)?;
        xfer.xfer_coord3d(&mut self.build_pos)?;
        xfer.xfer_real(&mut self.lateral_speed)?;
        xfer.xfer_real(&mut self.vertical_speed)?;
        xfer.xfer_coord3d(&mut self.target_pos)
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

    fn as_bridge_scaffold(&self) -> Option<&dyn BridgeScaffoldFacet> {
        Some(self)
    }

    fn as_bridge_scaffold_mut(&mut self) -> Option<&mut dyn BridgeScaffoldFacet> {
        Some(self)
    }
}

pub fn parse_bridge_scaffold_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    reader.read_block_fields(|_field, _r| Err(IniError::UnknownToken))?;
    Ok(Box::new(BridgeScaffoldBehaviorModuleData {
        tag_key,
        module_name_key,
    }))
}

pub fn create_bridge_scaffold_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BridgeScaffoldBehaviorModuleData>()
        .expect("BridgeScaffoldBehavior data type mismatch")
        .clone();
    Box::new(BridgeScaffoldBehavior::new(typed))
}
