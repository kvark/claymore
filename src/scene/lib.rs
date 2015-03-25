#[macro_use]
extern crate log;
extern crate id;
extern crate cgmath;
extern crate gfx;
extern crate gfx_phase;
extern crate gfx_scene;

pub mod space;


pub type Transform<S> = cgmath::Decomposed<
    S,
    cgmath::Vector3<S>,
    cgmath::Quaternion<S>
>;

pub struct ViewInfo<S> {
    pub mx_vertex: cgmath::Matrix4<S>,
    pub mx_normal: cgmath::Matrix3<S>,
}

impl<S: cgmath::BaseFloat> gfx_phase::ToDepth for ViewInfo<S> {
    type Depth = S;
    fn to_depth(&self) -> S {
        self.mx_vertex.w.z / self.mx_vertex.w.w
    }
}

impl<S: cgmath::BaseFloat + 'static> gfx_scene::ViewInfo<S, Transform<S>> for ViewInfo<S> {
    fn new(mvp: cgmath::Matrix4<S>, view: Transform<S>, _model: Transform<S>) -> ViewInfo<S> {
        use cgmath::ToMatrix3;
        ViewInfo {
            mx_vertex: mvp,
            mx_normal: view.rot.to_matrix3(),
        }
    }
}


pub type World<S> = ::space::World<S, Transform<S>>;
pub type Node<S> = ::space::Node<Transform<S>>;
pub type Skeleton<S> = ::space::Skeleton<Transform<S>>;
pub type Scene<R, M, S> = gfx_scene::Scene<
    R, M,
    World<S>,
    cgmath::Aabb3<S>,
    cgmath::PerspectiveFov<S, cgmath::Rad<S>>,
    ViewInfo<S>
>;
