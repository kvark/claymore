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
    Program(String, super::program::Error),
    Texture(String, super::TextureError),
    SamplerFilter(String, u8),
    SamplerWrap(String, i8),
}

pub fn load<D: gfx::Device>(mat: &reflect::Material,
            context: &mut super::Context<D>)
            -> Result<Material, Error> {
    let program = match context.request_program(mat.shader.as_slice()) {
        Ok(p) => p.clone(),
        Err(e) => return Err(Error::Program(mat.shader.clone(), e)),
    };
    let state = gfx::DrawState::new().depth(
        gfx::state::Comparison::LessEqual,
        true
    );
    let mut data = super::program::Params {
        mvp: [[0.0; 4]; 4],
        normal: [[0.0; 3]; 3],
        color: [1.0, 1.0, 1.0, 1.0],
        texture: (context.texture_black, Some(context.sampler_point)),
    };
    match mat.textures.first() {
        Some(ref rt) => match context.request_texture(rt.path.as_slice()) {
            Ok(t) => {
                let sampler = context.device.create_sampler(gfx::tex::SamplerInfo::new(
                    match rt.filter {
                        1 => gfx::tex::FilterMethod::Scale,
                        2 => gfx::tex::FilterMethod::Bilinear,
                        3 => gfx::tex::FilterMethod::Trilinear,
                        other => return Err(Error::SamplerFilter(rt.name.clone(), other)),
                    },
                    match rt.wrap {
                        -1 => gfx::tex::WrapMode::Mirror,
                        0 => gfx::tex::WrapMode::Clamp,
                        1 => gfx::tex::WrapMode::Tile,
                        other => return Err(Error::SamplerWrap(rt.name.clone(), other)),
                    },
                ));
                data.texture = (t, Some(sampler));
            },
            Err(e) => return Err(Error::Texture(rt.path.clone(), e)),
        },
        None => (),
    };
    match mat.data.get("DiffuseColor") {
        Some(&(_, ref vec)) => {
            data.color = [vec[0], vec[1], vec[2], 1.0];
        },
        None => (),
    }
    Ok(Material {
        program: program,
        state: state,
        data: data,
    })
}
