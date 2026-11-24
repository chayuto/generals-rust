//! `PKT-200`: `CommandSetUpgrade` (`CommandSetUpgrade.h` / `CommandSetUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, FALSE};

/// Template configuration parsed for `CommandSetUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandSetUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub new_command_set: String,
    pub new_command_set_alt: String,
    pub trigger_alt: String,
}

impl ModuleData for CommandSetUpgradeModuleData {
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

pub fn parse_command_set_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut new_command_set = String::new();
    let mut new_command_set_alt = String::new();
    let mut trigger_alt = String::new();
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("CommandSet") {
                reader.parse_ascii_string(&mut new_command_set)
            } else if field.eq_ignore_ascii_case("CommandSetAlt") {
                reader.parse_ascii_string(&mut new_command_set_alt)
            } else if field.eq_ignore_ascii_case("TriggerAlt") {
                reader.parse_ascii_string(&mut trigger_alt)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CommandSetUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        new_command_set,
        new_command_set_alt,
        trigger_alt,
    }))
}

#[derive(Debug, Clone)]
pub struct CommandSetUpgrade {
    base: UpgradeModule,
    data: CommandSetUpgradeModuleData,
}

impl CommandSetUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CommandSetUpgradeModuleData>()
            .expect("CommandSetUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            data: typed.clone(),
        })
    }
}

impl UpgradeBehaviorImpl for CommandSetUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let use_alt = if let Some(alt_mask) = sim
            .rules()
            .upgrade_center
            .lookup_upgrade(&self.data.trigger_alt, &sim.rules().name_keys)
            .map(|t| t.upgrade_mask().clone())
        {
            let team_id = sim.object(ctx.owner_slot).map(|o| o.team_id);
            let player_has = team_id.is_some_and(|tid| sim.player_has_upgrade(tid, &alt_mask) != FALSE);
            let obj_has = sim.object_has_upgrade(ctx.owner_slot, &alt_mask) != FALSE;
            player_has || obj_has
        } else {
            false
        };
        if use_alt {
            sim.set_command_set(ctx.owner_slot, &self.data.new_command_set_alt);
        } else {
            sim.set_command_set(ctx.owner_slot, &self.data.new_command_set);
        }
    }
}

impl_upgrade_behavior!(CommandSetUpgrade);
