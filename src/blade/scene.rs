use std::num::ToPrimitive;
use cgmath;
use gfx;

use Id;

pub type Transform<S> = cgmath::Decomposed<
    S,
    cgmath::Vector3<S>,
    cgmath::Quaternion<S>
>;

pub type World<S> = ::space::World<S, Transform<S>>;
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

impl Params {
    pub fn new() -> Params {
        Params {
            mvp: [[0.0; 4]; 4],
            color: [0.0; 4],
        }
    }
}

pub struct Entity<S, P: gfx::shade::ShaderParam> {
    pub name: String,
    pub batch: gfx::batch::RefBatch<P>,
    pub params: Params,
    pub node: Id<Node<S>>,
    pub skeleton: Option<Id<Skeleton<S>>>,
}

pub struct Camera<S, P> {
    pub name: String,
    pub node: Id<Node<S>>,
    pub projection: P,
}

pub struct Scene<S, P> {
    pub entities: Vec<Entity<S, Params>>,
    pub camera: Camera<S, P>,
    pub batch_context: gfx::batch::Context,
}

impl<
    S: 'static + ToPrimitive + cgmath::BaseFloat,
    P: cgmath::Projection<S>
> Scene<S, P> {
    pub fn new(entities: Vec<Entity<S, Params>>, camera: Camera<S, P>,
               batch_con: gfx::batch::Context) -> Scene<S, P> {
        //TODO: validate
        Scene {
            entities: entities,
            camera: camera,
            batch_context: batch_con,
        }
    }

    pub fn update(&mut self, world: &World<S>) {
        use cgmath::{FixedArray, Matrix, ToMatrix4, Transform};
        let cam_world = world.get_node(self.camera.node).world;
        let cam_inverse = self.camera.projection.to_matrix4().mul_m(
            &cam_world.invert().unwrap().to_matrix4());
        for ent in self.entities.iter_mut() {
            let transform = world.get_node(ent.node).world.to_matrix4();
            let m = cam_inverse.mul_m(&transform).into_fixed();
            ent.params.color = [0.0, 1.0, 1.0, 1.0];
            //ent.params.mvp = m;   //can't cast
            // ugly workaround
            for i in (0..4) {
                for j in (0.. 4) {
                    ent.params.mvp[i][j] = m[i][j].to_f32().unwrap();
                }
            }
        }
    }

    pub fn draw<C: gfx::CommandBuffer>(&self, renderer: &mut gfx::Renderer<C>,
                frame: &gfx::Frame) {
        for ent in self.entities.iter() {
            let batch = (&ent.batch, &ent.params, &self.batch_context);
            renderer.draw(&batch, frame).unwrap();
        }
    }
}
