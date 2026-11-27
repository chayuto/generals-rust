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
use zh_common::types::{ObjectId, Real, Relationship, FALSE};

use super::{SimApi, SimContext, SimView};
use crate::arena::ObjectSlot;
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::combat::locomotor::LocomotorAppearance;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::update::ai_update::AiUpdate;
use crate::pathfinder::ai_pathfind::{
    AdjustUnit, Footprint, GoalMover, NoPathCost, PathAgent, PathUnit, PathUnits, PathWorld,
    Pathfinder,
};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
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
        let Some(mut pathfinder) = self.world.pathfinder.take() else {
            return;
        };
        let frame = self.world.frame;
        let fallers = pathfinder.classify_object_footprint(&footprint, insert, self, frame);
        self.world.pathfinder = Some(pathfinder);
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
struct SimPathUnits<'a>(&'a SimContext);

impl PathUnits for SimPathUnits<'_> {
    fn unit(&self, id: ObjectId) -> Option<PathUnit> {
        let sim = self.0;
        let slot = sim.world.objects.slot_by_id(id)?;
        let obj = sim.world.objects.get(slot)?;
        Some(PathUnit {
            has_ai: obj.ai_slot.is_some(),
            crushable_level: sim.object_crushable_level(slot) as u8,
            infantry: sim.object_is_kind_of(slot, KindOfType::Infantry) != FALSE,
        })
    }

    fn is_ally(&self, agent: ObjectId, other: ObjectId) -> bool {
        let objects = &self.0.world.objects;
        match (objects.slot_by_id(agent), objects.slot_by_id(other)) {
            (Some(a), Some(b)) => self.0.player_relationship(a, b) == Relationship::Allies,
            _ => false,
        }
    }

    /// `canCrushOrSquish(other, TEST_CRUSH_OR_SQUISH)`.
    fn can_crush_or_squish(&self, agent: ObjectId, other: ObjectId) -> bool {
        let sim = self.0;
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

impl SimContext {
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
    fn adjust_unit(&self, slot: ObjectSlot) -> Option<AdjustUnit> {
        let obj = self.world.objects.get(slot)?;
        let human = self.world.player_is_human.get(&obj.team_id) != Some(&FALSE);
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

    /// `Pathfinder::updatePos` for the object where it stands. Nothing in the
    /// sim calls it when an object moves yet.
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
            &SimPathUnits(self),
            &mut NoPathCost,
            &unit,
            dest,
            None,
        );
        self.world.pathfinder = Some(pathfinder);
        Some(found)
    }

    /// `Pathfinder::adjustToPossibleDestination`, or `None` without a map.
    pub(super) fn adjust_to_possible_destination_on_map(
        &self,
        slot: ObjectSlot,
        dest: &mut Coord3D,
    ) -> Option<bool> {
        let pathfinder = self.world.pathfinder.as_ref()?;
        let unit = self.adjust_unit(slot)?;
        Some(pathfinder.adjust_to_possible_destination(
            self.terrain.as_ref(),
            &SimPathUnits(self),
            &unit,
            dest,
        ))
    }
}
