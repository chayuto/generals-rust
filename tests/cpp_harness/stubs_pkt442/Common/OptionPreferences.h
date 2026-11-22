#pragma once
#include "WW3D2/ww3d.h"
#include "WW3D2/texturefilter.h"
#include "Common/GlobalData.h"

struct OptionPreferencesOverrides {
    bool passthrough = true;

    Bool useAlternateMouse = FALSE;
    Bool useRightMouseScrollWithAlternateMouse = TRUE;
    Bool clientRetaliationModeEnabled = TRUE;
    Bool doubleClickAttackMove = FALSE;
    Int jpegQuality = 80;
    Real keyboardScrollFactor = 0.5f;
    Bool drawScrollAnchor = FALSE;
    Bool moveScrollAnchor = TRUE;
    UnsignedInt defaultIP = 0;
    Int firewallBehavior = 0;
    Short firewallPortAllocationDelta = 0;
    UnsignedShort firewallPortOverride = 0;
    Bool saveCameraInReplay = TRUE;
    Bool useCameraInReplay = TRUE;
    Bool enablePlayerObserver = TRUE;
    Int networkLatencyFontSize = 8;
    Int renderFpsFontSize = 8;
    Int systemTimeFontSize = 8;
    Int gameTimeFontSize = 8;
    Int playerInfoListFontSize = 8;
    Bool showMoneyPerMinute = FALSE;
    Real gameWindowTransitionSpeedMultiplier = 1.0f;

    bool hasMaxCameraHeight = false;
    Real maxCameraHeight = 300.0f;
    bool hasTerrainDrawDistanceScale = false;
    Real terrainDrawDistanceScale = 1.0f;

    Int skirmishTickRate = 30;
    Bool advancedShadows = TRUE;
    Bool advancedLighting = TRUE;
    Bool useShadowVolumesForSkins = TRUE;
    Bool extendedShadowFrustumCulling = TRUE;
    Bool naturalSunElevation = TRUE;
    Bool shadowsForProjectiles = TRUE;
    Bool shadowsForProps = TRUE;
    Bool bridgeShadows = TRUE;
    Bool waterReflections = TRUE;
    Bool explosionDynamicLights = TRUE;
    Bool vehicleHeadlights = TRUE;
    Bool enablePostProcessing = TRUE;
    Bool postProcessBloom = TRUE;
    Bool postProcessHDR = TRUE;
    Bool postProcessFXAA = FALSE;
    Bool airUnitGroundIndicator = TRUE;

    UnsignedInt antiAliasing = WW3D::MULTISAMPLE_MODE_NONE;
    UnsignedInt textureFilterMode = TextureFilterClass::TEXTURE_FILTER_BILINEAR;
    UnsignedInt textureAnisotropyLevel = TextureFilterClass::TEXTURE_FILTER_ANISOTROPIC_2X;

    Real gammaValue = 50.0f;
    Int xResolution = DEFAULT_DISPLAY_WIDTH;
    Int yResolution = DEFAULT_DISPLAY_HEIGHT;
};

extern OptionPreferencesOverrides g_optionPrefs;

class OptionPreferences {
public:
    struct const_iterator {
        bool found = false;
        bool operator!=(const const_iterator& o) const { return found != o.found; }
        bool operator==(const const_iterator& o) const { return found == o.found; }
    };

    const_iterator find(const char* key) const {
        if (g_optionPrefs.passthrough || !key) return const_iterator{false};
        if (std::strcmp(key, "MaxCameraHeight") == 0) return const_iterator{g_optionPrefs.hasMaxCameraHeight};
        if (std::strcmp(key, "TerrainDrawDistanceScale") == 0) return const_iterator{g_optionPrefs.hasTerrainDrawDistanceScale};
        return const_iterator{false};
    }

    const_iterator end() const { return const_iterator{false}; }

    Bool getAlternateMouseModeEnabled() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_useAlternateMouse : g_optionPrefs.useAlternateMouse;
    }
    Bool getRightMouseScrollWithAlternateMouseEnabled() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_useRightMouseScrollWithAlternateMouse : g_optionPrefs.useRightMouseScrollWithAlternateMouse;
    }
    Bool getRetaliationModeEnabled() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_clientRetaliationModeEnabled : g_optionPrefs.clientRetaliationModeEnabled;
    }
    Bool getDoubleClickAttackMoveEnabled() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_doubleClickAttackMove : g_optionPrefs.doubleClickAttackMove;
    }
    Int getJpegQuality() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_jpegQuality : g_optionPrefs.jpegQuality;
    }
    Real getScrollFactor() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_keyboardScrollFactor : g_optionPrefs.keyboardScrollFactor;
    }
    Bool getDrawScrollAnchor() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_drawScrollAnchor : g_optionPrefs.drawScrollAnchor;
    }
    Bool getMoveScrollAnchor() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_moveScrollAnchor : g_optionPrefs.moveScrollAnchor;
    }
    UnsignedInt getLANIPAddress() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_defaultIP : g_optionPrefs.defaultIP;
    }
    Int getFirewallBehavior() const {
        return g_optionPrefs.passthrough ? static_cast<Int>(TheGlobalData->m_firewallBehavior) : g_optionPrefs.firewallBehavior;
    }
    Short getFirewallPortAllocationDelta() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_firewallPortAllocationDelta : g_optionPrefs.firewallPortAllocationDelta;
    }
    UnsignedShort getFirewallPortOverride() const {
        return g_optionPrefs.passthrough ? static_cast<UnsignedShort>(TheGlobalData->m_firewallPortOverride) : g_optionPrefs.firewallPortOverride;
    }
    Bool saveCameraInReplays() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_saveCameraInReplay : g_optionPrefs.saveCameraInReplay;
    }
    Bool useCameraInReplays() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_useCameraInReplay : g_optionPrefs.useCameraInReplay;
    }
    Bool getPlayerObserverEnabled() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_enablePlayerObserver : g_optionPrefs.enablePlayerObserver;
    }
    Int getNetworkLatencyFontSize() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_networkLatencyFontSize : g_optionPrefs.networkLatencyFontSize;
    }
    Int getRenderFpsFontSize() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_renderFpsFontSize : g_optionPrefs.renderFpsFontSize;
    }
    Int getSystemTimeFontSize() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_systemTimeFontSize : g_optionPrefs.systemTimeFontSize;
    }
    Int getGameTimeFontSize() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_gameTimeFontSize : g_optionPrefs.gameTimeFontSize;
    }
    Int getPlayerInfoListFontSize() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_playerInfoListFontSize : g_optionPrefs.playerInfoListFontSize;
    }
    Bool getShowMoneyPerMinute() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_showMoneyPerMinute : g_optionPrefs.showMoneyPerMinute;
    }
    Real getGameWindowTransitionSpeedMultiplier() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_gameWindowTransitionSpeedMultiplier : g_optionPrefs.gameWindowTransitionSpeedMultiplier;
    }
    Real getMaxCameraHeight() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_maxCameraHeight : g_optionPrefs.maxCameraHeight;
    }
    Real getTerrainDrawDistanceScale() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_terrainDrawDistanceScale : g_optionPrefs.terrainDrawDistanceScale;
    }
    Int getSkirmishTickRate() const {
        return g_optionPrefs.passthrough ? TheGlobalData->m_skirmishTickRate : g_optionPrefs.skirmishTickRate;
    }
    Bool getAdvancedShadows(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.advancedShadows; }
    Bool getAdvancedLighting(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.advancedLighting; }
    Bool getUseShadowVolumesForSkins(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.useShadowVolumesForSkins; }
    Bool getExtendedShadowFrustumCulling(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.extendedShadowFrustumCulling; }
    Bool getNaturalSunElevation(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.naturalSunElevation; }
    Bool getShadowsForProjectiles(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.shadowsForProjectiles; }
    Bool getShadowsForProps(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.shadowsForProps; }
    Bool getBridgeShadows(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.bridgeShadows; }
    Bool getWaterReflections(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.waterReflections; }
    Bool getExplosionDynamicLights(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.explosionDynamicLights; }
    Bool getVehicleHeadlights(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.vehicleHeadlights; }
    Bool getEnablePostProcessing(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.enablePostProcessing; }
    Bool getPostProcessBloom(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.postProcessBloom; }
    Bool getPostProcessHDR(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.postProcessHDR; }
    Bool getPostProcessFXAA(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.postProcessFXAA; }
    Bool getAirUnitGroundIndicator(Bool def) const { return g_optionPrefs.passthrough ? def : g_optionPrefs.airUnitGroundIndicator; }

    WW3D::MultiSampleModeEnum getAntiAliasing() const {
        return static_cast<WW3D::MultiSampleModeEnum>(
            g_optionPrefs.passthrough ? TheGlobalData->m_antiAliasLevel : g_optionPrefs.antiAliasing);
    }
    TextureFilterClass::TextureFilterMode getTextureFilterMode() const {
        return static_cast<TextureFilterClass::TextureFilterMode>(
            g_optionPrefs.passthrough ? TheGlobalData->m_textureFilteringMode : g_optionPrefs.textureFilterMode);
    }
    TextureFilterClass::AnisotropicFilterMode getTextureAnisotropyLevel() const {
        return static_cast<TextureFilterClass::AnisotropicFilterMode>(
            g_optionPrefs.passthrough ? TheGlobalData->m_textureAnisotropyLevel : g_optionPrefs.textureAnisotropyLevel);
    }
    Real getGammaValue() const {
        return g_optionPrefs.passthrough ? 50.0f : g_optionPrefs.gammaValue;
    }
    void getResolution(Int* xres, Int* yres) const {
        if (g_optionPrefs.passthrough) {
            *xres = TheGlobalData->m_xResolution;
            *yres = TheGlobalData->m_yResolution;
        } else {
            *xres = g_optionPrefs.xResolution;
            *yres = g_optionPrefs.yResolution;
        }
    }
};