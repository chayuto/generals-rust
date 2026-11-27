// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces `exitObjectViaDoor`, `getExitPosition` and `getNaturalRallyPoint` of
// `DefaultProductionExitUpdate` with the ported module (PKT-315). The rally point
// and its `crc` and `xfer` stay in C++.

#include "zh_ffi_engine_sim.inc"

namespace
{

ZhExitData zhExitData(const DefaultProductionExitUpdateModuleData* md)
{
	const ZhExitData data = {
		{ md->m_unitCreatePoint.x, md->m_unitCreatePoint.y, md->m_unitCreatePoint.z },
		{ md->m_naturalRallyPoint.x, md->m_naturalRallyPoint.y, md->m_naturalRallyPoint.z },
	};
	return data;
}

ZhEngineSimCallbacks zhExitCallbacks(ZhSimBridge* bridge)
{
	ZhEngineSimCallbacks callbacks = zhSimCallbacks(bridge);
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault: the producer's facing is read half a radian off.
	callbacks.object_orientation = [](void* ctx) { return zhSimOrientation(ctx) + 0.5f; };
#endif
	return callbacks;
}

} // namespace

void DefaultProductionExitUpdate::exitObjectViaDoor( Object *newObj, ExitDoorType exitDoor )
{
	DEBUG_ASSERTCRASH(exitDoor == DOOR_1, ("multiple exit doors not supported here"));
	if (getObject() == nullptr)
		return;
	const ZhExitData data = zhExitData(getDefaultProductionExitUpdateModuleData());
	const float rally[3] = { m_rallyPoint.x, m_rallyPoint.y, m_rallyPoint.z };
	ZhSimBridge bridge = { getObject(), 0, false, newObj };
	ZhEngineSimCallbacks callbacks = zhExitCallbacks(&bridge);
	zh_exit_object_via_door(&callbacks, &data, m_rallyPointExists ? rally : nullptr);
}

Bool DefaultProductionExitUpdate::getExitPosition( Coord3D& exitPosition ) const
{
	const Object *obj = getObject();
	if (obj == nullptr)
		return FALSE;
	const ZhExitData data = zhExitData(getDefaultProductionExitUpdateModuleData());
	ZhSimBridge bridge = { const_cast<Object*>(obj), 0, false };
	ZhEngineSimCallbacks callbacks = zhExitCallbacks(&bridge);
	float out[3] = { 0.0f, 0.0f, 0.0f };
	if (!zh_exit_position(&callbacks, &data, out))
		return FALSE;
	exitPosition.x = out[0];
	exitPosition.y = out[1];
	exitPosition.z = out[2];
	return TRUE;
}

Bool DefaultProductionExitUpdate::getNaturalRallyPoint( Coord3D& rallyPoint, Bool offset ) const
{
	const Object *obj = getObject();
	if (obj == nullptr)
		return FALSE;
	const ZhExitData data = zhExitData(getDefaultProductionExitUpdateModuleData());
	ZhSimBridge bridge = { const_cast<Object*>(obj), 0, false };
	ZhEngineSimCallbacks callbacks = zhExitCallbacks(&bridge);
	float out[3] = { 0.0f, 0.0f, 0.0f };
	if (!zh_exit_natural_rally_point(&callbacks, &data, offset, out))
		return FALSE;
	rallyPoint.x = out[0];
	rallyPoint.y = out[1];
	rallyPoint.z = out[2];
	return TRUE;
}
