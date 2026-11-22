// PKT-406 (Radar, RadarObject) (WP-6.1) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against pre-built vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & run command:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt406_radar.cpp -o /tmp/gen_pkt406
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pkt406 docs/golden_vectors/PKT-406.json
//
// Determinism & UB verification builds (must emit byte-identical JSON):
//   --opt O0, --opt O1, --opt O2, and --sanitize (UBSan).

#include <cstdint>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <string>
#include <vector>
#include <unistd.h>

#include "Common/CriticalSection.h"
#include "Common/GameMemory.h"
#include "Common/GlobalData.h"
#include "Common/NameKeyGenerator.h"
#include "Common/Player.h"
#include "Common/PlayerList.h"
#include "Common/Radar.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/crc.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/RankInfo.h"

int __argc = 1;
char* __argv_buf[] = { (char*)"gen_pkt406_radar", nullptr };
char** __argv = __argv_buf;
HWND ApplicationHWnd = nullptr;
struct SDL_Window;
SDL_Window* TheSDL3Window = nullptr;
const Char* g_csfFile = "data/%s/generals.csf";
const Char* g_strFile = "data/Generals.str";
void OSDisplaySetBusyState(bool, bool) {}
void OSDisplayWarningBox(AsciiString, AsciiString, unsigned int, unsigned int) {}

template<typename Tag, typename Tag::type M>
struct MemberAccess {
    friend typename Tag::type access_member(Tag) { return M; }
};
struct FrameTag { using type = UnsignedInt GameLogic::*; };
template struct MemberAccess<FrameTag, &GameLogic::m_frame>;
UnsignedInt GameLogic::* access_member(FrameTag);

struct ColorTag { using type = Color Player::*; };
template struct MemberAccess<ColorTag, &Player::m_color>;
Color Player::* access_member(ColorTag);

class TestRadar : public RadarDummy {
public:
    void configureExtent(const Region3D& ext) {
        m_mapExtent = ext;
        m_xSample = ext.width() / static_cast<Real>(RADAR_CELL_WIDTH);
        m_ySample = ext.height() / static_cast<Real>(RADAR_CELL_HEIGHT);
    }
    Real xSample() const { return m_xSample; }
    Real ySample() const { return m_ySample; }
    UnsignedInt queueFrame() const { return m_queueTerrainRefreshFrame; }
    Int nextFree() const { return m_nextFreeRadarEvent; }
    Int lastEventIdx() const { return m_lastRadarEvent; }
    const RadarEvent& eventAt(Int i) const { return m_event[i]; }
    Int activeCount() const {
        Int n = 0;
        for (Int i = 0; i < MAX_RADAR_EVENTS; ++i) if (m_event[i].active) ++n;
        return n;
    }
    void runXfer(Xfer* x) { xfer(x); }
};

class MemoryXfer : public Xfer {
public:
    std::vector<uint8_t> bytes;
    size_t cursor = 0;
    explicit MemoryXfer(XferMode mode) { m_xferMode = mode; }
    void open(AsciiString id) override { m_identifier = id; cursor = 0; if (m_xferMode == XFER_SAVE) bytes.clear(); }
    void close() override {}
    Int beginBlock() override { return 0; }
    void endBlock() override {}
    void skip(Int n) override { cursor += static_cast<size_t>(n); }
    void xferSnapshot(Snapshot* s) override;
protected:
    void xferImplementation(void* data, Int dataSize) override {
        if (dataSize <= 0) return;
        size_t n = static_cast<size_t>(dataSize);
        if (m_xferMode == XFER_SAVE) {
            const uint8_t* p = static_cast<const uint8_t*>(data);
            bytes.insert(bytes.end(), p, p + n);
        } else if (m_xferMode == XFER_LOAD) {
            if (cursor + n > bytes.size()) { std::fprintf(stderr, "MemoryXfer EOF\n"); std::exit(2); }
            std::memcpy(data, bytes.data() + cursor, n);
            cursor += n;
        }
    }
};

void MemoryXfer::xferSnapshot(Snapshot* s) {
    if (auto* tr = dynamic_cast<TestRadar*>(s)) tr->runXfer(this);
}

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
    std::vector<uint8_t> d;
    std::fseek(f, 0, SEEK_END);
    long sz = std::ftell(f);
    std::fseek(f, 0, SEEK_SET);
    if (sz > 0) { d.resize(static_cast<size_t>(sz)); if (std::fread(d.data(), 1, static_cast<size_t>(sz), f) != static_cast<size_t>(sz)) std::exit(1); }
    std::fclose(f);
    uint64_t bl = static_cast<uint64_t>(d.size()) * 8ULL;
    d.push_back(0x80u);
    while ((d.size() % 64) != 56) d.push_back(0u);
    for (int i = 7; i >= 0; --i) d.push_back(static_cast<uint8_t>((bl >> (i * 8)) & 0xffu));
    uint32_t H[8] = {0x6a09e667u,0xbb67ae85u,0x3c6ef372u,0xa54ff53au,0x510e527fu,0x9b05688cu,0x1f83d9abu,0x5be0cd19u};
    auto r = [](uint32_t x, uint32_t n) { return (x >> n) | (x << (32 - n)); };
    for (size_t off = 0; off < d.size(); off += 64) {
        uint32_t W[64];
        for (int i = 0; i < 16; ++i) W[i] = (uint32_t(d[off+i*4])<<24)|(uint32_t(d[off+i*4+1])<<16)|(uint32_t(d[off+i*4+2])<<8)|uint32_t(d[off+i*4+3]);
        for (int i = 16; i < 64; ++i) W[i] = W[i-16] + (r(W[i-15],7)^r(W[i-15],18)^(W[i-15]>>3)) + W[i-7] + (r(W[i-2],17)^r(W[i-2],19)^(W[i-2]>>10));
        uint32_t a=H[0],b=H[1],c=H[2],d0=H[3],e=H[4],f2=H[5],g=H[6],h=H[7];
        for (int i = 0; i < 64; ++i) {
            uint32_t t1 = h + (r(e,6)^r(e,11)^r(e,25)) + ((e&f2)^((~e)&g)) + K[i] + W[i];
            uint32_t t2 = (r(a,2)^r(a,13)^r(a,22)) + ((a&b)^(a&c)^(b&c));
            h=g; g=f2; f2=e; e=d0+t1; d0=c; c=b; b=a; a=t1+t2;
        }
        H[0]+=a;H[1]+=b;H[2]+=c;H[3]+=d0;H[4]+=e;H[5]+=f2;H[6]+=g;H[7]+=h;
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
static uint32_t fbits(Real v) { uint32_t b = 0; std::memcpy(&b, &v, 4); return b; }
static Real from_bits(uint32_t b) { Real v = 0; std::memcpy(&v, &b, 4); return v; }

int main(int argc, char** argv) {
    const char* out = (argc > 1) ? argv[1] : "docs/golden_vectors/PKT-406.json";
    CriticalSection cs1, cs2, cs3, cs4;
    TheAsciiStringCriticalSection = &cs1;
    TheUnicodeStringCriticalSection = &cs2;
    TheDmaCriticalSection = &cs3;
    TheMemoryPoolCriticalSection = &cs4;
    initMemoryManager();
    TheWritableGlobalData = NEW GlobalData;
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();
    TheRankInfoStore = NEW RankInfoStore;
    TheRankInfoStore->init();
    TheGameLogic = NEW GameLogic;
    ThePlayerList = NEW PlayerList;

    const char* h_path = "vendor/GeneralsX/Core/GameEngine/Include/Common/Radar.h";
    const char* cpp_path = "vendor/GeneralsX/Core/GameEngine/Source/Common/System/Radar.cpp";
    std::string sha_h = sha256_file(h_path), sha_cpp = sha256_file(cpp_path);

    std::string tmp = std::string(out) + ".tmp";
    FILE* fp = std::fopen(tmp.c_str(), "wb");
    if (!fp) { std::perror(tmp.c_str()); return 1; }
    std::fprintf(fp,
        "{\n  \"packet\": \"PKT-406\",\n  \"module\": \"Radar\",\n  \"vector_count\": 1000,\n"
        "  \"source_sha256\": {\n    \"%s\": \"%s\",\n    \"%s\": \"%s\"\n  },\n  \"vectors\": [\n",
        h_path, sha_h.c_str(), cpp_path, sha_cpp.c_str());

    UnsignedInt prio_vis = 0;
    for (int p = -1; p <= 6; ++p)
        if (Radar::isPriorityVisible(static_cast<RadarPriorityType>(p))) prio_vis |= (1u << (p + 1));

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x4060beefu;
        if (rng == 0) rng = 0x12345678u;

        Real lo_x = (seed % 5 == 0) ? 0.0f : static_cast<Real>(splitmix32(rng) % 200);
        Real lo_y = (seed % 5 == 0) ? 0.0f : static_cast<Real>(splitmix32(rng) % 200);
        Real hi_x = lo_x + 500.0f + static_cast<Real>(splitmix32(rng) % 6000);
        Real hi_y = lo_y + 500.0f + static_cast<Real>(splitmix32(rng) % 6000);
        Region3D ext; ext.lo = {lo_x, lo_y, 0.0f}; ext.hi = {hi_x, hi_y, 100.0f};

        Real wx = static_cast<Real>(static_cast<int32_t>(splitmix32(rng) % 9000) - 1000) + ((splitmix32(rng) % 16) * 0.125f);
        Real wy = static_cast<Real>(static_cast<int32_t>(splitmix32(rng) % 9000) - 1000) + ((splitmix32(rng) % 16) * 0.125f);
        Int rx = static_cast<Int>(splitmix32(rng) % 180) - 20;
        Int ry = static_cast<Int>(splitmix32(rng) % 180) - 20;
        Int sx = static_cast<Int>(splitmix32(rng) % 50), sy = static_cast<Int>(splitmix32(rng) % 50);
        Int dw = 64 + static_cast<Int>(splitmix32(rng) % 400), dh = 64 + static_cast<Int>(splitmix32(rng) % 400);

        Int local_p = static_cast<Int>(seed % MAX_PLAYER_COUNT);
        Color p_col = static_cast<Color>(0xff000000u | (splitmix32(rng) & 0x00ffffffu));
        uint16_t hide_m = static_cast<uint16_t>(splitmix32(rng) & 0xffffu);
        uint16_t force_m = static_cast<uint16_t>(splitmix32(rng) & 0xffffu);

        ThePlayerList->setLocalPlayer(ThePlayerList->getNthPlayer(local_p));
        ThePlayerList->getLocalPlayer()->*access_member(ColorTag()) = p_col;

        TestRadar radar;
        radar.configureExtent(ext);
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            radar.hide(p, ((hide_m >> p) & 1u) != 0);
            radar.forceOn(p, ((force_m >> p) & 1u) != 0);
        }

        Coord3D w_in{wx, wy, 0.0f};
        ICoord2D w2r_out{}; radar.worldToRadar(&w_in, &w2r_out);
        ICoord2D r_in{rx, ry};
        Coord3D r2w_out{0.0f, 0.0f, 0.0f}; radar.radarToWorld2D(&r_in, &r2w_out);
        ICoord2D ul{}, lr{}; radar.findDrawPositions(sx, sy, dw, dh, &ul, &lr);

        // Flat op tuples [kind, a0, a1, a2, a3]:
        //  0: advance frame=a0 & update(), 1: createEvent(type=a0, wx=a1, wy=a2, ttl=a3),
        //  2: createPlayerEvent(type=a0, wx=a1, wy=a2, ttl=a3), 3: tryEvent(type=a0, wx=a1, wy=a2),
        //  4: queueTerrainRefresh(), 5: reset()
        std::vector<int32_t> ops;
        std::vector<int32_t> try_rets;
        auto push_op = [&](int32_t k, int32_t a0, int32_t a1, int32_t a2, int32_t a3) {
            ops.push_back(k); ops.push_back(a0); ops.push_back(a1); ops.push_back(a2); ops.push_back(a3);
        };
        auto exec_op = [&](int32_t k, int32_t a0, int32_t a1, int32_t a2, int32_t a3) {
            push_op(k, a0, a1, a2, a3);
            if (k == 0) {
                TheGameLogic->*access_member(FrameTag()) = static_cast<UnsignedInt>(a0);
                radar.update();
            } else if (k == 1 || k == 2) {
                Coord3D pos{from_bits(static_cast<uint32_t>(a1)), from_bits(static_cast<uint32_t>(a2)), 0.0f};
                Real ttl = from_bits(static_cast<uint32_t>(a3));
                if (k == 1) radar.createEvent(&pos, static_cast<RadarEventType>(a0), ttl);
                else radar.createPlayerEvent(ThePlayerList->getLocalPlayer(), &pos, static_cast<RadarEventType>(a0), ttl);
            } else if (k == 3) {
                Coord3D pos{from_bits(static_cast<uint32_t>(a1)), from_bits(static_cast<uint32_t>(a2)), 0.0f};
                Bool ok = radar.tryEvent(static_cast<RadarEventType>(a0), &pos);
                try_rets.push_back(ok ? 1 : 0);
            } else if (k == 4) {
                radar.queueTerrainRefresh();
            } else if (k == 5) {
                radar.reset();
            }
        };

        UnsignedInt frame = (seed == 0) ? 100u : (splitmix32(rng) % 500u);
        TheGameLogic->*access_member(FrameTag()) = frame;
        if (seed == 0) {
            // Deterministic edge & wrap-around suite on seed 0
            exec_op(2, RADAR_EVENT_CONSTRUCTION, fbits(250.0f), fbits(350.0f), fbits(4.0f));
            exec_op(1, RADAR_EVENT_BEACON_PULSE, fbits(50.0f), fbits(75.0f), fbits(2.0f));
            exec_op(3, RADAR_EVENT_UNDER_ATTACK, fbits(100.0f), fbits(100.0f), 0);
            exec_op(3, RADAR_EVENT_UNDER_ATTACK, fbits(2000.0f), fbits(2000.0f), 0); // suppressed map-wide
            exec_op(0, static_cast<int32_t>(frame + 299u), 0, 0, 0);
            exec_op(3, RADAR_EVENT_UNDER_ATTACK, fbits(2000.0f), fbits(2000.0f), 0); // still < 300
            exec_op(0, static_cast<int32_t>(frame + 300u), 0, 0, 0);
            exec_op(3, RADAR_EVENT_UNDER_ATTACK, fbits(2000.0f), fbits(2000.0f), 0); // accepted at == 300
            exec_op(4, 0, 0, 0, 0);
            exec_op(0, static_cast<int32_t>(frame + 390u), 0, 0, 0); // diff == 90 -> not yet
            exec_op(0, static_cast<int32_t>(frame + 391u), 0, 0, 0); // diff == 91 -> refreshed
            for (int n = 0; n < 66; ++n) {
                RadarEventType et = static_cast<RadarEventType>(1 + (n % 8));
                if (et == RADAR_EVENT_BEACON_PULSE) et = RADAR_EVENT_INFORMATION;
                exec_op(1, et, fbits(10.0f * n), fbits(15.0f * n), fbits(1.5f));
            }
        } else {
            int nops = 6 + static_cast<int>(splitmix32(rng) % 10);
            for (int n = 0; n < nops; ++n) {
                uint32_t kind_roll = splitmix32(rng) % 10u;
                if (kind_roll < 2u) {
                    frame += 1u + (splitmix32(rng) % 180u);
                    exec_op(0, static_cast<int32_t>(frame), 0, 0, 0);
                } else if (kind_roll < 5u) {
                    RadarEventType et = static_cast<RadarEventType>(1 + (splitmix32(rng) % (RADAR_EVENT_NUM_EVENTS - 1)));
                    Real px = static_cast<Real>(splitmix32(rng) % 5000);
                    Real py = static_cast<Real>(splitmix32(rng) % 5000);
                    Real ttl = 1.0f + static_cast<Real>(splitmix32(rng) % 8);
                    exec_op((splitmix32(rng) & 1u) ? 1 : 2, et, fbits(px), fbits(py), fbits(ttl));
                } else if (kind_roll < 8u) {
                    RadarEventType et = static_cast<RadarEventType>(splitmix32(rng) % (RADAR_EVENT_NUM_EVENTS + 1));
                    Real px = static_cast<Real>(splitmix32(rng) % 1200);
                    Real py = static_cast<Real>(splitmix32(rng) % 1200);
                    exec_op(3, et, fbits(px), fbits(py), 0);
                } else if (kind_roll == 8u) {
                    exec_op(4, 0, 0, 0, 0);
                } else if ((seed % 11u) == 0u) {
                    exec_op(5, 0, 0, 0, 0);
                }
            }
        }

        Coord3D last_loc{0.0f, 0.0f, 0.0f};
        Bool last_ok = radar.getLastEventLoc(&last_loc);
        uint16_t hide_out = 0, force_out = 0;
        for (Int p = 0; p < MAX_PLAYER_COUNT; ++p) {
            if (radar.isRadarHidden(p)) hide_out |= (1u << p);
            if (radar.isRadarForced(p)) force_out |= (1u << p);
        }

        MemoryXfer xsave(XFER_SAVE); xsave.open("m"); radar.runXfer(&xsave); xsave.close();
        XferCRC xcrc; xcrc.open("c"); radar.runXfer(&xcrc); xcrc.close();

        // Self-verify round-trip save->load->re-save inside C++
        TestRadar loaded;
        MemoryXfer xload(XFER_LOAD); xload.bytes = xsave.bytes; xload.open("l"); loaded.runXfer(&xload); xload.close();
        MemoryXfer xresave(XFER_SAVE); xresave.open("r"); loaded.runXfer(&xresave); xresave.close();
        if (xsave.bytes != xresave.bytes) { std::fprintf(stderr, "Round-trip xfer mismatch seed %u\n", seed); return 3; }

        CRC bcrc; bcrc.computeCRC(xsave.bytes.data(), static_cast<Int>(xsave.bytes.size()));

        std::fprintf(fp,
            "    {\"seed\":%u,\"ext\":[%u,%u,%u,%u],\"wq\":[%u,%u],\"rq\":[%d,%d],\"dr\":[%d,%d,%d,%d],"
            "\"lp\":%d,\"pcol\":%d,\"hm\":%u,\"fm\":%u,\"ops\":[",
            seed, fbits(lo_x), fbits(lo_y), fbits(hi_x), fbits(hi_y), fbits(wx), fbits(wy),
            rx, ry, sx, sy, dw, dh, local_p, static_cast<int>(p_col), hide_m, force_m);
        for (size_t i = 0; i < ops.size(); ++i) std::fprintf(fp, "%s%d", i ? "," : "", ops[i]);
        std::fprintf(fp, "],\"w2r\":[%d,%d],\"r2w\":[%u,%u],\"ul_lr\":[%d,%d,%d,%d],\"pvis\":%u,\"try_rets\":[",
            w2r_out.x, w2r_out.y, fbits(r2w_out.x), fbits(r2w_out.y), ul.x, ul.y, lr.x, lr.y, prio_vis);
        for (size_t i = 0; i < try_rets.size(); ++i) std::fprintf(fp, "%s%d", i ? "," : "", try_rets[i]);
        std::fprintf(fp,
            "],\"last_ok\":%d,\"last_loc\":[%u,%u,%u],\"next_free\":%d,\"last_idx\":%d,\"act\":%d,"
            "\"qframe\":%u,\"h_out\":%u,\"f_out\":%u,\"xlen\":%zu,\"xbcrc\":%u,\"xcrc\":%u",
            last_ok ? 1 : 0, fbits(last_loc.x), fbits(last_loc.y), fbits(last_loc.z),
            radar.nextFree(), radar.lastEventIdx(), radar.activeCount(), radar.queueFrame(),
            hide_out, force_out, xsave.bytes.size(), bcrc.get(), xcrc.getCRC());
        if (seed == 0) {
            const auto& ev = radar.eventAt(0);
            std::fprintf(fp,
                ",\"s0_samp\":[%u,%u],\"s0_ev0\":[%d,%d,%u,%u,%u,%d,%d,%d,%d,%d,%d],\"s0_xhex\":\"",
                fbits(radar.xSample()), fbits(radar.ySample()), static_cast<int>(ev.type), ev.active ? 1 : 0,
                ev.createFrame, ev.dieFrame, ev.fadeFrame, ev.color1.red, ev.color1.green, ev.color1.blue,
                ev.color1.alpha, ev.radarLoc.x, ev.radarLoc.y);
            for (uint8_t b : xsave.bytes) std::fprintf(fp, "%02x", b);
            std::fprintf(fp, "\"");
        }
        std::fprintf(fp, "}%s\n", (seed + 1u < 1000u) ? "," : "");
    }
    std::fprintf(fp, "  ]\n}\n");
    if (std::ferror(fp) || std::fclose(fp) != 0) { std::perror("write/close"); std::remove(tmp.c_str()); return 1; }
    if (std::rename(tmp.c_str(), out) != 0) { std::perror("rename"); std::remove(tmp.c_str()); return 1; }
    std::fflush(stdout);
    shutdownMemoryManager();
    _exit(0);
}
