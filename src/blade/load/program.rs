use cgmath;
use gfx::ShaderSource;
use super::scene::Scalar;

pub static VERTEX_SRC: ShaderSource<'static> = shaders! {
    glsl_150: b"#version 150 core

    in vec3 a_Position;
    //in vec2 a_TexCoord;
    //out vec2 v_TexCoord;

    uniform mat4 u_Transform;

    void main() {
        //v_TexCoord = a_TexCoord;
        gl_Position = u_Transform * vec4(a_Position, 1.0);
    }
    "
};

pub static FRAGMENT_SRC: ShaderSource<'static> = shaders! {
    glsl_150: b"#version 150 core

    //in vec2 v_TexCoord;
    out vec4 o_Color;

    uniform vec4 u_Color;

    void main() {
        o_Color = u_Color;
    }
    "
};

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
