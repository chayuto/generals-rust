//! The sim's side of `TerrainLogic`'s bridges (`TerrainLogic.cpp`):
//! `addBridgeToLogic`, `addLandmarkBridgeToLogic` with the towers of
//! `Bridge(Object*)` and `Bridge::createTower`, and `updateBridgeDamageStates`.
//!
//! Golden test: `crates/zh_logic/tests/golden_bridge_behavior.rs`.

use std::f32::consts::PI;
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{Bool, ObjectId, Real, FALSE, TRUE};

use super::{SimApi, SimContext};
use crate::arena::{ModuleSidecar, ModuleSlot, ObjectSlot};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::module::ScaffoldTargetMotion;
use crate::modules::module::{BehaviorModule, BridgeScaffoldFacet, BridgeTowerType};
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::terrain::bridge::{Bridge, BridgeGeometry, BridgeList, BridgeWorld, DeckObject};

/// `PATHFIND_CELL_SIZE_F / 2`: the tower offset used when a tower has no template.
const TOWER_OFFSET: Real = 5.0;

const TOWERS: [BridgeTowerType; BridgeTowerType::MAX_TOWERS] = [
    BridgeTowerType::FromLeft,
    BridgeTowerType::FromRight,
    BridgeTowerType::ToLeft,
    BridgeTowerType::ToRight,
];

impl SimContext {
    /// `TheTerrainLogic`'s bridges; `None` over terrain that is not a map.
    pub fn bridge_list(&self) -> Option<&BridgeList> {
        Some(self.terrain.terrain_logic()?.bridges())
    }

    /// The linking half of `addBridgeToLogic`: the bridge goes to the head of the
    /// list and takes a pathfind layer.
    pub fn add_bridge_to_logic(&mut self, mut bridge: Bridge) {
        let Some(terrain) = self.terrain.terrain_logic_mut() else {
            return;
        };
        match self.world.pathfinder.as_mut() {
            Some(pathfinder) => terrain.add_bridge(bridge, pathfinder),
            None => {
                bridge.layer = PathfindLayerEnum::Ground;
                terrain.bridges_mut().push(bridge);
            }
        }
    }

    /// `addLandmarkBridgeToLogic`: the bridge object's box is the deck, and towers
    /// are made at its corners. Returns the number of towers made.
    pub fn add_landmark_bridge(&mut self, slot: ObjectSlot) -> u32 {
        let Some(obj) = self.world.objects.get(slot) else {
            return 0;
        };
        let id = obj.id;
        let name = obj
            .thing
            .template_override()
            .get_final(&self.rules.thing_factory)
            .map_or_else(String::new, |t| t.get_name().to_owned());
        let geometry = BridgeGeometry::from_box(
            obj.thing.get_position(),
            obj.thing.get_orientation(),
            obj.geometry.get_major_radius(),
            obj.geometry.get_minor_radius(),
        );
        let mut bridge = Bridge::new(geometry, &name, id);
        let tower_names = self
            .rules
            .terrain_roads
            .find_bridge(&name)
            .map(|road| road.tower_object_name.clone());
        let mut made = 0;
        if let Some(tower_names) = tower_names {
            let mut across = Coord2D {
                x: geometry.to_left.x - geometry.to_right.x,
                y: geometry.to_left.y - geometry.to_right.y,
            };
            across.normalize();
            // A tower without a template keeps the offset of the one before it.
            let mut offset = TOWER_OFFSET;
            for (tower_type, tower_name) in TOWERS.into_iter().zip(&tower_names) {
                if let Some(template) = self.rules.thing_factory.find_template_final(tower_name) {
                    offset = template.get_template_geometry_info().get_major_radius();
                }
                // The left towers move outward along `across`, the right ones against it.
                let (mut pos, left) = match tower_type {
                    BridgeTowerType::FromLeft => (geometry.from_left, true),
                    BridgeTowerType::FromRight => (geometry.from_right, false),
                    BridgeTowerType::ToLeft => (geometry.to_left, true),
                    BridgeTowerType::ToRight => (geometry.to_right, false),
                };
                if left {
                    pos.x += across.x * offset;
                    pos.y += across.y * offset;
                } else {
                    pos.x -= across.x * offset;
                    pos.y -= across.y * offset;
                }
                if let Some(tower) = self.create_bridge_tower(&pos, tower_type, tower_name, slot) {
                    if let Some(tower_id) = self.world.objects.id_of(tower) {
                        bridge.set_tower(tower_type.as_i32() as usize, tower_id);
                        made += 1;
                    }
                }
            }
        }
        self.add_bridge_to_logic(bridge);
        made
    }

    /// `Bridge::createTower`.
    fn create_bridge_tower(
        &mut self,
        pos: &Coord3D,
        tower_type: BridgeTowerType,
        template_name: &str,
        bridge: ObjectSlot,
    ) -> Option<ObjectSlot> {
        let template = self.rules.thing_factory.find_template_id(template_name)?;
        let record = self.world.objects.get(bridge)?;
        let (bridge_id, team, facing) = (record.id, record.team_id, record.thing.get_orientation());
        let tower = self.factory_new_object(template, team);
        let angle = match tower_type {
            BridgeTowerType::FromLeft | BridgeTowerType::FromRight => facing + PI,
            BridgeTowerType::ToLeft | BridgeTowerType::ToRight => facing,
        };
        self.set_object_position(tower, pos);
        self.set_object_orientation(tower, angle);
        let tower_id = self.world.objects.id_of(tower)?;
        if let Some(slot) = self.first_behavior(bridge, |m| m.as_bridge().is_some()) {
            if let Some(module) = self.world.modules.payload_mut(slot) {
                if let Some(facet) = module.as_bridge_mut() {
                    facet.set_tower(tower_type, tower_id);
                }
            }
        }
        if let Some(slot) = self.first_behavior(tower, |m| m.as_bridge_tower().is_some()) {
            if let Some(module) = self.world.modules.payload_mut(slot) {
                if let Some(facet) = module.as_bridge_tower_mut() {
                    facet.set_bridge(bridge_id);
                    facet.set_tower_type(tower_type);
                }
            }
        }
        if self.body_is_indestructible(bridge) {
            self.set_body_indestructible(tower, TRUE);
        }
        Some(tower)
    }

    /// The first behavior module of `owner` with the wanted interface.
    fn first_behavior(
        &self,
        owner: ObjectSlot,
        wanted: impl Fn(&dyn BehaviorModule) -> bool,
    ) -> Option<ModuleSlot> {
        self.world
            .objects
            .get(owner)?
            .behaviors
            .iter()
            .map(|&(slot, _)| slot)
            .find(|&slot| {
                self.world
                    .modules
                    .payload(slot)
                    .is_some_and(|m| wanted(&**m))
            })
    }

    fn body_is_indestructible(&self, owner: ObjectSlot) -> bool {
        self.world
            .objects
            .get(owner)
            .and_then(|o| o.body_slot)
            .and_then(|slot| self.world.modules.payload(slot))
            .and_then(|m| m.as_body())
            .is_some_and(|b| b.is_indestructible() != FALSE)
    }

    /// The body's damage state, also while the body module is out running a
    /// damage callback.
    pub(super) fn bridge_body_state(&self, owner: ObjectSlot) -> BodyDamageType {
        let Some(slot) = self.world.objects.get(owner).and_then(|o| o.body_slot) else {
            return BodyDamageType::Pristine;
        };
        if let Some(body) = self.world.modules.payload(slot).and_then(|m| m.as_body()) {
            return body.get_damage_state();
        }
        match self.world.modules.sidecar(slot) {
            ModuleSidecar::Body { damage_state, .. } => {
                BodyDamageType::from_i32(i32::from(damage_state))
                    .unwrap_or(BodyDamageType::Pristine)
            }
            _ => BodyDamageType::Pristine,
        }
    }

    /// `Pathfinder::changeBridgeState`.
    pub(super) fn change_bridge_layer(&mut self, layer: PathfindLayerEnum, repaired: bool) {
        let frame = self.world.frame;
        if let Some(pathfinder) = self.world.pathfinder.as_mut() {
            pathfinder.change_bridge_state(layer, repaired, self.terrain.as_ref(), frame);
        }
    }

    /// `updateBridgeDamageStates`: closes the layer of a bridge that fell and drops
    /// what stood on it, and opens the layer of one that was repaired.
    pub fn update_all_bridge_damage_states(&mut self) {
        let count = self.bridge_list().map_or(0, BridgeList::len);
        for i in 0..count {
            let Some(terrain) = self.terrain.terrain_logic_mut() else {
                return;
            };
            let mut bridges = std::mem::take(terrain.bridges_mut());
            let mut world = BridgeSnapshot {
                sim: self,
                fallers: Vec::new(),
            };
            let mut changes = Vec::new();
            bridges.update_damage_state(i, &mut world, &mut |layer, repaired| {
                changes.push((layer, repaired));
            });
            let fallers = world.fallers;
            if let Some(terrain) = self.terrain.terrain_logic_mut() {
                *terrain.bridges_mut() = bridges;
            }
            for (layer, repaired) in changes {
                self.change_bridge_layer(layer, repaired);
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
    }

    pub(super) fn bridge_scaffold_motion_of(
        &self,
        owner: ObjectSlot,
    ) -> Option<ScaffoldTargetMotion> {
        let slot = self.world.objects.get(owner)?.bridge_scaffold_slot?;
        let module = self.world.modules.payload(slot)?;
        Some(module.as_bridge_scaffold()?.get_current_motion())
    }

    pub(super) fn with_scaffold_facet(
        &mut self,
        owner: ObjectSlot,
        f: &mut dyn FnMut(&mut dyn BridgeScaffoldFacet),
    ) {
        let Some(slot) = self
            .world
            .objects
            .get(owner)
            .and_then(|o| o.bridge_scaffold_slot)
        else {
            return;
        };
        if let Some(facet) = self
            .world
            .modules
            .payload_mut(slot)
            .and_then(|m| m.as_bridge_scaffold_mut())
        {
            f(facet);
        }
    }

    /// What `BridgeBehavior::xfer` hands the terrain's bridge after a load.
    pub(super) fn restore_bridge_links(
        &mut self,
        pos: &Coord3D,
        object_id: ObjectId,
        tower_ids: [ObjectId; 4],
    ) {
        let Some(terrain) = self.terrain.terrain_logic_mut() else {
            return;
        };
        if let Some(bridge) = terrain
            .bridges_mut()
            .iter_mut()
            .find(|b| b.geometry.is_point_on(pos))
        {
            bridge.object_id = object_id;
            bridge.tower_ids = tower_ids;
        }
    }
}

/// The world as one bridge's damage pass reads it; falls are dealt afterwards.
struct BridgeSnapshot<'a> {
    sim: &'a SimContext,
    fallers: Vec<ObjectId>,
}

impl BridgeWorld for BridgeSnapshot<'_> {
    fn bridge_damage_state(&self, id: ObjectId) -> Option<BodyDamageType> {
        let slot = self.sim.world.objects.slot_by_id(id)?;
        Some(self.sim.bridge_body_state(slot))
    }

    fn scaffold_present(&self, id: ObjectId) -> bool {
        let present: Option<Bool> = (|| {
            let slot = self.sim.world.objects.slot_by_id(id)?;
            let module = self.sim.world.objects.get(slot)?.bridge_slot?;
            let facet = self.sim.world.modules.payload(module)?.as_bridge()?;
            Some(facet.is_scaffold_present())
        })();
        present.is_some_and(|p| p != FALSE)
    }

    fn objects(&self) -> Vec<DeckObject> {
        self.sim
            .world
            .objects
            .world_iter()
            .filter_map(|slot| {
                let obj = self.sim.world.objects.get(slot)?;
                Some(DeckObject {
                    id: obj.id,
                    position: *obj.thing.get_position(),
                    minor_radius: obj.geometry.get_minor_radius(),
                    layer: obj.layer,
                })
            })
            .collect()
    }

    fn kill_by_fall(&mut self, id: ObjectId) {
        self.fallers.push(id);
    }
}
