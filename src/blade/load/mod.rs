pub mod chunk;
mod mesh;
mod reflect;
mod scene;

use std::collections::HashMap;
use rustc_serialize::json;
use std::old_io as io;
use gfx;


pub static PREFIX_ATTRIB : &'static str = "a_";
pub static PREFIX_UNIFORM: &'static str = "u_";

pub struct Cache {
    meshes: HashMap<String, mesh::Success>,
}

impl Cache {
    pub fn new() -> Cache {
        Cache {
            meshes: HashMap::new(),
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
