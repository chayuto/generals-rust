//! `PartitionManager`, shroud half: the per-player shroud levels of every cell.
//!
//! Source: `GameLogic/Object/PartitionManager.cpp` and `Common/DiscreteCircle.cpp`.
//! Scope note: `docs/PARTITION_SCOPE.md` (stage 4a).
//! Golden test: `crates/zh_logic/tests/golden_partition_shroud.rs`
//! (harness `tests/cpp_harness/gen_partition_shroud.cpp`).
//!
//! The grid knows players only as indices. The reveal and cover calls take a
//! `player_count` (`ThePlayerList->getPlayerCount()`) and a mask with bit `i` for
//! player `i`. Who is in a mask is the caller's business (`Object::look` and
//! friends, not ported).
//!
//! Not ported: the display and radar updates, and invalidating the per-object
//! shroud status when a cell changes (stage 4c).

use std::collections::VecDeque;

use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, Real, UnsignedInt, UnsignedShort, XferVersion, FALSE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

use crate::combat::damage::PlayerMaskType;
use crate::economy::MAX_PLAYER_COUNT;
use crate::spatial::partition_grid::PartitionGrid;

/// `ShroudLevel`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShroudLevel {
    /// 1 is shrouded, 0 fogged, a negative value is minus the number of lookers.
    pub current_shroud: i16,
    /// The number of shrouders (jammers).
    pub active_shroud_level: i16,
}

/// `CellShroudStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellShroudStatus {
    Clear,
    Fogged,
    Shrouded,
}

/// The shroud state of one `PartitionCell`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShroudCell {
    levels: [ShroudLevel; MAX_PLAYER_COUNT],
    x: i16,
    y: i16,
}

impl ShroudCell {
    fn new(x: Int, y: Int) -> Self {
        // Passive shroud.
        let level = ShroudLevel {
            current_shroud: 1,
            active_shroud_level: 0,
        };
        Self {
            levels: [level; MAX_PLAYER_COUNT],
            x: x as i16,
            y: y as i16,
        }
    }

    pub fn level(&self, player: usize) -> ShroudLevel {
        self.levels[player]
    }

    /// `getShroudStatusForPlayer`.
    pub fn status(&self, player: usize) -> CellShroudStatus {
        match self.levels[player].current_shroud {
            1 => CellShroudStatus::Shrouded,
            0 => CellShroudStatus::Fogged,
            _ => CellShroudStatus::Clear,
        }
    }

    fn add_looker(&mut self, player: usize) {
        let level = &mut self.levels[player];
        level.current_shroud = (Int::from(level.current_shroud) - 1).min(-1) as i16;
    }

    fn remove_looker(&mut self, player: usize) {
        let level = &mut self.levels[player];
        if level.current_shroud == -1 {
            level.current_shroud = level.active_shroud_level.min(1);
        } else {
            level.current_shroud = level.current_shroud.wrapping_add(1);
        }
    }

    fn add_shrouder(&mut self, player: usize) {
        let level = &mut self.levels[player];
        level.active_shroud_level = level.active_shroud_level.wrapping_add(1);
        if level.current_shroud == 0 {
            level.current_shroud = 1;
        }
    }

    fn remove_shrouder(&mut self, player: usize) {
        let level = &mut self.levels[player];
        level.active_shroud_level = level.active_shroud_level.wrapping_sub(1);
    }

    /// `m_shroudLevel` as the C++ lays it out in memory.
    fn level_bytes(&self) -> [u8; 4 * MAX_PLAYER_COUNT] {
        let mut bytes = [0; 4 * MAX_PLAYER_COUNT];
        for (level, out) in self.levels.iter().zip(bytes.chunks_exact_mut(4)) {
            out[..2].copy_from_slice(&level.current_shroud.to_le_bytes());
            out[2..].copy_from_slice(&level.active_shroud_level.to_le_bytes());
        }
        bytes
    }

    fn set_level_bytes(&mut self, bytes: &[u8; 4 * MAX_PLAYER_COUNT]) {
        for (level, from) in self.levels.iter_mut().zip(bytes.chunks_exact(4)) {
            level.current_shroud = i16::from_le_bytes([from[0], from[1]]);
            level.active_shroud_level = i16::from_le_bytes([from[2], from[3]]);
        }
    }
}

impl Snapshot for ShroudCell {
    fn crc(&self, xfer: &mut dyn Xfer) {
        // Three separate calls: the CRC pads each one to whole words.
        let _ = xfer.xfer_user(&mut self.level_bytes());
        let _ = xfer.xfer_user(&mut self.x.to_le_bytes());
        let _ = xfer.xfer_user(&mut self.y.to_le_bytes());
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut bytes = self.level_bytes();
        xfer.xfer_user(&mut bytes)?;
        self.set_level_bytes(&bytes);
        Ok(())
    }
}

/// `SightingInfo`, as queued for a delayed unlook.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SightingInfo {
    pub center: Coord3D,
    pub how_far: Real,
    pub for_whom: PlayerMaskType,
    /// The unlook runs once the frame is past this.
    pub data: UnsignedInt,
}

impl SightingInfo {
    /// `SightingInfo::reset`.
    #[inline]
    pub fn reset(&mut self) {
        self.center = Coord3D::default();
        self.how_far = 0.0;
        self.for_whom = 0;
        self.data = 0;
    }

    /// `SightingInfo::isInvalid`.
    #[inline]
    pub fn is_invalid(&self) -> bool {
        self.how_far == 0.0
    }
}

impl Snapshot for SightingInfo {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_coord3d(&mut self.center)?;
        xfer.xfer_real(&mut self.how_far)?;
        let mut mask: [u8; 2] = UnsignedShort::to_le_bytes(self.for_whom);
        xfer.xfer_user(&mut mask)?;
        self.for_whom = UnsignedShort::from_le_bytes(mask);
        xfer.xfer_unsigned_int(&mut self.data)
    }
}

/// The shroud state held by [`PartitionGrid`].
#[derive(Debug, Clone)]
pub(crate) struct Shroud {
    /// Index `y * cell_count_x + x`.
    cells: Vec<ShroudCell>,
    /// `m_pendingUndoShroudReveals`, oldest first.
    pending_unlooks: VecDeque<SightingInfo>,
}

impl Shroud {
    pub(crate) fn new(cell_count_x: Int, cell_count_y: Int) -> Self {
        let total = cell_count_x.wrapping_mul(cell_count_y) as usize;
        let width = cell_count_x.max(1);
        Self {
            cells: (0..total as Int)
                .map(|i| ShroudCell::new(i % width, i / width))
                .collect(),
            pending_unlooks: VecDeque::new(),
        }
    }
}

/// What a cell is remembered as across a map boundary change.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Stored {
    DontTouch,
    Fog,
    PermanentlyRevealed,
}

/// `ShroudStatusStoreRestore`.
#[derive(Debug, Clone, Default)]
pub struct ShroudStatusStoreRestore {
    cells_wide: Int,
    /// One list per player.
    cells: Vec<Vec<Stored>>,
}

/// The rows of `DiscreteCircle` as `(x_start, x_end, y)`, one per `y`, covering
/// the half at and above the center.
fn circle_edges(x_center: Int, y_center: Int, radius: Int) -> Vec<(Int, Int, Int)> {
    let mut edges: Vec<(Int, Int, Int)> = Vec::new();
    let mut x: Int = 0;
    let mut y = radius;
    let mut d = 1i32.wrapping_sub(radius).wrapping_shl(1);
    while y >= 0 {
        let edge = (
            x_center.wrapping_sub(x),
            x_center.wrapping_add(x),
            y_center.wrapping_add(y),
        );
        // Of the edges sharing a row, the last one stays.
        match edges.last_mut() {
            Some(last) if last.2 == edge.2 => *last = edge,
            _ => edges.push(edge),
        }
        if d.wrapping_add(y) > 0 {
            y -= 1;
            d = d.wrapping_sub(y.wrapping_shl(1).wrapping_sub(1));
        }
        if x > d {
            x += 1;
            d = d.wrapping_add(x.wrapping_shl(1).wrapping_add(1));
        }
    }
    edges
}

impl PartitionGrid {
    // ------------------------------------------------------------------
    // Reveal and cover
    // ------------------------------------------------------------------

    /// `doShroudReveal`: adds a looker for each player in the mask.
    pub fn do_shroud_reveal(
        &mut self,
        center_x: Real,
        center_y: Real,
        radius: Real,
        mask: PlayerMaskType,
        player_count: usize,
    ) {
        self.draw_shroud_circle(
            center_x,
            center_y,
            radius,
            mask,
            player_count,
            ShroudCell::add_looker,
        );
    }

    /// `undoShroudReveal`.
    pub fn undo_shroud_reveal(
        &mut self,
        center_x: Real,
        center_y: Real,
        radius: Real,
        mask: PlayerMaskType,
        player_count: usize,
    ) {
        self.draw_shroud_circle(
            center_x,
            center_y,
            radius,
            mask,
            player_count,
            ShroudCell::remove_looker,
        );
    }

    /// `doShroudCover`: adds a shrouder for each player in the mask.
    pub fn do_shroud_cover(
        &mut self,
        center_x: Real,
        center_y: Real,
        radius: Real,
        mask: PlayerMaskType,
        player_count: usize,
    ) {
        self.draw_shroud_circle(
            center_x,
            center_y,
            radius,
            mask,
            player_count,
            ShroudCell::add_shrouder,
        );
    }

    /// `undoShroudCover`.
    pub fn undo_shroud_cover(
        &mut self,
        center_x: Real,
        center_y: Real,
        radius: Real,
        mask: PlayerMaskType,
        player_count: usize,
    ) {
        self.draw_shroud_circle(
            center_x,
            center_y,
            radius,
            mask,
            player_count,
            ShroudCell::remove_shrouder,
        );
    }

    /// Applies `op` to the circle's cells, highest player index first.
    fn draw_shroud_circle(
        &mut self,
        center_x: Real,
        center_y: Real,
        radius: Real,
        mask: PlayerMaskType,
        player_count: usize,
        op: fn(&mut ShroudCell, usize),
    ) {
        let (cell_x, cell_y) = self.world_to_cell(center_x, center_y);
        let cell_radius = self.world_to_cell_dist(radius).max(1);
        let edges = circle_edges(cell_x, cell_y, cell_radius);
        for player in (0..player_count.min(MAX_PLAYER_COUNT)).rev() {
            if mask & (1 << player) == 0 {
                continue;
            }
            for &(x1, x2, y) in &edges {
                self.shroud_line(x1, x2, y, player, op);
                if y != cell_y {
                    let mirrored = cell_y.wrapping_shl(1).wrapping_sub(y);
                    self.shroud_line(x1, x2, mirrored, player, op);
                }
            }
        }
    }

    fn shroud_line(
        &mut self,
        x1: Int,
        x2: Int,
        y: Int,
        player: usize,
        op: fn(&mut ShroudCell, usize),
    ) {
        if y < 0 || y >= self.cell_count_y() {
            return;
        }
        for x in x1.max(0)..=x2.min(self.cell_count_x() - 1) {
            let index = (y * self.cell_count_x() + x) as usize;
            op(&mut self.shroud.cells[index], player);
        }
    }

    // ------------------------------------------------------------------
    // Delayed unlooks
    // ------------------------------------------------------------------

    /// `queueUndoShroudReveal`. `persist` is `m_unlookPersistDuration`.
    pub fn queue_undo_shroud_reveal(
        &mut self,
        center_x: Real,
        center_y: Real,
        radius: Real,
        mask: PlayerMaskType,
        frame: UnsignedInt,
        persist: UnsignedInt,
    ) {
        self.shroud.pending_unlooks.push_back(SightingInfo {
            center: Coord3D {
                x: center_x,
                y: center_y,
                z: 0.0,
            },
            how_far: radius,
            for_whom: mask,
            data: frame.wrapping_add(persist),
        });
    }

    /// `processPendingUndoShroudRevealQueue`, which `PartitionManager::update`
    /// runs after the contacts. Stops at the first entry that is not yet due.
    pub fn process_pending_undo_shroud_reveals(&mut self, frame: UnsignedInt, player_count: usize) {
        self.process_unlooks_before(frame, player_count);
    }

    /// `processEntirePendingUndoShroudRevealQueue`.
    pub fn process_all_pending_undo_shroud_reveals(&mut self, player_count: usize) {
        self.process_unlooks_before(UnsignedInt::MAX, player_count);
    }

    fn process_unlooks_before(&mut self, frame: UnsignedInt, player_count: usize) {
        while self
            .shroud
            .pending_unlooks
            .front()
            .is_some_and(|info| info.data < frame)
        {
            if let Some(info) = self.shroud.pending_unlooks.pop_front() {
                self.undo_shroud_reveal(
                    info.center.x,
                    info.center.y,
                    info.how_far,
                    info.for_whom,
                    player_count,
                );
            }
        }
    }

    /// The queued unlooks, oldest first.
    pub fn pending_unlooks(&self) -> impl Iterator<Item = &SightingInfo> {
        self.shroud.pending_unlooks.iter()
    }

    // ------------------------------------------------------------------
    // Whole map
    // ------------------------------------------------------------------

    /// `revealMapForPlayer`: clears the passive shroud.
    pub fn reveal_map_for_player(&mut self, player: usize) {
        for cell in &mut self.shroud.cells {
            cell.add_looker(player);
            cell.remove_looker(player);
        }
    }

    /// `revealMapForPlayerPermanently`.
    pub fn reveal_map_for_player_permanently(&mut self, player: usize) {
        for cell in &mut self.shroud.cells {
            cell.add_looker(player);
        }
    }

    /// `undoRevealMapForPlayerPermanently`. Runs the whole queue first.
    pub fn undo_reveal_map_for_player_permanently(&mut self, player: usize, player_count: usize) {
        self.process_all_pending_undo_shroud_reveals(player_count);
        for cell in &mut self.shroud.cells {
            cell.remove_looker(player);
        }
    }

    /// `shroudMapForPlayer`. Runs the whole queue first.
    pub fn shroud_map_for_player(&mut self, player: usize, player_count: usize) {
        self.process_all_pending_undo_shroud_reveals(player_count);
        for cell in &mut self.shroud.cells {
            cell.add_shrouder(player);
            cell.remove_shrouder(player);
        }
    }

    // ------------------------------------------------------------------
    // Queries
    // ------------------------------------------------------------------

    /// The shroud state of the cell at index `y * cell_count_x + x`.
    pub fn shroud_cell(&self, index: usize) -> Option<&ShroudCell> {
        self.shroud.cells.get(index)
    }

    /// `getShroudStatusForPlayer` by cell coordinates. Off the map, or for a
    /// negative player, everything is shrouded. `player` must be below 16.
    pub fn shroud_status(&self, player: Int, x: Int, y: Int) -> CellShroudStatus {
        if player < 0 {
            return CellShroudStatus::Shrouded;
        }
        self.cell_index(x, y)
            .map_or(CellShroudStatus::Shrouded, |index| {
                self.shroud.cells[index].status(player as usize)
            })
    }

    /// `getShroudStatusForPlayer` by world position.
    pub fn shroud_status_at(&self, player: Int, pos: &Coord3D) -> CellShroudStatus {
        let (x, y) = self.world_to_cell(pos.x, pos.y);
        self.shroud_status(player, x, y)
    }

    // ------------------------------------------------------------------
    // Map boundary changes
    // ------------------------------------------------------------------

    /// `storeFoggedCells`. The first pass (`store_to_fog`) resets the store and
    /// notes the fogged cells; the second notes the cells still clear.
    pub fn store_fogged_cells(&self, store: &mut ShroudStatusStoreRestore, store_to_fog: Bool) {
        let total = self.shroud.cells.len();
        if store_to_fog != FALSE {
            store.cells_wide = self.cell_count_x();
            store.cells = vec![vec![Stored::DontTouch; total]; MAX_PLAYER_COUNT];
        }
        for (player, list) in store.cells.iter_mut().enumerate() {
            if list.len() != total {
                continue;
            }
            for (out, cell) in list.iter_mut().zip(&self.shroud.cells) {
                match cell.status(player) {
                    CellShroudStatus::Fogged if store_to_fog != FALSE => *out = Stored::Fog,
                    CellShroudStatus::Clear if store_to_fog == FALSE => {
                        *out = Stored::PermanentlyRevealed;
                    }
                    _ => {}
                }
            }
        }
    }

    /// `restoreFoggedCells`, onto a grid that may have another size.
    pub fn restore_fogged_cells(
        &mut self,
        store: &ShroudStatusStoreRestore,
        restore_to_fog: Bool,
    ) {
        let width = store.cells_wide;
        let Some(first) = store.cells.first() else {
            return;
        };
        if width <= 0 {
            return;
        }
        let height = first.len() as Int / width;
        for (player, list) in store.cells.iter().enumerate() {
            for j in 0..height {
                if j >= self.cell_count_y() {
                    // The original returns here, so on a shorter map only the
                    // first player is restored.
                    return;
                }
                for i in 0..width.min(self.cell_count_x()) {
                    let index = (j * self.cell_count_x() + i) as usize;
                    let cell = &mut self.shroud.cells[index];
                    match list[(j * width + i) as usize] {
                        Stored::Fog if restore_to_fog != FALSE => {
                            cell.add_looker(player);
                            cell.remove_looker(player);
                        }
                        Stored::PermanentlyRevealed if restore_to_fog == FALSE => {
                            cell.add_looker(player);
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// `PartitionManager::crc` and `xfer`. Only the shroud is saved; the object
/// filing is rebuilt from the objects.
impl Snapshot for PartitionGrid {
    fn crc(&self, xfer: &mut dyn Xfer) {
        for cell in &self.shroud.cells {
            cell.crc(xfer);
        }
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 2;
        xfer.xfer_version(&mut version, 2)?;

        let mut cell_size = self.cell_size();
        xfer.xfer_real(&mut cell_size)?;
        if cell_size != self.cell_size() {
            return Err(XferError::InvalidData);
        }

        let mut total = self.shroud.cells.len() as Int;
        xfer.xfer_int(&mut total)?;
        if total != self.shroud.cells.len() as Int {
            return Err(XferError::InvalidData);
        }
        for cell in &mut self.shroud.cells {
            xfer.xfer_snapshot(cell)?;
        }

        if xfer.mode() == XferMode::Load {
            self.clear_updated_since_last_reset();
        }

        if version >= 2 {
            let mut size = self.shroud.pending_unlooks.len() as Int;
            xfer.xfer_int(&mut size)?;
            if xfer.mode() == XferMode::Load {
                // Appended: a load can already have queued unlooks.
                for _ in 0..size {
                    let mut info = SightingInfo::default();
                    xfer.xfer_snapshot(&mut info)?;
                    self.shroud.pending_unlooks.push_back(info);
                }
            } else {
                for info in &mut self.shroud.pending_unlooks {
                    xfer.xfer_snapshot(info)?;
                }
            }
        }
        Ok(())
    }
}
