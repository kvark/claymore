pub mod chunk;
mod mesh;
mod program;
mod reflect;
mod scene;

use std::collections::HashMap;
use rustc_serialize::json;
use std::old_io as io;
use gfx;
use image;


pub static PREFIX_ATTRIB : &'static str = "a_";
pub static PREFIX_UNIFORM: &'static str = "u_";
pub static PREFIX_TEXTURE: &'static str = "t_";

#[derive(Debug)]
pub enum TextureError {
    Image(image::ImageError),
    Format,
    Texture(gfx::tex::TextureError),
    Upload(gfx::tex::TextureError),
}

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
}

impl<'a, D: gfx::Device> Context<'a, D> {
    pub fn new(device: &'a mut D) -> Context<'a, D> {
        Context {
            cache: Cache::new(),
            device: device,
            prefix: String::new(),
        }
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

    fn upload_texture<P: image::Pixel + 'static>(&mut self,
                      buffer: image::ImageBuffer<P, Vec<u8>>,
                      components: gfx::tex::Components)
                      -> Result<gfx::TextureHandle, TextureError>
                      where P::Subpixel: 'static {
        let tex_info = gfx::tex::TextureInfo {
            width: buffer.width() as u16,
            height: buffer.height() as u16,
            depth: 1,
            levels: 99,
            kind: gfx::tex::TextureKind::Texture2D,
            format: gfx::tex::Format::Unsigned(components, 8,
                gfx::attrib::IntSubType::Normalized),
        };
        let image_info = tex_info.to_image_info();
        match self.device.create_texture(tex_info) {
            Ok(handle) => {
                match self.device.update_texture_raw(&handle, &image_info, buffer.as_slice()) {
                    Ok(()) => {
                        self.device.generate_mipmap(&handle);
                        Ok(handle)
                    },
                    Err(e) => Err(TextureError::Upload(e)),
                }
            },
            Err(e) => Err(TextureError::Texture(e)),
        }
    }

    pub fn request_texture(&mut self, path_str: &str)
                           -> Result<gfx::TextureHandle, TextureError> {
        match self.cache.textures.get(path_str) {
            Some(result) => return *result,
            None => (),
        };
        info!("Loading texture from {}", path_str);
        let tex_maybe = match image::open(&Path::new(path_str)) {
            Ok(image::ImageLuma8(img)) =>
                self.upload_texture(img, gfx::tex::Components::R),
            //Ok(image::ImageLumaA8(ref img)) => {},
            Ok(image::ImageRgb8(img)) =>
                self.upload_texture(img, gfx::tex::Components::RGB),
            Ok(image::ImageRgba8(img)) =>
                self.upload_texture(img, gfx::tex::Components::RGBA),
            Ok(_) => Err(TextureError::Format),
            Err(e) => Err(TextureError::Image(e)),
        };
        self.cache.textures.insert(path_str.to_string(), tex_maybe);
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
