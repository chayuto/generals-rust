//! `PKT-189`: `TempWeaponBonusHelper` (`TempWeaponBonusHelper.h` / `TempWeaponBonusHelper.cpp`).

use crate::arena::ModuleContext;
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::helper::object_helper::ObjectHelperBase;
use crate::modules::module::{BehaviorModule, HelperFacet, UpdateFacet};
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType};
use crate::scheduler::SleepyUpdatePhase;
use crate::sim::{SimApi, SimView};
use zh_common::types::{Int, NameKeyType, UnsignedInt, UpdateSleepTime, XferVersion};
use zh_common::xfer::{Xfer, XferError};

/// `WEAPONBONUSCONDITION_INVALID = -1` in `TempWeaponBonusHelper.cpp`.
pub const WEAPON_BONUS_CONDITION_INVALID: Int = -1;

#[derive(Debug, Clone)]
pub struct TempWeaponBonusHelper {
    pub base: ObjectHelperBase,
    pub current_bonus: Int,
    pub frame_to_remove: UnsignedInt,
}

impl TempWeaponBonusHelper {
    #[inline]
    pub fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: ObjectHelperBase::new(module_name_key, SleepyUpdatePhase::Normal),
            current_bonus: WEAPON_BONUS_CONDITION_INVALID,
            frame_to_remove: 0,
        }
    }

    fn clear_temp_weapon_bonus(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.current_bonus != WEAPON_BONUS_CONDITION_INVALID {
            if let Some(cond) = WeaponBonusConditionType::from_i32(self.current_bonus) {
                sim.clear_weapon_bonus_condition(ctx.owner_slot, cond);
            }
            self.current_bonus = WEAPON_BONUS_CONDITION_INVALID;
            self.frame_to_remove = 0;
        }
    }
}

impl BehaviorModule for TempWeaponBonusHelper {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.bind_slot(ctx);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer_base(xfer, sim)?;
        xfer.xfer_int(&mut self.current_bonus)?;
        xfer.xfer_unsigned_int(&mut self.frame_to_remove)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.load_post_process_base(sim, ctx);
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_helper(&self) -> Option<&dyn HelperFacet> {
        Some(self)
    }

    fn as_helper_mut(&mut self) -> Option<&mut dyn HelperFacet> {
        Some(self)
    }
}

impl UpdateFacet for TempWeaponBonusHelper {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.base.bind_slot(ctx);
        self.clear_temp_weapon_bonus(sim, ctx);
        UpdateSleepTime::FOREVER
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

impl HelperFacet for TempWeaponBonusHelper {
    fn do_temp_weapon_bonus(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        bonus_type: WeaponBonusConditionType,
        duration: UnsignedInt,
    ) {
        self.base.bind_slot(ctx);
        let target_raw = bonus_type.as_i32();
        if self.current_bonus != target_raw {
            self.clear_temp_weapon_bonus(sim, ctx);
        }
        sim.set_weapon_bonus_condition(ctx.owner_slot, bonus_type);
        self.current_bonus = target_raw;
        self.frame_to_remove = sim.frame().saturating_add(duration);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(duration.max(1)));
    }
}
