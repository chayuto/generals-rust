//! Gate V2-Golden test suite for `PKT-015` + `PKT-016` (`BezFwdIterator` & `BezierSegment`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-015` and `PKT-016`, remove the `#[ignore]` attribute below so this suite
//! runs automatically in `cargo test --workspace` and CI!

use zh_common::golden_json::{extract_vector_lines, get_i32, get_u32, get_u32_array};
use zh_common::math::bezier_segment::BezierSegment;
use zh_common::math::Coord3D;

const PKT016_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-016.json");

fn rotate_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[test]
#[ignore = "Un-ignore when implementing PKT-015 & PKT-016 (bez_fwd_iterator.rs & bezier_segment.rs)"]
fn test_pkt015_016_bezier_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT016_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-016.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let cp_bits: [u32; 12] = get_u32_array(line, "cp_bits").expect("cp_bits");
        let mut cp = [0.0f32; 12];
        for i in 0..12 {
            cp[i] = f32::from_bits(cp_bits[i]);
        }
        let t = f32::from_bits(get_u32(line, "t_bits").expect("t_bits"));
        let num_segments = get_i32(line, "num_segments").expect("num_segments");

        let seg = BezierSegment::from_coords(&cp);
        let mut eval_pt = Coord3D::default();
        seg.evaluate_bez_segment_at_t(t, &mut eval_pt);
        assert_eq!(
            eval_pt.x.to_bits(),
            get_u32(line, "eval_x_bits").unwrap(),
            "seed {} eval_x_bits",
            seed
        );
        assert_eq!(
            eval_pt.y.to_bits(),
            get_u32(line, "eval_y_bits").unwrap(),
            "seed {} eval_y_bits",
            seed
        );
        assert_eq!(
            eval_pt.z.to_bits(),
            get_u32(line, "eval_z_bits").unwrap(),
            "seed {} eval_z_bits",
            seed
        );

        let mut pts = Vec::new();
        seg.get_segment_points(num_segments, &mut pts);
        let mut pts_crc: u32 = 0;
        for p in &pts {
            pts_crc = rotate_crc(pts_crc, p.x.to_bits());
            pts_crc = rotate_crc(pts_crc, p.y.to_bits());
            pts_crc = rotate_crc(pts_crc, p.z.to_bits());
        }
        assert_eq!(pts_crc, get_u32(line, "pts_crc").unwrap(), "seed {} pts_crc", seed);

        let approx_len = seg.get_approximate_length(1.0);
        assert_eq!(
            approx_len.to_bits(),
            get_u32(line, "approx_len_bits").unwrap(),
            "seed {} approx_len_bits",
            seed
        );

        let mut s1 = BezierSegment::default();
        let mut s2 = BezierSegment::default();
        seg.split_segment_at_t(t, &mut s1, &mut s2);
        let mut s1_mid = Coord3D::default();
        let mut s2_mid = Coord3D::default();
        s1.evaluate_bez_segment_at_t(0.5, &mut s1_mid);
        s2.evaluate_bez_segment_at_t(0.5, &mut s2_mid);
        assert_eq!(
            s1_mid.x.to_bits(),
            get_u32(line, "s1_mid_x_bits").unwrap(),
            "seed {} s1_mid_x_bits",
            seed
        );
        assert_eq!(
            s2_mid.z.to_bits(),
            get_u32(line, "s2_mid_z_bits").unwrap(),
            "seed {} s2_mid_z_bits",
            seed
        );
    }
}
