//! `PKT-190`: `TransitionDamageFX` (`TransitionDamageFX.h` / `TransitionDamageFX.cpp`).

use std::collections::BTreeSet;

use crate::arena::{FacetMask, ModuleContext};
use crate::combat::damage::DamageInfo;
use crate::gamelogic::global_data::{BodyDamageType, BODY_DAMAGE_TYPE_COUNT};
use crate::modules::damage::damage_module::DamageModuleBase;
use crate::modules::module::{BehaviorModule, DamageFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::{parse_damage_type_flags, DamageTypeFlags};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_COLON};
use zh_common::math::Coord3D;
use zh_common::types::{Bool, Int, NameKeyType, UnsignedInt, XferVersion, FALSE};
use zh_common::xfer::{Xfer, XferError};

pub const DAMAGE_MODULE_MAX_FX: usize = 12;

/// Effect placement anchor mode (`TransitionDamageFX.h`).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FxDamageLocType {
    Bone = 0,
    #[default]
    Coord = 1,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxLocInfo {
    pub loc_type: FxDamageLocType,
    pub bone_name: String,
    pub random_bone: Bool,
    pub loc: Coord3D,
}

impl FxLocInfo {
    pub const fn new() -> Self {
        Self {
            loc_type: FxDamageLocType::Coord,
            bone_name: String::new(),
            random_bone: FALSE,
            loc: Coord3D::new(0.0, 0.0, 0.0),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxDamageFxListInfo {
    pub fx_list: Option<String>,
    pub loc_info: FxLocInfo,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxDamageOclInfo {
    pub ocl: Option<String>,
    pub loc_info: FxLocInfo,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct FxDamageParticleSystemInfo {
    pub particle_sys_template: Option<String>,
    pub loc_info: FxLocInfo,
}

/// Template configuration parsed for `TransitionDamageFX`.
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionDamageFxModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub damage_fx_types: DamageTypeFlags,
    pub damage_ocl_types: DamageTypeFlags,
    pub damage_particle_types: DamageTypeFlags,
    pub fx_list: [[FxDamageFxListInfo; DAMAGE_MODULE_MAX_FX]; BODY_DAMAGE_TYPE_COUNT],
    pub ocl: [[FxDamageOclInfo; DAMAGE_MODULE_MAX_FX]; BODY_DAMAGE_TYPE_COUNT],
    pub particle_system:
        [[FxDamageParticleSystemInfo; DAMAGE_MODULE_MAX_FX]; BODY_DAMAGE_TYPE_COUNT],
}

impl TransitionDamageFxModuleData {
    pub fn new(tag_key: NameKeyType, module_name_key: NameKeyType) -> Self {
        Self {
            tag_key,
            module_name_key,
            damage_fx_types: DamageTypeFlags::new_set_all(),
            damage_ocl_types: DamageTypeFlags::new_set_all(),
            damage_particle_types: DamageTypeFlags::new_set_all(),
            fx_list: std::array::from_fn(|_| std::array::from_fn(|_| FxDamageFxListInfo::default())),
            ocl: std::array::from_fn(|_| std::array::from_fn(|_| FxDamageOclInfo::default())),
            particle_system: std::array::from_fn(|_| {
                std::array::from_fn(|_| FxDamageParticleSystemInfo::default())
            }),
        }
    }
}

impl ModuleData for TransitionDamageFxModuleData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn tag_key(&self) -> NameKeyType {
        self.tag_key
    }

    fn module_name_key(&self) -> NameKeyType {
        self.module_name_key
    }

    fn facet_mask(&self) -> FacetMask {
        FacetMask::DAMAGE
    }
}

fn parse_fx_loc_info(ini: &mut IniReader<'_>, loc: &mut FxLocInfo) -> Result<(), IniError> {
    let tok = ini.next_token(SEPS_COLON)?;
    if tok.eq_ignore_ascii_case("bone") {
        loc.bone_name = ini.next_token(SEPS_COLON)?.to_owned();
        loc.loc_type = FxDamageLocType::Bone;
        let rb_val = ini.next_sub_token("randombone")?;
        loc.random_bone = IniReader::scan_bool(rb_val)?;
        Ok(())
    } else if tok.eq_ignore_ascii_case("loc") {
        loc.loc.x = IniReader::scan_real(ini.next_sub_token("X")?)?;
        loc.loc.y = IniReader::scan_real(ini.next_sub_token("Y")?)?;
        loc.loc.z = IniReader::scan_real(ini.next_sub_token("Z")?)?;
        loc.loc_type = FxDamageLocType::Coord;
        Ok(())
    } else {
        Err(IniError::InvalidData)
    }
}

fn resolve_named_asset(tok: &str, valid_names: &BTreeSet<String>) -> Option<String> {
    if tok.eq_ignore_ascii_case("None") {
        return None;
    }
    valid_names
        .iter()
        .find(|n| n.eq_ignore_ascii_case(tok))
        .cloned()
        .or_else(|| Some(tok.to_owned()))
}

fn match_transition_slot(field: &str, kind_suffix: &str) -> Option<(usize, usize)> {
    let prefixes = [
        ("Damaged", BodyDamageType::Damaged as usize),
        ("ReallyDamaged", BodyDamageType::ReallyDamaged as usize),
        ("Rubble", BodyDamageType::Rubble as usize),
    ];
    for (prefix, state_idx) in prefixes {
        if let Some(rest) = field.strip_prefix(prefix) {
            if let Some(num_str) = rest.strip_prefix(kind_suffix) {
                if let Ok(slot_num) = num_str.parse::<usize>() {
                    if (1..=DAMAGE_MODULE_MAX_FX).contains(&slot_num) {
                        return Some((state_idx, slot_num - 1));
                    }
                }
            }
        }
    }
    None
}

pub fn parse_transition_damage_fx_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut data = TransitionDamageFxModuleData::new(tag_key, module_name_key);
    ini.read_block_fields(|field, reader| {
        if field.eq_ignore_ascii_case("DamageFXTypes") {
            data.damage_fx_types = parse_damage_type_flags(reader)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("DamageOCLTypes") {
            data.damage_ocl_types = parse_damage_type_flags(reader)?;
            Ok(())
        } else if field.eq_ignore_ascii_case("DamageParticleTypes") {
            data.damage_particle_types = parse_damage_type_flags(reader)?;
            Ok(())
        } else if let Some((st, idx)) = match_transition_slot(field, "FXList") {
            let slot = &mut data.fx_list[st][idx];
            parse_fx_loc_info(reader, &mut slot.loc_info)?;
            let asset = reader.next_sub_token("fxlist")?;
            slot.fx_list = resolve_named_asset(asset, parse_ctx.fx_list_names);
            Ok(())
        } else if let Some((st, idx)) = match_transition_slot(field, "OCL") {
            let slot = &mut data.ocl[st][idx];
            parse_fx_loc_info(reader, &mut slot.loc_info)?;
            let asset = reader.next_sub_token("ocl")?;
            slot.ocl = resolve_named_asset(asset, parse_ctx.ocl_names);
            Ok(())
        } else if let Some((st, idx)) = match_transition_slot(field, "ParticleSystem") {
            let slot = &mut data.particle_system[st][idx];
            parse_fx_loc_info(reader, &mut slot.loc_info)?;
            let asset = reader.next_sub_token("psys")?;
            slot.particle_sys_template =
                resolve_named_asset(asset, parse_ctx.particle_system_names);
            Ok(())
        } else {
            Err(IniError::UnknownToken)
        }
    })?;
    Ok(Box::new(data))
}

#[derive(Debug, Clone)]
pub struct TransitionDamageFX {
    base: DamageModuleBase,
    data: TransitionDamageFxModuleData,
    pub particle_system_id: [[UnsignedInt; DAMAGE_MODULE_MAX_FX]; BODY_DAMAGE_TYPE_COUNT],
}

impl TransitionDamageFX {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<TransitionDamageFxModuleData>()
            .expect("TransitionDamageFxModuleData mismatch");
        Box::new(Self {
            base: DamageModuleBase::new(typed.module_name_key),
            data: typed.clone(),
            particle_system_id: [[0; DAMAGE_MODULE_MAX_FX]; BODY_DAMAGE_TYPE_COUNT],
        })
    }

    /// Selects a local effect position matching C++ `getLocalEffectPos` with optional
    /// client-provided bone positions (falls back to `loc_info.loc` in headless mode).
    pub fn get_local_effect_pos(
        loc_info: &FxLocInfo,
        bones: &[Coord3D],
        sim: &mut dyn SimApi,
    ) -> Coord3D {
        if loc_info.loc_type == FxDamageLocType::Bone && !bones.is_empty() {
            if loc_info.random_bone == FALSE {
                return bones[0];
            }
            let bone_count = bones.len().min(32) as Int;
            let pick = sim.rng_logic_int(0, bone_count - 1) as usize;
            return bones[pick];
        }
        loc_info.loc
    }

    #[inline]
    pub const fn data(&self) -> &TransitionDamageFxModuleData {
        &self.data
    }
}

impl BehaviorModule for TransitionDamageFX {
    fn module_name_key(&self) -> NameKeyType {
        self.base.module_name_key
    }

    fn xfer(&mut self, xfer: &mut dyn Xfer, _sim: &dyn SimView) -> Result<(), XferError> {
        let mut version: XferVersion = 1;
        xfer.xfer_version(&mut version, 1)?;
        self.base.xfer_base(xfer)?;
        for row in &mut self.particle_system_id {
            for id in row {
                xfer.xfer_unsigned_int(id)?;
            }
        }
        Ok(())
    }

    fn as_damage_mut(&mut self) -> Option<&mut dyn DamageFacet> {
        Some(self)
    }
}

impl DamageFacet for TransitionDamageFX {
    fn on_damage(&mut self, _sim: &mut dyn SimApi, _ctx: ModuleContext, _dmg: &mut DamageInfo) {}

    fn on_body_damage_state_change(
        &mut self,
        sim: &mut dyn SimApi,
        ctx: ModuleContext,
        dmg: &DamageInfo,
        old_state: BodyDamageType,
        new_state: BodyDamageType,
    ) {
        let old_idx = old_state as usize;
        let new_idx = new_state as usize;

        if old_idx < BODY_DAMAGE_TYPE_COUNT {
            for id in &mut self.particle_system_id[old_idx] {
                if *id != 0 {
                    *id = 0;
                }
            }
        }

        if new_state.as_i32() <= old_state.as_i32() || new_idx >= BODY_DAMAGE_TYPE_COUNT {
            return;
        }

        let owner_mtx = sim
            .object(ctx.owner_slot)
            .map(|o| *o.thing.get_transform_matrix())
            .unwrap_or_else(zh_common::math::Matrix3D::identity);
        let source_pos = sim
            .find_object(dmg.input.source_id)
            .and_then(|s| sim.object(s))
            .map(|o| *o.thing.get_position());
        let dt_idx = dmg.input.damage_type.as_i32();

        for i in 0..DAMAGE_MODULE_MAX_FX {
            if let Some(ref fx_name) = self.data.fx_list[new_idx][i].fx_list {
                if self.data.damage_fx_types.test(dt_idx) != FALSE {
                    let local = Self::get_local_effect_pos(
                        &self.data.fx_list[new_idx][i].loc_info,
                        &[],
                        sim,
                    );
                    let world = zh_common::math::Matrix3D::transform_vector(&owner_mtx, &local);
                    sim.trigger_fx_list(fx_name, &world);
                }
            }

            if let Some(src_p) = source_pos {
                if let Some(ref ocl_name) = self.data.ocl[new_idx][i].ocl {
                    if self.data.damage_ocl_types.test(dt_idx) != FALSE {
                        let local = Self::get_local_effect_pos(
                            &self.data.ocl[new_idx][i].loc_info,
                            &[],
                            sim,
                        );
                        let world = zh_common::math::Matrix3D::transform_vector(&owner_mtx, &local);
                        sim.spawn_ocl(ocl_name, ctx.owner_slot, &world, &src_p);
                    }
                }
            }
        }
    }
}
