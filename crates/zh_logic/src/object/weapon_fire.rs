//! The weapon firing state machine (`Weapon.cpp`), shared by every module that fires a weapon.
//!
//! Each function takes the weapon, the simulation and the owning object. A weapon that sits in the
//! owner's weapon set is fired on a copy which the caller writes back; the shared-reload updates
//! that reach the weapon itself only restate values it already holds.
//!
//! `WeaponTemplate::fireWeaponTemplate` and the damage it deals are in
//! [`super::weapon_effects`]. Unported: `DAMAGE_DEPLOY`'s assault order,
//! `processRequestAssistance` and the victim effects of `DAMAGE_DISARM`.
//!
//! Golden test: `crates/zh_logic/tests/golden_wp55_fire.rs`.

use zh_common::math::{real_to_int_floor, Coord3D, Matrix3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, UnsignedInt, FALSE, TRUE};

use super::weapon::{
    Weapon, WeaponReloadType, NO_MAX_SHOTS_LIMIT, WEAPON_COLLIDE_ALLIES,
    WEAPON_COLLIDE_BALLISTIC_MISSILES, WEAPON_COLLIDE_CONTROLLED_STRUCTURES,
    WEAPON_COLLIDE_ENEMIES, WEAPON_COLLIDE_PROJECTILES, WEAPON_COLLIDE_SHRUBBERY,
    WEAPON_COLLIDE_SMALL_MISSILES, WEAPON_COLLIDE_STRUCTURES, WEAPON_COLLIDE_WALLS,
};
use super::weapon_effects::{self, Shot};
use super::weapon_set::WeaponSlotType;
use crate::arena::ObjectSlot;
use crate::gamelogic::global_data::{WeaponBonus, WeaponBonusField};
use crate::modules::update::physics_update;
use crate::rules::combat_types::{DamageType, WeaponStatus};
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
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

/// `Weapon::onWeaponBonusChange` for the weapon in `wslot` of the owner's set: a running
/// reload or shot delay restarts from now with the new rate of fire.
pub fn on_weapon_bonus_change(sim: &mut dyn SimApi, owner: ObjectSlot, wslot: WeaponSlotType) {
    let Some(mut weapon) = sim
        .object(owner)
        .and_then(|o| o.weapon_set.get_weapon_in_weapon_slot(wslot))
        .cloned()
    else {
        return;
    };
    let bonus = compute_bonus(&weapon, sim, owner, 0);
    let now = sim.frame();
    // `getStatus` stores the refreshed status in the weapon.
    let delay = match weapon.get_status(now) {
        WeaponStatus::ReloadingClip => Some(weapon.template.get_clip_reload_time(&bonus)),
        WeaponStatus::BetweenFiringShots => Some(delay_between_shots(&weapon, sim, &bonus)),
        _ => None,
    };
    if let Some(delay) = delay {
        weapon.when_last_reload_started = now;
        weapon.when_we_can_fire_again = now.wrapping_add(delay as UnsignedInt);
    }
    let when = weapon.when_we_can_fire_again;
    if let Some(stored) = sim
        .object_mut(owner)
        .and_then(|o| o.weapon_set.get_weapon_in_weapon_slot_mut(wslot))
    {
        *stored = weapon;
    }
    if delay.is_some() && sim.is_reload_time_shared(owner) != FALSE {
        sim.share_weapon_reload(owner, when, WeaponStatus::ReloadingClip);
    }
}

/// `WeaponTemplate::getDelayBetweenShots`, drawing from the logic stream only for a real range.
pub fn delay_between_shots(weapon: &Weapon, sim: &mut dyn SimApi, bonus: &WeaponBonus) -> Int {
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

/// What a shot is: aimed by the weapon's own rules, forced past the range checks, or the
/// detonation of a projectile with the launcher's bonus flags.
#[derive(Clone, Copy)]
enum FireMode {
    Normal,
    IgnoreRanges,
    Detonation(UnsignedInt),
}

/// `Weapon::privateFireWeapon`. Returns `TRUE` when the clip was reloaded by this shot, and
/// the projectile it launched. Exactly one of `victim` and `victim_pos` is given.
fn private_fire_weapon(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim: Option<ObjectSlot>,
    victim_pos: Option<Coord3D>,
    mode: FireMode,
) -> (Bool, Option<ObjectSlot>) {
    let detonation = match mode {
        FireMode::Detonation(flags) => Some(flags),
        _ => None,
    };
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
            return (TRUE, None);
        }
        return (FALSE, None);
    }

    let bonus = compute_bonus(weapon, sim, owner, detonation.unwrap_or(0));
    let now = sim.frame();
    if weapon.get_status(now) != WeaponStatus::ReadyToFire || weapon.ammo_in_clip == 0 {
        return (FALSE, None);
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
    let mut projectile = None;
    if sim.record_weapon_fire(owner, &weapon.template.name, target, &pos, detonation) != FALSE {
        let shot = Shot {
            victim: target,
            victim_pos: pos,
            bonus: &bonus,
            is_detonation: Bool::from(detonation.is_some()),
            ignore_ranges: Bool::from(matches!(mode, FireMode::IgnoreRanges)),
            inflict_damage: TRUE,
        };
        projectile = weapon_effects::fire_weapon_template(sim, weapon, owner, &shot);
    }

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
            return (TRUE, projectile);
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
    (FALSE, projectile)
}

/// `Weapon::fireWeapon(source, target)`.
pub fn fire_weapon_at_object(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    victim: ObjectSlot,
) -> Bool {
    private_fire_weapon(weapon, sim, owner, Some(victim), None, FireMode::Normal).0
}

/// `Weapon::fireWeapon(source, pos)`.
pub fn fire_weapon_at_position(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    pos: &Coord3D,
) -> Bool {
    private_fire_weapon(weapon, sim, owner, None, Some(*pos), FireMode::Normal).0
}

/// `Weapon::forceFireWeapon`: no range checks. Returns the projectile it launched.
pub fn force_fire_weapon(
    weapon: &mut Weapon,
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    pos: &Coord3D,
) -> Option<ObjectSlot> {
    private_fire_weapon(weapon, sim, owner, None, Some(*pos), FireMode::IgnoreRanges).1
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

/// `WeaponStore::handleProjectileDetonation`: a temporary weapon fired at `pos` with the
/// launcher's bonus flags.
pub fn handle_projectile_detonation(
    sim: &mut dyn SimApi,
    owner: ObjectSlot,
    name: &str,
    pos: &Coord3D,
    extra_bonus_flags: UnsignedInt,
) {
    let Some(mut weapon) = allocate_temp_weapon(sim, name) else {
        return;
    };
    load_ammo_now(&mut weapon, sim, owner);
    private_fire_weapon(
        &mut weapon,
        sim,
        owner,
        None,
        Some(*pos),
        FireMode::Detonation(extra_bonus_flags),
    );
}

/// `Weapon::positionProjectileForLaunch`. Launch bones and turrets come from the unported
/// draw and AI modules, so the projectile starts at the launcher's own transform, which is
/// what C++ does for a launcher with neither.
pub fn position_projectile_for_launch(
    sim: &mut dyn SimApi,
    projectile: ObjectSlot,
    launcher: Option<ObjectSlot>,
    _wslot: WeaponSlotType,
    _specific_barrel_to_use: Int,
) {
    let Some((launcher, launcher_id, container, launcher_mtx)) = launcher.and_then(|l| {
        sim.object(l)
            .map(|o| (l, o.id, o.contained_by, *o.thing.get_transform_matrix()))
    }) else {
        sim.destroy_object(projectile);
        return;
    };

    let enclosed = container
        .is_some_and(|c| sim.contain_is_enclosing_container_for(c, launcher) != FALSE);
    let world = if enclosed {
        launcher_mtx
    } else {
        Matrix3D::mul(&launcher_mtx, &Matrix3D::identity())
    };
    let world_pos = Coord3D::new(world.rows[0][3], world.rows[1][3], world.rows[2][3]);
    sim.set_object_transform_matrix(projectile, &world);
    sim.set_object_position(projectile, &world_pos);
    if let Some(p) = sim.object_mut(projectile) {
        p.experience_tracker.experience_sink = launcher_id;
    }

    if sim.has_physics(launcher) != FALSE && sim.has_physics(projectile) != FALSE {
        physics_update::transfer_velocity_to(sim, launcher, projectile);
        if let Some(p) = physics_update::physics_mut(sim, projectile) {
            p.ignore_collisions_with = launcher_id;
        }
    }
}

/// `WeaponTemplate::shouldProjectileCollideWith`. The sneaky-targeting immunity of
/// `JetAIUpdate` is not modelled (that AI is unported).
pub fn should_projectile_collide_with(
    sim: &dyn SimView,
    weapon_name: Option<&str>,
    launcher: Option<ObjectSlot>,
    projectile: ObjectSlot,
    other: ObjectSlot,
    intended_victim_id: ObjectId,
) -> bool {
    let (Some(shell), Some(thing)) = (sim.object(projectile), sim.object(other)) else {
        return false;
    };
    if intended_victim_id == thing.id {
        return true;
    }
    if let Some(l) = launcher {
        if l == other || sim.object(l).and_then(|o| o.contained_by) == Some(other) {
            return false;
        }
    }
    // C++ would dereference a null template from here on; the collision is accepted.
    let Some(template) = weapon_name.and_then(|n| sim.rules().weapon_store.find_weapon_template(n))
    else {
        return true;
    };
    if matches!(
        template.damage_type,
        DamageType::Flame | DamageType::ParticleBeam
    ) && thing.status.test(ObjectStatusType::Burned.as_i32()) != FALSE
    {
        return false;
    }
    let is = |kind| sim.object_is_kind_of(other, kind) != FALSE;
    if is(KindOfType::FsAirfield)
        && sim.object_has_reserved_space(other, intended_victim_id) != FALSE
    {
        return false;
    }

    let mut required = match sim.player_relationship(projectile, other) {
        Relationship::Allies => WEAPON_COLLIDE_ALLIES,
        Relationship::Enemies => WEAPON_COLLIDE_ENEMIES,
        _ => 0,
    };
    if is(KindOfType::Structure) {
        if sim.player_index(thing.team_id) == sim.player_index(shell.team_id) {
            required |= WEAPON_COLLIDE_CONTROLLED_STRUCTURES;
        } else {
            required |= WEAPON_COLLIDE_STRUCTURES;
        }
    }
    if is(KindOfType::Shrubbery) {
        required |= WEAPON_COLLIDE_SHRUBBERY;
    }
    if is(KindOfType::Projectile) {
        required |= WEAPON_COLLIDE_PROJECTILES;
    }
    let fenced = thing
        .thing
        .template_override()
        .get_final(&sim.rules().thing_factory)
        .is_some_and(|t| t.get_fence_width() > 0.0);
    if fenced {
        required |= WEAPON_COLLIDE_WALLS;
    }
    if is(KindOfType::SmallMissile) {
        required |= WEAPON_COLLIDE_SMALL_MISSILES;
    }
    if is(KindOfType::BallisticMissile) {
        required |= WEAPON_COLLIDE_BALLISTIC_MISSILES;
    }
    (template.collide_mask & required) != 0
}
