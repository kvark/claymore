pub mod chunk;
mod mesh;
mod reflect;
mod scene;

use rustc_serialize::json;
use std::old_io as io;
use gfx;


#[derive(Debug)]
pub enum SceneError {
    Read(io::IoError),
    Decode(json::DecoderError),
    Parse(scene::Error),
}

pub fn scene<D: gfx::Device>(path_str: &str, device: &mut D)
             -> Result<scene::SceneJson, SceneError> {
    info!("Loading scene from {}", path_str);
    let path = Path::new(format!("{}.json", path_str).as_slice());
    match io::File::open(&path).read_to_string() {
        Ok(data) => match json::decode(data.as_slice()) {
            Ok(raw) => match scene::load(raw, device) {
                Ok(s) => Ok(s),
                Err(e) => Err(SceneError::Parse(e)),
            },
            Err(e) => Err(SceneError::Decode(e)),
        },
        Err(e) => Err(SceneError::Read(e)),
    }
}

pub fn mesh<D: gfx::Device>(path_str: &str, device: &mut D)
            -> Result<mesh::Success, mesh::Error> {
    info!("Loading mesh from {}", path_str);
    Err(mesh::Error::Path)
}
