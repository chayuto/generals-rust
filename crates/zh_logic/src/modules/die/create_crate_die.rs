//! `PKT-205`: `CreateCrateDie` (`CreateCrateDie.h` / `CreateCrateDie.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{
    impl_die_behavior, parse_die_data_with, DieBehaviorImpl, DieModule, DieMuxData,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::kind_of::KindOfMaskType;
use crate::rules::science::SCIENCE_INVALID;
use crate::sim::{SimApi, SimView};
use crate::spatial::crate_system::{CrateTemplate, VETERANCY_LEVEL_INVALID};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, Relationship, FALSE};

/// Template configuration parsed for `CreateCrateDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateCrateDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub crate_names: Vec<String>,
}

impl ModuleData for CreateCrateDieModuleData {
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

pub fn parse_create_crate_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut crate_names = Vec::new();
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("CrateData") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                crate_names.push(tok.to_owned());
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CreateCrateDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        crate_names,
    }))
}

#[derive(Debug, Clone)]
pub struct CreateCrateDie {
    base: DieModule,
    crate_names: Vec<String>,
}

impl CreateCrateDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CreateCrateDieModuleData>()
            .expect("CreateCrateDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            crate_names: typed.crate_names.clone(),
        })
    }

    fn test_creation_chance(sim: &mut dyn SimApi, tmpl: &CrateTemplate) -> Bool {
        let roll = sim.rng_logic_real(0.0, 1.0);
        Bool::from(roll < tmpl.creation_chance)
    }

    fn test_veterancy_level(sim: &dyn SimView, owner: ObjectSlot, tmpl: &CrateTemplate) -> Bool {
        Bool::from(sim.object_veterancy_level(owner).as_i32() == tmpl.veterancy_level)
    }

    fn test_killer_type(
        sim: &dyn SimView,
        killer: Option<ObjectSlot>,
        tmpl: &CrateTemplate,
    ) -> Bool {
        let Some(ks) = killer else {
            return FALSE;
        };
        let none = KindOfMaskType::new();
        sim.object_is_kind_of_multi(ks, &tmpl.killed_by_type_kindof, &none)
    }

    fn test_killer_science(
        sim: &dyn SimView,
        killer: Option<ObjectSlot>,
        tmpl: &CrateTemplate,
    ) -> Bool {
        let Some(ks) = killer else {
            return FALSE;
        };
        let Some(killer_team) = sim.object(ks).map(|o| o.team_id) else {
            return FALSE;
        };
        sim.player_has_science(killer_team, tmpl.killer_science)
    }

    fn create_crate(
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        killer: Option<ObjectSlot>,
        tmpl: &CrateTemplate,
    ) {
        let pick = sim.rng_logic_real(0.0, 1.0);
        let mut running_total = 0.0_f32;
        let mut chosen_name = String::new();
        for entry in &tmpl.possible_crates {
            running_total += entry.crate_chance;
            if running_total > pick {
                chosen_name = entry.crate_name.clone();
                break;
            }
        }
        if chosen_name.is_empty() {
            return;
        }
        if sim
            .rules()
            .thing_factory
            .find_template_id(&chosen_name)
            .is_some()
        {
            sim.create_crate_at(&chosen_name, killer, owner);
        }
    }
}

impl DieBehaviorImpl for CreateCrateDie {
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

        let killer = sim.find_object(dmg.input.source_id);
        if let Some(ks) = killer {
            if sim.player_relationship(ks, ctx.owner_slot) == Relationship::Allies {
                return;
            }
        }

        let templates: Vec<Option<CrateTemplate>> = self
            .crate_names
            .iter()
            .map(|name| sim.rules().crate_system.find_crate_template(name).cloned())
            .collect();

        for tmpl_opt in templates {
            let Some(tmpl) = tmpl_opt else {
                continue;
            };
            if Self::test_creation_chance(sim, &tmpl) == FALSE {
                continue;
            }
            if tmpl.veterancy_level != VETERANCY_LEVEL_INVALID
                && Self::test_veterancy_level(sim, ctx.owner_slot, &tmpl) == FALSE
            {
                continue;
            }
            if tmpl.killed_by_type_kindof.any() != FALSE
                && Self::test_killer_type(sim, killer, &tmpl) == FALSE
            {
                continue;
            }
            if tmpl.killer_science != SCIENCE_INVALID
                && Self::test_killer_science(sim, killer, &tmpl) == FALSE
            {
                continue;
            }
            Self::create_crate(sim, ctx.owner_slot, killer, &tmpl);
        }
    }
}

impl_die_behavior!(CreateCrateDie);
