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

/// Length of the `CompressionManager` blob header: 4-byte magic + little-endian `i32` length.
pub const COMPRESSION_HEADER_LEN: usize = 8;

/// Cap pre-allocation capacity on untrusted decoder headers so corrupt lengths do not OOM.
const MAX_INITIAL_CAPACITY: usize = 65536;

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
    if mem.len() < COMPRESSION_HEADER_LEN || get_compression_type(mem) == CompressionType::None {
        return mem.len() as i32;
    }
    i32::from_le_bytes([mem[4], mem[5], mem[6], mem[7]])
}

/// `CompressionManager::compressData` for `RefPack`, `BTree` and `Huff`. Returns the full blob
/// (header + stream). Returns `None` for every other type, and wherever the C++ returns 0.
pub fn compress_data(comp_type: CompressionType, src: &[u8]) -> Option<Vec<u8>> {
    let len = i32::try_from(src.len()).ok()?;
    let (magic, stream) = match comp_type {
        CompressionType::RefPack => (b"EAR\0", refpack::encode(src)),
        CompressionType::BTree => (b"EAB\0", btree::encode(src)),
        CompressionType::Huff => (b"EAH\0", huff::encode(src)),
        _ => return None,
    };
    if stream.is_empty() {
        return None;
    }
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
    match ty {
        CompressionType::None => Err(CodecError::BadHeader),
        CompressionType::NoxLzh
        | CompressionType::Zlib1
        | CompressionType::Zlib2
        | CompressionType::Zlib3
        | CompressionType::Zlib4
        | CompressionType::Zlib5
        | CompressionType::Zlib6
        | CompressionType::Zlib7
        | CompressionType::Zlib8
        | CompressionType::Zlib9 => Err(CodecError::Unsupported),
        CompressionType::RefPack | CompressionType::BTree | CompressionType::Huff => {
            let declared = i32::from_le_bytes([src[4], src[5], src[6], src[7]]);
            if declared < 0 {
                return Err(CodecError::BadHeader);
            }
            let payload = &src[COMPRESSION_HEADER_LEN..];
            let out = match ty {
                CompressionType::RefPack => refpack::decode(payload)?,
                CompressionType::BTree => btree::decode(payload)?,
                CompressionType::Huff => huff::decode(payload)?,
                _ => return Err(CodecError::BadHeader),
            };
            if out.len() != declared as usize {
                Err(CodecError::LengthMismatch)
            } else {
                Ok(out)
            }
        }
    }
}

/// RefPack (`refencode.cpp`, `refdecode.cpp`). `stream` is the codec stream after the
/// manager header, starting with the `10fb` / `90fb` type word.
pub mod refpack {
    use super::{CodecError, MAX_INITIAL_CAPACITY};

    const MAX_BACK: usize = 131071;

    /// `REF_is`.
    pub fn is(stream: &[u8]) -> bool {
        if stream.len() < 2 {
            return false;
        }
        matches!(
            u16::from_be_bytes([stream[0], stream[1]]),
            0x10fb | 0x11fb | 0x90fb | 0x91fb
        )
    }

    /// `REF_size`: the uncompressed length from the stream header.
    pub fn size(stream: &[u8]) -> i32 {
        if stream.len() < 2 {
            return 0;
        }
        let packtype = u16::from_be_bytes([stream[0], stream[1]]);
        let ssize = if (packtype & 0x8000) != 0 { 4 } else { 3 };
        let off = if (packtype & 0x0100) != 0 {
            2 + ssize
        } else {
            2
        };
        if stream.len() < off + ssize {
            return 0;
        }
        if ssize == 4 {
            i32::from_be_bytes([stream[off], stream[off + 1], stream[off + 2], stream[off + 3]])
        } else {
            i32::from_be_bytes([0, stream[off], stream[off + 1], stream[off + 2]])
        }
    }

    #[inline]
    fn ref_hash(src: &[u8], idx: usize) -> usize {
        ((((src[idx] as u32) << 8) | (src[idx + 2] as u32)) ^ ((src[idx + 1] as u32) << 4))
            as usize
    }

    fn flush_long_literals(out: &mut Vec<u8>, src: &[u8], rptr: &mut usize, run: &mut usize) {
        while *run > 3 {
            let tlen = 112usize.min(*run & !3);
            *run -= tlen;
            out.push((0xe0 + (tlen >> 2) - 1) as u8);
            out.extend_from_slice(&src[*rptr..*rptr + tlen]);
            *rptr += tlen;
        }
    }

    /// `REF_encode` with `opts == nullptr` (`maxback = 131072`, `quick = 0`).
    pub fn encode(src: &[u8]) -> Vec<u8> {
        if src.len() > i32::MAX as usize {
            return Vec::new();
        }
        let mut out = Vec::with_capacity(16 + src.len());
        if src.len() > 0x00ff_ffff {
            out.extend_from_slice(&0x90fb_u16.to_be_bytes());
            out.extend_from_slice(&(src.len() as u32).to_be_bytes());
        } else {
            out.extend_from_slice(&0x10fb_u16.to_be_bytes());
            let be = (src.len() as u32).to_be_bytes();
            out.extend_from_slice(&be[1..4]);
        }

        let mut hashtbl = vec![-1i32; 65536];
        let mut link = vec![0i32; 131072];

        let mut len: isize = (src.len() as isize) - 4;
        let mut cptr: usize = 0;
        let mut rptr: usize = 0;
        let mut run: usize = 0;

        while len >= 0 {
            let mut blen: u32 = 2;
            let mut bcost: u32 = 2;
            let mut boffset: u32 = 0;
            let mlen: usize = (len as usize).min(1028);
            let hash = ref_hash(src, cptr);
            let mut hoffset = hashtbl[hash];
            let minhoffset = (cptr as isize - MAX_BACK as isize).max(0) as i32;

            if hoffset >= minhoffset {
                loop {
                    let tptr = hoffset as usize;
                    if src[cptr + blen as usize] == src[tptr + blen as usize] {
                        let mut tlen = 0usize;
                        while tlen < mlen && src[cptr + tlen] == src[tptr + tlen] {
                            tlen += 1;
                        }
                        let tlen_u32 = tlen as u32;
                        if tlen_u32 > blen {
                            let toffset = (cptr - 1 - tptr) as u32;
                            let tcost = if toffset < 1024 && tlen_u32 <= 10 {
                                2
                            } else if toffset < 16384 && tlen_u32 <= 67 {
                                3
                            } else {
                                4
                            };
                            if tlen_u32.wrapping_sub(tcost).wrapping_add(4)
                                > blen.wrapping_sub(bcost).wrapping_add(4)
                            {
                                blen = tlen_u32;
                                bcost = tcost;
                                boffset = toffset;
                                if blen >= 1028 {
                                    break;
                                }
                            }
                        }
                    }
                    hoffset = link[(hoffset as usize) & 131071];
                    if hoffset < minhoffset {
                        break;
                    }
                }
            }

            if bcost >= blen || len < 4 {
                let h = cptr as i32;
                link[cptr & 131071] = hashtbl[hash];
                hashtbl[hash] = h;
                run += 1;
                cptr += 1;
                len -= 1;
            } else {
                flush_long_literals(&mut out, src, &mut rptr, &mut run);
                let run_u32 = run as u32;
                if bcost == 2 {
                    out.push((((boffset >> 8) << 5) + ((blen - 3) << 2) + run_u32) as u8);
                    out.push(boffset as u8);
                } else if bcost == 3 {
                    out.push((0x80 + (blen - 4)) as u8);
                    out.push(((run_u32 << 6) + (boffset >> 8)) as u8);
                    out.push(boffset as u8);
                } else {
                    out.push(
                        (0xc0 + ((boffset >> 16) << 4) + (((blen - 5) >> 8) << 2) + run_u32) as u8,
                    );
                    out.push((boffset >> 8) as u8);
                    out.push(boffset as u8);
                    out.push((blen - 5) as u8);
                }
                if run != 0 {
                    out.extend_from_slice(&src[rptr..rptr + run]);
                    run = 0;
                }
                for _ in 0..blen {
                    let h_idx = ref_hash(src, cptr);
                    let h = cptr as i32;
                    link[cptr & 131071] = hashtbl[h_idx];
                    hashtbl[h_idx] = h;
                    cptr += 1;
                }
                rptr = cptr;
                len -= blen as isize;
            }
        }

        len += 4;
        run += len as usize;
        flush_long_literals(&mut out, src, &mut rptr, &mut run);
        out.push((0xfc + run) as u8);
        if run != 0 {
            out.extend_from_slice(&src[rptr..rptr + run]);
        }
        out
    }

    fn copy_literals(
        stream: &[u8],
        pos: &mut usize,
        out: &mut Vec<u8>,
        ulen: usize,
        count: usize,
    ) -> Result<(), CodecError> {
        if *pos + count > stream.len() {
            return Err(CodecError::Truncated);
        }
        if out.len() + count > ulen {
            return Err(CodecError::LengthMismatch);
        }
        out.extend_from_slice(&stream[*pos..*pos + count]);
        *pos += count;
        Ok(())
    }

    fn copy_backref(
        out: &mut Vec<u8>,
        ulen: usize,
        dist: usize,
        blen: usize,
    ) -> Result<(), CodecError> {
        if dist == 0 || dist > out.len() {
            return Err(CodecError::BadReference);
        }
        if out.len() + blen > ulen {
            return Err(CodecError::LengthMismatch);
        }
        for _ in 0..blen {
            let b = out[out.len() - dist];
            out.push(b);
        }
        Ok(())
    }

    /// `REF_decode`.
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        if !is(stream) {
            return Err(CodecError::BadHeader);
        }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let four = (ty & 0x8000) != 0;
        let ssize = if four { 4 } else { 3 };
        let mut pos = 2usize;
        if (ty & 0x0100) != 0 {
            pos += ssize;
        }
        if pos + ssize > stream.len() {
            return Err(CodecError::Truncated);
        }
        let ulen = if four {
            let raw = i32::from_be_bytes([
                stream[pos],
                stream[pos + 1],
                stream[pos + 2],
                stream[pos + 3],
            ]);
            if raw < 0 {
                return Err(CodecError::BadHeader);
            }
            raw as usize
        } else {
            ((stream[pos] as usize) << 16)
                | ((stream[pos + 1] as usize) << 8)
                | (stream[pos + 2] as usize)
        };
        pos += ssize;

        let mut out = Vec::with_capacity(ulen.min(MAX_INITIAL_CAPACITY));
        loop {
            let first = *stream.get(pos).ok_or(CodecError::Truncated)?;
            pos += 1;
            if (first & 0x80) == 0 {
                let second = *stream.get(pos).ok_or(CodecError::Truncated)?;
                pos += 1;
                let lit = (first & 3) as usize;
                copy_literals(stream, &mut pos, &mut out, ulen, lit)?;
                let dist = (((first & 0x60) as usize) << 3) + (second as usize) + 1;
                let blen = (((first & 0x1c) >> 2) as usize) + 3;
                copy_backref(&mut out, ulen, dist, blen)?;
            } else if (first & 0x40) == 0 {
                let second = *stream.get(pos).ok_or(CodecError::Truncated)?;
                let third = *stream.get(pos + 1).ok_or(CodecError::Truncated)?;
                pos += 2;
                let lit = (second >> 6) as usize;
                copy_literals(stream, &mut pos, &mut out, ulen, lit)?;
                let dist = (((second & 0x3f) as usize) << 8) + (third as usize) + 1;
                let blen = ((first & 0x3f) as usize) + 4;
                copy_backref(&mut out, ulen, dist, blen)?;
            } else if (first & 0x20) == 0 {
                let second = *stream.get(pos).ok_or(CodecError::Truncated)?;
                let third = *stream.get(pos + 1).ok_or(CodecError::Truncated)?;
                let fourth = *stream.get(pos + 2).ok_or(CodecError::Truncated)?;
                pos += 3;
                let lit = (first & 3) as usize;
                copy_literals(stream, &mut pos, &mut out, ulen, lit)?;
                let dist = ((((first & 0x10) >> 4) as usize) << 16)
                    + ((second as usize) << 8)
                    + (third as usize)
                    + 1;
                let blen = ((((first & 0x0c) >> 2) as usize) << 8) + (fourth as usize) + 5;
                copy_backref(&mut out, ulen, dist, blen)?;
            } else {
                let lit_block = (((first & 0x1f) as usize) << 2) + 4;
                if lit_block <= 112 {
                    copy_literals(stream, &mut pos, &mut out, ulen, lit_block)?;
                } else {
                    let lit = (first & 3) as usize;
                    copy_literals(stream, &mut pos, &mut out, ulen, lit)?;
                    return if out.len() == ulen {
                        Ok(out)
                    } else {
                        Err(CodecError::LengthMismatch)
                    };
                }
            }
        }
    }
}

/// BTree (`btreeencode.cpp`, `btreedecode.cpp`). `stream` starts with the `46fb` type word.
pub mod btree {
    use super::{CodecError, MAX_INITIAL_CAPACITY};

    const BTREE_CODES: usize = 256;
    const BTREE_BIGNUM: u32 = 32000;
    const BTREE_SLOPAGE: usize = 16384;

    struct BitWriter {
        out: Vec<u8>,
        packbits: u32,
        workpattern: u32,
        masks: [u32; 17],
    }

    impl BitWriter {
        fn new(cap: usize) -> Self {
            let mut masks = [0u32; 17];
            for i in 1..17 {
                masks[i] = (masks[i - 1] << 1) + 1;
            }
            Self {
                out: Vec::with_capacity(cap),
                packbits: 0,
                workpattern: 0,
                masks,
            }
        }

        fn write_bits(&mut self, bitpattern: u32, len: u32) {
            if len > 16 {
                self.write_bits(bitpattern >> 16, len - 16);
                self.write_bits(bitpattern, 16);
            } else {
                self.packbits = self.packbits.wrapping_add(len);
                self.workpattern = self.workpattern.wrapping_add(
                    (bitpattern & self.masks[len as usize])
                        << (24u32.wrapping_sub(self.packbits)),
                );
                while self.packbits > 7 {
                    self.out.push((self.workpattern >> 16) as u8);
                    self.workpattern = self.workpattern.wrapping_shl(8);
                    self.packbits = self.packbits.wrapping_sub(8);
                }
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
        if stream.len() < 2 {
            return 0;
        }
        let off = if u16::from_be_bytes([stream[0], stream[1]]) == 0x46fb {
            2
        } else {
            5
        };
        if stream.len() < off + 3 {
            return 0;
        }
        ((stream[off] as i32) << 16) | ((stream[off + 1] as i32) << 8) | (stream[off + 2] as i32)
    }

    fn clear_count(tryq: &mut [u8; BTREE_CODES], count: &mut [i16]) {
        for j in 0..BTREE_CODES {
            if tryq[j] != 0 {
                tryq[j] = 1;
                count[j * 256..(j + 1) * 256].fill(0);
            }
        }
    }

    fn adj_count(buf: &[u8], count: &mut [i16]) {
        if buf.len() < 2 {
            return;
        }
        let mut pair = buf[0] as u16;
        for &b in &buf[1..] {
            pair = (pair << 8) | (b as u16);
            let slot = &mut count[pair as usize];
            *slot = slot.wrapping_add(1);
        }
    }

    fn join_nodes(
        src: &[u8],
        dst: &mut Vec<u8>,
        clueq: &[u8; BTREE_CODES],
        right: &[u8; BTREE_CODES],
        join: &[u8; BTREE_CODES],
        clue: u8,
    ) {
        dst.clear();
        let mut s = 0usize;
        let len = src.len();
        while s < len {
            let b = src[s];
            s += 1;
            let cls = clueq[b as usize];
            if cls == 0 {
                dst.push(b);
            } else if cls == 1 {
                if s < len && src[s] == right[b as usize] {
                    dst.push(join[b as usize]);
                    s += 1;
                } else {
                    dst.push(b);
                }
            } else if cls == 3 {
                dst.push(clue);
                dst.push(b);
            } else {
                dst.push(b);
                if s < len {
                    dst.push(src[s]);
                    s += 1;
                }
            }
        }
    }

    fn find_best(
        count: &[i16],
        tryq: &[u8; BTREE_CODES],
        bestn: &mut [u32; BTREE_CODES],
        bestval: &mut [u32; BTREE_CODES],
        ratio: u32,
    ) -> usize {
        let mut bestsize = 1usize;
        let mut threshold = 3u32;
        let mut l = 0u32;
        let mut count_idx = 0usize;
        for i2 in 0..BTREE_CODES {
            if tryq[i2] != 0 {
                for i1 in 0..BTREE_CODES {
                    let c = count[count_idx];
                    count_idx += 1;
                    if c > (threshold as i16) && tryq[i1] != 0 {
                        let val = (c as i32) as u32;
                        let mut i3 = bestsize;
                        while bestval[i3 - 1] < val {
                            bestn[i3] = bestn[i3 - 1];
                            bestval[i3] = bestval[i3 - 1];
                            i3 -= 1;
                        }
                        bestn[i3] = l + (i1 as u32);
                        bestval[i3] = val;
                        if bestsize < 48 {
                            bestsize += 1;
                        }
                        while bestval[bestsize - 1] < (bestval[1] / ratio) {
                            bestsize -= 1;
                        }
                        threshold = if bestsize < 48 {
                            bestval[1] / ratio
                        } else {
                            bestval[bestsize - 1]
                        };
                    }
                }
            } else {
                count_idx += 256;
            }
            l += 256;
        }
        bestsize
    }

    /// `BTREE_encode` with `opts == nullptr`.
    pub fn encode(src: &[u8]) -> Vec<u8> {
        let mut writer = BitWriter::new(src.len() + 32);
        writer.write_bits(0x46fb, 16);
        writer.write_bits(src.len() as u32, 24);

        let work_cap = src.len() * 3 / 2 + BTREE_SLOPAGE;
        let mut active = Vec::with_capacity(work_cap);
        active.extend_from_slice(src);
        let mut other = Vec::with_capacity(work_cap);

        let mut count = vec![0i16; 65536];
        let mut count2 = [0u32; BTREE_CODES];
        for &b in &active {
            count2[b as usize] += 1;
        }

        let mut clueq = [0u8; BTREE_CODES];
        let mut right = [0u8; BTREE_CODES];
        let mut join = [0u8; BTREE_CODES];
        let mut freeq = [1u8; BTREE_CODES];
        let mut tryq = [0u8; BTREE_CODES];
        for i in 0..BTREE_CODES {
            if count2[i] > 3 {
                tryq[i] = 1;
            }
        }
        count2[0] = BTREE_BIGNUM;

        let mut sortptr = [0usize; BTREE_CODES];
        for (i, slot) in sortptr.iter_mut().enumerate() {
            *slot = i;
        }
        let mut swapped = true;
        while swapped {
            swapped = false;
            for i in 1..BTREE_CODES {
                let cur = sortptr[i];
                let prev = sortptr[i - 1];
                if count2[cur] < count2[prev] || (count2[cur] == count2[prev] && cur > prev) {
                    sortptr[i] = prev;
                    sortptr[i - 1] = cur;
                    swapped = true;
                }
            }
        }

        let mut freeptr = 0usize;
        let clue_usize = sortptr[freeptr];
        freeptr += 1;
        let clue = clue_usize as u8;
        freeq[clue_usize] = 0;
        tryq[clue_usize] = 0;
        clueq[clue_usize] = 3;

        if count2[clue_usize] != 0 {
            join_nodes(&active, &mut other, &clueq, &right, &join, clue);
            std::mem::swap(&mut active, &mut other);
        }
        clueq[clue_usize] = 2;

        let mut bestjoin = [0u8; BTREE_CODES];
        let mut bestn = [0u32; BTREE_CODES];
        let mut bestval = [0u32; BTREE_CODES];
        bestval[0] = u32::MAX;

        let mut bt_size = 0usize;
        let mut bt_node = [0u8; BTREE_CODES];
        let mut bt_left = [0u8; BTREE_CODES];
        let mut bt_right = [0u8; BTREE_CODES];

        let mut passes = 256u32;
        let multimax = 32usize;
        let ratio = 2u32;
        let mut domore = passes;

        while domore != 0 {
            clear_count(&mut tryq, &mut count);
            adj_count(&active, &mut count);
            let mut bestsize = find_best(&count, &tryq, &mut bestn, &mut bestval, ratio);

            domore = 0;
            if bestsize > 1 {
                let mut i = 1usize;
                let mut i1 = 1usize;
                let mut inner_more = true;
                while inner_more {
                    let leftnode = ((bestn[i] >> 8) & 255) as usize;
                    let rightnode = (bestn[i] & 255) as usize;
                    if tryq[leftnode] == 1 && tryq[rightnode] == 1 {
                        inner_more = false;
                        while freeptr < BTREE_CODES && freeq[sortptr[freeptr]] == 0 {
                            freeptr += 1;
                        }
                        if freeptr < BTREE_CODES {
                            let joinnode = sortptr[freeptr];
                            let cost = 3 + count2[joinnode];
                            let save = bestval[i];
                            if cost < save {
                                bestjoin[i1] = joinnode as u8;
                                bestn[i1] = bestn[i];
                                i1 += 1;

                                freeq[joinnode] = 0;
                                tryq[joinnode] = 2;
                                clueq[joinnode] = 3;

                                freeq[leftnode] = 0;
                                tryq[leftnode] = 2;
                                clueq[leftnode] = 1;
                                right[leftnode] = rightnode as u8;
                                join[leftnode] = joinnode as u8;

                                freeq[rightnode] = 0;
                                tryq[rightnode] = 2;

                                bt_node[bt_size] = joinnode as u8;
                                bt_left[bt_size] = leftnode as u8;
                                bt_right[bt_size] = rightnode as u8;
                                bt_size += 1;

                                if i1 <= multimax {
                                    inner_more = true;
                                }
                            }
                        }
                    }
                    i += 1;
                    if i >= bestsize {
                        inner_more = false;
                    }
                }

                bestsize = i1;
                if bestsize > 1 {
                    join_nodes(&active, &mut other, &clueq, &right, &join, clue);
                    std::mem::swap(&mut active, &mut other);
                    for j in 1..bestsize {
                        let leftnode = ((bestn[j] >> 8) & 255) as usize;
                        let joinnode = bestjoin[j] as usize;
                        clueq[leftnode] = 0;
                        clueq[joinnode] = 0;
                    }
                    passes -= 1;
                    domore = passes;
                }
            }
        }

        writer.write_bits(clue as u32, 8);
        writer.write_bits(bt_size as u32, 8);
        for i in 0..bt_size {
            writer.write_bits(bt_node[i] as u32, 8);
            writer.write_bits(bt_left[i] as u32, 8);
            writer.write_bits(bt_right[i] as u32, 8);
        }
        for &b in &active {
            writer.write_bits(b as u32, 8);
        }
        writer.write_bits(clue as u32, 8);
        writer.write_bits(0, 8);
        writer.write_bits(0, 7);
        writer.out
    }

    fn validate_tree(
        node: usize,
        cluetbl: &[i8; BTREE_CODES],
        left: &[u8; BTREE_CODES],
        right: &[u8; BTREE_CODES],
        state: &mut [u8; BTREE_CODES],
        expanded_len: &mut [usize; BTREE_CODES],
        ulen: usize,
    ) -> Result<usize, CodecError> {
        match state[node] {
            1 => return Err(CodecError::BadHeader),
            2 => return Ok(expanded_len[node]),
            _ => {}
        }
        if cluetbl[node] > 0 {
            return Err(CodecError::BadHeader);
        }
        if cluetbl[node] == 0 {
            state[node] = 2;
            expanded_len[node] = 1;
            return Ok(1);
        }
        state[node] = 1;
        let l_len = validate_tree(
            left[node] as usize,
            cluetbl,
            left,
            right,
            state,
            expanded_len,
            ulen,
        )?;
        let r_len = validate_tree(
            right[node] as usize,
            cluetbl,
            left,
            right,
            state,
            expanded_len,
            ulen,
        )?;
        let total = l_len.saturating_add(r_len).min(ulen.saturating_add(1));
        expanded_len[node] = total;
        state[node] = 2;
        Ok(total)
    }

    /// `BTREE_decode`.
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        if !is(stream) {
            return Err(CodecError::BadHeader);
        }
        let ty = u16::from_be_bytes([stream[0], stream[1]]);
        let mut pos = 2usize;
        if ty == 0x47fb {
            pos += 3;
        }
        if pos + 3 > stream.len() {
            return Err(CodecError::Truncated);
        }
        let ulen = ((stream[pos] as usize) << 16)
            | ((stream[pos + 1] as usize) << 8)
            | (stream[pos + 2] as usize);
        pos += 3;
        if pos + 2 > stream.len() {
            return Err(CodecError::Truncated);
        }
        let clue = stream[pos] as usize;
        let nodes = stream[pos + 1] as usize;
        pos += 2;
        if pos + nodes * 3 > stream.len() {
            return Err(CodecError::Truncated);
        }

        let mut cluetbl = [0i8; BTREE_CODES];
        let mut left = [0u8; BTREE_CODES];
        let mut right = [0u8; BTREE_CODES];
        cluetbl[clue] = 1;
        for _ in 0..nodes {
            let node = stream[pos] as usize;
            left[node] = stream[pos + 1];
            right[node] = stream[pos + 2];
            cluetbl[node] = -1;
            pos += 3;
        }

        let mut state = [0u8; BTREE_CODES];
        let mut expanded_len = [0usize; BTREE_CODES];
        for n in 0..BTREE_CODES {
            if cluetbl[n] < 0 {
                validate_tree(
                    n,
                    &cluetbl,
                    &left,
                    &right,
                    &mut state,
                    &mut expanded_len,
                    ulen,
                )?;
            }
        }

        let mut out = Vec::with_capacity(ulen.min(MAX_INITIAL_CAPACITY));
        let mut stack = [0u8; BTREE_CODES];
        loop {
            let node = *stream.get(pos).ok_or(CodecError::Truncated)? as usize;
            pos += 1;
            let c = cluetbl[node];
            if c == 0 {
                if out.len() >= ulen {
                    return Err(CodecError::LengthMismatch);
                }
                out.push(node as u8);
            } else if c < 0 {
                if out.len().saturating_add(expanded_len[node]) > ulen {
                    return Err(CodecError::LengthMismatch);
                }
                stack[0] = right[node];
                stack[1] = left[node];
                let mut sp = 2usize;
                while sp > 0 {
                    sp -= 1;
                    let cur = stack[sp] as usize;
                    if cluetbl[cur] == 0 {
                        out.push(cur as u8);
                    } else {
                        if sp + 2 > BTREE_CODES {
                            return Err(CodecError::BadHeader);
                        }
                        stack[sp] = right[cur];
                        stack[sp + 1] = left[cur];
                        sp += 2;
                    }
                }
            } else {
                let next = *stream.get(pos).ok_or(CodecError::Truncated)?;
                pos += 1;
                if next != 0 {
                    if out.len() >= ulen {
                        return Err(CodecError::LengthMismatch);
                    }
                    out.push(next);
                } else {
                    return if out.len() == ulen {
                        Ok(out)
                    } else {
                        Err(CodecError::LengthMismatch)
                    };
                }
            }
        }
    }
}

/// Huffman (`huffencode.cpp`, `huffdecode.cpp`). `stream` starts with the `30fb` type word.
pub mod huff {
    use super::{CodecError, MAX_INITIAL_CAPACITY};

    const HUFF_BIGNUM: u32 = 32000;
    const HUFF_TREESIZE: usize = 520;
    const HUFF_CODES: usize = 256;
    const HUFF_MAXBITS: usize = 16;
    const HUFF_REPTBL: usize = 252;

    struct HuffEncodeContext {
        qleapcode: [u8; HUFF_CODES],
        count: [u32; 768],
        bitnum: [u32; HUFF_MAXBITS + 1],
        repbits: [u32; HUFF_REPTBL],
        repbase: [u32; HUFF_REPTBL],
        tree_left: [u32; HUFF_TREESIZE],
        tree_right: [u32; HUFF_TREESIZE],
        bitsarray: [u32; HUFF_CODES],
        patternarray: [u32; HUFF_CODES],
        masks: [u32; 17],
        packbits: u32,
        workpattern: u32,
        mostbits: u32,
        codes: usize,
        clue: usize,
        clues: u32,
        plen: u32,
        ulen: u32,
        sortptr: [usize; HUFF_CODES],
        out: Vec<u8>,
    }

    impl HuffEncodeContext {
        fn new(src_len: usize) -> Self {
            let mut masks = [0u32; 17];
            for i in 1..17 {
                masks[i] = (masks[i - 1] << 1) + 1;
            }
            let mut repbits = [0u32; HUFF_REPTBL];
            let mut repbase = [0u32; HUFF_REPTBL];
            let mut i = 0usize;
            while i < 4 {
                repbits[i] = 0;
                repbase[i] = 0;
                i += 1;
            }
            while i < 12 {
                repbits[i] = 1;
                repbase[i] = 4;
                i += 1;
            }
            while i < 28 {
                repbits[i] = 2;
                repbase[i] = 12;
                i += 1;
            }
            while i < 60 {
                repbits[i] = 3;
                repbase[i] = 28;
                i += 1;
            }
            while i < 124 {
                repbits[i] = 4;
                repbase[i] = 60;
                i += 1;
            }
            while i < 252 {
                repbits[i] = 5;
                repbase[i] = 124;
                i += 1;
            }
            Self {
                qleapcode: [0; HUFF_CODES],
                count: [0; 768],
                bitnum: [0; HUFF_MAXBITS + 1],
                repbits,
                repbase,
                tree_left: [0; HUFF_TREESIZE],
                tree_right: [0; HUFF_TREESIZE],
                bitsarray: [0; HUFF_CODES],
                patternarray: [0; HUFF_CODES],
                masks,
                packbits: 0,
                workpattern: 0,
                mostbits: 0,
                codes: 0,
                clue: 0,
                clues: 0,
                plen: 0,
                ulen: src_len as u32,
                sortptr: [0; HUFF_CODES],
                out: Vec::with_capacity(src_len + 64),
            }
        }

        fn write_bits(&mut self, bitpattern: u32, len: u32) {
            if len > 16 {
                self.write_bits(bitpattern >> 16, len - 16);
                self.write_bits(bitpattern, 16);
            } else {
                self.packbits = self.packbits.wrapping_add(len);
                self.workpattern = self.workpattern.wrapping_add(
                    (bitpattern & self.masks[len as usize])
                        << (24u32.wrapping_sub(self.packbits)),
                );
                while self.packbits > 7 {
                    self.out.push((self.workpattern >> 16) as u8);
                    self.workpattern = self.workpattern.wrapping_shl(8);
                    self.packbits = self.packbits.wrapping_sub(8);
                    self.plen = self.plen.wrapping_add(1);
                }
            }
        }

        fn write_num(&mut self, num: u32) {
            let (dphuf, dbase) = if (num as usize) < HUFF_REPTBL {
                (self.repbits[num as usize], self.repbase[num as usize])
            } else if num < 508 {
                (6, 252)
            } else if num < 1020 {
                (7, 508)
            } else if num < 2044 {
                (8, 1020)
            } else if num < 4092 {
                (9, 2044)
            } else if num < 8188 {
                (10, 4092)
            } else if num < 16380 {
                (11, 8188)
            } else if num < 32764 {
                (12, 16380)
            } else if num < 65532 {
                (13, 32764)
            } else if num < 131068 {
                (14, 65532)
            } else if num < 262140 {
                (15, 131068)
            } else if num < 524288 {
                (16, 262140)
            } else if num < 1048576 {
                (17, 524288)
            } else {
                (18, 1048576)
            };
            self.write_bits(1, dphuf + 1);
            self.write_bits(num.wrapping_sub(dbase), dphuf + 2);
        }

        fn write_exp(&mut self, code: usize) {
            self.write_bits(self.patternarray[self.clue], self.bitsarray[self.clue]);
            self.write_num(0);
            self.write_bits(code as u32, 9);
        }

        fn write_code(&mut self, code: usize) {
            if code == self.clue {
                self.write_exp(code);
            } else {
                self.write_bits(self.patternarray[code], self.bitsarray[code]);
            }
        }

        fn tree_chase(&mut self, node: usize, bits: u32) {
            if node < HUFF_CODES {
                self.bitsarray[node] = bits;
            } else {
                let l = self.tree_left[node] as usize;
                let r = self.tree_right[node] as usize;
                self.tree_chase(l, bits + 1);
                self.tree_chase(r, bits + 1);
            }
        }

        fn make_tree(&mut self) {
            let mut list_count = [0u32; HUFF_CODES + 2];
            let mut list_ptr = [0u32; HUFF_CODES + 2];

            let mut nodes = HUFF_CODES;
            let mut i1 = 0usize;
            list_count[i1] = 0;
            i1 += 1;
            for i in 0..HUFF_CODES {
                self.bitsarray[i] = 99;
                if self.count[i] != 0 {
                    list_count[i1] = self.count[i];
                    list_ptr[i1] = i as u32;
                    i1 += 1;
                }
            }
            self.codes = i1 - 1;

            if i1 > 2 {
                while i1 > 2 {
                    let mut i = i1;
                    i -= 1;
                    let mut val1 = list_count[i];
                    let mut ptr1 = i;
                    i -= 1;
                    let mut val2 = list_count[i];
                    let mut ptr2 = i;
                    if val1 < val2 {
                        val2 = val1;
                        ptr2 = ptr1;
                        val1 = list_count[i];
                        ptr1 = i;
                    }
                    while i != 0 {
                        i -= 1;
                        while list_count[i] > val1 {
                            i -= 1;
                        }
                        if i != 0 {
                            val1 = list_count[i];
                            ptr1 = i;
                            if val1 <= val2 {
                                val1 = val2;
                                ptr1 = ptr2;
                                val2 = list_count[i];
                                ptr2 = i;
                            }
                        }
                    }
                    self.tree_left[nodes] = list_ptr[ptr1];
                    self.tree_right[nodes] = list_ptr[ptr2];
                    list_count[ptr1] = val1.wrapping_add(val2);
                    list_ptr[ptr1] = nodes as u32;
                    i1 -= 1;
                    list_count[ptr2] = list_count[i1];
                    list_ptr[ptr2] = list_ptr[i1];
                    nodes += 1;
                }
                self.tree_chase(nodes - 1, 0);
            } else {
                let single = list_ptr[self.codes] as usize;
                self.tree_chase(single, 1);
            }
        }

        fn analysis(&mut self, src: &[u8]) {
            let opt = 57u32;
            let chainsaw = 15u32;
            self.count.fill(0);

            let mut pos = 0usize;
            let mut i1 = 256usize;
            while pos < src.len() {
                let mut i = src[pos] as usize;
                pos += 1;
                if i == i1 {
                    let mut i2 = 0usize;
                    let limit = (pos + 30000).min(src.len());
                    while i == i1 && pos < limit {
                        i2 += 1;
                        i = src[pos] as usize;
                        pos += 1;
                    }
                    if i2 < 255 {
                        self.count[512 + i2] += 1;
                    } else {
                        self.count[512] += 1;
                    }
                }
                self.count[i] += 1;
                i1 = i;
            }
            if self.count[512] == 0 {
                self.count[512] += 1;
            }

            self.clues = 0;
            let mut dclues = 0u32;
            let mut dclue = 0usize;
            let mut i3 = 0usize;
            let mut i = 0usize;
            while i < HUFF_CODES {
                let start = i;
                let mut run_zeros = 0u32;
                if self.count[i] < self.count[i3] {
                    i3 = i;
                }
                while i < HUFF_CODES && self.count[i] == 0 {
                    run_zeros += 1;
                    i += 1;
                }
                if run_zeros >= dclues {
                    dclue = start;
                    dclues = run_zeros;
                    if dclues >= self.clues {
                        dclue = self.clue;
                        dclues = self.clues;
                        self.clue = start;
                        self.clues = run_zeros;
                    }
                }
                i += 1;
            }
            let _ = dclue;

            if (opt & 32) != 0 && self.clues == 0 {
                self.clues = 1;
                self.clue = i3;
            }
            if self.clues > 1 {
                self.clues = 1;
            }
            if self.clues != 0 {
                self.count[self.clue] = self.count[512];
            }

            self.make_tree();

            let mut count2 = [0u32; HUFF_CODES];
            for idx in 0..HUFF_CODES {
                count2[idx] = self.count[idx];
                self.count[idx] = 0;
                self.count[256 + idx] = 0;
                self.count[512 + idx] = 0;
            }

            pos = 0;
            i1 = 256;
            while pos < src.len() {
                let mut i = src[pos] as usize;
                pos += 1;
                if i == i1 {
                    let mut i2 = 0usize;
                    let limit = (pos + 30000).min(src.len());
                    while i == i1 && pos < limit {
                        i = src[pos] as usize;
                        pos += 1;
                        i2 += 1;
                    }
                    let mut repn = HUFF_BIGNUM;
                    let ncode = (i2 as u32).wrapping_mul(self.bitsarray[i1]);
                    if self.clues != 0 && count2[self.clue] != 0 {
                        repn = 20;
                        if i2 < HUFF_REPTBL {
                            repn = self.bitsarray[self.clue] + 3 + self.repbits[i2] * 2;
                        }
                    }
                    if ncode <= repn {
                        self.count[i1] += i2 as u32;
                    } else {
                        self.count[self.clue] += 1;
                    }
                }
                self.count[i] += 1;
                i1 = i;
            }

            if (opt & 32) != 0 {
                self.count[self.clue] += 1;
            }

            self.make_tree();

            let mut max_depth = 99u32;
            let mut i2 = 0usize;
            let mut i3 = 0usize;
            while max_depth > chainsaw {
                max_depth = 0;
                for c in 0..HUFF_CODES {
                    if self.count[c] != 0 && self.bitsarray[c] >= max_depth {
                        i3 = i2;
                        i2 = c;
                        max_depth = self.bitsarray[c];
                    }
                }
                if max_depth > chainsaw {
                    let mut graft = 0usize;
                    while graft < HUFF_CODES {
                        if self.count[graft] != 0 && self.bitsarray[graft] < chainsaw {
                            break;
                        }
                        graft += 1;
                    }
                    for c in graft..HUFF_CODES {
                        if self.count[c] != 0
                            && self.bitsarray[c] < chainsaw
                            && self.bitsarray[c] > self.bitsarray[graft]
                        {
                            graft = c;
                        }
                    }
                    let new_len = self.bitsarray[graft] + 1;
                    self.bitsarray[graft] = new_len;
                    self.bitsarray[i2] = new_len;
                    self.bitsarray[i3] -= 1;
                    max_depth = 99;
                }
            }

            self.bitnum.fill(0);
            for c in 0..HUFF_CODES {
                if (self.bitsarray[c] as usize) <= HUFF_MAXBITS {
                    self.bitnum[self.bitsarray[c] as usize] += 1;
                }
            }

            let mut sorted = 0usize;
            let mut most = 0u32;
            for bits in 1..=HUFF_MAXBITS {
                if self.bitnum[bits] != 0 {
                    for c in 0..HUFF_CODES {
                        if self.bitsarray[c] == bits as u32 {
                            self.sortptr[sorted] = c;
                            sorted += 1;
                        }
                    }
                    most = bits as u32;
                }
            }
            self.mostbits = most;

            let mut pattern = 0u32;
            let mut cur_bits = 0u32;
            for idx in 0..self.codes {
                let code = self.sortptr[idx];
                while cur_bits < self.bitsarray[code] {
                    cur_bits += 1;
                    pattern <<= 1;
                }
                self.patternarray[code] = pattern;
                pattern += 1;
            }
        }

        fn pack(&mut self, src: &[u8]) {
            self.write_bits(self.clue as u32, 8);
            for b in 1..=self.mostbits as usize {
                self.write_num(self.bitnum[b]);
            }
            self.qleapcode.fill(0);
            let mut i2 = 255usize;
            for idx in 0..self.codes {
                let target = self.sortptr[idx];
                let mut di = -1i32;
                loop {
                    i2 = (i2 + 1) & 255;
                    if self.qleapcode[i2] == 0 {
                        di += 1;
                    }
                    if i2 == target {
                        break;
                    }
                }
                self.qleapcode[i2] = 1;
                self.write_num(di as u32);
            }

            let mut pos = 0usize;
            let mut i1 = 256usize;
            while pos < src.len() {
                let mut i = src[pos] as usize;
                pos += 1;
                if i == i1 {
                    let mut i2 = 0usize;
                    let limit = (pos + 30000).min(src.len());
                    while i == i1 && pos < limit {
                        i = src[pos] as usize;
                        pos += 1;
                        i2 += 1;
                    }
                    let mut repn = HUFF_BIGNUM;
                    let ncode = (i2 as u32).wrapping_mul(self.bitsarray[i1]);
                    if self.clues != 0 && self.count[self.clue] != 0 {
                        repn = 20;
                        if i2 < HUFF_REPTBL {
                            repn = self.bitsarray[self.clue] + 3 + self.repbits[i2] * 2;
                        }
                    }
                    if ncode <= repn {
                        for _ in 0..i2 {
                            self.write_code(i1);
                        }
                    } else {
                        self.write_bits(self.patternarray[self.clue], self.bitsarray[self.clue]);
                        self.write_num(i2 as u32);
                    }
                }
                i1 = i;
                self.write_code(i);
            }

            self.write_bits(self.patternarray[self.clue], self.bitsarray[self.clue]);
            self.write_num(0);
            self.write_bits(2, 2);
            self.write_bits(0, 7);
        }
    }

    /// `HUFF_is`.
    pub fn is(stream: &[u8]) -> bool {
        if stream.len() < 2 {
            return false;
        }
        matches!(
            u16::from_be_bytes([stream[0], stream[1]]),
            0x30fb
                | 0x31fb
                | 0x32fb
                | 0x33fb
                | 0x34fb
                | 0x35fb
                | 0xb0fb
                | 0xb1fb
                | 0xb2fb
                | 0xb3fb
                | 0xb4fb
                | 0xb5fb
        )
    }

    /// `HUFF_size`.
    pub fn size(stream: &[u8]) -> i32 {
        if stream.len() < 2 {
            return 0;
        }
        let packtype = u16::from_be_bytes([stream[0], stream[1]]);
        let ssize = if (packtype & 0x8000) != 0 { 4 } else { 3 };
        let off = if (packtype & 0x0100) != 0 {
            2 + ssize
        } else {
            2
        };
        if stream.len() < off + ssize {
            return 0;
        }
        if ssize == 4 {
            i32::from_be_bytes([stream[off], stream[off + 1], stream[off + 2], stream[off + 3]])
        } else {
            i32::from_be_bytes([0, stream[off], stream[off + 1], stream[off + 2]])
        }
    }

    /// `HUFF_encode` with `opts == nullptr` (no delta pre-pass).
    pub fn encode(src: &[u8]) -> Vec<u8> {
        if src.len() > i32::MAX as usize {
            return Vec::new();
        }
        let mut ctx = HuffEncodeContext::new(src.len());
        ctx.analysis(src);
        let ulen = ctx.ulen;
        if ulen > 0x00ff_ffff {
            ctx.write_bits(0xb0fb, 16);
            ctx.write_bits(ulen, 32);
        } else {
            ctx.write_bits(0x30fb, 16);
            ctx.write_bits(ulen, 24);
        }
        ctx.pack(src);
        ctx.out
    }

    struct HuffReader<'a> {
        data: &'a [u8],
        pos: usize,
        bits: u32,
        bitsunshifted: u32,
        bitsleft: i32,
    }

    impl<'a> HuffReader<'a> {
        fn new(data: &'a [u8]) -> Result<Self, CodecError> {
            let mut r = Self {
                data,
                pos: 0,
                bits: 0,
                bitsunshifted: 0,
                bitsleft: -16,
            };
            r.get_bits(0)?;
            Ok(r)
        }

        #[inline]
        fn next_byte(&mut self) -> Result<u32, CodecError> {
            if self.pos < self.data.len() {
                let b = self.data[self.pos] as u32;
                self.pos += 1;
                Ok(b)
            } else if self.pos < self.data.len() + 4 {
                self.pos += 1;
                Ok(0)
            } else {
                Err(CodecError::Truncated)
            }
        }

        #[inline]
        fn refill16(&mut self) -> Result<(), CodecError> {
            let b0 = self.next_byte()?;
            let b1 = self.next_byte()?;
            self.bitsunshifted = b0 | self.bitsunshifted.wrapping_shl(8);
            self.bitsunshifted = b1 | self.bitsunshifted.wrapping_shl(8);
            self.bits = self.bitsunshifted.wrapping_shl((-self.bitsleft) as u32);
            self.bitsleft += 16;
            Ok(())
        }

        #[inline]
        fn get_bits(&mut self, n: u32) -> Result<u32, CodecError> {
            let mut v = 0u32;
            if n != 0 {
                v = self.bits >> (32 - n);
                self.bits = self.bits.wrapping_shl(n);
                self.bitsleft -= n as i32;
            }
            if self.bitsleft < 0 {
                self.refill16()?;
            }
            Ok(v)
        }

        fn get_num(&mut self) -> Result<u32, CodecError> {
            if (self.bits as i32) < 0 {
                let v = self.get_bits(3)?;
                return Ok(v.wrapping_sub(4));
            }
            let n: u32 = if (self.bits >> 16) != 0 {
                let mut count = 2u32;
                loop {
                    self.bits = self.bits.wrapping_shl(1);
                    count += 1;
                    if (self.bits as i32) < 0 {
                        break;
                    }
                }
                self.bits = self.bits.wrapping_shl(1);
                self.bitsleft -= (count - 1) as i32;
                self.get_bits(0)?;
                count
            } else {
                let mut count = 2u32;
                loop {
                    count += 1;
                    if count > 31 {
                        return Err(CodecError::Truncated);
                    }
                    if self.get_bits(1)? != 0 {
                        break;
                    }
                }
                count
            };

            if n > 16 {
                let v = self.get_bits(n - 16)?;
                let v1 = self.get_bits(16)?;
                Ok((v1 | v.wrapping_shl(16))
                    .wrapping_add(1u32.wrapping_shl(n))
                    .wrapping_sub(4))
            } else {
                let v = self.get_bits(n)?;
                Ok(v.wrapping_add(1u32.wrapping_shl(n)).wrapping_sub(4))
            }
        }
    }

    /// `HUFF_decode` for plain `30fb`/`31fb`/`b0fb`/`b1fb` streams. Delta (`32fb`, `33fb`) and
    /// accelerated (`34fb`, `35fb`) variants return [`CodecError::Unsupported`].
    pub fn decode(stream: &[u8]) -> Result<Vec<u8>, CodecError> {
        if stream.len() < 5 || !is(stream) {
            return Err(CodecError::BadHeader);
        }
        let raw_type = u16::from_be_bytes([stream[0], stream[1]]);
        let base_type = raw_type & !0x0100;
        if matches!(base_type, 0x32fb | 0xb2fb | 0x34fb | 0xb4fb) {
            return Err(CodecError::Unsupported);
        }
        if !matches!(base_type, 0x30fb | 0xb0fb) {
            return Err(CodecError::BadHeader);
        }

        let mut reader = HuffReader::new(stream)?;
        let header_type = reader.get_bits(16)?;
        let ulen_raw = if (header_type & 0x8000) != 0 {
            if (header_type & 0x0100) != 0 {
                let _ = reader.get_bits(16)?;
                let _ = reader.get_bits(16)?;
            }
            let hi = reader.get_bits(16)?;
            let lo = reader.get_bits(16)?;
            lo | (hi << 16)
        } else {
            if (header_type & 0x0100) != 0 {
                let _ = reader.get_bits(8)?;
                let _ = reader.get_bits(16)?;
            }
            let hi = reader.get_bits(8)?;
            let lo = reader.get_bits(16)?;
            lo | (hi << 16)
        };
        if (ulen_raw as i32) < 0 {
            return Err(CodecError::BadHeader);
        }
        let ulen = ulen_raw as usize;

        let clue = reader.get_bits(8)? as u8;

        let mut bitnumtbl = [0u32; 17];
        let mut deltatbl = [0u32; 17];
        let mut cmptbl = [0u32; 17];
        let mut numchars = 0usize;
        let mut numbits = 1usize;
        let mut basecmp = 0u32;
        loop {
            if numbits > 16 {
                return Err(CodecError::BadHeader);
            }
            basecmp = basecmp.wrapping_shl(1);
            deltatbl[numbits] = basecmp.wrapping_sub(numchars as u32);

            let bitnum = reader.get_num()?;
            if bitnum > 256 || numchars + (bitnum as usize) > 256 {
                return Err(CodecError::BadHeader);
            }
            bitnumtbl[numbits] = bitnum;
            numchars += bitnum as usize;
            basecmp = basecmp.wrapping_add(bitnum);
            if basecmp > (1u32 << numbits) {
                return Err(CodecError::BadHeader);
            }

            let cmp = if bitnum != 0 {
                basecmp.wrapping_shl((16 - numbits) as u32) & 0xffff
            } else {
                0
            };
            cmptbl[numbits] = cmp;
            numbits += 1;
            if bitnum != 0 && cmp == 0 {
                break;
            }
        }
        let mostbits = numbits - 1;
        cmptbl[mostbits] = u32::MAX;

        let mut codetbl = [0u8; 256];
        let mut leap = [0u8; 256];
        let mut nextchar = 255u8;
        for i in 0..numchars {
            let leapdelta = reader.get_num()?;
            if leapdelta >= (256 - i) as u32 {
                return Err(CodecError::BadHeader);
            }
            let mut rem = leapdelta + 1;
            while rem > 0 {
                nextchar = nextchar.wrapping_add(1);
                if leap[nextchar as usize] == 0 {
                    rem -= 1;
                }
            }
            leap[nextchar as usize] = 1;
            codetbl[i] = nextchar;
        }

        let mut quickcodetbl = [0u8; 256];
        let mut quicklentbl = [64u8; 256];
        let mut cluelen = 0u8;
        let mut code_idx = 0usize;
        let mut quick_idx = 0usize;
        for bits in 1..=mostbits.min(8) {
            let count = bitnumtbl[bits] as usize;
            let numbitentries = 1usize << (8 - bits);
            for _ in 0..count {
                let nextcode = codetbl[code_idx];
                code_idx += 1;
                let entry_len = if nextcode == clue {
                    cluelen = bits as u8;
                    96u8
                } else {
                    bits as u8
                };
                if quick_idx + numbitentries > 256 {
                    return Err(CodecError::BadHeader);
                }
                for k in 0..numbitentries {
                    quickcodetbl[quick_idx + k] = nextcode;
                    quicklentbl[quick_idx + k] = entry_len;
                }
                quick_idx += numbitentries;
            }
        }

        let mut out = Vec::with_capacity(ulen.min(MAX_INITIAL_CAPACITY));
        loop {
            let idx = (reader.bits >> 24) as usize;
            let qlen = quicklentbl[idx];
            if qlen <= 8 {
                if out.len() >= ulen {
                    return Err(CodecError::LengthMismatch);
                }
                out.push(quickcodetbl[idx]);
                let _ = reader.get_bits(qlen as u32)?;
                continue;
            }

            let nb = if qlen == 96 {
                if cluelen == 0 || cluelen > 8 {
                    return Err(CodecError::BadHeader);
                }
                cluelen as usize
            } else {
                if mostbits <= 8 {
                    return Err(CodecError::BadHeader);
                }
                let cmp16 = reader.bits >> 16;
                let mut len = 8usize;
                loop {
                    len += 1;
                    if cmp16 < cmptbl[len] {
                        break;
                    }
                    if len >= mostbits {
                        return Err(CodecError::BadHeader);
                    }
                }
                len
            };

            let raw = reader.get_bits(nb as u32)?;
            let code_pos = raw.wrapping_sub(deltatbl[nb]) as usize;
            if code_pos >= numchars {
                return Err(CodecError::BadHeader);
            }
            let code = codetbl[code_pos];
            if code != clue {
                if out.len() >= ulen {
                    return Err(CodecError::LengthMismatch);
                }
                out.push(code);
                continue;
            }

            let runlen = reader.get_num()?;
            if runlen != 0 {
                let prev = *out.last().ok_or(CodecError::BadReference)?;
                let run = runlen as usize;
                if out.len().saturating_add(run) > ulen {
                    return Err(CodecError::LengthMismatch);
                }
                out.resize(out.len() + run, prev);
            } else {
                if reader.get_bits(1)? != 0 {
                    break;
                }
                let explicit = reader.get_bits(8)? as u8;
                if out.len() >= ulen {
                    return Err(CodecError::LengthMismatch);
                }
                out.push(explicit);
            }
        }

        if out.len() == ulen {
            Ok(out)
        } else {
            Err(CodecError::LengthMismatch)
        }
    }
}
