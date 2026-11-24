//! `PKT-207`: `CrushDie` (`CrushDie.h` / `CrushDie.cpp`).

use crate::arena::{FacetMask, ModuleContext, ModuleSidecar, ObjectSlot};
use crate::combat::damage::DamageInfo;
use crate::modules::die::die_module::{
    impl_die_behavior, parse_die_data_with, DieBehaviorImpl, DieModule, DieMuxData,
};
use crate::modules::module::{BehaviorModule, DieFacet, ModuleData};
use crate::modules::module_factory::ModuleParseContext;
use crate::rules::combat_types::DamageType;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::sim::{SimApi, SimView};
use zh_common::ini::{IniError, IniReader, SEPS_DEFAULT};
use zh_common::types::{Bool, Int, NameKeyType, Real, FALSE};

/// Crush hit classification matching C++ `CrushEnum`.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrushEnum {
    TotalCrush = 0,
    BackEndCrush = 1,
    FrontEndCrush = 2,
    NoCrush = 3,
}

const CRUSH_COUNT: usize = 4;

fn parse_audio_name(reader: &mut IniReader<'_>, out: &mut String) -> Result<(), IniError> {
    let tok = reader.next_token(SEPS_DEFAULT)?;
    if tok.eq_ignore_ascii_case("NoSound") {
        out.clear();
    } else {
        *out = tok.to_owned();
    }
    Ok(())
}

/// Template configuration parsed for `CrushDie`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrushDieModuleData {
    pub tag_key: NameKeyType,
    pub module_name_key: NameKeyType,
    pub die_mux: DieMuxData,
    pub crush_sounds: [String; CRUSH_COUNT],
    pub crush_sound_percent: [Int; CRUSH_COUNT],
}

impl ModuleData for CrushDieModuleData {
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
        FacetMask::DIE
    }
}

pub fn parse_crush_die_data(
    ini: &mut IniReader<'_>,
    tag_key: NameKeyType,
    module_name_key: NameKeyType,
    parse_ctx: &mut ModuleParseContext<'_>,
) -> Result<Box<dyn ModuleData>, IniError> {
    let mut crush_sounds: [String; CRUSH_COUNT] = Default::default();
    let mut crush_sound_percent = [100; CRUSH_COUNT];
    let base = parse_die_data_with(
        ini,
        tag_key,
        module_name_key,
        parse_ctx,
        |field, reader, _ctx| {
            if field.eq_ignore_ascii_case("TotalCrushSound") {
                parse_audio_name(reader, &mut crush_sounds[CrushEnum::TotalCrush as usize])
            } else if field.eq_ignore_ascii_case("BackEndCrushSound") {
                parse_audio_name(reader, &mut crush_sounds[CrushEnum::BackEndCrush as usize])
            } else if field.eq_ignore_ascii_case("FrontEndCrushSound") {
                parse_audio_name(reader, &mut crush_sounds[CrushEnum::FrontEndCrush as usize])
            } else if field.eq_ignore_ascii_case("TotalCrushSoundPercent") {
                reader.parse_int(&mut crush_sound_percent[CrushEnum::TotalCrush as usize])
            } else if field.eq_ignore_ascii_case("BackEndCrushSoundPercent") {
                reader.parse_int(&mut crush_sound_percent[CrushEnum::BackEndCrush as usize])
            } else if field.eq_ignore_ascii_case("FrontEndCrushSoundPercent") {
                reader.parse_int(&mut crush_sound_percent[CrushEnum::FrontEndCrush as usize])
            } else {
                Err(IniError::UnknownToken)
            }
        },
    )?;
    Ok(Box::new(CrushDieModuleData {
        tag_key: base.tag_key,
        module_name_key: base.module_name_key,
        die_mux: base.die_mux,
        crush_sounds,
        crush_sound_percent,
    }))
}

#[derive(Debug, Clone)]
pub struct CrushDie {
    base: DieModule,
    crush_sounds: [String; CRUSH_COUNT],
    crush_sound_percent: [Int; CRUSH_COUNT],
}

impl CrushDie {
    pub fn create(data: &dyn ModuleData, _ctx: ModuleContext) -> Box<dyn BehaviorModule> {
        let typed = data
            .as_any()
            .downcast_ref::<CrushDieModuleData>()
            .expect("CrushDieModuleData mismatch");
        Box::new(Self {
            base: DieModule::new(typed.module_name_key, typed.die_mux.clone()),
            crush_sounds: typed.crush_sounds.clone(),
            crush_sound_percent: typed.crush_sound_percent,
        })
    }

    fn crush_location_check(
        sim: &dyn SimView,
        crusher_slot: ObjectSlot,
        victim_slot: ObjectSlot,
    ) -> CrushEnum {
        let Some(crusher) = sim.object(crusher_slot) else {
            return CrushEnum::NoCrush;
        };
        let Some(victim) = sim.object(victim_slot) else {
            return CrushEnum::NoCrush;
        };

        let (front_crushed, back_crushed) = victim
            .body_slot
            .map(|bs| match sim.module_sidecar(bs) {
                ModuleSidecar::Body {
                    front_crushed,
                    back_crushed,
                    ..
                } => (front_crushed != FALSE, back_crushed != FALSE),
                _ => (false, false),
            })
            .unwrap_or((false, false));

        let other_dir = victim.thing.get_unit_direction_vector_2d();
        let pos = crusher.thing.get_position();
        let other_pos = victim.thing.get_position();
        let crush_point_offset_dist = victim.geometry.get_major_radius() * 0.5_f32;
        let off_x = other_dir.x * crush_point_offset_dist;
        let off_y = other_dir.y * crush_point_offset_dist;

        let mut retval = CrushEnum::NoCrush;
        let mut best_dist: Real = 99999.0;

        if !front_crushed && !back_crushed {
            let dx = other_pos.x - pos.x;
            let dy = other_pos.y - pos.y;
            let dist = dx * dx + dy * dy;
            retval = CrushEnum::TotalCrush;
            best_dist = dist;
        }

        if !front_crushed {
            let dx = (other_pos.x + off_x) - pos.x;
            let dy = (other_pos.y + off_y) - pos.y;
            let dist = dx * dx + dy * dy;
            if dist < best_dist {
                retval = if back_crushed {
                    CrushEnum::TotalCrush
                } else {
                    CrushEnum::FrontEndCrush
                };
                best_dist = dist;
            }
        }

        if !back_crushed {
            let dx = (other_pos.x - off_x) - pos.x;
            let dy = (other_pos.y - off_y) - pos.y;
            let dist = dx * dx + dy * dy;
            if dist < best_dist {
                retval = if front_crushed {
                    CrushEnum::TotalCrush
                } else {
                    CrushEnum::BackEndCrush
                };
            }
        }

        retval
    }
}

impl DieBehaviorImpl for CrushDie {
    fn base(&self) -> &DieModule {
        &self.base
    }

    fn base_mut(&mut self) -> &mut DieModule {
        &mut self.base
    }

    fn on_die(&mut self, sim: &mut dyn SimApi, ctx: ModuleContext, dmg: &DamageInfo) {
        if self.base.is_die_applicable(sim, ctx, dmg) == FALSE {
            return;
        }
        if dmg.input.damage_type != DamageType::Crush {
            return;
        }

        let crush_type = if let Some(dealer) = sim.find_object(dmg.input.source_id) {
            Self::crush_location_check(sim, dealer, ctx.owner_slot)
        } else {
            CrushEnum::TotalCrush
        };

        if crush_type != CrushEnum::NoCrush {
            let idx = crush_type as usize;
            if !self.crush_sounds[idx].is_empty() {
                let roll = sim.rng_logic_int(0, 99);
                let _play_sound = roll < self.crush_sound_percent[idx];
            }

            let front = Bool::from(
                crush_type == CrushEnum::TotalCrush || crush_type == CrushEnum::FrontEndCrush,
            );
            let back = Bool::from(
                crush_type == CrushEnum::TotalCrush || crush_type == CrushEnum::BackEndCrush,
            );
            sim.set_crushed(ctx.owner_slot, Some(front), Some(back));

            let mut clear = ModelConditionFlags::new();
            clear.set(ModelConditionFlagType::BackCrushed.as_i32(), 1);
            clear.set(ModelConditionFlagType::FrontCrushed.as_i32(), 1);

            let mut set = ModelConditionFlags::new();
            if front != FALSE {
                set.set(ModelConditionFlagType::FrontCrushed.as_i32(), 1);
            }
            if back != FALSE {
                set.set(ModelConditionFlagType::BackCrushed.as_i32(), 1);
            }
            sim.clear_and_set_model_condition_flags(ctx.owner_slot, &clear, &set);
        }
    }
}

impl_die_behavior!(CrushDie);
