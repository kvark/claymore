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

pub trait ShaderParam<S>: gfx::shade::ShaderParam + Sized {
    fn set_transform(&mut self,
                     camera: &cgmath::Matrix4<S>,
                     model: &cgmath::Matrix4<S>,
                     view_transform: &Transform<S>);
}

pub struct Entity<S, P: gfx::shade::ShaderParam> {
    pub name: String,
    pub batch: gfx::batch::RefBatch<P>,
    pub params: P,
    pub node: Id<Node<S>>,
    pub skeleton: Option<Id<Skeleton<S>>>,
}

pub struct Camera<S, R> {
    pub name: String,
    pub node: Id<Node<S>>,
    pub projection: R,
}

pub struct Scene<S, P: ShaderParam<S>, R> {
    pub entities: Vec<Entity<S, P>>,
    pub camera: Camera<S, R>,
    pub batch_context: gfx::batch::Context,
}

impl<
    S: 'static + ToPrimitive + cgmath::BaseFloat,
    P: ShaderParam<S>,
    R: cgmath::Projection<S>
> Scene<S, P, R> {
    pub fn new(entities: Vec<Entity<S, P>>, camera: Camera<S, R>,
               batch_con: gfx::batch::Context) -> Scene<S, P, R> {
        //TODO: validate
        Scene {
            entities: entities,
            camera: camera,
            batch_context: batch_con,
        }
    }

    pub fn update(&mut self, world: &World<S>) {
        use cgmath::{Matrix, ToMatrix4, Transform};
        let cam_inverse = world.get_node(self.camera.node).world
                               .invert().unwrap();
        let projection = self.camera.projection.to_matrix4()
                             .mul_m(&cam_inverse.to_matrix4());
        for ent in self.entities.iter_mut() {
            let model = &world.get_node(ent.node).world;
            let view = cam_inverse.concat(&model);
            ent.params.set_transform(&projection, &model.to_matrix4(), &view);
        }
    }

    pub fn draw<C: gfx::CommandBuffer>(&self,
                renderer: &mut gfx::Renderer<C>,
                frame: &gfx::Frame) {
        for ent in self.entities.iter() {
            let batch = (&ent.batch, &ent.params, &self.batch_context);
            renderer.draw(&batch, frame).unwrap();
        }
    }
}
