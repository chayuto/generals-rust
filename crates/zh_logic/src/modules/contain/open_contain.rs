//! `PKT-282`: `OpenContainModuleData`, `OpenContainCore`, and `OpenContain`
//! (`OpenContain.h`, `OpenContain.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    damage_info_from_die_op, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::contain::contain_module::CONTAIN_MAX_UNKNOWN;
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{
    BehaviorModule, CommandSourceType, EvacDisposition, ExitDoorType, ModuleData,
    ObjectEnterExitType,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::disabled_types::DisabledMaskType;
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
use crate::rules::model_state::{
    ModelConditionFlagType, ModelConditionFlags, MODEL_CONDITION_NAMES,
};
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use crate::gamelogic::global_data::BodyDamageType;
use std::any::Any;
use std::collections::BTreeMap;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D, PI};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UnsignedShort,
    UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const MAX_FIRE_POINTS: usize = 32;

/// INI template data matching `OpenContainModuleData` (`OpenContain.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct OpenContainModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub contain_max: Int,
    pub enter_sound: String,
    pub exit_sound: String,
    pub passengers_allowed_to_fire: Bool,
    pub passengers_in_turret: Bool,
    pub number_of_exit_paths: Int,
    pub damage_percentage_to_units: Real,
    pub is_burned_death_to_units: Bool,
    pub door_open_time: UnsignedInt,
    pub allow_inside_kind_of: KindOfMaskType,
    pub forbid_inside_kind_of: KindOfMaskType,
    pub weapon_bonus_passed_to_passengers: Bool,
    pub allow_allies_inside: Bool,
    pub allow_enemies_inside: Bool,
    pub allow_neutral_inside: Bool,
}

impl OpenContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut allow_inside = KindOfMaskType::new();
        allow_inside.flip();
        Self {
            tag_key,
            module_name_key,
            die_mux: DieMuxData::new(),
            contain_max: CONTAIN_MAX_UNKNOWN,
            enter_sound: String::new(),
            exit_sound: String::new(),
            passengers_allowed_to_fire: FALSE,
            passengers_in_turret: FALSE,
            number_of_exit_paths: 1,
            damage_percentage_to_units: 0.0,
            is_burned_death_to_units: TRUE,
            door_open_time: 1,
            allow_inside_kind_of: allow_inside,
            forbid_inside_kind_of: KindOfMaskType::new(),
            weapon_bonus_passed_to_passengers: FALSE,
            allow_allies_inside: TRUE,
            allow_enemies_inside: TRUE,
            allow_neutral_inside: TRUE,
        }
    }

    /// Parses one `OpenContainModuleData` INI field, returning `None` if unhandled.
    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        _parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("ContainMax") {
            Some(reader.parse_int(&mut self.contain_max))
        } else if field.eq_ignore_ascii_case("EnterSound") {
            Some(parse_audio_event_name(reader, &mut self.enter_sound))
        } else if field.eq_ignore_ascii_case("ExitSound") {
            Some(parse_audio_event_name(reader, &mut self.exit_sound))
        } else if field.eq_ignore_ascii_case("DamagePercentToUnits") {
            Some(reader.parse_percent_to_real(&mut self.damage_percentage_to_units))
        } else if field.eq_ignore_ascii_case("BurnedDeathToUnits") {
            Some(reader.parse_bool(&mut self.is_burned_death_to_units))
        } else if field.eq_ignore_ascii_case("AllowInsideKindOf") {
            Some(self.allow_inside_kind_of.parse_from_ini(reader, &KINDOF_NAMES))
        } else if field.eq_ignore_ascii_case("ForbidInsideKindOf") {
            Some(self.forbid_inside_kind_of.parse_from_ini(reader, &KINDOF_NAMES))
        } else if field.eq_ignore_ascii_case("PassengersAllowedToFire") {
            Some(reader.parse_bool(&mut self.passengers_allowed_to_fire))
        } else if field.eq_ignore_ascii_case("PassengersInTurret") {
            Some(reader.parse_bool(&mut self.passengers_in_turret))
        } else if field.eq_ignore_ascii_case("NumberOfExitPaths") {
            Some(reader.parse_int(&mut self.number_of_exit_paths))
        } else if field.eq_ignore_ascii_case("DoorOpenTime") {
            Some(reader.parse_duration_unsigned_int(&mut self.door_open_time))
        } else if field.eq_ignore_ascii_case("WeaponBonusPassedToPassengers") {
            Some(reader.parse_bool(&mut self.weapon_bonus_passed_to_passengers))
        } else if field.eq_ignore_ascii_case("AllowAlliesInside") {
            Some(reader.parse_bool(&mut self.allow_allies_inside))
        } else if field.eq_ignore_ascii_case("AllowEnemiesInside") {
            Some(reader.parse_bool(&mut self.allow_enemies_inside))
        } else if field.eq_ignore_ascii_case("AllowNeutralInside") {
            Some(reader.parse_bool(&mut self.allow_neutral_inside))
        } else {
            self.die_mux.parse_field(field, reader)
        }
    }
}

fn parse_audio_event_name(reader: &mut IniReader<'_>, out: &mut String) -> Result<(), IniError> {
    let tok = reader.next_token(SEPS_DEFAULT)?;
    if tok.eq_ignore_ascii_case("NoSound") || tok.eq_ignore_ascii_case("None") {
        out.clear();
    } else {
        *out = tok.to_owned();
    }
    Ok(())
}

impl ModuleData for OpenContainModuleData {
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
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0)
    }
}

fn identity_fire_points() -> [Matrix3D; MAX_FIRE_POINTS] {
    let mut m = Matrix3D::default();
    m.make_identity();
    [m; MAX_FIRE_POINTS]
}

/// Reusable containment core state (`OpenContain`) shared across `WP-5.3` / `WP-5.4` container subclasses.
#[derive(Debug, Clone, PartialEq)]
pub struct OpenContainCore {
    pub contain_list: Vec<ObjectId>,
    pub contain_list_size: UnsignedInt,
    pub object_enter_exit_info: BTreeMap<ObjectId, ObjectEnterExitType>,
    pub stealth_units_contained: UnsignedInt,
    pub hero_units_contained: UnsignedInt,
    pub which_exit_path: Int,
    pub door_close_countdown: UnsignedInt,
    pub xfer_contain_id_list: Vec<ObjectId>,
    pub player_entered_mask: UnsignedShort,
    pub last_unload_sound_frame: UnsignedInt,
    pub last_load_sound_frame: UnsignedInt,
    pub condition_state: ModelConditionFlags,
    pub fire_points: [Matrix3D; MAX_FIRE_POINTS],
    pub fire_point_start: Int,
    pub fire_point_next: Int,
    pub fire_point_size: Int,
    pub no_fire_points_in_art: Bool,
    pub rally_point: Coord3D,
    pub rally_point_exists: Bool,
    pub load_sounds_enabled: Bool,
    pub passenger_allowed_to_fire: Bool,
    pub schedule: UpdateScheduleTracker,
}

impl OpenContainCore {
    pub fn new(passengers_allowed_to_fire: Bool) -> Self {
        Self {
            contain_list: Vec::new(),
            contain_list_size: 0,
            object_enter_exit_info: BTreeMap::new(),
            stealth_units_contained: 0,
            hero_units_contained: 0,
            which_exit_path: 1,
            door_close_countdown: 0,
            xfer_contain_id_list: Vec::new(),
            player_entered_mask: 0,
            last_unload_sound_frame: 0,
            last_load_sound_frame: 0,
            condition_state: ModelConditionFlags::new(),
            fire_points: identity_fire_points(),
            fire_point_start: -1,
            fire_point_next: 0,
            fire_point_size: 0,
            no_fire_points_in_art: FALSE,
            rally_point: Coord3D::default(),
            rally_point_exists: FALSE,
            load_sounds_enabled: TRUE,
            passenger_allowed_to_fire: passengers_allowed_to_fire,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    pub fn sidecar(&self, contain_max: Int, enclosing: Bool, garrisonable: Bool) -> ModuleSidecar {
        ModuleSidecar::Contain {
            count: self.contain_list_size,
            max: contain_max,
            enclosing,
            stealth_units: self.stealth_units_contained,
            garrisoned: FALSE,
            garrisonable,
            rider_id: ObjectId::INVALID,
        }
    }

    #[inline]
    pub fn sync_sidecar_to_sim(&self, sim: &mut dyn SimApi, ctx: ModuleContext, contain_max: Int) {
        sim.set_module_sidecar(ctx.mod_slot, self.sidecar(contain_max, TRUE, FALSE));
    }

    #[inline]
    pub fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, contain_max: Int) {
        self.schedule.bind(ctx);
        self.sync_sidecar_to_sim(sim, ctx, contain_max);
    }

    pub fn is_valid_container_for(
        &self,
        data: &OpenContainModuleData,
        sim: &dyn SimView,
        ctx: ModuleContext,
        obj: ObjectSlot,
    ) -> Bool {
        let Some(rider) = sim.object(obj) else {
            return FALSE;
        };
        let tf = &sim.rules().thing_factory;
        if rider.thing.is_any_kind_of(tf, &data.allow_inside_kind_of) == FALSE
            || rider.thing.is_any_kind_of(tf, &data.forbid_inside_kind_of) != FALSE
        {
            return FALSE;
        }
        match sim.player_relationship(obj, ctx.owner_slot) {
            Relationship::Allies if data.allow_allies_inside == FALSE => FALSE,
            Relationship::Enemies if data.allow_enemies_inside == FALSE => FALSE,
            Relationship::Neutral if data.allow_neutral_inside == FALSE => FALSE,
            _ => TRUE,
        }
    }

    pub fn add_to_contain_list(&mut self, sim: &dyn SimView, rider: ObjectSlot) {
        let Some(rec) = sim.object(rider) else {
            return;
        };
        let tf = &sim.rules().thing_factory;
        // Mirrors OpenContain.cpp (`m_containList.push_back(rider)`).
        self.contain_list.push(rec.id);
        self.contain_list_size += 1;
        if rec.thing.is_kind_of(tf, KindOfType::StealthGarrison) != FALSE {
            self.stealth_units_contained += 1;
        }
        if rec.thing.is_kind_of(tf, KindOfType::Hero) != FALSE {
            self.hero_units_contained += 1;
        }
    }

    /// `OpenContain::redeployOccupants` (headless art has no fire points).
    pub fn redeploy_occupants(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.no_fire_points_in_art = FALSE;
        self.fire_point_start = -1;
        self.fire_point_next = 0;
        self.fire_point_size = 0;
        if self.contain_list.is_empty() {
            return;
        }
        self.no_fire_points_in_art = TRUE;
        if let Some(c_pos) = sim.object(ctx.owner_slot).map(|o| *o.thing.get_position()) {
            let ids = self.contain_list.clone();
            for id in ids.into_iter().rev() {
                if let Some(rslot) = sim.find_object(id) {
                    sim.set_object_position(rslot, &c_pos);
                }
            }
        }
    }

    pub fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// Serializes `OpenContain::xfer` (version `2`).
    pub fn xfer(
        &mut self,
        xfer: &mut dyn Xfer,
        sim: &dyn SimView,
    ) -> Result<(), XferError> {
        let version = xfer_module_version(xfer, 2)?;
        self.schedule.xfer_update_header(xfer, sim)?;

        if xfer.mode() == XferMode::Save {
            xfer.xfer_unsigned_int(&mut self.contain_list_size)?;
            for id in &mut self.contain_list {
                xfer.xfer_object_id(id)?;
            }
        } else {
            if !self.contain_list.is_empty() {
                self.contain_list.clear();
                self.contain_list_size = 0;
            }
            xfer.xfer_unsigned_int(&mut self.contain_list_size)?;
            self.xfer_contain_id_list.clear();
            for _ in 0..self.contain_list_size {
                let mut id = ObjectId::INVALID;
                xfer.xfer_object_id(&mut id)?;
                self.xfer_contain_id_list.push(id);
            }
        }

        xfer.xfer_unsigned_short(&mut self.player_entered_mask)?;
        xfer.xfer_unsigned_int(&mut self.last_unload_sound_frame)?;
        xfer.xfer_unsigned_int(&mut self.last_load_sound_frame)?;
        xfer.xfer_unsigned_int(&mut self.stealth_units_contained)?;
        xfer.xfer_unsigned_int(&mut self.door_close_countdown)?;
        self.condition_state
            .xfer_named(xfer, &MODEL_CONDITION_NAMES)?;

        for mtx in &mut self.fire_points {
            for row in &mut mtx.rows {
                for val in row.iter_mut() {
                    xfer.xfer_real(val)?;
                }
            }
        }

        xfer.xfer_int(&mut self.fire_point_start)?;
        xfer.xfer_int(&mut self.fire_point_next)?;
        xfer.xfer_int(&mut self.fire_point_size)?;
        xfer.xfer_bool(&mut self.no_fire_points_in_art)?;
        xfer.xfer_coord3d(&mut self.rally_point)?;
        xfer.xfer_bool(&mut self.rally_point_exists)?;

        let mut enter_exit_count = self.object_enter_exit_info.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut enter_exit_count)?;
        if xfer.mode() == XferMode::Save {
            for (&id, &wants) in &self.object_enter_exit_info {
                let mut id_copy = id;
                xfer.xfer_object_id(&mut id_copy)?;
                let mut raw_wants = wants as Int;
                xfer.xfer_int(&mut raw_wants)?;
            }
        } else {
            if !self.object_enter_exit_info.is_empty() {
                return Err(XferError::ListNotEmpty);
            }
            for _ in 0..enter_exit_count {
                let mut id = ObjectId::INVALID;
                let mut raw_wants: Int = 0;
                xfer.xfer_object_id(&mut id)?;
                xfer.xfer_int(&mut raw_wants)?;
                let wants = match raw_wants {
                    0 => ObjectEnterExitType::WantsToEnter,
                    1 => ObjectEnterExitType::WantsToExit,
                    _ => ObjectEnterExitType::WantsNeither,
                };
                self.object_enter_exit_info.insert(id, wants);
            }
        }

        xfer.xfer_int(&mut self.which_exit_path)?;
        if version >= 2 {
            xfer.xfer_bool(&mut self.passenger_allowed_to_fire)?;
        }
        Ok(())
    }

    pub fn load_post_process(
        &mut self,
        data: &OpenContainModuleData,
        enclosing: Bool,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) {
        self.schedule.load_post_process(sim, ctx);
        if !self.contain_list.is_empty() {
            return;
        }
        self.contain_list_size = 0;
        self.stealth_units_contained = 0;
        self.hero_units_contained = 0;
        let ids = std::mem::take(&mut self.xfer_contain_id_list);
        for id in ids {
            if let Some(slot) = sim.find_object(id) {
                self.add_to_contain_list(sim, slot);
                if enclosing != FALSE {
                    add_or_remove_obj_from_world(sim, slot, FALSE);
                }
                sim.set_contained_by(slot, Some(ctx.owner_slot));
            }
        }
        self.sync_sidecar_to_sim(sim, ctx, data.contain_max);
    }

    /// `OpenContain::isPassengerAllowedToFire`.
    pub fn is_passenger_allowed_to_fire(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        if self.passenger_allowed_to_fire == FALSE {
            return FALSE;
        }
        if let Some(parent_slot) = sim.object(ctx.owner_slot).and_then(|o| o.contained_by) {
            if let Some(parent) = sim.object(parent_slot) {
                return parent.passengers_allowed_to_fire;
            }
        }
        TRUE
    }

    pub fn get_closest_rider(&self, sim: &dyn SimView, pos: &Coord3D) -> Option<ObjectSlot> {
        let mut best: Option<(ObjectSlot, Real)> = None;
        for &id in &self.contain_list {
            let Some(slot) = sim.find_object(id) else {
                continue;
            };
            let Some(rec) = sim.object(slot) else {
                continue;
            };
            let rpos = rec.thing.get_position();
            let dx = rpos.x - pos.x;
            let dy = rpos.y - pos.y;
            let d2 = dx * dx + dy * dy;
            if best.is_none_or(|(_, bd)| d2 < bd) {
                best = Some((slot, d2));
            }
        }
        best.map(|(s, _)| s)
    }
}

/// The C++ virtual table of `OpenContain`: every default is `OpenContain`'s own behaviour and
/// each subclass overrides what its header overrides. [`impl_contain_module!`] turns an
/// implementor into a [`BehaviorModule`] with all contain facets.
pub trait ContainBehavior: Any + Send {
    fn open_data(&self) -> &OpenContainModuleData;
    fn open_core(&self) -> &OpenContainCore;
    fn open_core_mut(&mut self) -> &mut OpenContainCore;

    fn module_key(&self) -> NameKeyType {
        self.open_data().module_name_key
    }
    fn construct(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let max = self.contain_max_value();
        let enclosing = self.is_enclosing();
        let garrisonable = self.is_garrisonable_container();
        let core = self.open_core_mut();
        core.schedule.bind(ctx);
        sim.set_module_sidecar(ctx.mod_slot, core.sidecar(max, enclosing, garrisonable));
    }
    fn delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let ids = self.open_core().contain_list.clone();
        for id in ids {
            if let Some(rider) = sim.find_object(id) {
                sim.destroy_object(rider);
                // `Object::onDestroy` removes the rider from its container immediately.
                self.remove_one(sim, ctx, rider, FALSE);
            }
        }
    }
    fn capture(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_team: TeamId,
        _new_team: TeamId,
    ) {
    }
    fn crc_module(&self, xfer: &mut dyn Xfer) {
        self.open_core().crc(xfer);
    }
    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.open_core_mut().xfer(xfer, sim)
    }
    fn post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let data = self.open_data().clone();
        let enclosing = self.is_enclosing();
        self.open_core_mut()
            .load_post_process(&data, enclosing, sim, ctx);
        sync_contain_sidecar(self, sim, ctx);
    }

    // UpdateModule
    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        update_open_contain(self, sim, ctx)
    }
    fn update_disabled_types(&self) -> DisabledMaskType {
        DisabledMaskType::new()
    }

    // ContainModuleInterface
    fn contain_max_value(&self) -> Int {
        self.open_data().contain_max
    }
    fn is_enclosing(&self) -> Bool {
        TRUE
    }
    fn is_garrisonable_container(&self) -> Bool {
        FALSE
    }
    fn is_bustable_container(&self) -> Bool {
        FALSE
    }
    fn is_heal_container(&self) -> Bool {
        FALSE
    }
    fn is_immune_to_clear(&self) -> Bool {
        FALSE
    }
    fn is_special_zero_slot(&self) -> Bool {
        FALSE
    }
    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        _check_capacity: Bool,
    ) -> Bool {
        self.open_core()
            .is_valid_container_for(self.open_data(), sim, ctx, rider)
    }
    fn passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        _id: ObjectId,
    ) -> Bool {
        self.open_core().is_passenger_allowed_to_fire(sim, ctx)
    }
    /// `OpenContain::onContaining` only plays audio.
    // C++ virtuals that shared or redirected passenger lists override.

    fn add_to_list(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext, rider: ObjectSlot) {
        self.open_core_mut().add_to_contain_list(sim, rider);
    }
    fn remove_one(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        rider: ObjectSlot,
        expose_stealthed: Bool,
    ) {
        remove_from_contain(self, sim, ctx, rider, expose_stealthed);
    }
    fn kill_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        kill_all_contained(self, sim, ctx);
    }
    fn harm_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        harm_and_force_exit_all_contained(self, sim, ctx, dmg);
    }
    fn order_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        cmd_source: CommandSourceType,
    ) {
        order_all_passengers_to_exit(self, sim, ctx, cmd_source);
    }
    fn contain_count_view(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt {
        self.open_core().contain_list_size
    }
    fn contain_max_view(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Int {
        self.contain_max_value()
    }
    fn contained_view<'a>(&'a self, _sim: &'a dyn SimView, _ctx: ModuleContext) -> &'a [ObjectId] {
        &self.open_core().contain_list
    }
    fn hero_count_view(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> UnsignedInt {
        self.open_core().hero_units_contained
    }
    /// `OpenContain::isContained` asks the object, not the list.
    fn contains_view(&self, sim: &dyn SimView, ctx: ModuleContext, id: ObjectId) -> Bool {
        Bool::from(
            sim.find_object(id)
                .and_then(|slot| sim.object(slot))
                .is_some_and(|o| o.contained_by == Some(ctx.owner_slot)),
        )
    }
    fn is_tunnel(&self) -> Bool {
        FALSE
    }
    fn kicks_out_on_capture(&self) -> Bool {
        TRUE
    }
    fn contained(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _rider: ObjectSlot) {}
    /// `OpenContain::onRemoving` only plays audio.
    fn removing(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _rider: ObjectSlot) {}
    fn remove_all(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, expose_stealthed: Bool) {
        remove_all_contained(self, sim, ctx, expose_stealthed);
    }
    fn kill_riders_not_free_to_exit(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {}
    fn selling(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        order_all_passengers_to_exit(self, sim, ctx, CommandSourceType::FromAi);
    }
    fn set_evac(&mut self, _disp: EvacDisposition) {}
    fn set_override_dest(&mut self, _pos: &Coord3D) {}
    /// `OpenContain::redeployOccupants`.
    fn redeploy(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.open_core_mut().redeploy_occupants(sim, ctx);
    }
    fn react_to_transform(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.redeploy(sim, ctx);
    }

    // CollideModule / DieModule / DamageModule
    fn collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        on_collide(self, sim, ctx, other);
    }
    fn die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        on_die(self, sim, ctx, dmg);
    }
    fn damaged(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}
    fn body_damage_state_changed(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_state: BodyDamageType,
        _new_state: BodyDamageType,
    ) {
    }

    // ExitInterface
    fn exit_busy(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
        FALSE
    }
    fn reserve_door(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        ExitDoorType::Door1
    }
    fn exit_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        _door: ExitDoorType,
    ) {
        exit_object_via_door(self, sim, ctx, obj);
    }

    // TransportPassengerInterface
    fn is_transport_passenger(&self) -> Bool {
        FALSE
    }
    fn try_evacuate(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _expose_stealthed: Bool,
    ) -> Bool {
        FALSE
    }
}

fn sync_contain_sidecar<C: ContainBehavior + ?Sized>(
    c: &C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) {
    sim.set_module_sidecar(
        ctx.mod_slot,
        c.open_core().sidecar(
            c.contain_max_value(),
            c.is_enclosing(),
            c.is_garrisonable_container(),
        ),
    );
}

/// The partition half of `OpenContain::addOrRemoveObjFromWorld`. Riders that `obj`
/// holds without enclosing them come and go with it.
pub(crate) fn add_or_remove_obj_from_world(sim: &mut dyn SimApi, obj: ObjectSlot, add: Bool) {
    sim.set_in_partition(obj, add);
    for id in sim.contain_items(obj) {
        if let Some(rider) = sim.find_object(id) {
            if sim.contain_is_enclosing_container_for(obj, rider) == FALSE {
                add_or_remove_obj_from_world(sim, rider, add);
            }
        }
    }
}

/// `Object::onContainedBy` (status bits plus back pointer and shroud refresh).
pub(crate) fn on_contained_by(sim: &mut dyn SimApi, rider: ObjectSlot, owner: ObjectSlot, enclosing: Bool) {
    sim.set_status(rider, ObjectStatusType::Unselectable);
    if enclosing != FALSE {
        sim.set_status(rider, ObjectStatusType::Masked);
    } else {
        sim.clear_status(rider, ObjectStatusType::Masked);
    }
    sim.set_contained_by(rider, Some(owner));
    sim.handle_partition_cell_maintenance(rider);
}

/// `Object::onRemovedFrom`.
pub(crate) fn on_removed_from(sim: &mut dyn SimApi, rider: ObjectSlot) {
    sim.clear_status(rider, ObjectStatusType::Masked);
    sim.clear_status(rider, ObjectStatusType::Unselectable);
    sim.set_contained_by(rider, None);
    sim.handle_partition_cell_maintenance(rider);
}

/// `Object::kill(damageType, deathType)`.
pub fn kill_object(
    sim: &mut dyn SimApi,
    victim: ObjectSlot,
    damage_type: DamageType,
    death_type: DeathType,
) {
    let mut dmg = DamageInfo::default();
    dmg.input.damage_type = damage_type;
    dmg.input.death_type = death_type;
    dmg.input.source_id = ObjectId::INVALID;
    dmg.input.amount = sim.object_max_health(victim);
    dmg.input.kill = TRUE;
    sim.attempt_damage(victim, &mut dmg);
}

/// `OpenContain::addToContain`.
pub fn add_to_contain<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    rider: ObjectSlot,
) {
    #[cfg(not(feature = "retail-crc"))]
    if sim.object(ctx.owner_slot).is_none_or(|o| o.is_destroyed()) {
        return;
    }
    let Some(rec) = sim.object(rider) else {
        return;
    };
    if rec.is_destroyed() || rec.contained_by.is_some() {
        return;
    }
    let rider_team = rec.team_id;
    c.add_to_list(sim, ctx, rider);
    c.open_core_mut().player_entered_mask = sim.player_mask(rider_team);
    sync_contain_sidecar(c, sim, ctx);

    if c.is_enclosing() != FALSE {
        add_or_remove_obj_from_world(sim, rider, FALSE);
    }

    #[cfg(feature = "retail-crc")]
    c.redeploy(sim, ctx);

    c.contained(sim, ctx, rider);

    #[cfg(not(feature = "retail-crc"))]
    c.redeploy(sim, ctx);

    on_contained_by(sim, rider, ctx.owner_slot, c.is_enclosing());

    let now = sim.frame();
    let core = c.open_core_mut();
    if core.load_sounds_enabled != FALSE {
        core.last_load_sound_frame = now;
    }
}

/// `OpenContain::removeFromContainViaIterator`.
fn remove_at<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    index: usize,
    expose_stealthed: Bool,
) {
    let rider_id = c.open_core_mut().contain_list.remove(index);
    c.open_core_mut().contain_list_size = c.open_core().contain_list_size.saturating_sub(1);
    let Some(rider) = sim.find_object(rider_id) else {
        sync_contain_sidecar(c, sim, ctx);
        return;
    };
    if sim.object_is_kind_of(rider, KindOfType::StealthGarrison) != FALSE {
        let core = c.open_core_mut();
        core.stealth_units_contained = core.stealth_units_contained.saturating_sub(1);
        if expose_stealthed != FALSE {
            sim.mark_stealth_detected(rider, 0);
        }
    }
    if sim.object_is_kind_of(rider, KindOfType::Hero) != FALSE {
        let core = c.open_core_mut();
        core.hero_units_contained = core.hero_units_contained.saturating_sub(1);
    }
    sync_contain_sidecar(c, sim, ctx);

    if c.is_enclosing() != FALSE {
        add_or_remove_obj_from_world(sim, rider, TRUE);
    }

    if let Some((c_pos, c_layer)) = sim
        .object(ctx.owner_slot)
        .map(|o| (*o.thing.get_position(), o.layer))
    {
        if c.is_enclosing() != FALSE {
            sim.set_object_position(rider, &c_pos);
        }
        sim.set_object_layer(rider, c_layer);
    }
    c.open_core_mut().last_unload_sound_frame = sim.frame();
    c.removing(sim, ctx, rider);
    on_removed_from(sim, rider);
}

/// `OpenContain::removeFromContain`.
pub fn remove_from_contain<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    rider: ObjectSlot,
    expose_stealthed: Bool,
) {
    let Some(rec) = sim.object(rider) else {
        return;
    };
    if rec.contained_by != Some(ctx.owner_slot) {
        return;
    }
    let rider_id = rec.id;
    if let Some(index) = c.open_core().contain_list.iter().position(|&id| id == rider_id) {
        remove_at(c, sim, ctx, index, expose_stealthed);
    }
}

/// `OpenContain::removeAllContained`.
pub fn remove_all_contained<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    expose_stealthed: Bool,
) {
    while !c.open_core().contain_list.is_empty() {
        remove_at(c, sim, ctx, 0, expose_stealthed);
    }
}

/// `OpenContain::killAllContained`: stealth and hero counts are deliberately left stale.
pub fn kill_all_contained<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) {
    while !c.open_core().contain_list.is_empty() {
        let rider_id = c.open_core_mut().contain_list.remove(0);
        c.open_core_mut().contain_list_size = c.open_core().contain_list_size.saturating_sub(1);
        sync_contain_sidecar(c, sim, ctx);

        if let Some(rider) = sim.find_object(rider_id) {
            c.removing(sim, ctx, rider);
            on_removed_from(sim, rider);
            kill_object(sim, rider, DamageType::Unresistable, DeathType::Normal);
        }
    }
    c.open_core_mut().contain_list.clear();
    c.open_core_mut().contain_list_size = 0;
    sync_contain_sidecar(c, sim, ctx);
}

/// `OpenContain::harmAndForceExitAllContained`.
pub fn harm_and_force_exit_all_contained<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    dmg: &DamageInfo,
) {
    while let Some(&rider_id) = c.open_core().contain_list.first() {
        if let Some(rider) = sim.find_object(rider_id) {
            c.remove_one(sim, ctx, rider, TRUE);
            let mut d = dmg.clone();
            sim.attempt_damage(rider, &mut d);
        }
        if c.open_core().contain_list.first() == Some(&rider_id) {
            c.open_core_mut().contain_list.remove(0);
            c.open_core_mut().contain_list_size =
                c.open_core().contain_list_size.saturating_sub(1);
            sync_contain_sidecar(c, sim, ctx);
        }
    }
}

/// `OpenContain::processDamageToContained`.
pub fn process_damage_to_contained<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    percent_damage: Real,
) {
    let death_type = if c.open_data().is_burned_death_to_units != FALSE {
        DeathType::Burned
    } else {
        DeathType::Normal
    };
    let kill_contained = percent_damage == 1.0;
    let owner_id = sim.object(ctx.owner_slot).map_or(ObjectId::INVALID, |o| o.id);
    let harm = |sim: &mut dyn SimApi, rider: ObjectSlot| {
        let mut d = DamageInfo::default();
        d.input.damage_type = DamageType::Unresistable;
        d.input.death_type = death_type;
        d.input.source_id = owner_id;
        d.input.amount = sim.object_max_health(rider) * percent_damage;
        sim.attempt_damage(rider, &mut d);
        if kill_contained
            && sim
                .object(rider)
                .is_some_and(|o| o.is_effectively_dead() == FALSE)
        {
            kill_object(sim, rider, DamageType::Unresistable, DeathType::Normal);
        }
    };

    #[cfg(feature = "retail-crc")]
    {
        let copy = c.open_core().contain_list.clone();
        for rider_id in copy {
            if let Some(rider) = sim.find_object(rider_id) {
                harm(sim, rider);
            }
        }
        let _ = ctx;
    }

    #[cfg(not(feature = "retail-crc"))]
    {
        // The list is detached while damaging so re-entrant removals cannot invalidate it.
        let mut detached = std::mem::take(&mut c.open_core_mut().contain_list);
        c.open_core_mut().contain_list_size = 0;
        sync_contain_sidecar(c, sim, ctx);

        let mut idx = 0;
        while idx < detached.len() {
            let Some(rider) = sim.find_object(detached[idx]) else {
                detached.remove(idx);
                continue;
            };
            harm(sim, rider);
            let remove_now = sim
                .object(rider)
                .is_none_or(|o| o.is_effectively_dead() != FALSE || o.is_destroyed());
            if remove_now {
                c.removing(sim, ctx, rider);
                on_removed_from(sim, rider);
                detached.remove(idx);
            } else {
                idx += 1;
            }
        }

        let size = detached.len() as UnsignedInt;
        c.open_core_mut().contain_list = detached;
        c.open_core_mut().contain_list_size = size;
        sync_contain_sidecar(c, sim, ctx);
    }
}

/// `OpenContain::scatterToNearbyPosition` (`AIUpdateInterface::ignoreObstacle` is unported).
pub fn scatter_to_nearby_position(sim: &mut dyn SimApi, container: ObjectSlot, rider: ObjectSlot) {
    let Some((c_pos, min_radius, c_layer)) = sim.object(container).map(|o| {
        (
            *o.thing.get_position(),
            o.geometry.get_bounding_circle_radius(),
            o.layer,
        )
    }) else {
        return;
    };
    let angle = sim.rng_logic_real(0.0, 2.0 * PI);
    let max_radius = min_radius + min_radius / 2.0;
    let dist = sim.rng_logic_real(min_radius, max_radius);
    let x = dist * gm_cosf(angle) + c_pos.x;
    let y = dist * gm_sinf(angle) + c_pos.y;
    let pos = Coord3D {
        x,
        y,
        z: sim.get_layer_height(x, y, c_layer),
    };
    sim.set_object_orientation(rider, angle);
    if sim.has_ai(rider) != FALSE {
        sim.set_object_position(rider, &c_pos);
        #[cfg(not(feature = "retail-crc"))]
        if let Some(o) = sim.object_mut(rider) {
            o.ai_goal_object = None;
        }
        sim.ai_move_to_position(rider, &pos, CommandSourceType::FromAi);
    } else {
        sim.set_object_position(rider, &pos);
    }
}

/// `OpenContain::exitObjectViaDoor` (headless art: exit bones resolve to the container).
///
/// Unported: the pathfinder reservation refresh for an idle vehicle container, and the exiting
/// unit's `ignoreObstacle` / `setIgnoreCollisionTime`.
pub fn exit_object_via_door<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    exit_obj: ObjectSlot,
) {
    c.remove_one(sim, ctx, exit_obj, FALSE);

    let door_open_time = c.open_data().door_open_time;
    c.open_core_mut().door_close_countdown = door_open_time;
    if door_open_time > 0 {
        let mut clear = ModelConditionFlags::new();
        clear.set(ModelConditionFlagType::Door1Closing as Int, TRUE as Int);
        let mut set = ModelConditionFlags::new();
        set.set(ModelConditionFlagType::Door1Opening as Int, TRUE as Int);
        sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
    }

    let number_exits = c.open_data().number_of_exit_paths;
    if number_exits <= 0 {
        sim.pathfinder_add_object(exit_obj);
        return;
    }
    if number_exits > 1 {
        let which = c.open_core().which_exit_path;
        c.open_core_mut().which_exit_path = (which % number_exits) + 1;
    }
    let Some((me_pos, me_angle, me_layer)) = sim.object(ctx.owner_slot).map(|o| {
        (
            *o.thing.get_position(),
            o.thing.get_orientation(),
            o.layer,
        )
    }) else {
        return;
    };
    let mut end_position = me_pos;
    let has_physics = sim.has_physics(exit_obj) != FALSE;
    let can_fall = sim.object(exit_obj).map_or(FALSE, |o| o.physics_allow_to_fall);

    sim.set_object_position(exit_obj, &me_pos);
    sim.set_object_orientation(exit_obj, me_angle);
    sim.set_object_layer(exit_obj, me_layer);
    sim.pathfinder_add_object(exit_obj);

    let has_ai = sim.has_ai(exit_obj) != FALSE;
    if has_ai {
        let _ = sim.pathfinder_adjust_to_possible_destination(exit_obj, &mut end_position);
    }
    let mut exit_path = vec![end_position, end_position];
    if c.open_core().rally_point_exists != FALSE {
        exit_path.push(c.open_core().rally_point);
    }
    if has_ai {
        if has_physics {
            sim.set_physics_allow_to_fall(exit_obj, FALSE);
        }
        sim.ai_follow_path(
            exit_obj,
            &exit_path,
            Some(ctx.owner_slot),
            CommandSourceType::FromAi,
        );
        let layer = sim.get_layer_for_destination(&end_position);
        sim.pathfinder_update_goal(exit_obj, &end_position, layer);
    }
    if has_physics {
        sim.set_physics_allow_to_fall(exit_obj, can_fall);
    }
}

/// `OpenContain::update`.
pub fn update_open_contain<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) -> UpdateSleepTime {
    c.open_core_mut().player_entered_mask = 0;

    // `monitorConditionChanges`.
    if let Some(current) = sim.object(ctx.owner_slot).map(|o| o.model_condition.clone()) {
        if current != c.open_core().condition_state {
            c.redeploy(sim, ctx);
            c.open_core_mut().condition_state = current;
        }
    }

    if c.open_core().door_close_countdown > 0 {
        c.open_core_mut().door_close_countdown -= 1;
        if c.open_core().door_close_countdown == 0 {
            let mut clear = ModelConditionFlags::new();
            clear.set(ModelConditionFlagType::Door1Opening as Int, TRUE as Int);
            let mut set = ModelConditionFlags::new();
            set.set(ModelConditionFlagType::Door1Closing as Int, TRUE as Int);
            sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
        }
    }

    // `pruneDeadWanters`.
    if !c.open_core().object_enter_exit_info.is_empty() {
        let dead_ids: Vec<ObjectId> = c
            .open_core()
            .object_enter_exit_info
            .keys()
            .copied()
            .filter(|&id| {
                sim.find_object(id)
                    .and_then(|slot| sim.object(slot))
                    .is_none_or(|o| o.is_effectively_dead() != FALSE)
            })
            .collect();
        for id in dead_ids {
            c.open_core_mut().object_enter_exit_info.remove(&id);
        }
    }

    UpdateSleepTime::NONE
}

/// `OpenContain::onDie`.
pub fn on_die<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    dmg: &DamageInfo,
) {
    if c.open_data()
        .die_mux
        .is_die_applicable(sim, ctx.owner_slot, dmg)
        == FALSE
    {
        return;
    }
    #[cfg(not(feature = "retail-crc"))]
    c.kill_riders_not_free_to_exit(sim, ctx);

    let percent = c.open_data().damage_percentage_to_units;
    if percent > 0.0 {
        process_damage_to_contained(c, sim, ctx, percent);
    }

    #[cfg(feature = "retail-crc")]
    c.kill_riders_not_free_to_exit(sim, ctx);

    c.remove_all(sim, ctx, FALSE);
}

/// `OpenContain::onCollide`.
pub fn on_collide<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    other: Option<ObjectSlot>,
) {
    let Some(other_slot) = other else {
        return;
    };
    if sim.object(other_slot).is_none_or(|o| o.ai_slot.is_none()) {
        return;
    }
    if sim.object_ai_enter_target(other_slot) != Some(ctx.owner_slot) {
        return;
    }
    let other_player = sim
        .object(other_slot)
        .map_or(0, |o| sim.player_index(o.team_id));

    let snapshot_ids = c.open_core().contain_list.clone();
    for rider_id in snapshot_ids {
        let Some(rider) = sim.find_object(rider_id) else {
            continue;
        };
        let Some(rec) = sim.object(rider) else {
            continue;
        };
        if sim.player_index(rec.team_id) != other_player {
            if rec.ai_slot.is_some() {
                if sim.object_is_kind_of(rider, KindOfType::StealthGarrison) != FALSE {
                    sim.mark_stealth_detected(rider, 0);
                }
                sim.order_ai_exit(rider, ctx.owner_slot, CommandSourceType::FromAi);
            } else {
                c.remove_one(sim, ctx, rider, TRUE);
            }
        }
    }

    if c.is_valid_container(sim, ctx, other_slot, TRUE) == FALSE {
        return;
    }
    add_to_contain(c, sim, ctx, other_slot);
}

/// `OpenContain::orderAllPassengersToExit`.
pub fn order_all_passengers_to_exit<C: ContainBehavior + ?Sized>(
    c: &mut C,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    cmd_source: CommandSourceType,
) {
    let ids = c.contained_view(sim, ctx).to_vec();
    for id in ids {
        if let Some(rider) = sim.find_object(id) {
            if sim.has_ai(rider) != FALSE {
                sim.order_ai_exit(rider, ctx.owner_slot, cmd_source);
            }
        }
    }
}

/// Shared `apply_deferred_op` for every `OpenContain` subclass.
pub fn apply_contain_deferred_op<C: ContainBehavior + ?Sized>(
    c: &mut C,
    op: DeferredSelfOp,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) {
    if let Some(dmg) = damage_info_from_die_op(&op) {
        c.die(sim, ctx, &dmg);
        return;
    }
    match op {
        DeferredSelfOp::ContainAdd(rider) => add_to_contain(c, sim, ctx, rider),
        DeferredSelfOp::ContainRemove {
            obj,
            expose_stealthed,
        } => c.remove_one(sim, ctx, obj, expose_stealthed),
        DeferredSelfOp::ContainRemoveAll { expose_stealthed } => {
            c.remove_all(sim, ctx, expose_stealthed);
        }
        DeferredSelfOp::ContainProcessDamage(percent) => {
            process_damage_to_contained(c, sim, ctx, percent);
        }
        _ => {}
    }
}

/// Implements [`BehaviorModule`] and every containment facet for a [`ContainBehavior`] type.
macro_rules! impl_contain_module {
    ($ty:ty $(, { $($extra:item)* })?) => {
        const _: () = {
            use $crate::arena::{DeferredSelfOp, ModuleContext, ModuleSidecar, ObjectSlot};
            use $crate::combat::damage::DamageInfo;
            use $crate::gamelogic::global_data::BodyDamageType;
            use $crate::modules::contain::open_contain as oc;
            use $crate::modules::contain::open_contain::ContainBehavior;
            use $crate::modules::module::{
                BehaviorModule, CollideFacet, CommandSourceType, ContainFacet, DamageFacet,
                DieFacet, EvacDisposition, ExitDoorType, ExitFacet, ObjectEnterExitType,
                TransportPassengerFacet, UpdateFacet,
            };
            use $crate::rules::disabled_types::DisabledMaskType;
            use $crate::rules::kind_of::KindOfType;
            use $crate::sim::{SimApi, SimView};
            use zh_common::math::Coord3D;
            use zh_common::types::{
                Bool, Int, NameKeyType, ObjectId, Real, TeamId, UnsignedInt, UnsignedShort,
                UpdateSleepTime, FALSE, TRUE,
            };
            use zh_common::xfer::{Xfer, XferError};

            impl BehaviorModule for $ty {
                fn module_name_key(&self) -> NameKeyType {
                    self.module_key()
                }
                fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
                    self.construct(sim, ctx);
                }
                fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
                    self.object_created(sim, ctx);
                }
                fn on_delete(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
                    self.delete(sim, ctx);
                }
                fn delete_child_objects(&self) -> Vec<ObjectId> {
                    self.open_core().contain_list.clone()
                }
                fn on_capture(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    old_team: TeamId,
                    new_team: TeamId,
                ) {
                    self.capture(sim, ctx, old_team, new_team);
                }
                fn apply_deferred_op(
                    &mut self,
                    op: DeferredSelfOp,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                ) {
                    oc::apply_contain_deferred_op(self, op, sim, ctx);
                }
                fn sync_sidecar(&self) -> ModuleSidecar {
                    self.open_core().sidecar(
                        self.contain_max_value(),
                        self.is_enclosing(),
                        self.is_garrisonable_container(),
                    )
                }
                fn crc(&self, xfer: &mut dyn Xfer) {
                    self.crc_module(xfer);
                }
                fn xfer(
                    &mut self,
                    xfer: &mut dyn Xfer,
                    sim: &dyn SimView,
                ) -> Result<(), XferError> {
                    self.xfer_module(xfer, sim)
                }
                fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
                    self.post_process(sim, ctx);
                }
                fn as_update(&self) -> Option<&dyn UpdateFacet> {
                    Some(self)
                }
                fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
                    Some(self)
                }
                fn as_contain(&self) -> Option<&dyn ContainFacet> {
                    Some(self)
                }
                fn as_contain_mut(&mut self) -> Option<&mut dyn ContainFacet> {
                    Some(self)
                }
                fn as_collide(&self) -> Option<&dyn CollideFacet> {
                    Some(self)
                }
                fn as_collide_mut(&mut self) -> Option<&mut dyn CollideFacet> {
                    Some(self)
                }
                $($($extra)*)?
                fn as_die(&self) -> Option<&dyn DieFacet> {
                    Some(self)
                }
                fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
                    Some(self)
                }
                fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
                    Some(self)
                }
                fn as_exit(&self) -> Option<&dyn ExitFacet> {
                    Some(self)
                }
                fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
                    Some(self)
                }
                fn as_transport_passenger(&self) -> Option<&dyn TransportPassengerFacet> {
                    if self.is_transport_passenger() != FALSE {
                        Some(self)
                    } else {
                        None
                    }
                }
                fn as_transport_passenger_mut(
                    &mut self,
                ) -> Option<&mut dyn TransportPassengerFacet> {
                    if self.is_transport_passenger() != FALSE {
                        Some(self)
                    } else {
                        None
                    }
                }
                fn as_any(&self) -> Option<&dyn std::any::Any> {
                    Some(self)
                }
                fn as_any_mut(&mut self) -> Option<&mut dyn std::any::Any> {
                    Some(self)
                }
            }

            impl UpdateFacet for $ty {
                fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
                    self.update_module(sim, ctx)
                }
                fn disabled_types_to_process(&self) -> DisabledMaskType {
                    self.update_disabled_types()
                }
            }

            impl CollideFacet for $ty {
                fn on_collide(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    other: Option<ObjectSlot>,
                    loc: Option<&Coord3D>,
                    normal: Option<&Coord3D>,
                ) {
                    self.collide(sim, ctx, other, loc, normal);
                }
            }

            impl DieFacet for $ty {
                fn is_die_applicable(
                    &self,
                    sim: &dyn SimView,
                    ctx: ModuleContext,
                    dmg: &DamageInfo,
                ) -> Bool {
                    self.open_data()
                        .die_mux
                        .is_die_applicable(sim, ctx.owner_slot, dmg)
                }
                fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
                    self.die(sim, ctx, dmg);
                }
            }

            impl DamageFacet for $ty {
                fn on_damage(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    dmg: &mut DamageInfo,
                ) {
                    self.damaged(sim, ctx, dmg);
                }
                fn on_body_damage_state_change(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    _dmg: &DamageInfo,
                    old_state: BodyDamageType,
                    new_state: BodyDamageType,
                ) {
                    self.body_damage_state_changed(sim, ctx, old_state, new_state);
                }
            }

            impl ExitFacet for $ty {
                fn is_exit_busy(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
                    self.exit_busy(sim, ctx)
                }
                fn reserve_door_for_exit(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    _tmpl_id: $crate::arena::ThingTemplateId,
                    specific_obj: Option<ObjectSlot>,
                ) -> ExitDoorType {
                    self.reserve_door(sim, ctx, specific_obj)
                }
                fn exit_object_via_door(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    new_obj: ObjectSlot,
                    exit_door: ExitDoorType,
                ) {
                    self.exit_via_door(sim, ctx, new_obj, exit_door);
                }
                fn exit_object_in_a_hurry(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    new_obj: ObjectSlot,
                ) {
                    oc::exit_object_via_door(self, sim, ctx, new_obj);
                }
                fn set_rally_point(&mut self, pos: &Coord3D) {
                    let core = self.open_core_mut();
                    core.rally_point = *pos;
                    core.rally_point_exists = TRUE;
                }
                fn get_rally_point(&self) -> Option<&Coord3D> {
                    let core = self.open_core();
                    if core.rally_point_exists != FALSE {
                        Some(&core.rally_point)
                    } else {
                        None
                    }
                }
                fn get_natural_rally_point(
                    &self,
                    sim: &dyn SimView,
                    ctx: ModuleContext,
                    rally_point: &mut Coord3D,
                    _offset: Bool,
                ) -> Bool {
                    if let Some(obj) = sim.object(ctx.owner_slot) {
                        *rally_point = *obj.thing.get_position();
                    }
                    TRUE
                }
            }

            impl TransportPassengerFacet for $ty {
                fn try_to_evacuate(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    expose_stealthed_units: Bool,
                ) -> Bool {
                    self.try_evacuate(sim, ctx, expose_stealthed_units)
                }
            }

            impl ContainFacet for $ty {
                fn is_weapon_bonus_passed_to_passengers(&self) -> Bool {
                    self.open_data().weapon_bonus_passed_to_passengers
                }
                fn get_contain_count(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
                    self.contain_count_view(sim, ctx)
                }
                fn get_contain_max(&self, sim: &dyn SimView, ctx: ModuleContext) -> Int {
                    self.contain_max_view(sim, ctx)
                }
                fn is_enclosing_container_for(&self, _other: ObjectSlot) -> Bool {
                    self.is_enclosing()
                }
                fn is_garrisonable(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
                    self.is_garrisonable_container()
                }
                fn is_bustable(&self) -> Bool {
                    self.is_bustable_container()
                }
                fn is_heal_contain(&self) -> Bool {
                    self.is_heal_container()
                }
                fn is_immune_to_clear_building_attacks(&self) -> Bool {
                    self.is_immune_to_clear()
                }
                fn is_special_zero_slot_container(&self) -> Bool {
                    self.is_special_zero_slot()
                }
                fn is_valid_container_for(
                    &self,
                    sim: &dyn SimView,
                    ctx: ModuleContext,
                    obj: ObjectSlot,
                    check_capacity: Bool,
                ) -> Bool {
                    self.is_valid_container(sim, ctx, obj, check_capacity)
                }
                fn get_contained_items<'a>(
                    &'a self,
                    sim: &'a dyn SimView,
                    ctx: ModuleContext,
                ) -> &'a [ObjectId] {
                    self.contained_view(sim, ctx)
                }
                fn add_to_contain(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    obj: ObjectSlot,
                ) {
                    oc::add_to_contain(self, sim, ctx, obj);
                }
                fn add_to_contain_list(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    obj: ObjectSlot,
                ) {
                    self.add_to_list(sim, ctx, obj);
                    let sidecar = BehaviorModule::sync_sidecar(self);
                    sim.set_module_sidecar(ctx.mod_slot, sidecar);
                }
                fn remove_from_contain(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    obj: ObjectSlot,
                    expose_stealthed: Bool,
                ) {
                    self.remove_one(sim, ctx, obj, expose_stealthed);
                }
                fn remove_all_contained(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    expose_stealthed: Bool,
                ) {
                    self.remove_all(sim, ctx, expose_stealthed);
                }
                fn kill_all_contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
                    self.kill_all(sim, ctx);
                }
                fn harm_and_force_exit_all_contained(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    dmg: &DamageInfo,
                ) {
                    self.harm_all(sim, ctx, dmg);
                }
                fn is_passenger_allowed_to_fire(
                    &self,
                    sim: &dyn SimView,
                    ctx: ModuleContext,
                    id: ObjectId,
                ) -> Bool {
                    self.passenger_allowed_to_fire(sim, ctx, id)
                }
                fn set_passenger_allowed_to_fire(&mut self, permission: Bool) {
                    self.open_core_mut().passenger_allowed_to_fire = permission;
                }
                fn get_stealth_units_contained(&self) -> UnsignedInt {
                    self.open_core().stealth_units_contained
                }
                fn get_hero_units_contained(&self, sim: &dyn SimView, ctx: ModuleContext) -> UnsignedInt {
                    self.hero_count_view(sim, ctx)
                }

                fn is_tunnel_contain(&self) -> Bool {
                    self.is_tunnel()
                }

                fn is_kick_out_on_capture(&self, _sim: &dyn SimView, _ctx: ModuleContext) -> Bool {
                    self.kicks_out_on_capture()
                }
                fn is_contained(
                    &self,
                    sim: &dyn SimView,
                    ctx: ModuleContext,
                    obj_id: ObjectId,
                ) -> Bool {
                    self.contains_view(sim, ctx, obj_id)
                }
                fn on_selling(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
                    self.selling(sim, ctx);
                }
                fn process_damage_to_contained(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    percent_damage: Real,
                ) {
                    oc::process_damage_to_contained(self, sim, ctx, percent_damage);
                }
                fn get_closest_rider(
                    &self,
                    sim: &dyn SimView,
                    pos: &Coord3D,
                ) -> Option<ObjectSlot> {
                    self.open_core().get_closest_rider(sim, pos)
                }
                fn order_all_passengers_to_exit(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    cmd_source: CommandSourceType,
                    _instantly: Bool,
                ) {
                    self.order_exit(sim, ctx, cmd_source);
                }
                fn order_all_passengers_to_idle(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                ) {
                    for id in self.contained_view(sim, ctx).to_vec() {
                        if let Some(rider) = sim.find_object(id) {
                            sim.ai_idle(rider, CommandSourceType::FromAi);
                        }
                    }
                }
                fn order_all_passengers_to_hack_internet(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                ) {
                    for id in self.contained_view(sim, ctx).to_vec() {
                        if let Some(rider) = sim.find_object(id) {
                            if sim.object_is_kind_of(rider, KindOfType::MoneyHacker) != FALSE {
                                sim.ai_hack_internet(rider, CommandSourceType::FromAi);
                            }
                        }
                    }
                }
                fn mark_all_passengers_detected(
                    &mut self,
                    sim: &mut dyn SimApi,
                    _ctx: ModuleContext,
                ) {
                    for id in self.open_core().contain_list.clone() {
                        if let Some(rider) = sim.find_object(id) {
                            if sim.object_is_kind_of(rider, KindOfType::StealthGarrison) != FALSE {
                                sim.mark_stealth_detected(rider, 0);
                            }
                        }
                    }
                }
                fn get_player_entered_mask(&self) -> UnsignedShort {
                    self.open_core().player_entered_mask
                }
                fn set_player_entered_mask(&mut self, mask: UnsignedShort) {
                    self.open_core_mut().player_entered_mask = mask;
                }
                fn on_object_wants_to_enter_or_exit(
                    &mut self,
                    obj_id: ObjectId,
                    wants: ObjectEnterExitType,
                ) {
                    let info = &mut self.open_core_mut().object_enter_exit_info;
                    if matches!(wants, ObjectEnterExitType::WantsNeither) {
                        info.remove(&obj_id);
                    } else {
                        info.insert(obj_id, wants);
                    }
                }
                fn has_objects_wanting_to_enter_or_exit(&self) -> Bool {
                    Bool::from(!self.open_core().object_enter_exit_info.is_empty())
                }
                fn on_containing(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    obj: ObjectSlot,
                ) {
                    self.contained(sim, ctx, obj);
                }
                fn on_removing(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                    obj: ObjectSlot,
                ) {
                    self.removing(sim, ctx, obj);
                }
                fn set_evac_disposition(&mut self, disp: EvacDisposition) {
                    self.set_evac(disp);
                }
                fn set_override_destination(&mut self, pos: &Coord3D) {
                    self.set_override_dest(pos);
                }
                fn contain_react_to_transform_change(
                    &mut self,
                    sim: &mut dyn SimApi,
                    ctx: ModuleContext,
                ) {
                    self.react_to_transform(sim, ctx);
                }
            }
        };
    };
}
pub(crate) use impl_contain_module;

/// Runtime instance of `OpenContain` (`PKT-282`).
pub struct OpenContain {
    pub data: OpenContainModuleData,
    pub core: OpenContainCore,
}

impl OpenContain {
    pub fn new(data: OpenContainModuleData) -> Self {
        let passenger_allowed_to_fire = data.passengers_allowed_to_fire;
        Self {
            data,
            core: OpenContainCore::new(passenger_allowed_to_fire),
        }
    }
}

impl ContainBehavior for OpenContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.core
    }
}

impl_contain_module!(OpenContain);

pub fn parse_open_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = OpenContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if let Some(res) = data.parse_field(field, r, parse_ctx) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

pub fn create_open_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<OpenContainModuleData>()
        .expect("OpenContain data type mismatch")
        .clone();
    Box::new(OpenContain::new(typed))
}
