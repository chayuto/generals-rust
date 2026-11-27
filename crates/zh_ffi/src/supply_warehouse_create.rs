//! Entry point that runs the ported `SupplyWarehouseCreate` (`PKT-183`) for an engine object.
//!
//! The `CreateModule` base, with its one saved flag, stays in C++. The walk over
//! the players' resource managers is the engine's answer to
//! `register_supply_warehouse`.

use crate::engine_sim::{EngineSim, ZhEngineSimCallbacks};
use zh_logic::modules::create::create_module::CreateModuleData;
use zh_logic::modules::create::supply_warehouse_create::SupplyWarehouseCreate;

/// `onCreate`.
///
/// # Safety
/// `callbacks` must be non-null and its functions valid for the call.
#[no_mangle]
pub unsafe extern "C" fn zh_supply_warehouse_on_create(callbacks: *const ZhEngineSimCallbacks) {
    let mut sim = EngineSim::new(unsafe { &*callbacks });
    let data = CreateModuleData {
        tag_key: Default::default(),
        module_name_key: Default::default(),
    };
    SupplyWarehouseCreate::create(&data, EngineSim::context())
        .as_create_mut()
        .expect("SupplyWarehouseCreate has a create facet")
        .on_create(&mut sim, EngineSim::context());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_engine::{callbacks, Engine};

    #[test]
    fn on_create_registers_the_warehouse_once() {
        let mut e = Engine::default();
        let cb = callbacks(&mut e);
        unsafe { zh_supply_warehouse_on_create(&cb) };
        assert_eq!(e.warehouses, 1);
        assert!(e.conditions.is_empty() && e.armor.is_empty() && e.wakes.is_empty());
    }
}
