//! `PKT-150`: Cave system tunnel index manager (`CaveSystem.h` / `CaveSystem.cpp`).

use zh_common::types::{Bool, Int, UnsignedInt, UnsignedShort, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

/// Interface implemented by per-cave `TunnelTracker` instances managed by [`CaveSystem`].
pub trait CaveTunnelTracker: Snapshot {
    fn contain_count(&self) -> UnsignedInt;
}

/// Deterministic standalone state matching the C++ oracle `TunnelTracker` contract.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CaveTunnelState {
    pub contain_count: Int,
    pub tag: UnsignedInt,
}

impl CaveTunnelTracker for CaveTunnelState {
    #[inline]
    fn contain_count(&self) -> UnsignedInt {
        if self.contain_count > 0 {
            self.contain_count as UnsignedInt
        } else {
            0
        }
    }
}

impl Snapshot for CaveTunnelState {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1u8;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_int(&mut self.contain_count)?;
        xfer.xfer_unsigned_int(&mut self.tag)?;
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

/// Sparse index-stable registry of tunnel trackers (`CaveSystem`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaveSystem<T: CaveTunnelTracker + Default = CaveTunnelState> {
    tunnel_trackers: Vec<Option<T>>,
}

impl<T: CaveTunnelTracker + Default> Default for CaveSystem<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: CaveTunnelTracker + Default> CaveSystem<T> {
    pub fn new() -> Self {
        Self {
            tunnel_trackers: Vec::new(),
        }
    }

    pub fn init(&mut self) {}

    pub fn reset(&mut self) {
        self.tunnel_trackers.clear();
    }

    pub fn update(&mut self) {}

    #[inline]
    pub fn len(&self) -> usize {
        self.tunnel_trackers.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.tunnel_trackers.is_empty()
    }

    /// Returns `TRUE` iff switching from `old_index` to `new_index` is safe (both are empty or unallocated).
    /// Matches C++ unsigned `size_t > Int` check when indices are negative.
    pub fn can_switch_index_to_index(&self, old_index: Int, new_index: Int) -> Bool {
        if let Ok(oi) = usize::try_from(old_index) {
            if let Some(Some(old_tracker)) = self.tunnel_trackers.get(oi) {
                if old_tracker.contain_count() > 0 {
                    return FALSE;
                }
            }
        }
        if let Ok(ni) = usize::try_from(new_index) {
            if let Some(Some(new_tracker)) = self.tunnel_trackers.get(ni) {
                if new_tracker.contain_count() > 0 {
                    return FALSE;
                }
            }
        }
        TRUE
    }

    /// Allocates slot `index` (padding intermediate gaps with `None`) if missing.
    pub fn register_new_cave(&mut self, index: Int) {
        let Ok(idx) = usize::try_from(index) else {
            return;
        };
        while self.tunnel_trackers.len() <= idx {
            self.tunnel_trackers.push(None);
        }
        if self.tunnel_trackers[idx].is_none() {
            self.tunnel_trackers[idx] = Some(T::default());
        }
    }

    /// No-op matching `CaveSystem::unregisterCave`.
    pub fn unregister_cave(&mut self, _index: Int) {}

    pub fn get_tunnel_tracker_for_cave_index(&self, index: Int) -> Option<&T> {
        let idx = usize::try_from(index).ok()?;
        self.tunnel_trackers.get(idx).and_then(|slot| slot.as_ref())
    }

    pub fn get_tunnel_tracker_for_cave_index_mut(&mut self, index: Int) -> Option<&mut T> {
        let idx = usize::try_from(index).ok()?;
        self.tunnel_trackers.get_mut(idx).and_then(|slot| slot.as_mut())
    }
}

impl<T: CaveTunnelTracker + Default> Snapshot for CaveSystem<T> {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1u8;
        xfer.xfer_version(&mut version, 1)?;

        let mut count: UnsignedShort = self.tunnel_trackers.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;

        if xfer.mode() == XferMode::Save {
            for slot in &mut self.tunnel_trackers {
                match slot {
                    Some(tracker) => xfer.xfer_snapshot(tracker)?,
                    None => return Err(XferError::InvalidData),
                }
            }
        } else {
            if !self.tunnel_trackers.is_empty() {
                return Err(XferError::InvalidData);
            }
            for _ in 0..count {
                let mut tracker = T::default();
                xfer.xfer_snapshot(&mut tracker)?;
                self.tunnel_trackers.push(Some(tracker));
            }
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}
