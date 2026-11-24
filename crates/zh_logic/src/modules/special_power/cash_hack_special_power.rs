//! `PKT-197`: `CashHackSpecialPower` (`CashHackSpecialPower.h` / `CashHackSpecialPower.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::rules::science::ScienceType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Int, NameKeyType, Real, UnsignedInt, FALSE};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashHackUpgrade {
    pub science: ScienceType,
    pub amount_to_steal: Int,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CashHackSpecialPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub upgrades: Vec<CashHackUpgrade>,
    pub default_amount_to_steal: Int,
}

impl ModuleData for CashHackSpecialPowerModuleData {
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

pub fn parse_cash_hack_special_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut upgrades = Vec::new();
    let mut default_amount_to_steal: Int = 0;
    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("UpgradeMoneyAmount") {
                let sci_tok = reader.next_token(SEPS_DEFAULT)?;
                let science = ctx
                    .science_store
                    .friend_lookup_science(sci_tok, ctx.name_keys)?;
                let mut amount_to_steal: Int = 0;
                reader.parse_int(&mut amount_to_steal)?;
                upgrades.push(CashHackUpgrade {
                    science,
                    amount_to_steal,
                });
                Ok(())
            } else if field.eq_ignore_ascii_case("MoneyAmount") {
                reader.parse_int(&mut default_amount_to_steal)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CashHackSpecialPowerModuleData {
        base,
        upgrades,
        default_amount_to_steal,
    }))
}

#[derive(Debug, Clone)]
pub struct CashHackSpecialPower {
    base: SpecialPowerModule,
    upgrades: Vec<CashHackUpgrade>,
    default_amount_to_steal: Int,
}

impl CashHackSpecialPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CashHackSpecialPowerModuleData>()
            .expect("CashHackSpecialPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            upgrades: typed.upgrades.clone(),
            default_amount_to_steal: typed.default_amount_to_steal,
        })
    }

    pub fn find_amount_to_steal(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int {
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            for u in &self.upgrades {
                if sim.player_has_science(team_id, u.science) != FALSE {
                    return u.amount_to_steal;
                }
            }
        }
        self.default_amount_to_steal
    }
}

impl SpecialPowerBehaviorImpl for CashHackSpecialPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
    }

    fn do_special_power_at_location(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _loc: &Coord3D,
        _angle: Real,
        _command_options: UnsignedInt,
    ) {
        let _ = SpecialPowerModule::is_owner_disabled(sim, ctx);
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        let Some(target_team) = sim.object(target).map(|o| o.team_id) else {
            return;
        };
        let Some(self_team) = sim.object(ctx.owner_slot).map(|o| o.team_id) else {
            return;
        };

        SpecialPowerFacet::do_special_power_at_object(
            &mut self.base,
            sim,
            ctx,
            target,
            command_options,
        );

        let avail = sim.player_money(target_team);
        let desired = self.find_amount_to_steal(sim, ctx) as UnsignedInt;
        let cash = desired.min(avail);
        if cash > 0 {
            let stolen = sim.withdraw_player_money(target_team, cash);
            let _ = sim.add_player_money(self_team, stolen as Int);
        }
    }
}

impl_special_power_behavior!(CashHackSpecialPower);
