//! `PKT-231`: `ReplaceObjectUpgrade` (`ReplaceObjectUpgrade.h` / `ReplaceObjectUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::NameKeyType;

/// Template configuration parsed for `ReplaceObjectUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplaceObjectUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub replace_object_name: String,
}

impl ModuleData for ReplaceObjectUpgradeModuleData {
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

pub fn parse_replace_object_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut replace_object_name = String::new();
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("ReplaceObject") {
                reader.parse_ascii_string(&mut replace_object_name)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(ReplaceObjectUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        replace_object_name,
    }))
}

#[derive(Debug, Clone)]
pub struct ReplaceObjectUpgrade {
    base: UpgradeModule,
    replace_object_name: String,
}

impl ReplaceObjectUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ReplaceObjectUpgradeModuleData>()
            .expect("ReplaceObjectUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            replace_object_name: typed.replace_object_name.clone(),
        })
    }
}

impl UpgradeBehaviorImpl for ReplaceObjectUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let _ = sim.replace_object(ctx.owner_slot, &self.replace_object_name);
    }
}

impl_upgrade_behavior!(ReplaceObjectUpgrade);
