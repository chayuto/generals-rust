//! `PKT-195`: `BaikonurLaunchPower` (`BaikonurLaunchPower.h` / `BaikonurLaunchPower.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::Coord3D;
use zh_common::types::{NameKeyType, Real, UnsignedInt};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaikonurLaunchPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub detonation_object: String,
}

impl ModuleData for BaikonurLaunchPowerModuleData {
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

pub fn parse_baikonur_launch_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut detonation_object = String::new();
    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("DetonationObject") {
                reader.parse_ascii_string(&mut detonation_object)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(BaikonurLaunchPowerModuleData {
        base,
        detonation_object,
    }))
}

#[derive(Debug, Clone)]
pub struct BaikonurLaunchPower {
    base: SpecialPowerModule,
    detonation_object: String,
}

impl BaikonurLaunchPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<BaikonurLaunchPowerModuleData>()
            .expect("BaikonurLaunchPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            detonation_object: typed.detonation_object.clone(),
        })
    }
}

impl SpecialPowerBehaviorImpl for BaikonurLaunchPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
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
        sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Door1Opening);
        sim.trigger_baikonur_launch(ctx.owner_slot);
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
        if self.detonation_object.is_empty() {
            return;
        }
        let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        if let Some(tmpl_id) = sim
            .rules()
            .thing_factory
            .find_template_id(&self.detonation_object)
        {
            let _ = sim.spawn_object(tmpl_id, team_id, *loc, 0.0);
        }
    }
}

impl_special_power_behavior!(BaikonurLaunchPower);
