use cgmath;
use gfx;

use Id;
use load;
use space;

pub type Transform<S> = cgmath::Decomposed<
    S,
    cgmath::Vector3<S>,
    cgmath::Quaternion<S>
>;

pub type Node<S> = space::Node<Transform<S>>;
pub type Skeleton<S> = space::Skeleton<Transform<S>>;

#[derive(Copy)]
#[shader_param]
pub struct Params {
    #[name = "u_Transform"]
    pub mvp: [[f32; 4]; 4],
    #[name = "u_Color"]
    pub color: [f32; 4],
}

pub struct Entity<S, P: gfx::shade::ShaderParam> {
    pub name: String,
    pub batch: gfx::batch::RefBatch<P>,
    pub node: Id<Node<S>>,
    pub skeleton: Option<Id<Skeleton<S>>>,
}

pub struct Camera<S, P: cgmath::Projection<S>> {
    pub name: String,
    pub node: Id<Node<S>>,
    pub projection: P,
}

pub struct Scene<S, P: cgmath::Projection<S>> {
    world: space::World<S, Transform<S>>,
    entities: Vec<Entity<S, Params>>,
    camera: Camera<S, P>,
    batch_context: gfx::batch::Context,
}

impl<S, P: cgmath::Projection<S>> Scene<S, P> {
    pub fn draw<C: gfx::CommandBuffer>(&self, renderer: &mut gfx::Renderer<C>) {
        //TODO
    }
}

static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
    glsl_150: b"#version 150 core

    in vec3 a_Pos;
    in vec2 a_TexCoord;
    out vec2 v_TexCoord;

    uniform mat4 u_Transform;

    void main() {
        v_TexCoord = a_TexCoord;
        gl_Position = u_Transform * vec4(a_Pos, 1.0);
    }
    "
};

static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
    glsl_150: b"#version 150 core

    in vec2 v_TexCoord;
    out vec4 o_Color;

    uniform vec4 color;

    void main() {
        o_Color = color;
    }
    "
};

#[derive(Debug)]
pub enum LoadMeshError {
    Path,
}

pub fn load_k3mesh<D: gfx::Device>(path: &str, device: &mut D)
        -> Result<gfx::Mesh, LoadMeshError> {
    Err(LoadMeshError::Path)
}

#[derive(Debug)]
pub enum LoadJsonError {
    Parse(load::Error),
    NoCamera,
    MissingNode(String),
    Program(gfx::ProgramError),
    Mesh(String, LoadMeshError),
    Batch(String, gfx::batch::BatchError),
}

pub type SceneJson = Scene<f32, cgmath::PerspectiveFov<f32, cgmath::Rad<f32>>>;

pub fn load_json<D: gfx::Device>(path: &str, device: &mut D)
        -> Result<SceneJson, LoadJsonError> {
    use gfx::DeviceHelper;
    fn read_space<S: cgmath::BaseFloat>(raw: &load::Space<S>) -> Transform<S> {
        cgmath::Decomposed {
            scale: raw.scale,
            rot: {
                let (x, y, z, w) = raw.rot;
                cgmath::Quaternion::new(w, x, y, z)
            },
            disp: {
                let (x, y, z) = raw.pos;
                cgmath::Vector3::new(x, y, z)
            },
        }
    }
    fn populate_world(
            world: &mut space::World<f32, Transform<f32>>,
            raw_nodes: &[load::Node],
            parent: space::Parent<Transform<f32>>) {
        for n in raw_nodes.iter() {
            let space = read_space(&n.space);
            let nid = world.add_node(n.name.clone(), parent, space);
            populate_world(world,
                n.children.as_slice(),
                space::Parent::Domestic(nid));
        }
    }
    // parse Json
    let raw = match load::json(format!("{}.json", path).as_slice()) {
        Ok(json) => json,
        Err(e) => return Err(LoadJsonError::Parse(e))
    };
    // create world
    let mut world = space::World::new();
    populate_world(&mut world, raw.nodes.as_slice(), space::Parent::None);
    // read camera
    let camera = {
        use std::num::Float;
        let cam = match raw.cameras.first() {
            Some(c) => c,
            None => return Err(LoadJsonError::NoCamera),
        };
        let node = match world.find_node(cam.node.as_slice()) {
            Some(n) => n,
            None => return Err(LoadJsonError::MissingNode(cam.node.clone())),
        };
        let (fovx, fovy) = cam.angle;
        let (near, far) = cam.range;
        let proj = cgmath::PerspectiveFov {
            fovy: cgmath::rad(fovy),
            aspect: fovx.tan() / fovy.tan(),
            near: near,
            far: far,
        };
        Camera {
            name: cam.name.clone(),
            node: node,
            projection: proj,
        }
    };
    // read entities
    let program = match device.link_program(VERTEX_SRC.clone(), FRAGMENT_SRC.clone()) {
        Ok(p) => p,
        Err(e) => return Err(LoadJsonError::Program(e)),
    };
    let mut entities = Vec::new();
    let mut batch_con = gfx::batch::Context::new();
    for ent in raw.entities.iter() {
        let node = match world.find_node(ent.node.as_slice()) {
            Some(n) => n,
            None => return Err(LoadJsonError::MissingNode(ent.node.clone())),
        };
        let mesh = match load_k3mesh(ent.mesh.as_slice(), device) {
            Ok(m) => m,
            Err(e) => return Err(LoadJsonError::Mesh(ent.mesh.clone(), e)),
        };
        let slice = {
            let (ra, rb) = ent.range;
            gfx::Slice {
                start: ra as u32,
                end: rb as u32,
                prim_type: gfx::PrimitiveType::TriangleList,
                kind: gfx::SliceKind::Vertex, //TODO
            }
        };
        let draw_state = gfx::DrawState::new().depth(
            gfx::state::Comparison::LessEqual,
            true
        );
        let batch = match batch_con.make_batch(&program, &mesh, slice, &draw_state) {
            Ok(b) => b,
            Err(e) => return Err(LoadJsonError::Batch(ent.mesh.clone(), e)),
        };
        entities.push(Entity {
            name: ent.mesh.clone(),
            batch: batch,
            node: node,
            skeleton: None, //TODO
        });
    }
    // done
    Ok(Scene {
        world: world,
        entities: entities,
        camera: camera,
        batch_context: batch_con,
    })
}
