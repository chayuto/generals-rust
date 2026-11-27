//! The sim's side of the pathfind map (`PKT-411`): what the pathfinder reads
//! from an object, and the calls that put objects on the map and take them off.
//!
//! Without a pathfinder (`SimWorld::pathfinder` is `None`) these do nothing.
//!
//! Golden test of the map itself: `crates/zh_logic/tests/golden_s2_pathfind_map.rs`.

use zh_common::math::{Coord3D, Region3D};
use zh_common::types::{ObjectId, Real, FALSE};

use super::{SimApi, SimContext, SimView};
use crate::arena::ObjectSlot;
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::gamelogic::global_data::BodyDamageType;
use crate::pathfinder::ai_pathfind::{Footprint, PathWorld, Pathfinder};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
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
