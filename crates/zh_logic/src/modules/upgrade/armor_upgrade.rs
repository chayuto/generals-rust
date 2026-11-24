//! `PKT-194`: `ArmorUpgrade` (`ArmorUpgrade.h` / `ArmorUpgrade.cpp`).

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_module_data, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeModuleData,
};
use crate::rules::combat_types::ArmorSetType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, FALSE, TRUE};

pub fn parse_armor_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_upgrade_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct ArmorUpgrade {
    base: UpgradeModule,
}

impl ArmorUpgrade {
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

impl UpgradeBehaviorImpl for ArmorUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.set_armor_set_flag(ctx.owner_slot, ArmorSetType::PlayerUpgrade, TRUE);
        // Client-side `Drawable::setTerrainDecal(TERRAIN_DECAL_CHEMSUIT)` when triggered by
        // `Upgrade_AmericaChemicalSuits`; checked here for parity.
        let _is_chem_suits = self.base.is_triggered_by("Upgrade_AmericaChemicalSuits") != FALSE;
    }
}

impl_upgrade_behavior!(ArmorUpgrade);
