use cgmath;
use gfx;

use Id;
use space;

pub type Transform<S> = cgmath::Decomposed<
    S,
    cgmath::Vector3<S>,
    cgmath::Quaternion<S>
>;

pub type Node<S> = space::Node<Transform<S>>;
pub type Skeleton<S> = space::Skeleton<Transform<S>>;

#[shader_param]
struct Params {
    #[name = "u_mvp"]
    mvp: [[f32; 4]; 4],
    #[name = "u_color"]
    color: [f32; 4],
}

pub struct Entity<S> {
    name: String,
    batch: gfx::batch::RefBatch<Params>,
    node: Id<Node<S>>,
    skeleton: Option<Id<Skeleton<S>>>,
}

pub struct Camera<S> {
    name: String,
    node: Id<Node<S>>,
}

pub struct Scene<S> {
    world: space::World<S, Transform<S>>,
    entities: Vec<Entity<S>>,
    camera: Camera<S>,
}

impl<S> Scene<S> {
    pub fn draw<C: gfx::CommandBuffer>(&self, renderer: &mut gfx::Renderer<C>) {
        //TODO
    }
}

pub fn load_json(path: &str) -> Scene<f32> {
    Scene {
        world: space::World::new(),
        entities: Vec::new(),
        camera: Camera {
            name: "cam".to_string(),
            node: Id(0),    //TODO
        },
    }
}
