//! `PKT-175`: `InactiveBody` (`InactiveBody.h` / `InactiveBody.cpp`).

use crate::arena::{DeferredSelfOp, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::body::body_module::{BodyModuleBase, BodyModuleData};
use crate::modules::module::{BehaviorModule, BodyFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::DamageType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Parses empty INI block for `InactiveBody` (`BodyModuleData` only).
pub fn parse_inactive_body_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    ini.read_block_fields(|_, _| Err(IniError::UnknownToken))?;
    Ok(Box::new(BodyModuleData {
        tag_key,
        module_name_key,
    }))
}

/// Non-damageable body with zero health that only responds to `DamageType::Unresistable` death
/// (`InactiveBody.h` / `InactiveBody.cpp`).
#[derive(Debug, Clone)]
pub struct InactiveBody {
    pub base: BodyModuleBase,
    pub die_called: Bool,
}

impl InactiveBody {
    #[inline]
    pub const fn new(module_name_key: NameKeyType) -> Self {
        Self {
            base: BodyModuleBase::new(module_name_key),
            die_called: FALSE,
        }
    }

    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        Box::new(Self::new(data.module_name_key()))
    }
}

impl BehaviorModule for InactiveBody {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(obj) = sim.object_mut(ctx.owner_slot) {
            obj.set_effectively_dead(TRUE);
        }
    }

    fn apply_deferred_op(
        &mut self,
        op: DeferredSelfOp,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
    ) {
        if let DeferredSelfOp::ApplyDamageScalar(scalar) = op {
            self.base.apply_damage_scalar(scalar);
        }
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer_base(xfer)
    }

    fn load_post_process(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl BodyFacet for InactiveBody {
    fn estimate_damage(&self, _sim: &dyn SimView, _ctx: ModuleContext, dmg: &DamageInfo) -> Real {
        if dmg.input.damage_type == DamageType::Unresistable {
            dmg.input.amount
        } else {
            0.0
        }
    }

    fn attempt_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        if dmg.input.damage_type == DamageType::Healing {
            self.attempt_healing(sim, ctx, dmg);
            return;
        }
        dmg.output.actual_damage_dealt = 0.0;
        dmg.output.actual_damage_clipped = 0.0;
        dmg.output.no_effect = TRUE;

        if dmg.input.damage_type == DamageType::Unresistable {
            dmg.output.no_effect = FALSE;
            if self.die_called == FALSE {
                sim.dispatch_on_die(ctx.owner_slot, dmg);
                self.die_called = TRUE;
            }
        }
    }

    fn attempt_healing(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        healing: &mut DamageInfo,
    ) {
        if healing.input.damage_type != DamageType::Healing {
            self.attempt_damage(sim, ctx, healing);
            return;
        }
        healing.output.actual_damage_dealt = 0.0;
        healing.output.actual_damage_clipped = 0.0;
        healing.output.no_effect = TRUE;
    }

    fn internal_change_health(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _delta: Real,
    ) {
    }

    fn get_health(&self) -> Real {
        0.0
    }

    fn get_damage_state(&self) -> BodyDamageType {
        BodyDamageType::Pristine
    }

    fn set_damage_state(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _new_state: BodyDamageType,
    ) {
    }

    fn get_damage_scalar(&self) -> Real {
        self.base.get_damage_scalar()
    }

    fn apply_damage_scalar(&mut self, scalar: Real) {
        self.base.apply_damage_scalar(scalar);
    }

    fn is_indestructible(&self) -> Bool {
        TRUE
    }

    fn get_last_damage_timestamp(&self) -> Option<UnsignedInt> {
        BodyModuleBase::get_last_damage_timestamp()
    }

    fn get_last_healing_timestamp(&self) -> Option<UnsignedInt> {
        BodyModuleBase::get_last_healing_timestamp()
    }

    fn is_inactive_body(&self) -> Bool {
        TRUE
    }
}
