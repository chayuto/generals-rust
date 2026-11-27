// SPDX-License-Identifier: GPL-3.0-or-later
// The engine side of the module adapter (`src/engine_sim.rs`): one object, seen
// through `ZhEngineSimCallbacks`. Every swapped behavior module includes this in
// its copy of the module's source file.

#include "zh_ffi.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/ResourceGatheringManager.h"
#include "GameLogic/AI.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Damage.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/TerrainLogic.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"

namespace
{

struct ZhSimBridge
{
	Object* object;
	UnsignedInt sleep;  // applied by the member function: setWakeFrame is protected
	bool wake;
	Object* guest;      // the second object of the call, if there is one
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

void zhSimTransform(void* ctx, float* rows)
{
	const Matrix3D* transform = zhSimObject(ctx)->getTransformMatrix();
	for (int row = 0; row < 3; ++row)
		for (int column = 0; column < 4; ++column)
			rows[row * 4 + column] = (*transform)[row][column];
}

float zhSimOrientation(void* ctx)
{
	return zhSimObject(ctx)->getOrientation();
}

int32_t zhSimLayer(void* ctx)
{
	return zhSimObject(ctx)->getLayer();
}

float zhSimLayerHeight(void*, float x, float y, int32_t layer)
{
	return TheTerrainLogic
		? TheTerrainLogic->getLayerHeight(x, y, static_cast<PathfindLayerEnum>(layer))
		: 0.0f;
}

Object* zhSimGuest(void* ctx)
{
	return static_cast<ZhSimBridge*>(ctx)->guest;
}

void zhSimSetPosition(void* ctx, float x, float y, float z)
{
	Coord3D position;
	position.x = x;
	position.y = y;
	position.z = z;
	zhSimGuest(ctx)->setPosition(&position);
}

void zhSimSetOrientation(void* ctx, float angle)
{
	zhSimGuest(ctx)->setOrientation(angle);
}

void zhSimSetLayer(void* ctx, int32_t layer)
{
	zhSimGuest(ctx)->setLayer(static_cast<PathfindLayerEnum>(layer));
}

void zhSimPathfinderAdd(void* ctx)
{
	TheAI->pathfinder()->addObjectToPathfindMap(zhSimGuest(ctx));
}

uint8_t zhSimHasAI(void* ctx)
{
	return zhSimGuest(ctx)->getAIUpdateInterface() != nullptr;
}

uint8_t zhSimGroundMovement(void* ctx)
{
	return zhSimGuest(ctx)->getAIUpdateInterface()->isDoingGroundMovement();
}

uint8_t zhSimAdjustDestination(void* ctx, float* xyz)
{
	Object* guest = zhSimGuest(ctx);
	Coord3D destination;
	destination.x = xyz[0];
	destination.y = xyz[1];
	destination.z = xyz[2];
	const Bool found = TheAI->pathfinder()->adjustDestination(
		guest, guest->getAIUpdateInterface()->getLocomotorSet(), &destination);
	xyz[0] = destination.x;
	xyz[1] = destination.y;
	xyz[2] = destination.z;
	return found;
}

void zhSimFollowExitPath(void* ctx, const float* xyz, int32_t count)
{
	std::vector<Coord3D> path(count);
	for (int32_t i = 0; i < count; ++i)
	{
		path[i].x = xyz[i * 3];
		path[i].y = xyz[i * 3 + 1];
		path[i].z = xyz[i * 3 + 2];
	}
	zhSimGuest(ctx)->getAIUpdateInterface()->aiFollowExitProductionPath(
		&path, zhSimObject(ctx), CMD_FROM_AI);
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
	callbacks.object_transform = zhSimTransform;
	callbacks.object_orientation = zhSimOrientation;
	callbacks.object_layer = zhSimLayer;
	callbacks.get_layer_height = zhSimLayerHeight;
	callbacks.set_object_position = zhSimSetPosition;
	callbacks.set_object_orientation = zhSimSetOrientation;
	callbacks.set_object_layer = zhSimSetLayer;
	callbacks.pathfinder_add_object = zhSimPathfinderAdd;
	callbacks.has_ai = zhSimHasAI;
	callbacks.ai_is_doing_ground_movement = zhSimGroundMovement;
	callbacks.pathfinder_adjust_destination = zhSimAdjustDestination;
	callbacks.ai_follow_exit_production_path = zhSimFollowExitPath;
	return callbacks;
}

} // namespace
