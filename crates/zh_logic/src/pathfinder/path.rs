//! `Path` (`PKT-412`): a list of waypoints with an optimized shortcut chain,
//! the point-on-path query and the snapshot.
//!
//! `Path::crc` is empty in the reference, so a path is not part of the game CRC.
//! `Path::xfer` writes an uninitialized word before the blocked flag; this port
//! writes zero there.
//!
//! Golden test: `crates/zh_logic/tests/golden_s3_path.rs`.

use zh_common::math::{gm_fabsf, gm_sqrtf, Coord2D, Coord3D};
use zh_common::types::{Bool, Int, Real, UnsignedInt, XferVersion};
use zh_common::xfer::{Xfer, XferError, XferMode};

use crate::object::thing::ThingTerrainAccess;
use crate::pathfinder::ai_pathfind::{PathAgent, PathUnits, Pathfinder};
use crate::pathfinder::path_node::{PathNode, PathNodeIdx};
use crate::pathfinder::pathfind_cell::CellType;
use crate::pathfinder::pathfind_constants::{PathfindLayerEnum, PATHFIND_CELL_SIZE_F};
use crate::rules::combat_types::LocomotorSurfaceTypeMask;

/// `PATHFIND_CLOSE_ENOUGH`.
pub const PATHFIND_CLOSE_ENOUGH: Real = 1.0;
const MAX_CPOP: Int = 20;
const CELL_SIZE: Int = 10;

/// `ClosestPointOnPathInfo`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClosestPointOnPathInfo {
    pub dist_along_path: Real,
    pub pos_on_path: Coord3D,
    pub layer: PathfindLayerEnum,
}

impl Default for ClosestPointOnPathInfo {
    fn default() -> Self {
        Self {
            dist_along_path: 0.0,
            pos_on_path: Coord3D::default(),
            layer: PathfindLayerEnum::Ground,
        }
    }
}

/// What a path asks the pathfinder while it optimizes and follows itself.
pub struct PathQuery<'a> {
    pub pathfinder: &'a Pathfinder,
    pub units: &'a dyn PathUnits,
    /// The moving object; the reference allows none while optimizing.
    pub agent: Option<&'a PathAgent>,
}

impl PathQuery<'_> {
    fn line_passable(
        &self,
        surfaces: LocomotorSurfaceTypeMask,
        layer: PathfindLayerEnum,
        from: &Coord3D,
        to: &Coord3D,
        blocked: bool,
        allow_pinched: bool,
    ) -> bool {
        self.pathfinder.is_line_passable(
            self.units,
            self.agent,
            surfaces,
            layer,
            from,
            to,
            blocked,
            allow_pinched,
        )
    }
}

#[derive(Debug, Clone)]
pub struct Path {
    nodes: Vec<PathNode>,
    head: Option<PathNodeIdx>,
    tail: Option<PathNodeIdx>,
    is_optimized: bool,
    blocked_by_ally: Bool,
    cpop_valid: bool,
    cpop_countdown: Int,
    cpop_in: Coord3D,
    cpop_out: ClosestPointOnPathInfo,
    cpop_recent_start: Option<PathNodeIdx>,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            nodes: Vec::new(),
            head: None,
            tail: None,
            is_optimized: false,
            blocked_by_ally: 0,
            cpop_valid: false,
            cpop_countdown: MAX_CPOP,
            cpop_in: Coord3D::default(),
            cpop_out: ClosestPointOnPathInfo::default(),
            cpop_recent_start: None,
        }
    }
}

/// The cheap closeness test of the point-on-path cache.
fn is_really_close(a: &Coord3D, b: &Coord3D) -> bool {
    const CLOSE_ENOUGH: Real = 0.1;
    gm_fabsf(a.x - b.x) <= CLOSE_ENOUGH
        && gm_fabsf(a.y - b.y) <= CLOSE_ENOUGH
        && gm_fabsf(a.z - b.z) <= CLOSE_ENOUGH
}

impl Path {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn first_node(&self) -> Option<PathNodeIdx> {
        self.head
    }

    pub fn last_node(&self) -> Option<PathNodeIdx> {
        self.tail
    }

    pub fn node(&self, idx: PathNodeIdx) -> &PathNode {
        &self.nodes[idx]
    }

    pub fn set_can_optimize(&mut self, idx: PathNodeIdx, can_optimize: bool) {
        self.nodes[idx].can_optimize = can_optimize;
    }

    /// `PathNode::setLayer`.
    pub fn set_layer(&mut self, idx: PathNodeIdx, layer: PathfindLayerEnum) {
        self.nodes[idx].layer = layer;
    }

    pub fn set_position(&mut self, idx: PathNodeIdx, pos: &Coord3D) {
        self.nodes[idx].pos = *pos;
    }

    pub fn is_optimized(&self) -> bool {
        self.is_optimized
    }

    pub fn mark_optimized(&mut self) {
        self.is_optimized = true;
    }

    pub fn set_blocked_by_ally(&mut self, blocked: Bool) {
        self.blocked_by_ally = blocked;
    }

    pub fn blocked_by_ally(&self) -> Bool {
        self.blocked_by_ally
    }

    /// `peekCachedPointOnPath`.
    pub fn peek_cached_point_on_path(&self) -> Coord3D {
        self.cpop_out.pos_on_path
    }

    /// `PathNode::setNextOptimized`.
    pub fn set_next_optimized(&mut self, idx: PathNodeIdx, next: Option<PathNodeIdx>) {
        self.nodes[idx].next_opti = next;
        let Some(next) = next else {
            self.nodes[idx].next_opti_dist = 0.0;
            return;
        };
        let to = self.nodes[next].pos;
        let node = &mut self.nodes[idx];
        let mut dir = Coord2D::new(to.x - node.pos.x, to.y - node.pos.y);
        let mut dist = dir.length();
        if dist == 0.0 {
            dist = 0.01;
        }
        dir.x /= dist;
        dir.y /= dist;
        node.next_opti_dist = dist;
        node.next_opti_dir = dir;
    }

    /// `PathNode::computeDirectionVector`.
    pub fn direction_vector(&self, idx: PathNodeIdx) -> Coord3D {
        match (self.nodes[idx].next, self.nodes[idx].prev) {
            (Some(next), _) => {
                let (a, b) = (self.nodes[idx].pos, self.nodes[next].pos);
                Coord3D::new(b.x - a.x, b.y - a.y, b.z - a.z)
            }
            (None, Some(prev)) => self.direction_vector(prev),
            (None, None) => Coord3D::default(),
        }
    }

    fn push(&mut self, pos: &Coord3D, layer: PathfindLayerEnum) -> PathNodeIdx {
        self.nodes.push(PathNode {
            pos: *pos,
            layer,
            ..PathNode::default()
        });
        self.nodes.len() - 1
    }

    fn link_front(&mut self, node: PathNodeIdx) {
        self.nodes[node].next = self.head;
        if let Some(head) = self.head {
            self.nodes[head].prev = Some(node);
        }
        self.nodes[node].prev = None;
        self.head = Some(node);
        if self.tail.is_none() {
            self.tail = Some(node);
        }
    }

    /// `PathNode::append`: a new node after `after`. The tail is not moved.
    pub fn insert_after(
        &mut self,
        after: PathNodeIdx,
        pos: &Coord3D,
        layer: PathfindLayerEnum,
    ) -> PathNodeIdx {
        let node = self.push(pos, layer);
        let next = self.nodes[after].next;
        self.nodes[node].next = next;
        self.nodes[node].prev = Some(after);
        if let Some(next) = next {
            self.nodes[next].prev = Some(node);
        }
        self.nodes[after].next = Some(node);
        node
    }

    pub fn prepend_node(&mut self, pos: &Coord3D, layer: PathfindLayerEnum) {
        let node = self.push(pos, layer);
        self.link_front(node);
        self.is_optimized = false;
    }

    pub fn append_node(&mut self, pos: &Coord3D, layer: PathfindLayerEnum) {
        if self.is_optimized {
            if let Some(tail) = self.tail {
                let last = self.nodes[tail].pos;
                if pos.x == last.x && pos.y == last.y {
                    return;
                }
            }
        }
        let (Some(_), Some(tail)) = (self.head, self.tail) else {
            let node = self.push(pos, layer);
            self.head = Some(node);
            self.tail = Some(node);
            return;
        };
        let node = self.insert_after(tail, pos, layer);
        if self.is_optimized {
            self.set_next_optimized(tail, Some(node));
        }
        self.tail = Some(node);
    }

    pub fn update_last_node(&mut self, terrain: &dyn ThingTerrainAccess, pos: &Coord3D) {
        let layer = terrain.get_layer_for_destination(pos);
        let Some(tail) = self.tail else { return };
        self.nodes[tail].pos = *pos;
        self.nodes[tail].layer = layer;
        if self.is_optimized {
            let mut node = self.head;
            while let Some(n) = node {
                if self.nodes[n].next_opti == Some(tail) {
                    self.set_next_optimized(n, Some(tail));
                    break;
                }
                node = self.nodes[n].next_opti;
            }
        }
    }

    /// The farthest node of the run starting after `anchor` that one shortcut
    /// may reach.
    fn run_end(
        &self,
        anchor: PathNodeIdx,
        layer: &mut PathfindLayerEnum,
        strict: bool,
    ) -> PathNodeIdx {
        const ALLOWED_STEPS: Int = 3;
        let mut cur_layer = self.nodes[anchor].layer;
        let mut count = 0;
        let mut node = self.nodes[anchor]
            .next
            .expect("anchor before the last node");
        while let Some(next) = self.nodes[node].next {
            count += 1;
            if cur_layer == PathfindLayerEnum::Ground {
                if self.nodes[node].layer != cur_layer {
                    *layer = self.nodes[node].layer;
                    if count > ALLOWED_STEPS {
                        break;
                    }
                }
            } else if self.nodes[next].layer != cur_layer && count > ALLOWED_STEPS {
                break;
            }
            cur_layer = self.nodes[node].layer;
            if strict && !self.nodes[node].can_optimize {
                break;
            }
            node = next;
        }
        node
    }

    /// Whether the step into `p` from its successor keeps `keep` true for every
    /// node between `node` and `anchor`.
    fn all_steps(
        &self,
        anchor: PathNodeIdx,
        node: PathNodeIdx,
        mut keep: impl FnMut(Coord3D, Coord3D) -> bool,
    ) -> bool {
        let mut result = true;
        let mut tmp = self.nodes[node].prev;
        while let Some(t) = tmp {
            if t == anchor {
                break;
            }
            let next = self.nodes[t].next.expect("walked back from a later node");
            if !keep(self.nodes[next].pos, self.nodes[t].pos) {
                result = false;
            }
            tmp = self.nodes[t].prev;
        }
        result
    }

    /// The straight and diagonal runs the reference accepts without a clear
    /// line. `dx` and `dy` are integers it keeps reusing, so each later test
    /// sees the values the earlier walks left behind.
    fn might_be_passable(&self, anchor: PathNodeIdx, node: PathNodeIdx) -> (Int, Int, bool) {
        let (a, n) = (self.nodes[anchor].pos, self.nodes[node].pos);
        let first_dx = (n.x - a.x) as Int;
        let first_dy = (n.y - a.y) as Int;
        let (mut dx, mut dy) = (first_dx, first_dy);
        let mut might = false;
        if dx == 0 {
            might = true;
            might &= self.all_steps(anchor, node, |next, cur| {
                dx = (next.x - cur.x) as Int;
                dx == 0
            });
        }
        if dy == 0 {
            might = true;
            might &= self.all_steps(anchor, node, |next, cur| {
                dy = (next.y - cur.y) as Int;
                dy == 0
            });
        }
        if dx == dy {
            might = true;
            might &= self.all_steps(anchor, node, |next, cur| {
                dx = (next.x - cur.x) as Int;
                dy = (next.y - cur.y) as Int;
                dy == dx
            });
        }
        if dx == dy.wrapping_neg() {
            might = true;
            might &= self.all_steps(anchor, node, |next, cur| {
                dx = (next.x - cur.x) as Int;
                dy = (next.y - cur.y) as Int;
                dy == dx.wrapping_neg()
            });
        }
        (first_dx, first_dy, might)
    }

    /// `optimize`. The path needs at least one node.
    pub fn optimize(
        &mut self,
        query: &PathQuery,
        surfaces: LocomotorSurfaceTypeMask,
        blocked: bool,
    ) {
        let (Some(mut anchor), Some(last)) = (self.head, self.tail) else {
            return;
        };
        let mut first_node = true;
        let first_layer = self.nodes[anchor].layer;
        while anchor != last {
            let mut layer = self.nodes[anchor].layer;
            let mut node = self.run_end(anchor, &mut layer, true);
            if first_node {
                layer = first_layer;
                first_node = false;
            }
            let mut optimized = false;
            while node != anchor {
                let (from, to) = (self.nodes[anchor].pos, self.nodes[node].pos);
                let mut passable = query.line_passable(surfaces, layer, &from, &to, blocked, false);
                if let Some(cell) = query.pathfinder.get_cell_at(layer, &to) {
                    let cell = query.pathfinder.map().grid().cell(cell);
                    if cell.cell_type == CellType::Cliff && !cell.pinched {
                        passable = true;
                    }
                }
                if !passable {
                    let (dx, dy, might) = self.might_be_passable(anchor, node);
                    let one_diagonal =
                        dx.wrapping_abs() == CELL_SIZE && dy.wrapping_abs() == CELL_SIZE;
                    passable = one_diagonal || might;
                }
                if passable {
                    self.set_next_optimized(anchor, Some(node));
                    anchor = node;
                    optimized = true;
                    break;
                }
                node = self.nodes[node].prev.expect("anchor precedes node");
            }
            if !optimized {
                let next = self.nodes[anchor]
                    .next
                    .expect("anchor before the last node");
                self.set_next_optimized(anchor, Some(next));
                anchor = next;
            }
        }
        self.is_optimized = true;
    }

    /// `optimizeGroundPath`.
    pub fn optimize_ground_path(
        &mut self,
        pathfinder: &Pathfinder,
        units: &dyn PathUnits,
        crusher: bool,
        diameter: Int,
    ) {
        let (Some(mut anchor), Some(last)) = (self.head, self.tail) else {
            return;
        };
        while anchor != last {
            let mut layer = self.nodes[anchor].layer;
            let mut node = self.run_end(anchor, &mut layer, false);
            let mut optimized = false;
            while node != anchor {
                let (from, to) = (self.nodes[anchor].pos, self.nodes[node].pos);
                let passable = pathfinder
                    .is_ground_path_passable(units, crusher, &from, layer, &to, diameter)
                    || self.might_be_passable(anchor, node).2;
                if passable {
                    self.set_next_optimized(anchor, Some(node));
                    anchor = node;
                    optimized = true;
                    break;
                }
                node = self.nodes[node].prev.expect("anchor precedes node");
            }
            if !optimized {
                let next = self.nodes[anchor]
                    .next
                    .expect("anchor before the last node");
                self.set_next_optimized(anchor, Some(next));
                anchor = next;
            }
        }
        // Remove jig/jogs.
        let mut cursor = self.head;
        while let Some(anchor) = cursor {
            if let Some(node) = self.nodes[anchor].next_opti {
                if let Some(after) = self.nodes[node].next_opti {
                    let dx = self.nodes[node].pos.x - self.nodes[anchor].pos.x;
                    let dy = self.nodes[node].pos.y - self.nodes[anchor].pos.y;
                    if dx * dx + dy * dy < (PATHFIND_CELL_SIZE_F * PATHFIND_CELL_SIZE_F) * 3.9 {
                        self.set_next_optimized(anchor, Some(after));
                    }
                }
            }
            cursor = self.nodes[anchor].next_opti;
        }
        self.is_optimized = true;
    }

    /// `computePointOnPath`. `surfaces` is the locomotor set's valid surfaces;
    /// `query.agent` is the moving object.
    pub fn compute_point_on_path(
        &mut self,
        query: &PathQuery,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
    ) -> ClosestPointOnPathInfo {
        let mut out = ClosestPointOnPathInfo::default();
        let (Some(head), Some(tail)) = (self.head, self.tail) else {
            self.cpop_valid = false;
            return out;
        };
        out.layer = self.nodes[head].layer;
        if self.cpop_valid && self.cpop_countdown > 0 && is_really_close(pos, &self.cpop_in) {
            self.cpop_countdown -= 1;
            return self.cpop_out;
        }
        self.cpop_countdown = MAX_CPOP;
        out.pos_on_path = self.nodes[tail].pos;

        let mut close_node = None;
        let mut close_dist_sqr: Real = 99_999_999.9;
        let mut total_length: Real = 0.0;
        let mut length_to_pos: Real = 0.0;
        let mut prev = head;
        let mut cursor = self.nodes[prev].next_opti;
        while let Some(node) = cursor {
            let (dir, segment) = self.nodes[prev].optimized_segment();
            let (a, b) = (self.nodes[prev].pos, self.nodes[node].pos);
            let to_pos = Coord2D::new(pos.x - a.x, pos.y - a.y);
            let mut along = dir.x * to_pos.x + dir.y * to_pos.y;
            let point;
            if along < 0.0 {
                along = 0.0;
                point = a;
            } else if along > segment {
                if self.nodes[node].next_opti.is_none() {
                    along = segment;
                    point = b;
                } else {
                    total_length += segment;
                    prev = node;
                    cursor = self.nodes[node].next_opti;
                    continue;
                }
            } else {
                point = Coord3D::new(a.x + along * dir.x, a.y + along * dir.y, 0.0);
            }
            let offset = Coord2D::new(pos.x - point.x, pos.y - point.y);
            let offset_sqr = offset.x * offset.x + offset.y * offset.y;
            if offset_sqr < close_dist_sqr {
                close_dist_sqr = offset_sqr;
                close_node = Some(prev);
                out.pos_on_path = point;
                length_to_pos = total_length + along;
            }
            total_length += segment;
            prev = node;
            cursor = self.nodes[node].next_opti;
        }

        if let Some(close) = close_node {
            if let Some(next) = self.nodes[close].next_opti {
                self.look_ahead(query, surfaces, pos, close, next, &mut out);
            }
        }

        out.dist_along_path = total_length - length_to_pos;
        let delta = Coord3D::new(out.pos_on_path.x - pos.x, out.pos_on_path.y - pos.y, 0.0);
        let length = delta.length();
        if length > out.dist_along_path && out.dist_along_path > PATHFIND_CLOSE_ENOUGH {
            out.dist_along_path = length;
        }
        self.cpop_in = *pos;
        self.cpop_out = out;
        self.cpop_valid = true;
        out
    }

    /// The second half of `computePointOnPath`: aim farther along the closest
    /// segment when the way there is clear.
    fn look_ahead(
        &self,
        query: &PathQuery,
        surfaces: LocomotorSurfaceTypeMask,
        pos: &Coord3D,
        close: PathNodeIdx,
        next: PathNodeIdx,
        out: &mut ClosestPointOnPathInfo,
    ) {
        let (dir, segment) = self.nodes[close].optimized_segment();
        let close_node = &self.nodes[close];
        let next_node = &self.nodes[next];
        let next_pos = next_node.pos;
        let close_pos = close_node.pos;
        let ground = PathfindLayerEnum::Ground as Int;

        if close_node
            .prev
            .is_some_and(|p| self.nodes[p].layer as Int > ground)
        {
            out.layer = close_node.layer;
        }
        if close_node.layer as Int > ground {
            out.layer = close_node.layer;
        }
        if next_node.layer as Int > ground {
            out.layer = next_node.layer;
        }

        let to_pos = Coord2D::new(pos.x - close_pos.x, pos.y - close_pos.y);
        let mut along = dir.x * to_pos.x + dir.y * to_pos.y;
        if along < 0.0 {
            along = 0.0;
        }
        let to_dist_sqr = to_pos.x * to_pos.x + to_pos.y * to_pos.y;
        let offset_sq = to_dist_sqr - along * along;
        let offset_dist = if offset_sq <= 0.0 {
            0.0
        } else {
            gm_sqrtf(offset_sq)
        };
        let max_error: Real = 3.0 * PATHFIND_CELL_SIZE_F;
        let max_error_inv: Real = 1.0 / max_error;
        let mut k = offset_dist * max_error_inv;
        if k > 1.0 {
            k = 1.0;
        }

        let mut got_pos = false;
        if query.line_passable(surfaces, out.layer, pos, &next_pos, false, true) {
            out.pos_on_path = next_pos;
            got_pos = true;
            let mut try_ahead = f64::from(along) > f64::from(segment) * 0.5;
            if !next_node.can_optimize {
                try_ahead = false;
            }
            if close_node.layer != next_node.layer {
                try_ahead = false;
            }
            if query
                .agent
                .is_some_and(|a| a.layer != PathfindLayerEnum::Ground)
            {
                try_ahead = false;
            }
            let mut very_close = false;
            if segment - along < 1.0 {
                try_ahead = true;
                very_close = true;
            }
            if try_ahead {
                if let Some(after) = next_node.next_opti {
                    let after_pos = self.nodes[after].pos;
                    let ahead = Coord3D::new(
                        (f64::from(next_pos.x + after_pos.x) * 0.5) as Real,
                        (f64::from(next_pos.y + after_pos.y) * 0.5) as Real,
                        next_pos.z,
                    );
                    if very_close
                        || query.line_passable(surfaces, next_node.layer, pos, &ahead, false, true)
                    {
                        got_pos = true;
                        out.pos_on_path = ahead;
                    }
                }
            }
        } else if k > 0.5 {
            let try_dist = (f64::from(along) + 0.5 * f64::from(segment - along)) as Real;
            out.pos_on_path = Coord3D::new(
                close_pos.x + try_dist * dir.x,
                close_pos.y + try_dist * dir.y,
                close_pos.z,
            );
            if query.line_passable(surfaces, out.layer, pos, &out.pos_on_path, false, true) {
                k = 0.5;
                got_pos = true;
            }
        }

        along += (1.0 - k) * (segment - along);

        if !got_pos {
            if along > segment {
                out.pos_on_path = next_pos;
            } else {
                out.pos_on_path = Coord3D::new(
                    close_pos.x + along * dir.x,
                    close_pos.y + along * dir.y,
                    close_pos.z,
                );
                let dx = gm_fabsf(pos.x - out.pos_on_path.x);
                let dy = gm_fabsf(pos.y - out.pos_on_path.y);
                if dx < 1.0 && dy < 1.0 {
                    if let Some(after) = next_node.next_opti {
                        out.pos_on_path = self.nodes[after].pos;
                    }
                }
            }
        }
    }

    /// `computeFlightDistToGoal`: the distance and the goal to fly at.
    pub fn compute_flight_dist_to_goal(&mut self, pos: &Coord3D) -> (Real, Coord3D) {
        let Some(head) = self.head else {
            return (0.0, Coord3D::default());
        };
        let mut cur = match self.cpop_recent_start {
            Some(start) => start,
            None => {
                self.cpop_recent_start = Some(head);
                head
            }
        };
        let mut next = self.nodes[cur].next_opti;
        let mut goal = self.nodes[cur].pos;
        let mut distance: Real = 0.0;
        let mut use_next = true;
        while let Some(n) = next {
            if use_next {
                goal = self.nodes[n].pos;
            }
            let (start, end) = (self.nodes[cur].pos, self.nodes[n].pos);
            let to_goal = Coord2D::new(end.x - pos.x, end.y - pos.y);
            let mut along = Coord2D::new(end.x - start.x, end.y - start.y);
            along.normalize();
            let dot = to_goal.x * along.x + to_goal.y * along.y;
            if dot >= 0.0 {
                distance += dot;
                use_next = false;
            } else if use_next {
                self.cpop_recent_start = Some(n);
            }
            cur = n;
            next = self.nodes[cur].next_opti;
        }
        (distance, goal)
    }

    /// `Path::xfer`, save and load.
    pub fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        const VERSION: XferVersion = 1;
        let mut version = VERSION;
        xfer.xfer_version(&mut version, VERSION)?;

        let mut count: Int = 0;
        let mut node = self.head;
        while let Some(n) = node {
            count += 1;
            node = self.nodes[n].next;
        }
        xfer.xfer_int(&mut count)?;

        if xfer.mode() == XferMode::Save {
            let mut node = self.tail;
            while let Some(n) = node {
                self.nodes[n].id = count;
                xfer.xfer_int(&mut count)?;
                let mut pos = self.nodes[n].pos;
                xfer.xfer_coord3d(&mut pos)?;
                let mut layer = (self.nodes[n].layer as Int).to_le_bytes();
                xfer.xfer_user(&mut layer)?;
                let mut can_optimize = Bool::from(self.nodes[n].can_optimize);
                xfer.xfer_bool(&mut can_optimize)?;
                let mut opti_id: Int = self.nodes[n].next_opti.map_or(-1, |o| self.nodes[o].id);
                xfer.xfer_int(&mut opti_id)?;
                count -= 1;
                node = self.nodes[n].prev;
            }
        } else {
            self.cpop_valid = false;
            while count != 0 {
                let mut id: Int = 0;
                xfer.xfer_int(&mut id)?;
                let mut pos = Coord3D::default();
                xfer.xfer_coord3d(&mut pos)?;
                let mut layer = [0u8; 4];
                xfer.xfer_user(&mut layer)?;
                let layer = PathfindLayerEnum::from_i32(Int::from_le_bytes(layer))
                    .ok_or(XferError::InvalidData)?;
                let mut can_optimize: Bool = 0;
                xfer.xfer_bool(&mut can_optimize)?;
                let mut opti_id: Int = -1;
                xfer.xfer_int(&mut opti_id)?;
                let node = self.push(&pos, layer);
                self.nodes[node].id = id;
                self.nodes[node].can_optimize = can_optimize != 0;
                let mut optimized = None;
                if opti_id > 0 {
                    optimized = self.head;
                    while let Some(o) = optimized {
                        if self.nodes[o].id == opti_id {
                            break;
                        }
                        optimized = self.nodes[o].next;
                    }
                }
                self.link_front(node);
                if optimized.is_some() {
                    self.set_next_optimized(node, optimized);
                }
                count -= 1;
            }
        }

        let mut is_optimized = Bool::from(self.is_optimized);
        xfer.xfer_bool(&mut is_optimized)?;
        self.is_optimized = is_optimized != 0;
        let mut obsolete: Int = 0;
        xfer.xfer_int(&mut obsolete)?;
        let mut uninitialized: UnsignedInt = 0;
        xfer.xfer_unsigned_int(&mut uninitialized)?;
        xfer.xfer_bool(&mut self.blocked_by_ally)?;
        Ok(())
    }
}
