//! `PKT-368`: `SpecialAbilityUpdateModuleData` and `SpecialAbilityUpdate`
//! (`SpecialAbilityUpdate.h`, `SpecialAbilityUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-368_SpecialAbilityUpdate.md`.
//! Golden test: `crates/zh_logic/tests/golden_pkt368_314_ability.rs`.
//!
//! A unit's targeted ability: approach, face, unpack, prepare, trigger, pack.
//! `AIUpdateInterface` is unported: the module orders it through
//! [`crate::modules::module::AiUpdateFacet`] and reads `isIdle`/`isMoving` back.
//! Sounds, EVA lines, radar events, lasers, particle systems and drawable
//! effects go to [`crate::sim::ClientSink`] as [`AbilityFeedback`].
//!
//! Kept quirks:
//! - `xfer` skips the `SpecialPowerUpdateModule` version byte and `crc` writes nothing.
//! - With `ApproachRequiresLOS` (the default) a position target is never in range:
//!   the search looks for the null target object.
//! - The Black Lotus cash hack steals 1000, whatever `EffectValue` says
//!   (`PRESERVE_HARDCODED_BLACK_LOTUS_CASH_HACK`).

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{
    SpecialPowerCaller, BehaviorModule, ModuleData, SpecialPowerUpdateFacet, UpdateFacet};
use crate::modules::module_factory::ModuleParseContext;
use crate::modules::update::physics_update::{physics_mut, APPLY_FRICTION2D_WHEN_AIRBORNE};
use crate::modules::CommandSourceType;
use crate::object::weapon::NO_MAX_SHOTS_LIMIT;
use crate::object::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::pathfinder::pathfind_constants::PATHFIND_CELL_SIZE_F;
use crate::rules::command_button::{CommandButton, CONTEXTMODE_COMMAND};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::special_power::SpecialPowerTemplate;
use crate::rules::special_power_types::SpecialPowerType;
use crate::sim::{RulesRegistry, SimApi, SimView};
use crate::spatial::partition_query::shrink;
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::math::{Coord3D, PI};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, UnsignedInt, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `SPECIAL_ABILITY_HUGE_DISTANCE`.
pub const SPECIAL_ABILITY_HUGE_DISTANCE: Real = 10_000_000.0;
/// The cash `PRESERVE_HARDCODED_BLACK_LOTUS_CASH_HACK` takes.
const BLACK_LOTUS_CASH_HACK_AMOUNT: UnsignedInt = 1000;
/// Targets smaller than this show the disable effect only every other time.
const SMALL_FOOTPRINT_AREA: Real = 300.0;

/// `PackingState`.
pub const STATE_NONE: Int = 0;
pub const STATE_PACKING: Int = 1;
pub const STATE_UNPACKING: Int = 2;
pub const STATE_PACKED: Int = 3;
pub const STATE_UNPACKED: Int = 4;

/// The abilities `initiateIntentToDoSpecialPower` stops before starting one.
const EXCLUSIVE_ABILITIES: [SpecialPowerType; 7] = [
    SpecialPowerType::BlackLotusDisableVehicleHack,
    SpecialPowerType::BlackLotusStealCashHack,
    SpecialPowerType::BlackLotusCaptureBuilding,
    SpecialPowerType::RemoteCharges,
    SpecialPowerType::TimedCharges,
    SpecialPowerType::InfantryCaptureBuilding,
    SpecialPowerType::BoobyTrap,
];

/// The client side of an ability.
#[derive(Debug, Clone, PartialEq)]
pub enum AbilityFeedback<'a> {
    /// An `AudioEventRTS` of the module data (pack, unpack, prep loop, trigger).
    Sound(&'a str),
    /// `removeAudioEvent` of the preparation loop.
    StopPreparationLoop,
    /// A per-unit voice line, by its `UnitSpecificSounds` key, when the ability succeeds.
    CompletionVoice(&'static str),
    AnimationCompletionTime(UnsignedInt),
    EvaBuildingBeingStolen,
    EvaBuildingStolen,
    EvaCashStolen,
    /// `Radar::tryInfiltrationEvent`.
    Infiltration(Option<ObjectId>),
    /// The house colour flash and tick of a capture.
    CaptureFlash(ObjectId),
    /// The two floating texts of the cash hack.
    CashStolen {
        target: ObjectId,
        amount: UnsignedInt,
    },
    /// `LaserUpdate::initLaser`.
    InitLaser {
        laser: ObjectId,
        target: ObjectId,
    },
    DisableEffect {
        template: &'a str,
        target: ObjectId,
        offset: Coord3D,
        lifetime: UnsignedInt,
    },
}

/// INI template data matching `SpecialAbilityUpdateModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct SpecialAbilityUpdateModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub special_object_name: String,
    pub special_object_attach_to_bone_name: String,
    pub special_power_name: String,
    pub special_power_id: UnsignedInt,
    pub start_ability_range: Real,
    pub ability_abort_range: Real,
    pub pack_unpack_variation_factor: Real,
    pub flee_range_after_completion: Real,
    pub effect_value: Int,
    pub award_xp_for_triggering: Int,
    pub skill_points_for_triggering: Int,
    pub preparation_frames: UnsignedInt,
    pub persistent_prep_frames: UnsignedInt,
    pub effect_duration: UnsignedInt,
    pub max_special_objects: UnsignedInt,
    pub pack_time: UnsignedInt,
    pub unpack_time: UnsignedInt,
    pub pre_trigger_unstealth_frames: UnsignedInt,
    pub skip_packing_with_no_target: Bool,
    pub special_objects_persistent: Bool,
    pub unique_special_object_targets: Bool,
    pub special_objects_persist_when_owner_dies: Bool,
    pub flip_object_after_packing: Bool,
    pub flip_object_after_unpacking: Bool,
    pub always_validate_special_objects: Bool,
    pub do_capture_fx: Bool,
    pub lose_stealth_on_trigger: Bool,
    pub approach_requires_los: Bool,
    pub need_to_face_target: Bool,
    pub persistence_requires_recharge: Bool,
    pub disable_fx_particle_system: String,
    pub pack_sound: String,
    pub unpack_sound: String,
    pub prep_sound_loop: String,
    pub trigger_sound: String,
}

impl SpecialAbilityUpdateModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            special_object_name: String::new(),
            special_object_attach_to_bone_name: String::new(),
            special_power_name: String::new(),
            special_power_id: 0,
            start_ability_range: SPECIAL_ABILITY_HUGE_DISTANCE,
            ability_abort_range: SPECIAL_ABILITY_HUGE_DISTANCE,
            pack_unpack_variation_factor: 0.0,
            flee_range_after_completion: 0.0,
            effect_value: 1,
            award_xp_for_triggering: 0,
            skill_points_for_triggering: -1,
            preparation_frames: 0,
            persistent_prep_frames: 0,
            effect_duration: 0,
            max_special_objects: 1,
            pack_time: 0,
            unpack_time: 0,
            pre_trigger_unstealth_frames: 0,
            skip_packing_with_no_target: FALSE,
            special_objects_persistent: FALSE,
            unique_special_object_targets: FALSE,
            special_objects_persist_when_owner_dies: FALSE,
            flip_object_after_packing: FALSE,
            flip_object_after_unpacking: FALSE,
            always_validate_special_objects: FALSE,
            do_capture_fx: FALSE,
            lose_stealth_on_trigger: FALSE,
            approach_requires_los: TRUE,
            need_to_face_target: TRUE,
            persistence_requires_recharge: FALSE,
            disable_fx_particle_system: String::new(),
            pack_sound: String::new(),
            unpack_sound: String::new(),
            prep_sound_loop: String::new(),
            trigger_sound: String::new(),
        }
    }

    /// `m_specialPowerTemplate`.
    pub fn template<'a>(&self, rules: &'a RulesRegistry) -> Option<&'a SpecialPowerTemplate> {
        let store = &rules.special_power_store;
        if self.special_power_id != 0 {
            store.find_special_power_template_by_id(self.special_power_id)
        } else if !self.special_power_name.is_empty() {
            store.find_special_power_template(&self.special_power_name)
        } else {
            None
        }
    }

    pub fn power_type(&self, rules: &RulesRegistry) -> SpecialPowerType {
        self.template(rules)
            .map_or(SpecialPowerType::Invalid, |t| t.special_power_type())
    }
}

impl ModuleData for SpecialAbilityUpdateModuleData {
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

pub fn parse_special_ability_update_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut d = SpecialAbilityUpdateModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, r| {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        if is("SpecialPowerTemplate") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            if tok.eq_ignore_ascii_case("None") {
                d.special_power_name.clear();
                d.special_power_id = 0;
            } else {
                d.special_power_name = tok.to_owned();
                d.special_power_id = parse_ctx
                    .special_power_store
                    .find_special_power_template(tok)
                    .map_or(0, |t| t.id());
            }
            Ok(())
        } else if is("DisableFXParticleSystem") {
            let tok = r.next_token(SEPS_DEFAULT)?;
            // An unknown name leaves the template pointer null.
            d.disable_fx_particle_system = if parse_ctx.particle_system_names.contains(tok) {
                tok.to_owned()
            } else {
                String::new()
            };
            Ok(())
        } else if is("StartAbilityRange") {
            r.parse_real(&mut d.start_ability_range)
        } else if is("AbilityAbortRange") {
            r.parse_real(&mut d.ability_abort_range)
        } else if is("PreparationTime") {
            r.parse_duration_unsigned_int(&mut d.preparation_frames)
        } else if is("PersistentPrepTime") {
            r.parse_duration_unsigned_int(&mut d.persistent_prep_frames)
        } else if is("PackTime") {
            r.parse_duration_unsigned_int(&mut d.pack_time)
        } else if is("UnpackTime") {
            r.parse_duration_unsigned_int(&mut d.unpack_time)
        } else if is("PreTriggerUnstealthTime") {
            r.parse_duration_unsigned_int(&mut d.pre_trigger_unstealth_frames)
        } else if is("SkipPackingWithNoTarget") {
            r.parse_bool(&mut d.skip_packing_with_no_target)
        } else if is("PackUnpackVariationFactor") {
            r.parse_real(&mut d.pack_unpack_variation_factor)
        } else if is("SpecialObject") {
            r.parse_ascii_string(&mut d.special_object_name)
        } else if is("SpecialObjectAttachToBone") {
            r.parse_ascii_string(&mut d.special_object_attach_to_bone_name)
        } else if is("MaxSpecialObjects") {
            r.parse_unsigned_int(&mut d.max_special_objects)
        } else if is("SpecialObjectsPersistent") {
            r.parse_bool(&mut d.special_objects_persistent)
        } else if is("EffectDuration") {
            r.parse_duration_unsigned_int(&mut d.effect_duration)
        } else if is("EffectValue") {
            r.parse_int(&mut d.effect_value)
        } else if is("UniqueSpecialObjectTargets") {
            r.parse_bool(&mut d.unique_special_object_targets)
        } else if is("SpecialObjectsPersistWhenOwnerDies") {
            r.parse_bool(&mut d.special_objects_persist_when_owner_dies)
        } else if is("AlwaysValidateSpecialObjects") {
            r.parse_bool(&mut d.always_validate_special_objects)
        } else if is("FlipOwnerAfterPacking") {
            r.parse_bool(&mut d.flip_object_after_packing)
        } else if is("FlipOwnerAfterUnpacking") {
            r.parse_bool(&mut d.flip_object_after_unpacking)
        } else if is("FleeRangeAfterCompletion") {
            r.parse_real(&mut d.flee_range_after_completion)
        } else if is("DoCaptureFX") {
            r.parse_bool(&mut d.do_capture_fx)
        } else if is("PackSound") {
            r.parse_ascii_string(&mut d.pack_sound)
        } else if is("UnpackSound") {
            r.parse_ascii_string(&mut d.unpack_sound)
        } else if is("PrepSoundLoop") {
            r.parse_ascii_string(&mut d.prep_sound_loop)
        } else if is("TriggerSound") {
            r.parse_ascii_string(&mut d.trigger_sound)
        } else if is("LoseStealthOnTrigger") {
            r.parse_bool(&mut d.lose_stealth_on_trigger)
        } else if is("AwardXPForTriggering") {
            r.parse_int(&mut d.award_xp_for_triggering)
        } else if is("SkillPointsForTriggering") {
            r.parse_int(&mut d.skill_points_for_triggering)
        } else if is("ApproachRequiresLOS") {
            r.parse_bool(&mut d.approach_requires_los)
        } else if is("NeedToFaceTarget") {
            r.parse_bool(&mut d.need_to_face_target)
        } else if is("PersistenceRequiresRecharge") {
            r.parse_bool(&mut d.persistence_requires_recharge)
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(d))
}

pub fn create_special_ability_update(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<SpecialAbilityUpdateModuleData>()
        .expect("SpecialAbilityUpdateModuleData mismatch");
    Box::new(SpecialAbilityUpdate::new(typed.clone()))
}

/// `MAKE_MODELCONDITION_MASKn`.
fn flags(list: &[ModelConditionFlagType]) -> ModelConditionFlags {
    let mut mask = ModelConditionFlags::new();
    for &f in list {
        mask.set(f.as_i32(), 1);
    }
    mask
}

fn is_nonzero(p: &Coord3D) -> bool {
    p.x != 0.0 || p.y != 0.0 || p.z != 0.0
}

fn find(sim: &dyn SimView, id: ObjectId) -> Option<ObjectSlot> {
    if id == ObjectId::INVALID {
        None
    } else {
        sim.find_object(id)
    }
}

fn test_status(sim: &dyn SimView, slot: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.status.test(status.as_i32()) != FALSE)
}

fn stealthed_and_undetected(sim: &dyn SimView, slot: ObjectSlot) -> bool {
    test_status(sim, slot, ObjectStatusType::Stealthed)
        && !test_status(sim, slot, ObjectStatusType::Detected)
}

fn is_effectively_dead(sim: &dyn SimView, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn team_of(sim: &dyn SimView, slot: ObjectSlot) -> Option<zh_common::types::TeamId> {
    sim.object(slot).map(|o| o.team_id)
}

fn is_locally_viewed(sim: &dyn SimView, slot: ObjectSlot) -> bool {
    team_of(sim, slot).is_some_and(|t| sim.player_index(t) == sim.local_player_index())
}

fn feedback(sim: &mut dyn SimApi, ctx: ModuleContext, event: &AbilityFeedback<'_>) {
    sim.client()
        .on_special_ability_feedback(ctx.owner_id, event);
}

fn sound(sim: &mut dyn SimApi, ctx: ModuleContext, name: &str) {
    if !name.is_empty() {
        feedback(sim, ctx, &AbilityFeedback::Sound(name));
    }
}

fn mark_detected(sim: &mut dyn SimApi, owner: ObjectSlot) {
    if sim.object(owner).is_some_and(|o| o.stealth_slot.is_some()) {
        sim.mark_stealth_detected(owner, 0);
    }
}

/// `getDistanceSquared(self, goal, FROM_BOUNDINGSPHERE_2D)`; `other_radius` is zero for a position.
fn bounding_distance_squared(from: &Coord3D, to: &Coord3D, total_radius: Real) -> Real {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    shrink(dx * dx + dy * dy, total_radius).0
}

/// Runtime instance of `SpecialAbilityUpdate`.
pub struct SpecialAbilityUpdate {
    pub data: SpecialAbilityUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    pub active: Bool,
    pub prep_frames: UnsignedInt,
    pub anim_frames: UnsignedInt,
    pub target_id: ObjectId,
    pub target_pos: Coord3D,
    pub location_count: Int,
    pub special_object_ids: Vec<ObjectId>,
    pub special_object_entries: UnsignedInt,
    pub no_target_command: Bool,
    pub packing_state: Int,
    pub facing_initiated: Bool,
    pub facing_complete: Bool,
    pub within_start_ability_range: Bool,
    pub do_disable_fx_particles: Bool,
    pub capture_flash_phase: Real,
}

impl SpecialAbilityUpdate {
    pub fn new(data: SpecialAbilityUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            active: FALSE,
            prep_frames: 0,
            anim_frames: 0,
            target_id: ObjectId::INVALID,
            target_pos: Coord3D::new(0.0, 0.0, 0.0),
            location_count: 0,
            special_object_ids: Vec::new(),
            special_object_entries: 0,
            no_target_command: FALSE,
            packing_state: STATE_NONE,
            facing_initiated: FALSE,
            facing_complete: FALSE,
            within_start_ability_range: FALSE,
            do_disable_fx_particles: TRUE,
            capture_flash_phase: 0.0,
        }
    }

    fn power_type(&self, sim: &dyn SimView) -> SpecialPowerType {
        self.data.power_type(sim.rules())
    }

    fn template_id(&self, sim: &dyn SimView) -> Option<UnsignedInt> {
        self.data.template(sim.rules()).map(|t| t.id())
    }

    /// `getMySPM` as an existence test.
    fn has_spm(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        self.template_id(sim)
            .is_some_and(|id| sim.find_special_power_module(owner, id).is_some())
    }

    fn start_spm_recharge(&self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if let Some(id) = self.template_id(sim) {
            sim.special_power_start_recharge(owner, id);
        }
    }

    fn is_persistent(&self) -> bool {
        self.data.persistent_prep_frames > 0
    }

    fn requires_recharge(&self) -> bool {
        self.data.persistence_requires_recharge != FALSE
    }

    fn calc_sleep_time(&self) -> UpdateSleepTime {
        if self.active != FALSE || self.data.always_validate_special_objects != FALSE {
            UpdateSleepTime::NONE
        } else {
            UpdateSleepTime::FOREVER
        }
    }

    fn need_to_pack(&self) -> bool {
        if self.packing_state == STATE_UNPACKED {
            if self.data.skip_packing_with_no_target != FALSE && self.no_target_command != FALSE {
                return false;
            }
            if self.data.pack_time != 0 {
                return true;
            }
        }
        false
    }

    fn need_to_unpack(&self) -> bool {
        if self.packing_state == STATE_PACKED {
            if self.data.skip_packing_with_no_target != FALSE && self.no_target_command != FALSE {
                return false;
            }
            if self.data.unpack_time != 0 {
                return true;
            }
        }
        false
    }

    fn clear_ability_model_flags(sim: &mut dyn SimApi, owner: ObjectSlot) {
        use ModelConditionFlagType as M;
        let clear = flags(&[M::Unpacking, M::Packing, M::FiringA, M::RaisingFlag]);
        sim.clear_and_set_model_condition_flags(owner, &clear, &ModelConditionFlags::new());
    }

    fn exit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, cleanup: bool) {
        let owner = ctx.owner_slot;
        Self::clear_ability_model_flags(sim, owner);
        sim.clear_status(owner, ObjectStatusType::IsUsingAbility);
        feedback(sim, ctx, &AbilityFeedback::StopPreparationLoop);
        self.end_preparation(sim, ctx);
        if self.data.special_objects_persistent == FALSE
            || (cleanup && self.data.special_objects_persist_when_owner_dies == FALSE)
        {
            self.kill_special_objects(sim, ctx);
        }
        self.active = FALSE;
        self.within_start_ability_range = FALSE;
        self.packing_state = STATE_NONE;
    }

    fn end_preparation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        use SpecialPowerType as P;
        sim.clear_status(ctx.owner_slot, ObjectStatusType::IsUsingAbility);
        feedback(sim, ctx, &AbilityFeedback::StopPreparationLoop);
        if matches!(
            self.power_type(sim),
            P::MissileDefenderLaserGuidedMissiles
                | P::HackerDisableBuilding
                | P::BlackLotusDisableVehicleHack
                | P::BlackLotusCaptureBuilding
                | P::BlackLotusStealCashHack
                | P::InfantryCaptureBuilding
        ) {
            self.kill_special_objects(sim, ctx);
        }
    }

    fn kill_special_objects(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        for id in std::mem::take(&mut self.special_object_ids) {
            if let Some(slot) = sim.find_object(id) {
                sim.destroy_object(slot);
            }
        }
        self.special_object_entries = 0;
        if self.power_type(sim) == SpecialPowerType::MissileDefenderLaserGuidedMissiles {
            sim.lock_weapon_slot(
                ctx.owner_slot,
                WeaponSlotType::Primary,
                WeaponLockType::LockedTemporarily,
            );
        }
    }

    fn validate_special_objects(&mut self, sim: &dyn SimView) {
        let before = self.special_object_ids.len();
        self.special_object_ids
            .retain(|&id| sim.find_object(id).is_some());
        let gone = (before - self.special_object_ids.len()) as UnsignedInt;
        self.special_object_entries = self.special_object_entries.wrapping_sub(gone);
    }

    fn create_special_object(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
    ) -> Option<ObjectSlot> {
        let owner = ctx.owner_slot;
        if self.special_object_entries == self.data.max_special_objects {
            if self.data.special_objects_persistent != FALSE {
                return None;
            }
            self.kill_special_objects(sim, ctx);
        }
        let template = sim
            .rules()
            .thing_factory
            .find_template_id(&self.data.special_object_name)?;
        let (team, pos, angle) = sim.object(owner).map(|o| {
            (
                o.team_id,
                *o.thing.get_position(),
                o.thing.get_orientation(),
            )
        })?;
        let special = sim.spawn_object(template, team, Coord3D::new(0.0, 0.0, 0.0), 0.0);
        let id = sim.object(special).map(|o| o.id)?;
        self.special_object_ids.push(id);
        self.special_object_entries = self.special_object_entries.wrapping_add(1);
        sim.set_object_position(special, &pos);
        sim.set_object_orientation(special, angle);
        if let Some(o) = sim.object_mut(special) {
            o.experience_tracker.experience_sink = ctx.owner_id;
        }
        if sim.has_physics(special) != FALSE {
            let pitch_rate = sim.physics_center_of_mass_offset(special);
            sim.physics_set_pitch_rate(special, pitch_rate);
            if let Some(p) = physics_mut(sim, special) {
                p.set_flag(APPLY_FRICTION2D_WHEN_AIRBORNE, FALSE);
            }
        }
        Some(special)
    }

    fn init_laser(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        special: ObjectSlot,
        target: ObjectSlot,
    ) -> bool {
        if sim.object_has_drawable(special) == FALSE
            || sim.object_has_template_module(special, "LaserUpdate") == FALSE
        {
            self.kill_special_objects(sim, ctx);
            return false;
        }
        let ids = sim.object(special).zip(sim.object(target));
        if let Some((laser, target)) = ids.map(|(l, t)| (l.id, t.id)) {
            feedback(sim, ctx, &AbilityFeedback::InitLaser { laser, target });
        }
        true
    }

    fn handle_packing_processing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
        let owner = ctx.owner_slot;
        if self.anim_frames == 0 {
            return false;
        }
        self.anim_frames -= 1;
        if self.anim_frames == 0 {
            use ModelConditionFlagType as M;
            let clear = flags(&[M::Unpacking, M::Packing]);
            sim.clear_and_set_model_condition_flags(owner, &clear, &ModelConditionFlags::new());
            if self.packing_state == STATE_UNPACKING {
                if self.data.flip_object_after_unpacking != FALSE {
                    Self::flip(sim, owner);
                }
                self.packing_state = STATE_UNPACKED;
            } else if self.packing_state == STATE_PACKING {
                if self.data.flip_object_after_packing != FALSE {
                    Self::flip(sim, owner);
                }
                self.packing_state = STATE_PACKED;
                self.finish_ability(sim, ctx);
                return true;
            }
            return false;
        }
        if self.data.lose_stealth_on_trigger != FALSE
            && self.anim_frames < self.data.pre_trigger_unstealth_frames
        {
            mark_detected(sim, owner);
        }
        true
    }

    fn flip(sim: &mut dyn SimApi, owner: ObjectSlot) {
        if let Some(angle) = sim.object(owner).map(|o| o.thing.get_orientation()) {
            sim.set_object_orientation(owner, angle + PI);
        }
    }

    /// The common part of `startPacking` and `startUnpacking`.
    fn start_anim(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        state: Int,
        time: UnsignedInt,
    ) {
        use ModelConditionFlagType as M;
        let owner = ctx.owner_slot;
        self.packing_state = state;
        let factor = self.data.pack_unpack_variation_factor;
        let variation = sim.rng_logic_real(1.0 - factor, 1.0 + factor);
        self.anim_frames = (time as Real * variation) as UnsignedInt;
        let packing = state == STATE_PACKING;
        let (clear, set) = if packing {
            (flags(&[M::Unpacking, M::RaisingFlag]), flags(&[M::Packing]))
        } else {
            (flags(&[M::Packing]), flags(&[M::Unpacking]))
        };
        sim.clear_and_set_model_condition_flags(owner, &clear, &set);
        let name = if packing {
            &self.data.pack_sound
        } else {
            &self.data.unpack_sound
        };
        sound(sim, ctx, name);
        if sim.object_has_drawable(owner) != FALSE {
            feedback(
                sim,
                ctx,
                &AbilityFeedback::AnimationCompletionTime(self.anim_frames),
            );
        }
        sim.ai_busy(owner, CommandSourceType::FromAi);
    }

    fn start_packing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, success: bool) {
        use SpecialPowerType as P;
        self.start_anim(sim, ctx, STATE_PACKING, self.data.pack_time);
        if success {
            let key = match self.power_type(sim) {
                P::BlackLotusCaptureBuilding => "VoiceCaptureBuildingComplete",
                P::BlackLotusDisableVehicleHack => "VoiceDisableVehicleComplete",
                P::BlackLotusStealCashHack => "VoiceStealCashComplete",
                _ => "VoiceTaskComplete",
            };
            feedback(sim, ctx, &AbilityFeedback::CompletionVoice(key));
        }
    }

    fn start_unpacking(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.start_anim(sim, ctx, STATE_UNPACKING, self.data.unpack_time);
    }

    /// The goal object (if any) and the squared distance to it, or `None` without a goal.
    fn goal_distance(
        &self,
        sim: &dyn SimView,
        owner: ObjectSlot,
    ) -> Option<(Option<ObjectSlot>, Real)> {
        let me = sim.object(owner)?;
        let from = *me.thing.get_position();
        let radius = me.geometry.get_bounding_circle_radius();
        if self.target_id != ObjectId::INVALID {
            let target = sim.find_object(self.target_id);
            let distance = target.and_then(|t| sim.object(t)).map_or(0.0, |o| {
                bounding_distance_squared(
                    &from,
                    o.thing.get_position(),
                    radius + o.geometry.get_bounding_circle_radius(),
                )
            });
            Some((target, distance))
        } else if is_nonzero(&self.target_pos) {
            Some((
                None,
                bounding_distance_squared(&from, &self.target_pos, radius),
            ))
        } else {
            None
        }
    }

    /// `startAbilityRange` less a quarter of a pathfind cell.
    fn undersized_range(&self) -> Real {
        let range = self.data.start_ability_range - PATHFIND_CELL_SIZE_F * 0.25;
        if 0.0 > range {
            0.0
        } else {
            range
        }
    }

    fn collides_with(sim: &dyn SimView, owner: ObjectSlot, target: Option<ObjectSlot>) -> bool {
        let Some(me) = sim.object(owner) else {
            return false;
        };
        let hits = sim.find_potential_collisions(me.thing.get_position(), &me.geometry, 0.0, FALSE);
        target.is_some_and(|t| hits.contains(&t))
    }

    fn is_within_start_ability_range(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        let range = self.undersized_range();
        if self.within_start_ability_range != FALSE {
            return true;
        }
        let Some((target, distance)) = self.goal_distance(sim, owner) else {
            return true;
        };
        if distance <= self.data.start_ability_range * self.data.start_ability_range {
            if range == 0.0 && self.target_id != ObjectId::INVALID {
                return Self::collides_with(sim, owner, target);
            }
            if self.data.approach_requires_los == FALSE {
                return true;
            }
            return target.is_some_and(|t| {
                sim.is_in_range_bounding_2d(owner, t, range) != FALSE
                    && sim.is_clear_line_of_sight(owner, t) != FALSE
            });
        }
        false
    }

    fn is_within_ability_abort_range(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        let range = self.undersized_range();
        let Some((target, distance)) = self.goal_distance(sim, owner) else {
            return true;
        };
        if distance <= self.data.ability_abort_range * self.data.ability_abort_range {
            if range == 0.0 && self.target_id != ObjectId::INVALID {
                return Self::collides_with(sim, owner, target);
            }
            return true;
        }
        false
    }

    fn approach_target(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if self.target_id != ObjectId::INVALID {
            if let Some(target) = sim.find_object(self.target_id) {
                sim.ai_ignore_obstacle(owner, self.target_id);
                sim.ai_move_to_object(owner, target, CommandSourceType::FromAi);
            }
        } else if is_nonzero(&self.target_pos) {
            sim.ai_move_to_position(owner, &self.target_pos, CommandSourceType::FromAi);
        }
    }

    fn start_preparation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        use ModelConditionFlagType as M;
        use SpecialPowerType as P;
        let owner = ctx.owner_slot;
        self.prep_frames = self.data.preparation_frames;
        let power = self.power_type(sim);
        match power {
            P::MissileDefenderLaserGuidedMissiles => {
                if let Some(target) = find(sim, self.target_id) {
                    if let Some(special) = self.create_special_object(sim, ctx) {
                        if !self.init_laser(sim, ctx, special, target) {
                            return;
                        }
                    }
                }
            }
            P::InfantryCaptureBuilding => {
                let target = find(sim, self.target_id);
                if let Some(target) = target {
                    if team_of(sim, target) == team_of(sim, owner) {
                        return;
                    }
                    if sim.check_and_detonate_booby_trap(target, owner) != FALSE
                        && is_effectively_dead(sim, target)
                    {
                        return;
                    }
                }
                sim.clear_and_set_model_condition_flags(
                    owner,
                    &flags(&[M::Unpacking]),
                    &flags(&[M::RaisingFlag]),
                );
                if sim.object_has_drawable(owner) != FALSE {
                    let frames = self.data.preparation_frames;
                    feedback(sim, ctx, &AbilityFeedback::AnimationCompletionTime(frames));
                }
                if target.is_some_and(|t| is_locally_viewed(sim, t)) {
                    feedback(sim, ctx, &AbilityFeedback::EvaBuildingBeingStolen);
                }
                let id = target.and_then(|t| sim.object(t)).map(|o| o.id);
                feedback(sim, ctx, &AbilityFeedback::Infiltration(id));
            }
            P::HackerDisableBuilding
            | P::BlackLotusCaptureBuilding
            | P::BlackLotusDisableVehicleHack
            | P::BlackLotusStealCashHack => {
                if let Some(target) = find(sim, self.target_id) {
                    if sim.player_relationship(owner, target) == Relationship::Allies {
                        return;
                    }
                    if let Some(special) = self.create_special_object(sim, ctx) {
                        if !self.init_laser(sim, ctx, special, target) {
                            return;
                        }
                        sim.clear_and_set_model_condition_flags(
                            owner,
                            &flags(&[M::Unpacking]),
                            &flags(&[M::FiringA]),
                        );
                    }
                    if power == P::BlackLotusCaptureBuilding && is_locally_viewed(sim, target) {
                        feedback(sim, ctx, &AbilityFeedback::EvaBuildingBeingStolen);
                    }
                    let id = sim.object(target).map(|o| o.id);
                    feedback(sim, ctx, &AbilityFeedback::Infiltration(id));
                }
            }
            _ => {}
        }
        if let Some(id) = self.template_id(sim) {
            sim.special_power_mark_triggered(owner, id);
        }
        sim.ai_idle(owner, CommandSourceType::FromAi);
        sim.set_status(owner, ObjectStatusType::IsUsingAbility);
        sound(sim, ctx, &self.data.prep_sound_loop);
    }

    fn continue_preparation(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
        use SpecialPowerType as P;
        let owner = ctx.owner_slot;
        if self.data.ability_abort_range < SPECIAL_ABILITY_HUGE_DISTANCE
            && !self.is_within_ability_abort_range(sim, owner)
        {
            return false;
        }
        let power = self.power_type(sim);
        match power {
            P::MissileDefenderLaserGuidedMissiles | P::BlackLotusDisableVehicleHack => {
                let Some(target) = find(sim, self.target_id) else {
                    return false;
                };
                if sim.player_relationship(owner, target) == Relationship::Allies {
                    return false;
                }
                // `init_laser` empties the list only when it fails, which ends the loop.
                for id in self.special_object_ids.clone() {
                    if let Some(special) = sim.find_object(id) {
                        if !self.init_laser(sim, ctx, special, target) {
                            return false;
                        }
                    }
                }
            }
            P::InfantryCaptureBuilding | P::BlackLotusCaptureBuilding => {
                let Some(target) = find(sim, self.target_id) else {
                    return false;
                };
                if sim.player_relationship(owner, target) == Relationship::Allies {
                    return false;
                }
                if self.data.do_capture_fx != FALSE && sim.object_has_drawable(target) != FALSE {
                    let last_phase = (self.capture_flash_phase as Int) & 1;
                    let denominator = self.data.preparation_frames.max(1) as Real;
                    let increment = 1.0 - (self.prep_frames as Real / denominator);
                    self.capture_flash_phase += increment / 3.0;
                    let this_phase = (self.capture_flash_phase as Int) & 1;
                    if last_phase != 0 && this_phase == 0 {
                        feedback(sim, ctx, &AbilityFeedback::CaptureFlash(self.target_id));
                    }
                }
                if power == P::InfantryCaptureBuilding && self.has_spm(sim, owner) {
                    self.start_spm_recharge(sim, owner);
                }
            }
            _ => {}
        }
        true
    }

    /// Plants a charge on `target`. `false` if the charge has no `StickyBombUpdate`.
    fn plant_charge(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
    ) -> bool {
        if let Some(charge) = self.create_special_object(sim, ctx) {
            if sim.object_has_template_module(charge, "StickyBombUpdate") == FALSE {
                self.kill_special_objects(sim, ctx);
                return false;
            }
            sim.sticky_bomb_init(charge, target, ctx.owner_slot);
        }
        true
    }

    /// `checkAndDetonateBoobyTrap`; `true` if the blast killed the target or the owner.
    fn booby_trap_killed(sim: &mut dyn SimApi, target: ObjectSlot, owner: ObjectSlot) -> bool {
        sim.check_and_detonate_booby_trap(target, owner) != FALSE
            && (is_effectively_dead(sim, target) || is_effectively_dead(sim, owner))
    }

    fn trigger_ability_effect(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        let xp = self.data.award_xp_for_triggering;
        if xp != 0 {
            sim.add_experience_points(owner, xp);
        }
        let skill_points = if self.data.skill_points_for_triggering != -1 {
            self.data.skill_points_for_triggering
        } else {
            xp
        };
        if skill_points > 0 {
            if let Some(team) = team_of(sim, owner) {
                sim.add_skill_points(team, skill_points);
            }
        }
        sound(sim, ctx, &self.data.trigger_sound);
        if let Some(ok_to_lose_stealth) = self.apply_effect(sim, ctx) {
            if self.data.lose_stealth_on_trigger != FALSE && ok_to_lose_stealth {
                mark_detected(sim, owner);
            }
        }
    }

    /// The type switch of `triggerAbilityEffect`. `None` where the reference returns
    /// early, otherwise `okToLoseStealth`.
    fn apply_effect(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<bool> {
        use SpecialPowerType as P;
        let owner = ctx.owner_slot;
        let mut ok_to_lose_stealth = true;
        let power = self.power_type(sim);
        match power {
            P::MissileDefenderLaserGuidedMissiles => {
                if let Some(target) = find(sim, self.target_id) {
                    let has_secondary = sim.object(owner).is_some_and(|o| {
                        o.weapon_set
                            .get_weapon_in_weapon_slot(WeaponSlotType::Secondary)
                            .is_some()
                    });
                    if has_secondary {
                        sim.lock_weapon_slot(
                            owner,
                            WeaponSlotType::Secondary,
                            WeaponLockType::LockedTemporarily,
                        );
                        sim.ai_attack_object(
                            owner,
                            target,
                            NO_MAX_SHOTS_LIMIT,
                            CommandSourceType::FromAi,
                        );
                    }
                }
            }
            P::HelixNapalmBomb => {
                self.create_special_object(sim, ctx);
            }
            P::TankHunterTntAttack | P::TimedCharges | P::BoobyTrap => {
                let target = find(sim, self.target_id)?;
                if Self::booby_trap_killed(sim, target, owner) {
                    return None;
                }
                if power == P::BoobyTrap && test_status(sim, target, ObjectStatusType::BoobyTrapped)
                {
                    return None;
                }
                if !self.plant_charge(sim, ctx, target) {
                    return None;
                }
            }
            P::HackerDisableBuilding | P::BlackLotusDisableVehicleHack => {
                let target = find(sim, self.target_id)?;
                if sim.player_relationship(owner, target) == Relationship::Allies {
                    return None;
                }
                let until = sim.frame().wrapping_add(self.data.effect_duration);
                sim.set_disabled_until(target, DisabledType::Hacked, until);
                self.show_disable_effect(sim, ctx, target);
            }
            P::InfantryCaptureBuilding | P::BlackLotusCaptureBuilding => {
                self.capture(sim, ctx, power)?;
            }
            P::BlackLotusStealCashHack => {
                let target = find(sim, self.target_id)?;
                self.steal_cash(sim, ctx, target);
            }
            P::RemoteCharges => {
                let target = find(sim, self.target_id);
                if let Some(target) = target {
                    if Self::booby_trap_killed(sim, target, owner) {
                        return None;
                    }
                }
                if self.target_id == ObjectId::INVALID && !is_nonzero(&self.target_pos) {
                    for id in self.special_object_ids.clone() {
                        if let Some(special) = sim.find_object(id) {
                            if sim.sticky_bomb_detonate(special) != FALSE {
                                ok_to_lose_stealth = false;
                            }
                        }
                    }
                } else if !self.plant_charge(sim, ctx, target?) {
                    return None;
                }
            }
            P::DisguiseAsVehicle => {
                if let Some(target) = find(sim, self.target_id) {
                    sim.stealth_disguise_as_object(owner, Some(target));
                }
            }
            _ => {}
        }
        Some(ok_to_lose_stealth)
    }

    fn show_disable_effect(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        target: ObjectSlot,
    ) {
        let Some((geometry, target_id)) = sim.object(target).map(|o| (o.geometry.clone(), o.id))
        else {
            return;
        };
        let mut interleave: UnsignedInt = 1;
        if geometry.get_footprint_area() < SMALL_FOOTPRINT_AREA
            && sim.object_is_kind_of(target, KindOfType::Structure) != FALSE
        {
            self.do_disable_fx_particles = Bool::from(self.do_disable_fx_particles == FALSE);
            interleave = 2;
        }
        if self.do_disable_fx_particles != FALSE && !self.data.disable_fx_particle_system.is_empty()
        {
            // The offset comes from the logic PRNG although the effect is client side.
            let mut offset = Coord3D::new(0.0, 0.0, 0.0);
            sim.random_offset_within_footprint(&geometry, &mut offset);
            let lifetime = self.data.effect_duration.wrapping_mul(interleave);
            let event = AbilityFeedback::DisableEffect {
                template: &self.data.disable_fx_particle_system,
                target: target_id,
                offset,
                lifetime,
            };
            feedback(sim, ctx, &event);
        }
    }

    fn capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        power: SpecialPowerType,
    ) -> Option<()> {
        let owner = ctx.owner_slot;
        let target = find(sim, self.target_id)?;
        if Self::booby_trap_killed(sim, target, owner) {
            return None;
        }
        let owner_team = team_of(sim, owner)?;
        if team_of(sim, target) == Some(owner_team) {
            return None;
        }
        if sim.object_has_contain(target) != FALSE && sim.contain_is_garrisonable(target) != FALSE {
            sim.contain_remove_all(target, TRUE);
            return Some(());
        }
        if is_locally_viewed(sim, target) {
            feedback(sim, ctx, &AbilityFeedback::EvaBuildingStolen);
        }
        let new_team = sim.player_default_team(owner_team);
        sim.defect_object(target, new_team, 1, TRUE);
        if power == SpecialPowerType::BlackLotusCaptureBuilding && self.has_spm(sim, owner) {
            self.start_spm_recharge(sim, owner);
        }
        if let Some(team) = team_of(sim, owner) {
            sim.record_building_capture(team);
        }
        Some(())
    }

    fn steal_cash(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, target: ObjectSlot) {
        let teams = team_of(sim, target).zip(team_of(sim, ctx.owner_slot));
        let Some((target_team, owner_team)) = teams else {
            return;
        };
        let cash = sim
            .player_money(target_team)
            .min(BLACK_LOTUS_CASH_HACK_AMOUNT);
        if cash > 0 {
            sim.withdraw_player_money(target_team, cash);
            sim.add_player_money(owner_team, cash as Int);
            sim.add_money_earned(owner_team, cash as Int);
            if is_locally_viewed(sim, target) {
                feedback(sim, ctx, &AbilityFeedback::EvaCashStolen);
            }
            if let Some(id) = sim.object(target).map(|o| o.id) {
                let event = AbilityFeedback::CashStolen {
                    target: id,
                    amount: cash,
                };
                feedback(sim, ctx, &event);
            }
        }
    }

    fn finish_ability(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        let owner = ctx.owner_slot;
        self.within_start_ability_range = FALSE;
        self.packing_state = STATE_NONE;
        let valid_target = is_nonzero(&self.target_pos) || self.target_id != ObjectId::INVALID;
        if self.data.flee_range_after_completion != 0.0 && valid_target {
            self.flee(sim, owner);
        } else {
            sim.ai_idle(owner, CommandSourceType::FromAi);
        }
        self.exit(sim, ctx, false);
    }

    fn flee(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let range = self.data.flee_range_after_completion;
        let Some((mut pos, mut dir, team)) = sim.object(owner).map(|o| {
            (
                *o.thing.get_position(),
                o.thing.get_unit_direction_vector_2d(),
                o.team_id,
            )
        }) else {
            return;
        };
        if sim.has_ai(owner) == FALSE {
            return;
        }
        dir.x *= range;
        dir.y *= range;
        dir.z *= range;
        if self.data.flip_object_after_unpacking != FALSE
            || self.data.flip_object_after_packing != FALSE
        {
            pos.x += dir.x;
            pos.y += dir.y;
            pos.z += dir.z;
        } else {
            pos.x -= dir.x;
            pos.y -= dir.y;
            pos.z -= dir.z;
        }
        // Keep clear of one's own mines.
        let mine = sim.find_closest_team_object_of_kind(&pos, range, KindOfType::Mine, team);
        if let Some(mine_pos) = mine
            .and_then(|m| sim.object(m))
            .map(|o| *o.thing.get_position())
        {
            dir = Coord3D::new(pos.x - mine_pos.x, pos.y - mine_pos.y, 0.0);
            dir.normalize();
            dir.x *= range;
            dir.y *= range;
            dir.z *= range;
            pos = Coord3D::new(mine_pos.x + dir.x, mine_pos.y + dir.y, mine_pos.z + dir.z);
        }
        if sim.has_physics(owner) != FALSE {
            sim.physics_apply_motive_force(owner, &Coord3D::new(0.0, 0.0, 0.0));
        }
        sim.ai_move_to_position(owner, &pos, CommandSourceType::FromAi);
        if sim.find_object(self.target_id).is_some() {
            sim.ai_ignore_obstacle(owner, self.target_id);
        }
    }

    fn is_facing(&mut self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        if sim.has_ai(owner) == FALSE {
            return true;
        }
        if self.facing_complete == FALSE && self.facing_initiated != FALSE {
            if sim.ai_is_idle(owner) != FALSE {
                self.facing_complete = TRUE;
                return false;
            }
            return true;
        }
        false
    }

    fn need_to_face(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        sim.has_ai(owner) != FALSE
            && self.data.need_to_face_target != FALSE
            && (self.facing_initiated == FALSE || self.facing_complete == FALSE)
    }

    fn start_facing(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        let target = find(sim, self.target_id);
        if sim.has_ai(owner) == FALSE {
            return;
        }
        sim.ai_idle(owner, CommandSourceType::FromAi);
        if sim.has_physics(owner) != FALSE {
            sim.reset_dynamic_physics(owner);
        }
        self.facing_initiated = TRUE;
        if let Some(target) = target {
            sim.ai_face_object(owner, target, CommandSourceType::FromAi);
        } else if is_nonzero(&self.target_pos) {
            sim.ai_face_position(owner, &self.target_pos, CommandSourceType::FromAi);
        }
    }

    /// Whether the target no longer permits the ability.
    fn should_abort(&self, sim: &dyn SimView, owner: ObjectSlot) -> bool {
        use SpecialPowerType as P;
        let Some(target) = find(sim, self.target_id) else {
            return false;
        };
        if is_effectively_dead(sim, target) {
            return true;
        }
        let preparing = self.prep_frames != 0;
        let same_team = team_of(sim, target) == team_of(sim, owner);
        match self.power_type(sim) {
            P::InfantryCaptureBuilding
            | P::BlackLotusCaptureBuilding
            | P::HackerDisableBuilding => {
                same_team || (stealthed_and_undetected(sim, target) && preparing)
            }
            P::BlackLotusStealCashHack | P::BoobyTrap => {
                stealthed_and_undetected(sim, target) && preparing
            }
            P::RemoteCharges | P::TimedCharges => {
                !self.need_to_unpack() && stealthed_and_undetected(sim, target) && preparing
            }
            P::MissileDefenderLaserGuidedMissiles => {
                sim.object_is_kind_of(target, KindOfType::Structure) != FALSE
                    || stealthed_and_undetected(sim, target)
            }
            P::BlackLotusDisableVehicleHack => stealthed_and_undetected(sim, target),
            _ => false,
        }
    }

    fn run(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        use SpecialPowerType as P;
        let owner = ctx.owner_slot;
        self.validate_special_objects(sim);
        if is_effectively_dead(sim, owner) {
            self.exit(sim, ctx, true);
            return;
        }
        if self.active == FALSE {
            return;
        }
        if sim.has_ai(owner) == FALSE
            || sim.ai_last_command_source(owner) != CommandSourceType::FromAi
        {
            self.exit(sim, ctx, false);
            return;
        }
        // A capture is given up as soon as the unit moves off while doing it.
        if sim.ai_is_moving(owner) != FALSE
            && self.is_power_currently_in_use(sim, None) != FALSE
            && self.facing_initiated == FALSE
            && matches!(
                self.power_type(sim),
                P::InfantryCaptureBuilding | P::BlackLotusCaptureBuilding
            )
        {
            self.exit(sim, ctx, false);
            return;
        }
        if self.handle_packing_processing(sim, ctx) {
            return;
        }
        let abort = self.should_abort(sim, owner);
        let spm = self
            .template_id(sim)
            .filter(|&id| sim.find_special_power_module(owner, id).is_some());
        let spm = match spm {
            Some(id) if !abort => id,
            _ => {
                sim.ai_idle(owner, CommandSourceType::FromAi);
                self.exit(sim, ctx, false);
                return;
            }
        };
        if self.prep_frames != 0 {
            let mut spm_ready = true;
            if self.is_persistent() && self.requires_recharge() {
                spm_ready = sim
                    .special_power_module_ready(owner, spm)
                    .is_some_and(|(ready, frame)| ready != FALSE && frame < sim.frame());
            }
            if spm_ready {
                self.prep_frames -= 1;
            }
            if self.prep_frames == 0 {
                self.trigger_ability_effect(sim, ctx);
                if self.is_persistent() {
                    self.prep_frames = self.data.persistent_prep_frames;
                    if self.requires_recharge() {
                        sim.special_power_start_recharge(owner, spm);
                    }
                } else {
                    self.end_preparation(sim, ctx);
                    self.pack_or_finish(sim, ctx, true);
                }
            } else if !self.continue_preparation(sim, ctx) {
                self.end_preparation(sim, ctx);
                self.pack_or_finish(sim, ctx, false);
            }
        } else if self.is_within_start_ability_range(sim, owner) {
            self.within_start_ability_range = TRUE;
            if !self.is_facing(sim, owner) && self.need_to_face(sim, owner) {
                self.start_facing(sim, owner);
                return;
            }
            if self.need_to_unpack() {
                self.start_unpacking(sim, ctx);
                return;
            }
            if self.packing_state == STATE_UNPACKED {
                self.start_preparation(sim, ctx);
                if self.prep_frames == 0 {
                    self.trigger_ability_effect(sim, ctx);
                    if self.is_persistent() && self.requires_recharge() {
                        self.prep_frames = self.data.persistent_prep_frames;
                        sim.special_power_start_recharge(owner, spm);
                        return;
                    }
                    self.end_preparation(sim, ctx);
                    self.pack_or_finish(sim, ctx, true);
                }
            }
        } else if sim.ai_is_idle(owner) != FALSE {
            self.approach_target(sim, owner);
        }
    }

    fn pack_or_finish(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, success: bool) {
        if self.need_to_pack() {
            self.start_packing(sim, ctx, success);
        } else {
            self.finish_ability(sim, ctx);
        }
    }
}

impl UpdateFacet for SpecialAbilityUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        self.run(sim, ctx);
        self.calc_sleep_time()
    }
}

impl SpecialPowerUpdateFacet for SpecialAbilityUpdate {
    fn initiate_intent_to_do_special_power(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _caller: SpecialPowerCaller<'_>,
        template_id: UnsignedInt,
        target: Option<ObjectSlot>,
        target_pos: Option<&Coord3D>,
        _command_options: UnsignedInt,
    ) -> Bool {
        let owner = ctx.owner_slot;
        if self.template_id(sim) != Some(template_id) {
            return FALSE;
        }
        self.target_id = ObjectId::INVALID;
        self.target_pos = Coord3D::new(0.0, 0.0, 0.0);
        self.location_count = 0;
        self.prep_frames = 0;
        self.anim_frames = 0;
        self.packing_state = STATE_PACKED;
        self.facing_initiated = FALSE;
        self.facing_complete = FALSE;
        self.within_start_ability_range = FALSE;
        Self::clear_ability_model_flags(sim, owner);
        if let Some(target) = target {
            self.target_id = sim.object(target).map_or(ObjectId::INVALID, |o| o.id);
        } else if let Some(pos) = target_pos {
            self.target_pos = *pos;
        }
        if sim.has_ai(owner) == FALSE {
            return FALSE;
        }
        sim.ai_idle(owner, CommandSourceType::FromAi);
        self.no_target_command = Bool::from(target.is_none() && target_pos.is_none());
        if self.data.unpack_time == 0
            || (self.no_target_command != FALSE && self.data.skip_packing_with_no_target != FALSE)
        {
            self.packing_state = STATE_UNPACKED;
        }
        self.active = TRUE;
        for power in EXCLUSIVE_ABILITIES {
            if let Some(other) = sim.find_special_ability_update(owner, power) {
                if other != ctx.mod_slot {
                    sim.exit_special_ability(other, FALSE);
                }
            }
        }
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::NONE);
        TRUE
    }

    fn is_active(&self) -> Bool {
        self.active
    }

    fn is_power_currently_in_use(
        &self,
        sim: &dyn SimView,
        command: Option<&CommandButton>,
    ) -> Bool {
        let template = command.and_then(|c| c.special_power_id).and_then(|id| {
            sim.rules()
                .special_power_store
                .find_special_power_template_by_id(id)
        });
        if let (Some(command), Some(template)) = (command, template) {
            if template.special_power_type() == SpecialPowerType::RemoteCharges
                && command.options & CONTEXTMODE_COMMAND == 0
            {
                return Bool::from(self.special_object_entries == 0);
            }
        }
        if self.packing_state != STATE_NONE {
            if (self.packing_state == STATE_PACKING || self.packing_state == STATE_PACKED)
                && template.is_some_and(|t| t.reload_time() == 0)
            {
                return FALSE;
            }
            if self.within_start_ability_range != FALSE {
                return TRUE;
            }
        }
        FALSE
    }

    fn on_exit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, cleanup: Bool) {
        self.exit(sim, ctx, cleanup != FALSE);
    }

    fn special_object_count(&self) -> UnsignedInt {
        self.special_object_entries
    }

    fn find_special_object_with_producer_id(
        &self,
        sim: &dyn SimView,
        producer: ObjectId,
    ) -> Option<ObjectSlot> {
        self.special_object_ids
            .iter()
            .filter_map(|&id| sim.find_object(id))
            .find(|&slot| sim.object(slot).is_some_and(|o| o.producer_id == producer))
    }
}

impl BehaviorModule for SpecialAbilityUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
        sim.set_wake_frame(ctx.mod_slot, UpdateSleepTime::FOREVER);
    }

    /// `~SpecialAbilityUpdate`.
    fn on_destruct(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.exit(sim, ctx, true);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        xfer.xfer_bool(&mut self.active)?;
        xfer.xfer_unsigned_int(&mut self.prep_frames)?;
        xfer.xfer_unsigned_int(&mut self.anim_frames)?;
        xfer.xfer_object_id(&mut self.target_id)?;
        xfer.xfer_coord3d(&mut self.target_pos)?;
        xfer.xfer_int(&mut self.location_count)?;
        xfer.xfer_stl_object_id_vec(&mut self.special_object_ids)?;
        xfer.xfer_unsigned_int(&mut self.special_object_entries)?;
        xfer.xfer_bool(&mut self.no_target_command)?;
        xfer.xfer_int(&mut self.packing_state)?;
        xfer.xfer_bool(&mut self.facing_initiated)?;
        xfer.xfer_bool(&mut self.facing_complete)?;
        xfer.xfer_bool(&mut self.within_start_ability_range)?;
        xfer.xfer_bool(&mut self.do_disable_fx_particles)?;
        xfer.xfer_real(&mut self.capture_flash_phase)
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

    fn as_special_power_update(&self) -> Option<&dyn SpecialPowerUpdateFacet> {
        Some(self)
    }

    fn as_special_power_update_mut(&mut self) -> Option<&mut dyn SpecialPowerUpdateFacet> {
        Some(self)
    }
}
