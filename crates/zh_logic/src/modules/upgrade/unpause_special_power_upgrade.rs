//! `PKT-251`: `UnpauseSpecialPowerUpgrade` (`UnpauseSpecialPowerUpgrade.h` / `UnpauseSpecialPowerUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{NameKeyType, UnsignedInt};

/// Template configuration parsed for `UnpauseSpecialPowerUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnpauseSpecialPowerUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub special_power_name: String,
    pub special_power_id: UnsignedInt,
}

impl ModuleData for UnpauseSpecialPowerUpgradeModuleData {
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

pub fn parse_unpause_special_power_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut special_power_name = String::new();
    let mut special_power_id: UnsignedInt = 0;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("SpecialPowerTemplate") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    special_power_name.clear();
                    special_power_id = 0;
                } else {
                    special_power_name = tok.to_owned();
                    special_power_id = ctx
                        .special_power_store
                        .find_special_power_template(tok)
                        .map_or(0, |t| t.id());
                }
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(UnpauseSpecialPowerUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        special_power_name,
        special_power_id,
    }))
}

#[derive(Debug, Clone)]
pub struct UnpauseSpecialPowerUpgrade {
    base: UpgradeModule,
    special_power_name: String,
    special_power_id: UnsignedInt,
}

impl UnpauseSpecialPowerUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<UnpauseSpecialPowerUpgradeModuleData>()
            .expect("UnpauseSpecialPowerUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            special_power_name: typed.special_power_name.clone(),
            special_power_id: typed.special_power_id,
        })
    }
}

impl UpgradeBehaviorImpl for UnpauseSpecialPowerUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let tid = if self.special_power_id != 0 {
            self.special_power_id
        } else if !self.special_power_name.is_empty() {
            sim.rules()
                .special_power_store
                .find_special_power_template(&self.special_power_name)
                .map_or(0, |t| t.id())
        } else {
            0
        };
        if tid != 0 {
            sim.unpause_special_power(ctx.owner_slot, tid);
        }
    }
}

impl_upgrade_behavior!(UnpauseSpecialPowerUpgrade);
