//! `PKT-289`: `PrisonBehavior` (`PrisonBehavior.h`, `PrisonBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-289_PrisonBehavior.md`.
//! Test: `crates/zh_logic/tests/golden_wp53_b.rs` (contract tests only: the class is compiled
//! out of the reference library behind `ALLOW_SURRENDER`, so there are no golden vectors).
//!
//! Registered only by [`super::register_surrender_behavior_modules`]. Prisoner drawables are
//! client objects: the visual list keeps object ids with invalid drawable ids, but the
//! orientation PRNG draw each visual makes is preserved.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    impl_contain_module, remove_from_contain, ContainBehavior, OpenContainCore,
    OpenContainModuleData,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::DisabledType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::TWO_PI;
use zh_common::types::{
    Bool, NameKeyType, ObjectId, UnsignedInt, UnsignedShort, FALSE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

const INVALID_DRAWABLE_ID: UnsignedInt = 0;

/// INI template data matching `PrisonBehaviorModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct PrisonBehaviorModuleData {
    pub open: OpenContainModuleData,
    /// Show the contained prisoners in the prison yard.
    pub show_prisoners: Bool,
    pub prison_yard_bone_prefix: String,
}

impl PrisonBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            open: OpenContainModuleData::new(tag_key, module_name_key),
            show_prisoners: FALSE,
            prison_yard_bone_prefix: String::new(),
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("ShowPrisoners") {
            Some(reader.parse_bool(&mut self.show_prisoners))
        } else if field.eq_ignore_ascii_case("YardBonePrefix") {
            Some(reader.parse_ascii_string(&mut self.prison_yard_bone_prefix))
        } else {
            self.open.parse_field(field, reader, parse_ctx)
        }
    }
}

impl ModuleData for PrisonBehaviorModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.open.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.open.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// `PrisonVisual`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrisonVisual {
    pub object_id: ObjectId,
    pub drawable_id: UnsignedInt,
}

/// Runtime instance of `PrisonBehavior` (`PKT-289`).
pub struct PrisonBehavior {
    pub data: PrisonBehaviorModuleData,
    pub core: OpenContainCore,
    /// Head of the C++ linked list is the front of this vector.
    pub visuals: Vec<PrisonVisual>,
}

impl PrisonBehavior {
    pub fn new(data: PrisonBehaviorModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        Self {
            data,
            core,
            visuals: Vec::new(),
        }
    }

    fn add_visual(&mut self, sim: &mut dyn SimApi, obj: ObjectSlot) {
        let Some(object_id) = sim.object(obj).map(|o| o.id) else {
            return;
        };
        // No yard bones headless, so `pickVisualLocation` draws nothing.
        let _orientation = sim.rng_logic_real(0.0, TWO_PI);
        self.visuals.insert(
            0,
            PrisonVisual {
                object_id,
                drawable_id: INVALID_DRAWABLE_ID,
            },
        );
    }

    fn remove_visual(&mut self, sim: &dyn SimView, obj: ObjectSlot) {
        let Some(object_id) = sim.object(obj).map(|o| o.id) else {
            return;
        };
        if let Some(index) = self.visuals.iter().position(|v| v.object_id == object_id) {
            self.visuals.remove(index);
        }
    }
}

impl ContainBehavior for PrisonBehavior {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }

    fn contained(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, rider: ObjectSlot) {
        sim.set_disabled(rider, DisabledType::Held);
        if self.data.show_prisoners != FALSE {
            self.add_visual(sim, rider);
        }
    }

    fn removing(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, rider: ObjectSlot) {
        if self.data.show_prisoners != FALSE {
            self.remove_visual(sim, rider);
        }
        sim.clear_disabled(rider, DisabledType::Held);
    }

    fn delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for id in self.core.contain_list.clone() {
            if let Some(rider) = sim.find_object(id) {
                sim.destroy_object(rider);
                remove_from_contain(self, sim, ctx, rider, FALSE);
            }
        }
        self.visuals.clear();
    }

    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)?;

        let mut count = self.visuals.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Load {
            if !self.visuals.is_empty() {
                return Err(XferError::ListNotEmpty);
            }
            for _ in 0..count {
                let mut visual = PrisonVisual {
                    object_id: ObjectId::INVALID,
                    drawable_id: INVALID_DRAWABLE_ID,
                };
                xfer.xfer_object_id(&mut visual.object_id)?;
                xfer.xfer_unsigned_int(&mut visual.drawable_id)?;
                self.visuals.insert(0, visual);
            }
        } else {
            for visual in &mut self.visuals {
                xfer.xfer_object_id(&mut visual.object_id)?;
                xfer.xfer_unsigned_int(&mut visual.drawable_id)?;
            }
        }
        Ok(())
    }
}

impl_contain_module!(PrisonBehavior);

pub fn parse_prison_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = PrisonBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_prison_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<PrisonBehaviorModuleData>()
        .expect("PrisonBehavior data type mismatch")
        .clone();
    Box::new(PrisonBehavior::new(typed))
}
