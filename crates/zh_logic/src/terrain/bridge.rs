//! The bridge half of `TerrainLogic` (`PKT-156`,
//! `Core/GameEngine/Source/GameLogic/Map/TerrainLogic.cpp`): `Bridge` geometry
//! and the bridge list queries.
//!
//! Golden test: `crates/zh_logic/tests/golden_s2_pathfind_map.rs`.
//!
//! Creating the bridge and tower objects is the caller's job: the ids are given
//! to [`Bridge::new`] and [`Bridge::set_tower`].

use zh_common::math::{gm_cosf, gm_sinf, Coord2D, Coord3D, Region2D};
use zh_common::types::{ObjectId, Real};

use crate::gamelogic::global_data::BodyDamageType;
use crate::pathfinder::pathfind_constants::PathfindLayerEnum;
use crate::terrain::terrain_logic::BridgeDeck;

const PATHFIND_CELL_SIZE: i32 = 10;
const PATHFIND_CELL_SIZE_F: Real = 10.0;
/// `LAYER_Z_CLOSE_ENOUGH_F`.
pub const LAYER_Z_CLOSE_ENOUGH_F: Real = 10.0;

/// The fixed shape of a bridge deck.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BridgeGeometry {
    /// Centre of the "from" end.
    pub from: Coord3D,
    pub to: Coord3D,
    pub from_left: Coord3D,
    pub from_right: Coord3D,
    pub to_left: Coord3D,
    pub to_right: Coord3D,
    pub bounds: Region2D,
}

fn coord2(p: &Coord3D) -> Coord2D {
    Coord2D { x: p.x, y: p.y }
}

/// `LineInRegion`: whether the segment, clipped as the original clips it, lies
/// in the region.
pub fn line_in_region(p1: &Coord2D, p2: &Coord2D, clip: &Region2D) -> bool {
    const LEFT: u32 = 0x01;
    const RIGHT: u32 = 0x02;
    const BOTTOM: u32 = 0x04;
    const TOP: u32 = 0x08;
    let (left, right, top, bottom) = (clip.lo.x, clip.hi.x, clip.lo.y, clip.hi.y);
    let (mut x1, mut y1, mut x2, mut y2) = (p1.x, p1.y, p2.x, p2.y);

    let code = |x: Real, y: Real| {
        let mut c = 0;
        if x < left {
            c = LEFT;
        } else if x > right {
            c = RIGHT;
        }
        if y < top {
            c |= TOP;
        } else if y > bottom {
            c |= BOTTOM;
        }
        c
    };
    let code1 = code(x1, y1);
    let code2 = code(x2, y2);
    if (code1 | code2) == 0 {
        return true;
    }
    if (code1 & code2) != 0 {
        return false;
    }

    if code1 != 0 {
        if code1 & TOP != 0 {
            let diff = y2 - y1;
            if diff == 0.0 {
                return false;
            }
            x1 += (x2 - x1) * (top - y1) / diff;
            y1 = top;
        } else if code1 & BOTTOM != 0 {
            let diff = y2 - y1;
            if diff == 0.0 {
                return false;
            }
            x1 += (x2 - x1) * (bottom - y1) / diff;
            y1 = bottom;
        }
        if x1 > right {
            let diff = x2 - x1;
            if diff == 0.0 {
                return false;
            }
            y1 += (y2 - y1) * (right - x1) / diff;
            x1 = right;
        } else if x1 < left {
            let diff = x2 - x1;
            if diff == 0.0 {
                return false;
            }
            y1 += (y2 - y1) * (left - x1) / diff;
            x1 = left;
        }
    }

    if code2 != 0 {
        if code2 & TOP != 0 {
            let diff = y2 - y1;
            if diff == 0.0 {
                return false;
            }
            x2 += (x2 - x1) * (top - y2) / diff;
            y2 = top;
        } else if code2 & BOTTOM != 0 {
            let diff = y2 - y1;
            if diff == 0.0 {
                return false;
            }
            x2 += (x2 - x1) * (bottom - y2) / diff;
            y2 = bottom;
        }
        if x2 > right {
            let diff = x2 - x1;
            if diff == 0.0 {
                return false;
            }
            y2 += (y2 - y1) * (right - x2) / diff;
            x2 = right;
        } else if x2 < left {
            let diff = x2 - x1;
            if diff == 0.0 {
                return false;
            }
            y2 += (y2 - y1) * (left - x2) / diff;
            x2 = left;
        }
    }

    x1 >= left
        && x1 <= right
        && y1 >= top
        && y1 <= bottom
        && x2 >= left
        && x2 <= right
        && y2 >= top
        && y2 <= bottom
}

fn sub2(a: &Coord3D, b: &Coord3D) -> Coord2D {
    Coord2D {
        x: a.x - b.x,
        y: a.y - b.y,
    }
}

/// `Vector2::Perp_Dot_Product`.
fn perp(a: &Coord2D, b: &Coord2D) -> Real {
    a.x * -b.y + a.y * b.x
}

fn len2(v: &Coord2D) -> Real {
    v.x * v.x + v.y * v.y
}

/// `Point_In_Triangle_2D` over x and y, including its degenerate branch (which
/// reads the length of the second edge for the third).
pub fn point_in_triangle_2d(p0: &Coord3D, p1: &Coord3D, p2: &Coord3D, test: &Coord3D) -> bool {
    let p0p1 = sub2(p1, p0);
    let p1p2 = sub2(p2, p1);
    let p2p0 = sub2(p0, p2);
    let p0p2 = sub2(p2, p0);
    let area = perp(&p0p1, &p0p2);
    if area != 0.0 {
        let side: Real = if area > 0.0 { 1.0 } else { -1.0 };
        if perp(&p0p1, &sub2(test, p0)) * side < 0.0 {
            return false;
        }
        if perp(&p1p2, &sub2(test, p1)) * side < 0.0 {
            return false;
        }
        return !(perp(&p2p0, &sub2(test, p2)) * side < 0.0);
    }

    let d01 = len2(&p0p1);
    let d12 = len2(&p1p2);
    let d20 = len2(&p1p2);
    let (end, point, max) = if d01 > d12 {
        if d01 > d20 {
            (p0p1, sub2(test, p0), d01)
        } else {
            (p2p0, sub2(test, p2), d20)
        }
    } else if d12 > d20 {
        (p1p2, sub2(test, p1), d12)
    } else {
        (p2p0, sub2(test, p2), d20)
    };
    if max != 0.0 {
        if perp(&end, &point) != 0.0 {
            return false;
        }
        let rest = Coord2D {
            x: point.x - end.x,
            y: point.y - end.y,
        };
        len2(&point) <= max && len2(&rest) <= max
    } else {
        len2(&point) == 0.0
    }
}

fn dot(a: &Coord3D, b: &Coord3D) -> Real {
    a.x * b.x + a.y * b.y + a.z * b.z
}

fn sub3(a: &Coord3D, b: &Coord3D) -> Coord3D {
    Coord3D::new(a.x - b.x, a.y - b.y, a.z - b.z)
}

impl BridgeGeometry {
    /// The shape given by the four corners and the two end centres.
    pub fn from_corners(
        from: Coord3D,
        to: Coord3D,
        from_left: Coord3D,
        from_right: Coord3D,
        to_left: Coord3D,
        to_right: Coord3D,
    ) -> Self {
        let mut bounds = Region2D {
            lo: coord2(&from_left),
            hi: coord2(&from_left),
        };
        bounds.unite_with_point(&coord2(&from_right));
        bounds.unite_with_point(&coord2(&to_left));
        bounds.unite_with_point(&coord2(&to_right));
        Self {
            from,
            to,
            from_left,
            from_right,
            to_left,
            to_right,
            bounds,
        }
    }

    /// The corners of `Bridge(Object*)`: a landmark bridge takes its box.
    pub fn from_box(pos: &Coord3D, angle: Real, half_x: Real, half_y: Real) -> Self {
        let c = gm_cosf(angle);
        let s = gm_sinf(angle);
        let from_left = Coord3D::new(
            pos.x - half_x * c - half_y * s,
            pos.y + half_y * c - half_x * s,
            pos.z,
        );
        let to_left = Coord3D::new(
            pos.x + half_x * c - half_y * s,
            pos.y + half_y * c + half_x * s,
            pos.z,
        );
        let from_right = Coord3D::new(
            pos.x - half_x * c + half_y * s,
            pos.y - half_y * c - half_x * s,
            pos.z,
        );
        let to_right = Coord3D::new(
            pos.x + half_x * c + half_y * s,
            pos.y - half_y * c + half_x * s,
            pos.z,
        );
        let mid = |a: &Coord3D, b: &Coord3D| {
            Coord3D::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0, (a.z + b.z) / 2.0)
        };
        Self::from_corners(
            mid(&from_left, &from_right),
            mid(&to_left, &to_right),
            from_left,
            from_right,
            to_left,
            to_right,
        )
    }

    /// `isPointOnBridge`.
    pub fn is_point_on(&self, p: &Coord3D) -> bool {
        if p.x < self.bounds.lo.x
            || p.x > self.bounds.hi.x
            || p.y < self.bounds.lo.y
            || p.y > self.bounds.hi.y
        {
            return false;
        }
        point_in_triangle_2d(&self.from_left, &self.from_right, &self.to_left, p)
            || point_in_triangle_2d(&self.from_right, &self.to_left, &self.to_right, p)
    }

    /// The direction across the "from" end.
    fn across(&self) -> Coord3D {
        let mut v = sub3(&self.from_right, &self.from_left);
        v.normalize();
        v
    }

    /// `isCellOnEnd`.
    pub fn is_cell_on_end(&self, cell: &Region2D) -> bool {
        let mut end = self.across();
        end.x *= PATHFIND_CELL_SIZE as Real;
        end.y *= PATHFIND_CELL_SIZE as Real;
        let inward = |p: &Coord3D| Coord2D {
            x: p.x + end.x,
            y: p.y + end.y,
        };
        let outward = |p: &Coord3D| Coord2D {
            x: p.x - end.x,
            y: p.y - end.y,
        };
        line_in_region(&inward(&self.from_left), &outward(&self.from_right), cell)
            || line_in_region(&inward(&self.to_left), &outward(&self.to_right), cell)
    }

    /// `isCellOnSide`.
    pub fn is_cell_on_side(&self, cell: &Region2D) -> bool {
        let mut end = self.across();
        end.x *= PATHFIND_CELL_SIZE as Real * 0.51;
        end.y *= PATHFIND_CELL_SIZE as Real * 0.51;
        let mut from_left = Coord2D {
            x: self.from_left.x - end.x,
            y: self.from_left.y - end.y,
        };
        let mut from_right = Coord2D {
            x: self.from_right.x + end.x,
            y: self.from_right.y + end.y,
        };
        let mut to_left = Coord2D {
            x: self.to_left.x - end.x,
            y: self.to_left.y - end.y,
        };
        let mut to_right = Coord2D {
            x: self.to_right.x + end.x,
            y: self.to_right.y + end.y,
        };
        if line_in_region(&from_left, &to_left, cell)
            || line_in_region(&from_right, &to_right, cell)
        {
            return true;
        }
        from_left.x -= end.x;
        from_left.y -= end.y;
        from_right.x += end.x;
        from_right.y += end.y;
        to_left.x -= end.x;
        to_left.y -= end.y;
        to_right.x += end.x;
        to_right.y += end.y;
        line_in_region(&from_left, &to_left, cell) || line_in_region(&from_right, &to_right, cell)
    }

    /// `isCellEntryPoint`.
    pub fn is_cell_entry_point(&self, cell: &Region2D) -> bool {
        let mut end = self.across();
        end.x *= PATHFIND_CELL_SIZE as Real;
        end.y *= PATHFIND_CELL_SIZE as Real;
        let mut along = sub3(&self.to, &self.from);
        along.normalize();
        along.x *= (PATHFIND_CELL_SIZE / 2) as Real;
        along.y *= (PATHFIND_CELL_SIZE / 2) as Real;
        let from_left = Coord2D {
            x: self.from_left.x - along.x + end.x,
            y: self.from_left.y - along.y + end.y,
        };
        let from_right = Coord2D {
            x: self.from_right.x - along.x - end.x,
            y: self.from_right.y - along.y - end.y,
        };
        let to_left = Coord2D {
            x: self.to_left.x + along.x + end.x,
            y: self.to_left.y + along.y + end.y,
        };
        let to_right = Coord2D {
            x: self.to_right.x + along.x - end.x,
            y: self.to_right.y + along.y - end.y,
        };
        line_in_region(&from_left, &from_right, cell) || line_in_region(&to_left, &to_right, cell)
    }

    /// `getBridgeHeight`: the deck plane under or over `p`. A vertical deck,
    /// for which the original reads an unset value, answers 0.
    pub fn height(&self, p: &Coord3D, normal: Option<&mut Coord3D>) -> Real {
        let a = sub3(&self.from_right, &self.from_left);
        let b = sub3(&self.to_left, &self.from_left);
        let mut n = Coord3D::new(
            a.y * b.z - a.z * b.y,
            a.z * b.x - a.x * b.z,
            a.x * b.y - a.y * b.x,
        );
        let d;
        if n.x != 0.0 || n.y != 0.0 || n.z != 0.0 {
            n.normalize_ww_vector3();
            d = dot(&n, &self.from_left);
        } else {
            n = Coord3D::new(0.0, 0.0, 1.0);
            d = 0.0;
        }
        const FACTOR: Real = 1000.0;
        let bottom = Coord3D::new(p.x, p.y, 0.0);
        let top = Coord3D::new(p.x, p.y, FACTOR);
        let den = dot(&n, &sub3(&top, &bottom));
        let t = if den == 0.0 {
            0.0
        } else {
            -(dot(&n, &bottom) - d) / den
        };
        if let Some(out) = normal {
            *out = n;
        }
        t * FACTOR
    }

    /// The square `objectInteractsWithBridgeLayer` tests against the ends.
    fn end_probe(&self, pos: &Coord3D, minor_radius: Real) -> Region2D {
        let radius = minor_radius + PATHFIND_CELL_SIZE_F / 2.0;
        let mut bounds = Region2D {
            lo: coord2(pos),
            hi: coord2(pos),
        };
        bounds.lo.x -= radius;
        bounds.lo.y -= radius;
        bounds.hi.x += radius;
        bounds.hi.y += radius;
        bounds
    }
}

/// `Bridge` with the fields of its `BridgeInfo` that the logic reads.
#[derive(Debug, Clone, PartialEq)]
pub struct Bridge {
    pub geometry: BridgeGeometry,
    pub template_name: String,
    pub object_id: ObjectId,
    pub tower_ids: [ObjectId; 4],
    pub damage_state: BodyDamageType,
    pub damage_state_changed: bool,
    pub layer: PathfindLayerEnum,
}

impl Bridge {
    pub fn new(geometry: BridgeGeometry, template_name: &str, object_id: ObjectId) -> Self {
        Self {
            geometry,
            template_name: template_name.to_string(),
            object_id,
            tower_ids: [ObjectId::INVALID; 4],
            damage_state: BodyDamageType::Pristine,
            damage_state_changed: false,
            layer: PathfindLayerEnum::Ground,
        }
    }

    /// `BridgeInfo::towerObjectID[type]`.
    pub fn set_tower(&mut self, tower_type: usize, id: ObjectId) {
        if let Some(slot) = self.tower_ids.get_mut(tower_type) {
            *slot = id;
        }
    }
}

/// `IntersectLine2D` (`Line2D.cpp`) as a yes or no.
fn segments_meet(a: &Coord2D, b: &Coord2D, c: &Coord2D, d: &Coord2D) -> bool {
    let denom = (b.x - a.x) * (d.y - c.y) - (b.y - a.y) * (d.x - c.x);
    if denom == 0.0 {
        return false;
    }
    let r = ((a.y - c.y) * (d.x - c.x) - (a.x - c.x) * (d.y - c.y)) / denom;
    let s = ((a.y - c.y) * (b.x - a.x) - (a.x - c.x) * (b.y - a.y)) / denom;
    (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&s)
}

/// `PointInsideArea2D`: even-odd crossings of a segment to a point far away.
pub fn point_inside_area_2d(p: &Coord3D, area: &[Coord3D]) -> bool {
    let far = Coord2D {
        x: 1_000_000.0,
        y: 1_000_000.0,
    };
    let from = Coord2D { x: p.x, y: p.y };
    let mut crossings = 0usize;
    for (i, a) in area.iter().enumerate() {
        let Some(b) = area.get((i + 1) % area.len()) else {
            continue;
        };
        let a = Coord2D { x: a.x, y: a.y };
        let b = Coord2D { x: b.x, y: b.y };
        if segments_meet(&from, &far, &a, &b) {
            crossings += 1;
        }
    }
    crossings % 2 == 1
}

/// `getLayerForDestination` over the surfaces at `pos`: the nearest in height.
pub fn nearest_layer(
    pos: &Coord3D,
    ground: Real,
    wall_height: Real,
    on_wall: impl FnOnce() -> bool,
    decks: &[BridgeDeck],
) -> PathfindLayerEnum {
    let mut best = PathfindLayerEnum::Ground;
    let mut distance = (pos.z - ground).abs();
    if distance > wall_height / 2.0 && on_wall() {
        let delta = (pos.z - wall_height).abs();
        if delta < distance {
            best = PathfindLayerEnum::Wall;
            distance = delta;
        }
    }
    for deck in decks {
        let delta = (pos.z - deck.height).abs();
        if delta < distance {
            best = deck.layer;
            distance = delta;
        }
    }
    best
}

/// `getHighestLayerForDestination`: the highest surface not above `pos`.
pub fn highest_layer(
    pos: &Coord3D,
    ground: Real,
    wall_height: Real,
    on_wall: impl FnOnce() -> bool,
    decks: &[BridgeDeck],
    only_healthy_bridges: bool,
) -> PathfindLayerEnum {
    let mut best = PathfindLayerEnum::Ground;
    let mut distance = pos.z - ground;
    if distance > wall_height / 2.0 && on_wall() {
        let delta = pos.z - wall_height;
        if delta >= 0.0 && delta.abs() < distance.abs() {
            best = PathfindLayerEnum::Wall;
            distance = delta;
        }
    }
    for deck in decks {
        if only_healthy_bridges && deck.rubble != 0 {
            continue;
        }
        let delta = pos.z - deck.height;
        if delta >= 0.0 && delta.abs() < distance.abs() {
            best = deck.layer;
            distance = delta;
        }
    }
    best
}

/// An object as the bridge damage pass sees it.
#[derive(Debug, Clone, Copy)]
pub struct DeckObject {
    pub id: ObjectId,
    pub position: Coord3D,
    pub minor_radius: Real,
    pub layer: PathfindLayerEnum,
}

/// What `Bridge::updateDamageState` asks of the game.
pub trait BridgeWorld {
    /// The bridge object's body damage state, `None` if the object is gone.
    fn bridge_damage_state(&self, id: ObjectId) -> Option<BodyDamageType>;
    /// `BridgeBehaviorInterface::isScaffoldPresent`, `FALSE` without one.
    fn scaffold_present(&self, _id: ObjectId) -> bool {
        false
    }
    /// All objects in `getFirstObject` order.
    fn objects(&self) -> Vec<DeckObject> {
        Vec::new()
    }
    /// Falling damage of `HUGE_DAMAGE_AMOUNT`, sourced by the object itself.
    fn kill_by_fall(&mut self, _id: ObjectId) {}
}

/// `TerrainLogic`'s bridge list. Index 0 is the head: the last bridge added.
#[derive(Debug, Clone, Default)]
pub struct BridgeList {
    bridges: Vec<Bridge>,
}

impl BridgeList {
    pub fn clear(&mut self) {
        self.bridges.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.bridges.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Bridge> {
        self.bridges.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Bridge> {
        self.bridges.iter_mut()
    }

    /// Links a bridge at the head.
    pub fn push(&mut self, bridge: Bridge) {
        self.bridges.insert(0, bridge);
    }

    /// `deleteBridge`, by bridge object.
    pub fn remove(&mut self, object_id: ObjectId) {
        if let Some(i) = self.bridges.iter().position(|b| b.object_id == object_id) {
            self.bridges.remove(i);
        }
    }

    /// The decks over `p`, in list order.
    pub fn decks_at(&self, p: &Coord3D) -> Vec<BridgeDeck> {
        self.bridges
            .iter()
            .filter(|b| b.geometry.is_point_on(p))
            .map(|b| BridgeDeck {
                layer: b.layer,
                height: b.geometry.height(p, None),
                rubble: u8::from(b.damage_state == BodyDamageType::Rubble),
            })
            .collect()
    }

    /// `findBridgeAt`.
    pub fn find_at(&self, p: &Coord3D) -> Option<&Bridge> {
        self.bridges.iter().find(|b| b.geometry.is_point_on(p))
    }

    /// `findBridgeLayerAt`.
    pub fn find_layer_at(
        &self,
        p: &Coord3D,
        layer: PathfindLayerEnum,
        clip: bool,
    ) -> Option<&Bridge> {
        if layer == PathfindLayerEnum::Ground {
            return None;
        }
        self.bridges
            .iter()
            .find(|b| b.layer == layer && (!clip || b.geometry.is_point_on(p)))
    }

    /// The bridge half of `objectInteractsWithBridgeLayer`.
    pub fn interacts_with_layer(
        &self,
        pos: &Coord3D,
        minor_radius: Real,
        layer: PathfindLayerEnum,
        consider_health: bool,
    ) -> bool {
        if layer == PathfindLayerEnum::Ground {
            return false;
        }
        let Some(bridge) = self.bridges.iter().find(|b| b.layer == layer) else {
            return false;
        };
        let g = &bridge.geometry;
        let on = g.is_point_on(pos) || g.is_cell_on_end(&g.end_probe(pos, minor_radius));
        if !on {
            return false;
        }
        if (pos.z - g.height(pos, None)).abs() > LAYER_Z_CLOSE_ENOUGH_F {
            return false;
        }
        !(consider_health && bridge.damage_state == BodyDamageType::Rubble)
    }

    /// `objectInteractsWithBridgeEnd`.
    pub fn interacts_with_end(
        &self,
        pos: &Coord3D,
        minor_radius: Real,
        layer: PathfindLayerEnum,
    ) -> bool {
        if layer == PathfindLayerEnum::Ground {
            return false;
        }
        let Some(bridge) = self.bridges.iter().find(|b| b.layer == layer) else {
            return false;
        };
        let g = &bridge.geometry;
        if !g.is_cell_on_end(&g.end_probe(pos, minor_radius)) {
            return false;
        }
        !((pos.z - g.height(pos, None)).abs() > LAYER_Z_CLOSE_ENOUGH_F)
    }

    /// `isBridgeRepaired`.
    pub fn is_repaired(&self, object_id: ObjectId) -> bool {
        self.bridges
            .iter()
            .find(|b| b.object_id == object_id)
            .is_some_and(|b| b.damage_state_changed && b.damage_state != BodyDamageType::Rubble)
    }

    /// `isBridgeBroken`.
    pub fn is_broken(&self, object_id: ObjectId) -> bool {
        self.bridges
            .iter()
            .find(|b| b.object_id == object_id)
            .is_some_and(|b| b.damage_state_changed && b.damage_state == BodyDamageType::Rubble)
    }

    /// `updateBridgeDamageStates` over `Bridge::updateDamageState`.
    /// `change` is `Pathfinder::changeBridgeState(layer, repaired)`.
    pub fn update_damage_states(
        &mut self,
        world: &mut dyn BridgeWorld,
        change: &mut dyn FnMut(PathfindLayerEnum, bool),
    ) {
        for i in 0..self.bridges.len() {
            self.update_damage_state(i, world, change);
        }
    }

    pub fn len(&self) -> usize {
        self.bridges.len()
    }

    /// `Bridge::updateDamageState` of the bridge at `i`.
    pub fn update_damage_state(
        &mut self,
        i: usize,
        world: &mut dyn BridgeWorld,
        change: &mut dyn FnMut(PathfindLayerEnum, bool),
    ) {
        if let Some(bridge) = self.bridges.get_mut(i) {
            bridge.damage_state_changed = false;
            let id = bridge.object_id;
            if id == ObjectId::INVALID {
                return;
            }
            let Some(state) = world.bridge_damage_state(id) else {
                self.bridges[i].object_id = ObjectId::INVALID;
                return;
            };
            let current = self.bridges[i].damage_state;
            if state != current {
                self.bridges[i].damage_state = state;
                if state == BodyDamageType::Rubble {
                    let layer = self.bridges[i].layer;
                    change(layer, false);
                    self.bridges[i].damage_state_changed = true;
                    for obj in world.objects() {
                        // A bridge given the wall's slot takes everything on
                        // that layer.
                        if obj.layer == layer
                            && (layer == PathfindLayerEnum::Wall
                                || self.interacts_with_layer(
                                    &obj.position,
                                    obj.minor_radius,
                                    layer,
                                    false,
                                ))
                        {
                            world.kill_by_fall(obj.id);
                        }
                    }
                }
                if current == BodyDamageType::Rubble {
                    if !world.scaffold_present(id) {
                        change(self.bridges[i].layer, true);
                    }
                    self.bridges[i].damage_state_changed = true;
                }
            }
        }
    }
}
