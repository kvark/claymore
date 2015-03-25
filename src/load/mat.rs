use gfx;
//use gfx_texture; //TODO
use claymore_scene::tech::{Material, Shader};
use super::reflect;

#[derive(Debug)]
pub enum Error {
    NotFound,
    Program(String),
    Texture(String, super::TextureError),
    SamplerFilter(String, u8),
    SamplerWrap(i8),
}

pub fn load<R: gfx::Resources, F: gfx::Factory<R>>(mat: &reflect::Material,
            context: &mut super::Context<R, F>) -> Result<Material<R>, Error> {
    let mut out = Material {
        shader: match mat.shader.as_slice() {
            "PHONG" => Shader::Phong,
            _ => return Err(Error::Program(mat.shader.clone())),
        },
        color: [1.0, 1.0, 1.0, 1.0],
        texture: (context.texture_black.clone(), Some(context.sampler_point.clone())),
    };
    match mat.textures.first() {
        Some(ref rt) => match context.request_texture(rt.path.as_slice()) {
            Ok(t) => {
                fn unwrap(mode: i8) -> Result<gfx::tex::WrapMode, Error> {
                    match mode {
                        -1 => Ok(gfx::tex::WrapMode::Mirror),
                        0 => Ok(gfx::tex::WrapMode::Clamp),
                        1 => Ok(gfx::tex::WrapMode::Tile),
                        _ => Err(Error::SamplerWrap(mode)),
                    }
                }
                let (wx, wy, wz) = (
                    try!(unwrap(rt.wrap.0)),
                    try!(unwrap(rt.wrap.1)),
                    try!(unwrap(rt.wrap.2)),
                );
                let filter = match rt.filter {
                    1 => gfx::tex::FilterMethod::Scale,
                    2 => gfx::tex::FilterMethod::Bilinear,
                    3 => gfx::tex::FilterMethod::Trilinear,
                    other => return Err(Error::SamplerFilter(rt.name.clone(), other)),
                };
                let mut sinfo = gfx::tex::SamplerInfo::new(filter, wx);
                sinfo.wrap_mode.1 = wy;
                sinfo.wrap_mode.2 = wz;
                let sampler = context.factory.create_sampler(sinfo);
                out.texture = (t, Some(sampler));
            },
            Err(e) => return Err(Error::Texture(rt.path.clone(), e)),
        },
        None => (),
    };
    match mat.data.get("DiffuseColor") {
        Some(&(_, ref vec)) => {
            out.color = [vec[0], vec[1], vec[2], 1.0];
        },
        None => (),
    }
    Ok(out)
}
