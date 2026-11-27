//! What an object does with the weapons of its set: firing and pre-attack
//! (`Object::fireCurrentWeapon`, `preFireCurrentWeapon`), reloading all of them, and the
//! assistance request a shot makes.
//!
//! Packet PKT-399. Golden test: `crates/zh_logic/tests/golden_s13_weapon_firing.rs`.
//!
//! A weapon in a set is worked on as a copy that is written back, so that the simulation can
//! be borrowed while it fires. Callbacks during the shot see the stored weapon as it was.

use zh_common::types::{Bool, Int, NameKeyType, ObjectId, UnsignedInt, FALSE, TRUE};

use super::weapon::{Weapon, WeaponPrefireType};
use super::weapon_fire::{
    compute_bonus, fire_weapon_at_object, fire_weapon_at_position, load_ammo_now, reload_ammo,
};
use super::weapon_set::{WeaponLockType, WeaponSlotType};
use crate::arena::ObjectSlot;
use crate::combat::firing_tracker::FiredWeaponInfo;
use crate::rules::combat_types::WeaponStatus;
use crate::sim::{SimApi, SimView};
use crate::spatial::partition_query::DistanceCalculationType;

/// Runs `f` on a copy of the weapon in `wslot` and stores the copy back.
pub fn with_weapon_in_slot<R>(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    wslot: WeaponSlotType,
    f: impl FnOnce(&mut Weapon, &mut dyn SimApi) -> R,
) -> Option<R> {
    let mut weapon = sim
        .object(owner)?
        .weapon_set
        .get_weapon_in_weapon_slot(wslot)?
        .clone();
    let result = f(&mut weapon, sim);
    if let Some(stored) = sim
        .object_mut(owner)
        .and_then(|o| o.weapon_set.get_weapon_in_weapon_slot_mut(wslot))
    {
        *stored = weapon;
    }
    Some(result)
}

/// `Object::reloadAllAmmo` (`WeaponSet::reloadAllAmmo`).
pub fn reload_all_ammo(sim: &mut dyn SimApi, owner: ObjectSlot, now: Bool) {
    for wslot in [
        WeaponSlotType::Primary,
        WeaponSlotType::Secondary,
        WeaponSlotType::Tertiary,
    ] {
        with_weapon_in_slot(sim, owner, wslot, |weapon, sim| {
            if now != FALSE {
                load_ammo_now(weapon, sim, owner);
            } else {
                reload_ammo(weapon, sim, owner);
            }
        });
    }
}

/// `Weapon::getPreAttackDelay`.
pub fn pre_attack_delay(
    weapon: &Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
) -> Int {
    match weapon.template.prefire_type {
        WeaponPrefireType::PerClip => {
            let clip = weapon.template.clip_size;
            if clip > 0 && weapon.ammo_in_clip < clip as UnsignedInt {
                return 0; // only the first shot of a clip
            }
        }
        WeaponPrefireType::PerAttack => {
            if sim.consecutive_shots_at_target(source, victim) > 0 {
                return 0; // only the first shot at this victim
            }
        }
        WeaponPrefireType::PerShot => {}
    }
    let bonus = compute_bonus(weapon, sim, source, 0);
    weapon.template.get_pre_attack_delay(&bonus)
}

/// `Weapon::preFireWeapon`.
pub fn pre_fire_weapon(
    weapon: &mut Weapon,
    sim: &dyn SimView,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
) {
    let delay = pre_attack_delay(weapon, sim, source, victim);
    if delay > 0 {
        weapon.status = WeaponStatus::PreAttack;
        weapon.set_pre_attack_finished_frame(sim.frame().wrapping_add(delay as UnsignedInt));
        if weapon.template.leech_range_weapon != FALSE {
            weapon.set_leech_range_active(TRUE);
        }
    }
}

fn current_weapon_slot(sim: &dyn SimView, owner: ObjectSlot) -> Option<WeaponSlotType> {
    let set = &sim.object(owner)?.weapon_set;
    let wslot = set.get_cur_weapon_slot();
    set.get_weapon_in_weapon_slot(wslot).map(|_| wslot)
}

fn forget_defection(sim: &mut dyn SimApi, owner: ObjectSlot) {
    if let Some(o) = sim.object_mut(owner) {
        o.set_undetected_defector(FALSE);
    }
}

/// `Object::preFireCurrentWeapon`: starts the pre-attack delay when the weapon can fire by
/// the next frame.
pub fn pre_fire_current_weapon(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim: Option<ObjectSlot>,
) {
    let Some(wslot) = current_weapon_slot(sim, owner) else {
        return;
    };
    let next_frame = sim.frame().wrapping_add(1);
    let fired = with_weapon_in_slot(sim, owner, wslot, |weapon, sim| {
        let ready = next_frame >= weapon.when_we_can_fire_again;
        if ready {
            pre_fire_weapon(weapon, sim, owner, victim);
        }
        ready
    });
    if fired == Some(true) {
        forget_defection(sim, owner);
    }
}

/// The part the two `fireCurrentWeapon` overloads share.
fn fire_current(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim_id: ObjectId,
    fire: impl FnOnce(&mut Weapon, &mut dyn SimApi) -> Bool,
) {
    let Some(wslot) = current_weapon_slot(sim, owner) else {
        return;
    };
    let now = sim.frame();
    // `getStatus` stores the refreshed status.
    let ready = with_weapon_in_slot(sim, owner, wslot, |weapon, _| {
        weapon.get_status(now) == WeaponStatus::ReadyToFire
    });
    if ready != Some(true) {
        return;
    }
    let Some(reloaded) = with_weapon_in_slot(sim, owner, wslot, fire) else {
        return;
    };
    notify_shot_fired(sim, owner, wslot, victim_id);
    if reloaded != FALSE {
        // A temporary lock lasts for one clip.
        if let Some(o) = sim.object_mut(owner) {
            o.weapon_set
                .release_weapon_lock(WeaponLockType::LockedTemporarily);
        }
    }
    forget_defection(sim, owner);
}

/// `Object::fireCurrentWeapon(Object*)`.
pub fn fire_current_weapon_at_object(sim: &mut dyn SimApi, owner: ObjectSlot, target: ObjectSlot) {
    let Some(victim_id) = sim.object(target).map(|o| o.id) else {
        return;
    };
    fire_current(sim, owner, victim_id, |weapon, sim| {
        fire_weapon_at_object(weapon, sim, owner, target)
    });
}

/// `Object::fireCurrentWeapon(const Coord3D*)`.
pub fn fire_current_weapon_at_position(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    pos: &zh_common::math::Coord3D,
) {
    fire_current(sim, owner, ObjectId::INVALID, |weapon, sim| {
        fire_weapon_at_position(weapon, sim, owner, pos)
    });
}

/// The linked turrets' shot from one slot: `fireWeapon(pos)`, then the unlock
/// after a reload and the firing tracker, whether or not it fired.
pub fn fire_linked_weapon_at_position(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    wslot: WeaponSlotType,
    pos: &zh_common::math::Coord3D,
) {
    let Some(reloaded) = with_weapon_in_slot(sim, owner, wslot, |weapon, sim| {
        fire_weapon_at_position(weapon, sim, owner, pos)
    }) else {
        return;
    };
    if reloaded != FALSE {
        if let Some(o) = sim.object_mut(owner) {
            o.weapon_set
                .release_weapon_lock(WeaponLockType::LockedTemporarily);
        }
    }
    notify_shot_fired(sim, owner, wslot, ObjectId::INVALID);
}

/// `Object::notifyFiringTrackerShotFired`.
fn notify_shot_fired(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    wslot: WeaponSlotType,
    victim_id: ObjectId,
) {
    let fired = sim
        .object(owner)
        .and_then(|o| o.weapon_set.get_weapon_in_weapon_slot(wslot))
        .map(|w| {
            // The sound's key only reaches the audio engine.
            FiredWeaponInfo::from_template(
                &w.template,
                w.when_we_can_fire_again,
                w.template.fire_sound_loop_time,
                NameKeyType::INVALID,
            )
        });
    if let Some(info) = fired {
        sim.notify_firing_tracker_shot_fired(owner, &info, victim_id);
    }
}

/// `Weapon::processRequestAssistance`: every unit of the same kind and player near enough,
/// that is free to, joins the attack on `victim`.
pub fn process_request_assistance(
    sim: &mut dyn SimApi,
    range: zh_common::types::Real,
    requesting: ObjectSlot,
    victim: ObjectSlot,
) {
    let Some((team, requesting_template)) = sim
        .object(requesting)
        .map(|o| (o.team_id, o.thing.template_override().clone()))
    else {
        return;
    };
    let range_sqr = range * range;
    for assistant in sim.controlling_player_objects(team) {
        if assistant == requesting {
            continue;
        }
        let factory = &sim.rules().thing_factory;
        let same_kind = sim.object(assistant).is_some_and(|o| {
            match (
                o.thing.template_override().get_final(factory),
                requesting_template.get_final(factory),
            ) {
                (Some(a), Some(b)) => a.is_equivalent_to(b) != FALSE,
                _ => false,
            }
        });
        if !same_kind {
            continue;
        }
        let dist_sqr = sim.partition_distance_squared(
            assistant,
            Some(requesting),
            &Default::default(),
            DistanceCalculationType::FromCenter2d,
        );
        if dist_sqr > range_sqr {
            continue;
        }
        sim.request_assist_from(assistant, requesting, victim);
    }
}
