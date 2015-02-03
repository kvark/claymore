use cgmath;
use gfx;

use Id;

pub type Transform<S> = cgmath::Decomposed<
    S,
    cgmath::Vector3<S>,
    cgmath::Quaternion<S>
>;

pub type Node<S> = ::space::Node<Transform<S>>;
pub type Skeleton<S> = ::space::Skeleton<Transform<S>>;

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
    world: ::space::World<S, Transform<S>>,
    entities: Vec<Entity<S, Params>>,
    camera: Camera<S, P>,
    batch_context: gfx::batch::Context,
}

impl<S, P: cgmath::Projection<S>> Scene<S, P> {
    pub fn new(world: ::space::World<S, Transform<S>>,
               entities: Vec<Entity<S, Params>>, camera: Camera<S, P>,
               batch_con: gfx::batch::Context) -> Scene<S, P> {
        //TODO: validate
        Scene {
            world: world,
            entities: entities,
            camera: camera,
            batch_context: batch_con,
        }
    }
    pub fn draw<C: gfx::CommandBuffer>(&self, renderer: &mut gfx::Renderer<C>) {
        //TODO
    }
}
