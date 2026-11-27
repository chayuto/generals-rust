//! Hierarchies and animations, and the per-frame channel values
//! (`htree.cpp`, `hrawanim.cpp`, `hcanim.cpp`, `motchan.cpp`).

use crate::chunk::{chunks, finish, records, Chunk, Cursor, Read, Report, Truncated};
use crate::wwmath::{fast_slerp, float_to_long, lerp, Quat};

pub const CHANNEL_X: u16 = 0;
pub const CHANNEL_Y: u16 = 1;
pub const CHANNEL_Z: u16 = 2;
pub const CHANNEL_Q: u16 = 6;
pub const FLAVOR_TIMECODED: u16 = 0;
const TIMECODE_FLAG: u32 = 0x8000_0000;

#[derive(Clone, Debug)]
pub struct Pivot {
    pub name: String,
    /// `-1` on the root.
    pub parent: i32,
    pub translation: [f32; 3],
    pub euler: [f32; 3],
    pub rotation: [f32; 4],
}

#[derive(Clone, Debug, Default)]
pub struct Hierarchy {
    pub version: u32,
    pub name: String,
    pub center: [f32; 3],
    /// Includes the root that files before 3.0 leave out.
    pub pivots: Vec<Pivot>,
    /// 3ds Max export data the engine does not read.
    pub fixups: Vec<[f32; 12]>,
}

pub fn parse_hierarchy(data: &[u8], path: &[u32], report: &mut Report) -> Read<Hierarchy> {
    let list = chunks(data, path, report);
    let first = list.first().ok_or(Truncated)?;
    if first.id != 0x101 {
        return Err(Truncated);
    }
    let mut c = Cursor::new(first.data);
    let version = c.u32()?;
    let name = c.name(16)?;
    let num_pivots = c.u32()?;
    let center = c.floats()?;
    finish(&c, "hierarchy header", report);
    let mut h = Hierarchy {
        version,
        name,
        center,
        ..Hierarchy::default()
    };
    let pre30 = version < super::version(3, 0);
    if pre30 {
        h.pivots.push(Pivot {
            name: "RootTransform".into(),
            parent: -1,
            translation: [0.0; 3],
            euler: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
        });
    }
    for chunk in &list[1..] {
        match chunk.id {
            0x102 => {
                let stored = records(chunk.data, "pivots", report, |c| {
                    Ok(Pivot {
                        name: c.name(16)?,
                        parent: c.i32()?,
                        translation: c.floats()?,
                        euler: c.floats()?,
                        rotation: c.floats()?,
                    })
                });
                if stored.len() != num_pivots as usize {
                    return Err(Truncated);
                }
                for mut p in stored {
                    if pre30 {
                        p.parent = p.parent.wrapping_add(1);
                    }
                    h.pivots.push(p);
                }
            }
            0x103 => h.fixups = records(chunk.data, "fixups", report, |c| c.floats()),
            _ => report.skip(path, chunk),
        }
    }
    // The engine follows parent pointers, so a bad index is not usable.
    for (i, p) in h.pivots.iter().enumerate().skip(1) {
        if p.parent < 0 || p.parent as usize >= i {
            return Err(Truncated);
        }
    }
    if h.pivots.is_empty() {
        return Err(Truncated);
    }
    Ok(h)
}

/// `MotionChannelClass`: one value per frame in `[first_frame, last_frame]`.
#[derive(Clone, Debug)]
pub struct RawChannel {
    pub first_frame: u16,
    pub last_frame: u16,
    pub vector_len: u16,
    pub kind: u16,
    pub pivot: u16,
    pub data: Vec<f32>,
}

impl RawChannel {
    fn get(&self, frame: i32, out: &mut [f32]) {
        if frame < i32::from(self.first_frame) || frame > i32::from(self.last_frame) {
            if self.kind == CHANNEL_Q {
                out[..4].copy_from_slice(&[0.0, 0.0, 0.0, 1.0]);
            } else {
                out[0] = 0.0;
            }
        } else {
            let len = usize::from(self.vector_len);
            let at = (frame - i32::from(self.first_frame)) as usize * len;
            for (i, v) in out.iter_mut().take(len).enumerate() {
                *v = self.data[at + i];
            }
        }
    }
}

/// `BitChannelClass`: visibility bits.
#[derive(Clone, Debug)]
pub struct BitChannel {
    pub first_frame: u16,
    pub last_frame: u16,
    pub kind: u16,
    pub pivot: u16,
    pub default: u8,
    pub bits: Vec<u8>,
}

impl BitChannel {
    fn get(&self, frame: i32) -> i32 {
        if frame < i32::from(self.first_frame) || frame > i32::from(self.last_frame) {
            i32::from(self.default)
        } else {
            let bit = (frame - i32::from(self.first_frame)) as usize;
            i32::from(self.bits[bit / 8] & (1 << (bit % 8)) != 0)
        }
    }
}

/// `TimeCodedMotionChannelClass`: keys of `1 + vector_len` words, time code first.
#[derive(Clone, Debug)]
pub struct TimeCodedChannel {
    pub num_time_codes: u32,
    pub pivot: u16,
    pub vector_len: u8,
    pub kind: u8,
    pub data: Vec<u32>,
}

impl TimeCodedChannel {
    fn packet(&self) -> usize {
        usize::from(self.vector_len) + 1
    }

    fn time(&self, key: usize) -> u32 {
        self.data[key * self.packet()] & !TIMECODE_FLAG
    }

    /// Index of the last key at or before `time_code` (`get_index`). The C++ cache
    /// only shortens the search.
    fn key_at(&self, time_code: u32) -> usize {
        let last = self.num_time_codes as usize - 1;
        if time_code >= self.time(last) {
            return last;
        }
        (0..last)
            .rev()
            .find(|&k| time_code >= self.time(k))
            .unwrap_or(0)
    }

    /// The key before `frame` and, when the value moves, the next key and ratio.
    fn span(&self, frame: f32) -> (usize, Option<(usize, f32)>) {
        let key = self.key_at(frame as u32);
        if key == self.num_time_codes as usize - 1 {
            return (key, None);
        }
        let next_word = self.data[(key + 1) * self.packet()];
        if next_word & TIMECODE_FLAG != 0 {
            return (key, None);
        }
        let time1 = (self.data[key * self.packet()] & !TIMECODE_FLAG) as f32;
        let time2 = (next_word & !TIMECODE_FLAG) as f32;
        (key, Some((key + 1, (frame - time1) / (time2 - time1))))
    }

    fn value(&self, key: usize, i: usize) -> f32 {
        f32::from_bits(self.data[key * self.packet() + 1 + i])
    }

    fn get(&self, frame: f32, out: &mut [f32]) {
        let (key, next) = self.span(frame);
        for (i, v) in out
            .iter_mut()
            .take(usize::from(self.vector_len))
            .enumerate()
        {
            *v = match next {
                None => self.value(key, i),
                Some((k2, ratio)) => lerp(self.value(key, i), self.value(k2, i), ratio),
            };
        }
    }

    fn quat(&self, frame: f32) -> Quat {
        let at = |k| {
            Quat([
                self.value(k, 0),
                self.value(k, 1),
                self.value(k, 2),
                self.value(k, 3),
            ])
        };
        match self.span(frame) {
            (key, None) => at(key),
            (key, Some((k2, ratio))) => fast_slerp(at(key), at(k2), ratio),
        }
    }
}

/// `TimeCodedBitChannelClass`: each word is a time code with the bit on top.
#[derive(Clone, Debug)]
pub struct TimeCodedBitChannel {
    pub pivot: u16,
    pub kind: u8,
    pub default: u8,
    pub codes: Vec<u32>,
}

impl TimeCodedBitChannel {
    fn get(&self, frame: i32) -> i32 {
        let mut idx = 0;
        while idx < self.codes.len() && frame >= (self.codes[idx] & !TIMECODE_FLAG) as i32 {
            idx += 1;
        }
        i32::from(self.codes[idx.saturating_sub(1)] & TIMECODE_FLAG != 0)
    }
}

#[derive(Clone, Debug)]
pub enum Channels {
    Raw {
        motion: Vec<RawChannel>,
        visibility: Vec<BitChannel>,
    },
    TimeCoded {
        motion: Vec<TimeCodedChannel>,
        visibility: Vec<TimeCodedBitChannel>,
    },
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub version: u32,
    pub name: String,
    pub hierarchy_name: String,
    pub num_frames: u32,
    pub frame_rate: u32,
    pub channels: Channels,
}

pub fn parse_raw(data: &[u8], path: &[u32], report: &mut Report) -> Read<Animation> {
    let list = chunks(data, path, report);
    let first = list.first().ok_or(Truncated)?;
    if first.id != 0x201 {
        return Err(Truncated);
    }
    let mut c = Cursor::new(first.data);
    let version = c.u32()?;
    let name = c.name(16)?;
    let hierarchy_name = c.name(16)?;
    let num_frames = c.u32()?;
    let frame_rate = c.u32()?;
    finish(&c, "animation header", report);
    let shift = u16::from(version < super::version(3, 0));
    let mut motion = Vec::new();
    let mut visibility = Vec::new();
    for chunk in &list[1..] {
        match chunk.id {
            0x202 => {
                let mut ch = raw_channel(chunk, report)?;
                ch.pivot = ch.pivot.wrapping_add(shift);
                motion.push(ch);
            }
            0x203 => {
                let mut ch = bit_channel(chunk, report)?;
                ch.pivot = ch.pivot.wrapping_add(shift);
                visibility.push(ch);
            }
            _ => report.skip(path, chunk),
        }
    }
    Ok(Animation {
        version,
        name,
        hierarchy_name,
        num_frames,
        frame_rate,
        channels: Channels::Raw { motion, visibility },
    })
}

fn raw_channel(chunk: &Chunk, report: &mut Report) -> Read<RawChannel> {
    let mut c = Cursor::new(chunk.data);
    let first_frame = c.u16()?;
    let last_frame = c.u16()?;
    let vector_len = c.u16()?;
    let kind = c.u16()?;
    let pivot = c.u16()?;
    c.u16()?;
    let frames = u32::from(last_frame)
        .checked_sub(u32::from(first_frame))
        .ok_or(Truncated)?
        + 1;
    let count = frames as usize * usize::from(vector_len);
    if count == 0 || (kind == CHANNEL_Q && vector_len < 4) {
        return Err(Truncated);
    }
    let mut values = Vec::new();
    for _ in 0..count {
        values.push(c.f32()?);
    }
    // Exporters pad the chunk; the engine seeks over the rest.
    report.padding += c.remaining() as u64;
    Ok(RawChannel {
        first_frame,
        last_frame,
        vector_len,
        kind,
        pivot,
        data: values,
    })
}

fn bit_channel(chunk: &Chunk, report: &mut Report) -> Read<BitChannel> {
    let mut c = Cursor::new(chunk.data);
    let first_frame = c.u16()?;
    let last_frame = c.u16()?;
    let kind = c.u16()?;
    let pivot = c.u16()?;
    let default = c.u8()?;
    let count = (u32::from(last_frame)
        .checked_sub(u32::from(first_frame))
        .ok_or(Truncated)?
        + 1)
    .div_ceil(8);
    let bits = c.bytes(count as usize)?.to_vec();
    finish(&c, "bit channel", report);
    Ok(BitChannel {
        first_frame,
        last_frame,
        kind,
        pivot,
        default,
        bits,
    })
}

pub fn parse_compressed(data: &[u8], path: &[u32], report: &mut Report) -> Read<Animation> {
    let list = chunks(data, path, report);
    let first = list.first().ok_or(Truncated)?;
    if first.id != 0x281 {
        return Err(Truncated);
    }
    let mut c = Cursor::new(first.data);
    let version = c.u32()?;
    let name = c.name(16)?;
    let hierarchy_name = c.name(16)?;
    let num_frames = c.u32()?;
    let frame_rate = u32::from(c.u16()?);
    let flavor = c.u16()?;
    finish(&c, "compressed animation header", report);
    let mut motion = Vec::new();
    let mut visibility = Vec::new();
    for chunk in &list[1..] {
        match chunk.id {
            // Adaptive delta is not decoded; no retail file uses it.
            0x282 if flavor == FLAVOR_TIMECODED => motion.push(time_coded_channel(chunk, report)?),
            0x283 => visibility.push(time_coded_bits(chunk, report)?),
            _ => report.skip(path, chunk),
        }
    }
    Ok(Animation {
        version,
        name,
        hierarchy_name,
        num_frames,
        frame_rate,
        channels: Channels::TimeCoded { motion, visibility },
    })
}

fn time_coded_channel(chunk: &Chunk, report: &mut Report) -> Read<TimeCodedChannel> {
    let mut c = Cursor::new(chunk.data);
    let num_time_codes = c.u32()?;
    let pivot = c.u16()?;
    let vector_len = c.u8()?;
    let kind = c.u8()?;
    let words = records(c.bytes(c.remaining())?, "time codes", report, |c| c.u32());
    let need = (num_time_codes as usize)
        .checked_mul(usize::from(vector_len) + 1)
        .ok_or(Truncated)?;
    if num_time_codes == 0
        || words.len() < need
        || (u16::from(kind) == CHANNEL_Q && vector_len != 4)
        || (u16::from(kind) != CHANNEL_Q && vector_len != 1)
    {
        return Err(Truncated);
    }
    Ok(TimeCodedChannel {
        num_time_codes,
        pivot,
        vector_len,
        kind,
        data: words,
    })
}

fn time_coded_bits(chunk: &Chunk, report: &mut Report) -> Read<TimeCodedBitChannel> {
    let mut c = Cursor::new(chunk.data);
    let count = c.u32()?;
    let pivot = c.u16()?;
    let kind = c.u8()?;
    let default = c.u8()?;
    let mut codes = Vec::new();
    for _ in 0..count {
        codes.push(c.u32()?);
    }
    if codes.is_empty() {
        return Err(Truncated);
    }
    finish(&c, "time coded bits", report);
    Ok(TimeCodedBitChannel {
        pivot,
        kind,
        default,
        codes,
    })
}

/// An animation bound to a hierarchy of `num_pivots`: channels for pivots it
/// does not have are dropped and a later channel replaces an earlier one.
pub struct BoundAnimation<'a> {
    animation: &'a Animation,
    /// X, Y, Z, Q and visibility channel of each pivot.
    nodes: Vec<[Option<usize>; 5]>,
}

impl<'a> BoundAnimation<'a> {
    pub fn new(animation: &'a Animation, num_pivots: usize) -> Self {
        let mut nodes = vec![[None; 5]; num_pivots];
        let mut set = |pivot: u16, slot: Option<usize>, i: usize| {
            if let (Some(node), Some(slot)) = (nodes.get_mut(usize::from(pivot)), slot) {
                node[slot] = Some(i);
            }
        };
        let slot = |kind: u16| match kind {
            CHANNEL_X => Some(0),
            CHANNEL_Y => Some(1),
            CHANNEL_Z => Some(2),
            CHANNEL_Q => Some(3),
            _ => None,
        };
        match &animation.channels {
            Channels::Raw { motion, visibility } => {
                for (i, c) in motion.iter().enumerate() {
                    set(c.pivot, slot(c.kind), i);
                }
                for (i, c) in visibility.iter().enumerate() {
                    set(c.pivot, (c.kind == 0).then_some(4), i);
                }
            }
            Channels::TimeCoded { motion, visibility } => {
                for (i, c) in motion.iter().enumerate() {
                    set(c.pivot, slot(u16::from(c.kind)), i);
                }
                for (i, c) in visibility.iter().enumerate() {
                    set(c.pivot, (c.kind == 0).then_some(4), i);
                }
            }
        }
        BoundAnimation { animation, nodes }
    }

    pub fn num_pivots(&self) -> usize {
        self.nodes.len()
    }

    pub fn num_frames(&self) -> u32 {
        self.animation.num_frames
    }

    /// The two raw frames around `frame` and the blend between them.
    fn raw_frames(&self, frame: f32) -> (i32, i32, f32) {
        let frame0 = float_to_long(frame - 0.499999);
        let mut frame1 = frame0.wrapping_add(1);
        let ratio = frame - frame0 as f32;
        if frame1 >= self.animation.num_frames as i32 {
            frame1 = 0;
        }
        (frame0, frame1, ratio)
    }

    pub fn translation(&self, pivot: usize, frame: f32) -> [f32; 3] {
        let node = &self.nodes[pivot];
        let mut out = [0.0f32; 3];
        match &self.animation.channels {
            Channels::Raw { motion, .. } => {
                if node[..3].iter().all(Option::is_none) {
                    return out;
                }
                let (frame0, frame1, ratio) = self.raw_frames(frame);
                let at = |f: i32| {
                    let mut v = [0.0f32; 3];
                    for axis in 0..3 {
                        if let Some(i) = node[axis] {
                            motion[i].get(f, &mut v[axis..]);
                        }
                    }
                    v
                };
                out = at(frame0);
                if ratio == 0.0 {
                    return out;
                }
                let next = at(frame1);
                for axis in 0..3 {
                    out[axis] += (next[axis] - out[axis]) * ratio;
                }
            }
            Channels::TimeCoded { motion, .. } => {
                for axis in 0..3 {
                    if let Some(i) = node[axis] {
                        motion[i].get(frame, &mut out[axis..]);
                    }
                }
            }
        }
        out
    }

    pub fn orientation(&self, pivot: usize, frame: f32) -> Quat {
        let channel = self.nodes[pivot][3];
        match &self.animation.channels {
            Channels::Raw { motion, .. } => {
                let (frame0, frame1, ratio) = self.raw_frames(frame);
                let at = |f: i32| match channel {
                    Some(i) => {
                        let mut v = [0.0f32; 4];
                        motion[i].get(f, &mut v);
                        Quat(v)
                    }
                    None => Quat::IDENTITY,
                };
                let q0 = at(frame0);
                if ratio == 0.0 {
                    return q0;
                }
                // `SPECIAL_GETVEC_AS_QUAT`: a ratio of exactly 1 takes the next frame.
                if ratio == 1.0 {
                    return at(frame1);
                }
                fast_slerp(q0, at(frame1), ratio)
            }
            Channels::TimeCoded { motion, .. } => {
                channel.map_or(Quat::IDENTITY, |i| motion[i].quat(frame))
            }
        }
    }

    pub fn visible(&self, pivot: usize, frame: f32) -> bool {
        let Some(i) = self.nodes[pivot][4] else {
            return true;
        };
        match &self.animation.channels {
            Channels::Raw { visibility, .. } => visibility[i].get(frame as i32) == 1,
            Channels::TimeCoded { visibility, .. } => visibility[i].get(frame as i32) == 1,
        }
    }
}
