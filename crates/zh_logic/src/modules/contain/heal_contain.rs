//! `PKT-274`: `HealContain` (`HealContain.h`, `HealContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-274_HealContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    impl_contain_module, update_open_contain, ContainBehavior, OpenContainCore,
    OpenContainModuleData,
};
use crate::modules::module::{BehaviorModule, ExitDoorType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{
    Bool, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `HealContainModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct HealContainModuleData {
    pub open: OpenContainModuleData,
    /// Frames until something becomes fully healed.
    pub frames_for_full_heal: UnsignedInt,
}

impl HealContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            open: OpenContainModuleData::new(tag_key, module_name_key),
            frames_for_full_heal: 0,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("TimeForFullHeal") {
            Some(reader.parse_duration_unsigned_int(&mut self.frames_for_full_heal))
        } else {
            self.open.parse_field(field, reader, parse_ctx)
        }
    }
}

impl ModuleData for HealContainModuleData {
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

/// Runtime instance of `HealContain` (`PKT-274`).
pub struct HealContain {
    pub data: HealContainModuleData,
    pub core: OpenContainCore,
}

impl HealContain {
    pub fn new(data: HealContainModuleData) -> Self {
        let core = OpenContainCore::new(data.open.passengers_allowed_to_fire);
        Self { data, core }
    }

    /// `HealContain::doHeal`: returns `TRUE` once the object is done healing.
    fn do_heal(sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot, frames: UnsignedInt) -> Bool {
        let source = sim.object(ctx.owner_slot).map(|o| o.id).unwrap_or(ctx.owner_id);
        let Some(contained_by_frame) = sim.object(obj).map(|o| o.contained_by_frame) else {
            return FALSE;
        };
        let max_health = sim.object_max_health(obj);

        let mut heal = DamageInfo::default();
        heal.input.damage_type = DamageType::Healing;
        heal.input.death_type = DeathType::NoneType;
        heal.input.source_id = source;

        let done = if sim.frame().wrapping_sub(contained_by_frame) >= frames {
            heal.input.amount = max_health;
            TRUE
        } else {
            // As if starting from zero health and fully healed after `frames`.
            heal.input.amount = max_health / (frames as Real);
            FALSE
        };
        sim.attempt_healing(obj, &mut heal);
        done
    }
}

impl ContainBehavior for HealContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }
    fn is_heal_container(&self) -> Bool {
        TRUE
    }
    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.core.xfer(xfer, sim)
    }
    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let _ = update_open_contain(self, sim, ctx);

        let frames = self.data.frames_for_full_heal;
        for id in self.core.contain_list.clone() {
            let Some(obj) = sim.find_object(id) else {
                continue;
            };
            if Self::do_heal(sim, ctx, obj, frames) != FALSE {
                let door = self.reserve_door(sim, ctx, Some(obj));
                if door != ExitDoorType::DoorNoneAvailable {
                    self.exit_via_door(sim, ctx, obj, door);
                }
            }
        }
        UpdateSleepTime::NONE
    }
}

impl_contain_module!(HealContain);

pub fn parse_heal_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = HealContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_heal_contain(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<HealContainModuleData>()
        .expect("HealContain data type mismatch")
        .clone();
    Box::new(HealContain::new(typed))
}
