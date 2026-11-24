//! `PKT-203`: `CostModifierUpgrade` (`CostModifierUpgrade.h` / `CostModifierUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::rules::kind_of::{KindOfMaskType, KINDOF_NAMES};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, Real, TeamId, FALSE, TRUE};

/// Template configuration parsed for `CostModifierUpgrade`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CostModifierUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub kind_of: KindOfMaskType,
    pub percentage: Real,
}

impl ModuleData for CostModifierUpgradeModuleData {
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

pub fn parse_cost_modifier_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut kind_of = KindOfMaskType::new();
    let mut percentage: Real = 0.0;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("EffectKindOf") {
                kind_of.parse_from_ini(reader, &KINDOF_NAMES)
            } else if field.eq_ignore_ascii_case("Percentage") {
                reader.parse_percent_to_real(&mut percentage)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CostModifierUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        kind_of,
        percentage,
    }))
}

#[derive(Debug, Clone)]
pub struct CostModifierUpgrade {
    base: UpgradeModule,
    data: CostModifierUpgradeModuleData,
}

impl CostModifierUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CostModifierUpgradeModuleData>()
            .expect("CostModifierUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            data: typed.clone(),
        })
    }
}

impl UpgradeBehaviorImpl for CostModifierUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            sim.add_cost_modifier(team_id, &self.data.kind_of, self.data.percentage);
        }
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            sim.remove_cost_modifier(team_id, &self.data.kind_of, self.data.percentage);
        }
        self.base.mux.set_upgrade_executed(FALSE);
    }

    fn on_capture(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        if self.base.mux.is_already_upgraded() == FALSE {
            return;
        }
        sim.remove_cost_modifier(old_team, &self.data.kind_of, self.data.percentage);
        self.base.mux.set_upgrade_executed(FALSE);
        sim.add_cost_modifier(new_team, &self.data.kind_of, self.data.percentage);
        self.base.mux.set_upgrade_executed(TRUE);
    }
}

impl_upgrade_behavior!(CostModifierUpgrade);
