//! `PKT-213`: `EjectPilotDie` (`EjectPilotDie.h` / `EjectPilotDie.cpp`).

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{parse_die_data_with, DieModule, DieMuxData};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, NameKeyType, UnsignedInt, FALSE};
use zh_common::xfer::{Xfer, XferError};

/// Template configuration parsed for `EjectPilotDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EjectPilotDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub ocl_in_air: Option<String>,
    pub ocl_on_ground: Option<String>,
    pub invulnerable_time: UnsignedInt,
}

impl ModuleData for EjectPilotDieModuleData {
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
        FacetMask(FacetMask::DIE.0 | FacetMask::EJECT_PILOT_DIE.0)
    }
}

fn resolve_ocl(tok: &str, ctx: &ModuleParseContext<'_>) -> Option<String> {
    if tok.eq_ignore_ascii_case("None") {
        None
    } else {
        Some(
            ctx.ocl_names
                .iter()
                .find(|n| n.eq_ignore_ascii_case(tok))
                .cloned()
                .unwrap_or_else(|| tok.to_owned()),
        )
    }
}

pub fn parse_eject_pilot_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut ocl_in_air: Option<String> = None;
    let mut ocl_on_ground: Option<String> = None;
    let mut invulnerable_time: UnsignedInt = 0;
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("AirCreationList") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                ocl_in_air = resolve_ocl(tok, ctx);
                Ok(())
            } else if field.eq_ignore_ascii_case("GroundCreationList") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                ocl_on_ground = resolve_ocl(tok, ctx);
                Ok(())
            } else if field.eq_ignore_ascii_case("InvulnerableTime") {
                reader.parse_duration_unsigned_int(&mut invulnerable_time)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(EjectPilotDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        ocl_in_air,
        ocl_on_ground,
        invulnerable_time,
    }))
}

#[derive(Debug, Clone)]
pub struct EjectPilotDie {
    base: DieModule,
    pub ocl_in_air: Option<String>,
    pub ocl_on_ground: Option<String>,
    pub invulnerable_time: UnsignedInt,
}

impl EjectPilotDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<EjectPilotDieModuleData>()
            .expect("EjectPilotDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            ocl_in_air: typed.ocl_in_air.clone(),
            ocl_on_ground: typed.ocl_on_ground.clone(),
            invulnerable_time: typed.invulnerable_time,
        })
    }

    fn eject_pilot(
        &self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        damage_dealer: Option<crate::arena::ObjectSlot>,
    ) {
        let ocl = if sim.object_is_significantly_above_terrain(ctx.owner_slot) != FALSE {
            self.ocl_in_air.as_deref()
        } else {
            self.ocl_on_ground.as_deref()
        };
        if let Some(ocl_name) = ocl {
            sim.create_ocl(
                ocl_name,
                crate::object::ocl_runtime::OclCreateArgs::object(ctx.owner_slot, damage_dealer),
            );
        }
    }
}

impl BehaviorModule for EjectPilotDie {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.base.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        self.base.xfer_subclass(xfer)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }
}

impl DieFacet for EjectPilotDie {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.base.is_die_applicable(sim, ctx, dmg) == FALSE {
            return;
        }
        let damage_dealer = sim.find_object(dmg.input.source_id);
        self.eject_pilot(sim, ctx, damage_dealer);
    }

    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.base.is_die_applicable(sim, ctx, dmg)
    }

    fn eject_pilot_dying(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.eject_pilot(sim, ctx, None);
    }
}
