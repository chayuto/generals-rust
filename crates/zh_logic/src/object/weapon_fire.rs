//! The weapon firing state machine (`Weapon.cpp`), shared by every module that fires a weapon.
//!
//! Each function takes the weapon, the simulation and the owning object. A weapon that sits in the
//! owner's weapon set is fired on a copy which the caller writes back; the shared-reload updates
//! that reach the weapon itself only restate values it already holds.
//!
//! `WeaponTemplate::fireWeaponTemplate` is unported: [`SimApi::fire_weapon_template`] is called
//! where C++ calls it. Also unported: `DAMAGE_DEPLOY`'s assault order, `processRequestAssistance`
//! and the victim effects of `DAMAGE_DISARM`.
//!
//! Golden test: `crates/zh_logic/tests/golden_wp55_fire.rs`.

use zh_common::math::{real_to_int_floor, Coord3D};
use zh_common::types::{Bool, Int, Real, UnsignedInt, FALSE, TRUE};

use super::weapon::{Weapon, WeaponReloadType, NO_MAX_SHOTS_LIMIT};
use super::weapon_set::WeaponSlotType;
use crate::arena::ObjectSlot;
use crate::gamelogic::global_data::{WeaponBonus, WeaponBonusField};
use crate::rules::combat_types::{DamageType, WeaponStatus};
use crate::sim::{SimApi, SimView};

/// `Weapon::computeBonus`. `weapon_bonus_condition_flags` already merges the container's flags.
pub fn compute_bonus(
    weapon: &Weapon,
    sim: &dyn SimView,
    owner: ObjectSlot,
    extra_flags: UnsignedInt,
) -> WeaponBonus {
    let mut bonus = WeaponBonus::new();
    let flags = sim.weapon_bonus_condition_flags(owner) | extra_flags;
    if let Some(global) = sim.rules().global_data.weapon_bonus_set.as_ref() {
        global.append_bonuses(flags, &mut bonus);
    }
    weapon.template.extra_bonus.append_bonuses(flags, &mut bonus);
    bonus
}

/// `Weapon::reloadWithBonus`.
pub fn reload_with_bonus(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    bonus: &WeaponBonus,
    load_instantly: Bool,
) {
    let clip = weapon.template.clip_size;
    let shared = sim.is_reload_time_shared(owner);
    if clip > 0 && weapon.ammo_in_clip == clip as UnsignedInt && shared == FALSE {
        return; // don't restart our reload delay
    }

    weapon.ammo_in_clip = clip as UnsignedInt;
    if weapon.ammo_in_clip == 0 {
        weapon.ammo_in_clip = NO_MAX_SHOTS_LIMIT as UnsignedInt;
    }

    weapon.status = WeaponStatus::ReloadingClip;
    let reload_time: Real = if load_instantly != FALSE {
        0.0
    } else {
        weapon.template.get_clip_reload_time(bonus) as Real
    };
    let now = sim.frame();
    weapon.when_last_reload_started = now;
    // `UnsignedInt + Real`: the frame is rounded to `f32` before the addition.
    weapon.when_we_can_fire_again = ((now as Real) + reload_time) as UnsignedInt;

    if shared != FALSE {
        sim.share_weapon_reload(owner, weapon.when_we_can_fire_again, WeaponStatus::ReloadingClip);
    }

    weapon.rebuild_scatter_targets();
}

/// `Weapon::loadAmmoNow`.
pub fn load_ammo_now(weapon: &mut Weapon, sim: &mut dyn SimApi, owner: ObjectSlot) {
    let bonus = compute_bonus(weapon, sim, owner, 0);
    reload_with_bonus(weapon, sim, owner, &bonus, TRUE);
}

/// `Weapon::reloadAmmo`.
pub fn reload_ammo(weapon: &mut Weapon, sim: &mut dyn SimApi, owner: ObjectSlot) {
    let bonus = compute_bonus(weapon, sim, owner, 0);
    reload_with_bonus(weapon, sim, owner, &bonus, FALSE);
}

/// `WeaponTemplate::getDelayBetweenShots`, drawing from the logic stream only for a real range.
fn delay_between_shots(weapon: &Weapon, sim: &mut dyn SimApi, bonus: &WeaponBonus) -> Int {
    let lo = weapon.template.min_delay_between_shots;
    let hi = weapon.template.max_delay_between_shots;
    let delay = if lo == hi { lo } else { sim.rng_logic_int(lo, hi) };
    real_to_int_floor((delay as Real) / bonus.get_field(WeaponBonusField::RateOfFire))
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

/// `Weapon::privateFireWeapon`. Returns `TRUE` when the clip was reloaded by this shot.
/// Exactly one of `victim` and `victim_pos` is given.
fn private_fire_weapon(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim: Option<ObjectSlot>,
    victim_pos: Option<Coord3D>,
) -> Bool {
    if weapon.template.leech_range_weapon != FALSE {
        weapon.set_leech_range_active(TRUE);
    }

    let auto_reloads = weapon.template.reload_type == WeaponReloadType::AutoReload;
    if weapon.template.damage_type == DamageType::Disarm {
        // No status check here, so an empty clip wraps around.
        weapon.max_shot_count = weapon.max_shot_count.wrapping_sub(1);
        weapon.ammo_in_clip = weapon.ammo_in_clip.wrapping_sub(1);
        if weapon.ammo_in_clip == 0 && auto_reloads {
            reload_ammo(weapon, sim, owner);
            return TRUE;
        }
        return FALSE;
    }

    let bonus = compute_bonus(weapon, sim, owner, 0);
    let now = sim.frame();
    if weapon.get_status(now) != WeaponStatus::ReadyToFire || weapon.ammo_in_clip == 0 {
        return FALSE;
    }

    let barrel_count = sim.weapon_barrel_count(owner, weapon.wslot);
    if weapon.cur_barrel >= barrel_count {
        weapon.cur_barrel = 0;
        weapon.num_shots_for_cur_barrel = weapon.template.shots_per_barrel;
    }

    let mut pos = match victim {
        Some(v) => position_of(sim, v),
        None => victim_pos.unwrap_or_default(),
    };
    let mut target = victim;
    if !weapon.scatter_targets_unused.is_empty() {
        // A scatter pattern aims at the ground around the target, not at the object.
        target = None;
        let last = (weapon.scatter_targets_unused.len() as Int).wrapping_sub(1);
        let pick = sim.rng_logic_int(0, last) as usize;
        let index = weapon.scatter_targets_unused[pick] as usize;
        let scalar = weapon.template.scatter_target_scalar;
        let mut offset = weapon
            .template
            .scatter_targets
            .get(index)
            .copied()
            .unwrap_or_default();
        offset.x *= scalar;
        offset.y *= scalar;
        pos.x += offset.x;
        pos.y += offset.y;
        pos.z = sim.terrain().get_ground_height(pos.x, pos.y, None);
        weapon.scatter_targets_unused.swap_remove(pick);
    }
    sim.fire_weapon_template(owner, &weapon.template.name, target, &pos);

    weapon.last_fire_frame = now;
    weapon.ammo_in_clip = weapon.ammo_in_clip.wrapping_sub(1);
    weapon.max_shot_count = weapon.max_shot_count.wrapping_sub(1);
    weapon.num_shots_for_cur_barrel = weapon.num_shots_for_cur_barrel.wrapping_sub(1);
    if weapon.num_shots_for_cur_barrel <= 0 {
        weapon.cur_barrel = weapon.cur_barrel.wrapping_add(1);
        weapon.num_shots_for_cur_barrel = weapon.template.shots_per_barrel;
    }

    if weapon.ammo_in_clip == 0 {
        if auto_reloads {
            reload_ammo(weapon, sim, owner);
            return TRUE;
        }
        weapon.status = WeaponStatus::OutOfAmmo;
        weapon.when_we_can_fire_again = NO_MAX_SHOTS_LIMIT as UnsignedInt;
    } else {
        weapon.status = WeaponStatus::BetweenFiringShots;
        let delay = delay_between_shots(weapon, sim, &bonus);
        weapon.when_last_reload_started = now;
        weapon.when_we_can_fire_again = now.wrapping_add(delay as UnsignedInt);
        if sim.is_reload_time_shared(owner) != FALSE {
            sim.share_weapon_reload(
                owner,
                weapon.when_we_can_fire_again,
                WeaponStatus::BetweenFiringShots,
            );
        }
    }
    FALSE
}

/// `Weapon::fireWeapon(source, target)`.
pub fn fire_weapon_at_object(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim: ObjectSlot,
) -> Bool {
    private_fire_weapon(weapon, sim, owner, Some(victim), None)
}

/// `Weapon::fireWeapon(source, pos)`.
pub fn fire_weapon_at_position(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    pos: &Coord3D,
) -> Bool {
    private_fire_weapon(weapon, sim, owner, None, Some(*pos))
}

/// `Weapon::forceFireWeapon`. The projectile it returns in C++ comes from the unported
/// `fireWeaponTemplate`.
pub fn force_fire_weapon(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    pos: &Coord3D,
) {
    private_fire_weapon(weapon, sim, owner, None, Some(*pos));
}

fn allocate_temp_weapon(sim: &dyn SimView, name: &str) -> Option<Weapon> {
    let template = sim.rules().weapon_store.find_weapon_template(name)?.clone();
    Some(Weapon::new(template, WeaponSlotType::Primary, sim.frame()))
}

/// `WeaponStore::createAndFireTempWeapon(wt, source, pos)`. An unknown name does nothing.
pub fn create_and_fire_temp_weapon(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    name: &str,
    pos: &Coord3D,
) {
    let Some(mut weapon) = allocate_temp_weapon(sim, name) else {
        return;
    };
    load_ammo_now(&mut weapon, sim, owner);
    fire_weapon_at_position(&mut weapon, sim, owner, pos);
}

/// `WeaponStore::createAndFireTempWeapon(wt, source, target)`.
pub fn create_and_fire_temp_weapon_at_object(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    name: &str,
    victim: ObjectSlot,
) {
    let Some(mut weapon) = allocate_temp_weapon(sim, name) else {
        return;
    };
    load_ammo_now(&mut weapon, sim, owner);
    fire_weapon_at_object(&mut weapon, sim, owner, victim);
}
