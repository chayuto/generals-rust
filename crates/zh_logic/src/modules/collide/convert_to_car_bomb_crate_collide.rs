//! `PKT-201`: `ConvertToCarBombCrateCollide`
//! (`ConvertToCarBombCrateCollide.h` / `ConvertToCarBombCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, parse_crate_collide_data_with, resolve_fx_name, CrateCollide,
    CrateCollideData, CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{WeaponSetFlags, WeaponSetType};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, FALSE, TRUE};

/// Template configuration parsed for `ConvertToCarBombCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvertToCarBombCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub range_of_effect: UnsignedInt,
    pub fx_list: Option<String>,
}

impl ModuleData for ConvertToCarBombCrateCollideModuleData {
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

pub fn parse_convert_to_car_bomb_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut fx_list: Option<String> = None;
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("FXList") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                fx_list = resolve_fx_name(tok, ctx);
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(ConvertToCarBombCrateCollideModuleData {
        crate_data,
        range_of_effect: 0,
        fx_list,
    }))
}

#[derive(Debug, Clone)]
pub struct ConvertToCarBombCrateCollide {
    base: CrateCollide,
    fx_list: Option<String>,
}

impl ConvertToCarBombCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ConvertToCarBombCrateCollideModuleData>()
            .expect("ConvertToCarBombCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            fx_list: typed.fx_list.clone(),
        })
    }
}

impl CrateCollideImpl for ConvertToCarBombCrateCollide {
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
        let Some(other_obj) = sim.object(other_slot) else {
            return FALSE;
        };
        if other_obj.is_effectively_dead() != FALSE {
            return FALSE;
        }
        if sim.object_is_kind_of(other_slot, KindOfType::Aircraft) != FALSE
            || sim.object_is_kind_of(other_slot, KindOfType::Boat) != FALSE
        {
            return FALSE;
        }
        if other_obj
            .status
            .test(ObjectStatusType::IsCarbomb.as_i32())
            != FALSE
        {
            return FALSE;
        }
        let mut flags = WeaponSetFlags::new();
        flags.set(WeaponSetType::Carbomb.as_i32(), 1);
        let Some(tmpl) = other_obj
            .thing
            .template_override()
            .get_final(&sim.rules().thing_factory)
        else {
            return FALSE;
        };
        let Some(set) = tmpl.find_weapon_template_set(&flags) else {
            return FALSE;
        };
        if set.test_weapon_set_flag(WeaponSetType::Carbomb) == FALSE {
            return FALSE;
        }
        if other_obj
            .weapon_set_flags
            .test(WeaponSetType::Carbomb.as_i32())
            != FALSE
        {
            return FALSE;
        }
        TRUE
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool {
        if sim.object_has_ai(ctx.owner_slot) != FALSE
            && sim.object_ai_goal_object(ctx.owner_slot) != Some(other)
        {
            return FALSE;
        }
        sim.convert_to_car_bomb(other, ctx.owner_slot);
        if let Some(ref fx_name) = self.fx_list {
            let pos = sim
                .object(other)
                .map(|o| *o.thing.get_position())
                .unwrap_or_default();
            sim.trigger_fx_list(fx_name, &pos);
        }
        let vision = sim.object_vision_range(ctx.owner_slot);
        sim.set_vision_range(other, vision);
        let crate_shroud = sim.object_shroud_clearing_range(ctx.owner_slot);
        sim.set_shroud_clearing_range(other, crate_shroud);
        let crate_vet = sim.object_veterancy_level(ctx.owner_slot);
        sim.set_veterancy_level(other, crate_vet, FALSE, TRUE);
        TRUE
    }

    fn is_car_bomb_crate_collide(&self) -> Bool {
        TRUE
    }
}

impl_crate_collide_behavior!(ConvertToCarBombCrateCollide);
