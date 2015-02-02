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
    #[name = "u_mvp"]
    pub mvp: [[f32; 4]; 4],
    #[name = "u_color"]
    pub color: [f32; 4],
}

pub struct Entity<S, P: gfx::shade::ShaderParam> {
    //name: String,
    pub batch: gfx::batch::RefBatch<P>,
    pub node: Id<Node<S>>,
    pub skeleton: Option<Id<Skeleton<S>>>,
}

pub struct Camera<S, P: cgmath::Projection<S>> {
    //name: String,
    pub node: Id<Node<S>>,
    pub projection: P,
}

pub struct Scene<S, P: cgmath::Projection<S>> {
    pub world: space::World<S, Transform<S>>,
    pub entities: Vec<Entity<S, Params>>,
    pub camera: Camera<S, P>,
}

impl<S, P: cgmath::Projection<S>> Scene<S, P> {
    pub fn draw<C: gfx::CommandBuffer>(&self, renderer: &mut gfx::Renderer<C>) {
        //TODO
    }
}

#[derive(Debug)]
pub enum LoadJsonError {
    Parse(load::Error),
    NoCamera,
    MissingNode(String),
}

pub type SceneJson = Scene<f32, cgmath::PerspectiveFov<f32, cgmath::Rad<f32>>>;

pub fn load_json(path: &str) -> Result<SceneJson, LoadJsonError> {
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
        let node = match world.find_node(cam.name.as_slice()) {
            Some(n) => n,
            None => return Err(LoadJsonError::MissingNode(cam.name.clone())),
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
            node: node,
            projection: proj,
        }
    };
    Ok(Scene {
        world: world,
        entities: Vec::new(),
        camera: camera,
    })
}
