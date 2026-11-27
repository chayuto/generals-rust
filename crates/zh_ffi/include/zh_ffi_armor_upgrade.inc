// SPDX-License-Identifier: GPL-3.0-or-later
// Replaces `ArmorUpgrade::upgradeImplementation` with the ported module (PKT-194).
// The upgrade mux stays in C++.

#include "zh_ffi_engine_sim.inc"

static_assert(ARMORSET_PLAYER_UPGRADE == ZH_ARMORSET_PLAYER_UPGRADE, "armor set numbering differs");

void ArmorUpgrade::upgradeImplementation()
{
	Object *obj = getObject();
	if( !obj )
		return;
	ZhSimBridge bridge = { obj, 0, false };
	ZhEngineSimCallbacks callbacks = zhSimCallbacks(&bridge);
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault: the armor upgrade is bought but never applied.
	callbacks.set_armor_set_flag = [](void*, int32_t, uint8_t) {};
#endif
	zh_armor_upgrade_implementation(&callbacks);
	// The ported module has no terrain decal call, so this part of the C++ stays.
	if(isTriggeredBy("Upgrade_AmericaChemicalSuits"))
	{
		obj->getDrawable()->setTerrainDecal(TERRAIN_DECAL_CHEMSUIT);
	}
}
