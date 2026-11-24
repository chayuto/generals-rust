//! `PKT-252`: `UpgradeDie` (`UpgradeDie.h` / `UpgradeDie.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{
    impl_die_behavior, parse_die_data_with, DieBehaviorImpl, DieModule, DieMuxData,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, ObjectId, FALSE};

/// Template configuration parsed for `UpgradeDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpgradeDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub upgrade_name: String,
}

impl ModuleData for UpgradeDieModuleData {
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

pub fn parse_upgrade_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut upgrade_name = String::new();
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("UpgradeToRemove") {
                reader.parse_ascii_string(&mut upgrade_name)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(UpgradeDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        upgrade_name,
    }))
}

#[derive(Debug, Clone)]
pub struct UpgradeDie {
    base: DieModule,
    pub upgrade_name: String,
}

impl UpgradeDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<UpgradeDieModuleData>()
            .expect("UpgradeDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            upgrade_name: typed.upgrade_name.clone(),
        })
    }
}

impl DieBehaviorImpl for UpgradeDie {
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
        let producer_id = sim
            .object(ctx.owner_slot)
            .map(|o| o.producer_id)
            .unwrap_or(ObjectId::INVALID);
        let Some(producer_slot) = sim.find_object(producer_id) else {
            return;
        };
        let mask = sim
            .rules()
            .upgrade_center
            .lookup_upgrade(&self.upgrade_name, &sim.rules().name_keys)
            .map(|u| u.upgrade_mask().clone());
        if let Some(mask) = mask {
            if sim.object_has_upgrade(producer_slot, &mask) != FALSE {
                sim.remove_upgrade(producer_slot, &self.upgrade_name);
            }
        }
    }
}

impl_die_behavior!(UpgradeDie);
