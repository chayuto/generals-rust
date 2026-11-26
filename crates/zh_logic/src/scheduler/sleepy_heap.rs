//! Exact deterministic port of Westwood/SAGE `GameLogic::m_sleepyUpdates` array min-heap
//! (`GeneralsMD/Code/GameEngine/Source/GameLogic/System/GameLogic.cpp` lines 3073–3359 & 4062–4113)
//! and `UpdateModule` scheduling metadata (`UpdateModule.h` / `UpdateModule.cpp`).

use zh_common::types::{Int, UnsignedInt, UpdateSleepTime};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const UPDATE_SLEEP_INVALID: UnsignedInt = 0;
pub const UPDATE_SLEEP_NONE: UnsignedInt = 1;
pub const UPDATE_SLEEP_FOREVER: UnsignedInt = 0x3fff_ffff;
pub const INVALID_LOGIC_INDEX: Int = -1;
pub const MAX_SLEEPY_UPDATES_PER_OBJECT: usize = 256;

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

/// What the heap needs from an update module: its priority and somewhere to keep its slot.
pub trait HeapStore<Handle> {
    fn priority(&self, handle: Handle) -> UnsignedInt;
    fn set_index(&mut self, handle: Handle, index: Int);
}

impl<Handle, S: ScheduleStore<Handle>> HeapStore<Handle> for S {
    #[inline]
    fn priority(&self, handle: Handle) -> UnsignedInt {
        self.schedule(handle).priority()
    }

    #[inline]
    fn set_index(&mut self, handle: Handle, index: Int) {
        self.schedule_mut(handle).set_index_in_logic(index);
    }
}

#[inline]
const fn is_lower_priority(a_pri: UnsignedInt, b_pri: UnsignedInt) -> bool {
    a_pri > b_pri
}

fn swap_slots<Handle: Copy, S: HeapStore<Handle>>(
    entries: &mut [Handle],
    store: &mut S,
    upper: usize,
    lower: usize,
) {
    let (a, b) = (entries[upper], entries[lower]);
    entries[lower] = a;
    entries[upper] = b;
    store.set_index(a, lower as Int);
    store.set_index(b, upper as Int);
}

/// `GameLogic::rebalanceParentSleepyUpdate`. Returns where the entry ended up.
pub fn sift_up<Handle: Copy, S: HeapStore<Handle>>(
    entries: &mut [Handle],
    store: &mut S,
    mut i: usize,
) -> usize {
    while i > 0 {
        let parent = (i - 1) >> 1;
        if !is_lower_priority(store.priority(entries[parent]), store.priority(entries[i])) {
            break;
        }
        swap_slots(entries, store, parent, i);
        i = parent;
    }
    i
}

/// `GameLogic::rebalanceChildSleepyUpdate`. Returns where the entry ended up.
pub fn sift_down<Handle: Copy, S: HeapStore<Handle>>(
    entries: &mut [Handle],
    store: &mut S,
    mut i: usize,
) -> usize {
    let sz = entries.len();
    let mut child = (i << 1) + 1;
    while child < sz {
        if child + 1 < sz
            && is_lower_priority(
                store.priority(entries[child]),
                store.priority(entries[child + 1]),
            )
        {
            child += 1;
        }
        if !is_lower_priority(store.priority(entries[i]), store.priority(entries[child])) {
            break;
        }
        swap_slots(entries, store, i, child);
        i = child;
        child = (i << 1) + 1;
    }
    i
}

/// `GameLogic::rebalanceSleepyUpdate`.
pub fn rebalance_slot<Handle: Copy, S: HeapStore<Handle>>(
    entries: &mut [Handle],
    store: &mut S,
    i: usize,
) {
    let next = sift_up(entries, store, i);
    sift_down(entries, store, next);
}

/// `GameLogic::pushSleepyUpdate`, once the new module is the last entry.
pub fn push_appended<Handle: Copy, S: HeapStore<Handle>>(entries: &mut [Handle], store: &mut S) {
    let Some(last) = entries.len().checked_sub(1) else {
        return;
    };
    store.set_index(entries[last], last as Int);
    sift_up(entries, store, last);
}

/// `GameLogic::eraseSleepyUpdate`. Returns the new length; the caller drops the tail.
pub fn remove_slot<Handle: Copy, S: HeapStore<Handle>>(
    entries: &mut [Handle],
    store: &mut S,
    i: usize,
) -> usize {
    store.set_index(entries[i], INVALID_LOGIC_INDEX);
    let last = entries.len() - 1;
    if i < last {
        entries[i] = entries[last];
        store.set_index(entries[i], i as Int);
        rebalance_slot(&mut entries[..last], store, i);
    }
    last
}

/// `GameLogic::remakeSleepyUpdate`: bottom-up heapify after loading.
pub fn heapify<Handle: Copy, S: HeapStore<Handle>>(entries: &mut [Handle], store: &mut S) {
    if entries.is_empty() {
        return;
    }
    let mut parent = entries.len() / 2;
    loop {
        sift_down(entries, store, parent);
        if parent == 0 {
            break;
        }
        parent -= 1;
    }
}

/// The cases where `GameLogic::friend_awakenUpdateModule` leaves the module alone.
#[inline]
pub const fn awaken_is_ignored(
    is_current_module: bool,
    next_call_frame: UnsignedInt,
    now: UnsignedInt,
    when_to_wake_up: UnsignedInt,
) -> bool {
    // A module inside its own update() is rescheduled by the return value instead, and
    // a module already awake this frame must not be pushed back to the next one.
    is_current_module
        || when_to_wake_up == next_call_frame
        || (now > 0 && next_call_frame == now && when_to_wake_up == now.wrapping_add(1))
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
        self.entries.push(handle);
        push_appended(&mut self.entries, store);
    }

    /// Appends `handle` for bulk load reconstruction prior to [`Self::remake`].
    pub fn push_raw_for_load<S: ScheduleStore<Handle>>(&mut self, store: &mut S, handle: Handle) {
        let idx = self.entries.len();
        self.entries.push(handle);
        store.schedule_mut(handle).set_index_in_logic(idx as Int);
    }

    /// Removes and returns the root handle (`GameLogic::popSleepyUpdate`).
    pub fn pop<S: ScheduleStore<Handle>>(&mut self, store: &mut S) -> Option<Handle> {
        let root = self.peek()?;
        self.erase(store, 0);
        Some(root)
    }

    /// Erases the entry at `idx` (`GameLogic::eraseSleepyUpdate`).
    pub fn erase<S: ScheduleStore<Handle>>(&mut self, store: &mut S, idx: usize) {
        debug_assert!(idx < self.entries.len(), "erase index out of bounds");
        if idx >= self.entries.len() {
            return;
        }
        let len = remove_slot(&mut self.entries, store, idx);
        self.entries.truncate(len);
    }

    pub fn rebalance_parent<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        i: usize,
    ) -> usize {
        sift_up(&mut self.entries, store, i)
    }

    pub fn rebalance_child<S: ScheduleStore<Handle>>(&mut self, store: &mut S, i: usize) -> usize {
        sift_down(&mut self.entries, store, i)
    }

    #[inline]
    pub fn rebalance<S: ScheduleStore<Handle>>(&mut self, store: &mut S, i: usize) {
        rebalance_slot(&mut self.entries, store, i);
    }

    /// Bottom-up heapify after loading (`GameLogic::remakeSleepyUpdate`).
    pub fn remake<S: ScheduleStore<Handle>>(&mut self, store: &mut S) {
        heapify(&mut self.entries, store);
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
        let is_current = self.cur_update_module == Some(handle);
        let next = store.schedule(handle).next_call_frame();
        if awaken_is_ignored(is_current, next, now, when_to_wake_up) {
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

    /// Reschedules the module that just ran and rebalances **slot 0**, as
    /// `GameLogic.cpp:4108-4109` does. `update()` can put another module there (an object
    /// created this frame whose phase runs earlier); C++ then leaves the updated module
    /// where it is, so the heap order is not restored, and this must match.
    pub fn finish_due_update<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        handle: Handle,
        now: UnsignedInt,
        mut sleep_len: UpdateSleepTime,
    ) {
        self.cur_update_module = None;
        if sleep_len.0 < 1 {
            sleep_len = UpdateSleepTime::NONE;
        }
        store
            .schedule_mut(handle)
            .set_next_call_frame(now.wrapping_add(sleep_len.0));
        if !self.entries.is_empty() {
            self.rebalance(store, 0);
        }
    }

    /// Restores `next_call_frame` unconditionally after snapshot load and rebalances if queued.
    pub fn restore_module_schedule<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        handle: Handle,
        next_call_frame: UnsignedInt,
    ) {
        store.schedule_mut(handle).set_next_call_frame(next_call_frame);
        let idx = store.schedule(handle).index_in_logic();
        if idx >= 0 {
            let usize_idx = idx as usize;
            if usize_idx < self.entries.len() && self.entries[usize_idx] == handle {
                self.rebalance(store, usize_idx);
            }
        }
    }

    /// Finalizes the executing update (`cur_update_module`) or the root entry if disabled-skipped.
    pub fn finish_due_update_at_root<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        now: UnsignedInt,
        sleep_len: UpdateSleepTime,
    ) {
        let Some(handle) = self.cur_update_module.take().or_else(|| self.peek()) else {
            return;
        };
        self.finish_due_update(store, handle, now, sleep_len);
    }

    /// Two-pass reverse-heap-index removal for all modules owned by a destroyed object
    /// (`GameLogic::processDestroyList`, `GameLogic.cpp:2704-2723`).
    pub fn erase_for_owner<S: ScheduleStore<Handle>>(
        &mut self,
        store: &mut S,
        mut belongs_to_owner: impl FnMut(&S, Handle) -> bool,
    ) -> usize {
        let mut matched: Vec<Handle> = Vec::new();
        for &h in &self.entries {
            if belongs_to_owner(store, h) && matched.len() < MAX_SLEEPY_UPDATES_PER_OBJECT {
                matched.push(h);
            }
        }
        let erased = matched.len();
        for h in matched.into_iter().rev() {
            let idx = store.schedule(h).index_in_logic();
            let usize_idx = usize::try_from(idx).unwrap_or(usize::MAX);
            assert!(
                usize_idx < self.entries.len() && self.entries[usize_idx] == h,
                "sleepy update erase_for_owner mismatch: idx={idx}"
            );
            self.erase(store, usize_idx);
            debug_assert_eq!(store.schedule(h).index_in_logic(), INVALID_LOGIC_INDEX);
        }
        erased
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
