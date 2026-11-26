//! `PKT-272`: `GenerateMinefieldBehaviorModuleData` and `GenerateMinefieldBehavior`
//! (`GenerateMinefieldBehavior.h`, `GenerateMinefieldBehavior.cpp`).
//!
//! Packet: `packets/phase5/PKT-272_GenerateMinefieldBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt272_279_minefield.rs`.
//!
//! As in C++, a box or ring with no room between its mines (a zero footprint) never
//! finishes placing them.

use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{gm_cosf, gm_sinf, gm_sqrtf, real_to_int_ceil, Coord3D, Matrix3D, PI};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedByte, UnsignedInt, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::arena::{FacetMask, ModuleContext, ObjectSlot, ThingTemplateId};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_behavior_module_header, xfer_module_version,
    UpdateScheduleTracker,
};
use crate::modules::module::{
    BehaviorModule, DieFacet, GenerateMinefieldFacet, ModuleData, UpdateFacet, UpgradeFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::upgrade::upgrade_module::{UpgradeMux, UpgradeMuxData};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::kind_of::KindOfType;
use crate::rules::upgrade::UpgradeMaskType;
use crate::sim::{SimApi, SimView};

/// The upgrade `update` waits for, whatever `UpgradedTriggeredBy` says.
const EMP_MINES_UPGRADE: &str = "Upgrade_ChinaEMPMines";

/// INI template data matching `GenerateMinefieldBehaviorModuleData`.
///
/// The C++ constructor copies the standard distance and density from `GlobalData`;
/// `None` reads them when they are used.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerateMinefieldBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub upgrade_mux: UpgradeMuxData,
    pub mine_name: String,
    pub mine_name_upgraded: String,
    pub mine_upgrade_trigger: String,
    pub gen_fx: Option<String>,
    pub distance_around_object: Option<Real>,
    pub mines_per_square_foot: Option<Real>,
    pub random_jitter: Real,
    pub skip_if_this_much_under_structure: Real,
    pub on_death: Bool,
    pub border_only: Bool,
    pub always_circular: Bool,
    pub upgradable: Bool,
    pub smart_border: Bool,
    pub smart_border_skip_interior: Bool,
}

impl GenerateMinefieldBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            upgrade_mux: UpgradeMuxData::new(),
            mine_name: String::new(),
            mine_name_upgraded: String::new(),
            mine_upgrade_trigger: String::new(),
            gen_fx: None,
            distance_around_object: None,
            mines_per_square_foot: None,
            random_jitter: 0.0,
            skip_if_this_much_under_structure: 0.33,
            on_death: FALSE,
            border_only: TRUE,
            always_circular: FALSE,
            upgradable: FALSE,
            smart_border: FALSE,
            smart_border_skip_interior: TRUE,
        }
    }
}

impl ModuleData for GenerateMinefieldBehaviorModuleData {
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
        FacetMask(0x083)
    }
}

/// What every mine of one `placeMines` shares.
struct MineKind {
    template: ThingTemplateId,
    geometry: GeometryInfo,
}

impl MineKind {
    fn radius(&self) -> Real {
        self.geometry.get_bounding_circle_radius()
    }
}

/// Runtime instance of `GenerateMinefieldBehavior`.
pub struct GenerateMinefieldBehavior {
    pub data: GenerateMinefieldBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
    pub upgrade: UpgradeMux,
    pub target: Coord3D,
    pub has_target: Bool,
    pub generated: Bool,
    pub upgraded: Bool,
    pub mine_list: Vec<ObjectId>,
}

fn sqr(x: Real) -> Real {
    x * x
}

fn position(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn default_team(sim: &dyn SimView, owner: ObjectSlot) -> TeamId {
    let team = sim.object(owner).map_or(TeamId::INVALID, |o| o.team_id);
    sim.player_default_team(team)
}

/// `Div_Safe(len, diameter, 1)` rounded up, at least one.
fn mines_along(len: Real, diameter: Real) -> Int {
    let divisor = if diameter > 1.0 { diameter } else { 1.0 };
    real_to_int_ceil(len / divisor).max(1)
}

/// `offsetBySmallRandomAmount`.
fn jitter(sim: &mut dyn SimApi, pt: &mut Coord3D, amount: Real) {
    pt.x += sim.rng_logic_real(-amount, amount);
    pt.y += sim.rng_logic_real(-amount, amount);
}

/// `isAnythingTooClose2D`, reading where the mines are now.
fn too_close(sim: &dyn SimView, mines: &[ObjectSlot], pos: &Coord3D, min_dist_sqr: Real) -> bool {
    mines.iter().any(|&mine| {
        let p = position(sim, mine);
        sqr(p.x - pos.x) + sqr(p.y - pos.y) < min_dist_sqr
    })
}

/// `geom.set(GEOMETRY_CYLINDER, false, 1, r, r)` with the bounding circle's radius.
fn make_circular(geom: &mut GeometryInfo) {
    let radius = geom.get_bounding_circle_radius();
    geom.set(GeometryType::Cylinder, FALSE, 1.0, radius, radius);
}

impl GenerateMinefieldBehavior {
    pub fn new(data: GenerateMinefieldBehaviorModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            upgrade: UpgradeMux::new(),
            target: Coord3D::default(),
            has_target: FALSE,
            generated: FALSE,
            upgraded: FALSE,
            mine_list: Vec::new(),
        }
    }

    fn distance_around_object(&self, sim: &dyn SimView) -> Real {
        self.data
            .distance_around_object
            .unwrap_or_else(|| sim.rules().global_data.standard_minefield_distance)
    }

    /// `placeMineAt`.
    fn place_mine_at(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        pt: &Coord3D,
        kind: &MineKind,
        team: TeamId,
    ) -> Option<ObjectSlot> {
        let mut tmp = *pt;
        tmp.z = 99999.0;
        if sim.get_highest_layer_for_destination(&tmp, FALSE) == PathfindLayerEnum::Ground
            && (sim.terrain().is_underwater(pt.x, pt.y, None, None) != FALSE
                || sim.terrain().is_cliff_cell(pt.x, pt.y) != FALSE)
        {
            return None;
        }

        let orient = sim.rng_logic_real(-PI, PI);

        let mut geom = kind.geometry.clone();
        geom.expand_footprint(kind.radius() * -(1.0 - self.data.skip_if_this_much_under_structure));
        let under_structure = sim
            .find_potential_collisions(pt, &geom, orient, FALSE)
            .into_iter()
            .any(|them| {
                sim.object(them).is_some_and(|o| {
                    o.thing
                        .is_kind_of(&sim.rules().thing_factory, KindOfType::Structure)
                        != FALSE
                })
            });
        if under_structure {
            return None;
        }

        let mine = sim.spawn_object(kind.template, team, Coord3D::default(), 0.0);
        sim.set_object_position(mine, pt);
        sim.set_object_orientation(mine, orient);
        let producer_id = sim
            .object(ctx.owner_slot)
            .map_or(ObjectId::INVALID, |o| o.id);
        if let Some(record) = sim.object_mut(mine) {
            record.producer_id = producer_id;
        }
        let start = position(sim, ctx.owner_slot);
        sim.set_land_mine_scoot_parms(mine, &start, pt);

        if self.data.upgradable != FALSE {
            if let Some(id) = sim.object(mine).map(|o| o.id) {
                self.mine_list.push(id);
            }
        }
        Some(mine)
    }

    /// The point on the ground a mine of a line or circle is placed at.
    fn place_on_ground(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        x: Real,
        y: Real,
        kind: &MineKind,
        team: TeamId,
    ) {
        let mut pt = Coord3D::new(x, y, sim.terrain().get_ground_height(x, y, None));
        jitter(sim, &mut pt, kind.radius() * self.data.random_jitter);
        self.place_mine_at(sim, ctx, &pt, kind, team);
    }

    /// `placeMinesAlongLine`.
    fn place_mines_along_line(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        start: &Coord3D,
        end: &Coord3D,
        kind: &MineKind,
        skip_one_at_start: bool,
    ) {
        let team = default_team(sim, ctx.owner_slot);
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        let len = gm_sqrtf(sqr(dx) + sqr(dy));
        let inc = len / mines_along(len, kind.radius() * 2.0) as Real;
        let mut place = if skip_one_at_start { inc } else { 0.0 };
        while place <= len {
            let x = start.x + place * dx / len;
            let y = start.y + place * dy / len;
            self.place_on_ground(sim, ctx, x, y, kind, team);
            place += inc;
        }
    }

    /// `placeMinesAroundRect`. The corners come from the object's transform, so the
    /// target does not move a rectangle.
    fn place_mines_around_rect(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        major: Real,
        minor: Real,
        kind: &MineKind,
    ) {
        let mtx = sim
            .object(ctx.owner_slot)
            .map_or_else(Matrix3D::identity, |o| *o.thing.get_transform_matrix());
        let corner = |x, y| Matrix3D::transform_vector(&mtx, &Coord3D::new(x, y, 0.0));
        let pt = [
            corner(major, minor),
            corner(-major, minor),
            corner(-major, -minor),
            corner(major, -minor),
        ];
        for i in 0..4 {
            self.place_mines_along_line(sim, ctx, &pt[i], &pt[(i + 1) % 4], kind, true);
        }
    }

    /// `placeMinesAroundCircle`.
    fn place_mines_around_circle(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        pos: &Coord3D,
        radius: Real,
        kind: &MineKind,
    ) {
        let team = default_team(sim, ctx.owner_slot);
        let circum = 2.0 * PI * radius;
        let angle_inc = (2.0 * PI) / mines_along(circum, kind.radius() * 2.0) as Real;
        let angle_lim = (2.0 * PI) - angle_inc * 0.5;
        let mut angle: Real = 0.0;
        while angle < angle_lim {
            let x = pos.x + radius * gm_cosf(angle);
            let y = pos.y + radius * gm_sinf(angle);
            self.place_on_ground(sim, ctx, x, y, kind, team);
            angle += angle_inc;
        }
    }

    /// A rectangle for a box that is not forced round, else a circle around `target`.
    fn place_border(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: &Coord3D,
        geom: &GeometryInfo,
        kind: &MineKind,
    ) {
        if geom.get_geom_type() == GeometryType::Box && self.data.always_circular == FALSE {
            self.place_mines_around_rect(
                sim,
                ctx,
                geom.get_major_radius(),
                geom.get_minor_radius(),
                kind,
            );
        } else {
            self.place_mines_around_circle(sim, ctx, target, geom.get_major_radius(), kind);
        }
    }

    /// `placeMinesInFootprint`.
    fn place_mines_in_footprint(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: &Coord3D,
        geom: &GeometryInfo,
        kind: &MineKind,
    ) {
        let owner = ctx.owner_slot;
        let team = default_team(sim, owner);
        let density = self
            .data
            .mines_per_square_foot
            .unwrap_or_else(|| sim.rules().global_data.standard_minefield_density);
        let count = real_to_int_ceil(density * geom.get_footprint_area()).max(1);
        let min_dist_sqr = sqr(kind.radius() * 2.0);
        let mut mines = Vec::new();
        for _ in 0..count {
            let mut pt = Coord3D::default();
            let mut max_retry = 100;
            loop {
                sim.random_offset_within_footprint(geom, &mut pt);
                pt.x += target.x;
                pt.y += target.y;
                pt.z += target.z;
                max_retry -= 1;
                if !(too_close(sim, &mines, &pt, min_dist_sqr) && max_retry > 0) {
                    break;
                }
            }
            // Not where the object itself would stand at the target.
            if sim
                .object(owner)
                .is_some_and(|o| o.geometry.is_point_in_footprint(target, &pt) != FALSE)
            {
                continue;
            }
            if let Some(mine) = self.place_mine_at(sim, ctx, &pt, kind, team) {
                mines.push(mine);
            }
        }
    }

    /// `placeMines`.
    fn place_mines(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.generated != FALSE {
            return;
        }
        self.generated = TRUE;
        let owner = ctx.owner_slot;
        let name = if self.upgraded != FALSE {
            &self.data.mine_name_upgraded
        } else {
            &self.data.mine_name
        };
        let factory = &sim.rules().thing_factory;
        let Some(template) = factory.find_template_id(name) else {
            return;
        };
        let Some(geometry) = factory
            .find_by_template_id_final(template.0 as u16)
            .map(|t| t.get_template_geometry_info().clone())
        else {
            return;
        };
        let kind = MineKind { template, geometry };
        let target = if self.has_target != FALSE {
            self.target
        } else {
            position(sim, owner)
        };
        let Some(own_geometry) = sim.object(owner).map(|o| o.geometry.clone()) else {
            return;
        };
        let distance = self.distance_around_object(sim);
        let mut geom = own_geometry;

        if self.data.smart_border != FALSE {
            if self.data.smart_border_skip_interior == FALSE {
                geom = kind.geometry.clone();
                let team = default_team(sim, owner);
                self.place_mine_at(sim, ctx, &target, &kind, team);
            }
            if self.data.always_circular != FALSE {
                make_circular(&mut geom);
            }
            let diameter = kind.radius() * 2.0;
            geom.expand_footprint(kind.radius());
            loop {
                self.place_border(sim, ctx, &target, &geom, &kind);
                geom.expand_footprint(diameter);
                // `<` with a NaN radius or distance stops, as in C++.
                if !(geom.get_bounding_circle_radius() < distance) {
                    break;
                }
            }
        } else if self.data.border_only != FALSE {
            geom.expand_footprint(distance);
            self.place_border(sim, ctx, &target, &geom, &kind);
        } else {
            geom.expand_footprint(distance);
            if self.data.always_circular != FALSE {
                make_circular(&mut geom);
            }
            self.place_mines_in_footprint(sim, ctx, &target, &geom, &kind);
        }

        if let Some(fx) = &self.data.gen_fx {
            let pos = position(sim, owner);
            sim.trigger_fx_list(fx, &pos);
        }
    }
}

impl GenerateMinefieldFacet for GenerateMinefieldBehavior {
    fn set_minefield_target(&mut self, pos: Option<&Coord3D>) {
        self.has_target = if pos.is_some() { TRUE } else { FALSE };
        self.target = pos.copied().unwrap_or_default();
    }
}

impl DieFacet for GenerateMinefieldBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        if self.data.on_death != FALSE {
            self.place_mines(sim, ctx);
        }
    }
}

impl UpdateFacet for GenerateMinefieldBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.upgraded != FALSE || self.data.upgradable == FALSE {
            return UpdateSleepTime::FOREVER;
        }
        if self.generated != FALSE {
            let rules = sim.rules();
            let upgraded = rules
                .upgrade_center
                .lookup_upgrade(EMP_MINES_UPGRADE, &rules.name_keys)
                .is_some_and(|upgrade| {
                    sim.object(ctx.owner_slot).is_some_and(|o| {
                        o.upgrades_completed.test_for_any(upgrade.upgrade_mask()) != FALSE
                    })
                });
            if upgraded {
                self.upgraded = TRUE;
                for id in std::mem::take(&mut self.mine_list) {
                    if let Some(mine) = sim.find_object(id) {
                        sim.destroy_object(mine);
                    }
                }
                self.generated = FALSE;
                self.place_mines(sim, ctx);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl UpgradeFacet for GenerateMinefieldBehavior {
    fn is_already_upgraded(&self) -> Bool {
        self.upgrade.is_already_upgraded()
    }

    fn attempt_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        if self.would_upgrade(sim, ctx, key_mask) == FALSE {
            return FALSE;
        }
        self.give_self_upgrade(sim, ctx);
        TRUE
    }

    fn would_upgrade(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .would_upgrade(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn reset_upgrade(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .reset_upgrade(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn test_upgrade_conditions(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        key_mask: &UpgradeMaskType,
    ) -> Bool {
        self.upgrade
            .test_upgrade_conditions(sim.rules(), &self.data.upgrade_mux, key_mask)
    }

    fn get_upgrade_activation_masks(
        &self,
        sim: &dyn SimView,
        _ctx: ModuleContext,
        activation: &mut UpgradeMaskType,
        conflicting: &mut UpgradeMaskType,
    ) {
        self.data
            .upgrade_mux
            .get_upgrade_activation_masks(sim.rules(), activation, conflicting);
    }

    fn perform_upgrade_fx(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.data
            .upgrade_mux
            .perform_upgrade_fx(sim, ctx.owner_slot);
    }

    fn is_sub_objects_upgrade(&self) -> Bool {
        FALSE
    }

    /// `giveSelfUpgrade`; `upgradeImplementation` places the mines.
    fn give_self_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.data
            .upgrade_mux
            .perform_upgrade_fx(sim, ctx.owner_slot);
        self.data
            .upgrade_mux
            .process_upgrade_removal(sim, ctx.owner_slot);
        self.place_mines(sim, ctx);
        self.upgrade.set_upgrade_executed(TRUE);
    }

    fn force_refresh_upgrade(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.upgrade.is_already_upgraded() != FALSE {
            self.place_mines(sim, ctx);
        }
    }
}

impl BehaviorModule for GenerateMinefieldBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    /// `BehaviorModule::crc` and `upgradeMuxCRC`.
    fn crc(&self, xfer: &mut dyn Xfer) {
        self.upgrade.crc(xfer);
    }

    /// There is no `UpdateModule` header, so the wake frame is not saved.
    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        xfer_behavior_module_header(xfer)?;
        self.upgrade.xfer(xfer)?;
        xfer.xfer_bool(&mut self.generated)?;
        xfer.xfer_bool(&mut self.has_target)?;
        xfer.xfer_bool(&mut self.upgraded)?;
        xfer.xfer_coord3d(&mut self.target)?;
        // The count is a byte, but saving writes every id.
        let mut count = self.mine_list.len() as UnsignedByte;
        xfer.xfer_unsigned_byte(&mut count)?;
        if xfer.mode() == XferMode::Save {
            for id in &mut self.mine_list {
                xfer.xfer_object_id(id)?;
            }
        } else if xfer.mode() == XferMode::Load {
            self.mine_list.clear();
            for _ in 0..UnsignedInt::from(count) {
                let mut id = ObjectId::INVALID;
                xfer.xfer_object_id(&mut id)?;
                self.mine_list.push(id);
            }
        }
        Ok(())
    }

    fn load_post_process(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
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

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_upgrade(&self) -> Option<&dyn UpgradeFacet> {
        Some(self)
    }

    fn as_upgrade_mut(&mut self) -> Option<&mut dyn UpgradeFacet> {
        Some(self)
    }

    fn as_generate_minefield_mut(&mut self) -> Option<&mut dyn GenerateMinefieldFacet> {
        Some(self)
    }
}

pub fn parse_generate_minefield_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = GenerateMinefieldBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if field.eq_ignore_ascii_case("MineName") {
            r.parse_ascii_string(&mut data.mine_name)
        } else if field.eq_ignore_ascii_case("UpgradedMineName") {
            r.parse_ascii_string(&mut data.mine_name_upgraded)
        } else if field.eq_ignore_ascii_case("UpgradedTriggeredBy") {
            r.parse_ascii_string(&mut data.mine_upgrade_trigger)
        } else if field.eq_ignore_ascii_case("GenerationFX") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            data.gen_fx = resolve_optional_asset_name(tok, parse_ctx.fx_list_names);
            Ok(())
        } else if field.eq_ignore_ascii_case("DistanceAroundObject") {
            let mut value = 0.0;
            r.parse_real(&mut value)?;
            data.distance_around_object = Some(value);
            Ok(())
        } else if field.eq_ignore_ascii_case("MinesPerSquareFoot") {
            let mut value = 0.0;
            r.parse_real(&mut value)?;
            data.mines_per_square_foot = Some(value);
            Ok(())
        } else if field.eq_ignore_ascii_case("GenerateOnlyOnDeath") {
            r.parse_bool(&mut data.on_death)
        } else if field.eq_ignore_ascii_case("BorderOnly") {
            r.parse_bool(&mut data.border_only)
        } else if field.eq_ignore_ascii_case("SmartBorder") {
            r.parse_bool(&mut data.smart_border)
        } else if field.eq_ignore_ascii_case("SmartBorderSkipInterior") {
            r.parse_bool(&mut data.smart_border_skip_interior)
        } else if field.eq_ignore_ascii_case("AlwaysCircular") {
            r.parse_bool(&mut data.always_circular)
        } else if field.eq_ignore_ascii_case("Upgradable") {
            r.parse_bool(&mut data.upgradable)
        } else if field.eq_ignore_ascii_case("RandomJitter") {
            r.parse_percent_to_real(&mut data.random_jitter)
        } else if field.eq_ignore_ascii_case("SkipIfThisMuchUnderStructure") {
            r.parse_percent_to_real(&mut data.skip_if_this_much_under_structure)
        } else if let Some(res) = data.upgrade_mux.parse_field(field, r, parse_ctx) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_generate_minefield_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<GenerateMinefieldBehaviorModuleData>()
        .expect("GenerateMinefieldBehavior data type mismatch")
        .clone();
    Box::new(GenerateMinefieldBehavior::new(typed))
}
