// Local-only oracle: loads the installed game's rule data with the reference parsers and
// writes one digest per (category, field). Output holds counts and digests, never data.
//   argv[1]: digest JSON. ZH_DIR: game directory (default $HOME/Games/generalszh).
//   ORACLE_DUMP: optional file for the "category|entry|field=value" lines (do not commit).
// Build: python3 scripts/build_oracle.py tests/cpp_harness/gen_game_data_load.cpp -o <bin>
#include <algorithm>
#include <cstdio>
#include <cstdlib>
#include <cstring>
#include <map>
#include <string>
#include <unordered_map>
#include <vector>

#define private public
#define protected public
#include "oracle_wp51_world.h"
#include "Common/BIGFileSystem.h"
#include "Common/FileSystem.h"
// The device include directory is not on the engine library's include path.
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/StdDevice/Common/StdLocalFileSystem.h"
#include "../../vendor/GeneralsX/Core/GameEngineDevice/Include/W3DDevice/Common/W3DModuleFactory.h"

// Labels stay untranslated: the data parsers reject an empty translation.
class LabelText : public oracle_wp51::NullGameText {
public:
    UnicodeString fetch(const Char* label, Bool* exists = nullptr) override {
        if (exists) *exists = TRUE;
        UnicodeString u;
        u.translate(label);
        return u;
    }
    UnicodeString fetch(AsciiString label, Bool* exists = nullptr) override {
        return fetch(label.str(), exists);
    }
};

static std::vector<std::string> g_lines;

static void put(const char* cat, const std::string& name, const char* field, const std::string& value) {
    g_lines.push_back(std::string(cat) + "|" + name + "|" + field + "=" + value);
}
static std::string R(Real v) {
    UnsignedInt u;
    memcpy(&u, &v, 4);
    char b[16];
    snprintf(b, sizeof(b), "%08x", u);
    return b;
}
static std::string I(long long v) { return std::to_string(v); }
static std::string S(const AsciiString& s) { return s.str(); }
static std::string key(Int k) { return TheNameKeyGenerator->keyToName((NameKeyType)k).str(); }

static void dumpWeapons() {
    for (auto& kv : TheWeaponStore->m_weaponTemplateHashMap) {
        const WeaponTemplate* w = kv.second;
        const std::string n = w->m_name.str();
#define F(name, m) put("weapon", n, name, R(w->m))
#define N(name, m) put("weapon", n, name, I((long long)w->m))
#define T(name, m) put("weapon", n, name, S(w->m))
        F("primary_damage", m_primaryDamage); F("primary_damage_radius", m_primaryDamageRadius);
        F("secondary_damage", m_secondaryDamage); F("secondary_damage_radius", m_secondaryDamageRadius);
        F("shock_wave_amount", m_shockWaveAmount); F("shock_wave_radius", m_shockWaveRadius);
        F("shock_wave_taper_off", m_shockWaveTaperOff); F("attack_range", m_attackRange);
        F("minimum_attack_range", m_minimumAttackRange); F("request_assist_range", m_requestAssistRange);
        F("aim_delta", m_aimDelta); F("scatter_radius", m_scatterRadius);
        F("scatter_target_scalar", m_scatterTargetScalar); F("scatter_radius_vs_infantry", m_infantryInaccuracyDist);
        N("damage_type", m_damageType); N("death_type", m_deathType);
        F("weapon_speed", m_weaponSpeed); F("min_weapon_speed", m_minWeaponSpeed);
        N("scale_weapon_speed", m_isScaleWeaponSpeed); F("weapon_recoil", m_weaponRecoil);
        F("min_target_pitch", m_minTargetPitch); F("max_target_pitch", m_maxTargetPitch);
        F("radius_damage_angle", m_radiusDamageAngle); N("clip_size", m_clipSize);
        N("clip_reload_time", m_clipReloadTime); N("min_delay_between_shots", m_minDelayBetweenShots);
        N("max_delay_between_shots", m_maxDelayBetweenShots);
        N("continuous_fire_one_shots_needed", m_continuousFireOneShotsNeeded);
        N("continuous_fire_two_shots_needed", m_continuousFireTwoShotsNeeded);
        N("continuous_fire_coast_frames", m_continuousFireCoastFrames);
        N("auto_reload_when_idle_frames", m_autoReloadWhenIdleFrames);
        N("shots_per_barrel", m_shotsPerBarrel); N("damage_dealt_at_self_position", m_damageDealtAtSelfPosition);
        N("reload_type", m_reloadType); N("prefire_type", m_prefireType);
        N("pre_attack_delay", m_preAttackDelay); F("continue_attack_range", m_continueAttackRange);
        N("affects_mask", m_affectsMask); N("collide_mask", m_collideMask); N("anti_mask", m_antiMask);
        N("leech_range_weapon", m_leechRangeWeapon); N("capable_of_following_waypoint", m_capableOfFollowingWaypoint);
        N("is_shows_ammo_pips", m_isShowsAmmoPips);
        N("allow_attack_garrisoned_buildings", m_allowAttackGarrisonedBldgs);
        N("play_fx_when_stealthed", m_playFXWhenStealthed); N("missile_calls_on_die", m_dieOnDetonate);
        N("suspend_fx_delay", m_suspendFXDelay); N("damage_status_type", m_damageStatusType);
        T("projectile_name", m_projectileName); T("projectile_stream_name", m_projectileStreamName);
        T("laser_name", m_laserName); T("laser_bone_name", m_laserBoneName);
        N("historic_bonus_time", m_historicBonusTime); F("historic_bonus_radius", m_historicBonusRadius);
        N("historic_bonus_count", m_historicBonusCount);
        std::string pts;
        for (const Coord2D& c : w->m_scatterTargets) pts += R(c.x) + R(c.y);
        put("weapon", n, "scatter_targets", pts);
#undef F
#undef N
#undef T
    }
}

static void dumpLocomotors() {
    for (auto& kv : TheLocomotorStore->m_locomotorTemplates) {
        const LocomotorTemplate* l = kv.second;
        const std::string n = l->m_name.str();
#define F(name, m) put("locomotor", n, name, R(l->m))
#define N(name, m) put("locomotor", n, name, I((long long)l->m))
        N("surfaces", m_surfaces); F("max_speed", m_maxSpeed); F("max_speed_damaged", m_maxSpeedDamaged);
        F("min_speed", m_minSpeed); F("max_turn_rate", m_maxTurnRate); F("max_turn_rate_damaged", m_maxTurnRateDamaged);
        F("acceleration", m_acceleration); F("acceleration_damaged", m_accelerationDamaged);
        F("lift", m_lift); F("lift_damaged", m_liftDamaged); F("braking", m_braking);
        F("min_turn_speed", m_minTurnSpeed); F("preferred_height", m_preferredHeight);
        F("preferred_height_damping", m_preferredHeightDamping); F("circling_radius", m_circlingRadius);
        F("speed_limit_z", m_speedLimitZ); F("extra_2d_friction", m_extra2DFriction);
        F("max_thrust_angle", m_maxThrustAngle); N("behavior_z", m_behaviorZ); N("appearance", m_appearance);
        N("move_priority", m_movePriority); F("accel_pitch_limit", m_accelPitchLimit);
        F("decel_pitch_limit", m_decelPitchLimit); F("bounce_kick", m_bounceKick);
        F("pitch_stiffness", m_pitchStiffness); F("roll_stiffness", m_rollStiffness);
        F("pitch_damping", m_pitchDamping); F("roll_damping", m_rollDamping);
        F("pitch_by_z_vel_coef", m_pitchByZVelCoef); F("thrust_roll", m_thrustRoll);
        F("wobble_rate", m_wobbleRate); F("min_wobble", m_minWobble); F("max_wobble", m_maxWobble);
        F("forward_vel_coef", m_forwardVelCoef); F("lateral_vel_coef", m_lateralVelCoef);
        F("forward_accel_coef", m_forwardAccelCoef); F("lateral_accel_coef", m_lateralAccelCoef);
        F("uniform_axial_damping", m_uniformAxialDamping); F("turn_pivot_offset", m_turnPivotOffset);
        N("airborne_targeting_height", m_airborneTargetingHeight); F("close_enough_dist", m_closeEnoughDist);
        N("is_close_enough_dist3d", m_isCloseEnoughDist3D);
        F("ultra_accurate_slide_factor", m_ultraAccurateSlideIntoPlaceFactor);
        N("locomotor_works_when_dead", m_locomotorWorksWhenDead);
        N("allow_motive_force_airborne", m_allowMotiveForceWhileAirborne);
        N("apply_2d_friction_airborne", m_apply2DFrictionWhenAirborne); N("downhill_only", m_downhillOnly);
        N("stick_to_ground", m_stickToGround); N("can_move_backward", m_canMoveBackward);
        N("has_suspension", m_hasSuspension); F("wheel_turn_angle", m_wheelTurnAngle);
        F("maximum_wheel_extension", m_maximumWheelExtension);
        F("maximum_wheel_compression", m_maximumWheelCompression);
        F("wander_width_factor", m_wanderWidthFactor); F("wander_length_factor", m_wanderLengthFactor);
        F("wander_about_point_radius", m_wanderAboutPointRadius);
        F("rudder_correction_degree", m_rudderCorrectionDegree); F("rudder_correction_rate", m_rudderCorrectionRate);
        F("elevator_correction_degree", m_elevatorCorrectionDegree);
        F("elevator_correction_rate", m_elevatorCorrectionRate);
#undef F
#undef N
    }
}

static void dumpArmor() {
    for (auto& kv : TheArmorStore->m_armorTemplates) {
        std::string c;
        for (int i = 0; i < DAMAGE_NUM_TYPES; ++i) c += R(kv.second.m_damageCoefficient[i]);
        put("armor", key(kv.first), "coefficients", c);
    }
}

static void dumpUpgrades() {
    for (UpgradeTemplate* u = TheUpgradeCenter->m_upgradeList; u; u = u->m_next) {
        const std::string n = u->m_name.str();
        put("upgrade", n, "build_frames", I((Int)(u->m_buildTime * LOGICFRAMES_PER_SECOND)));
        put("upgrade", n, "cost", I(u->m_cost));
    }
}

static void dumpSciences() {
    for (const ScienceInfo* info : TheScienceStore->m_sciences) {
        const ScienceType st = info->m_science;
        const std::string n = key(st);
        put("science", n, "cost", I(TheScienceStore->getSciencePurchaseCost(st)));
        put("science", n, "grantable", I(TheScienceStore->isScienceGrantable(st)));
    }
}

static void dumpSpecialPowers() {
    for (const SpecialPowerTemplate* p : TheSpecialPowerStore->m_specialPowerTemplates) {
        const std::string n = p->getName().str();
        put("special_power", n, "id", I(p->getID()));
        put("special_power", n, "type", I(p->getSpecialPowerType()));
        put("special_power", n, "reload_time", I(p->getReloadTime()));
        const ScienceType st = p->getRequiredScience();
        put("special_power", n, "required_science", st == SCIENCE_INVALID ? std::string() : key(st));
        put("special_power", n, "detection_time", I(p->getDetectionTime()));
        put("special_power", n, "view_object_duration", I(p->getViewObjectDuration()));
        put("special_power", n, "view_object_range", R(p->getViewObjectRange()));
        put("special_power", n, "radius_cursor_radius", R(p->getRadiusCursorRadius()));
        put("special_power", n, "public_timer", I(p->hasPublicTimer()));
        put("special_power", n, "shared_n_sync", I(p->isSharedNSync()));
    }
}

static void dumpCommandSets() {
    for (CommandSet* s = TheControlBar->m_commandSets; s; s = (CommandSet*)s->friend_getNext()) {
        const std::string n = s->m_name.str();
        std::string slots;
        for (int i = 0; i < MAX_COMMANDS_PER_SET; ++i) {
            const CommandButton* b = s->m_command[i];
            if (!b) continue;
            const char type = b->m_command == GUI_COMMAND_UNIT_BUILD ? 'u'
                : b->m_command == GUI_COMMAND_DOZER_CONSTRUCT ? 'd' : 'o';
            const ThingTemplate* t = b->m_thingTemplate;
            const UpgradeTemplate* u = b->m_upgradeTemplate;
            slots += std::string(1, type) + ":" + (t ? t->getName().str() : "") + ":" +
                (u ? u->m_name.str() : "") + ";";
        }
        put("command_set", n, "slots", slots);
    }
}

static void dumpTemplates() {
    for (const ThingTemplate* t = TheThingFactory->firstTemplate(); t; t = t->friend_getNextTemplate()) {
        const std::string n = t->getName().str();
#define F(name, m) put("template", n, name, R(t->m))
#define N(name, m) put("template", n, name, I((long long)t->m))
        N("template_id", m_templateID); N("build_cost", m_buildCost); F("build_time", m_buildTime);
        N("refund_value", m_refundValue); F("vision_range", m_visionRange);
        F("shroud_clearing_range", m_shroudClearingRange); N("energy_production", m_energyProduction);
        N("energy_bonus", m_energyBonus); N("crusher_level", m_crusherLevel);
        N("crushable_level", m_crushableLevel); N("transport_slot_count", m_transportSlotCount);
        N("threat_value", m_threatValue); N("is_trainable", m_isTrainable); F("asset_scale", m_assetScale);
        put("template", n, "command_set", S(t->m_commandSetString));
        std::string kinds, xp;
        for (int i = 0; i < KINDOF_COUNT; ++i) kinds += t->isKindOf((KindOfType)i) ? '1' : '0';
        for (int i = 0; i < LEVEL_COUNT; ++i)
            xp += I(t->m_skillPointValues[i]) + "," + I(t->m_experienceValues[i]) + "," +
                I(t->m_experienceRequired[i]) + ";";
        put("template", n, "kindof", kinds);
        put("template", n, "experience", xp);
#undef F
#undef N
    }
}

int main(int argc, char** argv) {
    if (argc < 2) return 2;
    oracle_wp51::World world;
    TheGameText = new LabelText;

    std::string dir = getenv("ZH_DIR") ? getenv("ZH_DIR") : std::string(getenv("HOME")) + "/Games/generalszh";
    TheLocalFileSystem = new StdLocalFileSystem;
    BIGFileSystem* big = new BIGFileSystem;
    TheArchiveFileSystem = big;
    TheFileSystem = new FileSystem;
    big->loadArchiveFilesFromDirectory(dir.c_str(), "*.big", FALSE, TRUE);
    big->loadArchiveFilesFromDirectory((dir + "/ZH_Generals").c_str(), "*.big", FALSE, FALSE);

    // The fixture's module factory has no draw modules.
    TheModuleFactory = new W3DModuleFactory;
    TheModuleFactory->init();

    static const char* PATHS[] = {
        "Data\\INI\\Default\\GameData", "Data\\INI\\GameData",
        "Data\\INI\\Default\\Science", "Data\\INI\\Science",
        "Data\\INI\\ParticleSystem",
        "Data\\INI\\Default\\FXList", "Data\\INI\\FXList",
        "Data\\INI\\Weapon",
        "Data\\INI\\Default\\ObjectCreationList", "Data\\INI\\ObjectCreationList",
        "Data\\INI\\Locomotor",
        "Data\\INI\\Default\\SpecialPower", "Data\\INI\\SpecialPower",
        "Data\\INI\\DamageFX", "Data\\INI\\Armor",
        "Data\\INI\\Default\\Object", "Data\\INI\\Object",
        "Data\\INI\\Default\\Upgrade", "Data\\INI\\Upgrade",
        "Data\\INI\\Default\\CommandButton", "Data\\INI\\CommandButton", "Data\\INI\\CommandSet",
        // Holds `Object` blocks as well as `CrateData`.
        "Data\\INI\\Default\\Crate", "Data\\INI\\Crate",
    };
    for (const char* p : PATHS) {
        fprintf(stdout, "load %s\n", p);
        fflush(stdout);
        try {
            INI ini;
            // The `Default` directories may be empty, as `initSubsystem` tolerates.
            ini.loadFileDirectory(p, INI_LOAD_OVERWRITE, nullptr, INI::LoadFlags_SearchSubDirs);
        } catch (...) {
            fprintf(stdout, "FAILED %s\n", p);
            return 1;
        }
    }

    dumpWeapons();
    dumpLocomotors();
    dumpArmor();
    dumpUpgrades();
    dumpSciences();
    dumpSpecialPowers();
    dumpCommandSets();
    dumpTemplates();

    std::sort(g_lines.begin(), g_lines.end());
    if (const char* dump = getenv("ORACLE_DUMP")) {
        FILE* f = fopen(dump, "wb");
        for (const std::string& l : g_lines) fprintf(f, "%s\n", l.c_str());
        fclose(f);
    }
    // FNV-1a 64 over the sorted lines of each (category, field).
    std::map<std::pair<std::string, std::string>, std::pair<unsigned, unsigned long long>> sums;
    for (const std::string& l : g_lines) {
        const size_t a = l.find('|');
        const size_t b = l.find('|', a + 1);
        const size_t c = l.find('=', b + 1);
        auto& s = sums[{l.substr(0, a), l.substr(b + 1, c - b - 1)}];
        if (s.first == 0) s.second = 0xcbf29ce484222325ULL;
        ++s.first;
        for (char ch : l + "\n") s.second = (s.second ^ (unsigned char)ch) * 0x100000001b3ULL;
    }
    FILE* out = fopen(argv[1], "wb");
    fprintf(out, "[\n");
    size_t i = 0;
    for (auto& kv : sums) {
        fprintf(out, "{\"cat\":\"%s\",\"field\":\"%s\",\"count\":%u,\"fnv\":\"%016llx\"}%s\n",
            kv.first.first.c_str(), kv.first.second.c_str(), kv.second.first, kv.second.second,
            ++i == sums.size() ? "" : ",");
    }
    fprintf(out, "]\n");
    fclose(out);
    _Exit(0);
}
