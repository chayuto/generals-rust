//! `PKT-193`: `ActiveShroudUpgrade` (`ActiveShroudUpgrade.h` / `ActiveShroudUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::IniError;
use zh_common::types::{NameKeyType, Real};

/// Template configuration parsed for `ActiveShroudUpgrade`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ActiveShroudUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub new_shroud_range: Real,
}

impl ModuleData for ActiveShroudUpgradeModuleData {
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

pub fn parse_active_shroud_upgrade_data(
    ini: &mut zh_common::ini::IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut new_shroud_range: Real = 0.0;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("NewShroudRange") {
                reader.parse_real(&mut new_shroud_range)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(ActiveShroudUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        new_shroud_range,
    }))
}

#[derive(Debug, Clone)]
pub struct ActiveShroudUpgrade {
    base: UpgradeModule,
    new_shroud_range: Real,
}

impl ActiveShroudUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ActiveShroudUpgradeModuleData>()
            .expect("ActiveShroudUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            new_shroud_range: typed.new_shroud_range,
        })
    }
}

impl UpgradeBehaviorImpl for ActiveShroudUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.set_shroud_clearing_range(ctx.owner_slot, self.new_shroud_range);
    }
}

impl_upgrade_behavior!(ActiveShroudUpgrade);
