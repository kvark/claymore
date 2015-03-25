use gfx;
use gfx_phase;

#[derive(Clone)]
#[shader_param]
pub struct Params<R: gfx::Resources> {
    #[name = "u_Transform"]
    pub mvp: [[f32; 4]; 4],
    #[name = "u_NormalRotation"]
    pub normal: [[f32; 3]; 3],
    #[name = "u_Color"]
    pub color: [f32; 4],
    #[name = "t_Diffuse"]
    pub texture: gfx::shade::TextureParam<R>,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum Shader {
    Phong,
}

#[derive(Clone)]
pub struct Material<R: gfx::Resources> {
    pub shader: Shader,
    pub color: [f32; 4],
    pub texture: gfx::shade::TextureParam<R>,
}

impl<R: gfx::Resources> gfx_phase::Material for Material<R> {}


pub struct Technique<R: gfx::Resources> {
    program: gfx::ProgramHandle<R>,
    state: gfx::DrawState,
    default_texture_param: gfx::shade::TextureParam<R>,
}

impl<R: gfx::Resources> Technique<R> {
    pub fn new(program: gfx::ProgramHandle<R>, tex_param: gfx::shade::TextureParam<R>)
               -> Technique<R> {
        let state = gfx::DrawState::new().depth(
            gfx::state::Comparison::LessEqual,
            true
        );
        Technique {
            program: program,
            state: state,
            default_texture_param: tex_param,
        }
    }
}

impl<R: gfx::Resources> gfx_phase::Technique<R, Material<R>, ::ViewInfo<f32>> for Technique<R> {
    type Kernel = Shader;
    type Params = Params<R>;

    fn test(&self, _mesh: &gfx::Mesh<R>, mat: &Material<R>) -> Option<Shader> {
        Some(mat.shader)
    }

    fn compile<'a>(&'a self, kernel: Shader, _space: ::ViewInfo<f32>)
                   -> gfx_phase::TechResult<'a, R, Params<R>> {
        (   match kernel {
                Shader::Phong => &self.program,
            },
            Params {
                mvp: [[0.0; 4]; 4],
                normal: [[0.0; 3]; 3],
                color: [0.0; 4],
                texture: self.default_texture_param.clone(),
            },
            None,
            &self.state,
        )
    }

    fn fix_params(&self, mat: &Material<R>, space: &::ViewInfo<f32>, params: &mut Params<R>) {
        use cgmath::FixedArray;
        params.mvp = *space.mx_vertex.as_fixed();
        params.normal = *space.mx_normal.as_fixed();
        params.color = mat.color;
        params.texture = mat.texture.clone();
    }
}
