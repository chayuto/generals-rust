//! `PKT-226`: `OCLSpecialPower` (`OCLSpecialPower.h` / `OCLSpecialPower.cpp`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::module::{BehaviorModule, ModuleData, SpecialPowerFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::special_power::special_power_module::{
    impl_special_power_behavior, parse_special_power_data_with, SpecialPowerBehaviorImpl,
    SpecialPowerModule, SpecialPowerModuleData,
};
use crate::rules::science::ScienceType;
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, Real, UnsignedInt, FALSE};

const CREATE_ABOVE_LOCATION_HEIGHT: Real = 300.0;
const INVALID_ANGLE: Real = crate::object::ocl_runtime::INVALID_ANGLE;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OclCreateLocType {
    #[default]
    CreateAtEdgeNearSource = 0,
    CreateAtEdgeNearTarget = 1,
    CreateAtLocation = 2,
    UseOwnerObject = 3,
    CreateAboveLocation = 4,
    CreateAtEdgeFarthestFromTarget = 5,
}

impl OclCreateLocType {
    pub const fn from_i32(v: Int) -> Option<Self> {
        match v {
            0 => Some(Self::CreateAtEdgeNearSource),
            1 => Some(Self::CreateAtEdgeNearTarget),
            2 => Some(Self::CreateAtLocation),
            3 => Some(Self::UseOwnerObject),
            4 => Some(Self::CreateAboveLocation),
            5 => Some(Self::CreateAtEdgeFarthestFromTarget),
            _ => None,
        }
    }
}

pub const OCL_CREATE_LOC_TYPE_NAMES: [&str; 6] = [
    "CREATE_AT_EDGE_NEAR_SOURCE",
    "CREATE_AT_EDGE_NEAR_TARGET",
    "CREATE_AT_LOCATION",
    "USE_OWNER_OBJECT",
    "CREATE_ABOVE_LOCATION",
    "CREATE_AT_EDGE_FARTHEST_FROM_TARGET",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OclUpgrade {
    pub science: ScienceType,
    pub ocl: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OCLSpecialPowerModuleData {
    pub base: SpecialPowerModuleData,
    pub upgrade_ocl: Vec<OclUpgrade>,
    pub default_ocl: Option<String>,
    pub create_loc: OclCreateLocType,
    pub is_ocl_adjust_position_to_passable: Bool,
    pub reference_thing_name: String,
}

impl ModuleData for OCLSpecialPowerModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.base.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::SPECIAL_POWER
    }
}

fn resolve_ocl_name(tok: &str, parse_ctx: &ModuleParseContext<'_>) -> Option<String> {
    if tok.eq_ignore_ascii_case("None") {
        None
    } else {
        Some(
            parse_ctx
                .ocl_names
                .iter()
                .find(|n| n.eq_ignore_ascii_case(tok))
                .cloned()
                .unwrap_or_else(|| tok.to_owned()),
        )
    }
}

pub fn parse_ocl_special_power_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut upgrade_ocl = Vec::new();
    let mut default_ocl: Option<String> = None;
    let mut create_loc = OclCreateLocType::CreateAtEdgeNearSource;
    let mut is_ocl_adjust_position_to_passable = FALSE;
    let mut reference_thing_name = String::new();

    let base = parse_special_power_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, ctx| {
            if field.eq_ignore_ascii_case("UpgradeOCL") {
                let sci_tok = reader.next_token(SEPS_DEFAULT)?;
                let science = ctx
                    .science_store
                    .friend_lookup_science(sci_tok, ctx.name_keys)?;
                let ocl_tok = reader.next_token(SEPS_DEFAULT)?;
                upgrade_ocl.push(OclUpgrade {
                    science,
                    ocl: resolve_ocl_name(ocl_tok, ctx),
                });
                Ok(())
            } else if field.eq_ignore_ascii_case("OCL") {
                let tok = reader.next_token(SEPS_DEFAULT)?;
                default_ocl = resolve_ocl_name(tok, ctx);
                Ok(())
            } else if field.eq_ignore_ascii_case("CreateLocation") {
                let mut raw: Int = 0;
                reader.parse_index_list(&mut raw, &OCL_CREATE_LOC_TYPE_NAMES)?;
                create_loc = OclCreateLocType::from_i32(raw).unwrap_or_default();
                Ok(())
            } else if field.eq_ignore_ascii_case("ReferenceObject") {
                reader.parse_ascii_string(&mut reference_thing_name)
            } else if field.eq_ignore_ascii_case("OCLAdjustPositionToPassable") {
                reader.parse_bool(&mut is_ocl_adjust_position_to_passable)
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;

    Ok(Box::new(OCLSpecialPowerModuleData {
        base,
        upgrade_ocl,
        default_ocl,
        create_loc,
        is_ocl_adjust_position_to_passable,
        reference_thing_name,
    }))
}

#[derive(Debug, Clone)]
pub struct OCLSpecialPower {
    base: SpecialPowerModule,
    data: OCLSpecialPowerModuleData,
}

impl OCLSpecialPower {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<OCLSpecialPowerModuleData>()
            .expect("OCLSpecialPowerModuleData mismatch");
        Box::new(Self {
            base: SpecialPowerModule::new(typed.base.clone()),
            data: typed.clone(),
        })
    }

    pub fn find_ocl(&self, sim: &dyn SimView, ctx: ModuleContext) -> Option<&str> {
        if let Some(team_id) = sim.object(ctx.owner_slot).map(|o| o.team_id) {
            for u in &self.data.upgrade_ocl {
                if sim.player_has_science(team_id, u.science) != FALSE {
                    return u.ocl.as_deref();
                }
            }
        }
        self.data.default_ocl.as_deref()
    }
}

impl SpecialPowerBehaviorImpl for OCLSpecialPower {
    fn base(&self) -> &SpecialPowerModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut SpecialPowerModule {
        &mut self.base
    }

    fn get_reference_thing_template_name(
        &self,
        _sim: &dyn SimView,
        _ctx: ModuleContext,
    ) -> Option<String> {
        if self.data.reference_thing_name.is_empty() {
            None
        } else {
            Some(self.data.reference_thing_name.clone())
        }
    }

    fn do_special_power_at_location(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        loc: &Coord3D,
        angle: Real,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        let target_coord = *loc;
        SpecialPowerFacet::do_special_power_at_location(
            &mut self.base,
            sim,
            ctx,
            &target_coord,
            angle,
            command_options,
        );
        if self.base.available_on_frame == u32::MAX {
            return;
        }

        let Some(ocl) = self.find_ocl(sim, ctx).map(str::to_owned) else {
            return;
        };
        let owner_pos = sim
            .object(ctx.owner_slot)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        let creation_coord = match self.data.create_loc {
            OclCreateLocType::CreateAtEdgeNearSource => owner_pos,
            OclCreateLocType::CreateAtEdgeNearTarget
            | OclCreateLocType::CreateAtLocation
            | OclCreateLocType::UseOwnerObject => target_coord,
            OclCreateLocType::CreateAboveLocation
            | OclCreateLocType::CreateAtEdgeFarthestFromTarget => Coord3D::new(
                target_coord.x,
                target_coord.y,
                target_coord.z + CREATE_ABOVE_LOCATION_HEIGHT,
            ),
        };
        sim.create_ocl(
            &ocl,
            crate::object::ocl_runtime::OclCreateArgs::position_angle(
                ctx.owner_slot,
                creation_coord,
                Some(target_coord),
                angle,
            ),
        );
    }

    fn do_special_power_at_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        if let Some(pos) = sim.object(target).map(|o| *o.thing.get_position()) {
            SpecialPowerFacet::do_special_power_at_location(
                self,
                sim,
                ctx,
                &pos,
                INVALID_ANGLE,
                command_options,
            );
        }
    }

    fn do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        command_options: UnsignedInt,
    ) {
        if SpecialPowerModule::is_owner_disabled(sim, ctx) {
            return;
        }
        let creation_coord = sim
            .object(ctx.owner_slot)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();
        SpecialPowerFacet::do_special_power_at_location(
            &mut self.base,
            sim,
            ctx,
            &creation_coord,
            INVALID_ANGLE,
            command_options,
        );
        if self.base.available_on_frame == u32::MAX {
            return;
        }
        if let Some(ocl) = self.find_ocl(sim, ctx).map(str::to_owned) {
            sim.create_ocl(
                &ocl,
                crate::object::ocl_runtime::OclCreateArgs::position_create_owner(
                    ctx.owner_slot,
                    creation_coord,
                    Some(creation_coord),
                    FALSE,
                ),
            );
        }
    }
}

impl_special_power_behavior!(OCLSpecialPower);
