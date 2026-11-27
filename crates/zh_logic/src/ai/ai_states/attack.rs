//! `PKT-429`: the attack states of `AIStates.cpp`: `AIAttackState` with its
//! `AttackStateMachine` of pursue, approach, aim and fire. Packet doc:
//! `packets/phase6/PKT-429_AIStates.md`. Golden test:
//! `tests/golden_s14_attack_states.rs`.
//!
//! `AIAttackState` sits in the AI's own machine and in the attack-move sub
//! machine ([`Host`]). Turret questions go to [`crate::ai::seams::AiSeams`] and
//! are asked only of a module whose data has a turret. Squad, hunt and attack
//! area are not ported.
//!
//! The reference deletes the attack machine from the attack state's `onExit`,
//! even while one of its states is running (an order given by what a shot set
//! off). What it does then reads freed memory; here the running state finds no
//! machine, works on a blank one and the call ends.

use super::movement::{self, InternalMove};
use crate::ai::ai_state_machine::state_id;
use crate::ai::seams::ai_seams;
use crate::arena::{ModuleContext, ObjectSlot};
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{
    self, ai, MoodMatrixAction, FAST_AS_POSSIBLE, MAX_TURRETS,
};
use crate::modules::update::physics_update;
use crate::object::weapon::{Weapon, NO_MAX_SHOTS_LIMIT};
use crate::object::weapon_actions;
use crate::object::weapon_queries;
use crate::object::weapon_set::{
    self, CanAttackResult, WeaponChoiceCriteria, WeaponSet, WeaponSetConditionType, WeaponSlotType, ATTACK_CONTINUED_TARGET, ATTACK_CONTINUED_TARGET_FORCED, ATTACK_NEW_TARGET,
};
use crate::pathfinder::pathfind_constants::PATHFIND_CELL_SIZE_F;
use crate::rules::combat_types::{DamageType, WeaponStatus};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{all_weapon_fire_flags, SimApi, PRIVATE_STATUS_OFF_MAP};
use crate::spatial::partition_query::DistanceCalculationType;
use zh_common::core::state_machine::{
    StateExitType, StateId, StateReturnType, EXIT_MACHINE_WITH_FAILURE, INVALID_STATE_ID,
    MACHINE_DONE_STATE_ID, MAX_TRANSITION_DEPTH,
};
use zh_common::math::{gm_fabsf, Coord3D};
use zh_common::types::{
    Bool, ObjectId, Real, Relationship, TeamId, UnsignedInt, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError};

/// `AttackStateMachine::StateType`.
pub const CHASE_TARGET: StateId = 0;
pub const APPROACH_TARGET: StateId = 1;
pub const AIM_AT_TARGET: StateId = 2;
pub const FIRE_WEAPON: StateId = 3;

/// `MIN_RECOMPUTE_TIME` of the approach and pursue states.
const MIN_RECOMPUTE_TIME: UnsignedInt = 10;
/// `MAA_Action_Ok`.
const MAA_ACTION_OK: UnsignedInt = 0x01;
/// `REL_THRESH`: about two degrees.
const REL_THRESH: Real = 0.035;

/// The machine an `AIAttackState` is a state of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Host {
    /// The AI's own machine.
    Main,
    /// The attack-move sub machine.
    AttackMove,
}

/// Whether the AI's machine has this attack state.
pub fn is_attack_state(id: StateId) -> bool {
    matches!(
        id,
        state_id::ATTACK_POSITION
            | state_id::ATTACK_OBJECT
            | state_id::FORCE_ATTACK_OBJECT
            | state_id::ATTACK_AND_FOLLOW_OBJECT
    )
}

/// What the movement code calls the approach and pursue states of the attack
/// machines, which keep their `AIInternalMoveToState` members here.
const MOVE_KEY: StateId = 0x4000_0000;

fn move_key(host: Host, state: StateId) -> StateId {
    MOVE_KEY | (StateId::from(host == Host::AttackMove) << 4) | state
}

/// The host and attack machine state a movement key names.
pub fn split_move_key(key: StateId) -> Option<(Host, StateId)> {
    (key & MOVE_KEY != 0).then(|| {
        let host = if key & 0x10 != 0 {
            Host::AttackMove
        } else {
            Host::Main
        };
        (host, key & 0x0f)
    })
}

/// The members of `AIAttackApproachTargetState` and
/// `AIAttackPursueTargetState`, which are the same.
#[derive(Debug, Clone, Copy)]
pub struct ApproachData {
    pub base: InternalMove,
    pub prev_victim_pos: Coord3D,
    pub approach_timestamp: UnsignedInt,
    pub stop_if_in_range: Bool,
    pub is_initial_approach: Bool,
}

impl Default for ApproachData {
    fn default() -> Self {
        Self {
            base: InternalMove::default(),
            prev_victim_pos: Coord3D::default(),
            approach_timestamp: 0,
            stop_if_in_range: FALSE,
            is_initial_approach: TRUE,
        }
    }
}

/// What `CHASE_TARGET` is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Chase {
    /// Pursue a target moving away.
    Pursue,
    /// Approach a spot.
    #[default]
    Approach,
    /// `ContinueState`: a gun riding on something else.
    Continue,
    /// `FailureState`: something that cannot move.
    Failure,
}

/// The `AttackStateMachine` and its states.
#[derive(Debug, Clone)]
pub struct AttackMachine {
    generation: u32,
    pub sleep_till: UnsignedInt,
    pub current: StateId,
    pub goal_object_id: ObjectId,
    pub goal_position: Coord3D,
    pub locked: Bool,
    pub default_state_inited: Bool,
    pub chase: Chase,
    pub has_approach: bool,
    pub follow: Bool,
    pub attacking_object: Bool,
    pub force_attacking: Bool,
    /// The state at `CHASE_TARGET`, when it moves.
    pub chase_state: ApproachData,
    /// The state at `APPROACH_TARGET`.
    pub approach_state: ApproachData,
    pub can_turn_in_place: Bool,
    pub set_locomotor: Bool,
}

impl Default for AttackMachine {
    fn default() -> Self {
        Self {
            generation: 0,
            sleep_till: 0,
            current: INVALID_STATE_ID,
            goal_object_id: ObjectId::INVALID,
            goal_position: Coord3D::default(),
            locked: FALSE,
            default_state_inited: FALSE,
            chase: Chase::Approach,
            has_approach: false,
            follow: FALSE,
            attacking_object: FALSE,
            force_attacking: FALSE,
            chase_state: ApproachData::default(),
            approach_state: ApproachData::default(),
            can_turn_in_place: FALSE,
            set_locomotor: FALSE,
        }
    }
}

impl AttackMachine {
    fn defines(&self, id: StateId) -> bool {
        matches!(id, CHASE_TARGET | AIM_AT_TARGET | FIRE_WEAPON)
            || id == APPROACH_TARGET && self.has_approach
    }

    /// The success and failure targets of a state.
    fn links(&self, id: StateId) -> (StateId, StateId) {
        match id {
            AIM_AT_TARGET => (FIRE_WEAPON, EXIT_MACHINE_WITH_FAILURE),
            FIRE_WEAPON => (AIM_AT_TARGET, AIM_AT_TARGET),
            APPROACH_TARGET => (AIM_AT_TARGET, EXIT_MACHINE_WITH_FAILURE),
            _ => match self.chase {
                Chase::Pursue => (APPROACH_TARGET, APPROACH_TARGET),
                Chase::Approach => (AIM_AT_TARGET, EXIT_MACHINE_WITH_FAILURE),
                Chase::Continue | Chase::Failure => {
                    (EXIT_MACHINE_WITH_FAILURE, EXIT_MACHINE_WITH_FAILURE)
                }
            },
        }
    }

    fn approach_data(&mut self, state: StateId) -> &mut ApproachData {
        if state == APPROACH_TARGET {
            &mut self.approach_state
        } else {
            &mut self.chase_state
        }
    }

    /// `AttackStateMachine::xfer`.
    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut base_version: XferVersion = 1;
        xfer.xfer_version(&mut base_version, 1)?;
        xfer.xfer_unsigned_int(&mut self.sleep_till)?;
        let mut default_state = AIM_AT_TARGET;
        xfer.xfer_unsigned_int(&mut default_state)?;
        let mut current = self.current;
        xfer.xfer_unsigned_int(&mut current)?;
        let mut snapshot_all = FALSE;
        xfer.xfer_bool(&mut snapshot_all)?;
        if self.current == INVALID_STATE_ID {
            self.current = AIM_AT_TARGET;
        }
        self.xfer_state(xfer)?;
        xfer.xfer_object_id(&mut self.goal_object_id)?;
        xfer.xfer_coord3d(&mut self.goal_position)?;
        xfer.xfer_bool(&mut self.locked)?;
        xfer.xfer_bool(&mut self.default_state_inited)
    }

    fn xfer_state(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let moves = match self.current {
            APPROACH_TARGET => true,
            CHASE_TARGET => matches!(self.chase, Chase::Pursue | Chase::Approach),
            AIM_AT_TARGET => {
                xfer.xfer_bool(&mut self.can_turn_in_place)?;
                return xfer.xfer_bool(&mut self.set_locomotor);
            }
            _ => false,
        };
        if !moves {
            return Ok(());
        }
        let (mut follow, mut attacking_object) = (self.follow, self.attacking_object);
        let current = self.current;
        let data = self.approach_data(current);
        data.base.xfer(xfer)?;
        xfer.xfer_coord3d(&mut data.prev_victim_pos)?;
        xfer.xfer_unsigned_int(&mut data.approach_timestamp)?;
        xfer.xfer_bool(&mut follow)?;
        xfer.xfer_bool(&mut attacking_object)?;
        xfer.xfer_bool(&mut data.stop_if_in_range)?;
        xfer.xfer_bool(&mut data.is_initial_approach)
    }
}

/// An `AIAttackState` object.
#[derive(Debug, Clone, Default)]
pub struct AttackState {
    /// `m_attackMachine`, while in the state.
    pub machine: Option<Box<AttackMachine>>,
    pub original_victim_pos: Coord3D,
    /// `m_victimTeam`; not saved.
    victim_team: Option<TeamId>,
    /// `m_lockedWeaponOnEnter`; not saved.
    locked_weapon_on_enter: Option<WeaponSlotType>,
}

impl AttackState {
    /// `AIAttackState::xfer`.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut has_machine = Bool::from(self.machine.is_some());
        xfer.xfer_bool(&mut has_machine)?;
        xfer.xfer_coord3d(&mut self.original_victim_pos)?;
        match &mut self.machine {
            Some(machine) => machine.xfer(xfer),
            None => Ok(()),
        }
    }
}

/// The attack state objects of an `AIStateMachine`.
#[derive(Debug, Clone, Default)]
pub struct AttackStates {
    /// The one state object of the AI's machine that is entered; the reference
    /// has one for each of the four attack states.
    pub main: AttackState,
    /// The one of the attack-move sub machine.
    pub attack_move: AttackState,
    /// What a state whose machine was deleted under it works on.
    orphan: Box<AttackMachine>,
    generations: u32,
}

impl AttackStates {
    pub fn state(&mut self, host: Host) -> &mut AttackState {
        match host {
            Host::Main => &mut self.main,
            Host::AttackMove => &mut self.attack_move,
        }
    }

    fn machine(&mut self, host: Host) -> &mut AttackMachine {
        let state = match host {
            Host::Main => &mut self.main,
            Host::AttackMove => &mut self.attack_move,
        };
        match &mut state.machine {
            Some(machine) => machine,
            None => &mut self.orphan,
        }
    }

    /// The `AIInternalMoveToState` members behind a movement key.
    pub(super) fn internal_move(&mut self, host: Host, state: StateId) -> &mut InternalMove {
        &mut self.machine(host).approach_data(state).base
    }
}

// -----------------------------------------------------------------------------
// The unit
// -----------------------------------------------------------------------------

fn machine(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> &mut AttackMachine {
    ai(sim, ctx).machine.attacks.machine(host)
}

fn approach(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> &mut ApproachData {
    machine(sim, ctx, host).approach_data(state)
}

fn attack_state(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> &mut AttackState {
    ai(sim, ctx).machine.attacks.state(host)
}

/// Whether the machine created as `generation` is still there.
fn alive(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, generation: u32) -> bool {
    attack_state(sim, ctx, host)
        .machine
        .as_ref()
        .is_some_and(|m| m.generation == generation)
}

fn position(sim: &dyn SimApi, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

fn is_kind(sim: &dyn SimApi, slot: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(slot, kind) != FALSE
}

fn has_status(sim: &dyn SimApi, slot: ObjectSlot, status: ObjectStatusType) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.status.test(status as i32) != FALSE)
}

fn is_effectively_dead(sim: &dyn SimApi, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| o.is_effectively_dead() != FALSE)
}

fn is_mobile(sim: &dyn SimApi, slot: ObjectSlot) -> bool {
    !is_kind(sim, slot, KindOfType::Immobile)
        && sim
            .object(slot)
            .is_some_and(|o| o.disabled_mask.to_unsigned_int() == 0)
}

fn is_human(sim: &dyn SimApi, slot: ObjectSlot) -> bool {
    sim.object(slot)
        .is_some_and(|o| sim.player_is_human(o.team_id) != FALSE)
}

fn id_of(sim: &dyn SimApi, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).map_or(ObjectId::INVALID, |o| o.id)
}

/// A copy of `getCurrentWeapon`, for what only asks.
fn current_weapon(sim: &dyn SimApi, owner: ObjectSlot) -> Option<Weapon> {
    let set = &sim.object(owner)?.weapon_set;
    if set.has_any_weapon() == FALSE {
        return None;
    }
    set.get_weapon_in_weapon_slot(set.get_cur_weapon_slot())
        .cloned()
}

fn current_weapon_mut(sim: &mut dyn SimApi, owner: ObjectSlot) -> Option<&mut Weapon> {
    let set: &mut WeaponSet = &mut sim.object_mut(owner)?.weapon_set;
    if set.has_any_weapon() == FALSE {
        return None;
    }
    let slot = set.get_cur_weapon_slot();
    set.get_weapon_in_weapon_slot_mut(slot)
}

fn reaches_object(
    sim: &dyn SimApi,
    weapon: &Weapon,
    owner: ObjectSlot,
    victim: ObjectSlot,
) -> bool {
    weapon_queries::is_within_attack_range(weapon, sim, owner, victim) != FALSE
}

fn reaches_position(sim: &dyn SimApi, weapon: &Weapon, owner: ObjectSlot, pos: &Coord3D) -> bool {
    weapon_queries::is_within_attack_range_of_position(weapon, sim, owner, pos) != FALSE
}

/// `isAttackViewBlockedByObstacle` from where the unit stands.
fn view_blocked(sim: &dyn SimApi, owner: ObjectSlot, victim: ObjectSlot) -> bool {
    let (from, to) = (position(sim, owner), position(sim, victim));
    sim.attack_view_blocked(owner, &from, Some(victim), &to)
}

fn on_ground(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    ai_update::is_doing_ground_movement(sim, ctx) != FALSE
}

fn significantly_above_terrain(sim: &dyn SimApi, slot: ObjectSlot) -> bool {
    sim.object_is_significantly_above_terrain(slot) != FALSE
}

/// `canCrushOrSquish(other, TEST_CRUSH_OR_SQUISH)`.
fn can_crush_or_squish(sim: &dyn SimApi, owner: ObjectSlot, other: ObjectSlot) -> bool {
    if sim.object_can_crush(owner, other) != FALSE {
        return true;
    }
    // Anything with a `SquishCollide` is squished at any crushable level.
    let unmanned = sim
        .object(owner)
        .is_some_and(|o| o.is_disabled_by_type(DisabledType::Unmanned) != FALSE);
    !unmanned
        && sim.object_crusher_level(owner) != 0
        && sim.player_relationship(owner, other) != Relationship::Allies
        && sim
            .rules()
            .name_keys
            .lookup_key("SquishCollide")
            .is_some_and(|key| sim.has_module_by_name_key(other, key))
}

// -----------------------------------------------------------------------------
// The turrets
// -----------------------------------------------------------------------------

fn has_turret(sim: &mut dyn SimApi, ctx: ModuleContext, turret: usize) -> bool {
    ai(sim, ctx)
        .data
        .turrets
        .get(turret)
        .is_some_and(Option::is_some)
}

/// `getWhichTurretForCurWeapon`.
fn cur_turret(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<usize> {
    if !(0..MAX_TURRETS).any(|t| has_turret(sim, ctx, t)) {
        return None;
    }
    ai_seams(sim).which_turret_for_cur_weapon(ctx.owner_id)
}

fn turret_target_object(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    turret: usize,
    victim: Option<ObjectSlot>,
    force: Bool,
) {
    if has_turret(sim, ctx, turret) {
        let victim = victim.map(|slot| id_of(sim, slot));
        ai_seams(sim).set_turret_target_object(ctx.owner_id, turret, victim, force);
    }
}

fn turret_target_position(sim: &mut dyn SimApi, ctx: ModuleContext, turret: usize, pos: &Coord3D) {
    if has_turret(sim, ctx, turret) {
        ai_seams(sim).set_turret_target_position(ctx.owner_id, turret, pos);
    }
}

/// While first closing in, a turret may shoot at whatever the mood suggests.
fn aim_turret_at_mood_target(sim: &mut dyn SimApi, ctx: ModuleContext, force: Bool) {
    let Some(turret) = cur_turret(sim, ctx) else {
        return;
    };
    let target = ai_seams(sim).next_mood_target(ctx.owner_id, TRUE, FALSE);
    if let Some(target) = target.and_then(|id| sim.find_object(id)) {
        turret_target_object(sim, ctx, turret, Some(target), force);
    }
}

// -----------------------------------------------------------------------------
// The goals of the two machines
// -----------------------------------------------------------------------------

/// `getMachineGoalObject` of a state of the attack machine.
fn goal_object(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> Option<ObjectSlot> {
    let id = machine(sim, ctx, host).goal_object_id;
    sim.find_object(id)
}

/// `isGoalObjectDestroyed` of the attack machine.
fn goal_destroyed(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> bool {
    machine(sim, ctx, host).goal_object_id != ObjectId::INVALID
        && goal_object(sim, ctx, host).is_none()
}

/// `setGoalObject` of the attack machine.
fn set_goal_object(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    victim: Option<ObjectSlot>,
) {
    let goal = victim.map(|slot| (id_of(sim, slot), position(sim, slot)));
    let m = machine(sim, ctx, host);
    if m.locked != FALSE {
        return;
    }
    match goal {
        Some((id, pos)) => {
            m.goal_object_id = id;
            m.goal_position = pos;
        }
        None => m.goal_object_id = ObjectId::INVALID,
    }
}

fn host_goal_id(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> ObjectId {
    let unit = ai(sim, ctx);
    match host {
        Host::Main => unit.machine.goal_object_id,
        Host::AttackMove => unit.machine.moves.attack_move.sub.goal_object_id,
    }
}

fn host_goal_position(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> Coord3D {
    let unit = ai(sim, ctx);
    match host {
        Host::Main => unit.machine.goal_position,
        Host::AttackMove => unit.machine.moves.attack_move.sub.goal_position,
    }
}

/// `setGoalObject` of the host machine.
fn set_host_goal_object(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, victim: ObjectSlot) {
    let (id, pos) = (id_of(sim, victim), position(sim, victim));
    let unit = ai(sim, ctx);
    match host {
        Host::Main => unit.machine.set_goal_object(Some((id, pos))),
        Host::AttackMove => {
            let sub = &mut unit.machine.moves.attack_move.sub;
            if sub.locked == FALSE {
                sub.goal_object_id = id;
                sub.goal_position = pos;
            }
        }
    }
}

// -----------------------------------------------------------------------------
// The transition conditions
// -----------------------------------------------------------------------------

/// Whether something in the way, or the range, keeps the weapon from the victim.
fn out_of_weapon_range_object(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> bool {
    let owner = ctx.owner_slot;
    let (Some(victim), Some(weapon)) = (goal_object(sim, ctx, host), current_weapon(sim, owner))
    else {
        return false;
    };
    let ground = line_of_sight_matters(sim, ctx);
    let blocked = weapon.template.is_contact_weapon() == FALSE
        && ground
        && !significantly_above_terrain(sim, victim)
        && view_blocked(sim, owner, victim);
    let leech = weapon.has_leech_range() != FALSE;
    !leech && (blocked || !reaches_object(sim, &weapon, owner, victim))
}

/// `onGround` of the range conditions.
fn line_of_sight_matters(sim: &mut dyn SimApi, ctx: ModuleContext) -> bool {
    let owner = ctx.owner_slot;
    let ground = on_ground(sim, ctx);
    if ground
        || is_kind(sim, owner, KindOfType::Immobile)
        || is_kind(sim, owner, KindOfType::SpawnsAreTheWeapons)
    {
        return true;
    }
    sim.object(owner)
        .and_then(|o| o.contained_by)
        .is_some_and(|c| {
            is_kind(sim, c, KindOfType::Structure)
                || !has_status(sim, c, ObjectStatusType::AirborneTarget)
        })
}

fn out_of_weapon_range_position(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> bool {
    let owner = ctx.owner_slot;
    let pos = machine(sim, ctx, host).goal_position;
    let Some(weapon) = current_weapon(sim, owner) else {
        return false;
    };
    if line_of_sight_matters(sim, ctx) {
        let from = position(sim, owner);
        if sim.attack_view_blocked(owner, &from, None, &pos) {
            return true;
        }
    }
    !reaches_position(sim, &weapon, owner, &pos)
}

fn want_to_squish_target(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> bool {
    let owner = ctx.owner_slot;
    let Some(victim) = goal_object(sim, ctx, host) else {
        return false;
    };
    if sim.object(victim).is_some_and(|o| o.contained_by.is_some()) {
        return false;
    }
    cur_turret(sim, ctx).is_some()
        && sim.rules().ai_data.ai_crushes_infantry != FALSE
        && !is_human(sim, owner)
        && can_crush_or_squish(sim, owner, victim)
        && !is_kind(sim, owner, KindOfType::DontAutoCrushInfantry)
}

fn cannot_possibly_attack(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    attack_type: UnsignedInt,
) -> bool {
    let owner = ctx.owner_slot;
    let Some(victim) = goal_object(sim, ctx, host) else {
        return false;
    };
    if sim.object_is_able_to_attack(owner) == FALSE {
        return true;
    }
    let source = ai(sim, ctx).last_command_source;
    let result = sim.able_to_attack_specific_object(owner, attack_type, victim, source);
    result != CanAttackResult::Possible && result != CanAttackResult::PossibleAfterMoving
}

/// The first condition of `state` that holds, as the state to go to.
fn condition_target(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> Option<StateId> {
    let m = machine(sim, ctx, host);
    let (object, forced, chase) = (
        m.attacking_object != FALSE,
        m.force_attacking != FALSE,
        m.chase,
    );
    if state == CHASE_TARGET && chase == Chase::Continue {
        return (!out_of_weapon_range_object(sim, ctx, host)).then_some(AIM_AT_TARGET);
    }
    if state != AIM_AT_TARGET && state != FIRE_WEAPON {
        return None;
    }
    if !object {
        return out_of_weapon_range_position(sim, ctx, host).then_some(CHASE_TARGET);
    }
    if out_of_weapon_range_object(sim, ctx, host) {
        return Some(CHASE_TARGET);
    }
    if forced {
        if cannot_possibly_attack(sim, ctx, host, ATTACK_CONTINUED_TARGET_FORCED) {
            return Some(EXIT_MACHINE_WITH_FAILURE);
        }
        want_to_squish_target(sim, ctx, host).then_some(CHASE_TARGET)
    } else {
        if want_to_squish_target(sim, ctx, host) {
            return Some(CHASE_TARGET);
        }
        cannot_possibly_attack(sim, ctx, host, ATTACK_CONTINUED_TARGET)
            .then_some(EXIT_MACHINE_WITH_FAILURE)
    }
}

// -----------------------------------------------------------------------------
// Pursue and approach
// -----------------------------------------------------------------------------

/// Whether the victim is moving away at a speed worth chasing.
fn can_pursue(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    weapon: &Weapon,
    victim: ObjectSlot,
) -> bool {
    let owner = ctx.owner_slot;
    if physics_update::physics(sim, victim).is_none() || cur_turret(sim, ctx).is_none() {
        return false;
    }
    if sim.rules().ai_data.ai_crushes_infantry != FALSE
        && !is_human(sim, owner)
        && can_crush_or_squish(sim, owner, victim)
    {
        return true;
    }
    if weapon_queries::is_too_close(weapon, sim, owner, victim) != FALSE {
        return false;
    }
    let condition = sim.object_body_damage_state(owner);
    let penalty = sim.rules().global_data.movement_penalty_damage_state;
    let ours = ai(sim, ctx)
        .cur_locomotor()
        .map_or(0.0, |l| l.get_max_speed_for_condition(condition, penalty));
    let theirs = physics_update::forward_speed_2d(sim, victim);
    if theirs >= ours || theirs < ours / 10.0 {
        return false;
    }
    let (from, to) = (position(sim, owner), position(sim, victim));
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    let direction = sim
        .object(victim)
        .map(|o| o.thing.get_unit_direction_vector_2d())
        .unwrap_or_default();
    !(dx * direction.x + dy * direction.y < 0.0)
}

fn is_pursue(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, state: StateId) -> bool {
    state == CHASE_TARGET && machine(sim, ctx, host).chase == Chase::Pursue
}

/// `computePath` of the two moving states.
pub(super) fn compute_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> bool {
    if is_pursue(sim, ctx, host, state) {
        pursue_compute_path(sim, ctx, host, state)
    } else {
        approach_compute_path(sim, ctx, host, state)
    }
}

/// The part both `computePath` functions open with: `Some` is their answer.
fn recompute_gate(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
    force: &mut bool,
) -> Option<bool> {
    if approach(sim, ctx, host, state).base.waiting_for_path != FALSE {
        return Some(true);
    }
    let unit = ai(sim, ctx);
    if !*force && unit.has_path == FALSE && unit.waiting_for_path == FALSE {
        *force = true;
    }
    let now = sim.frame();
    let data = approach(sim, ctx, host, state);
    if !*force && now.wrapping_sub(data.approach_timestamp) < MIN_RECOMPUTE_TIME {
        return Some(true);
    }
    data.approach_timestamp = now;
    None
}

fn approach_compute_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> bool {
    let owner = ctx.owner_slot;
    if !is_mobile(sim, owner) {
        return false;
    }
    let mut force = ai(sim, ctx).is_blocked_and_stuck != FALSE;
    if let Some(answer) = recompute_gate(sim, ctx, host, state, &mut force) {
        return answer;
    }

    if let Some(victim) = goal_object(sim, ctx, host) {
        let (ours, theirs) = (position(sim, owner), position(sim, victim));
        let previous = approach(sim, ctx, host, state).prev_victim_pos;
        if !force && movement::is_same_position(&ours, &previous, &theirs) {
            return true;
        }
        let Some(weapon) = current_weapon(sim, owner) else {
            return false;
        };
        approach(sim, ctx, host, state).prev_victim_pos = theirs;
        if can_pursue(sim, ctx, &weapon, victim) {
            return false;
        }
        approach(sim, ctx, host, state).base.adjust_destinations = TRUE;
        if weapon.template.is_contact_weapon() != FALSE {
            // Run into the target, without slowing down.
            let id = id_of(sim, victim);
            ai(sim, ctx).ignore_obstacle_id = id;
            approach(sim, ctx, host, state).base.adjust_destinations = FALSE;
            ai(sim, ctx).path_extra_distance = 10.0 * PATHFIND_CELL_SIZE_F;
        }
        let data = approach(sim, ctx, host, state);
        data.base.goal_position = data.prev_victim_pos;
        data.base.waiting_for_path = TRUE;
        let mut center = Coord3D::default();
        if let Some(o) = sim.object(victim) {
            o.geometry
                .get_center_position(o.thing.get_position(), &mut center);
        }
        let id = id_of(sim, victim);
        ai_update::request_attack_path(sim, ctx, id, &center);
        approach(sim, ctx, host, state).stop_if_in_range = FALSE;
    } else {
        let goal = machine(sim, ctx, host).goal_position;
        let data = approach(sim, ctx, host, state);
        data.base.adjust_destinations = TRUE;
        data.stop_if_in_range = FALSE;
        data.base.goal_position = goal;
        // A spot does not move.
        if !force {
            return true;
        }
        data.base.waiting_for_path = TRUE;
        ai_update::request_attack_path(sim, ctx, ObjectId::INVALID, &goal);
    }
    true
}

fn pursue_compute_path(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> bool {
    let owner = ctx.owner_slot;
    if !is_mobile(sim, owner) || ai(sim, ctx).is_blocked_and_stuck != FALSE {
        return false;
    }
    let mut force = false;
    if let Some(answer) = recompute_gate(sim, ctx, host, state, &mut force) {
        return answer;
    }
    let Some(victim) = goal_object(sim, ctx, host) else {
        return false;
    };
    let (ours, theirs) = (position(sim, owner), position(sim, victim));
    let previous = approach(sim, ctx, host, state).prev_victim_pos;
    if !force && movement::is_same_position(&ours, &previous, &theirs) {
        return true;
    }
    let Some(weapon) = current_weapon(sim, owner) else {
        return false;
    };
    if !can_pursue(sim, ctx, &weapon, victim) {
        return false;
    }
    let data = approach(sim, ctx, host, state);
    data.prev_victim_pos = theirs;
    data.base.adjust_destinations = TRUE;
    data.base.goal_position = theirs;
    data.base.waiting_for_path = TRUE;
    ai_update::request_path(sim, ctx, &theirs, FALSE);
    approach(sim, ctx, host, state).stop_if_in_range = FALSE;
    true
}

/// `getCurrentStateID() != AI_GUARD_RETALIATE`, and what `AI_HUNT` asks.
fn current_state(sim: &mut dyn SimApi, ctx: ModuleContext) -> StateId {
    ai(sim, ctx).machine.current_state_id()
}

/// A computer player's units do not chase aircraft unless they are hunting.
fn ignores_aircraft(sim: &mut dyn SimApi, ctx: ModuleContext, victim: ObjectSlot) -> bool {
    current_state(sim, ctx) != state_id::HUNT
        && is_kind(sim, victim, KindOfType::Aircraft)
        && has_status(sim, victim, ObjectStatusType::AirborneTarget)
}

fn approach_on_enter(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    let owner = ctx.owner_slot;
    if is_kind(sim, owner, KindOfType::Projectile) {
        set_precise_z(sim, ctx, TRUE);
    }
    if goal_destroyed(sim, ctx, host) {
        return StateReturnType::Success;
    }
    let data = approach(sim, ctx, host, state);
    data.prev_victim_pos = Coord3D::default();
    data.approach_timestamp = 0u32.wrapping_sub(MIN_RECOMPUTE_TIME);

    let victim = goal_object(sim, ctx, host);
    if let Some(victim) = victim {
        let Some(weapon) = current_weapon(sim, owner) else {
            return StateReturnType::Failure;
        };
        if reaches_object(sim, &weapon, owner, victim) {
            let blocked = on_ground(sim, ctx)
                && !significantly_above_terrain(sim, victim)
                && view_blocked(sim, owner, victim);
            if !blocked {
                return StateReturnType::Success;
            }
        }
        if current_state(sim, ctx) != state_id::GUARD_RETALIATE {
            if is_human(sim, owner) {
                // `isAllowedToChase` is only set by the assault transport.
                if ai(sim, ctx).last_command_source == CommandSourceType::FromAi
                    && weapon.template.is_contact_weapon() == FALSE
                {
                    return StateReturnType::Failure;
                }
            } else if ignores_aircraft(sim, ctx, victim) {
                return StateReturnType::Failure;
            }
        }
        if can_pursue(sim, ctx, &weapon, victim) {
            return StateReturnType::Success;
        }
    } else {
        ai(sim, ctx).destroy_path();
    }

    if let Some(turret) = cur_turret(sim, ctx) {
        let m = machine(sim, ctx, host);
        let (object, force, goal) = (m.attacking_object, m.force_attacking, m.goal_position);
        if object != FALSE {
            turret_target_object(sim, ctx, turret, victim, force);
        } else {
            turret_target_position(sim, ctx, turret, &goal);
        }
    }
    if !approach_compute_path(sim, ctx, host, state) {
        return StateReturnType::Failure;
    }
    approach(sim, ctx, host, state).base.adjust_destinations = FALSE;
    let result = movement::internal_on_enter(sim, ctx, move_key(host, state));
    approach(sim, ctx, host, state).base.adjust_destinations = TRUE;
    result
}

/// Stealthed, undetected and not disguised.
fn is_hidden(sim: &dyn SimApi, victim: ObjectSlot) -> bool {
    has_status(sim, victim, ObjectStatusType::Stealthed)
        && !has_status(sim, victim, ObjectStatusType::Detected)
        && !has_status(sim, victim, ObjectStatusType::Disguised)
}

/// Whether the weapon is clearing the mine or trap it is aimed at.
fn is_disarming(sim: &dyn SimApi, weapon: Option<&Weapon>, victim: ObjectSlot) -> bool {
    !cfg!(feature = "retail-crc")
        && weapon.is_some_and(|w| w.template.damage_type == DamageType::Disarm)
        && (is_kind(sim, victim, KindOfType::Mine)
            || is_kind(sim, victim, KindOfType::BoobyTrap)
            || is_kind(sim, victim, KindOfType::Demotrap))
}

/// The state's own victim is gone.
fn forget_victim(sim: &mut dyn SimApi, ctx: ModuleContext) {
    // `notifyVictimIsDead` is only for jets.
    ai_update::set_current_victim(sim, ctx, None);
}

fn set_current_victim(sim: &mut dyn SimApi, ctx: ModuleContext, victim: ObjectSlot) {
    let id = id_of(sim, victim);
    ai_update::set_current_victim(sim, ctx, Some(id));
}

fn approach_update_internal(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    let owner = ctx.owner_slot;
    if goal_destroyed(sim, ctx, host) {
        forget_victim(sim, ctx);
        return StateReturnType::Failure;
    }
    let stop = Bool::from(ai(sim, ctx).is_attack_path == FALSE);
    approach(sim, ctx, host, state).stop_if_in_range = stop;
    let weapon = current_weapon(sim, owner);
    let key = move_key(host, state);

    if let Some(victim) = goal_object(sim, ctx, host) {
        if !is_human(sim, owner) && ignores_aircraft(sim, ctx, victim) {
            return StateReturnType::Failure;
        }
        if is_hidden(sim, victim) && !is_disarming(sim, weapon.as_ref(), victim) {
            return StateReturnType::Failure;
        }
        set_current_victim(sim, ctx, victim);
        if let Some(weapon) = &weapon {
            if weapon.template.is_contact_weapon() != FALSE
                && reaches_object(sim, weapon, owner, victim)
            {
                return StateReturnType::Success;
            }
            if stop != FALSE && reaches_object(sim, weapon, owner, victim) {
                let blocked = on_ground(sim, ctx)
                    && !significantly_above_terrain(sim, victim)
                    && view_blocked(sim, owner, victim);
                if !blocked {
                    return StateReturnType::Success;
                }
            }
        }
        if !approach_compute_path(sim, ctx, host, state) {
            return StateReturnType::Success;
        }
        // A failure would leave the whole attack.
        match movement::internal_update(sim, ctx, key) {
            StateReturnType::Continue => StateReturnType::Continue,
            _ => StateReturnType::Success,
        }
    } else {
        let goal = approach(sim, ctx, host, state).base.goal_position;
        if stop != FALSE
            && weapon
                .as_ref()
                .is_some_and(|w| reaches_position(sim, w, owner, &goal))
        {
            let blocked = on_ground(sim, ctx) && {
                let from = position(sim, owner);
                sim.attack_view_blocked(owner, &from, None, &goal)
            };
            if !blocked {
                return StateReturnType::Success;
            }
        }
        if !approach_compute_path(sim, ctx, host, state) {
            return StateReturnType::Failure;
        }
        movement::internal_update(sim, ctx, key)
    }
}

fn approach_update(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    let owner = ctx.owner_slot;
    let mut code = approach_update_internal(sim, ctx, host, state);
    let m = machine(sim, ctx, host);
    let (follow, object, force) = (m.follow, m.attacking_object, m.force_attacking);
    if follow != FALSE && object != FALSE {
        if let Some(victim) = goal_object(sim, ctx, host) {
            if is_mobile(sim, owner) && !is_kind(sim, victim, KindOfType::Immobile) {
                if code != StateReturnType::Continue {
                    approach(sim, ctx, host, state).is_initial_approach = FALSE;
                }
                code = StateReturnType::Continue;
            }
        }
    }
    if approach(sim, ctx, host, state).is_initial_approach != FALSE {
        aim_turret_at_mood_target(sim, ctx, force);
    }
    code
}

fn set_precise_z(sim: &mut dyn SimApi, ctx: ModuleContext, precise: Bool) {
    if let Some(locomotor) = ai(sim, ctx).cur_locomotor_mut() {
        locomotor.set_use_precise_z_pos(precise);
    }
}

fn approach_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, state: StateId) {
    let owner = ctx.owner_slot;
    movement::internal_on_exit(sim, ctx, move_key(host, state));
    ai(sim, ctx).ignore_obstacle_id = ObjectId::INVALID;
    if is_kind(sim, owner, KindOfType::Projectile) {
        set_precise_z(sim, ctx, FALSE);
    }
    if on_ground(sim, ctx) {
        let goal = approach(sim, ctx, host, state).base.goal_position;
        let pos = position(sim, owner);
        let (dx, dy) = (goal.x - pos.x, goal.y - pos.y);
        // The reference compares in double precision.
        if f64::from(dx * dx + dy * dy)
            < f64::from(PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F) * 0.125
        {
            sim.set_object_position(owner, &goal);
        }
    }
    approach(sim, ctx, host, state).is_initial_approach = FALSE;
}

fn pursue_on_enter(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    let owner = ctx.owner_slot;
    // Projectiles go straight to the approach.
    if is_kind(sim, owner, KindOfType::Projectile)
        || goal_destroyed(sim, ctx, host)
        || machine(sim, ctx, host).attacking_object == FALSE
    {
        return StateReturnType::Success;
    }
    approach(sim, ctx, host, state).base.adjust_destinations = FALSE;
    if current_state(sim, ctx) != state_id::GUARD_RETALIATE
        && is_human(sim, owner)
        && ai(sim, ctx).last_command_source == CommandSourceType::FromAi
    {
        return StateReturnType::Success;
    }
    let data = approach(sim, ctx, host, state);
    data.prev_victim_pos = Coord3D::default();
    data.approach_timestamp = 0u32.wrapping_sub(MIN_RECOMPUTE_TIME);
    let Some(victim) = goal_object(sim, ctx, host) else {
        return StateReturnType::Success;
    };
    let Some(weapon) = current_weapon(sim, owner) else {
        return StateReturnType::Failure;
    };
    if !can_pursue(sim, ctx, &weapon, victim) {
        return StateReturnType::Success;
    }
    // Only a turret can fire on the run.
    let Some(turret) = cur_turret(sim, ctx) else {
        return StateReturnType::Success;
    };
    let force = machine(sim, ctx, host).force_attacking;
    turret_target_object(sim, ctx, turret, Some(victim), force);
    if !pursue_compute_path(sim, ctx, host, state) {
        return StateReturnType::Success;
    }
    movement::internal_on_enter(sim, ctx, move_key(host, state))
}

fn pursue_update_internal(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    let owner = ctx.owner_slot;
    if goal_destroyed(sim, ctx, host) {
        forget_victim(sim, ctx);
        return StateReturnType::Failure;
    }
    approach(sim, ctx, host, state).stop_if_in_range = FALSE;
    let Some(victim) = goal_object(sim, ctx, host) else {
        return StateReturnType::Failure;
    };
    if is_hidden(sim, victim) {
        return StateReturnType::Failure;
    }
    set_current_victim(sim, ctx, victim);
    if !pursue_compute_path(sim, ctx, host, state) {
        return StateReturnType::Failure;
    }
    let code = movement::internal_update(sim, ctx, move_key(host, state));
    if code != StateReturnType::Continue {
        return StateReturnType::Success;
    }
    let Some(weapon) = current_weapon(sim, owner) else {
        return StateReturnType::Failure;
    };
    let Some(turret) = cur_turret(sim, ctx) else {
        return StateReturnType::Success;
    };
    let blocked = on_ground(sim, ctx)
        && !significantly_above_terrain(sim, victim)
        && view_blocked(sim, owner, victim);
    if !blocked
        && physics_update::physics(sim, victim).is_some()
        && reaches_object(sim, &weapon, owner, victim)
    {
        let force = machine(sim, ctx, host).force_attacking;
        turret_target_object(sim, ctx, turret, Some(victim), force);
        approach(sim, ctx, host, state).is_initial_approach = FALSE;
        let mut speed = physics_update::forward_speed_2d(sim, victim);
        let (from, to) = (position(sim, owner), position(sim, victim));
        if weapon_queries::is_goal_pos_within_attack_range(
            &weapon,
            sim,
            owner,
            &from,
            Some(victim),
            &to,
        ) != FALSE
        {
            speed *= 0.95;
        }
        if can_crush_or_squish(sim, owner, victim) {
            speed = FAST_AS_POSSIBLE;
        }
        ai(sim, ctx).desired_speed = speed;
    } else {
        ai(sim, ctx).desired_speed = FAST_AS_POSSIBLE;
    }
    code
}

fn pursue_update(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    let code = pursue_update_internal(sim, ctx, host, state);
    if approach(sim, ctx, host, state).is_initial_approach != FALSE {
        let force = machine(sim, ctx, host).force_attacking;
        aim_turret_at_mood_target(sim, ctx, force);
    }
    code
}

fn pursue_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, state: StateId) {
    movement::internal_on_exit(sim, ctx, move_key(host, state));
    approach(sim, ctx, host, state).is_initial_approach = FALSE;
}

// -----------------------------------------------------------------------------
// `AIAttackAimAtTargetState`
// -----------------------------------------------------------------------------

fn aim_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> StateReturnType {
    let owner = ctx.owner_slot;
    let weapon = current_weapon(sim, owner);
    let victim = goal_object(sim, ctx, host);
    let target = machine(sim, ctx, host).goal_position;
    let turns = ai(sim, ctx)
        .cur_locomotor()
        .is_some_and(|l| l.get_min_speed() == 0.0);
    machine(sim, ctx, host).can_turn_in_place = Bool::from(turns);

    let Some(weapon) = weapon else {
        return StateReturnType::Failure;
    };
    let container = sim.object(owner).and_then(|o| o.contained_by);
    let enclosed =
        container.is_some_and(|c| sim.contain_is_enclosing_container_for(c, owner) != FALSE);
    let in_range = if enclosed {
        let victim_id = victim.map(|v| id_of(sim, v));
        let at = victim.map_or(target, |v| position(sim, v));
        ai_seams(sim).attempt_best_fire_point(ctx.owner_id, victim_id, &at) != FALSE
    } else if let Some(victim) = victim {
        reaches_object(sim, &weapon, owner, victim)
    } else {
        reaches_position(sim, &weapon, owner, &target)
    };
    // `addTargeter` on the victim's AI is only for jets.

    let m = machine(sim, ctx, host);
    let (object, force) = (m.attacking_object != FALSE, m.force_attacking);
    let aim_turret = |sim: &mut dyn SimApi, turret| {
        if object {
            turret_target_object(sim, ctx, turret, victim, force);
        } else {
            turret_target_position(sim, ctx, turret, &target);
        }
    };
    if ai(sim, ctx).data.turrets_linked != FALSE {
        for turret in 0..MAX_TURRETS {
            aim_turret(sim, turret);
        }
    } else if let Some(turret) = cur_turret(sim, ctx) {
        aim_turret(sim, turret);
    } else if weapon.template.is_contact_weapon() != FALSE && in_range {
        // Nothing but a jet's AI holds off the aim.
        return StateReturnType::Success;
    }
    machine(sim, ctx, host).set_locomotor = FALSE;
    sim.set_status(owner, ObjectStatusType::IsAimingWeapon);
    StateReturnType::Continue
}

fn aim_update(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> StateReturnType {
    let owner = ctx.owner_slot;
    if sim
        .object(owner)
        .map_or(true, |o| o.weapon_set.has_any_weapon() == FALSE)
    {
        return StateReturnType::Failure;
    }
    let victim = goal_object(sim, ctx, host);
    let m = machine(sim, ctx, host);
    let (object, force, turns, target) = (
        m.attacking_object != FALSE,
        m.force_attacking,
        m.can_turn_in_place != FALSE,
        m.goal_position,
    );
    if object && victim.map_or(true, |v| is_effectively_dead(sim, v)) {
        return StateReturnType::Failure;
    }
    if let Some(turret) = cur_turret(sim, ctx) {
        if object {
            turret_target_object(sim, ctx, turret, victim, force);
        } else {
            turret_target_position(sim, ctx, turret, &target);
        }
        if ai_seams(sim).turret_turn_rate(ctx.owner_id, turret) != 0.0 {
            return StateReturnType::Continue;
        }
    }

    // What the weapon is pointed at.
    let at = match victim {
        Some(v) if object => position(sim, v),
        _ => target,
    };
    let relative = ai_update::relative_angle_2d(sim, owner, &at);
    let weapon = current_weapon(sim, owner);
    let mut aim_delta = weapon.as_ref().map_or(0.0, |w| w.template.aim_delta);
    if aim_delta < REL_THRESH {
        aim_delta = REL_THRESH;
    }
    if turns {
        if gm_fabsf(relative) > aim_delta {
            let desired = sim.object(owner).map_or(0.0, |o| o.thing.get_orientation()) + relative;
            ai(sim, ctx).set_goal_orientation(desired);
            machine(sim, ctx, host).set_locomotor = TRUE;
        }
    } else {
        ai(sim, ctx).set_goal_position_explicit(&at);
    }
    if gm_fabsf(relative) < aim_delta {
        return StateReturnType::Success;
    }

    if sim
        .object(owner)
        .is_some_and(|o| o.is_disabled_by_type(DisabledType::Held) != FALSE)
    {
        let in_range = current_weapon(sim, owner).is_some_and(|w| match victim {
            Some(v) if object => reaches_object(sim, &w, owner, v),
            _ if object => false,
            _ => reaches_position(sim, &w, owner, &target),
        });
        if !in_range {
            return StateReturnType::Failure;
        }
    }
    StateReturnType::Continue
}

fn aim_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) {
    let m = machine(sim, ctx, host);
    if m.can_turn_in_place != FALSE && m.set_locomotor != FALSE {
        ai_update::set_locomotor_goal_none(sim, ctx);
    }
    sim.clear_status(ctx.owner_slot, ObjectStatusType::IsAimingWeapon);
}

// -----------------------------------------------------------------------------
// `AIAttackFireWeaponState`
// -----------------------------------------------------------------------------

fn fire_on_enter(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> StateReturnType {
    let owner = ctx.owner_slot;
    let adjust = ai_update::mood_matrix_action_adjustment(sim, ctx, MoodMatrixAction::Attack);
    if adjust & MAA_ACTION_OK == 0 {
        return StateReturnType::Failure;
    }
    let victim = goal_object(sim, ctx, host);
    if let Some(victim) = victim {
        let id = id_of(sim, victim);
        ai_seams(sim).offer_team_target(ctx.owner_id, id);
    }
    sim.set_status(owner, ObjectStatusType::IsFiringWeapon);
    weapon_actions::pre_fire_current_weapon(sim, owner, victim);
    StateReturnType::Continue
}

/// `setFiringConditionForCurrentWeapon`.
fn set_firing_condition(sim: &mut dyn SimApi, owner: ObjectSlot) {
    let Some(slot) = sim
        .object(owner)
        .map(|o| o.weapon_set.get_cur_weapon_slot())
    else {
        return;
    };
    let clear = all_weapon_fire_flags(slot);
    let set = WeaponSet::get_model_condition_for_weapon_slot(slot, WeaponSetConditionType::Firing);
    sim.clear_and_set_model_condition_flags(owner, &clear, &set);
}

fn fire_update(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> StateReturnType {
    let owner = ctx.owner_slot;
    let victim = goal_object(sim, ctx, host);
    let object = machine(sim, ctx, host).attacking_object != FALSE;
    if object && victim.map_or(true, |v| is_effectively_dead(sim, v)) {
        return StateReturnType::Failure;
    }
    let now = sim.frame();
    let Some(weapon) = current_weapon_mut(sim, owner) else {
        return StateReturnType::Failure;
    };
    match weapon.get_status(now) {
        WeaponStatus::PreAttack => return StateReturnType::Continue,
        WeaponStatus::ReadyToFire => {}
        _ => return StateReturnType::Failure,
    }
    // Every slot is fine for an `AIAttackState`.
    let continue_range = weapon.template.continue_attack_range;

    set_firing_condition(sim, owner);
    let target = machine(sim, ctx, host).goal_position;
    if object {
        if let Some(victim) = victim {
            weapon_actions::fire_current_weapon_at_object(sim, owner, victim);
        }
        let current = ai(sim, ctx).current_victim_id;
        let current = sim.find_object(current);
        if victim != current {
            set_goal_object(sim, ctx, host, current);
        }
        sim.clear_status(owner, ObjectStatusType::IgnoringStealth);
        if continue_range > 0.0 {
            if let Some(victim) = victim.filter(|&v| is_finished_off(sim, v)) {
                take_next_victim(sim, ctx, host, victim, continue_range);
            }
        }
    } else {
        if ai(sim, ctx).data.turrets_linked != FALSE {
            for slot in [
                WeaponSlotType::Primary,
                WeaponSlotType::Secondary,
                WeaponSlotType::Tertiary,
            ] {
                weapon_actions::fire_linked_weapon_at_position(sim, owner, slot, &target);
            }
        } else {
            weapon_actions::fire_current_weapon_at_position(sim, owner, &target);
        }
        sim.clear_status(owner, ObjectStatusType::IgnoringStealth);
    }
    StateReturnType::Success
}

/// Whether the victim is done with, so a continuing weapon looks for the next.
fn is_finished_off(sim: &dyn SimApi, victim: ObjectSlot) -> bool {
    sim.object(victim)
        .map_or(true, |o| o.is_destroyed() || o.is_effectively_dead() != FALSE)
        || is_kind(sim, victim, KindOfType::Mine)
            && has_status(sim, victim, ObjectStatusType::Masked)
}

/// A weapon with a continue range goes on to the next of the same player near
/// where the first victim stood.
fn take_next_victim(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    victim: ObjectSlot,
    range: Real,
) {
    let owner = ctx.owner_slot;
    let origin = attack_state(sim, ctx, host).original_victim_pos;
    let source = ai(sim, ctx).last_command_source;
    let Some(player) = sim.object(victim).map(|o| sim.player_index(o.team_id)) else {
        return;
    };
    let off_map = |sim: &dyn SimApi, slot| {
        sim.object(slot)
            .map_or(0, |o| o.private_status & PRIVATE_STATUS_OFF_MAP)
    };
    let mine = off_map(sim, owner);
    let found = {
        let sim: &dyn SimApi = sim;
        sim.closest_object_to_position(
            &origin,
            range,
            DistanceCalculationType::FromCenter2d,
            &mut |other| {
                let result =
                    sim.able_to_attack_specific_object(owner, ATTACK_NEW_TARGET, other, source);
                (result == CanAttackResult::Possible
                    || result == CanAttackResult::PossibleAfterMoving)
                    && sim
                        .object(other)
                        .is_some_and(|o| sim.player_index(o.team_id) == player)
                    && off_map(sim, other) == mine
            },
        )
    };
    if let Some(next) = found {
        set_goal_object(sim, ctx, host, Some(next));
        // `notifyNewVictimChosen`.
        set_host_goal_object(sim, ctx, host, next);
        set_goal_object(sim, ctx, host, Some(next));
    }
}

fn fire_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext) {
    let owner = ctx.owner_slot;
    sim.clear_status(owner, ObjectStatusType::IsFiringWeapon);
    sim.clear_status(owner, ObjectStatusType::IgnoringStealth);
    let now = sim.frame();
    if let Some(weapon) = current_weapon_mut(sim, owner) {
        if weapon.get_status(now) == WeaponStatus::PreAttack {
            weapon.set_pre_attack_finished_frame(0);
        }
    }
}

// -----------------------------------------------------------------------------
// The attack machine
// -----------------------------------------------------------------------------

fn sub_on_enter(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    match state {
        AIM_AT_TARGET => aim_on_enter(sim, ctx, host),
        FIRE_WEAPON => fire_on_enter(sim, ctx, host),
        APPROACH_TARGET => approach_on_enter(sim, ctx, host, state),
        _ => match machine(sim, ctx, host).chase {
            Chase::Pursue => pursue_on_enter(sim, ctx, host, state),
            Chase::Approach => approach_on_enter(sim, ctx, host, state),
            Chase::Continue => StateReturnType::Continue,
            Chase::Failure => StateReturnType::Failure,
        },
    }
}

fn sub_update(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    state: StateId,
) -> StateReturnType {
    match state {
        AIM_AT_TARGET => aim_update(sim, ctx, host),
        FIRE_WEAPON => fire_update(sim, ctx, host),
        APPROACH_TARGET => approach_update(sim, ctx, host, state),
        _ => match machine(sim, ctx, host).chase {
            Chase::Pursue => pursue_update(sim, ctx, host, state),
            Chase::Approach => approach_update(sim, ctx, host, state),
            Chase::Continue => StateReturnType::Continue,
            Chase::Failure => StateReturnType::Failure,
        },
    }
}

fn sub_on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, state: StateId) {
    match state {
        AIM_AT_TARGET => aim_on_exit(sim, ctx, host),
        FIRE_WEAPON => fire_on_exit(sim, ctx),
        APPROACH_TARGET => approach_on_exit(sim, ctx, host, state),
        _ => match machine(sim, ctx, host).chase {
            Chase::Pursue => pursue_on_exit(sim, ctx, host, state),
            Chase::Approach => approach_on_exit(sim, ctx, host, state),
            Chase::Continue | Chase::Failure => {}
        },
    }
}

/// `internalSetState`.
fn internal_set_state(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    new_id: StateId,
) -> StateReturnType {
    let m = machine(sim, ctx, host);
    let generation = m.generation;
    m.sleep_till = 0;
    let mut next = INVALID_STATE_ID;
    if new_id != MACHINE_DONE_STATE_ID {
        next = new_id;
        // The reference throws for a state the machine does not define.
        if !m.defines(next) {
            return StateReturnType::Failure;
        }
    }
    let old = m.current;
    if old != INVALID_STATE_ID {
        sub_on_exit(sim, ctx, host, old);
    }
    if !alive(sim, ctx, host, generation) {
        return StateReturnType::Failure;
    }
    machine(sim, ctx, host).current = next;
    if next == INVALID_STATE_ID {
        return StateReturnType::Continue;
    }
    let status = sub_on_enter(sim, ctx, host, next);
    finish_state_call(sim, ctx, host, generation, next, status)
}

/// What follows `onEnter` and `update` alike.
fn finish_state_call(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    generation: u32,
    ran: StateId,
    mut status: StateReturnType,
) -> StateReturnType {
    if !alive(sim, ctx, host, generation) {
        return StateReturnType::Failure;
    }
    let current = machine(sim, ctx, host).current;
    if current == INVALID_STATE_ID {
        return StateReturnType::Failure;
    }
    if current != ran {
        status = StateReturnType::Continue;
    }
    if let StateReturnType::Sleep(frames) = status {
        let now = sim.frame();
        machine(sim, ctx, host).sleep_till = now.wrapping_add(frames);
    }
    check_transitions(sim, ctx, host, status, current)
}

/// `State::friend_checkForTransitions` and its sleeping form. The depth count is
/// shared with the AI's machine, as the reference's is by every machine.
fn check_transitions(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    status: StateReturnType,
    state: StateId,
) -> StateReturnType {
    let unit = ai(sim, ctx);
    unit.machine.transition_depth = unit.machine.transition_depth.wrapping_add(1);
    let result = if unit.machine.transition_depth >= MAX_TRANSITION_DEPTH {
        StateReturnType::Failure
    } else {
        match status {
            StateReturnType::Success | StateReturnType::Failure => {
                let (success, failure) = machine(sim, ctx, host).links(state);
                let next = if status == StateReturnType::Success {
                    success
                } else {
                    failure
                };
                if next == EXIT_MACHINE_WITH_FAILURE {
                    let _ = internal_set_state(sim, ctx, host, MACHINE_DONE_STATE_ID);
                    StateReturnType::Failure
                } else {
                    internal_set_state(sim, ctx, host, next)
                }
            }
            StateReturnType::Continue | StateReturnType::Sleep(_) => {
                match condition_target(sim, ctx, host, state) {
                    Some(EXIT_MACHINE_WITH_FAILURE) => StateReturnType::Failure,
                    Some(next) => internal_set_state(sim, ctx, host, next),
                    None => status,
                }
            }
        }
    };
    let unit = ai(sim, ctx);
    unit.machine.transition_depth = unit.machine.transition_depth.wrapping_sub(1);
    result
}

/// `updateStateMachine` of the attack machine.
fn update_machine(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> StateReturnType {
    let now = sim.frame();
    let m = machine(sim, ctx, host);
    let (generation, current) = (m.generation, m.current);
    if m.sleep_till != 0 && now < m.sleep_till {
        if current == INVALID_STATE_ID {
            return StateReturnType::Failure;
        }
        let left = StateReturnType::Sleep(m.sleep_till.wrapping_sub(now));
        return check_transitions(sim, ctx, host, left, current);
    }
    m.sleep_till = 0;
    if current == INVALID_STATE_ID {
        return StateReturnType::Failure;
    }
    let status = sub_update(sim, ctx, host, current);
    finish_state_call(sim, ctx, host, generation, current, status)
}

/// Deletes the attack machine; its state is left as its destructor leaves it.
fn delete_machine(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) {
    let Some((generation, current)) = attack_state(sim, ctx, host)
        .machine
        .as_ref()
        .map(|m| (m.generation, m.current))
    else {
        return;
    };
    if current != INVALID_STATE_ID {
        sub_on_exit(sim, ctx, host, current);
    }
    if alive(sim, ctx, host, generation) {
        attack_state(sim, ctx, host).machine = None;
    }
}

// -----------------------------------------------------------------------------
// `AIAttackState`
// -----------------------------------------------------------------------------

/// The attack machine the constructor builds for this unit.
fn new_machine(sim: &mut dyn SimApi, ctx: ModuleContext, id: StateId) -> Box<AttackMachine> {
    let owner = ctx.owner_slot;
    let attacking_object = id != state_id::ATTACK_POSITION;
    let attacks = &mut ai(sim, ctx).machine.attacks;
    attacks.generations = attacks.generations.wrapping_add(1);
    let generation = attacks.generations;
    let (chase, has_approach) = if is_kind(sim, owner, KindOfType::Immobile) {
        (Chase::Failure, false)
    } else if is_kind(sim, owner, KindOfType::PortableStructure)
        && is_kind(sim, owner, KindOfType::CanAttack)
    {
        (Chase::Continue, false)
    } else if attacking_object {
        (Chase::Pursue, true)
    } else {
        (Chase::Approach, true)
    };
    Box::new(AttackMachine {
        generation,
        chase,
        has_approach,
        follow: Bool::from(id == state_id::ATTACK_AND_FOLLOW_OBJECT),
        attacking_object: Bool::from(attacking_object),
        force_attacking: Bool::from(id == state_id::FORCE_ATTACK_OBJECT),
        // The pursue constructor starts with these clear too.
        ..AttackMachine::default()
    })
}

fn is_out_of_ammo(sim: &mut dyn SimApi, owner: ObjectSlot) -> bool {
    let now = sim.frame();
    let out = sim
        .object_mut(owner)
        .is_some_and(|o| o.weapon_set.is_out_of_ammo(now) != FALSE);
    out && !is_kind(sim, owner, KindOfType::Projectile)
}

/// `chooseWeapon`.
fn choose_weapon(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> bool {
    let owner = ctx.owner_slot;
    let victim = host_goal_id(sim, ctx, host);
    let victim = sim.find_object(victim);
    let object = attack_state(sim, ctx, host)
        .machine
        .as_ref()
        .is_some_and(|m| m.attacking_object != FALSE);
    if object && victim.is_none() {
        return false;
    }
    let source = ai(sim, ctx).last_command_source;
    let found = weapon_set::choose_best_weapon_for_target(
        sim,
        owner,
        victim,
        WeaponChoiceCriteria::PreferMostDamage,
        source,
    );
    sim.trigger_weapon_status(owner);
    found
}

fn current_slot_and_lock(sim: &dyn SimApi, owner: ObjectSlot) -> (Option<WeaponSlotType>, bool) {
    sim.object(owner).map_or((None, false), |o| {
        let set = &o.weapon_set;
        let slot = set.get_cur_weapon_slot();
        let held =
            set.has_any_weapon() != FALSE && set.get_weapon_in_weapon_slot(slot).is_some();
        (held.then_some(slot), set.is_cur_weapon_locked() != FALSE)
    })
}

/// `AIAttackState::onEnter` of the state `id` in `host`.
pub fn on_enter(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    host: Host,
    id: StateId,
) -> StateReturnType {
    let owner = ctx.owner_slot;
    let adjust = ai_update::mood_matrix_action_adjustment(sim, ctx, MoodMatrixAction::Attack);
    if adjust & MAA_ACTION_OK == 0 {
        return StateReturnType::Success;
    }
    if has_status(sim, owner, ObjectStatusType::UnderConstruction) || is_out_of_ammo(sim, owner) {
        return StateReturnType::Failure;
    }
    let machine_for_unit = new_machine(sim, ctx, id);
    let attacking_object = machine_for_unit.attacking_object != FALSE;
    attack_state(sim, ctx, host).machine = Some(machine_for_unit);

    if attacking_object {
        let victim = host_goal_id(sim, ctx, host);
        let victim = sim
            .find_object(victim)
            .filter(|&v| !is_effectively_dead(sim, v));
        let Some(victim) = victim else {
            return StateReturnType::Failure;
        };
        let team = sim.object(victim).map(|o| o.team_id);
        let pos = position(sim, victim);
        let state = attack_state(sim, ctx, host);
        state.victim_team = team;
        state.original_victim_pos = pos;
        set_goal_object(sim, ctx, host, Some(victim));
    } else {
        let pos = host_goal_position(sim, ctx, host);
        machine(sim, ctx, host).goal_position = pos;
        attack_state(sim, ctx, host).original_victim_pos = pos;
    }
    if !choose_weapon(sim, ctx, host) {
        return StateReturnType::Failure;
    }
    let mut continues = false;
    if let Some(weapon) = current_weapon_mut(sim, owner) {
        weapon.max_shot_count = NO_MAX_SHOTS_LIMIT;
        continues = weapon.template.continue_attack_range > 0.0;
    }
    if continues {
        sim.set_status(owner, ObjectStatusType::IgnoringStealth);
    }
    let (slot, locked) = current_slot_and_lock(sim, owner);
    attack_state(sim, ctx, host).locked_weapon_on_enter = slot.filter(|_| locked);

    // `initDefaultState`.
    machine(sim, ctx, host).default_state_inited = TRUE;
    let result = internal_set_state(sim, ctx, host, AIM_AT_TARGET);
    if result == StateReturnType::Continue {
        sim.set_status(owner, ObjectStatusType::IsAttacking);
        sim.set_model_condition_state(owner, ModelConditionFlagType::Attacking);
    }
    result
}

/// Gives up on a victim that may no longer be attacked.
fn release_victim(sim: &mut dyn SimApi, ctx: ModuleContext, victim: ObjectSlot) {
    ai_update::friend_set_goal_object(sim, ctx, None);
    let id = id_of(sim, victim);
    ai_seams(sim).drop_team_target(ctx.owner_id, id);
}

/// Whether a victim that changed sides is still to be attacked.
fn still_attackable(sim: &mut dyn SimApi, ctx: ModuleContext, victim: ObjectSlot) -> bool {
    let owner = ctx.owner_slot;
    let empty_building = !has_status(sim, victim, ObjectStatusType::CanAttack)
        && sim.object_has_contain(victim) != FALSE
        && sim.contain_is_garrisonable(victim) != FALSE
        && sim.contain_count(victim) == 0
        && sim.player_relationship(owner, victim) == Relationship::Neutral;
    !empty_building && sim.player_relationship(owner, victim) == Relationship::Enemies
}

/// `AIAttackState::update`.
pub fn update(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host) -> StateReturnType {
    let owner = ctx.owner_slot;
    if is_out_of_ammo(sim, owner) {
        return StateReturnType::Failure;
    }
    let Some(object) = attack_state(sim, ctx, host)
        .machine
        .as_ref()
        .map(|m| m.attacking_object != FALSE)
    else {
        return StateReturnType::Failure;
    };
    if object {
        let victim = host_goal_id(sim, ctx, host);
        let victim = sim
            .find_object(victim)
            .filter(|&v| !is_effectively_dead(sim, v));
        let Some(victim) = victim else {
            return StateReturnType::Success;
        };
        set_current_victim(sim, ctx, victim);
        let team = sim.object(victim).map(|o| o.team_id);
        if team != attack_state(sim, ctx, host).victim_team
            && !still_attackable(sim, ctx, victim)
        {
            release_victim(sim, ctx, victim);
            return StateReturnType::Failure;
        }
        let victim_id = id_of(sim, victim);
        if machine(sim, ctx, host).goal_object_id != victim_id {
            set_goal_object(sim, ctx, host, Some(victim));
        }
    }
    if !choose_weapon(sim, ctx, host) {
        return StateReturnType::Failure;
    }
    let (slot, _) = current_slot_and_lock(sim, owner);
    let locked = attack_state(sim, ctx, host).locked_weapon_on_enter;
    if locked.is_some() && locked != slot {
        return StateReturnType::Failure;
    }
    if current_weapon_mut(sim, owner).map_or(true, |w| w.max_shot_count <= 0) {
        return StateReturnType::Failure;
    }
    // A sleeping sub machine is still called every frame.
    match update_machine(sim, ctx, host) {
        StateReturnType::Sleep(_) => StateReturnType::Continue,
        status => status,
    }
}

/// `AIAttackState::onExit`.
pub fn on_exit(sim: &mut dyn SimApi, ctx: ModuleContext, host: Host, _status: StateExitType) {
    let owner = ctx.owner_slot;
    delete_machine(sim, ctx, host);
    for status in [
        ObjectStatusType::IsFiringWeapon,
        ObjectStatusType::IsAimingWeapon,
        ObjectStatusType::IsAttacking,
        ObjectStatusType::IgnoringStealth,
    ] {
        sim.clear_status(owner, status);
    }
    sim.clear_model_condition_state(owner, ModelConditionFlagType::Attacking);
    if let Some(o) = sim.object_mut(owner) {
        o.weapon_set.clear_leech_range_mode_for_all_weapons();
    }
    ai_update::set_current_victim(sim, ctx, None);
    for turret in 0..MAX_TURRETS {
        turret_target_object(sim, ctx, turret, None, FALSE);
    }
    ai_update::friend_set_goal_object(sim, ctx, None);
}
