//! `PKT-295`: `SlowDeathBehaviorModuleData`, `SlowDeathCore`, and `SlowDeathBehavior`
//! (`SlowDeathBehavior.h`, `SlowDeathBehavior.cpp`).

use crate::object::weapon_fire;
use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::behavior::behavior_module::{
    resolve_optional_asset_name, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::die::die_module::DieMuxData;
use crate::modules::module::{
    BehaviorModule, DieFacet, ModuleData, SlowDeathFacet, UpdateFacet,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::disabled_types::{disabled_mask_all, DisabledMaskType, DisabledType};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, LOGICFRAMES_PER_SECOND, SEPS_DEFAULT};
use zh_common::math::{gm_atan2f, gm_cosf, gm_sinf, Coord3D};
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

const BEGIN_MIDPOINT_RATIO: Real = 0.35;
const END_MIDPOINT_RATIO: Real = 0.65;
const PLAYERMASK_ALL: u16 = 0xffff;

pub const SLOW_DEATH_PHASE_NAMES: [&str; 3] = ["INITIAL", "MIDPOINT", "FINAL"];

/// Phases of a `SlowDeathBehavior` sequence (`SlowDeathPhaseType`).
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlowDeathPhaseType {
    Initial = 0,
    Midpoint = 1,
    Final = 2,
}

impl SlowDeathPhaseType {
    pub const COUNT: usize = 3;
}

pub const SD_FLAG_SLOW_DEATH_ACTIVATED: u32 = 1 << 0;
pub const SD_FLAG_MIDPOINT_EXECUTED: u32 = 1 << 1;
pub const SD_FLAG_FLUNG_INTO_AIR: u32 = 1 << 2;
pub const SD_FLAG_BOUNCED: u32 = 1 << 3;

pub const SD_EFFECT_HAS_FX: u32 = 1 << 0;
pub const SD_EFFECT_HAS_OCL: u32 = 1 << 1;
pub const SD_EFFECT_HAS_WEAPON: u32 = 1 << 2;

/// INI template data matching `SlowDeathBehaviorModuleData` (`SlowDeathBehavior.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct SlowDeathBehaviorModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub sink_rate: Real,
    pub probability_modifier: Int,
    pub modifier_bonus_per_overkill_percent: Real,
    pub sink_delay: UnsignedInt,
    pub sink_delay_variance: UnsignedInt,
    pub destruction_delay: UnsignedInt,
    pub destruction_delay_variance: UnsignedInt,
    pub destruction_altitude: Real,
    pub mask_of_loaded_effects: UnsignedInt,
    pub fx: [Vec<Option<String>>; SlowDeathPhaseType::COUNT],
    pub ocls: [Vec<Option<String>>; SlowDeathPhaseType::COUNT],
    pub weapons: [Vec<Option<String>>; SlowDeathPhaseType::COUNT],
    pub fling_force: Real,
    pub fling_force_variance: Real,
    pub fling_pitch: Real,
    pub fling_pitch_variance: Real,
}

impl SlowDeathBehaviorModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            die_mux: DieMuxData::new(),
            sink_rate: 0.0,
            probability_modifier: 10,
            modifier_bonus_per_overkill_percent: 0.0,
            sink_delay: 0,
            sink_delay_variance: 0,
            destruction_delay: 0,
            destruction_delay_variance: 0,
            destruction_altitude: -10.0,
            mask_of_loaded_effects: 0,
            fx: [Vec::new(), Vec::new(), Vec::new()],
            ocls: [Vec::new(), Vec::new(), Vec::new()],
            weapons: [Vec::new(), Vec::new(), Vec::new()],
            fling_force: 0.0,
            fling_force_variance: 0.0,
            fling_pitch: 0.0,
            fling_pitch_variance: 0.0,
        }
    }

    #[inline]
    pub const fn has_non_lod_effects(&self) -> bool {
        (self.mask_of_loaded_effects & (SD_EFFECT_HAS_OCL | SD_EFFECT_HAS_WEAPON)) != 0
    }

    /// Parses one `SlowDeathBehaviorModuleData` INI field, returning `None` if unhandled.
    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        if field.eq_ignore_ascii_case("SinkRate") {
            Some(reader.parse_velocity_real(&mut self.sink_rate))
        } else if field.eq_ignore_ascii_case("ProbabilityModifier") {
            Some(reader.parse_int(&mut self.probability_modifier))
        } else if field.eq_ignore_ascii_case("ModifierBonusPerOverkillPercent") {
            Some(reader.parse_percent_to_real(&mut self.modifier_bonus_per_overkill_percent))
        } else if field.eq_ignore_ascii_case("SinkDelay") {
            Some(reader.parse_duration_unsigned_int(&mut self.sink_delay))
        } else if field.eq_ignore_ascii_case("SinkDelayVariance") {
            Some(reader.parse_duration_unsigned_int(&mut self.sink_delay_variance))
        } else if field.eq_ignore_ascii_case("DestructionDelay") {
            Some(reader.parse_duration_unsigned_int(&mut self.destruction_delay))
        } else if field.eq_ignore_ascii_case("DestructionDelayVariance") {
            Some(reader.parse_duration_unsigned_int(&mut self.destruction_delay_variance))
        } else if field.eq_ignore_ascii_case("DestructionAltitude") {
            Some(reader.parse_real(&mut self.destruction_altitude))
        } else if field.eq_ignore_ascii_case("FX") {
            Some(self.parse_fx(reader, parse_ctx))
        } else if field.eq_ignore_ascii_case("OCL") {
            Some(self.parse_ocl(reader, parse_ctx))
        } else if field.eq_ignore_ascii_case("Weapon") {
            Some(self.parse_weapon(reader, parse_ctx))
        } else if field.eq_ignore_ascii_case("FlingForce") {
            Some(reader.parse_real(&mut self.fling_force))
        } else if field.eq_ignore_ascii_case("FlingForceVariance") {
            Some(reader.parse_real(&mut self.fling_force_variance))
        } else if field.eq_ignore_ascii_case("FlingPitch") {
            Some(reader.parse_angle_real(&mut self.fling_pitch))
        } else if field.eq_ignore_ascii_case("FlingPitchVariance") {
            Some(reader.parse_angle_real(&mut self.fling_pitch_variance))
        } else {
            self.die_mux.parse_field(field, reader)
        }
    }

    fn parse_phase(reader: &mut IniReader<'_>) -> Result<usize, IniError> {
        let mut phase_idx: Int = 0;
        reader.parse_index_list(&mut phase_idx, &SLOW_DEATH_PHASE_NAMES)?;
        if phase_idx < 0 || (phase_idx as usize) >= SlowDeathPhaseType::COUNT {
            return Err(IniError::InvalidData);
        }
        Ok(phase_idx as usize)
    }

    fn parse_fx(
        &mut self,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Result<(), IniError> {
        let phase = Self::parse_phase(reader)?;
        let first = reader.next_token(SEPS_DEFAULT)?;
        let mut cur = Some(first);
        while let Some(tok) = cur {
            let asset = if tok.eq_ignore_ascii_case("None") {
                None
            } else {
                resolve_optional_asset_name(tok, parse_ctx.fx_list_names)
            };
            if asset.is_some() {
                self.mask_of_loaded_effects |= SD_EFFECT_HAS_FX;
            }
            self.fx[phase].push(asset);
            cur = reader.next_token_or_null(SEPS_DEFAULT);
        }
        Ok(())
    }

    fn parse_ocl(
        &mut self,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Result<(), IniError> {
        let phase = Self::parse_phase(reader)?;
        let first = reader.next_token(SEPS_DEFAULT)?;
        let mut cur = Some(first);
        while let Some(tok) = cur {
            let asset = if tok.eq_ignore_ascii_case("None") {
                None
            } else {
                resolve_optional_asset_name(tok, parse_ctx.ocl_names)
            };
            if asset.is_some() {
                self.mask_of_loaded_effects |= SD_EFFECT_HAS_OCL;
            }
            self.ocls[phase].push(asset);
            cur = reader.next_token_or_null(SEPS_DEFAULT);
        }
        Ok(())
    }

    fn parse_weapon(
        &mut self,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Result<(), IniError> {
        let phase = Self::parse_phase(reader)?;
        let first = reader.next_token(SEPS_DEFAULT)?;
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
            if weapon.is_some() {
                self.mask_of_loaded_effects |= SD_EFFECT_HAS_WEAPON;
            }
            self.weapons[phase].push(weapon);
            cur = reader.next_token_or_null(SEPS_DEFAULT);
        }
        Ok(())
    }
}

impl ModuleData for SlowDeathBehaviorModuleData {
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
        FacetMask(0x003 | FacetMask::SLOW_DEATH.0)
    }
}

/// Reusable runtime core state for `SlowDeathBehavior` and its `WP-5.3-B` subclasses.
#[derive(Debug, Clone, PartialEq)]
pub struct SlowDeathCore {
    pub sink_frame: UnsignedInt,
    pub midpoint_frame: UnsignedInt,
    pub destruction_frame: UnsignedInt,
    pub accelerated_time_scale: Real,
    pub flags: UnsignedInt,
    pub schedule: UpdateScheduleTracker,
}

impl SlowDeathCore {
    #[inline]
    pub const fn new() -> Self {
        Self {
            sink_frame: 0,
            midpoint_frame: 0,
            destruction_frame: 0,
            accelerated_time_scale: 1.0,
            flags: 0,
            schedule: UpdateScheduleTracker::new(),
        }
    }

    #[inline]
    pub const fn is_slow_death_activated(&self) -> Bool {
        if (self.flags & SD_FLAG_SLOW_DEATH_ACTIVATED) != 0 {
            TRUE
        } else {
            FALSE
        }
    }

    #[inline]
    pub fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    #[inline]
    pub fn is_die_applicable(
        &self,
        data: &SlowDeathBehaviorModuleData,
        sim: &dyn SimView,
        owner: ObjectSlot,
        dmg: &DamageInfo,
    ) -> Bool {
        data.die_mux.is_die_applicable(sim, owner, dmg)
    }

    /// Computes weighted death probability including overkill bonus (`SlowDeathBehavior::getProbabilityModifier`).
    pub fn get_probability_modifier(
        &self,
        data: &SlowDeathBehaviorModuleData,
        sim: &dyn SimView,
        owner: ObjectSlot,
        dmg: &DamageInfo,
    ) -> Int {
        let overkill: Int = (dmg.output.actual_damage_dealt - dmg.output.actual_damage_clipped) as Int;
        let max_health = sim.object_max_health(owner);
        let overkill_percent: Real = if max_health == 0.0 {
            0.0
        } else {
            (overkill as Real) / max_health
        };
        let overkill_modifier: Int =
            (overkill_percent * data.modifier_bonus_per_overkill_percent) as Int;
        (data.probability_modifier + overkill_modifier).max(1)
    }

    fn calc_random_force(
        sim: &mut dyn SimApi,
        min_mag: Real,
        max_mag: Real,
        min_pitch: Real,
        max_pitch: Real,
    ) -> Coord3D {
        let angle = sim.rng_logic_real(-std::f32::consts::PI, std::f32::consts::PI);
        let pitch = sim.rng_logic_real(min_pitch, max_pitch);
        let mag = sim.rng_logic_real(min_mag, max_mag);

        // Mirrors WWMath::Matrix3D::Make_Identity -> Scale(mag) -> Rotate_Z(angle) -> Rotate_Y(-pitch) -> Get_X_Vector.
        let cos_a = gm_cosf(angle);
        let sin_a = gm_sinf(angle);
        let r00 = cos_a * mag;
        let r10 = sin_a * mag;

        let sin_p = gm_sinf(-pitch);
        let cos_p = gm_cosf(-pitch);
        Coord3D {
            x: cos_p * r00,
            y: cos_p * r10,
            z: -sin_p * mag,
        }
    }

    /// Executes FX/OCL/Weapon selections for one slow-death phase (`SlowDeathBehavior::doPhaseStuff`).
    pub fn do_phase_stuff(
        data: &SlowDeathBehaviorModuleData,
        sim: &mut dyn SimApi,
        owner: ObjectSlot,
        phase: SlowDeathPhaseType,
    ) {
        if data.mask_of_loaded_effects == 0 {
            return;
        }
        let phase_idx = phase as usize;
        let pos = sim
            .object(owner)
            .map(|o| *o.thing.get_position())
            .unwrap_or_default();

        let fx_vec = &data.fx[phase_idx];
        if !fx_vec.is_empty() {
            let idx = sim.rng_logic_int(0, (fx_vec.len() as Int) - 1) as usize;
            if let Some(ref fx_name) = fx_vec[idx] {
                sim.trigger_fx_list(fx_name, &pos);
            }
        }

        let ocl_vec = &data.ocls[phase_idx];
        if !ocl_vec.is_empty() {
            let idx = sim.rng_logic_int(0, (ocl_vec.len() as Int) - 1) as usize;
            if let Some(ref ocl_name) = ocl_vec[idx] {
                sim.spawn_ocl(ocl_name, owner, &pos, &pos);
            }
        }

        let wpn_vec = &data.weapons[phase_idx];
        if !wpn_vec.is_empty() {
            let idx = sim.rng_logic_int(0, (wpn_vec.len() as Int) - 1) as usize;
            if let Some(ref wpn_name) = wpn_vec[idx] {
                weapon_fire::create_and_fire_temp_weapon(sim, owner, wpn_name, &pos);
            }
        }
    }

    /// Begins the slow death timer and initial phase (`SlowDeathBehavior::beginSlowDeath`).
    pub fn begin_slow_death(
        &mut self,
        data: &SlowDeathBehaviorModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _dmg: &DamageInfo,
    ) {
        if self.is_slow_death_activated() != FALSE {
            return;
        }
        let owner = ctx.owner_slot;
        let time_scale = sim.slow_death_scale();
        self.accelerated_time_scale = 1.0;

        if time_scale == 0.0 && !data.has_non_lod_effects() {
            sim.destroy_object(owner);
            return;
        }

        let is_hulk = sim.object_is_kind_of(owner, KindOfType::Hulk) != FALSE;
        if is_hulk && sim.hulk_max_lifetime_override() != -1 {
            self.sink_frame = 1;
            self.midpoint_frame = ((LOGICFRAMES_PER_SECOND as UnsignedInt) / 2) + 1;
            self.destruction_frame = (LOGICFRAMES_PER_SECOND as UnsignedInt) + 1;
            self.accelerated_time_scale = 1.0;
        } else {
            let sink_var = sim.rng_logic_int(0, data.sink_delay_variance as Int) as UnsignedInt;
            self.sink_frame = (time_scale * ((data.sink_delay + sink_var) as Real)) as UnsignedInt;

            let dest_var =
                sim.rng_logic_int(0, data.destruction_delay_variance as Int) as UnsignedInt;
            self.destruction_frame =
                (time_scale * ((data.destruction_delay + dest_var) as Real)) as UnsignedInt;

            let lo = (BEGIN_MIDPOINT_RATIO * (self.destruction_frame as Real)) as Int;
            let hi = (END_MIDPOINT_RATIO * (self.destruction_frame as Real)) as Int;
            self.midpoint_frame = sim.rng_logic_int(lo, hi) as UnsignedInt;
            self.accelerated_time_scale = time_scale;
        }

        let now = sim.frame();

        if data.fling_force > 0.0 {
            let (is_held, has_physics, mut pos) = sim
                .object(owner)
                .map(|o| {
                    (
                        o.disabled_mask.test(DisabledType::Held as Int) != FALSE,
                        o.physics_slot.is_some(),
                        *o.thing.get_position(),
                    )
                })
                .unwrap_or((false, false, Coord3D::default()));

            if is_held {
                sim.notify_slaver_die(owner);
            }

            if has_physics {
                const MIN_ALTITUDE: Real = 1.0;
                if sim.object_height_above_terrain(owner) < MIN_ALTITUDE {
                    pos.z += MIN_ALTITUDE;
                    sim.set_object_position(owner, &pos);
                }

                let force = Self::calc_random_force(
                    sim,
                    data.fling_force,
                    data.fling_force + data.fling_force_variance,
                    data.fling_pitch,
                    data.fling_pitch + data.fling_pitch_variance,
                );
                let orientation = gm_atan2f(force.y, force.x);
                sim.apply_fling_physics(owner, &force, orientation);
                sim.set_model_condition_state(owner, ModelConditionFlagType::ExplodedFlailing);
                self.flags |= SD_FLAG_FLUNG_INTO_AIR;
            }
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        } else {
            let mut when_to_wake = self.sink_frame as Int;
            if when_to_wake > (self.destruction_frame as Int) {
                when_to_wake = self.destruction_frame as Int;
            }
            if when_to_wake > (self.midpoint_frame as Int) {
                when_to_wake = self.midpoint_frame as Int;
            }
            // `UPDATE_SLEEP(0)` is legal here and wakes the module this very frame.
            sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime(when_to_wake as UnsignedInt));
        }

        self.sink_frame = self.sink_frame.wrapping_add(now);
        self.destruction_frame = self.destruction_frame.wrapping_add(now);
        self.midpoint_frame = self.midpoint_frame.wrapping_add(now);

        self.flags |= SD_FLAG_SLOW_DEATH_ACTIVATED;

        Self::do_phase_stuff(data, sim, owner, SlowDeathPhaseType::Initial);
    }

    /// `SlowDeathBehavior::onDie` up to the virtual `beginSlowDeath` call: returns `TRUE` when
    /// this module won the selection and the caller must run its own `begin_slow_death`.
    pub fn on_die(
        &self,
        data: &SlowDeathBehaviorModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        if self.is_die_applicable(data, sim, ctx.owner_slot, dmg) == FALSE {
            return FALSE;
        }
        if sim.object(ctx.owner_slot).is_some_and(|o| o.ai_slot.is_some()) {
            if sim.ai_is_in_dead_state(ctx.owner_slot) != FALSE {
                return FALSE;
            }
            sim.ai_mark_as_dead(ctx.owner_slot);
        }
        sim.deselect_object(ctx.owner_slot, PLAYERMASK_ALL, TRUE);

        let caller_prob = self.get_probability_modifier(data, sim, ctx.owner_slot, dmg);
        sim.trigger_slow_death_with_caller(ctx.owner_slot, ctx.mod_slot, TRUE, caller_prob, dmg)
    }

    /// Standard `SlowDeathBehavior::update` tick shared with subclasses.
    pub fn update_core(
        &mut self,
        data: &SlowDeathBehaviorModuleData,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) -> UpdateSleepTime {
        let owner = ctx.owner_slot;
        let time_scale = sim.slow_death_scale();

        if time_scale != 1.0 && self.accelerated_time_scale == 1.0 && !data.has_non_lod_effects() {
            if time_scale == 0.0 {
                sim.destroy_object(owner);
                return UpdateSleepTime::NONE;
            }
            self.sink_frame = ((self.sink_frame as Real) * time_scale) as UnsignedInt;
            self.midpoint_frame = ((self.midpoint_frame as Real) * time_scale) as UnsignedInt;
            self.destruction_frame = ((self.destruction_frame as Real) * time_scale) as UnsignedInt;
            self.accelerated_time_scale = time_scale;
        }

        let now = sim.frame();

        if (self.flags & SD_FLAG_FLUNG_INTO_AIR) != 0 && (self.flags & SD_FLAG_BOUNCED) == 0 {
            self.sink_frame = self.sink_frame.wrapping_add(1);
            self.midpoint_frame = self.midpoint_frame.wrapping_add(1);
            self.destruction_frame = self.destruction_frame.wrapping_add(1);

            if sim.object_is_above_terrain(owner) == FALSE {
                let mut clear = crate::rules::model_state::ModelConditionFlags::new();
                clear.set(ModelConditionFlagType::ExplodedFlailing as Int, TRUE as Int);
                let mut set = crate::rules::model_state::ModelConditionFlags::new();
                set.set(ModelConditionFlagType::ExplodedBouncing as Int, TRUE as Int);
                sim.clear_and_set_model_condition_flags(owner, &clear, &set);
                self.flags |= SD_FLAG_BOUNCED;
            }

            let tree_id = sim.physics_last_collidee(owner);
            if let Some(tree_slot) = sim.find_object(tree_id) {
                if sim.object_is_kind_of(tree_slot, KindOfType::Shrubbery) != FALSE {
                    sim.set_disabled(owner, DisabledType::Held);
                    sim.clear_model_condition_state(
                        owner,
                        ModelConditionFlagType::ExplodedFlailing,
                    );
                    sim.clear_model_condition_state(
                        owner,
                        ModelConditionFlagType::ExplodedBouncing,
                    );
                    sim.set_model_condition_state(owner, ModelConditionFlagType::Parachuting);

                    if let Some(mut pos) = sim.object(owner).map(|o| *o.thing.get_position()) {
                        pos.z -= data.sink_rate * 50.0;
                        sim.set_object_position(owner, &pos);
                    }
                    if sim.object_is_above_terrain(owner) == FALSE {
                        sim.destroy_object(owner);
                    }
                }
            }
        }

        if now >= self.sink_frame && data.sink_rate > 0.0 {
            sim.set_disabled(owner, DisabledType::Held);
            if let Some(mut pos) = sim.object(owner).map(|o| *o.thing.get_position()) {
                pos.z -= data.sink_rate / self.accelerated_time_scale;
                sim.set_object_position(owner, &pos);
            }
        }

        if now >= self.midpoint_frame && (self.flags & SD_FLAG_MIDPOINT_EXECUTED) == 0 {
            Self::do_phase_stuff(data, sim, owner, SlowDeathPhaseType::Midpoint);
            self.flags |= SD_FLAG_MIDPOINT_EXECUTED;
        }

        if now >= self.destruction_frame {
            Self::do_phase_stuff(data, sim, owner, SlowDeathPhaseType::Final);
            sim.destroy_object(owner);
        }

        UpdateSleepTime::NONE
    }

    pub fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    /// Serializes `SlowDeathBehavior::xfer` (version `1`).
    pub fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_unsigned_int(&mut self.sink_frame)?;
        xfer.xfer_unsigned_int(&mut self.midpoint_frame)?;
        xfer.xfer_unsigned_int(&mut self.destruction_frame)?;
        xfer.xfer_real(&mut self.accelerated_time_scale)?;
        xfer.xfer_unsigned_int(&mut self.flags)
    }

    #[inline]
    pub fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }
}

/// Runtime instance of `SlowDeathBehavior` (`PKT-295`).
pub struct SlowDeathBehavior {
    pub data: SlowDeathBehaviorModuleData,
    pub core: SlowDeathCore,
}

impl SlowDeathBehavior {
    pub fn new(data: SlowDeathBehaviorModuleData) -> Self {
        Self {
            data,
            core: SlowDeathCore::new(),
        }
    }
}

impl BehaviorModule for SlowDeathBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_object_created(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.on_object_created(sim, ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.core.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        self.core.xfer(xfer, sim)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.core.load_post_process(sim, ctx);
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

    fn as_slow_death(&self) -> Option<&dyn SlowDeathFacet> {
        Some(self)
    }

    fn as_slow_death_mut(&mut self) -> Option<&mut dyn SlowDeathFacet> {
        Some(self)
    }
}

impl DieFacet for SlowDeathBehavior {
    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.core
            .is_die_applicable(&self.data, sim, ctx.owner_slot, dmg)
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.core.on_die(&self.data, sim, ctx, dmg) != FALSE {
            self.core.begin_slow_death(&self.data, sim, ctx, dmg);
        }
    }
}

impl SlowDeathFacet for SlowDeathBehavior {
    fn is_die_applicable(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Bool {
        self.core
            .is_die_applicable(&self.data, sim, ctx.owner_slot, dmg)
    }

    fn get_probability_modifier(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        dmg: &DamageInfo,
    ) -> Int {
        self.core
            .get_probability_modifier(&self.data, sim, ctx.owner_slot, dmg)
    }

    fn begin_slow_death(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        self.core.begin_slow_death(&self.data, sim, ctx, dmg);
    }

    fn is_slow_death_in_progress(&self) -> Bool {
        self.core.is_slow_death_activated()
    }
}

impl UpdateFacet for SlowDeathBehavior {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.core.update_core(&self.data, sim, ctx)
    }

    fn disabled_types_to_process(&self) -> DisabledMaskType {
        disabled_mask_all()
    }
}

pub fn parse_slow_death_behavior_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = SlowDeathBehaviorModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        if let Some(res) = data.parse_field(field, r, parse_ctx) {
            res
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    if data.probability_modifier < 1 {
        return Err(IniError::InvalidData);
    }
    Ok(Box::new(data))
}

pub fn create_slow_death_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SlowDeathBehaviorModuleData>()
        .expect("SlowDeathBehavior data type mismatch")
        .clone();
    Box::new(SlowDeathBehavior::new(typed))
}
