//! The asset readers against the reference loaders
//! (`tests/cpp_harness/gen_ast_assets.cpp`). Case rows carry their own input.
//! Retail rows hold counts and digests and are checked only where the game is
//! installed.

use std::collections::{BTreeMap, BTreeSet};

use zh_assets::w3d::{self, anim::BoundAnimation, Object, W3dFile};
use zh_assets::{csf, dds, model, tga, wwmath::Matrix3D};
use zh_common::golden_json::{decode_hex_bytes, extract_vector_lines, get_str, get_u32};
use zh_common::io::{BigFileSystem, FileInfo};

const JSON: &str = include_str!("../../../docs/golden_vectors/AST_assets.json");

fn rows(kind: &str) -> Vec<&'static str> {
    extract_vector_lines(JSON)
        .into_iter()
        .filter(|l| get_str(l, "case") == Some(kind))
        .collect()
}

/// The canonical stream the generator writes.
#[derive(Default)]
struct Out(Vec<u8>);

impl Out {
    fn word(&mut self, v: u32) {
        self.0.extend_from_slice(format!("{v:08x}").as_bytes());
    }
    fn float(&mut self, f: f32) {
        self.word(f.to_bits());
    }
    fn floats(&mut self, f: &[f32]) {
        f.iter().for_each(|&f| self.float(f));
    }
    /// One byte per char, as the names were read.
    fn text(&mut self, s: &str) {
        self.0.extend(s.chars().map(|c| c as u8));
        self.0.push(b';');
    }
    fn hex(&mut self, bytes: &[u8]) {
        const D: &[u8; 16] = b"0123456789abcdef";
        for &b in bytes {
            self.0.push(D[usize::from(b >> 4)]);
            self.0.push(D[usize::from(b & 15)]);
        }
    }
    fn matrix(&mut self, m: &Matrix3D) {
        m.0.iter().for_each(|r| self.floats(r));
    }
    fn pose(&mut self, pose: &[model::Pose]) {
        for p in pose {
            self.matrix(&p.transform);
            self.word(p.visible.into());
        }
    }
    fn take(&mut self) -> String {
        String::from_utf8(std::mem::take(&mut self.0)).expect("ascii case output")
    }
}

fn dump_hierarchy(out: &mut Out, s: &model::Skeleton) {
    out.word(s.num_bones() as u32);
    for i in 0..s.num_bones() {
        out.text(s.bone_name(i));
        out.word(s.parent_index(i) as u32);
    }
    out.pose(&s.pose(&Matrix3D::IDENTITY, None));
}

fn dump_anim(out: &mut Out, s: &model::Skeleton, a: &BoundAnimation, frames: &[f32]) {
    out.word(a.num_frames());
    for &f in frames {
        out.pose(&s.pose(&Matrix3D::IDENTITY, Some((a, f))));
    }
}

fn sample_frames(n: i32) -> [f32; 6] {
    [
        0.0,
        0.5,
        1.0,
        (n / 2) as f32 + 0.25,
        (n - 1) as f32,
        if n > 1 { (n - 1) as f32 - 0.25 } else { 0.0 },
    ]
}

fn dump_mesh(out: &mut Out, m: &w3d::mesh::Mesh) {
    out.text(&m.name());
    out.word(m.header.num_vertices);
    out.word(m.header.num_triangles);
    m.vertices.iter().for_each(|v| out.floats(v));
    m.normals.iter().for_each(|v| out.floats(v));
    for t in &m.triangles {
        // The engine keeps 16-bit indices and an 8-bit surface type.
        t.vertices
            .iter()
            .for_each(|&i| out.word(u32::from(i as u16)));
        out.word(u32::from(t.surface_type as u8));
    }
    out.word((!m.bone_links.is_empty()).into());
    m.bone_links.iter().for_each(|&l| out.word(l.into()));
    out.word((!m.shade_indices.is_empty()).into());
    m.shade_indices.iter().for_each(|&i| out.word(i));
    out.floats(&m.header.min);
    out.floats(&m.header.max);
    out.floats(&m.header.sphere_center);
    out.float(m.header.sphere_radius);
}

fn dump_hlod(out: &mut Out, h: &w3d::objects::HLod) {
    out.text(&h.name);
    out.text(&h.hierarchy_name);
    out.word(h.lods.len() as u32);
    for a in &h.lods {
        out.float(a.max_screen_size);
        out.word(a.models.len() as u32);
        for m in &a.models {
            out.text(&m.name);
            out.word(m.bone_index);
        }
    }
    let aggregates = h.aggregates.as_ref().map_or(&[][..], |a| &a.models);
    out.word(aggregates.len() as u32);
    for m in aggregates {
        out.text(&m.name);
        out.word(m.bone_index);
    }
}

fn dump_tga(out: &mut Out, data: &[u8]) -> bool {
    let Ok(t) = tga::parse(data) else {
        return false;
    };
    out.word(t.width.into());
    out.word(t.height.into());
    out.word(t.pixel_depth.into());
    out.word(t.image_type.into());
    out.word(t.descriptor.into());
    out.hex(&t.pixels);
    if t.color_map_type == 1 {
        out.text("pal");
        out.hex(&t.palette);
    }
    true
}

fn dump_dds(out: &mut Out, d: &dds::Dds) {
    out.word(d.engine_size().0);
    out.word(d.engine_size().1);
    out.word(d.engine_mip_levels());
    out.word(d.is_cube_map().into());
    for level in &d.levels[..d.engine_mip_levels() as usize] {
        out.word(level.len() as u32);
        out.hex(level);
    }
}

fn dump_csf(out: &mut Out, f: &csf::Csf) {
    out.word(1);
    out.word(f.declared_labels);
    out.word(f.language);
    for e in &f.entries {
        out.text(&e.label);
        out.word(e.text.len() as u32);
        e.text.iter().for_each(|&u| out.word(u.into()));
        out.text(&e.speech);
    }
}

fn parse_clean(bytes: &[u8]) -> W3dFile {
    let f = w3d::parse(bytes);
    assert!(f.report.is_clean(), "{:?}", f.report);
    f
}

#[test]
fn models_match_the_reference() {
    let rows = rows("model");
    assert_eq!(rows.len(), 36);
    for line in rows {
        let index = get_u32(line, "index").unwrap();
        let bytes = decode_hex_bytes(get_str(line, "w3d").unwrap()).unwrap();
        let file = parse_clean(&bytes);
        let mut hierarchy = None;
        let mut anims = Vec::new();
        let mut hlod = None;
        let mut boxes = Vec::new();
        for o in &file.objects {
            match o {
                Object::Hierarchy(h) => hierarchy = Some(h),
                Object::Animation(a) => anims.push(a),
                Object::HLod(h) => hlod = Some(h),
                Object::Box(b) => boxes.push(b.name.clone()),
                _ => panic!("model {index}: unexpected object"),
            }
        }
        let (hierarchy, hlod) = (hierarchy.unwrap(), hlod.unwrap());
        let m = model::Model::new(hlod, hierarchy, |n| boxes.iter().any(|b| b == n));
        let skeleton = m.skeleton();
        let mut out = Out::default();

        dump_hierarchy(&mut out, skeleton);
        assert_eq!(
            out.take(),
            get_str(line, "hierarchy").unwrap(),
            "model {index} base pose"
        );

        let bound: Vec<_> = anims
            .iter()
            .map(|a| BoundAnimation::new(a, skeleton.num_bones()))
            .collect();
        for (a, key) in bound.iter().zip(["raw", "timecoded"]) {
            let (frames, expected) = get_str(line, key).unwrap().split_once('|').unwrap();
            let frames: Vec<f32> = (0..frames.len() / 8)
                .map(|i| {
                    f32::from_bits(u32::from_str_radix(&frames[i * 8..i * 8 + 8], 16).unwrap())
                })
                .collect();
            dump_anim(&mut out, skeleton, a, &frames);
            assert_eq!(out.take(), expected, "model {index} {key}");
        }

        for (name, bone) in m.sub_objects() {
            out.text(name);
            out.word(*bone as u32);
        }
        assert_eq!(
            out.take(),
            get_str(line, "subs").unwrap(),
            "model {index} sub-objects"
        );

        let scale = f32::from_bits(get_u32(line, "scale").unwrap());
        let which = get_u32(line, "which").unwrap() as usize;
        let last = get_u32(line, "last").unwrap() == 1;
        let animation = (which != 0).then(|| {
            let a = &bound[which - 1];
            let frame = if last { a.num_frames() - 1 } else { 0 };
            (a, frame as f32)
        });
        let names = get_str(line, "names").unwrap().split_terminator(',');
        let pristine = m.pristine_bones(scale, animation, names);
        for (name, b) in pristine.iter() {
            out.text(name);
            out.word(b.bone_index as u32);
            out.matrix(&b.transform);
        }
        assert_eq!(
            out.take(),
            get_str(line, "bones").unwrap(),
            "model {index} pristine bones"
        );

        let slots: Vec<&str> = get_str(line, "slots")
            .unwrap()
            .split_terminator(',')
            .collect();
        assert_eq!(slots.len(), 12);
        for s in slots.chunks(4) {
            let barrels = pristine.barrels(model::WeaponBones {
                fire_fx: s[0],
                recoil: s[1],
                muzzle_flash: s[2],
                projectile_launch: s[3],
            });
            out.word(barrels.len() as u32);
            for b in barrels {
                out.word(b.recoil_bone as u32);
                out.word(b.muzzle_flash_bone as u32);
                out.word(b.fx_bone as u32);
                out.matrix(&b.projectile_offset);
            }
        }
        assert_eq!(
            out.take(),
            get_str(line, "barrels").unwrap(),
            "model {index} barrels"
        );
    }
}

#[test]
fn meshes_match_the_reference() {
    let rows = rows("mesh");
    assert_eq!(rows.len(), 20);
    for line in rows {
        let bytes = decode_hex_bytes(get_str(line, "w3d").unwrap()).unwrap();
        let file = parse_clean(&bytes);
        let [Object::Mesh(m)] = &file.objects[..] else {
            panic!("one mesh expected");
        };
        let mut out = Out::default();
        dump_mesh(&mut out, m);
        let (got, want) = (out.take(), get_str(line, "out").unwrap());
        let at = got.bytes().zip(want.bytes()).position(|(a, b)| a != b);
        let from = at.unwrap_or(0).saturating_sub(40);
        assert!(
            got == want,
            "mesh {:?} differs at {at:?}: lengths {} {}\n{}\n{}",
            get_u32(line, "index"),
            got.len(),
            want.len(),
            &got[from..],
            &want[from..]
        );
    }
}

#[test]
fn textures_and_strings_match_the_reference() {
    let mut out = Out::default();
    let mut seen = [0; 3];
    for (k, kind) in ["tga", "dds", "csf"].into_iter().enumerate() {
        for line in rows(kind) {
            let data = decode_hex_bytes(get_str(line, "data").unwrap()).unwrap();
            match kind {
                "tga" => assert!(dump_tga(&mut out, &data)),
                "dds" => {
                    let d = dds::parse(&data).unwrap();
                    assert_eq!(d.trailing, 0);
                    dump_dds(&mut out, &d);
                }
                _ => {
                    let f = csf::parse(&data).unwrap();
                    assert_eq!(f.trailing, 0);
                    dump_csf(&mut out, &f);
                }
            }
            assert_eq!(
                out.take(),
                get_str(line, "out").unwrap(),
                "{kind} {:?}",
                get_u32(line, "index")
            );
            seen[k] += 1;
        }
    }
    assert_eq!(seen, [40, 12, 25]);
}

#[test]
fn slerp_matches_the_reference() {
    let rows = rows("slerp");
    assert_eq!(rows.len(), 160);
    for line in rows {
        let data = get_str(line, "data").unwrap();
        let v: Vec<f32> = (0..9)
            .map(|i| f32::from_bits(u32::from_str_radix(&data[i * 8..i * 8 + 8], 16).unwrap()))
            .collect();
        let q = |at: usize| zh_assets::wwmath::Quat([v[at], v[at + 1], v[at + 2], v[at + 3]]);
        let mut out = Out::default();
        out.floats(&zh_assets::wwmath::fast_slerp(q(0), q(4), v[8]).0);
        assert_eq!(
            out.take(),
            get_str(line, "out").unwrap(),
            "{:?}",
            get_u32(line, "index")
        );
    }
}

// ---- retail ----------------------------------------------------------------

struct Digest {
    hash: u64,
    count: u32,
    failed: u32,
}

impl Default for Digest {
    fn default() -> Self {
        Digest {
            hash: 0xcbf2_9ce4_8422_2325,
            count: 0,
            failed: 0,
        }
    }
}

impl Digest {
    fn add(&mut self, out: &mut Out) {
        for &b in &out.0 {
            self.hash = (self.hash ^ u64::from(b)).wrapping_mul(0x100_0000_01b3);
        }
        out.0.clear();
        self.count += 1;
    }
}

/// Calls `f` for every entry with one of `extensions`: archives by path, names
/// within each.
fn each_entry(root: &std::path::Path, extensions: &[&str], mut f: impl FnMut(&str, &[u8])) {
    let mut archives = Vec::new();
    for sub in ["", "ZH_Generals"] {
        let Ok(dir) = std::fs::read_dir(root.join(sub)) else {
            continue;
        };
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if name.to_ascii_lowercase().ends_with(".big") {
                archives.push(if sub.is_empty() {
                    name
                } else {
                    format!("{sub}/{name}")
                });
            }
        }
    }
    archives.sort();
    for a in archives {
        let bytes = std::fs::read(root.join(&a)).unwrap();
        let archive = BigFileSystem::open_archive_bytes(&a, bytes, FileInfo::default()).unwrap();
        let mut set = BTreeSet::new();
        archive.get_file_list_in_directory("", "*", &mut set);
        let mut names: Vec<(String, String)> = set
            .iter()
            .map(|k| (k.0.to_ascii_lowercase(), k.0.clone()))
            .collect();
        names.sort();
        for (lower, name) in names {
            if extensions.iter().any(|e| lower.ends_with(e)) {
                let file = archive.open_file(&name).expect("listed file opens");
                f(&lower, file.as_bytes());
            }
        }
    }
}

/// Counts chunk headers from the sub-chunk bit alone, independently of the
/// readers, which descend by chunk id.
fn count_headers(data: &[u8]) -> u64 {
    let mut n = 0;
    let mut at = 0;
    while data.len() - at >= 8 {
        let raw = u32::from_le_bytes(data[at + 4..at + 8].try_into().unwrap());
        let size = (raw & 0x7fff_ffff) as usize;
        at += 8;
        if size > data.len() - at {
            break;
        }
        n += 1;
        if raw >> 31 != 0 {
            n += count_headers(&data[at..at + size]);
        }
        at += size;
    }
    n
}

#[test]
fn every_retail_file_parses_as_the_reference_reads_it() {
    let root = std::env::var_os("CNC_GENERALS_ZH_PATH")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::Path::new(&h).join("Games/generalszh"))
        });
    let Some(root) = root.filter(|r| r.join("W3DZH.big").is_file()) else {
        eprintln!("retail data absent: skipped");
        return;
    };

    // Hierarchies first, since animations name theirs; the first of a name wins.
    let mut skeletons: BTreeMap<String, model::Skeleton> = BTreeMap::new();
    let mut digests: BTreeMap<&str, Digest> = BTreeMap::new();
    let mut out = Out::default();
    each_entry(&root, &[".w3d"], |_, data| {
        for o in w3d::parse(data).objects {
            if let Object::Hierarchy(h) = o {
                let s = model::Skeleton::new(&h);
                dump_hierarchy(&mut out, &s);
                digests.entry("hierarchy").or_default().add(&mut out);
                skeletons.entry(h.name.to_ascii_lowercase()).or_insert(s);
            }
        }
    });

    let (mut w3d_files, mut headers, mut seen, mut padding) = (0u32, 0u64, 0u64, 0u64);
    let mut objects: BTreeMap<&str, u32> = BTreeMap::new();
    let mut skipped: BTreeMap<(Vec<u32>, u32), u32> = BTreeMap::new();
    let mut damaged: BTreeMap<String, usize> = BTreeMap::new();
    let mut tga_shapes: BTreeMap<(u8, u8, bool), u32> = BTreeMap::new();
    let mut tga_unexplained = 0u32;
    let mut dds_formats: BTreeMap<[u8; 4], u32> = BTreeMap::new();
    let mut dds_trailing = 0u32;
    let mut csf_totals = Vec::new();
    each_entry(&root, &[".w3d", ".tga", ".dds", ".csf"], |name, data| {
        if name.ends_with(".w3d") {
            w3d_files += 1;
            let file = w3d::parse(data);
            headers += count_headers(data);
            seen += file.report.chunks;
            padding += file.report.padding;
            if !file.report.damaged.is_empty() {
                *damaged.entry(name.to_string()).or_default() += file.report.damaged.len();
            }
            for s in &file.report.skipped {
                *skipped.entry((s.path.clone(), s.id)).or_default() += 1;
            }
            for o in &file.objects {
                let kind = match o {
                    Object::Mesh(m) => {
                        dump_mesh(&mut out, m);
                        digests.entry("mesh").or_default().add(&mut out);
                        "mesh"
                    }
                    Object::Hierarchy(_) => "hierarchy",
                    Object::Animation(a) => {
                        match skeletons.get(&a.hierarchy_name.to_ascii_lowercase()) {
                            Some(s) => {
                                out.text(&format!("{}.{}", a.hierarchy_name, a.name));
                                let bound = BoundAnimation::new(a, s.num_bones());
                                let frames = sample_frames(a.num_frames as i32);
                                dump_anim(&mut out, s, &bound, &frames);
                                digests.entry("animation").or_default().add(&mut out);
                            }
                            None => digests.entry("animation").or_default().failed += 1,
                        }
                        "animation"
                    }
                    Object::HLod(h) => {
                        dump_hlod(&mut out, h);
                        digests.entry("hlod").or_default().add(&mut out);
                        "hlod"
                    }
                    Object::Box(_) => "box",
                    Object::Emitter(_) => "emitter",
                };
                *objects.entry(kind).or_default() += 1;
            }
        } else if name.ends_with(".tga") {
            if dump_tga(&mut out, data) {
                digests.entry("tga").or_default().add(&mut out);
                let t = tga::parse(data).unwrap();
                *tga_shapes
                    .entry((t.image_type, t.pixel_depth, t.has_footer))
                    .or_default() += 1;
                // After the image come only the TGA 2.0 areas, which end in the footer.
                if t.trailing != 0 && !t.has_footer {
                    tga_unexplained += 1;
                }
            } else {
                digests.entry("tga").or_default().failed += 1;
            }
        } else if name.ends_with(".dds") {
            let d = dds::parse(data).unwrap_or_else(|_| panic!("dds {name}"));
            *dds_formats.entry(d.four_cc).or_default() += 1;
            dds_trailing += u32::from(d.trailing != 0);
            dump_dds(&mut out, &d);
            digests.entry("dds").or_default().add(&mut out);
        } else {
            let f = csf::parse(data).unwrap_or_else(|_| panic!("csf {name}"));
            assert_eq!(f.entries.len(), f.declared_labels as usize);
            assert_eq!(
                f.entries.iter().map(|e| e.strings).sum::<u32>(),
                f.declared_strings
            );
            assert_eq!(f.trailing, 0);
            csf_totals.push(f.declared_labels);
            dump_csf(&mut out, &f);
            digests.entry("csf").or_default().add(&mut out);
        }
    });

    eprintln!(
        "w3d files {w3d_files}, chunk headers {headers}, read {seen}, padding bytes {padding}"
    );
    eprintln!("objects {objects:?}");
    eprintln!("skipped {skipped:x?}; damaged {damaged:?}");
    eprintln!("tga (type, depth, footer) {tga_shapes:?}; unexplained tails {tga_unexplained}");
    eprintln!(
        "dds formats {:?}; with trailing bytes {dds_trailing}",
        dds_formats
            .iter()
            .map(|(k, v)| (String::from_utf8_lossy(k).into_owned(), *v))
            .collect::<Vec<_>>()
    );
    eprintln!("csf labels {csf_totals:?}");

    let mut checked = 0;
    for line in extract_vector_lines(JSON) {
        let Some(kind) = get_str(line, "retail") else {
            continue;
        };
        let d = &digests[kind];
        assert_eq!(
            (d.count, d.failed, format!("{:016x}", d.hash)),
            (
                get_u32(line, "count").unwrap(),
                get_u32(line, "failed").unwrap(),
                get_str(line, "digest").unwrap().to_string()
            ),
            "retail {kind}"
        );
        checked += 1;
    }
    assert_eq!(checked, 7);
    assert!(skipped.is_empty(), "unknown chunks");
    assert_eq!(headers, seen, "chunk headers the readers did not visit");
    // Counts taken by a separate script walking the archives.
    assert_eq!(w3d_files, 9000);
    assert_eq!(
        objects.into_iter().collect::<Vec<_>>(),
        [
            ("animation", 4138),
            ("box", 705),
            ("emitter", 3),
            ("hierarchy", 6573),
            ("hlod", 6597),
            ("mesh", 71487),
        ]
    );
    // Five retail animation files end in bytes that are not chunks.
    assert_eq!(damaged.len(), 5, "{damaged:?}");
    assert!(damaged
        .iter()
        .all(|(n, &notes)| n.contains("uisabotr_") && notes == 2));
    assert_eq!(tga_unexplained, 0);
}
