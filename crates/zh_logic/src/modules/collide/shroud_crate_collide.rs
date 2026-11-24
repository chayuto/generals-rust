//! `PKT-241`: `ShroudCrateCollide` (`ShroudCrateCollide.h` / `ShroudCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, CrateCollide, CrateCollideData,
    CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, FALSE, TRUE};

/// Template configuration parsed for `ShroudCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct ShroudCrateCollideModuleData {
    pub crate_data: CrateCollideData,
}

impl ModuleData for ShroudCrateCollideModuleData {
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

pub fn parse_shroud_crate_collide_data(
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
    Ok(Box::new(ShroudCrateCollideModuleData { crate_data }))
}

#[derive(Debug, Clone)]
pub struct ShroudCrateCollide {
    base: CrateCollide,
}

impl ShroudCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ShroudCrateCollideModuleData>()
            .expect("ShroudCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
        })
    }
}

impl CrateCollideImpl for ShroudCrateCollide {
    fn crate_base(&self) -> &CrateCollide {
        &self.base
    }

    fn crate_base_mut(&mut self) -> &mut CrateCollide {
        &mut self.base
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool {
        let Some(other_team) = sim.object(other).map(|o| o.team_id) else {
            return FALSE;
        };
        sim.reveal_map_for_player(other_team);
        TRUE
    }
}

impl_crate_collide_behavior!(ShroudCrateCollide);
