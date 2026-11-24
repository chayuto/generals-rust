//! `PKT-282`: `OpenContainModuleData`, `OpenContainCore`, and `OpenContain`
//! (`OpenContain.h`, `OpenContain.cpp`).

use crate::arena::{DeferredSelfOp, FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::contain::contain_module::CONTAIN_MAX_UNKNOWN;
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{
    BehaviorModule, CollideFacet, CommandSourceType, ContainFacet, DamageFacet, DieFacet,
    ExitDoorType, ExitFacet, ModuleData, ObjectEnterExitType, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::{KindOfMaskType, KindOfType, KINDOF_NAMES};
use crate::rules::model_state::{
    ModelConditionFlagType, ModelConditionFlags, MODEL_CONDITION_NAMES,
};
use crate::sim::{SimApi, SimView};
use std::any::Any;
use std::collections::BTreeMap;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, UnsignedInt, UnsignedShort,
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

    pub fn sidecar(&self, contain_max: Int, enclosing: Bool) -> ModuleSidecar {
        ModuleSidecar::Contain {
            count: self.contain_list_size,
            max: contain_max,
            enclosing,
            stealth_units: self.stealth_units_contained,
            garrisoned: FALSE,
            rider_id: ObjectId::INVALID,
        }
    }

    #[inline]
    pub fn sync_sidecar_to_sim(&self, sim: &mut dyn SimApi, ctx: ModuleContext, contain_max: Int) {
        sim.set_module_sidecar(ctx.mod_slot, self.sidecar(contain_max, TRUE));
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

    pub fn add_to_contain(
        &mut self,
        data: &OpenContainModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        rider: ObjectSlot,
    ) {
        let Some(rec) = sim.object(rider) else {
            return;
        };
        if rec.is_destroyed() || rec.contained_by.is_some() {
            return;
        }
        let rider_team = rec.team_id;
        self.add_to_contain_list(sim, rider);
        self.player_entered_mask = sim.player_mask(rider_team);
        self.sync_sidecar_to_sim(sim, ctx, data.contain_max);

        self.redeploy_occupants(sim, ctx);

        if self.load_sounds_enabled != FALSE {
            self.last_load_sound_frame = sim.frame();
        }
        sim.set_contained_by(rider, Some(ctx.owner_slot));
    }

    fn redeploy_occupants(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
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

    pub fn remove_from_contain(
        &mut self,
        data: &OpenContainModuleData,
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
        let is_stealth_garrison =
            rec.thing
                .is_kind_of(&sim.rules().thing_factory, KindOfType::StealthGarrison)
                != FALSE;
        let is_hero =
            rec.thing
                .is_kind_of(&sim.rules().thing_factory, KindOfType::Hero)
                != FALSE;

        let Some(pos_idx) = self.contain_list.iter().position(|&id| id == rider_id) else {
            return;
        };
        self.contain_list.remove(pos_idx);
        self.contain_list_size = self.contain_list_size.saturating_sub(1);
        if is_stealth_garrison {
            self.stealth_units_contained = self.stealth_units_contained.saturating_sub(1);
            if expose_stealthed != FALSE {
                sim.mark_stealth_detected(rider, 0);
            }
        }
        if is_hero {
            self.hero_units_contained = self.hero_units_contained.saturating_sub(1);
        }
        self.sync_sidecar_to_sim(sim, ctx, data.contain_max);

        if let Some((c_pos, c_layer)) = sim
            .object(ctx.owner_slot)
            .map(|c| (*c.thing.get_position(), c.layer))
        {
            sim.set_object_position(rider, &c_pos);
            sim.set_object_layer(rider, c_layer);
        }
        self.last_unload_sound_frame = sim.frame();
        sim.set_contained_by(rider, None);
    }

    pub fn remove_all_contained(
        &mut self,
        data: &OpenContainModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        expose_stealthed: Bool,
    ) {
        while let Some(&rider_id) = self.contain_list.first() {
            if let Some(rider_slot) = sim.find_object(rider_id) {
                self.remove_from_contain(data, sim, ctx, rider_slot, expose_stealthed);
            } else {
                self.contain_list.remove(0);
                self.contain_list_size = self.contain_list_size.saturating_sub(1);
                self.sync_sidecar_to_sim(sim, ctx, data.contain_max);
            }
        }
    }

    pub fn kill_all_contained(
        &mut self,
        data: &OpenContainModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) {
        // Matches OpenContain::killAllContained: pops head & decrements contain_list_size without
        // decrementing stealth/hero counts before calling rider->kill().
        while !self.contain_list.is_empty() {
            let rider_id = self.contain_list.remove(0);
            self.contain_list_size = self.contain_list_size.saturating_sub(1);
            self.sync_sidecar_to_sim(sim, ctx, data.contain_max);

            if let Some(rider_slot) = sim.find_object(rider_id) {
                sim.set_contained_by(rider_slot, None);
                let mut kill_dmg = DamageInfo::default();
                kill_dmg.input.damage_type = DamageType::Unresistable;
                kill_dmg.input.death_type = DeathType::Normal;
                kill_dmg.input.amount = HUGE_DAMAGE_AMOUNT;
                sim.attempt_damage(rider_slot, &mut kill_dmg);
            }
        }
        self.contain_list.clear();
        self.contain_list_size = 0;
        self.sync_sidecar_to_sim(sim, ctx, data.contain_max);
    }

    pub fn harm_and_force_exit_all_contained(
        &mut self,
        data: &OpenContainModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) {
        while let Some(&rider_id) = self.contain_list.first() {
            if let Some(rider_slot) = sim.find_object(rider_id) {
                self.remove_from_contain(data, sim, ctx, rider_slot, TRUE);
                let mut d = dmg.clone();
                sim.attempt_damage(rider_slot, &mut d);
            } else {
                self.contain_list.remove(0);
                self.contain_list_size = self.contain_list_size.saturating_sub(1);
                self.sync_sidecar_to_sim(sim, ctx, data.contain_max);
            }
        }
    }

    pub fn process_damage_to_contained(
        &mut self,
        data: &OpenContainModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        percent_damage: Real,
    ) {
        let death_type = if data.is_burned_death_to_units != FALSE {
            DeathType::Burned
        } else {
            DeathType::Normal
        };
        let kill_contained = percent_damage == 1.0;
        let owner_id = sim.object(ctx.owner_slot).map_or(ObjectId::INVALID, |o| o.id);

        #[cfg(feature = "retail-crc")]
        {
            let copy = self.contain_list.clone();
            for rider_id in copy {
                let Some(rider_slot) = sim.find_object(rider_id) else {
                    continue;
                };
                let max_hp = sim.object_max_health(rider_slot);
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::Unresistable;
                d.input.death_type = death_type;
                d.input.source_id = owner_id;
                d.input.amount = max_hp * percent_damage;
                sim.attempt_damage(rider_slot, &mut d);
                if kill_contained
                    && sim
                        .object(rider_slot)
                        .is_some_and(|o| o.is_effectively_dead() == FALSE)
                {
                    let mut k = DamageInfo::default();
                    k.input.damage_type = DamageType::Unresistable;
                    k.input.death_type = DeathType::Normal;
                    k.input.amount = HUGE_DAMAGE_AMOUNT;
                    sim.attempt_damage(rider_slot, &mut k);
                }
            }
        }

        #[cfg(not(feature = "retail-crc"))]
        {
            let mut detached = std::mem::take(&mut self.contain_list);
            self.contain_list_size = 0;
            self.sync_sidecar_to_sim(sim, ctx, data.contain_max);

            let mut idx = 0;
            while idx < detached.len() {
                let rider_id = detached[idx];
                let Some(rider_slot) = sim.find_object(rider_id) else {
                    detached.remove(idx);
                    continue;
                };
                let max_hp = sim.object_max_health(rider_slot);
                let mut d = DamageInfo::default();
                d.input.damage_type = DamageType::Unresistable;
                d.input.death_type = death_type;
                d.input.source_id = owner_id;
                d.input.amount = max_hp * percent_damage;
                sim.attempt_damage(rider_slot, &mut d);

                if kill_contained
                    && sim
                        .object(rider_slot)
                        .is_some_and(|o| o.is_effectively_dead() == FALSE)
                {
                    let mut k = DamageInfo::default();
                    k.input.damage_type = DamageType::Unresistable;
                    k.input.death_type = DeathType::Normal;
                    k.input.amount = HUGE_DAMAGE_AMOUNT;
                    sim.attempt_damage(rider_slot, &mut k);
                }

                let remove_now = sim.object(rider_slot).is_none_or(|o| {
                    o.is_effectively_dead() != FALSE || o.is_destroyed()
                });
                if remove_now {
                    sim.set_contained_by(rider_slot, None);
                    detached.remove(idx);
                } else {
                    idx += 1;
                }
            }

            self.contain_list = detached;
            self.contain_list_size = self.contain_list.len() as UnsignedInt;
            self.sync_sidecar_to_sim(sim, ctx, data.contain_max);
        }
    }

    pub fn update_core(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) -> UpdateSleepTime {
        self.player_entered_mask = 0;

        if let Some(curr_cond) = sim.object(ctx.owner_slot).map(|o| o.model_condition.clone()) {
            if curr_cond != self.condition_state {
                self.redeploy_occupants(sim, ctx);
                self.condition_state = curr_cond;
            }
        }

        if self.door_close_countdown > 0 {
            self.door_close_countdown -= 1;
            if self.door_close_countdown == 0 {
                let mut clear = ModelConditionFlags::new();
                clear.set(ModelConditionFlagType::Door1Opening as Int, TRUE as Int);
                let mut set = ModelConditionFlags::new();
                set.set(ModelConditionFlagType::Door1Closing as Int, TRUE as Int);
                sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
            }
        }

        if !self.object_enter_exit_info.is_empty() {
            let dead_ids: Vec<ObjectId> = self
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
                self.object_enter_exit_info.remove(&id);
            }
        }

        UpdateSleepTime::NONE
    }

    pub fn on_collide(
        &mut self,
        data: &OpenContainModuleData,
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

        let snapshot_ids = self.contain_list.clone();
        for rider_id in snapshot_ids {
            let Some(rider_slot) = sim.find_object(rider_id) else {
                continue;
            };
            let Some(rider_rec) = sim.object(rider_slot) else {
                continue;
            };
            let rider_player = sim.player_index(rider_rec.team_id);
            if rider_player != other_player {
                let has_ai = rider_rec.ai_slot.is_some();
                let is_stealth_garrison = rider_rec
                    .thing
                    .is_kind_of(&sim.rules().thing_factory, KindOfType::StealthGarrison)
                    != FALSE;
                if has_ai {
                    if is_stealth_garrison {
                        sim.mark_stealth_detected(rider_slot, 0);
                    }
                    sim.order_ai_exit(rider_slot, ctx.owner_slot, CommandSourceType::FromAi);
                } else {
                    self.remove_from_contain(data, sim, ctx, rider_slot, TRUE);
                }
            }
        }

        if self.is_valid_container_for(data, sim, ctx, other_slot) == FALSE {
            return;
        }
        self.add_to_contain(data, sim, ctx, other_slot);
    }

    pub fn scatter_to_nearby_position(
        sim: &mut dyn SimApi,
        container: ObjectSlot,
        rider: ObjectSlot,
    ) {
        let Some((c_pos, c_radius, c_layer)) = sim.object(container).map(|c| {
            (
                *c.thing.get_position(),
                c.geometry.get_bounding_circle_radius(),
                c.layer,
            )
        }) else {
            return;
        };
        let angle = sim.rng_logic_real(0.0, 2.0 * std::f32::consts::PI);
        let max_radius = c_radius + c_radius / 2.0;
        let dist = sim.rng_logic_real(c_radius, max_radius);
        let x = dist * gm_cosf(angle) + c_pos.x;
        let y = dist * gm_sinf(angle) + c_pos.y;
        let z = sim.terrain().get_ground_height(x, y, None);
        let _ = c_layer;
        let pos = Coord3D { x, y, z };
        sim.set_object_orientation(rider, angle);
        sim.set_object_position(rider, &pos);
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
                sim.set_contained_by(slot, Some(ctx.owner_slot));
            }
        }
        self.sync_sidecar_to_sim(sim, ctx, data.contain_max);
    }
}

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

impl BehaviorModule for OpenContain {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.on_object_created(sim, ctx, self.data.contain_max);
    }

    fn on_delete(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        let ids = self.core.contain_list.clone();
        for id in ids {
            if let Some(rider_slot) = sim.find_object(id) {
                sim.destroy_object(rider_slot);
            }
        }
    }

    fn apply_deferred_op(
        &mut self,
        op: DeferredSelfOp,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) {
        match op {
            DeferredSelfOp::ContainAdd(rider) => {
                self.core.add_to_contain(&self.data, sim, ctx, rider);
            }
            DeferredSelfOp::ContainRemove {
                obj,
                expose_stealthed,
            } => {
                self.core
                    .remove_from_contain(&self.data, sim, ctx, obj, expose_stealthed);
            }
            DeferredSelfOp::ContainRemoveAll { expose_stealthed } => {
                self.core
                    .remove_all_contained(&self.data, sim, ctx, expose_stealthed);
            }
            _ => {}
        }
    }

    fn sync_sidecar(&self) -> ModuleSidecar {
        self.core.sidecar(self.data.contain_max, TRUE)
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.core.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.core.xfer(xfer, sim)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.load_post_process(&self.data, sim, ctx);
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

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_exit(&self) -> Option<&dyn ExitFacet> {
        Some(self)
    }

    fn as_exit_mut(&mut self) -> Option<&mut dyn ExitFacet> {
        Some(self)
    }
}

impl UpdateFacet for OpenContain {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.core.update_core(sim, ctx)
    }
}

impl CollideFacet for OpenContain {
    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        self.core.on_collide(&self.data, sim, ctx, other);
    }
}

impl DieFacet for OpenContain {
    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.data.die_mux.is_die_applicable(sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self
            .data
            .die_mux
            .is_die_applicable(sim, ctx.owner_slot, dmg)
            == FALSE
        {
            return;
        }
        if self.data.damage_percentage_to_units > 0.0 {
            self.core.process_damage_to_contained(
                &self.data,
                sim,
                ctx,
                self.data.damage_percentage_to_units,
            );
        }
        self.core.remove_all_contained(&self.data, sim, ctx, FALSE);
    }
}

impl DamageFacet for OpenContain {
    fn on_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}
}

impl ExitFacet for OpenContain {
    fn reserve_door_for_exit(
        &mut self,
        _sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _tmpl_id: crate::arena::ThingTemplateId,
        _specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        ExitDoorType::Door1
    }

    fn exit_object_via_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
        _exit_door: ExitDoorType,
    ) {
        self.core
            .remove_from_contain(&self.data, sim, ctx, new_obj, FALSE);
        self.core.door_close_countdown = self.data.door_open_time;
        if self.core.door_close_countdown > 0 {
            let mut clear = ModelConditionFlags::new();
            clear.set(ModelConditionFlagType::Door1Closing as Int, TRUE as Int);
            let mut set = ModelConditionFlags::new();
            set.set(ModelConditionFlagType::Door1Opening as Int, TRUE as Int);
            sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
        }
        if self.data.number_of_exit_paths > 0 {
            self.core.which_exit_path += 1;
            if self.core.which_exit_path > self.data.number_of_exit_paths {
                self.core.which_exit_path = 1;
            }
        }
    }

    fn exit_object_in_a_hurry(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        new_obj: ObjectSlot,
    ) {
        self.exit_object_via_door(sim, ctx, new_obj, ExitDoorType::Door1);
    }

    fn set_rally_point(&mut self, pos: &Coord3D) {
        self.core.rally_point = *pos;
        self.core.rally_point_exists = TRUE;
    }

    fn get_rally_point(&self) -> Option<&Coord3D> {
        if self.core.rally_point_exists != FALSE {
            Some(&self.core.rally_point)
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

impl ContainFacet for OpenContain {
    fn get_contain_count(&self) -> UnsignedInt {
        self.core.contain_list_size
    }

    fn get_contain_max(&self) -> Int {
        self.data.contain_max
    }

    fn is_enclosing_container_for(&self, _other: ObjectSlot) -> Bool {
        TRUE
    }

    fn is_valid_container_for(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        obj: ObjectSlot,
        _check_capacity: Bool,
    ) -> Bool {
        self.core.is_valid_container_for(&self.data, sim, ctx, obj)
    }

    fn get_contained_items(&self) -> &[ObjectId] {
        &self.core.contain_list
    }

    fn add_to_contain(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        self.core.add_to_contain(&self.data, sim, ctx, obj);
    }

    fn add_to_contain_list(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, obj: ObjectSlot) {
        self.core.add_to_contain_list(sim, obj);
        self.core
            .sync_sidecar_to_sim(sim, ctx, self.data.contain_max);
    }

    fn remove_from_contain(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        obj: ObjectSlot,
        expose_stealthed: Bool,
    ) {
        self.core
            .remove_from_contain(&self.data, sim, ctx, obj, expose_stealthed);
    }

    fn remove_all_contained(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        expose_stealthed: Bool,
    ) {
        self.core
            .remove_all_contained(&self.data, sim, ctx, expose_stealthed);
    }

    fn kill_all_contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.kill_all_contained(&self.data, sim, ctx);
    }

    fn harm_and_force_exit_all_contained(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) {
        self.core
            .harm_and_force_exit_all_contained(&self.data, sim, ctx, dmg);
    }

    fn is_passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        _id: ObjectId,
    ) -> Bool {
        if self.core.passenger_allowed_to_fire == FALSE {
            return FALSE;
        }
        if let Some(parent_slot) = sim.object(ctx.owner_slot).and_then(|o| o.contained_by) {
            if let Some(parent) = sim.object(parent_slot) {
                return parent.passengers_allowed_to_fire;
            }
        }
        TRUE
    }

    fn set_passenger_allowed_to_fire(&mut self, permission: Bool) {
        self.core.passenger_allowed_to_fire = permission;
    }

    fn get_stealth_units_contained(&self) -> UnsignedInt {
        self.core.stealth_units_contained
    }

    fn get_hero_units_contained(&self) -> UnsignedInt {
        self.core.hero_units_contained
    }

    fn is_contained(&self, obj_id: ObjectId) -> Bool {
        Bool::from(self.core.contain_list.contains(&obj_id))
    }

    fn on_selling(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.order_all_passengers_to_exit(sim, ctx, CommandSourceType::FromAi, FALSE);
    }

    fn process_damage_to_contained(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        percent_damage: Real,
    ) {
        self.core
            .process_damage_to_contained(&self.data, sim, ctx, percent_damage);
    }

    fn get_closest_rider(&self, sim: &dyn SimView, pos: &Coord3D) -> Option<ObjectSlot> {
        let mut best: Option<(ObjectSlot, Real)> = None;
        for &id in &self.core.contain_list {
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

    fn order_all_passengers_to_exit(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        cmd_source: CommandSourceType,
        _instantly: Bool,
    ) {
        let ids = self.core.contain_list.clone();
        for id in ids {
            if let Some(rider) = sim.find_object(id) {
                if sim.object(rider).is_some_and(|o| o.ai_slot.is_some()) {
                    sim.order_ai_exit(rider, ctx.owner_slot, cmd_source);
                }
            }
        }
    }

    fn mark_all_passengers_detected(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        let ids = self.core.contain_list.clone();
        for id in ids {
            if let Some(rider) = sim.find_object(id) {
                if sim.object_is_kind_of(rider, KindOfType::StealthGarrison) != FALSE {
                    sim.mark_stealth_detected(rider, 0);
                }
            }
        }
    }

    fn get_player_entered_mask(&self) -> UnsignedShort {
        self.core.player_entered_mask
    }

    fn set_player_entered_mask(&mut self, mask: UnsignedShort) {
        self.core.player_entered_mask = mask;
    }

    fn on_object_wants_to_enter_or_exit(
        &mut self,
        obj_id: ObjectId,
        wants: ObjectEnterExitType,
    ) {
        if matches!(wants, ObjectEnterExitType::WantsNeither) {
            self.core.object_enter_exit_info.remove(&obj_id);
        } else {
            self.core.object_enter_exit_info.insert(obj_id, wants);
        }
    }

    fn has_objects_wanting_to_enter_or_exit(&self) -> Bool {
        Bool::from(!self.core.object_enter_exit_info.is_empty())
    }
}

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
