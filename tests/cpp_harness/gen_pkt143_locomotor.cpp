// PKT-143 (Locomotor) & PKT-144 (LocomotorSet) (WP-3.3) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against pre-built vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & run command:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt143_locomotor.cpp -o /tmp/gen_pkt143
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pkt143 docs/golden_vectors/PKT-143.json
//
// Determinism & UB verification builds (must emit byte-identical JSON):
//   --opt O0, --opt O1, --opt O2, and --sanitize (UBSan).

#include "oracle_common.h"

#include <sstream>
#include <unistd.h>

#include "Common/GlobalData.h"
#include "Common/NameKeyGenerator.h"
#include "Common/RandomValue.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Locomotor.h"
#include "GameLogic/LocomotorSet.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/ScriptEngine.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_pkt143_locomotor")

struct GLFrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<GLFrameTag, &GameLogic::m_frame>;
UnsignedInt GameLogic::* access_member(GLFrameTag);

struct SELetterBoxTag { using type = Bool ScriptEngine::*; };
template struct MemberAccess<SELetterBoxTag, &ScriptEngine::m_letterBoxActive>;
Bool ScriptEngine::* access_member(SELetterBoxTag);

struct LocoMaintainTag { using type = Coord3D Locomotor::*; };
template struct MemberAccess<LocoMaintainTag, &Locomotor::m_maintainPos>;
Coord3D Locomotor::* access_member(LocoMaintainTag);

struct LocoFlagsTag { using type = UnsignedInt Locomotor::*; };
template struct MemberAccess<LocoFlagsTag, &Locomotor::m_flags>;
UnsignedInt Locomotor::* access_member(LocoFlagsTag);

struct LocoAngleOffTag { using type = Real Locomotor::*; };
template struct MemberAccess<LocoAngleOffTag, &Locomotor::m_angleOffset>;
Real Locomotor::* access_member(LocoAngleOffTag);

struct LocoOffIncTag { using type = Real Locomotor::*; };
template struct MemberAccess<LocoOffIncTag, &Locomotor::m_offsetIncrement>;
Real Locomotor::* access_member(LocoOffIncTag);

struct LocoDonutTag { using type = UnsignedInt Locomotor::*; };
template struct MemberAccess<LocoDonutTag, &Locomotor::m_donutTimer>;
UnsignedInt Locomotor::* access_member(LocoDonutTag);

struct SetLocosTag { using type = LocomotorVector LocomotorSet::*; };
template struct MemberAccess<SetLocosTag, &LocomotorSet::m_locomotors>;
LocomotorVector LocomotorSet::* access_member(SetLocosTag);

#define DECL_TMPL_TAG(Name, Ty, Field) \
    struct Name { using type = Ty LocomotorTemplate::*; }; \
    template struct MemberAccess<Name, &LocomotorTemplate::Field>; \
    Ty LocomotorTemplate::* access_member(Name);

DECL_TMPL_TAG(TSurfTag, LocomotorSurfaceTypeMask, m_surfaces)
DECL_TMPL_TAG(TMaxSpdTag, Real, m_maxSpeed)
DECL_TMPL_TAG(TMaxSpdDmgTag, Real, m_maxSpeedDamaged)
DECL_TMPL_TAG(TMinSpdTag, Real, m_minSpeed)
DECL_TMPL_TAG(TTurnTag, Real, m_maxTurnRate)
DECL_TMPL_TAG(TTurnDmgTag, Real, m_maxTurnRateDamaged)
DECL_TMPL_TAG(TAccTag, Real, m_acceleration)
DECL_TMPL_TAG(TAccDmgTag, Real, m_accelerationDamaged)
DECL_TMPL_TAG(TLiftTag, Real, m_lift)
DECL_TMPL_TAG(TLiftDmgTag, Real, m_liftDamaged)
DECL_TMPL_TAG(TBrakeTag, Real, m_braking)
DECL_TMPL_TAG(TMinTurnSpdTag, Real, m_minTurnSpeed)
DECL_TMPL_TAG(TPrefHTag, Real, m_preferredHeight)
DECL_TMPL_TAG(TPrefHDmpTag, Real, m_preferredHeightDamping)
DECL_TMPL_TAG(TCircTag, Real, m_circlingRadius)
DECL_TMPL_TAG(TZLimTag, Real, m_speedLimitZ)
DECL_TMPL_TAG(TExtraFricTag, Real, m_extra2DFriction)
DECL_TMPL_TAG(TMaxThrustTag, Real, m_maxThrustAngle)
DECL_TMPL_TAG(TBehZTag, LocomotorBehaviorZ, m_behaviorZ)
DECL_TMPL_TAG(TAppTag, LocomotorAppearance, m_appearance)
DECL_TMPL_TAG(TPrioTag, LocomotorPriority, m_movePriority)
DECL_TMPL_TAG(TDownhillTag, Bool, m_downhillOnly)
DECL_TMPL_TAG(TWanderLenTag, Real, m_wanderLengthFactor)

static const char* kAppearances[LOCOMOTOR_APPEARANCE_COUNT] = {
    "TWO_LEGS", "FOUR_WHEELS", "TREADS", "HOVER", "THRUST",
    "WINGS", "CLIMBER", "OTHER", "MOTORCYCLE"
};
static const char* kZBehaviors[LOCOMOTOR_BEHAVIOR_Z_COUNT] = {
    "NO_Z_MOTIVE_FORCE", "SEA_LEVEL", "SURFACE_RELATIVE_HEIGHT", "ABSOLUTE_HEIGHT",
    "FIXED_SURFACE_RELATIVE_HEIGHT", "FIXED_ABSOLUTE_HEIGHT",
    "FIXED_RELATIVE_TO_GROUND_AND_BUILDINGS", "RELATIVE_TO_HIGHEST_LAYER"
};
static const char* kPriorities[LOCOMOTOR_PRIORITY_COUNT] = {
    "MOVES_BACK", "MOVES_MIDDLE", "MOVES_FRONT"
};
static const char* kSurfaces[5] = { "GROUND", "WATER", "CLIFF", "AIR", "RUBBLE" };

static std::string build_ini(
    const char* name, uint32_t& rng, uint32_t seed, int app_idx, int z_idx,
    int prio_idx, uint32_t surf_mask, bool omit_dmg) {
    auto rfloat = [&](int lo, int hi, Real scale) -> Real {
        int32_t span = hi - lo + 1;
        return static_cast<Real>(lo + static_cast<int32_t>(oracle::splitmix32(rng) % span)) * scale;
    };
    Real speed = (app_idx == LOCO_THRUST && (seed % 7 == 0)) ? 0.0f : rfloat(20, 220, 0.5f);
    Real speed_dmg = rfloat(10, 120, 0.5f);
    Real turn = rfloat(15, 360, 0.5f);
    Real turn_dmg = rfloat(10, 200, 0.5f);
    Real accel = rfloat(5, 160, 0.5f);
    Real accel_dmg = rfloat(5, 100, 0.5f);
    Real lift = (app_idx == LOCO_THRUST) ? 0.0f : rfloat(0, 90, 0.5f);
    Real lift_dmg = (app_idx == LOCO_THRUST) ? 0.0f : rfloat(0, 60, 0.5f);
    Real braking = rfloat(5, 140, 0.5f);
    Real min_spd = (app_idx == LOCO_WINGS && (seed % 9 == 0)) ? 0.0f : rfloat(0, 60, 0.25f);
    Real min_turn_spd = (app_idx == LOCO_WINGS && (seed % 9 == 0)) ? 0.0f : rfloat(0, 40, 0.25f);
    Real extra_fric = rfloat(0, 30, 0.05f);
    Real wander_len = rfloat(1, 30, 0.1f);
    Real close_enough = rfloat(1, 40, 0.25f);
    Real pref_h = (app_idx == LOCO_THRUST) ? 0.0f : rfloat(0, 200, 0.5f);
    Real pref_damp = rfloat(1, 10, 0.1f);
    Real circ_r = static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 240) - 120);
    Real slide_t = rfloat(100, 2000, 1.0f);

    std::ostringstream ss;
    ss << "Locomotor " << name << "\n";
    ss << "  Surfaces =";
    for (int i = 0; i < 5; ++i) {
        if ((surf_mask >> i) & 1u) ss << " " << kSurfaces[i];
    }
    if ((surf_mask & 31u) == 0) ss << " GROUND";
    ss << "\n";
    ss << "  Speed = " << speed << "\n";
    if (!omit_dmg) ss << "  SpeedDamaged = " << speed_dmg << "\n";
    ss << "  TurnRate = " << turn << "\n";
    if (!omit_dmg) ss << "  TurnRateDamaged = " << turn_dmg << "\n";
    ss << "  Acceleration = " << accel << "\n";
    if (!omit_dmg) ss << "  AccelerationDamaged = " << accel_dmg << "\n";
    if (app_idx != LOCO_THRUST) {
        ss << "  Lift = " << lift << "\n";
        if (!omit_dmg) ss << "  LiftDamaged = " << lift_dmg << "\n";
    }
    ss << "  Braking = " << braking << "\n";
    ss << "  MinSpeed = " << min_spd << "\n";
    ss << "  MinTurnSpeed = " << min_turn_spd << "\n";
    ss << "  Extra2DFriction = " << extra_fric << "\n";
    ss << "  PreferredHeight = " << pref_h << "\n";
    ss << "  PreferredHeightDamping = " << pref_damp << "\n";
    ss << "  CirclingRadius = " << circ_r << "\n";
    ss << "  SpeedLimitZ = " << rfloat(20, 300, 0.5f) << "\n";
    ss << "  MaxThrustAngle = " << rfloat(5, 85, 0.5f) << "\n";
    ss << "  ZAxisBehavior = " << kZBehaviors[z_idx] << "\n";
    ss << "  Appearance = " << kAppearances[app_idx] << "\n";
    ss << "  GroupMovementPriority = " << kPriorities[prio_idx] << "\n";
    ss << "  AccelerationPitchLimit = " << rfloat(1, 30, 0.5f) << "\n";
    ss << "  DecelerationPitchLimit = " << rfloat(1, 30, 0.5f) << "\n";
    ss << "  BounceAmount = " << rfloat(0, 60, 0.5f) << "\n";
    ss << "  CloseEnoughDist = " << close_enough << "\n";
    ss << "  CloseEnoughDist3D = " << ((seed & 1) ? "Yes" : "No") << "\n";
    ss << "  SlideIntoPlaceTime = " << slide_t << "\n";
    ss << "  WanderWidthFactor = " << rfloat(0, 20, 0.1f) << "\n";
    ss << "  WanderLengthFactor = " << wander_len << "\n";
    ss << "  WanderAboutPointRadius = " << rfloat(0, 80, 0.5f) << "\n";
    ss << "  DownhillOnly = " << ((seed % 6 == 0) ? "Yes" : "No") << "\n";
    ss << "  CanMoveBackwards = " << ((seed % 4 == 0) ? "Yes" : "No") << "\n";
    ss << "  StickToGround = " << ((seed % 3 == 0) ? "Yes" : "No") << "\n";
    ss << "  AllowAirborneMotiveForce = " << ((seed % 5 == 0) ? "Yes" : "No") << "\n";
    ss << "  Apply2DFrictionWhenAirborne = " << ((seed % 2 == 0) ? "Yes" : "No") << "\n";
    ss << "  LocomotorWorksWhenDead = " << ((seed % 8 == 0) ? "Yes" : "No") << "\n";
    ss << "  HasSuspension = " << ((app_idx == LOCO_WHEELS_FOUR) ? "Yes" : "No") << "\n";
    ss << "  RudderCorrectionDegree = " << rfloat(0, 25, 0.25f) << "\n";
    ss << "  RudderCorrectionRate = " << rfloat(0, 10, 0.1f) << "\n";
    ss << "  ElevatorCorrectionDegree = " << rfloat(0, 25, 0.25f) << "\n";
    ss << "  ElevatorCorrectionRate = " << rfloat(0, 10, 0.1f) << "\n";
    ss << "End\n";
    return ss.str();
}

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/PKT-143.json";
    oracle::init_memory();
    TheWritableGlobalData = NEW GlobalData;
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheGameLogic = NEW GameLogic;
    TheScriptEngine = NEW ScriptEngine;
    TheLocomotorStore = NEW LocomotorStore;

    const char* h_loco = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/Locomotor.h";
    const char* cpp_loco = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Locomotor.cpp";
    const char* h_set = "vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/GameLogic/LocomotorSet.h";

    std::ostringstream out;
    out << "{\n"
        << "  \"packet_id\": \"PKT-143\",\n"
        << "  \"packets\": [\"PKT-143\", \"PKT-144\"],\n"
        << "  \"modules\": [\"Locomotor\", \"LocomotorSet\"],\n"
        << "  \"vector_count\": 1000,\n"
        << "  \"use_retail_physics_forward_speed_average\": " << (USE_RETAIL_PHYSICS_FORWARD_SPEED_AVERAGE() ? 1 : 0) << ",\n"
        << "  \"source_sha256\": {\n"
        << "    \"" << h_loco << "\": \"" << oracle::sha256_file(h_loco) << "\",\n"
        << "    \"" << cpp_loco << "\": \"" << oracle::sha256_file(cpp_loco) << "\",\n"
        << "    \"" << h_set << "\": \"" << oracle::sha256_file(h_set) << "\"\n"
        << "  },\n  \"vectors\": [\n";

    oracle::MemINI mem_ini;
    std::vector<std::string> known_names;

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x143a77b1u;
        if (rng == 0) rng = 0x55551111u;

        int app_idx = (seed == 0) ? LOCO_WHEELS_FOUR : static_cast<int>(seed % LOCOMOTOR_APPEARANCE_COUNT);
        int z_idx = (app_idx == LOCO_THRUST) ? Z_NO_Z_MOTIVE_FORCE : static_cast<int>(oracle::splitmix32(rng) % LOCOMOTOR_BEHAVIOR_Z_COUNT);
        int prio_idx = static_cast<int>(oracle::splitmix32(rng) % LOCOMOTOR_PRIORITY_COUNT);
        uint32_t surf_mask = (seed == 0) ? (LOCOMOTORSURFACE_GROUND | LOCOMOTORSURFACE_RUBBLE)
                                         : (1u + (oracle::splitmix32(rng) % 31u));
        bool omit_dmg = (seed % 3 == 1);
        std::string tname = "Loco_" + std::to_string(seed);
        std::string ini_text = build_ini(tname.c_str(), rng, seed, app_idx, z_idx, prio_idx, surf_mask, omit_dmg);

        mem_ini.parse_blocks(ini_text, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
            if (std::strcmp(tok, "Locomotor") == 0) LocomotorStore::parseLocomotorTemplateDefinition(ini);
        });

        bool do_override = (seed % 5 == 0);
        std::string ovr_ini;
        if (do_override) {
            ovr_ini = "Locomotor " + tname + "\n  Speed = 135\n  TurnRate = 210\nEnd\n";
            mem_ini.parse_blocks(ovr_ini, INI_LOAD_CREATE_OVERRIDES, [](INI* ini, const char* tok) {
                if (std::strcmp(tok, "Locomotor") == 0) LocomotorStore::parseLocomotorTemplateDefinition(ini);
            });
        }

        bool bad_threw = false;
        std::string bad_ini;
        if (seed == 0 || (seed % 47 == 0)) {
            bad_ini = "Locomotor BadThrust_" + std::to_string(seed) + "\n  Appearance = THRUST\n  Lift = 10\n  ZAxisBehavior = SEA_LEVEL\nEnd\n";
            try {
                mem_ini.parse_blocks(bad_ini, INI_LOAD_OVERWRITE, [](INI* ini, const char* tok) {
                    if (std::strcmp(tok, "Locomotor") == 0) LocomotorStore::parseLocomotorTemplateDefinition(ini);
                });
            } catch (...) {
                bad_threw = true;
            }
        }

        const LocomotorTemplate* base_tmpl = TheLocomotorStore->findLocomotorTemplate(NAMEKEY(tname.c_str()));
        const LocomotorTemplate* tmpl = do_override ? reinterpret_cast<const LocomotorTemplate*>(base_tmpl->friend_getFinalOverride()) : base_tmpl;
        known_names.push_back(tname);

        uint32_t rand_seed = 0x1000u + seed * 7919u;
        UnsignedInt frame0 = (seed == 0) ? 100u : (oracle::splitmix32(rng) % 5000u);
        UnsignedInt start_frame = frame0 + 1u + (oracle::splitmix32(rng) % 60u);
        Int move_pen_state = static_cast<Int>(oracle::splitmix32(rng) % BODYDAMAGETYPE_COUNT);
        Bool legacy_fwd = (seed % 4 == 2);

        TheWritableGlobalData->m_movementPenaltyDamageState = static_cast<BodyDamageType>(move_pen_state);
        TheScriptEngine->*access_member(SELetterBoxTag()) = legacy_fwd;
        TheGameLogic->*access_member(GLFrameTag()) = frame0;
        InitRandom(rand_seed);

        Locomotor* loco = TheLocomotorStore->newLocomotor(tmpl);
        UnsignedInt rand_crc_after = GetGameLogicRandomSeedCRC();

        Coord3D mpos{
            static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 2000) - 500) * 0.25f,
            static_cast<Real>(static_cast<int32_t>(oracle::splitmix32(rng) % 2000) - 500) * 0.25f,
            static_cast<Real>(oracle::splitmix32(rng) % 400) * 0.25f
        };
        loco->*access_member(LocoMaintainTag()) = mpos;

        Real ov_spd = (seed % 2 == 0) ? static_cast<Real>(10 + (oracle::splitmix32(rng) % 300)) * 0.05f : 99999.0f;
        Real ov_turn = (seed % 3 == 0) ? static_cast<Real>(1 + (oracle::splitmix32(rng) % 100)) * 0.005f : 99999.0f;
        Real ov_acc = (seed % 3 == 1) ? static_cast<Real>(1 + (oracle::splitmix32(rng) % 100)) * 0.01f : 99999.0f;
        Real ov_lift = (seed % 4 == 0) ? static_cast<Real>(1 + (oracle::splitmix32(rng) % 80)) * 0.01f : 99999.0f;
        Real ov_brk = (seed % 5 == 1) ? static_cast<Real>(1 + (oracle::splitmix32(rng) % 90)) * 0.01f : 99999.0f;
        Real new_ce = static_cast<Real>(1 + (oracle::splitmix32(rng) % 40)) * 0.5f;
        Real new_ph = static_cast<Real>(oracle::splitmix32(rng) % 120) * 0.5f;
        Bool b_inv = (oracle::splitmix32(rng) & 1u) != 0;
        Bool b_ce3 = (oracle::splitmix32(rng) & 1u) != 0;
        Bool b_pz = (oracle::splitmix32(rng) & 1u) != 0;
        Bool b_ns = (oracle::splitmix32(rng) & 1u) != 0;
        Bool b_ua = (oracle::splitmix32(rng) & 1u) != 0;

        if (ov_spd < 99999.0f) loco->setMaxSpeed(ov_spd);
        if (ov_turn < 99999.0f) loco->setMaxTurnRate(ov_turn);
        if (ov_acc < 99999.0f) loco->setMaxAcceleration(ov_acc);
        if (ov_lift < 99999.0f) loco->setMaxLift(ov_lift);
        if (ov_brk < 99999.0f) loco->setMaxBraking(ov_brk);
        loco->setCloseEnoughDist(new_ce);
        loco->setCloseEnoughDist3D(b_ce3);
        loco->setAllowInvalidPosition(b_inv);
        loco->setPreferredHeight(new_ph);
        loco->setUsePreciseZPos(b_pz);
        loco->setNoSlowDownAsApproachingDest(b_ns);
        loco->setUltraAccurate(b_ua);
        TheGameLogic->*access_member(GLFrameTag()) = start_frame;
        loco->startMove();

        uint32_t spd_bits[4], trn_bits[4], acc_bits[4], lft_bits[4], rad_bits[4], tim_bits[4];
        for (int c = 0; c < BODYDAMAGETYPE_COUNT; ++c) {
            BodyDamageType bdt = static_cast<BodyDamageType>(c);
            spd_bits[c] = oracle::fbits(loco->getMaxSpeedForCondition(bdt));
            trn_bits[c] = oracle::fbits(loco->getMaxTurnRate(bdt));
            acc_bits[c] = oracle::fbits(loco->getMaxAcceleration(bdt));
            lft_bits[c] = oracle::fbits(loco->getMaxLift(bdt));
            Real tt = -1.0f;
            rad_bits[c] = oracle::fbits(loco->calcMinTurnRadius(bdt, &tt));
            tim_bits[c] = oracle::fbits(tt);
        }
        uint32_t brk_bits = oracle::fbits(loco->getBraking());

        oracle::MemoryXfer l_save(XFER_SAVE); l_save.open("ls");
        oracle::call_snapshot_xfer(loco, &l_save); l_save.close();
        Locomotor* l_loaded = TheLocomotorStore->newLocomotor(tmpl);
        oracle::MemoryXfer l_load(XFER_LOAD); l_load.bytes = l_save.bytes; l_load.open("ll");
        oracle::call_snapshot_xfer(l_loaded, &l_load); l_load.close();
        oracle::MemoryXfer l_resave(XFER_SAVE); l_resave.open("lr");
        oracle::call_snapshot_xfer(l_loaded, &l_resave); l_resave.close();
        if (l_save.bytes != l_resave.bytes) {
            std::fprintf(stderr, "Locomotor round-trip mismatch seed %u\n", seed);
            return 3;
        }
        deleteInstance(l_loaded);

        // PKT-144 LocomotorSet test
        uint32_t set_rand_seed = rand_seed ^ 0x55aa55aau;
        InitRandom(set_rand_seed);
        TheGameLogic->*access_member(GLFrameTag()) = frame0;
        size_t set_cnt = 1u + (seed % 3u);
        if (set_cnt > known_names.size()) set_cnt = known_names.size();
        LocomotorSet lset;
        std::vector<std::string> set_names;
        for (size_t k = 0; k < set_cnt; ++k) {
            size_t idx = (seed + known_names.size() - 1u - k) % known_names.size();
            const LocomotorTemplate* base_k = TheLocomotorStore->findLocomotorTemplate(NAMEKEY(known_names[idx].c_str()));
            const LocomotorTemplate* kt = reinterpret_cast<const LocomotorTemplate*>(base_k->friend_getFinalOverride());
            lset.addLocomotor(kt);
            set_names.push_back(known_names[idx]);
        }
        for (Locomotor* lk : lset.*access_member(SetLocosTag())) {
            if (lk) lk->*access_member(LocoMaintainTag()) = Coord3D{0.0f, 0.0f, 0.0f};
        }
        uint32_t qmasks[4] = {
            LOCOMOTORSURFACE_GROUND, LOCOMOTORSURFACE_WATER,
            LOCOMOTORSURFACE_AIR, LOCOMOTORSURFACE_CLIFF | LOCOMOTORSURFACE_RUBBLE
        };
        std::string qhits[4];
        for (int q = 0; q < 4; ++q) {
            Locomotor* found = lset.findLocomotor(static_cast<LocomotorSurfaceTypeMask>(qmasks[q]));
            qhits[q] = found ? found->getTemplateName().str() : "";
        }
        int cur_idx = (seed % 4 == 3) ? -1 : static_cast<int>(seed % set_cnt);
        Locomotor* cur_ptr = (cur_idx >= 0) ? lset.findLocomotor(ALL_SURFACES) : nullptr;
        if (cur_idx > 0 && cur_ptr) {
            Locomotor* alt = lset.findLocomotor(static_cast<LocomotorSurfaceTypeMask>(qmasks[cur_idx % 4]));
            if (alt) cur_ptr = alt;
        }

        oracle::MemoryXfer s_save(XFER_SAVE); s_save.open("ss");
        lset.xferSelfAndCurLocoPtr(&s_save, &cur_ptr); s_save.close();
        LocomotorSet s_loaded_set;
        Locomotor* s_loaded_cur = nullptr;
        oracle::MemoryXfer s_load(XFER_LOAD); s_load.bytes = s_save.bytes; s_load.open("sl");
        s_loaded_set.xferSelfAndCurLocoPtr(&s_load, &s_loaded_cur); s_load.close();
        if ((cur_ptr == nullptr) != (s_loaded_cur == nullptr) ||
            (cur_ptr && s_loaded_cur && cur_ptr->getTemplateName() != s_loaded_cur->getTemplateName())) {
            std::fprintf(stderr, "LocomotorSet round-trip cur pointer mismatch seed %u\n", seed);
            return 4;
        }

        out << "    {\"seed\":" << seed
            << ",\"ini\":\"" << oracle::json_escape(ini_text) << "\""
            << ",\"ovr\":\"" << oracle::json_escape(ovr_ini) << "\""
            << ",\"bad\":\"" << oracle::json_escape(bad_ini) << "\""
            << ",\"bad_threw\":" << (bad_threw ? "true" : "false")
            << ",\"rand_seed\":" << rand_seed
            << ",\"set_rand_seed\":" << set_rand_seed
            << ",\"frame0\":" << frame0
            << ",\"start_frame\":" << start_frame
            << ",\"move_pen\":" << move_pen_state
            << ",\"legacy_fwd\":" << (legacy_fwd ? "true" : "false")
            << ",\"mpos\":[" << oracle::fbits(mpos.x) << "," << oracle::fbits(mpos.y) << "," << oracle::fbits(mpos.z) << "]"
            << ",\"ov\":[" << oracle::fbits(ov_spd) << "," << oracle::fbits(ov_turn) << "," << oracle::fbits(ov_acc)
            << "," << oracle::fbits(ov_lift) << "," << oracle::fbits(ov_brk) << "," << oracle::fbits(new_ce)
            << "," << oracle::fbits(new_ph) << "," << (b_inv?1:0) << "," << (b_ce3?1:0) << "," << (b_pz?1:0)
            << "," << (b_ns?1:0) << "," << (b_ua?1:0) << "]"
            << ",\"set_names\":[";
        for (size_t k = 0; k < set_names.size(); ++k) out << (k ? "," : "") << "\"" << set_names[k] << "\"";
        out << "],\"cur_name\":\"" << (cur_ptr ? cur_ptr->getTemplateName().str() : "") << "\""
            << ",\"rand_crc\":" << rand_crc_after
            << ",\"spd\":[" << spd_bits[0] << "," << spd_bits[1] << "," << spd_bits[2] << "," << spd_bits[3] << "]"
            << ",\"trn\":[" << trn_bits[0] << "," << trn_bits[1] << "," << trn_bits[2] << "," << trn_bits[3] << "]"
            << ",\"acc\":[" << acc_bits[0] << "," << acc_bits[1] << "," << acc_bits[2] << "," << acc_bits[3] << "]"
            << ",\"lft\":[" << lft_bits[0] << "," << lft_bits[1] << "," << lft_bits[2] << "," << lft_bits[3] << "]"
            << ",\"rad\":[" << rad_bits[0] << "," << rad_bits[1] << "," << rad_bits[2] << "," << rad_bits[3] << "]"
            << ",\"tim\":[" << tim_bits[0] << "," << tim_bits[1] << "," << tim_bits[2] << "," << tim_bits[3] << "]"
            << ",\"brk\":" << brk_bits
            << ",\"lflags\":" << loco->*access_member(LocoFlagsTag())
            << ",\"lxlen\":" << l_save.bytes.size()
            << ",\"lxcrc\":" << l_save.bytes_crc()
            << ",\"set_surf\":" << lset.getValidSurfaces()
            << ",\"set_dh\":" << (lset.isDownhillOnly() ? 1 : 0)
            << ",\"qhits\":[\"" << qhits[0] << "\",\"" << qhits[1] << "\",\"" << qhits[2] << "\",\"" << qhits[3] << "\"]"
            << ",\"sxlen\":" << s_save.bytes.size()
            << ",\"sxcrc\":" << s_save.bytes_crc();

        if (seed == 0) {
            out << ",\"s0_tmpl\":{"
                << "\"surf\":" << tmpl->*access_member(TSurfTag())
                << ",\"app\":" << static_cast<int>(tmpl->*access_member(TAppTag()))
                << ",\"bz\":" << static_cast<int>(tmpl->*access_member(TBehZTag()))
                << ",\"prio\":" << static_cast<int>(tmpl->*access_member(TPrioTag()))
                << ",\"dh\":" << (tmpl->*access_member(TDownhillTag()) ? 1 : 0)
                << ",\"max_spd\":" << oracle::fbits(tmpl->*access_member(TMaxSpdTag()))
                << ",\"max_spd_dmg\":" << oracle::fbits(tmpl->*access_member(TMaxSpdDmgTag()))
                << ",\"min_spd\":" << oracle::fbits(tmpl->*access_member(TMinSpdTag()))
                << ",\"turn\":" << oracle::fbits(tmpl->*access_member(TTurnTag()))
                << ",\"turn_dmg\":" << oracle::fbits(tmpl->*access_member(TTurnDmgTag()))
                << ",\"acc\":" << oracle::fbits(tmpl->*access_member(TAccTag()))
                << ",\"acc_dmg\":" << oracle::fbits(tmpl->*access_member(TAccDmgTag()))
                << ",\"lift\":" << oracle::fbits(tmpl->*access_member(TLiftTag()))
                << ",\"lift_dmg\":" << oracle::fbits(tmpl->*access_member(TLiftDmgTag()))
                << ",\"brk\":" << oracle::fbits(tmpl->*access_member(TBrakeTag()))
                << ",\"min_turn_spd\":" << oracle::fbits(tmpl->*access_member(TMinTurnSpdTag()))
                << ",\"extra_fric\":" << oracle::fbits(tmpl->*access_member(TExtraFricTag()))
                << ",\"wander_len\":" << oracle::fbits(tmpl->*access_member(TWanderLenTag()))
                << "}"
                << ",\"s0_loco\":{"
                << "\"ang_off\":" << oracle::fbits(loco->*access_member(LocoAngleOffTag()))
                << ",\"off_inc\":" << oracle::fbits(loco->*access_member(LocoOffIncTag()))
                << ",\"donut\":" << loco->*access_member(LocoDonutTag())
                << ",\"l_xhex\":\"" << l_save.hex() << "\""
                << "}"
                << ",\"s0_set\":{"
                << "\"valid_surfaces\":" << lset.getValidSurfaces()
                << ",\"downhill_only\":" << (lset.isDownhillOnly() ? 1 : 0)
                << ",\"s_xhex\":\"" << s_save.hex() << "\""
                << "}";
        }
        out << "}" << ((seed + 1u < 1000u) ? "," : "") << "\n";

        deleteInstance(loco);
        TheLocomotorStore->reset();
    }

    out << "  ]\n}\n";
    oracle::write_file_atomic(out_path, out.str());
    shutdownMemoryManager();
    _exit(0);
}
