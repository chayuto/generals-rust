//! `PKT-198`: `CleanupAreaPower` (`CleanupAreaPower.h` / `CleanupAreaPower.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, UnsignedInt};

#[derive(Debug, Clone, PartialEq)]
pub struct CleanupAreaPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub cleanup_move_range: Real,
}

impl ModuleData for CleanupAreaPowerModuleData {
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

pub fn parse_cleanup_area_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut cleanup_move_range: Real = 0.0;
    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("MaxMoveDistanceFromLocation") {
                reader.parse_real(&mut cleanup_move_range)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CleanupAreaPowerModuleData {
        base,
        cleanup_move_range,
    }))
}

#[derive(Debug, Clone)]
pub struct CleanupAreaPower {
    base: SpecialPowerModule,
    pub cleanup_move_range: Real,
}

impl CleanupAreaPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CleanupAreaPowerModuleData>()
            .expect("CleanupAreaPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            cleanup_move_range: typed.cleanup_move_range,
        })
    }
}

impl SpecialPowerBehaviorImpl for CleanupAreaPower {
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
        loc: &Coord3D,
        _angle: Real,
        _command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        sim.trigger_cleanup_area(ctx.owner_slot, loc);
    }
}

impl_special_power_behavior!(CleanupAreaPower);
