//! Mesh chunk (`W3D_CHUNK_MESH`): geometry, materials and the collision tree.
//! Geometry follows `MeshGeometryClass::Load_W3D`; material data is kept as stored.

use crate::chunk::{c_string, chunks, finish, records, Chunk, Cursor, Read, Report, Truncated};

pub const FLAG_COLLISION_TYPE_MASK: u32 = 0x0000_0ff0;
pub const FLAG_GEOMETRY_TYPE_MASK: u32 = 0x00ff_0000;
pub const GEOMETRY_TYPE_SKIN: u32 = 0x0002_0000;

#[derive(Clone, Debug, Default)]
pub struct MeshHeader {
    pub version: u32,
    pub attributes: u32,
    pub mesh_name: String,
    pub container_name: String,
    pub num_triangles: u32,
    pub num_vertices: u32,
    pub num_materials: u32,
    pub num_damage_stages: u32,
    pub sort_level: i32,
    pub prelit_version: u32,
    pub future_counts: u32,
    pub vertex_channels: u32,
    pub face_channels: u32,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub sphere_center: [f32; 3],
    pub sphere_radius: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Triangle {
    pub vertices: [u32; 3],
    pub surface_type: u32,
    pub normal: [f32; 3],
    pub distance: f32,
}

#[derive(Clone, Debug, Default)]
pub struct VertexMaterialInfo {
    pub attributes: u32,
    /// RGB plus one pad byte each: ambient, diffuse, specular, emissive.
    pub colors: [[u8; 4]; 4],
    pub shininess: f32,
    pub opacity: f32,
    pub translucency: f32,
}

#[derive(Clone, Debug, Default)]
pub struct VertexMaterial {
    pub name: String,
    pub info: Option<VertexMaterialInfo>,
    pub mapper_args: [String; 2],
}

#[derive(Clone, Copy, Debug)]
pub struct TextureInfo {
    pub attributes: u16,
    pub anim_type: u16,
    pub frame_count: u32,
    pub frame_rate: f32,
}

#[derive(Clone, Debug, Default)]
pub struct TextureRef {
    pub name: String,
    pub info: Option<TextureInfo>,
}

#[derive(Clone, Debug, Default)]
pub struct TextureStage {
    pub texture_ids: Vec<u32>,
    pub texcoords: Vec<[f32; 2]>,
    pub per_face_texcoord_ids: Vec<[u32; 3]>,
}

#[derive(Clone, Debug, Default)]
pub struct MaterialPass {
    pub vertex_material_ids: Vec<u32>,
    pub shader_ids: Vec<u32>,
    /// Per-vertex diffuse, diffuse illumination and specular colours (RGBA).
    pub dcg: Vec<[u8; 4]>,
    pub dig: Vec<[u8; 4]>,
    pub scg: Vec<[u8; 4]>,
    pub stages: Vec<TextureStage>,
}

#[derive(Clone, Copy, Debug)]
pub struct AabNode {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub front_or_first_poly: u32,
    pub back_or_poly_count: u32,
}

#[derive(Clone, Debug, Default)]
pub struct AabTree {
    pub node_count: u32,
    pub poly_count: u32,
    pub poly_indices: Vec<u32>,
    pub nodes: Vec<AabNode>,
}

#[derive(Clone, Debug, Default)]
pub struct Mesh {
    pub header: MeshHeader,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub triangles: Vec<Triangle>,
    /// Bone of each vertex of a skin.
    pub bone_links: Vec<u16>,
    pub shade_indices: Vec<u32>,
    pub user_text: Option<String>,
    /// Pass, vertex material, shader and texture counts.
    pub material_info: Option<[u32; 4]>,
    pub shaders: Vec<[u8; 16]>,
    /// 12-byte PS2 shader records; the PC engine ignores them.
    pub ps2_shaders: Vec<[u8; 12]>,
    pub vertex_materials: Vec<VertexMaterial>,
    pub textures: Vec<TextureRef>,
    pub passes: Vec<MaterialPass>,
    pub aabtree: Option<AabTree>,
}

impl Mesh {
    /// `Container.Mesh`, or the mesh name alone.
    pub fn name(&self) -> String {
        if self.header.container_name.is_empty() {
            self.header.mesh_name.clone()
        } else {
            format!("{}.{}", self.header.container_name, self.header.mesh_name)
        }
    }

    /// The `SKIN` flag: set by the geometry type or by vertex influences.
    pub fn is_skin(&self) -> bool {
        !self.bone_links.is_empty()
            || (self.header.version >= super::version(4, 1)
                && self.header.attributes & FLAG_GEOMETRY_TYPE_MASK == GEOMETRY_TYPE_SKIN)
    }
}

fn header(c: &mut Cursor) -> Read<MeshHeader> {
    Ok(MeshHeader {
        version: c.u32()?,
        attributes: c.u32()?,
        mesh_name: c.name(16)?,
        container_name: c.name(16)?,
        num_triangles: c.u32()?,
        num_vertices: c.u32()?,
        num_materials: c.u32()?,
        num_damage_stages: c.u32()?,
        sort_level: c.i32()?,
        prelit_version: c.u32()?,
        future_counts: c.u32()?,
        vertex_channels: c.u32()?,
        face_channels: c.u32()?,
        min: c.floats()?,
        max: c.floats()?,
        sphere_center: c.floats()?,
        sphere_radius: c.f32()?,
    })
}

pub fn parse(data: &[u8], path: &[u32], report: &mut Report) -> Read<Mesh> {
    let list = chunks(data, path, report);
    let first = list.first().ok_or(Truncated)?;
    if first.id != 0x1f {
        return Err(Truncated);
    }
    let mut c = Cursor::new(first.data);
    let mut m = Mesh {
        header: header(&mut c)?,
        ..Mesh::default()
    };
    finish(&c, "mesh header", report);
    for chunk in &list[1..] {
        let d = chunk.data;
        match chunk.id {
            0x02 => m.vertices = records(d, "vertices", report, |c| c.floats()),
            0x03 | 0x04 => m.normals = records(d, "normals", report, |c| c.floats()),
            0x0c => m.user_text = Some(c_string(d)),
            0x0e => {
                m.bone_links = records(d, "influences", report, |c| {
                    let bone = c.u16()?;
                    c.bytes(6)?;
                    Ok(bone)
                })
            }
            0x20 => {
                m.triangles = records(d, "triangles", report, |c| {
                    Ok(Triangle {
                        vertices: c.uints()?,
                        surface_type: c.u32()?,
                        normal: c.floats()?,
                        distance: c.f32()?,
                    })
                })
            }
            0x22 => m.shade_indices = records(d, "shade indices", report, |c| c.u32()),
            0x28 => {
                let mut c = Cursor::new(d);
                m.material_info = Some(c.uints()?);
                finish(&c, "material info", report);
            }
            0x29 => m.shaders = records(d, "shaders", report, |c| c.array()),
            0x80 => m.ps2_shaders = records(d, "ps2 shaders", report, |c| c.array()),
            0x2a => each(chunk, path, 0x2b, report, |sub, p, report| {
                m.vertex_materials.push(vertex_material(sub, p, report)?);
                Ok(())
            })?,
            0x30 => each(chunk, path, 0x31, report, |sub, p, report| {
                m.textures.push(texture(sub, p, report)?);
                Ok(())
            })?,
            0x38 => {
                let pass = material_pass(chunk, &push(path, 0x38), report);
                m.passes.push(pass);
            }
            0x90 => m.aabtree = Some(aabtree(chunk, &push(path, 0x90), report)?),
            _ => report.skip(path, chunk),
        }
    }
    check_count(report, "vertices", m.vertices.len(), m.header.num_vertices);
    check_count(
        report,
        "triangles",
        m.triangles.len(),
        m.header.num_triangles,
    );
    // Influences make a mesh a skin; before 3.0 the root pivot was implicit.
    if m.header.version < super::version(3, 0) {
        for link in &mut m.bone_links {
            *link = link.wrapping_add(1);
        }
    }
    Ok(m)
}

fn check_count(report: &mut Report, what: &str, found: usize, declared: u32) {
    if found != declared as usize {
        report
            .damaged
            .push(format!("mesh {what}: {found} stored, {declared} declared"));
    }
}

fn push(path: &[u32], id: u32) -> Vec<u32> {
    let mut p = path.to_vec();
    p.push(id);
    p
}

/// Runs `f` on every `want` sub-chunk of `chunk`.
fn each(
    chunk: &Chunk,
    path: &[u32],
    want: u32,
    report: &mut Report,
    mut f: impl FnMut(&Chunk, &[u32], &mut Report) -> Read<()>,
) -> Read<()> {
    let p = push(path, chunk.id);
    for sub in chunks(chunk.data, &p, report) {
        if sub.id == want {
            f(&sub, &p, report)?;
        } else {
            report.skip(&p, &sub);
        }
    }
    Ok(())
}

fn vertex_material(chunk: &Chunk, path: &[u32], report: &mut Report) -> Read<VertexMaterial> {
    let p = push(path, chunk.id);
    let mut v = VertexMaterial::default();
    for sub in chunks(chunk.data, &p, report) {
        match sub.id {
            0x2c => v.name = c_string(sub.data),
            0x2d => {
                let mut c = Cursor::new(sub.data);
                v.info = Some(VertexMaterialInfo {
                    attributes: c.u32()?,
                    colors: [c.array()?, c.array()?, c.array()?, c.array()?],
                    shininess: c.f32()?,
                    opacity: c.f32()?,
                    translucency: c.f32()?,
                });
                finish(&c, "vertex material info", report);
            }
            0x2e => v.mapper_args[0] = c_string(sub.data),
            0x2f => v.mapper_args[1] = c_string(sub.data),
            _ => report.skip(&p, &sub),
        }
    }
    Ok(v)
}

fn texture(chunk: &Chunk, path: &[u32], report: &mut Report) -> Read<TextureRef> {
    let p = push(path, chunk.id);
    let mut t = TextureRef::default();
    for sub in chunks(chunk.data, &p, report) {
        match sub.id {
            0x32 => t.name = c_string(sub.data),
            0x33 => {
                let mut c = Cursor::new(sub.data);
                t.info = Some(TextureInfo {
                    attributes: c.u16()?,
                    anim_type: c.u16()?,
                    frame_count: c.u32()?,
                    frame_rate: c.f32()?,
                });
                finish(&c, "texture info", report);
            }
            _ => report.skip(&p, &sub),
        }
    }
    Ok(t)
}

fn material_pass(chunk: &Chunk, path: &[u32], report: &mut Report) -> MaterialPass {
    let mut m = MaterialPass::default();
    for sub in chunks(chunk.data, path, report) {
        let d = sub.data;
        match sub.id {
            0x39 => m.vertex_material_ids = records(d, "material ids", report, |c| c.u32()),
            0x3a => m.shader_ids = records(d, "shader ids", report, |c| c.u32()),
            0x3b => m.dcg = records(d, "dcg", report, |c| c.array()),
            0x3c => m.dig = records(d, "dig", report, |c| c.array()),
            0x3e => m.scg = records(d, "scg", report, |c| c.array()),
            0x48 => {
                let p = push(path, 0x48);
                let mut stage = TextureStage::default();
                for s in chunks(d, &p, report) {
                    match s.id {
                        0x49 => {
                            stage.texture_ids = records(s.data, "texture ids", report, |c| c.u32())
                        }
                        0x4a => {
                            stage.texcoords = records(s.data, "texcoords", report, |c| c.floats())
                        }
                        0x4b => {
                            stage.per_face_texcoord_ids =
                                records(s.data, "face texcoords", report, |c| c.uints())
                        }
                        _ => report.skip(&p, &s),
                    }
                }
                m.stages.push(stage);
            }
            _ => report.skip(path, &sub),
        }
    }
    m
}

fn aabtree(chunk: &Chunk, path: &[u32], report: &mut Report) -> Read<AabTree> {
    let mut t = AabTree::default();
    for sub in chunks(chunk.data, path, report) {
        match sub.id {
            0x91 => {
                let mut c = Cursor::new(sub.data);
                t.node_count = c.u32()?;
                t.poly_count = c.u32()?;
                c.bytes(24)?;
                finish(&c, "tree header", report);
            }
            0x92 => t.poly_indices = records(sub.data, "tree polys", report, |c| c.u32()),
            0x93 => {
                t.nodes = records(sub.data, "tree nodes", report, |c| {
                    Ok(AabNode {
                        min: c.floats()?,
                        max: c.floats()?,
                        front_or_first_poly: c.u32()?,
                        back_or_poly_count: c.u32()?,
                    })
                })
            }
            _ => report.skip(path, &sub),
        }
    }
    Ok(t)
}
