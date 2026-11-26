//! `PKT-329`: `FireWeaponUpdate`, which fires its weapon at its owner's position whenever ready.
//!
//! Packet: `packets/phase5/PKT-329_FireWeaponUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp55_fire.rs`.

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon::{Weapon, WeaponTemplate};
use crate::object::weapon_fire;
use crate::object::weapon_set::WeaponSlotType;
use crate::rules::combat_types::WeaponStatus;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, NameKeyType, UnsignedInt, UpdateSleepTime, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

#[derive(Debug, Clone)]
pub struct FireWeaponUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub weapon_template: Option<WeaponTemplate>,
    pub initial_delay_frames: UnsignedInt,
    /// If non-zero, any weapon of the owner having fired this recently keeps the module quiet.
    pub exclusive_weapon_delay: UnsignedInt,
}

impl FireWeaponUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            weapon_template: None,
            initial_delay_frames: 0,
            exclusive_weapon_delay: 0,
        }
    }
}

impl ModuleData for FireWeaponUpdateModuleData {
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

/// Runtime instance of `FireWeaponUpdate`.
pub struct FireWeaponUpdate {
    pub data: FireWeaponUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub weapon: Option<Weapon>,
    pub initial_delay_frame: UnsignedInt,
}

impl FireWeaponUpdate {
    pub fn new(data: FireWeaponUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            weapon: None,
            initial_delay_frame: 0,
        }
    }

    /// `isOkayToFire`.
    fn is_okay_to_fire(&mut self, sim: &dyn SimApi, ctx: ModuleContext) -> Bool {
        let now = sim.frame();
        let Some(weapon) = self.weapon.as_mut() else {
            return FALSE;
        };
        if weapon.get_status(now) != WeaponStatus::ReadyToFire {
            return FALSE;
        }
        let building = sim.object(ctx.owner_slot).is_some_and(|o| {
            o.status.test(ObjectStatusType::UnderConstruction as Int) != FALSE
        });
        if building {
            return FALSE; // no hitting with a 0% building
        }
        let delay = self.data.exclusive_weapon_delay;
        if delay > 0 && now < sim.last_shot_fired_frame(ctx.owner_slot).wrapping_add(delay) {
            return FALSE;
        }
        TRUE
    }
}

impl UpdateFacet for FireWeaponUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if sim.frame() < self.initial_delay_frame {
            return UpdateSleepTime::NONE;
        }
        if self.is_okay_to_fire(sim, ctx) != FALSE {
            let pos = sim
                .object(ctx.owner_slot)
                .map(|o| *o.thing.get_position())
                .unwrap_or_default();
            if let Some(weapon) = self.weapon.as_mut() {
                weapon_fire::force_fire_weapon(weapon, sim, ctx.owner_slot, &pos);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for FireWeaponUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    /// C++ constructor: the weapon starts loaded.
    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        if let Some(template) = self.data.weapon_template.as_ref() {
            let mut weapon = Weapon::new(template.clone(), WeaponSlotType::Primary, sim.frame());
            weapon_fire::load_ammo_now(&mut weapon, sim, ctx.owner_slot);
            self.weapon = Some(weapon);
        }
        self.initial_delay_frame = sim.frame().wrapping_add(self.data.initial_delay_frames);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        // C++ requires a weapon here; without a template there is nothing to serialize.
        if let Some(weapon) = self.weapon.as_mut() {
            weapon.xfer(xfer)?;
            if xfer.mode() == XferMode::Load {
                if let Some(t) = sim.rules().weapon_store.find_weapon_template(&weapon.template_name)
                {
                    weapon.template = t.clone();
                }
            }
        }
        if version >= 2 {
            xfer.xfer_unsigned_int(&mut self.initial_delay_frame)?;
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
}

pub fn parse_fire_weapon_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = FireWeaponUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("Weapon") {
            let token = r.next_token(SEPS_DEFAULT)?;
            data.weapon_template = parse_ctx.weapon_store.find_weapon_template(token).cloned();
            Ok(())
        } else if field.eq_ignore_ascii_case("InitialDelay") {
            r.parse_duration_unsigned_int(&mut data.initial_delay_frames)
        } else if field.eq_ignore_ascii_case("ExclusiveWeaponDelay") {
            r.parse_duration_unsigned_int(&mut data.exclusive_weapon_delay)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_fire_weapon_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<FireWeaponUpdateModuleData>()
        .expect("FireWeaponUpdate data type mismatch")
        .clone();
    Box::new(FireWeaponUpdate::new(typed))
}
