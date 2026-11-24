//! `PKT-176`: `LockWeaponCreate` (`LockWeaponCreate.h` / `LockWeaponCreate.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::create::create_module::CreateModuleBase;
use crate::modules::module::{BehaviorModule, CreateFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::object::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType};
use zh_common::xfer::{Xfer, XferError};

const WEAPON_SLOT_LOOKUP_NAMES: [&str; 3] = ["PRIMARY", "SECONDARY", "TERTIARY"];

/// Template configuration parsed for `LockWeaponCreate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LockWeaponCreateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub slot_to_lock: WeaponSlotType,
}

impl ModuleData for LockWeaponCreateModuleData {
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

pub fn parse_lock_weapon_create_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    _parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = LockWeaponCreateModuleData {
        tag_key,
        module_name_key,
        slot_to_lock: WeaponSlotType::Primary,
    };
    ini.read_block_fields(|field, reader| {
        if field.eq_ignore_ascii_case("SlotToLock") {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            let idx = IniReader::scan_index_list(tok, &WEAPON_SLOT_LOOKUP_NAMES)?;
            data.slot_to_lock = WeaponSlotType::from_i32(idx).ok_or(IniError::InvalidData)?;
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

#[derive(Debug, Clone)]
pub struct LockWeaponCreate {
    base: CreateModuleBase,
    data: LockWeaponCreateModuleData,
}

impl LockWeaponCreate {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<LockWeaponCreateModuleData>()
            .expect("LockWeaponCreateModuleData mismatch");
        Box::new(Self {
            base: CreateModuleBase::new(typed.module_name_key),
            data: *typed,
        })
    }
}

impl BehaviorModule for LockWeaponCreate {
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

impl CreateFacet for LockWeaponCreate {
    fn on_create(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}

    fn on_build_complete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.base.on_build_complete();
        let _ = sim.lock_weapon_slot(
            ctx.owner_slot,
            self.data.slot_to_lock,
            WeaponLockType::LockedPermanently,
        );
    }

    fn should_do_on_build_complete(&self) -> Bool {
        self.base.should_do_on_build_complete()
    }
}
