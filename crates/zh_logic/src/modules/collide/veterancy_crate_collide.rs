//! `PKT-254`: `VeterancyCrateCollide` (`VeterancyCrateCollide.h` / `VeterancyCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::locomotor_set::LocomotorSetType;
use crate::modules::collide::crate_collide::{
    gain_exp_for_level, impl_crate_collide_behavior, parse_crate_collide_data_with, CrateCollide,
    CrateCollideData, CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::LocomotorSurfaceType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, FALSE, TRUE};

/// Template configuration parsed for `VeterancyCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct VeterancyCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub range_of_effect: UnsignedInt,
    pub adds_owner_veterancy: Bool,
    pub is_pilot: Bool,
}

impl ModuleData for VeterancyCrateCollideModuleData {
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

pub fn parse_veterancy_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut range_of_effect: UnsignedInt = 0;
    let mut adds_owner_veterancy = FALSE;
    let mut is_pilot = FALSE;

    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("EffectRange") {
                reader.parse_unsigned_int(&mut range_of_effect)
            } else if field.eq_ignore_ascii_case("AddsOwnerVeterancy") {
                reader.parse_bool(&mut adds_owner_veterancy)
            } else if field.eq_ignore_ascii_case("IsPilot") {
                reader.parse_bool(&mut is_pilot)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;

    Ok(Box::new(VeterancyCrateCollideModuleData {
        crate_data,
        range_of_effect,
        adds_owner_veterancy,
        is_pilot,
    }))
}

#[derive(Debug, Clone)]
pub struct VeterancyCrateCollide {
    base: CrateCollide,
    range_of_effect: UnsignedInt,
    adds_owner_veterancy: Bool,
    is_pilot: Bool,
}

impl VeterancyCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<VeterancyCrateCollideModuleData>()
            .expect("VeterancyCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
            range_of_effect: typed.range_of_effect,
            adds_owner_veterancy: typed.adds_owner_veterancy,
            is_pilot: typed.is_pilot,
        })
    }

    fn get_levels_to_gain(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int {
        if self.adds_owner_veterancy == FALSE {
            1
        } else {
            sim.object_veterancy_level(ctx.owner_slot).as_i32()
        }
    }

    fn is_using_airborne_locomotor(sim: &dyn SimView, slot: ObjectSlot) -> Bool {
        let Some(obj) = sim.object(slot) else {
            return FALSE;
        };
        if let Some(ps) = obj.physics_slot {
            if let ModuleSidecar::Physics { airborne, .. } = sim.module_sidecar(ps) {
                if airborne != FALSE {
                    return TRUE;
                }
            }
        }
        let loco_set = if let Some(ai_slot) = obj.ai_slot {
            if let ModuleSidecar::Ai {
                cur_locomotor_set, ..
            } = sim.module_sidecar(ai_slot)
            {
                LocomotorSetType::from_i32(cur_locomotor_set as Int)
                    .unwrap_or(obj.cur_locomotor_set)
            } else {
                obj.cur_locomotor_set
            }
        } else {
            return FALSE;
        };
        let Some(tmpl) = obj
            .thing
            .template_override()
            .get_final(&sim.rules().thing_factory)
        else {
            return FALSE;
        };
        let air_mask = LocomotorSurfaceType::Air.as_i32();
        for name in tmpl.get_locomotor_set(loco_set) {
            if let Some(lt) = sim.rules().locomotor_store.find_by_name(name) {
                if (lt.surfaces & air_mask) != 0 {
                    return TRUE;
                }
            }
        }
        FALSE
    }
}

impl CrateCollideImpl for VeterancyCrateCollide {
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
        if sim.object_is_significantly_above_terrain(other_slot) != FALSE {
            return FALSE;
        }
        let levels_to_gain = self.get_levels_to_gain(sim, ctx);
        if levels_to_gain <= 0 {
            return FALSE;
        }
        if other_obj.experience_tracker.is_trainable() == FALSE
            || other_obj
                .experience_tracker
                .can_gain_exp_for_level(levels_to_gain)
                == FALSE
        {
            return FALSE;
        }
        if self.is_pilot != FALSE {
            let Some(owner_obj) = sim.object(ctx.owner_slot) else {
                return FALSE;
            };
            if other_obj.team_id != owner_obj.team_id {
                return FALSE;
            }
            if Self::is_using_airborne_locomotor(sim, other_slot) != FALSE {
                return FALSE;
            }
        }
        TRUE
    }

    fn execute_crate_behavior(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: ObjectSlot,
    ) -> Bool {
        if sim.object_has_ai(ctx.owner_slot) == FALSE
            || sim.object_ai_goal_object(ctx.owner_slot) != Some(other)
        {
            return FALSE;
        }
        let levels_to_gain = self.get_levels_to_gain(sim, ctx);
        let can_scale = Bool::from(self.is_pilot == FALSE);
        if self.range_of_effect == 0 {
            let _ = gain_exp_for_level(sim, other, levels_to_gain, can_scale);
        } else {
            let Some((other_pos, other_team)) = sim
                .object(other)
                .map(|o| (*o.thing.get_position(), o.team_id))
            else {
                return FALSE;
            };
            let candidates = sim.find_objects_in_range(&other_pos, self.range_of_effect as Real);
            for slot in candidates {
                if slot == other {
                    continue;
                }
                let same_player = sim.object(slot).is_some_and(|o| o.team_id == other_team);
                if same_player {
                    let _ = gain_exp_for_level(sim, slot, levels_to_gain, can_scale);
                }
            }
        }
        TRUE
    }
}

impl_crate_collide_behavior!(VeterancyCrateCollide);
