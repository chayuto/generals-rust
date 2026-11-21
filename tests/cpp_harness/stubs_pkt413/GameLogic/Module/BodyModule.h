#pragma once

enum BodyDamageType CPP_11(: Int) {
    BODY_PRISTINE = 0,
    BODY_DAMAGED = 1,
    BODY_REALLYDAMAGED = 2,
    BODY_RUBBLE = 3,
    BODYDAMAGETYPE_COUNT = 4
};

class BodyModuleInterface {
public:
    BodyDamageType m_damageState = BODY_PRISTINE;
    BodyDamageType getDamageState() const { return m_damageState; }
};
