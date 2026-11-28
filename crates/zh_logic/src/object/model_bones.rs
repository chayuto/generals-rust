//! The bones logic reads from a drawable's models: `ModelConditionInfo`'s pristine
//! bones (`validateCachedBones`), and `getPristineBonePositions`, `getBarrelCount`
//! and `getProjectileLaunchOffset` of `Drawable` over its `W3DModelDraw` modules.
//!
//! The catalog resolves the bones of every condition state of the loaded templates
//! up front. A state the templates did not have then (a map override naming other
//! bones or models) has none.
//!
//! This is device code with no packet. Golden test:
//! `crates/zh_logic/tests/golden_model_bones.rs`.

use std::collections::BTreeMap;

use zh_assets::model::{Model, PristineBones, Skeleton, WeaponBones};
use zh_assets::w3d::anim::{Animation, BoundAnimation, Hierarchy};
use zh_assets::w3d::objects::HLod;
use zh_assets::w3d::{self, Object as W3dObject};
use zh_assets::wwmath::Matrix3D as AssetMatrix;
use zh_common::math::{gm_cosf, gm_sinf, Coord3D, Matrix3D};
use zh_common::types::{Int, Real};

use crate::arena::ModuleDataId;
use crate::combat::turret_ai::WhichTurretType;
use crate::gamelogic::global_data::{TimeOfDay, Weather};
use crate::object::model_draw::{
    add_public_bone, ModelConditionBoneSpec, ModelDrawModuleData, ModelDrawSpec,
};
use crate::object::thing_template::ThingTemplate;
use crate::rules::model_state::{ModelConditionFlagType, ModelConditionFlags};
use crate::sim::{ObjectRecord, RulesRegistry};

/// `MAX_BONE_GET`.
const MAX_BONE_GET: usize = 64;

/// The pristine bones of one validated condition state.
#[derive(Debug, Clone)]
struct StateBones {
    animation: Option<String>,
    final_frame: bool,
    scale_bits: u32,
    public_bones: Vec<String>,
    extra_public_bones: Vec<String>,
    bones: PristineBones,
}

impl StateBones {
    fn is_for(&self, spec: &ModelDrawSpec, state: &ModelConditionBoneSpec, scale: Real) -> bool {
        self.scale_bits == scale.to_bits()
            && self.animation == state.first_animation
            && self.final_frame == state.pristine_bones_in_final_frame()
            && self.public_bones == state.public_bones
            && self.extra_public_bones == spec.extra_public_bones
    }
}

/// The pristine bones of the models, by lower-case model name.
#[derive(Debug, Clone, Default)]
pub struct ModelBoneCatalog {
    models: BTreeMap<String, Vec<StateBones>>,
}

impl ModelBoneCatalog {
    /// Reads the models the templates' draw modules name. `read` returns the bytes
    /// of a file by its archive path.
    pub fn load(rules: &RulesRegistry, mut read: impl FnMut(&str) -> Option<Vec<u8>>) -> Self {
        let mut assets = Assets::default();
        let mut catalog = Self::default();
        let standard = &rules.global_data.standard_public_bones;
        for template in rules.thing_factory.templates_in_first_order() {
            let scale = template.get_asset_scale();
            for (_, spec) in draw_specs(rules, template) {
                for state in &spec.condition_states {
                    if state.model_name.is_empty() {
                        continue;
                    }
                    let entries = catalog.models.entry(state.model_name.clone()).or_default();
                    if entries.iter().any(|e| e.is_for(spec, state, scale)) {
                        continue;
                    }
                    let mut names = state.public_bones.clone();
                    for extra in &spec.extra_public_bones {
                        add_public_bone(&mut names, extra);
                    }
                    let names = standard.iter().chain(&names).map(String::as_str);
                    entries.push(StateBones {
                        animation: state.first_animation.clone(),
                        final_frame: state.pristine_bones_in_final_frame(),
                        scale_bits: scale.to_bits(),
                        public_bones: state.public_bones.clone(),
                        extra_public_bones: spec.extra_public_bones.clone(),
                        bones: assets.pristine_bones(state, scale, names, &mut read),
                    });
                }
            }
        }
        catalog
    }

    /// The pristine bones of a condition state at a drawable scale; `None` for a
    /// state with no model.
    pub fn state_bones(
        &self,
        spec: &ModelDrawSpec,
        state: &ModelConditionBoneSpec,
        scale: Real,
    ) -> Option<&PristineBones> {
        self.models
            .get(&state.model_name)?
            .iter()
            .find(|e| e.is_for(spec, state, scale))
            .map(|e| &e.bones)
    }
}

/// The `W3DModelDraw` module data of a template, in draw module order.
pub fn draw_specs<'a>(
    rules: &'a RulesRegistry,
    template: &'a ThingTemplate,
) -> impl Iterator<Item = (ModuleDataId, &'a ModelDrawSpec)> + 'a {
    template
        .get_draw_module_info()
        .nuggets()
        .iter()
        .filter_map(|n| {
            let data = rules.module_data_store.get(n.data_id)?;
            let draw = data.as_any().downcast_ref::<ModelDrawModuleData>()?;
            Some((n.data_id, &draw.spec))
        })
}

/// `m_attachToDrawableBoneOffset` of the module data that have computed it. The
/// reference computes it once, from whichever drawable asks first, and keeps it.
pub type AttachBoneOffsets = BTreeMap<ModuleDataId, Coord3D>;

/// What `getProjectileLaunchOffset` writes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectileLaunch {
    /// `None` when no module has a barrel for the slot (the call returns false).
    pub launch: Option<Matrix3D>,
    pub turret_rot_pos: Coord3D,
    pub turret_pitch_pos: Coord3D,
}

/// A drawable's model draw modules in some condition state.
pub struct DrawableBones<'a> {
    catalog: &'a ModelBoneCatalog,
    specs: Vec<(ModuleDataId, &'a ModelDrawSpec)>,
    scale: Real,
    conditions: ModelConditionFlags,
}

impl<'a> DrawableBones<'a> {
    /// `None` without loaded models.
    pub fn new(
        rules: &'a RulesRegistry,
        template: &'a ThingTemplate,
        conditions: ModelConditionFlags,
    ) -> Option<Self> {
        Some(Self {
            catalog: rules.model_bones.as_deref()?,
            specs: draw_specs(rules, template).collect(),
            scale: template.get_asset_scale(),
            conditions,
        })
    }

    /// `findBestInfo` of a module and the bones of that state.
    fn state(
        &self,
        spec: &'a ModelDrawSpec,
    ) -> Option<(&'a ModelConditionBoneSpec, Option<&'a PristineBones>)> {
        let state = &spec.condition_states[spec.find_best_info(&self.conditions)?];
        Some((state, self.catalog.state_bones(spec, state, self.scale)))
    }

    /// `Drawable::getPristineBonePositions`: model space transforms of `prefix`
    /// (`start_index` 0) or of `prefix01`.. up to the first one missing, over every
    /// module until `max_bones` are found.
    pub fn pristine_bone_transforms(
        &self,
        prefix: &str,
        start_index: Int,
        max_bones: usize,
    ) -> Vec<Matrix3D> {
        let mut out = Vec::new();
        for &(_, spec) in &self.specs {
            let wanted = max_bones.saturating_sub(out.len()).min(MAX_BONE_GET);
            let Some((_, Some(bones))) = self.state(spec) else {
                continue;
            };
            let end = if start_index == 0 { 0 } else { 99 };
            let mut found = 0;
            for i in start_index..=end {
                if found >= wanted {
                    break;
                }
                let name = if i == 0 {
                    prefix.to_ascii_lowercase()
                } else {
                    format!("{prefix}{i:02}").to_ascii_lowercase()
                };
                let Some(bone) = bones.find(&name) else {
                    break;
                };
                out.push(matrix(&bone.transform));
                found += 1;
            }
        }
        out
    }

    /// `Drawable::getBarrelCount`. The reference asks the module's current state,
    /// which a transition state can differ from; this asks the best state.
    pub fn barrel_count(&self, wslot: usize) -> Int {
        for &(_, spec) in &self.specs {
            let Some((state, Some(bones))) = self.state(spec) else {
                continue;
            };
            let count = bones.barrels(weapon_bones(state, wslot)).len();
            if count != 0 {
                return count as Int;
            }
        }
        0
    }

    /// `Drawable::getProjectileLaunchOffset`.
    pub fn projectile_launch_offset(
        &self,
        wslot: usize,
        barrel: Int,
        turret: WhichTurretType,
        attach_offsets: &mut AttachBoneOffsets,
    ) -> ProjectileLaunch {
        let mut out = ProjectileLaunch {
            launch: None,
            turret_rot_pos: Coord3D::default(),
            turret_pitch_pos: Coord3D::default(),
        };
        for &(id, spec) in &self.specs {
            let Some((state, bones)) = self.state(spec) else {
                continue;
            };
            let barrels = bones.map_or_else(Vec::new, |b| b.barrels(weapon_bones(state, wslot)));
            let tur = usize::try_from(turret as i32)
                .ok()
                .map(|t| &state.turrets[t]);

            let index = usize::try_from(barrel)
                .ok()
                .filter(|&i| i < barrels.len())
                .unwrap_or(0);
            if let Some(b) = barrels.get(index) {
                let mut launch = matrix(&b.projectile_offset);
                if let Some(t) = tur {
                    pre_rotate_z(&mut launch, t.art_angle);
                    pre_rotate_y(&mut launch, -t.art_pitch);
                }
                out.launch = Some(launch);
            }

            out.turret_rot_pos = Coord3D::default();
            out.turret_pitch_pos = Coord3D::default();
            if let Some(t) = tur {
                // `AttachToBoneInAnotherModule` carries the turret with its bone.
                let attach = if spec.attach_to_drawable_bone.is_empty() {
                    Coord3D::default()
                } else {
                    *attach_offsets.entry(id).or_insert_with(|| {
                        let found =
                            self.pristine_bone_transforms(&spec.attach_to_drawable_bone, 0, 1);
                        found
                            .first()
                            .map_or_else(Coord3D::default, Matrix3D::get_translation)
                    })
                };
                // `findPristineBonePos`: zero without the bone.
                let pos = |name: &str| {
                    let p = bones
                        .filter(|_| !name.is_empty())
                        .and_then(|b| b.find(name))
                        .map_or_else(Coord3D::default, |b| matrix(&b.transform).get_translation());
                    if spec.attach_to_drawable_bone.is_empty() {
                        p
                    } else {
                        Coord3D::new(p.x + attach.x, p.y + attach.y, p.z + attach.z)
                    }
                };
                out.turret_rot_pos = pos(&t.angle_bone);
                out.turret_pitch_pos = pos(&t.pitch_bone);
            }
            if out.launch.is_some() {
                break;
            }
        }
        out
    }
}

/// The drawable bones of an object, in the condition state of its drawable: the
/// object's flags, plus night and snow where models follow the weather
/// (`friend_bindToDrawable`). A map object's own time and weather are not kept.
pub fn object_bones<'a>(rules: &'a RulesRegistry, object: &ObjectRecord) -> Option<DrawableBones<'a>> {
    let template = object
        .thing
        .template_override()
        .get_final(&rules.thing_factory)?;
    let global = &rules.global_data;
    let mut conditions = object.model_condition.clone();
    if global.force_models_to_follow_time_of_day != 0 && global.time_of_day == TimeOfDay::Night {
        conditions.set(ModelConditionFlagType::Night.as_i32(), 1);
    }
    if global.force_models_to_follow_weather != 0 && global.weather == Weather::Snowy {
        conditions.set(ModelConditionFlagType::Snow.as_i32(), 1);
    }
    DrawableBones::new(rules, template, conditions)
}

fn weapon_bones(state: &ModelConditionBoneSpec, wslot: usize) -> WeaponBones<'_> {
    let w = &state.weapons[wslot];
    WeaponBones {
        fire_fx: &w.fire_fx,
        recoil: &w.recoil,
        muzzle_flash: &w.muzzle_flash,
        projectile_launch: &w.launch,
    }
}

fn matrix(m: &AssetMatrix) -> Matrix3D {
    Matrix3D { rows: m.0 }
}

/// `Matrix3D::Pre_Rotate_Z`: the whole matrix, translation included.
fn pre_rotate_z(m: &mut Matrix3D, theta: Real) {
    let c = gm_cosf(theta);
    let s = gm_sinf(theta);
    for col in 0..4 {
        let t1 = m.rows[0][col];
        let t2 = m.rows[1][col];
        m.rows[0][col] = c * t1 - s * t2;
        m.rows[1][col] = s * t1 + c * t2;
    }
}

/// `Matrix3D::Pre_Rotate_Y`.
fn pre_rotate_y(m: &mut Matrix3D, theta: Real) {
    let c = gm_cosf(theta);
    let s = gm_sinf(theta);
    for col in 0..4 {
        let t1 = m.rows[0][col];
        let t2 = m.rows[2][col];
        m.rows[0][col] = c * t1 + s * t2;
        m.rows[2][col] = -s * t1 + c * t2;
    }
}

/// The prototypes of the files read so far, by lower-case name.
#[derive(Default)]
struct Assets {
    files: BTreeMap<String, ()>,
    /// Render objects without bones: meshes, boxes and emitters.
    plain: BTreeMap<String, ()>,
    hlods: BTreeMap<String, HLod>,
    hierarchies: BTreeMap<String, Hierarchy>,
    animations: BTreeMap<String, Animation>,
}

impl Assets {
    /// `Load_3D_Assets` of `<stem>.w3d`, once; the first prototype of a name stays.
    fn load_file(&mut self, stem: &str, read: &mut impl FnMut(&str) -> Option<Vec<u8>>) {
        if self.files.insert(stem.to_owned(), ()).is_some() {
            return;
        }
        let Some(bytes) = read(&format!("Art\\W3D\\{stem}.w3d")) else {
            return;
        };
        let lower = str::to_ascii_lowercase;
        for object in w3d::parse(&bytes).objects {
            match object {
                W3dObject::Mesh(mesh) => {
                    self.plain.insert(lower(&mesh.name()), ());
                }
                W3dObject::Box(b) => {
                    self.plain.insert(lower(&b.name), ());
                }
                W3dObject::Emitter(e) => {
                    self.plain.insert(lower(&e.name), ());
                }
                W3dObject::HLod(hlod) => {
                    self.hlods.entry(lower(&hlod.name)).or_insert(hlod);
                }
                W3dObject::Hierarchy(tree) => {
                    self.hierarchies.entry(lower(&tree.name)).or_insert(tree);
                }
                W3dObject::Animation(anim) => {
                    let name = format!("{}.{}", anim.hierarchy_name, anim.name);
                    self.animations.entry(lower(&name)).or_insert(anim);
                }
            }
        }
    }

    fn exists(&self, name: &str) -> bool {
        self.plain.contains_key(name) || self.hlods.contains_key(name)
    }

    /// Loads on demand the file a render object comes from: the part before `.`.
    fn demand(&mut self, name: &str, read: &mut impl FnMut(&str) -> Option<Vec<u8>>) {
        if !self.exists(name) {
            self.load_file(name.split('.').next().unwrap_or(name), read);
        }
    }

    /// The hierarchy of a name, from `<name>.w3d` when not read yet.
    fn skeleton(
        &mut self,
        name: &str,
        read: &mut impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> Option<&Hierarchy> {
        if !self.hierarchies.contains_key(name) {
            self.load_file(name, read);
        }
        self.hierarchies.get(name)
    }

    /// `Get_HAnim`: an animation comes from the file named after its second part.
    /// It has a node for each pivot of its own hierarchy, and does not load
    /// without that.
    fn animation(
        &mut self,
        name: &str,
        read: &mut impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> Option<(Animation, usize)> {
        if !self.animations.contains_key(name) {
            self.load_file(name.split_once('.')?.1, read);
        }
        let animation = self.animations.get(name)?.clone();
        let tree = animation.hierarchy_name.to_ascii_lowercase();
        let nodes = Skeleton::new(self.skeleton(&tree, read)?).num_bones();
        Some((animation, nodes))
    }

    /// `validateCachedBones` on a model created at `scale`. Only an HLOD has bones.
    fn pristine_bones<'n>(
        &mut self,
        state: &ModelConditionBoneSpec,
        scale: Real,
        names: impl IntoIterator<Item = &'n str>,
        read: &mut impl FnMut(&str) -> Option<Vec<u8>>,
    ) -> PristineBones {
        let name = state.model_name.as_str();
        self.demand(name, read);
        let Some(hlod) = self.hlods.get(name).cloned() else {
            return PristineBones::default();
        };
        for sub in hlod
            .lods
            .iter()
            .chain(&hlod.aggregates)
            .flat_map(|a| &a.models)
        {
            self.demand(&sub.name.to_ascii_lowercase(), read);
        }
        let tree = hlod.hierarchy_name.to_ascii_lowercase();
        let animation = state
            .first_animation
            .as_deref()
            .and_then(|a| self.animation(a, read));
        let Some(hierarchy) = self.skeleton(&tree, read).cloned() else {
            return PristineBones::default();
        };
        let model = Model::new(&hlod, &hierarchy, |n| self.exists(&n.to_ascii_lowercase()));
        let bound = animation
            .as_ref()
            .map(|(a, nodes)| BoundAnimation::new(a, *nodes));
        let frame = |a: &BoundAnimation| {
            if state.pristine_bones_in_final_frame() {
                a.num_frames().saturating_sub(1) as Real
            } else {
                0.0
            }
        };
        model.pristine_bones(scale, bound.as_ref().map(|a| (a, frame(a))), names)
    }
}
