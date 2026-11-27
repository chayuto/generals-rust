//! Entry point that runs the ported `ModelConditionUpgrade` (`PKT-223`) for an engine object.
//!
//! Only `upgradeImplementation` runs in Rust; the upgrade mux stays in C++, as in
//! [`crate::armor_upgrade`].

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks};
use zh_logic::modules::upgrade::{ModelConditionUpgrade, ModelConditionUpgradeModuleData};
use zh_logic::rules::model_state::ModelConditionFlagType;

/// `upgradeImplementation`. `condition_flag` is the module data's `ConditionFlag`;
/// a number the Rust enum lacks aborts rather than becoming "no flag".
///
/// # Safety
/// `callbacks` must be non-null and its functions valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_model_condition_upgrade_implementation(
    callbacks: *const ZhEngineSimCallbacks,
    condition_flag: i32,
) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let flag = ModelConditionFlagType::from_i32(condition_flag);
    assert_eq!(flag as i32, condition_flag, "unknown model condition flag");
    let data = ModelConditionUpgradeModuleData {
        condition_flag: flag,
        ..Default::default()
    };
    ModelConditionUpgrade::create(&data, EngineSim::context())
        .as_upgrade_mut()
        .expect("ModelConditionUpgrade has an upgrade facet")
        .give_self_upgrade(&mut sim, EngineSim::context());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{callbacks, Engine};

    #[test]
    fn implementation_sets_the_configured_flag_unless_it_is_invalid() {
        let configured = ModelConditionFlagType::RadarUpgraded as i32;
        for (flag, expected) in [(configured, vec![(configured, true)]), (-1, vec![])] {
            let mut e = Engine::default();
            let cb = callbacks(&mut e);
            unsafe { zh_model_condition_upgrade_implementation(&cb, flag) };
            assert_eq!(e.conditions, expected);
        }
    }
}
