// SPDX-License-Identifier: GPL-3.0-or-later
// The engine side of the module adapter (`src/engine_sim.rs`): one object, seen
// through `ZhEngineSimCallbacks`. Every swapped behavior module includes this in
// its copy of the module's source file.

#include "zh_ffi.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ResourceGatheringManager.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/BodyModule.h"

namespace
{

struct ZhSimBridge
{
	Object* object;
	UnsignedInt sleep;  // applied by the member function: setWakeFrame is protected
	bool wake;
};

Object* zhSimObject(void* ctx)
{
	return static_cast<ZhSimBridge*>(ctx)->object;
}

uint32_t zhSimFrame(void*)
{
	return TheGameLogic->getFrame();
}

int32_t zhSimRandom(void*, int32_t lo, int32_t hi)
{
	return GameLogicRandomValue(lo, hi);
}

void zhSimWake(void* ctx, uint32_t sleep)
{
	ZhSimBridge& bridge = *static_cast<ZhSimBridge*>(ctx);
	bridge.sleep = sleep;
	bridge.wake = true;
}

int32_t zhSimHulkOverride(void*)
{
	return TheGameLogic->getHulkMaxLifetimeOverride();
}

uint8_t zhSimIsKindOf(void* ctx, int32_t kindof)
{
	return zhSimObject(ctx)->isKindOf(static_cast<KindOfType>(kindof));
}

float zhSimMaxHealth(void* ctx)
{
	return zhSimObject(ctx)->getBodyModule()->getMaxHealth();
}

void zhSimDamage(void* ctx, int32_t damageType, int32_t deathType, int32_t sourceId,
	float amount, uint8_t kill)
{
	DamageInfo damageInfo;
	damageInfo.in.m_damageType = static_cast<DamageType>(damageType);
	damageInfo.in.m_deathType = static_cast<DeathType>(deathType);
	damageInfo.in.m_sourceID = static_cast<ObjectID>(sourceId);
	damageInfo.in.m_amount = amount;
	damageInfo.in.m_kill = kill != 0;
	zhSimObject(ctx)->attemptDamage(&damageInfo);
}

uint8_t zhSimHasDrawable(void* ctx)
{
	return zhSimObject(ctx)->getDrawable() != nullptr;
}

// Both do nothing without a drawable, as the C++ modules' own checks do.
void zhSimSetCondition(void* ctx, int32_t flag)
{
	zhSimObject(ctx)->setModelConditionState(static_cast<ModelConditionFlagType>(flag));
}

void zhSimClearCondition(void* ctx, int32_t flag)
{
	zhSimObject(ctx)->clearModelConditionState(static_cast<ModelConditionFlagType>(flag));
}

void zhSimArmorSetFlag(void* ctx, int32_t flag, uint8_t set)
{
	BodyModuleInterface* body = zhSimObject(ctx)->getBodyModule();
	if (body == nullptr)
		return;
	if (set)
		body->setArmorSetFlag(static_cast<ArmorSetType>(flag));
	else
		body->clearArmorSetFlag(static_cast<ArmorSetType>(flag));
}

// The walk in `SupplyWarehouseCreate::onCreate`.
void zhSimRegisterSupplyWarehouse(void* ctx)
{
	if (ThePlayerList == nullptr)
		return;
	for (Int playerIndex = ThePlayerList->getPlayerCount() - 1; playerIndex >= 0; playerIndex--)
	{
		Player* currentPlayer = ThePlayerList->getNthPlayer(playerIndex);
		if (currentPlayer == nullptr)
			continue;
		ResourceGatheringManager* manager = currentPlayer->getResourceGatheringManager();
		if (manager == nullptr)
			continue;
		manager->addSupplyWarehouse(zhSimObject(ctx));
	}
}

ZhEngineSimCallbacks zhSimCallbacks(ZhSimBridge* bridge)
{
	ZhEngineSimCallbacks callbacks = {};
	callbacks.ctx = bridge;
	callbacks.frame = zhSimFrame;
	callbacks.rng_logic_int = zhSimRandom;
	callbacks.set_wake_frame = zhSimWake;
	callbacks.hulk_max_lifetime_override = zhSimHulkOverride;
	callbacks.object_is_kind_of = zhSimIsKindOf;
	callbacks.object_max_health = zhSimMaxHealth;
	callbacks.attempt_damage = zhSimDamage;
	callbacks.object_has_drawable = zhSimHasDrawable;
	callbacks.set_model_condition_state = zhSimSetCondition;
	callbacks.clear_model_condition_state = zhSimClearCondition;
	callbacks.set_armor_set_flag = zhSimArmorSetFlag;
	callbacks.register_supply_warehouse = zhSimRegisterSupplyWarehouse;
	return callbacks;
}

} // namespace
