// PKT-163 (Module) + PKT-164 (ModuleFactory) (WP-5.0) C++ Golden Vector Generator (Zero-Stub Build).
// Compiles directly against vendor/GeneralsX/build/macos-vulkan via scripts/build_oracle.py.
//
// Canonical build & verify commands:
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt164_module_factory.cpp -o /tmp/gen_pkt164_o0 --opt O0
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt164_module_factory.cpp -o /tmp/gen_pkt164_o2 --opt O2
//   python3 scripts/build_oracle.py tests/cpp_harness/gen_pkt164_module_factory.cpp -o /tmp/gen_pkt164_ubsan --opt O1 --sanitize
//   SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy /tmp/gen_pkt164_o2 docs/golden_vectors/PKT-164.json

#include "oracle_common.h"

#include <sstream>
#include <string>
#include <vector>

#include "Common/Module.h"
#include "Common/ModuleFactory.h"
#include "Common/NameKeyGenerator.h"

ORACLE_DEFINE_LINK_GLOBALS("gen_pkt164_module_factory")

namespace {

// All unique module class names listed in ModuleFactory.cpp (including #ifdef-guarded names
// so the oracle also verifies disabled modules return 0 mask when compiled out).
static const char* const kCandidateModuleNames[] = {
    "AIUpdateInterface",
    "ActiveBody",
    "ActiveShroudUpgrade",
    "AnimatedParticleSysBoneClientUpdate",
    "AnimationSteeringUpdate",
    "ArmorUpgrade",
    "AssaultTransportAIUpdate",
    "AssistedTargetingUpdate",
    "AutoDepositUpdate",
    "AutoFindHealingUpdate",
    "AutoHealBehavior",
    "BaikonurLaunchPower",
    "BaseRegenerateUpdate",
    "BattleBusSlowDeathBehavior",
    "BattlePlanUpdate",
    "BeaconClientUpdate",
    "BoneFXDamage",
    "BoneFXUpdate",
    "BridgeBehavior",
    "BridgeScaffoldBehavior",
    "BridgeTowerBehavior",
    "BunkerBusterBehavior",
    "CashBountyPower",
    "CashHackSpecialPower",
    "CaveContain",
    "CheckpointUpdate",
    "ChinookAIUpdate",
    "CleanupAreaPower",
    "CleanupHazardUpdate",
    "CommandButtonHuntUpdate",
    "CommandSetUpgrade",
    "ConvertToCarBombCrateCollide",
    "ConvertToHijackedVehicleCrateCollide",
    "CostModifierUpgrade",
    "CountermeasuresBehavior",
    "CreateCrateDie",
    "CreateObjectDie",
    "CrushDie",
    "DamDie",
    "DefaultProductionExitUpdate",
    "DefectorSpecialPower",
    "DeletionUpdate",
    "DeliverPayloadAIUpdate",
    "DemoTrapUpdate",
    "DemoralizeSpecialPower",
    "DeployStyleAIUpdate",
    "DestroyDie",
    "DozerAIUpdate",
    "DumbProjectileBehavior",
    "DynamicGeometryInfoUpdate",
    "DynamicShroudClearingRangeUpdate",
    "EMPUpdate",
    "EjectPilotDie",
    "EnemyNearUpdate",
    "ExperienceScalarUpgrade",
    "FXListDie",
    "FireOCLAfterWeaponCooldownUpdate",
    "FireSpreadUpdate",
    "FireWeaponCollide",
    "FireWeaponPower",
    "FireWeaponUpdate",
    "FireWeaponWhenDamagedBehavior",
    "FireWeaponWhenDeadBehavior",
    "FirestormDynamicGeometryInfoUpdate",
    "FlammableUpdate",
    "FlightDeckBehavior",
    "FloatUpdate",
    "GarrisonContain",
    "GenerateMinefieldBehavior",
    "GrantScienceUpgrade",
    "GrantStealthBehavior",
    "GrantUpgradeCreate",
    "HackInternetAIUpdate",
    "HealContain",
    "HealCrateCollide",
    "HeightDieUpdate",
    "HelicopterSlowDeathBehavior",
    "HelixContain",
    "HighlanderBody",
    "HijackerUpdate",
    "HiveStructureBody",
    "HordeUpdate",
    "ImmortalBody",
    "InactiveBody",
    "InstantDeathBehavior",
    "InternetHackContain",
    "JetAIUpdate",
    "JetSlowDeathBehavior",
    "KeepObjectDie",
    "LaserUpdate",
    "LeafletDropBehavior",
    "LifetimeUpdate",
    "LockWeaponCreate",
    "LocomotorSetUpgrade",
    "MaxHealthUpgrade",
    "MinefieldBehavior",
    "MissileAIUpdate",
    "MissileLauncherBuildingUpdate",
    "MobMemberSlavedUpdate",
    "MobNexusContain",
    "ModelConditionUpgrade",
    "MoneyCrateCollide",
    "NeutronBlastBehavior",
    "NeutronMissileSlowDeathBehavior",
    "NeutronMissileUpdate",
    "OCLSpecialPower",
    "OCLUpdate",
    "ObjectCreationUpgrade",
    "OpenContain",
    "OverchargeBehavior",
    "OverlordContain",
    "POWTruckAIUpdate",
    "POWTruckBehavior",
    "ParachuteContain",
    "ParkingPlaceBehavior",
    "ParticleUplinkCannonUpdate",
    "PassengersFireUpgrade",
    "PhysicsBehavior",
    "PilotFindVehicleUpdate",
    "PointDefenseLaserUpdate",
    "PoisonedBehavior",
    "PowerPlantUpdate",
    "PowerPlantUpgrade",
    "PreorderCreate",
    "PrisonBehavior",
    "PrisonDockUpdate",
    "ProductionUpdate",
    "ProjectileStreamUpdate",
    "ProneUpdate",
    "PropagandaCenterBehavior",
    "PropagandaTowerBehavior",
    "QueueProductionExitUpdate",
    "RadarUpdate",
    "RadarUpgrade",
    "RadiusDecalUpdate",
    "RailedTransportAIUpdate",
    "RailedTransportContain",
    "RailedTransportDockUpdate",
    "RailroadBehavior",
    "RebuildHoleBehavior",
    "RebuildHoleExposeDie",
    "RepairDockUpdate",
    "ReplaceObjectUpgrade",
    "RiderChangeContain",
    "SabotageCommandCenterCrateCollide",
    "SabotageFakeBuildingCrateCollide",
    "SabotageInternetCenterCrateCollide",
    "SabotageMilitaryFactoryCrateCollide",
    "SabotagePowerPlantCrateCollide",
    "SabotageSuperweaponCrateCollide",
    "SabotageSupplyCenterCrateCollide",
    "SabotageSupplyDropzoneCrateCollide",
    "SalvageCrateCollide",
    "ShroudCrateCollide",
    "SlavedUpdate",
    "SlowDeathBehavior",
    "SmartBombTargetHomingUpdate",
    "SpawnBehavior",
    "SpawnPointProductionExitUpdate",
    "SpecialAbility",
    "SpecialAbilityUpdate",
    "SpecialPowerCompletionDie",
    "SpecialPowerCreate",
    "SpectreGunshipDeploymentUpdate",
    "SpectreGunshipUpdate",
    "SpyVisionSpecialPower",
    "SpyVisionUpdate",
    "SquishCollide",
    "StatusBitsUpgrade",
    "StealthDetectorUpdate",
    "StealthUpdate",
    "StealthUpgrade",
    "StickyBombUpdate",
    "StructureBody",
    "StructureCollapseUpdate",
    "StructureToppleUpdate",
    "SubObjectsUpgrade",
    "SupplyCenterCreate",
    "SupplyCenterDockUpdate",
    "SupplyCenterProductionExitUpdate",
    "SupplyTruckAIUpdate",
    "SupplyWarehouseCreate",
    "SupplyWarehouseCripplingBehavior",
    "SupplyWarehouseDockUpdate",
    "SwayClientUpdate",
    "TechBuildingBehavior",
    "TensileFormationUpdate",
    "ToppleUpdate",
    "TransitionDamageFX",
    "TransportAIUpdate",
    "TransportContain",
    "TunnelContain",
    "UndeadBody",
    "UnitCrateCollide",
    "UnpauseSpecialPowerUpgrade",
    "UpgradeDie",
    "VeterancyCrateCollide",
    "VeterancyGainCreate",
    "W3DModelDraw",
    "WanderAIUpdate",
    "WaveGuideUpdate",
    "WeaponBonusUpdate",
    "WeaponBonusUpgrade",
    "WeaponSetUpgrade",
    "WorkerAIUpdate",
};

struct CatalogEntry {
    std::string name;
    Int module_type;
    Int interface_mask;
    Int decorated_key;
};

static Int decorated_key_for(const std::string& name, ModuleType mtype) {
    char tmp[256];
    tmp[0] = static_cast<char>('0' + static_cast<int>(mtype));
    std::snprintf(tmp + 1, sizeof(tmp) - 1, "%s", name.c_str());
    return static_cast<Int>(TheNameKeyGenerator->nameToKey(tmp));
}

static uint32_t rot_crc(uint32_t crc, uint32_t word) {
    return ((crc << 1) | (crc >> 31)) + word;
}

class ProbeModule : public Module {
public:
    explicit ProbeModule(const ModuleData* data) : Module(data) {}
    NameKeyType getModuleNameKey() const override { return NAMEKEY_INVALID; }
    void runCrc(Xfer* x) { Module::crc(x); }
    void runXfer(Xfer* x) { Module::xfer(x); }
};

} // namespace

int main(int argc, char** argv) {
    if (argc < 2) {
        std::fprintf(stderr, "Usage: %s <out.json>\n", argv[0]);
        return 1;
    }
    const char* out_path = argv[1];

    oracle::init_memory();
    TheNameKeyGenerator = NEW NameKeyGenerator;
    TheNameKeyGenerator->init();

    TheModuleFactory = NEW ModuleFactory;
    TheModuleFactory->init();

    // Build complete catalog with deterministic fresh NameKeyGenerator decorated keys.
    std::vector<CatalogEntry> catalog;
    const size_t num_names = sizeof(kCandidateModuleNames) / sizeof(kCandidateModuleNames[0]);
    const ModuleType types[3] = { MODULETYPE_BEHAVIOR, MODULETYPE_DRAW, MODULETYPE_CLIENT_UPDATE };

    for (size_t i = 0; i < num_names; ++i) {
        const char* name = kCandidateModuleNames[i];
        for (int t = 0; t < 3; ++t) {
            Int mask = TheModuleFactory->findModuleInterfaceMask(AsciiString(name), types[t]);
            if (mask != 0) {
                Int dkey = decorated_key_for(name, types[t]);
                catalog.push_back(CatalogEntry{ name, static_cast<Int>(types[t]), mask, dkey });
            }
        }
    }

    // Verify Module & ModuleFactory Xfer/CRC behavior on a clean factory instance.
    ModuleData dummy_data;
    ProbeModule probe_mod(&dummy_data);
    oracle::MemoryXfer mod_save(XFER_SAVE);
    mod_save.open("probe");
    probe_mod.runXfer(&mod_save);
    XferCRC mod_crc;
    mod_crc.open("probe_crc");
    probe_mod.runCrc(&mod_crc);
    mod_crc.close();

    ModuleFactory xfer_factory;
    xfer_factory.init();
    xfer_factory.newModuleDataFromINI(nullptr, AsciiString("DestroyDie"), MODULETYPE_BEHAVIOR, AsciiString("ModuleTag_01"));
    xfer_factory.newModuleDataFromINI(nullptr, AsciiString("ActiveBody"), MODULETYPE_BEHAVIOR, AsciiString("ModuleTag_02"));
    oracle::MemoryXfer mf_save(XFER_SAVE);
    mf_save.open("mf");
    xfer_factory.xfer(&mf_save);
    XferCRC mf_crc;
    mf_crc.open("mf_crc");
    xfer_factory.crc(&mf_crc);
    mf_crc.close();

    // Generate 1,000 deterministic query & xfer parity vectors.
    static const char* const kExtraNames[] = {
        "",
        "UnknownBehavior",
        "activebody",
        "ACTIVEBODY",
        "aiupdateinterface",
        "W3DModelDraw",
        "POWTruckBehavior",
        "DemoralizeSpecialPower",
    };
    const size_t num_extra = sizeof(kExtraNames) / sizeof(kExtraNames[0]);

    std::ostringstream out;
    out << "{\n";
    out << "  \"packet_id\": \"PKT-164\",\n";
    out << "  \"module_name\": \"ModuleFactory\",\n";
    out << "  \"reference_commit\": \"84a7a20769aa5749f388b1f49bd542c79b3c33e8\",\n";
    out << "  \"regime\": \"USE_DETERMINISTIC_MATH=1,RETAIL_COMPATIBLE_CRC=0\",\n";
    out << "  \"source_sha256\": {\n";
    out << "    \"Module.h\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/Module.h") << "\",\n";
    out << "    \"Module.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/Thing/Module.cpp") << "\",\n";
    out << "    \"ModuleFactory.h\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Include/Common/ModuleFactory.h") << "\",\n";
    out << "    \"ModuleFactory.cpp\": \"" << oracle::sha256_file("vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/Common/Thing/ModuleFactory.cpp") << "\"\n";
    out << "  },\n";
    out << "  \"seed_0_diagnostics\": {\n";
    out << "    \"registered_count\": " << catalog.size() << ",\n";
    out << "    \"module_xfer_hex\": \"" << mod_save.hex() << "\",\n";
    out << "    \"module_crc\": " << mod_crc.getCRC() << ",\n";
    out << "    \"factory_xfer_hex\": \"" << mf_save.hex() << "\",\n";
    out << "    \"factory_crc\": " << mf_crc.getCRC() << "\n";
    out << "  },\n";
    out << "  \"seed_0_catalog\": [\n";
    for (size_t i = 0; i < catalog.size(); ++i) {
        const auto& e = catalog[i];
        out << "    {\"name\":\"" << oracle::json_escape(e.name) << "\""
            << ",\"module_type\":" << e.module_type
            << ",\"interface_mask\":" << e.interface_mask
            << ",\"decorated_key\":" << e.decorated_key
            << "}" << (i + 1 < catalog.size() ? "," : "") << "\n";
    }
    out << "  ],\n";
    out << "  \"vector_count\": 1000,\n";
    out << "  \"vectors\": [\n";

    for (uint32_t seed = 0; seed < 1000; ++seed) {
        uint32_t rng = seed * 0x9e3779b9u + 0x7f4a7c15u;
        uint32_t query_crc = 0;
        std::vector<std::string> q_names;
        std::vector<Int> q_types;
        std::vector<Int> q_masks;
        std::vector<Int> q_keys;

        for (int q = 0; q < 8; ++q) {
            uint32_t pick = oracle::splitmix32(rng);
            std::string qname;
            Int mtype = static_cast<Int>(oracle::splitmix32(rng) % 3u);
            if ((pick % 5u) == 0u) {
                qname = kExtraNames[oracle::splitmix32(rng) % num_extra];
            } else {
                const auto& entry = catalog[oracle::splitmix32(rng) % catalog.size()];
                qname = entry.name;
                if ((pick % 4u) != 0u) {
                    mtype = entry.module_type;
                }
            }
            ModuleType mt = static_cast<ModuleType>(mtype);
            Int mask = TheModuleFactory->findModuleInterfaceMask(AsciiString(qname.c_str()), mt);
            Int dkey = decorated_key_for(qname, mt);
            query_crc = rot_crc(query_crc, static_cast<uint32_t>(mask));
            query_crc = rot_crc(query_crc, static_cast<uint32_t>(dkey));
            q_names.push_back(qname);
            q_types.push_back(mtype);
            q_masks.push_back(mask);
            q_keys.push_back(dkey);
        }

        out << "    {\"seed\":" << seed << ",\"queries\":[";
        for (size_t q = 0; q < q_names.size(); ++q) {
            out << "[\"" << oracle::json_escape(q_names[q]) << "\"," << q_types[q] << "]";
            if (q + 1 < q_names.size()) out << ",";
        }
        out << "],\"masks\":[";
        for (size_t q = 0; q < q_masks.size(); ++q) {
            out << q_masks[q] << (q + 1 < q_masks.size() ? "," : "");
        }
        out << "],\"keys\":[";
        for (size_t q = 0; q < q_keys.size(); ++q) {
            out << q_keys[q] << (q + 1 < q_keys.size() ? "," : "");
        }
        out << "],\"query_crc\":" << query_crc << "}" << (seed + 1 < 1000 ? "," : "") << "\n";
    }

    out << "  ]\n}\n";
    oracle::write_file_atomic(out_path, out.str());
    std::fprintf(stderr, "Generated PKT-164 oracle (%zu catalog entries, 1000 vectors) -> %s\n",
                 catalog.size(), out_path);

    delete TheModuleFactory;
    TheModuleFactory = nullptr;
    delete TheNameKeyGenerator;
    TheNameKeyGenerator = nullptr;
    return 0;
}
