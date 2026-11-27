//! `PKT-310`: `BoneFXUpdate` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::{parse, push, DataPart, FieldList, FieldText};
use crate::modules::module_factory::ModuleParseContext;
use std::collections::BTreeSet;
use zh_common::ini::{DamageTypeFlags, IniError, IniReader, SEPS_COLON, SEPS_DEFAULT};
use zh_common::types::{Bool, Real, TRUE};

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
