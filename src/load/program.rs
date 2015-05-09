//use std::io;
use std::fs::File;
use gfx;
use gfx::traits::*;

#[derive(Clone, Debug)]
// https://github.com/rust-lang/rust/issues/24135
pub enum Error {
    Open(String, String),
    Read(String),
    Create(gfx::ProgramError),
}

pub fn load<R: gfx::Resources, F: gfx::Factory<R>>(name: &str, factory: &mut F)
            -> Result<gfx::handle::Program<R>, Error> {
    use std::io::Read;
    // vertex
    let mut src_vert = Vec::new();
    let path = format!("shader/{}.glslv", name);
    match File::open(&path) {
        Ok(mut file) => match file.read_to_end(&mut src_vert) {
            Ok(_) => (),
            Err(e) => return Err(Error::Read(e.to_string())),
        },
        Err(e) => return Err(Error::Open(path, e.to_string())),
    }
    // fragment
    let mut src_frag = Vec::new();
    let path = format!("shader/{}.glslf", name);
    match File::open(&path) {
        Ok(mut file) => match file.read_to_end(&mut src_frag) {
            Ok(_) => (),
            Err(e) => return Err(Error::Read(e.to_string())),
        },
        Err(e) => return Err(Error::Open(path, e.to_string())),
    }
    // program
    factory.link_program(&src_vert, &src_frag)
           .map_err(|e| Error::Create(e))
}
