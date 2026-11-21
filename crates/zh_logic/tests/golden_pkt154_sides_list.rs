//! Gate V2 Golden Vector parity tests for `PKT-154` (`WP-4.1` `SidesList`).
//!
//! Active structural check (`test_pkt154_oracle_structure_and_constants`) verifies the committed
//! C++ oracle and constants. Remove the two `#[ignore]` attributes once `PKT-154` is implemented.

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_vec, get_u32, parse_i64_token,
};
use zh_common::ini::{IniLoadType, IniReader, SEPS_DEFAULT};
use zh_common::io::data_chunk::{CachedFileInputStream, DataChunkOutput};
use zh_common::math::Coord3D;
use zh_common::types::{ObjectId, FALSE, TRUE};
use zh_common::xfer::{Xfer, XferCrc, XferError, XferLoad, XferSave};
use zh_logic::spatial::{
    BuildListInfo, SidesList, SidesScriptList, WellKnownSidesKeys, MAX_PLAYER_COUNT,
    MAX_RESOURCE_GATHERERS, MAX_SKIRMISH_SIDES, MAX_TEAM_DEPTH,
    PLAYERS_NAMES_FOR_SCRIPTS_VERSION_1, PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2,
    SIDES_DATA_VERSION_1, SIDES_DATA_VERSION_2, SIDES_DATA_VERSION_3, UNLIMITED_REBUILDS,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-154.json");

fn find_json_value_start<'a>(obj: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{key}\":");
    let pos = obj.find(&pat)? + pat.len();
    Some(obj[pos..].trim_start())
}

fn get_unescaped_json_str(obj: &str, key: &str) -> Option<String> {
    let rest = find_json_value_start(obj, key)?;
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'"') {
        return None;
    }
    let mut out = String::new();
    let mut i = 1usize;
    while i < bytes.len() {
        let b = bytes[i];
        if b == b'"' {
            return Some(out);
        }
        if b == b'\\' {
            i += 1;
            if i >= bytes.len() {
                return None;
            }
            match bytes[i] {
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                b'\\' => out.push('\\'),
                b'"' => out.push('"'),
                other => out.push(other as char),
            }
        } else {
            out.push(b as char);
        }
        i += 1;
    }
    None
}

fn parse_str_array(rest: &str) -> Option<Vec<String>> {
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }
    let mut i = 1usize;
    let mut out = Vec::new();
    while i < bytes.len() {
        while i < bytes.len() && matches!(bytes[i], b' ' | b'\t' | b',') {
            i += 1;
        }
        if i >= bytes.len() {
            return None;
        }
        if bytes[i] == b']' {
            return Some(out);
        }
        if bytes[i] != b'"' {
            return None;
        }
        i += 1;
        let mut s = String::new();
        while i < bytes.len() && bytes[i] != b'"' {
            if bytes[i] == b'\\' {
                i += 1;
            }
            if i >= bytes.len() {
                return None;
            }
            s.push(bytes[i] as char);
            i += 1;
        }
        i += 1;
        out.push(s);
    }
    None
}

fn split_top_tuples(rest: &str) -> Option<Vec<&str>> {
    if !rest.starts_with('[') {
        return None;
    }
    let bytes = rest.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut start = None;
    let mut in_str = false;
    let mut escape = false;
    for (idx, &b) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        if in_str {
            if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'[' => {
                if depth == 1 {
                    start = Some(idx);
                }
                depth += 1;
            }
            b']' => {
                depth -= 1;
                if depth == 1 {
                    if let Some(s) = start.take() {
                        out.push(&rest[s..=idx]);
                    }
                } else if depth == 0 {
                    return Some(out);
                }
            }
            _ => {}
        }
    }
    None
}

fn split_tuple_fields(tuple: &str) -> Vec<&str> {
    let inner = tuple.trim().trim_start_matches('[').trim_end_matches(']');
    let bytes = inner.as_bytes();
    let mut out = Vec::new();
    let mut depth = 0i32;
    let mut in_str = false;
    let mut escape = false;
    let mut start = 0usize;
    for (idx, &b) in bytes.iter().enumerate() {
        if escape {
            escape = false;
            continue;
        }
        if in_str {
            if b == b'\\' {
                escape = true;
            } else if b == b'"' {
                in_str = false;
            }
            continue;
        }
        match b {
            b'"' => in_str = true,
            b'[' => depth += 1,
            b']' => depth -= 1,
            b',' if depth == 0 => {
                out.push(inner[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start <= inner.len() && !inner[start..].trim().is_empty() {
        out.push(inner[start..].trim());
    }
    out
}

fn unquote(tok: &str) -> String {
    tok.trim().trim_matches('"').to_owned()
}

#[test]
fn test_pkt154_oracle_structure_and_constants() {
    assert!(VECTORS_JSON.contains("\"packet_id\": \"PKT-154\""));
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    assert!(VECTORS_JSON.contains("SidesList.h"));
    assert!(VECTORS_JSON.contains("SidesList.cpp"));

    assert_eq!(MAX_PLAYER_COUNT, 16);
    assert_eq!(MAX_TEAM_DEPTH, 3);
    assert_eq!(MAX_SKIRMISH_SIDES, 64);
    assert_eq!(UNLIMITED_REBUILDS, 0xFFFF_FFFF);
    assert_eq!(MAX_RESOURCE_GATHERERS, 10);
    assert_eq!(SIDES_DATA_VERSION_1, 1);
    assert_eq!(SIDES_DATA_VERSION_2, 2);
    assert_eq!(SIDES_DATA_VERSION_3, 3);
    assert_eq!(PLAYERS_NAMES_FOR_SCRIPTS_VERSION_1, 1);
    assert_eq!(PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2, 2);

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_pkt154_build_list_and_xfer_vectors() {
    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let keys = WellKnownSidesKeys::intern(&mut name_keys);

    let mut sl = SidesList::new();
    sl.clear();
    sl.add_player_by_template("FactionAmerica", &keys);
    assert_eq!(sl.num_sides(), 1);
    sl.remove_side(0);
    assert_eq!(sl.num_sides(), 1);
    assert_eq!(sl.validate_sides(&keys), TRUE);
    assert_eq!(sl.num_sides(), 2);

    let mut bli = BuildListInfo::new();
    bli.num_rebuilds = UNLIMITED_REBUILDS;
    bli.decrement_num_rebuilds();
    bli.increment_num_rebuilds();
    assert_eq!(bli.num_rebuilds, UNLIMITED_REBUILDS);
    assert_eq!(bli.is_buildable(), TRUE);
    bli.num_rebuilds = 1;
    bli.decrement_num_rebuilds();
    assert_eq!(bli.num_rebuilds, 0);
    assert_eq!(bli.is_buildable(), FALSE);

    let mut xs = XferSave::new();
    xs.xfer_snapshot(&mut sl).unwrap();
    let bytes = xs.as_bytes().to_vec();

    let mut mismatch = SidesList::new();
    let mut xl = XferLoad::new(&bytes);
    assert_eq!(xl.xfer_snapshot(&mut mismatch), Err(XferError::InvalidData));
}

#[test]
fn test_pkt154_sides_list_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let keys = WellKnownSidesKeys::intern(&mut name_keys);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let mut sides = SidesList::new();
        sides.clear();

        let tmpls = parse_str_array(find_json_value_start(line, "init_tmpls").unwrap()).unwrap();
        for tmpl in &tmpls {
            sides.add_player_by_template(tmpl, &keys);
        }

        let n_sides = sides.num_sides() as usize;
        for i in 0..n_sides {
            let pname = sides
                .side_info(i as i32)
                .unwrap()
                .dict
                .get_ascii_string(keys.player_name, None)
                .to_owned();
            let mut allies = format!("NoSuchPlayer {pname}");
            if i + 1 < n_sides {
                let next_name = sides
                    .side_info((i + 1) as i32)
                    .unwrap()
                    .dict
                    .get_ascii_string(keys.player_name, None)
                    .to_owned();
                allies.push(' ');
                allies.push_str(&next_name);
            }
            sides
                .side_info_mut(i as i32)
                .unwrap()
                .dict
                .set_ascii_string(keys.player_allies, &allies);
        }

        for b_tuple in split_top_tuples(find_json_value_start(line, "init_blds").unwrap()).unwrap() {
            let f = split_tuple_fields(b_tuple);
            let side_idx = parse_i64_token(f[0]).unwrap() as i32;
            let pos = parse_i64_token(f[1]).unwrap() as i32;
            let mut bli = BuildListInfo::new();
            bli.building_name = unquote(f[2]);
            bli.template_name = unquote(f[3]);
            bli.location = Coord3D {
                x: f32::from_bits(parse_i64_token(f[4]).unwrap() as u32),
                y: f32::from_bits(parse_i64_token(f[5]).unwrap() as u32),
                z: f32::from_bits(parse_i64_token(f[6]).unwrap() as u32),
            };
            bli.angle = f32::from_bits(parse_i64_token(f[7]).unwrap() as u32);
            bli.is_initially_built = parse_i64_token(f[8]).unwrap() as u8;
            bli.num_rebuilds = parse_i64_token(f[9]).unwrap() as u32;
            bli.script = unquote(f[10]);
            bli.health = parse_i64_token(f[11]).unwrap() as i32;
            bli.whiner = parse_i64_token(f[12]).unwrap() as u8;
            bli.unsellable = parse_i64_token(f[13]).unwrap() as u8;
            bli.repairable = parse_i64_token(f[14]).unwrap() as u8;
            bli.object_id = ObjectId(parse_i64_token(f[15]).unwrap() as i32);
            bli.object_timestamp = parse_i64_token(f[16]).unwrap() as u32;
            bli.under_construction = parse_i64_token(f[17]).unwrap() as u8;
            bli.is_supply_building = parse_i64_token(f[18]).unwrap() as u8;
            bli.desired_gatherers = parse_i64_token(f[19]).unwrap() as i32;
            bli.current_gatherers = parse_i64_token(f[20]).unwrap() as i32;
            bli.priority_build = parse_i64_token(f[21]).unwrap() as u8;
            let gath_inner = f[22].trim().trim_start_matches('[').trim_end_matches(']');
            for (g_idx, g_tok) in gath_inner.split(',').enumerate() {
                bli.set_gatherer_id(g_idx as i32, ObjectId(parse_i64_token(g_tok).unwrap() as i32));
            }
            sides
                .side_info_mut(side_idx)
                .unwrap()
                .add_to_build_list(bli, pos);
        }

        for r_idx in get_i32_vec(line, "init_reorders").unwrap() {
            sides.side_info_mut(r_idx).unwrap().reorder_in_build_list(0, 2);
        }

        for s_tuple in split_top_tuples(find_json_value_start(line, "init_scripts").unwrap()).unwrap() {
            let f = split_tuple_fields(s_tuple);
            let side_idx = parse_i64_token(f[0]).unwrap() as i32;
            let hs = parse_i64_token(f[1]).unwrap() as u8;
            let hg = parse_i64_token(f[2]).unwrap() as u8;
            let tag = parse_i64_token(f[3]).unwrap() as i32;
            sides.side_info_mut(side_idx).unwrap().scripts = Some(SidesScriptList::new(hs, hg, tag));
        }

        for bt in split_top_tuples(find_json_value_start(line, "init_bad_teams").unwrap()).unwrap() {
            let f = split_tuple_fields(bt);
            let mut d = Dict::new();
            d.set_ascii_string(keys.team_name, &unquote(f[0]));
            d.set_ascii_string(keys.team_owner, &unquote(f[1]));
            d.set_bool(keys.team_is_singleton, parse_i64_token(f[2]).unwrap() as u8);
            sides.add_team(Some(&d));
        }

        let val_mod = sides.validate_sides(&keys);
        assert_eq!(val_mod as u32, get_u32(line, "val_mod").unwrap(), "seed {seed} val_mod");
        assert_eq!(sides.num_sides(), get_i32(line, "num_sides").unwrap(), "seed {seed} num_sides");
        assert_eq!(sides.num_teams(), get_i32(line, "num_teams").unwrap(), "seed {seed} num_teams");

        // BuildListInfo::parse_structure INI check
        let bli_ini = get_unescaped_json_str(line, "bli_ini").unwrap();
        let mut ini = IniReader::new("bli.ini", bli_ini.as_bytes(), IniLoadType::Overwrite);
        let mut parsed_bli: Option<BuildListInfo> = None;
        ini.load_blocks(None, |_block, reader| {
            let _kw = reader.next_token(SEPS_DEFAULT)?;
            parsed_bli = Some(BuildListInfo::parse_structure(reader)?);
            Ok(())
        })
        .unwrap();
        let mut parsed_bli = parsed_bli.expect("AIData block parsed");
        let mut bix = XferSave::new();
        bix.xfer_snapshot(&mut parsed_bli).unwrap();
        assert_eq!(
            bix.as_bytes(),
            decode_hex_bytes(&get_unescaped_json_str(line, "bli_ini_xfer_hex").unwrap())
                .unwrap()
                .as_slice(),
            "seed {seed} bli_ini_xfer"
        );

        // WriteSidesDataChunk & ParseSidesDataChunk check
        let mut chunk_bytes = Vec::new();
        {
            let mut cw = DataChunkOutput::new(&mut chunk_bytes);
            sides.write_sides_data_chunk(&mut cw, &keys, &name_keys).unwrap();
            cw.finish().unwrap();
        }
        assert_eq!(
            chunk_bytes,
            decode_hex_bytes(&get_unescaped_json_str(line, "sides_chunk_hex").unwrap()).unwrap(),
            "seed {seed} sides_chunk_hex"
        );

        let mut parsed_sides = SidesList::new();
        let mut stream = CachedFileInputStream::from_bytes(&chunk_bytes);
        parsed_sides
            .parse_sides_chunk_stream(&mut stream, &keys, &mut name_keys)
            .unwrap();
        assert_eq!(parsed_sides.num_sides(), get_i32(line, "parsed_sides").unwrap());
        assert_eq!(parsed_sides.num_teams(), get_i32(line, "parsed_teams").unwrap());

        // XferSave & XferCRC check
        let mut xsave = XferSave::new();
        xsave.xfer_snapshot(&mut sides).unwrap();
        assert_eq!(
            xsave.as_bytes(),
            decode_hex_bytes(&get_unescaped_json_str(line, "sides_xfer_hex").unwrap())
                .unwrap()
                .as_slice(),
            "seed {seed} sides_xfer"
        );

        let mut xcrc = XferCrc::new();
        xcrc.xfer_snapshot(&mut sides).unwrap();
        assert_eq!(xcrc.get_crc(), get_u32(line, "sides_crc").unwrap(), "seed {seed} sides_crc");

        // BuildListInfo[0] v2 xfer check
        let exp_bli_hex = get_unescaped_json_str(line, "bli_xfer_hex").unwrap();
        if !exp_bli_hex.is_empty() {
            let mut bx = XferSave::new();
            let b0 = &mut sides.side_info_mut(0).unwrap().build_list[0];
            bx.xfer_snapshot(b0).unwrap();
            assert_eq!(
                bx.as_bytes(),
                decode_hex_bytes(&exp_bli_hex).unwrap().as_slice(),
                "seed {seed} bli_xfer"
            );
        }

        // prepare_for_mp_or_skirmish check
        let force_empty = get_u32(line, "force_empty_scripts").unwrap() != 0;
        let scb_bytes = decode_hex_bytes(&get_unescaped_json_str(line, "skirmish_scb_hex").unwrap()).unwrap();
        let mut mp_sides = SidesList::new();
        mp_sides.clear();
        for i in 0..sides.num_sides() {
            let s = sides.side_info(i).unwrap();
            mp_sides.add_side(Some(&s.dict));
            if !force_empty {
                if let Some(ref sl) = s.scripts {
                    mp_sides.side_info_mut(i).unwrap().scripts = Some(sl.clone());
                }
            }
        }
        for i in 0..sides.num_teams() {
            let t = sides.team_info(i).unwrap();
            mp_sides.add_team(Some(&t.dict));
        }
        let scb_arg = if scb_bytes.is_empty() { None } else { Some(scb_bytes.as_slice()) };
        mp_sides
            .prepare_for_mp_or_skirmish(scb_arg, &keys, &mut name_keys)
            .unwrap();

        let mut script_sum = 0i32;
        for i in 0..mp_sides.num_skirmish_sides() {
            if let Some(ref psl) = mp_sides.skirmish_side_info(i).unwrap().scripts {
                script_sum += psl.tag
                    + if psl.has_script != FALSE { 1 } else { 0 }
                    + if psl.has_group != FALSE { 2 } else { 0 };
            }
        }
        assert_eq!(
            vec![
                mp_sides.num_sides(),
                mp_sides.num_skirmish_sides(),
                mp_sides.num_teams(),
                mp_sides.num_skirmish_teams(),
                script_sum,
            ],
            get_i32_vec(line, "prep_res").unwrap(),
            "seed {seed} prep_res"
        );

        let exp_sides = split_top_tuples(find_json_value_start(line, "final_sides").unwrap()).unwrap();
        for (i, st) in exp_sides.iter().enumerate() {
            let f = split_tuple_fields(st);
            let sd = &sides.side_info(i as i32).unwrap().dict;
            assert_eq!(sd.get_ascii_string(keys.player_name, None), unquote(f[0]));
            assert_eq!(sd.get_ascii_string(keys.player_allies, None), unquote(f[1]));
            assert_eq!(sd.get_ascii_string(keys.player_enemies, None), unquote(f[2]));
        }

        let exp_teams = split_top_tuples(find_json_value_start(line, "final_teams").unwrap()).unwrap();
        for (i, tt) in exp_teams.iter().enumerate() {
            let f = split_tuple_fields(tt);
            let ti = sides.team_info(i as i32).unwrap();
            assert_eq!(ti.dict.get_ascii_string(keys.team_name, None), unquote(f[0]));
            assert_eq!(ti.dict.get_ascii_string(keys.team_owner, None), unquote(f[1]));
            assert_eq!(sides.is_player_default_team(ti, &keys) as i32, parse_i64_token(f[2]).unwrap() as i32);
        }
    }
}
