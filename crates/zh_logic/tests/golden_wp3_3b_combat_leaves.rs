//! Gate V2 Golden Vector parity tests for `WP-3.3b` (`PKT-141` `DamageFX`,
//! `PKT-142` `FiringTracker`, and manifest enum re-exports `PKT-135`, `PKT-136`,
//! `PKT-146`–`PKT-149`).

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_str, get_u32, get_u32_array,
};
use zh_common::ini::{IniError, IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, UpdateSleepTime, VeterancyLevel, FALSE,
    TRUE,
};
use zh_common::xfer::{XferCrc, XferLoad, XferSave};

use zh_logic::arena::FacetMask;
use zh_logic::combat::damage_fx::{DamageFX, DamageFXStore};
use zh_logic::combat::firing_tracker::{
    FiredWeaponInfo, FiringTracker, FiringTrackerHost, FiringTrackerModuleData, AHSV_NO_SOUND,
};
use zh_logic::combat::weapon_bonus_condition_flags::{
    WeaponBonusConditionFlags, WeaponBonusConditionType,
};
use zh_logic::combat::weapon_set_flags::{WeaponSetFlags, WEAPONSET_COUNT, WEAPON_SET_NAMES};
use zh_logic::combat::weapon_set_type::WeaponSetType;
use zh_logic::combat::weapon_status::{WeaponStatus, WEAPON_STATUS_COUNT};
use zh_logic::modules::{BehaviorModule, ModuleData};
use zh_logic::object::thing_template::LEVEL_COUNT;
use zh_logic::rules::combat_types::{DamageType, DAMAGE_NUM_TYPES};
use zh_logic::rules::disabled_types::DisabledMaskType;
use zh_logic::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use zh_logic::rules::special_power_mask_type::{
    SpecialPowerMaskType, SPECIALPOWER_COUNT, SPECIAL_POWER_NAMES,
};
use zh_logic::rules::special_power_type::SpecialPowerType;
use zh_logic::scheduler::SleepyUpdatePhase;

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_3b_CombatLeaves.json");

const FX_NAMES: [&str; 13] = [
    "None",
    "FX_Alpha",
    "FX_Beta",
    "FX_Gamma",
    "FX_Delta",
    "FX_Epsilon",
    "FX_Zeta",
    "FX_Eta",
    "FX_Theta",
    "FX_Iota",
    "FX_Kappa",
    "FX_Lambda",
    "FX_Mu",
];

#[inline]
fn fnv_step(mut h: u32, v: u32) -> u32 {
    h ^= v;
    h = h.wrapping_mul(16_777_619u32);
    h
}

fn fx_key_to_id(key: Option<NameKeyType>, fx_keys: &[NameKeyType; 13]) -> i32 {
    match key {
        None => 0,
        Some(k) => fx_keys
            .iter()
            .enumerate()
            .skip(1)
            .find_map(|(i, &fk)| if fk == k { Some(i as i32) } else { None })
            .unwrap_or(-1),
    }
}

struct MockTrackerHost {
    frame: UnsignedInt,
    shooter_id: ObjectId,
    victim_a_id: ObjectId,
    victim_b_id: ObjectId,
    victim_a_ff: Bool,
    victim_b_ff: Bool,
    weapon_bonus_mask: UnsignedInt,
    model_cond: ModelConditionFlags,
    next_audio_handle: UnsignedInt,
    audio_playing: Bool,
    audio_add_count: u32,
    audio_remove_count: u32,
    rapid_fire_count: u32,
    wake_frame: UnsignedInt,
}

impl MockTrackerHost {
    fn new(
        frame: UnsignedInt,
        shooter_id: ObjectId,
        victim_a_id: ObjectId,
        victim_b_id: ObjectId,
        victim_a_ff: Bool,
        victim_b_ff: Bool,
    ) -> Self {
        Self {
            frame,
            shooter_id,
            victim_a_id,
            victim_b_id,
            victim_a_ff,
            victim_b_ff,
            weapon_bonus_mask: 0,
            model_cond: ModelConditionFlags::new(),
            next_audio_handle: 100,
            audio_playing: TRUE,
            audio_add_count: 0,
            audio_remove_count: 0,
            rapid_fire_count: 0,
            wake_frame: frame.wrapping_add(UpdateSleepTime::FOREVER.0),
        }
    }
}

impl FiringTrackerHost for MockTrackerHost {
    fn current_frame(&self) -> UnsignedInt {
        self.frame
    }

    fn object_id(&self) -> ObjectId {
        self.shooter_id
    }

    fn victim_has_faerie_fire(&self, victim_id: ObjectId) -> Bool {
        if victim_id == self.victim_a_id {
            self.victim_a_ff
        } else if victim_id == self.victim_b_id {
            self.victim_b_ff
        } else {
            FALSE
        }
    }

    fn test_weapon_bonus_condition(&self, cond: WeaponBonusConditionType) -> Bool {
        Bool::from((self.weapon_bonus_mask & (1u32 << (cond as u32))) != 0)
    }

    fn set_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType) {
        self.weapon_bonus_mask |= 1u32 << (cond as u32);
    }

    fn clear_weapon_bonus_condition(&mut self, cond: WeaponBonusConditionType) {
        self.weapon_bonus_mask &= !(1u32 << (cond as u32));
    }

    fn clear_and_set_model_condition_flags(
        &mut self,
        clr: &ModelConditionFlags,
        set: &ModelConditionFlags,
    ) {
        self.model_cond.clear_and_set(clr, set);
    }

    fn reload_all_ammo(&mut self, _now: Bool) {}

    fn is_audio_playing(&self, _handle: UnsignedInt) -> Bool {
        self.audio_playing
    }

    fn add_fire_sound(&mut self, _sound_key: NameKeyType, _object_id: ObjectId) -> UnsignedInt {
        self.audio_add_count = self.audio_add_count.wrapping_add(1);
        let h = self.next_audio_handle;
        self.next_audio_handle = self.next_audio_handle.wrapping_add(1);
        h
    }

    fn add_rapid_fire_voice(&mut self, _object_id: ObjectId) -> UnsignedInt {
        self.audio_add_count = self.audio_add_count.wrapping_add(1);
        self.rapid_fire_count = self.rapid_fire_count.wrapping_add(1);
        let h = self.next_audio_handle;
        self.next_audio_handle = self.next_audio_handle.wrapping_add(1);
        h
    }

    fn remove_audio_event(&mut self, audio_event: UnsignedInt) {
        if audio_event != AHSV_NO_SOUND {
            self.audio_remove_count = self.audio_remove_count.wrapping_add(1);
        }
    }

    fn set_wake_frame(&mut self, wake_delay: UpdateSleepTime) {
        self.wake_frame = self.frame.wrapping_add(wake_delay.0);
    }
}

/// Parses `[[k, dt, off, play], ...]` from a JSON vector line.
fn parse_ft_steps(line: &str) -> Vec<[u32; 4]> {
    let key = "\"ft_steps\":[[";
    let start = line.find(key).expect("ft_steps") + "\"ft_steps\":[".len();
    let rest = &line[start..];
    let mut depth = 1i32;
    let mut end = 0usize;
    for (idx, b) in rest.bytes().enumerate() {
        match b {
            b'[' => depth += 1,
            b']' => {
                depth -= 1;
                if depth == 0 {
                    end = idx;
                    break;
                }
            }
            _ => {}
        }
    }
    let inner = &rest[..end];
    let mut out = Vec::new();
    for chunk in inner.split("],[") {
        let trimmed = chunk.trim_matches(|c| c == '[' || c == ']');
        let nums: Vec<u32> = trimmed
            .split(',')
            .map(|s| IniReader::scan_int(s.trim()).expect("step u32") as u32)
            .collect();
        assert_eq!(nums.len(), 4);
        out.push([nums[0], nums[1], nums[2], nums[3]]);
    }
    out
}

#[test]
fn test_wp3_3b_oracle_structure_and_manifest_enums() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("\"seed_0_diagnostics\""));
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    // Verify all 6 manifest-aligned enum/flag re-exports (`PKT-135`, `PKT-136`, `PKT-146`..`PKT-149`).
    assert_eq!(SPECIALPOWER_COUNT, 67);
    assert_eq!(SPECIAL_POWER_NAMES.len(), 67);
    let sp_mask = SpecialPowerMaskType::from_single_index(SpecialPowerType::DaisyCutter.as_i32());
    assert_eq!(sp_mask.test(SpecialPowerType::DaisyCutter.as_i32()), TRUE);

    let wb_flags: WeaponBonusConditionFlags =
        1u32 << (WeaponBonusConditionType::ContinuousFireMean as u32);
    assert_ne!(wb_flags, 0);

    assert_eq!(WEAPONSET_COUNT, 17);
    assert_eq!(WEAPON_SET_NAMES.len(), 17);
    let ws_flags = WeaponSetFlags::from_single_index(WeaponSetType::Veteran.as_i32());
    assert_eq!(ws_flags.test(WeaponSetType::Veteran.as_i32()), TRUE);

    assert_eq!(WEAPON_STATUS_COUNT, 5);
    assert_eq!(WeaponStatus::ReadyToFire.as_i32(), 0);
    assert_eq!(WeaponStatus::PreAttack.as_i32(), 4);

    // Verify FiringTrackerModuleData metadata contract.
    let mdata = FiringTrackerModuleData::default();
    assert_eq!(mdata.facet_mask(), FacetMask::UPDATE);
    assert_eq!(mdata.update_phase(), SleepyUpdatePhase::Final);
    assert!(mdata.as_any().is::<FiringTrackerModuleData>());
}

#[test]
fn test_wp3_3b_damage_fx_1000_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let mut fx_keys = [NameKeyType::INVALID; 13];
    for i in 1..13 {
        fx_keys[i] = name_keys.name_to_key(FX_NAMES[i]);
    }
    let resolve_fx = |name: &str| -> Option<NameKeyType> {
        FX_NAMES
            .iter()
            .enumerate()
            .skip(1)
            .find_map(|(i, &n)| if n.eq_ignore_ascii_case(name) { Some(fx_keys[i]) } else { None })
    };

    // Default DamageFX returns None / 0 for all damage types.
    let empty_dfx = DamageFX::new();
    for dt in 0..DAMAGE_NUM_TYPES {
        let d = DamageType::from_i32(dt as i32).unwrap();
        assert_eq!(empty_dfx.get_damage_fx_throttle_time(d, None), 0);
        assert_eq!(empty_dfx.get_damage_fx_list(d, 100.0_f32, None), None);
    }

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let dfx_name = get_str(line, "dfx_name").unwrap();
        let dfx_ini = get_str(line, "dfx_ini").unwrap().replace("\\n", "\n");
        let bad_ini = get_str(line, "bad_ini").unwrap().replace("\\n", "\n");
        let bad_threw = line.contains("\"bad_threw\":true");
        let expected_table_hash = get_u32(line, "dfx_table_hash").unwrap();
        let p_dt: [i32; 4] = get_i32_vec(line, "p_dt").unwrap().try_into().unwrap();
        let p_vet: [i32; 4] = get_i32_vec(line, "p_vet").unwrap().try_into().unwrap();
        let p_amt_bits: [u32; 4] = get_u32_array(line, "p_amt_bits").unwrap();
        let p_fx: [i32; 4] = get_i32_vec(line, "p_fx").unwrap().try_into().unwrap();
        let p_thr: [u32; 4] = get_u32_array(line, "p_thr").unwrap();

        let mut store = DamageFXStore::new();
        store.init();
        let mut ini = IniReader::new("DamageFX.ini", dfx_ini.as_bytes(), IniLoadType::Overwrite);
        store
            .load_from_ini(&mut ini, &mut name_keys, resolve_fx)
            .unwrap_or_else(|e| panic!("seed {seed}: DamageFX load_from_ini failed: {e:?}"));

        if bad_threw {
            let mut bad_store = DamageFXStore::new();
            let mut bini = IniReader::new("BadDFX.ini", bad_ini.as_bytes(), IniLoadType::Overwrite);
            assert_eq!(
                bad_store.load_from_ini(&mut bini, &mut name_keys, resolve_fx),
                Err(IniError::InvalidData),
                "seed {seed}: expected InvalidData for bad INI"
            );
        }

        assert!(store
            .find_damage_fx_by_name(&mut name_keys, "NonExistent_DFX_XYZ")
            .is_none());

        let dfx = store
            .find_damage_fx_by_name(&mut name_keys, dfx_name)
            .unwrap_or_else(|| panic!("seed {seed}: missing DamageFX {dfx_name}"));

        let mut table_hash = 2166136261u32;
        for dt_idx in 0..DAMAGE_NUM_TYPES {
            let dt = DamageType::from_i32(dt_idx as i32).unwrap();
            for v_idx in 0..LEVEL_COUNT {
                let vet = Some(VeterancyLevel::from_i32(v_idx as i32));
                let thr = dfx.get_damage_fx_throttle_time(dt, vet);
                let f0 = fx_key_to_id(dfx.get_damage_fx_list(dt, 0.0_f32, vet), &fx_keys);
                let f_neg = fx_key_to_id(dfx.get_damage_fx_list(dt, -1.0_f32, vet), &fx_keys);
                let f_mid = fx_key_to_id(dfx.get_damage_fx_list(dt, 50.0_f32, vet), &fx_keys);
                let f_hi = fx_key_to_id(dfx.get_damage_fx_list(dt, 1000.0_f32, vet), &fx_keys);
                table_hash = fnv_step(table_hash, thr);
                table_hash = fnv_step(table_hash, f0 as u32);
                table_hash = fnv_step(table_hash, f_neg as u32);
                table_hash = fnv_step(table_hash, f_mid as u32);
                table_hash = fnv_step(table_hash, f_hi as u32);
            }
        }
        assert_eq!(
            table_hash, expected_table_hash,
            "seed {seed}: DamageFX table_hash mismatch"
        );

        for p in 0..4 {
            let dt = DamageType::from_i32(p_dt[p]).unwrap();
            let vet = if p_vet[p] < 0 {
                None
            } else {
                Some(VeterancyLevel::from_i32(p_vet[p]))
            };
            let amt = Real::from_bits(p_amt_bits[p]);
            let got_fx = fx_key_to_id(dfx.get_damage_fx_list(dt, amt, vet), &fx_keys);
            let got_thr = dfx.get_damage_fx_throttle_time(dt, vet);
            assert_eq!(got_fx, p_fx[p], "seed {seed} probe {p}: fx mismatch");
            assert_eq!(got_thr, p_thr[p], "seed {seed} probe {p}: throttle mismatch");

            let mut callback_fx = 0i32;
            dfx.do_damage_fx(dt, amt, vet, |k| {
                callback_fx = fx_key_to_id(Some(k), &fx_keys);
            });
            assert_eq!(callback_fx, p_fx[p], "seed {seed} probe {p}: do_damage_fx");
        }
    }
}

#[test]
fn test_wp3_3b_firing_tracker_1000_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let frame0 = get_u32(line, "ft_frame0").unwrap();
        let shots_one = get_i32(line, "ft_shots_one").unwrap();
        let shots_two = get_i32(line, "ft_shots_two").unwrap();
        let coast_ms = get_u32(line, "ft_coast_ms").unwrap();
        let reload_ms = get_u32(line, "ft_reload_ms").unwrap();
        let loop_ms = get_u32(line, "ft_loop_ms").unwrap();
        let shooter_id = ObjectId(get_i32(line, "ft_shooter_id").unwrap());
        let victim_a_id = ObjectId(get_i32(line, "ft_victim_a_id").unwrap());
        let victim_b_id = ObjectId(get_i32(line, "ft_victim_b_id").unwrap());
        let victim_a_ff = get_u32(line, "ft_victim_a_ff").unwrap() as Bool;
        let victim_b_ff = get_u32(line, "ft_victim_b_ff").unwrap() as Bool;

        let exp_init_phase = get_u32(line, "ft_init_phase").unwrap();
        let exp_init_wake = get_u32(line, "ft_init_wake").unwrap();
        let exp_init_dis_all = get_u32(line, "ft_init_dis_all").unwrap();
        let exp_trace_hash = get_u32(line, "ft_trace_hash").unwrap();
        let exp_final_consec = get_i32(line, "ft_final_consec").unwrap();
        let exp_final_victim = get_i32(line, "ft_final_victim").unwrap();
        let exp_final_cd = get_u32(line, "ft_final_cd").unwrap();
        let exp_final_rl = get_u32(line, "ft_final_rl").unwrap();
        let exp_final_lp = get_u32(line, "ft_final_lp").unwrap();
        let exp_next_call_raw = get_u32(line, "ft_next_call_raw").unwrap();
        let exp_xfer_bytes = decode_hex_bytes(get_str(line, "ft_xfer_hex").unwrap()).unwrap();
        let exp_xfer_crc = get_u32(line, "ft_xfer_crc").unwrap();

        // Parse weapon durations matching C++ INI::parseDurationUnsignedInt (`convertDurationFromMsecsToFrames`).
        let parse_dur = |ms: u32| -> u32 {
            if ms == 0 {
                return 0;
            }
            let s = format!("D = {ms}\n");
            let mut r = IniReader::new("w.ini", s.as_bytes(), IniLoadType::Overwrite);
            r.read_line(None).unwrap();
            let _ = r.next_token(SEPS_DEFAULT).unwrap();
            let mut out = 0u32;
            r.parse_duration_unsigned_int(&mut out).unwrap();
            out
        };
        let coast_frames = parse_dur(coast_ms);
        let reload_frames = parse_dur(reload_ms);
        let loop_frames = parse_dur(loop_ms);

        let mut ft = FiringTracker::new(NameKeyType::INVALID, frame0);
        let mut host = MockTrackerHost::new(
            frame0,
            shooter_id,
            victim_a_id,
            victim_b_id,
            victim_a_ff,
            victim_b_ff,
        );

        assert_eq!(ft.update_phase() as u32, exp_init_phase, "seed {seed}: phase");
        assert_eq!(
            ft.schedule().get_wake_frame(frame0).0,
            exp_init_wake,
            "seed {seed}: init_wake"
        );
        assert_eq!(
            (ft.disabled_types_to_process() == DisabledMaskType::new_set_all()) as u32,
            exp_init_dis_all,
            "seed {seed}: init_dis_all"
        );
        assert_eq!(
            BehaviorModule::as_update(&ft).unwrap().get_update_phase(),
            SleepyUpdatePhase::Final
        );

        let steps = parse_ft_steps(line);
        let mut cur_frame = frame0;
        let mut trace_hash = 2166136261u32;

        for (st_idx, [kind, dt_frames, next_shot_off, audio_play]) in steps.iter().enumerate() {
            cur_frame = cur_frame.wrapping_add(*dt_frames);
            host.frame = cur_frame;
            host.audio_playing = if *audio_play != 0 { TRUE } else { FALSE };

            let mut upd_sleep = 0u32;
            if *kind < 3 {
                let vid = match *kind {
                    0 => victim_a_id,
                    1 => victim_b_id,
                    _ => ObjectId::INVALID,
                };
                let wpn = FiredWeaponInfo {
                    auto_reload_when_idle_frames: reload_frames,
                    continuous_fire_coast_frames: coast_frames,
                    possible_next_shot_frame: cur_frame.wrapping_add(*next_shot_off),
                    continuous_fire_one_shots_needed: shots_one,
                    continuous_fire_two_shots_needed: shots_two,
                    fire_sound_loop_time: loop_frames,
                    fire_sound_key: NameKeyType(1),
                };
                ft.shot_fired(&mut host, &wpn, vid);
            } else {
                let sl = ft.update_with_host(&mut host);
                upd_sleep = sl.0;
            }

            let consec = ft.get_num_consecutive_shots_at_victim(Some(victim_a_id));
            let consec_b = ft.get_num_consecutive_shots_at_victim(Some(victim_b_id));
            let consec_null = ft.get_num_consecutive_shots_at_victim(None);
            let last_v = ft.get_last_shot_victim();
            let wake_f = ft.schedule().get_wake_frame(cur_frame).0;
            let cd_f = ft.frame_to_start_cooldown();
            let rl_f = ft.frame_to_force_reload();
            let lp_f = ft.frame_to_stop_looping_sound();
            let wb_mask = host.weapon_bonus_mask;
            let mc_mean = u32::from(
                host.model_cond
                    .test(ModelConditionFlagType::ContinuousFireMean as Int)
                    != FALSE,
            );
            let mc_fast = u32::from(
                host.model_cond
                    .test(ModelConditionFlagType::ContinuousFireFast as Int)
                    != FALSE,
            );
            let mc_slow = u32::from(
                host.model_cond
                    .test(ModelConditionFlagType::ContinuousFireSlow as Int)
                    != FALSE,
            );

            trace_hash = fnv_step(trace_hash, upd_sleep);
            trace_hash = fnv_step(trace_hash, consec as u32);
            trace_hash = fnv_step(trace_hash, consec_b as u32);
            trace_hash = fnv_step(trace_hash, consec_null as u32);
            trace_hash = fnv_step(trace_hash, last_v.0 as u32);
            trace_hash = fnv_step(trace_hash, wake_f);
            trace_hash = fnv_step(trace_hash, cd_f);
            trace_hash = fnv_step(trace_hash, rl_f);
            trace_hash = fnv_step(trace_hash, lp_f);
            trace_hash = fnv_step(trace_hash, wb_mask);
            trace_hash = fnv_step(trace_hash, (mc_mean << 2) | (mc_fast << 1) | mc_slow);
            trace_hash = fnv_step(trace_hash, host.audio_add_count);
            trace_hash = fnv_step(trace_hash, host.audio_remove_count);
            trace_hash = fnv_step(trace_hash, host.rapid_fire_count);

            let _ = st_idx;
        }

        assert_eq!(
            trace_hash, exp_trace_hash,
            "seed {seed}: FiringTracker trace_hash mismatch"
        );
        assert_eq!(ft.consecutive_shots(), exp_final_consec, "seed {seed}: consec");
        assert_eq!(ft.get_last_shot_victim().0, exp_final_victim, "seed {seed}: victim");
        assert_eq!(ft.frame_to_start_cooldown(), exp_final_cd, "seed {seed}: cd");
        assert_eq!(ft.frame_to_force_reload(), exp_final_rl, "seed {seed}: rl");
        assert_eq!(ft.frame_to_stop_looping_sound(), exp_final_lp, "seed {seed}: lp");
        assert_eq!(
            ft.schedule().priority(),
            exp_next_call_raw,
            "seed {seed}: m_nextCallFrameAndPhase"
        );

        // XferSave parity
        let mut saver = XferSave::new();
        ft.xfer_snapshot(&mut saver).unwrap();
        #[cfg(feature = "retail-xfer-save")]
        assert_eq!(
            saver.as_bytes(),
            exp_xfer_bytes.as_slice(),
            "seed {seed}: retail XferSave mismatch"
        );
        #[cfg(not(feature = "retail-xfer-save"))]
        {
            let bytes = saver.as_bytes();
            assert_eq!(bytes.len(), exp_xfer_bytes.len() + 8, "seed {seed}: v2 len");
            assert_eq!(bytes[0], 2, "seed {seed}: v2 version byte");
            assert_eq!(&bytes[1..21], &exp_xfer_bytes[1..21], "seed {seed}: v2 prefix");
            assert_eq!(
                &bytes[21..25],
                &exp_final_rl.to_le_bytes(),
                "seed {seed}: v2 reload"
            );
            assert_eq!(
                &bytes[25..29],
                &exp_final_lp.to_le_bytes(),
                "seed {seed}: v2 loop"
            );

            // Round-trip v2 save through XferLoad.
            let mut ft_v2 = FiringTracker::default();
            let mut loader_v2 = XferLoad::new(bytes);
            ft_v2.xfer_snapshot(&mut loader_v2).unwrap();
            assert_eq!(loader_v2.position(), bytes.len());
            assert_eq!(ft_v2.consecutive_shots(), ft.consecutive_shots());
            assert_eq!(ft_v2.get_last_shot_victim(), ft.get_last_shot_victim());
            assert_eq!(ft_v2.frame_to_start_cooldown(), ft.frame_to_start_cooldown());
            assert_eq!(ft_v2.frame_to_force_reload(), ft.frame_to_force_reload());
            assert_eq!(
                ft_v2.frame_to_stop_looping_sound(),
                ft.frame_to_stop_looping_sound()
            );
            assert_eq!(ft_v2.schedule().priority(), ft.schedule().priority());
            assert_eq!(ft_v2.audio_handle(), AHSV_NO_SOUND);
        }

        // Load C++ v1 retail save bytes cleanly in both modes.
        let mut ft_v1 = FiringTracker::default();
        let mut loader_v1 = XferLoad::new(&exp_xfer_bytes);
        ft_v1.xfer_snapshot(&mut loader_v1).unwrap();
        assert_eq!(loader_v1.position(), exp_xfer_bytes.len());
        assert_eq!(ft_v1.consecutive_shots(), exp_final_consec);
        assert_eq!(ft_v1.get_last_shot_victim().0, exp_final_victim);
        assert_eq!(ft_v1.frame_to_start_cooldown(), exp_final_cd);
        assert_eq!(ft_v1.schedule().priority(), exp_next_call_raw);

        // XferCRC parity
        let mut xcrc = XferCrc::new();
        ft.crc_snapshot(&mut xcrc);
        assert_eq!(xcrc.get_crc(), exp_xfer_crc, "seed {seed}: XferCRC mismatch");

        // Cleanup audio on destruction
        let prev_rem = host.audio_remove_count;
        let had_handle = ft.audio_handle() != AHSV_NO_SOUND;
        ft.on_destroy_audio(&mut host);
        assert_eq!(ft.audio_handle(), AHSV_NO_SOUND);
        if had_handle {
            assert_eq!(host.audio_remove_count, prev_rem + 1);
        }
    }
}
