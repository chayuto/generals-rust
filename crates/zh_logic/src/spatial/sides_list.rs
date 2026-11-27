//! `PKT-154`: Map player sides, teams, build lists, skirmish scripts, and DataChunk/Xfer
//! serialization (`Core/GameEngine/Include/GameLogic/SidesList.h` &
//! `Core/GameEngine/Source/GameLogic/Map/SidesList.cpp`).
//!
//! Reference packet: `packets/phase4/PKT-154_SidesList.md`
//! Golden vectors: `crates/zh_logic/tests/golden_pkt154_sides_list.rs` (`docs/golden_vectors/PKT-154.json`)

#![allow(unused_variables, dead_code)]

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{
    FieldParseEntry, FieldUserData, IniError, IniReader, SEPS_DEFAULT,
};
use zh_common::io::data_chunk::{
    ChunkInputStream, DataChunkError, DataChunkInfo, DataChunkInput, DataChunkOutput,
    DataChunkOutputStream, DataChunkVersionType,
};
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const MAX_PLAYER_COUNT: usize = 16;
pub const MAX_TEAM_DEPTH: Int = 3;
pub const MAX_SKIRMISH_SIDES: usize = 64;

pub const UNLIMITED_REBUILDS: UnsignedInt = 0xFFFF_FFFF;
pub const MAX_RESOURCE_GATHERERS: usize = 10;

pub const SIDES_DATA_VERSION_1: DataChunkVersionType = 1;
pub const SIDES_DATA_VERSION_2: DataChunkVersionType = 2;
pub const SIDES_DATA_VERSION_3: DataChunkVersionType = 3;

pub const PLAYERS_NAMES_FOR_SCRIPTS_VERSION_1: DataChunkVersionType = 1;
pub const PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2: DataChunkVersionType = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidesError {
    BadArg,
    CorruptFormat,
    InvalidData,
    DataChunk(DataChunkError),
    Ini(IniError),
    Xfer(XferError),
}

impl From<DataChunkError> for SidesError {
    fn from(err: DataChunkError) -> Self {
        Self::DataChunk(err)
    }
}

impl From<IniError> for SidesError {
    fn from(err: IniError) -> Self {
        Self::Ini(err)
    }
}

impl From<XferError> for SidesError {
    fn from(err: XferError) -> Self {
        Self::Xfer(err)
    }
}

/// Pre-interned `Dict` key set matching `Common/WellKnownKeys.h`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WellKnownSidesKeys {
    pub player_name: NameKeyType,
    pub player_is_human: NameKeyType,
    pub player_is_skirmish: NameKeyType,
    pub player_display_name: NameKeyType,
    pub player_faction: NameKeyType,
    pub player_allies: NameKeyType,
    pub player_enemies: NameKeyType,
    pub team_name: NameKeyType,
    pub team_owner: NameKeyType,
    pub team_is_singleton: NameKeyType,
}

impl WellKnownSidesKeys {
    /// Interns the well-known keys in exact C++ oracle order.
    pub fn intern(name_keys: &mut NameKeyGenerator) -> Self {
        Self {
            player_name: name_keys.name_to_key("playerName"),
            player_is_human: name_keys.name_to_key("playerIsHuman"),
            player_is_skirmish: name_keys.name_to_key("playerIsSkirmish"),
            player_display_name: name_keys.name_to_key("playerDisplayName"),
            player_faction: name_keys.name_to_key("playerFaction"),
            player_allies: name_keys.name_to_key("playerAllies"),
            player_enemies: name_keys.name_to_key("playerEnemies"),
            team_name: name_keys.name_to_key("teamName"),
            team_owner: name_keys.name_to_key("teamOwner"),
            team_is_singleton: name_keys.name_to_key("teamIsSingleton"),
        }
    }
}

/// Lightweight deterministic `ScriptList` adapter matching the `PKT-154` C++ oracle codec.
/// It is not the script chunk format of map files; `scripts::ScriptList` reads that.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SidesScriptList {
    pub has_script: Bool,
    pub has_group: Bool,
    pub tag: Int,
}

impl SidesScriptList {
    pub fn new(has_script: Bool, has_group: Bool, tag: Int) -> Self {
        Self {
            has_script,
            has_group,
            tag,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.has_script == FALSE && self.has_group == FALSE
    }

    pub fn write_scripts_data_chunk<O: DataChunkOutputStream>(
        chunk_writer: &mut DataChunkOutput<'_, O>,
        lists: &[Option<&SidesScriptList>],
    ) -> Result<(), DataChunkError> {
        chunk_writer.open_data_chunk("PlayerScriptsList", 1)?;
        chunk_writer.write_int(lists.len() as Int)?;
        for slot in lists {
            match slot {
                Some(sl) => {
                    chunk_writer.write_byte(1)?;
                    chunk_writer.write_byte(if sl.has_script != FALSE { 1 } else { 0 })?;
                    chunk_writer.write_byte(if sl.has_group != FALSE { 1 } else { 0 })?;
                    chunk_writer.write_int(sl.tag)?;
                }
                None => {
                    chunk_writer.write_byte(0)?;
                    chunk_writer.write_byte(0)?;
                    chunk_writer.write_byte(0)?;
                    chunk_writer.write_int(0)?;
                }
            }
        }
        chunk_writer.close_data_chunk()?;
        Ok(())
    }

    pub fn parse_scripts_data_chunk<Ctx>(
        input: &mut DataChunkInput<'_, Ctx>,
        _info: &DataChunkInfo,
    ) -> Result<Vec<Option<SidesScriptList>>, DataChunkError> {
        let count = input.read_int()?;
        let mut out = Vec::new();
        for i in 0..count {
            let present = input.read_byte()?;
            let hs = if input.read_byte()? != 0 { TRUE } else { FALSE };
            let hg = if input.read_byte()? != 0 { TRUE } else { FALSE };
            let tag = input.read_int()?;
            if (i as usize) < MAX_PLAYER_COUNT {
                if present != 0 {
                    out.push(Some(SidesScriptList::new(hs, hg, tag)));
                } else {
                    out.push(None);
                }
            }
        }
        Ok(out)
    }
}

impl Snapshot for SidesScriptList {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut hs = self.has_script as i8;
        let mut hg = self.has_group as i8;
        xfer.xfer_byte(&mut hs)?;
        xfer.xfer_byte(&mut hg)?;
        xfer.xfer_int(&mut self.tag)?;
        self.has_script = if hs != 0 { TRUE } else { FALSE };
        self.has_group = if hg != 0 { TRUE } else { FALSE };
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

/// Map/skirmish build list item matching C++ `BuildListInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildListInfo {
    pub building_name: String,
    pub template_name: String,
    pub location: Coord3D,
    pub rally_point_offset: Coord2D,
    pub angle: Real,
    pub is_initially_built: Bool,
    pub num_rebuilds: UnsignedInt,
    pub script: String,
    pub health: Int,
    pub whiner: Bool,
    pub unsellable: Bool,
    pub repairable: Bool,
    pub automatically_build: Bool,
    pub object_id: ObjectId,
    pub object_timestamp: UnsignedInt,
    pub under_construction: Bool,
    pub resource_gatherers: [ObjectId; MAX_RESOURCE_GATHERERS],
    pub is_supply_building: Bool,
    pub desired_gatherers: Int,
    pub current_gatherers: Int,
    pub priority_build: Bool,
}

impl Default for BuildListInfo {
    fn default() -> Self {
        Self::new()
    }
}

impl BuildListInfo {
    pub fn new() -> Self {
        Self {
            building_name: String::new(),
            template_name: String::new(),
            location: Coord3D::default(),
            rally_point_offset: Coord2D::default(),
            angle: 0.0,
            is_initially_built: FALSE,
            num_rebuilds: 0,
            script: String::new(),
            health: 100,
            whiner: TRUE,
            unsellable: FALSE,
            repairable: TRUE,
            automatically_build: TRUE,
            object_id: ObjectId::INVALID,
            object_timestamp: 0,
            under_construction: FALSE,
            resource_gatherers: [ObjectId::INVALID; MAX_RESOURCE_GATHERERS],
            is_supply_building: FALSE,
            desired_gatherers: 0,
            current_gatherers: 0,
            priority_build: FALSE,
        }
    }

    pub fn decrement_num_rebuilds(&mut self) {
        if self.num_rebuilds != UNLIMITED_REBUILDS && self.num_rebuilds > 0 {
            self.num_rebuilds -= 1;
        }
    }

    pub fn increment_num_rebuilds(&mut self) {
        if self.num_rebuilds != UNLIMITED_REBUILDS {
            self.num_rebuilds = self.num_rebuilds.wrapping_add(1);
        }
    }

    pub fn is_buildable(&self) -> Bool {
        if self.num_rebuilds > 0 || self.num_rebuilds == UNLIMITED_REBUILDS {
            TRUE
        } else {
            FALSE
        }
    }

    pub fn get_gatherer_id(&self, ndx: Int) -> ObjectId {
        if ndx < 0 {
            ObjectId::INVALID
        } else {
            self.resource_gatherers
                .get(ndx as usize)
                .copied()
                .unwrap_or(ObjectId::INVALID)
        }
    }

    pub fn set_gatherer_id(&mut self, ndx: Int, id: ObjectId) {
        if ndx >= 0 {
            if let Some(slot) = self.resource_gatherers.get_mut(ndx as usize) {
                *slot = id;
            }
        }
    }

    /// Parses one `Structure`/`AIData` build list entry via [`IniReader`].
    pub fn parse_structure(ini: &mut IniReader<'_>) -> Result<Self, IniError> {
        let template_name = ini.next_token(SEPS_DEFAULT)?.to_owned();
        let mut out = Self::new();
        out.template_name = template_name;
        ini.init_from_ini(&mut out, BUILD_LIST_FIELD_PARSE)?;
        Ok(out)
    }
}

fn parse_bli_name(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_ascii_string(&mut target.building_name)
}

fn parse_bli_location(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut coord = Coord2D::default();
    ini.parse_coord2d(&mut coord)?;
    target.location.x = coord.x;
    target.location.y = coord.y;
    target.location.z = 0.0;
    Ok(())
}

fn parse_bli_rebuilds(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    let mut rebuilds: Int = 0;
    ini.parse_int(&mut rebuilds)?;
    target.num_rebuilds = rebuilds as UnsignedInt;
    Ok(())
}

fn parse_bli_angle(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_angle_real(&mut target.angle)
}

fn parse_bli_initial(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.is_initially_built)
}

fn parse_bli_rally(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_coord2d(&mut target.rally_point_offset)
}

fn parse_bli_auto(
    ini: &mut IniReader<'_>,
    target: &mut BuildListInfo,
    _ud: &FieldUserData,
) -> Result<(), IniError> {
    ini.parse_bool(&mut target.automatically_build)
}

static BUILD_LIST_FIELD_PARSE: &[FieldParseEntry<BuildListInfo>] = &[
    FieldParseEntry { token: Some("Name"), parse: parse_bli_name, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("Location"), parse: parse_bli_location, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("Rebuilds"), parse: parse_bli_rebuilds, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("Angle"), parse: parse_bli_angle, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("InitiallyBuilt"), parse: parse_bli_initial, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("RallyPointOffset"), parse: parse_bli_rally, user_data: FieldUserData::None },
    FieldParseEntry { token: Some("AutomaticallyBuild"), parse: parse_bli_auto, user_data: FieldUserData::None },
];

impl Snapshot for BuildListInfo {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 2;
        xfer.xfer_version(&mut version, 2)?;
        xfer.xfer_ascii_string(&mut self.building_name)?;
        xfer.xfer_ascii_string(&mut self.template_name)?;
        xfer.xfer_coord3d(&mut self.location)?;
        xfer.xfer_coord2d(&mut self.rally_point_offset)?;
        xfer.xfer_real(&mut self.angle)?;
        xfer.xfer_bool(&mut self.is_initially_built)?;
        xfer.xfer_unsigned_int(&mut self.num_rebuilds)?;
        xfer.xfer_ascii_string(&mut self.script)?;
        xfer.xfer_int(&mut self.health)?;
        xfer.xfer_bool(&mut self.whiner)?;
        xfer.xfer_bool(&mut self.unsellable)?;
        xfer.xfer_bool(&mut self.repairable)?;
        xfer.xfer_bool(&mut self.automatically_build)?;
        xfer.xfer_object_id(&mut self.object_id)?;
        xfer.xfer_unsigned_int(&mut self.object_timestamp)?;
        xfer.xfer_bool(&mut self.under_construction)?;
        let mut raw = [0u8; MAX_RESOURCE_GATHERERS * 4];
        match xfer.mode() {
            XferMode::Save | XferMode::Crc => {
                for (idx, id) in self.resource_gatherers.iter().enumerate() {
                    let start = idx * 4;
                    raw[start..start + 4].copy_from_slice(&id.0.to_le_bytes());
                }
            }
            XferMode::Load => {}
        }
        xfer.xfer_bytes(&mut raw)?;
        if matches!(xfer.mode(), XferMode::Load) {
            for (idx, chunk) in raw.chunks_exact(4).enumerate() {
                self.resource_gatherers[idx] = ObjectId(Int::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
            }
        }
        xfer.xfer_bool(&mut self.is_supply_building)?;
        xfer.xfer_int(&mut self.desired_gatherers)?;
        xfer.xfer_bool(&mut self.priority_build)?;
        if version >= 2 {
            xfer.xfer_int(&mut self.current_gatherers)?;
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

/// Side descriptor + ordered build list + optional `SidesScriptList` (`SidesInfo`).
#[derive(Debug, Clone, Default)]
pub struct SidesInfo {
    pub build_list: Vec<BuildListInfo>,
    pub dict: Dict,
    pub scripts: Option<SidesScriptList>,
}

impl SidesInfo {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, d: Option<&Dict>) {
        self.build_list.clear();
        self.scripts = None;
        self.dict.clear();
        if let Some(src) = d {
            self.dict = src.clone();
        }
    }

    pub fn clear(&mut self) {
        self.init(None);
    }

    pub fn add_to_build_list(&mut self, item: BuildListInfo, position: Int) {
        let len = self.build_list.len() as Int;
        if position <= 0 {
            self.build_list.insert(0, item);
        } else if position >= len {
            self.build_list.push(item);
        } else {
            self.build_list.insert(position as usize, item);
        }
    }

    pub fn remove_from_build_list(&mut self, index: usize) -> Int {
        if index >= self.build_list.len() {
            0
        } else {
            self.build_list.remove(index);
            index as Int
        }
    }

    pub fn reorder_in_build_list(&mut self, index: usize, new_position: Int) {
        if index >= self.build_list.len() {
            return;
        }
        let item = self.build_list.remove(index);
        self.add_to_build_list(item, new_position);
    }

    pub fn release_build_list(&mut self) {
        self.build_list.clear();
    }
}

/// Single team dictionary wrapper (`TeamsInfo`).
#[derive(Debug, Clone, Default)]
pub struct TeamsInfo {
    pub dict: Dict,
}

impl TeamsInfo {
    pub fn init(&mut self, d: Option<&Dict>) {
        self.dict.clear();
        if let Some(src) = d {
            self.dict = src.clone();
        }
    }

    pub fn clear(&mut self) {
        self.init(None);
    }
}

/// Resizable collection of `TeamsInfo` (`TeamsInfoRec`).
#[derive(Debug, Clone, Default)]
pub struct TeamsInfoRec {
    pub teams: Vec<TeamsInfo>,
}

impl TeamsInfoRec {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.teams.clear();
    }

    pub fn find_team_info(
        &self,
        name: &str,
        keys: &WellKnownSidesKeys,
    ) -> Option<Int> {
        for (i, team) in self.teams.iter().enumerate() {
            if team.dict.get_ascii_string(keys.team_name, None) == name {
                return Some(i as Int);
            }
        }
        None
    }

    pub fn add_team(&mut self, d: Option<&Dict>) {
        let mut team = TeamsInfo::default();
        team.init(d);
        self.teams.push(team);
    }

    pub fn remove_team(&mut self, i: Int) {
        if i < 0 || i as usize >= self.teams.len() || self.teams.len() <= 1 {
            return;
        }
        self.teams.remove(i as usize);
    }

    #[inline]
    pub fn num_teams(&self) -> Int {
        self.teams.len() as Int
    }

    pub fn team_info(&self, team: Int) -> Result<&TeamsInfo, SidesError> {
        if team < 0 {
            return Err(SidesError::BadArg);
        }
        self.teams.get(team as usize).ok_or(SidesError::BadArg)
    }

    pub fn team_info_mut(&mut self, team: Int) -> Result<&mut TeamsInfo, SidesError> {
        if team < 0 {
            return Err(SidesError::BadArg);
        }
        self.teams.get_mut(team as usize).ok_or(SidesError::BadArg)
    }
}

/// Singleton map sides, build lists, teams, and script manager matching C++ `SidesList`.
#[derive(Debug, Clone, Default)]
pub struct SidesList {
    pub sides: Vec<SidesInfo>,
    pub skirmish_sides: Vec<SidesInfo>,
    pub team_rec: TeamsInfoRec,
    pub skirmish_team_rec: TeamsInfoRec,
}

impl SidesList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self) {}
    pub fn update(&mut self) {}

    pub fn reset(&mut self) {
        self.clear();
    }

    pub fn clear(&mut self) {
        self.empty_sides();
        self.empty_teams();
    }

    #[inline]
    pub fn num_sides(&self) -> Int {
        self.sides.len() as Int
    }

    #[inline]
    pub fn num_skirmish_sides(&self) -> Int {
        self.skirmish_sides.len() as Int
    }

    #[inline]
    pub fn num_teams(&self) -> Int {
        self.team_rec.num_teams()
    }

    #[inline]
    pub fn num_skirmish_teams(&self) -> Int {
        self.skirmish_team_rec.num_teams()
    }

    pub fn empty_sides(&mut self) {
        self.sides.clear();
        self.skirmish_sides.clear();
    }

    pub fn add_side(&mut self, d: Option<&Dict>) {
        if self.sides.len() >= MAX_PLAYER_COUNT {
            return;
        }
        let mut side = SidesInfo::new();
        side.init(d);
        self.sides.push(side);
    }

    pub fn remove_side(&mut self, i: Int) {
        if i < 0 || i as usize >= self.sides.len() || self.sides.len() <= 1 {
            return;
        }
        self.sides.remove(i as usize);
    }

    pub fn side_info(&self, side: Int) -> Result<&SidesInfo, SidesError> {
        if side < 0 {
            return Err(SidesError::BadArg);
        }
        self.sides.get(side as usize).ok_or(SidesError::BadArg)
    }

    pub fn side_info_mut(&mut self, side: Int) -> Result<&mut SidesInfo, SidesError> {
        if side < 0 {
            return Err(SidesError::BadArg);
        }
        self.sides.get_mut(side as usize).ok_or(SidesError::BadArg)
    }

    pub fn skirmish_side_info(&self, side: Int) -> Result<&SidesInfo, SidesError> {
        if side < 0 {
            return Err(SidesError::BadArg);
        }
        self.skirmish_sides.get(side as usize).ok_or(SidesError::BadArg)
    }

    pub fn skirmish_side_info_mut(&mut self, side: Int) -> Result<&mut SidesInfo, SidesError> {
        if side < 0 {
            return Err(SidesError::BadArg);
        }
        self.skirmish_sides.get_mut(side as usize).ok_or(SidesError::BadArg)
    }

    pub fn add_skirmish_side(&mut self, d: Option<&Dict>) {
        if self.skirmish_sides.len() >= MAX_SKIRMISH_SIDES {
            return;
        }
        let mut side = SidesInfo::new();
        side.init(d);
        self.skirmish_sides.push(side);
    }

    pub fn find_side_info(&self, name: &str, keys: &WellKnownSidesKeys) -> Option<Int> {
        for (i, side) in self.sides.iter().enumerate() {
            if side.dict.get_ascii_string(keys.player_name, None) == name {
                return Some(i as Int);
            }
        }
        None
    }

    pub fn find_skirmish_side_info(&self, name: &str, keys: &WellKnownSidesKeys) -> Option<Int> {
        for (i, side) in self.skirmish_sides.iter().enumerate() {
            if side.dict.get_ascii_string(keys.player_name, None) == name {
                return Some(i as Int);
            }
        }
        None
    }

    pub fn empty_teams(&mut self) {
        self.team_rec.clear();
        self.skirmish_team_rec.clear();
    }

    pub fn add_team(&mut self, d: Option<&Dict>) {
        self.team_rec.add_team(d);
    }

    pub fn remove_team(&mut self, i: Int) {
        self.team_rec.remove_team(i);
    }

    pub fn team_info(&self, team: Int) -> Result<&TeamsInfo, SidesError> {
        self.team_rec.team_info(team)
    }

    pub fn team_info_mut(&mut self, team: Int) -> Result<&mut TeamsInfo, SidesError> {
        self.team_rec.team_info_mut(team)
    }

    pub fn skirmish_team_info(&self, team: Int) -> Result<&TeamsInfo, SidesError> {
        self.skirmish_team_rec.team_info(team)
    }

    pub fn add_skirmish_team(&mut self, d: Option<&Dict>) {
        self.skirmish_team_rec.add_team(d);
    }

    pub fn find_team_info(&self, name: &str, keys: &WellKnownSidesKeys) -> Option<Int> {
        self.team_rec.find_team_info(name, keys)
    }

    pub fn is_player_default_team(&self, team: &TeamsInfo, keys: &WellKnownSidesKeys) -> Bool {
        let team_name = team.dict.get_ascii_string(keys.team_name, None);
        if team_name.is_empty() {
            return FALSE;
        }
        if team_name.starts_with("team") {
            let pname = &team_name[4..];
            if team.dict.get_ascii_string(keys.team_owner, None) == pname
                && team.dict.get_bool(keys.team_is_singleton, None) == TRUE
            {
                return TRUE;
            }
        }
        FALSE
    }

    pub fn add_player_by_template(
        &mut self,
        player_template_name: &str,
        keys: &WellKnownSidesKeys,
    ) {
        let (player_name, display_name, is_human) = if player_template_name.is_empty() {
            (String::new(), "Neutral".to_owned(), FALSE)
        } else {
            let suffix = if let Some(stripped) = player_template_name.strip_prefix("Faction") {
                stripped
            } else {
                player_template_name
            };
            let pname = if suffix == player_template_name {
                format!("Plyr{suffix}")
            } else {
                format!("Plyr{suffix}")
            };
            let human = if pname == "PlyrCivilian" { FALSE } else { TRUE };
            (pname.clone(), pname, human)
        };

        let mut side_dict = Dict::new();
        side_dict.set_ascii_string(keys.player_name, &player_name);
        side_dict.set_bool(keys.player_is_human, is_human);
        let mut display_units = Vec::with_capacity(display_name.len());
        for b in display_name.as_bytes() {
            display_units.push(*b as u16);
        }
        side_dict.set_unicode_string(keys.player_display_name, &display_units);
        side_dict.set_ascii_string(keys.player_faction, player_template_name);
        side_dict.set_ascii_string(keys.player_allies, "");
        side_dict.set_ascii_string(keys.player_enemies, "");
        self.add_side(Some(&side_dict));

        let mut team_dict = Dict::new();
        let team_name = format!("team{player_name}");
        team_dict.set_ascii_string(keys.team_name, &team_name);
        team_dict.set_ascii_string(keys.team_owner, &player_name);
        team_dict.set_bool(keys.team_is_singleton, TRUE);
        self.add_team(Some(&team_dict));
    }

    pub fn validate_sides(&mut self, keys: &WellKnownSidesKeys) -> Bool {
        let mut modified = FALSE;

        let has_neutral = self.sides.iter().any(|side| {
            side.dict.get_ascii_string(keys.player_name, None).is_empty()
        });
        if !has_neutral {
            self.add_player_by_template("", keys);
            modified = TRUE;
        }

        for i in 0..self.num_sides() {
            let pname = self
                .side_info(i)
                .unwrap()
                .dict
                .get_ascii_string(keys.player_name, None);
            let team_name = format!("team{pname}");
            match self.team_rec.find_team_info(&team_name, keys) {
                Some(team_idx) => {
                    let team = self.team_rec.team_info_mut(team_idx).unwrap();
                    if team.dict.get_ascii_string(keys.team_owner, None) != pname {
                        team.dict.set_ascii_string(keys.team_owner, &pname);
                        modified = TRUE;
                    }
                    if team.dict.get_bool(keys.team_is_singleton, None) != TRUE {
                        team.dict.set_bool(keys.team_is_singleton, TRUE);
                        modified = TRUE;
                    }
                }
                None => {
                    let mut team = Dict::new();
                    team.set_ascii_string(keys.team_name, &team_name);
                    team.set_ascii_string(keys.team_owner, &pname);
                    team.set_bool(keys.team_is_singleton, TRUE);
                    self.add_team(Some(&team));
                    modified = TRUE;
                }
            }

            let allies = self.side_info(i).unwrap().dict.get_ascii_string(keys.player_allies, None);
            let enemies = self.side_info(i).unwrap().dict.get_ascii_string(keys.player_enemies, None);
            let prune = |value: &str| {
                let mut kept: Vec<&str> = Vec::new();
                for token in value.split(|c| c == ' ' || c == '\n' || c == '\r' || c == '\t') {
                    if token.is_empty() || token == pname.as_str() {
                        continue;
                    }
                    if self.find_side_info(token, keys).is_some() {
                        kept.push(token);
                    }
                }
                kept.join(" ")
            };
            let allies = prune(&allies);
            let enemies = prune(&enemies);
            if allies != self.side_info(i).unwrap().dict.get_ascii_string(keys.player_allies, None) {
                self.side_info_mut(i).unwrap().dict.set_ascii_string(keys.player_allies, &allies);
                modified = TRUE;
            }
            if enemies != self.side_info(i).unwrap().dict.get_ascii_string(keys.player_enemies, None) {
                self.side_info_mut(i).unwrap().dict.set_ascii_string(keys.player_enemies, &enemies);
                modified = TRUE;
            }
        }

        loop {
            let mut remove_idx = None;
            for i in 0..self.num_teams() {
                let tname = self.team_info(i).unwrap().dict.get_ascii_string(keys.team_name, None);
                if self.find_side_info(&tname, keys).is_some() {
                    remove_idx = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_idx {
                self.team_rec.remove_team(i);
                modified = TRUE;
            } else {
                break;
            }
        }

        for i in 0..self.num_teams() {
            let (tname, owner) = {
                let team = self.team_info(i).unwrap();
                (
                    team.dict.get_ascii_string(keys.team_name, None),
                    team.dict.get_ascii_string(keys.team_owner, None),
                )
            };
            if self.find_side_info(&owner, keys).is_none() || owner == tname {
                self.team_info_mut(i).unwrap().dict.set_ascii_string(keys.team_owner, "");
                modified = TRUE;
            }
        }

        modified
    }

    pub fn prepare_for_mp_or_skirmish(
        &mut self,
        skirmish_scripts_scb: Option<&[u8]>,
        keys: &WellKnownSidesKeys,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), SidesError> {
        self.skirmish_team_rec.clear();
        for i in 0..self.num_teams() {
            let dict = self.team_info(i).unwrap().dict.clone();
            self.skirmish_team_rec.add_team(Some(&dict));
        }
        self.team_rec.clear();

        self.skirmish_sides.clear();
        for side in &self.sides {
            self.skirmish_sides.push(side.clone());
        }

        loop {
            let mut remove_idx = None;
            for i in 0..self.num_sides() {
                let faction = self.side_info(i).unwrap().dict
                    .get_ascii_string(keys.player_faction, None);
                if faction != "FactionCivilian" && self.num_sides() > 1 {
                    remove_idx = Some(i);
                    break;
                }
            }
            if let Some(i) = remove_idx {
                self.remove_side(i);
            } else {
                break;
            }
        }

        let mut got_scripts = FALSE;
        for side in &self.skirmish_sides {
            let faction = side.dict.get_ascii_string(keys.player_faction, None);
            if faction != "FactionCivilian" {
                if let Some(script) = &side.scripts {
                    if !script.is_empty() {
                        got_scripts = TRUE;
                    }
                }
            }
        }

        if got_scripts == FALSE {
            self.skirmish_team_rec.clear();
            if let Some(bytes) = skirmish_scripts_scb {
                let mut stream = zh_common::io::data_chunk::CachedFileInputStream::from_bytes(bytes);
                let mut input: DataChunkInput<'_, ()> = DataChunkInput::new(&mut stream);
                if !input.is_valid_file_type() {
                    return Err(SidesError::CorruptFormat);
                }
                input.reset()?;
                let mut script_names: Vec<String> = Vec::new();
                let mut script_slots: Vec<Option<SidesScriptList>> = Vec::new();
                while !input.at_end_of_file() {
                    let (label, version) = input.open_data_chunk()?;
                    match label.as_str() {
                        "ScriptsPlayers" => {
                            let read_dicts = if version >= PLAYERS_NAMES_FOR_SCRIPTS_VERSION_2 {
                                input.read_int()? != 0
                            } else {
                                false
                            };
                            let count = input.read_int()?.min(MAX_SKIRMISH_SIDES as Int);
                            for i in 0..count {
                                let player_name = input.read_ascii_string()?;
                                if i < MAX_SKIRMISH_SIDES as Int {
                                    script_names.push(player_name.clone());
                                }
                                if read_dicts {
                                    let mut dict = input.read_dict(name_keys)?;
                                    if !player_name.is_empty()
                                        && self.find_skirmish_side_info(&player_name, keys).is_none()
                                    {
                                        if dict.get_ascii_string(keys.player_name, None).is_empty() {
                                            dict.set_ascii_string(keys.player_name, &player_name);
                                        }
                                        self.add_skirmish_side(Some(&dict));
                                    }
                                }
                            }
                        }
                        "ScriptTeams" => {
                            while !input.at_end_of_chunk() {
                                let dict = input.read_dict(name_keys)?;
                                let owner = dict.get_ascii_string(keys.team_owner, None);
                                if self.find_skirmish_side_info(&owner, keys).is_some() {
                                    self.skirmish_team_rec.add_team(Some(&dict));
                                }
                            }
                        }
                        "PlayerScriptsList" => {
                            let info = DataChunkInfo {
                                label: label.clone(),
                                parent_label: String::new(),
                                version,
                                data_size: input.get_chunk_data_size().unwrap_or(0) as Int,
                            };
                            script_slots = SidesScriptList::parse_scripts_data_chunk(&mut input, &info)?;
                        }
                        _ => {}
                    }
                    input.close_data_chunk()?;
                }
                for (i, slot) in script_slots.iter().enumerate() {
                    if let Some(name) = script_names.get(i) {
                        if let Some(side_idx) = self.find_skirmish_side_info(name, keys) {
                            self.skirmish_side_info_mut(side_idx).unwrap().scripts = slot.clone();
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn write_sides_data_chunk<O: DataChunkOutputStream>(
        &mut self,
        chunk_writer: &mut DataChunkOutput<'_, O>,
        keys: &WellKnownSidesKeys,
        name_keys: &NameKeyGenerator,
    ) -> Result<(), SidesError> {
        chunk_writer.open_data_chunk("SidesList", SIDES_DATA_VERSION_3)?;
        chunk_writer.write_int(self.num_sides())?;
        for i in 0..self.num_sides() {
            let side = self.side_info(i).unwrap();
            chunk_writer.write_dict(&side.dict, name_keys)?;
            chunk_writer.write_int(side.build_list.len() as Int)?;
            for build in &side.build_list {
                chunk_writer.write_ascii_string(&build.building_name)?;
                chunk_writer.write_ascii_string(&build.template_name)?;
                chunk_writer.write_real(build.location.x)?;
                chunk_writer.write_real(build.location.y)?;
                chunk_writer.write_real(build.location.z)?;
                chunk_writer.write_real(build.angle)?;
                chunk_writer.write_byte(build.is_initially_built as i8)?;
                chunk_writer.write_int(build.num_rebuilds as Int)?;
                chunk_writer.write_ascii_string(&build.script)?;
                chunk_writer.write_int(build.health)?;
                chunk_writer.write_byte(build.whiner as i8)?;
                chunk_writer.write_byte(build.unsellable as i8)?;
                chunk_writer.write_byte(build.repairable as i8)?;
            }
        }
        chunk_writer.write_int(self.num_teams())?;
        for i in 0..self.num_teams() {
            let team = self.team_info(i).unwrap();
            chunk_writer.write_dict(&team.dict, name_keys)?;
        }
        let script_slots: Vec<Option<&SidesScriptList>> = (0..self.num_sides())
            .map(|i| self.side_info(i).unwrap().scripts.as_ref())
            .collect();
        SidesScriptList::write_scripts_data_chunk(chunk_writer, &script_slots)?;
        chunk_writer.close_data_chunk()?;
        self.validate_sides(keys);
        Ok(())
    }

    /// The sides, build lists and teams of an open `SidesList` chunk. The script
    /// lists that follow, and `validateSides`, are left to the caller.
    pub fn read_sides_and_teams<Ctx>(
        &mut self,
        input: &mut DataChunkInput<'_, Ctx>,
        version: DataChunkVersionType,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), SidesError> {
        let side_count = input.read_int()?.max(0);
        self.clear();
        for i in 0..side_count {
            let dict = input.read_dict(name_keys)?;
            let mut side = SidesInfo::new();
            if i < MAX_PLAYER_COUNT as Int {
                side.init(Some(&dict));
                let build_count = input.read_int()?.max(0);
                for _ in 0..build_count {
                    let mut build = BuildListInfo::new();
                    build.building_name = input.read_ascii_string()?;
                    build.template_name = input.read_ascii_string()?;
                    build.location.x = input.read_real()?;
                    build.location.y = input.read_real()?;
                    build.location.z = input.read_real()?;
                    build.location.z = 0.0;
                    build.angle = input.read_real()?;
                    build.is_initially_built = if input.read_byte()? != 0 { TRUE } else { FALSE };
                    let rebuilds = input.read_int()?;
                    build.num_rebuilds = rebuilds as UnsignedInt;
                    if version >= SIDES_DATA_VERSION_3 {
                        build.script = input.read_ascii_string()?;
                        build.health = input.read_int()?;
                        build.whiner = if input.read_byte()? != 0 { TRUE } else { FALSE };
                        build.unsellable = if input.read_byte()? != 0 { TRUE } else { FALSE };
                        build.repairable = if input.read_byte()? != 0 { TRUE } else { FALSE };
                    }
                    side.build_list.push(build);
                }
                self.sides.push(side);
            } else {
                let build_count = input.read_int()?.max(0);
                for _ in 0..build_count {
                    let _ = input.read_ascii_string()?;
                    let _ = input.read_ascii_string()?;
                    let _ = input.read_real()?;
                    let _ = input.read_real()?;
                    let _ = input.read_real()?;
                    let _ = input.read_real()?;
                    let _ = input.read_byte()?;
                    let _ = input.read_int()?;
                    if version >= SIDES_DATA_VERSION_3 {
                        let _ = input.read_ascii_string()?;
                        let _ = input.read_int()?;
                        let _ = input.read_byte()?;
                        let _ = input.read_byte()?;
                        let _ = input.read_byte()?;
                    }
                }
            }
        }

        if version >= SIDES_DATA_VERSION_2 {
            let team_count = input.read_int()?.max(0);
            for _ in 0..team_count {
                let dict = input.read_dict(name_keys)?;
                self.team_rec.add_team(Some(&dict));
            }
        }
        Ok(())
    }

    pub fn parse_sides_chunk_stream(
        &mut self,
        stream: &mut dyn ChunkInputStream,
        keys: &WellKnownSidesKeys,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), SidesError> {
        let mut input: DataChunkInput<'_, ()> = DataChunkInput::new(stream);
        if !input.is_valid_file_type() {
            return Err(SidesError::CorruptFormat);
        }
        input.reset()?;
        let (label, version) = input.open_data_chunk()?;
        if label != "SidesList" {
            return Err(SidesError::CorruptFormat);
        }
        self.read_sides_and_teams(&mut input, version, name_keys)?;

        if !input.at_end_of_chunk() {
            let (label, script_version) = input.open_data_chunk()?;
            if label == "PlayerScriptsList" {
                let info = DataChunkInfo {
                    label,
                    parent_label: "SidesList".to_owned(),
                    version: script_version,
                    data_size: input.get_chunk_data_size().unwrap_or(0) as Int,
                };
                let scripts = SidesScriptList::parse_scripts_data_chunk(&mut input, &info)?;
                for (i, script) in scripts.into_iter().enumerate() {
                    if i < self.sides.len() {
                        self.sides[i].scripts = script;
                    }
                }
            }
            input.close_data_chunk()?;
        }
        input.close_data_chunk()?;
        self.validate_sides(keys);
        Ok(())
    }
}

impl Snapshot for SidesList {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut side_count = self.num_sides();
        xfer.xfer_int(&mut side_count)?;
        match xfer.mode() {
            XferMode::Load => {
                if side_count != self.num_sides() {
                    return Err(XferError::InvalidData);
                }
            }
            XferMode::Save | XferMode::Crc => {
                if side_count != self.num_sides() {
                    return Err(XferError::InvalidData);
                }
            }
        }
        for i in 0..side_count {
            let side = self.side_info_mut(i).map_err(|_| XferError::InvalidData)?;
            let mut script_list_present = if side.scripts.is_some() { TRUE } else { FALSE };
            xfer.xfer_bool(&mut script_list_present)?;
            if script_list_present != FALSE {
                let mut script = side.scripts.clone().unwrap_or_default();
                script.xfer(xfer)?;
                side.scripts = Some(script);
            } else {
                side.scripts = None;
            }
        }
        Ok(())
    }

    fn load_post_process(&mut self) {}
}
