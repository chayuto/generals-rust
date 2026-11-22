# Translation Packet `PKT-399`: `Weapon`

## 0. Lean Port & Oracle Contract (`env:ephemeral-web`, Zero Pre-Created Skeletons)
- **Zero-Skeleton Handoff:** No Rust files exist under `crates/zh_logic/src/object/` for `Weapon` or `WeaponSet` yet. Create and export:
  1. `crates/zh_logic/src/object/weapon.rs` (`PKT-399`: `NO_MAX_SHOTS_LIMIT = 0x7fff_ffff`, `WeaponReloadType`, `WeaponPrefireType`, `WeaponAntiMaskType`, `WeaponAffectsMaskType`, `WeaponCollideMaskType`, `WeaponBonusConditionType`, `WeaponBonus`, `WeaponBonusSet`, `HistoricWeaponDamageInfo`, `WeaponTemplate`, `Weapon`, `WeaponDelayedDamageInfo`, `WeaponStore`)
  2. `crates/zh_logic/src/object/weapon_set.rs` (`PKT-400`: `WEAPONSLOT_COUNT = 3`, `WeaponSlotType` (`Primary = 0`, `Secondary = 1`, `Tertiary = 2`), `WeaponSetConditionType`, `WeaponChoiceCriteria`, `WeaponLockType`, `CanAttackResult`, `WeaponTemplateSet`, `WeaponSet`, `get_model_condition_for_weapon_slot`, `WEAPON_SET_TYPE_TO_MODEL_CONDITION_TYPE_MAP`)
  3. Wire `pub mod weapon;` and `pub mod weapon_set;` in `crates/zh_logic/src/object/mod.rs`.
  4. Replace the `#[ignore]` body in `crates/zh_logic/tests/golden_pkt399_weapon.rs` with the full 1,000-seed differential loop against `docs/golden_vectors/PKT-399.json`, remove `#[ignore]`, and run `python3 scripts/port_map.py --write` to update `docs/PORT_MAP.md`.
- **Reuse Existing Crate Types (Do Not Duplicate):**
  - `zh_logic::rules::combat_types::{DamageType, DamageTypeFlags, DeathType, WeaponBonusConditionFlags, WeaponSetFlags, WeaponSetType, WeaponStatus, DAMAGE_TYPE_NAMES, DEATH_TYPE_NAMES, WEAPON_SET_NAMES}`
  - `zh_logic::rules::kind_of::{KindOfMaskType, KINDOF_NAMES}`
  - `zh_logic::rules::model_state::{ModelConditionFlagType, ModelConditionFlags, MODEL_CONDITION_NAMES}`
  - `zh_logic::rules::object_status_types::{ObjectStatusMaskType, ObjectStatusType, OBJECT_STATUS_NAMES}`
  - `zh_common::ini` scanners, `zh_common::random_value::PrngState`, `zh_common::math::{Coord2D, Coord3D, real_to_int_floor, real_to_int_ceil}`, `zh_common::gamemath::gm_ceilf`, `zh_common::xfer::{Snapshot, Xfer, XferCrc, XferLoad, XferSave}`
- **Exact C++ Numerical & Serialization Invariants (`PKT-399.json`):**
  1. **`RATIONALIZE_ATTACK_RANGE` is defined (`Weapon.cpp:80`):** `PATHFIND_CELL_SIZE_F = 10.0_f32`, `UNDERSIZE = 10.0_f32 * 0.25_f32 = 2.5_f32`.
     - `WeaponTemplate::get_attack_range(bonus)`: `let mut r = self.attack_range * bonus.get_field(WeaponBonusField::Range) - 2.5_f32; if r < 0.0 { r = 0.0; } r`
     - `WeaponTemplate::get_minimum_attack_range()`: `let mut r = self.minimum_attack_range - 2.5_f32; if r < 0.0 { r = 0.0; } r`
     - `WeaponTemplate::is_contact_weapon()`: `(self.attack_range - 2.5_f32) < 10.0_f32`
  2. **Shot delay, reload, and pre-attack math (`Weapon.cpp:371-404, 502-531`):**
     - `parse_shot_delay`: parses either `<ms>` or `Min:<ms> Max:<ms>`, then converts each to frames via `gm_ceilf((ms as f32) * (30.0_f32 / 1000.0_f32)) as i32` (`WWMath::Ceilf(ConvertDurationFromMsecsToFrames((Real)ms))`). Note `ClipReloadTime`, `PreAttackDelay`, `ContinuousFireCoast`, `AutoReloadWhenIdle`, `SuspendFXDelay`, `FireSoundLoopTime`, and `HistoricBonusTime` use `INI::parseDurationUnsignedInt` (`gm_ceilf(ms * (30.0 / 1000.0)) as u32`).
     - `get_delay_between_shots(bonus, rng)`: if `min_delay == max_delay`, `delay = min_delay` (does **not** draw RNG!); else `delay = rng.get_game_logic_random_value(min_delay, max_delay)`. Returns `real_to_int_floor((delay as f32) / bonus.get_field(RateOfFire))`.
     - `get_clip_reload_time(bonus)`: `real_to_int_floor((self.clip_reload_time as f32) / bonus.get_field(RateOfFire))`.
     - `get_pre_attack_delay(bonus)`: `((self.pre_attack_delay as f32) * bonus.get_field(PreAttack)) as i32` (C++ `Int * Real` truncates toward zero!).
  3. **`WeaponBonus::append_bonuses` (`Weapon.cpp:3581-3587`):** `for f in 0..5 { bonus.field[f] += self.field[f] - 1.0_f32; }`. `WeaponBonusSet::append_bonuses(flags, bonus)` iterates `i in 0..27` and appends `self.bonus[i]` when `(flags & (1 << i)) != 0`.
  4. **`Weapon::new` & `set_clip_percent_full` (`Weapon.cpp:1840-1859, 1961-1977`):**
     - Constructor: `status = OutOfAmmo`, `ammo_in_clip = 0`, `pitch_limited = (tmpl.min_target_pitch > -PI || tmpl.max_target_pitch < PI)`, `max_shot_count = NO_MAX_SHOTS_LIMIT`, `cur_barrel = 0`, `num_shots_for_cur_barrel = tmpl.shots_per_barrel`, `suspend_fx_frame = current_frame.wrapping_add(tmpl.suspend_fx_delay)`.
     - `set_clip_percent_full(percent, allow_reduction, current_frame)`: if `clip_size == 0`, returns immediately; `ammo = real_to_int_floor((clip_size as f32) * percent)`; Note C++'s signed/unsigned comparison `(ammo as u32) > self.ammo_in_clip || (allow_reduction && (ammo as u32) < self.ammo_in_clip)`. When taken: `self.ammo_in_clip = ammo as u32`, `self.status = if self.ammo_in_clip != 0 { OutOfAmmo } else { ReadyToFire }`, `when_last_reload_started = current_frame`, `when_we_can_fire_again = current_frame`, and `rebuild_scatter_targets()` (`0..tmpl.scatter_targets.len() as i32`).
  5. **`Weapon::get_status`, `get_percent_ready_to_fire`, `get_remaining_ammo` (`Weapon.cpp:2407-2449, 2854-2870`):**
     - `get_status(now)` (**mutates** `m_status` when `now >= when_we_can_fire_again` and `now >= when_pre_attack_finished`!): if `now < when_pre_attack_finished`, returns `PreAttack` without mutating `m_status`; else if `now >= when_we_can_fire_again`, sets `m_status = if ammo_in_clip > 0 { ReadyToFire } else { OutOfAmmo }` and returns `m_status`; else returns `m_status`.
     - `get_remaining_ammo(now)`: `if self.get_status(now) == ReloadingClip { 0 } else { self.ammo_in_clip }`.
  6. **`Weapon::crc` vs `Weapon::xfer` (`Weapon.cpp:3255-3560`):**
     - `Weapon::crc` does **not** write a version byte, `m_status`, or `m_suspendFXFrame`: writes `tmpl_name` (`AsciiString`), `wslot` (`i32`), `ammo_in_clip` (`u32`), `when_we_can_fire_again` (`u32`), `when_pre_attack_finished` (`u32`), `when_last_reload_started` (`u32`), `last_fire_frame` (`u32`), `projectile_stream_id` (`ObjectId`), `laser_id_unused = 0` (`ObjectId`), `max_shot_count` (`i32`), `cur_barrel` (`i32`), `num_shots_for_cur_barrel` (`i32`), `scatter_count` (`u16`) + each `i32`, `pitch_limited` (`Bool`), `leech_weapon_range_active` (`Bool`).
     - `Weapon::xfer` writes `version = 3` (`u8`), `tmpl_name` (`AsciiString`), `wslot` (`i32`), `status` (`i32`), `ammo_in_clip` (`u32`), `when_we_can_fire_again` (`u32`), `when_pre_attack_finished` (`u32`), `when_last_reload_started` (`u32`), `last_fire_frame` (`u32`), `suspend_fx_frame` (`u32`), `projectile_stream_id` (`ObjectId`), `laser_id_unused = 0` (`ObjectId`), `max_shot_count` (`i32`), `cur_barrel` (`i32`), `num_shots_for_cur_barrel` (`i32`), `scatter_count` (`u16`) + each `i32`, `pitch_limited` (`Bool`), `leech_weapon_range_active` (`Bool`).
  7. **`WeaponSet::xfer` (`WeaponSet.cpp:214-309`):**
     - Default build has `RETAIL_COMPATIBLE_XFER_SAVE = 1`, so `currentVersion = 1`!
     - Writes `version = 1` (`u8`), `thing_template_name` (`"DummyThing"` in oracle when template set is non-null, else `""`), `ws_flags.xfer(xfer)` (`BitFlags<17>` token-list xfer using `WEAPON_SET_NAMES`), then for each slot `0..3`: `has_weapon` (`Bool`) and if true `xfer.xfer_snapshot(weapon)` (which writes a 4-byte `begin_block`/`end_block` size around `Weapon::xfer`!), then `cur_weapon` (`i32`), `cur_weapon_locked_status` (`i32`), `filled_weapon_slot_mask` (`u32`), `total_anti_mask` (`i32`), then **`xfer_bool(if version >= 3 { &mut has_pitch_limit } else { &mut has_damage_weapon })`** followed by **`xfer_bool(&mut has_damage_weapon)`** (note: in version 1, `has_damage_weapon` is serialized twice!), then `total_damage_type_mask.xfer(xfer)` (`BitFlags<38>` token-list xfer using `DAMAGE_TYPE_NAMES`).
- **JSON Vector Contract (`docs/golden_vectors/PKT-399.json`):**
  - Inputs per seed: `ini` (`Weapon Wpn_<seed>` INI block), `ovr` (optional override block with `INI_LOAD_CREATE_OVERRIDES`: `PrimaryDamage = 275`, `AttackRange = 320`), `wts_ini` (`WeaponSet ... End` block parsed into `WeaponTemplateSet` with `thing_template_name = "DummyThing"`), `rand_seed` (for `get_delay_between_shots`), `bflags` (`WeaponBonusConditionFlags`), `frame0`, `f1`, `query_f`, `wslot`, `pcts` (`[pct1_bits, pct2_bits, allow_red]`), `wstate` (`[status_override, next_shot_f, pre_atk_f, last_fire_f, stream_id, max_shots, leech_act]`), `lock_op` (`[op, lock_slot, ws_cond]`).
  - Note on `WeaponSet` slot 0 initialization in oracle: slot 0 copies `wpn`'s state (`status`, `ammo`, `next_shot_f`, `pre_atk_f`, `f1`, `last_fire_f`, `suspend_fx_frame`, `stream_id`, `max_shots`, `cur_barrel`, `num_shots_for_cur_barrel`, `scatter_targets_unused`, `leech_act`) while slots 1..2 (when present in `wts`) are freshly allocated via `Weapon::new(slot_wt, slot)` at `query_f` (`suspend_fx_frame = query_f + slot_wt.suspend_fx_delay`).
  - Expected outputs per seed: `bf` (`[u32; 5]` bonus field bits), `t_stats` (`[atk_rng, unmod_rng, min_rng, prim_dmg, prim_rad, sec_dmg, sec_rad]` f32 bits), `t_ints` (`[shot_delay, clip_reload, pre_atk_delay, is_contact, is_dmg_wpn, is_pitch_lim]`), `rand_crc`, `w_q` (`[queried_status, pct_ready_bits, rem_ammo]` at `query_f`), `wxlen`, `wxb_crc`, `wxcrc`, `ws_q` (`[cur_slot, is_locked, filled_mask, total_anti, has_pitch, has_dmg, most_ready_pct, out_of_ammo, primary_cmd_mask, share_reload, share_lock]`), `mc_crc` (CRC over `XferSave` bytes of `get_model_condition_for_weapon_slot(wslot, ws_cond)`), `wsxlen`, `wsxcrc`, plus Seed-0 checkpoints (`s0_tmpl`, `s0_wpn` with `w_xhex`, `s0_wset` with `ws_xhex` and `mc_xhex`).

## 1. Compartmentalization & Routing Metadata
- **Packet ID:** `PKT-399`
- **Work Package:** `WP-5.5` (Phase 5)
- **Complexity & Assigned Tier:** Class `C3` -> `tier:2-engineer`
- **Environment Access:** `env:hybrid-ci`
- **FFI Seam Pattern:** Core Spine
- **Allowed Rust Output File:** `crates/zh_logic/src/object/weapon.rs` *(Plus removing `#[ignore]` in the corresponding `crates/zh_logic/tests/golden_*.rs` test file; do not modify other files)*
- **Mandatory V&V Gates:** `V1, V2-Golden, V2-FFI, V3, V4`
- **Local / Ephemeral-Web Verification Commands:**
  ```bash
  python3 scripts/lint_determinism.py && cargo test --workspace
  ```

## 2. AST/Source-Extracted Determinism Invariants
- **Direct Engine `#include` Dependencies (41):**
  - `Common/AudioEventRTS.h`
  - `Common/CRCDebug.h`
  - `Common/GameAudio.h`
  - `Common/GameCommon.h`
  - `Common/GameState.h`
  - `Common/INI.h`
  - `Common/PerfTimer.h`
  - `Common/Player.h`
  - `Common/ThingFactory.h`
  - `Common/ThingTemplate.h`
  - `Common/Xfer.h`
  - `Common/crc.h`
  - `GameClient/Drawable.h`
  - `GameClient/FXList.h`
  - `GameClient/InGameUI.h`
  - `GameClient/ParticleSys.h`
  - `GameLogic/AIPathfind.h`
  - `GameLogic/Damage.h`
  - `GameLogic/ExperienceTracker.h`
  - `GameLogic/GameLogic.h`
  - `GameLogic/Module/AIUpdate.h`
  - `GameLogic/Module/AssaultTransportAIUpdate.h`
  - `GameLogic/Module/AssistedTargetingUpdate.h`
  - `GameLogic/Module/BehaviorModule.h`
  - `GameLogic/Module/BodyModule.h`
  - `GameLogic/Module/ContainModule.h`
  - `GameLogic/Module/LaserUpdate.h`
  - `GameLogic/Module/PhysicsUpdate.h`
  - `GameLogic/Module/ProjectileStreamUpdate.h`
  - `GameLogic/Module/SpawnBehavior.h`
  - `GameLogic/Module/SpecialPowerCompletionDie.h`
  - `GameLogic/Module/UpdateModule.h`
  - `GameLogic/Object.h`
  - `GameLogic/ObjectCreationList.h`
  - `GameLogic/PartitionManager.h`
  - `GameLogic/TerrainLogic.h`
  - `GameLogic/Weapon.h`
  - `GameLogic/WeaponBonusConditionFlags.h`
  - `GameLogic/WeaponStatus.h`
  - `PreRTS.h`
  - `WWMath/matrix3d.h`
- **INI `FieldParse` Table Entries (76):**
  - `"PrimaryDamage" => INI::parseReal`
  - `"PrimaryDamageRadius" => INI::parseReal`
  - `"SecondaryDamage" => INI::parseReal`
  - `"SecondaryDamageRadius" => INI::parseReal`
  - `"ShockWaveAmount" => INI::parseReal`
  - `"ShockWaveRadius" => INI::parseReal`
  - `"ShockWaveTaperOff" => INI::parseReal`
  - `"AttackRange" => INI::parseReal`
  - `"MinimumAttackRange" => INI::parseReal`
  - `"RequestAssistRange" => INI::parseReal`
  - `"AcceptableAimDelta" => INI::parseAngleReal`
  - `"ScatterRadius" => INI::parseReal`
  - `"ScatterTargetScalar" => INI::parseReal`
  - `"ScatterRadiusVsInfantry" => INI::parseReal`
  - `"DamageType" => DamageTypeFlags::parseSingleBitFromINI`
  - `"DamageStatusType" => ObjectStatusMaskType::parseSingleBitFromINI`
  - `"DeathType" => INI::parseIndexList`
  - `"WeaponSpeed" => INI::parseVelocityReal`
  - `"MinWeaponSpeed" => INI::parseVelocityReal`
  - `"ScaleWeaponSpeed" => INI::parseBool`
  - `"WeaponRecoil" => INI::parseAngleReal`
  - `"MinTargetPitch" => INI::parseAngleReal`
  - `"MaxTargetPitch" => INI::parseAngleReal`
  - `"RadiusDamageAngle" => INI::parseAngleReal`
  - `"ProjectileObject" => INI::parseAsciiString`
  - `"FireSound" => INI::parseAudioEventRTS`
  - `"FireSoundLoopTime" => INI::parseDurationUnsignedInt`
  - `"FireFX" => parseAllVetLevelsFXList`
  - `"ProjectileDetonationFX" => parseAllVetLevelsFXList`
  - `"FireOCL" => parseAllVetLevelsAsciiString`
  - `"ProjectileDetonationOCL" => parseAllVetLevelsAsciiString`
  - `"ProjectileExhaust" => parseAllVetLevelsPSys`
  - `"VeterancyFireFX" => parsePerVetLevelFXList`
  - `"VeterancyProjectileDetonationFX" => parsePerVetLevelFXList`
  - `"VeterancyFireOCL" => parsePerVetLevelAsciiString`
  - `"VeterancyProjectileDetonationOCL" => parsePerVetLevelAsciiString`
  - `"VeterancyProjectileExhaust" => parsePerVetLevelPSys`
  - `"ClipSize" => INI::parseInt`
  - `"ContinuousFireOne" => INI::parseInt`
  - `"ContinuousFireTwo" => INI::parseInt`
  - `"ContinuousFireCoast" => INI::parseDurationUnsignedInt`
  - `"AutoReloadWhenIdle" => INI::parseDurationUnsignedInt`
  - `"ClipReloadTime" => INI::parseDurationUnsignedInt`
  - `"DelayBetweenShots" => WeaponTemplate::parseShotDelay`
  - `"ShotsPerBarrel" => INI::parseInt`
  - `"DamageDealtAtSelfPosition" => INI::parseBool`
  - `"RadiusDamageAffects" => INI::parseBitString32`
  - `"ProjectileCollidesWith" => INI::parseBitString32`
  - `"AntiAirborneVehicle" => INI::parseBitInInt32`
  - `"AntiGround" => INI::parseBitInInt32`
  - `"AntiProjectile" => INI::parseBitInInt32`
  - `"AntiSmallMissile" => INI::parseBitInInt32`
  - `"AntiMine" => INI::parseBitInInt32`
  - `"AntiParachute" => INI::parseBitInInt32`
  - `"AntiAirborneInfantry" => INI::parseBitInInt32`
  - `"AntiBallisticMissile" => INI::parseBitInInt32`
  - `"AutoReloadsClip" => INI::parseIndexList`
  - `"ProjectileStreamName" => INI::parseAsciiString`
  - `"LaserName" => INI::parseAsciiString`
  - `"LaserBoneName" => INI::parseAsciiString`
  - `"WeaponBonus" => WeaponTemplate::parseWeaponBonusSet`
  - `"HistoricBonusTime" => INI::parseDurationUnsignedInt`
  - `"HistoricBonusRadius" => INI::parseReal`
  - `"HistoricBonusCount" => INI::parseInt`
  - `"HistoricBonusWeapon" => INI::parseWeaponTemplate`
  - `"LeechRangeWeapon" => INI::parseBool`
  - `"ScatterTarget" => WeaponTemplate::parseScatterTarget`
  - `"CapableOfFollowingWaypoints" => INI::parseBool`
  - `"ShowsAmmoPips" => INI::parseBool`
  - `"AllowAttackGarrisonedBldgs" => INI::parseBool`
  - `"PlayFXWhenStealthed" => INI::parseBool`
  - `"PreAttackDelay" => INI::parseDurationUnsignedInt`
  - `"PreAttackType" => INI::parseIndexList`
  - `"ContinueAttackRange" => INI::parseReal`
  - `"SuspendFXDelay" => INI::parseDurationUnsignedInt`
  - `"MissileCallsOnDie" => INI::parseBool`
- **`Snapshot::crc` / `Snapshot::xfer` Serialization Sequence (37 calls — preserve order identically):**
  1. `xferAsciiString(&tmplName)`
  2. `xferUser( &m_wslot, sizeof( WeaponSlotType ) )`
  3. `xferUser( &m_status, sizeof( WeaponStatus ) )`
  4. `xferUnsignedInt( &m_ammoInClip )`
  5. `xferUnsignedInt( &m_whenWeCanFireAgain )`
  6. `xferUnsignedInt( &m_whenPreAttackFinished )`
  7. `xferUnsignedInt( &m_whenLastReloadStarted )`
  8. `xferUnsignedInt( &m_lastFireFrame )`
  9. `xferObjectID( &m_projectileStreamID )`
  10. `xferObjectID( &laserIDUnused )`
  11. `xferInt( &m_maxShotCount )`
  12. `xferInt( &m_curBarrel )`
  13. `xferInt( &m_numShotsForCurBarrel )`
  14. `xferUnsignedShort( &scatterCount )`
  15. `xferInt( &intData )`
  16. `xferBool( &m_pitchLimited )`
  17. `xferBool( &m_leechWeaponRangeActive )`
  18. `xferVersion( &version, currentVersion )`
  19. `xferAsciiString(&tmplName)`
  20. `xferUser( &m_wslot, sizeof( WeaponSlotType ) )`
  21. `xferUser( &m_status, sizeof( WeaponStatus ) )`
  22. `xferUnsignedInt( &m_ammoInClip )`
  23. `xferUnsignedInt( &m_whenWeCanFireAgain )`
  24. `xferUnsignedInt( &m_whenPreAttackFinished )`
  25. `xferUnsignedInt( &m_whenLastReloadStarted )`
  26. `xferUnsignedInt( &m_lastFireFrame )`
  27. `xferUnsignedInt( &m_suspendFXFrame )`
  28. `xferObjectID( &m_projectileStreamID )`
  29. `xferObjectID( &laserIDUnused )`
  30. `xferInt( &m_maxShotCount )`
  31. `xferInt( &m_curBarrel )`
  32. `xferInt( &m_numShotsForCurBarrel )`
  33. `xferUnsignedShort( &scatterCount )`
  34. `xferInt( &intData )`
  35. `xferInt( &intData )`
  36. `xferBool( &m_pitchLimited )`
  37. `xferBool( &m_leechWeaponRangeActive )`
- **PRNG Calls (4 — preserve call order & `lo >= hi` no-advance guard):**
  - `GameLogicRandomValue( m_minDelayBetweenShots, m_maxDelayBetweenShots )`
  - `GameLogicRandomValueReal( 0, scatterRadius )`
  - `GameLogicRandomValueReal( 0, 2*PI )`
  - `GameLogicRandomValue( 0, m_scatterTargetsUnused.size() - 1 )`
- **Compile-Time `RETAIL_COMPATIBLE_*` Guards (translate to `#[cfg(feature = "retail-crc")]`):**
  - `#if RETAIL_COMPATIBLE_CRC || PRESERVE_SNIPING_EMPTY_STINGER_SITES`
  - `#if RETAIL_COMPATIBLE_CRC || PRESERVE_UNRELIABLE_FIRESTORMS`

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

### `GeneralsMD/Code/GameEngine/Include/GameLogic/Weapon.h`
```cpp
// FILE: Weapon.h /////////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   Weapon Descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

#pragma once

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "Common/AudioEventRTS.h"
#include "Common/GameCommon.h"

#include "GameLogic/Damage.h"

#include "WWMath/matrix3d.h"

// FORWARD REFERENCES /////////////////////////////////////////////////////////////////////////////
struct FieldParse;
class FXList;
class ObjectCreationList;
class ThingTemplate;
class Object;
class Weapon;
class WeaponTemplate;
class INI;
class ParticleSystemTemplate;
enum NameKeyType CPP_11(: Int);

//-------------------------------------------------------------------------------------------------
const Int NO_MAX_SHOTS_LIMIT = 0x7fffffff;

//-------------------------------------------------------------------------------------------------
enum WeaponReloadType CPP_11(: Int)
{
	AUTO_RELOAD,
	NO_RELOAD,
	RETURN_TO_BASE_TO_RELOAD,

	WEAPON_RELOAD_COUNT
};

#ifdef DEFINE_WEAPONRELOAD_NAMES
static const char *const TheWeaponReloadNames[] =
{
	"YES",
	"NO",
	"RETURN_TO_BASE",
	nullptr
};
static_assert(ARRAY_SIZE(TheWeaponReloadNames) == WEAPON_RELOAD_COUNT + 1, "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
enum WeaponPrefireType CPP_11(: Int)
{
	PREFIRE_PER_SHOT,		///< Use the prefire delay for every shot we make
	PREFIRE_PER_ATTACK,	///< Use the prefire delay each time we attack a new target
	PREFIRE_PER_CLIP,		///< Use the prefire attack delay for each new clip

	PREFIRE_COUNT
};

#ifdef DEFINE_WEAPONPREFIRE_NAMES
static const char *const TheWeaponPrefireNames[] =
{
	"PER_SHOT",
	"PER_ATTACK",
	"PER_CLIP",
	nullptr
};
static_assert(ARRAY_SIZE(TheWeaponPrefireNames) == PREFIRE_COUNT + 1, "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
enum WeaponAntiMaskType CPP_11(: Int)
{
	WEAPON_ANTI_AIRBORNE_VEHICLE	= 0x01,
	WEAPON_ANTI_GROUND						= 0x02,
	WEAPON_ANTI_PROJECTILE				= 0x04,
	WEAPON_ANTI_SMALL_MISSILE			= 0x08, // All missiles are also projectiles (but not all projectiles are missiles)
	WEAPON_ANTI_MINE							= 0x10,
	WEAPON_ANTI_AIRBORNE_INFANTRY	= 0x20, // When set, anti-air weapons will not target infantry paratroopers.
	WEAPON_ANTI_BALLISTIC_MISSILE	= 0x40, // Specifically large missiles that are vulnerable to base defenses
	WEAPON_ANTI_PARACHUTE					= 0x80, // Parachutes can be targeted directly
};

//-------------------------------------------------------------------------------------------------
enum WeaponAffectsMaskType CPP_11(: Int)
{
	WEAPON_AFFECTS_SELF						= 0x01,
	WEAPON_AFFECTS_ALLIES					= 0x02,
	WEAPON_AFFECTS_ENEMIES				= 0x04,	// that is, enemies that aren't the primary target
	WEAPON_AFFECTS_NEUTRALS				= 0x08,	// ditto
	WEAPON_KILLS_SELF							= 0x10,	// ensures that it's not possible to survive self damage
	WEAPON_DOESNT_AFFECT_SIMILAR	= 0x20, // Doesn't affect others that are the same as us (like other terrorists for a terrorist bomb to prevent chain reaction)
	WEAPON_DOESNT_AFFECT_AIRBORNE	= 0x40, // Radius damage doesn't affect airborne units, unless they are the primary target. (used for poison fields.)

	WEAPON_AFFECTS_MASK_TYPE_END // keep after the last named flag
};

//#ifdef DEFINE_WEAPONAFFECTSMASK_NAMES ; Removed protection so other classes can use these strings... not sure why this was protected in the 1st place
static const char *const TheWeaponAffectsMaskNames[] =
{
	"SELF",
	"ALLIES",
	"ENEMIES",
	"NEUTRALS",
	"SUICIDE",
	"NOT_SIMILAR",
	"NOT_AIRBORNE",
	nullptr
};
static_assert(equalsEnumBitFlagsCount(ARRAY_SIZE(TheWeaponAffectsMaskNames) - 1, WEAPON_AFFECTS_MASK_TYPE_END), "Incorrect array size");
//#endif

//-------------------------------------------------------------------------------------------------
enum WeaponCollideMaskType CPP_11(: Int)
{
	// all of these apply to *nontargeted* things that might just happen to get in the way...
	// the target can always be collided with, regardless of flags
	WEAPON_COLLIDE_ALLIES									= 0x0001,
	WEAPON_COLLIDE_ENEMIES								= 0x0002,
	WEAPON_COLLIDE_STRUCTURES							= 0x0004,	// this is "all structures EXCEPT for structures belonging to the projectile's controller".
	WEAPON_COLLIDE_SHRUBBERY							= 0x0008,
	WEAPON_COLLIDE_PROJECTILE							= 0x0010,
	WEAPON_COLLIDE_WALLS									= 0x0020,
	WEAPON_COLLIDE_SMALL_MISSILES					= 0x0040, //All missiles are also projectiles!
	WEAPON_COLLIDE_BALLISTIC_MISSILES			= 0x0080, //All missiles are also projectiles!
	WEAPON_COLLIDE_CONTROLLED_STRUCTURES	= 0x0100,	//this is "ONLY structures belonging to the projectile's controller".

	WEAPON_COLLIDE_MASK_TYPE_END // keep after the last named flag
};

#ifdef DEFINE_WEAPONCOLLIDEMASK_NAMES
static const char *const TheWeaponCollideMaskNames[] =
{
	"ALLIES",
	"ENEMIES",
	"STRUCTURES",
	"SHRUBBERY",
	"PROJECTILES",
	"WALLS",
	"SMALL_MISSILES",			//All missiles are also projectiles!
	"BALLISTIC_MISSILES", //All missiles are also projectiles!
	"CONTROLLED_STRUCTURES",
	nullptr
};
static_assert(equalsEnumBitFlagsCount(ARRAY_SIZE(TheWeaponCollideMaskNames) - 1, WEAPON_COLLIDE_MASK_TYPE_END), "Incorrect array size");
#endif

//-------------------------------------------------------------------------------------------------
//
// Note: these values are saved in save files, so you MUST NOT REMOVE OR CHANGE
// existing values!
//
enum WeaponBonusConditionType CPP_11(: Int)
{
	// The access and use of this enum has the bit shifting built in, so this is a 0,1,2,3,4,5 enum
	WEAPONBONUSCONDITION_INVALID = -1,

	WEAPONBONUSCONDITION_GARRISONED = 0,
	WEAPONBONUSCONDITION_HORDE,
	WEAPONBONUSCONDITION_CONTINUOUS_FIRE_MEAN,
	WEAPONBONUSCONDITION_CONTINUOUS_FIRE_FAST,
	WEAPONBONUSCONDITION_NATIONALISM,
	WEAPONBONUSCONDITION_PLAYER_UPGRADE,
	WEAPONBONUSCONDITION_DRONE_SPOTTING,
#ifdef ALLOW_DEMORALIZE
	WEAPONBONUSCONDITION_DEMORALIZED,
#else
	WEAPONBONUSCONDITION_DEMORALIZED_OBSOLETE,
#endif
	WEAPONBONUSCONDITION_ENTHUSIASTIC,
	WEAPONBONUSCONDITION_VETERAN,
	WEAPONBONUSCONDITION_ELITE,
	WEAPONBONUSCONDITION_HERO,
	WEAPONBONUSCONDITION_BATTLEPLAN_BOMBARDMENT,
	WEAPONBONUSCONDITION_BATTLEPLAN_HOLDTHELINE,
	WEAPONBONUSCONDITION_BATTLEPLAN_SEARCHANDDESTROY,
	WEAPONBONUSCONDITION_SUBLIMINAL,
	WEAPONBONUSCONDITION_SOLO_HUMAN_EASY,
	WEAPONBONUSCONDITION_SOLO_HUMAN_NORMAL,
	WEAPONBONUSCONDITION_SOLO_HUMAN_HARD,
	WEAPONBONUSCONDITION_SOLO_AI_EASY,
	WEAPONBONUSCONDITION_SOLO_AI_NORMAL,
	WEAPONBONUSCONDITION_SOLO_AI_HARD,
	WEAPONBONUSCONDITION_TARGET_FAERIE_FIRE,
  WEAPONBONUSCONDITION_FANATICISM, // FOR THE NEW GC INFANTRY GENERAL... adds to nationalism
	WEAPONBONUSCONDITION_FRENZY_ONE,
	WEAPONBONUSCONDITION_FRENZY_TWO,
	WEAPONBONUSCONDITION_FRENZY_THREE,

	WEAPONBONUSCONDITION_COUNT
};
#ifdef DEFINE_WEAPONBONUSCONDITION_NAMES
static const char *const TheWeaponBonusNames[] =
{
	// This is a RHS enum (weapon.ini will have WeaponBonus = IT) so it is all caps
	"GARRISONED",
	"HORDE",
	"CONTINUOUS_FIRE_MEAN",
	"CONTINUOUS_FIRE_FAST",
	"NATIONALISM",
	"PLAYER_UPGRADE",
	"DRONE_SPOTTING",
#ifdef ALLOW_DEMORALIZE
	"DEMORALIZED",
#else
	"DEMORALIZED_OBSOLETE",
#endif
	"ENTHUSIASTIC",
	"VETERAN",
	"ELITE",
	"HERO",
	"BATTLEPLAN_BOMBARDMENT",
	"BATTLEPLAN_HOLDTHELINE",
	"BATTLEPLAN_SEARCHANDDESTROY",
	"SUBLIMINAL",
	"SOLO_HUMAN_EASY",
	"SOLO_HUMAN_NORMAL",
	"SOLO_HUMAN_HARD",
	"SOLO_AI_EASY",
	"SOLO_AI_NORMAL",
	"SOLO_AI_HARD",
	"TARGET_FAERIE_FIRE",
  "FANATICISM", // FOR THE NEW GC INFANTRY GENERAL... adds to nationalism
	"FRENZY_ONE",
	"FRENZY_TWO",
	"FRENZY_THREE",

	nullptr
};
static_assert(ARRAY_SIZE(TheWeaponBonusNames) == WEAPONBONUSCONDITION_COUNT + 1, "Incorrect array size");
#endif

// For WeaponBonusConditionFlags
// part of detangling
#include "GameLogic/WeaponBonusConditionFlags.h"
#include "GameLogic/WeaponStatus.h"

//-------------------------------------------------------------------------------------------------
class WeaponBonus
{
public:

	enum Field
	{
		DAMAGE = 0,
		RADIUS,
		RANGE,
		RATE_OF_FIRE,
		PRE_ATTACK,

		FIELD_COUNT
	};

	WeaponBonus()
	{
		clear();
	}

	void clear()
	{
		for (int i = 0; i < FIELD_COUNT; ++i)
			m_field[i] = 1.0f;
	}

	Real getField(Field f) const { return m_field[f]; }
	void setField(Field f, Real v) { m_field[f] = v; }

	void appendBonuses(WeaponBonus& bonus) const;

private:
	Real m_field[FIELD_COUNT];

};

#ifdef DEFINE_WEAPONBONUSFIELD_NAMES
static const char *const TheWeaponBonusFieldNames[] =
{
	"DAMAGE",
	"RADIUS",
	"RANGE",
	"RATE_OF_FIRE",
	"PRE_ATTACK",
	nullptr
};
static_assert(ARRAY_SIZE(TheWeaponBonusFieldNames) == WeaponBonus::FIELD_COUNT + 1, "Incorrect array size");
#endif


//-------------------------------------------------------------------------------------------------
class WeaponBonusSet : public MemoryPoolObject
{
	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( WeaponBonusSet, "WeaponBonusSet" )
private:
	WeaponBonus m_bonus[WEAPONBONUSCONDITION_COUNT];

public:
	void appendBonuses(WeaponBonusConditionFlags flags, WeaponBonus& bonus) const;

	void parseWeaponBonusSet(INI* ini);
	static void parseWeaponBonusSet(INI* ini, void *instance, void* /*store*/, const void* /*userData*/);
	static void parseWeaponBonusSetPtr(INI* ini, void *instance, void* /*store*/, const void* /*userData*/);
};
EMPTY_DTOR(WeaponBonusSet)

//-------------------------------------------------------------------------------------------------
struct HistoricWeaponDamageInfo
{
	// The time and location this weapon was fired
	UnsignedInt						frame;
	Coord3D								location;
	UnsignedInt						triggerId; ///< Unique Id assigned to any grouped damage instances

	HistoricWeaponDamageInfo(UnsignedInt f, const Coord3D& l) :
		frame(f), location(l), triggerId(0)
	{
	}
};

typedef std::list<HistoricWeaponDamageInfo> HistoricWeaponDamageList;

//-------------------------------------------------------------------------------------------------
class WeaponTemplate : public MemoryPoolObject
{
	friend class WeaponStore;

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( WeaponTemplate, "WeaponTemplate" )

public:

	WeaponTemplate();
	// virtual destructor declared by memory pool

	void reset();

	void friend_setNextTemplate(WeaponTemplate *nextTemplate) { m_nextTemplate = nextTemplate; }
	WeaponTemplate *friend_clearNextTemplate() {	WeaponTemplate *ret = m_nextTemplate; m_nextTemplate = nullptr; return ret; }
	Bool isOverride() { return m_nextTemplate != nullptr; }

	/// field table for loading the values from an INI
	const FieldParse *getFieldParse() const { return TheWeaponTemplateFieldParseTable; }

	/**
		fire the weapon. return the logic-frame in which the damage will be dealt.

		If the damage will be determined at an indeterminate later date (eg, via Projectile),
		or will never be dealt (eg, target was out of range), return zero.

		You may not pass null for source or target.
	*/
	UnsignedInt fireWeaponTemplate
	(
		const Object *sourceObj,
		WeaponSlotType wslot,
		Int specificBarrelToUse,
		Object *victimObj,
		const Coord3D* victimPos,
		const WeaponBonus& bonus,
		Bool isProjectileDetonation,
		Bool ignoreRanges,
		Weapon *firingWeapon,
		ObjectID* projectileID,
		Bool inflictDamage
	) const;

	/**
		return the estimate damage that would be done to the given target, taking bonuses, armor, etc
		into account. (this isn't guaranteed to be 100% accurate; it is intended to be used to
		decide which weapon should be used when a unit has multiple weapons.) Note that it DOES NOT
		take weapon range into account -- it ASSUMES that the victim is within range!
	*/
	Real estimateWeaponTemplateDamage(
		const Object *sourceObj,
		const Object *victimObj,
		const Coord3D* victimPos,
		const WeaponBonus& bonus
	) const;

	Real getAttackRange(const WeaponBonus& bonus) const;
	Real getUnmodifiedAttackRange() const;
	Real getMinimumAttackRange() const;
	Int getDelayBetweenShots(const WeaponBonus& bonus) const;
	Int getClipReloadTime(const WeaponBonus& bonus) const;
	Real getPrimaryDamage(const WeaponBonus& bonus) const;
	Real getPrimaryDamageRadius(const WeaponBonus& bonus) const;
	Real getSecondaryDamage(const WeaponBonus& bonus) const;
	Real getSecondaryDamageRadius(const WeaponBonus& bonus) const;
	Int getPreAttackDelay(const WeaponBonus& bonus) const;
	Bool isContactWeapon() const;

	Real getShockWaveAmount() const { return m_shockWaveAmount; }
	Real getShockWaveRadius() const { return m_shockWaveRadius; }
	Real getShockWaveTaperOff() const { return m_shockWaveTaperOff; }

	Real getRequestAssistRange() const {return m_requestAssistRange;}
	AsciiString getName() const { return m_name; }
	AsciiString getProjectileStreamName() const { return m_projectileStreamName; }
	AsciiString getLaserName() const { return m_laserName; }
	const AsciiString& getLaserBoneName() const { return m_laserBoneName; }
	NameKeyType getNameKey() const { return m_nameKey; }
	Real getWeaponSpeed() const { return m_weaponSpeed; }
	Real getMinWeaponSpeed() const { return m_minWeaponSpeed; }
	Bool isScaleWeaponSpeed() const { return m_isScaleWeaponSpeed; }
	Real getWeaponRecoilAmount() const { return m_weaponRecoil; }
	Real getMinTargetPitch() const { return m_minTargetPitch; }
	Real getMaxTargetPitch() const { return m_maxTargetPitch; }
	Real getRadiusDamageAngle() const { return m_radiusDamageAngle; }
	DamageType getDamageType() const { return m_damageType; }
	ObjectStatusTypes getDamageStatusType() const { return m_damageStatusType; }
	DeathType getDeathType() const { return m_deathType; }
	Real getContinueAttackRange() const { return m_continueAttackRange; }
	Real getInfantryInaccuracyDist() const { return m_infantryInaccuracyDist; }
	Real getAimDelta() const { return m_aimDelta; }
	Real getScatterRadius() const { return m_scatterRadius; }
	Real getScatterTargetScalar() const { return m_scatterTargetScalar; }
	const ThingTemplate* getProjectileTemplate() const { return m_projectileTmpl; }
	Bool getDamageDealtAtSelfPosition() const { return m_damageDealtAtSelfPosition; }
	Int getAffectsMask() const { return m_affectsMask; }
	Int getProjectileCollideMask() const { return m_collideMask; }
	WeaponReloadType getReloadType() const { return m_reloadType; }
	WeaponPrefireType getPrefireType() const { return m_prefireType; }
	Bool getAutoReloadsClip() const { return m_reloadType == AUTO_RELOAD; }
	Int getClipSize() const { return m_clipSize; }
	Int getContinuousFireOneShotsNeeded() const { return m_continuousFireOneShotsNeeded; }
	Int getContinuousFireTwoShotsNeeded() const { return m_continuousFireTwoShotsNeeded; }
	UnsignedInt getContinuousFireCoastFrames() const { return m_continuousFireCoastFrames; }
 	UnsignedInt getAutoReloadWhenIdleFrames() const { return m_autoReloadWhenIdleFrames; }
	UnsignedInt getSuspendFXDelay() const { return m_suspendFXDelay; }

	const FXList* getFireFX(VeterancyLevel v) const { return m_fireFXs[v]; }
	const FXList* getProjectileDetonateFX(VeterancyLevel v) const { return m_projectileDetonateFXs[v]; }
	const ObjectCreationList* getFireOCL(VeterancyLevel v) const { return m_fireOCLs[v]; }
	const ObjectCreationList* getProjectileDetonationOCL(VeterancyLevel v) const { return m_projectileDetonationOCLs[v]; }
	const ParticleSystemTemplate* getProjectileExhaust(VeterancyLevel v) const { return m_projectileExhausts[v]; }

	const AudioEventRTS& getFireSound() const { return m_fireSound; }
	UnsignedInt getFireSoundLoopTime() const { return m_fireSoundLoopTime; }
	const std::vector<Coord2D>& getScatterTargetsVector() const { return m_scatterTargets; }
	const WeaponBonusSet* getExtraBonus() const { return m_extraBonus; }
	Int getShotsPerBarrel() const { return m_shotsPerBarrel; }
	Int getAntiMask() const { return m_antiMask; }
	Bool isLeechRangeWeapon() const { return m_leechRangeWeapon; }
	Bool isCapableOfFollowingWaypoint() const { return m_capableOfFollowingWaypoint; }
	Bool isShowsAmmoPips() const { return m_isShowsAmmoPips; }
	Bool isPlayFXWhenStealthed() const { return m_playFXWhenStealthed; }
	Bool getDieOnDetonate() const { return m_dieOnDetonate; }

	Bool shouldProjectileCollideWith(
		const Object* projectileLauncher,
		const Object* projectile,
		const Object* thingWeCollidedWith,
		ObjectID intendedVictimID	// could be INVALID_ID for a position-shot
	) const;

	void postProcessLoad();

protected:

	// actually deal out the damage.
	void dealDamageInternal(ObjectID sourceID, ObjectID victimID, const Coord3D *pos, const WeaponBonus& bonus, Bool isProjectileDetonation) const;
	void trimOldHistoricDamage() const;
	void trimTriggeredHistoricDamage() const;
	void processHistoricDamage(const Object* source, const Coord3D* pos) const;

private:

	// NOTE: m_nextTemplate will be cleaned up if it is NON-nullptr.
	WeaponTemplate *m_nextTemplate;

	static void parseWeaponBonusSet( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ );
	static void parseScatterTarget( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ );
	static void parseShotDelay( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ );

	static const FieldParse TheWeaponTemplateFieldParseTable[];		///< the parse table for INI definition

	AsciiString m_name;											///< name for this weapon
	NameKeyType m_nameKey;									///< unique name key for this weapon template
	AsciiString m_projectileStreamName;			///< Name of object that tracks are stream, if we have one
	AsciiString m_laserName;								///< Name of the laser object that persists.
	AsciiString m_laserBoneName;						///< Where to put the laser object
	Real m_primaryDamage;										///< primary damage amount
	Real m_primaryDamageRadius;							///< primary damage radius range
	Real m_secondaryDamage;									///< secondary damage amount
	Real m_secondaryDamageRadius;						///< secondary damage radius range
	Real m_shockWaveAmount;									///( How much shockwave generated
	Real m_shockWaveRadius;									///( How far shockwave effect affects objects
	Real m_shockWaveTaperOff;								///( How much shockwave is left at the tip of the shockwave radius
	Real m_attackRange;											///< max distance the weapon can deal damage
	Real m_minimumAttackRange;							///< Min distance the weapon should be fired from
	Real m_requestAssistRange;							///< My object will look this far around to get people to join in the attack.
	Real m_aimDelta;												///< when aiming, consider yourself "aimed" if you are within +/- this much of an angle
	Real m_scatterRadius;										///< Radius of area actual fire point will be in, default is zero for no deviation
	Real m_scatterTargetScalar;							///< Radius of area covered by the coordinates in the scatterTarget table
	std::vector<Coord2D> m_scatterTargets;	///< instead of pure randomness, this is the list of places I will randomly choose from to attack
	DamageType m_damageType;								///< damage type enum
	DeathType m_deathType;									///< death type enum
	Real m_weaponSpeed;											///< speed of damage travel, in dist/frame
	Real m_minWeaponSpeed;									///< speed of damage travel, in dist/frame
	Bool m_isScaleWeaponSpeed;							///< Scale from min to normal based on range (for lobbers)
	Real m_weaponRecoil;										///< amt of recoil caused to firer, in rads
	Real m_minTargetPitch;									///< min pitch from source->victim allowable in order to target
	Real m_maxTargetPitch;									///< max pitch from source->victim allowable in order to target
	Real m_radiusDamageAngle;								///< Damage is directional, so max defelection of straight at target (cone) you do damage
	AsciiString m_projectileName;																			///< if projectile, object name to "fire"
	const ThingTemplate* m_projectileTmpl;														///< direct access to projectile object type to "fire"
	AsciiString m_fireOCLNames[LEVEL_COUNT];														///< Name of OCL to create at firing
	AsciiString m_projectileDetonationOCLNames[LEVEL_COUNT];						///< Name of OCL to create at firing at missile's end
	const ParticleSystemTemplate* m_projectileExhausts[LEVEL_COUNT];			///< Templates of particle systems for projectile exhaust
	const ObjectCreationList* m_fireOCLs[LEVEL_COUNT];									///< Post-loaded lookup of name string for ease and speed
	const ObjectCreationList* m_projectileDetonationOCLs[LEVEL_COUNT];	///< Post-loaded lookup of name string for ease and speed (and subsystem init order)
	const FXList* m_fireFXs[LEVEL_COUNT];															///< weapon is fired fx
	const FXList* m_projectileDetonateFXs[LEVEL_COUNT];								///< if we have a projectile, fx for projectile blowing up
	AudioEventRTS m_fireSound;							///< weapon is fired sound
	UnsignedInt m_fireSoundLoopTime;				///< if nonzero, num frames for looping of fire sound
	WeaponBonusSet* m_extraBonus;						///< optional extra per-weapon bonus
	Int m_clipSize;													///< number of 'shots' in a clip
	Int m_clipReloadTime;										///< when 'clip' is empty, how long it takes to reload (frames)
	Int m_minDelayBetweenShots;							///< min time allowed between firing single shots (frames)
	Int m_maxDelayBetweenShots;							///< max time allowed between firing single shots (frames)
	Int m_continuousFireOneShotsNeeded;			///< How many consecutive shots will give my owner the ContinuousFire Property
	Int m_continuousFireTwoShotsNeeded;			///< How many consecutive shots will give my owner the ContinuousFireTwo Property
	UnsignedInt m_continuousFireCoastFrames;///< How long after we should have shot should we start to wind down from Continuous fire mode
 	UnsignedInt m_autoReloadWhenIdleFrames;	///< How long we have to wait after our last shot to force a reload
	Int m_shotsPerBarrel;										///< If non zero, don't cycle through your launch points every shot, mod the shot by this to get even chucks of firing
	Int m_antiMask;													///< what we can target
	Int m_affectsMask;											///< what we can affect
	Int m_collideMask;											///< what we can collide with (projectiles only)
	Bool m_damageDealtAtSelfPosition;				///< if T, weapon damage is done at source's position, not victim's pos. (useful for suicide weapons.)
	WeaponReloadType m_reloadType;					///< does the weapon auto-reload a clip when empty?
	WeaponPrefireType m_prefireType;				///< The way this weapon handles its prefire delay
	UnsignedInt m_historicBonusTime;				///< if 'count' instances of this weapon do damage within 'time' and 'radius' from each other, fire the historic bonus weapon
	Real m_historicBonusRadius;							///< see above
	Int m_historicBonusCount;								///< see above
	const WeaponTemplate* m_historicBonusWeapon;	///< see above
	Bool m_leechRangeWeapon;								///< once the weapon has fired once at the proper range, the weapon gains unlimited range for the remainder of the attack cycle
	Bool m_capableOfFollowingWaypoint;			///< determines if the weapon is capable of following a waypoint path.
	Bool m_isShowsAmmoPips;									///< shows ammo pips
	Bool m_allowAttackGarrisonedBldgs;			///< allow attacks on garrisoned bldgs, even if estimated damage would be zero
	Bool m_playFXWhenStealthed;					///< Ignores rule about not playing FX when stealthed
	Int m_preAttackDelay;										///< doesn't attack until preAttack delay is finish (triggering detonation, aiming a snipe shot, etc.)
	Real m_continueAttackRange;							///< if nonzero: when you destroy something, look for a similar obj controlled by same player to attack (used mainly for mine-clearing)
	Real m_infantryInaccuracyDist;					///< When this weapon is used against infantry, it can randomly miss by as much as this distance.
	ObjectStatusTypes m_damageStatusType;		///< If our damage is Status damage, the status we apply
	UnsignedInt m_suspendFXDelay;						///< The fx can be suspended for any delay, in frames, then they will execute as normal
	Bool m_dieOnDetonate;

	mutable HistoricWeaponDamageList m_historicDamage;
	mutable UnsignedInt m_historicDamageTriggerId;
};

// ---------------------------------------------------------
class Weapon : public MemoryPoolObject,
							 public Snapshot
{
	friend class WeaponStore;

	MEMORY_POOL_GLUE_WITH_USERLOOKUP_CREATE( Weapon, "Weapon" )

private:

	// only our friend (WeaponStore) is allowed to use our ctor
	// Weapon(); -- nope, no default ctor anymore! (srj)
	Weapon(const Weapon& that);
	Weapon& operator=(const Weapon& that);

protected:

	// snapshot methods
	virtual void crc( Xfer *xfer ) override;
	virtual void xfer( Xfer *xfer ) override;
	virtual void loadPostProcess() override;

public:

//~Weapon();

	// return true if we auto-reloaded our clip after firing.
	Bool fireWeapon(const Object *source, Object *target, ObjectID* projectileID = nullptr);

	// return true if we auto-reloaded our clip after firing.
	Bool fireWeapon(const Object *source, const Coord3D* pos, ObjectID* projectileID = nullptr);

	void fireProjectileDetonationWeapon(const Object *source, Object *target, WeaponBonusConditionFlags extraBonusFlags, Bool inflictDamage = TRUE );

	void fireProjectileDetonationWeapon(const Object *source, const Coord3D* pos, WeaponBonusConditionFlags extraBonusFlags, Bool inflictDamage = TRUE );

	void preFireWeapon( const Object *source, const Object *victim );

	//Currently, this function was added to allow a script to force fire a weapon,
	//and immediately gain control of the weapon that was fired to give it special orders...
	Object* forceFireWeapon( const Object *source, const Coord3D *pos );

	/**
		return the estimate damage that would be done to the given target, taking bonuses, armor, etc
		into account. (this isn't guaranteed to be 100% accurate; it is intended to be used to
		decide which weapon should be used when a unit has multiple weapons.) Note that it DOES NOT
		take weapon range into account -- it ASSUMES that the victim is within range!
	*/
	Real estimateWeaponDamage(const Object *source, const Object *target)
	{
		return estimateWeaponDamage(source, target, nullptr);
	}

	Real estimateWeaponDamage(const Object *source, const Coord3D* pos)
	{
		return estimateWeaponDamage(source, nullptr, pos);
	}

	void onWeaponBonusChange(const Object *source);///< Our Object's weapon bonus changed, so we need to update to reflect that instead of waiting

	/** return true if the target is within attack range, false otherwise.
	*/
	Bool isWithinAttackRange(const Object *source, const Object *target) const;
	Bool isWithinAttackRange(const Object *source, const Coord3D* pos) const;

	Bool isTooClose(const Object *source, const Object *target) const;
	Bool isTooClose(const Object *source, const Coord3D *pos) const;

	Bool isGoalPosWithinAttackRange(const Object *source, const Coord3D* goalPos, const Object *target, const Coord3D* targetPos) const;

	//Used only by garrison contains that move objects around before doing the range check.
	//If target object is specified, we'll use his position, but if it's nullptr we will use the
	//target position passed in.
	//NOTE: This is not a user friendly function -- use with caution if at all! -- Kris
	Bool isSourceObjectWithGoalPositionWithinAttackRange(const Object *source, const Coord3D *goalPos, const Object *target, const Coord3D *targetPos) const;

	/**
		Compute a target position from which 'source' can target 'target'. Note that this doesn't
		guarantee that source can actually move to the location, it just guarantees that the
		target position is plausibly within attack range. returns TRUE if the source's current pos
		is already close enough (in which case that current pos is stuffed into the return arg).
	*/
	Bool computeApproachTarget(const Object *source, const Object *target, const Coord3D *pos, Real angleOffset, Coord3D& approachTargetPos) const;

	/**
		note that "load" makes the ammo instantly available, while "reload" keeps the weapon in the RELOADING
		state until the clip-reload-time is up. Normally you should use "load" only for newly-created
		units, who presumably come into existence with fully loaded weapons.
	*/
	void loadAmmoNow(const Object *source);
	void reloadAmmo(const Object *source);

	WeaponStatus getStatus() const;

	/// Some AI needs to know when the soonest I can possibly next shoot.  Bonuses have been figured into this number already.
	UnsignedInt getPossibleNextShotFrame() const { return m_whenWeCanFireAgain; }
	UnsignedInt getPreAttackFinishedFrame() const { return m_whenPreAttackFinished; }
	UnsignedInt getLastReloadStartedFrame() const { return m_whenLastReloadStarted; }
	Real getPercentReadyToFire() const;

	// do not ever use this unless you are weaponset.cpp
	void setPossibleNextShotFrame( UnsignedInt frameNum ) { m_whenWeCanFireAgain = frameNum; }
	void setPreAttackFinishedFrame( UnsignedInt frameNum ) { m_whenPreAttackFinished = frameNum; }
	void setLastReloadStartedFrame( UnsignedInt frameNum ) { m_whenLastReloadStarted = frameNum; }

	//Transfer the reload times and status from the passed in weapon.
	void transferNextShotStatsFrom( const Weapon &weapon );

	// we must pass the source object for these (and for ANY FUTURE ADDITIONS)
	// so that we can take the source's weapon bonuses, if any, into account.
	// Also note: you should RARELY need to call getAttackRange. If what you want is to
	// determine if you are within attack range, please call isWithinAttackRange instead.
	Real getAttackRange(const Object *source) const;

	// Returns the max distance between the centerpoints of source & victim	for victim to be in range.
	Real getAttackDistance(const Object *source, const Object *victim, const Coord3D* victimPos) const;

	void newProjectileFired( const Object *sourceObj, const Object *projectile, const Object *victimObj, const Coord3D *victimPos );///<I just made this projectile and may need to keep track of it

	Bool isLaser() const { return m_template->getLaserName().isNotEmpty(); }
	void createLaser( const Object *sourceObj, const Object *victimObj, const Coord3D *victimPos );

	const WeaponTemplate* getTemplate() const { return m_template; }
	WeaponSlotType getWeaponSlot() const { return m_wslot; }
	AsciiString getName() const { return m_template->getName(); }
	UnsignedInt getLastShotFrame() const { return m_lastFireFrame; }						///< frame a shot was last fired on
	// If we are "reloading", then m_ammoInClip is a lie.  It will say full.
	UnsignedInt getRemainingAmmo() const { return (getStatus() == RELOADING_CLIP) ? 0 : m_ammoInClip; }
	WeaponReloadType getReloadType() const { return m_template->getReloadType(); }
	Bool getAutoReloadsClip() const { return m_template->getAutoReloadsClip(); }
	Real getAimDelta() const { return m_template->getAimDelta(); }
	Real getScatterRadius() const { return m_template->getScatterRadius(); }
	Real getScatterTargetScalar() const { return m_template->getScatterTargetScalar(); }
	Int getAntiMask() const { return m_template->getAntiMask(); }
	Bool isCapableOfFollowingWaypoint() const { return m_template->isCapableOfFollowingWaypoint(); }
	Int getContinuousFireOneShotsNeeded() const { return m_template->getContinuousFireOneShotsNeeded(); }
	Int getContinuousFireTwoShotsNeeded() const { return m_template->getContinuousFireTwoShotsNeeded(); }
	UnsignedInt getContinuousFireCoastFrames() const { return m_template->getContinuousFireCoastFrames(); }
 	UnsignedInt getAutoReloadWhenIdleFrames() const { return m_template->getAutoReloadWhenIdleFrames(); }
	const AudioEventRTS& getFireSound() const { return m_template->getFireSound(); }
	UnsignedInt getFireSoundLoopTime() const { return m_template->getFireSoundLoopTime(); }
	DamageType getDamageType() const { return m_template->getDamageType(); }
	DeathType getDeathType() const { return m_template->getDeathType(); }
	Real getContinueAttackRange() const { return m_template->getContinueAttackRange(); }
	Bool isShowsAmmoPips() const { return m_template->isShowsAmmoPips(); }
	Int getClipSize() const { return m_template->getClipSize(); }
	// Contact weapons (like car bombs) need to basically collide with their target.
	Bool isContactWeapon() const { return m_template->isContactWeapon(); }

	Int getClipReloadTime(const Object *source) const;

	Real getPrimaryDamageRadius(const Object *source) const;

	Int getPreAttackDelay( const Object *source, const Object *victim ) const;

	Bool isDamageWeapon() const;

	Bool isPitchLimited() const { return m_pitchLimited; }
	Bool isWithinTargetPitch(const Object *source, const Object *victim) const;

	//Leech range functionality simply means this weapon has unlimited range temporarily. How it works is if the
	//weapon template has the LeechRangeWeapon set, it means that once the unit has closed to standard weapon range
	//it fires the weapon, and will be able to hit the target even if it moves out of range! The unit will simply
	//stand there. This functionality is used by hack attacks.
	void setLeechRangeActive( Bool active ) { m_leechWeaponRangeActive = active; }
	Bool hasLeechRange() const { return m_leechWeaponRangeActive; }

	void setMaxShotCount(Int maxShots) { m_maxShotCount = maxShots; }
	Int getMaxShotCount() const { return m_maxShotCount; }

	Bool isClearFiringLineOfSightTerrain(const Object* source, const Object* victim) const;
	Bool isClearFiringLineOfSightTerrain(const Object* source, const Coord3D& victimPos) const;

	Bool isClearGoalFiringLineOfSightTerrain(const Object* source, const Coord3D& goalPos, const Object* victim) const;
	Bool isClearGoalFiringLineOfSightTerrain(const Object* source, const Coord3D& goalPos, const Coord3D& victimPos) const;

	static void calcProjectileLaunchPosition(
		const Object* launcher,
		WeaponSlotType wslot,
		Int specificBarrelToUse,
		Matrix3D& worldTransform,
		Coord3D& worldPos
	);

	static void positionProjectileForLaunch(
		Object* projectile,
		const Object *launcher,
		WeaponSlotType wslot,
		Int specificBarrelToUse
	);

	/**
		special purpose call for jets in airfields: directly set the ammoinclip to a certain
		percentage full (note that percent=1.0 is full, NOT percent=100.0!). this will end up
		with status as READY_TO_FIRE or OUT_OF_AMMO, but never RELOADING_CLIP!
	*/
	void setClipPercentFull(Real percent, Bool allowReduction);
	UnsignedInt getSuspendFXFrame() const { return m_suspendFXFrame; }

protected:

	Weapon(const WeaponTemplate* tmpl, WeaponSlotType wslot);

	// return true if we auto-reloaded our clip after firing.
	Bool privateFireWeapon(
		const Object *sourceObj,
		Object *victimObj,
		const Coord3D* victimPos,
		Bool isProjectileDetonation,
		Bool ignoreRanges,
		WeaponBonusConditionFlags extraBonusFlags,
		ObjectID* projectileID,
		Bool inflictDamage
	);
	Real estimateWeaponDamage(const Object *sourceObj, const Object *victimObj, const Coord3D* victimPos);
	void reloadWithBonus(const Object *source, const WeaponBonus& bonus, Bool loadInstantly);

	void processRequestAssistance( const Object *requestingObject, Object *victimObject ); ///< Weapons can call for extra attacks from nearby objects

	void getFiringLineOfSightOrigin(const Object* source, Coord3D& origin) const;

	void computeBonus(const Object *source, WeaponBonusConditionFlags extraBonusFlags, WeaponBonus& bonus) const;

	void rebuildScatterTargets();


private:
	const WeaponTemplate*			m_template;									///< the kind of weapon this is
	WeaponSlotType						m_wslot;										///< are we primary, secondary, etc. weapon? (used for projectile placement on reload)
	mutable WeaponStatus			m_status;										///< weapon status
	UnsignedInt								m_ammoInClip;								///< how many shots left in current clip
	UnsignedInt								m_whenWeCanFireAgain;				///< the first frame the weapon can fire again
	UnsignedInt								m_whenPreAttackFinished;		///< the frame the pre attack will complete.
	UnsignedInt								m_whenLastReloadStarted;		///< the frame the current reload/between-shots began. (only valid if status is RELOADING_CLIP or BETWEEN_FIRING_SHOTS)
	UnsignedInt								m_lastFireFrame;						///< frame a shot was last fired on
	UnsignedInt								m_suspendFXFrame;						///< The fireFX can be suspended for any delay, in frames, then they will execute as normal
	ObjectID									m_projectileStreamID;				///< the object that is tracking our stream if we have one.  It can't go away without us.
	Int												m_maxShotCount;							///< used for limiting consecutive firing
	Int												m_curBarrel;								///< current barrel used for firing
	Int												m_numShotsForCurBarrel;			///< how many shots to fire from cur barrel before moving to next barrel
	std::vector<Int>					m_scatterTargetsUnused;			///< A running memory of which targets I've used, so I can shoot them all at random
	Bool											m_pitchLimited;
	Bool											m_leechWeaponRangeActive;		///< This weapon has unlimited range until attack state is aborted!

	// setter function for status that should not be used outside this class
	void setStatus( WeaponStatus status) { m_status = status; }
};

//-------------------------------------------------------------------------------------------------
/**
	The "store" used to hold all the WeaponTemplates in existence. This is usually used when creating
	an Object, but can be used at any time after that. (It is explicitly
	OK to swap an Object's Weapon out at any given time.)
*/
//-------------------------------------------------------------------------------------------------
class WeaponStore : public SubsystemInterface
{
	friend class WeaponTemplate;

public:

	WeaponStore();
	virtual ~WeaponStore() override;

	virtual void init() override { };
	virtual void postProcessLoad() override;
	virtual void reset() override;
	virtual void update() override;

	/**
		Find the WeaponTemplate with the given name. If no such WeaponTemplate exists, return null.
	*/
	const WeaponTemplate *findWeaponTemplate(const AsciiString& name) const;
	const WeaponTemplate *findWeaponTemplate(const char* name) const;
	const WeaponTemplate *findWeaponTemplateByNameKey( NameKeyType key ) const { return findWeaponTemplatePrivate( key ); }

	// this dynamically allocates a new Weapon, which is owned (and must be freed!) by the caller.
	Weapon* allocateNewWeapon(const WeaponTemplate *tmpl, WeaponSlotType wslot) const
	{
		return newInstance(Weapon)(tmpl, wslot);	// my, that was easy
	}

	void createAndFireTempWeapon(const WeaponTemplate* w, const Object *source, const Coord3D* pos);
	void createAndFireTempWeapon(const WeaponTemplate* w, const Object *source, Object *target);

	void handleProjectileDetonation( const WeaponTemplate* w, const Object *source, const Coord3D* pos, WeaponBonusConditionFlags extraBonusFlags, Bool inflictDamage = TRUE );

	static void parseWeaponTemplateDefinition(INI* ini);

protected:

	WeaponTemplate *findWeaponTemplatePrivate( NameKeyType key ) const;

	WeaponTemplate *newWeaponTemplate( AsciiString name );
	WeaponTemplate *newOverride( WeaponTemplate *weaponTemplate );

	void deleteAllDelayedDamage();
	void resetWeaponTemplates();
	void setDelayedDamage(const WeaponTemplate *weapon, const Coord3D* pos, UnsignedInt whichFrame, ObjectID sourceID, ObjectID victimID, const WeaponBonus& bonus);

private:

	/**
		WeaponDelayedDamageInfo is a utility class used by the WeaponStore to keep track
		of what damage will need to be dealt in the future. It is never used for Projectile
		weaponry, but rather for weapons that do damage in the short-term future without
		instantiating full-fledged Objects as a bullet. (e.g., tank shells do this.)
	*/
	struct WeaponDelayedDamageInfo
	{
	public:
		const WeaponTemplate *m_delayedWeapon;			///< if delayed damage is pending, the weapon to deal the damage
		Coord3D m_delayDamagePos;										///< where to do the delay damage when it's time
		UnsignedInt m_delayDamageFrame;							///< frames we do the damage
		ObjectID m_delaySourceID;										///< who dealt the damage (by ID since it might be dead due to delay)
		ObjectID m_delayIntendedVictimID;						///< who the damage was intended for (or zero if no specific target)
		WeaponBonus m_bonus;												///< the weapon bonus to use
	};

	std::vector<WeaponTemplate*> m_weaponTemplateVector;

	// TheSuperHackers @performance IamInnocent 01/01/2026 - Now additionally stores the same weapon templates in a hash map to optimize lookups by name key
	typedef std::hash_map<NameKeyType, WeaponTemplate*, rts::hash<NameKeyType>, rts::equal_to<NameKeyType>/**/> WeaponTemplateMap;
	WeaponTemplateMap m_weaponTemplateHashMap;

	std::list<WeaponDelayedDamageInfo> m_weaponDDI;
};

// EXTERNALS //////////////////////////////////////////////////////////////////////////////////////
extern WeaponStore *TheWeaponStore;
```

### `GeneralsMD/Code/GameEngine/Source/GameLogic/Object/Weapon.cpp`
```cpp
// FILE: Weapon.cpp ///////////////////////////////////////////////////////////////////////////////
// Author: Colin Day, November 2001
// Desc:   Weapon descriptions
///////////////////////////////////////////////////////////////////////////////////////////////////

// INCLUDES ///////////////////////////////////////////////////////////////////////////////////////
#include "PreRTS.h"	// This must go first in EVERY cpp file in the GameEngine

#define DEFINE_DEATH_NAMES
#define DEFINE_WEAPONBONUSCONDITION_NAMES
#define DEFINE_WEAPONBONUSFIELD_NAMES
#define DEFINE_WEAPONCOLLIDEMASK_NAMES
#define DEFINE_WEAPONAFFECTSMASK_NAMES
#define DEFINE_WEAPONRELOAD_NAMES
#define DEFINE_WEAPONPREFIRE_NAMES

#include "Common/crc.h"
#include "Common/CRCDebug.h"
#include "Common/GameAudio.h"
#include "Common/GameState.h"
#include "Common/INI.h"
#include "Common/PerfTimer.h"
#include "Common/Player.h"
#include "Common/ThingFactory.h"
#include "Common/ThingTemplate.h"
#include "Common/Xfer.h"

#include "GameClient/Drawable.h"
#include "GameClient/FXList.h"
#include "GameClient/InGameUI.h"
#include "GameClient/ParticleSys.h"

#include "GameLogic/Damage.h"
#include "GameLogic/ExperienceTracker.h"
#include "GameLogic/GameLogic.h"
#include "GameLogic/AIPathfind.h"
#include "GameLogic/Module/BehaviorModule.h"
#include "GameLogic/Module/BodyModule.h"
#include "GameLogic/Module/ContainModule.h"
#include "GameLogic/Module/LaserUpdate.h"
#include "GameLogic/Module/UpdateModule.h"
#include "GameLogic/Module/SpecialPowerCompletionDie.h"
#include "GameLogic/Module/AssaultTransportAIUpdate.h"
#include "GameLogic/Object.h"
#include "GameLogic/ObjectCreationList.h"
#include "GameLogic/PartitionManager.h"
#include "GameLogic/Weapon.h"

#include "GameLogic/Module/AIUpdate.h"
#include "GameLogic/Module/AssistedTargetingUpdate.h"
#include "GameLogic/Module/ProjectileStreamUpdate.h"
#include "GameLogic/Module/PhysicsUpdate.h"
#include "GameLogic/Module/SpawnBehavior.h"
#include "GameLogic/TerrainLogic.h"

#define RATIONALIZE_ATTACK_RANGE
#define ATTACK_RANGE_IS_2D

#ifdef ATTACK_RANGE_IS_2D
	const DistanceCalculationType ATTACK_RANGE_CALC_TYPE = FROM_BOUNDINGSPHERE_2D;
#else
	const DistanceCalculationType ATTACK_RANGE_CALC_TYPE = FROM_BOUNDINGSPHERE_3D;
#endif


// damage is ALWAYS 3d
const DistanceCalculationType DAMAGE_RANGE_CALC_TYPE = FROM_BOUNDINGSPHERE_3D;

//-------------------------------------------------------------------------------------------------
static void parsePerVetLevelAsciiString( INI* ini, void* /*instance*/, void * store, const void* /*userData*/ )
{
	AsciiString* s = (AsciiString*)store;
	VeterancyLevel v = (VeterancyLevel)INI::scanIndexList(ini->getNextToken(), TheVeterancyNames);
	s[v] = ini->getNextAsciiString();
}

//-------------------------------------------------------------------------------------------------
static void parseAllVetLevelsAsciiString( INI* ini, void* /*instance*/, void * store, const void* /*userData*/ )
{
	AsciiString* s = (AsciiString*)store;
	AsciiString a = ini->getNextAsciiString();
	for (Int i = LEVEL_FIRST; i <= LEVEL_LAST; ++i)
		s[i] = a;
}

//-------------------------------------------------------------------------------------------------
static void parsePerVetLevelFXList( INI* ini, void* /*instance*/, void * store, const void* /*userData*/ )
{
	typedef const FXList* ConstFXListPtr;
	ConstFXListPtr* s = (ConstFXListPtr*)store;
	VeterancyLevel v = (VeterancyLevel)INI::scanIndexList(ini->getNextToken(), TheVeterancyNames);
	const FXList* fx = nullptr;
	INI::parseFXList(ini, nullptr, &fx, nullptr);
	s[v] = fx;
}

//-------------------------------------------------------------------------------------------------
static void parseAllVetLevelsFXList( INI* ini, void* /*instance*/, void * store, const void* /*userData*/ )
{
	typedef const FXList* ConstFXListPtr;
	ConstFXListPtr* s = (ConstFXListPtr*)store;
	const FXList* fx = nullptr;
	INI::parseFXList(ini, nullptr, &fx, nullptr);
	for (Int i = LEVEL_FIRST; i <= LEVEL_LAST; ++i)
		s[i] = fx;
}

//-------------------------------------------------------------------------------------------------
static void parsePerVetLevelPSys( INI* ini, void* /*instance*/, void * store, const void* /*userData*/ )
{
	typedef const ParticleSystemTemplate* ConstParticleSystemTemplatePtr;
	ConstParticleSystemTemplatePtr* s = (ConstParticleSystemTemplatePtr*)store;
	VeterancyLevel v = (VeterancyLevel)INI::scanIndexList(ini->getNextToken(), TheVeterancyNames);
	ConstParticleSystemTemplatePtr pst = nullptr;
	INI::parseParticleSystemTemplate(ini, nullptr, &pst, nullptr);
	s[v] = pst;
}

//-------------------------------------------------------------------------------------------------
static void parseAllVetLevelsPSys( INI* ini, void* /*instance*/, void * store, const void* /*userData*/ )
{
	typedef const ParticleSystemTemplate* ConstParticleSystemTemplatePtr;
	ConstParticleSystemTemplatePtr* s = (ConstParticleSystemTemplatePtr*)store;
	ConstParticleSystemTemplatePtr pst = nullptr;
	INI::parseParticleSystemTemplate(ini, nullptr, &pst, nullptr);
	for (Int i = LEVEL_FIRST; i <= LEVEL_LAST; ++i)
		s[i] = pst;
}

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC DATA ////////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
WeaponStore *TheWeaponStore = nullptr;					///< the weapon store definition


///////////////////////////////////////////////////////////////////////////////////////////////////
// PRIVATE DATA ///////////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////
const FieldParse WeaponTemplate::TheWeaponTemplateFieldParseTable[] =
{

	{ "PrimaryDamage",						INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_primaryDamage) },
	{ "PrimaryDamageRadius",			INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_primaryDamageRadius) },
	{ "SecondaryDamage",					INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_secondaryDamage) },
	{ "SecondaryDamageRadius",		INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_secondaryDamageRadius) },
	{ "ShockWaveAmount",					INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_shockWaveAmount) },
	{ "ShockWaveRadius",					INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_shockWaveRadius) },
	{ "ShockWaveTaperOff",				INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_shockWaveTaperOff) },
	{ "AttackRange",							INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_attackRange) },
	{ "MinimumAttackRange",				INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_minimumAttackRange) },
	{ "RequestAssistRange",				INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_requestAssistRange) },
	{ "AcceptableAimDelta",				INI::parseAngleReal,										nullptr,							offsetof(WeaponTemplate, m_aimDelta) },
	{ "ScatterRadius",						INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_scatterRadius) },
	{ "ScatterTargetScalar",			INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_scatterTargetScalar) },
	{ "ScatterRadiusVsInfantry",	INI::parseReal,													nullptr,							offsetof( WeaponTemplate, m_infantryInaccuracyDist ) },
	{ "DamageType",								DamageTypeFlags::parseSingleBitFromINI,	nullptr,							offsetof(WeaponTemplate, m_damageType) },
	{ "DamageStatusType",					ObjectStatusMaskType::parseSingleBitFromINI,	nullptr,				offsetof(WeaponTemplate, m_damageStatusType) },
	{ "DeathType",								INI::parseIndexList,										TheDeathNames,		offsetof(WeaponTemplate, m_deathType) },
	{ "WeaponSpeed",							INI::parseVelocityReal,									nullptr,							offsetof(WeaponTemplate, m_weaponSpeed) },
	{ "MinWeaponSpeed",						INI::parseVelocityReal,									nullptr,							offsetof(WeaponTemplate, m_minWeaponSpeed) },
	{ "ScaleWeaponSpeed",					INI::parseBool,													nullptr,							offsetof(WeaponTemplate, m_isScaleWeaponSpeed) },
	{ "WeaponRecoil",							INI::parseAngleReal,										nullptr,							offsetof(WeaponTemplate, m_weaponRecoil) },
	{ "MinTargetPitch",						INI::parseAngleReal,										nullptr,							offsetof(WeaponTemplate, m_minTargetPitch) },
	{ "MaxTargetPitch",						INI::parseAngleReal,										nullptr,							offsetof(WeaponTemplate, m_maxTargetPitch) },
	{ "RadiusDamageAngle",				INI::parseAngleReal,										nullptr,							offsetof(WeaponTemplate, m_radiusDamageAngle) },
	{ "ProjectileObject",					INI::parseAsciiString,									nullptr,							offsetof(WeaponTemplate, m_projectileName) },
	{ "FireSound",								INI::parseAudioEventRTS,								nullptr,							offsetof(WeaponTemplate, m_fireSound) },
	{ "FireSoundLoopTime",				INI::parseDurationUnsignedInt,					nullptr,							offsetof(WeaponTemplate, m_fireSoundLoopTime) },
	{ "FireFX",											parseAllVetLevelsFXList,							nullptr,							offsetof(WeaponTemplate, m_fireFXs) },
	{ "ProjectileDetonationFX",			parseAllVetLevelsFXList,							nullptr,							offsetof(WeaponTemplate, m_projectileDetonateFXs) },
	{ "FireOCL",										parseAllVetLevelsAsciiString,					nullptr,							offsetof(WeaponTemplate, m_fireOCLNames) },
	{ "ProjectileDetonationOCL",		parseAllVetLevelsAsciiString,					nullptr,							offsetof(WeaponTemplate, m_projectileDetonationOCLNames) },
	{ "ProjectileExhaust",					parseAllVetLevelsPSys,								nullptr,							offsetof(WeaponTemplate, m_projectileExhausts) },
	{ "VeterancyFireFX",										parsePerVetLevelFXList,				nullptr,							offsetof(WeaponTemplate, m_fireFXs) },
	{ "VeterancyProjectileDetonationFX",		parsePerVetLevelFXList,				nullptr,							offsetof(WeaponTemplate, m_projectileDetonateFXs) },
	{ "VeterancyFireOCL",										parsePerVetLevelAsciiString,	nullptr,							offsetof(WeaponTemplate, m_fireOCLNames) },
	{ "VeterancyProjectileDetonationOCL",		parsePerVetLevelAsciiString,	nullptr,							offsetof(WeaponTemplate, m_projectileDetonationOCLNames) },
	{ "VeterancyProjectileExhaust",					parsePerVetLevelPSys,					nullptr,							offsetof(WeaponTemplate, m_projectileExhausts) },
	{ "ClipSize",									INI::parseInt,													nullptr,							offsetof(WeaponTemplate, m_clipSize) },
	{ "ContinuousFireOne",				INI::parseInt,													nullptr,							offsetof(WeaponTemplate, m_continuousFireOneShotsNeeded) },
	{ "ContinuousFireTwo",				INI::parseInt,													nullptr,							offsetof(WeaponTemplate, m_continuousFireTwoShotsNeeded) },
	{ "ContinuousFireCoast",			INI::parseDurationUnsignedInt,					nullptr,							offsetof(WeaponTemplate, m_continuousFireCoastFrames) },
 	{ "AutoReloadWhenIdle",				INI::parseDurationUnsignedInt,					nullptr,							offsetof(WeaponTemplate, m_autoReloadWhenIdleFrames) },
	{ "ClipReloadTime",						INI::parseDurationUnsignedInt,					nullptr,							offsetof(WeaponTemplate, m_clipReloadTime) },
	{ "DelayBetweenShots",				WeaponTemplate::parseShotDelay,					nullptr,							0 },
	{ "ShotsPerBarrel",						INI::parseInt,													nullptr,							offsetof(WeaponTemplate, m_shotsPerBarrel) },
	{ "DamageDealtAtSelfPosition",INI::parseBool,													nullptr,							offsetof(WeaponTemplate, m_damageDealtAtSelfPosition) },
	{ "RadiusDamageAffects",			INI::parseBitString32,	TheWeaponAffectsMaskNames,				offsetof(WeaponTemplate, m_affectsMask) },
	{ "ProjectileCollidesWith",		INI::parseBitString32,	TheWeaponCollideMaskNames,				offsetof(WeaponTemplate, m_collideMask) },
	{ "AntiAirborneVehicle",			INI::parseBitInInt32,										(void*)WEAPON_ANTI_AIRBORNE_VEHICLE,	offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiGround",								INI::parseBitInInt32,										(void*)WEAPON_ANTI_GROUND,						offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiProjectile",						INI::parseBitInInt32,										(void*)WEAPON_ANTI_PROJECTILE,				offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiSmallMissile",					INI::parseBitInInt32,										(void*)WEAPON_ANTI_SMALL_MISSILE,			offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiMine",									INI::parseBitInInt32,										(void*)WEAPON_ANTI_MINE,							offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiParachute",						INI::parseBitInInt32,										(void*)WEAPON_ANTI_PARACHUTE,					offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiAirborneInfantry",			INI::parseBitInInt32,										(void*)WEAPON_ANTI_AIRBORNE_INFANTRY, offsetof(WeaponTemplate, m_antiMask) },
	{ "AntiBallisticMissile",			INI::parseBitInInt32,										(void*)WEAPON_ANTI_BALLISTIC_MISSILE, offsetof(WeaponTemplate, m_antiMask) },
	{ "AutoReloadsClip",					INI::parseIndexList,										TheWeaponReloadNames,							offsetof(WeaponTemplate, m_reloadType) },
	{ "ProjectileStreamName",			INI::parseAsciiString,									nullptr,							offsetof(WeaponTemplate, m_projectileStreamName) },
	{ "LaserName",								INI::parseAsciiString,									nullptr,							offsetof(WeaponTemplate, m_laserName) },
	{ "LaserBoneName",						INI::parseAsciiString,									nullptr,							offsetof(WeaponTemplate, m_laserBoneName) },
	{ "WeaponBonus",							WeaponTemplate::parseWeaponBonusSet,		nullptr,							0 },
	{ "HistoricBonusTime",				INI::parseDurationUnsignedInt,					nullptr,							offsetof(WeaponTemplate, m_historicBonusTime) },
	{ "HistoricBonusRadius",			INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_historicBonusRadius) },
	{ "HistoricBonusCount",				INI::parseInt,													nullptr,							offsetof(WeaponTemplate, m_historicBonusCount) },
	{ "HistoricBonusWeapon",			INI::parseWeaponTemplate,								nullptr,							offsetof(WeaponTemplate, m_historicBonusWeapon) },
	{ "LeechRangeWeapon",					INI::parseBool,													nullptr,							offsetof(WeaponTemplate, m_leechRangeWeapon) },
	{ "ScatterTarget",						WeaponTemplate::parseScatterTarget,			nullptr,							0 },
	{ "CapableOfFollowingWaypoints", INI::parseBool,											nullptr,							offsetof(WeaponTemplate, m_capableOfFollowingWaypoint) },
	{ "ShowsAmmoPips",						INI::parseBool,													nullptr,							offsetof(WeaponTemplate, m_isShowsAmmoPips) },
	{ "AllowAttackGarrisonedBldgs", INI::parseBool,												nullptr,							offsetof(WeaponTemplate, m_allowAttackGarrisonedBldgs) },
	{ "PlayFXWhenStealthed",			INI::parseBool,													nullptr,							offsetof(WeaponTemplate, m_playFXWhenStealthed) },
	{ "PreAttackDelay",						INI::parseDurationUnsignedInt,					nullptr,							offsetof( WeaponTemplate, m_preAttackDelay ) },
	{ "PreAttackType",						INI::parseIndexList,										TheWeaponPrefireNames, offsetof(WeaponTemplate, m_prefireType) },
	{ "ContinueAttackRange",			INI::parseReal,													nullptr,							offsetof(WeaponTemplate, m_continueAttackRange) },
	{ "SuspendFXDelay",						INI::parseDurationUnsignedInt,					nullptr,							offsetof(WeaponTemplate, m_suspendFXDelay) },
	{ "MissileCallsOnDie",			INI::parseBool,													nullptr,							offsetof(WeaponTemplate, m_dieOnDetonate) },
	{ nullptr,												nullptr,																		nullptr,							0 }

};

///////////////////////////////////////////////////////////////////////////////////////////////////
// PUBLIC FUNCTIONS ///////////////////////////////////////////////////////////////////////////////
///////////////////////////////////////////////////////////////////////////////////////////////////

//-------------------------------------------------------------------------------------------------
WeaponTemplate::WeaponTemplate() : m_nextTemplate(nullptr)
{

	m_name													= "NoNameWeapon";
	m_nameKey												= NAMEKEY_INVALID;
	m_primaryDamage									= 0.0f;
	m_primaryDamageRadius						= 0.0f;
	m_secondaryDamage								= 0.0f;
	m_secondaryDamageRadius					= 0.0f;
	m_attackRange										= 0.0f;
	m_minimumAttackRange						= 0.0f;
	m_requestAssistRange						= 0.0f;
	m_aimDelta											= 0.0f;
	m_scatterRadius									= 0.0f;
	m_scatterTargetScalar						= 0.0f;
	m_shockWaveAmount								= 0.0f;
	m_shockWaveRadius								= 0.0f;
	m_shockWaveTaperOff							= 0.0f;
	m_damageType										= DAMAGE_EXPLOSION;
	m_deathType											= DEATH_NORMAL;
	m_weaponSpeed										= 999999.0f;	// effectively instant
	m_minWeaponSpeed								= 999999.0f;	// effectively instant
	m_isScaleWeaponSpeed						= FALSE;
	m_weaponRecoil									= 0.0f;		// no recoil
	m_minTargetPitch								= -PI;
	m_maxTargetPitch								= PI;
	m_radiusDamageAngle							= PI;	// PI each way, so full circle
	m_projectileName.clear();					// no projectile
	m_projectileTmpl								= nullptr;
	for (Int i = LEVEL_FIRST; i <= LEVEL_LAST; ++i)
	{
		m_fireOCLNames[i].clear();
		m_projectileDetonationOCLNames[i].clear();
		m_projectileExhausts[i]					= nullptr;
		m_fireOCLs[i]										= nullptr;
		m_projectileDetonationOCLs[i]		= nullptr;
		m_fireFXs[i]										= nullptr;
		m_projectileDetonateFXs[i]			= nullptr;
	}
	m_damageDealtAtSelfPosition			= false;
	m_affectsMask										= (WEAPON_AFFECTS_ALLIES | WEAPON_AFFECTS_ENEMIES | WEAPON_AFFECTS_NEUTRALS);
	// most projectile weapons don't want to collide with nontargeted enemies/allies or trees...
	m_collideMask										= (WEAPON_COLLIDE_STRUCTURES);
	m_reloadType										= AUTO_RELOAD;
	m_prefireType										= PREFIRE_PER_SHOT;
	m_clipSize											= 0;
	m_continuousFireOneShotsNeeded	= INT_MAX;
	m_continuousFireTwoShotsNeeded	= INT_MAX;
	m_continuousFireCoastFrames			= 0;
 	m_autoReloadWhenIdleFrames			= 0;
	m_clipReloadTime								= 0;
	m_minDelayBetweenShots					= 0;
	m_maxDelayBetweenShots					= 0;
	m_fireSoundLoopTime							= 0;
	m_extraBonus										= nullptr;
	m_shotsPerBarrel								= 1;
	m_antiMask											= WEAPON_ANTI_GROUND;	// but not air or projectile.
	m_projectileStreamName.clear();
	m_laserName.clear();
	m_laserBoneName.clear();
	m_historicBonusTime							= 0;
	m_historicBonusCount						= 0;
	m_historicBonusRadius						= 0;
	m_historicBonusWeapon						= nullptr;
	m_leechRangeWeapon							= FALSE;
	m_capableOfFollowingWaypoint		= FALSE;
	m_isShowsAmmoPips								= FALSE;
	m_allowAttackGarrisonedBldgs		= FALSE;
	m_playFXWhenStealthed						= FALSE;
	m_preAttackDelay								= 0;
	m_continueAttackRange						= 0.0f;
	m_infantryInaccuracyDist				= 0.0f;
	m_damageStatusType							= OBJECT_STATUS_NONE;
	m_suspendFXDelay								= 0;
	m_dieOnDetonate						= FALSE;

	m_historicDamageTriggerId = 0;
}

//-------------------------------------------------------------------------------------------------
WeaponTemplate::~WeaponTemplate()
{
	deleteInstance(m_nextTemplate);

	// delete any extra-bonus that's present
	deleteInstance(m_extraBonus);
}

// ------------------------------------------------------------------------------------------------
void WeaponTemplate::reset()
{
	m_historicDamage.clear();
}

//-------------------------------------------------------------------------------------------------
/*static*/ void WeaponTemplate::parseWeaponBonusSet( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	WeaponTemplate* self = (WeaponTemplate*)instance;

	if (!self->m_extraBonus)
		self->m_extraBonus = newInstance(WeaponBonusSet);

	self->m_extraBonus->parseWeaponBonusSet(ini);
}

//-------------------------------------------------------------------------------------------------
/*static*/ void WeaponTemplate::parseScatterTarget( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	// Accept multiple listings of Coord2D's.
	WeaponTemplate* self = (WeaponTemplate*)instance;

	Coord2D target;
	target.x = 0;
	target.y = 0;
	INI::parseCoord2D( ini, nullptr, &target, nullptr );

	self->m_scatterTargets.push_back(target);
}

//-------------------------------------------------------------------------------------------------
/*static*/ void WeaponTemplate::parseShotDelay( INI* ini, void *instance, void * /*store*/, const void* /*userData*/ )
{
	// This smart parser allows both a single number for traditional delay, and a labeled pair of numbers for a delay range
	WeaponTemplate* self = (WeaponTemplate*)instance;
	static const char *MIN_LABEL = "Min";
	static const char *MAX_LABEL = "Max";

	const char* token = ini->getNextToken(ini->getSepsColon());
	if( stricmp(token, MIN_LABEL) == 0 )
	{
		// Two entry min/max
		self->m_minDelayBetweenShots = INI::scanInt(ini->getNextToken(ini->getSepsColon()));
		token = ini->getNextToken(ini->getSepsColon());
		if( stricmp(token, MAX_LABEL) != 0 )
		{
			// Messed up double entry
			self->m_maxDelayBetweenShots = self->m_minDelayBetweenShots;
		}
		else
			self->m_maxDelayBetweenShots = INI::scanInt(ini->getNextToken(ini->getSepsColon()));
	}
	else
	{
		// single entry, as in no label so the first token is just a number
		self->m_minDelayBetweenShots = INI::scanInt(token);
		self->m_maxDelayBetweenShots = self->m_minDelayBetweenShots;
	}

	// No matter what we have now, we want to convert it to frames from msec.
	// ShotDelay used to use parseDurationUnsignedInt, and we are expanding on that.
	self->m_minDelayBetweenShots = WWMath::Ceilf(ConvertDurationFromMsecsToFrames((Real)self->m_minDelayBetweenShots));
	self->m_maxDelayBetweenShots = WWMath::Ceilf(ConvertDurationFromMsecsToFrames((Real)self->m_maxDelayBetweenShots));

}

//-------------------------------------------------------------------------------------------------
void WeaponTemplate::postProcessLoad()
{
	if (!TheThingFactory)
	{
		DEBUG_CRASH(("you must call this after TheThingFactory is inited"));
		return;
	}

	if (m_projectileName.isEmpty())
	{
		m_projectileTmpl = nullptr;
	}
	else
	{
		m_projectileTmpl = TheThingFactory->findTemplate(m_projectileName);
		DEBUG_ASSERTCRASH(m_projectileTmpl, ("projectile %s not found!",m_projectileName.str()));

#ifdef DEBUG_LOGGING
		if (m_projectileTmpl && m_primaryDamage > 0.0)
		{
			const Real projectileRadius = m_projectileTmpl->getTemplateGeometryInfo().getMajorRadius();
			if (m_primaryDamageRadius < projectileRadius)
				DEBUG_LOG(("Weapon template %s has a PrimaryDamageRadius (%f) smaller than its projectile object's GeometryMajorRadius (%f)! This may lead to inconsistent damage application.", getName().str(), m_primaryDamageRadius, projectileRadius));
		}
#endif
	}

	for (Int i = LEVEL_FIRST; i <= LEVEL_LAST; ++i)
	{
		// And the OCL if there is one
		if (m_fireOCLNames[i].isEmpty())
		{
			m_fireOCLs[i] = nullptr;
		}
		else
		{
			m_fireOCLs[i] = TheObjectCreationListStore->findObjectCreationList(m_fireOCLNames[i].str() );
			DEBUG_ASSERTCRASH(m_fireOCLs[i], ("OCL %s not found in a weapon!",m_fireOCLNames[i].str()));
		}
		m_fireOCLNames[i].clear();

		// And the other OCL if there is one
		if (m_projectileDetonationOCLNames[i].isEmpty() )
		{
			m_projectileDetonationOCLs[i] = nullptr;
		}
		else
		{
			m_projectileDetonationOCLs[i] = TheObjectCreationListStore->findObjectCreationList(m_projectileDetonationOCLNames[i].str() );
			DEBUG_ASSERTCRASH(m_projectileDetonationOCLs[i], ("OCL %s not found in a weapon!",m_projectileDetonationOCLNames[i].str()));
		}
		m_projectileDetonationOCLNames[i].clear();
	}

}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getAttackRange(const WeaponBonus& bonus) const
{
#ifdef RATIONALIZE_ATTACK_RANGE
	// Note - undersize by 1/4 of a pathfind cell, so that the goal is not teetering on the edge
	// of firing range.  jba.
	const Real UNDERSIZE = PATHFIND_CELL_SIZE_F*0.25f;
	Real r = m_attackRange * bonus.getField(WeaponBonus::RANGE) - UNDERSIZE;
	if (r < 0.0f) r = 0.0f;
	return r;
#else
// fudge this a little to account for pathfinding roundoff & such
	const Real ATTACK_RANGE_FUDGE = 1.05f;
	return m_attackRange * bonus.getField(WeaponBonus::RANGE) * ATTACK_RANGE_FUDGE;
#endif
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getMinimumAttackRange() const
{
#ifdef RATIONALIZE_ATTACK_RANGE
	// Note - undersize by 1/4 of a pathfind cell, so that the goal is not teetering on the edge
	// of firing range.  jba.
	const Real UNDERSIZE = PATHFIND_CELL_SIZE_F*0.25f;
	Real r = m_minimumAttackRange - UNDERSIZE;
	if (r < 0.0f) r = 0.0f;
	return r;
#else
	return m_minimumAttackRange;
#endif
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getUnmodifiedAttackRange() const
{
	return m_attackRange;
}

//-------------------------------------------------------------------------------------------------
Int WeaponTemplate::getDelayBetweenShots(const WeaponBonus& bonus) const
{
	// yes, divide, not multiply; the larger the rate-of-fire bonus, the shorter
	// we want the delay time to be.
	Int delayToUse;
	if( m_minDelayBetweenShots == m_maxDelayBetweenShots )
		delayToUse = m_minDelayBetweenShots; // Random number thing doesn't like this case
	else
		delayToUse = GameLogicRandomValue( m_minDelayBetweenShots, m_maxDelayBetweenShots );

	Real bonusROF = bonus.getField(WeaponBonus::RATE_OF_FIRE);
	//CRCDEBUG_LOG(("WeaponTemplate::getDelayBetweenShots() - min:%d max:%d val:%d, bonusROF=%g/%8.8X",
		//m_minDelayBetweenShots, m_maxDelayBetweenShots, delayToUse, bonusROF, AS_INT(bonusROF)));

	return REAL_TO_INT_FLOOR(delayToUse / bonusROF);
}

//-------------------------------------------------------------------------------------------------
Int WeaponTemplate::getClipReloadTime(const WeaponBonus& bonus) const
{
	// yes, divide, not multiply; the larger the rate-of-fire bonus, the shorter
	// we want the reload time to be.
	return REAL_TO_INT_FLOOR(m_clipReloadTime / bonus.getField(WeaponBonus::RATE_OF_FIRE));
}

//-------------------------------------------------------------------------------------------------
Int WeaponTemplate::getPreAttackDelay( const WeaponBonus& bonus ) const
{
	return m_preAttackDelay * bonus.getField( WeaponBonus::PRE_ATTACK );
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getPrimaryDamage(const WeaponBonus& bonus) const
{
	return m_primaryDamage * bonus.getField(WeaponBonus::DAMAGE);
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getPrimaryDamageRadius(const WeaponBonus& bonus) const
{
	return m_primaryDamageRadius * bonus.getField(WeaponBonus::RADIUS);
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getSecondaryDamage(const WeaponBonus& bonus) const
{
	return m_secondaryDamage * bonus.getField(WeaponBonus::DAMAGE);
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::getSecondaryDamageRadius(const WeaponBonus& bonus) const
{
	return m_secondaryDamageRadius * bonus.getField(WeaponBonus::RADIUS);
}

//-------------------------------------------------------------------------------------------------
Bool WeaponTemplate::isContactWeapon() const
{
#ifdef RATIONALIZE_ATTACK_RANGE
	// Note - undersize by 1/4 of a pathfind cell, so that the goal is not teetering on the edge
	// of firing range.  jba.
	const Real UNDERSIZE = PATHFIND_CELL_SIZE_F*0.25f;
	return (m_attackRange - UNDERSIZE) < PATHFIND_CELL_SIZE_F;
#else
// fudge this a little to account for pathfinding roundoff & such
	const Real ATTACK_RANGE_FUDGE = 1.05f;
	return m_attackRange * ATTACK_RANGE_FUDGE < PATHFIND_CELL_SIZE_F;
#endif
}

//-------------------------------------------------------------------------------------------------
Real WeaponTemplate::estimateWeaponTemplateDamage(
	const Object *sourceObj,
	const Object *victimObj,
	const Coord3D* victimPos,
	const WeaponBonus& bonus
) const
{
	if (sourceObj == nullptr || (victimObj == nullptr && victimPos == nullptr))
	{
		DEBUG_CRASH(("bad args to estimate"));
		return 0.0f;
	}

	const Real damageAmount = getPrimaryDamage(bonus);
	if ( victimObj == nullptr )
	{
		return damageAmount;
	}

	DamageType damageType = getDamageType();
	DeathType deathType = getDeathType();

	if ( victimObj->isKindOf(KINDOF_SHRUBBERY) )
	{
		if (deathType == DEATH_BURNED)
		{
			// this is just a nonzero value, to ensure we can target shrubbery with flame weapons, regardless...
			return 1.0f;
		}
		else
		{
			return 0.0f;
		}
	}


  // hmm.. must be shooting a firebase or such, if there is noone home to take the bullet, return 0!
  if ( victimObj->isKindOf( KINDOF_STRUCTURE) && damageType == DAMAGE_SNIPER )
  {

#if RETAIL_COMPATIBLE_CRC || PRESERVE_SNIPING_EMPTY_STINGER_SITES
    if ( victimObj->getContain() )
    {
      if ( victimObj->getContain()->getContainCount() == 0 )
        return 0.0f;
    }
#else
		// TheSuperHackers @bugfix Stubbjax 22/06/2026 Only allow targeting Stinger Sites when they contain Soldiers.
		const Bool hasOccupants = victimObj->getContain() && victimObj->getContain()->getContainCount() > 0;
		const Bool hasSlaves = victimObj->getSpawnBehaviorInterface() && victimObj->getSpawnBehaviorInterface()->getSlaveCount() > 0;

		if (!hasOccupants && !hasSlaves)
			return 0.0f;
#endif
  }




	if ( damageType == DAMAGE_SURRENDER || m_allowAttackGarrisonedBldgs )
	{
		ContainModuleInterface* contain = victimObj->getContain();
		if( contain && contain->getContainCount() > 0 && contain->isGarrisonable() && !contain->isImmuneToClearBuildingAttacks() )
		{
			// this is just a nonzero value, to ensure we can target garrisoned things with surrender weapons, regardless...
			return 1.0f;
		}
	}

	if( damageType == DAMAGE_DISARM )
	{
		if( victimObj->isKindOf( KINDOF_MINE ) || victimObj->isKindOf( KINDOF_BOOBY_TRAP ) || victimObj->isKindOf( KINDOF_DEMOTRAP ) )
		{
			// this is just a nonzero value, to ensure we can target mines with disarm weapons, regardless...
			return 1.0f;
		}
		return 0.0f;
	}
	if( damageType == DAMAGE_DEPLOY && !victimObj->isAirborneTarget() )
	{
		return 1.0f;
	}

	//@todo Kris need to examine the DAMAGE_HACK type for damage estimation purposes.
	//Likely this damage type will have threat implications that won't properly be dealt with until resolved.

	DamageInfoInput damageInfo;
	damageInfo.m_damageType = damageType;
	damageInfo.m_deathType = deathType;
	damageInfo.m_sourceID = sourceObj->getID();
	damageInfo.m_amount = damageAmount;
	return victimObj->estimateDamage(damageInfo);
}

//-------------------------------------------------------------------------------------------------
Bool WeaponTemplate::shouldProjectileCollideWith(
	const Object* projectileLauncher,
	const Object* projectile,
	const Object* thingWeCollidedWith,
	ObjectID intendedVictimID	// could be INVALID_ID for a position-shot
) const
{
 	if (!projectile || !thingWeCollidedWith)
 		return false;

	// if it's our intended victim, we want to collide with it, regardless of any other considerations.
	if (intendedVictimID == thingWeCollidedWith->getID())
		return true;

 	if (projectileLauncher != nullptr)
 	{

 		// Don't hit your own launcher, ever.
 		if (projectileLauncher == thingWeCollidedWith)
 			return false;

 		// If our launcher is inside something, and that something is 'thingWeCollidedWith' we won't collide
 		const Object *launcherContainedBy = projectileLauncher->getContainedBy();
 		if( launcherContainedBy == thingWeCollidedWith )
 			return false;

 	}

	// never bother burning already-burned things. (srj)
	if (getDamageType() == DAMAGE_FLAME || getDamageType() == DAMAGE_PARTICLE_BEAM)
	{
		if (thingWeCollidedWith->testStatus(OBJECT_STATUS_BURNED))
		{
			return false;
		}
	}

	// horrible special case for airplanes sitting on airfields: the projectile might
	// "collide" with the airfield's (invisible) collision geometry when a resting plane
	// is targeted. we don't want this. special case it:
	if (thingWeCollidedWith->isKindOf(KINDOF_FS_AIRFIELD))
	{
		//
		// ok, so if we are an airfield, and our intended victim has a reserved space
		// with us, it "belongs" to us and collisions intended for it should not detonate
		// as a result of colliding with us.
		//
		// notes:
		//	-- we have already established that "thingWeCollidedWith" is not the intended victim (above)
		//	-- this does NOT verify that the plane is actually parked at the airfield; it might be elsewhere
		//			(but if it is, it's highly unlikely that this sort of collision could occur)
		//
		for (BehaviorModule** i = thingWeCollidedWith->getBehaviorModules(); *i; ++i)
		{
			ParkingPlaceBehaviorInterface* pp = (*i)->getParkingPlaceBehaviorInterface();
			if (pp != nullptr && pp->hasReservedSpace(intendedVictimID))
				return false;
		}
	}

	// if something has a Sneaky Target offset, it is momentarily immune to being hit...
	// normally this shouldn't happen, but occasionally can by accident. so avoid it. (srj)
	const AIUpdateInterface* ai = thingWeCollidedWith->getAI();
	if (ai != nullptr && ai->getSneakyTargetingOffset(nullptr))
	{
		return false;
	}

	Int requiredMask = 0;

	Relationship r = projectile->getRelationship(thingWeCollidedWith);
	if (r == ALLIES) requiredMask = WEAPON_COLLIDE_ALLIES;
	else if (r == ENEMIES) requiredMask = WEAPON_COLLIDE_ENEMIES;

	if (thingWeCollidedWith->isKindOf(KINDOF_STRUCTURE))
	{
		if (thingWeCollidedWith->getControllingPlayer() == projectile->getControllingPlayer())
			requiredMask |= WEAPON_COLLIDE_CONTROLLED_STRUCTURES;
		else
			requiredMask |= WEAPON_COLLIDE_STRUCTURES;
	}
	if (thingWeCollidedWith->isKindOf(KINDOF_SHRUBBERY))					requiredMask |= WEAPON_COLLIDE_SHRUBBERY;
	if (thingWeCollidedWith->isKindOf(KINDOF_PROJECTILE))					requiredMask |= WEAPON_COLLIDE_PROJECTILE;
	if (thingWeCollidedWith->getTemplate()->getFenceWidth() > 0)	requiredMask |= WEAPON_COLLIDE_WALLS;
	if (thingWeCollidedWith->isKindOf(KINDOF_SMALL_MISSILE))			requiredMask |= WEAPON_COLLIDE_SMALL_MISSILES;			//All missiles are also projectiles!
	if (thingWeCollidedWith->isKindOf(KINDOF_BALLISTIC_MISSILE))	requiredMask |= WEAPON_COLLIDE_BALLISTIC_MISSILES;	//All missiles are also projectiles!

	// if any in requiredMask are present in collidemask, do the collision. (srj)
	if ((getProjectileCollideMask() & requiredMask) != 0)
		return true;

	//DEBUG_LOG(("Rejecting projectile collision between %s and %s!",projectile->getTemplate()->getName().str(),thingWeCollidedWith->getTemplate()->getName().str()));
	return false;
}

//-------------------------------------------------------------------------------------------------
UnsignedInt WeaponTemplate::fireWeaponTemplate
(
	const Object *sourceObj,
	WeaponSlotType wslot,
	Int specificBarrelToUse,
	Object *victimObj,
	const Coord3D* victimPos,
	const WeaponBonus& bonus,
	Bool isProjectileDetonation,
	Bool ignoreRanges,
	Weapon *firingWeapon,
	ObjectID* projectileID,
	Bool inflictDamage
) const
{

	//-extraLogging
	#if defined(RTS_DEBUG)
		AsciiString targetStr;
		if( TheGlobalData->m_extraLogging )
		{
			if( victimObj )
				targetStr.format( "%s", victimObj->getTemplate()->getName().str() );
			else if( victimPos )
				targetStr.format( "%d,%d,%d", victimPos->x, victimPos->y, victimPos->z );
			else
				targetStr.format( "SELF." );

			DEBUG_LOG( ("%d - WeaponTemplate::fireWeaponTemplate() begin - %s attacking %s",
				TheGameLogic->getFrame(), sourceObj->getTemplate()->getName().str(), targetStr.str() ) );
		}
	#endif
	//end -extraLogging

	//CRCDEBUG_LOG(("WeaponTemplate::fireWeaponTemplate() from %s", DescribeObject(sourceObj).str()));
	DEBUG_ASSERTCRASH(specificBarrelToUse >= 0, ("specificBarrelToUse should no longer be -1"));

	if (sourceObj == nullptr || (victimObj == nullptr && victimPos == nullptr))
	{
		//-extraLogging
		#if defined(RTS_DEBUG)
			if( TheGlobalData->m_extraLogging )
				DEBUG_LOG( ("FAIL 1 (sourceObj %d == nullptr || (victimObj %d == nullptr && victimPos %d == nullptr)", sourceObj != 0, victimObj != 0, victimPos != 0) );
		#endif
		//end -extraLogging

		return 0;
	}

	DEBUG_ASSERTCRASH((m_primaryDamage > 0)  ||  (victimObj == nullptr), ("You can't really shoot a zero damage weapon at an Object.") );

	ObjectID sourceID = sourceObj->getID();
	const Coord3D* sourcePos = sourceObj->getPosition();

	Real distSqr;
	ObjectID victimID;
	TBridgeAttackInfo info;
	Coord3D victimPosStorage;
	if (victimObj)
	{
		DEBUG_ASSERTLOG(sourceObj != victimObj, ("*** firing weapon at self -- is this really what you want?"));
		victimPos = victimObj->getPosition();
		victimID = victimObj->getID();

		Coord3D sneakyOffset;
		const AIUpdateInterface* ai = victimObj->getAI();
		if (ai != nullptr && ai->getSneakyTargetingOffset(&sneakyOffset))
		{
			victimPosStorage = *victimPos;
			victimPosStorage.x += sneakyOffset.x;
			victimPosStorage.y += sneakyOffset.y;
			victimPosStorage.z += sneakyOffset.z;

			victimPos = &victimPosStorage;
			// for a sneaky offset, we always target a position rather than an object
			victimObj = nullptr;
			victimID = INVALID_ID;
			distSqr = ThePartitionManager->getDistanceSquared(sourceObj, victimPos, ATTACK_RANGE_CALC_TYPE);
		}
		else
		{
			if (victimObj->isKindOf(KINDOF_BRIDGE))
			{
				// Bridges are kind of oddball - they have 2 target points at either end.
				TheTerrainLogic->getBridgeAttackPoints(victimObj, &info);
				distSqr = ThePartitionManager->getDistanceSquared( sourceObj, &info.attackPoint1, ATTACK_RANGE_CALC_TYPE );
				victimPos = &info.attackPoint1;
 				Real distSqr2 = ThePartitionManager->getDistanceSquared( sourceObj, &info.attackPoint2, ATTACK_RANGE_CALC_TYPE );
				if (distSqr > distSqr2)
				{
					// Try the other one.
					distSqr = distSqr2;
					victimPos = &info.attackPoint2;
				}
			}
			else
			{
				distSqr = ThePartitionManager->getDistanceSquared(sourceObj, victimObj, ATTACK_RANGE_CALC_TYPE);
			}
		}
	}
	else
	{
		victimID = INVALID_ID;
		distSqr = ThePartitionManager->getDistanceSquared(sourceObj, victimPos, ATTACK_RANGE_CALC_TYPE);
	}

//	DEBUG_LOG(("WeaponTemplate::fireWeaponTemplate: firing weapon %s (source=%s, victim=%s)",
//		m_name.str(),sourceObj->getTemplate()->getName().str(),victimObj?victimObj->getTemplate()->getName().str():"null"));

	//Only perform this check if the weapon isn't a leech range weapon (which can have unlimited range!)
	if( !ignoreRanges && !isLeechRangeWeapon() )
	{
		Real attackRangeSqr = sqr(getAttackRange(bonus));
		if (distSqr > attackRangeSqr)
		{
			//DEBUG_ASSERTCRASH(distSqr < 5*5 || distSqr < attackRangeSqr*1.2f, ("*** victim is out of range (%f vs %f) of this weapon -- why did we attempt to fire?",WWMath::Sqrtf(distSqr),WWMath::Sqrtf(attackRangeSqr)));

			//-extraLogging
			#if defined(RTS_DEBUG)
				if( TheGlobalData->m_extraLogging )
					DEBUG_LOG( ("FAIL 2 (distSqr %.2f > attackRangeSqr %.2f)", distSqr, attackRangeSqr ) );
			#endif
			//end -extraLogging

			return 0;
		}
	}

	if (!ignoreRanges)
	{
		Real minAttackRangeSqr = sqr(getMinimumAttackRange());
#ifdef RATIONALIZE_ATTACK_RANGE
		if (distSqr < minAttackRangeSqr && !isProjectileDetonation)
#else
		if (distSqr < minAttackRangeSqr-0.5f && !isProjectileDetonation)
#endif
		{
			DEBUG_ASSERTCRASH(distSqr > minAttackRangeSqr*0.8f, ("*** victim is closer than min attack range (%f vs %f) of this weapon -- why did we attempt to fire?",WWMath::Sqrtf(distSqr),WWMath::Sqrtf(minAttackRangeSqr)));

			//-extraLogging
			#if defined(RTS_DEBUG)
				if( TheGlobalData->m_extraLogging )
					DEBUG_LOG( ("FAIL 3 (distSqr %.2f< minAttackRangeSqr %.2f - 0.5f && !isProjectileDetonation %d)", distSqr, minAttackRangeSqr, isProjectileDetonation ) );
			#endif
			//end -extraLogging

			return 0;
		}
	}

	// call this even if FXList is null, because this also handles stuff like Gun Barrel Recoil
	if (sourceObj && sourceObj->getDrawable())
	{
		Coord3D targetPos;
		if( victimObj )
		{
			victimObj->getGeometryInfo().getCenterPosition( *victimObj->getPosition(), targetPos );
		}
		else
		{
			targetPos.set( *victimPos );
		}
		Real reAngle = getWeaponRecoilAmount();
		Real reDir = reAngle != 0.0f ? (WWMath::Atan2(victimPos->y - sourcePos->y, victimPos->x - sourcePos->x)) : 0.0f;
		VeterancyLevel v = sourceObj->getVeterancyLevel();
		const FXList* fx = isProjectileDetonation ? getProjectileDetonateFX(v) : getFireFX(v);

		if ( TheGameLogic->getFrame() < firingWeapon->getSuspendFXFrame() )
			fx = nullptr;

		Bool handled;

		// TheSuperHackers @todo: Remove hardcoded KINDOF_MINE check and apply PlayFXWhenStealthed = Yes to the mine weapons instead.

		if (!sourceObj->isLogicallyVisible() &&									// if user watching cannot see us
			!sourceObj->isKindOf(KINDOF_MINE) &&								// and not a mine (which always do the FX, even if hidden)...
			!isPlayFXWhenStealthed()													// and not a weapon marked to playwhenstealthed
			)
		{
			handled = TRUE;		// then let's just pretend like we did the fx by returning true
		}
		else
		{
			handled = sourceObj->getDrawable()->handleWeaponFireFX(wslot,
																															specificBarrelToUse,
																															fx,
																															getWeaponSpeed(),
																															reAngle,
																															reDir,
																															&targetPos,
																															getPrimaryDamageRadius(bonus)
																															);
		}

		if (handled == false && fx != nullptr)
		{
			// bah. just play it at the drawable's pos.
			//DEBUG_LOG(("*** WeaponFireFX not fully handled by the client"));
			const Coord3D* where = isContactWeapon() ? &targetPos : sourceObj->getDrawable()->getPosition();
			FXList::doFXPos(fx, where, sourceObj->getDrawable()->getTransformMatrix(), getWeaponSpeed(), &targetPos, getPrimaryDamageRadius(bonus));
		}
	}

	// Now do the FireOCL if there is one
	if( sourceObj )
	{
		VeterancyLevel v = sourceObj->getVeterancyLevel();
		const ObjectCreationList *oclToUse = isProjectileDetonation ? getProjectileDetonationOCL(v) : getFireOCL(v);
		if( oclToUse )
			ObjectCreationList::create( oclToUse, sourceObj, nullptr );
	}

	Coord3D projectileDestination = *victimPos; //Need to copy this, as we have a pointer to their actual position
	Real scatterRadius = 0.0f;
	if( m_scatterRadius > 0.0f || (m_infantryInaccuracyDist > 0.0f && victimObj && victimObj->isKindOf( KINDOF_INFANTRY )) )
	{
		// This weapon scatters, so clear the victimObj, as we are no longer shooting it directly,
		// and find a random point within the radius to shoot at as victimPos
		scatterRadius = m_scatterRadius;

		// if it's an object, aim at the center, not the ground part (srj)
		PathfindLayerEnum targetLayer = LAYER_GROUND;
		if( victimObj )
		{
			if( victimObj->isKindOf( KINDOF_STRUCTURE ) )
			{
				victimObj->getGeometryInfo().getCenterPosition(*victimObj->getPosition(), projectileDestination);
			}
			if( m_infantryInaccuracyDist > 0.0f && victimObj->isKindOf( KINDOF_INFANTRY ) )
			{
				//If we are firing a weapon that is considered inaccurate against infantry, then add it to
				//the scatter radius!
				scatterRadius += m_infantryInaccuracyDist;
			}
			targetLayer = victimObj->getLayer();
		}

		//victimObj = nullptr; // his position is already in victimPos, if he existed

		//Randomize the scatter radius (sometimes it can be more accurate than others)
		scatterRadius = GameLogicRandomValueReal( 0, scatterRadius );
		Real scatterAngleRadian = GameLogicRandomValueReal( 0, 2*PI );

		Coord3D firingOffset;
		firingOffset.zero();
		firingOffset.x = scatterRadius * Cos( scatterAngleRadian );
		firingOffset.y = scatterRadius * Sin( scatterAngleRadian );

		projectileDestination.x += firingOffset.x;
		projectileDestination.y += firingOffset.y;

		//What's suddenly become crucially important is to FIRE at the ground at this location!!!
		//If we aim for the center point of our target and miss, the shot will go much farther than
		//we expect!
		// srj sez: we should actually fire at the layer the victim is on, if possible, in case it is on a bridge...
		projectileDestination.z = TheTerrainLogic->getLayerHeight( projectileDestination.x, projectileDestination.y, targetLayer );
	}

	if (getProjectileTemplate() == nullptr || isProjectileDetonation)
	{
		// see if we need to be called back at a later point to deal the damage.
		Coord3D v;
		v.x = victimPos->x - sourcePos->x;
		v.y = victimPos->y - sourcePos->y;
		v.z = victimPos->z - sourcePos->z;
		// don't round the result; we WANT a fractional-frame-delay in this case.
		Real delayInFrames = WWMath::Div_Safe(v.length(), getWeaponSpeed());

		ObjectID damageID = getDamageDealtAtSelfPosition() ? INVALID_ID : victimID;

		if( firingWeapon->isLaser() )
		{
			if( scatterRadius <= getPrimaryDamageRadius( bonus ) || scatterRadius <= getSecondaryDamageRadius( bonus ) )
			{
				//The laser is close enough to damage the object, so just hit it directly. Some victim objects
				//adjust the laser's position to prevent it from hitting the ground.
				if( victimObj )
				{
					projectileDestination.set( *victimObj->getPosition() );
				}
				firingWeapon->createLaser( sourceObj, victimObj, &projectileDestination );
			}
			else
			{
				//We are missing our intended target, so now we want to aim at the ground at the projectile offset.
				damageID = INVALID_ID;
				firingWeapon->createLaser( sourceObj, nullptr, &projectileDestination );
			}
			if( inflictDamage )
			{
				dealDamageInternal( sourceID, damageID, &projectileDestination, bonus, isProjectileDetonation );
			}
			return TheGameLogic->getFrame();
		}

		const Coord3D* damagePos = getDamageDealtAtSelfPosition() ? sourcePos : victimPos;
		if (delayInFrames < 1.0f)
		{
			// go ahead and do it now
			//DEBUG_LOG(("WeaponTemplate::fireWeaponTemplate: firing weapon immediately!"));
			if( inflictDamage )
			{
				dealDamageInternal(sourceID, damageID, damagePos, bonus, isProjectileDetonation);
			}

			//-extraLogging
			#if defined(RTS_DEBUG)
				if( TheGlobalData->m_extraLogging )
					DEBUG_LOG( ("EARLY 4 (delayed damage applied now)") );
			#endif
			//end -extraLogging


			return TheGameLogic->getFrame();
		}
		else
		{
			UnsignedInt when = 0;
			if( TheWeaponStore && inflictDamage )
			{
				UnsignedInt delayInWholeFrames = REAL_TO_INT_CEIL(delayInFrames);
				when = TheGameLogic->getFrame() + delayInWholeFrames;
				//DEBUG_LOG(("WeaponTemplate::fireWeaponTemplate: firing weapon in %d frames (= %d)!", delayInWholeFrames,when));
				TheWeaponStore->setDelayedDamage(this, damagePos, when, sourceID, damageID, bonus);
			}

			//-extraLogging
			#if defined(RTS_DEBUG)
				if( TheGlobalData->m_extraLogging )
					DEBUG_LOG( ("EARLY 5 (delaying damage applied until frame %d)", when ) );
			#endif
			//end -extraLogging


			return when;
		}
	}
	else	// must be a projectile
	{
		Player *owningPlayer = sourceObj->getControllingPlayer(); //Need to know so missiles don't collide with firer
		Object *projectile = TheThingFactory->newObject( getProjectileTemplate(), owningPlayer->getDefaultTeam() );
		projectile->setProducer(sourceObj);

		//If the player has battle plans (America Strategy Center), then apply those bonuses
		//to this object if applicable. Internally it validates certain kinds of objects.
		//When projectiles are created, weapon bonuses such as damage may get applied.
		if( owningPlayer->getNumBattlePlansActive() > 0 )
		{
			owningPlayer->applyBattlePlanBonusesForObject( projectile );
		}


		//Store the project ID in the object as the last projectile fired!
		if (projectileID)
			*projectileID = projectile->getID();

		// Notify special power tracking
		SpecialPowerCompletionDie *die = sourceObj->findSpecialPowerCompletionDie();
		if (die)
		{
			die->notifyScriptEngine();
			die = projectile->findSpecialPowerCompletionDie();
			if (die)
			{
				die->setCreator(INVALID_ID);
			}
		}
		else
		{
			die = projectile->findSpecialPowerCompletionDie();
			if (die)
			{
				die->setCreator(sourceObj->getID());
			}
		}

		firingWeapon->newProjectileFired( sourceObj, projectile, victimObj, victimPos );//The actual logic weapon needs to know this was created.

		ProjectileUpdateInterface* pui = nullptr;
		for (BehaviorModule** u = projectile->getBehaviorModules(); *u; ++u)
		{
			if ((pui = (*u)->getProjectileUpdateInterface()) != nullptr)
				break;
		}
		if (pui)
		{
			VeterancyLevel v = sourceObj->getVeterancyLevel();
			if( scatterRadius > 0.0f )
			{
				//With a scatter radius, don't follow the victim (overriding the intent).
				pui->projectileLaunchAtObjectOrPosition( nullptr, &projectileDestination, sourceObj, wslot, specificBarrelToUse, this, m_projectileExhausts[v] );
			}
			else
			{
				pui->projectileLaunchAtObjectOrPosition(victimObj, &projectileDestination, sourceObj, wslot, specificBarrelToUse, this, m_projectileExhausts[v]);
			}
		}
		else
		{
			//DEBUG_CRASH(("Projectiles should implement ProjectileUpdateInterface!"));
			// actually, this is ok, for things like Firestorm.... (srj)
			projectile->setPosition(&projectileDestination);
		}

		//If we're launching a missile at a unit with valid countermeasures, then communicate it
		if( projectile->isKindOf( KINDOF_SMALL_MISSILE ) && victimObj && victimObj->hasCountermeasures() )
		{
			const AIUpdateInterface *ai = victimObj->getAI();
			//Only allow jets not currently supersonic to launch countermeasures
			if( ai && ai->getCurLocomotorSetType() != LOCOMOTORSET_SUPERSONIC )
			{
				//This function will determine now whether or not the fired projectile will be diverted to
				//an available decoy flare.
				victimObj->reportMissileForCountermeasures( projectile );
			}

		}
		//-extraLogging
		#if defined(RTS_DEBUG)
			if( TheGlobalData->m_extraLogging )
				DEBUG_LOG( ("DONE") );
		#endif
		//end -extraLogging

		return 0;
	}
}

//-------------------------------------------------------------------------------------------------
#if RETAIL_COMPATIBLE_CRC || PRESERVE_UNRELIABLE_FIRESTORMS
void WeaponTemplate::trimOldHistoricDamage() const
{
	UnsignedInt expirationDate = TheGameLogic->getFrame() - TheGlobalData->m_historicDamageLimit;
	while (!m_historicDamage.empty())
	{
		HistoricWeaponDamageInfo& h = m_historicDamage.front();
		if (h.frame <= expirationDate)
		{
			m_historicDamage.pop_front();
			continue;
		}
		else
		{
			// since they are in strict chronological order,
			// stop as soon as we get to a nonexpired one
			break;
		}
	}
}
#else
void WeaponTemplate::trimOldHistoricDamage() const
{
	if (m_historicDamage.empty())
		return;

	const UnsignedInt currentFrame = TheGameLogic->getFrame();
	const UnsignedInt expirationFrame = currentFrame - m_historicBonusTime;

	HistoricWeaponDamageList::iterator it = m_historicDamage.begin();

	while (it != m_historicDamage.end())
	{
		if (it->frame <= expirationFrame)
			it = m_historicDamage.erase(it);
		else
			break;
	}
}
#endif

//-------------------------------------------------------------------------------------------------
void WeaponTemplate::trimTriggeredHistoricDamage() const
{
	HistoricWeaponDamageList::iterator it = m_historicDamage.begin();

	while (it != m_historicDamage.end())
	{
		if (it->triggerId == m_historicDamageTriggerId)
			it = m_historicDamage.erase(it);
		else
			++it;
	}
}

//-------------------------------------------------------------------------------------------------
static Bool is2DDistSquaredLessThan(const Coord3D& a, const Coord3D& b, Real distSqr)
{
	Real da = sqr(a.x - b.x) + sqr(a.y - b.y);
	return da <= distSqr;
}

//-------------------------------------------------------------------------------------------------
#if RETAIL_COMPATIBLE_CRC || PRESERVE_UNRELIABLE_FIRESTORMS
void WeaponTemplate::processHistoricDamage(const Object* source, const Coord3D* pos) const
{
	//
	/** @todo We need to rewrite the historic stuff ... if you fire 5 missiles, and the 5th,
	// one creates a firestorm ... and then half a second later another volley of 5 missiles
	// come in, the second wave of 5 missiles would all do a historic weapon, making 5 more
	// firestorms (CBD) */
	//

	if( m_historicBonusCount > 0 && m_historicBonusWeapon != this )
	{
		trimOldHistoricDamage();

		Real radSqr = m_historicBonusRadius * m_historicBonusRadius;
		Int count = 0;
		UnsignedInt frameNow = TheGameLogic->getFrame();
		UnsignedInt oldestThatWillCount = frameNow - m_historicBonusTime; // Anything before this frame is "more than two seconds ago" eg
		for( HistoricWeaponDamageList::const_iterator it = m_historicDamage.begin(); it != m_historicDamage.end(); ++it )
		{
			if( it->frame >= oldestThatWillCount &&
					is2DDistSquaredLessThan( *pos, it->location, radSqr ) )
			{
				// This one is close enough in time and distance, so count it. This is tracked by template since it applies
				// across units, so don't try to clear historicDamage on success in here.
				++count;
			}
		}

		if( count >= m_historicBonusCount - 1 )	// minus 1 since we include ourselves implicitly
		{
		  TheWeaponStore->createAndFireTempWeapon(m_historicBonusWeapon, source, pos);

			/** @todo E3 hack! Clear the list for now to make sure we don't have multiple firestorms
				* remove this when the branches merge back into one.  What is causing the
				* multiple firestorms, who is to say ... this is a plug, not a fix! */
			m_historicDamage.clear();

		}
		else
		{

			// add AFTER checking for historic stuff
			m_historicDamage.push_back( HistoricWeaponDamageInfo(frameNow, *pos) );

		}

	}
}
#else
void WeaponTemplate::processHistoricDamage(const Object* source, const Coord3D* pos) const
{
	if (m_historicBonusCount > 0 && m_historicBonusWeapon != this)
	{
		trimOldHistoricDamage();

		++m_historicDamageTriggerId;

		const Int requiredCount = m_historicBonusCount - 1; // minus 1 since we include ourselves implicitly
		if (m_historicDamage.size() >= requiredCount)
		{
			const Real radSqr = m_historicBonusRadius * m_historicBonusRadius;
			Int count = 0;

			for (HistoricWeaponDamageList::iterator it = m_historicDamage.begin(); it != m_historicDamage.end(); ++it)
			{
				if (is2DDistSquaredLessThan(*pos, it->location, radSqr))
				{
					// This one is close enough in time and distance, so count it. This is tracked by template since it applies
					// across units, so don't try to clear historicDamage on success in here.
					it->triggerId = m_historicDamageTriggerId;

					if (++count == requiredCount)
					{
						TheWeaponStore->createAndFireTempWeapon(m_historicBonusWeapon, source, pos);
						trimTriggeredHistoricDamage();
						return;
					}
				}
			}
		}

		// add AFTER checking for historic stuff
		m_historicDamage.push_back(HistoricWeaponDamageInfo(TheGameLogic->getFrame(), *pos));
	}
}
#endif

//-------------------------------------------------------------------------------------------------
void WeaponTemplate::dealDamageInternal(ObjectID sourceID, ObjectID victimID, const Coord3D *pos, const WeaponBonus& bonus, Bool isProjectileDetonation) const
{
	if (sourceID == 0)	// must have a source
		return;

	if (victimID == 0 && pos == nullptr)	// must have some sort of destination
		return;

	Object *source = TheGameLogic->findObjectByID(sourceID);	// might be null...

	processHistoricDamage(source, pos);

//DEBUG_LOG(("WeaponTemplate::dealDamageInternal: dealing damage %s at frame %d",m_name.str(),TheGameLogic->getFrame()));

	// if there's a specific victim, use it's pos (overriding the value passed in)
	Object *primaryVictim = victimID ? TheGameLogic->findObjectByID(victimID) : nullptr;	// might be null...
	if (primaryVictim)
	{
		pos = primaryVictim->getPosition();
	}

	DamageType damageType = getDamageType();
	DeathType deathType = getDeathType();
	ObjectStatusTypes damageStatusType = getDamageStatusType();
	if (getProjectileTemplate() == nullptr || isProjectileDetonation)
	{
		SimpleObjectIterator *iter;
		Object *curVictim;
		Real curVictimDistSqr;

		Real primaryRadius = getPrimaryDamageRadius(bonus);
		Real secondaryRadius = getSecondaryDamageRadius(bonus);
		Real primaryDamage = getPrimaryDamage(bonus);
		Real secondaryDamage = getSecondaryDamage(bonus);
		Int affects = getAffectsMask();

		DEBUG_ASSERTCRASH(secondaryRadius >= primaryRadius || secondaryRadius == 0.0f, ("secondary radius should be >= primary radius (or zero)"));

		Real primaryRadiusSqr = sqr(primaryRadius);
		Real radius = max(primaryRadius, secondaryRadius);
		if (radius > 0.0f)
		{
			iter = ThePartitionManager->iterateObjectsInRange(pos, radius, DAMAGE_RANGE_CALC_TYPE);
			curVictim = iter->firstWithNumeric(&curVictimDistSqr);
		}
		else
		{
			//DEBUG_ASSERTCRASH(primaryVictim != nullptr, ("weapons without radii should always pass in specific victims"));
			// check against victimID rather than primaryVictim, since we may have targeted a legitimate victim
			// that got killed before the damage was dealt... (srj)
			//DEBUG_ASSERTCRASH(victimID != 0, ("weapons without radii should always pass in specific victims"));
			iter = nullptr;
			curVictim = primaryVictim;
			curVictimDistSqr = 0.0f;

			if( affects & WEAPON_KILLS_SELF )
			{
				DamageInfo damageInfo;
				damageInfo.in.m_damageType = damageType;
				damageInfo.in.m_deathType = deathType;
				damageInfo.in.m_sourceID = sourceID;
				damageInfo.in.m_sourcePlayerMask = 0;
				damageInfo.in.m_damageStatusType = damageStatusType;
				damageInfo.in.m_amount = HUGE_DAMAGE_AMOUNT;
				source->attemptDamage( &damageInfo );
				return;
			}
		}
		MemoryPoolObjectHolder hold(iter);

		for (; curVictim != nullptr; curVictim = iter ? iter->nextWithNumeric(&curVictimDistSqr) : nullptr)
		{
			Bool killSelf = false;
			if (source != nullptr)
			{
				// anytime something is designated as the "primary victim" (ie, the direct target
				// of the weapon), we ignore all the "affects" flags.
				if (curVictim != primaryVictim)
				{

					if( (affects & WEAPON_KILLS_SELF) && source == curVictim )
					{
						killSelf = true;
					}
					else
					{

						// should object ever be allowed to damage themselves? methinks not...
						// exception: a few weapons allow this (eg, for suicide bombers).
						if( (affects & WEAPON_AFFECTS_SELF) == 0 )
						{
							// Remember that source is a missile for some units, and they don't want to injure them'selves' either
							if( source == curVictim || source->getProducerID() == curVictim->getID() )
							{
								//DEBUG_LOG(("skipping damage done to SELF..."));
								continue;
							}
						}

						if( affects & WEAPON_DOESNT_AFFECT_SIMILAR )
						{
							//This means we probably are affecting allies, but don't want to kill nearby members that are the same type as us.
							//A good example are a group of terrorists blowing themselves up. We don't want to cause a domino effect that kills
							//all of them.
							if( source->getTemplate()->isEquivalentTo(curVictim->getTemplate()) && source->getRelationship( curVictim ) == ALLIES )
							{
								continue;
							}
						}

						if ((affects & WEAPON_DOESNT_AFFECT_AIRBORNE) != 0 && curVictim->isSignificantlyAboveTerrain())
						{
							continue;
						}

						/*
							The idea here is: if its our ally(/enemies), AND it's not the direct target, AND the weapon doesn't
							do radius-damage to allies(/enemies)... skip it.
						*/
						Relationship r = curVictim->getRelationship(source);
						Int requiredMask;
						if (r == ALLIES)
							requiredMask = WEAPON_AFFECTS_ALLIES;
						else if (r == ENEMIES)
							requiredMask = WEAPON_AFFECTS_ENEMIES;
						else /* r == NEUTRAL */
							requiredMask = WEAPON_AFFECTS_NEUTRALS;

						if( !(affects & requiredMask) )
						{
							//Skip if we aren't affected by this weapon.
							continue;
						}
					}
				}
			}

			DamageInfo damageInfo;
			damageInfo.in.m_damageType = damageType;
			damageInfo.in.m_deathType = deathType;
			damageInfo.in.m_sourceID = sourceID;
			damageInfo.in.m_sourcePlayerMask = 0;
			damageInfo.in.m_damageStatusType = damageStatusType;

			Coord3D damageDirection;
			damageDirection.zero();
			if( curVictim && source )
			{
				damageDirection.set( *curVictim->getPosition() );
				damageDirection.sub( *source->getPosition() );
			}

			Real allowedAngle = getRadiusDamageAngle();
			if( allowedAngle < PI )
			{
				if( curVictim == nullptr  ||  source == nullptr )
					continue; // We are directional damage, but can't figure out our direction.  Just bail.

				// People can only be hit in a cone oriented as the firer is oriented
				Vector3 sourceVector = source->getTransformMatrix()->Get_X_Vector();
				Vector3 damageVector(damageDirection.x, damageDirection.y, damageDirection.z);
				sourceVector.Normalize();
				damageVector.Normalize();

				// These are now normalized, so the dot productis actually the Cos of the angle they form
				// A smaller Cos would mean a more obtuse angle
				if( Vector3::Dot_Product(sourceVector, damageVector) < Cos(allowedAngle) )
					continue;// Too far to the side, can't hurt them.
			}

			// Grab the vector between the source object causing the damage and the victim in order that we can
			// simulate a shockwave pushing objects around
			damageInfo.in.m_shockWaveAmount = m_shockWaveAmount;
			if (damageInfo.in.m_shockWaveAmount > 0.0f)
			{
				// Calculate the vector of the shockwave
				Coord3D shockWaveVector = damageDirection;

				// Guard against zero vector. Make vector straight up if that is the case
				if (WWMath::Fabs(shockWaveVector.x) < WWMATH_EPSILON &&
						WWMath::Fabs(shockWaveVector.y) < WWMATH_EPSILON &&
						WWMath::Fabs(shockWaveVector.z) < WWMATH_EPSILON)
				{
					shockWaveVector.z = 1.0f;
				}

				// Populate the damage information with the shockwave information
				damageInfo.in.m_shockWaveVector = shockWaveVector;
				damageInfo.in.m_shockWaveRadius = m_shockWaveRadius;
				damageInfo.in.m_shockWaveTaperOff = m_shockWaveTaperOff;
			}

      if (source && source->getControllingPlayer()) {
				damageInfo.in.m_sourcePlayerMask = source->getControllingPlayer()->getPlayerMask();
			}
			// note, don't bother with damage multipliers here...
			// that's handled internally by the attemptDamage() method.
			damageInfo.in.m_amount = (curVictimDistSqr <= primaryRadiusSqr) ? primaryDamage : secondaryDamage;

			if( killSelf )
			{
				//Deal enough damage to kill yourself. I thought about getting the current health and applying
				//enough unresistable damage to die... however it's possible that we have different types of
				//deaths based on damage type and/or the possibility to resist certain damage types and
				//surviving -- so instead, I'm blindly inflicting a very high value of the intended damage type.
				damageInfo.in.m_amount = HUGE_DAMAGE_AMOUNT;
				//BodyModuleInterface* body = curVictim->getBodyModule();
				//if( body )
				//{
				//	Real curVictimHealth = curVictim->getBodyModule()->getHealth();
				//	damageInfo.in.m_amount = __max( damageInfo.in.m_amount, curVictimHealth );
				//}
			}

			// if the damage-dealer is a projectile, designate the damage as done by its launcher, not the projectile.
			// this is much more useful for the AI...
			if (source && source->isKindOf(KINDOF_PROJECTILE))
			{
				for (BehaviorModule** u = source->getBehaviorModules(); *u; ++u)
				{
					ProjectileUpdateInterface* pui = (*u)->getProjectileUpdateInterface();
					if (pui != nullptr)
					{
						damageInfo.in.m_sourceID = pui->projectileGetLauncherID();
						break;
					}
				}
			}

			curVictim->attemptDamage(&damageInfo);
			//DEBUG_ASSERTLOG(damageInfo.out.m_noEffect, ("WeaponTemplate::dealDamageInternal: dealt to %s %08lx: attempted %f, actual %f (%f)",
			//	curVictim->getTemplate()->getName().str(),curVictim,
			//	damageInfo.in.m_amount, damageInfo.out.m_actualDamageDealt, damageInfo.out.m_actualDamageClipped));
		}
	}
	else
	{
		DEBUG_CRASH(("projectile weapons should never get dealDamage called directly"));
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
WeaponStore::WeaponStore()
{
}

//-------------------------------------------------------------------------------------------------
WeaponStore::~WeaponStore()
{
	deleteAllDelayedDamage();

	for (size_t i = 0; i < m_weaponTemplateVector.size(); i++)
	{
		WeaponTemplate* wt = m_weaponTemplateVector[i];
		deleteInstance(wt);
	}
	m_weaponTemplateVector.clear();
	m_weaponTemplateHashMap.clear();
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::handleProjectileDetonation(const WeaponTemplate* wt, const Object *source, const Coord3D* pos, WeaponBonusConditionFlags extraBonusFlags, Bool inflictDamage )
{
	Weapon* w = allocateNewWeapon(wt, PRIMARY_WEAPON);
	w->loadAmmoNow(source);
	w->fireProjectileDetonationWeapon( source, pos, extraBonusFlags, inflictDamage );
	deleteInstance(w);
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::createAndFireTempWeapon(const WeaponTemplate* wt, const Object *source, const Coord3D* pos)
{
	if (wt == nullptr)
		return;
	Weapon* w = allocateNewWeapon(wt, PRIMARY_WEAPON);
	w->loadAmmoNow(source);
	w->fireWeapon(source, pos);
	deleteInstance(w);
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::createAndFireTempWeapon(const WeaponTemplate* wt, const Object *source, Object *target)
{
	//CRCDEBUG_LOG(("WeaponStore::createAndFireTempWeapon() for %s", DescribeObject(source)));
	if (wt == nullptr)
		return;
	Weapon* w = allocateNewWeapon(wt, PRIMARY_WEAPON);
	w->loadAmmoNow(source);
	w->fireWeapon(source, target);
	deleteInstance(w);
}

//-------------------------------------------------------------------------------------------------
const WeaponTemplate *WeaponStore::findWeaponTemplate( const AsciiString& name ) const
{
	if (name.compareNoCase("None") == 0)
		return nullptr;
	const WeaponTemplate * wt = findWeaponTemplatePrivate( TheNameKeyGenerator->nameToKey( name ) );
	DEBUG_ASSERTCRASH(wt != nullptr, ("Weapon %s not found!", name.str()));
	return wt;
}

//-------------------------------------------------------------------------------------------------
const WeaponTemplate *WeaponStore::findWeaponTemplate( const char* name ) const
{
	if (stricmp(name, "None") == 0)
		return nullptr;
	const WeaponTemplate * wt = findWeaponTemplatePrivate( TheNameKeyGenerator->nameToKey( name ) );
	DEBUG_ASSERTCRASH(wt != nullptr, ("Weapon %s not found!",name));
	return wt;
}

//-------------------------------------------------------------------------------------------------
WeaponTemplate *WeaponStore::findWeaponTemplatePrivate( NameKeyType key ) const
{
	// search weapon list for name
	WeaponTemplateMap::const_iterator it = m_weaponTemplateHashMap.find(key);
	if(it != m_weaponTemplateHashMap.end())
		return it->second;

	return nullptr;

}

//-------------------------------------------------------------------------------------------------
WeaponTemplate *WeaponStore::newWeaponTemplate(AsciiString name)
{

	// sanity
	if(name.isEmpty())
		return nullptr;

	// allocate a new weapon
	WeaponTemplate *wt = newInstance(WeaponTemplate);
	wt->m_name = name;
	wt->m_nameKey = TheNameKeyGenerator->nameToKey( name );
	m_weaponTemplateVector.push_back(wt);
	m_weaponTemplateHashMap[wt->m_nameKey] = wt;

	return wt;
}

//-------------------------------------------------------------------------------------------------
WeaponTemplate *WeaponStore::newOverride(WeaponTemplate *weaponTemplate)
{
	if (!weaponTemplate)
		return nullptr;

	// allocate a new weapon
	WeaponTemplate *wt = newInstance(WeaponTemplate);
	(*wt) = (*weaponTemplate);
	(wt)->friend_setNextTemplate(weaponTemplate);

	return wt;
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::update()
{
	for (std::list<WeaponDelayedDamageInfo>::iterator ddi = m_weaponDDI.begin(); ddi != m_weaponDDI.end(); )
	{
		UnsignedInt curFrame = TheGameLogic->getFrame();
		if (curFrame >= ddi->m_delayDamageFrame)
		{
			// we never do projectile-detonation-damage via this code path.
			const Bool isProjectileDetonation = false;
			ddi->m_delayedWeapon->dealDamageInternal(ddi->m_delaySourceID, ddi->m_delayIntendedVictimID, &ddi->m_delayDamagePos, ddi->m_bonus, isProjectileDetonation);
			ddi = m_weaponDDI.erase(ddi);
		}
		else
		{
			++ddi;
		}
	}
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::deleteAllDelayedDamage()
{
	m_weaponDDI.clear();
}

// ------------------------------------------------------------------------------------------------
void WeaponStore::resetWeaponTemplates()
{

	for (size_t i = 0; i < m_weaponTemplateVector.size(); i++)
	{
		WeaponTemplate* wt = m_weaponTemplateVector[i];
		wt->reset();
	}

}

//-------------------------------------------------------------------------------------------------
void WeaponStore::reset()
{
	// clean up any overrides.
	for (size_t i = 0; i < m_weaponTemplateVector.size(); ++i)
	{
		WeaponTemplate *wt = m_weaponTemplateVector[i];
		if (wt->isOverride())
		{
			WeaponTemplate *overrideData = wt;
			wt = wt->friend_clearNextTemplate();
			deleteInstance(overrideData);
		}
	}

	deleteAllDelayedDamage();
	resetWeaponTemplates();
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::setDelayedDamage(const WeaponTemplate *weapon, const Coord3D* pos, UnsignedInt whichFrame, ObjectID sourceID, ObjectID victimID, const WeaponBonus& bonus)
{
	WeaponDelayedDamageInfo wi;
	wi.m_delayedWeapon = weapon;
	wi.m_delayDamagePos = *pos;
	wi.m_delayDamageFrame = whichFrame;
	wi.m_delaySourceID = sourceID;
	wi.m_delayIntendedVictimID = victimID;
	wi.m_bonus = bonus;
	m_weaponDDI.push_back(wi);
}

//-------------------------------------------------------------------------------------------------
void WeaponStore::postProcessLoad()
{
	if (!TheThingFactory)
	{
		DEBUG_CRASH(("you must call this after TheThingFactory is inited"));
		return;
	}

	for (size_t i = 0; i < m_weaponTemplateVector.size(); i++)
	{
		WeaponTemplate* wt = m_weaponTemplateVector[i];
		if (wt)
			wt->postProcessLoad();
	}

}

//-------------------------------------------------------------------------------------------------
/*static*/ void WeaponStore::parseWeaponTemplateDefinition(INI* ini)
{
	AsciiString name;

	// read the weapon name
	const char* c = ini->getNextToken();
	name.set(c);

	// find existing item if present
	WeaponTemplate *weapon = TheWeaponStore->findWeaponTemplatePrivate( TheNameKeyGenerator->nameToKey( name ) );
	if (weapon)
	{
		if (ini->getLoadType() == INI_LOAD_CREATE_OVERRIDES)
			weapon = TheWeaponStore->newOverride(weapon);
		else
		{
			DEBUG_CRASH(("Weapon '%s' already exists, but OVERRIDE not specified", c));
			return;
		}

	}
	else
	{
		// no item is present, create a new one
		weapon = TheWeaponStore->newWeaponTemplate(name);
	}

	// parse the ini weapon definition
	ini->initFromINI(weapon, weapon->getFieldParse());

	if (weapon->m_projectileName.isNone())
		weapon->m_projectileName.clear();

#if defined(RTS_DEBUG)
	if (!weapon->getFireSound().getEventName().isEmpty() && weapon->getFireSound().getEventName().compareNoCase("NoSound") != 0)
	{
		DEBUG_ASSERTCRASH(TheAudio->isValidAudioEvent(&weapon->getFireSound()), ("Invalid FireSound %s in Weapon '%s'.", weapon->getFireSound().getEventName().str(), weapon->getName().str()));
	}
#endif

}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
Weapon::Weapon(const WeaponTemplate* tmpl, WeaponSlotType wslot)
{
	// Weapons start empty; you must reload before use.
	// (however, there is no delay for reloading the first time.)
	m_template = tmpl;
	m_wslot = wslot;
	m_status = OUT_OF_AMMO;
	m_ammoInClip = 0;
	m_whenWeCanFireAgain = 0;
	m_whenPreAttackFinished = 0;
	m_whenLastReloadStarted = 0;
	m_projectileStreamID = INVALID_ID;
	m_leechWeaponRangeActive = false;
	m_pitchLimited = (m_template->getMinTargetPitch() > -PI || m_template->getMaxTargetPitch() < PI);
	m_maxShotCount = NO_MAX_SHOTS_LIMIT;
	m_curBarrel = 0;
	m_numShotsForCurBarrel = 	m_template->getShotsPerBarrel();
	m_lastFireFrame = 0;
	m_suspendFXFrame = TheGameLogic->getFrame() + m_template->getSuspendFXDelay();
}

//-------------------------------------------------------------------------------------------------
Weapon::Weapon(const Weapon& that)
{
	// Weapons lose all ammo when copied.
	this->m_template = that.m_template;
	this->m_wslot = that.m_wslot;
	this->m_status = OUT_OF_AMMO;
	this->m_ammoInClip = 0;
	this->m_whenPreAttackFinished = 0;
	this->m_whenLastReloadStarted = 0;
	this->m_whenWeCanFireAgain = 0;
	this->m_projectileStreamID = INVALID_ID;
	this->m_leechWeaponRangeActive = false;
	this->m_pitchLimited = (m_template->getMinTargetPitch() > -PI || m_template->getMaxTargetPitch() < PI);
	this->m_maxShotCount = NO_MAX_SHOTS_LIMIT;
	this->m_curBarrel = 0;
	this->m_numShotsForCurBarrel = m_template->getShotsPerBarrel();
	this->m_lastFireFrame = 0;
	this->m_suspendFXFrame = that.getSuspendFXFrame();
}

//-------------------------------------------------------------------------------------------------
Weapon& Weapon::operator=(const Weapon& that)
{
	if (this != &that)
	{
		// Weapons lose all ammo when copied.
		this->m_template = that.m_template;
		this->m_wslot = that.m_wslot;
		this->m_status = OUT_OF_AMMO;
		this->m_ammoInClip = 0;
		this->m_whenPreAttackFinished = 0;
		this->m_whenLastReloadStarted = 0;
		this->m_whenWeCanFireAgain = 0;
		this->m_leechWeaponRangeActive = false;
		this->m_pitchLimited = (m_template->getMinTargetPitch() > -PI || m_template->getMaxTargetPitch() < PI);
		this->m_maxShotCount = NO_MAX_SHOTS_LIMIT;
		this->m_curBarrel = 0;
		this->m_lastFireFrame = 0;
		this->m_suspendFXFrame = that.getSuspendFXFrame();
		this->m_numShotsForCurBarrel = m_template->getShotsPerBarrel();
		this->m_projectileStreamID = INVALID_ID;
	}
	return *this;
}

//-------------------------------------------------------------------------------------------------
Weapon::~Weapon()
{
}

//-------------------------------------------------------------------------------------------------
void Weapon::computeBonus(const Object *source, WeaponBonusConditionFlags extraBonusFlags, WeaponBonus& bonus) const
{
	bonus.clear();
	WeaponBonusConditionFlags flags = source->getWeaponBonusCondition();
	//CRCDEBUG_LOG(("Weapon::computeBonus() - flags are %X for %s", flags, DescribeObject(source).str()));
	flags |= extraBonusFlags;

	if( source->getContainedBy() )
	{
		// We may be able to add in our container's flags
		const ContainModuleInterface *theirContain = source->getContainedBy()->getContain();
		if( theirContain && theirContain->isWeaponBonusPassedToPassengers() )
			flags |= theirContain->getWeaponBonusPassedToPassengers();
	}

	if (TheGlobalData->m_weaponBonusSet)
		TheGlobalData->m_weaponBonusSet->appendBonuses(flags, bonus);
	const WeaponBonusSet* extra = m_template->getExtraBonus();
	if (extra)
		extra->appendBonuses(flags, bonus);
}

//-------------------------------------------------------------------------------------------------
void Weapon::loadAmmoNow(const Object *sourceObj)
{
	WeaponBonus bonus;
	computeBonus(sourceObj, 0, bonus);
	reloadWithBonus(sourceObj, bonus, true);
}

//-------------------------------------------------------------------------------------------------
void Weapon::reloadAmmo(const Object *sourceObj)
{

	WeaponBonus bonus;
	computeBonus(sourceObj, 0, bonus);
	reloadWithBonus(sourceObj, bonus, false);
}

//-------------------------------------------------------------------------------------------------
Int Weapon::getClipReloadTime(const Object *source) const
{
	WeaponBonus bonus;
	computeBonus(source, 0, bonus);
	return m_template->getClipReloadTime(bonus);
}

//-------------------------------------------------------------------------------------------------
void Weapon::setClipPercentFull(Real percent, Bool allowReduction)
{
	if (m_template->getClipSize() == 0)
		return;

	Int ammo = REAL_TO_INT_FLOOR(m_template->getClipSize() * percent);
	if (ammo > m_ammoInClip || (allowReduction && ammo < m_ammoInClip))
	{
		m_ammoInClip = ammo;
		m_status = m_ammoInClip ? OUT_OF_AMMO : READY_TO_FIRE;
		//CRCDEBUG_LOG(("Weapon::setClipPercentFull() just set m_status to %d (ammo in clip is %d)", m_status, m_ammoInClip));
		m_whenLastReloadStarted = TheGameLogic->getFrame();
		m_whenWeCanFireAgain = m_whenLastReloadStarted;
		//CRCDEBUG_LOG(("Just set m_whenWeCanFireAgain to %d in Weapon::setClipPercentFull", m_whenWeCanFireAgain));
		rebuildScatterTargets();
	}
}

//-------------------------------------------------------------------------------------------------
void Weapon::rebuildScatterTargets()
{
	m_scatterTargetsUnused.clear();
	Int scatterTargetsCount = m_template->getScatterTargetsVector().size();
	if (scatterTargetsCount)
	{
		// When I reload, I need to rebuild the list of ScatterTargets to shoot at.
		for (Int targetIndex = 0; targetIndex < scatterTargetsCount; targetIndex++)
			m_scatterTargetsUnused.push_back( targetIndex );
	}
}

//-------------------------------------------------------------------------------------------------
void Weapon::reloadWithBonus(const Object *sourceObj, const WeaponBonus& bonus, Bool loadInstantly)
{
	if (m_template->getClipSize() > 0 &&
			m_ammoInClip == m_template->getClipSize() &&
			!sourceObj->isReloadTimeShared())
		return;	// don't restart our reload delay.

	m_ammoInClip = m_template->getClipSize();
	if (m_ammoInClip <= 0)
		m_ammoInClip = 0x7fffffff;	// 0 == unlimited (or effectively so)

	m_status = RELOADING_CLIP;
	Real reloadTime = loadInstantly ? 0 : m_template->getClipReloadTime(bonus);
	m_whenLastReloadStarted = TheGameLogic->getFrame();
	m_whenWeCanFireAgain = m_whenLastReloadStarted + reloadTime;
	//CRCDEBUG_LOG(("Just set m_whenWeCanFireAgain to %d in Weapon::reloadWithBonus 1", m_whenWeCanFireAgain));

			// if we are sharing reload times
			// go through other weapons in weapon set
			// set their m_whenWeCanFireAgain to this guy's delay
			// set their m_status to this guy's status
	if (sourceObj->isReloadTimeShared())
	{
		for (Int wt = 0; wt<WEAPONSLOT_COUNT; wt++)
		{
			Weapon *weapon = sourceObj->getWeaponInWeaponSlot((WeaponSlotType)wt);
			if (weapon)
			{
				weapon->setPossibleNextShotFrame(m_whenWeCanFireAgain);
				//CRCDEBUG_LOG(("Just set m_whenWeCanFireAgain to %d in Weapon::reloadWithBonus 2", m_whenWeCanFireAgain));
				weapon->setStatus(RELOADING_CLIP);
			}
		}
	}

	rebuildScatterTargets();
}

//-------------------------------------------------------------------------------------------------
static void clipToTerrainExtent(Coord3D& approachTargetPos)
{
	Region3D bounds;
	TheTerrainLogic->getExtent(&bounds);
	if (approachTargetPos.x < bounds.lo.x+PATHFIND_CELL_SIZE_F) {
		approachTargetPos.x = bounds.lo.x+PATHFIND_CELL_SIZE_F;
	}
	if (approachTargetPos.y < bounds.lo.y+PATHFIND_CELL_SIZE_F) {
		approachTargetPos.y = bounds.lo.y+PATHFIND_CELL_SIZE_F;
	}
	if (approachTargetPos.x > bounds.hi.x-PATHFIND_CELL_SIZE_F) {
		approachTargetPos.x = bounds.hi.x-PATHFIND_CELL_SIZE_F;
	}
	if (approachTargetPos.y > bounds.hi.y-PATHFIND_CELL_SIZE_F) {
		approachTargetPos.y = bounds.hi.y-PATHFIND_CELL_SIZE_F;
	}
}

//-------------------------------------------------------------------------------------------------
void Weapon::onWeaponBonusChange(const Object *source)
{
	// We are concerned with our reload times being off if our ROF just changed.

	WeaponBonus bonus;
	computeBonus(source, 0, bonus); // The middle arg is for projectiles to inherit damage bonus from launcher

	Int newDelay;
	Bool needUpdate = FALSE;

	if( getStatus() == RELOADING_CLIP )
	{
		newDelay = m_template->getClipReloadTime(bonus);
		needUpdate = TRUE;
	}
	else if( getStatus() == BETWEEN_FIRING_SHOTS )
	{
		newDelay = m_template->getDelayBetweenShots(bonus);
		needUpdate = TRUE;
	}

	if( needUpdate )
	{
		m_whenLastReloadStarted = TheGameLogic->getFrame();
		m_whenWeCanFireAgain = m_whenLastReloadStarted + newDelay;

		if (source->isReloadTimeShared())
		{
			for (Int wt = 0; wt<WEAPONSLOT_COUNT; wt++)
			{
				Weapon *weapon = source->getWeaponInWeaponSlot((WeaponSlotType)wt);
				if (weapon)
				{
					weapon->setPossibleNextShotFrame(m_whenWeCanFireAgain);
					weapon->setStatus(RELOADING_CLIP);
				}
			}
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::computeApproachTarget(const Object *source, const Object *target, const Coord3D *pos, Real angleOffset, Coord3D& approachTargetPos) const
{
	// compute unit direction vector from us to our victim
	const Coord3D *targetPos;
	Coord3D dir;
	if (target)
	{
		targetPos = target->getPosition();
		ThePartitionManager->getVectorTo( target, source, ATTACK_RANGE_CALC_TYPE, dir );
	}
	else if (pos)
	{
		targetPos = pos;
		ThePartitionManager->getVectorTo( source, pos, ATTACK_RANGE_CALC_TYPE, dir );
		// Flip the vector to get from source to pos.
		dir.x = -dir.x;
		dir.y = -dir.y;
		dir.z = -dir.z;
	}
	else
	{
		DEBUG_CRASH(("error"));
		approachTargetPos.zero();
		return false;
	}

	Real dist = dir.length();
	Real minAttackRange = m_template->getMinimumAttackRange();
	if (minAttackRange > PATHFIND_CELL_SIZE_F && dist < minAttackRange)
	{
		// We aret too close, so move away from the target.
		DEBUG_ASSERTCRASH((minAttackRange<0.9f*getAttackRange(source)), ("Min attack range is too near attack range."));
		// Recompute dir, cause if the bounding spheres touch, it will be 0.
		Coord3D srcPos = *source->getPosition();
		dir.x = srcPos.x-targetPos->x;
		dir.y = srcPos.y-targetPos->y;
#ifdef ATTACK_RANGE_IS_2D
		dir.z = 0.0f;
#else
		dir.z = srcPos.z-targetPos->z;
#endif
		dir.normalize();

		// if we're airborne and too close, just head for the opposite side.
		if (source->isAboveTerrain())
		{
			// Don't do a 180 degree turn.
			Real angle = WWMath::Atan2(-dir.y, -dir.x);
			Real relAngle = source->getOrientation()- angle;
			if (relAngle>2*PI) relAngle -= 2*PI;
			if (relAngle<-2*PI) relAngle += 2*PI;
			if (WWMath::Fabs(relAngle)<PI/2) {
				dir.x = -dir.x;
				dir.y = -dir.y;
				dir.z = -dir.z;
			}
		}

		if (angleOffset != 0.0f)
		{
			Real angle = WWMath::Atan2(dir.y, dir.x);
			dir.x = (Real)Cos(angle + angleOffset);
			dir.y = (Real)Sin(angle + angleOffset);
		}

		// select a spot along the line between us, halfway between the min & max range.
		Real attackRange = (getAttackRange(source) + minAttackRange)/2.0f;
#ifdef ATTACK_RANGE_IS_2D
		if (target)
			attackRange += target->getGeometryInfo().getBoundingCircleRadius();
		attackRange += source->getGeometryInfo().getBoundingCircleRadius();
#else
		if (target)
			attackRange += target->getGeometryInfo().getBoundingSphereRadius();
		attackRange += source->getGeometryInfo().getBoundingSphereRadius();
#endif
		approachTargetPos.x = attackRange * dir.x + targetPos->x;
		approachTargetPos.y = attackRange * dir.y + targetPos->y;
		approachTargetPos.z = attackRange * dir.z + targetPos->z;
		///@todo - make sure we can get to the approach position.
		clipToTerrainExtent(approachTargetPos);
		return false;
	}

	const Real FUDGE = 0.001f;
	if (dist < FUDGE)
	{
		// we're close enough!
		approachTargetPos = *source->getPosition();
		return true;
	}
	else
	{
		if (isContactWeapon())
		{
			// Weapon is basically a contact weapon, like a car bomb.  The approach target logic
			// has been modified to let it approach the object, so just return the target position.	jba.
			approachTargetPos = *targetPos;
			return false;
		}

		dir.x /= dist;
		dir.y /= dist;
		dir.z /= dist;

		if (angleOffset != 0.0f)
		{
			Real angle = WWMath::Atan2(dir.y, dir.x);
			dir.x = (Real)Cos(angle + angleOffset);
			dir.y = (Real)Sin(angle + angleOffset);
		}

		// select a spot along the line between us, in range of our weapon
		const Real ATTACK_RANGE_APPROACH_FUDGE = 0.9f;
		Real attackRange = getAttackRange(source) * ATTACK_RANGE_APPROACH_FUDGE;
		approachTargetPos.x = attackRange * dir.x + targetPos->x;
		approachTargetPos.y = attackRange * dir.y + targetPos->y;
		approachTargetPos.z = attackRange * dir.z + targetPos->z;

		if (source->getAI() && source->getAI()->isAircraftThatAdjustsDestination()) {
			// Adjust the target so that we are not stacked atop another aircraft.
			TheAI->pathfinder()->adjustTargetDestination(source, target, pos, this, &approachTargetPos);
		}

		return false;
	}
}

//-------------------------------------------------------------------------------------------------
//Special case attack range calculate that fakes moving the object (to a garrisoned point) without
//actually moving the object. This is used to help determine if a garrisoned unit not yet
//positioned can attack someone.
//-------------------------------------------------------------------------------------------------
Bool Weapon::isSourceObjectWithGoalPositionWithinAttackRange( const Object *source, const Coord3D *goalPos, const Object *target, const Coord3D *targetPos ) const
{

	Real distSqr;
	if( target )
		distSqr = ThePartitionManager->getGoalDistanceSquared( source, goalPos, target, ATTACK_RANGE_CALC_TYPE );
	else if( targetPos )
		distSqr = ThePartitionManager->getGoalDistanceSquared( source, goalPos, targetPos, ATTACK_RANGE_CALC_TYPE );
	else
		return false;

	Real attackRangeSqr = sqr( getAttackRange( source ) );
	Real minAttackRangeSqr = sqr(m_template->getMinimumAttackRange());
#ifdef RATIONALIZE_ATTACK_RANGE
	if (distSqr < minAttackRangeSqr)
#else
	if (distSqr < minAttackRangeSqr-0.5f)
#endif
	{
		return false;
	}
	return (distSqr <= attackRangeSqr);
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isWithinAttackRange(const Object *source, const Coord3D* pos) const
{
	Real distSqr = ThePartitionManager->getDistanceSquared( source, pos, ATTACK_RANGE_CALC_TYPE );
	Real attackRangeSqr = sqr(getAttackRange(source));
	Real minAttackRangeSqr = sqr(m_template->getMinimumAttackRange());
#ifdef RATIONALIZE_ATTACK_RANGE
	if (distSqr < minAttackRangeSqr)
#else
	if (distSqr < minAttackRangeSqr-0.5f)
#endif
	{
		return false;
	}
	return (distSqr <= attackRangeSqr);
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isWithinAttackRange(const Object *source, const Object *target) const
{
	Real distSqr;
	Real attackRangeSqr = sqr(getAttackRange(source));

	if( !target->isKindOf(KINDOF_BRIDGE) )
	{
		distSqr = ThePartitionManager->getDistanceSquared( source, target, ATTACK_RANGE_CALC_TYPE );
	}
	else
	{
		// Special case - bridges have two attackable points at either end.
		TBridgeAttackInfo info;
		TheTerrainLogic->getBridgeAttackPoints(target, &info);
		distSqr = ThePartitionManager->getDistanceSquared( source, &info.attackPoint1, ATTACK_RANGE_CALC_TYPE );
		if (distSqr>attackRangeSqr)
		{
			// Try the other one.
			distSqr = ThePartitionManager->getDistanceSquared( source, &info.attackPoint2, ATTACK_RANGE_CALC_TYPE );
		}
	}

	Real minAttackRangeSqr = sqr(m_template->getMinimumAttackRange());
#ifdef RATIONALIZE_ATTACK_RANGE
	if (distSqr < minAttackRangeSqr)
#else
	if (distSqr < minAttackRangeSqr-0.5f)
#endif
	{
		// too close. can't attack.
		return false;
	}

	if( distSqr <= attackRangeSqr )
	{
		// Note - only compare contact weapons with structures.  If you do the collision check
		// against vehicles, the attacker may get close enough to the vehicle to get crushed
		// before it fires its weapon.  jba.
		if( isContactWeapon() && target->isKindOf(KINDOF_STRUCTURE))
		{
			//We're close enough to fire off ranged weapons -- but in the case of contact weapons
			//we want to do a more detailed check to see if we're actually colliding with the target.
			ObjectIterator *iter = ThePartitionManager->iteratePotentialCollisions( source->getPosition(), source->getGeometryInfo(), 0.0f );
			MemoryPoolObjectHolder hold( iter );
			for( Object *them = iter->first(); them; them = iter->next() )
			{
				if( target == them )
				{
					return true;
				}
			}
			return false;
		}
		return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isTooClose(const Object *source, const Object *target) const
{
	Real minAttackRange = m_template->getMinimumAttackRange();
	if (minAttackRange == 0.0f)
		return false;

	Real distSqr = ThePartitionManager->getDistanceSquared( source, target, ATTACK_RANGE_CALC_TYPE );
	if (distSqr < sqr(minAttackRange))
	{
		return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isTooClose( const Object *source, const Coord3D *pos ) const
{
	Real minAttackRange = m_template->getMinimumAttackRange();
	if (minAttackRange == 0.0f)
		return false;

	Real distSqr = ThePartitionManager->getDistanceSquared( source, pos, ATTACK_RANGE_CALC_TYPE );
	if (distSqr < sqr(minAttackRange))
	{
		return true;
	}
	return false;
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isGoalPosWithinAttackRange(const Object *source, const Coord3D* goalPos, const Object *target, const Coord3D* targetPos)	const
{
	Real distSqr;
	// Note - undersize by 1/4 of a pathfind cell, so that the goal is not teetering on the edge
	// of firing range.  jba.
	// Note 2 - even with RATIONALIZE_ATTACK_RANGE, we still need to subtract 1/4 of a pathfind cell,
	// otherwise if it teters on the edge, attacks can fail.  jba.
	Real attackRangeSqr = sqr(getAttackRange(source)-(PATHFIND_CELL_SIZE_F*0.25f));

	if (target != nullptr)
	{
		if (target->isKindOf(KINDOF_BRIDGE))
		{
			// Special case - bridges have two attackable points at either end.
			TBridgeAttackInfo info;
			TheTerrainLogic->getBridgeAttackPoints(target, &info);
			distSqr = ThePartitionManager->getGoalDistanceSquared( source, goalPos, &info.attackPoint1, ATTACK_RANGE_CALC_TYPE );
			if (distSqr>attackRangeSqr)
			{
				// Try the other one.
				distSqr = ThePartitionManager->getGoalDistanceSquared( source, goalPos, &info.attackPoint2, ATTACK_RANGE_CALC_TYPE );
			}
		}
		else
		{
			distSqr = ThePartitionManager->getGoalDistanceSquared( source, goalPos, target, ATTACK_RANGE_CALC_TYPE );
		}
	}
	else
	{
		distSqr = ThePartitionManager->getGoalDistanceSquared( source, goalPos, targetPos, ATTACK_RANGE_CALC_TYPE );
	}

	// Note - oversize by 1/4 of a pathfind cell, so that the goal is not teetering on the edge
	// of firing range.  jba.
	// Note 2 - even with RATIONALIZE_ATTACK_RANGE, we still need to add 1/4 of a pathfind cell,
	// otherwise if it teters on the edge, attacks can fail.  jba.
	Real minAttackRangeSqr = sqr(m_template->getMinimumAttackRange()+(PATHFIND_CELL_SIZE_F*0.25f));
#ifdef RATIONALIZE_ATTACK_RANGE
	if (distSqr < minAttackRangeSqr)
#else
	if (distSqr < minAttackRangeSqr-0.5f)
#endif
	{
		return false;
	}
	return (distSqr <= attackRangeSqr);
}

//-------------------------------------------------------------------------------------------------
Real Weapon::getPercentReadyToFire() const
{
	switch (getStatus())
	{
		case OUT_OF_AMMO:
		case PRE_ATTACK:
			return 0.0f;

		case READY_TO_FIRE:
			return 1.0f;

		case BETWEEN_FIRING_SHOTS:
		case RELOADING_CLIP:
		{
			UnsignedInt now = TheGameLogic->getFrame();
			UnsignedInt nextShot = getPossibleNextShotFrame();
			DEBUG_ASSERTCRASH(now >= m_whenLastReloadStarted, ("now >= m_whenLastReloadStarted"));
			if (now >= nextShot)
				return 1.0f;

			DEBUG_ASSERTCRASH(nextShot >= m_whenLastReloadStarted, ("nextShot >= m_whenLastReloadStarted"));
			UnsignedInt totalTime = nextShot - m_whenLastReloadStarted;
			if (totalTime == 0)
			{
				return 1.0f;
			}

			UnsignedInt timeLeft = nextShot - now;
			DEBUG_ASSERTCRASH(timeLeft <= totalTime, ("timeLeft <= totalTime"));
			UnsignedInt timeSoFar = totalTime - timeLeft;
			if (timeSoFar >= totalTime)
			{
				return 1.0f;
			}
			else
			{
				return (Real)timeSoFar / (Real)totalTime;
			}
		}
	}
	DEBUG_CRASH(("should not get here"));
	return 0.0f;
}

//-------------------------------------------------------------------------------------------------
Real Weapon::getAttackRange(const Object *source) const
{
	WeaponBonus bonus;
	computeBonus(source, 0, bonus);
	return m_template->getAttackRange(bonus);

	//Contained objects have longer ranges.
	//const Object *container = source->getContainedBy();
	//if( container )
	//{
	//	attackRange += container->getGeometryInfo().getBoundingCircleRadius();
	//}
	//return attackRange;
}

//-------------------------------------------------------------------------------------------------
Real Weapon::getAttackDistance(const Object *source, const Object *victimObj, const Coord3D* victimPos) const
{
	Real range = getAttackRange(source);

	if (victimObj != nullptr)
	{
	#ifdef ATTACK_RANGE_IS_2D
		range += source->getGeometryInfo().getBoundingCircleRadius();
		range += victimObj->getGeometryInfo().getBoundingCircleRadius();
	#else
		range += source->getGeometryInfo().getBoundingSphereRadius();
		range += victimObj->getGeometryInfo().getBoundingSphereRadius();
	#endif
	}

	return range;
}

//-------------------------------------------------------------------------------------------------
Real Weapon::estimateWeaponDamage(const Object *sourceObj, const Object *victimObj, const Coord3D* victimPos)
{
	if (!m_template)
		return 0.0f;

	// if the weapon is just reloading, it's ok. if it's out of ammo
	// (and won't autoreload), then we aren't gonna do any damage.
	if (getStatus() == OUT_OF_AMMO && !m_template->getAutoReloadsClip())
		return 0.0f;

	WeaponBonus bonus;
	computeBonus(sourceObj, 0, bonus);

	return m_template->estimateWeaponTemplateDamage(sourceObj, victimObj, victimPos, bonus);
}

//-------------------------------------------------------------------------------------------------
void Weapon::newProjectileFired(const Object *sourceObj, const Object *projectile, const Object *victimObj, const Coord3D *victimPos )
{
	// If I have a stream, I need to tell it about this new guy
	if( m_template->getProjectileStreamName().isEmpty() )
		return; // nope, no streak logic to do

	Object* projectileStream = TheGameLogic->findObjectByID(m_projectileStreamID);
	if( projectileStream == nullptr )
	{
		m_projectileStreamID = INVALID_ID;	// reset, since it might have been "valid" but deleted out from under us
		const ThingTemplate* pst = TheThingFactory->findTemplate(m_template->getProjectileStreamName());
		projectileStream = TheThingFactory->newObject( pst, sourceObj->getControllingPlayer()->getDefaultTeam() );
		if( projectileStream == nullptr )
			return;
		m_projectileStreamID = projectileStream->getID();
	}

	//Check for projectile stream update
	static NameKeyType key_ProjectileStreamUpdate = NAMEKEY("ProjectileStreamUpdate");
	ProjectileStreamUpdate* update = (ProjectileStreamUpdate*)projectileStream->findUpdateModule(key_ProjectileStreamUpdate);
	if( update )
	{
		update->setPosition( sourceObj->getPosition() );
		update->addProjectile( sourceObj->getID(), projectile->getID(), victimObj ? victimObj->getID() : INVALID_ID, victimPos );
		return;
	}

}

//-------------------------------------------------------------------------------------------------
void Weapon::createLaser( const Object *sourceObj, const Object *victimObj, const Coord3D *victimPos )
{
	const ThingTemplate* pst = TheThingFactory->findTemplate(m_template->getLaserName());
	if( !pst )
	{
		DEBUG_CRASH( ("Weapon::createLaser(). %s could not find template for its laser %s.",
			sourceObj->getTemplate()->getName().str(), m_template->getLaserName().str() ) );
		return;
	}
	Object* laser = TheThingFactory->newObject( pst, sourceObj->getControllingPlayer()->getDefaultTeam() );
	if( laser == nullptr )
		return;

	// Give it a good basis in reality to ensure it can draw when on screen.
	laser->setPosition(sourceObj->getPosition());

	//Check for laser update
	Drawable *draw = laser->getDrawable();
	if( draw )
	{
		static NameKeyType key_LaserUpdate = NAMEKEY( "LaserUpdate" );
		LaserUpdate *update = (LaserUpdate*)draw->findClientUpdateModule( key_LaserUpdate );
		if( update )
		{
			Coord3D pos = *victimPos;
			if( victimObj && !victimObj->isKindOf( KINDOF_PROJECTILE ) && !victimObj->isAirborneTarget() )
			{
				//Targets are positioned on the ground, so raise the beam up so we're not shooting their feet.
				//Projectiles are a different story, target their exact position.
				pos.z += 10.0f;
			}
			update->initLaser( sourceObj, victimObj, sourceObj->getPosition(), &pos, m_template->getLaserBoneName() );
		}
	}
}

//-------------------------------------------------------------------------------------------------
// return true if we auto-reloaded our clip after firing.
//DECLARE_PERF_TIMER(fireWeapon)
Bool Weapon::privateFireWeapon(
	const Object *sourceObj,
	Object *victimObj,
	const Coord3D* victimPos,
	Bool isProjectileDetonation,
	Bool ignoreRanges,
	WeaponBonusConditionFlags extraBonusFlags,
	ObjectID* projectileID,
	Bool inflictDamage
)
{
	//CRCDEBUG_LOG(("Weapon::privateFireWeapon() for %s", DescribeObject(sourceObj).str()));
	//USE_PERF_TIMER(fireWeapon)
	if (projectileID)
		*projectileID = INVALID_ID;

	if (!m_template)
		return false;

	// If we are a networked weapon, tell everyone nearby they might want to get in on this shot
	if( m_template->getRequestAssistRange()  &&  victimObj )
		processRequestAssistance( sourceObj, victimObj );

	//For weapon templates that have the leech range weapon flag set, it essentially grants
	//the weapon unlimited range for the remainder of the attack. While it's triggered here
	//it's the AIAttackState machine that actually uses and resets this value.
	//This makes the ASSUMPTION that it is IMPOSSIBLE TO FIRE A WEAPON WITHOUT BEING IN AN AIATTACKSTATE
	//
	// @todo srj -- this isn't a universally true assertion! eg, FireWeaponDie lets you do this easily.
	//
	if( m_template->isLeechRangeWeapon() )
	{
		setLeechRangeActive( TRUE );
	}

	//Special case damage type overrides requiring special handling.
	switch( m_template->getDamageType() )
	{
		case DAMAGE_DEPLOY:
		{
			const AIUpdateInterface *ai = sourceObj->getAI();
			if( ai )
			{
				const AssaultTransportAIInterface *atInterface = ai->getAssaultTransportAIInterface();
				if( atInterface )
				{
					atInterface->beginAssault( victimObj );
				}
			}
			break;
		}

		case DAMAGE_DISARM:
		{
			if (sourceObj && victimObj)
			{
				Bool found = false;
				for (BehaviorModule** bmi = victimObj->getBehaviorModules(); *bmi; ++bmi)
				{
					LandMineInterface* lmi = (*bmi)->getLandMineInterface();
					if (lmi)
					{
						VeterancyLevel v = sourceObj->getVeterancyLevel();
						FXList::doFXPos(m_template->getFireFX(v), victimObj->getPosition(), victimObj->getTransformMatrix(), 0, victimObj->getPosition(), 0);
						lmi->disarm();
						found = true;
						break;
					}
				}

				// it's a mine, but doesn't have LandMineInterface...
				if( (!found && victimObj->isKindOf( KINDOF_MINE )) || victimObj->isKindOf( KINDOF_BOOBY_TRAP ) || victimObj->isKindOf( KINDOF_DEMOTRAP ) )
				{
					VeterancyLevel v = sourceObj->getVeterancyLevel();
					FXList::doFXPos(m_template->getFireFX(v), victimObj->getPosition(), victimObj->getTransformMatrix(), 0, victimObj->getPosition(), 0);
					TheGameLogic->destroyObject( victimObj );// douse this thing before somebody gets hurt!
					found = true;
				}

				if( found )
				{
					sourceObj->getControllingPlayer()->getAcademyStats()->recordMineCleared();
				}
			}

			--m_maxShotCount;
			--m_ammoInClip;	// so we can use the delay between shots on the mine clearing weapon
			if (m_ammoInClip <= 0 && m_template->getAutoReloadsClip())
			{
				reloadAmmo(sourceObj);
				return TRUE;	// reloaded
			}
			else
			{
				return FALSE;	// did not reload
			}
		}

		case DAMAGE_HACK:
		{
			//We're using a hacker unit to hack a target. Hacking has various effects and
			//instead of inflicting damage, we are waiting for a period of time until the hack takes effect.
			//return FALSE;
		}
	}

	WeaponBonus bonus;
	computeBonus(sourceObj, extraBonusFlags, bonus);

	DEBUG_ASSERTCRASH(getStatus() != OUT_OF_AMMO, ("Hmm, firing weapon that is OUT_OF_AMMO"));
	DEBUG_ASSERTCRASH(getStatus() == READY_TO_FIRE, ("Hmm, Weapon is firing more often than should be possible"));
	DEBUG_ASSERTCRASH(m_ammoInClip > 0, ("Hmm, firing an empty weapon"));

	if (getStatus() != READY_TO_FIRE)
		return false;

	UnsignedInt now = TheGameLogic->getFrame();
	Bool reloaded = false;
	if (m_ammoInClip > 0)
	{
		// TheSuperHackers @logic-client-separation helmutbuhler 11/04/2025
		// barrelCount shouln't depend on Drawable, which belongs to client.
		Int barrelCount = sourceObj->getDrawable()->getBarrelCount(m_wslot);
		if (m_curBarrel >= barrelCount)
		{
			m_curBarrel = 0;
			m_numShotsForCurBarrel = m_template->getShotsPerBarrel();
		}

		if( !m_scatterTargetsUnused.empty() )
		{
			// If I have a set scatter pattern, I need to offset the target by a random pick from that pattern
			if( victimObj )
			{
				victimPos = victimObj->getPosition();
				victimObj = nullptr;
			}
			Coord3D targetPos = *victimPos; // need to copy, as this pointer is actually inside somebody potentially
			Int randomPick = GameLogicRandomValue( 0, m_scatterTargetsUnused.size() - 1 );
			Int targetIndex = m_scatterTargetsUnused[randomPick];

			Real scatterTargetScalar = getScatterTargetScalar();// essentially a radius, but operates only on this scatterTarget table
			Coord2D scatterOffset = m_template->getScatterTargetsVector().at( targetIndex );

			scatterOffset.x *= scatterTargetScalar;
			scatterOffset.y *= scatterTargetScalar;

			targetPos.x += scatterOffset.x;
			targetPos.y += scatterOffset.y;

			targetPos.z = TheTerrainLogic->getGroundHeight(targetPos.x, targetPos.y);

			// To erase from a vector, put the last on the one you used and pop the back.
			m_scatterTargetsUnused[randomPick] = m_scatterTargetsUnused.back();
			m_scatterTargetsUnused.pop_back();
			m_template->fireWeaponTemplate(sourceObj, m_wslot, m_curBarrel, victimObj, &targetPos, bonus, isProjectileDetonation, ignoreRanges, this, projectileID, inflictDamage );
		}
		else
		{
			m_template->fireWeaponTemplate(sourceObj, m_wslot, m_curBarrel, victimObj, victimPos, bonus, isProjectileDetonation, ignoreRanges, this, projectileID, inflictDamage );
		}

		m_lastFireFrame = now;
		--m_ammoInClip;
		--m_maxShotCount;
		--m_numShotsForCurBarrel;
		if (m_numShotsForCurBarrel <= 0)
		{
			++m_curBarrel;
			m_numShotsForCurBarrel = m_template->getShotsPerBarrel();
		}

		if (m_ammoInClip <= 0)
		{
			if (m_template->getAutoReloadsClip())
			{
				reloadAmmo(sourceObj);
				reloaded = true;
			}
			else
			{
				m_status = OUT_OF_AMMO;
				m_whenWeCanFireAgain = 0x7fffffff;
				//CRCDEBUG_LOG(("Just set m_whenWeCanFireAgain to %d in Weapon::privateFireWeapon 1", m_whenWeCanFireAgain));
			}
		}
		else
		{
			m_status = BETWEEN_FIRING_SHOTS;
			//CRCDEBUG_LOG(("Weapon::privateFireWeapon() just set m_status to BETWEEN_FIRING_SHOTS"));
			Int delay = m_template->getDelayBetweenShots(bonus);
			m_whenLastReloadStarted = now;
			m_whenWeCanFireAgain = now + delay;
			//CRCDEBUG_LOG(("Just set m_whenWeCanFireAgain to %d (delay is %d) in Weapon::privateFireWeapon", m_whenWeCanFireAgain, delay));

			// if we are sharing reload times
			// go through other weapons in weapon set
			// set their m_whenWeCanFireAgain to this guy's delay
			// set their m_status to this guy's status

			if ( sourceObj->isReloadTimeShared() )
			{
				for (Int wt = 0; wt<WEAPONSLOT_COUNT; wt++)
				{
					Weapon *weapon = sourceObj->getWeaponInWeaponSlot((WeaponSlotType)wt);
					if (weapon)
					{
						weapon->setPossibleNextShotFrame(m_whenWeCanFireAgain);
						//CRCDEBUG_LOG(("Just set m_whenWeCanFireAgain to %d in Weapon::privateFireWeapon 3", m_whenWeCanFireAgain));
						weapon->setStatus(BETWEEN_FIRING_SHOTS);
					}
				}
			}

		}
	}

	return reloaded;
}


//-------------------------------------------------------------------------------------------------
void Weapon::preFireWeapon( const Object *source, const Object *victim )
{
	Int delay = getPreAttackDelay( source, victim );
	if( delay > 0 )
	{
		setStatus( PRE_ATTACK );
		setPreAttackFinishedFrame( TheGameLogic->getFrame() + delay );
		if( m_template->isLeechRangeWeapon() )
		{
			setLeechRangeActive( TRUE );
		}
	}
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::fireWeapon(const Object *source, Object *target, ObjectID* projectileID)
{
	//CRCDEBUG_LOG(("Weapon::fireWeapon() for %s at %s", DescribeObject(source).str(), DescribeObject(target).str()));
	return privateFireWeapon( source, target, nullptr, false, false, 0, projectileID, TRUE );
}

//-------------------------------------------------------------------------------------------------
// return true if we auto-reloaded our clip after firing.
Bool Weapon::fireWeapon(const Object *source, const Coord3D* pos, ObjectID* projectileID)
{
	//CRCDEBUG_LOG(("Weapon::fireWeapon() for %s", DescribeObject(source).str()));
	return privateFireWeapon( source, nullptr, pos, false, false, 0, projectileID, TRUE );
}

//-------------------------------------------------------------------------------------------------
void Weapon::fireProjectileDetonationWeapon(const Object *source, Object *target, WeaponBonusConditionFlags extraBonusFlags, Bool inflictDamage )
{
	//CRCDEBUG_LOG(("Weapon::fireProjectileDetonationWeapon() for %sat %s", DescribeObject(source).str(), DescribeObject(target).str()));
	privateFireWeapon( source, target, nullptr, true, false, extraBonusFlags, nullptr, inflictDamage );
}

//-------------------------------------------------------------------------------------------------
void Weapon::fireProjectileDetonationWeapon(const Object *source, const Coord3D* pos, WeaponBonusConditionFlags extraBonusFlags, Bool inflictDamage )
{
	//CRCDEBUG_LOG(("Weapon::fireProjectileDetonationWeapon() for %s", DescribeObject(source).str()));
	privateFireWeapon( source, nullptr, pos, true, false, extraBonusFlags, nullptr, inflictDamage );
}

//-------------------------------------------------------------------------------------------------
//Currently, this function was added to allow a script to force fire a weapon,
//and immediately gain control of the weapon that was fired to give it special orders...
Object* Weapon::forceFireWeapon( const Object *source, const Coord3D *pos)
{
	//CRCDEBUG_LOG(("Weapon::forceFireWeapon() for %s", DescribeObject(source).str()));
	//Force the ammo to load instantly.
	//loadAmmoNow( source );
	//Fire the weapon at the position. Internally, it'll store the weapon projectile ID if so created.
	ObjectID projectileID = INVALID_ID;
	const Bool ignoreRange = true;
	privateFireWeapon(source, nullptr, pos, false, ignoreRange, 0, &projectileID, TRUE );
	return TheGameLogic->findObjectByID( projectileID );
}

//-------------------------------------------------------------------------------------------------
WeaponStatus Weapon::getStatus() const
{
	UnsignedInt now = TheGameLogic->getFrame();
	if( now < m_whenPreAttackFinished )
	{
		return PRE_ATTACK;
	}
	if( now >= m_whenWeCanFireAgain )
	{
		if (m_ammoInClip > 0)
			m_status = READY_TO_FIRE;
		else
			m_status = OUT_OF_AMMO;
		//CRCDEBUG_LOG(("Weapon::getStatus() just set m_status to %d (ammo in clip is %d)", m_status, m_ammoInClip));
	}
	return m_status;
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isWithinTargetPitch(const Object *source, const Object *victim) const
{
	if (isContactWeapon() || !isPitchLimited())
		return true;

	const Coord3D* src = source->getPosition();
	const Coord3D* dst = victim->getPosition();

	const Real ACCEPTABLE_DZ = 10.0f;
	if (WWMath::Fabs(dst->z - src->z) < ACCEPTABLE_DZ)
		return true;	// always good enough if dz is small, regardless of pitch

	Real minPitch, maxPitch;
	source->getGeometryInfo().calcPitches(*src, victim->getGeometryInfo(), *dst, minPitch, maxPitch);

	// if there's any intersection between the the two pitch ranges, we're good to go.
	if ((minPitch >= m_template->getMinTargetPitch() && minPitch <= m_template->getMaxTargetPitch()) ||
			(maxPitch >= m_template->getMinTargetPitch() && maxPitch <= m_template->getMaxTargetPitch()) ||
			(minPitch <= m_template->getMinTargetPitch() && maxPitch >= m_template->getMaxTargetPitch()))
		return true;

	//DEBUG_LOG(("pitch %f-%f is out of range",rad2deg(minPitch),rad2deg(maxPitch),rad2deg(m_template->getMinTargetPitch()),rad2deg(m_template->getMaxTargetPitch())));
	return false;
}

//-------------------------------------------------------------------------------------------------
Real Weapon::getPrimaryDamageRadius(const Object *source) const
{
	WeaponBonus bonus;
	computeBonus(source, 0, bonus);
	return m_template->getPrimaryDamageRadius(bonus);
}

//-------------------------------------------------------------------------------------------------
Bool Weapon::isDamageWeapon() const
{
	//These damage types are special attacks that don't do damage directly, even
	//if they can indirectly. These are here to prevent the UI from allowing the
	//user to mouseover a target and think it can attack it using these types.
	switch( m_template->getDamageType() )
	{
		case DAMAGE_DEPLOY:
			//Kris @todo
			//Evaluate a better way to handle this weapon type... doesn't fit being a damage weapon.
			//May want to check if cargo can attack!
			return TRUE;

		case DAMAGE_DISARM:
			return TRUE;	// hmm, can only "damage" mines, but still...

		case DAMAGE_HACK:
			return FALSE;
	}

	//Use no bonus
	WeaponBonus whoCares;
	if( m_template->getPrimaryDamage( whoCares ) > 0.0f || m_template->getSecondaryDamage( whoCares ) > 0.0f )
	{
		return TRUE;
	}

	return FALSE;
}

//-------------------------------------------------------------------------------------------------
Int Weapon::getPreAttackDelay( const Object *source, const Object *victim ) const
{
	// Look for a reason to return zero and have no delay.
	WeaponPrefireType type = m_template->getPrefireType();
	if( type == PREFIRE_PER_CLIP )
	{
		if( m_template->getClipSize() > 0  &&  m_ammoInClip < m_template->getClipSize() )
			return 0;// I only delay once a clip, and this is not the first shot
	}
	else if( type == PREFIRE_PER_ATTACK )
	{
		if( source->getNumConsecutiveShotsFiredAtTarget( victim ) > 0 )
			return 0;// I only delay once an attack, and I have already shot this guy
	}
	//else it is per shot, so it always applies

	WeaponBonus bonus;
	computeBonus(source, 0, bonus);
	return m_template->getPreAttackDelay( bonus );
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
class AssistanceRequestData
{
public:
	AssistanceRequestData();

	const Object *m_requestingObject;
	Object *m_victimObject;
	Real m_requestDistanceSquared;
};

//-------------------------------------------------------------------------------------------------
AssistanceRequestData::AssistanceRequestData()
{
	m_requestingObject = nullptr;
	m_victimObject = nullptr;
	m_requestDistanceSquared = 0.0f;
}

//-------------------------------------------------------------------------------------------------
static void makeAssistanceRequest( Object *requestOf, void *userData )
{
	AssistanceRequestData *requestData = (AssistanceRequestData *)userData;

	// Don't ask ourselves (can't believe I forgot this one)
	if( requestOf == requestData->m_requestingObject )
		return;

	// Only request of our kind of people
	if( !requestOf->getTemplate()->isEquivalentTo( requestData->m_requestingObject->getTemplate() ) )
		return;

	// Who are close enough
	Real distSq = ThePartitionManager->getDistanceSquared( requestOf, requestData->m_requestingObject, FROM_CENTER_2D );
	if( distSq > requestData->m_requestDistanceSquared )
		return;

	// and respond to requests
	static const NameKeyType key_assistUpdate = NAMEKEY("AssistedTargetingUpdate");
	AssistedTargetingUpdate *assistModule = (AssistedTargetingUpdate*)requestOf->findUpdateModule(key_assistUpdate);
	if( assistModule == nullptr )
		return;

	// and say yes
	if( !assistModule->isFreeToAssist() )
		return;

	assistModule->assistAttack( requestData->m_requestingObject, requestData->m_victimObject );
}

//-------------------------------------------------------------------------------------------------
void Weapon::processRequestAssistance( const Object *requestingObject, Object *victimObject )
{
	// Iterate through our player's objects, and tell everyone like us within our assistance range
	// who is free to do so to assist us on this shot.
	Player *ourPlayer = requestingObject->getControllingPlayer();
	if( !ourPlayer )
		return;

	AssistanceRequestData requestData;
	requestData.m_requestingObject = requestingObject;
	requestData.m_victimObject = victimObject;
	requestData.m_requestDistanceSquared = m_template->getRequestAssistRange() * m_template->getRequestAssistRange();

	ourPlayer->iterateObjects( makeAssistanceRequest, &requestData );
}

//-------------------------------------------------------------------------------------------------
/*static*/ void Weapon::calcProjectileLaunchPosition(
	const Object* launcher,
	WeaponSlotType wslot,
	Int specificBarrelToUse,
	Matrix3D& worldTransform,
	Coord3D& worldPos
)
{
	if( launcher->getContainedBy() )
	{
		// If we are in an enclosing container, our launch position is our actual position.  Yes, I am putting
		// a minor case and an oft used function, but the major case is huge and full of math.
		if(launcher->getContainedBy()->getContain()->isEnclosingContainerFor(launcher))
		{
			worldTransform = *launcher->getTransformMatrix();
			Vector3 tmp = worldTransform.Get_Translation();
			worldPos.x = tmp.X;
			worldPos.y = tmp.Y;
			worldPos.z = tmp.Z;
			return;
		}
	}

	Real turretAngle = 0.0f;
	Real turretPitch = 0.0f;
	const AIUpdateInterface* ai = launcher->getAIUpdateInterface();
	WhichTurretType tur = ai ? ai->getWhichTurretForWeaponSlot(wslot, &turretAngle, &turretPitch) : TURRET_INVALID;
	//CRCDEBUG_LOG(("calcProjectileLaunchPosition(): Turret %d, slot %d, barrel %d for %s", tur, wslot, specificBarrelToUse, DescribeObject(launcher).str()));

	Matrix3D attachTransform(true);
	Coord3D turretRotPos = {0.0f, 0.0f, 0.0f};
	Coord3D turretPitchPos = {0.0f, 0.0f, 0.0f};
	const Drawable* draw = launcher->getDrawable();
	//CRCDEBUG_LOG(("Do we have a drawable? %d", (draw != nullptr)));
	if (!draw || !draw->getProjectileLaunchOffset(wslot, specificBarrelToUse, &attachTransform, tur, &turretRotPos, &turretPitchPos))
	{
		//CRCDEBUG_LOG(("ProjectileLaunchPos %d %d not found!",wslot, specificBarrelToUse));
		DEBUG_CRASH(("ProjectileLaunchPos %d %d not found!",wslot, specificBarrelToUse));
		attachTransform.Make_Identity();
		turretRotPos.zero();
		turretPitchPos.zero();
	}
	if (tur != TURRET_INVALID)
	{
		// The attach transform is the pristine front and center position of the fire point
		// We can't read from the client, so we need to reproduce the actual point that
		// takes turn and pitch into account.
		Matrix3D turnAdjustment(1);
		Matrix3D pitchAdjustment(1);

		// To rotate about a point, move that point to 0,0, rotate, then move it back.
		// Pre rotate will keep the first twist from screwing the angle of the second pitch
		pitchAdjustment.Translate( turretPitchPos.x, turretPitchPos.y, turretPitchPos.z );
		pitchAdjustment.In_Place_Pre_Rotate_Y(-turretPitch);
		pitchAdjustment.Translate( -turretPitchPos.x, -turretPitchPos.y, -turretPitchPos.z );

		turnAdjustment.Translate( turretRotPos.x, turretRotPos.y, turretRotPos.z );
		turnAdjustment.In_Place_Pre_Rotate_Z(turretAngle);
		turnAdjustment.Translate( -turretRotPos.x, -turretRotPos.y, -turretRotPos.z );

#ifdef ALLOW_TEMPORARIES
		attachTransform = turnAdjustment * pitchAdjustment * attachTransform;
#else
		Matrix3D tmp = attachTransform;
		attachTransform.mul(turnAdjustment, pitchAdjustment);
		attachTransform.postMul(tmp);
#endif
	}

//#if defined(RTS_DEBUG)
//  Real muzzleHeight = attachTransform.Get_Z_Translation();
//  DEBUG_ASSERTCRASH( muzzleHeight > 0.001f, ("YOUR TURRET HAS A VERY LOW PROJECTILE LAUNCH POSITION, BUT FOUND A VALID BONE. DID YOU PICK THE WRONG ONE? %s", launcher->getTemplate()->getName().str()));
//#endif

  launcher->convertBonePosToWorldPos(nullptr, &attachTransform, nullptr, &worldTransform);

	Vector3 tmp = worldTransform.Get_Translation();
	worldPos.x = tmp.X;
	worldPos.y = tmp.Y;
	worldPos.z = tmp.Z;
}

//-------------------------------------------------------------------------------------------------
/*static*/ void Weapon::positionProjectileForLaunch(
	Object* projectile,
	const Object* launcher,
	WeaponSlotType wslot,
	Int specificBarrelToUse
)
{
	//CRCDEBUG_LOG(("Weapon::positionProjectileForLaunch() for %s from %s",
		//DescribeObject(projectile).str(), DescribeObject(launcher).str()));

	// if our launch vehicle is gone, destroy ourselves
	if (launcher == nullptr)
	{
		TheGameLogic->destroyObject( projectile );
		return;
	}

	Matrix3D worldTransform(true);
	Coord3D worldPos;

	Weapon::calcProjectileLaunchPosition(launcher, wslot, specificBarrelToUse, worldTransform, worldPos);

	projectile->getDrawable()->setDrawableHidden(false);
	projectile->setTransformMatrix(&worldTransform);
	projectile->setPosition(&worldPos);
	projectile->getExperienceTracker()->setExperienceSink( launcher->getID() );

	const PhysicsBehavior* launcherPhys = launcher->getPhysics();
	PhysicsBehavior* missilePhys = projectile->getPhysics();
	if (launcherPhys && missilePhys)
	{
		launcherPhys->transferVelocityTo(missilePhys);
		missilePhys->setIgnoreCollisionsWith(launcher);
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
void Weapon::getFiringLineOfSightOrigin(const Object* source, Coord3D& origin) const
{
	//GS 1-6-03
	// Sorry, but we have to simplify this.  If we take the actual projectile launch pos, then
	// that point can change. Take a Ranger with his gun on his shoulder.  His point is very high so
	// he clears this check and transitions to attacking.  This puts his gun at waist level and
	// now he fails this check so he transitions back.  Our height won't change.
	origin.z += source->getGeometryInfo().getMaxHeightAbovePosition();

/*
	if (m_template->getProjectileTemplate() == nullptr)
	{
		// note that we want to measure from the top of the collision
		// shape, not the bottom! (most objects have eyes a lot closer
		// to their head than their feet. if we have really odd critters
		// with eye-feet, we'll need to change this assumption.)
		origin.z += source->getGeometryInfo().getMaxHeightAbovePosition();
	}
	else
	{
		Matrix3D tmp(true);
		Coord3D launchPos = {0.0f, 0.0f, 0.0f};
		calcProjectileLaunchPosition(source, m_wslot, m_curBarrel, tmp, launchPos);
		origin.x += launchPos.x - source->getPosition()->x;
		origin.y += launchPos.y - source->getPosition()->y;
		origin.z += launchPos.z - source->getPosition()->z;
	}
*/
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool Weapon::isClearFiringLineOfSightTerrain(const Object* source, const Object* victim) const
{
	Coord3D origin;
	origin = *source->getPosition();
	//CRCDEBUG_LOG(("Weapon::isClearFiringLineOfSightTerrain(Object) for %s", DescribeObject(source).str()));
	//DUMPCOORD3D(&origin);
	getFiringLineOfSightOrigin(source, origin);
	Coord3D victimPos;
	victim->getGeometryInfo().getCenterPosition( *victim->getPosition(), victimPos );
	//CRCDEBUG_LOG(("Weapon::isClearFiringLineOfSightTerrain() - victimPos is (%g,%g,%g) (%X,%X,%X)",
	//	victimPos.x, victimPos.y, victimPos.z,
	//	AS_INT(victimPos.x),AS_INT(victimPos.y),AS_INT(victimPos.z)));
	return ThePartitionManager->isClearLineOfSightTerrain(nullptr, origin, nullptr, victimPos);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
Bool Weapon::isClearFiringLineOfSightTerrain(const Object* source, const Coord3D& victimPos) const
{
	Coord3D origin;
	origin = *source->getPosition();
	//CRCDEBUG_LOG(("Weapon::isClearFiringLineOfSightTerrain(Coord3D) for %s", DescribeObject(source).str()));
	//DUMPCOORD3D(&origin);
	getFiringLineOfSightOrigin(source, origin);
	return ThePartitionManager->isClearLineOfSightTerrain(nullptr, origin, nullptr, victimPos);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/** Determine whether if source was at goalPos whether it would have clear line of sight. */
Bool Weapon::isClearGoalFiringLineOfSightTerrain(const Object* source, const Coord3D& goalPos, const Object* victim) const
{
	Coord3D origin=goalPos;
	//CRCDEBUG_LOG(("Weapon::isClearGoalFiringLineOfSightTerrain(Object) for %s", DescribeObject(source).str()));
	//DUMPCOORD3D(&origin);
	getFiringLineOfSightOrigin(source, origin);
	Coord3D victimPos;
	victim->getGeometryInfo().getCenterPosition( *victim->getPosition(), victimPos );
	return ThePartitionManager->isClearLineOfSightTerrain(nullptr, origin, nullptr, victimPos);
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------
/** Determine whether if source was at goalPos whether it would have clear line of sight. */
Bool Weapon::isClearGoalFiringLineOfSightTerrain(const Object* source, const Coord3D& goalPos, const Coord3D& victimPos) const
{
	Coord3D origin=goalPos;
	//CRCDEBUG_LOG(("Weapon::isClearGoalFiringLineOfSightTerrain(Coord3D) for %s", DescribeObject(source).str()));
	//DUMPCOORD3D(&origin);
	getFiringLineOfSightOrigin(source, origin);
	//CRCDEBUG_LOG(("Weapon::isClearFiringLineOfSightTerrain() - victimPos is (%g,%g,%g) (%X,%X,%X)",
	//	victimPos.x, victimPos.y, victimPos.z,
	//	AS_INT(victimPos.x),AS_INT(victimPos.y),AS_INT(victimPos.z)));
	return ThePartitionManager->isClearLineOfSightTerrain(nullptr, origin, nullptr, victimPos);
}

//-------------------------------------------------------------------------------------------------
//Kris: Patch 1.01 - November 10, 2003
//This function was added to transfer key weapon stats for Jarmen Kell to and from the bike for
//the sniper attack, so he can share the stats.
//-------------------------------------------------------------------------------------------------
void Weapon::transferNextShotStatsFrom( const Weapon &weapon )
{
	m_whenWeCanFireAgain = weapon.getPossibleNextShotFrame();
	m_whenLastReloadStarted = weapon.getLastReloadStartedFrame();
	m_status = weapon.getStatus();
}


// ------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------
void Weapon::crc( Xfer *xfer )
{
#ifdef DEBUG_CRC
	AsciiString logString;
	AsciiString tmp;
	Bool doLogging = g_logObjectCRCs;
	if (doLogging)
	{
		tmp.format("CRC of weapon %s: ", m_template->getName().str());
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	AsciiString tmplName = m_template->getName();
	xfer->xferAsciiString(&tmplName);

	// slot
	xfer->xferUser( &m_wslot, sizeof( WeaponSlotType ) );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_wslot %d ", m_wslot);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// status
	/*
	xfer->xferUser( &m_status, sizeof( WeaponStatus ) );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_status %d ", m_status);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC
	*/

	// ammo
	xfer->xferUnsignedInt( &m_ammoInClip );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_ammoInClip %d ", m_ammoInClip);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// when can fire again
	xfer->xferUnsignedInt( &m_whenWeCanFireAgain );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_whenWeCanFireAgain %d ", m_whenWeCanFireAgain);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// when pre attack finished
	xfer->xferUnsignedInt( &m_whenPreAttackFinished );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_whenPreAttackFinished %d ", m_whenPreAttackFinished);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// when last reload started
	xfer->xferUnsignedInt( &m_whenLastReloadStarted );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_whenLastReloadStarted %d ", m_whenLastReloadStarted);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// last fire frame
	xfer->xferUnsignedInt( &m_lastFireFrame );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_lastFireFrame %d ", m_lastFireFrame);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// projectile stream object
	xfer->xferObjectID( &m_projectileStreamID );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("projectileStreamID %d ", m_projectileStreamID);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// laser object (defunct)
	ObjectID laserIDUnused = INVALID_ID;
	xfer->xferObjectID( &laserIDUnused );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("laserID %d ", laserIDUnused);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// max shot count
	xfer->xferInt( &m_maxShotCount );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_maxShotCount %d ", m_maxShotCount);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// current barrel
	xfer->xferInt( &m_curBarrel );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_curBarrel %d ", m_curBarrel);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// num shots for current barrel
	xfer->xferInt( &m_numShotsForCurBarrel );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_numShotsForCurBarrel %d ", m_numShotsForCurBarrel);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// scatter targets unused
	UnsignedShort scatterCount = m_scatterTargetsUnused.size();
	xfer->xferUnsignedShort( &scatterCount );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("scatterCount %d ", scatterCount);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	Int intData;

	std::vector< Int >::const_iterator it;

	for( it = m_scatterTargetsUnused.begin(); it != m_scatterTargetsUnused.end(); ++it )
	{

		intData = *it;
		xfer->xferInt( &intData );
#ifdef DEBUG_CRC
		if (doLogging)
		{
			tmp.format("%d ", intData);
			logString.concat(tmp);
		}
#endif // DEBUG_CRC

	}

	// pitch limited
	xfer->xferBool( &m_pitchLimited );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_pitchLimited %d ", m_pitchLimited);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC

	// leech weapon range active
	xfer->xferBool( &m_leechWeaponRangeActive );
#ifdef DEBUG_CRC
	if (doLogging)
	{
		tmp.format("m_leechWeaponRangeActive %d ", m_leechWeaponRangeActive);
		logString.concat(tmp);
	}
#endif // DEBUG_CRC


#ifdef DEBUG_CRC
	if (doLogging)
	{
		CRCDEBUG_LOG(("%s", logString.str()));
	}
#endif // DEBUG_CRC

}

// ------------------------------------------------------------------------------------------------
/** Xfer
	* Version Info:
	* 1: Initial version */
// ------------------------------------------------------------------------------------------------
void Weapon::xfer( Xfer *xfer )
{
	// version
	const XferVersion currentVersion = 3;
	XferVersion version = currentVersion;
	xfer->xferVersion( &version, currentVersion );

	if (version >= 2)
	{
		AsciiString tmplName = m_template->getName();
		xfer->xferAsciiString(&tmplName);
		if (xfer->getXferMode() == XFER_LOAD)
		{
			m_template = TheWeaponStore->findWeaponTemplate(tmplName);
			if (m_template == nullptr)
				throw INI_INVALID_DATA;
		}
	}

	// slot
	xfer->xferUser( &m_wslot, sizeof( WeaponSlotType ) );

	// status
	xfer->xferUser( &m_status, sizeof( WeaponStatus ) );

	// ammo
	xfer->xferUnsignedInt( &m_ammoInClip );

	// when can fire again
	xfer->xferUnsignedInt( &m_whenWeCanFireAgain );

	// when pre attack finished
	xfer->xferUnsignedInt( &m_whenPreAttackFinished );

	// when last reload started
	xfer->xferUnsignedInt( &m_whenLastReloadStarted );

	// last fire frame
	xfer->xferUnsignedInt( &m_lastFireFrame );

	// suspendFXFrame, this affects client only
	if ( version >= 3 )
		xfer->xferUnsignedInt( &m_suspendFXFrame );
	else
		m_suspendFXFrame = 0;

	// projectile stream object
	xfer->xferObjectID( &m_projectileStreamID );

	// laser object
	ObjectID laserIDUnused = INVALID_ID;
	xfer->xferObjectID( &laserIDUnused );

	// max shot count
	xfer->xferInt( &m_maxShotCount );

	// current barrel
	xfer->xferInt( &m_curBarrel );

	// num shots for current barrel
	xfer->xferInt( &m_numShotsForCurBarrel );

	// scatter targets unused
	UnsignedShort scatterCount = m_scatterTargetsUnused.size();
	xfer->xferUnsignedShort( &scatterCount );
	Int intData;
	if( xfer->getXferMode() == XFER_SAVE )
	{
		std::vector< Int >::const_iterator it;

		for( it = m_scatterTargetsUnused.begin(); it != m_scatterTargetsUnused.end(); ++it )
		{

			intData = *it;
			xfer->xferInt( &intData );

		}

	}
	else
	{

		// sanity, the scatter targets must be empty
		m_scatterTargetsUnused.clear();

		for( UnsignedShort i = 0; i < scatterCount; ++i )
		{

			xfer->xferInt( &intData );
			m_scatterTargetsUnused.push_back( intData );

		}

	}

	// pitch limited
	xfer->xferBool( &m_pitchLimited );

	// leech weapon range active
	xfer->xferBool( &m_leechWeaponRangeActive );

}

// ------------------------------------------------------------------------------------------------
/** Load post process */
// ------------------------------------------------------------------------------------------------
void Weapon::loadPostProcess()
{
	if( m_projectileStreamID != INVALID_ID )
	{
		Object* projectileStream = TheGameLogic->findObjectByID( m_projectileStreamID );
		if( projectileStream == nullptr )
		{
			m_projectileStreamID = INVALID_ID;
		}
	}
}

//-------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------

//-------------------------------------------------------------------------------------------------
void WeaponBonus::appendBonuses(WeaponBonus& bonus) const
{
	for (int f = 0; f < WeaponBonus::FIELD_COUNT; ++f)
	{
		bonus.m_field[f] += this->m_field[f] - 1.0f;
	}
}

//-------------------------------------------------------------------------------------------------
/*static*/ void WeaponBonusSet::parseWeaponBonusSet(INI* ini, void* /*instance*/, void* store, const void* /*userData*/)
{
	WeaponBonusSet* self = (WeaponBonusSet*)store;
	self->parseWeaponBonusSet(ini);
}

//-------------------------------------------------------------------------------------------------
/*static*/ void WeaponBonusSet::parseWeaponBonusSetPtr(INI* ini, void* /*instance*/, void* store, const void* /*userData*/)
{
	WeaponBonusSet** selfPtr = (WeaponBonusSet**)store;
	(*selfPtr)->parseWeaponBonusSet(ini);
}

//-------------------------------------------------------------------------------------------------
void WeaponBonusSet::parseWeaponBonusSet(INI* ini)
{
	WeaponBonusConditionType wb = (WeaponBonusConditionType)INI::scanIndexList(ini->getNextToken(), TheWeaponBonusNames);
	WeaponBonus::Field wf = (WeaponBonus::Field)INI::scanIndexList(ini->getNextToken(), TheWeaponBonusFieldNames);
	m_bonus[wb].setField(wf, INI::scanPercentToReal(ini->getNextToken()));
}

//-------------------------------------------------------------------------------------------------
void WeaponBonusSet::appendBonuses(WeaponBonusConditionFlags flags, WeaponBonus& bonus) const
{
	if (flags == 0)
		return;	// my, that was easy

	for (int i = 0; i < WEAPONBONUSCONDITION_COUNT; ++i)
	{
		if ((flags & (1 << i)) == 0)
			continue;

		this->m_bonus[i].appendBonuses(bonus);
	}
}
```
