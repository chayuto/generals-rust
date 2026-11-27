// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces `ModelConditionUpgrade::upgradeImplementation` with the ported module
// (PKT-223). The upgrade mux stays in C++.

#include "zh_ffi_engine_sim.inc"

void ModelConditionUpgrade::upgradeImplementation()
{
	const ModelConditionUpgradeModuleData *data = getModelConditionUpgradeModuleData();
	ZhSimBridge bridge = { getObject(), 0, false };
	ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault: the condition is never set.
	callbacks.set_model_condition_state = [](void*, int32_t) {};
#endif
	zh_model_condition_upgrade_implementation(&callbacks, data->m_conditionFlag);
}
