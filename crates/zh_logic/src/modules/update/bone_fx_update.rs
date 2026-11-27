//! `PKT-310`: `BoneFXUpdate` and its module data (`BoneFXUpdate.h`,
//! `BoneFXUpdate.cpp`).
//!
//! Packet: `packets/phase5/PKT-310_BoneFXUpdate.md` (not generated).
//! Golden test: `crates/zh_logic/tests/golden_bone_fx_update.rs`.
//!
//! Particle systems are the client's: their frames are drawn from the client
//! stream and saved, but no system is made, so the saved list of ids stays empty.

use crate::arena::{ModuleContext, ObjectSlot};
use crate::gamelogic::global_data::BodyDamageType;
use crate::modules::behavior::behavior_module::{xfer_module_version, UpdateScheduleTracker};
use crate::modules::module::{BehaviorModule, ModuleData, UpdateFacet};
use crate::modules::module_data::module_data;
use crate::modules::module_data::{parse, parse_data, push, DataPart, FieldList, FieldText};
use crate::modules::module_factory::ModuleParseContext;
use crate::sim::{SimApi, SimView};
use std::any::Any;
use std::collections::BTreeSet;
use zh_common::ini::{DamageTypeFlags, IniError, IniReader, SEPS_COLON, SEPS_DEFAULT};
use zh_common::math::Coord3D;
use zh_common::types::{
    Bool, Int, NameKeyType, Real, UnsignedInt, UnsignedShort, UpdateSleepTime, FALSE, TRUE,
};
use zh_common::xfer::{Xfer, XferError, XferMode};

pub const BONE_FX_MAX_BONES: usize = 8;
const BODY_STATES: usize = 4;
const BODY_NAMES: [&str; BODY_STATES] = ["Pristine", "Damaged", "ReallyDamaged", "Rubble"];
/// The table's key part, and the label the line's last pair must carry.
const KINDS: [(&str, &str); 3] = [
    ("FXList", "fxlist"),
    ("OCL", "ocl"),
    ("ParticleSystem", "psys"),
];

/// `BoneFXListInfo`, `BoneOCLInfo` and `BoneParticleSystemInfo`.
#[derive(Debug, Clone)]
pub struct BoneEffect {
    pub bone_name: String,
    pub only_once: Bool,
    /// `setRange` of a uniform distribution, in frames: the game logic variable for
    /// FX lists and creation lists, the client one for particle systems.
    pub delay_low: Real,
    pub delay_high: Real,
    pub name: Option<String>,
}

impl Default for BoneEffect {
    fn default() -> Self {
        Self {
            bone_name: String::new(),
            only_once: TRUE,
            delay_low: 0.0,
            delay_high: 0.0,
            name: None,
        }
    }
}

/// Indexed by kind (FX list, creation list, particle system), body state and bone slot.
#[derive(Debug, Clone, Default)]
pub struct BoneFxTable(pub [[[BoneEffect; BONE_FX_MAX_BONES]; BODY_STATES]; 3]);

fn key(kind: usize, body: usize, slot: usize) -> String {
    format!("{}{}{}", BODY_NAMES[body], KINDS[kind].0, slot + 1)
}

fn find_key(field: &str) -> Option<(usize, usize, usize)> {
    for kind in 0..KINDS.len() {
        for body in 0..BODY_STATES {
            for slot in 0..BONE_FX_MAX_BONES {
                if field == key(kind, body, slot) {
                    return Some((kind, body, slot));
                }
            }
        }
    }
    None
}

fn expect(token: &str, name: &str) -> Result<(), IniError> {
    if token.eq_ignore_ascii_case(name) {
        Ok(())
    } else {
        Err(IniError::InvalidData)
    }
}

/// A `<name>:` label.
fn label_token(r: &mut IniReader<'_>, name: &str) -> Result<(), IniError> {
    expect(r.next_token(SEPS_COLON)?, name)?;
    r.consume_separator();
    Ok(())
}

/// `bone:<name> onlyonce:<bool> <min> <max> <label>:<name>`.
fn bone_effect(
    effect: &mut BoneEffect,
    label: &str,
    known: &BTreeSet<String>,
    r: &mut IniReader<'_>,
) -> Result<(), IniError> {
    label_token(r, "bone")?;
    effect.bone_name = r.next_token(SEPS_DEFAULT)?.to_owned();
    label_token(r, "onlyonce")?;
    r.parse_bool(&mut effect.only_once)?;
    let (mut low, mut high) = (0.0, 0.0);
    r.parse_duration_real(&mut low)?;
    r.parse_duration_real(&mut high)?;
    effect.delay_low = low;
    effect.delay_high = high;
    label_token(r, label)?;
    effect.name = parse::lookup(r.next_token(SEPS_DEFAULT)?, known);
    Ok(())
}

impl DataPart for BoneFxTable {
    fn parse_part(
        &mut self,
        field: &str,
        reader: &mut IniReader<'_>,
        ctx: &ModuleParseContext<'_>,
    ) -> Option<Result<(), IniError>> {
        let (kind, body, slot) = find_key(field)?;
        let known = [ctx.fx_list_names, ctx.ocl_names, ctx.particle_system_names][kind];
        let effect = &mut self.0[kind][body][slot];
        Some(bone_effect(effect, KINDS[kind].1, known, reader))
    }

    fn part_fields(&self, _: Real, out: &mut FieldList) {
        for (kind, bodies) in self.0.iter().enumerate() {
            for (body, slots) in bodies.iter().enumerate() {
                for (slot, e) in slots.iter().enumerate() {
                    let text = format!(
                        "{}|{}|{}|{}|{}",
                        e.bone_name,
                        e.only_once,
                        e.delay_low.field_text(),
                        e.delay_high.field_text(),
                        e.name.field_text()
                    );
                    push(out, &key(kind, body, slot), &text);
                }
            }
        }
    }
}

fn damage_types(
    v: &mut DamageTypeFlags,
    r: &mut IniReader<'_>,
    _: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    r.parse_damage_type_flags(v)
}

module_data! {
    /// `BoneFXUpdateModuleData`.
    BoneFXUpdateModuleData = "BoneFXUpdate";
    part bones: BoneFxTable = |_, _| BoneFxTable::default();
    "DamageFXTypes" => damage_fx_types: DamageTypeFlags = DamageTypeFlags::new_set_all(),
        damage_types;
    "DamageOCLTypes" => damage_ocl_types: DamageTypeFlags = DamageTypeFlags::new_set_all(),
        damage_types;
    "DamageParticleTypes" =>
        damage_particle_types: DamageTypeFlags = DamageTypeFlags::new_set_all(), damage_types;
}

const KIND_FX: usize = 0;
const KIND_OCL: usize = 1;
const KIND_PSYS: usize = 2;

type Frames = [[Int; BONE_FX_MAX_BONES]; BODY_STATES];
type Positions = [[Coord3D; BONE_FX_MAX_BONES]; BODY_STATES];

/// `BoneFXUpdate`: timed effects at bones, by body damage state.
#[derive(Debug, Clone)]
pub struct BoneFXUpdate {
    pub data: BoneFXUpdateModuleData,
    pub schedule: UpdateScheduleTracker,
    /// The client's systems; nothing here makes one.
    pub particle_system_ids: Vec<UnsignedInt>,
    /// By kind (FX list, creation list, particle system): -1 is never.
    pub next_frames: [Frames; 3],
    pub bone_positions: [Positions; 3],
    pub cur_body_state: Int,
    pub bones_resolved: [Bool; BODY_STATES],
    pub active: Bool,
}

impl BoneFXUpdate {
    pub fn new(data: BoneFXUpdateModuleData) -> Self {
        Self {
            data,
            schedule: UpdateScheduleTracker::default(),
            particle_system_ids: Vec::new(),
            next_frames: [[[-1; BONE_FX_MAX_BONES]; BODY_STATES]; 3],
            bone_positions: [[[Coord3D::default(); BONE_FX_MAX_BONES]; BODY_STATES]; 3],
            cur_body_state: BodyDamageType::Pristine as Int,
            bones_resolved: [FALSE; BODY_STATES],
            active: FALSE,
        }
    }

    fn body(&self) -> usize {
        (self.cur_body_state as usize).min(BODY_STATES - 1)
    }

    /// The frame of an effect drawn now. Particle systems draw from the client
    /// stream, the others from the logic stream.
    fn draw_frame(&self, sim: &mut dyn SimApi, kind: usize, slot: usize) -> Int {
        let effect = &self.data.bones.0[kind][self.body()][slot];
        let delay = if kind == KIND_PSYS {
            sim.rng_client_real(effect.delay_low, effect.delay_high)
        } else {
            sim.rng_logic_real(effect.delay_low, effect.delay_high)
        };
        (sim.frame() as Int).wrapping_add(delay as Int)
    }

    fn init_times(&mut self, sim: &mut dyn SimApi) {
        let body = self.body();
        for slot in 0..BONE_FX_MAX_BONES {
            for kind in [KIND_FX, KIND_OCL, KIND_PSYS] {
                self.next_frames[kind][body][slot] =
                    if self.data.bones.0[kind][body][slot].bone_name.is_empty() {
                        -1
                    } else {
                        self.draw_frame(sim, kind, slot)
                    };
            }
        }
    }

    fn compute_next_time(&mut self, sim: &mut dyn SimApi, kind: usize, slot: usize) {
        let body = self.body();
        self.next_frames[kind][body][slot] =
            if self.data.bones.0[kind][body][slot].only_once != FALSE {
                -1
            } else {
                self.draw_frame(sim, kind, slot)
            };
    }

    /// `resolveBoneLocations`: nothing without a drawable; a bone the model does
    /// not have keeps the position it had.
    fn resolve_bone_locations(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot) {
        if sim.object_has_drawable(owner) == FALSE {
            return;
        }
        let body = self.body();
        for slot in 0..BONE_FX_MAX_BONES {
            for kind in [KIND_FX, KIND_OCL, KIND_PSYS] {
                let name = &self.data.bones.0[kind][body][slot].bone_name;
                if name.is_empty() {
                    continue;
                }
                if let Some(bone) = sim.pristine_bone_transforms(owner, name).first() {
                    self.bone_positions[kind][body][slot] = bone.get_translation();
                }
            }
        }
        self.bones_resolved[body] = TRUE;
    }

    /// `doFXListAtBone`, `doOCLAtBone` and `doParticleSystemAtBone`.
    fn do_at_bone(&mut self, sim: &mut dyn SimApi, owner: ObjectSlot, kind: usize, slot: usize) {
        let body = self.body();
        if self.bones_resolved[body] == FALSE {
            self.resolve_bone_locations(sim, owner);
        }
        let types = match kind {
            KIND_FX => &self.data.damage_fx_types,
            KIND_OCL => &self.data.damage_ocl_types,
            _ => &self.data.damage_particle_types,
        };
        if let Some(damage) = sim.object_last_damage_type(owner) {
            if types.test(damage as Int) == FALSE {
                return;
            }
        }
        // The particle system itself is the client's.
        if kind == KIND_PSYS {
            return;
        }
        let Some(name) = self.data.bones.0[kind][body][slot].name.clone() else {
            return;
        };
        let local = self.bone_positions[kind][body][slot];
        let mut world = Coord3D::default();
        if let Some(object) = sim.object(owner) {
            object
                .thing
                .convert_bone_pos_to_world_pos(Some(&local), None, Some(&mut world), None);
        }
        if kind == KIND_FX {
            sim.trigger_fx_list(&name, &world);
        } else {
            sim.spawn_ocl(&name, owner, &world, &world);
        }
    }
}

impl BehaviorModule for BoneFXUpdate {
    fn module_name_key(&self) -> NameKeyType {
        self.data.module_name_key
    }

    fn on_construct(&mut self, _sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.bind(ctx);
    }

    fn crc(&self, xfer: &mut dyn Xfer) {
        self.schedule.crc(xfer);
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, sim: &dyn SimView) -> Result<(), XferError> {
        let _ = xfer_module_version(xfer, 1)?;
        self.schedule.xfer_update_header(xfer, sim)?;
        let mut count = self.particle_system_ids.len() as UnsignedShort;
        xfer.xfer_unsigned_short(&mut count)?;
        if xfer.mode() == XferMode::Load {
            if !self.particle_system_ids.is_empty() {
                return Err(XferError::InvalidData);
            }
            self.particle_system_ids = vec![0; usize::from(count)];
        }
        for id in &mut self.particle_system_ids {
            xfer_raw_int(xfer, id)?;
        }
        // The arrays are saved as raw blocks, one kind after another.
        for frames in &mut self.next_frames {
            let mut bytes: Vec<u8> = frames
                .iter()
                .flatten()
                .flat_map(|f| f.to_le_bytes())
                .collect();
            xfer.xfer_user(&mut bytes)?;
            for (frame, b) in frames.iter_mut().flatten().zip(bytes.chunks_exact(4)) {
                *frame = Int::from_le_bytes([b[0], b[1], b[2], b[3]]);
            }
        }
        for positions in &mut self.bone_positions {
            let mut bytes: Vec<u8> = positions
                .iter()
                .flatten()
                .flat_map(|p| [p.x, p.y, p.z])
                .flat_map(|v| v.to_bits().to_le_bytes())
                .collect();
            xfer.xfer_user(&mut bytes)?;
            let mut values = bytes
                .chunks_exact(4)
                .map(|b| Real::from_bits(u32::from_le_bytes([b[0], b[1], b[2], b[3]])));
            for p in positions.iter_mut().flatten() {
                for v in [&mut p.x, &mut p.y, &mut p.z] {
                    *v = values.next().unwrap_or_default();
                }
            }
        }
        let mut state = self.cur_body_state.to_le_bytes();
        xfer.xfer_user(&mut state)?;
        self.cur_body_state = Int::from_le_bytes(state);
        let mut resolved = self.bones_resolved.map(|b| u8::from(b != FALSE));
        xfer.xfer_user(&mut resolved)?;
        self.bones_resolved = resolved.map(Bool::from);
        xfer.xfer_bool(&mut self.active)
    }

    fn load_post_process(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) {
        self.schedule.load_post_process(sim, ctx);
    }

    fn as_any(&self) -> Option<&dyn Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn Any> {
        Some(self)
    }

    fn as_update(&self) -> Option<&dyn UpdateFacet> {
        Some(self)
    }

    fn as_update_mut(&mut self) -> Option<&mut dyn UpdateFacet> {
        Some(self)
    }
}

fn xfer_raw_int(xfer: &mut dyn Xfer, value: &mut UnsignedInt) -> Result<(), XferError> {
    let mut bytes = value.to_le_bytes();
    xfer.xfer_user(&mut bytes)?;
    *value = UnsignedInt::from_le_bytes(bytes);
    Ok(())
}

impl UpdateFacet for BoneFXUpdate {
    fn update(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext) -> UpdateSleepTime {
        let now = sim.frame() as Int;
        if self.active == FALSE {
            self.init_times(sim);
            self.active = TRUE;
        }
        for slot in 0..BONE_FX_MAX_BONES {
            for kind in [KIND_FX, KIND_OCL, KIND_PSYS] {
                let next = self.next_frames[kind][self.body()][slot];
                if next != -1 && next <= now {
                    self.do_at_bone(sim, ctx.owner_slot, kind, slot);
                    self.compute_next_time(sim, kind, slot);
                }
            }
        }
        UpdateSleepTime::NONE
    }

    fn change_body_damage_state(
        &mut self,
        sim: &mut dyn SimApi,
        _ctx: ModuleContext,
        _old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        self.cur_body_state = new_state as Int;
        self.particle_system_ids.clear();
        self.init_times(sim);
    }

    fn stop_all_bone_fx(&mut self) {
        self.next_frames = [[[-1; BONE_FX_MAX_BONES]; BODY_STATES]; 3];
        self.particle_system_ids.clear();
    }
}

pub fn parse_bone_fx_update_data(
    reader: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    parse_data::<BoneFXUpdateModuleData>(reader, tag_key, module_name_key, ctx)
}

pub fn create_bone_fx_update(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
    let typed = data
        .as_any()
        .downcast_ref::<BoneFXUpdateModuleData>()
        .expect("BoneFXUpdate data type mismatch")
        .clone();
    Box::new(BoneFXUpdate::new(typed))
}
