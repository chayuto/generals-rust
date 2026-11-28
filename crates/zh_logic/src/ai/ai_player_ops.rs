//! Remaining `PKT-426` (`AIPlayer`) and `PKT-427` (`AISkirmishPlayer`) methods:
//! non-skirmish base building, repair queuing and bridge repair, upgrade
//! production, skill-set selection, supply-source attack/safety checks and
//! guarding, construction-zone / nearest-team placement, superweapon targeting,
//! base-defense placement, and bridge checks along waypoints.

use zh_common::math::{
    div_safe_f32, gm_cosf, gm_sinf, gm_sqrtf, real_to_int_ceil, Coord2D, Coord3D, Region2D,
    Region3D,
};
use zh_common::types::{Int, ObjectId, Real, TeamId, UnsignedInt, FALSE, TRUE};

use crate::ai::ai_group;
use crate::ai::ai_player::{
    ai_mut, build_structure_with_dozer, find_dozer, find_supply_center, is_location_safe,
    owner_of, queue_dozer, AiPlayer, FRAMES_PER_SECOND, MAX_STRUCTURES_TO_REPAIR,
    PATHFIND_CELL_SIZE_F, SUPPLY_CENTER_CLOSE_DIST,
};
use crate::ai::ai_skirmish_player::get_ai_enemy;
use crate::ai::ai_skirmish_player::EnemyView;
use crate::arena::{FacetMask, ObjectSlot, ThingTemplateId};
use crate::gamelogic::global_data::{BodyDamageType, PlayerType};
use crate::modules::module::CommandSourceType;
use crate::modules::update::ai_update::{AiCommandParms, AiCommandType, AiUpdate, DerivedAi};
use crate::modules::update::dozer_ai_update::is_task_pending;
use crate::rules::LocomotorSurfaceTypeMask;
use crate::player::build_assistant::{
    is_location_legal_to_build, legal_options, BuildContext, LegalBuildCode, PlacementQuery, PI,
};
use crate::player::player::Player;
use crate::player::player_list::PlayerList;
use crate::player::seams::PlayerIndex;
use crate::player::team::estimate_team_position;
use crate::rules::kind_of::KindOfType;
use crate::rules::object_status_types::ObjectStatusType;
use crate::rules::special_power::SpecialPowerTemplate;
use crate::rules::special_power_types::SpecialPowerType;
use crate::rules::upgrade::{UpgradePlayerAccess, UpgradeType};
use crate::sim::{SimContext, SimView};
use crate::spatial::sides_list::BuildListInfo;
use crate::terrain::terrain_logic::Waypoint;

/// `GuardMode::GUARDMODE_NORMAL` (`GameCommon.h:178`).
const GUARDMODE_NORMAL: Int = 0;

/// Finds a `KINDOF_REBUILD_HOLE` whose spawner matches `prior_id`.
pub(super) fn find_rebuild_hole(sim: &SimContext, prior_id: ObjectId) -> Option<ObjectId> {
    for slot in sim.world.objects.world_iter() {
        if sim.object_is_kind_of(slot, KindOfType::RebuildHole) == FALSE {
            continue;
        }
        let Some(obj) = sim.world.objects.get(slot) else {
            continue;
        };
        let matched = obj
            .behaviors
            .iter()
            .filter(|(_, facets)| facets.contains(FacetMask::REBUILD_HOLE))
            .any(|&(ms, _)| {
                sim.world
                    .modules
                    .payload(ms)
                    .and_then(|m| m.as_rebuild_hole())
                    .is_some_and(|rh| rh.get_spawner_id() == prior_id)
            });
        if matched {
            return Some(obj.id);
        }
    }
    None
}

/// Resets `AIPlayer` base-building timers after queuing a build list structure.
pub(super) fn reset_structure_timer(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let frame = cx.sim.world.frame;
    let data = &cx.sim.rules.ai_data;
    let (seconds, poor, wealthy, poor_mod, wealthy_mod) = (
        data.structure_seconds,
        data.resources_poor,
        data.resources_wealthy,
        data.structures_poor_mod,
        data.structures_wealthy_mod,
    );
    let money = cx
        .sim
        .world
        .player_default_teams
        .get(&player)
        .map_or(0, |&team| cx.sim.player_money(team));
    if let Some(ai) = ai_mut(cx, player) {
        ai.ready_to_build_structure = FALSE;
        ai.structure_timer = (seconds * FRAMES_PER_SECOND as Real) as Int;
        if money < poor as UnsignedInt {
            ai.structure_timer = (ai.structure_timer as Real / poor_mod) as Int;
        } else if money > wealthy as UnsignedInt {
            ai.structure_timer = (ai.structure_timer as Real / wealthy_mod) as Int;
        }
        ai.frame_last_building_built = frame as Int;
    }
}

/// `AIPlayer::processBaseBuilding` for a non-skirmish player (`AIPlayer.cpp:707-857`).
pub(super) fn process_base_building(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    if !ai_mut(cx, player).is_some_and(|ai| ai.ready_to_build_structure != FALSE) {
        return;
    }
    let frame = cx.sim.world.frame;
    let rebuild_delay = cx
        .sim
        .rules
        .ai_data
        .rebuild_delay_seconds
        .wrapping_mul(FRAMES_PER_SECOND) as UnsignedInt;
    let len = cx.players.player(player).map_or(0, |p| p.build_list.len());
    for at in 0..len {
        let Some((name, mut id)) = cx
            .players
            .player(player)
            .and_then(|p| p.build_list.get(at))
            .map(|info| (info.template_name.clone(), info.object_id))
        else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let Some(bldg_plan) = cx.sim.rules.thing_factory.find_template_id(&name) else {
            continue;
        };
        if id != ObjectId::INVALID {
            if let Some(bldg_slot) = cx.sim.world.objects.slot_by_id(id) {
                if owner_of(cx.sim, bldg_slot) == Some(player) {
                    let under_construction = cx.sim.world.objects.get(bldg_slot).is_some_and(|o| {
                        o.status
                            .test(ObjectStatusType::UnderConstruction.as_i32())
                            != FALSE
                    });
                    if under_construction {
                        let (builder_id, bldg_pos) = cx
                            .sim
                            .world
                            .objects
                            .get(bldg_slot)
                            .map_or((ObjectId::INVALID, Coord3D::default()), |o| {
                                (o.builder_id, *o.thing.get_position())
                            });
                        let mut my_dozer = cx.sim.world.objects.slot_by_id(builder_id);
                        if my_dozer.is_none() {
                            my_dozer = find_dozer(cx, player, &bldg_pos);
                            let Some(d) = my_dozer.filter(|&d| {
                                cx.sim.world.objects.get(d).and_then(|o| o.ai_slot).is_some()
                            }) else {
                                continue;
                            };
                            let mut order = AiCommandParms::new(
                                AiCommandType::ResumeConstruction,
                                CommandSourceType::FromAi,
                            );
                            order.obj = Some(id);
                            cx.sim.deliver_ai_command(d, order);
                        } else if let Some(d) = my_dozer {
                            let mut order = AiCommandParms::new(
                                AiCommandType::ResumeConstruction,
                                CommandSourceType::FromAi,
                            );
                            order.obj = Some(id);
                            cx.sim.deliver_ai_command(d, order);
                        }
                    }
                } else {
                    if let Some(info) = cx
                        .players
                        .player_mut(player)
                        .and_then(|p| p.build_list.get_mut(at))
                    {
                        info.object_id = ObjectId::INVALID;
                        info.object_timestamp = frame.wrapping_add(1);
                    }
                }
            } else {
                let hole_id = find_rebuild_hole(cx.sim, id);
                id = hole_id.unwrap_or(ObjectId::INVALID);
                if let Some(info) = cx
                    .players
                    .player_mut(player)
                    .and_then(|p| p.build_list.get_mut(at))
                {
                    info.object_id = id;
                    info.object_timestamp = frame.wrapping_add(1);
                }
            }
        }
        let Some((cur_id, cur_ts, buildable)) = cx
            .players
            .player(player)
            .and_then(|p| p.build_list.get(at))
            .map(|info| {
                (
                    info.object_id,
                    info.object_timestamp,
                    info.is_buildable() != FALSE,
                )
            })
        else {
            continue;
        };
        if cur_id == ObjectId::INVALID && cur_ts > 0 {
            if cur_ts.wrapping_add(rebuild_delay) > frame {
                continue;
            }
            if let Some(info) = cx
                .players
                .player_mut(player)
                .and_then(|p| p.build_list.get_mut(at))
            {
                info.object_timestamp = 0;
            }
        }
        if buildable && cx.sim.world.objects.slot_by_id(cur_id).is_none() {
            if build_structure_with_dozer(cx, player, bldg_plan, at).is_some() {
                if let Some(info) = cx
                    .players
                    .player_mut(player)
                    .and_then(|p| p.build_list.get_mut(at))
                {
                    info.decrement_num_rebuilds();
                }
                reset_structure_timer(cx, player);
                break;
            }
        }
    }
}

/// `AIPlayer::repairStructure` (`AIPlayer.cpp:2318-2344`).
pub fn repair_structure(sim: &SimContext, ai: &mut AiPlayer, structure: ObjectId) {
    let Some(slot) = sim.world.objects.slot_by_id(structure) else {
        return;
    };
    let Some(obj) = sim.world.objects.get(slot) else {
        return;
    };
    let Some(body_slot) = obj.body_slot else {
        return;
    };
    let Some(damage_state) = sim
        .world
        .modules
        .payload(body_slot)
        .and_then(|m| m.as_body())
        .map(|b| b.get_damage_state())
    else {
        return;
    };
    if damage_state == BodyDamageType::Pristine {
        return;
    }
    let count = (ai.structures_in_queue.max(0) as usize).min(MAX_STRUCTURES_TO_REPAIR);
    if ai.structures_to_repair[..count].contains(&obj.id) {
        return;
    }
    if count >= MAX_STRUCTURES_TO_REPAIR {
        return;
    }
    ai.structures_to_repair[count] = obj.id;
    ai.structures_in_queue += 1;
}

/// `AIPlayer::selectSkillset` (`AIPlayer.cpp:2349-2355`).
pub fn select_skillset(ai: &mut AiPlayer, skillset: Int) {
    ai.skillset_selector = skillset;
}

fn shift_repair_queue(ai: &mut AiPlayer) {
    let count = (ai.structures_in_queue.max(0) as usize).min(MAX_STRUCTURES_TO_REPAIR);
    if count == 0 {
        return;
    }
    for i in 0..count - 1 {
        ai.structures_to_repair[i] = ai.structures_to_repair[i + 1];
    }
    ai.structures_in_queue -= 1;
}

fn dozer_any_task_pending(sim: &SimContext, slot: ObjectSlot) -> Option<bool> {
    let module = sim.world.objects.get(slot)?.ai_slot?;
    let ai = sim
        .world
        .modules
        .payload(module)?
        .as_any()?
        .downcast_ref::<AiUpdate>()?;
    let brain = match &ai.derived {
        DerivedAi::Dozer(d) => &d.brain,
        DerivedAi::Worker(w) => &w.brain,
        DerivedAi::Base => return None,
    };
    Some((0..brain.tasks.len() as Int).any(|t| is_task_pending(brain, t)))
}

/// `AIPlayer::updateBridgeRepair` (`AIPlayer.cpp:2360-2448`).
pub(super) fn update_bridge_repair(cx: &mut BuildContext<'_>, player: PlayerIndex) {
    let Some(ai) = ai_mut(cx, player) else {
        return;
    };
    if ai.structures_in_queue == 0 {
        return;
    }
    ai.bridge_timer = ai.bridge_timer.wrapping_sub(1);
    if ai.bridge_timer > 0 {
        return;
    }
    ai.bridge_timer = FRAMES_PER_SECOND;

    let mut bridge_slot = None;
    loop {
        let Some(id) = ai_mut(cx, player)
            .filter(|ai| ai.structures_in_queue > 0)
            .map(|ai| ai.structures_to_repair[0])
        else {
            break;
        };
        if let Some(slot) = cx.sim.world.objects.slot_by_id(id) {
            bridge_slot = Some((id, slot));
            break;
        }
        if let Some(ai) = ai_mut(cx, player) {
            shift_repair_queue(ai);
        }
    }
    let Some((bridge_id, bridge_slot)) = bridge_slot else {
        return;
    };
    let (bridge_pos, bridge_state) = {
        let Some(obj) = cx.sim.world.objects.get(bridge_slot) else {
            return;
        };
        let state = obj
            .body_slot
            .and_then(|bs| cx.sim.world.modules.payload(bs))
            .and_then(|m| m.as_body())
            .map_or(BodyDamageType::Pristine, |b| b.get_damage_state());
        (*obj.thing.get_position(), state)
    };

    let (repair_dozer, dozer_queued_for_repair, dozer_is_repairing) = {
        let Some(ai) = ai_mut(cx, player) else {
            return;
        };
        (
            ai.repair_dozer,
            ai.dozer_queued_for_repair != FALSE,
            ai.dozer_is_repairing != FALSE,
        )
    };

    if repair_dozer == ObjectId::INVALID {
        if let Some(ai) = ai_mut(cx, player) {
            ai.dozer_is_repairing = FALSE;
        }
        if dozer_queued_for_repair {
            return;
        }
        if let Some(dozer_slot) = find_dozer(cx, player, &bridge_pos) {
            let (dozer_id, dozer_pos) = cx
                .sim
                .world
                .objects
                .get(dozer_slot)
                .map_or((ObjectId::INVALID, Coord3D::default()), |o| {
                    (o.id, *o.thing.get_position())
                });
            if let Some(ai) = ai_mut(cx, player) {
                ai.repair_dozer = dozer_id;
                ai.repair_dozer_origin = dozer_pos;
                ai.dozer_is_repairing = TRUE;
            }
            let mut order = AiCommandParms::new(AiCommandType::Repair, CommandSourceType::FromAi);
            order.obj = Some(bridge_id);
            cx.sim.deliver_ai_command(dozer_slot, order);
            return;
        }
        queue_dozer(cx, player);
        if let Some(ai) = ai_mut(cx, player) {
            ai.dozer_queued_for_repair = TRUE;
        }
        return;
    }

    let Some(dozer_slot) = cx.sim.world.objects.slot_by_id(repair_dozer) else {
        if let Some(ai) = ai_mut(cx, player) {
            ai.repair_dozer = ObjectId::INVALID;
            ai.bridge_timer = 0;
        }
        return;
    };
    let Some(any_task_pending) = dozer_any_task_pending(cx.sim, dozer_slot) else {
        return;
    };

    if dozer_is_repairing {
        if !any_task_pending {
            if bridge_state == BodyDamageType::Pristine {
                let go_home = {
                    let Some(ai) = ai_mut(cx, player) else {
                        return;
                    };
                    shift_repair_queue(ai);
                    ai.dozer_is_repairing = FALSE;
                    if ai.structures_in_queue == 0 {
                        Some(if ai.base_center_set != FALSE {
                            ai.base_center
                        } else {
                            ai.repair_dozer_origin
                        })
                    } else {
                        None
                    }
                };
                if let Some(mut pos) = go_home {
                    cx.sim
                        .adjust_to_possible_destination_on_map(dozer_slot, &mut pos);
                    let mut order = AiCommandParms::new(
                        AiCommandType::MoveToPosition,
                        CommandSourceType::FromAi,
                    );
                    order.pos = pos;
                    cx.sim.deliver_ai_command(dozer_slot, order);
                    return;
                }
            }
        } else {
            return;
        }
    }

    let mut order = AiCommandParms::new(AiCommandType::Repair, CommandSourceType::FromAi);
    order.obj = Some(bridge_id);
    cx.sim.deliver_ai_command(dozer_slot, order);
    if let Some(ai) = ai_mut(cx, player) {
        ai.dozer_is_repairing = TRUE;
    }
}

struct UpgradeMoneyView(UnsignedInt);

impl UpgradePlayerAccess for UpgradeMoneyView {
    fn money_count(&self) -> UnsignedInt {
        self.0
    }
}

/// `AIPlayer::buildUpgrade` (`AIPlayer.cpp:1786-1878`).
pub fn build_upgrade(
    sim: &mut SimContext,
    player: PlayerIndex,
    build_list: &[BuildListInfo],
    upgrade: &str,
) {
    let (upgrade_type, key, upgrade_name, mask, cost) = {
        let Some(tmpl) = sim
            .rules
            .upgrade_center
            .find_upgrade(upgrade, &mut sim.rules.name_keys)
        else {
            return;
        };
        let money = sim
            .world
            .player_money
            .get(&player)
            .copied()
            .unwrap_or(0);
        let access = UpgradeMoneyView(money);
        (
            tmpl.upgrade_type(),
            tmpl.upgrade_name_key(),
            tmpl.upgrade_name().to_owned(),
            tmpl.upgrade_mask().clone(),
            tmpl.calc_cost_to_build(Some(&access)),
        )
    };
    if upgrade_type == UpgradeType::Object {
        return;
    }
    let Some(&default_team) = sim.world.player_default_teams.get(&player) else {
        return;
    };
    if sim.player_has_upgrade_in_production(default_team, &mask) != FALSE {
        return;
    }
    if sim.player_has_upgrade(default_team, &mask) != FALSE {
        return;
    }
    if (sim.player_money(default_team) as i64) < (cost as i64) {
        return;
    }
    for info in build_list {
        let Some(factory) = sim.world.objects.slot_by_id(info.object_id) else {
            continue;
        };
        let Some(obj) = sim.world.objects.get(factory) else {
            continue;
        };
        if obj
            .status
            .test(ObjectStatusType::UnderConstruction.as_i32())
            != FALSE
            || obj.status.test(ObjectStatusType::Sold.as_i32()) != FALSE
        {
            continue;
        }
        let can_upgrade_here = sim.command_set_of(factory).is_some_and(|buttons| {
            buttons
                .iter()
                .any(|b| !b.upgrade.is_empty() && b.upgrade == upgrade_name)
        });
        if !can_upgrade_here {
            continue;
        }
        if sim.with_production(factory, |pu, sim, ctx| {
            pu.queue_upgrade(sim, ctx, key) != FALSE
        }) == Some(true)
        {
            return;
        }
    }
}

/// Shared wiggle search preserving the exact C++ `AIPlayer.cpp:2031-2106` / `2143-2206`
/// initial `valid = false` quirk and missing `if (valid) break;` after the second inner test.
fn wiggle_construction_zone(
    sim: &SimContext,
    what: ThingTemplateId,
    angle: Real,
    skirmish_ai: bool,
    pathfind_extent: &Region3D,
    location: &Coord3D,
) -> (bool, Coord3D) {
    use legal_options::*;
    let legal = |at: &Coord3D, options| {
        is_location_legal_to_build(
            sim,
            &PlacementQuery {
                pos: at,
                what,
                angle,
                options,
                builder: None,
                skirmish_ai,
                pathfind_extent,
            },
        ) == LegalBuildCode::Ok
    };
    let mut valid = false;
    let mut new_pos = *location;
    if !legal(location, NO_OBJECT_OVERLAP) {
        let full = CLEAR_PATH | TERRAIN_RESTRICTIONS | NO_OBJECT_OVERLAP;
        let cell = PATHFIND_CELL_SIZE_F;
        let mut pos_offset: Real = 0.0;
        while pos_offset < 2.0 * SUPPLY_CENTER_CLOSE_DIST {
            let offset = pos_offset / 2.0;
            let y_pos = location.y - offset;
            let mut x_pos = location.x - offset;
            while x_pos <= location.x + offset {
                new_pos.x = x_pos;
                new_pos.y = y_pos;
                valid = legal(&new_pos, full);
                if valid {
                    break;
                }
                new_pos.y = y_pos + pos_offset;
                valid = legal(&new_pos, full);
                x_pos += cell;
            }
            if valid {
                break;
            }
            let x_pos = location.x - offset;
            let mut y_pos = location.y - offset;
            while y_pos <= location.y + offset {
                new_pos.x = x_pos;
                new_pos.y = y_pos;
                valid = legal(&new_pos, full);
                if valid {
                    break;
                }
                new_pos.x = x_pos + pos_offset;
                valid = legal(&new_pos, full);
                y_pos += cell;
            }
            if valid {
                break;
            }
            pos_offset += 2.0 * cell;
        }
    }
    (valid, new_pos)
}

/// `AIPlayer::calcClosestConstructionZoneLocation` (`AIPlayer.cpp:2013-2112`).
pub fn calc_closest_construction_zone_location(
    sim: &SimContext,
    ai: &AiPlayer,
    template_name: &str,
    location: &mut Coord3D,
    pathfind_extent: &Region3D,
) -> bool {
    let Some(what) = sim.rules.thing_factory.find_template_id(template_name) else {
        return false;
    };
    let Some(tmpl) = sim.rules.thing_factory.find_template_final(template_name) else {
        return false;
    };
    let mut offset = Coord2D {
        x: location.x - ai.base_center.x,
        y: location.y - ai.base_center.y,
    };
    offset.normalize();
    let angle = tmpl.get_placement_view_angle();
    let (valid, new_pos) = wiggle_construction_zone(
        sim,
        what,
        angle,
        ai.skirmish.is_some(),
        pathfind_extent,
        location,
    );
    if valid {
        *location = new_pos;
        true
    } else {
        *location = Coord3D::default();
        false
    }
}

/// `AIPlayer::buildSpecificBuildingNearestTeam` (`AIPlayer.cpp:2117-2215`).
pub fn build_specific_building_nearest_team(
    sim: &SimContext,
    ai: &AiPlayer,
    player: &mut Player,
    thing_name: &str,
    team: TeamId,
    pathfind_extent: &Region3D,
) {
    let Some(what) = sim.rules.thing_factory.find_template_id(thing_name) else {
        return;
    };
    let Some(tmpl) = sim.rules.thing_factory.find_template_final(thing_name) else {
        return;
    };
    let Some(location) = estimate_team_position(sim, team) else {
        return;
    };
    let mut offset = Coord2D {
        x: location.x - ai.base_center.x,
        y: location.y - ai.base_center.y,
    };
    offset.normalize();
    let angle = tmpl.get_placement_view_angle();
    let (valid, mut new_pos) = wiggle_construction_zone(
        sim,
        what,
        angle,
        ai.skirmish.is_some(),
        pathfind_extent,
        &location,
    );
    if valid {
        new_pos.z = 0.0;
        add_to_priority_build_list(player, thing_name, &new_pos, angle);
    }
}

/// `Player::addToPriorityBuildList` (`Player.cpp:656-666`).
pub(super) fn add_to_priority_build_list(
    player: &mut Player,
    template_name: &str,
    location: &Coord3D,
    angle: Real,
) {
    let mut info = BuildListInfo::new();
    info.template_name = template_name.to_owned();
    info.location = *location;
    info.angle = angle;
    info.priority_build = TRUE;
    info.num_rebuilds = 1;
    player.build_list.insert(0, info);
}

/// `AIPlayer::isSupplySourceAttacked` (`AIPlayer.cpp:936-997`).
pub fn is_supply_source_attacked(sim: &SimContext, ai: &mut AiPlayer) -> bool {
    const REFRESH_RATE: UnsignedInt = 10;
    let scan_window: UnsignedInt = if cfg!(feature = "retail-crc") {
        10
    } else {
        10 * FRAMES_PER_SECOND as UnsignedInt
    };
    let cur_frame = sim.world.frame;
    if cur_frame == 0 {
        ai.supply_source_attack_check_frame = cur_frame.wrapping_add(REFRESH_RATE);
        return false;
    }
    ai.attacked_supply_center = ObjectId::INVALID;
    if cur_frame < ai.supply_source_attack_check_frame {
        return false;
    }
    let attacked_frame = sim
        .world
        .player_attacked_frame
        .get(&ai.player)
        .copied()
        .unwrap_or(0);
    if attacked_frame.wrapping_add(scan_window) < cur_frame {
        return false;
    }
    ai.supply_source_attack_check_frame = cur_frame.wrapping_add(REFRESH_RATE);

    for slot in sim.player_object_slots(ai.player) {
        let kind = |k| sim.object_is_kind_of(slot, k) != FALSE;
        if !kind(KindOfType::CashGenerator)
            && !kind(KindOfType::Dozer)
            && !kind(KindOfType::Harvester)
        {
            continue;
        }
        let Some(obj) = sim.world.objects.get(slot) else {
            continue;
        };
        let Some(body) = obj
            .body_slot
            .and_then(|bs| sim.world.modules.payload(bs))
            .and_then(|m| m.as_body())
        else {
            continue;
        };
        let Some(info) = body.get_last_damage_info() else {
            continue;
        };
        if info.output.no_effect != FALSE {
            continue;
        }
        if let Some(ts) = body.get_last_damage_timestamp() {
            if ts.wrapping_add(scan_window) > cur_frame {
                ai.attacked_supply_center = obj.id;
                return true;
            }
        }
    }
    false
}

/// `AIPlayer::isSupplySourceSafe` (`AIPlayer.cpp:1002-1007`).
pub fn is_supply_source_safe(order: &crate::ai::ai_player::SupplyOrder<'_>, min_supplies: Int) -> bool {
    let Some(warehouse) = find_supply_center(order, min_supplies) else {
        return true;
    };
    let Some(obj) = order.sim.world.objects.get(warehouse) else {
        return true;
    };
    let Some(tmpl) = obj
        .thing
        .template_override()
        .get_final(&order.sim.rules.thing_factory)
    else {
        return true;
    };
    is_location_safe(
        order.sim,
        order.ai.player,
        obj.thing.get_position(),
        tmpl.get_template_geometry_info().get_bounding_circle_radius(),
    )
}

/// `AIPlayer::guardSupplyCenter` (`AIPlayer.cpp:893-931`).
pub fn guard_supply_center(
    sim: &mut SimContext,
    ai: &mut AiPlayer,
    player: &mut Player,
    enemy_center: Option<Coord2D>,
    pathfind_extent: Region3D,
    team: TeamId,
    min_supplies: Int,
) {
    ai.supply_source_attack_check_frame = 0;
    let attacked = if is_supply_source_attacked(sim, ai) {
        sim.world.objects.slot_by_id(ai.attacked_supply_center)
    } else {
        None
    };
    let warehouse = attacked.or_else(|| {
        let order = crate::ai::ai_player::SupplyOrder {
            sim,
            ai,
            player,
            enemy_center,
            pathfind_extent,
        };
        find_supply_center(&order, min_supplies)
    });
    let Some(warehouse) = warehouse else {
        return;
    };
    let Some((mut location, radius)) = sim.world.objects.get(warehouse).map(|o| {
        (
            *o.thing.get_position(),
            o.geometry.get_bounding_circle_radius() * 0.8,
        )
    }) else {
        return;
    };
    let enemy = enemy_center.unwrap_or_default();
    let mut offset = Coord3D {
        x: location.x - enemy.x,
        y: location.y - enemy.y,
        z: 0.0,
    };
    offset.normalize();
    location.x -= offset.x * radius;
    location.y -= offset.y * radius;

    let group = ai_group::create_group(sim);
    let members: Vec<ObjectId> = sim
        .world
        .team_members
        .get(&team)
        .map(|slots| {
            slots
                .iter()
                .filter_map(|&s| sim.world.objects.id_of(s))
                .collect()
        })
        .unwrap_or_default();
    for id in members {
        ai_group::add(sim, group, id);
    }
    ai_group::group_guard_position(
        sim,
        group,
        &location,
        GUARDMODE_NORMAL,
        CommandSourceType::FromScript,
    );
}

/// `AIPlayer::getPlayerStructureBounds` (`AIPlayer.cpp:3830-3899`).
pub fn get_player_structure_bounds(
    sim: &SimContext,
    player: PlayerIndex,
    conservative: bool,
) -> Region2D {
    let mut bounds = Region2D::default();
    let mut first = true;
    for slot in sim.player_object_slots(player) {
        if sim.object_is_kind_of(slot, KindOfType::Structure) == FALSE {
            continue;
        }
        if conservative && sim.object_is_kind_of(slot, KindOfType::ConservativeBuilding) != FALSE {
            continue;
        }
        let Some(pos) = sim.world.objects.get(slot).map(|o| *o.thing.get_position()) else {
            continue;
        };
        if first {
            bounds.lo = Coord2D { x: pos.x, y: pos.y };
            bounds.hi = Coord2D { x: pos.x, y: pos.y };
            first = false;
        } else {
            let p = Coord2D { x: pos.x, y: pos.y };
            bounds.unite_with(&Region2D { lo: p, hi: p });
        }
    }
    bounds
}

/// `AIPlayer::getPlayerSuperweaponValue` (`AIPlayer.cpp:1327-1411`).
pub fn get_player_superweapon_value(
    sim: &SimContext,
    center: &Coord3D,
    player: PlayerIndex,
    radius: Real,
    include_military_units: bool,
) -> Int {
    let radius = if radius < 4.0 * PATHFIND_CELL_SIZE_F {
        4.0 * PATHFIND_CELL_SIZE_F
    } else {
        radius
    };
    let rad_sqr = radius * radius;
    let mut cash: Real = 0.0;
    for slot in sim.player_object_slots(player) {
        let kind = |k| sim.object_is_kind_of(slot, k) != FALSE;
        let mut apply_neg_value = false;
        if !include_military_units {
            if kind(KindOfType::FsBaseDefense) || kind(KindOfType::TechBaseDefense) {
                apply_neg_value = true;
            } else if (kind(KindOfType::Vehicle) || kind(KindOfType::Infantry))
                && !kind(KindOfType::Dozer)
                && !kind(KindOfType::Harvester)
            {
                apply_neg_value = true;
            }
        } else if kind(KindOfType::Aircraft) && sim.object_is_significantly_above_terrain(slot) != FALSE {
            continue;
        }
        let Some(obj) = sim.world.objects.get(slot) else {
            continue;
        };
        let pos = *obj.thing.get_position();
        let dx = center.x - pos.x;
        let dy = center.y - pos.y;
        let dist_sqr = dx * dx + dy * dy;
        if dist_sqr < rad_sqr {
            let dist = gm_sqrtf(dist_sqr);
            let factor = 1.0 - (dist / (2.0 * radius));
            let mut value = obj
                .thing
                .template_override()
                .get_final(&sim.rules.thing_factory)
                .map_or(0, |t| {
                    sim.unit_cost_to_build(obj.team_id, ThingTemplateId(t.get_template_id() as u32))
                }) as Real;
            if kind(KindOfType::CommandCenter) {
                if !include_military_units {
                    value *= 5.0;
                } else {
                    value /= 10.0;
                }
            }
            if kind(KindOfType::FsSuperweapon) {
                if !include_military_units {
                    value *= 5.0;
                } else {
                    value /= 10.0;
                }
            }
            if apply_neg_value {
                cash -= factor * value * 5.0;
            } else {
                cash += factor * value;
            }
        }
    }
    cash as Int
}

/// `AIPlayer::computeSuperweaponTarget` (`AIPlayer.cpp:1174-1321`).
pub fn compute_superweapon_target(
    sim: &mut SimContext,
    power: &SpecialPowerTemplate,
    target_player: PlayerIndex,
    mut weapon_radius: Real,
) -> Option<Coord3D> {
    let mut bounds = get_player_structure_bounds(sim, target_player, false);
    if bounds.hi.x == 0.0 && bounds.lo.x == 0.0 && bounds.hi.y == 0.0 && bounds.lo.y == 0.0 {
        let extent = sim
            .terrain
            .terrain_logic()
            .map_or_else(Region3D::default, |t| t.extent());
        bounds.hi.x = extent.hi.x;
        bounds.lo.x = extent.lo.x;
        bounds.hi.y = extent.hi.y;
        bounds.lo.y = extent.lo.y;
    }
    if weapon_radius < 1.0 {
        weapon_radius = 1.0;
    }
    bounds.lo.x += weapon_radius;
    bounds.hi.x -= weapon_radius;
    if bounds.hi.x < bounds.lo.x {
        let mid = (bounds.hi.x + bounds.lo.x) / 2.0;
        bounds.hi.x = mid;
        bounds.lo.x = mid;
    }
    if bounds.hi.y < bounds.lo.y {
        let mid = (bounds.hi.y + bounds.lo.y) / 2.0;
        bounds.hi.y = mid;
        bounds.lo.y = mid;
    }
    let x_count = (real_to_int_ceil(bounds.width() / weapon_radius) + 1).min(10);
    let y_count = (real_to_int_ceil(bounds.height() / weapon_radius) + 1).min(10);
    let target_military_units = power.special_power_type() != SpecialPowerType::SneakAttack;

    let (x_delta, y_delta, x_start, y_start) = match sim.world.rng.get_game_logic_random_value(1, 4)
    {
        1 => (1, 1, 0, 0),
        2 => (-1, 1, x_count, 0),
        3 => (1, -1, 0, y_count),
        _ => (-1, -1, x_count, y_count),
    };

    let mut cash = -1;
    let mut best_pos = Coord3D::default();
    let mut success = false;
    let mut x_index = x_start;
    for _ in 0..x_count {
        let mut y_index = y_start;
        for _ in 0..y_count {
            let pos = Coord3D {
                x: bounds.lo.x + (bounds.width() * x_index as Real) / x_count as Real,
                y: bounds.lo.y + (bounds.height() * y_index as Real) / y_count as Real,
                z: 0.0,
            };
            let cur_cash = get_player_superweapon_value(
                sim,
                &pos,
                target_player,
                2.0 * weapon_radius,
                target_military_units,
            );
            if cur_cash > cash {
                cash = cur_cash;
                best_pos = pos;
                success = true;
            }
            y_index += y_delta;
        }
        x_index += x_delta;
    }
    if !success {
        return None;
    }

    let mut very_best_pos = Coord3D::default();
    cash = -1;
    let mut count = 0;
    for x in 0..11 {
        for _y in 0..11 {
            // Exact C++ quirk (`AIPlayer.cpp:1291`): both x and y use `(x - 5)`.
            let pos = Coord3D {
                x: best_pos.x + (x - 5) as Real * (weapon_radius / 10.0),
                y: best_pos.y + (x - 5) as Real * (weapon_radius / 10.0),
                z: 0.0,
            };
            let cur_cash = get_player_superweapon_value(
                sim,
                &pos,
                target_player,
                weapon_radius,
                target_military_units,
            );
            if cur_cash > cash {
                cash = cur_cash;
                very_best_pos = pos;
                count = 1;
            } else if cur_cash == cash {
                very_best_pos.x += pos.x;
                very_best_pos.y += pos.y;
                count += 1;
            }
        }
    }
    if count > 1 {
        very_best_pos.x /= count as Real;
        very_best_pos.y /= count as Real;
    }
    very_best_pos.z = sim
        .terrain
        .get_ground_height(very_best_pos.x, very_best_pos.y, None);
    (cash > -1).then_some(very_best_pos)
}

/// `AISkirmishPlayer::getMyEnemyPlayerIndex` (`AISkirmishPlayer.cpp:463-475`).
pub(super) fn skirmish_enemy_index(
    ai: &AiPlayer,
    players: &PlayerList,
) -> PlayerIndex {
    if let Some(enemy) = ai.skirmish.as_ref().and_then(|s| s.current_enemy) {
        return enemy;
    }
    for i in 0..players.player_count() {
        if players
            .player(i)
            .is_some_and(|p| p.player_type == PlayerType::Human)
        {
            return i;
        }
    }
    players.player_count()
}

/// `AISkirmishPlayer::computeSuperweaponTarget` (`AISkirmishPlayer.cpp:1134-1178`).
pub fn compute_superweapon_target_skirmish(
    sim: &mut SimContext,
    ai: &AiPlayer,
    players: &PlayerList,
    player: &Player,
    power: &SpecialPowerTemplate,
    target_player: PlayerIndex,
    weapon_radius: Real,
) -> Option<Coord3D> {
    let _bounds = get_player_structure_bounds(sim, target_player, false);
    if matches!(
        power.special_power_type(),
        SpecialPowerType::ClusterMines | SpecialPowerType::NukeClusterMines
    ) {
        let mode = sim.world.rng.get_game_logic_random_value(0, 2);
        let n = player.mp_start_index + 1;
        let path_label = match mode {
            1 => format!("Flank{n}"),
            2 => format!("Backdoor{n}"),
            _ => format!("Center{n}"),
        };
        let mut goal_pos = ai.base_center;
        let waypoint_loc = sim
            .terrain
            .terrain_logic()
            .and_then(|t| t.closest_waypoint_on_path(&goal_pos, &path_label))
            .map(|w| w.location);
        if let Some(loc) = waypoint_loc {
            goal_pos = loc;
        } else {
            let bounds = get_player_structure_bounds(sim, skirmish_enemy_index(ai, players), false);
            goal_pos.x = bounds.lo.x + bounds.width() / 2.0;
            goal_pos.y = bounds.lo.y + bounds.height() / 2.0;
        }
        let mut offset = Coord2D {
            x: goal_pos.x - ai.base_center.x,
            y: goal_pos.y - ai.base_center.y,
        };
        offset.normalize();
        offset.x *= ai.base_radius;
        offset.y *= ai.base_radius;
        let mut ret = ai.base_center;
        ret.x += offset.x;
        ret.y += offset.y;
        ret.z = sim.terrain.get_ground_height(ret.x, ret.y, None);
        return Some(ret);
    }
    compute_superweapon_target(sim, power, target_player, weapon_radius)
}

/// `AISkirmishPlayer::buildAIBaseDefense` (`AISkirmishPlayer.cpp:561-575`).
pub fn build_ai_base_defense(
    sim: &SimContext,
    ai: &mut AiPlayer,
    players: &mut PlayerList,
    player_idx: PlayerIndex,
    flank: bool,
    pathfind_extent: &Region3D,
) {
    if ai.skirmish.is_none() {
        return;
    }
    let Some(side) = players.player(player_idx).map(|p| p.side.as_str()) else {
        return;
    };
    let Some(defense) = sim
        .rules
        .ai_data
        .side_info
        .iter()
        .find(|info| info.side == side)
        .map(|info| info.base_defense_structure1.clone())
    else {
        return;
    };
    build_ai_base_defense_structure(sim, ai, players, player_idx, &defense, flank, pathfind_extent);
}

/// `AISkirmishPlayer::buildAIBaseDefenseStructure` (`AISkirmishPlayer.cpp:599-709`).
pub fn build_ai_base_defense_structure(
    sim: &SimContext,
    ai: &mut AiPlayer,
    players: &mut PlayerList,
    player_idx: PlayerIndex,
    thing_name: &str,
    flank: bool,
    pathfind_extent: &Region3D,
) {
    use legal_options::*;
    if ai.skirmish.is_none() {
        return;
    }
    let Some(what) = sim.rules.thing_factory.find_template_id(thing_name) else {
        return;
    };
    let Some(tmpl) = sim.rules.thing_factory.find_template_final(thing_name) else {
        return;
    };
    let Some(mp_start_index) = players.player(player_idx).map(|p| p.mp_start_index) else {
        return;
    };
    let enemy_ndx = skirmish_enemy_index(ai, players);
    let structure_radius = tmpl
        .get_template_geometry_info()
        .get_bounding_circle_radius();
    let place_angle = tmpl.get_placement_view_angle();
    let defense_distance = ai.base_radius + sim.rules.ai_data.skirmish_base_defense_extra_distance;
    let base_circumference = 2.0 * PI * defense_distance;
    let angle_offset = 2.0 * PI * div_safe_f32(structure_radius * 4.0, base_circumference);
    let n = mp_start_index + 1;

    loop {
        let (flank_count, front_count) = {
            let s = ai.skirmish.as_ref().expect("checked above");
            (s.cur_flank_base_defense, s.cur_front_base_defense)
        };
        let path_label = if flank {
            if flank_count & 1 != 0 {
                format!("Flank{n}")
            } else {
                format!("Backdoor{n}")
            }
        } else {
            format!("Center{n}")
        };
        let mut goal_pos = ai.base_center;
        let way_loc = sim
            .terrain
            .terrain_logic()
            .and_then(|t| t.closest_waypoint_on_path(&goal_pos, &path_label))
            .map(|w| w.location);
        if let Some(loc) = way_loc {
            goal_pos = loc;
        } else {
            if flank {
                return;
            }
            let bounds = get_player_structure_bounds(sim, enemy_ndx, false);
            goal_pos.x = bounds.lo.x + bounds.width() / 2.0;
            goal_pos.y = bounds.lo.y + bounds.height() / 2.0;
        }
        let mut offset = Coord2D {
            x: goal_pos.x - ai.base_center.x,
            y: goal_pos.y - ai.base_center.y,
        };
        offset.normalize();
        offset.x *= defense_distance;
        offset.y *= defense_distance;

        let angle = {
            let s = ai.skirmish.as_mut().expect("checked above");
            if flank {
                let selector = s.cur_flank_base_defense >> 1;
                if s.cur_flank_base_defense & 1 != 0 {
                    if selector & 1 != 0 {
                        s.cur_left_flank_right_defense_angle -= angle_offset;
                        s.cur_left_flank_right_defense_angle
                    } else {
                        let a = s.cur_left_flank_left_defense_angle;
                        s.cur_left_flank_left_defense_angle += angle_offset;
                        a
                    }
                } else if selector & 1 != 0 {
                    s.cur_right_flank_right_defense_angle -= angle_offset;
                    s.cur_right_flank_right_defense_angle
                } else {
                    let a = s.cur_right_flank_left_defense_angle;
                    s.cur_right_flank_left_defense_angle += angle_offset;
                    a
                }
            } else {
                let selector = front_count;
                if selector & 1 != 0 {
                    s.cur_front_right_defense_angle -= angle_offset;
                    s.cur_front_right_defense_angle
                } else {
                    let a = s.cur_front_left_defense_angle;
                    s.cur_front_left_defense_angle += angle_offset;
                    a
                }
            }
        };

        if angle > PI / 3.0 {
            break;
        }
        let s = gm_sinf(angle);
        let c = gm_cosf(angle);
        let mut build_pos = ai.base_center;
        build_pos.x += offset.x * c - offset.y * s;
        build_pos.y += offset.y * c + offset.x * s;

        let can_build = is_location_legal_to_build(
            sim,
            &PlacementQuery {
                pos: &build_pos,
                what,
                angle: place_angle,
                options: TERRAIN_RESTRICTIONS | NO_OBJECT_OVERLAP,
                builder: None,
                skirmish_ai: true,
                pathfind_extent,
            },
        ) == LegalBuildCode::Ok;

        if let Some(sk) = ai.skirmish.as_mut() {
            if flank {
                sk.cur_flank_base_defense += 1;
            } else {
                sk.cur_front_base_defense += 1;
            }
        }
        if can_build {
            if let Some(p) = players.player_mut(player_idx) {
                add_to_priority_build_list(p, thing_name, &build_pos, place_angle);
            }
            break;
        }
    }
}

/// `AISkirmishPlayer::checkBridges` (`AISkirmishPlayer.cpp:716-735`).
pub fn check_bridges(
    sim: &SimContext,
    ai: &mut AiPlayer,
    unit: ObjectSlot,
    start_waypoint: Option<&Waypoint>,
) -> bool {
    if ai.skirmish.is_none() {
        return false;
    }
    let (Some(obj), Some(pathfinder), Some(terrain)) = (
        sim.world.objects.get(unit),
        sim.world.pathfinder.as_deref(),
        sim.terrain.terrain_logic(),
    ) else {
        return false;
    };
    let Some(ai_module) = obj.ai_slot else {
        return false;
    };
    let Some(unit_ai) = sim
        .world
        .modules
        .payload(ai_module)
        .and_then(|m| m.as_any())
        .and_then(|a| a.downcast_ref::<AiUpdate>())
    else {
        return false;
    };
    let unit_pos = *obj.thing.get_position();
    let loco_set: LocomotorSurfaceTypeMask = unit_ai.locomotor_set.get_valid_surfaces();
    let Some(start) = start_waypoint else {
        return false;
    };
    for cur_way in terrain
        .waypoints()
        .skip_while(|w| w.id != start.id)
    {
        if pathfinder.quick_path_exists(sim.terrain(), loco_set, &unit_pos, &cur_way.location) {
            continue;
        }
        if let Some(broken_bridge) =
            pathfinder.find_broken_bridge(sim.terrain(), loco_set, &unit_pos, &cur_way.location)
        {
            repair_structure(sim, ai, broken_bridge);
            return true;
        }
    }
    false
}

/// Resolves the `LiveWorld`-style enemy center for `guard_supply_center` without
/// touching `sim.world.ai_players` when `ai_players` is temporarily borrowed elsewhere.
pub fn supply_guard_enemy_center(
    sim: &SimContext,
    ai_players: &mut std::collections::BTreeMap<Int, AiPlayer>,
    view: &dyn EnemyView,
    player: PlayerIndex,
) -> Option<Coord2D> {
    let enemy = get_ai_enemy(ai_players, view, sim.world.frame, player)?;
    let bounds = get_player_structure_bounds(sim, enemy, false);
    Some(Coord2D {
        x: (bounds.lo.x + bounds.hi.x) * 0.5,
        y: (bounds.lo.y + bounds.hi.y) * 0.5,
    })
}
