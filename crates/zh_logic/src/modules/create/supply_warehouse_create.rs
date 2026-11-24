//! `PKT-188`: `SupplyWarehouseCreate` (`SupplyWarehouseCreate.h` / `SupplyWarehouseCreate.cpp`).

use crate::arena::ModuleContext;
use crate::modules::create::create_module::{CreateModuleBase, CreateModuleData};
use crate::modules::module::{BehaviorModule, CreateFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader};
use zh_common::types::{Bool, NameKeyType};
use zh_common::xfer::{Xfer, XferError};

pub fn parse_supply_warehouse_create_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    ini.read_block_fields(|_field, _reader| Err(IniError::UnknownToken))?;
    Ok(Box::new(CreateModuleData {
        tag_key,
        module_name_key,
    }))
}

#[derive(Debug, Clone)]
pub struct SupplyWarehouseCreate {
    base: CreateModuleBase,
}

impl SupplyWarehouseCreate {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        Box::new(Self {
            base: CreateModuleBase::new(data.module_name_key()),
        })
    }
}

impl BehaviorModule for SupplyWarehouseCreate {
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

impl CreateFacet for SupplyWarehouseCreate {
    fn on_create(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        sim.register_supply_warehouse(ctx.owner_slot);
    }

    fn on_build_complete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.base.on_build_complete();
    }

    fn should_do_on_build_complete(&self) -> Bool {
        self.base.should_do_on_build_complete()
    }
}
