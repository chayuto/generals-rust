// PKT-444 (MessageStream) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX/Core/GameEngine/Source/Common/MessageStream.cpp
// in Release configuration (RTS_DEBUG, _ALLOW_DEBUG_CHEATS_IN_RELEASE, ALLOW_SURRENDER,
// and DUMP_PERF_STATS all undefined).
//
// Canonical build & run command (macos arm64):
//   clang++ -std=c++17 -O1 -ffp-contract=off \
//     -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE -UALLOW_SURRENDER -UDUMP_PERF_STATS \
//     -Wno-switch \
//     -Itests/cpp_harness/stubs_pkt444 \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     tests/cpp_harness/gen_pkt444_messagestream.cpp \
//     -o /tmp/pkt444_o1 && /tmp/pkt444_o1 docs/golden_vectors/PKT-444.json
//
// Determinism & UB verification: build & run at -O0, -O2, and -fsanitize=address,undefined;
// all JSON outputs must be byte-identical.
//
// __APPLE__ audit: 0 __APPLE__ branches in MessageStream.h / MessageStream.cpp.

#include "PreRTS.h"
#include "Common/MessageStream.h"
#include "Common/PlayerList.h"
#include "Common/Recorder.h"
#include "GameLogic/GameLogic.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/MessageStream.cpp"

const AsciiString AsciiString::TheEmptyString("");
static PlayerList g_player_list;
PlayerList* ThePlayerList = &g_player_list;
static Recorder g_recorder;
Recorder* TheRecorder = &g_recorder;
static GameLogic g_game_logic;
GameLogic* TheGameLogic = &g_game_logic;

static std::string sha256_file(const char* path) {
    static const uint32_t K[64] = {
        0x428a2f98u, 0x71374491u, 0xb5c0fbcfu, 0xe9b5dba5u, 0x3956c25bu, 0x59f111f1u, 0x923f82a4u, 0xab1c5ed5u,
        0xd807aa98u, 0x12835b01u, 0x243185beu, 0x550c7dc3u, 0x72be5d74u, 0x80deb1feu, 0x9bdc06a7u, 0xc19bf174u,
        0xe49b69c1u, 0xefbe4786u, 0x0fc19dc6u, 0x240ca1ccu, 0x2de92c6fu, 0x4a7484aau, 0x5cb0a9dcu, 0x76f988dau,
        0x983e5152u, 0xa831c66du, 0xb00327c8u, 0xbf597fc7u, 0xc6e00bf3u, 0xd5a79147u, 0x06ca6351u, 0x14292967u,
        0x27b70a85u, 0x2e1b2138u, 0x4d2c6dfcu, 0x53380d13u, 0x650a7354u, 0x766a0abbu, 0x81c2c92eu, 0x92722c85u,
        0xa2bfe8a1u, 0xa81a664bu, 0xc24b8b70u, 0xc76c51a3u, 0xd192e819u, 0xd6990624u, 0xf40e3585u, 0x106aa070u,
        0x19a4c116u, 0x1e376c08u, 0x2748774cu, 0x34b0bcb5u, 0x391c0cb3u, 0x4ed8aa4au, 0x5b9cca4fu, 0x682e6ff3u,
        0x748f82eeu, 0x78a5636fu, 0x84c87814u, 0x8cc70208u, 0x90befffau, 0xa4506cebu, 0xbef9a3f7u, 0xc67178f2u
    };
    FILE* f = std::fopen(path, "rb");
    if (!f) { std::perror(path); std::exit(1); }
    std::vector<uint8_t> data;
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (sz > 0) {
        data.resize(static_cast<size_t>(sz));
        if (std::fread(data.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) {
            std::perror("fread"); std::exit(1);
        }
    }
    std::fclose(f);
    uint64_t bit_len = static_cast<uint64_t>(data.size()) * 8ULL;
    data.push_back(0x80u);
    while ((data.size() % 64u) != 56u) data.push_back(0x00u);
    for (int shift = 56; shift >= 0; shift -= 8) {
        data.push_back(static_cast<uint8_t>((bit_len >> shift) & 0xFFu));
    }
    uint32_t H[8] = {
        0x6a09e667u, 0xbb67ae85u, 0x3c6ef372u, 0xa54ff53au,
        0x510e527fu, 0x9b05688cu, 0x1f83d9abu, 0x5be0cd19u
    };
    auto rotr = [](uint32_t x, uint32_t n) -> uint32_t { return (x >> n) | (x << (32u - n)); };
    for (size_t off = 0; off < data.size(); off += 64u) {
        uint32_t w[64]{};
        for (int i = 0; i < 16; ++i) {
            w[i] = (static_cast<uint32_t>(data[off + i * 4]) << 24) |
                   (static_cast<uint32_t>(data[off + i * 4 + 1]) << 16) |
                   (static_cast<uint32_t>(data[off + i * 4 + 2]) << 8) |
                   static_cast<uint32_t>(data[off + i * 4 + 3]);
        }
        for (int i = 16; i < 64; ++i) {
            uint32_t s0 = rotr(w[i - 15], 7) ^ rotr(w[i - 15], 18) ^ (w[i - 15] >> 3);
            uint32_t s1 = rotr(w[i - 2], 17) ^ rotr(w[i - 2], 19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16] + s0 + w[i - 7] + s1;
        }
        uint32_t a = H[0], b = H[1], c = H[2], d = H[3];
        uint32_t e = H[4], f0 = H[5], g = H[6], h = H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t S1 = rotr(e, 6) ^ rotr(e, 11) ^ rotr(e, 25);
            uint32_t ch = (e & f0) ^ ((~e) & g);
            uint32_t temp1 = h + S1 + ch + K[i] + w[i];
            uint32_t S0 = rotr(a, 2) ^ rotr(a, 13) ^ rotr(a, 22);
            uint32_t maj = (a & b) ^ (a & c) ^ (b & c);
            uint32_t temp2 = S0 + maj;
            h = g; g = f0; f0 = e; e = d + temp1;
            d = c; c = b; b = a; a = temp1 + temp2;
        }
        H[0] += a; H[1] += b; H[2] += c; H[3] += d;
        H[4] += e; H[5] += f0; H[6] += g; H[7] += h;
    }
    char hex[65];
    for (int i = 0; i < 8; ++i) std::snprintf(hex + i * 8, 9, "%08x", H[i]);
    hex[64] = '\0';
    return std::string(hex);
}

static uint32_t splitmix32(uint32_t& s) {
    uint32_t z = (s += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint32_t fbits(Real v) {
    uint32_t b = 0;
    std::memcpy(&b, &v, 4);
    return b;
}

class RecordingTranslator : public GameMessageTranslator {
public:
    UnsignedInt m_tag;
    GameMessage::Type m_destroy_type;
    std::vector<Int>* m_trace;

    RecordingTranslator(UnsignedInt tag, GameMessage::Type destroy_type, std::vector<Int>* trace)
        : m_tag(tag), m_destroy_type(destroy_type), m_trace(trace) {}

    GameMessageDisposition translateGameMessage(const GameMessage* msg) override {
        m_trace->push_back(static_cast<Int>(m_tag));
        m_trace->push_back(static_cast<Int>(msg->getType()));
        return (msg->getType() == m_destroy_type) ? DESTROY_MESSAGE : KEEP_MESSAGE;
    }
};

static void run_startup_self_checks() {
    if (GameMessage::MSG_BEGIN_NETWORK_MESSAGES != 1000 ||
        GameMessage::MSG_END_NETWORK_MESSAGES != 1999) {
        std::fprintf(stderr, "Self-check failed: network message bounds shifted (%d..%d)\n",
                     GameMessage::MSG_BEGIN_NETWORK_MESSAGES, GameMessage::MSG_END_NETWORK_MESSAGES);
        std::exit(1);
    }
    if (std::strcmp(GameMessage::getCommandTypeAsString(GameMessage::MSG_COUNT), "Invalid command") != 0 ||
        std::strcmp(GameMessage::getCommandTypeAsString(static_cast<GameMessage::Type>(999)), "UnknownMessage") != 0) {
        std::fprintf(stderr, "Self-check failed: getCommandTypeAsString fallbacks unexpected\n");
        std::exit(1);
    }
    GameMessage m(GameMessage::MSG_DO_MOVETO);
    if (m.getArgumentCount() != 0 ||
        m.getArgumentDataType(0) != ARGUMENTDATATYPE_UNKNOWN ||
        m.getArgumentDataType(-1) != ARGUMENTDATATYPE_UNKNOWN ||
        m.getArgument(0)->integer != 0 ||
        m.getArgument(-1)->timestamp != 0) {
        std::fprintf(stderr, "Self-check failed: out-of-range getArgument fallback mismatch\n");
        std::exit(1);
    }
    ICoord2D a{100, -20}, b{-50, 80};
    IRegion2D r{};
    buildRegion(&a, &b, &r);
    if (r.lo.x != -50 || r.hi.x != 100 || r.lo.y != -20 || r.hi.y != 80) {
        std::fprintf(stderr, "Self-check failed: buildRegion mismatch\n");
        std::exit(1);
    }
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: gen_pkt444_messagestream <out.json>\n");
        return 1;
    }
    run_startup_self_checks();

    TheMessageStream = new MessageStream();
    TheCommandList = new CommandList();
    TheMessageStream->init();
    TheCommandList->init();

    const char* source_paths[] = {
        "vendor/GeneralsX/Core/GameEngine/Include/Common/MessageStream.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/MessageStream.cpp"
    };

    const char* out_path = argv[1];
    std::string tmp_out = std::string(out_path) + ".tmp";
    FILE* fp = std::fopen(tmp_out.c_str(), "wb");
    if (!fp) { std::perror(tmp_out.c_str()); return 1; }

    std::fprintf(fp, "{\n  \"packet_id\": \"PKT-444\",\n"
                     "  \"module\": \"MessageStream\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified MessageStream.cpp (Release build config)\",\n"
                     "  \"msg_begin_network\": %d,\n"
                     "  \"msg_end_network\": %d,\n"
                     "  \"msg_begin_debug_network\": %d,\n"
                     "  \"msg_count\": %d,\n"
                     "  \"source_sha256\": {\n",
                 static_cast<Int>(GameMessage::MSG_BEGIN_NETWORK_MESSAGES),
                 static_cast<Int>(GameMessage::MSG_END_NETWORK_MESSAGES),
                 static_cast<Int>(GameMessage::MSG_BEGIN_DEBUG_NETWORK_MESSAGES),
                 static_cast<Int>(GameMessage::MSG_COUNT));
    for (size_t i = 0; i < ARRAY_SIZE(source_paths); ++i) {
        std::string digest = sha256_file(source_paths[i]);
        std::fprintf(fp, "    \"%s\": \"%s\"%s\n",
                     source_paths[i], digest.c_str(),
                     (i + 1 < ARRAY_SIZE(source_paths)) ? "," : "");
    }
    std::fprintf(fp, "  },\n  \"enum_table\": [\n");
    for (Int t = 0; t <= GameMessage::MSG_COUNT; ++t) {
        const char* label = GameMessage::getCommandTypeAsString(static_cast<GameMessage::Type>(t));
        if (t <= 260 || t >= 998 || std::strcmp(label, "UnknownMessage") != 0) {
            std::fprintf(fp, "    [%d, \"%s\"]%s\n", t, label, (t < GameMessage::MSG_COUNT) ? "," : "");
        }
    }
    std::fprintf(fp, "  ],\n  \"vector_count\": 1000,\n  \"vectors\": [\n");

    const GameMessage::Type sample_types[] = {
        GameMessage::MSG_INVALID,
        GameMessage::MSG_FRAME_TICK,
        GameMessage::MSG_RAW_MOUSE_POSITION,
        GameMessage::MSG_MOUSE_LEFT_CLICK,
        GameMessage::MSG_CREATE_SELECTED_GROUP,
        GameMessage::MSG_CREATE_SELECTED_GROUP_NO_SOUND,
        GameMessage::MSG_DESTROY_SELECTED_GROUP,
        GameMessage::MSG_SELECT_TEAM0,
        GameMessage::MSG_SELECT_TEAM9,
        GameMessage::MSG_DO_MOVETO,
        GameMessage::MSG_DO_ATTACK_OBJECT,
        GameMessage::MSG_SET_REPLAY_CAMERA,
        GameMessage::MSG_LOGIC_CRC,
        GameMessage::MSG_ENABLE_RETALIATION_MODE,
        GameMessage::MSG_TIMESTAMP,
        GameMessage::MSG_OBJECT_JOINED_TEAM
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x243f6a88u;
        TheMessageStream->reset();
        TheCommandList->reset();

        Int player_idx = static_cast<Int>(seed % 8u);
        g_player_list.setLocalPlayerIndex(player_idx);

        // Part A: Argument storage + buildRegion + command string
        GameMessage::Type query_type = sample_types[seed % ARRAY_SIZE(sample_types)];
        const char* cmd_str = GameMessage::getCommandTypeAsString(query_type);
        Int arg_i = (seed == 0) ? INT32_MIN : static_cast<Int>(splitmix32(rng));
        Real arg_r = (seed == 0) ? -0.0f : static_cast<Real>(static_cast<Int>(splitmix32(rng) % 2001u) - 1000) * 0.25f;
        Bool arg_b = (splitmix32(rng) & 1u) ? TRUE : FALSE;
        ObjectID arg_obj = static_cast<ObjectID>(splitmix32(rng));
        DrawableID arg_draw = static_cast<DrawableID>(splitmix32(rng));
        UnsignedInt arg_team = splitmix32(rng);
        Coord3D arg_loc {
            static_cast<Real>(static_cast<Int>(splitmix32(rng) % 1001u) - 500),
            static_cast<Real>(static_cast<Int>(splitmix32(rng) % 1001u) - 500),
            static_cast<Real>(static_cast<Int>(splitmix32(rng) % 1001u) - 500)
        };
        ICoord2D anchor {
            static_cast<Int>(splitmix32(rng) % 2001u) - 1000,
            static_cast<Int>(splitmix32(rng) % 2001u) - 1000
        };
        ICoord2D dest {
            static_cast<Int>(splitmix32(rng) % 2001u) - 1000,
            static_cast<Int>(splitmix32(rng) % 2001u) - 1000
        };
        IRegion2D region{};
        buildRegion(&anchor, &dest, &region);
        UnsignedInt arg_ts = splitmix32(rng);
        WideChar arg_wc = static_cast<WideChar>(0x0041u + (splitmix32(rng) % 0x4000u));

        GameMessage single_msg(query_type);
        single_msg.appendIntegerArgument(arg_i);
        single_msg.appendRealArgument(arg_r);
        single_msg.appendBooleanArgument(arg_b);
        single_msg.appendObjectIDArgument(arg_obj);
        single_msg.appendDrawableIDArgument(arg_draw);
        single_msg.appendTeamIDArgument(arg_team);
        single_msg.appendLocationArgument(arg_loc);
        single_msg.appendPixelArgument(anchor);
        single_msg.appendPixelRegionArgument(region);
        single_msg.appendTimestampArgument(arg_ts);
        single_msg.appendWideCharArgument(arg_wc);
        if (single_msg.getArgumentCount() != 11) {
            std::fprintf(stderr, "Seed %u: expected 11 arguments\n", seed);
            return 1;
        }

        // Part B: Translator priority tie-breaking, removal, insertMessage, propagateMessages, redundancy
        std::vector<Int> trace;
        UnsignedInt prios[4];
        prios[0] = (seed == 0) ? 10u : (splitmix32(rng) % 5u);
        prios[1] = (seed == 0) ? 10u : prios[0]; // Equal priority tests FIFO preservation!
        prios[2] = (seed == 0) ? 5u  : (splitmix32(rng) % 5u);
        prios[3] = (seed == 0) ? 20u : (splitmix32(rng) % 5u);
        GameMessage::Type drop0 = (seed % 2 == 0) ? GameMessage::MSG_RAW_MOUSE_POSITION : GameMessage::MSG_FRAME_TICK;
        GameMessage::Type drop1 = GameMessage::MSG_INVALID;
        GameMessage::Type drop2 = GameMessage::MSG_MOUSE_LEFT_CLICK;
        GameMessage::Type drop3 = GameMessage::MSG_RAW_MOUSE_POSITION;

        TranslatorID tid0 = TheMessageStream->attachTranslator(new RecordingTranslator(10, drop0, &trace), prios[0]);
        TranslatorID tid1 = TheMessageStream->attachTranslator(new RecordingTranslator(11, drop1, &trace), prios[1]);
        TranslatorID tid2 = TheMessageStream->attachTranslator(new RecordingTranslator(12, drop2, &trace), prios[2]);
        TranslatorID tid3 = TheMessageStream->attachTranslator(new RecordingTranslator(13, drop3, &trace), prios[3]);
        Int remove_which = static_cast<Int>(splitmix32(rng) % 4u);
        TranslatorID tids[4] = { tid0, tid1, tid2, tid3 };
        if (seed > 0) {
            TheMessageStream->removeTranslator(tids[remove_which]);
        }

        // Append deterministic command sequence exercising redundancy patterns + insertMessage
        GameMessage::Type seq_types[8]{};
        Bool seq_has_bool[8]{};
        Bool seq_bool_val[8]{};
        Int seq_count = 0;

        if (seed == 0) {
            // Craft Destroy->Create(true) (redundant!), Destroy->Create(false) (keep!),
            // SetReplayCamera (redundant because later SetReplayCamera exists!), raw msg dropped
            seq_types[0] = GameMessage::MSG_SET_REPLAY_CAMERA;
            seq_types[1] = GameMessage::MSG_DESTROY_SELECTED_GROUP;
            seq_types[2] = GameMessage::MSG_CREATE_SELECTED_GROUP; seq_has_bool[2] = TRUE; seq_bool_val[2] = TRUE;
            seq_types[3] = GameMessage::MSG_RAW_MOUSE_POSITION; // dropped by translator
            seq_types[4] = GameMessage::MSG_DESTROY_SELECTED_GROUP;
            seq_types[5] = GameMessage::MSG_CREATE_SELECTED_GROUP_NO_SOUND; seq_has_bool[5] = TRUE; seq_bool_val[5] = FALSE;
            seq_types[6] = GameMessage::MSG_SET_REPLAY_CAMERA;
            seq_count = 7;
        } else if (seed == 1) {
            seq_types[0] = GameMessage::MSG_DESTROY_SELECTED_GROUP;
            seq_types[1] = GameMessage::MSG_DESTROY_SELECTED_GROUP; // first dropped
            seq_types[2] = GameMessage::MSG_SELECT_TEAM3;            // second dropped
            seq_types[3] = GameMessage::MSG_DESTROY_SELECTED_GROUP;
            seq_types[4] = GameMessage::MSG_CREATE_SELECTED_GROUP; // 0 args -> NOT redundant!
            seq_types[5] = GameMessage::MSG_DO_MOVETO;
            seq_count = 6;
        } else {
            seq_count = 4 + static_cast<Int>(splitmix32(rng) % 5u);
            for (Int i = 0; i < seq_count; ++i) {
                seq_types[i] = sample_types[splitmix32(rng) % ARRAY_SIZE(sample_types)];
                if (seq_types[i] == GameMessage::MSG_CREATE_SELECTED_GROUP ||
                    seq_types[i] == GameMessage::MSG_CREATE_SELECTED_GROUP_NO_SOUND) {
                    seq_has_bool[i] = (splitmix32(rng) % 3u != 0u) ? TRUE : FALSE;
                    seq_bool_val[i] = (splitmix32(rng) & 1u) ? TRUE : FALSE;
                }
            }
        }

        bool use_insert2 = (seed >= 2 && seq_count >= 3);
        GameMessage* inserted_after = nullptr;
        for (Int i = 0; i < seq_count; ++i) {
            GameMessage* m = (use_insert2 && i == 2 && inserted_after != nullptr)
                ? TheMessageStream->insertMessage(seq_types[i], inserted_after)
                : TheMessageStream->appendMessage(seq_types[i]);
            if (seq_has_bool[i]) {
                m->appendBooleanArgument(seq_bool_val[i]);
            }
            if (i == 0) inserted_after = m;
        }

        Bool contains_replay_cam = TheMessageStream->containsMessageOfType(GameMessage::MSG_SET_REPLAY_CAMERA);
        TheMessageStream->propagateMessages();

        // Collect surviving TheCommandList message types & player indices, and trace
        std::vector<Int> survivors;
        for (GameMessage* m = TheCommandList->getFirstMessage(); m; m = m->next()) {
            survivors.push_back(static_cast<Int>(m->getType()));
        }

        // Remove remaining translators so next reset starts with fresh MessageStream translator IDs?
        // Note: reset() does NOT clear translators in C++, only clear messages! Keep or recreate.
        delete TheMessageStream;
        TheMessageStream = new MessageStream();
        TheMessageStream->init();

        std::fprintf(fp,
            "    {\"seed\": %u, \"player_idx\": %d, \"query_type\": %d, \"cmd_str\": \"%s\", "
            "\"arg_i\": %d, \"arg_r_bits\": \"0x%08x\", \"arg_b\": %u, \"arg_obj\": %d, "
            "\"arg_draw\": %d, \"arg_team\": %u, \"loc_x_bits\": \"0x%08x\", \"loc_y_bits\": \"0x%08x\", \"loc_z_bits\": \"0x%08x\", "
            "\"ax\": %d, \"ay\": %d, \"dx\": %d, \"dy\": %d, "
            "\"rlo_x\": %d, \"rlo_y\": %d, \"rhi_x\": %d, \"rhi_y\": %d, "
            "\"arg_ts\": %u, \"arg_wc\": %u, "
            "\"prios\": [%u, %u, %u, %u], \"remove_idx\": %d, \"insert2\": %u, \"contains_cam\": %u, "
            "\"seq_types\": [",
            seed, player_idx, static_cast<Int>(query_type), cmd_str,
            arg_i, fbits(arg_r), static_cast<uint32_t>(arg_b), static_cast<Int>(arg_obj),
            static_cast<Int>(arg_draw), arg_team, fbits(arg_loc.x), fbits(arg_loc.y), fbits(arg_loc.z),
            anchor.x, anchor.y, dest.x, dest.y,
            region.lo.x, region.lo.y, region.hi.x, region.hi.y,
            arg_ts, static_cast<uint32_t>(arg_wc),
            prios[0], prios[1], prios[2], prios[3], (seed == 0 ? -1 : remove_which),
            use_insert2 ? 1u : 0u,
            static_cast<uint32_t>(contains_replay_cam));
        for (Int i = 0; i < seq_count; ++i) {
            std::fprintf(fp, "%d%s", static_cast<Int>(seq_types[i]), (i + 1 < seq_count) ? ", " : "");
        }
        std::fprintf(fp, "], \"seq_bool_mode\": [");
        for (Int i = 0; i < seq_count; ++i) {
            Int bm = !seq_has_bool[i] ? -1 : (seq_bool_val[i] ? 1 : 0);
            std::fprintf(fp, "%d%s", bm, (i + 1 < seq_count) ? ", " : "");
        }
        std::fprintf(fp, "], \"trace\": [");
        for (size_t i = 0; i < trace.size(); ++i) {
            std::fprintf(fp, "%d%s", trace[i], (i + 1 < trace.size()) ? ", " : "");
        }
        std::fprintf(fp, "], \"survivors\": [");
        for (size_t i = 0; i < survivors.size(); ++i) {
            std::fprintf(fp, "%d%s", survivors[i], (i + 1 < survivors.size()) ? ", " : "");
        }
        std::fprintf(fp, "]}%s\n", (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    if (std::fflush(fp) != 0 || std::fclose(fp) != 0) {
        std::perror("flush/close");
        std::remove(tmp_out.c_str());
        return 1;
    }
    if (std::rename(tmp_out.c_str(), out_path) != 0) {
        std::perror("rename");
        return 1;
    }

    delete TheCommandList;
    TheCommandList = nullptr;
    delete TheMessageStream;
    TheMessageStream = nullptr;
    return 0;
}
