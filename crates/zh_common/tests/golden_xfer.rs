//! Gate V2-Golden test suite for `WP-2.1` (`Xfer`, `XferCrc`, `XferDeepCrc`, `XferSave`,
//! `XferLoad`, and `Snapshot`, 1,000 C++ reference vectors generated from unmodified
//! `vendor/GeneralsX/Core/GameEngine/Source/Common/System/Xfer*.cpp`).

use zh_common::golden_json::{
    decode_hex_bytes, extract_vector_lines, get_i32, get_i32_array, get_i32_vec, get_i64, get_str,
    get_u16_vec, get_u32, get_u32_array, get_u64,
};
use zh_common::math::{
    Coord2D, Coord3D, ICoord2D, ICoord3D, IRegion2D, IRegion3D, Matrix3D, RGBAColorInt,
    RGBAColorReal, RGBColor, RealRange, Region2D, Region3D,
};
use zh_common::types::{
    Bool, Byte, DrawableId, Int, ObjectId, Real, Short, UnsignedByte, UnsignedInt, UnsignedShort,
    XferVersion,
};
use zh_common::xfer::{
    Snapshot, Xfer, XferCrc, XferDeepCrc, XferError, XferLoad, XferMode, XferSave,
};

const XFER_JSON: &str = include_str!("../../../docs/golden_vectors/WP2_1_Xfer.json");

#[derive(Debug, Clone, PartialEq)]
struct CompositeRecord {
    ver: XferVersion,
    b_val: Byte,
    ub_val: UnsignedByte,
    bool_val: Bool,
    s_val: Short,
    us_val: UnsignedShort,
    i_val: Int,
    i64_val: i64,
    ui_val: UnsignedInt,
    r_val: Real,
    d_val: f64,
    ascii: String,
    unicode: Vec<u16>,
    c2: Coord2D,
    ic2: ICoord2D,
    reg2: Region2D,
    ireg2: IRegion2D,
    c3: Coord3D,
    ic3: ICoord3D,
    reg3: Region3D,
    ireg3: IRegion3D,
    rr: RealRange,
    color: Int,
    rgb: RGBColor,
    rgba_real: RGBAColorReal,
    rgba_int: RGBAColorInt,
    obj_id: ObjectId,
    draw_id: DrawableId,
    obj_vec: Vec<ObjectId>,
    int_list: Vec<Int>,
    mtx: Matrix3D,
    user_tail: Vec<u8>,
}

impl Default for CompositeRecord {
    fn default() -> Self {
        Self {
            ver: 1,
            b_val: 0,
            ub_val: 0,
            bool_val: 0,
            s_val: 0,
            us_val: 0,
            i_val: 0,
            i64_val: 0,
            ui_val: 0,
            r_val: 0.0,
            d_val: 0.0,
            ascii: String::new(),
            unicode: Vec::new(),
            c2: Coord2D::default(),
            ic2: ICoord2D::default(),
            reg2: Region2D::default(),
            ireg2: IRegion2D::default(),
            c3: Coord3D::default(),
            ic3: ICoord3D::default(),
            reg3: Region3D::default(),
            ireg3: IRegion3D::default(),
            rr: RealRange::default(),
            color: 0,
            rgb: RGBColor::default(),
            rgba_real: RGBAColorReal::default(),
            rgba_int: RGBAColorInt::default(),
            obj_id: ObjectId::INVALID,
            draw_id: DrawableId::INVALID,
            obj_vec: Vec::new(),
            int_list: Vec::new(),
            mtx: Matrix3D::default(),
            user_tail: Vec::new(),
        }
    }
}

impl CompositeRecord {
    fn visit(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        xfer.xfer_marker_label("CompositeRecord");
        xfer.xfer_byte(&mut self.b_val)?;
        xfer.xfer_unsigned_byte(&mut self.ub_val)?;
        xfer.xfer_bool(&mut self.bool_val)?;
        xfer.xfer_short(&mut self.s_val)?;
        xfer.xfer_unsigned_short(&mut self.us_val)?;
        xfer.xfer_int(&mut self.i_val)?;
        xfer.xfer_int64(&mut self.i64_val)?;
        xfer.xfer_unsigned_int(&mut self.ui_val)?;
        xfer.xfer_real(&mut self.r_val)?;
        xfer.xfer_double(&mut self.d_val)?;
        xfer.xfer_ascii_string(&mut self.ascii)?;
        xfer.xfer_unicode_string(&mut self.unicode)?;
        xfer.xfer_coord2d(&mut self.c2)?;
        xfer.xfer_icoord2d(&mut self.ic2)?;
        xfer.xfer_region2d(&mut self.reg2)?;
        xfer.xfer_iregion2d(&mut self.ireg2)?;
        xfer.xfer_coord3d(&mut self.c3)?;
        xfer.xfer_icoord3d(&mut self.ic3)?;
        xfer.xfer_region3d(&mut self.reg3)?;
        xfer.xfer_iregion3d(&mut self.ireg3)?;
        xfer.xfer_real_range(&mut self.rr)?;
        xfer.xfer_color(&mut self.color)?;
        xfer.xfer_rgb_color(&mut self.rgb)?;
        xfer.xfer_rgba_color_real(&mut self.rgba_real)?;
        xfer.xfer_rgba_color_int(&mut self.rgba_int)?;
        xfer.xfer_object_id(&mut self.obj_id)?;
        xfer.xfer_drawable_id(&mut self.draw_id)?;
        let block_sz = xfer.begin_block()?;
        if xfer.mode() == XferMode::Load {
            assert!(block_sz > 0);
        }
        xfer.xfer_stl_object_id_vec(&mut self.obj_vec)?;
        xfer.xfer_stl_int_list(&mut self.int_list)?;
        xfer.xfer_matrix3d(&mut self.mtx)?;
        let mut tail_n = self.user_tail.len() as UnsignedByte;
        xfer.xfer_unsigned_byte(&mut tail_n)?;
        if xfer.mode() == XferMode::Load {
            self.user_tail.resize(tail_n as usize, 0);
        }
        if tail_n > 0 {
            xfer.xfer_user(&mut self.user_tail)?;
        }
        xfer.end_block()?;
        Ok(())
    }
}

impl Snapshot for CompositeRecord {
    fn crc(&self, xfer: &mut dyn Xfer) {
        let mut clone = self.clone();
        clone.visit(xfer).expect("CRC xfer must succeed");
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut v = self.ver;
        xfer.xfer_version(&mut v, 1)?;
        self.ver = v;
        self.visit(xfer)
    }
}

#[test]
fn test_wp2_1_xfer_1000_golden_vectors() {
    let lines = extract_vector_lines(XFER_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in WP2_1_Xfer.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let raw_bytes = decode_hex_bytes(get_str(line, "raw_hex").expect("raw_hex")).expect("hex");

        let mut raw_xcrc = XferCrc::new();
        raw_xcrc.xfer_user(&mut raw_bytes.clone()).unwrap();

        #[cfg(not(feature = "retail-crc"))]
        assert_eq!(raw_xcrc.get_crc(), get_u32(line, "raw_crc").unwrap(), "seed {seed} raw_crc");
        #[cfg(feature = "retail-crc")]
        assert_eq!(
            raw_xcrc.get_crc(),
            get_u32(line, "retail_raw_crc").unwrap(),
            "seed {seed} retail_raw_crc"
        );

        let c2 = get_u32_array::<2>(line, "c2").unwrap();
        let ic2 = get_i32_array::<2>(line, "ic2").unwrap();
        let reg2 = get_u32_array::<4>(line, "reg2").unwrap();
        let ireg2 = get_i32_array::<4>(line, "ireg2").unwrap();
        let c3 = get_u32_array::<3>(line, "c3").unwrap();
        let ic3 = get_i32_array::<3>(line, "ic3").unwrap();
        let reg3 = get_u32_array::<6>(line, "reg3").unwrap();
        let ireg3 = get_i32_array::<6>(line, "ireg3").unwrap();
        let rr = get_u32_array::<2>(line, "rr").unwrap();
        let rgba_int = get_u32_array::<4>(line, "rgba_int").unwrap();
        let obj_vec = get_i32_vec(line, "obj_vec")
            .unwrap()
            .into_iter()
            .map(ObjectId)
            .collect();
        let int_list = get_i32_vec(line, "int_list").unwrap();
        let mtx_u = get_u32_array::<12>(line, "mtx").unwrap();
        let mtx = Matrix3D {
            rows: [
                [
                    f32::from_bits(mtx_u[0]),
                    f32::from_bits(mtx_u[1]),
                    f32::from_bits(mtx_u[2]),
                    f32::from_bits(mtx_u[3]),
                ],
                [
                    f32::from_bits(mtx_u[4]),
                    f32::from_bits(mtx_u[5]),
                    f32::from_bits(mtx_u[6]),
                    f32::from_bits(mtx_u[7]),
                ],
                [
                    f32::from_bits(mtx_u[8]),
                    f32::from_bits(mtx_u[9]),
                    f32::from_bits(mtx_u[10]),
                    f32::from_bits(mtx_u[11]),
                ],
            ],
        };

        let mut rec = CompositeRecord {
            ver: 1,
            b_val: get_i32(line, "b").unwrap() as i8,
            ub_val: get_u32(line, "ub").unwrap() as u8,
            bool_val: get_u32(line, "bool_v").unwrap() as u8,
            s_val: get_i32(line, "s").unwrap() as i16,
            us_val: get_u32(line, "us").unwrap() as u16,
            i_val: get_i32(line, "i").unwrap(),
            i64_val: get_i64(line, "i64").unwrap(),
            ui_val: get_u32(line, "ui").unwrap(),
            r_val: f32::from_bits(get_u32(line, "r_bits").unwrap()),
            d_val: f64::from_bits(get_u64(line, "d_bits").unwrap()),
            ascii: get_str(line, "ascii").unwrap().to_owned(),
            unicode: get_u16_vec(line, "u16").unwrap(),
            c2: Coord2D::new(f32::from_bits(c2[0]), f32::from_bits(c2[1])),
            ic2: ICoord2D::new(ic2[0], ic2[1]),
            reg2: Region2D {
                lo: Coord2D::new(f32::from_bits(reg2[0]), f32::from_bits(reg2[1])),
                hi: Coord2D::new(f32::from_bits(reg2[2]), f32::from_bits(reg2[3])),
            },
            ireg2: IRegion2D {
                lo: ICoord2D::new(ireg2[0], ireg2[1]),
                hi: ICoord2D::new(ireg2[2], ireg2[3]),
            },
            c3: Coord3D::new(
                f32::from_bits(c3[0]),
                f32::from_bits(c3[1]),
                f32::from_bits(c3[2]),
            ),
            ic3: ICoord3D::new(ic3[0], ic3[1], ic3[2]),
            reg3: Region3D {
                lo: Coord3D::new(
                    f32::from_bits(reg3[0]),
                    f32::from_bits(reg3[1]),
                    f32::from_bits(reg3[2]),
                ),
                hi: Coord3D::new(
                    f32::from_bits(reg3[3]),
                    f32::from_bits(reg3[4]),
                    f32::from_bits(reg3[5]),
                ),
            },
            ireg3: IRegion3D {
                lo: ICoord3D::new(ireg3[0], ireg3[1], ireg3[2]),
                hi: ICoord3D::new(ireg3[3], ireg3[4], ireg3[5]),
            },
            rr: RealRange::new(f32::from_bits(rr[0]), f32::from_bits(rr[1])),
            color: get_i32(line, "color").unwrap(),
            rgb: RGBColor {
                red: 0.25,
                green: 0.5,
                blue: 0.75,
            },
            rgba_real: RGBAColorReal {
                red: 0.125,
                green: 0.375,
                blue: 0.625,
                alpha: 0.875,
            },
            rgba_int: RGBAColorInt {
                red: rgba_int[0],
                green: rgba_int[1],
                blue: rgba_int[2],
                alpha: rgba_int[3],
            },
            obj_id: ObjectId(get_i32(line, "obj_id").unwrap()),
            draw_id: DrawableId(get_i32(line, "draw_id").unwrap()),
            obj_vec,
            int_list,
            mtx,
            user_tail: decode_hex_bytes(get_str(line, "user_tail_hex").unwrap()).unwrap(),
        };

        let mut rec_xcrc = XferCrc::new();
        rec_xcrc.xfer_snapshot(&mut rec).unwrap();

        let mut xsave = XferSave::new();
        xsave.xfer_snapshot(&mut rec).unwrap();
        let saved = xsave.into_bytes();
        assert_eq!(saved.len() as u32, get_u32(line, "save_len").unwrap(), "seed {seed} save_len");

        let mut loaded = CompositeRecord::default();
        let mut xload = XferLoad::new(&saved);
        xload.xfer_snapshot(&mut loaded).unwrap();
        assert_eq!(loaded, rec, "seed {seed} XferLoad round-trip");
        assert_eq!(xload.position(), saved.len(), "seed {seed} XferLoad EOF pos");

        let mut deep = XferDeepCrc::new();
        deep.xfer_snapshot(&mut rec).unwrap();
        assert_eq!(
            deep.as_bytes().len() as u32,
            get_u32(line, "deep_len").unwrap(),
            "seed {seed} deep_len"
        );

        #[cfg(not(feature = "retail-crc"))]
        {
            assert_eq!(rec_xcrc.get_crc(), get_u32(line, "rec_crc").unwrap(), "seed {seed} rec_crc");

            let mut save_xcrc = XferCrc::new();
            save_xcrc.xfer_implementation(&saved);
            assert_eq!(
                save_xcrc.get_crc(),
                get_u32(line, "save_bytes_crc").unwrap(),
                "seed {seed} save_bytes_crc"
            );

            assert_eq!(deep.get_crc(), get_u32(line, "deep_crc").unwrap(), "seed {seed} deep_crc");

            let mut deep_bytes_xcrc = XferCrc::new();
            deep_bytes_xcrc.xfer_implementation(deep.as_bytes());
            assert_eq!(
                deep_bytes_xcrc.get_crc(),
                get_u32(line, "deep_bytes_crc").unwrap(),
                "seed {seed} deep_bytes_crc"
            );
        }

        #[cfg(feature = "retail-crc")]
        assert_eq!(
            deep.get_crc(),
            get_u32(line, "retail_deep_crc").unwrap(),
            "seed {seed} retail_deep_crc"
        );
    }
}

#[test]
fn test_wp2_1_xfer_blocks_skip_and_errors() {
    let mut xs = XferSave::new();
    xs.begin_block().unwrap();
    let mut x = 0x1122_3344_i32;
    xs.xfer_int(&mut x).unwrap();
    xs.begin_block().unwrap();
    let mut s = "Hi!".to_owned();
    xs.xfer_ascii_string(&mut s).unwrap();
    xs.end_block().unwrap();
    xs.end_block().unwrap();
    assert_eq!(xs.end_block(), Err(XferError::BeginEndMismatch));

    let buf = xs.into_bytes();
    let mut xl = XferLoad::new(&buf);
    let outer_sz = xl.begin_block().unwrap();
    assert_eq!(outer_sz as usize, buf.len() - 4);
    let mut y = 0i32;
    xl.xfer_int(&mut y).unwrap();
    assert_eq!(y, 0x1122_3344);
    let inner_sz = xl.begin_block().unwrap();
    assert_eq!(inner_sz, 4); // 1 byte len + 3 bytes "Hi!"
    xl.skip(inner_sz).unwrap();
    xl.end_block().unwrap();
    xl.end_block().unwrap();
    assert_eq!(xl.position(), buf.len());
    assert_eq!(xl.skip(1), Err(XferError::SkipOutOfBounds));

    // Version check and ListNotEmpty check
    let mut ver: XferVersion = 3;
    let mut bad_crc = XferCrc::new();
    assert_eq!(
        bad_crc.xfer_version(&mut ver, 2),
        Err(XferError::VersionTooNew {
            found: 3,
            max_supported: 2,
        })
    );

    let mut xs2 = XferSave::new();
    let mut ids = vec![ObjectId(7)];
    xs2.xfer_stl_object_id_vec(&mut ids).unwrap();
    let b2 = xs2.into_bytes();
    let mut non_empty = vec![ObjectId(99)];
    let mut xl2 = XferLoad::new(&b2);
    assert_eq!(xl2.xfer_stl_object_id_vec(&mut non_empty), Err(XferError::ListNotEmpty));
}
