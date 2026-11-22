//! `PKT-406`: deterministic radar event queue and coordinate transforms.

use zh_common::math::{Coord3D, ICoord2D, RGBAColorInt, Region3D};
use zh_common::types::{Bool, Int, UnsignedInt, UnsignedShort, XferVersion, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const RADAR_CELL_WIDTH: usize = 128;
pub const RADAR_CELL_HEIGHT: usize = 128;
pub const MAX_RADAR_EVENTS: usize = 64;
pub const MAX_PLAYER_COUNT: usize = 16;
pub const RADAR_QUEUE_TERRAIN_REFRESH_DELAY: f32 = 90.0;
pub const LOGIC_FRAMES_PER_SECOND: UnsignedInt = 30;

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RadarEventType {
    #[default]
    Invalid = 0,
    Construction = 1,
    Upgrade = 2,
    UnderAttack = 3,
    Information = 4,
    BeaconPulse = 5,
    Infiltration = 6,
    BattlePlan = 7,
    StealthDiscovered = 8,
    StealthNeutralized = 9,
    Fake = 10,
}

impl RadarEventType {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::Construction),
            2 => Some(Self::Upgrade),
            3 => Some(Self::UnderAttack),
            4 => Some(Self::Information),
            5 => Some(Self::BeaconPulse),
            6 => Some(Self::Infiltration),
            7 => Some(Self::BattlePlan),
            8 => Some(Self::StealthDiscovered),
            9 => Some(Self::StealthNeutralized),
            10 => Some(Self::Fake),
            _ => None,
        }
    }
    pub const fn as_i32(self) -> i32 {
        self as i32
    }
}

#[repr(i32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum RadarPriorityType {
    #[default]
    Invalid = 0,
    NotOnRadar = 1,
    Structure = 2,
    Unit = 3,
    LocalUnitOnly = 4,
}

impl RadarPriorityType {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            0 => Some(Self::Invalid),
            1 => Some(Self::NotOnRadar),
            2 => Some(Self::Structure),
            3 => Some(Self::Unit),
            4 => Some(Self::LocalUnitOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadarObject {
    object_id: i32,
    priority: RadarPriorityType,
    color: i32,
}

impl Default for RadarObject {
    fn default() -> Self {
        Self {
            object_id: 0,
            priority: RadarPriorityType::Invalid,
            color: -1,
        }
    }
}

impl RadarObject {
    pub fn new(object_id: i32, priority: RadarPriorityType, color: i32) -> Self {
        Self {
            object_id,
            priority,
            color,
        }
    }
    pub fn object_id(&self) -> i32 {
        self.object_id
    }
    pub fn priority(&self) -> RadarPriorityType {
        self.priority
    }
    pub fn color(&self) -> i32 {
        self.color
    }
}

impl Snapshot for RadarObject {
    fn crc(&self, _xfer: &mut dyn Xfer) {}
    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        xfer.xfer_int(&mut self.object_id)?;
        xfer.xfer_int(&mut self.color)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadarEvent {
    pub event_type: RadarEventType,
    pub active: Bool,
    pub create_frame: UnsignedInt,
    pub die_frame: UnsignedInt,
    pub fade_frame: UnsignedInt,
    pub color1: RGBAColorInt,
    pub color2: RGBAColorInt,
    pub world_loc: Coord3D,
    pub radar_loc: ICoord2D,
    pub sound_played: Bool,
}

impl Default for RadarEvent {
    fn default() -> Self {
        Self {
            event_type: RadarEventType::Invalid,
            active: FALSE,
            create_frame: 0,
            die_frame: 0,
            fade_frame: 0,
            color1: RGBAColorInt {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            },
            color2: RGBAColorInt {
                red: 0,
                green: 0,
                blue: 0,
                alpha: 0,
            },
            world_loc: Coord3D::default(),
            radar_loc: ICoord2D::new(0, 0),
            sound_played: FALSE,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Radar {
    m_radar_hidden: [Bool; MAX_PLAYER_COUNT],
    m_radar_force_on: [Bool; MAX_PLAYER_COUNT],
    m_local_object_list: Vec<RadarObject>,
    m_object_list: Vec<RadarObject>,
    m_x_sample: f32,
    m_y_sample: f32,
    pub m_map_extent: Region3D,
    m_event: [RadarEvent; MAX_RADAR_EVENTS],
    m_next_free_radar_event: Int,
    m_last_radar_event: Int,
    m_queue_terrain_refresh_frame: UnsignedInt,
    m_frame: UnsignedInt,
    m_local_player_index: i32,
}

impl Default for Radar {
    fn default() -> Self {
        Self::new()
    }
}

impl Radar {
    pub fn new() -> Self {
        let mut out = Self {
            m_radar_hidden: [FALSE; MAX_PLAYER_COUNT],
            m_radar_force_on: [FALSE; MAX_PLAYER_COUNT],
            m_local_object_list: Vec::new(),
            m_object_list: Vec::new(),
            m_x_sample: 0.0,
            m_y_sample: 0.0,
            m_map_extent: Region3D {
                lo: Coord3D::default(),
                hi: Coord3D::default(),
            },
            m_event: [RadarEvent::default(); MAX_RADAR_EVENTS],
            m_next_free_radar_event: 0,
            m_last_radar_event: -1,
            m_queue_terrain_refresh_frame: 0,
            m_frame: 0,
            m_local_player_index: 0,
        };
        out.clear_all_events();
        out
    }

    pub fn configure_extent(&mut self, ext: &Region3D) {
        self.m_map_extent = *ext;
        self.m_x_sample = ext.width() / (RADAR_CELL_WIDTH as f32);
        self.m_y_sample = ext.height() / (RADAR_CELL_HEIGHT as f32);
    }

    pub fn set_frame(&mut self, frame: UnsignedInt) {
        self.m_frame = frame;
    }
    pub fn frame(&self) -> UnsignedInt {
        self.m_frame
    }
    pub fn set_local_player_index(&mut self, player_index: i32) {
        if (0..MAX_PLAYER_COUNT as i32).contains(&player_index) {
            self.m_local_player_index = player_index;
        }
    }

    pub fn world_to_radar(&self, world: &Coord3D, radar: &mut ICoord2D) -> Bool {
        radar.x = (world.x / self.m_x_sample) as i32;
        radar.y = (world.y / self.m_y_sample) as i32;
        radar.x = radar.x.clamp(0, (RADAR_CELL_WIDTH - 1) as i32);
        radar.y = radar.y.clamp(0, (RADAR_CELL_HEIGHT - 1) as i32);
        TRUE
    }

    pub fn radar_to_world_2d(&self, radar: &ICoord2D, world: &mut Coord3D) -> Bool {
        let x = radar.x.clamp(0, (RADAR_CELL_WIDTH - 1) as i32);
        let y = radar.y.clamp(0, (RADAR_CELL_HEIGHT - 1) as i32);
        world.x = (x as f32) * self.m_x_sample;
        world.y = (y as f32) * self.m_y_sample;
        TRUE
    }

    pub fn find_draw_positions(
        &self,
        start_x: i32,
        start_y: i32,
        width: i32,
        height: i32,
    ) -> (ICoord2D, ICoord2D) {
        let ratio_width = self.m_map_extent.width() / (width as f32 * 1.0);
        let ratio_height = self.m_map_extent.height() / (height as f32 * 1.0);
        let mut ul = ICoord2D::new(0, 0);
        let mut lr = ICoord2D::new(0, 0);
        if ratio_width >= ratio_height {
            let radar_x = self.m_map_extent.width() / ratio_width;
            let radar_y = self.m_map_extent.height() / ratio_width;
            ul.x = 0;
            ul.y = ((height as f32 - radar_y) / 2.0) as i32;
            lr.x = radar_x as i32;
            lr.y = height - ul.y;
        } else {
            let radar_x = self.m_map_extent.width() / ratio_height;
            let radar_y = self.m_map_extent.height() / ratio_height;
            ul.x = ((width as f32 - radar_x) / 2.0) as i32;
            ul.y = 0;
            lr.x = width - ul.x;
            lr.y = radar_y as i32;
        }
        ul.x += start_x;
        ul.y += start_y;
        lr.x += start_x;
        lr.y += start_y;
        (ul, lr)
    }

    pub fn is_priority_visible(priority: RadarPriorityType) -> Bool {
        if matches!(
            priority,
            RadarPriorityType::Invalid | RadarPriorityType::NotOnRadar
        ) {
            FALSE
        } else {
            TRUE
        }
    }

    pub fn hide(&mut self, player_index: i32, hide: Bool) {
        if (0..MAX_PLAYER_COUNT as i32).contains(&player_index) {
            self.m_radar_hidden[player_index as usize] = hide;
        }
    }
    pub fn is_radar_hidden(&self, player_index: i32) -> Bool {
        if (0..MAX_PLAYER_COUNT as i32).contains(&player_index) {
            self.m_radar_hidden[player_index as usize]
        } else {
            FALSE
        }
    }
    pub fn force_on(&mut self, player_index: i32, force: Bool) {
        if (0..MAX_PLAYER_COUNT as i32).contains(&player_index) {
            self.m_radar_force_on[player_index as usize] = force;
        }
    }
    pub fn is_radar_forced(&self, player_index: i32) -> Bool {
        if (0..MAX_PLAYER_COUNT as i32).contains(&player_index) {
            self.m_radar_force_on[player_index as usize]
        } else {
            FALSE
        }
    }

    fn event_colors(event_type: RadarEventType) -> (RGBAColorInt, RGBAColorInt) {
        match event_type {
            RadarEventType::Construction => (
                RGBAColorInt {
                    red: 128,
                    green: 128,
                    blue: 255,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 128,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
            ),
            RadarEventType::Upgrade => (
                RGBAColorInt {
                    red: 128,
                    green: 0,
                    blue: 64,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 255,
                    green: 185,
                    blue: 220,
                    alpha: 255,
                },
            ),
            RadarEventType::UnderAttack => (
                RGBAColorInt {
                    red: 255,
                    green: 0,
                    blue: 0,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 255,
                    green: 128,
                    blue: 128,
                    alpha: 255,
                },
            ),
            RadarEventType::Information => (
                RGBAColorInt {
                    red: 255,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 255,
                    green: 255,
                    blue: 128,
                    alpha: 255,
                },
            ),
            RadarEventType::BeaconPulse => (
                RGBAColorInt {
                    red: 255,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 255,
                    green: 255,
                    blue: 128,
                    alpha: 255,
                },
            ),
            RadarEventType::Infiltration => (
                RGBAColorInt {
                    red: 0,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 128,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
            ),
            RadarEventType::BattlePlan => (
                RGBAColorInt {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 255,
                    green: 255,
                    blue: 255,
                    alpha: 255,
                },
            ),
            RadarEventType::StealthDiscovered => (
                RGBAColorInt {
                    red: 0,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 0,
                    green: 128,
                    blue: 0,
                    alpha: 255,
                },
            ),
            RadarEventType::StealthNeutralized => (
                RGBAColorInt {
                    red: 0,
                    green: 255,
                    blue: 0,
                    alpha: 255,
                },
                RGBAColorInt {
                    red: 0,
                    green: 128,
                    blue: 0,
                    alpha: 255,
                },
            ),
            RadarEventType::Fake => (
                RGBAColorInt {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
                RGBAColorInt {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
            ),
            RadarEventType::Invalid => (
                RGBAColorInt {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
                RGBAColorInt {
                    red: 0,
                    green: 0,
                    blue: 0,
                    alpha: 0,
                },
            ),
        }
    }

    fn internal_create_event(
        &mut self,
        world: &Coord3D,
        event_type: RadarEventType,
        seconds_to_live: f32,
        color1: RGBAColorInt,
        color2: RGBAColorInt,
    ) {
        let mut radar = ICoord2D::new(0, 0);
        self.world_to_radar(world, &mut radar);
        let frame = self.m_frame;
        let die_frame = ((frame as f32) + 30.0 * seconds_to_live) as u32;
        let fade_frame = ((die_frame as f32) - 30.0 * 0.5) as u32;
        let idx = self.m_next_free_radar_event as usize;
        self.m_event[idx] = RadarEvent {
            event_type,
            active: TRUE,
            create_frame: frame,
            die_frame,
            fade_frame,
            color1,
            color2,
            world_loc: *world,
            radar_loc: radar,
            sound_played: FALSE,
        };
        if event_type != RadarEventType::BeaconPulse {
            self.m_last_radar_event = idx as i32;
        }
        self.m_next_free_radar_event = ((idx + 1) % MAX_RADAR_EVENTS) as i32;
    }

    pub fn create_event(
        &mut self,
        world: &Coord3D,
        event_type: RadarEventType,
        seconds_to_live: f32,
    ) {
        let (color1, color2) = Self::event_colors(event_type);
        self.internal_create_event(world, event_type, seconds_to_live, color1, color2);
    }

    pub fn create_player_event(
        &mut self,
        player_color: i32,
        world: &Coord3D,
        event_type: RadarEventType,
        seconds_to_live: f32,
    ) {
        let r = ((player_color as u32 >> 16) & 0xff) as u32;
        let g = ((player_color as u32 >> 8) & 0xff) as u32;
        let b = (player_color as u32 & 0xff) as u32;
        let a = ((player_color as u32 >> 24) & 0xff) as u32;
        let color1 = RGBAColorInt {
            red: r,
            green: g,
            blue: b,
            alpha: a,
        };
        let dr = ((r as f32) * 0.75) as i32;
        let dg = ((g as f32) * 0.75) as i32;
        let db = ((b as f32) * 0.75) as i32;
        let r2 = r.wrapping_sub(dr as u32);
        let g2 = g.wrapping_sub(dg as u32);
        let b2 = b.wrapping_sub(db as u32);
        let color2 = RGBAColorInt {
            red: r2,
            green: g2,
            blue: b2,
            alpha: a,
        };
        self.internal_create_event(world, event_type, seconds_to_live, color1, color2);
    }

    pub fn try_event(&mut self, event_type: RadarEventType, pos: &Coord3D) -> Bool {
        if event_type == RadarEventType::Invalid {
            return FALSE;
        }
        if event_type.as_i32() >= 11 {
            return FALSE;
        }
        let current = self.m_frame;
        for ev in &self.m_event {
            if ev.event_type == event_type {
                let dx = ev.world_loc.x - pos.x;
                let dy = ev.world_loc.y - pos.y;
                let dist_sq = (dx * dx) + (dy * dy);
                let is_close =
                    dist_sq <= 250.0 * 250.0 || event_type == RadarEventType::UnderAttack;
                if is_close && current.wrapping_sub(ev.create_frame) < 300 {
                    return FALSE;
                }
            }
        }
        self.create_event(pos, event_type, 4.0);
        TRUE
    }

    pub fn get_last_event_loc(&self) -> Option<Coord3D> {
        if self.m_last_radar_event >= 0 {
            Some(self.m_event[self.m_last_radar_event as usize].world_loc)
        } else {
            None
        }
    }

    pub fn update(&mut self) {
        let this_frame = self.m_frame;
        for ev in &mut self.m_event {
            if ev.active == TRUE && ev.create_frame != 0 && this_frame > ev.die_frame {
                ev.active = FALSE;
            }
        }
        if self.m_queue_terrain_refresh_frame != 0
            && (this_frame.wrapping_sub(self.m_queue_terrain_refresh_frame) as f32)
                > RADAR_QUEUE_TERRAIN_REFRESH_DELAY
        {
            self.refresh_terrain();
        }
    }

    pub fn reset(&mut self) {
        self.m_local_object_list.clear();
        self.m_object_list.clear();
        self.clear_all_events();
        self.m_radar_force_on = [FALSE; MAX_PLAYER_COUNT];
    }

    pub fn clear_all_events(&mut self) {
        self.m_next_free_radar_event = 0;
        self.m_last_radar_event = -1;
        for ev in &mut self.m_event {
            ev.event_type = RadarEventType::Invalid;
            ev.active = FALSE;
            ev.create_frame = 0;
            ev.die_frame = 0;
            ev.fade_frame = 0;
            ev.color1.red = 0;
            ev.color1.green = 0;
            ev.color1.blue = 0;
            ev.color2.red = 0;
            ev.color2.green = 0;
            ev.color2.blue = 0;
            ev.world_loc = Coord3D::default();
            ev.radar_loc = ICoord2D::new(0, 0);
            ev.sound_played = FALSE;
        }
    }

    pub fn queue_terrain_refresh(&mut self) {
        self.m_queue_terrain_refresh_frame = self.m_frame;
    }
    pub fn refresh_terrain(&mut self) {
        self.m_queue_terrain_refresh_frame = 0;
    }
    pub fn queue_terrain_refresh_frame(&self) -> UnsignedInt {
        self.m_queue_terrain_refresh_frame
    }

    pub fn link_radar_object(&mut self, object: RadarObject, local: bool) {
        let list = if local {
            &mut self.m_local_object_list
        } else {
            &mut self.m_object_list
        };
        list.push(object);
        list.sort_by_key(|v| v.priority);
    }
    pub fn remove_radar_object(&mut self, object_id: i32) -> Bool {
        let before = self.m_local_object_list.len() + self.m_object_list.len();
        self.m_local_object_list
            .retain(|v| v.object_id != object_id);
        self.m_object_list.retain(|v| v.object_id != object_id);
        if before != self.m_local_object_list.len() + self.m_object_list.len() {
            TRUE
        } else {
            FALSE
        }
    }
    pub fn local_object_list(&self) -> &[RadarObject] {
        &self.m_local_object_list
    }
    pub fn object_list(&self) -> &[RadarObject] {
        &self.m_object_list
    }
    pub fn next_free_radar_event(&self) -> i32 {
        self.m_next_free_radar_event
    }
    pub fn last_radar_event(&self) -> i32 {
        self.m_last_radar_event
    }
    pub fn active_event_count(&self) -> usize {
        self.m_event.iter().filter(|e| e.active == TRUE).count()
    }
    pub fn event(&self, index: usize) -> Option<RadarEvent> {
        self.m_event.get(index).copied()
    }
}

impl Snapshot for Radar {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let current_version: XferVersion = 1;
        let mut version = current_version;
        xfer.xfer_version(&mut version, current_version)?;
        if version <= 1 {
            let mut hidden = self.m_radar_hidden[self.m_local_player_index as usize];
            xfer.xfer_bool(&mut hidden)?;
            self.m_radar_hidden[self.m_local_player_index as usize] = hidden;
            let mut force = self.m_radar_force_on[self.m_local_player_index as usize];
            xfer.xfer_bool(&mut force)?;
            self.m_radar_force_on[self.m_local_player_index as usize] = force;
        } else {
            let mut hidden_raw = [FALSE; MAX_PLAYER_COUNT];
            hidden_raw.copy_from_slice(&self.m_radar_hidden);
            xfer.xfer_user(&mut hidden_raw)?;
            for (i, &v) in hidden_raw.iter().enumerate() {
                self.m_radar_hidden[i] = v;
            }
            let mut force_raw = [FALSE; MAX_PLAYER_COUNT];
            force_raw.copy_from_slice(&self.m_radar_force_on);
            xfer.xfer_user(&mut force_raw)?;
            for (i, &v) in force_raw.iter().enumerate() {
                self.m_radar_force_on[i] = v;
            }
        }

        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut local_count =
            self.m_local_object_list.len().min(u16::MAX as usize) as UnsignedShort;
        xfer.xfer_unsigned_short(&mut local_count)?;
        match xfer.mode() {
            XferMode::Load => {
                self.m_local_object_list.clear();
                for _ in 0..local_count {
                    let mut object = RadarObject::default();
                    xfer.xfer_snapshot(&mut object)?;
                    self.m_local_object_list.push(object);
                }
            }
            XferMode::Save | XferMode::Crc => {
                for object in &mut self.m_local_object_list[..local_count as usize] {
                    xfer.xfer_snapshot(object)?;
                }
            }
        }

        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        let mut object_count = self.m_object_list.len().min(u16::MAX as usize) as UnsignedShort;
        xfer.xfer_unsigned_short(&mut object_count)?;
        match xfer.mode() {
            XferMode::Load => {
                self.m_object_list.clear();
                for _ in 0..object_count {
                    let mut object = RadarObject::default();
                    xfer.xfer_snapshot(&mut object)?;
                    self.m_object_list.push(object);
                }
            }
            XferMode::Save | XferMode::Crc => {
                for object in &mut self.m_object_list[..object_count as usize] {
                    xfer.xfer_snapshot(object)?;
                }
            }
        }

        let mut event_count: UnsignedShort = MAX_RADAR_EVENTS as UnsignedShort;
        xfer.xfer_unsigned_short(&mut event_count)?;
        if event_count as usize != MAX_RADAR_EVENTS {
            return Err(XferError::InvalidData);
        }
        for ev in &mut self.m_event {
            let mut event_type = ev.event_type.as_i32();
            xfer.xfer_enum_i32(&mut event_type)?;
            ev.event_type = RadarEventType::from_i32(event_type).unwrap_or(RadarEventType::Invalid);
            xfer.xfer_bool(&mut ev.active)?;
            xfer.xfer_unsigned_int(&mut ev.create_frame)?;
            xfer.xfer_unsigned_int(&mut ev.die_frame)?;
            xfer.xfer_unsigned_int(&mut ev.fade_frame)?;
            xfer.xfer_rgba_color_int(&mut ev.color1)?;
            xfer.xfer_rgba_color_int(&mut ev.color2)?;
            xfer.xfer_coord3d(&mut ev.world_loc)?;
            xfer.xfer_icoord2d(&mut ev.radar_loc)?;
            xfer.xfer_bool(&mut ev.sound_played)?;
        }
        xfer.xfer_int(&mut self.m_next_free_radar_event)?;
        xfer.xfer_int(&mut self.m_last_radar_event)
    }
}
