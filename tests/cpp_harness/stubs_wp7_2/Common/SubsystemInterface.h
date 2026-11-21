#pragma once

class SubsystemInterface {
public:
    virtual ~SubsystemInterface() = default;
    virtual void init() = 0;
    virtual void reset() = 0;
    virtual void update() = 0;
    virtual void postProcessLoad() {}
};
