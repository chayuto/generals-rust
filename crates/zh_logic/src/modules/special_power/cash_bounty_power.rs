//! `PKT-196`: `CashBountyPower` (`CashBountyPower.h` / `CashBountyPower.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::rules::science::SCIENCE_INVALID;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, Real, UnsignedInt, FALSE};

#[derive(Debug, Clone, PartialEq)]
pub struct CashBountyPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub default_bounty: Real,
}

impl ModuleData for CashBountyPowerModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.base.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::SPECIAL_POWER
    }
}

pub fn parse_cash_bounty_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut default_bounty: Real = 0.0;
    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("Bounty") {
                reader.parse_percent_to_real(&mut default_bounty)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CashBountyPowerModuleData {
        base,
        default_bounty,
    }))
}

#[derive(Debug, Clone)]
pub struct CashBountyPower {
    base: SpecialPowerModule,
    default_bounty: Real,
}

impl CashBountyPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CashBountyPowerModuleData>()
            .expect("CashBountyPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            default_bounty: typed.default_bounty,
        })
    }

    #[inline]
    pub const fn find_bounty(&self) -> Real {
        self.default_bounty
    }
}

impl SpecialPowerBehaviorImpl for CashBountyPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.on_object_created(sim, ctx);
        let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        let req_sci = self.base.get_required_science(sim, ctx);
        if req_sci != SCIENCE_INVALID && sim.player_has_science(team_id, req_sci) != FALSE {
            let bounty = self.find_bounty();
            if bounty > sim.player_bounty_percent(team_id) {
                sim.apply_cash_bounty_percent(team_id, bounty);
            }
        }
    }

    fn on_special_power_creation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        SpecialPowerFacet::on_special_power_creation(&mut self.base, sim, ctx);
        let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };
        let bounty = self.find_bounty();
        if bounty > sim.player_bounty_percent(team_id) {
            sim.apply_cash_bounty_percent(team_id, bounty);
        }
    }

    fn do_special_power(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _command_options: UnsignedInt,
    ) {
    }
}

impl_special_power_behavior!(CashBountyPower);
