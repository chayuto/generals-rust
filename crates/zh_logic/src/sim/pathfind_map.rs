//! The sim's side of the pathfind map (`PKT-411`): what the pathfinder reads
//! from an object, and the calls that put objects on the map and take them off.
//!
//! Without a pathfinder (`SimWorld::pathfinder` is `None`) these do nothing.
//!
//! Golden test of the map itself: `crates/zh_logic/tests/golden_s2_pathfind_map.rs`;
//! of the reservations and destination searches:
//! `crates/zh_logic/tests/golden_s4_reservations.rs`. How the sim feeds them is
//! tested in `crates/zh_logic/tests/pathfind_reservations_sim.rs`.

use zh_common::math::{Coord3D, ICoord2D, Region3D};
use zh_common::types::{Int, ObjectId, Real, Relationship, FALSE};

use super::{partition_object, SimApi, SimContext, SimView};
use crate::spatial::partition_query::{DistanceCalculationType, QueryCenter};
use crate::arena::ObjectSlot;
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::combat::locomotor::LocomotorAppearance;
use crate::gamelogic::global_data::{BodyDamageType, PlayerType};
use crate::modules::update::ai_update::AiUpdate;
use crate::pathfinder::ai_pathfind::{
    AdjustUnit, Footprint, GoalMover, GoalObject, NoPathCost, PathAgent, PathUnit, PathUnits,
    PathWorld, Pathfinder, TallBuilding, TallBuildings, TerrainSight, ViewParty,
};
use crate::object::weapon_queries;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::rules::combat_types::{DamageType, DeathType, LocomotorSurfaceTypeMask};
use crate::rules::disabled_types::DisabledType;
use crate::rules::kind_of::KindOfType;

impl PathWorld for SimContext {
    fn find_object(&self, id: ObjectId) -> Option<Footprint> {
        self.footprint(self.world.objects.slot_by_id(id)?)
    }

    fn objects_on_wall_layer(&self) -> Vec<(ObjectId, Coord3D)> {
        self.world
            .objects
            .world_iter()
            .filter_map(|slot| {
                let obj = self.world.objects.get(slot)?;
                let id = self.world.objects.id_of(slot)?;
                (obj.layer == PathfindLayerEnum::Wall).then(|| (id, *obj.thing.get_position()))
            })
            .collect()
    }
}

impl SimContext {
    /// What the pathfinder reads from the object in `slot`.
    pub fn footprint(&self, slot: ObjectSlot) -> Option<Footprint> {
        let id = self.world.objects.id_of(slot)?;
        let obj = self.world.objects.get(slot)?;
        let template = obj
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory);
        let kind = |k| self.object_is_kind_of(slot, k) != FALSE;
        Some(Footprint {
            id,
            position: *obj.thing.get_position(),
            orientation: obj.thing.get_orientation(),
            geometry: obj.geometry.clone(),
            fence_width: template.map_or(0.0, |t| t.get_fence_width()),
            fence_x_offset: template.map_or(0.0, |t| t.get_fence_x_offset()),
            mine: kind(KindOfType::Mine),
            projectile: kind(KindOfType::Projectile),
            bridge_tower: kind(KindOfType::BridgeTower),
            defensive_wall: kind(KindOfType::DefensiveWall),
            blast_crater: kind(KindOfType::BlastCrater),
            walk_on_top_of_wall: kind(KindOfType::WalkOnTopOfWall),
            structure: kind(KindOfType::Structure),
            see_through: kind(KindOfType::CanSeeThroughStructure),
            mobile: !kind(KindOfType::Immobile) && obj.is_disabled() == FALSE,
            height_above_terrain: self.object_height_above_terrain(slot),
            rubble: self.object_body_damage_state(slot) == BodyDamageType::Rubble,
        })
    }

    /// What a line of sight check reads from the object in `slot`.
    pub(super) fn view_party_of(&self, slot: ObjectSlot) -> Option<ViewParty> {
        let objects = &self.world.objects;
        let id = objects.id_of(slot)?;
        let obj = objects.get(slot)?;
        let kind = self.object_is_kind_of(slot, KindOfType::AttackNeedsLineOfSight);
        let slaver = obj
            .slaved_slot
            .and_then(|ms| self.world.modules.payload(ms)?.as_slaved_update())
            .map(|slaved| slaved.get_slaver_id());
        Some(ViewParty {
            id,
            position: *obj.thing.get_position(),
            layer: obj.layer,
            container: obj
                .contained_by
                .and_then(|c| objects.id_of(c))
                .unwrap_or(ObjectId::INVALID),
            slaver: slaver.unwrap_or(ObjectId::INVALID),
            needs_line_of_sight: kind != FALSE,
            significantly_above_terrain: self.object_is_significantly_above_terrain(slot) != FALSE,
        })
    }

    /// The start of `processPathfindQueue`: whether the pathfinder's queue is
    /// served this frame. Needs a pathfinder.
    pub(crate) fn begin_path_queue_on_map(&mut self) -> bool {
        let frame = self.world.frame;
        let extent = self.terrain.playable_extent();
        let Some(pathfinder) = self.world.pathfinder.as_deref_mut() else {
            return false;
        };
        let extent = extent.unwrap_or_else(|| {
            let hi = pathfinder.extent().hi;
            let edge = |cell: i32| (cell + 1) as Real * PATHFIND_CELL_SIZE_F;
            Region3D {
                lo: Coord3D::default(),
                hi: Coord3D::new(edge(hi.x), edge(hi.y), 0.0),
            }
        });
        pathfinder.begin_path_queue(&extent, frame)
    }

    /// `TheAI->pathfinder()->newMap()` over the sim's terrain and objects,
    /// creating the pathfinder if there is none. `extent` is
    /// `getMaximumPathfindExtent`, `wall_height` is `AIData::m_wallHeight`.
    pub fn new_pathfind_map(&mut self, extent: &Region3D, wall_height: Real) {
        let mut pathfinder = self
            .world
            .pathfinder
            .take()
            .unwrap_or_else(|| Box::new(Pathfinder::new(wall_height)));
        let objects: Vec<Footprint> = self
            .world
            .objects
            .world_iter()
            .filter_map(|slot| self.footprint(slot))
            .collect();
        let frame = self.world.frame;
        pathfinder.new_map(
            self.terrain.as_ref(),
            extent,
            wall_height,
            &objects,
            self,
            frame,
        );
        self.world.pathfinder = Some(pathfinder);
    }

    /// `Pathfinder::forceMapRecalculation`.
    pub fn force_pathfind_recalculation(&mut self) {
        let Some(mut pathfinder) = self.world.pathfinder.take() else {
            return;
        };
        pathfinder.force_map_recalculation(self.terrain.as_ref(), self);
        self.world.pathfinder = Some(pathfinder);
    }

    /// `addObjectToPathfindMap` / `removeObjectFromPathfindMap`. Objects that
    /// stood on a removed wall piece take their falling damage afterwards.
    pub(super) fn classify_on_pathfind_map(&mut self, slot: ObjectSlot, insert: bool) {
        if self.world.pathfinder.is_none() {
            return;
        }
        let Some(footprint) = self.footprint(slot) else {
            return;
        };
        // A removal first clears the cells the object holds as a unit.
        let kept = footprint.mine
            || footprint.projectile
            || footprint.bridge_tower
            || footprint.blast_crater
            || (footprint.fence_width > 0.0 && !footprint.defensive_wall);
        if !insert && !kept {
            self.remove_unit_from_pathfind_map(slot);
        }
        let Some(mut pathfinder) = self.world.pathfinder.take() else {
            return;
        };
        let frame = self.world.frame;
        let fallers = pathfinder.classify_object_footprint(&footprint, insert, self, frame);
        self.world.pathfinder = Some(pathfinder);
        // A removal also takes a unit's own cells off the map, with the same
        // exceptions as the footprint.
        let kept = footprint.mine
            || footprint.projectile
            || footprint.bridge_tower
            || footprint.blast_crater
            || (footprint.fence_width > 0.0 && !footprint.defensive_wall);
        if !insert && !kept {
            self.remove_unit_from_pathfind_map(slot);
        }
        for id in fallers {
            let Some(victim) = self.world.objects.slot_by_id(id) else {
                continue;
            };
            let mut damage = DamageInfo::default();
            damage.input.damage_type = DamageType::Falling;
            damage.input.death_type = DeathType::Splatted;
            damage.input.source_id = id;
            damage.input.amount = HUGE_DAMAGE_AMOUNT;
            self.attempt_damage(victim, &mut damage);
        }
    }

    /// `Pathfinder::addObjectToPathfindMap` for a caller outside the sim.
    pub fn add_object_to_pathfind_map(&mut self, slot: ObjectSlot) {
        self.classify_on_pathfind_map(slot, true);
    }

    /// `Pathfinder::removeObjectFromPathfindMap` for a caller outside the sim.
    pub fn remove_object_from_pathfind_map(&mut self, slot: ObjectSlot) {
        self.classify_on_pathfind_map(slot, false);
    }

    /// `createAWallFromMyFootprint` / `removeWallFromMyFootprint`.
    pub(super) fn classify_pathfind_wall(&mut self, slot: ObjectSlot, insert: bool) {
        if self.world.pathfinder.is_none() {
            return;
        }
        let Some(footprint) = self.footprint(slot) else {
            return;
        };
        let frame = self.world.frame;
        if let Some(pathfinder) = self.world.pathfinder.as_mut() {
            pathfinder.classify_footprint_cells(&footprint, insert, frame);
        }
    }
}

/// The units the pathfinder looks up while it holds the map.
impl PathUnits for SimContext {
    fn unit(&self, id: ObjectId) -> Option<PathUnit> {
        let sim = self;
        let slot = sim.world.objects.slot_by_id(id)?;
        let obj = sim.world.objects.get(slot)?;
        Some(PathUnit {
            has_ai: obj.ai_slot.is_some(),
            crushable_level: sim.object_crushable_level(slot) as u8,
            infantry: sim.object_is_kind_of(slot, KindOfType::Infantry) != FALSE,
        })
    }

    fn is_ally(&self, agent: ObjectId, other: ObjectId) -> bool {
        let objects = &self.world.objects;
        match (objects.slot_by_id(agent), objects.slot_by_id(other)) {
            (Some(a), Some(b)) => self.player_relationship(a, b) == Relationship::Allies,
            _ => false,
        }
    }

    fn is_non_enemy_object(&self, agent: ObjectId, other: ObjectId) -> bool {
        let objects = &self.world.objects;
        match (objects.slot_by_id(agent), objects.slot_by_id(other)) {
            (Some(a), Some(b)) => self.player_relationship(a, b) != Relationship::Enemies,
            _ => false,
        }
    }

    fn goal_object(&self, id: ObjectId) -> Option<GoalObject> {
        let slot = self.world.objects.slot_by_id(id)?;
        let obj = self.world.objects.get(slot)?;
        Some(GoalObject {
            position: *obj.thing.get_position(),
            layer: obj.layer,
            airfield: self.object_is_kind_of(slot, KindOfType::FsAirfield) != FALSE,
        })
    }

    fn is_idle(&self, id: ObjectId) -> bool {
        self.world
            .objects
            .slot_by_id(id)
            .and_then(|slot| self.world.objects.get(slot)?.ai_slot)
            .and_then(|ms| self.world.modules.payload(ms)?.as_ai())
            .is_some_and(|ai| ai.is_idle() != FALSE)
    }

    /// `canCrushOrSquish(other, TEST_CRUSH_OR_SQUISH)`.
    fn can_crush_or_squish(&self, agent: ObjectId, other: ObjectId) -> bool {
        let sim = self;
        let objects = &sim.world.objects;
        let (Some(a), Some(b)) = (objects.slot_by_id(agent), objects.slot_by_id(other)) else {
            return false;
        };
        if sim.object_can_crush(a, b) != FALSE {
            return true;
        }
        // Anything with a `SquishCollide` is squished at any crushable level.
        let unmanned = objects
            .get(a)
            .is_some_and(|o| o.is_disabled_by_type(DisabledType::Unmanned) != FALSE);
        !unmanned
            && sim.object_crusher_level(a) != 0
            && !self.is_ally(agent, other)
            && sim
                .rules
                .name_keys
                .lookup_key("SquishCollide")
                .is_some_and(|key| sim.has_module_by_name_key(b, key))
    }
}

impl TallBuildings for SimContext {
    fn tall_building(&self, id: ObjectId) -> Option<TallBuilding> {
        let slot = self.world.objects.slot_by_id(id)?;
        if self.object_is_kind_of(slot, KindOfType::AircraftPathAround) == FALSE {
            return None;
        }
        let obj = self.world.objects.get(slot)?;
        Some(TallBuilding {
            id,
            position: *obj.thing.get_position(),
            radius: obj.geometry.get_bounding_circle_radius(),
        })
    }

    /// `getClosestObject` from a position by bounding sphere: every building it
    /// reaches into is at distance zero, and the first one visited wins.
    fn closest_tall_building(&self, pos: &Coord3D, radius: Real) -> Option<TallBuilding> {
        let objects = &self.world.objects;
        let factory = &self.rules.thing_factory;
        let found = self.world.partition.get_closest_object(
            QueryCenter::Position(*pos),
            radius,
            DistanceCalculationType::FromBoundingSphere2d,
            &|id| partition_object(objects, factory, id),
            &mut |id| self.tall_building(id).is_some(),
        );
        found.id.and_then(|id| self.tall_building(id))
    }
}

impl SimContext {
    /// `updatePos` as `setTriggerAreaFlagsForChangeInPosition` calls it.
    pub(super) fn update_pathfind_position_after_move(&mut self, slot: ObjectSlot) {
        if self.world.pathfinder.is_none()
            || self.object_is_kind_of(slot, KindOfType::Projectile) != FALSE
            || self.object_is_kind_of(slot, KindOfType::Inert) != FALSE
            || self
                .world
                .objects
                .get(slot)
                .map_or(true, |o| o.ai_slot.is_none())
        {
            return;
        }
        // `setTriggerAreaFlagsForChangeInPosition` returns early for these and
        // while the position truncates to the same integers.
        if self.object_is_kind_of(slot, KindOfType::Projectile) != FALSE
            || self.object_is_kind_of(slot, KindOfType::Inert) != FALSE
        {
            return;
        }
        let Some(pos) = self
            .world
            .objects
            .get(slot)
            .map(|o| *o.thing.get_position())
        else {
            return;
        };
        let whole = (pos.x as Int, pos.y as Int);
        if let Some(ai) = self.resident_ai(slot) {
            if ai.whole_position == whole {
                return;
            }
            ai.whole_position = whole;
        }
        self.update_pathfind_position(slot);
    }

    /// `Pathfinder::removePos`.
    pub(super) fn remove_pathfind_position(&mut self, slot: ObjectSlot) {
        self.with_goal_mover(slot, Pathfinder::remove_pos);
    }

    fn resident_ai(&mut self, slot: ObjectSlot) -> Option<&mut AiUpdate> {
        let module = self.world.objects.get(slot)?.ai_slot?;
        self.resident_state_mut(module)?.downcast_mut::<AiUpdate>()
    }

    /// What the pathfinder reads from a unit.
    fn path_agent(&self, slot: ObjectSlot) -> Option<PathAgent> {
        let id = self.world.objects.id_of(slot)?;
        let obj = self.world.objects.get(slot)?;
        let ai = obj
            .ai_slot
            .and_then(|ms| self.world.modules.payload(ms))
            .and_then(|m| m.as_ai());
        let adjusts = self.cur_locomotor(slot).is_some_and(|l| {
            matches!(
                l.template.appearance,
                LocomotorAppearance::Hover | LocomotorAppearance::Wings
            )
        });
        Some(PathAgent {
            id,
            has_ai: obj.ai_slot.is_some(),
            ignored_obstacle: ai.map_or(ObjectId::INVALID, |a| a.get_ignored_obstacle_id()),
            aircraft_adjusts: obj.ai_slot.is_some() && adjusts,
            crusher_level: self.object_crusher_level(slot) as u8,
            radius: obj.geometry.get_bounding_circle_radius(),
            layer: obj.layer,
            infantry: self.object_is_kind_of(slot, KindOfType::Infantry) != FALSE,
        })
    }

    /// `LocomotorSet::getValidSurfaces` of the set the object has chosen.
    pub(super) fn locomotor_set_surfaces(&self, slot: ObjectSlot) -> LocomotorSurfaceTypeMask {
        let Some(obj) = self.world.objects.get(slot) else {
            return 0;
        };
        let Some(set) = obj.ai_locomotor_set else {
            return 0;
        };
        obj.thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .map_or(0, |t| {
                t.get_locomotor_set(set)
                    .iter()
                    .filter_map(|name| self.rules.locomotor_store.find_by_name(name))
                    .fold(0, |mask, l| mask | l.surfaces)
            })
    }

    /// What the destination searches read from a unit. A team with no player
    /// type counts as a person's, as an object without a player does.
    pub(super) fn adjust_unit(&self, slot: ObjectSlot) -> Option<AdjustUnit> {
        let obj = self.world.objects.get(slot)?;
        let human = self
            .player_of_member(obj.team_id)
            .is_none_or(|p| p.player_type == PlayerType::Human);
        Some(AdjustUnit {
            agent: self.path_agent(slot)?,
            position: *obj.thing.get_position(),
            surfaces: self.locomotor_set_surfaces(slot),
            aircraft: self.object_is_kind_of(slot, KindOfType::Aircraft) != FALSE,
            projectile: self.object_is_kind_of(slot, KindOfType::Projectile) != FALSE,
            human,
        })
    }

    /// The unit as the reservation updates see it. Without an `AIUpdate` module
    /// there is nowhere to keep its cells, so the updates leave it alone.
    fn goal_mover(&mut self, slot: ObjectSlot) -> Option<GoalMover> {
        let mut agent = self.path_agent(slot)?;
        let cells = self
            .resident_ai(slot)
            .map(|ai| (ai.pathfind_goal_cell, ai.pathfind_cur_cell));
        agent.has_ai = cells.is_some();
        let none = ICoord2D { x: -1, y: -1 };
        let (goal_cell, cur_cell) = cells.unwrap_or((none, none));
        let ground_movement = self.ai_is_doing_ground_movement(slot) != FALSE;
        let obj = self.world.objects.get(slot)?;
        Some(GoalMover {
            agent,
            immobile: self.object_is_kind_of(slot, KindOfType::Immobile) != FALSE,
            ground_movement,
            unmanned_helicopter: obj.is_disabled_by_type(DisabledType::Unmanned) != FALSE
                && self.object_is_kind_of(slot, KindOfType::ProducedAtHelipad) != FALSE,
            destination_layer: obj.destination_layer,
            goal_cell,
            cur_cell,
        })
    }

    fn store_goal_mover(&mut self, slot: ObjectSlot, mover: &GoalMover) {
        if let Some(obj) = self.world.objects.get_mut(slot) {
            obj.destination_layer = mover.destination_layer;
        }
        if let Some(ai) = self.resident_ai(slot) {
            ai.pathfind_goal_cell = mover.goal_cell;
            ai.pathfind_cur_cell = mover.cur_cell;
        }
    }

    /// Runs `act` on the map's pathfinder and the unit in `slot`.
    fn with_goal_mover(
        &mut self,
        slot: ObjectSlot,
        act: impl FnOnce(&mut Pathfinder, &mut GoalMover),
    ) {
        if self.world.pathfinder.is_none() {
            return;
        }
        let Some(mut mover) = self.goal_mover(slot) else {
            return;
        };
        if let Some(pathfinder) = self.world.pathfinder.as_mut() {
            act(pathfinder, &mut mover);
        }
        self.store_goal_mover(slot, &mover);
    }

    /// `Pathfinder::updateGoal`. The sim's terrain has no bridges, so no unit
    /// is at a bridge end.
    pub(super) fn update_goal_on_map(
        &mut self,
        slot: ObjectSlot,
        pos: &Coord3D,
        layer: PathfindLayerEnum,
    ) {
        self.with_goal_mover(slot, |pf, mover| {
            pf.update_goal(mover, pos, layer, &mut |_| false)
        });
    }

    /// `Pathfinder::removeGoal`.
    pub(super) fn remove_goal_on_map(&mut self, slot: ObjectSlot) {
        self.with_goal_mover(slot, Pathfinder::remove_goal);
    }

    /// `Pathfinder::snapClosestGoalPosition`.
    pub(super) fn snap_closest_goal_on_map(&mut self, slot: ObjectSlot, pos: &mut Coord3D) {
        let Some(agent) = self.path_agent(slot) else {
            return;
        };
        let Some(pathfinder) = self.world.pathfinder.take() else {
            return;
        };
        pathfinder.snap_closest_goal_position(self.terrain.as_ref(), self, &agent, pos);
        self.world.pathfinder = Some(pathfinder);
    }

    /// `Pathfinder::updatePos` for the object where it stands.
    pub fn update_pathfind_position(&mut self, slot: ObjectSlot) {
        let Some(pos) = self
            .world
            .objects
            .get(slot)
            .map(|o| *o.thing.get_position())
        else {
            return;
        };
        self.with_goal_mover(slot, |pf, mover| pf.update_pos(mover, &pos, &mut |_| false));
    }

    /// `Pathfinder::removeUnitFromPathfindMap`.
    pub fn remove_unit_from_pathfind_map(&mut self, slot: ObjectSlot) {
        self.with_goal_mover(slot, Pathfinder::remove_unit_from_map);
    }

    /// `Pathfinder::adjustDestination` for the unit's own locomotors, or `None`
    /// without a pathfind map.
    pub(super) fn adjust_destination_on_map(
        &mut self,
        slot: ObjectSlot,
        dest: &mut Coord3D,
    ) -> Option<bool> {
        self.world.pathfinder.as_ref()?;
        let unit = self.adjust_unit(slot)?;
        let mut pathfinder = self.world.pathfinder.take()?;
        let found = pathfinder.adjust_destination(
            self.terrain.as_ref(),
            self,
            &mut NoPathCost,
            &unit,
            dest,
            None,
        );
        self.world.pathfinder = Some(pathfinder);
        Some(found)
    }

    /// `Pathfinder::adjustToPossibleDestination`, or `None` without a map.
    pub(crate) fn adjust_to_possible_destination_on_map(
        &self,
        slot: ObjectSlot,
        dest: &mut Coord3D,
    ) -> Option<bool> {
        let pathfinder = self.world.pathfinder.as_ref()?;
        let unit = self.adjust_unit(slot)?;
        Some(pathfinder.adjust_to_possible_destination(
            self.terrain.as_ref(),
            self,
            &unit,
            dest,
        ))
    }
}

impl SimContext {
    /// `Pathfinder::isAttackViewBlockedByObstacle`: whether `owner`, standing at
    /// `from`, cannot see to shoot at `victim` (or the spot) at `to`. Never
    /// blocked without a pathfind map.
    pub(super) fn attack_view_blocked_by_obstacle(
        &self,
        owner: ObjectSlot,
        from: &Coord3D,
        victim: Option<ObjectSlot>,
        to: &Coord3D,
    ) -> bool {
        let uses = self.rules.ai_data.attack_uses_line_of_sight != FALSE;
        let (Some(map), Some(attacker)) = (self.world.pathfinder.as_ref(), self.view_party(owner))
        else {
            return false;
        };
        if !uses || !attacker.needs_line_of_sight {
            return false;
        }
        let victim_party = victim.and_then(|slot| self.view_party(slot));
        let weapon = self
            .world
            .objects
            .get(owner)
            .filter(|_| self.object_is_kind_of(owner, KindOfType::Immobile) == FALSE)
            .and_then(|o| {
                let set = &o.weapon_set;
                set.get_weapon_in_weapon_slot(set.get_cur_weapon_slot())
            });
        let sight = weapon.map_or(TerrainSight::NoWeapon, |w| {
            let clear = match victim {
                Some(v) => weapon_queries::is_clear_goal_firing_los_terrain(self, owner, from, v),
                None => weapon_queries::is_clear_goal_firing_los_terrain_to_position(
                    self, owner, from, to,
                ),
            };
            let terrain_blocked = clear == FALSE;
            if w.template.is_contact_weapon() != FALSE {
                TerrainSight::Contact { terrain_blocked }
            } else {
                TerrainSight::Weapon { terrain_blocked }
            }
        });
        map.is_attack_view_blocked(uses, &attacker, from, victim_party.as_ref(), to, sight)
    }

    /// What a line of sight question reads from one end.
    fn view_party(&self, slot: ObjectSlot) -> Option<ViewParty> {
        let obj = self.world.objects.get(slot)?;
        let id_of = |slot| self.world.objects.id_of(slot);
        let slaver = obj
            .slaved_slot
            .and_then(|ms| self.world.modules.payload(ms)?.as_slaved_update())
            .map(|slaved| slaved.get_slaver_id());
        Some(ViewParty {
            id: obj.id,
            position: *obj.thing.get_position(),
            layer: obj.layer,
            container: obj.contained_by.and_then(id_of).unwrap_or(ObjectId::INVALID),
            slaver: slaver.unwrap_or(ObjectId::INVALID),
            needs_line_of_sight: self.object_is_kind_of(slot, KindOfType::AttackNeedsLineOfSight)
                != FALSE,
            significantly_above_terrain: self.object_is_significantly_above_terrain(slot) != FALSE,
        })
    }
}
