//! `PKT-243`: `SpecialPowerCompletionDie`
//! (`SpecialPowerCompletionDie.h` / `SpecialPowerCompletionDie.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{parse_die_data_with, DieModule, DieMuxData};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::special_power_types::SpecialPowerType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, ObjectId, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `SpecialPowerCompletionDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecialPowerCompletionDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub special_power_name: String,
    pub special_power_type: SpecialPowerType,
}

impl ModuleData for SpecialPowerCompletionDieModuleData {
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

pub fn parse_special_power_completion_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut special_power_name = String::new();
    let mut special_power_type = SpecialPowerType::Invalid;
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("SpecialPowerTemplate") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                if tok.eq_ignore_ascii_case("None") {
                    special_power_name.clear();
                    special_power_type = SpecialPowerType::Invalid;
                } else {
                    special_power_name = tok.to_owned();
                    special_power_type = ctx
                        .special_power_store
                        .find_special_power_template(tok)
                        .map(|t| t.special_power_type())
                        .unwrap_or(SpecialPowerType::Invalid);
                }
                Ok(())
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(SpecialPowerCompletionDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        special_power_name,
        special_power_type,
    }))
}

#[derive(Debug, Clone)]
pub struct SpecialPowerCompletionDie {
    base: DieModule,
    pub special_power_name: String,
    pub special_power_type: SpecialPowerType,
    creator_id: ObjectId,
    creator_set: Bool,
}

impl SpecialPowerCompletionDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SpecialPowerCompletionDieModuleData>()
            .expect("SpecialPowerCompletionDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            special_power_name: typed.special_power_name.clone(),
            special_power_type: typed.special_power_type,
            creator_id: ObjectId::INVALID,
            creator_set: FALSE,
        })
    }

    #[inline]
    pub const fn creator_id(&self) -> ObjectId {
        self.creator_id
    }

    #[inline]
    pub const fn is_creator_set(&self) -> Bool {
        self.creator_set
    }
}

impl BehaviorModule for SpecialPowerCompletionDie {
    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.base.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer(xfer)?;
        xfer.xfer_object_id(&mut self.creator_id)?;
        xfer.xfer_bool(&mut self.creator_set)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

impl DieFacet for SpecialPowerCompletionDie {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.base.is_die_applicable(sim, ctx, dmg) == FALSE {
            return;
        }
        self.notify_script_engine(sim, ctx);
    }

    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.base.is_die_applicable(sim, ctx, dmg)
    }

    fn set_creator(&mut self, creator_id: ObjectId) {
        if self.creator_set == FALSE {
            self.creator_set = TRUE;
            self.creator_id = creator_id;
        }
    }

    fn notify_script_engine(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        if self.creator_id != ObjectId::INVALID {
            sim.notify_special_power_complete(self.creator_id, self.special_power_type);
        }
    }
}
