// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces the constructor, `extendRods` and `update` of PowerPlantUpdate.cpp with
// the ported module (PKT-351). `m_extended` stays in C++, so `crc` and `xfer` are
// unchanged.

#include "zh_ffi_engine_sim.inc"

static_assert(MODELCONDITION_POWER_PLANT_UPGRADING == ZH_MODEL_POWER_PLANT_UPGRADING, "condition numbering differs");
static_assert(MODELCONDITION_POWER_PLANT_UPGRADED == ZH_MODEL_POWER_PLANT_UPGRADED, "condition numbering differs");

PowerPlantUpdate::PowerPlantUpdate( Thing *thing, const ModuleData *moduleData )
												: UpdateModule( thing, moduleData )
{
	m_extended = FALSE;
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
	zh_power_plant_construct(&callbacks);
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault on the one path the replays run: wake next frame, not never.
	bridge.sleep = UPDATE_SLEEP_NONE;
#endif
	if (bridge.wake)
		setWakeFrame(getObject(), UPDATE_SLEEP(bridge.sleep));
}

void PowerPlantUpdate::extendRods( Bool extend )
{
	const PowerPlantUpdateModuleData *modData = getPowerPlantUpdateModuleData();
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
	uint8_t extended = m_extended;
	zh_power_plant_extend_rods(&callbacks, modData->m_rodsExtendTime, extend, &extended);
	m_extended = extended != 0;
	if (bridge.wake)
		setWakeFrame(getObject(), UPDATE_SLEEP(bridge.sleep));
}

UpdateSleepTime PowerPlantUpdate::update()
{
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
	uint8_t extended = m_extended;
	const uint32_t sleep = zh_power_plant_update(&callbacks, &extended);
	m_extended = extended != 0;
	return static_cast<UpdateSleepTime>(sleep);
}
