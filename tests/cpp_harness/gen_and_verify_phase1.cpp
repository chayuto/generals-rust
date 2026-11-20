// tests/cpp_harness/gen_and_verify_phase1.cpp
// Gate V2 C++ Golden Vector Generator & In-Process Differential Verifier (PKT-001, PKT-002, PKT-003)
// Compiles directly against unmodified vendor/GeneralsX sources at pinned commit 84a7a20.

#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/RandomValue.h"
#include "Common/Xfer.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"

#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <cmath>
#include <fstream>
#include <iomanip>
#include <iostream>
#include <sstream>
#include <string>
#include <vector>

// Include unmodified reference C++ implementations from vendor/GeneralsX
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ExperienceTracker.cpp"

#ifdef VERIFY_FFI
#include "../../crates/zh_ffi/include/zh_ffi.h"
#endif

GameLogic g_harnessGameLogic;
GameLogic* TheGameLogic = &g_harnessGameLogic;

static std::string to_hex_u32(uint32_t v) {
    char buf[16];
    std::snprintf(buf, sizeof(buf), "0x%08X", v);
    return std::string(buf);
}

static std::string bytes_to_hex(const std::vector<uint8_t>& bytes) {
    static const char* lut = "0123456789abcdef";
    std::string out;
    out.reserve(bytes.size() * 2);
    for (uint8_t b : bytes) {
        out.push_back(lut[(b >> 4) & 0xF]);
        out.push_back(lut[b & 0xF]);
    }
    return out;
}

static uint32_t float_to_bits(float f) {
    uint32_t u = 0;
    std::memcpy(&u, &f, sizeof(uint32_t));
    return u;
}

static float bits_to_float(uint32_t u) {
    float f = 0.0f;
    std::memcpy(&f, &u, sizeof(float));
    return f;
}

// SplitMix32 deterministic harness driver PRNG (independent from engine PRNG)
struct HarnessRng {
    uint32_t state;
    explicit HarnessRng(uint32_t s) : state(s) {}
    uint32_t next_u32() {
        uint32_t z = (state += 0x9e3779b9u);
        z = (z ^ (z >> 16)) * 0x85ebca6bu;
        z = (z ^ (z >> 13)) * 0xc2b2ae35u;
        return z ^ (z >> 16);
    }
    int32_t next_i32(int32_t lo, int32_t hi) {
        uint32_t span = static_cast<uint32_t>(hi - lo + 1);
        return lo + static_cast<int32_t>(next_u32() % span);
    }
};

// ============================================================================
// 1. PKT-001 (crc): 1,000 Golden Vectors
// ============================================================================
static void emit_pkt001_golden(const std::string& out_dir) {
    std::ofstream f(out_dir + "/PKT-001.json");
    f << "{\n"
      << "  \"packet_id\": \"PKT-001\",\n"
      << "  \"module_name\": \"crc\",\n"
      << "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n"
      << "  \"regime\": \"USE_DETERMINISTIC_MATH=1,RETAIL_COMPATIBLE_CRC=0\",\n"
      << "  \"vector_count\": 1000,\n"
      << "  \"vectors\": [\n";

    for (int i = 0; i < 1000; ++i) {
        HarnessRng rng(0x10010000u + static_cast<uint32_t>(i));
        std::vector<uint8_t> chunk1;
        std::vector<uint8_t> chunk2;
        int32_t passed_len1 = 0;

        if (i == 0) {
            // Empty buffer len=0
            passed_len1 = 0;
        } else if (i == 1) {
            // Negative len=-5 with non-empty buffer
            chunk1 = {0xAA, 0xBB, 0xCC, 0xDD};
            passed_len1 = -5;
        } else if (i == 2) {
            // High-bit carry cascade: 32 bytes of 0xFF
            chunk1.assign(32, 0xFF);
            passed_len1 = 32;
        } else if (i == 3) {
            // Single 0x80 byte repeated
            chunk1.assign(64, 0x80);
            passed_len1 = 64;
        } else if (i == 4) {
            // All zeros
            chunk1.assign(37, 0x00);
            passed_len1 = 37;
        } else {
            uint32_t n1 = rng.next_u32() % 96;
            uint32_t n2 = rng.next_u32() % 64;
            chunk1.resize(n1);
            chunk2.resize(n2);
            for (uint32_t k = 0; k < n1; ++k) chunk1[k] = static_cast<uint8_t>(rng.next_u32() & 0xFF);
            for (uint32_t k = 0; k < n2; ++k) chunk2[k] = static_cast<uint8_t>(rng.next_u32() & 0xFF);
            passed_len1 = static_cast<int32_t>(n1);
        }

        CRC c;
        c.computeCRC(chunk1.empty() ? nullptr : chunk1.data(), passed_len1);
        uint32_t after_chunk1 = c.get();
        c.computeCRC(chunk2.empty() ? nullptr : chunk2.data(), static_cast<int32_t>(chunk2.size()));
        uint32_t after_chunk2 = c.get();

        f << "    {\"seed\":" << i
          << ",\"chunk1_hex\":\"" << bytes_to_hex(chunk1) << "\""
          << ",\"passed_len1\":" << passed_len1
          << ",\"chunk2_hex\":\"" << bytes_to_hex(chunk2) << "\""
          << ",\"expected_crc1\":\"" << to_hex_u32(after_chunk1) << "\""
          << ",\"expected_crc2\":\"" << to_hex_u32(after_chunk2) << "\"}";
        if (i + 1 < 1000) f << ",";
        f << "\n";
    }
    f << "  ]\n}\n";
}

// ============================================================================
// 2. PKT-002 (RandomValue): 1,000 Golden Vectors
// ============================================================================
static void emit_pkt002_golden(const std::string& out_dir) {
    std::ofstream f(out_dir + "/PKT-002.json");
    f << "{\n"
      << "  \"packet_id\": \"PKT-002\",\n"
      << "  \"module_name\": \"RandomValue\",\n"
      << "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n"
      << "  \"regime\": \"USE_DETERMINISTIC_MATH=1,RETAIL_COMPATIBLE_CRC=0\",\n"
      << "  \"vector_count\": 1000,\n"
      << "  \"vectors\": [\n";

    for (int i = 0; i < 1000; ++i) {
        HarnessRng rng(0x20020000u + static_cast<uint32_t>(i));
        uint32_t init_seed = (i < 16) ? static_cast<uint32_t>(i) : rng.next_u32();
        if (i == 16) init_seed = 0xFFFFFFFFu;
        if (i == 17) init_seed = 0x80000000u;
        if (i == 18) init_seed = 0x0d22f1aau; // triggers seed[0] == 0

        InitRandom(init_seed);
        uint32_t init_crc = GetGameLogicRandomSeedCRC();

        int32_t lo_i = rng.next_i32(-50000, 50000);
        int32_t hi_i = rng.next_i32(-50000, 50000);
        // Ensure explicit coverage of lo == hi and lo > hi and normal lo < hi
        if (i % 5 == 0) {
            hi_i = lo_i; // lo == hi (no advance!)
        } else if (i % 5 == 1) {
            if (lo_i <= hi_i) std::swap(lo_i, hi_i);
            if (lo_i == hi_i) lo_i += 7; // lo > hi (no advance!)
        } else {
            if (lo_i > hi_i) std::swap(lo_i, hi_i);
            if (lo_i == hi_i) hi_i += 1;
        }

        float lo_f = static_cast<float>(rng.next_i32(-10000, 10000)) * 0.125f;
        float hi_f = static_cast<float>(rng.next_i32(-10000, 10000)) * 0.125f;
        if (i % 7 == 0) {
            hi_f = lo_f;
        } else if (i % 7 == 1) {
            if (lo_f <= hi_f) std::swap(lo_f, hi_f);
            if (lo_f == hi_f) lo_f += 1.5f;
        } else {
            if (lo_f > hi_f) std::swap(lo_f, hi_f);
            if (lo_f == hi_f) hi_f += 2.25f;
        }

        // 1. Unchanged calls must NOT mutate seed CRC
        int32_t unch_i = GetGameLogicRandomValueUnchanged(lo_i, hi_i, "golden", i);
        float   unch_f = GetGameLogicRandomValueRealUnchanged(lo_f, hi_f, "golden", i);
        uint32_t crc_after_unch = GetGameLogicRandomSeedCRC();

        // 2. Logic int + logic real calls
        int32_t logic_i1 = GetGameLogicRandomValue(lo_i, hi_i, "golden", i);
        uint32_t crc_after_i1 = GetGameLogicRandomSeedCRC();
        float   logic_f1 = GetGameLogicRandomValueReal(lo_f, hi_f, "golden", i);
        int32_t logic_i2 = GetGameLogicRandomValue(-100, 100, "golden", i);
        float   logic_f2 = GetGameLogicRandomValueReal(0.0f, 1.0f, "golden", i);
        uint32_t final_crc = GetGameLogicRandomSeedCRC();

        // 3. Client & Audio PRNG streams
        int32_t client_i = GetGameClientRandomValue(lo_i, hi_i, "golden", i);
        float   client_f = GetGameClientRandomValueReal(lo_f, hi_f, "golden", i);
        int32_t audio_i  = GetGameAudioRandomValue(lo_i, hi_i, "golden", i);
        float   audio_f  = GetGameAudioRandomValueReal(lo_f, hi_f, "golden", i);

        f << "    {\"seed_index\":" << i
          << ",\"init_seed\":" << init_seed
          << ",\"lo_i\":" << lo_i
          << ",\"hi_i\":" << hi_i
          << ",\"lo_f_bits\":\"" << to_hex_u32(float_to_bits(lo_f)) << "\""
          << ",\"hi_f_bits\":\"" << to_hex_u32(float_to_bits(hi_f)) << "\""
          << ",\"init_crc\":\"" << to_hex_u32(init_crc) << "\""
          << ",\"unch_i\":" << unch_i
          << ",\"unch_f_bits\":\"" << to_hex_u32(float_to_bits(unch_f)) << "\""
          << ",\"crc_after_unch\":\"" << to_hex_u32(crc_after_unch) << "\""
          << ",\"logic_i1\":" << logic_i1
          << ",\"crc_after_i1\":\"" << to_hex_u32(crc_after_i1) << "\""
          << ",\"logic_f1_bits\":\"" << to_hex_u32(float_to_bits(logic_f1)) << "\""
          << ",\"logic_i2\":" << logic_i2
          << ",\"logic_f2_bits\":\"" << to_hex_u32(float_to_bits(logic_f2)) << "\""
          << ",\"final_crc\":\"" << to_hex_u32(final_crc) << "\""
          << ",\"client_i\":" << client_i
          << ",\"client_f_bits\":\"" << to_hex_u32(float_to_bits(client_f)) << "\""
          << ",\"audio_i\":" << audio_i
          << ",\"audio_f_bits\":\"" << to_hex_u32(float_to_bits(audio_f)) << "\""
          << ",\"state_words\":["
          << theGameLogicSeed[0] << "," << theGameLogicSeed[1] << "," << theGameLogicSeed[2] << ","
          << theGameLogicSeed[3] << "," << theGameLogicSeed[4] << "," << theGameLogicSeed[5] << "]}";
        if (i + 1 < 1000) f << ",";
        f << "\n";
    }
    f << "  ]\n}\n";
}

// ============================================================================
// 3. PKT-003 (ExperienceTracker): 1,000 Golden Vectors
// ============================================================================
static void emit_pkt003_golden(const std::string& out_dir) {
    std::ofstream f(out_dir + "/PKT-003.json");
    f << "{\n"
      << "  \"packet_id\": \"PKT-003\",\n"
      << "  \"module_name\": \"ExperienceTracker\",\n"
      << "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n"
      << "  \"regime\": \"USE_DETERMINISTIC_MATH=1,RETAIL_COMPATIBLE_CRC=0\",\n"
      << "  \"vector_count\": 1000,\n"
      << "  \"vectors\": [\n";

    for (int i = 0; i < 1000; ++i) {
        HarnessRng rng(0x30030000u + static_cast<uint32_t>(i));

        g_harnessGameLogic.m_objects.clear();
        Object parentObj;
        Object sinkObj;
        Object killerObj;

        parentObj.m_id = static_cast<ObjectID>(1);
        parentObj.m_template.m_isTrainable = (rng.next_u32() % 10 != 0) ? TRUE : FALSE;
        int32_t t1 = rng.next_i32(20, 80);
        int32_t t2 = t1 + rng.next_i32(30, 120);
        int32_t t3 = t2 + rng.next_i32(50, 200);
        parentObj.m_template.m_expRequired[0] = 0;
        parentObj.m_template.m_expRequired[1] = t1;
        parentObj.m_template.m_expRequired[2] = t2;
        parentObj.m_template.m_expRequired[3] = t3;

        parentObj.m_template.m_expValue[0] = rng.next_i32(5, 25);
        parentObj.m_template.m_expValue[1] = rng.next_i32(26, 50);
        parentObj.m_template.m_expValue[2] = rng.next_i32(51, 80);
        parentObj.m_template.m_expValue[3] = rng.next_i32(81, 150);

        sinkObj.m_id = static_cast<ObjectID>(2);
        sinkObj.m_template = parentObj.m_template;
        sinkObj.m_template.m_isTrainable = TRUE;

        killerObj.m_relWithTarget = static_cast<Relationship>(rng.next_u32() % 3);

        ExperienceTracker tracker(&parentObj);
        ExperienceTracker sinkTracker(&sinkObj);
        parentObj.m_tracker = &tracker;
        sinkObj.m_tracker = &sinkTracker;

        g_harnessGameLogic.m_objects[parentObj.m_id] = &parentObj;
        int32_t sink_id_cfg = 0;
        bool sink_alive = false;
        if (i % 8 == 1) {
            // Active living sink
            sink_id_cfg = 2;
            sink_alive = true;
            g_harnessGameLogic.m_objects[sinkObj.m_id] = &sinkObj;
            tracker.setExperienceSink(sinkObj.m_id);
        } else if (i % 8 == 2) {
            // Dead/missing sink (not in TheGameLogic) -> falls back to self!
            sink_id_cfg = 99;
            sink_alive = false;
            tracker.setExperienceSink(static_cast<ObjectID>(99));
        }

        // Scalar choices including fractional floats like 1.25f, 1.3333334f, 0.75f, 2.5f
        static const float kScalars[8] = {1.0f, 1.25f, 0.75f, 1.5f, 2.0f, 0.5f, 1.3333334f, 1.1f};
        float scalar = kScalars[rng.next_u32() % 8];
        tracker.setExperienceScalar(scalar);

        // Initial level & experience setup via XferLoad to test arbitrary valid starting states
        int32_t init_level = rng.next_i32(0, 3);
        int32_t init_exp = rng.next_i32(0, t3 + 20);
        uint8_t init_trainable = parentObj.m_template.m_isTrainable;
        if (i == 0) {
            // Exact case from docs/ARCHITECTURE_SPEC.md
            parentObj.m_template.m_expRequired[0] = 0;
            parentObj.m_template.m_expRequired[1] = 50;
            parentObj.m_template.m_expRequired[2] = 100;
            parentObj.m_template.m_expRequired[3] = 200;
            t1 = 50; t2 = 100; t3 = 200;
            init_level = 0;
            init_exp = 40;
            scalar = 1.25f;
            init_trainable = 1;
            sink_id_cfg = 0;
            sink_alive = false;
        }
        {
            HarnessXferSave initSave;
            XferVersion ver = 2;
            initSave.xferVersion(&ver, 2);
            VeterancyLevel lvl = static_cast<VeterancyLevel>(init_level);
            initSave.xferUser(&lvl, sizeof(VeterancyLevel));
            initSave.xferInt(&init_exp);
            ObjectID sid = static_cast<ObjectID>(sink_id_cfg);
            initSave.xferObjectID(&sid);
            initSave.xferReal(&scalar);
            initSave.xferBool(&init_trainable);

            HarnessXferLoad initLoad(initSave.bytes.data(), static_cast<Int>(initSave.bytes.size()));
            tracker.xfer(&initLoad);
        }

        parentObj.m_levelChangedCalled = FALSE;
        parentObj.m_levelChangedCount = 0;
        parentObj.m_lastOldLevel = LEVEL_REGULAR;
        parentObj.m_lastNewLevel = LEVEL_REGULAR;
        parentObj.m_lastProvideFeedback = FALSE;

        int op_kind = (i == 0) ? 0 : (i % 6);
        int32_t arg1 = 0;
        uint8_t arg_bool = 1;
        int32_t ret_i32 = 0;

        switch (op_kind) {
        case 0: { // addExperiencePoints
            arg1 = (i == 0) ? 9 : rng.next_i32(-10, 180);
            arg_bool = (i == 0) ? 1 : static_cast<uint8_t>(rng.next_u32() & 1);
            tracker.addExperiencePoints(arg1, arg_bool);
            break;
        }
        case 1: { // setMinVeterancyLevel
            arg1 = rng.next_i32(0, 3);
            arg_bool = static_cast<uint8_t>(rng.next_u32() & 1);
            tracker.setMinVeterancyLevel(static_cast<VeterancyLevel>(arg1), arg_bool);
            break;
        }
        case 2: { // setVeterancyLevel
            arg1 = rng.next_i32(0, 3);
            arg_bool = static_cast<uint8_t>(rng.next_u32() & 1);
            tracker.setVeterancyLevel(static_cast<VeterancyLevel>(arg1), arg_bool);
            break;
        }
        case 3: { // gainExpForLevel + canGainExpForLevel
            arg1 = rng.next_i32(-1, 4);
            arg_bool = static_cast<uint8_t>(rng.next_u32() & 1);
            uint8_t can_gain = tracker.canGainExpForLevel(arg1);
            uint8_t did_gain = tracker.gainExpForLevel(arg1, arg_bool);
            ret_i32 = (static_cast<int32_t>(can_gain) << 8) | static_cast<int32_t>(did_gain);
            break;
        }
        case 4: { // setExperienceAndLevel
            arg1 = rng.next_i32(-20, t3 + 50);
            arg_bool = static_cast<uint8_t>(rng.next_u32() & 1);
            tracker.setExperienceAndLevel(arg1, arg_bool);
            break;
        }
        case 5: { // getExperienceValue + isAcceptingExperiencePoints
            int32_t val = tracker.getExperienceValue(&killerObj);
            uint8_t acc = tracker.isAcceptingExperiencePoints();
            ret_i32 = (val << 8) | static_cast<int32_t>(acc);
            break;
        }
        }

        HarnessXferCRC xferCrc;
        tracker.crc(&xferCrc);
        HarnessXferSave xferSave;
        tracker.xfer(&xferSave);

        f << "    {\"seed\":" << i
          << ",\"thresholds\":[0," << t1 << "," << t2 << "," << t3 << "]"
          << ",\"exp_values\":[" << parentObj.m_template.m_expValue[0] << ","
          << parentObj.m_template.m_expValue[1] << ","
          << parentObj.m_template.m_expValue[2] << ","
          << parentObj.m_template.m_expValue[3] << "]"
          << ",\"killer_rel\":" << static_cast<int32_t>(killerObj.m_relWithTarget)
          << ",\"init_level\":" << init_level
          << ",\"init_exp\":" << init_exp
          << ",\"init_sink\":" << sink_id_cfg
          << ",\"sink_alive\":" << (sink_alive ? 1 : 0)
          << ",\"scalar_bits\":\"" << to_hex_u32(float_to_bits(scalar)) << "\""
          << ",\"init_trainable\":" << static_cast<int32_t>(init_trainable)
          << ",\"op_kind\":" << op_kind
          << ",\"arg1\":" << arg1
          << ",\"arg_bool\":" << static_cast<int32_t>(arg_bool)
          << ",\"ret_i32\":" << ret_i32
          << ",\"post_level\":" << static_cast<int32_t>(tracker.getVeterancyLevel())
          << ",\"post_exp\":" << tracker.getCurrentExperience()
          << ",\"post_trainable\":" << static_cast<int32_t>(tracker.isTrainable())
          << ",\"cb_fired\":" << static_cast<int32_t>(parentObj.m_levelChangedCalled)
          << ",\"cb_old_level\":" << static_cast<int32_t>(parentObj.m_lastOldLevel)
          << ",\"cb_new_level\":" << static_cast<int32_t>(parentObj.m_lastNewLevel)
          << ",\"cb_feedback\":" << static_cast<int32_t>(parentObj.m_lastProvideFeedback)
          << ",\"sink_post_exp\":" << sinkTracker.getCurrentExperience()
          << ",\"sink_post_level\":" << static_cast<int32_t>(sinkTracker.getVeterancyLevel())
          << ",\"expected_xfer_crc\":\"" << to_hex_u32(xferCrc.getCRC()) << "\""
          << ",\"expected_xfer_hex\":\"" << bytes_to_hex(xferSave.bytes) << "\"}";
        if (i + 1 < 1000) f << ",";
        f << "\n";
    }
    f << "  ]\n}\n";
}

#ifdef VERIFY_FFI
// ============================================================================
// Gate V2-FFI: In-Process C++ vs. Rust Differential Harness (1,000 seeds x 3 modules)
// ============================================================================
struct FfiCallbackBridgeCtx {
    Object* parent;
    bool sink_alive;
    ZhExpTrackerState* sink_rust_state;
    ZhExpTrackerParentCallbacks* sink_rust_cb;
    Bool cb_fired;
    int32_t cb_old_level;
    int32_t cb_new_level;
    Bool cb_feedback;
};

static uint8_t cb_is_template_trainable(void* ctx) {
    auto* c = static_cast<FfiCallbackBridgeCtx*>(ctx);
    return c->parent->getTemplate()->isTrainable();
}
static int32_t cb_get_experience_required(void* ctx, int32_t level) {
    auto* c = static_cast<FfiCallbackBridgeCtx*>(ctx);
    return c->parent->getTemplate()->getExperienceRequired(level);
}
static int32_t cb_get_experience_value(void* ctx, int32_t level) {
    auto* c = static_cast<FfiCallbackBridgeCtx*>(ctx);
    return c->parent->getTemplate()->getExperienceValue(level);
}
static int32_t cb_get_relationship_with_killer(void* /*ctx*/, const void* killer_ctx) {
    const auto* killer = static_cast<const Object*>(killer_ctx);
    return static_cast<int32_t>(killer->m_relWithTarget);
}
static void cb_on_veterancy_level_changed(void* ctx, int32_t old_level, int32_t new_level, uint8_t provide_feedback) {
    auto* c = static_cast<FfiCallbackBridgeCtx*>(ctx);
    c->cb_fired = TRUE;
    c->cb_old_level = old_level;
    c->cb_new_level = new_level;
    c->cb_feedback = provide_feedback;
}
static uint8_t cb_forward_experience_to_sink(void* ctx, int32_t /*sink_obj_id*/, int32_t scaled_exp_gain, uint8_t can_scale_for_bonus) {
    auto* c = static_cast<FfiCallbackBridgeCtx*>(ctx);
    if (!c->sink_alive || !c->sink_rust_state) return 0;
    zh_exp_tracker_add_experience_points(c->sink_rust_state, c->sink_rust_cb, scaled_exp_gain, can_scale_for_bonus);
    return 1;
}
static uint8_t cb_forward_set_exp_and_level_to_sink(void* ctx, int32_t /*sink_obj_id*/, int32_t exp_in, uint8_t provide_feedback) {
    auto* c = static_cast<FfiCallbackBridgeCtx*>(ctx);
    if (!c->sink_alive || !c->sink_rust_state) return 0;
    zh_exp_tracker_set_experience_and_level(c->sink_rust_state, c->sink_rust_cb, exp_in, provide_feedback);
    return 1;
}

static int run_ffi_differential_verification() {
    // 1. Verify PKT-001 (crc) across 1,000 seeds
    for (int i = 0; i < 1000; ++i) {
        HarnessRng rng(0x10010000u + static_cast<uint32_t>(i));
        uint32_t n1 = rng.next_u32() % 128;
        std::vector<uint8_t> buf(n1);
        for (uint32_t k = 0; k < n1; ++k) buf[k] = static_cast<uint8_t>(rng.next_u32() & 0xFF);
        int32_t len = (i == 1) ? -5 : static_cast<int32_t>(n1);

        CRC c;
        c.computeCRC(buf.empty() ? nullptr : buf.data(), len);
        uint32_t rust_crc = zh_crc_compute_bytes(0, buf.empty() ? nullptr : buf.data(), len);
        if (c.get() != rust_crc) {
            std::cerr << "FAIL PKT-001 seed " << i << ": C++=" << to_hex_u32(c.get())
                      << " Rust=" << to_hex_u32(rust_crc) << "\n";
            return 1;
        }
    }
    std::cout << "  [Gate V2-FFI] PKT-001 (crc): 1,000/1,000 differential seeds PASSED\n";

    // 2. Verify PKT-002 (RandomValue) across 1,000 seeds
    for (int i = 0; i < 1000; ++i) {
        HarnessRng rng(0x20020000u + static_cast<uint32_t>(i));
        uint32_t init_seed = (i < 16) ? static_cast<uint32_t>(i) : rng.next_u32();
        InitRandom(init_seed);
        zh_random_init_seed(init_seed);

        if (GetGameLogicRandomSeedCRC() != zh_random_get_logic_seed_crc()) {
            std::cerr << "FAIL PKT-002 init seed CRC at seed " << i << "\n";
            return 1;
        }

        for (int step = 0; step < 10; ++step) {
            int32_t lo_i = rng.next_i32(-50000, 50000);
            int32_t hi_i = rng.next_i32(-50000, 50000);
            float lo_f = static_cast<float>(rng.next_i32(-1000, 1000)) * 0.25f;
            float hi_f = static_cast<float>(rng.next_i32(-1000, 1000)) * 0.25f;

            if (GetGameLogicRandomValueUnchanged(lo_i, hi_i, "ffi", i) != zh_random_get_logic_int_unchanged(lo_i, hi_i)) return 1;
            if (float_to_bits(GetGameLogicRandomValueRealUnchanged(lo_f, hi_f, "ffi", i)) != float_to_bits(zh_random_get_logic_real_unchanged(lo_f, hi_f))) return 1;
            if (GetGameLogicRandomValue(lo_i, hi_i, "ffi", i) != zh_random_get_logic_int(lo_i, hi_i)) return 1;
            if (float_to_bits(GetGameLogicRandomValueReal(lo_f, hi_f, "ffi", i)) != float_to_bits(zh_random_get_logic_real(lo_f, hi_f))) return 1;
            if (GetGameClientRandomValue(lo_i, hi_i, "ffi", i) != zh_random_get_client_int(lo_i, hi_i)) return 1;
            if (float_to_bits(GetGameClientRandomValueReal(lo_f, hi_f, "ffi", i)) != float_to_bits(zh_random_get_client_real(lo_f, hi_f))) return 1;
            if (GetGameAudioRandomValue(lo_i, hi_i, "ffi", i) != zh_random_get_audio_int(lo_i, hi_i)) return 1;
            if (float_to_bits(GetGameAudioRandomValueReal(lo_f, hi_f, "ffi", i)) != float_to_bits(zh_random_get_audio_real(lo_f, hi_f))) return 1;
            if (GetGameLogicRandomSeedCRC() != zh_random_get_logic_seed_crc()) return 1;
        }
    }
    std::cout << "  [Gate V2-FFI] PKT-002 (RandomValue): 1,000/1,000 differential seeds (10,000 steps) PASSED\n";

    // 3. Verify PKT-003 (ExperienceTracker) across 1,000 seeds
    for (int i = 0; i < 1000; ++i) {
        HarnessRng rng(0x30030000u + static_cast<uint32_t>(i));
        g_harnessGameLogic.m_objects.clear();
        Object parentObj, sinkObj, killerObj;
        parentObj.m_id = static_cast<ObjectID>(1);
        parentObj.m_template.m_isTrainable = (rng.next_u32() % 10 != 0) ? TRUE : FALSE;
        parentObj.m_template.m_expRequired[0] = 0;
        parentObj.m_template.m_expRequired[1] = rng.next_i32(20, 80);
        parentObj.m_template.m_expRequired[2] = parentObj.m_template.m_expRequired[1] + rng.next_i32(30, 120);
        parentObj.m_template.m_expRequired[3] = parentObj.m_template.m_expRequired[2] + rng.next_i32(50, 200);
        sinkObj.m_id = static_cast<ObjectID>(2);
        sinkObj.m_template = parentObj.m_template;
        sinkObj.m_template.m_isTrainable = TRUE;
        killerObj.m_relWithTarget = static_cast<Relationship>(rng.next_u32() % 3);

        ExperienceTracker cppTracker(&parentObj);
        ExperienceTracker cppSinkTracker(&sinkObj);
        parentObj.m_tracker = &cppTracker;
        sinkObj.m_tracker = &cppSinkTracker;

        FfiCallbackBridgeCtx sinkCtx{&sinkObj, false, nullptr, nullptr, FALSE, 0, 0, FALSE};
        ZhExpTrackerParentCallbacks sinkCb{
            &sinkCtx,
            cb_is_template_trainable,
            cb_get_experience_required,
            cb_get_experience_value,
            cb_get_relationship_with_killer,
            cb_on_veterancy_level_changed,
            cb_forward_experience_to_sink,
            cb_forward_set_exp_and_level_to_sink
        };
        ZhExpTrackerState rustSinkState{};
        zh_exp_tracker_init(&rustSinkState, &sinkCb);

        bool sink_alive = (i % 8 == 1);
        if (sink_alive) {
            g_harnessGameLogic.m_objects[sinkObj.m_id] = &sinkObj;
        }
        FfiCallbackBridgeCtx parentCtx{&parentObj, sink_alive, &rustSinkState, &sinkCb, FALSE, 0, 0, FALSE};
        ZhExpTrackerParentCallbacks parentCb{
            &parentCtx,
            cb_is_template_trainable,
            cb_get_experience_required,
            cb_get_experience_value,
            cb_get_relationship_with_killer,
            cb_on_veterancy_level_changed,
            cb_forward_experience_to_sink,
            cb_forward_set_exp_and_level_to_sink
        };
        ZhExpTrackerState rustState{};
        zh_exp_tracker_init(&rustState, &parentCb);

        static const float kScalars[8] = {1.0f, 1.25f, 0.75f, 1.5f, 2.0f, 0.5f, 1.3333334f, 1.1f};
        float scalar = kScalars[rng.next_u32() % 8];
        cppTracker.setExperienceScalar(scalar);
        rustState.experience_scalar = scalar;

        if (i % 8 == 1) {
            cppTracker.setExperienceSink(sinkObj.m_id);
            rustState.experience_sink = 2;
        } else if (i % 8 == 2) {
            cppTracker.setExperienceSink(static_cast<ObjectID>(99));
            rustState.experience_sink = 99;
        }

        int32_t gain = rng.next_i32(-10, 220);
        uint8_t can_scale = static_cast<uint8_t>(rng.next_u32() & 1);
        cppTracker.addExperiencePoints(gain, can_scale);
        zh_exp_tracker_add_experience_points(&rustState, &parentCb, gain, can_scale);

        HarnessXferCRC xferCrc;
        cppTracker.crc(&xferCrc);
        uint32_t rustCrc = zh_exp_tracker_crc(&rustState, 0);

        HarnessXferSave xferSave;
        cppTracker.xfer(&xferSave);
        uint8_t rustBytes[64] = {0};
        int32_t rustLen = zh_exp_tracker_xfer_save(&rustState, rustBytes, 64);

        if (static_cast<int32_t>(cppTracker.getVeterancyLevel()) != rustState.current_level ||
            cppTracker.getCurrentExperience() != rustState.current_experience ||
            xferCrc.getCRC() != rustCrc ||
            static_cast<int32_t>(xferSave.bytes.size()) != rustLen ||
            std::memcmp(xferSave.bytes.data(), rustBytes, static_cast<size_t>(rustLen)) != 0 ||
            cppSinkTracker.getCurrentExperience() != rustSinkState.current_experience) {
            std::cerr << "FAIL PKT-003 seed " << i << "\n";
            return 1;
        }
    }
    std::cout << "  [Gate V2-FFI] PKT-003 (ExperienceTracker): 1,000/1,000 differential seeds PASSED\n";
    return 0;
}
#endif

int main(int argc, char** argv) {
    if (argc == 3 && std::string(argv[1]) == "--emit-golden") {
        std::string out_dir = argv[2];
        emit_pkt001_golden(out_dir);
        emit_pkt002_golden(out_dir);
        emit_pkt003_golden(out_dir);
        std::cout << "Generated 1,000-seed C++ Golden Vectors in " << out_dir
                  << "/PKT-001.json, PKT-002.json, PKT-003.json\n";
        return 0;
    }
#ifdef VERIFY_FFI
    if (argc == 2 && std::string(argv[1]) == "--verify-ffi") {
        return run_ffi_differential_verification();
    }
#endif
    std::cerr << "Usage: " << argv[0] << " --emit-golden <dir> | --verify-ffi\n";
    return 1;
}
