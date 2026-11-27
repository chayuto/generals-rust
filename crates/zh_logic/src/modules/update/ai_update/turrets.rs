//! The turrets of an AI module (`m_turretAI`): their construction, the calls the
//! AI update, the attack states and the weapon code make on them, and what a
//! [`TurretAI`] asks of the game. Packets `PKT-145` and the weapon half of
//! `PKT-301`; golden test `tests/golden_s14_attack_states.rs`.
//!
//! A turret runs on a copy while the module keeps the other, and the two are
//! swapped around every call that can come back to this turret
//! ([`TurretWorldAccess::lend_turret`]), so what is called from a shot or from a
//! target search sees and changes the running turret.

use super::{ai, mood_matrix_action_adjustment, next_mood_target, AiUpdate, MoodMatrixAction};
use super::{relative_angle_2d, reset_next_mood_check_time, MAX_TURRETS};
use crate::ai::ai_states::attack;
use crate::ai::mood::MAA_AFFECT_RANGE_IGNORE_ALL;
use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::turret_ai::{
    FireStateStep, TurretAI, TurretStateType, TurretTargetType, TurretWorldAccess, WhichTurretType,
};
use crate::modules::module::CommandSourceType;
use crate::object::weapon::Weapon;
use crate::object::weapon_queries;
use crate::object::weapon_set::{
    self, CanAttackResult, WeaponChoiceCriteria, WeaponSlotType, ATTACK_CONTINUED_TARGET,
    ATTACK_CONTINUED_TARGET_FORCED,
};
use crate::rules::kind_of::KindOfType;
use crate::rules::model_state::ModelConditionFlagType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use zh_common::core::state_machine::StateReturnType;
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, ObjectId, Real, TeamId, UnsignedInt, FALSE, TRUE};

/// The turrets of one module.
pub type Turrets = [Option<Box<TurretAI>>; MAX_TURRETS];

fn which(turret: usize) -> WhichTurretType {
    WhichTurretType::from_i32(turret as Int)
}

/// `getCurrentWeapon(&wslot)`.
fn current_slot(sim: &dyn SimView, owner: ObjectSlot) -> Option<WeaponSlotType> {
    let set = &sim.object(owner)?.weapon_set;
    if set.has_any_weapon() == FALSE {
        return None;
    }
    let slot = set.get_cur_weapon_slot();
    set.get_weapon_in_weapon_slot(slot).map(|_| slot)
}

fn weapon_in(sim: &dyn SimView, owner: ObjectSlot, slot: WeaponSlotType) -> Option<Weapon> {
    sim.object(owner)?
        .weapon_set
        .get_weapon_in_weapon_slot(slot)
        .cloned()
}

// -----------------------------------------------------------------------------
// The calls on the turrets
// -----------------------------------------------------------------------------

/// The turrets the constructor of `AIUpdateInterface` makes. The object has no
/// AI yet while it does, so a new idle turret only draws its first scan time.
pub(super) fn construct(state: &mut AiUpdate, sim: &mut dyn SimApi, ctx: ModuleContext) {
    for turret in 0..MAX_TURRETS {
        let Some(data) = state.data.turrets[turret].clone() else {
            continue;
        };
        let mut world = World {
            sim,
            ctx,
            turret,
            constructing: true,
            linked: state.data.turrets_linked,
        };
        // A turret without weapon slots makes the reference throw.
        state.turrets[turret] = TurretAI::new(ctx.owner_id, data, which(turret), &mut world)
            .ok()
            .map(Box::new);
    }
}

/// Runs `f` on a turret of the unit, or nothing when there is none.
fn run<R>(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    turret: usize,
    f: impl FnOnce(&mut TurretAI, &mut dyn TurretWorldAccess) -> R,
) -> Option<R> {
    let state = ai(sim, ctx);
    let linked = state.data.turrets_linked;
    let mut live = state.turrets.get(turret)?.as_deref()?.clone();
    let mut world = World {
        sim,
        ctx,
        turret,
        constructing: false,
        linked,
    };
    let result = f(&mut live, &mut world);
    if let Some(held) = ai(sim, ctx).turrets[turret].as_deref_mut() {
        *held = live;
    }
    Some(result)
}

/// `updateTurretAI`.
pub(super) fn update(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    turret: usize,
) -> Option<UnsignedInt> {
    run(sim, ctx, turret, |t, w| t.update_turret_ai(w).0)
}

/// `setTurretTargetObject`.
pub fn set_target_object(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    turret: usize,
    victim: Option<ObjectId>,
    force: Bool,
) {
    let victim = victim.unwrap_or(ObjectId::INVALID);
    run(sim, ctx, turret, |t, w| {
        t.set_turret_target_object(victim, force, w)
    });
}

/// `setTurretTargetPosition`.
pub fn set_target_position(
    sim: &mut dyn SimApi,
    ctx: ModuleContext,
    turret: usize,
    pos: Option<&Coord3D>,
) {
    run(sim, ctx, turret, |t, w| {
        t.set_turret_target_position(pos, w)
    });
}

/// `setTurretEnabled`.
pub(super) fn set_enabled(sim: &mut dyn SimApi, ctx: ModuleContext, turret: usize, enabled: Bool) {
    run(sim, ctx, turret, |t, w| t.set_turret_enabled(enabled, w));
}

/// `recenterTurret`.
pub(super) fn recenter(sim: &mut dyn SimApi, ctx: ModuleContext, turret: usize) {
    run(sim, ctx, turret, |t, w| t.recenter_turret(w));
}

/// `getWhichTurretForCurWeapon`.
pub fn which_for_cur_weapon(
    sim: &dyn SimView,
    state: &AiUpdate,
    owner: ObjectSlot,
) -> Option<usize> {
    let slot = current_slot(sim, owner)?;
    which_for_weapon_slot(state, slot)
}

/// `getWhichTurretForCurWeapon` of the running module.
pub fn cur_turret(sim: &mut dyn SimApi, ctx: ModuleContext) -> Option<usize> {
    let slot = current_slot(&*sim, ctx.owner_slot)?;
    which_for_weapon_slot(ai(sim, ctx), slot)
}

/// `getWhichTurretForWeaponSlot`.
pub fn which_for_weapon_slot(state: &AiUpdate, slot: WeaponSlotType) -> Option<usize> {
    state.turrets.iter().position(|t| {
        t.as_ref()
            .is_some_and(|t| t.is_weapon_slot_on_turret(slot) != FALSE)
    })
}

/// The angle and pitch of the turret carrying a weapon slot of an object
/// (`getWhichTurretForWeaponSlot`).
pub fn rot_and_pitch_for_slot(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    slot: WeaponSlotType,
) -> Option<(Real, Real)> {
    let ms = sim.object(owner)?.ai_slot?;
    let state = sim.resident_state_mut(ms)?.downcast_mut::<AiUpdate>()?;
    let turret = state.turrets[which_for_weapon_slot(state, slot)?].as_ref()?;
    Some((turret.angle, turret.pitch))
}

/// `isTurretInNaturalPosition` of the turret, for an owner under construction or
/// not.
pub(super) fn is_in_natural_position(
    state: &AiUpdate,
    turret: usize,
    under_construction: bool,
) -> Bool {
    let Some(Some(t)) = state.turrets.get(turret) else {
        return FALSE;
    };
    Bool::from(
        under_construction
            || (t.data.natural_turret_angle == t.angle && t.data.natural_turret_pitch == t.pitch),
    )
}

/// `isWeaponSlotOnTurretAndAimingAtTarget` of the object. Asking forgets an
/// aimed-at target that has died.
pub fn weapon_slot_aiming_at(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    slot: WeaponSlotType,
    victim: ObjectId,
) -> bool {
    let Some(ms) = sim.object(owner).and_then(|o| o.ai_slot) else {
        return false;
    };
    let Some(turret) = sim
        .resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .and_then(|state| which_for_weapon_slot(state, slot).map(|t| (state, t)))
        .and_then(|(state, t)| state.turrets[t].as_deref().map(|held| (t, held)))
        .map(|(t, held)| {
            (
                t,
                held.sm.get_current_state(),
                held.target,
                held.sm.get_goal_object_id(),
            )
        })
    else {
        return false;
    };
    let (index, current, target, goal) = turret;
    if current != Some(TurretStateType::Aim) || target != TurretTargetType::Object {
        return false;
    }
    let live = sim
        .find_object(goal)
        .and_then(|s| sim.object(s))
        .is_some_and(|o| o.is_effectively_dead() == FALSE);
    if live {
        return goal == victim;
    }
    if let Some(held) = sim
        .resident_state_mut(ms)
        .and_then(|a| a.downcast_mut::<AiUpdate>())
        .and_then(|state| state.turrets[index].as_deref_mut())
    {
        held.drop_dead_target();
    }
    false
}

// -----------------------------------------------------------------------------
// What a turret asks
// -----------------------------------------------------------------------------

struct World<'a> {
    sim: &'a mut dyn SimApi,
    ctx: ModuleContext,
    turret: usize,
    /// Inside the module's constructor: the object has no AI yet.
    constructing: bool,
    linked: Bool,
}

impl World<'_> {
    fn owner(&self) -> ObjectSlot {
        self.ctx.owner_slot
    }

    fn slot_of(&self, id: ObjectId) -> Option<ObjectSlot> {
        self.sim.find_object(id)
    }

    fn swap(&mut self, turret: &mut TurretAI) {
        let index = self.turret;
        if let Some(held) = ai(self.sim, self.ctx).turrets[index].as_deref_mut() {
            std::mem::swap(held, turret);
        }
    }
}

impl TurretWorldAccess for World<'_> {
    fn current_frame(&self) -> UnsignedInt {
        self.sim.frame()
    }
    fn random_int(&mut self, lo: Int, hi: Int) -> Int {
        self.sim.rng_logic_int(lo, hi)
    }
    fn random_real(&mut self, lo: Real, hi: Real) -> Real {
        self.sim.rng_logic_real(lo, hi)
    }
    fn object_exists(&self, id: ObjectId) -> bool {
        self.slot_of(id).is_some()
    }
    fn is_object_effectively_dead(&self, id: ObjectId) -> bool {
        self.slot_of(id)
            .and_then(|s| self.sim.object(s))
            .map_or(true, |o| o.is_effectively_dead() != FALSE)
    }
    fn get_object_team(&self, id: ObjectId) -> Option<TeamId> {
        self.slot_of(id)
            .and_then(|s| self.sim.object(s))
            .map(|o| o.team_id)
    }
    fn get_object_position(&self, id: ObjectId) -> Option<Coord3D> {
        self.slot_of(id)
            .and_then(|s| self.sim.object(s))
            .map(|o| *o.thing.get_position())
    }
    fn is_object_kind_of(&self, id: ObjectId, kind: KindOfType) -> Bool {
        self.slot_of(id)
            .map_or(FALSE, |s| self.sim.object_is_kind_of(s, kind))
    }
    fn is_owner_under_construction(&self, _owner: ObjectId) -> Bool {
        Bool::from(
            self.sim.object(self.owner()).is_some_and(|o| {
                o.status.test(ObjectStatusType::UnderConstruction as i32) != FALSE
            }),
        )
    }
    fn set_model_condition_turret_rotate(&mut self, _owner: ObjectId, set: Bool) {
        let owner = self.owner();
        if set != FALSE {
            self.sim
                .set_model_condition_state(owner, ModelConditionFlagType::TurretRotate);
        } else {
            self.sim
                .clear_model_condition_state(owner, ModelConditionFlagType::TurretRotate);
        }
    }
    fn react_to_turret_change(
        &mut self,
        _owner: ObjectId,
        _turret: WhichTurretType,
        _old_angle: Real,
        _new_pitch: Real,
    ) {
        let owner = self.owner();
        self.sim.contain_react_to_turret_change(owner);
    }
    fn get_owner_current_weapon_slot(&self, _owner: ObjectId) -> Option<WeaponSlotType> {
        current_slot(&*self.sim, self.owner())
    }
    fn get_weapon_attack_range(&self, _owner: ObjectId, slot: WeaponSlotType) -> Real {
        let owner = self.owner();
        weapon_in(&*self.sim, owner, slot)
            .map_or(0.0, |w| weapon_queries::attack_range(&w, &*self.sim, owner))
    }
    fn is_weapon_in_range_of_object(
        &self,
        _owner: ObjectId,
        slot: WeaponSlotType,
        target: ObjectId,
    ) -> Bool {
        let owner = self.owner();
        match (weapon_in(&*self.sim, owner, slot), self.slot_of(target)) {
            (Some(w), Some(victim)) => {
                weapon_queries::is_within_attack_range(&w, &*self.sim, owner, victim)
            }
            _ => FALSE,
        }
    }
    fn is_weapon_in_range_of_pos(
        &self,
        _owner: ObjectId,
        slot: WeaponSlotType,
        pos: &Coord3D,
    ) -> Bool {
        let owner = self.owner();
        weapon_in(&*self.sim, owner, slot).map_or(FALSE, |w| {
            weapon_queries::is_within_attack_range_of_position(&w, &*self.sim, owner, pos)
        })
    }
    fn are_turrets_linked(&self, _owner: ObjectId) -> Bool {
        self.linked
    }
    fn get_owner_ai_goal_object(&mut self, _owner: ObjectId) -> ObjectId {
        let goal = ai(self.sim, self.ctx).machine.goal_object_id;
        match self.slot_of(goal) {
            Some(_) => goal,
            None => ObjectId::INVALID,
        }
    }
    fn is_owner_able_to_attack(&mut self, _owner: ObjectId) -> Bool {
        self.sim.object_is_able_to_attack(self.owner())
    }
    fn can_owner_attack_specific_object(
        &mut self,
        _owner: ObjectId,
        target: ObjectId,
        force_attacking: Bool,
    ) -> CanAttackResult {
        let source = ai(self.sim, self.ctx).last_command_source;
        let kind = if force_attacking != FALSE {
            ATTACK_CONTINUED_TARGET_FORCED
        } else {
            ATTACK_CONTINUED_TARGET
        };
        let owner = self.owner();
        match self.slot_of(target) {
            Some(victim) => self
                .sim
                .able_to_attack_specific_object(owner, kind, victim, source),
            None => CanAttackResult::NotPossible,
        }
    }
    // Only a jet's AI keeps targeters or holds off an aim.
    fn add_targeter(&mut self, _target: ObjectId, _targeter: ObjectId, _add: Bool) {}
    fn is_temporarily_preventing_aim_success(&self, _target: ObjectId) -> Bool {
        FALSE
    }
    fn get_bridge_closest_attack_point(&self, _owner: ObjectId, bridge: ObjectId) -> Coord3D {
        let owner = self.owner();
        self.slot_of(bridge).map_or_else(Coord3D::default, |b| {
            weapon_queries::closest_bridge_attack_point(&*self.sim, owner, b)
        })
    }
    fn get_relative_angle_2d(&self, _owner: ObjectId, pos: &Coord3D) -> Real {
        relative_angle_2d(&*self.sim, self.owner(), pos)
    }
    fn get_vector_to_center_3d(&self, _owner: ObjectId, pos: &Coord3D) -> Coord3D {
        // `FROM_CENTER_3D` measures from the object's position.
        let from = self
            .sim
            .object(self.owner())
            .map_or_else(Coord3D::default, |o| *o.thing.get_position());
        Coord3D::new(pos.x - from.x, pos.y - from.y, pos.z - from.z)
    }
    fn get_owner_max_height_above_position(&self, _owner: ObjectId) -> Real {
        self.sim
            .object(self.owner())
            .map_or(0.0, |o| o.geometry.get_max_height_above_position())
    }
    fn get_next_idle_mood_target_frame(&mut self, _owner: ObjectId) -> UnsignedInt {
        if self.constructing {
            return self.sim.frame();
        }
        ai(self.sim, self.ctx).next_mood_check_time
    }
    fn reset_next_mood_check_time(&mut self, _owner: ObjectId) {
        if !self.constructing {
            reset_next_mood_check_time(self.sim, self.ctx);
        }
    }
    fn get_turret_sync(&mut self, _owner: ObjectId) -> WhichTurretType {
        if self.constructing {
            return WhichTurretType::Invalid;
        }
        WhichTurretType::from_i32(ai(self.sim, self.ctx).turret_sync_flag)
    }
    fn set_turret_sync(&mut self, _owner: ObjectId, turret: WhichTurretType) {
        if !self.constructing {
            ai(self.sim, self.ctx).turret_sync_flag = turret.as_i32();
        }
    }
    fn is_idle_mood_ignored(&mut self, _owner: ObjectId) -> Bool {
        let adjust = mood_matrix_action_adjustment(self.sim, self.ctx, MoodMatrixAction::Idle);
        Bool::from(adjust & MAA_AFFECT_RANGE_IGNORE_ALL != 0)
    }
    fn acquire_idle_mood_target(&mut self, _owner: ObjectId) -> ObjectId {
        next_mood_target(self.sim, self.ctx, TRUE, TRUE).unwrap_or(ObjectId::INVALID)
    }
    fn choose_best_weapon_for_target(&mut self, _owner: ObjectId, target: ObjectId) {
        let owner = self.owner();
        let victim = self.slot_of(target);
        weapon_set::choose_best_weapon_for_target(
            self.sim,
            owner,
            victim,
            WeaponChoiceCriteria::PreferMostDamage,
            CommandSourceType::FromAi,
        );
    }
    fn out_of_weapon_range_object(&mut self, _owner: ObjectId, victim: ObjectId) -> Bool {
        match self.slot_of(victim) {
            Some(victim) => Bool::from(attack::out_of_range_of(self.sim, self.ctx, victim)),
            None => FALSE,
        }
    }
    fn on_enter_fire_state(&mut self, _owner: ObjectId, victim: ObjectId) -> StateReturnType {
        let victim = self.slot_of(victim);
        attack::enter_fire(self.sim, self.ctx, victim)
    }
    fn on_update_fire_state(
        &mut self,
        _owner: ObjectId,
        _turret: WhichTurretType,
        is_attacking_object: Bool,
        victim: ObjectId,
        goal_pos: &Coord3D,
        is_slot_ok_to_fire: Bool,
    ) -> FireStateStep {
        let victim = self.slot_of(victim);
        let shot = attack::shoot(
            self.sim,
            self.ctx,
            is_attacking_object != FALSE,
            victim,
            goal_pos,
            is_slot_ok_to_fire != FALSE,
        );
        let mut step = FireStateStep {
            status: shot.status,
            transfer_victim: None,
            did_fire: Bool::from(shot.fired),
        };
        // The shot may have moved the attack on to another victim; a turret has
        // no place of its own to look for the next one around.
        if shot.fired && is_attacking_object != FALSE {
            let owner = self.owner();
            self.sim
                .clear_status(owner, ObjectStatusType::IgnoringStealth);
            let current = ai(self.sim, self.ctx).current_victim_id;
            let current_slot = self.slot_of(current);
            if victim != current_slot {
                step.transfer_victim = Some(match current_slot {
                    Some(_) => current,
                    None => ObjectId::INVALID,
                });
            }
        }
        step
    }
    fn on_exit_fire_state(&mut self, _owner: ObjectId) {
        attack::exit_fire(self.sim, self.ctx);
    }
    fn lend_turret(&mut self, turret: &mut TurretAI) {
        self.swap(turret);
    }
    fn reclaim_turret(&mut self, turret: &mut TurretAI) {
        self.swap(turret);
    }
}
