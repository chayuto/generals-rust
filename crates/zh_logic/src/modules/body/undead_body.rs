//! `PKT-191`: `UndeadBody` (`UndeadBody.h` / `UndeadBody.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar};
use crate::combat::damage::DamageInfo;
use crate::delegate_body_facet_to_inner;
use crate::modules::body::active_body::{ActiveBody, ActiveBodyModuleData};
use crate::modules::module::{BehaviorModule, BodyFacet, MaxHealthChangeType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{is_health_damaging_damage, ArmorSetType, DamageType};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Real, UnsignedInt, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `UndeadBody`.
#[derive(Debug, Clone, PartialEq)]
pub struct UndeadBodyModuleData {
    pub base: ActiveBodyModuleData,
    pub second_life_max_health: Real,
}

impl UndeadBodyModuleData {
    pub const fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            base: ActiveBodyModuleData::new(tag_key, module_name_key),
            second_life_max_health: 1.0,
        }
    }
}

impl ModuleData for UndeadBodyModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.base.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::BODY
    }

    fn subdual_damage_heal_rate(&self) -> UnsignedInt {
        self.base.subdual_damage_heal_rate
    }

    fn subdual_damage_heal_amount(&self) -> Real {
        self.base.subdual_damage_heal_amount
    }
}

pub fn parse_undead_body_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = UndeadBodyModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if data.base.parse_field(field, reader)? {
            Ok(())
        } else if field.eq_ignore_ascii_case("SecondLifeMaxHealth") {
            reader.parse_real(&mut data.second_life_max_health)?;
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

/// Body that intercepts lethal first-life damage, converts into a second life via `SlowDeathBehavior`,
/// and applies second-life armor/health (`UndeadBody.h` / `UndeadBody.cpp`).
#[derive(Debug, Clone)]
pub struct UndeadBody {
    pub inner: ActiveBody,
    pub second_life_max_health: Real,
    pub is_second_life: Bool,
}

impl UndeadBody {
    pub fn from_module_data(module_name_key: NameKeyType, data: &UndeadBodyModuleData) -> Self {
        Self {
            inner: ActiveBody::from_active_data(module_name_key, &data.base, false),
            second_life_max_health: data.second_life_max_health,
            is_second_life: FALSE,
        }
    }

    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let ubd = data
            .as_any()
            .downcast_ref::<UndeadBodyModuleData>()
            .expect("UndeadBodyModuleData");
        Box::new(Self::from_module_data(data.module_name_key(), ubd))
    }

    pub fn start_second_life(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.is_second_life = TRUE;
        self.inner.set_max_health(
            sim,
            ctx,
            self.second_life_max_health,
            MaxHealthChangeType::FullyHeal,
        );
        self.inner.set_armor_set_flag(ArmorSetType::SecondLife);
        let sub = self.inner.is_subdued(sim, ctx);
        sim.set_module_sidecar(ctx.mod_slot, self.inner.compute_sidecar(sub));
        sim.trigger_slow_death(ctx.owner_slot, dmg);
    }
}

impl BehaviorModule for UndeadBody {
    fn module_name_key(&self) -> NameKeyType {
        self.inner.module_name_key()
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.inner.on_object_created(sim, ctx);
    }

    fn apply_deferred_op(&mut self, op: DeferredSelfOp, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.inner.apply_deferred_op(op, sim, ctx);
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        self.inner.sync_sidecar()
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.inner.xfer(xfer, sim)?;
        xfer.xfer_bool(&mut self.is_second_life)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.inner.load_post_process(sim, ctx);
    }

    fn as_body(&self) -> Option<&dyn BodyFacet> {
        Some(self)
    }

    fn as_body_mut(&mut self) -> Option<&mut dyn BodyFacet> {
        Some(self)
    }
}

impl BodyFacet for UndeadBody {
    fn attempt_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let mut should_start_second_life = false;
        let cur_health = self.inner.get_health();
        #[cfg(feature = "retail-crc")]
        let lethal_threshold = dmg.input.amount >= cur_health;
        #[cfg(not(feature = "retail-crc"))]
        let lethal_threshold = self.inner.estimate_damage(sim, ctx, dmg) >= cur_health;

        if dmg.input.damage_type != DamageType::Unresistable
            && self.is_second_life == FALSE
            && lethal_threshold
            && is_health_damaging_damage(dmg.input.damage_type.as_i32())
        {
            let cap = cur_health - 1.0;
            if cap < dmg.input.amount {
                dmg.input.amount = cap;
            }
            should_start_second_life = true;
        }

        self.inner.attempt_damage(sim, ctx, dmg);

        if should_start_second_life {
            self.start_second_life(sim, ctx, dmg);
        }
    }

    delegate_body_facet_to_inner!(inner);
}
