//! `PKT-218`: `GrantScienceUpgrade` (`GrantScienceUpgrade.h` / `GrantScienceUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::NameKeyType;

/// Template configuration parsed for `GrantScienceUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GrantScienceUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub grant_science_name: String,
}

impl ModuleData for GrantScienceUpgradeModuleData {
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

pub fn parse_grant_science_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut grant_science_name = String::new();
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("GrantScience") {
                reader.parse_ascii_string(&mut grant_science_name)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(GrantScienceUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        grant_science_name,
    }))
}

#[derive(Debug, Clone)]
pub struct GrantScienceUpgrade {
    base: UpgradeModule,
    grant_science_name: String,
    science_type: ScienceType,
}

impl GrantScienceUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<GrantScienceUpgradeModuleData>()
            .expect("GrantScienceUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            grant_science_name: typed.grant_science_name.clone(),
            science_type: SCIENCE_INVALID,
        })
    }
}

impl UpgradeBehaviorImpl for GrantScienceUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.science_type == SCIENCE_INVALID && !self.grant_science_name.is_empty() {
            if let Some(key) = sim.rules().name_keys.lookup_key(&self.grant_science_name) {
                self.science_type = key.0;
            }
        }
        if self.science_type != SCIENCE_INVALID {
            if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
                let _ = sim.grant_player_science(team_id, self.science_type);
            }
        }
    }
}

impl_upgrade_behavior!(GrantScienceUpgrade);
