use gfx;
//use gfx_texture;
use super::reflect;

#[derive(Clone)]
pub struct Material {
    pub program: gfx::ProgramHandle,
    pub state: gfx::DrawState,
    pub data: super::program::Params,
}

#[derive(Debug)]
pub enum Error {
    NotFound,
    Shader(String),
    //Texture(String, super::TextureError),
}

pub fn load<D: gfx::Device>(mat: &reflect::Material,
            context: &mut super::Context<D>)
            -> Result<Material, Error> {
    let program = match mat.shader.as_slice() {
        "phong" => context.shader_phong.clone(),
        name => return Err(Error::Shader(name.to_string())),
    };
    let state = gfx::DrawState::new().depth(
        gfx::state::Comparison::LessEqual,
        true
    );
    let data = super::program::Params::new();
    Ok(Material {
        program: program,
        state: state,
        data: data,
    })
}
