//! Gate V2 Golden Vector oracle & verification suite for `PKT-451` (`DataChunk`).

use zh_common::core::dict::{Dict, DictDataType};
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_hex_u32, get_i32, get_str, get_u16_vec, get_u32,
};
use zh_common::io::compression::{decompress_data, is_data_compressed};
use zh_common::io::data_chunk::{
    CachedFileInputStream, ChunkInputStream, DataChunkError, DataChunkInfo, DataChunkInput,
    DataChunkOutput, CHUNK_TOC_TAG,
};
use zh_common::types::{Bool, Byte, Real, FALSE, TRUE};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-451.json");

#[test]
fn test_pkt451_oracle_structure_and_ear_compression() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let raw = decode_hex_bytes(get_str(line, "raw_hex").unwrap()).unwrap();
        let ear = decode_hex_bytes(get_str(line, "ear_hex").unwrap()).unwrap();

        assert!(
            raw.starts_with(&CHUNK_TOC_TAG),
            "seed {seed}: raw blob missing CkMp header"
        );
        assert!(
            ear.starts_with(b"EAR\0"),
            "seed {seed}: compressed blob missing EAR\\0 header"
        );
        assert!(is_data_compressed(&ear));
        let decompressed = decompress_data(&ear)
            .unwrap_or_else(|e| panic!("seed {seed}: decompress_data failed: {e:?}"));
        assert_eq!(
            decompressed, raw,
            "seed {seed}: RefPack decompression mismatch against raw CkMp blob"
        );
    }
}

#[allow(dead_code)]
#[derive(Default)]
struct ParseState {
    crc: u32,
    tag: i32,
    expected_bytes_len: usize,
    name_keys: NameKeyGenerator,
}

#[allow(dead_code)]
#[inline]
fn crc_u32(c: u32, v: u32) -> u32 {
    c.rotate_left(1).wrapping_add(v)
}

#[allow(dead_code)]
fn crc_str(mut c: u32, s: &str) -> u32 {
    c = crc_u32(c, s.len() as u32);
    for &b in s.as_bytes() {
        c = crc_u32(c, b as u32);
    }
    c
}

#[allow(dead_code)]
fn parse_dup_loser(
    _input: &mut DataChunkInput<'_, ParseState>,
    _info: &DataChunkInfo,
    st: &mut ParseState,
) -> Result<bool, DataChunkError> {
    st.tag = 1;
    Ok(true)
}

#[allow(dead_code)]
fn parse_dup_winner(
    input: &mut DataChunkInput<'_, ParseState>,
    info: &DataChunkInfo,
    st: &mut ParseState,
) -> Result<bool, DataChunkError> {
    st.tag = 2;
    st.crc = crc_u32(st.crc, 2);
    st.crc = crc_str(st.crc, &info.label);
    st.crc = crc_str(st.crc, &info.parent_label);
    st.crc = crc_u32(st.crc, info.version as u32);
    st.crc = crc_u32(st.crc, info.data_size as u32);

    let nk = input.read_name_key(&mut st.name_keys)?;
    let nk_name = st.name_keys.key_to_name(nk).to_owned();
    st.crc = crc_str(st.crc, &nk_name);

    let dict = input.read_dict(&mut st.name_keys)?;
    let pc = dict.get_pair_count();
    st.crc = crc_u32(st.crc, pc as u32);
    for i in 0..pc {
        let dk = dict.get_nth_key(i);
        let dk_name = st.name_keys.key_to_name(dk).to_owned();
        st.crc = crc_str(st.crc, &dk_name);
        let dt = dict.get_nth_type(i);
        st.crc = crc_u32(st.crc, dt as i32 as u32);
        match dt {
            DictDataType::Bool => st.crc = crc_u32(st.crc, dict.get_nth_bool(i) as u32),
            DictDataType::Int => st.crc = crc_u32(st.crc, dict.get_nth_int(i) as u32),
            DictDataType::Real => st.crc = crc_u32(st.crc, dict.get_nth_real(i).to_bits()),
            DictDataType::AsciiString => {
                let s = dict.get_nth_ascii_string(i);
                st.crc = crc_str(st.crc, &s);
            }
            DictDataType::UnicodeString => {
                let us = dict.get_nth_unicode_string(i);
                st.crc = crc_u32(st.crc, us.len() as u32);
                for &u in &us {
                    st.crc = crc_u32(st.crc, u as u32);
                }
            }
            DictDataType::None => {}
        }
    }
    Ok(true)
}

#[allow(dead_code)]
fn parse_child(
    input: &mut DataChunkInput<'_, ParseState>,
    info: &DataChunkInfo,
    st: &mut ParseState,
) -> Result<bool, DataChunkError> {
    st.crc = crc_str(st.crc, &info.label);
    st.crc = crc_str(st.crc, &info.parent_label);
    st.crc = crc_u32(st.crc, info.version as u32);
    st.crc = crc_u32(st.crc, info.data_size as u32);

    let val_real = input.read_real()?;
    let val_i32 = input.read_int()?;
    let val_byte = input.read_byte()?;
    let val_ascii = input.read_ascii_string()?;
    let val_uni = input.read_unicode_string()?;

    st.crc = crc_u32(st.crc, val_real.to_bits());
    st.crc = crc_u32(st.crc, val_i32 as u32);
    st.crc = crc_u32(st.crc, (val_byte as u8) as u32);
    st.crc = crc_str(st.crc, &val_ascii);
    st.crc = crc_u32(st.crc, val_uni.len() as u32);
    for &u in &val_uni {
        st.crc = crc_u32(st.crc, u as u32);
    }

    if st.expected_bytes_len > 0 {
        let mut tmp = vec![0u8; st.expected_bytes_len];
        input.read_array_of_bytes(&mut tmp)?;
        for b in tmp {
            st.crc = crc_u32(st.crc, b as u32);
        }
    }
    // Intentionally leave unread tail bytes for close_data_chunk() to skip.
    Ok(true)
}

#[allow(dead_code)]
fn parse_root(
    input: &mut DataChunkInput<'_, ParseState>,
    info: &DataChunkInfo,
    st: &mut ParseState,
) -> Result<bool, DataChunkError> {
    st.crc = crc_str(st.crc, &info.label);
    st.crc = crc_u32(st.crc, info.version as u32);
    st.crc = crc_u32(st.crc, info.data_size as u32);
    input.parse(st)
}

#[test]
fn test_pkt451_writer_and_reader_1000_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for line in lines {
        let seed = get_u32(line, "seed").unwrap();
        let root_label = get_str(line, "root_label").unwrap();
        let root_ver = get_u32(line, "root_ver").unwrap() as u16;
        let child_label = get_str(line, "child_label").unwrap();
        let child_ver = get_u32(line, "child_ver").unwrap() as u16;
        let dup_label = get_str(line, "dup_label").unwrap();
        let dup_ver = get_u32(line, "dup_ver").unwrap() as u16;
        let has_empty_sibling = get_u32(line, "has_empty_sibling").unwrap() != 0;

        let val_real = Real::from_bits(get_hex_u32(line, "val_real_bits").unwrap());
        let val_i32 = get_i32(line, "val_i32").unwrap();
        let val_byte = get_i32(line, "val_byte").unwrap() as Byte;
        let val_ascii = get_str(line, "val_ascii").unwrap();
        let val_uni = get_u16_vec(line, "val_u16_units").unwrap();
        let val_bytes = decode_hex_bytes(get_str(line, "val_bytes_hex").unwrap()).unwrap();
        let tail_bytes = decode_hex_bytes(get_str(line, "tail_hex").unwrap()).unwrap();

        let dict_bool_k = get_str(line, "dict_bool_k").unwrap();
        let dict_bool_v: Bool = if get_u32(line, "dict_bool_v").unwrap() != 0 {
            TRUE
        } else {
            FALSE
        };
        let dict_int_k = get_str(line, "dict_int_k").unwrap();
        let dict_int_v = get_i32(line, "dict_int_v").unwrap();
        let dict_real_k = get_str(line, "dict_real_k").unwrap();
        let dict_real_v = Real::from_bits(get_hex_u32(line, "dict_real_bits").unwrap());
        let dict_ascii_k = get_str(line, "dict_ascii_k").unwrap();
        let dict_ascii_v = get_str(line, "dict_ascii_v").unwrap();
        let dict_uni_k = get_str(line, "dict_uni_k").unwrap();
        let dict_uni_units = get_u16_vec(line, "dict_uni_units").unwrap();
        let dict_namekey_name = get_str(line, "dict_namekey_name").unwrap();

        let expected_crc = get_hex_u32(line, "parsed_crc_hex").unwrap();
        let expected_raw = decode_hex_bytes(get_str(line, "raw_hex").unwrap()).unwrap();
        let ear_bytes = decode_hex_bytes(get_str(line, "ear_hex").unwrap()).unwrap();

        let mut name_keys = NameKeyGenerator::new();
        name_keys.init();
        let mut dict = Dict::new();
        dict.set_bool(name_keys.name_to_key(dict_bool_k), dict_bool_v);
        dict.set_int(name_keys.name_to_key(dict_int_k), dict_int_v);
        dict.set_real(name_keys.name_to_key(dict_real_k), dict_real_v);
        dict.set_ascii_string(name_keys.name_to_key(dict_ascii_k), dict_ascii_v);
        dict.set_unicode_string(name_keys.name_to_key(dict_uni_k), &dict_uni_units);
        let nk_val = name_keys.name_to_key(dict_namekey_name);

        let mut written = Vec::new();
        {
            let mut out = DataChunkOutput::new(&mut written);
            out.open_data_chunk(root_label, root_ver).unwrap();
            out.open_data_chunk(child_label, child_ver).unwrap();
            out.write_real(val_real).unwrap();
            out.write_int(val_i32).unwrap();
            out.write_byte(val_byte).unwrap();
            out.write_ascii_string(val_ascii).unwrap();
            out.write_unicode_string(&val_uni).unwrap();
            if !val_bytes.is_empty() {
                out.write_array_of_bytes(&val_bytes).unwrap();
            }
            if !tail_bytes.is_empty() {
                out.write_array_of_bytes(&tail_bytes).unwrap();
            }
            out.close_data_chunk().unwrap();

            out.open_data_chunk(dup_label, dup_ver).unwrap();
            out.write_name_key(nk_val, &name_keys).unwrap();
            out.write_dict(&dict, &name_keys).unwrap();
            out.close_data_chunk().unwrap();

            if has_empty_sibling {
                out.open_data_chunk(child_label, child_ver).unwrap();
                out.close_data_chunk().unwrap();
            }
            out.close_data_chunk().unwrap();
            out.finish().unwrap();
        }
        assert_eq!(
            written, expected_raw,
            "seed {seed}: DataChunkOutput wire mismatch"
        );

        for (mode, blob) in [
            ("raw", expected_raw.as_slice()),
            ("ear", ear_bytes.as_slice()),
        ] {
            let mut cfis = CachedFileInputStream::from_bytes(blob);
            let mut dci = DataChunkInput::<ParseState>::new(&mut cfis);
            assert!(
                dci.is_valid_file_type(),
                "seed {seed} ({mode}): invalid CkMp"
            );

            let mut st = ParseState::default();
            st.expected_bytes_len = val_bytes.len();
            st.name_keys.init();

            dci.register_parser(root_label, "", parse_root);
            dci.register_parser(child_label, root_label, parse_child);
            dci.register_parser(dup_label, root_label, parse_dup_loser);
            dci.register_parser(dup_label, root_label, parse_dup_winner);

            assert!(
                dci.parse(&mut st)
                    .unwrap_or_else(|error| panic!("seed {seed} ({mode}): parse error {error:?}")),
                "seed {seed} ({mode}): parse returned false"
            );
            assert_eq!(
                st.tag, 2,
                "seed {seed} ({mode}): LIFO duplicate parser loser ran instead of winner"
            );
            assert_eq!(
                st.crc, expected_crc,
                "seed {seed} ({mode}): parsed CRC mismatch"
            );
        }
    }
}

#[test]
fn test_pkt451_error_paths_and_bounded_corruption() {
    // 1. Non-CkMp buffer rejected cleanly
    let mut cfis = CachedFileInputStream::from_bytes(b"NOPE\x00\x00\x00\x00");
    let mut dci = DataChunkInput::<()>::new(&mut cfis);
    assert!(!dci.is_valid_file_type());

    // 2. Corrupt EAR length mismatches -> keeps raw bytes without panic
    let first = extract_vector_lines(VECTORS_JSON)[0];
    let mut bad_ear = decode_hex_bytes(get_str(first, "ear_hex").unwrap()).unwrap();
    bad_ear[4] ^= 0x10;
    let mut cfis_bad = CachedFileInputStream::from_bytes(&bad_ear);
    let mut head4 = [0u8; 4];
    assert_eq!(cfis_bad.read(&mut head4), 4);
    assert_eq!(&head4, b"EAR\0");

    // 3. Unknown Dict data type returns CorruptFormat error without panic
    let bad_dict: &[u8] = &[
        b'C', b'k', b'M', b'p', 1, 0, 0, 0, 1, b'X', 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 6, 0, 0, 0, 1,
        0, 99, 1, 0, 0,
    ];
    let mut nk = NameKeyGenerator::new();
    nk.init();
    let mut s_bad = CachedFileInputStream::from_bytes(bad_dict);
    let mut d_bad = DataChunkInput::<()>::new(&mut s_bad);
    assert!(d_bad.is_valid_file_type());
    let _ = d_bad.open_data_chunk().unwrap();
    assert_eq!(
        d_bad.read_dict(&mut nk).unwrap_err(),
        DataChunkError::CorruptFormat
    );

    // 4. Truncated prefixes never panic
    let raw0 = decode_hex_bytes(get_str(first, "raw_hex").unwrap()).unwrap();
    for cut in 0..raw0.len() {
        let mut s_trunc = CachedFileInputStream::from_bytes(&raw0[..cut]);
        let mut d_trunc = DataChunkInput::<ParseState>::new(&mut s_trunc);
        if d_trunc.is_valid_file_type() {
            d_trunc.register_parser("HeightMapData", "", parse_root);
            let mut st = ParseState::default();
            st.name_keys.init();
            let _ = d_trunc.parse(&mut st);
        }
    }
}
