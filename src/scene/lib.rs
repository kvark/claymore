#[macro_use]
extern crate log;
extern crate id;
extern crate cgmath;
extern crate gfx_scene;
extern crate gfx_pipeline;

pub mod space;
pub use gfx_pipeline::{Material, Transparency, ViewInfo};
pub use gfx_scene as base;


pub type Transform<S> = cgmath::Decomposed<
    S,
    cgmath::Vector3<S>,
    cgmath::Quaternion<S>
>;

pub type Parent<S> = space::Parent<Transform<S>>;
pub type World<S> = space::World<S, Transform<S>>;
pub type Node<S> = space::Node<Transform<S>>;
pub type NodeId<S> = id::Id<space::Node<Transform<S>>>;
pub type Skeleton<S> = space::Skeleton<Transform<S>>;
pub type Projection<S> = cgmath::PerspectiveFov<S, cgmath::Rad<S>>;
pub type Camera<S> = gfx_scene::Camera<Projection<S>, NodeId<S>>;

pub type Fragment<R> = gfx_scene::Fragment<R,
    gfx_pipeline::Material<R>
>;

pub type Entity<R, S> = gfx_scene::Entity<R,
	gfx_pipeline::Material<R>,
	World<S>,
	cgmath::Aabb3<S>,
>;

pub type Scene<R, S> = gfx_scene::Scene<R,
    gfx_pipeline::Material<R>,
    World<S>,
    cgmath::Aabb3<S>,
    Projection<S>,
    gfx_pipeline::ViewInfo<S>,
>;
