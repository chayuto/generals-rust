//! `PKT-224`: `MoneyCrateCollide` (`MoneyCrateCollide.h` / `MoneyCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, CrateCollide, CrateCollideData,
    CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, NameKeyType, UnsignedInt, FALSE, TRUE};

/// One `UpgradedBoost` entry parsed via `parseUpgradePair`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradePair {
    pub upgrade_type: String,
    pub amount: Int,
}

/// Template configuration parsed for `MoneyCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct MoneyCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub money_provided: UnsignedInt,
    pub upgrade_boost: Vec<UpgradePair>,
}

impl ModuleData for MoneyCrateCollideModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.crate_data.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.crate_data.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::COLLIDE
    }
}

fn parse_upgrade_pair(reader: &mut IniReader<'_>) -> Result<UpgradePair, IniError> {
    let mut upgrade_type = String::new();
    let mut amount: Int = 0;
    while let Some(tok) = reader.next_token_or_null(SEPS_DEFAULT) {
        if let Some((k, v)) = tok.split_once(':') {
            if k.eq_ignore_ascii_case("UpgradeType") {
                upgrade_type = v.to_owned();
            } else if k.eq_ignore_ascii_case("Boost") {
                amount = IniReader::scan_int(v).map_err(|_| IniError::InvalidData)?;
            } else {
                return Err(IniError::UnknownToken);
            }
        } else if tok.eq_ignore_ascii_case("UpgradeType") {
            upgrade_type = reader.next_token(SEPS_DEFAULT)?.to_owned();
        } else if tok.eq_ignore_ascii_case("Boost") {
            reader.parse_int(&mut amount)?;
        } else if upgrade_type.is_empty() {
            upgrade_type = tok.to_owned();
        } else {
            amount = IniReader::scan_int(tok).map_err(|_| IniError::InvalidData)?;
        }
    }
    Ok(UpgradePair {
        upgrade_type,
        amount,
    })
}

pub fn parse_money_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut money_provided: UnsignedInt = 0;
    let mut upgrade_boost = Vec::new();
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("MoneyProvided") {
                reader.parse_unsigned_int(&mut money_provided)
            } else if field.eq_ignore_ascii_case("UpgradedBoost") {
                let pair = parse_upgrade_pair(reader)?;
                upgrade_boost.push(pair);
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(MoneyCrateCollideModuleData {
        crate_data,
        money_provided,
        upgrade_boost,
    }))
}

#[derive(Debug, Clone)]
pub struct MoneyCrateCollide {
    base: CrateCollide,
    money_provided: UnsignedInt,
    upgrade_boost: Vec<UpgradePair>,
}

impl MoneyCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<MoneyCrateCollideModuleData>()
            .expect("MoneyCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            money_provided: typed.money_provided,
            upgrade_boost: typed.upgrade_boost.clone(),
        })
    }

    pub fn get_upgraded_supply_boost(&self, sim: &dyn SimView, other: ObjectSlot) -> Int {
        let Some(other_team) = sim.object(other).map(|o| o.team_id) else {
            return 0;
        };
        let rules = sim.rules();
        for pair in &self.upgrade_boost {
            if let Some(ut) = rules
                .upgrade_center
                .lookup_upgrade(&pair.upgrade_type, &rules.name_keys)
            {
                if sim.player_has_upgrade(other_team, ut.upgrade_mask()) != FALSE {
                    return pair.amount;
                }
            }
        }
        0
    }
}

impl CrateCollideImpl for MoneyCrateCollide {
    fn crate_base(&self) -> &CrateCollide {
        &self.base
    }

    fn crate_base_mut(&mut self) -> &mut CrateCollide {
        &mut self.base
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool {
        let mut money = self.money_provided;
        let boost = self.get_upgraded_supply_boost(sim, other);
        money = money.wrapping_add(boost as UnsignedInt);
        if let Some(other_team) = sim.object(other).map(|o| o.team_id) {
            let _ = sim.add_player_money(other_team, money as Int);
        }
        TRUE
    }
}

impl_crate_collide_behavior!(MoneyCrateCollide);
