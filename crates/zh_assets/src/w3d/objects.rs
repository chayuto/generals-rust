//! HLOD definitions, collision boxes and particle emitters
//! (`HLodDefClass`, `W3dBoxStruct`, `ParticleEmitterDefClass`).

use crate::chunk::{c_string, chunks, finish, records, Chunk, Cursor, Read, Report, Truncated};

#[derive(Clone, Debug)]
pub struct SubObject {
    pub bone_index: u32,
    /// Render object name, usually `Container.Mesh`.
    pub name: String,
}

#[derive(Clone, Debug, Default)]
pub struct SubObjectArray {
    pub max_screen_size: f32,
    pub models: Vec<SubObject>,
}

#[derive(Clone, Debug, Default)]
pub struct HLod {
    pub version: u32,
    pub name: String,
    pub hierarchy_name: String,
    /// In file order, which is the order the engine keeps.
    pub lods: Vec<SubObjectArray>,
    pub aggregates: Option<SubObjectArray>,
    pub proxies: Option<SubObjectArray>,
}

fn sub_object_array(chunk: &Chunk, path: &[u32], report: &mut Report) -> Read<SubObjectArray> {
    let mut p = path.to_vec();
    p.push(chunk.id);
    let list = chunks(chunk.data, &p, report);
    let first = list.first().ok_or(Truncated)?;
    if first.id != 0x703 {
        return Err(Truncated);
    }
    let mut c = Cursor::new(first.data);
    let count = c.u32()?;
    let mut array = SubObjectArray {
        max_screen_size: c.f32()?,
        models: Vec::new(),
    };
    finish(&c, "array header", report);
    for sub in &list[1..] {
        if sub.id != 0x704 {
            report.skip(&p, sub);
            continue;
        }
        let mut c = Cursor::new(sub.data);
        array.models.push(SubObject {
            bone_index: c.u32()?,
            name: c.name(32)?,
        });
        finish(&c, "sub object", report);
    }
    if array.models.len() != count as usize {
        return Err(Truncated);
    }
    Ok(array)
}

pub fn parse_hlod(data: &[u8], path: &[u32], report: &mut Report) -> Read<HLod> {
    let list = chunks(data, path, report);
    let first = list.first().ok_or(Truncated)?;
    if first.id != 0x701 {
        return Err(Truncated);
    }
    let mut c = Cursor::new(first.data);
    let version = c.u32()?;
    let lod_count = c.u32()? as usize;
    let mut h = HLod {
        version,
        name: c.name(16)?,
        hierarchy_name: c.name(16)?,
        ..HLod::default()
    };
    finish(&c, "hlod header", report);
    let mut rest = list[1..].iter();
    for _ in 0..lod_count {
        let chunk = rest.next().filter(|c| c.id == 0x702).ok_or(Truncated)?;
        h.lods.push(sub_object_array(chunk, path, report)?);
    }
    for chunk in rest {
        match chunk.id {
            0x705 => h.aggregates = Some(sub_object_array(chunk, path, report)?),
            0x706 => h.proxies = Some(sub_object_array(chunk, path, report)?),
            _ => report.skip(path, chunk),
        }
    }
    Ok(h)
}

#[derive(Clone, Debug)]
pub struct CollisionBox {
    pub version: u32,
    pub attributes: u32,
    pub name: String,
    pub color: [u8; 4],
    pub center: [f32; 3],
    pub extent: [f32; 3],
}

pub fn parse_box(data: &[u8], report: &mut Report) -> Read<CollisionBox> {
    let mut c = Cursor::new(data);
    let b = CollisionBox {
        version: c.u32()?,
        attributes: c.u32()?,
        name: c.name(32)?,
        color: c.array()?,
        center: c.floats()?,
        extent: c.floats()?,
    };
    finish(&c, "box", report);
    Ok(b)
}

#[derive(Clone, Debug)]
pub struct EmitterInfo {
    pub texture: String,
    /// Start size, end size, lifetime, emission rate, max emissions, velocity
    /// random, position random, fade time, gravity, elasticity.
    pub params: [f32; 10],
    pub velocity: [f32; 3],
    pub acceleration: [f32; 3],
    pub start_color: [u8; 4],
    pub end_color: [u8; 4],
}

#[derive(Clone, Copy, Debug)]
pub struct VolumeRandomizer {
    pub class_id: u32,
    pub values: [f32; 3],
    pub reserved: [u32; 4],
}

#[derive(Clone, Debug)]
pub struct EmitterInfoV2 {
    pub burst_size: u32,
    pub creation_volume: VolumeRandomizer,
    pub velocity_random: VolumeRandomizer,
    pub outward_velocity: f32,
    pub velocity_inherit: f32,
    pub shader: [u8; 16],
    pub render_mode: u32,
    pub frame_mode: u32,
    pub reserved: [u32; 6],
}

/// A keyframe list: `(time, value)` pairs and the random range around them.
#[derive(Clone, Debug, Default)]
pub struct Keys {
    pub random: f32,
    /// Only the rotation list has one.
    pub orientation_random: f32,
    pub keys: Vec<(f32, f32)>,
}

#[derive(Clone, Debug, Default)]
pub struct EmitterProps {
    pub color_random: [u8; 4],
    pub opacity_random: f32,
    pub size_random: f32,
    pub colors: Vec<(f32, [u8; 4])>,
    pub opacities: Vec<(f32, f32)>,
    pub sizes: Vec<(f32, f32)>,
}

#[derive(Clone, Debug, Default)]
pub struct Emitter {
    pub version: u32,
    pub name: String,
    pub user_type: u32,
    pub user_string: String,
    pub info: Option<EmitterInfo>,
    pub info_v2: Option<EmitterInfoV2>,
    pub props: Option<EmitterProps>,
    /// Flags, subdivision level, then five floats and nine reserved words.
    pub line_properties: Option<[u32; 16]>,
    pub rotations: Option<Keys>,
    pub frames: Option<Keys>,
    pub blur_times: Option<Keys>,
}

fn randomizer(c: &mut Cursor) -> Read<VolumeRandomizer> {
    Ok(VolumeRandomizer {
        class_id: c.u32()?,
        values: c.floats()?,
        reserved: c.uints()?,
    })
}

/// A header (`count`, `random`, one or two more words) followed by the keys. The
/// engine reads `count` keys and one more for time zero.
fn keys(
    data: &[u8],
    rotation: bool,
    header_words: usize,
    what: &str,
    report: &mut Report,
) -> Read<Keys> {
    let mut c = Cursor::new(data);
    let _count = c.u32()?;
    let mut k = Keys {
        random: c.f32()?,
        ..Keys::default()
    };
    if rotation {
        k.orientation_random = c.f32()?;
        c.u32()?;
    } else {
        for _ in 0..header_words {
            c.u32()?;
        }
    }
    k.keys = records(c.bytes(c.remaining())?, what, report, |c| {
        Ok((c.f32()?, c.f32()?))
    });
    Ok(k)
}

pub fn parse_emitter(data: &[u8], path: &[u32], report: &mut Report) -> Read<Emitter> {
    let mut e = Emitter::default();
    for chunk in chunks(data, path, report) {
        let d = chunk.data;
        let mut c = Cursor::new(d);
        match chunk.id {
            0x501 => {
                e.version = c.u32()?;
                e.name = c.name(16)?;
            }
            0x502 => {
                e.user_type = c.u32()?;
                let _size = c.u32()?;
                e.user_string = c_string(c.bytes(c.remaining())?);
            }
            0x503 => {
                e.info = Some(EmitterInfo {
                    texture: c.name(260)?,
                    params: c.floats()?,
                    velocity: c.floats()?,
                    acceleration: c.floats()?,
                    start_color: c.array()?,
                    end_color: c.array()?,
                })
            }
            0x504 => {
                e.info_v2 = Some(EmitterInfoV2 {
                    burst_size: c.u32()?,
                    creation_volume: randomizer(&mut c)?,
                    velocity_random: randomizer(&mut c)?,
                    outward_velocity: c.f32()?,
                    velocity_inherit: c.f32()?,
                    shader: c.array()?,
                    render_mode: c.u32()?,
                    frame_mode: c.u32()?,
                    reserved: c.uints()?,
                })
            }
            0x505 => {
                let color_count = c.u32()?;
                let opacity_count = c.u32()?;
                let size_count = c.u32()?;
                let mut p = EmitterProps {
                    color_random: c.array()?,
                    opacity_random: c.f32()?,
                    size_random: c.f32()?,
                    ..EmitterProps::default()
                };
                c.bytes(16)?;
                for _ in 0..color_count {
                    p.colors.push((c.f32()?, c.array()?));
                }
                for _ in 0..opacity_count {
                    p.opacities.push((c.f32()?, c.f32()?));
                }
                for _ in 0..size_count {
                    p.sizes.push((c.f32()?, c.f32()?));
                }
                e.props = Some(p);
            }
            0x509 => e.line_properties = Some(c.uints()?),
            0x50a => {
                e.rotations = Some(keys(d, true, 0, "rotation keys", report)?);
                continue;
            }
            0x50b => {
                e.frames = Some(keys(d, false, 2, "frame keys", report)?);
                continue;
            }
            0x50c => {
                e.blur_times = Some(keys(d, false, 1, "blur keys", report)?);
                continue;
            }
            _ => {
                report.skip(path, &chunk);
                continue;
            }
        }
        finish(&c, "emitter chunk", report);
    }
    Ok(e)
}
