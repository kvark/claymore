use cgmath;
use super::scene::Scalar;

pub static VERTEX_SRC: &'static [u8] = b"
    #version 150 core

    uniform mat4 u_Transform;
    uniform mat3 u_NormalRotation;

    in vec3 a_Position;
    in vec3 a_Normal;

    out vec3 v_Normal;

    void main() {
        gl_Position = u_Transform * vec4(a_Position, 1.0);
        v_Normal = u_NormalRotation * a_Normal;
    }
";

pub static FRAGMENT_SRC: &'static [u8] = b"
    #version 150 core

    const vec3 c_LightPos = vec3(10.0, 10.0, 10.0); //view space
    uniform vec4 u_Color;

    in vec3 v_Normal;
    out vec4 o_Color;

    void main() {
        vec3 N = normalize(v_Normal);
        vec3 L = normalize(c_LightPos);
        o_Color = u_Color * dot(N, L);
    }
";

#[derive(Clone, Copy)]
#[shader_param]
pub struct Params {
    #[name = "u_Transform"]
    pub mvp: [[f32; 4]; 4],
    #[name = "u_NormalRotation"]
    pub normal: [[f32; 3]; 3],
    #[name = "u_Color"]
    pub color: [f32; 4],
}

impl Params {
    pub fn new() -> Params {
        Params {
            mvp: [[0.0; 4]; 4],
            normal: [[0.0; 3]; 3],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

impl ::scene::ShaderParam<Scalar> for Params {
    fn set_transform(&mut self,
                     camera: &cgmath::Matrix4<Scalar>,
                     model: &cgmath::Matrix4<Scalar>,
                     view: &::scene::Transform<Scalar>) {
        use cgmath::{Matrix, ToMatrix3, FixedArray};
        self.mvp = camera.mul_m(model).into_fixed();
        self.normal = view.rot.to_matrix3().into_fixed();
    }
}
