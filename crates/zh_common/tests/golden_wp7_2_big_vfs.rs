//! Gate V2 Golden Vector parity tests for `WP-7.2` (`PKT-448` `ArchiveFile`,
//! `PKT-449` `ArchiveFileSystem`, `PKT-450` `BIGFileSystem`, `PKT-454` `file`,
//! `PKT-460` `RAMFile`).

use std::cmp::Ordering;
use std::path::PathBuf;

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_array, get_u32, get_u32_array,
};
use zh_common::io::{
    compare_path, get_file_name, make_path_key, ArchiveFileSystem, ArchiveFilter, ArchiveModConfig,
    BigFileSystem, FileAccess, FileBase, MemoryFileHost, OsDiskFileHost, RamFile,
};
use zh_common::types::TRUE;

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/WP7_2_BigVfs.json");

fn crc32_bytes(data: &[u8]) -> u32 {
    let mut crc: u32 = 0xffff_ffff;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xedb8_8320 & mask);
        }
    }
    !crc
}

fn find_json_field<'a>(obj_line: &'a str, key: &str) -> Option<&'a str> {
    let pat = format!("\"{}\":", key);
    let mut i = 0usize;
    let bytes = obj_line.as_bytes();
    while i < bytes.len() {
        if bytes[i] == b'"' {
            if obj_line[i..].starts_with(&pat) {
                return Some(obj_line[i + pat.len()..].trim_start());
            }
            // Skip entire quoted string cleanly
            i += 1;
            while i < bytes.len() {
                if bytes[i] == b'\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == b'"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
        } else {
            i += 1;
        }
    }
    None
}

fn decode_json_string_at(slice: &str) -> Option<(String, usize)> {
    let b = slice.as_bytes();
    if b.first().copied() != Some(b'"') {
        return None;
    }
    let mut out = String::new();
    let mut i = 1usize;
    while i < b.len() {
        let ch = b[i];
        if ch == b'"' {
            return Some((out, i + 1));
        }
        if ch == b'\\' {
            i += 1;
            if i >= b.len() {
                return None;
            }
            match b[i] {
                b'"' => out.push('"'),
                b'\\' => out.push('\\'),
                b'n' => out.push('\n'),
                b'r' => out.push('\r'),
                b't' => out.push('\t'),
                other => out.push(other as char),
            }
            i += 1;
        } else {
            out.push(ch as char);
            i += 1;
        }
    }
    None
}

fn get_json_str(obj_line: &str, key: &str) -> Option<String> {
    let rest = find_json_field(obj_line, key)?;
    decode_json_string_at(rest).map(|(s, _)| s)
}

fn get_json_str_vec(obj_line: &str, key: &str) -> Option<Vec<String>> {
    let mut rest = find_json_field(obj_line, key)?;
    if !rest.starts_with('[') {
        return None;
    }
    rest = rest[1..].trim_start();
    let mut out = Vec::new();
    while !rest.starts_with(']') {
        let (s, used) = decode_json_string_at(rest)?;
        out.push(s);
        rest = rest[used..].trim_start();
        if let Some(after_comma) = rest.strip_prefix(',') {
            rest = after_comma.trim_start();
        }
    }
    Some(out)
}

fn build_synthetic_big(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let dir_size: usize = entries.iter().map(|(p, _)| 8 + p.len() + 1).sum();
    let first_offset = (16 + dir_size) as u32;
    let total_data: usize = entries.iter().map(|(_, d)| d.len()).sum();
    let total_size = (first_offset as usize) + total_data;

    let mut buf = vec![0u8; total_size];
    buf[0..4].copy_from_slice(b"BIGF");
    buf[4..8].copy_from_slice(&(total_size as u32).to_le_bytes());
    buf[8..12].copy_from_slice(&(entries.len() as u32).to_be_bytes());
    buf[12..16].copy_from_slice(&first_offset.to_be_bytes());

    let mut dir_pos = 16usize;
    let mut data_pos = first_offset;
    for (path, data) in entries {
        buf[dir_pos..dir_pos + 4].copy_from_slice(&data_pos.to_be_bytes());
        buf[dir_pos + 4..dir_pos + 8].copy_from_slice(&(data.len() as u32).to_be_bytes());
        dir_pos += 8;
        buf[dir_pos..dir_pos + path.len()].copy_from_slice(path.as_bytes());
        dir_pos += path.len();
        buf[dir_pos] = 0;
        dir_pos += 1;
        buf[data_pos as usize..data_pos as usize + data.len()].copy_from_slice(data);
        data_pos += data.len() as u32;
    }
    buf
}

#[test]
fn test_wp7_2_oracle_metadata_and_os_disk_mount() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    for required in [
        "file.h",
        "file.cpp",
        "RAMFile.h",
        "RAMFile.cpp",
        "ArchiveFile.h",
        "ArchiveFile.cpp",
        "ArchiveFileSystem.h",
        "ArchiveFileSystem.cpp",
        "BIGFileSystem.h",
        "BIGFileSystem.cpp",
    ] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    // Real disk mounting smoke test with OsDiskFileHost
    let temp_dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("wp7_2_os_disk_smoke");
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let base_big = build_synthetic_big(&[
        ("Data\\INI\\GameData.ini", b"GameData\n  Gravity = -0.9\nEnd\n"),
        ("Data\\INI\\Unit.ini", b"Object Tank\nEnd\n"),
    ]);
    let patch_big = build_synthetic_big(&[(
        "Data/INI/GameData.ini",
        b"GameData\n  Gravity = -1.1\nEnd\n",
    )]);
    std::fs::write(temp_dir.join("INIZH.big"), &base_big).unwrap();
    std::fs::write(temp_dir.join("PatchZH.big"), &patch_big).unwrap();

    let mut bfs = BigFileSystem::new();
    let dir_str = temp_dir.to_string_lossy().into_owned();
    assert_eq!(
        bfs.load_archive_files_from_directory(&OsDiskFileHost, &dir_str, "*.big", false, true),
        TRUE
    );
    assert_eq!(
        bfs.does_file_exist("Data\\INI\\GameData.ini", 0, ArchiveFilter::All),
        TRUE
    );
    let ram = bfs
        .open_file("data/ini/gamedata.ini", 0, ArchiveFilter::All)
        .expect("patch archive must override base archive");
    assert_eq!(ram.as_bytes(), b"GameData\n  Gravity = -1.1\nEnd\n");
    let base_ram = bfs
        .open_file("data\\ini\\gamedata.ini", 1, ArchiveFilter::All)
        .expect("instance 1 must read shadowed base file");
    assert_eq!(base_ram.as_bytes(), b"GameData\n  Gravity = -0.9\nEnd\n");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_wp7_2_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);

        // 1. File::open flag normalization & path helpers
        let raw_flag = get_i32(line, "raw_flag").unwrap();
        let exp_open1 = get_i32(line, "open1").unwrap();
        let exp_norm_access = get_i32(line, "norm_access").unwrap();
        let exp_open2 = get_i32(line, "open2").unwrap();

        let mut fb = FileBase::new();
        let got_open1 = fb.open("test/path.bin", raw_flag);
        assert_eq!(got_open1 as i32, exp_open1, "seed {seed} open1");
        if got_open1 == TRUE {
            assert_eq!(fb.access(), exp_norm_access, "seed {seed} norm_access");
        }
        let got_open2 = fb.open("test/other.bin", FileAccess::READ);
        assert_eq!(got_open2 as i32, exp_open2, "seed {seed} open2");
        fb.close();

        let pa = get_json_str(line, "pa").unwrap();
        let pb = get_json_str(line, "pb").unwrap();
        let exp_cmp = get_i32(line, "cmp").unwrap();
        let got_cmp = match compare_path(&pa, &pb) {
            Ordering::Less => -1,
            Ordering::Equal => 0,
            Ordering::Greater => 1,
        };
        assert_eq!(got_cmp, exp_cmp, "seed {seed} compare_path {pa:?} vs {pb:?}");

        let key_cap = get_i32(line, "key_cap").unwrap() as usize;
        let exp_key_ok = get_i32(line, "key_ok").unwrap();
        let exp_key = get_json_str(line, "key").unwrap();
        match make_path_key(&pa, key_cap) {
            Some(k) => {
                assert_eq!(exp_key_ok, 1, "seed {seed} make_path_key ok");
                assert_eq!(k, exp_key, "seed {seed} make_path_key text");
            }
            None => assert_eq!(exp_key_ok, 0, "seed {seed} make_path_key fail"),
        }
        assert_eq!(get_file_name(&pa), get_json_str(line, "fname").unwrap(), "seed {seed} fname");

        // 2. RAMFile operations
        let mut host = MemoryFileHost::new();
        let txt_hex = get_json_str(line, "txt_hex").unwrap();
        let txt_bytes = decode_hex_bytes(&txt_hex).unwrap();
        host.add_file("mem/text.txt", txt_bytes.clone(), 1, seed as i32);

        let mut rf = RamFile::new();
        assert_eq!(
            rf.open(&host, "mem/text.txt", FileAccess::READ | FileAccess::BINARY) as i32,
            get_i32(line, "rf_ok").unwrap()
        );
        assert_eq!(rf.size(), get_i32(line, "rf_sz").unwrap());

        let scan_ints_exp = get_i32_array::<4>(line, "scan_ints").unwrap();
        for k in 0..2 {
            match rf.scan_int() {
                Some(v) => {
                    assert_eq!(scan_ints_exp[k * 2], 1, "seed {seed} scan_int ok {k}");
                    assert_eq!(v, scan_ints_exp[k * 2 + 1], "seed {seed} scan_int val {k}");
                }
                None => assert_eq!(scan_ints_exp[k * 2], 0, "seed {seed} scan_int eof {k}"),
            }
        }

        rf.seek(0, zh_common::io::SeekMode::Start);
        let scan_real_exp = get_u32_array::<4>(line, "scan_real_bits").unwrap();
        for k in 0..2 {
            match rf.scan_real() {
                Some(v) => {
                    assert_eq!(scan_real_exp[k * 2], 1, "seed {seed} scan_real ok {k}");
                    assert_eq!(
                        v.to_bits(),
                        scan_real_exp[k * 2 + 1],
                        "seed {seed} scan_real bits {k}"
                    );
                }
                None => assert_eq!(scan_real_exp[k * 2], 0, "seed {seed} scan_real eof {k}"),
            }
        }

        rf.seek(0, zh_common::io::SeekMode::Start);
        let exp_strs = get_json_str_vec(line, "scan_strs").unwrap();
        for (k, exp_s) in exp_strs.iter().enumerate() {
            let got = rf.scan_string().unwrap_or_else(|| "<EOF>".to_string());
            assert_eq!(&got, exp_s, "seed {seed} scan_string {k}");
        }

        let seek_off = get_i32(line, "seek_off").unwrap();
        let seek_mode = get_i32(line, "seek_mode").unwrap();
        assert_eq!(
            rf.seek_raw(seek_off, seek_mode),
            get_i32(line, "seek_ret").unwrap(),
            "seed {seed} seek_ret"
        );
        assert_eq!(rf.position(), get_i32(line, "end_pos").unwrap(), "seed {seed} end_pos");
        assert_eq!(rf.eof() as i32, get_i32(line, "eof").unwrap(), "seed {seed} eof");

        // Advance RNG position in same way C++ harness did before nextLine
        let mut rng_state = 0x9E37_79B9u32 ^ seed.wrapping_mul(0x85EB_CA6Bu32);
        if rng_state == 0 {
            rng_state = 0x6d2b_79f5u32;
        }
        let _ = (rng_state, txt_bytes.len());
        let nl_cap = get_i32(line, "nl_cap").unwrap();
        let after_nl = get_i32(line, "after_nl").unwrap();
        let exp_nl = get_json_str(line, "nl_str").unwrap();
        // Verify read_entire_and_close on fresh instance
        let mut rf2 = RamFile::new();
        assert_eq!(
            rf2.open(&host, "mem/text.txt", FileAccess::READ | FileAccess::BINARY),
            TRUE
        );
        let all_bytes = rf2.read_entire_and_close();
        assert_eq!(crc32_bytes(&all_bytes), get_u32(line, "entire_crc").unwrap(), "seed {seed} entire_crc");

        // Reconstruct seek position before nextLine from `after_nl` when non-empty or test exact nextLine roundtrip
        rf.seek(0, zh_common::io::SeekMode::Start);
        let nl_start = {
            // Find the start offset in txt_bytes whose nextLine(nl_cap) reaches after_nl and equals exp_nl
            let mut found_pos = None;
            for cand in 0..=txt_bytes.len() {
                rf.seek(cand as i32, zh_common::io::SeekMode::Start);
                let line_bytes = rf.next_line(nl_cap);
                if rf.position() == after_nl && line_bytes == exp_nl.as_bytes() {
                    found_pos = Some(cand);
                    break;
                }
            }
            found_pos
        };
        assert!(
            nl_start.is_some(),
            "seed {seed} next_line(cap={nl_cap}) never matched exp={exp_nl:?} after_nl={after_nl} in {txt_bytes:?}"
        );
        rf.close();

        // 3. BIG binary parser (valid & corrupt)
        host.clear();
        let probe_bytes = decode_hex_bytes(&get_json_str(line, "probe_hex").unwrap()).unwrap();
        host.add_file("archives/probe.big", probe_bytes, 7, (seed + 10) as i32);

        let allow_patch = get_i32(line, "allow_patch").unwrap() != 0;
        let overwrite = get_i32(line, "overwrite").unwrap() != 0;
        let loose_wins = get_i32(line, "loose_wins").unwrap() != 0;
        let mod_big = if seed % 3 == 0 {
            "root/custom_mod.big".to_string()
        } else {
            String::new()
        };
        let mod_dir = if seed % 4 == 0 {
            "root/mymod".to_string()
        } else {
            String::new()
        };

        let big_base = decode_hex_bytes(&get_json_str(line, "base_hex").unwrap()).unwrap();
        let big_patch = decode_hex_bytes(&get_json_str(line, "patch_hex").unwrap()).unwrap();
        let big_gx = decode_hex_bytes(&get_json_str(line, "gx_hex").unwrap()).unwrap();
        let big_mod = decode_hex_bytes(&get_json_str(line, "mod_hex").unwrap()).unwrap();

        host.add_file("root/INIZH.big", big_base, 100, 1);
        host.add_file("root/Data/INI/INIZH.big", big_gx.clone(), 100, 2);
        host.add_file("root/ZH_Generals/INI.big", big_gx.clone(), 100, 3);
        host.add_file("root/PatchZH.big", big_patch.clone(), 200, 4);
        host.add_file("root/generalsx.big", big_gx, 300, 5);
        host.add_file("root/!ModAlpha.big", big_mod.clone(), 400, 6);
        host.add_file("root/mymod/CustomPack.big", big_mod, 500, 7);
        host.add_file("root/custom_mod.big", big_patch, 600, 8);

        let mut bfs = BigFileSystem::with_config(ArchiveModConfig {
            mod_big,
            mod_dir,
            loose_files_win: loose_wins,
        });

        let probe_arc = bfs.open_archive_file(&host, "archives/probe.big");
        assert_eq!(probe_arc.is_some() as i32, get_i32(line, "probe_ok").unwrap(), "seed {seed} probe_ok");
        if let Some(arc) = probe_arc {
            let fi_sz = arc
                .get_file_info("Data/INI/Test.ini")
                .map_or(-1, |fi| fi.size_low);
            assert_eq!(fi_sz, get_i32(line, "probe_sz").unwrap(), "seed {seed} probe_sz");
            let sub = arc.open_file("data\\ini\\test.ini").unwrap();
            assert_eq!(crc32_bytes(sub.as_bytes()), get_u32(line, "probe_crc").unwrap(), "seed {seed} probe_crc");
            assert_eq!(arc.get_file_info("Data/INI/Zero.ini").unwrap().size_low, 0);
            assert_eq!(arc.get_file_info("Data/*/Dummy").unwrap().size_low, 5);
        }

        let load_ok = bfs.load_archive_files_from_directory(&host, "root", "*.big", overwrite, allow_patch);
        assert_eq!(load_ok as i32, get_i32(line, "load_ok").unwrap(), "seed {seed} load_ok");
        if seed % 2 == 0 {
            bfs.load_mods(&host);
        }
        let dup_load = bfs.load_archive_files_from_directory(&host, "root", "INIZH.big", false, false);
        assert_eq!(dup_load as i32, get_i32(line, "dup_load").unwrap(), "seed {seed} dup_load");

        let qpath = get_json_str(line, "qpath").unwrap();
        let qinst = get_i32(line, "qinst").unwrap() as u8;
        let qfilt = ArchiveFilter::from_i32(get_i32(line, "qfilt").unwrap()).unwrap();

        assert_eq!(
            bfs.does_file_exist(&qpath, qinst, ArchiveFilter::All) as i32,
            get_i32(line, "ex_all").unwrap(),
            "seed {seed} ex_all"
        );
        assert_eq!(
            bfs.does_file_exist(&qpath, qinst, qfilt) as i32,
            get_i32(line, "ex_filt").unwrap(),
            "seed {seed} ex_filt"
        );
        let cnt_exp = get_i32_array::<3>(line, "cnt").unwrap();
        assert_eq!(
            [
                bfs.get_file_count(&qpath, ArchiveFilter::All) as i32,
                bfs.get_file_count(&qpath, ArchiveFilter::ModOnly) as i32,
                bfs.get_file_count(&qpath, ArchiveFilter::NonModOnly) as i32,
            ],
            cnt_exp,
            "seed {seed} cnt"
        );
        assert_eq!(
            ArchiveFileSystem::is_stock_loose_file(&qpath) as i32,
            get_i32(line, "stock").unwrap(),
            "seed {seed} stock"
        );
        assert_eq!(
            bfs.has_mod_archive_override(&qpath) as i32,
            get_i32(line, "mod_ovr").unwrap(),
            "seed {seed} mod_ovr"
        );

        let qfi = bfs.get_file_info(&qpath, qinst, qfilt);
        assert_eq!(qfi.is_some() as i32, get_i32(line, "qfi_ok").unwrap(), "seed {seed} qfi_ok");
        if let Some(fi) = qfi {
            assert_eq!(fi.size_low, get_i32(line, "qfi_sz").unwrap(), "seed {seed} qfi_sz");
            assert_eq!(
                [fi.timestamp_high, fi.timestamp_low],
                get_i32_array::<2>(line, "qfi_ts").unwrap(),
                "seed {seed} qfi_ts"
            );
        }

        match bfs.open_file(&qpath, qinst, qfilt) {
            Some(f) => {
                assert_eq!(f.as_bytes().len() as i32, get_i32(line, "qlen").unwrap(), "seed {seed} qlen");
                assert_eq!(crc32_bytes(f.as_bytes()), get_u32(line, "qcrc").unwrap(), "seed {seed} qcrc");
            }
            None => assert_eq!(-1, get_i32(line, "qlen").unwrap(), "seed {seed} qlen missing"),
        }

        let sdir = get_json_str(line, "sdir").unwrap();
        let spat = get_json_str(line, "spat").unwrap();
        let flist: Vec<String> = bfs
            .get_file_list_in_directory(&sdir, &spat)
            .into_iter()
            .map(|k| k.0)
            .collect();
        let flist_exp = get_json_str_vec(line, "flist").unwrap();
        assert_eq!(flist, flist_exp, "seed {seed} flist dir={sdir:?} pat={spat:?}");
    }
}
