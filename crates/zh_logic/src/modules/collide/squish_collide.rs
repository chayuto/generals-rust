//! `PKT-246`: `SquishCollide` (`SquishCollide.h` / `SquishCollide.cpp`).

use crate::arena::{ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::{DamageInfo, HUGE_DAMAGE_AMOUNT};
use crate::modules::collide::collide_module::{
    impl_collide_behavior, parse_collide_module_data, CollideBehaviorImpl, CollideModule,
    CollideModuleData,
};
use crate::modules::module::{BehaviorModule, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{DamageType, DeathType};
use crate::sim::SimApi;
use zh_common::ini::{IniError, IniReader};
use zh_common::math::gamemath::{gm_cosf, gm_sinf};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, NameKeyType, Real, Relationship, FALSE};

pub type SquishCollideModuleData = CollideModuleData;

pub fn parse_squish_collide_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_collide_module_data(ini, tag_key, module_name_key, parse_ctx)
}

#[derive(Debug, Clone)]
pub struct SquishCollide {
    base: CollideModule,
}

impl SquishCollide {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<SquishCollideModuleData>()
            .expect("SquishCollideModuleData mismatch");
        Box::new(Self {
            base: CollideModule::new(typed.module_name_key),
        })
    }
}

#[inline]
fn clamp_real(v: Real, lo: Real, hi: Real) -> Real {
    if v < lo {
        lo
    } else if v > hi {
        hi
    } else {
        v
    }
}

fn circle_collides_box(
    circle_pos: &Coord3D,
    circle_r: Real,
    box_pos: &Coord3D,
    box_geom: &GeometryInfo,
    box_angle: Real,
) -> bool {
    let c = gm_cosf(box_angle);
    let s = gm_sinf(box_angle);
    let dx = circle_pos.x - box_pos.x;
    let dy = circle_pos.y - box_pos.y;
    let lx = dx * c + dy * s;
    let ly = -dx * s + dy * c;
    let maj = box_geom.get_major_radius();
    let min = box_geom.get_minor_radius();
    let cx = clamp_real(lx, -maj, maj);
    let cy = clamp_real(ly, -min, min);
    let rx = lx - cx;
    let ry = ly - cy;
    (rx * rx + ry * ry) <= (circle_r * circle_r)
}

fn box_collides_box(
    p1: &Coord3D,
    g1: &GeometryInfo,
    a1: Real,
    p2: &Coord3D,
    g2: &GeometryInfo,
    a2: Real,
) -> bool {
    let (c1, s1) = (gm_cosf(a1), gm_sinf(a1));
    let (c2, s2) = (gm_cosf(a2), gm_sinf(a2));
    let axes = [(c1, s1), (-s1, c1), (c2, s2), (-s2, c2)];
    let d_x = p2.x - p1.x;
    let d_y = p2.y - p1.y;
    let u1 = (c1, s1);
    let v1 = (-s1, c1);
    let u2 = (c2, s2);
    let v2 = (-s2, c2);
    for (ax, ay) in axes {
        let dist = (d_x * ax + d_y * ay).abs();
        let proj1 = g1.get_major_radius() * (u1.0 * ax + u1.1 * ay).abs()
            + g1.get_minor_radius() * (v1.0 * ax + v1.1 * ay).abs();
        let proj2 = g2.get_major_radius() * (u2.0 * ax + u2.1 * ay).abs()
            + g2.get_minor_radius() * (v2.0 * ax + v2.1 * ay).abs();
        if dist > proj1 + proj2 {
            return false;
        }
    }
    true
}

/// Evaluates `PartitionManager::geomCollidesWithGeom` in 3D/2D footprint space.
fn geom_collides_with_geom(
    pos1: &Coord3D,
    geom1: &GeometryInfo,
    angle1: Real,
    pos2: &Coord3D,
    geom2: &GeometryInfo,
    angle2: Real,
) -> Bool {
    let z1_lo = pos1.z - geom1.get_max_height_below_position();
    let z1_hi = pos1.z + geom1.get_max_height_above_position();
    let z2_lo = pos2.z - geom2.get_max_height_below_position();
    let z2_hi = pos2.z + geom2.get_max_height_above_position();
    if z1_hi < z2_lo || z2_hi < z1_lo {
        return FALSE;
    }

    let dx = pos2.x - pos1.x;
    let dy = pos2.y - pos1.y;
    let br_sum = geom1.get_bounding_circle_radius() + geom2.get_bounding_circle_radius();
    if (dx * dx + dy * dy) > (br_sum * br_sum) {
        return FALSE;
    }

    let hit = match (geom1.get_geom_type(), geom2.get_geom_type()) {
        (GeometryType::Box, GeometryType::Box) => {
            box_collides_box(pos1, geom1, angle1, pos2, geom2, angle2)
        }
        (GeometryType::Box, _) => {
            circle_collides_box(pos2, geom2.get_major_radius(), pos1, geom1, angle1)
        }
        (_, GeometryType::Box) => {
            circle_collides_box(pos1, geom1.get_major_radius(), pos2, geom2, angle2)
        }
        _ => {
            let r_sum = geom1.get_major_radius() + geom2.get_major_radius();
            (dx * dx + dy * dy) <= (r_sum * r_sum)
        }
    };
    Bool::from(hit)
}

impl CollideBehaviorImpl for SquishCollide {
    fn collide_base(&self) -> &CollideModule {
        &self.base
    }

    fn collide_base_mut(&mut self) -> &mut CollideModule {
        &mut self.base
    }

    fn on_collide(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        other: Option<ObjectSlot>,
        _loc: Option<&Coord3D>,
        _normal: Option<&Coord3D>,
    ) {
        let Some(other_slot) = other else {
            return;
        };

        if sim.object_has_ai(ctx.owner_slot) != FALSE
            && sim.object_ai_goal_object(ctx.owner_slot) == Some(other_slot)
        {
            let has_hijacker = sim
                .rules()
                .name_keys
                .lookup_key("HijackerUpdate")
                .is_some_and(|k| sim.has_module_by_name_key(ctx.owner_slot, k));
            if has_hijacker {
                return;
            }
        }

        let Some(self_obj) = sim.object(ctx.owner_slot) else {
            return;
        };
        if self_obj.is_undetected_defector() != FALSE {
            return;
        }
        let self_pos = *self_obj.thing.get_position();
        let self_angle = self_obj.thing.get_orientation();
        let mut my_geom = self_obj.geometry.clone();
        my_geom.set_major_radius(1.0);
        my_geom.set_minor_radius(1.0);

        if sim.object_crusher_level(other_slot) <= 0
            || sim.player_relationship(other_slot, ctx.owner_slot) == Relationship::Allies
        {
            return;
        }

        let Some(other_obj) = sim.object(other_slot) else {
            return;
        };
        let Some(phys_slot) = other_obj.physics_slot else {
            return;
        };
        let ModuleSidecar::Physics { velocity, .. } = sim.module_sidecar(phys_slot) else {
            return;
        };
        let other_id = other_obj.id;
        let other_pos = *other_obj.thing.get_position();
        let other_angle = other_obj.thing.get_orientation();
        let other_geom = other_obj.geometry.clone();

        if geom_collides_with_geom(
            &other_pos,
            &other_geom,
            other_angle,
            &self_pos,
            &my_geom,
            self_angle,
        ) == FALSE
        {
            return;
        }

        let to_x = self_pos.x - other_pos.x;
        let to_y = self_pos.y - other_pos.y;
        if (to_x * velocity.x + to_y * velocity.y) > 0.0_f32 {
            let mut dmg = DamageInfo::default();
            dmg.input.damage_type = DamageType::Crush;
            dmg.input.death_type = DeathType::Crushed;
            dmg.input.source_id = other_id;
            dmg.input.amount = HUGE_DAMAGE_AMOUNT;
            sim.attempt_damage(ctx.owner_slot, &mut dmg);
            if let Some(self_mut) = sim.object_mut(ctx.owner_slot) {
                self_mut.set_undetected_defector(FALSE);
            }
        }
    }
}

impl_collide_behavior!(SquishCollide);
