// SPDX-License-Identifier: GPL-3.0-or-later
// Rust-backed replacements for the behaviour of the engine's ExperienceTracker.cpp.
// `scripts/build_rust_engine.py` includes this in a copy of that file, in place of the
// original functions. The members stay in C++, so `crc` and `xfer` are unchanged.

#include "zh_ffi.h"

#if RETAIL_COMPATIBLE_CRC
#error "libzh_ffi is built without the retail-crc feature; its tracker would differ"
#endif

namespace
{

struct ZhExpBridge
{
	Object* parent;
	VeterancyLevel* level;     // the tracker's members, written before the engine is told
	Int* experience;
	ZhExpTrackerState state;
	bool notified;
};

uint8_t zhExpTrainable(void* ctx)
{
	return static_cast<ZhExpBridge*>(ctx)->parent->getTemplate()->isTrainable();
}

int32_t zhExpRequired(void* ctx, int32_t level)
{
	return static_cast<ZhExpBridge*>(ctx)->parent->getTemplate()->getExperienceRequired(level);
}

int32_t zhExpValue(void* ctx, int32_t level)
{
	return static_cast<ZhExpBridge*>(ctx)->parent->getTemplate()->getExperienceValue(level);
}

int32_t zhExpRelationship(void* ctx, const void* killer)
{
	return static_cast<const Object*>(killer)->getRelationship(
		static_cast<ZhExpBridge*>(ctx)->parent);
}

void zhExpStore(ZhExpBridge& bridge)
{
	*bridge.level = static_cast<VeterancyLevel>(bridge.state.current_level);
	*bridge.experience = bridge.state.current_experience;
}

void zhExpLevelChanged(void* ctx, int32_t oldLevel, int32_t newLevel, uint8_t feedback)
{
	ZhExpBridge& bridge = *static_cast<ZhExpBridge*>(ctx);
	zhExpStore(bridge);
	bridge.notified = true;
	bridge.parent->onVeterancyLevelChanged(static_cast<VeterancyLevel>(oldLevel),
		static_cast<VeterancyLevel>(newLevel), feedback != 0);
}

uint8_t zhExpForwardAdd(void*, int32_t sinkId, int32_t gain, uint8_t canScale)
{
	Object* sink = TheGameLogic->findObjectByID(static_cast<ObjectID>(sinkId));
	if (!sink)
		return 0;
	sink->getExperienceTracker()->addExperiencePoints(gain, canScale != 0);
	return 1;
}

uint8_t zhExpForwardSet(void*, int32_t sinkId, int32_t experience, uint8_t feedback)
{
	Object* sink = TheGameLogic->findObjectByID(static_cast<ObjectID>(sinkId));
	if (!sink)
		return 0;
	sink->getExperienceTracker()->setExperienceAndLevel(experience, feedback != 0);
	return 1;
}

ZhExpTrackerParentCallbacks zhExpCallbacks(ZhExpBridge* bridge)
{
	const ZhExpTrackerParentCallbacks callbacks = {
		bridge, zhExpTrainable, zhExpRequired, zhExpValue, zhExpRelationship,
		zhExpLevelChanged, zhExpForwardAdd, zhExpForwardSet };
	return callbacks;
}

// Runs one mutating tracker operation and keeps the members in step. Nothing is
// written after the engine has been told, or when the tracker did not change.
template <typename Op>
void zhExpRun(ZhExpBridge& bridge, const Op& op)
{
	const ZhExpTrackerState before = bridge.state;
	const ZhExpTrackerParentCallbacks callbacks = zhExpCallbacks(&bridge);
	op(&bridge.state, &callbacks);
	if (!bridge.notified && (bridge.state.current_level != before.current_level
		|| bridge.state.current_experience != before.current_experience))
	{
		zhExpStore(bridge);
	}
}

} // namespace

#define ZH_EXP_BRIDGE(bridge) \
	ZhExpBridge bridge = { m_parent, &m_currentLevel, &m_currentExperience, \
		{ m_currentLevel, m_currentExperience, static_cast<int32_t>(m_experienceSink), \
		  m_experienceScalar, static_cast<uint8_t>(m_isTrainable) }, false }

Int ExperienceTracker::getExperienceValue( const Object* killer ) const
{
	ZhExpBridge bridge = { m_parent, nullptr, nullptr,
		{ m_currentLevel, m_currentExperience, static_cast<int32_t>(m_experienceSink),
		  m_experienceScalar, static_cast<uint8_t>(m_isTrainable) }, false };
	const ZhExpTrackerParentCallbacks callbacks = zhExpCallbacks(&bridge);
	return zh_exp_tracker_get_experience_value(&bridge.state, &callbacks, killer);
}

void ExperienceTracker::setMinVeterancyLevel( VeterancyLevel newLevel, Bool provideFeedback )
{
	ZH_EXP_BRIDGE(bridge);
	zhExpRun(bridge, [&](ZhExpTrackerState* s, const ZhExpTrackerParentCallbacks* cb) {
		zh_exp_tracker_set_min_veterancy_level(s, cb, newLevel, provideFeedback);
	});
}

void ExperienceTracker::setVeterancyLevel( VeterancyLevel newLevel, Bool provideFeedback )
{
	ZH_EXP_BRIDGE(bridge);
	zhExpRun(bridge, [&](ZhExpTrackerState* s, const ZhExpTrackerParentCallbacks* cb) {
		zh_exp_tracker_set_veterancy_level(s, cb, newLevel, provideFeedback);
	});
}

Bool ExperienceTracker::gainExpForLevel(Int levelsToGain, Bool canScaleForBonus)
{
	ZH_EXP_BRIDGE(bridge);
	Bool gained = false;
	zhExpRun(bridge, [&](ZhExpTrackerState* s, const ZhExpTrackerParentCallbacks* cb) {
		gained = zh_exp_tracker_gain_exp_for_level(s, cb, levelsToGain, canScaleForBonus) != 0;
	});
	return gained;
}

Bool ExperienceTracker::canGainExpForLevel(Int levelsToGain) const
{
	const ZhExpTrackerState state = { m_currentLevel, m_currentExperience,
		static_cast<int32_t>(m_experienceSink), m_experienceScalar,
		static_cast<uint8_t>(m_isTrainable) };
	return zh_exp_tracker_can_gain_exp_for_level(&state, levelsToGain) != 0;
}

void ExperienceTracker::addExperiencePoints( Int experienceGain, Bool canScaleForBonus)
{
#ifdef ZH_SEAM_MUTATE
	// Deliberate fault, used to prove that the replay check fails: one point too many.
	++experienceGain;
#endif
	ZH_EXP_BRIDGE(bridge);
	zhExpRun(bridge, [&](ZhExpTrackerState* s, const ZhExpTrackerParentCallbacks* cb) {
		zh_exp_tracker_add_experience_points(s, cb, experienceGain, canScaleForBonus);
	});
}

void ExperienceTracker::setExperienceAndLevel( Int experienceIn, Bool provideFeedback )
{
	ZH_EXP_BRIDGE(bridge);
	zhExpRun(bridge, [&](ZhExpTrackerState* s, const ZhExpTrackerParentCallbacks* cb) {
		zh_exp_tracker_set_experience_and_level(s, cb, experienceIn, provideFeedback);
	});
}

#undef ZH_EXP_BRIDGE
