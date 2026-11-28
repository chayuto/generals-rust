// Differential C++ oracle for PKT-391: ObjectCreationList runtime (`create` + 5 nugget types).
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ocl_runtime.cpp --opt O0 -o <dir>/gen_ocl_runtime_o0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ocl_runtime.cpp --opt O2 -o <dir>/gen_ocl_runtime_o2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_ocl_runtime.cpp --opt O2 --sanitize -o <dir>/gen_ocl_runtime_ubsan
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy <dir>/gen_ocl_runtime_o2 docs/golden_vectors/OclRuntime.json

#include "oracle_wp51_world.h"

#include "Common/ModuleFactory.h"
#include "Common/RandomValue.h"
#include "Common/DrawModule.h"
#include "GameClient/Shadow.h"
#include "GameClient/TerrainVisual.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/Module/ActiveBody.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/DeliverPayloadAIUpdate.h"
#include "GameLogic/Module/FloatUpdate.h"
#include "GameLogic/Module/LifetimeUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/RadiusDecalUpdate.h"
#include "GameLogic/Module/SlavedUpdate.h"
#include "GameLogic/Module/SpecialPowerCompletionDie.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/WeaponSet.h"

#include <cstdio>
#include <cstring>
#include <string>
#include <vector>

template <class Tag, typename Tag::type M>
struct OclRob {
    friend typename Tag::type access_member(Tag) { return M; }
};

struct OclPhysicsYawTag {
    using type = Real PhysicsBehavior::*;
    friend type access_member(OclPhysicsYawTag);
};
template struct OclRob<OclPhysicsYawTag, &PhysicsBehavior::m_yawRate>;

struct OclPhysicsRollTag {
    using type = Real PhysicsBehavior::*;
    friend type access_member(OclPhysicsRollTag);
};
template struct OclRob<OclPhysicsRollTag, &PhysicsBehavior::m_rollRate>;

struct OclPhysicsPitchTag {
    using type = Real PhysicsBehavior::*;
    friend type access_member(OclPhysicsPitchTag);
};
template struct OclRob<OclPhysicsPitchTag, &PhysicsBehavior::m_pitchRate>;

struct OclPhysicsBounceTag {
    using type = Real PhysicsBehavior::*;
    friend type access_member(OclPhysicsBounceTag);
};
template struct OclRob<OclPhysicsBounceTag, &PhysicsBehavior::m_extraBounciness>;

struct OclPhysicsFrictionTag {
    using type = Real PhysicsBehavior::*;
    friend type access_member(OclPhysicsFrictionTag);
};
template struct OclRob<OclPhysicsFrictionTag, &PhysicsBehavior::m_extraFriction>;

struct OclPhysicsIgnoreTag {
    using type = ObjectID PhysicsBehavior::*;
    friend type access_member(OclPhysicsIgnoreTag);
};
template struct OclRob<OclPhysicsIgnoreTag, &PhysicsBehavior::m_ignoreCollisionsWith>;

struct OclPhysicsAccelTag {
    using type = Coord3D PhysicsBehavior::*;
    friend type access_member(OclPhysicsAccelTag);
};
template struct OclRob<OclPhysicsAccelTag, &PhysicsBehavior::m_accel>;

struct OclFloatEnabledTag {
    using type = Bool FloatUpdate::*;
    friend type access_member(OclFloatEnabledTag);
};
template struct OclRob<OclFloatEnabledTag, &FloatUpdate::m_enabled>;

struct OclRadiusKillTag {
    using type = Bool RadiusDecalUpdate::*;
    friend type access_member(OclRadiusKillTag);
};
template struct OclRob<OclRadiusKillTag, &RadiusDecalUpdate::m_killWhenNoLongerAttacking>;

struct OclSpdCreatorTag {
    using type = ObjectID SpecialPowerCompletionDie::*;
    friend type access_member(OclSpdCreatorTag);
};
template struct OclRob<OclSpdCreatorTag, &SpecialPowerCompletionDie::m_creatorID>;

struct OclScriptStatusTag {
    using type = UnsignedByte Object::*;
    friend type access_member(OclScriptStatusTag);
};
template struct OclRob<OclScriptStatusTag, &Object::m_scriptStatus>;

class OracleW3DDebrisDraw : public DrawModule, public DebrisDrawInterface {
    MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE(OracleW3DDebrisDraw, "W3DDebrisDraw")
public:
    OracleW3DDebrisDraw(Thing* thing, const ModuleData* md) : DrawModule(thing, md) {}
    NameKeyType getModuleNameKey() const override {
        static NameKeyType k = NAMEKEY("W3DDebrisDraw");
        return k;
    }
    void doDrawModule(const Matrix3D*) override {}
    void setShadowsEnabled(Bool) override {}
    void releaseShadows() override {}
    void allocateShadows() override {}
    void setFullyObscuredByShroud(Bool) override {}
    void reactToTransformChange(const Matrix3D*, const Coord3D*, Real) override {}
    void reactToGeometryChange() override {}
    DebrisDrawInterface* getDebrisDrawInterface() override { return this; }
    const DebrisDrawInterface* getDebrisDrawInterface() const override { return this; }
    void setModelName(AsciiString, Color, ShadowType) override {}
    void setAnimNames(AsciiString, AsciiString, AsciiString, const FXList*) override {}
    static Module* create_inst(Thing* t, const ModuleData* md) {
        return newInstance(OracleW3DDebrisDraw)(t, md);
    }
    static ModuleData* create_data(INI* ini) {
        ModuleData* md = NEW ModuleData();
        MultiIniFieldParse p;
        ini->initFromINIMulti(md, p);
        return md;
    }
};
EMPTY_DTOR(OracleW3DDebrisDraw)

class NullProjectedShadowManager : public ProjectedShadowManager {
public:
    Shadow* addDecal(RenderObjClass*, Shadow::ShadowTypeInfo*) override { return nullptr; }
    Shadow* addDecal(Shadow::ShadowTypeInfo*) override { return nullptr; }
};

class NullTerrainVisual : public TerrainVisual {
public:
    void getTerrainColorAt(Real, Real, RGBColor*) override {}
    TerrainType* getTerrainTile(Real, Real) override { return nullptr; }
    void enableWaterGrid(Bool) override {}
    void setWaterGridHeightClamps(const WaterHandle*, Real, Real) override {}
    void setWaterAttenuationFactors(const WaterHandle*, Real, Real, Real, Real) override {}
    void setWaterTransform(const WaterHandle*, Real, Real, Real, Real) override {}
    void setWaterTransform(const Matrix3D*) override {}
    void getWaterTransform(const WaterHandle*, Matrix3D*) override {}
    void setWaterGridResolution(const WaterHandle*, Real, Real, Real) override {}
    void getWaterGridResolution(const WaterHandle*, Real*, Real*, Real*) override {}
    void changeWaterHeight(Real, Real, Real) override {}
    void addWaterVelocity(Real, Real, Real, Real) override {}
    Bool getWaterGridHeight(Real, Real, Real*) override { return FALSE; }
    void setTerrainTracksDetail() override {}
    void setShoreLineDetail() override {}
    void addFactionBib(Object*, Bool, Real = 0) override {}
    void removeFactionBib(Object*) override {}
    void addFactionBibDrawable(Drawable*, Bool, Real = 0) override {}
    void removeFactionBibDrawable(Drawable*) override {}
    void removeAllBibs() override {}
    void removeBibHighlighting() override {}
    void removeTreesAndPropsForConstruction(const Coord3D*, const GeometryInfo&, Real) override {}
    void addProp(const ThingTemplate*, const Coord3D*, Real) override {}
    void setRawMapHeight(const ICoord2D*, Int) override {}
    Int getRawMapHeight(const ICoord2D*) override { return 0; }
    void replaceSkyboxTextures(const AsciiString*[NumSkyboxTextures], const AsciiString*[NumSkyboxTextures]) override {}
};

class OclTerrainLogic : public oracle_wp51::OracleTerrainLogic {
public:
    Real getGroundHeight(Real, Real, Coord3D* normal = nullptr) const override {
        if (normal) {
            normal->x = 0.0f;
            normal->y = 0.0f;
            normal->z = 1.0f;
        }
        return 12.5f;
    }

    Real getLayerHeight(Real x, Real y, PathfindLayerEnum layer, Coord3D* normal = nullptr,
                        Bool = true) const override {
        if (layer != LAYER_GROUND) {
            if (normal) {
                normal->x = 0.0f;
                normal->y = 0.0f;
                normal->z = 1.0f;
            }
            return 35.0f;
        }
        return getGroundHeight(x, y, normal);
    }

    Bool isUnderwater(Real x, Real y, Real* waterZ = nullptr,
                      Real* terrainZ = nullptr) override {
        if (x >= 700.0f && x < 800.0f && y >= 100.0f && y < 300.0f) {
            if (waterZ) *waterZ = 25.0f;
            if (terrainZ) *terrainZ = 5.0f;
            return TRUE;
        }
        if (waterZ) *waterZ = 0.0f;
        if (terrainZ) *terrainZ = 12.5f;
        return FALSE;
    }
};

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, sizeof(b));
    return b;
}

static std::string json_escape(const std::string& s) {
    std::string out;
    out.reserve(s.size() + 32);
    for (unsigned char c : s) {
        switch (c) {
            case '"': out += "\\\""; break;
            case '\\': out += "\\\\"; break;
            case '\n': out += "\\n"; break;
            case '\r': out += "\\r"; break;
            case '\t': out += "\\t"; break;
            default: out.push_back(static_cast<char>(c)); break;
        }
    }
    return out;
}

static const char* kFixtureIni = R"INI(
Locomotor TransportLoco
  Surfaces = AIR
  Speed = 120.0
  SpeedDamaged = 80.0
  MinSpeed = 0.0
  TurnRate = 100.0
  TurnRateDamaged = 100.0
  Acceleration = 100.0
  AccelerationDamaged = 100.0
  Lift = 100.0
  LiftDamaged = 100.0
  Braking = 100.0
  MinTurnSpeed = 0.0
  PreferredHeight = 120.0
  AllowAirborneMotiveForce = Yes
  ZAxisBehavior = SURFACE_RELATIVE_HEIGHT
  Appearance = WINGS
End

Locomotor GroundLoco
  Surfaces = GROUND
  Speed = 60.0
  SpeedDamaged = 40.0
  TurnRate = 180.0
  Acceleration = 100.0
  Braking = 100.0
  Appearance = TWO_LEGS
End

Weapon OclTempWeapon
  PrimaryDamage = 25.0
  PrimaryDamageRadius = 0.0
  AttackRange = 500.0
  DamageType = EXPLOSION
  DeathType = EXPLODED
End

Object GenericDebris
  KindOf = UNATTACKABLE
  Draw = W3DDebrisDraw ModuleTag_01
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 1.0
    AllowBouncing = No
  End
  Behavior = LifetimeUpdate ModuleTag_Life
    MinLifetime = 600
    MaxLifetime = 600
  End
End

Object OclPrimaryUnit
  KindOf = SELECTABLE CAN_ATTACK
  WeaponSet
    Conditions = None
    Weapon = PRIMARY OclTempWeapon
    Weapon = SECONDARY OclTempWeapon
  End
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200.0
    InitialHealth = 200.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 5.0
  End
  Behavior = AIUpdateInterface ModuleTag_AI
  End
  Locomotor = SET_NORMAL GroundLoco
  Behavior = RadiusDecalUpdate ModuleTag_Decal
  End
End

Object OclTargetUnit
  KindOf = SELECTABLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200.0
    InitialHealth = 200.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 4.0
  End
End

Object OclSlaveUnit
  KindOf = SELECTABLE
  ExperienceRequired = 0 10 20 30
  ExperienceValue = 10 10 10 10
  IsTrainable = Yes
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 100.0
    InitialHealth = 100.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 2.0
    AllowBouncing = No
  End
  Behavior = LifetimeUpdate ModuleTag_Life
    MinLifetime = 300
    MaxLifetime = 300
  End
  Behavior = FloatUpdate ModuleTag_Float
    Enabled = No
  End
  Behavior = SlavedUpdate ModuleTag_Slave
    GuardMaxRange = 100.0
    StayOnSameLayerAsMaster = No
  End
End

Object OclContainerUnit
  KindOf = SELECTABLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 200.0
    InitialHealth = 200.0
  End
  Behavior = TransportContain ModuleTag_Contain
    Slots = 4
  End
End

Object OclSmallBox
  KindOf = SELECTABLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 80.0
    InitialHealth = 80.0
  End
  Behavior = TransportContain ModuleTag_Contain
    Slots = 2
  End
End

Object OclStructure
  KindOf = STRUCTURE SELECTABLE
  Geometry = BOX
  GeometryMajorRadius = 20.0
  GeometryMinorRadius = 15.0
  GeometryHeight = 10.0
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 300.0
    InitialHealth = 300.0
  End
End

Object OclTransport
  KindOf = AIRCRAFT VEHICLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 250.0
    InitialHealth = 250.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 10.0
  End
  Behavior = TransportContain ModuleTag_Contain
    Slots = 6
  End
  Behavior = DeliverPayloadAIUpdate ModuleTag_DP
    DoorDelay = 100
    DeliveryDistance = 50.0
  End
  Locomotor = SET_NORMAL TransportLoco
  Behavior = SpecialPowerCompletionDie ModuleTag_SPD
  End
End

Object OclPayloadUnit
  KindOf = SELECTABLE
  Body = ActiveBody ModuleTag_Body
    MaxHealth = 120.0
    InitialHealth = 120.0
  End
  Behavior = PhysicsBehavior ModuleTag_Phys
    Mass = 1.5
  End
  Behavior = SpecialPowerCompletionDie ModuleTag_SPD
  End
End

ObjectCreationList OCL_DebrisComplex
  CreateDebris
    ModelNames = ModelA ModelB ModelC
    Count = 2
    Mass = 2.5
    AnimationSet = AnimInitA AnimFlyA AnimFinalA
    AnimationSet = AnimInitB AnimFlyB AnimFinalB
    Offset = X:4.0 Y:-2.0 Z:3.0
    Disposition = SEND_IT_FLYING INHERIT_VELOCITY WHIRLING
    DispositionIntensity = 3.5
    ExtraBounciness = 0.4
    ExtraFriction = 0.6
    SpinRate = 1.2
    YawRate = 0.5
    OrientInForceDirection = Yes
    IgnorePrimaryObstacle = Yes
    MinLifetime = 1200
    MaxLifetime = 2400
    SpreadFormation = Yes
    MinDistanceAFormation = 5.0
    MinDistanceBFormation = 15.0
    MaxDistanceFormation = 30.0
    PreserveLayer = Yes
  End
End

ObjectCreationList OCL_RandomForceSendUp
  CreateObject
    ObjectNames = OclSlaveUnit OclTargetUnit
    Count = 1
    Disposition = SEND_IT_UP FLOATING
    DispositionIntensity = 2.0
    MinHealth = 40%
    MaxHealth = 80%
    InvulnerableTime = 2000
    InheritsVeterancy = Yes
    MinLifetime = 500
    MaxLifetime = 900
  End
End

ObjectCreationList OCL_RandomForceCustom
  CreateObject
    ObjectNames = OclSlaveUnit
    Count = 1
    Disposition = RANDOM_FORCE
    MinForceMagnitude = 10.0
    MaxForceMagnitude = 25.0
    MinForcePitch = 15
    MaxForcePitch = 45
    SpinRate = 2.0
    OrientInForceDirection = No
  End
End

ObjectCreationList OCL_SendItOut
  CreateObject
    ObjectNames = OclSlaveUnit
    Count = 1
    Disposition = SEND_IT_OUT ON_GROUND_ALIGNED
    DispositionIntensity = 3.0
    ExtraFriction = 0.3
    OrientInForceDirection = Yes
  End
End

ObjectCreationList OCL_LikeExistingStructure
  CreateObject
    ObjectNames = OclStructure
    Count = 1
    Offset = X:10.0 Y:0.0 Z:5.0
    Disposition = LIKE_EXISTING
    SkipIfSignificantlyAirborne = Yes
  End
End

ObjectCreationList OCL_LikeExistingUnit
  CreateObject
    ObjectNames = OclSlaveUnit
    Count = 1
    Disposition = LIKE_EXISTING
    RequiresLivePlayer = Yes
  End
End

ObjectCreationList OCL_ContainInside
  CreateObject
    ObjectNames = OclSlaveUnit
    Count = 2
    ContainInsideSourceObject = Yes
    Disposition = LIKE_EXISTING
  End
End

ObjectCreationList OCL_PutInContainer
  CreateObject
    ObjectNames = OclSlaveUnit
    PutInContainer = OclContainerUnit
    Count = 2
    Disposition = LIKE_EXISTING
  End
End

ObjectCreationList OCL_BadLand
  CreateObject
    ObjectNames = OclSlaveUnit
    Count = 1
    Disposition = LIKE_EXISTING
    DiesOnBadLand = Yes
  End
End

ObjectCreationList OCL_ApplyRandomForce
  ApplyRandomForce
    SpinRate = 1.8
    MinForceMagnitude = 8.0
    MaxForceMagnitude = 16.0
    MinForcePitch = 20
    MaxForcePitch = 60
  End
End

ObjectCreationList OCL_FireWeapon
  FireWeapon
    Weapon = OclTempWeapon
  End
End

ObjectCreationList OCL_Attack
  Attack
    NumberOfShots = 3
    WeaponSlot = SECONDARY
    DeliveryDecal
      Texture = TestDecal
      OpacityMin = 50%
      OpacityMax = 90%
    End
    DeliveryDecalRadius = 45.0
  End
End

ObjectCreationList OCL_DeliverPayload
  DeliverPayload
    Transport = OclTransport
    StartAtPreferredHeight = Yes
    StartAtMaxSpeed = Yes
    FormationSize = 3
    FormationSpacing = 30.0
    WeaponConvergenceFactor = 0.25
    WeaponErrorRadius = 15.0
    DelayDeliveryMax = 1000
    DeliveryDistance = 80.0
    DeliveryDecalRadius = 60.0
    Payload = OclPayloadUnit 2
    PutInContainer = OclSmallBox
  End
End

ObjectCreationList OCL_MultiNugget
  FireWeapon
    Weapon = OclTempWeapon
  End
  CreateObject
    ObjectNames = OclSlaveUnit
    Count = 1
    Disposition = LIKE_EXISTING
  End
  CreateObject
    ObjectNames = OclTargetUnit
    Count = 1
    Disposition = LIKE_EXISTING
  End
End
)INI";

struct ObjSnap {
    uint32_t id = 0;
    uint32_t pos[3] = {0, 0, 0};
    uint32_t orient = 0;
    uint32_t vel[3] = {0, 0, 0};
    uint32_t accel[3] = {0, 0, 0};
    uint32_t yaw_rate = 0;
    uint32_t roll_rate = 0;
    uint32_t pitch_rate = 0;
    uint32_t mass = 0;
    uint32_t extra_bounce = 0;
    uint32_t extra_friction = 0;
    uint32_t ignore_id = 0;
    uint32_t health = 0;
    int32_t vet = 0;
    int32_t layer = 0;
    uint32_t producer = 0;
    uint32_t slaver = 0;
    uint32_t lifetime_die = 0;
    int32_t float_en = 0;
    int32_t undetected = 0;
    uint32_t script_status = 0;
    uint32_t disabled_till0 = 0;
    uint32_t spd_creator = 0;
    uint32_t contained_cnt = 0;
    int32_t destroyed = 0;
    int32_t off_map = 0;
    int32_t radius_kill = -1;
    uint32_t dp_move_to[3] = {0, 0, 0};
    uint32_t dp_target[3] = {0, 0, 0};
};

static ObjSnap snap_obj(const Object* obj) {
    ObjSnap s;
    if (!obj) return s;
    s.id = static_cast<uint32_t>(obj->getID());
    const Coord3D* p = obj->getPosition();
    s.pos[0] = fbits(p->x);
    s.pos[1] = fbits(p->y);
    s.pos[2] = fbits(p->z);
    s.orient = fbits(obj->getOrientation());
    const PhysicsBehavior* phys = obj->getPhysics();
    if (phys) {
        const Coord3D* v = phys->getVelocity();
        const Coord3D& a = phys->*access_member(OclPhysicsAccelTag());
        s.vel[0] = fbits(v->x);
        s.vel[1] = fbits(v->y);
        s.vel[2] = fbits(v->z);
        s.accel[0] = fbits(a.x);
        s.accel[1] = fbits(a.y);
        s.accel[2] = fbits(a.z);
        s.yaw_rate = fbits(phys->*access_member(OclPhysicsYawTag()));
        s.roll_rate = fbits(phys->*access_member(OclPhysicsRollTag()));
        s.pitch_rate = fbits(phys->*access_member(OclPhysicsPitchTag()));
        s.mass = fbits(phys->getMass());
        s.extra_bounce = fbits(phys->*access_member(OclPhysicsBounceTag()));
        s.extra_friction = fbits(phys->*access_member(OclPhysicsFrictionTag()));
        s.ignore_id = static_cast<uint32_t>(phys->*access_member(OclPhysicsIgnoreTag()));
    }
    if (const BodyModuleInterface* body = obj->getBodyModule()) {
        s.health = fbits(body->getHealth());
    }
    s.vet = static_cast<int32_t>(obj->getVeterancyLevel());
    s.layer = static_cast<int32_t>(obj->getLayer());
    s.producer = static_cast<uint32_t>(obj->getProducerID());
    s.undetected = obj->getIsUndetectedDefector() ? 1 : 0;
    s.script_status = static_cast<uint32_t>(obj->*access_member(OclScriptStatusTag()));
    s.disabled_till0 = obj->getDisabledUntil(DISABLED_DEFAULT);
    s.destroyed = obj->isDestroyed() ? 1 : 0;
    s.off_map = obj->isOffMap() ? 1 : 0;

    Object* mutable_obj = const_cast<Object*>(obj);
    static NameKeyType key_lu = NAMEKEY("LifetimeUpdate");
    if (auto* lu = static_cast<LifetimeUpdate*>(mutable_obj->findUpdateModule(key_lu))) {
        s.lifetime_die = lu->getDieFrame();
    }
    static NameKeyType key_fu = NAMEKEY("FloatUpdate");
    if (auto* fu = static_cast<FloatUpdate*>(mutable_obj->findUpdateModule(key_fu))) {
        s.float_en = (fu->*access_member(OclFloatEnabledTag())) ? 1 : 0;
    }
    static NameKeyType key_rdu = NAMEKEY("RadiusDecalUpdate");
    if (auto* rdu = static_cast<RadiusDecalUpdate*>(mutable_obj->findUpdateModule(key_rdu))) {
        s.radius_kill = (rdu->*access_member(OclRadiusKillTag())) ? 1 : 0;
    }
    for (BehaviorModule** bm = mutable_obj->getBehaviorModules(); bm && *bm; ++bm) {
        if (SlavedUpdateInterface* su = (*bm)->getSlavedUpdateInterface()) {
            s.slaver = static_cast<uint32_t>(su->getSlaverID());
            break;
        }
    }
    if (const SpecialPowerCompletionDie* spd = mutable_obj->findSpecialPowerCompletionDie()) {
        s.spd_creator = static_cast<uint32_t>(spd->*access_member(OclSpdCreatorTag()));
    }
    if (const ContainModuleInterface* cm = obj->getContain()) {
        s.contained_cnt = cm->getContainCount();
    }
    static NameKeyType key_dp = NAMEKEY("DeliverPayloadAIUpdate");
    if (auto* dp = static_cast<DeliverPayloadAIUpdate*>(mutable_obj->findUpdateModule(key_dp))) {
        const Coord3D* mt = dp->getMoveToPos();
        const Coord3D* tp = dp->getTargetPos();
        s.dp_move_to[0] = fbits(mt->x);
        s.dp_move_to[1] = fbits(mt->y);
        s.dp_move_to[2] = fbits(mt->z);
        s.dp_target[0] = fbits(tp->x);
        s.dp_target[1] = fbits(tp->y);
        s.dp_target[2] = fbits(tp->z);
    }
    return s;
}

struct CaseOut {
    const char* name;
    uint32_t rng_crc;
    uint32_t extra_u32;
    ObjSnap obj;
};

static const ObjectCreationList* find_ocl(const char* name) {
    return TheObjectCreationListStore->findObjectCreationList(name);
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/OclRuntime.json";

    oracle_wp51::World w;
    struct OracleModuleFactory : public ModuleFactory {
        using ModuleFactory::addModuleInternal;
    };
    static_cast<OracleModuleFactory*>(TheModuleFactory)->addModuleInternal(
        OracleW3DDebrisDraw::create_inst,
        OracleW3DDebrisDraw::create_data,
        DrawModule::getModuleType(),
        AsciiString("W3DDebrisDraw"),
        DrawModule::getInterfaceMask());
    delete TheTerrainLogic;
    TheTerrainLogic = NEW OclTerrainLogic;
    TheTerrainLogic->init();
    TheTerrainVisual = new NullTerrainVisual;
    TheTerrainVisual->init();
    TheProjectedShadowManager = new NullProjectedShadowManager;
    ThePartitionManager->init();
    TheAI->pathfinder()->newMap();
    {
        std::string all(kFixtureIni);
        size_t start = 0;
        while (start < all.size()) {
            size_t end = all.find("\n\n", start);
            std::string blk = (end == std::string::npos) ? all.substr(start) : all.substr(start, end - start);
            bool has_non_ws = false;
            for (char ch : blk) {
                if (ch != ' ' && ch != '\n' && ch != '\r' && ch != '\t') { has_non_ws = true; break; }
            }
            if (has_non_ws) {
                try {
                    w.load_ini(blk);
                } catch (ErrorCode err) {
                    std::fprintf(stderr, "FAILED INI BLOCK (err=%d):\n%s\n", static_cast<int>(err), blk.c_str());
                    return 2;
                } catch (...) {
                    std::fprintf(stderr, "FAILED INI BLOCK (unknown):\n%s\n", blk.c_str());
                    return 2;
                }
            }
            if (end == std::string::npos) break;
            start = end + 2;
        }
    }

    // Mark bad land cells for diesOnBadLand tests:
    // (450, 450) -> cell (45, 45) CLIFF
    // (470, 470) -> cell (47, 47) WATER
    // (490, 490) -> cell (49, 49) IMPASSABLE
    if (PathfindCell* c = TheAI->pathfinder()->getCell(LAYER_GROUND, 45, 45)) {
        c->setType(PathfindCell::CELL_CLIFF);
    }
    if (PathfindCell* c = TheAI->pathfinder()->getCell(LAYER_GROUND, 47, 47)) {
        c->setType(PathfindCell::CELL_WATER);
    }
    if (PathfindCell* c = TheAI->pathfinder()->getCell(LAYER_GROUND, 49, 49)) {
        c->setType(PathfindCell::CELL_IMPASSABLE);
    }

    FILE* fp = std::fopen(out_path, "w");
    if (!fp) {
        std::fprintf(stderr, "cannot open %s\n", out_path);
        return 1;
    }

    std::fprintf(fp, "{\n");
    std::fprintf(fp, "  \"ini\": \"%s\",\n", json_escape(kFixtureIni).c_str());
    std::fprintf(fp, "  \"seeds\": [\n");

    const uint32_t kSeeds = 16;
    for (uint32_t seed_idx = 0; seed_idx < kSeeds; ++seed_idx) {
        w.set_frame(100 + seed_idx);
        const uint32_t seed_val = 0x13572468u + seed_idx * 977u;
        InitRandom(seed_val);

        std::vector<CaseOut> cases;
        auto push_case = [&](const char* cname, const Object* o, uint32_t extra = 0) {
            CaseOut co;
            co.name = cname;
            co.rng_crc = GetGameLogicRandomSeedCRC();
            co.extra_u32 = extra;
            co.obj = snap_obj(o);
            cases.push_back(co);
        };

        // 1. Primary spawner (veteran hero, moving velocity, bridge layer, rotated)
        Object* primary = w.spawn_object("OclPrimaryUnit", w.team_a, {120.0f, 160.0f, 12.5f}, 0.75f);
        primary->getExperienceTracker()->setVeterancyLevel(LEVEL_HEROIC, FALSE);
        {
            Coord3D init_vel = {3.0f, -1.5f, 0.5f};
            primary->getPhysics()->applyForce(&init_vel);
            // Step velocity slightly by adding directly? applyForce updates accel; set velocity via another force or keep accel.
        }
        primary->setLayer(static_cast<PathfindLayerEnum>(2));
        Object* target = w.spawn_object("OclTargetUnit", w.team_b, {220.0f, 190.0f, 12.5f}, 0.0f);
        ThePartitionManager->update();

        // Case 0: Generic CreateDebris + W3DDebrisDraw anim_sets + spreadFormation + SEND_IT_FLYING + WHIRLING + offset rotation
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_DebrisComplex"), primary, target, 0);
            push_case("debris_complex", first);
        }
        // Case 1: Generic CreateObject + SEND_IT_UP + FLOATING + invulnerableTime + inheritsVeterancy + lifetimeFrames override
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_RandomForceSendUp"), primary, nullptr, 45);
            push_case("send_up_float_vet", first);
        }
        // Case 2: RANDOM_FORCE + calcRandomForce + spinRate
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_RandomForceCustom"), primary, nullptr, 0);
            push_case("random_force_custom", first);
        }
        // Case 3: SEND_IT_OUT + ON_GROUND_ALIGNED + orientInForceDirection
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_SendItOut"), primary, nullptr, 0);
            push_case("send_it_out_aligned", first);
        }
        // Case 4: LIKE_EXISTING structure (flattenTerrain + pathfind map) while primary is on ground
        {
            primary->setLayer(LAYER_GROUND);
            Coord3D ground_pos = {150.0f, 150.0f, 12.5f};
            primary->setPosition(&ground_pos);
            Object* first = ObjectCreationList::create(find_ocl("OCL_LikeExistingStructure"), primary, nullptr, 0);
            push_case("like_existing_structure", first);
        }
        // Case 5: skipIfSignificantlyAirborne triggers when primary is high up
        {
            Coord3D air_pos = {150.0f, 150.0f, 100.0f};
            Coord3D ground_pos = {150.0f, 150.0f, 12.5f};
            primary->setPosition(&air_pos);
            Object* first = ObjectCreationList::create(find_ocl("OCL_LikeExistingStructure"), primary, nullptr, 0);
            push_case("skip_if_airborne", first);
            primary->setPosition(&ground_pos);
        }
        // Case 6: Positional + angle overload with explicit angle
        {
            Coord3D ppos = {300.0f, 320.0f, 12.5f};
            Object* first = ObjectCreationList::create(find_ocl("OCL_LikeExistingStructure"), primary, &ppos, nullptr, 1.25f, 0);
            push_case("pos_angle_explicit", first);
        }
        // Case 7: Positional with INVALID_ANGLE defaults to 0.0
        {
            Coord3D ppos = {340.0f, 320.0f, 12.5f};
            Object* first = ObjectCreationList::create(find_ocl("OCL_LikeExistingUnit"), primary, &ppos, nullptr, INVALID_ANGLE, 0);
            push_case("pos_angle_invalid", first);
        }
        // Case 8: ContainInsideSourceObject (valid container) vs invalid (non-container primary -> destroyed)
        {
            Object* cont = w.spawn_object("OclContainerUnit", w.team_a, {200.0f, 200.0f, 12.5f}, 0.0f);
            Object* first = ObjectCreationList::create(find_ocl("OCL_ContainInside"), cont, nullptr, 0);
            push_case("contain_inside_valid", cont, first ? static_cast<uint32_t>(first->getID()) : 0);
        }
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_ContainInside"), primary, nullptr, 0);
            push_case("contain_inside_invalid_destroys", first);
        }
        // Case 10: PutInContainer
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_PutInContainer"), primary, nullptr, 0);
            push_case("put_in_container", first);
        }
        // Case 11: diesOnBadLand - underwater (x=720, y=150, z=15 <= waterZ+10=35)
        {
            Coord3D uw = {720.0f, 150.0f, 15.0f};
            Object* first = ObjectCreationList::create(find_ocl("OCL_BadLand"), primary, &uw, nullptr, 0.0f, 0);
            push_case("bad_land_underwater", first);
        }
        // Case 12: diesOnBadLand - cliff cell (455, 455)
        {
            Coord3D cliff = {455.0f, 455.0f, 12.5f};
            Object* first = ObjectCreationList::create(find_ocl("OCL_BadLand"), primary, &cliff, nullptr, 0.0f, 0);
            push_case("bad_land_cliff", first);
        }
        // Case 13: diesOnBadLand - off map (-50, 100)
        {
            Coord3D off = {-50.0f, 100.0f, 12.5f};
            Object* first = ObjectCreationList::create(find_ocl("OCL_BadLand"), primary, &off, nullptr, 0.0f, 0);
            push_case("bad_land_off_map", first);
        }
        // Case 14: ApplyRandomForce object overload + positional overload (ignored)
        {
            Object* arf_obj = w.spawn_object("OclTargetUnit", w.team_a, {180.0f, 180.0f, 12.5f}, 0.0f);
            Object* res = ObjectCreationList::create(find_ocl("OCL_ApplyRandomForce"), arf_obj, nullptr, 0);
            push_case("apply_random_force_obj", arf_obj, res ? 1u : 0u);

            Coord3D ppos = {180.0f, 180.0f, 12.5f};
            ObjectCreationList::create(find_ocl("OCL_ApplyRandomForce"), arf_obj, &ppos, nullptr, 0.0f, 0);
            push_case("apply_random_force_pos_noop", arf_obj);
        }
        // Case 16: FireWeapon nugget (damages target at secondary_pos)
        {
            Coord3D ppos = *primary->getPosition();
            Coord3D spos = *target->getPosition();
            ObjectCreationList::create(find_ocl("OCL_FireWeapon"), primary, &ppos, &spos, 0.0f, 0);
            push_case("fire_weapon", target);
        }
        // Case 17: Attack nugget (locks secondary slot, RadiusDecalUpdate killWhenNoLongerAttacking)
        {
            Coord3D ppos = *primary->getPosition();
            Coord3D spos = {250.0f, 260.0f, 12.5f};
            ObjectCreationList::create(find_ocl("OCL_Attack"), primary, &ppos, &spos, 0.0f, 0);
            push_case("attack_nugget", primary);
        }
        // Case 18: DeliverPayload (createOwner = TRUE, formationSize = 3, errorRadius, payload + PutInContainer)
        {
            Coord3D ppos = {50.0f, 60.0f, 12.5f};
            Coord3D spos = {500.0f, 400.0f, 12.5f};
            Object* first_tr = ObjectCreationList::create(find_ocl("OCL_DeliverPayload"), primary, &ppos, &spos, static_cast<Bool>(TRUE), 0);
            push_case("deliver_payload_create_owner", first_tr);
        }
        // Case 19: Multi-nugget returns FIRST created object (FireWeapon returns null, first CreateObject returns SlaveUnit)
        {
            Object* first = ObjectCreationList::create(find_ocl("OCL_MultiNugget"), primary, target, 0);
            push_case("multi_nugget_first_obj", first);
        }

        // Flush destroy list and clean up objects created during this seed to keep memory tidy
        // (Actually keep only flushing destroyed objects so IDs remain clean and monotonic across seeds)
        w.flush_destroy_list();

        std::fprintf(fp, "    {\n");
        std::fprintf(fp, "      \"seed_idx\": %u,\n", seed_idx);
        std::fprintf(fp, "      \"seed\": %u,\n", seed_val);
        std::fprintf(fp, "      \"frame\": %u,\n", 100 + seed_idx);
        std::fprintf(fp, "      \"cases\": [\n");
        for (size_t i = 0; i < cases.size(); ++i) {
            const auto& c = cases[i];
            const auto& o = c.obj;
            std::fprintf(fp,
                "        {\"name\":\"%s\",\"rng_crc\":%u,\"extra\":%u,"
                "\"id\":%u,\"pos\":[%u,%u,%u],\"orient\":%u,"
                "\"vel\":[%u,%u,%u],\"accel\":[%u,%u,%u],"
                "\"yaw\":%u,\"roll\":%u,\"pitch\":%u,\"mass\":%u,"
                "\"bounce\":%u,\"friction\":%u,\"ignore_id\":%u,"
                "\"health\":%u,\"vet\":%d,\"layer\":%d,\"producer\":%u,"
                "\"slaver\":%u,\"lifetime_die\":%u,\"float_en\":%d,"
                "\"undetected\":%d,\"script_status\":%u,\"disabled0\":%u,"
                "\"spd_creator\":%u,\"contained\":%u,\"destroyed\":%d,"
                "\"off_map\":%d,\"radius_kill\":%d,"
                "\"dp_move_to\":[%u,%u,%u],\"dp_target\":[%u,%u,%u]}%s\n",
                c.name, c.rng_crc, c.extra_u32,
                o.id, o.pos[0], o.pos[1], o.pos[2], o.orient,
                o.vel[0], o.vel[1], o.vel[2], o.accel[0], o.accel[1], o.accel[2],
                o.yaw_rate, o.roll_rate, o.pitch_rate, o.mass,
                o.extra_bounce, o.extra_friction, o.ignore_id,
                o.health, o.vet, o.layer, o.producer,
                o.slaver, o.lifetime_die, o.float_en,
                o.undetected, o.script_status, o.disabled_till0,
                o.spd_creator, o.contained_cnt, o.destroyed,
                o.off_map, o.radius_kill,
                o.dp_move_to[0], o.dp_move_to[1], o.dp_move_to[2],
                o.dp_target[0], o.dp_target[1], o.dp_target[2],
                (i + 1 < cases.size()) ? "," : "");
        }
        std::fprintf(fp, "      ]\n");
        std::fprintf(fp, "    }%s\n", (seed_idx + 1 < kSeeds) ? "," : "");
    }

    std::fprintf(fp, "  ]\n");
    std::fprintf(fp, "}\n");
    std::fclose(fp);
    return 0;
}
