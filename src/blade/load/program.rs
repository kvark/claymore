use std::old_io as io;
use cgmath;
use gfx;
use super::scene::Scalar;

#[derive(Clone, Copy)]
#[shader_param]
pub struct Params {
    #[name = "u_Transform"]
    pub mvp: [[f32; 4]; 4],
    #[name = "u_NormalRotation"]
    pub normal: [[f32; 3]; 3],
    #[name = "u_Color"]
    pub color: [f32; 4],
    #[name = "t_Diffuse"]
    pub texture: gfx::shade::TextureParam,
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

#[derive(Clone, Debug)]
pub enum Error {
    Read(Path, io::IoError),
    Create(gfx::ProgramError),
}

pub fn load<D: gfx::Device>(name: &str, device: &mut D)
    -> Result<gfx::ProgramHandle, Error> {
    use gfx::DeviceExt;
    let src_vert = {
        let path = Path::new(format!("shader/{}.glslv", name));
        match io::File::open(&path).read_to_end() {
            Ok(c) => c,
            Err(e) => return Err(Error::Read(path, e)),
        }
    };
    let src_frag = {
        let path = Path::new(format!("shader/{}.glslf", name));
        match io::File::open(&path).read_to_end() {
            Ok(c) => c,
            Err(e) => return Err(Error::Read(path, e)),
        }
    };
    device.link_program(src_vert.as_slice(), src_frag.as_slice())
          .map_err(|e| Error::Create(e))
}
