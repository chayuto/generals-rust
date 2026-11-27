// SPDX-License-Identifier: GPL-3.0-or-later
// Golden vectors for the replay file and its game options string (PKT-461), run on the
// unmodified reference `RecorderClass`, `ParseAsciiStringToGameInfo` and
// `GameInfoToAsciiString`. Every input is synthetic.
//
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt461_recorder.cpp -o /tmp/gen
//   /tmp/gen docs/golden_vectors/PKT-461.json
//   /tmp/gen --local <replay directory> docs/golden_vectors/PKT-461_local.json
//
// The second form needs local replays and records only counts and digests of them.
//
// Recording writes `00000000.rep` into the user data directory, so the harness first
// moves that directory into a fresh temporary one and refuses to run otherwise.

#include "oracle_wp51_world.h"

#include "Common/GameEngine.h"
#include "Common/GlobalData.h"
#include "Common/MessageStream.h"
#include "Common/Recorder.h"
#include "Common/version.h"
#include "GameClient/MapUtil.h"
#include "GameNetwork/GameInfo.h"
#include "GameLogic/GameLogic.h"
#include "Common/GameState.h"
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/StdDevice/Common/StdLocalFileSystem.h"
#include "Common/FileSystem.h"

#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <dirent.h>
#include <map>
#include <string>
#include <unistd.h>
#include <vector>

struct UserDataDirTag { using type = AsciiString GlobalData::*; };
template struct MemberAccess<UserDataDirTag, &GlobalData::m_userDataDir>;
inline AsciiString GlobalData::* access_member(UserDataDirTag);

// `stopPlayback` leaves the game, which tells the engine object to stop being busy.
struct IdleEngine : public GameEngine {
    LocalFileSystem* createLocalFileSystem() override { return nullptr; }
    ArchiveFileSystem* createArchiveFileSystem() override { return nullptr; }
    GameLogic* createGameLogic() override { return nullptr; }
    GameClient* createGameClient() override { return nullptr; }
    ModuleFactory* createModuleFactory() override { return nullptr; }
    ThingFactory* createThingFactory() override { return nullptr; }
    FunctionLexicon* createFunctionLexicon() override { return nullptr; }
    Radar* createRadar(Bool) override { return nullptr; }
    WebBrowser* createWebBrowser() override { return nullptr; }
    ParticleSystemManager* createParticleSystemManager(Bool) override { return nullptr; }
    AudioManager* createAudioManager(Bool) override { return nullptr; }
};

static std::string g_tmp;
static uint32_t g_state = 0x46100461u;
static uint32_t pick(uint32_t n) {
    g_state ^= g_state << 13;
    g_state ^= g_state >> 17;
    g_state ^= g_state << 5;
    return g_state % n;
}

static void die(const char* what) {
    std::fprintf(stderr, "FATAL: %s\n", what);
    std::exit(1);
}

static std::string hex(const void* data, size_t n) {
    static const char* d = "0123456789abcdef";
    const unsigned char* p = static_cast<const unsigned char*>(data);
    std::string out;
    for (size_t i = 0; i < n; ++i) {
        out += d[p[i] >> 4];
        out += d[p[i] & 15];
    }
    return out;
}
static std::string hex(const std::string& s) { return hex(s.data(), s.size()); }

static std::string num(long long v) { return std::to_string(v); }
static std::string unum(unsigned long long v) { return std::to_string(v); }

static std::string wide(const UnicodeString& s) {
    std::string out;
    for (const WideChar* c = s.str(); *c; ++c) {
        if (!out.empty()) out += '.';
        out += unum(static_cast<uint32_t>(*c));
    }
    return out;
}

static std::string info_dump(GameInfo& g) {
    const AsciiString portable = TheGameState->realMapPathToPortableMapPath(g.getMap());
    std::string out = num(g.getUseStats()) + " " + num(g.getMapContentsMask()) + " " +
                      hex(portable.str()) + " " + unum(g.getMapCRC()) + " " +
                      unum(g.getMapSize()) + " " + num(g.getSeed()) + " " +
                      num(g.getCRCInterval()) + " " + unum(g.getSuperweaponRestriction()) + " " +
                      unum(g.getStartingCash().countMoney()) + " " +
                      num(g.oldFactionsOnly() ? 1 : 0) + ";";
    for (int i = 0; i < MAX_SLOTS; ++i) {
        const GameSlot* s = g.getConstSlot(i);
        out += num(s->getState()) + "," + num(s->isAccepted()) + "," + num(s->hasMap()) + "," +
               num(s->getColor()) + "," + num(s->getPlayerTemplate()) + "," +
               num(s->getStartPos()) + "," + num(s->getTeamNumber()) + "," +
               num(s->getNATBehavior()) + "," +
               (s->isHuman() ? wide(s->getName()) : std::string()) + "," + unum(s->getIP()) + "," +
               unum(s->getPort()) + "|";
    }
    return out;
}

static std::string options_row(const std::string& in) {
    ReplayGameInfo g;
    g.reset();
    g.enterGame();
    const Bool ok = ParseAsciiStringToGameInfo(&g, AsciiString(in.c_str()));
    std::string row = "{\"in\":\"" + hex(in) + "\",\"ok\":" + (ok ? "1" : "0");
    if (ok) {
        const AsciiString back = GameInfoToAsciiString(&g);
        row += ",\"info\":\"" + info_dump(g) + "\",\"out\":\"" + hex(back.str()) + "\"";
    }
    return row + "}";
}

// ---- options strings -----------------------------------------------------------------

static const char* const kHead = "US=1;M=07maps/alpine assault;MC=ABCD1234;MS=123456;SD=42;C=100;SR=0;SC=10000;O=N;";
static const char* const kSlots = "S=HPlayer,C0A80101,8088,TT,1,2,3,0,1:CM,2,-1,-1,-1:O:X:X:X:X:X:;";

static std::string with_slots(const std::string& slots) { return std::string(kHead) + "S=" + slots + ";"; }
static std::string human(const std::string& rest) { return with_slots(rest + ":O:O:O:O:O:O:O:"); }

static std::string random_valid_options() {
    static const char* maps[] = {"maps/alpine assault", "maps/x", "userdata/maps/my map", "maps/a%3Bb%3D%5B%5D%25c",
                                 "maps\\mixed\\sep", "MAPS/Upper Case", "maps/dir/sub"};
    std::string s = "US=" + num(pick(2)) + ";M=" + (pick(2) ? "0f" : "3a") + maps[pick(7)] +
                    ";MC=" + unum(pick(0xffffffffu)) + ";MS=" + unum(pick(0xffffffffu)) +
                    ";SD=" + num(static_cast<int>(pick(0xffffffffu))) + ";C=" + num(pick(300)) +
                    ";SR=" + num(pick(70000)) + ";SC=" + unum(pick(100000)) + ";O=" + (pick(2) ? "Y" : "N") + ";S=";
    static const char* names[] = {"a", "Bob", "x y", "\xc3\xa9t\xc3\xa9", "\xf0\x9f\x98\x80", "Z q", "\xc0\xaf", "raw\x80"};
    for (int i = 0; i < 8; ++i) {
        switch (pick(4)) {
            case 0: {
                char buf[256];
                std::snprintf(buf, sizeof buf, "H%s,%X,%u,%c%c,%d,%d,%d,%d,%u:", names[pick(8)], pick(0xffffffffu),
                              pick(65536), pick(2) ? 'T' : 'F', pick(2) ? 'T' : 'F', (int)pick(9) - 1,
                              (int)pick(14) - 2, (int)pick(9) - 1, (int)pick(5) - 1, pick(129));
                s += buf;
                break;
            }
            case 1: {
                char buf[128];
                std::snprintf(buf, sizeof buf, "C%c,%d,%d,%d,%d:", "EMH"[pick(3)], (int)pick(9) - 1,
                              (int)pick(14) - 2, (int)pick(9) - 1, (int)pick(5) - 1);
                s += buf;
                break;
            }
            case 2: s += "O:"; break;
            default: s += "X:"; break;
        }
    }
    return s + ";";
}

static std::vector<std::string> options_inputs() {
    std::vector<std::string> v;
    const std::string good = std::string(kHead) + kSlots;
    v.push_back(good);
    // Required keys, one removed at a time.
    for (const char* key : {"US=", "M=", "MC=", "MS=", "SD=", "C=", "SR=", "SC=", "O=", "S="}) {
        std::string s = good;
        const size_t at = s.find(std::string(";") + key) != std::string::npos ? s.find(std::string(";") + key) + 1 : 0;
        s.erase(at, s.find(';', at) - at + 1);
        v.push_back(s);
    }
    auto sub = [&](const std::string& from, const std::string& to) {
        std::string s = good;
        s.replace(s.find(from), from.size(), to);
        v.push_back(s);
    };
    sub("SD=42", "SD=");
    sub("SD=42", "SD==5");
    sub("SD=42", "SD= -17xyz");
    sub("SD=42", "SD=99999999999999999999");
    sub("SD=42", "SD=-99999999999999999999");
    sub("SD=42", "SD=4294967297");
    sub("SD=42", "=SD=7");
    sub("SD=42", "SD=1\n2");
    sub("SD=42", "SD=\n3");
    sub("SD=42", "ZZ=1");
    sub("SD=42", "sd=1");
    sub("SD=42", "SD");
    sub("SD=42", "===");
    sub("MC=ABCD1234", "MC=0x1F");
    sub("MC=ABCD1234", "MC=-1");
    sub("MC=ABCD1234", "MC=zz");
    sub("MC=ABCD1234", "MC= +12g");
    sub("MC=ABCD1234", "MC=123456789ABCDEF01234");
    sub("MC=ABCD1234", "MC=1FFFFFFFF");
    sub("MC=ABCD1234", "MC=0x");
    sub("MS=123456", "MS=-5");
    sub("C=100", "C=-1");
    sub("SR=0", "SR=65537");
    sub("SR=0", "SR=-1");
    sub("SC=10000", "SC=-1");
    sub("SC=10000", "SC=0");
    sub("SC=10000", "SC=99999999999999999999999");
    sub("O=N", "O=y");
    sub("O=N", "O=Yes");
    sub("US=1", "US=-3");
    for (const char* m : {"07", "7", "07x", "xymaps/a", "7zmaps/a", " 7maps/a", "-1maps/a", "0xmaps/a",
                          "07elsewhere/a", "07maps", "07maps/", "07maps//a", "07maps/a//", "07/maps/a", "07maps/a/b/c",
                          "07maps/%41%7a", "07maps/%4", "07maps/%zz", "07maps/%4g", "07maps/% 5", "07maps/%-1",
                          "07maps/%+5", "07maps/%00q", "07maps/%0x", "07maps/a%", "07maps/..", "07maps/..%2F..%2Fetc",
                          "07userdata/maps/u", "07save/s"}) {
        sub("07maps/alpine assault", m);
    }
    for (const char* h : {
             "H", "Hn", "Hn,1", "Hn,1,2", "Hn,1,2,T", "Hn,1,2,TTT", "Hn,1,2,XX", "Hn,1,2,FT", "Hn,1,2,TF",
             "Hn,1,2,TT,0", "Hn,1,2,TT,0,0", "Hn,1,2,TT,0,0,0", "Hn,1,2,TT,0,0,0,0", "Hn,1,2,TT,0,0,0,0,0",
             "Hn,1,2,TT,-1,-2,-1,-1,128", "Hn,1,2,TT,-2,0,0,0,1", "Hn,1,2,TT,8,0,0,0,1", "Hn,1,2,TT,0,-3,0,0,1",
             "Hn,1,2,TT,0,12,0,0,1", "Hn,1,2,TT,0,11,7,3,1", "Hn,1,2,TT,0,0,8,0,1", "Hn,1,2,TT,0,0,-2,0,1",
             "Hn,1,2,TT,0,0,0,4,1", "Hn,1,2,TT,0,0,0,-2,1", "Hn,1,2,TT,0,0,0,0,129", "Hn,1,2,TT,0,0,0,0,-1",
             "Hn,,,1,,2,,TT,0,0,0,0,1", "Hn,zz,zz,TT,a,b,c,d,e", "Hn,0xFFFFFFFF,70000,TT,0,0,0,0,1",
             "Hn,-2,-3,TT,0,0,0,0,1", "H,1,2,TT,0,0,0,0,1", "H\r\nx,1,2,TT,0,0,0,0,1",
             "H\xed\xa0\x80,1,2,TT,0,0,0,0,1", "H\xf4\x90\x80\x80,1,2,TT,0,0,0,0,1",
             "H\xe2\x82,1,2,TT,0,0,0,0,1", "Hn,1,2,TT,0,0,0,0,1,extra",
             "C", "CE", "CE,0", "CE,0,0", "CE,0,0,0", "CE,0,0,0,0", "CM,7,11,7,3", "CH,-1,-2,-1,-1",
             "CQ,0,0,0,0", "CE,8,0,0,0", "CE,0,12,0,0", "CE,0,0,8,0", "CE,0,0,0,4", "CE,0,0,-2,0",
             "C,E,0,0,0,0", "CEzz,0,0,0,0", "Z", "Oanything", "Xanything"}) {
        v.push_back(human(h));
    }
    v.push_back(with_slots("O:O:O:O:O:O:O:"));
    v.push_back(with_slots("O:O:O:O:O:O:O:O:O:"));
    v.push_back(with_slots("::O::O:O:O:O:O:O:O:"));
    v.push_back(with_slots("CE,0,0,2,0:CE,0,0,2,0:O:O:O:O:O:O:"));
    v.push_back(with_slots("X:Hn,1,2,FF,0,0,0,0,1:O:O:O:O:O:O:"));
    v.push_back(std::string(kHead) + "S=;");
    v.push_back(good + ";;;");
    v.push_back(";;" + good);
    v.push_back(good + "S=X:X:X:X:X:X:X:X:;");
    v.push_back(good + "bad");
    v.push_back("bad;" + good);
    v.push_back("");
    // A late error after every required key was seen.
    for (const char* tail : {"M=ab", "M=07elsewhere/a", "SD=", "ZZ=1", "="}) v.push_back(good + tail);
    sub("SD=42", "SD=-10000000000000000000");
    v.push_back(human("Hn\x01m,1,2,TT,0,0,0,0,1"));
    // A second list over slots already filled.
    v.push_back(with_slots("O:Hn,1,2,FF,0,0,0,0,1:CE,1,1,1,1:O:O:O:O:O:") + "S=O:X:CM,2,2,2,2:O:O:O:O:O:;");
    v.push_back(with_slots("O:CE,1,1,1,1:CE,1,1,1,1:O:O:O:O:O:") + "S=O:CH,9,0,0,0:Hq,5,6,TT,0,0,0,0,1:O:O:O:O:O:;");
    v.push_back(with_slots("O:CE,1,1,1,1:O:O:O:O:O:O:") + "S=O:CQ:O:O:O:O:O:O:;");
    for (int i = 0; i < 150; ++i) v.push_back(random_valid_options());
    // Random damage.
    for (int i = 0; i < 400; ++i) {
        std::string s = i % 2 ? random_valid_options() : good;
        const int edits = 1 + pick(3);
        for (int e = 0; e < edits; ++e) {
            const size_t at = pick(static_cast<uint32_t>(s.size()));
            static const char alphabet[] = ",:;=0123456789-HCOXSTFEM xaz%\n";
            switch (pick(3)) {
                case 0: s[at] = alphabet[pick(sizeof alphabet - 1)]; break;
                case 1: s.erase(at, 1); break;
                default: s.insert(at, 1, alphabet[pick(sizeof alphabet - 1)]); break;
            }
        }
        v.push_back(s);
    }
    return v;
}

// ---- command dumps -------------------------------------------------------------------

static size_t kind_size(int kind) {
    switch (kind) {
        case ARGUMENTDATATYPE_BOOLEAN: return 1;
        case ARGUMENTDATATYPE_LOCATION: return 12;
        case ARGUMENTDATATYPE_PIXEL: return 8;
        case ARGUMENTDATATYPE_PIXELREGION: return 16;
        default: return 4;
    }
}

static std::string command_dump(const GameMessage* m) {
    std::string out = num(m->getType()) + " " + num(m->getPlayerIndex()) + " " + num(m->getArgumentCount());
    for (int i = 0; i < m->getArgumentCount(); ++i) {
        const int kind = const_cast<GameMessage*>(m)->getArgumentDataType(i);
        out += " " + num(kind) + ":" + hex(m->getArgument(i), kind_size(kind));
    }
    return out;
}

// ---- replay playback -----------------------------------------------------------------

static std::string write_tmp(const std::string& bytes) {
    const std::string path = g_tmp + "/in.rep";
    FILE* f = std::fopen(path.c_str(), "wb");
    if (!f || std::fwrite(bytes.data(), 1, bytes.size(), f) != bytes.size()) die("tmp write");
    std::fclose(f);
    return path;
}

static std::string header_dump(const RecorderClass::ReplayHeader& h) {
    std::string d = num(h.startTime) + " " + num(h.endTime) + " " + unum(h.frameCount) + " " +
                    num(h.desyncGame) + " " + num(h.quitEarly) + " ";
    for (int i = 0; i < MAX_SLOTS; ++i) d += num(h.playerDiscons[i]);
    d += " [" + wide(h.replayName) + "] " + hex(&h.timeVal, sizeof(h.timeVal)) + " [" + wide(h.versionString) +
         "] [" + wide(h.versionTimeString) + "] " + unum(h.versionNumber) + " " + unum(h.exeCRC) + " " +
         unum(h.iniCRC) + " " + hex(h.gameOptions.str()) + " " + num(h.localPlayerIndex);
    return d;
}

struct Played {
    bool header_ok = false;
    bool play_ok = false;
    std::string header;
    std::string new_game;
    int mode = 0;
    std::vector<std::pair<uint32_t, std::string>> commands;
    std::vector<int> types;
    std::vector<std::string> crcs;
};

static Played play(const std::string& path, oracle_wp51::World& w, uint32_t frame_limit) {
    Played p;
    TheRecorder->reset();
    TheCommandList->reset();
    RecorderClass::ReplayHeader h = {};
    p.header_ok = TheRecorder->readReplayHeader(h, AsciiString(path.c_str()), FALSE);
    if (!p.header_ok) return p;
    p.header = header_dump(h);
    TheRecorder->reset();
    TheCommandList->reset();
    w.set_frame(0);
    p.play_ok = TheRecorder->playbackFile(AsciiString(path.c_str()));
    if (!p.play_ok) return p;
    for (GameMessage* m = TheCommandList->getFirstMessage(); m; m = m->next()) p.new_game += command_dump(m);
    p.mode = TheRecorder->getGameMode();
    for (uint32_t frame = 0; TheRecorder->getMode() == RECORDERMODETYPE_PLAYBACK; ++frame) {
        if (frame > frame_limit) die("playback did not stop");
        TheCommandList->reset();
        w.set_frame(frame);
        TheRecorder->update();
        for (GameMessage* m = TheCommandList->getFirstMessage(); m; m = m->next()) {
            p.commands.push_back({frame, command_dump(m)});
            p.types.push_back(m->getType());
            if (m->getType() == GameMessage::MSG_LOGIC_CRC)
                p.crcs.push_back(num(frame) + " " + num(m->getPlayerIndex()) + " " + unum(static_cast<uint32_t>(m->getArgument(0)->integer)));
        }
    }
    TheCommandList->reset();
    TheMessageStream->reset();
    return p;
}

static void put_u8(std::string& s, uint32_t v) { s += static_cast<char>(v); }
static void put_u16(std::string& s, uint32_t v) { put_u8(s, v & 255); put_u8(s, (v >> 8) & 255); }
static void put_u32(std::string& s, uint32_t v) { put_u16(s, v & 0xffff); put_u16(s, v >> 16); }
static void put_wide(std::string& s, const std::vector<uint32_t>& chars, int width) {
    for (uint32_t c : chars) width == 2 ? put_u16(s, c) : put_u32(s, c);
    width == 2 ? put_u16(s, 0) : put_u32(s, 0);
}

struct Head {
    std::string magic = "GENREP";
    int width = 2;
    std::vector<uint32_t> name = {'L', 'a', 's', 't'};
    std::string options = std::string(kHead) + kSlots;
    std::string local = "0";
};

static std::string build_header(const Head& h) {
    std::string s = h.magic;
    put_u32(s, 1000 + pick(100));
    put_u32(s, 2000 + pick(100));
    put_u32(s, pick(5000));
    put_u8(s, pick(2));
    put_u8(s, pick(2));
    for (int i = 0; i < 8; ++i) put_u8(s, pick(2));
    put_wide(s, h.name, h.width);
    for (int i = 0; i < 8; ++i) put_u16(s, 2000 + i);
    put_wide(s, {'1', '.', '0', '4'}, h.width);
    put_wide(s, {'t', 0x00e9, 0x4e2d}, h.width);
    put_u32(s, pick(0xffffffffu));
    put_u32(s, pick(0xffffffffu));
    put_u32(s, pick(0xffffffffu));
    s += h.options;
    s += '\0';
    s += h.local;
    s += '\0';
    put_u32(s, pick(3));
    put_u32(s, 3);
    put_u32(s, pick(4));
    put_u32(s, pick(2) ? 30 : 0);
    return s;
}

struct Group { uint32_t kind; uint32_t count; };

static void put_command(std::string& s, uint32_t frame, uint32_t type, uint32_t player,
                        const std::vector<Group>& groups, const std::vector<std::string>& args) {
    put_u32(s, frame);
    put_u32(s, type);
    put_u32(s, player);
    put_u8(s, static_cast<uint32_t>(groups.size()));
    for (const Group& g : groups) { put_u8(s, g.kind); put_u8(s, g.count); }
    for (const std::string& a : args) s += a;
}

static std::string random_bytes(size_t n) {
    std::string s;
    for (size_t i = 0; i < n; ++i) put_u8(s, pick(256));
    return s;
}

static std::string random_arg(uint32_t kind, int width) {
    if (kind == ARGUMENTDATATYPE_BOOLEAN) return std::string(1, static_cast<char>(pick(2)));
    if (kind == ARGUMENTDATATYPE_WIDECHAR) return random_bytes(width);
    if (kind >= ARGUMENTDATATYPE_UNKNOWN) return std::string();
    return random_bytes(kind_size(kind));
}

static void random_command(std::string& s, uint32_t frame, int width) {
    static const uint32_t types[] = {1001, 1095, 1068, 1043, 1047, 1000, 27, 1, 1999, 2047};
    std::vector<Group> groups;
    std::vector<std::string> args;
    const int n = pick(4);
    for (int g = 0; g < n; ++g) {
        const uint32_t kind = pick(12);
        const uint32_t count = pick(8) ? 1 + pick(3) : 0;
        groups.push_back({kind, count});
    }
    // The data must follow the same walk as the reader, zero counts included.
    size_t group = 0, total = 0;
    for (const Group& g : groups) total += g.count;
    uint32_t kind = groups.empty() ? 11 : groups[0].kind;
    int left = groups.empty() ? 0 : static_cast<int>(groups[0].count);
    for (size_t j = 0; j < total; ++j) {
        args.push_back(random_arg(kind, width));
        if (--left == 0 && ++group < groups.size()) {
            kind = groups[group].kind;
            left = static_cast<int>(groups[group].count);
        }
    }
    put_command(s, frame, types[pick(10)], pick(10) ? pick(8) : 0xffffffffu, groups, args);
}

static std::vector<std::string> replay_inputs() {
    std::vector<std::string> v;
    Head plain;
    // Every argument type.
    {
        std::string s = build_header(plain);
        for (uint32_t k = 0; k <= 11; ++k)
            put_command(s, k, 1068, 2, {{k, 2}}, {random_arg(k, 2), random_arg(k, 2)});
        put_command(s, 12, 1095, 0, {{0, 1}, {2, 1}}, {std::string("\x78\x56\x34\x12", 4), std::string(1, '\0')});
        put_command(s, 12, 1000, 0, {}, {});
        put_command(s, 12, 27, 0, {}, {});
        put_command(s, 13, 2047, 99, {}, {});
        v.push_back(s);
    }
    for (int width : {2, 4}) {
        Head h;
        h.width = width;
        std::string s = build_header(h);
        put_command(s, 3, 1075, 1, {{10, 2}}, {random_arg(10, width), random_arg(10, width)});
        put_command(s, 9, 1075, 1, {{10, 1}}, {std::string(width, '\xff')});
        // A wide character cut short by the end of the file reads as `EOF`.
        put_command(s, 10, 1075, 1, {{10, 1}}, {std::string(1, 'a')});
        v.push_back(s);
    }
    for (int variant = 0; variant < 14; ++variant) {
        Head h;
        switch (variant) {
            case 0: h.magic = "GENREQ"; break;
            case 1: h.options = "nonsense"; break;
            case 2: h.local = "9"; break;
            case 3: h.local = "-2"; break;
            case 4: h.local = "-1"; break;
            case 5: h.local = "abc"; break;
            case 6: h.local = "7"; break;
            case 7: h.local = "8"; break;
            case 8: h.name = {}; break;
            case 9: h.name = {}; h.width = 4; break;
            case 10: h.name = std::vector<uint32_t>(1023, 'n'); break;
            case 11: h.name = std::vector<uint32_t>(1024, 'n'); break;
            case 12: h.options.insert(h.options.find("SC="), "\xff"); break;
            default: h.options = std::string(kHead) + "SD=" + std::string(900, '0') + ";" + kSlots; break;
        }
        std::string s = build_header(h);
        put_command(s, 1, 1068, 2, {{0, 1}}, {random_arg(0, h.width)});
        v.push_back(s);
    }
    {
        // Options up to the buffer length. Longer ones overrun the C++ stack buffer.
        for (int extra : {1020, 1021, 1022, 1023}) {
            Head h;
            const std::string base = std::string(kHead) + kSlots;
            h.options = base + std::string(extra - base.size(), ';');
            std::string s = build_header(h);
            put_command(s, 1, 1068, 2, {}, {});
            v.push_back(s);
        }
    }
    {
        const std::string head = build_header(plain);
        std::string one = head;
        put_command(one, 5, 1068, 2, {{0, 1}}, {random_arg(0, 2)});
        v.push_back(head);  // no commands
        {
            // Cut after a descriptor's type, and after the descriptor count.
            std::string s = one;
            std::string next;
            put_command(next, 6, 1068, 3, {{0, 2}}, {});
            v.push_back(s + next.substr(0, next.size() - 1));
            v.push_back(s + next.substr(0, next.size() - 2));
        }
        for (int cut : {1, 2, 3}) v.push_back(head + std::string(cut, '\x01'));
        for (int extra = 4; extra <= 13; ++extra) {
            // A second command cut inside its frame, type, player or count.
            std::string s = one;
            std::string next;
            put_command(next, 6, 1068, 3, {}, {});
            s += next.substr(0, std::min<size_t>(extra, next.size()));
            v.push_back(s);
        }
        for (uint32_t second : {4u, 5u, 6u, 5u + 108000u - 1u, 5u + 108000u, 0xffffffffu}) {
            std::string s = one;
            put_command(s, second, 1074, 2, {}, {});
            v.push_back(s);
        }
        for (uint32_t first : {0u, 107999u, 108000u, 0xfffffffeu}) {
            std::string s = head;
            put_command(s, first, 1074, 2, {}, {});
            v.push_back(s);
        }
    }
    {
        // Descriptors with a zero count.
        std::string s = build_header(plain);
        put_command(s, 1, 1068, 2, {{1, 0}, {0, 3}}, {random_bytes(4), random_bytes(4), random_bytes(4)});
        put_command(s, 2, 1068, 2, {{0, 1}, {6, 0}, {2, 2}}, {random_bytes(4), random_bytes(12), random_bytes(12)});
        put_command(s, 3, 1068, 2, {{0, 0}}, {});
        v.push_back(s);
    }
    {
        // More than 255 arguments: the count wraps.
        std::string s = build_header(plain);
        std::vector<std::string> args;
        for (int i = 0; i < 258; ++i) args.push_back(random_bytes(4));
        put_command(s, 1, 1068, 2, {{0, 200}, {3, 58}}, args);
        v.push_back(s);
    }
    for (int i = 0; i < 120; ++i) {
        Head h;
        h.width = i % 3 ? 2 : 4;
        h.options = random_valid_options();
        h.local = num(static_cast<int>(pick(9)) - 1);
        std::string s = build_header(h);
        uint32_t frame = pick(4);
        const int n = 1 + pick(12);
        for (int c = 0; c < n; ++c) {
            random_command(s, frame, h.width);
            frame += pick(3) ? pick(4) : 0;
        }
        v.push_back(s);
    }
    return v;
}

static std::string replay_row(const std::string& bytes, oracle_wp51::World& w) {
    const Played p = play(write_tmp(bytes), w, 200000);
    std::string row = "{\"in\":\"" + hex(bytes) + "\",\"header\":\"" + (p.header_ok ? p.header : "") +
                      "\",\"play\":" + (p.play_ok ? "1" : "0") + ",\"mode\":" + num(p.mode) + ",\"new_game\":\"" +
                      p.new_game + "\",\"commands\":[";
    for (size_t i = 0; i < p.commands.size(); ++i)
        row += (i ? ",\"" : "\"") + num(p.commands[i].first) + " " + p.commands[i].second + "\"";
    return row + "]}";
}

// ---- recording -----------------------------------------------------------------------

static void random_args(GameMessage* m, int count) {
    for (int i = 0; i < count; ++i) {
        const uint32_t kind = pick(11);
        const int run = pick(3) ? 1 : 2;
        for (int r = 0; r < run; ++r) {
            const uint32_t a = pick(0xffffffffu), b = pick(0xffffffffu), c = pick(0xffffffffu), d = pick(0xffffffffu);
            switch (kind) {
                case 0: m->appendIntegerArgument(static_cast<Int>(a)); break;
                case 1: m->appendRealArgument(oracle::from_bits(a)); break;
                case 2: m->appendBooleanArgument(a & 1); break;
                case 3: m->appendObjectIDArgument(static_cast<ObjectID>(a)); break;
                case 4: m->appendDrawableIDArgument(static_cast<DrawableID>(a)); break;
                case 5: m->appendTeamIDArgument(a); break;
                case 6: { Coord3D l = {oracle::from_bits(a), oracle::from_bits(b), oracle::from_bits(c)}; m->appendLocationArgument(l); break; }
                case 7: { ICoord2D p = {static_cast<Int>(a), static_cast<Int>(b)}; m->appendPixelArgument(p); break; }
                case 8: { IRegion2D g = {{static_cast<Int>(a), static_cast<Int>(b)}, {static_cast<Int>(c), static_cast<Int>(d)}}; m->appendPixelRegionArgument(g); break; }
                case 9: m->appendTimestampArgument(a); break;
                default: m->appendWideCharArgument(static_cast<WideChar>(a & (pick(2) ? 0xffff : 0x1fffff))); break;
            }
        }
    }
}

static std::string slurp(const std::string& path) {
    FILE* f = std::fopen(path.c_str(), "rb");
    if (!f) die("cannot read back a file");
    std::string s;
    char buf[4096];
    size_t n;
    while ((n = std::fread(buf, 1, sizeof buf, f)) > 0) s.append(buf, n);
    std::fclose(f);
    return s;
}

static std::string record_row(oracle_wp51::World& w, int index) {
    TheRecorder->reset();
    TheCommandList->reset();
    ReplayGameInfo parsed;
    parsed.reset();
    parsed.enterGame();
    const std::string options = index ? random_valid_options() : std::string(kHead) + kSlots;
    if (!ParseAsciiStringToGameInfo(&parsed, AsciiString(options.c_str()))) die("writer options");
    SkirmishGameInfo skirmish;
    skirmish.init();
    skirmish.enterGame();
    for (int i = 0; i < MAX_SLOTS; ++i) skirmish.setSlot(i, *parsed.getConstSlot(i));
    skirmish.setMap(parsed.getMap());
    skirmish.setMapCRC(parsed.getMapCRC());
    skirmish.setMapSize(parsed.getMapSize());
    skirmish.setMapContentsMask(parsed.getMapContentsMask());
    skirmish.setSeed(parsed.getSeed());
    skirmish.setStartingCash(parsed.getStartingCash());
    TheSkirmishGameInfo = &skirmish;
    TheWritableGlobalData->m_exeCRC = pick(0xffffffffu);
    TheWritableGlobalData->m_iniCRC = pick(0xffffffffu);

    std::string inputs;
    uint32_t frame = 0;
    w.set_frame(frame);
    GameMessage* start = newInstance(GameMessage)(GameMessage::MSG_NEW_GAME);
    start->appendIntegerArgument(GAME_SKIRMISH);
    const int extra = index % 4;
    if (extra >= 1) start->appendIntegerArgument(pick(3));
    if (extra >= 2) start->appendIntegerArgument(pick(9));
    if (extra >= 3) start->appendIntegerArgument(30 + pick(100));
    TheCommandList->appendMessage(start);
    TheRecorder->update();
    if (TheRecorder->getMode() != RECORDERMODETYPE_RECORD) die("recording did not start");

    const int n = 1 + pick(15);
    for (int c = 0; c < n; ++c) {
        frame += pick(5);
        w.set_frame(frame);
        TheCommandList->reset();
        const int batch = 1 + pick(3);
        for (int b = 0; b < batch; ++b) {
            static const int types[] = {1000, 1001, 1002, 1095, 1068, 1043, 1097, 1999, 29, 5, 999, 2000};
            const int type = types[pick(12)];
            GameMessage* m = newInstance(GameMessage)(static_cast<GameMessage::Type>(type));
            m->friend_setPlayerIndex(pick(8));
            random_args(m, c == 1 && b == 0 ? 0 : pick(5));
            TheCommandList->appendMessage(m);
            inputs += (inputs.empty() ? "\"" : ",\"") + num(frame) + " " + command_dump(m) + "\"";
        }
        TheRecorder->update();
    }
    frame += 1 + pick(5);
    w.set_frame(frame);
    TheCommandList->reset();
    GameMessage* end = newInstance(GameMessage)(GameMessage::MSG_CLEAR_GAME_DATA);
    TheCommandList->appendMessage(end);
    inputs += (inputs.empty() ? "\"" : ",\"") + num(frame) + " " + command_dump(end) + "\"";
    TheRecorder->update();
    TheCommandList->reset();
    TheSkirmishGameInfo = nullptr;

    std::string bytes = slurp(std::string(RecorderClass::getReplayDir().str()) + "00000000.rep");
    // The wall clock: start time, end time and the SYSTEMTIME after the name.
    if (bytes.size() < 64) die("short recording");
    std::memset(&bytes[6], 0, 8);
    size_t at = 28;
    while (at + 1 < bytes.size() && (bytes[at] || bytes[at + 1])) at += 2;
    at += 2;
    for (int i = 0; i < 16; ++i) bytes[at + i] = static_cast<char>(i % 2 ? 7 : 0xea + i);
    return "{\"end_frame\":" + num(frame) + ",\"out\":\"" + hex(bytes) + "\",\"inputs\":[" + inputs + "]}";
}

// ---- local replays -------------------------------------------------------------------

static uint64_t fnv(const std::string& s, uint64_t h = 0xcbf29ce484222325ull) {
    for (unsigned char c : s) h = (h ^ c) * 0x100000001b3ull;
    return h;
}

static std::string local_row(oracle_wp51::World& w, const std::string& dir, const std::string& name) {
    const Played p = play(dir + "/" + name, w, 2000000);
    if (!p.header_ok || !p.play_ok) die(("cannot play " + name).c_str());
    std::string all, crcs;
    for (const auto& c : p.commands) all += num(c.first) + " " + c.second + "\n";
    for (const auto& c : p.crcs) crcs += c + "\n";
    std::map<int, int> by_type;
    for (int t : p.types) ++by_type[t];
    std::string counts;
    for (const auto& kv : by_type) counts += (counts.empty() ? "" : " ") + num(kv.first) + ":" + num(kv.second);
    return "{\"file\":\"" + oracle::json_escape(name) + "\",\"header_fnv\":\"" + unum(fnv(p.header)) +
           "\",\"commands\":" + num(p.commands.size()) + ",\"commands_fnv\":\"" + unum(fnv(all)) +
           "\",\"crc_messages\":" + num(p.crcs.size()) + ",\"crc_fnv\":\"" + unum(fnv(crcs)) +
           "\",\"by_type\":\"" + counts + "\"}";
}

int main(int argc, char** argv) {
    const bool local = argc == 4 && std::string(argv[1]) == "--local";
    if (argc != 2 && !local) die("usage: gen <out.json> | --local <dir> <out.json>");
    oracle_wp51::World w;

    const char* base = std::getenv("TMPDIR");
    std::string tmpl = std::string(base ? base : "/tmp") + "/zh_rep_oracle.XXXXXX";
    const char* made = mkdtemp(&tmpl[0]);
    if (!made) die("mkdtemp");
    g_tmp = made;
    TheWritableGlobalData->*access_member(UserDataDirTag()) = AsciiString((g_tmp + "/").c_str());
    TheWritableGlobalData->m_defaultStartingCash.init();
    TheWritableGlobalData->m_defaultStartingCash.deposit(10000, FALSE, FALSE);

    TheGameEngine = new IdleEngine;
    TheLocalFileSystem = NEW StdLocalFileSystem;
    TheFileSystem = NEW FileSystem;
    TheVersion = NEW Version;
    TheMapCache = NEW MapCache;
    TheCommandList = NEW CommandList;
    TheRecorder = NEW RecorderClass;
    TheRecorder->init();
    TheRecorder->setArchiveEnabled(FALSE);
    if (std::string(RecorderClass::getReplayDir().str()).rfind(g_tmp, 0) != 0) die("replay directory is not temporary");

    const int colors = 8;
    for (int i = 0; i < colors; ++i)
        TheMultiplayerSettings->newMultiplayerColorDefinition(AsciiString(("OracleColor" + std::to_string(i)).c_str()));
    const int templates = local ? 64 : 12;
    std::string ini;
    for (int i = 0; i < templates; ++i) ini += "PlayerTemplate FactionOracle" + std::to_string(i) + "\nEnd\n";
    w.load_ini(ini);
    if (TheMultiplayerSettings->getNumColors() != colors) die("color count");
    if (ThePlayerTemplateStore->getPlayerTemplateCount() != templates) die("template count");

    std::string out;
    if (local) {
        std::vector<std::string> names;
        DIR* d = opendir(argv[2]);
        if (!d) die("no replay directory");
        while (dirent* e = readdir(d)) {
            const std::string n = e->d_name;
            if (n.size() > 4 && n.compare(n.size() - 4, 4, ".rep") == 0) names.push_back(n);
        }
        closedir(d);
        std::sort(names.begin(), names.end());
        out = "{\n\"colors\":8,\"templates\":64,\n\"replays\":[\n";
        for (size_t i = 0; i < names.size(); ++i)
            out += local_row(w, argv[2], names[i]) + (i + 1 < names.size() ? ",\n" : "\n");
        out += "]\n}\n";
        oracle::write_file_atomic(argv[3], out);
    } else {
        const std::string root = "vendor/GeneralsX/";
        out = "{\n\"source_sha256\":{\"recorder\":\"" +
              oracle::sha256_file((root + "GeneralsMD/Code/GameEngine/Source/Common/Recorder.cpp").c_str()) +
              "\",\"game_info\":\"" +
              oracle::sha256_file((root + "Core/GameEngine/Source/GameNetwork/GameInfo.cpp").c_str()) +
              "\"},\n\"colors\":8,\"templates\":" + num(templates) + ",\n\"options\":[\n";
        const std::vector<std::string> opts = options_inputs();
        int accepted = 0;
        for (size_t i = 0; i < opts.size(); ++i) {
            const std::string row = options_row(opts[i]);
            accepted += row.find("\"ok\":1") != std::string::npos;
            out += row + (i + 1 < opts.size() ? ",\n" : "\n");
        }
        out += "],\n\"replays\":[\n";
        const std::vector<std::string> reps = replay_inputs();
        for (size_t i = 0; i < reps.size(); ++i)
            out += replay_row(reps[i], w) + (i + 1 < reps.size() ? ",\n" : "\n");
        out += "],\n\"recordings\":[\n";
        const int recordings = 60;
        for (int i = 0; i < recordings; ++i)
            out += record_row(w, i) + (i + 1 < recordings ? ",\n" : "\n");
        out += "]\n}\n";
        std::fprintf(stderr, "options %zu (%d accepted), replays %zu\n", opts.size(), accepted, reps.size());
        oracle::write_file_atomic(argv[1], out);
    }

    unlink((g_tmp + "/in.rep").c_str());
    unlink((g_tmp + "/Replays/00000000.rep").c_str());
    rmdir((g_tmp + "/Replays").c_str());
    rmdir(g_tmp.c_str());
    std::fflush(nullptr);
    // The partial world is not torn down.
    _exit(0);
}
