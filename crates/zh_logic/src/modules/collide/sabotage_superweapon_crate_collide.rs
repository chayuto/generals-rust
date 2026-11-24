//! `PKT-237`: `SabotageSuperweaponCrateCollide`
//! (`SabotageSuperweaponCrateCollide.h` / `SabotageSuperweaponCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, sabotage_goal_matches,
    sabotage_is_valid_to_execute, CrateCollide, CrateCollideData, CrateCollideImpl,
    SabotageVictimType,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, FALSE, TRUE};

/// Template configuration parsed for `SabotageSuperweaponCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct SabotageSuperweaponCrateCollideModuleData {
    pub crate_data: CrateCollideData,
}

impl ModuleData for SabotageSuperweaponCrateCollideModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.crate_data.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.crate_data.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::COLLIDE
    }
}

pub fn parse_sabotage_superweapon_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |_field, _reader, _ctx| Err(IniError::UnknownToken),
    )?;
    Ok(Box::new(SabotageSuperweaponCrateCollideModuleData {
        crate_data,
    }))
}

#[derive(Debug, Clone)]
pub struct SabotageSuperweaponCrateCollide {
    base: CrateCollide,
}

impl SabotageSuperweaponCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SabotageSuperweaponCrateCollideModuleData>()
            .expect("SabotageSuperweaponCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
        })
    }
}

impl CrateCollideImpl for SabotageSuperweaponCrateCollide {
    fn crate_base(&self) -> &CrateCollide {
        &self.base
    }

    fn crate_base_mut(&mut self) -> &mut CrateCollide {
        &mut self.base
    }

    fn is_valid_to_execute(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
    ) -> Bool {
        sabotage_is_valid_to_execute(&self.base, sim, ctx, other, |s, slot| {
            Bool::from(
                s.object_is_kind_of(slot, KindOfType::FsSuperweapon) != FALSE
                    || s.object_is_kind_of(slot, KindOfType::FsStrategyCenter) != FALSE,
            )
        })
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool {
        if sabotage_goal_matches(sim, ctx.owner_slot, other) == FALSE {
            return FALSE;
        }
        self.base
            .do_sabotage_feedback_fx(sim, ctx, other, SabotageVictimType::Superweapon);
        sim.reset_special_power_timer(other, 0);
        TRUE
    }

    fn is_sabotage_building_crate_collide(&self) -> Bool {
        TRUE
    }
}

impl_crate_collide_behavior!(SabotageSuperweaponCrateCollide);
