//! Gate V2-Golden test suite for `PKT-087` (`INI` parser & scanners, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-087`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

#![allow(dead_code)]

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_str, get_u32,
};
use zh_common::ini::{
    Color, DamageTypeFlags, DeathTypeFlags, FieldParseEntry, FieldUserData,
    GameClientRandomVariable, IniLoadType, IniReader, LookupListRec, MultiIniFieldParse,
    VeterancyLevelFlags,
};
use zh_common::math::{Coord2D, Coord3D, ICoord2D, RGBAColorInt, RGBColor};
use zh_common::types::{Bool, Byte, Int, Real, Short, UnsignedByte, UnsignedInt, UnsignedShort};
#[cfg(not(feature = "retail-crc"))]
use zh_common::xfer::XferCrc;

const PKT087_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-087.json");

fn rot_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

fn str_crc(mut crc: u32, s: &str) -> u32 {
    let bytes = s.as_bytes();
    crc = rot_crc(crc, bytes.len() as u32);
    for &b in bytes {
        crc = rot_crc(crc, b as u32);
    }
    crc
}

#[derive(Default)]
struct IniBlockSample {
    u8_val: UnsignedByte,
    i16_val: Short,
    u16_val: UnsignedShort,
    i32_val: Int,
    u32_val: UnsignedInt,
    f32_val: Real,
    pos_f32: Real,
    pct_val: Real,
    angle_val: Real,
    ang_vel: Real,
    vel_val: Real,
    accel_val: Real,
    dur_real: Real,
    dur_uint: UnsignedInt,
    dur_ushort: UnsignedShort,
    bool_val: Bool,
    bit32_mask: UnsignedInt,
    bit8_mask: Byte,
    bit32_flags: UnsignedInt,
    idx_val: Int,
    idx_u8: Byte,
    lookup_val: Int,
    ascii_val: String,
    quoted_val: String,
    vec_val: Vec<String>,
    sounds_val: Vec<String>,
    rgb: RGBColor,
    rgba_int: RGBAColorInt,
    color_int: Color,
    c2: Coord2D,
    c3: Coord3D,
    ic2: ICoord2D,
    rand_var: GameClientRandomVariable,
    vet_flags: VeterancyLevelFlags,
    dmg_flags: DamageTypeFlags,
    death_flags: DeathTypeFlags,
}

static TEST_LOOKUP: [LookupListRec; 4] = [
    LookupListRec { name: "ALPHA", value: 10 },
    LookupListRec { name: "BETA", value: -5 },
    LookupListRec { name: "GAMMA", value: 42 },
    LookupListRec { name: "DELTA", value: 1000 },
];

static TEST_FLAGS: [&str; 10] = [
    "FLAG_A", "FLAG_B", "FLAG_C", "FLAG_D", "FLAG_E", "FLAG_F", "FLAG_G", "FLAG_H", "FLAG_I",
    "FLAG_J",
];

static SAMPLE_PART1: [FieldParseEntry<IniBlockSample>; 17] = [
    FieldParseEntry {
        token: Some("U8Val"),
        parse: |ini, s, _| ini.parse_unsigned_byte(&mut s.u8_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("I16Val"),
        parse: |ini, s, _| ini.parse_short(&mut s.i16_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("U16Val"),
        parse: |ini, s, _| ini.parse_unsigned_short(&mut s.u16_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("I32Val"),
        parse: |ini, s, _| ini.parse_int(&mut s.i32_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("U32Val"),
        parse: |ini, s, _| ini.parse_unsigned_int(&mut s.u32_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("F32Val"),
        parse: |ini, s, _| ini.parse_real(&mut s.f32_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("PosF32"),
        parse: |ini, s, _| ini.parse_positive_non_zero_real(&mut s.pos_f32),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("PctVal"),
        parse: |ini, s, _| ini.parse_percent_to_real(&mut s.pct_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("AngleVal"),
        parse: |ini, s, _| ini.parse_angle_real(&mut s.angle_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("AngVel"),
        parse: |ini, s, _| ini.parse_angular_velocity_real(&mut s.ang_vel),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("VelVal"),
        parse: |ini, s, _| ini.parse_velocity_real(&mut s.vel_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("AccelVal"),
        parse: |ini, s, _| ini.parse_acceleration_real(&mut s.accel_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DurReal"),
        parse: |ini, s, _| ini.parse_duration_real(&mut s.dur_real),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DurUInt"),
        parse: |ini, s, _| ini.parse_duration_unsigned_int(&mut s.dur_uint),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DurUShort"),
        parse: |ini, s, _| ini.parse_duration_unsigned_short(&mut s.dur_ushort),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("BoolVal"),
        parse: |ini, s, _| ini.parse_bool(&mut s.bool_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("BitIn32"),
        parse: |ini, s, ud| match ud {
            FieldUserData::Mask(mask) => ini.parse_bit_in_int32(&mut s.bit32_mask, *mask),
            _ => unreachable!(),
        },
        user_data: FieldUserData::Mask(1 << 7),
    },
];

static SAMPLE_PART2: [FieldParseEntry<IniBlockSample>; 20] = [
    FieldParseEntry {
        token: Some("BitStr8"),
        parse: |ini, s, ud| match ud {
            FieldUserData::Names(names) => ini.parse_bit_string8(&mut s.bit8_mask, names),
            _ => unreachable!(),
        },
        user_data: FieldUserData::Names(&TEST_FLAGS),
    },
    FieldParseEntry {
        token: Some("BitStr32"),
        parse: |ini, s, ud| match ud {
            FieldUserData::Names(names) => ini.parse_bit_string32(&mut s.bit32_flags, names),
            _ => unreachable!(),
        },
        user_data: FieldUserData::Names(&TEST_FLAGS),
    },
    FieldParseEntry {
        token: Some("IdxList"),
        parse: |ini, s, ud| match ud {
            FieldUserData::Names(names) => ini.parse_index_list(&mut s.idx_val, names),
            _ => unreachable!(),
        },
        user_data: FieldUserData::Names(&TEST_FLAGS),
    },
    FieldParseEntry {
        token: Some("IdxU8"),
        parse: |ini, s, ud| match ud {
            FieldUserData::Names(names) => ini.parse_byte_sized_index_list(&mut s.idx_u8, names),
            _ => unreachable!(),
        },
        user_data: FieldUserData::Names(&TEST_FLAGS),
    },
    FieldParseEntry {
        token: Some("Lookup"),
        parse: |ini, s, ud| match ud {
            FieldUserData::Lookup(rec) => ini.parse_lookup_list(&mut s.lookup_val, rec),
            _ => unreachable!(),
        },
        user_data: FieldUserData::Lookup(&TEST_LOOKUP),
    },
    FieldParseEntry {
        token: Some("AsciiVal"),
        parse: |ini, s, _| ini.parse_ascii_string(&mut s.ascii_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("QuotedVal"),
        parse: |ini, s, _| ini.parse_quoted_ascii_string(&mut s.quoted_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("VecVal"),
        parse: |ini, s, _| ini.parse_ascii_string_vector(&mut s.vec_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("VecAppend"),
        parse: |ini, s, _| ini.parse_ascii_string_vector_append(&mut s.vec_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("SoundsVal"),
        parse: |ini, s, _| ini.parse_sounds_list(&mut s.sounds_val),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Rgb"),
        parse: |ini, s, _| ini.parse_rgb_color(&mut s.rgb),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("RgbaInt"),
        parse: |ini, s, _| ini.parse_rgba_color_int(&mut s.rgba_int),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ColorInt"),
        parse: |ini, s, _| ini.parse_color_int(&mut s.color_int),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Coord2"),
        parse: |ini, s, _| ini.parse_coord2d(&mut s.c2),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("Coord3"),
        parse: |ini, s, _| ini.parse_coord3d(&mut s.c3),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("ICoord2"),
        parse: |ini, s, _| ini.parse_icoord2d(&mut s.ic2),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("RandVar"),
        parse: |ini, s, _| ini.parse_game_client_random_variable(&mut s.rand_var),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("VetFlags"),
        parse: |ini, s, _| ini.parse_veterancy_level_flags(&mut s.vet_flags),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DmgFlags"),
        parse: |ini, s, _| ini.parse_damage_type_flags(&mut s.dmg_flags),
        user_data: FieldUserData::None,
    },
    FieldParseEntry {
        token: Some("DeathFlags"),
        parse: |ini, s, _| ini.parse_death_type_flags(&mut s.death_flags),
        user_data: FieldUserData::None,
    },
];

fn sample_crc(s: &IniBlockSample) -> u32 {
    let mut crc = 0u32;
    crc = rot_crc(crc, s.u8_val as u32);
    crc = rot_crc(crc, s.i16_val as i32 as u32);
    crc = rot_crc(crc, s.u16_val as u32);
    crc = rot_crc(crc, s.i32_val as u32);
    crc = rot_crc(crc, s.u32_val);
    crc = rot_crc(crc, s.f32_val.to_bits());
    crc = rot_crc(crc, s.pos_f32.to_bits());
    crc = rot_crc(crc, s.pct_val.to_bits());
    crc = rot_crc(crc, s.angle_val.to_bits());
    crc = rot_crc(crc, s.ang_vel.to_bits());
    crc = rot_crc(crc, s.vel_val.to_bits());
    crc = rot_crc(crc, s.accel_val.to_bits());
    crc = rot_crc(crc, s.dur_real.to_bits());
    crc = rot_crc(crc, s.dur_uint);
    crc = rot_crc(crc, s.dur_ushort as u32);
    crc = rot_crc(crc, s.bool_val as u32);
    crc = rot_crc(crc, s.bit32_mask);
    crc = rot_crc(crc, (s.bit8_mask as i8 as i32) as u32);
    crc = rot_crc(crc, s.bit32_flags);
    crc = rot_crc(crc, s.idx_val as u32);
    crc = rot_crc(crc, (s.idx_u8 as i8 as i32) as u32);
    crc = rot_crc(crc, s.lookup_val as u32);
    crc = str_crc(crc, &s.ascii_val);
    crc = str_crc(crc, &s.quoted_val);
    crc = rot_crc(crc, s.vec_val.len() as u32);
    for v in &s.vec_val {
        crc = str_crc(crc, v);
    }
    crc = rot_crc(crc, s.sounds_val.len() as u32);
    for v in &s.sounds_val {
        crc = str_crc(crc, v);
    }
    crc = rot_crc(crc, s.rgb.red.to_bits());
    crc = rot_crc(crc, s.rgb.green.to_bits());
    crc = rot_crc(crc, s.rgb.blue.to_bits());
    crc = rot_crc(crc, s.rgba_int.red);
    crc = rot_crc(crc, s.rgba_int.green);
    crc = rot_crc(crc, s.rgba_int.blue);
    crc = rot_crc(crc, s.rgba_int.alpha);
    crc = rot_crc(crc, s.color_int);
    crc = rot_crc(crc, s.c2.x.to_bits());
    crc = rot_crc(crc, s.c2.y.to_bits());
    crc = rot_crc(crc, s.c3.x.to_bits());
    crc = rot_crc(crc, s.c3.y.to_bits());
    crc = rot_crc(crc, s.c3.z.to_bits());
    crc = rot_crc(crc, s.ic2.x as u32);
    crc = rot_crc(crc, s.ic2.y as u32);
    crc = rot_crc(crc, (s.rand_var.dist_type as i32) as u32);
    crc = rot_crc(crc, s.rand_var.low.to_bits());
    crc = rot_crc(crc, s.rand_var.high.to_bits());
    crc = rot_crc(crc, s.vet_flags);
    for b in 0..38 {
        crc = rot_crc(crc, s.dmg_flags.test(b) as u32);
    }
    crc = rot_crc(crc, s.death_flags);
    crc
}

#[test]
#[ignore = "Un-ignore when implementing PKT-087 (crates/zh_common/src/ini/mod.rs)"]
fn test_pkt087_ini_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT087_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-087.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let itok = get_str(line, "itok").expect("itok");
        match IniReader::scan_int(itok) {
            Ok(val) => {
                assert_eq!(get_u32(line, "i_ok"), Some(1), "seed {} itok={}", seed, itok);
                assert_eq!(val, get_i32(line, "i_val").unwrap(), "seed {} i_val", seed);
            }
            Err(_) => {
                assert_eq!(get_u32(line, "i_ok"), Some(0), "seed {} itok={}", seed, itok);
            }
        }

        let utok = get_str(line, "utok").expect("utok");
        match IniReader::scan_unsigned_int(utok) {
            Ok(val) => {
                assert_eq!(get_u32(line, "u_ok"), Some(1), "seed {} utok={}", seed, utok);
                assert_eq!(val, get_u32(line, "u_val").unwrap(), "seed {} u_val", seed);
            }
            Err(_) => {
                assert_eq!(get_u32(line, "u_ok"), Some(0), "seed {} utok={}", seed, utok);
            }
        }

        let rtok = get_str(line, "rtok").expect("rtok");
        match IniReader::scan_real(rtok) {
            Ok(val) => {
                assert_eq!(get_u32(line, "r_ok"), Some(1), "seed {} rtok={}", seed, rtok);
                assert_eq!(val.to_bits(), get_u32(line, "r_bits").unwrap(), "seed {} r_bits", seed);
            }
            Err(_) => {
                assert_eq!(get_u32(line, "r_ok"), Some(0), "seed {} rtok={}", seed, rtok);
            }
        }

        let btok = get_str(line, "btok").expect("btok");
        match IniReader::scan_bool(btok) {
            Ok(val) => {
                assert_eq!(get_u32(line, "b_ok"), Some(1), "seed {} btok={}", seed, btok);
                assert_eq!(val as u32, get_u32(line, "b_val").unwrap(), "seed {} b_val", seed);
            }
            Err(_) => {
                assert_eq!(get_u32(line, "b_ok"), Some(0), "seed {} btok={}", seed, btok);
            }
        }

        let dline = decode_hex_bytes(get_str(line, "dline_hex").unwrap()).unwrap();
        assert_eq!(
            IniReader::is_declaration_of_type("Object", "AmericaTankCrusader", &dline) as u32,
            get_u32(line, "is_decl").unwrap(),
            "seed {} is_decl",
            seed
        );
        assert_eq!(
            IniReader::is_end_of_block(&dline) as u32,
            get_u32(line, "is_end").unwrap(),
            "seed {} is_end",
            seed
        );

        let ini_bytes = decode_hex_bytes(get_str(line, "ini_hex").unwrap()).unwrap();
        let mut sample = IniBlockSample {
            bit8_mask: get_u32(line, "bit8_init").unwrap() as u8 as i8,
            bit32_mask: get_u32(line, "bit32_init").unwrap(),
            ..IniBlockSample::default()
        };
        let mut reader = IniReader::new("Data\\INI\\Test.ini", &ini_bytes, IniLoadType::Overwrite);

        #[cfg(not(feature = "retail-crc"))]
        {
            let mut xfer_crc = XferCrc::new();
            let res = reader.load_blocks(Some(&mut xfer_crc), |block, r| {
                assert_eq!(block, "GameData");
                let mut multi = MultiIniFieldParse::new();
                multi.add(&SAMPLE_PART1)?;
                multi.add(&SAMPLE_PART2)?;
                r.init_from_ini_multi(&mut sample, &multi)
            });
            assert_eq!(res.is_ok() as u32, get_u32(line, "load_ok").unwrap(), "seed {} load_ok", seed);
            assert_eq!(xfer_crc.get_crc(), get_u32(line, "line_crc").unwrap(), "seed {} line_crc", seed);
        }
        #[cfg(feature = "retail-crc")]
        {
            let res = reader.load_blocks(None, |block, r| {
                assert_eq!(block, "GameData");
                let mut multi = MultiIniFieldParse::new();
                multi.add(&SAMPLE_PART1)?;
                multi.add(&SAMPLE_PART2)?;
                r.init_from_ini_multi(&mut sample, &multi)
            });
            assert_eq!(res.is_ok() as u32, get_u32(line, "load_ok").unwrap(), "seed {} load_ok", seed);
        }

        assert_eq!(sample_crc(&sample), get_u32(line, "out_crc").unwrap(), "seed {} out_crc", seed);
    }
}
