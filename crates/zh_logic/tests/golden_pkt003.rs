//! Gate V2-Golden Test Oracle for `PKT-003` (`ExperienceTracker`).
//! Verifies 1,000 C++-recorded Golden Test Vectors in `docs/golden_vectors/PKT-003.json`.

use zh_common::golden_json::{
    encode_hex_bytes, extract_vector_lines, get_hex_u32, get_i32, get_i32_array, get_str,
};
use zh_common::types::{Bool, Int, ObjectId, Relationship, VeterancyLevel, FALSE, TRUE};
use zh_common::xfer::{Snapshot, XferCrc, XferLoad, XferSave};
use zh_logic::object::experience_tracker::{ExperienceParentAccess, ExperienceTrackerState};

const PKT003_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-003.json");

struct MockSink {
    alive: bool,
    thresholds: [Int; 4],
    exp_values: [Int; 4],
    tracker: ExperienceTrackerState,
}

impl MockSink {
    fn new(alive: bool, thresholds: [Int; 4], exp_values: [Int; 4]) -> Self {
        Self {
            alive,
            thresholds,
            exp_values,
            tracker: ExperienceTrackerState {
                current_level: VeterancyLevel::Regular,
                current_experience: 0,
                experience_sink: ObjectId::INVALID,
                experience_scalar: 1.0,
                is_trainable: TRUE,
            },
        }
    }
}

struct MockSinkParent {
    thresholds: [Int; 4],
    exp_values: [Int; 4],
}

impl ExperienceParentAccess for MockSinkParent {
    fn is_template_trainable(&self) -> Bool {
        TRUE
    }
    fn get_experience_required(&self, level: Int) -> Int {
        if (0..4).contains(&level) {
            self.thresholds[level as usize]
        } else {
            0
        }
    }
    fn get_experience_value(&self, level: Int) -> Int {
        if (0..4).contains(&level) {
            self.exp_values[level as usize]
        } else {
            0
        }
    }
    fn get_relationship_with_killer(&self) -> Relationship {
        Relationship::Enemies
    }
    fn on_veterancy_level_changed(
        &mut self,
        _old_level: VeterancyLevel,
        _new_level: VeterancyLevel,
        _provide_feedback: Bool,
    ) {
    }
    fn forward_experience_to_sink(
        &mut self,
        _sink_id: ObjectId,
        _scaled_exp_gain: Int,
        _can_scale_for_bonus: Bool,
    ) -> bool {
        false
    }
    fn forward_set_exp_and_level_to_sink(
        &mut self,
        _sink_id: ObjectId,
        _experience_in: Int,
        _provide_feedback: Bool,
    ) -> bool {
        false
    }
}

struct MockExperienceParent {
    trainable: Bool,
    thresholds: [Int; 4],
    exp_values: [Int; 4],
    killer_rel: Relationship,
    cb_fired: Bool,
    cb_old_level: VeterancyLevel,
    cb_new_level: VeterancyLevel,
    cb_feedback: Bool,
    sink: MockSink,
}

impl ExperienceParentAccess for MockExperienceParent {
    fn is_template_trainable(&self) -> Bool {
        self.trainable
    }

    fn get_experience_required(&self, level: Int) -> Int {
        if (0..4).contains(&level) {
            self.thresholds[level as usize]
        } else {
            0
        }
    }

    fn get_experience_value(&self, level: Int) -> Int {
        if (0..4).contains(&level) {
            self.exp_values[level as usize]
        } else {
            0
        }
    }

    fn get_relationship_with_killer(&self) -> Relationship {
        self.killer_rel
    }

    fn on_veterancy_level_changed(
        &mut self,
        old_level: VeterancyLevel,
        new_level: VeterancyLevel,
        provide_feedback: Bool,
    ) {
        self.cb_fired = TRUE;
        self.cb_old_level = old_level;
        self.cb_new_level = new_level;
        self.cb_feedback = provide_feedback;
    }

    fn forward_experience_to_sink(
        &mut self,
        _sink_id: ObjectId,
        scaled_exp_gain: Int,
        can_scale_for_bonus: Bool,
    ) -> bool {
        if !self.sink.alive {
            return false;
        }
        let mut sink_parent = MockSinkParent {
            thresholds: self.sink.thresholds,
            exp_values: self.sink.exp_values,
        };
        self.sink.tracker.add_experience_points(
            &mut sink_parent,
            scaled_exp_gain,
            can_scale_for_bonus,
        );
        true
    }

    fn forward_set_exp_and_level_to_sink(
        &mut self,
        _sink_id: ObjectId,
        experience_in: Int,
        provide_feedback: Bool,
    ) -> bool {
        if !self.sink.alive {
            return false;
        }
        let mut sink_parent = MockSinkParent {
            thresholds: self.sink.thresholds,
            exp_values: self.sink.exp_values,
        };
        self.sink.tracker.set_experience_and_level(
            &mut sink_parent,
            experience_in,
            provide_feedback,
        );
        true
    }
}

#[test]
fn gate_v2_golden_pkt003_experience_tracker_1000_vectors() {
    let lines = extract_vector_lines(PKT003_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-003.json");

    for line in lines {
        let seed = get_i32(line, "seed").expect("seed");
        let thresholds = get_i32_array::<4>(line, "thresholds").expect("thresholds");
        let exp_values = get_i32_array::<4>(line, "exp_values").expect("exp_values");
        let killer_rel = Relationship::from_i32(get_i32(line, "killer_rel").expect("killer_rel"));
        let init_level = VeterancyLevel::from_i32(get_i32(line, "init_level").expect("init_level"));
        let init_exp = get_i32(line, "init_exp").expect("init_exp");
        let init_sink = ObjectId(get_i32(line, "init_sink").expect("init_sink"));
        let sink_alive = get_i32(line, "sink_alive").expect("sink_alive") != 0;
        let scalar = f32::from_bits(get_hex_u32(line, "scalar_bits").expect("scalar_bits"));
        let init_trainable = get_i32(line, "init_trainable").expect("init_trainable") as u8;
        let op_kind = get_i32(line, "op_kind").expect("op_kind");
        let arg1 = get_i32(line, "arg1").expect("arg1");
        let arg_bool = get_i32(line, "arg_bool").expect("arg_bool") as u8;
        let expected_ret_i32 = get_i32(line, "ret_i32").expect("ret_i32");

        let expected_post_level = get_i32(line, "post_level").expect("post_level");
        let expected_post_exp = get_i32(line, "post_exp").expect("post_exp");
        let expected_post_trainable = get_i32(line, "post_trainable").expect("post_trainable") as u8;
        let expected_cb_fired = get_i32(line, "cb_fired").expect("cb_fired") as u8;
        let expected_cb_old_level = get_i32(line, "cb_old_level").expect("cb_old_level");
        let expected_cb_new_level = get_i32(line, "cb_new_level").expect("cb_new_level");
        let expected_cb_feedback = get_i32(line, "cb_feedback").expect("cb_feedback") as u8;
        let expected_sink_exp = get_i32(line, "sink_post_exp").expect("sink_post_exp");
        let expected_sink_level = get_i32(line, "sink_post_level").expect("sink_post_level");
        let expected_xfer_crc = get_hex_u32(line, "expected_xfer_crc").expect("expected_xfer_crc");
        let expected_xfer_hex = get_str(line, "expected_xfer_hex").expect("expected_xfer_hex");

        let mut parent = MockExperienceParent {
            trainable: init_trainable,
            thresholds,
            exp_values,
            killer_rel,
            cb_fired: FALSE,
            cb_old_level: VeterancyLevel::Regular,
            cb_new_level: VeterancyLevel::Regular,
            cb_feedback: FALSE,
            sink: MockSink::new(sink_alive, thresholds, exp_values),
        };

        let mut tracker = ExperienceTrackerState {
            current_level: init_level,
            current_experience: init_exp,
            experience_sink: init_sink,
            experience_scalar: scalar,
            is_trainable: init_trainable,
        };

        let mut ret_i32: i32 = 0;
        match op_kind {
            0 => tracker.add_experience_points(&mut parent, arg1, arg_bool),
            1 => tracker.set_min_veterancy_level(
                &mut parent,
                VeterancyLevel::from_i32(arg1),
                arg_bool,
            ),
            2 => tracker.set_veterancy_level(
                &mut parent,
                VeterancyLevel::from_i32(arg1),
                arg_bool,
            ),
            3 => {
                let can_gain = tracker.can_gain_exp_for_level(arg1);
                let did_gain = tracker.gain_exp_for_level(&mut parent, arg1, arg_bool);
                ret_i32 = ((can_gain as i32) << 8) | (did_gain as i32);
            }
            4 => tracker.set_experience_and_level(&mut parent, arg1, arg_bool),
            5 => {
                let val = tracker.get_experience_value(&parent);
                let acc = tracker.is_accepting_experience_points();
                ret_i32 = (val << 8) | (acc as i32);
            }
            _ => unreachable!(),
        }

        assert_eq!(ret_i32, expected_ret_i32, "PKT-003 ret_i32 mismatch at seed {}", seed);
        assert_eq!(
            tracker.get_veterancy_level().as_i32(),
            expected_post_level,
            "PKT-003 post_level mismatch at seed {}",
            seed
        );
        assert_eq!(
            tracker.get_current_experience(),
            expected_post_exp,
            "PKT-003 post_exp mismatch at seed {}",
            seed
        );
        assert_eq!(
            tracker.is_trainable(),
            expected_post_trainable,
            "PKT-003 post_trainable mismatch at seed {}",
            seed
        );
        assert_eq!(parent.cb_fired, expected_cb_fired, "PKT-003 cb_fired mismatch at seed {}", seed);
        assert_eq!(
            parent.cb_old_level.as_i32(),
            expected_cb_old_level,
            "PKT-003 cb_old_level mismatch at seed {}",
            seed
        );
        assert_eq!(
            parent.cb_new_level.as_i32(),
            expected_cb_new_level,
            "PKT-003 cb_new_level mismatch at seed {}",
            seed
        );
        assert_eq!(
            parent.cb_feedback, expected_cb_feedback,
            "PKT-003 cb_feedback mismatch at seed {}",
            seed
        );
        assert_eq!(
            parent.sink.tracker.get_current_experience(),
            expected_sink_exp,
            "PKT-003 sink_post_exp mismatch at seed {}",
            seed
        );
        assert_eq!(
            parent.sink.tracker.get_veterancy_level().as_i32(),
            expected_sink_level,
            "PKT-003 sink_post_level mismatch at seed {}",
            seed
        );

        let mut xfer_crc = XferCrc::new();
        tracker.crc(&mut xfer_crc);
        assert_eq!(
            xfer_crc.get_crc(),
            expected_xfer_crc,
            "PKT-003 XferCRC mismatch at seed {}: got 0x{:08X}, expected 0x{:08X}",
            seed,
            xfer_crc.get_crc(),
            expected_xfer_crc
        );

        let mut xfer_save = XferSave::new();
        tracker.xfer(&mut xfer_save).expect("xfer save");
        let actual_hex = encode_hex_bytes(xfer_save.as_bytes());
        assert_eq!(
            actual_hex, expected_xfer_hex,
            "PKT-003 XferSave bytes mismatch at seed {}",
            seed
        );

        // Verify XferLoad round-trip
        let mut loaded = ExperienceTrackerState::default();
        let mut xfer_load = XferLoad::new(xfer_save.as_bytes());
        loaded.xfer(&mut xfer_load).expect("xfer load");
        assert_eq!(loaded, tracker, "PKT-003 XferLoad round-trip mismatch at seed {}", seed);
    }
}
