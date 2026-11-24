//! `PKT-242`: `SpecialAbility` (`SpecialAbility.h` / `SpecialAbility.cpp`).

use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_module_data, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, UnsignedInt};

pub fn parse_special_ability_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_special_power_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct SpecialAbility {
    base: SpecialPowerModule,
}

impl SpecialAbility {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SpecialPowerModuleData>()
            .expect("SpecialPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.clone()),
        })
    }
}

impl SpecialPowerBehaviorImpl for SpecialAbility {
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
        angle: Real,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        SpecialPowerFacet::do_special_power_at_location(
            &mut self.base,
            sim,
            ctx,
            loc,
            angle,
            command_options,
        );
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) || sim.object(target).is_none() {
            return;
        }
        SpecialPowerFacet::do_special_power_at_object(
            &mut self.base,
            sim,
            ctx,
            target,
            command_options,
        );
    }

    fn do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        SpecialPowerFacet::do_special_power(&mut self.base, sim, ctx, command_options);
    }
}

impl_special_power_behavior!(SpecialAbility);
