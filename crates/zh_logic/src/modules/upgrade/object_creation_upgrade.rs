//! `PKT-225`: `ObjectCreationUpgrade` (`ObjectCreationUpgrade.h` / `ObjectCreationUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::NameKeyType;

/// Template configuration parsed for `ObjectCreationUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjectCreationUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub ocl_name: Option<String>,
}

impl ModuleData for ObjectCreationUpgradeModuleData {
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

pub fn parse_object_creation_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut ocl_name = None;
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("UpgradeObject") {
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
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(ObjectCreationUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        ocl_name,
    }))
}

#[derive(Debug, Clone)]
pub struct ObjectCreationUpgrade {
    base: UpgradeModule,
    ocl_name: Option<String>,
}

impl ObjectCreationUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<ObjectCreationUpgradeModuleData>()
            .expect("ObjectCreationUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            ocl_name: typed.ocl_name.clone(),
        })
    }
}

impl UpgradeBehaviorImpl for ObjectCreationUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if let Some(ref ocl) = self.ocl_name {
            sim.create_ocl(
                ocl,
                crate::object::ocl_runtime::OclCreateArgs::object(ctx.owner_slot, None),
            );
        }
    }
}

impl_upgrade_behavior!(ObjectCreationUpgrade);
