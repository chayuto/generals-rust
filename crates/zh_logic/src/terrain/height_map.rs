//! The logic height map: `WorldHeightMap` as far as the simulation reads it, and
//! the queries of `BaseHeightMapRenderObjClass` over it.
//!
//! - `WorldHeightMap.cpp`: `ParseSizeOnly`, `ParseHeightMapData`,
//!   `ParseBlendTileData` up to the cliff flags, `initCliffFlagsFromHeights`.
//! - `BaseHeightMap.cpp`: `getHeightMapHeight`, `isClearLineOfSight`,
//!   `getMaxCellHeight`, `isCliffCell`, and `getClipHeight` of the header.
//! - `W3DTerrainVisual.cpp`: `getRawMapHeight`, `setRawMapHeight`.
//!
//! Golden test: `crates/zh_logic/tests/golden_s1_terrain.rs`.

use crate::spatial::terrain::CliffMap;
use zh_common::io::data_chunk::{DataChunkError, DataChunkInfo, DataChunkInput};
use zh_common::io::map_object::{MAP_HEIGHT_SCALE, MAP_XY_FACTOR};
use zh_common::io::map_reader_writer_info::*;
use zh_common::math::{Coord3D, ICoord2D};
use zh_common::types::{Bool, Int, Real, FALSE, TRUE};

/// `K_MAX_HEIGHT`: what `getMaxHeightValue` answers.
pub const MAX_HEIGHT_VALUE: Int = 255;

/// Height samples, boundaries and cliff flags of a map.
#[derive(Debug, Clone, Default)]
pub struct HeightMap {
    width: Int,
    height: Int,
    border: Int,
    /// `m_boundaries`: the playable sizes, in cells.
    boundaries: Vec<ICoord2D>,
    /// `m_dataSize`: the samples `getHeight` may read.
    data_size: Int,
    data: Vec<u8>,
    /// Empty until a blend tile chunk has been read.
    cliffs: CliffMap,
    /// `getMaxHeight` of the render object, taken when the map was loaded.
    max_height: Real,
}

fn valid_total_map_size(width: Int, height: Int, border: Int) -> bool {
    if width <= 0 || width > MAX_MAP_DIMENSION || height <= 0 || height > MAX_MAP_DIMENSION {
        return false;
    }
    if border < 0 || border > (width - 1) / 2 || border > (height - 1) / 2 {
        return false;
    }
    width * height <= MAX_MAP_AREA
}

/// `x*MAP_XY_FACTOR_INV` floored, and the fraction.
fn split_cell(world: Real) -> (Real, Real) {
    let inverse = 1.0 / MAP_XY_FACTOR;
    let div = world * inverse;
    #[cfg(feature = "retail-crc")]
    let floor = zh_common::math::fast_float_floor(div);
    #[cfg(not(feature = "retail-crc"))]
    let floor = zh_common::math::gm_floorf(div);
    (floor, div - floor)
}

fn flat_normal(normal: Option<&mut Coord3D>) {
    if let Some(n) = normal {
        *n = Coord3D::new(0.0, 0.0, 1.0);
    }
}

impl HeightMap {
    /// A map built from samples, with the cliff flags taken from the heights.
    pub fn from_samples(
        width: Int,
        height: Int,
        border: Int,
        boundaries: Vec<ICoord2D>,
        data: Vec<u8>,
    ) -> Self {
        let mut map = Self {
            width,
            height,
            border,
            boundaries,
            data_size: data.len() as Int,
            data,
            cliffs: CliffMap::default(),
            max_height: 0.0,
        };
        map.init_cliff_flags_from_heights();
        map.update_max_height();
        map
    }

    pub fn width(&self) -> Int {
        self.width
    }

    pub fn height(&self) -> Int {
        self.height
    }

    pub fn border_size(&self) -> Int {
        self.border
    }

    pub fn boundaries(&self) -> &[ICoord2D] {
        &self.boundaries
    }

    pub fn cliffs(&self) -> &CliffMap {
        &self.cliffs
    }

    /// The samples `getDataPtr` points at.
    pub fn samples(&self) -> &[u8] {
        &self.data
    }

    /// `BaseHeightMapRenderObjClass::getMaxHeight`.
    pub fn max_height(&self) -> Real {
        self.max_height
    }

    /// The scan of `initHeightData`; the caller of the C++ runs it once, at load.
    pub(crate) fn update_max_height(&mut self) {
        let (_, max) = self.height_range();
        self.max_height = max as Real * MAP_HEIGHT_SCALE;
    }

    /// The lowest and highest raw samples, starting from 255 and 0.
    pub(crate) fn height_range(&self) -> (Int, Int) {
        let mut low = MAX_HEIGHT_VALUE;
        let mut high = 0;
        for j in 0..self.height {
            for i in 0..self.width {
                let current = Int::from(self.raw_height(i, j));
                if current < low {
                    low = current;
                }
                if high < current {
                    high = current;
                }
            }
        }
        (low, high)
    }

    fn index(&self, x: Int, y: Int) -> Option<usize> {
        let ndx = y.wrapping_mul(self.width).wrapping_add(x);
        (ndx >= 0 && ndx < self.data_size && (ndx as usize) < self.data.len())
            .then_some(ndx as usize)
    }

    /// `WorldHeightMap::getHeight`. Only the index is checked, so an `x` past a
    /// row reads the next one.
    pub fn raw_height(&self, x: Int, y: Int) -> u8 {
        self.index(x, y).map_or(0, |i| self.data[i])
    }

    /// `W3DTerrainVisual::getRawMapHeight`, in cells without the border.
    pub fn raw_map_height(&self, cell: ICoord2D) -> Int {
        Int::from(self.raw_height(cell.x + self.border, cell.y + self.border))
    }

    /// `W3DTerrainVisual::setRawMapHeight`: only ever lowers a sample.
    pub fn set_raw_map_height(&mut self, cell: ICoord2D, height: Int) {
        let (x, y) = (cell.x + self.border, cell.y + self.border);
        if Int::from(self.raw_height(x, y)) > height {
            if let Some(i) = self.index(x, y) {
                self.data[i] = height as u8;
            }
        }
    }

    /// A sample by its index in the data; zero past the end.
    fn sample(&self, index: Int) -> u8 {
        usize::try_from(index)
            .ok()
            .and_then(|i| self.data.get(i))
            .copied()
            .unwrap_or(0)
    }

    /// `getClipHeight`.
    fn clip_height(&self, x: Int, y: Int) -> u8 {
        let x = x.min(self.width - 1).max(0);
        let y = y.min(self.height - 1).max(0);
        self.sample(x + y * self.width)
    }

    /// `getHeightMapHeight`: the height of the triangle under the point, and its
    /// normal interpolated from the surrounding samples.
    pub fn ground_height(&self, x: Real, y: Real, normal: Option<&mut Coord3D>) -> Real {
        let (x_floor, fx) = split_cell(x);
        let (y_floor, fy) = split_cell(y);
        let ix = zh_common::math::gamemath::gm_lroundf(x_floor).wrapping_add(self.border);
        let iy = zh_common::math::gamemath::gm_lroundf(y_floor).wrapping_add(self.border);
        let x_extent = self.width;
        if ix > x_extent - 3 || iy > self.height - 3 || iy < 1 || ix < 1 {
            flat_normal(normal);
            return Real::from(self.clip_height(ix, iy)) * MAP_HEIGHT_SCALE;
        }

        let idx = ix + iy * x_extent;
        let at = |i: Int| Real::from(self.sample(i));
        let p0 = at(idx);
        let p2 = at(idx + x_extent + 1);
        let height = if fy > fx {
            let p3 = at(idx + x_extent);
            (p3 + (1.0 - fy) * (p0 - p3) + fx * (p2 - p3)) * MAP_HEIGHT_SCALE
        } else {
            let p1 = at(idx + 1);
            (p1 + fy * (p2 - p1) + (1.0 - fx) * (p0 - p1)) * MAP_HEIGHT_SCALE
        };

        if let Some(normal) = normal {
            let idx4 = ix + (iy - 1) * x_extent;
            let idx0 = idx;
            let idx3 = idx + x_extent;
            let idx9 = ix + (iy + 2) * x_extent;
            let byte = |i: Int| Int::from(self.sample(i));
            let d0 = byte(idx0);
            let d1 = byte(idx0 + 1);
            let d2 = byte(idx3 + 1);
            let d3 = byte(idx3);
            let d4 = byte(idx4);
            let d5 = byte(idx4 + 1);
            let d6 = byte(idx0 + 2);
            let d7 = byte(idx3 + 2);
            let d8 = byte(idx9 + 1);
            let d9 = byte(idx9);
            let d11 = byte(idx0 - 1);

            let x0 = (d1 - d11) as Real;
            let x1 = (d6 - d0) as Real;
            let x2 = (d7 - d3) as Real;
            // The original takes this difference from the wrong pair of samples.
            let x3 = (d6 - d0) as Real;
            let y0 = (d3 - d4) as Real;
            let y1 = (d2 - d5) as Real;
            let y2 = (d8 - d1) as Real;
            let y3 = (d9 - d0) as Real;

            // `1.0-fy` is a double in the original, so the last blend is too.
            let blend = |left: Real, right: Real| {
                (f64::from(left) * (1.0_f64 - f64::from(fy)) + f64::from(fy * right)) as Real
            };
            let delta_x = blend(x0 * (1.0 - fx) + fx * x3, x1 * (1.0 - fx) + fx * x2);
            let delta_y = blend(y0 * (1.0 - fx) + fx * y3, y1 * (1.0 - fx) + fx * y2);

            let step = 2.0 * MAP_XY_FACTOR / MAP_HEIGHT_SCALE;
            let l2r = Coord3D::new(step, 0.0, delta_x);
            let n2f = Coord3D::new(0.0, step, delta_y);
            Coord3D::cross_product(&l2r, &n2f, normal);
            normal.normalize_ww_vector3();
        }
        height
    }

    /// The cell holding the point, clamped to the map.
    fn clamped_cell(&self, x: Real, y: Real) -> (Int, Int) {
        let cell = |world: Real, extent: Int| {
            let i = ((world / MAP_XY_FACTOR) as Int)
                .wrapping_add(self.border)
                .max(0);
            if i >= extent - 1 {
                extent - 2
            } else {
                i
            }
        };
        (cell(x, self.width), cell(y, self.height))
    }

    /// The highest corner of the cell at the indices.
    fn cell_max_sample(&self, x: Int, y: Int) -> Real {
        let idx = x + y * self.width;
        let mut high = Real::from(self.sample(idx));
        for i in [idx + 1, idx + self.width + 1, idx + self.width] {
            let h = Real::from(self.sample(i));
            if high <= h {
                high = h;
            }
        }
        high
    }

    /// `getMaxCellHeight`.
    pub fn max_cell_height(&self, x: Real, y: Real) -> Real {
        let (ix, iy) = self.clamped_cell(x, y);
        self.cell_max_sample(ix, iy) * MAP_HEIGHT_SCALE
    }

    /// `BaseHeightMapRenderObjClass::isCliffCell`.
    pub fn is_cliff_cell(&self, x: Real, y: Real) -> Bool {
        let (ix, iy) = self.clamped_cell(x, y);
        self.cliffs.get_cliff_state(ix, iy)
    }

    /// `isClearLineOfSight`: walks the cells of the line and compares the highest
    /// corner of each with the height of the sight line.
    pub fn is_clear_line_of_sight(&self, pos: &Coord3D, other: &Coord3D) -> Bool {
        let inverse = 1.0 / MAP_XY_FACTOR;
        let cell = |world: Real| {
            zh_common::math::gamemath::real_to_int_floor(world * inverse).wrapping_add(self.border)
        };
        let (mut x, mut y) = (cell(pos.x), cell(pos.y));
        let (end_x, end_y) = (cell(other.x), cell(other.y));
        let delta_x = end_x.wrapping_sub(x).wrapping_abs();
        let delta_y = end_y.wrapping_sub(y).wrapping_abs();
        let step_x: Int = if end_x >= x { 1 } else { -1 };
        let step_y: Int = if end_y >= y { 1 } else { -1 };
        let (mut xinc1, mut xinc2, mut yinc1, mut yinc2) = (step_x, step_x, step_y, step_y);
        let (den, mut num, numadd, pixels) = if delta_x >= delta_y {
            xinc1 = 0;
            yinc2 = 0;
            (delta_x, delta_x / 2, delta_y, delta_x)
        } else {
            xinc2 = 0;
            yinc1 = 0;
            (delta_y, delta_y / 2, delta_x, delta_y)
        };

        let pixels_inverse = 1.0 / pixels as Real;
        let mut z = pos.z;
        let z_inc = (other.z - z) * pixels_inverse;
        for _ in 0..pixels {
            if x < 0 || y < 0 || x >= self.width - 1 || y >= self.height - 1 {
                break;
            }
            let height = self.cell_max_sample(x, y) * MAP_HEIGHT_SCALE;
            const FUDGE: Real = 0.5;
            if height > z + FUDGE {
                return FALSE;
            }
            if z >= self.max_height && z_inc > 0.0 {
                break;
            }
            z += z_inc;
            num += numadd;
            if num >= den {
                num -= den;
                x += xinc1;
                y += yinc1;
            }
            x += xinc2;
            y += yinc2;
        }
        TRUE
    }

    /// `initCliffFlagsFromHeights`.
    fn init_cliff_flags_from_heights(&mut self) {
        let samples = usize::try_from(self.data_size)
            .ok()
            .map_or(&self.data[..], |n| &self.data[..n.min(self.data.len())]);
        self.cliffs = CliffMap::from_heights(self.width, self.height, self.border, samples);
    }

    /// The size fields, boundaries and samples shared by both readers of a
    /// `HeightMapData` chunk.
    fn read_height_data<Ctx>(
        &mut self,
        file: &mut DataChunkInput<'_, Ctx>,
        info: &DataChunkInfo,
    ) -> Result<(), DataChunkError> {
        self.width = file.read_int()?;
        self.height = file.read_int()?;
        self.border = if info.version >= K_HEIGHT_MAP_VERSION_3 {
            file.read_int()?
        } else {
            0
        };
        if !valid_total_map_size(self.width, self.height, self.border) {
            return Err(DataChunkError::CorruptFormat);
        }
        if info.version >= K_HEIGHT_MAP_VERSION_4 {
            let count = file.read_int()?;
            self.boundaries.clear();
            for _ in 0..count {
                let x = file.read_int()?;
                let y = file.read_int()?;
                self.boundaries.push(ICoord2D { x, y });
            }
        } else {
            self.boundaries = vec![ICoord2D {
                x: self.width - 2 * self.border,
                y: self.height - 2 * self.border,
            }];
        }
        self.data_size = file.read_int()?;
        let left = file.get_chunk_data_size_left().unwrap_or(0);
        if self.data_size <= 0
            || self.data_size != self.width * self.height
            || self.data_size as u32 > left
        {
            return Err(DataChunkError::CorruptFormat);
        }
        self.data = vec![0; self.data_size as usize];
        file.read_array_of_bytes(&mut self.data)?;
        if info.version == K_HEIGHT_MAP_VERSION_1 {
            // Every other sample, in place: the map had cells half the size.
            let new_width = (self.width + 1) / 2;
            let new_height = (self.height + 1) / 2;
            for i in 0..new_height {
                for j in 0..new_width {
                    self.data[(i * new_width + j) as usize] =
                        self.data[(2 * i * self.width + 2 * j) as usize];
                }
            }
        }
        Ok(())
    }

    /// `ParseSizeOnly`, the reader of the pass that measures the map.
    pub(crate) fn parse_size_only<Ctx>(
        &mut self,
        file: &mut DataChunkInput<'_, Ctx>,
        info: &DataChunkInfo,
    ) -> Result<(), DataChunkError> {
        self.read_height_data(file, info)?;
        if info.version == K_HEIGHT_MAP_VERSION_1 {
            self.width = (self.width + 1) / 2;
            self.height = (self.height + 1) / 2;
        }
        Ok(())
    }

    /// `ParseHeightMapData`. A version 1 map keeps its width until the blend
    /// tile chunk halves it.
    pub(crate) fn parse_height_map_data<Ctx>(
        &mut self,
        file: &mut DataChunkInput<'_, Ctx>,
        info: &DataChunkInfo,
    ) -> Result<(), DataChunkError> {
        self.read_height_data(file, info)
    }

    /// `ParseBlendTileData`, up to the cliff flags; the rest is texturing.
    pub(crate) fn parse_blend_tile_data<Ctx>(
        &mut self,
        file: &mut DataChunkInput<'_, Ctx>,
        info: &DataChunkInfo,
    ) -> Result<(), DataChunkError> {
        let len = file.read_int()?;
        if self.data_size != len {
            return Err(DataChunkError::CorruptFormat);
        }
        let mut shorts = vec![0u8; self.data_size.max(0) as usize * 2];
        // Tile and blend indices, then the extra blend and cliff info indices.
        let mut arrays = 2;
        if info.version >= K_BLEND_TILE_VERSION_6 {
            arrays += 1;
        }
        if info.version >= K_BLEND_TILE_VERSION_5 {
            arrays += 1;
        }
        for _ in 0..arrays {
            file.read_array_of_bytes(&mut shorts)?;
        }

        if info.version >= K_BLEND_TILE_VERSION_7 {
            self.cliffs = CliffMap::new(self.width, self.height, self.border);
            let width = self.width;
            let rows_count = self.height as usize;
            let (stride, bits) = self.cliffs.rows_mut();
            if info.version == K_BLEND_TILE_VERSION_7 {
                // Version 7 saved rows one byte short for most widths.
                let byte_width = ((width + 1) / 8) as usize;
                let mut data = vec![0u8; rows_count * byte_width];
                file.read_array_of_bytes(&mut data)?;
                for j in 0..rows_count {
                    for i in 0..byte_width {
                        bits[j * stride + i] = data[j * byte_width + i];
                    }
                }
            } else {
                file.read_array_of_bytes(bits)?;
            }
        } else {
            self.init_cliff_flags_from_heights();
        }

        if info.version == K_BLEND_TILE_VERSION_1 {
            self.width = (self.width + 1) / 2;
            self.height = (self.height + 1) / 2;
            self.data_size = self.width * self.height;
            self.cliffs.set_sample_size(self.width, self.height);
        }
        Ok(())
    }
}
