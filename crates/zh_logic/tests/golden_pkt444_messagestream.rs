//! Gate V2 Golden Vector oracle & verification suite for `PKT-444` (`MessageStream`).

use std::any::Any;
use zh_common::golden_json::{
    extract_vector_lines, get_hex_u32, get_i32, get_i32_vec, get_str, get_u32, get_u32_array,
};
use zh_common::math::{Coord3D, ICoord2D};
use zh_common::types::{Bool, DrawableId, Int, ObjectId, Real, FALSE, TRUE};
use zh_logic::gamelogic::message_stream::{
    build_region, CommandList, GameMessage, GameMessageArgumentDataType, GameMessageDisposition,
    GameMessageTranslator, GameMessageType, MessageId, MessageStream, MSG_COUNT_VALUE,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-444.json");

fn parse_enum_table(json: &str) -> Vec<(Int, &str)> {
    let mut out = Vec::new();
    let mut in_table = false;
    for line in json.lines() {
        let t = line.trim();
        if t.starts_with("\"enum_table\"") {
            in_table = true;
            continue;
        }
        if in_table {
            if t.starts_with(']') {
                break;
            }
            let inner = t.trim_end_matches(',').trim();
            if inner.starts_with('[') && inner.ends_with(']') {
                let body = &inner[1..inner.len() - 1];
                let comma = body.find(',').unwrap();
                let ord: Int = body[..comma].trim().parse().unwrap();
                let q1 = body.find('"').unwrap();
                let q2 = body.rfind('"').unwrap();
                out.push((ord, &body[q1 + 1..q2]));
            }
        }
    }
    out
}

#[test]
fn test_pkt444_oracle_structure_and_enum_discriminants() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"msg_begin_network\": 1000"));
    assert!(VECTORS_JSON.contains("\"msg_end_network\": 1999"));
    assert!(VECTORS_JSON.contains("\"msg_count\": 2006"));
    assert_eq!(GameMessageType::MsgBeginNetworkMessages.as_i32(), 1000);
    assert_eq!(GameMessageType::MsgBeginDebugNetworkMessages.as_i32(), 1900);
    assert_eq!(GameMessageType::MsgEndNetworkMessages.as_i32(), 1999);
    assert_eq!(GameMessageType::MsgCount.as_i32(), MSG_COUNT_VALUE);

    let table = parse_enum_table(VECTORS_JSON);
    assert!(!table.is_empty());
    let mut named_count = 0usize;
    for (ord, name) in table {
        if name != "UnknownMessage" && name != "Invalid command" {
            let parsed = GameMessageType::from_i32(ord)
                .unwrap_or_else(|| panic!("missing enum discriminant {ord} ({name})"));
            assert_eq!(parsed.as_i32(), ord);
            named_count += 1;
        } else {
            assert!(GameMessageType::from_i32(ord).is_none() || ord == MSG_COUNT_VALUE);
        }
    }
    assert_eq!(named_count, 288);

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[allow(dead_code)]
struct RecordingTranslator {
    tag: Int,
    destroy_type: GameMessageType,
    trace: Vec<Int>,
}

impl GameMessageTranslator for RecordingTranslator {
    fn translate_game_message(&mut self, msg: &GameMessage) -> GameMessageDisposition {
        self.trace.push(self.tag);
        self.trace.push(msg.message_type().as_i32());
        if msg.message_type() == self.destroy_type {
            GameMessageDisposition::DestroyMessage
        } else {
            GameMessageDisposition::KeepMessage
        }
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }
}

#[test]
#[ignore = "PKT-444: remove when MessageStream is implemented"]
fn test_pkt444_enum_strings_and_fallbacks() {
    for (ord, expected_name) in parse_enum_table(VECTORS_JSON) {
        assert_eq!(
            GameMessage::get_command_type_as_string_raw(ord),
            expected_name,
            "enum string mismatch at ordinal {ord}"
        );
    }
    assert_eq!(GameMessage::get_command_type_as_string_raw(-1), "UnknownMessage");
    assert_eq!(GameMessage::get_command_type_as_string_raw(999), "UnknownMessage");
    assert_eq!(
        GameMessage::get_command_type_as_string_raw(MSG_COUNT_VALUE + 42),
        "Invalid command"
    );
}

#[test]
#[ignore = "PKT-444: remove when MessageStream is implemented"]
fn test_pkt444_arguments_and_stream_1000_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let player_idx = get_i32(line, "player_idx").unwrap();
        let query_type = GameMessageType::from_i32(get_i32(line, "query_type").unwrap()).unwrap();
        let cmd_str = get_str(line, "cmd_str").unwrap();

        let arg_i = get_i32(line, "arg_i").unwrap();
        let arg_r = Real::from_bits(get_hex_u32(line, "arg_r_bits").unwrap());
        let arg_b: Bool = if get_u32(line, "arg_b").unwrap() != 0 { TRUE } else { FALSE };
        let arg_obj = ObjectId(get_i32(line, "arg_obj").unwrap());
        let arg_draw = DrawableId(get_i32(line, "arg_draw").unwrap());
        let arg_team = get_u32(line, "arg_team").unwrap();
        let loc = Coord3D {
            x: Real::from_bits(get_hex_u32(line, "loc_x_bits").unwrap()),
            y: Real::from_bits(get_hex_u32(line, "loc_y_bits").unwrap()),
            z: Real::from_bits(get_hex_u32(line, "loc_z_bits").unwrap()),
        };
        let anchor = ICoord2D {
            x: get_i32(line, "ax").unwrap(),
            y: get_i32(line, "ay").unwrap(),
        };
        let dest = ICoord2D {
            x: get_i32(line, "dx").unwrap(),
            y: get_i32(line, "dy").unwrap(),
        };
        let reg = build_region(anchor, dest);
        assert_eq!(reg.lo.x, get_i32(line, "rlo_x").unwrap());
        assert_eq!(reg.lo.y, get_i32(line, "rlo_y").unwrap());
        assert_eq!(reg.hi.x, get_i32(line, "rhi_x").unwrap());
        assert_eq!(reg.hi.y, get_i32(line, "rhi_y").unwrap());

        let arg_ts = get_u32(line, "arg_ts").unwrap();
        let arg_wc = get_u32(line, "arg_wc").unwrap() as u16;

        let mut msg = GameMessage::new(query_type, player_idx);
        assert_eq!(msg.get_command_as_string(), cmd_str);
        assert_eq!(msg.player_index(), player_idx);
        assert_eq!(msg.get_argument_data_type(0), GameMessageArgumentDataType::Unknown);
        assert_eq!(msg.get_argument(0).integer, 0);
        assert_eq!(msg.get_argument(-1).timestamp, 0);

        msg.append_integer_argument(arg_i);
        msg.append_real_argument(arg_r);
        msg.append_boolean_argument(arg_b);
        msg.append_object_id_argument(arg_obj);
        msg.append_drawable_id_argument(arg_draw);
        msg.append_team_id_argument(arg_team);
        msg.append_location_argument(loc);
        msg.append_pixel_argument(anchor);
        msg.append_pixel_region_argument(reg);
        msg.append_timestamp_argument(arg_ts);
        msg.append_wide_char_argument(arg_wc);
        assert_eq!(msg.get_argument_count(), 11, "seed {seed}: arg count");
        assert_eq!(msg.get_argument_data_type(0), GameMessageArgumentDataType::Integer);
        assert_eq!(msg.get_argument(0).integer, arg_i);
        assert_eq!(msg.get_argument(1).real.to_bits(), arg_r.to_bits());
        assert_eq!(msg.get_argument(2).boolean, arg_b);
        assert_eq!(msg.get_argument(3).object_id, arg_obj);
        assert_eq!(msg.get_argument(4).drawable_id, arg_draw);
        assert_eq!(msg.get_argument(5).team_id, arg_team);
        assert_eq!(msg.get_argument(6).location.x.to_bits(), loc.x.to_bits());
        assert_eq!(msg.get_argument(6).location.y.to_bits(), loc.y.to_bits());
        assert_eq!(msg.get_argument(6).location.z.to_bits(), loc.z.to_bits());
        assert_eq!(msg.get_argument(7).pixel, anchor);
        assert_eq!(msg.get_argument(8).pixel_region, reg);
        assert_eq!(msg.get_argument(9).timestamp, arg_ts);
        assert_eq!(msg.get_argument(10).wide_char, arg_wc);
        assert_eq!(msg.get_argument_data_type(11), GameMessageArgumentDataType::Unknown);
        assert_eq!(msg.get_argument(11).integer, 0);

        let prios: [u32; 4] = get_u32_array(line, "prios").unwrap();
        let remove_idx = get_i32(line, "remove_idx").unwrap();
        let insert2 = get_u32(line, "insert2").unwrap() != 0;
        let contains_cam = get_u32(line, "contains_cam").unwrap() != 0;
        let seq_types = get_i32_vec(line, "seq_types").unwrap();
        let seq_bool_mode = get_i32_vec(line, "seq_bool_mode").unwrap();
        let expected_trace = get_i32_vec(line, "trace").unwrap();
        let expected_survivors = get_i32_vec(line, "survivors").unwrap();

        let mut stream = MessageStream::new();
        let mut cmd_list = CommandList::new();
        stream.init();
        cmd_list.init();

        let drop0 = if seed % 2 == 0 {
            GameMessageType::MsgRawMousePosition
        } else {
            GameMessageType::MsgFrameTick
        };
        let tids = [
            stream.attach_translator(
                Box::new(RecordingTranslator {
                    tag: 10,
                    destroy_type: drop0,
                    trace: Vec::new(),
                }),
                prios[0],
            ),
            stream.attach_translator(
                Box::new(RecordingTranslator {
                    tag: 11,
                    destroy_type: GameMessageType::MsgInvalid,
                    trace: Vec::new(),
                }),
                prios[1],
            ),
            stream.attach_translator(
                Box::new(RecordingTranslator {
                    tag: 12,
                    destroy_type: GameMessageType::MsgMouseLeftClick,
                    trace: Vec::new(),
                }),
                prios[2],
            ),
            stream.attach_translator(
                Box::new(RecordingTranslator {
                    tag: 13,
                    destroy_type: GameMessageType::MsgRawMousePosition,
                    trace: Vec::new(),
                }),
                prios[3],
            ),
        ];
        if remove_idx >= 0 {
            stream.remove_translator(tids[remove_idx as usize]);
        }

        let mut first_id: Option<MessageId> = None;
        for i in 0..seq_types.len() {
            let t = GameMessageType::from_i32(seq_types[i]).unwrap();
            let id = if insert2 && i == 2 {
                stream.insert_message_after(t, player_idx, first_id.unwrap()).unwrap()
            } else {
                stream.append_message(t, player_idx)
            };
            if seq_bool_mode[i] >= 0 {
                stream
                    .get_message_mut(id)
                    .unwrap()
                    .append_boolean_argument(if seq_bool_mode[i] > 0 { TRUE } else { FALSE });
            }
            if i == 0 {
                first_id = Some(id);
            }
        }

        assert_eq!(
            stream.contains_message_of_type(GameMessageType::MsgSetReplayCamera),
            contains_cam,
            "seed {seed}: contains_message_of_type mismatch"
        );
        stream.propagate_messages(&mut cmd_list);

        let mut actual_trace = Vec::new();
        for tid in stream.translator_ids() {
            let rec = stream
                .find_translator(tid)
                .unwrap()
                .as_any()
                .unwrap()
                .downcast_ref::<RecordingTranslator>()
                .unwrap();
            actual_trace.extend_from_slice(&rec.trace);
        }
        let actual_survivors: Vec<Int> = cmd_list
            .messages()
            .iter()
            .map(|m| m.message_type().as_i32())
            .collect();
        assert_eq!(
            actual_trace,
            expected_trace,
            "seed {seed}: translator call trace mismatch"
        );
        assert_eq!(
            actual_survivors, expected_survivors,
            "seed {seed}: surviving CommandList mismatch"
        );
    }
}
