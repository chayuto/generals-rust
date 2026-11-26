// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the behaviour of the engine's LifetimeUpdate.cpp.
// `scripts/build_rust_engine.py` includes this in a copy of that file, in place of the
// constructor, `setLifetimeRange`, `calcSleepDelay` and `update`. `m_dieFrame` stays
// in C++, so `crc` and `xfer` are unchanged.

#include "zh_ffi.h"

#include "GameLogic/Damage.h"
#include "GameLogic/Module/BodyModule.h"

static_assert(KINDOF_HULK == ZH_KINDOF_HULK, "kind-of numbering differs");
static_assert(DAMAGE_UNRESISTABLE == ZH_DAMAGE_UNRESISTABLE, "damage numbering differs");
static_assert(DEATH_NORMAL == ZH_DEATH_NORMAL, "death numbering differs");
static_assert(UPDATE_SLEEP_FOREVER == ZH_SLEEP_FOREVER, "sleep numbering differs");

namespace
{

struct ZhLifetimeBridge
{
	Object* object;
	UnsignedInt sleep;  // applied by the member function: setWakeFrame is protected
	bool wake;
};

Object* zhLifetimeObject(void* ctx)
{
	return static_cast<ZhLifetimeBridge*>(ctx)->object;
}

uint32_t zhLifetimeFrame(void*)
{
	return TheGameLogic->getFrame();
}

int32_t zhLifetimeRandom(void*, int32_t lo, int32_t hi)
{
	return GameLogicRandomValue(lo, hi);
}

void zhLifetimeWake(void* ctx, uint32_t sleep)
{
	ZhLifetimeBridge& bridge = *static_cast<ZhLifetimeBridge*>(ctx);
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault, used to prove that the replay check fails: one frame too long.
	++sleep;
#endif
	bridge.sleep = sleep;
	bridge.wake = true;
}

int32_t zhLifetimeHulkOverride(void*)
{
	return TheGameLogic->getHulkMaxLifetimeOverride();
}

uint8_t zhLifetimeIsKindOf(void* ctx, int32_t kindof)
{
	return zhLifetimeObject(ctx)->isKindOf(static_cast<KindOfType>(kindof));
}

float zhLifetimeMaxHealth(void* ctx)
{
	return zhLifetimeObject(ctx)->getBodyModule()->getMaxHealth();
}

void zhLifetimeDamage(void* ctx, int32_t damageType, int32_t deathType, int32_t sourceId,
	float amount, uint8_t kill)
{
	DamageInfo damageInfo;
	damageInfo.in.m_damageType = static_cast<DamageType>(damageType);
	damageInfo.in.m_deathType = static_cast<DeathType>(deathType);
	damageInfo.in.m_sourceID = static_cast<ObjectID>(sourceId);
	damageInfo.in.m_amount = amount;
	damageInfo.in.m_kill = kill != 0;
	zhLifetimeObject(ctx)->attemptDamage(&damageInfo);
}

ZhEngineSimCallbacks zhLifetimeCallbacks(ZhLifetimeBridge* bridge)
{
	const ZhEngineSimCallbacks callbacks = {
		bridge, zhLifetimeFrame, zhLifetimeRandom, zhLifetimeWake, zhLifetimeHulkOverride,
		zhLifetimeIsKindOf, zhLifetimeMaxHealth, zhLifetimeDamage };
	return callbacks;
}

} // namespace

LifetimeUpdate::LifetimeUpdate( Thing *thing, const ModuleData* moduleData ) : UpdateModule( thing, moduleData )
{
	const LifetimeUpdateModuleData* d = getLifetimeUpdateModuleData();
	m_dieFrame = 0;
	ZhLifetimeBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhLifetimeCallbacks(&bridge);
	m_dieFrame = zh_lifetime_construct(&callbacks, d->m_minFrames, d->m_maxFrames);
	if (bridge.wake)
		setWakeFrame(getObject(), UPDATE_SLEEP(bridge.sleep));
}

void LifetimeUpdate::setLifetimeRange( UnsignedInt minFrames, UnsignedInt maxFrames )
{
	ZhLifetimeBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhLifetimeCallbacks(&bridge);
	m_dieFrame = zh_lifetime_set_range(&callbacks, minFrames, maxFrames);
	if (bridge.wake)
		setWakeFrame(getObject(), UPDATE_SLEEP(bridge.sleep));
}

UpdateSleepTime LifetimeUpdate::update()
{
	// Nothing is written here: the kill runs die modules, which see this module.
	ZhLifetimeBridge bridge = { getObject(), 0, false };
	const ZhEngineSimCallbacks callbacks = zhLifetimeCallbacks(&bridge);
	return static_cast<UpdateSleepTime>(zh_lifetime_update(&callbacks));
}
