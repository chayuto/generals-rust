//! `PKT-250`: `UnitCrateCollide` (`UnitCrateCollide.h` / `UnitCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, CrateCollide, CrateCollideData,
    CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, FALSE, TRUE};

/// Template configuration parsed for `UnitCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct UnitCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub unit_count: UnsignedInt,
    pub unit_type: String,
}

impl ModuleData for UnitCrateCollideModuleData {
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

pub fn parse_unit_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut unit_count: UnsignedInt = 0;
    let mut unit_type = String::new();
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("UnitCount") {
                reader.parse_unsigned_int(&mut unit_count)
            } else if field.eq_ignore_ascii_case("UnitName") {
                reader.parse_ascii_string(&mut unit_type)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(UnitCrateCollideModuleData {
        crate_data,
        unit_count,
        unit_type,
    }))
}

#[derive(Debug, Clone)]
pub struct UnitCrateCollide {
    base: CrateCollide,
    unit_count: UnsignedInt,
    unit_type: String,
}

impl UnitCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<UnitCrateCollideModuleData>()
            .expect("UnitCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            unit_count: typed.unit_count,
            unit_type: typed.unit_type.clone(),
        })
    }
}

impl CrateCollideImpl for UnitCrateCollide {
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
        let Some(unit_tmpl_id) = sim.rules().thing_factory.find_template_id(&self.unit_type) else {
            return FALSE;
        };
        let Some((team_id, pos, angle)) = sim.object(other).map(|o| {
            (
                o.team_id,
                *o.thing.get_position(),
                o.thing.get_orientation(),
            )
        }) else {
            return FALSE;
        };
        for _ in 0..self.unit_count {
            let _ = sim.spawn_object(unit_tmpl_id, team_id, pos, angle);
        }
        TRUE
    }
}

impl_crate_collide_behavior!(UnitCrateCollide);
