pub mod chunk;
mod mesh;
mod mat;
mod program;
mod reflect;
mod scene;

use std::collections::HashMap;
use rustc_serialize::json;
use std::old_io as io;
use gfx;
use gfx_texture;


pub static PREFIX_ATTRIB : &'static str = "a_";
pub static PREFIX_UNIFORM: &'static str = "u_";
pub static PREFIX_TEXTURE: &'static str = "t_";

pub type TextureError = String;

pub struct Cache {
    meshes: HashMap<String, mesh::Success>,
    textures: HashMap<String, Result<gfx::TextureHandle, TextureError>>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            meshes: HashMap::new(),
            textures: HashMap::new(),
        }
    }
}

pub struct Context<'a, D: 'a> {
    pub cache: Cache,
    pub device: &'a mut D,
    pub prefix: String,
    pub texture_black: gfx::TextureHandle,
    pub shader_phong: gfx::ProgramHandle,
}

#[derive(Clone, Debug)]
pub enum ContextError {
    Texture(gfx::tex::TextureError),
    Program(gfx::ProgramError),
}

impl<'a, D: gfx::Device> Context<'a, D> {
    pub fn new(device: &'a mut D) -> Result<Context<'a, D>, ContextError> {
        use gfx::DeviceExt;
        let tinfo = gfx::tex::TextureInfo {
            width: 1,
            height: 1,
            depth: 1,
            levels: 1,
            format: gfx::tex::RGBA8,
            kind: gfx::tex::TextureKind::Texture2D,
        };
        let image_info = tinfo.to_image_info();
        let texture = match device.create_texture(tinfo) {
            Ok(t) => match device.update_texture(&t, &image_info, &[0u8, 0, 0, 0]) {
                Ok(()) => t,
                Err(e) => return Err(ContextError::Texture(e)),
            },
            Err(e) => return Err(ContextError::Texture(e)),
        };
        let program = match device.link_program(
            program::VERTEX_SRC, program::FRAGMENT_SRC) {
            Ok(p) => p,
            Err(e) => return Err(ContextError::Program(e)),
        };

        Ok(Context {
            cache: Cache::new(),
            device: device,
            prefix: String::new(),
            texture_black: texture,
            shader_phong: program,
        })
    }

    fn read_mesh_collection(&mut self, path_str: &str) -> Result<(), mesh::Error> {
        info!("Loading mesh collection from {}", path_str);
        let path = Path::new(format!("{}/{}.k3mesh", self.prefix, path_str).as_slice());
        match io::File::open(&path) {
            Ok(file) => {
                let size = file.stat().unwrap().size as u32;
                let mut reader = chunk::Root::new(path_str.to_string(), file);
                while reader.get_pos() < size {
                    let (name, success) = try!(mesh::load(&mut reader, self.device));
                    let full_name = format!("{}@{}", name, path_str);
                    self.cache.meshes.insert(full_name, success);
                }
                Ok(())
            },
            Err(e) => Err(mesh::Error::Path(e)),
        }
    }

    pub fn request_mesh(&mut self, path: &str)
                        -> Result<mesh::Success, mesh::Error> {
        match self.cache.meshes.get(path) {
            Some(m) => return Ok(m.clone()),
            None => (),
        }
        let mut split = path.split('@');
        split.next().unwrap();  //skip name
        match split.next() {
            Some(container) => {
                try!(self.read_mesh_collection(container));
                Ok(self.cache.meshes[path.to_string()].clone())
            },
            None => Err(mesh::Error::Other),
        }
    }

    pub fn request_texture(&mut self, path_str: &str)
                           -> Result<gfx::TextureHandle, TextureError> {
        match self.cache.textures.get(path_str) {
            Some(result) => return result.clone(),
            None => (),
        };
        info!("Loading texture from {}", path_str);
        let tex_maybe = gfx_texture::Texture::from_path(
            self.device, &Path::new(path_str))
            .map(|t| t.handle);
        self.cache.textures.insert(path_str.to_string(), tex_maybe.clone());
        tex_maybe
    }
}

#[derive(Debug)]
pub enum SceneError {
    Read(io::IoError),
    Decode(json::DecoderError),
    Parse(scene::Error),
}

pub fn scene<'a, D: gfx::Device>(path_str: &str, context: &mut Context<'a, D>)
             -> Result<scene::SceneJson, SceneError> {
    info!("Loading scene from {}", path_str);
    context.prefix = path_str.to_string();
    let path = Path::new(format!("{}.json", path_str).as_slice());
    match io::File::open(&path).read_to_string() {
        Ok(data) => match json::decode(data.as_slice()) {
            Ok(raw) => match scene::load(raw, context) {
                Ok(s) => Ok(s),
                Err(e) => Err(SceneError::Parse(e)),
            },
            Err(e) => Err(SceneError::Decode(e)),
        },
        Err(e) => Err(SceneError::Read(e)),
    }
}

pub fn mesh<D: gfx::Device>(path_str: &str, device: &mut D)
            -> Result<(String, mesh::Success), mesh::Error> {
    info!("Loading mesh from {}", path_str);
    let path = Path::new(format!("{}.k3mesh", path_str).as_slice());
    match io::File::open(&path) {
        Ok(file) => {
            let mut reader = chunk::Root::new(path_str.to_string(), file);
            mesh::load(&mut reader, device)
        },
        Err(e) => Err(mesh::Error::Path(e)),
    }
}
