//! `PKT-248`: `StealthUpgrade` (`StealthUpgrade.h` / `StealthUpgrade.cpp`).

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_module_data, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeModuleData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::NameKeyType;

pub fn parse_stealth_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_upgrade_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct StealthUpgrade {
    base: UpgradeModule,
}

impl StealthUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<UpgradeModuleData>()
            .expect("UpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
        })
    }
}

impl UpgradeBehaviorImpl for StealthUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.grant_stealth(ctx.owner_slot);
    }
}

impl_upgrade_behavior!(StealthUpgrade);
