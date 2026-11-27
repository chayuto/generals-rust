// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces `RadarUpdate::extendRadar` and `update` with the ported module
// (PKT-358). The members stay in C++, so `crc` and `xfer` are unchanged.

#include "zh_ffi_engine_sim.inc"

static_assert(MODELCONDITION_RADAR_EXTENDING == ZH_MODEL_RADAR_EXTENDING, "condition numbering differs");
static_assert(MODELCONDITION_RADAR_UPGRADED == ZH_MODEL_RADAR_UPGRADED, "condition numbering differs");

void RadarUpdate::extendRadar()
{
	const RadarUpdateModuleData *modData = getRadarUpdateModuleData();
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
	ZhRadarState state = { m_extendDoneFrame, m_extendComplete, m_radarActive };
	zh_radar_extend(&callbacks, modData->m_radarExtendTime, &state);
	m_extendDoneFrame = state.extend_done_frame;
	m_extendComplete = state.extend_complete != 0;
	m_radarActive = state.radar_active != 0;
}

UpdateSleepTime RadarUpdate::update()
{
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
	ZhRadarState state = { m_extendDoneFrame, m_extendComplete, m_radarActive };
	const uint32_t sleep = zh_radar_update(&callbacks, &state);
	m_extendDoneFrame = state.extend_done_frame;
	m_extendComplete = state.extend_complete != 0;
	m_radarActive = state.radar_active != 0;
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault: the module never runs again. A fault in the members or the
	// model conditions passes the replay check (docs/BASELINE.md).
	return UPDATE_SLEEP_FOREVER;
#else
	return static_cast<UpdateSleepTime>(sleep);
#endif
}
