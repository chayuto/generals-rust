//! The bone queries the simulation makes of a model: `HTreeClass` transforms
//! and `W3DModelDraw`'s pristine bones and weapon barrels.
//!
//! The caller supplies the bone names and the animation (INI data of the draw
//! module, which is not read here).

use std::collections::BTreeMap;

use crate::w3d::anim::{BoundAnimation, Hierarchy};
use crate::w3d::objects::HLod;
use crate::wwmath::Matrix3D;

/// `W3DAssetManager`'s `scale_epsilon`.
const SCALE_EPSILON: f32 = 0.01;
const WEAPON_BONE_LIMIT: u32 = 99;

#[derive(Clone, Debug)]
struct Bone {
    name: String,
    parent: usize,
    base: Matrix3D,
}

#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub transform: Matrix3D,
    pub visible: bool,
}

/// `HTreeClass`.
#[derive(Clone, Debug)]
pub struct Skeleton {
    bones: Vec<Bone>,
    scale: f32,
}

impl Skeleton {
    pub fn new(h: &Hierarchy) -> Skeleton {
        let bones = h
            .pivots
            .iter()
            .map(|p| {
                let mut base = Matrix3D::IDENTITY;
                base.translate(p.translation);
                let rotation = Matrix3D::from_quat(crate::wwmath::Quat(p.rotation));
                Bone {
                    name: p.name.clone(),
                    parent: p.parent.max(0) as usize,
                    base: Matrix3D::mul(&base, &rotation),
                }
            })
            .collect();
        Skeleton { bones, scale: 1.0 }
    }

    pub fn num_bones(&self) -> usize {
        self.bones.len()
    }

    pub fn bone_name(&self, i: usize) -> &str {
        &self.bones[i].name
    }

    pub fn parent_index(&self, i: usize) -> usize {
        if i == 0 {
            0
        } else {
            self.bones[i].parent
        }
    }

    /// `Get_Bone_Index`: case-blind, and 0 (the root) when there is none.
    pub fn bone_index(&self, name: &str) -> usize {
        self.bones
            .iter()
            .position(|b| b.name.eq_ignore_ascii_case(name))
            .unwrap_or(0)
    }

    /// `HTreeClass::Scale`.
    pub fn scale(&mut self, factor: f32) {
        if factor == 1.0 {
            return;
        }
        for b in &mut self.bones {
            for r in &mut b.base.0 {
                r[3] *= factor;
            }
        }
        self.scale *= factor;
    }

    /// `Base_Update`, or `Anim_Update` when there is an animation.
    pub fn pose(&self, root: &Matrix3D, animation: Option<(&BoundAnimation, f32)>) -> Vec<Pose> {
        let mut out = vec![Pose {
            transform: *root,
            visible: true,
        }];
        for (i, b) in self.bones.iter().enumerate().skip(1) {
            let mut m = Matrix3D::mul(&out[b.parent].transform, &b.base);
            let mut visible = true;
            if let Some((anim, frame)) = animation {
                if i < anim.num_pivots() {
                    let t = anim.translation(i, frame);
                    m.translate([t[0] * self.scale, t[1] * self.scale, t[2] * self.scale]);
                    let q = anim.orientation(i, frame);
                    // `postMul` adds the old translation last, as `mul` does.
                    m = Matrix3D::mul(&m, &Matrix3D::from_quat(q));
                    visible = anim.visible(i, frame);
                }
            }
            out.push(Pose {
                transform: m,
                visible,
            });
        }
        out
    }
}

/// A render object with bones: an HLOD and its skeleton.
#[derive(Clone, Debug)]
pub struct Model {
    skeleton: Skeleton,
    /// `(name, bone)` of every sub-object: the LOD arrays, then the aggregates.
    sub_objects: Vec<(String, usize)>,
    first_lod_len: usize,
}

impl Model {
    /// `HLodClass(def)`. `exists` says whether the asset manager can create a
    /// sub-object; missing ones are left out, as in the engine.
    pub fn new(def: &HLod, hierarchy: &Hierarchy, exists: impl Fn(&str) -> bool) -> Model {
        let mut sub_objects = Vec::new();
        let mut first_lod_len = 0;
        for (i, lod) in def.lods.iter().enumerate() {
            for m in lod.models.iter().filter(|m| exists(&m.name)) {
                sub_objects.push((m.name.clone(), m.bone_index as usize));
            }
            if i == 0 {
                first_lod_len = sub_objects.len();
            }
        }
        for m in def.aggregates.iter().flat_map(|a| &a.models) {
            if exists(&m.name) {
                sub_objects.push((m.name.clone(), m.bone_index as usize));
            }
        }
        Model {
            skeleton: Skeleton::new(hierarchy),
            sub_objects,
            first_lod_len,
        }
    }

    pub fn skeleton(&self) -> &Skeleton {
        &self.skeleton
    }

    pub fn sub_objects(&self) -> &[(String, usize)] {
        &self.sub_objects
    }

    /// `Get_Sub_Object_By_Name`: the full name first, then the part after `.`.
    pub fn sub_object_index(&self, name: &str) -> Option<usize> {
        let eq = |n: &str| n.eq_ignore_ascii_case(name);
        self.sub_objects
            .iter()
            .position(|(n, _)| eq(n))
            .or_else(|| {
                self.sub_objects
                    .iter()
                    .position(|(n, _)| eq(n.split_once('.').map_or(n.as_str(), |s| s.1)))
            })
    }

    /// `ModelConditionInfo::validateCachedBones` for a model created at `scale`.
    /// `names` are the standard public bones followed by the draw module's.
    /// `animation` is the state's first animation and the frame to sample.
    pub fn pristine_bones<'n>(
        &self,
        scale: f32,
        animation: Option<(&BoundAnimation, f32)>,
        names: impl IntoIterator<Item = &'n str>,
    ) -> PristineBones {
        let mut skeleton = self.skeleton.clone();
        // The asset manager scales the model, then the draw module scales the root
        // transform too, so translations get the factor twice.
        if (scale - 1.0).abs() > SCALE_EPSILON {
            skeleton.scale(scale);
        }
        let mut root = Matrix3D::IDENTITY;
        root.scale(scale);
        let pose = skeleton.pose(&root, animation);
        let mut out = PristineBones::default();
        for name in names {
            self.add_bone(&mut out, &pose, &name.to_ascii_lowercase());
        }
        out
    }

    /// `doSingleBoneName`: `name`, `name01`..`name99`, else sub-objects so named.
    fn add_bone(&self, out: &mut PristineBones, pose: &[Pose], name: &str) {
        let bone = |n: &str| match self.skeleton.bone_index(n) {
            0 => None,
            i => Some(PristineBone {
                transform: pose[i].transform,
                bone_index: i as i32,
            }),
        };
        let sub_object = |n: &str| {
            let at = self.sub_object_index(n)?;
            let own = self.sub_objects[at].1;
            // The engine asks LOD 0 for the bone of flat index `at`. Past the end
            // of LOD 0 it reads out of bounds; the sub-object's bone stands in.
            let reported = if at < self.first_lod_len {
                self.sub_objects[at].1
            } else {
                own
            };
            Some(PristineBone {
                transform: pose.get(own).unwrap_or(&pose[0]).transform,
                bone_index: reported as i32,
            })
        };
        let mut numbered = |find: &dyn Fn(&str) -> Option<PristineBone>| {
            let mut found = false;
            if let Some(b) = find(name) {
                out.0.insert(name.to_string(), b);
                found = true;
            }
            for i in 1..=WEAPON_BONE_LIMIT {
                let n = format!("{name}{i:02}");
                let Some(b) = find(&n) else { break };
                out.0.insert(n, b);
                found = true;
            }
            found
        };
        if !name.is_empty() && name != "none" && !numbered(&bone) {
            numbered(&sub_object);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PristineBone {
    pub transform: Matrix3D,
    pub bone_index: i32,
}

/// Cached bones by lower-case name; lookups are case-sensitive, as name keys are.
#[derive(Clone, Debug, Default)]
pub struct PristineBones(BTreeMap<String, PristineBone>);

/// Bone names of one weapon slot, lower case as the INI parser stores them. An
/// empty string means the field was not given.
#[derive(Clone, Copy, Debug, Default)]
pub struct WeaponBones<'a> {
    pub fire_fx: &'a str,
    pub recoil: &'a str,
    pub muzzle_flash: &'a str,
    pub projectile_launch: &'a str,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Barrel {
    pub recoil_bone: i32,
    pub muzzle_flash_bone: i32,
    pub fx_bone: i32,
    pub projectile_offset: Matrix3D,
}

impl PristineBones {
    /// `findPristineBone`: door, turret, dock and launch bones are all read this way.
    pub fn find(&self, name: &str) -> Option<&PristineBone> {
        self.0.get(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&str, &PristineBone)> {
        self.0.iter().map(|(n, b)| (n.as_str(), b))
    }

    fn index(&self, name: &str) -> i32 {
        self.find(name).map_or(0, |b| b.bone_index)
    }

    /// `validateWeaponBarrelInfo` for one slot. The length is what
    /// `getBarrelCount` returns.
    pub fn barrels(&self, names: WeaponBones) -> Vec<Barrel> {
        let WeaponBones {
            fire_fx,
            recoil,
            muzzle_flash,
            projectile_launch,
        } = names;
        let mut out = Vec::new();
        if [fire_fx, recoil, muzzle_flash, projectile_launch]
            .iter()
            .all(|n| n.is_empty())
        {
            return out;
        }
        let numbered = |base: &str, i: u32| {
            if base.is_empty() {
                None
            } else {
                self.find(&format!("{base}{i:02}"))
            }
        };
        let index = |b: Option<&PristineBone>| b.map_or(0, |b| b.bone_index);
        let mut previous_fx = 0;
        for i in 1..=WEAPON_BONE_LIMIT {
            let mut b = Barrel {
                recoil_bone: index(numbered(recoil, i)),
                muzzle_flash_bone: index(numbered(muzzle_flash, i)),
                fx_bone: 0,
                projectile_offset: Matrix3D::IDENTITY,
            };
            if !fire_fx.is_empty() {
                b.fx_bone = index(numbered(fire_fx, i));
                if b.fx_bone == 0 && b.muzzle_flash_bone != 0 {
                    b.fx_bone = previous_fx;
                }
            }
            let launch = numbered(projectile_launch, i);
            if let Some(l) = launch {
                b.projectile_offset = l.transform;
            }
            if b.fx_bone == 0
                && b.recoil_bone == 0
                && b.muzzle_flash_bone == 0
                && index(launch) == 0
            {
                break;
            }
            previous_fx = b.fx_bone;
            out.push(b);
        }
        if out.is_empty() {
            let plain = |n: &str| if n.is_empty() { 0 } else { self.index(n) };
            let launch = if projectile_launch.is_empty() {
                None
            } else {
                self.find(projectile_launch)
            };
            let b = Barrel {
                recoil_bone: plain(recoil),
                muzzle_flash_bone: plain(muzzle_flash),
                fx_bone: plain(fire_fx),
                projectile_offset: launch.map_or(Matrix3D::IDENTITY, |l| l.transform),
            };
            if b.fx_bone != 0 || b.recoil_bone != 0 || b.muzzle_flash_bone != 0 || launch.is_some()
            {
                out.push(b);
            }
        }
        out
    }
}
