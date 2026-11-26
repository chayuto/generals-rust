//! `PKT-202`: `ConvertToHijackedVehicleCrateCollide`
//! (`ConvertToHijackedVehicleCrateCollide.h` / `ConvertToHijackedVehicleCrateCollide.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::modules::collide::crate_collide::{
    impl_crate_collide_behavior, object_shroud_clearing_range, parse_crate_collide_data_with,
    CrateCollide, CrateCollideData, CrateCollideImpl,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Relationship, UnsignedInt, FALSE, TRUE};

/// Template configuration parsed for `ConvertToHijackedVehicleCrateCollide`.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvertToHijackedVehicleCrateCollideModuleData {
    pub crate_data: CrateCollideData,
    pub range_of_effect: UnsignedInt,
}

impl ModuleData for ConvertToHijackedVehicleCrateCollideModuleData {
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

pub fn parse_convert_to_hijacked_vehicle_crate_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let crate_data = parse_crate_collide_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |_field, _reader, _ctx| Err(IniError::UnknownToken),
    )?;
    Ok(Box::new(ConvertToHijackedVehicleCrateCollideModuleData {
        crate_data,
        range_of_effect: 0,
    }))
}

#[derive(Debug, Clone)]
pub struct ConvertToHijackedVehicleCrateCollide {
    base: CrateCollide,
}

impl ConvertToHijackedVehicleCrateCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ConvertToHijackedVehicleCrateCollideModuleData>()
            .expect("ConvertToHijackedVehicleCrateCollideModuleData mismatch");
        Box::new(Self {
            base: CrateCollide::new(typed.crate_data.clone()),
        })
    }
}

impl CrateCollideImpl for ConvertToHijackedVehicleCrateCollide {
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
        if sim.object_is_kind_of(other_slot, KindOfType::ImmuneToCapture) != FALSE
            || sim.object_is_kind_of(other_slot, KindOfType::Aircraft) != FALSE
            || sim.object_is_kind_of(other_slot, KindOfType::Boat) != FALSE
            || sim.object_is_kind_of(other_slot, KindOfType::Drone) != FALSE
        {
            return FALSE;
        }
        if other_obj.status.test(ObjectStatusType::Hijacked.as_i32()) != FALSE {
            return FALSE;
        }
        if sim.player_relationship(ctx.owner_slot, other_slot) != Relationship::Enemies {
            return FALSE;
        }
        if sim.object_is_kind_of(other_slot, KindOfType::Transport) != FALSE {
            if let Some(contain_slot) = other_obj.contain_slot {
                if let ModuleSidecar::Contain { count, .. } = sim.module_sidecar(contain_slot) {
                    if count > 0 {
                        return FALSE;
                    }
                }
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
        if sim.object_has_ai(ctx.owner_slot) != FALSE
            && sim.object_ai_goal_object(ctx.owner_slot) != Some(other)
        {
            return FALSE;
        }

        sim.hijack_vehicle(other, ctx.owner_slot);

        let target_vet = sim.object_veterancy_level(other);
        let jacker_vet = sim.object_veterancy_level(ctx.owner_slot);
        let highest = if target_vet.as_i32() >= jacker_vet.as_i32() {
            target_vet
        } else {
            jacker_vet
        };
        sim.set_veterancy_level(ctx.owner_slot, highest, FALSE, FALSE);
        sim.set_veterancy_level(other, highest, FALSE, FALSE);

        let target_can_eject = sim.object(other).is_some_and(|o| {
            o.behaviors
                .iter()
                .any(|(_, mask)| mask.contains(FacetMask::EJECT_PILOT_DIE))
        });
        if !target_can_eject {
            sim.destroy_object(ctx.owner_slot);
            return TRUE;
        }

        let has_hijacker_update = sim
            .rules()
            .name_keys
            .lookup_key("HijackerUpdate")
            .is_some_and(|k| sim.has_module_by_name_key(ctx.owner_slot, k));
        if has_hijacker_update {
            sim.set_status(ctx.owner_slot, ObjectStatusType::NoCollisions);
            sim.set_status(ctx.owner_slot, ObjectStatusType::Masked);
            sim.set_status(ctx.owner_slot, ObjectStatusType::Unselectable);
        }

        let shroud_range = object_shroud_clearing_range(sim, ctx.owner_slot);
        sim.set_shroud_clearing_range(other, shroud_range);
        sim.set_in_partition(ctx.owner_slot, FALSE);
        FALSE
    }

    fn is_hijacked_vehicle_crate_collide(&self) -> Bool {
        TRUE
    }
}

impl_crate_collide_behavior!(ConvertToHijackedVehicleCrateCollide);
