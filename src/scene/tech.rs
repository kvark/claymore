use cgmath;
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

#[derive(Clone)]
pub struct Material<R: gfx::Resources> {
    pub program: gfx::ProgramHandle<R>,
    pub state: gfx::DrawState,
    pub data: Params<R>,
}

impl<R: gfx::Resources> gfx_phase::Material for Material<R> {}

pub struct Technique;

impl<S: cgmath::BaseFloat, R: gfx::Resources>
gfx_phase::Technique<R, Material<R>, ::ViewInfo<S>> for Technique {
    type Kernel = (); //TODO
    type Params = Params<R>;

    fn test(&self, _mesh: &gfx::Mesh<R>, _mat: &Material<R>) -> Option<()> {
        Some(())
    }

    fn compile<'a>(&'a self, _kernel: (), _space: ::ViewInfo<S>)
                   -> gfx_phase::TechResult<'a, R, Params<R>> {
        (   &mat.program,
            mat.params,
            None,
            &mat.state,
        )
    }

    fn fix_params(&self, _mat: &Material, space: &::ViewInfo<S>, params: &mut Params<R>) {
        params.mvp = space.mx_vertex.to_fixed();
        params.normal = space.mx_normal.to_fixed();
    }
}
