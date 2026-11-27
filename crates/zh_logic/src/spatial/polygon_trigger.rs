//! `PKT-152`: Map polygon trigger & water areas (`PolygonTrigger.h` / `PolygonTrigger.cpp`).

use zh_common::io::data_chunk::{
    CachedFileInputStream, DataChunkError, DataChunkInfo, DataChunkInput, DataChunkOutput,
    DataChunkOutputStream, DataChunkVersionType,
};
use zh_common::math::{gm_sqrtf, Coord3D, ICoord2D, ICoord3D, IRegion2D};
use zh_common::types::{Bool, Int, Real, FALSE, TRUE};
use zh_common::xfer::{Snapshot, Xfer, XferError, XferMode};

pub const K_TRIGGERS_VERSION_1: DataChunkVersionType = 1;
pub const K_TRIGGERS_VERSION_2: DataChunkVersionType = 2;
pub const K_TRIGGERS_VERSION_3: DataChunkVersionType = 3;
pub const K_TRIGGERS_VERSION_4: DataChunkVersionType = 4;

pub const MAP_XY_FACTOR: Int = 10;
const BIG_INT: Int = 0x7f_fff0;

/// Water area handle returned when `PolygonTrigger::is_water_area != FALSE` (`WaterHandle`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaterHandle {
    pub polygon_id: Int,
}

/// Polygon trigger region or water body matching C++ `PolygonTrigger` (`PolygonTrigger.h`).
#[derive(Debug, Clone, PartialEq)]
pub struct PolygonTrigger {
    pub trigger_name: String,
    pub trigger_id: Int,
    pub points: Vec<ICoord3D>,
    pub bounds: IRegion2D,
    pub radius: Real,
    pub river_start: Int,
    pub bounds_needs_update: Bool,
    pub export_with_scripts: Bool,
    pub is_water_area: Bool,
    pub is_river: Bool,
    pub layer_name: String,
    pub should_render: Bool,
    pub selected: Bool,
    /// When `true`, replicates retail 1.04 `RETAIL_COMPATIBLE_CRC` bounds-height bug
    /// (`(hi.y + lo.y) / 2.0` instead of `(hi.y - lo.y) / 2.0`).
    pub retail_compatible_crc: bool,
}

impl Default for PolygonTrigger {
    fn default() -> Self {
        Self::new(0)
    }
}

impl PolygonTrigger {
    pub fn new(trigger_id: Int) -> Self {
        Self {
            trigger_name: String::new(),
            trigger_id,
            points: Vec::new(),
            bounds: IRegion2D::default(),
            radius: 0.0,
            river_start: 0,
            bounds_needs_update: FALSE,
            export_with_scripts: FALSE,
            is_water_area: FALSE,
            is_river: FALSE,
            layer_name: String::new(),
            should_render: TRUE,
            selected: FALSE,
            retail_compatible_crc: false,
        }
    }

    #[inline]
    pub fn id(&self) -> Int {
        self.trigger_id
    }

    #[inline]
    pub fn num_points(&self) -> Int {
        self.points.len() as Int
    }

    /// Returns vertex clamped to `[0, num_points - 1]` matching C++ `PolygonTrigger::getPoint`.
    pub fn get_point(&self, ndx: Int) -> Option<ICoord3D> {
        if self.points.is_empty() {
            return None;
        }
        let clamped = if ndx < 0 {
            0
        } else if (ndx as usize) >= self.points.len() {
            self.points.len() - 1
        } else {
            ndx as usize
        };
        Some(self.points[clamped])
    }

    pub fn add_point(&mut self, point: ICoord3D) {
        self.points.push(point);
        self.bounds_needs_update = TRUE;
    }

    pub fn set_point(&mut self, point: ICoord3D, ndx: Int) {
        if ndx < 0 {
            return;
        }
        let idx = ndx as usize;
        if idx == self.points.len() {
            self.add_point(point);
            return;
        }
        if idx > self.points.len() {
            return;
        }
        self.points[idx] = point;
        self.bounds_needs_update = TRUE;
    }

    pub fn insert_point(&mut self, point: ICoord3D, ndx: Int) {
        if ndx < 0 {
            return;
        }
        let idx = ndx as usize;
        if idx == self.points.len() {
            self.add_point(point);
            return;
        }
        if idx > self.points.len() {
            return;
        }
        self.points.insert(idx, point);
        self.bounds_needs_update = TRUE;
    }

    pub fn delete_point(&mut self, ndx: Int) {
        if ndx < 0 {
            return;
        }
        let idx = ndx as usize;
        if idx >= self.points.len() {
            return;
        }
        self.points.remove(idx);
        self.bounds_needs_update = TRUE;
    }

    /// Recomputes 2D bounding box and bounding radius using deterministic `gm_sqrtf`.
    pub fn update_bounds(&mut self) {
        self.bounds.lo.x = BIG_INT;
        self.bounds.lo.y = BIG_INT;
        self.bounds.hi.x = -BIG_INT;
        self.bounds.hi.y = -BIG_INT;
        for pt in &self.points {
            let p2 = ICoord2D::new(pt.x, pt.y);
            self.bounds.unite_with_point(&p2);
        }
        self.bounds_needs_update = FALSE;
        let half_width: Real = (self.bounds.width() as Real) / 2.0_f32;
        let half_height: Real = if self.retail_compatible_crc {
            ((self.bounds.hi.y.wrapping_add(self.bounds.lo.y)) as Real) / 2.0_f32
        } else {
            (self.bounds.height() as Real) / 2.0_f32
        };
        self.radius = gm_sqrtf(half_height * half_height + half_width * half_width);
    }

    /// Computes center `(x, y)` of the 2D bounds with `z = ground_height(x, y)`.
    pub fn get_center_point<F: FnMut(Real, Real) -> Real>(&mut self, mut ground_height: F) -> Coord3D {
        if self.bounds_needs_update != FALSE {
            self.update_bounds();
        }
        let x = ((self.bounds.lo.x.wrapping_add(self.bounds.hi.x)) as Real) / 2.0_f32;
        let y = ((self.bounds.lo.y.wrapping_add(self.bounds.hi.y)) as Real) / 2.0_f32;
        let z = ground_height(x, y);
        Coord3D::new(x, y, z)
    }

    pub fn get_radius(&mut self) -> Real {
        if self.bounds_needs_update != FALSE {
            self.update_bounds();
        }
        self.radius
    }

    /// Exact raycast parity matching `PolygonTrigger::pointInTrigger`.
    pub fn point_in_trigger(&mut self, point: ICoord3D) -> Bool {
        if self.bounds_needs_update != FALSE {
            self.update_bounds();
        }
        self.contains_point(point)
    }

    /// [`Self::point_in_trigger`] for a trigger whose bounds are up to date.
    pub fn contains_point(&self, point: ICoord3D) -> Bool {
        if point.x < self.bounds.lo.x
            || point.y < self.bounds.lo.y
            || point.x > self.bounds.hi.x
            || point.y > self.bounds.hi.y
        {
            return FALSE;
        }
        let mut inside = false;
        let n = self.points.len();
        for i in 0..n {
            let pt1 = self.points[i];
            let pt2 = if i == n - 1 {
                self.points[0]
            } else {
                self.points[i + 1]
            };
            if pt1.y == pt2.y {
                continue;
            }
            if pt1.y < point.y && pt2.y < point.y {
                continue;
            }
            if pt1.y >= point.y && pt2.y >= point.y {
                continue;
            }
            if pt1.x < point.x && pt2.x < point.x {
                continue;
            }
            let dy = pt2.y.wrapping_sub(pt1.y);
            let dx = pt2.x.wrapping_sub(pt1.x);
            let intersection_x = (pt1.x as Real)
                + ((dx.wrapping_mul(point.y.wrapping_sub(pt1.y)) as Real) / (dy as Real));
            if intersection_x >= (point.x as Real) {
                inside = !inside;
            }
        }
        Bool::from(inside)
    }

    #[inline]
    pub fn is_valid(&self) -> Bool {
        if self.points.is_empty() {
            FALSE
        } else {
            TRUE
        }
    }

    #[inline]
    pub fn water_handle(&self) -> Option<WaterHandle> {
        if self.is_water_area != FALSE {
            Some(WaterHandle {
                polygon_id: self.trigger_id,
            })
        } else {
            None
        }
    }
}

impl Snapshot for PolygonTrigger {
    fn crc(&self, _xfer: &mut dyn Xfer) {}

    fn xfer(&mut self, xfer: &mut dyn Xfer) -> Result<(), XferError> {
        let mut version = 1u8;
        xfer.xfer_version(&mut version, 1)?;

        let mut num_points: Int = self.points.len() as Int;
        xfer.xfer_int(&mut num_points)?;
        if xfer.mode() == XferMode::Load {
            let count = usize::try_from(num_points).map_err(|_| XferError::InvalidData)?;
            self.points.resize(count, ICoord3D::default());
        }
        for pt in &mut self.points {
            xfer.xfer_icoord3d(pt)?;
        }
        xfer.xfer_iregion2d(&mut self.bounds)?;
        xfer.xfer_real(&mut self.radius)?;
        xfer.xfer_bool(&mut self.bounds_needs_update)?;
        Ok(())
    }

    fn load_post_process(&mut self) {}
}

/// Global polygon trigger registry replacing C++ `ThePolygonTriggerListPtr` & `s_currentID`.
#[derive(Debug, Clone)]
pub struct PolygonTriggerList {
    triggers: Vec<PolygonTrigger>,
    current_id: Int,
    pub retail_compatible_crc: bool,
    pub retail_compatible_data: bool,
}

impl Default for PolygonTriggerList {
    fn default() -> Self {
        Self::new()
    }
}

impl PolygonTriggerList {
    pub fn new() -> Self {
        Self {
            triggers: Vec::new(),
            current_id: 1,
            retail_compatible_crc: false,
            retail_compatible_data: false,
        }
    }

    /// Allocates a new empty trigger with the next sequential ID (without inserting it).
    pub fn create_trigger(&mut self) -> PolygonTrigger {
        let id = self.current_id;
        self.current_id = self.current_id.wrapping_add(1);
        let mut trig = PolygonTrigger::new(id);
        trig.retail_compatible_crc = self.retail_compatible_crc;
        trig
    }

    #[inline]
    pub fn triggers(&self) -> &[PolygonTrigger] {
        &self.triggers
    }

    #[inline]
    pub fn triggers_mut(&mut self) -> &mut Vec<PolygonTrigger> {
        &mut self.triggers
    }

    /// Prepends a trigger at the head of the list matching `PolygonTrigger::addPolygonTrigger`.
    pub fn add_polygon_trigger(&mut self, trig: PolygonTrigger) {
        self.triggers.insert(0, trig);
    }

    /// Removes and returns the trigger with `trigger_id` if present.
    pub fn remove_polygon_trigger(&mut self, trigger_id: Int) -> Option<PolygonTrigger> {
        let idx = self
            .triggers
            .iter()
            .position(|t| t.trigger_id == trigger_id)?;
        Some(self.triggers.remove(idx))
    }

    pub fn get_polygon_trigger_by_id(&self, trigger_id: Int) -> Option<&PolygonTrigger> {
        self.triggers.iter().find(|t| t.trigger_id == trigger_id)
    }

    pub fn get_polygon_trigger_by_id_mut(&mut self, trigger_id: Int) -> Option<&mut PolygonTrigger> {
        self.triggers.iter_mut().find(|t| t.trigger_id == trigger_id)
    }

    pub fn delete_triggers(&mut self) {
        self.triggers.clear();
        self.current_id = 1;
    }

    pub fn write_polygon_triggers_data_chunk<O: DataChunkOutputStream>(
        &self,
        out: &mut O,
    ) -> Result<(), DataChunkError> {
        let version = if self.retail_compatible_data {
            K_TRIGGERS_VERSION_3
        } else {
            K_TRIGGERS_VERSION_4
        };
        let mut writer = DataChunkOutput::new(out);
        writer.open_data_chunk("PolygonTriggers", version)?;
        writer.write_int(self.triggers.len() as Int)?;
        for trig in &self.triggers {
            writer.write_ascii_string(&trig.trigger_name)?;
            if version >= K_TRIGGERS_VERSION_4 {
                writer.write_ascii_string(&trig.layer_name)?;
            }
            writer.write_int(trig.trigger_id)?;
            writer.write_byte(if trig.is_water_area != FALSE { 1 } else { 0 })?;
            writer.write_byte(if trig.is_river != FALSE { 1 } else { 0 })?;
            writer.write_int(trig.river_start)?;
            writer.write_int(trig.points.len() as Int)?;
            for pt in &trig.points {
                writer.write_int(pt.x)?;
                writer.write_int(pt.y)?;
                writer.write_int(pt.z)?;
            }
        }
        writer.close_data_chunk()?;
        writer.finish()
    }

    /// Parses a complete `DataChunk` buffer containing a `"PolygonTriggers"` chunk.
    pub fn parse_polygon_triggers_chunk_stream(
        &mut self,
        chunk_bytes: &[u8],
        water_extent_x: Int,
        water_extent_y: Int,
    ) -> Result<bool, DataChunkError> {
        struct ParseCtx<'a> {
            list: &'a mut PolygonTriggerList,
            water_extent_x: Int,
            water_extent_y: Int,
        }

        fn on_polygon_triggers(
            input: &mut DataChunkInput<'_, ParseCtx<'_>>,
            info: &DataChunkInfo,
            ctx: &mut ParseCtx<'_>,
        ) -> Result<bool, DataChunkError> {
            ctx.list.delete_triggers();
            let mut max_trigger_id: Int = 0;
            let count = input.read_int()?;
            for _ in 0..count {
                let trigger_name = input.read_ascii_string()?;
                let layer_name = if info.version >= K_TRIGGERS_VERSION_4 {
                    input.read_ascii_string()?
                } else {
                    String::new()
                };
                let trigger_id = input.read_int()?;
                let is_water = if info.version >= K_TRIGGERS_VERSION_2 {
                    Bool::from(input.read_byte()? != 0)
                } else {
                    FALSE
                };
                let (is_river, river_start) = if info.version >= K_TRIGGERS_VERSION_3 {
                    (Bool::from(input.read_byte()? != 0), input.read_int()?)
                } else {
                    (FALSE, 0)
                };
                let num_points = input.read_int()?;
                let mut trig = PolygonTrigger::new(trigger_id);
                trig.retail_compatible_crc = ctx.list.retail_compatible_crc;
                trig.trigger_name = trigger_name;
                trig.layer_name = layer_name;
                trig.is_water_area = is_water;
                trig.is_river = is_river;
                trig.river_start = river_start;
                if trigger_id > max_trigger_id {
                    max_trigger_id = trigger_id;
                }
                for _ in 0..num_points {
                    let x = input.read_int()?;
                    let y = input.read_int()?;
                    let z = input.read_int()?;
                    trig.add_point(ICoord3D::new(x, y, z));
                }
                if !ctx.list.retail_compatible_crc && num_points < 2 {
                    continue;
                }
                ctx.list.triggers.push(trig);
            }
            if info.version == K_TRIGGERS_VERSION_1 {
                let mut trig = PolygonTrigger::new(max_trigger_id);
                max_trigger_id = max_trigger_id.wrapping_add(1);
                trig.retail_compatible_crc = ctx.list.retail_compatible_crc;
                trig.is_water_area = TRUE;
                let mut loc = ICoord3D::new(-30 * MAP_XY_FACTOR, -30 * MAP_XY_FACTOR, 7);
                trig.add_point(loc);
                loc.x = 30 * MAP_XY_FACTOR + ctx.water_extent_x;
                trig.add_point(loc);
                loc.y = 30 * MAP_XY_FACTOR + ctx.water_extent_y;
                trig.add_point(loc);
                loc.x = -30 * MAP_XY_FACTOR;
                trig.add_point(loc);
                ctx.list.triggers.push(trig);
            }
            ctx.list.current_id = max_trigger_id.wrapping_add(1);
            if !input.at_end_of_chunk() {
                return Err(DataChunkError::CorruptFormat);
            }
            Ok(true)
        }

        let mut stream = CachedFileInputStream::from_bytes(chunk_bytes);
        let mut input = DataChunkInput::new(&mut stream);
        if !input.is_valid_file_type() {
            return Ok(false);
        }
        input.register_parser("PolygonTriggers", "", on_polygon_triggers);
        let mut ctx = ParseCtx {
            list: self,
            water_extent_x,
            water_extent_y,
        };
        input.parse(&mut ctx)
    }
}
