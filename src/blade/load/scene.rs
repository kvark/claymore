use cgmath;
use gfx;
use super::reflect as json;

static VERTEX_SRC: gfx::ShaderSource<'static> = shaders! {
    glsl_150: b"#version 150 core

    in vec3 a_Position;
    //in vec2 a_TexCoord;
    //out vec2 v_TexCoord;

    uniform mat4 u_Transform;

    void main() {
        //v_TexCoord = a_TexCoord;
        gl_Position = u_Transform * vec4(a_Position, 1.0);
    }
    "
};

static FRAGMENT_SRC: gfx::ShaderSource<'static> = shaders! {
    glsl_150: b"#version 150 core

    //in vec2 v_TexCoord;
    out vec4 o_Color;

    uniform vec4 u_Color;

    void main() {
        o_Color = u_Color;
    }
    "
};

#[derive(Debug)]
pub enum Error {
    NoCamera,
    MissingNode(String),
    Program(gfx::ProgramError),
    Mesh(String, super::mesh::Error),
    Batch(String, gfx::batch::BatchError),
}

pub type SceneJson = ::scene::Scene<f32,
    cgmath::PerspectiveFov<f32, cgmath::Rad<f32>>
>;

pub fn load<'a, D: gfx::Device>(raw: json::Scene,
            context: &mut super::Context<D>)
            -> Result<SceneJson, Error> {
    use gfx::DeviceHelper;
    fn read_space<S: cgmath::BaseFloat>(space: &json::Space<S>)
                  -> ::scene::Transform<S> {
        cgmath::Decomposed {
            scale: space.scale,
            rot: {
                let (x, y, z, w) = space.rot;
                cgmath::Quaternion::new(w, x, y, z)
            },
            disp: {
                let (x, y, z) = space.pos;
                cgmath::Vector3::new(x, y, z)
            },
        }
    }
    fn populate_world(world: &mut ::space::World<f32, ::scene::Transform<f32>>,
                      raw_nodes: &[json::Node],
                      parent: ::space::Parent<::scene::Transform<f32>>) {
        for n in raw_nodes.iter() {
            let space = read_space(&n.space);
            let nid = world.add_node(n.name.clone(), parent, space);
            populate_world(world,
                n.children.as_slice(),
                ::space::Parent::Domestic(nid)
            );
        }
    }
    // create world
    let mut world = ::space::World::new();
    populate_world(&mut world, raw.nodes.as_slice(), ::space::Parent::None);
    // read camera
    let camera = {
        use std::num::Float;
        let cam = match raw.cameras.first() {
            Some(c) => c,
            None => return Err(Error::NoCamera),
        };
        let node = match world.find_node(cam.node.as_slice()) {
            Some(n) => n,
            None => return Err(Error::MissingNode(cam.node.clone())),
        };
        let (fovx, fovy) = cam.angle;
        let (near, far) = cam.range;
        let proj = cgmath::PerspectiveFov {
            fovy: cgmath::rad(fovy),
            aspect: fovx.tan() / fovy.tan(),
            near: near,
            far: far,
        };
        ::scene::Camera {
            name: cam.name.clone(),
            node: node,
            projection: proj,
        }
    };
    // read entities
    let program = match context.device.link_program(
            VERTEX_SRC.clone(), FRAGMENT_SRC.clone()) {
        Ok(p) => p,
        Err(e) => return Err(Error::Program(e)),
    };
    let mut entities = Vec::new();
    let mut batch_con = gfx::batch::Context::new();
    for ent in raw.entities.iter() {
        let node = match world.find_node(ent.node.as_slice()) {
            Some(n) => n,
            None => return Err(Error::MissingNode(ent.node.clone())),
        };
        let (mesh, mut slice) = match context.request_mesh(ent.mesh.as_slice()) {
            Ok(success) => success,
            Err(e) => return Err(Error::Mesh(ent.mesh.clone(), e)),
        };
        let (ra, rb) = ent.range;
        slice.start = ra as gfx::VertexCount;
        slice.end = rb as gfx::VertexCount;
        let draw_state = gfx::DrawState::new().depth(
            gfx::state::Comparison::LessEqual,
            true
        );
        let batch = match batch_con.make_batch(&program, &mesh, slice, &draw_state) {
            Ok(b) => b,
            Err(e) => return Err(Error::Batch(ent.mesh.clone(), e)),
        };
        entities.push(::scene::Entity {
            name: ent.mesh.clone(),
            batch: batch,
            node: node,
            skeleton: None, //TODO
        });
    }
    // done
    Ok(::scene::Scene::new(world, entities, camera, batch_con))
}
