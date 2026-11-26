//! `PKT-304`: `AssistedTargetingUpdateModuleData` and `AssistedTargetingUpdate`
//! (`AssistedTargetingUpdate.h`, `AssistedTargetingUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-304_AssistedTargetingUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt336_370_306_304_leaves.rs`.
//!
//! The caller is the assistance request weapon firing makes (`Weapon.cpp`,
//! unported); it reaches this module through `SimApi::request_assist_from`.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot, ThingTemplateId};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    AssistedTargetingFacet, BehaviorModule, CommandSourceType, ModuleData, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::rules::combat_types::WeaponStatus;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

const WEAPON_SLOT_NAMES: [&str; 3] = ["PRIMARY", "SECONDARY", "TERTIARY"];

/// INI template data matching `AssistedTargetingUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct AssistedTargetingUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub clip_size: Int,
    pub weapon_slot: WeaponSlotType,
    pub laser_from_assisted_name: String,
    pub laser_to_target_name: String,
}

impl AssistedTargetingUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            clip_size: 1,
            weapon_slot: WeaponSlotType::Primary,
            laser_from_assisted_name: String::new(),
            laser_to_target_name: String::new(),
        }
    }
}

impl ModuleData for AssistedTargetingUpdateModuleData {
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

/// Runtime instance of `AssistedTargetingUpdate`.
pub struct AssistedTargetingUpdate {
    pub data: AssistedTargetingUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// Looked up at construction; not saved.
    pub laser_from_assisted: Option<ThingTemplateId>,
    pub laser_to_target: Option<ThingTemplateId>,
}

impl AssistedTargetingUpdate {
    pub fn new(data: AssistedTargetingUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            laser_from_assisted: None,
            laser_to_target: None,
        }
    }

    /// `makeFeedbackLaser`: a laser object drawn from `from` to `to`.
    fn make_feedback_laser(
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        template: ThingTemplateId,
        from: ObjectSlot,
        to: ObjectSlot,
    ) {
        let Some((me, team)) = sim.object(owner).map(|o| (o.id, o.team_id)) else {
            return;
        };
        let laser_team = sim.player_default_team(team);
        let laser = sim.spawn_object(template, laser_team, Coord3D::new(0.0, 0.0, 0.0), 0.0);
        let Some(laser_id) = sim.object(laser).map(|o| o.id) else {
            return;
        };
        if sim.object_has_client_update(laser, "LaserUpdate") == FALSE {
            sim.destroy_object(laser);
            return;
        }
        let (Some(start), Some((end_id, end))) = (
            sim.object(from).map(|o| *o.thing.get_position()),
            sim.object(to).map(|o| (o.id, *o.thing.get_position())),
        ) else {
            return;
        };
        sim.set_object_position(laser, &start);
        // `LaserUpdate::initLaser` is client only.
        sim.client().on_laser_init(laser_id, me, end_id, start, end);
    }
}

impl AssistedTargetingFacet for AssistedTargetingUpdate {
    /// `isFreeToAssist`. Asking for the weapon status also stores it.
    fn is_free_to_assist(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> Bool {
        if sim.object_is_able_to_attack(ctx.owner_slot) == FALSE {
            return FALSE;
        }
        let now = sim.frame();
        let Some(obj) = sim.object_mut(ctx.owner_slot) else {
            return FALSE;
        };
        if obj.weapon_set.has_any_weapon() == FALSE {
            return FALSE;
        }
        let slot = obj.weapon_set.get_cur_weapon_slot();
        let status = obj
            .weapon_set
            .get_weapon_in_weapon_slot_mut(slot)
            .map(|weapon| weapon.get_status(now));
        Bool::from(status == Some(WeaponStatus::ReadyToFire))
    }

    /// `assistAttack`.
    fn assist_attack(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        requesting: ObjectSlot,
        victim: ObjectSlot,
    ) {
        let owner = ctx.owner_slot;
        if sim.has_ai(owner) == FALSE {
            return;
        }
        // Make me attack with my special slot weapon.
        let _ = sim.lock_weapon_slot(
            owner,
            self.data.weapon_slot,
            WeaponLockType::LockedTemporarily,
        );
        sim.ai_attack_object(
            owner,
            victim,
            self.data.clip_size,
            CommandSourceType::FromAi,
        );

        if let Some(template) = self.laser_from_assisted {
            Self::make_feedback_laser(sim, owner, template, requesting, owner);
        }
        if let Some(template) = self.laser_to_target {
            Self::make_feedback_laser(sim, owner, template, owner, victim);
        }
    }
}

impl UpdateFacet for AssistedTargetingUpdate {
    fn update(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) -> UpdateSleepTime {
        UpdateSleepTime::FOREVER
    }
}

impl BehaviorModule for AssistedTargetingUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        let factory = &sim.rules().thing_factory;
        let find = |name: &str| {
            if name.is_empty() {
                None
            } else {
                factory.find_template_id(name)
            }
        };
        self.laser_from_assisted = find(&self.data.laser_from_assisted_name);
        self.laser_to_target = find(&self.data.laser_to_target_name);
        #[cfg(not(feature = "retail-crc"))]
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)
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

    fn as_assisted_targeting_mut(&mut self) -> Option<&mut dyn AssistedTargetingFacet> {
        Some(self)
    }
}

pub fn parse_assisted_targeting_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = AssistedTargetingUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("AssistingClipSize") {
            r.parse_int(&mut data.clip_size)
        } else if is("AssistingWeaponSlot") {
            let mut index = 0;
            r.parse_index_list(&mut index, &WEAPON_SLOT_NAMES)?;
            data.weapon_slot = match index {
                0 => WeaponSlotType::Primary,
                1 => WeaponSlotType::Secondary,
                _ => WeaponSlotType::Tertiary,
            };
            Ok(())
        } else if is("LaserFromAssisted") {
            r.parse_ascii_string(&mut data.laser_from_assisted_name)
        } else if is("LaserToTarget") {
            r.parse_ascii_string(&mut data.laser_to_target_name)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_assisted_targeting_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<AssistedTargetingUpdateModuleData>()
        .expect("AssistedTargetingUpdate data type mismatch")
        .clone();
    Box::new(AssistedTargetingUpdate::new(typed))
}
