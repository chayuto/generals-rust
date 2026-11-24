//! `PKT-214`: `ExperienceScalarUpgrade` (`ExperienceScalarUpgrade.h` / `ExperienceScalarUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, Real};

/// Template configuration parsed for `ExperienceScalarUpgrade`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExperienceScalarUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub add_xp_scalar: Real,
}

impl ModuleData for ExperienceScalarUpgradeModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::UPGRADE
    }
}

pub fn parse_experience_scalar_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut add_xp_scalar: Real = 0.0;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("AddXPScalar") {
                reader.parse_real(&mut add_xp_scalar)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(ExperienceScalarUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        add_xp_scalar,
    }))
}

#[derive(Debug, Clone)]
pub struct ExperienceScalarUpgrade {
    base: UpgradeModule,
    add_xp_scalar: Real,
}

impl ExperienceScalarUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ExperienceScalarUpgradeModuleData>()
            .expect("ExperienceScalarUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            add_xp_scalar: typed.add_xp_scalar,
        })
    }
}

impl UpgradeBehaviorImpl for ExperienceScalarUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.add_experience_scalar(ctx.owner_slot, self.add_xp_scalar);
    }
}

impl_upgrade_behavior!(ExperienceScalarUpgrade);
