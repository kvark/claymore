use gfx;
use claymore_scene::{Material, Transparency};
use super::reflect;

#[derive(Debug)]
pub enum Error {
    NotFound,
    //Program(String),
    Texture(String, super::TextureError),
    SamplerFilter(String, u8),
    SamplerWrap(i8),
}

pub fn load<R: gfx::Resources, F: gfx::Factory<R>>(mat: &reflect::Material,
            context: &mut super::Context<R, F>) -> Result<Material<R>, Error> {
    let mut out = Material {
        color: [1.0, 1.0, 1.0, 1.0],
        texture: None,
        transparency: match (mat.transparent, context.alpha_test) {
            (true, Some(v)) => Transparency::Cutout(v),
            (true, None)    => Transparency::Blend(gfx::BlendPreset::Alpha),
            (false, _)      => Transparency::Opaque,
        },
    };
    if let Some(ref rt) = mat.textures.first() {
        let space = match rt.image.space.as_ref() {
            "Linear" => false,
            "sRGB" => true,
            other => {
                warn!("Unknown color space: {}", other);
                false
            }
        };
        match context.request_texture(&rt.image.path, space) {
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
                out.texture = Some((t, Some(sampler)));
            },
            Err(_) if context.forgive => (), //already errored in request_texture()
            Err(e) => return Err(Error::Texture(rt.image.path.clone(), e)),
        }
    };
    if let Some(&(_, ref vec)) = mat.data.get("DiffuseColor") {
        out.color = [vec[0], vec[1], vec[2], 1.0];
    }
    Ok(out)
}
