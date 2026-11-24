//! `PKT-229`: `RadarUpgrade` (`RadarUpgrade.h` / `RadarUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, TeamId, FALSE, TRUE};

/// Template configuration parsed for `RadarUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RadarUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub is_disable_proof: Bool,
}

impl ModuleData for RadarUpgradeModuleData {
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

pub fn parse_radar_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut is_disable_proof = FALSE;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("DisableProof") {
                reader.parse_bool(&mut is_disable_proof)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(RadarUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        is_disable_proof,
    }))
}

#[derive(Debug, Clone)]
pub struct RadarUpgrade {
    base: UpgradeModule,
    pub is_disable_proof: Bool,
}

impl RadarUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<RadarUpgradeModuleData>()
            .expect("RadarUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            is_disable_proof: typed.is_disable_proof,
        })
    }
}

impl UpgradeBehaviorImpl for RadarUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            sim.add_radar(team_id, self.is_disable_proof);
        }
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        let Some((disabled, team_id)) = sim
            .object(ctx.owner_slot)
            .map(|o| (o.is_disabled() != FALSE, o.team_id))
        else {
            return;
        };
        if disabled {
            return;
        }
        sim.remove_radar(team_id, self.is_disable_proof);
        self.base.mux.set_upgrade_executed(FALSE);
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        if sim
            .object(ctx.owner_slot)
            .is_some_and(|o| o.is_disabled() != FALSE)
        {
            return;
        }
        sim.remove_radar(old_team, self.is_disable_proof);
        self.base.mux.set_upgrade_executed(FALSE);
        sim.add_radar(new_team, self.is_disable_proof);
        self.base.mux.set_upgrade_executed(TRUE);
    }

    fn on_disabled_edge(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, disabled: Bool) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            if disabled != FALSE {
                sim.remove_radar(team_id, self.is_disable_proof);
            } else {
                sim.add_radar(team_id, self.is_disable_proof);
            }
        }
    }
}

impl_upgrade_behavior!(RadarUpgrade);
