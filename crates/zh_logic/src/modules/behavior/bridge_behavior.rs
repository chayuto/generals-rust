//! `PKT-260`: `BridgeBehavior` module data. The behaviour is not ported.

use crate::modules::module_data::module_data;
use crate::modules::module_data::parse::velocity;
use crate::modules::module_data::{parse, FieldText};
use crate::modules::module_factory::ModuleParseContext;
use std::collections::BTreeSet;
use zh_common::ini::{IniError, IniReader, SEPS_COLON};
use zh_common::types::{Real, UnsignedInt};

/// `BridgeFXInfo` and `BridgeOCLInfo`.
#[derive(Debug, Clone, PartialEq)]
pub struct BridgeEffect {
    pub name: Option<String>,
    /// `TimeAndLocationInfo`.
    pub delay: UnsignedInt,
    pub bone_name: String,
}

impl FieldText for Vec<BridgeEffect> {
    fn field_text(&self) -> String {
        self.iter()
            .map(|e| format!("{}|{}|{}", e.name.field_text(), e.delay, e.bone_name))
            .collect::<Vec<_>>()
            .join(",")
    }
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

/// `<label>:<name> Delay:<msec> [Bone:<bone>]`.
fn die_effect(
    list: &mut Vec<BridgeEffect>,
    label: &str,
    known: &BTreeSet<String>,
    r: &mut IniReader<'_>,
) -> Result<(), IniError> {
    label_token(r, label)?;
    let name = parse::lookup(r.next_token(zh_common::ini::SEPS_DEFAULT)?, known);
    label_token(r, "Delay")?;
    let mut delay = 0;
    r.parse_duration_unsigned_int(&mut delay)?;
    let mut bone_name = String::new();
    if let Some(token) = r.next_token_or_null(SEPS_COLON) {
        expect(token, "Bone")?;
        r.consume_separator();
        bone_name = r.next_ascii_string()?;
    }
    list.push(BridgeEffect {
        name,
        delay,
        bone_name,
    });
    Ok(())
}

fn die_effect_fx(
    v: &mut Vec<BridgeEffect>,
    r: &mut IniReader<'_>,
    c: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    die_effect(v, "FX", c.fx_list_names, r)
}

fn die_effect_ocl(
    v: &mut Vec<BridgeEffect>,
    r: &mut IniReader<'_>,
    c: &ModuleParseContext<'_>,
) -> Result<(), IniError> {
    die_effect(v, "OCL", c.ocl_names, r)
}

module_data! {
    /// `BridgeBehaviorModuleData`.
    BridgeBehaviorModuleData = "BridgeBehavior";
    "LateralScaffoldSpeed" => lateral_scaffold_speed: Real = 1.0, velocity;
    "VerticalScaffoldSpeed" => vertical_scaffold_speed: Real = 1.0, velocity;
    "BridgeDieFX" => die_fx: Vec<BridgeEffect> = Vec::new(), die_effect_fx;
    "BridgeDieOCL" => die_ocl: Vec<BridgeEffect> = Vec::new(), die_effect_ocl;
}
