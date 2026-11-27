//! `PKT-087`: Deterministic `.ini` line reader, tokenizer, 28+ type/unit/flag scanners,
//! and `FieldParse` block dispatcher (`Core/GameEngine/Source/Common/INI/INI.cpp`).
//!
//! Reference C++ packet: `packets/phase2/PKT-087_INI.md`
//! Golden Vector test: `crates/zh_common/tests/golden_pkt087.rs` (`docs/golden_vectors/PKT-087.json`)

#![allow(unused_variables, dead_code)]

pub mod ini_terrain_bridge;
pub mod ini_terrain_road;

use crate::math::{gm_ceilf, Coord2D, Coord3D, ICoord2D, RGBAColorInt, RGBColor, PI};
use crate::random_value::DistributionType;
use crate::types::{Bool, Byte, Int, Real, Short, UnsignedByte, UnsignedInt, UnsignedShort, FALSE, TRUE};
use crate::xfer::bit_flags::BitFlags;
use crate::xfer::Xfer;

pub const INI_MAX_CHARS_PER_LINE: usize = 1028;

pub const LOGICFRAMES_PER_SECOND: Int = 30;
pub const MSEC_PER_SECOND: Int = 1000;
pub const LOGICFRAMES_PER_MSEC_REAL: Real =
    (LOGICFRAMES_PER_SECOND as Real) / (MSEC_PER_SECOND as Real);
pub const MSEC_PER_LOGICFRAME_REAL: Real =
    (MSEC_PER_SECOND as Real) / (LOGICFRAMES_PER_SECOND as Real);
pub const LOGICFRAMES_PER_SECONDS_REAL: Real = LOGICFRAMES_PER_SECOND as Real;
pub const SECONDS_PER_LOGICFRAME_REAL: Real = 1.0_f32 / LOGICFRAMES_PER_SECONDS_REAL;
pub const RADS_PER_DEGREE: Real = PI / 180.0_f32;

pub const SEPS_DEFAULT: &str = " \n\r\t=";
pub const SEPS_PERCENT: &str = " \n\r\t=%";
pub const SEPS_COLON: &str = " \n\r\t=:";
pub const SEPS_QUOTE: &str = "\"\n=";
pub const END_TOKEN: &str = "End";

pub type Color = UnsignedInt;
pub type VeterancyLevelFlags = UnsignedInt;
pub type DeathTypeFlags = UnsignedInt;
pub type DamageTypeFlags = BitFlags<38>;

pub const VETERANCY_LEVEL_FLAGS_ALL: VeterancyLevelFlags = 0xffff_ffff;
pub const VETERANCY_LEVEL_FLAGS_NONE: VeterancyLevelFlags = 0x0000_0000;

pub const DEATH_TYPE_FLAGS_ALL: DeathTypeFlags = 0xffff_ffff;
pub const DEATH_TYPE_FLAGS_NONE: DeathTypeFlags = 0x0000_0000;

pub const VETERANCY_NAMES: [&str; 4] = ["REGULAR", "VETERAN", "ELITE", "HEROIC"];

pub const DISTRIBUTION_TYPE_NAMES: [&str; 6] = [
    "CONSTANT",
    "UNIFORM",
    "GAUSSIAN",
    "TRIANGULAR",
    "LOW_BIAS",
    "HIGH_BIAS",
];

/// Zero Hour 38-entry damage type name table matching `Damage.cpp` (`-DRTS_ZEROHOUR=1 -DRTS_GENERALS=0`).
pub const DAMAGE_TYPE_NAMES: [&str; 38] = [
    "EXPLOSION",
    "CRUSH",
    "ARMOR_PIERCING",
    "SMALL_ARMS",
    "GATTLING",
    "RADIATION",
    "FLAME",
    "LASER",
    "SNIPER",
    "POISON",
    "HEALING",
    "UNRESISTABLE",
    "WATER",
    "DEPLOY",
    "SURRENDER",
    "HACK",
    "KILL_PILOT",
    "PENALTY",
    "FALLING",
    "MELEE",
    "DISARM",
    "HAZARD_CLEANUP",
    "PARTICLE_BEAM",
    "TOPPLING",
    "INFANTRY_MISSILE",
    "AURORA_BOMB",
    "LAND_MINE",
    "JET_MISSILES",
    "STEALTHJET_MISSILES",
    "MOLOTOV_COCKTAIL",
    "COMANCHE_VULCAN",
    "SUBDUAL_MISSILE",
    "SUBDUAL_VEHICLE",
    "SUBDUAL_BUILDING",
    "SUBDUAL_UNRESISTABLE",
    "MICROWAVE",
    "KILL_GARRISONED",
    "STATUS",
];

/// Zero Hour 21-entry death type name table matching `TheDeathNames` in `Damage.h`.
pub const DEATH_TYPE_NAMES: [&str; 21] = [
    "NORMAL",
    "NONE",
    "CRUSHED",
    "BURNED",
    "EXPLODED",
    "POISONED",
    "TOPPLED",
    "FLOODED",
    "SUICIDED",
    "LASERED",
    "DETONATED",
    "SPLATTED",
    "POISONED_BETA",
    "EXTRA_2",
    "EXTRA_3",
    "EXTRA_4",
    "EXTRA_5",
    "EXTRA_6",
    "EXTRA_7",
    "EXTRA_8",
    "POISONED_GAMMA",
];

#[inline]
pub const fn game_make_color(
    red: UnsignedByte,
    green: UnsignedByte,
    blue: UnsignedByte,
    alpha: UnsignedByte,
) -> Color {
    ((alpha as u32) << 24) | ((red as u32) << 16) | ((green as u32) << 8) | (blue as u32)
}

#[inline]
pub fn convert_duration_from_msecs_to_frames(msec: Real) -> Real {
    msec * LOGICFRAMES_PER_MSEC_REAL
}

#[inline]
pub fn convert_velocity_in_secs_to_frames(dist_per_sec: Real) -> Real {
    dist_per_sec * SECONDS_PER_LOGICFRAME_REAL
}

#[inline]
pub fn convert_acceleration_in_secs_to_frames(dist_per_sec2: Real) -> Real {
    let sec_per_logicframe_sqr = SECONDS_PER_LOGICFRAME_REAL * SECONDS_PER_LOGICFRAME_REAL;
    dist_per_sec2 * sec_per_logicframe_sqr
}

#[inline]
pub fn convert_angular_velocity_in_degrees_per_sec_to_rads_per_frame(deg_per_sec: Real) -> Real {
    deg_per_sec * (SECONDS_PER_LOGICFRAME_REAL * RADS_PER_DEGREE)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IniError {
    InvalidData,
    InvalidNameList,
    UnknownToken,
    MissingEndToken,
    InvalidParams,
    InvalidDirectory,
    FileAlreadyOpen,
    CantOpenFile,
    OutOfRange,
    Bug,
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IniLoadType {
    #[default]
    Invalid = 0,
    Overwrite = 1,
    CreateOverrides = 2,
    Multifile = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LookupListRec {
    pub name: &'static str,
    pub value: Int,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct GameClientRandomVariable {
    pub dist_type: DistributionType,
    pub low: Real,
    pub high: Real,
}

impl GameClientRandomVariable {
    #[inline]
    pub fn set_range(&mut self, low: Real, high: Real, dist_type: DistributionType) {
        self.low = low;
        self.high = high;
        self.dist_type = dist_type;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FieldUserData {
    None,
    Mask(UnsignedInt),
    Names(&'static [&'static str]),
    Lookup(&'static [LookupListRec]),
}

pub type IniFieldHandler<T> =
    fn(&mut IniReader<'_>, &mut T, &FieldUserData) -> Result<(), IniError>;

pub struct FieldParseEntry<T: 'static> {
    pub token: Option<&'static str>,
    pub parse: IniFieldHandler<T>,
    pub user_data: FieldUserData,
}

pub struct MultiIniFieldParse<T: 'static> {
    tables: Vec<&'static [FieldParseEntry<T>]>,
}

impl<T: 'static> Default for MultiIniFieldParse<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> MultiIniFieldParse<T> {
    pub const MAX_MULTI_FIELDS: usize = 16;

    pub fn new() -> Self {
        Self { tables: Vec::new() }
    }

    pub fn add(&mut self, table: &'static [FieldParseEntry<T>]) -> Result<(), IniError> {
        if self.tables.len() >= Self::MAX_MULTI_FIELDS {
            return Err(IniError::Bug);
        }
        self.tables.push(table);
        Ok(())
    }
}

#[derive(Clone)]
pub struct IniReader<'a> {
    filename: String,
    load_type: IniLoadType,
    line_num: UnsignedInt,
    data: &'a [u8],
    pos: usize,
    end_of_file: bool,
    line_buf: Vec<u8>,
    token_cursor: usize,
    capture_xfer_lines: bool,
    captured_xfer_lines: Vec<Vec<u8>>,
}

impl<'a> IniReader<'a> {
    pub fn new(filename: &str, data: &'a [u8], load_type: IniLoadType) -> Self {
        Self {
            filename: filename.to_string(),
            load_type,
            line_num: 0,
            data,
            pos: 0,
            end_of_file: false,
            line_buf: Vec::with_capacity(INI_MAX_CHARS_PER_LINE),
            token_cursor: 0,
            capture_xfer_lines: false,
            captured_xfer_lines: Vec::new(),
        }
    }

    #[inline]
    pub fn get_filename(&self) -> &str {
        &self.filename
    }

    #[inline]
    pub const fn get_load_type(&self) -> IniLoadType {
        self.load_type
    }

    #[inline]
    pub const fn get_line_num(&self) -> UnsignedInt {
        self.line_num
    }

    #[inline]
    pub const fn is_eof(&self) -> bool {
        self.end_of_file
    }

    #[inline]
    pub fn current_line_bytes(&self) -> &[u8] {
        &self.line_buf
    }

    pub fn read_line(&mut self, xfer: Option<&mut dyn Xfer>) -> Result<(), IniError> {
        self.line_buf.clear();
        self.token_cursor = 0;
        if self.end_of_file {
            if let Some(xfer) = xfer {
                let mut empty: [u8; 0] = [];
                xfer.xfer_bytes(&mut empty).map_err(|_| IniError::InvalidData)?;
            }
            return Ok(());
        }

        let mut truncated = false;
        while self.line_buf.len() < INI_MAX_CHARS_PER_LINE && self.pos < self.data.len() {
            let b = self.data[self.pos];
            self.pos += 1;
            if b == b'\n' {
                break;
            }
            if truncated {
                continue;
            }
            if b == b';' {
                truncated = true;
                continue;
            }
            let out = if b > 0 && b < 32 { b' ' } else { b };
            self.line_buf.push(out);
        }
        if self.pos >= self.data.len() {
            self.end_of_file = true;
        }
        self.line_num = self.line_num.wrapping_add(1);
        if self.capture_xfer_lines {
            self.captured_xfer_lines.push(self.line_buf.clone());
        }

        if let Some(xfer) = xfer {
            let mut bytes = self.line_buf.clone();
            xfer.xfer_bytes(&mut bytes).map_err(|_| IniError::InvalidData)?;
        }
        Ok(())
    }

    pub fn next_token(&mut self, seps: &str) -> Result<&str, IniError> {
        self.next_token_or_null(seps).ok_or(IniError::InvalidData)
    }

    pub fn next_token_or_null(&mut self, seps: &str) -> Option<&str> {
        let bytes = &self.line_buf;
        while self.token_cursor < bytes.len()
            && seps.as_bytes().contains(&bytes[self.token_cursor])
        {
            self.token_cursor += 1;
        }
        if self.token_cursor >= bytes.len() {
            return None;
        }
        let start = self.token_cursor;
        while self.token_cursor < bytes.len()
            && !seps.as_bytes().contains(&bytes[self.token_cursor])
        {
            self.token_cursor += 1;
        }
        let end = self.token_cursor;
        std::str::from_utf8(&bytes[start..end]).ok()
    }

    /// `strtok` overwrites the separator that ended a token; [`Self::next_token`] leaves
    /// it for the next call. Needed where the separator set changes after a `:`.
    pub fn consume_separator(&mut self) {
        if self.token_cursor < self.line_buf.len() {
            self.token_cursor += 1;
        }
    }

    pub fn next_sub_token(&mut self, expected: &str) -> Result<&str, IniError> {
        let token = self.next_token(SEPS_COLON)?;
        if !token.eq_ignore_ascii_case(expected) {
            return Err(IniError::InvalidData);
        }
        self.next_token(SEPS_COLON)
    }

    /// `INI::getNextAsciiString`. A quoted value is read as two tokens: the first up to
    /// a separator, the rest up to a quote. The two are joined by a space only when the
    /// rest is longer than one character, and only a value with no rest loses a closing
    /// quote from its first token.
    pub fn next_ascii_string(&mut self) -> Result<String, IniError> {
        let Some(token) = self.next_token_or_null(SEPS_DEFAULT) else {
            return Ok(String::new());
        };
        let Some(first) = token.strip_prefix('"') else {
            return Ok(token.to_string());
        };
        let mut result = first.to_string();
        // `strtok` has consumed the separator that ended the first token.
        if self.token_cursor < self.line_buf.len() {
            self.token_cursor += 1;
        }
        match self.next_token_or_null(SEPS_QUOTE) {
            Some(rest) => {
                if rest.len() > 1 && rest.as_bytes()[1] != b'\t' {
                    result.push(' ');
                }
                result.push_str(rest);
                // The quote that ended it is consumed too.
                if self.token_cursor < self.line_buf.len() {
                    self.token_cursor += 1;
                }
            }
            None => {
                if result.ends_with('"') {
                    result.pop();
                }
            }
        }
        Ok(result)
    }

    pub fn next_quoted_ascii_string(&mut self) -> Result<String, IniError> {
        let Some(token) = self.next_token_or_null(SEPS_DEFAULT) else {
            return Ok(String::new());
        };
        if token.starts_with('"') {
            let mut result = token[1..].to_string();
            if result.ends_with('"') {
                result.pop();
                return Ok(result);
            }
            if let Some(rest) = self.next_token_or_null(SEPS_QUOTE) {
                if !rest.is_empty() {
                    result.push(' ');
                    result.push_str(rest.trim_start_matches(char::is_whitespace).trim_end_matches('"'));
                }
            }
            Ok(result)
        } else {
            Ok(token.to_string())
        }
    }

    pub fn load_blocks<F>(
        &mut self,
        xfer: Option<&mut dyn Xfer>,
        mut dispatch: F,
    ) -> Result<UnsignedInt, IniError>
    where
        F: FnMut(&str, &mut IniReader<'a>) -> Result<(), IniError>,
    {
        let mut count = 0u32;
        self.capture_xfer_lines = true;
        self.captured_xfer_lines.clear();
        loop {
            self.read_line(None)?;
            if self.is_eof() && self.current_line_bytes().is_empty() {
                break;
            }
            let block = match std::str::from_utf8(self.current_line_bytes()) {
                Ok(s) => s.split_ascii_whitespace().next().map(str::to_string),
                Err(_) => None,
            };
            if let Some(block) = block {
                if block.eq_ignore_ascii_case("End") {
                    continue;
                }
                count = count.wrapping_add(1);
                let block_name = block;
                dispatch(&block_name, self)?;
            }
            if self.is_eof() {
                break;
            }
        }
        self.capture_xfer_lines = false;
        if let Some(xfer) = xfer {
            for line in self.captured_xfer_lines.drain(..) {
                let mut bytes = line;
                xfer.xfer_bytes(&mut bytes).map_err(|_| IniError::InvalidData)?;
            }
        } else {
            self.captured_xfer_lines.clear();
        }
        Ok(count)
    }

    pub fn init_from_ini<T: 'static>(
        &mut self,
        target: &mut T,
        table: &'static [FieldParseEntry<T>],
    ) -> Result<(), IniError> {
        let mut multi = MultiIniFieldParse::new();
        multi.add(table)?;
        self.init_from_ini_multi(target, &multi)
    }

    pub fn init_from_ini_multi<T: 'static>(
        &mut self,
        target: &mut T,
        multi: &MultiIniFieldParse<T>,
    ) -> Result<(), IniError> {
        self.read_block_fields(|field, ini| {
            let mut found = false;
            for table in &multi.tables {
                if let Some(entry) = table.iter().find(|entry| {
                    entry.token.is_some_and(|token| token.eq_ignore_ascii_case(field))
                }) {
                    (entry.parse)(ini, target, &entry.user_data)?;
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(IniError::UnknownToken);
            }
            Ok(())
        })
    }

    /// Reads lines inside a block until a standalone `End` token, positioning `token_cursor`
    /// right after each field name before invoking `on_field`.
    pub fn read_block_fields<F>(&mut self, mut on_field: F) -> Result<(), IniError>
    where
        F: FnMut(&str, &mut IniReader<'a>) -> Result<(), IniError>,
    {
        loop {
            self.read_line(None)?;
            let field = match std::str::from_utf8(self.current_line_bytes()) {
                Ok(s) => {
                    // `strtok` with `INI::getSeps()`: `=` ends a field name too.
                    let is_sep = |c: char| c.is_whitespace() || c == '=';
                    let trimmed = s.trim_start_matches(is_sep);
                    let field_end = trimmed.find(is_sep).unwrap_or(trimmed.len());
                    let field = &trimmed[..field_end];
                    let start = s.len() - trimmed.len();
                    let cursor = start + field_end;
                    if field.is_empty() { None } else { Some(field.to_string()) }
                        .map(|field| (field, cursor))
                }
                Err(_) => None,
            };
            if let Some((field, cursor)) = field {
                self.token_cursor = cursor;
                if field.eq_ignore_ascii_case("End") || Self::is_end_of_block(&self.line_buf) != FALSE {
                    if self.is_eof() && !field.eq_ignore_ascii_case("End") {
                        return Err(IniError::MissingEndToken);
                    }
                    break;
                }
                on_field(&field, self)?;
            }
            if self.is_eof() {
                return Err(IniError::MissingEndToken);
            }
        }
        Ok(())
    }

    /// Consumes lines inside an INI block until its closing `End` token.
    pub fn skip_block(&mut self) -> Result<(), IniError> {
        self.read_block_fields(|_field, _ini| Ok(()))
    }

    pub fn is_declaration_of_type(block_type: &str, block_name: &str, line: &[u8]) -> Bool {
        if block_type.is_empty() || block_name.is_empty() {
            return FALSE;
        }
        let mut i = 0;
        while i < line.len() && matches!(line[i], b' '|b'\t'|b'\r'|b'\n') {
            i += 1;
        }
        let bt = block_type.as_bytes();
        if line.len() < i + bt.len()
            || !line[i..i + bt.len()].eq_ignore_ascii_case(bt)
        {
            return FALSE;
        }
        i += bt.len();
        if i >= line.len() || !matches!(line[i], b' '|b'\t'|b'\r'|b'\n') {
            return FALSE;
        }
        i += 1;
        while i < line.len() && matches!(line[i], b' '|b'\t'|b'\r'|b'\n') {
            i += 1;
        }
        let bn = block_name.as_bytes();
        if line.len() < i + bn.len()
            || !line[i..i + bn.len()].eq_ignore_ascii_case(bn)
        {
            return FALSE;
        }
        i += bn.len();
        while i < line.len() && matches!(line[i], b' '|b'\t'|b'\r'|b'\n') {
            i += 1;
        }
        if i == line.len() { TRUE } else { FALSE }
    }

    pub fn is_end_of_block(line: &[u8]) -> Bool {
        let mut i = 0;
        while i < line.len() && matches!(line[i], b' '|b'\t'|b'\r'|b'\n') {
            i += 1;
        }
        if line.len().saturating_sub(i) <= 3 {
            return FALSE;
        }
        if i + 3 > line.len()
            || !line[i..i + 3].eq_ignore_ascii_case(b"End")
        {
            return FALSE;
        }
        i += 3;
        while i < line.len() && matches!(line[i], b' '|b'\t'|b'\r'|b'\n') {
            i += 1;
        }
        if i == line.len() { TRUE } else { FALSE }
    }

    pub fn scan_int(token: &str) -> Result<Int, IniError> {
        if token.is_empty() || token.starts_with("+-") {
            return Err(IniError::InvalidData);
        }
        let bytes = token.as_bytes();
        let mut i = 0;
        if bytes[i] == b'+' {
            i += 1;
        }
        let negative = i < bytes.len() && bytes[i] == b'-';
        if negative {
            i += 1;
        }
        let start = i;
        let mut magnitude: i128 = 0;
        let mut digits = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            digits += 1;
            magnitude = magnitude.saturating_mul(10).saturating_add((bytes[i] - b'0') as i128);
            i += 1;
        }
        if digits == 0 {
            return Err(IniError::InvalidData);
        }
        let _ = start;
        let value = if negative { -magnitude } else { magnitude };
        if value > i32::MAX as i128 {
            Ok(i32::MAX)
        } else if value < i32::MIN as i128 {
            Ok(i32::MIN)
        } else {
            Ok(value as i32)
        }
    }

    pub fn scan_unsigned_int(token: &str) -> Result<UnsignedInt, IniError> {
        if token.is_empty() || token.starts_with("+-") {
            return Err(IniError::InvalidData);
        }
        let bytes = token.as_bytes();
        let mut i = 0;
        if bytes[i] == b'+' {
            i += 1;
        }
        let negative = i < bytes.len() && bytes[i] == b'-';
        if negative {
            i += 1;
        }
        let mut magnitude: i128 = 0;
        let mut digits = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            digits += 1;
            magnitude = magnitude.saturating_mul(10).saturating_add((bytes[i] - b'0') as i128);
            i += 1;
        }
        if digits == 0 {
            return Err(IniError::InvalidData);
        }
        if negative {
            Ok((0i128.wrapping_sub(magnitude) as i64) as u32)
        } else if magnitude > u32::MAX as i128 {
            Ok(u32::MAX)
        } else {
            Ok(magnitude as u32)
        }
    }

    pub fn scan_real(token: &str) -> Result<Real, IniError> {
        if token.is_empty() || token.starts_with("+-") {
            return Err(IniError::InvalidData);
        }
        let bytes = token.as_bytes();
        let mut i = 0;
        if bytes[i] == b'+' || bytes[i] == b'-' {
            i += 1;
        }
        let mut digits = 0;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            digits += 1;
            i += 1;
        }
        if i < bytes.len() && bytes[i] == b'.' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                digits += 1;
                i += 1;
            }
        }
        if digits == 0 {
            return Err(IniError::InvalidData);
        }
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            i += 1;
            if i < bytes.len() && (bytes[i] == b'+' || bytes[i] == b'-') {
                i += 1;
            }
            let exp_start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            if i == exp_start {
                return Err(IniError::InvalidData);
            }
        }
        let parsed: f64 = token[..i].parse::<f64>() // determinism-ok: pre-validated decimal prefix for C++ from_chars/strtod parity
            .map_err(|_| IniError::InvalidData)?;
        let value = parsed as Real;
        if value.is_infinite() || value.is_nan() {
            Err(IniError::InvalidData)
        } else {
            Ok(value)
        }
    }

    pub fn scan_percent_to_real(token: &str) -> Result<Real, IniError> {
        Ok(Self::scan_real(token)? / 100.0_f32)
    }

    pub fn scan_bool(token: &str) -> Result<Bool, IniError> {
        if token.eq_ignore_ascii_case("Yes") {
            Ok(TRUE)
        } else if token.eq_ignore_ascii_case("No") {
            Ok(FALSE)
        } else {
            Err(IniError::InvalidData)
        }
    }

    pub fn scan_index_list(token: &str, name_list: &[&str]) -> Result<Int, IniError> {
        if name_list.is_empty() {
            return Err(IniError::InvalidNameList);
        }
        for (i, &name) in name_list.iter().enumerate() {
            if token.eq_ignore_ascii_case(name) {
                return Ok(i as Int);
            }
        }
        Err(IniError::InvalidData)
    }

    pub fn scan_lookup_list(token: &str, lookup_list: &[LookupListRec]) -> Result<Int, IniError> {
        if lookup_list.is_empty() {
            return Err(IniError::InvalidNameList);
        }
        for rec in lookup_list {
            if token.eq_ignore_ascii_case(rec.name) {
                return Ok(rec.value);
            }
        }
        Err(IniError::InvalidData)
    }

    pub fn parse_unsigned_byte(&mut self, store: &mut UnsignedByte) -> Result<(), IniError> {
        let value = Self::scan_int(self.next_token(SEPS_DEFAULT)?)?;
        if !(0..=255).contains(&value) { return Err(IniError::OutOfRange); }
        *store = value as UnsignedByte;
        Ok(())
    }

    pub fn parse_short(&mut self, store: &mut Short) -> Result<(), IniError> {
        let value = Self::scan_int(self.next_token(SEPS_DEFAULT)?)?;
        if value < -32768 || value > 32767 { return Err(IniError::OutOfRange); }
        *store = value as Short;
        Ok(())
    }

    pub fn parse_unsigned_short(&mut self, store: &mut UnsignedShort) -> Result<(), IniError> {
        let value = Self::scan_int(self.next_token(SEPS_DEFAULT)?)?;
        if value < 0 || value > 65535 { return Err(IniError::OutOfRange); }
        *store = value as UnsignedShort;
        Ok(())
    }

    pub fn parse_int(&mut self, store: &mut Int) -> Result<(), IniError> {
        *store = Self::scan_int(self.next_token(SEPS_DEFAULT)?)?;
        Ok(())
    }

    pub fn parse_unsigned_int(&mut self, store: &mut UnsignedInt) -> Result<(), IniError> {
        *store = Self::scan_unsigned_int(self.next_token(SEPS_DEFAULT)?)?;
        Ok(())
    }

    pub fn parse_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        *store = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        Ok(())
    }

    pub fn parse_positive_non_zero_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        let value = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        if value <= 0.0 { return Err(IniError::InvalidData); }
        *store = value;
        Ok(())
    }

    pub fn parse_bool(&mut self, store: &mut Bool) -> Result<(), IniError> {
        *store = Self::scan_bool(self.next_token(SEPS_DEFAULT)?)?;
        Ok(())
    }

    pub fn parse_bit_in_int32(
        &mut self,
        store: &mut UnsignedInt,
        mask: UnsignedInt,
    ) -> Result<(), IniError> {
        if Self::scan_bool(self.next_token(SEPS_DEFAULT)?)? == TRUE {
            *store |= mask;
        } else {
            *store &= !mask;
        }
        Ok(())
    }

    pub fn parse_ascii_string(&mut self, store: &mut String) -> Result<(), IniError> {
        *store = self.next_ascii_string()?;
        Ok(())
    }

    pub fn parse_quoted_ascii_string(&mut self, store: &mut String) -> Result<(), IniError> {
        *store = self.next_quoted_ascii_string()?;
        Ok(())
    }

    pub fn parse_ascii_string_vector(&mut self, store: &mut Vec<String>) -> Result<(), IniError> {
        store.clear();
        while let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            store.push(token.to_string());
        }
        Ok(())
    }

    pub fn parse_ascii_string_vector_append(
        &mut self,
        store: &mut Vec<String>,
    ) -> Result<(), IniError> {
        while let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            store.push(token.to_string());
        }
        Ok(())
    }

    pub fn parse_sounds_list(&mut self, store: &mut Vec<String>) -> Result<(), IniError> {
        store.clear();
        while let Some(token) = self.next_token_or_null(" \t,=") {
            store.push(token.to_string());
        }
        Ok(())
    }

    pub fn parse_percent_to_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        *store = Self::scan_percent_to_real(self.next_token(SEPS_PERCENT)?)?;
        Ok(())
    }

    pub fn parse_angle_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        *store = Self::scan_real(self.next_token(SEPS_DEFAULT)?)? * RADS_PER_DEGREE;
        Ok(())
    }

    pub fn parse_angular_velocity_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        let val = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        *store = convert_angular_velocity_in_degrees_per_sec_to_rads_per_frame(val);
        Ok(())
    }

    pub fn parse_velocity_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        let val = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        *store = convert_velocity_in_secs_to_frames(val);
        Ok(())
    }

    pub fn parse_acceleration_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        let val = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        *store = convert_acceleration_in_secs_to_frames(val);
        Ok(())
    }

    pub fn parse_duration_real(&mut self, store: &mut Real) -> Result<(), IniError> {
        let val = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        *store = convert_duration_from_msecs_to_frames(val);
        Ok(())
    }

    pub fn parse_duration_unsigned_int(&mut self, store: &mut UnsignedInt) -> Result<(), IniError> {
        let val = Self::scan_unsigned_int(self.next_token(SEPS_DEFAULT)?)?;
        let frames = convert_duration_from_msecs_to_frames(val as Real);
        *store = gm_ceilf(frames) as UnsignedInt;
        Ok(())
    }

    pub fn parse_duration_unsigned_short(
        &mut self,
        store: &mut UnsignedShort,
    ) -> Result<(), IniError> {
        let val = Self::scan_unsigned_int(self.next_token(SEPS_DEFAULT)?)?;
        let frames = convert_duration_from_msecs_to_frames(val as Real);
        *store = gm_ceilf(frames) as UnsignedShort;
        Ok(())
    }

    pub fn parse_rgb_color(&mut self, store: &mut RGBColor) -> Result<(), IniError> {
        let r = Self::scan_int(self.next_sub_token("R")?)?;
        let g = Self::scan_int(self.next_sub_token("G")?)?;
        let b = Self::scan_int(self.next_sub_token("B")?)?;
        if r < 0 || g < 0 || b < 0 || r > 255 || g > 255 || b > 255 {
            return Err(IniError::InvalidData);
        }
        store.red = r as Real / 255.0_f32;
        store.green = g as Real / 255.0_f32;
        store.blue = b as Real / 255.0_f32;
        Ok(())
    }

    pub fn parse_rgba_color_int(&mut self, store: &mut RGBAColorInt) -> Result<(), IniError> {
        let r = Self::scan_int(self.next_sub_token("R")?)?;
        let g = Self::scan_int(self.next_sub_token("G")?)?;
        let b = Self::scan_int(self.next_sub_token("B")?)?;
        let a = match self.next_token_or_null(SEPS_COLON) {
            Some(name) if name.eq_ignore_ascii_case("A") =>
                Some(Self::scan_int(self.next_token(SEPS_COLON)?)?),
            Some(_) => return Err(IniError::InvalidData),
            None => Some(255),
        };
        let values = [r, g, b, a.unwrap_or(255)];
        if values.iter().any(|&v| !(0..=255).contains(&v)) {
            return Err(IniError::InvalidData);
        }
        store.red = r as UnsignedInt;
        store.green = g as UnsignedInt;
        store.blue = b as UnsignedInt;
        store.alpha = values[3] as UnsignedInt;
        Ok(())
    }

    pub fn parse_color_int(&mut self, store: &mut Color) -> Result<(), IniError> {
        let r = Self::scan_int(self.next_sub_token("R")?)?;
        let g = Self::scan_int(self.next_sub_token("G")?)?;
        let b = Self::scan_int(self.next_sub_token("B")?)?;
        let a = match self.next_token_or_null(SEPS_COLON) {
            Some(name) if name.eq_ignore_ascii_case("A") =>
                Some(Self::scan_int(self.next_token(SEPS_COLON)?)?),
            Some(_) => return Err(IniError::InvalidData),
            None => Some(255),
        }.unwrap_or(255);
        if [r,g,b,a].iter().any(|&v| !(0..=255).contains(&v)) {
            return Err(IniError::InvalidData);
        }
        *store = game_make_color(r as UnsignedByte, g as UnsignedByte, b as UnsignedByte, a as UnsignedByte);
        Ok(())
    }

    pub fn parse_coord2d(&mut self, store: &mut Coord2D) -> Result<(), IniError> {
        store.x = Self::scan_real(self.next_sub_token("X")?)?;
        store.y = Self::scan_real(self.next_sub_token("Y")?)?;
        Ok(())
    }

    pub fn parse_coord3d(&mut self, store: &mut Coord3D) -> Result<(), IniError> {
        store.x = Self::scan_real(self.next_sub_token("X")?)?;
        store.y = Self::scan_real(self.next_sub_token("Y")?)?;
        store.z = Self::scan_real(self.next_sub_token("Z")?)?;
        Ok(())
    }

    pub fn parse_icoord2d(&mut self, store: &mut ICoord2D) -> Result<(), IniError> {
        store.x = Self::scan_int(self.next_sub_token("X")?)?;
        store.y = Self::scan_int(self.next_sub_token("Y")?)?;
        Ok(())
    }

    pub fn parse_game_client_random_variable(
        &mut self,
        store: &mut GameClientRandomVariable,
    ) -> Result<(), IniError> {
        let low = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        let high = Self::scan_real(self.next_token(SEPS_DEFAULT)?)?;
        let mut dist = DistributionType::Uniform;
        if let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            let idx = Self::scan_index_list(token, &DISTRIBUTION_TYPE_NAMES)?;
            if !(0..=DistributionType::Count as i32).contains(&idx) {
                return Err(IniError::InvalidData);
            }
            dist = match idx {
                0 => DistributionType::Constant,
                1 => DistributionType::Uniform,
                2 => DistributionType::Gaussian,
                3 => DistributionType::Triangular,
                4 => DistributionType::LowBias,
                5 => DistributionType::HighBias,
                _ => return Err(IniError::InvalidData),
            };
        }
        store.set_range(low, high, dist);
        Ok(())
    }

    pub fn parse_bit_string8(
        &mut self,
        store: &mut Byte,
        flag_list: &[&str],
    ) -> Result<(), IniError> {
        let mut tmp = *store as UnsignedInt & 0xff;
        self.parse_bit_string32(&mut tmp, flag_list)?;
        if tmp & 0xffffff00 != 0 { return Err(IniError::OutOfRange); }
        *store = tmp as Byte;
        Ok(())
    }

    pub fn parse_bit_string32(
        &mut self,
        store: &mut UnsignedInt,
        flag_list: &[&str],
    ) -> Result<(), IniError> {
        if flag_list.is_empty() {
            return Err(IniError::InvalidNameList);
        }
        let mut found_normal = false;
        let mut found_add_or_sub = false;
        while let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            if token.eq_ignore_ascii_case("NONE") {
                if found_normal || found_add_or_sub {
                    return Err(IniError::InvalidNameList);
                }
                *store = 0;
                break;
            }
            if let Some(name) = token.strip_prefix('+') {
                if found_normal { return Err(IniError::InvalidNameList); }
                let bit = Self::scan_index_list(name, flag_list)?;
                if bit >= 0 { *store |= 1u32 << bit as u32; }
                found_add_or_sub = true;
            } else if let Some(name) = token.strip_prefix('-') {
                if found_normal { return Err(IniError::InvalidNameList); }
                let bit = Self::scan_index_list(name, flag_list)?;
                if bit >= 0 { *store &= !(1u32 << bit as u32); }
                found_add_or_sub = true;
            } else {
                if found_add_or_sub { return Err(IniError::InvalidNameList); }
                if !found_normal { *store = 0; }
                let bit = Self::scan_index_list(token, flag_list)?;
                if bit >= 0 { *store |= 1u32 << bit as u32; }
                found_normal = true;
            }
        }
        Ok(())
    }

    pub fn parse_index_list(
        &mut self,
        store: &mut Int,
        name_list: &[&str],
    ) -> Result<(), IniError> {
        *store = Self::scan_index_list(self.next_token(SEPS_DEFAULT)?, name_list)?;
        Ok(())
    }

    pub fn parse_byte_sized_index_list(
        &mut self,
        store: &mut Byte,
        name_list: &[&str],
    ) -> Result<(), IniError> {
        let value = Self::scan_index_list(self.next_token(SEPS_DEFAULT)?, name_list)?;
        if !(0..=255).contains(&value) { return Err(IniError::OutOfRange); }
        *store = value as Byte;
        Ok(())
    }

    pub fn parse_lookup_list(
        &mut self,
        store: &mut Int,
        lookup_list: &[LookupListRec],
    ) -> Result<(), IniError> {
        *store = Self::scan_lookup_list(self.next_token(SEPS_DEFAULT)?, lookup_list)?;
        Ok(())
    }

    pub fn parse_veterancy_level_flags(
        &mut self,
        store: &mut VeterancyLevelFlags,
    ) -> Result<(), IniError> {
        let mut flags = VETERANCY_LEVEL_FLAGS_ALL;
        while let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            if token.eq_ignore_ascii_case("ALL") {
                flags = VETERANCY_LEVEL_FLAGS_ALL;
            } else if token.eq_ignore_ascii_case("NONE") {
                flags = VETERANCY_LEVEL_FLAGS_NONE;
            } else if let Some(name) = token.strip_prefix('+') {
                let idx = Self::scan_index_list(name, &VETERANCY_NAMES)?;
                if idx >= 0 { flags |= 1u32 << idx as u32; }
            } else if let Some(name) = token.strip_prefix('-') {
                let idx = Self::scan_index_list(name, &VETERANCY_NAMES)?;
                if idx >= 0 { flags &= !(1u32 << idx as u32); }
            } else {
                return Err(IniError::UnknownToken);
            }
        }
        *store = flags;
        Ok(())
    }

    pub fn parse_damage_type_flags(&mut self, store: &mut DamageTypeFlags) -> Result<(), IniError> {
        let mut flags = DamageTypeFlags::new_set_all();
        while let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            if token.eq_ignore_ascii_case("ALL") {
                flags = DamageTypeFlags::new_set_all();
            } else if token.eq_ignore_ascii_case("NONE") {
                flags = DamageTypeFlags::new();
            } else if let Some(name) = token.strip_prefix('+') {
                let idx = Self::scan_index_list(name, &DAMAGE_TYPE_NAMES)?;
                flags.set(idx, 1);
            } else if let Some(name) = token.strip_prefix('-') {
                let idx = Self::scan_index_list(name, &DAMAGE_TYPE_NAMES)?;
                flags.set(idx, 0);
            } else {
                return Err(IniError::UnknownToken);
            }
        }
        *store = flags;
        Ok(())
    }

    pub fn parse_death_type_flags(&mut self, store: &mut DeathTypeFlags) -> Result<(), IniError> {
        let mut flags = DEATH_TYPE_FLAGS_ALL;
        while let Some(token) = self.next_token_or_null(SEPS_DEFAULT) {
            if token.eq_ignore_ascii_case("ALL") {
                flags = DEATH_TYPE_FLAGS_ALL;
            } else if token.eq_ignore_ascii_case("NONE") {
                flags = DEATH_TYPE_FLAGS_NONE;
            } else if let Some(name) = token.strip_prefix('+') {
                let idx = Self::scan_index_list(name, &DEATH_TYPE_NAMES)?;
                if idx >= 0 { flags |= 1u32 << idx as u32; }
            } else if let Some(name) = token.strip_prefix('-') {
                let idx = Self::scan_index_list(name, &DEATH_TYPE_NAMES)?;
                if idx >= 0 { flags &= !(1u32 << idx as u32); }
            } else {
                return Err(IniError::UnknownToken);
            }
        }
        *store = flags;
        Ok(())
    }
}
