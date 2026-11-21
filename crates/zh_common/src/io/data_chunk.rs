//! `PKT-451`: Tagged hierarchical binary chunk container (`DataChunk.h` / `DataChunk.cpp`
//! and `MapReaderWriterInfo.h`) used by `.map`, `.wak`, and binary map/mission assets.
//!
//! Reference packet: `packets/phase7/PKT-451_DataChunk.md`
//! Golden vectors: `crates/zh_common/tests/golden_pkt451_datachunk.rs` (`docs/golden_vectors/PKT-451.json`)

#![allow(unused_variables, dead_code)]

use crate::core::dict::Dict;
use crate::core::dict::DictDataType;
use crate::core::name_key_generator::NameKeyGenerator;
use crate::io::compression;
use crate::types::{Byte, Int, NameKeyType, Real, UnsignedInt, UnsignedShort};

/// 4-byte TOC header tag written at the start of every `DataChunkOutput` file (`"CkMp"`).
pub const CHUNK_TOC_TAG: [u8; 4] = *b"CkMp";

/// Chunk format version tag (`DataChunkVersionType`).
pub type DataChunkVersionType = UnsignedShort;

/// Bounded error type returned by `DataChunkOutput` / `DataChunkInput` / `CachedFileInputStream`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataChunkError {
    UnexpectedEof,
    InvalidHeader,
    CorruptFormat,
    NotOpen,
    UnknownChunk,
    StringTooLong,
}

/// Chunk metadata passed to registered parsers (`DataChunkInfo`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DataChunkInfo {
    pub label: String,
    pub parent_label: String,
    pub version: DataChunkVersionType,
    pub data_size: Int,
}

/// Abstract readable seekable stream matching C++ `ChunkInputStream` (`MapReaderWriterInfo.h`).
pub trait ChunkInputStream {
    fn read(&mut self, buf: &mut [u8]) -> usize;
    fn tell(&self) -> UnsignedInt;
    fn absolute_seek(&mut self, pos: UnsignedInt) -> bool;
    fn eof(&self) -> bool;
}

/// Abstract writable stream matching C++ `OutputStream` (`MapReaderWriterInfo.h`).
pub trait DataChunkOutputStream {
    fn write(&mut self, data: &[u8]) -> usize;
}

impl DataChunkOutputStream for Vec<u8> {
    fn write(&mut self, data: &[u8]) -> usize {
        self.extend_from_slice(data);
        data.len()
    }
}

fn read_stream_exact(
    input: &mut dyn ChunkInputStream,
    buf: &mut [u8],
) -> Result<usize, DataChunkError> {
    let mut read_total = 0usize;
    while read_total < buf.len() {
        let read = input.read(&mut buf[read_total..]);
        if read == 0 {
            return Err(DataChunkError::UnexpectedEof);
        }
        read_total = read_total.saturating_add(read);
    }
    Ok(read_total)
}

/// In-memory file stream mirroring C++ `CachedFileInputStream`.
///
/// Automatically checks `compression::is_data_compressed` (`EAR\0` etc.) and replaces the buffer
/// with decompressed bytes when `decompress_data` succeeds with the header's uncompressed length;
/// otherwise retains the raw input buffer cleanly without panicking.
#[derive(Debug, Clone, Default)]
pub struct CachedFileInputStream {
    data: Vec<u8>,
    pos: usize,
}

impl CachedFileInputStream {
    /// Creates an empty stream.
    pub fn new() -> Self {
        Self::default()
    }

    /// Loads `raw_file_bytes`, decompressing if it carries a valid EAC header whose decoded length
    /// matches `compression::get_uncompressed_size`.
    pub fn from_bytes(raw_file_bytes: &[u8]) -> Self {
        let data = if compression::is_data_compressed(raw_file_bytes) {
            match compression::decompress_data(raw_file_bytes) {
                Ok(decompressed) => {
                    let expected = compression::get_uncompressed_size(raw_file_bytes);
                    if expected >= 0 && decompressed.len() == expected as usize {
                        decompressed
                    } else {
                        raw_file_bytes.to_vec()
                    }
                }
                Err(_) => raw_file_bytes.to_vec(),
            }
        } else {
            raw_file_bytes.to_vec()
        };
        Self { data, pos: 0 }
    }

    /// Returns the underlying buffer slice currently exposed by this stream.
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    /// Rewinds the read cursor back to byte `0`.
    pub fn rewind(&mut self) {
        self.pos = 0;
    }

    /// Exposes compression helper linkage check without dead-code warnings.
    pub fn is_compressed_buffer(buf: &[u8]) -> bool {
        compression::is_data_compressed(buf)
    }
}

impl ChunkInputStream for CachedFileInputStream {
    fn read(&mut self, buf: &mut [u8]) -> usize {
        if buf.is_empty() {
            return 0;
        }
        let remaining = self.data.len().saturating_sub(self.pos);
        let count = remaining.min(buf.len());
        buf[..count].copy_from_slice(&self.data[self.pos..self.pos + count]);
        self.pos += count;
        count
    }

    fn tell(&self) -> UnsignedInt {
        self.pos as UnsignedInt
    }

    fn absolute_seek(&mut self, pos: UnsignedInt) -> bool {
        let pos = pos as usize;
        self.pos = pos.min(self.data.len());
        true
    }

    fn eof(&self) -> bool {
        self.pos >= self.data.len()
    }
}

#[derive(Debug, Clone)]
struct TocEntry {
    name: String,
    id: UnsignedInt,
}

/// Chunk label <-> integer ID table (`DataChunkTableOfContents`).
///
/// Preserve C++ LIFO ordering when writing TOC entries to disk after `"CkMp"`.
#[derive(Debug, Clone, Default)]
pub struct DataChunkTableOfContents {
    entries: Vec<TocEntry>,
    next_id: UnsignedInt,
    header_opened: bool,
}

impl DataChunkTableOfContents {
    /// Constructs an empty TOC starting ID allocation at `1`.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
            header_opened: false,
        }
    }

    /// Looks up the integer ID assigned to `name`.
    pub fn get_id(&self, name: &str) -> Option<UnsignedInt> {
        self.entries
            .iter()
            .find(|entry| entry.name == name)
            .map(|entry| entry.id)
    }

    /// Looks up the string label assigned to `id`.
    pub fn get_name(&self, id: UnsignedInt) -> Option<&str> {
        self.entries
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.name.as_str())
    }

    /// Allocates a new ID for `name` or returns its existing ID.
    pub fn allocate_id(&mut self, name: &str) -> UnsignedInt {
        if let Some(id) = self.get_id(name) {
            return id;
        }
        let id = self.next_id;
        self.next_id = self.next_id.wrapping_add(1);
        self.entries.push(TocEntry {
            name: name.to_owned(),
            id,
        });
        id
    }

    /// Writes `"CkMp"` header, `i32` count, and LIFO entries (`u8 len`, ASCII bytes, `u32 id` LE).
    pub fn write(&self, out: &mut dyn DataChunkOutputStream) -> Result<(), DataChunkError> {
        out.write(&CHUNK_TOC_TAG);
        let count = i32::try_from(self.entries.len()).map_err(|_| DataChunkError::CorruptFormat)?;
        out.write(&count.to_le_bytes());
        for entry in self.entries.iter().rev() {
            let len = u8::try_from(entry.name.len()).map_err(|_| DataChunkError::StringTooLong)?;
            if !entry.name.is_ascii() {
                return Err(DataChunkError::StringTooLong);
            }
            out.write(&[len]);
            out.write(entry.name.as_bytes());
            out.write(&entry.id.to_le_bytes());
        }
        Ok(())
    }

    /// Reads `"CkMp"` header and populates the TOC from `input`.
    pub fn read(&mut self, input: &mut dyn ChunkInputStream) -> Result<(), DataChunkError> {
        let mut tag = [0u8; 4];
        if read_stream_exact(input, &mut tag)? != 4 {
            return Err(DataChunkError::InvalidHeader);
        }
        if tag != CHUNK_TOC_TAG {
            return Err(DataChunkError::InvalidHeader);
        }
        let mut count_bytes = [0u8; 4];
        if read_stream_exact(input, &mut count_bytes)? != 4 {
            return Err(DataChunkError::InvalidHeader);
        }
        let count = i32::from_le_bytes(count_bytes);
        if count < 0 {
            return Err(DataChunkError::InvalidHeader);
        }
        self.entries.clear();
        self.next_id = 1;
        self.header_opened = false;
        let count = count as usize;
        let mut max_id = 0u32;
        for _ in 0..count {
            let mut len_bytes = [0u8; 1];
            if read_stream_exact(input, &mut len_bytes)? != 1 {
                return Err(DataChunkError::UnexpectedEof);
            }
            let len = len_bytes[0] as usize;
            let mut name_bytes = vec![0u8; len];
            if len != 0 && read_stream_exact(input, &mut name_bytes)? != len {
                return Err(DataChunkError::UnexpectedEof);
            }
            let mut id_bytes = [0u8; 4];
            if read_stream_exact(input, &mut id_bytes)? != 4 {
                return Err(DataChunkError::UnexpectedEof);
            }
            let name = String::from_utf8(name_bytes).map_err(|_| DataChunkError::CorruptFormat)?;
            if !name.is_ascii() {
                return Err(DataChunkError::CorruptFormat);
            }
            let id = u32::from_le_bytes(id_bytes);
            max_id = max_id.max(id);
            self.entries.push(TocEntry { name, id });
        }
        self.next_id = max_id.wrapping_add(1).max(1);
        self.header_opened = count > 0 && !input.eof();
        Ok(())
    }

    /// Returns whether the TOC header was non-empty on read.
    pub fn is_opened_for_read(&self) -> bool {
        self.header_opened
    }
}

#[derive(Debug, Clone)]
struct OutputChunk {
    id: UnsignedInt,
    data_size_offset: usize,
}

/// Hierarchical tagged chunk container writer (`DataChunkOutput`).
pub struct DataChunkOutput<'a, O: DataChunkOutputStream> {
    out: &'a mut O,
    payload: Vec<u8>,
    contents: DataChunkTableOfContents,
    chunk_stack: Vec<OutputChunk>,
}

impl<'a, O: DataChunkOutputStream> DataChunkOutput<'a, O> {
    /// Creates a new `DataChunkOutput` writing into `out` when `finish()` is called.
    pub fn new(out: &'a mut O) -> Self {
        Self {
            out,
            payload: Vec::new(),
            contents: DataChunkTableOfContents::new(),
            chunk_stack: Vec::new(),
        }
    }

    /// Opens a nested data chunk with `name` and `version`.
    pub fn open_data_chunk(
        &mut self,
        name: &str,
        version: DataChunkVersionType,
    ) -> Result<(), DataChunkError> {
        let id = self.contents.allocate_id(name);
        self.payload.extend_from_slice(&id.to_le_bytes());
        self.payload.extend_from_slice(&version.to_le_bytes());
        let data_size_offset = self.payload.len();
        self.payload.extend_from_slice(&0i32.to_le_bytes());
        self.chunk_stack.push(OutputChunk {
            id,
            data_size_offset,
        });
        Ok(())
    }

    /// Closes the active data chunk and back-patches its `i32` payload size.
    pub fn close_data_chunk(&mut self) -> Result<(), DataChunkError> {
        let chunk = self.chunk_stack.pop().ok_or(DataChunkError::NotOpen)?;
        let end = self.payload.len();
        let size = end
            .checked_sub(chunk.data_size_offset)
            .and_then(|n| n.checked_sub(4))
            .ok_or(DataChunkError::CorruptFormat)?;
        let size = i32::try_from(size).map_err(|_| DataChunkError::CorruptFormat)?;
        let offset = chunk.data_size_offset;
        self.payload[offset..offset + 4].copy_from_slice(&size.to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `f32`.
    pub fn write_real(&mut self, value: Real) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        self.payload
            .extend_from_slice(&value.to_bits().to_le_bytes());
        Ok(())
    }

    /// Writes a little-endian `i32`.
    pub fn write_int(&mut self, value: Int) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        self.payload.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }

    /// Writes a signed byte (`i8`).
    pub fn write_byte(&mut self, value: Byte) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        self.payload.push(value as u8);
        Ok(())
    }

    /// Writes a `u16` length followed by raw ASCII bytes.
    pub fn write_ascii_string(&mut self, value: &str) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        if !value.is_ascii() {
            return Err(DataChunkError::CorruptFormat);
        }
        let len = u16::try_from(value.len()).map_err(|_| DataChunkError::StringTooLong)?;
        self.payload.extend_from_slice(&len.to_le_bytes());
        self.payload.extend_from_slice(value.as_bytes());
        Ok(())
    }

    /// Writes a `u16` code-unit count followed by UTF-16LE bytes.
    pub fn write_unicode_string(&mut self, units: &[u16]) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        let len = u16::try_from(units.len()).map_err(|_| DataChunkError::StringTooLong)?;
        self.payload.extend_from_slice(&len.to_le_bytes());
        for unit in units {
            self.payload.extend_from_slice(&unit.to_le_bytes());
        }
        Ok(())
    }

    /// Writes raw bytes into the active chunk.
    pub fn write_array_of_bytes(&mut self, bytes: &[u8]) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        i32::try_from(bytes.len()).map_err(|_| DataChunkError::CorruptFormat)?;
        self.payload.extend_from_slice(bytes);
        Ok(())
    }

    /// Writes a `NameKeyType` encoded as `(toc_id << 8) | DictDataType::AsciiString`.
    pub fn write_name_key(
        &mut self,
        key: NameKeyType,
        name_keys: &NameKeyGenerator,
    ) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        let name = name_keys.key_to_name(key);
        let toc_id = self.contents.allocate_id(name);
        let encoded = ((toc_id as i64) << 8) | DictDataType::AsciiString as i64;
        let encoded = i32::try_from(encoded).map_err(|_| DataChunkError::CorruptFormat)?;
        self.payload.extend_from_slice(&encoded.to_le_bytes());
        Ok(())
    }

    /// Writes a `Dict` (`u16` pair count + typed entries referencing the chunk TOC).
    pub fn write_dict(
        &mut self,
        dict: &Dict,
        name_keys: &NameKeyGenerator,
    ) -> Result<(), DataChunkError> {
        if self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        let pair_count = dict.get_pair_count();
        let pair_count = u16::try_from(pair_count).map_err(|_| DataChunkError::CorruptFormat)?;
        self.payload.extend_from_slice(&pair_count.to_le_bytes());
        for i in 0..pair_count as i32 {
            let key = dict.get_nth_key(i);
            let name = name_keys.key_to_name(key);
            let toc_id = self.contents.allocate_id(name);
            let dtype = dict.get_nth_type(i);
            if dtype == DictDataType::None {
                return Err(DataChunkError::CorruptFormat);
            }
            let encoded = ((toc_id as i64) << 8) | dtype as i64;
            let encoded = i32::try_from(encoded).map_err(|_| DataChunkError::CorruptFormat)?;
            self.payload.extend_from_slice(&encoded.to_le_bytes());
            match dtype {
                DictDataType::Bool => {
                    self.payload
                        .push(if dict.get_nth_bool(i) != 0 { 1 } else { 0 })
                }
                DictDataType::Int => self
                    .payload
                    .extend_from_slice(&dict.get_nth_int(i).to_le_bytes()),
                DictDataType::Real => self
                    .payload
                    .extend_from_slice(&dict.get_nth_real(i).to_bits().to_le_bytes()),
                DictDataType::AsciiString => {
                    let value = dict.get_nth_ascii_string(i);
                    let len =
                        u16::try_from(value.len()).map_err(|_| DataChunkError::StringTooLong)?;
                    if !value.is_ascii() {
                        return Err(DataChunkError::CorruptFormat);
                    }
                    self.payload.extend_from_slice(&len.to_le_bytes());
                    self.payload.extend_from_slice(value.as_bytes());
                }
                DictDataType::UnicodeString => {
                    let value = dict.get_nth_unicode_string(i);
                    let len =
                        u16::try_from(value.len()).map_err(|_| DataChunkError::StringTooLong)?;
                    self.payload.extend_from_slice(&len.to_le_bytes());
                    for unit in value {
                        self.payload.extend_from_slice(&unit.to_le_bytes());
                    }
                }
                DictDataType::None => return Err(DataChunkError::CorruptFormat),
            }
        }
        Ok(())
    }

    /// Flushes the `"CkMp"` TOC header followed by buffered chunk payload to `self.out`.
    pub fn finish(self) -> Result<(), DataChunkError> {
        if !self.chunk_stack.is_empty() {
            return Err(DataChunkError::NotOpen);
        }
        self.contents
            .write(self.out as &mut dyn DataChunkOutputStream)?;
        self.out.write(&self.payload);
        Ok(())
    }
}

/// Callback signature invoked by [`DataChunkInput::parse`] for matching chunk headers.
pub type DataChunkParserFn<Ctx> =
    fn(&mut DataChunkInput<'_, Ctx>, &DataChunkInfo, &mut Ctx) -> Result<bool, DataChunkError>;

#[derive(Clone)]
struct RegisteredParser<Ctx> {
    label: String,
    parent_label: String,
    parser: DataChunkParserFn<Ctx>,
}

#[derive(Debug, Clone)]
struct InputChunk {
    id: UnsignedInt,
    version: DataChunkVersionType,
    chunk_start: UnsignedInt,
    data_size: Int,
    data_left: Int,
}

/// Hierarchical tagged chunk container reader (`DataChunkInput`).
pub struct DataChunkInput<'a, Ctx = ()> {
    stream: &'a mut dyn ChunkInputStream,
    contents: DataChunkTableOfContents,
    filepos_of_first_chunk: UnsignedInt,
    parsers: Vec<RegisteredParser<Ctx>>,
    chunk_stack: Vec<InputChunk>,
}

impl<'a, Ctx> DataChunkInput<'a, Ctx> {
    /// Creates a new `DataChunkInput` bound to `stream`.
    pub fn new(stream: &'a mut dyn ChunkInputStream) -> Self {
        Self {
            stream,
            contents: DataChunkTableOfContents::new(),
            filepos_of_first_chunk: 0,
            parsers: Vec::new(),
            chunk_stack: Vec::new(),
        }
    }

    /// Registers a parser callback for `(label, parent_label)`.
    ///
    /// Preserve C++ LIFO precedence: later registrations for the same `(label, parent_label)`
    /// match first during [`Self::parse`].
    pub fn register_parser(
        &mut self,
        label: &str,
        parent_label: &str,
        parser: DataChunkParserFn<Ctx>,
    ) {
        self.parsers.insert(
            0,
            RegisteredParser {
                label: label.to_owned(),
                parent_label: parent_label.to_owned(),
                parser,
            },
        );
    }

    /// Parses sibling chunks at the current nesting depth using registered parsers.
    pub fn parse(&mut self, ctx: &mut Ctx) -> Result<bool, DataChunkError> {
        if !self.contents.is_opened_for_read() {
            return Ok(false);
        }
        while !self.stream.eof() {
            if let Some(chunk) = self.chunk_stack.last() {
                if chunk.data_left < 10 {
                    if chunk.data_left != 0 {
                        return Err(DataChunkError::CorruptFormat);
                    }
                    break;
                }
            }

            let parent_label = self
                .chunk_stack
                .last()
                .and_then(|chunk| self.contents.get_name(chunk.id))
                .unwrap_or("")
                .to_owned();
            let (label, version) = self.open_data_chunk()?;
            if self.stream.eof() && self.chunk_stack.last().is_some() {
                // A complete 10-byte header with no payload is still a valid empty chunk.
            }

            let mut parser_error = None;
            let data_size = self
                .chunk_stack
                .last()
                .map(|chunk| chunk.data_size)
                .unwrap_or(0);
            for registered in &self.parsers {
                if data_size == 0 {
                    break;
                }
                if registered.label == label && registered.parent_label == parent_label {
                    let info = DataChunkInfo {
                        label: label.clone(),
                        parent_label: parent_label.clone(),
                        version,
                        data_size,
                    };
                    match (registered.parser)(self, &info, ctx) {
                        Ok(true) => {}
                        Ok(false) => return Ok(false),
                        Err(error) => {
                            parser_error = Some(error);
                        }
                    }
                    break;
                }
            }
            if let Some(error) = parser_error {
                return Err(error);
            }
            self.close_data_chunk()?;
        }
        Ok(true)
    }

    /// Reads and validates whether the stream begins with a valid `"CkMp"` TOC.
    pub fn is_valid_file_type(&mut self) -> bool {
        if self.contents.read(self.stream).is_err() {
            self.contents.header_opened = false;
            return false;
        }
        self.filepos_of_first_chunk = self.stream.tell();
        self.contents.is_opened_for_read()
    }

    /// Opens the next chunk header and returns `(label, version)`.
    pub fn open_data_chunk(&mut self) -> Result<(String, DataChunkVersionType), DataChunkError> {
        let mut id_bytes = [0u8; 4];
        read_stream_exact(self.stream, &mut id_bytes).map_err(|_| DataChunkError::UnexpectedEof)?;
        let mut version_bytes = [0u8; 2];
        read_stream_exact(self.stream, &mut version_bytes)
            .map_err(|_| DataChunkError::UnexpectedEof)?;
        let mut size_bytes = [0u8; 4];
        read_stream_exact(self.stream, &mut size_bytes)
            .map_err(|_| DataChunkError::UnexpectedEof)?;

        let id = u32::from_le_bytes(id_bytes);
        let version = u16::from_le_bytes(version_bytes);
        let data_size = i32::from_le_bytes(size_bytes);
        if data_size < 0 {
            return Err(DataChunkError::CorruptFormat);
        }
        let label = self
            .contents
            .get_name(id)
            .ok_or(DataChunkError::UnknownChunk)?
            .to_owned();

        self.decrement_data_left(10);
        let chunk_start = self.stream.tell();
        self.chunk_stack.push(InputChunk {
            id,
            version,
            chunk_start,
            data_size,
            data_left: data_size,
        });
        Ok((label, version))
    }

    /// Closes the current chunk and skips any unread `data_left` bytes.
    pub fn close_data_chunk(&mut self) -> Result<(), DataChunkError> {
        let chunk = self
            .chunk_stack
            .last()
            .cloned()
            .ok_or(DataChunkError::NotOpen)?;
        if chunk.data_left > 0 {
            let remaining = chunk.data_left as u32;
            let current = self.stream.tell();
            let target = current
                .checked_add(remaining)
                .ok_or(DataChunkError::CorruptFormat)?;
            if !self.stream.absolute_seek(target) {
                return Err(DataChunkError::CorruptFormat);
            }
            if self.stream.tell() != target {
                return Err(DataChunkError::UnexpectedEof);
            }
            self.decrement_data_left(chunk.data_left);
        }
        self.chunk_stack.pop();
        Ok(())
    }

    /// Returns `true` when the underlying stream is at EOF.
    pub fn at_end_of_file(&self) -> bool {
        self.stream.eof()
    }

    /// Returns `true` when `data_left <= 0` for the current open chunk (or no chunk is open).
    pub fn at_end_of_chunk(&self) -> bool {
        self.chunk_stack
            .last()
            .is_none_or(|chunk| chunk.data_left <= 0)
    }

    /// Resets the stream cursor back to the first chunk after the TOC.
    pub fn reset(&mut self) -> Result<(), DataChunkError> {
        self.chunk_stack.clear();
        if !self.stream.absolute_seek(self.filepos_of_first_chunk) {
            return Err(DataChunkError::CorruptFormat);
        }
        if self.stream.tell() != self.filepos_of_first_chunk {
            return Err(DataChunkError::UnexpectedEof);
        }
        Ok(())
    }

    fn decrement_data_left(&mut self, size: i32) {
        for chunk in &mut self.chunk_stack {
            chunk.data_left = chunk.data_left.saturating_sub(size);
        }
    }

    fn read_chunk_bytes(&mut self, out: &mut [u8]) -> Result<(), DataChunkError> {
        if out.is_empty() {
            return Ok(());
        }
        let size = i32::try_from(out.len()).map_err(|_| DataChunkError::CorruptFormat)?;
        let chunk = self.chunk_stack.last().ok_or(DataChunkError::NotOpen)?;
        if chunk.data_left < size {
            return Err(DataChunkError::UnexpectedEof);
        }
        read_stream_exact(self.stream, out).map_err(|_| DataChunkError::UnexpectedEof)?;
        self.decrement_data_left(size);
        Ok(())
    }

    /// Returns the label of the innermost open chunk.
    pub fn get_chunk_label(&self) -> Option<&str> {
        self.chunk_stack
            .last()
            .and_then(|c| self.contents.get_name(c.id))
    }

    /// Returns the version of the innermost open chunk.
    pub fn get_chunk_version(&self) -> Option<DataChunkVersionType> {
        self.chunk_stack.last().map(|c| c.version)
    }

    /// Returns the declared byte size of the innermost open chunk.
    pub fn get_chunk_data_size(&self) -> Option<UnsignedInt> {
        self.chunk_stack.last().map(|c| c.data_size as UnsignedInt)
    }

    /// Returns the remaining unread byte count in the innermost open chunk.
    pub fn get_chunk_data_size_left(&self) -> Option<UnsignedInt> {
        self.chunk_stack.last().map(|c| c.data_left as UnsignedInt)
    }

    pub fn read_real(&mut self) -> Result<Real, DataChunkError> {
        let mut bytes = [0u8; 4];
        self.read_chunk_bytes(&mut bytes)?;
        Ok(Real::from_bits(u32::from_le_bytes(bytes)))
    }

    pub fn read_int(&mut self) -> Result<Int, DataChunkError> {
        let mut bytes = [0u8; 4];
        self.read_chunk_bytes(&mut bytes)?;
        Ok(Int::from_le_bytes(bytes))
    }

    pub fn read_byte(&mut self) -> Result<Byte, DataChunkError> {
        let mut bytes = [0u8; 1];
        self.read_chunk_bytes(&mut bytes)?;
        Ok(bytes[0] as i8)
    }

    pub fn read_ascii_string(&mut self) -> Result<String, DataChunkError> {
        let mut len_bytes = [0u8; 2];
        self.read_chunk_bytes(&mut len_bytes)?;
        let len = u16::from_le_bytes(len_bytes) as usize;
        let mut bytes = vec![0u8; len];
        self.read_chunk_bytes(&mut bytes)?;
        let text = String::from_utf8(bytes).map_err(|_| DataChunkError::CorruptFormat)?;
        if !text.is_ascii() {
            return Err(DataChunkError::CorruptFormat);
        }
        Ok(text)
    }

    pub fn read_unicode_string(&mut self) -> Result<Vec<u16>, DataChunkError> {
        let mut len_bytes = [0u8; 2];
        self.read_chunk_bytes(&mut len_bytes)?;
        let len = u16::from_le_bytes(len_bytes) as usize;
        let byte_len = len.checked_mul(2).ok_or(DataChunkError::CorruptFormat)?;
        let mut bytes = vec![0u8; byte_len];
        self.read_chunk_bytes(&mut bytes)?;
        let mut units = Vec::with_capacity(len);
        for pair in bytes.chunks_exact(2) {
            units.push(u16::from_le_bytes([pair[0], pair[1]]));
        }
        Ok(units)
    }

    pub fn read_dict(&mut self, name_keys: &mut NameKeyGenerator) -> Result<Dict, DataChunkError> {
        let mut len_bytes = [0u8; 2];
        self.read_chunk_bytes(&mut len_bytes)?;
        let len = u16::from_le_bytes(len_bytes);
        let mut dict = Dict::with_capacity(len as i32);
        for _ in 0..len {
            let encoded = self.read_int()?;
            let dtype = DictDataType::from_i32(encoded & 0xff);
            if dtype == DictDataType::None {
                return Err(DataChunkError::CorruptFormat);
            }
            let toc_id = (encoded >> 8) as i32 as u32;
            let name = self
                .contents
                .get_name(toc_id)
                .ok_or(DataChunkError::UnknownChunk)?;
            let key = name_keys.name_to_key(name);
            match dtype {
                DictDataType::Bool => {
                    let value = self.read_byte()? != 0;
                    dict.set_bool(
                        key,
                        if value {
                            crate::types::TRUE
                        } else {
                            crate::types::FALSE
                        },
                    );
                }
                DictDataType::Int => dict.set_int(key, self.read_int()?),
                DictDataType::Real => dict.set_real(key, self.read_real()?),
                DictDataType::AsciiString => {
                    dict.set_ascii_string(key, &self.read_ascii_string()?);
                }
                DictDataType::UnicodeString => {
                    dict.set_unicode_string(key, &self.read_unicode_string()?);
                }
                DictDataType::None => return Err(DataChunkError::CorruptFormat),
            }
        }
        Ok(dict)
    }

    pub fn read_array_of_bytes(&mut self, out: &mut [u8]) -> Result<(), DataChunkError> {
        self.read_chunk_bytes(out)
    }

    pub fn read_name_key(
        &mut self,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<NameKeyType, DataChunkError> {
        let encoded = self.read_int()?;
        let dtype = DictDataType::from_i32(encoded & 0xff);
        if dtype != DictDataType::AsciiString {
            return Err(DataChunkError::CorruptFormat);
        }
        let toc_id = (encoded >> 8) as i32 as u32;
        let name = self
            .contents
            .get_name(toc_id)
            .ok_or(DataChunkError::UnknownChunk)?;
        Ok(name_keys.name_to_key(name))
    }
}
