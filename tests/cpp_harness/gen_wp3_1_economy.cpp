// WP-3.1a C++ Golden Vector Generator: Money, Energy, Handicap, MissionStats.
// Compiles unmodified vendor/GeneralsX RTS economy & snapshot reference sources.

#include "PreRTS.h"
#include "Common/crc.h"
#include "Common/Snapshot.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/GameAudio.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Dict.h"
#include "Common/Money.h"
#include "Common/Energy.h"
#include "Common/Handicap.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/Object.h"

#define private public
#include "Common/MissionStats.h"
#undef private

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/NameKeyGenerator.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/Dict.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Money.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Energy.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/Handicap.cpp"
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/RTS/MissionStats.cpp"

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
static AudioManager g_audio;
AudioManager* TheAudio = &g_audio;
static PlayerList g_player_list;
PlayerList* ThePlayerList = &g_player_list;
static GameLogic g_game_logic;
GameLogic* TheGameLogic = &g_game_logic;

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

static Real ffrom_bits(uint32_t b) {
    Real v = 0.0f;
    std::memcpy(&v, &b, 4);
    return v;
}

static uint32_t rot_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
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

static void write_file_bytes(const char* path, const std::vector<uint8_t>& bytes) {
    FILE* fp = std::fopen(path, "wb");
    if (!fp) { std::perror(path); std::exit(1); }
    if (!bytes.empty() && std::fwrite(bytes.data(), 1, bytes.size(), fp) != bytes.size()) {
        std::perror("fwrite");
        std::exit(1);
    }
    std::fclose(fp);
}

static void push_le_u32(std::vector<uint8_t>& out, uint32_t v) {
    out.push_back(static_cast<uint8_t>(v & 0xffu));
    out.push_back(static_cast<uint8_t>((v >> 8) & 0xffu));
    out.push_back(static_cast<uint8_t>((v >> 16) & 0xffu));
    out.push_back(static_cast<uint8_t>((v >> 24) & 0xffu));
}

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "Usage: %s <out.json>\n", argv[0]);
        return 1;
    }

    TheNameKeyGenerator = new NameKeyGenerator();
    TheNameKeyGenerator->init();

    const char* tmp_save = "/tmp/wp3_1_economy_save.bin";
    FILE* fp = std::fopen(argv[1], "wb");
    if (!fp) { std::perror(argv[1]); return 1; }

    std::fprintf(fp, "{\n"
                     "  \"work_package\": \"WP-3.1a\",\n"
                     "  \"modules\": [\"Money\", \"Energy\", \"Handicap\", \"MissionStats\"],\n"
                     "  \"vector_count\": 1000,\n"
                     "  \"vectors\": [\n");

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x74779640u + 0x9e3779b9u;
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            ThePlayerList->m_players[p].m_playerIndex = p;
        }

        // ---------------- 1. Money (PKT-121) ----------------
        g_audio.m_traceCrc = 0;
        g_audio.m_eventCount = 0;
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            ThePlayerList->m_players[p].m_academyStats.m_incomeCalls = 0;
        }

        Int m_pidx = (seed % 7 == 0) ? -1 : static_cast<Int>(splitmix32(rng) % MAX_PLAYER_COUNT);
        UnsignedInt m_start = splitmix32(rng) % 50000u;
        TheGameLogic->m_frame = 1u + (seed % 90u);

        Money money;
        money.setPlayerIndex(m_pidx);
        money.setStartingCash(m_start);

        uint32_t m_ops[12][4];
        uint32_t m_trace_crc = 0;
        for (int s = 0; s < 12; ++s) {
            uint32_t kind = splitmix32(rng) % 4u;
            uint32_t arg1 = 0, arg2 = 0;
            uint32_t frame = TheGameLogic->m_frame + (splitmix32(rng) % 120u);
            TheGameLogic->m_frame = frame;
            if (kind == 0) {
                arg1 = (splitmix32(rng) % 10u == 0) ? 0u : (1u + (splitmix32(rng) % 8000u));
                arg2 = (splitmix32(rng) & 1u) | ((splitmix32(rng) & 1u) << 1);
                money.deposit(arg1, (arg2 & 1u) ? TRUE : FALSE, (arg2 & 2u) ? TRUE : FALSE);
            } else if (kind == 1) {
                arg1 = (splitmix32(rng) % 10u == 0) ? 0u : (1u + (splitmix32(rng) % 12000u));
                arg2 = splitmix32(rng) & 1u;
                UnsignedInt got = money.withdraw(arg1, arg2 ? TRUE : FALSE);
                m_trace_crc = rot_crc(m_trace_crc, got);
            } else if (kind == 2) {
                money.updateIncomeBucket();
            } else {
                arg1 = splitmix32(rng) % 10000u;
                money.setStartingCash(arg1);
            }
            m_ops[s][0] = kind;
            m_ops[s][1] = arg1;
            m_ops[s][2] = arg2;
            m_ops[s][3] = frame;
            m_trace_crc = rot_crc(m_trace_crc, money.countMoney());
            m_trace_crc = rot_crc(m_trace_crc, money.getCashPerMinute());
        }

        Money other_same;
        other_same.setPlayerIndex((m_pidx + 1) % MAX_PLAYER_COUNT);
        other_same.setStartingCash(money.countMoney());
        Bool m_eq_same = money.amountEqual(other_same);
        other_same.deposit(1, FALSE, FALSE);
        Bool m_eq_diff = money.amountEqual(other_same);

        UnsignedInt m_rec_calls = (m_pidx >= 0 && m_pidx < MAX_PLAYER_COUNT)
            ? ThePlayerList->m_players[m_pidx].m_academyStats.m_incomeCalls : 0u;

        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(&money);
            xsave.close();
        }
        std::vector<uint8_t> m_save_bytes = read_file_bytes(tmp_save);

        Money m_loaded;
        m_loaded.setPlayerIndex(m_pidx);
        {
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(&m_loaded);
            xload.close();
        }

        // Legacy Money v1 XferLoad check
        UnsignedInt v1_cash = splitmix32(rng) % 100000u;
        {
            std::vector<uint8_t> v1_buf;
            v1_buf.push_back(1);
            push_le_u32(v1_buf, v1_cash);
            write_file_bytes(tmp_save, v1_buf);
        }
        Money m_v1;
        m_v1.deposit(999, FALSE, TRUE);
        {
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(&m_v1);
            xload.close();
        }

        // ---------------- 2. Energy (PKT-118) ----------------
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            ThePlayerList->m_players[p].m_brownoutCallbacks = 0;
            ThePlayerList->m_players[p].m_brownoutTraceCrc = 0;
        }
        Int e_owner = static_cast<Int>(splitmix32(rng) % MAX_PLAYER_COUNT);
        Energy energy;
        energy.init(&ThePlayerList->m_players[e_owner]);
        TheGameLogic->m_frame = 10u + (seed % 50u);

        Int e_ops[12][5];
        uint32_t e_trace_crc = 0;
        for (int s = 0; s < 12; ++s) {
            Int kind = static_cast<Int>(splitmix32(rng) % 7u);
            Int a1 = 0, a2 = 0, a3 = 0;
            TheGameLogic->m_frame += (splitmix32(rng) % 25u);
            if (kind == 0) {
                a1 = static_cast<Int>(splitmix32(rng) % 41u) - 20;
                a2 = static_cast<Int>(splitmix32(rng) & 1u);
                energy.adjustPower(a1, a2 ? TRUE : FALSE);
            } else if (kind == 1 || kind == 2) {
                a1 = static_cast<Int>(splitmix32(rng) % 31u) - 15; // energyProduction
                a2 = static_cast<Int>(splitmix32(rng) % 10u);      // energyBonus
                a3 = static_cast<Int>((splitmix32(rng) % 5u) == 0); // null or non-null
                if (a3) {
                    if (kind == 1) energy.objectEnteringInfluence(nullptr);
                    else energy.objectLeavingInfluence(nullptr);
                } else {
                    Object obj(TRUE, a1, a2, FALSE);
                    if (kind == 1) energy.objectEnteringInfluence(&obj);
                    else energy.objectLeavingInfluence(&obj);
                }
            } else if (kind == 3 || kind == 4) {
                a1 = static_cast<Int>(splitmix32(rng) % 15u);
                a2 = static_cast<Int>(splitmix32(rng) & 1u); // disabled
                a3 = static_cast<Int>((splitmix32(rng) % 5u) == 0); // null flag
                if (a3) {
                    if (kind == 3) energy.addPowerBonus(nullptr);
                    else energy.removePowerBonus(nullptr);
                } else {
                    Object obj(TRUE, 10, a1, (kind == 4 && a2) ? TRUE : FALSE);
                    if (kind == 3) energy.addPowerBonus(&obj);
                    else energy.removePowerBonus(&obj);
                }
            } else if (kind == 5) {
                a1 = static_cast<Int>(TheGameLogic->m_frame) + static_cast<Int>(splitmix32(rng) % 30u) - 5;
                if (a1 < 0) a1 = 0;
                energy.setPowerSabotagedTillFrame(static_cast<UnsignedInt>(a1));
            }
            e_ops[s][0] = kind;
            e_ops[s][1] = a1;
            e_ops[s][2] = a2;
            e_ops[s][3] = a3;
            e_ops[s][4] = static_cast<Int>(TheGameLogic->m_frame);
            e_trace_crc = rot_crc(e_trace_crc, static_cast<uint32_t>(energy.getProduction()));
            e_trace_crc = rot_crc(e_trace_crc, static_cast<uint32_t>(energy.getConsumption()));
            e_trace_crc = rot_crc(e_trace_crc, static_cast<uint32_t>(energy.hasSufficientPower()));
            e_trace_crc = rot_crc(e_trace_crc, fbits(energy.getEnergySupplyRatio()));
            e_trace_crc = rot_crc(e_trace_crc, ThePlayerList->m_players[e_owner].m_brownoutCallbacks);
            e_trace_crc = rot_crc(e_trace_crc, ThePlayerList->m_players[e_owner].m_brownoutTraceCrc);
        }
        // Keep exact production/consumption non-negative before Xfer check too
        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(&energy);
            xsave.close();
        }
        std::vector<uint8_t> e_save_bytes = read_file_bytes(tmp_save);

        // Energy legacy v1 load test
        Int e_v1_prod = static_cast<Int>(splitmix32(rng) % 200u);
        Int e_v1_cons = static_cast<Int>(splitmix32(rng) % 200u);
        Int e_v1_owner = static_cast<Int>(splitmix32(rng) % MAX_PLAYER_COUNT);
        {
            std::vector<uint8_t> v1_buf;
            v1_buf.push_back(1);
            push_le_u32(v1_buf, static_cast<uint32_t>(e_v1_prod));
            push_le_u32(v1_buf, static_cast<uint32_t>(e_v1_cons));
            push_le_u32(v1_buf, static_cast<uint32_t>(e_v1_owner));
            write_file_bytes(tmp_save, v1_buf);
        }
        Energy e_v1;
        e_v1.setPowerSabotagedTillFrame(777);
        {
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(&e_v1);
            xload.close();
        }

        // ---------------- 3. Handicap (PKT-119) ----------------
        static const char* kHandicapNames[4] = {
            "HANDICAP_BUILDCOST_GENERIC",
            "HANDICAP_BUILDCOST_BUILDINGS",
            "HANDICAP_BUILDTIME_GENERIC",
            "HANDICAP_BUILDTIME_BUILDINGS"
        };
        uint32_t h_d1_mask = splitmix32(rng) & 0xfu;
        uint32_t h_d2_mask = splitmix32(rng) & 0xfu;
        uint32_t h_d1_vals[4] = {0, 0, 0, 0};
        uint32_t h_d2_vals[4] = {0, 0, 0, 0};
        Dict d1, d2;
        for (int i = 0; i < 4; ++i) {
            NameKeyType k = TheNameKeyGenerator->nameToKey(kHandicapNames[i]);
            if (h_d1_mask & (1u << i)) {
                Real v = 0.25f + static_cast<Real>(splitmix32(rng) % 200u) * 0.01f;
                h_d1_vals[i] = fbits(v);
                d1.setReal(k, ffrom_bits(h_d1_vals[i]));
            }
            if (h_d2_mask & (1u << i)) {
                Real v = 0.50f + static_cast<Real>(splitmix32(rng) % 150u) * 0.01f;
                h_d2_vals[i] = fbits(v);
                d2.setReal(k, ffrom_bits(h_d2_vals[i]));
            }
        }
        // Add decoy int / wrong type on one key when not present
        d1.setInt(TheNameKeyGenerator->nameToKey("HANDICAP_DECOY"), 42);
        Handicap hcap;
        hcap.readFromDict(&d1);
        hcap.readFromDict(&d2);
        ThingTemplate tmpl_gen(FALSE, 0, 0);
        ThingTemplate tmpl_bld(TRUE, 0, 0);
        uint32_t h_out[4] = {
            fbits(hcap.getHandicap(Handicap::BUILDCOST, &tmpl_gen)),
            fbits(hcap.getHandicap(Handicap::BUILDCOST, &tmpl_bld)),
            fbits(hcap.getHandicap(Handicap::BUILDTIME, &tmpl_gen)),
            fbits(hcap.getHandicap(Handicap::BUILDTIME, &tmpl_bld)),
        };

        // ---------------- 4. MissionStats (PKT-120) ----------------
        MissionStats mstats;
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            mstats.m_unitsKilled[p] = static_cast<Int>(splitmix32(rng) % 1000u);
            mstats.m_buildingsKilled[p] = static_cast<Int>(splitmix32(rng) % 500u);
        }
        mstats.m_unitsLost = static_cast<Int>(splitmix32(rng) % 1000u);
        mstats.m_buildingsLost = static_cast<Int>(splitmix32(rng) % 500u);

        XferCRC ms_xcrc;
        ms_xcrc.open("ms_crc");
        ms_xcrc.xferSnapshot(&mstats);
        UnsignedInt ms_crc_val = ms_xcrc.getCRC();

        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(&mstats);
            xsave.close();
        }
        std::vector<uint8_t> ms_save_bytes = read_file_bytes(tmp_save);

        // Emit one compact JSON object per vector line
        std::fprintf(fp,
            "    {\"seed\":%u,\"init_frame\":%u,\"m_pidx\":%d,\"m_start\":%u,\"m_ops\":[",
            seed, 1u + (seed % 90u), m_pidx, m_start);
        for (int s = 0; s < 12; ++s) {
            std::fprintf(fp, "%u,%u,%u,%u%s", m_ops[s][0], m_ops[s][1], m_ops[s][2], m_ops[s][3],
                         (s + 1 < 12) ? "," : "");
        }
        std::fprintf(fp,
            "],\"m_final\":%u,\"m_cpm\":%u,\"m_cbucket\":%u,\"m_rec\":%u,\"m_acrc\":%u,"
            "\"m_tcrc\":%u,\"m_eq_s\":%u,\"m_eq_d\":%u,\"m_v1_cash\":%u,\"m_v1_loaded\":%u,\"m_v1_cpm\":%u,"
            "\"m_save_hex\":\"",
            m_loaded.countMoney(), m_loaded.getCashPerMinute(), money.countMoney(),
            m_rec_calls, g_audio.m_traceCrc, m_trace_crc, m_eq_same, m_eq_diff,
            v1_cash, m_v1.countMoney(), m_v1.getCashPerMinute());
        for (uint8_t b : m_save_bytes) std::fprintf(fp, "%02x", b);

        std::fprintf(fp, "\",\"e_owner\":%d,\"e_end_frame\":%u,\"e_ops\":[",
                     e_owner, TheGameLogic->m_frame);
        for (int s = 0; s < 12; ++s) {
            std::fprintf(fp, "%d,%d,%d,%d,%d%s",
                         e_ops[s][0], e_ops[s][1], e_ops[s][2], e_ops[s][3], e_ops[s][4],
                         (s + 1 < 12) ? "," : "");
        }
        std::fprintf(fp,
            "],\"e_tcrc\":%u,\"e_prod\":%d,\"e_cons\":%d,\"e_sab\":%u,\"e_ratio\":%u,"
            "\"e_v1_in\":[%d,%d,%d],\"e_v1_out\":[%d,%d,%u],\"e_save_hex\":\"",
            e_trace_crc, energy.getProduction(), energy.getConsumption(),
            energy.getPowerSabotagedTillFrame(), fbits(energy.getEnergySupplyRatio()),
            e_v1_prod, e_v1_cons, e_v1_owner,
            e_v1.getProduction(), e_v1.getConsumption(), e_v1.getPowerSabotagedTillFrame());
        for (uint8_t b : e_save_bytes) std::fprintf(fp, "%02x", b);

        std::fprintf(fp,
            "\",\"h_m1\":%u,\"h_m2\":%u,\"h_d1\":[%u,%u,%u,%u],\"h_d2\":[%u,%u,%u,%u],\"h_out\":[%u,%u,%u,%u],"
            "\"ms_crc\":%u,\"ms_ul\":%d,\"ms_bl\":%d,\"ms_uk\":[",
            h_d1_mask, h_d2_mask,
            h_d1_vals[0], h_d1_vals[1], h_d1_vals[2], h_d1_vals[3],
            h_d2_vals[0], h_d2_vals[1], h_d2_vals[2], h_d2_vals[3],
            h_out[0], h_out[1], h_out[2], h_out[3],
            ms_crc_val, mstats.m_unitsLost, mstats.m_buildingsLost);
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            std::fprintf(fp, "%d%s", mstats.m_unitsKilled[p], (p + 1 < MAX_PLAYER_COUNT) ? "," : "");
        }
        std::fprintf(fp, "],\"ms_bk\":[");
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            std::fprintf(fp, "%d%s", mstats.m_buildingsKilled[p], (p + 1 < MAX_PLAYER_COUNT) ? "," : "");
        }
        std::fprintf(fp, "],\"ms_save_hex\":\"");
        for (uint8_t b : ms_save_bytes) std::fprintf(fp, "%02x", b);
        std::fprintf(fp, "\"}%s\n", (seed + 1u < 1000u) ? "," : "");
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::remove(tmp_save);
    delete TheNameKeyGenerator;
    TheNameKeyGenerator = nullptr;
    std::fprintf(stderr, "Generated 1000 WP-3.1a vectors to %s\n", argv[1]);
    return 0;
}
