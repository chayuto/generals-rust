// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the behaviour of the engine's LifetimeUpdate.cpp.
// `scripts/build_rust_engine.py` includes this in a copy of that file, in place of the
// constructor, `setLifetimeRange`, `calcSleepDelay` and `update`. `m_dieFrame` stays
// in C++, so `crc` and `xfer` are unchanged.

#include "zh_ffi_engine_sim.inc"

static_assert(KINDOF_HULK == ZH_KINDOF_HULK, "kind-of numbering differs");
static_assert(DAMAGE_UNRESISTABLE == ZH_DAMAGE_UNRESISTABLE, "damage numbering differs");
static_assert(DEATH_NORMAL == ZH_DEATH_NORMAL, "death numbering differs");
static_assert(UPDATE_SLEEP_FOREVER == ZH_SLEEP_FOREVER, "sleep numbering differs");

namespace
{

#ifdef ZH_SEAM_MUTATE
// Deliberate fault, used to prove that the replay check fails: one frame too long.
void zhLifetimeLateWake(void* ctx, uint32_t sleep)
{
	zhSimWake(ctx, sleep + 1);
}
#endif

ZhEngineSimCallbacks zhLifetimeCallbacks(ZhSimBridge* bridge)
{
	ZhEngineSimCallbacks callbacks = zhSimCallbacks(bridge);
#ifdef ZH_SEAM_MUTATE
	callbacks.set_wake_frame = zhLifetimeLateWake;
#endif
	return callbacks;
}

} // namespace

LifetimeUpdate::LifetimeUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const LifetimeUpdateModuleData* d = getLifetimeUpdateModuleData();
	m_dieFrame = 0;
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhLifetimeCallbacks(&bridge);
	m_dieFrame = zh_lifetime_construct(&callbacks, d->m_minFrames, d->m_maxFrames);
	if (bridge.wake)
		setWakeFrame(getObject(), UPDATE_SLEEP(bridge.sleep));
}

void LifetimeUpdate::setLifetimeRange( UnsignedInt minFrames, UnsignedInt maxFrames )
{
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhLifetimeCallbacks(&bridge);
	m_dieFrame = zh_lifetime_set_range(&callbacks, minFrames, maxFrames);
	if (bridge.wake)
		setWakeFrame(getObject(), UPDATE_SLEEP(bridge.sleep));
}

UpdateSleepTime LifetimeUpdate::update()
{
	// Nothing is written here: the kill runs die modules, which see this module.
	ZhSimBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhLifetimeCallbacks(&bridge);
	return static_cast<UpdateSleepTime>(zh_lifetime_update(&callbacks));
}
