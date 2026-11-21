//! Exact deterministic port of Westwood/SAGE `GameLogic::m_sleepyUpdates` array min-heap
//! (`GeneralsMD/Code/GameEngine/Source/GameLogic/System/GameLogic.cpp` lines 3073–3359 & 4062–4113)
//! and `UpdateModule` scheduling metadata (`UpdateModule.h` / `UpdateModule.cpp`).

use zh_common::types::{Int, UnsignedInt, UpdateSleepTime};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const UPDATE_SLEEP_INVALID: UnsignedInt = 0;
pub const UPDATE_SLEEP_NONE: UnsignedInt = 1;
pub const UPDATE_SLEEP_FOREVER: UnsignedInt = 0x3fff_ffff;
pub const INVALID_LOGIC_INDEX: Int = -1;

/// Execution phase within a frame packed into the bottom 2 bits of `next_call_frame_and_phase`.
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum SleepyUpdatePhase {
    Initial = 0,
    Physics = 1,
    #[default]
    Normal = 2,
    Final = 3,
}

impl SleepyUpdatePhase {
    #[inline]
    pub const fn from_bits(bits: u32) -> Self {
        match bits & 3 {
            0 => Self::Initial,
            1 => Self::Physics,
            2 => Self::Normal,
            _ => Self::Final,
        }
    }
}

/// Per-module sleepy-scheduler state matching C++ `UpdateModule` fields
/// `m_nextCallFrameAndPhase` and `m_indexInLogic`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UpdateModuleScheduleState {
    next_call_frame_and_phase: UnsignedInt,
    index_in_logic: Int,
    phase: SleepyUpdatePhase,
}

impl Default for UpdateModuleScheduleState {
    fn default() -> Self {
        Self::new(SleepyUpdatePhase::Normal)
    }
}

impl UpdateModuleScheduleState {
    #[inline]
    pub const fn new(phase: SleepyUpdatePhase) -> Self {
        Self {
            next_call_frame_and_phase: 0,
            index_in_logic: INVALID_LOGIC_INDEX,
            phase,
        }
    }

    #[inline]
    pub const fn priority(&self) -> UnsignedInt {
        self.next_call_frame_and_phase
    }

    #[inline]
    pub const fn next_call_frame(&self) -> UnsignedInt {
        self.next_call_frame_and_phase >> 2
    }

    #[inline]
    pub const fn next_call_phase(&self) -> SleepyUpdatePhase {
        SleepyUpdatePhase::from_bits(self.next_call_frame_and_phase)
    }

    #[inline]
    pub const fn update_phase(&self) -> SleepyUpdatePhase {
        self.phase
    }

    #[inline]
    pub fn set_update_phase(&mut self, phase: SleepyUpdatePhase) {
        self.phase = phase;
    }

    #[inline]
    pub const fn index_in_logic(&self) -> Int {
        self.index_in_logic
    }

    #[inline]
    pub fn set_index_in_logic(&mut self, index: Int) {
        self.index_in_logic = index;
    }

    /// Sets the target wake frame, clamping `> UPDATE_SLEEP_FOREVER` and packing the phase bits.
    #[inline]
    pub fn set_next_call_frame(&mut self, mut frame: UnsignedInt) {
        if frame > UPDATE_SLEEP_FOREVER {
            frame = UPDATE_SLEEP_FOREVER;
        }
        self.next_call_frame_and_phase = (frame << 2) | (self.phase as u32);
    }

    /// Sets the raw packed priority (for tests/state restoration; use `set_next_call_frame` normally).
    #[inline]
    pub fn set_raw_priority(&mut self, raw: UnsignedInt) {
        self.next_call_frame_and_phase = raw;
    }

    /// Computes relative sleep delay from the scheduled wake frame and current frame (`UpdateModule::getWakeFrame`).
    #[inline]
    pub const fn get_wake_frame(&self, now: UnsignedInt) -> UpdateSleepTime {
        let next_frame = self.next_call_frame();
        if next_frame > now {
            UpdateSleepTime(next_frame - now)
        } else {
            UpdateSleepTime::NONE
        }
    }

    /// Computes minimum sleep delay across up to four target frames (`UpdateModule::frameToSleepTime`).
    #[inline]
    pub const fn frame_to_sleep_time(
        now: UnsignedInt,
        frame1: UnsignedInt,
        frame2: UnsignedInt,
        frame3: UnsignedInt,
        frame4: UnsignedInt,
    ) -> UpdateSleepTime {
        let mut min_f = frame1;
        if min_f > frame2 {
            min_f = frame2;
        }
        if min_f > frame3 {
            min_f = frame3;
        }
        if min_f > frame4 {
            min_f = frame4;
        }
        if min_f > now {
            UpdateSleepTime(min_f - now)
        } else {
            UpdateSleepTime::NONE
        }
    }

    /// Serializes `m_nextCallFrameAndPhase` and re-normalizes phase/index on load (`UpdateModule::xfer`).
    pub fn xfer_schedule(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        xfer.xfer_unsigned_int(&mut self.next_call_frame_and_phase)?;
        if xfer.mode() == XferMode::Load {
            self.next_call_frame_and_phase =
                (self.next_call_frame_and_phase & !3) | (self.phase as u32);
            self.index_in_logic = INVALID_LOGIC_INDEX;
        }
        Ok(())
    }
}

/// Evaluates whether an update module runs given the owner's active `DisabledMask` bits and the
/// module's `getDisabledTypesToProcess()` mask bits (`GameLogic.cpp` lines 4083–4091).
#[inline]
pub const fn should_process_disabled_update(
    disabled_mask_bits: u64,
    process_mask_bits: u64,
) -> bool {
    if cfg!(feature = "retail-crc") {
        disabled_mask_bits == 0 || (disabled_mask_bits & process_mask_bits) != 0
    } else {
        (disabled_mask_bits & !process_mask_bits) == 0
    }
}

/// Backing store providing schedule state access for handles stored in [`SleepyUpdateHeap`].
pub trait ScheduleStore<Handle> {
    fn schedule(&self, handle: Handle) -> &UpdateModuleScheduleState;
    fn schedule_mut(&mut self, handle: Handle) -> &mut UpdateModuleScheduleState;
}

#[inline]
const fn is_lower_priority(a_pri: UnsignedInt, b_pri: UnsignedInt) -> bool {
    a_pri > b_pri
}

/// Deterministic array-backed min-heap with O(1) index tracking matching `GameLogic::m_sleepyUpdates`.
///
/// Strict `>` comparisons (`is_lower_priority`) guarantee exact tie-breaking parity with C++
/// `GameLogic.cpp` (`std::collections::BinaryHeap` must never be used here).
#[derive(Debug, Clone)]
pub struct SleepyUpdateHeap<Handle: Copy + Eq> {
    entries: Vec<Handle>,
    cur_update_module: Option<Handle>,
}

impl<Handle: Copy + Eq> Default for SleepyUpdateHeap<Handle> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Handle: Copy + Eq> SleepyUpdateHeap<Handle> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            cur_update_module: None,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[inline]
    pub fn as_slice(&self) -> &[Handle] {
        &self.entries
    }

    #[inline]
    pub fn cur_update_module(&self) -> Option<Handle> {
        self.cur_update_module
    }

    #[inline]
    pub fn set_cur_update_module(&mut self, current: Option<Handle>) {
        self.cur_update_module = current;
    }

    pub fn clear<S: ScheduleStore<Handle>>(&mut self, store: &mut S) {
        for &h in &self.entries {
            store.schedule_mut(h).set_index_in_logic(INVALID_LOGIC_INDEX);
        }
        self.entries.clear();
        self.cur_update_module = None;
    }

    #[inline]
    pub fn peek(&self) -> Option<Handle> {
        self.entries.first().copied()
    }

    /// Returns the root handle if it is due on or before `now`.
    #[inline]
    pub fn peek_due<S: ScheduleStore<Handle>>(&self, store: &S, now: UnsignedInt) -> Option<Handle> {
        let root = self.peek()?;
        if store.schedule(root).next_call_frame() > now {
            None
        } else {
            Some(root)
        }
    }

    /// Pushes `handle` onto the heap and sifts up to its parent (`GameLogic::pushSleepyUpdate`).
    pub fn push<S: ScheduleStore<Handle>>(&mut self, store: &mut S, handle: Handle) {
        let idx = self.entries.len();
        self.entries.push(handle);
        store.schedule_mut(handle).set_index_in_logic(idx as Int);
        self.rebalance_parent(store, idx);
    }

    /// Appends `handle` for bulk load reconstruction prior to [`Self::remake`].
    pub fn push_raw_for_load<S: ScheduleStore<Handle>>(&mut self, store: &mut S, handle: Handle) {
        let idx = self.entries.len();
        self.entries.push(handle);
        store.schedule_mut(handle).set_index_in_logic(idx as Int);
    }

    /// Removes and returns the root handle (`GameLogic::popSleepyUpdate`).
    pub fn pop<S: ScheduleStore<Handle>>(&mut self, store: &mut S) -> Option<Handle> {
        let sz = self.entries.len();
        if sz == 0 {
            return None;
        }
        let root = self.entries[0];
        store.schedule_mut(root).set_index_in_logic(INVALID_LOGIC_INDEX);
        if sz > 1 {
            let last = self.entries[sz - 1];
            self.entries[0] = last;
            store.schedule_mut(last).set_index_in_logic(0);
            self.entries.pop();
            self.rebalance_child(store, 0);
        } else {
            self.entries.pop();
        }
        Some(root)
    }

    /// Erases the entry at `idx` (`GameLogic::eraseSleepyUpdate`).
    pub fn erase<S: ScheduleStore<Handle>>(&mut self, store: &mut S, idx: usize) {
        let sz = self.entries.len();
        debug_assert!(idx < sz, "erase index out of bounds");
        if idx >= sz {
            return;
        }
        let removed = self.entries[idx];
        store.schedule_mut(removed).set_index_in_logic(INVALID_LOGIC_INDEX);
        let last_idx = sz - 1;
        if idx < last_idx {
            let moved = self.entries[last_idx];
            self.entries[idx] = moved;
            store.schedule_mut(moved).set_index_in_logic(idx as Int);
            self.entries.pop();
            self.rebalance(store, idx);
        } else {
            self.entries.pop();
        }
    }

    pub fn rebalance_parent<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        mut i: usize,
    ) -> usize {
        while i > 0 {
            let parent = (i - 1) >> 1;
            let h_parent = self.entries[parent];
            let h_i = self.entries[i];
            let pri_parent = store.schedule(h_parent).priority();
            let pri_i = store.schedule(h_i).priority();
            if !is_lower_priority(pri_parent, pri_i) {
                break;
            }
            self.entries[i] = h_parent;
            self.entries[parent] = h_i;
            store.schedule_mut(h_parent).set_index_in_logic(i as Int);
            store.schedule_mut(h_i).set_index_in_logic(parent as Int);
            i = parent;
        }
        i
    }

    pub fn rebalance_child<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        mut i: usize,
    ) -> usize {
        let sz = self.entries.len();
        let mut child = (i << 1) + 1;
        while child < sz {
            if child + 1 < sz {
                let pri_left = store.schedule(self.entries[child]).priority();
                let pri_right = store.schedule(self.entries[child + 1]).priority();
                if is_lower_priority(pri_left, pri_right) {
                    child += 1;
                }
            }
            let h_i = self.entries[i];
            let h_child = self.entries[child];
            let pri_i = store.schedule(h_i).priority();
            let pri_child = store.schedule(h_child).priority();
            if !is_lower_priority(pri_i, pri_child) {
                break;
            }
            self.entries[i] = h_child;
            self.entries[child] = h_i;
            store.schedule_mut(h_child).set_index_in_logic(i as Int);
            store.schedule_mut(h_i).set_index_in_logic(child as Int);
            i = child;
            child = (i << 1) + 1;
        }
        i
    }

    #[inline]
    pub fn rebalance<S: ScheduleStore<Handle>>(&mut self, store: &mut S, i: usize) {
        let next_i = self.rebalance_parent(store, i);
        self.rebalance_child(store, next_i);
    }

    /// Bottom-up heapify after loading (`GameLogic::remakeSleepyUpdate`).
    pub fn remake<S: ScheduleStore<Handle>>(&mut self, store: &mut S) {
        if self.entries.is_empty() {
            return;
        }
        let mut parent = self.entries.len() / 2;
        loop {
            self.rebalance_child(store, parent);
            if parent == 0 {
                break;
            }
            parent -= 1;
        }
    }

    /// Registers an update module into the scheduler (`GameLogic::registerObject`).
    pub fn register_update_module<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        handle: Handle,
        now: UnsignedInt,
    ) {
        let eff_now = if now == 0 { 1 } else { now };
        if store.schedule(handle).next_call_frame() == 0 {
            store.schedule_mut(handle).set_next_call_frame(eff_now);
        }
        self.push(store, handle);
    }

    /// Reschedules `handle` to wake on `when_to_wake_up` (`GameLogic::friend_awakenUpdateModule`).
    ///
    /// Returns `true` if `next_call_frame` changed.
    pub fn awaken_module<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        handle: Handle,
        in_world_list: bool,
        now: UnsignedInt,
        when_to_wake_up: UnsignedInt,
    ) -> bool {
        if self.cur_update_module == Some(handle) {
            return false;
        }
        let cur_next = store.schedule(handle).next_call_frame();
        if when_to_wake_up == cur_next {
            return false;
        }
        if now > 0 && cur_next == now && when_to_wake_up == now.wrapping_add(1) {
            return false;
        }

        let idx = store.schedule(handle).index_in_logic();
        if in_world_list {
            let usize_idx = usize::try_from(idx).unwrap_or(usize::MAX);
            assert!(
                usize_idx < self.entries.len() && self.entries[usize_idx] == handle,
                "sleepy update module index mismatch: idx={idx}"
            );
            store.schedule_mut(handle).set_next_call_frame(when_to_wake_up);
            self.rebalance(store, usize_idx);
            true
        } else {
            assert_eq!(
                idx, INVALID_LOGIC_INDEX,
                "unregistered sleepy update module should have index -1"
            );
            store.schedule_mut(handle).set_next_call_frame(when_to_wake_up);
            true
        }
    }

    /// Finalizes the root update at index `0` after `update()` completes or is skipped due to disabled state.
    pub fn finish_due_update_at_root<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        now: UnsignedInt,
        mut sleep_len: UpdateSleepTime,
    ) {
        self.cur_update_module = None;
        let Some(root) = self.peek() else {
            return;
        };
        if sleep_len.0 < 1 {
            sleep_len = UpdateSleepTime::NONE;
        }
        store
            .schedule_mut(root)
            .set_next_call_frame(now.saturating_add(sleep_len.0));
        self.rebalance(store, 0);
    }

    /// Verifies index-tracking and min-heap invariants across all entries (`GameLogic::validateSleepyUpdate`).
    pub fn validate_invariants<S: ScheduleStore<Handle>>(&self, store: &S) -> Result<(), String> {
        let sz = self.entries.len();
        for (i, &h) in self.entries.iter().enumerate() {
            let st = store.schedule(h);
            if st.index_in_logic() != i as Int {
                return Err(format!(
                    "index mismatch at slot {i}: expected {i}, got {}",
                    st.index_in_logic()
                ));
            }
            let pri = st.priority();
            if i > 0 {
                let p = (i - 1) >> 1;
                let p_pri = store.schedule(self.entries[p]).priority();
                if pri < p_pri {
                    return Err(format!(
                        "heap violated at {i} vs parent {p}: pri={pri} < parent_pri={p_pri}"
                    ));
                }
            }
        }
        for i in 0..sz {
            let pri = store.schedule(self.entries[i]).priority();
            let c1 = (i << 1) + 1;
            let c2 = (i << 1) + 2;
            if c1 < sz && pri > store.schedule(self.entries[c1]).priority() {
                return Err(format!("left child heap violation at {i} -> {c1}"));
            }
            if c2 < sz && pri > store.schedule(self.entries[c2]).priority() {
                return Err(format!("right child heap violation at {i} -> {c2}"));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_mask_parity_respects_retail_crc_flag() {
        // Object disabled bits = {0, 1} (= 0b11), module accepts only bit {0} (= 0b01).
        let allowed = should_process_disabled_update(0b11, 0b01);
        if cfg!(feature = "retail-crc") {
            assert!(allowed);
        } else {
            assert!(!allowed);
        }
        // Object disabled bits = {0}, module accepts {0, 2}: allowed in both modes.
        assert!(should_process_disabled_update(0b001, 0b101));
    }
}
