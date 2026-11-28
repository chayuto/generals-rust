//! What a fired weapon does (`Weapon.cpp`): `WeaponTemplate::fireWeaponTemplate`,
//! `dealDamageInternal`, the historic bonus and the delayed damage queue of `WeaponStore`.
//!
//! Packet PKT-399. Golden test: `crates/zh_logic/tests/golden_pkt399_weapon_effects.rs`.
//!
//! Client work (fire FX, recoil, the laser beam) goes to the `ClientSink`. The sneaky
//! targeting offset of `JetAIUpdate` and the records of bridges are asked of
//! [`crate::ai::seams::AiSeams`]. A projectile without a `ProjectileUpdateInterface` is put
//! at its destination, as C++ does; that includes every `MissileAIUpdate` projectile, which
//! therefore never detonates.

use std::collections::BTreeMap;

use zh_common::math::{self, div_safe_f32, gm_fabsf, real_to_int_ceil, Coord2D, Coord3D};
use zh_common::types::{Bool, Int, ObjectId, Real, Relationship, UnsignedInt, FALSE};

use super::weapon::{
    HistoricWeaponDamageInfo, Weapon, WeaponTemplate, WEAPON_AFFECTS_ALLIES,
    WEAPON_AFFECTS_ENEMIES, WEAPON_AFFECTS_NEUTRALS, WEAPON_AFFECTS_NOT_AIRBORNE,
    WEAPON_AFFECTS_NOT_SIMILAR, WEAPON_AFFECTS_SELF, WEAPON_AFFECTS_SUICIDE,
};
use super::weapon_fire;
use crate::ai::seams::ai_seams;
use crate::arena::ObjectSlot;
use crate::combat::damage::DamageInfo;
use crate::combat::locomotor_set::LocomotorSetType;
use crate::gamelogic::global_data::WeaponBonus;
use crate::modules::update::battle_plan_update;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::sim::{SimApi, SimView};
use crate::spatial::partition_manager::DistanceCalculationType;

const PI: Real = std::f32::consts::PI;
/// `HUGE_DAMAGE_AMOUNT`.
pub const HUGE_DAMAGE_AMOUNT: Real = 999_999.0;
/// `ATTACK_RANGE_CALC_TYPE`.
const ATTACK_RANGE: DistanceCalculationType = DistanceCalculationType::FromBoundingSphere2d;

/// `WeaponDelayedDamageInfo`.
#[derive(Debug, Clone)]
pub struct DelayedDamage {
    pub weapon_name: String,
    pub pos: Coord3D,
    pub frame: UnsignedInt,
    pub source_id: ObjectId,
    pub victim_id: ObjectId,
    pub bonus: WeaponBonus,
}

/// `m_historicDamage` and `m_historicDamageTriggerId` of one weapon template.
#[derive(Debug, Clone, Default)]
pub struct WeaponHistory {
    pub damage: Vec<HistoricWeaponDamageInfo>,
    pub trigger_id: UnsignedInt,
}

/// The state C++ keeps in `WeaponStore` and its templates. None of it is saved or checksummed.
#[derive(Debug, Clone, Default)]
pub struct WeaponStoreState {
    /// `m_weaponDDI`, oldest first.
    pub delayed: Vec<DelayedDamage>,
    /// By template name.
    pub history: BTreeMap<String, WeaponHistory>,
    /// The projectile that is detonating and its launcher. Its module is executing then, so
    /// `projectileGetLauncherID` cannot be asked of it.
    pub detonating: Option<(ObjectSlot, ObjectId)>,
}

/// How one shot is fired: the arguments of `fireWeaponTemplate` besides the firing weapon.
pub struct Shot<'a> {
    pub victim: Option<ObjectSlot>,
    /// Used when there is no victim.
    pub victim_pos: Coord3D,
    pub bonus: &'a WeaponBonus,
    pub is_detonation: Bool,
    pub ignore_ranges: Bool,
    pub inflict_damage: Bool,
}

fn sqr(x: Real) -> Real {
    x * x
}

fn id_of(sim: &dyn SimView, slot: ObjectSlot) -> ObjectId {
    sim.object(slot).map_or(ObjectId::INVALID, |o| o.id)
}

fn position_of(sim: &dyn SimView, slot: ObjectSlot) -> Coord3D {
    sim.object(slot)
        .map(|o| *o.thing.get_position())
        .unwrap_or_default()
}

fn is_kind(sim: &dyn SimView, slot: ObjectSlot, kind: KindOfType) -> bool {
    sim.object_is_kind_of(slot, kind) != FALSE
}

fn center_of(sim: &dyn SimView, slot: ObjectSlot, out: &mut Coord3D) {
    if let Some(o) = sim.object(slot) {
        o.geometry.get_center_position(o.thing.get_position(), out);
    }
}

fn template_of(sim: &dyn SimView, name: &str) -> Option<WeaponTemplate> {
    sim.rules().weapon_store.find_weapon_template(name).cloned()
}

/// `m_projectileTmpl`: a projectile name the thing factory does not know is no projectile.
fn has_projectile(sim: &dyn SimView, template: &WeaponTemplate) -> bool {
    !template.projectile_name.is_empty()
        && sim
            .rules()
            .thing_factory
            .find_template_id(&template.projectile_name)
            .is_some()
}

/// Where a shot goes: the victim it still has, the position aimed at and the squared
/// distance the range checks use.
struct Aim {
    victim: Option<ObjectSlot>,
    pos: Coord3D,
    dist_sqr: Real,
}

fn aim_at(sim: &dyn SimView, source: ObjectSlot, shot: &Shot<'_>) -> Aim {
    let distance = |victim, pos: &Coord3D| {
        sim.partition_distance_squared(source, victim, pos, ATTACK_RANGE)
    };
    let Some(v) = shot.victim else {
        return Aim {
            victim: None,
            pos: shot.victim_pos,
            dist_sqr: distance(None, &shot.victim_pos),
        };
    };
    let mut pos = position_of(sim, v);
    if let Some((id, true)) = sim.object(v).map(|o| (o.id, o.ai_slot.is_some())) {
        if let Some(offset) = ai_seams(sim).sneaky_targeting_offset(id) {
            pos.x += offset.x;
            pos.y += offset.y;
            pos.z += offset.z;
            // A dodging jet is shot at as a position.
            return Aim {
                victim: None,
                pos,
                dist_sqr: distance(None, &pos),
            };
        }
    }
    if !is_kind(sim, v, KindOfType::Bridge) {
        return Aim {
            victim: Some(v),
            pos,
            dist_sqr: distance(Some(v), &pos),
        };
    }
    // A bridge is aimed at the nearer of its two attack points.
    let points = ai_seams(sim).bridge_attack_points(id_of(sim, v), &pos);
    let mut dist_sqr = distance(None, &points[0]);
    pos = points[0];
    let second = distance(None, &points[1]);
    if dist_sqr > second {
        dist_sqr = second;
        pos = points[1];
    }
    Aim {
        victim: Some(v),
        pos,
        dist_sqr,
    }
}

/// `WeaponTemplate::fireWeaponTemplate`. Returns the projectile it launched, if any.
pub fn fire_weapon_template(
    sim: &mut dyn SimApi,
    weapon: &mut Weapon,
    source: ObjectSlot,
    shot: &Shot<'_>,
) -> Option<ObjectSlot> {
    let source_id = sim.object(source)?.id;
    let source_pos = position_of(sim, source);
    let template = weapon.template.clone();
    let bonus = shot.bonus;
    let Aim {
        victim,
        pos: victim_pos,
        dist_sqr,
    } = aim_at(sim, source, shot);
    let victim_id = victim.map_or(ObjectId::INVALID, |v| id_of(sim, v));

    if shot.ignore_ranges == FALSE
        && template.leech_range_weapon == FALSE
        && dist_sqr > sqr(template.get_attack_range(bonus))
    {
        return None;
    }
    if shot.ignore_ranges == FALSE {
        let min_sqr = sqr(template.get_minimum_attack_range());
        if dist_sqr < min_sqr && shot.is_detonation == FALSE {
            return None;
        }
    }

    sim.client()
        .on_weapon_fire(source_id, &template.name, &victim_pos, shot.is_detonation);

    let veterancy = sim
        .object(source)
        .map_or(0, |o| o.experience_tracker.get_veterancy_level() as usize);
    let ocl = if shot.is_detonation != FALSE {
        &template.projectile_detonation_ocl[veterancy]
    } else {
        &template.fire_ocl[veterancy]
    };
    if !ocl.is_empty() {
        sim.create_ocl(
            ocl,
            crate::object::ocl_runtime::OclCreateArgs::object(source, None),
        );
    }

    let mut destination = victim_pos;
    let mut scatter_radius: Real = 0.0;
    let victim_is_infantry = victim.is_some_and(|v| is_kind(sim, v, KindOfType::Infantry));
    if template.scatter_radius > 0.0
        || (template.scatter_radius_vs_infantry > 0.0 && victim_is_infantry)
    {
        scatter_radius = template.scatter_radius;
        let mut layer = PathfindLayerEnum::Ground;
        if let Some(v) = victim {
            if is_kind(sim, v, KindOfType::Structure) {
                center_of(sim, v, &mut destination);
            }
            if template.scatter_radius_vs_infantry > 0.0 && victim_is_infantry {
                scatter_radius += template.scatter_radius_vs_infantry;
            }
            layer = sim.object_layer(v).unwrap_or(PathfindLayerEnum::Ground);
        }

        scatter_radius = sim.rng_logic_real(0.0, scatter_radius);
        let angle = sim.rng_logic_real(0.0, 2.0 * PI);
        destination.x += scatter_radius * math::gamemath::cos(angle);
        destination.y += scatter_radius * math::gamemath::sin(angle);
        destination.z = sim.get_layer_height(destination.x, destination.y, layer);
    }

    if shot.is_detonation != FALSE || !has_projectile(sim, &template) {
        let delta = Coord3D::new(
            victim_pos.x - source_pos.x,
            victim_pos.y - source_pos.y,
            victim_pos.z - source_pos.z,
        );
        let delay_in_frames = div_safe_f32(delta.length(), template.weapon_speed);
        let mut damage_id = if template.damage_dealt_at_self_position != FALSE {
            ObjectId::INVALID
        } else {
            victim_id
        };

        if !template.laser_name.is_empty() {
            if scatter_radius <= template.get_primary_damage_radius(bonus)
                || scatter_radius <= template.get_secondary_damage_radius(bonus)
            {
                if let Some(v) = victim {
                    destination = position_of(sim, v);
                }
                create_laser(sim, &template, source, victim, &destination);
            } else {
                damage_id = ObjectId::INVALID;
                create_laser(sim, &template, source, None, &destination);
            }
            if shot.inflict_damage != FALSE {
                deal_damage_internal(
                    sim,
                    &template,
                    source_id,
                    damage_id,
                    &destination,
                    bonus,
                    shot.is_detonation,
                );
            }
            return None;
        }

        let damage_pos = if template.damage_dealt_at_self_position != FALSE {
            source_pos
        } else {
            victim_pos
        };
        if delay_in_frames < 1.0 {
            if shot.inflict_damage != FALSE {
                deal_damage_internal(
                    sim,
                    &template,
                    source_id,
                    damage_id,
                    &damage_pos,
                    bonus,
                    shot.is_detonation,
                );
            }
        } else if shot.inflict_damage != FALSE {
            let whole = real_to_int_ceil(delay_in_frames) as UnsignedInt;
            let frame = sim.frame().wrapping_add(whole);
            sim.weapon_store_state().delayed.push(DelayedDamage {
                weapon_name: template.name.clone(),
                pos: damage_pos,
                frame,
                source_id,
                victim_id: damage_id,
                bonus: bonus.clone(),
            });
        }
        return None;
    }

    launch(
        sim,
        weapon,
        source,
        victim,
        &victim_pos,
        &destination,
        scatter_radius,
    )
}

/// The projectile half of `fireWeaponTemplate`.
fn launch(
    sim: &mut dyn SimApi,
    weapon: &mut Weapon,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
    victim_pos: &Coord3D,
    destination: &Coord3D,
    scatter_radius: Real,
) -> Option<ObjectSlot> {
    let (source_id, team) = sim.object(source).map(|o| (o.id, o.team_id))?;
    let default_team = sim.player_default_team(team);
    let projectile =
        sim.new_object_from_template(&weapon.template.projectile_name, default_team)?;
    if let Some(p) = sim.object_mut(projectile) {
        p.producer_id = source_id;
    }

    let player = sim.player_index(team);
    let plans = sim.player_battle_plans(player).and_then(|p| {
        if p.num_active() > 0 {
            p.bonuses.clone()
        } else {
            None
        }
    });
    if let Some(bonuses) = plans {
        battle_plan_update::apply_bonuses_to_object(sim, projectile, &bonuses);
    }

    // A launcher that completes a special power reports it and takes the credit; otherwise
    // the projectile completes it for the launcher.
    if sim.notify_special_power_completion(source) != FALSE {
        sim.set_special_power_creator(projectile, ObjectId::INVALID);
    } else {
        sim.set_special_power_creator(projectile, source_id);
    }

    new_projectile_fired(sim, weapon, source, projectile, victim, victim_pos);

    if sim
        .object(projectile)
        .is_some_and(|p| p.projectile_slot.is_some())
    {
        let aim = if scatter_radius > 0.0 { None } else { victim };
        let name = weapon.template.name.clone();
        sim.launch_projectile(projectile, aim, destination, source, weapon.wslot, &name);
    } else {
        sim.set_object_position(projectile, destination);
    }

    if let Some(v) = victim {
        // Only a jet that is not supersonic may drop decoys.
        if is_kind(sim, projectile, KindOfType::SmallMissile)
            && sim.object(v).is_some_and(|o| {
                o.countermeasures_slot.is_some()
                    && o.ai_slot.is_some()
                    && o.cur_locomotor_set != LocomotorSetType::Supersonic
            })
        {
            sim.report_missile_to_countermeasures(v, projectile);
        }
    }
    Some(projectile)
}

/// `Weapon::newProjectileFired`: adds the projectile to the weapon's stream object.
fn new_projectile_fired(
    sim: &mut dyn SimApi,
    weapon: &mut Weapon,
    source: ObjectSlot,
    projectile: ObjectSlot,
    victim: Option<ObjectSlot>,
    victim_pos: &Coord3D,
) {
    if weapon.template.projectile_stream_name.is_empty() {
        return;
    }
    let Some((source_id, team)) = sim.object(source).map(|o| (o.id, o.team_id)) else {
        return;
    };
    let mut stream = sim.find_object(weapon.projectile_stream_id);
    if stream.is_none() {
        weapon.projectile_stream_id = ObjectId::INVALID;
        let default_team = sim.player_default_team(team);
        stream =
            sim.new_object_from_template(&weapon.template.projectile_stream_name, default_team);
        match stream {
            Some(s) => weapon.projectile_stream_id = id_of(sim, s),
            None => return,
        }
    }
    let Some(stream) = stream else { return };
    let source_pos = position_of(sim, source);
    let projectile_id = id_of(sim, projectile);
    let victim_id = victim.map_or(ObjectId::INVALID, |v| id_of(sim, v));
    sim.add_to_projectile_stream(
        stream,
        &source_pos,
        source_id,
        projectile_id,
        victim_id,
        victim_pos,
    );
}

/// `Weapon::createLaser`: an object that only carries the beam's drawable.
fn create_laser(
    sim: &mut dyn SimApi,
    template: &WeaponTemplate,
    source: ObjectSlot,
    victim: Option<ObjectSlot>,
    victim_pos: &Coord3D,
) {
    let Some((source_id, team)) = sim.object(source).map(|o| (o.id, o.team_id)) else {
        return;
    };
    let default_team = sim.player_default_team(team);
    let Some(laser) = sim.new_object_from_template(&template.laser_name, default_team) else {
        return;
    };
    let start = position_of(sim, source);
    sim.set_object_position(laser, &start);

    let mut end = *victim_pos;
    if let Some(v) = victim {
        let airborne = sim
            .object(v)
            .is_some_and(|o| o.status.test(ObjectStatusType::AirborneTarget.as_i32()) != FALSE);
        if !is_kind(sim, v, KindOfType::Projectile) && !airborne {
            end.z += 10.0;
        }
    }
    let laser_id = id_of(sim, laser);
    let target_id = victim.map_or(ObjectId::INVALID, |v| id_of(sim, v));
    sim.client()
        .on_laser_init(laser_id, source_id, target_id, start, end);
}

/// `is2DDistSquaredLessThan`.
fn within_2d(a: &Coord3D, b: &Coord2D, dist_sqr: Real) -> bool {
    sqr(a.x - b.x) + sqr(a.y - b.y) <= dist_sqr
}

/// `WeaponTemplate::processHistoricDamage`: enough hits close together fire a bonus weapon.
fn process_historic_damage(
    sim: &mut dyn SimApi,
    template: &WeaponTemplate,
    source: Option<ObjectSlot>,
    pos: &Coord3D,
) {
    if template.historic_bonus_count <= 0 || template.historic_bonus_weapon == template.name {
        return;
    }
    let now = sim.frame();
    #[cfg(feature = "retail-crc")]
    let fire = {
        let limit = sim.rules().global_data.historic_damage_limit;
        let history = sim
            .weapon_store_state()
            .history
            .entry(template.name.clone())
            .or_default();
        let expiration = now.wrapping_sub(limit);
        let old = history
            .damage
            .iter()
            .take_while(|h| h.frame <= expiration)
            .count();
        history.damage.drain(..old);

        let radius_sqr = template.historic_bonus_radius * template.historic_bonus_radius;
        let oldest = now.wrapping_sub(template.historic_bonus_time);
        let count = history
            .damage
            .iter()
            .filter(|h| h.frame >= oldest && within_2d(pos, &h.location, radius_sqr))
            .count() as Int;
        // The list is cleared after the bonus weapon has fired, so a bonus weapon that
        // adds to it is also forgotten.
        if count >= template.historic_bonus_count - 1 {
            true
        } else {
            history.damage.push(HistoricWeaponDamageInfo {
                frame: now,
                location: Coord2D { x: pos.x, y: pos.y },
                trigger_id: 0,
            });
            false
        }
    };
    #[cfg(not(feature = "retail-crc"))]
    let fire = {
        let history = sim
            .weapon_store_state()
            .history
            .entry(template.name.clone())
            .or_default();
        // `trimOldHistoricDamage`. The subtraction wraps while the game is young.
        let expiration = now.wrapping_sub(template.historic_bonus_time);
        let old = history
            .damage
            .iter()
            .take_while(|h| h.frame <= expiration)
            .count();
        history.damage.drain(..old);

        history.trigger_id = history.trigger_id.wrapping_add(1);
        let trigger = history.trigger_id;
        let required = (template.historic_bonus_count - 1) as usize;
        let mut fire = false;
        if history.damage.len() >= required {
            let radius_sqr = template.historic_bonus_radius * template.historic_bonus_radius;
            let mut count = 0usize;
            for h in &mut history.damage {
                if within_2d(pos, &h.location, radius_sqr) {
                    h.trigger_id = trigger;
                    count += 1;
                    if count == required {
                        fire = true;
                        break;
                    }
                }
            }
        }
        if !fire {
            history.damage.push(HistoricWeaponDamageInfo {
                frame: now,
                location: Coord2D { x: pos.x, y: pos.y },
                trigger_id: 0,
            });
        }
        fire
    };

    if !fire {
        return;
    }
    // Without a source object C++ dereferences null while loading the temporary weapon.
    if let Some(source) = source {
        weapon_fire::create_and_fire_temp_weapon(sim, source, &template.historic_bonus_weapon, pos);
    }
    let Some(history) = sim.weapon_store_state().history.get_mut(&template.name) else {
        return;
    };
    #[cfg(feature = "retail-crc")]
    history.damage.clear();
    // `trimTriggeredHistoricDamage`, with whatever id is current after the bonus weapon.
    #[cfg(not(feature = "retail-crc"))]
    {
        let trigger = history.trigger_id;
        history.damage.retain(|h| h.trigger_id != trigger);
    }
}

fn new_damage(template: &WeaponTemplate, source_id: ObjectId) -> DamageInfo {
    let mut info = DamageInfo::default();
    info.input.damage_type = template.damage_type;
    info.input.death_type = template.death_type;
    info.input.source_id = source_id;
    info.input.source_player_mask = 0;
    info.input.damage_status_type = template.damage_status_type;
    info
}

/// `projectileGetLauncherID`.
fn launcher_of(sim: &mut dyn SimApi, projectile: ObjectSlot) -> Option<ObjectId> {
    match sim.weapon_store_state().detonating {
        Some((slot, launcher)) if slot == projectile => Some(launcher),
        _ => sim.projectile_launcher_id(projectile),
    }
}

/// `WeaponTemplate::dealDamageInternal`.
pub fn deal_damage_internal(
    sim: &mut dyn SimApi,
    template: &WeaponTemplate,
    source_id: ObjectId,
    victim_id: ObjectId,
    pos: &Coord3D,
    bonus: &WeaponBonus,
    is_detonation: Bool,
) {
    if source_id == ObjectId::INVALID {
        return;
    }
    // The source may be gone by now.
    let source = sim.find_object(source_id);
    process_historic_damage(sim, template, source, pos);

    let primary_victim = if victim_id == ObjectId::INVALID {
        None
    } else {
        sim.find_object(victim_id)
    };
    let pos = primary_victim.map_or(*pos, |v| position_of(sim, v));

    // Projectile weapons never get their damage dealt directly.
    if is_detonation == FALSE && has_projectile(sim, template) {
        return;
    }

    let primary_radius = template.get_primary_damage_radius(bonus);
    let secondary_radius = template.get_secondary_damage_radius(bonus);
    let primary_damage = template.get_primary_damage(bonus);
    let secondary_damage = template.get_secondary_damage(bonus);
    let affects = template.affects_mask;
    let primary_radius_sqr = sqr(primary_radius);
    // `max(primaryRadius, secondaryRadius)`.
    let radius = if primary_radius < secondary_radius {
        secondary_radius
    } else {
        primary_radius
    };

    let victims: Vec<(ObjectSlot, Real)> = if radius > 0.0 {
        sim.damage_candidates(&pos, radius)
    } else {
        if affects & WEAPON_AFFECTS_SUICIDE != 0 {
            // Without a source C++ dereferences null here.
            if let Some(source) = source {
                let mut info = new_damage(template, source_id);
                info.input.amount = HUGE_DAMAGE_AMOUNT;
                sim.attempt_damage(source, &mut info);
            }
            return;
        }
        primary_victim.map(|v| (v, 0.0)).into_iter().collect()
    };

    for (victim, dist_sqr) in victims {
        let mut kill_self = false;
        if let Some(source) = source {
            // Anything may be hurt by being the victim aimed at.
            if Some(victim) != primary_victim {
                if affects & WEAPON_AFFECTS_SUICIDE != 0 && source == victim {
                    kill_self = true;
                } else if !radius_damage_affects(sim, affects, source, victim) {
                    continue;
                }
            }
        }

        let mut info = new_damage(template, source_id);
        let mut direction = Coord3D::default();
        if let Some(source) = source {
            let to = position_of(sim, victim);
            let from = position_of(sim, source);
            direction = Coord3D::new(to.x - from.x, to.y - from.y, to.z - from.z);
        }

        let allowed_angle = template.radius_damage_angle;
        if allowed_angle < PI {
            let Some(source) = source else { continue };
            let Some(mtx) = sim.object(source).map(|o| *o.thing.get_transform_matrix()) else {
                continue;
            };
            let mut facing = Coord3D::new(mtx.rows[0][0], mtx.rows[1][0], mtx.rows[2][0]);
            let mut toward = direction;
            facing.normalize_ww_vector3();
            toward.normalize_ww_vector3();
            if facing.x * toward.x + facing.y * toward.y + facing.z * toward.z
                < math::gamemath::cos(allowed_angle)
            {
                continue;
            }
        }

        info.input.shock_wave_amount = template.shock_wave_amount;
        if info.input.shock_wave_amount > 0.0 {
            let mut shock = direction;
            if gm_fabsf(shock.x) < math::WWMATH_EPSILON
                && gm_fabsf(shock.y) < math::WWMATH_EPSILON
                && gm_fabsf(shock.z) < math::WWMATH_EPSILON
            {
                shock.z = 1.0;
            }
            info.input.shock_wave_vector = shock;
            info.input.shock_wave_radius = template.shock_wave_radius;
            info.input.shock_wave_taper_off = template.shock_wave_taper_off;
        }

        if let Some(team) = source.and_then(|s| sim.object(s)).map(|o| o.team_id) {
            info.input.source_player_mask = sim.player_mask(team);
        }

        info.input.amount = if dist_sqr <= primary_radius_sqr {
            primary_damage
        } else {
            secondary_damage
        };
        if kill_self {
            info.input.amount = HUGE_DAMAGE_AMOUNT;
        }

        // Damage dealt by a projectile is credited to its launcher.
        if let Some(source) = source {
            if is_kind(sim, source, KindOfType::Projectile) {
                if let Some(launcher) = launcher_of(sim, source) {
                    info.input.source_id = launcher;
                }
            }
        }

        sim.attempt_damage(victim, &mut info);
    }
}

/// The `RadiusDamageAffects` filters for a victim that was not aimed at.
fn radius_damage_affects(
    sim: &dyn SimView,
    affects: Int,
    source: ObjectSlot,
    victim: ObjectSlot,
) -> bool {
    let (Some(from), Some(to)) = (sim.object(source), sim.object(victim)) else {
        return false;
    };
    // The source, and whoever made it, are spared unless it is meant to hurt itself.
    if affects & WEAPON_AFFECTS_SELF == 0 && (source == victim || from.producer_id == to.id) {
        return false;
    }
    if affects & WEAPON_AFFECTS_NOT_SIMILAR != 0 {
        let factory = &sim.rules().thing_factory;
        let same = from
            .thing
            .template_override()
            .get_final(factory)
            .zip(to.thing.template_override().get_final(factory))
            .is_some_and(|(a, b)| a.is_equivalent_to(b) != FALSE);
        if same && sim.player_relationship(source, victim) == Relationship::Allies {
            return false;
        }
    }
    if affects & WEAPON_AFFECTS_NOT_AIRBORNE != 0
        && sim.object_is_significantly_above_terrain(victim) != FALSE
    {
        return false;
    }
    let required = match sim.player_relationship(victim, source) {
        Relationship::Allies => WEAPON_AFFECTS_ALLIES,
        Relationship::Enemies => WEAPON_AFFECTS_ENEMIES,
        _ => WEAPON_AFFECTS_NEUTRALS,
    };
    affects & required != 0
}

/// `WeaponStore::update`: deals the damage that is due, oldest first. Damage queued while
/// doing so is visited in the same pass.
pub fn update_delayed_damage(sim: &mut dyn SimApi) {
    let mut i = 0;
    while i < sim.weapon_store_state().delayed.len() {
        let due = sim.frame() >= sim.weapon_store_state().delayed[i].frame;
        if !due {
            i += 1;
            continue;
        }
        let entry = sim.weapon_store_state().delayed[i].clone();
        if let Some(template) = template_of(sim, &entry.weapon_name) {
            deal_damage_internal(
                sim,
                &template,
                entry.source_id,
                entry.victim_id,
                &entry.pos,
                &entry.bonus,
                FALSE,
            );
        }
        sim.weapon_store_state().delayed.remove(i);
    }
}

/// Runs `body` while `projectile`, launched by `launcher`, detonates.
pub fn while_detonating<R>(
    sim: &mut dyn SimApi,
    projectile: ObjectSlot,
    launcher: ObjectId,
    body: impl FnOnce(&mut dyn SimApi) -> R,
) -> R {
    let outer = sim
        .weapon_store_state()
        .detonating
        .replace((projectile, launcher));
    let result = body(sim);
    sim.weapon_store_state().detonating = outer;
    result
}
