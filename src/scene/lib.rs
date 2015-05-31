#[macro_use]
extern crate log;
extern crate id;
extern crate cgmath;
extern crate gfx;
extern crate gfx_phase;
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
pub type Fragment<R> = gfx_scene::Fragment<R, Material<R>>;

pub struct Pair<A, B>(A, B);

/// A simple camera with generic projection and spatial relation.
#[derive(Clone)]
pub struct Camera<S> {
    /// Name of the camera.
    pub name: String,
    /// Generic projection.
    pub projection: Projection<S>,
    /// Generic spatial node.
    pub node: NodeId<S>,
}

impl<'a, S: cgmath::BaseFloat> gfx_scene::Node for Pair<&'a Camera<S>, &'a World<S>> {
    type Transform = Transform<S>;
    fn get_transform(&self) -> Transform<S> {
        self.1.get_node(self.0.node).world.clone()
    }
}

impl<'a, S: cgmath::BaseFloat> gfx_scene::Camera<S> for Pair<&'a Camera<S>, &'a World<S>> {
    type Projection = Projection<S>;
    fn get_projection(&self) -> Projection<S> { self.0.projection.clone() }
}

#[derive(Clone)]
pub struct Entity<R: gfx::Resources, S: cgmath::BaseNum> {
    pub name: String,
    pub visible: bool,
    pub mesh: gfx::Mesh<R>,
    pub node: NodeId<S>,
    pub skeleton: Option<id::Id<Skeleton<S>>>,
	pub bound: cgmath::Aabb3<S>,
    pub fragments: Vec<Fragment<R>>,
}

impl<R: gfx::Resources, S: cgmath::BaseNum> Entity<R, S> {
    /// Create a minimal new `Entity`.
    pub fn new(mesh: gfx::Mesh<R>, node: NodeId<S>, bound: cgmath::Aabb3<S>)
               -> Entity<R, S>
    {
        Entity {
            name: String::new(),
            visible: true,
            mesh: mesh,
            node: node,
            skeleton: None,
            bound: bound,
            fragments: Vec::new(),
        }
    }
}

impl<'a,
    R: gfx::Resources,
    S: cgmath::BaseFloat,
> gfx_scene::Node for Pair<&'a Entity<R, S>, &'a World<S>> {
    type Transform = Transform<S>;
    fn get_transform(&self) -> Transform<S> {
        self.1.get_node(self.0.node).world.clone()
    }
}

impl<'a,
    R: gfx::Resources,
    S: cgmath::BaseFloat,
> gfx_scene::Entity<R, Material<R>> for Pair<&'a Entity<R, S>, &'a World<S>> {
    type Bound = cgmath::Aabb3<S>;
    fn is_visible(&self) -> bool { self.0.visible }
    fn get_bound(&self) -> cgmath::Aabb3<S> { self.0.bound.clone() }
    fn get_mesh(&self) -> &gfx::Mesh<R> { &self.0.mesh }
    fn get_fragments(&self) -> &[Fragment<R>] { &self.0.fragments }
}

/// An example scene type.
pub struct Scene<R: gfx::Resources, S: cgmath::BaseNum> {
    pub entities: Vec<Entity<R, S>>,
    pub world: World<S>,
}

impl<
    R: gfx::Resources,
    S: cgmath::BaseFloat,
> gfx_scene::AbstractScene<R> for Scene<R, S> {
    type ViewInfo = ViewInfo<S>;
    type Material = Material<R>;
    type Camera = Camera<S>;
    type Status = gfx_scene::Report;

    fn draw<H, X>(&self, phase: &mut H, camera: &Camera<S>,
            stream: &mut X) -> Result<gfx_scene::Report, gfx_scene::Error> where
        H: gfx_phase::AbstractPhase<R, Material<R>, ViewInfo<S>>,
        X: gfx::Stream<R>,
    {
        let mut culler = gfx_scene::Frustum::new();
        gfx_scene::Context::new(&mut culler, &Pair(camera, &self.world))
                           .draw(self.entities.iter().map(|e| &Pair(e, &self.world)),
                                 phase, stream)
    }
}
