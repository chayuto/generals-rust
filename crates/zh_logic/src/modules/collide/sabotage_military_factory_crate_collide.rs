//! `PKT-235`: `SabotageMilitaryFactoryCrateCollide`
//! (`SabotageMilitaryFactoryCrateCollide.h` / `SabotageMilitaryFactoryCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, sabotage_goal_matches,
    sabotage_is_valid_to_execute, CrateCollide, CrateCollideData, CrateCollideImpl,
    SabotageVictimType,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, FALSE, TRUE};

/// Template configuration parsed for `SabotageMilitaryFactoryCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct SabotageMilitaryFactoryCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub sabotage_frames: UnsignedInt,
}

impl ModuleData for SabotageMilitaryFactoryCrateCollideModuleData {
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

pub fn parse_sabotage_military_factory_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut sabotage_frames: UnsignedInt = 0;
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("SabotageDuration") {
                reader.parse_duration_unsigned_int(&mut sabotage_frames)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(SabotageMilitaryFactoryCrateCollideModuleData {
        crate_data,
        sabotage_frames,
    }))
}

#[derive(Debug, Clone)]
pub struct SabotageMilitaryFactoryCrateCollide {
    base: CrateCollide,
    sabotage_frames: UnsignedInt,
}

impl SabotageMilitaryFactoryCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SabotageMilitaryFactoryCrateCollideModuleData>()
            .expect("SabotageMilitaryFactoryCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            sabotage_frames: typed.sabotage_frames,
        })
    }
}

impl CrateCollideImpl for SabotageMilitaryFactoryCrateCollide {
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
            if s.object_is_kind_of(slot, KindOfType::AircraftCarrier) != FALSE {
                return FALSE;
            }
            Bool::from(
                s.object_is_kind_of(slot, KindOfType::FsBarracks) != FALSE
                    || s.object_is_kind_of(slot, KindOfType::FsWarfactory) != FALSE
                    || s.object_is_kind_of(slot, KindOfType::FsAirfield) != FALSE,
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
            .do_sabotage_feedback_fx(sim, ctx, other, SabotageVictimType::MilitaryFactory);
        let until_frame = sim.frame().saturating_add(self.sabotage_frames);
        sim.set_disabled_until(other, DisabledType::Hacked, until_frame);
        TRUE
    }

    fn is_sabotage_building_crate_collide(&self) -> Bool {
        TRUE
    }
}

impl_crate_collide_behavior!(SabotageMilitaryFactoryCrateCollide);
