//! `PKT-171`: `GrantUpgradeCreate` (`GrantUpgradeCreate.h` / `GrantUpgradeCreate.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::create::create_module::CreateModuleBase;
use crate::modules::module::{BehaviorModule, CreateFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType, OBJECT_STATUS_NAMES};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `GrantUpgradeCreate`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GrantUpgradeCreateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_name: String,
    pub exempt_status: ObjectStatusMaskType,
}

impl ModuleData for GrantUpgradeCreateModuleData {
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
        FacetMask::CREATE
    }
}

pub fn parse_grant_upgrade_create_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = GrantUpgradeCreateModuleData {
        tag_key,
        module_name_key,
        upgrade_name: String::new(),
        exempt_status: ObjectStatusMaskType::new(),
    };
    ini.read_block_fields(|field, reader| {
        if field.eq_ignore_ascii_case("UpgradeToGrant") {
            data.upgrade_name = reader.next_token(SEPS_DEFAULT)?.to_owned();
            Ok(())
        } else if field.eq_ignore_ascii_case("ExemptStatus") {
            data.exempt_status.parse_from_ini(reader, &OBJECT_STATUS_NAMES)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

#[derive(Debug, Clone)]
pub struct GrantUpgradeCreate {
    base: CreateModuleBase,
    data: GrantUpgradeCreateModuleData,
}

impl GrantUpgradeCreate {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<GrantUpgradeCreateModuleData>()
            .expect("GrantUpgradeCreateModuleData mismatch");
        Box::new(Self {
            base: CreateModuleBase::new(typed.module_name_key),
            data: typed.clone(),
        })
    }
}

impl BehaviorModule for GrantUpgradeCreate {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        self.base.xfer_subclass(xfer)
    }

    fn as_create_mut(&mut self) -> Option<&mut dyn CreateFacet> {
        Some(self)
    }
}

impl CreateFacet for GrantUpgradeCreate {
    fn on_create(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let under_const = ObjectStatusType::UnderConstruction.as_i32();
        if self.data.exempt_status.test(under_const) != FALSE {
            let currently_under_const = sim
                .object(ctx.owner_slot)
                .is_some_and(|o| o.status.test(under_const) != FALSE);
            if !currently_under_const {
                sim.grant_upgrade(ctx.owner_slot, &self.data.upgrade_name, TRUE);
            }
        }
    }

    fn on_build_complete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.should_do_on_build_complete() == FALSE {
            return;
        }
        self.base.on_build_complete();
        sim.grant_upgrade(ctx.owner_slot, &self.data.upgrade_name, FALSE);
    }

    fn should_do_on_build_complete(&self) -> Bool {
        self.base.should_do_on_build_complete()
    }
}
