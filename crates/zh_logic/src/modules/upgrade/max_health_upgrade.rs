//! `PKT-222`: `MaxHealthUpgrade` (`MaxHealthUpgrade.h` / `MaxHealthUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{
    BehaviorModule, MaxHealthChangeType, ModuleData, MAX_HEALTH_CHANGE_NAMES,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, Real};

/// Template configuration parsed for `MaxHealthUpgrade`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MaxHealthUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub add_max_health: Real,
    pub max_health_change_type: MaxHealthChangeType,
}

impl ModuleData for MaxHealthUpgradeModuleData {
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

pub fn parse_max_health_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut add_max_health: Real = 0.0;
    let mut change_type = MaxHealthChangeType::SameCurrentHealth;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("AddMaxHealth") {
                reader.parse_real(&mut add_max_health)
            } else if field.eq_ignore_ascii_case("ChangeType") {
                let mut idx = 0;
                reader.parse_index_list(&mut idx, &MAX_HEALTH_CHANGE_NAMES)?;
                change_type =
                    MaxHealthChangeType::from_i32(idx).ok_or(IniError::InvalidNameList)?;
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(MaxHealthUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        add_max_health,
        max_health_change_type: change_type,
    }))
}

#[derive(Debug, Clone)]
pub struct MaxHealthUpgrade {
    base: UpgradeModule,
    add_max_health: Real,
    max_health_change_type: MaxHealthChangeType,
}

impl MaxHealthUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<MaxHealthUpgradeModuleData>()
            .expect("MaxHealthUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            add_max_health: typed.add_max_health,
            max_health_change_type: typed.max_health_change_type,
        })
    }
}

impl UpgradeBehaviorImpl for MaxHealthUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let cur_max = sim.object_max_health(ctx.owner_slot);
        sim.set_max_health(
            ctx.owner_slot,
            cur_max + self.add_max_health,
            self.max_health_change_type,
        );
    }
}

impl_upgrade_behavior!(MaxHealthUpgrade);
