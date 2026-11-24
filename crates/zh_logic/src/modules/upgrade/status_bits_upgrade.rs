//! `PKT-247`: `StatusBitsUpgrade` (`StatusBitsUpgrade.h` / `StatusBitsUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::rules::object_status_types::{
    ObjectStatusMaskType, ObjectStatusType, OBJECT_STATUS_COUNT, OBJECT_STATUS_NAMES,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Int, NameKeyType, FALSE};

/// Template configuration parsed for `StatusBitsUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StatusBitsUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub status_to_set: ObjectStatusMaskType,
    pub status_to_clear: ObjectStatusMaskType,
}

impl ModuleData for StatusBitsUpgradeModuleData {
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
        FacetMask::UPGRADE
    }
}

pub fn parse_status_bits_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut status_to_set = ObjectStatusMaskType::new();
    let mut status_to_clear = ObjectStatusMaskType::new();
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("StatusToSet") {
                status_to_set.parse_from_ini(reader, &OBJECT_STATUS_NAMES)
            } else if field.eq_ignore_ascii_case("StatusToClear") {
                status_to_clear.parse_from_ini(reader, &OBJECT_STATUS_NAMES)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(StatusBitsUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        status_to_set,
        status_to_clear,
    }))
}

#[derive(Debug, Clone)]
pub struct StatusBitsUpgrade {
    base: UpgradeModule,
    status_to_set: ObjectStatusMaskType,
    status_to_clear: ObjectStatusMaskType,
}

impl StatusBitsUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<StatusBitsUpgradeModuleData>()
            .expect("StatusBitsUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            status_to_set: typed.status_to_set.clone(),
            status_to_clear: typed.status_to_clear.clone(),
        })
    }
}

impl UpgradeBehaviorImpl for StatusBitsUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for idx in 0..OBJECT_STATUS_COUNT {
            let bit = idx as Int;
            if let Some(st) = ObjectStatusType::from_i32(bit) {
                if self.status_to_set.test(bit) != FALSE {
                    sim.set_status(ctx.owner_slot, st);
                }
                if self.status_to_clear.test(bit) != FALSE {
                    sim.clear_status(ctx.owner_slot, st);
                }
            }
        }
    }
}

impl_upgrade_behavior!(StatusBitsUpgrade);
