//! Gate V2 Golden Vector parity tests for `WP-4.1` Spatial Leaves
//! (`CaveSystem`, `CrateSystem` + `INICrate`, `PolygonTrigger`, `Powers`,
//! `Terrain`, `TerrainTypes` + `INITerrain`).

use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::golden_json::{
    decode_hex_bytes, encode_hex_bytes, extract_vector_lines, get_hex_u32, get_i32,
    get_i32_array, get_i32_vec, get_str, get_u32, get_u32_array,
};
use zh_common::ini::{IniLoadType, IniReader};
use zh_common::math::ICoord3D;
use zh_common::types::{Bool, Int, UnsignedInt, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferCrc, XferLoad, XferSave};
use zh_logic::rules::science::{ScienceStore, ScienceTextLookup, SCIENCE_INVALID};
use zh_logic::spatial::{
    CaveSystem, CaveTunnelTracker, CrateSystem, PolygonTrigger, PolygonTriggerList, TerrainClass,
    TerrainTypeCollection, UnitPower, K_TRIGGERS_VERSION_1, K_TRIGGERS_VERSION_2,
    K_TRIGGERS_VERSION_3, K_TRIGGERS_VERSION_4, MAX_TERRAIN_NAME_LEN, POWERS_NUM_POWERS,
    POWER_NAMES, TERRAIN_NUM_CLASSES, TERRAIN_TYPE_NAMES,
};

const VECTORS_JSON: &str = include_str!("../../../docs/golden_vectors/WP4_1_SpatialLeaves.json");

fn push_u8(out: &mut Vec<u8>, v: u8) {
    out.push(v);
}

fn push_i32(out: &mut Vec<u8>, v: i32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_u32(out: &mut Vec<u8>, v: u32) {
    out.extend_from_slice(&v.to_le_bytes());
}

fn push_str(out: &mut Vec<u8>, s: &str) {
    push_i32(out, s.len() as i32);
    out.extend_from_slice(s.as_bytes());
}

fn serialize_crate_queries(
    cs: &CrateSystem,
    sciences: &ScienceStore,
    name_keys: &NameKeyGenerator,
    queries: &[&str],
) -> String {
    let mut bytes = Vec::new();
    for q in queries {
        let Some(ct) = cs.find_crate_template(q) else {
            push_u8(&mut bytes, 0);
            continue;
        };
        push_u8(&mut bytes, 1);
        push_str(&mut bytes, ct.name());
        push_u32(&mut bytes, ct.creation_chance.to_bits());
        push_i32(&mut bytes, ct.veterancy_level);
        push_str(&mut bytes, &ct.killed_by_type_kindof.to_hex_string());
        let sci_name = sciences.get_internal_name_for_science(ct.killer_science, name_keys);
        push_str(&mut bytes, sci_name);
        push_u8(&mut bytes, u8::from(ct.is_owned_by_maker != FALSE));
        push_i32(&mut bytes, ct.possible_crates.len() as i32);
        for entry in &ct.possible_crates {
            push_str(&mut bytes, &entry.crate_name);
            push_u32(&mut bytes, entry.crate_chance.to_bits());
        }
    }
    encode_hex_bytes(&bytes)
}

fn serialize_terrain_list(tc: &TerrainTypeCollection) -> String {
    let mut bytes = Vec::new();
    push_i32(&mut bytes, tc.len() as i32);
    for t in tc.iter() {
        push_str(&mut bytes, &t.name);
        push_str(&mut bytes, &t.texture);
        push_u8(&mut bytes, u8::from(t.blend_edge_texture != FALSE));
        push_i32(&mut bytes, t.class.as_i32());
        push_u8(&mut bytes, u8::from(t.restrict_construction != FALSE));
    }
    encode_hex_bytes(&bytes)
}

fn serialize_triggers_summary(list: &PolygonTriggerList) -> String {
    let mut bytes = Vec::new();
    push_i32(&mut bytes, list.triggers().len() as i32);
    for p in list.triggers() {
        push_str(&mut bytes, &p.trigger_name);
        push_str(&mut bytes, &p.layer_name);
        push_i32(&mut bytes, p.id());
        push_u8(&mut bytes, u8::from(p.is_water_area != FALSE));
        push_u8(&mut bytes, u8::from(p.is_river != FALSE));
        push_i32(&mut bytes, p.river_start);
        push_i32(&mut bytes, p.num_points());
        for i in 0..p.num_points() {
            let pt = p.get_point(i).unwrap();
            push_i32(&mut bytes, pt.x);
            push_i32(&mut bytes, pt.y);
            push_i32(&mut bytes, pt.z);
        }
    }
    encode_hex_bytes(&bytes)
}

#[test]
fn test_wp4_1_oracle_metadata_and_constants() {
    assert!(VECTORS_JSON.contains("\"source_sha256\""));
    assert!(VECTORS_JSON.contains("\"vector_count\": 1000"));
    for required in [
        "CaveSystem.h",
        "CaveSystem.cpp",
        "CrateSystem.h",
        "CrateSystem.cpp",
        "INICrate.cpp",
        "PolygonTrigger.h",
        "PolygonTrigger.cpp",
        "Powers.h",
        "Terrain.h",
        "TerrainTypes.h",
        "TerrainTypes.cpp",
        "INITerrain.cpp",
    ] {
        assert!(VECTORS_JSON.contains(required), "missing sha256 for {required}");
    }

    assert_eq!(POWERS_NUM_POWERS, 4);
    assert_eq!(MAX_TERRAIN_NAME_LEN, 64);
    assert_eq!(TERRAIN_NUM_CLASSES, 38);
    assert_eq!(K_TRIGGERS_VERSION_1, 1);
    assert_eq!(K_TRIGGERS_VERSION_2, 2);
    assert_eq!(K_TRIGGERS_VERSION_3, 3);
    assert_eq!(K_TRIGGERS_VERSION_4, 4);

    for i in 0..POWERS_NUM_POWERS as i32 {
        let p = UnitPower::from_i32(i).unwrap();
        assert_eq!(p.name(), POWER_NAMES[i as usize]);
    }
    assert!(UnitPower::from_i32(4).is_none());

    for i in 0..TERRAIN_NUM_CLASSES as i32 {
        let tc = TerrainClass::from_i32(i).unwrap();
        assert_eq!(tc.as_i32(), i);
        assert!(!TERRAIN_TYPE_NAMES[i as usize].is_empty());
    }
    assert_eq!(TERRAIN_TYPE_NAMES[20], "BLEND_EDGE");
    assert_eq!(TERRAIN_TYPE_NAMES[37], "URBAN");

    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);
}

#[test]
fn test_wp4_1_cave_and_polygon_boundaries() {
    // CaveSystem non-empty load rejection & unregister no-op.
    let mut cave: CaveSystem = CaveSystem::new();
    cave.register_new_cave(0);
    cave.get_tunnel_tracker_for_cave_index_mut(0)
        .unwrap()
        .contain_count = 3;
    let mut xs = XferSave::new();
    cave.xfer(&mut xs).unwrap();
    let bytes = xs.into_bytes();

    let mut xl = XferLoad::new(&bytes);
    assert!(cave.xfer(&mut xl).is_err());

    let mut cave2: CaveSystem = CaveSystem::new();
    let mut xl2 = XferLoad::new(&bytes);
    cave2.xfer(&mut xl2).unwrap();
    assert_eq!(
        cave2
            .get_tunnel_tracker_for_cave_index(0)
            .unwrap()
            .contain_count(),
        3
    );
    cave2.unregister_cave(0);
    assert_eq!(cave2.can_switch_index_to_index(-1, 0), FALSE);
    assert_eq!(cave2.can_switch_index_to_index(-1, 1), TRUE);

    // PolygonTrigger retail_compatible_crc radius quirk toggle.
    let mut pt_modern = PolygonTrigger::new(1);
    pt_modern.add_point(ICoord3D::new(10, 20, 0));
    pt_modern.add_point(ICoord3D::new(50, 80, 0));
    let rad_modern = pt_modern.get_radius();

    let mut pt_retail = pt_modern.clone();
    pt_retail.bounds_needs_update = TRUE;
    pt_retail.retail_compatible_crc = true;
    let rad_retail = pt_retail.get_radius();
    assert_ne!(rad_modern.to_bits(), rad_retail.to_bits());
}

#[test]
fn test_wp4_1_spatial_leaves_1000_golden_vectors() {
    let lines = extract_vector_lines(VECTORS_JSON);
    assert_eq!(lines.len(), 1000);

    struct NoopTextLookup;
    impl ScienceTextLookup for NoopTextLookup {
        fn fetch_label(&mut self, label: &str) -> String {
            label.to_string()
        }
    }

    let mut name_keys = NameKeyGenerator::new();
    name_keys.init();
    let mut sciences = ScienceStore::new();
    sciences.init();
    let mut sci_ini = IniReader::new(
        "sciences.ini",
        b"Science SCIENCE_CRATE_ALPHA\n  SciencePurchasePointCost = 1\n  IsGrantable = Yes\nEnd\n\
          Science SCIENCE_CRATE_BETA\n  SciencePurchasePointCost = 2\n  IsGrantable = Yes\nEnd\n\
          Science SCIENCE_CRATE_GAMMA\n  SciencePurchasePointCost = 3\n  IsGrantable = Yes\nEnd\n",
        IniLoadType::Overwrite,
    );
    let mut text_lookup = NoopTextLookup;
    sciences
        .load_from_ini(&mut sci_ini, &mut name_keys, &mut text_lookup)
        .unwrap();
    assert_ne!(SCIENCE_INVALID, 0);

    let crate_queries = [
        "DefaultCrate",
        "CrateA",
        "CrateB",
        "CrateOvOnly",
        "MissingCrate",
    ];

    for (expected_seed, line) in lines.iter().enumerate() {
        let seed = get_u32(line, "seed").unwrap();
        assert_eq!(seed as usize, expected_seed);

        // --- 1. CaveSystem ---
        let mut cave: CaveSystem = CaveSystem::new();
        cave.init();
        let cave_ops = get_i32_vec(line, "cave_ops").unwrap();
        for triple in cave_ops.chunks_exact(3) {
            let idx = triple[0];
            let cnt = triple[1];
            let tag = triple[2] as UnsignedInt;
            cave.register_new_cave(idx);
            let slot = cave.get_tunnel_tracker_for_cave_index_mut(idx).unwrap();
            slot.contain_count = cnt;
            slot.tag = tag;
        }
        let switches = get_i32_array::<4>(line, "cave_switches").unwrap();
        let switch_ok_exp = get_i32_array::<2>(line, "cave_switch_ok").unwrap();
        assert_eq!(
            cave.can_switch_index_to_index(switches[0], switches[1]) as i32,
            switch_ok_exp[0],
            "seed {seed} switch A"
        );
        assert_eq!(
            cave.can_switch_index_to_index(switches[2], switches[3]) as i32,
            switch_ok_exp[1],
            "seed {seed} switch B"
        );
        let queries = get_i32_vec(line, "cave_queries").unwrap();
        let query_exp = get_i32_vec(line, "cave_query_counts").unwrap();
        for (q, exp) in queries.iter().zip(query_exp.iter()) {
            let got = cave
                .get_tunnel_tracker_for_cave_index(*q)
                .map_or(-1, |t| t.contain_count);
            assert_eq!(got, *exp, "seed {seed} cave query {q}");
        }
        cave.reset();

        let dense_counts = get_i32_vec(line, "cave_dense_counts").unwrap();
        let dense_tags = get_i32_vec(line, "cave_dense_tags").unwrap();
        for (i, (&c, &t)) in dense_counts.iter().zip(dense_tags.iter()).enumerate() {
            cave.register_new_cave(i as Int);
            let slot = cave.get_tunnel_tracker_for_cave_index_mut(i as Int).unwrap();
            slot.contain_count = c;
            slot.tag = t as UnsignedInt;
        }
        let mut xs = XferSave::new();
        cave.xfer(&mut xs).unwrap();
        let got_cave_bytes = xs.into_bytes();
        let exp_cave_bytes = decode_hex_bytes(get_str(line, "cave_xfer_hex").unwrap()).unwrap();
        assert_eq!(got_cave_bytes, exp_cave_bytes, "seed {seed} cave xfer bytes");

        let mut xcrc = XferCrc::new();
        xcrc.xfer_snapshot(&mut cave).unwrap();
        assert_eq!(
            xcrc.get_crc(),
            get_hex_u32(line, "cave_xfer_crc").unwrap(),
            "seed {seed} cave xfer crc"
        );

        let mut cave_loaded: CaveSystem = CaveSystem::new();
        let mut xl = XferLoad::new(&got_cave_bytes);
        cave_loaded.xfer(&mut xl).unwrap();
        assert_eq!(cave_loaded, cave, "seed {seed} cave roundtrip");

        // --- 2. CrateSystem + INICrate ---
        let mut crate_sys = CrateSystem::new();
        crate_sys.init();
        let base_ini = decode_hex_bytes(get_str(line, "crate_base_ini_hex").unwrap()).unwrap();
        let ov_ini = decode_hex_bytes(get_str(line, "crate_ov_ini_hex").unwrap()).unwrap();
        crate_sys
            .load_from_ini(&base_ini, IniLoadType::Overwrite, &sciences, &mut name_keys)
            .unwrap();
        crate_sys
            .load_from_ini(
                &ov_ini,
                IniLoadType::CreateOverrides,
                &sciences,
                &mut name_keys,
            )
            .unwrap();
        assert_eq!(
            serialize_crate_queries(&crate_sys, &sciences, &name_keys, &crate_queries),
            get_str(line, "crate_pre_hex").unwrap(),
            "seed {seed} crate pre"
        );
        crate_sys.reset();
        assert_eq!(
            serialize_crate_queries(&crate_sys, &sciences, &name_keys, &crate_queries),
            get_str(line, "crate_post_hex").unwrap(),
            "seed {seed} crate post"
        );

        // --- 3. PolygonTrigger ---
        let mut poly_list = PolygonTriggerList::new();
        let mut trig = poly_list.create_trigger();
        trig.trigger_name = format!("Trig{seed}");
        trig.layer_name = if seed % 2 == 0 {
            "LayerMain".to_string()
        } else {
            String::new()
        };
        let flags = get_i32_array::<3>(line, "poly_flags").unwrap();
        let poly_ops = get_i32_array::<11>(line, "poly_ops").unwrap();
        let water_ext_y = get_i32(line, "poly_water_ext_y").unwrap();
        trig.is_water_area = Bool::from(flags[0] != 0);
        trig.is_river = Bool::from(flags[1] != 0);
        trig.river_start = poly_ops[10];
        let water_ext_x = flags[2];

        let init_pts = get_i32_vec(line, "poly_init_pts").unwrap();
        for c in init_pts.chunks_exact(3) {
            trig.add_point(ICoord3D::new(c[0], c[1], c[2]));
        }
        trig.insert_point(
            ICoord3D::new(poly_ops[1], poly_ops[2], poly_ops[3]),
            poly_ops[0],
        );
        trig.set_point(
            ICoord3D::new(poly_ops[5], poly_ops[6], poly_ops[7]),
            poly_ops[4],
        );
        trig.delete_point(poly_ops[8]);

        let clamped_got = trig.get_point(poly_ops[9]).unwrap();
        let clamped_exp = get_i32_array::<3>(line, "poly_clamped_pt").unwrap();
        assert_eq!(
            [clamped_got.x, clamped_got.y, clamped_got.z],
            clamped_exp,
            "seed {seed} clamped pt"
        );

        let rad_got = trig.get_radius();
        assert_eq!(
            rad_got.to_bits(),
            get_u32(line, "poly_radius_bits").unwrap(),
            "seed {seed} radius"
        );
        let center_got = trig.get_center_point(|x, y| 12.5_f32 + x * 0.01_f32 - y * 0.02_f32);
        let center_exp = get_u32_array::<3>(line, "poly_center_bits").unwrap();
        assert_eq!(
            [
                center_got.x.to_bits(),
                center_got.y.to_bits(),
                center_got.z.to_bits()
            ],
            center_exp,
            "seed {seed} center"
        );

        let probes = get_i32_vec(line, "poly_probe_pts").unwrap();
        let inside_exp = get_i32_vec(line, "poly_probe_inside").unwrap();
        for (c, &exp) in probes.chunks_exact(3).zip(inside_exp.iter()) {
            let got = trig.point_in_trigger(ICoord3D::new(c[0], c[1], c[2])) as i32;
            assert_eq!(got, exp, "seed {seed} point_in_trigger");
        }

        let mut pxs = XferSave::new();
        trig.xfer(&mut pxs).unwrap();
        let got_px_bytes = pxs.into_bytes();
        let exp_px_bytes = decode_hex_bytes(get_str(line, "poly_xfer_hex").unwrap()).unwrap();
        assert_eq!(got_px_bytes, exp_px_bytes, "seed {seed} poly xfer bytes");

        let mut pxc = XferCrc::new();
        pxc.xfer_snapshot(&mut trig).unwrap();
        assert_eq!(
            pxc.get_crc(),
            get_hex_u32(line, "poly_xfer_crc").unwrap(),
            "seed {seed} poly xfer crc"
        );

        let mut trig_loaded = PolygonTrigger::new(trig.id());
        let mut pxl = XferLoad::new(&got_px_bytes);
        trig_loaded.xfer(&mut pxl).unwrap();
        assert_eq!(trig_loaded.points, trig.points);
        assert_eq!(trig_loaded.bounds, trig.bounds);
        assert_eq!(trig_loaded.radius.to_bits(), trig.radius.to_bits());

        // Write & round-trip DataChunk v4 with SecondPoly.
        poly_list.add_polygon_trigger(trig);
        let mut trig2 = poly_list.create_trigger();
        trig2.trigger_name = "SecondPoly".to_string();
        trig2.layer_name = "Secondary".to_string();
        trig2.add_point(ICoord3D::new(10, 10, 5));
        trig2.add_point(ICoord3D::new(50, 10, 5));
        trig2.add_point(ICoord3D::new(30, 50, 5));
        poly_list.add_polygon_trigger(trig2);

        let mut v4_bytes = Vec::new();
        poly_list.write_polygon_triggers_data_chunk(&mut v4_bytes).unwrap();
        assert_eq!(
            encode_hex_bytes(&v4_bytes),
            get_str(line, "poly_v4_chunk_hex").unwrap(),
            "seed {seed} v4 chunk hex"
        );
        assert!(poly_list
            .parse_polygon_triggers_chunk_stream(&v4_bytes, 0, 0)
            .unwrap());
        assert_eq!(
            serialize_triggers_summary(&poly_list),
            get_str(line, "poly_v4_rt_hex").unwrap(),
            "seed {seed} v4 roundtrip"
        );

        let legacy_bytes = decode_hex_bytes(get_str(line, "poly_legacy_chunk_hex").unwrap()).unwrap();
        assert!(poly_list
            .parse_polygon_triggers_chunk_stream(&legacy_bytes, water_ext_x, water_ext_y)
            .unwrap());
        assert_eq!(
            serialize_triggers_summary(&poly_list),
            get_str(line, "poly_legacy_rt_hex").unwrap(),
            "seed {seed} legacy roundtrip"
        );
        let id_probe = poly_list.create_trigger();
        assert_eq!(
            id_probe.id(),
            get_i32(line, "poly_next_id").unwrap(),
            "seed {seed} next trigger id"
        );

        // --- 4. TerrainTypes + INITerrain ---
        let mut tc = TerrainTypeCollection::new();
        tc.init();
        let terrain_ini = decode_hex_bytes(get_str(line, "terrain_ini_hex").unwrap()).unwrap();
        tc.load_from_ini(&terrain_ini).unwrap();
        assert_eq!(
            serialize_terrain_list(&tc),
            get_str(line, "terrain_hex").unwrap(),
            "seed {seed} terrain list"
        );
    }
}
