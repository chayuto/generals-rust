//! `PKT-206`: `CreateObjectDie` (`CreateObjectDie.h` / `CreateObjectDie.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{
    impl_die_behavior, parse_die_data_with, DieBehaviorImpl, DieModule, DieMuxData,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, FALSE};

/// Template configuration parsed for `CreateObjectDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateObjectDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub ocl_name: Option<String>,
    pub transfer_previous_health: Bool,
    pub transfer_selection: Bool,
}

impl ModuleData for CreateObjectDieModuleData {
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

pub fn parse_create_object_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut ocl_name: Option<String> = None;
    let mut transfer_previous_health: Bool = FALSE;
    let mut transfer_selection: Bool = FALSE;
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("CreationList") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    ocl_name = None;
                } else {
                    let resolved = ctx
                        .ocl_names
                        .iter()
                        .find(|n| n.eq_ignore_ascii_case(tok))
                        .cloned()
                        .unwrap_or_else(|| tok.to_owned());
                    ocl_name = Some(resolved);
                }
                Ok(())
            } else if field.eq_ignore_ascii_case("TransferPreviousHealth") {
                reader.parse_bool(&mut transfer_previous_health)
            } else if field.eq_ignore_ascii_case("TransferSelection") {
                reader.parse_bool(&mut transfer_selection)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CreateObjectDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        ocl_name,
        transfer_previous_health,
        transfer_selection,
    }))
}

#[derive(Debug, Clone)]
pub struct CreateObjectDie {
    base: DieModule,
    pub ocl_name: Option<String>,
    pub transfer_previous_health: Bool,
    pub transfer_selection: Bool,
}

impl CreateObjectDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CreateObjectDieModuleData>()
            .expect("CreateObjectDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            ocl_name: typed.ocl_name.clone(),
            transfer_previous_health: typed.transfer_previous_health,
            transfer_selection: typed.transfer_selection,
        })
    }
}

impl DieBehaviorImpl for CreateObjectDie {
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
        if let Some(ref ocl_name) = self.ocl_name {
            let damage_dealer = sim.find_object(dmg.input.source_id);
            sim.create_ocl(
                ocl_name,
                crate::object::ocl_runtime::OclCreateArgs::object(ctx.owner_slot, damage_dealer),
            );
        }
    }
}

impl_die_behavior!(CreateObjectDie);
