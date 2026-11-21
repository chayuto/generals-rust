#pragma once
class ParticleSystemTemplate {};
class ParticleSystemManager {
public:
    const ParticleSystemTemplate* findTemplate(const AsciiString&) const { return nullptr; }
};
extern ParticleSystemManager* TheParticleSystemManager;
