//! `PKT-209`: `DefectorSpecialPower` (`DefectorSpecialPower.h` / `DefectorSpecialPower.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::rules::special_power::DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, UnsignedInt, TRUE};

#[derive(Debug, Clone, PartialEq)]
pub struct DefectorSpecialPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub fat_cursor_radius: Real,
}

impl ModuleData for DefectorSpecialPowerModuleData {
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
        FacetMask::SPECIAL_POWER
    }
}

pub fn parse_defector_special_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut fat_cursor_radius: Real = 0.0;
    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("FatCursorRadius") {
                reader.parse_real(&mut fat_cursor_radius)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(DefectorSpecialPowerModuleData {
        base,
        fat_cursor_radius,
    }))
}

#[derive(Debug, Clone)]
pub struct DefectorSpecialPower {
    base: SpecialPowerModule,
    pub fat_cursor_radius: Real,
}

impl DefectorSpecialPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<DefectorSpecialPowerModuleData>()
            .expect("DefectorSpecialPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            fat_cursor_radius: typed.fat_cursor_radius,
        })
    }
}

impl SpecialPowerBehaviorImpl for DefectorSpecialPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
    }

    fn do_special_power_at_location(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _loc: &Coord3D,
        _angle: Real,
        _command_options: UnsignedInt,
    ) {
        let _ = SpecialPowerModule::is_owner_disabled(sim, ctx);
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        if sim.object(target).is_none() {
            return;
        }
        let Some(self_team) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };

        SpecialPowerFacet::do_special_power_at_object(
            &mut self.base,
            sim,
            ctx,
            target,
            command_options,
        );

        let detection_time = self
            .base
            .resolve_template(sim)
            .map_or(DEFAULT_DEFECTION_DETECTION_PROTECTION_TIME_LIMIT, |t| {
                t.detection_time()
            });
        sim.defect_object(target, self_team, detection_time, TRUE);
    }
}

impl_special_power_behavior!(DefectorSpecialPower);
