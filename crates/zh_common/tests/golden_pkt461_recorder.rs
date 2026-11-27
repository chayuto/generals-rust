//! `PKT-461`: replay files and options strings against the reference `RecorderClass`
//! (`tests/cpp_harness/gen_pkt461_recorder.cpp`).
//!
//! `local_replays_match_the_reference` runs only where the local replays exist; it
//! compares counts and digests, never their content.

use std::collections::BTreeMap;
use std::path::PathBuf;

use zh_common::golden_json::{decode_hex_bytes, encode_hex_bytes, get_str, get_u32};
use zh_common::io::recorder::*;
use zh_common::runtime::game_logic_dispatch::is_routed;

const VECTORS: &str = include_str!("../../../docs/golden_vectors/PKT-461.json");
const LOCAL: &str = include_str!("../../../docs/golden_vectors/PKT-461_local.json");

/// The host answers the oracle's game gives.
struct Rules {
    colors: i32,
    templates: i32,
}

fn leaf_and_dir(path: &[u8]) -> &[u8] {
    let sep = |b: &u8| *b == b'\\' || *b == b'/';
    match path.iter().rposition(sep) {
        Some(p) => match path[..p].iter().rposition(sep) {
            Some(q) => &path[q + 1..],
            None => path,
        },
        None => path,
    }
}

impl ReplayRules for Rules {
    fn num_colors(&self) -> i32 {
        self.colors
    }
    fn player_template_count(&self) -> i32 {
        self.templates
    }
    fn default_starting_cash(&self) -> u32 {
        10000
    }
    fn real_map_path(&self, portable: &[u8]) -> Option<Vec<u8>> {
        let lower = portable.to_ascii_lowercase();
        let (prefix, tail): (&[u8], &[u8]) = if lower.starts_with(b"save\\") {
            let leaf = portable
                .iter()
                .rposition(|&b| b == b'\\' || b == b'/')
                .map_or(portable, |p| &portable[p + 1..]);
            (b"save\\", leaf)
        } else if lower.starts_with(b"maps\\") {
            (b"maps\\", leaf_and_dir(portable))
        } else if lower.starts_with(b"userdata\\maps\\") {
            (b"userdata\\maps\\", leaf_and_dir(portable))
        } else {
            return None;
        };
        // `isPathInDirectory`.
        if tail.split(|&b| b == b'\\' || b == b'/').any(|c| c == b"..") {
            return None;
        }
        Some([prefix, tail].concat())
    }
    fn portable_map_path(&self, real: &[u8]) -> Vec<u8> {
        real.iter()
            .map(|&b| {
                if b == b'/' {
                    b'\\'
                } else {
                    b.to_ascii_lowercase()
                }
            })
            .collect()
    }
}

fn rules(json: &str) -> Rules {
    let line = |key: &str| {
        json.lines()
            .find(|l| l.contains(key))
            .and_then(|l| get_u32(l, key.trim_matches(['"', ':'])))
            .expect("rule value") as i32
    };
    Rules {
        colors: line("\"colors\":"),
        templates: line("\"templates\":"),
    }
}

fn section<'a>(json: &'a str, name: &str) -> Vec<&'a str> {
    let open = format!("\"{name}\":[");
    json.lines()
        .skip_while(|l| *l != open)
        .skip(1)
        .take_while(|l| l.starts_with('{'))
        .map(|l| l.trim_end_matches(','))
        .collect()
}

/// The strings of a `"key":["..",".."]` array that ends the row.
fn strings<'a>(row: &'a str, key: &str) -> Vec<&'a str> {
    let open = format!("\"{key}\":[");
    let start = row.find(&open).expect("array") + open.len();
    let body = &row[start..row.rfind(']').expect("array end")];
    body.split('"').skip(1).step_by(2).collect()
}

fn bytes(row: &str, key: &str) -> Vec<u8> {
    decode_hex_bytes(get_str(row, key).expect(key)).expect(key)
}

fn wide(s: &[u32]) -> String {
    s.iter().map(u32::to_string).collect::<Vec<_>>().join(".")
}

fn info_dump(o: &GameOptions, rules: &Rules) -> String {
    let mut out = format!(
        "{} {} {} {} {} {} {} {} {} {};",
        o.use_stats,
        o.map_contents_mask,
        encode_hex_bytes(&rules.portable_map_path(&o.map)),
        o.map_crc,
        o.map_size,
        o.seed,
        o.crc_interval,
        o.superweapon_restriction,
        o.starting_cash,
        u8::from(o.old_factions_only),
    );
    for s in &o.slots {
        let name = if s.is_human() {
            wide(&s.name)
        } else {
            String::new()
        };
        out += &format!(
            "{},{},{},{},{},{},{},{},{},{},{}|",
            s.state,
            u8::from(s.accepted),
            u8::from(s.has_map),
            s.color,
            s.player_template,
            s.start_pos,
            s.team_number,
            s.nat_behavior,
            name,
            s.ip,
            s.port,
        );
    }
    out
}

fn command_dump(m: &ReplayCommand) -> String {
    let mut out = format!("{} {} {}", m.msg_type, m.player_index, m.argument_count());
    for a in m.args.iter().take(usize::from(m.argument_count())) {
        out += &format!(" {}:{}", a.kind, encode_hex_bytes(a.payload()));
    }
    out
}

fn parse_command(text: &str) -> (u32, ReplayCommand) {
    let mut f = text.split(' ');
    let mut number = || f.next().unwrap().parse::<i64>().unwrap();
    let (frame, kind, player, _count) = (number(), number(), number(), number());
    let mut msg = ReplayCommand::new(kind as i32, player as i32);
    for arg in f {
        let (k, hex) = arg.split_once(':').unwrap();
        msg.args.push(ReplayArg::new(
            k.parse().unwrap(),
            &decode_hex_bytes(hex).unwrap(),
        ));
    }
    (frame as u32, msg)
}

fn header_dump(h: &ReplayHeader) -> String {
    let discons: String = h.player_discons.iter().map(u8::to_string).collect();
    format!(
        "{} {} {} {} {} {} [{}] {} [{}] [{}] {} {} {} {} {}",
        h.start_time,
        h.end_time,
        h.frame_count,
        h.desync_game,
        h.quit_early,
        discons,
        wide(&h.replay_name),
        encode_hex_bytes(&h.system_time),
        wide(&h.version_string),
        wide(&h.version_time_string),
        h.version_number,
        h.exe_crc,
        h.ini_crc,
        encode_hex_bytes(&h.game_options),
        h.local_player_index,
    )
}

#[test]
fn options_strings() {
    let rules = rules(VECTORS);
    let rows = section(VECTORS, "options");
    assert!(rows.len() > 600, "{}", rows.len());
    let mut accepted = 0;
    for (i, row) in rows.iter().enumerate() {
        let input = bytes(row, "in");
        let got = GameOptions::parse(&input, &rules);
        let ok = get_u32(row, "ok") == Some(1);
        assert_eq!(
            got.is_some(),
            ok,
            "option row {i}: {:?}",
            String::from_utf8_lossy(&input)
        );
        let Some(got) = got else { continue };
        accepted += 1;
        assert_eq!(
            info_dump(&got, &rules),
            get_str(row, "info").unwrap(),
            "option row {i}: {:?}",
            String::from_utf8_lossy(&input)
        );
        assert_eq!(
            encode_hex_bytes(&got.to_ascii(&rules)),
            get_str(row, "out").unwrap(),
            "option row {i}"
        );
    }
    assert!(accepted > 300, "{accepted}");
}

#[test]
fn playback() {
    let rules = rules(VECTORS);
    let rows = section(VECTORS, "replays");
    assert!(rows.len() > 150);
    for (i, row) in rows.iter().enumerate() {
        let data = bytes(row, "in");
        let header = get_str(row, "header").unwrap();
        match read_replay_header(&data, &rules) {
            Ok((h, _)) => assert_eq!(header_dump(&h), header, "replay {i}"),
            Err(e) => assert_eq!("", header, "replay {i}: {e:?}"),
        }
        let played = ReplayReader::open(&data, &rules);
        assert_eq!(
            played.is_ok(),
            get_u32(row, "play") == Some(1),
            "replay {i}"
        );
        let Ok(mut reader) = played else { continue };
        assert_eq!(
            Some(reader.params.original_game_mode as u32),
            get_u32(row, "mode"),
            "replay {i}"
        );
        assert_eq!(
            command_dump(&reader.new_game_command()),
            get_str(row, "new_game").unwrap(),
            "replay {i}"
        );
        // Frame by frame, as the oracle drives `updatePlayback`.
        let mut got = Vec::new();
        let mut frame = 0u32;
        while reader.next_frame().is_some() {
            for m in reader.commands_for_frame(frame) {
                if m.is_delivered() {
                    got.push(format!("{frame} {}", command_dump(&m)));
                }
            }
            frame += 1;
            assert!(frame <= 200_000, "replay {i} does not stop");
        }
        assert_eq!(reader.error(), None, "replay {i}");
        assert_eq!(got, strings(row, "commands"), "replay {i}");
    }
}

#[test]
fn recording() {
    let rules = rules(VECTORS);
    let rows = section(VECTORS, "recordings");
    assert_eq!(rows.len(), 60);
    let mut written = 0;
    for (i, row) in rows.iter().enumerate() {
        let expected = bytes(row, "out");
        let reader = ReplayReader::open(&expected, &rules).expect("reference file");
        let mut header = reader.header.clone();
        assert_eq!((header.start_time, header.end_time), (0, 0));
        header.frame_count = 0;
        let mut writer = ReplayWriter::new(&header, &reader.params);
        let mut kept = Vec::new();
        for text in strings(row, "inputs") {
            let (frame, msg) = parse_command(text);
            if msg.is_recorded() {
                writer.write_command(frame, &msg);
                kept.push((frame, msg));
                written += 1;
            }
        }
        writer.set_end(0, get_u32(row, "end_frame").unwrap());
        assert_eq!(
            encode_hex_bytes(&writer.bytes),
            encode_hex_bytes(&expected),
            "recording {i}"
        );
        // The options string written is the one `GameInfoToAsciiString` gives back.
        assert_eq!(
            reader.options.to_ascii(&rules),
            reader.header.game_options,
            "recording {i}"
        );

        // Reading returns the same commands, wide characters cut to 16 bits.
        let mut reader = ReplayReader::open(&expected, &rules).unwrap();
        for (_, msg) in &mut kept {
            for a in &mut msg.args {
                if a.kind == arg_type::WIDE_CHAR {
                    *a = ReplayArg::new(a.kind, &(a.word(0) & 0xffff).to_le_bytes());
                }
            }
        }
        assert_eq!(reader.read_all(), kept, "recording {i}");
        assert_eq!(reader.bytes_left(), 0, "recording {i}");
    }
    assert!(written > 300, "{written}");
}

/// Header patches the vectors do not reach, at the offsets `Recorder.cpp` names.
#[test]
fn header_patches() {
    let header = ReplayHeader::default();
    let mut w = ReplayWriter::new(&header, &PlaybackParams::default());
    w.set_start_time(0x0102_0304);
    w.set_end(0x0506_0708, 9);
    w.set_desync();
    w.set_quit_early();
    w.set_player_disconnected(0);
    w.set_player_disconnected(7);
    w.set_player_disconnected(8);
    let mut expected = b"GENREP".to_vec();
    expected.extend_from_slice(&[4, 3, 2, 1, 8, 7, 6, 5, 9, 0, 0, 0, 1, 1]);
    expected.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 1]);
    assert_eq!(w.bytes[..expected.len()], expected[..]);
    assert_eq!(w.bytes[expected.len()], 0, "the name must stay empty");
}

#[test]
fn crc_queue() {
    let mut crc = CrcInfo::new(0);
    crc.add_playback_crc(10);
    crc.add_playback_crc(20);
    assert_eq!(crc.check_recorded_crc(99, false, 500), None);
    assert_eq!(
        crc.check_recorded_crc(11, true, 500),
        Some(CrcMismatch {
            frame: 498,
            in_game: 10,
            replay: 11
        })
    );
    assert_eq!(crc.check_recorded_crc(20, true, 500), None);
    // Nothing queued reads as zero, and the first frames are never reported.
    assert_eq!(crc.check_recorded_crc(5, true, 0), None);
    assert_eq!(crc.check_recorded_crc(5, true, 1), None);
    assert_eq!(crc.check_recorded_crc(0, true, 9), None);
    assert_eq!(crc.check_recorded_crc(5, true, 9).map(|m| m.frame), Some(8));
    crc.saw_mismatch = true;
    assert_eq!(crc.check_recorded_crc(5, true, 9), None);
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
    })
}

fn replay_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    let dir = PathBuf::from(home).join("Library/Application Support/GeneralsX/GeneralsZH/Replays");
    dir.is_dir().then_some(dir)
}

#[test]
fn local_replays_match_the_reference() {
    let rules = rules(LOCAL);
    let rows = section(LOCAL, "replays");
    assert_eq!(rows.len(), 6);
    let baseline = include_str!("../../../docs/baselines/replay_crc.json");
    let Some(dir) = replay_dir() else {
        eprintln!("skipped: no local replays");
        return;
    };
    let mut totals: BTreeMap<i32, usize> = BTreeMap::new();
    let mut baseline_points = 0;
    for row in rows {
        let name = get_str(row, "file").unwrap();
        let Ok(data) = std::fs::read(dir.join(name)) else {
            eprintln!("skipped: {name} is missing");
            continue;
        };
        let mut reader = ReplayReader::open(&data, &rules).expect(name);
        assert_eq!(
            fnv(&header_dump(&reader.header)).to_string(),
            get_str(row, "header_fnv").unwrap(),
            "{name}"
        );
        let all = reader.read_all();
        assert_eq!(reader.error(), None, "{name}");
        assert_eq!(reader.bytes_left(), 0, "{name}: not read to the end");

        let (mut dump, mut crcs, mut crc_count) = (String::new(), String::new(), 0);
        let mut by_type: BTreeMap<i32, usize> = BTreeMap::new();
        let mut recorded = Vec::new();
        for (frame, m) in all.iter().filter(|(_, m)| m.is_delivered()) {
            dump += &format!("{frame} {}\n", command_dump(m));
            *by_type.entry(m.msg_type).or_default() += 1;
            *totals.entry(m.msg_type).or_default() += 1;
            if m.msg_type == MSG_LOGIC_CRC {
                let value = m.arg(0).as_int() as u32;
                crcs += &format!("{frame} {} {value}\n", m.player_index);
                crc_count += 1;
                recorded.push((*frame, value));
            }
        }
        let count = dump.lines().count() as u32;
        assert_eq!(Some(count), get_u32(row, "commands"), "{name}");
        assert_eq!(
            fnv(&dump).to_string(),
            get_str(row, "commands_fnv").unwrap(),
            "{name}"
        );
        assert_eq!(Some(crc_count), get_u32(row, "crc_messages"), "{name}");
        assert_eq!(
            fnv(&crcs).to_string(),
            get_str(row, "crc_fnv").unwrap(),
            "{name}"
        );
        let counts: Vec<String> = by_type.iter().map(|(t, n)| format!("{t}:{n}")).collect();
        assert_eq!(counts.join(" "), get_str(row, "by_type").unwrap(), "{name}");

        // The engine's own replay check prints the recorded checksum for each point.
        let points = baseline_replay_crcs(baseline, name);
        // Every recorded checksum but the first, whose frame the engine treats as zero.
        let engine: Vec<u32> = points.iter().map(|p| p.1).collect();
        let file: Vec<u32> = recorded.iter().skip(1).map(|p| p.1).collect();
        assert_eq!(file, engine, "{name}");
        baseline_points += points.len();
        eprintln!(
            "{name}: {} commands, {count} delivered, {crc_count} checksums, {} engine points",
            all.len(),
            points.len()
        );
    }
    let mut ordered: Vec<(i32, usize)> = totals.into_iter().collect();
    ordered.sort_by_key(|&(t, n)| (std::cmp::Reverse(n), t));
    for (t, n) in ordered {
        let route = if is_routed(t) { "routed" } else { "recorded" };
        eprintln!("type {t}: {n} ({route})");
    }
    assert!(baseline_points > 0);
}

/// The `Replay:` column of `docs/baselines/replay_crc.json` for one file.
fn baseline_replay_crcs(json: &str, name: &str) -> Vec<(u32, u32)> {
    let Some(start) = json.find(&format!("\"{name}\"")) else {
        return Vec::new();
    };
    let body = &json[start..];
    let list = &body[body.find("\"crcs\"").expect("crcs")..];
    let list = &list[..list.find("\"game_time\"").unwrap_or(list.len())];
    let mut out = Vec::new();
    // Tokens come in threes: frame, in-game, replay.
    let tokens: Vec<&str> = list["\"crcs\"".len()..]
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|t| !t.is_empty())
        .collect();
    for t in tokens.chunks_exact(3) {
        out.push((
            t[0].parse().unwrap(),
            u32::from_str_radix(t[2], 16).unwrap(),
        ));
    }
    out
}
