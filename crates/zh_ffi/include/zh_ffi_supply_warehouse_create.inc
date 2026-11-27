// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces `SupplyWarehouseCreate::onCreate` with the ported module (PKT-183).

#include "zh_ffi_engine_sim.inc"

void SupplyWarehouseCreate::onCreate()
{
	ZhSimBridge bridge = { getObject(), 0, false };
	ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault: no player ever learns of the warehouse.
	callbacks.register_supply_warehouse = [](void*) {};
#endif
	zh_supply_warehouse_on_create(&callbacks);
}
