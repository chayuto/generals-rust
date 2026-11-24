//! `PKT-192`: `VeterancyGainCreate` (`VeterancyGainCreate.h` / `VeterancyGainCreate.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::modules::create::create_module::CreateModuleBase;
use crate::modules::module::{BehaviorModule, CreateFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT, VETERANCY_NAMES};
use zh_common::types::{Bool, NameKeyType, VeterancyLevel, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `VeterancyGainCreate`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VeterancyGainCreateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub starting_level: VeterancyLevel,
    pub science_required: ScienceType,
}

impl Default for VeterancyGainCreateModuleData {
    fn default() -> Self {
        Self {
            tag_key: NameKeyType::INVALID,
            module_name_key: NameKeyType::INVALID,
            starting_level: VeterancyLevel::Regular,
            science_required: SCIENCE_INVALID,
        }
    }
}

impl ModuleData for VeterancyGainCreateModuleData {
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

pub fn parse_veterancy_gain_create_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = VeterancyGainCreateModuleData {
        tag_key,
        module_name_key,
        starting_level: VeterancyLevel::Regular,
        science_required: SCIENCE_INVALID,
    };
    ini.read_block_fields(|field, reader| {
        if field.eq_ignore_ascii_case("StartingLevel") {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            let idx = IniReader::scan_index_list(tok, &VETERANCY_NAMES)?;
            data.starting_level = VeterancyLevel::from_i32(idx);
            Ok(())
        } else if field.eq_ignore_ascii_case("ScienceRequired") {
            let tok = reader.next_token(SEPS_DEFAULT)?;
            data.science_required = parse_ctx
                .science_store
                .friend_lookup_science(tok, parse_ctx.name_keys)?;
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

#[derive(Debug, Clone)]
pub struct VeterancyGainCreate {
    base: CreateModuleBase,
    data: VeterancyGainCreateModuleData,
}

impl VeterancyGainCreate {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<VeterancyGainCreateModuleData>()
            .expect("VeterancyGainCreateModuleData mismatch");
        Box::new(Self {
            base: CreateModuleBase::new(typed.module_name_key),
            data: *typed,
        })
    }
}

impl BehaviorModule for VeterancyGainCreate {
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

impl CreateFacet for VeterancyGainCreate {
    fn on_create(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let has_req = sim.object(ctx.owner_slot).is_some_and(|obj| {
            self.data.science_required == SCIENCE_INVALID
                || sim.player_has_science(obj.team_id, self.data.science_required) != FALSE
        });
        let is_trainable = sim
            .object(ctx.owner_slot)
            .is_some_and(|obj| obj.experience_tracker.is_trainable() != FALSE);
        if has_req && is_trainable {
            sim.set_veterancy_level(ctx.owner_slot, self.data.starting_level, TRUE, FALSE);
        }
    }

    fn on_build_complete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.base.on_build_complete();
    }

    fn should_do_on_build_complete(&self) -> Bool {
        self.base.should_do_on_build_complete()
    }
}
