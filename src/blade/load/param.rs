use cgmath;
use super::scene::Scalar;

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
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl ::scene::ShaderParam<Scalar> for Params {
    fn set_transform(&mut self,
                     camera: &cgmath::Matrix4<Scalar>,
                     model: &cgmath::Matrix4<Scalar>,
                     _view: &::scene::Transform<Scalar>) {
        use cgmath::{Matrix, FixedArray};
        self.mvp = camera.mul_m(model).into_fixed();
    }
}
