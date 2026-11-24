//! `PKT-230`: `RebuildHoleExposeDie` (`RebuildHoleExposeDie.h` / `RebuildHoleExposeDie.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{
    impl_die_behavior, parse_die_data_with, DieBehaviorImpl, DieModule, DieMuxData,
};
use crate::modules::module::{BehaviorModule, DieFacet, MaxHealthChangeType, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::{test_object_status_mask, ObjectStatusType};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, Real, FALSE, TRUE};

/// Template configuration parsed for `RebuildHoleExposeDie`.
#[derive(Debug, Clone, PartialEq)]
pub struct RebuildHoleExposeDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub hole_name: String,
    pub hole_max_health: Real,
    pub transfer_attackers: Bool,
}

impl ModuleData for RebuildHoleExposeDieModuleData {
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
        FacetMask::DIE
    }
}

pub fn parse_rebuild_hole_expose_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut hole_name = String::new();
    let mut hole_max_health: Real = 0.0_f32;
    let mut transfer_attackers: Bool = TRUE;
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("HoleName") {
                reader.parse_ascii_string(&mut hole_name)
            } else if field.eq_ignore_ascii_case("HoleMaxHealth") {
                reader.parse_real(&mut hole_max_health)
            } else if field.eq_ignore_ascii_case("TransferAttackers") {
                reader.parse_bool(&mut transfer_attackers)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(RebuildHoleExposeDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        hole_name,
        hole_max_health,
        transfer_attackers,
    }))
}

#[derive(Debug, Clone)]
pub struct RebuildHoleExposeDie {
    base: DieModule,
    pub hole_name: String,
    pub hole_max_health: Real,
    pub transfer_attackers: Bool,
}

impl RebuildHoleExposeDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<RebuildHoleExposeDieModuleData>()
            .expect("RebuildHoleExposeDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            hole_name: typed.hole_name.clone(),
            hole_max_health: typed.hole_max_health,
            transfer_attackers: typed.transfer_attackers,
        })
    }
}

impl DieBehaviorImpl for RebuildHoleExposeDie {
    fn base(&self) -> &DieModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut DieModule {
        &mut self.base
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.base.is_die_applicable(sim, ctx, dmg) == FALSE {
            return;
        }
        let Some((team_id, status, pos, angle, geom)) = sim.object(ctx.owner_slot).map(|o| {
            (
                o.team_id,
                o.status.clone(),
                *o.thing.get_position(),
                o.thing.get_orientation(),
                o.geometry.clone(),
            )
        }) else {
            return;
        };

        if sim.is_neutral_controlled(ctx.owner_slot) != FALSE
            || sim.player_is_active(team_id) == FALSE
            || test_object_status_mask(&status, ObjectStatusType::UnderConstruction) != FALSE
        {
            return;
        }

        let Some(hole_tmpl_id) = sim.rules().thing_factory.find_template_id(&self.hole_name)
        else {
            return;
        };
        let hole_slot = sim.spawn_object(hole_tmpl_id, team_id, pos, angle);
        sim.set_geometry_info(hole_slot, &geom);
        sim.set_max_health(
            hole_slot,
            self.hole_max_health,
            MaxHealthChangeType::SameCurrentHealth,
        );
        sim.notify_rebuild_hole_exposed(ctx.owner_slot, hole_slot, self.hole_max_health);
    }
}

impl_die_behavior!(RebuildHoleExposeDie);
