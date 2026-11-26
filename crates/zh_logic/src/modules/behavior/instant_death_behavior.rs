//! `PKT-276`: `InstantDeathBehaviorModuleData` and `InstantDeathBehavior`
//! (`InstantDeathBehavior.h`, `InstantDeathBehavior.cpp`).

use crate::object::weapon_fire;
use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version,
};
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData, ModuleSnapshotHeader};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, NameKeyType, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// INI template data matching `InstantDeathBehaviorModuleData` (`InstantDeathBehavior.h`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstantDeathBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub fx: Vec<Option<String>>,
    pub ocls: Vec<Option<String>>,
    pub weapons: Vec<Option<String>>,
}

impl InstantDeathBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            die_mux: DieMuxData::new(),
            fx: Vec::new(),
            ocls: Vec::new(),
            weapons: Vec::new(),
        }
    }
}

impl ModuleData for InstantDeathBehaviorModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x002)
    }
}

/// Runtime instance of `InstantDeathBehavior` (`PKT-276`).
pub struct InstantDeathBehavior {
    pub data: InstantDeathBehaviorModuleData,
}

impl InstantDeathBehavior {
    pub fn new(data: InstantDeathBehaviorModuleData) -> Self {
        Self { data }
    }
}

impl BehaviorModule for InstantDeathBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        ModuleSnapshotHeader::xfer_die_module_header(xfer)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

impl DieFacet for InstantDeathBehavior {
    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.data.die_mux.is_die_applicable(sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        let owner = ctx.owner_slot;
        if self.data.die_mux.is_die_applicable(sim, owner, dmg) == FALSE {
            return;
        }

        #[cfg(feature = "retail-crc")]
        {
            if sim.object(owner).is_some_and(|o| o.ai_slot.is_some()) {
                if sim.ai_is_in_dead_state(owner) != FALSE {
                    return;
                }
                sim.ai_mark_as_dead(owner);
            }
        }

        let (pos, under_construction) = sim
            .object(owner)
            .map(|o| {
                (
                    *o.thing.get_position(),
                    o.status.test(ObjectStatusType::UnderConstruction as Int) != FALSE,
                )
            })
            .unwrap_or_default();

        if !self.data.fx.is_empty() {
            let idx = sim.rng_logic_int(0, (self.data.fx.len() as Int) - 1) as usize;
            if let Some(ref fx_name) = self.data.fx[idx] {
                sim.trigger_fx_list(fx_name, &pos);
            }
        }

        if !self.data.ocls.is_empty() {
            let idx = sim.rng_logic_int(0, (self.data.ocls.len() as Int) - 1) as usize;
            if let Some(ref ocl_name) = self.data.ocls[idx] {
                sim.spawn_ocl(ocl_name, owner, &pos, &pos);
            }
        }

        #[cfg(not(feature = "retail-crc"))]
        let fire_weapons = !under_construction;
        #[cfg(feature = "retail-crc")]
        let fire_weapons = true;
        #[cfg(feature = "retail-crc")]
        let _ = under_construction;

        if fire_weapons && !self.data.weapons.is_empty() {
            let idx = sim.rng_logic_int(0, (self.data.weapons.len() as Int) - 1) as usize;
            if let Some(ref wpn_name) = self.data.weapons[idx] {
                weapon_fire::create_and_fire_temp_weapon(sim, owner, wpn_name, &pos);
            }
        }

        #[cfg(not(feature = "retail-crc"))]
        {
            if sim.object(owner).is_some_and(|o| o.ai_slot.is_some()) {
                if sim.ai_is_in_dead_state(owner) != FALSE {
                    return;
                }
                sim.ai_mark_as_dead(owner);
            }
        }

        sim.destroy_object(owner);
    }
}

pub fn parse_instant_death_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = InstantDeathBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("FX") {
            let first = r.next_token(SEPS_DEFAULT)?;
            let mut cur = Some(first);
            while let Some(tok) = cur {
                let asset = if tok.eq_ignore_ascii_case("None") {
                    None
                } else {
                    resolve_optional_asset_name(tok, parse_ctx.fx_list_names)
                };
                data.fx.push(asset);
                cur = r.next_token_or_null(SEPS_DEFAULT);
            }
            Ok(())
        } else if field.eq_ignore_ascii_case("OCL") {
            let first = r.next_token(SEPS_DEFAULT)?;
            let mut cur = Some(first);
            while let Some(tok) = cur {
                let asset = if tok.eq_ignore_ascii_case("None") {
                    None
                } else {
                    resolve_optional_asset_name(tok, parse_ctx.ocl_names)
                };
                data.ocls.push(asset);
                cur = r.next_token_or_null(SEPS_DEFAULT);
            }
            Ok(())
        } else if field.eq_ignore_ascii_case("Weapon") {
            let first = r.next_token(SEPS_DEFAULT)?;
            let mut cur = Some(first);
            while let Some(tok) = cur {
                let weapon = if tok.eq_ignore_ascii_case("None") {
                    None
                } else {
                    parse_ctx
                        .weapon_store
                        .find_weapon_template(tok)
                        .map(|wt| wt.name.clone())
                };
                data.weapons.push(weapon);
                cur = r.next_token_or_null(SEPS_DEFAULT);
            }
            Ok(())
        } else if let Some(res) = data.die_mux.parse_field(field, r) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_instant_death_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<InstantDeathBehaviorModuleData>()
        .expect("InstantDeathBehavior data type mismatch")
        .clone();
    Box::new(InstantDeathBehavior::new(typed))
}
