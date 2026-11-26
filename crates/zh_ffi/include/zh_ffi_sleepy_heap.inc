// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the sleepy-update heap functions of the engine's
// GameLogic.cpp. `scripts/build_rust_engine.py` includes this in a copy of that file,
// in place of the originals. `m_sleepyUpdates` stays the storage, so the code that
// walks it (reset, object destruction, load) is unchanged.

#include "zh_ffi.h"

namespace
{

uint32_t zhSleepyPriority(void* module)
{
	const uint32_t priority = static_cast<UpdateModule*>(module)->friend_getPriority();
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault, used to prove that the replay check fails: the update phase,
	// held in the low two bits, no longer orders modules due in the same frame.
	return priority & ~3u;
#else
	return priority;
#endif
}

void zhSleepySetIndex(void* module, int32_t index)
{
	static_cast<UpdateModule*>(module)->friend_setIndexInLogic(index);
}

const ZhSleepyCallbacks zhSleepyCallbacks = { zhSleepyPriority, zhSleepySetIndex };

void** zhSleepyEntries(std::vector<UpdateModulePtr>& modules)
{
	static_assert(sizeof(UpdateModulePtr) == sizeof(void*), "module handle is not a pointer");
	return reinterpret_cast<void**>(modules.data());
}

} // namespace

void GameLogic::eraseSleepyUpdate(Int i)
{
	DEBUG_ASSERTCRASH(i >= 0 && i < m_sleepyUpdates.size(), ("bad sleepy idx"));
	zh_sleepy_remove(zhSleepyEntries(m_sleepyUpdates), static_cast<Int>(m_sleepyUpdates.size()), i,
		&zhSleepyCallbacks);
	m_sleepyUpdates.pop_back();
}

Int GameLogic::rebalanceParentSleepyUpdate(Int i)
{
	return zh_sleepy_sift_up(zhSleepyEntries(m_sleepyUpdates),
		static_cast<Int>(m_sleepyUpdates.size()), i, &zhSleepyCallbacks);
}

Int GameLogic::rebalanceChildSleepyUpdate(Int i)
{
	return zh_sleepy_sift_down(zhSleepyEntries(m_sleepyUpdates),
		static_cast<Int>(m_sleepyUpdates.size()), i, &zhSleepyCallbacks);
}

void GameLogic::rebalanceSleepyUpdate(Int i)
{
	zh_sleepy_rebalance(zhSleepyEntries(m_sleepyUpdates),
		static_cast<Int>(m_sleepyUpdates.size()), i, &zhSleepyCallbacks);
}

void GameLogic::remakeSleepyUpdate()
{
	zh_sleepy_remake(zhSleepyEntries(m_sleepyUpdates),
		static_cast<Int>(m_sleepyUpdates.size()), &zhSleepyCallbacks);
	validateSleepyUpdate();
}

void GameLogic::pushSleepyUpdate(UpdateModulePtr u)
{
	DEBUG_ASSERTCRASH(u != nullptr, ("You may not pass null for sleepy update info"));
	m_sleepyUpdates.push_back(u);
	zh_sleepy_pushed(zhSleepyEntries(m_sleepyUpdates), static_cast<Int>(m_sleepyUpdates.size()),
		&zhSleepyCallbacks);
}
