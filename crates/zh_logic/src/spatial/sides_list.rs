//! `PKT-154`: Map player sides, teams, build lists, skirmish scripts, and DataChunk/Xfer
//! serialization (`Core/GameEngine/Include/GameLogic/SidesList.h` &
//! `Core/GameEngine/Source/GameLogic/Map/SidesList.cpp`).
//!
//! Reference packet: `packets/phase4/PKT-154_SidesList.md`
//! Golden vectors: `crates/zh_logic/tests/golden_pkt154_sides_list.rs` (`docs/golden_vectors/PKT-154.json`)

#![allow(unused_variables, dead_code)]

use zh_common::core::dict::Dict;
use zh_common::core::name_key_generator::NameKeyGenerator;
use zh_common::ini::{IniError, IniReader};
use zh_common::io::data_chunk::{
    ChunkInputStream, DataChunkError, DataChunkInfo, DataChunkInput, DataChunkOutput,
    DataChunkOutputStream, DataChunkVersionType,
};
use zh_common::math::{Coord2D, Coord3D};
use zh_common::types::{
    Bool, Int, NameKeyType, ObjectId, Real, UnsignedInt, XferVersion, FALSE, TRUE,
};
use zh_common::xfer::{Snapshot, Xfer, XferError};

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
        todo!("PKT-154")
    }

    pub fn increment_num_rebuilds(&mut self) {
        todo!("PKT-154")
    }

    pub fn is_buildable(&self) -> Bool {
        todo!("PKT-154")
    }

    pub fn get_gatherer_id(&self, ndx: Int) -> ObjectId {
        todo!("PKT-154")
    }

    pub fn set_gatherer_id(&mut self, ndx: Int, id: ObjectId) {
        todo!("PKT-154")
    }

    /// Parses one `Structure`/`AIData` build list entry via [`IniReader`].
    pub fn parse_structure(ini: &mut IniReader<'_>) -> Result<Self, IniError> {
        todo!("PKT-154")
    }
}

impl Snapshot for BuildListInfo {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        todo!("PKT-154")
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
        todo!("PKT-154")
    }

    pub fn clear(&mut self) {
        self.init(None);
    }

    pub fn add_to_build_list(&mut self, item: BuildListInfo, position: Int) {
        todo!("PKT-154")
    }

    pub fn remove_from_build_list(&mut self, index: usize) -> Int {
        todo!("PKT-154")
    }

    pub fn reorder_in_build_list(&mut self, index: usize, new_position: Int) {
        todo!("PKT-154")
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
        todo!("PKT-154")
    }

    pub fn find_team_info(
        &self,
        name: &str,
        keys: &WellKnownSidesKeys,
    ) -> Option<Int> {
        todo!("PKT-154")
    }

    pub fn add_team(&mut self, d: Option<&Dict>) {
        todo!("PKT-154")
    }

    pub fn remove_team(&mut self, i: Int) {
        todo!("PKT-154")
    }

    #[inline]
    pub fn num_teams(&self) -> Int {
        self.teams.len() as Int
    }

    pub fn team_info(&self, team: Int) -> Result<&TeamsInfo, SidesError> {
        todo!("PKT-154")
    }

    pub fn team_info_mut(&mut self, team: Int) -> Result<&mut TeamsInfo, SidesError> {
        todo!("PKT-154")
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
        todo!("PKT-154")
    }

    pub fn add_side(&mut self, d: Option<&Dict>) {
        todo!("PKT-154")
    }

    pub fn remove_side(&mut self, i: Int) {
        todo!("PKT-154")
    }

    pub fn side_info(&self, side: Int) -> Result<&SidesInfo, SidesError> {
        todo!("PKT-154")
    }

    pub fn side_info_mut(&mut self, side: Int) -> Result<&mut SidesInfo, SidesError> {
        todo!("PKT-154")
    }

    pub fn skirmish_side_info(&self, side: Int) -> Result<&SidesInfo, SidesError> {
        todo!("PKT-154")
    }

    pub fn skirmish_side_info_mut(&mut self, side: Int) -> Result<&mut SidesInfo, SidesError> {
        todo!("PKT-154")
    }

    pub fn add_skirmish_side(&mut self, d: Option<&Dict>) {
        todo!("PKT-154")
    }

    pub fn find_side_info(&self, name: &str, keys: &WellKnownSidesKeys) -> Option<Int> {
        todo!("PKT-154")
    }

    pub fn find_skirmish_side_info(&self, name: &str, keys: &WellKnownSidesKeys) -> Option<Int> {
        todo!("PKT-154")
    }

    pub fn empty_teams(&mut self) {
        todo!("PKT-154")
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
        todo!("PKT-154")
    }

    pub fn add_player_by_template(
        &mut self,
        player_template_name: &str,
        keys: &WellKnownSidesKeys,
    ) {
        todo!("PKT-154")
    }

    pub fn validate_sides(&mut self, keys: &WellKnownSidesKeys) -> Bool {
        todo!("PKT-154")
    }

    pub fn prepare_for_mp_or_skirmish(
        &mut self,
        skirmish_scripts_scb: Option<&[u8]>,
        keys: &WellKnownSidesKeys,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), SidesError> {
        todo!("PKT-154")
    }

    pub fn write_sides_data_chunk<O: DataChunkOutputStream>(
        &mut self,
        chunk_writer: &mut DataChunkOutput<'_, O>,
        keys: &WellKnownSidesKeys,
        name_keys: &NameKeyGenerator,
    ) -> Result<(), SidesError> {
        todo!("PKT-154")
    }

    pub fn parse_sides_chunk_stream(
        &mut self,
        stream: &mut dyn ChunkInputStream,
        keys: &WellKnownSidesKeys,
        name_keys: &mut NameKeyGenerator,
    ) -> Result<(), SidesError> {
        todo!("PKT-154")
    }
}

impl Snapshot for SidesList {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        todo!("PKT-154")
    }

    fn load_post_process(&mut self) {}
}
