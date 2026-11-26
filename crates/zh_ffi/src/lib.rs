//! C-ABI Strangler FFI bridge (`libzh_ffi.a`) matching `crates/zh_ffi/include/zh_ffi.h`.

pub mod economy;
pub mod engine_sim;
mod engine_sim_generated;
pub mod lifetime_update;
pub mod sleepy_heap;

use core::ffi::c_void;
use core::slice;
use zh_common::crc::Crc;
use zh_common::random_value::{
    get_audio_state_words, get_client_state_words, get_game_audio_random_value,
    get_game_audio_random_value_real, get_game_client_random_value,
    get_game_client_random_value_real, get_game_logic_random_seed, get_game_logic_random_seed_crc,
    get_game_logic_random_value, get_game_logic_random_value_real,
    get_game_logic_random_value_real_unchanged, get_game_logic_random_value_unchanged,
    get_logic_state_words, init_random, init_random_time, set_audio_state_words,
    set_client_state_words, set_logic_state_words,
};
use zh_common::types::{Bool, Int, ObjectId, Relationship, VeterancyLevel, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferLoad, XferSave};
use zh_logic::object::experience_tracker::{ExperienceParentAccess, ExperienceTrackerState};

// ============================================================================
// Pattern 1: PKT-001 (crc) & PKT-002 (RandomValue)
// ============================================================================

#[no_mangle]
pub unsafe extern "C" fn zh_crc_compute_bytes(
    initial_crc: u32,
    buf: *const u8,
    len: i32,
) -> u32 {
    let mut c = Crc::with_initial(initial_crc);
    if !buf.is_null() && len > 0 {
        let bytes = unsafe { slice::from_raw_parts(buf, len as usize) };
        c.compute_crc(bytes, len);
    } else {
        c.compute_crc(&[], len);
    }
    c.get()
}

#[no_mangle]
pub extern "C" fn zh_random_init_time() {
    init_random_time();
}

#[no_mangle]
pub extern "C" fn zh_random_init_seed(seed: u32) {
    init_random(seed);
}

#[no_mangle]
pub extern "C" fn zh_random_get_logic_base_seed() -> u32 {
    get_game_logic_random_seed()
}

#[no_mangle]
pub extern "C" fn zh_random_get_logic_seed_crc() -> u32 {
    get_game_logic_random_seed_crc()
}

#[no_mangle]
pub extern "C" fn zh_random_get_logic_int(lo: i32, hi: i32) -> i32 {
    get_game_logic_random_value(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_logic_real(lo: f32, hi: f32) -> f32 {
    get_game_logic_random_value_real(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_logic_int_unchanged(lo: i32, hi: i32) -> i32 {
    get_game_logic_random_value_unchanged(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_logic_real_unchanged(lo: f32, hi: f32) -> f32 {
    get_game_logic_random_value_real_unchanged(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_client_int(lo: i32, hi: i32) -> i32 {
    get_game_client_random_value(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_client_real(lo: f32, hi: f32) -> f32 {
    get_game_client_random_value_real(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_audio_int(lo: i32, hi: i32) -> i32 {
    get_game_audio_random_value(lo, hi)
}

#[no_mangle]
pub extern "C" fn zh_random_get_audio_real(lo: f32, hi: f32) -> f32 {
    get_game_audio_random_value_real(lo, hi)
}

#[no_mangle]
pub unsafe extern "C" fn zh_random_get_logic_state_words(out_words: *mut u32) {
    if !out_words.is_null() {
        let words = get_logic_state_words();
        unsafe {
            core::ptr::copy_nonoverlapping(words.as_ptr(), out_words, 6);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_random_set_logic_state_words(in_words: *const u32, base_seed: u32) {
    if !in_words.is_null() {
        let mut words = [0u32; 6];
        unsafe {
            core::ptr::copy_nonoverlapping(in_words, words.as_mut_ptr(), 6);
        }
        set_logic_state_words(words, base_seed);
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_random_get_client_state_words(out_words: *mut u32) {
    if !out_words.is_null() {
        let words = get_client_state_words();
        unsafe {
            core::ptr::copy_nonoverlapping(words.as_ptr(), out_words, 6);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_random_set_client_state_words(in_words: *const u32) {
    if !in_words.is_null() {
        let mut words = [0u32; 6];
        unsafe {
            core::ptr::copy_nonoverlapping(in_words, words.as_mut_ptr(), 6);
        }
        set_client_state_words(words);
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_random_get_audio_state_words(out_words: *mut u32) {
    if !out_words.is_null() {
        let words = get_audio_state_words();
        unsafe {
            core::ptr::copy_nonoverlapping(words.as_ptr(), out_words, 6);
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_random_set_audio_state_words(in_words: *const u32) {
    if !in_words.is_null() {
        let mut words = [0u32; 6];
        unsafe {
            core::ptr::copy_nonoverlapping(in_words, words.as_mut_ptr(), 6);
        }
        set_audio_state_words(words);
    }
}

// ============================================================================
// Pattern 2: PKT-003 (ExperienceTracker)
// ============================================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ZhExpTrackerState {
    pub current_level: i32,
    pub current_experience: i32,
    pub experience_sink: i32,
    pub experience_scalar: f32,
    pub is_trainable: u8,
}

impl ZhExpTrackerState {
    #[inline]
    fn to_rust(self) -> ExperienceTrackerState {
        ExperienceTrackerState {
            current_level: VeterancyLevel::from_i32(self.current_level),
            current_experience: self.current_experience,
            experience_sink: ObjectId(self.experience_sink),
            experience_scalar: self.experience_scalar,
            is_trainable: self.is_trainable,
        }
    }

    #[inline]
    fn write_from_rust(&mut self, s: &ExperienceTrackerState) {
        self.current_level = s.current_level.as_i32();
        self.current_experience = s.current_experience;
        self.experience_sink = s.experience_sink.0;
        self.experience_scalar = s.experience_scalar;
        self.is_trainable = s.is_trainable;
    }
}

#[repr(C)]
pub struct ZhExpTrackerParentCallbacks {
    pub parent_ctx: *mut c_void,
    pub is_template_trainable: Option<unsafe extern "C" fn(*mut c_void) -> u8>,
    pub get_experience_required: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub get_experience_value: Option<unsafe extern "C" fn(*mut c_void, i32) -> i32>,
    pub get_relationship_with_killer:
        Option<unsafe extern "C" fn(*mut c_void, *const c_void) -> i32>,
    pub on_veterancy_level_changed:
        Option<unsafe extern "C" fn(*mut c_void, i32, i32, u8)>,
    pub forward_experience_to_sink:
        Option<unsafe extern "C" fn(*mut c_void, i32, i32, u8) -> u8>,
    pub forward_set_exp_and_level_to_sink:
        Option<unsafe extern "C" fn(*mut c_void, i32, i32, u8) -> u8>,
}

struct FfiExperienceParent<'a> {
    cb: &'a ZhExpTrackerParentCallbacks,
    killer_ctx: *const c_void,
    /// A level change to report once the new state is in place.
    level_change: Option<(VeterancyLevel, VeterancyLevel, Bool)>,
}

/// Runs `op` on the tracker in `state`, stores the result, then reports a level change.
///
/// C++ writes the new level and experience before `onVeterancyLevelChanged`, which
/// reads them and may change the tracker again, so nothing is written after it.
/// An operation that leaves its own tracker alone (forwarding to a sink) writes
/// nothing, so changes the sink makes to it survive.
unsafe fn run_tracker_op<R>(
    state: *mut ZhExpTrackerState,
    cb: &ZhExpTrackerParentCallbacks,
    op: impl FnOnce(&mut ExperienceTrackerState, &mut FfiExperienceParent<'_>) -> R,
) -> R {
    let mut parent = FfiExperienceParent {
        cb,
        killer_ctx: core::ptr::null(),
        level_change: None,
    };
    let before = unsafe { (*state).to_rust() };
    let mut after = before.clone();
    let out = op(&mut after, &mut parent);
    if after != before {
        unsafe { (*state).write_from_rust(&after) };
    }
    if let (Some((old, new, feedback)), Some(f)) =
        (parent.level_change, cb.on_veterancy_level_changed)
    {
        unsafe { f(cb.parent_ctx, old.as_i32(), new.as_i32(), feedback) };
    }
    out
}

impl<'a> ExperienceParentAccess for FfiExperienceParent<'a> {
    fn is_template_trainable(&self) -> Bool {
        if let Some(f) = self.cb.is_template_trainable {
            unsafe { f(self.cb.parent_ctx) }
        } else {
            FALSE
        }
    }

    fn get_experience_required(&self, level: Int) -> Int {
        if let Some(f) = self.cb.get_experience_required {
            unsafe { f(self.cb.parent_ctx, level) }
        } else {
            0
        }
    }

    fn get_experience_value(&self, level: Int) -> Int {
        if let Some(f) = self.cb.get_experience_value {
            unsafe { f(self.cb.parent_ctx, level) }
        } else {
            0
        }
    }

    fn get_relationship_with_killer(&self) -> Relationship {
        if let Some(f) = self.cb.get_relationship_with_killer {
            Relationship::from_i32(unsafe { f(self.cb.parent_ctx, self.killer_ctx) })
        } else {
            Relationship::Enemies
        }
    }

    fn on_veterancy_level_changed(
        &mut self,
        old_level: VeterancyLevel,
        new_level: VeterancyLevel,
        provide_feedback: Bool,
    ) {
        // Each tracker operation reports at most one change, as its last step.
        self.level_change = Some((old_level, new_level, provide_feedback));
    }

    fn forward_experience_to_sink(
        &mut self,
        sink_id: ObjectId,
        scaled_exp_gain: Int,
        can_scale_for_bonus: Bool,
    ) -> bool {
        if let Some(f) = self.cb.forward_experience_to_sink {
            (unsafe { f(self.cb.parent_ctx, sink_id.0, scaled_exp_gain, can_scale_for_bonus) }) != 0
        } else {
            false
        }
    }

    fn forward_set_exp_and_level_to_sink(
        &mut self,
        sink_id: ObjectId,
        experience_in: Int,
        provide_feedback: Bool,
    ) -> bool {
        if let Some(f) = self.cb.forward_set_exp_and_level_to_sink {
            (unsafe { f(self.cb.parent_ctx, sink_id.0, experience_in, provide_feedback) }) != 0
        } else {
            false
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_init(
    state: *mut ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
) {
    if state.is_null() || cb.is_null() {
        return;
    }
    let parent = FfiExperienceParent {
        cb: unsafe { &*cb },
        killer_ctx: core::ptr::null(),
        level_change: None,
    };
    let s = ExperienceTrackerState::new(&parent);
    unsafe { (*state).write_from_rust(&s) };
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_get_experience_value(
    state: *const ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
    killer_ctx: *const c_void,
) -> i32 {
    if state.is_null() || cb.is_null() {
        return 0;
    }
    let parent = FfiExperienceParent {
        cb: unsafe { &*cb },
        killer_ctx,
        level_change: None,
    };
    let s = unsafe { (*state).to_rust() };
    s.get_experience_value(&parent)
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_is_accepting_exp(
    state: *const ZhExpTrackerState,
) -> u8 {
    if state.is_null() {
        return FALSE;
    }
    let s = unsafe { (*state).to_rust() };
    s.is_accepting_experience_points()
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_set_min_veterancy_level(
    state: *mut ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
    new_level: i32,
    provide_feedback: u8,
) {
    if state.is_null() || cb.is_null() {
        return;
    }
    let level = VeterancyLevel::from_i32(new_level);
    unsafe {
        run_tracker_op(state, &*cb, |s, p| s.set_min_veterancy_level(p, level, provide_feedback));
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_set_veterancy_level(
    state: *mut ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
    new_level: i32,
    provide_feedback: u8,
) {
    if state.is_null() || cb.is_null() {
        return;
    }
    let level = VeterancyLevel::from_i32(new_level);
    unsafe {
        run_tracker_op(state, &*cb, |s, p| s.set_veterancy_level(p, level, provide_feedback));
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_gain_exp_for_level(
    state: *mut ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
    levels_to_gain: i32,
    can_scale_for_bonus: u8,
) -> u8 {
    if state.is_null() || cb.is_null() {
        return FALSE;
    }
    unsafe {
        run_tracker_op(state, &*cb, |s, p| {
            s.gain_exp_for_level(p, levels_to_gain, can_scale_for_bonus)
        })
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_can_gain_exp_for_level(
    state: *const ZhExpTrackerState,
    levels_to_gain: i32,
) -> u8 {
    if state.is_null() {
        return FALSE;
    }
    let s = unsafe { (*state).to_rust() };
    s.can_gain_exp_for_level(levels_to_gain)
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_add_experience_points(
    state: *mut ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
    experience_gain: i32,
    can_scale_for_bonus: u8,
) {
    if state.is_null() || cb.is_null() {
        return;
    }
    unsafe {
        run_tracker_op(state, &*cb, |s, p| {
            s.add_experience_points(p, experience_gain, can_scale_for_bonus)
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_set_experience_and_level(
    state: *mut ZhExpTrackerState,
    cb: *const ZhExpTrackerParentCallbacks,
    experience_in: i32,
    provide_feedback: u8,
) {
    if state.is_null() || cb.is_null() {
        return;
    }
    unsafe {
        run_tracker_op(state, &*cb, |s, p| {
            s.set_experience_and_level(p, experience_in, provide_feedback)
        });
    }
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_crc(
    state: *const ZhExpTrackerState,
    initial_crc: u32,
) -> u32 {
    if state.is_null() {
        return initial_crc;
    }
    let s = unsafe { (*state).to_rust() };
    let mut xfer = XferCrc::with_initial(initial_crc);
    s.crc(&mut xfer);
    xfer.get_crc()
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_xfer_save(
    state: *const ZhExpTrackerState,
    out_buf: *mut u8,
    max_len: i32,
) -> i32 {
    if state.is_null() || out_buf.is_null() || max_len <= 0 {
        return -1;
    }
    let mut s = unsafe { (*state).to_rust() };
    let mut xfer = XferSave::new();
    if s.xfer(&mut xfer).is_err() {
        return -1;
    }
    let bytes = xfer.as_bytes();
    if bytes.len() > (max_len as usize) {
        return -1;
    }
    unsafe {
        core::ptr::copy_nonoverlapping(bytes.as_ptr(), out_buf, bytes.len());
    }
    bytes.len() as i32
}

#[no_mangle]
pub unsafe extern "C" fn zh_exp_tracker_xfer_load(
    state: *mut ZhExpTrackerState,
    in_buf: *const u8,
    len: i32,
) -> u8 {
    if state.is_null() || in_buf.is_null() || len <= 0 {
        return FALSE;
    }
    let bytes = unsafe { slice::from_raw_parts(in_buf, len as usize) };
    let mut xfer = XferLoad::new(bytes);
    let mut s = unsafe { (*state).to_rust() };
    if s.xfer(&mut xfer).is_err() {
        return FALSE;
    }
    unsafe { (*state).write_from_rust(&s) };
    TRUE
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Parent {
        state: *mut ZhExpTrackerState,
        seen: Vec<(i32, i32, i32, i32)>,
        sink_hits: u32,
    }

    unsafe extern "C" fn required(_ctx: *mut c_void, level: i32) -> i32 {
        level * 10
    }

    unsafe extern "C" fn changed(ctx: *mut c_void, old: i32, new: i32, _feedback: u8) {
        let p = unsafe { &mut *ctx.cast::<Parent>() };
        let state = unsafe { &mut *p.state };
        p.seen
            .push((old, new, state.current_level, state.current_experience));
        // A change made while handling the notice must survive.
        state.experience_scalar = 7.0;
    }

    unsafe extern "C" fn sink(ctx: *mut c_void, _id: i32, _gain: i32, _can: u8) -> u8 {
        let p = unsafe { &mut *ctx.cast::<Parent>() };
        p.sink_hits += 1;
        unsafe { (*p.state).experience_scalar = 3.0 };
        1
    }

    fn setup(parent: &mut Parent, state: &mut ZhExpTrackerState) -> ZhExpTrackerParentCallbacks {
        parent.state = state;
        ZhExpTrackerParentCallbacks {
            parent_ctx: (parent as *mut Parent).cast(),
            is_template_trainable: None,
            get_experience_required: Some(required),
            get_experience_value: None,
            get_relationship_with_killer: None,
            on_veterancy_level_changed: Some(changed),
            forward_experience_to_sink: Some(sink),
            forward_set_exp_and_level_to_sink: None,
        }
    }

    fn fresh() -> (Parent, ZhExpTrackerState) {
        (
            Parent {
                state: core::ptr::null_mut(),
                seen: Vec::new(),
                sink_hits: 0,
            },
            ZhExpTrackerState {
                current_level: 0,
                current_experience: 0,
                experience_sink: 0,
                experience_scalar: 1.0,
                is_trainable: 1,
            },
        )
    }

    #[test]
    fn level_change_is_reported_after_the_state_is_stored() {
        let (mut parent, mut state) = fresh();
        let cb = setup(&mut parent, &mut state);
        let state_ptr = parent.state;
        unsafe { zh_exp_tracker_add_experience_points(state_ptr, &cb, 25, 1) };
        assert_eq!(parent.seen, vec![(0, 2, 2, 25)]);
        assert_eq!(state.experience_scalar, 7.0);
        unsafe { zh_exp_tracker_set_veterancy_level(state_ptr, &cb, 3, 0) };
        assert_eq!(parent.seen[1], (2, 3, 3, 30));
    }

    #[test]
    fn forwarding_to_a_sink_does_not_overwrite_the_tracker() {
        let (mut parent, mut state) = fresh();
        state.experience_sink = 9;
        let cb = setup(&mut parent, &mut state);
        let state_ptr = parent.state;
        unsafe { zh_exp_tracker_add_experience_points(state_ptr, &cb, 25, 1) };
        assert_eq!(parent.sink_hits, 1);
        assert_eq!(state.experience_scalar, 3.0);
        assert_eq!(state.current_experience, 0);
    }
}
