//! Entry point that runs the ported `ArmorUpgrade` (`PKT-194`) for an engine object.
//!
//! Only `upgradeImplementation` runs in Rust. The upgrade mux (conditions, FX,
//! removals, the executed flag, `crc` and `xfer`) stays in C++, so the Rust module
//! gets empty mux data and `give_self_upgrade` reduces to its implementation.
//!
//! The ported module does not issue the chemical-suit terrain decal, so
//! `zh_ffi_armor_upgrade.inc` keeps that call in C++.

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks};
use zh_logic::modules::upgrade::upgrade_module::UpgradeModuleData;
use zh_logic::modules::upgrade::ArmorUpgrade;

/// `upgradeImplementation`.
///
/// # Safety
/// `callbacks` must be non-null and its functions valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_armor_upgrade_implementation(callbacks: *const ZhEngineSimCallbacks) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    ArmorUpgrade::create(&UpgradeModuleData::default(), EngineSim::context())
        .as_upgrade_mut()
        .expect("ArmorUpgrade has an upgrade facet")
        .give_self_upgrade(&mut sim, EngineSim::context());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{assert_header_number, callbacks, Engine};
    use zh_logic::rules::combat_types::ArmorSetType;

    #[test]
    fn implementation_sets_the_player_upgrade_armor_flag_and_nothing_else() {
        let mut e = Engine::default();
        let cb = callbacks(&mut e);
        unsafe { zh_armor_upgrade_implementation(&cb) };
        assert_eq!(e.armor, [(ArmorSetType::PlayerUpgrade as i32, 1)]);
        assert!(e.conditions.is_empty() && e.wakes.is_empty() && e.warehouses == 0);
        assert_header_number(
            "ZH_ARMORSET_PLAYER_UPGRADE",
            ArmorSetType::PlayerUpgrade as i32,
        );
    }
}
