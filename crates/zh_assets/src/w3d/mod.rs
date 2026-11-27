//! W3D model files (`WW3D2/w3d_file.h`).

pub mod anim;
pub mod mesh;
pub mod objects;

use crate::chunk::{chunks, Report};

/// `W3D_MAKE_VERSION`.
pub const fn version(major: u32, minor: u32) -> u32 {
    (major << 16) | minor
}

#[derive(Clone, Debug)]
pub enum Object {
    Mesh(Box<mesh::Mesh>),
    Hierarchy(anim::Hierarchy),
    Animation(anim::Animation),
    HLod(objects::HLod),
    Box(objects::CollisionBox),
    Emitter(Box<objects::Emitter>),
}

#[derive(Clone, Debug, Default)]
pub struct W3dFile {
    /// Top-level objects in file order.
    pub objects: Vec<Object>,
    pub report: Report,
}

/// Reads every object of a file. Never fails: what cannot be read is in the report.
/// HMODEL, morph animation, collection, aggregate, light, dazzle and other
/// object kinds no retail file holds are reported as skipped.
pub fn parse(data: &[u8]) -> W3dFile {
    let mut file = W3dFile::default();
    let mut report = Report::default();
    for chunk in chunks(data, &[], &mut report) {
        let path = [chunk.id];
        let d = chunk.data;
        let object = match chunk.id {
            0x000 => mesh::parse(d, &path, &mut report).map(|m| Object::Mesh(Box::new(m))),
            0x100 => anim::parse_hierarchy(d, &path, &mut report).map(Object::Hierarchy),
            0x200 => anim::parse_raw(d, &path, &mut report).map(Object::Animation),
            0x280 => anim::parse_compressed(d, &path, &mut report).map(Object::Animation),
            0x500 => objects::parse_emitter(d, &path, &mut report)
                .map(|e| Object::Emitter(Box::new(e))),
            0x700 => objects::parse_hlod(d, &path, &mut report).map(Object::HLod),
            0x740 => objects::parse_box(d, &mut report).map(Object::Box),
            _ => {
                report.skip(&[], &chunk);
                continue;
            }
        };
        match object {
            Ok(o) => file.objects.push(o),
            Err(_) => report
                .damaged
                .push(format!("object {:#x} could not be read", chunk.id)),
        }
    }
    file.report = report;
    file
}
