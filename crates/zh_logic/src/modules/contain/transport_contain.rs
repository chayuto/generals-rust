//! `PKT-299`: `TransportContain` (`TransportContain.h`, `TransportContain.cpp`).
//!
//! Packet: `packets/phase5/PKT-299_TransportContain.md`.
//! Golden test: `crates/zh_logic/tests/golden_wp53_b.rs`.
//!
//! Not modelled: `ExitBone` repositioning (headless art has no bones) and the combat-bike
//! weapon reload transfer for salvager heroes.
//!
//! Subclasses ([`crate::modules::contain::internet_hack_contain`], later Overlord / Helix
//! containers) implement [`TransportBehavior`] and reuse the `transport_*` functions.

use crate::arena::{FacetMask, ModuleContext, ObjectSlot};
use crate::modules::behavior::behavior_module::xfer_module_version;
use crate::modules::contain::open_contain::{
    add_to_contain, impl_contain_module, kill_object, order_all_passengers_to_exit,
    remove_from_contain, scatter_to_nearby_position, update_open_contain, ContainBehavior,
    OpenContainCore, OpenContainModuleData,
};
use crate::modules::module::{
    AttitudeType, BehaviorModule, CommandSourceType, ExitDoorType, ModuleData,
};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{
    DamageType, DeathType, LocomotorSurfaceType, WeaponSetType,
};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::{KindOfMaskType, KindOfType};
use crate::rules::model_state::ModelConditionFlagType;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use zh_common::ini::{IniError, IniReader, SECONDS_PER_LOGICFRAME_REAL, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, Relationship, TeamId, UnsignedInt, UpdateSleepTime,
    FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `InitialPayload = <template> [count]`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitialPayload {
    pub name: String,
    pub count: Int,
}

impl InitialPayload {
    pub fn parse(&mut self, reader: &mut IniReader<'_>) -> Result<(), IniError> {
        let name = reader.next_token(SEPS_DEFAULT)?.to_owned();
        let count = match reader.next_token_or_null(SEPS_DEFAULT) {
            Some(tok) => IniReader::scan_int(tok)?,
            None => 1,
        };
        self.name = name;
        self.count = count;
        Ok(())
    }
}

/// Default `AllowInsideKindOf` for transports: infantry only (any INI value replaces it).
pub fn infantry_kind_of_mask() -> KindOfMaskType {
    let mut mask = KindOfMaskType::new();
    mask.set(KindOfType::Infantry as Int, TRUE as Int);
    mask
}

/// INI template data matching `TransportContainModuleData`.
#[derive(Debug, Clone, PartialEq)]
pub struct TransportContainModuleData {
    pub open: OpenContainModuleData,
    pub slot_capacity: Int,
    pub exit_pitch_rate: Real,
    pub exit_bone: String,
    pub initial_payload: InitialPayload,
    pub health_regen: Real,
    pub exit_delay: UnsignedInt,
    pub scatter_nearby_on_exit: Bool,
    pub orient_like_container_on_exit: Bool,
    pub keep_container_velocity_on_exit: Bool,
    pub go_aggressive_on_exit: Bool,
    pub armed_riders_upgrade_weapon_set: Bool,
    pub reset_mood_check_time_on_exit: Bool,
    pub destroy_riders_who_are_not_free_to_exit: Bool,
    pub is_delay_exit_in_air: Bool,
}

impl TransportContainModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        let mut open = OpenContainModuleData::new(tag_key, module_name_key);
        open.allow_inside_kind_of = infantry_kind_of_mask();
        Self {
            open,
            slot_capacity: 0,
            exit_pitch_rate: 0.0,
            exit_bone: String::new(),
            initial_payload: InitialPayload::default(),
            health_regen: 0.0,
            exit_delay: 0,
            scatter_nearby_on_exit: TRUE,
            orient_like_container_on_exit: FALSE,
            keep_container_velocity_on_exit: FALSE,
            go_aggressive_on_exit: FALSE,
            armed_riders_upgrade_weapon_set: FALSE,
            reset_mood_check_time_on_exit: TRUE,
            destroy_riders_who_are_not_free_to_exit: FALSE,
            is_delay_exit_in_air: FALSE,
        }
    }

    pub fn parse_field(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        parse_ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let is = |name: &str| field.eq_ignore_ascii_case(name);
        let result = if is("Slots") {
            reader.parse_int(&mut self.slot_capacity)
        } else if is("ScatterNearbyOnExit") {
            reader.parse_bool(&mut self.scatter_nearby_on_exit)
        } else if is("OrientLikeContainerOnExit") {
            reader.parse_bool(&mut self.orient_like_container_on_exit)
        } else if is("KeepContainerVelocityOnExit") {
            reader.parse_bool(&mut self.keep_container_velocity_on_exit)
        } else if is("GoAggressiveOnExit") {
            reader.parse_bool(&mut self.go_aggressive_on_exit)
        } else if is("ResetMoodCheckTimeOnExit") {
            reader.parse_bool(&mut self.reset_mood_check_time_on_exit)
        } else if is("DestroyRidersWhoAreNotFreeToExit") {
            reader.parse_bool(&mut self.destroy_riders_who_are_not_free_to_exit)
        } else if is("ExitBone") {
            reader.parse_ascii_string(&mut self.exit_bone)
        } else if is("ExitPitchRate") {
            reader.parse_angular_velocity_real(&mut self.exit_pitch_rate)
        } else if is("InitialPayload") {
            self.initial_payload.parse(reader)
        } else if is("HealthRegen%PerSec") {
            reader.parse_real(&mut self.health_regen)
        } else if is("ExitDelay") {
            reader.parse_duration_unsigned_int(&mut self.exit_delay)
        } else if is("ArmedRidersUpgradeMyWeaponSet") {
            reader.parse_bool(&mut self.armed_riders_upgrade_weapon_set)
        } else if is("DelayExitInAir") {
            reader.parse_bool(&mut self.is_delay_exit_in_air)
        } else {
            return self.open.parse_field(field, reader, parse_ctx);
        };
        Some(result)
    }
}

impl ModuleData for TransportContainModuleData {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.open.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.open.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask(0x057 | FacetMask::EXIT_INTERFACE.0)
    }
}

/// `TransportContain`'s own members on top of [`OpenContainCore`].
#[derive(Debug, Clone, PartialEq)]
pub struct TransportContainState {
    pub core: OpenContainCore,
    pub payload_created: Bool,
    pub extra_slots_in_use: Int,
    pub frame_exit_not_busy: UnsignedInt,
}

impl TransportContainState {
    pub fn new(passengers_allowed_to_fire: Bool) -> Self {
        Self {
            core: OpenContainCore::new(passengers_allowed_to_fire),
            payload_created: FALSE,
            extra_slots_in_use: 0,
            frame_exit_not_busy: 0,
        }
    }
}

/// Access to the `TransportContain` base of a transport-derived container.
pub trait TransportBehavior: ContainBehavior {
    fn transport_data(&self) -> &TransportContainModuleData;
    fn transport(&self) -> &TransportContainState;
    fn transport_mut(&mut self) -> &mut TransportContainState;
}

/// `Object::getTransportSlotCount`.
pub fn transport_slot_count(sim: &dyn SimView, obj: ObjectSlot) -> Int {
    if sim.contain_is_special_zero_slot(obj) != FALSE {
        return sim
            .contain_items(obj)
            .into_iter()
            .filter_map(|id| sim.find_object(id))
            .map(|rider| transport_slot_count(sim, rider))
            .sum();
    }
    sim.object(obj)
        .and_then(|o| {
            o.thing
                .template_override()
                .get_final(&sim.rules().thing_factory)
        })
        .map_or(0, |t| t.get_raw_transport_slot_count())
}

/// Replaces a "fake" container (a parachute) by the first object inside it.
pub fn unwrap_special_zero_slot_rider(sim: &dyn SimView, rider: ObjectSlot) -> ObjectSlot {
    if sim.contain_is_special_zero_slot(rider) != FALSE {
        if let Some(first) = sim
            .contain_items(rider)
            .first()
            .and_then(|&id| sim.find_object(id))
        {
            return first;
        }
    }
    rider
}

/// `Object::isUsingAirborneLocomotor`.
pub fn is_using_airborne_locomotor(sim: &dyn SimView, obj: ObjectSlot) -> bool {
    sim.object(obj)
        .and_then(|o| o.ai_locomotor.as_ref())
        .is_some_and(|l| (l.get_legal_surfaces() & (LocomotorSurfaceType::Air as Int)) != 0)
}

/// Exit adjustments shared by transports and mob nexuses: orientation and inherited velocity.
pub fn inherit_container_motion(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    rider: ObjectSlot,
    orient_like_container: Bool,
    keep_container_velocity: Bool,
    exit_pitch_rate: Real,
) {
    if orient_like_container != FALSE {
        if let Some(angle) = sim.object(owner).map(|o| o.thing.get_orientation()) {
            sim.set_object_orientation(rider, angle);
        }
    }
    if keep_container_velocity != FALSE
        && sim.has_physics(owner) != FALSE
        && sim.has_physics(rider) != FALSE
    {
        let mut force = sim.physics_velocity(owner);
        let mass = sim.physics_mass(rider);
        force.x *= mass;
        force.y *= mass;
        force.z *= mass;
        sim.physics_apply_motive_force(rider, &force);

        let pitch_rate = sim.physics_center_of_mass_offset(rider) * exit_pitch_rate;
        sim.physics_set_pitch_rate(rider, pitch_rate);
    }
}

/// The `HealthRegen%PerSec` loop shared by transports and mob nexuses.
pub fn regenerate_riders(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    riders: &[ObjectId],
    health_regen: Real,
) {
    let source = sim.object(owner).map_or(ObjectId::INVALID, |o| o.id);
    for &id in riders {
        let Some(rider) = sim.find_object(id) else {
            continue;
        };
        let Some(body) = sim.object(rider).and_then(|o| o.body_slot) else {
            continue;
        };
        let crate::arena::ModuleSidecar::Body {
            health, max_health, ..
        } = sim.module_sidecar(body)
        else {
            continue;
        };
        if health < max_health {
            let regen = max_health * health_regen / 100.0 * SECONDS_PER_LOGICFRAME_REAL;
            // `Object::attemptHealing(regen, source)`.
            let mut healing = crate::combat::damage::DamageInfo::default();
            healing.input.damage_type = DamageType::Healing;
            healing.input.death_type = DeathType::NoneType;
            healing.input.source_id = source;
            healing.input.amount = regen;
            sim.attempt_healing(rider, &mut healing);
        }
    }
}

/// `TransportContain::isValidContainerFor`.
pub fn transport_is_valid_container_for<T: TransportBehavior + ?Sized>(
    t: &T,
    sim: &dyn SimView,
    ctx: ModuleContext,
    rider: ObjectSlot,
    check_capacity: Bool,
) -> Bool {
    let rider = unwrap_special_zero_slot_rider(sim, rider);
    let state = t.transport();
    if state
        .core
        .is_valid_container_for(&t.transport_data().open, sim, ctx, rider)
        == FALSE
    {
        return FALSE;
    }
    // Only our own units can be transported.
    let (Some(r), Some(me)) = (sim.object(rider), sim.object(ctx.owner_slot)) else {
        return FALSE;
    };
    if sim.player_index(r.team_id) != sim.player_index(me.team_id) {
        return FALSE;
    }
    let slots = transport_slot_count(sim, rider);
    if slots == 0 {
        return FALSE;
    }
    if check_capacity != FALSE {
        let used = state.extra_slots_in_use + state.core.contain_list_size as Int;
        Bool::from(used + slots <= t.contain_max_value())
    } else {
        TRUE
    }
}

/// `TransportContain::letRidersUpgradeWeaponSet`.
fn let_riders_upgrade_weapon_set<T: TransportBehavior + ?Sized>(
    t: &T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) {
    if t.transport_data().armed_riders_upgrade_weapon_set == FALSE {
        return;
    }
    let mut any_viable = FALSE;
    for &id in &t.transport().core.contain_list {
        let Some(rider) = sim.find_object(id) else {
            continue;
        };
        if sim.object_is_kind_of(rider, KindOfType::Infantry) == FALSE {
            continue;
        }
        let viable = sim.object(rider).is_some_and(|o| {
            o.weapon_set.weapons.iter().flatten().any(|w| {
                w.template.is_contact_weapon() == FALSE && w.template.is_damage_weapon() == TRUE
            })
        });
        if viable {
            any_viable = TRUE;
        }
    }
    sim.set_weapon_set_flag(ctx.owner_slot, WeaponSetType::PlayerUpgrade, any_viable);
}

/// `TransportContain::onContaining`.
pub fn transport_on_containing<T: TransportBehavior + ?Sized>(
    t: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    rider: ObjectSlot,
) {
    // Objects inside a transport are held.
    sim.set_disabled(rider, DisabledType::Held);

    let slots = transport_slot_count(sim, rider);
    t.transport_mut().extra_slots_in_use += slots - 1;

    if t.transport().core.contain_list_size == 1
        && sim.object_has_drawable(ctx.owner_slot) != FALSE
    {
        sim.set_model_condition_state(ctx.owner_slot, ModelConditionFlagType::Loaded);
    }
    let_riders_upgrade_weapon_set(t, sim, ctx);
}

/// `TransportContain::onRemoving`.
pub fn transport_on_removing<T: TransportBehavior + ?Sized>(
    t: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    rider: ObjectSlot,
) {
    let owner = ctx.owner_slot;
    // No longer held inside a transport.
    sim.clear_disabled(rider, DisabledType::Held);

    let data = t.transport_data();
    let go_aggressive = data.go_aggressive_on_exit;
    let reset_mood = data.reset_mood_check_time_on_exit;
    let exit_delay = data.exit_delay;
    inherit_container_motion(
        sim,
        owner,
        rider,
        data.orient_like_container_on_exit,
        data.keep_container_velocity_on_exit,
        data.exit_pitch_rate,
    );

    let slots = transport_slot_count(sim, rider);
    t.transport_mut().extra_slots_in_use -= slots - 1;

    if t.transport().core.contain_list_size == 0 && sim.object_has_drawable(owner) != FALSE {
        sim.clear_model_condition_state(owner, ModelConditionFlagType::Loaded);
    }

    // Let riders fall even if their locomotor sticks to the ground.
    if sim.object_is_above_terrain(owner) != FALSE && sim.has_physics(rider) != FALSE {
        sim.set_physics_allow_to_fall(rider, TRUE);
    }
    if go_aggressive != FALSE && sim.has_ai(rider) != FALSE {
        sim.ai_set_attitude(rider, AttitudeType::Aggressive);
    }
    if sim
        .object(owner)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
    {
        scatter_to_nearby_position(sim, owner, rider);
    }
    if reset_mood != FALSE && sim.has_ai(rider) != FALSE {
        sim.ai_wake_up_and_attempt_to_target(rider);
    }

    t.transport_mut().frame_exit_not_busy = sim.frame().wrapping_add(exit_delay);

    let_riders_upgrade_weapon_set(t, sim, ctx);
}

/// `TransportContain::createPayload`.
fn transport_create_payload<T: TransportBehavior + ?Sized>(
    t: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) {
    let payload = t.transport_data().initial_payload.clone();
    let template = sim.rules().thing_factory.find_template_id(&payload.name);
    let owner_team = sim.object(ctx.owner_slot).map(|o| o.team_id);

    if let (Some(template), Some(owner_team)) = (template, owner_team) {
        let team = sim.player_default_team(owner_team);
        t.transport_mut().core.load_sounds_enabled = FALSE;
        for _ in 0..payload.count {
            let rider = sim.spawn_object(template, team, Coord3D::default(), 0.0);
            if t.is_valid_container(sim, ctx, rider, TRUE) != FALSE {
                add_to_contain(t, sim, ctx, rider);
            }
        }
        t.transport_mut().core.load_sounds_enabled = TRUE;
    }
    t.transport_mut().payload_created = TRUE;
}

/// `TransportContain::update`.
pub fn transport_update<T: TransportBehavior + ?Sized>(
    t: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) -> UpdateSleepTime {
    if t.transport().payload_created == FALSE {
        #[cfg(feature = "retail-crc")]
        transport_create_payload(t, sim, ctx);
        #[cfg(not(feature = "retail-crc"))]
        if sim.object(ctx.owner_slot).is_some_and(|o| !o.is_destroyed()) {
            transport_create_payload(t, sim, ctx);
        }
    }

    let health_regen = t.transport_data().health_regen;
    if health_regen != 0.0 {
        let riders = t.transport().core.contain_list.clone();
        regenerate_riders(sim, ctx.owner_slot, &riders, health_regen);
    }
    update_open_contain(t, sim, ctx)
}

/// `TransportContain::isSpecificRiderFreeToExit`.
pub fn transport_is_rider_free_to_exit(
    sim: &dyn SimView,
    ctx: ModuleContext,
    rider: Option<ObjectSlot>,
) -> bool {
    let Some(rider) = rider else {
        return true; // I can, in general, exit people.
    };
    let me = ctx.owner_slot;
    if sim.has_ai(me) != FALSE && sim.ai_free_to_exit(me) == FALSE {
        return false;
    }

    // A contained container may only unload through a special overlord-style outer one.
    #[cfg(not(feature = "retail-crc"))]
    if let Some(outer) = sim.object(me).and_then(|o| o.contained_by) {
        if sim.contain_is_special_overlord_style(outer) == FALSE {
            return false;
        }
    }

    if is_using_airborne_locomotor(sim, me) {
        return true;
    }
    if sim.has_ai(rider) == FALSE || sim.ai_has_cur_locomotor(rider) == FALSE {
        return false;
    }
    let Some((layer, pos)) = sim.object(me).map(|o| (o.layer, *o.thing.get_position())) else {
        return false;
    };
    // Amphibious transports: he cannot be forced somewhere he cannot walk to.
    sim.valid_movement_terrain(layer, rider, &pos) != FALSE
}

/// `TransportContain::killRidersWhoAreNotFreeToExit`.
pub fn transport_kill_riders_not_free_to_exit<T: TransportBehavior + ?Sized>(
    t: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
) {
    let destroy = t.transport_data().destroy_riders_who_are_not_free_to_exit;
    #[cfg(not(feature = "retail-crc"))]
    let death = if t.transport_data().open.is_burned_death_to_units != FALSE {
        DeathType::Burned
    } else {
        DeathType::Normal
    };
    #[cfg(feature = "retail-crc")]
    let death = DeathType::Normal;

    for id in t.transport().core.contain_list.clone() {
        let Some(rider) = sim.find_object(id) else {
            continue;
        };
        if transport_is_rider_free_to_exit(sim, ctx, Some(rider)) {
            continue;
        }
        if destroy != FALSE {
            sim.destroy_object(rider);
            // `Object::onDestroy` removes the rider from its container immediately.
            remove_from_contain(t, sim, ctx, rider, FALSE);
        } else {
            kill_object(sim, rider, DamageType::Unresistable, death);
        }
    }
}

/// `TransportContain::isPassengerAllowedToFire`.
pub fn transport_passenger_allowed_to_fire(
    core: &OpenContainCore,
    sim: &dyn SimView,
    ctx: ModuleContext,
    id: ObjectId,
) -> Bool {
    if let Some(passenger) = sim.find_object(id) {
        // Only infantry and the like, no vehicles.
        if sim.object_is_kind_of(passenger, KindOfType::Infantry) == FALSE {
            return FALSE;
        }
    }
    // We may be riding on an Overlord.
    if let Some(outer) = sim.object(ctx.owner_slot).and_then(|o| o.contained_by) {
        if sim.contain_is_special_overlord_style(outer) != FALSE {
            return sim.contain_is_passenger_allowed_to_fire(outer, id);
        }
    }
    core.is_passenger_allowed_to_fire(sim, ctx)
}

/// `TransportContain::isExitBusy`.
pub fn transport_exit_busy<T: TransportBehavior + ?Sized>(
    t: &T,
    sim: &dyn SimView,
    ctx: ModuleContext,
) -> Bool {
    if t.transport_data().is_delay_exit_in_air != FALSE
        && sim.object_is_above_terrain(ctx.owner_slot) != FALSE
    {
        return TRUE;
    }
    Bool::from(sim.frame() < t.transport().frame_exit_not_busy)
}

/// `TransportContain::reserveDoorForExit`.
pub fn transport_reserve_door(
    sim: &dyn SimView,
    ctx: ModuleContext,
    rider: Option<ObjectSlot>,
) -> ExitDoorType {
    if transport_is_rider_free_to_exit(sim, ctx, rider) {
        ExitDoorType::Door1
    } else {
        ExitDoorType::DoorNoneAvailable
    }
}

/// `TransportContain::onCapture`.
pub fn transport_on_capture<T: TransportBehavior + ?Sized>(
    t: &mut T,
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    old_team: TeamId,
    new_team: TeamId,
) {
    let old_player = sim.player_index(old_team);
    let new_player = sim.player_index(new_team);
    if old_player == new_player {
        return;
    }
    let unmanned = sim
        .object(ctx.owner_slot)
        .is_some_and(|o| o.is_disabled_by_type(DisabledType::Unmanned) != FALSE);
    if unmanned {
        // Sniped: eject everyone, otherwise the next one to exit recaptures the vehicle.
        t.remove_all(sim, ctx, FALSE);
        return;
    }
    #[cfg(not(feature = "retail-crc"))]
    if sim.team_relationship_to_player(old_team, new_player) == Relationship::Allies {
        return;
    }
    #[cfg(feature = "retail-crc")]
    let _ = Relationship::Allies;
    order_all_passengers_to_exit(t, sim, ctx, CommandSourceType::FromAi);
}

/// Serializes `TransportContain::xfer` (version `1`).
pub fn transport_xfer(
    state: &mut TransportContainState,
    xfer: &mut dyn Xfer,
    sim: &dyn SimView,
) -> Result<(), XferError> {
    let _ = xfer_module_version(xfer, 1)?;
    state.core.xfer(xfer, sim)?;
    xfer.xfer_bool(&mut state.payload_created)?;
    xfer.xfer_int(&mut state.extra_slots_in_use)?;
    xfer.xfer_unsigned_int(&mut state.frame_exit_not_busy)
}

/// Runtime instance of `TransportContain` (`PKT-299`).
pub struct TransportContain {
    pub data: TransportContainModuleData,
    pub state: TransportContainState,
}

impl TransportContain {
    pub fn new(data: TransportContainModuleData) -> Self {
        let state = TransportContainState::new(data.open.passengers_allowed_to_fire);
        Self { data, state }
    }
}

impl TransportBehavior for TransportContain {
    fn transport_data(&self) -> &TransportContainModuleData {
        &self.data
    }
    fn transport(&self) -> &TransportContainState {
        &self.state
    }
    fn transport_mut(&mut self) -> &mut TransportContainState {
        &mut self.state
    }
}

impl ContainBehavior for TransportContain {
    fn open_data(&self) -> &OpenContainModuleData {
        &self.data.open
    }
    fn open_core(&self) -> &OpenContainCore {
        &self.state.core
    }
    fn open_core_mut(&mut self) -> &mut OpenContainCore {
        &mut self.state.core
    }
    fn contain_max_value(&self) -> Int {
        self.data.slot_capacity
    }
    fn xfer_module(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        transport_xfer(&mut self.state, xfer, sim)
    }
    fn update_module(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        transport_update(self, sim, ctx)
    }
    fn is_valid_container(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        rider: ObjectSlot,
        check_capacity: Bool,
    ) -> Bool {
        transport_is_valid_container_for(self, sim, ctx, rider, check_capacity)
    }
    fn passenger_allowed_to_fire(
        &self,
        sim: &dyn SimView,
        ctx: ModuleContext,
        id: ObjectId,
    ) -> Bool {
        transport_passenger_allowed_to_fire(&self.state.core, sim, ctx, id)
    }
    fn contained(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_containing(self, sim, ctx, rider);
    }
    fn removing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, rider: ObjectSlot) {
        transport_on_removing(self, sim, ctx, rider);
    }
    fn kill_riders_not_free_to_exit(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        transport_kill_riders_not_free_to_exit(self, sim, ctx);
    }
    fn capture(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        old_team: TeamId,
        new_team: TeamId,
    ) {
        transport_on_capture(self, sim, ctx, old_team, new_team);
    }
    fn exit_busy(&self, sim: &dyn SimView, ctx: ModuleContext) -> Bool {
        transport_exit_busy(self, sim, ctx)
    }
    fn reserve_door(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        specific_obj: Option<ObjectSlot>,
    ) -> ExitDoorType {
        transport_reserve_door(sim, ctx, specific_obj)
    }
}

impl_contain_module!(TransportContain);

pub fn parse_transport_contain_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = TransportContainModuleData::new(tag_key, module_name_key);
    reader.read_block_fields(|field, r| {
        data.parse_field(field, r, parse_ctx)
            .unwrap_or(Err(IniError::UnknownToken))
    })?;
    Ok(Box::new(data))
}

pub fn create_transport_contain(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<TransportContainModuleData>()
        .expect("TransportContain data type mismatch")
        .clone();
    Box::new(TransportContain::new(typed))
}
