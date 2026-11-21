#pragma once

class Energy {
public:
    Bool m_sufficientPower = TRUE;
    Bool hasSufficientPower() const { return m_sufficientPower; }
};
