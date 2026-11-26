//! `PKT-388`: `WeaponBonusUpdate`
//! (`WeaponBonusUpdate.h`, `WeaponBonusUpdate.cpp`).
//!
//! Golden test: `crates/zh_logic/tests/golden_pkt388.rs`.
//!
//! Without a `BonusConditionType` the reference shifts by -1, which is undefined; here
//! nothing is granted.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::gamelogic::global_data::{WeaponBonusConditionType, THE_WEAPON_BONUS_NAMES};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::sim::{SimApi, SimView, PRIVATE_STATUS_OFF_MAP};
use std::any::Any;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// `WEAPONBONUSCONDITION_INVALID`.
const CONDITION_INVALID: Int = -1;

/// INI template data matching `WeaponBonusUpdateModuleData`.
#[derive(Debug, Clone)]
pub struct WeaponBonusUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub required_affect_kind_of: KindOfMaskType,
    pub forbidden_affect_kind_of: KindOfMaskType,
    pub bonus_duration: UnsignedInt,
    pub bonus_delay: UnsignedInt,
    pub bonus_range: Real,
    pub bonus_condition_type: Int,
}

impl WeaponBonusUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            required_affect_kind_of: KindOfMaskType::new(),
            forbidden_affect_kind_of: KindOfMaskType::new(),
            bonus_duration: 0,
            bonus_delay: 0,
            bonus_range: 0.0,
            bonus_condition_type: CONDITION_INVALID,
        }
    }
}

impl ModuleData for WeaponBonusUpdateModuleData {
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

/// Runtime instance of `WeaponBonusUpdate` (`PKT-388`).
pub struct WeaponBonusUpdate {
    pub data: WeaponBonusUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
}

impl WeaponBonusUpdate {
    pub fn new(data: WeaponBonusUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    /// `doTempWeaponBonus` on `obj`, if it passes the kind-of test.
    fn grant(&self, sim: &mut dyn SimApi, obj: ObjectSlot) {
        let wanted = sim.object_is_kind_of_multi(
            obj,
            &self.data.required_affect_kind_of,
            &self.data.forbidden_affect_kind_of,
        );
        if wanted == FALSE {
            return;
        }
        if let Some(bonus) = WeaponBonusConditionType::from_i32(self.data.bonus_condition_type) {
            sim.trigger_temp_weapon_bonus(obj, bonus, self.data.bonus_duration);
        }
    }
}

/// `Object::getRelationship(that) == ALLIES`, including undetected defectors.
fn is_ally(sim: &dyn SimView, me: ObjectSlot, that: ObjectSlot) -> bool {
    let defector = |slot| {
        sim.object(slot)
            .is_some_and(|o| o.is_undetected_defector() != FALSE)
    };
    if defector(me) {
        false
    } else {
        defector(that) || sim.player_relationship(me, that) == Relationship::Allies
    }
}

impl UpdateFacet for WeaponBonusUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let me = ctx.owner_slot;
        let sleep = UpdateSleepTime(self.data.bonus_delay);
        let Some((center, my_off_map)) = sim.object(me).map(|o| {
            (
                *o.thing.get_position(),
                o.private_status & PRIVATE_STATUS_OFF_MAP,
            )
        }) else {
            return sleep;
        };

        // The kind-of test is not a search filter: riders of a rejected container still count.
        for obj in sim.find_objects_in_range(&center, self.data.bonus_range) {
            if !is_ally(sim, me, obj) {
                continue;
            }
            let passes = sim.object(obj).is_some_and(|o| {
                o.is_effectively_dead() == FALSE
                    && o.private_status & PRIVATE_STATUS_OFF_MAP == my_off_map
            });
            if !passes {
                continue;
            }
            self.grant(sim, obj);
            for id in sim.contain_items(obj) {
                if let Some(rider) = sim.find_object(id) {
                    self.grant(sim, rider);
                }
            }
        }
        sleep
    }
}

impl BehaviorModule for WeaponBonusUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)
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

pub fn create_weapon_bonus_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<WeaponBonusUpdateModuleData>()
        .expect("WeaponBonusUpdate data type mismatch")
        .clone();
    Box::new(WeaponBonusUpdate::new(typed))
}

pub fn parse_weapon_bonus_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = WeaponBonusUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("RequiredAffectKindOf") {
            data.required_affect_kind_of
                .parse_from_ini(r, &KINDOF_NAMES)
        } else if field.eq_ignore_ascii_case("ForbiddenAffectKindOf") {
            data.forbidden_affect_kind_of
                .parse_from_ini(r, &KINDOF_NAMES)
        } else if field.eq_ignore_ascii_case("BonusDuration") {
            r.parse_duration_unsigned_int(&mut data.bonus_duration)
        } else if field.eq_ignore_ascii_case("BonusDelay") {
            r.parse_duration_unsigned_int(&mut data.bonus_delay)
        } else if field.eq_ignore_ascii_case("BonusRange") {
            r.parse_real(&mut data.bonus_range)
        } else if field.eq_ignore_ascii_case("BonusConditionType") {
            r.parse_index_list(&mut data.bonus_condition_type, &THE_WEAPON_BONUS_NAMES)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}
