#pragma once
#include "Common/GameMemory.h"

class INI;

enum WeaponBonusConditionType CPP_11(: Int)
{
    WEAPONBONUSCONDITION_INVALID = -1,

    WEAPONBONUSCONDITION_GARRISONED = 0,
    WEAPONBONUSCONDITION_HORDE,
    WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN,
    WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST,
    WEAPONBONUSCONDITION_NATIONALISM,
    WEAPONBONUSCONDITION_PLAYER_UPGRADE,
    WEAPONBONUSCONDITION_DRONE_SPOTTING,
#ifdef ALLOW_DEMORALIZE
    WEAPONBONUSCONDITION_DEMORALIZED,
#else
    WEAPONBONUSCONDITION_DEMORALIZED_OBSOLETE,
#endif
    WEAPONBONUSCONDITION_ENTHUSIASTIC,
    WEAPONBONUSCONDITION_VETERAN,
    WEAPONBONUSCONDITION_ELITE,
    WEAPONBONUSCONDITION_HERO,
    WEAPONBONUSCONDITION_BATTLEPLAN_BOMBARDMENT,
    WEAPONBONUSCONDITION_BATTLEPLAN_HOLDTHELINE,
    WEAPONBONUSCONDITION_BATTLEPLAN_SEARCHANDDESTROY,
    WEAPONBONUSCONDITION_SUBLIMINAL,
    WEAPONBONUSCONDITION_SOLO_HUMAN_EASY,
    WEAPONBONUSCONDITION_SOLO_HUMAN_NORMAL,
    WEAPONBONUSCONDITION_SOLO_HUMAN_HARD,
    WEAPONBONUSCONDITION_SOLO_AI_EASY,
    WEAPONBONUSCONDITION_SOLO_AI_NORMAL,
    WEAPONBONUSCONDITION_SOLO_AI_HARD,
    WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE,
    WEAPONBONUSCONDITION_FANATICISM,
    WEAPONBONUSCONDITION_FRENZY_ONE,
    WEAPONBONUSCONDITION_FRENZY_TWO,
    WEAPONBONUSCONDITION_FRENZY_THREE,

    WEAPONBONUSCONDITION_COUNT
};

#ifdef DEFINE_WEAPONBONUSCONDITION_NAMES
static const char* const TheWeaponBonusNames[] =
{
    "GARRISONED",
    "HORDE",
    "CONTINUOUS_FIRE_MEAN",
    "CONTINUOUS_FIRE_FAST",
    "NATIONALISM",
    "PLAYER_UPGRADE",
    "DRONE_SPOTTING",
#ifdef ALLOW_DEMORALIZE
    "DEMORALIZED",
#else
    "DEMORALIZED_OBSOLETE",
#endif
    "ENTHUSIASTIC",
    "VETERAN",
    "ELITE",
    "HERO",
    "BATTLEPLAN_BOMBARDMENT",
    "BATTLEPLAN_HOLDTHELINE",
    "BATTLEPLAN_SEARCHANDDESTROY",
    "SUBLIMINAL",
    "SOLO_HUMAN_EASY",
    "SOLO_HUMAN_NORMAL",
    "SOLO_HUMAN_HARD",
    "SOLO_AI_EASY",
    "SOLO_AI_NORMAL",
    "SOLO_AI_HARD",
    "TARGET_FAERIE_FIRE",
    "FANATICISM",
    "FRENZY_ONE",
    "FRENZY_TWO",
    "FRENZY_THREE",
    nullptr
};
static_assert(ARRAY_SIZE(TheWeaponBonusNames) == WEAPONBONUSCONDITION_COUNT + 1, "Incorrect array size");
#endif

typedef UnsignedInt WeaponBonusConditionFlags;

class WeaponBonus
{
public:
    enum Field
    {
        DAMAGE = 0,
        RADIUS,
        RANGE,
        RATE_OF_FIRE,
        PRE_ATTACK,
        FIELD_COUNT
    };

    WeaponBonus() { clear(); }

    void clear()
    {
        for (int i = 0; i < FIELD_COUNT; ++i)
            m_field[i] = 1.0f;
    }

    Real getField(Field f) const { return m_field[f]; }
    void setField(Field f, Real v) { m_field[f] = v; }

    void appendBonuses(WeaponBonus& bonus) const;

private:
    Real m_field[FIELD_COUNT];
};

#ifdef DEFINE_WEAPONBONUSFIELD_NAMES
static const char* const TheWeaponBonusFieldNames[] =
{
    "DAMAGE",
    "RADIUS",
    "RANGE",
    "RATE_OF_FIRE",
    "PRE_ATTACK",
    nullptr
};
static_assert(ARRAY_SIZE(TheWeaponBonusFieldNames) == WeaponBonus::FIELD_COUNT + 1, "Incorrect array size");
#endif

class WeaponBonusSet : public MemoryPoolObject
{
    MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(WeaponBonusSet, "WeaponBonusSet")
private:
    WeaponBonus m_bonus[WEAPONBONUSCONDITION_COUNT];

public:
    const WeaponBonus& getBonusEntry(Int i) const { return m_bonus[i]; }
    void appendBonuses(WeaponBonusConditionFlags flags, WeaponBonus& bonus) const;

    void parseWeaponBonusSet(INI* ini);
    static void parseWeaponBonusSet(INI* ini, void* instance, void* store, const void* userData);
    static void parseWeaponBonusSetPtr(INI* ini, void* instance, void* store, const void* userData);
};
EMPTY_DTOR(WeaponBonusSet)

class WeaponTemplate {};
class WeaponStore {
public:
    const WeaponTemplate* findWeaponTemplate(const char*) const { return nullptr; }
};
extern WeaponStore* TheWeaponStore;