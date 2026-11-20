//! Gate V2-Golden test suite for `PKT-017` (`DiscreteCircle`, 1,000 C++ reference vectors).
//!
//! When implementing `PKT-017`, remove the `#[ignore]` attribute below so this suite runs
//! automatically in `cargo test --workspace` and CI!

use zh_common::golden_json::{extract_vector_lines, get_i32, get_u32};
use zh_common::math::discrete_circle::DiscreteCircle;

const PKT017_JSON: &str = include_str!("../../../docs/golden_vectors/PKT-017.json");

fn rotate_crc(crc: u32, word: u32) -> u32 {
    crc.rotate_left(1).wrapping_add(word)
}

#[test]
#[ignore = "Un-ignore when implementing PKT-017 (crates/zh_common/src/math/discrete_circle.rs)"]
fn test_pkt017_discrete_circle_1000_golden_vectors() {
    let lines = extract_vector_lines(PKT017_JSON);
    assert_eq!(lines.len(), 1000, "Expected 1,000 golden vectors in PKT-017.json");

    for line in lines {
        let seed = get_u32(line, "seed").expect("seed");
        let xc = get_i32(line, "xc").expect("xc");
        let yc = get_i32(line, "yc").expect("yc");
        let radius = get_i32(line, "radius").expect("radius");

        let dc = DiscreteCircle::new(xc, yc, radius);
        assert_eq!(
            dc.get_edge_count(),
            get_i32(line, "edge_count").unwrap(),
            "seed {} edge_count",
            seed
        );

        let edges = dc.get_edges();
        let (first_y, first_xs, first_xe) = edges
            .first()
            .map(|e| (e.y_pos, e.x_start, e.x_end))
            .unwrap_or((0, 0, 0));
        let (last_y, last_xs, last_xe) = edges
            .last()
            .map(|e| (e.y_pos, e.x_start, e.x_end))
            .unwrap_or((0, 0, 0));

        assert_eq!(first_y, get_i32(line, "first_y").unwrap(), "seed {} first_y", seed);
        assert_eq!(first_xs, get_i32(line, "first_xs").unwrap(), "seed {} first_xs", seed);
        assert_eq!(first_xe, get_i32(line, "first_xe").unwrap(), "seed {} first_xe", seed);
        assert_eq!(last_y, get_i32(line, "last_y").unwrap(), "seed {} last_y", seed);
        assert_eq!(last_xs, get_i32(line, "last_xs").unwrap(), "seed {} last_xs", seed);
        assert_eq!(last_xe, get_i32(line, "last_xe").unwrap(), "seed {} last_xe", seed);

        let mut edges_crc: u32 = 0;
        for e in edges {
            edges_crc = rotate_crc(edges_crc, e.y_pos as u32);
            edges_crc = rotate_crc(edges_crc, e.x_start as u32);
            edges_crc = rotate_crc(edges_crc, e.x_end as u32);
        }
        assert_eq!(
            edges_crc,
            get_u32(line, "edges_crc").unwrap(),
            "seed {} edges_crc",
            seed
        );

        let mut draw_crc: u32 = 0;
        dc.draw_circle(|xs, xe, y| {
            draw_crc = rotate_crc(draw_crc, xs as u32);
            draw_crc = rotate_crc(draw_crc, xe as u32);
            draw_crc = rotate_crc(draw_crc, y as u32);
        });
        assert_eq!(
            draw_crc,
            get_u32(line, "draw_crc").unwrap(),
            "seed {} draw_crc",
            seed
        );
    }
}
