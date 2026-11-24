//! `PKT-249`: `SubObjectsUpgrade` (`SubObjectsUpgrade.h` / `SubObjectsUpgrade.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{
    impl_upgrade_behavior, parse_upgrade_data_with, UpgradeBehaviorImpl, UpgradeModule,
    UpgradeMuxData,
};
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType, FALSE, TRUE};

/// Template configuration parsed for `SubObjectsUpgrade`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SubObjectsUpgradeModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub mux_data: UpgradeMuxData,
    pub show_sub_object_names: Vec<String>,
    pub hide_sub_object_names: Vec<String>,
}

impl ModuleData for SubObjectsUpgradeModuleData {
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

pub fn parse_sub_objects_upgrade_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut show_sub_object_names = Vec::new();
    let mut hide_sub_object_names = Vec::new();
    let base = parse_upgrade_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("ShowSubObjects") {
                reader.parse_ascii_string_vector_append(&mut show_sub_object_names)
            } else if field.eq_ignore_ascii_case("HideSubObjects") {
                reader.parse_ascii_string_vector_append(&mut hide_sub_object_names)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(SubObjectsUpgradeModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        mux_data: base.mux_data,
        show_sub_object_names,
        hide_sub_object_names,
    }))
}

#[derive(Debug, Clone)]
pub struct SubObjectsUpgrade {
    base: UpgradeModule,
    data: SubObjectsUpgradeModuleData,
}

impl SubObjectsUpgrade {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SubObjectsUpgradeModuleData>()
            .expect("SubObjectsUpgradeModuleData mismatch");
        Box::new(Self {
            base: UpgradeModule::new(typed.module_name_key, typed.mux_data.clone()),
            data: typed.clone(),
        })
    }
}

impl UpgradeBehaviorImpl for SubObjectsUpgrade {
    fn base(&self) -> &UpgradeModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut UpgradeModule {
        &mut self.base
    }

    fn is_sub_objects_upgrade(&self) -> Bool {
        TRUE
    }

    fn upgrade_implementation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let mut activation = UpgradeMaskType::new();
        let mut conflicting = UpgradeMaskType::new();
        self.base.mux_data.get_upgrade_activation_masks(
            sim.rules(),
            &mut activation,
            &mut conflicting,
        );
        if sim.object_has_upgrade(ctx.owner_slot, &conflicting) != FALSE {
            return;
        }
        if sim
            .object(ctx.owner_slot)
            .is_some_and(|o| sim.player_has_upgrade(o.team_id, &conflicting) != FALSE)
        {
            return;
        }
        sim.update_sub_objects(
            ctx.owner_slot,
            &self.data.show_sub_object_names,
            &self.data.hide_sub_object_names,
        );
    }
}

impl_upgrade_behavior!(SubObjectsUpgrade);
