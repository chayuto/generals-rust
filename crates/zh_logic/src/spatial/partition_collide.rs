//! `PartitionManager` collision detection: the shape tests and the contact list.
//!
//! Source: `GameLogic/Object/PartitionManager.cpp`.
//! Golden test: `crates/zh_logic/tests/golden_partition_collide.rs`
//! (harness `tests/cpp_harness/gen_partition_collide.cpp`).
//!
//! `PartitionGrid::update` gathers the contacts; [`process_contacts`] is
//! `PartitionContactList::processContactList`. The caller runs it after `update`
//! and after handling the returned cell changes.
//!
//! # Uninitialised C++ output
//!
//! When two non-sphere shapes pass the "close enough" preflight, the C++ never
//! calls the XY test, so `loc.x`, `loc.y` and all of `normal` keep whatever the
//! caller's stack held. Those values cannot be reproduced. Here they are zero
//! (so the second object of a pair receives a normal of negative zeros), and the
//! golden vectors do not compare them.
//!
//! `Coord3D::length` calls the C library `sqrt` under `RETAIL_COMPATIBLE_CRC`.
//! A square root rounded from double equals the single precision one, so the
//! normalisation here needs no `retail-crc` branch.

use zh_common::math::gamemath::gm_fabsf;
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{gm_cosf, gm_sinf, gm_sqrtf, Coord2D, Coord3D};
use zh_common::types::{Int, ObjectId, Real, FALSE};

use super::partition_grid::{PartitionGrid, PartitionObject};

/// What a shape test reads from one object.
#[derive(Debug, Clone)]
pub struct CollideInfo {
    pub pos: Coord3D,
    pub geometry: GeometryInfo,
    pub angle: Real,
}

/// The contact point and normal of a collision.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct CollideLocAndNormal {
    pub loc: Coord3D,
    pub normal: Coord3D,
}

/// What contact processing needs from the owner of the grid.
pub trait CollisionHost {
    fn grid_mut(&mut self) -> &mut PartitionGrid;
    /// The object's current data, or `None` if it no longer exists.
    fn partition_object(&self, id: ObjectId) -> Option<PartitionObject>;
    /// `KINDOF_NO_COLLIDE`.
    fn is_no_collide(&self, id: ObjectId) -> bool;
    /// `OBJECT_STATUS_NO_COLLISIONS`.
    fn has_no_collisions_status(&self, id: ObjectId) -> bool;
    /// `Object::isDestroyed`.
    fn is_destroyed(&self, id: ObjectId) -> bool;
    /// `Object::onCollide`. It can be called on an object whose
    /// `OBJECT_STATUS_NO_COLLISIONS` the partner's reaction has just set: the C++
    /// `onCollide` tests that status before each collide module, so the
    /// implementation must do the same.
    fn on_collide(&mut self, id: ObjectId, other: ObjectId, loc: &Coord3D, normal: &Coord3D);
}

type CollideTestProc = fn(&CollideInfo, &CollideInfo, &mut CollideLocAndNormal) -> bool;

#[inline]
fn sqr(x: Real) -> Real {
    x * x
}

fn vec_diff_2d(a: &Coord3D, b: &Coord3D) -> Coord3D {
    Coord3D {
        x: a.x - b.x,
        y: a.y - b.y,
        z: 0.0,
    }
}

fn vec_diff_3d(a: &Coord3D, b: &Coord3D) -> Coord3D {
    Coord3D {
        x: a.x - b.x,
        y: a.y - b.y,
        z: a.z - b.z,
    }
}

fn project_coord3d(coord: &mut Coord3D, unit_dir: &Coord3D, dist: Real) {
    coord.x += unit_dir.x * dist;
    coord.y += unit_dir.y * dist;
    coord.z += unit_dir.z * dist;
}

fn flip_coord3d(coord: &mut Coord3D) {
    coord.x = -coord.x;
    coord.y = -coord.y;
    coord.z = -coord.z;
}

fn test_rotated_points_against_rect(
    pts: &[Coord2D; 4],
    a: &CollideInfo,
    avg: &mut Coord2D,
    avg_tot: &mut Int,
) {
    let major = a.geometry.get_major_radius();
    let minor = if a.geometry.get_geom_type() == GeometryType::Sphere {
        a.geometry.get_major_radius()
    } else {
        a.geometry.get_minor_radius()
    };

    let c = gm_cosf(-a.angle);
    let s = gm_sinf(-a.angle);

    for pt in pts {
        // Delta relative to the rectangle centre, rotated into its frame.
        let ptx = pt.x - a.pos.x;
        let pty = pt.y - a.pos.y;

        let ptx_new = gm_fabsf(ptx * c - pty * s);
        let pty_new = gm_fabsf(ptx * s + pty * c);

        if ptx_new <= major && pty_new <= minor {
            avg.x += pt.x;
            avg.y += pt.y;
            *avg_tot += 1;
        }
    }
}

fn rect_to_four_points(a: &CollideInfo) -> [Coord2D; 4] {
    let c = gm_cosf(a.angle);
    let s = gm_sinf(a.angle);
    let exc = a.geometry.get_major_radius() * c;
    let eyc = a.geometry.get_minor_radius() * c;
    let exs = a.geometry.get_major_radius() * s;
    let eys = a.geometry.get_minor_radius() * s;

    let mut pts = [Coord2D::default(); 4];
    pts[0].x = a.pos.x - exc - eys;
    pts[0].y = a.pos.y + eyc - exs;
    pts[1].x = a.pos.x + exc - eys;
    pts[1].y = a.pos.y + eyc + exs;
    pts[2].x = a.pos.x - exc + eys;
    pts[2].y = a.pos.y - eyc - exs;
    pts[3].x = a.pos.x + exc + eys;
    pts[3].y = a.pos.y - eyc + exs;
    pts
}

fn xy_circle_rect(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let result = xy_rect_circle(b, a, cinfo);
    flip_coord3d(&mut cinfo.normal);
    result
}

/// The circle is treated as a square of its radius, at its own angle.
fn xy_rect_circle(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let mut btmp = b.clone();
    let major = btmp.geometry.get_major_radius();
    btmp.geometry.set_minor_radius(major);
    xy_rect_rect(a, &btmp, cinfo)
}

fn xy_circle_circle(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let diff = vec_diff_2d(&b.pos, &a.pos);
    let dist_sqr = sqr(diff.x) + sqr(diff.y);
    let touching_dist_sqr = sqr(a.geometry.get_major_radius() + b.geometry.get_major_radius());
    if dist_sqr <= touching_dist_sqr {
        cinfo.normal = diff;
        cinfo.normal.normalize();
        cinfo.loc = a.pos;
        project_coord3d(&mut cinfo.loc, &cinfo.normal, a.geometry.get_major_radius());
        return true;
    }
    false
}

fn xy_rect_rect(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let mut avg = Coord2D { x: 0.0, y: 0.0 };
    let mut avg_tot: Int = 0;
    let pts = rect_to_four_points(a);
    test_rotated_points_against_rect(&pts, b, &mut avg, &mut avg_tot);
    let pts = rect_to_four_points(b);
    test_rotated_points_against_rect(&pts, a, &mut avg, &mut avg_tot);
    if avg_tot > 0 {
        cinfo.loc.x = avg.x / avg_tot as Real;
        cinfo.loc.y = avg.y / avg_tot as Real;
        cinfo.loc.z = (a.pos.z + b.pos.z) * 0.5;
        cinfo.normal = vec_diff_2d(&b.pos, &a.pos);
        cinfo.normal.normalize();
        return true;
    }
    false
}

fn z_sphere_nonsphere(
    xyproc: CollideTestProc,
    a: &CollideInfo,
    b: &CollideInfo,
    cinfo: &mut CollideLocAndNormal,
) -> bool {
    if a.pos.z >= b.pos.z && a.pos.z <= b.pos.z + b.geometry.get_max_height_above_position() {
        if xyproc(a, b, cinfo) {
            cinfo.loc.z = a.pos.z;
            return true;
        }
        return false;
    }

    let b_bot = b.pos.z;
    if a.pos.z < b_bot && a.pos.z + a.geometry.get_major_radius() >= b_bot {
        let mut amod = a.clone();
        amod.pos.z = b_bot;
        amod.geometry.set_major_radius(gm_sqrtf(
            sqr(a.geometry.get_major_radius()) - sqr(b_bot - a.pos.z),
        ));
        return xyproc(&amod, b, cinfo);
    }

    let b_top = b.pos.z + b.geometry.get_max_height_above_position();
    if a.pos.z > b_top && a.pos.z - a.geometry.get_major_radius() <= b_top {
        let mut amod = a.clone();
        amod.pos.z = b_top;
        amod.geometry.set_major_radius(gm_sqrtf(
            sqr(a.geometry.get_major_radius()) - sqr(a.pos.z - b_top),
        ));
        return xyproc(&amod, b, cinfo);
    }
    false
}

/// The caller has already established the z overlap.
fn z_nonsphere_nonsphere(
    xyproc: CollideTestProc,
    a: &CollideInfo,
    b: &CollideInfo,
    cinfo: &mut CollideLocAndNormal,
) -> bool {
    let d_sqr = sqr(a.pos.x - b.pos.x) + sqr(a.pos.y - b.pos.y);
    let mut min_radius = a.geometry.get_major_radius();
    let mut r = a.geometry.get_minor_radius();
    if min_radius > r {
        min_radius = r;
    }
    r = b.geometry.get_major_radius();
    if min_radius > r {
        min_radius = r;
    }
    r = b.geometry.get_minor_radius();
    if min_radius > r {
        min_radius = r;
    }

    let close_enough = sqr(min_radius) > d_sqr;

    if close_enough || xyproc(a, b, cinfo) {
        // Every collision is treated as side to side; only the height is set.
        if b.pos.z > a.pos.z {
            cinfo.loc.z = (b.pos.z + a.pos.z + a.geometry.get_max_height_above_position()) * 0.5;
        } else {
            cinfo.loc.z = (a.pos.z + b.pos.z + b.geometry.get_max_height_above_position()) * 0.5;
        }
        return true;
    }
    false
}

fn sphere_sphere(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let diff = vec_diff_3d(&b.pos, &a.pos);
    let dist_sqr = sqr(diff.x) + sqr(diff.y) + sqr(diff.z);
    let touching_dist_sqr = sqr(a.geometry.get_major_radius() + b.geometry.get_major_radius());
    if dist_sqr <= touching_dist_sqr {
        cinfo.normal = diff;
        cinfo.normal.normalize();
        cinfo.loc = a.pos;
        project_coord3d(&mut cinfo.loc, &cinfo.normal, a.geometry.get_major_radius());
        return true;
    }
    false
}

fn sphere_cylinder(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    z_sphere_nonsphere(xy_circle_circle, a, b, cinfo)
}

fn sphere_box(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    z_sphere_nonsphere(xy_circle_rect, a, b, cinfo)
}

fn cylinder_sphere(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let result = z_sphere_nonsphere(xy_circle_circle, b, a, cinfo);
    flip_coord3d(&mut cinfo.normal);
    result
}

fn cylinder_cylinder(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    z_nonsphere_nonsphere(xy_circle_circle, a, b, cinfo)
}

fn cylinder_box(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    z_nonsphere_nonsphere(xy_circle_rect, a, b, cinfo)
}

fn box_sphere(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    let result = z_sphere_nonsphere(xy_circle_rect, b, a, cinfo);
    flip_coord3d(&mut cinfo.normal);
    result
}

fn box_cylinder(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    z_nonsphere_nonsphere(xy_rect_circle, a, b, cinfo)
}

fn box_box(a: &CollideInfo, b: &CollideInfo, cinfo: &mut CollideLocAndNormal) -> bool {
    z_nonsphere_nonsphere(xy_rect_rect, a, b, cinfo)
}

/// `theCollideTestProcs`, indexed `first * 3 + second` by geometry type.
const COLLIDE_TEST_PROCS: [CollideTestProc; 9] = [
    sphere_sphere,
    sphere_cylinder,
    sphere_box,
    cylinder_sphere,
    cylinder_cylinder,
    cylinder_box,
    box_sphere,
    box_cylinder,
    box_box,
];

fn run_test_proc(a: &CollideInfo, b: &CollideInfo) -> Option<CollideLocAndNormal> {
    let index = a.geometry.get_geom_type().as_i32() * 3 + b.geometry.get_geom_type().as_i32();
    let mut cinfo = CollideLocAndNormal::default();
    if COLLIDE_TEST_PROCS[index as usize](a, b, &mut cinfo) {
        Some(cinfo)
    } else {
        None
    }
}

/// The geometric part of `PartitionData::collidesWith`: the height overlap, then
/// the shape test. `KINDOF_NO_COLLIDE` is the caller's to check.
pub fn collides_with(a: &CollideInfo, b: &CollideInfo) -> Option<CollideLocAndNormal> {
    let a_top = a.pos.z + a.geometry.get_max_height_above_position();
    let a_bot = a.pos.z - a.geometry.get_max_height_below_position();
    let b_top = b.pos.z + b.geometry.get_max_height_above_position();
    let b_bot = b.pos.z - b.geometry.get_max_height_below_position();

    if a_top >= b_bot && a_bot <= b_top {
        run_test_proc(a, b)
    } else {
        None
    }
}

/// `PartitionManager::geomCollidesWithGeom`. Unlike [`collides_with`], the height
/// test ignores how far either shape reaches below its position.
pub fn geom_collides_with_geom(a: &CollideInfo, b: &CollideInfo) -> bool {
    if a.pos.z + a.geometry.get_max_height_above_position() >= b.pos.z
        && a.pos.z <= b.pos.z + b.geometry.get_max_height_above_position()
    {
        run_test_proc(a, b).is_some()
    } else {
        false
    }
}

/// `PartitionData::collidesWith` for two objects of the host.
fn objects_collide<H: CollisionHost>(
    host: &H,
    a: ObjectId,
    b: ObjectId,
) -> Option<CollideLocAndNormal> {
    if host.is_no_collide(a) || host.is_no_collide(b) {
        return None;
    }
    let a = host.partition_object(a)?;
    let b = host.partition_object(b)?;
    let info = |object: PartitionObject| CollideInfo {
        pos: object.pos,
        geometry: object.geometry,
        angle: object.angle,
    };
    collides_with(&info(a), &info(b))
}

/// `PartitionManager::isColliding`: false unless both objects are registered.
pub fn is_colliding<H: CollisionHost>(host: &mut H, a: ObjectId, b: ObjectId) -> bool {
    let grid = host.grid_mut();
    if !grid.is_registered(a) || !grid.is_registered(b) {
        return false;
    }
    objects_collide(host, a, b).is_some()
}

/// `PartitionContactList::processContactList` over the contacts the last
/// `PartitionGrid::update` gathered, newest first. Positions are read now.
/// Unregistering an object, from `on_collide` or before this call, drops its
/// pending contacts.
pub fn process_contacts<H: CollisionHost>(host: &mut H) {
    while let Some((obj, other)) = host.grid_mut().take_contact() {
        let Some(mut cinfo) = objects_collide(host, obj, other) else {
            continue;
        };
        if host.has_no_collisions_status(obj) || host.has_no_collisions_status(other) {
            continue;
        }

        host.on_collide(obj, other, &cinfo.loc, &cinfo.normal);
        flip_coord3d(&mut cinfo.normal);
        if !host.is_destroyed(obj) && !host.is_destroyed(other) {
            host.on_collide(other, obj, &cinfo.loc, &cinfo.normal);
        }

        // Colliding mobile objects are checked again at the next update.
        for id in [obj, other] {
            if !host.is_destroyed(id)
                && host.grid_mut().is_registered(id)
                && host
                    .partition_object(id)
                    .is_some_and(|object| object.is_immobile == FALSE)
            {
                host.grid_mut().make_dirty_for_collision_check(id);
            }
        }
    }
}
