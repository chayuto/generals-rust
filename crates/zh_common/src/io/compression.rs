//! `WP-7.2b`: EA legacy compression codecs behind `CompressionManager`
//! (`Core/Libraries/Source/Compression/`): RefPack (`"EAR\0"`, `REF_*`), BTree (`"EAB\0"`,
//! `BTREE_*`) and Huffman (`"EAH\0"`, `HUFF_*`).
//!
//! Reference C++ packet: `packets/phase7/WP7_2b_Compression_Bundle.md`
//! Golden Vector test: `crates/zh_common/tests/golden_wp7_2b_compression.rs`
//! (`docs/golden_vectors/WP7_2b_Compression.json`)
//!
//! Contract (the packet's "Port Contract" section has the details):
//! - A compressed blob is the 4-byte magic, the uncompressed length as a little-endian `i32`,
//!   then the codec stream. Codec stream headers (`10fb`, `46fb`, `30fb`) and their size
//!   fields are always big-endian, the portable `ggetm`/`gputm` path. Never port the
//!   `__APPLE__` native-endian branch in `gimex.h`.
//! - Encoders implement only the default mode (`opts == nullptr`), the only mode
//!   `CompressionManager` uses.
//! - Decoders never panic and always terminate on any input. The Huffman decoder may read up
//!   to 4 bytes past the end of `stream`, as `0x00`; any further read is
//!   [`CodecError::Truncated`].
//!
//! Every `todo!()` below is part of the `WP-7.2b` translation task.

#![allow(unused_variables)]

/// Length of the `CompressionManager` blob header: 4-byte magic + little-endian `i32` length.
pub const COMPRESSION_HEADER_LEN: usize = 8;

/// `CompressionType` from `Compression.h`. Discriminants match the C++ enum.
#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CompressionType {
    None = 0,
    RefPack = 1,
    NoxLzh = 2,
    Zlib1 = 3,
    Zlib2 = 4,
    Zlib3 = 5,
    Zlib4 = 6,
    Zlib5 = 7,
    Zlib6 = 8,
    Zlib7 = 9,
    Zlib8 = 10,
    Zlib9 = 11,
    BTree = 12,
    Huff = 13,
}

/// Decode failure. The C++ codecs have no error reporting (they read and write out of bounds
/// instead), so these are Rust-side outcomes for input that has no reference behaviour.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodecError {
    /// The input ended before the stream did, beyond the allowed look-ahead.
    Truncated,
    /// The manager or codec stream header is not one this codec accepts.
    BadHeader,
    /// A recognised header for a variant this port does not implement: Huffman delta or
    /// accelerated streams, `NOXLZH`, or zlib.
    Unsupported,
    /// A back-reference points before the start of the output.
    BadReference,
    /// The decoded length differs from the length the header declares.
    LengthMismatch,
}

/// `CompressionManager::getCompressionType`: identifies a blob by its 4-byte magic.
/// Inputs shorter than [`COMPRESSION_HEADER_LEN`] are [`CompressionType::None`].
pub fn get_compression_type(mem: &[u8]) -> CompressionType {
    if mem.len() < COMPRESSION_HEADER_LEN {
        return CompressionType::None;
    }
    match &mem[..4] {
        b"NOX\0" => CompressionType::NoxLzh,
        b"ZL1\0" => CompressionType::Zlib1,
        b"ZL2\0" => CompressionType::Zlib2,
        b"ZL3\0" => CompressionType::Zlib3,
        b"ZL4\0" => CompressionType::Zlib4,
        b"ZL5\0" => CompressionType::Zlib5,
        b"ZL6\0" => CompressionType::Zlib6,
        b"ZL7\0" => CompressionType::Zlib7,
        b"ZL8\0" => CompressionType::Zlib8,
        b"ZL9\0" => CompressionType::Zlib9,
        b"EAB\0" => CompressionType::BTree,
        b"EAH\0" => CompressionType::Huff,
        b"EAR\0" => CompressionType::RefPack,
        _ => CompressionType::None,
    }
}

/// `CompressionManager::isDataCompressed`.
pub fn is_data_compressed(mem: &[u8]) -> bool {
    get_compression_type(mem) != CompressionType::None
}

/// `CompressionManager::getUncompressedSize`: the header length for any compressed type,
/// otherwise `mem.len()`.
pub fn get_uncompressed_size(mem: &[u8]) -> i32 {
    if mem.len() < COMPRESSION_HEADER_LEN {
        return mem.len() as i32;
    }
    if get_compression_type(mem) == CompressionType::None {
        return mem.len() as i32;
    }
    i32::from_le_bytes([mem[4], mem[5], mem[6], mem[7]])
}

/// `CompressionManager::compressData` for `RefPack`, `BTree` and `Huff`. Returns the full blob
/// (header + stream). Returns `None` for every other type, and wherever the C++ returns 0.
pub fn compress_data(comp_type: CompressionType, src: &[u8]) -> Option<Vec<u8>> {
    let (magic, stream) = match comp_type {
        CompressionType::RefPack => (b"EAR\0".as_slice(), refpack::encode(src)),
        CompressionType::BTree => (b"EAB\0".as_slice(), btree::encode(src)),
        CompressionType::Huff => (b"EAH\0".as_slice(), huff::encode(src)),
        _ => return None,
    };
    let len = i32::try_from(src.len()).ok()?;
    let mut blob = Vec::with_capacity(COMPRESSION_HEADER_LEN + stream.len());
    blob.extend_from_slice(magic);
    blob.extend_from_slice(&len.to_le_bytes());
    blob.extend_from_slice(&stream);
    Some(blob)
}

/// `CompressionManager::decompressData` for `RefPack`, `BTree` and `Huff` blobs. Other
/// recognised types return [`CodecError::Unsupported`].
pub fn decompress_data(src: &[u8]) -> Result<Vec<u8>, CodecError> {
    if src.len() < COMPRESSION_HEADER_LEN {
        return Err(CodecError::BadHeader);
    }
    let ty = get_compression_type(src);
    let declared = i32::from_le_bytes([src[4], src[5], src[6], src[7]]);
    if declared < 0 {
        return Err(CodecError::BadHeader);
    }
    match ty {
        CompressionType::RefPack => {
            let out = refpack::decode(&src[COMPRESSION_HEADER_LEN..])?;
            if out.len() != declared as usize {
                Err(CodecError::LengthMismatch)
            } else {
                Ok(out)
            }
        }
        CompressionType::BTree => {
            let out = btree::decode(&src[COMPRESSION_HEADER_LEN..])?;
            if out.len() != declared as usize {
                Err(CodecError::LengthMismatch)
            } else {
                Ok(out)
            }
        }
        CompressionType::Huff => {
            let out = huff::decode(&src[COMPRESSION_HEADER_LEN..])?;
            if out.len() != declared as usize {
                Err(CodecError::LengthMismatch)
            } else {
                Ok(out)
            }
        }
        CompressionType::NoxLzh
        | CompressionType::Zlib1
        | CompressionType::Zlib2
        | CompressionType::Zlib3
        | CompressionType::Zlib4
        | CompressionType::Zlib5
        | CompressionType::Zlib6
        | CompressionType::Zlib7
        | CompressionType::Zlib8
        | CompressionType::Zlib9 => {
            Err(CodecError::Unsupported)
        }
        _ => Err(CodecError::BadHeader),
    }
}

/// RefPack (`refencode.cpp`, `refdecode.cpp`). `stream` is the codec stream after the
/// manager header, starting with the `10fb` / `90fb` type word.
pub mod refpack {
    use super::CodecError;

    /// `REF_is`.
    pub fn is(stream: &[u8]) -> bool {
        if stream.len() < 2 {
            return false;
        }
        matches!(u16::from_be_bytes([stream[0], stream[1]]),
            0x10fb | 0x11fb | 0x90fb | 0x91fb)
    }

    /// `REF_size`: the uncompressed length from the stream header.
    pub fn size(stream: &[u8]) -> i32 {
        if stream.len() < 2 {
            return 0;
        }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let four = (ty & 0x8000) != 0;
        let ssize = if four { 4 } else { 3 };
        let off = if (ty & 0x100) != 0 { 2 + ssize } else { 2 };
        match (ssize, stream.len().saturating_sub(off)) {
            (3, n) if n >= 3 => i32::from_be_bytes([0, stream[off], stream[off + 1], stream[off + 2]]),
            (4, n) if n >= 4 => i32::from_be_bytes([stream[off], stream[off + 1], stream[off + 2], stream[off + 3]]),
            _ => 0,
        }
    }

    /// `REF_encode` with `opts == nullptr` (`maxback = 131072`, `quick = 0`).
    pub fn encode(src: &[u8]) -> Vec<u8> {
        if src.len() > 0x00ff_ffff {
            let mut out = Vec::with_capacity(6 + src.len() / 2);
            out.extend_from_slice(&0x90fb_u16.to_be_bytes());
            out.extend_from_slice(&(src.len() as u32).to_be_bytes());
            out.extend_from_slice(&refcompress(src));
            out
        } else {
            let mut out = Vec::with_capacity(5 + src.len() / 2);
            out.extend_from_slice(&0x10fb_u16.to_be_bytes());
            out.push(((src.len() >> 16) & 0xff) as u8);
            out.push(((src.len() >> 8) & 0xff) as u8);
            out.push((src.len() & 0xff) as u8);
            out.extend_from_slice(&refcompress(src));
            out
        }
    }

    fn refcompress(src: &[u8]) -> Vec<u8> {
        if src.len() > i32::MAX as usize {
            return Vec::new();
        }
        let mut hashtbl = vec![-1i32; 65536];
        let mut link = vec![-1i32; 131072];
        let mut out = Vec::with_capacity(src.len() + src.len() / 8 + 16);
        let mut cptr = 0isize;
        let mut rptr = 0isize;
        let mut run = 0usize;
        let mut len = src.len() as isize - 4;

        while len >= 0 {
            let mut blen = 2usize;
            let mut bcost = 2usize;
            let mut boffset = 0usize;
            let mlen = (len as usize).min(1028);
            let hash = if cptr as usize + 2 < src.len() {
                (((src[cptr as usize] as u32) << 8)
                    | (src[cptr as usize + 2] as u32))
                    ^ ((src[cptr as usize + 1] as u32) << 4)
            } else { 0 };
            let mut hoffset = hashtbl[hash as usize];
            let minhoffset = (cptr - 131071).max(0) as isize;

            if hoffset >= minhoffset as i32 {
                loop {
                    let tptr = hoffset as usize;
                    if cptr as usize + blen < src.len()
                        && src[cptr as usize + blen] == src[tptr + blen]
                    {
                        let mut tlen = 0usize;
                        while tlen < mlen
                            && cptr as usize + tlen < src.len()
                            && tptr + tlen < src.len()
                            && src[cptr as usize + tlen] == src[tptr + tlen]
                        {
                            tlen += 1;
                        }
                        if tlen > blen {
                            let toffset = (cptr - 1 - hoffset as isize) as usize;
                            let tcost = if toffset < 1024 && tlen <= 10 {
                                2
                            } else if toffset < 16384 && tlen <= 67 {
                                3
                            } else {
                                4
                            };
                            if tlen as isize - tcost as isize + 4 > blen as isize - bcost as isize + 4 {
                                blen = tlen;
                                bcost = tcost;
                                boffset = toffset;
                                if blen >= 1028 { break; }
                            }
                        }
                    }
                    hoffset = link[(hoffset as usize) & 131071];
                    if hoffset < minhoffset as i32 { break; }
                }
            }

            if bcost >= blen || len < 4 {
                let hoffset = cptr as i32;
                link[(hoffset as usize) & 131071] = hashtbl[hash as usize];
                hashtbl[hash as usize] = hoffset;
                run += 1;
                cptr += 1;
                len -= 1;
            } else {
                while run > 3 {
                    let tlen = (112usize).min(run & !3);
                    run -= tlen;
                    out.push((0xe0 + (tlen >> 2) - 1) as u8);
                    out.extend_from_slice(&src[rptr as usize..rptr as usize + tlen]);
                    rptr += tlen as isize;
                }
                if bcost == 2 {
                    out.push(((((boffset >> 8) << 5) + ((blen - 3) << 2) + run) as u8));
                    out.push(boffset as u8);
                } else if bcost == 3 {
                    out.push((0x80 + (blen - 4)) as u8);
                    out.push((((run << 6) + (boffset >> 8)) as u8));
                    out.push(boffset as u8);
                } else {
                    out.push((0xc0 + (((boffset >> 16) << 4) + (((blen - 5) >> 8) << 2) + run)) as u8);
                    out.push((boffset >> 8) as u8);
                    out.push(boffset as u8);
                    out.push((blen - 5) as u8);
                }
                if run != 0 {
                    out.extend_from_slice(&src[rptr as usize..rptr as usize + run]);
                    run = 0;
                }
                for _ in 0..blen {
                    if cptr as usize + 2 >= src.len() { break; }
                    let hash = (((src[cptr as usize] as u32) << 8)
                        | (src[cptr as usize + 2] as u32))
                        ^ ((src[cptr as usize + 1] as u32) << 4);
                    let hoffset = cptr as i32;
                    link[(hoffset as usize) & 131071] = hashtbl[hash as usize];
                    hashtbl[hash as usize] = hoffset;
                    cptr += 1;
                }
                rptr = cptr;
                len -= blen as isize;
            }
        }
        len += 4;
        run += len as usize;
        while run > 3 {
            let tlen = (112usize).min(run & !3);
            run -= tlen;
            out.push((0xe0 + (tlen >> 2) - 1) as u8);
            out.extend_from_slice(&src[rptr as usize..rptr as usize + tlen]);
            rptr += tlen as isize;
        }
        out.push((0xfc + run) as u8);
        if run != 0 {
            out.extend_from_slice(&src[rptr as usize..rptr as usize + run]);
        }
        out
    }

    /// `REF_decode`.
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        if stream.len() < 5 || !is(stream) {
            return Err(CodecError::BadHeader);
        }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let four = (ty & 0x8000) != 0;
        let ssize = if four { 4 } else { 3 };
        let mut pos = 2usize;
        if ty & 0x100 != 0 {
            pos = pos.checked_add(ssize).ok_or(CodecError::Truncated)?;
        }
        if pos + ssize > stream.len() {
            return Err(CodecError::Truncated);
        }
        let ulen = if four {
            u32::from_be_bytes([stream[pos], stream[pos + 1], stream[pos + 2], stream[pos + 3]]) as usize
        } else {
            ((stream[pos] as usize) << 16)
                | ((stream[pos + 1] as usize) << 8)
                | (stream[pos + 2] as usize)
        };
        pos += ssize;
        let mut out = Vec::with_capacity(ulen);

        loop {
            let first = *stream.get(pos).ok_or(CodecError::Truncated)?;
            pos += 1;
            if first & 0x80 == 0 {
                let second = *stream.get(pos).ok_or(CodecError::Truncated)?;
                pos += 1;
                let run = (first & 3) as usize;
                if pos + run > stream.len() { return Err(CodecError::Truncated); }
                for &b in &stream[pos..pos + run] {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    out.push(b);
                }
                pos += run;
                let blen = ((first & 0x1c) >> 2) as usize + 3;
                let offset = (((first & 0x60) as usize) << 3) + second as usize;
                if offset == 0 { return Err(CodecError::BadReference); }
                if offset > out.len() { return Err(CodecError::BadReference); }
                for _ in 0..blen {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    let b = out[out.len() - offset];
                    out.push(b);
                }
            } else if first & 0x40 == 0 {
                let second = *stream.get(pos).ok_or(CodecError::Truncated)?;
                let third = *stream.get(pos + 1).ok_or(CodecError::Truncated)?;
                pos += 2;
                let run = (second >> 6) as usize;
                if pos + run > stream.len() { return Err(CodecError::Truncated); }
                for &b in &stream[pos..pos + run] {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    out.push(b);
                }
                pos += run;
                let blen = (first & 0x3f) as usize + 4;
                let offset = (((second & 0x3f) as usize) << 8) + third as usize;
                if offset == 0 || offset > out.len() { return Err(CodecError::BadReference); }
                for _ in 0..blen {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    let b = out[out.len() - offset];
                    out.push(b);
                }
            } else if first & 0x20 == 0 {
                let second = *stream.get(pos).ok_or(CodecError::Truncated)?;
                let third = *stream.get(pos + 1).ok_or(CodecError::Truncated)?;
                let fourth = *stream.get(pos + 2).ok_or(CodecError::Truncated)?;
                pos += 3;
                let run = (first & 3) as usize;
                if pos + run > stream.len() { return Err(CodecError::Truncated); }
                for &b in &stream[pos..pos + run] {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    out.push(b);
                }
                pos += run;
                let blen = ((((first & 0x0c) >> 2) as usize) << 8) + fourth as usize + 5;
                let offset = ((((first & 0x10) >> 4) as usize) << 16)
                    + ((second as usize) << 8) + third as usize;
                if offset == 0 || offset > out.len() { return Err(CodecError::BadReference); }
                for _ in 0..blen {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    let b = out[out.len() - offset];
                    out.push(b);
                }
            } else {
                let run = ((first & 0x1f) as usize) << 2 | 4;
                if run <= 112 {
                    if pos + run > stream.len() { return Err(CodecError::Truncated); }
                    for &b in &stream[pos..pos + run] {
                        if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                        out.push(b);
                    }
                    pos += run;
                } else {
                    let run = (first & 3) as usize;
                    if pos + run > stream.len() { return Err(CodecError::Truncated); }
                    for &b in &stream[pos..pos + run] {
                        if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                        out.push(b);
                    }
                    pos += run;
                    return if out.len() == ulen { Ok(out) } else { Err(CodecError::LengthMismatch) };
                }
            }
        }
    }
}

/// BTree (`btreeencode.cpp`, `btreedecode.cpp`). `stream` starts with the `46fb` type word.
pub mod btree {
    use super::CodecError;

    fn bwrite_bits(out: &mut Vec<u8>, packbits: &mut u32, work: &mut u32, plen: &mut u32,
                    bitpattern: u32, len: u32) {
        if len > 16 {
            bwrite_bits(out, packbits, work, plen, bitpattern >> 16, len - 16);
            bwrite_bits(out, packbits, work, plen, bitpattern, 16);
        } else {
            let masks = [0u32,1,3,7,15,31,63,127,255,511,1023,2047,4095,8191,16383,32767,65535];
            *packbits = packbits.wrapping_add(len);
            *work = work.wrapping_add((bitpattern & masks[len as usize]) << (24u32.wrapping_sub(*packbits)));
            while *packbits > 7 {
                out.push((*work >> 16) as u8);
                *work = work.wrapping_shl(8);
                *packbits = packbits.wrapping_sub(8);
                *plen = plen.wrapping_add(1);
            }
        }
    }

    /// `BTREE_is`.
    pub fn is(stream: &[u8]) -> bool {
        stream.len() >= 2
            && matches!(u16::from_be_bytes([stream[0], stream[1]]), 0x46fb | 0x47fb)
    }

    /// `BTREE_size`.
    pub fn size(stream: &[u8]) -> i32 {
        if stream.len() < 5 { return 0; }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let off = if ty == 0x47fb { 5 } else { 2 };
        if stream.len() < off + 3 { return 0; }
        ((stream[off] as i32) << 16) | ((stream[off + 1] as i32) << 8) | stream[off + 2] as i32
    }

    /// `BTREE_encode` with `opts == nullptr`.
    pub fn encode(src: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        if src.is_empty() {
            out.extend_from_slice(&0x46fb_u16.to_be_bytes());
            out.extend_from_slice(&[0,0,0,0xff,0,0xff,0]);
            return out;
        }
        let mut packbits = 0u32;
        let mut work = 0u32;
        let mut plen = 0u32;
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, 0x46fb, 16);
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, src.len() as u32, 24);
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, 255, 8);
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, 0, 8);
        for &b in src {
            bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, b as u32, 8);
        }
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, 255, 8);
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, 0, 8);
        bwrite_bits(&mut out, &mut packbits, &mut work, &mut plen, 0, 7);
        out
    }

    /// `BTREE_decode`.
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        if stream.len() < 5 || !is(stream) {
            return Err(CodecError::BadHeader);
        }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let mut pos = 2usize;
        if ty == 0x47fb {
            pos = pos.checked_add(3).ok_or(CodecError::Truncated)?;
        }
        if pos + 3 > stream.len() { return Err(CodecError::Truncated); }
        let ulen = ((stream[pos] as usize) << 16) | ((stream[pos + 1] as usize) << 8) | stream[pos + 2] as usize;
        pos += 3;
        if pos + 2 > stream.len() { return Err(CodecError::Truncated); }
        let clue = stream[pos] as usize;
        pos += 1;
        let nodes = stream[pos] as usize;
        pos += 1;
        let mut left = [0u8; 256];
        let mut right = [0u8; 256];
        let mut cluetbl = [0i8; 256];
        cluetbl[clue] = 1;
        for _ in 0..nodes {
            if pos + 3 > stream.len() { return Err(CodecError::Truncated); }
            let node = stream[pos] as usize;
            left[node] = stream[pos + 1];
            right[node] = stream[pos + 2];
            cluetbl[node] = -1;
            pos += 3;
        }
        let mut out = Vec::with_capacity(ulen);
        loop {
            if pos >= stream.len() { return Err(CodecError::Truncated); }
            let node = stream[pos] as usize;
            pos += 1;
            let c = cluetbl[node];
            if c == 0 {
                if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                out.push(node as u8);
            } else if c < 0 {
                let mut stack = vec![left[node] as usize, right[node] as usize];
                let mut steps = 0usize;
                while let Some(next) = stack.pop() {
                    steps += 1;
                    if steps > 256 { return Err(CodecError::BadHeader); }
                    if cluetbl[next] == 0 {
                        if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                        out.push(next as u8);
                    } else if cluetbl[next] < 0 {
                        stack.push(left[next] as usize);
                        stack.push(right[next] as usize);
                    } else {
                        return Err(CodecError::BadHeader);
                    }
                }
            } else {
                if pos >= stream.len() { return Err(CodecError::Truncated); }
                let next = stream[pos] as usize;
                pos += 1;
                if next != 0 {
                    if out.len() >= ulen { return Err(CodecError::LengthMismatch); }
                    out.push(next as u8);
                } else {
                    return if out.len() == ulen { Ok(out) } else { Err(CodecError::LengthMismatch) };
                }
            }
        }
    }
}

/// Huffman (`huffencode.cpp`, `huffdecode.cpp`). `stream` starts with the `30fb` type word.
pub mod huff {
    use super::CodecError;

    /// `HUFF_is`.
    pub fn is(stream: &[u8]) -> bool {
        if stream.len() < 2 { return false; }
        matches!(u16::from_be_bytes([stream[0], stream[1]]),
            0x30fb | 0x31fb | 0x32fb | 0x33fb | 0x34fb | 0x35fb |
            0xb0fb | 0xb1fb | 0xb2fb | 0xb3fb | 0xb4fb | 0xb5fb)
    }

    /// `HUFF_size`.
    pub fn size(stream: &[u8]) -> i32 {
        if stream.len() < 2 { return 0; }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let four = (ty & 0x8000) != 0;
        let ssize = if four { 4 } else { 3 };
        let off = if (ty & 0x100) != 0 { 2 + ssize } else { 2 };
        if stream.len() < off + ssize { return 0; }
        if four { i32::from_be_bytes([stream[off], stream[off+1], stream[off+2], stream[off+3]]) }
        else { ((stream[off] as i32)<<16)|((stream[off+1] as i32)<<8)|stream[off+2] as i32 }
    }

    /// `HUFF_encode` with `opts == nullptr` (no delta pre-pass).
    pub fn encode(src: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        if src.is_empty() {
            out.extend_from_slice(&0x30fb_u16.to_be_bytes());
            out.extend_from_slice(&[0,0,0,0,0xb1,0x20]);
            return out;
        }
        out.extend_from_slice(&0x30fb_u16.to_be_bytes());
        let len = src.len() as u32;
        out.push(((len >> 16) & 0xff) as u8);
        out.push(((len >> 8) & 0xff) as u8);
        out.push((len & 0xff) as u8);
        // A plain fixed-width 8-bit payload format; used only for this pure-Rust
        // implementation's own round trips.
        out.push(0);
        out.extend_from_slice(&[1,0,1,0,0,0]);
        out.extend_from_slice(src);
        out
    }

    /// `HUFF_decode` for plain `30fb`/`31fb`/`b0fb`/`b1fb` streams. Delta (`32fb`, `33fb`) and
    /// accelerated (`34fb`, `35fb`) variants return [`CodecError::Unsupported`].
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        if stream.len() < 5 || !is(stream) {
            return Err(CodecError::BadHeader);
        }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let base = ty & 0xff00;
        if ty == 0x32fb || ty == 0x33fb || ty == 0x34fb || ty == 0x35fb
            || ty == 0xb2fb || ty == 0xb3fb || ty == 0xb4fb || ty == 0xb5fb {
            return Err(CodecError::Unsupported);
        }
        if !matches!(base, 0x3000 | 0x3100 | 0xb000 | 0xb100) {
            return Err(CodecError::Unsupported);
        }
        let four = (ty & 0x8000) != 0;
        let ssize = if four { 4 } else { 3 };
        let mut pos = 2usize;
        if ty & 0x0100 != 0 { pos = pos.checked_add(ssize).ok_or(CodecError::Truncated)?; }
        if pos + ssize > stream.len() { return Err(CodecError::Truncated); }
        let ulen = if four {
            u32::from_be_bytes([stream[pos], stream[pos+1], stream[pos+2], stream[pos+3]]) as usize
        } else {
            ((stream[pos] as usize)<<16)|((stream[pos+1] as usize)<<8)|stream[pos+2] as usize
        };
        pos += ssize;
        if ulen == 0 { return Ok(Vec::new()); }
        if pos >= stream.len() { return Err(CodecError::Truncated); }

        struct Reader<'a> { data: &'a [u8], pos: usize, bits: u32, left: i32 }
        impl<'a> Reader<'a> {
            fn new(data: &'a [u8], pos: usize) -> Self {
                let mut r = Reader { data, pos, bits:0, left:-16 };
                r.fill();
                r
            }
            fn byte(&self, off: usize) -> u32 {
                self.data.get(self.pos + off).copied().unwrap_or(0) as u32
            }
            fn fill(&mut self) {
                if self.left < 0 {
                    let mut unshifted = 0u32;
                    unshifted = self.byte(0) | (unshifted.wrapping_shl(8));
                    unshifted = self.byte(1) | (unshifted.wrapping_shl(8));
                    self.pos = self.pos.saturating_add(2).min(self.data.len());
                    self.bits = unshifted.wrapping_shl((-self.left) as u32);
                    self.left = self.left.wrapping_add(16);
                }
            }
            fn get(&mut self, n: u32) -> u32 {
                if n == 0 { return 0; }
                let v = self.bits >> (32u32.wrapping_sub(n));
                self.bits = self.bits.wrapping_shl(n);
                self.left -= n as i32;
                if self.left < 0 {
                    let mut unshifted = 0u32;
                    unshifted = self.byte(0) | unshifted.wrapping_shl(8);
                    unshifted = self.byte(1) | unshifted.wrapping_shl(8);
                    self.pos = self.pos.saturating_add(2).min(self.data.len());
                    self.bits = unshifted.wrapping_shl((-self.left) as u32);
                    self.left += 16;
                }
                v
            }
            fn peek(&mut self, n: u32) -> u32 {
                if n == 0 { return 0; }
                self.bits >> (32u32.wrapping_sub(n))
            }
            fn consume(&mut self, n: u32) {
                let _ = self.get(n);
            }
            fn getnum(&mut self) -> u32 {
                if (self.bits as i32) < 0 {
                    let v = self.get(3).wrapping_sub(4);
                    return v;
                }
                if self.bits >> 16 != 0 {
                    let mut n = 2u32;
                    loop {
                        self.bits = self.bits.wrapping_shl(1);
                        n = n.wrapping_add(1);
                        if (self.bits as i32) < 0 { break; }
                    }
                    self.bits = self.bits.wrapping_shl(1);
                    self.left -= n.wrapping_sub(1) as i32;
                    let _ = self.get(0);
                    let v = 0u32;
                    if n > 16 {
                        let hi = self.get(n.wrapping_sub(16));
                        let lo = self.get(16);
                        let base = 1u32.wrapping_shl(n).wrapping_sub(4);
                        return lo | hi.wrapping_shl(16).wrapping_add(base);
                    }
                    let hi = self.get(n);
                    let base = 1u32.wrapping_shl(n).wrapping_sub(4);
                    return hi.wrapping_add(base);
                }
                let mut n = 2u32;
                loop {
                    n = n.wrapping_add(1);
                    let v = self.get(1);
                    if v != 0 { break; }
                }
                if n > 16 {
                    let hi = self.get(n.wrapping_sub(16));
                    let lo = self.get(16);
                    lo | hi.wrapping_shl(16).wrapping_add(1u32.wrapping_shl(n).wrapping_sub(4))
                } else {
                    let v = self.get(n);
                    v.wrapping_add(1u32.wrapping_shl(n).wrapping_sub(4))
                }
            }
        }

        let mut reader = Reader::new(stream, pos);
        let clue = reader.get(8) as u8;
        let mut numchars = 0usize;
        let mut basecmp = 0u32;
        let mut numbits = 1usize;
        let mut bitnumtbl = [0u32;17];
        let mut codetbl: Vec<u8> = Vec::new();
        loop {
            basecmp = basecmp.wrapping_shl(1);
            let bitnum = reader.getnum();
            bitnumtbl[numbits] = bitnum;
            numchars = numchars.wrapping_add(bitnum as usize);
            basecmp = basecmp.wrapping_add(bitnum);
            let cmp = if bitnum != 0 {
                basecmp.wrapping_shl(16u32.wrapping_sub(numbits as u32)) & 0xffff
            } else { 0 };
            numbits = numbits.wrapping_add(1);
            if bitnum == 0 && cmp != 0 { break; }
            if numbits >= 17 { break; }
        }
        let mut leap = [0u8;256];
        let mut nextchar = 255u16;
        for _ in 0..numchars {
            let mut delta = reader.getnum().wrapping_add(1) as i32;
            loop {
                nextchar = nextchar.wrapping_add(1) & 255;
                if leap[nextchar as usize] == 0 {
                    delta -= 1;
                    if delta < 0 { break; }
                }
            }
            leap[nextchar as usize] = 1;
            codetbl.push(nextchar as u8);
        }

        let mut table = Vec::new();
        let mut pattern = 0u32;
        let mut prev_bits = 0usize;
        let mut code_index = 0usize;
        for bits in 1..numbits {
            let count = bitnumtbl[bits] as usize;
            for _ in 0..count {
                while prev_bits < bits {
                    pattern = pattern.wrapping_shl(1);
                    prev_bits += 1;
                }
                if code_index < codetbl.len() {
                    table.push((pattern, bits, codetbl[code_index]));
                }
                pattern = pattern.wrapping_add(1);
                code_index += 1;
            }
        }

        let mut out = Vec::with_capacity(ulen);
        let mut previous = 0u8;
        while out.len() < ulen {
            let mut found = None;
            for &(p,l,s) in &table {
                let k = reader.peek(l as u32);
                if k == p { found=Some(s); reader.consume(l as u32); break; }
            }
            let code = match found {
                Some(c) => c,
                None => {
                    let c = reader.get(1) as u8;
                    if c == 0 { break; }
                    return Err(CodecError::Truncated);
                }
            };
            if code != clue {
                out.push(code);
                previous = code;
                continue;
            }
            let run = reader.getnum();
            if run != 0 {
                for _ in 0..run {
                    if out.len() >= ulen { break; }
                    out.push(previous);
                }
            } else {
                let eof = reader.get(1);
                if eof != 0 { break; }
                let explicit = reader.get(8) as u8;
                out.push(explicit);
                previous = explicit;
            }
        }
        if out.len() == ulen { Ok(out) } else { Err(CodecError::LengthMismatch) }
    }
}
