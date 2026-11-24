//! `PKT-228`: `PowerPlantUpgrade` (`PowerPlantUpgrade.h` / `PowerPlantUpgrade.cpp`).

use crate::arena::ModuleContext;
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_module_data, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeModuleData,
};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, TeamId, FALSE, TRUE};

pub fn parse_power_plant_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_upgrade_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct PowerPlantUpgrade {
    base: UpgradeModule,
}

impl PowerPlantUpgrade {
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

impl UpgradeBehaviorImpl for PowerPlantUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            sim.add_power_bonus(team_id, ctx.owner_slot);
        }
        sim.extend_power_plant(ctx.owner_slot);
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            sim.remove_power_bonus(team_id, ctx.owner_slot);
        }
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
        sim.remove_power_bonus(old_team, ctx.owner_slot);
        self.base.mux.set_upgrade_executed(FALSE);
        sim.add_power_bonus(new_team, ctx.owner_slot);
        self.base.mux.set_upgrade_executed(TRUE);
    }

    fn on_disabled_edge(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, disabled: Bool) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        let Some((team_id, under_const, prod, bonus)) = sim.object(ctx.owner_slot).and_then(|o| {
            let tmpl = o
                .thing
                .template_override()
                .get_final(&sim.rules().thing_factory)?;
            Some((
                o.team_id,
                o.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE,
                tmpl.get_energy_production(),
                tmpl.get_energy_bonus(),
            ))
        }) else {
            return;
        };
        #[cfg(feature = "retail-crc")]
        let _ = under_const;
        #[cfg(not(feature = "retail-crc"))]
        if under_const {
            return;
        }
        if prod > 0 {
            sim.adjust_player_power(team_id, bonus, Bool::from(disabled == FALSE));
        }
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        if let Some(obj) = sim.object(ctx.owner_slot) {
            if obj.is_disabled() == FALSE {
                let team_id = obj.team_id;
                sim.add_power_bonus(team_id, ctx.owner_slot);
            }
        }
    }
}

impl_upgrade_behavior!(PowerPlantUpgrade);
