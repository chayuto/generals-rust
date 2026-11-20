// WP-2.1 C++ Golden Vector Generator for Xfer, XferCRC, XferDeepCRC, XferSave, XferLoad.
// Compiles directly against unmodified vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer*.cpp.

#include "PreRTS.h"
#include "Common/Xfer.h"
#include "Common/XferCRC.h"
#include "Common/XferDeepCRC.h"
#include "Common/XferSave.h"
#include "Common/XferLoad.h"
#include "Common/Snapshot.h"

#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferCRC.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferSave.cpp"
#include "../../vendor/GeneralsX/Core/GameEngine/Source/Common/System/XferLoad.cpp"

const AsciiString AsciiString::TheEmptyString("");
const UnicodeString UnicodeString::TheEmptyString;

static GameStateStub g_game_state;
GameStateStub* TheGameState = &g_game_state;
static ScienceStoreStub g_science_store;
ScienceStoreStub* TheScienceStore = &g_science_store;
static UpgradeCenterStub g_upgrade_center;
UpgradeCenterStub* TheUpgradeCenter = &g_upgrade_center;

static uint32_t splitmix32(uint32_t& state) {
    uint32_t z = (state += 0x9e3779b9u);
    z = (z ^ (z >> 16)) * 0x85ebca6bu;
    z = (z ^ (z >> 13)) * 0xc2b2ae35u;
    return z ^ (z >> 16);
}

static uint64_t splitmix64_from_two(uint32_t& s) {
    uint64_t hi = splitmix32(s);
    uint64_t lo = splitmix32(s);
    return (hi << 32) | lo;
}

static uint32_t bswap32(uint32_t v) {
    return ((v & 0x000000FFu) << 24) |
           ((v & 0x0000FF00u) << 8) |
           ((v & 0x00FF0000u) >> 8) |
           ((v & 0xFF000000u) >> 24);
}

static uint32_t compute_retail_xfer_crc_step(uint32_t crc, const void* raw_data, Int dataSize) {
    const uint8_t* ptr = static_cast<const uint8_t*>(raw_data);
    Int n = raw_data ? dataSize : 0;
    Int words = n / 4;
    for (Int i = 0; i < words; ++i) {
        uint32_t w = 0;
        std::memcpy(&w, ptr + i * 4, 4);
        crc = (crc << 1) + bswap32(w) + ((crc >> 31) & 0x01u);
    }
    const uint8_t* c = ptr + words * 4;
    Int rem = n & 3;
    if (rem != 0) {
        uint32_t val = 0;
        if (rem == 3) val += static_cast<uint32_t>(c[2]) << 16;
        if (rem >= 2) val += static_cast<uint32_t>(c[1]) << 8;
        val += static_cast<uint32_t>(c[0]);
        crc = (crc << 1) + val + ((crc >> 31) & 0x01u);
    }
    return crc;
}

class RetailDeepCrcMirror : public Xfer {
public:
    uint32_t m_crc = 0;
    RetailDeepCrcMirror() { m_xferMode = XFER_CRC; }
    void open(AsciiString id) override { Xfer::open(id); m_crc = 0; }
    void close() override {}
    Int beginBlock() override { return 0; }
    void endBlock() override {}
    void skip(Int) override {}
    void xferSnapshot(Snapshot* s) override { if (s) s->crc(this); }
    void xferMarkerLabel(AsciiString) override {}
    void xferAsciiString(AsciiString* s) override {
        UnsignedShort len = static_cast<UnsignedShort>(s->getLength());
        xferUnsignedShort(&len);
        if (len > 0) xferUser(const_cast<char*>(s->str()), len);
    }
    void xferUnicodeString(UnicodeString* s) override {
        Byte len = static_cast<Byte>(s->getLength());
        xferByte(&len);
        if (len > 0) xferUser(const_cast<WideChar*>(s->str()), sizeof(WideChar) * len);
    }
    void xferImplementation(void* data, Int dataSize) override {
        if (!data || dataSize < 1) return;
        m_crc = compute_retail_xfer_crc_step(m_crc, data, dataSize);
    }
    uint32_t getCRC() const { return bswap32(m_crc); }
};

static std::vector<uint8_t> read_file_bytes(const char* path) {
    FILE* fp = std::fopen(path, "rb");
    if (!fp) { std::perror(path); std::exit(1); }
    std::fseek(fp, 0, SEEK_END);
    long sz = std::ftell(fp);
    std::fseek(fp, 0, SEEK_SET);
    std::vector<uint8_t> out(static_cast<size_t>(sz));
    if (sz > 0) {
        size_t n = std::fread(out.data(), 1, static_cast<size_t>(sz), fp);
        if (n != static_cast<size_t>(sz)) { std::perror("fread"); std::exit(1); }
    }
    std::fclose(fp);
    return out;
}

static std::string to_hex(const uint8_t* p, size_t n) {
    static const char* kHex = "0123456789abcdef";
    std::string out;
    out.reserve(n * 2);
    for (size_t i = 0; i < n; ++i) {
        out.push_back(kHex[(p[i] >> 4) & 0xF]);
        out.push_back(kHex[p[i] & 0xF]);
    }
    return out;
}

struct CompositeRecord : public Snapshot {
    XferVersion ver = 1;
    Byte b_val = 0;
    UnsignedByte ub_val = 0;
    Bool bool_val = FALSE;
    Short s_val = 0;
    UnsignedShort us_val = 0;
    Int i_val = 0;
    Int64 i64_val = 0;
    UnsignedInt ui_val = 0;
    Real r_val = 0.0f;
    double d_val = 0.0;
    AsciiString ascii;
    UnicodeString unicode;
    Coord2D c2{};
    ICoord2D ic2{};
    Region2D reg2{};
    IRegion2D ireg2{};
    Coord3D c3{};
    ICoord3D ic3{};
    Region3D reg3{};
    IRegion3D ireg3{};
    RealRange rr{};
    Color color = 0;
    RGBColor rgb{};
    RGBAColorReal rgba_real{};
    RGBAColorInt rgba_int{};
    ObjectID obj_id = INVALID_ID;
    DrawableID draw_id = INVALID_DRAWABLE_ID;
    std::vector<ObjectID> obj_vec;
    std::list<Int> int_list;
    Matrix3D mtx{};
    std::vector<uint8_t> user_tail; // length 1..7 (non-empty for fwrite)

    void crc(Xfer* xfer) override {
        xfer->xferMarkerLabel("CompositeRecord");
        xfer->xferByte(&b_val);
        xfer->xferUnsignedByte(&ub_val);
        xfer->xferBool(&bool_val);
        xfer->xferShort(&s_val);
        xfer->xferUnsignedShort(&us_val);
        xfer->xferInt(&i_val);
        xfer->xferInt64(&i64_val);
        xfer->xferUnsignedInt(&ui_val);
        xfer->xferReal(&r_val);
        xfer->xferDouble(&d_val);
        xfer->xferAsciiString(&ascii);
        xfer->xferUnicodeString(&unicode);
        xfer->xferCoord2D(&c2);
        xfer->xferICoord2D(&ic2);
        xfer->xferRegion2D(&reg2);
        xfer->xferIRegion2D(&ireg2);
        xfer->xferCoord3D(&c3);
        xfer->xferICoord3D(&ic3);
        xfer->xferRegion3D(&reg3);
        xfer->xferIRegion3D(&ireg3);
        xfer->xferRealRange(&rr);
        xfer->xferColor(&color);
        xfer->xferRGBColor(&rgb);
        xfer->xferRGBAColorReal(&rgba_real);
        xfer->xferRGBAColorInt(&rgba_int);
        xfer->xferObjectID(&obj_id);
        xfer->xferDrawableID(&draw_id);
        xfer->beginBlock();
        xfer->xferSTLObjectIDVector(&obj_vec);
        xfer->xferSTLIntList(&int_list);
        xfer->xferMatrix3D(&mtx);
        UnsignedByte tail_n = static_cast<UnsignedByte>(user_tail.size());
        xfer->xferUnsignedByte(&tail_n);
        if (xfer->getXferMode() == XFER_LOAD) {
            user_tail.resize(tail_n);
        }
        if (tail_n > 0) {
            xfer->xferUser(user_tail.data(), static_cast<Int>(tail_n));
        }
        xfer->endBlock();
    }

    void xfer(Xfer* xf) override {
        XferVersion v = ver;
        xf->xferVersion(&v, 1);
        crc(xf);
    }

    void loadPostProcess() override {}
};

int main() {
    const uint32_t count = 1000;
    const char* out_path = "docs/golden_vectors/WP2_1_Xfer.json";
    const char* tmp_save = "/tmp/gen_xfer_tmp.bin";
    const char* tmp_deep = "/tmp/gen_xfer_deep.bin";

    FILE* fp = std::fopen(out_path, "wb");
    if (!fp) { std::perror(out_path); return 1; }
    std::fprintf(fp, "{\n  \"packet_id\": \"WP-2.1\",\n  \"module\": \"Xfer\",\n"
                     "  \"oracle\": \"vendor/GeneralsX unmodified Xfer.cpp + XferCRC.cpp + XferSave.cpp + XferLoad.cpp\",\n"
                     "  \"vector_count\": %u,\n  \"vectors\": [\n", count);

    for (uint32_t seed = 0; seed < count; ++seed) {
        uint32_t rng = seed * 0x7feb352du + 0x13572468u;

        // 1) Raw buffer CRC test (0..64 bytes covering 4-byte aligned words + 1/2/3 trailing bytes)
        uint32_t raw_len = (seed < 8) ? seed : (splitmix32(rng) % 65u);
        std::vector<uint8_t> raw_buf(raw_len);
        for (uint32_t i = 0; i < raw_len; ++i) {
            raw_buf[i] = static_cast<uint8_t>(splitmix32(rng) & 0xFFu);
        }
        XferCRC raw_xcrc;
        raw_xcrc.open("raw");
        if (raw_len > 0) {
            raw_xcrc.xferUser(raw_buf.data(), static_cast<Int>(raw_len));
        }
        uint32_t raw_crc = raw_xcrc.getCRC();
        uint32_t retail_raw_crc = bswap32(compute_retail_xfer_crc_step(
            0, raw_buf.data(), static_cast<Int>(raw_buf.size())));

        // 2) Full composite Snapshot record
        CompositeRecord rec;
        rec.b_val = static_cast<Byte>((splitmix32(rng) & 0xFFu) - 128);
        rec.ub_val = static_cast<UnsignedByte>(splitmix32(rng) & 0xFFu);
        rec.bool_val = (splitmix32(rng) & 1u) ? TRUE : FALSE;
        rec.s_val = static_cast<Short>((splitmix32(rng) & 0xFFFFu) - 32768);
        rec.us_val = static_cast<UnsignedShort>(splitmix32(rng) & 0xFFFFu);
        rec.i_val = static_cast<Int>(splitmix32(rng));
        rec.i64_val = static_cast<Int64>(splitmix64_from_two(rng));
        rec.ui_val = splitmix32(rng);
        rec.r_val = static_cast<Real>(static_cast<int32_t>(splitmix32(rng) % 40001u) - 20000) * 0.125f;
        rec.d_val = static_cast<double>(static_cast<int32_t>(splitmix32(rng) % 40001u) - 20000) * 0.0625;

        uint32_t ascii_len = (seed == 0) ? 0u : (splitmix32(rng) % 19u);
        std::string astr;
        for (uint32_t i = 0; i < ascii_len; ++i) {
            astr.push_back(static_cast<char>('A' + (splitmix32(rng) % 26u)));
        }
        rec.ascii = AsciiString(astr.c_str());

        uint32_t uni_len = (seed == 0) ? 0u : (splitmix32(rng) % 14u);
        std::vector<WideChar> uvec(uni_len);
        for (uint32_t i = 0; i < uni_len; ++i) {
            uvec[i] = static_cast<WideChar>(0x0041u + (splitmix32(rng) % 0x4D00u));
        }
        rec.unicode.setUnits(uvec.data(), static_cast<Int>(uni_len));

        auto rval = [&]() -> Real {
            return static_cast<Real>(static_cast<int32_t>(splitmix32(rng) % 2001u) - 1000) * 0.25f;
        };
        auto ival = [&]() -> Int {
            return static_cast<Int>(splitmix32(rng) % 20001u) - 10000;
        };

        rec.c2 = {rval(), rval()};
        rec.ic2 = {ival(), ival()};
        rec.reg2 = {{rval(), rval()}, {rval(), rval()}};
        rec.ireg2 = {{ival(), ival()}, {ival(), ival()}};
        rec.c3 = {rval(), rval(), rval()};
        rec.ic3 = {ival(), ival(), ival()};
        rec.reg3 = {{rval(), rval(), rval()}, {rval(), rval(), rval()}};
        rec.ireg3 = {{ival(), ival(), ival()}, {ival(), ival(), ival()}};
        rec.rr = {rval(), rval()};
        rec.color = static_cast<Color>(splitmix32(rng));
        rec.rgb = {0.25f, 0.5f, 0.75f};
        rec.rgba_real = {0.125f, 0.375f, 0.625f, 0.875f};
        rec.rgba_int = {splitmix32(rng) & 255u, splitmix32(rng) & 255u, splitmix32(rng) & 255u, 255u};
        rec.obj_id = static_cast<ObjectID>(splitmix32(rng) % 50000u);
        rec.draw_id = static_cast<DrawableID>(splitmix32(rng) % 50000u);

        uint32_t ovec_len = splitmix32(rng) % 6u;
        for (uint32_t i = 0; i < ovec_len; ++i) {
            rec.obj_vec.push_back(static_cast<ObjectID>((splitmix32(rng) % 9000u) + 1u));
        }
        uint32_t ilist_len = splitmix32(rng) % 6u;
        for (uint32_t i = 0; i < ilist_len; ++i) {
            rec.int_list.push_back(ival());
        }
        for (int row = 0; row < 3; ++row) {
            rec.mtx[row].X = rval();
            rec.mtx[row].Y = rval();
            rec.mtx[row].Z = rval();
            rec.mtx[row].W = rval();
        }
        uint32_t tail_len = (splitmix32(rng) % 7u) + 1u;
        rec.user_tail.resize(tail_len);
        for (uint32_t i = 0; i < tail_len; ++i) {
            rec.user_tail[i] = static_cast<uint8_t>(splitmix32(rng) & 0xFFu);
        }

        // Run XferCRC (note XferCRC::xferSnapshot calls snapshot->crc(this))
        XferCRC rec_xcrc;
        rec_xcrc.open("rec");
        rec_xcrc.xferSnapshot(&rec);
        uint32_t rec_crc = rec_xcrc.getCRC();

        // Run XferSave (note XferSave::xferSnapshot calls snapshot->xfer(this))
        {
            XferSave xsave;
            xsave.open(tmp_save);
            xsave.xferSnapshot(&rec);
            xsave.close();
        }
        std::vector<uint8_t> save_bytes = read_file_bytes(tmp_save);
        XferCRC save_crc;
        save_crc.xferUser(save_bytes.data(), static_cast<Int>(save_bytes.size()));

        // Verify XferLoad round-trip in C++
        {
            CompositeRecord loaded;
            XferLoad xload;
            xload.open(tmp_save);
            xload.xferSnapshot(&loaded);
            xload.close();
            if (loaded.i_val != rec.i_val || loaded.i64_val != rec.i64_val ||
                !(loaded.ascii == rec.ascii) || loaded.obj_vec != rec.obj_vec ||
                loaded.int_list != rec.int_list || loaded.user_tail != rec.user_tail) {
                std::fprintf(stderr, "C++ XferLoad round-trip mismatch at seed %u!\n", seed);
                return 1;
            }
        }

        // Run XferDeepCRC (note XferDeepCRC::xferSnapshot inherits XferCRC -> snapshot->crc(this))
        uint32_t deep_crc = 0;
        {
            XferDeepCRC xdeep;
            xdeep.open(tmp_deep);
            xdeep.xferSnapshot(&rec);
            deep_crc = xdeep.getCRC();
            xdeep.close();
        }
        std::vector<uint8_t> deep_bytes = read_file_bytes(tmp_deep);
        RetailDeepCrcMirror retail_deep;
        retail_deep.open("retail");
        retail_deep.xferSnapshot(&rec);
        uint32_t retail_rec_crc = retail_deep.getCRC();
        XferCRC deep_bytes_crc;
        if (!deep_bytes.empty()) {
            deep_bytes_crc.xferUser(deep_bytes.data(), static_cast<Int>(deep_bytes.size()));
        }

        uint32_t r_bits = 0;
        uint64_t d_bits = 0;
        std::memcpy(&r_bits, &rec.r_val, 4);
        std::memcpy(&d_bits, &rec.d_val, 8);

        auto fbits = [](Real v) -> uint32_t {
            uint32_t b = 0;
            std::memcpy(&b, &v, 4);
            return b;
        };

        std::fprintf(
            fp,
            "    {\"seed\": %u, \"raw_hex\": \"%s\", \"raw_crc\": %u, \"retail_raw_crc\": %u, "
            "\"b\": %d, \"ub\": %u, \"bool_v\": %u, \"s\": %d, \"us\": %u, \"i\": %d, \"i64\": %lld, \"ui\": %u, "
            "\"r_bits\": %u, \"d_bits\": %llu, \"ascii\": \"%s\", \"u16\": [%s], "
            "\"c2\": [%u,%u], \"ic2\": [%d,%d], "
            "\"reg2\": [%u,%u,%u,%u], \"ireg2\": [%d,%d,%d,%d], "
            "\"c3\": [%u,%u,%u], \"ic3\": [%d,%d,%d], "
            "\"reg3\": [%u,%u,%u,%u,%u,%u], \"ireg3\": [%d,%d,%d,%d,%d,%d], "
            "\"rr\": [%u,%u], \"color\": %d, \"rgba_int\": [%u,%u,%u,%u], "
            "\"obj_id\": %d, \"draw_id\": %d, \"obj_vec\": [%s], \"int_list\": [%s], "
            "\"mtx\": [%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u,%u], \"user_tail_hex\": \"%s\", "
            "\"rec_crc\": %u, \"save_len\": %u, \"save_bytes_crc\": %u, "
            "\"deep_len\": %u, \"deep_crc\": %u, \"deep_bytes_crc\": %u, \"retail_deep_crc\": %u}%s\n",
            seed,
            to_hex(raw_buf.data(), raw_buf.size()).c_str(),
            raw_crc,
            retail_raw_crc,
            static_cast<int>(rec.b_val),
            static_cast<unsigned>(rec.ub_val),
            static_cast<unsigned>(rec.bool_val),
            static_cast<int>(rec.s_val),
            static_cast<unsigned>(rec.us_val),
            rec.i_val,
            static_cast<long long>(rec.i64_val),
            rec.ui_val,
            r_bits,
            static_cast<unsigned long long>(d_bits),
            astr.c_str(),
            [&]() {
                std::string out;
                for (size_t i = 0; i < uvec.size(); ++i) {
                    if (i > 0) out += ",";
                    out += std::to_string(uvec[i]);
                }
                return out;
            }().c_str(),
            fbits(rec.c2.x), fbits(rec.c2.y),
            rec.ic2.x, rec.ic2.y,
            fbits(rec.reg2.lo.x), fbits(rec.reg2.lo.y), fbits(rec.reg2.hi.x), fbits(rec.reg2.hi.y),
            rec.ireg2.lo.x, rec.ireg2.lo.y, rec.ireg2.hi.x, rec.ireg2.hi.y,
            fbits(rec.c3.x), fbits(rec.c3.y), fbits(rec.c3.z),
            rec.ic3.x, rec.ic3.y, rec.ic3.z,
            fbits(rec.reg3.lo.x), fbits(rec.reg3.lo.y), fbits(rec.reg3.lo.z),
            fbits(rec.reg3.hi.x), fbits(rec.reg3.hi.y), fbits(rec.reg3.hi.z),
            rec.ireg3.lo.x, rec.ireg3.lo.y, rec.ireg3.lo.z,
            rec.ireg3.hi.x, rec.ireg3.hi.y, rec.ireg3.hi.z,
            fbits(rec.rr.lo), fbits(rec.rr.hi),
            rec.color,
            rec.rgba_int.red, rec.rgba_int.green, rec.rgba_int.blue, rec.rgba_int.alpha,
            static_cast<int>(rec.obj_id),
            static_cast<int>(rec.draw_id),
            [&]() {
                std::string out;
                for (size_t i = 0; i < rec.obj_vec.size(); ++i) {
                    if (i > 0) out += ",";
                    out += std::to_string(static_cast<int>(rec.obj_vec[i]));
                }
                return out;
            }().c_str(),
            [&]() {
                std::string out;
                size_t idx = 0;
                for (Int v : rec.int_list) {
                    if (idx++ > 0) out += ",";
                    out += std::to_string(v);
                }
                return out;
            }().c_str(),
            fbits(rec.mtx[0].X), fbits(rec.mtx[0].Y), fbits(rec.mtx[0].Z), fbits(rec.mtx[0].W),
            fbits(rec.mtx[1].X), fbits(rec.mtx[1].Y), fbits(rec.mtx[1].Z), fbits(rec.mtx[1].W),
            fbits(rec.mtx[2].X), fbits(rec.mtx[2].Y), fbits(rec.mtx[2].Z), fbits(rec.mtx[2].W),
            to_hex(rec.user_tail.data(), rec.user_tail.size()).c_str(),
            rec_crc,
            static_cast<uint32_t>(save_bytes.size()),
            save_crc.getCRC(),
            static_cast<uint32_t>(deep_bytes.size()),
            deep_crc,
            deep_bytes_crc.getCRC(),
            retail_rec_crc,
            (seed + 1 < count) ? "," : ""
        );
    }

    std::fprintf(fp, "  ]\n}\n");
    std::fclose(fp);
    std::remove(tmp_save);
    std::remove(tmp_deep);
    std::printf("Generated %u WP-2.1 Xfer golden vectors from unmodified Xfer*.cpp.\n", count);
    return 0;
}
