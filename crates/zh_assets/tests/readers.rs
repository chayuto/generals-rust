//! Hand-built inputs for what no reference loader is compared on (materials,
//! collision trees, emitters) and for the readers' reporting and error paths.

use zh_assets::chunk::{Skipped, Truncated};
use zh_assets::w3d::{self, anim::BoundAnimation, anim::Channels, Object};
use zh_assets::{csf, dds, tga};

#[derive(Default)]
struct W(Vec<u8>);

impl W {
    fn u8s(mut self, v: &[u8]) -> Self {
        self.0.extend_from_slice(v);
        self
    }
    fn u16(self, v: u16) -> Self {
        self.u8s(&v.to_le_bytes())
    }
    fn u32(self, v: u32) -> Self {
        self.u8s(&v.to_le_bytes())
    }
    fn words(self, v: &[u32]) -> Self {
        v.iter().fold(self, |w, &v| w.u32(v))
    }
    fn f(self, v: &[f32]) -> Self {
        v.iter().fold(self, |w, &v| w.u32(v.to_bits()))
    }
    fn name(mut self, s: &str, width: usize) -> Self {
        let start = self.0.len();
        self.0.extend_from_slice(s.as_bytes());
        self.0.resize(start + width, 0);
        self
    }
    fn chunk(self, id: u32, body: W) -> Self {
        let n = body.0.len() as u32;
        self.u32(id).u32(n).u8s(&body.0)
    }
    fn parent(self, id: u32, body: W) -> Self {
        let n = body.0.len() as u32 | 0x8000_0000;
        self.u32(id).u32(n).u8s(&body.0)
    }
}

fn w() -> W {
    W::default()
}

fn mesh_header(version: u32, attributes: u32, tris: u32, verts: u32) -> W {
    w().words(&[version, attributes])
        .name("BODY", 16)
        .name("TANK", 16)
        .words(&[tris, verts, 1, 2])
        .u32(-3i32 as u32)
        .words(&[4, 5, 6, 7])
        .f(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
}

#[test]
fn mesh_materials_and_tree() {
    let material = w()
        .chunk(0x2c, w().name("paint", 6))
        .chunk(
            0x2d,
            w().u32(9)
                .u8s(&[1, 2, 3, 0, 4, 5, 6, 0, 7, 8, 9, 0, 10, 11, 12, 0])
                .f(&[0.5, 0.25, 0.125]),
        )
        .chunk(0x2e, w().name("arg0", 5))
        .chunk(0x2f, w().name("arg1", 5));
    let texture = w()
        .chunk(0x32, w().name("a.tga", 6))
        .chunk(0x33, w().u16(3).u16(4).u32(5).f(&[6.0]));
    let stage = w()
        .chunk(0x49, w().words(&[7, 8]))
        .chunk(0x4a, w().f(&[0.5, 1.5]))
        .chunk(0x4b, w().words(&[1, 2, 3]));
    let pass = w()
        .chunk(0x39, w().words(&[11]))
        .chunk(0x3a, w().words(&[12]))
        .chunk(0x3b, w().u8s(&[1, 2, 3, 4]))
        .chunk(0x3c, w().u8s(&[5, 6, 7, 8]))
        .chunk(0x3e, w().u8s(&[9, 10, 11, 12]))
        .parent(0x48, stage);
    let tree = w()
        .chunk(0x91, w().words(&[2, 3, 0, 0, 0, 0, 0, 0]))
        .chunk(0x92, w().words(&[4, 5, 6]))
        .chunk(0x93, w().f(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]).words(&[7, 8]));
    let body = w()
        .chunk(0x1f, mesh_header(w3d::version(4, 2), 0x0002_0000, 0, 0))
        .chunk(0x0c, w().name("note", 6))
        .chunk(0x28, w().words(&[1, 2, 3, 4]))
        .chunk(
            0x29,
            w().u8s(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
        )
        .chunk(0x80, w().u8s(&[1; 24]))
        .parent(0x2a, w().parent(0x2b, material))
        .parent(0x30, w().parent(0x31, texture))
        .parent(0x38, pass)
        .parent(0x90, tree);
    let file = w3d::parse(&w().parent(0, body).0);
    assert!(file.report.is_clean(), "{:?}", file.report);
    // 1 mesh, 9 under it, 5 material, 3 texture, 6 + 3 pass, 3 tree.
    assert_eq!(file.report.chunks, 30);
    let [Object::Mesh(m)] = &file.objects[..] else {
        panic!("mesh");
    };
    assert_eq!(m.name(), "TANK.BODY");
    assert!(m.is_skin());
    let h = &m.header;
    assert_eq!(
        (
            h.num_materials,
            h.num_damage_stages,
            h.sort_level,
            h.prelit_version
        ),
        (1, 2, -3, 4)
    );
    assert_eq!(
        (h.future_counts, h.vertex_channels, h.face_channels),
        (5, 6, 7)
    );
    assert_eq!(m.user_text.as_deref(), Some("note"));
    assert_eq!(m.material_info, Some([1, 2, 3, 4]));
    assert_eq!(m.shaders[0][15], 16);
    assert_eq!(m.ps2_shaders, vec![[1; 12]; 2]);
    let v = &m.vertex_materials[0];
    assert_eq!(v.name, "paint");
    assert_eq!(v.mapper_args, ["arg0".to_string(), "arg1".to_string()]);
    let info = v.info.as_ref().unwrap();
    assert_eq!(info.attributes, 9);
    assert_eq!(
        info.colors,
        [[1, 2, 3, 0], [4, 5, 6, 0], [7, 8, 9, 0], [10, 11, 12, 0]]
    );
    assert_eq!(
        (info.shininess, info.opacity, info.translucency),
        (0.5, 0.25, 0.125)
    );
    assert_eq!(m.textures[0].name, "a.tga");
    let t = m.textures[0].info.unwrap();
    assert_eq!(
        (t.attributes, t.anim_type, t.frame_count, t.frame_rate),
        (3, 4, 5, 6.0)
    );
    let p = &m.passes[0];
    assert_eq!(
        (&p.vertex_material_ids[..], &p.shader_ids[..]),
        (&[11][..], &[12][..])
    );
    assert_eq!(
        (p.dcg[0], p.dig[0], p.scg[0]),
        ([1, 2, 3, 4], [5, 6, 7, 8], [9, 10, 11, 12])
    );
    let s = &p.stages[0];
    assert_eq!(s.texture_ids, [7, 8]);
    assert_eq!(s.texcoords, [[0.5, 1.5]]);
    assert_eq!(s.per_face_texcoord_ids, [[1, 2, 3]]);
    let tree = m.aabtree.as_ref().unwrap();
    assert_eq!((tree.node_count, tree.poly_count), (2, 3));
    assert_eq!(tree.poly_indices, [4, 5, 6]);
    let n = tree.nodes[0];
    assert_eq!((n.min, n.max), ([1.0, 2.0, 3.0], [4.0, 5.0, 6.0]));
    assert_eq!((n.front_or_first_poly, n.back_or_poly_count), (7, 8));
}

#[test]
fn emitter() {
    let info = w()
        .name("spark.tga", 260)
        .f(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])
        .f(&[11.0, 12.0, 13.0, 14.0, 15.0, 16.0])
        .u8s(&[1, 2, 3, 4, 5, 6, 7, 8]);
    let randomizer = |n| w().u32(n).f(&[1.0, 2.0, 3.0]).words(&[4, 5, 6, 7]).0;
    let info2 = w()
        .u32(21)
        .u8s(&randomizer(1))
        .u8s(&randomizer(2))
        .f(&[0.5, 0.25])
        .u8s(&[3; 16])
        .words(&[31, 32, 1, 2, 3, 4, 5, 6]);
    let props = w()
        .words(&[1, 2, 1])
        .u8s(&[9, 8, 7, 6])
        .f(&[0.1, 0.2])
        .words(&[0; 4])
        .f(&[0.0])
        .u8s(&[1, 1, 1, 1])
        .f(&[0.0, 1.0, 1.0, 0.5, 0.0, 2.0]);
    let body = w()
        .chunk(0x501, w().u32(7).name("Smoke", 16))
        .chunk(0x502, w().words(&[3, 3]).name("hi", 3))
        .chunk(0x503, info)
        .chunk(0x504, info2)
        .chunk(0x505, props)
        .chunk(
            0x509,
            w().words(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
        )
        .chunk(
            0x50a,
            w().u32(1).f(&[0.5, 0.75]).u32(0).f(&[0.0, 1.0, 2.0, 3.0]),
        )
        .chunk(0x50b, w().u32(0).f(&[1.5]).words(&[0, 0]).f(&[0.0, 4.0]))
        .chunk(0x50c, w().u32(0).f(&[2.5]).u32(0).f(&[0.0, 5.0]));
    let file = w3d::parse(&w().parent(0x500, body).0);
    assert!(file.report.is_clean(), "{:?}", file.report);
    let [Object::Emitter(e)] = &file.objects[..] else {
        panic!("emitter");
    };
    assert_eq!((e.version, e.name.as_str()), (7, "Smoke"));
    assert_eq!((e.user_type, e.user_string.as_str()), (3, "hi"));
    let i = e.info.as_ref().unwrap();
    assert_eq!(i.texture, "spark.tga");
    assert_eq!((i.params[0], i.params[9]), (1.0, 10.0));
    assert_eq!(
        (i.velocity, i.acceleration),
        ([11.0, 12.0, 13.0], [14.0, 15.0, 16.0])
    );
    assert_eq!((i.start_color, i.end_color), ([1, 2, 3, 4], [5, 6, 7, 8]));
    let i2 = e.info_v2.as_ref().unwrap();
    assert_eq!(
        (
            i2.burst_size,
            i2.creation_volume.class_id,
            i2.velocity_random.class_id
        ),
        (21, 1, 2)
    );
    assert_eq!(i2.creation_volume.values, [1.0, 2.0, 3.0]);
    assert_eq!(i2.creation_volume.reserved, [4, 5, 6, 7]);
    assert_eq!((i2.outward_velocity, i2.velocity_inherit), (0.5, 0.25));
    assert_eq!(
        (i2.shader, i2.render_mode, i2.frame_mode),
        ([3; 16], 31, 32)
    );
    assert_eq!(i2.reserved, [1, 2, 3, 4, 5, 6]);
    let p = e.props.as_ref().unwrap();
    assert_eq!(
        (p.color_random, p.opacity_random, p.size_random),
        ([9, 8, 7, 6], 0.1, 0.2)
    );
    assert_eq!(p.colors, [(0.0, [1, 1, 1, 1])]);
    assert_eq!(p.opacities, [(0.0, 1.0), (1.0, 0.5)]);
    assert_eq!(p.sizes, [(0.0, 2.0)]);
    assert_eq!(e.line_properties.unwrap()[15], 16);
    let r = e.rotations.as_ref().unwrap();
    assert_eq!((r.random, r.orientation_random), (0.5, 0.75));
    assert_eq!(r.keys, [(0.0, 1.0), (2.0, 3.0)]);
    let f = e.frames.as_ref().unwrap();
    assert_eq!((f.random, &f.keys[..]), (1.5, &[(0.0, 4.0)][..]));
    let b = e.blur_times.as_ref().unwrap();
    assert_eq!((b.random, &b.keys[..]), (2.5, &[(0.0, 5.0)][..]));
}

#[test]
fn collision_box_and_proxies() {
    let boxed = w()
        .words(&[1, 2])
        .name("A.BOX", 32)
        .u8s(&[3, 4, 5, 0])
        .f(&[1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let array = w()
        .chunk(0x703, w().u32(1).f(&[2.5]))
        .chunk(0x704, w().u32(3).name("P", 32));
    let hlod = w()
        .chunk(0x701, w().words(&[1, 0]).name("M", 16).name("S", 16))
        .parent(0x706, array);
    let file = w3d::parse(&w().chunk(0x740, boxed).parent(0x700, hlod).0);
    assert!(file.report.is_clean(), "{:?}", file.report);
    let [Object::Box(b), Object::HLod(h)] = &file.objects[..] else {
        panic!("box and hlod");
    };
    assert_eq!((b.version, b.attributes, b.name.as_str()), (1, 2, "A.BOX"));
    assert_eq!(
        (b.color, b.center, b.extent),
        ([3, 4, 5, 0], [1.0, 2.0, 3.0], [4.0, 5.0, 6.0])
    );
    let p = h.proxies.as_ref().unwrap();
    assert_eq!(
        (
            p.max_screen_size,
            p.models[0].bone_index,
            p.models[0].name.as_str()
        ),
        (2.5, 3, "P")
    );
}

#[test]
fn nothing_is_skipped_silently() {
    let body = w()
        .chunk(0x1f, mesh_header(w3d::version(4, 2), 0, 1, 1))
        .chunk(0x58, w().u32(1))
        .chunk(0x02, w().f(&[1.0, 2.0, 3.0, 4.0]));
    let mut data = w().chunk(0x300, w().u32(0)).parent(0, body).0;
    data.extend_from_slice(&[1, 2, 3]);
    let file = w3d::parse(&data);
    assert_eq!(
        file.report.skipped,
        [
            Skipped {
                path: vec![],
                id: 0x300,
                size: 4
            },
            Skipped {
                path: vec![0],
                id: 0x58,
                size: 4
            },
        ]
    );
    // Trailing bytes, a partial vertex and the missing triangle.
    assert_eq!(file.report.damaged.len(), 3, "{:?}", file.report.damaged);
    assert!(!file.report.is_clean());
}

#[test]
fn overruns_and_bad_objects_are_damage() {
    let overrun = w().u32(0x100).u32(9).u8s(&[0; 8]).0;
    let file = w3d::parse(&overrun);
    assert!(file.objects.is_empty());
    assert_eq!(file.report.damaged.len(), 1);
    // A child may not point past itself.
    let pivot = |parent: u32| w().name("p", 16).u32(parent).f(&[0.0; 10]).0;
    let hierarchy = |second: u32| {
        let body = w()
            .chunk(
                0x101,
                w().u32(w3d::version(4, 1))
                    .name("S", 16)
                    .u32(2)
                    .f(&[0.0; 3]),
            )
            .chunk(0x102, w().u8s(&pivot(u32::MAX)).u8s(&pivot(second)));
        w3d::parse(&w().parent(0x100, body).0)
    };
    assert_eq!(hierarchy(0).objects.len(), 1);
    assert!(hierarchy(0).report.is_clean());
    for bad in [1, 2, u32::MAX] {
        assert!(hierarchy(bad).objects.is_empty(), "parent {bad}");
        assert_eq!(hierarchy(bad).report.damaged.len(), 1);
    }
}

#[test]
fn old_animation_shifts_pivots_and_counts_padding() {
    let channel = w()
        .u16(0)
        .u16(1)
        .u16(1)
        .u16(1)
        .u16(0)
        .u16(0)
        .f(&[3.0, 4.0])
        .u32(0);
    let bits = w().u16(0).u16(8).u16(0).u16(0).u8s(&[0, 0xfe, 0x01]);
    let body = w()
        .chunk(
            0x201,
            w().u32(w3d::version(2, 0))
                .name("a", 16)
                .name("h", 16)
                .words(&[2, 15]),
        )
        .chunk(0x202, channel)
        .chunk(0x203, bits);
    let file = w3d::parse(&w().parent(0x200, body).0);
    assert!(file.report.is_clean(), "{:?}", file.report);
    assert_eq!(file.report.padding, 4);
    let [Object::Animation(a)] = &file.objects[..] else {
        panic!("animation");
    };
    assert_eq!((a.num_frames, a.frame_rate), (2, 15));
    let Channels::Raw { motion, visibility } = &a.channels else {
        panic!("raw");
    };
    assert_eq!((motion[0].pivot, visibility[0].pivot), (1, 1));
    // Pivot 1 carries Y only; frame 0.75 is a quarter past halfway.
    let bound = BoundAnimation::new(a, 2);
    assert_eq!(bound.translation(1, 0.75), [0.0, 3.75, 0.0]);
    assert_eq!(bound.translation(0, 0.75), [0.0; 3]);
    // Visibility: bit 0 clear, bit 8 set, the default (0) past the range.
    assert!(!bound.visible(1, 0.0));
    assert!(bound.visible(1, 8.0));
    assert!(!bound.visible(1, 9.0));
    assert!(bound.visible(1, 1.0));
    assert!(bound.visible(0, 0.0));
}

#[test]
fn skin_flag_and_link_shift_boundaries() {
    let mesh = |version: u32, attributes: u32, links: bool| {
        let mut body = w().chunk(0x1f, mesh_header(version, attributes, 0, u32::from(links)));
        if links {
            body = body
                .chunk(0x02, w().f(&[0.0; 3]))
                .chunk(0x0e, w().u16(5).u8s(&[0; 6]));
        }
        let file = w3d::parse(&w().parent(0, body).0);
        assert!(file.report.is_clean(), "{:?}", file.report);
        match file.objects.into_iter().next() {
            Some(Object::Mesh(m)) => m,
            _ => panic!("mesh"),
        }
    };
    assert_eq!(mesh(w3d::version(3, 0), 0, true).bone_links, [5]);
    assert_eq!(mesh(w3d::version(2, 9), 0, true).bone_links, [6]);
    assert!(mesh(w3d::version(3, 0), 0, true).is_skin());
    // The geometry type counts from 4.1 on.
    assert!(mesh(w3d::version(4, 1), 0x0002_0000, false).is_skin());
    assert!(!mesh(w3d::version(4, 0), 0x0002_0000, false).is_skin());
    assert!(!mesh(w3d::version(4, 1), 0, false).is_skin());
}

#[test]
fn bones_past_the_animation_keep_the_base_pose() {
    let pivot = |parent: u32, x: f32| {
        w().name("p", 16)
            .u32(parent)
            .f(&[x, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0])
            .0
    };
    let hierarchy = w()
        .chunk(
            0x101,
            w().u32(w3d::version(4, 1))
                .name("h", 16)
                .u32(3)
                .f(&[0.0; 3]),
        )
        .chunk(
            0x102,
            w().u8s(&pivot(u32::MAX, 0.0))
                .u8s(&pivot(0, 1.0))
                .u8s(&pivot(1, 2.0)),
        );
    let animation = w()
        .chunk(
            0x201,
            w().u32(w3d::version(4, 1))
                .name("a", 16)
                .name("h", 16)
                .words(&[1, 15]),
        )
        .chunk(
            0x202,
            w().u16(0).u16(0).u16(1).u16(0).u16(1).u16(0).f(&[10.0]),
        );
    let file = w3d::parse(&w().parent(0x100, hierarchy).parent(0x200, animation).0);
    let [Object::Hierarchy(h), Object::Animation(a)] = &file.objects[..] else {
        panic!("{:?}", file.report);
    };
    let skeleton = zh_assets::model::Skeleton::new(h);
    // Bound to two of the three pivots.
    let bound = BoundAnimation::new(a, 2);
    let pose = skeleton.pose(&zh_assets::wwmath::Matrix3D::IDENTITY, Some((&bound, 0.0)));
    assert_eq!(pose[1].transform.0[0][3], 11.0);
    assert_eq!(pose[2].transform.0[0][3], 13.0);
}

#[test]
fn adaptive_delta_is_reported() {
    let body = w()
        .chunk(
            0x281,
            w().u32(1).name("a", 16).name("h", 16).u32(4).u16(30).u16(1),
        )
        .chunk(0x282, w().u32(0));
    let file = w3d::parse(&w().parent(0x280, body).0);
    assert_eq!(file.report.skipped.len(), 1);
}

#[test]
fn tga_footer_tail_and_errors() {
    let header = |kind: u8, depth: u8| {
        w().u8s(&[0, 0, kind])
            .u8s(&[0; 5])
            .u8s(&[0; 4])
            .u16(2)
            .u16(1)
            .u8s(&[depth, 0])
    };
    let t = tga::parse(&header(3, 8).u8s(&[4, 5]).name("TRUEVISION-XFILE.", 18).0).unwrap();
    assert_eq!(
        (t.pixels.as_slice(), t.trailing, t.has_footer),
        (&[4, 5][..], 18, true)
    );
    assert_eq!(t.bytes_per_pixel(), 1);
    assert!(!tga::parse(&header(3, 8).u8s(&[4, 5]).0).unwrap().has_footer);
    // Short pixels, an unknown type, a run past the image, no depth.
    for bad in [
        header(3, 8).u8s(&[4]),
        header(4, 8).u8s(&[4, 5]),
        header(11, 8).u8s(&[0x82, 1]),
        header(3, 0),
    ] {
        assert_eq!(tga::parse(&bad.0).unwrap_err(), Truncated);
    }
}

#[test]
fn dds_cube_volume_and_uncompressed() {
    let file = |four_cc: &[u8; 4], bits: u32, caps2: u32, depth: u32, mips: u32, body: usize| {
        w().u8s(b"DDS ")
            .words(&[124, 1, 4, 8, 99, depth, mips])
            .words(&[0; 11])
            .words(&[32, 0x40])
            .u8s(four_cc)
            .words(&[bits, 1, 2, 3, 4])
            .words(&[0x1000, caps2, 0, 0, 0])
            .u8s(&vec![7; body])
            .0
    };
    // 8x4 at 32 bits, two levels: 128 + 32 bytes.
    let data = file(&[0; 4], 32, 0, 0, 2, 161);
    let d = dds::parse(&data).unwrap();
    assert_eq!((d.width, d.height, d.pitch_or_linear_size), (8, 4, 99));
    assert_eq!(
        (d.flags, d.pixel_flags, d.masks, d.caps),
        (1, 0x40, [1, 2, 3, 4], [0x1000, 0])
    );
    assert_eq!(
        d.levels.iter().map(|l| l.len()).collect::<Vec<_>>(),
        [128, 32]
    );
    assert_eq!(
        (d.trailing, d.engine_mip_levels(), d.engine_size()),
        (1, 1, (8, 4))
    );
    assert!(!d.is_cube_map());
    // A cube map with two faces of one DXT1 level each.
    let data = file(b"DXT1", 0, 0x200 | 0x400 | 0x800, 0, 1, 32);
    let d = dds::parse(&data).unwrap();
    assert!(d.is_cube_map());
    assert_eq!((d.levels.len(), d.trailing, d.block_bytes()), (2, 0, 8));
    // A volume of four slices, then two.
    let data = file(b"DXT5", 0, 0x20_0000, 4, 2, 128 + 32);
    let d = dds::parse(&data).unwrap();
    assert_eq!(
        d.levels.iter().map(|l| l.len()).collect::<Vec<_>>(),
        [128, 32]
    );
    // Depth halves to 2 and then 1; eight bits per pixel is allowed.
    let data = file(&[0; 4], 8, 0x20_0000, 4, 3, 128 + 16 + 2);
    let d = dds::parse(&data).unwrap();
    assert_eq!(
        d.levels.iter().map(|l| l.len()).collect::<Vec<_>>(),
        [128, 16, 2]
    );
    assert!(dds::parse(&file(b"DXT1", 0, 0, 0, 1, 15)).is_err());
    assert!(dds::parse(&file(&[0; 4], 4, 0, 0, 1, 64)).is_err());
    assert!(dds::parse(b"DDS?").is_err());
}

#[test]
fn csf_skip_trailing_and_errors() {
    let head = |skip: u32| w().words(&[0x4353_4620, 3, 1, 1, skip, 2]);
    let label = |id: u32| w().words(&[0x4c42_4c20, 1, 1]).u8s(b"L").words(&[id, 0]).0;
    let good = head(0).u8s(&label(0x5354_5220)).u8s(&[0; 2]);
    let f = csf::parse(&good.0).unwrap();
    assert_eq!(
        (f.version, f.declared_labels, f.declared_strings, f.language),
        (3, 1, 1, 2)
    );
    assert_eq!(
        (
            f.trailing,
            f.entries[0].strings,
            f.entries[0].label.as_str()
        ),
        (2, 1, "L")
    );
    assert!(csf::parse(&head(0).u8s(&label(1)).0).is_err());
    assert!(csf::parse(&head(0).u32(5).0).is_err());
    assert!(csf::parse(&w().words(&[1, 3, 0, 0, 0, 0]).0).is_err());
    let long = head(0)
        .words(&[0x4c42_4c20, 0, 10 * 1024])
        .u8s(&vec![b'a'; 10 * 1024]);
    assert!(csf::parse(&long.0).is_err());
}
