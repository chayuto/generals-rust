// Local-only oracle: loads the installed game's rule data with the reference parsers and
// writes one digest per (category, field), then every line read from the synthetic
// game_data_probe*.ini beside this file (a second run of this program, so the stores
// stay separate).
// For the retail data the output holds counts and digests, never data.
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

// The defines of ObjectCreationList.cpp, needed before its headers are first read.
#define DEFINE_SHADOW_NAMES
#define DEFINE_WEAPONSLOTTYPE_NAMES
#define private public
#define protected public
#include "oracle_wp51_world.h"
// The creation nugget classes are local to their source file.
#include "../../vendor/GeneralsX/GeneralsMD/Code/GameEngine/Source/GameLogic/Object/ObjectCreationList.cpp"
#include "GameClient/TerrainRoads.h"
#include "GameLogic/SidesList.h"
#include "Common/XferCRC.h"
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

static std::string noneToEmpty(const AsciiString& s) {
    return s.compareNoCase("None") == 0 ? std::string() : std::string(s.str());
}
static std::string fxName(const FXList* fx) {
    if (!fx) return "";
    for (auto& kv : TheFXListStore->m_fxmap)
        if (&kv.second == fx) return TheNameKeyGenerator->keyToName(kv.first).str();
    return "?";
}
static std::string psName(const ParticleSystemTemplate* t) { return t ? t->getName().str() : ""; }
static std::string weaponName(const WeaponTemplate* w) { return w ? w->m_name.str() : ""; }
static std::string XY(const Coord2D& c) { return R(c.x) + "," + R(c.y); }
static std::string XYZ(const Coord3D& c) { return R(c.x) + "," + R(c.y) + "," + R(c.z); }

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
        put("weapon", n, "fire_sound", S(w->m_fireSound.getEventName()));
        put("weapon", n, "fire_sound_loop_time", I(w->m_fireSoundLoopTime));
        std::string ffx, dfx, ex, focl, docl, bonus;
        for (int i = 0; i < LEVEL_COUNT; ++i) {
            ffx += fxName(w->m_fireFXs[i]) + ",";
            dfx += fxName(w->m_projectileDetonateFXs[i]) + ",";
            ex += psName(w->m_projectileExhausts[i]) + ",";
            focl += noneToEmpty(w->m_fireOCLNames[i]) + ",";
            docl += noneToEmpty(w->m_projectileDetonationOCLNames[i]) + ",";
        }
        put("weapon", n, "fire_fx", ffx);
        put("weapon", n, "projectile_detonation_fx", dfx);
        put("weapon", n, "projectile_exhaust", ex);
        put("weapon", n, "fire_ocl", focl);
        put("weapon", n, "projectile_detonation_ocl", docl);
        put("weapon", n, "historic_bonus_weapon",
            w->m_historicBonusWeapon ? S(w->m_historicBonusWeapon->m_name) : "");
        for (int c = 0; c < WEAPONBONUSCONDITION_COUNT; ++c) {
            WeaponBonus none;
            const WeaponBonus& b = w->m_extraBonus ? w->m_extraBonus->m_bonus[c] : none;
            for (int f = 0; f < WeaponBonus::FIELD_COUNT; ++f) bonus += R(b.m_field[f]);
        }
        put("weapon", n, "extra_bonus", bonus);
#undef F
#undef N
#undef T
    }
}

static void dumpLocomotors() {
    for (auto& kv : TheLocomotorStore->m_locomotorTemplates) {
        const LocomotorTemplate* l = (const LocomotorTemplate*)kv.second->getFinalOverride();
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

static void dumpAiData() {
    const TAiData* d = TheAI->m_aiData;
    const std::string n = "AIData";
#define F(name) put("ai_data", n, #name, R(d->m_##name))
#define N(member, name) put("ai_data", n, name, I((long long)d->member))
    F(structureSeconds); F(teamSeconds); N(m_resourcesWealthy, "resourcesWealthy"); N(m_resourcesPoor, "resourcesPoor");
    N(m_forceIdleFramesCount, "forceIdleFramesCount"); F(structuresWealthyMod); F(teamWealthyMod); F(structuresPoorMod);
    F(teamPoorMod); F(teamResourcesToBuild); F(guardInnerModifierAI); F(guardOuterModifierAI);
    F(guardInnerModifierHuman); F(guardOuterModifierHuman); N(m_guardChaseUnitFrames, "guardChaseUnitFrames");
    N(m_guardEnemyScanRate, "guardEnemyScanRate"); N(m_guardEnemyReturnScanRate, "guardEnemyReturnScanRate");
    F(skirmishGroupFudgeValue); F(repulsedDistance); N(m_enableRepulsors, "enableRepulsors");
    F(alertRangeModifier); F(aggressiveRangeModifier); N(m_forceSkirmishAI, "forceSkirmishAI");
    N(m_rotateSkirmishBases, "rotateSkirmishBases"); N(m_attackUsesLineOfSight, "attackUsesLineOfSight");
    N(m_attackIgnoreInsignificantBuildings, "attackIgnoreInsignificantBuildings");
    F(attackPriorityDistanceModifier); F(maxRecruitDistance); F(skirmishBaseDefenseExtraDistance);
    F(wallHeight); N(m_minInfantryForGroup, "minInfantryForGroup"); N(m_minVehiclesForGroup, "minVehiclesForGroup");
    F(minDistanceForGroup); F(distanceRequiresGroup); F(minClumpDensity);
    N(m_infantryPathfindDiameter, "infantryPathfindDiameter"); N(m_vehiclePathfindDiameter, "vehiclePathfindDiameter");
    N(m_rebuildDelaySeconds, "rebuildDelaySeconds"); F(supplyCenterSafeRadius); F(aiDozerBoredRadiusModifier);
    N(m_aiCrushesInfantry, "aiCrushesInfantry"); F(maxRetaliateDistance); F(retaliateFriendsRadius);
#undef F
#undef N
    int at = 0;
    for (const AISideInfo* i = d->m_sideInfo; i; i = i->m_next) {
        std::string v = S(i->m_side) + ";" + I(i->m_easy) + "," + I(i->m_normal) + "," + I(i->m_hard) + ";" +
            S(i->m_baseDefenseStructure1) + ";";
        const TSkillSet* sets[] = {&i->m_skillSet1, &i->m_skillSet2, &i->m_skillSet3, &i->m_skillSet4, &i->m_skillSet5};
        for (const TSkillSet* set : sets) {
            for (int k = 0; k < set->m_numSkills; ++k) v += key(set->m_skills[k]) + ",";
            v += ";";
        }
        put("ai_side", std::to_string(1000 + at++), "info", v);
    }
    at = 0;
    for (const AISideBuildList* l = d->m_sideBuildLists; l; l = l->m_next) {
        std::string v = S(l->m_side) + ";";
        for (const BuildListInfo* b = l->m_buildList; b; b = b->m_nextBuildList) {
            v += S(b->m_templateName) + "," + S(b->m_buildingName) + "," + XYZ(b->m_location) + "," +
                XY(b->m_rallyPointOffset) + "," + R(b->m_angle) + "," + I(b->m_isInitiallyBuilt) + "," +
                I(b->m_numRebuilds) + "," + I(b->m_automaticallyBuild) + "," + I(b->m_health) + "," +
                I(b->m_whiner) + "," + I(b->m_unsellable) + "," + I(b->m_repairable) + ";";
        }
        put("ai_build", std::to_string(1000 + at++), "list", v);
    }
    XferCRC crc;
    crc.open("probe");
    TheAI->m_aiData->crc(&crc);
    crc.close();
    put("ai_data", n, "crc", I((UnsignedInt)crc.getCRC()));
}

static std::string decal(const RadiusDecalTemplate& t) {
    return S(t.m_name) + "," + I(t.m_shadowType) + "," + R(t.m_minOpacity) + "," + R(t.m_maxOpacity) + "," +
        I(t.m_opacityThrobTime) + "," + I((UnsignedInt)t.m_color) + "," + I(t.m_onlyVisibleToOwningPlayer);
}

static void dumpCreationLists() {
    for (auto& kv : TheObjectCreationListStore->m_ocls) {
        const std::string n = key(kv.first);
        const auto& nuggets = kv.second.m_nuggets;
        put("ocl", n, "count", I(nuggets.size()));
        for (size_t at = 0; at < nuggets.size(); ++at) {
            const std::string id = n + "#" + std::to_string(at);
            const ObjectCreationNugget* base = nuggets[at];
            auto f = [&](const char* field, const std::string& v) { put("nugget", id, field, v); };
#define F(name, m) f(name, R(g->m))
#define N(name, m) f(name, I((long long)g->m))
#define T(name, m) f(name, S(g->m))
            if (auto* g = dynamic_cast<const GenericObjectCreationNugget*>(base)) {
                f("type", "generic");
                std::string names, anims;
                for (const AsciiString& a : g->m_names) names += S(a) + ",";
                for (const auto& a : g->m_animSets)
                    anims += S(a.m_animInitial) + "," + S(a.m_animFlying) + "," + S(a.m_animFinal) + ";";
                f("names", names); f("anim_sets", anims); f("fx_final", fxName(g->m_fxFinal));
                f("offset", XYZ(g->m_offset)); f("bounce_sound", S(g->m_bounceSound.getEventName()));
                T("put_in_container", m_putInContainer); T("particle_system_name", m_particleSysName);
                T("fade_sound_name", m_fadeSoundName);
                N("names_are_objects", m_nameAreObjects); N("debris_to_generate", m_debrisToGenerate);
                F("mass", m_mass); F("extra_bounciness", m_extraBounciness); F("extra_friction", m_extraFriction);
                N("disposition", m_disposition); F("disposition_intensity", m_dispositionIntensity);
                F("spin_rate", m_spinRate); F("yaw_rate", m_yawRate); F("roll_rate", m_rollRate);
                F("pitch_rate", m_pitchRate); F("min_mag", m_minMag); F("max_mag", m_maxMag);
                F("min_pitch", m_minPitch); F("max_pitch", m_maxPitch); N("min_frames", m_minFrames);
                N("max_frames", m_maxFrames); N("shadow_type", m_shadowType);
                N("min_lod_required", m_minLODRequired); N("invulnerable_time", m_invulnerableTime);
                F("min_health", m_minHealth); F("max_health", m_maxHealth); N("fade_frames", m_fadeFrames);
                F("min_distance_a_formation", m_minDistanceAFormation);
                F("min_distance_b_formation", m_minDistanceBFormation);
                F("max_distance_formation", m_maxDistanceFormation); N("object_count", m_objectCount);
                N("requires_live_player", m_requiresLivePlayer);
                N("contain_inside_source_object", m_containInsideSourceObject);
                N("preserve_layer", m_preserveLayer); N("ok_to_change_model_color", m_okToChangeModelColor);
                N("orient_in_force_direction", m_orientInForceDirection);
                N("spread_formation", m_spreadFormation); N("fade_in", m_fadeIn); N("fade_out", m_fadeOut);
                N("ignore_primary_obstacle", m_ignorePrimaryObstacle);
                N("inherits_veterancy", m_inheritsVeterancy); N("dies_on_bad_land", m_diesOnBadLand);
                N("skip_if_significantly_airborne", m_skipIfSignificantlyAirborne);
            } else if (auto* g = dynamic_cast<const ApplyRandomForceNugget*>(base)) {
                f("type", "force");
                F("spin_rate", m_spinRate); F("min_mag", m_minMag); F("max_mag", m_maxMag);
                F("min_pitch", m_minPitch); F("max_pitch", m_maxPitch);
            } else if (auto* g = dynamic_cast<const DeliverPayloadNugget*>(base)) {
                f("type", "payload");
                T("transport_name", m_transportName); T("put_in_container", m_putInContainerName);
                N("start_at_preferred_height", m_startAtPreferredHeight); N("start_at_max_speed", m_startAtMaxSpeed);
                N("formation_size", m_formationSize); F("formation_spacing", m_formationSpacing);
                F("convergence_factor", m_convergenceFactor); F("error_radius", m_errorRadius);
                N("delay_delivery_frames_max", m_delayDeliveryFramesMax);
                std::string payload;
                for (const auto& pl : g->m_payload) payload += S(pl.m_payloadName) + "," + I(pl.m_payloadCount) + ";";
                f("payload", payload);
                const DeliverPayloadData& d = g->m_data;
                f("visible_drop_bone_name", S(d.m_visibleDropBoneName));
                f("visible_sub_object_name", S(d.m_visibleSubObjectName));
                f("visible_payload_template_name", S(d.m_visiblePayloadTemplateName));
                f("dist_to_target", R(d.m_distToTarget)); f("pre_open_distance", R(d.m_preOpenDistance));
                f("max_attempts", I(d.m_maxAttempts)); f("drop_offset", XYZ(d.m_dropOffset));
                f("drop_variance", XYZ(d.m_dropVariance)); f("drop_delay", I(d.m_dropDelay));
                f("fire_weapon", I(d.m_fireWeapon)); f("self_destruct_object", I(d.m_selfDestructObject));
                f("visible_num_bones", I(d.m_visibleNumBones)); f("dive_start_distance", R(d.m_diveStartDistance));
                f("dive_end_distance", R(d.m_diveEndDistance)); f("strafing_weapon_slot", I((Int)d.m_strafingWeaponSlot));
                f("visible_items_dropped_per_interval", I(d.m_visibleItemsDroppedPerInterval));
                f("inherit_transport_velocity", I(d.m_inheritTransportVelocity));
                f("is_parachute_directly", I(d.m_isParachuteDirectly)); f("exit_pitch_rate", R(d.m_exitPitchRate));
                f("strafe_fx", fxName(d.m_strafeFX)); f("strafe_length", R(d.m_strafeLength));
                f("visible_payload_weapon", weaponName(d.m_visiblePayloadWeaponTemplate));
                f("delivery_decal", decal(d.m_deliveryDecalTemplate));
                f("delivery_decal_radius", R(d.m_deliveryDecalRadius));
            } else if (auto* g = dynamic_cast<const FireWeaponNugget*>(base)) {
                f("type", "fire_weapon");
                f("weapon", weaponName(g->m_weapon));
            } else if (auto* g = dynamic_cast<const AttackNugget*>(base)) {
                f("type", "attack");
                N("number_of_shots", m_numberOfShots); N("weapon_slot", m_weaponSlot);
                f("delivery_decal", decal(g->m_deliveryDecalTemplate));
                F("delivery_decal_radius", m_deliveryDecalRadius);
            } else {
                f("type", "unknown");
            }
#undef F
#undef N
#undef T
        }
    }
}

static void dumpRoad(const char* cat, const TerrainRoadType* r) {
    const std::string n = r->m_name.str();
    auto f = [&](const char* field, const std::string& v) { put(cat, n, field, v); };
#define F(name, m) f(name, R(r->m))
#define N(name, m) f(name, I((long long)r->m))
#define T(name, m) f(name, S(r->m))
    N("id", m_id); N("is_bridge", m_isBridge); T("texture", m_texture); F("road_width", m_roadWidth);
    F("road_width_in_texture", m_roadWidthInTexture); F("bridge_scale", m_bridgeScale);
    T("scaffold_object_name", m_scaffoldObjectName); T("scaffold_support_object_name", m_scaffoldSupportObjectName);
    f("radar_color", R(r->m_radarColor.red) + R(r->m_radarColor.green) + R(r->m_radarColor.blue));
    F("transition_effects_height", m_transitionEffectsHeight); N("num_fx_per_type", m_numFXPerType);
    T("bridge_model_name", m_bridgeModelName); T("bridge_model_name_damaged", m_bridgeModelNameDamaged);
    T("bridge_model_name_really_damaged", m_bridgeModelNameReallyDamaged);
    T("bridge_model_name_broken", m_bridgeModelNameBroken); T("texture_damaged", m_textureDamaged);
    T("texture_really_damaged", m_textureReallyDamaged); T("texture_broken", m_textureBroken);
    std::string towers, sounds, effects;
    for (int i = 0; i < BRIDGE_MAX_TOWERS; ++i) towers += S(r->m_towerObjectName[i]) + ",";
    for (int s = 0; s < BODYDAMAGETYPE_COUNT; ++s) {
        sounds += S(r->m_damageToSoundString[s]) + "," + S(r->m_repairedToSoundString[s]) + ";";
        for (int i = 0; i < MAX_BRIDGE_BODY_FX; ++i)
            effects += S(r->m_damageToOCLString[s][i]) + "," + S(r->m_damageToFXString[s][i]) + "," +
                S(r->m_repairedToOCLString[s][i]) + "," + S(r->m_repairedToFXString[s][i]) + ";";
    }
    f("tower_object_name", towers); f("sounds", sounds); f("effects", effects);
#undef F
#undef N
#undef T
}

static void dumpRoads() {
    for (const TerrainRoadType* r = TheTerrainRoads->m_roadList; r; r = r->m_next) dumpRoad("road", r);
    for (const TerrainRoadType* r = TheTerrainRoads->m_bridgeList; r; r = r->m_next) dumpRoad("bridge", r);
}

static void dumpRuleData() {
    dumpWeapons();
    dumpLocomotors();
    dumpAiData();
    dumpCreationLists();
    dumpRoads();
}

static void loadFile(const std::string& path, INILoadType type) {
    try {
        INI ini;
        ini.load(path.c_str(), type, nullptr);
    } catch (...) {
        fprintf(stdout, "FAILED %s\n", path.c_str());
        fflush(stdout);
        _Exit(1);
    }
}

// The second run: reads the two probe files and writes the sorted lines to `out`.
static void runProbe(const std::string& harnessDir, const std::string& out) {
    // `TAiData` as its constructor leaves it.
    g_lines.clear();
    dumpAiData();
    std::vector<std::string> defaults = g_lines;
    for (std::string& l : defaults) l.replace(0, strlen("ai_data"), "ai_default");
    loadFile(harnessDir + "game_data_probe.ini", INI_LOAD_OVERWRITE);
    loadFile(harnessDir + "game_data_probe_override.ini", INI_LOAD_CREATE_OVERRIDES);
    g_lines = defaults;
    dumpRuleData();
    std::sort(g_lines.begin(), g_lines.end());
    FILE* f = fopen(out.c_str(), "wb");
    for (const std::string& l : g_lines) fprintf(f, "%s\n", l.c_str());
    fclose(f);
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

    std::string harnessDir = __FILE__;
    harnessDir.erase(harnessDir.find_last_of('/') + 1);
    if (argc > 2) {
        runProbe(harnessDir, argv[1]);
        _Exit(0);
    }
    const std::string probeOut = std::string(argv[1]) + ".probe";
    const std::string command = "\"" + std::string(argv[0]) + "\" \"" + probeOut + "\" --probe";
    fflush(stdout);
    if (system(command.c_str()) != 0) return 1;

    static const char* PATHS[] = {
        "Data\\INI\\Default\\GameData", "Data\\INI\\GameData",
        "Data\\INI\\Default\\Science", "Data\\INI\\Science",
        "Data\\INI\\Default\\Roads", "Data\\INI\\Roads",
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
        "Data\\INI\\Default\\AIData", "Data\\INI\\AIData",
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

    dumpRuleData();
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
        fprintf(out, "{\"cat\":\"%s\",\"field\":\"%s\",\"count\":%u,\"fnv\":\"%016llx\"},\n",
            kv.first.first.c_str(), kv.first.second.c_str(), kv.second.first, kv.second.second);
    }
    std::vector<std::string> probe;
    {
        FILE* f = fopen(probeOut.c_str(), "rb");
        char line[4096];
        while (f && fgets(line, sizeof(line), f)) {
            std::string l = line;
            if (!l.empty() && l.back() == '\n') l.pop_back();
            // The vector reader has no escapes: a quote in a value is written as a backtick.
            std::replace(l.begin(), l.end(), '"', '`');
            probe.push_back(l);
        }
        if (f) fclose(f);
        remove(probeOut.c_str());
    }
    for (const std::string& l : probe) {
        fprintf(out, "{\"probe\":\"%s\"}%s\n", l.c_str(), ++i == probe.size() ? "" : ",");
    }
    fprintf(out, "]\n");
    fclose(out);
    _Exit(0);
}
