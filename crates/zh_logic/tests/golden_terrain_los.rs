//! Edges of the terrain line of sight and cliff queries against vectors recorded by
//! `tests/cpp_harness/gen_terrain_los.cpp`.

use std::collections::BTreeMap;

use zh_common::golden_json::{decode_hex_bytes, get_str, parse_hex_u32};
use zh_common::math::Coord3D;
use zh_logic::terrain::height_map::HeightMap;

const JSON: &str = include_str!("../../../docs/golden_vectors/TerrainLos.json");

fn floats<const N: usize>(text: &str) -> [f32; N] {
    assert_eq!(text.len(), N * 8);
    std::array::from_fn(|i| f32::from_bits(parse_hex_u32(&text[i * 8..i * 8 + 8]).unwrap()))
}

#[test]
fn line_of_sight_and_cliff_cells_match_the_reference() {
    let mut maps = BTreeMap::new();
    let (mut cliffs, mut lines) = (0, 0);
    for line in JSON.lines().map(|l| l.trim_end_matches(',')) {
        if get_str(line, "case").is_none() {
            continue;
        }
        let field = |key| get_str(line, key).unwrap();
        match field("case") {
            "map" => {
                let size: Vec<i32> = field("size")
                    .split(' ')
                    .map(|n| n.parse().unwrap())
                    .collect();
                let data = decode_hex_bytes(field("data")).unwrap();
                let map = HeightMap::from_samples(size[0], size[1], size[2], Vec::new(), data);
                let flags: String = (0..size[1])
                    .flat_map(|y| (0..size[0]).map(move |x| (x, y)))
                    .map(|(x, y)| {
                        if map.cliffs().get_cliff_state(x, y) != 0 {
                            '1'
                        } else {
                            '0'
                        }
                    })
                    .collect();
                assert_eq!(flags, field("cliffs"), "map {}", field("id"));
                assert_eq!(
                    map.max_height().to_bits(),
                    floats::<1>(field("max"))[0].to_bits()
                );
                maps.insert(field("id"), map);
            }
            "cliff" => {
                let [x, y] = floats(field("at"));
                let out = maps[field("map")].is_cliff_cell(x, y) != 0;
                assert_eq!(u8::from(out).to_string(), field("out"), "{line}");
                cliffs += 1;
            }
            "los" => {
                let [ax, ay, az, bx, by, bz] = floats(field("line"));
                let out = maps[field("map")]
                    .is_clear_line_of_sight(&Coord3D::new(ax, ay, az), &Coord3D::new(bx, by, bz))
                    != 0;
                assert_eq!(u8::from(out).to_string(), field("out"), "{line}");
                lines += 1;
            }
            other => panic!("unknown row {other}"),
        }
    }
    assert_eq!((maps.len(), cliffs, lines), (12, 12 * 150, 12 * 260));
}
