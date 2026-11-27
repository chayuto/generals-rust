//! `PKT-461`: the replay file (`Recorder.cpp`) and the game options string it carries
//! (`GameInfo.cpp`: `ParseAsciiStringToGameInfo` / `GameInfoToAsciiString`).
//!
//! Golden vectors: `docs/golden_vectors/PKT-461.json`
//! (`tests/cpp_harness/gen_pkt461_recorder.cpp`), `crates/zh_common/tests/golden_pkt461_recorder.rs`.
//!
//! The layout is the one the reference build on Apple Silicon reads and writes. Wide
//! characters are 32-bit code points in memory and 16 bits in the file.
//!
//! Not ported: archiving, stats logs, the LAN name truncation of `GameInfoToAsciiString`,
//! and the user-interface side of `RecorderClass`.

use std::collections::VecDeque;

pub const MAX_SLOTS: usize = 8;
pub const REPLAY_CRC_INTERVAL: i32 = 100;
const GENREP: &[u8] = b"GENREP";
const LOGICFRAMES_PER_SECOND: u32 = 30;
const NO_FRAME: u32 = u32::MAX;
const EOF: i32 = -1;

pub const MSG_CLEAR_GAME_DATA: i32 = 27;
pub const MSG_NEW_GAME: i32 = 28;
pub const MSG_BEGIN_NETWORK_MESSAGES: i32 = 1000;
pub const MSG_LOGIC_CRC: i32 = 1095;
pub const MSG_END_NETWORK_MESSAGES: i32 = 1999;

/// `GameMessageArgumentDataType` values.
pub mod arg_type {
    pub const INTEGER: u8 = 0;
    pub const REAL: u8 = 1;
    pub const BOOLEAN: u8 = 2;
    pub const OBJECT_ID: u8 = 3;
    pub const DRAWABLE_ID: u8 = 4;
    pub const TEAM_ID: u8 = 5;
    pub const LOCATION: u8 = 6;
    pub const PIXEL: u8 = 7;
    pub const PIXEL_REGION: u8 = 8;
    pub const TIMESTAMP: u8 = 9;
    pub const WIDE_CHAR: u8 = 10;
    pub const UNKNOWN: u8 = 11;
}

/// Bytes an argument of `kind` occupies in `GameMessageArgumentType`.
fn union_size(kind: u8) -> usize {
    match kind {
        arg_type::BOOLEAN => 1,
        arg_type::LOCATION => 12,
        arg_type::PIXEL => 8,
        arg_type::PIXEL_REGION => 16,
        arg_type::INTEGER..=arg_type::TIMESTAMP => 4,
        arg_type::WIDE_CHAR => 4,
        _ => 0,
    }
}

/// One message argument, held as the C++ union: the kind and its bytes, zero filled.
/// Reading it as another kind reads the same bytes, as `getArgument()->objectID` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReplayArg {
    pub kind: u8,
    pub bytes: [u8; 16],
}

impl ReplayArg {
    pub fn new(kind: u8, data: &[u8]) -> Self {
        let mut bytes = [0u8; 16];
        for (to, from) in bytes.iter_mut().zip(data) {
            *to = *from;
        }
        Self { kind, bytes }
    }

    pub fn integer(v: i32) -> Self {
        Self::new(arg_type::INTEGER, &v.to_le_bytes())
    }

    pub fn real(v: f32) -> Self {
        Self::new(arg_type::REAL, &v.to_bits().to_le_bytes())
    }

    pub fn boolean(v: bool) -> Self {
        Self::new(arg_type::BOOLEAN, &[u8::from(v)])
    }

    pub fn object_id(v: u32) -> Self {
        Self::new(arg_type::OBJECT_ID, &v.to_le_bytes())
    }

    pub fn location(x: f32, y: f32, z: f32) -> Self {
        let mut data = [0u8; 12];
        data[0..4].copy_from_slice(&x.to_bits().to_le_bytes());
        data[4..8].copy_from_slice(&y.to_bits().to_le_bytes());
        data[8..12].copy_from_slice(&z.to_bits().to_le_bytes());
        Self::new(arg_type::LOCATION, &data)
    }

    /// The 32-bit word at `index` of the union.
    pub fn word(&self, index: usize) -> u32 {
        let start = index * 4;
        let mut w = [0u8; 4];
        w.copy_from_slice(&self.bytes[start..start + 4]);
        u32::from_le_bytes(w)
    }

    pub fn as_int(&self) -> i32 {
        self.word(0) as i32
    }

    pub fn as_object_id(&self) -> u32 {
        self.word(0)
    }

    pub fn as_bool(&self) -> bool {
        self.bytes[0] != 0
    }

    pub fn as_real(&self) -> f32 {
        f32::from_bits(self.word(0))
    }

    pub fn as_location(&self) -> [f32; 3] {
        [
            f32::from_bits(self.word(0)),
            f32::from_bits(self.word(1)),
            f32::from_bits(self.word(2)),
        ]
    }

    /// The bytes that belong to this argument's own kind.
    pub fn payload(&self) -> &[u8] {
        &self.bytes[..union_size(self.kind)]
    }
}

/// A recorded `GameMessage`.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayCommand {
    pub msg_type: i32,
    pub player_index: i32,
    pub args: Vec<ReplayArg>,
}

impl ReplayCommand {
    pub fn new(msg_type: i32, player_index: i32) -> Self {
        Self {
            msg_type,
            player_index,
            args: Vec::new(),
        }
    }

    pub fn with(mut self, arg: ReplayArg) -> Self {
        self.args.push(arg);
        self
    }

    /// `getArgumentCount`: an `UnsignedByte`, so it wraps.
    pub fn argument_count(&self) -> u8 {
        self.args.len() as u8
    }

    /// `getArgument`: all zero out of range.
    pub fn arg(&self, index: usize) -> ReplayArg {
        self.args.get(index).copied().unwrap_or_default()
    }

    pub fn is_network_message(&self) -> bool {
        self.msg_type > MSG_BEGIN_NETWORK_MESSAGES && self.msg_type < MSG_END_NETWORK_MESSAGES
    }

    /// Whether `updateRecord` writes this message while a recording is open.
    pub fn is_recorded(&self) -> bool {
        self.is_network_message() || self.msg_type == MSG_CLEAR_GAME_DATA
    }

    /// Whether playback puts this message on the command list.
    pub fn is_delivered(&self) -> bool {
        self.msg_type != MSG_BEGIN_NETWORK_MESSAGES && self.msg_type != MSG_CLEAR_GAME_DATA
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    /// The file does not start with `GENREP`.
    BadMagic,
    /// The options string was rejected.
    BadOptions,
    /// The local slot number is outside `-1..8`.
    BadLocalSlot,
    /// No first frame number could be read.
    NoCommands,
    /// An argument ran past the end of the file (the C++ reads garbage there).
    Truncated,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ReplayHeader {
    pub start_time: i32,
    pub end_time: i32,
    pub frame_count: u32,
    pub desync_game: u8,
    pub quit_early: u8,
    pub player_discons: [u8; MAX_SLOTS],
    /// 2, or 4 for a file whose first name character has a zero upper half.
    pub wide_char_bytes: u8,
    pub replay_name: Vec<u32>,
    /// `SYSTEMTIME`.
    pub system_time: [u8; 16],
    pub version_string: Vec<u32>,
    pub version_time_string: Vec<u32>,
    pub version_number: u32,
    pub exe_crc: u32,
    pub ini_crc: u32,
    pub game_options: Vec<u8>,
    pub local_player_index: i32,
}

/// The four words `startRecording` writes after the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PlaybackParams {
    pub difficulty: i32,
    pub original_game_mode: i32,
    pub rank_points: i32,
    pub max_fps: i32,
}

struct Cursor<'a> {
    data: &'a [u8],
    pos: usize,
    wide_char_bytes: usize,
}

impl Cursor<'_> {
    /// `File::read`: copies what is left into the front of `out`; returns the count.
    fn read(&mut self, out: &mut [u8]) -> usize {
        let left = self.data.get(self.pos..).unwrap_or(&[]);
        let n = left.len().min(out.len());
        out[..n].copy_from_slice(&left[..n]);
        self.pos += n;
        n
    }

    fn u8(&mut self, init: u8) -> u8 {
        let mut b = [init];
        self.read(&mut b);
        b[0]
    }

    fn u32(&mut self, init: u32) -> u32 {
        let mut b = init.to_le_bytes();
        self.read(&mut b);
        u32::from_le_bytes(b)
    }

    /// `readReplayWideChar`.
    fn wide_char(&mut self) -> i32 {
        let mut b = [0u8; 4];
        let width = self.wide_char_bytes;
        if self.read(&mut b[..width]) != width {
            return EOF;
        }
        i32::from_le_bytes(b)
    }

    fn unicode_string(&mut self) -> Vec<u32> {
        let mut out = Vec::new();
        while out.len() < 1023 {
            let c = self.wide_char();
            if c == EOF || c == 0 {
                break;
            }
            out.push(c as u32);
        }
        out
    }

    /// `File::readChar`: a signed char, so byte `0xFF` looks like the end of the file.
    fn read_char(&mut self) -> i32 {
        let mut b = [0u8];
        if self.read(&mut b) == 1 {
            i32::from(b[0] as i8)
        } else {
            EOF
        }
    }

    /// `readAsciiString`.
    fn ascii_string(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        // The first character is stored even when it equals `EOF`.
        let mut last = self.read_char() as u8;
        out.push(last);
        let mut index = 0usize;
        while index < 1024 && last != 0 {
            index += 1;
            let c = self.read_char();
            if c == EOF {
                break;
            }
            last = c as u8;
            out.push(last);
        }
        out.truncate(1023);
        if let Some(end) = out.iter().position(|&b| b == 0) {
            out.truncate(end);
        }
        out
    }
}

/// The header part of `readReplayHeader`, up to the options string and local slot.
/// The options string itself is checked by [`GameOptions::parse`].
fn read_header(c: &mut Cursor<'_>) -> Result<ReplayHeader, ReplayError> {
    let mut h = ReplayHeader::default();
    let mut magic = [0u8; 6];
    c.read(&mut magic);
    if magic != GENREP {
        return Err(ReplayError::BadMagic);
    }
    h.start_time = c.u32(0) as i32;
    h.end_time = c.u32(h.start_time as u32) as i32;
    h.frame_count = c.u32(0);
    h.desync_game = c.u8(0);
    h.quit_early = c.u8(0);
    for d in &mut h.player_discons {
        *d = c.u8(0);
    }
    let mark = c.pos;
    let mut probe = [0u8; 4];
    let got = c.read(&mut probe);
    c.pos = mark;
    c.wide_char_bytes = if got == 4 && u32::from_le_bytes(probe) & 0xFFFF_0000 == 0 {
        4
    } else {
        2
    };
    h.wide_char_bytes = c.wide_char_bytes as u8;
    h.replay_name = c.unicode_string();
    c.read(&mut h.system_time);
    h.version_string = c.unicode_string();
    h.version_time_string = c.unicode_string();
    h.version_number = c.u32(0);
    h.exe_crc = c.u32(0);
    h.ini_crc = c.u32(0);
    h.game_options = c.ascii_string();
    Ok(h)
}

/// `readReplayHeader`: the header, the parsed options string, and the cursor after
/// the local slot number.
fn open_header<'a>(
    data: &'a [u8],
    rules: &dyn ReplayRules,
) -> Result<(Cursor<'a>, ReplayHeader, GameOptions), ReplayError> {
    let mut cursor = Cursor {
        data,
        pos: 0,
        wide_char_bytes: 2,
    };
    let mut header = read_header(&mut cursor)?;
    let mut options =
        GameOptions::parse(&header.game_options, rules).ok_or(ReplayError::BadOptions)?;
    let local = cursor.ascii_string();
    header.local_player_index = atoi(&local);
    if header.local_player_index < -1 || header.local_player_index >= MAX_SLOTS as i32 {
        return Err(ReplayError::BadLocalSlot);
    }
    options.local_slot = header.local_player_index;
    Ok((cursor, header, options))
}

/// `readReplayHeader` without playback, as the replay menu uses it.
pub fn read_replay_header(
    data: &[u8],
    rules: &dyn ReplayRules,
) -> Result<(ReplayHeader, GameOptions), ReplayError> {
    open_header(data, rules).map(|(_, header, options)| (header, options))
}

/// Playback of one replay held in memory: `playbackFile`, `readNextFrame` and
/// `appendNextCommand`.
pub struct ReplayReader<'a> {
    cursor: Cursor<'a>,
    pub header: ReplayHeader,
    pub options: GameOptions,
    pub params: PlaybackParams,
    next_frame: u32,
    stopped: bool,
    error: Option<ReplayError>,
}

impl<'a> ReplayReader<'a> {
    pub fn open(data: &'a [u8], rules: &dyn ReplayRules) -> Result<Self, ReplayError> {
        let (mut cursor, header, options) = open_header(data, rules)?;
        let params = PlaybackParams {
            difficulty: cursor.u32(0) as i32,
            original_game_mode: cursor.u32(0) as i32,
            rank_points: cursor.u32(0) as i32,
            max_fps: cursor.u32(0) as i32,
        };
        let mut reader = Self {
            cursor,
            header,
            options,
            params,
            next_frame: 0,
            stopped: false,
            error: None,
        };
        reader.read_next_frame();
        if reader.stopped {
            return Err(ReplayError::NoCommands);
        }
        Ok(reader)
    }

    /// The `MSG_NEW_GAME` playback appends before the first recorded command.
    pub fn new_game_command(&self) -> ReplayCommand {
        const GAME_REPLAY: i32 = 3;
        let mut msg = ReplayCommand::new(MSG_NEW_GAME, 0)
            .with(ReplayArg::integer(GAME_REPLAY))
            .with(ReplayArg::integer(self.params.difficulty))
            .with(ReplayArg::integer(self.params.rank_points));
        if self.params.max_fps != 0 {
            msg = msg.with(ReplayArg::integer(self.params.max_fps));
        }
        msg
    }

    /// The frame of the next recorded command; `None` once playback has stopped.
    pub fn next_frame(&self) -> Option<u32> {
        (!self.stopped).then_some(self.next_frame)
    }

    pub fn error(&self) -> Option<&ReplayError> {
        self.error.as_ref()
    }

    /// Bytes not yet consumed.
    pub fn bytes_left(&self) -> usize {
        self.cursor.data.len() - self.cursor.pos.min(self.cursor.data.len())
    }

    fn stop(&mut self) {
        self.stopped = true;
        self.next_frame = NO_FRAME;
    }

    fn read_next_frame(&mut self) {
        let current = self.next_frame;
        let mut b = current.to_le_bytes();
        if self.cursor.read(&mut b) != 4 {
            self.stop();
            return;
        }
        self.next_frame = u32::from_le_bytes(b);
        let limit = current.wrapping_add(3600 * LOGICFRAMES_PER_SECOND);
        if !(self.next_frame >= current && self.next_frame < limit) {
            self.stop();
        }
    }

    /// `appendNextCommand`; `None` when no message type can be read.
    fn read_command(&mut self) -> Option<ReplayCommand> {
        let c = &mut self.cursor;
        let mut b = [0u8; 4];
        if c.read(&mut b) != 4 {
            return None;
        }
        let msg_type = i32::from_le_bytes(b);
        let mut msg = ReplayCommand::new(msg_type, c.u32(u32::MAX) as i32);
        let count = c.u8(0);
        let mut groups = Vec::new();
        let mut total = 0usize;
        for _ in 0..count {
            let kind = c.u8(arg_type::UNKNOWN);
            let n = c.u8(0);
            groups.push((kind, i32::from(n)));
            total += usize::from(n);
        }
        let mut group = 0usize;
        let (mut kind, mut left) = groups.first().copied().unwrap_or((arg_type::UNKNOWN, 0));
        for _ in 0..total {
            if kind == arg_type::WIDE_CHAR {
                let ch = c.wide_char() as u32;
                msg.args
                    .push(ReplayArg::new(arg_type::WIDE_CHAR, &ch.to_le_bytes()));
            } else if kind < arg_type::UNKNOWN {
                let mut data = [0u8; 16];
                let size = union_size(kind);
                if c.read(&mut data[..size]) != size {
                    self.error = Some(ReplayError::Truncated);
                    return None;
                }
                msg.args.push(ReplayArg { kind, bytes: data });
            }
            // A descriptor with a count of zero is never left: the count goes negative.
            left -= 1;
            if left == 0 {
                group += 1;
                if let Some(&(k, n)) = groups.get(group) {
                    kind = k;
                    left = n;
                }
            }
        }
        Some(msg)
    }

    /// The loop of `updatePlayback`: every command recorded for `frame`, including
    /// those playback drops (see [`ReplayCommand::is_delivered`]).
    pub fn commands_for_frame(&mut self, frame: u32) -> Vec<ReplayCommand> {
        let mut out = Vec::new();
        while !self.stopped && self.next_frame == frame {
            if let Some(msg) = self.read_command() {
                out.push(msg);
            }
            if self.error.is_some() {
                self.stop();
                break;
            }
            self.read_next_frame();
        }
        out
    }

    /// Every remaining command with the frame it was recorded on.
    pub fn read_all(&mut self) -> Vec<(u32, ReplayCommand)> {
        let mut out = Vec::new();
        while let Some(frame) = self.next_frame() {
            for msg in self.commands_for_frame(frame) {
                out.push((frame, msg));
            }
        }
        out
    }
}

/// `cullBadCommands`: removes live network commands while a replay plays, and reports
/// whether the list holds `MSG_CLEAR_GAME_DATA` or `MSG_NEW_GAME`.
pub fn cull_bad_commands(list: &mut Vec<ReplayCommand>) -> (bool, bool) {
    list.retain(|m| !m.is_network_message());
    (
        list.iter().any(|m| m.msg_type == MSG_CLEAR_GAME_DATA),
        list.iter().any(|m| m.msg_type == MSG_NEW_GAME),
    )
}

/// `startRecording`, `writeToFile` and the header patches, into memory.
#[derive(Debug, Clone, Default)]
pub struct ReplayWriter {
    pub bytes: Vec<u8>,
}

const END_TIME_OFFSET: usize = 10;
const FRAME_COUNT_OFFSET: usize = 14;
const DESYNC_OFFSET: usize = 18;
const QUIT_EARLY_OFFSET: usize = 19;
const DISCON_OFFSET: usize = 20;

impl ReplayWriter {
    pub fn new(header: &ReplayHeader, params: &PlaybackParams) -> Self {
        let mut w = Self::default();
        w.bytes.extend_from_slice(GENREP);
        w.put(header.start_time as u32);
        w.put(header.end_time as u32);
        w.put(header.frame_count);
        w.bytes.push(header.desync_game);
        w.bytes.push(header.quit_early);
        w.bytes.extend_from_slice(&header.player_discons);
        w.unicode_string(&header.replay_name);
        w.bytes.extend_from_slice(&header.system_time);
        w.unicode_string(&header.version_string);
        w.unicode_string(&header.version_time_string);
        w.put(header.version_number);
        w.put(header.exe_crc);
        w.put(header.ini_crc);
        w.bytes.extend_from_slice(&header.game_options);
        w.bytes.push(0);
        w.bytes
            .extend_from_slice(header.local_player_index.to_string().as_bytes());
        w.bytes.push(0);
        w.put(params.difficulty as u32);
        w.put(params.original_game_mode as u32);
        w.put(params.rank_points as u32);
        w.put(params.max_fps as u32);
        w
    }

    fn put(&mut self, v: u32) {
        self.bytes.extend_from_slice(&v.to_le_bytes());
    }

    fn wide_char(&mut self, c: u32) {
        self.bytes.extend_from_slice(&(c as u16).to_le_bytes());
    }

    fn unicode_string(&mut self, s: &[u32]) {
        for &c in s.iter().take_while(|&&c| c != 0) {
            self.wide_char(c);
        }
        self.wide_char(0);
    }

    /// `writeToFile`.
    pub fn write_command(&mut self, frame: u32, msg: &ReplayCommand) {
        self.put(frame);
        self.put(msg.msg_type as u32);
        self.put(msg.player_index as u32);
        let args = &msg.args[..usize::from(msg.argument_count())];
        // `GameMessageParser`: runs of equal types.
        let mut runs: Vec<(u8, i32)> = Vec::new();
        let mut last = arg_type::UNKNOWN;
        let mut n = 0i32;
        for a in args {
            if a.kind != last {
                if n > 0 {
                    runs.push((last, n));
                }
                last = a.kind;
                n = 0;
            }
            n += 1;
        }
        if n > 0 {
            runs.push((last, n));
        }
        self.bytes.push(runs.len() as u8);
        for (kind, count) in runs {
            self.bytes.push(kind);
            self.bytes.push(count as u8);
        }
        for a in args {
            if a.kind == arg_type::WIDE_CHAR {
                self.wide_char(a.word(0));
            } else if a.kind < arg_type::UNKNOWN {
                self.bytes.extend_from_slice(a.payload());
            }
        }
    }

    fn patch(&mut self, offset: usize, data: &[u8]) {
        self.bytes[offset..offset + data.len()].copy_from_slice(data);
    }

    /// `logGameStart`'s start time.
    pub fn set_start_time(&mut self, time: i32) {
        self.patch(GENREP.len(), &time.to_le_bytes());
    }

    /// `logGameEnd`.
    pub fn set_end(&mut self, end_time: i32, frame_count: u32) {
        self.patch(END_TIME_OFFSET, &end_time.to_le_bytes());
        self.patch(FRAME_COUNT_OFFSET, &frame_count.to_le_bytes());
    }

    /// `logCRCMismatch`.
    pub fn set_desync(&mut self) {
        self.patch(DESYNC_OFFSET, &[1]);
    }

    pub fn set_quit_early(&mut self) {
        self.patch(QUIT_EARLY_OFFSET, &[1]);
    }

    /// `logPlayerDisconnect`.
    pub fn set_player_disconnected(&mut self, slot: usize) {
        if slot < MAX_SLOTS {
            self.patch(DISCON_OFFSET + slot, &[1]);
        }
    }
}

/// A recorded checksum that differs from the one playback computed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CrcMismatch {
    pub frame: u32,
    pub in_game: u32,
    pub replay: u32,
}

/// `RecorderClass::CRCInfo` and the comparison in `handleCRCMessage`.
#[derive(Debug, Clone, Default)]
pub struct CrcInfo {
    queue: VecDeque<u32>,
    pub local_player: i32,
    pub saw_mismatch: bool,
}

impl CrcInfo {
    pub fn new(local_player: i32) -> Self {
        Self {
            local_player,
            ..Self::default()
        }
    }

    /// A checksum computed by this playback (`MSG_LOGIC_CRC_PLAYBACK`).
    pub fn add_playback_crc(&mut self, crc: u32) {
        self.queue.push_back(crc);
    }

    /// A recorded checksum (`MSG_LOGIC_CRC`). `is_local` is the host's answer to
    /// whether the sender is the local player; `frame` is the current logic frame.
    pub fn check_recorded_crc(
        &mut self,
        recorded: u32,
        is_local: bool,
        frame: u32,
    ) -> Option<CrcMismatch> {
        if !is_local {
            return None;
        }
        let computed = self.queue.pop_front().unwrap_or(0);
        let queued = self.queue.len() as u32;
        let mismatch_frame = if frame > queued {
            frame.wrapping_sub(queued).wrapping_sub(1)
        } else {
            0
        };
        (mismatch_frame > 0 && recorded != computed && !self.saw_mismatch).then_some(CrcMismatch {
            frame: mismatch_frame,
            in_game: computed,
            replay: recorded,
        })
    }
}

// ---------------------------------------------------------------------------------------
// The game options string

/// `SlotState`.
pub mod slot_state {
    pub const OPEN: i32 = 0;
    pub const CLOSED: i32 = 1;
    pub const EASY_AI: i32 = 2;
    pub const MED_AI: i32 = 3;
    pub const BRUTAL_AI: i32 = 4;
    pub const PLAYER: i32 = 5;
}

const PLAYERTEMPLATE_MIN: i32 = -2;
const FIREWALL_TYPE_SIMPLE: i32 = 1;
const FIREWALL_MAX: i32 = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaySlot {
    pub state: i32,
    pub accepted: bool,
    pub has_map: bool,
    pub color: i32,
    pub start_pos: i32,
    pub player_template: i32,
    pub team_number: i32,
    pub nat_behavior: i32,
    /// Human slots only; the C++ gives the others localized labels.
    pub name: Vec<u32>,
    pub ip: u32,
    pub port: u16,
}

impl Default for ReplaySlot {
    fn default() -> Self {
        Self {
            state: slot_state::CLOSED,
            accepted: false,
            has_map: true,
            color: -1,
            start_pos: -1,
            player_template: -1,
            team_number: -1,
            nat_behavior: FIREWALL_TYPE_SIMPLE,
            name: Vec::new(),
            ip: 0,
            port: 0,
        }
    }
}

impl ReplaySlot {
    pub fn is_human(&self) -> bool {
        self.state == slot_state::PLAYER
    }

    pub fn is_ai(&self) -> bool {
        (slot_state::EASY_AI..=slot_state::BRUTAL_AI).contains(&self.state)
    }

    /// `GameSlot::setState`. A second `S=` list reaches slots already filled.
    fn set_state(&mut self, state: i32, name: Vec<u32>, ip: u32) {
        let stays_ai =
            self.is_ai() && (slot_state::EASY_AI..=slot_state::BRUTAL_AI).contains(&state);
        if !stays_ai {
            self.color = -1;
            self.start_pos = -1;
            self.player_template = -1;
            self.team_number = -1;
        }
        if state == slot_state::PLAYER {
            *self = Self {
                state,
                name,
                ..Self::default()
            };
        } else {
            self.state = state;
            self.accepted = true;
            self.has_map = true;
        }
        self.ip = ip;
    }
}

/// The game data the options string is checked against.
pub trait ReplayRules {
    /// `MultiplayerSettings::getNumColors`.
    fn num_colors(&self) -> i32;
    /// `PlayerTemplateStore::getPlayerTemplateCount`.
    fn player_template_count(&self) -> i32;
    /// `GlobalData::m_defaultStartingCash`.
    fn default_starting_cash(&self) -> u32;
    /// `GameState::portableMapPathToRealMapPath`; `None` for an empty result.
    fn real_map_path(&self, portable: &[u8]) -> Option<Vec<u8>>;
    /// `GameState::realMapPathToPortableMapPath` (lower case, `\` separators).
    fn portable_map_path(&self, real: &[u8]) -> Vec<u8>;
    /// The `MapCache` search by CRC and size when `real` is not a known map.
    fn find_map_by_crc(&self, _real: &[u8], _crc: u32, _size: u32) -> Option<Vec<u8>> {
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GameOptions {
    pub slots: [ReplaySlot; MAX_SLOTS],
    pub map: Vec<u8>,
    pub map_crc: u32,
    pub map_size: u32,
    pub map_contents_mask: i32,
    pub seed: i32,
    pub crc_interval: i32,
    pub use_stats: i32,
    pub superweapon_restriction: u16,
    pub starting_cash: u32,
    pub old_factions_only: bool,
    /// Not part of the string; set from the replay header.
    pub local_slot: i32,
}

/// `strtok_r`: skips leading delimiters, cuts at the next one.
fn next_token<'a>(rest: &mut &'a [u8], delims: &[u8]) -> Option<&'a [u8]> {
    let start = rest.iter().position(|b| !delims.contains(b));
    let Some(start) = start else {
        *rest = &[];
        return None;
    };
    let s: &'a [u8] = &rest[start..];
    match s.iter().position(|b| delims.contains(b)) {
        Some(end) => {
            *rest = &s[end + 1..];
            Some(&s[..end])
        }
        None => {
            *rest = &[];
            Some(s)
        }
    }
}

/// `AsciiString::nextToken`: unlike `strtok`, the delimiter stays in `rest`.
fn ascii_next_token(rest: &mut Vec<u8>, seps: &[u8]) -> Vec<u8> {
    let start = rest
        .iter()
        .position(|b| !seps.contains(b))
        .unwrap_or(rest.len());
    let end = rest[start..]
        .iter()
        .position(|b| seps.contains(b))
        .map_or(rest.len(), |n| start + n);
    if end > start {
        let token = rest[start..end].to_vec();
        rest.drain(..end);
        token
    } else {
        rest.clear();
        Vec::new()
    }
}

fn is_c_space(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0b | 0x0c)
}

/// The `strtol` family on a 64-bit `long`: sign, saturating magnitude. `None` when
/// there is no digit.
fn scan_number(s: &[u8], radix: u32) -> Option<(bool, u64, bool)> {
    let mut i = s.iter().position(|&b| !is_c_space(b)).unwrap_or(s.len());
    let negative = s.get(i) == Some(&b'-');
    if matches!(s.get(i), Some(b'-' | b'+')) {
        i += 1;
    }
    let digit = |i: usize| s.get(i).and_then(|&b| char::from(b).to_digit(radix));
    if radix == 16
        && s.get(i) == Some(&b'0')
        && matches!(s.get(i + 1), Some(b'x' | b'X'))
        && digit(i + 2).is_some()
    {
        i += 2;
    }
    digit(i)?;
    let mut value = 0u64;
    let mut overflow = false;
    while let Some(d) = digit(i) {
        match value
            .checked_mul(u64::from(radix))
            .and_then(|v| v.checked_add(u64::from(d)))
        {
            Some(v) => value = v,
            None => overflow = true,
        }
        i += 1;
    }
    Some((negative, value, overflow))
}

/// `strtol` truncated to `int`, which is `atoi`.
fn strtol(s: &[u8], radix: u32) -> i32 {
    match scan_number(s, radix) {
        None => 0,
        Some((true, v, over)) => {
            let limit = i64::MIN.unsigned_abs();
            if over || v >= limit {
                i64::MIN as i32
            } else {
                (v as i64).wrapping_neg() as i32
            }
        }
        Some((false, v, over)) => {
            if over || v > i64::MAX as u64 {
                i64::MAX as i32
            } else {
                v as i32
            }
        }
    }
}

fn atoi(s: &[u8]) -> i32 {
    strtol(s, 10)
}

/// `strtoul` (and `sscanf` with `%x`) truncated to `unsigned int`.
fn strtoul(s: &[u8], radix: u32) -> Option<u32> {
    scan_number(s, radix).map(|(negative, v, over)| {
        if over {
            u32::MAX
        } else if negative {
            v.wrapping_neg() as u32
        } else {
            v as u32
        }
    })
}

/// `MultiByteToWideCharSingleLine`.
fn wide_from_bytes(s: &[u8]) -> Vec<u32> {
    let mut out: Vec<u32> = match std::str::from_utf8(s) {
        Ok(text) => text.chars().map(u32::from).collect(),
        Err(_) => s.iter().map(|&b| u32::from(b)).collect(),
    };
    for c in &mut out {
        if *c == u32::from(b'\n') || *c == u32::from(b'\r') {
            *c = u32::from(b' ');
        }
    }
    out
}

/// `WideCharStringToMultiByte`.
fn bytes_from_wide(s: &[u32]) -> Vec<u8> {
    let mut out = Vec::new();
    for &c in s {
        let c = char::from_u32(c).unwrap_or(char::REPLACEMENT_CHARACTER);
        let mut b = [0u8; 4];
        out.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
    }
    out
}

fn percent_decode(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < s.len() {
        if s[i] == b'%' && i + 2 < s.len() {
            if let Some((negative, v, _)) = scan_number(&s[i + 1..i + 3], 16) {
                if (!negative || v == 0) && v <= 255 {
                    // Concatenating a zero character appends nothing.
                    if v != 0 {
                        out.push(v as u8);
                    }
                    i += 3;
                    continue;
                }
            }
        }
        out.push(s[i]);
        i += 1;
    }
    out
}

fn percent_encode(s: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for &b in s {
        match b {
            b'%' => out.extend_from_slice(b"%25"),
            b'[' => out.extend_from_slice(b"%5B"),
            b']' => out.extend_from_slice(b"%5D"),
            b';' => out.extend_from_slice(b"%3B"),
            b'=' => out.extend_from_slice(b"%3D"),
            _ => out.push(b),
        }
    }
    out
}

/// The `M=` value: the portable directory, turned into the real path of its map file.
fn resolve_map(value: &[u8], rules: &dyn ReplayRules) -> Option<Vec<u8>> {
    const SEPS: &[u8] = b"\\/";
    let portable = percent_decode(&value[2..]);
    let mut rest = portable.clone();
    let mut token = ascii_next_token(&mut rest, SEPS);
    let mut legacy = Vec::new();
    while !rest.is_empty() {
        legacy.extend_from_slice(&token);
        legacy.push(b'\\');
        token = ascii_next_token(&mut rest, SEPS);
    }
    legacy.extend_from_slice(&token);
    legacy.push(b'\\');
    legacy.extend_from_slice(&token);
    legacy.extend_from_slice(b".map");
    rules.real_map_path(&legacy).or_else(|| {
        let mut flat = portable;
        flat.extend_from_slice(b".map");
        rules.real_map_path(&flat)
    })
}

/// The `S=` value. `false` marks the whole string as bad.
fn parse_slots(val: &[u8], slots: &mut [ReplaySlot; MAX_SLOTS], rules: &dyn ReplayRules) -> bool {
    let mut ok = true;
    let mut rest = val;
    for slot in slots.iter_mut() {
        let raw = next_token(&mut rest, b":").unwrap_or(&[]);
        ok &= match raw.first() {
            Some(b'H') => parse_human(raw, slot, rules).is_some(),
            Some(b'C') => parse_ai(raw, slot, rules).is_some(),
            Some(b'O') => {
                slot.set_state(slot_state::OPEN, Vec::new(), 0);
                true
            }
            Some(b'X') => {
                slot.set_state(slot_state::CLOSED, Vec::new(), 0);
                true
            }
            _ => false,
        };
    }
    ok
}

fn bounded(s: &[u8], low: i32, high: i32) -> Option<i32> {
    let v = atoi(s);
    (v >= low && v <= high).then_some(v)
}

fn parse_human(raw: &[u8], slot: &mut ReplaySlot, rules: &dyn ReplayRules) -> Option<()> {
    let mut rest = raw;
    let mut field = || next_token(&mut rest, b",");
    let name = wide_from_bytes(&field()?[1..]);
    let ip = strtoul(field()?, 16).unwrap_or(0);
    slot.set_state(slot_state::PLAYER, name, ip);
    slot.port = strtol(field()?, 10) as u16;
    let flags = field()?;
    if flags.len() != 2 {
        return None;
    }
    slot.accepted = flags[0] == b'T';
    slot.has_map = flags[1] == b'T';
    slot.color = bounded(field()?, -1, rules.num_colors() - 1)?;
    slot.player_template = bounded(
        field()?,
        PLAYERTEMPLATE_MIN,
        rules.player_template_count() - 1,
    )?;
    slot.start_pos = bounded(field()?, -1, MAX_SLOTS as i32 - 1)?;
    slot.team_number = bounded(field()?, -1, MAX_SLOTS as i32 / 2 - 1)?;
    slot.nat_behavior = bounded(field()?, 0, FIREWALL_MAX)?;
    Some(())
}

fn parse_ai(raw: &[u8], slot: &mut ReplaySlot, rules: &dyn ReplayRules) -> Option<()> {
    let mut rest = raw;
    let mut field = || next_token(&mut rest, b",");
    let state = match field()?.get(1) {
        Some(b'E') => Some(slot_state::EASY_AI),
        Some(b'M') => Some(slot_state::MED_AI),
        Some(b'H') => Some(slot_state::BRUTAL_AI),
        _ => None,
    };
    // An unknown level fails the string but the remaining fields are still read.
    if let Some(state) = state {
        slot.set_state(state, Vec::new(), 0);
    }
    slot.color = bounded(field()?, -1, rules.num_colors() - 1)?;
    slot.player_template = bounded(
        field()?,
        PLAYERTEMPLATE_MIN,
        rules.player_template_count() - 1,
    )?;
    // The C++ duplicate check compares the slot with itself before it is set, so
    // two computer players may share a start position.
    slot.start_pos = bounded(field()?, -1, MAX_SLOTS as i32 - 1)?;
    slot.team_number = bounded(field()?, -1, MAX_SLOTS as i32 / 2 - 1)?;
    state.map(|_| ())
}

impl GameOptions {
    /// `ParseAsciiStringToGameInfo`.
    pub fn parse(options: &[u8], rules: &dyn ReplayRules) -> Option<Self> {
        let options = &options[..options
            .iter()
            .position(|&b| b == 0)
            .unwrap_or(options.len())];
        let mut info = GameOptions {
            crc_interval: 100,
            use_stats: 1,
            starting_cash: rules.default_starting_cash(),
            ..Self::default()
        };
        let mut ok = true;
        let (mut saw_map, mut saw_crc, mut saw_size) = (false, false, false);
        let (mut saw_seed, mut saw_slots, mut saw_interval) = (false, false, false);
        let mut rest = options;
        while let Some(pair) = next_token(&mut rest, b";") {
            let mut inner = pair;
            let key = next_token(&mut inner, b"=").unwrap_or(&[]);
            let val = next_token(&mut inner, b"\n").unwrap_or(&[]);
            if val.is_empty() {
                ok = false;
                break;
            }
            match key {
                b"US" => info.use_stats = atoi(val),
                b"M" => {
                    if val.len() < 3 {
                        ok = false;
                        break;
                    }
                    // `grabHexInt`: `strtol` of `0x` and the first two characters.
                    info.map_contents_mask = val[..2]
                        .iter()
                        .map_while(|&b| char::from(b).to_digit(16))
                        .fold(0, |acc, d| acc * 16 + d as i32);
                    match resolve_map(val, rules) {
                        Some(real) => info.map = real,
                        None => {
                            ok = false;
                            break;
                        }
                    }
                    saw_map = true;
                }
                b"MC" => {
                    info.map_crc = strtoul(val, 16).unwrap_or(0);
                    saw_crc = true;
                }
                b"MS" => {
                    info.map_size = atoi(val) as u32;
                    saw_size = true;
                }
                b"SD" => {
                    info.seed = atoi(val);
                    saw_seed = true;
                }
                b"C" => {
                    // `setCRCInterval` caps the interval at 100.
                    info.crc_interval = atoi(val).min(REPLAY_CRC_INTERVAL);
                    saw_interval = true;
                }
                b"SR" => info.superweapon_restriction = atoi(val) as u16,
                b"SC" => info.starting_cash = strtoul(val, 10).unwrap_or(0),
                b"O" => info.old_factions_only = val.eq_ignore_ascii_case(b"Y"),
                b"S" => {
                    saw_slots = true;
                    ok &= parse_slots(val, &mut info.slots, rules);
                }
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if !(ok && saw_map && saw_crc && saw_size && saw_seed && saw_slots && saw_interval) {
            return None;
        }
        if let Some(found) = rules.find_map_by_crc(&info.map, info.map_crc, info.map_size) {
            info.map = found;
        }
        // `GameInfo::setSlot` on slot 0.
        info.slots[0].accepted = true;
        info.slots[0].has_map = true;
        info.local_slot = -1;
        Some(info)
    }

    /// `GameInfoToAsciiString` outside a LAN game.
    pub fn to_ascii(&self, rules: &dyn ReplayRules) -> Vec<u8> {
        const SEPS: &[u8] = b"\\/";
        let mut portable = rules.portable_map_path(&self.map);
        let mut directory = Vec::new();
        if !portable.is_empty() {
            let mut token = ascii_next_token(&mut portable, SEPS);
            while portable.iter().any(|b| SEPS.contains(b)) {
                if !directory.is_empty() {
                    directory.push(b'/');
                }
                directory.extend_from_slice(&token);
                token = ascii_next_token(&mut portable, SEPS);
            }
        }
        let mut out =
            format!("US={};M={:02x}", self.use_stats, self.map_contents_mask).into_bytes();
        out.extend_from_slice(&percent_encode(&directory));
        out.extend_from_slice(
            format!(
                ";MC={:X};MS={};SD={};C={};SR={};SC={};O={};S=",
                self.map_crc,
                self.map_size as i32,
                self.seed,
                self.crc_interval,
                self.superweapon_restriction,
                self.starting_cash,
                if self.old_factions_only { 'Y' } else { 'N' },
            )
            .as_bytes(),
        );
        let flag = |v: bool| if v { 'T' } else { 'F' };
        for s in &self.slots {
            if s.is_human() {
                out.push(b'H');
                out.extend_from_slice(&bytes_from_wide(&s.name));
                out.extend_from_slice(
                    format!(
                        ",{:X},{},{}{},{},{},{},{},{}:",
                        s.ip,
                        s.port,
                        flag(s.accepted),
                        flag(s.has_map),
                        s.color,
                        s.player_template,
                        s.start_pos,
                        s.team_number,
                        s.nat_behavior
                    )
                    .as_bytes(),
                );
            } else if s.is_ai() {
                let level = match s.state {
                    slot_state::EASY_AI => 'E',
                    slot_state::MED_AI => 'M',
                    _ => 'H',
                };
                out.extend_from_slice(
                    format!(
                        "C{level},{},{},{},{}:",
                        s.color, s.player_template, s.start_pos, s.team_number
                    )
                    .as_bytes(),
                );
            } else if s.state == slot_state::OPEN {
                out.extend_from_slice(b"O:");
            } else {
                out.extend_from_slice(b"X:");
            }
        }
        out.push(b';');
        out
    }
}
