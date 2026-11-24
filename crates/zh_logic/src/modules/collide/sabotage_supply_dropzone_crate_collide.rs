//! `PKT-239`: `SabotageSupplyDropzoneCrateCollide`
//! (`SabotageSupplyDropzoneCrateCollide.h` / `SabotageSupplyDropzoneCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, sabotage_goal_matches,
    sabotage_is_valid_to_execute, sabotage_steal_cash, CrateCollide, CrateCollideData,
    CrateCollideImpl, SabotageVictimType,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, FALSE, TRUE};

/// Template configuration parsed for `SabotageSupplyDropzoneCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct SabotageSupplyDropzoneCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub steal_cash_amount: UnsignedInt,
}

impl ModuleData for SabotageSupplyDropzoneCrateCollideModuleData {
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

pub fn parse_sabotage_supply_dropzone_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut steal_cash_amount: UnsignedInt = 0;
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("StealCashAmount") {
                reader.parse_unsigned_int(&mut steal_cash_amount)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(SabotageSupplyDropzoneCrateCollideModuleData {
        crate_data,
        steal_cash_amount,
    }))
}

#[derive(Debug, Clone)]
pub struct SabotageSupplyDropzoneCrateCollide {
    base: CrateCollide,
    steal_cash_amount: UnsignedInt,
}

impl SabotageSupplyDropzoneCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SabotageSupplyDropzoneCrateCollideModuleData>()
            .expect("SabotageSupplyDropzoneCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            steal_cash_amount: typed.steal_cash_amount,
        })
    }
}

impl CrateCollideImpl for SabotageSupplyDropzoneCrateCollide {
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
            s.object_is_kind_of(slot, KindOfType::FsSupplyDropzone)
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
            .do_sabotage_feedback_fx(sim, ctx, other, SabotageVictimType::DropZone);
        sim.reset_supply_dropzone_timer(other);
        sabotage_steal_cash(sim, ctx.owner_slot, other, self.steal_cash_amount);
        TRUE
    }

    fn is_sabotage_building_crate_collide(&self) -> Bool {
        TRUE
    }
}

impl_crate_collide_behavior!(SabotageSupplyDropzoneCrateCollide);
