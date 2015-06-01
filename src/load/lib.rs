#[macro_use]
extern crate log;
extern crate rustc_serialize;
extern crate cgmath;
extern crate gfx;
extern crate gfx_texture;
extern crate claymore_scene;

mod aux;
pub mod chunk;
mod mesh;
mod mat;
mod program;
mod reflect;
mod scene;

use std::collections::hash_map::{HashMap, Entry};
use std::io;
use std::fs::File;
use rustc_serialize::json;
use claymore_scene as cs;

pub use self::scene::Scalar;


pub static PREFIX_ATTRIB : &'static str = "a_";
pub static PREFIX_UNIFORM: &'static str = "u_";
pub static PREFIX_TEXTURE: &'static str = "t_";

pub type TextureError = String;

pub struct Cache<R: gfx::Resources> {
    meshes: HashMap<String, mesh::Success<R>>,
    textures: HashMap<String, Result<gfx::handle::Texture<R>, TextureError>>,
    programs: HashMap<String, Result<gfx::handle::Program<R>, program::Error>>,
}

impl<R: gfx::Resources> Cache<R> {
    pub fn new() -> Cache<R> {
        Cache {
            meshes: HashMap::new(),
            textures: HashMap::new(),
            programs: HashMap::new(),
        }
    }
}

pub struct Context<'a, R: 'a + gfx::Resources, F: 'a + gfx::Factory<R>> {
    pub cache: Cache<R>,
    pub factory: &'a mut F,
    pub base_path: String,
    prefix: String,
    pub alpha_test: Option<u8>,
    pub flip_textures: bool,
    pub forgive: bool,
}

impl<'a, R: gfx::Resources, F: gfx::Factory<R>> Context<'a, R, F> {
    pub fn new(factory: &'a mut F, path: String) -> Context<'a, R, F> {
        Context {
            cache: Cache::new(),
            factory: factory,
            base_path: path,
            prefix: String::new(),
            alpha_test: None,
            flip_textures: true,    // following Blender
            forgive: false,         // panic out
        }
    }

    fn read_mesh_collection(&mut self, path_str: &str) -> Result<(), mesh::Error> {
        info!("Loading mesh collection from {}", path_str);
        let path = format!("{}/{}.k3mesh", self.prefix, path_str);
        match File::open(path) {
            Ok(file) => {
                let size = file.metadata().unwrap().len() as u32;
                let mut reader = chunk::Root::new(path_str.to_string(), file);
                while reader.get_pos() < size {
                    debug_assert_eq!(reader.tell(), reader.get_pos());
                    debug!("Current position {}/{}", reader.get_pos(), size);
                    let (name, success) = try!(mesh::load(&mut reader, self.factory));
                    let full_name = format!("{}@{}", name, path_str);
                    self.cache.meshes.insert(full_name, success);
                }
                Ok(())
            },
            Err(e) => Err(mesh::Error::Path(e)),
        }
    }

    pub fn request_mesh(&mut self, path: &str)
                        -> Result<mesh::Success<R>, mesh::Error> {
        match self.cache.meshes.get(path) {
            Some(m) => return Ok(m.clone()),
            None => (),
        }
        let mut split = path.split('@');
        let _name = split.next().unwrap();
        match split.next() {
            Some(container) => {
                try!(self.read_mesh_collection(container));
                match self.cache.meshes.get(path) {
                    Some(m) => Ok(m.clone()),
                    None => Err(mesh::Error::NameNotInCollection),
                }
            },
            None => Err(mesh::Error::Other),
        }
    }

    pub fn request_texture(&mut self, path_str: &str, srgb: bool)
                           -> Result<gfx::handle::Texture<R>, TextureError> {
        match self.cache.textures.entry(path_str.to_string()) {
            Entry::Occupied(v) => v.get().clone(),
            Entry::Vacant(v) => {
                info!("Loading texture from {}", path_str);
                let path = format!("{}{}", self.prefix, path_str);
                let mut settings = gfx_texture::Settings::new();
                settings.flip_vertical = true;
                settings.convert_gamma = srgb;
                settings.generate_mipmap = true;
                let tex_result = gfx_texture::Texture::from_path(
                    self.factory, path, &settings);
                let tex = match tex_result {
                    Ok(t) => Ok(t.handle()),
                    Err(e) => {
                        if self.forgive {
                            error!("Texture failed to load: {:?}", e);
                        }
                        Err(e)
                    },
                };
                v.insert(tex).clone()
            },
        }
    }

    pub fn request_program(&mut self, name: &str)
                           -> Result<gfx::handle::Program<R>, program::Error> {
        match self.cache.programs.entry(name.to_string()) {
            Entry::Occupied(v) => v.get().clone(),
            Entry::Vacant(v) => {
                info!("Loading program {}", name);
                let prog_maybe = program::load(name, self.factory);
                v.insert(prog_maybe).clone()
            },
        }
    }
}

#[derive(Debug)]
pub enum SceneError {
    Open(io::Error),
    Read(io::Error),
    Decode(json::DecoderError),
    Parse(scene::Error),
}

impl<'a, R: gfx::Resources, F: gfx::Factory<R>> Context<'a, R, F> {
    pub fn load_scene_into(&mut self, scene: &mut cs::Scene<R, Scalar>,
                           global_parent: cs::Parent<Scalar>,
                           path_str: &str) -> Result<(), SceneError>
    {
        use std::io::Read;
        info!("Loading scene from {}", path_str);
        self.prefix = format!("{}/{}", self.base_path, path_str);
        let path = format!("{}.json", self.prefix);
        match File::open(&path) {
            Ok(mut file) => {
                let mut s = String::new();
                match file.read_to_string(&mut s) {
                    Ok(_) => match json::decode(&s) {
                        Ok(raw) => match scene::load_into(scene, global_parent, raw, self) {
                            Ok(s) => Ok(s),
                            Err(e) => Err(SceneError::Parse(e)),
                        },
                        Err(e) => Err(SceneError::Decode(e)),
                    },
                    Err(e) => Err(SceneError::Read(e)),
                }
            },
            Err(e) => Err(SceneError::Open(e)),
        }
    }

    pub fn load_scene(&mut self, path_str: &str)
                      -> Result<cs::Scene<R, Scalar>, SceneError>
    {
        let mut scene = cs::Scene::new();
        match self.load_scene_into(&mut scene, cs::space::Parent::None, path_str) {
            Ok(()) => Ok(scene),
            Err(e) => Err(e),
        }
    }

    pub fn extend_scene(&mut self, scene: &mut cs::Scene<R, Scalar>, path_str: &str)
                        -> Result<cs::NodeId<Scalar>, SceneError>
    {
        let nid = scene.world.add_node(
            path_str.to_string(),
            cs::space::Parent::None,
            cgmath::Transform::identity()
        );
        self.load_scene_into(scene, cs::space::Parent::Domestic(nid), path_str)
            .map(|_| nid)
    }
}

pub fn load_mesh<'a, R: gfx::Resources, F: gfx::Factory<R>>(path_str: &str, factory: &mut F)
                 -> Result<(String, mesh::Success<R>), mesh::Error> {
    info!("Loading mesh from {}", path_str);
    let path = format!("{}.k3mesh", path_str);
    match File::open(&path) {
        Ok(file) => {
            let mut reader = chunk::Root::new(path, file);
            mesh::load(&mut reader, factory)
        },
        Err(e) => Err(mesh::Error::Path(e)),
    }
}
