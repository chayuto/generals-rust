//! Entry points that run the ported sleepy-update heap (`PKT-087` spine, `WP-7.1`)
//! on the engine's own `GameLogic::m_sleepyUpdates` array.
//!
//! The C++ vector stays the storage. Rust reorders its elements in place and reads
//! each module's priority and writes its slot through the callbacks, which must not
//! touch the vector.

use core::ffi::c_void;
use core::slice;
use zh_common::types::{Int, UnsignedInt};
use zh_logic::scheduler::{
    awaken_is_ignored, heapify, push_appended, rebalance_slot, remove_slot, sift_down, sift_up,
    HeapStore,
};

/// Mirrors `ZhSleepyCallbacks` in `zh_ffi.h`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ZhSleepyCallbacks {
    pub priority: extern "C" fn(*mut c_void) -> u32,
    pub set_index: extern "C" fn(*mut c_void, i32),
}

impl HeapStore<*mut c_void> for ZhSleepyCallbacks {
    fn priority(&self, module: *mut c_void) -> UnsignedInt {
        (self.priority)(module)
    }

    fn set_index(&mut self, module: *mut c_void, index: Int) {
        (self.set_index)(module, index);
    }
}

/// # Safety
/// `entries` must point to `len` module pointers (it may be null when `len` is 0) and
/// `callbacks` must be non-null.
unsafe fn heap<'a>(
    entries: *mut *mut c_void,
    len: i32,
    callbacks: *const ZhSleepyCallbacks,
) -> (&'a mut [*mut c_void], ZhSleepyCallbacks) {
    let len = usize::try_from(len).expect("negative heap length");
    let entries = if len == 0 {
        &mut []
    } else {
        unsafe { slice::from_raw_parts_mut(entries, len) }
    };
    (entries, unsafe { *callbacks })
}

fn slot(i: i32) -> usize {
    usize::try_from(i).expect("negative heap slot")
}

/// # Safety
/// As [`heap`]; `i` must be below `len`.
#[no_mangle]
pub unsafe extern "C" fn zh_sleepy_sift_up(
    entries: *mut *mut c_void,
    len: i32,
    i: i32,
    callbacks: *const ZhSleepyCallbacks,
) -> i32 {
    let (entries, mut store) = unsafe { heap(entries, len, callbacks) };
    sift_up(entries, &mut store, slot(i)) as i32
}

/// # Safety
/// As [`heap`]; `i` must be below `len`, or both 0.
#[no_mangle]
pub unsafe extern "C" fn zh_sleepy_sift_down(
    entries: *mut *mut c_void,
    len: i32,
    i: i32,
    callbacks: *const ZhSleepyCallbacks,
) -> i32 {
    let (entries, mut store) = unsafe { heap(entries, len, callbacks) };
    sift_down(entries, &mut store, slot(i)) as i32
}

/// # Safety
/// As [`heap`]; `i` must be below `len`.
#[no_mangle]
pub unsafe extern "C" fn zh_sleepy_rebalance(
    entries: *mut *mut c_void,
    len: i32,
    i: i32,
    callbacks: *const ZhSleepyCallbacks,
) {
    let (entries, mut store) = unsafe { heap(entries, len, callbacks) };
    rebalance_slot(entries, &mut store, slot(i));
}

/// The caller has appended the new module.
///
/// # Safety
/// As [`heap`].
#[no_mangle]
pub unsafe extern "C" fn zh_sleepy_pushed(
    entries: *mut *mut c_void,
    len: i32,
    callbacks: *const ZhSleepyCallbacks,
) {
    let (entries, mut store) = unsafe { heap(entries, len, callbacks) };
    push_appended(entries, &mut store);
}

/// Returns the new length; the caller drops the last element.
///
/// # Safety
/// As [`heap`]; `i` must be below `len`.
#[no_mangle]
pub unsafe extern "C" fn zh_sleepy_remove(
    entries: *mut *mut c_void,
    len: i32,
    i: i32,
    callbacks: *const ZhSleepyCallbacks,
) -> i32 {
    let (entries, mut store) = unsafe { heap(entries, len, callbacks) };
    remove_slot(entries, &mut store, slot(i)) as i32
}

/// # Safety
/// As [`heap`].
#[no_mangle]
pub unsafe extern "C" fn zh_sleepy_remake(
    entries: *mut *mut c_void,
    len: i32,
    callbacks: *const ZhSleepyCallbacks,
) {
    let (entries, mut store) = unsafe { heap(entries, len, callbacks) };
    heapify(entries, &mut store);
}

#[no_mangle]
pub extern "C" fn zh_sleepy_awaken_ignored(
    is_current_module: u8,
    next_call_frame: u32,
    now: u32,
    when_to_wake_up: u32,
) -> u8 {
    u8::from(awaken_is_ignored(
        is_current_module != 0,
        next_call_frame,
        now,
        when_to_wake_up,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Module {
        priority: u32,
        index: i32,
    }

    extern "C" fn priority(module: *mut c_void) -> u32 {
        unsafe { (*module.cast::<Module>()).priority }
    }
    extern "C" fn set_index(module: *mut c_void, index: i32) {
        unsafe { (*module.cast::<Module>()).index = index }
    }
    const CALLBACKS: ZhSleepyCallbacks = ZhSleepyCallbacks {
        priority,
        set_index,
    };

    fn assert_indices(entries: &[*mut c_void]) {
        for (i, &module) in entries.iter().enumerate() {
            assert_eq!(unsafe { (*module.cast::<Module>()).index }, i as i32);
        }
    }

    #[test]
    fn orders_the_callers_array_and_keeps_slots_current() {
        let mut modules: Vec<Module> = [50, 10, 40, 20, 30]
            .iter()
            .map(|&priority| Module {
                priority,
                index: -1,
            })
            .collect();
        let base = modules.as_mut_ptr();
        let mut entries: Vec<*mut c_void> = Vec::new();
        for i in 0..5 {
            entries.push(unsafe { base.add(i) }.cast());
            unsafe { zh_sleepy_pushed(entries.as_mut_ptr(), entries.len() as i32, &CALLBACKS) };
            assert_indices(&entries);
        }
        let mut popped = Vec::new();
        while !entries.is_empty() {
            popped.push(priority(entries[0]));
            let root = entries[0];
            let len = unsafe {
                zh_sleepy_remove(entries.as_mut_ptr(), entries.len() as i32, 0, &CALLBACKS)
            };
            entries.truncate(len as usize);
            assert_eq!(unsafe { (*root.cast::<Module>()).index }, -1);
            assert_indices(&entries);
        }
        assert_eq!(popped, [10, 20, 30, 40, 50]);
    }

    #[test]
    fn remake_accepts_an_empty_vector() {
        unsafe { zh_sleepy_remake(core::ptr::null_mut(), 0, &CALLBACKS) };
    }
}
