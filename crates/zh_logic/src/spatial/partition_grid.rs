//! `PartitionManager`, first half: filing objects into the partition cells.
//!
//! Source: `GameLogic/Object/PartitionManager.cpp` and `GameLogic/PartitionManager.h`.
//! Scope note: `docs/PARTITION_SCOPE.md`.
//! Golden test: `crates/zh_logic/tests/golden_partition_grid.rs`
//! (harness `tests/cpp_harness/gen_partition_grid.cpp`).
//!
//! Ported: `init`, `registerObject`, `unRegisterObject`, `makeDirty`, the filing
//! half of `update`, the cell fill routines and the radius-ring table. The
//! searches are in `partition_query`, the shape tests and contact processing in
//! `partition_collide` (`update` gathers the contacts here). The shroud levels are
//! in `partition_shroud`. Not ported: threat and value maps, ghost objects,
//! cached terrain heights.
//!
//! # List order
//!
//! The order of every list here is observable by searches and collision handling.
//! The C++ lists are intrusive and insert at the head. Here each is a `Vec` that
//! inserts with `push`, so **the C++ head is the last element**. The accessors
//! iterate in C++ list order (head first).
//!
//! # Caller contract
//!
//! * The grid never reads the simulation. `update` asks the caller for each dirty
//!   object's current data.
//! * `register` and `unregister` stand for `registerObject` / `unRegisterObject`.
//! * A position or orientation change needs `make_dirty`.
//! * A geometry change must be `unregister` then `register`, as
//!   `Object::setGeometryInfo` does: the number of cells an object may occupy is
//!   fixed per registration, and re-registration moves it to the list heads.
//!   (`setGeometryInfoZ` changes only the height and does neither.)
//!
//! # Retail features
//!
//! Under `retail-crc` the sphere and cylinder cell capacity, the circle fill loop
//! bound and the radius-ring precision follow the retail branches. The golden
//! vectors cover only the default branches.

use std::collections::{BTreeMap, BTreeSet};

use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{
    gm_cosf, gm_sinf, gm_sqrtf, real_to_int_ceil, real_to_int_floor, Coord3D, Region3D,
};
use zh_common::types::{Bool, Int, ObjectId, Real, FALSE};

use crate::spatial::partition_shroud::Shroud;

/// What the filing code and the searches read from an object.
#[derive(Debug, Clone)]
pub struct PartitionObject {
    pub id: ObjectId,
    pub pos: Coord3D,
    /// `Object::getOrientation()`.
    pub angle: Real,
    pub geometry: GeometryInfo,
    /// `KINDOF_IMMOBILE`. Filing ignores it; the collision check does not.
    pub is_immobile: Bool,
    /// `ThingTemplate::friend_getBuildCost()`. Read only by the searches that
    /// sort by cost.
    pub build_cost: Int,
}

/// `PartitionData::DirtyStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DirtyStatus {
    NotDirty,
    NeedCollisionCheck,
    NeedCellUpdateAndCollisionCheck,
}

/// The filing state of one registration (`PartitionData`).
#[derive(Debug, Clone)]
struct Entry {
    dirty: DirtyStatus,
    /// `m_coiArrayCount`. Set at the first filing; geometry cannot change before
    /// then without a re-registration.
    capacity: Option<usize>,
    /// Touched cell indices, in `m_coiArray` order.
    touched: Vec<usize>,
    /// `m_lastCell`: the cell under the object's center at its last filing.
    last_cell: Option<usize>,
}

/// The cell grid of `PartitionManager`.
#[derive(Debug, Clone)]
pub struct PartitionGrid {
    world_extents: Region3D,
    cell_size: Real,
    cell_size_inv: Real,
    cell_count_x: Int,
    cell_count_y: Int,
    /// Index `y * cell_count_x + x`. Each list is stored tail first.
    cells: Vec<Vec<ObjectId>>,
    entries: BTreeMap<ObjectId, Entry>,
    /// `m_moduleList`, stored tail first.
    module_list: Vec<ObjectId>,
    /// `m_dirtyModules`, stored tail first.
    dirty_list: Vec<ObjectId>,
    /// The unprocessed part of `PartitionContactList`, stored tail first.
    contacts: Vec<(ObjectId, ObjectId)>,
    updated_since_last_reset: bool,
    /// `m_radiusVec`: offsets `(x, y)` by minimum ring distance.
    radius_vec: Vec<Vec<(Int, Int)>>,
    pub(crate) shroud: Shroud,
}

impl PartitionGrid {
    /// `PartitionManager::init`, with `m_partitionCellSize` and the terrain extent.
    pub fn new(cell_size: Real, extent: Region3D) -> Self {
        let mut cell_size = cell_size;
        if cell_size < 1.0 {
            cell_size = 1.0;
        }
        // The C++ divides in double and narrows.
        let cell_size_inv = (1.0 / f64::from(cell_size)) as Real;
        let mut world_extents = extent;
        if world_extents.width() < 1.0 {
            world_extents.hi.x = world_extents.lo.x + 1.0;
        }
        if world_extents.height() < 1.0 {
            world_extents.hi.y = world_extents.lo.y + 1.0;
        }
        let cell_count_x = real_to_int_ceil(world_extents.width() * cell_size_inv);
        let cell_count_y = real_to_int_ceil(world_extents.height() * cell_size_inv);
        let total = cell_count_x.wrapping_mul(cell_count_y);
        let mut grid = Self {
            world_extents,
            cell_size,
            cell_size_inv,
            cell_count_x,
            cell_count_y,
            cells: vec![Vec::new(); total as usize],
            entries: BTreeMap::new(),
            module_list: Vec::new(),
            dirty_list: Vec::new(),
            contacts: Vec::new(),
            updated_since_last_reset: false,
            radius_vec: Vec::new(),
            shroud: Shroud::new(cell_count_x, cell_count_y),
        };
        grid.calc_radius_vec();
        grid
    }

    // ------------------------------------------------------------------
    // Registration and dirtiness
    // ------------------------------------------------------------------

    /// `PartitionManager::registerObject`. Does nothing if already registered.
    pub fn register(&mut self, id: ObjectId) {
        if self.entries.contains_key(&id) {
            return;
        }
        self.entries.insert(
            id,
            Entry {
                dirty: DirtyStatus::NotDirty,
                capacity: None,
                touched: Vec::new(),
                last_cell: None,
            },
        );
        self.module_list.push(id);
        self.make_dirty(id);
    }

    /// `PartitionManager::unRegisterObject` for an object without a ghost.
    pub fn unregister(&mut self, id: ObjectId) {
        if !self.entries.contains_key(&id) {
            return;
        }
        self.remove_all_touched_cells(id);
        self.entries.remove(&id);
        self.module_list.retain(|&other| other != id);
        self.dirty_list.retain(|&other| other != id);
        self.contacts.retain(|&(a, b)| a != id && b != id);
    }

    /// `PartitionData::makeDirty(true)`: re-file at the next update. An object
    /// already in the dirty list keeps its place.
    pub fn make_dirty(&mut self, id: ObjectId) {
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        if entry.dirty == DirtyStatus::NotDirty {
            self.dirty_list.push(id);
        }
        entry.dirty = DirtyStatus::NeedCellUpdateAndCollisionCheck;
    }

    /// `PartitionData::makeDirty(false)`: collision check only. The C++ caller is
    /// `PartitionContactList::processContactList`, for colliding mobile pairs.
    pub fn make_dirty_for_collision_check(&mut self, id: ObjectId) {
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        if entry.dirty == DirtyStatus::NotDirty {
            entry.dirty = DirtyStatus::NeedCollisionCheck;
            self.dirty_list.push(id);
        }
    }

    // ------------------------------------------------------------------
    // Update
    // ------------------------------------------------------------------

    /// `PartitionManager::update` up to `processContactList`.
    ///
    /// Drains the dirty list from its head, re-filing the objects that need it and
    /// gathering the possible collisions of each mobile one. Contacts left over
    /// from the previous update are discarded; run
    /// `partition_collide::process_contacts` afterwards.
    /// An object `lookup` does not know keeps the cells it had.
    ///
    /// Returns, in processing order, the objects whose center cell changed: the
    /// ones for which the C++ calls `Object::onPartitionCellChange` (from inside
    /// the loop).
    pub fn update(
        &mut self,
        lookup: &dyn Fn(ObjectId) -> Option<PartitionObject>,
    ) -> Vec<ObjectId> {
        self.updated_since_last_reset = true;
        self.contacts.clear();
        // Unordered pairs already listed.
        let mut listed = BTreeSet::new();
        let mut cell_changes = Vec::new();
        while let Some(id) = self.dirty_list.pop() {
            let Some(entry) = self.entries.get_mut(&id) else {
                continue;
            };
            let update_cells = entry.dirty == DirtyStatus::NeedCellUpdateAndCollisionCheck;
            entry.dirty = DirtyStatus::NotDirty;
            let Some(object) = lookup(id) else {
                continue;
            };
            if update_cells && self.update_cells_touched(id, &object) {
                cell_changes.push(id);
            }
            // This must run between re-filings: it sees the lists as they are now.
            if object.is_immobile == FALSE {
                self.add_possible_collisions(id, &mut listed);
            }
        }
        cell_changes
    }

    /// `PartitionData::addPossibleCollisions`.
    fn add_possible_collisions(
        &mut self,
        id: ObjectId,
        listed: &mut BTreeSet<(ObjectId, ObjectId)>,
    ) {
        let Some(entry) = self.entries.get(&id) else {
            return;
        };
        for &index in &entry.touched {
            let cell = &self.cells[index];
            if cell.len() < 2 {
                continue;
            }
            for &other in cell.iter().rev() {
                if other == id {
                    continue;
                }
                let key = if id < other { (id, other) } else { (other, id) };
                if listed.insert(key) {
                    // Listed oldest first here; the C++ prepends.
                    self.contacts.push((id, other));
                }
            }
        }
    }

    /// Removes and returns the next contact to process: the newest one.
    pub(crate) fn take_contact(&mut self) -> Option<(ObjectId, ObjectId)> {
        self.contacts.pop()
    }

    /// The contacts still to process, in processing order. The first of each pair
    /// is the object whose check found it.
    pub fn pending_contacts(&self) -> impl Iterator<Item = (ObjectId, ObjectId)> + '_ {
        self.contacts.iter().rev().copied()
    }

    /// `PartitionData::updateCellsTouched`. True if the center cell changed.
    fn update_cells_touched(&mut self, id: ObjectId, object: &PartitionObject) -> bool {
        let geometry = &object.geometry;
        let geom = geometry.get_geom_type();
        let major = geometry.get_major_radius();
        let minor = geometry.get_minor_radius();
        let is_small = geometry.get_is_small() != 0;
        let pos = object.pos;

        let capacity = match self.entries.get(&id).and_then(|e| e.capacity) {
            Some(capacity) => capacity,
            None => self.max_cells_for_shape(geom, major, minor),
        };

        self.remove_all_touched_cells(id);
        let mut fill = Fill {
            id,
            capacity,
            touched: Vec::new(),
        };
        if is_small {
            self.do_small_fill(&mut fill, pos.x, pos.y, major);
        } else {
            match geom {
                GeometryType::Sphere | GeometryType::Cylinder => {
                    self.do_circle_fill(&mut fill, pos.x, pos.y, major);
                }
                GeometryType::Box => {
                    self.do_rect_fill(&mut fill, pos.x, pos.y, major, minor, object.angle);
                }
            }
        }

        let (cx, cy) = self.world_to_cell(pos.x, pos.y);
        let current = self.cell_index(cx, cy);
        let Some(entry) = self.entries.get_mut(&id) else {
            return false;
        };
        entry.capacity = Some(capacity);
        entry.touched = fill.touched;
        let changed = current != entry.last_cell;
        entry.last_cell = current;
        changed
    }

    fn remove_all_touched_cells(&mut self, id: ObjectId) {
        let Some(entry) = self.entries.get_mut(&id) else {
            return;
        };
        for index in entry.touched.drain(..) {
            let list = &mut self.cells[index];
            if let Some(at) = list.iter().position(|&other| other == id) {
                list.remove(at);
            }
        }
    }

    /// `calcMaxCoiForShape`.
    fn max_cells_for_shape(&self, geom: GeometryType, major: Real, minor: Real) -> usize {
        let square = |width: Real| {
            let cells = self.world_to_cell_dist(width).wrapping_add(1);
            cells.wrapping_mul(cells)
        };
        let box_cells = || square(gm_sqrtf(major * major + minor * minor) * 2.0);
        let result = match geom {
            // The retail build falls through into the box case.
            GeometryType::Sphere | GeometryType::Cylinder => {
                if cfg!(feature = "retail-crc") {
                    box_cells()
                } else {
                    square(major * 2.0)
                }
            }
            GeometryType::Box => box_cells(),
        };
        result.max(4) as usize
    }

    /// `addSubPixToCoverage`: cells past the capacity are silently dropped.
    fn add_sub_pix(&mut self, fill: &mut Fill, index: usize) {
        // Only this object is being filed and it was removed from every list
        // first, so it covers a cell exactly when it is that list's head.
        if self.cells[index].last() == Some(&fill.id) {
            return;
        }
        if fill.touched.len() < fill.capacity {
            self.cells[index].push(fill.id);
            fill.touched.push(index);
        }
    }

    fn add_cell_at(&mut self, fill: &mut Fill, x: Int, y: Int) {
        if let Some(index) = self.cell_index(x, y) {
            self.add_sub_pix(fill, index);
        }
    }

    /// `doSmallFill`.
    ///
    /// The C++ writes these cells without a capacity check. Rounding can make the
    /// span three cells wide; with more cells than capacity the C++ overruns its
    /// array, which has no behaviour to match. Here every cell is kept.
    fn do_small_fill(&mut self, fill: &mut Fill, center_x: Real, center_y: Real, radius: Real) {
        let half_cell = self.cell_size * 0.5;
        let radius = if radius > half_cell {
            half_cell
        } else {
            radius
        };
        let (x1, y1) = self.world_to_cell(center_x - radius, center_y - radius);
        let (x2, y2) = self.world_to_cell(center_x + radius, center_y + radius);
        for x in x1..=x2 {
            for y in y1..=y2 {
                if let Some(index) = self.cell_index(x, y) {
                    self.cells[index].push(fill.id);
                    fill.touched.push(index);
                }
            }
        }
    }

    fn h_line_circle(&mut self, fill: &mut Fill, x1: Int, x2: Int, y: Int) {
        for x in x1..=x2 {
            self.add_cell_at(fill, x, y);
        }
    }

    /// `doCircleFill` (the retail-compatible circle algorithm, which both builds
    /// use).
    fn do_circle_fill(&mut self, fill: &mut Fill, center_x: Real, center_y: Real, radius: Real) {
        let (cx, cy) = self.world_to_cell(center_x, center_y);
        let cell_radius = self.world_to_cell_dist(radius).max(1);
        let mut y = cell_radius - 1;
        let mut dec = 3i32.wrapping_sub(cell_radius.wrapping_mul(2));
        // Retail bounds small radii by the shrinking `y`, large ones by its start.
        let retail_end = cell_radius - 1;
        let mut x: Int = 0;
        loop {
            let end = if cfg!(feature = "retail-crc") && cell_radius >= 240 {
                retail_end
            } else {
                y
            };
            if x > end {
                break;
            }
            self.h_line_circle(
                fill,
                cx.wrapping_sub(x),
                cx.wrapping_add(x),
                cy.wrapping_add(y),
            );
            self.h_line_circle(
                fill,
                cx.wrapping_sub(x),
                cx.wrapping_add(x),
                cy.wrapping_sub(y),
            );
            self.h_line_circle(
                fill,
                cx.wrapping_sub(y),
                cx.wrapping_add(y),
                cy.wrapping_add(x),
            );
            self.h_line_circle(
                fill,
                cx.wrapping_sub(y),
                cx.wrapping_add(y),
                cy.wrapping_sub(x),
            );
            if dec >= 0 {
                dec = dec.wrapping_add((1 - y).wrapping_shl(2));
                y -= 1;
            }
            dec = dec.wrapping_add((x.wrapping_shl(2)).wrapping_add(6));
            x += 1;
        }
    }

    /// `doRectFill`: samples the rotated box at half-cell steps.
    fn do_rect_fill(
        &mut self,
        fill: &mut Fill,
        center_x: Real,
        center_y: Real,
        half_x: Real,
        half_y: Real,
        angle: Real,
    ) {
        let c = gm_cosf(angle);
        let s = gm_sinf(angle);
        let step = self.cell_size * 0.5;
        let ydx = s * step;
        let ydy = -c * step;
        let xdx = c * step;
        let xdy = s * step;
        let step_inv_times_2 = 2.0 * (1.0 / step);
        let steps_x = real_to_int_ceil(half_x * step_inv_times_2);
        let steps_y = real_to_int_ceil(half_y * step_inv_times_2);
        let mut tl_x = center_x - half_x * c - half_y * s;
        let mut tl_y = center_y + half_y * c - half_x * s;
        for _ in 0..steps_y {
            let mut x = tl_x;
            let mut y = tl_y;
            for _ in 0..steps_x {
                let (cell_x, cell_y) = self.world_to_cell(x, y);
                self.add_cell_at(fill, cell_x, cell_y);
                x += xdx;
                y += xdy;
            }
            tl_x += ydx;
            tl_y += ydy;
        }
    }

    // ------------------------------------------------------------------
    // Radius-ring table
    // ------------------------------------------------------------------

    /// `calcMinRadius`: the ring distance between the closest corners of the
    /// center cell and the cell at offset `(x, y)`.
    fn calc_min_radius(&self, x: Int, y: Int) -> Int {
        let half = (f64::from(self.cell_size) * 0.5) as Real;
        let center = [(-half, -half), (half, -half), (-half, half), (half, half)];
        let x = x as Real * self.cell_size;
        let y = y as Real * self.cell_size;
        let other = [
            (x - half, y - half),
            (x + half, y - half),
            (x - half, y + half),
            (x + half, y + half),
        ];
        #[cfg(feature = "retail-crc")]
        {
            let mut min_sqr: f64 = 1e12;
            for a in center {
                for b in other {
                    let dx = f64::from(a.0 - b.0);
                    let dy = f64::from(a.1 - b.1);
                    let sqr = dx * dx + dy * dy;
                    if min_sqr > sqr {
                        min_sqr = sqr;
                    }
                }
            }
            let dist = f64::from(gm_sqrtf(min_sqr as Real));
            real_to_int_ceil((dist / f64::from(self.cell_size)) as Real)
        }
        #[cfg(not(feature = "retail-crc"))]
        {
            let mut min_sqr: Real = 1e12;
            for a in center {
                for b in other {
                    let dx = a.0 - b.0;
                    let dy = a.1 - b.1;
                    let sqr = dx * dx + dy * dy;
                    if min_sqr > sqr {
                        min_sqr = sqr;
                    }
                }
            }
            real_to_int_ceil(gm_sqrtf(min_sqr) / self.cell_size)
        }
    }

    /// `calcRadiusVec`.
    fn calc_radius_vec(&mut self) {
        let cx = self.cell_count_x;
        let cy = self.cell_count_y;
        #[cfg(feature = "retail-crc")]
        let max_radius = {
            let dx = f64::from(cx) * f64::from(self.cell_size);
            let dy = f64::from(cy) * f64::from(self.cell_size);
            let dist = zh_common::math::gm_sqrt(dx * dx + dy * dy);
            real_to_int_ceil((dist / f64::from(self.cell_size)) as Real)
        };
        #[cfg(not(feature = "retail-crc"))]
        let max_radius = {
            let dx = cx as Real * self.cell_size;
            let dy = cy as Real * self.cell_size;
            real_to_int_ceil(gm_sqrtf(dx * dx + dy * dy) / self.cell_size)
        };
        let mut rings = vec![Vec::new(); (max_radius + 1) as usize];
        for y in (1 - cy)..cy {
            for x in (1 - cx)..cx {
                let radius = self.calc_min_radius(x, y);
                if radius <= max_radius {
                    rings[radius as usize].push((x, y));
                }
            }
        }
        self.radius_vec = rings;
    }

    // ------------------------------------------------------------------
    // Accessors
    // ------------------------------------------------------------------

    /// The effective cell size (at least 1).
    pub fn cell_size(&self) -> Real {
        self.cell_size
    }

    /// The effective extent (at least 1 wide and high).
    pub fn world_extents(&self) -> &Region3D {
        &self.world_extents
    }

    pub fn cell_count_x(&self) -> Int {
        self.cell_count_x
    }

    pub fn cell_count_y(&self) -> Int {
        self.cell_count_y
    }

    /// `worldToCell`. The result may lie outside the grid.
    pub fn world_to_cell(&self, wx: Real, wy: Real) -> (Int, Int) {
        (
            real_to_int_floor((wx - self.world_extents.lo.x) * self.cell_size_inv),
            real_to_int_floor((wy - self.world_extents.lo.y) * self.cell_size_inv),
        )
    }

    /// `worldToCellDist`.
    pub fn world_to_cell_dist(&self, w: Real) -> Int {
        real_to_int_ceil(w * self.cell_size_inv)
    }

    /// `getCellAt` as an index `y * cell_count_x + x`; `None` off the grid.
    pub fn cell_index(&self, x: Int, y: Int) -> Option<usize> {
        if x < 0 || y < 0 || x >= self.cell_count_x || y >= self.cell_count_y {
            None
        } else {
            Some((y * self.cell_count_x + x) as usize)
        }
    }

    /// The objects of a cell in list order (head first).
    pub fn cell_objects(&self, index: usize) -> impl Iterator<Item = ObjectId> + '_ {
        self.cells
            .get(index)
            .into_iter()
            .flat_map(|list| list.iter().rev().copied())
    }

    /// `getCoiCount`.
    pub fn cell_object_count(&self, index: usize) -> usize {
        self.cells.get(index).map_or(0, Vec::len)
    }

    /// The cell indices an object occupies, in the order it touched them.
    pub fn object_cells(&self, id: ObjectId) -> &[usize] {
        self.entries.get(&id).map_or(&[], |e| e.touched.as_slice())
    }

    /// `m_coiArrayCount`: the most cells this registration may occupy. `None`
    /// until its first filing.
    pub fn object_cell_capacity(&self, id: ObjectId) -> Option<usize> {
        self.entries.get(&id).and_then(|e| e.capacity)
    }

    /// `m_lastCell`: the center cell at the last filing.
    pub fn object_center_cell(&self, id: ObjectId) -> Option<usize> {
        self.entries.get(&id).and_then(|e| e.last_cell)
    }

    pub fn is_registered(&self, id: ObjectId) -> bool {
        self.entries.contains_key(&id)
    }

    /// All registered objects in `m_moduleList` order (latest registration first).
    pub fn registered_objects(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.module_list.iter().rev().copied()
    }

    /// The dirty list in the order `update` will process it.
    pub fn dirty_objects(&self) -> impl Iterator<Item = ObjectId> + '_ {
        self.dirty_list.iter().rev().copied()
    }

    /// `getUpdatedSinceLastReset`.
    pub fn updated_since_last_reset(&self) -> bool {
        self.updated_since_last_reset
    }

    /// A load makes the shroud queries unsettled until the next update.
    pub(crate) fn clear_updated_since_last_reset(&mut self) {
        self.updated_since_last_reset = false;
    }

    /// `m_maxGcoRadius`.
    pub fn max_gco_radius(&self) -> Int {
        self.radius_vec.len() as Int - 1
    }

    /// The cell offsets `(x, y)` of one ring of `m_radiusVec`, in table order.
    pub fn radius_offsets(&self, radius: Int) -> &[(Int, Int)] {
        usize::try_from(radius)
            .ok()
            .and_then(|r| self.radius_vec.get(r))
            .map_or(&[], Vec::as_slice)
    }
}

/// The state of one object's fill.
struct Fill {
    id: ObjectId,
    capacity: usize,
    touched: Vec<usize>,
}
