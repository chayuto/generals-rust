//! Entry points that run the ported `RadarUpdate` (`PKT-358`) for an engine object.
//!
//! The C++ module keeps the three saved members, so `crc` and `xfer` are unchanged.
//! The constructor, which only zeroes them, stays in C++.

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks};
use zh_logic::modules::module::RadarFacet;
use zh_logic::modules::update::radar_update::{RadarUpdate, RadarUpdateModuleData};
use zh_logic::modules::UpdateFacet;

/// Mirrors `ZhRadarState` in `zh_ffi.h`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ZhRadarState {
    pub extend_done_frame: u32,
    pub extend_complete: u8,
    pub radar_active: u8,
}

fn module(radar_extend_time: f32, state: ZhRadarState) -> RadarUpdate {
    let mut data = RadarUpdateModuleData::new(Default::default(), Default::default());
    data.radar_extend_time = radar_extend_time;
    let mut update = RadarUpdate::new(data);
    update.extend_done_frame = state.extend_done_frame;
    update.extend_complete = state.extend_complete;
    update.radar_active = state.radar_active;
    update
}

fn saved(update: &RadarUpdate) -> ZhRadarState {
    ZhRadarState {
        extend_done_frame: update.extend_done_frame,
        extend_complete: update.extend_complete,
        radar_active: update.radar_active,
    }
}

/// `extendRadar`.
///
/// # Safety
/// `callbacks` and `state` must be non-null, and the functions valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_radar_extend(
    callbacks: *const ZhEngineSimCallbacks,
    radar_extend_time: f32,
    state: *mut ZhRadarState,
) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut update = module(radar_extend_time, unsafe { *state });
    update.extend_radar(&mut sim, EngineSim::context());
    unsafe { *state = saved(&update) };
}

/// `update`. Returns the sleep time.
///
/// # Safety
/// As [`zh_radar_extend`].
#[no_mangle]
pub unsafe extern "C" fn zh_radar_update(
    callbacks: *const ZhEngineSimCallbacks,
    state: *mut ZhRadarState,
) -> u32 {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut update = module(0.0, unsafe { *state });
    let sleep = update.update(&mut sim, EngineSim::context());
    unsafe { *state = saved(&update) };
    sleep.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{assert_header_number, callbacks, Engine};
    use zh_common::types::UpdateSleepTime;
    use zh_logic::rules::model_state::ModelConditionFlagType as Flag;

    fn state(extend_done_frame: u32, extend_complete: u8, radar_active: u8) -> ZhRadarState {
        ZhRadarState {
            extend_done_frame,
            extend_complete,
            radar_active,
        }
    }

    #[test]
    fn extend_sets_the_done_frame_and_the_condition_only_with_a_drawable() {
        for drawable in [false, true] {
            let mut e = Engine {
                drawable,
                ..Engine::default()
            };
            let cb = callbacks(&mut e);
            let mut s = state(0, 1, 0);
            unsafe { zh_radar_extend(&cb, 30.5, &mut s) };
            assert_eq!(s, state(130, 1, 1));
            let expected = drawable.then_some((Flag::RadarExtending as i32, true));
            assert_eq!(e.conditions, Vec::from_iter(expected));
        }
    }

    #[test]
    fn update_finishes_only_after_the_done_frame() {
        // The stand-in engine is at frame 100.
        let finished = vec![
            (Flag::RadarExtending as i32, false),
            (Flag::RadarUpgraded as i32, true),
        ];
        for (before, after, conditions) in [
            (state(0, 0, 0), state(0, 0, 0), vec![]),
            (state(100, 0, 1), state(100, 0, 1), vec![]),
            (state(99, 1, 1), state(99, 1, 1), vec![]),
            (state(99, 0, 1), state(0, 1, 1), finished),
        ] {
            let mut e = Engine {
                drawable: true,
                ..Engine::default()
            };
            let cb = callbacks(&mut e);
            let mut s = before;
            let sleep = unsafe { zh_radar_update(&cb, &mut s) };
            assert_eq!(sleep, UpdateSleepTime::NONE.0);
            assert_eq!(s, after);
            assert_eq!(e.conditions, conditions);
        }
        assert_header_number("ZH_MODEL_RADAR_EXTENDING", Flag::RadarExtending as i32);
        assert_header_number("ZH_MODEL_RADAR_UPGRADED", Flag::RadarUpgraded as i32);
    }
}
