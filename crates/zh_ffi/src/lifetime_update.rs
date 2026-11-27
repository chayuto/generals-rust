//! Entry points that run the ported `LifetimeUpdate` (`PKT-339`) for an engine object.
//!
//! The C++ module keeps `m_dieFrame` (so `crc` and `xfer` are unchanged). Each
//! call builds the Rust module, runs it against [`EngineSim`], and returns what
//! C++ has to store.

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks};
use zh_logic::modules::update::lifetime_update::{LifetimeUpdate, LifetimeUpdateModuleData};
use zh_logic::modules::{BehaviorModule, UpdateFacet};

fn module(min_frames: u32, max_frames: u32) -> LifetimeUpdate {
    let mut data = LifetimeUpdateModuleData::new(Default::default(), Default::default());
    data.min_frames = min_frames;
    data.max_frames = max_frames;
    LifetimeUpdate::new(data)
}

/// The constructor. Returns the die frame.
///
/// # Safety
/// `callbacks` must be non-null and its functions valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_lifetime_construct(
    callbacks: *const ZhEngineSimCallbacks,
    min_frames: u32,
    max_frames: u32,
) -> u32 {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut update = module(min_frames, max_frames);
    update.on_construct(&mut sim, EngineSim::context());
    update.die_frame
}

/// `setLifetimeRange`. Returns the die frame.
///
/// # Safety
/// As [`zh_lifetime_construct`].
#[no_mangle]
pub unsafe extern "C" fn zh_lifetime_set_range(
    callbacks: *const ZhEngineSimCallbacks,
    min_frames: u32,
    max_frames: u32,
) -> u32 {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut update = module(0, 0);
    update.set_lifetime_range(&mut sim, EngineSim::context(), min_frames, max_frames);
    update.die_frame
}

/// `update`. Returns the sleep time; writes nothing, because the kill re-enters the engine.
///
/// # Safety
/// As [`zh_lifetime_construct`].
#[no_mangle]
pub unsafe extern "C" fn zh_lifetime_update(callbacks: *const ZhEngineSimCallbacks) -> u32 {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    module(0, 0).update(&mut sim, EngineSim::context()).0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{callbacks, Engine};
    use zh_logic::rules::combat_types::{DamageType, DeathType};
    use zh_logic::rules::kind_of::KindOfType;

    #[test]
    fn construct_draws_once_and_returns_the_die_frame() {
        for (hulk, over, expected) in [
            (false, 7, (10, 20)),
            (true, -1, (10, 20)),
            (true, 7, (7, 7)),
        ] {
            let mut e = Engine {
                hulk,
                hulk_override: over,
                ..Engine::default()
            };
            let cb = callbacks(&mut e);
            let die = unsafe { zh_lifetime_construct(&cb, 10, 20) };
            assert_eq!(e.draws, [expected]);
            assert_eq!(e.wakes, [expected.1 as u32]);
            assert_eq!(die, 100 + expected.1 as u32);
        }
    }

    #[test]
    fn set_range_sleeps_at_least_one_frame() {
        let mut e = Engine::default();
        let cb = callbacks(&mut e);
        assert_eq!(unsafe { zh_lifetime_set_range(&cb, 0, 0) }, 101);
        assert_eq!(e.wakes, [1]);
    }

    #[test]
    fn update_kills_as_object_kill_does_and_sleeps_forever() {
        let mut e = Engine::default();
        let cb = callbacks(&mut e);
        let sleep = unsafe { zh_lifetime_update(&cb) };
        assert_eq!(sleep, zh_common::types::UpdateSleepTime::FOREVER.0);
        assert_eq!(
            e.damage,
            [(
                DamageType::Unresistable as i32,
                DeathType::Normal as i32,
                0,
                250.0,
                1
            )]
        );
    }

    /// The `.inc` file asserts these same numbers against the engine's enums.
    #[test]
    fn header_numbers_match_the_rust_enums() {
        let header = include_str!("../include/zh_ffi.h");
        for (name, value) in [
            ("ZH_KINDOF_HULK", KindOfType::Hulk as i32),
            ("ZH_DAMAGE_UNRESISTABLE", DamageType::Unresistable as i32),
            ("ZH_DEATH_NORMAL", DeathType::Normal as i32),
            (
                "ZH_SLEEP_FOREVER",
                zh_common::types::UpdateSleepTime::FOREVER.0 as i32,
            ),
        ] {
            let line = format!("#define {name} {value}");
            assert!(header.lines().any(|l| l == line), "missing: {line}");
        }
    }
}
