// WP-5.2-D2 CollideModule & CrateCollide remaining 10 packets:
// PKT-201, 202, 215, 219, 224, 240, 241, 246, 250, 254
// C++ Golden Vector Generator (Zero-Stub Build against oracle_wp51_world.h).
//
// Canonical 3-mode build & run:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_wp52_collide_crates.cpp --opt O2 --out /tmp/gen_wp52_d2_o2
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_wp52_d2_o2 docs/golden_vectors/WP52_CollideCrates.json

#include "oracle_wp51_world.h"

#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/RandomValue.h"
#include "Common/Team.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameClient/FXList.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/CollideModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/ConvertToCarBombCrateCollide.h"
#include "GameLogic/Module/ConvertToHijackedVehicleCrateCollide.h"
#include "GameLogic/Module/CrateCollide.h"
#include "GameLogic/Module/FireWeaponCollide.h"
#include "GameLogic/Module/HealCrateCollide.h"
#include "GameLogic/Module/MoneyCrateCollide.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/SalvageCrateCollide.h"
#include "GameLogic/Module/ShroudCrateCollide.h"
#include "GameLogic/Module/SquishCollide.h"
#include "GameLogic/Module/UnitCrateCollide.h"
#include "GameLogic/Module/VeterancyCrateCollide.h"
#include "GameLogic/Object.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/WeaponSet.h"

#include <sstream>
#include <string>
#include <unordered_set>
#include <vector>

struct PlayerTypeD2Tag { using type = PlayerType Player::*; };
template struct MemberAccess<PlayerTypeD2Tag, &Player::m_playerType>;
PlayerType Player::* access_member(PlayerTypeD2Tag);

struct PlayerUpgD2Tag { using type = UpgradeMaskType Player::*; };
template struct MemberAccess<PlayerUpgD2Tag, &Player::m_upgradesCompleted>;
UpgradeMaskType Player::* access_member(PlayerUpgD2Tag);

struct FireCollideWpnTag { using type = Weapon* FireWeaponCollide::*; };
template struct MemberAccess<FireCollideWpnTag, &FireWeaponCollide::m_collideWeapon>;
Weapon* FireWeaponCollide::* access_member(FireCollideWpnTag);

struct FireCollideEverTag { using type = Bool FireWeaponCollide::*; };
template struct MemberAccess<FireCollideEverTag, &FireWeaponCollide::m_everFired>;
Bool FireWeaponCollide::* access_member(FireCollideEverTag);

static CollideModule* find_first_collide(Object* obj) {
    if (!obj) return nullptr;
    for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
        if ((*m)->getCollide()) {
            return static_cast<CollideModule*>(*m);
        }
    }
    return nullptr;
}

static void set_player_money(Player* p, UnsignedInt amount) {
    if (!p) return;
    Money* m = p->getMoney();
    if (!m) return;
    UnsignedInt cur = m->countMoney();
    if (cur > 0) m->withdraw(cur, FALSE);
    if (amount > 0) m->deposit(amount, FALSE, FALSE);
}

static std::vector<uint8_t> hex_to_bytes(const std::string& hex) {
    std::vector<uint8_t> out;
    out.reserve(hex.size() / 2);
    auto nibble = [](char c) -> uint8_t {
        if (c >= '0' && c <= '9') return static_cast<uint8_t>(c - '0');
        if (c >= 'a' && c <= 'f') return static_cast<uint8_t>(10 + (c - 'a'));
        if (c >= 'A' && c <= 'F') return static_cast<uint8_t>(10 + (c - 'A'));
        return 0;
    };
    for (size_t i = 0; i + 1 < hex.size(); i += 2) {
        out.push_back(static_cast<uint8_t>((nibble(hex[i]) << 4) | nibble(hex[i + 1])));
    }
    return out;
}

static void destroy_if_live(Object* obj) {
    if (obj && !obj->isDestroyed()) {
        TheGameLogic->destroyObject(obj);
    }
}

class HeadlessInGameUID2 : public oracle_wp51::NullInGameUI {
public:
    void addFloatingText(const UnicodeString&, const Coord3D*, Color) override {}
};

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP52_CollideCrates.json";
    oracle_wp51::World w;
    TheInGameUI = new HeadlessInGameUID2;

    // Player 0 (p_a / team_a): Neutral.
    // Player 1 (p_b / team_b): Active player B (crate owners / terrorist / hijacker / pilot / victim).
    // Player 2 (p_c / team_c): Active player C (target vehicles / pickers / crushers).
    Player* p_a = ThePlayerList->getNthPlayer(0);
    Player* p_b = ThePlayerList->getNthPlayer(1);
    Player* p_c = ThePlayerList->getNthPlayer(2);
    ThePlayerList->*access_member(Wp51PlayerListCountTag()) = 3;
    p_c->init(nullptr);
    if (!p_c->getResourceGatheringManager()) {
        p_c->*access_member(Wp51PlayerRgmTag()) = newInstance(ResourceGatheringManager);
    }
    Dict dict_c;
    TheTeamFactory->initTeam("TeamC", AsciiString::TheEmptyString, TRUE, &dict_c);
    Team* team_c = TheTeamFactory->findTeam("TeamC");
    team_c->setControllingPlayer(p_c);
    p_c->*access_member(Wp51PlayerDefaultTeamTag()) = team_c;

    p_a->setPlayerRelationship(p_b, NEUTRAL);
    p_b->setPlayerRelationship(p_a, NEUTRAL);
    p_a->setPlayerRelationship(p_c, NEUTRAL);
    p_c->setPlayerRelationship(p_a, NEUTRAL);

    // Static rules, upgrades, weapons, locomotors, targets, and fixed-param crates.
    const std::string static_ini =
        "FXList FX_CrateExec\n"
        "End\n"
        "FXList FX_CarBomb\n"
        "End\n"
        "Upgrade Upg_MoneyBoost\n"
        "  Type = PLAYER\n"
        "End\n"
        "Weapon Wpn_CollideLaser\n"
        "  PrimaryDamage = 10\n"
        "  AttackRange = 100\n"
        "  DamageType = SMALL_ARMS\n"
        "  DeathType = NORMAL\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 3\n"
        "  ClipReloadTime = 900\n"
        "  DelayBetweenShots = 60\n"
        "  AutoReloadsClip = Yes\n"
        "  LeechRangeWeapon = Yes\n"
        "End\n"
        "Weapon Wpn_CollideSingle\n"
        "  PrimaryDamage = 20\n"
        "  AttackRange = 100\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = EXPLODED\n"
        "  WeaponSpeed = 999999\n"
        "  ClipSize = 1\n"
        "  ClipReloadTime = 300\n"
        "  DelayBetweenShots = 30\n"
        "  AutoReloadsClip = No\n"
        "End\n"
        "Weapon Wpn_CarBombBlast\n"
        "  PrimaryDamage = 500\n"
        "  AttackRange = 100\n"
        "  DamageType = EXPLOSION\n"
        "  DeathType = EXPLODED\n"
        "  WeaponSpeed = 999999\n"
        "End\n"
        "Locomotor Loco_Ground\n"
        "  Surfaces = GROUND\n"
        "  Speed = 30\n"
        "  SpeedDamaged = 20\n"
        "  TurnRate = 90\n"
        "  TurnRateDamaged = 90\n"
        "  Acceleration = 10\n"
        "  Braking = 10\n"
        "  Appearance = TREADS\n"
        "End\n"
        "Locomotor Loco_Air\n"
        "  Surfaces = AIR\n"
        "  Speed = 60\n"
        "  SpeedDamaged = 40\n"
        "  TurnRate = 120\n"
        "  TurnRateDamaged = 120\n"
        "  Acceleration = 20\n"
        "  Braking = 20\n"
        "  Appearance = WINGS\n"
        "End\n"
        "ObjectCreationList OCL_EjectEmpty\n"
        "End\n"
        "Object Cr52_SpawnedUnit\n"
        "  KindOf = INFANTRY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50\n"
        "    InitialHealth = 50\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_Decoy\n"
        "  KindOf = INFANTRY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50\n"
        "    InitialHealth = 50\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_HealInf\n"
        "  KindOf = INFANTRY\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50\n"
        "    InitialHealth = 50\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_HealBldg\n"
        "  KindOf = STRUCTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "End\n"
        "Object Cr52_VictimCar\n"
        "  KindOf = VEHICLE\n"
        "  VisionRange = 120\n"
        "  ShroudClearingRange = 140\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  WeaponSet\n"
        "    Weapon = PRIMARY Wpn_CollideSingle\n"
        "  End\n"
        "  WeaponSet\n"
        "    Conditions = CARBOMB\n"
        "    Weapon = PRIMARY Wpn_CarBombBlast\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  ExperienceValue = 10 20 30 40\n"
        "End\n"
        "Object Cr52_PlainCar\n"
        "  KindOf = VEHICLE\n"
        "  CrusherLevel = 2\n"
        "  Geometry = CYLINDER\n"
        "  GeometryMajorRadius = 15\n"
        "  GeometryHeight = 10\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Physics\n"
        "  End\n"
        "End\n"
        "Object Cr52_BoxTank\n"
        "  KindOf = VEHICLE\n"
        "  CrusherLevel = 2\n"
        "  Geometry = BOX\n"
        "  GeometryMajorRadius = 20\n"
        "  GeometryMinorRadius = 10\n"
        "  GeometryHeight = 10\n"
        "  VisionRange = 120\n"
        "  ShroudClearingRange = 130\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  ExperienceValue = 10 20 30 40\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 300\n"
        "    InitialHealth = 300\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Physics\n"
        "  End\n"
        "  Behavior = EjectPilotDie ModuleTag_Eject\n"
        "    AirCreationList = OCL_EjectEmpty\n"
        "    GroundCreationList = OCL_EjectEmpty\n"
        "    InvulnerableTime = 30\n"
        "  End\n"
        "End\n"
        "Object Cr52_NonCrusherVeh\n"
        "  KindOf = VEHICLE\n"
        "  CrusherLevel = 0\n"
        "  Geometry = CYLINDER\n"
        "  GeometryMajorRadius = 15\n"
        "  GeometryHeight = 10\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = PhysicsBehavior ModuleTag_Physics\n"
        "  End\n"
        "End\n"
        "Object Cr52_Transport\n"
        "  KindOf = VEHICLE TRANSPORT\n"
        "  VisionRange = 120\n"
        "  ShroudClearingRange = 120\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = TransportContain ModuleTag_Contain\n"
        "    ContainMax = 2\n"
        "    AllowInsideKindOf = INFANTRY\n"
        "  End\n"
        "End\n"
        "Object Cr52_Boat\n"
        "  KindOf = VEHICLE BOAT\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_Drone\n"
        "  KindOf = VEHICLE DRONE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_Immune\n"
        "  KindOf = VEHICLE IMMUNE_TO_CAPTURE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_AirUnit\n"
        "  KindOf = AIRCRAFT VEHICLE\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL Loco_Air\n"
        "End\n"
        "Object Cr52_TrainableVeh\n"
        "  KindOf = VEHICLE\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  ExperienceValue = 10 20 30 40\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL Loco_Ground\n"
        "End\n"
        "Object Cr52_NonTrainableVeh\n"
        "  KindOf = VEHICLE\n"
        "  IsTrainable = No\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Locomotor = SET_NORMAL Loco_Ground\n"
        "End\n"
        "Object Cr52_SalvagerAll\n"
        "  KindOf = VEHICLE SALVAGER WEAPON_SALVAGER ARMOR_SALVAGER\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_SalvagerWpn\n"
        "  KindOf = VEHICLE SALVAGER WEAPON_SALVAGER\n"
        "  IsTrainable = Yes\n"
        "  ExperienceRequired = 0 50 100 200\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_SalvagerPlain\n"
        "  KindOf = VEHICLE SALVAGER\n"
        "  IsTrainable = No\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 200\n"
        "    InitialHealth = 200\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "End\n"
        "Object Cr52_Mod_Heal\n"
        "  KindOf = IMMOBILE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 10\n"
        "    InitialHealth = 10\n"
        "  End\n"
        "  Behavior = HealCrateCollide ModuleTag_Collide\n"
        "    ExecuteFX = FX_CrateExec\n"
        "  End\n"
        "End\n"
        "Object Cr52_Mod_Shroud\n"
        "  KindOf = IMMOBILE\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 10\n"
        "    InitialHealth = 10\n"
        "  End\n"
        "  Behavior = ShroudCrateCollide ModuleTag_Collide\n"
        "    ExecuteFX = FX_CrateExec\n"
        "  End\n"
        "End\n"
        "Object Cr52_Mod_CarBomb\n"
        "  KindOf = INFANTRY\n"
        "  VisionRange = 180\n"
        "  ShroudClearingRange = 220\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50\n"
        "    InitialHealth = 50\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = ConvertToCarBombCrateCollide ModuleTag_Collide\n"
        "    FXList = FX_CarBomb\n"
        "    ExecuteFX = FX_CrateExec\n"
        "  End\n"
        "End\n"
        "Object Cr52_Mod_Hijack\n"
        "  KindOf = INFANTRY\n"
        "  VisionRange = 160\n"
        "  ShroudClearingRange = 190\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 50\n"
        "    InitialHealth = 50\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = HijackerUpdate ModuleTag_Hijack\n  End\n"
        "  Behavior = ConvertToHijackedVehicleCrateCollide ModuleTag_Collide\n"
        "    ExecuteFX = FX_CrateExec\n"
        "  End\n"
        "End\n"
        "Object Cr52_Mod_Squish\n"
        "  KindOf = INFANTRY\n"
        "  Geometry = CYLINDER\n"
        "  GeometryMajorRadius = 5\n"
        "  GeometryHeight = 5\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100\n"
        "    InitialHealth = 100\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = SquishCollide ModuleTag_Collide\n  End\n"
        "End\n"
        "Object Cr52_Mod_SquishHijacker\n"
        "  KindOf = INFANTRY\n"
        "  Geometry = CYLINDER\n"
        "  GeometryMajorRadius = 5\n"
        "  GeometryHeight = 5\n"
        "  Body = ActiveBody ModuleTag_Body\n"
        "    MaxHealth = 100\n"
        "    InitialHealth = 100\n"
        "  End\n"
        "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
        "  Behavior = HijackerUpdate ModuleTag_Hijack\n  End\n"
        "  Behavior = SquishCollide ModuleTag_Collide\n  End\n"
        "End\n";

    try {
        std::istringstream ss(static_ini);
        std::string line, cur;
        int depth = 0;
        while (std::getline(ss, line)) {
            cur += line + "\n";
            // check top-level End vs nested End
            std::string trimmed = line;
            size_t a = trimmed.find_first_not_of(" \t");
            if (a == std::string::npos) continue;
            trimmed = trimmed.substr(a);
            if (depth == 0) {
                depth = 1;
            } else if (trimmed == "End" || trimmed == "END") {
                --depth;
                if (depth == 0) {
                    try {
                        w.load_ini(cur, INI_LOAD_OVERWRITE);
                    } catch (...) {
                        std::fprintf(stderr, "INI FAIL STATIC BLOCK:\n%s\n", cur.c_str());
                        return 1;
                    }
                    cur.clear();
                }
            } else if (trimmed.rfind("WeaponSet", 0) == 0 || trimmed.rfind("Behavior =", 0) == 0 || trimmed.rfind("Body =", 0) == 0) {
                ++depth;
            }
        }
    } catch (...) {
        std::fprintf(stderr, "INI FAIL OUTER\n");
        return 1;
    }

    const UpgradeTemplate* ut_boost = TheUpgradeCenter->findUpgrade("Upg_MoneyBoost");
    static const VeterancyLevel kVetLevels[4] = {
        LEVEL_REGULAR, LEVEL_VETERAN, LEVEL_ELITE, LEVEL_HEROIC
    };
    const Coord3D col_loc = {0.0f, 0.0f, 0.0f};
    const Coord3D col_norm = {0.0f, 0.0f, 1.0f};

    std::ostringstream vecs;
    std::string s0_diag;
    uint32_t rng = 0x52D201u;
    auto next_u32 = [&]() -> uint32_t {
        return oracle::splitmix32(rng);
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        const std::string sfx = std::to_string(seed);
        const UnsignedInt frame0 = 10u + (next_u32() % 200u);
        w.set_frame(frame0);

        const uint32_t rel_roll = (seed == 0) ? 0u : (next_u32() % 4u);
        const Relationship rel_bc = (rel_roll < 3u)
            ? ENEMIES
            : ((next_u32() & 1u) ? ALLIES : NEUTRAL);
        p_b->setPlayerRelationship(p_c, rel_bc);
        p_c->setPlayerRelationship(p_b, rel_bc);

        (p_b->*access_member(PlayerUpgD2Tag())) = UpgradeMaskType();
        (p_c->*access_member(PlayerUpgD2Tag())) = UpgradeMaskType();

        // Draw parameters across all 10 modules (seed 0 curated to hit all primary branches)
        // 1) CarBomb
        const uint32_t cb_target_sel = (seed == 0) ? 0u : (next_u32() % 7u);
        const bool cb_goal_match = (seed == 0) ? true : ((next_u32() % 4u) != 0);
        const uint32_t cb_vet = (seed == 0) ? 2u : (next_u32() % 4u);

        // 2) Hijack
        const uint32_t hj_target_sel = (seed == 0) ? 0u : (next_u32() % 10u);
        const bool hj_goal_match = (seed == 0) ? true : ((next_u32() % 4u) != 0);
        const uint32_t hj_jacker_vet = (seed == 0) ? 2u : (next_u32() % 4u);
        const uint32_t hj_target_vet = (seed == 0) ? 1u : (next_u32() % 4u);

        // 3) FireWeaponCollide
        const uint32_t fw_wpn_mode = (seed == 0) ? 0u : (next_u32() & 1u); // 0=Laser,1=Single (C++ ctor crashes if CollideWeapon is null)
        const bool fw_fire_once = (seed == 0) ? false : ((next_u32() & 1u) != 0);
        const uint32_t fw_req_mode = (seed == 0) ? 0u : (next_u32() % 3u); // 0=none,1=AIRBORNE_TARGET,2=HIJACKED
        const uint32_t fw_forb_mode = (seed == 0) ? 0u : (next_u32() % 3u); // 0=none,1=UNDER_CONSTRUCTION,2=HIJACKED
        const uint32_t fw_status_mode = (seed == 0) ? 0u : (next_u32() % 4u); // 0=none,1=AIRBORNE_TARGET,2=UNDER_CONSTRUCTION,3=HIJACKED
        const bool fw_collide_ground = (seed == 0) ? false : ((next_u32() % 5u) == 0);

        // 4) HealCrateCollide
        const uint32_t hl_picker_mode = (seed == 0) ? 0u : (next_u32() % 3u); // 0=u2(team_c), 1=enemy(team_b), 2=dead(team_c)

        // 5) MoneyCrateCollide
        const UnsignedInt mn_provided = (seed == 0) ? 350u : (next_u32() % 1001u);
        const bool mn_has_boost_cfg = (seed == 0) ? true : ((next_u32() & 1u) != 0);
        const Int mn_boost_amt = (seed == 0) ? 150 : static_cast<Int>(next_u32() % 501u);
        const bool mn_has_upg = (seed == 0) ? true : ((next_u32() & 1u) != 0);
        const UnsignedInt mn_pc_money0 = (seed == 0) ? 100u : (next_u32() % 500u);

        // 6) SalvageCrateCollide
        const uint32_t sv_target_sel = (seed == 0) ? 0u : (next_u32() % 4u);
        const uint32_t sv_pre_armor = (seed == 0) ? 1u : (next_u32() % 3u);
        const uint32_t sv_pre_wpn = (seed == 0) ? 0u : (next_u32() % 3u);
        const uint32_t sv_pre_vet = (seed == 0) ? 0u : (next_u32() % 4u);
        const uint32_t sv_wpn_pct = (seed == 0) ? 50u : ((next_u32() % 3u) * 50u); // 0, 50, 100
        const uint32_t sv_lvl_pct = (seed == 0) ? 50u : ((next_u32() % 3u) * 50u); // 0, 50, 100
        const UnsignedInt sv_min_money = (seed == 0) ? 25u : (next_u32() % 80u);
        const UnsignedInt sv_max_money = sv_min_money + ((seed == 0) ? 50u : (next_u32() % 80u));
        const UnsignedInt sv_rng_seed = next_u32();

        // 7) SquishCollide
        const uint32_t sq_crusher_kind = (seed == 0) ? 1u : (next_u32() % 3u); // 0=Cyl,1=Box,2=NonCrusher
        const uint32_t sq_vel_mode = (seed == 0) ? 0u : (next_u32() % 3u); // 0=toward,1=away,2=zero
        const uint32_t sq_dist_mode = (seed == 0) ? 0u : (next_u32() % 2u); // 0=near,1=far
        const Real sq_angle = (seed == 0) ? 0.5f : (static_cast<Real>(next_u32() % 360u) * (3.14159265f / 180.0f));
        const bool sq_hijack_goal = (seed == 0) ? false : ((next_u32() % 4u) == 0);
        const bool sq_undet_def = (seed == 0) ? false : ((next_u32() & 1u) != 0);
        const bool sq_ground = (seed == 0) ? false : ((next_u32() % 6u) == 0);

        // 8) UnitCrateCollide
        const UnsignedInt un_count = (seed == 0) ? 2u : (next_u32() % 4u);
        const bool un_valid_tmpl = (seed == 0) ? true : ((next_u32() % 5u) != 0);
        const UnsignedInt un_rng_seed = next_u32();

        // 9) VeterancyCrateCollide
        const UnsignedInt vt_range = (seed == 0) ? 120u : ((next_u32() & 1u) ? 0u : (80u + (next_u32() % 100u)));
        const bool vt_adds_owner = (seed == 0) ? true : ((next_u32() & 1u) != 0);
        const bool vt_is_pilot = (seed == 0) ? false : ((next_u32() & 1u) != 0);
        const uint32_t vt_owner_vet = (seed == 0) ? 2u : (next_u32() % 4u);
        const uint32_t vt_target_sel = (seed == 0) ? 0u : (next_u32() % 6u);
        const bool vt_goal_match = (seed == 0) ? true : ((next_u32() % 4u) != 0);

        // Emit 5 dynamic templates per seed with unique names.
        const std::string t_fire = "Cr52_Fire_" + sfx;
        const std::string t_money = "Cr52_Money_" + sfx;
        const std::string t_salv = "Cr52_Salv_" + sfx;
        const std::string t_unit = "Cr52_Unit_" + sfx;
        const std::string t_vet = "Cr52_Vet_" + sfx;

        std::ostringstream dyn_ini;
        dyn_ini << "Object " << t_fire << "\n"
                << "  KindOf = IMMOBILE\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 100\n"
                << "    InitialHealth = 100\n"
                << "  End\n"
                << "  Behavior = FireWeaponCollide ModuleTag_Collide\n";
        if (fw_wpn_mode == 0) dyn_ini << "    CollideWeapon = Wpn_CollideLaser\n";
        else dyn_ini << "    CollideWeapon = Wpn_CollideSingle\n";
        dyn_ini << "    FireOnce = " << (fw_fire_once ? "Yes" : "No") << "\n";
        if (fw_req_mode == 1) dyn_ini << "    RequiredStatus = AIRBORNE_TARGET\n";
        else if (fw_req_mode == 2) dyn_ini << "    RequiredStatus = HIJACKED\n";
        if (fw_forb_mode == 1) dyn_ini << "    ForbiddenStatus = UNDER_CONSTRUCTION\n";
        else if (fw_forb_mode == 2) dyn_ini << "    ForbiddenStatus = HIJACKED\n";
        dyn_ini << "  End\n"
                << "End\n";

        dyn_ini << "Object " << t_money << "\n"
                << "  KindOf = IMMOBILE\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 10\n"
                << "    InitialHealth = 10\n"
                << "  End\n"
                << "  Behavior = MoneyCrateCollide ModuleTag_Collide\n"
                << "    MoneyProvided = " << mn_provided << "\n";
        if (mn_has_boost_cfg) {
            dyn_ini << "    UpgradedBoost = UpgradeType:Upg_MoneyBoost Boost:" << mn_boost_amt << "\n";
        }
        dyn_ini << "    ExecuteFX = FX_CrateExec\n"
                << "  End\n"
                << "End\n";

        dyn_ini << "Object " << t_salv << "\n"
                << "  KindOf = IMMOBILE\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 10\n"
                << "    InitialHealth = 10\n"
                << "  End\n"
                << "  Behavior = SalvageCrateCollide ModuleTag_Collide\n"
                << "    WeaponChance = " << sv_wpn_pct << "%\n"
                << "    LevelChance = " << sv_lvl_pct << "%\n"
                << "    MoneyChance = 75%\n"
                << "    MinMoney = " << sv_min_money << "\n"
                << "    MaxMoney = " << sv_max_money << "\n"
                << "    ExecuteFX = FX_CrateExec\n"
                << "  End\n"
                << "End\n";

        dyn_ini << "Object " << t_unit << "\n"
                << "  KindOf = IMMOBILE\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 10\n"
                << "    InitialHealth = 10\n"
                << "  End\n"
                << "  Behavior = UnitCrateCollide ModuleTag_Collide\n"
                << "    UnitCount = " << un_count << "\n"
                << "    UnitName = " << (un_valid_tmpl ? "Cr52_SpawnedUnit" : "Cr52_DoesNotExist") << "\n"
                << "    ExecuteFX = FX_CrateExec\n"
                << "  End\n"
                << "End\n";

        dyn_ini << "Object " << t_vet << "\n"
                << "  KindOf = INFANTRY\n"
                << "  IsTrainable = Yes\n"
                << "  ExperienceRequired = 0 50 100 200\n"
                << "  Body = ActiveBody ModuleTag_Body\n"
                << "    MaxHealth = 50\n"
                << "    InitialHealth = 50\n"
                << "  End\n"
                << "  Behavior = AIUpdateInterface ModuleTag_AI\n  End\n"
                << "  Behavior = VeterancyCrateCollide ModuleTag_Collide\n"
                << "    EffectRange = " << vt_range << "\n"
                << "    AddsOwnerVeterancy = " << (vt_adds_owner ? "Yes" : "No") << "\n"
                << "    IsPilot = " << (vt_is_pilot ? "Yes" : "No") << "\n"
                << "    ExecuteFX = FX_CrateExec\n"
                << "  End\n"
                << "End\n";

        try {
            w.load_ini(dyn_ini.str(), INI_LOAD_OVERWRITE);
        } catch (...) {
            std::fprintf(stderr, "INI FAIL DYN seed %u:\n%s\n", seed, dyn_ini.str().c_str());
            return 1;
        }

        // =====================================================================
        // 1. ConvertToCarBombCrateCollide (PKT-201)
        // =====================================================================
        Object* cb_crate = w.spawn_object("Cr52_Mod_CarBomb", w.team_b, {10.0f, 10.0f, 0.0f});
        cb_crate->getExperienceTracker()->setVeterancyLevel(kVetLevels[cb_vet], FALSE);
        Object* cb_decoy = w.spawn_object("Cr52_Decoy", team_c, {30.0f, 10.0f, 0.0f});
        Object* cb_tgt = nullptr;
        switch (cb_target_sel) {
            case 0: cb_tgt = w.spawn_object("Cr52_VictimCar", team_c, {10.0f, 10.0f, 0.0f}); break;
            case 1: cb_tgt = w.spawn_object("Cr52_PlainCar", team_c, {10.0f, 10.0f, 0.0f}); break;
            case 2: cb_tgt = w.spawn_object("Cr52_Boat", team_c, {10.0f, 10.0f, 0.0f}); break;
            case 3: cb_tgt = w.spawn_object("Cr52_AirUnit", team_c, {10.0f, 10.0f, 0.0f}); break;
            case 4:
                cb_tgt = w.spawn_object("Cr52_VictimCar", team_c, {10.0f, 10.0f, 0.0f});
                cb_tgt->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_IS_CARBOMB));
                break;
            case 5:
                cb_tgt = w.spawn_object("Cr52_VictimCar", team_c, {10.0f, 10.0f, 0.0f});
                cb_tgt->setWeaponSetFlag(WEAPONSET_CARBOMB);
                break;
            case 6:
            default:
                cb_tgt = w.spawn_object("Cr52_VictimCar", team_c, {10.0f, 10.0f, 0.0f});
                cb_tgt->setEffectivelyDead(TRUE);
                break;
        }
        cb_crate->getAIUpdateInterface()->friend_setGoalObject(cb_goal_match ? cb_tgt : cb_decoy);
        CollideModule* m_cb = find_first_collide(cb_crate);
        const Bool cb_is_cb = m_cb->isCarBombCrateCollide();
        const Bool cb_is_rr = m_cb->isRailroad();
        const Bool cb_would = m_cb->wouldLikeToCollideWith(cb_tgt);
        m_cb->onCollide(cb_tgt, &col_loc, &col_norm);
        const Bool cb_dest = cb_crate->isDestroyed();
        const Bool cb_has_ws = cb_tgt->testWeaponSetFlag(WEAPONSET_CARBOMB);
        const Bool cb_has_st = cb_tgt->getStatusBits().test(OBJECT_STATUS_IS_CARBOMB);
        const Bool cb_team_b = (cb_tgt->getControllingPlayer() == p_b) ? TRUE : FALSE;
        const Int cb_tgt_vet = static_cast<Int>(cb_tgt->getVeterancyLevel());
        const Real cb_tgt_shroud = cb_tgt->getShroudClearingRange();
        const uint32_t cb_crc = oracle_wp51::World::snapshot_crc(m_cb);
        const std::string cb_xfer = oracle_wp51::World::snapshot_save_hex(m_cb);
        oracle_wp51::World::snapshot_load_bytes(m_cb, hex_to_bytes(cb_xfer));
        const std::string cb_rt_xfer = oracle_wp51::World::snapshot_save_hex(m_cb);
        destroy_if_live(cb_crate);
        destroy_if_live(cb_tgt);
        destroy_if_live(cb_decoy);
        w.flush_destroy_list();

        // =====================================================================
        // 2. ConvertToHijackedVehicleCrateCollide (PKT-202)
        // =====================================================================
        Object* hj_crate = w.spawn_object("Cr52_Mod_Hijack", w.team_b, {20.0f, 10.0f, 0.0f});
        hj_crate->getExperienceTracker()->setVeterancyLevel(kVetLevels[hj_jacker_vet], FALSE);
        Object* hj_decoy = w.spawn_object("Cr52_Decoy", team_c, {40.0f, 10.0f, 0.0f});
        Object* hj_pass = nullptr;
        Object* hj_tgt = nullptr;
        switch (hj_target_sel) {
            case 0: hj_tgt = w.spawn_object("Cr52_BoxTank", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 1: hj_tgt = w.spawn_object("Cr52_PlainCar", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 2: hj_tgt = w.spawn_object("Cr52_Transport", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 3:
                hj_tgt = w.spawn_object("Cr52_Transport", team_c, {20.0f, 10.0f, 0.0f});
                hj_pass = w.spawn_object("Cr52_Decoy", team_c, {20.0f, 10.0f, 0.0f});
                hj_tgt->getContain()->addToContain(hj_pass);
                break;
            case 4: hj_tgt = w.spawn_object("Cr52_Immune", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 5: hj_tgt = w.spawn_object("Cr52_AirUnit", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 6: hj_tgt = w.spawn_object("Cr52_Boat", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 7: hj_tgt = w.spawn_object("Cr52_Drone", team_c, {20.0f, 10.0f, 0.0f}); break;
            case 8:
                hj_tgt = w.spawn_object("Cr52_BoxTank", team_c, {20.0f, 10.0f, 0.0f});
                hj_tgt->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_HIJACKED));
                break;
            case 9:
            default:
                hj_tgt = w.spawn_object("Cr52_BoxTank", team_c, {20.0f, 10.0f, 0.0f});
                hj_tgt->setEffectivelyDead(TRUE);
                break;
        }
        hj_tgt->getExperienceTracker()->setVeterancyLevel(kVetLevels[hj_target_vet], FALSE);
        hj_crate->getAIUpdateInterface()->friend_setGoalObject(hj_goal_match ? hj_tgt : hj_decoy);
        CollideModule* m_hj = find_first_collide(hj_crate);
        const Bool hj_is_hj = m_hj->isHijackedVehicleCrateCollide();
        const Bool hj_would = m_hj->wouldLikeToCollideWith(hj_tgt);
        m_hj->onCollide(hj_tgt, &col_loc, &col_norm);
        const Bool hj_dest = hj_crate->isDestroyed();
        const Bool hj_has_st = hj_tgt->getStatusBits().test(OBJECT_STATUS_HIJACKED);
        const Bool hj_team_b = (hj_tgt->getControllingPlayer() == p_b) ? TRUE : FALSE;
        const Int hj_tgt_vet_after = static_cast<Int>(hj_tgt->getVeterancyLevel());
        const Int hj_jkr_vet_after = static_cast<Int>(hj_crate->getVeterancyLevel());
        const Bool hj_jkr_masked = hj_crate->getStatusBits().test(OBJECT_STATUS_MASKED);
        const Real hj_tgt_shroud = hj_tgt->getShroudClearingRange();
        const uint32_t hj_crc = oracle_wp51::World::snapshot_crc(m_hj);
        const std::string hj_xfer = oracle_wp51::World::snapshot_save_hex(m_hj);
        destroy_if_live(hj_pass);
        destroy_if_live(hj_crate);
        destroy_if_live(hj_tgt);
        destroy_if_live(hj_decoy);
        w.flush_destroy_list();

        // =====================================================================
        // 3. FireWeaponCollide (PKT-215)
        // =====================================================================
        Object* fw_obj = w.spawn_object(t_fire.c_str(), w.team_b, {30.0f, 10.0f, 0.0f});
        Object* fw_tgt = w.spawn_object("Cr52_PlainCar", team_c, {30.0f, 10.0f, 0.0f});
        if (fw_status_mode == 1) fw_obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_AIRBORNE_TARGET));
        else if (fw_status_mode == 2) fw_obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_UNDER_CONSTRUCTION));
        else if (fw_status_mode == 3) fw_obj->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_HIJACKED));
        CollideModule* m_fw = find_first_collide(fw_obj);
        FireWeaponCollide* fw_typed = static_cast<FireWeaponCollide*>(m_fw);
        const Bool fw_has_wpn = (fw_typed->*access_member(FireCollideWpnTag()) != nullptr) ? TRUE : FALSE;
        // Avoid C++ nullptr dereference when CollideWeapon is omitted and shouldFireWeapon() would be TRUE.
        Object* fw_arg = (fw_collide_ground || !fw_has_wpn) ? nullptr : fw_tgt;
        m_fw->onCollide(fw_arg, &col_loc, &col_norm);
        w.set_frame(frame0 + 5u);
        m_fw->onCollide(fw_arg, &col_loc, &col_norm);
        Weapon* fw_w = fw_typed->*access_member(FireCollideWpnTag());
        const UnsignedInt fw_ammo = fw_w ? fw_w->getRemainingAmmo() : 0u;
        const Int fw_wstatus = fw_w ? static_cast<Int>(fw_w->getStatus()) : -1;
        const UnsignedInt fw_next_fire = fw_w ? fw_w->getPossibleNextShotFrame() : 0u;
        const UnsignedInt fw_last_fire = fw_w ? fw_w->getLastShotFrame() : 0u;
        const Bool fw_ever = fw_typed->*access_member(FireCollideEverTag());
        const uint32_t fw_crc = oracle_wp51::World::snapshot_crc(m_fw);
        const std::string fw_xfer = oracle_wp51::World::snapshot_save_hex(m_fw);
        oracle_wp51::World::snapshot_load_bytes(m_fw, hex_to_bytes(fw_xfer));
        const std::string fw_rt_xfer = oracle_wp51::World::snapshot_save_hex(m_fw);
        destroy_if_live(fw_obj);
        destroy_if_live(fw_tgt);
        w.flush_destroy_list();
        w.set_frame(frame0);

        // =====================================================================
        // 4. HealCrateCollide (PKT-219)
        // =====================================================================
        Object* hl_crate = w.spawn_object("Cr52_Mod_Heal", w.team_b, {40.0f, 10.0f, 0.0f});
        Object* hl_u1 = w.spawn_object("Cr52_HealInf", team_c, {41.0f, 10.0f, 0.0f});
        Object* hl_u2 = w.spawn_object("Cr52_PlainCar", team_c, {42.0f, 10.0f, 0.0f});
        Object* hl_u3 = w.spawn_object("Cr52_HealBldg", team_c, {43.0f, 10.0f, 0.0f});
        Object* hl_u4 = w.spawn_object("Cr52_HealInf", team_c, {44.0f, 10.0f, 0.0f});
        Object* hl_en = w.spawn_object("Cr52_HealInf", w.team_b, {45.0f, 10.0f, 0.0f});
        auto dmg_to = [](Object* o, Real dmg) {
            DamageInfo d;
            d.in.m_damageType = DAMAGE_UNRESISTABLE;
            d.in.m_deathType = DEATH_NONE;
            d.in.m_amount = dmg;
            o->attemptDamage(&d);
        };
        dmg_to(hl_u1, 30.0f);
        dmg_to(hl_u2, 120.0f);
        dmg_to(hl_u3, 100.0f);
        dmg_to(hl_u4, 30.0f);
        hl_u4->setEffectivelyDead(TRUE);
        dmg_to(hl_en, 25.0f);
        Object* hl_picker = (hl_picker_mode == 0) ? hl_u2 : ((hl_picker_mode == 1) ? hl_en : hl_u4);
        CollideModule* m_hl = find_first_collide(hl_crate);
        const Bool hl_would = m_hl->wouldLikeToCollideWith(hl_picker);
        m_hl->onCollide(hl_picker, &col_loc, &col_norm);
        const Bool hl_dest = hl_crate->isDestroyed();
        const Real hl_u1_hp = hl_u1->getBodyModule()->getHealth();
        const Real hl_u2_hp = hl_u2->getBodyModule()->getHealth();
        const Real hl_u3_hp = hl_u3->getBodyModule()->getHealth();
        const Real hl_u4_hp = hl_u4->getBodyModule()->getHealth();
        const Real hl_en_hp = hl_en->getBodyModule()->getHealth();
        const uint32_t hl_crc = oracle_wp51::World::snapshot_crc(m_hl);
        const std::string hl_xfer = oracle_wp51::World::snapshot_save_hex(m_hl);
        destroy_if_live(hl_crate);
        destroy_if_live(hl_u1);
        destroy_if_live(hl_u2);
        destroy_if_live(hl_u3);
        destroy_if_live(hl_u4);
        destroy_if_live(hl_en);
        w.flush_destroy_list();

        // =====================================================================
        // 5. MoneyCrateCollide (PKT-224)
        // =====================================================================
        set_player_money(p_c, mn_pc_money0);
        if (mn_has_upg && ut_boost) {
            (p_c->*access_member(PlayerUpgD2Tag())).set(ut_boost->getUpgradeMask());
        }
        Object* mn_crate = w.spawn_object(t_money.c_str(), w.team_b, {50.0f, 10.0f, 0.0f});
        Object* mn_picker = w.spawn_object("Cr52_PlainCar", team_c, {50.0f, 10.0f, 0.0f});
        CollideModule* m_mn = find_first_collide(mn_crate);
        const Bool mn_would = m_mn->wouldLikeToCollideWith(mn_picker);
        m_mn->onCollide(mn_picker, &col_loc, &col_norm);
        const Bool mn_dest = mn_crate->isDestroyed();
        const UnsignedInt mn_money_after = p_c->getMoney()->countMoney();
        const uint32_t mn_crc = oracle_wp51::World::snapshot_crc(m_mn);
        const std::string mn_xfer = oracle_wp51::World::snapshot_save_hex(m_mn);
        destroy_if_live(mn_crate);
        destroy_if_live(mn_picker);
        w.flush_destroy_list();

        // =====================================================================
        // 6. SalvageCrateCollide (PKT-240)
        // =====================================================================
        set_player_money(p_c, 0u);
        Object* sv_crate = w.spawn_object(t_salv.c_str(), w.team_b, {60.0f, 10.0f, 0.0f});
        const char* sv_tmpl = (sv_target_sel == 0) ? "Cr52_SalvagerAll"
                            : (sv_target_sel == 1) ? "Cr52_SalvagerWpn"
                            : (sv_target_sel == 2) ? "Cr52_SalvagerPlain"
                                                   : "Cr52_PlainCar";
        Object* sv_picker = w.spawn_object(sv_tmpl, team_c, {60.0f, 10.0f, 0.0f});
        if (sv_pre_armor == 1) {
            sv_picker->setArmorSetFlag(ARMORSET_CRATE_UPGRADE_ONE);
        } else if (sv_pre_armor == 2) {
            sv_picker->setArmorSetFlag(ARMORSET_CRATE_UPGRADE_TWO);
        }
        if (sv_pre_wpn == 1) {
            sv_picker->setWeaponSetFlag(WEAPONSET_CRATEUPGRADE_ONE);
        } else if (sv_pre_wpn == 2) {
            sv_picker->setWeaponSetFlag(WEAPONSET_CRATEUPGRADE_TWO);
        }
        sv_picker->getExperienceTracker()->setVeterancyLevel(kVetLevels[sv_pre_vet], FALSE);
        CollideModule* m_sv = find_first_collide(sv_crate);
        const Bool sv_is_sv = m_sv->isSalvageCrateCollide();
        const Bool sv_would = m_sv->wouldLikeToCollideWith(sv_picker);
        InitRandom(sv_rng_seed);
        m_sv->onCollide(sv_picker, &col_loc, &col_norm);
        const UnsignedInt sv_post_rng_crc = GetGameLogicRandomSeedCRC();
        const Bool sv_dest = sv_crate->isDestroyed();
        const uint32_t sv_armor_mask =
            (sv_picker->testArmorSetFlag(ARMORSET_CRATE_UPGRADE_ONE) ? 1u : 0u) |
            (sv_picker->testArmorSetFlag(ARMORSET_CRATE_UPGRADE_TWO) ? 2u : 0u);
        const uint32_t sv_wpn_mask =
            (sv_picker->testWeaponSetFlag(WEAPONSET_CRATEUPGRADE_ONE) ? 1u : 0u) |
            (sv_picker->testWeaponSetFlag(WEAPONSET_CRATEUPGRADE_TWO) ? 2u : 0u);
        const Int sv_vet_after = static_cast<Int>(sv_picker->getVeterancyLevel());
        const Int sv_exp_after = sv_picker->getExperienceTracker()->getCurrentExperience();
        const UnsignedInt sv_money_after = p_c->getMoney()->countMoney();
        const uint32_t sv_crc = oracle_wp51::World::snapshot_crc(m_sv);
        const std::string sv_xfer = oracle_wp51::World::snapshot_save_hex(m_sv);
        destroy_if_live(sv_crate);
        destroy_if_live(sv_picker);
        w.flush_destroy_list();

        // =====================================================================
        // 7. ShroudCrateCollide (PKT-241)
        // =====================================================================
        Object* sh_crate = w.spawn_object("Cr52_Mod_Shroud", w.team_b, {70.0f, 10.0f, 0.0f});
        Object* sh_picker = w.spawn_object("Cr52_PlainCar", team_c, {70.0f, 10.0f, 0.0f});
        CollideModule* m_sh = find_first_collide(sh_crate);
        const Bool sh_would = m_sh->wouldLikeToCollideWith(sh_picker);
        m_sh->onCollide(sh_picker, &col_loc, &col_norm);
        const Bool sh_dest = sh_crate->isDestroyed();
        const uint32_t sh_crc = oracle_wp51::World::snapshot_crc(m_sh);
        const std::string sh_xfer = oracle_wp51::World::snapshot_save_hex(m_sh);
        destroy_if_live(sh_crate);
        destroy_if_live(sh_picker);
        w.flush_destroy_list();

        // =====================================================================
        // 8. SquishCollide (PKT-246)
        // =====================================================================
        const Coord3D sq_self_pos = {100.0f, 100.0f, 0.0f};
        const Coord3D sq_crusher_pos = (sq_dist_mode == 0)
            ? Coord3D{92.0f, 96.0f, 0.0f}
            : Coord3D{50.0f, 50.0f, 0.0f};
        Object* sq_self = w.spawn_object(
            sq_hijack_goal ? "Cr52_Mod_SquishHijacker" : "Cr52_Mod_Squish",
            w.team_b,
            sq_self_pos
        );
        if (sq_undet_def) sq_self->friend_setUndetectedDefector(TRUE);
        const char* sq_cr_tmpl = (sq_crusher_kind == 0) ? "Cr52_PlainCar"
                               : (sq_crusher_kind == 1) ? "Cr52_BoxTank"
                                                        : "Cr52_NonCrusherVeh";
        Object* sq_crusher = w.spawn_object(sq_cr_tmpl, team_c, sq_crusher_pos, sq_angle);
        Object* sq_decoy = w.spawn_object("Cr52_Decoy", team_c, {120.0f, 100.0f, 0.0f});
        if (sq_hijack_goal) {
            sq_self->getAIUpdateInterface()->friend_setGoalObject(sq_crusher);
        } else {
            sq_self->getAIUpdateInterface()->friend_setGoalObject(sq_decoy);
        }
        if (PhysicsBehavior* phys = sq_crusher->getPhysics()) {
            Coord3D vel = {0.0f, 0.0f, 0.0f};
            if (sq_vel_mode == 0) { vel = {8.0f, 4.0f, 0.0f}; }
            else if (sq_vel_mode == 1) { vel = {-8.0f, -4.0f, 0.0f}; }
            phys->addVelocityTo(&vel);
        }
        CollideModule* m_sq = find_first_collide(sq_self);
        m_sq->onCollide(sq_ground ? nullptr : sq_crusher, &col_loc, &col_norm);
        const Bool sq_dead = sq_self->isEffectivelyDead();
        const Real sq_hp = sq_self->getBodyModule()->getHealth();
        const Bool sq_def = sq_self->getIsUndetectedDefector();
        const uint32_t sq_crc = oracle_wp51::World::snapshot_crc(m_sq);
        const std::string sq_xfer = oracle_wp51::World::snapshot_save_hex(m_sq);
        destroy_if_live(sq_self);
        destroy_if_live(sq_crusher);
        destroy_if_live(sq_decoy);
        w.flush_destroy_list();

        // =====================================================================
        // 9. UnitCrateCollide (PKT-250)
        // =====================================================================
        Object* un_crate = w.spawn_object(t_unit.c_str(), w.team_b, {200.0f, 200.0f, 0.0f});
        Object* un_picker = w.spawn_object("Cr52_PlainCar", team_c, {200.0f, 200.0f, 0.0f}, 0.75f);
        std::unordered_set<Object*> un_before;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            un_before.insert(o);
        }
        CollideModule* m_un = find_first_collide(un_crate);
        const Bool un_would = m_un->wouldLikeToCollideWith(un_picker);
        InitRandom(un_rng_seed);
        m_un->onCollide(un_picker, &col_loc, &col_norm);
        const UnsignedInt un_post_rng_crc = GetGameLogicRandomSeedCRC();
        const Bool un_dest = un_crate->isDestroyed();
        uint32_t un_spawned = 0u;
        Coord3D un_first_pos = {0.0f, 0.0f, 0.0f};
        std::vector<Object*> un_created;
        for (Object* o = TheGameLogic->getFirstObject(); o; o = o->getNextObject()) {
            if (un_before.find(o) == un_before.end() && !o->isDestroyed()) {
                if (un_spawned == 0u) un_first_pos = *o->getPosition();
                ++un_spawned;
                un_created.push_back(o);
            }
        }
        const uint32_t un_crc = oracle_wp51::World::snapshot_crc(m_un);
        const std::string un_xfer = oracle_wp51::World::snapshot_save_hex(m_un);
        for (Object* o : un_created) destroy_if_live(o);
        destroy_if_live(un_crate);
        destroy_if_live(un_picker);
        w.flush_destroy_list();

        // =====================================================================
        // 10. VeterancyCrateCollide (PKT-254)
        // =====================================================================
        Object* vt_crate = w.spawn_object(t_vet.c_str(), w.team_b, {300.0f, 300.0f, 0.0f});
        vt_crate->getExperienceTracker()->setVeterancyLevel(kVetLevels[vt_owner_vet], FALSE);
        Object* vt_decoy = w.spawn_object("Cr52_Decoy", team_c, {500.0f, 500.0f, 0.0f});
        Object* vt_tgt = nullptr;
        switch (vt_target_sel) {
            case 0: vt_tgt = w.spawn_object("Cr52_TrainableVeh", w.team_b, {300.0f, 300.0f, 0.0f}); break;
            case 1: vt_tgt = w.spawn_object("Cr52_TrainableVeh", team_c, {300.0f, 300.0f, 0.0f}); break;
            case 2: vt_tgt = w.spawn_object("Cr52_AirUnit", w.team_b, {300.0f, 300.0f, 0.0f}); break;
            case 3: vt_tgt = w.spawn_object("Cr52_NonTrainableVeh", w.team_b, {300.0f, 300.0f, 0.0f}); break;
            case 4: vt_tgt = w.spawn_object("Cr52_TrainableVeh", w.team_b, {300.0f, 300.0f, 40.0f}); break;
            case 5:
            default:
                vt_tgt = w.spawn_object("Cr52_TrainableVeh", w.team_b, {300.0f, 300.0f, 0.0f});
                vt_tgt->getExperienceTracker()->setVeterancyLevel(LEVEL_HEROIC, FALSE);
                break;
        }
        Team* vt_tgt_team = vt_tgt->getTeam();
        Team* vt_other_team = (vt_tgt_team == w.team_b) ? team_c : w.team_b;
        Object* vt_in = w.spawn_object("Cr52_TrainableVeh", vt_tgt_team, {350.0f, 300.0f, 0.0f});
        Object* vt_masked = w.spawn_object("Cr52_TrainableVeh", vt_tgt_team, {340.0f, 300.0f, 0.0f});
        vt_masked->setStatus(MAKE_OBJECT_STATUS_MASK(OBJECT_STATUS_MASKED));
        Object* vt_out = w.spawn_object("Cr52_TrainableVeh", vt_tgt_team, {600.0f, 300.0f, 0.0f});
        Object* vt_enemy = w.spawn_object("Cr52_TrainableVeh", vt_other_team, {320.0f, 300.0f, 0.0f});

        ThePartitionManager->update();
        vt_crate->getAIUpdateInterface()->friend_setGoalObject(vt_goal_match ? vt_tgt : vt_decoy);
        CollideModule* m_vt = find_first_collide(vt_crate);
        const Bool vt_would = m_vt->wouldLikeToCollideWith(vt_tgt);
        m_vt->onCollide(vt_tgt, &col_loc, &col_norm);
        const Bool vt_dest = vt_crate->isDestroyed();
        const Int vt_tgt_vet = static_cast<Int>(vt_tgt->getVeterancyLevel());
        const Int vt_tgt_exp = vt_tgt->getExperienceTracker()->getCurrentExperience();
        const Int vt_in_vet = static_cast<Int>(vt_in->getVeterancyLevel());
        const Int vt_in_exp = vt_in->getExperienceTracker()->getCurrentExperience();
        const Int vt_masked_vet = static_cast<Int>(vt_masked->getVeterancyLevel());
        const Int vt_out_vet = static_cast<Int>(vt_out->getVeterancyLevel());
        const Int vt_enemy_vet = static_cast<Int>(vt_enemy->getVeterancyLevel());
        const uint32_t vt_crc = oracle_wp51::World::snapshot_crc(m_vt);
        const std::string vt_xfer = oracle_wp51::World::snapshot_save_hex(m_vt);
        destroy_if_live(vt_crate);
        destroy_if_live(vt_decoy);
        destroy_if_live(vt_tgt);
        destroy_if_live(vt_in);
        destroy_if_live(vt_masked);
        destroy_if_live(vt_out);
        destroy_if_live(vt_enemy);
        w.flush_destroy_list();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"cb\":{\"is_cb\":" << (cb_is_cb ? 1 : 0) << ",\"is_rr\":" << (cb_is_rr ? 1 : 0)
              << ",\"would\":" << (cb_would ? 1 : 0) << ",\"dest\":" << (cb_dest ? 1 : 0)
              << ",\"ws\":" << (cb_has_ws ? 1 : 0) << ",\"st\":" << (cb_has_st ? 1 : 0)
              << ",\"team_b\":" << (cb_team_b ? 1 : 0) << ",\"vet\":" << cb_tgt_vet
              << ",\"shroud_bits\":" << oracle::fbits(cb_tgt_shroud)
              << ",\"xfer\":\"" << cb_xfer << "\",\"rt_xfer\":\"" << cb_rt_xfer << "\"},"
              << "\"hj\":{\"is_hj\":" << (hj_is_hj ? 1 : 0) << ",\"would\":" << (hj_would ? 1 : 0)
              << ",\"dest\":" << (hj_dest ? 1 : 0) << ",\"st\":" << (hj_has_st ? 1 : 0)
              << ",\"team_b\":" << (hj_team_b ? 1 : 0) << ",\"tgt_vet\":" << hj_tgt_vet_after
              << ",\"jkr_vet\":" << hj_jkr_vet_after << ",\"masked\":" << (hj_jkr_masked ? 1 : 0)
              << ",\"shroud_bits\":" << oracle::fbits(hj_tgt_shroud)
              << ",\"xfer\":\"" << hj_xfer << "\"},"
              << "\"fw\":{\"ammo\":" << fw_ammo << ",\"wstatus\":" << fw_wstatus
              << ",\"next_fire\":" << fw_next_fire << ",\"last_fire\":" << fw_last_fire
              << ",\"ever\":" << (fw_ever ? 1 : 0)
              << ",\"xfer\":\"" << fw_xfer << "\",\"rt_xfer\":\"" << fw_rt_xfer << "\"},"
              << "\"hl\":{\"would\":" << (hl_would ? 1 : 0) << ",\"dest\":" << (hl_dest ? 1 : 0)
              << ",\"u1\":" << oracle::fbits(hl_u1_hp) << ",\"u2\":" << oracle::fbits(hl_u2_hp)
              << ",\"u3\":" << oracle::fbits(hl_u3_hp) << ",\"u4\":" << oracle::fbits(hl_u4_hp)
              << ",\"en\":" << oracle::fbits(hl_en_hp) << ",\"xfer\":\"" << hl_xfer << "\"},"
              << "\"mn\":{\"would\":" << (mn_would ? 1 : 0) << ",\"dest\":" << (mn_dest ? 1 : 0)
              << ",\"money\":" << mn_money_after << ",\"xfer\":\"" << mn_xfer << "\"},"
              << "\"sv\":{\"is_sv\":" << (sv_is_sv ? 1 : 0) << ",\"would\":" << (sv_would ? 1 : 0)
              << ",\"dest\":" << (sv_dest ? 1 : 0) << ",\"armor\":" << sv_armor_mask
              << ",\"wpn\":" << sv_wpn_mask << ",\"vet\":" << sv_vet_after
              << ",\"exp\":" << sv_exp_after << ",\"money\":" << sv_money_after
              << ",\"rng_crc\":" << sv_post_rng_crc << ",\"xfer\":\"" << sv_xfer << "\"},"
              << "\"sh\":{\"would\":" << (sh_would ? 1 : 0) << ",\"dest\":" << (sh_dest ? 1 : 0)
              << ",\"xfer\":\"" << sh_xfer << "\"},"
              << "\"sq\":{\"dead\":" << (sq_dead ? 1 : 0) << ",\"hp\":" << oracle::fbits(sq_hp)
              << ",\"def\":" << (sq_def ? 1 : 0) << ",\"xfer\":\"" << sq_xfer << "\"},"
              << "\"un\":{\"would\":" << (un_would ? 1 : 0) << ",\"dest\":" << (un_dest ? 1 : 0)
              << ",\"spawned\":" << un_spawned
              << ",\"px\":" << oracle::fbits(un_first_pos.x)
              << ",\"py\":" << oracle::fbits(un_first_pos.y)
              << ",\"rng_crc\":" << un_post_rng_crc << ",\"xfer\":\"" << un_xfer << "\"},"
              << "\"vt\":{\"would\":" << (vt_would ? 1 : 0) << ",\"dest\":" << (vt_dest ? 1 : 0)
              << ",\"tgt_vet\":" << vt_tgt_vet << ",\"tgt_exp\":" << vt_tgt_exp
              << ",\"in_vet\":" << vt_in_vet << ",\"in_exp\":" << vt_in_exp
              << ",\"masked_vet\":" << vt_masked_vet << ",\"out_vet\":" << vt_out_vet
              << ",\"enemy_vet\":" << vt_enemy_vet << ",\"xfer\":\"" << vt_xfer << "\"}"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{"
             << "\"frame\":" << frame0
             << ",\"rel_bc\":" << static_cast<int>(rel_bc)
             << ",\"cb_sel\":" << cb_target_sel
             << ",\"cb_goal\":" << (cb_goal_match ? 1 : 0)
             << ",\"cb_vet\":" << cb_vet
             << ",\"hj_sel\":" << hj_target_sel
             << ",\"hj_goal\":" << (hj_goal_match ? 1 : 0)
             << ",\"hj_jvet\":" << hj_jacker_vet
             << ",\"hj_tvet\":" << hj_target_vet
             << ",\"fw_wpn\":" << fw_wpn_mode
             << ",\"fw_once\":" << (fw_fire_once ? 1 : 0)
             << ",\"fw_req\":" << fw_req_mode
             << ",\"fw_forb\":" << fw_forb_mode
             << ",\"fw_st\":" << fw_status_mode
             << ",\"fw_gnd\":" << (fw_collide_ground ? 1 : 0)
             << ",\"hl_pick\":" << hl_picker_mode
             << ",\"mn_prov\":" << mn_provided
             << ",\"mn_bcfg\":" << (mn_has_boost_cfg ? 1 : 0)
             << ",\"mn_bamt\":" << mn_boost_amt
             << ",\"mn_upg\":" << (mn_has_upg ? 1 : 0)
             << ",\"mn_m0\":" << mn_pc_money0
             << ",\"sv_sel\":" << sv_target_sel
             << ",\"sv_arm\":" << sv_pre_armor
             << ",\"sv_wpn\":" << sv_pre_wpn
             << ",\"sv_vet\":" << sv_pre_vet
             << ",\"sv_wpct\":" << sv_wpn_pct
             << ",\"sv_lpct\":" << sv_lvl_pct
             << ",\"sv_min\":" << sv_min_money
             << ",\"sv_max\":" << sv_max_money
             << ",\"sv_rng\":" << sv_rng_seed
             << ",\"sq_kind\":" << sq_crusher_kind
             << ",\"sq_vel\":" << sq_vel_mode
             << ",\"sq_dist\":" << sq_dist_mode
             << ",\"sq_ang\":" << oracle::fbits(sq_angle)
             << ",\"sq_hjk\":" << (sq_hijack_goal ? 1 : 0)
             << ",\"sq_def\":" << (sq_undet_def ? 1 : 0)
             << ",\"sq_gnd\":" << (sq_ground ? 1 : 0)
             << ",\"un_cnt\":" << un_count
             << ",\"un_ok\":" << (un_valid_tmpl ? 1 : 0)
             << ",\"un_rng\":" << un_rng_seed
             << ",\"vt_rng\":" << vt_range
             << ",\"vt_add\":" << (vt_adds_owner ? 1 : 0)
             << ",\"vt_plt\":" << (vt_is_pilot ? 1 : 0)
             << ",\"vt_ovet\":" << vt_owner_vet
             << ",\"vt_sel\":" << vt_target_sel
             << ",\"vt_goal\":" << (vt_goal_match ? 1 : 0)
             << "}"
             << ",\"out\":{"
             << "\"cb\":[" << (cb_is_cb ? 1 : 0) << "," << (cb_is_rr ? 1 : 0) << ","
             << (cb_would ? 1 : 0) << "," << (cb_dest ? 1 : 0) << ","
             << (cb_has_ws ? 1 : 0) << "," << (cb_has_st ? 1 : 0) << ","
             << (cb_team_b ? 1 : 0) << "," << cb_tgt_vet << "," << oracle::fbits(cb_tgt_shroud) << "]"
             << ",\"cb_crc\":" << cb_crc << ",\"cb_xfer\":\"" << cb_xfer << "\",\"cb_rt\":\"" << cb_rt_xfer << "\""
             << ",\"hj\":[" << (hj_is_hj ? 1 : 0) << "," << (hj_would ? 1 : 0) << ","
             << (hj_dest ? 1 : 0) << "," << (hj_has_st ? 1 : 0) << ","
             << (hj_team_b ? 1 : 0) << "," << hj_tgt_vet_after << "," << hj_jkr_vet_after << ","
             << (hj_jkr_masked ? 1 : 0) << "," << oracle::fbits(hj_tgt_shroud) << "]"
             << ",\"hj_crc\":" << hj_crc << ",\"hj_xfer\":\"" << hj_xfer << "\""
             << ",\"fw\":[" << fw_ammo << "," << fw_wstatus << ","
             << fw_next_fire << "," << fw_last_fire << "," << (fw_ever ? 1 : 0) << "]"
             << ",\"fw_crc\":" << fw_crc << ",\"fw_xfer\":\"" << fw_xfer << "\",\"fw_rt\":\"" << fw_rt_xfer << "\""
             << ",\"hl\":[" << (hl_would ? 1 : 0) << "," << (hl_dest ? 1 : 0) << ","
             << oracle::fbits(hl_u1_hp) << "," << oracle::fbits(hl_u2_hp) << ","
             << oracle::fbits(hl_u3_hp) << "," << oracle::fbits(hl_u4_hp) << ","
             << oracle::fbits(hl_en_hp) << "]"
             << ",\"hl_crc\":" << hl_crc << ",\"hl_xfer\":\"" << hl_xfer << "\""
             << ",\"mn\":[" << (mn_would ? 1 : 0) << "," << (mn_dest ? 1 : 0) << ","
             << mn_money_after << "]"
             << ",\"mn_crc\":" << mn_crc << ",\"mn_xfer\":\"" << mn_xfer << "\""
             << ",\"sv\":[" << (sv_is_sv ? 1 : 0) << "," << (sv_would ? 1 : 0) << ","
             << (sv_dest ? 1 : 0) << "," << sv_armor_mask << "," << sv_wpn_mask << ","
             << sv_vet_after << "," << sv_exp_after << "," << sv_money_after << ","
             << sv_post_rng_crc << "]"
             << ",\"sv_crc\":" << sv_crc << ",\"sv_xfer\":\"" << sv_xfer << "\""
             << ",\"sh\":[" << (sh_would ? 1 : 0) << "," << (sh_dest ? 1 : 0) << "]"
             << ",\"sh_crc\":" << sh_crc << ",\"sh_xfer\":\"" << sh_xfer << "\""
             << ",\"sq\":[" << (sq_dead ? 1 : 0) << "," << oracle::fbits(sq_hp) << ","
             << (sq_def ? 1 : 0) << "]"
             << ",\"sq_crc\":" << sq_crc << ",\"sq_xfer\":\"" << sq_xfer << "\""
             << ",\"un\":[" << (un_would ? 1 : 0) << "," << (un_dest ? 1 : 0) << ","
             << un_spawned << "," << oracle::fbits(un_first_pos.x) << ","
             << oracle::fbits(un_first_pos.y) << "," << un_post_rng_crc << "]"
             << ",\"un_crc\":" << un_crc << ",\"un_xfer\":\"" << un_xfer << "\""
             << ",\"vt\":[" << (vt_would ? 1 : 0) << "," << (vt_dest ? 1 : 0) << ","
             << vt_tgt_vet << "," << vt_tgt_exp << "," << vt_in_vet << "," << vt_in_exp << ","
             << vt_masked_vet << "," << vt_out_vet << "," << vt_enemy_vet << "]"
             << ",\"vt_crc\":" << vt_crc << ",\"vt_xfer\":\"" << vt_xfer << "\""
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.2-D2\",\n"
        << "  \"modules\": [\"ConvertToCarBombCrateCollide\",\"ConvertToHijackedVehicleCrateCollide\",\"FireWeaponCollide\",\"HealCrateCollide\",\"MoneyCrateCollide\",\"SalvageCrateCollide\",\"ShroudCrateCollide\",\"SquishCollide\",\"UnitCrateCollide\",\"VeterancyCrateCollide\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_sha256\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Collide/CrateCollide/CrateCollide.cpp") << "\",\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
