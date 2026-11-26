//! `PKT-370`: `SpectreGunshipDeploymentUpdateModuleData` and
//! `SpectreGunshipDeploymentUpdate` (`SpectreGunshipDeploymentUpdate.h`,
//! `SpectreGunshipDeploymentUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-370_SpectreGunshipDeploymentUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt336_370_306_304_leaves.rs`.
//!
//! Where the reference would dereference null (no special power module for the
//! template, no gunship template, no target position, a gunship without a
//! current locomotor) the port gives up or leaves the height at the edge point.

use crate::arena::{FacetMask, ModuleContext, ModuleSlot, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    BehaviorModule, ModuleData, SpecialPowerCaller, SpecialPowerUpdateFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{DisabledMaskType, DisabledType};
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::science::{ScienceType, SCIENCE_INVALID};
use crate::sim::{SimApi, SimView};
use crate::spatial::terrain::{find_closest_edge_point, find_farthest_edge_point};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_atan2f, Coord3D};
use zh_common::types::{
    Bool, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `COMMAND_FIRED_BY_SCRIPT` (`ControlBar.h`).
pub const COMMAND_FIRED_BY_SCRIPT: UnsignedInt = 0x0004_0000;

/// `INVALID_ANGLE`.
const INVALID_ANGLE: Real = -100.0;

const CREATE_LOCATION_NAMES: [&str; 4] = [
    "CREATE_AT_EDGE_NEAR_SOURCE",
    "CREATE_AT_EDGE_FARTHEST_FROM_SOURCE",
    "CREATE_AT_EDGE_NEAR_TARGET",
    "CREATE_AT_EDGE_FARTHEST_FROM_TARGET",
];

/// `GunshipCreateLocType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GunshipCreateLocType {
    EdgeNearSource,
    EdgeFarthestFromSource,
    EdgeNearTarget,
    EdgeFarthestFromTarget,
}

/// INI template data matching `SpectreGunshipDeploymentUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpectreGunshipDeploymentUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub special_power_name: String,
    pub extra_required_science: ScienceType,
    pub gunship_template_name: String,
    /// Parsed; nothing in this module reads it.
    pub attack_area_radius: Real,
    /// Never initialised or parsed by the reference. Module data comes from
    /// zeroed memory, so it is zero.
    pub gunship_orbit_radius: Real,
    pub create_loc: GunshipCreateLocType,
}

impl SpectreGunshipDeploymentUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            special_power_name: String::new(),
            extra_required_science: SCIENCE_INVALID,
            gunship_template_name: String::new(),
            attack_area_radius: 200.0,
            gunship_orbit_radius: 0.0,
            create_loc: GunshipCreateLocType::EdgeFarthestFromTarget,
        }
    }
}

impl ModuleData for SpectreGunshipDeploymentUpdateModuleData {
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
        FacetMask(0x001 | FacetMask::SPECIAL_POWER_UPDATE.0)
    }
}

/// Runtime instance of `SpectreGunshipDeploymentUpdate`.
pub struct SpectreGunshipDeploymentUpdate {
    pub data: SpectreGunshipDeploymentUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// `m_specialPowerModule`, found when the object is made.
    pub special_power_module: Option<ModuleSlot>,
    /// Not saved.
    pub initial_target_position: Coord3D,
    pub gunship_id: ObjectId,
}

impl SpectreGunshipDeploymentUpdate {
    pub fn new(data: SpectreGunshipDeploymentUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            special_power_module: None,
            initial_target_position: Coord3D::new(0.0, 0.0, 0.0),
            gunship_id: ObjectId::INVALID,
        }
    }

    /// Where the gunship appears: an edge point chosen by the data, pushed out
    /// along the line to the target.
    fn creation_point(&self, sim: &dyn SimView, owner: ObjectSlot, target: &Coord3D) -> Coord3D {
        use GunshipCreateLocType as Loc;
        let extent = sim.terrain_extent();
        let terrain = sim.terrain();
        let source = sim
            .object(owner)
            .map_or(Coord3D::new(0.0, 0.0, 0.0), |o| *o.thing.get_position());
        let mut point = match self.data.create_loc {
            Loc::EdgeNearSource => find_closest_edge_point(terrain, &extent, &source),
            Loc::EdgeFarthestFromSource => find_farthest_edge_point(terrain, &extent, &source),
            Loc::EdgeNearTarget => find_closest_edge_point(terrain, &extent, target),
            Loc::EdgeFarthestFromTarget => find_farthest_edge_point(terrain, &extent, target),
        };

        let mut delta = self.initial_target_position;
        delta.sub(&point);
        let distance = delta.length();
        delta.normalize();
        delta.x *= distance + self.data.gunship_orbit_radius;
        delta.y *= distance + self.data.gunship_orbit_radius;

        point.x = self.initial_target_position.x - delta.x;
        point.y = self.initial_target_position.y - delta.y;
        point
    }

    /// Makes the gunship at the map edge and hands the order on to it.
    fn deploy_gunship(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        template_id: UnsignedInt,
        target: &Coord3D,
        command_options: UnsignedInt,
    ) {
        let owner = ctx.owner_slot;
        // An earlier gunship is forgotten, not removed.
        if sim.find_object(self.gunship_id).is_some() {
            self.gunship_id = ObjectId::INVALID;
        }
        let Some(gunship_template) = sim
            .rules()
            .thing_factory
            .find_template_id(&self.data.gunship_template_name)
        else {
            return;
        };
        let Some(team) = sim.object(owner).map(|o| o.team_id) else {
            return;
        };
        let gunship = sim.spawn_object(gunship_template, team, Coord3D::new(0.0, 0.0, 0.0), 0.0);
        if let Some(obj) = sim.object_mut(gunship) {
            obj.producer_id = ctx.owner_id;
        }

        let mut point = self.creation_point(sim, owner, target);
        if let Some(height) = sim.ai_cur_locomotor_preferred_height(gunship) {
            point.z = height;
        }
        sim.set_object_position(gunship, &point);

        let orient = gm_atan2f(
            self.initial_target_position.y - point.y,
            self.initial_target_position.x - point.x,
        );
        sim.set_object_orientation(gunship, orient);
        let Some(gunship_id) = sim.object(gunship).map(|o| o.id) else {
            return;
        };
        self.gunship_id = gunship_id;

        let position = self.initial_target_position;
        if let Some(module) = sim.find_special_power_module(gunship, template_id) {
            sim.with_special_power_module(module, &mut |spm, sim, spm_ctx| {
                spm.mark_special_power_triggered(sim, spm_ctx, Some(&position));
                spm.do_special_power_at_location(
                    sim,
                    spm_ctx,
                    &position,
                    INVALID_ANGLE,
                    command_options,
                );
            });
        }
        let mask = sim.object(owner).map_or(0, |o| sim.player_mask(o.team_id));
        sim.select_object(gunship, TRUE, mask);
    }
}

impl SpecialPowerUpdateFacet for SpectreGunshipDeploymentUpdate {
    fn initiate_intent_to_do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        mut caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        _target_obj: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        command_options: UnsignedInt,
    ) -> Bool {
        let Some(own_module) = self.special_power_module else {
            return FALSE;
        };
        let mut own_template = None;
        caller.with(sim, own_module, &mut |spm, sim, spm_ctx| {
            own_template = Some(spm.get_special_power_id(&*sim, spm_ctx));
        });
        if own_template != Some(template_id) {
            return FALSE;
        }
        let Some(target) = target_pos else {
            return FALSE;
        };

        if command_options & COMMAND_FIRED_BY_SCRIPT != 0 {
            let now = sim.frame();
            caller.with(sim, own_module, &mut |spm, sim, spm_ctx| {
                spm.set_ready_frame(sim, spm_ctx, now);
            });
        }
        self.initial_target_position = *target;

        self.deploy_gunship(sim, ctx, template_id, target, command_options);

        let position = self.initial_target_position;
        caller.with(sim, own_module, &mut |spm, sim, spm_ctx| {
            spm.mark_special_power_triggered(sim, spm_ctx, Some(&position));
        });
        TRUE
    }
}

impl UpdateFacet for SpectreGunshipDeploymentUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let done = sim.object(ctx.owner_slot).is_some_and(|me| {
            me.status.test(ObjectStatusType::Sold.as_i32()) != FALSE
                || me.status.test(ObjectStatusType::UnderConstruction.as_i32()) != FALSE
                || me.is_effectively_dead() != FALSE
        });
        if done {
            UpdateSleepTime::FOREVER
        } else {
            UpdateSleepTime::NONE
        }
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        let mut mask = DisabledMaskType::default();
        for kind in [
            DisabledType::Subdued,
            DisabledType::Underpowered,
            DisabledType::Emp,
            DisabledType::Hacked,
        ] {
            mask.set(kind as i32, 1);
        }
        mask
    }
}

impl BehaviorModule for SpectreGunshipDeploymentUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let template_id = sim
            .rules()
            .special_power_store
            .find_special_power_template(&self.data.special_power_name)
            .map(|t| t.id());
        let Some(template_id) = template_id else {
            return;
        };
        self.special_power_module = sim.find_special_power_module(ctx.owner_slot, template_id);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// The `SpecialPowerUpdateModule` version byte is skipped: the reference
    /// calls `UpdateModule::xfer` directly.
    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_object_id(&mut self.gunship_id)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_special_power_update_mut(&mut self) -> Option<&mut dyn SpecialPowerUpdateFacet> {
        Some(self)
    }
}

pub fn parse_spectre_gunship_deployment_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SpectreGunshipDeploymentUpdateModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("GunshipTemplateName") {
            r.parse_ascii_string(&mut data.gunship_template_name)
        } else if is("RequiredScience") {
            let token = r.next_token(SEPS_DEFAULT)?;
            data.extra_required_science = parse_ctx
                .science_store
                .get_science_from_internal_name(token, parse_ctx.name_keys);
            Ok(())
        } else if is("SpecialPowerTemplate") {
            let token = r.next_token(SEPS_DEFAULT)?;
            data.special_power_name = if token.eq_ignore_ascii_case("None") {
                String::new()
            } else {
                token.to_owned()
            };
            Ok(())
        } else if is("AttackAreaRadius") {
            r.parse_real(&mut data.attack_area_radius)
        } else if is("CreateLocation") {
            let mut index = 0;
            r.parse_index_list(&mut index, &CREATE_LOCATION_NAMES)?;
            data.create_loc = match index {
                0 => GunshipCreateLocType::EdgeNearSource,
                1 => GunshipCreateLocType::EdgeFarthestFromSource,
                2 => GunshipCreateLocType::EdgeNearTarget,
                _ => GunshipCreateLocType::EdgeFarthestFromTarget,
            };
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_spectre_gunship_deployment_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SpectreGunshipDeploymentUpdateModuleData>()
        .expect("SpectreGunshipDeploymentUpdate data type mismatch")
        .clone();
    Box::new(SpectreGunshipDeploymentUpdate::new(typed))
}
