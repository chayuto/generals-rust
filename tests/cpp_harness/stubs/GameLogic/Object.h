#pragma once
#include "PreRTS.h"
#include "Common/ThingTemplate.h"

class ExperienceTracker;

class Object {
public:
    ObjectID m_id = INVALID_ID;
    ThingTemplate m_template;
    Relationship m_relWithTarget = ENEMIES;
    ExperienceTracker* m_tracker = nullptr;

    Bool m_levelChangedCalled = FALSE;
    VeterancyLevel m_lastOldLevel = LEVEL_REGULAR;
    VeterancyLevel m_lastNewLevel = LEVEL_REGULAR;
    Bool m_lastProvideFeedback = FALSE;
    Int m_levelChangedCount = 0;

    const ThingTemplate* getTemplate() const { return &m_template; }
    Relationship getRelationship(const Object* /*that*/) const { return m_relWithTarget; }
    ExperienceTracker* getExperienceTracker() { return m_tracker; }

    void onVeterancyLevelChanged(VeterancyLevel oldLevel, VeterancyLevel newLevel, Bool provideFeedback = TRUE) {
        m_levelChangedCalled = TRUE;
        m_lastOldLevel = oldLevel;
        m_lastNewLevel = newLevel;
        m_lastProvideFeedback = provideFeedback;
        m_levelChangedCount++;
    }
};
