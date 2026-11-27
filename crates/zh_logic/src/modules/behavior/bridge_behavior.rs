//! `PKT-260`: `BridgeBehavior` (`BridgeBehavior.h`, `BridgeBehavior.cpp`): the bridge
//! object's module. It keeps the tower table, passes damage and healing on to the
//! towers, follows the body's damage state into the pathfind layer, drops what
//! stood on a fallen deck and puts up the repair scaffold.
//!
//! Packet: `packets/phase5/PKT-260_BridgeBehavior.md`.
//! Golden test: `crates/zh_logic/tests/golden_bridge_behavior.rs`. Sounds and the
//! radar refresh are client work and are left out.

use crate::arena::{ModuleContext, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::behavior_module::{
    lend_self, xfer_module_version, UpdateScheduleTracker,
};
use crate::modules::contain::open_contain::kill_object;
use crate::modules::module::{
    BehaviorModule, BridgeFacet, BridgeTowerType, DamageFacet, DieFacet, ModuleData,
    ScaffoldTargetMotion, UpdateFacet,
};
use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::velocity;
use crate::modules::module_data::{parse, FieldText};
use crate::modules::module_factory::ModuleParseContext;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::rules::kind_of::KindOfType;
use crate::sim::{SimApi, SimView};
use crate::terrain::bridge::{point_inside_area_2d, Bridge, BridgeGeometry};
use std::any::Any;
use std::collections::BTreeSet;
use zh_common::ini::ini_terrain_road::{TerrainRoadType, BRIDGE_BODY_STATES, MAX_BRIDGE_BODY_FX};
use zh_common::ini::{IniError, IniReader, SEPS_COLON};
use zh_common::math::gamemath::{div_safe_f32, real_to_int_ceil, reference_div_f32};
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UnsignedShort, UpdateSleepTime, FALSE,
    TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

/// `BridgeFXInfo` and `BridgeOCLInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeEffect {
    pub name: Option<String>,
    /// `TimeAndLocationInfo`.
    pub delay: UnsignedInt,
    pub bone_name: String,
}

impl FieldText for Vec<BridgeEffect> {
    fn field_text(&self) -> String {
        self.iter()
            .map(|e| format!("{}|{}|{}", e.name.field_text(), e.delay, e.bone_name))
            .collect::<Vec<_>>()
            .join(",")
    }
}

fn expect(token: &str, name: &str) -> Result<(), IniError> {
    if token.eq_ignore_ascii_case(name) {
        Ok(())
    } else {
        Err(IniError::InvalidData)
    }
}

/// A `<name>:` label.
fn label_token(r: &mut IniReader<'_>, name: &str) -> Result<(), IniError> {
    expect(r.next_token(SEPS_COLON)?, name)?;
    r.consume_separator();
    Ok(())
}

/// `<label>:<name> Delay:<msec> [Bone:<bone>]`.
fn die_effect(
    list: &mut Vec<BridgeEffect>,
    label: &str,
    known: &BTreeSet<String>,
    r: &mut IniReader<'_>,
) -> Result<(), IniError> {
    label_token(r, label)?;
    let name = parse::lookup(r.next_token(zh_common::ini::SEPS_DEFAULT)?, known);
    label_token(r, "Delay")?;
    let mut delay = 0;
    r.parse_duration_unsigned_int(&mut delay)?;
    let mut bone_name = String::new();
    if let Some(token) = r.next_token_or_null(SEPS_COLON) {
        expect(token, "Bone")?;
        r.consume_separator();
        bone_name = r.next_ascii_string()?;
    }
    list.push(BridgeEffect {
        name,
        delay,
        bone_name,
    });
    Ok(())
}

fn die_effect_fx(
    v: &mut Vec<BridgeEffect>,
    r: &mut IniReader<'_>,
    c: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    die_effect(v, "FX", c.fx_list_names, r)
}

fn die_effect_ocl(
    v: &mut Vec<BridgeEffect>,
    r: &mut IniReader<'_>,
    c: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    die_effect(v, "OCL", c.ocl_names, r)
}

module_data! {
    /// `BridgeBehaviorModuleData`.
    BridgeBehaviorModuleData = "BridgeBehavior";
    "LateralScaffoldSpeed" => lateral_scaffold_speed: Real = 1.0, velocity;
    "VerticalScaffoldSpeed" => vertical_scaffold_speed: Real = 1.0, velocity;
    "BridgeDieFX" => die_fx: Vec<BridgeEffect> = Vec::new(), die_effect_fx;
    "BridgeDieOCL" => die_ocl: Vec<BridgeEffect> = Vec::new(), die_effect_ocl;
}

/// `setScaffoldData`'s distance between the sunken position and the ground.
const SCAFFOLD_FUDGE: Real = 8.0;

/// The names of one transition's creation lists and effects, `None` where the store
/// has no such entry.
type Resolved = [[Option<String>; MAX_BRIDGE_BODY_FX]; BRIDGE_BODY_STATES];

/// What `resolveFX` finds in the bridge's road definition. Not saved.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BridgeTransitions {
    pub damage_ocl: Resolved,
    pub damage_fx: Resolved,
    pub repair_ocl: Resolved,
    pub repair_fx: Resolved,
}

/// The effects of one transition to one body state.
struct AreaEffects {
    geometry: BridgeGeometry,
    count: Int,
    height: Real,
    lists: [(Option<String>, Option<String>); MAX_BRIDGE_BODY_FX],
}

/// Where one scaffold piece starts, comes up to and is built across to.
struct ScaffoldPath {
    angle: Real,
    height: Real,
    rise_to: Coord3D,
    build: Coord3D,
    center: Coord3D,
}

/// Runtime instance of `BridgeBehavior`.
#[derive(Debug, Clone)]
pub struct BridgeBehavior {
    pub data: BridgeBehaviorModuleData,
    pub schedule: UpdateScheduleTracker,
    pub tower_ids: [ObjectId; BridgeTowerType::MAX_TOWERS],
    /// `m_fxResolved` with what it resolved.
    pub transitions: Option<Box<BridgeTransitions>>,
    pub scaffold_present: Bool,
    pub scaffold_ids: Vec<ObjectId>,
    pub death_frame: UnsignedInt,
}

fn road_of(sim: &dyn SimView, bridge: &Bridge) -> Option<TerrainRoadType> {
    sim.rules()
        .terrain_roads
        .find_bridge(&bridge.template_name)
        .cloned()
}

fn position_of(sim: &dyn SimView, owner: ObjectSlot) -> Coord3D {
    sim.object(owner)
        .map_or_else(Coord3D::default, |o| *o.thing.get_position())
}

/// `getRandomSurfacePosition`.
fn random_surface_position(
    sim: &mut dyn SimApi,
    geometry: &BridgeGeometry,
    effects_height: Real,
) -> Coord3D {
    let along = |a: &Coord3D, b: &Coord3D| Coord3D::new(a.x - b.x, a.y - b.y, a.z - b.z);
    let mut v1 = along(&geometry.to_left, &geometry.from_left);
    let scale = sim.rng_logic_real(0.0, 1.0);
    v1.x *= scale;
    v1.y *= scale;
    v1.z *= scale;
    let mut v2 = along(&geometry.from_right, &geometry.from_left);
    let scale = sim.rng_logic_real(0.0, 1.0);
    v2.x *= scale;
    v2.y *= scale;
    v2.z *= scale;
    let mut pos = Coord3D::new(
        v1.x + v2.x + geometry.from_left.x,
        v1.y + v2.y + geometry.from_left.y,
        v1.z + v2.z + geometry.from_left.z,
    );
    pos.z += sim.rng_logic_real(0.0, effects_height);
    pos
}

/// `Object::getSingleLogicalBonePosition`: the object's position without the bone.
fn bone_position(sim: &dyn SimView, owner: ObjectSlot, bone: &str) -> Coord3D {
    let mut world = position_of(sim, owner);
    if let Some(transform) = sim.pristine_bone_transforms(owner, bone).first() {
        let local = transform.get_translation();
        if let Some(obj) = sim.object(owner) {
            obj.thing
                .convert_bone_pos_to_world_pos(Some(&local), None, Some(&mut world), None);
        }
    }
    world
}

impl BridgeBehavior {
    pub fn new(data: BridgeBehaviorModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::new(),
            tower_ids: [ObjectId::INVALID; BridgeTowerType::MAX_TOWERS],
            transitions: None,
            scaffold_present: FALSE,
            scaffold_ids: Vec::new(),
            death_frame: 0,
        }
    }

    /// `resolveFX`: left unresolved while there is no bridge or definition.
    fn resolve_transitions(&mut self, sim: &dyn SimView, owner: ObjectSlot) {
        let Some(road) = sim
            .find_bridge_at(&position_of(sim, owner))
            .and_then(|bridge| road_of(sim, &bridge))
        else {
            return;
        };
        let rules = sim.rules();
        let resolve = |names: &[[String; MAX_BRIDGE_BODY_FX]; BRIDGE_BODY_STATES],
                       known: &BTreeSet<String>| {
            let mut out = Resolved::default();
            for (slots, state) in out.iter_mut().zip(names) {
                for (slot, name) in slots.iter_mut().zip(state) {
                    *slot = known.contains(name).then(|| name.clone());
                }
            }
            out
        };
        self.transitions = Some(Box::new(BridgeTransitions {
            damage_ocl: resolve(&road.damage_to_ocl, &rules.ocl_names),
            damage_fx: resolve(&road.damage_to_fx, &rules.fx_list_names),
            repair_ocl: resolve(&road.repaired_to_ocl, &rules.ocl_names),
            repair_fx: resolve(&road.repaired_to_fx, &rules.fx_list_names),
        }));
    }

    /// `doAreaEffects` for each effect number.
    fn area_effects(sim: &mut dyn SimApi, owner: ObjectSlot, effects: &AreaEffects) {
        for (ocl, fx) in &effects.lists {
            if ocl.is_none() && fx.is_none() {
                continue;
            }
            for _ in 0..effects.count {
                if let Some(fx) = fx {
                    let pos = random_surface_position(sim, &effects.geometry, effects.height);
                    sim.trigger_fx_list(fx, &pos);
                }
                if let Some(ocl) = ocl {
                    let pos = random_surface_position(sim, &effects.geometry, effects.height);
                    sim.spawn_ocl(ocl, owner, &pos, &pos);
                }
            }
        }
    }

    /// `handleObjectsOnBridgeOnDie`.
    fn drop_objects_on_deck(sim: &mut dyn SimApi, owner: ObjectSlot) {
        let center = position_of(sim, owner);
        let Some(bridge) = sim.find_bridge_at(&center) else {
            return;
        };
        let g = &bridge.geometry;
        let polygon = [g.from_left, g.from_right, g.to_right, g.to_left];
        let mut low = polygon[0].z;
        for corner in &polygon {
            if corner.z < low {
                low = corner.z;
            }
        }
        let radius = Coord2D {
            x: g.to_left.x - center.x,
            y: g.to_left.y - center.y,
        }
        .length();
        for other in sim.find_objects_in_range(&center, radius) {
            if sim.object_is_kind_of(other, KindOfType::Bridge) != FALSE
                || sim.object_is_kind_of(other, KindOfType::BridgeTower) != FALSE
            {
                continue;
            }
            if sim.object_is_above_terrain(other) != FALSE {
                continue;
            }
            let pos = position_of(sim, other);
            if pos.z < low || !point_inside_area_2d(&pos, &polygon) {
                continue;
            }
            if sim.object_layer(other) != Some(bridge.layer) {
                continue;
            }
            sim.set_object_layer(other, PathfindLayerEnum::Ground);
            if sim.has_physics(other) != FALSE {
                sim.set_physics_allow_to_fall(other, TRUE);
            } else {
                kill_object(sim, other, DamageType::Unresistable, DeathType::Normal);
            }
        }
    }

    /// `setScaffoldData`.
    fn start_scaffold(&self, sim: &mut dyn SimApi, piece: ObjectSlot, path: &ScaffoldPath) {
        let mut sunken = path.rise_to;
        sunken.z = sunken.z - path.height - SCAFFOLD_FUDGE;
        sim.set_object_position(piece, &sunken);
        sim.with_bridge_scaffold(piece, &mut |scaffold| {
            scaffold.set_positions(&sunken, &path.rise_to, &path.build);
            scaffold.set_motion(ScaffoldTargetMotion::Rise);
        });
        sim.set_object_orientation(piece, path.angle);
        let from_rise = |p: &Coord3D| {
            Coord3D::new(
                p.x - path.rise_to.x,
                p.y - path.rise_to.y,
                p.z - path.rise_to.z,
            )
        };
        let lateral = self.data.lateral_scaffold_speed
            * reference_div_f32(
                from_rise(&path.build).length(),
                from_rise(&path.center).length(),
            );
        let vertical = self.data.vertical_scaffold_speed;
        sim.with_bridge_scaffold(piece, &mut |scaffold| {
            scaffold.set_lateral_speed(lateral);
            scaffold.set_vertical_speed(vertical);
        });
    }

    /// `createScaffolding`.
    fn raise_scaffold(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if self.scaffold_present == TRUE {
            return;
        }
        let center = position_of(sim, owner);
        // The reference does not check for a bridge or a definition.
        let Some(bridge) = sim.find_bridge_at(&center) else {
            return;
        };
        let Some(road) = road_of(sim, &bridge) else {
            return;
        };
        let geometry_of = |sim: &dyn SimView, name: &str| {
            sim.rules()
                .thing_factory
                .find_template_final(name)
                .map(|t| t.get_template_geometry_info().clone())
        };
        let Some(piece) = geometry_of(sim, &road.scaffold_object_name) else {
            return;
        };
        let Some(support) = geometry_of(sim, &road.scaffold_support_object_name) else {
            return;
        };
        let spacing = piece.get_major_radius() * 2.0;
        let height = piece.get_max_height_above_position() + piece.get_max_height_below_position();
        let support_height =
            support.get_max_height_above_position() + support.get_max_height_below_position();
        let g = &bridge.geometry;
        let middle = |a: &Coord3D, b: &Coord3D| {
            Coord3D::new(
                ((a.x - b.x) / 2.0) + b.x,
                ((a.y - b.y) / 2.0) + b.y,
                ((a.z - b.z) / 2.0) + b.z,
            )
        };
        let left_start = middle(&g.from_left, &g.from_right);
        let right_start = middle(&g.to_left, &g.to_right);
        let left_angle = Coord2D {
            x: right_start.x - left_start.x,
            y: right_start.y - left_start.y,
        }
        .to_angle();
        let right_angle = left_angle + std::f32::consts::TAU;
        let mut left_vector = Coord3D::new(
            right_start.x - left_start.x,
            right_start.y - left_start.y,
            right_start.z - left_start.z,
        );
        let mut right_vector = Coord3D::new(
            left_start.x - right_start.x,
            left_start.y - right_start.y,
            left_start.z - right_start.z,
        );
        let objects = real_to_int_ceil(div_safe_f32(left_vector.length(), spacing)).wrapping_add(1);
        let iterations = real_to_int_ceil(objects as Real / 2.0);
        left_vector.normalize();
        right_vector.normalize();
        let team = sim
            .object(owner)
            .map_or_else(|| sim.neutral_team_id(), |o| o.team_id);

        let mut created: Int = 0;
        for i in 0..iterations {
            let ends = [
                (left_start, left_angle, left_vector, false),
                (right_start, right_angle, right_vector, true),
            ];
            for (start, angle, vector, second) in ends {
                if second && created >= objects {
                    continue;
                }
                let Some(obj) = sim.new_object_from_template(&road.scaffold_object_name, team)
                else {
                    return;
                };
                let along = spacing * i as Real;
                let mut path = ScaffoldPath {
                    angle,
                    height,
                    rise_to: start,
                    build: Coord3D::new(
                        vector.x * along + start.x + 0.1,
                        vector.y * along + start.y,
                        vector.z * along + start.z,
                    ),
                    center,
                };
                self.start_scaffold(sim, obj, &path);
                created = created.wrapping_add(1);
                self.keep_scaffold(sim, obj);

                // Supports stacked below until one would start under height zero.
                let mut offset = start.z;
                path.height = support_height;
                while offset >= 0.0 {
                    path.rise_to.z -= support_height;
                    path.build.z -= support_height;
                    path.center.z -= support_height;
                    let Some(obj) =
                        sim.new_object_from_template(&road.scaffold_support_object_name, team)
                    else {
                        return;
                    };
                    self.start_scaffold(sim, obj, &path);
                    self.keep_scaffold(sim, obj);
                    offset -= support_height;
                    // The reference loops for ever on a support without height.
                    if support_height <= 0.0 {
                        break;
                    }
                }
            }
        }
        self.scaffold_present = TRUE;
        sim.change_bridge_state(bridge.layer, FALSE);
    }

    fn keep_scaffold(&mut self, sim: &dyn SimView, piece: ObjectSlot) {
        if let Some(obj) = sim.object(piece) {
            self.scaffold_ids.push(obj.id);
        }
    }

    /// The towers' share of damage or healing done to the bridge itself, `None` for
    /// what a tower did.
    fn tower_share(sim: &dyn SimView, owner: ObjectSlot, dmg: &DamageInfo) -> Option<Real> {
        let percentage = dmg.input.amount / sim.object_max_health(owner);
        let from_tower = sim
            .find_object(dmg.input.source_id)
            .is_some_and(|s| sim.object_is_kind_of(s, KindOfType::BridgeTower) != FALSE);
        (!from_tower).then_some(percentage)
    }
}

impl BridgeFacet for BridgeBehavior {
    fn set_tower(&mut self, tower_type: BridgeTowerType, tower_id: ObjectId) {
        if let Some(slot) = self.tower_ids.get_mut(tower_type.as_i32() as usize) {
            *slot = tower_id;
        }
    }

    fn get_tower_id(&self, tower_type: BridgeTowerType) -> ObjectId {
        self.tower_ids
            .get(tower_type.as_i32() as usize)
            .copied()
            .unwrap_or(ObjectId::INVALID)
    }

    fn create_scaffolding(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.raise_scaffold(sim, ctx.owner_slot);
    }

    /// `removeScaffolding`.
    fn remove_scaffolding(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        if self.scaffold_present == FALSE {
            return;
        }
        for &id in &self.scaffold_ids {
            if let Some(obj) = sim.find_object(id) {
                sim.with_bridge_scaffold(obj, &mut |scaffold| scaffold.reverse_motion());
            }
        }
        self.scaffold_ids.clear();
        self.scaffold_present = FALSE;
        let owner = ctx.owner_slot;
        if sim.object_body_damage_state(owner) != BodyDamageType::Rubble {
            if let Some(bridge) = sim.find_bridge_at(&position_of(sim, owner)) {
                sim.change_bridge_state(bridge.layer, TRUE);
            }
        }
    }

    fn is_scaffold_in_motion(&self, sim: &dyn SimView) -> Bool {
        let moving = self.scaffold_ids.iter().any(|&id| {
            sim.find_object(id)
                .and_then(|obj| sim.bridge_scaffold_motion(obj))
                .is_some_and(|motion| motion != ScaffoldTargetMotion::Still)
        });
        Bool::from(moving)
    }

    fn is_scaffold_present(&self) -> Bool {
        self.scaffold_present
    }
}

impl DamageFacet for BridgeBehavior {
    fn on_damage(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let Some(percentage) = Self::tower_share(sim, ctx.owner_slot, dmg) else {
            return;
        };
        let towers = self.tower_ids;
        let source = ctx.owner_id;
        let (damage_type, death_type) = (dmg.input.damage_type, dmg.input.death_type);
        lend_self(self, sim, ctx, |sim| {
            for id in towers {
                let Some(tower) = sim.find_object(id) else {
                    continue;
                };
                let mut passed = DamageInfo::default();
                passed.input.amount = percentage * sim.object_max_health(tower);
                passed.input.source_id = source;
                passed.input.damage_type = damage_type;
                passed.input.death_type = death_type;
                sim.attempt_damage(tower, &mut passed);
            }
        });
    }

    fn on_healing(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &mut DamageInfo) {
        let Some(percentage) = Self::tower_share(sim, ctx.owner_slot, dmg) else {
            return;
        };
        let towers = self.tower_ids;
        let source = ctx.owner_id;
        lend_self(self, sim, ctx, |sim| {
            for id in towers {
                let Some(tower) = sim.find_object(id) else {
                    continue;
                };
                let mut healing = DamageInfo::default();
                healing.input.damage_type = DamageType::Healing;
                healing.input.death_type = DeathType::NoneType;
                healing.input.source_id = source;
                healing.input.amount = percentage * sim.object_max_health(tower);
                sim.attempt_healing(tower, &mut healing);
            }
        });
    }

    fn on_body_damage_state_change(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        _dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        if new_state != BodyDamageType::Rubble {
            self.death_frame = 0;
        }
        let owner = ctx.owner_slot;
        if self.transitions.is_none() {
            self.resolve_transitions(sim, owner);
        }
        let Some(transitions) = self.transitions.as_deref() else {
            return;
        };
        let Some(bridge) = sim.find_bridge_at(&position_of(sim, owner)) else {
            return;
        };
        let Some(road) = road_of(sim, &bridge) else {
            return;
        };
        // `IS_CONDITION_WORSE( oldState, newState )`: it was worse before.
        let (ocl, fx) = if old_state as i32 > new_state as i32 {
            (&transitions.repair_ocl, &transitions.repair_fx)
        } else {
            (&transitions.damage_ocl, &transitions.damage_fx)
        };
        let state = new_state as usize;
        let effects = AreaEffects {
            geometry: bridge.geometry,
            count: road.num_fx_per_type,
            height: road.transition_effects_height,
            lists: std::array::from_fn(|i| {
                let name = |names: &Resolved| names.get(state).and_then(|s| s[i].clone());
                (name(ocl), name(fx))
            }),
        };
        lend_self(self, sim, ctx, |sim| {
            Self::area_effects(sim, owner, &effects);
            sim.update_bridge_damage_states();
        });
    }
}

impl DieFacet for BridgeBehavior {
    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, _dmg: &DamageInfo) {
        let towers = self.tower_ids;
        let owner = ctx.owner_slot;
        lend_self(self, sim, ctx, |sim| {
            for id in towers {
                if let Some(tower) = sim.find_object(id) {
                    kill_object(sim, tower, DamageType::Unresistable, DeathType::Normal);
                }
            }
            Self::drop_objects_on_deck(sim, owner);
        });
        self.death_frame = sim.frame();
    }
}

impl UpdateFacet for BridgeBehavior {
    /// The die effects, each at its delay after the bridge fell. A position is drawn
    /// whether or not the effect exists.
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        if self.death_frame == 0 {
            return UpdateSleepTime::NONE;
        }
        let owner = ctx.owner_slot;
        let surface = sim
            .find_bridge_at(&position_of(sim, owner))
            .and_then(|b| Some((b.geometry, road_of(sim, &b)?.transition_effects_height)));
        let elapsed = sim.frame().wrapping_sub(self.death_frame);
        let surface_or_own = |sim: &mut dyn SimApi| match &surface {
            Some((geometry, height)) => random_surface_position(sim, geometry, *height),
            None => position_of(sim, owner),
        };
        for effect in &self.data.die_fx {
            if elapsed != effect.delay {
                continue;
            }
            let pos = if effect.bone_name.is_empty() {
                surface_or_own(sim)
            } else {
                bone_position(sim, owner, &effect.bone_name)
            };
            if let Some(name) = &effect.name {
                sim.trigger_fx_list(name, &pos);
            }
        }
        for effect in &self.data.die_ocl {
            if elapsed != effect.delay {
                continue;
            }
            let pos = if effect.bone_name == "ParentObject" {
                position_of(sim, owner)
            } else if effect.bone_name.is_empty() {
                surface_or_own(sim)
            } else {
                bone_position(sim, owner, &effect.bone_name)
            };
            if let Some(name) = &effect.name {
                sim.spawn_ocl(name, owner, &pos, &pos);
            }
        }
        UpdateSleepTime::NONE
    }
}

impl BehaviorModule for BridgeBehavior {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key()
    }

    fn on_object_created(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn on_delete(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext) {
        self.scaffold_ids.clear();
    }

    /// The destructor destroys the towers.
    fn on_destruct(&mut self, sim: &mut dyn SimApi, _ctx: ModuleContext) {
        for id in self.tower_ids {
            if let Some(tower) = sim.find_object(id) {
                sim.destroy_object(tower);
            }
        }
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        for id in &mut self.tower_ids {
            xfer.xfer_object_id(id)?;
        }
        xfer.xfer_bool(&mut self.scaffold_present)?;
        let mut count = self.scaffold_ids.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Load {
            for _ in 0..count {
                let mut id = ObjectId::INVALID;
                xfer.xfer_object_id(&mut id)?;
                self.scaffold_ids.push(id);
            }
        } else {
            for id in &mut self.scaffold_ids {
                xfer.xfer_object_id(id)?;
            }
        }
        xfer.xfer_unsigned_int(&mut self.death_frame)
    }

    /// The reference hands the bridge its ids while loading; the terrain is reached
    /// from this call.
    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
        let pos = position_of(sim, ctx.owner_slot);
        sim.restore_bridge_record(&pos, ctx.owner_id, self.tower_ids);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }

    fn as_die(&self) -> Option<&dyn DieFacet> {
        Some(self)
    }

    fn as_die_mut(&mut self) -> Option<&mut dyn DieFacet> {
        Some(self)
    }

    fn as_bridge(&self) -> Option<&dyn BridgeFacet> {
        Some(self)
    }

    fn as_bridge_mut(&mut self) -> Option<&mut dyn BridgeFacet> {
        Some(self)
    }
}

pub fn create_bridge_behavior(
    data: &dyn ModuleData,
    _ctx: ModuleContext,
) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BridgeBehaviorModuleData>()
        .expect("BridgeBehavior data type mismatch")
        .clone();
    Box::new(BridgeBehavior::new(typed))
}
