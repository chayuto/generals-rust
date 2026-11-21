// PKT-116 (AcademyStats) (WP-3.1) C++ Golden Vector Generator.
// Compiles unmodified vendor/GeneralsX AcademyStats.cpp + Xfer*.cpp + crc.cpp + RandomValue.cpp.
//
// Canonical build & run command (macOS arm64, Release-aligned macros):
//   clang++ -std=c++17 -O1 -ffp-contract=off -URTS_DEBUG -U_ALLOW_DEBUG_CHEATS_IN_RELEASE \
//     -DRTS_ZEROHOUR=1 -DRTS_GENERALS=0 -DRETAIL_COMPATIBLE_CRC=0 -DRETAIL_COMPATIBLE_XFER_SAVE=1 \
//     -Wno-macro-redefined -Wno-switch \
//     -Itests/cpp_harness/stubs_pkt116 \
//     -Ivendor/GeneralsX/GeneralsMD/Code/GameEngine/Include \
//     -Ivendor/GeneralsX/Core/GameEngine/Include \
//     -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
//     tests/cpp_harness/gen_pkt116_academy_stats.cpp \
//     vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a \
//     -o /tmp/gen_pkt116 && /tmp/gen_pkt116 docs/golden_vectors/PKT-116.json
//
// Determinism & UB verification builds (must produce byte-identical JSON):
//   -O0, -O1, -O2, and -O1 -g -fsanitize=address,undefined.

#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/RandomValue.h"
#include "GameClient/ClientRandomValue.h"
#include "GameLogic/LogicRandomValue.h"
#include "Common/AudioRandomValue.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/AcademyStats.h"
#include "Common/Energy.h"
#include "Common/Money.h"
#include "Common/PlayerTemplate.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Science.h"
#include "Common/SpecialPower.h"
#include "Common/ThingTemplate.h"
#include "Common/Upgrade.h"
#include "GameClient/ControlBar.h"
#include "GameClient/GameText.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"
#include "GameLogic/Module/ContainModule.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/crc.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RandomValue.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/RTS/AcademyStats.cpp"

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

static DynamicMemoryAllocator g_dma;
DynamicMemoryAllocator* TheDynamicMemoryAllocator = &g_dma;

static GameStateStub g_game_state;
GameStateStub* TheGameState = &g_game_state;

static ScienceStoreStub g_science_store;
ScienceStoreStub* TheScienceStore = &g_science_store;

static UpgradeCenterStub g_upgrade_center;
UpgradeCenterStub* TheUpgradeCenter = &g_upgrade_center;

static GlobalDataStub g_global_data;
GlobalDataStub* TheGlobalData = &g_global_data;

static ControlBar g_control_bar;
ControlBar* TheControlBar = &g_control_bar;

static GameTextInterface g_game_text;
GameTextInterface* TheGameText = &g_game_text;

static PlayerList g_player_list;
PlayerList* ThePlayerList = &g_player_list;

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
            std::perror("fread");
            std::exit(1);
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

static std::vector<uint8_t> read_file_bytes(const char* path) {
    FILE* fp = std::fopen(path, "rb");
    if (!fp) { std::perror(path); std::exit(1); }
    std::fseek(fp, 0, SEEK_END);
    long sz = std::ftell(fp);
    std::fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> buf(static_cast<size_t>(sz));
    if (sz > 0 && std::fread(buf.data(), 1, buf.size(), fp) != buf.size()) {
        std::perror("fread");
        std::exit(1);
    }
    std::fclose(fp);
    return buf;
}

static std::string hex_encode(const std::vector<uint8_t>& bytes) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(bytes.size() * 2);
    for (uint8_t b : bytes) {
        out.push_back(kHex[b >> 4]);
        out.push_back(kHex[b & 0x0f]);
    }
    return out;
}

static std::string advice_to_ascii(const UnicodeString& u) {
    std::string s;
    const WideChar* w = u.str();
    Int len = u.getLength();
    for (Int i = 0; i < len; ++i) {
        if (w[i] >= 32 && w[i] < 127) {
            s.push_back(static_cast<char>(w[i]));
        }
    }
    return s;
}

int main(int argc, char** argv) {
    if (argc != 2) {
        std::fprintf(stderr, "Usage: %s <output.json>\n", argv[0]);
        return 1;
    }

    const char* source_paths[2] = {
        "vendor/GeneralsX/Core/GameEngine/Include/Common/AcademyStats.h",
        "vendor/GeneralsX/Core/GameEngine/Source/Common/RTS/AcademyStats.cpp",
    };

    const char* tmp_save = "/tmp/pkt116_academy_save.bin";
    FILE* out = std::fopen(argv[1], "wb");
    if (!out) { std::perror(argv[1]); return 1; }

    std::fprintf(out, "{\n");
    std::fprintf(out, "  \"packet_id\": \"PKT-116\",\n");
    std::fprintf(out, "  \"work_package\": \"WP-3.1\",\n");
    std::fprintf(out, "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n");
    std::fprintf(out, "  \"vector_count\": 1000,\n");
    std::fprintf(out, "  \"constants\": {\n");
    std::fprintf(out, "    \"MAX_ADVICE_TIPS\": %d,\n", MAX_ADVICE_TIPS);
    std::fprintf(out, "    \"FRAMES_BETWEEN_UPDATES\": %d,\n", FRAMES_BETWEEN_UPDATES);
    std::fprintf(out, "    \"ACT_COUNT\": %d\n", ACT_COUNT);
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"source_sha256\": {\n");
    for (size_t i = 0; i < 2; ++i) {
        std::string sha = sha256_file(source_paths[i]);
        std::fprintf(out, "    \"%s\": \"%s\"%s\n", source_paths[i], sha.c_str(), i + 1 < 2 ? "," : "");
    }
    std::fprintf(out, "  },\n");
    std::fprintf(out, "  \"vectors\": [\n");

    static const char* kSides[5] = { "USA", "China", "GLA", "Observer", "" };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x74779640u + 0x9e3779b9u;
        InitRandom(seed);

        Int side_idx = (seed % 11 == 0) ? 3 : ((seed % 17 == 0) ? 4 : static_cast<Int>(seed % 3));
        PlayerTemplate pt;
        pt.m_baseSide.set(kSides[side_idx]);

        Player& plyr = ThePlayerList->m_localPlayer;
        Player& neut = ThePlayerList->m_neutralPlayer;
        plyr.m_template = (side_idx == 4) ? nullptr : &pt;
        plyr.m_objects.clear();
        neut.m_objects.clear();

        // Setup initial objects on player before init() / first update()
        Int init_obj_mask = static_cast<Int>(splitmix32(rng) & 0x3Fu);
        Object init_objs[3];
        Int init_obj_count = 0;
        if (init_obj_mask & 1) {
            init_objs[init_obj_count].m_kindMask = (1u << KINDOF_DOZER);
            plyr.m_objects.push_back(&init_objs[init_obj_count++]);
        }
        if (init_obj_mask & 2) {
            init_objs[init_obj_count].m_kindMask = (1u << KINDOF_FS_SUPPLY_CENTER);
            plyr.m_objects.push_back(&init_objs[init_obj_count++]);
        }
        if (init_obj_mask & 4) {
            init_objs[init_obj_count].m_kindMask = (1u << KINDOF_INFANTRY) | (1u << KINDOF_HERO);
            plyr.m_objects.push_back(&init_objs[init_obj_count++]);
        }

        UnsignedInt init_frame = splitmix32(rng) % 100u;
        TheGameLogic->m_frame = init_frame;
        TheGlobalData->m_useAlternateMouse = (splitmix32(rng) & 1u) ? TRUE : FALSE;

        AcademyStats stats;
        stats.init(&plyr);
        neut.m_academyStats.init(&neut);

        UnsignedInt local_mines = (splitmix32(rng) % 3 == 0) ? (1u + (splitmix32(rng) % 5u)) : 0u;
        UnsignedInt neut_sniped = (splitmix32(rng) % 3 == 0) ? (1u + (splitmix32(rng) % 4u)) : 0u;
        plyr.m_academyStats. init(&plyr);
        for (UnsignedInt m = 0; m < local_mines; ++m) {
            plyr.m_academyStats.recordMine();
            stats.recordMine();
        }
        for (UnsignedInt v = 0; v < neut_sniped; ++v) {
            neut.m_academyStats.recordVehicleSniped();
        }

        Int ops[16][4]{};
        for (int s = 0; s < 16; ++s) {
            Int op_kind = static_cast<Int>(splitmix32(rng) % 14u);
            Int a1 = 0, a2 = 0;
            Int frame_step = (s == 0 && (seed % 3 == 0))
                ? static_cast<Int>(splitmix32(rng) % 30u)
                : static_cast<Int>(splitmix32(rng) % 1500u);
            if (seed % 5 == 1 && s < 7) {
                // Satisfy all Tier 1 conditions, leaving Tier 2 conditions open for selection
                static const Int kTier1OnlyOps[7] = { 0, 1, 2, 3, 4, 5, 6 };
                op_kind = kTier1OnlyOps[s];
                frame_step = 15;
            } else if (seed % 5 == 2 && s < 15) {
                static const Int kTier2ClearOps[15] = { 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 1 };
                op_kind = kTier2ClearOps[s];
                frame_step = 15;
            }
            TheGameLogic->m_frame += static_cast<UnsignedInt>(frame_step);

            switch (op_kind) {
                case 0: { // update() with money & power state
                    a1 = static_cast<Int>(splitmix32(rng) % 2500u);
                    a2 = static_cast<Int>(splitmix32(rng) & 1u);
                    plyr.m_money.m_money = static_cast<UnsignedInt>(a1);
                    plyr.m_energy.m_sufficientPower = a2 ? TRUE : FALSE;
                    stats.update();
                    break;
                }
                case 1: { // recordProduction: supply center + 2 dozers + harvester + hero + secondary income
                    a1 = (seed % 5 == 1 || seed % 5 == 2)
                        ? ((1 << KINDOF_FS_SUPPLY_CENTER) | (1 << KINDOF_DOZER) | (1 << KINDOF_HARVESTER) | (1 << KINDOF_HERO) | (1 << KINDOF_MONEY_HACKER))
                        : static_cast<Int>(splitmix32(rng) & 0xFFFEu);
                    a2 = static_cast<Int>(splitmix32(rng) & 1u);
                    Object prod_obj;
                    prod_obj.m_kindMask = static_cast<uint32_t>(a1);
                    prod_obj.m_hasContain = a2 ? TRUE : FALSE;
                    prod_obj.m_contain.m_isTunnel = a2 ? TRUE : FALSE;
                    stats.recordProduction(&prod_obj, nullptr);
                    if (seed % 5 == 1 || seed % 5 == 2) {
                        // Also record a military unit & 2nd supply center / dozer
                        Object mil_obj;
                        mil_obj.m_kindMask = (1u << KINDOF_INFANTRY) | (1u << KINDOF_FS_SUPPLY_CENTER);
                        stats.recordProduction(&mil_obj, nullptr);
                        Object dozer2;
                        dozer2.m_kindMask = (1u << KINDOF_DOZER);
                        stats.recordProduction(&dozer2, nullptr);
                    }
                    break;
                }
                case 2: { // recordUpgrade
                    a1 = (seed % 5 == 1 || seed % 5 == 2) ? ACT_UPGRADE_RADAR : static_cast<Int>(splitmix32(rng) % ACT_COUNT);
                    a2 = (seed % 5 == 1 || seed % 5 == 2) ? 0 : static_cast<Int>(splitmix32(rng) & 1u);
                    UpgradeTemplate upg;
                    upg.m_class = static_cast<AcademyClassificationType>(a1);
                    stats.recordUpgrade(&upg, a2 ? TRUE : FALSE);
                    break;
                }
                case 3: { // recordSpecialPowerUsed
                    a1 = (seed % 5 == 1 || seed % 5 == 2) ? ACT_SUPERPOWER : static_cast<Int>(splitmix32(rng) % ACT_COUNT);
                    SpecialPowerTemplate sp;
                    sp.m_class = static_cast<AcademyClassificationType>(a1);
                    stats.recordSpecialPowerUsed(&sp);
                    break;
                }
                case 4: { // recordIncome
                    stats.recordIncome();
                    break;
                }
                case 5: { // recordBuildingCapture + recordGeneralsPointsSpent
                    a1 = 1 + static_cast<Int>(splitmix32(rng) % 5u);
                    stats.recordBuildingCapture();
                    stats.recordGeneralsPointsSpent(a1);
                    break;
                }
                case 6: { // recordBuildingGarrisoned + recordDragSelection
                    stats.recordBuildingGarrisoned();
                    stats.recordDragSelection();
                    break;
                }
                case 7: { // recordStrategyCenter + recordBattlePlanSelected
                    a1 = (seed % 5 == 1 || seed % 5 == 2) ? 1 : static_cast<Int>(splitmix32(rng) & 1u);
                    stats.recordStrategyCenter();
                    if (a1) stats.recordBattlePlanSelected();
                    break;
                }
                case 8: { // recordUnitEnteredTunnelNetwork + recordControlGroupsUsed
                    stats.recordUnitEnteredTunnelNetwork();
                    stats.recordControlGroupsUsed();
                    break;
                }
                case 9: { // recordClearedGarrisonedBuilding + recordSalvageCollected
                    stats.recordClearedGarrisonedBuilding();
                    stats.recordSalvageCollected();
                    break;
                }
                case 10: { // recordGuardAbilityUsed + recordDoubleClickAttackMoveOrderGiven
                    stats.recordGuardAbilityUsed();
                    stats.recordDoubleClickAttackMoveOrderGiven();
                    break;
                }
                case 11: { // recordVehicleDisguised + recordFirestormCreated
                    stats.recordVehicleDisguised();
                    stats.recordFirestormCreated();
                    break;
                }
                case 12: { // recordMineCleared
                    stats.recordMineCleared();
                    break;
                }
                case 13: { // recordMine + recordVehicleSniped on stats
                    stats.recordMine();
                    stats.recordVehicleSniped();
                    break;
                }
            }
            ops[s][0] = op_kind;
            ops[s][1] = a1;
            ops[s][2] = a2;
            ops[s][3] = frame_step;
        }

        // Snapshot XferSave BEFORE calculateAcademyAdvice (since calculateAcademyAdvice mutates idle/power/income max frames!)
        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(&stats);
            xsave.close();
        }
        std::vector<uint8_t> pre_save_bytes = read_file_bytes(tmp_save);

        XferCRC xcrc_pre;
        xcrc_pre.open("pre_crc");
        xcrc_pre.xferSnapshot(&stats);
        xcrc_pre.close();
        UnsignedInt pre_crc = xcrc_pre.getCRC();

        // Run calculateAcademyAdvice
        AcademyAdviceInfo info{};
        Bool has_advice = stats.calculateAcademyAdvice(&info);
        std::string tip0 = (info.numTips > 0) ? advice_to_ascii(info.advice[0]) : "";

        // Snapshot XferSave & XferCRC AFTER calculateAcademyAdvice
        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(&stats);
            xsave.close();
        }
        std::vector<uint8_t> post_save_bytes = read_file_bytes(tmp_save);

        XferCRC xcrc_post;
        xcrc_post.open("post_crc");
        xcrc_post.xferSnapshot(&stats);
        xcrc_post.close();
        UnsignedInt post_crc = xcrc_post.getCRC();

        // Round-trip XferLoad verification
        AcademyStats loaded;
        loaded.init(&plyr);
        {
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(&loaded);
            xload.close();
        }
        XferCRC xcrc_loaded;
        xcrc_loaded.open("loaded_crc");
        xcrc_loaded.xferSnapshot(&loaded);
        xcrc_loaded.close();
        if (xcrc_loaded.getCRC() != post_crc) {
            std::fprintf(stderr, "XferLoad roundtrip mismatch on seed %u\n", seed);
            return 1;
        }

        // Capture client PRNG draw after advice evaluation to lock exact GameClientRandomValue draw count
        Int post_client_rng = GameClientRandomValue(0, 1000000);

        std::fprintf(out,
            "    {\"seed\":%u,\"side_idx\":%d,\"init_obj_mask\":%d,\"init_frame\":%u,\"alt_mouse\":%u,"
            "\"local_mines\":%u,\"neut_sniped\":%u,\"ops\":[",
            seed, side_idx, init_obj_mask, init_frame,
            static_cast<unsigned>(TheGlobalData->m_useAlternateMouse),
            local_mines, neut_sniped);
        for (int s = 0; s < 16; ++s) {
            std::fprintf(out, "[%d,%d,%d,%d]%s",
                ops[s][0], ops[s][1], ops[s][2], ops[s][3], s + 1 < 16 ? "," : "");
        }
        std::fprintf(out,
            "],\"is_first\":%u,\"had_supply\":%u,\"mines\":%u,\"sniped\":%u,"
            "\"has_advice\":%u,\"num_tips\":%u,\"tip0\":\"%s\",\"post_client_rng\":%d,"
            "\"pre_crc\":%u,\"pre_xfer_hex\":\"%s\",\"post_crc\":%u,\"post_xfer_hex\":\"%s\"}%s\n",
            static_cast<unsigned>(stats.isFirstUpdate()),
            static_cast<unsigned>(stats.hadASupplyCenter()),
            stats.getMines(),
            stats.getVehiclesSniped(),
            static_cast<unsigned>(has_advice),
            info.numTips,
            tip0.c_str(),
            post_client_rng,
            pre_crc,
            hex_encode(pre_save_bytes).c_str(),
            post_crc,
            hex_encode(post_save_bytes).c_str(),
            seed + 1 < 1000 ? "," : "");
    }

    std::fprintf(out, "  ]\n}\n");
    std::fclose(out);
    std::remove(tmp_save);
    return 0;
}
