//! `PKT-256`: `WeaponSetUpgrade` (`WeaponSetUpgrade.h` / `WeaponSetUpgrade.cpp`).

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_module_data, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeModuleData,
};
use crate::rules::combat_types::WeaponSetType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, TRUE};

pub fn parse_weapon_set_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_upgrade_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct WeaponSetUpgrade {
    base: UpgradeModule,
}

impl WeaponSetUpgrade {
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

impl UpgradeBehaviorImpl for WeaponSetUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.set_weapon_set_flag(ctx.owner_slot, WeaponSetType::PlayerUpgrade, TRUE);
    }
}

impl_upgrade_behavior!(WeaponSetUpgrade);
