# Translation Packet `PKT-442`: `GlobalData`

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-442`
- **Work Package:** `WP-7.1` (Phase 7)
- **Complexity & Assigned Tier:** Class `C2` -> `tier:1b-fast` *(360-entry `FieldParse` settings container, override stack, `OptionPreferences` clamping, and `exe`+`.scb` CRC computation)*
- **Environment Access:** `env:ephemeral-web` *(seeded with 1,000-seed C++ Release oracle `docs/golden_vectors/PKT-442.json` & compiling Rust skeleton)*
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output Files:**
  - `crates/zh_logic/src/gamelogic/global_data.rs` *(implement all `todo!("PKT-442")` items without changing public signatures)*
  - `crates/zh_logic/tests/golden_pkt442_global_data.rs` *(remove both `#[ignore]` attributes once implemented)*
  - `docs/PORT_MAP.md` *(regenerate via `python3 scripts/port_map.py --write`)*
- **Mandatory V&V Gates:** `V1, V2-Golden`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test -p zh_logic --test golden_pkt442_global_data && python3 scripts/port_map.py --write && python3 scripts/port_map.py --check && cargo test --workspace && cargo check --workspace --all-targets
  ```

## Port Contract (read first)

1. **Release Macro Configuration (Zero Hour `1.04` / `GeneralsX` Release):**
   - `PRESERVE_RETAIL_SCRIPTED_CAMERA=1` (`camera_height` IS present and parsed via `"CameraHeight"`).
   - `ALLOW_MONEY_PER_MINUTE_FOR_PLAYER=0`, `ENABLE_CONFIGURABLE_SHROUD=1`, `RETAIL_COMPATIBLE_CRC=0`, `RETAIL_COMPATIBLE_XFER_SAVE=1`.
   - `RTS_DEBUG`, `_ALLOW_DEBUG_CHEATS_IN_RELEASE`, `ALLOW_SURRENDER`, `DUMP_PERF_STATS`, and `ALLOW_DEMORALIZE` are **NOT** defined in Release builds — ignore `#ifdef RTS_DEBUG` / `#ifdef ALLOW_SURRENDER` / `#ifdef DUMP_PERF_STATS` INI fields and struct members (`WeaponBonusConditionType` uses `DemoralizedObsolete` = `"DEMORALIZED_OBSOLETE"` at index `7`).

2. **Exact C++ Field & `FieldParse` Quirks (Bit-Exact Parity Required):**
   - **`DrawSkyBox`:** `draw_sky_box: Real` (default `0.0`), parsed via `INI::parseBool`! In C++, `parseBool` overwrites byte 0 of little-endian `f32`: `f32::from_bits((self.draw_sky_box.to_bits() & 0xffff_ff00) | (b as u32))`.
   - **`DebugAI`:** `debug_ai: Int` (`AIDebugOptions`, default `0`), parsed via `INI::parseBool`! Overwrites byte 0: `(self.debug_ai & !0xff) | (b as i32)`.
   - **`FirewallPortAllocationDelta`:** `firewall_port_allocation_delta: Short` (`i16`, default `0`), parsed via `INI::parseInt` (`scan_int` truncated to `Short`).
   - **`FirewallBehavior`, `FirewallPortOverride`, and all 9 `Network*` fields** (`NetworkFPSHistoryLength`..`NetworkDisconnectScreenNotifyTime`): fields are `UnsignedInt`, parsed via `INI::parseInt` (`scan_int(...) as UnsignedInt`).
   - **`SkirmishTickRate` and `TickRate`:** Both INI keys map to `skirmish_tick_rate` (default `LOGICFRAMES_PER_SECOND = 30`).
   - **Excluded entries:** `HealthBonus_Regular` and `DoubleClickTimeMS` are commented out in `s_GlobalDataFieldParseTable` — do NOT add them to the INI parse table. `double_click_time_ms` defaults to `500` (`GetDoubleClickTime()`).
   - **`TerrainAmbientRGB`, `TerrainDiffuseRGB`, `TerrainLightPos`, `InGame`, `Latency*`, `PacketLoss`:** Commented out in `s_GlobalDataFieldParseTable` — do NOT add them.
   - **`DefaultStartingCash`:** Calls `Money::set_starting_cash(IniReader::scan_unsigned_int(...))` (matching `Money::parseMoneyAmount`).
   - **`WeaponBonus`:** Calls `WeaponBonusSet::parse_weapon_bonus_set(&mut IniReader)` (`scan_index_list` over `THE_WEAPON_BONUS_NAMES` and `THE_WEAPON_BONUS_FIELD_NAMES`, then `scan_percent_to_real` into `entries[cond].fields[field]`). `WeaponBonus::clear()` initializes all 5 fields to `1.0_f32`. `append_bonuses` accumulates `bonus.fields[f] += entry.fields[f] - 1.0_f32` in loop `c = 0..WEAPONBONUSCONDITION_COUNT` whenever `(flags & (1u32 << c)) != 0`.
   - **`StandardPublicBone`:** Uses `parse_ascii_string_vector_append`.
   - **`TimeOfDay` & `Weather`:** Note `GlobalData::new()` initializes `time_of_day = TimeOfDay::Afternoon` (`2`) and immediately calls `set_time_of_day(TimeOfDay::Afternoon)`, setting `terrain_ambient`/`terrain_diffuse`/`terrain_light_pos` from `terrain_lighting[Afternoon][0..3]`. Also note C++ `GlobalData::GlobalData()` initializes `infantry_light_scale[1..=4]` to `1.5_f32` (index `0` = `TIME_OF_DAY_INVALID` is untouched; tests hash `1..=4`).

3. **`set_time_of_day`, `apply_option_preferences`, `parse_custom_definition`, & `GlobalDataStore`:**
   - `set_time_of_day(&mut self, tod_raw: Int) -> Bool`: If `tod_raw >= TIME_OF_DAY_COUNT || tod_raw < TIME_OF_DAY_FIRST`, return `FALSE` without mutating state; otherwise set `time_of_day`, copy `terrain_lighting[tod][0..3]` into `terrain_ambient`, `terrain_diffuse`, `terrain_light_pos`, and return `TRUE`.
   - `apply_option_preferences(&mut self, prefs: &OptionPreferencesOverrides)`: Copy all option fields per `GlobalData.cpp:1273-1361`:
     - If `prefs.max_camera_height` is `Some(v)`: `!v.is_finite() || v < 100.0 => 100.0`, `v > 1000.0 => 1000.0`, else `v`.
     - If `prefs.terrain_draw_distance_scale` is `Some(v)`: `!v.is_finite() || v < 1.0 => 1.0`, `v > 2.0 => 2.0`, else `v`.
     - `gamma_value`: `let val = prefs.gamma_value as Int;` (C++ `Int val = optionPref.getGammaValue();` truncates toward zero). If `val < 50`: `if val <= 0 { 0.6_f32 } else { 1.0_f32 - 0.4_f32 * ((50 - val) as Real) / 50.0_f32 }`. Else if `val > 50`: `1.0_f32 + 1.0_f32 * ((val - 50) as Real) / 50.0_f32`. Else (`val == 50`): leave `self.display_gamma` unchanged!
   - `parse_custom_definition(&mut self, has_fullviewport_dat: Bool)`: if `has_fullviewport_dat != FALSE`, set `self.viewport_height_scale = 1.0`.
   - `GlobalDataStore`: maintains a base + override stack (`Vec<GlobalData>`, never empty). On `parse_game_data_definition(ini, prefs)`: if `ini.get_load_type() == IniLoadType::CreateOverrides` push a clone (`new_override()`) before parsing INI into the active top entry; `Overwrite` and `Multifile` mutate the current top entry in place; after `parse_ini_block(ini)?`, if `Some(p) = prefs` call `apply_option_preferences(p)`. `reset()` pops all overrides back to the original base entry (depth `0`).

4. **`compute_exe_crc_with_reader` / `compute_exe_crc`:**
   - Hashes executable stream in `65536`-byte chunks via `zh_common::crc::Crc`: increments `read_count` per positive read and breaks `if read_count > 1000` (**1,001** chunks max = `65,601,536` bytes).
   - If `version_number` is `Some(v)`, hashes `v.to_le_bytes()`.
   - Then hashes `Data\Scripts\SkirmishScripts.scb` in `65536`-byte chunks (if present), then `Data\Scripts\MultiplayerScripts.scb` in `65536`-byte chunks (if present), stores result in `self.exe_crc`, and returns it.

5. **14 Inline Getter Methods (`GlobalData.h:181-195`):** Preserve exact boolean conjuncts/disjuncts (`is_enable_post_processing` requires `enable_post_processing != 0 && (post_process_bloom != 0 || post_process_hdr != 0 || post_process_fxaa != 0)`, `is_water_reflections` returns `water_reflections`, `is_air_unit_ground_indicator` returns `air_unit_ground_indicator`, etc.).

6. **Determinism & 3-Attempt Escalation Rule:** Zero external crates, no `unsafe`, no `HashMap`/`HashSet`/`BinaryHeap`, no `Rc`/`RefCell`, no std float math. If 3 consecutive fix attempts fail to shrink failing vectors, isolate the minimal failing seed or escalate (`tier:3-architect`).

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (27):**
  - `Common/AddonCompat.h`
  - `Common/AsciiString.h`
  - `Common/FileSystem.h`
  - `Common/GameAudio.h`
  - `Common/GameCommon.h`
  - `Common/GameMemory.h`
  - `Common/GameType.h`
  - `Common/GlobalData.h`
  - `Common/INI.h`
  - `Common/Money.h`
  - `Common/OptionPreferences.h`
  - `Common/Registry.h`
  - `Common/STLTypedefs.h`
  - `Common/SubsystemInterface.h`
  - `Common/crc.h`
  - `Common/file.h`
  - `Common/version.h`
  - `GameClient/Color.h`
  - `GameClient/Display.h`
  - `GameClient/TerrainVisual.h`
  - `GameLogic/AI.h`
  - `GameLogic/Module/BodyModule.h`
  - `GameLogic/Weapon.h`
  - `GameNetwork/FirewallHelper.h`
  - `PreRTS.h`
  - `WW3D2/texturefilter.h`
  - `WW3D2/ww3d.h`
- **INI `FieldParse` Table Entries (405):**
  - `"Windowed" => INI::parseBool`
  - `"CheckForUpdates" => INI::parseBool`
  - `"XResolution" => INI::parseInt`
  - `"YResolution" => INI::parseInt`
  - `"MapName" => INI::parseAsciiString`
  - `"MoveHintName" => INI::parseAsciiString`
  - `"UseTrees" => INI::parseBool`
  - `"UseFPSLimit" => INI::parseBool`
  - `"DumpAssetUsage" => INI::parseBool`
  - `"FramesPerSecondLimit" => INI::parseInt`
  - `"SkirmishTickRate" => INI::parseInt`
  - `"TickRate" => INI::parseInt`
  - `"ChipsetType" => INI::parseInt`
  - `"MaxShellScreens" => INI::parseInt`
  - `"UseCloudMap" => INI::parseBool`
  - `"UseLightMap" => INI::parseBool`
  - `"BilinearTerrainTex" => INI::parseBool`
  - `"TrilinearTerrainTex" => INI::parseBool`
  - `"MultiPassTerrain" => INI::parseBool`
  - `"AdjustCliffTextures" => INI::parseBool`
  - `"Use3WayTerrainBlends" => INI::parseInt`
  - `"StretchTerrain" => INI::parseBool`
  - `"UseHalfHeightMap" => INI::parseBool`
  - `"DrawEntireTerrain" => INI::parseBool`
  - `"TerrainLOD" => INI::parseIndexList`
  - `"TerrainLODTargetTimeMS" => INI::parseInt`
  - `"RightMouseAlwaysScrolls" => INI::parseBool`
  - `"UseWaterPlane" => INI::parseBool`
  - `"UseCloudPlane" => INI::parseBool`
  - `"DownwindAngle" => INI::parseReal`
  - `"UseShadowVolumes" => INI::parseBool`
  - `"UseShadowVolumesForSkins" => INI::parseBool`
  - `"ShadowsForProjectiles" => INI::parseBool`
  - `"ShadowsForProps" => INI::parseBool`
  - `"AdvancedShadows" => INI::parseBool`
  - `"AdvancedLighting" => INI::parseBool`
  - `"ExtendedShadowFrustumCulling" => INI::parseBool`
  - `"NaturalSunElevation" => INI::parseBool`
  - `"BridgeShadows" => INI::parseBool`
  - `"WaterReflections" => INI::parseBool`
  - `"ExplosionDynamicLights" => INI::parseBool`
  - `"VehicleHeadlights" => INI::parseBool`
  - `"EnablePostProcessing" => INI::parseBool`
  - `"PostProcessBloom" => INI::parseBool`
  - `"PostProcessHDR" => INI::parseBool`
  - `"PostProcessFXAA" => INI::parseBool`
  - `"AirUnitGroundIndicator" => INI::parseBool`
  - `"UseShadowDecals" => INI::parseBool`
  - `"TextureReductionFactor" => INI::parseInt`
  - `"UseBehindBuildingMarker" => INI::parseBool`
  - `"WaterPositionX" => INI::parseReal`
  - `"WaterPositionY" => INI::parseReal`
  - `"WaterPositionZ" => INI::parseReal`
  - `"WaterExtentX" => INI::parseReal`
  - `"WaterExtentY" => INI::parseReal`
  - `"WaterType" => INI::parseInt`
  - `"FeatherWater" => INI::parseInt`
  - `"ShowSoftWaterEdge" => INI::parseBool`
  - `"VertexWaterAvailableMaps1" => INI::parseAsciiString`
  - `"VertexWaterHeightClampLow1" => INI::parseReal`
  - `"VertexWaterHeightClampHi1" => INI::parseReal`
  - `"VertexWaterAngle1" => INI::parseAngleReal`
  - `"VertexWaterXPosition1" => INI::parseReal`
  - `"VertexWaterYPosition1" => INI::parseReal`
  - `"VertexWaterZPosition1" => INI::parseReal`
  - `"VertexWaterXGridCells1" => INI::parseInt`
  - `"VertexWaterYGridCells1" => INI::parseInt`
  - `"VertexWaterGridSize1" => INI::parseReal`
  - `"VertexWaterAttenuationA1" => INI::parseReal`
  - `"VertexWaterAttenuationB1" => INI::parseReal`
  - `"VertexWaterAttenuationC1" => INI::parseReal`
  - `"VertexWaterAttenuationRange1" => INI::parseReal`
  - `"VertexWaterAvailableMaps2" => INI::parseAsciiString`
  - `"VertexWaterHeightClampLow2" => INI::parseReal`
  - `"VertexWaterHeightClampHi2" => INI::parseReal`
  - `"VertexWaterAngle2" => INI::parseAngleReal`
  - `"VertexWaterXPosition2" => INI::parseReal`
  - `"VertexWaterYPosition2" => INI::parseReal`
  - `"VertexWaterZPosition2" => INI::parseReal`
  - `"VertexWaterXGridCells2" => INI::parseInt`
  - `"VertexWaterYGridCells2" => INI::parseInt`
  - `"VertexWaterGridSize2" => INI::parseReal`
  - `"VertexWaterAttenuationA2" => INI::parseReal`
  - `"VertexWaterAttenuationB2" => INI::parseReal`
  - `"VertexWaterAttenuationC2" => INI::parseReal`
  - `"VertexWaterAttenuationRange2" => INI::parseReal`
  - `"VertexWaterAvailableMaps3" => INI::parseAsciiString`
  - `"VertexWaterHeightClampLow3" => INI::parseReal`
  - `"VertexWaterHeightClampHi3" => INI::parseReal`
  - `"VertexWaterAngle3" => INI::parseAngleReal`
  - `"VertexWaterXPosition3" => INI::parseReal`
  - `"VertexWaterYPosition3" => INI::parseReal`
  - `"VertexWaterZPosition3" => INI::parseReal`
  - `"VertexWaterXGridCells3" => INI::parseInt`
  - `"VertexWaterYGridCells3" => INI::parseInt`
  - `"VertexWaterGridSize3" => INI::parseReal`
  - `"VertexWaterAttenuationA3" => INI::parseReal`
  - `"VertexWaterAttenuationB3" => INI::parseReal`
  - `"VertexWaterAttenuationC3" => INI::parseReal`
  - `"VertexWaterAttenuationRange3" => INI::parseReal`
  - `"VertexWaterAvailableMaps4" => INI::parseAsciiString`
  - `"VertexWaterHeightClampLow4" => INI::parseReal`
  - `"VertexWaterHeightClampHi4" => INI::parseReal`
  - `"VertexWaterAngle4" => INI::parseAngleReal`
  - `"VertexWaterXPosition4" => INI::parseReal`
  - `"VertexWaterYPosition4" => INI::parseReal`
  - `"VertexWaterZPosition4" => INI::parseReal`
  - `"VertexWaterXGridCells4" => INI::parseInt`
  - `"VertexWaterYGridCells4" => INI::parseInt`
  - `"VertexWaterGridSize4" => INI::parseReal`
  - `"VertexWaterAttenuationA4" => INI::parseReal`
  - `"VertexWaterAttenuationB4" => INI::parseReal`
  - `"VertexWaterAttenuationC4" => INI::parseReal`
  - `"VertexWaterAttenuationRange4" => INI::parseReal`
  - `"SkyBoxPositionZ" => INI::parseReal`
  - `"SkyBoxScale" => INI::parseReal`
  - `"DrawSkyBox" => INI::parseBool`
  - `"ViewportHeightScale" => INI::parseReal`
  - `"CameraPitch" => INI::parseReal`
  - `"CameraYaw" => INI::parseReal`
  - `"CameraHeight" => INI::parseReal`
  - `"MaxCameraHeight" => INI::parseReal`
  - `"MinCameraHeight" => INI::parseReal`
  - `"TerrainDrawDistanceScale" => INI::parseReal`
  - `"TerrainHeightAtEdgeOfMap" => INI::parseReal`
  - `"UnitDamagedThreshold" => INI::parseReal`
  - `"UnitReallyDamagedThreshold" => INI::parseReal`
  - `"GroundStiffness" => INI::parseReal`
  - `"StructureStiffness" => INI::parseReal`
  - `"Gravity" => INI::parseAccelerationReal`
  - `"StealthFriendlyOpacity" => INI::parsePercentToReal`
  - `"DefaultOcclusionDelay" => INI::parseDurationUnsignedInt`
  - `"PartitionCellSize" => INI::parseReal`
  - `"AmmoPipScaleFactor" => INI::parseReal`
  - `"ContainerPipScaleFactor" => INI::parseReal`
  - `"AmmoPipWorldOffset" => INI::parseCoord3D`
  - `"ContainerPipWorldOffset" => INI::parseCoord3D`
  - `"AmmoPipScreenOffset" => INI::parseCoord2D`
  - `"ContainerPipScreenOffset" => INI::parseCoord2D`
  - `"HistoricDamageLimit" => INI::parseDurationUnsignedInt`
  - `"MaxTerrainTracks" => INI::parseInt`
  - `"TimeOfDay" => INI::parseIndexList`
  - `"Weather" => INI::parseIndexList`
  - `"MakeTrackMarks" => INI::parseBool`
  - `"HideGarrisonFlags" => INI::parseBool`
  - `"ForceModelsToFollowTimeOfDay" => INI::parseBool`
  - `"ForceModelsToFollowWeather" => INI::parseBool`
  - `"LevelGainAnimationName" => INI::parseAsciiString`
  - `"LevelGainAnimationTime" => INI::parseReal`
  - `"LevelGainAnimationZRise" => INI::parseReal`
  - `"GetHealedAnimationName" => INI::parseAsciiString`
  - `"GetHealedAnimationTime" => INI::parseReal`
  - `"GetHealedAnimationZRise" => INI::parseReal`
  - `"TerrainLightingMorningAmbient" => INI::parseRGBColor`
  - `"TerrainLightingMorningDiffuse" => INI::parseRGBColor`
  - `"TerrainLightingMorningLightPos" => INI::parseCoord3D`
  - `"TerrainLightingAfternoonAmbient" => INI::parseRGBColor`
  - `"TerrainLightingAfternoonDiffuse" => INI::parseRGBColor`
  - `"TerrainLightingAfternoonLightPos" => INI::parseCoord3D`
  - `"TerrainLightingEveningAmbient" => INI::parseRGBColor`
  - `"TerrainLightingEveningDiffuse" => INI::parseRGBColor`
  - `"TerrainLightingEveningLightPos" => INI::parseCoord3D`
  - `"TerrainLightingNightAmbient" => INI::parseRGBColor`
  - `"TerrainLightingNightDiffuse" => INI::parseRGBColor`
  - `"TerrainLightingNightLightPos" => INI::parseCoord3D`
  - `"TerrainObjectsLightingMorningAmbient" => INI::parseRGBColor`
  - `"TerrainObjectsLightingMorningDiffuse" => INI::parseRGBColor`
  - `"TerrainObjectsLightingMorningLightPos" => INI::parseCoord3D`
  - `"TerrainObjectsLightingAfternoonAmbient" => INI::parseRGBColor`
  - `"TerrainObjectsLightingAfternoonDiffuse" => INI::parseRGBColor`
  - `"TerrainObjectsLightingAfternoonLightPos" => INI::parseCoord3D`
  - `"TerrainObjectsLightingEveningAmbient" => INI::parseRGBColor`
  - `"TerrainObjectsLightingEveningDiffuse" => INI::parseRGBColor`
  - `"TerrainObjectsLightingEveningLightPos" => INI::parseCoord3D`
  - `"TerrainObjectsLightingNightAmbient" => INI::parseRGBColor`
  - `"TerrainObjectsLightingNightDiffuse" => INI::parseRGBColor`
  - `"TerrainObjectsLightingNightLightPos" => INI::parseCoord3D`
  - `"TerrainLightingMorningAmbient2" => INI::parseRGBColor`
  - `"TerrainLightingMorningDiffuse2" => INI::parseRGBColor`
  - `"TerrainLightingMorningLightPos2" => INI::parseCoord3D`
  - `"TerrainLightingAfternoonAmbient2" => INI::parseRGBColor`
  - `"TerrainLightingAfternoonDiffuse2" => INI::parseRGBColor`
  - `"TerrainLightingAfternoonLightPos2" => INI::parseCoord3D`
  - `"TerrainLightingEveningAmbient2" => INI::parseRGBColor`
  - `"TerrainLightingEveningDiffuse2" => INI::parseRGBColor`
  - `"TerrainLightingEveningLightPos2" => INI::parseCoord3D`
  - `"TerrainLightingNightAmbient2" => INI::parseRGBColor`
  - `"TerrainLightingNightDiffuse2" => INI::parseRGBColor`
  - `"TerrainLightingNightLightPos2" => INI::parseCoord3D`
  - `"TerrainObjectsLightingMorningAmbient2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingMorningDiffuse2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingMorningLightPos2" => INI::parseCoord3D`
  - `"TerrainObjectsLightingAfternoonAmbient2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingAfternoonDiffuse2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingAfternoonLightPos2" => INI::parseCoord3D`
  - `"TerrainObjectsLightingEveningAmbient2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingEveningDiffuse2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingEveningLightPos2" => INI::parseCoord3D`
  - `"TerrainObjectsLightingNightAmbient2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingNightDiffuse2" => INI::parseRGBColor`
  - `"TerrainObjectsLightingNightLightPos2" => INI::parseCoord3D`
  - `"TerrainLightingMorningAmbient3" => INI::parseRGBColor`
  - `"TerrainLightingMorningDiffuse3" => INI::parseRGBColor`
  - `"TerrainLightingMorningLightPos3" => INI::parseCoord3D`
  - `"TerrainLightingAfternoonAmbient3" => INI::parseRGBColor`
  - `"TerrainLightingAfternoonDiffuse3" => INI::parseRGBColor`
  - `"TerrainLightingAfternoonLightPos3" => INI::parseCoord3D`
  - `"TerrainLightingEveningAmbient3" => INI::parseRGBColor`
  - `"TerrainLightingEveningDiffuse3" => INI::parseRGBColor`
  - `"TerrainLightingEveningLightPos3" => INI::parseCoord3D`
  - `"TerrainLightingNightAmbient3" => INI::parseRGBColor`
  - `"TerrainLightingNightDiffuse3" => INI::parseRGBColor`
  - `"TerrainLightingNightLightPos3" => INI::parseCoord3D`
  - `"TerrainObjectsLightingMorningAmbient3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingMorningDiffuse3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingMorningLightPos3" => INI::parseCoord3D`
  - `"TerrainObjectsLightingAfternoonAmbient3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingAfternoonDiffuse3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingAfternoonLightPos3" => INI::parseCoord3D`
  - `"TerrainObjectsLightingEveningAmbient3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingEveningDiffuse3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingEveningLightPos3" => INI::parseCoord3D`
  - `"TerrainObjectsLightingNightAmbient3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingNightDiffuse3" => INI::parseRGBColor`
  - `"TerrainObjectsLightingNightLightPos3" => INI::parseCoord3D`
  - `"NumberGlobalLights" => INI::parseInt`
  - `"InfantryLightMorningScale" => INI::parseReal`
  - `"InfantryLightAfternoonScale" => INI::parseReal`
  - `"InfantryLightEveningScale" => INI::parseReal`
  - `"InfantryLightNightScale" => INI::parseReal`
  - `"MaxTranslucentObjects" => INI::parseInt`
  - `"OccludedColorLuminanceScale" => INI::parseReal`
  - `"TerrainAmbientRGB" => INI::parseRGBColor`
  - `"TerrainDiffuseRGB" => INI::parseRGBColor`
  - `"TerrainLightPos" => INI::parseCoord3D`
  - `"MaxRoadSegments" => INI::parseInt`
  - `"MaxRoadVertex" => INI::parseInt`
  - `"MaxRoadIndex" => INI::parseInt`
  - `"MaxRoadTypes" => INI::parseInt`
  - `"ValuePerSupplyBox" => INI::parseInt`
  - `"AudioOn" => INI::parseBool`
  - `"MusicOn" => INI::parseBool`
  - `"SoundsOn" => INI::parseBool`
  - `"Sounds3DOn" => INI::parseBool`
  - `"SpeechOn" => INI::parseBool`
  - `"VideoOn" => INI::parseBool`
  - `"DisableCameraMovements" => INI::parseBool`
  - `"InGame" => INI::parseBool`
  - `"DebugAI" => INI::parseBool`
  - `"DebugAIObstacles" => INI::parseBool`
  - `"ShowClientPhysics" => INI::parseBool`
  - `"ShowTerrainNormals" => INI::parseBool`
  - `"ShowObjectHealth" => INI::parseBool`
  - `"ParticleScale" => INI::parseReal`
  - `"AutoFireParticleSmallPrefix" => INI::parseAsciiString`
  - `"AutoFireParticleSmallSystem" => INI::parseAsciiString`
  - `"AutoFireParticleSmallMax" => INI::parseInt`
  - `"AutoFireParticleMediumPrefix" => INI::parseAsciiString`
  - `"AutoFireParticleMediumSystem" => INI::parseAsciiString`
  - `"AutoFireParticleMediumMax" => INI::parseInt`
  - `"AutoFireParticleLargePrefix" => INI::parseAsciiString`
  - `"AutoFireParticleLargeSystem" => INI::parseAsciiString`
  - `"AutoFireParticleLargeMax" => INI::parseInt`
  - `"AutoSmokeParticleSmallPrefix" => INI::parseAsciiString`
  - `"AutoSmokeParticleSmallSystem" => INI::parseAsciiString`
  - `"AutoSmokeParticleSmallMax" => INI::parseInt`
  - `"AutoSmokeParticleMediumPrefix" => INI::parseAsciiString`
  - `"AutoSmokeParticleMediumSystem" => INI::parseAsciiString`
  - `"AutoSmokeParticleMediumMax" => INI::parseInt`
  - `"AutoSmokeParticleLargePrefix" => INI::parseAsciiString`
  - `"AutoSmokeParticleLargeSystem" => INI::parseAsciiString`
  - `"AutoSmokeParticleLargeMax" => INI::parseInt`
  - `"AutoAflameParticlePrefix" => INI::parseAsciiString`
  - `"AutoAflameParticleSystem" => INI::parseAsciiString`
  - `"AutoAflameParticleMax" => INI::parseInt`
  - `"LatencyAverage" => INI::parseInt`
  - `"LatencyAmplitude" => INI::parseInt`
  - `"LatencyPeriod" => INI::parseInt`
  - `"LatencyNoise" => INI::parseInt`
  - `"PacketLoss" => INI::parseInt`
  - `"BuildSpeed" => INI::parseReal`
  - `"MinDistFromEdgeOfMapForBuild" => INI::parseReal`
  - `"SupplyBuildBorder" => INI::parseReal`
  - `"AllowedHeightVariationForBuilding" => INI::parseReal`
  - `"MinLowEnergyProductionSpeed" => INI::parseReal`
  - `"MaxLowEnergyProductionSpeed" => INI::parseReal`
  - `"LowEnergyPenaltyModifier" => INI::parseReal`
  - `"MultipleFactory" => INI::parseReal`
  - `"RefundPercent" => INI::parsePercentToReal`
  - `"CommandCenterHealRange" => INI::parseReal`
  - `"CommandCenterHealAmount" => INI::parseReal`
  - `"StandardMinefieldDensity" => INI::parseReal`
  - `"StandardMinefieldDistance" => INI::parseReal`
  - `"MaxLineBuildObjects" => INI::parseInt`
  - `"MaxTunnelCapacity" => INI::parseInt`
  - `"MaxParticleCount" => INI::parseInt`
  - `"MaxFieldParticleCount" => INI::parseInt`
  - `"HorizontalScrollSpeedFactor" => INI::parseReal`
  - `"VerticalScrollSpeedFactor" => INI::parseReal`
  - `"ScrollAmountCutoff" => INI::parseReal`
  - `"CameraAdjustSpeed" => INI::parseReal`
  - `"EnforceMaxCameraHeight" => INI::parseBool`
  - `"KeyboardScrollSpeedFactor" => INI::parseReal`
  - `"MouseScrollSpeedFactor" => INI::parseReal`
  - `"KeyboardDefaultScrollSpeedFactor" => INI::parseReal`
  - `"KeyboardCameraRotateSpeed" => INI::parseReal`
  - `"MovementPenaltyDamageState" => INI::parseIndexList`
  - `"HealthBonus_Regular" => INI::parsePercentToReal`
  - `"HealthBonus_Veteran" => INI::parsePercentToReal`
  - `"HealthBonus_Elite" => INI::parsePercentToReal`
  - `"HealthBonus_Heroic" => INI::parsePercentToReal`
  - `"HumanSoloPlayerHealthBonus_Easy" => INI::parsePercentToReal`
  - `"HumanSoloPlayerHealthBonus_Normal" => INI::parsePercentToReal`
  - `"HumanSoloPlayerHealthBonus_Hard" => INI::parsePercentToReal`
  - `"AISoloPlayerHealthBonus_Easy" => INI::parsePercentToReal`
  - `"AISoloPlayerHealthBonus_Normal" => INI::parsePercentToReal`
  - `"AISoloPlayerHealthBonus_Hard" => INI::parsePercentToReal`
  - `"WeaponBonus" => WeaponBonusSet::parseWeaponBonusSetPtr`
  - `"DefaultStructureRubbleHeight" => INI::parseReal`
  - `"FixedSeed" => INI::parseInt`
  - `"ShellMapName" => INI::parseAsciiString`
  - `"ShellMapOn" => INI::parseBool`
  - `"PlayIntro" => INI::parseBool`
  - `"FirewallBehavior" => INI::parseInt`
  - `"FirewallPortOverride" => INI::parseInt`
  - `"FirewallPortAllocationDelta" => INI::parseInt`
  - `"GroupSelectMinSelectSize" => INI::parseInt`
  - `"GroupSelectVolumeBase" => INI::parseReal`
  - `"GroupSelectVolumeIncrement" => INI::parseReal`
  - `"MaxUnitSelectSounds" => INI::parseInt`
  - `"SelectionFlashSaturationFactor" => INI::parseReal`
  - `"SelectionFlashHouseColor" => INI::parseBool`
  - `"CameraAudibleRadius" => INI::parseReal`
  - `"GroupMoveClickToGatherAreaFactor" => INI::parseReal`
  - `"AllowMoneyPerMinuteForPlayer" => INI::parseBool`
  - `"ShakeSubtleIntensity" => INI::parseReal`
  - `"ShakeNormalIntensity" => INI::parseReal`
  - `"ShakeStrongIntensity" => INI::parseReal`
  - `"ShakeSevereIntensity" => INI::parseReal`
  - `"ShakeCineExtremeIntensity" => INI::parseReal`
  - `"ShakeCineInsaneIntensity" => INI::parseReal`
  - `"MaxShakeIntensity" => INI::parseReal`
  - `"MaxShakeRange" => INI::parseReal`
  - `"SellPercentage" => INI::parsePercentToReal`
  - `"BaseRegenHealthPercentPerSecond" => INI::parsePercentToReal`
  - `"BaseRegenDelay" => INI::parseDurationUnsignedInt`
  - `"PrisonBountyMultiplier" => INI::parseReal`
  - `"PrisonBountyTextColor" => INI::parseColorInt`
  - `"SpecialPowerViewObject" => INI::parseAsciiString`
  - `"ObjectPlacementOpacity" => INI::parseReal`
  - `"ObjectPlacementShadows" => INI::parseBool`
  - `"StandardPublicBone" => INI::parseAsciiStringVectorAppend`
  - `"ShowMetrics" => INI::parseBool`
  - `"DefaultStartingCash" => Money::parseMoneyAmount`
  - `"DoubleClickTimeMS" => INI::parseUnsignedInt`
  - `"ShroudColor" => INI::parseRGBColor`
  - `"ClearAlpha" => INI::parseUnsignedByte`
  - `"FogAlpha" => INI::parseUnsignedByte`
  - `"ShroudAlpha" => INI::parseUnsignedByte`
  - `"HotKeyTextColor" => INI::parseColorInt`
  - `"PowerBarBase" => INI::parseInt`
  - `"PowerBarIntervals" => INI::parseReal`
  - `"PowerBarYellowRange" => INI::parseInt`
  - `"UnlookPersistDuration" => INI::parseDurationUnsignedInt`
  - `"NetworkFPSHistoryLength" => INI::parseInt`
  - `"NetworkLatencyHistoryLength" => INI::parseInt`
  - `"NetworkRunAheadMetricsTime" => INI::parseInt`
  - `"NetworkCushionHistoryLength" => INI::parseInt`
  - `"NetworkRunAheadSlack" => INI::parseInt`
  - `"NetworkKeepAliveDelay" => INI::parseInt`
  - `"NetworkDisconnectTime" => INI::parseInt`
  - `"NetworkPlayerTimeoutTime" => INI::parseInt`
  - `"NetworkDisconnectScreenNotifyTime" => INI::parseInt`
  - `"PlayStats" => INI::parseInt`
  - `"DisableCameraFade" => INI::parseBool`
  - `"DisableScriptedInputDisabling" => INI::parseBool`
  - `"DisableMilitaryCaption" => INI::parseBool`
  - `"BenchmarkTimer" => INI::parseInt`
  - `"CheckMemoryLeaks" => INI::parseBool`
  - `"Wireframe" => INI::parseBool`
  - `"StateMachineDebug" => INI::parseBool`
  - `"UseCameraConstraints" => INI::parseBool`
  - `"ShroudOn" => INI::parseBool`
  - `"FogOfWarOn" => INI::parseBool`
  - `"ShowCollisionExtents" => INI::parseBool`
  - `"ShowAudioLocations" => INI::parseBool`
  - `"DebugProjectileTileWidth" => INI::parseReal`
  - `"DebugProjectileTileDuration" => INI::parseInt`
  - `"DebugProjectileTileColor" => INI::parseRGBColor`
  - `"DebugVisibilityTileCount" => INI::parseInt`
  - `"DebugVisibilityTileWidth" => INI::parseReal`
  - `"DebugVisibilityTileDuration" => INI::parseInt`
  - `"DebugVisibilityTileTargettableColor" => INI::parseRGBColor`
  - `"DebugVisibilityTileDeshroudColor" => INI::parseRGBColor`
  - `"DebugVisibilityTileGapColor" => INI::parseRGBColor`
  - `"DebugThreatMapTileDuration" => INI::parseInt`
  - `"MaxDebugThreatMapValue" => INI::parseUnsignedInt`
  - `"DebugCashValueMapTileDuration" => INI::parseInt`
  - `"MaxDebugCashValueMapValue" => INI::parseUnsignedInt`
  - `"VTune" => INI::parseBool`
  - `"SaveStats" => INI::parseBool`
  - `"UseLocalMOTD" => INI::parseBool`
  - `"BaseStatsDir" => INI::parseAsciiString`
  - `"LocalMOTDPath" => INI::parseAsciiString`
  - `"ExtraLogging" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Calls:** None in module.
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if (defined(_MSC_VER) && _MSC_VER < 1300) && RETAIL_COMPATIBLE_CRC`

---

## Compact Worker Cheat Sheet (Embedded in All Translation Packets)

### 1. Primitive Type Mapping
| C++ Type | Rust Type | Size | Notes |
|---|---|---:|---|
| `Int` | `i32` | 4 | Two's-complement 32-bit signed |
| `UnsignedInt` | `u32` | 4 | 32-bit unsigned |
| `Short` | `i16` | 2 | 16-bit signed |
| `UnsignedShort` | `u16` | 2 | 16-bit unsigned |
| `Byte` / `char` | `i8` | 1 | Signed 8-bit (`char` is signed on x86/macOS) |
| `UnsignedByte` | `u8` | 1 | Unsigned 8-bit |
| `Bool` | `u8` (`Bool`) | 1 | `FALSE = 0`, `TRUE = 1` (1 byte in `Xfer`!) |
| `Real` | `f32` | 4 | IEEE-754 binary32, round-to-nearest-even, no FMA |
| `ObjectID` / `TeamID` / `DrawableID` | `ObjectId(pub i32)` etc. | 4 | `INVALID_ID = 0` |
| `NameKeyType` | `NameKeyType(pub i32)` | 4 | `NAMEKEY_INVALID = 0` |
| `UpdateSleepTime` | `UpdateSleepTime(pub u32)` | 4 | `NONE = 1`, `FOREVER = 0x3fff_ffff` |

### 2. Ten Non-Negotiable Determinism Rules
1. **Wrapping Integer Arithmetic:** Never use bare `+`, `-`, `*`, `+=`, `-=`, `*=` on `i32`/`u32` where C++ can overflow or wrap. Use `.wrapping_add()`, `.wrapping_sub()`, `.wrapping_mul()`, `.wrapping_neg()`.
2. **Unfused IEEE-754 `f32` Arithmetic:** Never call `f32::mul_add` (FMA). Keep exact C++ parenthesization and left-to-right operator grouping: `(a * b) * c + d` must not be rewritten as `a * (b * c) + d`.
3. **Deterministic Transcendentals via `GameMath`:** Never call standard `f32::sin`, `cos`, `tan`, `asin`, `acos`, `atan2`, `sqrt`, `floor`, `ceil`, `powf`, `exp`, or `ln`. Always call `zh_common::gamemath::{gm_sinf, gm_cosf, gm_tanf, gm_asinf, gm_acosf, gm_atan2f, gm_sqrtf, gm_floorf, gm_ceilf, gm_powf, gm_expf, gm_logf}`.
4. **Mixed `Int * Real` Promotion & Float-to-Int Truncation:**
   - In C++, `Int a; Real s; a *= s;` evaluates `a = static_cast<Int>(static_cast<Real>(a) * s)`.
   - In Rust, always write: `a = ((a as f32) * s) as i32;` (or `zh_common::math::cpp_mul_i32_f32(a, s)`).
   - For `REAL_TO_INT_FLOOR(x)` and `REAL_TO_INT_CEIL(x)`, call `zh_common::math::{real_to_int_floor, real_to_int_ceil}` (which route to `gm_floorf(x) as i32` / `gm_ceilf(x) as i32` in default deterministic mode, and `fast_float_floor`/`ceil` under `#[cfg(feature = "retail-crc")]`).
5. **No `HashMap` or `HashSet`:** Never import `std::collections::{HashMap, HashSet}`. Always use `std::collections::{BTreeMap, BTreeSet}` keyed by a deterministic integer ID (`ObjectId`, `NameKeyType`, `TeamId`, `u32`), never by raw pointer address.
6. **No `BinaryHeap` for Sleepy Updates:** Use `zh_logic::SleepyUpdateHeap` (strict `>` comparator, left-child tie-breaking, in-place root sifting, reverse-heap-index removal).
7. **Evaluation Order of Side-Effecting Arguments:** When a C++ function call has 2+ arguments that call `GameLogicRandomValue*` or mutate state, evaluate them into explicit `let arg0 = ...; let arg1 = ...;` bindings in left-to-right order before the call.
8. **Exact PRNG Call Parity:** `GameLogicRandomValue(lo, hi)` and `GameLogicRandomValueReal(lo, hi)` return `hi` **without advancing the seed** when `lo >= hi` (under default `RETAIL_COMPATIBLE_CRC == 0`). Never reorder, hoist, or short-circuit a branch that calls `GameLogicRandomValue*`.
9. **Exact `Xfer` & `XferCRC` Field Order and Widths:**
   - Serialize fields in the **exact** C++ `crc(Xfer*)` and `xfer(Xfer*)` order.
   - Never serialize `usize` or `isize`; cast lengths to `u16`, `u32`, or `i32` matching the C++ local variable type (`UnsignedShort`, `UnsignedInt`, `Int`).
   - Remember `Bool` is `1 byte` (`u8`), and enum values serialized via `xferUser(&e, sizeof(Enum))` are `4 bytes` (`i32`).
10. **Preserve `#if !RETAIL_COMPATIBLE_CRC` Guards:** Translate `#if RETAIL_COMPATIBLE_CRC` / `#if !RETAIL_COMPATIBLE_CRC` blocks using `#[cfg(feature = "retail-crc")]` and `#[cfg(not(feature = "retail-crc"))]`.

---

## 3. Reference C++ Source (`fbraz3/GeneralsX` at `84a7a20`)

### `GeneralsMD/Code/GameEngine/Include/Common/GlobalData.h`
```cpp
// FILE: GlobalData.h /////////////////////////////////////////////////////////////////////////////
// Global data used by both the client and logic
// Author: trolfs, Michae Booth, Colin Day, April 2001
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

#include "Common/GameCommon.h"	// ensure we get DUMP_PERF_STATS, or not
#include "Common/AsciiString.h"
#include "Common/GameType.h"
#include "Common/GameMemory.h"
#include "Common/SubsystemInterface.h"
#include "GameClient/Color.h"
#include "Common/STLTypedefs.h"
#include "Common/Money.h"

// FORWARD DECLARATIONS ///////////////////////////////////////////////////////////////////////////
struct FieldParse;
enum _TerrainLOD CPP_11(: Int);
class CommandLine;
class GlobalData;
class INI;
class WeaponBonusSet;
enum BodyDamageType CPP_11(: Int);
enum AIDebugOptions CPP_11(: Int);

// PUBLIC /////////////////////////////////////////////////////////////////////////////////////////

constexpr const Int MAX_GLOBAL_LIGHTS = 3;
constexpr const Int SIMULATE_REPLAYS_SEQUENTIAL = -1;

//-------------------------------------------------------------------------------------------------
// Command-line parsing state is stored here instead of in CommandLine because
// the parsing result belongs to the GlobalData instance created during startup.
class CommandLineData
{
	friend class CommandLine;
	friend class GlobalData;

	CommandLineData()
		: m_hasParsedCommandLineForStartup(false)
		, m_hasParsedCommandLineForEngineInit(false)
	{}

	Bool m_hasParsedCommandLineForStartup;
	Bool m_hasParsedCommandLineForEngineInit;
	BoolVector m_parsedArguments;
};

//-------------------------------------------------------------------------------------------------
/** Global data container class
  *	Defines all global game data used by the system
	* @todo Change this entire system. Otherwise this will end up a huge class containing tons of variables,
	* and will cause re-compilation dependencies throughout the code base.
  * OOPS -- TOO LATE! :) */
//-------------------------------------------------------------------------------------------------
class GlobalData : public SubsystemInterface
{

public:

	GlobalData();
	virtual ~GlobalData() override;

	virtual void init() override;
	virtual void reset() override;
	virtual void update() override { }

	Bool setTimeOfDay( TimeOfDay tod );		///< Use this function to set the Time of day;

	static void parseGameDataDefinition( INI* ini );
	void parseCustomDefinition();

	//-----------------------------------------------------------------------------------------------
	struct TerrainLighting
	{
		RGBColor ambient;
		RGBColor diffuse;
		Coord3D lightPos;
	};

	//-----------------------------------------------------------------------------------------------
	//-----------------------------------------------------------------------------------------------
	//-----------------------------------------------------------------------------------------------

	CommandLineData m_commandLineData;

	AsciiString m_mapName;  ///< hack for now, this whole this is going away
	AsciiString m_moveHintName;
	Bool m_useTrees;
	Bool m_useTreeSway;
	Bool m_useDrawModuleLOD;
	Bool m_useHeatEffects;
	Bool m_useFpsLimit;
	Bool m_dumpAssetUsage;
	Int m_framesPerSecondLimit;
	// GeneralsX @feature felipebraz 17/09/2026 Skirmish simulation tick rate configuration (#281)
	Int m_skirmishTickRate;
	Int	m_chipSetType;	///<See W3DShaderManager::ChipsetType for options

	// TheSuperHackers @feature helmutbuhler 11/04/2025
	// Run game without graphics, input or audio.
	Bool m_headless;

	// GeneralsX @feature BenderAI 21/04/2026 Opt-out toggle for the in-game update checker.
	Bool m_checkForUpdates;

	Bool m_windowed;
	Int m_xResolution;
	Int m_yResolution;
	Int m_maxShellScreens;  ///< this many shells layouts can be loaded at once
	Bool m_useCloudMap;
	Int  m_use3WayTerrainBlends;	///< 0 is none, 1 is normal, 2 is debug.
	Bool m_useLightMap;
	Bool m_bilinearTerrainTex;
	Bool m_trilinearTerrainTex;
	Bool m_multiPassTerrain;
	Bool m_adjustCliffTextures;
	Bool m_stretchTerrain; // TheSuperHackers @info Legacy, unused
	Bool m_useHalfHeightMap; // TheSuperHackers @info Legacy, unused
	Bool m_drawEntireTerrain;
	_TerrainLOD m_terrainLOD;
	Bool m_enableDynamicLOD;
	Bool m_enableStaticLOD;
	Int m_terrainLODTargetTimeMS;
	Bool m_useAlternateMouse;
	Bool m_useRightMouseScrollWithAlternateMouse; // TheSuperHackers @feature User option for RMB scroll in Alternate Mouse mode.
	Bool m_clientRetaliationModeEnabled;
	Bool m_doubleClickAttackMove;
	Bool m_rightMouseAlwaysScrolls;
	Int m_jpegQuality; // TheSuperHackers @feature Quality for JPEG screenshots.
	Bool m_useWaterPlane;
	Bool m_useCloudPlane;
	Bool m_useShadowVolumes;
	// GeneralsX @feature Olcay Seygan / Ilyas Akin 08/10/2026 Also cast volume shadows off skinned meshes
	Bool m_useShadowVolumesForSkins;
	// GeneralsX @feature Olcay Seygan / Ilyas Akin 08/10/2026 Missiles and bombs get decal shadows
	Bool m_shadowsForProjectiles;
	// GeneralsX @feature Olcay Seygan / Ilyas Akin 08/10/2026 Scenery and props without shadows get one
	Bool m_shadowsForProps;
	// GeneralsX @feature fbraz3 08/10/2026 Enhanced graphics & lighting toggleable options
	Bool m_advancedShadows;
	Bool m_advancedLighting;
	Bool m_extendedShadowFrustumCulling;
	Bool m_naturalSunElevation;
	Bool m_bridgeShadows;
	Bool m_waterReflections;
	Bool m_explosionDynamicLights;
	Bool m_vehicleHeadlights;
	Bool m_enablePostProcessing;
	Bool m_postProcessBloom;
	Bool m_postProcessHDR;
	Bool m_postProcessFXAA;
	// GeneralsX @feature Mr. Meeseeks 16/10/2026 Tactical ground indicator for airborne units (C&C3 style ground ring)
	Bool m_airUnitGroundIndicator;

	Bool isUseShadowVolumesForSkins() const { return m_advancedShadows && m_useShadowVolumesForSkins; }
	Bool isShadowsForProjectiles() const { return m_advancedShadows && m_shadowsForProjectiles; }
	Bool isShadowsForProps() const { return m_advancedShadows && m_shadowsForProps; }
	Bool isExtendedShadowFrustumCulling() const { return m_advancedShadows && m_extendedShadowFrustumCulling; }
	Bool isNaturalSunElevation() const { return m_advancedShadows && m_naturalSunElevation; }
	Bool isBridgeShadows() const { return m_advancedShadows && m_bridgeShadows; }
	// GeneralsX @feature fbraz3 08/10/2026 Water reflection toggle via INI/Options
	Bool isWaterReflections() const { return m_waterReflections; }
	Bool isExplosionDynamicLights() const { return m_advancedLighting && m_explosionDynamicLights; }
	Bool isVehicleHeadlights() const { return m_advancedLighting && m_vehicleHeadlights; }
	Bool isEnablePostProcessing() const { return m_enablePostProcessing && (m_postProcessBloom || m_postProcessHDR || m_postProcessFXAA); }
	Bool isPostProcessBloom() const { return m_enablePostProcessing && m_postProcessBloom; }
	Bool isPostProcessHDR() const { return m_enablePostProcessing && m_postProcessHDR; }
	Bool isPostProcessFXAA() const { return m_enablePostProcessing && m_postProcessFXAA; }
	Bool isAirUnitGroundIndicator() const { return m_airUnitGroundIndicator; }
	Bool m_useShadowDecals;
	Int  m_textureReductionFactor;	//how much to cut texture resolution: 2 is half, 3 is quarter, etc.
	Bool m_enableBehindBuildingMarkers;
	Real m_waterPositionX;
	Real m_waterPositionY;
	Real m_waterPositionZ;
	Real m_waterExtentX;
	Real m_waterExtentY;
	Int	m_waterType;
	Bool m_showSoftWaterEdge;
	Bool m_usingWaterTrackEditor;
	Bool m_isWorldBuilder;

	Int m_featherWater;

	// these are for WATER_TYPE_3 vertex animated water
	enum { MAX_WATER_GRID_SETTINGS = 4 };
	AsciiString m_vertexWaterAvailableMaps[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterHeightClampLow[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterHeightClampHi[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterAngle[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterXPosition[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterYPosition[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterZPosition[ MAX_WATER_GRID_SETTINGS ];
	Int m_vertexWaterXGridCells[ MAX_WATER_GRID_SETTINGS ];
	Int m_vertexWaterYGridCells[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterGridSize[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterAttenuationA[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterAttenuationB[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterAttenuationC[ MAX_WATER_GRID_SETTINGS ];
	Real m_vertexWaterAttenuationRange[ MAX_WATER_GRID_SETTINGS ];

	Real m_downwindAngle;
	Real m_skyBoxPositionZ;
	Real m_drawSkyBox;
	Real m_skyBoxScale;
	Real m_viewportHeightScale; // The height scale of the tactical view ranging 0..1. Used to hide the world behind the Control Bar.
	Real m_cameraPitch;
	Real m_cameraYaw;
#if PRESERVE_RETAIL_SCRIPTED_CAMERA
	Real m_cameraHeight;
#endif
	Real m_maxCameraHeight;
	Real m_minCameraHeight;

	Real m_terrainDrawDistanceScale;

	Real m_terrainHeightAtEdgeOfMap;
	Real m_unitDamagedThresh;
	Real m_unitReallyDamagedThresh;
	Real m_groundStiffness;
	Real m_structureStiffness;
	Real m_gravity;	// acceleration due to gravity, in dist/frame^2
	Real m_stealthFriendlyOpacity;
	UnsignedInt m_defaultOcclusionDelay;	///<time to delay building occlusion after object is created.

	Bool m_preloadAssets;
	Bool m_preloadEverything;			///< Preload everything, everywhere (for debugging only)
	Bool m_preloadReport;					///< dump a log of all W3D assets that are being preloaded.

	Real m_partitionCellSize;

	Coord3D m_ammoPipWorldOffset;
	Coord3D m_containerPipWorldOffset;
	Coord2D m_ammoPipScreenOffset;
	Coord2D m_containerPipScreenOffset;
	Real m_ammoPipScaleFactor;
	Real m_containerPipScaleFactor;

	UnsignedInt m_historicDamageLimit;

	//Settings for terrain tracks left by vehicles with treads or wheels
	Int m_maxTerrainTracks; ///<maximum number of objects allowed to generate tracks.
	Int m_maxTankTrackEdges;	///<maximum length of tank track
	Int m_maxTankTrackOpaqueEdges;	///<maximum length of tank track before it starts fading.
	Int m_maxTankTrackFadeDelay;	///<maximum amount of time a tank track segment remains visible.

	AsciiString m_levelGainAnimationName; ///< The animation to play when a level is gained.
	Real m_levelGainAnimationDisplayTimeInSeconds;		///< time to play animation for
	Real m_levelGainAnimationZRisePerSecond;					///< rise animation up while playing

	AsciiString m_getHealedAnimationName; ///< The animation to play when emergency repair does its thing.
	Real m_getHealedAnimationDisplayTimeInSeconds;		///< time to play animation for
	Real m_getHealedAnimationZRisePerSecond;					///< rise animation up while playing

	TimeOfDay	m_timeOfDay;
	Weather m_weather;
	Bool m_makeTrackMarks;
	Bool m_hideGarrisonFlags;
	Bool m_forceModelsToFollowTimeOfDay;
	Bool m_forceModelsToFollowWeather;

	TerrainLighting	m_terrainLighting[TIME_OF_DAY_COUNT][MAX_GLOBAL_LIGHTS];
	TerrainLighting	m_terrainObjectsLighting[TIME_OF_DAY_COUNT][MAX_GLOBAL_LIGHTS];

	//Settings for each global light
	RGBColor m_terrainAmbient[MAX_GLOBAL_LIGHTS];
	RGBColor m_terrainDiffuse[MAX_GLOBAL_LIGHTS];
	Coord3D m_terrainLightPos[MAX_GLOBAL_LIGHTS];

	Real m_infantryLightScale[TIME_OF_DAY_COUNT];
	Real m_scriptOverrideInfantryLightScale;

	Real m_soloPlayerHealthBonusForDifficulty[PLAYERTYPE_COUNT][DIFFICULTY_COUNT];

	Int m_maxVisibleTranslucentObjects;
	Int m_maxVisibleOccluderObjects;
	Int m_maxVisibleOccludeeObjects;
	Int m_maxVisibleNonOccluderOrOccludeeObjects;
	Real m_occludedLuminanceScale;

	Int m_numGlobalLights;	//number of active global lights
	Int m_maxRoadSegments;
	Int m_maxRoadVertex;
	Int m_maxRoadIndex;
	Int m_maxRoadTypes;

	Bool m_audioOn;
	Bool m_musicOn;
	Bool m_soundsOn;
	Bool m_sounds3DOn;
	Bool m_speechOn;
	Bool m_videoOn;
	Bool m_disableCameraMovement;

	Bool m_useFX;									///< If false, don't render effects
	Bool m_showClientPhysics;
	Bool m_showTerrainNormals;

	UnsignedInt m_noDraw;					///< Used to disable drawing, to profile game logic code.
	AIDebugOptions m_debugAI;			///< Used to display AI debug information
	Bool m_debugSupplyCenterPlacement; ///< Dumps to log everywhere it thinks about placing a supply center
	Bool m_debugAIObstacles;			///< Used to display AI obstacle debug information
	Bool m_showObjectHealth;			///< debug display object health
	Bool m_scriptDebug;						///< Should we attempt to load the script debugger window (.DLL)
	Bool m_particleEdit;					///< Should we attempt to load the particle editor (.DLL)
	Bool m_displayDebug;					///< Used to display display debug info
	Bool m_winCursors;						///< Should we force use of windows cursors?
	Bool m_constantDebugUpdate;		///< should we update the debug stats constantly, vs every 2 seconds?
	Bool m_showTeamDot;						///< Shows the little colored team dot representing which team you are controlling.

#ifdef DUMP_PERF_STATS
	Bool m_dumpPerformanceStatistics;
  Bool  m_dumpStatsAtInterval;///< should I automatically dump stats every N frames
  Int   m_statsInterval;       ///< if so, how many is N?
#endif

	Bool m_forceBenchmark;	///<forces running of CPU detection benchmark, even on known cpu's.

	Int m_fixedSeed;							///< fixed random seed for game logic (less than 0 to disable)

	Real m_particleScale;					///< Global size modifier for particle effects.

	AsciiString m_autoFireParticleSmallPrefix;
	AsciiString m_autoFireParticleSmallSystem;
	Int m_autoFireParticleSmallMax;
	AsciiString m_autoFireParticleMediumPrefix;
	AsciiString m_autoFireParticleMediumSystem;
	Int m_autoFireParticleMediumMax;
	AsciiString m_autoFireParticleLargePrefix;
	AsciiString m_autoFireParticleLargeSystem;
	Int m_autoFireParticleLargeMax;
	AsciiString m_autoSmokeParticleSmallPrefix;
	AsciiString m_autoSmokeParticleSmallSystem;
	Int m_autoSmokeParticleSmallMax;
	AsciiString m_autoSmokeParticleMediumPrefix;
	AsciiString m_autoSmokeParticleMediumSystem;
	Int m_autoSmokeParticleMediumMax;
	AsciiString m_autoSmokeParticleLargePrefix;
	AsciiString m_autoSmokeParticleLargeSystem;
	Int m_autoSmokeParticleLargeMax;
	AsciiString m_autoAflameParticlePrefix;
	AsciiString m_autoAflameParticleSystem;
	Int m_autoAflameParticleMax;

	// Latency insertion, packet loss for network debugging
	Int m_netMinPlayers;					///< Min players needed to start a net game

	UnsignedInt m_defaultIP;			///< preferred IP address for LAN
	UnsignedInt m_firewallBehavior;	///< Last detected firewall behavior
	UnsignedInt m_firewallPortOverride;	///< User-specified port to be used
	Short m_firewallPortAllocationDelta; ///< the port allocation delta last detected.

	Int m_baseValuePerSupplyBox;
	Real m_BuildSpeed;
	Real m_MinDistFromEdgeOfMapForBuild;
	Real m_SupplyBuildBorder;
	Real m_allowedHeightVariationForBuilding;  ///< how "flat" is still flat enough to build on
	Real m_MinLowEnergyProductionSpeed;
	Real m_MaxLowEnergyProductionSpeed;
	Real m_LowEnergyPenaltyModifier;
	Real m_MultipleFactory;
	Real m_RefundPercent;

	Real m_commandCenterHealRange;		///< radius in which close by ally things are healed
	Real m_commandCenterHealAmount;   ///< health per logic frame close by things are healed
	Int m_maxLineBuildObjects;				///< line style builds can be no longer than this
	Int m_maxTunnelCapacity;					///< Max people in Player's tunnel network
	Real m_horizontalScrollSpeedFactor;	///< Factor applied to the game screen scrolling speed.
	Real m_verticalScrollSpeedFactor;		///< Separated because of our aspect ratio
	Real m_scrollAmountCutoff;				///< Scroll speed to not adjust camera height
	Real m_cameraAdjustSpeed;					///< Rate at which we adjust camera height
	Bool m_enforceMaxCameraHeight;		///< Enforce max camera height while scrolling?
	Bool m_buildMapCache;
	AsciiString m_initialFile;				///< If this is specified, load a specific map from the command-line
	AsciiString m_pendingFile;				///< If this is specified, use this map at the next game start
	AsciiString m_loadSaveGame;				///< If this is specified, load a save game file from the command-line
	AsciiString m_loadReplayGame;			///< If this is specified, play a replay file from the command-line

	std::vector<AsciiString> m_simulateReplays; ///< If not empty, simulate this list of replays and exit.
	Int m_simulateReplayJobs; ///< Maximum number of processes to use for simulation, or SIMULATE_REPLAYS_SEQUENTIAL for sequential simulation

	Int m_maxParticleCount;						///< maximum number of particles that can exist
	Int m_maxFieldParticleCount;			///< maximum number of field-type particles that can exist (roughly)
	WeaponBonusSet* m_weaponBonusSet;
	Real m_healthBonus[LEVEL_COUNT];			///< global bonuses to health for veterancy.
	Real m_defaultStructureRubbleHeight;	///< for rubbled structures, compress height to this if none specified

	AsciiString m_shellMapName;				///< Holds the shell map name
	Bool m_shellMapOn;								///< User can set the shell map not to load
	Bool m_playIntro;									///< Flag to say if we're to play the intro or not
	Bool m_playSizzle;								///< Flag to say whether we play the sizzle movie after the logo movie.

	Bool m_loadScreenRender;						///< flag to disallow rendering of almost everything during a loadscreen

	Real m_keyboardScrollFactor;			///< Factor applied to game scrolling speed via keyboard scrolling
	// GeneralsX @feature Gabriel Petry 30/09/2026 Add configurable mouse scrolling speed.
	Real m_mouseScrollFactor;				///< Factor applied to game scrolling speed via mouse scrolling
	Real m_keyboardDefaultScrollFactor;			///< Factor applied to game scrolling speed via keyboard scrolling
	Bool m_drawScrollAnchor;					///< Set that the scroll anchor should be enabled
	Bool m_moveScrollAnchor;					///< set that the scroll anchor should move

	Bool m_animateWindows;						///< Should we animate window transitions?

	Bool m_incrementalAGPBuf;

	UnsignedInt m_iniCRC;							///< CRC of important INI files
	UnsignedInt m_exeCRC;							///< CRC of the executable

	BodyDamageType m_movementPenaltyDamageState;	///< at this body damage state, we have movement penalties

	Int m_groupSelectMinSelectSize;		// min number of units to treat as group select for audio feedback
	Real m_groupSelectVolumeBase;			// base volume for group select sound
	Real m_groupSelectVolumeIncrement;// increment to volume for selecting more units
	Int m_maxUnitSelectSounds;				// max number of select sounds to play per selection

	Real m_selectionFlashSaturationFactor; /// how colorful should the selection flash be? 0-4
	Bool m_selectionFlashHouseColor ;  /// skip the house color and just use white.

	Real m_cameraAudibleRadius;				///< If the camera is being used as the position of audio, then how far can we hear?
	Real m_groupMoveClickToGatherFactor; /** if you take all the selected units and calculate the smallest possible rectangle
																			 that contains them all, and click within that, all the selected units will break
																			 formation and gather at the point the user clicked (if the value is 1.0). If it's 0.0,
																			 units will always keep their formation. If it's <1.0, then the user must click a
																			 smaller area within the rectangle to order the gather. */

	UnsignedInt m_antiAliasLevel;          ///< value of selected antialias level in the game options
	UnsignedInt m_textureFilteringMode;       ///< value related to TextureFilterClass::TextureFilterModeEnum
	UnsignedInt m_textureAnisotropyLevel;     ///< value related to TextureFilterClass::AnisotropicFilterMode

	Bool m_languageFilterPref;        ///< Bool if user wants to filter language
	Bool m_loadScreenDemo;						///< Bool if true, run the loadscreen demo movie
	Bool m_disableRender;							///< if true, no rendering!

	Bool m_saveCameraInReplay;
	Bool m_useCameraInReplay;

	// TheSuperHackers @feature xezon 09/09/2025 Enable the shroud and everything related for observing individual players in any game mode.
	// Enabling this does have a performance cost because the ghost object manager must keep track of all relevant objects for all players.
	Bool m_enablePlayerObserver;

	// TheSuperHackers @feature L3-M 05/09/2025 allow the network latency counter and render fps counter font size to be set, a size of zero disables them
	Int m_networkLatencyFontSize;
	Int m_renderFpsFontSize;
	// TheSuperHackers @feature Mauller 21/06/2025 allow the system time and game time font size to be set, a size of zero disables them
	Int m_systemTimeFontSize;
	Int m_gameTimeFontSize;
	// TheSuperHackers @feature L3-M 05/11/2025 allow the player info list font size to be set, a size of zero disables it
	Int m_playerInfoListFontSize;

	// TheSuperHackers @feature L3-M 21/08/2025 toggle the money per minute display, false shows only the original current money
	Bool m_showMoneyPerMinute;
	Bool m_allowMoneyPerMinuteForPlayer;

	// TheSuperHackers @feature bobtista 28/06/2026 user-configurable speed multiplier for game window transitions
	Real m_gameWindowTransitionSpeedMultiplier;

	Real m_shakeSubtleIntensity;			///< Intensity for shaking a camera with SHAKE_SUBTLE
	Real m_shakeNormalIntensity;			///< Intensity for shaking a camera with SHAKE_NORMAL
	Real m_shakeStrongIntensity;			///< Intensity for shaking a camera with SHAKE_STRONG
	Real m_shakeSevereIntensity;			///< Intensity for shaking a camera with SHAKE_SEVERE
	Real m_shakeCineExtremeIntensity; ///< Intensity for shaking a camera with SHAKE_CINE_EXTREME
	Real m_shakeCineInsaneIntensity;  ///< Intensity for shaking a camera with SHAKE_CINE_INSANE
	Real m_maxShakeIntensity;					///< The maximum shake intensity we can have
	Real m_maxShakeRange;							///< The maximum shake range we can have

	Real m_sellPercentage;						///< when objects are sold, you get this much of the cost it would take to build it back
	Real m_baseRegenHealthPercentPerSecond;	///< auto healing for bases
	UnsignedInt m_baseRegenDelay;			///< delay in frames we must be damage free before we can auto heal

#ifdef ALLOW_SURRENDER
	Real m_prisonBountyMultiplier;		///< the cost of the unit is multiplied by this and given to the player when prisoners are returned to the a prison with KINDOF_COLLECTS_PRISON_BOUNTY
	Color m_prisonBountyTextColor;		///< color of the text that displays the money acquired at the prison
#endif

	Color m_hotKeyTextColor;					///< standard color for all hotkeys.

  //THis is put on ice until later - M Lorenzen
  //	Int m_cheaterHasBeenSpiedIfMyLowestBitIsTrue; ///< says it all.. this lives near other "colors" cause it is masquerading as one

	AsciiString m_specialPowerViewObjectName;	///< Created when certain special powers are fired so players can watch.

	Real m_objectPlacementOpacity; ///< Sets the opacity of build preview objects.
	Bool m_objectPlacementShadows; ///< Enables or disables shadows of build preview objects.

	std::vector<AsciiString> m_standardPublicBones;

	Real m_standardMinefieldDensity;
	Real m_standardMinefieldDistance;


	Bool  m_showMetrics;								///< whether or not to show the metrics.
	Money m_defaultStartingCash;				///< The amount of cash a player starts with by default.

	Bool m_debugShowGraphicalFramerate;		///< Whether or not to show the graphical framerate bar.

	Int m_powerBarBase;										///< Logarithmic base for the power bar scale
	Real m_powerBarIntervals;							///< how many logarithmic intervals the width will be divided into
	Int m_powerBarYellowRange;						///< Red if consumption exceeds production, yellow if consumption this close but under, green if further under
	Real m_displayGamma;									///<display gamma that's adjusted with "brightness" control on options screen.

	UnsignedInt m_unlookPersistDuration;	///< How long after unlook until the sighting info executes the undo

	Bool m_shouldUpdateTGAToDDS;					///< Should we attempt to update old TGAs to DDS stuff on loadup?

	UnsignedInt m_doubleClickTimeMS;	///< What is the maximum amount of time that can separate two clicks in order
																		///< for us to generate a double click message?

	RGBColor m_shroudColor;						///< What color should the shroud be?  Remember, this is a lighting multiply, not an add
	UnsignedByte m_clearAlpha;				///< 255 means perfect visibility
	UnsignedByte m_fogAlpha;					///< 127 means fog is half as obscuring as shroud
	UnsignedByte m_shroudAlpha;				///< 0 makes this opaque, but they may get fancy

	// network timing values.
	UnsignedInt m_networkFPSHistoryLength;		      	///< The number of fps history entries
	UnsignedInt m_networkLatencyHistoryLength;      	///< The number of ping history entries.
	UnsignedInt m_networkCushionHistoryLength;      	///< The number of cushion values to keep.
	UnsignedInt m_networkRunAheadMetricsTime;	      	///< The number of miliseconds between run ahead metrics things
	UnsignedInt m_networkKeepAliveDelay;			      	///< The number of seconds between when the connections to each player send a keep-alive packet.
	UnsignedInt m_networkRunAheadSlack;				      	///< The amount of slack in the run ahead value. This is the percentage of the calculated run ahead that is added.
	UnsignedInt m_networkDisconnectTime;			      	///< The number of milliseconds between when the game gets stuck on a frame for a network stall and when the disconnect dialog comes up.
	UnsignedInt m_networkPlayerTimeoutTime;		      	///< The number of milliseconds between when a player's last keep alive command was received and when they are considered disconnected from the game.
	UnsignedInt	m_networkDisconnectScreenNotifyTime;  ///< The number of milliseconds between when the disconnect screen comes up and when the other players are notified that we are on the disconnect screen.

	Real				m_keyboardCameraRotateSpeed;    ///< How fast the camera rotates when rotated via keyboard controls.
  Int					m_playStats;									///< Int whether we want to log play stats or not, if <= 0 then we don't log

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	Bool m_specialPowerUsesDelay ;
#endif
  Bool m_TiVOFastMode;            ///< When true, the client speeds up the framerate... set by HOTKEY!

#if defined(RTS_DEBUG) || ENABLE_CONFIGURABLE_SHROUD
	Bool m_shroudOn;
#endif

#if defined(RTS_DEBUG)
	Bool m_wireframe;
	Bool m_stateMachineDebug;
	Bool m_useCameraConstraints;
	Bool m_fogOfWarOn;
	Bool m_jabberOn;
	Bool m_munkeeOn;
	Bool m_allowUnselectableSelection;			///< Are we allowed to select things that are unselectable?
	Bool m_disableCameraFade;								///< if true, script commands affecting camera are disabled
	Bool m_disableScriptedInputDisabling;		///< if true, script commands can't disable input
	Bool m_disableMilitaryCaption;					///< if true, military briefings go fast
	Int m_benchmarkTimer;										///< how long to play the game in benchmark mode?
  Bool m_checkForLeaks;
	Bool m_vTune;
	Bool m_debugCamera;						///< Used to display Camera debug information
	Bool m_debugVisibility;						///< Should we actively debug the visibility
	Int m_debugVisibilityTileCount;		///< How many tiles we should show when debugging visibility
	Real m_debugVisibilityTileWidth;	///< How wide should these tiles be?
	Int m_debugVisibilityTileDuration;	///< How long should these tiles stay around, in frames?
	Bool m_debugThreatMap;						///< Should we actively debug the threat map
	UnsignedInt m_maxDebugThreat;			///< This value (and any values greater) will appear full RED.
	Int m_debugThreatMapTileDuration;	///< How long should these tiles stay around, in frames?
	Bool m_debugCashValueMap;					///< Should we actively debug the threat map
	UnsignedInt m_maxDebugValue;			///< This value (and any values greater) will appear full GREEN.
	Int m_debugCashValueMapTileDuration;	///< How long should these tiles stay around, in frames?
	RGBColor m_debugVisibilityTargettableColor;	///< What color should the targettable cells be?
	RGBColor m_debugVisibilityDeshroudColor;			///< What color should the deshrouding cells be?
	RGBColor m_debugVisibilityGapColor;					///< What color should the gap generator cells be?
	Bool m_debugProjectilePath;						///< Should we actively debug the bezier paths on projectiles
	Real m_debugProjectileTileWidth;			///< How wide should these tiles be?
	Int m_debugProjectileTileDuration;		///< How long should these tiles stay around, in frames?
	RGBColor m_debugProjectileTileColor;	///< What color should these tiles be?
	Bool m_showCollisionExtents;	///< Used to display collision extents
  Bool m_showAudioLocations;    ///< Used to display audio markers and ambient sound radii
	Bool m_saveStats;
	Bool m_saveAllStats;
	Bool m_useLocalMOTD;
	AsciiString m_baseStatsDir;
	AsciiString m_MOTDPath;
	Int m_latencyAverage;					///< Average latency to insert
	Int m_latencyAmplitude;				///< Amplitude of sinusoidal modulation of latency
	Int m_latencyPeriod;					///< Period of sinusoidal modulation of latency
	Int m_latencyNoise;						///< Max amplitude of jitter to throw in
	Int m_packetLoss;							///< Percent of packets to drop
	Bool m_extraLogging;					///< More expensive debug logging to catch crashes.
#endif

#ifdef DEBUG_CRASHING
	Bool m_debugIgnoreAsserts;						///< Ignore all asserts.
#endif

#ifdef DEBUG_STACKTRACE
	Bool m_debugIgnoreStackTrace;					///< No stacktraces for asserts.
#endif

	Bool				m_isBreakableMovie;							///< if we enter a breakable movie, set this flag
	Bool				m_breakTheMovie;								///< The user has hit escape!
	AsciiString m_modDir;
	AsciiString m_modBIG;
	// GeneralsX @feature felipebraz 13/11/2026 Runtime asset path overrides via command line (-assetPath / -generalsAssetPath)
	AsciiString m_assetPath;
	AsciiString m_generalsAssetPath;
	// GeneralsX @feature dogefly233/felipebraz 13/11/2026 Force loose files on disk to override mod archives (#397)
	Bool        m_looseFilesWin;

	//-allAdvice feature
	//Bool m_allAdvice;


	// the trailing '\' is included!
  const AsciiString &getPath_UserData() const { return m_userDataDir; }

private:

	static UnsignedInt generateExeCRC();

	static const FieldParse s_GlobalDataFieldParseTable[];

	// this is private, since we read the info from Windows and cache it for
	// future use. No one is allowed to change it, ever. (srj)
	AsciiString m_userDataDir;
	AsciiString BuildUserDataPathFromRegistry();

	static GlobalData *m_theOriginal;		///< the original global data instance (no overrides)
	GlobalData *m_next;									///< next instance (for overrides)
	virtual GlobalData *newOverride();		/** create a new override, copy data from previous
																			override, and return it */

#if defined(_MSC_VER) && _MSC_VER < 1300
	GlobalData(const GlobalData& that) { DEBUG_CRASH(("unimplemented")); }
	GlobalData& operator=(const GlobalData& that) { DEBUG_CRASH(("unimplemented")); return *this; }
#else
	GlobalData(const GlobalData& that) = delete;
	GlobalData& operator=(const GlobalData& that) = default;
#endif

};

// singleton
extern GlobalData* TheWritableGlobalData;

// use TheGlobalData for all read-only accesses
#if __cplusplus >= 201703L
inline const GlobalData* const& TheGlobalData = TheWritableGlobalData;
#else
#define TheGlobalData ((const GlobalData*)TheWritableGlobalData)
#endif
```

### `GeneralsMD/Code/GameEngine/Source/Common/GlobalData.cpp`
```cpp
// FILE: GlobalData.cpp ///////////////////////////////////////////////////////////////////////////
// The GameLogicData object
// Author: trolfs, Michael Booth, Colin Day, April 2001
///////////////////////////////////////////////////////////////////////////////////////////////////

//#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#include "WW3D2/ww3d.h"
#include "WW3D2/texturefilter.h"

#include "Common/GlobalData.h"

#define DEFINE_TERRAIN_LOD_NAMES
#define DEFINE_TIME_OF_DAY_NAMES
#define DEFINE_WEATHER_NAMES
#define DEFINE_BODYDAMAGETYPE_NAMES
#define DEFINE_PANNING_NAMES

#include "Common/AddonCompat.h"
#include "Common/crc.h"
#include "Common/file.h"
#include "Common/FileSystem.h"
#include "Common/GameAudio.h"
#include "Common/INI.h"
#include "Common/Registry.h"
#include "Common/OptionPreferences.h"
#include "Common/version.h"

#include "GameLogic/AI.h"
#include "GameLogic/Weapon.h"
#include "GameLogic/Module/BodyModule.h"

#include "GameClient/Color.h"
#include "GameClient/Display.h"
#include "GameClient/TerrainVisual.h"

#include "GameNetwork/FirewallHelper.h"

// TheSuperHackers @build fighter19 11/02/2026 Linux POSIX compatibility headers
#ifndef _WIN32
#include <unistd.h>     // readlink()
#include <sys/stat.h>   // mkdir()
#include <cstdlib>      // getenv()
#include <filesystem>   // std::filesystem::create_directories()

// GeneralsX @feature BenderAI 24/02/2026 Phase 5 macOS cross-platform exe path detection
#ifdef __APPLE__
#include <mach-o/dyld.h>  // _NSGetExecutablePath()
#endif
#endif

// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
GlobalData* TheWritableGlobalData = nullptr;				///< The global data singleton

//-------------------------------------------------------------------------------------------------
GlobalData* GlobalData::m_theOriginal = nullptr;

///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
/*static*/ const FieldParse GlobalData::s_GlobalDataFieldParseTable[] =
{
	{ "Windowed",									INI::parseBool,				nullptr,			offsetof( GlobalData, m_windowed ) },
	// GeneralsX @feature BenderAI 21/04/2026 Opt-out for in-game update checker via Options.ini
	{ "CheckForUpdates",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_checkForUpdates ) },
	{ "XResolution",							INI::parseInt,				nullptr,			offsetof( GlobalData, m_xResolution ) },
	{ "YResolution",							INI::parseInt,				nullptr,			offsetof( GlobalData, m_yResolution ) },
	{ "MapName",									INI::parseAsciiString,nullptr,			offsetof( GlobalData, m_mapName ) },
	{ "MoveHintName",							INI::parseAsciiString,nullptr,			offsetof( GlobalData, m_moveHintName ) },
	{ "UseTrees",									INI::parseBool,				nullptr,			offsetof( GlobalData, m_useTrees ) },
	{ "UseFPSLimit",							INI::parseBool,				nullptr,			offsetof( GlobalData, m_useFpsLimit ) },
	{ "DumpAssetUsage",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_dumpAssetUsage ) },
	{ "FramesPerSecondLimit",			INI::parseInt,				nullptr,			offsetof( GlobalData, m_framesPerSecondLimit ) },
	// GeneralsX @feature felipebraz 17/09/2026 Skirmish simulation tick rate configuration (#281)
	{ "SkirmishTickRate",					INI::parseInt,				nullptr,			offsetof( GlobalData, m_skirmishTickRate ) },
	{ "TickRate",									INI::parseInt,				nullptr,			offsetof( GlobalData, m_skirmishTickRate ) },
	{ "ChipsetType",							INI::parseInt,				nullptr,			offsetof( GlobalData, m_chipSetType ) },
	{ "MaxShellScreens",					INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxShellScreens ) },
	{ "UseCloudMap",							INI::parseBool,				nullptr,			offsetof( GlobalData, m_useCloudMap ) },
	{ "UseLightMap",							INI::parseBool,				nullptr,			offsetof( GlobalData, m_useLightMap ) },
	{ "BilinearTerrainTex",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_bilinearTerrainTex ) },
	{ "TrilinearTerrainTex",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_trilinearTerrainTex ) },
	{ "MultiPassTerrain",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_multiPassTerrain ) },
	{ "AdjustCliffTextures",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_adjustCliffTextures ) },
	{ "Use3WayTerrainBlends",			INI::parseInt,				nullptr,			offsetof( GlobalData, m_use3WayTerrainBlends ) },
	{ "StretchTerrain",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_stretchTerrain ) },
	{ "UseHalfHeightMap",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_useHalfHeightMap ) },


	{ "DrawEntireTerrain",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_drawEntireTerrain ) },
	{ "TerrainLOD",									INI::parseIndexList,	TerrainLODNames,	offsetof( GlobalData, m_terrainLOD ) },
	{ "TerrainLODTargetTimeMS",			INI::parseInt,				nullptr,			offsetof( GlobalData, m_terrainLODTargetTimeMS ) },
	{ "RightMouseAlwaysScrolls",		INI::parseBool,				nullptr,			offsetof( GlobalData, m_rightMouseAlwaysScrolls ) },
	{ "UseWaterPlane",							INI::parseBool,				nullptr,			offsetof( GlobalData, m_useWaterPlane ) },
	{ "UseCloudPlane",							INI::parseBool,				nullptr,			offsetof( GlobalData, m_useCloudPlane ) },
	{ "DownwindAngle",							INI::parseReal,				nullptr,			offsetof( GlobalData, m_downwindAngle ) },
	{ "UseShadowVolumes",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_useShadowVolumes ) },
	// GeneralsX @feature Olcay Seygan / Ilyas Akin 08/10/2026 Also cast volume shadows off skinned meshes
	{ "UseShadowVolumesForSkins",		INI::parseBool,				nullptr,			offsetof( GlobalData, m_useShadowVolumesForSkins ) },
	// GeneralsX @feature Olcay Seygan / Ilyas Akin 08/10/2026 Missiles and bombs get decal shadows
	{ "ShadowsForProjectiles",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_shadowsForProjectiles ) },
	// GeneralsX @feature Olcay Seygan / Ilyas Akin 08/10/2026 Scenery and props without shadows get one
	{ "ShadowsForProps",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_shadowsForProps ) },
	// GeneralsX @feature fbraz3 08/10/2026 Enhanced graphics & lighting toggleable options
	{ "AdvancedShadows",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_advancedShadows ) },
	{ "AdvancedLighting",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_advancedLighting ) },
	{ "ExtendedShadowFrustumCulling",	INI::parseBool,				nullptr,			offsetof( GlobalData, m_extendedShadowFrustumCulling ) },
	{ "NaturalSunElevation",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_naturalSunElevation ) },
	{ "BridgeShadows",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_bridgeShadows ) },
	{ "WaterReflections",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_waterReflections ) },
	{ "ExplosionDynamicLights",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_explosionDynamicLights ) },
	{ "VehicleHeadlights",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_vehicleHeadlights ) },
	{ "EnablePostProcessing",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_enablePostProcessing ) },
	{ "PostProcessBloom",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_postProcessBloom ) },
	{ "PostProcessHDR",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_postProcessHDR ) },
	{ "PostProcessFXAA",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_postProcessFXAA ) },
	// GeneralsX @feature Mr. Meeseeks 16/10/2026 Tactical ground indicator for airborne units (C&C3 style ground ring)
	{ "AirUnitGroundIndicator",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_airUnitGroundIndicator ) },
	{ "UseShadowDecals",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_useShadowDecals ) },
	{ "TextureReductionFactor",			INI::parseInt,				nullptr,			offsetof( GlobalData, m_textureReductionFactor ) },
	{ "UseBehindBuildingMarker",		INI::parseBool,				nullptr,			offsetof( GlobalData, m_enableBehindBuildingMarkers ) },
	{ "WaterPositionX",							INI::parseReal,				nullptr,			offsetof( GlobalData, m_waterPositionX ) },
	{ "WaterPositionY",							INI::parseReal,				nullptr,			offsetof( GlobalData, m_waterPositionY ) },
	{ "WaterPositionZ",							INI::parseReal,				nullptr,			offsetof( GlobalData, m_waterPositionZ ) },
	{ "WaterExtentX",								INI::parseReal,				nullptr,			offsetof( GlobalData, m_waterExtentX ) },
	{ "WaterExtentY",								INI::parseReal,				nullptr,			offsetof( GlobalData, m_waterExtentY ) },
	{ "WaterType",									INI::parseInt,				nullptr,			offsetof( GlobalData, m_waterType ) },
	{ "FeatherWater",						  	INI::parseInt,				nullptr,			offsetof( GlobalData, m_featherWater ) },
	{ "ShowSoftWaterEdge",					INI::parseBool,				nullptr,			offsetof( GlobalData, m_showSoftWaterEdge ) },

	// nasty ick, we need to save this data with a map and not hard code INI values
	{ "VertexWaterAvailableMaps1",		INI::parseAsciiString,	nullptr,		offsetof( GlobalData, m_vertexWaterAvailableMaps[ 0 ] ) },
	{ "VertexWaterHeightClampLow1",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampLow[ 0 ] ) },
	{ "VertexWaterHeightClampHi1",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampHi[ 0 ] ) },
	{ "VertexWaterAngle1",						INI::parseAngleReal,		nullptr,		offsetof( GlobalData, m_vertexWaterAngle[ 0 ] ) },
	{ "VertexWaterXPosition1",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterXPosition[ 0 ] ) },
	{ "VertexWaterYPosition1",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterYPosition[ 0 ] ) },
	{ "VertexWaterZPosition1",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterZPosition[ 0 ] ) },
	{ "VertexWaterXGridCells1",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterXGridCells[ 0 ] ) },
	{ "VertexWaterYGridCells1",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterYGridCells[ 0 ] ) },
	{ "VertexWaterGridSize1",					INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterGridSize[ 0 ] ) },
	{ "VertexWaterAttenuationA1",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationA[ 0 ] ) },
	{ "VertexWaterAttenuationB1",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationB[ 0 ] ) },
	{ "VertexWaterAttenuationC1",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationC[ 0 ] ) },
	{ "VertexWaterAttenuationRange1",	INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationRange[ 0 ] ) },

	// nasty ick, we need to save this data with a map and not hard code INI values
	{ "VertexWaterAvailableMaps2",		INI::parseAsciiString,	nullptr,		offsetof( GlobalData, m_vertexWaterAvailableMaps[ 1 ] ) },
	{ "VertexWaterHeightClampLow2",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampLow[ 1 ] ) },
	{ "VertexWaterHeightClampHi2",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampHi[ 1 ] ) },
	{ "VertexWaterAngle2",						INI::parseAngleReal,		nullptr,		offsetof( GlobalData, m_vertexWaterAngle[ 1 ] ) },
	{ "VertexWaterXPosition2",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterXPosition[ 1 ] ) },
	{ "VertexWaterYPosition2",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterYPosition[ 1 ] ) },
	{ "VertexWaterZPosition2",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterZPosition[ 1 ] ) },
	{ "VertexWaterXGridCells2",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterXGridCells[ 1 ] ) },
	{ "VertexWaterYGridCells2",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterYGridCells[ 1 ] ) },
	{ "VertexWaterGridSize2",					INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterGridSize[ 1 ] ) },
	{ "VertexWaterAttenuationA2",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationA[ 1 ] ) },
	{ "VertexWaterAttenuationB2",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationB[ 1 ] ) },
	{ "VertexWaterAttenuationC2",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationC[ 1 ] ) },
	{ "VertexWaterAttenuationRange2",	INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationRange[ 1 ] ) },

	// nasty ick, we need to save this data with a map and not hard code INI values
	{ "VertexWaterAvailableMaps3",		INI::parseAsciiString,	nullptr,		offsetof( GlobalData, m_vertexWaterAvailableMaps[ 2 ] ) },
	{ "VertexWaterHeightClampLow3",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampLow[ 2 ] ) },
	{ "VertexWaterHeightClampHi3",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampHi[ 2 ] ) },
	{ "VertexWaterAngle3",						INI::parseAngleReal,		nullptr,		offsetof( GlobalData, m_vertexWaterAngle[ 2 ] ) },
	{ "VertexWaterXPosition3",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterXPosition[ 2 ] ) },
	{ "VertexWaterYPosition3",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterYPosition[ 2 ] ) },
	{ "VertexWaterZPosition3",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterZPosition[ 2 ] ) },
	{ "VertexWaterXGridCells3",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterXGridCells[ 2 ] ) },
	{ "VertexWaterYGridCells3",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterYGridCells[ 2 ] ) },
	{ "VertexWaterGridSize3",					INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterGridSize[ 2 ] ) },
	{ "VertexWaterAttenuationA3",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationA[ 2 ] ) },
	{ "VertexWaterAttenuationB3",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationB[ 2 ] ) },
	{ "VertexWaterAttenuationC3",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationC[ 2 ] ) },
	{ "VertexWaterAttenuationRange3",	INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationRange[ 2 ] ) },

	// nasty ick, we need to save this data with a map and not hard code INI values
	{ "VertexWaterAvailableMaps4",		INI::parseAsciiString,	nullptr,		offsetof( GlobalData, m_vertexWaterAvailableMaps[ 3 ] ) },
	{ "VertexWaterHeightClampLow4",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampLow[ 3 ] ) },
	{ "VertexWaterHeightClampHi4",		INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterHeightClampHi[ 3 ] ) },
	{ "VertexWaterAngle4",						INI::parseAngleReal,		nullptr,		offsetof( GlobalData, m_vertexWaterAngle[ 3 ] ) },
	{ "VertexWaterXPosition4",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterXPosition[ 3 ] ) },
	{ "VertexWaterYPosition4",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterYPosition[ 3 ] ) },
	{ "VertexWaterZPosition4",				INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterZPosition[ 3 ] ) },
	{ "VertexWaterXGridCells4",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterXGridCells[ 3 ] ) },
	{ "VertexWaterYGridCells4",				INI::parseInt,					nullptr,		offsetof( GlobalData, m_vertexWaterYGridCells[ 3 ] ) },
	{ "VertexWaterGridSize4",					INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterGridSize[ 3 ] ) },
	{ "VertexWaterAttenuationA4",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationA[ 3 ] ) },
	{ "VertexWaterAttenuationB4",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationB[ 3 ] ) },
	{ "VertexWaterAttenuationC4",			INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationC[ 3 ] ) },
	{ "VertexWaterAttenuationRange4",	INI::parseReal,					nullptr,		offsetof( GlobalData, m_vertexWaterAttenuationRange[ 3 ] ) },

	{ "SkyBoxPositionZ",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_skyBoxPositionZ ) },
	{ "SkyBoxScale",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_skyBoxScale ) },
	{ "DrawSkyBox",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_drawSkyBox ) },
	{ "ViewportHeightScale", INI::parseReal, nullptr, offsetof( GlobalData, m_viewportHeightScale ) },
	{ "CameraPitch",								INI::parseReal,				nullptr,			offsetof( GlobalData, m_cameraPitch ) },
	{ "CameraYaw",									INI::parseReal,				nullptr,			offsetof( GlobalData, m_cameraYaw ) },
#if PRESERVE_RETAIL_SCRIPTED_CAMERA
	{ "CameraHeight",								INI::parseReal,				nullptr,			offsetof( GlobalData, m_cameraHeight ) },
#endif
	{ "MaxCameraHeight",						INI::parseReal,				nullptr,			offsetof( GlobalData, m_maxCameraHeight ) },
	{ "MinCameraHeight",						INI::parseReal,				nullptr,			offsetof( GlobalData, m_minCameraHeight ) },
	{ "TerrainDrawDistanceScale",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_terrainDrawDistanceScale ) },
	{ "TerrainHeightAtEdgeOfMap",					INI::parseReal,				nullptr,			offsetof( GlobalData, m_terrainHeightAtEdgeOfMap ) },
	{ "UnitDamagedThreshold",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_unitDamagedThresh ) },
	{ "UnitReallyDamagedThreshold",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_unitReallyDamagedThresh ) },
	{ "GroundStiffness",					INI::parseReal,				nullptr,				offsetof( GlobalData, m_groundStiffness ) },
	{ "StructureStiffness",					INI::parseReal,				nullptr,				offsetof( GlobalData, m_structureStiffness ) },
	{ "Gravity",									INI::parseAccelerationReal,				nullptr,				offsetof( GlobalData, m_gravity ) },
	{ "StealthFriendlyOpacity",		INI::parsePercentToReal,				nullptr,				offsetof( GlobalData, m_stealthFriendlyOpacity ) },
	{ "DefaultOcclusionDelay",				INI::parseDurationUnsignedInt,				nullptr,			offsetof( GlobalData, m_defaultOcclusionDelay ) },

	{ "PartitionCellSize",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_partitionCellSize ) },

	{ "AmmoPipScaleFactor",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_ammoPipScaleFactor ) },
	{ "ContainerPipScaleFactor",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_containerPipScaleFactor ) },
	{ "AmmoPipWorldOffset",						INI::parseCoord3D,				nullptr,			offsetof( GlobalData, m_ammoPipWorldOffset ) },
	{ "ContainerPipWorldOffset",				INI::parseCoord3D,				nullptr,			offsetof( GlobalData, m_containerPipWorldOffset ) },
	{ "AmmoPipScreenOffset",						INI::parseCoord2D,				nullptr,			offsetof( GlobalData, m_ammoPipScreenOffset ) },
	{ "ContainerPipScreenOffset",				INI::parseCoord2D,				nullptr,			offsetof( GlobalData, m_containerPipScreenOffset ) },

	{ "HistoricDamageLimit",				INI::parseDurationUnsignedInt,				nullptr,			offsetof( GlobalData, m_historicDamageLimit ) },

	{ "MaxTerrainTracks",					INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxTerrainTracks ) },
	{ "TimeOfDay",								INI::parseIndexList,	TimeOfDayNames,			offsetof( GlobalData, m_timeOfDay ) },
	{ "Weather",									INI::parseIndexList,	WeatherNames,			offsetof( GlobalData, m_weather ) },
	{ "MakeTrackMarks",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_makeTrackMarks ) },
	{ "HideGarrisonFlags",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_hideGarrisonFlags ) },
	{ "ForceModelsToFollowTimeOfDay",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_forceModelsToFollowTimeOfDay ) },
	{ "ForceModelsToFollowWeather",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_forceModelsToFollowWeather ) },

	{ "LevelGainAnimationName",		INI::parseAsciiString,	nullptr,	offsetof( GlobalData, m_levelGainAnimationName ) },
	{ "LevelGainAnimationTime",		INI::parseReal,					nullptr, offsetof( GlobalData, m_levelGainAnimationDisplayTimeInSeconds ) },
	{ "LevelGainAnimationZRise",	INI::parseReal,					nullptr, offsetof( GlobalData, m_levelGainAnimationZRisePerSecond ) },

	{ "GetHealedAnimationName",		INI::parseAsciiString,	nullptr,	offsetof( GlobalData, m_getHealedAnimationName ) },
	{ "GetHealedAnimationTime",		INI::parseReal,					nullptr, offsetof( GlobalData, m_getHealedAnimationDisplayTimeInSeconds ) },
	{ "GetHealedAnimationZRise",	INI::parseReal,					nullptr, offsetof( GlobalData, m_getHealedAnimationZRisePerSecond ) },

	{ "TerrainLightingMorningAmbient",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][0].ambient ) },
	{ "TerrainLightingMorningDiffuse",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][0].diffuse ) },
	{ "TerrainLightingMorningLightPos",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][0].lightPos ) },
	{ "TerrainLightingAfternoonAmbient",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][0].ambient ) },
	{ "TerrainLightingAfternoonDiffuse",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][0].diffuse ) },
	{ "TerrainLightingAfternoonLightPos",	INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][0].lightPos ) },
	{ "TerrainLightingEveningAmbient",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][0].ambient ) },
	{ "TerrainLightingEveningDiffuse",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][0].diffuse ) },
	{ "TerrainLightingEveningLightPos",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][0].lightPos ) },
	{ "TerrainLightingNightAmbient",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][0].ambient ) },
	{ "TerrainLightingNightDiffuse",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][0].diffuse ) },
	{ "TerrainLightingNightLightPos",			INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][0].lightPos ) },

	{ "TerrainObjectsLightingMorningAmbient",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][0].ambient ) },
	{ "TerrainObjectsLightingMorningDiffuse",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][0].diffuse ) },
	{ "TerrainObjectsLightingMorningLightPos",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][0].lightPos ) },
	{ "TerrainObjectsLightingAfternoonAmbient",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][0].ambient ) },
	{ "TerrainObjectsLightingAfternoonDiffuse",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][0].diffuse ) },
	{ "TerrainObjectsLightingAfternoonLightPos",	INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][0].lightPos ) },
	{ "TerrainObjectsLightingEveningAmbient",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][0].ambient ) },
	{ "TerrainObjectsLightingEveningDiffuse",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][0].diffuse ) },
	{ "TerrainObjectsLightingEveningLightPos",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][0].lightPos ) },
	{ "TerrainObjectsLightingNightAmbient",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][0].ambient ) },
	{ "TerrainObjectsLightingNightDiffuse",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][0].diffuse ) },
	{ "TerrainObjectsLightingNightLightPos",			INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][0].lightPos ) },

	//Secondary global light
	{ "TerrainLightingMorningAmbient2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][1].ambient ) },
	{ "TerrainLightingMorningDiffuse2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][1].diffuse ) },
	{ "TerrainLightingMorningLightPos2",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][1].lightPos ) },
	{ "TerrainLightingAfternoonAmbient2",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][1].ambient ) },
	{ "TerrainLightingAfternoonDiffuse2",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][1].diffuse ) },
	{ "TerrainLightingAfternoonLightPos2",	INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][1].lightPos ) },
	{ "TerrainLightingEveningAmbient2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][1].ambient ) },
	{ "TerrainLightingEveningDiffuse2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][1].diffuse ) },
	{ "TerrainLightingEveningLightPos2",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][1].lightPos ) },
	{ "TerrainLightingNightAmbient2",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][1].ambient ) },
	{ "TerrainLightingNightDiffuse2",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][1].diffuse ) },
	{ "TerrainLightingNightLightPos2",			INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][1].lightPos ) },

	{ "TerrainObjectsLightingMorningAmbient2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][1].ambient ) },
	{ "TerrainObjectsLightingMorningDiffuse2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][1].diffuse ) },
	{ "TerrainObjectsLightingMorningLightPos2",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][1].lightPos ) },
	{ "TerrainObjectsLightingAfternoonAmbient2",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][1].ambient ) },
	{ "TerrainObjectsLightingAfternoonDiffuse2",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][1].diffuse ) },
	{ "TerrainObjectsLightingAfternoonLightPos2",	INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][1].lightPos ) },
	{ "TerrainObjectsLightingEveningAmbient2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][1].ambient ) },
	{ "TerrainObjectsLightingEveningDiffuse2",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][1].diffuse ) },
	{ "TerrainObjectsLightingEveningLightPos2",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][1].lightPos ) },
	{ "TerrainObjectsLightingNightAmbient2",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][1].ambient ) },
	{ "TerrainObjectsLightingNightDiffuse2",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][1].diffuse ) },
	{ "TerrainObjectsLightingNightLightPos2",			INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][1].lightPos ) },

	//Third global light
	{ "TerrainLightingMorningAmbient3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][2].ambient ) },
	{ "TerrainLightingMorningDiffuse3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][2].diffuse ) },
	{ "TerrainLightingMorningLightPos3",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_MORNING ][2].lightPos ) },
	{ "TerrainLightingAfternoonAmbient3",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][2].ambient ) },
	{ "TerrainLightingAfternoonDiffuse3",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][2].diffuse ) },
	{ "TerrainLightingAfternoonLightPos3",	INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_AFTERNOON ][2].lightPos ) },
	{ "TerrainLightingEveningAmbient3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][2].ambient ) },
	{ "TerrainLightingEveningDiffuse3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][2].diffuse ) },
	{ "TerrainLightingEveningLightPos3",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_EVENING ][2].lightPos ) },
	{ "TerrainLightingNightAmbient3",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][2].ambient ) },
	{ "TerrainLightingNightDiffuse3",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][2].diffuse ) },
	{ "TerrainLightingNightLightPos3",			INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainLighting[ TIME_OF_DAY_NIGHT ][2].lightPos ) },

	{ "TerrainObjectsLightingMorningAmbient3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][2].ambient ) },
	{ "TerrainObjectsLightingMorningDiffuse3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][2].diffuse ) },
	{ "TerrainObjectsLightingMorningLightPos3",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_MORNING ][2].lightPos ) },
	{ "TerrainObjectsLightingAfternoonAmbient3",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][2].ambient ) },
	{ "TerrainObjectsLightingAfternoonDiffuse3",		INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][2].diffuse ) },
	{ "TerrainObjectsLightingAfternoonLightPos3",	INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_AFTERNOON ][2].lightPos ) },
	{ "TerrainObjectsLightingEveningAmbient3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][2].ambient ) },
	{ "TerrainObjectsLightingEveningDiffuse3",			INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][2].diffuse ) },
	{ "TerrainObjectsLightingEveningLightPos3",		INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_EVENING ][2].lightPos ) },
	{ "TerrainObjectsLightingNightAmbient3",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][2].ambient ) },
	{ "TerrainObjectsLightingNightDiffuse3",				INI::parseRGBColor,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][2].diffuse ) },
	{ "TerrainObjectsLightingNightLightPos3",			INI::parseCoord3D,			nullptr,			offsetof( GlobalData, m_terrainObjectsLighting[ TIME_OF_DAY_NIGHT ][2].lightPos ) },


	{ "NumberGlobalLights",								INI::parseInt,				nullptr,			offsetof( GlobalData, m_numGlobalLights)},
	{ "InfantryLightMorningScale",				INI::parseReal,			nullptr,			offsetof( GlobalData, m_infantryLightScale[TIME_OF_DAY_MORNING] ) },
	{ "InfantryLightAfternoonScale",				INI::parseReal,			nullptr,			offsetof( GlobalData, m_infantryLightScale[TIME_OF_DAY_AFTERNOON] ) },
	{ "InfantryLightEveningScale",				INI::parseReal,			nullptr,			offsetof( GlobalData, m_infantryLightScale[TIME_OF_DAY_EVENING] ) },
	{ "InfantryLightNightScale",				INI::parseReal,			nullptr,			offsetof( GlobalData, m_infantryLightScale[TIME_OF_DAY_NIGHT] ) },

	{ "MaxTranslucentObjects",						INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxVisibleTranslucentObjects) },
	{ "OccludedColorLuminanceScale",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_occludedLuminanceScale) },

/* These are internal use only, they do not need file definitions
	{ "TerrainAmbientRGB",				INI::parseRGBColor,		nullptr,			offsetof( GlobalData, m_terrainAmbient ) },
	{ "TerrainDiffuseRGB",				INI::parseRGBColor,		nullptr,			offsetof( GlobalData, m_terrainDiffuse ) },
	{ "TerrainLightPos",					INI::parseCoord3D,		nullptr,			offsetof( GlobalData, m_terrainLightPos ) },
*/
	{ "MaxRoadSegments",						INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxRoadSegments ) },
	{ "MaxRoadVertex",							INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxRoadVertex ) },
	{ "MaxRoadIndex",								INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxRoadIndex ) },
	{ "MaxRoadTypes",								INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxRoadTypes ) },

	{ "ValuePerSupplyBox",					INI::parseInt,				nullptr,			offsetof( GlobalData, m_baseValuePerSupplyBox ) },

	{ "AudioOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_audioOn ) },
	{ "MusicOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_musicOn ) },
	{ "SoundsOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_soundsOn ) },
	{ "Sounds3DOn",									INI::parseBool,				nullptr,			offsetof( GlobalData, m_sounds3DOn ) },
	{ "SpeechOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_speechOn ) },
	{ "VideoOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_videoOn ) },
	{ "DisableCameraMovements",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_disableCameraMovement ) },

/* These are internal use only, they do not need file definitions
	/// @todo remove this hack
	{ "InGame",											INI::parseBool,				nullptr,			offsetof( GlobalData, m_inGame ) },
*/

	{ "DebugAI",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_debugAI ) },
	{ "DebugAIObstacles",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_debugAIObstacles ) },
	{ "ShowClientPhysics",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_showClientPhysics ) },
	{ "ShowTerrainNormals",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_showTerrainNormals ) },
	{ "ShowObjectHealth",						INI::parseBool,				nullptr,			offsetof( GlobalData, m_showObjectHealth ) },

	{ "ParticleScale",										INI::parseReal,					nullptr,	 offsetof( GlobalData, m_particleScale ) },
	{ "AutoFireParticleSmallPrefix",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoFireParticleSmallPrefix ) },
	{ "AutoFireParticleSmallSystem",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoFireParticleSmallSystem ) },
	{ "AutoFireParticleSmallMax",					INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoFireParticleSmallMax ) },
	{ "AutoFireParticleMediumPrefix",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoFireParticleMediumPrefix ) },
	{ "AutoFireParticleMediumSystem",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoFireParticleMediumSystem ) },
	{ "AutoFireParticleMediumMax",				INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoFireParticleMediumMax ) },
	{ "AutoFireParticleLargePrefix",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoFireParticleLargePrefix ) },
	{ "AutoFireParticleLargeSystem",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoFireParticleLargeSystem ) },
	{ "AutoFireParticleLargeMax",					INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoFireParticleLargeMax ) },
	{ "AutoSmokeParticleSmallPrefix",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoSmokeParticleSmallPrefix ) },
	{ "AutoSmokeParticleSmallSystem",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoSmokeParticleSmallSystem ) },
	{ "AutoSmokeParticleSmallMax",				INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoSmokeParticleSmallMax ) },
	{ "AutoSmokeParticleMediumPrefix",		INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoSmokeParticleMediumPrefix ) },
	{ "AutoSmokeParticleMediumSystem",		INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoSmokeParticleMediumSystem ) },
	{ "AutoSmokeParticleMediumMax",				INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoSmokeParticleMediumMax ) },
	{ "AutoSmokeParticleLargePrefix",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoSmokeParticleLargePrefix ) },
	{ "AutoSmokeParticleLargeSystem",			INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoSmokeParticleLargeSystem ) },
	{ "AutoSmokeParticleLargeMax",				INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoSmokeParticleLargeMax ) },
	{ "AutoAflameParticlePrefix",					INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoAflameParticlePrefix ) },
	{ "AutoAflameParticleSystem",					INI::parseAsciiString,  nullptr,  offsetof( GlobalData, m_autoAflameParticleSystem ) },
	{ "AutoAflameParticleMax",						INI::parseInt,					nullptr,	 offsetof( GlobalData, m_autoAflameParticleMax ) },

/* These are internal use only, they do not need file definitions
	{ "LatencyAverage",							INI::parseInt,				nullptr,			offsetof( GlobalData, m_latencyAverage ) },
	{ "LatencyAmplitude",						INI::parseInt,				nullptr,			offsetof( GlobalData, m_latencyAmplitude ) },
	{ "LatencyPeriod",							INI::parseInt,				nullptr,			offsetof( GlobalData, m_latencyPeriod ) },
	{ "LatencyNoise",								INI::parseInt,				nullptr,			offsetof( GlobalData, m_latencyNoise ) },
	{ "PacketLoss",									INI::parseInt,				nullptr,			offsetof( GlobalData, m_packetLoss ) },
*/

	{ "BuildSpeed",									INI::parseReal,				nullptr,			offsetof( GlobalData, m_BuildSpeed ) },
	{ "MinDistFromEdgeOfMapForBuild",	 INI::parseReal,				nullptr,			offsetof( GlobalData, m_MinDistFromEdgeOfMapForBuild ) },
	{ "SupplyBuildBorder",	 INI::parseReal,				nullptr,			offsetof( GlobalData, m_SupplyBuildBorder ) },
	{ "AllowedHeightVariationForBuilding", INI::parseReal,nullptr,			offsetof( GlobalData, m_allowedHeightVariationForBuilding ) },
	{ "MinLowEnergyProductionSpeed",INI::parseReal,				nullptr,			offsetof( GlobalData, m_MinLowEnergyProductionSpeed ) },
	{ "MaxLowEnergyProductionSpeed",INI::parseReal,				nullptr,			offsetof( GlobalData, m_MaxLowEnergyProductionSpeed ) },
	{ "LowEnergyPenaltyModifier",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_LowEnergyPenaltyModifier ) },
	{ "MultipleFactory",						INI::parseReal,				nullptr,			offsetof( GlobalData, m_MultipleFactory ) },
	{ "RefundPercent",							INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_RefundPercent ) },

	{ "CommandCenterHealRange",			INI::parseReal,				nullptr,			offsetof( GlobalData, m_commandCenterHealRange ) },
	{ "CommandCenterHealAmount",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_commandCenterHealAmount ) },

	{ "StandardMinefieldDensity",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_standardMinefieldDensity ) },
	{ "StandardMinefieldDistance",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_standardMinefieldDistance ) },

	{ "MaxLineBuildObjects",				INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxLineBuildObjects ) },
	{ "MaxTunnelCapacity",					INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxTunnelCapacity ) },

	{ "MaxParticleCount",						INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxParticleCount ) },
	{ "MaxFieldParticleCount",						INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxFieldParticleCount ) },
	{ "HorizontalScrollSpeedFactor",INI::parseReal,				nullptr,			offsetof( GlobalData, m_horizontalScrollSpeedFactor ) },
	{ "VerticalScrollSpeedFactor",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_verticalScrollSpeedFactor ) },
	{ "ScrollAmountCutoff",					INI::parseReal,				nullptr,			offsetof( GlobalData, m_scrollAmountCutoff ) },
	{ "CameraAdjustSpeed",					INI::parseReal,				nullptr,			offsetof( GlobalData, m_cameraAdjustSpeed ) },
	{ "EnforceMaxCameraHeight",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_enforceMaxCameraHeight ) },
	{ "KeyboardScrollSpeedFactor",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_keyboardScrollFactor ) },
	// GeneralsX @feature Gabriel Petry 30/09/2026 Load configurable mouse scrolling speed.
	{ "MouseScrollSpeedFactor",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_mouseScrollFactor ) },
	{ "KeyboardDefaultScrollSpeedFactor",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_keyboardDefaultScrollFactor ) },
	{ "KeyboardCameraRotateSpeed", INI::parseReal, nullptr, offsetof( GlobalData, m_keyboardCameraRotateSpeed ) },
	{ "MovementPenaltyDamageState",	INI::parseIndexList,	TheBodyDamageTypeNames,	 offsetof( GlobalData, m_movementPenaltyDamageState ) },

// you cannot set this; it always has a value of 100%.
//{ "HealthBonus_Regular",				INI::parsePercentToReal, nullptr,	offsetof( GlobalData, m_healthBonus[LEVEL_REGULAR]) },
	{ "HealthBonus_Veteran",				INI::parsePercentToReal, nullptr,	offsetof( GlobalData, m_healthBonus[LEVEL_VETERAN]) },
	{ "HealthBonus_Elite",					INI::parsePercentToReal, nullptr,	offsetof( GlobalData, m_healthBonus[LEVEL_ELITE]) },
	{ "HealthBonus_Heroic",					INI::parsePercentToReal, nullptr,	offsetof( GlobalData, m_healthBonus[LEVEL_HEROIC]) },

	{ "HumanSoloPlayerHealthBonus_Easy",					INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_soloPlayerHealthBonusForDifficulty[PLAYER_HUMAN][DIFFICULTY_EASY] ) },
	{ "HumanSoloPlayerHealthBonus_Normal",				INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_soloPlayerHealthBonusForDifficulty[PLAYER_HUMAN][DIFFICULTY_NORMAL] ) },
	{ "HumanSoloPlayerHealthBonus_Hard",				INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_soloPlayerHealthBonusForDifficulty[PLAYER_HUMAN][DIFFICULTY_HARD] ) },

	{ "AISoloPlayerHealthBonus_Easy",					INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_soloPlayerHealthBonusForDifficulty[PLAYER_COMPUTER][DIFFICULTY_EASY] ) },
	{ "AISoloPlayerHealthBonus_Normal",				INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_soloPlayerHealthBonusForDifficulty[PLAYER_COMPUTER][DIFFICULTY_NORMAL] ) },
	{ "AISoloPlayerHealthBonus_Hard",				INI::parsePercentToReal,			nullptr,			offsetof( GlobalData, m_soloPlayerHealthBonusForDifficulty[PLAYER_COMPUTER][DIFFICULTY_HARD] ) },

	{ "WeaponBonus",								WeaponBonusSet::parseWeaponBonusSetPtr,	nullptr,	offsetof( GlobalData, m_weaponBonusSet ) },

	{ "DefaultStructureRubbleHeight",	INI::parseReal,			nullptr,			offsetof( GlobalData, m_defaultStructureRubbleHeight ) },

	{ "FixedSeed",									INI::parseInt,				nullptr,			offsetof( GlobalData, m_fixedSeed ) },

	{ "ShellMapName",								INI::parseAsciiString,nullptr,			offsetof( GlobalData, m_shellMapName ) },
	{ "ShellMapOn",									INI::parseBool,				nullptr,			offsetof( GlobalData, m_shellMapOn ) },
	{	"PlayIntro",									INI::parseBool,				nullptr,			offsetof( GlobalData, m_playIntro ) },

	{ "FirewallBehavior",						INI::parseInt,				nullptr,			offsetof( GlobalData, m_firewallBehavior ) },
	{ "FirewallPortOverride",				INI::parseInt,				nullptr,			offsetof( GlobalData, m_firewallPortOverride ) },
	{	"FirewallPortAllocationDelta",INI::parseInt,				nullptr,			offsetof( GlobalData, m_firewallPortAllocationDelta) },

	{	"GroupSelectMinSelectSize",		INI::parseInt,				nullptr,			offsetof( GlobalData, m_groupSelectMinSelectSize ) },
	{	"GroupSelectVolumeBase",			INI::parseReal,				nullptr,			offsetof( GlobalData, m_groupSelectVolumeBase ) },
	{	"GroupSelectVolumeIncrement",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_groupSelectVolumeIncrement ) },
	{	"MaxUnitSelectSounds",				INI::parseInt,				nullptr,			offsetof( GlobalData, m_maxUnitSelectSounds ) },

	{	"SelectionFlashSaturationFactor",	INI::parseReal,		nullptr,			offsetof( GlobalData, m_selectionFlashSaturationFactor ) },
	{	"SelectionFlashHouseColor",	      INI::parseBool,		nullptr,			offsetof( GlobalData, m_selectionFlashHouseColor ) },

	{	"CameraAudibleRadius",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_cameraAudibleRadius ) },
	{ "GroupMoveClickToGatherAreaFactor", INI::parseReal,	nullptr,			offsetof( GlobalData, m_groupMoveClickToGatherFactor ) },

#if ALLOW_MONEY_PER_MINUTE_FOR_PLAYER
	{ "AllowMoneyPerMinuteForPlayer",	INI::parseBool,			nullptr,			offsetof( GlobalData, m_allowMoneyPerMinuteForPlayer ) },
#endif

	{ "ShakeSubtleIntensity",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_shakeSubtleIntensity ) },
	{ "ShakeNormalIntensity",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_shakeNormalIntensity ) },
	{ "ShakeStrongIntensity",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_shakeStrongIntensity ) },
	{ "ShakeSevereIntensity",				INI::parseReal,				nullptr,			offsetof( GlobalData, m_shakeSevereIntensity ) },
	{ "ShakeCineExtremeIntensity",	INI::parseReal,				nullptr,			offsetof( GlobalData, m_shakeCineExtremeIntensity ) },
	{ "ShakeCineInsaneIntensity",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_shakeCineInsaneIntensity ) },
	{ "MaxShakeIntensity",					INI::parseReal,				nullptr,			offsetof( GlobalData, m_maxShakeIntensity ) },
	{ "MaxShakeRange",							INI::parseReal,				nullptr,			offsetof( GlobalData, m_maxShakeRange) },
	{ "SellPercentage",							INI::parsePercentToReal,	nullptr,			offsetof( GlobalData, m_sellPercentage ) },
	{ "BaseRegenHealthPercentPerSecond", INI::parsePercentToReal, nullptr,	offsetof( GlobalData, m_baseRegenHealthPercentPerSecond ) },
	{ "BaseRegenDelay",							INI::parseDurationUnsignedInt, nullptr,offsetof( GlobalData, m_baseRegenDelay ) },

#ifdef ALLOW_SURRENDER
	{ "PrisonBountyMultiplier",			INI::parseReal,				nullptr,			offsetof( GlobalData, m_prisonBountyMultiplier ) },
	{ "PrisonBountyTextColor",			INI::parseColorInt,		nullptr,			offsetof( GlobalData, m_prisonBountyTextColor ) },
#endif

	{ "SpecialPowerViewObject",			INI::parseAsciiString,	nullptr,			offsetof( GlobalData, m_specialPowerViewObjectName ) },

	// TheSuperHackers @feature Customize the opacity (0..1) and shadows of build preview objects. Shadows are enabled by default.
	// Note that disabling shadows loses a fair bit of contrast visually and warrants raising the opacity.
	{ "ObjectPlacementOpacity", INI::parseReal, nullptr, offsetof( GlobalData, m_objectPlacementOpacity ) },
	{ "ObjectPlacementShadows", INI::parseBool, nullptr, offsetof( GlobalData, m_objectPlacementShadows ) },

	{ "StandardPublicBone", INI::parseAsciiStringVectorAppend, nullptr, offsetof(GlobalData, m_standardPublicBones) },
	{ "ShowMetrics",								INI::parseBool,				   nullptr,		offsetof( GlobalData, m_showMetrics ) },
  { "DefaultStartingCash",				Money::parseMoneyAmount, nullptr,		offsetof( GlobalData, m_defaultStartingCash ) },

// NOTE: m_doubleClickTimeMS is still in use, but we disallow setting it from the GameData.ini file. It is now set in the constructor according to the windows parameter.
//	{ "DoubleClickTimeMS",									INI::parseUnsignedInt,			nullptr, offsetof( GlobalData, m_doubleClickTimeMS ) },

	{ "ShroudColor",		INI::parseRGBColor,						nullptr,	offsetof( GlobalData, m_shroudColor) },
	{ "ClearAlpha",			INI::parseUnsignedByte,				nullptr,	offsetof( GlobalData, m_clearAlpha) },
	{ "FogAlpha",				INI::parseUnsignedByte,				nullptr,	offsetof( GlobalData, m_fogAlpha) },
	{ "ShroudAlpha",		INI::parseUnsignedByte,				nullptr,	offsetof( GlobalData, m_shroudAlpha) },

	{ "HotKeyTextColor",										INI::parseColorInt,					nullptr,	offsetof( GlobalData, m_hotKeyTextColor ) },

	{ "PowerBarBase",												INI::parseInt,							nullptr,	offsetof( GlobalData, m_powerBarBase) },
	{ "PowerBarIntervals",									INI::parseReal,							nullptr,	offsetof( GlobalData, m_powerBarIntervals) },
	{ "PowerBarYellowRange",								INI::parseInt,							nullptr,	offsetof( GlobalData, m_powerBarYellowRange) },
	{ "UnlookPersistDuration",							INI::parseDurationUnsignedInt, nullptr, offsetof( GlobalData, m_unlookPersistDuration) },

	{ "NetworkFPSHistoryLength", INI::parseInt, nullptr, offsetof(GlobalData, m_networkFPSHistoryLength) },
	{ "NetworkLatencyHistoryLength", INI::parseInt, nullptr, offsetof(GlobalData, m_networkLatencyHistoryLength) },
	{ "NetworkRunAheadMetricsTime", INI::parseInt, nullptr, offsetof(GlobalData, m_networkRunAheadMetricsTime) },
	{ "NetworkCushionHistoryLength", INI::parseInt, nullptr, offsetof(GlobalData, m_networkCushionHistoryLength) },
	{ "NetworkRunAheadSlack", INI::parseInt, nullptr, offsetof(GlobalData, m_networkRunAheadSlack) },
	{ "NetworkKeepAliveDelay", INI::parseInt, nullptr, offsetof(GlobalData, m_networkKeepAliveDelay) },
	{ "NetworkDisconnectTime", INI::parseInt, nullptr, offsetof(GlobalData, m_networkDisconnectTime) },
	{ "NetworkPlayerTimeoutTime", INI::parseInt, nullptr, offsetof(GlobalData, m_networkPlayerTimeoutTime) },
	{ "NetworkDisconnectScreenNotifyTime", INI::parseInt, nullptr, offsetof(GlobalData, m_networkDisconnectScreenNotifyTime) },

	{ "PlayStats",									INI::parseInt,				nullptr,			offsetof( GlobalData, m_playStats ) },

#if defined(RTS_DEBUG)
	{ "DisableCameraFade",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_disableCameraFade ) },
	{ "DisableScriptedInputDisabling",			INI::parseBool,		nullptr,			offsetof( GlobalData, m_disableScriptedInputDisabling ) },
	{ "DisableMilitaryCaption",			INI::parseBool,				nullptr,			offsetof( GlobalData, m_disableMilitaryCaption ) },
	{ "BenchmarkTimer",			INI::parseInt,				nullptr,			offsetof( GlobalData, m_benchmarkTimer ) },
	{ "CheckMemoryLeaks", INI::parseBool, nullptr, offsetof(GlobalData, m_checkForLeaks) },
	{ "Wireframe",								INI::parseBool,				nullptr,			offsetof( GlobalData, m_wireframe ) },
	{ "StateMachineDebug",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_stateMachineDebug ) },
	{ "UseCameraConstraints",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_useCameraConstraints ) },
	{ "ShroudOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_shroudOn ) },
	{ "FogOfWarOn",										INI::parseBool,				nullptr,			offsetof( GlobalData, m_fogOfWarOn ) },
	{ "ShowCollisionExtents",				INI::parseBool,				nullptr,			offsetof( GlobalData, m_showCollisionExtents ) },
  { "ShowAudioLocations",  				INI::parseBool,				nullptr,			offsetof( GlobalData, m_showAudioLocations ) },
	{ "DebugProjectileTileWidth",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_debugProjectileTileWidth) },
	{ "DebugProjectileTileDuration",INI::parseInt,				nullptr,			offsetof( GlobalData, m_debugProjectileTileDuration) },
	{ "DebugProjectileTileColor",		INI::parseRGBColor,		nullptr,			offsetof( GlobalData, m_debugProjectileTileColor) },
	{ "DebugVisibilityTileCount",		INI::parseInt,				nullptr,			offsetof( GlobalData, m_debugVisibilityTileCount) },
	{ "DebugVisibilityTileWidth",		INI::parseReal,				nullptr,			offsetof( GlobalData, m_debugVisibilityTileWidth) },
	{ "DebugVisibilityTileDuration",INI::parseInt,				nullptr,			offsetof( GlobalData, m_debugVisibilityTileDuration) },
	{ "DebugVisibilityTileTargettableColor",INI::parseRGBColor, nullptr,	offsetof( GlobalData, m_debugVisibilityTargettableColor) },
	{ "DebugVisibilityTileDeshroudColor",		INI::parseRGBColor,	nullptr,	offsetof( GlobalData, m_debugVisibilityDeshroudColor) },
	{ "DebugVisibilityTileGapColor",				INI::parseRGBColor,	nullptr,	offsetof( GlobalData, m_debugVisibilityGapColor) },
	{ "DebugThreatMapTileDuration",					INI::parseInt,							nullptr,	offsetof( GlobalData, m_debugThreatMapTileDuration) },
	{ "MaxDebugThreatMapValue",							INI::parseUnsignedInt,			nullptr,	offsetof( GlobalData, m_maxDebugThreat) },
	{ "DebugCashValueMapTileDuration",			INI::parseInt,							nullptr,	offsetof( GlobalData, m_debugCashValueMapTileDuration) },
	{ "MaxDebugCashValueMapValue",					INI::parseUnsignedInt,			nullptr,	offsetof( GlobalData, m_maxDebugValue) },
	{ "VTune", INI::parseBool,	nullptr,			offsetof( GlobalData, m_vTune ) },
	{ "SaveStats",									INI::parseBool,				nullptr,			offsetof( GlobalData, m_saveStats ) },
	{ "UseLocalMOTD",								INI::parseBool,				nullptr,			offsetof( GlobalData, m_useLocalMOTD ) },
	{ "BaseStatsDir",								INI::parseAsciiString,nullptr,			offsetof( GlobalData, m_baseStatsDir ) },
	{ "LocalMOTDPath",							INI::parseAsciiString,nullptr,			offsetof( GlobalData, m_MOTDPath ) },
	{ "ExtraLogging",								INI::parseBool,				nullptr,			offsetof( GlobalData, m_extraLogging ) },
#endif

	{ nullptr,					nullptr,						nullptr,						0 }

};


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GlobalData::GlobalData()
{
	Int i, j;

	//
	// we have now instanced a global data instance, if theOriginal is null, this is
	// *the* very first instance and it shall be recorded.  This way, when we load
	// overrides of the global data, we can revert to the common, original data
	// in m_theOriginal
	//
	if( m_theOriginal == nullptr )
		m_theOriginal = this;
	m_next = nullptr;

#if defined(RTS_DEBUG) || defined(_ALLOW_DEBUG_CHEATS_IN_RELEASE)
	m_specialPowerUsesDelay = TRUE;
#endif
  m_TiVOFastMode = FALSE;

#if defined(RTS_DEBUG) || ENABLE_CONFIGURABLE_SHROUD
	m_shroudOn = TRUE;
#endif

#if defined(RTS_DEBUG)
	m_wireframe = 0;
	m_stateMachineDebug = FALSE;
	m_useCameraConstraints = TRUE;
	m_fogOfWarOn = FALSE;
	m_jabberOn = FALSE;
	m_munkeeOn = FALSE;
	m_showCollisionExtents = FALSE;
  m_showAudioLocations = FALSE;
	m_debugCamera = FALSE;
	m_debugVisibility = FALSE;
	m_debugVisibilityTileCount = 32;	// default to 32.
	m_debugVisibilityTileDuration = LOGICFRAMES_PER_SECOND;
	m_debugProjectilePath = FALSE;
	m_debugProjectileTileWidth = 10;
	m_debugProjectileTileDuration = LOGICFRAMES_PER_SECOND;
	m_debugThreatMap = FALSE;
	m_maxDebugThreat = 5000;
	m_debugThreatMapTileDuration = LOGICFRAMES_PER_SECOND;
	m_debugCashValueMap = FALSE;
	m_maxDebugValue = 10000;
	m_debugCashValueMapTileDuration = LOGICFRAMES_PER_SECOND;
	m_vTune = false;
	m_checkForLeaks = TRUE;
	m_benchmarkTimer = -1;


	m_allowUnselectableSelection = FALSE;
	m_disableCameraFade = false;
	m_disableScriptedInputDisabling = false;
	m_disableMilitaryCaption = false;
	m_latencyAverage = 0;
	m_latencyAmplitude = 0;
	m_latencyPeriod = 0;
	m_latencyNoise = 0;
	m_packetLoss = 0;
	m_saveStats = FALSE;
	m_saveAllStats = FALSE;
	m_useLocalMOTD = FALSE;
	m_baseStatsDir = ".\\";
	m_MOTDPath = "MOTD.txt";
	m_extraLogging = FALSE;
#endif

#ifdef DEBUG_CRASHING
	m_debugIgnoreAsserts = FALSE;
#endif

#ifdef DEBUG_STACKTRACE
	m_debugIgnoreStackTrace = FALSE;
#endif

	m_playStats = -1;
	m_incrementalAGPBuf = FALSE;
	m_mapName.clear();
	m_moveHintName.clear();
	m_useTrees = 0;
	m_useTreeSway = TRUE;
	m_useDrawModuleLOD = FALSE;
	m_useHeatEffects = TRUE;
	// GeneralsX @tweak felipebraz 20/06/2026 Default render FPS limit to 60 FPS instead of uncapped/0.
	m_useFpsLimit = TRUE;
	m_dumpAssetUsage = FALSE;
	m_framesPerSecondLimit = 60;
	// GeneralsX @feature felipebraz 17/09/2026 Skirmish simulation tick rate configuration (#281)
	m_skirmishTickRate = LOGICFRAMES_PER_SECOND;
	m_chipSetType = 0;
	m_headless = FALSE;
	m_checkForUpdates = TRUE;
	m_windowed = 0;
	m_xResolution = DEFAULT_DISPLAY_WIDTH;
	m_yResolution = DEFAULT_DISPLAY_HEIGHT;
	m_maxShellScreens = 0;
	m_useCloudMap = FALSE;
	m_use3WayTerrainBlends = 1;
	m_useLightMap = FALSE;
	m_bilinearTerrainTex = FALSE;
	m_trilinearTerrainTex = FALSE;
	m_multiPassTerrain = FALSE;
	m_adjustCliffTextures = FALSE;
	m_stretchTerrain = FALSE;
	m_useHalfHeightMap = FALSE;
	m_terrainLOD = TERRAIN_LOD_AUTOMATIC;
	m_terrainLODTargetTimeMS = 0;
	m_enableDynamicLOD = TRUE;
	m_enableStaticLOD = TRUE;
	m_rightMouseAlwaysScrolls = FALSE;
	m_jpegQuality = DEFAULT_JPEG_QUALITY;
	m_useWaterPlane = FALSE;
	m_useCloudPlane = FALSE;
	m_downwindAngle = ( -0.785f );//Northeast!
	m_useShadowVolumes = FALSE;
	m_useShadowVolumesForSkins = TRUE;
	m_shadowsForProjectiles = TRUE;
	m_shadowsForProps = TRUE;
	// GeneralsX @feature fbraz3 08/10/2026 Enhanced graphics & lighting toggleable options defaults
	m_advancedShadows = TRUE;
	m_advancedLighting = TRUE;
	m_extendedShadowFrustumCulling = TRUE;
	m_naturalSunElevation = TRUE;
	m_bridgeShadows = TRUE;
	m_waterReflections = TRUE;
	m_explosionDynamicLights = TRUE;
	m_vehicleHeadlights = TRUE;
	m_enablePostProcessing = TRUE;
	m_postProcessBloom = TRUE;
	m_postProcessHDR = TRUE;
	m_postProcessFXAA = FALSE;
	m_airUnitGroundIndicator = TRUE;
	m_useShadowDecals = FALSE;
	m_textureReductionFactor = -1;
	m_enableBehindBuildingMarkers = TRUE;
	m_scriptDebug = FALSE;
	m_particleEdit = FALSE;
	m_displayDebug = FALSE;
	m_winCursors = TRUE;
	m_constantDebugUpdate = FALSE;
	m_showTeamDot = FALSE;
	m_fixedSeed = -1; // disabled
	m_horizontalScrollSpeedFactor = 1.0;
	m_verticalScrollSpeedFactor = 1.0;

	m_waterPositionX = 0.0f;
	m_waterPositionY = 0.0f;
	m_waterPositionZ = 0.0f;
	m_waterExtentX = 0.0f;
	m_waterExtentY = 0.0f;
	m_waterType = 0;
	m_featherWater = FALSE;
	m_showSoftWaterEdge = TRUE;	//display soft water edge
	m_usingWaterTrackEditor = FALSE;
	m_isWorldBuilder = FALSE;

	m_showMetrics = false;

	for( i = 0; i < MAX_WATER_GRID_SETTINGS; i++ )
	{

		m_vertexWaterHeightClampLow[ i ] = 0.0f;
		m_vertexWaterHeightClampHi[ i ] = 0.0f;
		m_vertexWaterAngle[ i ] = 0.0f;
		m_vertexWaterXPosition[ i ] = 0.0f;
		m_vertexWaterYPosition[ i ] = 0.0f;
		m_vertexWaterZPosition[ i ] = 0.0f;
		m_vertexWaterXGridCells[ i ] = 0;
		m_vertexWaterYGridCells[ i ] = 0;
		m_vertexWaterGridSize[ i ] = 0.0f;
		m_vertexWaterAttenuationA[ i ] = 0.0f;
		m_vertexWaterAttenuationB[ i ] = 0.0f;
		m_vertexWaterAttenuationC[ i ] = 0.0f;
		m_vertexWaterAttenuationRange[ i ] = 0.0f;
		m_vertexWaterAvailableMaps[i].clear();
	}

	m_skyBoxPositionZ = 0.0f;
	m_drawSkyBox = FALSE;
	m_skyBoxScale = 4.5f;

	m_historicDamageLimit = 0;
	m_maxTerrainTracks = 0;

	m_levelGainAnimationDisplayTimeInSeconds = 0.0f;
	m_levelGainAnimationZRisePerSecond = 0.0f;

	m_getHealedAnimationDisplayTimeInSeconds = 0.0f;
	m_getHealedAnimationZRisePerSecond = 0.0f;

	m_maxTankTrackEdges=100;
	m_maxTankTrackOpaqueEdges=25;
	m_maxTankTrackFadeDelay=300000;

	m_timeOfDay = TIME_OF_DAY_AFTERNOON;
	m_weather = WEATHER_NORMAL;
	m_makeTrackMarks = FALSE;
	m_hideGarrisonFlags = FALSE;
	m_forceModelsToFollowTimeOfDay = true;
	m_forceModelsToFollowWeather = true;

	m_partitionCellSize = 0.0f;
	m_ammoPipScaleFactor = 1.0f;
	m_containerPipScaleFactor = 1.0f;
	m_ammoPipWorldOffset.zero();
	m_containerPipWorldOffset.zero();
	m_ammoPipScreenOffset.x = m_ammoPipScreenOffset.y = 0;
	m_containerPipScreenOffset.x = m_containerPipScreenOffset.y = 0;

	for (i=0; i<MAX_GLOBAL_LIGHTS; i++)
	{
		m_terrainAmbient[i].red = 0.0f;
		m_terrainAmbient[i].green = 0.0f;
		m_terrainAmbient[i].blue = 0.0f;
		m_terrainDiffuse[i].red = 0.0f;
		m_terrainDiffuse[i].green = 0.0f;
		m_terrainDiffuse[i].blue = 0.0f;
		m_terrainLightPos[i].x = 0.0f;
		m_terrainLightPos[i].y = 0.0f;
		m_terrainLightPos[i].z = -1.0f;

		for (j=0; j<TIME_OF_DAY_COUNT; j++)
		{	m_terrainLighting[ j ][i].ambient.red=0;
			m_terrainLighting[ j ][i].ambient.green=0;
			m_terrainLighting[ j ][i].ambient.blue=0;
			m_terrainLighting[ j ][i].diffuse.red=0;
			m_terrainLighting[ j ][i].diffuse.green=0;
			m_terrainLighting[ j ][i].diffuse.blue=0;
			m_terrainLighting[ j ][i].lightPos.x=0;
			m_terrainLighting[ j ][i].lightPos.y=0;
			m_terrainLighting[ j ][i].lightPos.z=-1.0f;

			m_terrainObjectsLighting[ j ][i].ambient.red=0;
			m_terrainObjectsLighting[ j ][i].ambient.green=0;
			m_terrainObjectsLighting[ j ][i].ambient.blue=0;
			m_terrainObjectsLighting[ j ][i].diffuse.red=0;
			m_terrainObjectsLighting[ j ][i].diffuse.green=0;
			m_terrainObjectsLighting[ j ][i].diffuse.blue=0;
			m_terrainObjectsLighting[ j ][i].lightPos.x=0;
			m_terrainObjectsLighting[ j ][i].lightPos.y=0;
			m_terrainObjectsLighting[ j ][i].lightPos.z=-1.0f;
		}
	}

	for (j=TIME_OF_DAY_FIRST; j<TIME_OF_DAY_COUNT; j++)
		m_infantryLightScale[j] = 1.5f;

	m_scriptOverrideInfantryLightScale = -1.0f;

	m_numGlobalLights = 3;
	m_maxRoadSegments = 0;
	m_maxRoadVertex = 0;
	m_maxRoadIndex = 0;
	m_maxRoadTypes = 0;

	m_baseValuePerSupplyBox = 100;

	m_audioOn = TRUE;
	m_musicOn = TRUE;
	m_soundsOn = TRUE;
	m_sounds3DOn = TRUE;
	m_speechOn = TRUE;
	m_videoOn = TRUE;
	m_disableCameraMovement = FALSE;
	m_maxVisibleTranslucentObjects = 512;
	m_maxVisibleOccluderObjects = 512;
	m_maxVisibleOccludeeObjects = 512;
	m_maxVisibleNonOccluderOrOccludeeObjects = 512;
	m_occludedLuminanceScale = 0.5f;

	m_useFX = TRUE;

//	m_inGame = FALSE;

	m_noDraw = 0;
	m_particleScale = 1.0f;

	m_autoFireParticleSmallMax = 0;
	m_autoFireParticleMediumMax = 0;
	m_autoFireParticleLargeMax = 0;
	m_autoSmokeParticleSmallMax = 0;
	m_autoSmokeParticleMediumMax = 0;
	m_autoSmokeParticleLargeMax = 0;
	m_autoAflameParticleMax = 0;

	m_autoFireParticleSmallPrefix.clear();
	m_autoFireParticleMediumPrefix.clear();
	m_autoFireParticleLargePrefix.clear();
	m_autoSmokeParticleSmallPrefix.clear();
	m_autoSmokeParticleMediumPrefix.clear();
	m_autoSmokeParticleLargePrefix.clear();
	m_autoAflameParticlePrefix.clear();

	m_autoFireParticleSmallSystem.clear();
	m_autoFireParticleMediumSystem.clear();
	m_autoFireParticleLargeSystem.clear();
	m_autoSmokeParticleSmallSystem.clear();
	m_autoSmokeParticleMediumSystem.clear();
	m_autoSmokeParticleLargeSystem.clear();
	m_autoAflameParticleSystem.clear();
	m_levelGainAnimationName.clear();
	m_getHealedAnimationName.clear();
	m_specialPowerViewObjectName.clear();

	m_drawEntireTerrain = FALSE;
	m_maxParticleCount = 0;
	m_maxFieldParticleCount = 30;

	m_debugAI = AI_DEBUG_NONE;
	m_debugSupplyCenterPlacement = FALSE;
	m_debugAIObstacles = FALSE;
	m_showClientPhysics = TRUE;
	m_showTerrainNormals = FALSE;
	m_showObjectHealth = FALSE;

	m_particleEdit = FALSE;

	m_viewportHeightScale = 0.80f; // Default value for the original Control Bar.

	m_cameraPitch = 0.0f;
	m_cameraYaw = 0.0f;
#if PRESERVE_RETAIL_SCRIPTED_CAMERA
	m_cameraHeight = 0.0f;
#endif
	m_minCameraHeight = 100.0f;
	m_maxCameraHeight = 300.0f;
	m_terrainDrawDistanceScale = 1.0f;
	m_terrainHeightAtEdgeOfMap = 0.0f;

	m_unitDamagedThresh = 0.5f;
	m_unitReallyDamagedThresh = 0.1f;
	m_groundStiffness = 0.5f;
	m_structureStiffness = 0.5f;
	m_gravity = -1.0f;
	m_stealthFriendlyOpacity = 0.5f;
	m_defaultOcclusionDelay = LOGICFRAMES_PER_SECOND * 3;	//default to 3 seconds

	m_preloadAssets = FALSE;
	m_preloadEverything = FALSE;
	m_preloadReport = FALSE;

	m_netMinPlayers = 1; // allowing sandbox mode

	m_defaultIP = 0;

	m_BuildSpeed = 0.0f;
	m_MinDistFromEdgeOfMapForBuild = 0.0f;
	m_SupplyBuildBorder = 0.0f;
	m_allowedHeightVariationForBuilding = 0.0f;
	m_MinLowEnergyProductionSpeed = 0.0f;
	m_MaxLowEnergyProductionSpeed = 0.0f;
	m_LowEnergyPenaltyModifier = 0.0f;
	m_MultipleFactory = 0.0f;
	m_RefundPercent = 0.0f;

	m_commandCenterHealRange = 0.0f;
	m_commandCenterHealAmount = 0.0f;
	m_maxTunnelCapacity = 0;
	m_maxLineBuildObjects = 0;

	m_standardMinefieldDensity = 0.01f;
	m_standardMinefieldDistance = 40.0f;

	m_objectPlacementOpacity = 0.45f;
	m_objectPlacementShadows = TRUE;

	m_groupSelectMinSelectSize = 5;
	m_groupSelectVolumeBase = 0.5f;
	m_groupSelectVolumeIncrement = 0.02f;
	m_maxUnitSelectSounds = 8;

	m_selectionFlashSaturationFactor = 0.5f; /// how colorful should the selection flash be? 0-4
	m_selectionFlashHouseColor = FALSE;  /// skip the house color and just use white.

	m_cameraAudibleRadius = 500.0;
	m_groupMoveClickToGatherFactor = 1.0f;

	m_shakeSubtleIntensity = 0.5f;
	m_shakeNormalIntensity = 1.0f;
	m_shakeStrongIntensity = 2.5f;
	m_shakeSevereIntensity = 5.0f;
	m_shakeCineExtremeIntensity = 8.0f;
	m_shakeCineInsaneIntensity = 12.0f;
	m_maxShakeIntensity = 10.0f;
	m_maxShakeRange = 150.f;

	m_sellPercentage = 1.0f;
	m_baseRegenHealthPercentPerSecond = 0.0f;
	m_baseRegenDelay = 0;

#ifdef ALLOW_SURRENDER
	m_prisonBountyMultiplier = 1.0f;
	m_prisonBountyTextColor = GameMakeColor( 255, 255, 255, 255 );
#endif

	m_hotKeyTextColor = GameMakeColor(255,255,0,255);

  // THis is put ON ice until later
  //  m_cheaterHasBeenSpiedIfMyLowestBitIsTrue = GameMakeColor(255,128,0,0);// orange, to the hacker's eye

	m_shroudColor.setFromInt( 0x00FFFFFF ) ;
	m_clearAlpha = 255;
	m_fogAlpha = 127;
	m_shroudAlpha = 0;

	m_powerBarBase = 7;
	m_powerBarIntervals = 3;
	m_powerBarYellowRange = 5;
	m_displayGamma = 1.0f;	//ramp that does nothing

	m_standardPublicBones.clear();

	m_antiAliasLevel = WW3D::MultiSampleModeEnum::MULTISAMPLE_MODE_NONE;
	m_textureFilteringMode = TextureFilterClass::TextureFilterMode::TEXTURE_FILTER_BILINEAR;
	m_textureAnisotropyLevel = TextureFilterClass::AnisotropicFilterMode::TEXTURE_FILTER_ANISOTROPIC_2X;

//	m_languageFilterPref = false;
	m_languageFilterPref = true;
	m_firewallBehavior = FirewallHelperClass::FIREWALL_TYPE_UNKNOWN;
	m_firewallPortOverride = 0;
	m_firewallPortAllocationDelta = 0;
	m_loadScreenDemo = FALSE;
	m_disableRender = false;

	m_saveCameraInReplay = FALSE;
	m_useCameraInReplay = FALSE;
	m_enablePlayerObserver = FALSE;

	m_networkLatencyFontSize = 8;
	m_renderFpsFontSize = 8;
	m_systemTimeFontSize = 8;
	m_gameTimeFontSize = 8;
	m_playerInfoListFontSize = 8;

	m_showMoneyPerMinute = FALSE;
	m_allowMoneyPerMinuteForPlayer = FALSE;

	m_gameWindowTransitionSpeedMultiplier = 1.0f;

	m_debugShowGraphicalFramerate = FALSE;

	// GeneralsX @feature dogefly233/felipebraz 13/11/2026 Force loose files on disk to override mod archives (#397)
	m_looseFilesWin = FALSE;

	// By default, show all asserts.

	m_unlookPersistDuration		= 30;

	//-----------------------------------------------------------------------------------------------

	// network timing values.  Having these default to 0 would be bad. - BGC
	m_networkFPSHistoryLength = 30;
	m_networkLatencyHistoryLength = 200;
	m_networkRunAheadMetricsTime = 500;
	m_networkCushionHistoryLength = 10;
	m_networkRunAheadSlack = 10;
	m_networkKeepAliveDelay = 20;
	m_networkDisconnectTime = 5000;
	m_networkPlayerTimeoutTime = 60000;
	m_networkDisconnectScreenNotifyTime = 15000;

	m_isBreakableMovie = FALSE;
	m_breakTheMovie = FALSE;
	m_looseFilesWin = FALSE;

	setTimeOfDay( m_timeOfDay );

	m_buildMapCache = FALSE;
	m_initialFile.clear();
	m_pendingFile.clear();

	m_simulateReplays.clear();
	m_simulateReplayJobs = SIMULATE_REPLAYS_SEQUENTIAL;

	for (i = LEVEL_FIRST; i <= LEVEL_LAST; ++i)
		m_healthBonus[i] = 1.0f;

	for (i = 0; i < PLAYERTYPE_COUNT; ++i)
	{
		for (j = 0; j < DIFFICULTY_COUNT; ++j)
		{
			m_soloPlayerHealthBonusForDifficulty[i][j] = 1.0f;
		}
	}

	m_defaultStructureRubbleHeight = 1.0f;
	m_weaponBonusSet = newInstance(WeaponBonusSet);

	m_shellMapName.set("Maps\\ShellMap1\\ShellMap1.map");
	m_shellMapOn =TRUE;
	m_playIntro = TRUE;
	m_playSizzle = TRUE;
	m_loadScreenRender = FALSE;

	m_keyboardDefaultScrollFactor = m_keyboardScrollFactor = 0.5f;
	// GeneralsX @bugfix Gabriel Petry 30/09/2026 Preserve retail mouse-wheel zoom speed by default.
	m_mouseScrollFactor = 1.0f;
	m_drawScrollAnchor = FALSE;
	m_moveScrollAnchor = FALSE;
	m_scrollAmountCutoff = 10.0f;
	m_cameraAdjustSpeed = 0.1f;
	m_enforceMaxCameraHeight = TRUE;

	m_animateWindows = TRUE;

	m_iniCRC = 0;
	m_exeCRC = 0;

	m_movementPenaltyDamageState = BODY_REALLYDAMAGED;

	m_shouldUpdateTGAToDDS = FALSE;

	// Default DoubleClickTime to System double click time.
	m_doubleClickTimeMS = GetDoubleClickTime(); // Note: This is actual MS, not frames.

#ifdef DUMP_PERF_STATS
	m_dumpPerformanceStatistics = FALSE;
  m_dumpStatsAtInterval = FALSE;
  m_statsInterval = 30;
#endif

	m_forceBenchmark = FALSE;	///<forces running of CPU detection benchmark, even on known cpu's.

	m_keyboardCameraRotateSpeed = 0.1f;

	// GeneralsX @feature Bender 01/04/2026 Cross-platform user data directory (Registry-based)
	// Adopts upstream refactoring with extended cross-platform support
	// Set user data directory based on registry settings instead of INI parameters.
	// This allows us to localize the leaf name.
	m_userDataDir = BuildUserDataPathFromRegistry();
	CreateDirectory(m_userDataDir.str(), nullptr);

	//-allAdvice feature
	//m_allAdvice = FALSE;

	m_useAlternateMouse = FALSE;
	// GeneralsX @bugfix Meeseeks 18/06/2026 Default to TRUE to allow right-click drag scrolling under alternate controls by default
	m_useRightMouseScrollWithAlternateMouse = TRUE;
	m_clientRetaliationModeEnabled = TRUE; //On by default.
	m_doubleClickAttackMove = FALSE;

}


//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
GlobalData::~GlobalData()
{
	DEBUG_ASSERTCRASH( TheWritableGlobalData->m_next == nullptr, ("~GlobalData: theOriginal is not original") );

	deleteInstance(m_weaponBonusSet);

	if( m_theOriginal == this )	{
		m_theOriginal = nullptr;
		TheWritableGlobalData = nullptr;
	}

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool GlobalData::setTimeOfDay( TimeOfDay tod )
{
	if( tod >= TIME_OF_DAY_COUNT || tod < TIME_OF_DAY_FIRST )
	{
		return FALSE;
	}

	m_timeOfDay = tod;
	for (Int i=0; i<MAX_GLOBAL_LIGHTS; i++)
	{	m_terrainAmbient[i] = m_terrainLighting[ tod ][i].ambient;
		m_terrainDiffuse[i] = m_terrainLighting[ tod ][i].diffuse;
		m_terrainLightPos[i] = m_terrainLighting[ tod ][i].lightPos;
	}

	return TRUE;

}

//-------------------------------------------------------------------------------------------------
/** Create a new global data instance to override the existing data set.  The
	* initial values of the newly created instance will be a copy of the current
	* data (or the most recently created override) */
//-------------------------------------------------------------------------------------------------
GlobalData *GlobalData::newOverride()
{
	// TheSuperHackers @info This copy is not implemented in VS6 builds
	GlobalData *overrideData = NEW GlobalData;

	// copy the data from the latest override (TheWritableGlobalData) to the newly created instance
	DEBUG_ASSERTCRASH( TheWritableGlobalData, ("GlobalData::newOverride() - no existing data") );
	*overrideData = *TheWritableGlobalData;

	//
	// link the override to the previously created one, the link order is important here
	// for the reset function, if you change the way things are linked
	// for overrides make sure you update the reset function
	//
	overrideData->m_next = TheWritableGlobalData;

	// set this new instance as the 'most current override' where we will access all data from
	TheWritableGlobalData = overrideData;

	return overrideData;

}

//-------------------------------------------------------------------------------------------------
void GlobalData::init()
{
	m_exeCRC = generateExeCRC();
}

//-------------------------------------------------------------------------------------------------
/** Reset, remove any override data instances and return to just the initial one
	*/
//-------------------------------------------------------------------------------------------------
void GlobalData::reset()
{
	DEBUG_ASSERTCRASH(this == TheWritableGlobalData, ("calling reset on wrong GlobalData"));

	//
	// delete	any data instances that were loaded as an override and set the original
	// global data instance as the singleton TheWritableGlobalData once again
	//
	while (TheWritableGlobalData != GlobalData::m_theOriginal)
	{

		// get next instance
		GlobalData* next = m_next;

		// delete the head of the global data list (the latest override)
		delete TheWritableGlobalData;

		// set next as top
		TheWritableGlobalData = next;

	}

	//
	// we now have the one single global data in TheWritableGlobalData singleton, lets sanity check
	// some of all that
	//
	DEBUG_ASSERTCRASH( TheWritableGlobalData->m_next == nullptr, ("ResetGlobalData: theOriginal is not original") );
	DEBUG_ASSERTCRASH( TheWritableGlobalData == GlobalData::m_theOriginal, ("ResetGlobalData: oops") );

}

//-------------------------------------------------------------------------------------------------
/** Parse GameData entry */
//-------------------------------------------------------------------------------------------------
void GlobalData::parseGameDataDefinition( INI* ini )
{
	if( TheWritableGlobalData && ini->getLoadType() != INI_LOAD_MULTIFILE)
	{

		//
		// if the type of loading we're doing creates override data, we need to
		// be loading into a new override item
		//
		if( ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES )
			TheWritableGlobalData->newOverride();

	}
	else if (!TheWritableGlobalData)
	{

		// we don't have any global data instance at all yet, create one
		TheWritableGlobalData = NEW GlobalData;

	}
	// If we're multifile, then continue loading stuff into the Global Data as normal.

	// parse the ini weapon definition
	ini->initFromINI( TheWritableGlobalData, s_GlobalDataFieldParseTable );


	// override INI values with user preferences
	OptionPreferences optionPref;
	TheWritableGlobalData->m_useAlternateMouse = optionPref.getAlternateMouseModeEnabled();
	TheWritableGlobalData->m_useRightMouseScrollWithAlternateMouse = optionPref.getRightMouseScrollWithAlternateMouseEnabled();
	TheWritableGlobalData->m_clientRetaliationModeEnabled = optionPref.getRetaliationModeEnabled();
	TheWritableGlobalData->m_doubleClickAttackMove = optionPref.getDoubleClickAttackMoveEnabled();
	TheWritableGlobalData->m_jpegQuality = optionPref.getJpegQuality();
	TheWritableGlobalData->m_keyboardScrollFactor = optionPref.getScrollFactor();
	TheWritableGlobalData->m_drawScrollAnchor = optionPref.getDrawScrollAnchor();
	TheWritableGlobalData->m_moveScrollAnchor = optionPref.getMoveScrollAnchor();
	TheWritableGlobalData->m_defaultIP = optionPref.getLANIPAddress();
	TheWritableGlobalData->m_firewallBehavior = optionPref.getFirewallBehavior();
	TheWritableGlobalData->m_firewallPortAllocationDelta = optionPref.getFirewallPortAllocationDelta();
	TheWritableGlobalData->m_firewallPortOverride = optionPref.getFirewallPortOverride();

	TheWritableGlobalData->m_saveCameraInReplay = optionPref.saveCameraInReplays();
	TheWritableGlobalData->m_useCameraInReplay = optionPref.useCameraInReplays();
	TheWritableGlobalData->m_enablePlayerObserver = optionPref.getPlayerObserverEnabled();

	TheWritableGlobalData->m_networkLatencyFontSize = optionPref.getNetworkLatencyFontSize();
	TheWritableGlobalData->m_renderFpsFontSize = optionPref.getRenderFpsFontSize();
	TheWritableGlobalData->m_systemTimeFontSize = optionPref.getSystemTimeFontSize();
	TheWritableGlobalData->m_gameTimeFontSize = optionPref.getGameTimeFontSize();
	TheWritableGlobalData->m_playerInfoListFontSize = optionPref.getPlayerInfoListFontSize();
	TheWritableGlobalData->m_showMoneyPerMinute = optionPref.getShowMoneyPerMinute();
	TheWritableGlobalData->m_gameWindowTransitionSpeedMultiplier = optionPref.getGameWindowTransitionSpeedMultiplier();
	// GeneralsX @feature fbraz3 14/10/2026 Camera height and terrain draw distance overridden from Options.ini
	// GeneralsX @bugfix fbraz3 15/10/2026 Validate finite Real before assigning to GlobalData
	if (optionPref.find("MaxCameraHeight") != optionPref.end())
	{
		Real maxCam = optionPref.getMaxCameraHeight();
		if (!isFinite(maxCam) || maxCam < 100.0f)
			maxCam = 100.0f;
		else if (maxCam > 1000.0f)
			maxCam = 1000.0f;
		TheWritableGlobalData->m_maxCameraHeight = maxCam;
	}

	if (optionPref.find("TerrainDrawDistanceScale") != optionPref.end())
	{
		Real drawDist = optionPref.getTerrainDrawDistanceScale();
		if (!isFinite(drawDist) || drawDist < 1.0f)
			drawDist = 1.0f;
		else if (drawDist > 2.0f)
			drawDist = 2.0f;
		TheWritableGlobalData->m_terrainDrawDistanceScale = drawDist;
	}
	// GeneralsX @feature felipebraz 17/09/2026 Skirmish simulation tick rate configuration (#281)
	TheWritableGlobalData->m_skirmishTickRate = optionPref.getSkirmishTickRate();

	// GeneralsX @feature fbraz3 08/10/2026 Enhanced graphics & lighting options overridden from Options.ini
	TheWritableGlobalData->m_advancedShadows = optionPref.getAdvancedShadows(TheWritableGlobalData->m_advancedShadows);
	TheWritableGlobalData->m_advancedLighting = optionPref.getAdvancedLighting(TheWritableGlobalData->m_advancedLighting);
	TheWritableGlobalData->m_useShadowVolumesForSkins = optionPref.getUseShadowVolumesForSkins(TheWritableGlobalData->m_useShadowVolumesForSkins);
	TheWritableGlobalData->m_extendedShadowFrustumCulling = optionPref.getExtendedShadowFrustumCulling(TheWritableGlobalData->m_extendedShadowFrustumCulling);
	TheWritableGlobalData->m_naturalSunElevation = optionPref.getNaturalSunElevation(TheWritableGlobalData->m_naturalSunElevation);
	TheWritableGlobalData->m_shadowsForProjectiles = optionPref.getShadowsForProjectiles(TheWritableGlobalData->m_shadowsForProjectiles);
	TheWritableGlobalData->m_shadowsForProps = optionPref.getShadowsForProps(TheWritableGlobalData->m_shadowsForProps);
	TheWritableGlobalData->m_bridgeShadows = optionPref.getBridgeShadows(TheWritableGlobalData->m_bridgeShadows);
	TheWritableGlobalData->m_waterReflections = optionPref.getWaterReflections(TheWritableGlobalData->m_waterReflections);
	TheWritableGlobalData->m_explosionDynamicLights = optionPref.getExplosionDynamicLights(TheWritableGlobalData->m_explosionDynamicLights);
	TheWritableGlobalData->m_vehicleHeadlights = optionPref.getVehicleHeadlights(TheWritableGlobalData->m_vehicleHeadlights);
	TheWritableGlobalData->m_enablePostProcessing = optionPref.getEnablePostProcessing(TheWritableGlobalData->m_enablePostProcessing);
	TheWritableGlobalData->m_postProcessBloom = optionPref.getPostProcessBloom(TheWritableGlobalData->m_postProcessBloom);
	TheWritableGlobalData->m_postProcessHDR = optionPref.getPostProcessHDR(TheWritableGlobalData->m_postProcessHDR);
	TheWritableGlobalData->m_postProcessFXAA = optionPref.getPostProcessFXAA(TheWritableGlobalData->m_postProcessFXAA);
	TheWritableGlobalData->m_airUnitGroundIndicator = optionPref.getAirUnitGroundIndicator(TheWritableGlobalData->m_airUnitGroundIndicator);

	TheWritableGlobalData->m_antiAliasLevel = optionPref.getAntiAliasing();
	TheWritableGlobalData->m_textureFilteringMode = optionPref.getTextureFilterMode();
	TheWritableGlobalData->m_textureAnisotropyLevel = optionPref.getTextureAnisotropyLevel();

	Int val=optionPref.getGammaValue();
	//generate a value between 0.6 and 2.0.
	if (val < 50)
	{
		//darker gamma
		if (val <= 0)
			TheWritableGlobalData->m_displayGamma = 0.6f;
		else
			TheWritableGlobalData->m_displayGamma=1.0f-(0.4f) * (Real)(50-val)/50.0f;
	}
	else
	if (val > 50)
		TheWritableGlobalData->m_displayGamma=1.0f+(1.0f) * (Real)(val-50)/50.0f;

	Int xres,yres;
	optionPref.getResolution(&xres, &yres);

	TheWritableGlobalData->m_xResolution = xres;
	TheWritableGlobalData->m_yResolution = yres;
}

void GlobalData::parseCustomDefinition()
{
	if (addon::HasFullviewportDat())
	{
		// TheSuperHackers @tweak xezon 03/08/2025 Force full viewport for 'Control Bar Pro' Addons like GenTool did it.
		m_viewportHeightScale = 1.0f;
	}
}

UnsignedInt GlobalData::generateExeCRC()
{
	DEBUG_ASSERTCRASH(TheFileSystem != nullptr, ("TheFileSystem is null"));

	// lets CRC the executable!  Whee!
	const Int blockSize = 65536;
	CRC exeCRC;
	File *fp;
	// TheSuperHackers @tweak SkyAero/xezon 27/05/2025
	// Simulate the EXE's CRC value to force Network and Replay compatibility with another build.
#if (defined(_MSC_VER) && _MSC_VER < 1300) && RETAIL_COMPATIBLE_CRC

#define GENERALSMD_104_CD_EXE_CRC    0x3b6fb2cfu
#define GENERALSMD_104_STEAM_EXE_CRC 0xf6a4221bu
#define GENERALSMD_104_EAAPP_EXE_CRC 0xc4181eb9u

	exeCRC.set(GENERALSMD_104_CD_EXE_CRC);
	DEBUG_LOG(("Fake EXE CRC is 0x%8.8X", exeCRC.get()));

#elif defined(__linux__)
	// GeneralsX @bugfix BenderAI 18/02/2026
	// On Linux, reading the entire 180MB+ binary for CRC is prohibitively slow
	// and unnecessary. Instead, use version-based CRC which is fast and sufficient.
	// Only compute CRC from version number below.

#else
	{
		Char buffer[ _MAX_PATH ];
// TheSuperHackers @build fighter19 11/02/2026 Cross-platform executable path retrieval (Phase 5: macOS)
#ifdef _WIN32
		GetModuleFileName( nullptr, buffer, sizeof( buffer ) );
#elif defined(__APPLE__)
		// macOS: Use _NSGetExecutablePath from mach-o/dyld.h
		uint32_t len = sizeof(buffer);
		int ret = _NSGetExecutablePath(buffer, &len);
		if (ret != 0) {
			buffer[0] = '\0';
		}
#else
		// Linux: Read /proc/self/exe symlink
		ssize_t len = readlink("/proc/self/exe", buffer, sizeof(buffer) - 1);
		if (len != -1) {
			buffer[len] = '\0';
		} else {
			buffer[0] = '\0';
		}
#endif
		fp = TheFileSystem->openFile(buffer, File::READ | File::BINARY);
		if (fp != nullptr) {
			unsigned char crcBlock[blockSize];
			Int amtRead = 0;
			Int readCount = 0;
			while ( (amtRead=fp->read(crcBlock, blockSize)) > 0 )
			{
				readCount++;
				exeCRC.computeCRC(crcBlock, amtRead);
				if (readCount > 1000) {
					break;
				}
			}
			DEBUG_LOG(("EXE CRC is 0x%8.8X", exeCRC.get()));
			fp->close();
			fp = nullptr;
		}
		else {
			DEBUG_CRASH(("Executable file has failed to open"));
		}
	}
#endif

	UnsignedInt version = 0;
	if (TheVersion)
	{
		version = TheVersion->getVersionNumber();
		exeCRC.computeCRC( &version, sizeof(UnsignedInt) );
	}
	// GeneralsX @bugfix Copilot 22/03/2026 Load shared script CRC inputs relative to the configured asset root.
	// Add in MP scripts to the EXE CRC, since the game will go out of sync if they change
	fp = TheFileSystem->openFile("Data\\Scripts\\SkirmishScripts.scb", File::READ | File::BINARY);
	if (fp != nullptr) {
		unsigned char crcBlock[blockSize];
		Int amtRead = 0;
		while ( (amtRead=fp->read(crcBlock, blockSize)) > 0 )
		{
			exeCRC.computeCRC(crcBlock, amtRead);
		}
		fp->close();
		fp = nullptr;
	}
	fp = TheFileSystem->openFile("Data\\Scripts\\MultiplayerScripts.scb", File::READ | File::BINARY);
	if (fp != nullptr) {
		unsigned char crcBlock[blockSize];
		Int amtRead = 0;
		while ( (amtRead=fp->read(crcBlock, blockSize)) > 0 )
		{
			exeCRC.computeCRC(crcBlock, amtRead);
		}
		fp->close();
		fp = nullptr;
	}

	DEBUG_LOG(("EXE+Version(%d.%d)+SCB CRC is 0x%8.8X", version >> 16, version & 0xffff, exeCRC.get()));
	return exeCRC.get();
}

AsciiString GlobalData::BuildUserDataPathFromRegistry()
{
	AsciiString userDataDir;

#ifdef _WIN32
	// GeneralsX @refactor Bender 01/04/2026 Windows-specific path handling (Registry-based)
	// Integrates upstream bug-fix for OneDrive/Group Policy folder redirection
#if defined(_MSC_VER) && (_MSC_VER < 1300)
	// VC6 lacks FOLDERID_Documents and KF_FLAG_DEFAULT
	const GUID FOLDERID_Documents = { 0xFDD39AD0, 0x238F, 0x46AF, 0xAD, 0xB4, 0x6C, 0x85, 0x48, 0x03, 0x69, 0xC7 };
	const DWORD KF_FLAG_DEFAULT = 0;
#endif

	typedef HRESULT(WINAPI* PFN_SHGetKnownFolderPath)(const GUID& rfid, DWORD dwFlags, HANDLE hToken, PWSTR* ppszPath);

	AsciiString myDocumentsDirectory;
	HMODULE shell32module = GetModuleHandleA("shell32.dll");
	PFN_SHGetKnownFolderPath pSHGetKnownFolderPath = nullptr;

	// TheSuperHackers @bugfix Mauller 20/03/2026 Fix the handling of folder redirection
	// OneDrive and Group Policy folder redirection is better supported by SHGetKnownFolderPath()
	// SHGetKnownFolderPath() is only supported in windows Vista onwards so we check for it being available
	if (shell32module) {
		pSHGetKnownFolderPath = (PFN_SHGetKnownFolderPath)GetProcAddress(shell32module, "SHGetKnownFolderPath");
	}

	if (pSHGetKnownFolderPath) {
		PWSTR pszPath = nullptr;
		HRESULT hr = pSHGetKnownFolderPath(FOLDERID_Documents, KF_FLAG_DEFAULT, nullptr, &pszPath);

		if (SUCCEEDED(hr) && pszPath) {
			myDocumentsDirectory.translate(pszPath);
			CoTaskMemFree(pszPath);
		}
	}
	else {
		char temp[_MAX_PATH + 1];
		if (SHGetSpecialFolderPath(nullptr, temp, CSIDL_PERSONAL, true)) {
			myDocumentsDirectory = temp;
		}
	}

	if (!myDocumentsDirectory.isEmpty()) {
		// Now build the full path string
		if (!myDocumentsDirectory.endsWith("\\"))
			myDocumentsDirectory.concat('\\');

		AsciiString leafName;
		if (!GetStringFromRegistry("", "UserDataLeafName", leafName))
		{
			// Use something, anything
			// [MH] had to remove this, otherwise mapcache build step won't run... DEBUG_CRASH( ( "Could not find registry key UserDataLeafName; defaulting to \"Command and Conquer Generals Zero Hour Data\" " ) );
			leafName = "Command and Conquer Generals Zero Hour Data";
		}

		myDocumentsDirectory.concat(leafName);
		if (!myDocumentsDirectory.endsWith("\\"))
			myDocumentsDirectory.concat('\\');
	}

	userDataDir = myDocumentsDirectory;

#elif defined(__APPLE__)
	// GeneralsX @feature Bender 01/04/2026 macOS user data directory
	// Uses ~/Library/Application Support as standard macOS location for Zero Hour
	{
		const char* home = getenv("HOME");
		if (home) {
			std::filesystem::path path = std::filesystem::path(home) / "Library" / "Application Support" / "GeneralsX" / "GeneralsZH";
			std::filesystem::create_directories(path);
			userDataDir = path.string().c_str();
			if (!userDataDir.endsWith("/"))
				userDataDir.concat('/');
		} else {
			userDataDir = "./";
		}
	}

#else
	// GeneralsX @feature Bender 01/04/2026 Linux user data directory
	// Uses XDG Base Directory specification for Zero Hour
	{
		std::filesystem::path path;
		const char* xdgDataHome = getenv("XDG_DATA_HOME");
		const char* home = getenv("HOME");
		
		if (xdgDataHome) {
			path = std::filesystem::path(xdgDataHome);
		} else if (home) {
			path = std::filesystem::path(home) / ".local" / "share";
		} else {
			path = "./";
		}

		path = path / "GeneralsX" / "GeneralsZH";
		std::filesystem::create_directories(path);
		userDataDir = path.string().c_str();
		if (!userDataDir.endsWith("/"))
			userDataDir.concat('/');
	}
#endif

	return userDataDir;
}
```
