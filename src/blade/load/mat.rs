use gfx;
use image;
use super::reflect;

pub struct Material {
    pub color: [f32; 4],
    pub texture: gfx::TextureHandle,
}

//pub fn load<D: gfx::Device>(mat: &reflect::Material, device: &mut D) -> Result<
