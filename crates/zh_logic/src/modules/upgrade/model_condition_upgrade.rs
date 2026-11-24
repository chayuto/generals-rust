//! `PKT-223`: `ModelConditionUpgrade` (`ModelConditionUpgrade.h` / `ModelConditionUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::rules::model_state::{ModelConditionFlagType, MODEL_CONDITION_NAMES};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::NameKeyType;

/// Template configuration parsed for `ModelConditionUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModelConditionUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub condition_flag: ModelConditionFlagType,
}

impl ModuleData for ModelConditionUpgradeModuleData {
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

pub fn parse_model_condition_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut condition_flag = ModelConditionFlagType::Invalid;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("ConditionFlag") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                let idx = IniReader::scan_index_list(tok, &MODEL_CONDITION_NAMES)?;
                condition_flag = ModelConditionFlagType::from_i32(idx);
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(ModelConditionUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        condition_flag,
    }))
}

#[derive(Debug, Clone)]
pub struct ModelConditionUpgrade {
    base: UpgradeModule,
    condition_flag: ModelConditionFlagType,
}

impl ModelConditionUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ModelConditionUpgradeModuleData>()
            .expect("ModelConditionUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            condition_flag: typed.condition_flag,
        })
    }
}

impl UpgradeBehaviorImpl for ModelConditionUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.condition_flag != ModelConditionFlagType::Invalid {
            sim.set_model_condition_state(ctx.owner_slot, self.condition_flag);
        }
    }
}

impl_upgrade_behavior!(ModelConditionUpgrade);
