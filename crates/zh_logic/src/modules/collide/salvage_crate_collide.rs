//! `PKT-240`: `SalvageCrateCollide` (`SalvageCrateCollide.h` / `SalvageCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::modules::collide::crate_collide::{
    gain_exp_for_level, impl_crate_collide_behavior, parse_crate_collide_data_with, CrateCollide,
    CrateCollideData, CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{ArmorSetType, WeaponSetType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{armor_set_flags_from_u8, SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, Int, NameKeyType, Real, VeterancyLevel, FALSE, TRUE};
use zh_common::xfer::bit_flags::ModelConditionFlags;

/// Template configuration parsed for `SalvageCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct SalvageCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub weapon_chance: Real,
    pub level_chance: Real,
    pub money_chance: Real,
    pub minimum_money: Int,
    pub maximum_money: Int,
}

impl ModuleData for SalvageCrateCollideModuleData {
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

pub fn parse_salvage_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut weapon_chance: Real = 1.0;
    let mut level_chance: Real = 0.25;
    let mut money_chance: Real = 0.75;
    let mut minimum_money: Int = 25;
    let mut maximum_money: Int = 75;

    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("WeaponChance") {
                reader.parse_percent_to_real(&mut weapon_chance)
            } else if field.eq_ignore_ascii_case("LevelChance") {
                reader.parse_percent_to_real(&mut level_chance)
            } else if field.eq_ignore_ascii_case("MoneyChance") {
                reader.parse_percent_to_real(&mut money_chance)
            } else if field.eq_ignore_ascii_case("MinMoney") {
                reader.parse_int(&mut minimum_money)
            } else if field.eq_ignore_ascii_case("MaxMoney") {
                reader.parse_int(&mut maximum_money)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;

    Ok(Box::new(SalvageCrateCollideModuleData {
        crate_data,
        weapon_chance,
        level_chance,
        money_chance,
        minimum_money,
        maximum_money,
    }))
}

#[derive(Debug, Clone)]
pub struct SalvageCrateCollide {
    base: CrateCollide,
    weapon_chance: Real,
    level_chance: Real,
    minimum_money: Int,
    maximum_money: Int,
}

impl SalvageCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SalvageCrateCollideModuleData>()
            .expect("SalvageCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            weapon_chance: typed.weapon_chance,
            level_chance: typed.level_chance,
            minimum_money: typed.minimum_money,
            maximum_money: typed.maximum_money,
        })
    }

    fn test_armor_set_flag(sim: &dyn SimView, other: ObjectSlot, flag: ArmorSetType) -> Bool {
        let Some(obj) = sim.object(other) else {
            return FALSE;
        };
        if let Some(bs) = obj.body_slot {
            if let ModuleSidecar::Body { armor_set_bits, .. } = sim.module_sidecar(bs) {
                return armor_set_flags_from_u8(armor_set_bits).test(flag.as_i32());
            }
        }
        obj.armor_set_flags.test(flag.as_i32())
    }

    fn eligible_for_armor_set(sim: &dyn SimView, other: ObjectSlot) -> Bool {
        if sim.object_is_kind_of(other, KindOfType::ArmorSalvager) == FALSE {
            return FALSE;
        }
        if Self::test_armor_set_flag(sim, other, ArmorSetType::CrateUpgradeTwo) != FALSE {
            return FALSE;
        }
        TRUE
    }

    fn eligible_for_weapon_set(sim: &dyn SimView, other: ObjectSlot) -> Bool {
        if sim.object_is_kind_of(other, KindOfType::WeaponSalvager) == FALSE {
            return FALSE;
        }
        let Some(obj) = sim.object(other) else {
            return FALSE;
        };
        if obj
            .weapon_set_flags
            .test(WeaponSetType::CrateUpgradeTwo.as_i32())
            != FALSE
        {
            return FALSE;
        }
        TRUE
    }

    fn eligible_for_level(sim: &dyn SimView, other: ObjectSlot) -> Bool {
        if sim.object_veterancy_level(other) == VeterancyLevel::Heroic {
            return FALSE;
        }
        if sim.object_is_trainable(other) == FALSE {
            return FALSE;
        }
        TRUE
    }

    fn test_weapon_chance(&self, sim: &mut dyn SimApi) -> Bool {
        if self.weapon_chance == 1.0_f32 {
            return TRUE;
        }
        let r = sim.rng_logic_real(0.0, 1.0);
        Bool::from(r < self.weapon_chance)
    }

    fn test_level_chance(&self, sim: &mut dyn SimApi) -> Bool {
        if self.level_chance == 1.0_f32 {
            return TRUE;
        }
        let r = sim.rng_logic_real(0.0, 1.0);
        Bool::from(r < self.level_chance)
    }

    fn do_armor_set(sim: &mut dyn SimApi, other: ObjectSlot) {
        if Self::test_armor_set_flag(sim, other, ArmorSetType::CrateUpgradeOne) != FALSE {
            sim.set_armor_set_flag(other, ArmorSetType::CrateUpgradeOne, FALSE);
            sim.set_armor_set_flag(other, ArmorSetType::CrateUpgradeTwo, TRUE);
            let mut clear = ModelConditionFlags::new();
            clear.set(ModelConditionFlagType::ArmorSetCrateUpgradeOne.as_i32(), 1);
            let mut set = ModelConditionFlags::new();
            set.set(ModelConditionFlagType::ArmorSetCrateUpgradeTwo.as_i32(), 1);
            sim.clear_and_set_model_condition_flags(other, &clear, &set);
        } else {
            sim.set_armor_set_flag(other, ArmorSetType::CrateUpgradeOne, TRUE);
            sim.set_model_condition_state(other, ModelConditionFlagType::ArmorSetCrateUpgradeOne);
        }
    }

    fn do_weapon_set(sim: &mut dyn SimApi, other: ObjectSlot) {
        let has_one = sim.object(other).is_some_and(|o| {
            o.weapon_set_flags
                .test(WeaponSetType::CrateUpgradeOne.as_i32())
                != FALSE
        });
        if has_one {
            sim.set_weapon_set_flag(other, WeaponSetType::CrateUpgradeOne, FALSE);
            sim.set_weapon_set_flag(other, WeaponSetType::CrateUpgradeTwo, TRUE);
        } else {
            sim.set_weapon_set_flag(other, WeaponSetType::CrateUpgradeOne, TRUE);
        }
    }

    fn do_money(&self, sim: &mut dyn SimApi, other: ObjectSlot) {
        let money = if self.minimum_money != self.maximum_money {
            sim.rng_logic_int(self.minimum_money, self.maximum_money)
        } else {
            self.minimum_money
        };
        if money > 0 {
            if let Some(other_team) = sim.object(other).map(|o| o.team_id) {
                let _ = sim.add_player_money(other_team, money);
            }
        }
    }
}

impl CrateCollideImpl for SalvageCrateCollide {
    fn crate_base(&self) -> &CrateCollide {
        &self.base
    }

    fn crate_base_mut(&mut self) -> &mut CrateCollide {
        &mut self.base
    }

    fn is_valid_to_execute(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
    ) -> Bool {
        if self.base.is_valid_to_execute(sim, ctx, other) == FALSE {
            return FALSE;
        }
        let Some(other_slot) = other else {
            return FALSE;
        };
        sim.object_is_kind_of(other_slot, KindOfType::Salvager)
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool {
        if Self::eligible_for_armor_set(sim, other) != FALSE {
            Self::do_armor_set(sim, other);
        } else if Self::eligible_for_weapon_set(sim, other) != FALSE
            && self.test_weapon_chance(sim) != FALSE
        {
            Self::do_weapon_set(sim, other);
        } else if Self::eligible_for_level(sim, other) != FALSE
            && self.test_level_chance(sim) != FALSE
        {
            let _ = gain_exp_for_level(sim, other, 1, TRUE);
        } else {
            self.do_money(sim, other);
        }
        TRUE
    }

    fn is_salvage_crate_collide(&self) -> Bool {
        TRUE
    }
}

impl_crate_collide_behavior!(SalvageCrateCollide);
