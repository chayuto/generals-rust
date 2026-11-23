//! `WP-7.1`: Deterministic 30 Hz sleepy-update scheduler (`GameLogic::m_sleepyUpdates`).

pub mod sleepy_heap;

pub use sleepy_heap::{
    should_process_disabled_update, ScheduleStore, SleepyUpdateHeap, SleepyUpdatePhase,
    UpdateModuleScheduleState, INVALID_LOGIC_INDEX, MAX_SLEEPY_UPDATES_PER_OBJECT,
    UPDATE_SLEEP_FOREVER, UPDATE_SLEEP_INVALID, UPDATE_SLEEP_NONE,
};
