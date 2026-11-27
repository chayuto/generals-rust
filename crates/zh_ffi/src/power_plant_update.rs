//! Entry points that run the ported `PowerPlantUpdate` (`PKT-351`) for an engine object.
//!
//! The C++ module keeps `m_extended`, so `crc` and `xfer` are unchanged.

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks};
use zh_logic::modules::module::PowerPlantFacet;
use zh_logic::modules::update::power_plant_update::{PowerPlantUpdate, PowerPlantUpdateModuleData};
use zh_logic::modules::{BehaviorModule, UpdateFacet};

fn module(rods_extend_time: u32, extended: u8) -> PowerPlantUpdate {
    let mut data = PowerPlantUpdateModuleData::new(Default::default(), Default::default());
    data.rods_extend_time = rods_extend_time;
    let mut update = PowerPlantUpdate::new(data);
    update.extended = extended;
    update
}

/// The constructor, which puts the module to sleep.
///
/// # Safety
/// `callbacks` must be non-null and its functions valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_power_plant_construct(callbacks: *const ZhEngineSimCallbacks) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    module(0, 0).on_construct(&mut sim, EngineSim::context());
}

/// `extendRods`.
///
/// # Safety
/// As [`zh_power_plant_construct`]; `extended` must be non-null.
#[no_mangle]
pub unsafe extern "C" fn zh_power_plant_extend_rods(
    callbacks: *const ZhEngineSimCallbacks,
    rods_extend_time: u32,
    extend: u8,
    extended: *mut u8,
) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut update = module(rods_extend_time, unsafe { *extended });
    update.extend_rods(&mut sim, EngineSim::context(), extend);
    unsafe { *extended = update.extended };
}

/// `update`. Returns the sleep time.
///
/// # Safety
/// As [`zh_power_plant_extend_rods`].
#[no_mangle]
pub unsafe extern "C" fn zh_power_plant_update(
    callbacks: *const ZhEngineSimCallbacks,
    extended: *mut u8,
) -> u32 {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let mut update = module(0, unsafe { *extended });
    let sleep = update.update(&mut sim, EngineSim::context());
    unsafe { *extended = update.extended };
    sleep.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{assert_header_number, callbacks, Engine};
    use zh_common::types::UpdateSleepTime;
    use zh_logic::rules::model_state::ModelConditionFlagType as Flag;

    const UPGRADING: i32 = Flag::PowerPlantUpgrading as i32;
    const UPGRADED: i32 = Flag::PowerPlantUpgraded as i32;
    const FOREVER: u32 = UpdateSleepTime::FOREVER.0;

    fn engine() -> Engine {
        Engine {
            drawable: true,
            ..Engine::default()
        }
    }

    #[test]
    fn construct_sleeps_forever() {
        let mut e = engine();
        let cb = callbacks(&mut e);
        unsafe { zh_power_plant_construct(&cb) };
        assert_eq!(e.wakes, [FOREVER]);
        assert!(e.conditions.is_empty());
    }

    #[test]
    fn extend_rods_in_every_state() {
        for (extend, before, after, conditions, wakes) in [
            (1, 0, 1, vec![(UPGRADING, true)], vec![45]),
            (1, 1, 1, vec![], vec![]),
            (
                0,
                1,
                0,
                vec![(UPGRADING, false), (UPGRADED, false)],
                vec![FOREVER],
            ),
        ] {
            let mut e = engine();
            let cb = callbacks(&mut e);
            let mut extended = before;
            unsafe { zh_power_plant_extend_rods(&cb, 45, extend, &mut extended) };
            assert_eq!(extended, after);
            assert_eq!(e.conditions, conditions);
            assert_eq!(e.wakes, wakes);
        }
    }

    #[test]
    fn update_finishes_the_extension_and_sleeps_forever() {
        let mut e = engine();
        let cb = callbacks(&mut e);
        let mut extended = 0;
        assert_eq!(
            unsafe { zh_power_plant_update(&cb, &mut extended) },
            FOREVER
        );
        assert_eq!(extended, 1);
        assert_eq!(e.conditions, [(UPGRADING, false), (UPGRADED, true)]);
        assert_header_number("ZH_MODEL_POWER_PLANT_UPGRADING", UPGRADING);
        assert_header_number("ZH_MODEL_POWER_PLANT_UPGRADED", UPGRADED);
    }
}
