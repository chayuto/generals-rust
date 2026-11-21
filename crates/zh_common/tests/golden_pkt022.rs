//! Gate V2-Golden test suite for `PKT-022` (`Geometry`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-022`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

use zh_common::golden_json::{extract_vector_lines, get_i32, get_u32, get_u32_array};
use zh_common::math::geometry::{GeometryInfo, GeometryType};
use zh_common::math::{Coord3D, Region2D};
use zh_common::random_value::PrngState;
use zh_common::types::Bool;
use zh_common::xfer::{Snapshot, XferLoad, XferSave};
#[cfg(not(feature = "retail-crc"))]
use zh_common::xfer::XferCrc;

const PKT022_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-022.json");

#[test]
fn test_pkt022_geometry_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT022_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-022.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let gtype = GeometryType::from_i32(get_i32(line, "gtype").expect("gtype"));
        let is_small = get_u32(line, "is_small").expect("is_small") as Bool;
        let height = f32::from_bits(get_u32(line, "height_bits").expect("height_bits"));
        let major_r = f32::from_bits(get_u32(line, "major_bits").expect("major_bits"));
        let minor_r = f32::from_bits(get_u32(line, "minor_bits").expect("minor_bits"));
        let angle = f32::from_bits(get_u32(line, "angle_bits").expect("angle_bits"));

        let mut rng = PrngState::new();
        rng.init_random_with_seed(seed + 1);

        let mut geom = GeometryInfo::new(gtype, is_small, height, major_r, minor_r);
        if (seed & 7) == 3 {
            geom.expand_footprint(2.5);
        } else if (seed & 7) == 5 {
            geom.set_max_height_above_position(height + 4.0);
        }

        let cin = get_u32_array::<3>(line, "center_in_bits").expect("center_in_bits");
        let center_in = Coord3D::new(
            f32::from_bits(cin[0]),
            f32::from_bits(cin[1]),
            f32::from_bits(cin[2]),
        );
        let mut center_out = Coord3D::default();
        geom.get_center_position(&center_in, &mut center_out);

        let mut bounds2d = Region2D::default();
        geom.get_2d_bounds(&center_in, angle, &mut bounds2d);

        let tpt = get_u32_array::<3>(line, "test_pt_bits").expect("test_pt_bits");
        let test_pt = Coord3D::new(
            f32::from_bits(tpt[0]),
            f32::from_bits(tpt[1]),
            f32::from_bits(tpt[2]),
        );
        let in_fp = geom.is_point_in_footprint(&center_in, &test_pt);
        let mut clipped_pt = test_pt;
        geom.clip_point_to_footprint(&center_in, &mut clipped_pt);

        let sfr = get_u32_array::<3>(line, "seg_from_bits").expect("seg_from_bits");
        let sto = get_u32_array::<3>(line, "seg_to_bits").expect("seg_to_bits");
        let seg_from = Coord3D::new(
            f32::from_bits(sfr[0]),
            f32::from_bits(sfr[1]),
            f32::from_bits(sfr[2]),
        );
        let seg_to = Coord3D::new(
            f32::from_bits(sto[0]),
            f32::from_bits(sto[1]),
            f32::from_bits(sto[2]),
        );
        let hit_seg = geom.is_intersected_by_line_segment(&center_in, &seg_from, &seg_to);

        let that_type = GeometryType::from_i32(((seed + 1) % 3) as i32);
        let that_geom = GeometryInfo::new(that_type, 0, height * 0.75, major_r * 0.5, minor_r * 0.5);
        let that_pos = Coord3D::new(center_in.x + 25.0, center_in.y - 15.0, center_in.z + 10.0);
        let mut min_pitch = 0.0f32;
        let mut max_pitch = 0.0f32;
        geom.calc_pitches(&center_in, &that_geom, &that_pos, &mut min_pitch, &mut max_pitch);

        assert_eq!(geom.get_bounding_circle_radius().to_bits(), get_u32(line, "bcircle_bits").unwrap(), "seed {seed} bcircle");
        assert_eq!(geom.get_bounding_sphere_radius().to_bits(), get_u32(line, "bsphere_bits").unwrap(), "seed {seed} bsphere");
        assert_eq!(geom.get_footprint_area().to_bits(), get_u32(line, "area_bits").unwrap(), "seed {seed} area");
        assert_eq!(geom.get_max_height_above_position().to_bits(), get_u32(line, "max_above_bits").unwrap(), "seed {seed} max_above");
        assert_eq!(geom.get_max_height_below_position().to_bits(), get_u32(line, "max_below_bits").unwrap(), "seed {seed} max_below");
        assert_eq!(geom.get_z_delta_to_center_position().to_bits(), get_u32(line, "zdelta_bits").unwrap(), "seed {seed} zdelta");
        assert_eq!(center_out.z.to_bits(), get_u32(line, "center_z_bits").unwrap(), "seed {seed} center_z");

        let want_b = get_u32_array::<4>(line, "bounds_bits").unwrap();
        assert_eq!(bounds2d.lo.x.to_bits(), want_b[0], "seed {seed} bounds.lo.x");
        assert_eq!(bounds2d.lo.y.to_bits(), want_b[1], "seed {seed} bounds.lo.y");
        assert_eq!(bounds2d.hi.x.to_bits(), want_b[2], "seed {seed} bounds.hi.x");
        assert_eq!(bounds2d.hi.y.to_bits(), want_b[3], "seed {seed} bounds.hi.y");

        assert_eq!(in_fp as u32, get_u32(line, "in_fp").unwrap(), "seed {seed} in_fp");
        let want_clip = get_u32_array::<2>(line, "clip_bits").unwrap();
        assert_eq!(clipped_pt.x.to_bits(), want_clip[0], "seed {seed} clip.x");
        assert_eq!(clipped_pt.y.to_bits(), want_clip[1], "seed {seed} clip.y");
        assert_eq!(hit_seg as u32, get_u32(line, "hit_seg").unwrap(), "seed {seed} hit_seg");

        assert_eq!(min_pitch.to_bits(), get_u32(line, "min_pitch_bits").unwrap(), "seed {seed} min_pitch");
        assert_eq!(max_pitch.to_bits(), get_u32(line, "max_pitch_bits").unwrap(), "seed {seed} max_pitch");

        let mut rand_fp = Coord3D::default();
        geom.make_random_offset_within_footprint(&mut rand_fp, &mut rng);
        let mut rand_perim = Coord3D::default();
        geom.make_random_offset_on_perimeter(&mut rand_perim, &mut rng);

        #[cfg(not(feature = "retail-crc"))]
        {
            let want_rfp = get_u32_array::<2>(line, "rand_fp_bits").unwrap();
            assert_eq!(rand_fp.x.to_bits(), want_rfp[0], "seed {seed} rand_fp.x");
            assert_eq!(rand_fp.y.to_bits(), want_rfp[1], "seed {seed} rand_fp.y");
            let want_rp = get_u32_array::<2>(line, "rand_perim_bits").unwrap();
            assert_eq!(rand_perim.x.to_bits(), want_rp[0], "seed {seed} rand_perim.x");
            assert_eq!(rand_perim.y.to_bits(), want_rp[1], "seed {seed} rand_perim.y");
            assert_eq!(rng.get_game_logic_random_seed_crc(), get_u32(line, "rng_crc").unwrap(), "seed {seed} rng_crc");

            let mut xcrc = XferCrc::new();
            geom.xfer(&mut xcrc).unwrap();
            assert_eq!(xcrc.get_crc(), get_u32(line, "xfer_crc").unwrap(), "seed {seed} xfer_crc");
        }

        let mut xsave = XferSave::new();
        geom.xfer(&mut xsave).unwrap();
        let mut loaded = GeometryInfo::default();
        let mut xload = XferLoad::new(xsave.as_bytes());
        loaded.xfer(&mut xload).unwrap();
        assert_eq!(loaded, geom, "seed {seed} XferSave/XferLoad round-trip");
    }
}
