//! `PartitionManager`, searches over the cells filed by [`PartitionGrid`].
//!
//! Source: `GameLogic/Object/PartitionManager.cpp` (`getClosestObjects` in its
//! `FASTER_GCO` form, `iterateObjectsInRange`, `iterateAllObjects`, the four
//! distance procedures) and `GameLogic/Object/SimpleObjectIterator.cpp`.
//! Scope note: `docs/PARTITION_SCOPE.md`.
//! Golden test: `crates/zh_logic/tests/golden_partition_query.rs`
//! (harness `tests/cpp_harness/gen_partition_query.cpp`).
//!
//! # What a search sees
//!
//! * A range or closest search walks the cells as the last `update` filed them.
//!   An object registered since then is in no cell and is not found. An object
//!   that moved since then is still looked for in its old cells.
//! * Distances use the data `lookup` returns now, not the data at filing.
//! * The ring walk stops at [`PartitionGrid::max_gco_radius`] rings around the
//!   center cell, so a center far off the map can find nothing at any radius.
//! * [`PartitionGrid::iterate_all_objects`] walks the registrations instead, so
//!   it includes objects that are not filed yet or are off the map.
//!
//! # Filters
//!
//! The `PartitionFilter` array of the C++ is one caller predicate (`true` keeps
//! the object); the caller chains its filters in array order and stops at the
//! first rejection. The predicate may have side effects:
//!
//! * range search: called once for each object that is found in a visited cell
//!   and is strictly closer than the radius, in ring, offset and cell list order;
//! * closest search: the same, except that the limit shrinks to the distance of
//!   the best accepted object, so later objects that are not strictly closer than
//!   it are never offered. A rejected object does not shrink the limit;
//! * all objects: called once for each registered object, latest registration
//!   first.
//!
//! The predicate cannot start a search on the same grid in the C++ (the routine
//! is not reentrant); that case is not modelled.
//!
//! # Retail features
//!
//! When a closest search finds nothing, the default build returns the radius in
//! all three components of the vector. The retail build leaves it uninitialized;
//! under `retail-crc` it is zero here.

use std::collections::BTreeSet;

use zh_common::math::{gm_sqrtf, Coord3D};
use zh_common::types::{Int, ObjectId, Real};

use super::partition_grid::{PartitionGrid, PartitionObject};

/// `HUGE_DIST`: a radius at least this large does not limit the ring walk.
pub const HUGE_DIST: Real = 1_000_000.0;

/// `DistanceCalculationType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum DistanceCalculationType {
    FromCenter2d = 0,
    FromCenter3d = 1,
    FromBoundingSphere2d = 2,
    FromBoundingSphere3d = 3,
}

/// `IterOrderType`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum IterOrderType {
    /// Unsorted: the reverse of the order the search found the objects.
    Fastest = 0,
    SortedNearToFar = 1,
    SortedFarToNear = 2,
    SortedCheapToExpensive = 3,
    SortedExpensiveToCheap = 4,
}

/// Where a search is measured from: the two C++ overloads.
#[derive(Debug, Clone, Copy)]
pub enum QueryCenter<'a> {
    /// From an object. It is never part of the result, and its geometry counts
    /// in the bounding sphere rules. It need not be registered.
    Object(&'a PartitionObject),
    /// From a point.
    Position(Coord3D),
}

/// The result of a closest object search.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClosestObject {
    pub id: Option<ObjectId>,
    /// `closestDist`: the distance to `id` under the rule used. With no object,
    /// the square root of the squared radius.
    pub dist: Real,
    /// `closestDistVec`: from the center toward `id`, shortened by the
    /// bounding radii under the bounding sphere rules.
    pub vec: Coord3D,
}

/// One `SimpleObjectIterator` entry.
#[derive(Debug, Clone, Copy)]
struct Clump {
    id: ObjectId,
    /// The squared distance.
    numeric: Real,
    build_cost: Int,
}

/// The common part of the four `distCalcProc_*` functions: the squared distance
/// and the vector from the center to `other`.
fn measure(
    rule: DistanceCalculationType,
    center_pos: &Coord3D,
    center: Option<&PartitionObject>,
    other: &PartitionObject,
) -> (Real, Coord3D) {
    use DistanceCalculationType as Rule;
    let mut diff = Coord3D {
        x: other.pos.x - center_pos.x,
        y: other.pos.y - center_pos.y,
        z: 0.0,
    };
    match rule {
        Rule::FromCenter2d => (diff.x * diff.x + diff.y * diff.y, diff),
        Rule::FromCenter3d => {
            diff.z = other.pos.z - center_pos.z;
            (diff.x * diff.x + diff.y * diff.y + diff.z * diff.z, diff)
        }
        Rule::FromBoundingSphere2d => {
            let dist_sqr = diff.x * diff.x + diff.y * diff.y;
            let total = center.map_or(0.0, |c| c.geometry.get_bounding_circle_radius())
                + other.geometry.get_bounding_circle_radius();
            let (dist_sqr, factor) = shrink(dist_sqr, total);
            diff.x *= factor;
            diff.y *= factor;
            (dist_sqr, diff)
        }
        Rule::FromBoundingSphere3d => {
            let center_delta = center.map_or(0.0, |c| c.geometry.get_z_delta_to_center_position());
            diff.z = (other.pos.z + other.geometry.get_z_delta_to_center_position())
                - (center_pos.z + center_delta);
            let dist_sqr = diff.x * diff.x + diff.y * diff.y + diff.z * diff.z;
            let total = center.map_or(0.0, |c| c.geometry.get_bounding_sphere_radius())
                + other.geometry.get_bounding_sphere_radius();
            let (dist_sqr, factor) = shrink(dist_sqr, total);
            diff.x *= factor;
            diff.y *= factor;
            diff.z *= factor;
            (dist_sqr, diff)
        }
    }
}

/// `PartitionManager::getDistanceSquared` between an object and another object or, when the
/// other has no geometry, a bare position.
pub fn measure_distance_squared(
    rule: DistanceCalculationType,
    center_pos: &Coord3D,
    center: Option<&PartitionObject>,
    other: &PartitionObject,
) -> Real {
    measure(rule, center_pos, center, other).0
}

/// Takes `total` off a distance given squared. Returns the new squared distance
/// and the factor for the vector.
pub(crate) fn shrink(dist_sqr: Real, total: Real) -> (Real, Real) {
    if total > 0.0 {
        let dist = gm_sqrtf(dist_sqr);
        let shrunk = dist - total;
        if shrunk <= 0.0 {
            (0.0, 0.0)
        } else {
            (shrunk * shrunk, shrunk / dist)
        }
    } else {
        (dist_sqr, 1.0)
    }
}

/// The merge sort of `SimpleObjectIterator::sort` over any list.
pub(crate) fn iterator_merge_sort<T: Copy>(clumps: &mut Vec<T>, compare: impl Fn(&T, &T) -> Real) {
    let len = clumps.len();
    let mut merged = Vec::with_capacity(len);
    let mut width = 1;
    while width < len {
        merged.clear();
        for pair in clumps.chunks(width * 2) {
            let (left, right) = pair.split_at(width.min(pair.len()));
            let (mut l, mut r) = (0, 0);
            while l < left.len() && r < right.len() {
                if compare(&left[l], &right[r]) <= 0.0 {
                    merged.push(left[l]);
                    l += 1;
                } else {
                    merged.push(right[r]);
                    r += 1;
                }
            }
            merged.extend_from_slice(&left[l..]);
            merged.extend_from_slice(&right[r..]);
        }
        std::mem::swap(clumps, &mut merged);
        width *= 2;
    }
}

/// `SimpleObjectIterator::sort`: a bottom-up merge sort that takes from the left
/// run while `compare(left, right) <= 0`, so equal entries keep their order.
fn sort_clumps(clumps: &mut Vec<Clump>, order: IterOrderType) {
    use IterOrderType as Order;
    let compare = |a: &Clump, b: &Clump| -> Real {
        match order {
            Order::Fastest => 0.0,
            Order::SortedNearToFar => a.numeric - b.numeric,
            Order::SortedFarToNear => b.numeric - a.numeric,
            Order::SortedCheapToExpensive => a.build_cost.wrapping_sub(b.build_cost) as Real,
            Order::SortedExpensiveToCheap => b.build_cost.wrapping_sub(a.build_cost) as Real,
        }
    };
    if order == Order::Fastest {
        return;
    }
    iterator_merge_sort(clumps, compare);
}

impl PartitionGrid {
    /// `getClosestObjects`. Calls `accept` for each object that passes the
    /// distance test and the filter. With `closest_only`, each accepted object
    /// becomes the new limit.
    #[allow(clippy::too_many_arguments)]
    fn search(
        &self,
        center: QueryCenter<'_>,
        max_dist: Real,
        rule: DistanceCalculationType,
        closest_only: bool,
        lookup: &dyn Fn(ObjectId) -> Option<PartitionObject>,
        filter: &mut dyn FnMut(ObjectId) -> bool,
        accept: &mut dyn FnMut(&PartitionObject, Real, Coord3D),
    ) -> Real {
        let (center_pos, center_object) = match center {
            QueryCenter::Object(object) => (object.pos, Some(object)),
            QueryCenter::Position(pos) => (pos, None),
        };
        let center_id = center_object.map(|object| object.id);
        let (center_x, center_y) = self.world_to_cell(center_pos.x, center_pos.y);
        let mut limit_sqr = max_dist * max_dist;

        let mut max_radius = self.max_gco_radius();
        if max_dist < HUGE_DIST {
            max_radius = max_radius.min(self.world_to_cell_dist(max_dist));
        }

        let mut found_any = false;
        // The done flags of the C++: each object is considered once.
        let mut done = BTreeSet::new();
        let mut radius = 0;
        while radius <= max_radius {
            for &(dx, dy) in self.radius_offsets(radius) {
                let Some(cell) =
                    self.cell_index(center_x.wrapping_add(dx), center_y.wrapping_add(dy))
                else {
                    continue;
                };
                for id in self.cell_objects(cell) {
                    if Some(id) == center_id || done.contains(&id) {
                        continue;
                    }
                    let Some(other) = lookup(id) else {
                        continue;
                    };
                    done.insert(id);
                    let (dist_sqr, vec) = measure(rule, &center_pos, center_object, &other);
                    // Strict, and false for NaN.
                    let in_range = dist_sqr < limit_sqr;
                    if !in_range || !filter(id) {
                        continue;
                    }
                    accept(&other, dist_sqr, vec);
                    if closest_only {
                        limit_sqr = dist_sqr;
                        if !found_any {
                            // Finish this ring, then stop.
                            max_radius = radius;
                        }
                        found_any = true;
                    }
                }
            }
            radius += 1;
        }
        limit_sqr
    }

    /// `PartitionManager::getClosestObject`.
    ///
    /// Only objects strictly closer than `max_dist` count, and among equally
    /// close objects the first one visited wins. The search finishes the ring in
    /// which it first accepts an object and looks no further out, so the result
    /// is not always the closest object under the distance rule.
    pub fn get_closest_object(
        &self,
        center: QueryCenter<'_>,
        max_dist: Real,
        rule: DistanceCalculationType,
        lookup: &dyn Fn(ObjectId) -> Option<PartitionObject>,
        filter: &mut dyn FnMut(ObjectId) -> bool,
    ) -> ClosestObject {
        #[cfg(not(feature = "retail-crc"))]
        let initial = Coord3D {
            x: max_dist,
            y: max_dist,
            z: max_dist,
        };
        #[cfg(feature = "retail-crc")]
        let initial = Coord3D {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
        let mut id = None;
        let mut vec = initial;
        let limit_sqr = self.search(
            center,
            max_dist,
            rule,
            true,
            lookup,
            filter,
            &mut |other, _, to_other| {
                id = Some(other.id);
                vec = to_other;
            },
        );
        ClosestObject {
            id,
            dist: gm_sqrtf(limit_sqr),
            vec,
        }
    }

    /// `PartitionManager::iterateObjectsInRange`: the objects strictly closer
    /// than `max_dist`, with their squared distances, in iteration order.
    pub fn iterate_objects_in_range(
        &self,
        center: QueryCenter<'_>,
        max_dist: Real,
        rule: DistanceCalculationType,
        order: IterOrderType,
        lookup: &dyn Fn(ObjectId) -> Option<PartitionObject>,
        filter: &mut dyn FnMut(ObjectId) -> bool,
    ) -> Vec<(ObjectId, Real)> {
        let mut clumps = Vec::new();
        self.search(
            center,
            max_dist,
            rule,
            false,
            lookup,
            filter,
            &mut |other, dist_sqr, _| {
                clumps.push(Clump {
                    id: other.id,
                    numeric: dist_sqr,
                    build_cost: other.build_cost,
                });
            },
        );
        // `SimpleObjectIterator::insert` puts each object at the head.
        clumps.reverse();
        sort_clumps(&mut clumps, order);
        clumps.iter().map(|c| (c.id, c.numeric)).collect()
    }

    /// `PartitionManager::iterateAllObjects`: every registered object the filter
    /// keeps, in iteration order (earliest registration first).
    pub fn iterate_all_objects(&self, filter: &mut dyn FnMut(ObjectId) -> bool) -> Vec<ObjectId> {
        let mut ids: Vec<ObjectId> = self.registered_objects().filter(|&id| filter(id)).collect();
        ids.reverse();
        ids
    }
}
