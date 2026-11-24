// WP-5.1-D DamageModule & DestroyModule (PKT-167, 169, 170, 190) C++ Golden Vector Generator
// Compiles directly against libz_gameengine.a via scripts/build_oracle.py.

#include "oracle_wp51_world.h"

#include "GameLogic/Module/BoneFXDamage.h"
#include "GameLogic/Module/DamageModule.h"
#include "GameLogic/Module/DestroyModule.h"
#include "GameLogic/Module/TransitionDamageFX.h"

int main(int argc, char** argv) {
    const char* out_path = (argc > 1) ? argv[1] : "docs/golden_vectors/WP51_DamageDestroy.json";
    oracle_wp51::World w;

    w.load_ini(
        "FXList FX_WP51_Spark\nEnd\n"
        "FXList FX_WP51_Boom\nEnd\n"
        "ObjectCreationList OCL_WP51_Debris\nEnd\n"
    );

    std::string s0_diag;
    std::ostringstream vecs;

    static const char* kDamageNames[6] = {
        "EXPLOSION", "SMALL_ARMS", "ARMOR_PIERCING", "FLAME", "POISON", "UNRESISTABLE"
    };

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed ^ 0x51d07799u;
        if (rng == 0) rng = 0x2468ace1u;

        auto rcoord = [&](int lo, int hi) -> Real {
            int span = hi - lo + 1;
            return static_cast<Real>(lo + static_cast<int32_t>(oracle::splitmix32(rng) % span)) * 0.5f;
        };

        int slot_d = static_cast<int>(1u + (oracle::splitmix32(rng) % 12u));
        int slot_rd = static_cast<int>(1u + (oracle::splitmix32(rng) % 12u));
        int slot_rb = static_cast<int>(1u + (oracle::splitmix32(rng) % 12u));

        Real lx = rcoord(-40, 40), ly = rcoord(-40, 40), lz = rcoord(0, 30);
        Real ox = rcoord(-20, 20), oy = rcoord(-20, 20), oz = rcoord(0, 20);
        bool rand_bone = ((oracle::splitmix32(rng) & 1u) != 0);
        bool filter_fx_only = (seed % 3 == 0);
        const char* dt_a = kDamageNames[oracle::splitmix32(rng) % 6u];
        const char* dt_b = kDamageNames[oracle::splitmix32(rng) % 6u];

        std::string obj_name = "D51_Unit_" + std::to_string(seed);
        std::ostringstream ini_ss;
        ini_ss << "Object " << obj_name << "\n"
               << "  Body = ActiveBody ModuleTag_Body\n    MaxHealth = 200\n    InitialHealth = 200\n  End\n"
               << "  Behavior = TransitionDamageFX ModuleTag_TDFX\n";
        if (filter_fx_only) {
            ini_ss << "    DamageFXTypes = NONE +" << dt_a << "\n"
                   << "    DamageOCLTypes = ALL -" << dt_b << "\n";
        }
        ini_ss << "    DamagedFXList" << slot_d << " = Loc: X:" << lx << " Y:" << ly << " Z:" << lz << " FXList:FX_WP51_Spark\n"
               << "    ReallyDamagedFXList" << slot_rd << " = Bone:Turret01 RandomBone:" << (rand_bone ? "Yes" : "No") << " FXList:FX_WP51_Boom\n"
               << "    RubbleOCL" << slot_rb << " = Loc: X:" << ox << " Y:" << oy << " Z:" << oz << " OCL:OCL_WP51_Debris\n"
               << "  End\n"
               << "End\n";
        w.load_ini(ini_ss.str(), INI_LOAD_OVERWRITE);

        Object* obj = w.spawn_object(obj_name.c_str(), w.team_a, {10.0f, 20.0f, 0.0f}, 0.0f);
        TransitionDamageFX* tdfx = nullptr;
        for (BehaviorModule** m = obj->getBehaviorModules(); m && *m; ++m) {
            if ((*m)->getDamage()) {
                tdfx = static_cast<TransitionDamageFX*>(*m);
                break;
            }
        }

        uint32_t tdfx_xfer_crc = 0;
        std::string tdfx_hex = oracle_wp51::World::snapshot_save_hex(tdfx, &tdfx_xfer_crc);
        uint32_t tdfx_crc = oracle_wp51::World::snapshot_crc(tdfx);

        // XferLoad roundtrip verification
        oracle::MemoryXfer xs(XFER_SAVE);
        xs.open("save");
        oracle::call_snapshot_xfer(tdfx, &xs);
        xs.close();
        oracle_wp51::World::snapshot_load_bytes(tdfx, xs.bytes);
        uint32_t round_xfer_crc = 0;
        oracle_wp51::World::snapshot_save_hex(tdfx, &round_xfer_crc);

        // Construct BoneFXDamage + DestroyModule directly on heap for Xfer layout verification
        static NameKeyType bfx_key = TheNameKeyGenerator->nameToKey("ModuleTag_BFX");
        DamageModuleData bfx_md;
        bfx_md.setModuleTagNameKey(bfx_key);
        BoneFXDamage* bfx = newInstance(BoneFXDamage)(obj, &bfx_md);
        uint32_t bfx_xfer_crc = 0;
        std::string bfx_hex = oracle_wp51::World::snapshot_save_hex(bfx, &bfx_xfer_crc);
        uint32_t bfx_crc = oracle_wp51::World::snapshot_crc(bfx);
        deleteInstance(bfx);

        TheGameLogic->destroyObject(obj);
        TheGameLogic->processDestroyList();

        if (seed == 0) {
            std::ostringstream d;
            d << "{"
              << "\"tdfx_hex_len\":" << tdfx_hex.size()
              << ",\"tdfx_xfer_crc\":" << tdfx_xfer_crc
              << ",\"bfx_hex\":\"" << bfx_hex << "\""
              << ",\"loc_bits\":[" << oracle::fbits(lx) << "," << oracle::fbits(ly) << "," << oracle::fbits(lz) << "]"
              << "}";
            s0_diag = d.str();
        }

        if (seed > 0) vecs << ",\n";
        vecs << "    {"
             << "\"seed\":" << seed
             << ",\"in\":{\"ini\":\"" << oracle::json_escape(ini_ss.str()) << "\""
             << ",\"slot_d\":" << slot_d << ",\"slot_rd\":" << slot_rd << ",\"slot_rb\":" << slot_rb
             << ",\"rand_bone\":" << (rand_bone ? 1 : 0)
             << ",\"loc\":[" << oracle::fbits(lx) << "," << oracle::fbits(ly) << "," << oracle::fbits(lz) << "]"
             << ",\"ocl_loc\":[" << oracle::fbits(ox) << "," << oracle::fbits(oy) << "," << oracle::fbits(oz) << "]"
             << ",\"filter_fx_only\":" << (filter_fx_only ? 1 : 0)
             << ",\"dt_a\":\"" << dt_a << "\",\"dt_b\":\"" << dt_b << "\""
             << "}"
             << ",\"out\":{\"tdfx_xfer\":\"" << tdfx_hex << "\""
             << ",\"tdfx_xfer_crc\":" << tdfx_xfer_crc
             << ",\"tdfx_crc\":" << tdfx_crc
             << ",\"tdfx_round_crc\":" << round_xfer_crc
             << ",\"bfx_xfer\":\"" << bfx_hex << "\""
             << ",\"bfx_xfer_crc\":" << bfx_xfer_crc
             << ",\"bfx_crc\":" << bfx_crc
             << "}"
             << "}";
    }

    std::ostringstream doc;
    doc << "{\n"
        << "  \"packet\": \"WP-5.1-D\",\n"
        << "  \"modules\": [\"DamageModule\",\"BoneFXDamage\",\"TransitionDamageFX\",\"DestroyModule\"],\n"
        << "  \"seed_count\": 1000,\n"
        << "  \"source_hashes\": {\n"
        << "    \"DamageModule.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Damage/DamageModule.cpp") << "\",\n"
        << "    \"BoneFXDamage.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Damage/BoneFXDamage.cpp") << "\",\n"
        << "    \"TransitionDamageFX.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Damage/TransitionDamageFX.cpp") << "\",\n"
        << "    \"DestroyModule.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Destroy/DestroyModule.cpp") << "\"\n"
        << "  },\n"
        << "  \"seed0_diagnostics\": " << s0_diag << ",\n"
        << "  \"vectors\": [\n" << vecs.str() << "\n  ]\n}\n";

    oracle::write_file_atomic(out_path, doc.str());
    return 0;
}
