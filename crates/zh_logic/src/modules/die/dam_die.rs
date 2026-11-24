//! `PKT-208`: `DamDie` (`DamDie.h` / `DamDie.cpp`).

use crate::arena::ModuleContext;
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{
    impl_die_behavior, parse_die_module_data, DieBehaviorImpl, DieModule, DieModuleData,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{NameKeyType, FALSE};

pub type DamDieModuleData = DieModuleData;

pub fn parse_dam_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_die_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct DamDie {
    base: DieModule,
}

impl DamDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<DamDieModuleData>()
            .expect("DamDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
        })
    }
}

impl DieBehaviorImpl for DamDie {
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
        let slots = sim.world_object_slots();
        for slot in slots {
            if sim.object_is_kind_of(slot, KindOfType::WaveGuide) != FALSE {
                let _ = sim.clear_disabled(slot, DisabledType::Default);
            }
        }
        sim.trigger_dam_break(ctx.owner_slot);
    }
}

impl_die_behavior!(DamDie);
