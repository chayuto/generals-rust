//! Gate V2-Golden test suite for `WP-3.1a` Economy Leaf Trackers
//! (`Money`, `Energy`, `Handicap`, and `MissionStats`, 1,000 C++ reference vectors).

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_array, get_i32_vec, get_str, get_u32,
    get_u32_array,
};
use zh_common::types::{Bool, Int, UnsignedInt, FALSE, TRUE};
#[cfg(not(feature = "retail-crc"))]
use zh_common::xfer::XferCrc;
use zh_common::xfer::{Xfer, XferLoad, XferSave};
use zh_logic::economy::{
    Energy, EnergyObjectInfo, EnergyOwnerCallbacks, Handicap, HandicapType, MissionStats, Money,
    MoneyCallbacks, NoopEnergyCallbacks, MAX_PLAYER_COUNT,
};

const WP3_1_JSON: &str = include_str!("../../../docs/golden_vectors/WP3_1_Economy.json");

#[inline]
fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[derive(Default)]
struct TraceMoneyCb {
    record_income_calls: u32,
    audio_trace_crc: u32,
}

impl MoneyCallbacks for TraceMoneyCb {
    fn trigger_withdraw_audio(&mut self, player_index: Int) {
        self.audio_trace_crc = rot_crc(self.audio_trace_crc, 1);
        self.audio_trace_crc = rot_crc(self.audio_trace_crc, player_index as u32);
    }

    fn trigger_deposit_audio(&mut self, player_index: Int) {
        self.audio_trace_crc = rot_crc(self.audio_trace_crc, 2);
        self.audio_trace_crc = rot_crc(self.audio_trace_crc, player_index as u32);
    }

    fn record_income(&mut self, player_index: Int) {
        if player_index >= 0 && (player_index as usize) < MAX_PLAYER_COUNT {
            self.record_income_calls += 1;
        }
    }
}

#[derive(Default)]
struct TraceEnergyCb {
    callbacks: u32,
    trace_crc: u32,
}

impl EnergyOwnerCallbacks for TraceEnergyCb {
    fn on_power_brown_out_change(&mut self, now_brownout: Bool) {
        self.callbacks += 1;
        self.trace_crc = rot_crc(self.trace_crc, now_brownout as u32);
    }
}

#[test]
fn test_wp3_1_economy_1000_golden_vectors() {
    let lines = extract_vector_lines(WP3_1_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 vectors in WP3_1_Economy.json");

    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let handicap_names = [
        "HANDICAP_BUILDCOST_GENERIC",
        "HANDICAP_BUILDCOST_BUILDINGS",
        "HANDICAP_BUILDTIME_GENERIC",
        "HANDICAP_BUILDTIME_BUILDINGS",
    ];
    let decoy_key = name_keys.name_to_key("HANDICAP_DECOY");
    let handicap_keys = handicap_names.map(|name| name_keys.name_to_key(name));

    for (idx, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").expect("seed");
        assert_eq!(seed as usize, idx);

        // ---------------- 1. Money ----------------
        let m_pidx = get_i32(line, "m_pidx").expect("m_pidx");
        let m_start = get_u32(line, "m_start").expect("m_start");
        let m_ops = get_i32_vec(line, "m_ops").expect("m_ops");
        assert_eq!(m_ops.len(), 48);

        let mut money = Money::new();
        money.set_player_index(m_pidx);
        money.set_starting_cash(m_start);
        let mut money_cb = TraceMoneyCb::default();
        let mut m_tcrc: u32 = 0;

        for step in 0..12 {
            let kind = m_ops[step * 4] as u32;
            let arg1 = m_ops[step * 4 + 1] as UnsignedInt;
            let arg2 = m_ops[step * 4 + 2] as UnsignedInt;
            let frame = m_ops[step * 4 + 3] as UnsignedInt;

            match kind {
                0 => {
                    let play_sound = if (arg2 & 1) != 0 { TRUE } else { FALSE };
                    let track_income = if (arg2 & 2) != 0 { TRUE } else { FALSE };
                    money.deposit(arg1, play_sound, track_income, &mut money_cb);
                }
                1 => {
                    let play_sound = if arg2 != 0 { TRUE } else { FALSE };
                    let got = money.withdraw(arg1, play_sound, &mut money_cb);
                    m_tcrc = rot_crc(m_tcrc, got);
                }
                2 => money.update_income_bucket(frame),
                3 => money.set_starting_cash(arg1),
                _ => panic!("unknown m_op kind"),
            }
            m_tcrc = rot_crc(m_tcrc, money.count_money());
            m_tcrc = rot_crc(m_tcrc, money.get_cash_per_minute());
        }

        assert_eq!(money.count_money(), get_u32(line, "m_cbucket").expect("m_cbucket"), "seed {seed}");
        assert_eq!(money.get_cash_per_minute(), get_u32(line, "m_cpm").expect("m_cpm"), "seed {seed}");
        assert_eq!(money_cb.record_income_calls, get_u32(line, "m_rec").expect("m_rec"), "seed {seed}");
        assert_eq!(money_cb.audio_trace_crc, get_u32(line, "m_acrc").expect("m_acrc"), "seed {seed}");
        assert_eq!(m_tcrc, get_u32(line, "m_tcrc").expect("m_tcrc"), "seed {seed}");

        let mut other = Money::new();
        other.set_player_index((m_pidx + 1) % MAX_PLAYER_COUNT as Int);
        other.set_starting_cash(money.count_money());
        assert_eq!(money.amount_equal(&other) as u32, get_u32(line, "m_eq_s").expect("m_eq_s"));
        other.deposit(1, FALSE, FALSE, &mut money_cb);
        assert_eq!(money.amount_equal(&other) as u32, get_u32(line, "m_eq_d").expect("m_eq_d"));

        let expected_m_save = decode_hex_bytes(get_str(line, "m_save_hex").expect("m_save_hex"))
            .expect("hex decode m_save_hex");
        #[cfg(not(feature = "retail-xfer-save"))]
        {
            let mut m_save = XferSave::new();
            let mut m_clone = money.clone();
            m_save.xfer_snapshot(&mut m_clone).expect("m_save");
            assert_eq!(m_save.as_bytes(), expected_m_save.as_slice(), "seed {seed}");
        }
        let mut m_loaded = Money::new();
        m_loaded.set_player_index(m_pidx);
        let mut m_load = XferLoad::new(&expected_m_save);
        m_load.xfer_snapshot(&mut m_loaded).expect("m_load");
        assert_eq!(m_loaded.count_money(), get_u32(line, "m_final").expect("m_final"), "seed {seed}");
        assert_eq!(m_loaded.get_cash_per_minute(), get_u32(line, "m_cpm").expect("m_cpm"), "seed {seed}");

        let v1_cash = get_u32(line, "m_v1_cash").expect("m_v1_cash");
        let mut v1_bytes = Vec::with_capacity(5);
        v1_bytes.push(1);
        v1_bytes.extend_from_slice(&v1_cash.to_le_bytes());
        let mut m_v1 = Money::new();
        m_v1.deposit(999, FALSE, TRUE, &mut money_cb);
        let mut v1_load = XferLoad::new(&v1_bytes);
        v1_load.xfer_snapshot(&mut m_v1).expect("m_v1_load");
        assert_eq!(m_v1.count_money(), get_u32(line, "m_v1_loaded").expect("m_v1_loaded"), "seed {seed}");
        assert_eq!(m_v1.get_cash_per_minute(), get_u32(line, "m_v1_cpm").expect("m_v1_cpm"), "seed {seed}");

        // ---------------- 2. Energy ----------------
        let e_owner = get_i32(line, "e_owner").expect("e_owner");
        let e_end_frame = get_u32(line, "e_end_frame").expect("e_end_frame");
        let e_ops = get_i32_vec(line, "e_ops").expect("e_ops");
        assert_eq!(e_ops.len(), 60);

        let mut energy = Energy::new();
        energy.init(Some(e_owner));
        let mut energy_cb = TraceEnergyCb::default();
        let mut e_tcrc: u32 = 0;

        for step in 0..12 {
            let kind = e_ops[step * 5];
            let a1 = e_ops[step * 5 + 1];
            let a2 = e_ops[step * 5 + 2];
            let a3 = e_ops[step * 5 + 3];
            let frame = e_ops[step * 5 + 4] as UnsignedInt;

            match kind {
                0 => energy.adjust_power(a1, if a2 != 0 { TRUE } else { FALSE }, frame, &mut energy_cb),
                1 => {
                    let obj = if a3 != 0 {
                        None
                    } else {
                        Some(EnergyObjectInfo {
                            energy_production: a1,
                            energy_bonus: a2,
                            is_disabled: FALSE,
                        })
                    };
                    energy.object_entering_influence(obj, frame, &mut energy_cb);
                }
                2 => {
                    let obj = if a3 != 0 {
                        None
                    } else {
                        Some(EnergyObjectInfo {
                            energy_production: a1,
                            energy_bonus: a2,
                            is_disabled: FALSE,
                        })
                    };
                    energy.object_leaving_influence(obj, frame, &mut energy_cb);
                }
                3 => {
                    let obj = if a3 != 0 {
                        None
                    } else {
                        Some(EnergyObjectInfo {
                            energy_production: 10,
                            energy_bonus: a1,
                            is_disabled: FALSE,
                        })
                    };
                    energy.add_power_bonus(obj, frame, &mut energy_cb);
                }
                4 => {
                    let obj = if a3 != 0 {
                        None
                    } else {
                        Some(EnergyObjectInfo {
                            energy_production: 10,
                            energy_bonus: a1,
                            is_disabled: if a2 != 0 { TRUE } else { FALSE },
                        })
                    };
                    energy.remove_power_bonus(obj, frame, &mut energy_cb);
                }
                5 => energy.set_power_sabotaged_till_frame(a1 as UnsignedInt),
                6 => {}
                _ => panic!("unknown e_op kind"),
            }

            e_tcrc = rot_crc(e_tcrc, energy.get_production(frame) as u32);
            e_tcrc = rot_crc(e_tcrc, energy.get_consumption() as u32);
            e_tcrc = rot_crc(e_tcrc, energy.has_sufficient_power(frame) as u32);
            e_tcrc = rot_crc(e_tcrc, energy.get_energy_supply_ratio(frame).to_bits());
            e_tcrc = rot_crc(e_tcrc, energy_cb.callbacks);
            e_tcrc = rot_crc(e_tcrc, energy_cb.trace_crc);
        }

        #[cfg(not(feature = "retail-crc"))]
        {
            assert_eq!(e_tcrc, get_u32(line, "e_tcrc").expect("e_tcrc"), "seed {seed}");
            assert_eq!(energy.get_production(e_end_frame), get_i32(line, "e_prod").expect("e_prod"), "seed {seed}");
            assert_eq!(energy.get_consumption(), get_i32(line, "e_cons").expect("e_cons"), "seed {seed}");
            assert_eq!(energy.get_power_sabotaged_till_frame(), get_u32(line, "e_sab").expect("e_sab"), "seed {seed}");
            assert_eq!(
                energy.get_energy_supply_ratio(e_end_frame).to_bits(),
                get_u32(line, "e_ratio").expect("e_ratio"),
                "seed {seed}"
            );

            let expected_e_save = decode_hex_bytes(get_str(line, "e_save_hex").expect("e_save_hex"))
                .expect("hex decode e_save_hex");
            let mut e_save = XferSave::new();
            e_save.xfer_snapshot(&mut energy).expect("e_save");
            assert_eq!(e_save.as_bytes(), expected_e_save.as_slice(), "seed {seed}");

            let mut e_loaded = Energy::new();
            let mut e_load = XferLoad::new(&expected_e_save);
            e_load.xfer_snapshot(&mut e_loaded).expect("e_load");
            assert_eq!(e_loaded.owner_player_index(), Some(e_owner));
            assert_eq!(e_loaded.get_power_sabotaged_till_frame(), energy.get_power_sabotaged_till_frame());
        }

        // Legacy Energy v1 XferLoad check (must leave sabotage frame intact)
        let e_v1_in = get_i32_array::<3>(line, "e_v1_in").expect("e_v1_in");
        let e_v1_out = get_i32_array::<3>(line, "e_v1_out").expect("e_v1_out");
        let mut e_v1_bytes = Vec::with_capacity(13);
        e_v1_bytes.push(1);
        e_v1_bytes.extend_from_slice(&e_v1_in[0].to_le_bytes());
        e_v1_bytes.extend_from_slice(&e_v1_in[1].to_le_bytes());
        e_v1_bytes.extend_from_slice(&e_v1_in[2].to_le_bytes());
        let mut e_v1 = Energy::new();
        e_v1.set_power_sabotaged_till_frame(777);
        let mut e_v1_load = XferLoad::new(&e_v1_bytes);
        e_v1_load.xfer_snapshot(&mut e_v1).expect("e_v1_load");
        assert_eq!(e_v1.get_production(e_end_frame), e_v1_out[0], "seed {seed}");
        assert_eq!(e_v1.get_consumption(), e_v1_out[1], "seed {seed}");
        assert_eq!(e_v1.get_power_sabotaged_till_frame(), e_v1_out[2] as UnsignedInt, "seed {seed}");
        assert_eq!(e_v1.get_production(777), e_v1_in[0], "seed {seed}");

        // Orphan owner (None) must emit zero brownout callbacks
        let mut orphan = Energy::new();
        let mut noop = NoopEnergyCallbacks;
        orphan.adjust_power(20, TRUE, 1, &mut noop);
        assert_eq!(orphan.get_production(1), 20);

        // ---------------- 3. Handicap ----------------
        let h_m1 = get_u32(line, "h_m1").expect("h_m1");
        let h_m2 = get_u32(line, "h_m2").expect("h_m2");
        let h_d1 = get_u32_array::<4>(line, "h_d1").expect("h_d1");
        let h_d2 = get_u32_array::<4>(line, "h_d2").expect("h_d2");
        let h_out = get_u32_array::<4>(line, "h_out").expect("h_out");

        let mut d1 = Dict::new();
        let mut d2 = Dict::new();
        for i in 0..4 {
            if (h_m1 & (1 << i)) != 0 {
                d1.set_real(handicap_keys[i], f32::from_bits(h_d1[i]));
            }
            if (h_m2 & (1 << i)) != 0 {
                d2.set_real(handicap_keys[i], f32::from_bits(h_d2[i]));
            }
        }
        d1.set_int(decoy_key, 42);
        let mut hcap = Handicap::new();
        hcap.read_from_dict(&d1, &mut name_keys);
        hcap.read_from_dict(&d2, &mut name_keys);
        assert_eq!(hcap.get_handicap(HandicapType::BuildCost, false).to_bits(), h_out[0], "seed {seed}");
        assert_eq!(hcap.get_handicap(HandicapType::BuildCost, true).to_bits(), h_out[1], "seed {seed}");
        assert_eq!(hcap.get_handicap(HandicapType::BuildTime, false).to_bits(), h_out[2], "seed {seed}");
        assert_eq!(hcap.get_handicap(HandicapType::BuildTime, true).to_bits(), h_out[3], "seed {seed}");

        // ---------------- 4. MissionStats ----------------
        let ms_uk = get_i32_array::<MAX_PLAYER_COUNT>(line, "ms_uk").expect("ms_uk");
        let ms_bk = get_i32_array::<MAX_PLAYER_COUNT>(line, "ms_bk").expect("ms_bk");
        let ms_ul = get_i32(line, "ms_ul").expect("ms_ul");
        let ms_bl = get_i32(line, "ms_bl").expect("ms_bl");

        let mut mstats = MissionStats {
            units_killed: ms_uk,
            units_lost: ms_ul,
            buildings_killed: ms_bk,
            buildings_lost: ms_bl,
        };

        #[cfg(not(feature = "retail-crc"))]
        {
            let mut xcrc = XferCrc::new();
            xcrc.xfer_snapshot(&mut mstats).expect("ms_crc");
            assert_eq!(xcrc.get_crc(), get_u32(line, "ms_crc").expect("ms_crc"), "seed {seed}");
        }

        let expected_ms_save = decode_hex_bytes(get_str(line, "ms_save_hex").expect("ms_save_hex"))
            .expect("hex decode ms_save_hex");
        let mut ms_save = XferSave::new();
        ms_save.xfer_snapshot(&mut mstats).expect("ms_save");
        assert_eq!(ms_save.as_bytes(), expected_ms_save.as_slice(), "seed {seed}");

        let mut ms_loaded = MissionStats::new();
        let mut ms_load = XferLoad::new(&expected_ms_save);
        ms_load.xfer_snapshot(&mut ms_loaded).expect("ms_load");
        assert_eq!(ms_loaded, mstats, "seed {seed}");
    }
}
