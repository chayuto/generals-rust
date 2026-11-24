//! `PKT-255`: `WeaponBonusUpgrade` (`WeaponBonusUpgrade.h` / `WeaponBonusUpgrade.cpp`).

use crate::arena::ModuleContext;
use crate::gamelogic::global_data::WeaponBonusConditionType;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_module_data, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeModuleData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::NameKeyType;

pub fn parse_weapon_bonus_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_upgrade_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct WeaponBonusUpgrade {
    base: UpgradeModule,
}

impl WeaponBonusUpgrade {
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

impl UpgradeBehaviorImpl for WeaponBonusUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.set_weapon_bonus_condition(ctx.owner_slot, WeaponBonusConditionType::PlayerUpgrade);
    }
}

impl_upgrade_behavior!(WeaponBonusUpgrade);
