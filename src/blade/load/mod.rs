mod mesh;
mod reflect;
mod scene;

use rustc_serialize::json;
use std::old_io as io;
use gfx::Device;

#[derive(Debug)]
pub enum Error {
    Read(io::IoError),
    Decode(json::DecoderError),
    Parse(scene::Error),
}

pub fn scene<D: Device>(path_str: &str, device: &mut D)
        -> Result<scene::SceneJson, Error> {
    let path = Path::new(format!("{}.json", path_str).as_slice());
    match io::File::open(&path).read_to_string() {
        Ok(data) => match json::decode(data.as_slice()) {
            Ok(raw) => match scene::load(raw, device) {
                Ok(s) => Ok(s),
                Err(e) => Err(Error::Parse(e)),
            },
            Err(e) => Err(Error::Decode(e)),
        },
        Err(e) => Err(Error::Read(e)),
    }
}
